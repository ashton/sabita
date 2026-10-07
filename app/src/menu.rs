use iced::{
    Background, Element,
    Length::Fill,
    Theme,
    widget::{Button, button, center_x, column, container, space},
};
use iced_font_awesome::fa_icon_solid;

pub struct MenuItem<Message> {
    icon: &'static str,
    active: bool,
    message: Option<Message>,
}

pub struct Menu<Message> {
    pub top_items: Vec<MenuItem<Message>>,
    pub bottom_items: Vec<MenuItem<Message>>,
}

fn menu_icon<'a, Message: Clone + 'a>(
    icon: &'static str,
    active: bool,
    on_press: Option<Message>,
) -> Button<'a, Message> {
    button(center_x(fa_icon_solid(icon).size(15.0_f32)))
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
            top_items,
            bottom_items,
        }
    }

    pub fn set_active(&mut self, message: &Message) {
        let target = std::mem::discriminant(message);

        for item in self.top_items.iter_mut().chain(self.bottom_items.iter_mut()) {
            item.active = item
                .message
                .as_ref()
                .is_some_and(|item_message| std::mem::discriminant(item_message) == target);
        }
    }

    pub fn view<'a>(&self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let mut children: Vec<Element<'a, Message>> = vec![];

        children.extend(self.top_items.iter().map(|item: &MenuItem<Message>| {
            menu_icon(item.icon, item.active, item.message.clone()).into()
        }));

        children.push(space::vertical().into());
        children.extend(self.bottom_items.iter().map(|item: &MenuItem<Message>| {
            menu_icon(item.icon, item.active, item.message.clone()).into()
        }));

        container(column(children).height(Fill))
            .center_x(44)
            .height(Fill)
            .into()
    }
}

impl<Message> MenuItem<Message> {
    pub fn new(icon: &'static str, active: bool, message: Option<Message>) -> Self {
        Self {
            icon,
            active,
            message,
        }
    }
}
