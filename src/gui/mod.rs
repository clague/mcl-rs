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

/// Mod management panel - displays and manages installed mods
pub mod mod_panel;

/// Mod search dialog - search and browse mods on Modrinth
pub mod mod_search;

/// Custom styles with rounded corners
pub mod styles;