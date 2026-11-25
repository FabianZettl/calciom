//! Pictures from free sources for what the game has none of (Fabian 24.11.: as few placeholders as possible):
//! stadiums, press outlets (logos) and journalists (photos). Wikidata finds the item (club → home venue P115 → image
//! P18; outlet → logo P154 or image P18; person → image P18, only for a journalist/reporter/commentator of that exact
//! name), Wikimedia Commons gives the file with its licence. Only files on Commons are taken (Commons hosts free files
//! only); photos need CC BY / CC BY-SA / CC0 / public domain and keep their credit line, logos only CC0 / public domain.
//! Everything is loaded on the person's computer by one background thread and kept in the cache folder
//! (`<cache>/netzbilder/<art>/<key>.{bild,json}`); nothing goes into the repository. Tests and previews stay offline
//! (PESM_KARRIEREN) unless PESM_NETZBILDER=1.
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{mpsc, Mutex, OnceLock};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Art { Stadion, Medium, Person }
impl Art {
    fn ordner(self) -> &'static str { match self { Art::Stadion => crate::id("stadion"), Art::Medium => crate::id("medium"), Art::Person => crate::id("person") } }
}

/// A picture on disk and its credit line ("" when none is needed).
#[derive(Clone, Debug, PartialEq)]
pub struct Bild { pub datei: PathBuf, pub nachweis: String }

#[derive(Clone, Debug, PartialEq)]
pub enum Stand { Laeuft, Fehlt, Da(Bild) }

const UA: &str = "PESM/0.1 (https://github.com/FabianZettl/pesm; football manager, loads free pictures)"; // roh

fn cache() -> PathBuf {
    if let Ok(d) = std::env::var("PESM_NETZBILDER_DIR") { return PathBuf::from(d); }
    let basis = std::env::var("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(crate::id(".cache")));
    basis.join(crate::id("pesm")).join(crate::id("netzbilder"))
}
fn offline() -> bool { std::env::var("PESM_NETZBILDER").map_or(std::env::var("PESM_KARRIEREN").is_ok(), |v| v == "0") }

/// The file name for a request: readable start plus a short hash, so different names never collide.
fn schluessel(name: &str) -> String {
    let kurz: String = name.chars().filter(|c| c.is_alphanumeric()).take(24).collect::<String>().to_lowercase();
    let h = name.bytes().fold(0xcbf29ce484222325u64, |a, b| (a ^ b as u64).wrapping_mul(0x100000001b3));
    format!("{kurz}-{:08x}", h as u32)
}

type Auftrag = (Art, String, String);
fn zustand() -> &'static Mutex<HashMap<(Art, String), Stand>> { static Z: OnceLock<Mutex<HashMap<(Art, String), Stand>>> = OnceLock::new(); Z.get_or_init(Default::default) }
fn arbeiter() -> &'static Mutex<mpsc::Sender<Auftrag>> {
    static A: OnceLock<Mutex<mpsc::Sender<Auftrag>>> = OnceLock::new();
    A.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Auftrag>();
        // one thread, one request after the other: kind to Wikimedia, never blocks the frame
        std::thread::spawn(move || for (art, name, land) in rx {
            let st = laden(art, &name, &land);
            zustand().lock().unwrap().insert((art, name), st);
        });
        Mutex::new(tx)
    })
}

/// The picture for `name` (club name for a stadium, outlet name, person name); `land` is a country code for the
/// search language ("DE", "GB" …). Laeuft while it is being looked up, then Da or Fehlt for good.
pub fn holen(art: Art, name: &str, land: &str) -> Stand {
    let k = (art, name.to_string());
    if let Some(s) = zustand().lock().unwrap().get(&k) { return s.clone(); }
    let st = match von_platte(art, name) {
        Some(s) => s,
        None if offline() || name.trim().is_empty() => Stand::Fehlt,
        None => { let _ = arbeiter().lock().unwrap().send((art, name.to_string(), land.to_string())); Stand::Laeuft }
    };
    zustand().lock().unwrap().insert(k, st.clone());
    st
}

fn von_platte(art: Art, name: &str) -> Option<Stand> {
    let d = cache().join(art.ordner());
    let k = schluessel(name);
    let j: Value = serde_json::from_str(&std::fs::read_to_string(d.join(format!("{k}.json"))).ok()?).ok()?;
    if j["fehlt"].as_bool() == Some(true) { return Some(Stand::Fehlt); }
    let datei = d.join(format!("{k}.bild"));
    datei.is_file().then(|| Stand::Da(Bild { datei, nachweis: j["nachweis"].as_str().unwrap_or("").to_string() }))
}

/// Looks up, downloads and stores; a network failure leaves no mark (asked again at the next start).
fn laden(art: Art, name: &str, land: &str) -> Stand {
    let d = cache().join(art.ordner());
    let _ = std::fs::create_dir_all(&d);
    let k = schluessel(name);
    let ergebnis = suchen(art, name, land);
    // a failed request counts as "no picture" for this run only (no file written, so the next start tries again)
    let Some(treffer) = ergebnis else { return Stand::Fehlt };
    let schreib = |v: &Value| { let tmp = d.join(format!("{k}.json.tmp")); let _ = std::fs::write(&tmp, v.to_string()); let _ = std::fs::rename(&tmp, d.join(format!("{k}.json"))); };
    match treffer {
        None => { schreib(&serde_json::json!({ crate::id("fehlt"): true, crate::id("name"): name })); Stand::Fehlt }
        Some(t) => {
            let datei = d.join(format!("{k}.bild"));
            let tmp = d.join(format!("{k}.bild.tmp"));
            let ok = std::process::Command::new(crate::id("curl")).args([crate::id("-sfL"), crate::id("-m"), "30", crate::id("--max-filesize"), "8000000", crate::id("-A"), UA, crate::id("-o")]).arg(&tmp).arg(&t.url).status().map_or(false, |s| s.success());
            // only a picture the game can show (the bytes come from the web: decode before keeping)
            if !ok || image::load_from_memory(&std::fs::read(&tmp).unwrap_or_default()).is_err() { let _ = std::fs::remove_file(&tmp); return Stand::Fehlt; }
            let _ = std::fs::rename(&tmp, &datei);
            schreib(&serde_json::json!({ crate::id("name"): name, crate::id("datei"): t.datei, crate::id("lizenz"): t.lizenz, crate::id("urheber"): t.urheber, crate::id("seite"): t.seite, crate::id("nachweis"): t.nachweis, crate::id("bezeichnung"): t.bezeichnung }));
            Stand::Da(Bild { datei, nachweis: t.nachweis })
        }
    }
}

#[derive(Debug)]
struct Treffer { url: String, datei: String, lizenz: String, urheber: String, seite: String, nachweis: String, bezeichnung: String }

fn json(url: &str, params: &[(&str, &str)]) -> Option<Value> {
    let mut c = std::process::Command::new(crate::id("curl"));
    c.args([crate::id("-sf"), crate::id("-m"), "15", crate::id("-A"), UA, crate::id("--get")]);
    for (a, b) in params { c.arg(crate::id("--data-urlencode")).arg(format!("{a}={b}")); }
    let o = c.arg(url).output().ok().filter(|o| o.status.success())?;
    serde_json::from_slice(&o.stdout).ok()
}
const WD: &str = "https://www.wikidata.org/w/api.php"; // roh
const COMMONS: &str = "https://commons.wikimedia.org/w/api.php"; // roh

fn sprache(land: &str) -> &'static str {
    match land { "DE" | "AT" | "CH" => "de", "ES" | "AR" | "MX" => "es", "IT" => "it", "FR" | "BE" => "fr", "PT" | "BR" => "pt", "NL" => "nl", "TR" => "tr", _ => "en" } // roh
}
fn norm(s: &str) -> String { s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect() }
fn ids(claims: &Value, p: &str) -> Vec<String> { claims[p].as_array().map(|a| a.iter().filter_map(|c| c["mainsnak"]["datavalue"]["value"]["id"].as_str().map(String::from)).collect()).unwrap_or_default() }
/// The current home venue: the preferred statement, else one without an end date (P582), else the last one.
fn heim_heute(c: &Value) -> Option<String> {
    let a = c["P115"].as_array()?;
    let id = |x: &Value| x["mainsnak"]["datavalue"]["value"]["id"].as_str().map(String::from);
    a.iter().find(|x| x["rank"] == "preferred").and_then(id)
        .or_else(|| a.iter().filter(|x| x["qualifiers"]["P582"].is_null()).last().and_then(id))
        .or_else(|| a.last().and_then(id))
}
fn datei_von(claims: &Value, p: &str) -> Option<String> { claims[p].as_array()?.iter().find_map(|c| c["mainsnak"]["datavalue"]["value"].as_str().map(String::from)) }

/// Outer None: the network did not answer (try again later); Some(None): there is no free picture.
fn suchen(art: Art, name: &str, land: &str) -> Option<Option<Treffer>> {
    // "BILD / SPORT BILD", "Süddeutsche Zeitung – Sport": the outlet's main name
    let q = if art == Art::Medium { [" / ", " – ", " - ", " ("].iter().fold(name, |n, t| n.split(t).next().unwrap_or(n)).trim().to_string() } else { name.to_string() };
    let mut kandidaten: Vec<(String, String, String)> = vec![];
    for l in [sprache(land), crate::id("en")] {
        let r = json(WD, &[(crate::id("action"), crate::id("wbsearchentities")), (crate::id("search"), &q), (crate::id("language"), l), (crate::id("uselang"), crate::id("en")), (crate::id("type"), crate::id("item")), (crate::id("limit"), "7"), (crate::id("format"), crate::id("json"))])?;
        for x in r["search"].as_array().into_iter().flatten() {
            let id = x["id"].as_str().unwrap_or("").to_string();
            if !kandidaten.iter().any(|k| k.0 == id) { kandidaten.push((id, x["label"].as_str().unwrap_or("").to_string(), x["description"].as_str().unwrap_or("").to_lowercase())); }
        }
        if l == "en" || !kandidaten.is_empty() { break; }
    }
    if kandidaten.is_empty() { return Some(None); }
    let liste = kandidaten.iter().map(|k| k.0.as_str()).collect::<Vec<_>>().join("|");
    let e = json(WD, &[(crate::id("action"), crate::id("wbgetentities")), (crate::id("ids"), &liste), (crate::id("props"), crate::id("claims|labels")), (crate::id("languages"), crate::id("en")), (crate::id("format"), crate::id("json"))])?;
    let mensch = |c: &Value| ids(c, crate::id("P31")).iter().any(|x| x == "Q5");
    for (id, label, besch) in &kandidaten {
        let c = &e["entities"][id]["claims"];
        let (datei, bez, logo) = match art {
            Art::Stadion => {
                let verein = ids(c, crate::id("P31")).iter().any(|x| x == "Q476028") || besch.contains(crate::id("football")) || besch.contains(crate::id("soccer"));
                if !verein || mensch(c) { continue; }
                let Some(ort) = heim_heute(c) else { continue };
                let o = json(WD, &[(crate::id("action"), crate::id("wbgetentities")), (crate::id("ids"), &ort), (crate::id("props"), crate::id("claims|labels")), (crate::id("languages"), crate::id("en")), (crate::id("format"), crate::id("json"))])?;
                let Some(f) = datei_von(&o["entities"][&ort]["claims"], crate::id("P18")) else { continue };
                (f, o["entities"][&ort]["labels"]["en"]["value"].as_str().unwrap_or("").to_string(), false)
            }
            Art::Person => {
                // shortcut: a namesake who is also a journalist would pass; the name must match exactly
                let beruf = ["Q1930187", "Q13219587", "Q2986228", "Q947873", "Q1371925", "Q1607826", "Q17351648"]; // roh
                let passt = mensch(c) && norm(label) == norm(name) && (ids(c, "P106").iter().any(|x| beruf.contains(&x.as_str())) || ["journalist", "reporter", "commentator", "presenter", "broadcaster"].iter().any(|w| besch.contains(w))); // roh
                if !passt { continue; }
                let Some(f) = datei_von(c, crate::id("P18")) else { continue };
                (f, label.clone(), false)
            }
            Art::Medium => {
                let medium = ["newspaper", "magazine", "tabloid", "website", "television", "channel", "radio", "podcast", "broadcaster", "media", "news", "sports"].iter().any(|w| besch.contains(w)); // roh
                if mensch(c) || !medium || !(norm(label).contains(&norm(&q)) || norm(&q).contains(&norm(label))) { continue; }
                let Some(f) = datei_von(c, crate::id("P154")).map(|f| (f, true)).or_else(|| datei_von(c, crate::id("P18")).map(|f| (f, false))) else { continue };
                (f.0, label.clone(), f.1)
            }
        };
        if let Some(t) = commons(&datei, logo || art == Art::Medium)? { return Some(Some(Treffer { bezeichnung: bez, ..t })); }
    }
    Some(None)
}

/// The file's thumbnail and licence from Commons; Some(None) when its licence does not fit.
fn commons(datei: &str, nur_frei_ohne_namen: bool) -> Option<Option<Treffer>> {
    let r = json(COMMONS, &[(crate::id("action"), crate::id("query")), (crate::id("titles"), &format!("File:{datei}")), (crate::id("prop"), crate::id("imageinfo")), (crate::id("iiprop"), crate::id("url|extmetadata")), (crate::id("iiurlwidth"), "800"), (crate::id("format"), crate::id("json"))])?;
    let Some(seite) = r["query"]["pages"].as_object().and_then(|p| p.values().next().cloned()) else { return Some(None) };
    let ii = &seite["imageinfo"][0];
    let m = &ii["extmetadata"];
    let lizenz = m["LicenseShortName"]["value"].as_str().unwrap_or("").to_string();
    let Some(art) = lizenz_art(&lizenz) else { return Some(None) };
    if nur_frei_ohne_namen && art != Lizenz::Frei { return Some(None); }
    let urheber = ohne_html(m["Artist"]["value"].as_str().unwrap_or(""));
    let url = ii["thumburl"].as_str().or(ii["url"].as_str()).unwrap_or("").to_string();
    if url.is_empty() { return Some(None); }
    let nachweis = if art == Lizenz::Frei { String::new() } else { format!("{} · {lizenz} · Wikimedia Commons", if urheber.is_empty() { crate::id("unknown") } else { &urheber }) }; // roh
    Some(Some(Treffer { url, datei: datei.to_string(), lizenz, urheber, seite: ii["descriptionurl"].as_str().unwrap_or("").to_string(), nachweis, bezeichnung: String::new() }))
}

#[derive(PartialEq, Debug)]
enum Lizenz { Frei, Namensnennung }
/// CC0 and public domain need no credit; CC BY and CC BY-SA need one; anything else (NC, ND, fair use) is refused.
fn lizenz_art(l: &str) -> Option<Lizenz> {
    let l = l.to_lowercase();
    if l.contains(crate::id("nc")) && l.starts_with(crate::id("cc")) || l.contains(crate::id("-nd")) || l.contains(crate::id(" nd")) { return None; }
    if l.starts_with("cc0") || l.contains("public domain") || l.starts_with("pd") { return Some(Lizenz::Frei); } // roh
    if l.starts_with("cc by") || l.starts_with("cc-by") { return Some(Lizenz::Namensnennung); } // roh
    None
}
fn ohne_html(s: &str) -> String {
    let mut out = String::new();
    let mut tag = false;
    for ch in s.chars() { match ch { '<' => tag = true, '>' => tag = false, _ if !tag => out.push(ch), _ => {} } }
    out.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(60).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lizenzen() {
        assert_eq!(lizenz_art("CC BY-SA 4.0"), Some(Lizenz::Namensnennung));
        assert_eq!(lizenz_art("CC BY 2.0"), Some(Lizenz::Namensnennung));
        assert_eq!(lizenz_art("CC0"), Some(Lizenz::Frei));
        assert_eq!(lizenz_art("Public domain"), Some(Lizenz::Frei));
        assert_eq!(lizenz_art("PD-textlogo"), Some(Lizenz::Frei));
        assert_eq!(lizenz_art("CC BY-NC 2.0"), None);
        assert_eq!(lizenz_art("CC BY-ND 4.0"), None);
        assert_eq!(lizenz_art("Fair use"), None);
        assert_eq!(ohne_html("<a href=\"x\">Jane  Doe</a>"), "Jane Doe");
        assert_ne!(schluessel("Kicker"), schluessel("kicker"));
    }
    /// How many pictures are missing and how many the web fills (needs the network, fills the real cache):
    /// `PESM_MESS_VEREINE=80 cargo test --release netz_messen -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn netz_messen() {
        let sider = crate::fl27::sider();
        let w = crate::world::World::from_pesdb(&sider.join("livecpk/_FL26Megapack/common/etc/pesdb"), 3).unwrap();
        let st = crate::fl27::stadien_index(&sider);
        let ohne: Vec<&crate::world::Club> = w.clubs.iter().filter(|c| !st.contains_key(&c.id)).collect();
        eprintln!("Vereine {}, mit Stadionbild im Spiel {}, ohne {}", w.clubs.len(), w.clubs.len() - ohne.len(), ohne.len());
        let n: usize = std::env::var("PESM_MESS_VEREINE").ok().and_then(|s| s.parse().ok()).unwrap_or(40);
        let mut gefunden = 0;
        // the clubs of the strongest leagues first (those a career meets most)
        let mut probe = ohne.clone();
        probe.sort_by_key(|c| (w.leagues[c.league].stufe == 0, w.leagues[c.league].stufe));
        for c in probe.iter().take(n) { if matches!(laden(Art::Stadion, &c.name, ""), Stand::Da(_)) { gefunden += 1; } }
        eprintln!("Stadien aus dem Netz: {gefunden} von {}", n.min(probe.len()));
        for art in [Art::Person, Art::Medium] {
            let liste: Vec<&(String, String, String, String)> = crate::medien::pool_eintraege().iter().filter(|e| (e.2 == "reporter") == (art == Art::Person)).collect();
            let da = liste.iter().filter(|e| matches!(laden(art, &e.0, &e.1), Stand::Da(_))).count();
            eprintln!("{art:?}: {da} von {} mit freiem Bild", liste.len());
        }
    }
    /// Live check against Wikimedia (needs the network): `cargo test --release netz_live -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn netz_live() {
        for (art, n, l) in [(Art::Stadion, "Sunderland", "GB"), (Art::Stadion, "Bayern München", "DE"), (Art::Medium, "BILD / SPORT BILD", "DE"), (Art::Medium, "Marca", "ES"), (Art::Person, "Marco Lindenqvist", "IT"), (Art::Person, "Jonas Feldmann", "DE")] {
            eprintln!("{art:?} {n}: {:?}", suchen(art, n, l).map(|t| t.map(|t| (t.bezeichnung, t.lizenz, t.nachweis))));
        }
    }
}
