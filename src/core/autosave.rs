//! Save tracking: coalesces edits, exposes an honest persisted-revision state.
//!
//! The UI owns the note; this module only decides *when* to persist it and what the header
//! badge should say. A save is considered done only when the writer acknowledges the revision.

use std::time::{Duration, Instant};

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveStatus {
    Saved,
    Saving,
    Error,
}

pub struct SaveTracker {
    dirty_since: Option<Instant>,
    /// The write being waited for. Acknowledgements are matched against it rather than against
    /// the active note, because the user may have switched notes while the write was queued.
    awaited: Option<(Uuid, i64)>,
    error: Option<String>,
    debounce: Duration,
    retry_at: Option<Instant>,
    last_persisted_revision: i64,
}

impl Default for SaveTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveTracker {
    pub fn new() -> Self {
        Self {
            dirty_since: None,
            awaited: None,
            error: None,
            debounce: Duration::from_millis(200),
            retry_at: None,
            last_persisted_revision: 0,
        }
    }

    /// Force a shorter/longer coalescing window (tests use a zero window).
    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }

    pub fn mark_dirty(&mut self, now: Instant) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(now);
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty_since.is_some()
    }

    pub fn is_inflight(&self) -> bool {
        self.awaited.is_some()
    }

    pub fn status(&self) -> SaveStatus {
        if self.error.is_some() {
            SaveStatus::Error
        } else if self.dirty_since.is_some() || self.is_inflight() {
            SaveStatus::Saving
        } else {
            SaveStatus::Saved
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Debounce elapsed and nothing is in flight: the caller should send the note now.
    pub fn should_save(&self, now: Instant) -> bool {
        if self.is_inflight() {
            return false;
        }
        if let Some(retry_at) = self.retry_at {
            if now < retry_at {
                return false;
            }
        }
        self.dirty_since
            .map(|since| now.duration_since(since) >= self.debounce)
            .unwrap_or(false)
    }

    /// Call immediately before sending the note to the writer.
    pub fn begin_save(&mut self, note_id: Uuid, revision: i64) {
        self.awaited = Some((note_id, revision));
        self.dirty_since = None;
        self.error = None;
        self.retry_at = None;
    }

    /// The writer persisted a revision. Writes other than the awaited one are ignored.
    pub fn ack(&mut self, note_id: Uuid, revision: i64) {
        if self.awaited == Some((note_id, revision)) {
            self.awaited = None;
            self.last_persisted_revision = revision;
            self.error = None;
        }
    }

    /// The awaited write failed for a note that is no longer active: stop waiting for it, so the
    /// note being edited now can still be saved.
    pub fn abandon(&mut self, note_id: Uuid, revision: i64) {
        if self.awaited == Some((note_id, revision)) {
            self.awaited = None;
        }
    }

    pub fn fail(&mut self, message: impl Into<String>, now: Instant) {
        self.awaited = None;
        self.error = Some(message.into());
        // Retry automatically, keeping the content in memory.
        self.retry_at = Some(now + Duration::from_secs(2));
        if self.dirty_since.is_none() {
            self.dirty_since = Some(now);
        }
    }

    pub fn last_persisted_revision(&self) -> i64 {
        self.last_persisted_revision
    }

    /// Blocking flush decision helper: returns true when a save is required right now.
    pub fn flush_now(&mut self) -> bool {
        if self.dirty_since.is_some() && !self.is_inflight() {
            self.dirty_since = Some(Instant::now() - self.debounce);
            true
        } else {
            self.is_inflight()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounce_waits_before_saving() {
        let mut tracker = SaveTracker::new().with_debounce(Duration::from_millis(50));
        let start = Instant::now();
        tracker.mark_dirty(start);
        assert!(!tracker.should_save(start));
        assert!(tracker.should_save(start + Duration::from_millis(60)));
    }

    #[test]
    fn inflight_blocks_a_second_save() {
        let mut tracker = SaveTracker::new().with_debounce(Duration::ZERO);
        let now = Instant::now();
        let note = Uuid::new_v4();
        tracker.mark_dirty(now);
        assert!(tracker.should_save(now));
        tracker.begin_save(note, 1);
        assert!(!tracker.should_save(now + Duration::from_secs(1)));
        tracker.ack(note, 1);
        assert_eq!(tracker.status(), SaveStatus::Saved);
    }

    #[test]
    fn failure_is_reported_and_retried() {
        let mut tracker = SaveTracker::new().with_debounce(Duration::ZERO);
        let now = Instant::now();
        tracker.mark_dirty(now);
        tracker.begin_save(Uuid::new_v4(), 1);
        tracker.fail("disk full", now);
        assert_eq!(tracker.status(), SaveStatus::Error);
        assert!(!tracker.should_save(now + Duration::from_millis(100)));
        assert!(tracker.should_save(now + Duration::from_secs(3)));
    }

    #[test]
    fn edits_during_save_keep_the_note_dirty() {
        let mut tracker = SaveTracker::new().with_debounce(Duration::ZERO);
        let now = Instant::now();
        let note = Uuid::new_v4();
        tracker.mark_dirty(now);
        tracker.begin_save(note, 1);
        tracker.mark_dirty(now);
        tracker.ack(note, 1);
        assert_eq!(tracker.status(), SaveStatus::Saving);
    }

    #[test]
    fn switching_notes_during_a_save_does_not_block_later_saves() {
        let mut tracker = SaveTracker::new().with_debounce(Duration::ZERO);
        let now = Instant::now();
        let previous_note = Uuid::new_v4();
        tracker.begin_save(previous_note, 7);

        // The user is now editing another note when the acknowledgement arrives.
        tracker.mark_dirty(now);
        assert!(!tracker.should_save(now));
        tracker.ack(previous_note, 7);
        assert!(tracker.should_save(now));
    }

    #[test]
    fn only_the_awaited_revision_ends_the_wait() {
        let mut tracker = SaveTracker::new();
        let note = Uuid::new_v4();
        tracker.begin_save(note, 1);
        tracker.begin_save(note, 2);
        tracker.ack(note, 1);
        assert!(tracker.is_inflight());
        tracker.ack(Uuid::new_v4(), 2);
        assert!(tracker.is_inflight());
        tracker.ack(note, 2);
        assert_eq!(tracker.status(), SaveStatus::Saved);
    }

    #[test]
    fn a_failed_write_for_another_note_stops_the_wait() {
        let mut tracker = SaveTracker::new();
        let note = Uuid::new_v4();
        tracker.begin_save(note, 3);
        tracker.abandon(note, 2);
        assert!(tracker.is_inflight());
        tracker.abandon(note, 3);
        assert!(!tracker.is_inflight());
    }
}
