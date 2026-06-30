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
                    current.left =
                        Self::insert_node(current.left.take(), interval.clone());
                } else {
                    current.right =
                        Self::insert_node(current.right.take(), interval.clone());
                }

                if interval.end > current.max_end {
                    current.max_end = interval.end;
                }

                Some(current)
            }
        }
    }

    pub fn find_overlap(
        &self,
        target: &AppointmentInterval,
    ) -> Option<AppointmentInterval> {
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
                        return Self::find_overlap_node(
                            &current.left,
                            target,
                        );
                    }
                }

                Self::find_overlap_node(
                    &current.right,
                    target,
                )
            }
        }
    }
}