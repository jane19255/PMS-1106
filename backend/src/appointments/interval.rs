use chrono::{DateTime, Duration, Utc};

#[derive(Clone, Debug)]
pub struct AppointmentInterval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl AppointmentInterval {
    pub fn new(start: DateTime<Utc>, duration_minutes: i64) -> Self {
        Self {
            start,
            end: start + Duration::minutes(duration_minutes),
        }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}
