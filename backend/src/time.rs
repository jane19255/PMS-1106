//! Singapore time helpers shared by clinical workflows.
//!
//! The application stores timestamps in UTC, but clinic-facing date rules should
//! follow Singapore local time for dashboard, appointment, staff, and patient logic.
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};

pub fn singapore_offset() -> FixedOffset {
    FixedOffset::east_opt(8 * 60 * 60).expect("Singapore offset is valid")
}

pub fn singapore_now() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&singapore_offset())
}

pub fn singapore_today() -> NaiveDate {
    // Clinic dates must follow Singapore time instead of the server's UTC date.
    singapore_now().date_naive()
}

pub fn singapore_day_bounds(date: NaiveDate) -> (DateTime<Utc>, DateTime<Utc>) {
    // Supabase stores UTC instants, so Singapore midnight is 16:00 UTC the day before.
    let start = singapore_offset()
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).expect("midnight is valid"))
        .single()
        .expect("Singapore has no daylight-saving ambiguity")
        .with_timezone(&Utc);
    (start, start + chrono::Duration::days(1))
}
