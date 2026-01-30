//! NetSentinel Aggregator Module
//!
//! Aggregates captured network data and persists to PostgreSQL/TimescaleDB.

pub mod config;
pub mod db;
pub mod pipeline;
pub mod state;

pub use config::Config;
pub use db::Database;
pub use pipeline::Pipeline;
pub use state::AggregatorState;
