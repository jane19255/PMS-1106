use chrono::{DateTime, Utc};

/// Represents a time interval with start and end timestamps.
#[derive(Debug, Clone)]
pub struct TimeInterval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeInterval {
    /// Returns true if this interval overlaps with another.
    pub fn overlaps(&self, other: &TimeInterval) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// A simple interval tree structure for overlap detection.
/// This implementation stores intervals in a vector and checks overlaps linearly.
#[derive(Debug, Clone, Default)]
pub struct IntervalTree {
    intervals: Vec<TimeInterval>,
}

impl IntervalTree {
    /// Creates a new empty interval tree.
    pub fn new() -> Self {
        Self { intervals: Vec::new() }
    }

    /// Inserts a new interval into the tree.
    pub fn insert(&mut self, interval: TimeInterval) {
        self.intervals.push(interval);
    }

    /// Checks if the given target interval overlaps with any stored interval.
    pub fn has_overlap(&self, target: &TimeInterval) -> bool {
        self.intervals.iter().any(|interval| interval.overlaps(target))
    }
}