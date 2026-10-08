use iced::{
    Element, Task,
    widget::{container, row},
};

#[derive(Debug, Default)]
pub struct Home;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {}

pub enum Action {}

impl Home {
    pub fn new() -> (Self, Task<Message>) {
        (Self, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {}
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        container(row![]).into()
    }
}
