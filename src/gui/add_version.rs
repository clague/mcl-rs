// Add Version Module
// Handles fetching Minecraft version manifest from Mojang and selecting versions to add

use iced::widget::{button, column, container, row, text, scrollable, radio, pick_list};
use iced::{Element, Length, Alignment, Task};

use crate::core::version::{Version, VersionInfo, VersionManifest, VersionType};
use crate::gui::styles;

/// Messages that can be dispatched to the add version component
#[derive(Debug, Clone)]
pub enum Message {
    /// Show the add version dialog
    ShowAddVersion,
    /// Version manifest loaded from Mojang API
    ManifestLoaded(Result<VersionManifest, String>),
    /// Version filter changed (Release/Snapshot/All)
    FilterChanged(VersionFilter),
    /// User selected a version from the list
    VersionSelected(usize),
    /// User confirmed adding the selected version
    ConfirmAdd,
    /// User cancelled adding a version
    Cancel,
}

/// Version type filter for the version list
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionFilter {
    /// Show only release versions
    Release,
    /// Show only snapshot versions
    Snapshot,
    /// Show all versions
    All,
}

impl VersionFilter {
    /// Returns all available filter options
    pub fn all() -> Vec<Self> {
        vec![Self::Release, Self::Snapshot, Self::All]
    }
}

impl std::fmt::Display for VersionFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Release => write!(f, "Release"),
            Self::Snapshot => write!(f, "Snapshot"),
            Self::All => write!(f, "All"),
        }
    }
}

/// Add version component state
pub struct AddVersion {
    /// Whether the dialog is visible
    is_visible: bool,
    /// Full version manifest from Mojang
    manifest: Option<VersionManifest>,
    /// Filtered list of versions based on current filter
    filtered_versions: Vec<VersionInfo>,
    /// Index of the currently selected version
    selected_index: Option<usize>,
    /// Current version type filter
    filter: VersionFilter,
    /// Whether we're currently loading the manifest
    is_loading: bool,
    /// Error message to display
    error: Option<String>,
}

impl AddVersion {
    /// Creates a new AddVersion component with initial state
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

    /// Shows the dialog and resets state for a new selection
    pub fn show(&mut self) {
        self.is_visible = true;
        self.is_loading = true;
        self.error = None;
        self.selected_index = None;
    }

    /// Hides the dialog
    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    /// Returns true if the dialog should be visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Returns the currently selected version info (if any)
    pub fn get_selected_version(&self) -> Option<&VersionInfo> {
        if let Some(index) = self.selected_index {
            self.filtered_versions.get(index)
        } else {
            None
        }
    }

    /// Applies the current filter to the version manifest
    fn apply_filter(&mut self) {
        if let Some(ref manifest) = self.manifest {
            let version_type = match self.filter {
                VersionFilter::Release => Some(VersionType::Release),
                VersionFilter::Snapshot => Some(VersionType::Snapshot),
                VersionFilter::All => None,
            };
            
            self.filtered_versions = manifest.filter_versions(version_type, None)
                .into_iter()
                .cloned()
                .collect();
            
            // Reset selection when filter changes
            self.selected_index = None;
        }
    }

    /// Handles incoming messages and updates state accordingly
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Show dialog and start loading manifest
            Message::ShowAddVersion => {
                self.show();
                Task::perform(
                    async { Version::fetch_manifest().await },
                    Message::ManifestLoaded,
                )
            }
            // Handle loaded manifest
            Message::ManifestLoaded(result) => {
                self.is_loading = false;
                match result {
                    Ok(manifest) => {
                        self.manifest = Some(manifest);
                        self.apply_filter();
                        Task::none()
                    }
                    Err(e) => {
                        self.error = Some(e);
                        Task::none()
                    }
                }
            }
            // Handle filter change
            Message::FilterChanged(filter) => {
                self.filter = filter;
                self.apply_filter();
                Task::none()
            }
            // Handle version selection
            Message::VersionSelected(index) => {
                self.selected_index = Some(index);
                Task::none()
            }
            // Confirm adding the selected version
            Message::ConfirmAdd => {
                if self.selected_index.is_some() {
                    self.is_visible = false;
                }
                Task::none()
            }
            // Cancel and close dialog
            Message::Cancel => {
                self.is_visible = false;
                Task::none()
            }
        }
    }

    /// Renders the add version dialog view
    pub fn view(&self) -> Element<'_, Message> {
        if !self.is_visible {
            return container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let title = text("Add New Version").size(24);

        // Version type filter dropdown
        let filter_row = row![
            text("Filter: ").size(14),
            pick_list(
                VersionFilter::all(),
                Some(self.filter.clone()),
                Message::FilterChanged,
            )
            .placeholder("Select filter...")
            .padding([8, 12]),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Version list or loading/error state
        let content: Element<Message> = if self.is_loading {
            // Loading state
            container(
                column![
                    text("Loading versions...").size(16),
                    text("Please wait...").size(14),
                ]
                .spacing(10)
                .align_x(Alignment::Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else if let Some(ref error) = self.error {
            // Error state with retry button
            column![
                text(format!("Error: {}", error)).size(16),
                button("Retry")
                    .on_press(Message::ShowAddVersion)
                    .padding([10, 20])
                    .style(styles::button_primary),
            ]
            .spacing(10)
            .into()
        } else {
            // Version list with radio buttons
            let versions_list = self.filtered_versions.iter().enumerate().fold(
                column![].spacing(4),
                |col, (index, version)| {
                    let label = format!("{} ({})", version.id, version.version_type);
                    
                    let item = radio(
                        label,
                        index,
                        self.selected_index,
                        Message::VersionSelected,
                    )
                    .style(styles::radio_style);
                    
                    col.push(item)
                },
            );

            let versions_scrollable = scrollable(versions_list)
                .height(Length::Fill);

            // Selected version info
            let selected_text = if let Some(index) = self.selected_index {
                if let Some(version) = self.filtered_versions.get(index) {
                    text(format!("Selected: {}", version.id)).size(14)
                } else {
                    text("").size(14)
                }
            } else {
                text("Select a version").size(14)
            };

            column![
                versions_scrollable,
                container(selected_text)
                    .padding([8, 0])
                    .align_x(Alignment::Center),
            ]
            .spacing(10)
            .into()
        };

        // Action buttons
        let confirm_button = button("Add Version")
            .on_press_maybe(if self.selected_index.is_some() {
                Some(Message::ConfirmAdd)
            } else {
                None
            })
            .padding([12, 24])
            .style(styles::button_success);

        let cancel_button = button("Cancel")
            .on_press(Message::Cancel)
            .padding([12, 24])
            .style(styles::button_secondary);

        let buttons = row![cancel_button, confirm_button]
            .spacing(10)
            .align_y(Alignment::Center);

        // Build the main dialog layout
        let main_content = column![
            title,
            filter_row,
            content,
            buttons,
        ]
        .spacing(15)
        .padding(25);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::card_container)
            .into()
    }
}