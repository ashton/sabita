use std::path::PathBuf;

use iced::{
    Element, Task,
    widget::{container, row},
};

use crate::models::{
    AsyncModel, integration::Integration as IntegrationModel, library::Library as LibraryModel,
};
use crate::repository::{integration as integration_repository, library as library_repository};

#[derive(Debug, Default)]
pub struct Library {
    items: AsyncModel<Vec<LibraryModel>, String>,
    integrations: AsyncModel<Vec<IntegrationModel>, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    LibrariesLoaded(Result<Vec<LibraryModel>, String>),
    IntegrationsLoaded(Result<Vec<IntegrationModel>, String>),
    AddFolderPressed,
    FolderPicked(Option<PathBuf>),
    LibraryCreated(Result<LibraryModel, String>),
    BrowseRemoteLibrary(String),
    BrowseLibrary(String),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    BrowseRemoteLibrary(String),
    BrowseLibrary(String),
}

impl Library {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                items: AsyncModel::Loading,
                integrations: AsyncModel::Loading,
            },
            Task::batch([
                Task::perform(library_repository::all(), Message::LibrariesLoaded),
                Task::perform(integration_repository::all(), Message::IntegrationsLoaded),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::LibrariesLoaded(result) => {
                self.items = result.into();
                Action::None
            }

            Message::IntegrationsLoaded(result) => {
                self.integrations = result.into();
                Action::None
            }

            Message::AddFolderPressed => {
                Action::Run(Task::perform(pick_folder(), Message::FolderPicked))
            }

            Message::FolderPicked(Some(folder)) => {
                let name = folder
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| folder.to_string_lossy().into_owned());

                Action::Run(Task::perform(
                    library_repository::create_from_folder(
                        folder.to_string_lossy().into_owned(),
                        name,
                    ),
                    Message::LibraryCreated,
                ))
            }
            Message::FolderPicked(None) => Action::None,

            Message::LibraryCreated(Ok(_)) => Action::Run(Task::perform(
                library_repository::all(),
                Message::LibrariesLoaded,
            )),
            Message::LibraryCreated(Err(_)) => Action::None,

            Message::BrowseRemoteLibrary(external_id) => Action::BrowseRemoteLibrary(external_id),
            Message::BrowseLibrary(library_id) => Action::BrowseLibrary(library_id),
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let integrations: &[IntegrationModel] = match &self.integrations {
            AsyncModel::Loaded(integrations) => integrations,
            _ => &[],
        };

        match &self.items {
            AsyncModel::Loaded(libraries) => view_helper::list(libraries, integrations),
            _ => container(row![]).into(),
        }
    }
}

async fn pick_folder() -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());

    rfd::AsyncFileDialog::new()
        .set_directory(home)
        .pick_folder()
        .await
        .map(|handle| handle.path().to_path_buf())
}

mod view_helper {
    use iced::{
        Background, Border, Center, Element,
        Length::Fill,
        Theme,
        widget::{button, column, container, row, space, svg, text},
    };
    use iced_font_awesome::fa_icon_solid;

    use crate::icons;
    use crate::models::integration::Integration;
    use crate::models::library::{Library, LibraryType};

    pub fn list<'a>(
        libraries: &'a [Library],
        integrations: &'a [Integration],
    ) -> Element<'a, super::Message> {
        if libraries.is_empty() {
            return empty_list();
        }

        let rows = libraries
            .iter()
            .map(|library| library_row(library, server_name(library, integrations)));

        container(column(rows).spacing(8)).padding(20).into()
    }

    fn server_name<'a>(library: &Library, integrations: &'a [Integration]) -> Option<&'a str> {
        let integration_id = library.integration_id.as_ref()?;

        integrations
            .iter()
            .find(|integration| &integration.id == integration_id)
            .map(|integration| integration.name.as_str())
    }

    fn library_row<'a>(
        library: &'a Library,
        server_name: Option<&'a str>,
    ) -> Element<'a, super::Message> {
        container(
            row![
                icon_badge(kind_icon(&library.kind)),
                column![
                    row![
                        text(&library.name),
                        text(library.kind.label()).size(12).style(text::secondary),
                    ]
                    .spacing(6)
                    .align_y(Center),
                    server_tag(server_name),
                ]
                .spacing(4),
                space::horizontal(),
                button(
                    row![fa_icon_solid("folder-open").size(14.0_f32), text("Browse")]
                        .spacing(6)
                        .align_y(Center)
                )
                .on_press(browse_message(library))
            ]
            .spacing(12)
            .align_y(Center)
            .padding(12),
        )
        .width(Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(
                theme.extended_palette().background.weak.color,
            )),
            border: Border {
                radius: 8.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
    }

    fn browse_message(library: &Library) -> super::Message {
        match &library.external_id {
            Some(external_id) => super::Message::BrowseRemoteLibrary(external_id.clone()),
            None => super::Message::BrowseLibrary(library.id.clone()),
        }
    }

    fn icon_badge<'a>(icon: &'static [u8]) -> Element<'a, super::Message> {
        container(
            svg(svg::Handle::from_memory(icon))
                .width(20)
                .height(20)
                .style(|theme: &Theme, _status| svg::Style {
                    color: Some(theme.palette().text),
                }),
        )
        .center_x(40)
        .center_y(40)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(
                theme.extended_palette().background.strong.color,
            )),
            border: Border {
                radius: 8.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
    }

    fn server_tag<'a>(server_name: Option<&'a str>) -> Element<'a, super::Message> {
        match server_name {
            Some(name) => container(text(name).size(12))
                .padding([2, 8])
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        background: Some(Background::Color(palette.secondary.weak.color)),
                        text_color: Some(palette.secondary.weak.text),
                        border: Border {
                            radius: 999.0.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    }
                })
                .into(),
            None => row![].into(),
        }
    }

    fn kind_icon(kind: &LibraryType) -> &'static [u8] {
        match kind {
            LibraryType::Manga => icons::MANGA,
            LibraryType::Comic => icons::COMIC,
            LibraryType::Ebook => icons::EBOOK,
        }
    }

    fn empty_list<'a>() -> Element<'a, super::Message> {
        let content = column![
            svg(svg::Handle::from_memory(icons::BOOK_OPEN))
                .height(64)
                .width(64)
                .style(|theme: &Theme, _status| svg::Style {
                        color: Some(theme.extended_palette().background.neutral.color),
                    }
                ),
            text!(
                "Você não tem nenhuma biblioteca cadastrada, adicione uma pasta ou uma integração com algum serviço"
            )
            .center(),
            row![
                button("Add Files"),
                button("Add Folder").on_press(super::Message::AddFolderPressed),
            ]
            .spacing(10),
        ]
        .spacing(20)
        .max_width(420)
        .align_x(Center);

        container(content).center(Fill).into()
    }
}

#[cfg(test)]
mod tests {
    use iced_test::simulator;

    use super::*;
    use crate::models::integration::IntegrationType;

    fn library_model(name: &str) -> LibraryModel {
        LibraryModel {
            id: "id".to_string(),
            kind: crate::models::library::LibraryType::Comic,
            name: name.to_string(),
            external_id: None,
            folder: None,
            cover: None,
            integration_id: None,
        }
    }

    fn library_model_with_integration(name: &str, integration_id: &str) -> LibraryModel {
        LibraryModel {
            integration_id: Some(integration_id.to_string()),
            ..library_model(name)
        }
    }

    fn library_model_with_remote_id(name: &str, external_id: &str) -> LibraryModel {
        LibraryModel {
            external_id: Some(external_id.to_string()),
            ..library_model(name)
        }
    }

    fn integration_model(id: &str, name: &str) -> IntegrationModel {
        IntegrationModel {
            id: id.to_string(),
            name: name.to_string(),
            integration_type: IntegrationType::Kavita,
            url: None,
            api_key: None,
        }
    }

    #[test]
    fn view_renders_empty_container_while_not_loaded() {
        let library = Library {
            items: AsyncModel::Loading,
            integrations: AsyncModel::Loading,
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("Add Folder").is_err());
    }

    #[test]
    fn view_renders_call_to_action_when_loaded_without_libraries() {
        let library = Library {
            items: AsyncModel::Loaded(vec![]),
            integrations: AsyncModel::Loaded(vec![]),
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("Add Folder").is_ok());
        assert!(ui.find("Add Files").is_ok());
    }

    #[test]
    fn view_renders_library_names_when_loaded() {
        let library = Library {
            items: AsyncModel::Loaded(vec![library_model("My Library")]),
            integrations: AsyncModel::Loaded(vec![]),
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("My Library").is_ok());
    }

    #[test]
    fn view_renders_library_kind_label_when_loaded() {
        let library = Library {
            items: AsyncModel::Loaded(vec![library_model("My Library")]),
            integrations: AsyncModel::Loaded(vec![]),
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("Comic").is_ok());
    }

    #[test]
    fn view_renders_browse_button_when_loaded() {
        let library = Library {
            items: AsyncModel::Loaded(vec![library_model("My Library")]),
            integrations: AsyncModel::Loaded(vec![]),
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("Browse").is_ok());
    }

    #[test]
    fn view_renders_server_tag_for_library_from_an_integration() {
        let library = Library {
            items: AsyncModel::Loaded(vec![library_model_with_integration(
                "My Library",
                "integration-id",
            )]),
            integrations: AsyncModel::Loaded(vec![integration_model(
                "integration-id",
                "My Kavita",
            )]),
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("My Kavita").is_ok());
    }

    #[test]
    fn view_renders_no_server_tag_for_local_library() {
        let library = Library {
            items: AsyncModel::Loaded(vec![library_model("My Library")]),
            integrations: AsyncModel::Loaded(vec![integration_model(
                "integration-id",
                "My Kavita",
            )]),
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("My Kavita").is_err());
    }

    #[test]
    fn clicking_add_folder_sends_add_folder_pressed_message() {
        let library = Library {
            items: AsyncModel::Loaded(vec![]),
            integrations: AsyncModel::Loaded(vec![]),
        };

        let mut ui = simulator(library.view());
        let _ = ui
            .click("Add Folder")
            .expect("Add Folder button should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(messages, vec![Message::AddFolderPressed]);
    }

    #[test]
    fn update_libraries_loaded_stores_result_and_returns_no_action() {
        let mut library = Library::default();

        let action = library.update(Message::LibrariesLoaded(Ok(vec![library_model(
            "My Library",
        )])));

        assert!(matches!(action, Action::None));
        assert!(matches!(library.items, AsyncModel::Loaded(ref libs) if libs.len() == 1));
    }

    #[test]
    fn update_libraries_loaded_stores_error_and_returns_no_action() {
        let mut library = Library::default();

        let action = library.update(Message::LibrariesLoaded(Err("boom".to_string())));

        assert!(matches!(action, Action::None));
        assert!(matches!(library.items, AsyncModel::Error(ref error) if error == "boom"));
    }

    #[test]
    fn update_integrations_loaded_stores_result_and_returns_no_action() {
        let mut library = Library::default();

        let action = library.update(Message::IntegrationsLoaded(Ok(vec![integration_model(
            "integration-id",
            "My Kavita",
        )])));

        assert!(matches!(action, Action::None));
        assert!(
            matches!(library.integrations, AsyncModel::Loaded(ref integrations) if integrations.len() == 1)
        );
    }

    #[test]
    fn update_add_folder_pressed_runs_a_task() {
        let mut library = Library::default();

        let action = library.update(Message::AddFolderPressed);

        assert!(matches!(action, Action::Run(_)));
    }

    #[test]
    fn update_folder_picked_none_returns_no_action() {
        let mut library = Library::default();

        let action = library.update(Message::FolderPicked(None));

        assert!(matches!(action, Action::None));
    }

    #[test]
    fn update_folder_picked_some_runs_a_task() {
        let mut library = Library::default();

        let action = library.update(Message::FolderPicked(Some(PathBuf::from("/tmp/comics"))));

        assert!(matches!(action, Action::Run(_)));
    }

    #[test]
    fn update_library_created_ok_reloads_libraries() {
        let mut library = Library::default();

        let action = library.update(Message::LibraryCreated(Ok(library_model("My Library"))));

        assert!(matches!(action, Action::Run(_)));
    }

    #[test]
    fn update_library_created_err_returns_no_action() {
        let mut library = Library::default();

        let action = library.update(Message::LibraryCreated(Err("boom".to_string())));

        assert!(matches!(action, Action::None));
    }

    #[test]
    fn update_browse_remote_library_returns_browse_remote_library_action() {
        let mut library = Library::default();

        let action = library.update(Message::BrowseRemoteLibrary("external-id".to_string()));

        assert!(matches!(action, Action::BrowseRemoteLibrary(id) if id == "external-id"));
    }

    #[test]
    fn update_browse_library_returns_browse_library_action() {
        let mut library = Library::default();

        let action = library.update(Message::BrowseLibrary("library-id".to_string()));

        assert!(matches!(action, Action::BrowseLibrary(id) if id == "library-id"));
    }

    #[test]
    fn clicking_browse_on_a_remote_library_sends_browse_remote_library_message() {
        let library = Library {
            items: AsyncModel::Loaded(vec![library_model_with_remote_id(
                "My Library",
                "external-id",
            )]),
            integrations: AsyncModel::Loaded(vec![]),
        };

        let mut ui = simulator(library.view());
        let _ = ui.click("Browse").expect("Browse button should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(
            messages,
            vec![Message::BrowseRemoteLibrary("external-id".to_string())]
        );
    }

    #[test]
    fn clicking_browse_on_a_local_library_sends_browse_library_message() {
        let library = Library {
            items: AsyncModel::Loaded(vec![library_model("My Library")]),
            integrations: AsyncModel::Loaded(vec![]),
        };

        let mut ui = simulator(library.view());
        let _ = ui.click("Browse").expect("Browse button should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(messages, vec![Message::BrowseLibrary("id".to_string())]);
    }
}
