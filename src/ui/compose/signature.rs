//! Signatures: which one, where it goes, and swapping one for another.

use super::messages::ComposeInput;
use super::types::{ComposeAccount, SigPlace};
use super::Compose;
use super::ComposeWidgets;
use crate::config::SignaturePosition;
use crate::i18n::i18n;
use crate::ui::context_menu::{show_context_menu, MenuEntry};
use crate::ui::rich_editor;
use adw::prelude::*;
use relm4::prelude::*;

/// A signature block as HTML, under a `-- ` line when `dashes` is set. The
/// stored signature is HTML (legacy plain-text signatures are converted).
pub(super) fn sig_html(sig: &str, dashes: bool) -> String {
    let body = rich_editor::signature_to_html(sig);
    let dashes = if dashes { "-- <br>" } else { "" };
    format!("<div class=\"tegami-sig\"><br>{dashes}{body}</div>")
}

impl Compose {
    /// The signature Insert Signature puts in: the one the message last
    /// had, if it is one of `account`'s, else its default, else its first.
    fn chosen_signature(&self, account: &ComposeAccount) -> String {
        if account
            .signatures
            .iter()
            .any(|s| s.html == self.current_sig)
        {
            return self.current_sig.clone();
        }
        if !account.signature.is_empty() {
            return account.signature.clone();
        }
        account
            .signatures
            .first()
            .map(|s| s.html.clone())
            .unwrap_or_default()
    }

    /// Put `new_sig` in the editor in place of the current signature block,
    /// or where the setting puts a signature when there is none.
    pub(super) fn put_signature(&mut self, new_sig: String) {
        {
            let replacement = if new_sig.is_empty() {
                String::new()
            } else {
                sig_html(&new_sig, self.sig_dashes)
            };
            // No signature block to replace (the previous account
            // had none): a new one goes where the setting puts it,
            // above the quoted original or at the end (#237).
            let anchor = self.rich_sig_anchor();
            let js = format!(
                "(function(){{var s=[].filter.call(document.querySelectorAll('.tegami-sig'),\
                 function(x){{return !x.closest('blockquote');}})[0];\
                 var h='{}';\
                 if(s){{if(h){{s.outerHTML=h;}}else{{s.remove();}}}}\
                 else if(h){{{anchor}}}\
                 window.__tegamiPlainFix&&window.__tegamiPlainFix();}})()",
                rich_editor::js_escape(&replacement)
            );
            self.editor.run_js(&js);
            self.current_sig = new_sig;
        }
    }

    /// Rich-mode JS that puts the HTML `h` where the setting puts a
    /// signature: above the quoted original, or at the end (#237).
    fn rich_sig_anchor(&self) -> &'static str {
        match self.signature_position {
            SignaturePosition::AboveQuote => {
                "var b=document.body;var q=null;\
                 var cands=b.querySelectorAll('.tegami-quote-attr,blockquote');\
                 for(var i=0;i<cands.length;i++){var t=cands[i];\
                 while(t.parentNode&&t.parentNode!==b)t=t.parentNode;\
                 if(t.parentNode===b&&(!q||(t.compareDocumentPosition(q)&Node.DOCUMENT_POSITION_FOLLOWING)))q=t;}\
                 if(q){q.insertAdjacentHTML('beforebegin',h);}else{b.insertAdjacentHTML('beforeend',h);}"
            }
            SignaturePosition::BelowQuote => "document.body.insertAdjacentHTML('beforeend',h);",
        }
    }

    /// Insert Signature (#385): `sig` at the caret, or at the top or the
    /// end of the message. The message's own signature block moves rather
    /// than doubling; one inside a quote is the other person's and stays.
    fn insert_signature(&mut self, sig: String, place: SigPlace) {
        if sig.is_empty() {
            return;
        }
        let place_js = match place {
            SigPlace::Cursor => "cursor",
            SigPlace::Start => "start",
            SigPlace::End => "end",
        };
        // At the caret the line it sits on is split, so the signature
        // stands as a block of its own between the two halves; an empty
        // line gives up its place to it.
        let js = r#"(function(){var b=document.body,h=@SIG@,where=@WHERE@;
var olds=[].filter.call(b.querySelectorAll('.tegami-sig'),function(s){return !s.closest('blockquote');});
var sel=window.getSelection(),r=null;
if(where==='cursor'&&sel.rangeCount){r=sel.getRangeAt(0).cloneRange();r.collapse(false);if(!b.contains(r.startContainer)||r.startContainer===b&&b.childNodes.length===0)r=null;}
if(r){for(var k=0;k<olds.length;k++){if(olds[k].contains(r.startContainer)){r.setStartBefore(olds[k]);r.collapse(true);}}}
var spot=null;
if(r){var top=r.startContainer;
 if(top===b){spot=b.childNodes[r.startOffset]||null;}
 else{while(top.parentNode!==b)top=top.parentNode;
  if(olds.indexOf(top)>=0){spot=top;}
  else{var tail=document.createRange();tail.setStart(r.startContainer,r.startOffset);tail.setEndAfter(top);
   var rest=tail.extractContents();var keep=function(n){return n&&(n.textContent.trim()!==''||n.querySelector&&n.querySelector('img,table,hr'));};
   spot=top.nextSibling;
   if(keep(rest)){b.insertBefore(rest,spot);spot=top.nextSibling;}
   if(!keep(top)){spot=top.nextSibling;top.remove();}}}}
for(var k=0;k<olds.length;k++){if(spot===olds[k])spot=olds[k].nextSibling;olds[k].remove();}
var tpl=document.createElement('template');tpl.innerHTML=h;var sig=tpl.content.firstElementChild;
if(where==='start'){b.insertBefore(sig,b.firstChild);}
else if(where==='end'){b.appendChild(sig);}
else if(r){b.insertBefore(sig,spot);var c=document.createRange();c.setStartBefore(sig);c.collapse(true);sel.removeAllRanges();sel.addRange(c);}
else{@ANCHOR@}
window.__tegamiPlainFix&&window.__tegamiPlainFix();
window.__tegamiDirty=true;})()"#
            .replace("@WHERE@", &format!("'{place_js}'"))
            .replace("@ANCHOR@", self.rich_sig_anchor())
            .replace("@SIG@", &format!("'{}'", rich_editor::js_escape(&sig_html(&sig, self.sig_dashes))));
        self.editor.run_js(&js);
        self.current_sig = sig;
    }

    /// Put Sign where the From account's default has it (#267), until the
    /// user has set it by hand. While encrypting it stays on: encrypted
    /// mail is always signed.
    pub(super) fn follow_account_signing(&mut self, widgets: &ComposeWidgets) {
        if self.sign_touched || self.encrypt || !crate::mail::pgp::available() {
            return;
        }
        let idx = self.from_index;
        let on = self.accounts.get(idx).is_some_and(|a| a.sign_default);
        if on != widgets.sign_btn.is_active() {
            self.sign_expected = Some(on);
            widgets.sign_btn.set_active(on);
        }
    }

    /// Handles [`ComposeInput::InsertSignature`].
    pub(super) fn on_insert_signature(&mut self, place: SigPlace) {
        let idx = self.from_index;
        let sig = self
            .accounts
            .get(idx)
            .map(|a| self.chosen_signature(a))
            .unwrap_or_default();
        self.insert_signature(sig, place);
    }

    /// Handles [`ComposeInput::SignatureClicked`].
    pub(super) fn on_signature_clicked(&mut self, sender: &ComponentSender<Self>) {
        let idx = self.from_index;
        let Some(account) = self.accounts.get(idx) else {
            return;
        };
        if account.signatures.len() < 2 {
            sender
                .input_sender()
                .emit(ComposeInput::InsertSignature(SigPlace::Cursor));
            return;
        }
        // The one in the message is drawn as chosen; a body that
        // started without one (Edit as New, a draft) may still
        // carry the default's text, which is no harm to show.
        let list: Vec<MenuEntry> = account
            .signatures
            .iter()
            .enumerate()
            .map(|(i, sig)| {
                let s = sender.clone();
                MenuEntry::new(&sig.name, move || {
                    s.input_sender().emit(ComposeInput::UseSignature(Some(i)))
                })
                .selected(!self.current_sig.is_empty() && sig.html == self.current_sig)
            })
            .collect();
        let s = sender.clone();
        let none = MenuEntry::new(i18n("No signature"), move || {
            s.input_sender().emit(ComposeInput::UseSignature(None))
        })
        .selected(self.current_sig.is_empty());
        let btn = &self.signature_btn;
        show_context_menu(
            btn,
            (btn.width() / 2) as f64,
            btn.height() as f64,
            vec![list, vec![none]],
        );
    }

    /// Handles [`ComposeInput::UseSignature`].
    pub(super) fn on_use_signature(&mut self, choice: Option<usize>) {
        let idx = self.from_index;
        let sig = match choice {
            Some(i) => match self.accounts.get(idx).and_then(|a| a.signatures.get(i)) {
                Some(sig) => sig.html.clone(),
                None => return,
            },
            None => String::new(),
        };
        if sig != self.current_sig {
            self.put_signature(sig);
        }
    }

    /// Handles [`ComposeInput::SignatureMenu`].
    pub(super) fn on_signature_menu(&mut self, sender: &ComponentSender<Self>) {
        let entry = |label: String, place: SigPlace| {
            let s = sender.clone();
            MenuEntry::new(&label, move || {
                s.input_sender().emit(ComposeInput::InsertSignature(place))
            })
        };
        let btn = &self.signature_btn;
        show_context_menu(
            btn,
            (btn.width() / 2) as f64,
            btn.height() as f64,
            vec![vec![
                entry(i18n("Insert before the text"), SigPlace::Start),
                entry(i18n("Insert after the text"), SigPlace::End),
            ]],
        );
    }
}
