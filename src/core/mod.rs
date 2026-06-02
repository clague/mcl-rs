// Core Module
// Contains business logic and data structures for the launcher

/// Account management and storage
pub mod account;

/// Minecraft version manifest and version data structures
pub mod version;

/// File download management with progress tracking
pub mod download;

/// Microsoft/Xbox OAuth authentication flow
pub mod auth;

/// Update checking and version comparison
pub mod update;

/// Game launching logic
pub mod launch;