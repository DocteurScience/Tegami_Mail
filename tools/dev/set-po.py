#!/usr/bin/env python3
"""Set translations in a .po file and drop their fuzzy flag and previous-msgid lines.
Usage: setpo.py FILE MSGID TRANSLATION [MSGID TRANSLATION ...]
A plural entry is given as MSGID and "singular|plural" (split on the first |)."""
import re, sys
path, args = sys.argv[1], sys.argv[2:]
text = open(path, encoding="utf-8").read()
blocks = text.split("\n\n")
def q(t): return t.replace("\\", "\\\\").replace('"', '\\"')
def field(lines, key):
    out, on = [], False
    for l in lines:
        if l.startswith(key + " "):
            on = True; out.append(l[len(key) + 1:])
        elif on and l.startswith('"'):
            out.append(l)
        else:
            on = False
    return "".join(re.findall(r'"((?:[^"\\]|\\.)*)"', "\n".join(out))) if out else None
for mid, tr in zip(args[::2], args[1::2]):
    hits = 0
    for i, blk in enumerate(blocks):
        lines = blk.split("\n")
        if field(lines, "msgid") != q(mid):
            continue
        hits += 1
        keep = [l for l in lines if not l.startswith("#|") and l != "#, fuzzy"]
        keep = [l.replace(", fuzzy", "") if l.startswith("#,") else l for l in keep]
        head, j = [], 0
        while j < len(keep) and not keep[j].startswith("msgstr"):
            head.append(keep[j]); j += 1
        if field(lines, "msgid_plural") is not None:
            one, many = tr.split("|", 1)
            body = [f'msgstr[0] "{q(one)}"', f'msgstr[1] "{q(many)}"']
        else:
            body = [f'msgstr "{q(tr)}"']
        blocks[i] = "\n".join(head + body)
    assert hits == 1, (mid, hits)
open(path, "w", encoding="utf-8").write("\n\n".join(blocks))
