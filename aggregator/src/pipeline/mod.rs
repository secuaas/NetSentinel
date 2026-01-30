//! Pipeline module for data processing

pub mod consumer;
pub mod persister;

pub use consumer::RedisConsumer;
pub use persister::Persister;

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, error};
use anyhow::Result;

use crate::config::Config;
use crate::state::AggregatorState;
use crate::db::Database;

/// Main pipeline orchestrator
pub struct Pipeline {
    config: Config,
    state: Arc<AggregatorState>,
    db: Arc<Database>,
    shutdown_tx: broadcast::Sender<()>,
}

impl Pipeline {
    /// Create a new pipeline
    pub async fn new(config: Config) -> Result<Self> {
        let state = Arc::new(AggregatorState::new());
        let db = Arc::new(Database::connect(&config.database).await?);
        let (shutdown_tx, _) = broadcast::channel(1);

        Ok(Self {
            config,
            state,
            db,
            shutdown_tx,
        })
    }

    /// Get the aggregator state
    pub fn state(&self) -> Arc<AggregatorState> {
        Arc::clone(&self.state)
    }

    /// Get the database
    pub fn database(&self) -> Arc<Database> {
        Arc::clone(&self.db)
    }

    /// Start the pipeline
    pub async fn run(&self) -> Result<()> {
        info!("Starting aggregation pipeline");

        // Create shutdown receivers for each component
        let consumer_shutdown = self.shutdown_tx.subscribe();
        let persister_shutdown = self.shutdown_tx.subscribe();
        let events_shutdown = self.shutdown_tx.subscribe();

        // Start Redis consumer
        let consumer = RedisConsumer::new(
            self.config.redis.clone(),
            Arc::clone(&self.state),
        );
        let consumer_handle = tokio::spawn(async move {
            if let Err(e) = consumer.run(consumer_shutdown).await {
                error!("Consumer error: {}", e);
            }
        });

        // Start persister
        let persister = Persister::new(
            self.config.aggregation.clone(),
            Arc::clone(&self.state),
            Arc::clone(&self.db),
        );
        let persister_handle = tokio::spawn(async move {
            if let Err(e) = persister.run(persister_shutdown).await {
                error!("Persister error: {}", e);
            }
        });

        // Start event publisher (optional)
        let events_handle = if self.config.events.publish_new_devices ||
                              self.config.events.publish_new_flows {
            let config = self.config.clone();
            let state = Arc::clone(&self.state);
            Some(tokio::spawn(async move {
                // Event publishing logic would go here
                let mut shutdown = events_shutdown;
                loop {
                    tokio::select! {
                        _ = shutdown.recv() => {
                            info!("Event publisher shutting down");
                            break;
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                            // Check for new events to publish
                        }
                    }
                }
            }))
        } else {
            None
        };

        // Wait for all tasks
        let _ = consumer_handle.await;
        let _ = persister_handle.await;
        if let Some(h) = events_handle {
            let _ = h.await;
        }

        info!("Pipeline stopped");
        Ok(())
    }

    /// Signal shutdown
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}
