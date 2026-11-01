//! The wrapper document's stylesheet: the cards, their headers, chips and
//! banners, in the reader's grounds and accent. One `format!` so the bytes
//! stay exactly what the page has always received.

/// The wrapper's stylesheet: the cards, their headers and chips, in the
/// reader's grounds and accent.
pub(super) fn wrapper_css(
    scheme: &str,
    page: &str,
    bg: &str,
    chrome: &str,
    accent: &str,
    toggle_color: &str,
    plain_css: &str,
) -> String {
    format!(
        "\
         :root{{color-scheme:{scheme};}}\
         body{{margin:0;padding:0;background:{page};font:14px/1.55 system-ui,sans-serif;}}\
         body.tegami-conv{{padding:14px;}}\
         iframe.tegami-frame{{width:100%;border:0;display:block;background:{bg};\
         visibility:hidden;}}\
         iframe.tegami-frame.tegami-live{{visibility:visible;}}\
         .tegami-pan{{overflow-x:auto;position:relative;}}\
         iframe.tegami-frame.anim{{transition:height 240ms cubic-bezier(0.4,0,0.2,1);}}\
         @media (prefers-reduced-motion:reduce){{iframe.tegami-frame.anim{{transition:none;}}}}\
         .tegami-msg{{background:{bg};\
         border-radius:12px;overflow:hidden;margin:0 0 14px;}}\
         .tegami-msg:last-child{{margin-bottom:0;}}\
         .tegami-msg{{user-select:none;}}\
         body:not(.tegami-conv) .tegami-msg{{border-radius:0;margin:0;}}\
         .tegami-msg.selected{{box-shadow:0 0 0 2px {accent};}}\
         .tegami-fbar{{display:none;}}\
         /* Folded, a card keeps its header exactly as it is open and\
         shows the preview under it, lined up with the sender's name\
         (16px padding, 26px circle, 8px gap). */\
         .tegami-msg.tegami-folded>.tegami-fbar{{display:-webkit-box;-webkit-box-orient:vertical;\
         overflow:hidden;opacity:0.6;font-size:0.92em;line-height:1.35;overflow-wrap:anywhere;\
         margin:-6px 0 12px;padding:0;padding-inline:50px 16px;position:relative;z-index:2;\
         cursor:pointer;user-select:none;}}\
         .tegami-msg.tegami-folded>:not(.tegami-fbar):not(.tegami-msg-hdr){{display:none;}}\
         .tegami-marks{{display:inline-flex;gap:8px;align-self:center;flex:none;}}\
         .tegami-mark{{display:inline-flex;opacity:0.55;line-height:0;}}\
         .tegami-mark svg{{width:16px;height:16px;}}\
         .tegami-mark svg,.tegami-mark svg *{{fill:currentColor;}}\
         .tegami-mark-none{{width:16px;height:16px;}}\
         /* The star keeps its place unstarred, so starring moves nothing. */\
         .tegami-mark-star{{color:#e5a50a;opacity:1;}}\
         .tegami-msg:not(.starred) .tegami-mark-star{{visibility:hidden;}}\
         body.tegami-folds .tegami-msg-hdr{{cursor:pointer;}}\
         @media print{{.tegami-msg.tegami-folded>:not(.tegami-fbar){{display:revert !important;}}\
         .tegami-msg.tegami-folded>.tegami-fbar{{display:none !important;}}}}\
         .tegami-msg-hdr{{cursor:pointer;}}\
         .tegami-msg-hdr{{padding:12px 16px;cursor:default;user-select:none;\
         position:sticky;top:0;z-index:1;background-color:{bg};}}\
         /* 8px between items on a line, but only 2px between wrapped\
         lines — the flex gap otherwise pads rows apart too. */\
         .tegami-hdr-line{{display:flex;column-gap:8px;row-gap:2px;\
         align-items:baseline;flex-wrap:nowrap;\
         min-width:0;position:relative;}}\
         .tegami-hdr-meta{{display:flex;gap:8px;align-items:baseline;flex:none;\
         margin-inline-start:auto;}}\
         /* Narrow pane: the meta group (folder chip, recipients chip, date)\
         drops to its own line as one unit — no mid-text wrapping — while\
         the sender and the action palette hold the first line. */\
         @media (max-width:620px){{\
         /* Tighten the right-hand insets (card gutter + header padding)\
         so the palette toggle sits nearer the pane's edge — but the\
         combined inset never drops below 20px. */\
         body.tegami-conv{{padding:10px;}}\
         body.tegami-conv .tegami-msg-hdr{{padding:12px 10px;}}\
         .tegami-msg.tegami-folded>.tegami-fbar{{padding-inline:44px 10px;}}\
         .tegami-hdr-line{{flex-wrap:wrap;}}\
         .tegami-hdr-meta{{order:10;flex-basis:100%;margin-inline-start:34px;\
         justify-content:flex-start;}}\
         /* Below the wrapped meta line the recipients need a touch\
         more air than the wide layout's 2px. */\
         .tegami-rcpt{{margin-top:6px;}}\
         /* With the meta group gone from the first line, the palette\
         pins itself to the corner (the \u{22ef} is already absolute). */\
         body:not([data-tegami-acts=\"toggle\"]) .tegami-acts{{margin-inline-start:auto;}}\
         }}\
         body:not(.tegami-conv) .tegami-msg-hdr{{background-color:{chrome};}}\
         {plain_css}\
         .tegami-ava{{width:26px;height:26px;border-radius:50%;flex:none;align-self:center;\
         display:flex;align-items:center;justify-content:center;color:#fff;\
         font-size:0.8em;font-weight:700;}}\
         /* Zero flex-basis: the name and address only absorb free space\
         (up to their natural width), so their length can never force\
         the palette onto another line. Under pressure the address\
         gives way first (tiny grow share), fading out at its right\
         edge; the name ellipsizes only after that. */\
         .tegami-from{{font-weight:700;flex:99 1 0;max-width:max-content;min-width:0;\
         white-space:nowrap;overflow:hidden;text-overflow:ellipsis;}}\
         .tegami-tags{{display:inline-flex;gap:4px;align-items:center;flex:0 0 auto;}}\
         .tegami-tags:empty{{display:none;}}\
         /* A card's own attachments (#213): chips beneath the body. */\
         .tegami-atts{{display:flex;flex-wrap:wrap;gap:6px;padding:2px 14px 12px;}}\
         .tegami-atts:empty{{display:none;}}\
         /* The Unsubscribe banner between a list message's header and\
         body: a line of text and one button. */\
         .tegami-tr{{display:flex;align-items:center;gap:10px;padding:8px 14px 9px;\
         font-size:0.9em;border-bottom:1px solid rgba(128,128,128,0.25);}}\
         .tegami-tr:empty{{display:none;}}\
         .tegami-tr-text{{flex:1 1 auto;min-width:0;opacity:0.8;}}\
         .tegami-tr-btn{{flex:none;border:none;border-radius:6px;\
         padding:3px 12px;background:color-mix(in srgb,currentColor 10%,transparent);\
         color:inherit;font:inherit;font-size:0.95em;font-weight:600;cursor:pointer;}}\
         .tegami-tr-btn:hover{{background:color-mix(in srgb,currentColor 15%,transparent);}}\
         .tegami-tr-btn:active{{background:color-mix(in srgb,currentColor 30%,transparent);}}\
         .tegami-unsub,.tegami-mdn{{display:flex;align-items:center;gap:10px;padding:8px 14px 9px;\
         font-size:0.9em;border-bottom:1px solid rgba(128,128,128,0.25);}}\
         .tegami-unsub:empty,.tegami-mdn:empty{{display:none;}}\
         .tegami-unsub-text{{flex:1 1 auto;min-width:0;opacity:0.8;}}\
         .tegami-unsub-btn{{flex:none;border:none;border-radius:6px;\
         padding:3px 12px;background:color-mix(in srgb,currentColor 10%,transparent);\
         color:inherit;font:inherit;font-size:0.95em;font-weight:600;cursor:pointer;}}\
         .tegami-unsub-btn:hover{{background:color-mix(in srgb,currentColor 15%,transparent);}}\
         .tegami-unsub-btn:active{{background:color-mix(in srgb,currentColor 30%,transparent);}}\
         /* The invitation banner (#223): the meeting at the top of\
         the card, and the answer buttons under it. */\
         .tegami-inv{{display:flex;gap:10px;padding:10px 14px 11px;font-size:0.9em;\
         border-bottom:1px solid rgba(128,128,128,0.25);}}\
         .tegami-inv:empty{{display:none;}}\
         .tegami-inv-icon{{flex:none;padding-top:1px;opacity:0.7;}}\
         .tegami-inv-icon svg{{width:20px;height:20px;display:block;}}\
         .tegami-inv-icon svg,.tegami-inv-icon svg *{{fill:currentColor;}}\
         .tegami-inv-body{{flex:1 1 auto;min-width:0;}}\
         .tegami-inv-head{{display:flex;align-items:baseline;gap:8px;flex-wrap:wrap;}}\
         .tegami-inv-title{{font-weight:700;font-size:1.05em;}}\
         .tegami-inv-title.off{{text-decoration:line-through;opacity:0.7;}}\
         .tegami-inv-kind{{font-size:0.85em;text-transform:uppercase;\
         letter-spacing:0.04em;opacity:0.6;}}\
         .tegami-inv-row{{opacity:0.8;margin-top:2px;}}\
         .tegami-inv-note{{margin-top:4px;opacity:0.8;font-style:italic;}}\
         .tegami-inv-acts{{display:flex;flex-wrap:wrap;gap:6px;margin-top:8px;}}\
         .tegami-inv-btn{{border:1px solid rgba(128,128,128,0.4);border-radius:6px;\
         padding:3px 12px;background:rgba(128,128,128,0.1);color:inherit;font:inherit;\
         font-size:0.95em;font-weight:600;cursor:pointer;}}\
         .tegami-inv-btn:hover{{background:rgba(128,128,128,0.22);}}\
         /* The answer already given is marked, so a meeting opened\
         again shows where it stands at a glance. */\
         .tegami-inv-btn.on{{border-color:{accent};color:{accent};\
         background:color-mix(in srgb, {accent} 16%, transparent);}}\
         .tegami-attw{{display:inline-flex;align-items:stretch;max-width:100%;\
         border:1px solid rgba(128,128,128,0.35);border-radius:8px;overflow:hidden;\
         background:rgba(128,128,128,0.08);}}\
         .tegami-attc{{display:inline-flex;align-items:center;gap:6px;padding:4px 8px;\
         border:none;background:none;color:inherit;font:inherit;font-size:0.85em;\
         cursor:pointer;min-width:0;max-width:320px;}}\
         .tegami-attc:hover{{background:rgba(128,128,128,0.18);}}\
         .tegami-attc svg{{width:16px;height:16px;flex:none;display:block;}}\
         .tegami-attc svg,.tegami-attc svg *{{fill:currentColor;}}\
         .tegami-attc.ftype-pdf svg{{color:#e01b24;}}\
         .tegami-attc.ftype-doc svg{{color:#3584e4;}}\
         .tegami-attc.ftype-sheet svg{{color:#2ec27e;}}\
         .tegami-attc.ftype-slides svg{{color:#ff7800;}}\
         .tegami-attc.ftype-archive svg{{color:#e5a50a;}}\
         .tegami-attc.ftype-audio svg{{color:#9141ac;}}\
         .tegami-attc.ftype-video svg{{color:#e6689e;}}\
         .tegami-attc.ftype-calendar svg{{color:#009e8f;}}\
         .tegami-attc.ftype-image svg{{color:#1c9bd8;}}\
         .tegami-attc.ftype-generic svg{{color:#8c8c8c;}}\
         .tegami-attn{{white-space:nowrap;overflow:hidden;text-overflow:ellipsis;min-width:0;}}\
         .tegami-attz{{opacity:0.55;white-space:nowrap;flex:none;}}\
         .tegami-attsave{{display:inline-flex;align-items:center;padding:0 7px;border:none;\
         border-inline-start:1px solid rgba(128,128,128,0.35);background:none;color:inherit;\
         cursor:pointer;opacity:0.6;}}\
         .tegami-attsave:hover{{opacity:1;background:rgba(128,128,128,0.18);}}\
         .tegami-attsave svg{{width:14px;height:14px;display:block;}}\
         .tegami-attsave svg,.tegami-attsave svg *{{fill:currentColor;}}\
         .tegami-atts.flash .tegami-attw{{animation:tegami-att-flash 1.2s ease-out;}}\
         @keyframes tegami-att-flash{{0%,40%{{box-shadow:0 0 0 3px rgba(53,132,228,0.7);}}\
         100%{{box-shadow:0 0 0 3px rgba(53,132,228,0);}}}}\
         .tegami-tag{{font-size:0.72em;font-weight:700;line-height:1.4;padding:1px 7px;\
         border-radius:9999px;white-space:nowrap;}}\
         .tegami-label{{max-width:16em;\
         overflow:hidden;text-overflow:ellipsis;}}\
         .tegami-addr{{opacity:0.55;font-size:0.9em;flex:1 1 0;max-width:max-content;\
         min-width:0;white-space:nowrap;overflow:hidden;text-overflow:clip;\
         position:relative;}}\
         /* The fade exists only while something actually collides with\
         the address (`clipped` is set by measurement, below). */\
         .tegami-addr.clipped{{\
         -webkit-mask-image:linear-gradient(to right,#000 calc(100% - 18px),transparent);\
         mask-image:linear-gradient(to right,#000 calc(100% - 18px),transparent);}}\
         /* Hovering a clipped address reveals it in full: an overlay\
         pinned to the span's spot, floating over whatever crowded it\
         out, gone when the cursor leaves. */\
         .tegami-addr.clipped:hover{{-webkit-mask-image:none;mask-image:none;\
         z-index:6;}}\
         /* Addresses act as links: click composes to them, right-click\
         offers Copy / New Message. */\
         /* Sender-authentication seal (#88): invisible until a\
         verdict arrives; a pass wears Bazaar's fixed blue (never the\
         system accent), problems the warning/error reds.\
         Sized in em and anchored to the text baseline, then eased\
         down to the name's optical centre with an em transform\
         (margins can't move it: WebKit synthesizes the missing\
         baseline before they apply) — a fixed centred pixel box sat\
         visibly low whenever a GNOME text scaling factor shrank the\
         type around it. */\
         .tegami-verify{{display:none;background:none;border:none;\
         padding:0 2px;margin-inline-start:2px;cursor:pointer;line-height:0;\
         align-self:baseline;transform:translateY(0.18em);flex:none;}}\
         .tegami-verify.on{{display:inline-flex;}}\
         .tegami-verify svg{{width:0.95em;height:0.95em;display:block;}}\
         .tegami-verify svg,.tegami-verify svg *{{fill:currentColor;}}\
         .tegami-verify.trust-pass{{color:#3584e4;}}\
         .tegami-verify.trust-unverified{{color:currentColor;opacity:0.4;}}\
         .tegami-verify.trust-suspicious{{color:#cd9309;}}\
         .tegami-verify.trust-fail{{color:#c01c28;}}\
         /* The OpenPGP chip (#133), a labelled pill since #300: the\
         verdict reads in words on a wash of its color. Text of its\
         own, so it sits on the name's baseline without the seal's\
         nudge. */\
         .tegami-pgp{{display:none;font:inherit;font-size:0.78em;font-weight:600;\
         border:none;border-radius:999px;padding-block:0.1em;padding-inline:0.45em 0.6em;\
         margin-inline-start:4px;cursor:pointer;align-items:center;gap:0.3em;\
         align-self:baseline;white-space:nowrap;flex:none;color:inherit;\
         background:rgba(128,128,128,0.16);}}\
         .tegami-pgp.on{{display:inline-flex;}}\
         .tegami-pgp svg{{width:1.1em;height:1.1em;display:none;flex:none;}}\
         .tegami-pgp svg,.tegami-pgp svg *{{fill:currentColor;}}\
         .tegami-pgp.enc svg:first-child{{display:block;}}\
         .tegami-pgp.sig svg:nth-child(2){{display:block;}}\
         .tegami-pgp.pgp-good{{color:#26a269;background:rgba(38,162,105,0.16);}}\
         .tegami-pgp.pgp-warn{{color:#cd9309;background:rgba(205,147,9,0.16);}}\
         .tegami-pgp.pgp-bad{{color:#c01c28;background:rgba(192,28,40,0.14);}}\
         .tegami-pgp:hover{{filter:brightness(1.1);}}\
         /* The sensitivity label (#411): the OpenPGP pill's shape in a\
         neutral wash, since a label is a statement, not a verdict.\
         Zero flex-basis and a small share, like the address: the\
         name fills first and a long label ellipsizes before it. */\
         .tegami-sens{{display:none;font-size:0.78em;font-weight:600;\
         border-radius:999px;padding-block:0.1em;padding-inline:0.45em 0.6em;\
         margin-inline-start:4px;align-items:center;gap:0.3em;\
         align-self:baseline;white-space:nowrap;flex:1 1 0;min-width:3.5em;max-width:max-content;\
         background:rgba(128,128,128,0.16);cursor:default;}}\
         .tegami-sens.on{{display:inline-flex;}}\
         .tegami-sens svg{{width:1em;height:1em;flex:none;}}\
         .tegami-sens svg,.tegami-sens svg *{{fill:var(--tegami-sens,currentColor);}}\
         .tegami-sens-text{{overflow:hidden;text-overflow:ellipsis;}}\
         /* The icons alone (Settings, #300): the chip as it was before\
         it had words, sitting on the name's line like the seal. */\
         body[data-tegami-pgpicons] .tegami-pgp{{background:none;padding:0 2px;\
         margin-inline-start:2px;gap:1px;font-size:inherit;line-height:0;\
         transform:translateY(0.18em);}}\
         body[data-tegami-pgpicons] .tegami-pgp svg{{width:0.95em;height:0.95em;}}\
         body[data-tegami-pgpicons] .tegami-pgp-text{{display:none;}}\
         .tegami-mail{{cursor:pointer;}}\
         .tegami-mail:hover{{text-decoration:underline;}}\
         /* Styled after the app's own context menus (context_menu.rs +\
         .context-menu-list in styles.css): same padding, min width,\
         row shape and weight, so both menu families read as one. */\
         .tegami-mailmenu-scrim{{position:fixed;left:0;top:0;right:0;bottom:0;\
         z-index:98;}}\
         .tegami-mailmenu{{position:fixed;z-index:99;background:{bg};\
         border:1px solid rgba(128,128,128,0.2);border-radius:12px;\
         box-shadow:0 6px 24px rgba(0,0,0,0.22);padding:6px;\
         min-width:190px;font-size:0.95em;}}\
         .tegami-mailmenu button{{display:block;width:100%;text-align:start;\
         background:none;border:none;padding:8px 10px;border-radius:6px;\
         color:inherit;cursor:pointer;font:inherit;font-weight:normal;}}\
         .tegami-mailmenu button:hover{{background:rgba(128,128,128,0.14);}}\
         .tegami-addr.clipped:hover::after{{content:attr(data-addr);\
         position:absolute;inset-inline-start:-6px;top:50%;transform:translateY(-50%);\
         background:{bg};border-radius:6px;padding:1px 6px;\
         white-space:nowrap;text-decoration:underline;}}\
         body:not(.tegami-conv) .tegami-addr.clipped:hover::after{{\
         background:{chrome};}}\
         .tegami-date{{opacity:0.55;font-size:0.85em;flex:none;white-space:nowrap;}}\
         .tegami-read{{display:none;flex:none;align-self:center;margin-inline-end:-3px;}}\
         .tegami-read.asked,.tegami-read.read{{display:inline-flex;}}\
         .tegami-read svg{{width:1.05em;height:1.05em;display:block;}}\
         .tegami-read svg,.tegami-read svg *{{fill:currentColor;}}\
         .tegami-read.asked{{opacity:0.55;}}\
         .tegami-read.read{{color:#26a269;}}\
         .tegami-dot{{width:8px;height:8px;border-radius:50%;background:#3584e4;\
         flex:none;align-self:center;}}\
         .tegami-end{{height:1px;}}\
         .tegami-quote{{display:block;margin:0 16px 12px;padding:0 7px;\
         font:inherit;font-size:0.7em;line-height:1.45;letter-spacing:0.06em;\
         color:inherit;opacity:0.6;background:rgba(128,128,128,0.16);\
         border:0;border-radius:999px;cursor:pointer;}}\
         .tegami-quote:hover{{opacity:0.95;background:rgba(128,128,128,0.28);}}\
         .tegami-quote.open{{opacity:0.95;position:absolute;inset-inline-start:0;z-index:2;}}\
         .tegami-acts{{display:flex;gap:2px;flex:none;align-self:center;margin-inline-start:4px;}}\
         /* Read-toggle: the icon showing is the ACTION (read envelope\
         means mark-as-read); the section's unread class decides. */\
         .tegami-act .tr-when-unread,.tegami-act .tr-when-read{{display:none;line-height:0;}}\
         .tegami-msg.unread .tegami-act[data-act=\"toggleread\"] .tr-when-unread{{display:inline-flex;}}\
         .tegami-msg:not(.unread) .tegami-act[data-act=\"toggleread\"] .tr-when-read{{display:inline-flex;}}\
         .tegami-act{{color:inherit;background:none;border:none;border-radius:6px;\
         padding:5px 8px;cursor:pointer;opacity:0.7;\
         display:inline-flex;align-items:center;justify-content:center;\
         vertical-align:middle;line-height:0;\
         transition:opacity 120ms ease,background 120ms ease;}}\
         .tegami-act:hover{{opacity:1;background:rgba(128,128,128,0.18);}}\
         .tegami-act:active{{background:rgba(128,128,128,0.3);}}\
         .tegami-act svg{{width:14px;height:14px;display:block;}}\
         .tegami-act svg,.tegami-act svg *{{fill:currentColor;}}\
         /* A set star: the one state that carries color. */\
         .tegami-act.on{{color:#e5a50a;opacity:1;}}\
         .tegami-acts-toggle{{display:none;}}\
         /* Behind-the-\u{22ef} mode: the palette overlays the header line as an\
         absolute strip anchored at the toggle, expanding leftward over\
         the text beneath (never pushing it), its left edge fading in\
         from transparent. The \u{22ef} itself never moves. */\
         body[data-tegami-acts=\"toggle\"] .tegami-acts{{position:absolute;\
         inset-inline-end:34px;top:13px;transform:translateY(-50%) scaleX(0.6);\
         transform-origin:right center;margin:0;\
         padding-block:2px;padding-inline:30px 6px;border-radius:8px;\
         background:linear-gradient(to right,transparent,{bg} 26px);\
         opacity:0;pointer-events:none;\
         transition:opacity 140ms ease,transform 180ms ease;}}\
         body[data-tegami-acts=\"toggle\"] .tegami-acts.open{{opacity:1;\
         pointer-events:auto;transform:translateY(-50%) scaleX(1);}}\
         body:not(.tegami-conv)[data-tegami-acts=\"toggle\"] .tegami-acts{{\
         background:linear-gradient(to right,transparent,{chrome} 26px);}}\
         /* Always visible (not just on card hover), and OUT OF FLOW:\
         absolutely pinned to the header's right edge, so no amount of\
         header content can push or wrap it. The line reserves the\
         button's footprint as fixed right padding instead. */\
         body[data-tegami-acts=\"toggle\"] .tegami-hdr-line{{padding-inline-end:38px;}}\
         /* Anchored to the FIRST row's centre (the 26px avatar/sender\
         line), not the line box's — when the meta row wraps beneath,\
         the box grows downward but the button must not move. */\
         body[data-tegami-acts=\"toggle\"] .tegami-acts-toggle{{display:block;\
         position:absolute;inset-inline-end:0;top:13px;transform:translateY(-50%);\
         color:{toggle_color};background:none;border:none;border-radius:6px;\
         padding:4px 8px;margin:0;cursor:pointer;\
         opacity:0.65;transition:opacity 120ms ease,background 120ms ease;}}\
         /* `svg *` too: the embedded symbolic paths carry their own fill\
         attribute, which a rule on the svg element alone can't beat. */\
         .tegami-acts-toggle svg{{width:14px;height:14px;display:block;}}\
         .tegami-acts-toggle svg,.tegami-acts-toggle svg *{{fill:currentColor;}}\
         body[data-tegami-acts=\"toggle\"] .tegami-acts-toggle:hover{{opacity:1;\
         background:rgba(128,128,128,0.18);}}\
         body[data-tegami-acts=\"toggle\"] .tegami-acts-toggle.open{{opacity:1;}}\
         body[data-tegami-acts=\"hover\"] .tegami-acts{{opacity:0;pointer-events:none;\
         transition:opacity 300ms ease var(--acts-delay,0ms);}}\
         body[data-tegami-acts=\"hover\"] .tegami-msg:hover .tegami-acts{{opacity:1;\
         pointer-events:auto;transition:opacity 300ms ease 0ms;}}\
         .tegami-folder{{padding:0.05em 0.45em;border-radius:0.7em;\
         font-size:0.78em;opacity:0.75;border:1px solid currentColor;\
         white-space:nowrap;flex:none;}}\
         .tegami-rcpt-toggle{{font:inherit;font-size:0.78em;color:inherit;\
         background:color-mix(in srgb,currentColor 10%,transparent);\
         border:none;border-radius:6px;padding:0.15em 0.6em;cursor:pointer;\
         white-space:nowrap;flex:none;transition:background 120ms ease;}}\
         .tegami-rcpt-toggle:hover{{background:color-mix(in srgb,currentColor 15%,transparent);}}\
         .tegami-rcpt-toggle.open{{background:color-mix(in srgb,currentColor 20%,transparent);}}\
         /* Indented past the avatar (26px + 8px gap), so the recipients\
         align with the sender's name — as does the wrapped meta line. */\
         .tegami-rcpt{{font-size:0.85em;opacity:0.75;\
         user-select:text;overflow-wrap:anywhere;margin:2px 0 0;margin-inline-start:34px;}}\
         .tegami-rcpt div{{margin-top:2px;}}\
         .tegami-loading{{opacity:0.5;padding:16px;}}\
         /* The Copied pill after Ctrl+C: a toast the page draws itself,\
         since the reader has no toast overlay of its own. */\
         .tegami-copied{{position:fixed;left:50%;bottom:22px;transform:translateX(-50%) translateY(8px);\
         padding:8px 18px;border-radius:999px;background:rgba(40,40,40,0.92);color:#fff;\
         font:inherit;font-size:0.9em;box-shadow:0 2px 10px rgba(0,0,0,0.25);\
         opacity:0;pointer-events:none;transition:opacity 180ms ease,transform 180ms ease;z-index:50;}}\
         .tegami-copied.on{{opacity:1;transform:translateX(-50%) translateY(0);}}\
         /* Right to left (#366): what logical properties cannot say. */\
         html[dir=rtl] .tegami-addr.clipped{{\
         -webkit-mask-image:linear-gradient(to left,#000 calc(100% - 18px),transparent);\
         mask-image:linear-gradient(to left,#000 calc(100% - 18px),transparent);}}\
         html[dir=rtl] .tegami-addr.clipped:hover{{-webkit-mask-image:none;mask-image:none;}}\
         html[dir=rtl] body[data-tegami-acts=\"toggle\"] .tegami-acts{{\
         transform-origin:left center;\
         background:linear-gradient(to left,transparent,{bg} 26px);}}\
         html[dir=rtl] body:not(.tegami-conv)[data-tegami-acts=\"toggle\"] .tegami-acts{{\
         background:linear-gradient(to left,transparent,{chrome} 26px);}}\
         /* Each chip's words in their own direction, as a GTK label\
         has them: an English 1 recipient in a right-to-left line\
         otherwise reads recipient 1. */\
         html[dir=rtl] :is(.tegami-rcpt-toggle,.tegami-date,\
         .tegami-folder,.tegami-rcpt div){{unicode-bidi:plaintext;}}\
         "
    )
}
