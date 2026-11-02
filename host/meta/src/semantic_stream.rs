//! Sequential trusted-host approvals delivered to one retained guest session.
//! Provider access and the independent intent gate live outside this adapter.
use crate::{batch_bridge, semantic};
use serde::Deserialize;
use serde_json::json;
use std::io::{self, BufRead, Write};
const MAX_EVENTS: u32 = 4;
const MAX_LINE: usize = 8192;
#[derive(Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
enum Input {
    Approved {
        request: semantic::Request,
        selection: semantic::Selection,
    },
    Rejected {
        request: semantic::Request,
    },
    Stop {},
}
fn line(reader: &mut impl BufRead) -> Result<Input, String> {
    // Bound allocation while reading, rather than reading an unbounded line and
    // checking afterward. EOF without an explicit stop is a failed session.
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(|_| "stream_read_failed")?;
        if available.is_empty() {
            return Err("unexpected_stream_eof".into());
        }
        let size = available
            .iter()
            .position(|b| *b == b'\n')
            .map_or(available.len(), |n| n + 1);
        if bytes.len() + size > MAX_LINE {
            return Err("stream_line_too_large".into());
        }
        let complete = available[size - 1] == b'\n';
        bytes.extend_from_slice(&available[..size]);
        reader.consume(size);
        if complete {
            return serde_json::from_slice(&bytes).map_err(|_| "invalid_stream_input".into());
        }
    }
}
fn advance(request: &semantic::Request, previous: &mut u32, count: &mut u32) -> Result<(), String> {
    crate::domain::validate_goal(&request.goal)?;
    if request.request_version <= *previous || *count == MAX_EVENTS {
        return Err("stale_or_exhausted_stream".into());
    }
    *previous = request.request_version;
    *count += 1;
    Ok(())
}
pub fn run(kernel: &str, graphics: bool) -> Result<(), String> {
    println!(
        "{}",
        json!({"event":"selection_stream_started","source":"streamed_host_approvals","intent_gate":"trusted_external_checker","graphics":graphics,"pause_ms":if graphics{1500}else{0},"max_events":MAX_EVENTS})
    );
    let input = io::stdin();
    let mut reader = input.lock();
    let (mut previous, mut count) = (0, 0);
    batch_bridge::run_updates(kernel, graphics, true, || {
        loop {
            println!(
                "{}",
                json!({"event":"stream_ready","last_request_version":previous,"events_received":count})
            );
            io::stdout().flush().map_err(|_| "stream_flush_failed")?;
            match line(&mut reader)? {
                Input::Stop {} => return Ok(None),
                Input::Rejected { request } => {
                    advance(&request, &mut previous, &mut count)?;
                    println!(
                        "{}",
                        json!({"event":"stream_rejected","request":request,"guest_work":false})
                    );
                }
                Input::Approved { request, selection } => {
                    advance(&request, &mut previous, &mut count)?;
                    let keys = semantic::validate(&selection, &request)?;
                    println!(
                        "{}",
                        json!({"event":"stream_approved","request":request,"selection":selection,"intent_verified_by_this_command":false})
                    );
                    if let Some(keys) = keys {
                        return Ok(Some(keys));
                    }
                    println!(
                        "{}",
                        json!({"event":"stream_no_execution","request_version":previous,"decision":selection.decision,"reason":selection.reason})
                    );
                }
            }
        }
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_is_bounded_explicit_and_strict() {
        assert!(line(&mut &b"{\"event\":\"stop\"}\n"[..]).is_ok());
        assert!(line(&mut &b"{\"event\":\"stop\",\"shell\":\"x\"}\n"[..]).is_err());
        assert!(line(&mut &b"{\"event\":\"stop\"}"[..]).is_err());
        assert!(line(&mut &vec![b' '; MAX_LINE + 1][..]).is_err());
        let request = semantic::Request {
            request_version: 1,
            goal: "A goal".into(),
        };
        let (mut previous, mut count) = (0, 0);
        advance(&request, &mut previous, &mut count).unwrap();
        assert!(advance(&request, &mut previous, &mut count).is_err());
        previous = 0;
        count = MAX_EVENTS;
        assert!(advance(&request, &mut previous, &mut count).is_err());
    }
}
