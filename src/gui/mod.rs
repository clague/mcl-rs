// GUI Module
// Contains all user interface components for the launcher

/// Main application window - coordinates between different views
pub mod main_window;

/// Login dialog - handles Microsoft/Xbox OAuth authentication
pub mod login;

/// Settings dialog - Java path, memory, and other configuration
pub mod settings;

/// Add version dialog - fetches and displays available Minecraft versions
pub mod add_version;

/// Custom styles with rounded corners
pub mod styles;