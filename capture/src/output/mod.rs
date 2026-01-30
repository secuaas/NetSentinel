//! Output module for sending captured frames to destinations

pub mod redis;

pub use redis::RedisOutput;
