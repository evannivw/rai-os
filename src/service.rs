//! Guest service use case: validated input, bounded execution, checked completion.
use crate::{
    mmu::Mmu,
    records::RecordId,
    schedule::{self, RunMode, RunReport},
    timer::Timer,
};
use core::fmt;
pub struct Report {
    id: RecordId,
    bytes: u64,
    tokens: u64,
    execution: RunReport,
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "record={}; bytes={}; tokens={}; worker=EL0; {}",
            self.id, self.bytes, self.tokens, self.execution
        )
    }
}
pub fn count(id: RecordId, timer: &mut Timer, mmu: &Mmu) -> Result<Report, &'static str> {
    let execution = schedule::run(timer, mmu, RunMode::RecordCount(id))?;
    let [(client_code, result), (worker_code, worker_result)] =
        execution.exits().ok_or("service_failed")?;
    if worker_code == 4 {
        return Err("service_request_rejected");
    }
    if client_code == 2 {
        return Err("service_reply_rejected");
    }
    if client_code != 0 || worker_code != 0 || result != worker_result {
        return Err("service_failed");
    }
    let bytes = (result >> 16) & 0xffff;
    let tokens = result & 0xffff;
    if result >> 32 != id.raw() || bytes != id.bytes() || tokens > bytes {
        return Err("service_bad_result");
    }
    Ok(Report {
        id,
        bytes,
        tokens,
        execution,
    })
}
