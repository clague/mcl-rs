// Main Window Module
// Handles the primary application window and coordinates between different views

use iced::widget::{button, column, container, row, text, text_input, scrollable, progress_bar, Stack, image, canvas};
use iced::{Element, Length, Alignment, Subscription, Task, Color, Point, Rectangle, Size, Renderer};
use iced_aw::widget::drop_down::DropDown;
use log::{info, warn, error, debug};
use std::collections::HashMap;

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

struct Spinner {
    angle: f32,
}

impl canvas::Program<Message> for Spinner {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, Size::new(bounds.width, bounds.height));
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = bounds.width.min(bounds.height) / 2.0 - 2.0;

        let bg_circle = canvas::Path::circle(center, radius);
        frame.fill(&bg_circle, Color::from_rgba(0.4, 0.4, 0.4, 0.3));

        let start_angle = iced::Radians(self.angle);
        let end_angle = iced::Radians(self.angle + std::f32::consts::PI * 1.5);
        let arc = canvas::Path::new(|builder| {
            builder.arc(canvas::path::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            });
        });
        frame.stroke(
            &arc,
            canvas::Stroke::default()
                .with_color(Color::WHITE)
                .with_width(2.0),
        );

        vec![frame.into_geometry()]
    }
}

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
    VersionSelected(String),
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
    /// Toggle account dropdown menu
    ToggleAccountMenu,
    /// Manually refresh account session
    RefreshSession,
    /// Avatar image fetched
    AvatarFetched(Result<Vec<u8>, String>),
    /// Toggle version settings panel
    ToggleVersionSettings,
    /// Display name input changed
    DisplayNameChanged(String),
    /// Save display name for selected version
    SaveDisplayName,
    /// Open version folder in file manager
    OpenVersionFolder,
    /// Delete selected version
    DeleteVersion,
    /// Show delete confirmation dialog
    ShowDeleteConfirm,
    /// Confirm version deletion
    ConfirmDelete,
    /// Cancel version deletion
    CancelDelete,
    /// Version icon fetched (icon_name, bytes)
    IconFetched(String, Result<Vec<u8>, String>),
    /// Mod panel messages
    ModPanel(crate::gui::mod_panel::Message),
    /// Mod search dialog messages
    ModSearch(crate::gui::mod_search::Message),
    /// Show icon picker dialog
    ShowIconPicker,
    /// Hide icon picker dialog
    HideIconPicker,
    /// User selected an icon from the picker
    SelectIcon(String),
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
    /// Unique identifier for selected version (UUID)
    selected_version: Option<String>,
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
    /// Whether token refresh has failed
    token_refresh_failed: bool,
    /// Whether account dropdown menu is shown
    show_account_menu: bool,
    /// Current language
    language: Language,
    /// Cached avatar image handle
    avatar: Option<image::Handle>,
    /// Cached version icon handles (icon_name -> Handle)
    version_icons: HashMap<String, image::Handle>,
    /// Animation tick counter for spinner
    animation_tick: u32,
    /// Whether version settings panel is shown
    show_version_settings: bool,
    /// Display name being edited
    editing_display_name: String,
    /// Whether delete confirmation dialog is shown
    show_delete_confirm: bool,
    /// Whether icon picker dialog is shown
    show_icon_picker: bool,
    /// Mod management panel
    mod_panel: crate::gui::mod_panel::ModPanel,
    /// Mod search dialog
    mod_search: crate::gui::mod_search::ModSearchDialog,
}

impl MainWindow {
    /// Creates a new MainWindow with default state
    pub fn new() -> Self {
        let config = Config::load();
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

        let avatar = Self::load_cached_avatar();

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
            token_refresh_failed: false,
            show_account_menu: false,
            avatar,
            version_icons: HashMap::new(),
            animation_tick: 0,
            show_version_settings: false,
            editing_display_name: String::new(),
            show_delete_confirm: false,
            show_icon_picker: false,
            mod_panel: crate::gui::mod_panel::ModPanel::new(),
            mod_search: crate::gui::mod_search::ModSearchDialog::new(),
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

        // Preload all Minecraft icons
        for icon_name in crate::core::version::MINECRAFT_ICONS {
            if !self.version_icons.contains_key(*icon_name) {
                let icon_name = icon_name.to_string();
                tasks.push(Task::perform(
                    async move {
                        let result = Self::fetch_icon_bytes(&icon_name).await;
                        Message::IconFetched(icon_name, result)
                    },
                    |msg| msg,
                ));
            }
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
            Message::ToggleAccountMenu => {
                self.show_account_menu = !self.show_account_menu;
                Task::none()
            }
            Message::RefreshSession => {
                if let Some(ref saved_session) = self.config.saved_session {
                    info!("Manually refreshing session for user: {}", saved_session.username);
                    self.refreshing_token = true;
                    self.token_refresh_failed = false;
                    let refresh_token = saved_session.refresh_token.clone();
                    Task::perform(
                        async move { auth::refresh_session(&refresh_token).await },
                        Message::TokenRefreshResult,
                    )
                } else {
                    Task::none()
                }
            }
            // Handle login messages
            Message::Login(login_message) => {
                match &login_message {
                    LoginMessage::CancelLogin => {
                        self.session = None;
                        self.config.clear_session();
                        self.show_account_menu = false;
                    }
                    LoginMessage::ShowLogin => {
                        self.show_account_menu = false;
                    }
                    LoginMessage::AuthResult(Ok(session)) => {
                        let uuid = session.minecraft_profile.id.clone();
                        self.session = Some(session.clone());
                        self.show_account_menu = false;
                        self.avatar = None;
                        self.config.save_session(
                            session.minecraft_profile.name.clone(),
                            session.minecraft_profile.id.clone(),
                            session.access_token.clone(),
                            session.refresh_token.clone(),
                        );
                        info!("Session saved for user: {}", session.minecraft_profile.name);
                        return Task::perform(
                            async move { Self::fetch_avatar_bytes(&uuid).await },
                            Message::AvatarFetched,
                        );
                    }
                    _ => {}
                }
                self.login.update(login_message).map(Message::Login)
            }
            // Handle settings messages
            Message::Settings(settings_message) => {
                if let SettingsMessage::ShowSettings = &settings_message {
                    self.settings.load_from_config(&self.config);
                }
                if let SettingsMessage::LanguageChanged(lang) = &settings_message {
                    self.language = *lang;
                    i18n::set_language(*lang);
                }
                if let SettingsMessage::SaveSettings = &settings_message {
                    if let Some(java_path) = self.settings.get_java_path() {
                        self.config.java_path = Some(java_path);
                    } else {
                        self.config.java_path = None;
                    }
                    self.config.memory = self.settings.get_memory();
                    self.config.auto_update = self.settings.get_auto_update();
                    self.config.max_connections = self.settings.get_max_connections();
                    self.config.save_language(self.settings.get_language().code());

                    if let Err(e) = self.config.save() {
                        error!("Failed to save config: {}", e);
                    } else {
                        info!("Configuration saved successfully");
                    }
                }
                if let SettingsMessage::CancelSettings = &settings_message {
                    let original_lang = Language::from_code(&self.config.language);
                    self.language = original_lang;
                    i18n::set_language(original_lang);
                }
                self.settings.update(settings_message).map(Message::Settings)
            }
            // Handle add version dialog messages
            Message::AddVersion(add_version_message) => {
                if let AddVersionMessage::ConfirmAdd = &add_version_message {
                    if let Some(version_info_ref) = self.add_version.get_selected_version() {
                        let mut version_info = version_info_ref.clone();
                        version_info.icon_name = crate::core::version::random_icon();
                        let icon_name = version_info.icon_name.clone();
                        let needs_fetch = !self.version_icons.contains_key(&icon_name);
                        self.versions.push(version_info.clone());
                        self.selected_version = Some(version_info.uuid.clone());
                        self.config.add_version(version_info.clone());
                        info!("Version {} added and saved with icon: {}", version_info.version, icon_name);
                        if needs_fetch {
                            return Task::perform(
                                async move {
                                    let result = Self::fetch_icon_bytes(&icon_name).await;
                                    Message::IconFetched(icon_name, result)
                                },
                                |msg| msg,
                            );
                        }
                    }
                }
                self.add_version.update(add_version_message)
                    .map(Message::AddVersion)
            }
            Message::VersionSelected(uuid) => {
                if self.selected_version.as_ref() == Some(&uuid) {
                    self.selected_version = None;
                    self.show_version_settings = false;
                } else {
                    self.selected_version = Some(uuid.clone());
                    self.show_version_settings = true;
                    if let Some(version) = self.versions.iter().find(|v| v.uuid == uuid) {
                        self.editing_display_name = if version.display_name.is_empty() {
                            version.version.clone()
                        } else {
                            version.display_name.clone()
                        };
                    }
                }
                Task::none()
            }
            Message::LaunchVersion => {
                if let Some(ref uuid) = self.selected_version {
                    if let Some(version) = self.versions.iter().find(|v| &v.uuid == uuid) {
                        if let Some(_session) = &self.session {
                            info!("Preparing to launch version: {}", version.version);
                            let launch_config = LaunchConfig::from_config(&self.config, &version.version);
                            let version_info = version.clone();
                            return Task::perform(
                                async move {
                                    let check_result = launch::check_version_ready(&launch_config, None).await;
                                    Message::LaunchCheckResult(version_info.version.clone(), check_result)
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
                            match launch::launch_game(&launch_config, session, None) {
                                Ok(()) => info!("Game launched successfully"),
                                Err(e) => error!("Failed to launch game: {}", e),
                            }
                        }
                    }
                    LaunchResult::NeedsDownload(files) => {
                        info!("Version {} needs download: {:?}", version_id, files);
                        if let Some(version_info) = self.versions.iter().find(|v| v.version == version_id) {
                            let version_info = version_info.clone();
                            let config = self.config.clone();
                            return Task::perform(
                                async move {
                                    let result = launch::download_version_files(&version_info, &config, None, |_| {}).await;
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
                            match launch::launch_game(&launch_config, session, None) {
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
            Message::Tick => {
                self.animation_tick = self.animation_tick.wrapping_add(1);
                Task::none()
            }
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
                        let uuid = session.minecraft_profile.id.clone();
                        self.config.save_session(
                            session.minecraft_profile.name.clone(),
                            session.minecraft_profile.id.clone(),
                            session.access_token.clone(),
                            session.refresh_token.clone(),
                        );
                        self.session = Some(session);
                        self.avatar = None;
                        return Task::perform(
                            async move { Self::fetch_avatar_bytes(&uuid).await },
                            Message::AvatarFetched,
                        );
                    }
                    Err(e) => {
                        warn!("Failed to refresh token: {}", e);
                        self.token_refresh_failed = true;
                    }
                }
                Task::none()
            }
            Message::AvatarFetched(result) => {
                match result {
                    Ok(bytes) => {
                        info!("Avatar fetched successfully ({} bytes)", bytes.len());
                        Self::save_cached_avatar(&bytes);
                        self.avatar = Some(image::Handle::from_bytes(bytes));
                    }
                    Err(e) => warn!("Failed to fetch avatar: {}", e),
                }
                Task::none()
            }
            Message::IconFetched(icon_name, result) => {
                match result {
                    Ok(bytes) => {
                        info!("Icon '{}' fetched ({} bytes)", icon_name, bytes.len());
                        self.version_icons.insert(icon_name.clone(), image::Handle::from_bytes(bytes));
                    }
                    Err(e) => warn!("Failed to fetch icon '{}': {}", icon_name, e),
                }
                Task::none()
            }
            Message::ToggleVersionSettings => {
                if self.show_version_settings {
                    self.show_version_settings = false;
                } else if let Some(ref uuid) = self.selected_version {
                    if let Some(version) = self.versions.iter().find(|v| &v.uuid == uuid) {
                        let name = if version.display_name.is_empty() {
                            version.version.clone()
                        } else {
                            version.display_name.clone()
                        };
                        self.editing_display_name = name;
                        self.show_version_settings = true;
                    }
                }
                Task::none()
            }
            Message::DisplayNameChanged(name) => {
                self.editing_display_name = name;
                Task::none()
            }
            Message::SaveDisplayName => {
                if let Some(ref uuid) = self.selected_version {
                    if let Some(version) = self.versions.iter_mut().find(|v| &v.uuid == uuid) {
                        version.display_name = self.editing_display_name.clone();
                    }
                    self.config.added_versions = self.versions.clone();
                    if let Err(e) = self.config.save() {
                        error!("Failed to save display name: {}", e);
                    } else if let Some(version) = self.versions.iter().find(|v| &v.uuid == uuid) {
                        info!("Display name saved for version {}", version.version);
                    }
                }
                Task::none()
            }
            Message::OpenVersionFolder => {
                if let Some(ref uuid) = self.selected_version {
                    if let Some(version) = self.versions.iter().find(|v| &v.uuid == uuid) {
                        let folder = self.config.versions_dir.join(&version.version);
                        if folder.exists() {
                            return Task::perform(
                                async move {
                                    let _ = open::that(&folder);
                                },
                                |_| Message::Tick,
                            );
                        } else {
                            warn!("Version folder does not exist: {:?}", folder);
                        }
                    }
                }
                Task::none()
            }
            Message::DeleteVersion => {
                self.show_delete_confirm = true;
                Task::none()
            }
            Message::ShowDeleteConfirm => {
                self.show_delete_confirm = true;
                Task::none()
            }
            Message::ConfirmDelete => {
                self.show_delete_confirm = false;
                if let Some(ref uuid) = self.selected_version.clone() {
                    if let Some(idx) = self.versions.iter().position(|v| &v.uuid == uuid) {
                        let version = &self.versions[idx];
                        let version_id = version.version.clone();
                        let version_dir = self.config.versions_dir.join(&version_id);

                        if version_dir.exists() {
                            match std::fs::remove_dir_all(&version_dir) {
                                Ok(()) => info!("Deleted version directory: {:?}", version_dir),
                                Err(e) => error!("Failed to delete version directory: {}", e),
                            }
                        }

                        self.versions.remove(idx);
                        self.config.remove_version(&version_id);
                        self.selected_version = None;
                        self.show_version_settings = false;
                        info!("Version {} deleted", version_id);
                    }
                }
                Task::none()
            }
            Message::CancelDelete => {
                self.show_delete_confirm = false;
                Task::none()
            }
            Message::ShowIconPicker => {
                self.show_icon_picker = true;
                Task::none()
            }
            Message::HideIconPicker => {
                self.show_icon_picker = false;
                Task::none()
            }
            Message::SelectIcon(icon_name) => {
                if let Some(ref uuid) = self.selected_version {
                    if let Some(version) = self.versions.iter_mut().find(|v| &v.uuid == uuid) {
                        version.icon_name = icon_name.clone();
                        let version_id = version.version.clone();
                        self.config.added_versions = self.versions.clone();
                        let _ = self.config.save();
                        info!("Icon changed to '{}' for version {}", icon_name, version_id);
                    }
                }
                self.show_icon_picker = false;
                Task::none()
            }
            Message::ModPanel(msg) => {
                if let crate::gui::mod_panel::Message::SearchMods = &msg {
                    self.mod_search.show();
                    return Task::none();
                }
                self.mod_panel.update(msg).map(Message::ModPanel)
            }
            Message::ModSearch(msg) => {
                self.mod_search.update(msg).map(Message::ModSearch)
            }
        }
    }

    async fn fetch_avatar_bytes(uuid: &str) -> Result<Vec<u8>, String> {
        let url = format!("https://api.mineatar.io/face/{}?scale=4", uuid);
        info!("Fetching avatar from: {}", url);

        let client = crate::utils::net::shared_client();
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Avatar request failed: {}", resp.status()));
        }

        resp.bytes().await
            .map(|b| b.to_vec())
            .map_err(|e| e.to_string())
    }

    fn avatar_cache_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("mcl-rs")
            .join("avatar.png")
    }

    fn load_cached_avatar() -> Option<image::Handle> {
        let path = Self::avatar_cache_path();
        if path.exists() {
            match std::fs::read(&path) {
                Ok(bytes) => {
                    info!("Loaded cached avatar from {:?}", path);
                    Some(image::Handle::from_bytes(bytes))
                }
                Err(e) => {
                    warn!("Failed to read cached avatar: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    fn save_cached_avatar(bytes: &[u8]) {
        let path = Self::avatar_cache_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&path, bytes) {
            Ok(()) => info!("Avatar cached to {:?}", path),
            Err(e) => warn!("Failed to cache avatar: {}", e),
        }
    }

    async fn fetch_icon_bytes(icon_name: &str) -> Result<Vec<u8>, String> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("mcl-rs")
            .join("icons");
        let cache_path = cache_dir.join(format!("{}.png", icon_name));

        if cache_path.exists() {
            if let Ok(bytes) = std::fs::read(&cache_path) {
                debug!("Icon '{}' loaded from cache ({} bytes)", icon_name, bytes.len());
                return Ok(bytes);
            }
        }

        let url = format!("https://mc-heads.net/head/{}", icon_name);
        info!("Fetching icon from: {}", url);

        let client = crate::utils::net::shared_client();
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Icon request failed: {}", resp.status()));
        }

        let bytes = resp.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string())?;

        let _ = std::fs::create_dir_all(&cache_dir);
        let _ = std::fs::write(&cache_path, &bytes);
        debug!("Icon '{}' cached to {:?}", icon_name, cache_path);

        Ok(bytes)
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
        if self.show_delete_confirm {
            return self.view_delete_confirm();
        }
        if self.show_icon_picker {
            return self.view_icon_picker();
        }
        if self.mod_search.is_visible() {
            return self.mod_search.view().map(Message::ModSearch);
        }
        if self.mod_panel.is_visible() {
            return self.mod_panel.view().map(Message::ModPanel);
        }

        let top_bar = self.view_top_bar();
        let status_bar = self.view_status_bar();

        let versions_area: Element<'_, Message> = if self.show_version_settings && self.selected_version.is_some() {
            container(
                row![
                    container(self.view_versions_panel())
                        .width(Length::FillPortion(3))
                        .height(Length::Fill),
                    self.view_version_settings(),
                ]
                .spacing(10)
                .height(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
        } else {
            container(self.view_versions_panel())
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20)
                .into()
        };

        let content = column![
            top_bar,
            versions_area,
            status_bar,
        ]
        .spacing(10);

        let content = container(content)
            .width(Length::Fill)
            .height(Length::Fill);

        content.into()
    }

    fn view_delete_confirm(&self) -> Element<'_, Message> {
        let s = strings();
        let content = column![
            text(s.delete_version).size(22),
            text(s.delete_confirm).size(16),
            row![
                button(s.cancel)
                    .on_press(Message::CancelDelete)
                    .padding([10, 20])
                    .style(styles::button_secondary),
                button(s.delete_version)
                    .on_press(Message::ConfirmDelete)
                    .padding([10, 20])
                    .style(styles::button_danger),
            ].spacing(10),
        ].spacing(15).padding(30).max_width(400);

        container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::card_container)
            .into()
    }

    fn view_icon_picker(&self) -> Element<'_, Message> {
        let s = strings();

        let current_icon = self.selected_version
            .as_ref()
            .and_then(|uuid| self.versions.iter().find(|v| &v.uuid == uuid))
            .map(|v| v.icon_name.as_str())
            .unwrap_or("");

        let mut icon_rows: Vec<Element<'_, Message>> = Vec::new();
        for chunk in crate::core::version::MINECRAFT_ICONS.chunks(5) {
            let mut row_items: Vec<Element<'_, Message>> = Vec::new();
            for icon_name in chunk {
                let icon_handle = self.version_icons.get(*icon_name).cloned();
                let is_selected = *icon_name == current_icon;

                let icon_btn = if let Some(handle) = icon_handle {
                    button(
                        image(handle)
                            .width(Length::Fixed(64.0))
                            .height(Length::Fixed(64.0))
                    )
                    .on_press(Message::SelectIcon(icon_name.to_string()))
                    .padding(4)
                    .style(if is_selected { styles::button_primary } else { styles::button_icon })
                } else {
                    button(text(*icon_name).size(12))
                        .on_press(Message::SelectIcon(icon_name.to_string()))
                        .padding([8, 12])
                        .style(if is_selected { styles::button_primary } else { styles::button_icon })
                };
                row_items.push(icon_btn.into());
            }
            icon_rows.push(row![iced::widget::Row::from_vec(row_items).spacing(8)].into());
        }

        let content = column![
            text(s.version_settings).size(22),
            text("Select an icon").size(16),
            iced::widget::Column::from_vec(icon_rows).spacing(8),
            button(s.cancel)
                .on_press(Message::HideIconPicker)
                .padding([10, 20])
                .width(Length::Fill)
                .style(styles::button_secondary),
        ].spacing(15).padding(25).width(Length::Shrink);

        container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::card_container)
            .into()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        let s = strings();
        let status_text: Element<'_, Message> = if self.refreshing_token {
            text(s.refreshing_token).size(16).into()
        } else if self.token_refresh_failed {
            row![
                text(s.session_refresh_failed_hint).size(16).color(Color::from_rgba(1.0, 0.6, 0.2, 1.0)),
                button(text(s.refresh_session).size(14))
                    .on_press(Message::RefreshSession)
                    .style(styles::button_primary)
                    .padding([2, 8]),
            ].spacing(8).align_y(Alignment::Center).into()
        } else {
            match &self.update_status {
                Some(UpdateStatus::UpToDate) => text(s.all_versions_up_to_date).size(16).into(),
                Some(UpdateStatus::UpdatesAvailable(v)) => text(s.updates_available.replace("{}", &v.len().to_string())).size(16).into(),
                Some(UpdateStatus::Error(e)) => text(s.update_check_failed.replace("{}", e)).size(16).into(),
                Some(UpdateStatus::Skipped) => text(s.auto_update_disabled).size(16).into(),
                None => if self.checking_updates { text(s.checking_updates).size(16).into() } else { text("").size(16).into() },
            }
        };

        row![
            status_text,
            container(iced::widget::row![]).align_x(Alignment::End),
        ]
        .spacing(10)
        .padding([8, 15])
        .align_y(Alignment::Center)
        .into()
    }

    fn view_account_dropdown(&self) -> Element<'_, Message> {
        let s = strings();

        let content = if self.token_refresh_failed {
            let username = self.saved_user_info.as_ref()
                .map(|(name, _)| name.as_str())
                .unwrap_or("Unknown");
            column![
                text(username).size(16),
                text(s.session_refresh_failed).size(14).color(Color::from_rgba(1.0, 0.4, 0.4, 1.0)),
                row![
                    button(s.refresh_session)
                        .on_press(Message::RefreshSession)
                        .style(styles::button_primary)
                        .padding([4, 12]),
                    button(s.logout)
                        .on_press(Message::Login(LoginMessage::CancelLogin))
                        .style(styles::button_outline)
                        .padding([4, 12]),
                ].spacing(8),
            ]
            .spacing(8)
            .align_x(Alignment::Center)
        } else if let Some(session) = &self.session {
            let mut items: Vec<Element<'_, Message>> = Vec::new();
            if let Some(avatar_handle) = &self.avatar {
                items.push(
                    image(avatar_handle.clone())
                        .width(Length::Fixed(48.0))
                        .height(Length::Fixed(48.0))
                        .into()
                );
            }
            items.push(text(&session.minecraft_profile.name).size(16).into());
            items.push(text(format!("ID: {}", &session.minecraft_profile.id[..8])).size(14).into());
            items.push(
                button(s.logout)
                    .on_press(Message::Login(LoginMessage::CancelLogin))
                    .style(styles::button_outline)
                    .padding([4, 12])
                    .into()
            );
            iced::widget::Column::from_vec(items)
                .spacing(8)
                .align_x(Alignment::Center)
        } else if let Some((username, uuid)) = &self.saved_user_info {
            column![
                text(username).size(16),
                text(format!("ID: {}", &uuid[..8])).size(14),
                text(s.refreshing_token).size(14),
            ]
            .spacing(8)
            .align_x(Alignment::Center)
        } else {
            column![
                text(s.not_logged_in).size(16),
                button(s.open_browser_to_login)
                    .on_press(Message::Login(LoginMessage::ShowLogin))
                    .style(styles::button_primary)
                    .padding([6, 14]),
            ]
            .spacing(8)
            .align_x(Alignment::Center)
        };

        container(content)
            .style(styles::card_container)
            .padding(12)
            .into()
    }

    fn view_version_settings(&self) -> Element<'_, Message> {
        let s = strings();

        let content = if let Some(ref uuid) = self.selected_version {
            if let Some(version) = self.versions.iter().find(|v| &v.uuid == uuid) {
                let display_name = if version.display_name.is_empty() {
                    &version.version
                } else {
                    &version.display_name
                };

                let mut items: Vec<Element<'_, Message>> = Vec::new();
                let title_row = if let Some(icon_handle) = self.version_icons.get(&version.icon_name) {
                    row![
                        text(s.version_settings).size(20).width(Length::Fill),
                        button(
                            image(icon_handle.clone())
                                .width(Length::Fixed(48.0))
                                .height(Length::Fixed(48.0))
                        )
                        .on_press(Message::ShowIconPicker)
                        .padding(4)
                        .style(styles::button_icon),
                    ]
                    .align_y(Alignment::Center)
                    .into()
                } else {
                    row![
                        text(s.version_settings).size(20).width(Length::Fill),
                        button(text("?").size(24))
                            .on_press(Message::ShowIconPicker)
                            .padding([4, 12])
                            .style(styles::button_icon),
                    ]
                    .align_y(Alignment::Center)
                    .into()
                };
                items.push(title_row);
                items.push(
                    row![
                        text(format!("{}: {}", s.version_number, version.version)).size(14),
                        text(format!("{}: {}", s.version_type, if version.version_type == "release" { s.version_type_release } else { s.version_type_snapshot })).size(14),
                    ]
                    .spacing(16)
                    .into()
                );
                items.push(text(s.display_name).size(16).into());
                items.push(
                    text_input(display_name, &self.editing_display_name)
                        .on_input(Message::DisplayNameChanged)
                        .on_submit(Message::SaveDisplayName)
                        .padding([8, 12])
                        .width(Length::Fill)
                        .into()
                );
                items.push(
                    button(container(text(s.save)).center_x(Length::Fill))
                        .on_press(Message::SaveDisplayName)
                        .padding([8, 16])
                        .width(Length::Fill)
                        .style(styles::button_primary)
                        .into()
                );
                items.push(
                    button(container(text(s.open_folder)).center_x(Length::Fill))
                        .on_press(Message::OpenVersionFolder)
                        .padding([8, 16])
                        .width(Length::Fill)
                        .style(styles::button_secondary)
                        .into()
                );
                items.push(
                    button(container(text(s.delete_version)).center_x(Length::Fill))
                        .on_press(Message::DeleteVersion)
                        .padding([8, 16])
                        .width(Length::Fill)
                        .style(styles::button_danger)
                        .into()
                );
                items.push(
                    button(container(text(s.mods)).center_x(Length::Fill))
                        .on_press(Message::ModPanel(crate::gui::mod_panel::Message::Show(
                            version.uuid.clone(),
                            Vec::new()
                        )))
                        .padding([8, 16])
                        .width(Length::Fill)
                        .style(styles::button_secondary)
                        .into()
                );

                iced::widget::Column::from_vec(items)
                    .spacing(12)
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Fill)
            } else {
                column![]
            }
        } else {
            column![]
        };

        container(content)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(styles::card_container)
            .into()
    }

    fn view_versions_panel(&self) -> Element<'_, Message> {
        let s = strings();

        let header = row![
            text(s.versions).size(22).width(Length::Fill),
            button(s.add)
                .on_press(Message::AddVersion(AddVersionMessage::ShowAddVersion))
                .padding([8, 16])
                .style(styles::button_primary),
        ]
        .align_y(Alignment::Center);

        let versions_list = if self.versions.is_empty() {
            container(
                column![
                    text(s.no_versions).size(18),
                    text(s.no_versions_hint).size(16),
                ]
                .spacing(10)
                .align_x(Alignment::Center)
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(30)
        } else {
            let mut wrap = iced_aw::Wrap::new();

            for version in &self.versions {
                let is_selected = self.selected_version.as_ref() == Some(&version.uuid);
                let version_type = if version.version_type == "release" { s.version_type_release } else { s.version_type_snapshot };
                let display_name = if version.display_name.is_empty() {
                    version.version.clone()
                } else {
                    version.display_name.clone()
                };

                let mut card_items: Vec<Element<'_, Message>> = Vec::new();
                if let Some(icon_handle) = self.version_icons.get(&version.icon_name) {
                    card_items.push(
                        image(icon_handle.clone())
                            .width(Length::Fixed(48.0))
                            .height(Length::Fixed(48.0))
                            .into()
                    );
                }
                card_items.push(
                    text(display_name).size(16).into()
                );
                card_items.push(
                    text(format!("{} · {}", version.version, version_type)).size(12).into()
                );

                let card_content = iced::widget::Column::from_vec(card_items)
                    .spacing(4)
                    .align_x(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Shrink);

                let card = button(card_content)
                    .on_press(Message::VersionSelected(version.uuid.clone()))
                    .padding([8, 8])
                    .width(Length::Fixed(120.0))
                    .height(Length::Shrink)
                    .style(if is_selected { styles::button_primary } else { styles::button_icon });

                wrap = wrap.push(card);
            }

            container(wrap.spacing(10).line_spacing(10))
                .padding(10)
        };

        let versions_scrollable = scrollable(versions_list).height(Length::Fill);

        let launch_hint = if self.refreshing_token {
            text(s.refreshing_session).size(18)
        } else if self.session.is_none() {
            text(s.please_login_first).size(18)
        } else if self.selected_version.is_none() {
            text(s.select_version_to_launch).size(18)
        } else {
            text("").size(18)
        };

        let launch_button = button(text(s.launch).size(20))
            .on_press_maybe(if self.selected_version.is_some() && self.session.is_some() {
                Some(Message::LaunchVersion)
            } else {
                None
            })
            .padding([10, 24])
            .style(styles::button_success);

        let panel_content = column![
            header,
            versions_scrollable,
            launch_hint,
        ]
        .spacing(12)
        .padding(15);

        let panel_base = container(panel_content)
            .width(Length::Fill)
            .height(Length::Fill);

        let fab_layer = container(launch_button)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::End)
            .align_y(Alignment::End)
            .padding(15);

        container(
            Stack::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .push(panel_base)
                .push(fab_layer)
        )
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .style(styles::card_container)
            .into()
    }

    fn view_account_panel(&self) -> Element<'_, Message> {
        let s = strings();

        let account_info = if let Some(session) = &self.session {
            column![
                text(&session.minecraft_profile.name).size(20),
                text(format!("ID: {}", &session.minecraft_profile.id[..8])).size(16),
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
                    text(username).size(20),
                    text(format!("ID: {}", &uuid[..8])).size(16),
                    text(s.refreshing_token).size(16),
                ]
                .spacing(10)
                .align_x(Alignment::Center)
            } else {
                column![text(s.refreshing_token).size(18)]
                    .spacing(10)
                    .align_x(Alignment::Center)
            }
        } else {
            column![
                text(s.not_logged_in).size(18),
                button(s.login_with_microsoft)
                    .on_press(Message::Login(LoginMessage::ShowLogin))
                    .padding([10, 20])
                    .style(styles::button_primary),
            ]
            .spacing(15)
            .align_x(Alignment::Center)
        };

        let content = column![
            text(s.account).size(22),
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
        let title = text(s.app_title).size(24);

        let download_progress = container(
            progress_bar(0.0..=1.0, self.download_progress)
                .style(styles::progress_bar_style)
        )
        .width(Length::Fill)
        .padding([4, 0]);

        let account_button = if let Some(session) = &self.session {
            let mut btn_content: Vec<Element<'_, Message>> = Vec::new();
            if let Some(avatar_handle) = &self.avatar {
                btn_content.push(
                    image(avatar_handle.clone())
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                        .into()
                );
            }
            btn_content.push(text(&session.minecraft_profile.name).into());
            button(row![iced::widget::Row::from_vec(btn_content).align_y(Alignment::Center).spacing(6)])
                .on_press(Message::ToggleAccountMenu)
                .padding([8, 16])
                .style(styles::button_success)
        } else if let Some((username, _)) = &self.saved_user_info {
            let angle = (self.animation_tick as f32) * 0.15;
            let spinner = canvas(Spinner { angle })
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0));
            button(row![spinner, text(username)].align_y(Alignment::Center).spacing(6))
                .on_press(Message::ToggleAccountMenu)
                .padding([8, 16])
                .style(styles::button_secondary)
        } else {
            button(text(s.login_with_microsoft))
                .on_press(Message::ToggleAccountMenu)
                .padding([8, 16])
                .style(styles::button_secondary)
        };

        let account_area: Element<'_, Message> = DropDown::new(
            account_button,
            self.view_account_dropdown(),
            self.show_account_menu,
        )
        .on_dismiss(Message::ToggleAccountMenu)
        .offset(4.0)
        .into();

        let settings_button = button(s.settings)
            .on_press(Message::Settings(SettingsMessage::ShowSettings))
            .padding([8, 16])
            .style(styles::button_primary);

        row![title, download_progress, account_area, settings_button]
            .spacing(15)
            .padding([10, 15])
            .align_y(Alignment::Center)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick)
    }
}
