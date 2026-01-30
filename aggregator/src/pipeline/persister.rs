//! Periodic persistence of aggregated state to PostgreSQL

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::AggregationConfig;
use crate::db::Database;
use crate::state::{AggregatorState, MacAddr};

/// Persists aggregated state to the database periodically
pub struct Persister {
    config: AggregationConfig,
    state: Arc<AggregatorState>,
    db: Arc<Database>,
    device_ids: HashMap<MacAddr, Uuid>,
}

impl Persister {
    /// Create a new persister
    pub fn new(
        config: AggregationConfig,
        state: Arc<AggregatorState>,
        db: Arc<Database>,
    ) -> Self {
        Self {
            config,
            state,
            db,
            device_ids: HashMap::new(),
        }
    }

    /// Run the persistence loop
    pub async fn run(mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let interval = tokio::time::Duration::from_secs(self.config.persist_interval_secs);

        info!(
            "Starting persister with interval of {} seconds",
            self.config.persist_interval_secs
        );

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Persister received shutdown signal");
                    // Final persistence before shutdown
                    if let Err(e) = self.persist_all().await {
                        error!("Error in final persistence: {}", e);
                    }
                    break;
                }
                _ = tokio::time::sleep(interval) => {
                    if let Err(e) = self.persist_all().await {
                        error!("Error persisting state: {}", e);
                    }
                }
            }
        }

        info!("Persister stopped");
        Ok(())
    }

    /// Persist all state to the database
    async fn persist_all(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        // Persist devices
        let device_count = self.persist_devices().await?;

        // Persist flows
        let flow_count = self.persist_flows().await?;

        // Persist protocols
        let protocol_count = self.persist_protocols().await?;

        // Persist VLANs
        let vlan_count = self.persist_vlans().await?;

        let elapsed = start.elapsed();
        info!(
            "Persisted {} devices, {} flows, {} protocols, {} vlans in {:?}",
            device_count, flow_count, protocol_count, vlan_count, elapsed
        );

        Ok(())
    }

    /// Persist all devices
    async fn persist_devices(&mut self) -> Result<usize> {
        let mut count = 0;

        // Iterate over all devices in state
        for entry in self.state.devices.iter() {
            let mac = entry.key().clone();
            let device = entry.value();

            match self.db.upsert_device(&mac, device).await {
                Ok(device_id) => {
                    // Cache the device ID for flow persistence
                    self.device_ids.insert(mac.clone(), device_id);

                    // Persist associated IPs
                    for ip_entry in device.ips.iter() {
                        let ip = *ip_entry.key();
                        let ip_state = ip_entry.value();
                        let vlan_id = ip_state.vlan_id;

                        if let Err(e) = self.db.upsert_device_ip(device_id, ip, vlan_id).await {
                            warn!("Failed to persist device IP {}: {}", ip, e);
                        }
                    }

                    count += 1;
                }
                Err(e) => {
                    warn!("Failed to persist device {}: {}", mac.to_string(), e);
                }
            }
        }

        Ok(count)
    }

    /// Persist all flows
    async fn persist_flows(&self) -> Result<usize> {
        let mut count = 0;

        for entry in self.state.flows.iter() {
            let key = entry.key();
            let flow = entry.value();

            // Look up device IDs
            let src_device_id = self.device_ids.get(&key.src_mac).copied();
            let dst_device_id = self.device_ids.get(&key.dst_mac).copied();

            match self.db.upsert_flow(key, flow, src_device_id, dst_device_id).await {
                Ok(_flow_id) => {
                    count += 1;
                }
                Err(e) => {
                    debug!("Failed to persist flow: {}", e);
                }
            }
        }

        Ok(count)
    }

    /// Persist protocol statistics
    async fn persist_protocols(&self) -> Result<usize> {
        let mut count = 0;

        for entry in self.state.protocols.iter() {
            let (ethertype, ip_protocol) = entry.key();
            let stats = entry.value();

            if let Err(e) = self.db.upsert_protocol(*ethertype, *ip_protocol, stats).await {
                debug!("Failed to persist protocol stats: {}", e);
            } else {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Persist VLAN statistics
    async fn persist_vlans(&self) -> Result<usize> {
        let mut count = 0;

        for entry in self.state.vlans.iter() {
            let vlan_id = *entry.key();
            let stats = entry.value();

            if let Err(e) = self.db.upsert_vlan(vlan_id, stats.outer_vlan_id, stats).await {
                debug!("Failed to persist VLAN stats: {}", e);
            } else {
                count += 1;
            }
        }

        Ok(count)
    }
}
