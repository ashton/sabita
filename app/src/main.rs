mod adapter;
pub mod database;
mod jobs;
mod menu;
mod models;
mod providers;
mod repository;
mod schema;
mod screen;

use crate::{
    menu::{Menu, MenuItem},
    screen::{Home, home, integrations, library, settings},
};
use iced::{
    Element,
    Length::{self, Fill},
    Task,
    widget::{Container, column, container, row},
};
use screen::Screen;

#[derive(Debug, Clone)]
enum Message {
    OpenSettings,
    OpenHome,
    OpenLibrary,
    OpenIntegrations,
    Settings(settings::Message),
    Home(home::Message),
    Library(library::Message),
    Integrations(integrations::Message),
}

struct SabitaApp {
    screen: Screen,
    menu: Menu<Message>,
}

impl SabitaApp {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::Home(Home {}),
                menu: Menu::new(
                    vec![
                        MenuItem::new("bars", false, None),
                        MenuItem::new("house", true, Some(Message::OpenHome)),
                        MenuItem::new("book_open", false, Some(Message::OpenLibrary)),
                        MenuItem::new("magnifying_glass", false, None),
                        MenuItem::new("download", false, None),
                        MenuItem::new("cloud", false, Some(Message::OpenIntegrations)),
                    ],
                    vec![MenuItem::new("gear", false, Some(Message::OpenSettings))],
                ),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Settings(msg) => {
                let Screen::Settings(settings) = &mut self.screen else {
                    return Task::none();
                };

                match settings.update(msg) {
                    settings::Action::None => Task::none(),
                    settings::Action::Run(task) => task.map(Message::Settings),
                }
            }

            Message::Home(msg) => {
                let Screen::Home(home) = &mut self.screen else {
                    return Task::none();
                };

                match home.update(msg) {}
            }

            Message::Library(msg) => {
                let Screen::Library(library) = &mut self.screen else {
                    return Task::none();
                };

                match library.update(msg) {
                    library::Action::None => Task::none(),
                    library::Action::Run(task) => task.map(Message::Library),
                }
            }

            Message::Integrations(msg) => {
                let Screen::Integrations(integrations) = &mut self.screen else {
                    return Task::none();
                };

                match integrations.update(msg) {
                    integrations::Action::None => Task::none(),
                    integrations::Action::Run(task) => task.map(Message::Integrations),
                }
            }

            Message::OpenSettings => self.open_settings(),
            Message::OpenHome => self.open_home(),
            Message::OpenLibrary => self.open_library(),
            Message::OpenIntegrations => self.open_integrations(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let screen = match &self.screen {
            Screen::Home(home) => home.view().map(Message::Home),
            Screen::Library(library) => library.view().map(Message::Library),
            Screen::Settings(settings) => settings.view().map(Message::Settings),
            Screen::Integrations(integrations) => integrations.view().map(Message::Integrations),
        };

        container(
            container(row![self.menu.view(), column![topbar(), screen].height(Fill)].height(Fill))
                .height(Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn open_settings(&mut self) -> Task<Message> {
        let (settings, task) = screen::Settings::new();
        self.screen = Screen::Settings(settings);
        self.menu.set_active(&Message::OpenSettings);
        task.map(Message::Settings)
    }

    fn open_home(&mut self) -> Task<Message> {
        let (params, task) = screen::Home::new();
        self.screen = Screen::Home(params);
        self.menu.set_active(&Message::OpenHome);

        task.map(Message::Home)
    }

    fn open_library(&mut self) -> Task<Message> {
        let (params, task) = screen::Library::new();
        self.screen = Screen::Library(params);
        self.menu.set_active(&Message::OpenLibrary);

        task.map(Message::Library)
    }

    fn open_integrations(&mut self) -> Task<Message> {
        let (params, task) = screen::Integrations::new();
        self.screen = Screen::Integrations(params);
        self.menu.set_active(&Message::OpenIntegrations);

        task.map(Message::Integrations)
    }
}

fn topbar<'a>() -> Container<'a, Message> {
    container(column![])
}

fn main() -> iced::Result {
    iced::application(SabitaApp::new, SabitaApp::update, SabitaApp::view).run()
}
