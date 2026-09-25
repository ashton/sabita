use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum PersonRole {
    Writer = 3,
    Penciller = 4,
    Inker = 5,
    Colorist = 6,
    Letterer = 7,
    CoverArtist = 8,
    Editor = 9,
    Publisher = 10,
    Character = 11,
    Translator = 12,
    Imprint = 13,
    Team = 14,
    Location = 15,
}

#[derive(Deserialize, Debug, Default)]
pub struct Person {
    id: u16,
    name: Option<String>,
    cover_image: Option<String>,
    description: Option<String>,
    roles: Vec<PersonRole>,
}
