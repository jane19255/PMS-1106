create extension if not exists pgcrypto;

create table if not exists public.patients (
  id text primary key,
  first_name text not null check (length(trim(first_name)) > 0),
  last_name text not null check (length(trim(last_name)) > 0),
  dob date not null check (dob <= current_date),
  gender text not null check (gender in ('Male', 'Female')),
  nric text not null unique
    check (nric ~ '^[STFGM][0-9]{7}[A-Z]$'),
  nationality text not null check (length(trim(nationality)) > 0),
  phone text not null check (phone ~ '^[689][0-9]{7}$'),
  email text not null check (email ~* '^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$'),
  emergency_name text,
  emergency_phone text check (emergency_phone is null or emergency_phone ~ '^[689][0-9]{7}$'),
  address text,
  allergies text,
  medications text,
  conditions text,
  status text not null default 'Active' check (status in ('Active', 'Inactive')),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists patients_status_idx on public.patients(status);
create index if not exists patients_name_idx on public.patients(last_name, first_name);
create unique index if not exists patients_email_lower_uidx on public.patients(lower(email));

create table if not exists public.staff (
  id text primary key,
  firebase_uid text not null unique,
  full_name text not null check (length(trim(full_name)) > 0),
  email text not null unique check (email ~* '^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$'),
  phone text check (phone is null or phone ~ '^[689][0-9]{7}$'),
  role text not null check (role in ('admin', 'doctor', 'receptionist', 'pharmacist')),
  status text not null default 'active' check (status in ('active', 'inactive')),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists staff_role_idx on public.staff(role);
create index if not exists staff_status_idx on public.staff(status);

create table if not exists public.doctors (
  id text primary key,
  staff_id text not null unique references public.staff(id) on update cascade on delete restrict,
  license_number text not null unique,
  specialty text not null check (length(trim(specialty)) > 0),
  room text,
  availability_status text not null default 'Available'
    check (availability_status in ('Available', 'Unavailable', 'On Leave')),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists doctors_specialty_idx on public.doctors(specialty);

create table if not exists public.appointments (
  id text primary key,
  patient_id text not null references public.patients(id) on update cascade on delete restrict,
  doctor_id text not null references public.doctors(id) on update cascade on delete restrict,
  scheduled_at timestamptz not null,
  duration_minutes integer not null default 30 check (duration_minutes between 5 and 480),
  reason text not null check (length(trim(reason)) > 0),
  status text not null default 'Scheduled'
    check (status in ('Scheduled', 'Checked In', 'In Consultation', 'Completed', 'Cancelled', 'No Show')),
  notes text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists appointments_patient_id_idx on public.appointments(patient_id);
create index if not exists appointments_doctor_id_idx on public.appointments(doctor_id);
create index if not exists appointments_scheduled_at_idx on public.appointments(scheduled_at);
create index if not exists appointments_status_idx on public.appointments(status);

create table if not exists public.patient_queue (
  id text primary key,
  appointment_id text not null unique references public.appointments(id) on update cascade on delete cascade,
  patient_id text not null references public.patients(id) on update cascade on delete restrict,
  queue_date date not null default current_date,
  queue_number integer not null check (queue_number > 0),
  status text not null default 'Waiting'
    check (status in ('Waiting', 'Called', 'In Consultation', 'Completed', 'Skipped', 'Cancelled')),
  checked_in_at timestamptz not null default now(),
  called_at timestamptz,
  completed_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists patient_queue_daily_number_uidx
  on public.patient_queue(queue_date, queue_number);
create index if not exists patient_queue_status_idx on public.patient_queue(status);

create table if not exists public.medical_records (
  id text primary key,
  patient_id text not null references public.patients(id) on update cascade on delete cascade,
  appointment_id text references public.appointments(id) on update cascade on delete set null,
  doctor_id text references public.doctors(id) on update cascade on delete set null,
  diagnosis text,
  doctor_notes text,
  treatment_plan text,
  recorded_at timestamptz not null default now(),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

alter table public.medical_records
  add column if not exists doctor_id text references public.doctors(id) on update cascade on delete set null;

create index if not exists medical_records_patient_id_idx on public.medical_records(patient_id);
create index if not exists medical_records_appointment_id_idx on public.medical_records(appointment_id);
create index if not exists medical_records_doctor_id_idx on public.medical_records(doctor_id);
create index if not exists medical_records_recorded_at_idx on public.medical_records(recorded_at desc);

create table if not exists public.medicine_inventory (
  id text primary key,
  name text not null check (length(trim(name)) > 0),
  strength text,
  dosage_form text,
  stock_quantity integer not null default 0 check (stock_quantity >= 0),
  reorder_level integer not null default 0 check (reorder_level >= 0),
  unit_cost numeric(12, 2) not null default 0 check (unit_cost >= 0),
  status text not null default 'Active' check (status in ('Active', 'Inactive')),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists medicine_inventory_name_strength_uidx
  on public.medicine_inventory(lower(name), coalesce(lower(strength), ''));
create index if not exists medicine_inventory_status_idx on public.medicine_inventory(status);

create table if not exists public.prescriptions (
  id text primary key,
  patient_id text not null references public.patients(id) on update cascade on delete restrict,
  medical_record_id text references public.medical_records(id) on update cascade on delete set null,
  doctor_id text references public.doctors(id) on update cascade on delete set null,
  status text not null default 'Prescribed'
    check (status in ('Prescribed', 'Dispensed', 'Cancelled')),
  prescribed_at timestamptz not null default now(),
  dispensed_at timestamptz,
  notes text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists prescriptions_patient_id_idx on public.prescriptions(patient_id);
create index if not exists prescriptions_medical_record_id_idx on public.prescriptions(medical_record_id);
create index if not exists prescriptions_status_idx on public.prescriptions(status);

create table if not exists public.prescription_items (
  id uuid primary key default gen_random_uuid(),
  prescription_id text not null references public.prescriptions(id) on update cascade on delete cascade,
  medicine_id text not null references public.medicine_inventory(id) on update cascade on delete restrict,
  dosage text not null check (length(trim(dosage)) > 0),
  frequency text not null check (length(trim(frequency)) > 0),
  duration text not null check (length(trim(duration)) > 0),
  quantity integer not null check (quantity > 0),
  instructions text,
  created_at timestamptz not null default now()
);

create index if not exists prescription_items_prescription_id_idx on public.prescription_items(prescription_id);
create index if not exists prescription_items_medicine_id_idx on public.prescription_items(medicine_id);

create or replace function public.set_updated_at()
returns trigger
language plpgsql
as $$
begin
  new.updated_at = now();
  return new;
end;
$$;

drop trigger if exists set_patients_updated_at on public.patients;
create trigger set_patients_updated_at
before update on public.patients
for each row execute function public.set_updated_at();

drop trigger if exists set_staff_updated_at on public.staff;
create trigger set_staff_updated_at
before update on public.staff
for each row execute function public.set_updated_at();

drop trigger if exists set_doctors_updated_at on public.doctors;
create trigger set_doctors_updated_at
before update on public.doctors
for each row execute function public.set_updated_at();

drop trigger if exists set_appointments_updated_at on public.appointments;
create trigger set_appointments_updated_at
before update on public.appointments
for each row execute function public.set_updated_at();

drop trigger if exists set_patient_queue_updated_at on public.patient_queue;
create trigger set_patient_queue_updated_at
before update on public.patient_queue
for each row execute function public.set_updated_at();

drop trigger if exists set_medical_records_updated_at on public.medical_records;
create trigger set_medical_records_updated_at
before update on public.medical_records
for each row execute function public.set_updated_at();

drop trigger if exists set_medicine_inventory_updated_at on public.medicine_inventory;
create trigger set_medicine_inventory_updated_at
before update on public.medicine_inventory
for each row execute function public.set_updated_at();

drop trigger if exists set_prescriptions_updated_at on public.prescriptions;
create trigger set_prescriptions_updated_at
before update on public.prescriptions
for each row execute function public.set_updated_at();

alter table public.patients enable row level security;
alter table public.staff enable row level security;
alter table public.doctors enable row level security;
alter table public.appointments enable row level security;
alter table public.patient_queue enable row level security;
alter table public.medical_records enable row level security;
alter table public.medicine_inventory enable row level security;
alter table public.prescriptions enable row level security;
alter table public.prescription_items enable row level security;

create table if not exists public.doctor_schedules (
  id text primary key,
  doctor_id text not null references public.doctors(id) on delete cascade,
  day_of_week text not null
    check (day_of_week in ('Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday')),
  start_time time not null,
  end_time time not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  constraint doctor_schedule_time_range check (start_time < end_time)
);

create index if not exists doctor_schedules_doctor_id_idx on public.doctor_schedules(doctor_id);
create index if not exists doctor_schedules_day_idx on public.doctor_schedules(day_of_week);

drop trigger if exists set_doctor_schedules_updated_at on public.doctor_schedules;
create trigger set_doctor_schedules_updated_at
before update on public.doctor_schedules
for each row
execute function public.set_updated_at();

alter table public.doctor_schedules enable row level security;
