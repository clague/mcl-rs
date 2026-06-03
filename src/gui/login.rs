// Login Module
// Handles Microsoft/Xbox OAuth authentication flow for Minecraft login

use iced::widget::{button, column, container, row, text, text_input, progress_bar};
use iced::{Element, Length, Alignment, Task};
use uuid::Uuid;

use crate::core::auth::{self, AccountSession, get_auth_url};
use crate::gui::styles;
use crate::i18n::strings;

#[derive(Debug, Clone)]
pub enum Message {
    ShowLogin,
    OpenAuthUrl,
    StartAutoLogin,
    WaitForCode,
    CodeReceived(Result<String, String>),
    AuthCodeChanged(String),
    SubmitCode,
    AuthResult(Result<AccountSession, String>),
    CancelLogin,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginState {
    Idle,
    OpeningBrowser,
    WaitingForCallback,
    WaitingForManualCode,
    Authenticating,
    Success,
    Error(String),
}

pub struct Login {
    state: LoginState,
    auth_code: String,
    auth_url: String,
    session: Option<AccountSession>,
    error: Option<String>,
    use_manual_input: bool,
}

impl Login {
    pub fn new() -> Self {
        Self {
            state: LoginState::Idle,
            auth_code: String::new(),
            auth_url: get_auth_url(&Uuid::new_v4().to_string()),
            session: None,
            error: None,
            use_manual_input: false,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.state != LoginState::Idle
    }

    pub fn show(&mut self) {
        self.state = LoginState::OpeningBrowser;
        self.auth_code = String::new();
        self.error = None;
        self.use_manual_input = false;
        self.auth_url = get_auth_url(&Uuid::new_v4().to_string());
    }

    pub fn hide(&mut self) {
        self.state = LoginState::Idle;
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
            Message::OpenAuthUrl => {
                if let Err(e) = open::that(&self.auth_url) {
                    self.error = Some(format!("Failed to open browser: {}", e));
                }
                Task::none()
            }
            Message::StartAutoLogin => {
                self.state = LoginState::WaitingForCallback;
                Task::perform(async { auth::wait_for_auth_code().await }, Message::CodeReceived)
            }
            Message::WaitForCode => {
                self.state = LoginState::WaitingForCallback;
                Task::perform(async { auth::wait_for_auth_code().await }, Message::CodeReceived)
            }
            Message::CodeReceived(result) => {
                match result {
                    Ok(code) => {
                        self.auth_code = code;
                        self.state = LoginState::Authenticating;
                        let code = self.auth_code.clone();
                        Task::perform(async move { auth::full_login_flow(&code).await }, Message::AuthResult)
                    }
                    Err(e) => {
                        if e.contains("busy") || e.contains("Address already in use") {
                            self.use_manual_input = true;
                            self.state = LoginState::WaitingForManualCode;
                            self.error = Some(strings().port_busy.to_string());
                        } else {
                            self.state = LoginState::Error(e);
                        }
                        Task::none()
                    }
                }
            }
            Message::AuthCodeChanged(code) => {
                self.auth_code = code;
                Task::none()
            }
            Message::SubmitCode => {
                if self.auth_code.is_empty() {
                    return Task::none();
                }
                self.state = LoginState::Authenticating;
                self.error = None;
                let code = self.auth_code.clone();
                Task::perform(async move { auth::full_login_flow(&code).await }, Message::AuthResult)
            }
            Message::AuthResult(result) => {
                match result {
                    Ok(session) => {
                        self.state = LoginState::Success;
                        self.session = Some(session);
                        self.error = None;
                    }
                    Err(e) => {
                        self.state = if self.use_manual_input { LoginState::WaitingForManualCode } else { LoginState::OpeningBrowser };
                        self.error = Some(e);
                    }
                }
                Task::none()
            }
            Message::CancelLogin => {
                self.hide();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let s = strings();
        
        if self.state == LoginState::Idle {
            return container(text("")).width(Length::Fill).height(Length::Fill).into();
        }

        let title = text(s.login_with_microsoft_account).size(26);

        let content = match &self.state {
            LoginState::Idle => unreachable!(),
            LoginState::OpeningBrowser => {
                column![
                    text(s.opening_browser).size(18),
                    text(s.complete_login_in_browser).size(16),
                    container(progress_bar(0.0..=1.0, 0.3).style(styles::progress_bar_style)).padding([4, 0]),
                    if self.error.is_some() {
                        button(s.open_login_page_again)
                            .on_press(Message::ShowLogin)
                            .padding([8, 16])
                            .style(styles::button_primary)
                    } else {
                        button(text("")).padding([0, 0])
                    },
                ].spacing(15)
            }
            LoginState::WaitingForCallback => {
                column![
                    text(s.waiting_for_login).size(20),
                    text(s.complete_login_in_browser).size(16),
                    text(s.code_captured_automatically).size(16),
                    container(progress_bar(0.0..=1.0, 0.5).style(styles::progress_bar_style)).padding([4, 0]),
                    button(s.enter_code_manually)
                        .on_press(Message::AuthCodeChanged(String::new()))
                        .padding([8, 16])
                        .style(styles::button_outline),
                ].spacing(15)
            }
            LoginState::WaitingForManualCode => {
                column![
                    text(s.manual_code_entry).size(20),
                    text(s.copy_code_from_url).size(16),
                    text(s.paste_code_below).size(16),
                    button(s.open_login_page_again)
                        .on_press(Message::OpenAuthUrl)
                        .padding([8, 16])
                        .style(styles::button_secondary),
                    column![
                        text(s.authorization_code).size(16),
                        text_input(s.paste_code_here, &self.auth_code)
                            .on_input(Message::AuthCodeChanged)
                            .padding(10)
                            .style(styles::text_input_style),
                    ].spacing(5),
                    if let Some(error) = &self.error {
                        text(s.note.replace("{}", error))
                    } else {
                        text("")
                    },
                ].spacing(15)
            }
            LoginState::Authenticating => {
                column![
                    text(s.authenticating).size(20),
                    text(s.verify_account).size(16),
                    container(progress_bar(0.0..=1.0, 0.7).style(styles::progress_bar_style)).padding([4, 0]),
                ].spacing(15)
            }
            LoginState::Success => {
                if let Some(session) = &self.session {
                    column![
                        text(s.login_successful).size(22),
                        text(s.welcome.replace("{}", &session.minecraft_profile.name)).size(18),
                        text(s.uuid.replace("{}", &session.minecraft_profile.id)).size(16),
                        button(s.close)
                            .on_press(Message::CancelLogin)
                            .padding([10, 20])
                            .style(styles::button_success),
                    ].spacing(15)
                } else {
                    column![text(s.login_successful).size(18)]
                }
            }
            LoginState::Error(e) => {
                column![
                    text(s.login_failed).size(22),
                    text(e).size(16),
                    row![
                        button(s.try_again).on_press(Message::ShowLogin).padding([10, 20]).style(styles::button_primary),
                        button(s.cancel).on_press(Message::CancelLogin).padding([10, 20]).style(styles::button_secondary),
                    ].spacing(10),
                ].spacing(15)
            }
        };

        let buttons = if self.state == LoginState::WaitingForManualCode {
            row![
                button(s.cancel).on_press(Message::CancelLogin).padding([10, 20]).style(styles::button_secondary),
                button(s.submit_code)
                    .on_press_maybe(if !self.auth_code.is_empty() { Some(Message::SubmitCode) } else { None })
                    .padding([10, 20])
                    .style(styles::button_primary),
            ].spacing(10)
        } else if matches!(self.state, LoginState::Success | LoginState::Error(_)) {
            row![]
        } else {
            row![button(s.cancel).on_press(Message::CancelLogin).padding([10, 20]).style(styles::button_secondary)]
        };

        let main_content = column![title, content, buttons.align_y(Alignment::Center)]
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