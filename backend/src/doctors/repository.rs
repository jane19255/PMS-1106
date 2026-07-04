//! Doctor repository implementations.
//!
//! Provides doctor and schedule persistence through a common interface for both tests
//! and the Supabase-backed application.
use super::models::{DayOfWeek, Doctor, DoctorSchedule, DoctorStatus};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

pub type RepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + Send + 'a>>;

#[derive(Debug)]
pub enum RepositoryError {
    DoctorHasAppointments,
    DuplicateDoctor,
    InvalidStaffId,
    NotFound,
    StorageUnavailable,
}

pub trait DoctorRepository: Send + Sync {
    fn create(&self, doctor: Doctor) -> RepositoryFuture<'_, Doctor>;
    fn find_by_id(&self, doctor_id: &str) -> RepositoryFuture<'_, Doctor>;
    fn list(&self) -> RepositoryFuture<'_, Vec<Doctor>>;
    fn update(&self, doctor: Doctor) -> RepositoryFuture<'_, Doctor>;
    fn delete(&self, doctor_id: &str) -> RepositoryFuture<'_, ()>;
    fn create_schedule(&self, schedule: DoctorSchedule) -> RepositoryFuture<'_, DoctorSchedule>;
    fn list_schedules(&self, doctor_id: &str) -> RepositoryFuture<'_, Vec<DoctorSchedule>>;
    fn delete_schedule(&self, schedule_id: &str) -> RepositoryFuture<'_, ()>;
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
    fn create(&self, doctor: Doctor) -> RepositoryFuture<'_, Doctor> {
        Box::pin(async move {
            let mut doctors = self
                .doctors
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            doctors.insert(doctor.id.clone(), doctor.clone());
            Ok(doctor)
        })
    }

    fn find_by_id(&self, doctor_id: &str) -> RepositoryFuture<'_, Doctor> {
        let doctor_id = doctor_id.to_string();
        Box::pin(async move {
            let doctors = self
                .doctors
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            doctors
                .get(&doctor_id)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<Doctor>> {
        Box::pin(async move {
            let doctors = self
                .doctors
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            let mut doctor_list: Vec<Doctor> = doctors.values().cloned().collect();
            doctor_list.sort_by(|left, right| left.name.cmp(&right.name));
            Ok(doctor_list)
        })
    }

    fn update(&self, doctor: Doctor) -> RepositoryFuture<'_, Doctor> {
        Box::pin(async move {
            let mut doctors = self
                .doctors
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !doctors.contains_key(&doctor.id) {
                return Err(RepositoryError::NotFound);
            }

            doctors.insert(doctor.id.clone(), doctor.clone());
            Ok(doctor)
        })
    }

    fn delete(&self, doctor_id: &str) -> RepositoryFuture<'_, ()> {
        let doctor_id = doctor_id.to_string();
        Box::pin(async move {
            let mut doctors = self
                .doctors
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            doctors
                .remove(&doctor_id)
                .map(|_| ())
                .ok_or(RepositoryError::NotFound)?;

            let mut schedules = self
                .schedules
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            schedules.retain(|_, schedule| schedule.doctor_id != doctor_id);
            Ok(())
        })
    }

    fn create_schedule(&self, schedule: DoctorSchedule) -> RepositoryFuture<'_, DoctorSchedule> {
        Box::pin(async move {
            self.find_by_id(&schedule.doctor_id).await?;

            let mut schedules = self
                .schedules
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            schedules.insert(schedule.id.clone(), schedule.clone());
            Ok(schedule)
        })
    }

    fn list_schedules(&self, doctor_id: &str) -> RepositoryFuture<'_, Vec<DoctorSchedule>> {
        let doctor_id = doctor_id.to_string();
        Box::pin(async move {
            self.find_by_id(&doctor_id).await?;

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
        })
    }

    fn delete_schedule(&self, schedule_id: &str) -> RepositoryFuture<'_, ()> {
        let schedule_id = schedule_id.to_string();
        Box::pin(async move {
            let mut schedules = self
                .schedules
                .lock()
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            schedules
                .remove(&schedule_id)
                .map(|_| ())
                .ok_or(RepositoryError::NotFound)
        })
    }
}

pub struct SupabaseDoctorRepository {
    url: String,
    key: String,
    client: Client,
}

impl SupabaseDoctorRepository {
    pub fn new(url: String, key: String) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            key,
            client: Client::new(),
        }
    }

    fn request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let builder = builder
            .header("apikey", &self.key)
            .header("Content-Type", "application/json");

        if self.key.starts_with("eyJ") {
            builder.header("Authorization", format!("Bearer {}", self.key))
        } else {
            builder
        }
    }

    fn doctors_url(&self, query: &str) -> String {
        format!("{}/rest/v1/doctors?{}", self.url, query)
    }

    fn schedules_url(&self, query: &str) -> String {
        format!("{}/rest/v1/doctor_schedules?{}", self.url, query)
    }

    fn room_status_url(&self, query: &str) -> String {
        format!("{}/rest/v1/room_status?{}", self.url, query)
    }

    async fn assign_room_to_doctor(&self, doctor_id: &str) -> Result<(), RepositoryError> {
        let response = self
            .request(self.client.get(self.room_status_url("select=room")))
            .send()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }

        let rows = response
            .json::<Vec<DatabaseRoomStatus>>()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        let mut used_numbers: Vec<i32> = rows
            .into_iter()
            .filter_map(|row| {
                row.room
                    .strip_prefix('R')
                    .and_then(|value| value.parse::<i32>().ok())
                    .map(|number| number - 100)
            })
            .collect();

        used_numbers.sort();

        let mut next_number = 1;
        while used_numbers.contains(&next_number) {
            next_number += 1;
        }

        let room = format!("R{}", 100 + next_number);

        let response = self
            .request(self.client.post(self.room_status_url("select=*")))
            .header("Prefer", "return=minimal")
            .json(&json!({
                "doctor_id": doctor_id,
                "room": room,
                "status": "Available",
                "current_appointment_id": null
            }))
            .send()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Self::response_error(response).await)
        }
    }

    async fn decode_doctor_rows(
        response: Response,
    ) -> Result<Vec<DatabaseDoctor>, RepositoryError> {
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }
        response
            .json::<Vec<DatabaseDoctor>>()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)
    }

    async fn decode_schedule_rows(
        response: Response,
    ) -> Result<Vec<DatabaseDoctorSchedule>, RepositoryError> {
        if !response.status().is_success() {
            return Err(Self::response_error(response).await);
        }
        response
            .json::<Vec<DatabaseDoctorSchedule>>()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)
    }

    async fn response_error(response: Response) -> RepositoryError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eprintln!("Supabase doctor error {status}: {body}");

        if status.as_u16() == 409 && body.contains("appointments_doctor_id_fkey") {
            RepositoryError::DoctorHasAppointments
        } else if status.as_u16() == 409 && body.contains("doctors_staff_id_fkey") {
            RepositoryError::InvalidStaffId
        } else if status.as_u16() == 409
            && (body.contains("doctors_staff_id_key") || body.contains("doctors_license_number_key"))
        {
            RepositoryError::DuplicateDoctor
        } else {
            RepositoryError::StorageUnavailable
        }
    }
}

impl DoctorRepository for SupabaseDoctorRepository {
    fn create(&self, doctor: Doctor) -> RepositoryFuture<'_, Doctor> {
        Box::pin(async move {
            let doctor_response = self
                .request(self.client.post(self.doctors_url("select=id")))
                .header("Prefer", "return=minimal")
                .json(&json!({
                    "id": doctor.id,
                    "staff_id": doctor.staff_id,
                    "license_number": doctor.license_number,
                    "specialty": doctor.specialization,
                    "availability_status": doctor.status,
                }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !doctor_response.status().is_success() {
                return Err(Self::response_error(doctor_response).await);
            }

            self.update_staff_profile(&doctor).await?;
            self.assign_room_to_doctor(&doctor.id).await?;
            self.find_by_id(&doctor.id).await
        })
    }

    fn find_by_id(&self, doctor_id: &str) -> RepositoryFuture<'_, Doctor> {
        let doctor_id = doctor_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&doctor_id);
            let query = format!(
                "select=*,staff:staff_id(id,full_name,email,phone,status)&id=eq.{}&limit=1",
                encoded_id
            );
            let response = self
                .request(self.client.get(self.doctors_url(&query)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut rows = Self::decode_doctor_rows(response).await?;
            rows.pop()
                .map(Doctor::from)
                .ok_or(RepositoryError::NotFound)
        })
    }

    fn list(&self) -> RepositoryFuture<'_, Vec<Doctor>> {
        Box::pin(async move {
            let response = self
                .request(self.client.get(self.doctors_url(
                    "select=*,staff:staff_id(id,full_name,email,phone,status)&order=id.asc",
                )))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut doctors: Vec<Doctor> = Self::decode_doctor_rows(response)
                .await?
                .into_iter()
                .map(Doctor::from)
                .collect();
            doctors.sort_by(|left, right| left.name.cmp(&right.name));
            Ok(doctors)
        })
    }

    fn update(&self, doctor: Doctor) -> RepositoryFuture<'_, Doctor> {
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&doctor.id);
            let response = self
                .request(
                    self.client
                        .patch(self.doctors_url(&format!("id=eq.{encoded_id}"))),
                )
                .header("Prefer", "return=minimal")
                .json(&json!({
                    "staff_id": doctor.staff_id,
                    "license_number": doctor.license_number,
                    "specialty": doctor.specialization,
                    "availability_status": doctor.status,
                }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if !response.status().is_success() {
                return Err(Self::response_error(response).await);
            }

            self.update_staff_profile(&doctor).await?;
            self.find_by_id(&doctor.id).await
        })
    }

    fn delete(&self, doctor_id: &str) -> RepositoryFuture<'_, ()> {
        let doctor_id = doctor_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&doctor_id);
            let response = self
                .request(
                    self.client
                        .delete(self.doctors_url(&format!("id=eq.{encoded_id}"))),
                )
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(Self::response_error(response).await)
            }
        })
    }

    fn create_schedule(&self, schedule: DoctorSchedule) -> RepositoryFuture<'_, DoctorSchedule> {
        Box::pin(async move {
            self.find_by_id(&schedule.doctor_id).await?;
            let response = self
                .request(self.client.post(self.schedules_url("select=*")))
                .header("Prefer", "return=representation")
                .json(&json!({
                    "id": schedule.id,
                    "doctor_id": schedule.doctor_id,
                    "day_of_week": schedule.day_of_week,
                    "start_time": schedule.start_time,
                    "end_time": schedule.end_time,
                }))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            let mut rows = Self::decode_schedule_rows(response).await?;
            rows.pop()
                .map(DoctorSchedule::from)
                .ok_or(RepositoryError::StorageUnavailable)
        })
    }

    fn list_schedules(&self, doctor_id: &str) -> RepositoryFuture<'_, Vec<DoctorSchedule>> {
        let doctor_id = doctor_id.to_string();
        Box::pin(async move {
            self.find_by_id(&doctor_id).await?;
            let encoded_id = urlencoding::encode(&doctor_id);
            let query = format!("select=*&doctor_id=eq.{encoded_id}&order=start_time.asc");
            let response = self
                .request(self.client.get(self.schedules_url(&query)))
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;
            Ok(Self::decode_schedule_rows(response)
                .await?
                .into_iter()
                .map(DoctorSchedule::from)
                .collect())
        })
    }

    fn delete_schedule(&self, schedule_id: &str) -> RepositoryFuture<'_, ()> {
        let schedule_id = schedule_id.to_string();
        Box::pin(async move {
            let encoded_id = urlencoding::encode(&schedule_id);
            let response = self
                .request(
                    self.client
                        .delete(self.schedules_url(&format!("id=eq.{encoded_id}"))),
                )
                .send()
                .await
                .map_err(|_| RepositoryError::StorageUnavailable)?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(Self::response_error(response).await)
            }
        })
    }
}

impl SupabaseDoctorRepository {
    async fn update_staff_profile(&self, doctor: &Doctor) -> Result<(), RepositoryError> {
        let encoded_id = urlencoding::encode(&doctor.staff_id);
        let url = format!("{}/rest/v1/staff?id=eq.{}", self.url, encoded_id);
        let response = self
            .request(self.client.patch(url))
            .header("Prefer", "return=minimal")
            .json(&json!({
                "full_name": doctor.name,
                "email": doctor.email,
                "phone": doctor.contact_number,
            }))
            .send()
            .await
            .map_err(|_| RepositoryError::StorageUnavailable)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Self::response_error(response).await)
        }
    }
}

#[derive(Deserialize)]
struct DatabaseDoctor {
    id: String,
    staff_id: String,
    license_number: String,
    specialty: String,
    availability_status: DoctorStatus,
    staff: Option<DatabaseStaff>,
}

#[derive(Deserialize)]
struct DatabaseStaff {
    full_name: String,
    email: String,
    phone: Option<String>,
}

impl From<DatabaseDoctor> for Doctor {
    fn from(row: DatabaseDoctor) -> Self {
        let staff = row.staff.unwrap_or_else(|| DatabaseStaff {
            full_name: row.id.clone(),
            email: String::new(),
            phone: None,
        });

        Self {
            id: row.id,
            staff_id: row.staff_id,
            license_number: row.license_number,
            name: staff.full_name,
            specialization: row.specialty,
            contact_number: staff.phone.unwrap_or_default(),
            email: staff.email,
            status: row.availability_status,
        }
    }
}

#[derive(Deserialize)]
struct DatabaseDoctorSchedule {
    id: String,
    doctor_id: String,
    day_of_week: DayOfWeek,
    start_time: String,
    end_time: String,
}

#[derive(Deserialize)]
struct DatabaseRoomStatus {
    room: String,
}

impl From<DatabaseDoctorSchedule> for DoctorSchedule {
    fn from(row: DatabaseDoctorSchedule) -> Self {
        Self {
            id: row.id,
            doctor_id: row.doctor_id,
            day_of_week: row.day_of_week,
            start_time: trim_seconds(row.start_time),
            end_time: trim_seconds(row.end_time),
        }
    }
}

fn trim_seconds(value: String) -> String {
    value.strip_suffix(":00").unwrap_or(&value).to_string()
}

