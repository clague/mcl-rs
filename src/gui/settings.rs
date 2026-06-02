// Settings Module
// Handles application settings like Java path, memory allocation, etc.

use iced::widget::{button, column, container, row, text, text_input, checkbox};
use iced::{Element, Length, Alignment, Task};

use crate::config::config::Config;
use crate::gui::styles;
use crate::i18n::strings;

#[derive(Debug, Clone)]
pub enum Message {
    ShowSettings,
    JavaPathChanged(String),
    MemoryChanged(String),
    AutoUpdateChanged(bool),
    SaveSettings,
    CancelSettings,
}

pub struct Settings {
    java_path: String,
    memory: String,
    auto_update: bool,
    is_visible: bool,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            java_path: String::new(),
            memory: String::new(),
            auto_update: true,
            is_visible: false,
        }
    }

    pub fn load_from_config(&mut self, config: &Config) {
        self.java_path = config.java_path.clone().unwrap_or_default();
        self.memory = config.memory.to_string();
        self.auto_update = config.auto_update;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn get_java_path(&self) -> Option<String> {
        if self.java_path.is_empty() { None } else { Some(self.java_path.clone()) }
    }

    pub fn get_memory(&self) -> Option<u32> {
        self.memory.parse().ok()
    }

    pub fn get_auto_update(&self) -> bool {
        self.auto_update
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowSettings => { self.is_visible = true; Task::none() }
            Message::JavaPathChanged(path) => { self.java_path = path; Task::none() }
            Message::MemoryChanged(memory) => { self.memory = memory; Task::none() }
            Message::AutoUpdateChanged(auto_update) => { self.auto_update = auto_update; Task::none() }
            Message::SaveSettings => { self.is_visible = false; Task::none() }
            Message::CancelSettings => { self.is_visible = false; Task::none() }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let s = strings();
        
        if !self.is_visible {
            return container(text("")).width(Length::Fill).height(Length::Fill).into();
        }

        let title = text(s.settings).size(24);

        let java_path_input = column![
            text(s.java_path).size(14),
            text_input(s.java_path_hint, &self.java_path)
                .on_input(Message::JavaPathChanged)
                .padding(10)
                .style(styles::text_input_style),
        ].spacing(5);

        let memory_input = column![
            text(s.memory_mb).size(14),
            text_input("2048", &self.memory)
                .on_input(Message::MemoryChanged)
                .padding(10)
                .style(styles::text_input_style),
        ].spacing(5);

        let auto_update_row = row![
            checkbox(self.auto_update).on_toggle(Message::AutoUpdateChanged).style(styles::checkbox_style),
            text(s.auto_update_on_startup).size(14),
        ].spacing(10).align_y(Alignment::Center);

        let save_button = button(s.save)
            .on_press(Message::SaveSettings)
            .width(Length::Fill)
            .padding(12)
            .style(styles::button_primary);

        let cancel_button = button(s.cancel)
            .on_press(Message::CancelSettings)
            .width(Length::Fill)
            .padding(12)
            .style(styles::button_secondary);

        let content = column![title, java_path_input, memory_input, auto_update_row, row![cancel_button, save_button].spacing(10)]
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