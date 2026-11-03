//! Attachments and the file-type helpers.

/// A decoded message attachment (fetched on demand for the reader).
#[derive(Debug, Clone)]
pub(crate) struct Attachment {
    pub(crate) name: String,
    pub(crate) data: Vec<u8>,
}

impl Attachment {
    /// Human-readable size, e.g. "12.3 KB".
    pub(crate) fn human_size(&self) -> String {
        human_size(self.data.len() as u64)
    }
}

/// Human-readable byte size, e.g. "12.3 KB".
pub(crate) fn human_size(bytes: u64) -> String {
    gtk::glib::format_size(bytes).into()
}

/// Whether a filename looks like a raster image we can thumbnail/preview inline.
pub(crate) fn is_image_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [
        ".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".ico", ".heic", ".heif", ".avif",
    ]
    .iter()
    .any(|ext| lower.ends_with(ext))
}
