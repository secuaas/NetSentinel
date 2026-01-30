//! Redis Stream consumer for captured frames

use anyhow::{Context, Result};
use redis::aio::MultiplexedConnection;
use redis::Client;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::config::RedisConfig;
use crate::state::{AggregatorState, CapturedFrame};

/// Redis stream consumer
pub struct RedisConsumer {
    config: RedisConfig,
    state: Arc<AggregatorState>,
}

impl RedisConsumer {
    /// Create a new consumer
    pub fn new(config: RedisConfig, state: Arc<AggregatorState>) -> Self {
        Self { config, state }
    }

    /// Connect to Redis
    async fn connect(&self) -> Result<MultiplexedConnection> {
        let client = Client::open(self.config.url.as_str())
            .with_context(|| format!("Failed to create Redis client: {}", self.config.url))?;

        let conn = client
            .get_multiplexed_async_connection()
            .await
            .with_context(|| "Failed to connect to Redis")?;

        info!("Connected to Redis at {}", self.config.url);
        Ok(conn)
    }

    /// Ensure consumer group exists
    async fn ensure_consumer_group(&self, conn: &mut MultiplexedConnection) -> Result<()> {
        let result: redis::RedisResult<()> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&self.config.stream_name)
            .arg(&self.config.consumer_group)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(conn)
            .await;

        match result {
            Ok(()) => {
                info!("Created consumer group '{}'", self.config.consumer_group);
            }
            Err(e) if e.to_string().contains("BUSYGROUP") => {
                debug!("Consumer group '{}' already exists", self.config.consumer_group);
            }
            Err(e) => {
                return Err(e).with_context(|| "Failed to create consumer group");
            }
        }

        Ok(())
    }

    /// Run the consumer loop
    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut conn = self.connect().await?;
        self.ensure_consumer_group(&mut conn).await?;

        let stream_name = &self.config.stream_name;
        let group_name = &self.config.consumer_group;
        let consumer_name = &self.config.consumer_name;
        let batch_size = self.config.batch_size;
        let block_ms = self.config.block_ms;

        info!(
            "Starting consumer: stream={}, group={}, consumer={}, batch={}",
            stream_name, group_name, consumer_name, batch_size
        );

        let mut processed_count: u64 = 0;
        let mut last_log = std::time::Instant::now();

        loop {
            // Check for shutdown
            if shutdown.try_recv().is_ok() {
                info!("Consumer received shutdown signal");
                break;
            }

            // Read from stream using consumer group
            let result: redis::RedisResult<redis::Value> = redis::cmd("XREADGROUP")
                .arg("GROUP")
                .arg(group_name)
                .arg(consumer_name)
                .arg("COUNT")
                .arg(batch_size)
                .arg("BLOCK")
                .arg(block_ms)
                .arg("STREAMS")
                .arg(stream_name)
                .arg(">")
                .query_async(&mut conn)
                .await;

            match result {
                Ok(redis::Value::Nil) => {
                    // No messages available, continue
                    continue;
                }
                Ok(value) => {
                    // Parse and process messages
                    if let Some(entries) = self.parse_stream_response(&value) {
                        for (entry_id, data) in entries {
                            if let Some(frame) = self.parse_frame_data(&data) {
                                // Process the frame
                                let result = self.state.process_frame(&frame);

                                // Log new devices/flows
                                for mac in &result.new_devices {
                                    debug!("New device discovered: {}", mac.to_string());
                                }
                                for flow in &result.new_flows {
                                    debug!("New flow: {}:{} -> {}:{}",
                                        flow.src_mac.to_string(),
                                        flow.src_port.unwrap_or(0),
                                        flow.dst_mac.to_string(),
                                        flow.dst_port.unwrap_or(0)
                                    );
                                }

                                processed_count += 1;

                                // Acknowledge the message
                                let _: redis::RedisResult<i64> = redis::cmd("XACK")
                                    .arg(stream_name)
                                    .arg(group_name)
                                    .arg(&entry_id)
                                    .query_async(&mut conn)
                                    .await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stream: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }

            // Periodic stats logging
            if last_log.elapsed().as_secs() >= 10 {
                let stats = self.state.stats_snapshot();
                info!(
                    "Stats: packets={}, bytes={}, devices={}, flows={}",
                    stats.total_packets, stats.total_bytes,
                    stats.total_devices, stats.total_flows
                );
                last_log = std::time::Instant::now();
            }
        }

        info!("Consumer stopped. Total processed: {}", processed_count);
        Ok(())
    }

    /// Parse Redis stream response into entry ID and data pairs
    fn parse_stream_response(&self, value: &redis::Value) -> Option<Vec<(String, String)>> {
        // Response format: [[stream_name, [[entry_id, [field, value, ...]], ...]]]
        let mut entries = Vec::new();

        if let redis::Value::Bulk(streams) = value {
            for stream in streams {
                if let redis::Value::Bulk(stream_data) = stream {
                    if stream_data.len() >= 2 {
                        if let redis::Value::Bulk(messages) = &stream_data[1] {
                            for message in messages {
                                if let redis::Value::Bulk(msg_data) = message {
                                    if msg_data.len() >= 2 {
                                        let entry_id = self.value_to_string(&msg_data[0]);
                                        if let redis::Value::Bulk(fields) = &msg_data[1] {
                                            // Look for "data" field
                                            let mut i = 0;
                                            while i < fields.len() - 1 {
                                                if let Some(key) = self.value_to_string(&fields[i]) {
                                                    if key == "data" {
                                                        if let Some(data) = self.value_to_string(&fields[i + 1]) {
                                                            if let Some(id) = entry_id.clone() {
                                                                entries.push((id, data));
                                                            }
                                                        }
                                                    }
                                                }
                                                i += 2;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if entries.is_empty() {
            None
        } else {
            Some(entries)
        }
    }

    /// Convert Redis Value to String
    fn value_to_string(&self, value: &redis::Value) -> Option<String> {
        match value {
            redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
            redis::Value::Status(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Parse frame data from JSON
    fn parse_frame_data(&self, data: &str) -> Option<CapturedFrame> {
        match serde_json::from_str(data) {
            Ok(frame) => Some(frame),
            Err(e) => {
                warn!("Failed to parse frame data: {}", e);
                None
            }
        }
    }
}
