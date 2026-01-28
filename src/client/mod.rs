//! BYOND Client Protocol Implementation
//!
//! This module implements a minimal BYOND client that can connect to a DreamDaemon
//! server as a guest and receive visual data for screenshot capture.

pub mod crypto;
pub mod packets;
pub mod protocol;

pub use protocol::BYONDClient;
