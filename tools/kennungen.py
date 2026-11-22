#!/usr/bin/env python3
"""Marks identifiers in the code: every word-like string literal that is no key, no path and no shown text gets
crate::id("…"). Same rules as the test inhalt::tests::sprache_abdeckung. `--nur-zaehlen` only counts.
Exempt: tests, comments, `// roh` lines, keys/paths (a.b, a/b, a_b), arguments of t / tf! / id / rule lookups,
match and matches! patterns, developer messages (panic!, expect …), include_str!/env::var."""
import re, sys, glob, os

APP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NACH_AUFRUF = re.compile(r"(\b(t|t_in|tf_in|id|anzeige|eigen|text_wert|wert|wert_oder|arr|f|f_oder|fz|tx|tx_liste|pw|j|get|contains_key|remove|entry|var|var_os|play)\(|tf!\(|[\w\])]\[)\s*&?\s*$")
# calls whose text arguments are shown: a word there is a text and needs a key, even when marked with id()
METHODEN = {"text", "text_semi", "title", "text_right", "text_center", "button", "button_gold", "card_titled", "fit", "segment", "paragraph",
            "empty", "banner", "stage", "toast", "msg", "news", "measure", "measure_semi", "measure_title", "pill_text", "tabs"}
FREI = {"tf!", "kpi", "player_row_head", "measure_text"}

def angezeigt(vor):
    """Is the literal at the end of `vor` an argument of a call that shows it? Only the innermost call counts;
    if/else blocks and lists are looked through, struct literals and closures are not."""
    stapel, i, n = [], 0, len(vor)
    while i < n:
        c = vor[i]
        if c == '"':
            i += 1
            while i < n and vor[i] != '"': i += 2 if vor[i] == "\\" else 1
        elif c == "(":
            m = re.search(r"(\.?)([A-Za-z_][A-Za-z0-9_]*!?)\s*$", vor[:i])
            stapel.append(("(", m.group(2) if m else "", bool(m and m.group(1))))
        elif c == "[": stapel.append(("[", "", False))
        elif c == "{": stapel.append(("{", "durch" if re.search(r"(\belse|\bif\b[^{};]*|=>)\s*$", vor[:i]) else "", False))
        elif c in ")]}" and stapel: stapel.pop()
        i += 1
    for art, name, punkt in reversed(stapel):
        if art == "[" or (art == "(" and name in ("", "id", "Some", "to_string", "into", "unwrap_or", "as_deref", "map")): continue
        if art == "{":
            if name == "durch": continue
            return False
        return (punkt and name in METHODEN) or name in FREI
    return False

ZWEITES = re.compile(r"\b(wert|wert_oder|arr|f|f_oder|stil_punkte)\(\s*(crate::id\()?\"[^\"]*\"\)?\s*,\s*&?\s*$")
ENTWICKLER = re.compile(r"(panic!|eprintln!|println!|assert!|assert_eq!|assert_ne!|debug_assert!|unreachable!|todo!|unimplemented!|\.expect|include_str!|include_bytes!|concat!|write!|writeln!)\s*\(")

def schluesselartig(x):
    if re.fullmatch(r"[^A-Za-zÄÖÜäöüß]*", x): return True                     # no letters: symbols, numbers, format
    ohne = re.sub(r"\{[^}]*\}", "", x)
    if not re.search(r"[A-Za-zÄÖÜäöüß]", ohne): return True                      # only placeholders
    if re.search(r"\s", x): return False
    if "/" in x: return True                                                     # file paths
    teile = re.split(r"[./:]", x)
    if len(teile) > 1 and all(teile): return True                               # a.b, a/b, a:b
    return "_" in x and re.fullmatch(r"[A-Za-z0-9_\-]+", x) is not None         # snake_case ids

def literale(z):
    """(start, end, value) of the normal string literals of a code line (no comments, chars, raw or byte strings)."""
    out, i, n = [], 0, len(z)
    while i < n:
        if z.startswith("//", i): break
        c = z[i]
        if c == "'":
            m = re.match(r"'(\\u\{[0-9a-fA-F]+\}|\\.|[^\\'])'", z[i:])
            i += len(m.group(0)) if m else 1; continue
        if c in "rb" and i + 1 < n and z[i+1] in '"#' and not (i and (z[i-1].isalnum() or z[i-1] == "_")):
            j = z.find('"', i + 2) if z[i+1] == '"' else -1
            if c == "b" and z[i+1] == '"':
                j = i + 2
                while j < n and z[j] != '"': j += 2 if z[j] == "\\" else 1
                i = j + 1; continue
            break  # raw string: rest of the line is not code we judge
        if c == '"':
            j = i + 1
            while j < n and z[j] != '"': j += 2 if z[j] == "\\" else 1
            out.append((i, j + 1, z[i+1:j])); i = j + 1; continue
        i += 1
    return out

def befund(z, s, e, x):
    """None (fine), 'id' (needs crate::id) or 'text' (a shown text without key)."""
    if schluesselartig(x): return None
    vor, nach = z[:s], z[e:]
    if re.search(r"(==|!=)\s*&?\s*(crate::id\()?\s*$", vor) or re.match(r"\)?\s*(==|!=)", nach): return None   # compared: an id
    if re.search(r"\w:\s*(crate::id\()?\s*$", vor): return None                          # struct field value: an id
    if angezeigt(vor) and not re.search(r"tf!\(\s*$", vor): return "text"                # shown: needs a key
    if NACH_AUFRUF.search(vor) or ZWEITES.search(vor): return None
    if ENTWICKLER.search(vor): return None
    if re.match(r"\s*(=>|\|)", nach) or re.search(r"\|\s*$", vor): return None   # match patterns
    m = re.search(r"=>", nach)
    if m and not re.search(r"[{;]|\b(if|else)\b", nach[:m.start()]): return None    # inside a longer pattern: (Some("x"), _) =>
    if re.match(r"\s*:(?!:)", nach): return None                                    # json!/map key "x": …
    if re.search(r"\b(format|format_args|print|eprint|write|json)!\(\s*$", vor): return None
    if "matches!(" in vor and vor.count("(") > vor.count(")"): return None
    if re.search(r"\b(if let|while let)\s+[^=]*$", vor): return None
    return "text" if re.search(r"\s", x.strip()) else "id"

def pruefen(datei, schreiben):
    t = open(datei).read()
    kopf, sep, rest = t.partition("\n#[cfg(test)]")
    zeilen = kopf.split("\n")
    funde = []
    for k, z in enumerate(zeilen):
        code = z.lstrip()
        if "// roh" in z or code.startswith("//") or code.startswith("#[") or code.startswith("///"): continue
        ersetz = []
        for s, e, x in literale(z):
            b = befund(z, s, e, x)
            if b: funde.append((b, k + 1, x))
            if b == "id": ersetz.append((s, e))
        if schreiben and ersetz:
            for s, e in reversed(ersetz): z = z[:s] + "crate::id(" + z[s:e] + ")" + z[e:]
            zeilen[k] = z
    if schreiben: open(datei, "w").write("\n".join(zeilen) + sep + rest)
    return funde

if __name__ == "__main__":
    schreiben = "--nur-zaehlen" not in sys.argv
    gesamt = {"id": 0, "text": 0}
    for f in sorted(glob.glob(APP + "/src/*.rs") + glob.glob(APP + "/src/bin/*.rs")):
        if f.endswith("/inhalt.rs"): continue  # the text machinery itself (language codes, its file names)
        for b, nr, x in pruefen(f, schreiben):
            gesamt[b] += 1
            if b == "text": print("Text ohne Schlüssel:", os.path.basename(f), nr, repr(x))
    print(gesamt)
