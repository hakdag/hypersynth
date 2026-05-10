#[derive(Debug)]
pub enum DocumentContentKind {
    Text(String),
    Image {
        media_type: String,
        /// Base64-encoded raw image bytes for the Anthropic API `source.data` field.
        data_base64: String,
    },
}
