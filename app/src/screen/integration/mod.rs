pub mod kavita;

use iced::{Element, Task, widget::container};

use crate::models::{
    AsyncModel,
    integration::{Integration as IntegrationModel, IntegrationType},
};
use crate::repository::integration as integration_repository;
use kavita::Kavita;

#[derive(Debug)]
enum Step {
    List(AsyncModel<Vec<IntegrationModel>, String>),
    ChooseType,
    Kavita(Kavita),
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
    Kavita(kavita::Message),
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
                self.step = Step::Kavita(Kavita::default());
                Action::None
            }

            Message::TypeSelected(_) => Action::None,

            Message::Kavita(msg) => {
                let Step::Kavita(kavita) = &mut self.step else {
                    return Action::None;
                };

                match kavita.update(msg) {
                    kavita::Action::None => Action::None,
                    kavita::Action::Run(task) => Action::Run(task.map(Message::Kavita)),
                    kavita::Action::BackPressed => self.reload_list(),
                    kavita::Action::Created(integration) => {
                        self.step = Step::List(AsyncModel::Loading);
                        Action::Run(Task::future(async move {
                            crate::jobs::sync_integration_libraries::spawn(integration.id);
                            Message::IntegrationsLoaded(integration_repository::all().await)
                        }))
                    }
                }
            }
        }
    }

    fn reload_list(&mut self) -> Action {
        self.step = Step::List(AsyncModel::Loading);
        Action::Run(Task::perform(
            integration_repository::all(),
            Message::IntegrationsLoaded,
        ))
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        match &self.step {
            Step::List(AsyncModel::Loaded(integrations)) => view_helper::list(integrations),
            Step::List(_) => container(iced::widget::row![]).into(),
            Step::ChooseType => view_helper::choose_type(),
            Step::Kavita(kavita) => kavita.view().map(Message::Kavita),
        }
    }
}

mod view_helper {
    use super::{IntegrationModel, IntegrationType, Message};
    use crate::icons;
    use iced::{
        Center, Element,
        Length::Fill,
        Theme,
        widget::{button, column, container, grid, row, svg, text},
    };

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
        let content = column![
            svg(svg::Handle::from_memory(icons::PLUG_CONNECT))
                .height(64)
                .width(64)
                .style(|theme: &Theme, _status| svg::Style {
                    color: Some(theme.extended_palette().background.neutral.color),
                }),
            text(
                "Você não tem nenhuma integração cadastrada, adicione uma integração com algum serviço"
            ),
            button("Add Integration").on_press(Message::AddIntegrationPressed)
        ]
        .spacing(20)
        .max_width(420)
        .align_x(Center);

        container(content).center(Fill).into()
    }

    pub fn choose_type<'a>() -> Element<'a, Message> {
        container(column![
            grid(vec![
                type_square("Kavita", icons::KAVITA, IntegrationType::Kavita),
                type_square("Komga", icons::KOMGA, IntegrationType::Komga),
                type_square("OPDS", icons::OPDS, IntegrationType::Opds),
                type_square("Suwayomi", icons::SUWAYOMI, IntegrationType::Suwayomi)
            ])
            .columns(2)
            .height(256.0)
            .width(256.0)
            .spacing(25),
        ])
        .center(Fill)
        .into()
    }

    fn type_square<'a>(
        name: &'a str,
        logo: &'static [u8],
        integration_type: IntegrationType,
    ) -> Element<'a, Message> {
        button(
            container(
                column![
                    svg(svg::Handle::from_memory(logo)).width(48).height(48),
                    text(name)
                ]
                .align_x(Center)
                .spacing(5),
            )
            .center(Fill),
        )
        .width(Fill)
        .height(Fill)
        .on_press(Message::TypeSelected(integration_type))
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
        assert!(matches!(integrations.step, Step::Kavita(_)));
    }

    #[test]
    fn update_kavita_back_pressed_returns_to_list_and_reloads() {
        let mut integrations = Integrations {
            step: Step::Kavita(Kavita::default()),
        };

        let action = integrations.update(Message::Kavita(kavita::Message::BackPressed));

        assert!(matches!(action, Action::Run(_)));
        assert!(matches!(integrations.step, Step::List(AsyncModel::Loading)));
    }

    #[test]
    fn update_kavita_created_returns_to_list_and_reloads() {
        let mut integrations = Integrations {
            step: Step::Kavita(Kavita::default()),
        };

        let action = integrations.update(Message::Kavita(kavita::Message::Created(Ok(
            integration_model("My Kavita"),
        ))));

        assert!(matches!(action, Action::Run(_)));
        assert!(matches!(integrations.step, Step::List(AsyncModel::Loading)));
    }
}
