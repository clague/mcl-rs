// Mod Panel Module
// Displays and manages installed mods for a selected Minecraft version

use iced::widget::{button, checkbox, column, container, row, scrollable, text};
use iced::{Element, Length, Alignment, Task};

use crate::config::config::InstalledModInfo;
use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    Show(String, Vec<InstalledModInfo>),
    Hide,
    ModSelected(usize),
    ToggleMod(String),
    RemoveMod(String),
    SearchMods,
}

pub struct ModPanel {
    installed_mods: Vec<InstalledModInfo>,
    selected_mod: Option<usize>,
    is_visible: bool,
    selected_version: Option<String>,
}

impl ModPanel {
    pub fn new() -> Self {
        Self {
            installed_mods: Vec::new(),
            selected_mod: None,
            is_visible: false,
            selected_version: None,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Returns the currently selected mod (if any)
    pub fn get_selected_mod(&self) -> Option<&InstalledModInfo> {
        self.selected_mod.and_then(|i| self.installed_mods.get(i))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Show(version_uuid, mods) => {
                self.selected_version = Some(version_uuid);
                if mods.is_empty() {
                    let config = crate::config::config::Config::load();
                    self.installed_mods = config.get_mods(self.selected_version.as_ref().unwrap());
                } else {
                    self.installed_mods = mods;
                }
                self.selected_mod = None;
                self.is_visible = true;
                Task::none()
            }
            Message::Hide => {
                self.is_visible = false;
                Task::none()
            }
            Message::ModSelected(index) => {
                if index < self.installed_mods.len() {
                    if self.selected_mod == Some(index) {
                        self.selected_mod = None;
                    } else {
                        self.selected_mod = Some(index);
                    }
                }
                Task::none()
            }
            Message::ToggleMod(project_id) => {
                if let Some(m) = self.installed_mods.iter_mut().find(|m| m.project_id == project_id) {
                    m.enabled = !m.enabled;
                }
                Task::none()
            }
            Message::RemoveMod(project_id) => {
                self.installed_mods.retain(|m| m.project_id != project_id);
                // Clear selection if the removed mod was selected
                if let Some(idx) = self.selected_mod {
                    if idx >= self.installed_mods.len() {
                        self.selected_mod = None;
                    }
                }
                Task::none()
            }
            Message::SearchMods => {
                // Placeholder: will open mod search dialog in the future
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if !self.is_visible {
            return container(text("")).width(Length::Fill).height(Length::Fill).into();
        }

        let title = text("Mods").size(26);

        let search_button = button("Search Mods")
            .on_press(Message::SearchMods)
            .padding([10, 20])
            .style(styles::button_primary);

        let header = row![title, search_button]
            .spacing(10)
            .align_y(Alignment::Center);

        let content: Element<'_, Message> = if self.installed_mods.is_empty() {
            container(
                column![
                    text("No mods installed").size(18),
                    text("Use \"Search Mods\" to find and install mods").size(14),
                ]
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(30)
            .into()
        } else {
            let mods_list = self.installed_mods.iter().enumerate().fold(
                column![].spacing(8),
                |col, (index, mod_info)| {
                    let is_selected = self.selected_mod == Some(index);

                    let title_text = text(&mod_info.title).size(16);
                    let version_text = text(format!("v{}", mod_info.version_number)).size(13);

                    let project_id = mod_info.project_id.clone();
                    let toggle = checkbox(mod_info.enabled)
                        .on_toggle(move |_| Message::ToggleMod(project_id.clone()))
                        .style(styles::checkbox_style);

                    let remove_btn = button("Remove")
                        .on_press(Message::RemoveMod(mod_info.project_id.clone()))
                        .padding([4, 10])
                        .style(styles::button_danger);

                    let mod_info_col = column![title_text, version_text].spacing(4);

                    let controls = row![toggle, remove_btn]
                        .spacing(8)
                        .align_y(Alignment::Center);

                    let mod_row = row![mod_info_col, controls]
                        .spacing(10)
                        .align_y(Alignment::Center);

                    let mod_card = container(mod_row)
                        .padding(10)
                        .width(Length::Fill)
                        .style(styles::panel_container);

                    let clickable = button(mod_card)
                        .on_press(Message::ModSelected(index))
                        .padding(0)
                        .width(Length::Fill)
                        .style(if is_selected {
                            styles::button_primary
                        } else {
                            styles::button_icon
                        });

                    col.push(clickable)
                },
            );

            scrollable(mods_list).height(Length::Fill).into()
        };

        let mod_count = text(format!("{} mod(s) installed", self.installed_mods.len())).size(14);

        let close_button = button("Close")
            .on_press(Message::Hide)
            .padding([10, 20])
            .style(styles::button_secondary);

        let bottom_bar = row![mod_count, close_button]
            .spacing(10)
            .align_y(Alignment::Center);

        let main_content = column![header, content, bottom_bar]
            .spacing(15)
            .padding(25)
            .max_width(600);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::card_container)
            .into()
    }
}
