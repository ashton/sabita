pub mod configuration;
pub mod library;
pub mod library_item;

#[derive(Debug, Default)]
pub enum AsyncModel<T, E> {
    #[default]
    NotLoaded,
    Loading,
    Loaded(T),
    Error(E),
}

impl<T, E> AsyncModel<T, E> {
    pub fn map<O, R>(self, op: O) -> AsyncModel<R, E>
    where
        O: FnOnce(T) -> R,
    {
        match self {
            AsyncModel::NotLoaded => AsyncModel::NotLoaded,
            AsyncModel::Loading => AsyncModel::Loading,
            AsyncModel::Loaded(value) => AsyncModel::Loaded(op(value)),
            AsyncModel::Error(e) => AsyncModel::Error(e),
        }
    }
}

impl<T, E> From<Result<T, E>> for AsyncModel<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => AsyncModel::Loaded(value),
            Err(error) => AsyncModel::Error(error),
        }
    }
}
