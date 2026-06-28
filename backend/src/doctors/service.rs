use super::models::{DayOfWeek, Doctor, DoctorSchedule, DoctorStatus};
use super::repository::{DoctorRepository, RepositoryError};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum DoctorError {
    InvalidInput(String),
    DoctorNotFound,
    StorageUnavailable,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateDoctorForm {
    pub staff_id: String,
    pub license_number: String,
    pub name: String,
    pub specialization: String,
    pub contact_number: String,
    pub email: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateDoctorForm {
    pub staff_id: String,
    pub license_number: String,
    pub name: String,
    pub specialization: String,
    pub contact_number: String,
    pub email: String,
    pub status: DoctorStatus,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateDoctorScheduleForm {
    pub day_of_week: DayOfWeek,
    pub start_time: String,
    pub end_time: String,
}

pub struct DoctorService {
    doctor_repository: Arc<dyn DoctorRepository>,
    deleted_doctors: Mutex<HashMap<String, DeletedDoctor>>,
}

struct DeletedDoctor {
    doctor: Doctor,
    schedules: Vec<DoctorSchedule>,
}

impl DoctorService {
    pub fn new(doctor_repository: Arc<dyn DoctorRepository>) -> Self {
        Self {
            doctor_repository,
            deleted_doctors: Mutex::new(HashMap::new()),
        }
    }

    pub async fn create_doctor(&self, form: CreateDoctorForm) -> Result<Doctor, DoctorError> {
        self.validate_doctor_fields(
            &form.staff_id,
            &form.license_number,
            &form.name,
            &form.specialization,
            &form.contact_number,
            &form.email,
        )?;

        let doctor = Doctor {
            id: format!("DOC-{}", Uuid::new_v4()),
            staff_id: form.staff_id.trim().to_string(),
            license_number: form.license_number.trim().to_string(),
            name: Self::normalize_doctor_name(&form.name),
            specialization: form.specialization.trim().to_string(),
            contact_number: form.contact_number.trim().to_string(),
            email: form.email.trim().to_string(),
            status: DoctorStatus::Available,
        };

        self.doctor_repository
            .create(doctor)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn list_doctors(&self) -> Result<Vec<Doctor>, DoctorError> {
        self.doctor_repository
            .list()
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn find_doctor(&self, doctor_id: &str) -> Result<Doctor, DoctorError> {
        if doctor_id.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "Doctor ID is required".to_string(),
            ));
        }

        self.doctor_repository
            .find_by_id(doctor_id.trim())
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn update_doctor(
        &self,
        doctor_id: &str,
        form: UpdateDoctorForm,
    ) -> Result<Doctor, DoctorError> {
        let existing = self.find_doctor(doctor_id).await?;
        self.validate_doctor_fields(
            &form.staff_id,
            &form.license_number,
            &form.name,
            &form.specialization,
            &form.contact_number,
            &form.email,
        )?;

        let doctor = Doctor {
            id: existing.id,
            staff_id: form.staff_id.trim().to_string(),
            license_number: form.license_number.trim().to_string(),
            name: Self::normalize_doctor_name(&form.name),
            specialization: form.specialization.trim().to_string(),
            contact_number: form.contact_number.trim().to_string(),
            email: form.email.trim().to_string(),
            status: form.status,
        };

        self.doctor_repository
            .update(doctor)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn delete_doctor(&self, doctor_id: &str) -> Result<(), DoctorError> {
        if doctor_id.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "Doctor ID is required".to_string(),
            ));
        }

        let doctor_id = doctor_id.trim();
        let doctor = self.find_doctor(doctor_id).await?;
        let schedules = self.list_schedules(doctor_id).await?;

        self.doctor_repository
            .delete(doctor_id)
            .await
            .map_err(Self::map_repository_error)?;

        let mut deleted_doctors = self
            .deleted_doctors
            .lock()
            .map_err(|_| DoctorError::StorageUnavailable)?;
        deleted_doctors.insert(doctor_id.to_string(), DeletedDoctor { doctor, schedules });

        Ok(())
    }

    pub async fn undo_delete_doctor(&self, doctor_id: &str) -> Result<(), DoctorError> {
        if doctor_id.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "Doctor ID is required".to_string(),
            ));
        }

        let doctor_id = doctor_id.trim();
        let deleted_doctor = {
            let mut deleted_doctors = self
                .deleted_doctors
                .lock()
                .map_err(|_| DoctorError::StorageUnavailable)?;
            deleted_doctors
                .remove(doctor_id)
                .ok_or(DoctorError::DoctorNotFound)?
        };

        self.doctor_repository
            .create(deleted_doctor.doctor)
            .await
            .map_err(Self::map_repository_error)?;

        for schedule in deleted_doctor.schedules {
            self.doctor_repository
                .create_schedule(schedule)
                .await
                .map_err(Self::map_repository_error)?;
        }

        Ok(())
    }

    pub async fn create_schedule(
        &self,
        doctor_id: &str,
        form: CreateDoctorScheduleForm,
    ) -> Result<DoctorSchedule, DoctorError> {
        let doctor = self.find_doctor(doctor_id).await?;
        self.validate_schedule(&form)?;

        let schedule = DoctorSchedule {
            id: format!("DSCH-{}", Uuid::new_v4()),
            doctor_id: doctor.id,
            day_of_week: form.day_of_week,
            start_time: form.start_time.trim().to_string(),
            end_time: form.end_time.trim().to_string(),
        };

        self.doctor_repository
            .create_schedule(schedule)
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn list_schedules(&self, doctor_id: &str) -> Result<Vec<DoctorSchedule>, DoctorError> {
        if doctor_id.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "Doctor ID is required".to_string(),
            ));
        }

        self.doctor_repository
            .list_schedules(doctor_id.trim())
            .await
            .map_err(Self::map_repository_error)
    }

    pub async fn delete_schedule(&self, schedule_id: &str) -> Result<(), DoctorError> {
        if schedule_id.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "Schedule ID is required".to_string(),
            ));
        }

        self.doctor_repository
            .delete_schedule(schedule_id.trim())
            .await
            .map_err(Self::map_repository_error)
    }

    fn normalize_doctor_name(name: &str) -> String {
        let trimmed = name.trim();
        let without_prefix = trimmed
            .strip_prefix("Dr. ")
            .or_else(|| trimmed.strip_prefix("Dr "))
            .or_else(|| trimmed.strip_prefix("dr. "))
            .or_else(|| trimmed.strip_prefix("dr "))
            .unwrap_or(trimmed)
            .trim();

        format!("Dr. {without_prefix}")
    }
    fn validate_doctor_fields(
        &self,
        staff_id: &str,
        license_number: &str,
        name: &str,
        specialization: &str,
        contact_number: &str,
        email: &str,
    ) -> Result<(), DoctorError> {
        if staff_id.trim().is_empty() {
            return Err(DoctorError::InvalidInput("Staff ID is required".to_string()));
        }

        if license_number.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "License number is required".to_string(),
            ));
        }

        if name.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "Doctor name is required".to_string(),
            ));
        }

        if specialization.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "Specialization is required".to_string(),
            ));
        }

        if contact_number.trim().is_empty() {
            return Err(DoctorError::InvalidInput(
                "Contact number is required".to_string(),
            ));
        }

        if email.trim().is_empty() {
            return Err(DoctorError::InvalidInput("Email is required".to_string()));
        }

        Ok(())
    }

    fn validate_schedule(&self, form: &CreateDoctorScheduleForm) -> Result<(), DoctorError> {
        let start_time = form.start_time.trim();
        let end_time = form.end_time.trim();

        if !Self::is_hh_mm(start_time) {
            return Err(DoctorError::InvalidInput(
                "Start time must use HH:MM format".to_string(),
            ));
        }

        if !Self::is_hh_mm(end_time) {
            return Err(DoctorError::InvalidInput(
                "End time must use HH:MM format".to_string(),
            ));
        }

        if start_time >= end_time {
            return Err(DoctorError::InvalidInput(
                "Start time must be before end time".to_string(),
            ));
        }

        Ok(())
    }

    fn is_hh_mm(value: &str) -> bool {
        let Some((hour, minute)) = value.split_once(':') else {
            return false;
        };

        if hour.len() != 2 || minute.len() != 2 {
            return false;
        }

        let Ok(hour) = hour.parse::<u8>() else {
            return false;
        };
        let Ok(minute) = minute.parse::<u8>() else {
            return false;
        };

        hour < 24 && minute < 60
    }

    fn map_repository_error(error: RepositoryError) -> DoctorError {
        match error {
            RepositoryError::NotFound => DoctorError::DoctorNotFound,
            RepositoryError::StorageUnavailable => DoctorError::StorageUnavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctors::repository::InMemoryDoctorRepository;

    fn service() -> DoctorService {
        DoctorService::new(Arc::new(InMemoryDoctorRepository::default()))
    }

    fn create_form() -> CreateDoctorForm {
        CreateDoctorForm {
            staff_id: "STAFF-TAN".to_string(),
            license_number: "M12345A".to_string(),
            name: "Dr. Tan".to_string(),
            specialization: "Cardiology".to_string(),
            contact_number: "81234567".to_string(),
            email: "tan@example.com".to_string(),
        }
    }

    #[actix_web::test]
    async fn creates_doctor_with_default_available_status() {
        let doctor = service().create_doctor(create_form()).await.unwrap();

        assert!(doctor.id.starts_with("DOC-"));
        assert_eq!(doctor.name, "Dr. Tan");
        assert_eq!(doctor.status, DoctorStatus::Available);
    }

    #[actix_web::test]
    async fn rejects_blank_license_number() {
        let mut form = create_form();
        form.license_number = "  ".to_string();

        let error = service().create_doctor(form).await.unwrap_err();

        assert_eq!(
            error,
            DoctorError::InvalidInput("License number is required".to_string())
        );
    }

    #[actix_web::test]
    async fn rejects_blank_staff_id() {
        let mut form = create_form();
        form.staff_id = "  ".to_string();

        let error = service().create_doctor(form).await.unwrap_err();

        assert_eq!(
            error,
            DoctorError::InvalidInput("Staff ID is required".to_string())
        );
    }

    #[actix_web::test]
    async fn rejects_blank_doctor_name() {
        let mut form = create_form();
        form.name = "  ".to_string();

        let error = service().create_doctor(form).await.unwrap_err();

        assert_eq!(
            error,
            DoctorError::InvalidInput("Doctor name is required".to_string())
        );
    }

    #[actix_web::test]
    async fn prefixes_doctor_name_when_missing() {
        let mut form = create_form();
        form.name = "Lee".to_string();

        let doctor = service().create_doctor(form).await.unwrap();

        assert_eq!(doctor.name, "Dr. Lee");
    }
    #[actix_web::test]
    async fn updates_existing_doctor_status() {
        let service = service();
        let doctor = service.create_doctor(create_form()).await.unwrap();

        let updated = service
            .update_doctor(
                &doctor.id,
                UpdateDoctorForm {
                    staff_id: doctor.staff_id.clone(),
                    license_number: "M12345A".to_string(),
                    name: "Dr. Tan".to_string(),
                    specialization: "Cardiology".to_string(),
                    contact_number: "81234567".to_string(),
                    email: "tan@example.com".to_string(),
                    status: DoctorStatus::Unavailable,
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.id, doctor.id);
        assert_eq!(updated.status, DoctorStatus::Unavailable);
    }

    #[actix_web::test]
    async fn deletes_doctor_and_schedules() {
        let service = service();
        let doctor = service.create_doctor(create_form()).await.unwrap();
        service
            .create_schedule(
                &doctor.id,
                CreateDoctorScheduleForm {
                    day_of_week: DayOfWeek::Monday,
                    start_time: "09:00".to_string(),
                    end_time: "17:00".to_string(),
                },
            )
            .await
            .unwrap();

        service.delete_doctor(&doctor.id).await.unwrap();

        assert_eq!(
            service.find_doctor(&doctor.id).await,
            Err(DoctorError::DoctorNotFound)
        );
    }

    #[actix_web::test]
    async fn restores_deleted_doctor_and_schedules() {
        let service = service();
        let doctor = service.create_doctor(create_form()).await.unwrap();
        service
            .create_schedule(
                &doctor.id,
                CreateDoctorScheduleForm {
                    day_of_week: DayOfWeek::Monday,
                    start_time: "09:00".to_string(),
                    end_time: "17:00".to_string(),
                },
            )
            .await
            .unwrap();

        service.delete_doctor(&doctor.id).await.unwrap();
        service.undo_delete_doctor(&doctor.id).await.unwrap();

        assert_eq!(service.find_doctor(&doctor.id).await.unwrap().name, doctor.name);
        assert_eq!(service.list_schedules(&doctor.id).await.unwrap().len(), 1);
    }

    #[actix_web::test]
    async fn rejects_invalid_schedule_time_range() {
        let service = service();
        let doctor = service.create_doctor(create_form()).await.unwrap();

        let error = service
            .create_schedule(
                &doctor.id,
                CreateDoctorScheduleForm {
                    day_of_week: DayOfWeek::Monday,
                    start_time: "17:00".to_string(),
                    end_time: "09:00".to_string(),
                },
            )
            .await
            .unwrap_err();

        assert_eq!(
            error,
            DoctorError::InvalidInput("Start time must be before end time".to_string())
        );
    }
}
