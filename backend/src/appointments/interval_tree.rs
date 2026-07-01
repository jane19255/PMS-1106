use chrono::{DateTime, Utc};

use super::interval::AppointmentInterval;

#[derive(Clone)]
pub struct IntervalNode {
    pub interval: AppointmentInterval,
    pub max_end: DateTime<Utc>,
    pub left: Option<Box<IntervalNode>>,
    pub right: Option<Box<IntervalNode>>,
}

impl IntervalNode {
    pub fn new(interval: AppointmentInterval) -> Self {
        Self {
            max_end: interval.end,
            interval,
            left: None,
            right: None,
        }
    }
}

pub struct IntervalTree {
    root: Option<Box<IntervalNode>>,
}

impl IntervalTree {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, interval: AppointmentInterval) {
        self.root = Self::insert_node(self.root.take(), interval);
    }

    fn insert_node(
        node: Option<Box<IntervalNode>>,
        interval: AppointmentInterval,
    ) -> Option<Box<IntervalNode>> {
        match node {
            None => Some(Box::new(IntervalNode::new(interval))),
            Some(mut current) => {
                if interval.start < current.interval.start {
                    current.left = Self::insert_node(current.left.take(), interval.clone());
                } else {
                    current.right = Self::insert_node(current.right.take(), interval.clone());
                }

                if interval.end > current.max_end {
                    current.max_end = interval.end;
                }

                Some(current)
            }
        }
    }

    pub fn find_overlap(&self, target: &AppointmentInterval) -> Option<AppointmentInterval> {
        Self::find_overlap_node(&self.root, target)
    }

    fn find_overlap_node(
        node: &Option<Box<IntervalNode>>,
        target: &AppointmentInterval,
    ) -> Option<AppointmentInterval> {
        match node {
            None => None,
            Some(current) => {
                if current.interval.overlaps(target) {
                    return Some(current.interval.clone());
                }

                if let Some(left) = &current.left {
                    if left.max_end >= target.start {
                        // The left branch may only touch the requested start time.
                        // If it has no overlap, the right branch still needs checking.
                        if let Some(overlap) = Self::find_overlap_node(&current.left, target) {
                            return Some(overlap);
                        }
                    }
                }

                Self::find_overlap_node(&current.right, target)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn interval(id: &str, hour: u32, minute: u32, duration: i64) -> AppointmentInterval {
        AppointmentInterval::new(
            id.to_string(),
            Utc.with_ymd_and_hms(2026, 7, 1, hour, minute, 0)
                .single()
                .unwrap(),
            duration,
        )
    }

    #[test]
    fn checks_right_branch_when_left_branch_has_no_overlap() {
        let mut tree = IntervalTree::new();
        tree.insert(interval("root", 10, 0, 60));
        tree.insert(interval("left", 8, 0, 240));
        tree.insert(interval("right", 11, 30, 120));

        let requested = interval("requested", 12, 0, 30);
        let overlap = tree.find_overlap(&requested);

        assert_eq!(overlap.map(|item| item.id), Some("right".to_string()));
    }

    #[test]
    fn allows_appointment_to_start_when_previous_one_ends() {
        let mut tree = IntervalTree::new();
        tree.insert(interval("existing", 9, 0, 30));

        let requested = interval("requested", 9, 30, 30);

        assert!(tree.find_overlap(&requested).is_none());
    }

    #[test]
    fn finds_a_normal_overlapping_appointment() {
        let mut tree = IntervalTree::new();
        tree.insert(interval("existing", 9, 0, 60));

        let requested = interval("requested", 9, 30, 30);

        assert_eq!(
            tree.find_overlap(&requested).map(|item| item.id),
            Some("existing".to_string())
        );
    }
}
