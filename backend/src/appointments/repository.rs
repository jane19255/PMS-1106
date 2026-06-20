use crate::db::SupabaseRestDb;

#[derive(Clone)]
pub struct AppointmentRepository {
    pub db: SupabaseRestDb,
}

impl AppointmentRepository {
    pub fn new(db: SupabaseRestDb) -> Self {
        Self { db }
    }
}