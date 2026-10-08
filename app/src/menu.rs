use std::time::Duration;

use iced::{
    Alignment::Center,
    Background, Element,
    Length::Fill,
    Theme,
    widget::{Button, button, column, container, row, space, text},
};
use iced_anim::{Animated, Animation, Event, transition::Easing};
use iced_font_awesome::fa_icon_solid;

/// Sidebar width when only the icons are visible.
const COLLAPSED_WIDTH: f32 = 44.0;
/// Sidebar width when the item labels are visible too.
const EXPANDED_WIDTH: f32 = 150.0;
/// Width reserved for the icon, so labels line up across items.
const ICON_WIDTH: f32 = 24.0;
/// How long the collapse/expand transition takes.
const TRANSITION: Duration = Duration::from_millis(200);

pub struct MenuItem<Message> {
    icon: &'static str,
    title: &'static str,
    active: bool,
    message: Option<Message>,
}

pub struct Menu<Message> {
    /// Animated sidebar width. Its target doubles as the collapsed/expanded
    /// state, so the two can never get out of sync.
    width: Animated<f32>,
    pub top_items: Vec<MenuItem<Message>>,
    pub bottom_items: Vec<MenuItem<Message>>,
}

fn menu_icon<'a, Message: Clone + 'a>(
    icon: &'static str,
    title: Option<&'static str>,
    active: bool,
    on_press: Option<Message>,
) -> Button<'a, Message> {
    let mut button_contents = row![
        text!(""),
        container(fa_icon_solid(icon).size(15.0_f32)).center_x(ICON_WIDTH)
    ]
    .spacing(10)
    .align_y(Center);

    if let Some(title) = title {
        button_contents = button_contents.push(text(title));
    } else {
        button_contents = button_contents.push(text!(""));
    }

    button(button_contents)
        .padding(10)
        .width(Fill)
        .height(iced::Length::Shrink)
        .on_press_maybe(on_press)
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();

            let background = if active {
                Some(palette.primary.weak.color)
            } else if status == button::Status::Hovered {
                Some(palette.background.weak.color)
            } else {
                None
            };

            button::Style {
                background: background.map(Background::Color),
                text_color: palette.background.base.text,
                ..button::Style::default()
            }
        })
}

impl<Message> Menu<Message>
where
    Message: Clone,
{
    pub fn new(top_items: Vec<MenuItem<Message>>, bottom_items: Vec<MenuItem<Message>>) -> Self {
        Self {
            width: Animated::transition(
                COLLAPSED_WIDTH,
                Easing::EASE_IN_OUT.with_duration(TRANSITION),
            ),
            top_items,
            bottom_items,
        }
    }

    /// Whether the sidebar is collapsed, or on its way to being collapsed.
    pub fn is_collapsed(&self) -> bool {
        *self.width.target() == COLLAPSED_WIDTH
    }

    /// Starts the transition towards the collapsed or expanded width.
    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.width.set_target(if collapsed {
            COLLAPSED_WIDTH
        } else {
            EXPANDED_WIDTH
        });
    }

    /// Advances the width transition. Feed it the events emitted by [`Menu::view`].
    pub fn animate(&mut self, event: Event<f32>) {
        self.width.update(event);
    }

    pub fn is_animating(&self) -> bool {
        self.width.is_animating()
    }

    pub fn set_active(&mut self, message: &Message) {
        let target = std::mem::discriminant(message);

        for item in self
            .top_items
            .iter_mut()
            .chain(self.bottom_items.iter_mut())
        {
            item.active = item
                .message
                .as_ref()
                .is_some_and(|item_message| std::mem::discriminant(item_message) == target);
        }
    }

    /// Horizontal space the sidebar currently takes, mid-transition included.
    pub fn width(&self) -> f32 {
        *self.width.value()
    }

    /// Builds the sidebar. `on_animate` is called with the ticks that drive the
    /// width transition; feed them back through [`Menu::animate`].
    pub fn view(&self, on_animate: impl Fn(Event<f32>) -> Message + 'static) -> Element<'_, Message>
    where
        Message: 'static,
    {
        // Labels stay rendered for as long as the sidebar is wider than its
        // collapsed width, so they slide in and out with it instead of
        // popping once the transition ends.
        let collapsed = self.width() <= COLLAPSED_WIDTH;
        let mut children: Vec<Element<'_, Message>> = vec![];

        children.extend(self.top_items.iter().map(|item: &MenuItem<Message>| {
            menu_icon(
                item.icon,
                item.title(collapsed),
                item.active,
                item.message.clone(),
            )
            .into()
        }));

        children.push(space::vertical().into());
        children.extend(self.bottom_items.iter().map(|item: &MenuItem<Message>| {
            menu_icon(
                item.icon,
                item.title(collapsed),
                item.active,
                item.message.clone(),
            )
            .into()
        }));

        let sidebar = container(column(children).height(Fill))
            .width(self.width())
            .height(Fill)
            .clip(true);

        Animation::new(&self.width, sidebar)
            .on_update(on_animate)
            .into()
    }
}

impl<Message> MenuItem<Message> {
    pub fn new(
        icon: &'static str,
        title: &'static str,
        active: bool,
        message: Option<Message>,
    ) -> Self {
        Self {
            icon,
            title,
            active,
            message,
        }
    }

    pub fn set_message(&mut self, message: Option<Message>) {
        self.message = message;
    }

    pub fn message(&self) -> Option<&Message> {
        self.message.as_ref()
    }

    pub fn title(&self, collapsed: bool) -> Option<&'static str> {
        if collapsed {
            return None;
        }

        Some(self.title)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use iced_test::simulator;

    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestMessage {
        ToggleMenu,
        OpenHome,
        #[allow(dead_code)]
        Animate(Event<f32>),
    }

    fn menu() -> Menu<TestMessage> {
        Menu::new(
            vec![
                MenuItem::new("bars", "", false, Some(TestMessage::ToggleMenu)),
                MenuItem::new("house", "Home", true, Some(TestMessage::OpenHome)),
                MenuItem::new("magnifying_glass", "Search", false, None),
            ],
            vec![MenuItem::new("gear", "Settings", false, None)],
        )
    }

    /// Jumps straight to the end of the transition, skipping the ticks.
    fn settle(menu: &mut Menu<TestMessage>) {
        menu.animate(Event::Settle);
    }

    #[test]
    fn width_is_narrow_when_collapsed() {
        let menu = menu();

        assert!(menu.is_collapsed());
        assert_eq!(menu.width(), COLLAPSED_WIDTH);
    }

    #[test]
    fn width_grows_when_expanded() {
        let mut menu = menu();
        menu.set_collapsed(false);
        settle(&mut menu);

        assert_eq!(menu.width(), EXPANDED_WIDTH);
        assert!(menu.width() > COLLAPSED_WIDTH);
    }

    #[test]
    fn expanding_animates_instead_of_jumping_to_the_target_width() {
        let mut menu = menu();

        menu.set_collapsed(false);

        assert!(menu.is_animating());
        assert_eq!(menu.width(), COLLAPSED_WIDTH);
        assert!(!menu.is_collapsed());
    }

    #[test]
    fn ticks_move_the_width_towards_the_target_without_overshooting() {
        let mut menu = menu();
        menu.set_collapsed(false);

        let start = Instant::now();
        menu.animate(Event::Tick(start));
        menu.animate(Event::Tick(start + TRANSITION / 2));
        let halfway = menu.width();

        assert!(halfway > COLLAPSED_WIDTH, "{halfway} should have grown");
        assert!(halfway < EXPANDED_WIDTH, "{halfway} should not be done yet");

        menu.animate(Event::Tick(start + TRANSITION));

        assert_eq!(menu.width(), EXPANDED_WIDTH);
        assert!(!menu.is_animating());
    }

    #[test]
    fn collapsing_animates_back_to_the_collapsed_width() {
        let mut menu = menu();
        menu.set_collapsed(false);
        settle(&mut menu);

        menu.set_collapsed(true);

        assert!(menu.is_collapsed());
        assert!(menu.is_animating());
        assert_eq!(menu.width(), EXPANDED_WIDTH);

        settle(&mut menu);

        assert_eq!(menu.width(), COLLAPSED_WIDTH);
        assert!(!menu.is_animating());
    }

    #[test]
    fn view_hides_labels_when_collapsed() {
        let menu = menu();

        let mut ui = simulator(menu.view(TestMessage::Animate));

        assert!(ui.find("Home").is_err());
        assert!(ui.find("Search").is_err());
        assert!(ui.find("Settings").is_err());
    }

    #[test]
    fn view_shows_labels_when_expanded() {
        let mut menu = menu();
        menu.set_collapsed(false);
        settle(&mut menu);

        let mut ui = simulator(menu.view(TestMessage::Animate));

        assert!(ui.find("Home").is_ok());
        assert!(ui.find("Search").is_ok());
        assert!(ui.find("Settings").is_ok());
    }

    #[test]
    fn view_shows_labels_while_still_expanding() {
        let mut menu = menu();
        menu.set_collapsed(false);

        let start = Instant::now();
        menu.animate(Event::Tick(start));
        menu.animate(Event::Tick(start + TRANSITION / 4));

        let mut ui = simulator(menu.view(TestMessage::Animate));

        assert!(ui.find("Home").is_ok());
    }

    #[test]
    fn clicking_an_expanded_item_sends_its_message() {
        let mut menu = menu();
        menu.set_collapsed(false);
        settle(&mut menu);

        let mut ui = simulator(menu.view(TestMessage::Animate));
        let _ = ui.click("Home").expect("Home button should be found");

        let messages: Vec<_> = ui.into_messages().collect();

        assert_eq!(messages, vec![TestMessage::OpenHome]);
    }

    #[test]
    fn set_active_marks_only_the_matching_item() {
        let mut menu = menu();

        menu.set_active(&TestMessage::ToggleMenu);

        assert!(menu.top_items[0].active);
        assert!(!menu.top_items[1].active);
        assert!(!menu.top_items[2].active);
        assert!(!menu.bottom_items[0].active);
    }

    #[test]
    fn set_message_replaces_the_item_message() {
        let mut menu = menu();

        menu.top_items[0].set_message(Some(TestMessage::OpenHome));

        assert_eq!(menu.top_items[0].message(), Some(&TestMessage::OpenHome));
    }
}
