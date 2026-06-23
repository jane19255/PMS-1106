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

create table if not exists public.medical_records (
  id text primary key,
  patient_id text not null references public.patients(id) on update cascade on delete cascade,
  appointment_id text,
  doctor_name text,
  diagnosis text,
  doctor_notes text,
  treatment_plan text,
  prescribed_medicines text,
  recorded_at timestamptz not null default now(),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists medical_records_patient_id_idx on public.medical_records(patient_id);
create index if not exists medical_records_appointment_id_idx on public.medical_records(appointment_id);
create index if not exists medical_records_recorded_at_idx on public.medical_records(recorded_at desc);

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

drop trigger if exists set_medical_records_updated_at on public.medical_records;

create trigger set_medical_records_updated_at
before update on public.medical_records
for each row
execute function public.set_updated_at();

alter table public.patients enable row level security;
alter table public.medical_records enable row level security;
