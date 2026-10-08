use iced::{Element, Task};

use crate::models::integration::{Integration as IntegrationModel, IntegrationType};
use crate::repository::integration as integration_repository;

#[derive(Debug, Default)]
pub struct Kavita {
    name: String,
    url: String,
    api_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    NameChanged(String),
    UrlChanged(String),
    ApiKeyChanged(String),
    BackPressed,
    Submitted,
    Created(Result<IntegrationModel, String>),
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
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        view_helper::form(self)
    }
}

mod view_helper {
    use super::{Kavita, Message};
    use iced::{
        Element,
        widget::{button, column, container, row, text_input},
    };

    pub fn form<'a>(kavita: &'a Kavita) -> Element<'a, Message> {
        container(
            column![
                text_input("Name", &kavita.name).on_input(Message::NameChanged),
                text_input("Server URL", &kavita.url).on_input(Message::UrlChanged),
                text_input("API Key", &kavita.api_key).on_input(Message::ApiKeyChanged),
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
}
