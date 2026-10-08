use iced::{Element, Task};

use crate::models::{AsyncModel, integration::IntegrationType, library_item::LibraryItem};
use crate::providers::kavita::provider::KavitaProvider;
use crate::repository::{integration as integration_repository, library as library_repository};

#[derive(Debug)]
pub struct Browse {
    items: AsyncModel<Vec<LibraryItem>, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    ItemsLoaded(Result<Vec<LibraryItem>, String>),
    BackPressed,
}

#[derive(Debug)]
pub enum Action {
    None,
    BackPressed,
}

impl Browse {
    pub fn new_remote(external_id: String) -> (Self, Task<Message>) {
        (
            Self {
                items: AsyncModel::Loading,
            },
            Task::perform(
                crate::runtime::on_tokio(fetch_remote_items(external_id)),
                Message::ItemsLoaded,
            ),
        )
    }

    /// Browsing local folders isn't implemented yet; there's no provider to
    /// query, so this just shows an empty list.
    pub fn new_local(_library_id: String) -> (Self, Task<Message>) {
        (
            Self {
                items: AsyncModel::Loaded(Vec::new()),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ItemsLoaded(result) => {
                self.items = result.into();
                Action::None
            }

            Message::BackPressed => Action::BackPressed,
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        view_helper::view(&self.items)
    }
}

async fn fetch_remote_items(external_id: String) -> Result<Vec<LibraryItem>, String> {
    let library = library_repository::find_by_external_id(&external_id).await?;
    let integration_id = library
        .integration_id
        .ok_or_else(|| "library has no associated integration".to_string())?;
    let integration = integration_repository::find(&integration_id).await?;

    match integration.integration_type {
        IntegrationType::Kavita => {
            let url = integration
                .url
                .ok_or_else(|| "integration is missing a url".to_string())?;
            let api_key = integration
                .api_key
                .ok_or_else(|| "integration is missing an api key".to_string())?;

            let provider = KavitaProvider::authenticate(url, api_key).await?;
            provider
                .list_series_of_library(external_id, None, None)
                .await
        }
        _ => Err("browsing is not supported for this integration type yet".to_string()),
    }
}

mod view_helper {
    use iced::{
        Background, Border, Center, Element,
        Length::Fill,
        Theme,
        widget::{button, column, container, grid, row, text},
    };

    use crate::models::{AsyncModel, library_item::LibraryItem};

    pub fn view<'a>(
        items: &'a AsyncModel<Vec<LibraryItem>, String>,
    ) -> Element<'a, super::Message> {
        column![top_bar(), body(items)]
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn top_bar<'a>() -> Element<'a, super::Message> {
        container(button("Back").on_press(super::Message::BackPressed))
            .padding(20)
            .into()
    }

    fn body<'a>(items: &'a AsyncModel<Vec<LibraryItem>, String>) -> Element<'a, super::Message> {
        match items {
            AsyncModel::Loaded(items) if items.is_empty() => empty_state(),
            AsyncModel::Loaded(items) => items_grid(items),
            AsyncModel::Error(error) => error_state(error),
            AsyncModel::NotLoaded | AsyncModel::Loading => container(row![]).into(),
        }
    }

    fn items_grid<'a>(items: &'a [LibraryItem]) -> Element<'a, super::Message> {
        let cards = items.iter().map(item_card);

        container(
            grid(cards)
                .fluid(160.0)
                .spacing(16.0)
                .height(grid::aspect_ratio(2.0, 3.0)),
        )
        .width(Fill)
        .padding(20)
        .into()
    }

    fn item_card<'a>(item: &'a LibraryItem) -> Element<'a, super::Message> {
        column![
            thumbnail(),
            text(&item.name).size(13),
            pages_label(item.pages)
        ]
        .width(Fill)
        .spacing(4)
        .align_x(Center)
        .into()
    }

    fn thumbnail<'a>() -> Element<'a, super::Message> {
        container(row![])
            .width(Fill)
            .height(Fill)
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

    fn pages_label<'a>(pages: Option<u16>) -> Element<'a, super::Message> {
        match pages {
            Some(pages) => text(format!("{pages} pages"))
                .size(11)
                .style(text::secondary)
                .into(),
            None => row![].into(),
        }
    }

    fn empty_state<'a>() -> Element<'a, super::Message> {
        container(text("Nenhum item encontrado nesta biblioteca"))
            .center(Fill)
            .into()
    }

    fn error_state<'a>(error: &'a str) -> Element<'a, super::Message> {
        container(text(error)).center(Fill).into()
    }
}

#[cfg(test)]
mod tests {
    use iced_test::simulator;

    use super::*;

    fn library_item(name: &str, pages: Option<u16>) -> LibraryItem {
        LibraryItem {
            name: name.to_string(),
            cover: String::new(),
            library_id: "1".to_string(),
            pages,
        }
    }

    #[test]
    fn new_local_loads_an_empty_list_without_a_task() {
        let (browse, _task) = Browse::new_local("library-id".to_string());

        assert!(matches!(browse.items, AsyncModel::Loaded(ref items) if items.is_empty()));
    }

    #[test]
    fn update_items_loaded_stores_result_and_returns_no_action() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
        };

        let action = browse.update(Message::ItemsLoaded(Ok(vec![library_item(
            "One Piece",
            Some(42),
        )])));

        assert!(matches!(action, Action::None));
        assert!(matches!(browse.items, AsyncModel::Loaded(ref items) if items.len() == 1));
    }

    #[test]
    fn update_items_loaded_stores_error_and_returns_no_action() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
        };

        let action = browse.update(Message::ItemsLoaded(Err("boom".to_string())));

        assert!(matches!(action, Action::None));
        assert!(matches!(browse.items, AsyncModel::Error(ref error) if error == "boom"));
    }

    #[test]
    fn update_back_pressed_returns_back_pressed_action() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
        };

        let action = browse.update(Message::BackPressed);

        assert!(matches!(action, Action::BackPressed));
    }

    #[test]
    fn view_renders_item_name_and_pages_when_loaded() {
        let browse = Browse {
            items: AsyncModel::Loaded(vec![library_item("One Piece", Some(42))]),
        };

        let mut ui = simulator(browse.view());

        assert!(ui.find("One Piece").is_ok());
        assert!(ui.find("42 pages").is_ok());
    }

    #[test]
    fn view_does_not_render_pages_when_absent() {
        let browse = Browse {
            items: AsyncModel::Loaded(vec![library_item("One Piece", None)]),
        };

        let mut ui = simulator(browse.view());

        assert!(ui.find("One Piece").is_ok());
        assert!(ui.find("pages").is_err());
    }

    #[test]
    fn view_renders_error_message_when_loading_failed() {
        let browse = Browse {
            items: AsyncModel::Error("boom".to_string()),
        };

        let mut ui = simulator(browse.view());

        assert!(ui.find("boom").is_ok());
    }

    #[test]
    fn clicking_back_sends_back_pressed_message() {
        let browse = Browse {
            items: AsyncModel::Loading,
        };

        let mut ui = simulator(browse.view());
        let _ = ui.click("Back").expect("Back button should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(messages, vec![Message::BackPressed]);
    }

    #[test]
    fn view_renders_items_with_a_visible_non_collapsed_size() {
        let browse = Browse {
            items: AsyncModel::Loaded(vec![library_item("One Piece", Some(42))]),
        };

        let mut ui = simulator(browse.view());
        let bounds = ui
            .find("One Piece")
            .expect("item name should be found")
            .visible_bounds()
            .expect("item name should be visible");

        assert!(bounds.height > 10.0, "height was {}", bounds.height);
    }
}
