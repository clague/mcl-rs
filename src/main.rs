// Minecraft Launcher - Main Entry Point
// A Minecraft Java Edition launcher built with Iced GUI framework

#![allow(dead_code)]

mod gui;
mod core;
mod config;
mod utils;
mod i18n;

use iced::{Task, Settings};
use log::info;

use gui::main_window::{MainWindow, Message};
use utils::font;

/// Boot function called once at application startup.
fn boot() -> (MainWindow, Task<Message>) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    info!("Minecraft Launcher starting...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    
    let config = config::config::Config::load();
    utils::net::init_client(config.max_connections);
    
    let font_family = font::get_default_font_family();
    info!("Detected font family: {}", font_family);
    
    let main_window = MainWindow::new();
    let startup_task = main_window.startup_tasks();
    
    (main_window, startup_task)
}

/// Application entry point.
fn main() -> iced::Result {
    // Get platform-specific font
    let default_font = font::get_system_font();
    info!("Using font: {:?}", default_font);
    
    iced::application(boot, MainWindow::update, MainWindow::view)
        .subscription(MainWindow::subscription)
        .settings(Settings {
            default_font,
            ..Settings::default()
        })
        .run()
}