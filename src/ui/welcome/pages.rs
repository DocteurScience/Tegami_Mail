//! The pieces every wizard page is made of (docs/DESIGN.md, "First run"):
//! a navigation page with its header bar, a centred column, its heading,
//! its sentence and its pill button. Then the welcome page (screen 49) and
//! the last one (52).

use super::*;

/// The column's width, as on screens 49 to 52.
const COLUMN: i32 = 456;

/// A navigation page titled `title` holding `content`: the header bar
/// brings the back button (where the page can be left) and the close
/// button; the content scrolls when the window is short.
pub(super) fn nav_page(tag: &str, title: &str, content: &gtk::Box) -> adw::NavigationPage {
    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(COLUMN);
    clamp.set_tightening_threshold(COLUMN);
    clamp.set_child(Some(content));
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_propagate_natural_height(true);
    scrolled.set_vexpand(true);
    scrolled.set_child(Some(&clamp));
    let header = adw::HeaderBar::new();
    header.set_show_title(false);
    let view = adw::ToolbarView::new();
    view.set_top_bar_style(adw::ToolbarStyle::RaisedBorder);
    view.add_top_bar(&header);
    view.set_content(Some(&scrolled));
    adw::NavigationPage::with_tag(&view, title, tag)
}

/// A page's column: centred, with the margins of the mockups.
pub(super) fn column() -> gtk::Box {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 12);
    b.set_valign(gtk::Align::Center);
    b.set_margin_top(24);
    b.set_margin_bottom(24);
    b.set_margin_start(16);
    b.set_margin_end(16);
    b
}

pub(super) fn heading(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.add_css_class("title-1");
    l.set_wrap(true);
    l.set_justify(gtk::Justification::Center);
    l
}

/// The sentence under a heading, in the readable secondary shade.
pub(super) fn sentence(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.add_css_class("body");
    l.add_css_class("secondary-text");
    l.set_wrap(true);
    l.set_justify(gtk::Justification::Center);
    l.set_max_width_chars(44);
    l
}

/// The page's one suggested button, as a pill.
pub(super) fn pill(label: &str) -> gtk::Button {
    let b = gtk::Button::with_label(label);
    b.add_css_class("pill");
    b.add_css_class("suggested-action");
    b.set_halign(gtk::Align::Center);
    b.set_margin_top(12);
    b
}

/// The app icon, at `size` px.
fn app_icon(size: i32) -> gtk::Image {
    crate::ui::wordmark::app_icon_image(size)
}

/// Screen 49: the app icon, a greeting, one sentence and "Get started".
/// No language: the system's is the app's.
pub(super) fn build_welcome_page(sender: &ComponentSender<Welcome>) -> adw::NavigationPage {
    let col = column();
    let icon = app_icon(128);
    icon.set_margin_bottom(12);
    col.append(&icon);
    col.append(&heading(&i18n("Welcome to Tegami Mail")));
    col.append(&sentence(&i18n(
        "Your mail, in one place, simply. Let’s start by connecting your first account.",
    )));
    let start = pill(&i18n("Get started"));
    {
        let s = sender.clone();
        start.connect_clicked(move |_| s.input(WelcomeInput::Start));
    }
    col.append(&start);
    nav_page("welcome", &i18n("Welcome to Tegami Mail"), &col)
}

/// Screen 52: the app icon, "You're all set", and "Start reading". The
/// page can't be left backwards: what was added is added.
pub(super) fn build_done_page(
    sender: &ComponentSender<Welcome>,
) -> (adw::NavigationPage, gtk::Label) {
    let col = column();
    let icon = app_icon(96);
    icon.set_margin_bottom(12);
    col.append(&icon);
    col.append(&heading(&i18n("You’re all set")));
    let text = sentence("");
    col.append(&text);
    let read = pill(&i18n("Start reading"));
    {
        let s = sender.clone();
        read.connect_clicked(move |_| s.input(WelcomeInput::Finish));
    }
    col.append(&read);
    let page = nav_page("done", &i18n("You’re all set"), &col);
    page.set_can_pop(false);
    (page, text)
}
