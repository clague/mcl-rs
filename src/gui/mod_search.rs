// Mod Search Dialog Module
// Handles searching and browsing mods on Modrinth for installation

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Alignment, Task};

use crate::core::{ModSearchHit, ModSearchResult};
use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    /// Show the mod search dialog
    Show,
    /// Hide the mod search dialog
    Hide,
    /// User typed in the search box
    SearchQueryChanged(String),
    /// User pressed the search button or hit Enter
    PerformSearch,
    /// Search results received from the API
    SearchResults(Result<ModSearchResult, String>),
    /// User selected a result by index
    SelectResult(usize),
    /// User clicked the install button for the selected mod
    InstallSelected,
    /// An error occurred
    Error(String),
}

pub struct ModSearchDialog {
    /// Current search query text
    search_query: String,
    /// Search results from the API
    results: Vec<ModSearchHit>,
    /// Whether a search request is in flight
    is_loading: bool,
    /// Whether the dialog is visible
    is_visible: bool,
    /// Index of the currently selected result
    selected_index: Option<usize>,
    /// Error message to display (if any)
    error: Option<String>,
    /// Total hits reported by the API
    total_hits: usize,
}

impl ModSearchDialog {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            results: Vec::new(),
            is_loading: false,
            is_visible: false,
            selected_index: None,
            error: None,
            total_hits: 0,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn show(&mut self) {
        self.is_visible = true;
        self.search_query = String::new();
        self.results.clear();
        self.selected_index = None;
        self.error = None;
        self.is_loading = false;
        self.total_hits = 0;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    /// Returns the Modrinth project_id of the currently selected result, if any.
    pub fn get_selected_project_id(&self) -> Option<String> {
        self.selected_index
            .and_then(|i| self.results.get(i))
            .map(|hit| hit.project_id.clone())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Show => {
                self.show();
                Task::none()
            }
            Message::Hide => {
                self.hide();
                Task::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                Task::none()
            }
            Message::PerformSearch => {
                let query = self.search_query.trim().to_string();
                if query.is_empty() {
                    return Task::none();
                }
                self.is_loading = true;
                self.error = None;
                self.selected_index = None;

                Task::perform(
                    async move {
                        let client = crate::core::modrinth::ModrinthClient::new();
                        client.search(&query, None, None, 0, 20).await
                    },
                    Message::SearchResults,
                )
            }
            Message::SearchResults(result) => {
                self.is_loading = false;
                match result {
                    Ok(search_result) => {
                        self.total_hits = search_result.total_hits;
                        self.results = search_result.hits;
                        self.selected_index = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                        self.results.clear();
                        self.total_hits = 0;
                    }
                }
                Task::none()
            }
            Message::SelectResult(index) => {
                if index < self.results.len() {
                    if self.selected_index == Some(index) {
                        self.selected_index = None;
                    } else {
                        self.selected_index = Some(index);
                    }
                }
                Task::none()
            }
            Message::InstallSelected => {
                // Selection is read by the parent via get_selected_project_id().
                // Hide dialog after install is triggered.
                self.is_visible = false;
                Task::none()
            }
            Message::Error(e) => {
                self.is_loading = false;
                self.error = Some(e);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if !self.is_visible {
            return container(text("")).width(Length::Fill).height(Length::Fill).into();
        }

        let title = text("Search Mods").size(26);

        // Search input row
        let search_input = text_input("Search for mods...", &self.search_query)
            .on_input(Message::SearchQueryChanged)
            .on_submit(Message::PerformSearch)
            .padding(10)
            .style(styles::text_input_style)
            .width(Length::Fill);

        let search_button = button("Search")
            .on_press_maybe(
                if !self.search_query.trim().is_empty() && !self.is_loading {
                    Some(Message::PerformSearch)
                } else {
                    None
                },
            )
            .padding([10, 20])
            .style(styles::button_primary);

        let search_row = row![search_input, search_button]
            .spacing(10)
            .align_y(Alignment::Center);

        // Results area
        let results_area: Element<Message> = if self.is_loading {
            container(
                column![
                    text("Searching...").size(18),
                    container(
                        iced::widget::progress_bar(0.0..=1.0, 0.5)
                            .style(styles::progress_bar_style)
                    )
                    .padding([4, 0])
                    .width(Length::Fill),
                ]
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else if let Some(ref error) = self.error {
            column![
                text(error.as_str()).size(16),
                button("Try Again")
                    .on_press(Message::PerformSearch)
                    .padding([8, 16])
                    .style(styles::button_outline),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
            .into()
        } else if self.results.is_empty() && !self.search_query.trim().is_empty() {
            container(
                column![
                    text("No results found").size(18),
                    text("Try a different search query").size(14),
                ]
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else if self.results.is_empty() {
            container(
                column![
                    text("Search for mods on Modrinth").size(18),
                    text("Type a query and press Search or Enter").size(14),
                ]
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            let hits_text = text(format!(
                "{} result(s) found",
                self.total_hits
            ))
            .size(13);

            let results_list = self.results.iter().enumerate().fold(
                column![].spacing(8),
                |col, (index, hit)| {
                    let is_selected = self.selected_index == Some(index);

                    let title_text = text(&hit.title).size(16);
                    let desc_text = text(&hit.description).size(13);
                    let downloads_text =
                        text(format!("{} downloads", format_number(hit.downloads))).size(12);

                    let info_col = column![title_text, desc_text, downloads_text].spacing(4);

                    let mod_row = row![info_col]
                        .spacing(10)
                        .align_y(Alignment::Center);

                    let mod_card = container(mod_row)
                        .padding(12)
                        .width(Length::Fill)
                        .style(styles::panel_container);

                    let clickable = button(mod_card)
                        .on_press(Message::SelectResult(index))
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

            column![
                hits_text,
                scrollable(results_list).height(Length::Fill),
            ]
            .spacing(8)
            .into()
        };

        // Install button (only when a result is selected)
        let install_button: Element<Message> = if self.selected_index.is_some() {
            button("Install")
                .on_press(Message::InstallSelected)
                .padding([10, 24])
                .style(styles::button_success)
                .into()
        } else {
            text("").into()
        };

        // Close button
        let close_button = button("Close")
            .on_press(Message::Hide)
            .padding([10, 20])
            .style(styles::button_secondary);

        let bottom_bar = row![install_button, close_button]
            .spacing(10)
            .align_y(Alignment::Center);

        let main_content = column![title, search_row, results_area, bottom_bar]
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

/// Formats a number with comma separators for display (e.g. 1234567 -> "1,234,567").
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
