//! One record -> exact EL0 index text, published only after successful termination.
use crate::{records::RecordId, timer::Timer, user};
use core::fmt;
pub struct Report {
    id: RecordId,
    execution: user::Report,
}
pub fn build(id: RecordId, timer: &mut Timer) -> Result<Report, &'static str> {
    let execution = user::run(timer, user::Program::RecordIndex(id))?;
    if execution.successful_output()?.is_empty() {
        return Err("index_bad_output");
    }
    Ok(Report { id, execution })
}
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = self.execution.successful_output().map_err(|_| fmt::Error)?;
        write!(
            f,
            "record={}; bytes={}; worker=EL0; state=completed; index={output}",
            self.id,
            self.id.bytes()
        )
    }
}
