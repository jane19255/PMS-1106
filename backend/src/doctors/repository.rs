use super::models::{Doctor, DoctorSchedule, DoctorStatus};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    StorageUnavailable,
}

pub trait DoctorRepository: Send + Sync {
    fn create(&self, doctor: Doctor) -> Result<Doctor, RepositoryError>;
    fn find_by_id(&self, doctor_id: &str) -> Result<Doctor, RepositoryError>;
    fn list(&self) -> Result<Vec<Doctor>, RepositoryError>;
    fn update(&self, doctor: Doctor) -> Result<Doctor, RepositoryError>;
    fn delete(&self, doctor_id: &str) -> Result<(), RepositoryError>;
    fn create_schedule(&self, schedule: DoctorSchedule) -> Result<DoctorSchedule, RepositoryError>;
    fn list_schedules(&self, doctor_id: &str) -> Result<Vec<DoctorSchedule>, RepositoryError>;
    fn delete_schedule(&self, schedule_id: &str) -> Result<(), RepositoryError>;
}

pub struct InMemoryDoctorRepository {
    doctors: Mutex<HashMap<String, Doctor>>,
    schedules: Mutex<HashMap<String, DoctorSchedule>>,
}

impl Default for InMemoryDoctorRepository {
    fn default() -> Self {
        let doctors = [
            Doctor {
                id: "DOC-RICHARD".to_string(),
                staff_id: "STAFF-RICHARD".to_string(),
                license_number: "M-RICHARD".to_string(),
                name: "Dr. Richard".to_string(),
                specialization: "General Medicine".to_string(),
                contact_number: "80000001".to_string(),
                email: "richard@cscare.local".to_string(),
                status: DoctorStatus::Available,
            },
            Doctor {
                id: "DOC-LEE".to_string(),
                staff_id: "STAFF-LEE".to_string(),
                license_number: "M-LEE".to_string(),
                name: "Dr. Lee".to_string(),
                specialization: "Family Medicine".to_string(),
                contact_number: "80000002".to_string(),
                email: "lee@cscare.local".to_string(),
                status: DoctorStatus::Available,
            },
            Doctor {
                id: "DOC-WONG".to_string(),
                staff_id: "STAFF-WONG".to_string(),
                license_number: "M-WONG".to_string(),
                name: "Dr. Wong".to_string(),
                specialization: "Internal Medicine".to_string(),
                contact_number: "80000003".to_string(),
                email: "wong@cscare.local".to_string(),
                status: DoctorStatus::Available,
            },
        ]
        .into_iter()
        .map(|doctor| (doctor.id.clone(), doctor))
        .collect();

        Self {
            doctors: Mutex::new(doctors),
            schedules: Mutex::new(HashMap::new()),
        }
    }
}

impl DoctorRepository for InMemoryDoctorRepository {
    fn create(&self, doctor: Doctor) -> Result<Doctor, RepositoryError> {
        let mut doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;
        doctors.insert(doctor.id.clone(), doctor.clone());
        Ok(doctor)
    }

    fn find_by_id(&self, doctor_id: &str) -> Result<Doctor, RepositoryError> {
        let doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        doctors
            .get(doctor_id)
            .cloned()
            .ok_or(RepositoryError::NotFound)
    }

    fn list(&self) -> Result<Vec<Doctor>, RepositoryError> {
        let doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        let mut doctor_list: Vec<Doctor> = doctors.values().cloned().collect();
        doctor_list.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(doctor_list)
    }

    fn update(&self, doctor: Doctor) -> Result<Doctor, RepositoryError> {
        let mut doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if !doctors.contains_key(&doctor.id) {
            return Err(RepositoryError::NotFound);
        }

        doctors.insert(doctor.id.clone(), doctor.clone());
        Ok(doctor)
    }

    fn delete(&self, doctor_id: &str) -> Result<(), RepositoryError> {
        let mut doctors = self
            .doctors
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        doctors
            .remove(doctor_id)
            .map(|_| ())
            .ok_or(RepositoryError::NotFound)?;

        let mut schedules = self
            .schedules
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;
        schedules.retain(|_, schedule| schedule.doctor_id != doctor_id);
        Ok(())
    }

    fn create_schedule(&self, schedule: DoctorSchedule) -> Result<DoctorSchedule, RepositoryError> {
        self.find_by_id(&schedule.doctor_id)?;

        let mut schedules = self
            .schedules
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;
        schedules.insert(schedule.id.clone(), schedule.clone());
        Ok(schedule)
    }

    fn list_schedules(&self, doctor_id: &str) -> Result<Vec<DoctorSchedule>, RepositoryError> {
        self.find_by_id(doctor_id)?;

        let schedules = self
            .schedules
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        let mut schedule_list: Vec<DoctorSchedule> = schedules
            .values()
            .filter(|schedule| schedule.doctor_id == doctor_id)
            .cloned()
            .collect();
        schedule_list.sort_by(|left, right| left.start_time.cmp(&right.start_time));
        Ok(schedule_list)
    }

    fn delete_schedule(&self, schedule_id: &str) -> Result<(), RepositoryError> {
        let mut schedules = self
            .schedules
            .lock()
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        schedules
            .remove(schedule_id)
            .map(|_| ())
            .ok_or(RepositoryError::NotFound)
    }
}
