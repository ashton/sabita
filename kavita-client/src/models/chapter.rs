use chrono::{DateTime, NaiveTime, Utc};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::models::{file::File, genre::GenreTag, person::Person};

#[derive(Deserialize_repr, Debug, Default)]
#[repr(u8)]
pub enum Format {
    Image = 0,
    Archive = 1,
    #[default]
    Unknown = 2,
    Epub = 3,
    Pdf = 4,
}

#[derive(Deserialize_repr, Debug, Default)]
#[repr(u8)]
pub enum PublicationStatus {
    #[default]
    OnGoing = 0,
    Hiatus = 1,
    Completed = 2,
    Cancelled = 3,
    Ended = 4,
}

#[derive(Deserialize_repr, Debug, Default)]
#[repr(i8)]
pub enum AgeRating {
    #[default]
    Unknown = 0,
    RatingPending = 1,
    EarlyChildhood = 2,
    Everyone = 3,
    G = 4,
    EveryoneTenPlus = 5,
    PG = 6,
    KidsToAdults = 7,
    Teen = 8,
    MatureFifteenPlus = 9,
    MatureSeventeenPlus = 10,
    Mature = 11,
    REighteenPlus = 12,
    AdultsOnly = 13,
    XEighteenPlus = 14,
    NotApplicable = -1,
}

#[derive(Deserialize, Debug, Default)]
pub struct Chapter {
    id: u16,
    range: String,
    min_number: f32,
    max_number: f32,
    sort_order: f32,
    pages: u16,
    is_special: bool,
    title: Option<String>,
    pages_read: u16,
    total_reads: u16,
    last_reading_progress_utc: DateTime<Utc>,
    last_reading_progress: NaiveTime,
    volume_id: u16,
    created_utc: DateTime<Utc>,
    last_modified_utc: DateTime<Utc>,
    created: NaiveTime,
    release_date: NaiveTime,
    title_name: Option<String>,
    summary: Option<String>,
    volume_title: Option<String>,
    isbn: Option<String>,
    language: Option<String>,
    cover_image: String,
    files: Vec<File>,
    age_rating: AgeRating,
    writers: Vec<Person>,
    cover_artists: Vec<Person>,
    publishers: Vec<Person>,
    characters: Vec<Person>,
    pencillers: Vec<Person>,
    inkers: Vec<Person>,
    imprints: Vec<Person>,
    colorists: Vec<Person>,
    letterers: Vec<Person>,
    editors: Vec<Person>,
    translators: Vec<Person>,
    teams: Vec<Person>,
    locations: Vec<Person>,
    genres: Vec<GenreTag>,
    publication_status: PublicationStatus,
    format: Format,
}
