// Login Module
// Handles Microsoft/Xbox OAuth authentication flow for Minecraft login

use iced::widget::{button, column, container, row, text, text_input, progress_bar};
use iced::{Element, Length, Alignment, Task};
use uuid::Uuid;

use crate::core::auth::{self, AccountSession, get_auth_url};
use crate::gui::styles;

/// Messages that can be dispatched to the login component
#[derive(Debug, Clone)]
pub enum Message {
    /// Show the login dialog
    ShowLogin,
    /// Open the OAuth URL in the default browser
    OpenAuthUrl,
    /// Start the automatic login flow (listen for callback)
    StartAutoLogin,
    /// Wait for the auth code from the local server
    WaitForCode,
    /// Auth code received from callback server
    CodeReceived(Result<String, String>),
    /// Auth code manually changed by user
    AuthCodeChanged(String),
    /// User submitted the manual auth code
    SubmitCode,
    /// Result of the full authentication flow
    AuthResult(Result<AccountSession, String>),
    /// Cancel login and close dialog
    CancelLogin,
}

/// Current state of the login flow
#[derive(Debug, Clone, PartialEq)]
pub enum LoginState {
    /// Login dialog is not visible
    Idle,
    /// Opening the browser for OAuth
    OpeningBrowser,
    /// Waiting for the OAuth callback
    WaitingForCallback,
    /// Port was busy, waiting for manual code input
    WaitingForManualCode,
    /// Authenticating with Xbox/Minecraft services
    Authenticating,
    /// Login completed successfully
    Success,
    /// Login failed with error message
    Error(String),
}

/// Login component state
pub struct Login {
    /// Current login flow state
    state: LoginState,
    /// OAuth authorization code
    auth_code: String,
    /// OAuth authorization URL
    auth_url: String,
    /// Authenticated session (if successful)
    session: Option<AccountSession>,
    /// Error message to display
    error: Option<String>,
    /// Whether we're using manual code input (port was busy)
    use_manual_input: bool,
}

impl Login {
    /// Creates a new Login component with initial state
    pub fn new() -> Self {
        let state = Uuid::new_v4().to_string();
        Self {
            state: LoginState::Idle,
            auth_code: String::new(),
            auth_url: get_auth_url(&state),
            session: None,
            error: None,
            use_manual_input: false,
        }
    }

    /// Returns true if the login dialog should be visible
    pub fn is_visible(&self) -> bool {
        self.state != LoginState::Idle
    }

    /// Shows the login dialog and resets state
    pub fn show(&mut self) {
        self.state = LoginState::OpeningBrowser;
        self.auth_code = String::new();
        self.error = None;
        self.use_manual_input = false;
        let state = Uuid::new_v4().to_string();
        self.auth_url = get_auth_url(&state);
    }

    /// Hides the login dialog
    pub fn hide(&mut self) {
        self.state = LoginState::Idle;
    }

    /// Returns the authenticated session if available
    pub fn get_session(&self) -> Option<&AccountSession> {
        self.session.as_ref()
    }

    /// Handles incoming messages and updates state accordingly
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Show login dialog and open browser
            Message::ShowLogin => {
                self.show();
                Task::perform(
                    async {
                        if let Err(e) = open::that(&get_auth_url(&Uuid::new_v4().to_string())) {
                            return Err(format!("Failed to open browser: {}", e));
                        }
                        Ok(())
                    },
                    |result| match result {
                        Ok(()) => Message::StartAutoLogin,
                        Err(e) => Message::CodeReceived(Err(e)),
                    },
                )
            }
            // Open OAuth URL in browser
            Message::OpenAuthUrl => {
                if let Err(e) = open::that(&self.auth_url) {
                    self.error = Some(format!("Failed to open browser: {}", e));
                }
                Task::none()
            }
            // Start automatic login flow (listen for callback)
            Message::StartAutoLogin => {
                self.state = LoginState::WaitingForCallback;
                Task::perform(
                    async { auth::wait_for_auth_code().await },
                    Message::CodeReceived,
                )
            }
            // Wait for auth code from callback server
            Message::WaitForCode => {
                self.state = LoginState::WaitingForCallback;
                Task::perform(
                    async { auth::wait_for_auth_code().await },
                    Message::CodeReceived,
                )
            }
            // Handle received auth code
            Message::CodeReceived(result) => {
                match result {
                    Ok(code) => {
                        // Code received, start authentication
                        self.auth_code = code;
                        self.state = LoginState::Authenticating;
                        let code = self.auth_code.clone();
                        Task::perform(
                            async move { auth::full_login_flow(&code).await },
                            Message::AuthResult,
                        )
                    }
                    Err(e) => {
                        // If port is busy, switch to manual input mode
                        if e.contains("busy") || e.contains("Address already in use") {
                            self.use_manual_input = true;
                            self.state = LoginState::WaitingForManualCode;
                            self.error = Some("Port 8080 is busy. Please enter the code manually.".to_string());
                        } else {
                            self.state = LoginState::Error(e);
                        }
                        Task::none()
                    }
                }
            }
            // Handle manual auth code input
            Message::AuthCodeChanged(code) => {
                self.auth_code = code;
                Task::none()
            }
            // Submit manual auth code
            Message::SubmitCode => {
                if self.auth_code.is_empty() {
                    self.error = Some("Please enter the authorization code".to_string());
                    return Task::none();
                }
                
                self.state = LoginState::Authenticating;
                self.error = None;
                
                let code = self.auth_code.clone();
                Task::perform(
                    async move { auth::full_login_flow(&code).await },
                    Message::AuthResult,
                )
            }
            // Handle authentication result
            Message::AuthResult(result) => {
                match result {
                    Ok(session) => {
                        self.state = LoginState::Success;
                        self.session = Some(session);
                        self.error = None;
                    }
                    Err(e) => {
                        // Return to appropriate input mode on error
                        if self.use_manual_input {
                            self.state = LoginState::WaitingForManualCode;
                        } else {
                            self.state = LoginState::OpeningBrowser;
                        }
                        self.error = Some(e);
                    }
                }
                Task::none()
            }
            // Cancel login and close dialog
            Message::CancelLogin => {
                self.hide();
                Task::none()
            }
        }
    }

    /// Renders the login dialog view based on current state
    pub fn view(&self) -> Element<'_, Message> {
        let title = text("Login with Microsoft Account").size(24);

        let content = match &self.state {
            // Idle state - return empty view (should not be visible)
            LoginState::Idle => {
                return container(text(""))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
            }
            // Opening browser state
            LoginState::OpeningBrowser => {
                column![
                    text("Opening browser...").size(16),
                    text("Please complete login in your browser").size(14),
                    container(
                        progress_bar(0.0..=1.0, 0.3)
                            .style(styles::progress_bar_style)
                    ).padding([4, 0]),
                ].spacing(15)
            }
            // Waiting for OAuth callback
            LoginState::WaitingForCallback => {
                let instructions = column![
                    text("Waiting for login...").size(18),
                    text("Complete the login in your browser").size(14),
                    text("The code will be captured automatically").size(14),
                ].spacing(8);

                let manual_button = button("Enter code manually")
                    .on_press(Message::AuthCodeChanged(String::new()))
                    .padding([8, 16])
                    .style(styles::button_outline);

                column![
                    instructions,
                    container(
                        progress_bar(0.0..=1.0, 0.5)
                            .style(styles::progress_bar_style)
                    ).padding([4, 0]),
                    manual_button,
                ].spacing(15)
            }
            // Manual code input mode (port was busy)
            LoginState::WaitingForManualCode => {
                let instructions = column![
                    text("Manual Code Entry").size(18),
                    text("1. Copy the code from the browser URL").size(14),
                    text("2. Paste it below").size(14),
                ].spacing(8);

                let open_button = button("Open Login Page Again")
                    .on_press(Message::OpenAuthUrl)
                    .padding([8, 16])
                    .style(styles::button_secondary);

                let code_input = column![
                    text("Authorization Code:").size(14),
                    text_input("Paste code here...", &self.auth_code)
                        .on_input(Message::AuthCodeChanged)
                        .padding(10)
                        .style(styles::text_input_style),
                ].spacing(5);

                let error_text = if let Some(error) = &self.error {
                    text(format!("Note: {}", error))
                } else {
                    text("")
                };

                column![
                    instructions,
                    open_button,
                    code_input,
                    error_text,
                ].spacing(15)
            }
            // Authenticating with Xbox/Minecraft
            LoginState::Authenticating => {
                column![
                    text("Authenticating...").size(18),
                    text("Please wait while we verify your account").size(14),
                    container(
                        progress_bar(0.0..=1.0, 0.7)
                            .style(styles::progress_bar_style)
                    ).padding([4, 0]),
                ].spacing(15)
            }
            // Login successful
            LoginState::Success => {
                if let Some(session) = &self.session {
                    column![
                        text("Login Successful!").size(20),
                        text(format!("Welcome, {}", session.minecraft_profile.name)).size(16),
                        text(format!("UUID: {}", session.minecraft_profile.id)).size(14),
                        button("Close")
                            .on_press(Message::CancelLogin)
                            .padding([10, 20])
                            .style(styles::button_success),
                    ].spacing(15)
                } else {
                    column![text("Login completed").size(16)]
                }
            }
            // Login failed
            LoginState::Error(e) => {
                column![
                    text("Login Failed").size(20),
                    text(e).size(14),
                    row![
                        button("Try Again")
                            .on_press(Message::ShowLogin)
                            .padding([10, 20])
                            .style(styles::button_primary),
                        button("Cancel")
                            .on_press(Message::CancelLogin)
                            .padding([10, 20])
                            .style(styles::button_secondary),
                    ].spacing(10),
                ].spacing(15)
            }
        };

        // Build action buttons based on current state
        let buttons = if self.state == LoginState::WaitingForManualCode {
            // Manual input mode: Cancel and Submit buttons
            row![
                button("Cancel")
                    .on_press(Message::CancelLogin)
                    .padding([10, 20])
                    .style(styles::button_secondary),
                button("Submit Code")
                    .on_press_maybe(if !self.auth_code.is_empty() {
                        Some(Message::SubmitCode)
                    } else {
                        None
                    })
                    .padding([10, 20])
                    .style(styles::button_primary),
            ].spacing(10)
        } else if self.state == LoginState::Success || matches!(self.state, LoginState::Error(_)) {
            // Success or error state: no buttons (handled in content)
            row![]
        } else {
            // Other states: just Cancel button
            row![
                button("Cancel")
                    .on_press(Message::CancelLogin)
                    .padding([10, 20])
                    .style(styles::button_secondary),
            ]
        };

        // Build the main dialog layout
        let main_content = column![
            title,
            content,
            buttons.align_y(Alignment::Center),
        ]
        .spacing(20)
        .padding(30)
        .max_width(500);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(styles::card_container)
            .into()
    }
}