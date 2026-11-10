//! Notices when the system was asleep, so timers can be pushed back by the
//! time nothing ran.
//!
//! A sleeping system runs neither the dummies nor the detecting client, so
//! that time does not count towards anything, yet wall-clock deadlines keep
//! passing. `Instant` is no help: its documentation states that it may or
//! may not count time spent suspended, depending on the platform. Instead,
//! the timer loop marks the wall-clock time at
//! the end of each pass; the next pass should come about one period later,
//! so a much longer gap means the system was suspended or its clock jumped
//! forward. Either way the games did not run for that time. Work done inside
//! a pass, such as downloading the game list, is not counted as a gap, since
//! the games keep running meanwhile.

use std::time::{Duration, SystemTime};

/// Gap beyond the loop period that counts as missed time. Larger than any
/// scheduling delay, much smaller than a sleep.
const TOLERANCE: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct SuspendDetector {
    period: Duration,
    last_pass: Option<SystemTime>,
}

impl SuspendDetector {
    /// `period` is how often the loop runs.
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            last_pass: None,
        }
    }

    /// Records that a pass ended at `now`.
    pub fn pass_ended(&mut self, now: SystemTime) {
        self.last_pass = Some(now);
    }

    /// The time missed since the last pass ended, if the pass starting at
    /// `now` comes too late. A clock that went back reports nothing: timers
    /// then run longer, never shorter.
    pub fn missed(&self, now: SystemTime) -> Option<Duration> {
        let gap = now.duration_since(self.last_pass?).ok()?;
        (gap > self.period + TOLERANCE).then(|| gap - self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    fn at(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn reports_nothing_before_the_first_pass() {
        let detector = SuspendDetector::new(Duration::from_secs(1));
        assert_eq!(detector.missed(at(1_000)), None);
    }

    #[test]
    fn ignores_ordinary_delays() {
        let mut detector = SuspendDetector::new(Duration::from_secs(1));
        detector.pass_ended(at(100));
        assert_eq!(detector.missed(at(101)), None);
        assert_eq!(detector.missed(at(106)), None);
    }

    #[test]
    fn reports_a_sleep_minus_the_period() {
        let mut detector = SuspendDetector::new(Duration::from_secs(1));
        detector.pass_ended(at(100));
        assert_eq!(detector.missed(at(1_901)), Some(Duration::from_secs(1_800)));
    }

    #[test]
    fn ignores_a_clock_that_went_back() {
        let mut detector = SuspendDetector::new(Duration::from_secs(1));
        detector.pass_ended(at(100));
        assert_eq!(detector.missed(at(50)), None);
    }
}
