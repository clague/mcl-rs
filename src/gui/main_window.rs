// Main Window Module
// Handles the primary application window and coordinates between different views

use iced::widget::{button, column, container, row, text, scrollable, progress_bar, pick_list};
use iced::{Element, Length, Alignment, Subscription, Task};
use log::{info, warn, error, debug};

use crate::core::auth::{self, AccountSession};
use crate::core::version::VersionInfo;
use crate::core::update::{self, UpdateStatus};
use crate::core::launch::{self, LaunchConfig, LaunchResult};
use crate::config::config::Config;
use crate::gui::login::{Login, Message as LoginMessage};
use crate::gui::settings::{Settings, Message as SettingsMessage};
use crate::gui::add_version::{AddVersion, Message as AddVersionMessage};
use crate::gui::styles;
use crate::i18n::{self, Language, strings};

/// Messages that can be dispatched to the main window
#[derive(Debug, Clone)]
pub enum Message {
    /// Login-related messages from the login module
    Login(LoginMessage),
    /// Settings-related messages
    Settings(SettingsMessage),
    /// Add version dialog messages
    AddVersion(AddVersionMessage),
    /// User selected a version from the list
    VersionSelected(usize),
    /// User clicked the launch button
    LaunchVersion,
    /// Download progress update (0.0 to 1.0)
    DownloadProgress(f32),
    /// Periodic tick for animations or polling
    Tick,
    /// Update check result
    UpdateCheckResult(UpdateStatus),
    /// Launch check result
    LaunchCheckResult(String, LaunchResult),
    /// Download version files result
    DownloadVersionResult(String, Result<(), String>),
    /// Token refresh result
    TokenRefreshResult(Result<AccountSession, String>),
    /// Language changed
    LanguageChanged(Language),
}

/// Main application state
pub struct MainWindow {
    /// Current login component state
    login: Login,
    /// Authenticated user session (if logged in)
    session: Option<AccountSession>,
    /// Temporary user info for display during token refresh (username, uuid)
    saved_user_info: Option<(String, String)>,
    /// List of Minecraft versions added by the user
    versions: Vec<VersionInfo>,
    /// Index of the currently selected version
    selected_version: Option<usize>,
    /// Current download progress (0.0 to 1.0)
    download_progress: f32,
    /// Add version dialog component
    add_version: AddVersion,
    /// Settings dialog component
    settings: Settings,
    /// Application configuration
    config: Config,
    /// Update check status
    update_status: Option<UpdateStatus>,
    /// Whether update check is in progress
    checking_updates: bool,
    /// Whether token refresh is in progress
    refreshing_token: bool,
    /// Current language
    language: Language,
}

impl MainWindow {
    /// Creates a new MainWindow with default state
    pub fn new() -> Self {
        let mut config = Config::load();
        info!("Configuration loaded: auto_update={}", config.auto_update);
        
        // Load saved versions from config
        let versions = config.added_versions.clone();
        if !versions.is_empty() {
            info!("Loaded {} saved versions", versions.len());
        }
        
        // Load language preference from config
        let language = Language::from_code(&config.language);
        i18n::set_language(language);
        info!("Language set to: {}", language.display_name());
        
        // Load saved user info for display during token refresh
        let saved_user_info = config.saved_session.as_ref().map(|s| {
            info!("Found saved session for user: {}", s.username);
            (s.username.clone(), s.uuid.clone())
        });
        
        let refreshing_token = saved_user_info.is_some();
        
        Self {
            login: Login::new(),
            session: None,
            saved_user_info,
            versions,
            selected_version: None,
            download_progress: 0.0,
            add_version: AddVersion::new(),
            settings: Settings::new(),
            config,
            update_status: None,
            checking_updates: false,
            refreshing_token,
            language,
        }
    }

    /// Returns tasks for startup operations (token refresh and update check)
    pub fn startup_tasks(&self) -> Task<Message> {
        let mut tasks = Vec::new();
        
        // Check if we have a saved session to refresh
        if let Some(ref saved_session) = self.config.saved_session {
            info!("Found saved session for user: {}, refreshing token...", saved_session.username);
            let refresh_token = saved_session.refresh_token.clone();
            tasks.push(Task::perform(
                async move { auth::refresh_session(&refresh_token).await },
                Message::TokenRefreshResult,
            ));
        }
        
        // Check for updates if enabled
        if self.config.auto_update {
            info!("Auto-update enabled, starting update check...");
            let config = self.config.clone();
            tasks.push(Task::perform(
                async move { update::check_for_updates(&config).await },
                Message::UpdateCheckResult,
            ));
        }
        
        Task::batch(tasks)
    }

    /// Handles incoming messages and updates state accordingly.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Handle language change
            Message::LanguageChanged(lang) => {
                self.language = lang;
                i18n::set_language(lang);
                // Save language preference
                self.config.save_language(lang.code());
                info!("Language changed to: {}", lang.display_name());
                Task::none()
            }
            // Handle login messages
            Message::Login(login_message) => {
                if let LoginMessage::AuthResult(Ok(ref session)) = login_message {
                    self.session = Some(session.clone());
                    self.config.save_session(
                        session.minecraft_profile.name.clone(),
                        session.minecraft_profile.id.clone(),
                        session.access_token.clone(),
                        session.refresh_token.clone(),
                    );
                    info!("Session saved for user: {}", session.minecraft_profile.name);
                }
                self.login.update(login_message).map(Message::Login)
            }
            // Handle settings messages
            Message::Settings(settings_message) => {
                if let SettingsMessage::SaveSettings = &settings_message {
                    if let Some(java_path) = self.settings.get_java_path() {
                        self.config.java_path = Some(java_path);
                    }
                    if let Some(memory) = self.settings.get_memory() {
                        self.config.memory = memory;
                    }
                    self.config.auto_update = self.settings.get_auto_update();
                    
                    if let Err(e) = self.config.save() {
                        error!("Failed to save config: {}", e);
                    } else {
                        info!("Configuration saved successfully");
                    }
                }
                self.settings.update(settings_message).map(Message::Settings)
            }
            // Handle add version dialog messages
            Message::AddVersion(add_version_message) => {
                if let AddVersionMessage::ConfirmAdd = &add_version_message {
                    if let Some(version_info) = self.add_version.get_selected_version() {
                        self.versions.push(version_info.clone());
                        self.selected_version = Some(self.versions.len() - 1);
                        self.config.add_version(version_info.clone());
                        info!("Version {} added and saved", version_info.id);
                    }
                }
                self.add_version.update(add_version_message)
                    .map(Message::AddVersion)
            }
            Message::VersionSelected(index) => {
                self.selected_version = Some(index);
                Task::none()
            }
            Message::LaunchVersion => {
                if let Some(index) = self.selected_version {
                    if let Some(version) = self.versions.get(index) {
                        if let Some(_session) = &self.session {
                            info!("Preparing to launch version: {}", version.id);
                            let launch_config = LaunchConfig::from_config(&self.config, &version.id);
                            let version_info = version.clone();
                            return Task::perform(
                                async move {
                                    let check_result = launch::check_version_ready(&launch_config).await;
                                    Message::LaunchCheckResult(version_info.id.clone(), check_result)
                                },
                                |msg| msg,
                            );
                        } else {
                            warn!("Cannot launch: no user session");
                        }
                    }
                }
                Task::none()
            }
            Message::LaunchCheckResult(version_id, result) => {
                match result {
                    LaunchResult::Success => {
                        info!("Version {} is ready, launching...", version_id);
                        if let Some(session) = &self.session {
                            let launch_config = LaunchConfig::from_config(&self.config, &version_id);
                            match launch::launch_game(&launch_config, session) {
                                Ok(()) => info!("Game launched successfully"),
                                Err(e) => error!("Failed to launch game: {}", e),
                            }
                        }
                    }
                    LaunchResult::NeedsDownload(files) => {
                        info!("Version {} needs download: {:?}", version_id, files);
                        if let Some(version_info) = self.versions.iter().find(|v| v.id == version_id) {
                            let version_info = version_info.clone();
                            let config = self.config.clone();
                            return Task::perform(
                                async move {
                                    let result = launch::download_version_files(&version_info, &config, |_| {}).await;
                                    Message::DownloadVersionResult(version_id, result)
                                },
                                |msg| msg,
                            );
                        }
                    }
                    LaunchResult::Error(e) => error!("Launch check failed: {}", e),
                }
                Task::none()
            }
            Message::DownloadVersionResult(version_id, result) => {
                match result {
                    Ok(()) => {
                        info!("Version {} files downloaded, launching...", version_id);
                        if let Some(session) = &self.session {
                            let launch_config = LaunchConfig::from_config(&self.config, &version_id);
                            match launch::launch_game(&launch_config, session) {
                                Ok(()) => info!("Game launched successfully"),
                                Err(e) => error!("Failed to launch game after download: {}", e),
                            }
                        }
                    }
                    Err(e) => error!("Failed to download version files: {}", e),
                }
                Task::none()
            }
            Message::DownloadProgress(progress) => {
                self.download_progress = progress;
                Task::none()
            }
            Message::Tick => Task::none(),
            Message::UpdateCheckResult(status) => {
                self.checking_updates = false;
                match &status {
                    UpdateStatus::UpToDate => info!("All versions are up to date"),
                    UpdateStatus::UpdatesAvailable(versions) => {
                        info!("Updates available for {} version(s)", versions.len());
                    }
                    UpdateStatus::Error(e) => warn!("Update check failed: {}", e),
                    UpdateStatus::Skipped => debug!("Update check was skipped"),
                }
                self.update_status = Some(status);
                Task::none()
            }
            Message::TokenRefreshResult(result) => {
                self.refreshing_token = false;
                self.saved_user_info = None;
                match result {
                    Ok(session) => {
                        info!("Token refreshed successfully for user: {}", session.minecraft_profile.name);
                        self.config.save_session(
                            session.minecraft_profile.name.clone(),
                            session.minecraft_profile.id.clone(),
                            session.access_token.clone(),
                            session.refresh_token.clone(),
                        );
                        self.session = Some(session);
                    }
                    Err(e) => {
                        warn!("Failed to refresh token: {}, user needs to login again", e);
                        self.config.clear_session();
                    }
                }
                Task::none()
            }
        }
    }

    /// Renders the main view
    pub fn view(&self) -> Element<'_, Message> {
        if self.login.is_visible() {
            return self.login.view().map(Message::Login);
        }
        if self.add_version.is_visible() {
            return self.add_version.view().map(Message::AddVersion);
        }
        if self.settings.is_visible() {
            return self.settings.view().map(Message::Settings);
        }

        let versions_panel = self.view_versions_panel();
        let account_panel = self.view_account_panel();
        let top_bar = self.view_top_bar();
        let status_bar = self.view_status_bar();

        let content = column![
            top_bar,
            row![versions_panel, account_panel].spacing(20).padding(20),
            status_bar,
        ]
        .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        let s = strings();
        let status_text = if self.refreshing_token {
            text(s.refreshing_token).size(12)
        } else {
            match &self.update_status {
                Some(UpdateStatus::UpToDate) => text(s.all_versions_up_to_date).size(12),
                Some(UpdateStatus::UpdatesAvailable(v)) => text(s.updates_available.replace("{}", &v.len().to_string())).size(12),
                Some(UpdateStatus::Error(e)) => text(s.update_check_failed.replace("{}", e)).size(12),
                Some(UpdateStatus::Skipped) => text(s.auto_update_disabled).size(12),
                None => if self.checking_updates { text(s.checking_updates).size(12) } else { text("").size(12) },
            }
        };

        // Language selector in bottom right
        let lang_selector = row![
            text("Language:").size(12),
            pick_list(Language::all(), Some(self.language), Message::LanguageChanged)
                .padding([4, 8]),
        ]
        .spacing(5)
        .align_y(Alignment::Center);

        row![
            status_text,
            container(lang_selector).align_x(Alignment::End),
        ]
        .spacing(10)
        .padding([8, 15])
        .align_y(Alignment::Center)
        .into()
    }

    fn view_versions_panel(&self) -> Element<'_, Message> {
        let s = strings();
        
        let header = row![
            text(s.versions).size(20).width(Length::Fill),
            button(s.add)
                .on_press(Message::AddVersion(AddVersionMessage::ShowAddVersion))
                .padding([8, 16])
                .style(styles::button_primary),
        ]
        .align_y(Alignment::Center);

        let versions_list = if self.versions.is_empty() {
            container(
                column![
                    text(s.no_versions).size(16),
                    text(s.no_versions_hint).size(14),
                ]
                .spacing(10)
                .align_x(Alignment::Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(30)
        } else {
            let list = self.versions.iter().enumerate().fold(
                column![].spacing(6),
                |col, (index, version)| {
                    let is_selected = self.selected_version == Some(index);
                    let version_type = if version.version_type == "release" { s.version_type_release } else { s.version_type_snapshot };
                    let label = if is_selected {
                        text(format!(" {} [{}]", version.id, version_type)).size(15)
                    } else {
                        text(format!(" {} [{}]", version.id, version_type)).size(15)
                    };
                    
                    let btn = button(label)
                        .on_press(Message::VersionSelected(index))
                        .width(Length::Fill)
                        .padding([10, 15])
                        .style(if is_selected { styles::button_primary } else { styles::button_secondary });
                    
                    col.push(btn)
                },
            );
            container(list)
        };

        let versions_scrollable = scrollable(versions_list).height(Length::Fill);

        let launch_button = button(text(s.launch).size(18))
            .on_press_maybe(if self.selected_version.is_some() && self.session.is_some() {
                Some(Message::LaunchVersion)
            } else {
                None
            })
            .width(Length::Fill)
            .padding(14)
            .style(styles::button_success);

        let launch_hint = if self.session.is_none() {
            text(s.please_login_first).size(12)
        } else if self.selected_version.is_none() {
            text(s.select_version_to_launch).size(12)
        } else {
            text("").size(12)
        };

        let content = column![header, versions_scrollable, launch_hint, launch_button]
            .spacing(12)
            .padding(15);

        container(content)
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .style(styles::card_container)
            .into()
    }

    fn view_account_panel(&self) -> Element<'_, Message> {
        let s = strings();
        
        let account_info = if let Some(session) = &self.session {
            column![
                text(&session.minecraft_profile.name).size(18),
                text(format!("ID: {}", &session.minecraft_profile.id[..8])).size(12),
                button(s.logout)
                    .on_press(Message::Login(LoginMessage::CancelLogin))
                    .padding([8, 16])
                    .style(styles::button_outline),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
        } else if self.refreshing_token {
            if let Some((username, uuid)) = &self.saved_user_info {
                column![
                    text(username).size(18),
                    text(format!("ID: {}", &uuid[..8])).size(12),
                    text(s.refreshing_token).size(12),
                ]
                .spacing(10)
                .align_x(Alignment::Center)
            } else {
                column![text(s.refreshing_token).size(16)]
                    .spacing(10)
                    .align_x(Alignment::Center)
            }
        } else {
            column![
                text(s.not_logged_in).size(16),
                button(s.login_with_microsoft)
                    .on_press(Message::Login(LoginMessage::ShowLogin))
                    .padding([10, 20])
                    .style(styles::button_primary),
            ]
            .spacing(15)
            .align_x(Alignment::Center)
        };

        let content = column![
            text(s.account).size(20),
            account_info,
        ]
        .spacing(15)
        .padding(15)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(styles::card_container)
            .into()
    }

    fn view_top_bar(&self) -> Element<'_, Message> {
        let s = strings();
        let title = text(s.app_title).size(22);

        let download_progress = container(
            progress_bar(0.0..=1.0, self.download_progress)
                .style(styles::progress_bar_style)
        )
        .width(Length::Fill)
        .padding([4, 0]);

        let settings_button = button(s.settings)
            .on_press(Message::Settings(SettingsMessage::ShowSettings))
            .padding([8, 16])
            .style(styles::button_secondary);

        row![title, download_progress, settings_button]
            .spacing(15)
            .padding([10, 15])
            .align_y(Alignment::Center)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick)
    }
}