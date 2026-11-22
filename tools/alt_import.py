#!/usr/bin/env python3
"""Reads a career of the Python version (fl-world karrieren/<id>: welt/welt.db) and writes the JSON that
`pesm --import-alt <json> <karriere-ordner>` turns into a Rust career: date, coach, players, league tables (the imported
base plus the matches played since), league and cup fixtures, club money. Nothing in the old career is changed.
Usage: python3 -I tools/alt_import.py <karriere-ordner> <ziel.json>
"""
import json, sqlite3, sys

ordner, ziel = sys.argv[1], sys.argv[2]
db = sqlite3.connect(f"file:{ordner}/welt/welt.db?mode=ro", uri=True)
meta = dict(db.execute("select k, v from meta"))
ext = {k: json.loads(v) for k, v in db.execute("select k, v from ext")}
saison_alt = int(meta["season"])

spieler = []
for r in db.execute("select id, club, form, fit, morale, inj, susp, yc, goals, apps, mins, rsum, contract_end, wage, x from player"):
    x = json.loads(r[14] or "{}")
    spieler.append({"id": r[0], "club": r[1], "form": r[2], "fit": r[3], "morale": r[4], "inj": r[5], "susp": r[6], "yc": r[7],
                    "goals": r[8], "apps": r[9], "rsum": r[11], "rated": x.get("rated", r[9]), "assists": x.get("assists", 0),
                    "contract_end": r[12] - saison_alt if r[12] is not None else None, "wage": r[13], "no": x.get("no")})

vereine = {}
for cid, x in db.execute("select id, x from club"):
    x = json.loads(x or "{}")
    fin = x.get("fin", {})
    vereine[cid] = {"balance": x.get("balance"), "tv": fin.get("tv"), "gate": fin.get("gate"), "comm": fin.get("comm"), "opex": fin.get("opex"), "fam": x.get("familiarity")}

basis = ext.get("gimport", {}).get("basis", {})
ligen, pokale = {}, {}
for cid, name, kind in db.execute("select id, name, kind from comp"):
    fx = []
    for f in db.execute("select day, rnd, home, away, hg, ag, status, info from fixture where comp=? and season=? order by day", (cid, saison_alt)):
        info = (json.loads(f[7]) if f[7] else None) or {}
        fx.append({"day": f[0], "rnd": f[1], "h": f[2], "a": f[3], "hg": f[4] if f[6] == 1 else -1, "ag": f[5] if f[6] == 1 else -1, "pens": info.get("pens")})
    if kind == "league":
        ligen[name] = {"basis": {int(k): [v["p"], v["w"], v["d"], v["l"], v["gf"], v["ga"], v["pts"]] for k, v in basis.get(str(cid), {}).items()}, "fixtures": fx}
    elif kind == "cup":
        pokale[name] = {"fixtures": fx}

# who played in Europe in the old career (the import must not put the own club there by itself)
europa = sorted({t for (t,) in db.execute("select home from fixture f join comp c on c.id=f.comp where c.kind='ext' union select away from fixture f join comp c on c.id=f.comp where c.kind='ext'")})
out = {"europa": europa, "budget": float(meta.get("budget") or 0), "quelle": ordner, "day": int(meta["day"]), "coach": json.loads(meta.get("manager", '""')), "own": int(meta["own"]),
       "spieler": spieler, "vereine": vereine, "ligen": ligen, "pokale": pokale}
json.dump(out, open(ziel, "w"), ensure_ascii=False)
print(f"{len(spieler)} Spieler, {len(vereine)} Vereine, {len(ligen)} Ligen, {len(pokale)} Pokale, Tag {out['day']}, Trainer {out['coach']}")
