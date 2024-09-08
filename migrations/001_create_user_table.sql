-- Migrations will appear here as you chat with AI

create table users (
  id bigint primary key generated always as identity,
  email text not null unique,
  username text not null unique,
  password text not null,
  created_at timestamp with time zone default now(),
  avatar text,
  tokens jsonb
);

alter table users
add column status text default 'active',
add column permissions jsonb,
add column last_login timestamp with time zone;