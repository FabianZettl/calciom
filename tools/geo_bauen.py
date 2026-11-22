#!/usr/bin/env python3
"""Builds the offline map data of the world map (Gesamtauftrag Punkt 13), once, with network:
1. assets/geo/laender.json: country outlines from Natural Earth Admin 0, 1:50m (public domain), simplified.
2. assets/geo/vereine.json: club positions from Wikidata (CC0): club -> home venue (P115) -> coordinates (P625),
   otherwise the club's own coordinates (P625) or its seat (P159). Name, country and source per entry.
PES M reads only these files; it never goes online. Raw downloads stay in tools/geo_roh/ (not in the program).
Usage: python3 -I tools/geo_bauen.py [laender] [vereine]
"""
import json, math, os, struct, sys, time, urllib.parse, urllib.request

APP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ROH = os.path.join(APP, "tools", "geo_roh")
ZIEL = os.path.join(APP, "assets", "geo")


def dbf_lesen(pfad):
    with open(pfad, "rb") as f:
        kopf = f.read(32)
        n, hl, rl = struct.unpack("<IHH", kopf[4:12])
        felder = []
        while True:
            d = f.read(32)
            if d[0] == 0x0D: break
            felder.append((d[:11].split(b"\0")[0].decode(), d[11:12].decode(), d[16]))
        f.seek(hl)
        zeilen = []
        for _ in range(n):
            r = f.read(rl)
            pos, z = 1, {}
            for name, typ, laenge in felder:
                z[name] = r[pos:pos + laenge].decode("utf-8", "replace").strip().strip("\0").strip()
                pos += laenge
            zeilen.append(z)
    return zeilen


def shp_lesen(pfad):
    """Polygons (type 5) of a shapefile: per record a list of rings [(lon, lat), ...]."""
    out = []
    with open(pfad, "rb") as f:
        f.seek(100)
        while True:
            k = f.read(8)
            if len(k) < 8: break
            _, laenge = struct.unpack(">ii", k)
            inhalt = f.read(laenge * 2)
            typ = struct.unpack("<i", inhalt[:4])[0]
            if typ != 5: out.append([]); continue
            nteile, npunkte = struct.unpack("<ii", inhalt[36:44])
            teile = list(struct.unpack("<%di" % nteile, inhalt[44:44 + 4 * nteile]))
            p0 = 44 + 4 * nteile
            pts = [struct.unpack("<dd", inhalt[p0 + 16 * i:p0 + 16 * i + 16]) for i in range(npunkte)]
            ringe = [pts[a:b] for a, b in zip(teile, teile[1:] + [npunkte])]
            out.append(ringe)
    return out


def vereinfachen(ring, eps):
    """Douglas-Peucker on lon/lat (eps in degrees)."""
    if len(ring) < 4: return ring
    def dp(a, b):
        (x1, y1), (x2, y2) = ring[a], ring[b]
        dx, dy = x2 - x1, y2 - y1
        n = math.hypot(dx, dy) or 1e-12
        best, idx = 0, a
        for i in range(a + 1, b):
            x, y = ring[i]
            d = abs(dy * x - dx * y + x2 * y1 - y2 * x1) / n
            if d > best: best, idx = d, i
        if best > eps: return dp(a, idx)[:-1] + dp(idx, b)
        return [ring[a], ring[b]]
    # a closed ring: split at the middle so both halves have distinct end points
    m = len(ring) // 2
    return dp(0, m)[:-1] + dp(m, len(ring) - 1)


def laender():
    ne = os.path.join(ROH, "ne")
    attr = dbf_lesen(os.path.join(ne, "ne_50m_admin_0_countries.dbf"))
    formen = shp_lesen(os.path.join(ne, "ne_50m_admin_0_countries.shp"))
    out = []
    for a, ringe in zip(attr, formen):
        rr = []
        for r in ringe:
            v = vereinfachen(r, 0.03)
            if len(v) >= 4: rr.append([[round(x, 2), round(y, 2)] for x, y in v])
        if rr: out.append({"iso": a.get("ADM0_A3") or a.get("ISO_A3"), "iso2": a.get("ISO_A2_EH") or a.get("ISO_A2"), "de": a.get("NAME_DE") or a.get("NAME"), "en": a.get("NAME_EN") or a.get("NAME"), "ringe": rr})
    os.makedirs(ZIEL, exist_ok=True)
    with open(os.path.join(ZIEL, "laender.json"), "w", encoding="utf-8") as f:
        json.dump({"quelle": "Natural Earth 1:50m Admin 0 Countries (public domain), vereinfacht", "laender": out}, f, ensure_ascii=False, separators=(",", ":"))
    print("laender", len(out), os.path.getsize(os.path.join(ZIEL, "laender.json")) // 1024, "KB")


SPARQL = """
SELECT ?club ?clubLabel ?clubLabelEn ?landLabel ?coord ?quelle WHERE {
  ?club wdt:P31 wd:Q476028 .
  { ?club wdt:P115 ?venue . ?venue wdt:P625 ?coord . BIND("stadion" AS ?quelle) }
  UNION { ?club wdt:P625 ?coord . BIND("verein" AS ?quelle) }
  OPTIONAL { ?club wdt:P17 ?land . }
  OPTIONAL { ?club rdfs:label ?clubLabelEn FILTER(LANG(?clubLabelEn) = "en") }
  SERVICE wikibase:label { bd:serviceParam wikibase:language "de,en". }
}
"""


def vereine():
    url = "https://query.wikidata.org/sparql?" + urllib.parse.urlencode({"query": SPARQL, "format": "json"})
    req = urllib.request.Request(url, headers={"User-Agent": "PESM-Rust geo_bauen/1.0 (private use)", "Accept": "application/sparql-results+json"})
    roh = os.path.join(ROH, "wikidata_vereine.json")
    if not os.path.exists(roh):
        with urllib.request.urlopen(req, timeout=300) as r, open(roh, "wb") as f: f.write(r.read())
    daten = json.load(open(roh, encoding="utf-8"))["results"]["bindings"]
    out = {}
    for b in daten:
        c = b["coord"]["value"]  # Point(lon lat)
        if not c.startswith("Point("): continue
        lon, lat = (float(x) for x in c[6:-1].split())
        q = b["club"]["value"].rsplit("/", 1)[1]
        e = out.get(q)
        quelle = b["quelle"]["value"]
        if e and e["q"] == "stadion": continue  # the venue wins over the club's own point
        out[q] = {"de": b.get("clubLabel", {}).get("value", ""), "en": b.get("clubLabelEn", {}).get("value", ""), "land": b.get("landLabel", {}).get("value", ""),
                  "lon": round(lon, 4), "lat": round(lat, 4), "q": quelle}
    os.makedirs(ZIEL, exist_ok=True)
    liste = [dict(id=k, **v) for k, v in out.items() if v["de"] and not v["de"].startswith("Q")]
    with open(os.path.join(ZIEL, "vereine.json"), "w", encoding="utf-8") as f:
        json.dump({"quelle": "Wikidata (CC0), Abruf %s: Verein -> Heimstadion (P115) -> Koordinaten (P625), sonst Koordinaten des Vereins" % time.strftime("%Y-%m-%d"), "vereine": liste}, f, ensure_ascii=False, separators=(",", ":"))
    print("vereine", len(liste), os.path.getsize(os.path.join(ZIEL, "vereine.json")) // 1024, "KB")


def orte():
    """Towns with at least 15,000 people (GeoNames cities15000, CC BY 4.0): (ascii name, country ISO2, lon, lat), the
    largest town per name and country. PES M finds a club's town in its name ("Sunderland AFC" -> Sunderland)."""
    import zipfile
    z = zipfile.ZipFile(os.path.join(ROH, "cities15000.zip"))
    beste = {}
    for zeile in z.read("cities15000.txt").decode("utf-8").splitlines():
        f = zeile.split("\t")
        name, land, lat, lon, ew = f[2], f[8], float(f[4]), float(f[5]), int(f[14] or 0)
        k = (name.lower(), land)
        if k not in beste or beste[k][3] < ew: beste[k] = (name, land, (round(lon, 3), round(lat, 3)), ew)
    out = [[n, l, p[0], p[1], e] for n, l, p, e in beste.values()]
    with open(os.path.join(ZIEL, "orte.json"), "w", encoding="utf-8") as f:
        json.dump({"quelle": "GeoNames cities15000 (CC BY 4.0, geonames.org)", "orte": out}, f, ensure_ascii=False, separators=(",", ":"))
    print("orte", len(out), os.path.getsize(os.path.join(ZIEL, "orte.json")) // 1024, "KB")


if __name__ == "__main__":
    was = sys.argv[1:] or ["laender", "orte", "vereine"]
    if "laender" in was: laender()
    if "orte" in was: orte()
    if "vereine" in was: vereine()
