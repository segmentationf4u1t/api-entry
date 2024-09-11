-- Drop existing tables
DROP TABLE IF EXISTS api_statistics;
DROP TABLE IF EXISTS api_traffic_distribution;
DROP TABLE IF EXISTS api_request_log;
DROP TABLE IF EXISTS api_error_log;
DROP TABLE IF EXISTS system_health;
DROP TABLE IF EXISTS users;

-- Recreate tables
CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  username TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  avatar TEXT,
  tokens JSONB,
  status TEXT DEFAULT 'active',
  permissions JSONB,
  last_login TIMESTAMP WITH TIME ZONE
);

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

CREATE TABLE api_traffic_distribution (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
  route TEXT NOT NULL,
  count BIGINT NOT NULL
);

CREATE TABLE api_request_log (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
  method TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  status SMALLINT NOT NULL
);

CREATE TABLE api_error_log (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
  message TEXT NOT NULL
);

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