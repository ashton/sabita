use iced::{Element, Task};

use crate::models::integration::{Integration as IntegrationModel, IntegrationType};
use crate::providers::kavita::provider::KavitaProvider;
use crate::repository::integration as integration_repository;

#[derive(Debug, Default)]
pub struct Kavita {
    name: String,
    url: String,
    api_key: String,
    connection_test: Option<Result<(), String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    NameChanged(String),
    UrlChanged(String),
    ApiKeyChanged(String),
    BackPressed,
    Submitted,
    Created(Result<IntegrationModel, String>),
    TestConnectionPressed,
    ConnectionTested(Result<(), String>),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    BackPressed,
    Created(IntegrationModel),
}

impl Kavita {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NameChanged(name) => {
                self.name = name;
                Action::None
            }

            Message::UrlChanged(url) => {
                self.url = url;
                Action::None
            }

            Message::ApiKeyChanged(api_key) => {
                self.api_key = api_key;
                Action::None
            }

            Message::BackPressed => Action::BackPressed,

            Message::Submitted => Action::Run(Task::perform(
                integration_repository::create(
                    self.name.clone(),
                    IntegrationType::Kavita,
                    Some(self.url.clone()),
                    Some(self.api_key.clone()),
                ),
                Message::Created,
            )),

            Message::Created(Ok(integration)) => Action::Created(integration),
            Message::Created(Err(_)) => Action::None,

            Message::TestConnectionPressed => {
                self.connection_test = None;
                Action::Run(Task::perform(
                    crate::runtime::on_tokio(KavitaProvider::ping(self.url.clone())),
                    Message::ConnectionTested,
                ))
            }

            Message::ConnectionTested(result) => {
                self.connection_test = Some(result);
                Action::None
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        view_helper::form(self)
    }
}

mod view_helper {
    use super::{Kavita, Message};
    use iced::{
        Center, Element,
        widget::{button, column, container, row, text, text_input},
    };

    pub fn form<'a>(kavita: &'a Kavita) -> Element<'a, Message> {
        container(
            column![
                text_input("Name", &kavita.name).on_input(Message::NameChanged),
                text_input("Server URL", &kavita.url).on_input(Message::UrlChanged),
                text_input("API Key", &kavita.api_key).on_input(Message::ApiKeyChanged),
                row![
                    button("Test Connection").on_press(Message::TestConnectionPressed),
                    connection_toast(kavita),
                ]
                .spacing(10)
                .align_y(Center),
                row![
                    button("Back").on_press(Message::BackPressed),
                    button("Save").on_press(Message::Submitted)
                ]
                .spacing(10)
            ]
            .spacing(10),
        )
        .into()
    }

    fn connection_toast<'a>(kavita: &'a Kavita) -> Element<'a, Message> {
        match &kavita.connection_test {
            Some(Ok(())) => text("Connection successful").style(text::success).into(),
            Some(Err(error)) => text(error).style(text::danger).into(),
            None => row![].into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use iced_test::simulator;

    use super::*;

    fn integration_model(name: &str) -> IntegrationModel {
        IntegrationModel {
            id: "id".to_string(),
            name: name.to_string(),
            integration_type: IntegrationType::Kavita,
            url: Some("http://localhost:5000".to_string()),
            api_key: Some("api-key".to_string()),
        }
    }

    #[test]
    fn view_renders_inputs_and_buttons() {
        let kavita = Kavita::default();

        let mut ui = simulator(kavita.view());

        assert!(ui.find("Back").is_ok());
        assert!(ui.find("Save").is_ok());
    }

    #[test]
    fn update_name_changed_stores_value() {
        let mut kavita = Kavita::default();

        let _ = kavita.update(Message::NameChanged("My Kavita".to_string()));

        assert_eq!(kavita.name, "My Kavita");
    }

    #[test]
    fn update_back_pressed_returns_back_pressed_action() {
        let mut kavita = Kavita::default();

        let action = kavita.update(Message::BackPressed);

        assert!(matches!(action, Action::BackPressed));
    }

    #[test]
    fn update_submitted_runs_a_task() {
        let mut kavita = Kavita {
            name: "My Kavita".to_string(),
            url: "http://localhost:5000".to_string(),
            api_key: "api-key".to_string(),
            connection_test: None,
        };

        let action = kavita.update(Message::Submitted);

        assert!(matches!(action, Action::Run(_)));
    }

    #[test]
    fn update_created_ok_returns_created_action() {
        let mut kavita = Kavita::default();

        let action = kavita.update(Message::Created(Ok(integration_model("My Kavita"))));

        assert!(matches!(action, Action::Created(_)));
    }

    #[test]
    fn update_created_err_returns_no_action() {
        let mut kavita = Kavita::default();

        let action = kavita.update(Message::Created(Err("boom".to_string())));

        assert!(matches!(action, Action::None));
    }

    #[test]
    fn view_renders_test_connection_button() {
        let kavita = Kavita::default();

        let mut ui = simulator(kavita.view());

        assert!(ui.find("Test Connection").is_ok());
    }

    #[test]
    fn clicking_test_connection_sends_test_connection_pressed_message() {
        let kavita = Kavita::default();

        let mut ui = simulator(kavita.view());
        let _ = ui
            .click("Test Connection")
            .expect("Test Connection button should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(messages, vec![Message::TestConnectionPressed]);
    }

    #[test]
    fn update_test_connection_pressed_clears_previous_result_and_runs_a_task() {
        let mut kavita = Kavita {
            connection_test: Some(Ok(())),
            ..Kavita::default()
        };

        let action = kavita.update(Message::TestConnectionPressed);

        assert!(matches!(action, Action::Run(_)));
        assert_eq!(kavita.connection_test, None);
    }

    #[test]
    fn update_connection_tested_ok_stores_success() {
        let mut kavita = Kavita::default();

        let action = kavita.update(Message::ConnectionTested(Ok(())));

        assert!(matches!(action, Action::None));
        assert_eq!(kavita.connection_test, Some(Ok(())));
    }

    #[test]
    fn update_connection_tested_err_stores_error() {
        let mut kavita = Kavita::default();

        let action = kavita.update(Message::ConnectionTested(Err("boom".to_string())));

        assert!(matches!(action, Action::None));
        assert_eq!(kavita.connection_test, Some(Err("boom".to_string())));
    }

    #[test]
    fn view_renders_success_message_after_successful_test() {
        let kavita = Kavita {
            connection_test: Some(Ok(())),
            ..Kavita::default()
        };

        let mut ui = simulator(kavita.view());

        assert!(ui.find("Connection successful").is_ok());
    }

    #[test]
    fn view_renders_error_message_after_failed_test() {
        let kavita = Kavita {
            connection_test: Some(Err("connection refused".to_string())),
            ..Kavita::default()
        };

        let mut ui = simulator(kavita.view());

        assert!(ui.find("connection refused").is_ok());
    }
}
