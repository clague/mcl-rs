// Settings Module
// Handles application settings like Java path, memory allocation, etc.

use iced::widget::{button, column, container, row, text, text_input, checkbox};
use iced::{Element, Length, Alignment, Task};

use crate::config::config::Config;
use crate::gui::styles;

/// Messages that can be dispatched to the settings component
#[derive(Debug, Clone)]
pub enum Message {
    /// Show the settings dialog
    ShowSettings,
    /// Java path input changed
    JavaPathChanged(String),
    /// Memory allocation input changed
    MemoryChanged(String),
    /// Auto-update toggle changed
    AutoUpdateChanged(bool),
    /// Save settings and close dialog
    SaveSettings,
    /// Cancel and close dialog
    CancelSettings,
}

/// Settings component state
pub struct Settings {
    /// Path to Java executable
    java_path: String,
    /// Memory allocation in MB
    memory: String,
    /// Whether to auto-update game files
    auto_update: bool,
    /// Whether the settings dialog is visible
    is_visible: bool,
}

impl Settings {
    /// Creates a new Settings component with default values
    pub fn new() -> Self {
        Self {
            java_path: String::new(),
            memory: String::new(),
            auto_update: true,
            is_visible: false,
        }
    }

    /// Loads settings from config
    pub fn load_from_config(&mut self, config: &Config) {
        self.java_path = config.java_path.clone().unwrap_or_default();
        self.memory = config.memory.to_string();
        self.auto_update = config.auto_update;
    }

    /// Returns true if the settings dialog should be visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Returns the Java path (if set)
    pub fn get_java_path(&self) -> Option<String> {
        if self.java_path.is_empty() {
            None
        } else {
            Some(self.java_path.clone())
        }
    }

    /// Returns the memory allocation in MB
    pub fn get_memory(&self) -> Option<u32> {
        self.memory.parse().ok()
    }

    /// Returns the auto-update setting
    pub fn get_auto_update(&self) -> bool {
        self.auto_update
    }

    /// Handles incoming messages and updates state accordingly
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowSettings => {
                self.is_visible = true;
                Task::none()
            }
            Message::JavaPathChanged(path) => {
                self.java_path = path;
                Task::none()
            }
            Message::MemoryChanged(memory) => {
                self.memory = memory;
                Task::none()
            }
            Message::AutoUpdateChanged(auto_update) => {
                self.auto_update = auto_update;
                Task::none()
            }
            Message::SaveSettings => {
                self.is_visible = false;
                Task::none()
            }
            Message::CancelSettings => {
                self.is_visible = false;
                Task::none()
            }
        }
    }

    /// Renders the settings dialog view
    pub fn view(&self) -> Element<'_, Message> {
        if !self.is_visible {
            return container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let title = text("Settings").size(24);

        let java_path_input = column![
            text("Java Path").size(14),
            text_input("Leave empty for auto-detect", &self.java_path)
                .on_input(Message::JavaPathChanged)
                .padding(10)
                .style(styles::text_input_style),
        ]
        .spacing(5);

        let memory_input = column![
            text("Memory (MB)").size(14),
            text_input("2048", &self.memory)
                .on_input(Message::MemoryChanged)
                .padding(10)
                .style(styles::text_input_style),
        ]
        .spacing(5);

        let auto_update_row = row![
            checkbox(self.auto_update)
                .on_toggle(Message::AutoUpdateChanged)
                .style(styles::checkbox_style),
            text("Auto-update on startup").size(14),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let save_button = button("Save")
            .on_press(Message::SaveSettings)
            .width(Length::Fill)
            .padding(12)
            .style(styles::button_primary);

        let cancel_button = button("Cancel")
            .on_press(Message::CancelSettings)
            .width(Length::Fill)
            .padding(12)
            .style(styles::button_secondary);

        let content = column![
            title,
            java_path_input,
            memory_input,
            auto_update_row,
            row![cancel_button, save_button].spacing(10),
        ]
        .spacing(20)
        .padding(25)
        .max_width(450);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::card_container)
            .into()
    }
}