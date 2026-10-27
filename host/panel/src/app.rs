//! Native host presentation. Values change only when the session worker reports them.
use crate::coordinator::{CAPACITY, Coordinator, JobState};
use crate::task_status::{State as TaskState, TaskStatus};
use crate::worker::{Command, Event, Worker};
use eframe::egui::{self, Color32, FontId, RichText, Stroke};
use std::collections::VecDeque;
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

const BG: Color32 = Color32::from_rgb(16, 19, 23);
const CARD: Color32 = Color32::from_rgb(25, 30, 35);
const MUTED: Color32 = Color32::from_rgb(156, 167, 175);
const ACCENT: Color32 = Color32::from_rgb(143, 215, 190);
const BORDER: Color32 = Color32::from_rgb(49, 58, 64);

#[derive(PartialEq)]
enum Phase {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

pub struct Panel {
    worker: Worker,
    phase: Phase,
    uptime: Option<u64>,
    sampled: Option<Instant>,
    input: String,
    task: Option<TaskStatus>,
    task_text: String,
    task_view: bool,
    queue_view: bool,
    queue: Coordinator,
    echo: Option<String>,
    error: Option<String>,
    pending: bool,
    show_activity: bool,
    log: VecDeque<(String, String)>,
    opened: Instant,
}

impl Panel {
    pub fn new(cc: &eframe::CreationContext<'_>, kernel: String) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = BG;
        style.visuals.override_text_color = Some(Color32::from_rgb(231, 237, 239));
        style.visuals.widgets.inactive.bg_fill = CARD;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(43, 58, 58);
        style.visuals.selection.bg_fill = Color32::from_rgb(49, 90, 78);
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.spacing.button_padding = egui::vec2(16.0, 10.0);
        cc.egui_ctx.set_style(style);
        let mut panel = Self {
            worker: Worker::new(kernel),
            phase: Phase::Stopped,
            uptime: None,
            sampled: None,
            input: "Hello from Device Lab".into(),
            task: None,
            task_text: "RAI-OS processes this text in small steps. The kernel keeps answering commands while it counts words. You can cancel the task or wait for the result.".into(),
            task_view: true,
            queue_view: true,
            queue: Coordinator::default(),
            echo: None,
            error: None,
            pending: false,
            show_activity: true,
            log: VecDeque::new(),
            opened: Instant::now(),
        };
        panel.record("READY", "Start a session to connect to the kernel.");
        panel
    }

    fn record(&mut self, kind: &str, text: &str) {
        if self.log.len() == 80 {
            self.log.pop_front();
        }
        self.log.push_back((
            format!(
                "{:02}:{:02}  {kind}",
                self.opened.elapsed().as_secs() / 60,
                self.opened.elapsed().as_secs() % 60
            ),
            text.into(),
        ));
    }

    fn send(&mut self, command: Command) -> bool {
        if let Err(error) = self.worker.send(command) {
            self.error = Some(error.clone());
            self.record("ERROR", &error);
            false
        } else {
            true
        }
    }

    fn events(&mut self) {
        loop {
            match self.worker.poll() {
                Ok(Event::Started(info)) => {
                    if self.phase != Phase::Stopping {
                        self.phase = Phase::Running;
                    }
                    self.record("SYSTEM", &info);
                    self.record("CONNECTED", "Kernel boot and UART response confirmed.");
                }
                Ok(Event::Queue(queue)) => {
                    for (index, job) in queue.jobs.iter().enumerate() {
                        if self
                            .queue
                            .jobs
                            .get(index)
                            .is_none_or(|old| old.state != job.state)
                        {
                            self.record("QUEUE", &format!("Job {}: {:?}", index + 1, job.state));
                        }
                    }
                    self.queue = queue;
                }
                Ok(Event::Task(task)) => {
                    if self
                        .task
                        .is_none_or(|old| old.id != task.id || old.state != task.state)
                    {
                        self.record(
                            "TASK",
                            &format!(
                                "Task {}: {} ({}/{} bytes).",
                                task.id,
                                task.state.label(),
                                task.processed,
                                task.total
                            ),
                        );
                    }
                    self.task = Some(task);
                }
                Ok(Event::Uptime {
                    milliseconds,
                    frequency,
                }) => {
                    if self.uptime.is_none() {
                        self.record(
                            "CLOCK",
                            &format!("Guest counter confirmed: {frequency} Hz."),
                        );
                    }
                    self.uptime = Some(milliseconds);
                    self.sampled = Some(Instant::now());
                }
                Ok(Event::Reply { command, payload }) => {
                    self.pending = false;
                    if command.starts_with("echo") {
                        self.echo = Some(payload.clone());
                    }
                    self.record("RESPONSE", &payload);
                }
                Ok(Event::Rejected(error)) => {
                    self.pending = false;
                    self.error = Some(error.clone());
                    self.record("REJECTED", &error);
                }
                Ok(Event::Stopped) => {
                    self.phase = Phase::Stopped;
                    self.pending = false;
                    self.task = None;
                    self.uptime = None;
                    self.sampled = None;
                    self.echo = None;
                    self.record("STOPPED", "QEMU stopped. Session resources released.");
                }
                Ok(Event::Failed(error)) => {
                    self.phase = Phase::Failed;
                    self.pending = false;
                    self.task = None;
                    self.uptime = None;
                    self.sampled = None;
                    self.echo = None;
                    self.error = Some(error.clone());
                    self.record("DISCONNECTED", &error);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if self.phase != Phase::Failed {
                        self.phase = Phase::Failed;
                        self.task = None;
                        self.uptime = None;
                        self.sampled = None;
                        self.echo = None;
                        self.pending = false;
                        self.error = Some("Session worker closed. Restart the panel.".into());
                    }
                    break;
                }
            }
        }
    }

    fn status(&self) -> &'static str {
        match self.phase {
            Phase::Stopped => "OFFLINE",
            Phase::Starting => "STARTING",
            Phase::Running => "CONNECTED",
            Phase::Stopping => "STOPPING",
            Phase::Failed => "CONNECTION FAILED",
        }
    }

    fn task_controls(&mut self, ui: &mut egui::Ui) {
        let connected = self.phase == Phase::Running && !self.pending && !self.queue.busy();
        let running = self
            .task
            .is_some_and(|task| task.state == TaskState::Running);
        ui.label(RichText::new("WORD COUNT").size(11.0).color(ACCENT));
        ui.label("A real job, processed inside the kernel.");
        ui.add_enabled(
            connected && !running,
            egui::TextEdit::multiline(&mut self.task_text)
                .desired_rows(3)
                .char_limit(192)
                .desired_width(308.0),
        );
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    connected && !running && self.task.is_some(),
                    egui::Button::new("Start task"),
                )
                .clicked()
            {
                self.error = None;
                if self.send(Command::Request(format!("task.start {}", self.task_text))) {
                    self.pending = true;
                    self.record("REQUEST", "Word count queued for kernel validation.");
                }
            }
            if ui
                .add_enabled(connected && running, egui::Button::new("Cancel task"))
                .clicked()
                && let Some(task) = self.task
            {
                self.error = None;
                if self.send(Command::Request(format!("task.cancel {}", task.id))) {
                    self.pending = true;
                    self.record(
                        "REQUEST",
                        "Cancellation requested for the observed task ID.",
                    );
                }
            }
        });
        if let Some(task) = self.task {
            let progress = if task.total == 0 {
                if task.state == TaskState::Completed {
                    1.0
                } else {
                    0.0
                }
            } else {
                task.processed as f32 / task.total as f32
            };
            ui.add(
                egui::ProgressBar::new(progress)
                    .text(format!(
                        "{} · {} / {} bytes",
                        task.state.label(),
                        task.processed,
                        task.total
                    ))
                    .animate(false),
            );
            match task.state {
                TaskState::Completed => {
                    ui.label(RichText::new(format!("Result: {} words", task.words)).color(ACCENT));
                }
                TaskState::Running | TaskState::Cancelled => {
                    ui.label(format!("Partial count: {} words", task.words));
                }
                TaskState::Idle => {
                    ui.label("No task started in this session.");
                }
            }
        } else {
            ui.label("Waiting for kernel task state.");
        }
        ui.label(
            RichText::new("Learning pace: 8 bytes / 100 ms")
                .size(10.0)
                .color(MUTED),
        );
        ui.label(
            RichText::new("ASCII letters and digits form words. Max 192 bytes.")
                .size(10.0)
                .color(MUTED),
        );
    }

    fn queue_controls(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("FIFO COORDINATOR · HOST")
                .size(11.0)
                .color(ACCENT),
        );
        ui.label("One guest job at a time. No AI yet.");
        ui.add(
            egui::TextEdit::multiline(&mut self.task_text)
                .desired_rows(2)
                .char_limit(192)
                .desired_width(308.0),
        );
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.queue.jobs.len() < CAPACITY,
                    egui::Button::new("Add job"),
                )
                .clicked()
            {
                self.error = None;
                self.send(Command::Enqueue(self.task_text.clone()));
            }
            if ui
                .add_enabled(!self.queue.busy(), egui::Button::new("Clear queue"))
                .clicked()
            {
                self.send(Command::ClearQueue);
            }
        });
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.phase == Phase::Running
                        && !self.queue.enabled
                        && self.queue.jobs.iter().any(|j| j.state == JobState::Pending),
                    egui::Button::new("Run queue"),
                )
                .clicked()
            {
                self.error = None;
                self.send(Command::RunQueue);
            }
            if ui
                .add_enabled(self.queue.enabled, egui::Button::new("Pause queue"))
                .clicked()
            {
                self.send(Command::PauseQueue);
            }
        });
        ui.label(if self.queue.enabled {
            "Automatic FIFO execution"
        } else if self.queue.busy() {
            "Paused · finishing current job"
        } else if self.queue.jobs.iter().any(|j| j.state == JobState::Pending) {
            "Paused · waiting for Run queue"
        } else {
            "Ready · add jobs to run"
        });
        egui::ScrollArea::vertical()
            .id_salt("queue_jobs")
            .max_height(140.0)
            .show(ui, |ui| {
                for (index, job) in self.queue.jobs.iter().enumerate() {
                    ui.label(format!(
                        "Job {} · {:?} · {} bytes",
                        index + 1,
                        job.state,
                        job.text.len()
                    ));
                    if let Some(task) = job.observed {
                        let count = if job.state == JobState::Completed {
                            "result"
                        } else {
                            "last observed count"
                        };
                        ui.label(
                            RichText::new(format!(
                                "Guest #{} · {}/{} bytes · {count}: {}",
                                task.id, task.processed, task.total, task.words
                            ))
                            .size(11.0)
                            .color(MUTED),
                        );
                    }
                }
            });
        ui.label(RichText::new("Max 8 jobs · 192 ASCII bytes each\nPause lets the current job finish.\nGuest learning pace: 8 bytes / 100 ms").size(10.0).color(MUTED));
    }

    fn device(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(CARD)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(32)
            .inner_margin(22)
            .show(ui, |ui| {
                ui.set_width(308.0);
                ui.set_min_height(455.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("RAI-OS").size(16.0).strong());
                    if self.queue_view {
                        ui.label(RichText::new(self.status()).size(11.0).color(ACCENT));
                        let time = self
                            .uptime
                            .map(|ms| format!("Kernel uptime · {} s", ms / 1000))
                            .unwrap_or_else(|| "Waiting for a real kernel reading".into());
                        ui.label(RichText::new(time).size(12.0).color(MUTED));
                        if self
                            .sampled
                            .is_some_and(|at| at.elapsed() > Duration::from_secs(3))
                        {
                            ui.label("Last reading · awaiting kernel");
                        }
                    } else {
                        ui.add_space(10.0);
                        let color = if self.phase == Phase::Running {
                            ACCENT
                        } else {
                            MUTED
                        };
                        ui.label(RichText::new(self.status()).size(11.0).color(color));
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(if self.task_view {
                                "Work, one step at a time."
                            } else {
                                "Your system.\nTaking shape."
                            })
                            .size(26.0)
                            .strong(),
                        );
                        ui.add_space(16.0);
                        ui.label(RichText::new("KERNEL UPTIME").size(10.0).color(MUTED));
                        let time = self
                            .uptime
                            .map(|ms| {
                                format!(
                                    "{:02}:{:02}:{:02}",
                                    ms / 3_600_000,
                                    ms / 60_000 % 60,
                                    ms / 1000 % 60
                                )
                            })
                            .unwrap_or_else(|| "--:--:--".into());
                        ui.label(
                            RichText::new(time).font(FontId::monospace(if self.task_view {
                                25.0
                            } else {
                                38.0
                            })),
                        );
                        let caption = match self.sampled {
                            Some(at) if at.elapsed() > Duration::from_secs(3) => {
                                "Last reading · awaiting kernel"
                            }
                            Some(_) => "Read from the running kernel",
                            None => "Waiting for a real kernel reading",
                        };
                        ui.label(RichText::new(caption).size(11.0).color(MUTED));
                        ui.add_space(16.0);
                    }
                });
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.queue_view, "Queue").clicked() {
                        self.queue_view = true;
                    }
                    if ui
                        .selectable_label(!self.queue_view && self.task_view, "Task")
                        .clicked()
                    {
                        self.queue_view = false;
                        self.task_view = true;
                    }
                    if ui
                        .selectable_label(!self.queue_view && !self.task_view, "Connection")
                        .clicked()
                    {
                        self.queue_view = false;
                        self.task_view = false;
                    }
                });
                if self.queue_view {
                    self.queue_controls(ui);
                } else if self.task_view {
                    self.task_controls(ui);
                } else {
                    ui.separator();
                    ui.label(RichText::new("CONNECTION CHECK").size(10.0).color(MUTED));
                    ui.label("Send a message. Get it back from RAI-OS.");
                    let enabled = self.phase == Phase::Running && !self.pending;
                    ui.add_enabled(
                        enabled,
                        egui::TextEdit::singleline(&mut self.input)
                            .desired_width(f32::INFINITY)
                            .char_limit(220),
                    );
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::new("Send message").min_size(egui::vec2(308.0, 40.0)),
                        )
                        .clicked()
                    {
                        self.error = None;
                        let command = format!("echo {}", self.input);
                        if self.send(Command::Request(command)) {
                            self.pending = true;
                            self.echo = None;
                            self.record("REQUEST", "Echo queued for validation.");
                        }
                    }
                    if let Some(echo) = &self.echo {
                        ui.label(RichText::new("KERNEL REPLY").size(10.0).color(ACCENT));
                        ui.add(egui::Label::new(echo).wrap());
                    }
                }
                ui.add_space(12.0);
                ui.label(
                    RichText::new("AI service · Not connected")
                        .size(11.0)
                        .color(MUTED),
                );
            });
    }
}

impl eframe::App for Panel {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.events();
        ctx.request_repaint_after(Duration::from_millis(100));
        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::new().fill(BG).inner_margin(24))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("RAI").size(24.0).strong().color(ACCENT));
                    ui.label(RichText::new("/  DEVICE LAB").size(12.0).color(MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.toggle_value(&mut self.show_activity, "Activity");
                        if ui
                            .add_enabled(
                                self.phase == Phase::Running && !self.pending,
                                egui::Button::new("System info"),
                            )
                            .clicked()
                        {
                            self.error = None;
                            self.show_activity = true;
                            if self.send(Command::Request("info".into())) {
                                self.pending = true;
                                self.record("REQUEST", "System information requested.");
                            }
                        }
                        if matches!(self.phase, Phase::Stopped | Phase::Failed) {
                            if ui
                                .add(
                                    egui::Button::new(RichText::new("Start kernel").color(BG))
                                        .fill(ACCENT),
                                )
                                .clicked()
                                && self.send(Command::Start)
                            {
                                self.phase = Phase::Starting;
                                self.error = None;
                                self.echo = None;
                                self.record("START", "Starting QEMU; waiting for the kernel boot.");
                            }
                        } else if ui
                            .add_enabled(
                                self.phase != Phase::Stopping,
                                egui::Button::new("Stop kernel"),
                            )
                            .clicked()
                            && self.send(Command::Stop)
                        {
                            self.phase = Phase::Stopping;
                            self.record("STOP", "Waiting for session cleanup.");
                        }
                    });
                });
            });
        egui::TopBottomPanel::bottom("footer")
            .frame(egui::Frame::new().fill(BG).inner_margin(16))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("HOST-RENDERED PREVIEW")
                            .size(10.0)
                            .color(MUTED),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new("Real RAI-OS kernel in QEMU · ARM64")
                            .size(11.0)
                            .color(MUTED),
                    );
                });
            });
        if self.show_activity {
            egui::SidePanel::right("activity")
                .exact_width(310.0)
                .resizable(false)
                .frame(egui::Frame::new().fill(BG).inner_margin(24))
                .show(ctx, |ui| {
                    ui.heading("Activity");
                    ui.label(
                        RichText::new("Observed events from this session")
                            .size(11.0)
                            .color(MUTED),
                    );
                    ui.add_space(12.0);
                    egui::ScrollArea::vertical()
                        .id_salt("activity_log")
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for (label, text) in &self.log {
                                ui.label(RichText::new(label).monospace().size(10.0).color(ACCENT));
                                ui.add(egui::Label::new(RichText::new(text).size(12.0)).wrap());
                                ui.add_space(10.0);
                            }
                        });
                });
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(24))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("A foundation for a different mobile OS.")
                                .size(14.0)
                                .color(MUTED),
                        );
                        ui.add_space(20.0);
                        self.device(ui);
                        ui.add_space(14.0);
                        if let Some(error) = &self.error {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(error).color(Color32::from_rgb(240, 175, 157)),
                                )
                                .wrap(),
                            );
                        }
                    });
                });
            });
    }
}
