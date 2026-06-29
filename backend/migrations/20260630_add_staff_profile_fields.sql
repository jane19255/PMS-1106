alter table public.staff
  add column if not exists dob date,
  add column if not exists gender text,
  add column if not exists nric text,
  add column if not exists address text,
  add column if not exists emergency_contact text;

alter table public.staff
  drop constraint if exists staff_dob_check,
  add constraint staff_dob_check check (dob is null or dob <= current_date),
  drop constraint if exists staff_gender_check,
  add constraint staff_gender_check check (gender is null or gender in ('Male', 'Female'));
