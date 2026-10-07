use iced::{Element, Task, widget::container};

use crate::models::{
    AsyncModel,
    integration::{Integration as IntegrationModel, IntegrationType},
};
use crate::repository::integration as integration_repository;

const KAVITA_LOGO: &[u8] = include_bytes!("../../assets/kavita-favicon.ico");

#[derive(Debug, Default)]
struct KavitaForm {
    name: String,
    url: String,
    api_key: String,
}

#[derive(Debug)]
enum Step {
    List(AsyncModel<Vec<IntegrationModel>, String>),
    ChooseType,
    KavitaForm(KavitaForm),
}

#[derive(Debug)]
pub struct Integrations {
    step: Step,
}

impl Default for Integrations {
    fn default() -> Self {
        Self {
            step: Step::List(AsyncModel::Loading),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    IntegrationsLoaded(Result<Vec<IntegrationModel>, String>),
    AddIntegrationPressed,
    TypeSelected(IntegrationType),
    BackToListPressed,
    KavitaNameChanged(String),
    KavitaUrlChanged(String),
    KavitaApiKeyChanged(String),
    KavitaFormSubmitted,
    IntegrationCreated(Result<IntegrationModel, String>),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
}

impl Integrations {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(integration_repository::all(), Message::IntegrationsLoaded),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::IntegrationsLoaded(result) => {
                self.step = Step::List(result.into());
                Action::None
            }

            Message::AddIntegrationPressed => {
                self.step = Step::ChooseType;
                Action::None
            }

            Message::TypeSelected(IntegrationType::Kavita) => {
                self.step = Step::KavitaForm(KavitaForm::default());
                Action::None
            }

            Message::BackToListPressed => {
                self.step = Step::List(AsyncModel::Loading);
                Action::Run(Task::perform(
                    integration_repository::all(),
                    Message::IntegrationsLoaded,
                ))
            }

            Message::KavitaNameChanged(name) => {
                if let Step::KavitaForm(form) = &mut self.step {
                    form.name = name;
                }
                Action::None
            }

            Message::KavitaUrlChanged(url) => {
                if let Step::KavitaForm(form) = &mut self.step {
                    form.url = url;
                }
                Action::None
            }

            Message::KavitaApiKeyChanged(api_key) => {
                if let Step::KavitaForm(form) = &mut self.step {
                    form.api_key = api_key;
                }
                Action::None
            }

            Message::KavitaFormSubmitted => {
                let Step::KavitaForm(form) = &self.step else {
                    return Action::None;
                };

                Action::Run(Task::perform(
                    integration_repository::create(
                        form.name.clone(),
                        IntegrationType::Kavita,
                        Some(form.url.clone()),
                        Some(form.api_key.clone()),
                    ),
                    Message::IntegrationCreated,
                ))
            }

            Message::IntegrationCreated(Ok(integration)) => {
                self.step = Step::List(AsyncModel::Loading);
                Action::Run(Task::future(async move {
                    crate::jobs::sync_integration_libraries::spawn(integration.id);
                    Message::IntegrationsLoaded(integration_repository::all().await)
                }))
            }
            Message::IntegrationCreated(Err(_)) => Action::None,
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        match &self.step {
            Step::List(AsyncModel::Loaded(integrations)) => view_helper::list(integrations),
            Step::List(_) => container(iced::widget::row![]).into(),
            Step::ChooseType => view_helper::choose_type(),
            Step::KavitaForm(form) => view_helper::kavita_form(form),
        }
    }
}

mod view_helper {
    use iced::{
        Alignment, Element,
        widget::{button, column, container, image, row, text, text_input},
    };

    use super::{IntegrationModel, IntegrationType, KAVITA_LOGO, KavitaForm, Message};

    pub fn list<'a>(integrations: &'a [IntegrationModel]) -> Element<'a, Message> {
        if integrations.is_empty() {
            return empty_list();
        }

        let items = integrations.iter().map(|integration| {
            row![
                text(&integration.name),
                text(integration.integration_type.label())
            ]
            .spacing(10)
            .into()
        });

        container(column(items)).into()
    }

    fn empty_list<'a>() -> Element<'a, Message> {
        container(column![
            text(
                "Você não tem nenhuma integração cadastrada, adicione uma integração com algum serviço"
            ),
            button("Add Integration").on_press(Message::AddIntegrationPressed)
        ])
        .into()
    }

    pub fn choose_type<'a>() -> Element<'a, Message> {
        container(column![
            row![type_square("Kavita", KAVITA_LOGO, IntegrationType::Kavita)].spacing(10),
            button("Back").on_press(Message::BackToListPressed)
        ])
        .into()
    }

    fn type_square<'a>(
        name: &'a str,
        logo: &'static [u8],
        integration_type: IntegrationType,
    ) -> Element<'a, Message> {
        button(
            column![image(image::Handle::from_bytes(logo)).width(64).height(64), text(name)]
                .align_x(Alignment::Center)
                .spacing(5),
        )
        .width(120)
        .height(120)
        .on_press(Message::TypeSelected(integration_type))
        .into()
    }

    pub fn kavita_form<'a>(form: &'a KavitaForm) -> Element<'a, Message> {
        container(column![
            text_input("Name", &form.name).on_input(Message::KavitaNameChanged),
            text_input("Server URL", &form.url).on_input(Message::KavitaUrlChanged),
            text_input("API Key", &form.api_key).on_input(Message::KavitaApiKeyChanged),
            row![
                button("Back").on_press(Message::BackToListPressed),
                button("Save").on_press(Message::KavitaFormSubmitted)
            ]
            .spacing(10)
        ]
        .spacing(10))
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
    fn view_renders_empty_container_while_not_loaded() {
        let integrations = Integrations {
            step: Step::List(AsyncModel::Loading),
        };

        let mut ui = simulator(integrations.view());

        assert!(ui.find("Add Integration").is_err());
    }

    #[test]
    fn view_renders_call_to_action_when_loaded_without_integrations() {
        let integrations = Integrations {
            step: Step::List(AsyncModel::Loaded(vec![])),
        };

        let mut ui = simulator(integrations.view());

        assert!(ui.find("Add Integration").is_ok());
    }

    #[test]
    fn view_renders_integration_names_when_loaded() {
        let integrations = Integrations {
            step: Step::List(AsyncModel::Loaded(vec![integration_model("My Kavita")])),
        };

        let mut ui = simulator(integrations.view());

        assert!(ui.find("My Kavita").is_ok());
    }

    #[test]
    fn clicking_add_integration_sends_add_integration_pressed_message() {
        let integrations = Integrations {
            step: Step::List(AsyncModel::Loaded(vec![])),
        };

        let mut ui = simulator(integrations.view());
        let _ = ui
            .click("Add Integration")
            .expect("Add Integration button should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(messages, vec![Message::AddIntegrationPressed]);
    }

    #[test]
    fn update_add_integration_pressed_shows_choose_type_step() {
        let mut integrations = Integrations::default();

        let action = integrations.update(Message::AddIntegrationPressed);

        assert!(matches!(action, Action::None));
        assert!(matches!(integrations.step, Step::ChooseType));
    }

    #[test]
    fn view_choose_type_renders_kavita_option() {
        let integrations = Integrations {
            step: Step::ChooseType,
        };

        let mut ui = simulator(integrations.view());

        assert!(ui.find("Kavita").is_ok());
    }

    #[test]
    fn clicking_kavita_square_sends_type_selected_message() {
        let integrations = Integrations {
            step: Step::ChooseType,
        };

        let mut ui = simulator(integrations.view());
        let _ = ui.click("Kavita").expect("Kavita square should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(
            messages,
            vec![Message::TypeSelected(IntegrationType::Kavita)]
        );
    }

    #[test]
    fn update_type_selected_shows_kavita_form_step() {
        let mut integrations = Integrations::default();

        let action = integrations.update(Message::TypeSelected(IntegrationType::Kavita));

        assert!(matches!(action, Action::None));
        assert!(matches!(integrations.step, Step::KavitaForm(_)));
    }

    #[test]
    fn view_kavita_form_renders_inputs_and_buttons() {
        let integrations = Integrations {
            step: Step::KavitaForm(KavitaForm::default()),
        };

        let mut ui = simulator(integrations.view());

        assert!(ui.find("Back").is_ok());
        assert!(ui.find("Save").is_ok());
    }

    #[test]
    fn update_kavita_name_changed_stores_value() {
        let mut integrations = Integrations {
            step: Step::KavitaForm(KavitaForm::default()),
        };

        let _ = integrations.update(Message::KavitaNameChanged("My Kavita".to_string()));

        let Step::KavitaForm(form) = &integrations.step else {
            panic!("expected KavitaForm step");
        };
        assert_eq!(form.name, "My Kavita");
    }

    #[test]
    fn update_kavita_form_submitted_runs_a_task() {
        let mut integrations = Integrations {
            step: Step::KavitaForm(KavitaForm {
                name: "My Kavita".to_string(),
                url: "http://localhost:5000".to_string(),
                api_key: "api-key".to_string(),
            }),
        };

        let action = integrations.update(Message::KavitaFormSubmitted);

        assert!(matches!(action, Action::Run(_)));
    }

    #[test]
    fn update_integration_created_ok_returns_to_list_and_reloads() {
        let mut integrations = Integrations {
            step: Step::KavitaForm(KavitaForm::default()),
        };

        let action = integrations.update(Message::IntegrationCreated(Ok(integration_model(
            "My Kavita",
        ))));

        assert!(matches!(action, Action::Run(_)));
        assert!(matches!(integrations.step, Step::List(AsyncModel::Loading)));
    }

    #[test]
    fn update_integration_created_err_returns_no_action() {
        let mut integrations = Integrations {
            step: Step::KavitaForm(KavitaForm::default()),
        };

        let action = integrations.update(Message::IntegrationCreated(Err("boom".to_string())));

        assert!(matches!(action, Action::None));
    }
}
