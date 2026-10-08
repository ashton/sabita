pub mod browse;
pub mod home;
pub mod integration;
pub mod library;
pub mod settings;

pub use browse::Browse;
pub use home::Home;
pub use integration::Integrations;
pub use library::Library;
pub use settings::Settings;

pub enum Screen {
    Home(Home),
    Library(Library),
    Browse(Browse),
    Integrations(Integrations),
    Settings(Settings),
}
