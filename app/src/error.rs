/// Logs `error` at debug level and converts it to a `String`.
///
/// Repository and provider functions return `Result<T, String>` so their
/// errors can flow straight into a `Message` variant; using this as the
/// `map_err` at that boundary means the original error is still visible in
/// the logs even though the UI only ever sees the stringified version.
pub fn log_and_stringify<E: std::fmt::Display>(error: E) -> String {
    tracing::debug!("{error}");
    error.to_string()
}
