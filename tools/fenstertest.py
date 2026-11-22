#!/usr/bin/env python3
# Window-size self-test: draws every main page in 1400×900, 1920×1080, 2560×1080 and 1280×1024 (pruefbild.py, PESM_PRUEF)
# and reports texts drawn beyond the window (Überlauf), texts cut to „…“ (gekürzt) and a large empty band on the right
# (Lücke: nothing drawn in the right fifth). Writes FENSTERTEST.md; exit code 1 when any page overflows.
# Usage: fenstertest.py [page …]  (default: all main pages); extra KEY=VAL are passed on (PESM_WELT=…).
import json, os, subprocess, sys, tempfile
APP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SEITEN = ["zentrale", "posteingang", "kalender", "kader", "aufstellung", "training", "medizin", "kabine", "entwicklung", "kaderplanung",
          "vergleich", "vorbereitung", "analyse", "transfermarkt", "verhandlungen", "vertraege", "scouting", "merkliste", "akademie", "jugendnetz",
          "praesidium", "ziele", "finanzen", "stadion", "marken", "fans", "klima", "trainer", "trainerteam", "personalmarkt", "manager",
          "kooperationen", "zweite", "tabellen", "ranglisten", "vereine", "vereinswelt", "portal", "geschichte", "tagebuch", "presse", "zeitung",
          "spielerprofil", "spieltag", "spieltag:PESM_LIVE=1", "analyse:PESM_REITER=1:PESM_DAYS=60", "analyse:PESM_REITER=2:PESM_DAYS=60", "spielerprofil:PESM_REITER=2:PESM_DAYS=60", "optionen", "handbuch", "regeln", "profi"]
GROESSEN = ["1400x900", "1920x1080", "2560x1080", "1280x1024"]
args = [a for a in sys.argv[1:] if "=" not in a or ":" in a]
extra = [a for a in sys.argv[1:] if "=" in a and ":" not in a]
seiten = args or SEITEN
tmp = tempfile.mkdtemp(prefix="pesm-fenster-")
zeilen, fehler = [], 0
for s in seiten:
    for g in GROESSEN:
        bericht = os.path.join(tmp, f"{s.replace(':', '-')}-{g}.json")
        # "seite:KEY=VAL" draws a page in a special state (spieltag:PESM_LIVE=1 = the live view of a match in FL27)
        seite, *zusatz = s.split(":")
        subprocess.run([sys.executable, os.path.join(APP, "tools/pruefbild.py"), os.path.join(tmp, f"{seite}-{g}.png"), f"PESM_PAGE={seite}", f"PESM_FENSTER={g}", f"PESM_PRUEF={bericht}", *zusatz, *extra],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=300)
        try: r = json.load(open(bericht))
        except Exception: zeilen.append(f"| {s} | {g} | kein Bericht | | |"); fehler += 1; continue
        luecke = r["rechts"] < r["breite"] * 0.8
        ueber = r["ueberlauf"]
        fehler += bool(ueber)
        karte = r.get("kartenrand", [])
        zeilen.append(f"| {s} | {g} | {len(ueber)} | {len(karte)} | {len(r['gekuerzt'])} | {'ja' if luecke else ''} |")
        for u in ueber: zeilen.append(f"|  |  | Überlauf: {u.replace('|', '/')} | | | |")
        for u in karte: zeilen.append(f"|  |  | | angepasst: {u.replace('|', '/')} | | |")
        print(s, g, "Überlauf", len(ueber), "Kartenrand", len(karte), "gekürzt", len(r["gekuerzt"]), "Lücke" if luecke else "", flush=True)
text = "# Fenstertest\n\nErzeugt von `tools/fenstertest.py`: jede Hauptseite in vier Fenstergrößen. Überlauf = Text über den Fensterrand hinaus; Kartenrand = Text, der über den Rand seiner Karte, Kachel oder frei gezeichneten Fläche (Spielfeld, Kopfband, Nachrichten-Band, Profilkopf, Seitenleiste) liefe und deshalb verkleinert bzw. mit „…“ gekürzt wurde (der volle Text steht beim Darüberfahren); gekürzt = Text, den die Seite selbst mit „…“ kürzt; Lücke = im rechten Fünftel wird nichts gezeichnet.\n\n| Seite | Größe | Überlauf | Kartenrand | gekürzt | Lücke |\n|---|---|---|---|---|---|\n" + "\n".join(zeilen) + "\n"
if args == [] : open(os.path.join(APP, "FENSTERTEST.md"), "w").write(text)
print("Seiten mit Überlauf:", fehler)
sys.exit(1 if fehler else 0)
