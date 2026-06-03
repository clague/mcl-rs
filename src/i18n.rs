// i18n Module
// Internationalization support for the launcher

use std::sync::LazyLock;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    /// Returns the display name of the language
    pub fn display_name(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }
    
    /// Returns the language code
    pub fn code(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
        }
    }
    
    /// Creates a Language from a language code
    pub fn from_code(code: &str) -> Language {
        match code {
            "zh" | "chinese" | "中文" => Language::Chinese,
            _ => Language::English,
        }
    }
    
    /// Returns all available languages
    pub fn all() -> Vec<Language> {
        vec![Language::English, Language::Chinese]
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Current language setting
static CURRENT_LANGUAGE: LazyLock<std::sync::Mutex<Language>> = 
    LazyLock::new(|| std::sync::Mutex::new(Language::English));

/// Get the current language
pub fn get_language() -> Language {
    *CURRENT_LANGUAGE.lock().unwrap()
}

/// Set the current language
pub fn set_language(lang: Language) {
    *CURRENT_LANGUAGE.lock().unwrap() = lang;
}

/// Localization strings
pub struct Strings {
    // Main window
    pub app_title: &'static str,
    pub versions: &'static str,
    pub account: &'static str,
    pub settings: &'static str,
    pub launch: &'static str,
    pub add: &'static str,
    
    // Version panel
    pub no_versions: &'static str,
    pub no_versions_hint: &'static str,
    pub version_type_release: &'static str,
    pub version_type_snapshot: &'static str,
    pub version_settings: &'static str,
    pub version_number: &'static str,
    pub version_type: &'static str,
    pub display_name: &'static str,
    pub open_folder: &'static str,
    pub delete_version: &'static str,
    pub delete_confirm: &'static str,
    
    // Account panel
    pub not_logged_in: &'static str,
    pub login_with_microsoft: &'static str,
    pub open_browser_to_login: &'static str,
    pub logout: &'static str,
    pub refreshing_token: &'static str,
    pub refreshing_session: &'static str,
    pub please_login_first: &'static str,
    pub select_version_to_launch: &'static str,
    pub session_refresh_failed: &'static str,
    pub refresh_session: &'static str,
    
    // Status bar
    pub all_versions_up_to_date: &'static str,
    pub updates_available: &'static str,
    pub update_check_failed: &'static str,
    pub auto_update_disabled: &'static str,
    pub session_refresh_failed_hint: &'static str,
    pub checking_updates: &'static str,
    
    // Login dialog
    pub login_with_microsoft_account: &'static str,
    pub opening_browser: &'static str,
    pub complete_login_in_browser: &'static str,
    pub waiting_for_login: &'static str,
    pub code_captured_automatically: &'static str,
    pub enter_code_manually: &'static str,
    pub manual_code_entry: &'static str,
    pub copy_code_from_url: &'static str,
    pub paste_code_below: &'static str,
    pub authorization_code: &'static str,
    pub paste_code_here: &'static str,
    pub open_login_page_again: &'static str,
    pub authenticating: &'static str,
    pub verify_account: &'static str,
    pub login_successful: &'static str,
    pub welcome: &'static str,
    pub uuid: &'static str,
    pub close: &'static str,
    pub login_failed: &'static str,
    pub try_again: &'static str,
    pub cancel: &'static str,
    pub submit_code: &'static str,
    pub note: &'static str,
    pub port_busy: &'static str,
    
    // Settings dialog
    pub java_path: &'static str,
    pub java_path_hint: &'static str,
    pub memory_mb: &'static str,
    pub auto_update_on_startup: &'static str,
    pub save: &'static str,
    
    // Add version dialog
    pub add_new_version: &'static str,
    pub filter: &'static str,
    pub select_filter: &'static str,
    pub release: &'static str,
    pub snapshot: &'static str,
    pub all: &'static str,
    pub loading_versions: &'static str,
    pub please_wait: &'static str,
    pub error: &'static str,
    pub retry: &'static str,
    pub selected: &'static str,
    pub select_a_version: &'static str,
    pub add_version: &'static str,
    
    // Download
    pub downloading: &'static str,
    pub download_complete: &'static str,
    pub download_failed: &'static str,
    
    // Errors
    pub failed_to_fetch_manifest: &'static str,
    pub failed_to_parse_version: &'static str,
    pub failed_to_download: &'static str,
    pub failed_to_launch: &'static str,
}

/// English strings
const EN: Strings = Strings {
    // Main window
    app_title: "Minecraft Launcher",
    versions: "Versions",
    account: "Account",
    settings: "Settings",
    launch: "Launch",
    add: "+ Add",
    
    // Version panel
    no_versions: "No versions added yet",
    no_versions_hint: "Click '+ Add' to add a Minecraft version",
    version_type_release: "Release",
    version_type_snapshot: "Snapshot",
    version_settings: "Version Settings",
    version_number: "Version",
    version_type: "Type",
    display_name: "Display Name",
    open_folder: "Open Folder",
    delete_version: "Delete Version",
    delete_confirm: "Delete version files? Shared resources (libraries, assets) will be kept.",
    
    // Account panel
    not_logged_in: "Not logged in",
    login_with_microsoft: "Login with Microsoft",
    open_browser_to_login: "Open browser to login",
    logout: "Logout",
    refreshing_token: "Refreshing token...",
    refreshing_session: "Refreshing session, please wait...",
    please_login_first: "Please login first to launch",
    select_version_to_launch: "Select a version to launch",
    session_refresh_failed: "Session refresh failed",
    refresh_session: "Refresh Session",
    
    // Status bar
    all_versions_up_to_date: "All versions up to date",
    updates_available: "{} update(s) available",
    update_check_failed: "Update check failed: {}",
    auto_update_disabled: "Auto-update disabled",
    checking_updates: "Checking for updates...",
    session_refresh_failed_hint: "Session expired, please refresh or logout",
    
    // Login dialog
    login_with_microsoft_account: "Login with Microsoft Account",
    opening_browser: "Opening browser...",
    complete_login_in_browser: "Please complete login in your browser",
    waiting_for_login: "Waiting for login...",
    code_captured_automatically: "The code will be captured automatically",
    enter_code_manually: "Enter code manually",
    manual_code_entry: "Manual Code Entry",
    copy_code_from_url: "1. Copy the code from the browser URL",
    paste_code_below: "2. Paste it below",
    authorization_code: "Authorization Code:",
    paste_code_here: "Paste code here...",
    open_login_page_again: "Open Login Page Again",
    authenticating: "Authenticating...",
    verify_account: "Please wait while we verify your account",
    login_successful: "Login Successful!",
    welcome: "Welcome, {}",
    uuid: "UUID: {}",
    close: "Close",
    login_failed: "Login Failed",
    try_again: "Try Again",
    cancel: "Cancel",
    submit_code: "Submit Code",
    note: "Note: {}",
    port_busy: "Port 8080 is busy. Please enter the code manually.",
    
    // Settings dialog
    java_path: "Java Path",
    java_path_hint: "Leave empty for auto-detect",
    memory_mb: "Memory (MB)",
    auto_update_on_startup: "Auto-update on startup",
    save: "Save",
    
    // Add version dialog
    add_new_version: "Add New Version",
    filter: "Filter: ",
    select_filter: "Select filter...",
    release: "Release",
    snapshot: "Snapshot",
    all: "All",
    loading_versions: "Loading versions...",
    please_wait: "Please wait...",
    error: "Error: {}",
    retry: "Retry",
    selected: "Selected: {}",
    select_a_version: "Select a version",
    add_version: "Add Version",
    
    // Download
    downloading: "Downloading...",
    download_complete: "Download complete",
    download_failed: "Download failed",
    
    // Errors
    failed_to_fetch_manifest: "Failed to fetch version manifest",
    failed_to_parse_version: "Failed to parse version detail",
    failed_to_download: "Failed to download files",
    failed_to_launch: "Failed to launch game",
};

/// Chinese strings
const ZH: Strings = Strings {
    // Main window
    app_title: "Minecraft 启动器",
    versions: "版本列表",
    account: "账号",
    settings: "设置",
    launch: "启动",
    add: "+ 添加",
    
    // Version panel
    no_versions: "还没有添加版本",
    no_versions_hint: "点击「+ 添加」来添加 Minecraft 版本",
    version_type_release: "正式版",
    version_type_snapshot: "快照版",
    version_settings: "版本设置",
    version_number: "版本号",
    version_type: "类型",
    display_name: "显示名称",
    open_folder: "打开文件夹",
    delete_version: "删除版本",
    delete_confirm: "删除版本文件？共享资源（libraries、assets）将保留。",
    
    // Account panel
    not_logged_in: "未登录",
    login_with_microsoft: "使用 Microsoft 登录",
    open_browser_to_login: "打开浏览器登录",
    logout: "退出登录",
    refreshing_token: "正在刷新令牌...",
    refreshing_session: "正在刷新会话，请稍候...",
    please_login_first: "请先登录再启动游戏",
    select_version_to_launch: "选择一个版本启动",
    session_refresh_failed: "会话刷新失败",
    refresh_session: "刷新会话",
    
    // Status bar
    all_versions_up_to_date: "所有版本已是最新",
    updates_available: "有 {} 个更新可用",
    update_check_failed: "更新检查失败: {}",
    auto_update_disabled: "自动更新已禁用",
    checking_updates: "正在检查更新...",
    session_refresh_failed_hint: "会话已过期，请刷新或退出登录",
    
    // Login dialog
    login_with_microsoft_account: "使用 Microsoft 账号登录",
    opening_browser: "正在打开浏览器...",
    complete_login_in_browser: "请在浏览器中完成登录",
    waiting_for_login: "等待登录中...",
    code_captured_automatically: "授权码将自动捕获",
    enter_code_manually: "手动输入授权码",
    manual_code_entry: "手动输入授权码",
    copy_code_from_url: "1. 从浏览器 URL 复制授权码",
    paste_code_below: "2. 粘贴到下方",
    authorization_code: "授权码：",
    paste_code_here: "在此粘贴授权码...",
    open_login_page_again: "重新打开登录页面",
    authenticating: "正在认证...",
    verify_account: "请稍候，正在验证您的账号",
    login_successful: "登录成功！",
    welcome: "欢迎, {}",
    uuid: "UUID: {}",
    close: "关闭",
    login_failed: "登录失败",
    try_again: "重试",
    cancel: "取消",
    submit_code: "提交授权码",
    note: "注意: {}",
    port_busy: "端口 8080 被占用，请手动输入授权码。",
    
    // Settings dialog
    java_path: "Java 路径",
    java_path_hint: "留空自动检测",
    memory_mb: "内存 (MB)",
    auto_update_on_startup: "启动时自动更新",
    save: "保存",
    
    // Add version dialog
    add_new_version: "添加新版本",
    filter: "筛选: ",
    select_filter: "选择筛选...",
    release: "正式版",
    snapshot: "快照版",
    all: "全部",
    loading_versions: "正在加载版本...",
    please_wait: "请稍候...",
    error: "错误: {}",
    retry: "重试",
    selected: "已选择: {}",
    select_a_version: "选择一个版本",
    add_version: "添加版本",
    
    // Download
    downloading: "正在下载...",
    download_complete: "下载完成",
    download_failed: "下载失败",
    
    // Errors
    failed_to_fetch_manifest: "获取版本清单失败",
    failed_to_parse_version: "解析版本详情失败",
    failed_to_download: "下载文件失败",
    failed_to_launch: "启动游戏失败",
};

/// Get localized strings for current language
pub fn strings() -> &'static Strings {
    match get_language() {
        Language::English => &EN,
        Language::Chinese => &ZH,
    }
}

/// Helper macro for quick access to localized strings
#[macro_export]
macro_rules! t {
    ($field:ident) => {
        $crate::i18n::strings().$field
    };
}