//! The list of accounts (drag to reorder) and the GNOME Online Accounts
//! that can still be imported.

use super::form::display_name;
use super::*;

/// GNOME Online Accounts mail accounts not (properly) configured in Tegami Mail.
/// A configured entry counts when it has an IMAP host or runs over Graph — an
/// entry with neither is a broken pre-#36 Microsoft 365 import, so its GOA
/// account is offered again and re-importing repairs it. Accounts GOA can
/// neither serve IMAP nor Graph mail for can never connect and aren't listed.
pub(super) fn importable_goa_accounts(
    configured: &[AccountConfig],
) -> Vec<crate::services::goa::GoaMailAccount> {
    importable_of(crate::services::goa::list_mail_accounts(), configured)
}

/// The importable ones among `listed` (see [`importable_goa_accounts`]).
fn importable_of(
    listed: Vec<crate::services::goa::GoaMailAccount>,
    configured: &[AccountConfig],
) -> Vec<crate::services::goa::GoaMailAccount> {
    listed
        .into_iter()
        .filter(|g| {
            !configured.iter().any(|a| {
                a.email.eq_ignore_ascii_case(&g.email)
                    && (!a.imap_host.trim().is_empty()
                        || a.protocol == crate::config::Protocol::Graph)
            })
        })
        .filter(|g| g.can_connect())
        .collect()
}

impl AccountsWindow {
    /// Rebuild the draggable account list.
    pub(super) fn rebuild_account_list(&self, list: &gtk::ListBox, sender: &ComponentSender<Self>) {
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let mut css = String::new();
        // As screen 26 draws them: the circle, the name over the address
        // (and where the account comes from), the switch that pauses it,
        // and the chevron to its page. Dragged to reorder.
        for (pos, acc) in self.accounts.iter().enumerate() {
            let row = gtk::ListBoxRow::new();
            row.set_activatable(true);

            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            hbox.add_css_class("account-list-row");
            if acc.is_local() {
                hbox.append(&local_circle());
            } else {
                hbox.append(&account_circle(acc, pos as u32 + 1, &mut css));
            }
            hbox.append(&account_titles(acc));
            if !acc.is_local() {
                hbox.append(&enable_switch(acc, pos, sender));
            }
            let next = gtk::Image::from_icon_name("go-next-symbolic");
            next.add_css_class("dim-label");
            next.set_accessible_role(gtk::AccessibleRole::Presentation);
            hbox.append(&next);

            row.set_child(Some(&hbox));
            row.update_property(&[gtk::accessible::Property::Label(&acc.display_label())]);
            add_reorder(&row, pos, sender);
            list.append(&row);
        }
        self.list_css.load_from_string(&css);

        // Last, the way to add one (screen 27).
        let add = super::pages::add_row(&i18n("Add account…"));
        let s = sender.input_sender().clone();
        add.connect_activated(move |_| s.emit(AccountsInput::AddMenu));
        list.append(&add);
    }

    /// Un-import a GNOME Online Account: drop it from Tegami Mail (the app removes
    /// the config entry and its stored copies) and return it to the "GNOME
    /// Online Accounts" import list below. The account stays in GNOME.
    pub(super) fn unimport_goa(
        &mut self,
        index: usize,
        widgets: &AccountsWindowWidgets,
        sender: &ComponentSender<Self>,
    ) {
        if index >= self.accounts.len() {
            return;
        }
        let email = self.accounts[index].email.clone();
        self.accounts.remove(index);
        let _ = sender.output(AccountsOutput::Removed { email });
        // Re-query GOA so the account's row reappears in the import list fresh.
        self.goa = importable_goa_accounts(&self.accounts);
        self.rebuild_account_list(&widgets.accounts.accounts_list, sender);
        self.rebuild_goa_list(&widgets.accounts.goa_list, sender);
        widgets.accounts.goa_group.set_visible(!self.goa.is_empty());
    }

    /// Populate the "GNOME Online Accounts" list with importable mail accounts.
    pub(super) fn rebuild_goa_list(&self, list: &gtk::ListBox, sender: &ComponentSender<Self>) {
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }
        for (pos, g) in self.goa.iter().enumerate() {
            let row = adw::ActionRow::new();
            row.set_title(&g.email);
            row.add_prefix(&crate::desktop::brand::mark(
                brand_for_goa(&g.provider),
                24,
                crate::desktop::brand::GENERIC_MAIL,
            ));
            let mut subtitle = if g.provider.is_empty() {
                i18n("Mail")
            } else {
                g.provider.clone()
            };
            // OAuth providers (Gmail, Microsoft 365) sign in with a token from
            // GNOME.
            if g.oauth2 && !g.password_based {
                subtitle.push_str(" · sign-in via GNOME");
            }
            row.set_subtitle(&subtitle);

            let toggle = gtk::Switch::new();
            toggle.set_valign(gtk::Align::Center);
            toggle.set_active(false);
            toggle.set_tooltip_text(Some(i18n("Use this account in Tegami Mail").as_str()));
            let ti = sender.input_sender().clone();
            let tpos = pos;
            toggle.connect_state_set(move |_, state| {
                if state {
                    let _ = ti.send(AccountsInput::ImportGoa(tpos));
                }
                gtk::glib::Propagation::Proceed
            });
            row.add_suffix(&toggle);
            list.append(&row);
        }
    }
}

impl AccountsWindow {
    pub(super) fn on_refresh_goa(
        &mut self,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        // Off the main thread: GOA answers over D-Bus, and a busy
        // goa-daemon is slow to.
        widgets.accounts.goa_refresh_btn.set_sensitive(false);
        sender.oneshot_command(async move {
            let listed = tokio::task::spawn_blocking(crate::services::goa::list_mail_accounts)
                .await
                .unwrap_or_default();
            AccountsCmd::GoaListed(listed)
        });
    }

    pub(super) fn on_move_row(
        &mut self,
        from: usize,
        to: usize,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        if from < self.accounts.len() {
            let acc = self.accounts.remove(from);
            let to = to.min(self.accounts.len());
            self.accounts.insert(to, acc);
            self.rebuild_account_list(&widgets.accounts.accounts_list, &sender);
            let emails = self.accounts.iter().map(|a| a.email.clone()).collect();
            let _ = sender.output(AccountsOutput::Reordered(emails));
        }
    }

    pub(super) fn on_toggle_enabled(
        &mut self,
        index: usize,
        enabled: bool,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        // A GNOME Online Account switched off here isn't paused — it is
        // un-imported: it drops out of Tegami Mail (the config entry and its
        // stored copies go) and returns to the "GNOME Online Accounts"
        // list below, ready to import again. The account itself stays
        // in GNOME untouched.
        if !enabled && self.accounts.get(index).is_some_and(|a| a.goa_id.is_some()) {
            self.unimport_goa(index, widgets, &sender);
            return;
        }
        if let Some(acc) = self.accounts.get_mut(index) {
            if acc.enabled != enabled {
                acc.enabled = enabled;
                let email = acc.email.clone();
                let _ = sender.output(AccountsOutput::EnabledChanged { email, enabled });
            }
        }
    }

    pub(super) fn on_import_goa(
        &mut self,
        index: usize,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        if let Some(g) = self.goa.get(index).cloned() {
            // Password-based providers: pull the password now. OAuth
            // providers: authenticate with a GOA token at connect time.
            // Either way the worker asks GOA again when it connects, so an
            // account still works if this read comes back empty.
            let (password, oauth) = if g.password_based {
                (
                    crate::services::goa::mail_passwords(&g.id)
                        .0
                        .unwrap_or_default(),
                    false,
                )
            } else {
                (String::new(), true)
            };
            let account = g.to_config(password, oauth);
            self.goa.remove(index);
            self.accounts.push(account.clone());
            self.rebuild_account_list(&widgets.accounts.accounts_list, &sender);
            self.rebuild_goa_list(&widgets.accounts.goa_list, &sender);
            widgets.accounts.goa_group.set_visible(!self.goa.is_empty());
            let _ = sender.output(AccountsOutput::ImportGoa(Box::new(account)));
        }
    }

    pub(super) fn on_goa_listed(
        &mut self,
        listed: Vec<crate::services::goa::GoaMailAccount>,
        widgets: &AccountsWindowWidgets,
        sender: ComponentSender<Self>,
    ) {
        widgets.accounts.goa_refresh_btn.set_sensitive(true);
        self.goa = importable_of(listed, &self.accounts);
        self.rebuild_goa_list(&widgets.accounts.goa_list, &sender);
        widgets.accounts.goa_group.set_visible(!self.goa.is_empty());
    }
}

/// On This Computer's circle: a folder on the neutral disc (screen 26).
fn local_circle() -> gtk::Box {
    let circle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    circle.add_css_class("account-circle");
    circle.add_css_class("account-local-circle");
    circle.set_valign(gtk::Align::Center);
    circle.set_size_request(30, 30);
    let icon = gtk::Image::from_icon_name("folder-symbolic");
    icon.set_hexpand(true);
    icon.set_halign(gtk::Align::Center);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    circle.append(&icon);
    circle
}

/// The account's circle, as the sidebar draws it: its color (or the
/// palette accent it would get) and its initials — of the label first, then
/// the name, then the address. Its color rule goes into `css`.
fn account_circle(acc: &AccountConfig, account_id: u32, css: &mut String) -> gtk::Box {
    let color = acc
        .color
        .clone()
        .unwrap_or_else(|| crate::worker::accent_for(account_id).to_string());
    // Ringed as in the sidebar, a pixel off the disc.
    css.push_str(&format!(
        ".acct-list-color-{account_id} {{ background-color: {color}; \
         outline: 2px solid {color}; outline-offset: 1px; }}\n"
    ));
    let circle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    circle.add_css_class("account-circle");
    circle.add_css_class(&format!("acct-list-color-{account_id}"));
    circle.set_valign(gtk::Align::Center);
    circle.set_halign(gtk::Align::Center);
    circle.set_hexpand(false);
    circle.set_size_request(30, 30);
    let shown = match acc.label.as_deref().map(str::trim) {
        Some(l) if !l.is_empty() && l != acc.email => l.to_string(),
        _ => display_name(acc),
    };
    // Always the initials on the account colour (docs/DESIGN.md,
    // "Preferences"), as the sidebar draws it.
    let glyph: gtk::Widget = crate::ui::initials::glyph_picture(
        &crate::ui::sidebar::account_initials(&shown, &acc.email),
        &color,
        0.47,
        30,
    )
    .upcast();
    circle.append(&glyph);
    circle
}

/// What the account goes by in the list and on its page's header: its
/// nickname, else its sender name, else its address (screens 26, 28).
pub(super) fn list_title(acc: &AccountConfig) -> String {
    match acc.label.as_deref().map(str::trim) {
        Some(l) if !l.is_empty() && l != acc.email => l.to_string(),
        _ if !acc.name.trim().is_empty() => acc.name.trim().to_string(),
        _ => acc.email.clone(),
    }
}

/// The account's name over its address, as screen 26 draws them, and
/// "GNOME Online Accounts" after the address when GNOME holds it.
fn account_titles(acc: &AccountConfig) -> gtk::Box {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.set_hexpand(true);
    vbox.set_valign(gtk::Align::Center);
    let title = list_title(acc);
    let mut second = if acc.is_local() {
        i18n("Mail saved in local folders")
    } else if title == acc.email {
        String::new()
    } else {
        acc.email.clone()
    };
    // Where it comes from, when GNOME holds it (screen 26).
    if acc.goa_id.is_some() {
        let goa = i18n("GNOME Online Accounts");
        second = if second.is_empty() {
            goa
        } else {
            format!("{second} \u{b7} {goa}")
        };
    }
    let name = gtk::Label::new(Some(&title));
    name.set_halign(gtk::Align::Start);
    name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let email = gtk::Label::new(Some(&second));
    email.set_halign(gtk::Align::Start);
    email.set_ellipsize(gtk::pango::EllipsizeMode::End);
    email.add_css_class("caption");
    email.add_css_class("dim-label");
    email.set_visible(!second.is_empty());
    vbox.append(&name);
    vbox.append(&email);
    vbox
}

/// Enable/disable toggle. Disabled accounts stay configured but don't sync
/// or appear in the sidebar.
fn enable_switch(
    acc: &AccountConfig,
    pos: usize,
    sender: &ComponentSender<AccountsWindow>,
) -> gtk::Switch {
    let toggle = gtk::Switch::new();
    toggle.set_valign(gtk::Align::Center);
    if acc.goa_mail_disabled {
        toggle.set_tooltip_text(Some(&i18n(
            "Paused: Mail is switched off for this account in GNOME Settings",
        )));
        toggle.set_sensitive(false);
    } else {
        toggle.set_tooltip_text(Some(i18n("Enable this account").as_str()));
    }
    toggle.set_active(acc.enabled);
    let ti = sender.input_sender().clone();
    let tpos = pos;
    toggle.connect_state_set(move |_, state| {
        let _ = ti.send(AccountsInput::ToggleEnabled {
            index: tpos,
            enabled: state,
        });
        gtk::glib::Propagation::Proceed
    });
    toggle
}

/// Drag a row to reorder the accounts.
fn add_reorder(row: &gtk::ListBoxRow, pos: usize, sender: &ComponentSender<AccountsWindow>) {
    let drag = gtk::DragSource::new();
    drag.set_actions(gtk::gdk::DragAction::MOVE);
    let from = pos as u32;
    drag.connect_prepare(move |_, _, _| {
        Some(gtk::gdk::ContentProvider::for_value(&from.to_value()))
    });
    row.add_controller(drag);

    let drop = gtk::DropTarget::new(gtk::glib::Type::U32, gtk::gdk::DragAction::MOVE);
    let to = pos;
    let input = sender.input_sender().clone();
    drop.connect_drop(move |_, value, _, _| {
        if let Ok(from) = value.get::<u32>() {
            let _ = input.send(AccountsInput::MoveRow {
                from: from as usize,
                to,
            });
            true
        } else {
            false
        }
    });
    row.add_controller(drop);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list names an account by its nickname, else its sender name,
    /// else its address (screens 26, 28); a nickname that is only the
    /// address does not count as one.
    #[test]
    fn accounts_are_listed_by_nickname_then_name_then_address() {
        let mut acc = crate::config::blank_account();
        acc.email = "jason@example.org".into();
        assert_eq!(list_title(&acc), "jason@example.org");
        acc.name = " Jason M. ".into();
        assert_eq!(list_title(&acc), "Jason M.");
        acc.label = Some("jason@example.org".into());
        assert_eq!(list_title(&acc), "Jason M.");
        acc.label = Some("Work".into());
        assert_eq!(list_title(&acc), "Work");
    }

    fn goa(
        email: &str,
        imap_host: &str,
        oauth2: bool,
        provider_type: &str,
    ) -> crate::services::goa::GoaMailAccount {
        crate::services::goa::GoaMailAccount {
            id: format!("account_{email}"),
            email: email.into(),
            name: String::new(),
            provider: String::new(),
            imap_host: imap_host.into(),
            imap_port: 993,
            imap_user: String::new(),
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_user: String::new(),
            smtp_separate: false,
            password_based: !oauth2,
            oauth2,
            provider_type: provider_type.into(),
            security: Default::default(),
        }
    }

    #[test]
    fn importable_goa_accounts_skip_configured_and_unconnectable_ones() {
        let listed = vec![
            goa("a@example.org", "imap.example.org", false, "imap_smtp"),
            goa("b@example.org", "imap.example.org", false, "imap_smtp"),
            goa("c@example.org", "", true, "ms_graph"),
            goa("d@example.org", "", true, "google"),
            goa("e@example.org", "imap.example.org", false, "imap_smtp"),
        ];
        let configured = vec![
            // Configured, case aside: not offered again.
            AccountConfig {
                email: "A@Example.org".into(),
                imap_host: "imap.example.org".into(),
                ..crate::config::blank_account()
            },
            // A broken import with neither an IMAP host nor Graph: offered again.
            AccountConfig {
                email: "b@example.org".into(),
                imap_host: "  ".into(),
                ..crate::config::blank_account()
            },
            // Over Graph with no IMAP host: configured.
            AccountConfig {
                email: "e@example.org".into(),
                protocol: Protocol::Graph,
                ..crate::config::blank_account()
            },
        ];
        let emails: Vec<String> = importable_of(listed, &configured)
            .into_iter()
            .map(|g| g.email)
            .collect();
        assert_eq!(emails, ["b@example.org", "c@example.org"]);
    }
}
