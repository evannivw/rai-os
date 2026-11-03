//! UART line transport and response rendering; no heap allocation or global buffers.
use crate::{
    clock::Clock, command, memory::Memory, mmu::Mmu, protocol, task::Task, timer::Timer, uart,
};
use core::fmt::Display;

const CAPACITY: usize = 256;

fn response(id: &str, status: &str, payload: impl Display) {
    uart::write("@rai1\t");
    uart::write(id);
    uart::put(b'\t');
    uart::write(status);
    uart::put(b'\t');
    uart::write_display(payload);
    uart::write("\n");
}

struct Console<'a> {
    clock: &'a Clock,
    task: Task,
    files: crate::ramfs::Store,
    workspace: crate::workspace::Workspace,
    apps: crate::app_ui::Controller,
    memory: &'a mut Memory,
    mmu: &'a Mmu,
    timer: &'a mut Timer,
    sessions: crate::schedule::Sessions,
    #[cfg(feature = "guest-display")]
    display: Option<crate::display::Display>,
}
impl Console<'_> {
    fn execute<'a>(
        &mut self,
        operation: command::Operation<'a>,
    ) -> Result<command::Reply<'a>, &'static str> {
        if let command::Operation::Files(operation) = operation {
            let result = self.files.execute(operation);
            #[cfg(feature = "guest-display")]
            if let Some(display) = &mut self.display {
                if self.workspace.active {
                    crate::display::view::render_workspace(display, &self.files, &self.workspace);
                } else {
                    crate::display::view::render_files(
                        display,
                        &self.files,
                        result.as_ref().err().copied(),
                    );
                }
            }
            return result.map(command::Reply::Files);
        }
        #[cfg(feature = "guest-display")]
        let is_index = matches!(operation, command::Operation::RecordIndex(_));
        #[cfg(feature = "guest-display")]
        let publication_before = self.sessions.coordinator.publication_status().target() != 0;
        #[cfg(feature = "guest-display")]
        let is_publication = matches!(
            operation,
            command::Operation::PublicationStatus
                | command::Operation::PublicationRead
                | command::Operation::PublicationPrepare
                | command::Operation::PublicationCommit(_)
                | command::Operation::PublicationReject(_)
        );
        #[cfg(feature = "guest-display")]
        let is_coordinator_index = matches!(
            operation,
            command::Operation::CoordIndexStart
                | command::Operation::CoordResult
                | command::Operation::CoordGoal(_)
                | command::Operation::CoordBundle
                | command::Operation::CoordBundleStatus
        ) || (self.sessions.coordinator.status().index_mode()
            && matches!(
                operation,
                command::Operation::CoordObserve(_)
                    | command::Operation::CoordSelect(_)
                    | command::Operation::CoordPropose(_)
                    | command::Operation::CoordStatus
                    | command::Operation::CoordStop
            ));
        let result = command::execute(
            operation,
            self.clock.snapshot(),
            &mut self.task,
            self.memory,
            self.mmu,
            self.timer,
            &mut self.sessions,
        );
        #[cfg(feature = "guest-display")]
        self.render_result(
            &result,
            is_index,
            is_coordinator_index,
            is_publication,
            publication_before,
        );
        result
    }
    // Keep rendering temporaries out of the command execution frame. In debug
    // builds the retained session and large replies already occupy stack space.
    #[cfg(feature = "guest-display")]
    fn render_result(
        &mut self,
        result: &Result<command::Reply<'_>, &'static str>,
        is_index: bool,
        is_coordinator_index: bool,
        is_publication: bool,
        publication_before: bool,
    ) {
        #[cfg(feature = "guest-display")]
        if is_index && let Some(display) = &mut self.display {
            use crate::display::view::{self, View};
            match result {
                Ok(command::Reply::Index(report)) => {
                    view::render(display, View::Index(report.id(), report.output()))
                }
                Err(code) => view::render(display, View::Error(code)),
                _ => {}
            }
        }
        #[cfg(feature = "guest-display")]
        if (is_coordinator_index || is_publication)
            && let Some(display) = &mut self.display
        {
            use crate::display::view::{self, View};
            let publication = self.sessions.coordinator.publication_status();
            if is_publication || publication_before || publication.target() != 0 {
                view::render(
                    display,
                    View::Publication(
                        publication,
                        self.sessions.coordinator.bundle_status().ok(),
                        result.as_ref().err().copied(),
                    ),
                );
                return;
            }
            match result {
                Err(code) => view::render(display, View::Error(code)),
                Ok(_) => {
                    let report = self.sessions.coordinator.status();
                    if let Ok(status) = self.sessions.coordinator.bundle_status() {
                        view::render(display, View::Bundle(status));
                    } else if let Ok(artifact) = self.sessions.coordinator.artifact() {
                        let value = if report.reused() {
                            View::Reused(artifact.id(), artifact.text())
                        } else {
                            View::Index(artifact.id(), artifact.text())
                        };
                        view::render(display, value);
                    } else if let Some((id, ticket)) = report.pending() {
                        view::render(display, View::Pending(id, ticket));
                    } else if let Some(error) = report.failure() {
                        view::render(display, View::Error(error));
                    } else {
                        view::render(
                            display,
                            if report.index_mode() {
                                View::CoordinatorReady
                            } else {
                                View::Ready
                            },
                        );
                    }
                }
            }
        }
    }
    fn parse_error(&mut self, line: &str, code: &str) {
        #[cfg(feature = "guest-display")]
        if (line.starts_with("coord.publish.")
            || line
                .split('\t')
                .nth(2)
                .is_some_and(|op| op.starts_with("coord.publish."))
            || line.starts_with("record.index")
            || line.split('\t').nth(2) == Some("record.index")
            || (self.sessions.coordinator.status().index_mode()
                && (line.starts_with("coord.")
                    || line
                        .split('\t')
                        .nth(2)
                        .is_some_and(|op| op.starts_with("coord.")))))
            && let Some(display) = &mut self.display
        {
            use crate::display::view::{self, View};
            let publication = self.sessions.coordinator.publication_status();
            let operation = if line.starts_with('@') {
                line.split('\t').nth(2).unwrap_or("")
            } else {
                line
            };
            if operation.starts_with("coord.")
                && (publication.target() != 0 || operation.starts_with("coord.publish."))
            {
                view::render(
                    display,
                    View::Publication(
                        publication,
                        self.sessions.coordinator.bundle_status().ok(),
                        Some(code),
                    ),
                );
            } else {
                view::render(display, View::Error(code));
            }
        }
        #[cfg(not(feature = "guest-display"))]
        let _ = (line, code);
    }
    fn workspace_command(&mut self, name: &str, arg: Option<&str>, id: Option<&str>) {
        if name == "ui.open" {
            self.apps.active = false;
        }
        let result = self.workspace.execute(name, arg, &mut self.files);
        #[cfg(feature = "guest-display")]
        if let Some(display) = &mut self.display {
            crate::display::view::render_workspace(display, &self.files, &self.workspace);
        }
        match result {
            Ok(reply) => {
                if let Some(id) = id {
                    response(id, "ok", reply)
                } else {
                    uart::write_display(reply);
                    uart::write("\n");
                }
            }
            Err(error) => {
                if let Some(id) = id {
                    response(id, "err", error)
                } else {
                    uart::write(error);
                    uart::write("\n");
                }
            }
        }
    }

    fn app_command(&mut self, name: &str, arg: Option<&str>, id: Option<&str>) {
        let id = id.unwrap_or("0");
        if name == "app.catalog" {
            if arg.is_some() {
                response(id, "err", "bad_args");
            } else {
                response(id, "ok", crate::microapp::Catalog(&self.apps.runtime));
            }
            return;
        }
        if name == "app.totals" {
            if let Some((app, scope)) = arg.and_then(|s| s.split_once(':'))
                && matches!(scope, "all" | "pending")
                && let Some(a) = self
                    .apps
                    .runtime
                    .find(app)
                    .and_then(|i| self.apps.runtime.app(i))
            {
                response(id, "ok", crate::microapp::Totals(a, scope == "pending"));
            } else {
                response(id, "err", "bad_args");
            }
            return;
        }
        if matches!(name, "app.schema" | "app.export") {
            if let Some(a) = arg
                .and_then(|s| self.apps.runtime.find(s))
                .and_then(|i| self.apps.runtime.app(i))
            {
                if name == "app.schema" {
                    response(id, "ok", a.spec);
                } else {
                    response(id, "ok", crate::microapp::Export(a));
                }
            } else {
                response(id, "err", "app_not_found");
            }
            return;
        }
        if name == "app.open" {
            self.workspace.active = false;
        }
        let result = self.apps.execute(name, arg);
        #[cfg(feature = "guest-display")]
        if let Some(display) = &mut self.display {
            crate::display::apps::render(display, &self.apps);
        }
        match result {
            Ok(reply) => response(id, "ok", reply),
            Err(error) => response(id, "err", error),
        }
    }
    fn dispatch(&mut self, line: &str) {
        if line.starts_with('@') {
            match protocol::parse(line) {
                Ok(request) if matches!(request.operation, command::Operation::App(_, _)) => {
                    if let command::Operation::App(n, a) = request.operation {
                        self.app_command(n, a, Some(request.id));
                    }
                }
                Ok(request) if matches!(request.operation, command::Operation::Ui(_, _)) => {
                    if let command::Operation::Ui(name, arg) = request.operation {
                        self.workspace_command(name, arg, Some(request.id));
                    }
                }
                Ok(request) => match self.execute(request.operation) {
                    Ok(reply) => response(request.id, "ok", reply),
                    Err(code) => response(request.id, "err", code),
                },
                Err(error) => {
                    self.parse_error(line, error.code);
                    response(error.id, "err", error.code);
                }
            }
        } else if !line.is_empty() {
            let result = match command::parse_shell(line) {
                Ok(command::Operation::App(n, a)) => {
                    self.app_command(n, a, None);
                    return;
                }
                Ok(command::Operation::Ui(name, arg)) => {
                    self.workspace_command(name, arg, None);
                    return;
                }
                Ok(operation) => self.execute(operation),
                Err(code) => {
                    self.parse_error(line, code);
                    Err(code)
                }
            };
            match result {
                Ok(reply) => uart::write_display(reply),
                Err(code) => {
                    uart::write("error: ");
                    uart::write(code);
                }
            }
            uart::write("\n");
        }
    }
}

pub fn run(clock: &Clock, memory: &mut Memory, mmu: &Mmu, timer: &mut Timer) -> ! {
    #[cfg(feature = "guest-display")]
    // SAFETY: sole owner at boot, fixed mappings active, one CPU/IRQs masked.
    let display = match unsafe { crate::display::Display::initialize() } {
        Ok(Some(mut display)) => {
            crate::display::view::render(&mut display, crate::display::view::View::Ready);
            uart::write("Display ready: 480x800 guest framebuffer\n");
            Some(display)
        }
        Ok(None) => {
            uart::write("Display unavailable: ramfb not configured\n");
            None
        }
        Err(error) => {
            uart::write(error);
            uart::write("\n");
            None
        }
    };
    let mut console = Console {
        clock,
        task: Task::new(),
        files: crate::ramfs::Store::new(),
        workspace: crate::workspace::Workspace::new(),
        apps: crate::app_ui::Controller::new(),
        memory,
        mmu,
        timer,
        sessions: crate::schedule::Sessions::new(),
        #[cfg(feature = "guest-display")]
        display,
    };
    let mut buffer = [0u8; CAPACITY];
    let mut length = 0;
    let mut rejected = None;
    let mut machine = false;
    let mut after_cr = false;
    uart::write("Console ready: type help for commands\n");
    loop {
        // Every loop gets a bounded task step, even during a partial/invalid frame.
        console.task.step(clock.snapshot().milliseconds);
        let byte = match uart::try_get() {
            Ok(Some(byte)) => byte,
            Ok(None) => {
                core::hint::spin_loop();
                continue;
            }
            Err(()) => {
                rejected = Some("uart_error");
                continue;
            }
        };
        if after_cr && byte == b'\n' {
            after_cr = false;
            continue;
        }
        after_cr = byte == b'\r';
        if byte == b'\r' || byte == b'\n' {
            if !machine {
                uart::write("\n");
            }
            if let Some(code) = rejected {
                // Framing errors cannot reliably preserve a request ID.
                response("0", "err", code);
            } else {
                // Input was restricted to ASCII; keep a checked conversion anyway.
                match core::str::from_utf8(&buffer[..length]) {
                    Ok(line) => console.dispatch(line),
                    Err(_) => response("0", "err", "invalid_byte"),
                }
            }
            length = 0;
            rejected = None;
            machine = false;
            continue;
        }
        if rejected.is_some() {
            continue; // Discard the ENTIRE rejected line, never execute its tail.
        }
        if length == 0 {
            machine = byte == b'@';
        }
        if !machine && (byte == 8 || byte == 127) {
            if length > 0 {
                length -= 1;
                uart::write("\x08 \x08");
            }
            continue;
        }
        if !(byte.is_ascii_graphic() || byte == b' ' || (machine && byte == b'\t')) {
            rejected = Some("invalid_byte");
        } else if length == CAPACITY {
            rejected = Some("line_too_long");
        } else {
            buffer[length] = byte;
            length += 1;
            if !machine {
                uart::put(byte);
            }
        }
    }
}
