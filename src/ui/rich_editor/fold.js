(function(){
  /* The conversation history under a reply or forward (docs/DESIGN.md,
     "Replying and composing"; folding from #549): a header names it, says
     how many messages it quotes and hides or shows it. The quoted messages
     are sent whether they are shown or not.

     The header and the "Show previous message" button are furniture, like
     the resize frame: `data-tegami-ui`, so nothing the document hands back
     carries them, and contenteditable=false, so the caret passes over them.
     The header stands right above the "wrote:" line; the stylesheet hides
     the two blocks after it (that line and the quote) while <html> has
     `tegami-folded`, and the quote's older messages (a quote inside the
     quote's quote) while it lacks `tegami-deep`. Both classes are outside
     the body the message is read from.

     Runs after the plain view has flattened the body, where the quote is
     the block after its "wrote:" line. `__tegamiFoldQuote` says whether the
     history starts hidden; `__tegamiHistory` holds the header's words,
     translated by the app. */
  var words = window.__tegamiHistory;
  if(!words) return;
  document.addEventListener('DOMContentLoaded', function(){
    var b = document.body, attr = null;
    for(var n = b.firstElementChild; n; n = n.nextElementSibling){
      if(n.classList.contains('tegami-quote-attr')){ attr = n; break; }
    }
    var quote = attr && attr.nextElementSibling;
    if(!quote || quote.hasAttribute('data-tegami-ui')) return;
    var root = document.documentElement;

    /* How many messages the quote holds: each one quoted a level deeper
       than the one that answered it. */
    function depth(el){
      var most = 0;
      for(var c = el.firstElementChild; c; c = c.nextElementSibling){
        most = Math.max(most, depth(c));
      }
      return most + (el.tagName === 'BLOCKQUOTE' ? 1 : 0);
    }
    var count = Math.max(1, depth(quote));
    var forms = words.count;
    var countText = forms[Math.min(count, forms.length - 1)].replace('{n}', count);

    function el(tag, cls, text){
      var e = document.createElement(tag);
      if(cls) e.className = cls;
      if(text) e.textContent = text;
      return e;
    }
    var head = el('div');
    head.setAttribute('data-tegami-ui', 'fold');
    head.setAttribute('contenteditable', 'false');
    head.setAttribute('role', 'button');
    var names = el('div', 'tegami-hist-names');
    names.appendChild(el('div', 'tegami-hist-title', words.title));
    names.appendChild(el('div', 'tegami-hist-count', countText));
    var toggle = el('div', 'tegami-hist-toggle');
    head.appendChild(names);
    head.appendChild(toggle);
    attr.before(head);

    /* Older messages than the quote's own and the one it answers: behind a
       button, as the reader folds older cards. */
    var deep = null;
    if(count > 2){
      deep = el('div', null, words.previous);
      deep.setAttribute('data-tegami-ui', 'deep');
      deep.setAttribute('contenteditable', 'false');
      deep.setAttribute('role', 'button');
      quote.after(deep);
      deep.addEventListener('mousedown', function(e){
        e.preventDefault();
        if(e.button !== 0) return;
        root.classList.add('tegami-deep');
        deep.remove();
      });
    }

    function folded(){ return root.classList.contains('tegami-folded'); }
    function fold(on){
      root.classList.toggle('tegami-folded', on);
      toggle.textContent = on ? words.show : words.hide;
      head.setAttribute('aria-expanded', on ? 'false' : 'true');
    }
    /* On the press, not the click: the press is also what would move the
       caret, and the caret stays where the writing is. */
    head.addEventListener('mousedown', function(e){
      e.preventDefault();
      if(e.button === 0) fold(!folded());
    });
    fold(!!window.__tegamiFoldQuote);

    /* An edit that would reach into the hidden history (Select All and
       Delete, Delete at the end of the line above it) shows it instead,
       so nothing is taken away unseen. The next one goes through. */
    function reaches(r){
      var a = document.createRange();
      try{ a.setStart(r.startContainer, r.startOffset); a.setEnd(r.endContainer, r.endOffset); }
      catch(_){ return false; }
      return a.intersectsNode(attr) || a.intersectsNode(quote);
    }
    document.addEventListener('beforeinput', function(e){
      if(!folded() || !/^(insert|delete)/.test(e.inputType || '')) return;
      var rs = e.getTargetRanges ? e.getTargetRanges() : [];
      if(!rs.length){
        var s = getSelection();
        rs = s.rangeCount ? [s.getRangeAt(0)] : [];
      }
      if(!rs.some(reaches)) return;
      e.preventDefault();
      fold(false);
    }, true);
    /* With the quote or its "wrote:" line edited away, or the quote no
       longer right under it, there is no history left to name. */
    document.addEventListener('input', function(){
      if(head.isConnected && attr.isConnected && attr.nextElementSibling === quote
         && head.nextElementSibling === attr) return;
      root.classList.remove('tegami-folded');
      root.classList.add('tegami-deep');
      head.remove();
      if(deep) deep.remove();
    });
  });
})();
