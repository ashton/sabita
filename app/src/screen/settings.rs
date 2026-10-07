use iced::{
    Element, Task,
    widget::{column, container, text_input},
};

pub struct Settings {
    url: String,
    name: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    UrlChanged(String),
    NameChanged(String),
}

pub enum Action {
    None,
    Run(Task<Message>),
}

impl Settings {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                url: String::from(""),
                name: String::from(""),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::UrlChanged(url) => {
                self.url = url;
                Action::None
            }
            Message::NameChanged(name) => {
                self.name = name;
                Action::None
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        container(column![
            text_input("name", &self.name).on_input(Message::NameChanged),
            text_input("url", &self.name).on_input(Message::UrlChanged)
        ])
        .into()
    }
}
