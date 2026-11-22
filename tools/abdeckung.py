#!/usr/bin/env python3
"""Builds ABDECKUNG.md (PAKETE.md Punkt 11): one row per file of the Python version (pesm-ml-raus/pesm, without tests,
__pycache__ and ui/demo) with its task (first docstring line), the Rust place and a status.

Status:
- gebaut: a Rust file names this Python file (comments like "wochenlauf.py") - checked by hand when it was built.
- Bereich: the Rust file of the same area exists, the file itself is not named - to be checked.
- entfällt: ML and Sider-reading parts (E186), the web backend and the Qt/HTML front end (Rust draws itself).
- offen: no Rust place found.
Usage: python3 -I tools/abdeckung.py [Pfad zu pesm-ml-raus/pesm]
"""
import ast, os, re, sys

APP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PY = sys.argv[1] if len(sys.argv) > 1 else os.path.expanduser("~/Projects/pesm/pesm-ml-raus/pesm")

# area of the Python version -> Rust files (by hand, from the module docs)
BEREICH = {
    "core/akademie": "akademie.rs", "core/jugend": "akademie.rs", "core/auszeichnungen": "ehrungen.rs",
    "core/entwicklung": "entwicklung.rs", "core/merkmale": "entwicklung.rs", "core/gespraeche": "mensch.rs",
    "core/kabine": "mensch.rs", "core/begruendung": "mensch.rs", "core/dialog": "mensch.rs",
    "core/transfers": "transfer.rs, kimarkt.rs", "core/verhandlung": "verhandlung.rs", "core/scouting": "scouting.rs",
    "core/personal": "personal.rs", "core/trainer": "trainerbank.rs, mensch.rs", "core/ki_trainer": "ki.rs",
    "core/wettbewerbe": "cups.rs, ligaregeln.rs", "core/wirtschaft": "wirtschaft.rs", "core/medien": "medien.rs",
    "core/zeitung": "medien.rs", "core/vorgang": "vorgang.rs", "core/vereinswelt": "farm.rs, seiten_farm.rs",
    "core/spieltag": "game.rs, livespiel.rs", "core/trainingsplan": "game.rs", "core/note": "world.rs",
    "core/laufbahn": "mensch.rs", "core/manager": "mensch.rs", "core/inhalt": "inhalt.rs", "core/texts": "inhalt.rs",
    "core/tagebuch": "spielgefuehl.rs", "core/gegner": "taktik.rs", "core/domain": "world.rs, game.rs",
    "core/services": "game.rs", "core/marken": "wirtschaft.rs", "core/archiv": "spielgefuehl.rs",
    "core/aenderungen": "game.rs", "core/assistent": "spielgefuehl.rs", "core/config": "regeln.rs",
    "core/mechanics": "world.rs", "adapters/pes": "pesdb.rs, fl27.rs", "adapters/karriere": "game.rs, geo.rs",
    "adapters/bilder": "ui.rs", "adapters/assets": "ui.rs", "adapters/storage": "game.rs", "fl/sim": "world.rs, cups.rs, ligaregeln.rs",
    "fl": "fl27.rs, live.rs", "tools": "tools/",
}
# single files at the top of core/ and pesm/ (by hand)
DATEI = {
    "core/attribute.py": "ui.rs, pages.rs", "core/aufstellung_eindeutig.py": "game.rs", "core/berechtigung.py": "mensch.rs",
    "core/dateistand.py": "entfällt (SQLite)", "core/einfluesse.py": "mensch.rs", "core/fans.py": "pages.rs (Fans), wirtschaft.rs",
    "core/gesamtwert.py": "pesdb.rs (ovr)", "core/handbuch.py": "pages.rs (Handbuch)", "core/hierarchie.py": "mensch.rs",
    "core/kaderuebersicht.py": "pages.rs (Kader)", "core/kleinigkeiten.py": "pages.rs, postkarten.rs", "core/nachweis.py": "fl27.rs",
    "core/persoenlichkeit.py": "mensch.rs", "core/ports.py": "entfällt", "core/sicherung.py": "game.rs (Sicherung)", "core/sicht.py": "scouting.rs",
    "core/sprache.py": "inhalt.rs", "core/startelfentwurf.py": "game.rs, fl27.rs", "core/stufen.py": "seiten.rs (stufe_wort)",
    "core/tabellensperre.py": "medien.rs", "core/taktik.py": "taktik.rs", "core/taktikengine.py": "taktik.rs", "core/taktikfm.py": "taktik.rs, pages.rs",
    "core/taktikmodus.py": "taktik.rs", "core/taktikvorschlag.py": "taktik.rs", "core/verfuegbarkeit.py": "game.rs",
    "pfade.py": "game.rs (Karrieren-Ordner)", "probleme.py": "pages.rs (Profi-Details)", "diagnose.py": "pages.rs (Profi-Details)",
    "gesundheit.py": "fl27.rs (Wächter)", "waechter.py": "entfällt (Qt)", "protocol.py": "entfällt (Backend)", "stand.py": "entfällt (Backend)",
}
ENTFAELLT = ("backend", "ui", "app", "fl/beobachter", "adapters/fremd")


def aufgabe(pfad):
    try:
        mod = ast.parse(open(pfad, encoding="utf-8").read())
        d = ast.get_docstring(mod)
    except Exception:
        d = None
    return (d or "").strip().splitlines()[0][:140] if d else ""


def main():
    genannt = {}
    for wurzel, _, dateien in os.walk(os.path.join(APP, "src")):
        for n in dateien:
            if n.endswith(".rs"):
                for m in set(re.findall(r"([a-z_0-9]+)\.py\b", open(os.path.join(wurzel, n), encoding="utf-8").read())):
                    genannt.setdefault(m, []).append(n)
    zeilen, zahl = [], {}
    for wurzel, ordner, dateien in os.walk(PY):
        ordner[:] = sorted(o for o in ordner if o not in ("__pycache__", "tests") and not (o == "demo" and wurzel.endswith("/ui")))
        for n in sorted(dateien):
            if not n.endswith(".py"):
                continue
            rel = os.path.relpath(os.path.join(wurzel, n), PY)
            teile = rel.split("/")
            bereich = "/".join(teile[:2]) if len(teile) > 2 else teile[0] if len(teile) > 1 else ""
            stamm = n[:-3]
            if stamm in genannt and stamm != "__init__":
                st, ort = "gebaut", ", ".join(sorted(set(genannt[stamm])))
            elif rel in DATEI:
                ort = DATEI[rel]
                st = "entfällt" if ort.startswith("entfällt") else "Bereich"
            elif n == "__init__.py":
                st, ort = "entfällt", "— (Paketdatei)"
            elif any(rel.startswith(e + "/") for e in ENTFAELLT):
                st, ort = "entfällt", "—"
            elif bereich in BEREICH:
                st, ort = "Bereich", BEREICH[bereich]
            elif teile[0] in BEREICH:
                st, ort = "Bereich", BEREICH[teile[0]]
            else:
                st, ort = "offen", ""
            zahl[st] = zahl.get(st, 0) + 1
            zeilen.append(f"| `{rel}` | {aufgabe(os.path.join(wurzel, n)).replace('|', '/')} | {ort} | {st} |")
    kopf = ("# Abdeckung der Python-Fassung\n\nErzeugt von `tools/abdeckung.py` (neu erzeugen nach jedem größeren Bau). Eine Zeile je Datei aus "
            "`pesm-ml-raus/pesm/` ohne Tests, `__pycache__` und `ui/demo/`. Status: **gebaut** = eine Rust-Datei nennt die Python-Datei; "
            "**Bereich** = die Rust-Datei des Bereichs gibt es, die Datei selbst ist dort nicht genannt (prüfen); **entfällt** = Web-Backend, "
            "Oberfläche der Python-Fassung, Speicher-Beobachter (Rust zeichnet selbst, liest über die Coaching Zone); **offen** = keine Rust-Stelle.\n\n"
            "| Datei | Aufgabe | Rust | Status |\n|---|---|---|---|\n")
    fuss = "\n**Zahlen:** " + ", ".join(f"{k} {v}" for k, v in sorted(zahl.items())) + f", zusammen {sum(zahl.values())}.\n"
    open(os.path.join(APP, "ABDECKUNG.md"), "w", encoding="utf-8").write(kopf + "\n".join(zeilen) + "\n" + fuss)
    print(fuss.strip())


if __name__ == "__main__":
    main()
