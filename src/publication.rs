//! In-memory publication lifecycle. No UART, provider, clock, or scheduling policy.
//! A commit acknowledges an independent check by the trusted host; this module
//! enforces freshness, not proof that the host actually ran that check.
use crate::bundle::Artifact;
use core::fmt;

pub struct Store {
    target: u32,
    candidate: Option<u32>,
    last: Option<Artifact>,
    failure: Option<&'static str>,
}
#[derive(Clone, Copy)]
pub struct Status {
    target: u32,
    published: u32,
    candidate: u32,
    state: &'static str,
    availability: &'static str,
    reason: &'static str,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "target_version={}; published_version={}; candidate_ticket={}; state={}; availability={}; reason={}",
            self.target, self.published, self.candidate, self.state, self.availability, self.reason
        )
    }
}
impl Store {
    pub const fn new() -> Self {
        Self {
            target: 0,
            candidate: None,
            last: None,
            failure: None,
        }
    }
    /// Called only after the guest coordinator has attested the new required set.
    pub fn begin(&mut self, generation: u32) {
        self.target = generation;
        self.candidate = None;
        self.failure = None;
    }
    pub fn can_prepare(&self, generation: u32) -> Result<(), &'static str> {
        if generation != self.target || generation == 0 {
            return Err("publication_stale_goal");
        }
        if self
            .last
            .as_ref()
            .is_some_and(|a| a.status.generation == generation)
        {
            return Err("publication_already_current");
        }
        Ok(())
    }
    /// The coordinator supplies a unique nonzero boot-scoped ticket after checking
    /// completeness. Immutable revision slots cannot change while this goal lives.
    pub fn prepare(&mut self, ticket: u32) {
        self.candidate = Some(ticket);
        self.failure = None;
    }
    pub fn check_ticket(&self, ticket: u32) -> Result<(), &'static str> {
        let expected = self.candidate.ok_or("publication_no_candidate")?;
        if ticket != expected {
            return Err("publication_stale_candidate");
        }
        Ok(())
    }
    pub fn commit(&mut self, ticket: u32, artifact: Artifact) -> Result<Status, &'static str> {
        self.check_ticket(ticket)?;
        if artifact.status.generation != self.target || !artifact.status.ready() {
            return Err("publication_stale_goal");
        }
        self.last = Some(artifact);
        self.candidate = None;
        self.failure = None;
        Ok(self.status())
    }
    pub fn reject(&mut self, ticket: u32) -> Result<Status, &'static str> {
        self.check_ticket(ticket)?;
        self.fail("validation_rejected");
        Ok(self.status())
    }
    /// Fatal service failures discard candidate authority but preserve owned output.
    pub fn fail(&mut self, reason: &'static str) {
        self.candidate = None;
        self.failure = Some(reason);
    }
    pub fn read(&self) -> Result<Artifact, &'static str> {
        self.last.ok_or("publication_unavailable")
    }
    pub fn status(&self) -> Status {
        let published = self.last.as_ref().map_or(0, |a| a.status.generation);
        let current = published != 0 && published == self.target && self.failure.is_none();
        Status {
            target: self.target,
            published,
            candidate: self.candidate.unwrap_or(0),
            state: if self.failure.is_some() {
                "blocked"
            } else if self.candidate.is_some() {
                "validating"
            } else if current {
                "current"
            } else if self.target != 0 {
                "building"
            } else {
                "empty"
            },
            availability: if published == 0 {
                "none"
            } else if current {
                "current"
            } else {
                "historical"
            },
            reason: self.failure.unwrap_or("-"),
        }
    }
}
