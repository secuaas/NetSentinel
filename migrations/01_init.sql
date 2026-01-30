-- NetSentinel - Initial Schema Migration
-- Version: 001
-- Description: Creates base tables for CMDB, Metrics, and Configuration

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- ============================================================================
-- CMDB Schema (Inventory)
-- ============================================================================

-- Devices table - Core CMDB entity
CREATE TABLE devices (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mac_address             MACADDR NOT NULL UNIQUE,
    oui_vendor              VARCHAR(128),
    oui_prefix              CHAR(8),
    device_type             VARCHAR(50) DEFAULT 'unknown',
    device_name             VARCHAR(255),
    device_notes            TEXT,
    first_seen              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_packets_sent      BIGINT DEFAULT 0,
    total_packets_received  BIGINT DEFAULT 0,
    total_bytes_sent        BIGINT DEFAULT 0,
    total_bytes_received    BIGINT DEFAULT 0,
    is_gateway              BOOLEAN DEFAULT FALSE,
    is_active               BOOLEAN DEFAULT TRUE,
    is_flagged              BOOLEAN DEFAULT FALSE,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Device IP addresses - Many IPs per device
CREATE TABLE device_ips (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id        UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    ip_address       INET NOT NULL,
    ip_version       SMALLINT NOT NULL DEFAULT 4,
    vlan_id          SMALLINT,
    subnet_mask      INET,
    first_seen       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    packets_sent     BIGINT DEFAULT 0,
    packets_received BIGINT DEFAULT 0,
    bytes_sent       BIGINT DEFAULT 0,
    bytes_received   BIGINT DEFAULT 0,
    CONSTRAINT uq_device_ip_vlan UNIQUE (device_id, ip_address, COALESCE(vlan_id, -1))
);

-- VLANs table
CREATE TABLE vlans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vlan_id         SMALLINT NOT NULL,
    outer_vlan_id   SMALLINT,  -- QinQ support
    vlan_name       VARCHAR(64),
    vlan_description TEXT,
    device_count    INTEGER DEFAULT 0,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_packets   BIGINT DEFAULT 0,
    total_bytes     BIGINT DEFAULT 0,
    CONSTRAINT uq_vlan_ids UNIQUE (vlan_id, COALESCE(outer_vlan_id, -1))
);

-- ============================================================================
-- Metrics Schema (TimescaleDB Time-series)
-- ============================================================================

-- Traffic flows - Unique communication pairs
CREATE TABLE traffic_flows (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    src_device_id   UUID REFERENCES devices(id) ON DELETE SET NULL,
    src_mac         MACADDR NOT NULL,
    src_ip          INET,
    src_port        INTEGER,
    dst_device_id   UUID REFERENCES devices(id) ON DELETE SET NULL,
    dst_mac         MACADDR NOT NULL,
    dst_ip          INET,
    dst_port        INTEGER,
    vlan_id         SMALLINT,
    outer_vlan_id   SMALLINT,
    ethertype       SMALLINT,
    ip_protocol     SMALLINT,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    packet_count    BIGINT NOT NULL DEFAULT 0,
    byte_count      BIGINT NOT NULL DEFAULT 0,
    tcp_flags_seen  SMALLINT DEFAULT 0
);

-- Unique index for flow deduplication
CREATE UNIQUE INDEX idx_flows_unique_tuple ON traffic_flows (
    src_mac,
    COALESCE(src_ip, '0.0.0.0'::inet),
    COALESCE(src_port, 0),
    dst_mac,
    COALESCE(dst_ip, '0.0.0.0'::inet),
    COALESCE(dst_port, 0),
    COALESCE(vlan_id, 0),
    COALESCE(ip_protocol, 0)
);

-- Traffic metrics - Time-series hypertable
CREATE TABLE traffic_metrics (
    time            TIMESTAMPTZ NOT NULL,
    bucket_size     INTERVAL NOT NULL DEFAULT '1 minute',
    flow_id         UUID REFERENCES traffic_flows(id) ON DELETE CASCADE,
    device_id       UUID REFERENCES devices(id) ON DELETE CASCADE,
    metric_type     VARCHAR(20) NOT NULL,  -- 'device_in', 'device_out', 'flow'
    packet_count    BIGINT DEFAULT 0,
    byte_count      BIGINT DEFAULT 0,
    avg_packet_size REAL,
    max_packet_size INTEGER,
    min_packet_size INTEGER,
    tcp_syn_count   INTEGER DEFAULT 0,
    tcp_rst_count   INTEGER DEFAULT 0,
    tcp_fin_count   INTEGER DEFAULT 0
);

-- Convert to hypertable
SELECT create_hypertable('traffic_metrics', 'time', chunk_time_interval => INTERVAL '1 day');

-- Add retention policy (30 days)
SELECT add_retention_policy('traffic_metrics', INTERVAL '30 days');

-- Protocol statistics
CREATE TABLE protocol_stats (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ethertype       SMALLINT NOT NULL,
    ip_protocol     SMALLINT,
    protocol_name   VARCHAR(64),
    packet_count    BIGINT DEFAULT 0,
    byte_count      BIGINT DEFAULT 0,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_protocol UNIQUE (ethertype, COALESCE(ip_protocol, -1))
);

-- ============================================================================
-- Configuration Schema
-- ============================================================================

-- Configuration settings
CREATE TABLE config_settings (
    key          VARCHAR(128) PRIMARY KEY,
    value        JSONB NOT NULL,
    value_type   VARCHAR(20) NOT NULL,  -- string, integer, boolean, json
    category     VARCHAR(64) NOT NULL,
    description  TEXT,
    is_sensitive BOOLEAN DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Users table
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      VARCHAR(64) NOT NULL UNIQUE,
    email         VARCHAR(255),
    password_hash VARCHAR(255) NOT NULL,  -- bcrypt
    role          VARCHAR(20) NOT NULL DEFAULT 'viewer',  -- admin, operator, viewer
    is_active     BOOLEAN DEFAULT TRUE,
    last_login    TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Audit log
CREATE TABLE audit_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    action      VARCHAR(64) NOT NULL,
    entity_type VARCHAR(64),
    entity_id   UUID,
    details     JSONB,
    ip_address  INET,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- Indexes
-- ============================================================================

-- Devices
CREATE INDEX idx_devices_mac ON devices(mac_address);
CREATE INDEX idx_devices_type ON devices(device_type);
CREATE INDEX idx_devices_last_seen ON devices(last_seen DESC);
CREATE INDEX idx_devices_active ON devices(is_active) WHERE is_active = TRUE;

-- Device IPs
CREATE INDEX idx_device_ips_device ON device_ips(device_id);
CREATE INDEX idx_device_ips_ip ON device_ips(ip_address);
CREATE INDEX idx_device_ips_vlan ON device_ips(vlan_id);

-- VLANs
CREATE INDEX idx_vlans_id ON vlans(vlan_id);

-- Traffic flows
CREATE INDEX idx_flows_src_device ON traffic_flows(src_device_id);
CREATE INDEX idx_flows_dst_device ON traffic_flows(dst_device_id);
CREATE INDEX idx_flows_src_mac ON traffic_flows(src_mac);
CREATE INDEX idx_flows_dst_mac ON traffic_flows(dst_mac);
CREATE INDEX idx_flows_last_seen ON traffic_flows(last_seen DESC);
CREATE INDEX idx_flows_vlan ON traffic_flows(vlan_id);

-- Traffic metrics (TimescaleDB will auto-create time indexes)
CREATE INDEX idx_metrics_device ON traffic_metrics(device_id, time DESC);
CREATE INDEX idx_metrics_flow ON traffic_metrics(flow_id, time DESC);

-- Audit log
CREATE INDEX idx_audit_user ON audit_log(user_id);
CREATE INDEX idx_audit_created ON audit_log(created_at DESC);

-- ============================================================================
-- Triggers
-- ============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_devices_updated_at
    BEFORE UPDATE ON devices
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_config_updated_at
    BEFORE UPDATE ON config_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Initial Data
-- ============================================================================

-- Default admin user (password: changeme)
-- Generated with: python -c "import bcrypt; print(bcrypt.hashpw(b'changeme', bcrypt.gensalt()).decode())"
INSERT INTO users (username, password_hash, role) VALUES
    ('admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.mHFHmYCg1HO.3q', 'admin');

-- Default configuration
INSERT INTO config_settings (key, value, value_type, category, description) VALUES
    ('capture.mode', '"mirror"', 'string', 'capture', 'Capture mode: mirror or bypass'),
    ('capture.interfaces', '["eth0"]', 'json', 'capture', 'Network interfaces to monitor'),
    ('retention.days', '30', 'integer', 'storage', 'Data retention period in days'),
    ('export.enabled', 'false', 'boolean', 'export', 'Enable cloud export'),
    ('export.schedule', '"0 2 * * *"', 'string', 'export', 'Export schedule (cron format)'),
    ('ui.theme', '"dark"', 'string', 'ui', 'UI theme: dark or light'),
    ('ui.refresh_interval', '5', 'integer', 'ui', 'Dashboard refresh interval in seconds');

-- Common protocol names
INSERT INTO protocol_stats (ethertype, ip_protocol, protocol_name) VALUES
    (2048, NULL, 'IPv4'),
    (2054, NULL, 'ARP'),
    (34525, NULL, 'IPv6'),
    (2048, 1, 'ICMP'),
    (2048, 6, 'TCP'),
    (2048, 17, 'UDP'),
    (2048, 47, 'GRE'),
    (2048, 50, 'ESP'),
    (2048, 89, 'OSPF');
