create table if not exists public.patients (
  id text primary key,
  first_name text not null,
  last_name text not null,
  dob date not null,
  gender text not null,
  nric text not null unique,
  nationality text not null,
  phone text not null,
  email text not null,
  emergency_name text,
  emergency_phone text,
  address text,
  allergies text,
  medications text,
  conditions text,
  status text not null default 'Active',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists patients_status_idx on public.patients(status);
create index if not exists patients_name_idx on public.patients(last_name, first_name);

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
for each row
execute function public.set_updated_at();

alter table public.patients enable row level security;

=========================================================
Appointments table - Stores appointment details and links to patients and doctors
=========================================================

create table if not exists public.appointments (
  id uuid primary key default gen_random_uuid(),

  patient_id text not null,
  doctor_id text not null,

  appointment_datetime timestamptz not null,
  duration_minutes integer not null default 30,

  status text not null default 'Scheduled',
  priority integer not null default 3, -- 1=Emergency, 2=Urgent, 3=Normal, 4=Follow-up

  queue_number integer,

  medical_record_id text,

  notes text,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists appointments_patient_idx
on public.appointments(patient_id);

create index if not exists appointments_doctor_idx
on public.appointments(doctor_id);

create index if not exists appointments_datetime_idx
on public.appointments(appointment_datetime);

create index if not exists appointments_doctor_datetime_idx
on public.appointments(doctor_id, appointment_datetime);

drop trigger if exists set_appointments_updated_at
on public.appointments;

create trigger set_appointments_updated_at
before update on public.appointments
for each row
execute function public.set_updated_at();

alter table public.appointments enable row level security;

=========================================================
Appointment Queue table - Tracks live queue state and patient priority
=========================================================

create table if not exists public.appointment_queue (
  id uuid primary key default gen_random_uuid(),

  appointment_id uuid not null references public.appointments(id) on delete cascade,
  patient_id text not null,
  doctor_id text not null,

  priority integer not null default 3, -- 1=Emergency, 2=Urgent, 3=Normal, 4=Follow-up
  status text not null default 'Waiting', -- Waiting, InProgress, Completed, Cancelled, Skipped
  queue_position integer,

  checked_in_at timestamptz,
  called_at timestamptz,
  completed_at timestamptz,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists appointment_queue_doctor_idx
on public.appointment_queue(doctor_id);

create index if not exists appointment_queue_status_idx
on public.appointment_queue(status);

create index if not exists appointment_queue_priority_idx
on public.appointment_queue(priority);

drop trigger if exists set_appointment_queue_updated_at
on public.appointment_queue;

create trigger set_appointment_queue_updated_at
before update on public.appointment_queue
for each row
execute function public.set_updated_at();

alter table public.appointment_queue enable row level security;

=========================================================
Doctor Availability table - Stores doctor working hours and availability windows
=========================================================

create table if not exists public.doctor_availability (
  id uuid primary key default gen_random_uuid(),

  doctor_id text not null,
  available_from timestamptz not null,
  available_to timestamptz not null,
  is_available boolean not null default true,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),

  check (available_to > available_from)
);

create index if not exists doctor_availability_doctor_idx
on public.doctor_availability(doctor_id);

create index if not exists doctor_availability_range_idx
on public.doctor_availability(doctor_id, available_from, available_to);

drop trigger if exists set_doctor_availability_updated_at
on public.doctor_availability;

create trigger set_doctor_availability_updated_at
before update on public.doctor_availability
for each row
execute function public.set_updated_at();

alter table public.doctor_availability enable row level security;