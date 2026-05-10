#[derive(Debug)]
pub enum DocumentContextError {
    NotFoundOrForbidden,
    ContentUnavailable,
    UnsupportedDocumentType,
}
