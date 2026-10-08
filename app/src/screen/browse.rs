use iced::{Element, Task};

use crate::models::{AsyncModel, integration::IntegrationType, library_item::LibraryItem};
use crate::providers::kavita::provider::KavitaProvider;
use crate::repository::{integration as integration_repository, library as library_repository};

/// Number of items fetched per page, and the unit eviction operates on: once
/// the user has scrolled far enough past a page, its items are dropped from
/// memory as a whole page rather than item-by-item.
const PAGE_SIZE: u32 = 20;

/// How close to the bottom of the scrollable (as a fraction of its scrollable
/// range, 0.0 = top, 1.0 = bottom) the user must be before the next page is
/// requested.
const LOAD_NEXT_PAGE_THRESHOLD: f32 = 0.8;

/// How many pages "behind" the current scroll position to keep loaded, in
/// case the user scrolls back up. Pages older than this are evicted.
const KEEP_PAGES_BEHIND: usize = 1;

#[derive(Debug)]
pub struct Browse {
    items: AsyncModel<Vec<Option<LibraryItem>>, String>,
    pagination: Option<Pagination>,
}

#[derive(Debug)]
struct Pagination {
    external_id: String,
    next_page: u32,
    page_size: u32,
    has_more: bool,
    loading_next: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    ItemsLoaded(Result<Vec<LibraryItem>, String>),
    NextPageLoaded(Result<Vec<LibraryItem>, String>),
    Scrolled(f32),
    BackPressed,
}

#[derive(Debug)]
pub enum Action {
    None,
    Run(Task<Message>),
    BackPressed,
}

impl Browse {
    pub fn new_remote(external_id: String) -> (Self, Task<Message>) {
        (
            Self {
                items: AsyncModel::Loading,
                pagination: Some(Pagination {
                    external_id: external_id.clone(),
                    next_page: 2,
                    page_size: PAGE_SIZE,
                    has_more: true,
                    loading_next: false,
                }),
            },
            Task::perform(
                crate::runtime::on_tokio(fetch_remote_items_page(external_id, 1, PAGE_SIZE)),
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
                pagination: None,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ItemsLoaded(result) => {
                let result = result.map(|items| {
                    let has_more = items.len() as u32 == PAGE_SIZE;
                    if let Some(pagination) = &mut self.pagination {
                        pagination.has_more = has_more;
                    }
                    items.into_iter().map(Some).collect::<Vec<_>>()
                });

                self.items = result.into();
                Action::None
            }

            Message::NextPageLoaded(Ok(new_items)) => {
                if let Some(pagination) = &mut self.pagination {
                    pagination.loading_next = false;
                    pagination.has_more = new_items.len() as u32 == pagination.page_size;
                    pagination.next_page += 1;
                }

                if let AsyncModel::Loaded(items) = &mut self.items {
                    items.extend(new_items.into_iter().map(Some));
                }

                Action::None
            }

            Message::NextPageLoaded(Err(_)) => {
                if let Some(pagination) = &mut self.pagination {
                    pagination.loading_next = false;
                }

                Action::None
            }

            Message::Scrolled(relative_offset_y) => {
                if let AsyncModel::Loaded(items) = &mut self.items {
                    let range = eviction_range(
                        items.len(),
                        PAGE_SIZE,
                        relative_offset_y,
                        KEEP_PAGES_BEHIND,
                    );

                    for slot in &mut items[range] {
                        *slot = None;
                    }
                }

                let Some(pagination) = &mut self.pagination else {
                    return Action::None;
                };

                if !should_load_next_page(
                    relative_offset_y,
                    pagination.has_more,
                    pagination.loading_next,
                    LOAD_NEXT_PAGE_THRESHOLD,
                ) {
                    return Action::None;
                }

                pagination.loading_next = true;

                let external_id = pagination.external_id.clone();
                let page_number = pagination.next_page;
                let page_size = pagination.page_size;

                Action::Run(Task::perform(
                    crate::runtime::on_tokio(fetch_remote_items_page(
                        external_id,
                        page_number,
                        page_size,
                    )),
                    Message::NextPageLoaded,
                ))
            }

            Message::BackPressed => Action::BackPressed,
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        view_helper::view(&self.items)
    }
}

/// Returns the (flat, page-aligned) index range of items that are far enough
/// above the current scroll position to be evicted from memory.
///
/// The current page is approximated from `relative_offset_y` (the
/// scrollable's vertical offset as a fraction of its scrollable range)
/// assuming pages contribute roughly equal height to the grid, since iced
/// doesn't expose which individual items are actually on-screen.
fn eviction_range(
    total_items: usize,
    page_size: u32,
    relative_offset_y: f32,
    keep_pages_behind: usize,
) -> std::ops::Range<usize> {
    if total_items == 0 || page_size == 0 {
        return 0..0;
    }

    let page_size = page_size as usize;
    let total_pages = total_items.div_ceil(page_size);
    let relative_offset_y = relative_offset_y.clamp(0.0, 1.0);
    let current_page = ((relative_offset_y * total_pages as f32) as usize).min(total_pages - 1);
    let evict_before_page = current_page.saturating_sub(keep_pages_behind);

    0..(evict_before_page * page_size).min(total_items)
}

fn should_load_next_page(
    relative_offset_y: f32,
    has_more: bool,
    loading_next: bool,
    threshold: f32,
) -> bool {
    has_more && !loading_next && relative_offset_y >= threshold
}

async fn fetch_remote_items_page(
    external_id: String,
    page_number: u32,
    page_size: u32,
) -> Result<Vec<LibraryItem>, String> {
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
                .list_series_of_library(external_id, Some(page_number), Some(page_size))
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
        widget::{button, column, container, grid, row, scrollable, text},
    };

    use crate::models::{AsyncModel, library_item::LibraryItem};

    pub fn view<'a>(
        items: &'a AsyncModel<Vec<Option<LibraryItem>>, String>,
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

    fn body<'a>(
        items: &'a AsyncModel<Vec<Option<LibraryItem>>, String>,
    ) -> Element<'a, super::Message> {
        match items {
            AsyncModel::Loaded(items) if items.is_empty() => empty_state(),
            AsyncModel::Loaded(items) => items_scrollable(items),
            AsyncModel::Error(error) => error_state(error),
            AsyncModel::NotLoaded | AsyncModel::Loading => container(row![]).into(),
        }
    }

    fn items_scrollable<'a>(items: &'a [Option<LibraryItem>]) -> Element<'a, super::Message> {
        scrollable(items_grid(items))
            .width(Fill)
            .height(Fill)
            .on_scroll(|viewport| super::Message::Scrolled(viewport.relative_offset().y))
            .into()
    }

    fn items_grid<'a>(items: &'a [Option<LibraryItem>]) -> Element<'a, super::Message> {
        let cards = items.iter().map(|item| match item {
            Some(item) => item_card(item),
            None => evicted_card(),
        });

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

    /// Placeholder for an item whose data has been evicted from memory
    /// because it was scrolled far off-screen; keeps the grid's item count
    /// (and thus its scroll geometry) stable.
    fn evicted_card<'a>() -> Element<'a, super::Message> {
        container(thumbnail()).width(Fill).height(Fill).into()
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

    fn pagination(has_more: bool, loading_next: bool) -> Pagination {
        Pagination {
            external_id: "lib-1".to_string(),
            next_page: 2,
            page_size: PAGE_SIZE,
            has_more,
            loading_next,
        }
    }

    #[test]
    fn new_local_loads_an_empty_list_without_a_task() {
        let (browse, _task) = Browse::new_local("library-id".to_string());

        assert!(matches!(browse.items, AsyncModel::Loaded(ref items) if items.is_empty()));
        assert!(browse.pagination.is_none());
    }

    #[test]
    fn new_remote_starts_pagination_at_the_second_page() {
        let (browse, _task) = Browse::new_remote("external-id".to_string());

        let pagination = browse.pagination.expect("pagination should be set up");
        assert_eq!(pagination.next_page, 2);
        assert!(pagination.has_more);
        assert!(!pagination.loading_next);
    }

    #[test]
    fn update_items_loaded_stores_result_and_returns_no_action() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
            pagination: Some(pagination(true, false)),
        };

        let action = browse.update(Message::ItemsLoaded(Ok(vec![library_item(
            "One Piece",
            Some(42),
        )])));

        assert!(matches!(action, Action::None));
        assert!(matches!(browse.items, AsyncModel::Loaded(ref items) if items.len() == 1));
    }

    #[test]
    fn update_items_loaded_with_a_full_page_keeps_has_more_true() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
            pagination: Some(pagination(true, false)),
        };

        let full_page: Vec<_> = (0..PAGE_SIZE)
            .map(|i| library_item(&format!("Item {i}"), None))
            .collect();

        browse.update(Message::ItemsLoaded(Ok(full_page)));

        assert!(browse.pagination.unwrap().has_more);
    }

    #[test]
    fn update_items_loaded_with_a_short_page_clears_has_more() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
            pagination: Some(pagination(true, false)),
        };

        browse.update(Message::ItemsLoaded(Ok(vec![library_item(
            "Only one", None,
        )])));

        assert!(!browse.pagination.unwrap().has_more);
    }

    #[test]
    fn update_items_loaded_stores_error_and_returns_no_action() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
            pagination: Some(pagination(true, false)),
        };

        let action = browse.update(Message::ItemsLoaded(Err("boom".to_string())));

        assert!(matches!(action, Action::None));
        assert!(matches!(browse.items, AsyncModel::Error(ref error) if error == "boom"));
    }

    #[test]
    fn update_back_pressed_returns_back_pressed_action() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
            pagination: None,
        };

        let action = browse.update(Message::BackPressed);

        assert!(matches!(action, Action::BackPressed));
    }

    #[test]
    fn scrolling_near_the_bottom_requests_the_next_page() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(
                (0..PAGE_SIZE)
                    .map(|i| Some(library_item(&format!("Item {i}"), None)))
                    .collect(),
            ),
            pagination: Some(pagination(true, false)),
        };

        let action = browse.update(Message::Scrolled(0.9));

        assert!(matches!(action, Action::Run(_)));
        assert!(browse.pagination.unwrap().loading_next);
    }

    #[test]
    fn scrolling_away_from_the_bottom_does_not_request_a_page() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(
                (0..PAGE_SIZE)
                    .map(|i| Some(library_item(&format!("Item {i}"), None)))
                    .collect(),
            ),
            pagination: Some(pagination(true, false)),
        };

        let action = browse.update(Message::Scrolled(0.3));

        assert!(matches!(action, Action::None));
        assert!(!browse.pagination.unwrap().loading_next);
    }

    #[test]
    fn scrolling_near_the_bottom_while_a_page_is_already_loading_does_nothing() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: Some(pagination(true, true)),
        };

        let action = browse.update(Message::Scrolled(0.95));

        assert!(matches!(action, Action::None));
    }

    #[test]
    fn scrolling_near_the_bottom_without_more_pages_does_nothing() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: Some(pagination(false, false)),
        };

        let action = browse.update(Message::Scrolled(0.95));

        assert!(matches!(action, Action::None));
    }

    #[test]
    fn next_page_loaded_appends_items_and_advances_pagination() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: Some(pagination(true, true)),
        };

        browse.update(Message::NextPageLoaded(Ok(vec![library_item(
            "Berserk", None,
        )])));

        let AsyncModel::Loaded(items) = &browse.items else {
            panic!("items should be loaded");
        };
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].as_ref().unwrap().name, "Berserk");

        let pagination = browse.pagination.unwrap();
        assert_eq!(pagination.next_page, 3);
        assert!(!pagination.loading_next);
        assert!(!pagination.has_more);
    }

    #[test]
    fn next_page_loaded_error_resets_loading_flag_without_dropping_items() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: Some(pagination(true, true)),
        };

        browse.update(Message::NextPageLoaded(Err("boom".to_string())));

        assert!(matches!(browse.items, AsyncModel::Loaded(ref items) if items.len() == 1));
        assert!(!browse.pagination.unwrap().loading_next);
    }

    #[test]
    fn scrolling_evicts_items_from_pages_scrolled_past() {
        let items: Vec<_> = (0..(PAGE_SIZE * 3))
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination(false, false)),
        };

        // Scrolled almost to the very bottom (page index 2 of 3): with
        // KEEP_PAGES_BEHIND = 1, page 0 should be evicted but page 1 kept.
        browse.update(Message::Scrolled(0.99));

        let AsyncModel::Loaded(items) = &browse.items else {
            panic!("items should be loaded");
        };

        assert!(items[0..PAGE_SIZE as usize].iter().all(Option::is_none));
        assert!(
            items[PAGE_SIZE as usize..(PAGE_SIZE as usize * 2)]
                .iter()
                .all(Option::is_some)
        );
    }

    #[test]
    fn scrolling_near_the_top_does_not_evict_anything() {
        let items: Vec<_> = (0..(PAGE_SIZE * 3))
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination(false, false)),
        };

        browse.update(Message::Scrolled(0.05));

        let AsyncModel::Loaded(items) = &browse.items else {
            panic!("items should be loaded");
        };

        assert!(items.iter().all(Option::is_some));
    }

    #[test]
    fn eviction_range_is_empty_for_an_empty_list() {
        assert_eq!(eviction_range(0, PAGE_SIZE, 1.0, KEEP_PAGES_BEHIND), 0..0);
    }

    #[test]
    fn view_renders_item_name_and_pages_when_loaded() {
        let browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", Some(42)))]),
            pagination: None,
        };

        let mut ui = simulator(browse.view());

        assert!(ui.find("One Piece").is_ok());
        assert!(ui.find("42 pages").is_ok());
    }

    #[test]
    fn view_does_not_render_pages_when_absent() {
        let browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: None,
        };

        let mut ui = simulator(browse.view());

        assert!(ui.find("One Piece").is_ok());
        assert!(ui.find("pages").is_err());
    }

    #[test]
    fn view_renders_error_message_when_loading_failed() {
        let browse = Browse {
            items: AsyncModel::Error("boom".to_string()),
            pagination: None,
        };

        let mut ui = simulator(browse.view());

        assert!(ui.find("boom").is_ok());
    }

    #[test]
    fn clicking_back_sends_back_pressed_message() {
        let browse = Browse {
            items: AsyncModel::Loading,
            pagination: None,
        };

        let mut ui = simulator(browse.view());
        let _ = ui.click("Back").expect("Back button should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(messages, vec![Message::BackPressed]);
    }

    #[test]
    fn view_renders_items_with_a_visible_non_collapsed_size() {
        let browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", Some(42)))]),
            pagination: None,
        };

        let mut ui = simulator(browse.view());
        let bounds = ui
            .find("One Piece")
            .expect("item name should be found")
            .visible_bounds()
            .expect("item name should be visible");

        assert!(bounds.height > 10.0, "height was {}", bounds.height);
    }

    #[test]
    fn scrolling_past_the_bottom_of_a_tall_grid_requests_the_next_page() {
        let items: Vec<_> = (0..PAGE_SIZE)
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();

        let browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination(true, false)),
        };

        let mut ui = simulator(browse.view());
        ui.point_at(iced::Point::new(100.0, 300.0));
        ui.simulate([iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
            delta: iced::mouse::ScrollDelta::Pixels {
                x: 0.0,
                y: -100_000.0,
            },
        })]);

        let messages: Vec<_> = ui.into_messages().collect();

        assert!(
            messages.iter().any(
                |message| matches!(message, Message::Scrolled(y) if *y >= LOAD_NEXT_PAGE_THRESHOLD)
            ),
            "expected a Scrolled message near the bottom, got {messages:?}"
        );
    }
}
