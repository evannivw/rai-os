//! Session coordinator. Only this thread owns QEMU and performs blocking UART I/O.
use crate::{model, qemu};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub enum Command {
    Start,
    Stop,
    Request(String),
}

#[derive(Debug)]
pub enum Event {
    Started(String),
    Uptime { milliseconds: u64, frequency: u32 },
    Reply { command: String, payload: String },
    Rejected(String),
    Stopped,
    Failed(String),
}

pub struct Worker {
    commands: mpsc::SyncSender<Command>,
    events: Option<mpsc::Receiver<Event>>,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(kernel: String) -> Self {
        let (commands, incoming) = mpsc::sync_channel(4);
        let (outgoing, events) = mpsc::sync_channel(32);
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = shutdown.clone();
        let thread = thread::spawn(move || run(kernel, incoming, outgoing, stop));
        Self {
            commands,
            events: Some(events),
            shutdown,
            thread: Some(thread),
        }
    }

    pub fn send(&self, command: Command) -> Result<(), String> {
        self.commands
            .try_send(command)
            .map_err(|e| format!("Session busy or closed: {e}"))
    }

    pub fn poll(&self) -> Result<Event, mpsc::TryRecvError> {
        self.events
            .as_ref()
            .expect("live event receiver")
            .try_recv()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        // Release the receiver before joining, so a full event queue cannot
        // strand the worker. Outstanding transport waits time out within 10 s.
        self.events.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub fn parse_uptime(payload: &str) -> Result<(u64, u32), String> {
    let (ms, hz) = payload
        .split_once("; counter_hz=")
        .ok_or("Invalid uptime fields")?;
    let ms = ms.strip_prefix("uptime_ms=").ok_or("Missing uptime")?;
    if ms.is_empty() || hz.is_empty() || !ms.bytes().chain(hz.bytes()).all(|b| b.is_ascii_digit()) {
        return Err("Invalid uptime digits".into());
    }
    let ms = ms.parse().map_err(|_| "Uptime overflow")?;
    let hz = hz.parse().map_err(|_| "Counter frequency overflow")?;
    if hz == 0 {
        return Err("Zero counter frequency".into());
    }
    Ok((ms, hz))
}

fn request(
    connection: &mut qemu::Connection,
    id: &mut u32,
    op: &model::Operation<'_>,
) -> Result<String, String> {
    *id = id.checked_add(1).ok_or("Request IDs exhausted")?;
    let reply = connection.request(*id, op)?;
    if reply.success {
        Ok(reply.payload)
    } else {
        Err(format!("Kernel rejected request: {}", reply.payload))
    }
}

fn run(
    kernel: String,
    incoming: mpsc::Receiver<Command>,
    outgoing: mpsc::SyncSender<Event>,
    shutdown: Arc<AtomicBool>,
) {
    let mut connection = None;
    let mut id = 0;
    let mut last_sample = None;
    let mut next_poll = Instant::now();
    while !shutdown.load(Ordering::Relaxed) {
        let command = match incoming.recv_timeout(Duration::from_millis(100)) {
            Ok(command) => Some(command),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        let result = (|| -> Result<(), String> {
            match command {
                Some(Command::Start) if connection.is_none() => {
                    let mut new_connection = qemu::Connection::start(&kernel)?;
                    if shutdown.load(Ordering::Relaxed) {
                        return Err("View closed".into());
                    }
                    id = 0;
                    let info = request(&mut new_connection, &mut id, &model::Operation::Info)?;
                    connection = Some(new_connection);
                    last_sample = None;
                    next_poll = Instant::now();
                    outgoing
                        .send(Event::Started(info))
                        .map_err(|_| "View closed")?;
                }
                Some(Command::Stop) => {
                    connection.take(); // Drop terminates and reaps QEMU before reporting stopped.
                    last_sample = None;
                    outgoing.send(Event::Stopped).map_err(|_| "View closed")?;
                }
                Some(Command::Request(text)) => {
                    let operation = model::parse_input(&text).and_then(|op| {
                        model::encode(1, &op)?;
                        Ok(op)
                    });
                    match (operation, connection.as_mut()) {
                        (Ok(op), Some(current)) => {
                            let payload = request(current, &mut id, &op)?;
                            outgoing
                                .send(Event::Reply {
                                    command: text,
                                    payload,
                                })
                                .map_err(|_| "View closed")?;
                        }
                        (Err(error), _) => {
                            outgoing
                                .send(Event::Rejected(error.into()))
                                .map_err(|_| "View closed")?;
                        }
                        (_, None) => {
                            outgoing
                                .send(Event::Rejected("Start the kernel first".into()))
                                .map_err(|_| "View closed")?;
                        }
                    }
                }
                _ => {}
            }
            if let Some(current) = connection.as_mut()
                && Instant::now() >= next_poll
                && !shutdown.load(Ordering::Relaxed)
            {
                let payload = request(current, &mut id, &model::Operation::Uptime)?;
                let (milliseconds, frequency) = parse_uptime(&payload)?;
                if let Some((prior, hz)) = last_sample
                    && (milliseconds < prior || frequency != hz)
                {
                    return Err("Guest clock changed unexpectedly".into());
                }
                last_sample = Some((milliseconds, frequency));
                outgoing
                    .send(Event::Uptime {
                        milliseconds,
                        frequency,
                    })
                    .map_err(|_| "View closed")?;
                next_poll = Instant::now() + Duration::from_secs(1);
            }
            Ok(())
        })();
        if let Err(error) = result {
            connection.take();
            if outgoing.send(Event::Failed(error)).is_err() {
                break;
            }
        }
    }
    // Dropping the owned connection cleans up the child on normal window close.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_measured_uptime() {
        assert_eq!(
            parse_uptime("uptime_ms=123; counter_hz=62500000").unwrap(),
            (123, 62500000)
        );
        for text in [
            "uptime_ms=1; counter_hz=0",
            "uptime_ms=-1; counter_hz=1",
            "uptime_ms=1; counter_hz=1; extra=2",
            "uptime_ms=18446744073709551616; counter_hz=1",
        ] {
            assert!(parse_uptime(text).is_err(), "{text}");
        }
    }

    fn wait(worker: &Worker, predicate: impl Fn(&Event) -> bool) -> Event {
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            match worker.poll() {
                Ok(event) if predicate(&event) => return event,
                Ok(Event::Failed(error)) => panic!("Unexpected session failure: {error}"),
                Ok(_) | Err(mpsc::TryRecvError::Empty) => thread::sleep(Duration::from_millis(10)),
                Err(error) => panic!("Worker disconnected: {error}"),
            }
        }
        panic!("Timed out waiting for worker event");
    }

    #[test]
    #[ignore = "Requires real QEMU, built kernel, and repository-root working directory"]
    fn real_qemu_lifecycle() {
        let worker = Worker::new("target/aarch64-unknown-none-softfloat/debug/rai-os".into());
        worker.send(Command::Start).unwrap();
        wait(
            &worker,
            |e| matches!(e, Event::Started(info) if info.contains("RAI-OS")),
        );
        let first = wait(&worker, |e| matches!(e, Event::Uptime { .. }));
        let second = wait(&worker, |e| matches!(e, Event::Uptime { .. }));
        match (first, second) {
            (
                Event::Uptime {
                    milliseconds: a, ..
                },
                Event::Uptime {
                    milliseconds: b, ..
                },
            ) => assert!(b > a),
            _ => unreachable!(),
        }
        worker
            .send(Command::Request("echo invalid\targument".into()))
            .unwrap();
        wait(&worker, |e| matches!(e, Event::Rejected(_)));
        worker
            .send(Command::Request("echo panel connected".into()))
            .unwrap();
        wait(
            &worker,
            |e| matches!(e, Event::Reply { payload, .. } if payload == "panel connected"),
        );
        worker.send(Command::Stop).unwrap();
        wait(&worker, |e| matches!(e, Event::Stopped));
        worker.send(Command::Start).unwrap();
        wait(&worker, |e| matches!(e, Event::Started(_)));
        wait(&worker, |e| matches!(e, Event::Uptime { .. }));
        // Drop exercises closing a running session, rather than only Stop.
        drop(worker);
    }

    #[test]
    #[ignore = "Requires repository-root working directory and QEMU"]
    fn missing_kernel_never_reports_connected() {
        let worker = Worker::new("/nonexistent/rai-panel-kernel".into());
        worker.send(Command::Start).unwrap();
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            assert!(Instant::now() < deadline);
            match worker.poll() {
                Ok(Event::Failed(_)) => break,
                Ok(event) => panic!("Unexpected event before failure: {event:?}"),
                Err(mpsc::TryRecvError::Empty) => thread::sleep(Duration::from_millis(20)),
                Err(error) => panic!("{error}"),
            }
        }
    }
}
