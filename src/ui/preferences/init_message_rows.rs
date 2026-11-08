//! Filling the Composing rows (Messages, and its Advanced sub-page) when
//! the dialog is built.

use super::choices::{compose_format_index, signature_position_index};
use super::rows::no_truncate;
use super::*;

impl Preferences {
    /// Settings → Composing, spelling included.
    pub(super) fn init_composing_rows(
        &mut self,
        widgets: &PreferencesWidgets,
        init: &PrefInit,
        sender: &ComponentSender<Self>,
    ) {
        // "Send new messages from": the open folder's account, then every
        // enabled account and alias, labelled as the composer's From row
        // labels them. Only meaningful with more than one identity.
        {
            let mut labels: Vec<String> = vec![i18n("Account of the current folder")];
            labels.extend(self.identities.iter().map(|(name, addr)| {
                if name.trim().is_empty() {
                    addr.clone()
                } else {
                    format!("{name} <{addr}>")
                }
            }));
            let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
            widgets
                .composing
                .default_from_row
                .set_model(Some(&gtk::StringList::new(&refs)));
            let want = init.compose_default_from.trim();
            let sel = self
                .identities
                .iter()
                .position(|(_, addr)| addr.eq_ignore_ascii_case(want))
                .map(|i| i + 1)
                .unwrap_or(0);
            widgets.composing.default_from_row.set_selected(sel as u32);
            widgets
                .composing
                .default_from_row
                .set_visible(self.identities.len() > 1);
            middle_ellipsize(&widgets.composing.default_from_row);
        }
        widgets
            .composing
            .paste_plain_row
            .set_active(init.paste_plain);
        widgets
            .composing
            .return_paragraph_row
            .set_active(init.return_paragraph);
        // "Always collapsed" in full, not ellipsized beside the description.
        widgets
            .composing
            .toolbar_row
            .set_factory(Some(&crate::ui::accounts::non_ellipsizing_factory()));
        widgets
            .composing
            .toolbar_row
            .set_selected(u32::from(init.toolbar_expanded));
        widgets
            .composing
            .compose_format_row
            .set_model(Some(&gtk::StringList::new(&[
                &i18n("Rich text"),
                &i18n("Plain text"),
            ])));
        no_truncate(&widgets.composing.compose_format_row);
        widgets
            .composing
            .compose_format_row
            .set_selected(compose_format_index(init.compose_format));
        compose_font_setup(
            &widgets.composing.compose_font_row,
            &init.compose_font,
            sender.clone(),
        );
        {
            let row = &widgets.composing.compose_size_row;
            let sizes = crate::ui::rich_editor::SIZES;
            let names: Vec<String> = std::iter::once(i18n("Default"))
                .chain(sizes.iter().map(|pt| pt.to_string()))
                .collect();
            row.set_model(Some(&gtk::StringList::new(
                &names.iter().map(String::as_str).collect::<Vec<_>>(),
            )));
            let at = sizes
                .iter()
                .position(|pt| *pt == init.compose_font_size)
                .map_or(0, |i| i + 1);
            row.set_selected(at as u32);
            let sender = sender.clone();
            row.connect_selected_notify(move |row| {
                let pt = row
                    .selected()
                    .checked_sub(1)
                    .and_then(|i| sizes.get(i as usize))
                    .copied()
                    .unwrap_or(0);
                let _ = sender.output(PrefOutput::SetComposeFontSize(pt));
            });
        }
        {
            // "Same as reading pane zoom" first: 0, the reader's default zoom.
            let row = &widgets.composing.compose_zoom_row;
            let steps = crate::config::READER_ZOOM_STEPS;
            let names: Vec<String> = std::iter::once(i18n("Same as reading pane zoom"))
                .chain(steps.iter().map(|z| format!("{z}%")))
                .collect();
            row.set_model(Some(&gtk::StringList::new(
                &names.iter().map(String::as_str).collect::<Vec<_>>(),
            )));
            no_truncate(row);
            let at = steps
                .iter()
                .position(|z| *z == init.compose_zoom)
                .map_or(0, |i| i + 1);
            row.set_selected(at as u32);
            let sender = sender.clone();
            row.connect_selected_notify(move |row| {
                let zoom = row
                    .selected()
                    .checked_sub(1)
                    .and_then(|i| steps.get(i as usize))
                    .copied()
                    .unwrap_or(0);
                let _ = sender.output(PrefOutput::SetComposeZoom(zoom));
            });
        }
        widgets
            .composing
            .follow_plain_row
            .set_active(init.follow_plain);
        widgets
            .composing
            .signature_position_row
            .set_model(Some(&gtk::StringList::new(&[
                &i18n("Above the quoted message"),
                &i18n("Below the quoted message"),
            ])));
        no_truncate(&widgets.composing.signature_position_row);
        widgets
            .composing
            .signature_position_row
            .set_selected(signature_position_index(init.signature_position));
        widgets
            .composing
            .signature_dashes_row
            .set_active(init.signature_dashes);
        widgets.composing.fold_quote_row.set_active(init.fold_quote);
        widgets.composing.set_reply_header(&init.reply_header);
        let undo_labels: Vec<String> = crate::config::UNDO_SEND_CHOICES
            .iter()
            .map(|&secs| match secs {
                0 => i18n("Off"),
                n => ni18n_f("{n} second", "{n} seconds", n, &[("n", &n.to_string())]),
            })
            .collect();
        let undo_labels: Vec<&str> = undo_labels.iter().map(String::as_str).collect();
        widgets
            .composing
            .undo_send_row
            .set_model(Some(&gtk::StringList::new(&undo_labels)));
        no_truncate(&widgets.composing.undo_send_row);
        // A delay set by hand in the file shows as the nearest one offered.
        let nearest = crate::config::UNDO_SEND_CHOICES
            .iter()
            .enumerate()
            .min_by_key(|(_, &secs)| secs.abs_diff(init.undo_send_secs))
            .map_or(0, |(i, _)| i as u32);
        widgets.composing.undo_send_row.set_selected(nearest);
        widgets.composing.spellcheck_row.set_active(init.spellcheck);
        // The language switches offer exactly what checking can use: the
        // installed dictionaries, behind a "System language" default. Typed
        // codes are gone — a language nobody has a dictionary for silently
        // checks nothing, so only real options are offered (#114). Any
        // number of them can be on at once (#365).
        {
            let rows = SpellLangRows::new(
                widgets.composing.spell_lang_row.clone(),
                init.spellcheck_langs.clone(),
            );
            self.spell_langs = Some(rows.clone());
            // Listing the installed dictionaries reads several directories;
            // done a moment after the window is up, not before it.
            let spelling_group = widgets.composing.spelling_group.clone();
            let sender = sender.clone();
            gtk::glib::idle_add_local_full(gtk::glib::Priority::LOW, move || {
                let dicts = crate::desktop::spell::installed_dictionaries();
                let system = adw::SwitchRow::builder()
                    .title(i18n("System language"))
                    .subtitle(gtk::glib::markup_escape_text(
                        &crate::desktop::spell::spell_language_names(
                            &crate::desktop::spell::system_spell_languages(),
                        ),
                    ))
                    .build();
                rows.expander.add_row(&system);
                *rows.system.borrow_mut() = Some(system.clone());
                for d in &dicts {
                    let row = adw::SwitchRow::builder()
                        .title(gtk::glib::markup_escape_text(
                            &crate::desktop::spell::language_display_name(d),
                        ))
                        .build();
                    rows.expander.add_row(&row);
                    rows.dicts.borrow_mut().push((d.clone(), row));
                }
                let current = rows.setting.borrow().clone();
                rows.show(&current);
                if dicts.is_empty() {
                    rows.expander.set_sensitive(false);
                    spelling_group.set_description(Some(&i18n(
                        "No dictionaries found. On Flatpak, add a language with: flatpak config --set extra-languages <code>",
                    )));
                }
                // Connected after the switches show the saved choice, and
                // guarded besides, so showing it never saves it again.
                let switches = std::iter::once((String::new(), system))
                    .chain(rows.dicts.borrow().iter().cloned())
                    .collect::<Vec<_>>();
                for (code, row) in switches {
                    let (rows, s) = (rows.clone(), sender.clone());
                    row.connect_active_notify(move |row| {
                        if let Some(next) = rows.toggled(&code, row.is_active()) {
                            let _ = s.output(PrefOutput::SetSpellcheckLangs(next));
                        }
                    });
                }
                gtk::glib::ControlFlow::Break
            });
        }
        {
            // The learned-words list manages itself entirely in widget-land:
            // the rows call the spell module directly and rebuild in place.
            let exp = widgets.composing.spell_words_row.clone();
            let add = adw::EntryRow::builder().title(i18n("Add a word")).build();
            add.set_show_apply_button(true);
            {
                let exp = exp.clone();
                add.connect_apply(move |row| {
                    crate::desktop::spell::add_personal_word(&row.text());
                    row.set_text("");
                    rebuild_personal_words(&exp);
                });
            }
            exp.add_row(&add);
            rebuild_personal_words(&exp);
        }
    }
}
