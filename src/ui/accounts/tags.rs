//! Tags (#71): the list (mockup 09), the New and Edit Tag dialogs
//! (`ui::tag_dialog`) and the tag finder.

use super::*;

impl AccountsWindow {
    /// The tag finder's button while a scan runs: a turning spinner beside
    /// a label saying so. A press meanwhile starts nothing.
    pub(super) fn set_tag_scanning(&mut self, on: bool) {
        self.tag_scanning = on;
        if let Some(spinner) = &self.find_tags_spinner {
            spinner.set_visible(on);
            spinner.set_spinning(on);
        }
        if let Some(row) = &self.find_tags_row {
            row.set_title(&if on {
                i18n("Searching…")
            } else {
                i18n("Import tags from the servers…")
            });
        }
    }

    /// The tag finder's report: every keyword found that is not a tag here
    /// yet, each with a proposed name and color, ticked to be imported.
    /// "Import All" takes the lot; "Import Selected" only the ticked ones.
    #[allow(deprecated)] // adw::MessageDialog: AlertDialog would change how it looks
    fn show_tag_findings(
        &self,
        parent: Option<&gtk::Window>,
        found: Vec<TagProposal>,
        sender: &ComponentSender<Self>,
    ) {
        if found.is_empty() {
            let dialog = adw::MessageDialog::new(
                parent,
                Some(i18n("No new tags").as_str()),
                Some(i18n("Every tag in your mailboxes is already set up here.").as_str()),
            );
            dialog.add_response("ok", &i18n("OK"));
            dialog.set_default_response(Some("ok"));
            dialog.set_close_response("ok");
            dialog.present();
            return;
        }

        let n = found.len();
        let dialog = adw::MessageDialog::new(
            parent,
            Some(i18n("Tags found").as_str()),
            Some(
                i18n_f(
                    "{n} tags are in use in your mailboxes but not set up here.",
                    &[("n", &n.to_string())],
                )
                .as_str(),
            ),
        );
        dialog.add_response("cancel", &i18n("Cancel"));
        dialog.add_response("selected", &i18n("Import selected"));
        dialog.add_response("all", &i18n("Import all"));
        dialog.set_response_appearance("all", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("all"));
        dialog.set_close_response("cancel");

        let list = gtk::ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(gtk::SelectionMode::None);
        let mut checks: Vec<(gtk::CheckButton, crate::config::Tag)> = Vec::new();
        for p in &found {
            let row = adw::ActionRow::new();
            row.set_title(&gtk::glib::markup_escape_text(&p.tag.name));
            let mut parts = vec![p.tag.keyword.clone()];
            if p.count > 0 {
                parts.push(i18n_f("{n} messages", &[("n", &p.count.to_string())]));
            }
            parts.extend(p.places.iter().cloned());
            row.set_subtitle(&gtk::glib::markup_escape_text(&parts.join(" · ")));
            row.add_prefix(&crate::ui::context_menu::swatch_widget(&p.tag.color, true));
            let check = gtk::CheckButton::new();
            check.set_active(true);
            check.set_valign(gtk::Align::Center);
            row.add_suffix(&check);
            row.set_activatable_widget(Some(&check));
            checks.push((check, p.tag.clone()));
            list.append(&row);
        }
        let scroller = gtk::ScrolledWindow::new();
        scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroller.set_propagate_natural_height(true);
        scroller.set_max_content_height(360);
        // Wider than the dialog's default column, so a row's keyword, count
        // and folders read on a line or two.
        scroller.set_size_request(520, -1);
        scroller.set_child(Some(&list));
        dialog.set_extra_child(Some(&scroller));

        // The report is the showcase's to capture (TEGAMI_SHOWCASE_FIND_TAGS).
        if std::env::var("TEGAMI_SHOWCASE_FIND_TAGS").is_ok() {
            if let Ok(path) = std::env::var("TEGAMI_SHOWCASE") {
                let d = dialog.clone();
                gtk::glib::timeout_add_seconds_local_once(2, move || {
                    crate::ui::showcase::showcase_capture(d.upcast_ref(), &path);
                });
            }
        }

        let s = sender.clone();
        dialog.connect_response(None, move |_, resp| {
            let tags: Vec<crate::config::Tag> = match resp {
                "all" => checks.iter().map(|(_, t)| t.clone()).collect(),
                "selected" => checks
                    .iter()
                    .filter(|(c, _)| c.is_active())
                    .map(|(_, t)| t.clone())
                    .collect(),
                _ => return,
            };
            if !tags.is_empty() {
                s.input_sender().emit(AccountsInput::ImportTags(tags));
            }
        });
        dialog.present();
    }

    /// The Tags list (#71), as mockup 09 draws it: a row per tag — its
    /// colour as a dot, its name, how many messages carry it and the
    /// colour's name — with Edit and Remove buttons, then a row that makes
    /// a new one. The order is the order made in: the sidebar's, and the
    /// 1–9 keys' (#157).
    pub(super) fn rebuild_tag_rows(&self, sender: &ComponentSender<Self>) {
        let Some(list) = &self.tags_list else { return };
        while let Some(row) = list.first_child() {
            list.remove(&row);
        }
        for (i, t) in self.tags.iter().enumerate() {
            let row = adw::ActionRow::new();
            row.set_title(&gtk::glib::markup_escape_text(&t.name));
            let count = self.tag_totals.get(&t.keyword).map(|&n| {
                ni18n_f(
                    "{count} message",
                    "{count} messages",
                    n,
                    &[("count", &n.to_string())],
                )
            });
            let color = crate::ui::tag_dialog::color_name(&t.color);
            let subtitle = match (count, color) {
                (Some(count), Some(color)) => {
                    i18n_f("{count} · {color}", &[("count", &count), ("color", &color)])
                }
                (Some(only), None) | (None, Some(only)) => only,
                (None, None) => String::new(),
            };
            row.set_subtitle(&gtk::glib::markup_escape_text(&subtitle));
            let dot = crate::ui::context_menu::swatch_widget(&t.color, true);
            dot.set_content_width(12);
            dot.set_content_height(12);
            dot.set_margin_start(4);
            dot.set_margin_end(4);
            row.add_prefix(&dot);
            let edit = gtk::Button::from_icon_name("document-edit-symbolic");
            let s = sender.clone();
            edit.connect_clicked(move |_| s.input_sender().emit(AccountsInput::EditTag(i)));
            let remove = gtk::Button::from_icon_name("user-trash-symbolic");
            let s = sender.clone();
            remove.connect_clicked(move |_| s.input_sender().emit(AccountsInput::RemoveTag(i)));
            for (button, label) in [(edit, i18n("Edit tag")), (remove, i18n("Remove tag"))] {
                button.add_css_class("flat");
                button.set_valign(gtk::Align::Center);
                button.set_tooltip_text(Some(&label));
                button.update_property(&[gtk::accessible::Property::Label(&label)]);
                row.add_suffix(&button);
            }
            list.append(&row);
        }
        // The last row makes a tag, as the mockup's "+ New tag…".
        let add = adw::ButtonRow::new();
        add.set_title(&i18n("New tag…"));
        add.set_start_icon_name(Some("list-add-symbolic"));
        let s = sender.clone();
        add.connect_activated(move |_| s.input_sender().emit(AccountsInput::AddTag));
        list.append(&add);
    }
}

impl AccountsWindow {
    /// The New Tag dialog (mockup 09), on the first colour the tags
    /// leave free.
    pub(super) fn on_add_tag(
        &mut self,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        let taken: Vec<String> = self.tags.iter().map(|t| t.color.clone()).collect();
        let color = crate::config::Tag::color_for_keyword("", &taken);
        let s = sender.clone();
        crate::ui::tag_dialog::show(&widgets.nav, &color, move |name, color| {
            s.input_sender()
                .emit(AccountsInput::TagAdded(crate::config::Tag {
                    name,
                    keyword: String::new(),
                    color,
                }));
        });
    }

    /// The same dialog on tag `i`, with Save. The keyword stays: messages
    /// carry it, not the name.
    pub(super) fn on_edit_tag(
        &mut self,
        i: usize,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        let Some(tag) = self.tags.get(i).cloned() else {
            return;
        };
        let s = sender.clone();
        crate::ui::tag_dialog::edit(&widgets.nav, &tag.name, &tag.color, move |name, color| {
            s.input_sender().emit(AccountsInput::TagEdited(
                i,
                crate::config::Tag {
                    name,
                    keyword: tag.keyword.clone(),
                    color,
                },
            ));
        });
    }

    pub(super) fn on_remove_tag(&mut self, i: usize, sender: ComponentSender<Self>) {
        if i < self.tags.len() {
            self.tags.remove(i);
            self.rebuild_tag_rows(&sender);
            // The rules name tags by keyword; a rule whose tag is gone
            // shows the bare keyword until it is edited.
            self.rebuild_filter_rows(&sender);
            let _ = sender.output(AccountsOutput::SetTags(self.tags.clone()));
        }
    }

    /// A tag from the New Tag dialog, its keyword made from its name and
    /// kept apart from the other tags'.
    pub(super) fn on_tag_added(
        &mut self,
        mut tag: crate::config::Tag,
        sender: ComponentSender<Self>,
    ) {
        if tag.keyword.is_empty() {
            tag.keyword = crate::config::Tag::unique_keyword(&tag.name, &self.tags);
        }
        self.tags.push(tag);
        self.rebuild_tag_rows(&sender);
        let _ = sender.output(AccountsOutput::SetTags(self.tags.clone()));
    }

    pub(super) fn on_tag_edited(
        &mut self,
        i: usize,
        tag: crate::config::Tag,
        sender: ComponentSender<Self>,
    ) {
        if let Some(slot) = self.tags.get_mut(i) {
            if *slot != tag {
                *slot = tag;
                self.rebuild_tag_rows(&sender);
                self.rebuild_filter_rows(&sender);
                let _ = sender.output(AccountsOutput::SetTags(self.tags.clone()));
            }
        }
    }

    pub(super) fn on_find_tags(&mut self, sender: ComponentSender<Self>) {
        if !self.tag_scanning {
            self.set_tag_scanning(true);
            let _ = sender.output(AccountsOutput::FindTags);
        }
    }

    pub(super) fn on_tag_findings(
        &mut self,
        found: Vec<TagProposal>,
        sender: ComponentSender<Self>,
        root: &adw::Bin,
    ) {
        self.set_tag_scanning(false);
        let parent = root.root().and_downcast::<gtk::Window>();
        self.show_tag_findings(parent.as_ref(), found, &sender);
    }

    pub(super) fn on_import_tags(
        &mut self,
        tags: Vec<crate::config::Tag>,
        sender: ComponentSender<Self>,
    ) {
        let mut added = 0;
        for t in tags {
            if !self
                .tags
                .iter()
                .any(|x| x.keyword.eq_ignore_ascii_case(&t.keyword))
            {
                self.tags.push(t);
                added += 1;
            }
        }
        if added > 0 {
            self.rebuild_tag_rows(&sender);
            self.rebuild_filter_rows(&sender);
            let _ = sender.output(AccountsOutput::SetTags(self.tags.clone()));
        }
    }
}
