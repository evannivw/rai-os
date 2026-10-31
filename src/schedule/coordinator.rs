//! Two retained EL0 services with coordinator-only host ingress/publication.
//! Transport, bounded dispatch and report validation; goal policy lives at EL0.
use super::{run_turn, runtime::valid_context};
use crate::{
    bundle::{self, Required},
    context::Frame,
    index_stream::{Stream, Text},
    ipc::{self, Mailboxes, Message, TaskId},
    mmu::{Mmu, TaskSpace},
    records::RecordId,
    timer::Timer,
    user_memory::Lease,
};
use core::{fmt, ptr};
#[derive(Clone, Copy, PartialEq, Eq)]
enum Wait {
    Ready,
    Host,
    Ipc,
}
struct Task {
    frame: Frame,
    wait: Wait,
}
impl Task {
    fn new() -> Self {
        Self {
            frame: Frame::default(),
            wait: Wait::Ready,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Report {
    state: &'static str,
    event: &'static str,
    ticket: u64,
    result: u64,
    completed: u64,
    calls: [u8; 2],
    slices: [u8; 2],
    trace: [u8; 384],
    index_mode: bool,
    active_reused: bool,
    cached: u64,
    hits: u64,
    output_bytes: usize,
    length: usize,
}
impl Report {
    #[cfg(feature = "guest-display")]
    pub fn index_mode(&self) -> bool {
        self.index_mode
    }
    #[cfg(feature = "guest-display")]
    pub fn pending(&self) -> Option<(RecordId, u64)> {
        if self.state != "awaiting_proposal" {
            return None;
        }
        RecordId::from_raw(self.result >> 32)
            .ok()
            .map(|id| (id, self.ticket))
    }
    #[cfg(feature = "guest-display")]
    pub fn reused(&self) -> bool {
        self.active_reused
    }
    #[cfg(feature = "guest-display")]
    pub fn failure(&self) -> Option<&'static str> {
        (self.state == "failed").then_some(self.event)
    }
    fn stopped() -> Self {
        Self {
            state: "stopped",
            event: "none",
            ticket: 0,
            result: 0,
            completed: 0,
            calls: [0; 2],
            slices: [0; 2],
            trace: [0; 384],
            index_mode: false,
            active_reused: false,
            cached: 0,
            hits: 0,
            output_bytes: 0,
            length: 0,
        }
    }
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "coordinator=EL0; worker=EL0; state={}; event={}; ticket={}; record=",
            self.state, self.event, self.ticket
        )?;
        if let Ok(id) = RecordId::from_raw(self.result >> 32) {
            write!(f, "{id}")?;
        } else {
            f.write_str("none")?;
        }
        write!(
            f,
            "; bytes={}; tokens={}; completed={}; a_calls={}; b_calls={}; a_slices={}; b_slices={}; trace=",
            (self.result >> 16) & 0xffff,
            self.result & 0xffff,
            self.completed,
            self.calls[0],
            self.calls[1],
            self.slices[0],
            self.slices[1]
        )?;
        for (i, event) in self.trace[..self.length].chunks_exact(2).enumerate() {
            if i != 0 {
                f.write_str(",")?;
            }
            f.write_str(core::str::from_utf8(event).map_err(|_| fmt::Error)?)?;
        }
        if self.index_mode {
            write!(
                f,
                "; operation=record.index; cached_records={}; cache_hits={}; output_bytes={}",
                self.cached, self.hits, self.output_bytes
            )?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
struct Cached {
    result: u64,
    text: Text,
}
pub struct Artifact {
    id: RecordId,
    text: Text,
}
impl Artifact {
    pub fn id(&self) -> RecordId {
        self.id
    }
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}
impl fmt::Display for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "record={}; bytes={}; worker=EL0; state=completed; index={}",
            self.id(),
            self.id().bytes(),
            self.text()
        )
    }
}
/// Typed proposal boundary; tickets are nonzero u32 values, keys use the record allowlist.
pub struct Proposal {
    pub ticket: u32,
    pub id: RecordId,
}
pub struct Coordinator {
    lease: Option<Lease>,
    tasks: [Task; 2],
    boxes: Mailboxes,
    report: Report,
    // Monotonic across stop/start within this boot. Never reuse a delayed proposal ticket.
    issued: u32,
    cache: [Option<Cached>; 6],
    input: Option<(u64, u32, RecordId)>,
    goal_input: Option<(u32, Required)>,
    goal: Option<(u32, Required)>,
    publication: crate::publication::Store,
}
fn endpoint(id: usize) -> TaskId {
    TaskId::from_raw(id as u64 + 1).expect("fixed task")
}
fn bounds(id: usize) -> ((u64, u64), (u64, u64)) {
    unsafe extern "C" {
        static mut __user_stack_bottom: [u8; 8192];
        static __user_text_start: u8;
        static __user_text_end: u8;
    }
    let bottom = ptr::addr_of_mut!(__user_stack_bottom) as u64 + id as u64 * 4096;
    (
        (bottom, bottom + 4096),
        (
            ptr::addr_of!(__user_text_start) as u64,
            ptr::addr_of!(__user_text_end) as u64,
        ),
    )
}
impl Coordinator {
    pub fn new() -> Self {
        Self {
            lease: None,
            tasks: [Task::new(), Task::new()],
            boxes: Mailboxes::new([0b10, 0b01]),
            report: Report::stopped(),
            issued: 0,
            cache: [None; 6],
            input: None,
            goal_input: None,
            goal: None,
            publication: crate::publication::Store::new(),
        }
    }
    pub fn status(&self) -> Report {
        self.report
    }
    fn cache_mask(&self) -> u64 {
        self.cache.iter().enumerate().fold(0, |mask, (n, entry)| {
            mask | (u64::from(entry.is_some()) << (n + 1))
        })
    }
    pub fn bundle_status(&self) -> Result<bundle::Status, &'static str> {
        if !self.report.index_mode {
            return Err("coord_not_indexing");
        }
        if self.lease.is_none() {
            return Err("coord_not_running");
        }
        let (generation, required) = self.goal.ok_or("coord_no_required_set")?;
        Ok(bundle::Status {
            generation,
            required,
            ready_mask: required.mask() & self.cache_mask(),
            completed: self.report.completed,
        })
    }
    pub fn bundle(&self) -> Result<bundle::Artifact, &'static str> {
        let status = self.bundle_status()?;
        if !status.ready() {
            return Err("coord_bundle_incomplete");
        }
        let mut indexes = [Text::EMPTY; bundle::MAX_RECORDS];
        for (n, id) in status.required.ids().enumerate() {
            indexes[n] = self.cache[id.raw() as usize - 1]
                .ok_or("coord_bundle_incomplete")?
                .text;
        }
        Ok(bundle::Artifact { status, indexes })
    }
    pub fn publication_status(&self) -> crate::publication::Status {
        self.publication.status()
    }
    pub fn publication_read(&self) -> Result<bundle::Artifact, &'static str> {
        self.publication.read()
    }
    pub fn publication_prepare(&mut self) -> Result<bundle::Artifact, &'static str> {
        let artifact = self.bundle()?;
        self.publication.can_prepare(artifact.status.generation)?;
        let ticket = self.issued.checked_add(1).ok_or("coord_ticket_limit")?;
        self.issued = ticket;
        self.publication.prepare(ticket);
        Ok(artifact)
    }
    pub fn publication_commit(
        &mut self,
        ticket: u32,
    ) -> Result<crate::publication::Status, &'static str> {
        self.publication.check_ticket(ticket)?;
        // Re-read guest completeness/generation after the host's independent check.
        // Single masked console owner: no event can interleave this commit.
        let artifact = self.bundle()?;
        self.publication.commit(ticket, artifact)
    }
    pub fn publication_reject(
        &mut self,
        ticket: u32,
    ) -> Result<crate::publication::Status, &'static str> {
        self.publication.reject(ticket)
    }
    pub fn set_goal(
        &mut self,
        required: Required,
        timer: &mut Timer,
        mmu: &Mmu,
    ) -> Result<bundle::Status, &'static str> {
        self.require_waiting()?;
        if !self.report.index_mode {
            return Err("coord_not_indexing");
        }
        let generation = self.issued.checked_add(1).ok_or("coord_ticket_limit")?;
        self.issued = generation;
        self.input = None;
        self.goal_input = Some((generation, required));
        self.tasks[0].frame.registers[..4].copy_from_slice(&[
            0,
            32,
            u64::from(generation),
            required.mask(),
        ]);
        self.tasks[0].wait = Wait::Ready;
        self.finish_turn(timer, mmu)?;
        let status = self.bundle_status()?;
        self.publication.begin(status.generation);
        Ok(status)
    }
    pub fn artifact(&self) -> Result<Artifact, &'static str> {
        if !self.report.index_mode {
            return Err("coord_not_indexing");
        }
        if self.report.state != "completed" || self.lease.is_none() {
            return Err("coord_result_unavailable");
        }
        let id = RecordId::from_raw(self.report.result >> 32)?;
        let entry = self.cache[id.raw() as usize - 1].ok_or("coord_result_unavailable")?;
        if entry.result != self.report.result {
            return Err("coord_result_unavailable");
        }
        Ok(Artifact {
            id,
            text: entry.text,
        })
    }
    fn retire(&mut self) {
        self.cache = [None; 6];
        self.input = None;
        self.goal_input = None;
        self.goal = None;
        if self.lease.is_some() {
            // SAFETY: exclusive lease over both stack pages; tasks suspended, base map
            // restored by run_turn; single masked CPU, no DMA or live user references.
            unsafe {
                ptr::write_bytes(bounds(0).0.0 as *mut u8, 0, 8192);
            }
        }
        self.tasks = [Task::new(), Task::new()];
        self.boxes = Mailboxes::new([0b10, 0b01]);
        self.lease = None;
    }
    pub fn stop(&mut self) -> Report {
        self.retire();
        self.publication = crate::publication::Store::new();
        self.report = Report::stopped();
        self.report
    }
    fn fail(&mut self, error: &'static str) -> &'static str {
        self.publication.fail(error);
        self.retire();
        self.report.cached = 0;
        self.report.output_bytes = 0;
        self.report.state = "failed";
        self.report.event = error;
        error
    }
    pub fn start(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<Report, &'static str> {
        self.start_mode(timer, mmu, false)
    }
    pub fn start_index(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<Report, &'static str> {
        self.start_mode(timer, mmu, true)
    }
    fn start_mode(
        &mut self,
        timer: &mut Timer,
        mmu: &Mmu,
        index_mode: bool,
    ) -> Result<Report, &'static str> {
        if self.report.state != "stopped" {
            return Err("coord_not_stopped");
        }
        self.lease = Some(Lease::acquire()?);
        self.report.index_mode = index_mode;
        // SAFETY: fresh exclusive lease, neither task entered; fixed linker-owned RAM.
        unsafe {
            ptr::write_bytes(bounds(0).0.0 as *mut u8, 0, 8192);
        }
        let entries = [
            super::coordinator_program::guest_coordinator as *const () as u64,
            super::worker_program::persistent_worker as *const () as u64,
        ];
        for (id, entry) in entries.into_iter().enumerate() {
            self.tasks[id].frame = Frame {
                elr: entry,
                user_sp: bounds(id).0.1,
                spsr: 0x340,
                ..Frame::default()
            };
        }
        self.tasks[0].frame.registers[0] = u64::from(index_mode);
        self.input = None;
        self.finish_turn(timer, mmu)
    }
    pub fn observe(
        &mut self,
        id: RecordId,
        timer: &mut Timer,
        mmu: &Mmu,
    ) -> Result<Report, &'static str> {
        self.require_waiting()?;
        let ticket = self.issued.checked_add(1).ok_or("coord_ticket_limit")?;
        self.issued = ticket;
        self.input(30, ticket, id);
        let report = self.finish_turn(timer, mmu)?;
        if report.event == "not_required" {
            return Err("coord_not_required");
        }
        Ok(report)
    }
    /// Bind a model-selected record to the exact required-set generation before
    /// issuing a per-record ticket. No host ingress interleaves this synchronous
    /// gate and observe turn. EL0 still admits the subsequent ticketed proposal.
    pub fn select(
        &mut self,
        selection: Proposal,
        timer: &mut Timer,
        mmu: &Mmu,
    ) -> Result<Report, &'static str> {
        self.require_waiting()?;
        let status = self.bundle_status()?;
        if selection.ticket != status.generation {
            return Err("coord_stale_goal");
        }
        let bit = 1_u64 << selection.id.raw();
        if status.required.mask() & bit == 0 {
            return Err("coord_not_required");
        }
        if status.ready_mask & bit != 0 {
            return Err("coord_already_completed");
        }
        self.observe(selection.id, timer, mmu)
    }
    pub fn propose(
        &mut self,
        proposal: Proposal,
        timer: &mut Timer,
        mmu: &Mmu,
    ) -> Result<Report, &'static str> {
        self.require_waiting()?;
        self.input(31, proposal.ticket, proposal.id);
        let report = self.finish_turn(timer, mmu)?;
        match report.event {
            "stale_proposal" => Err("coord_stale_proposal"),
            "wrong_record" => Err("coord_wrong_record"),
            "already_completed" => Err("coord_already_completed"),
            "no_goal" => Err("coord_no_goal"),
            "not_required" => Err("coord_not_required"),
            _ => Ok(report),
        }
    }
    fn require_waiting(&self) -> Result<(), &'static str> {
        if self.lease.is_some()
            && self.tasks[0].wait == Wait::Host
            && self.tasks[1].wait == Wait::Ipc
        {
            Ok(())
        } else {
            Err("coord_not_running")
        }
    }
    fn input(&mut self, kind: u64, ticket: u32, id: RecordId) {
        self.input = Some((kind, ticket, id));
        self.goal_input = None;
        // EL0 owns cache-use policy; the gateway supplies only attested metadata.
        let value = if self.report.index_mode {
            self.cache[id.raw() as usize - 1].map_or(id.raw() << 32, |entry| entry.result)
        } else {
            id.raw()
        };
        self.tasks[0].frame.registers[..4].copy_from_slice(&[0, kind, ticket as u64, value]);
        self.tasks[0].wait = Wait::Ready;
    }
    fn trace(&mut self, id: usize, event: u8) -> Result<(), &'static str> {
        let n = self.report.length;
        let limit = if self.report.index_mode { 384 } else { 192 };
        if n + 2 > limit {
            return Err("coord_trace_limit");
        }
        self.report.trace[n] = b'A' + id as u8;
        self.report.trace[n + 1] = event;
        self.report.length += 2;
        Ok(())
    }
    fn deliver(&mut self, id: usize, message: Message) {
        self.tasks[id].frame.registers[..4].copy_from_slice(&[
            0,
            message.sender.raw(),
            message.kind as u64,
            message.value,
        ]);
        self.tasks[id].wait = Wait::Ready;
    }
    fn finish_turn(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<Report, &'static str> {
        // Publish to the host only after BOTH continuations are waiting again.
        let previous = self.report;
        if let Err(error) = self.pump(timer, mmu) {
            let (calls, slices, trace, length) = (
                self.report.calls,
                self.report.slices,
                self.report.trace,
                self.report.length,
            );
            self.report = previous;
            self.report.calls = calls;
            self.report.slices = slices;
            self.report.trace = trace;
            self.report.length = length;
            return Err(self.fail(error));
        }
        Ok(self.report)
    }
    fn pump(&mut self, timer: &mut Timer, mmu: &Mmu) -> Result<(), &'static str> {
        self.report.calls = [0; 2];
        self.report.slices = [0; 2];
        self.report.length = 0;
        let mut cursor = 0;
        let mut publication = None;
        let mut goal_publication = None;
        let mut worker_result = None;
        let mut worker_sequence = None;
        let mut stream = Stream::new();
        let mut end = None;
        let mut requested = false;
        loop {
            let ready = [cursor, cursor ^ 1]
                .into_iter()
                .find(|id| self.tasks[*id].wait == Wait::Ready);
            let Some(id) = ready else {
                break;
            };
            let (stack, text) = bounds(id);
            if self.lease.is_none() || !valid_context(&self.tasks[id].frame, stack, text) {
                return Err("coord_context_invalid");
            }
            // SAFETY: lease owns both user pages across requests; validated EL0 PC,
            // private aligned SP/PSTATE; one masked kernel CPU. Each turn selects
            // its task map, captures to separate output, then restores the base map.
            let observed = unsafe {
                run_turn(
                    timer,
                    mmu,
                    TaskSpace::from_index(id)?,
                    &self.tasks[id].frame,
                )
            }?;
            self.tasks[id].frame = observed.frame;
            if !valid_context(&self.tasks[id].frame, stack, text) {
                return Err("coord_context_invalid");
            }
            if observed.vector == 9 {
                self.report.slices[id] += 1;
                self.trace(id, b'P')?;
                if self.report.slices[id] == 6 {
                    return Err("coord_slice_limit");
                }
            } else {
                if observed.vector != 8 || observed.frame.esr != 0x5600_0000 {
                    self.trace(id, b'F')?;
                    return Err("coord_task_fault");
                }
                self.report.calls[id] += 1;
                let limit = if self.report.index_mode {
                    128
                } else {
                    ipc::MAX_CALLS
                };
                if self.report.calls[id] == limit {
                    self.trace(id, b'L')?;
                    return Err("coord_call_limit");
                }
                let args = self.tasks[id].frame.registers;
                self.tasks[id].frame.registers[..4].fill(0);
                match args[8] {
                    10 => {
                        match self.boxes.send(endpoint(id), args[0], args[1], args[2]) {
                            Ok(()) => {
                                if self.report.index_mode {
                                    if id == 0 {
                                        let (kind, ticket, record) =
                                            self.input.ok_or("coord_index_request")?;
                                        if requested
                                            || args[0] != 2
                                            || args[1] != 40
                                            || kind != 31
                                            || self.report.state != "awaiting_proposal"
                                            || u64::from(ticket) != self.report.ticket
                                            || record.raw() != self.report.result >> 32
                                            || args[2] != record.raw()
                                            || self.goal.is_some_and(|(_, required)| {
                                                required.mask() & (1 << record.raw()) == 0
                                            })
                                        {
                                            return Err("coord_index_request");
                                        }
                                        requested = true;
                                    } else {
                                        if !requested || args[0] != 1 || end.is_some() {
                                            return Err("coord_index_order");
                                        }
                                        match args[1] {
                                            41 if worker_result.is_none() => {
                                                worker_result = Some(args[2])
                                            }
                                            42 if worker_result.is_some() => {
                                                stream.push(args[2])?
                                            }
                                            43 if worker_result.is_some() => {
                                                if stream.len() == 0
                                                    || args[2] & 0xffff_ffff != stream.len() as u64
                                                {
                                                    return Err("coord_index_length");
                                                }
                                                end = Some(args[2] & 0xffff_ffff);
                                                worker_sequence = Some(args[2] >> 32);
                                            }
                                            _ => return Err("coord_index_order"),
                                        }
                                    }
                                } else if id == 1 && args[1] == 21 {
                                    worker_result = Some(args[2]);
                                }
                                if !self.report.index_mode && id == 1 && args[1] == 23 {
                                    worker_sequence = Some(args[2]);
                                }
                            }
                            Err(error) => self.tasks[id].frame.registers[0] = error as u64,
                        }
                        self.trace(id, b'S')?;
                    }
                    12 => match self.boxes.receive_wait(endpoint(id)) {
                        Ok(Some(message)) => {
                            self.deliver(id, message);
                            self.trace(id, b'R')?;
                        }
                        Ok(None) => {
                            self.tasks[id].wait = Wait::Ipc;
                            self.trace(id, b'D')?;
                        }
                        Err(_) => return Err("coord_ipc_closed"),
                    },
                    13 if id == 0 => {
                        self.tasks[id].wait = Wait::Host;
                        self.trace(id, b'H')?;
                    }
                    15 if id == 0 && self.report.index_mode => {
                        let (generation, required) = self.goal_input.ok_or("coord_bad_goal")?;
                        if goal_publication.is_some()
                            || args[0] != u64::from(generation)
                            || args[1] != required.mask()
                            || args[2] != self.cache_mask()
                        {
                            return Err("coord_bad_goal");
                        }
                        goal_publication = Some((generation, required));
                        self.trace(id, b'G')?;
                    }
                    14 if id == 0 => {
                        if publication.is_some() {
                            return Err("coord_extra_report");
                        }
                        publication = Some([args[0], args[1], args[2], args[3], args[4]]);
                        self.trace(id, b'O')?;
                    }
                    _ => {
                        self.trace(id, b'F')?;
                        return Err("coord_bad_syscall");
                    }
                }
                for waiting in 0..2 {
                    if self.tasks[waiting].wait == Wait::Ipc {
                        match self.boxes.receive_wait(endpoint(waiting)) {
                            Ok(Some(message)) => {
                                self.deliver(waiting, message);
                                self.trace(waiting, b'W')?;
                            }
                            Ok(None) => {}
                            Err(_) => return Err("coord_ipc_closed"),
                        }
                    }
                }
            }
            cursor = id ^ 1;
        }
        if self.tasks[0].wait != Wait::Host || self.tasks[1].wait != Wait::Ipc {
            return Err("coord_deadlocked");
        }
        let [state, ticket, result, completed, event] =
            publication.ok_or("coord_missing_report")?;
        if state > 2
            || event > (if self.report.index_mode { 10 } else { 7 })
            || ticket > self.issued as u64
            || completed > self.issued as u64
        {
            return Err("coord_bad_report");
        }
        if state == 0 {
            if ticket != 0 || result != 0 || (!self.report.index_mode && completed != 0) {
                return Err("coord_bad_report");
            }
        } else {
            let id = RecordId::from_raw(result >> 32).map_err(|_| "coord_bad_report")?;
            if ticket == 0
                || (state == 1 && result != id.raw() << 32)
                || (state == 2
                    && (((result >> 16) & 0xffff) != id.bytes() || (result & 0xffff) > id.bytes()))
            {
                return Err("coord_bad_report");
            }
        }
        if self.goal_input.is_some() {
            if event != 9 || state != 0 || ticket != 0 || result != 0 || goal_publication.is_none()
            {
                return Err("coord_bad_goal");
            }
        } else if event == 9 || goal_publication.is_some() {
            return Err("coord_bad_goal");
        }
        if event == 3 {
            if state != 2
                || worker_result != Some(result)
                || worker_sequence != Some(completed)
                || self.report.completed.checked_add(1) != Some(completed)
            {
                return Err("coord_bad_report");
            }
            if self.report.index_mode {
                let (kind, input_ticket, input_id) = self.input.ok_or("coord_bad_report")?;
                if kind != 31
                    || u64::from(input_ticket) != ticket
                    || input_id.raw() != result >> 32
                    || ticket != self.report.ticket
                    || !requested
                {
                    return Err("coord_bad_report");
                }
            }
        } else if completed != self.report.completed
            || worker_result.is_some()
            || worker_sequence.is_some()
        {
            return Err("coord_bad_report");
        }
        // A non-executing event may repeat a confirmed result, never invent one.
        if state == 2
            && event != 3
            && event != 8
            && (self.report.state != "completed"
                || result != self.report.result
                || ticket != self.report.ticket)
        {
            return Err("coord_bad_report");
        }
        if self.report.index_mode {
            if event == 3 {
                let text = stream.finish(end.ok_or("coord_index_length")?, result & 0xffff)?;
                let slot = (result >> 32) as usize - 1;
                if let Some(old) = self.cache[slot]
                    && (old.result != result || old.text != text)
                {
                    return Err("coord_index_changed");
                }
                // Only publish/store after full stream, attested worker completion and both waits.
                self.cache[slot] = Some(Cached { result, text });
            } else if event == 8 {
                let (kind, input_ticket, input_id) = self.input.ok_or("coord_bad_report")?;
                if state != 2
                    || kind != 30
                    || u64::from(input_ticket) != ticket
                    || input_id.raw() != result >> 32
                    || self.cache[input_id.raw() as usize - 1]
                        .is_none_or(|entry| entry.result != result)
                {
                    return Err("coord_bad_report");
                }
                self.report.hits = self.report.hits.checked_add(1).ok_or("coord_cache_limit")?;
            }
            if let Some(goal) = goal_publication {
                self.goal = Some(goal);
            }
            if event == 1 || event == 3 || event == 9 {
                self.report.active_reused = false;
            } else if event == 8 {
                self.report.active_reused = true;
            }
            self.report.cached = self.cache.iter().filter(|entry| entry.is_some()).count() as u64;
            self.report.output_bytes = if state == 2 {
                self.cache[(result >> 32) as usize - 1]
                    .ok_or("coord_bad_report")?
                    .text
                    .len()
            } else {
                0
            };
        }
        self.report.state = ["idle", "awaiting_proposal", "completed"][state as usize];
        self.report.event = [
            "started",
            "observed",
            "duplicate",
            "executed",
            "stale_proposal",
            "wrong_record",
            "already_completed",
            "no_goal",
            "reused",
            "goal_changed",
            "not_required",
        ][event as usize];
        self.report.ticket = ticket;
        self.report.result = result;
        self.report.completed = completed;
        Ok(())
    }
}
