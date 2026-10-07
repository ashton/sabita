pub mod home;
pub mod library;
pub mod settings;

pub use home::Home;
pub use library::Library;
pub use settings::Settings;

pub enum Screen {
    Home(Home),
    Library(Library),
    Settings(Settings),
}
