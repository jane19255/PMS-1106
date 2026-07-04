//! Appointment feature module.
//!
//! Groups appointment HTTP handlers, interval scheduling helpers, and the scheduler
//! used to detect booking conflicts.
pub mod handlers;
pub mod interval;
pub mod interval_tree;
pub mod scheduler;