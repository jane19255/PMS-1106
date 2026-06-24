-- PostgreSQL / Supabase billing schema.
-- Run after supabase_schema.sql so public.patients and public.appointments already exist.

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
