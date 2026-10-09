use iced::advanced::widget::Id as WidgetId;
use iced::{Element, Task};

use crate::models::{AsyncModel, integration::IntegrationType, library_item::LibraryItem};
use crate::providers::kavita::provider::KavitaProvider;
use crate::repository::{integration as integration_repository, library as library_repository};

/// Number of items fetched per page, and the unit eviction operates on:
/// pages are dropped from (or re-fetched back into) memory as a whole,
/// rather than item-by-item.
const PAGE_SIZE: u32 = 20;

/// How close to the bottom of the scrollable (as a fraction of its scrollable
/// range, 0.0 = top, 1.0 = bottom) the user must be before the next page is
/// requested.
const LOAD_NEXT_PAGE_THRESHOLD: f32 = 0.8;

/// How many of the most recently loaded pages to keep fully populated in
/// memory (4 pages * PAGE_SIZE = 80 items). Older pages are evicted, but
/// only once this many pages have been loaded ahead of them — eviction
/// tracks loading progress, not the live scroll position, so scrolling back
/// up by a page or two never shows placeholders.
const KEEP_PAGES_LOADED: usize = 4;

/// Identifies the items scrollable so it can be measured after a page loads,
/// to tell whether the loaded items actually overflow the viewport (see
/// [`measure_items_scrollable`]).
const ITEMS_SCROLLABLE_ID: &str = "browse-items";

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
    /// 0-based page indices currently being re-fetched after eviction (see
    /// `request_page_refetch`), keyed separately from `loading_next` since
    /// several evicted pages can be scrolled through in quick succession.
    refetching_pages: std::collections::HashSet<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    ItemsLoaded(Result<Vec<LibraryItem>, String>),
    NextPageLoaded(Result<Vec<LibraryItem>, String>),
    PageRefetched(usize, Result<Vec<LibraryItem>, String>),
    Scrolled(f32),
    ContentOverflowChecked(bool),
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
                    refetching_pages: std::collections::HashSet::new(),
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

                let loaded = result.is_ok();
                self.items = result.into();

                if loaded {
                    Action::Run(measure_items_scrollable())
                } else {
                    Action::None
                }
            }

            Message::NextPageLoaded(Ok(new_items)) => {
                let mut last_loaded_page = None;

                if let Some(pagination) = &mut self.pagination {
                    pagination.loading_next = false;
                    pagination.has_more = new_items.len() as u32 == pagination.page_size;
                    pagination.next_page += 1;
                    last_loaded_page = Some((pagination.next_page as usize).saturating_sub(2));
                }

                if let AsyncModel::Loaded(items) = &mut self.items {
                    items.extend(new_items.into_iter().map(Some));

                    // Forward loading only ever happens once the user has
                    // scrolled near the bottom, so by the time a page this
                    // far ahead loads, pages this far behind are safely
                    // off-screen.
                    if let Some(last_loaded_page) = last_loaded_page {
                        let range = eviction_range(
                            items.len(),
                            PAGE_SIZE,
                            last_loaded_page,
                            KEEP_PAGES_LOADED,
                        );
                        for slot in &mut items[range] {
                            *slot = None;
                        }
                    }
                }

                Action::Run(measure_items_scrollable())
            }

            Message::NextPageLoaded(Err(_)) => {
                if let Some(pagination) = &mut self.pagination {
                    pagination.loading_next = false;
                }

                Action::None
            }

            Message::Scrolled(relative_offset_y) => {
                let mut page_needing_refetch = None;

                if let AsyncModel::Loaded(items) = &self.items
                    && let Some(current_page) =
                        current_page_index(items.len(), PAGE_SIZE, relative_offset_y)
                {
                    let start = current_page * PAGE_SIZE as usize;
                    let end = (start + PAGE_SIZE as usize).min(items.len());

                    if items[start..end].iter().any(Option::is_none) {
                        page_needing_refetch = Some(current_page);
                    }
                }

                if let Some(page_index) = page_needing_refetch
                    && let Some(action) = self.request_page_refetch(page_index)
                {
                    return action;
                }

                if !is_near_bottom(relative_offset_y, LOAD_NEXT_PAGE_THRESHOLD) {
                    return Action::None;
                }

                self.request_next_page().unwrap_or(Action::None)
            }

            Message::PageRefetched(page_index, Ok(refetched_items)) => {
                if let Some(pagination) = &mut self.pagination {
                    pagination.refetching_pages.remove(&page_index);
                }

                if let AsyncModel::Loaded(items) = &mut self.items {
                    let start = page_index * PAGE_SIZE as usize;

                    for (offset, item) in refetched_items.into_iter().enumerate() {
                        if let Some(slot) = items.get_mut(start + offset) {
                            *slot = Some(item);
                        }
                    }
                }

                Action::None
            }

            Message::PageRefetched(page_index, Err(_)) => {
                if let Some(pagination) = &mut self.pagination {
                    pagination.refetching_pages.remove(&page_index);
                }

                Action::None
            }

            // The items scrollable doesn't overflow its viewport, so a
            // scroll gesture can never happen on its own: eagerly request
            // another page, as if the user had scrolled to the bottom. This
            // repeats (via the Action::Run chain from NextPageLoaded) until
            // either enough items are loaded to overflow the viewport, or
            // the library is exhausted.
            Message::ContentOverflowChecked(overflows) => {
                if overflows {
                    return Action::None;
                }

                self.request_next_page().unwrap_or(Action::None)
            }

            Message::BackPressed => Action::BackPressed,
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        view_helper::view(&self.items)
    }

    /// Requests the next page, if pagination is set up for it, there isn't
    /// already a request in flight, and there's more to fetch.
    fn request_next_page(&mut self) -> Option<Action> {
        let pagination = self.pagination.as_mut()?;

        if pagination.loading_next || !pagination.has_more {
            return None;
        }

        pagination.loading_next = true;

        let external_id = pagination.external_id.clone();
        let page_number = pagination.next_page;
        let page_size = pagination.page_size;

        Some(Action::Run(Task::perform(
            crate::runtime::on_tokio(fetch_remote_items_page(external_id, page_number, page_size)),
            Message::NextPageLoaded,
        )))
    }

    /// Re-fetches a single (0-based) page that was previously evicted, if it
    /// isn't already being re-fetched.
    fn request_page_refetch(&mut self, page_index: usize) -> Option<Action> {
        let pagination = self.pagination.as_mut()?;

        if pagination.refetching_pages.contains(&page_index) {
            return None;
        }

        pagination.refetching_pages.insert(page_index);

        let external_id = pagination.external_id.clone();
        let page_number = page_index as u32 + 1;
        let page_size = pagination.page_size;

        Some(Action::Run(Task::perform(
            crate::runtime::on_tokio(fetch_remote_items_page(external_id, page_number, page_size)),
            move |result| Message::PageRefetched(page_index, result),
        )))
    }
}

/// Returns the (flat, page-aligned) index range of items old enough to be
/// evicted from memory: everything before the last `keep_pages_loaded`
/// pages, relative to `last_loaded_page` (the most recently loaded page).
fn eviction_range(
    total_items: usize,
    page_size: u32,
    last_loaded_page: usize,
    keep_pages_loaded: usize,
) -> std::ops::Range<usize> {
    let evict_before_page = last_loaded_page.saturating_sub(keep_pages_loaded.saturating_sub(1));

    0..(evict_before_page * page_size as usize).min(total_items)
}

/// Approximates the (0-based) page the user is currently scrolled to, from
/// `relative_offset_y` (the scrollable's vertical offset as a fraction of
/// its scrollable range) assuming pages contribute roughly equal height to
/// the grid, since iced doesn't expose which individual items are actually
/// on-screen.
fn current_page_index(total_items: usize, page_size: u32, relative_offset_y: f32) -> Option<usize> {
    if total_items == 0 || page_size == 0 {
        return None;
    }

    let total_pages = total_items.div_ceil(page_size as usize);
    let relative_offset_y = relative_offset_y.clamp(0.0, 1.0);

    Some(((relative_offset_y * total_pages as f32) as usize).min(total_pages - 1))
}

fn is_near_bottom(relative_offset_y: f32, threshold: f32) -> bool {
    relative_offset_y >= threshold
}

/// Builds a task that measures the items scrollable's viewport against its
/// content and reports whether the content overflows it (i.e. whether
/// there's anything to scroll at all).
fn measure_items_scrollable() -> Task<Message> {
    iced::advanced::widget::operate(viewport_probe::overflow_probe(WidgetId::new(
        ITEMS_SCROLLABLE_ID,
    )))
    .map(Message::ContentOverflowChecked)
}

/// A widget [`Operation`](iced::advanced::widget::Operation) that finds a
/// scrollable by [`Id`](WidgetId) and reports whether its content overflows
/// its viewport, without needing to track viewport state across updates.
mod viewport_probe {
    use iced::Rectangle;
    use iced::advanced::widget::operation::{Outcome, Scrollable};
    use iced::advanced::widget::{Id, Operation};

    struct OverflowProbe {
        target: Id,
        overflows: Option<bool>,
    }

    impl Operation<bool> for OverflowProbe {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<bool>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: iced::Vector,
            _state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                self.overflows = Some(content_bounds.height > bounds.height);
            }
        }

        fn finish(&self) -> Outcome<bool> {
            match self.overflows {
                Some(overflows) => Outcome::Some(overflows),
                None => Outcome::None,
            }
        }
    }

    pub fn overflow_probe(target: Id) -> impl Operation<bool> {
        OverflowProbe {
            target,
            overflows: None,
        }
    }

    #[cfg(test)]
    mod tests {
        use iced::Size;
        use iced::advanced::widget::operation::scrollable::{AbsoluteOffset, RelativeOffset};

        use super::*;

        struct NoopScrollableState;

        impl Scrollable for NoopScrollableState {
            fn snap_to(&mut self, _offset: RelativeOffset<Option<f32>>) {}
            fn scroll_to(&mut self, _offset: AbsoluteOffset<Option<f32>>) {}
            fn scroll_by(
                &mut self,
                _offset: AbsoluteOffset,
                _bounds: Rectangle,
                _content_bounds: Rectangle,
            ) {
            }
        }

        fn rectangle(height: f32) -> Rectangle {
            Rectangle::new(iced::Point::ORIGIN, Size::new(100.0, height))
        }

        #[test]
        fn reports_overflow_when_content_is_taller_than_the_viewport() {
            let target = Id::new("items");
            let mut probe = overflow_probe(target.clone());

            probe.scrollable(
                Some(&target),
                rectangle(400.0),
                rectangle(1200.0),
                iced::Vector::default(),
                &mut NoopScrollableState,
            );

            assert!(matches!(probe.finish(), Outcome::Some(true)));
        }

        #[test]
        fn reports_no_overflow_when_content_fits_the_viewport() {
            let target = Id::new("items");
            let mut probe = overflow_probe(target.clone());

            probe.scrollable(
                Some(&target),
                rectangle(800.0),
                rectangle(400.0),
                iced::Vector::default(),
                &mut NoopScrollableState,
            );

            assert!(matches!(probe.finish(), Outcome::Some(false)));
        }

        #[test]
        fn ignores_scrollables_with_a_different_id() {
            let target = Id::new("items");
            let other = Id::new("something-else");
            let mut probe = overflow_probe(target);

            probe.scrollable(
                Some(&other),
                rectangle(400.0),
                rectangle(1200.0),
                iced::Vector::default(),
                &mut NoopScrollableState,
            );

            assert!(matches!(probe.finish(), Outcome::None));
        }
    }
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
            .id(iced::advanced::widget::Id::new(super::ITEMS_SCROLLABLE_ID))
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
            refetching_pages: std::collections::HashSet::new(),
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
    fn update_items_loaded_stores_result_and_checks_the_viewport() {
        let mut browse = Browse {
            items: AsyncModel::Loading,
            pagination: Some(pagination(true, false)),
        };

        let action = browse.update(Message::ItemsLoaded(Ok(vec![library_item(
            "One Piece",
            Some(42),
        )])));

        assert!(matches!(action, Action::Run(_)));
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

        let action = browse.update(Message::NextPageLoaded(Ok(vec![library_item(
            "Berserk", None,
        )])));

        assert!(matches!(action, Action::Run(_)));

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
    fn content_overflow_checked_true_does_not_request_a_page() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: Some(pagination(true, false)),
        };

        let action = browse.update(Message::ContentOverflowChecked(true));

        assert!(matches!(action, Action::None));
        assert!(!browse.pagination.unwrap().loading_next);
    }

    #[test]
    fn content_overflow_checked_false_requests_the_next_page_to_fill_the_viewport() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: Some(pagination(true, false)),
        };

        let action = browse.update(Message::ContentOverflowChecked(false));

        assert!(matches!(action, Action::Run(_)));
        assert!(browse.pagination.unwrap().loading_next);
    }

    #[test]
    fn content_overflow_checked_false_without_more_pages_does_nothing() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: Some(pagination(false, false)),
        };

        let action = browse.update(Message::ContentOverflowChecked(false));

        assert!(matches!(action, Action::None));
    }

    #[test]
    fn content_overflow_checked_false_while_already_loading_does_nothing() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(vec![Some(library_item("One Piece", None))]),
            pagination: Some(pagination(true, true)),
        };

        let action = browse.update(Message::ContentOverflowChecked(false));

        assert!(matches!(action, Action::None));
    }

    #[test]
    fn content_overflow_checked_false_without_pagination_does_nothing() {
        let mut browse = Browse {
            items: AsyncModel::Loaded(Vec::new()),
            pagination: None,
        };

        let action = browse.update(Message::ContentOverflowChecked(false));

        assert!(matches!(action, Action::None));
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
    fn next_page_loaded_evicts_pages_outside_the_keep_window() {
        // KEEP_PAGES_LOADED pages are already loaded (indices 0..KEEP_PAGES_LOADED);
        // loading one more page should push page 0 out of the window.
        let items: Vec<_> = (0..(PAGE_SIZE * KEEP_PAGES_LOADED as u32))
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();

        let mut pagination_state = pagination(true, false);
        pagination_state.next_page = KEEP_PAGES_LOADED as u32 + 1;

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination_state),
        };

        let new_page: Vec<_> = (0..PAGE_SIZE)
            .map(|i| library_item(&format!("New {i}"), None))
            .collect();
        browse.update(Message::NextPageLoaded(Ok(new_page)));

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
    fn next_page_loaded_keeps_everything_within_the_keep_window() {
        let items: Vec<_> = (0..PAGE_SIZE)
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination(true, false)),
        };

        let new_page: Vec<_> = (0..PAGE_SIZE)
            .map(|i| library_item(&format!("New {i}"), None))
            .collect();
        browse.update(Message::NextPageLoaded(Ok(new_page)));

        let AsyncModel::Loaded(items) = &browse.items else {
            panic!("items should be loaded");
        };

        assert!(items.iter().all(Option::is_some));
    }

    #[test]
    fn scrolling_does_not_evict_anything_on_its_own() {
        let items: Vec<_> = (0..(PAGE_SIZE * 3))
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination(false, false)),
        };

        browse.update(Message::Scrolled(0.99));

        let AsyncModel::Loaded(items) = &browse.items else {
            panic!("items should be loaded");
        };

        assert!(items.iter().all(Option::is_some));
    }

    #[test]
    fn eviction_range_is_empty_for_an_empty_list() {
        assert_eq!(eviction_range(0, PAGE_SIZE, 10, KEEP_PAGES_LOADED), 0..0);
    }

    #[test]
    fn scrolling_back_onto_an_evicted_page_refetches_it() {
        let mut items: Vec<_> = (0..(PAGE_SIZE * 3))
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();
        for slot in &mut items[0..PAGE_SIZE as usize] {
            *slot = None;
        }

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination(false, false)),
        };

        // Scroll back up onto the evicted page 0.
        let action = browse.update(Message::Scrolled(0.0));

        assert!(matches!(action, Action::Run(_)));
        assert!(browse.pagination.unwrap().refetching_pages.contains(&0));
    }

    #[test]
    fn scrolling_onto_a_page_that_is_not_evicted_does_not_refetch() {
        let items: Vec<_> = (0..(PAGE_SIZE * 3))
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination(false, false)),
        };

        let action = browse.update(Message::Scrolled(0.0));

        assert!(matches!(action, Action::None));
        assert!(browse.pagination.unwrap().refetching_pages.is_empty());
    }

    #[test]
    fn scrolling_onto_an_evicted_page_already_being_refetched_does_not_request_again() {
        let mut items: Vec<_> = (0..(PAGE_SIZE * 3))
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();
        for slot in &mut items[0..PAGE_SIZE as usize] {
            *slot = None;
        }

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination(false, false)),
        };

        browse.update(Message::Scrolled(0.0));
        let action = browse.update(Message::Scrolled(0.01));

        assert!(matches!(action, Action::None));
    }

    #[test]
    fn page_refetched_ok_splices_items_back_at_the_right_position_and_clears_the_flag() {
        let mut items: Vec<_> = (0..(PAGE_SIZE * 2))
            .map(|i| Some(library_item(&format!("Item {i}"), None)))
            .collect();
        for slot in &mut items[0..PAGE_SIZE as usize] {
            *slot = None;
        }

        let mut pagination_state = pagination(false, false);
        pagination_state.refetching_pages.insert(0);

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination_state),
        };

        let refetched: Vec<_> = (0..PAGE_SIZE)
            .map(|i| library_item(&format!("Refetched {i}"), None))
            .collect();

        let action = browse.update(Message::PageRefetched(0, Ok(refetched)));

        assert!(matches!(action, Action::None));
        assert!(!browse.pagination.unwrap().refetching_pages.contains(&0));

        let AsyncModel::Loaded(items) = &browse.items else {
            panic!("items should be loaded");
        };
        assert_eq!(items[0].as_ref().unwrap().name, "Refetched 0");
        assert!(items[0..PAGE_SIZE as usize].iter().all(Option::is_some));
        assert!(items[PAGE_SIZE as usize..].iter().all(|item| {
            item.as_ref()
                .is_some_and(|item| item.name.starts_with("Item"))
        }));
    }

    #[test]
    fn page_refetched_err_clears_the_flag_without_touching_items() {
        let items = vec![None, Some(library_item("One Piece", None))];

        let mut pagination_state = pagination(false, false);
        pagination_state.refetching_pages.insert(0);

        let mut browse = Browse {
            items: AsyncModel::Loaded(items),
            pagination: Some(pagination_state),
        };

        let action = browse.update(Message::PageRefetched(0, Err("boom".to_string())));

        assert!(matches!(action, Action::None));
        assert!(!browse.pagination.unwrap().refetching_pages.contains(&0));
        assert!(matches!(browse.items, AsyncModel::Loaded(ref items) if items[0].is_none()));
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
