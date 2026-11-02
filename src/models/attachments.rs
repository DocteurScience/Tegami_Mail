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

/// The file's extension, lower-cased, or empty when the name has none: a
/// generated "attachment-1", or a dotless name from a sender's client.
/// Whatever follows the last dot only counts as an extension when it looks
/// like one (short, alphanumeric), so "Report v1.2 draft" has none either.
pub(crate) fn ext_of(name: &str) -> String {
    let Some((_, ext)) = name.rsplit_once('.') else {
        return String::new();
    };
    if ext.is_empty() || ext.len() > 8 || !ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        return String::new();
    }
    ext.to_ascii_lowercase()
}

/// Searchable category words for a file, keyed off its extension, so a query
/// like "image" or "spreadsheet" matches even when the word isn't in the name.
pub(crate) fn type_keywords(name: &str) -> &'static str {
    match ext_of(name).as_str() {
        "pdf" => "pdf document",
        "doc" | "docx" | "odt" | "rtf" => "word document",
        "xls" | "xlsx" | "ods" | "csv" => "excel spreadsheet",
        "ppt" | "pptx" | "odp" => "powerpoint presentation slides",
        "zip" | "gz" | "tar" | "7z" | "rar" | "xz" | "bz2" => "archive compressed",
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "audio music sound",
        "mp4" | "mov" | "mkv" | "webm" | "avi" | "m4v" => "video movie",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "heic" | "heif" | "avif" | "ico" => {
            "image photo picture"
        }
        "ics" => "calendar event",
        "txt" | "md" => "text document",
        _ => "file",
    }
}

/// Which row of the gallery's type dropdown a file belongs to: 1 images,
/// 2 PDFs, 3 documents, 4 archives, 5 audio/video, 6 anything else. Stored
/// alongside each attachment so the filter and the "Type" sort are one SQL
/// query rather than a pass over every row.
pub(crate) fn type_bucket(name: &str) -> u32 {
    match ext_of(name).as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "heic" | "heif" | "avif" | "ico" => 1,
        "pdf" => 2,
        "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "xls" | "xlsx" | "ods" | "csv" | "ppt"
        | "pptx" | "odp" | "ics" => 3,
        "zip" | "gz" | "tar" | "7z" | "rar" | "xz" | "bz2" => 4,
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "mp4" | "mov" | "mkv" | "webm" | "avi"
        | "m4v" => 5,
        _ => 6,
    }
}

/// The file extension matching an image's magic bytes ("jpg" when unsure —
/// for content that is known to be an image but arrived without a name).
pub(crate) fn image_ext(data: &[u8]) -> &'static str {
    match data {
        [0x89, b'P', b'N', b'G', ..] => "png",
        [b'G', b'I', b'F', b'8', ..] => "gif",
        [b'B', b'M', ..] => "bmp",
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "webp",
        _ => "jpg",
    }
}

/// What the server says about one attachment, without its bytes, as the
/// cache's `attachment_meta` records it.
#[derive(Debug, Clone)]
pub(crate) struct AttachmentMeta {
    /// Which attachment of the message this is, in the order the scan found them.
    pub(crate) idx: u32,
    pub(crate) name: String,
    /// MIME type as declared, e.g. "application/pdf"; empty when unknown.
    pub(crate) mime: String,
    /// Decoded size in bytes (BODYSTRUCTURE reports the encoded size, which
    /// base64 inflates by 4/3).
    pub(crate) size: u64,
    /// IMAP part section, e.g. "2" or "1.3" — what to FETCH to get just this
    /// file. Empty when the scan could not work the structure out, in which
    /// case opening it falls back to fetching the whole message.
    pub(crate) section: String,
}
