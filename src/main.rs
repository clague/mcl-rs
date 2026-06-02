// Minecraft Launcher - Main Entry Point
// A Minecraft Java Edition launcher built with Iced GUI framework

#![allow(dead_code)]

mod gui;
mod core;
mod config;
mod utils;

use iced::Task;
use log::info;

use gui::main_window::{MainWindow, Message};

/// Boot function called once at application startup.
/// Returns the initial state and any startup tasks.
fn boot() -> (MainWindow, Task<Message>) {
    // Initialize the logger with default level info
    // Set RUST_LOG=debug for more verbose output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    info!("Minecraft Launcher starting...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    
    let main_window = MainWindow::new();
    
    // Run startup tasks (token refresh and update check)
    let startup_task = main_window.startup_tasks();
    
    (main_window, startup_task)
}

/// Application entry point.
/// Initializes the Iced application with the boot, update, and view functions.
fn main() -> iced::Result {
    iced::application(boot, MainWindow::update, MainWindow::view)
        .subscription(MainWindow::subscription)
        .run()
}