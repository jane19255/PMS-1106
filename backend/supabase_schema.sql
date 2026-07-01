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
  dob date check (dob is null or dob <= current_date),
  gender text check (gender is null or gender in ('Male', 'Female')),
  nric text,
  email text not null unique check (email ~* '^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$'),
  phone text check (phone is null or phone ~ '^[689][0-9]{7}$'),
  role text not null check (role in ('admin', 'doctor', 'receptionist', 'pharmacist')),
  status text not null default 'active' check (status in ('active', 'inactive')),
  address text,
  emergency_contact text,
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

-- Keep the database statuses in sync with the reception workflow.
alter table public.appointments
  drop constraint if exists appointments_status_check;
alter table public.appointments
  add constraint appointments_status_check
  check (status in (
    'Scheduled',
    'Checked In',
    'Vitals Recorded',
    'In Consultation',
    'Completed',
    'Cancelled',
    'No Show'
  ));

alter table public.appointments
  add column if not exists priority text not null default 'Normal'
    check (priority in ('Emergency', 'Urgent', 'Normal', 'Follow-up')),
  add column if not exists room text,
  add column if not exists appointment_type text not null default 'Routine Checkup'
    check (appointment_type in ('Routine Checkup', 'Follow-up', 'New Consultation', 'Emergency')),
  add column if not exists referring_provider text,
  add column if not exists special_requirements text[] not null default '{}';

-- Store the end time so PostgreSQL can enforce non-overlapping appointments.
-- A trigger keeps it in sync whenever the start time or duration changes.
alter table public.appointments
  add column if not exists scheduled_end_at timestamptz;

create or replace function public.set_appointment_end_time()
returns trigger
language plpgsql
as $$
begin
  new.scheduled_end_at := new.scheduled_at
    + make_interval(mins => new.duration_minutes);
  return new;
end;
$$;

drop trigger if exists set_appointment_end_time on public.appointments;
create trigger set_appointment_end_time
before insert or update of scheduled_at, duration_minutes
on public.appointments
for each row execute function public.set_appointment_end_time();

update public.appointments
set scheduled_end_at = scheduled_at + make_interval(mins => duration_minutes)
where scheduled_end_at is null
   or scheduled_end_at <> scheduled_at + make_interval(mins => duration_minutes);

alter table public.appointments
  alter column scheduled_end_at set not null;

create extension if not exists btree_gist;

alter table public.appointments
  drop constraint if exists appointments_doctor_time_excl;
alter table public.appointments
  add constraint appointments_doctor_time_excl
  exclude using gist (
    doctor_id with =,
    tstzrange(scheduled_at, scheduled_end_at, '[)') with &&
  )
  where (status not in ('Cancelled', 'No Show'));

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

-- Priority is recorded when the receptionist saves the patient's vitals.
alter table public.patient_queue
  add column if not exists priority text not null default 'Normal',
  add column if not exists priority_reason text;

alter table public.patient_queue
  drop constraint if exists patient_queue_priority_check;
alter table public.patient_queue
  add constraint patient_queue_priority_check
  check (priority in ('Normal', 'Urgent', 'Emergency'));

-- Queue numbers must be assigned inside one database transaction. Calculating
-- max + 1 in Rust can give two patients the same number during simultaneous check-ins.
create or replace function public.enqueue_patient(
  p_appointment_id text,
  p_patient_id text,
  p_queue_date date,
  p_priority text,
  p_priority_reason text
)
returns setof public.patient_queue
language plpgsql
as $$
declare
  next_number integer;
  appointment_patient_id text;
  appointment_status text;
  action_time timestamptz := now();
begin
  select appointment.patient_id, appointment.status
    into appointment_patient_id, appointment_status
  from public.appointments as appointment
  where appointment.id = p_appointment_id
  for update;

  if not found then
    raise exception 'Appointment not found';
  end if;
  if appointment_patient_id <> p_patient_id then
    raise exception 'Appointment belongs to a different patient';
  end if;
  if p_priority not in ('Normal', 'Urgent', 'Emergency') then
    raise exception 'Invalid queue priority';
  end if;
  if p_priority in ('Urgent', 'Emergency')
     and nullif(trim(p_priority_reason), '') is null then
    raise exception 'Priority reason is required';
  end if;

  lock table public.patient_queue in share row exclusive mode;

  -- Repeated check-in requests return the existing row instead of duplicating it.
  return query
    select queue_row.*
    from public.patient_queue as queue_row
    where queue_row.appointment_id = p_appointment_id;
  if found then
    if appointment_status = 'Scheduled' then
      update public.appointments
      set status = 'Checked In', updated_at = action_time
      where id = p_appointment_id;
    end if;
    return;
  end if;

  if appointment_status <> 'Scheduled' then
    raise exception 'Only scheduled appointments can be checked in';
  end if;

  select coalesce(max(queue_row.queue_number), 0) + 1
    into next_number
  from public.patient_queue as queue_row
  where queue_row.queue_date = p_queue_date;

  return query
    insert into public.patient_queue (
      id,
      appointment_id,
      patient_id,
      queue_date,
      queue_number,
      status,
      priority,
      priority_reason,
      checked_in_at
    ) values (
      'Q-' || gen_random_uuid()::text,
      p_appointment_id,
      p_patient_id,
      p_queue_date,
      next_number,
      'Waiting',
      p_priority,
      p_priority_reason,
      action_time
    )
    returning *;

  update public.appointments
  set status = 'Checked In', updated_at = action_time
  where id = p_appointment_id;
end;
$$;

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

alter table public.medical_records
  add column if not exists reason_of_visit text,
  add column if not exists clinical_findings text,
  add column if not exists blood_pressure text,
  add column if not exists temperature text,
  add column if not exists pulse_rate text,
  add column if not exists height_cm text,
  add column if not exists weight_kg text;

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

create table if not exists public.room_status (
  doctor_id text primary key references public.doctors(id) on update cascade on delete cascade,
  room text not null unique,
  status text not null default 'Available'
    check (status in ('Available', 'Occupied', 'Unavailable')),
  current_appointment_id text references public.appointments(id) on update cascade on delete set null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index if not exists room_status_room_idx on public.room_status(room);
create index if not exists room_status_status_idx on public.room_status(status);

drop trigger if exists set_room_status_updated_at on public.room_status;
create trigger set_room_status_updated_at
before update on public.room_status
for each row execute function public.set_updated_at();

alter table public.room_status enable row level security;

-- Moving a patient into a room changes three related records. Keeping these
-- updates in one function prevents a partly updated consultation state.
create or replace function public.send_patient_to_room(
  p_appointment_id text,
  p_doctor_id text
)
returns jsonb
language plpgsql
as $$
declare
  appointment_doctor_id text;
  appointment_status text;
  room_appointment_id text;
  action_time timestamptz := now();
begin
  select appointment.doctor_id, appointment.status
    into appointment_doctor_id, appointment_status
  from public.appointments as appointment
  where appointment.id = p_appointment_id
  for update;

  if not found then
    raise exception 'Appointment not found';
  end if;
  if appointment_doctor_id <> p_doctor_id then
    raise exception 'Appointment is assigned to a different doctor';
  end if;
  if appointment_status not in ('Checked In', 'Vitals Recorded', 'In Consultation') then
    raise exception 'Patient must be checked in before entering a room';
  end if;

  select room.current_appointment_id
    into room_appointment_id
  from public.room_status as room
  where room.doctor_id = p_doctor_id
  for update;

  if not found then
    raise exception 'Doctor does not have an assigned room';
  end if;
  if room_appointment_id is not null and room_appointment_id <> p_appointment_id then
    raise exception 'Consultation room is already occupied';
  end if;

  perform 1
  from public.patient_queue as queue_row
  where queue_row.appointment_id = p_appointment_id
  for update;
  if not found then
    raise exception 'Patient is not in the queue';
  end if;

  update public.appointments
  set status = 'In Consultation', updated_at = action_time
  where id = p_appointment_id;

  update public.patient_queue
  set status = 'Called', called_at = action_time, updated_at = action_time
  where appointment_id = p_appointment_id;

  update public.room_status
  set status = 'Occupied',
      current_appointment_id = p_appointment_id,
      updated_at = action_time
  where doctor_id = p_doctor_id;

  return jsonb_build_object(
    'appointment_id', p_appointment_id,
    'doctor_id', p_doctor_id,
    'status', 'In Consultation'
  );
end;
$$;

-- The doctor starts only the consultation assigned to their occupied room.
create or replace function public.start_consultation(
  p_appointment_id text,
  p_doctor_id text
)
returns jsonb
language plpgsql
as $$
declare
  appointment_doctor_id text;
  appointment_status text;
  queue_status text;
  room_doctor_id text;
begin
  select appointment.doctor_id, appointment.status
    into appointment_doctor_id, appointment_status
  from public.appointments as appointment
  where appointment.id = p_appointment_id
  for update;

  if not found then
    raise exception 'Appointment not found';
  end if;
  if appointment_doctor_id <> p_doctor_id then
    raise exception 'Appointment belongs to a different doctor';
  end if;
  if appointment_status <> 'In Consultation' then
    raise exception 'Patient has not been sent to the consultation room';
  end if;

  select queue_row.status
    into queue_status
  from public.patient_queue as queue_row
  where queue_row.appointment_id = p_appointment_id
  for update;
  if not found then
    raise exception 'Patient queue entry not found';
  end if;

  select room.doctor_id
    into room_doctor_id
  from public.room_status as room
  where room.current_appointment_id = p_appointment_id
  for update;
  if not found then
    raise exception 'Consultation room is not assigned to this appointment';
  end if;
  if room_doctor_id <> p_doctor_id then
    raise exception 'Consultation room belongs to a different doctor';
  end if;

  if queue_status = 'In Consultation' then
    return jsonb_build_object(
      'appointment_id', p_appointment_id,
      'doctor_id', p_doctor_id,
      'status', 'In Consultation'
    );
  end if;
  if queue_status <> 'Called' then
    raise exception 'Patient must be called before consultation starts';
  end if;

  update public.patient_queue
  set status = 'In Consultation', updated_at = now()
  where appointment_id = p_appointment_id;

  return jsonb_build_object(
    'appointment_id', p_appointment_id,
    'doctor_id', p_doctor_id,
    'status', 'In Consultation'
  );
end;
$$;

-- Completing a consultation releases the room and closes the appointment and
-- queue entry together. Any validation error rolls back the whole operation.
create or replace function public.complete_consultation(
  p_appointment_id text,
  p_doctor_id text
)
returns jsonb
language plpgsql
as $$
declare
  appointment_doctor_id text;
  appointment_status text;
  room_doctor_id text;
  action_time timestamptz := now();
begin
  select appointment.doctor_id, appointment.status
    into appointment_doctor_id, appointment_status
  from public.appointments as appointment
  where appointment.id = p_appointment_id
  for update;

  if not found then
    raise exception 'Appointment not found';
  end if;
  if appointment_doctor_id <> p_doctor_id then
    raise exception 'Appointment belongs to a different doctor';
  end if;

  -- A repeated request after a successful completion does not change data.
  if appointment_status = 'Completed' then
    return jsonb_build_object(
      'appointment_id', p_appointment_id,
      'doctor_id', p_doctor_id,
      'status', 'Completed'
    );
  end if;
  if appointment_status <> 'In Consultation' then
    raise exception 'Appointment is not in consultation';
  end if;

  perform 1
  from public.patient_queue as queue_row
  where queue_row.appointment_id = p_appointment_id
  for update;
  if not found then
    raise exception 'Patient queue entry not found';
  end if;

  select room.doctor_id
    into room_doctor_id
  from public.room_status as room
  where room.current_appointment_id = p_appointment_id
  for update;
  if not found then
    raise exception 'Consultation room is not assigned to this appointment';
  end if;
  if room_doctor_id <> p_doctor_id then
    raise exception 'Consultation room belongs to a different doctor';
  end if;

  update public.appointments
  set status = 'Completed', updated_at = action_time
  where id = p_appointment_id;

  update public.patient_queue
  set status = 'Completed', completed_at = action_time, updated_at = action_time
  where appointment_id = p_appointment_id;

  update public.room_status
  set status = 'Available', current_appointment_id = null, updated_at = action_time
  where doctor_id = p_doctor_id;

  return jsonb_build_object(
    'appointment_id', p_appointment_id,
    'doctor_id', p_doctor_id,
    'status', 'Completed'
  );
end;
$$;

create table if not exists public.patient_vitals (
  id text primary key,
  appointment_id text not null references public.appointments(id) on update cascade on delete cascade,
  bp text not null,
  temp numeric not null,
  pulse integer not null,
  height numeric not null,
  weight numeric not null,
  recorded_at timestamptz not null default now(),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists patient_vitals_appointment_id_uidx
  on public.patient_vitals(appointment_id);
create index if not exists patient_vitals_recorded_at_idx on public.patient_vitals(recorded_at);

-- Vitals, triage priority, and appointment status form one reception action.
create or replace function public.save_patient_vitals(
  p_appointment_id text,
  p_bp text,
  p_temp numeric,
  p_pulse integer,
  p_height numeric,
  p_weight numeric,
  p_priority text,
  p_priority_reason text
)
returns jsonb
language plpgsql
as $$
declare
  appointment_status text;
  action_time timestamptz := now();
begin
  if p_temp < 35 or p_temp > 42 then
    raise exception 'Temperature out of valid range (35-42 C)';
  end if;
  if p_pulse < 40 or p_pulse > 180 then
    raise exception 'Pulse out of valid range (40-180 bpm)';
  end if;
  if p_height < 50 or p_height > 250 then
    raise exception 'Height value invalid';
  end if;
  if p_weight < 1 or p_weight > 300 then
    raise exception 'Weight value invalid';
  end if;
  if p_priority not in ('Normal', 'Urgent', 'Emergency') then
    raise exception 'Invalid queue priority';
  end if;
  if p_priority in ('Urgent', 'Emergency')
     and nullif(trim(p_priority_reason), '') is null then
    raise exception 'Priority reason is required';
  end if;

  select appointment.status
    into appointment_status
  from public.appointments as appointment
  where appointment.id = p_appointment_id
  for update;
  if not found then
    raise exception 'Appointment not found';
  end if;
  if appointment_status not in ('Checked In', 'Vitals Recorded') then
    raise exception 'Patient must be checked in before recording vitals';
  end if;

  perform 1
  from public.patient_queue as queue_row
  where queue_row.appointment_id = p_appointment_id
  for update;
  if not found then
    raise exception 'Patient queue entry not found';
  end if;

  insert into public.patient_vitals (
    id,
    appointment_id,
    bp,
    temp,
    pulse,
    height,
    weight,
    recorded_at
  ) values (
    'V-' || gen_random_uuid()::text,
    p_appointment_id,
    trim(p_bp),
    p_temp,
    p_pulse,
    p_height,
    p_weight,
    action_time
  )
  on conflict (appointment_id) do update set
    bp = excluded.bp,
    temp = excluded.temp,
    pulse = excluded.pulse,
    height = excluded.height,
    weight = excluded.weight,
    recorded_at = excluded.recorded_at,
    updated_at = action_time;

  update public.patient_queue
  set priority = p_priority,
      priority_reason = nullif(trim(p_priority_reason), ''),
      updated_at = action_time
  where appointment_id = p_appointment_id;

  update public.appointments
  set status = 'Vitals Recorded', updated_at = action_time
  where id = p_appointment_id;

  return jsonb_build_object(
    'appointment_id', p_appointment_id,
    'status', 'Vitals Recorded',
    'priority', p_priority
  );
end;
$$;

drop trigger if exists set_patient_vitals_updated_at on public.patient_vitals;
create trigger set_patient_vitals_updated_at
before update on public.patient_vitals
for each row execute function public.set_updated_at();

alter table public.patient_vitals enable row level security;

-- ============================================================================
-- Billing
-- ============================================================================

create table if not exists public.invoices (
  id text primary key,
  patient_id text not null references public.patients(id) on update cascade,
  appointment_id text references public.appointments(id) on update cascade on delete set null,
  subtotal numeric(12, 2) not null check (subtotal >= 0),
  total numeric(12, 2) not null check (total >= 0),
  status text not null default 'Pending'
    check (status in ('Pending', 'Paid', 'Cancelled')),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  paid_at timestamptz,
  cancelled_at timestamptz,
  constraint paid_invoice_has_paid_at check (status <> 'Paid' or paid_at is not null),
  constraint cancelled_invoice_has_cancelled_at
    check (status <> 'Cancelled' or cancelled_at is not null)
);

create unique index if not exists invoices_active_appointment_uidx
  on public.invoices(appointment_id)
  where appointment_id is not null and status <> 'Cancelled';

create table if not exists public.invoice_items (
  id uuid primary key default gen_random_uuid(),
  invoice_id text not null references public.invoices(id) on delete cascade,
  item_type text not null check (item_type in ('Treatment', 'Prescription')),
  name text not null check (length(trim(name)) > 0),
  description text,
  cost numeric(12, 2) not null check (cost >= 0),
  created_at timestamptz not null default now()
);

create table if not exists public.payments (
  id text primary key,
  invoice_id text not null references public.invoices(id) on delete restrict,
  amount numeric(12, 2) not null check (amount > 0),
  payment_method text not null check (length(trim(payment_method)) > 0),
  transaction_reference text,
  paid_at timestamptz not null default now()
);

create index if not exists invoices_patient_id_idx on public.invoices(patient_id);
create index if not exists invoices_appointment_id_idx on public.invoices(appointment_id);
create index if not exists invoices_status_idx on public.invoices(status);
create index if not exists invoice_items_invoice_id_idx on public.invoice_items(invoice_id);
create index if not exists payments_invoice_id_idx on public.payments(invoice_id);

create or replace function public.set_billing_updated_at()
returns trigger
language plpgsql
as $$
begin
  new.updated_at = now();
  return new;
end;
$$;

drop trigger if exists set_invoices_updated_at on public.invoices;
create trigger set_invoices_updated_at
before update on public.invoices
for each row execute function public.set_billing_updated_at();

alter table public.invoices enable row level security;
alter table public.invoice_items enable row level security;
alter table public.payments enable row level security;

-- The backend uses the Supabase secret key and bypasses RLS. Add explicit
-- authenticated-user policies before exposing these tables directly to clients.
create or replace function public.billing_create_invoice(p_invoice jsonb)
returns void
language plpgsql
security definer
set search_path = public
as $$
declare
  item jsonb;
begin
  insert into public.invoices (
    id, patient_id, appointment_id, subtotal, total, status, created_at
  ) values (
    p_invoice->>'id',
    p_invoice->>'patient_id',
    nullif(trim(p_invoice->>'appointment_id'), ''),
    (p_invoice->>'subtotal')::numeric,
    (p_invoice->>'total')::numeric,
    'Pending',
    (p_invoice->>'created_at')::timestamptz
  );

  for item in select value from jsonb_array_elements(p_invoice->'items')
  loop
    insert into public.invoice_items (invoice_id, item_type, name, description, cost)
    values (
      p_invoice->>'id',
      item->>'item_type',
      item->>'name',
      nullif(trim(item->>'description'), ''),
      (item->>'cost')::numeric
    );
  end loop;
exception
  when unique_violation then
    raise exception 'active invoice already exists for appointment' using errcode = '23505';
end;
$$;

create or replace function public.billing_update_invoice(p_invoice jsonb)
returns void
language plpgsql
security definer
set search_path = public
as $$
declare
  current_invoice public.invoices%rowtype;
  payment jsonb;
  paid_total numeric(12, 2);
  requested_status text := p_invoice->>'status';
begin
  select * into current_invoice
  from public.invoices
  where id = p_invoice->>'id'
  for update;

  if not found then
    raise exception 'invoice not found' using errcode = 'P0002';
  end if;

  for payment in select value from jsonb_array_elements(p_invoice->'payments')
  loop
    insert into public.payments (
      id, invoice_id, amount, payment_method, transaction_reference, paid_at
    ) values (
      payment->>'id',
      current_invoice.id,
      (payment->>'amount')::numeric,
      payment->>'payment_method',
      nullif(trim(payment->>'transaction_reference'), ''),
      (payment->>'paid_at')::timestamptz
    ) on conflict (id) do nothing;
  end loop;

  select coalesce(sum(amount), 0) into paid_total
  from public.payments
  where invoice_id = current_invoice.id;

  if paid_total > current_invoice.total then
    raise exception 'payment exceeds outstanding balance' using errcode = '23514';
  end if;

  if requested_status = 'Cancelled' then
    if paid_total > 0 or current_invoice.status = 'Paid' then
      raise exception 'invoice with payments cannot be cancelled' using errcode = '23514';
    end if;
    update public.invoices
    set status = 'Cancelled', cancelled_at = now(), paid_at = null
    where id = current_invoice.id;
  elsif paid_total = current_invoice.total then
    update public.invoices
    set status = 'Paid', paid_at = now(), cancelled_at = null
    where id = current_invoice.id;
  else
    update public.invoices
    set status = 'Pending', paid_at = null, cancelled_at = null
    where id = current_invoice.id;
  end if;
end;
$$;

revoke all on function public.billing_create_invoice(jsonb) from public, anon, authenticated;
revoke all on function public.billing_update_invoice(jsonb) from public, anon, authenticated;
grant execute on function public.billing_create_invoice(jsonb) to service_role;
grant execute on function public.billing_update_invoice(jsonb) to service_role;

