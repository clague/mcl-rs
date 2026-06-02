// Add Version Module
// Handles fetching Minecraft version manifest from Mojang and selecting versions to add

use iced::widget::{button, column, container, row, text, scrollable, radio, pick_list};
use iced::{Element, Length, Alignment, Task};

use crate::core::version::{Version, VersionInfo, VersionManifest, VersionType};
use crate::gui::styles;
use crate::i18n::strings;

#[derive(Debug, Clone)]
pub enum Message {
    ShowAddVersion,
    ManifestLoaded(Result<VersionManifest, String>),
    FilterChanged(VersionFilter),
    VersionSelected(usize),
    ConfirmAdd,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionFilter {
    Release,
    Snapshot,
    All,
}

impl VersionFilter {
    pub fn all() -> Vec<Self> {
        vec![Self::Release, Self::Snapshot, Self::All]
    }
    
    pub fn display_name(&self) -> &'static str {
        let s = strings();
        match self {
            Self::Release => s.release,
            Self::Snapshot => s.snapshot,
            Self::All => s.all,
        }
    }
}

impl std::fmt::Display for VersionFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

pub struct AddVersion {
    is_visible: bool,
    manifest: Option<VersionManifest>,
    filtered_versions: Vec<VersionInfo>,
    selected_index: Option<usize>,
    filter: VersionFilter,
    is_loading: bool,
    error: Option<String>,
}

impl AddVersion {
    pub fn new() -> Self {
        Self {
            is_visible: false,
            manifest: None,
            filtered_versions: Vec::new(),
            selected_index: None,
            filter: VersionFilter::Release,
            is_loading: false,
            error: None,
        }
    }

    pub fn show(&mut self) {
        self.is_visible = true;
        self.is_loading = true;
        self.error = None;
        self.selected_index = None;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn get_selected_version(&self) -> Option<&VersionInfo> {
        self.selected_index.and_then(|i| self.filtered_versions.get(i))
    }

    fn apply_filter(&mut self) {
        if let Some(ref manifest) = self.manifest {
            let version_type = match self.filter {
                VersionFilter::Release => Some(VersionType::Release),
                VersionFilter::Snapshot => Some(VersionType::Snapshot),
                VersionFilter::All => None,
            };
            self.filtered_versions = manifest.filter_versions(version_type, None).into_iter().cloned().collect();
            self.selected_index = None;
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowAddVersion => {
                self.show();
                Task::perform(async { Version::fetch_manifest().await }, Message::ManifestLoaded)
            }
            Message::ManifestLoaded(result) => {
                self.is_loading = false;
                match result {
                    Ok(manifest) => { self.manifest = Some(manifest); self.apply_filter(); }
                    Err(e) => { self.error = Some(e); }
                }
                Task::none()
            }
            Message::FilterChanged(filter) => { self.filter = filter; self.apply_filter(); Task::none() }
            Message::VersionSelected(index) => { self.selected_index = Some(index); Task::none() }
            Message::ConfirmAdd => { if self.selected_index.is_some() { self.is_visible = false; } Task::none() }
            Message::Cancel => { self.is_visible = false; Task::none() }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let s = strings();
        
        if !self.is_visible {
            return container(text("")).width(Length::Fill).height(Length::Fill).into();
        }

        let title = text(s.add_new_version).size(24);

        let filter_row = row![
            text(s.filter).size(14),
            pick_list(VersionFilter::all(), Some(self.filter.clone()), Message::FilterChanged)
                .placeholder(s.select_filter)
                .padding([8, 12]),
        ].spacing(10).align_y(Alignment::Center);

        let content: Element<Message> = if self.is_loading {
            container(
                column![text(s.loading_versions).size(16), text(s.please_wait).size(14)]
                    .spacing(10).align_x(Alignment::Center)
            ).center_x(Length::Fill).center_y(Length::Fill).into()
        } else if let Some(ref error) = self.error {
            column![
                text(s.error.replace("{}", error)).size(16),
                button(s.retry).on_press(Message::ShowAddVersion).padding([10, 20]).style(styles::button_primary),
            ].spacing(10).into()
        } else {
            let versions_list = self.filtered_versions.iter().enumerate().fold(
                column![].spacing(4),
                |col, (index, version)| {
                    let version_type = if version.version_type == "release" { s.release } else { s.snapshot };
                    let label = format!("{} ({})", version.id, version_type);
                    col.push(radio(label, index, self.selected_index, Message::VersionSelected).style(styles::radio_style))
                },
            );

            let selected_text = if let Some(index) = self.selected_index {
                if let Some(version) = self.filtered_versions.get(index) {
                    text(s.selected.replace("{}", &version.id)).size(14)
                } else { text("").size(14) }
            } else {
                text(s.select_a_version).size(14)
            };

            column![
                scrollable(versions_list).height(Length::Fill),
                container(selected_text).padding([8, 0]).align_x(Alignment::Center),
            ].spacing(10).into()
        };

        let confirm_button = button(s.add_version)
            .on_press_maybe(if self.selected_index.is_some() { Some(Message::ConfirmAdd) } else { None })
            .padding([12, 24]).style(styles::button_success);

        let cancel_button = button(s.cancel)
            .on_press(Message::Cancel)
            .padding([12, 24]).style(styles::button_secondary);

        let buttons = row![cancel_button, confirm_button].spacing(10).align_y(Alignment::Center);

        let main_content = column![title, filter_row, content, buttons]
            .spacing(15).padding(25);

        container(main_content)
            .width(Length::Fill).height(Length::Fill)
            .center_x(Length::Fill).center_y(Length::Fill)
            .style(styles::card_container)
            .into()
    }
}