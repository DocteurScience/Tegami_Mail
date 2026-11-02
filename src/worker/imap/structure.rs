//! Whether BODYSTRUCTURE says a message has an attachment, without
//! downloading anything.

use crate::worker::render::attachments::INLINE_ATTACHMENT_MIN;

/// Whether an IMAP BODYSTRUCTURE contains a part marked as an attachment.
///
/// `Content-Disposition: attachment` is the obvious case, but Apple Mail sends
/// iPhone photos as *inline* parts of a multipart/mixed, so disposition alone
/// misses them. Any non-text part therefore counts, except a small one carrying a
/// Content-ID — see [`INLINE_ATTACHMENT_MIN`].
pub(in crate::worker) fn structure_has_attachment(
    bs: &async_imap::imap_proto::types::BodyStructure,
) -> bool {
    use async_imap::imap_proto::types::{BodyStructure as Bs, ContentEncoding};

    let is_attachment = |common: &async_imap::imap_proto::types::BodyContentCommon| {
        common
            .disposition
            .as_ref()
            .is_some_and(|d| d.ty.eq_ignore_ascii_case("attachment"))
    };
    // BODYSTRUCTURE reports the *encoded* size; base64 inflates by 4/3.
    let decoded_size = |other: &async_imap::imap_proto::types::BodyContentSinglePart| {
        let octets = other.octets as usize;
        match other.transfer_encoding {
            ContentEncoding::Base64 => octets / 4 * 3,
            _ => octets,
        }
    };

    match bs {
        Bs::Multipart { bodies, .. } => bodies.iter().any(structure_has_attachment),
        // A text part is an attachment only when it says so — except a
        // calendar part, which a meeting request sends as a plain
        // alternative with no disposition at all (#223). That part is the
        // meeting: it has to be listed, and the reader has to know the
        // message carries files before it will ask for them.
        Bs::Text { common, .. } => is_attachment(common) || is_calendar(common),
        Bs::Basic { common, other, .. } | Bs::Message { common, other, .. } => {
            is_attachment(common)
                || (!is_receipt_report(common)
                    && (other.id.is_none() || decoded_size(other) >= INLINE_ATTACHMENT_MIN))
        }
    }
}

/// Whether a BODYSTRUCTURE part is a read receipt's report (#520), which
/// the reader keeps out of the attachments.
fn is_receipt_report(common: &async_imap::imap_proto::types::BodyContentCommon) -> bool {
    common.ty.ty.eq_ignore_ascii_case("message")
        && common
            .ty
            .subtype
            .eq_ignore_ascii_case("disposition-notification")
}

/// Whether a BODYSTRUCTURE part is a `text/calendar` one (#223).
fn is_calendar(common: &async_imap::imap_proto::types::BodyContentCommon) -> bool {
    common.ty.ty.eq_ignore_ascii_case("text") && common.ty.subtype.eq_ignore_ascii_case("calendar")
}

#[cfg(test)]
mod structure_tests {
    use super::INLINE_ATTACHMENT_MIN;
    use crate::worker::imap::structure::structure_has_attachment;

    #[test]
    fn large_cid_image_counts_toward_the_paperclip() {
        // BODYSTRUCTURE reports the base64 size: 4 characters per 3 bytes, padded.
        let octets = INLINE_ATTACHMENT_MIN.div_ceil(3) * 4;
        let raw = format!(
            concat!(
                "* 1 FETCH (BODYSTRUCTURE (",
                "(\"TEXT\" \"HTML\" (\"CHARSET\" \"utf-8\") NIL NIL \"7BIT\" 20 1 NIL NIL NIL NIL)",
                "(\"IMAGE\" \"JPEG\" (\"NAME\" \"pic.jpg\") \"<pic>\" NIL \"BASE64\" {} NIL ",
                "(\"INLINE\" (\"FILENAME\" \"pic.jpg\")) NIL NIL)",
                " \"RELATED\" (\"BOUNDARY\" \"r\") NIL NIL NIL))\r\n",
            ),
            octets
        );
        assert!(structure_has_attachment(&bodystructure(&raw)));
    }

    /// Parse a real server BODYSTRUCTURE response into the structure our IMAP
    /// summary path sees.
    fn bodystructure(raw: &str) -> async_imap::imap_proto::types::BodyStructure<'_> {
        use async_imap::imap_proto::{parser::parse_response, AttributeValue, Response};
        let (_, resp) = parse_response(raw.as_bytes()).expect("parses");
        match resp {
            Response::Fetch(_, attrs) => attrs
                .into_iter()
                .find_map(|a| match a {
                    AttributeValue::BodyStructure(bs) => Some(bs),
                    _ => None,
                })
                .expect("has BODYSTRUCTURE"),
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn inline_iphone_photo_counts_as_an_attachment() {
        // Apple marks the JPEG `INLINE` with no Content-ID; requiring a disposition
        // of `attachment` missed it, so no paperclip and no download.
        let bs = bodystructure(concat!(
            "* 1 FETCH (BODYSTRUCTURE (",
            "(\"TEXT\" \"PLAIN\" (\"CHARSET\" \"us-ascii\") NIL NIL \"7BIT\" 4 2 NIL NIL NIL NIL)",
            "(\"IMAGE\" \"JPEG\" (\"NAME\" \"a.jpeg\") NIL NIL \"BASE64\" 100 NIL ",
            "(\"INLINE\" (\"FILENAME\" \"a.jpeg\")) NIL NIL)",
            "(\"TEXT\" \"PLAIN\" (\"CHARSET\" \"us-ascii\") NIL NIL \"7BIT\" 40 4 NIL NIL NIL NIL)",
            " \"MIXED\" (\"BOUNDARY\" \"b\") NIL NIL NIL))\r\n",
        ));
        assert!(structure_has_attachment(&bs));
    }

    #[test]
    fn cid_logo_does_not_count_as_an_attachment() {
        // Same inline disposition, but a Content-ID: it's rendered in the body.
        let bs = bodystructure(concat!(
            "* 1 FETCH (BODYSTRUCTURE (",
            "(\"TEXT\" \"HTML\" (\"CHARSET\" \"utf-8\") NIL NIL \"7BIT\" 20 1 NIL NIL NIL NIL)",
            "(\"IMAGE\" \"PNG\" (\"NAME\" \"logo.png\") \"<logo>\" NIL \"BASE64\" 100 NIL ",
            "(\"INLINE\" (\"FILENAME\" \"logo.png\")) NIL NIL)",
            " \"RELATED\" (\"BOUNDARY\" \"r\") NIL NIL NIL))\r\n",
        ));
        assert!(!structure_has_attachment(&bs));
    }

    #[test]
    fn plain_alternative_has_no_attachment() {
        let bs = bodystructure(concat!(
            "* 1 FETCH (BODYSTRUCTURE (",
            "(\"TEXT\" \"PLAIN\" (\"CHARSET\" \"utf-8\") NIL NIL \"7BIT\" 10 1 NIL NIL NIL NIL)",
            "(\"TEXT\" \"HTML\" (\"CHARSET\" \"utf-8\") NIL NIL \"7BIT\" 20 1 NIL NIL NIL NIL)",
            " \"ALTERNATIVE\" (\"BOUNDARY\" \"a\") NIL NIL NIL))\r\n",
        ));
        assert!(!structure_has_attachment(&bs));
    }

    /// A meeting request's calendar part is an attachment, however the
    /// sender labelled it (#223): it arrives as a plain alternative with no
    /// disposition at all, which is the one text part that still counts.
    #[test]
    fn a_calendar_part_counts_as_an_attachment() {
        let bs = bodystructure(concat!(
            "* 1 FETCH (BODYSTRUCTURE (",
            "(\"TEXT\" \"PLAIN\" (\"CHARSET\" \"utf-8\") NIL NIL \"7BIT\" 10 1 NIL NIL NIL NIL)",
            "(\"TEXT\" \"HTML\" (\"CHARSET\" \"utf-8\") NIL NIL \"7BIT\" 20 1 NIL NIL NIL NIL)",
            "(\"TEXT\" \"CALENDAR\" (\"CHARSET\" \"utf-8\" \"METHOD\" \"REQUEST\") NIL NIL ",
            "\"7BIT\" 900 20 NIL NIL NIL NIL)",
            " \"ALTERNATIVE\" (\"BOUNDARY\" \"a\") NIL NIL NIL))\r\n",
        ));
        assert!(structure_has_attachment(&bs));
    }
}
