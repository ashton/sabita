use std::path::PathBuf;

use iced::{
    Element, Task,
    widget::{container, row},
};

use crate::models::{AsyncModel, library::Library as LibraryModel};
use crate::repository::library as library_repository;

#[derive(Debug, Default)]
pub struct Library {
    items: AsyncModel<Vec<LibraryModel>, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    LibrariesLoaded(Result<Vec<LibraryModel>, String>),
    AddFolderPressed,
    FolderPicked(Option<PathBuf>),
    LibraryCreated(Result<LibraryModel, String>),
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
}

impl Library {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                items: AsyncModel::Loading,
            },
            Task::perform(library_repository::all(), Message::LibrariesLoaded),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::LibrariesLoaded(result) => {
                self.items = result.into();
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
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        match &self.items {
            AsyncModel::Loaded(libraries) => view_helper::list(libraries),
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
        Element,
        widget::{button, column, container, row, text},
    };

    use crate::models::library::Library;

    pub fn list<'a>(libraries: &'a [Library]) -> Element<'a, super::Message> {
        if libraries.is_empty() {
            return empty_list();
        }

        let items = row![];

        let items = items.extend(libraries.iter().map(|lib| text(&lib.name).into()));

        container(items).into()
    }

    fn empty_list<'a>() -> Element<'a, super::Message> {
        container(column![
        text!(
            "Você não tem nenhuma biblioteca cadastrada, adicione uma pasta ou uma integração com algum serviço"
        ),
        row![
            button("Add Folder").on_press(super::Message::AddFolderPressed),
            button("Add Connection")
        ]
        ]).into()
    }
}

#[cfg(test)]
mod tests {
    use iced_test::simulator;

    use super::*;

    fn library_model(name: &str) -> LibraryModel {
        LibraryModel {
            id: "id".to_string(),
            name: name.to_string(),
            external_id: None,
            folder: None,
            cover: None,
        }
    }

    #[test]
    fn view_renders_empty_container_while_not_loaded() {
        let library = Library {
            items: AsyncModel::Loading,
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("Add Folder").is_err());
    }

    #[test]
    fn view_renders_call_to_action_when_loaded_without_libraries() {
        let library = Library {
            items: AsyncModel::Loaded(vec![]),
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("Add Folder").is_ok());
        assert!(ui.find("Add Connection").is_ok());
    }

    #[test]
    fn view_renders_library_names_when_loaded() {
        let library = Library {
            items: AsyncModel::Loaded(vec![library_model("My Library")]),
        };

        let mut ui = simulator(library.view());

        assert!(ui.find("My Library").is_ok());
    }

    #[test]
    fn clicking_add_folder_sends_add_folder_pressed_message() {
        let library = Library {
            items: AsyncModel::Loaded(vec![]),
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
}
