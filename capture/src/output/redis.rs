//! Redis Streams output for captured frames

use anyhow::{Context, Result, bail};
use redis::{Client, RedisResult};
use redis::aio::MultiplexedConnection;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::capture::frame::CapturedFrame;
use crate::config::RedisConfig;

/// Output statistics
#[derive(Debug, Default)]
pub struct OutputStats {
    /// Frames sent successfully
    pub frames_sent: AtomicU64,
    /// Frames dropped (channel full)
    pub frames_dropped: AtomicU64,
    /// Send errors
    pub send_errors: AtomicU64,
    /// Total bytes sent
    pub bytes_sent: AtomicU64,
}

/// Redis Streams output
pub struct RedisOutput {
    config: RedisConfig,
    stats: Arc<OutputStats>,
}

impl RedisOutput {
    /// Create a new Redis output
    pub fn new(config: RedisConfig) -> Self {
        Self {
            config,
            stats: Arc::new(OutputStats::default()),
        }
    }

    /// Get output statistics
    pub fn stats(&self) -> Arc<OutputStats> {
        Arc::clone(&self.stats)
    }

    /// Connect to Redis and return an async connection
    pub async fn connect(&self) -> Result<MultiplexedConnection> {
        let client = Client::open(self.config.url.as_str())
            .with_context(|| format!("Failed to create Redis client: {}", self.config.url))?;

        let conn = client
            .get_multiplexed_async_connection()
            .await
            .with_context(|| "Failed to connect to Redis")?;

        info!("Connected to Redis at {}", self.config.url);
        Ok(conn)
    }

    /// Start the output loop that consumes frames from a channel and sends to Redis
    pub async fn run(
        &self,
        mut frame_rx: mpsc::Receiver<CapturedFrame>,
        batch_size: usize,
        flush_interval_ms: u64,
    ) -> Result<()> {
        let mut conn = self.connect().await?;
        let stream_name = &self.config.stream_name;
        let max_len = self.config.max_stream_length;
        let stats = Arc::clone(&self.stats);

        let mut batch: Vec<CapturedFrame> = Vec::with_capacity(batch_size);
        let flush_interval = Duration::from_millis(flush_interval_ms);
        let mut last_flush = std::time::Instant::now();

        info!(
            "Redis output started: stream={}, batch_size={}, flush_interval={}ms",
            stream_name, batch_size, flush_interval_ms
        );

        loop {
            // Try to receive with timeout
            match tokio::time::timeout(flush_interval, frame_rx.recv()).await {
                Ok(Some(frame)) => {
                    batch.push(frame);

                    // Flush if batch is full
                    if batch.len() >= batch_size {
                        if let Err(e) = Self::flush_batch(&mut conn, stream_name, max_len, &batch, &stats).await {
                            error!("Failed to flush batch: {}", e);
                        }
                        batch.clear();
                        last_flush = std::time::Instant::now();
                    }
                }
                Ok(None) => {
                    // Channel closed
                    info!("Frame channel closed, flushing remaining frames");
                    if !batch.is_empty() {
                        if let Err(e) = Self::flush_batch(&mut conn, stream_name, max_len, &batch, &stats).await {
                            error!("Failed to flush final batch: {}", e);
                        }
                    }
                    break;
                }
                Err(_) => {
                    // Timeout - check if we need to flush
                    if !batch.is_empty() && last_flush.elapsed() >= flush_interval {
                        if let Err(e) = Self::flush_batch(&mut conn, stream_name, max_len, &batch, &stats).await {
                            error!("Failed to flush batch on timeout: {}", e);
                        }
                        batch.clear();
                        last_flush = std::time::Instant::now();
                    }
                }
            }
        }

        info!("Redis output stopped");
        Ok(())
    }

    /// Flush a batch of frames to Redis Stream
    async fn flush_batch(
        conn: &mut MultiplexedConnection,
        stream_name: &str,
        max_len: usize,
        batch: &[CapturedFrame],
        stats: &OutputStats,
    ) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        // Use pipeline for batch writes
        let mut pipe = redis::pipe();

        for frame in batch {
            let json = serde_json::to_string(frame)
                .with_context(|| "Failed to serialize frame")?;

            // XADD with MAXLEN ~ for approximate trimming
            pipe.cmd("XADD")
                .arg(stream_name)
                .arg("MAXLEN")
                .arg("~")
                .arg(max_len)
                .arg("*")
                .arg("data")
                .arg(&json);

            stats.bytes_sent.fetch_add(json.len() as u64, Ordering::Relaxed);
        }

        // Execute pipeline
        let _: Vec<String> = pipe.query_async(conn).await
            .with_context(|| "Failed to execute Redis pipeline")?;

        stats.frames_sent.fetch_add(batch.len() as u64, Ordering::Relaxed);

        debug!("Flushed {} frames to Redis stream '{}'", batch.len(), stream_name);

        Ok(())
    }

    /// Send a single frame to Redis (for testing or low-volume scenarios)
    pub async fn send_frame(&self, conn: &mut MultiplexedConnection, frame: &CapturedFrame) -> Result<String> {
        let json = serde_json::to_string(frame)
            .with_context(|| "Failed to serialize frame")?;

        let entry_id: String = redis::cmd("XADD")
            .arg(&self.config.stream_name)
            .arg("MAXLEN")
            .arg("~")
            .arg(self.config.max_stream_length)
            .arg("*")
            .arg("data")
            .arg(&json)
            .query_async(conn)
            .await
            .with_context(|| "Failed to XADD to Redis stream")?;

        self.stats.frames_sent.fetch_add(1, Ordering::Relaxed);
        self.stats.bytes_sent.fetch_add(json.len() as u64, Ordering::Relaxed);

        Ok(entry_id)
    }
}

/// Create a consumer group for the stream if it doesn't exist
pub async fn ensure_consumer_group(
    conn: &mut MultiplexedConnection,
    stream_name: &str,
    group_name: &str,
) -> Result<()> {
    // Try to create the group, ignore error if it already exists
    let result: RedisResult<()> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(stream_name)
        .arg(group_name)
        .arg("0")
        .arg("MKSTREAM")
        .query_async(conn)
        .await;

    match result {
        Ok(()) => {
            info!("Created consumer group '{}' on stream '{}'", group_name, stream_name);
        }
        Err(e) if e.to_string().contains("BUSYGROUP") => {
            debug!("Consumer group '{}' already exists", group_name);
        }
        Err(e) => {
            bail!("Failed to create consumer group: {}", e);
        }
    }

    Ok(())
}

/// Get stream information using XLEN command
pub async fn stream_info(conn: &mut MultiplexedConnection, stream_name: &str) -> Result<StreamInfo> {
    // Use XLEN for simple length query (most compatible approach)
    let length: u64 = redis::cmd("XLEN")
        .arg(stream_name)
        .query_async(conn)
        .await
        .unwrap_or(0);

    Ok(StreamInfo {
        length,
        first_entry: None,
        last_entry: None,
    })
}

/// Stream information
#[derive(Debug)]
pub struct StreamInfo {
    pub length: u64,
    pub first_entry: Option<String>,
    pub last_entry: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::frame::MacAddr;

    fn test_frame() -> CapturedFrame {
        CapturedFrame::new(
            "eth0",
            MacAddr::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
            MacAddr::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
            0x0800,
            64,
        )
    }

    #[test]
    fn test_frame_serialization() {
        let frame = test_frame();
        let json = serde_json::to_string(&frame).unwrap();

        assert!(json.contains("eth0"));
        assert!(json.contains("00:11:22:33:44:55"));
        assert!(json.contains("ff:ff:ff:ff:ff:ff"));
    }

    #[tokio::test]
    #[ignore] // Requires running Redis
    async fn test_redis_connection() {
        let config = RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            stream_name: "test:frames".to_string(),
            max_stream_length: 1000,
            pool_size: 1,
        };

        let output = RedisOutput::new(config);
        let conn = output.connect().await;

        assert!(conn.is_ok());
    }
}
