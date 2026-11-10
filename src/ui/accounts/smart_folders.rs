//! Smart folders (#417): saved searches, their list and their editor.

use super::criteria::{form_scaffold, Criteria, FieldSpec};
use super::*;

impl AccountsWindow {
    /// The Smart Folders list (#417): a row per folder, saying where it
    /// looks, activating to edit.
    pub(super) fn rebuild_smart_rows(&self, sender: &ComponentSender<Self>) {
        self.send_counts(sender);
        let Some(list) = &self.smart_list else { return };
        while let Some(row) = list.first_child() {
            list.remove(&row);
        }
        for (i, f) in self.smart_folders.iter().enumerate() {
            let row = adw::ActionRow::new();
            row.set_activatable(true);
            let s = sender.clone();
            row.connect_activated(move |_| {
                s.input_sender().emit(AccountsInput::EditSmartFolder(i))
            });
            row.set_title(&gtk::glib::markup_escape_text(&f.name));
            let account = if f.account_email.is_empty() {
                i18n("All accounts")
            } else {
                f.account_email.clone()
            };
            let folder = f
                .folder
                .map_or_else(|| i18n("All but trash and junk"), Self::smart_kind_label);
            row.set_subtitle(&gtk::glib::markup_escape_text(&format!(
                "{account} · {folder}"
            )));
            // Opens the editor, where it is deleted too (screens 39, 40).
            let next = gtk::Image::from_icon_name("go-next-symbolic");
            next.add_css_class("dim-label");
            next.set_accessible_role(gtk::AccessibleRole::Presentation);
            row.add_suffix(&next);
            list.append(&row);
        }
        let add = super::pages::add_row(&i18n("Add smart folder…"));
        let s = sender.clone();
        add.connect_activated(move |_| s.input_sender().emit(AccountsInput::AddSmartFolder));
        list.append(&add);
    }

    fn smart_kind_label(k: crate::config::SmartFolderKind) -> String {
        use crate::config::SmartFolderKind::*;
        match k {
            Inbox => i18n("Inbox"),
            Sent => i18n("Sent"),
            Drafts => i18n("Drafts"),
            Archive => i18n("Archive"),
            Junk => i18n("Junk"),
            Trash => i18n("Trash"),
        }
    }

    fn smart_field_label(f: crate::config::SmartField) -> String {
        use crate::config::SmartField::*;
        match f {
            Starred => i18n("Starred"),
            NotStarred => i18n("Not starred"),
            Unread => i18n("Unread"),
            Read => i18n("Read"),
            Attachment => i18n("Has attachments"),
            Tagged => i18n("Tagged"),
            Body => i18n("Message preview"),
            other => other
                .filter_field()
                .map(Self::field_label)
                .unwrap_or_default(),
        }
    }

    /// The smart folder editor (#417; screen 40): a name and where it
    /// looks; the criteria, first the fixed rows (Reading, Starred,
    /// Attachment, Tag), then any on the sender, subject or recipients;
    /// Advanced, the rare ones (the preview, Reply-To); and Delete when
    /// editing. With `edit` it opens on that folder and saves over it.
    pub(super) fn open_smart_page(
        &self,
        nav: &adw::NavigationView,
        sender: &ComponentSender<Self>,
        edit: Option<(usize, crate::config::SmartFolder)>,
    ) {
        use crate::config::{SmartCondition, SmartField, SmartFolder, SmartFolderKind};
        let (title, verb) = if edit.is_some() {
            (i18n("Edit smart folder"), i18n("Save"))
        } else {
            (i18n("Add smart folder"), i18n("Add smart folder"))
        };
        let emails: Vec<String> = self.accounts.iter().map(|a| a.email.clone()).collect();

        let (page_box, scrolled) = form_scaffold();

        let where_group = adw::PreferencesGroup::new();
        let name_row = adw::EntryRow::new();
        name_row.set_title(&i18n("Name"));
        where_group.add(&name_row);
        let account_row = adw::ComboRow::new();
        account_row.set_title(&i18n("Account"));
        {
            let all = i18n("All accounts");
            let mut names: Vec<&str> = vec![all.as_str()];
            names.extend(emails.iter().map(|e| e.as_str()));
            account_row.set_model(Some(&gtk::StringList::new(&names)));
        }
        account_row.set_subtitle(&i18n("Those in the unified section."));
        where_group.add(&account_row);
        let folder_row = adw::ComboRow::new();
        folder_row.set_title(&i18n("Folders"));
        {
            let mut names = vec![i18n("All but trash and junk")];
            names.extend(
                SmartFolderKind::ALL
                    .iter()
                    .map(|k| Self::smart_kind_label(*k)),
            );
            let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            folder_row.set_model(Some(&gtk::StringList::new(&refs)));
        }
        where_group.add(&folder_row);

        // The text criteria: what a smart folder shares with the filters.
        let text_fields = [
            SmartField::FromAddress,
            SmartField::FromName,
            SmartField::Subject,
            SmartField::Recipients,
            SmartField::FromInContacts,
            SmartField::FromNotInContacts,
            SmartField::Body,
            SmartField::ReplyTo,
        ];
        let specs: Vec<FieldSpec> = text_fields
            .iter()
            .map(|f| FieldSpec {
                label: Self::smart_field_label(*f),
                text: f.takes_text(),
                contains_only: false,
            })
            .collect();
        let at = move |f: SmartField| text_fields.iter().position(|x| *x == f);
        let enter: std::rc::Rc<dyn Fn()> = {
            let form_save = self.form_save.clone();
            let nav = nav.clone();
            std::rc::Rc::new(move || {
                let saved = form_save.borrow().as_ref().is_some_and(|save| save());
                if saved {
                    nav.pop();
                }
            })
        };
        let criteria = Criteria::new(
            &i18n("Criteria"),
            None,
            specs,
            (0..6).collect(),
            std::rc::Rc::new(|| {}),
            enter,
        );

        // The fixed rows (screen 40), each "Any" or one state.
        let choice = |title: &str, options: &[String]| {
            let row = adw::ComboRow::new();
            row.set_title(title);
            let refs: Vec<&str> = options.iter().map(String::as_str).collect();
            row.set_model(Some(&gtk::StringList::new(&refs)));
            row
        };
        let any = i18n("Any");
        let reading = choice(
            &i18n("Reading"),
            &[any.clone(), i18n("Unread"), i18n("Read")],
        );
        let starred = choice(
            &i18n("Starred"),
            &[any.clone(), i18n("Starred"), i18n("Not starred")],
        );
        let attachment = choice(
            &i18n("Attachment"),
            &[any.clone(), i18n("With attachments")],
        );
        let tags = self.tags.clone();
        let tag_names: Vec<String> = [any.clone(), i18n("Any tag")]
            .into_iter()
            .chain(tags.iter().map(|t| t.name.clone()))
            .collect();
        let tag = choice(&i18n("Tag"), &tag_names);
        {
            let rows = [
                reading.clone(),
                starred.clone(),
                attachment.clone(),
                tag.clone(),
            ];
            let count = std::rc::Rc::new(move || rows.iter().filter(|r| r.selected() != 0).count());
            let widgets: Vec<gtk::Widget> = [&reading, &starred, &attachment, &tag]
                .iter()
                .map(|r| (*r).clone().upcast())
                .collect();
            criteria.set_fixed(&widgets, count);
            for row in [&reading, &starred, &attachment, &tag] {
                let criteria = criteria.clone();
                row.connect_selected_notify(move |_| criteria.refresh());
            }
        }

        // Advanced: the preview text and Reply-To.
        let advanced_group = adw::PreferencesGroup::new();
        let advanced = adw::ExpanderRow::new();
        advanced.set_title(&i18n("Advanced"));
        advanced.set_subtitle(&i18n("Preview text, From address without Reply-To"));
        let rare: Vec<usize> = [SmartField::Body, SmartField::ReplyTo]
            .into_iter()
            .filter_map(at)
            .collect();
        advanced
            .add_row(&criteria.add_row(&i18n("Add a criterion on the preview or Reply-To…"), rare));
        advanced_group.add(&advanced);

        let edit_index = edit.as_ref().map(|(i, _)| *i);
        let id = edit
            .as_ref()
            .map_or_else(crate::config::new_smart_folder_id, |(_, f)| f.id.clone());
        if let Some((_, f)) = &edit {
            name_row.set_text(&f.name);
            if let Some(i) = emails
                .iter()
                .position(|e| e.eq_ignore_ascii_case(&f.account_email))
            {
                account_row.set_selected(i as u32 + 1);
            }
            if let Some(i) = f
                .folder
                .and_then(|k| SmartFolderKind::ALL.iter().position(|x| *x == k))
            {
                folder_row.set_selected(i as u32 + 1);
            }
            criteria.set_any(f.any);
            // Each state into its row (the first of each kind, should a
            // folder hold two), the text criteria into the list.
            for c in &f.conditions {
                match c.field {
                    SmartField::Unread if reading.selected() == 0 => reading.set_selected(1),
                    SmartField::Read if reading.selected() == 0 => reading.set_selected(2),
                    SmartField::Starred if starred.selected() == 0 => starred.set_selected(1),
                    SmartField::NotStarred if starred.selected() == 0 => starred.set_selected(2),
                    SmartField::Attachment => attachment.set_selected(1),
                    SmartField::Tagged if tag.selected() == 0 => {
                        let i = tags
                            .iter()
                            .position(|t| t.keyword.eq_ignore_ascii_case(&c.value))
                            .map_or(1, |i| i + 2);
                        tag.set_selected(i as u32);
                    }
                    field => {
                        if let Some(i) = at(field) {
                            criteria.insert(i, c.matcher, &c.value);
                        }
                    }
                }
            }
            advanced.set_expanded(
                f.conditions
                    .iter()
                    .any(|c| matches!(c.field, SmartField::Body | SmartField::ReplyTo)),
            );
        }

        page_box.append(&where_group);
        page_box.append(&criteria.group);
        page_box.append(&advanced_group);

        // Deleting is done here, the one place a smart folder is edited.
        if let Some(i) = edit_index {
            let group = adw::PreferencesGroup::new();
            let delete = super::pages::action_row(
                &i18n("Delete smart folder"),
                Some("user-trash-symbolic"),
                "error",
            );
            let s = sender.clone();
            let nav = nav.clone();
            delete.connect_activated(move |_| {
                s.input_sender().emit(AccountsInput::RemoveSmartFolder(i));
                nav.pop();
            });
            group.add(&delete);
            page_box.append(&group);
        }

        let s = sender.clone();
        self.push_form_content(
            nav,
            "smart",
            &title,
            &verb,
            scrolled.upcast_ref(),
            vec![name_row.clone()],
            move || {
                let name = name_row.text().trim().to_string();
                let mut conditions = Vec::new();
                let state = |field: SmartField| SmartCondition {
                    field,
                    matcher: crate::config::FilterMatch::Contains,
                    value: String::new(),
                };
                match reading.selected() {
                    1 => conditions.push(state(SmartField::Unread)),
                    2 => conditions.push(state(SmartField::Read)),
                    _ => {}
                }
                match starred.selected() {
                    1 => conditions.push(state(SmartField::Starred)),
                    2 => conditions.push(state(SmartField::NotStarred)),
                    _ => {}
                }
                if attachment.selected() == 1 {
                    conditions.push(state(SmartField::Attachment));
                }
                match tag.selected() {
                    0 => {}
                    1 => conditions.push(state(SmartField::Tagged)),
                    i => conditions.push(SmartCondition {
                        value: tags
                            .get(i as usize - 2)
                            .map(|t| t.keyword.clone())
                            .unwrap_or_default(),
                        ..state(SmartField::Tagged)
                    }),
                }
                // A text criterion with no text says nothing and is
                // dropped; the contact checks need none.
                for (i, matcher, value) in criteria.read() {
                    let Some(field) = text_fields.get(i).copied() else {
                        continue;
                    };
                    if field.takes_text() && value.is_empty() {
                        continue;
                    }
                    conditions.push(SmartCondition {
                        field,
                        matcher,
                        value,
                    });
                }
                if name.is_empty() || conditions.is_empty() {
                    return false;
                }
                let account_email = match account_row.selected() {
                    0 => String::new(),
                    i => emails.get(i as usize - 1).cloned().unwrap_or_default(),
                };
                let folder = match folder_row.selected() {
                    0 => None,
                    i => SmartFolderKind::ALL.get(i as usize - 1).copied(),
                };
                let folder = SmartFolder {
                    id: id.clone(),
                    name,
                    account_email,
                    folder,
                    any: criteria.any(),
                    conditions,
                };
                s.input_sender()
                    .emit(AccountsInput::SmartFolderSaved(edit_index, folder));
                true
            },
        );
    }
}

impl AccountsWindow {
    pub(super) fn on_edit_smart_folder(
        &mut self,
        i: usize,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        if let Some(f) = self.smart_folders.get(i).cloned() {
            self.open_smart_page(&widgets.nav, &sender, Some((i, f)));
        }
    }

    pub(super) fn on_remove_smart_folder(&mut self, i: usize, sender: ComponentSender<Self>) {
        if i < self.smart_folders.len() {
            self.smart_folders.remove(i);
            self.rebuild_smart_rows(&sender);
            let _ = sender.output(AccountsOutput::SetSmartFolders(self.smart_folders.clone()));
        }
    }

    pub(super) fn on_smart_folder_saved(
        &mut self,
        at: Option<usize>,
        folder: crate::config::SmartFolder,
        sender: ComponentSender<Self>,
    ) {
        match at.filter(|i| *i < self.smart_folders.len()) {
            Some(i) => self.smart_folders[i] = folder,
            None => self.smart_folders.push(folder),
        }
        self.rebuild_smart_rows(&sender);
        let _ = sender.output(AccountsOutput::SetSmartFolders(self.smart_folders.clone()));
    }

    pub(super) fn on_set_smart_folders(
        &mut self,
        folders: Vec<crate::config::SmartFolder>,
        sender: ComponentSender<Self>,
    ) {
        if folders != self.smart_folders {
            self.smart_folders = folders;
            self.rebuild_smart_rows(&sender);
        }
    }
}
