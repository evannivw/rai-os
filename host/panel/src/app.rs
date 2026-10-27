//! Native host presentation. Values change only when the session worker reports them.
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
                    if command != "info" {
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
                    self.uptime = None;
                    self.sampled = None;
                    self.echo = None;
                    self.record("STOPPED", "QEMU stopped. Session resources released.");
                }
                Ok(Event::Failed(error)) => {
                    self.phase = Phase::Failed;
                    self.pending = false;
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
                    ui.add_space(10.0);
                    let color = if self.phase == Phase::Running {
                        ACCENT
                    } else {
                        MUTED
                    };
                    ui.label(RichText::new(self.status()).size(11.0).color(color));
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Your system.\nTaking shape.")
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
                    ui.label(RichText::new(time).font(FontId::monospace(38.0)));
                    let caption = match self.sampled {
                        Some(at) if at.elapsed() > Duration::from_secs(3) => {
                            "Last reading · awaiting kernel"
                        }
                        Some(_) => "Read from the running kernel",
                        None => "Waiting for a real kernel reading",
                    };
                    ui.label(RichText::new(caption).size(11.0).color(MUTED));
                    ui.add_space(16.0);
                });
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
