#!/usr/bin/env python3
"""PES M -> CalcioM: the mechanical part of the rename, repeatable on a newer main (Fabian 26.11.2026).

Run from the repository root: python3 tools/calciom_umbenennen.py [--pruefen]
--pruefen only lists what would change. Running it twice changes nothing the second time.

Kept on purpose (names people already have on disk or other programs use, see docs/CALCIOM.md):
- old career folders and save files PESM<Verein>01 (still recognised, new ones are CalcioM<Verein>01)
- content/pesm-livepatch and pesm-livepatch.lua (the FL Stats Converter writes there too)
- fl26match.lua.pesm-original, *.txt.vor-pesm (backups made by older versions are restored from these names)
- the phone cookie pesm_geraet (paired phones stay paired)
- internal folders assets/inhalt/pesm/** (mirror the Python original), Lua names pesm_stadion/pesm_idx
- the hashes in assets/fl27/module/*.sha256 (of the other mods' unchanged originals)
"""
import re
import subprocess
import sys

DATEIEN = [  # git mv: old -> new
    ("src/bin/pesm-speicher.rs", "src/bin/calciom-speicher.rs"),
    ("assets/fl27/direktmatch/pesm_direktmatch.lua", "assets/fl27/direktmatch/calciom_direktmatch.lua"),
]

AUSGENOMMEN = re.compile(r"^(Cargo\.lock|assets/fl27/module/.*\.sha256$|assets/wappen/|tests_data/|\.werkzeug/|tools/calciom_umbenennen\.py$|docs/CALCIOM\.md$|docs/intern/RELEASE-)")

# order matters: the specific rules first, the general word rules last
REGELN = [
    # trademark in the subtitle and its text key
    (r"Pro Evolution Soccer Manager", "Calcio Management"),
    (r"pro_evolution_soccer_manager_fussball", "calcio_management_fussball"),
    # own career folders: old (PESM...) and new (CalcioM...) names both count as ours
    (r'name\.to_uppercase\(\)\.starts_with\(crate::id\("PESM"\)\)', "crate::fl27::eigener_cpk(&name)"),
    (r'(\w+)\.starts_with\(crate::id\("PESM"\)\)', r"crate::fl27::eigener_cpk(\1)"),
    # our marker in other mods' Lua files: old installs carry "PES M", new ones "CalcioM"
    (r'String::from_utf8_lossy\(&b\)\.contains\("PES M"\)', "crate::fl27::von_uns(&String::from_utf8_lossy(&b))"),
    (r'\.map\(\|t\| t\.contains\("PES M"\)\)', ".map(|t| crate::fl27::von_uns(&t))"),
    # folders and files we put into the game
    (r"PESM_DirectMatch", "CalcioM_DirectMatch"),
    (r"pesm_direktmatch\.lua", "calciom_direktmatch.lua"),
    (r"pesm-direktmatch", "calciom-direktmatch"),
    # environment variables (developer and test switches)
    (r"\bPESM_([A-Z0-9_]+)\b", r"CALCIOM_\1"),
    # programs, data folders, log
    (r'name = "pesm"', 'name = "calciom"'),
    (r'default-run = "pesm"', 'default-run = "calciom"'),
    (r"\bpesm-speicher\b", "calciom-speicher"),
    (r"target/(release|debug)/pesm\b", r"target/\1/calciom"),
    (r"\.local/share/pesm-rust", ".local/share/calciom"),
    (r'join\(crate::id\("pesm"\)\)', 'join(crate::id("calciom"))'),
    (r"%APPDATA%\\pesm\b", r"%APPDATA%\\calciom"),
    (r"\bpesm\.log\b", "calciom.log"),
    (r"\bpesm-(tmp|diagnose|stadion|formation|vorgaben|schlank|fl27|zuschlag|paket1|starttest|live|musik|mpv|cpk|karr)\b", r"calciom-\1"),
    (r"bak-pesm-", "bak-calciom-"),
    (r"\bpesm_handy_", "calciom_handy_"),
    (r"User-Agent: pesm\b", "User-Agent: calciom"),
    (r"\bPESM/0\.1\b", "CalcioM/0.1"),
    # the test of the name a new career folder gets
    (r'(assert_eq!\(cpk_name\(.*?\)\), )"PESM', r'\1"CalcioM'),
    # the name itself
    (r"PES M", "CalcioM"),
    (r"\bPESM\b", "CalcioM"),
]


def git(*a):
    return subprocess.run(["git", *a], check=True, capture_output=True, text=True).stdout


def umbenennen(text):
    for alt, neu in REGELN:
        text = re.sub(alt, neu, text)
    return text


def selbsttest():
    f = umbenennen
    assert f('window_title: "PES M".into()') == 'window_title: "CalcioM".into()'
    assert f("PES Ms Zahlen") == "CalcioMs Zahlen"
    assert f('format!("PESM{base}{n:02}")') == 'format!("CalcioM{base}{n:02}")'
    assert f('if n.starts_with(crate::id("PESM")) && n') == "if crate::fl27::eigener_cpk(n) && n"
    assert f('std::env::var("PESM_KARRIEREN")') == 'std::env::var("CALCIOM_KARRIEREN")'
    assert f(r'\livecpk\PESM_DirectMatch"') == r'\livecpk\CalcioM_DirectMatch"'
    assert f("PESMChelseaFC01 PESM_Bayern") == "PESMChelseaFC01 PESM_Bayern"  # old career names stay as they are
    assert f("content/pesm-livepatch pesm-livepatch.lua fl26match.lua.pesm-original") == "content/pesm-livepatch pesm-livepatch.lua fl26match.lua.pesm-original"
    assert f("PESM-LivePatch") == "CalcioM-LivePatch"
    assert f('.join("pesm.log")') == '.join("calciom.log")'
    assert f(f(f("PES M und PESM_WELT"))) == "CalcioM und CALCIOM_WELT"


def main():
    selbsttest()
    pruefen = "--pruefen" in sys.argv
    for alt, neu in DATEIEN:
        if subprocess.run(["git", "ls-files", "--error-unmatch", alt], capture_output=True).returncode == 0:
            print(f"mv {alt} -> {neu}")
            if not pruefen:
                git("mv", alt, neu)
    geaendert = 0
    for pfad in git("ls-files").splitlines():
        if AUSGENOMMEN.match(pfad):
            continue
        try:
            with open(pfad, encoding="utf-8", newline="") as fh:
                alt = fh.read()
        except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError):
            continue
        neu = umbenennen(alt)
        if neu != alt:
            geaendert += 1
            if pruefen:
                print(pfad)
            else:
                with open(pfad, "w", encoding="utf-8", newline="") as fh:
                    fh.write(neu)
    print(f"{geaendert} Dateien {'würden geändert' if pruefen else 'geändert'}")


if __name__ == "__main__":
    main()
