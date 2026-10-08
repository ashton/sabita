use serde::Serialize;
use serde_repr::Serialize_repr;

#[derive(Serialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SeriesFilterCombination {
    Or = 0,
    And = 1,
}

#[derive(Serialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SeriesFilterEntityType {
    Series = 0,
    ReadingList = 1,
    Person = 2,
    Annotation = 3,
}

#[derive(Serialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SeriesFilterComparison {
    Equal = 0,
    GreaterThan = 1,
    GreaterThanEqual = 2,
    LessThan = 3,
    LessThanEqual = 4,
    Contains = 5,
    MustContains = 6,
    Matches = 7,
    NotContains = 8,
    NotEqual = 9,
    BeginsWith = 10,
    EndsWith = 11,
    IsBefore = 12,
    IsAfter = 13,
    IsInLast = 14,
    IsNotInLast = 15,
    IsEmpty = 16,
    IsNotEmpty = 17,
}

#[derive(Serialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SeriesFilterField {
    Summary = 0,
    SeriesName = 1,
    PublicationStatus = 2,
    Languages = 3,
    AgeRating = 4,
    UserRating = 5,
    Tags = 6,
    CollectionTags = 7,
    Translators = 8,
    Characters = 9,
    Publisher = 10,
    Editor = 11,
    CoverArtist = 12,
    Letterer = 13,
    Colorist = 14,
    Inker = 15,
    Penciller = 16,
    Writers = 17,
    Genres = 18,
    Libraries = 19,
    ReadProgress = 20,
    Formats = 21,
    ReleaseYear = 22,
    ReadTime = 23,
    Path = 24,
    FilePath = 25,
    WantToRead = 26,
    ReadingDate = 27,
    AverageRating = 28,
    Imprint = 29,
    Team = 30,
    Location = 31,
    ReadLast = 32,
    FileSize = 33,
    CollapseSeriesRelationships = 34,
}

#[derive(Serialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SeriesSortField {
    SortName = 1,
    CreatedDate = 2,
    LastModifiedDate = 3,
    LastChapterAdded = 4,
    TimeToRead = 5,
    ReleaseYear = 6,
    ReadProgress = 7,
    AverageRating = 8,
    Random = 9,
    UserRating = 10,
    UnreadChapterCount = 11,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesSortOption {
    pub sort_field: SeriesSortField,
    pub is_ascending: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesFilterStatement {
    pub comparison: SeriesFilterComparison,
    pub field: SeriesFilterField,
    pub value: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesFilter {
    /// Kavita's `id` is a non-nullable `int` (0 means "not a saved smart
    /// filter"); sending `null` fails its model validation with a 400.
    pub id: u32,
    pub name: Option<String>,
    pub statements: Vec<SeriesFilterStatement>,
    /// Also a non-nullable field on Kavita's side, for the same reason.
    pub combination: SeriesFilterCombination,
    pub sort_options: SeriesSortOption,
    pub entity_type: SeriesFilterEntityType,
    /// Also non-nullable; 0 means "no limit".
    pub limit_to: u32,
}
