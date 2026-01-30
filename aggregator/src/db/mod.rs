//! Database module for PostgreSQL/TimescaleDB persistence

use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::types::Uuid;
use tracing::{info, debug};
use chrono::{DateTime, Utc};

use crate::config::DatabaseConfig;
use crate::state::{MacAddr, DeviceState, FlowState, FlowKey, ProtocolStats, VlanStats};

/// Database connection pool
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Connect to the database
    pub async fn connect(config: &DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await
            .with_context(|| format!("Failed to connect to database: {}", config.url))?;

        info!("Connected to database");
        Ok(Self { pool })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Upsert a device
    pub async fn upsert_device(&self, mac: &MacAddr, device: &DeviceState) -> Result<Uuid> {
        let mac_str = mac.to_string();
        let now = Utc::now();

        let row: (Uuid,) = sqlx::query_as(r#"
            INSERT INTO devices (mac_address, oui_prefix, first_seen, last_seen,
                                total_packets_sent, total_packets_received,
                                total_bytes_sent, total_bytes_received)
            VALUES ($1::macaddr, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (mac_address) DO UPDATE SET
                last_seen = EXCLUDED.last_seen,
                total_packets_sent = EXCLUDED.total_packets_sent,
                total_packets_received = EXCLUDED.total_packets_received,
                total_bytes_sent = EXCLUDED.total_bytes_sent,
                total_bytes_received = EXCLUDED.total_bytes_received,
                updated_at = NOW()
            RETURNING id
        "#)
            .bind(&mac_str)
            .bind(mac.oui_prefix())
            .bind(device.first_seen)
            .bind(now)
            .bind(device.packets_sent.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .bind(device.packets_received.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .bind(device.bytes_sent.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .bind(device.bytes_received.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .fetch_one(&self.pool)
            .await
            .with_context(|| format!("Failed to upsert device {}", mac_str))?;

        debug!("Upserted device {} with id {}", mac_str, row.0);
        Ok(row.0)
    }

    /// Upsert a device IP
    pub async fn upsert_device_ip(
        &self,
        device_id: Uuid,
        ip: std::net::Ipv4Addr,
        vlan_id: Option<u16>,
    ) -> Result<()> {
        let vlan = vlan_id.map(|v| v as i16);

        sqlx::query(r#"
            INSERT INTO device_ips (device_id, ip_address, vlan_id, first_seen, last_seen)
            VALUES ($1, $2::inet, $3, NOW(), NOW())
            ON CONFLICT ON CONSTRAINT uq_device_ip_vlan DO UPDATE SET
                last_seen = NOW()
        "#)
            .bind(device_id)
            .bind(ip.to_string())
            .bind(vlan)
            .execute(&self.pool)
            .await
            .with_context(|| format!("Failed to upsert device IP {}", ip))?;

        Ok(())
    }

    /// Upsert a flow
    pub async fn upsert_flow(
        &self,
        key: &FlowKey,
        flow: &FlowState,
        src_device_id: Option<Uuid>,
        dst_device_id: Option<Uuid>,
    ) -> Result<Uuid> {
        let src_mac = key.src_mac.to_string();
        let dst_mac = key.dst_mac.to_string();
        let src_ip = key.src_ip.map(|ip| ip.to_string());
        let dst_ip = key.dst_ip.map(|ip| ip.to_string());
        let now = Utc::now();

        let row: (Uuid,) = sqlx::query_as(r#"
            INSERT INTO traffic_flows (
                src_device_id, src_mac, src_ip, src_port,
                dst_device_id, dst_mac, dst_ip, dst_port,
                vlan_id, ip_protocol,
                first_seen, last_seen, packet_count, byte_count, tcp_flags_seen
            )
            VALUES ($1, $2::macaddr, $3::inet, $4, $5, $6::macaddr, $7::inet, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT ON CONSTRAINT traffic_flows_unique_tuple DO UPDATE SET
                last_seen = EXCLUDED.last_seen,
                packet_count = EXCLUDED.packet_count,
                byte_count = EXCLUDED.byte_count,
                tcp_flags_seen = traffic_flows.tcp_flags_seen | EXCLUDED.tcp_flags_seen
            RETURNING id
        "#)
            .bind(src_device_id)
            .bind(&src_mac)
            .bind(&src_ip)
            .bind(key.src_port.map(|p| p as i32))
            .bind(dst_device_id)
            .bind(&dst_mac)
            .bind(&dst_ip)
            .bind(key.dst_port.map(|p| p as i32))
            .bind(key.vlan_id.map(|v| v as i16))
            .bind(key.protocol.map(|p| p as i16))
            .bind(flow.first_seen)
            .bind(now)
            .bind(flow.packet_count.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .bind(flow.byte_count.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .bind(flow.tcp_flags_seen.load(std::sync::atomic::Ordering::Relaxed) as i16)
            .fetch_one(&self.pool)
            .await
            .with_context(|| format!("Failed to upsert flow {}->{}",src_mac, dst_mac))?;

        Ok(row.0)
    }

    /// Update protocol statistics
    pub async fn upsert_protocol(&self, ethertype: u16, ip_protocol: Option<u8>, stats: &ProtocolStats) -> Result<()> {
        let now = Utc::now();

        sqlx::query(r#"
            INSERT INTO protocol_stats (ethertype, ip_protocol, packet_count, byte_count, first_seen, last_seen)
            VALUES ($1, $2, $3, $4, $5, $5)
            ON CONFLICT ON CONSTRAINT uq_protocol DO UPDATE SET
                packet_count = EXCLUDED.packet_count,
                byte_count = EXCLUDED.byte_count,
                last_seen = EXCLUDED.last_seen
        "#)
            .bind(ethertype as i16)
            .bind(ip_protocol.map(|p| p as i16))
            .bind(stats.packet_count.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .bind(stats.byte_count.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update VLAN statistics
    pub async fn upsert_vlan(&self, vlan_id: u16, outer_vlan_id: Option<u16>, stats: &VlanStats) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO vlans (vlan_id, outer_vlan_id, first_seen, last_seen, total_packets, total_bytes)
            VALUES ($1, $2, $3, NOW(), $4, $5)
            ON CONFLICT ON CONSTRAINT uq_vlan_ids DO UPDATE SET
                last_seen = NOW(),
                total_packets = EXCLUDED.total_packets,
                total_bytes = EXCLUDED.total_bytes
        "#)
            .bind(vlan_id as i16)
            .bind(outer_vlan_id.map(|v| v as i16))
            .bind(stats.first_seen)
            .bind(stats.packet_count.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .bind(stats.byte_count.load(std::sync::atomic::Ordering::Relaxed) as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Insert traffic metrics for time-series data
    pub async fn insert_metrics(
        &self,
        device_id: Option<Uuid>,
        flow_id: Option<Uuid>,
        metric_type: &str,
        packet_count: u64,
        byte_count: u64,
    ) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO traffic_metrics (time, bucket_size, device_id, flow_id, metric_type, packet_count, byte_count)
            VALUES (NOW(), '1 minute', $1, $2, $3, $4, $5)
        "#)
            .bind(device_id)
            .bind(flow_id)
            .bind(metric_type)
            .bind(packet_count as i64)
            .bind(byte_count as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get device by MAC address
    pub async fn get_device_by_mac(&self, mac: &str) -> Result<Option<Uuid>> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM devices WHERE mac_address = $1::macaddr"
        )
            .bind(mac)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.0))
    }
}
