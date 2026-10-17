//! Generic snapshot-based undo/redo history.
//!
//! [`UndoHistory`] stores immutable snapshots of a piece of application state
//! (a text buffer, a form model, a canvas document, …) on two stacks. Push a
//! committed snapshot after each discrete edit; [`UndoHistory::undo`] walks
//! backward and [`UndoHistory::redo`] forward. Making a fresh edit after
//! undoing truncates the redo branch, matching the standard linear-edit model.
//!
//! State only needs [`Clone`]; snapshots are cheap when the state is an
//! `Rc`/arc-based or copy-on-write model. A capacity cap bounds memory: when
//! the past grows past it the oldest snapshot is dropped.

/// Linear undo/redo stack over snapshots of `T`.
#[derive(Clone, Debug)]
pub struct UndoHistory<T> {
    past: Vec<T>,
    present: T,
    future: Vec<T>,
    capacity: usize,
}

impl<T: Clone> UndoHistory<T> {
    /// Start a history at an initial (committed) state with unlimited capacity.
    pub fn new(initial: T) -> Self {
        Self::with_capacity(initial, usize::MAX)
    }

    /// Start a history that retains at most `capacity` past snapshots.
    ///
    /// A capacity of zero stores no past, so undo is always a no-op.
    pub fn with_capacity(initial: T, capacity: usize) -> Self {
        UndoHistory {
            past: Vec::new(),
            present: initial,
            future: Vec::new(),
            capacity,
        }
    }

    /// The current state.
    pub fn present(&self) -> &T {
        &self.present
    }

    /// Whether an earlier snapshot can be restored.
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    /// Whether a later snapshot (from a prior undo) can be restored.
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Commit a new state as the latest edit. The current state becomes the
    /// most recent past snapshot and any redo branch is discarded.
    pub fn push(&mut self, next: T) {
        let previous = std::mem::replace(&mut self.present, next);
        self.past.push(previous);
        self.future.clear();
        if self.capacity != usize::MAX && self.past.len() > self.capacity {
            let excess = self.past.len() - self.capacity;
            self.past.drain(0..excess);
        }
    }

    /// Move one step backward, returning the restored state. Returns `None`
    /// (without changing the history) when there is nothing to undo.
    pub fn undo(&mut self) -> Option<&T> {
        let previous = self.past.pop()?;
        let current = std::mem::replace(&mut self.present, previous);
        self.future.push(current);
        Some(&self.present)
    }

    /// Move one step forward after an undo, returning the restored state.
    /// Returns `None` when there is nothing to redo.
    pub fn redo(&mut self) -> Option<&T> {
        let next = self.future.pop()?;
        let current = std::mem::replace(&mut self.present, next);
        self.past.push(current);
        Some(&self.present)
    }

    /// Number of committed snapshots available to undo into.
    pub fn undo_depth(&self) -> usize {
        self.past.len()
    }

    /// Number of snapshots available to redo into.
    pub fn redo_depth(&self) -> usize {
        self.future.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_undo_redo_round_trip() {
        let mut h = UndoHistory::new(0i32);
        h.push(1);
        h.push(2);
        assert_eq!(h.present(), &2);
        assert!(h.can_undo());

        assert_eq!(h.undo(), Some(&1));
        assert_eq!(h.undo(), Some(&0));
        assert!(!h.can_undo());
        assert_eq!(h.undo(), None);

        assert_eq!(h.redo(), Some(&1));
        assert_eq!(h.redo(), Some(&2));
        assert!(!h.can_redo());
    }

    #[test]
    fn push_after_undo_clears_redo_branch() {
        let mut h = UndoHistory::new("a".to_string());
        h.push("b".to_string());
        h.push("c".to_string());
        h.undo(); // back to b
        h.push("d".to_string()); // branches off b
        assert!(!h.can_redo(), "redo branch must be discarded on new edit");
        assert_eq!(h.present(), "d");
        assert_eq!(h.undo(), Some(&"b".to_string()));
    }

    #[test]
    fn capacity_drops_oldest_snapshots() {
        let mut h = UndoHistory::with_capacity(0, 2);
        h.push(1);
        h.push(2);
        h.push(3);
        assert_eq!(h.undo_depth(), 2, "only two past snapshots are retained");
        h.undo();
        assert_eq!(h.present(), &2);
        h.undo();
        assert_eq!(h.present(), &1);
        assert!(!h.can_undo(), "the dropped snapshot of 0 is gone");
    }
}
