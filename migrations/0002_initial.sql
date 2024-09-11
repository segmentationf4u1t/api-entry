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

-- Create api_statistics table
CREATE TABLE api_statistics (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
  total_requests BIGINT NOT NULL,
  avg_response_time DOUBLE PRECISION NOT NULL,
  error_rate DOUBLE PRECISION NOT NULL,
  uptime DOUBLE PRECISION NOT NULL,
  register_requests BIGINT NOT NULL,
  register_success BIGINT NOT NULL,
  get_user_requests BIGINT NOT NULL,
  get_user_success BIGINT NOT NULL
);

-- Create api_traffic_distribution table
CREATE TABLE api_traffic_distribution (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
  route TEXT NOT NULL,
  count BIGINT NOT NULL
);

-- Create api_request_log table
CREATE TABLE api_request_log (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
  method TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  status SMALLINT NOT NULL
);

-- Create api_error_log table
CREATE TABLE api_error_log (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
  message TEXT NOT NULL
);

-- Create system_health table
CREATE TABLE system_health (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
  cpu_usage_1min DOUBLE PRECISION NOT NULL,
  cpu_usage_5min DOUBLE PRECISION NOT NULL,
  cpu_usage_15min DOUBLE PRECISION NOT NULL,
  memory_usage DOUBLE PRECISION NOT NULL,
  disk_usage DOUBLE PRECISION NOT NULL
);

-- Create indexes for better query performance
CREATE INDEX idx_api_statistics_timestamp ON api_statistics(timestamp);
CREATE INDEX idx_api_traffic_distribution_timestamp ON api_traffic_distribution(timestamp);
CREATE INDEX idx_api_request_log_timestamp ON api_request_log(timestamp);
CREATE INDEX idx_api_error_log_timestamp ON api_error_log(timestamp);
CREATE INDEX idx_system_health_timestamp ON system_health(timestamp);