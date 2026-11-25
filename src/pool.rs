//! Inhaltspool im Posteingang (Punkt 5, Teil 1.2): port of pesm/core/inhalt/{komposition,pool,vorgang}.py and the pool
//! part of backend/vorgang.py. Themes (assets/inhalt/pesm/core/inhalt/themen/*.toml), catalogue (katalog.toml),
//! occasions (anlaesse.toml) and texts (pool_*.json, `pool.<thema>.<e|a|r|x>.<id>`, building blocks `pool.b.<slot>`)
//! are the original's, copied unchanged; the numbers that stood in its code are in assets/regeln/posteingang.toml.
//! A conversation: an opening coloured by the player's personality, 4–6 answers (at least one that solves, one that
//! makes it worse and one neutral, when the theme has them), the reaction (how the answer lands with this type) and
//! the outcome; the effects go through PES M's own ways (morale, relation, promises, money, board/fans/media).
use crate::game::*;
use crate::mensch::{arr, cw, zufall};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

fn r(k: &str) -> f32 { crate::regeln::f("posteingang", k) }
fn t(k: &str) -> &'static str { crate::inhalt::t(k) }
fn r_d(k: &str) -> usize { crate::regeln::f("drosselung", k) as usize }

fn pfad(name: &str) -> std::path::PathBuf { crate::inhalt::ordner().join("pesm/core/inhalt").join(name) }
/// The catalogue: strategies, tones, groups, effect weights, limits.
pub fn katalog() -> &'static Value {
    static K: OnceLock<Value> = OnceLock::new();
    K.get_or_init(|| crate::inhalt::lesen(&pfad("katalog.toml")).unwrap_or(Value::Null))
}
fn anlaesse() -> &'static Value {
    static A: OnceLock<Value> = OnceLock::new();
    A.get_or_init(|| crate::inhalt::lesen(&pfad("anlaesse.toml")).map(|v| v["anlass"].clone()).unwrap_or(Value::Null))
}
/// All themes (id → theme with the original's defaults filled in); a theme id twice is an error of the data.
pub fn themen() -> &'static BTreeMap<String, Value> {
    static T: OnceLock<BTreeMap<String, Value>> = OnceLock::new();
    T.get_or_init(|| {
        let mut aus = BTreeMap::new();
        let mut dateien: Vec<_> = std::fs::read_dir(pfad(crate::id("themen"))).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |e| e == "toml")).collect()).unwrap_or_default();
        dateien.sort();
        for f in dateien {
            let Ok(v) = crate::inhalt::lesen(&f) else { continue };
            for (tid, th) in v["thema"].as_object().cloned().unwrap_or_default() {
                let mut m: Map<String, Value> = serde_json::from_value(serde_json::json!({"id": tid, "wenn": [], "kanal": crate::id("mail"), "dringlichkeit": 1, "sperre_tage": 21,
                    "haeufigkeit": 1.0, "eroeffnung": [], "antwort": [], "reaktion": [], "ausgang": []})).unwrap();
                for (k, w) in th.as_object().cloned().unwrap_or_default() { m.insert(k, w); }
                assert!(aus.insert(tid.clone(), Value::Object(m)).is_none(), "Thema {tid} doppelt"); // roh
            }
        }
        aus
    })
}

// ---------------------------------------------------------------- komposition.py
/// Personality type from the traits, else from the character values (0…1); "neutral" when nothing stands out.
pub fn typ(merkmale: &[String], ch: &[f32]) -> &'static str {
    for paar in arr("posteingang", "typ.merkmale") {
        let name = paar[0].as_str().unwrap_or_default();
        if paar[1].as_array().map_or(false, |ids| ids.iter().any(|i| merkmale.iter().any(|m| Some(m.as_str()) == i.as_str()))) {
            return Box::leak(name.to_string().into_boxed_str());
        }
    }
    let w = |i: usize| ch.get(i).copied().unwrap_or(0.5);
    use crate::mensch::{EHRGEIZ, LOYAL, NERVEN, TEMPERAMENT};
    if w(TEMPERAMENT) >= r("typ.hitzig_temperament_ab") { return crate::id("hitzig"); }
    if w(LOYAL) <= r("typ.wechselfreudig_loyalitaet_bis") { return crate::id("wechselfreudig"); }
    if w(NERVEN) <= r("typ.sensibel_nerven_bis") { return crate::id("sensibel"); }
    if w(EHRGEIZ) >= r("typ.ehrgeizig_ehrgeiz_ab") { return crate::id("ehrgeizig"); }
    if w(LOYAL) >= r("typ.loyal_loyalitaet_ab") { return crate::id("loyal"); }
    if w(NERVEN) >= r("typ.gelassen_nerven_ab") && w(TEMPERAMENT) <= r("typ.gelassen_temperament_bis") { return crate::id("gelassen"); }
    crate::id("neutral")
}
/// One element, the same for the same seed; what was already said is skipped while there is something else.
pub fn waehle<'a, T>(liste: &'a [T], samen: &str, vermeide: &[String], schluessel: impl Fn(&T) -> String) -> Option<&'a T> {
    if liste.is_empty() { return None; }
    let i0 = (zufall(&[crate::id("waehle"), samen]) * liste.len() as f32) as usize % liste.len();
    (0..liste.len()).map(|o| &liste[(i0 + o) % liste.len()]).find(|x| !vermeide.contains(&schluessel(x))).or(Some(&liste[i0]))
}
/// The variants of a text: a list `key.0, key.1 …` or one text.
pub fn varianten(key: &str) -> Vec<&'static str> {
    let eins = t(key);
    if !eins.is_empty() { return vec![eins]; }
    (0..).map(|i| t(&format!("{key}.{i}"))).take_while(|s| !s.is_empty()).collect()
}
/// Building block `pool.b.<slot>`; a variant for the type (`pool.b.<slot>.<typ>`) goes first (colouring by personality).
fn baustein(slot: &str, tp: &str, samen: &str) -> &'static str {
    let mut v = varianten(&format!("pool.b.{slot}.{tp}"));
    if v.is_empty() { v = varianten(&format!("pool.b.{slot}")); }
    waehle(&v, &format!("{samen}|{slot}"), &[], |x| x.to_string()).copied().unwrap_or("")
}
/// Puts a template together: `{b.slot}` = building block (two levels deep), `{name}` = variable; spaces tidied.
pub fn komponiere(vorlage: &str, tp: &str, samen: &str, var: &BTreeMap<String, String>, tiefe: u8) -> String {
    let mut aus = String::new();
    let mut rest = vorlage;
    while let Some(a) = rest.find('{') {
        aus.push_str(&rest[..a]);
        let nach = &rest[a + 1..];
        match nach.find('}') {
            Some(e) if nach[..e].chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.') && !nach[..e].is_empty() => {
                let k = &nach[..e];
                if let Some(slot) = k.strip_prefix(crate::id("b.")) {
                    let b = baustein(slot, tp, samen);
                    aus.push_str(&if tiefe < 2 { komponiere(b, tp, &format!("{samen}|{k}"), var, tiefe + 1) } else { b.to_string() });
                } else { aus.push_str(var.get(k).map(|s| s.as_str()).unwrap_or("")); }
                rest = &nach[e + 1..];
            }
            _ => { aus.push('{'); rest = nach; }
        }
    }
    aus.push_str(rest);
    let mut s = aus.split_whitespace().collect::<Vec<_>>().join(" ");
    for z in [",", ".", "!", "?", ";", ":"] { s = s.replace(&format!(" {z}"), z); }
    s.trim().to_string()
}

// ---------------------------------------------------------------- pool.py
/// One condition against the context (missing value = not met): numbers with <, <=, >, >=; lists and texts `feld:a,b`.
pub fn bedingung(b: &str, k: &Map<String, Value>) -> bool {
    let i = b.find(|c: char| !(c.is_ascii_lowercase() || c == '_')).unwrap_or(b.len());
    let (feld, rest) = b.split_at(i);
    let (op, wert) = ["<=", ">=", "<", ">", ":"].iter().find_map(|o| rest.strip_prefix(o).map(|w| (*o, w))).unwrap_or(("?", ""));
    let Some(v) = k.get(feld) else { return false };
    if op == ":" {
        let gesucht: Vec<&str> = wert.split(',').collect();
        return match v { Value::Array(a) => a.iter().any(|x| x.as_str().map_or(false, |s| gesucht.contains(&s))), Value::String(s) => gesucht.contains(&s.as_str()), x => gesucht.contains(&x.to_string().as_str()) };
    }
    let (Some(v), Ok(z)) = (v.as_f64(), wert.parse::<f64>()) else { return false };
    match op { "<" => v < z, "<=" => v <= z, ">" => v > z, ">=" => v >= z, _ => false }
}
pub fn passt(wenn: &Value, k: &Map<String, Value>) -> bool { wenn.as_array().map_or(true, |w| w.iter().all(|b| b.as_str().map_or(true, |b| bedingung(b, k)))) }
/// Sum of the weighted effects (catalogue): positive solves, negative makes it worse.
pub fn bewertung(wirkungen: &Value) -> f32 {
    let kat = &katalog()["wirkung"];
    let mut s = 0.0;
    for w in wirkungen.as_array().into_iter().flatten() {
        let ziel = w["ziel"].as_str().unwrap_or_default();
        let Some(g) = kat[ziel]["gewicht"].as_f64() else { continue };
        let d = w["delta"].as_f64().unwrap_or(0.0);
        s += match ziel { "zusage" => g, "kosten" => g * w["betrag"].as_f64().unwrap_or(0.0) / 1_000_000.0, "nachlass" => g * d * 100.0 / 4.0, "rolle" => 0.0, _ => g * d / 4.0 };
    }
    s as f32
}
pub fn klasse(wirkungen: &Value) -> &'static str {
    let s = bewertung(wirkungen) as f64;
    let b = &katalog()["bewertung"];
    if s >= b["loesend"].as_f64().unwrap_or(2.0) { crate::id("loesend") } else if s <= b["verschlimmernd"].as_f64().unwrap_or(-2.0) { crate::id("verschlimmernd") } else { crate::id("neutral") }
}
/// The pool theme for an occasion: the first whose triggers hold; `erzwingen` (the talk is wanted anyway) takes the first there is.
pub fn thema_fuer(anlass: &str, k: &Map<String, Value>, erzwingen: bool) -> Option<String> {
    let liste: Vec<&str> = anlaesse()[anlass].as_array().into_iter().flatten().filter_map(|x| x.as_str()).filter(|x| themen().contains_key(*x)).collect();
    liste.iter().find(|x| passt(&themen()[**x]["wenn"], k)).or(if erzwingen { liste.first() } else { None }).map(|x| x.to_string())
}
/// Themes whose triggers hold, urgent ones first.
pub fn kandidaten(k: &Map<String, Value>) -> Vec<&'static Value> {
    let mut v: Vec<&Value> = themen().values().filter(|x| passt(&x["wenn"], k)).collect();
    v.sort_by_key(|x| (-x["dringlichkeit"].as_i64().unwrap_or(1), x["id"].as_str().unwrap_or_default().to_string()));
    v
}
fn text(key: &str, tp: &str, samen: &str, var: &BTreeMap<String, String>) -> String {
    let v = varianten(key);
    let vorlage = waehle(&v, &format!("{samen}|{key}"), &[], |x| x.to_string()).copied().unwrap_or("");
    komponiere(vorlage, tp, &format!("{samen}|{key}"), var, 0)
}
/// Of a list with `typ` and `wenn`: the ones for this type first, else the ones without a type, else any that hold.
fn nach_typ<'a>(liste: &'a Value, tp: &str, k: &Map<String, Value>) -> Vec<&'a Value> {
    let ok: Vec<&Value> = liste.as_array().into_iter().flatten().filter(|e| passt(&e["wenn"], k)).collect();
    let typisch: Vec<&Value> = ok.iter().copied().filter(|e| e["typ"].as_array().map_or(false, |a| a.iter().any(|x| x.as_str() == Some(tp)))).collect();
    if !typisch.is_empty() { return typisch; }
    let ohne: Vec<&Value> = ok.iter().copied().filter(|e| e["typ"].as_array().map_or(true, |a| a.is_empty())).collect();
    if ohne.is_empty() { ok } else { ohne }
}
/// The opening: an id not in `vermeide` (the ones shown last to this player) while there is another that fits; (id, text).
pub fn eroeffnung(tid: &str, k: &Map<String, Value>, tp: &str, samen: &str, var: &BTreeMap<String, String>, vermeide: &[String]) -> (String, String) {
    let l = nach_typ(&themen()[tid]["eroeffnung"], tp, k);
    let id = |x: &&Value| x["id"].as_str().unwrap_or_default().to_string();
    // the type's own openings first; when this player has seen them all, the general ones that fit come before a repeat
    let neu = |l: Vec<&'static Value>| -> Vec<&'static Value> { l.into_iter().filter(|x| !vermeide.contains(&id(x))).collect() };
    let alle: Vec<&Value> = themen()[tid]["eroeffnung"].as_array().into_iter().flatten().filter(|e| passt(&e["wenn"], k)).collect();
    let (a, b) = (neu(l.clone()), neu(alle));
    let wahl = if !a.is_empty() { a } else if !b.is_empty() { b } else { l };
    let Some(e) = waehle(&wahl, &format!("{samen}|e"), vermeide, id) else { return (String::new(), String::new()) };
    let id = e["id"].as_str().unwrap_or_default().to_string();
    let text = text(&format!("pool.{tid}.e.{id}"), tp, samen, var);
    (id, text)
}
/// One answer offered: id, strategy, tone, class (solves / neutral / worse) and its text.
#[derive(Clone, Debug)]
pub struct Option_ { pub id: String, pub strategie: String, pub ton: String, pub klasse: &'static str, pub text: String, pub wirkungen: Value }
/// 4 to 6 answers of this round: only offered ones, at least one per class (when the theme has it), different
/// strategies, at most two of one tone; ordered by strategy.
pub fn optionen(tid: &str, k: &Map<String, Value>, tp: &str, samen: &str, var: &BTreeMap<String, String>) -> Vec<Option_> {
    let gr = &katalog()["grenzen"]["optionen_je_runde"];
    let (lo, hi) = (gr[0].as_u64().unwrap_or(4) as usize, gr[1].as_u64().unwrap_or(6) as usize);
    let pool: Vec<(&Value, &'static str)> = themen()[tid]["antwort"].as_array().into_iter().flatten().filter(|a| passt(&a["wenn"], k)).map(|a| (a, klasse(&a["wirkungen"]))).collect();
    let n = lo + (zufall(&[samen, tid, crate::id("n")]) * (hi - lo + 1) as f32) as usize;
    let id = |a: &Value| a["id"].as_str().unwrap_or_default().to_string();
    let mut gew: Vec<(&Value, &'static str)> = vec![];
    for kl in [crate::id("loesend"), crate::id("verschlimmernd"), crate::id("neutral")] {
        let l: Vec<&(&Value, &str)> = pool.iter().filter(|x| x.1 == kl && !gew.iter().any(|g| id(g.0) == id(x.0))).collect();
        if let Some(x) = waehle(&l, &format!("{samen}|{kl}"), &[], |x| id(x.0)) { gew.push(**x); }
    }
    let mut rest: Vec<(&Value, &'static str)> = pool.iter().copied().filter(|x| !gew.iter().any(|g| id(g.0) == id(x.0))).collect();
    rest.sort_by(|a, b| zufall(&[samen, &id(a.0)]).partial_cmp(&zufall(&[samen, &id(b.0)])).unwrap());
    let s = |a: &Value| a["strategie"].as_str().unwrap_or_default().to_string();
    let ton = |a: &Value| a["ton"].as_str().unwrap_or_default().to_string();
    for x in &rest {
        if gew.len() >= n { break; }
        let strategien: Vec<String> = gew.iter().map(|g| s(g.0)).collect();
        if strategien.contains(&s(x.0)) && { let mut u = strategien.clone(); u.sort(); u.dedup(); u.len() } < 3 { continue; }
        if gew.iter().filter(|g| ton(g.0) == ton(x.0)).count() >= 2 { continue; }
        gew.push(*x);
    }
    for x in &rest { if gew.len() >= n.min(pool.len()) { break; } if !gew.iter().any(|g| id(g.0) == id(x.0)) { gew.push(*x); } }
    let reihe = |a: &Value| katalog()["strategien"].as_array().and_then(|l| l.iter().position(|x| x.as_str() == a["strategie"].as_str())).unwrap_or(99);
    gew.sort_by_key(|x| (reihe(x.0), id(x.0)));
    gew.into_iter().map(|(a, kl)| {
        let mut v = var.clone();
        if let Some(fr) = a["wirkungen"].as_array().into_iter().flatten().find(|w| w["ziel"] == "zusage").and_then(|w| w["frist"].as_u64()) { v.insert(crate::id("frist").into(), fr.to_string()); }
        Option_ { id: id(a), strategie: s(a), ton: ton(a), klasse: kl, text: text(&format!("pool.{tid}.a.{}", id(a)), tp, samen, &v), wirkungen: a["wirkungen"].clone() }
    }).collect()
}
/// How the answer lands: its class, moved by the personality type; moody players tip one way or the other by seed.
pub fn gruppe(o: &Option_, tp: &str, samen: &str) -> &'static str {
    let mut stufe: i32 = match o.klasse { "verschlimmernd" => 0, "neutral" => 1, _ => 2 };
    let tw = crate::regeln::wert("posteingang", &format!("typ_wirkung.{tp}"));
    stufe += tw[format!("ton:{}", o.ton).as_str()].as_i64().unwrap_or(0) as i32 + tw[format!("strategie:{}", o.strategie).as_str()].as_i64().unwrap_or(0) as i32;
    if tp == "launisch" {
        let z = zufall(&[samen, crate::id("laune")]);
        stufe += if z > r("typ.launisch_besser_ab") { 1 } else if z < r("typ.launisch_schlechter_unter") { -1 } else { 0 };
    }
    [crate::id("schlecht"), crate::id("neutral"), crate::id("gut")][stufe.clamp(0, 2) as usize]
}
pub fn reaktion(tid: &str, o: &Option_, g: &str, k: &Map<String, Value>, tp: &str, samen: &str, var: &BTreeMap<String, String>) -> String {
    let alle = &themen()[tid]["reaktion"];
    let passend = Value::Array(alle.as_array().into_iter().flatten().filter(|x| x["gruppe"] == g && x["fuer"].as_array().map_or(true, |f| f.is_empty() || f.iter().any(|s| s.as_str() == Some(&o.strategie)))).cloned().collect());
    let l = nach_typ(&passend, tp, k);
    let Some(x) = waehle(&l, &format!("{samen}|r"), &[], |x| x["id"].as_str().unwrap_or_default().to_string()) else { return String::new() };
    text(&format!("pool.{tid}.r.{}", x["id"].as_str().unwrap_or_default()), tp, samen, var)
}
/// The outcome: its kind (from the answer, else from the group) and its closing text with what was decided.
pub fn ausgang(tid: &str, o: &Option_, ausgang_art: Option<&str>, g: &str, tp: &str, samen: &str, var: &BTreeMap<String, String>) -> (String, String) {
    let art = ausgang_art.unwrap_or(match g { "gut" => "geloest", "neutral" => "offen", _ => crate::id("eskalation") }).to_string();
    let alle: Vec<&Value> = themen()[tid]["ausgang"].as_array().into_iter().flatten().collect();
    let ok: Vec<&Value> = alle.iter().copied().filter(|a| a["art"].as_str() == Some(&art)).collect();
    let l = if ok.is_empty() { alle } else { ok };
    let Some(a) = waehle(&l, &format!("{samen}|x"), &[], |x| x["id"].as_str().unwrap_or_default().to_string()) else { return (art, String::new()) };
    let mut v = var.clone();
    v.insert(crate::id("entscheidung").into(), entscheidung(o, var));
    (art, text(&format!("pool.{tid}.x.{}", a["id"].as_str().unwrap_or_default()), tp, samen, &v))
}
fn entscheidung(o: &Option_, var: &BTreeMap<String, String>) -> String {
    let frist = o.wirkungen.as_array().into_iter().flatten().find(|w| w["ziel"] == "zusage").and_then(|w| w["frist"].as_u64());
    let key = if o.strategie == "zusage" && frist.is_none() { "pool.entscheidung.zusage_wort".to_string() } else { format!("pool.entscheidung.{}", o.strategie) };
    let mut s = varianten(&key).first().copied().unwrap_or("").to_string();
    for (k, x) in var { s = s.replace(&format!("{{{k}}}"), x); }
    s.replace("{frist}", &frist.map(|f| f.to_string()).unwrap_or_default())
}
/// The person behind a sender without a player (Teil 2, faces and voices): "Präsidium" → its president, "Presse" → a
/// reporter … (posteingang.toml [von] and [namen], invented names of config/story/absender.json, picked by the club so
/// a career keeps the same people). (name, role word) or None for senders that are a person already.
pub fn absender_person(club_id: u32, from: &str) -> Option<(String, String)> {
    static A: OnceLock<Value> = OnceLock::new();
    let a = A.get_or_init(|| crate::inhalt::lesen(&crate::inhalt::ordner().join("pesm/core/config/story/absender.json")).unwrap_or(Value::Null));
    let rolle = crate::regeln::wert_oder("posteingang", &format!("von.{from}"))?.as_str()?;
    let n = crate::regeln::wert_oder("posteingang", &format!("namen.{rolle}"))?;
    let liste = a[n["liste"].as_str()?].as_array()?;
    if liste.is_empty() { return None; }
    let name = liste[(club_id as usize + n["versatz"].as_u64().unwrap_or(0) as usize) % liste.len()].as_str()?.to_string();
    let sp = if crate::inhalt::sprache() == crate::inhalt::Sprache::En { crate::id("en") } else { crate::id("de") };
    let wort = crate::regeln::wert_oder("posteingang", &format!("rolle.{sp}.{rolle}")).and_then(|v| v.as_str()).unwrap_or_default().to_string();
    Some((name, wort))
}
/// Like `absender_person`, but the club's real president, sporting director or finance director where the list
/// (vereinsfuehrung.toml) knows one.
pub fn absender_echt(club_id: u32, club: &str, from: &str) -> Option<(String, String)> {
    let (name, wort) = absender_person(club_id, from)?;
    let rolle = crate::regeln::wert_oder("posteingang", &format!("von.{from}")).and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default();
    Some((crate::vereinsfuehrung::person(club, crate::vereinsfuehrung::amt(&rolle)).unwrap_or(name), wort))
}
pub fn titel(tid: &str) -> String { varianten(&format!("pool.{tid}.titel")).first().map(|s| s.to_string()).unwrap_or_else(|| tid.replace('_', " ")) }

// ---------------------------------------------------------------- PES M: context, start, answer, daily beat
impl Career {
    /// Context for the conditions, variables for the texts and the personality type of an own player (only read).
    pub fn pool_kontext(&self, p: usize) -> (Map<String, Value>, BTreeMap<String, String>, &'static str) {
        let pl = &self.world.players[p];
        let ch: Vec<f32> = (0..10).map(|i| cw(pl, i) / 100.0).collect();
        let merkmale = crate::entwicklung::merkmale_schaetzen(pl);
        // play time only counts once the season has a few matches and he was there for them (Fabian 23.11.: no
        // "too little play time" before the first matchday); until then it reads as full
        let mindest = crate::regeln::f("gespraeche", "motor.einsatz_ab_spielen") as usize;
        // and only when he really sat out several matches in a row (the texts say "{spiele} Spiele ohne Einsatz")
        let einsatz = if self.spielfrei() || pl.ps.einsatz.len() < mindest || self.results.len() < mindest || (pl.benched as f32) < crate::regeln::f("gespraeche", "motor.bank_ab_spielen") { 1.0 } else { pl.ps.einsatz.iter().sum::<f32>() / pl.ps.einsatz.len() as f32 };
        let g = |k: &str| crate::regeln::f("gespraeche", &format!("motor.{k}"));
        let stimmung = (g("stimmung_start") + (pl.morale - 50.0) * g("moral_einfluss") + pl.ps.bez * g("beziehung_einfluss")).clamp(5.0, 95.0);
        let haken: Vec<String> = self.mensch.versprechen.iter().filter(|v| v.pid == p).rev().take(6)
            .map(|v| format!("zusage_{}", [crate::id("offen"), crate::id("gehalten"), crate::id("gebrochen")][v.stand.min(2) as usize])).collect();
        use crate::mensch::{EGO, EHRGEIZ, FUEHRUNG, LOYAL, TEMPERAMENT};
        let k = serde_json::json!({
            "moral": pl.morale, "form": (crate::mensch::pfeil(pl.form) as f32 - 2.0) * 3.0, "rest": self.monate_rest(p), "alter": pl.age, "einsatz": einsatz,
            "wechsel": pl.ps.wb, "vorstand": self.mensch.vorstand, "fans": self.fans, "medien": self.media,
            "fuehrung": ch[FUEHRUNG], "ehrgeiz": ch[EHRGEIZ], "ego": ch[EGO], "loyalitaet": ch[LOYAL], "temperament": ch[TEMPERAMENT],
            "budget": self.world.clubs[self.own].balance, "potenzial_luecke": pl.pot - pl.ovr, "belastet": pl.ps.einsatz.iter().rev().take(5).filter(|x| **x > 0.9).count(),
            "stimmung": stimmung, "beziehung": pl.ps.bez, "rolle": [crate::mensch::ROLLEN[pl.ps.rolle.min(6) as usize]], "position": [pl.pos.clone()],
            "merkmal": merkmale.clone(), "haken": haken, "kanal": [crate::id("mail"), crate::id("chat"), crate::id("persoenlich")],
        });
        let typ = typ(&merkmale, &ch);
        // the teammate a complaint is about: the one who sat out longest (Fabian 24.11.: name him)
        let gegner = self.squad().into_iter().filter(|&q| q != p && self.world.players[q].youth == 0).max_by_key(|&q| (self.world.players[q].benched, std::cmp::Reverse(self.world.players[q].morale as i32)));
        let var = BTreeMap::from([
            (crate::id("name").to_string(), pl.name.clone()), (crate::id("position").into(), crate::world::pos_de(&pl.pos).to_string()), (crate::id("verein").into(), self.club().name.clone()),
            (crate::id("monate").into(), format!("{:.0}", self.monate_rest(p))), (crate::id("spiele").into(), if pl.benched > 0 { pl.benched.to_string() } else { t("ui.pool.mehreren").to_string() }),
            (crate::id("gegner").into(), gegner.map(|g| self.world.players[g].name.clone()).unwrap_or_else(|| t("ui.pool.mitspieler").to_string())),
            (crate::id("gegner_spiele").into(), gegner.map(|g| self.world.players[g].benched.max(2).to_string()).unwrap_or_else(|| t("ui.pool.mehreren").to_string())),
        ]);
        (k.as_object().cloned().unwrap_or_default(), var, typ)
    }
    /// Starts a pool conversation with an own player in the inbox: opening and answers; false when nothing fits.
    pub fn pool_starten(&mut self, p: usize, tid: &str) -> bool {
        let Some(th) = themen().get(tid) else { return false };
        let abs = self.abs();
        let (k, var, tp) = self.pool_kontext(p);
        let samen = format!("{abs}|{tid}|{}", self.world.players[p].id);
        // repeat protection (Teil 2): the openings this player last saw of this theme come again only after all others that fit
        let merk = format!("{tid}|{}", self.world.players[p].id);
        let gezeigt = self.pool_gezeigt.iter().find(|x| x.0 == merk).map(|x| x.1.clone()).unwrap_or_default();
        let (eid, text) = eroeffnung(tid, &k, tp, &samen, &var, &gezeigt);
        let ops = optionen(tid, &k, tp, &samen, &var);
        if text.is_empty() || ops.is_empty() { return false; }
        let n_e = th["eroeffnung"].as_array().map_or(1, |a| a.len()).min(r("wiederholung.merken") as usize);
        match self.pool_gezeigt.iter_mut().find(|x| x.0 == merk) {
            Some(x) => { x.1.retain(|i| *i != eid); x.1.push(eid); while x.1.len() >= n_e { x.1.remove(0); } }
            None => self.pool_gezeigt.push((merk, vec![eid])),
        }
        let sperre = th["sperre_tage"].as_u64().unwrap_or(21) as u32;
        let pl = &mut self.world.players[p];
        pl.ps.sperre.retain(|s| abs < s.1);
        pl.ps.sperre.push((format!("pool:{tid}"), abs + sperre));
        let n = pl.name.clone();
        let absender = th["absender"].as_str().unwrap_or(crate::id("spieler"));
        // who speaks is always named with his role (Fabian 24.11.: player or assistant was not clear); the captain
        // speaks about the player, so he is not the player himself
        let kap = self.kapitaen().filter(|&k| k != p).map(|k| self.world.players[k].name.clone());
        let from = if absender == "spieler" { n.clone() } else if let (true, Some(k)) = (absender == "kapitaen", kap) { crate::tf!("ui.pool.von_kapitaen", k) } else if absender == "co_trainer" || absender == "kapitaen" { crate::tf!("ui.pool.von_co", self.co_name.clone()) } else { crate::regeln::wert("posteingang", &format!("rolle.de.{absender}")).as_str().unwrap_or(absender).to_string() };
        let cat = crate::regeln::wert("posteingang", &format!("ordner.{}", th["kategorie"].as_str().unwrap_or(crate::id("spieler")))).as_u64().unwrap_or(1) as u8;
        let opts = ops.iter().map(|o| (o.id.clone(), o.text.clone())).collect();
        self.msg(cat, &from, &format!("{n} · {}", titel(tid)), &text, Action::Wahl { art: format!("pool:{tid}:{samen}"), bezug: vec![p], optionen: opts });
        true
    }
    /// The answer to a pool conversation: reaction and outcome as text; the effects go through PES M's ways. After
    /// three answers of one player in seven days the mood effects stop (the original's limit), the rest still counts.
    pub fn pool_antwort(&mut self, rest: &str, p: usize, key: &str) -> String {
        let Some((tid, samen)) = rest.split_once(':') else { return String::new() };
        let Some(a) = themen().get(tid).and_then(|th| th["antwort"].as_array()).and_then(|l| l.iter().find(|a| a["id"].as_str() == Some(key))) else { return String::new() };
        let (k, var, tp) = self.pool_kontext(p);
        let o = Option_ { id: key.into(), strategie: a["strategie"].as_str().unwrap_or_default().into(), ton: a["ton"].as_str().unwrap_or_default().into(), klasse: klasse(&a["wirkungen"]), text: String::new(), wirkungen: a["wirkungen"].clone() };
        let samen = format!("{samen}|{key}");
        let g = gruppe(&o, tp, &samen);
        let reakt = reaktion(tid, &o, g, &k, tp, &samen, &var);
        let (art, schluss) = ausgang(tid, &o, a["ausgang"].as_str(), g, tp, &samen, &var);
        self.kette_weiter(tid, p, &art);
        let abs = self.abs();
        let id = self.world.players[p].id;
        self.pool_antworten.retain(|x| abs < x.1 + r("vorgang.stimmung_tage") as u32);
        let zu_viel = self.pool_antworten.iter().filter(|x| x.0 == id).count() >= r("vorgang.stimmung_max_je_woche") as usize;
        self.pool_antworten.push((id, abs));
        let grund = crate::tf!("ui.pool.grund", titel(tid));
        let vorher = (self.vertrauen(p), self.world.players[p].ps.bez, self.mensch.teamgeist);
        let tage = crate::regeln::f("gespraeche", "motor.folge_tage") as u32;
        let mf = r("vorgang.moral_faktor");
        let (mut d, mut zusage) = (0.0, None);
        for w in o.wirkungen.as_array().cloned().unwrap_or_default() {
            let x = w["delta"].as_f64().unwrap_or(0.0) as f32;
            // after three answers in a week the mood effects stop (backend/vorgang.py STIMMUNG)
            let ziel = w["ziel"].as_str().unwrap_or_default();
            let ziel = if zu_viel && matches!(ziel, "moral" | "wechselwunsch") { "" } else { ziel };
            match ziel {
                "moral" => { self.moral_wirkung(p, x * mf, tage, &grund); d += x * mf; }
                // the wish to leave follows the morale (entwicklung.toml [wechsel] moral_hebel 1.6)
                "wechselwunsch" => { let m = -x / 1.6 * mf; self.moral_wirkung(p, m, tage, &grund); d += m; }
                "beziehung" => self.beziehung(p, x, &grund),
                "zusage" => if let Some(art) = w["art"].as_str() { self.versprechen_geben(p, art); zusage = Some(art.to_string()); },
                "kosten" => self.book(t("ui.pool.buchung"), -w["betrag"].as_f64().unwrap_or(0.0)),
                "fanstimmung" => self.klima(x, 0.0, 0.0, &grund),
                "medienbild" => self.klima(0.0, x, 0.0, &grund),
                "vorstandsvertrauen" => self.klima(0.0, 0.0, x, &grund),
                _ => {}
            }
        }
        self.wellen(p, d, zusage.as_deref(), &grund, vorher);
        // the talk also shows on the pitch (Fabian 24.11.): a solved worry lifts his form, an escalation weighs on it
        let fo = match art.as_str() { "geloest" => crate::regeln::f_oder("gespraeche", "antwort.form_geloest", 0.4), "eskalation" => crate::regeln::f_oder("gespraeche", "antwort.form_eskalation", -0.5), _ => 0.0 }; // roh
        if fo != 0.0 { let pl = &mut self.world.players[p]; pl.form = (pl.form + fo).clamp(-3.0, 3.0); }
        let wirkung = if d != 0.0 || fo != 0.0 { crate::tf!("ui.pool.wirkung", format!("{d:+.0}"), format!("{fo:+.1}")) } else { String::new() };
        [reakt, schluss, wirkung].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ")
    }
    /// Stories in several steps (posteingang.toml [[kette]]): a conversation, then after some days a follow-up talk or a
    /// scene (a story rule). After the answer to the start theme or a step's theme, the next step whose `nur_nach` fits the
    /// outcome is scheduled; a scene ends the chain (it has its own follow-ups).
    pub fn kette_weiter(&mut self, tid: &str, p: usize, ausgang_art: &str) {
        let pid = self.world.players[p].id;
        let abs = self.abs();
        for k in arr("posteingang", "kette") {
            let kid = k["id"].as_str().unwrap_or_default().to_string();
            let schritte = k["schritte"].as_array().cloned().unwrap_or_default();
            let ab = if k["start"].as_str() == Some(tid) { 0 }
                else if let Some(i) = self.pool_ketten.iter().position(|x| x.0 == kid && x.1 == pid) {
                    let n = self.pool_ketten[i].2;
                    if schritte.get(n).and_then(|s| s["thema"].as_str()) != Some(tid) { continue; }
                    self.pool_ketten.remove(i);
                    n + 1
                } else { continue };
            let Some((j, s)) = schritte.iter().enumerate().skip(ab).find(|(_, s)| s["nur_nach"].as_array().map_or(true, |l| l.is_empty() || l.iter().any(|x| x.as_str() == Some(ausgang_art)))) else { continue };
            let faellig = abs + s["nach_tagen"].as_u64().unwrap_or(14) as u32;
            if let Some(th) = s["thema"].as_str() { self.pool_folgen.push((th.to_string(), pid, faellig)); self.pool_ketten.push((kid, pid, j)); }
            else if let Some(st) = s["story"].as_str() { self.mensch.story_folgen.push((st.to_string(), Some(p), faellig)); }
        }
    }
    /// Daily beat (backend/vorgang.py pool_auto, without the central throttle and the day phases): pool themes start by
    /// themselves for own players whose triggers hold, by frequency roll and lock time; at most `neu_je_tag` a day, one
    /// open conversation per player, and nothing new while `offen_hoechstens` are waiting. Urgent ones first.
    pub fn pool_tag(&mut self) {
        if !self.feature("gespraeche") { return; } // roh
        let abs = self.abs();
        if abs < self.ruhe_bis { return; }
        // due steps of a chain come first and are never throttled: the story was already begun
        let faellig: Vec<(String, u32)> = self.pool_folgen.iter().filter(|x| abs >= x.2).map(|x| (x.0.clone(), x.1)).collect();
        self.pool_folgen.retain(|x| abs < x.2);
        for (tid, pid) in faellig { if let Some(p) = self.squad().into_iter().find(|&q| self.world.players[q].id == pid) { self.pool_starten(p, &tid); } }
        let offen: Vec<usize> = self.inbox.iter().filter(|m| !m.done).filter_map(|m| match &m.action { Action::Wahl { art, bezug, .. } if art.starts_with("pool:") => bezug.first().copied(), _ => None }).collect();
        let platz = (r("vorgang.neu_je_tag") as usize).min((r("vorgang.offen_hoechstens") as usize).saturating_sub(offen.len()));
        if platz == 0 { return; }
        let nicht: Vec<String> = arr("posteingang", "vorgang.gremien").iter().chain(arr("posteingang", "vorgang.nicht_von_selbst")).filter_map(|x| x.as_str().map(String::from)).collect();
        let mut kand: Vec<(i64, f32, String, usize)> = vec![];
        for p in self.squad() {
            if offen.contains(&p) { continue; }
            let (k, _, _) = self.pool_kontext(p);
            let pid = self.world.players[p].id.to_string();
            for th in kandidaten(&k) {
                let tid = th["id"].as_str().unwrap_or_default();
                if nicht.iter().any(|n| th["kategorie"].as_str() == Some(n)) { continue; }
                if self.world.players[p].ps.sperre.iter().any(|s| s.0 == format!("pool:{tid}") && abs < s.1) { continue; }
                // a row in the dressing room or a late arrival is an event, not a state: the same theme comes back in the
                // whole squad only after its lock time
                if self.pool_zuletzt.iter().any(|z| z.0 == tid && abs < z.1 + th["sperre_tage"].as_u64().unwrap_or(21) as u32) { continue; }
                let wurf = zufall(&[&abs.to_string(), tid, &pid]);
                if wurf >= th["haeufigkeit"].as_f64().unwrap_or(1.0) as f32 { continue; }
                kand.push((th["dringlichkeit"].as_i64().unwrap_or(1), wurf, tid.to_string(), p));
                break; // one candidate per player: the most urgent one that holds
            }
        }
        kand.sort_by(|a, b| (-a.0, a.1).partial_cmp(&(-b.0, b.1)).unwrap());
        let mut themen_heute: Vec<String> = vec![];
        let mut neu = 0;
        // central throttle (core/gespraeche/drosselung.py, drosselung.toml): important themes always; the others share the
        // day and week quota with the players' other concerns (Career::anliegen), at most one of the "noise" level a week
        let stufe = |tid: &str, s: &str| arr("drosselung", &format!("stufe.{s}")).iter().any(|x| x.as_str() == Some(tid));
        kand.sort_by_key(|x| !stufe(&x.2, crate::id("wichtig")));
        self.mensch.anliegen.retain(|&d| abs < d + 7);
        for (_, _, tid, p) in kand {
            if neu >= platz { break; }
            if themen_heute.contains(&tid) { continue; }
            let wichtig = stufe(&tid, crate::id("wichtig"));
            if !wichtig {
                let tag = self.mensch.anliegen.iter().filter(|&&d| d == abs).count();
                let rauschen = stufe(&tid, crate::id("rauschen")) && self.pool_rauschen.iter().any(|&d| abs < d + 7);
                if tag >= r_d("deckel.tag") || self.mensch.anliegen.len() >= r_d("deckel.woche") || rauschen { continue; }
            }
            if self.pool_starten(p, &tid) {
                neu += 1;
                self.pool_zuletzt.retain(|z| z.0 != tid);
                self.pool_zuletzt.push((tid.clone(), abs));
                themen_heute.push(tid.clone());
                if !wichtig { self.mensch.anliegen.push(abs); }
                if stufe(&tid, crate::id("rauschen")) { self.pool_rauschen.retain(|&d| abs < d + 7); self.pool_rauschen.push(abs); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// Fabian 24.11.: a complaint about a teammate names him, with his own games out.
    #[test]
    fn gegner_ist_ein_echter_mitspieler() {
        let mut c = crate::game::Career::new(crate::world::testwelt(8), 2, "Test");
        let sq = c.squad();
        c.world.players[sq[5]].benched = 6;
        let (_, var, _) = c.pool_kontext(sq[0]);
        assert_eq!(var["gegner"], c.world.players[sq[5]].name);
        assert_eq!(var["gegner_spiele"], "6");
    }
    /// Fabian 24.11.: after the season's end nobody complains about playing time.
    #[test]
    fn keine_spielzeitklage_nach_saisonende() {
        let mut c = crate::game::Career::new(crate::world::testwelt(7), 0, "Test");
        c.world.day = crate::world::LEAGUE_END + 10;
        let p = c.squad()[20];
        c.world.players[p].ps.einsatz = vec![0.0; 8];
        c.world.players[p].benched = 8;
        assert_eq!(c.pool_kontext(p).0["einsatz"].as_f64(), Some(1.0));
        let n = c.inbox.len();
        c.anliegen(p, "spielzeit", true);
        assert_eq!(c.inbox.len(), n, "no playing-time complaint in the break");
    }
    #[test]
    fn keine_spielzeitklage_vor_dem_ersten_spieltag() {
        let w = crate::world::testwelt(7);
        let c = crate::game::Career::new(w, 0, "Test");
        for p in c.squad() { assert_eq!(c.pool_kontext(p).0["einsatz"].as_f64(), Some(1.0), "before matches play time counts as full"); }
    }

    use super::*;
    #[test]
    fn themen_katalog_und_komposition() {
        assert!(themen().len() > 50, "{} Themen", themen().len());
        assert_eq!(klasse(&serde_json::json!([{"ziel": "moral", "delta": 6}, {"ziel": "beziehung", "delta": 3}])), "loesend");
        assert_eq!(klasse(&serde_json::json!([{"ziel": "moral", "delta": -6}, {"ziel": "beziehung", "delta": -4}])), "verschlimmernd");
        let k: Map<String, Value> = serde_json::json!({"rolle": ["rotationsspieler"], "einsatz": 0.1, "alter": 27, "budget": 1e6, "beziehung": 5, "stimmung": 50}).as_object().unwrap().clone();
        assert!(bedingung("rolle:rotationsspieler,perspektivspieler", &k) && bedingung("einsatz<0.25", &k) && !bedingung("alter<25", &k) && !bedingung("gibtsnicht>1", &k));
        assert_eq!(thema_fuer("spielzeit", &k, false).as_deref(), Some("reservist_unzufrieden"));
        let var = BTreeMap::from([("name".to_string(), "Paul Test".to_string())]);
        let (eid, e) = eroeffnung("reservist_unzufrieden", &k, "hitzig", "s1", &var, &[]);
        assert!(e.len() > 20 && !e.contains('{') && !e.contains("  "), "{e}");
        assert_eq!(e, eroeffnung("reservist_unzufrieden", &k, "hitzig", "s1", &var, &[]).1, "the same seed gives the same text");
        assert_ne!(eroeffnung("reservist_unzufrieden", &k, "hitzig", "s1", &var, &[eid.clone()]).0, eid, "an opening shown last is avoided");
        let ops = optionen("reservist_unzufrieden", &k, "hitzig", "s1", &var);
        assert!((4..=6).contains(&ops.len()), "{}", ops.len());
        for kl in ["loesend", "verschlimmernd", "neutral"] { assert!(ops.iter().any(|o| o.klasse == kl), "class {kl} offered"); }
        assert!(ops.iter().all(|o| !o.text.is_empty() && !o.text.contains('{')));
        // a hothead takes a hard word badly, a caring answer well
        let hart = ops.iter().find(|o| o.ton == "hart").cloned().unwrap_or(Option_ { id: "a4".into(), strategie: "ablehnen".into(), ton: "hart".into(), klasse: "verschlimmernd", text: String::new(), wirkungen: Value::Null });
        assert_eq!(gruppe(&hart, "hitzig", "s"), "schlecht");
        let (art, x) = ausgang("reservist_unzufrieden", &hart, None, "schlecht", "hitzig", "s", &var);
        assert_eq!(art, "eskalation");
        assert!(!x.is_empty() && !x.contains('{'), "{x}");
    }
    /// The pool's texts of one language (de: pesm/core/texts/pool_*.json, en: texts/en): key → variants.
    fn pool_texte(en: bool) -> BTreeMap<String, Vec<String>> {
        let mut d = crate::inhalt::ordner().join("pesm/core/texts");
        if en { d = d.join("en"); }
        let mut aus = BTreeMap::new();
        fn flach(v: &Value, pfad: &str, aus: &mut BTreeMap<String, Vec<String>>) {
            match v {
                Value::Object(o) => for (k, w) in o { flach(w, &if pfad.is_empty() { k.clone() } else { format!("{pfad}.{k}") }, aus) },
                Value::Array(a) => { aus.insert(pfad.to_string(), a.iter().filter_map(|x| x.as_str().map(String::from)).collect()); }
                Value::String(s) => { aus.insert(pfad.to_string(), vec![s.clone()]); }
                _ => {}
            }
        }
        let mut dateien: Vec<_> = std::fs::read_dir(&d).unwrap().flatten().map(|e| e.path()).filter(|p| p.file_name().map_or(false, |n| n.to_string_lossy().starts_with("pool_") && n.to_string_lossy().ends_with(".json"))).collect();
        dateien.sort();
        for f in dateien { flach(&crate::inhalt::lesen(&f).unwrap(), "", &mut aus); }
        aus
    }
    fn platzhalter(s: &str) -> Vec<String> {
        let mut out = vec![];
        let mut rest = s;
        while let Some(a) = rest.find('{') {
            let nach = &rest[a + 1..];
            match nach.find('}') { Some(e) if !nach[..e].is_empty() && nach[..e].chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.') => { out.push(nach[..e].to_string()); rest = &nach[e + 1..]; } _ => rest = nach }
        }
        out
    }
    /// Teil 1.3: the content pool's quality rules (pesm/core/inhalt/pruefung.py), every finding listed.
    fn themenpruefung() -> Vec<String> {
        let kat = katalog();
        let g = &kat["grenzen"];
        let liste = |v: &Value| -> Vec<String> { v.as_array().into_iter().flatten().filter_map(|x| x.as_str().map(String::from)).collect() };
        let variablen = liste(crate::regeln::wert("posteingang", "pruefung.variablen"));
        let berecht = liste(crate::regeln::wert("posteingang", "pruefung.berechtigungen"));
        let stil = liste(crate::regeln::wert("posteingang", "pruefung.stilplatz"));
        let texte = [pool_texte(false), pool_texte(true)];
        let mut f: Vec<String> = vec![];
        let bed = |b: &str| -> Option<String> {
            let i = b.find(|c: char| !(c.is_ascii_lowercase() || c == '_')).unwrap_or(b.len());
            let (feld, rest) = b.split_at(i);
            let Some((op, z)) = ["<=", ">=", "<", ">", ":"].iter().find_map(|o| rest.strip_prefix(o).map(|w| (*o, w))) else { return Some("unlesbar".into()) };
            if feld.is_empty() || z.is_empty() { return Some("unlesbar".into()); }
            if op == ":" { return (kat["felder"].get(feld).is_none() && !liste(&kat["listen"]).iter().any(|l| l == feld)).then(|| "Feld unbekannt".into()); }
            let Some(sp) = kat["felder"].get(feld) else { return Some("Feld unbekannt".into()) };
            let Ok(z) = z.parse::<f64>() else { return Some("Zahl unlesbar".into()) };
            let (lo, hi) = (sp[0].as_f64().unwrap(), sp[1].as_f64().unwrap());
            (!(lo <= z && z <= hi)).then(|| format!("Zahl {z} außerhalb {lo}…{hi}"))
        };
        let arten = [("eroeffnung", "e"), ("antwort", "a"), ("reaktion", "r"), ("ausgang", "x")];
        for (tid, th) in themen() {
            for feld in ["kategorie", "absender", "kanal", "dringlichkeit"] { if th.get(feld).is_none() { f.push(format!("{tid}: Feld {feld} fehlt")); } }
            if let Some(b) = th["berechtigung"].as_str() { if !berecht.iter().any(|x| x == b) { f.push(format!("{tid}: Berechtigung {b} unbekannt")); } }
            if !liste(&kat["kanaele"]).iter().any(|k| Some(k.as_str()) == th["kanal"].as_str()) { f.push(format!("{tid}: Kanal {} unbekannt", th["kanal"])); }
            for x in th["variablen"].as_object().into_iter().flatten().map(|x| x.0) { if !variablen.contains(x) { f.push(format!("{tid}: Variable {x} unbekannt")); } }
            let wenn = liste(&th["wenn"]);
            if wenn.is_empty() { f.push(format!("{tid}: kein Auslöser")); }
            for b in &wenn { if let Some(e) = bed(b) { f.push(format!("{tid}: Bedingung {b}: {e}")); } }
            let kategorie = th["kategorie"].as_str().unwrap_or_default();
            let gk = |k: &str| kat["grenzen_kategorie"][kategorie].get(k).unwrap_or(&g[k]).clone();
            for (art, sp) in [("eroeffnung", "eroeffnungen"), ("antwort", "antworten"), ("ausgang", "ausgaenge")] {
                let n = th[art].as_array().map_or(0, |a| a.len()) as u64;
                let s = gk(sp);
                if !(s[0].as_u64().unwrap() <= n && n <= s[1].as_u64().unwrap()) { f.push(format!("{tid}: {n} {art}-Einträge (erlaubt {}–{})", s[0], s[1])); }
            }
            for gr in liste(&kat["gruppen"]) {
                let n = th["reaktion"].as_array().into_iter().flatten().filter(|r| r["gruppe"].as_str() == Some(&gr)).count() as u64;
                let s = gk("reaktionen");
                if !(s[0].as_u64().unwrap() <= n && n <= s[1].as_u64().unwrap()) { f.push(format!("{tid}: {n} Reaktionen der Gruppe {gr}")); }
            }
            for (art, _) in arten {
                let ids: Vec<&str> = th[art].as_array().into_iter().flatten().filter_map(|x| x["id"].as_str()).collect();
                let mut u = ids.clone(); u.sort(); u.dedup();
                if u.len() != ids.len() { f.push(format!("{tid}: doppelte Kennung bei {art}")); }
                for x in th[art].as_array().into_iter().flatten() { for b in liste(&x["wenn"]) { if let Some(e) = bed(&b) { f.push(format!("{tid}.{art}.{}: Bedingung {b}: {e}", x["id"])); } } }
            }
            let erlaubt = kat["kategorie_ziele"].get(kategorie).map(|v| liste(v));
            for a in th["antwort"].as_array().into_iter().flatten() {
                let aid = a["id"].as_str().unwrap_or_default();
                if !liste(&kat["strategien"]).iter().any(|x| Some(x.as_str()) == a["strategie"].as_str()) { f.push(format!("{tid}.{aid}: Strategie unbekannt")); }
                if !liste(&kat["toene"]).iter().any(|x| Some(x.as_str()) == a["ton"].as_str()) { f.push(format!("{tid}.{aid}: Ton unbekannt")); }
                if let Some(x) = a["ausgang"].as_str() { if !liste(&kat["ausgang_arten"]).iter().any(|y| y == x) { f.push(format!("{tid}.{aid}: Ausgang {x} unbekannt")); } }
                for w in a["wirkungen"].as_array().into_iter().flatten() {
                    let z = w["ziel"].as_str().unwrap_or_default();
                    let Some(k) = kat["wirkung"].get(z) else { f.push(format!("{tid}.{aid}: Wirkung {z} nicht im Katalog")); continue };
                    if let Some(e) = &erlaubt { if !e.iter().any(|x| x == z) { f.push(format!("{tid}.{aid}: Wirkung {z} passt nicht zur Kategorie {kategorie}")); } }
                    let im = |v: f64, b: &Value| b[0].as_f64().unwrap() <= v && v <= b[1].as_f64().unwrap();
                    let ok = match z {
                        "zusage" => liste(&k["arten"]).iter().any(|x| Some(x.as_str()) == w["art"].as_str()) && im(w["frist"].as_f64().unwrap_or(0.0), &k["frist"]),
                        "kosten" => im(w["betrag"].as_f64().unwrap_or(0.0), &k["bereich"]),
                        "rolle" => liste(&k["rollen"]).iter().any(|x| Some(x.as_str()) == w["nach"].as_str()),
                        _ => w.get("delta").is_some() && im(w["delta"].as_f64().unwrap_or(0.0), &k["bereich"]),
                    };
                    if !ok { f.push(format!("{tid}.{aid}: Wirkung {z} ungültig oder außerhalb des Bereichs")); }
                }
            }
            let klassen: Vec<&str> = th["antwort"].as_array().into_iter().flatten().map(|a| klasse(&a["wirkungen"])).collect();
            for kl in ["loesend", "verschlimmernd", "neutral"] { if !klassen.contains(&kl) { f.push(format!("{tid}: keine {kl}e Antwort")); } }
            let mut st: Vec<&str> = th["antwort"].as_array().into_iter().flatten().filter_map(|a| a["strategie"].as_str()).collect(); st.sort(); st.dedup();
            if st.len() < 3 { f.push(format!("{tid}: weniger als drei Strategien")); }
            let mut to: Vec<&str> = th["antwort"].as_array().into_iter().flatten().filter_map(|a| a["ton"].as_str()).collect(); to.sort(); to.dedup();
            if to.len() < 2 { f.push(format!("{tid}: weniger als zwei Töne")); }
            for (si, sp) in ["de", "en"].iter().enumerate() {
                let tx = &texte[si];
                if tx.get(&format!("pool.{tid}.titel")).map_or(true, |v| v.is_empty()) { f.push(format!("{sp} pool.{tid}.titel: Titel fehlt")); }
                for (art, kurz) in arten {
                    for x in th[art].as_array().into_iter().flatten() {
                        let key = format!("pool.{tid}.{kurz}.{}", x["id"].as_str().unwrap_or_default());
                        let v = tx.get(&key).cloned().unwrap_or_default();
                        if v.is_empty() { f.push(format!("{sp} {key}: Text fehlt")); }
                        for s in &v {
                            let n = s.chars().count() as u64;
                            if s.trim().is_empty() { f.push(format!("{sp} {key}: leerer Text")); }
                            else if !(g["text_min"].as_u64().unwrap() <= n && n <= g["text_max"].as_u64().unwrap()) { f.push(format!("{sp} {key}: Länge {n}")); }
                            let ohne: String = { let mut o = String::new(); let mut tief = 0; for ch in s.chars() { if ch == '{' { tief += 1; } else if ch == '}' { tief -= 1; } else if tief == 0 { o.push(ch); } } o };
                            if ohne.chars().any(|c| c.is_ascii_digit()) { f.push(format!("{sp} {key}: feste Zahl im Text")); }
                            if s.contains('%') || stil.iter().any(|w| s.to_lowercase().contains(w.as_str())) { f.push(format!("{sp} {key}: Platzhaltertext oder %-Zeichen")); }
                            for p in platzhalter(s) {
                                if let Some(b) = p.strip_prefix("b.") { if tx.get(&format!("pool.b.{b}")).map_or(true, |v| v.is_empty()) { f.push(format!("{sp} {key}: Baustein {p} fehlt")); } }
                                else if !variablen.contains(&p) { f.push(format!("{sp} {key}: Variable {{{p}}} unbekannt")); }
                            }
                        }
                    }
                }
            }
            for (art, kurz) in arten {
                for x in th[art].as_array().into_iter().flatten() {
                    let key = format!("pool.{tid}.{kurz}.{}", x["id"].as_str().unwrap_or_default());
                    let (a, b) = (texte[0].get(&key).cloned().unwrap_or_default(), texte[1].get(&key).cloned().unwrap_or_default());
                    if a.len() != b.len() { f.push(format!("{key}: de hat {} Varianten, en {}", a.len(), b.len())); }
                    else if a.iter().zip(&b).any(|(x, y)| { let (mut p, mut q) = (platzhalter(x), platzhalter(y)); p.sort(); q.sort(); p != q }) { f.push(format!("{key}: Platzhalter in de und en verschieden")); }
                }
            }
            // every type and group has a reaction (its own, or a neutral one whose building blocks are coloured for the type)
            for tp in liste(&kat["typen"]) {
                for gr in liste(&kat["gruppen"]) {
                    let ok = th["reaktion"].as_array().into_iter().flatten().filter(|r| r["gruppe"].as_str() == Some(&gr)).any(|r| {
                        if liste(&r["typ"]).contains(&tp) { return true; }
                        if !liste(&r["typ"]).is_empty() { return false; }
                        let key = format!("pool.{tid}.r.{}", r["id"].as_str().unwrap_or_default());
                        tp == "neutral" || texte[0].get(&key).into_iter().flatten().flat_map(|s| platzhalter(s)).filter_map(|p| p.strip_prefix("b.").map(String::from))
                            .any(|slot| texte[0].get(&format!("pool.b.{slot}.{tp}")).map_or(false, |v| !v.is_empty()))
                    });
                    if !ok { f.push(format!("{tid}: keine Reaktion für Typ {tp} in der Gruppe {gr}")); }
                }
            }
        }
        for key in texte[0].keys().chain(texte[1].keys()) { if !texte[0].contains_key(key) || !texte[1].contains_key(key) { f.push(format!("{key}: nur in einer Sprache")); } }
        for (si, sp) in ["de", "en"].iter().enumerate() {
            let mut gesehen: BTreeMap<&str, &str> = BTreeMap::new();
            for (key, v) in &texte[si] {
                for s in v { if let Some(k) = gesehen.get(s.as_str()) { if *k != key { f.push(format!("{sp} Dublette: {key} und {k}")); } } else { gesehen.insert(s, key); } }
            }
        }
        f.sort(); f.dedup();
        f
    }
    #[test]
    fn themenpruefung_wie_im_original() {
        let (de, en) = (pool_texte(false), pool_texte(true));
        assert!(de.len() > 2000 && en.len() == de.len(), "{} / {} pool texts", de.len(), en.len());
        let f = themenpruefung();
        assert!(f.is_empty(), "{} Befunde, z. B.:\n{}", f.len(), f.iter().take(30).cloned().collect::<Vec<_>>().join("\n"));
    }

    /// Teil 2: in 60 days no opening comes twice to the same player while the theme has others; every message has a
    /// sender; a chain's next step is scheduled after the answer.
    #[test]
    fn sechzig_tage_ohne_wiederholung_und_ketten() {
        let w = crate::world::testwelt(7);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(0); // roh
        let mut c = Career::new(w, own, "Test"); // roh
        let mut eroeffnungen: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for _ in 0..60 {
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            for i in 0..c.inbox.len() {
                if c.inbox[i].done { continue; }
                if let Action::Wahl { art, bezug, .. } = &c.inbox[i].action {
                    if let Some(rest) = art.strip_prefix("pool:") {
                        let tid = rest.split(':').next().unwrap().to_string();
                        eroeffnungen.entry(format!("{tid}|{}", bezug[0])).or_default().push(c.inbox[i].body.clone());
                    }
                    c.answer(i, true, 0);
                }
            }
            while c.press.is_some() { c.answer_press(Some(0)); }
            c.ultimatum = None;
            c.end_day().unwrap();
        }
        // the weekly paper quotes the coach and sets it against the results since
        let woche: Vec<&crate::game::Msg> = c.inbox.iter().filter(|m| m.title == crate::inhalt::t("ui.zeitung.wochenschau")).collect();
        assert!(woche.len() >= 4, "{} weekly papers in 60 days", woche.len());
        assert!(woche.iter().any(|m| m.body.contains('„')), "a weekly paper quotes the coach");
        assert!(eroeffnungen.len() >= 5, "only {} pool conversations in 60 days", eroeffnungen.len());
        for (k, v) in &eroeffnungen {
            let n = themen()[k.split('|').next().unwrap()]["eroeffnung"].as_array().unwrap().len();
            let mut u = v.clone(); u.sort(); u.dedup();
            assert!(u.len() == v.len() || v.len() > n, "{k}: an opening came twice: {v:?}");
        }
        assert!(c.inbox.iter().all(|m| !m.from.trim().is_empty()), "every message has a sender");
        // offices have a person with a role; a player is a person already
        let (name, rolle) = absender_person(42, "Präsidium").expect("the board has a president"); // roh
        assert!(!name.is_empty() && rolle == "Präsidium", "{name} {rolle}");
        assert_eq!(absender_person(42, "Präsidium"), absender_person(42, "Präsident"), "same person for the same club"); // roh
        assert!(absender_person(42, "David Raya").is_none()); // roh
        // a chain: the answer to its start theme schedules the next step
        let p = c.squad()[0];
        c.kette_weiter("heimweh_familie", p, "geloest");
        assert!(c.mensch.story_folgen.iter().any(|x| x.0 == "kb_heimweh_gut" && x.1 == Some(p)), "good outcome: the good scene follows");
        c.kette_weiter("neuzugang_aussen_vor", p, "offen");
        assert!(c.pool_folgen.iter().any(|x| x.0 == "mentor_vorschlag"), "the follow-up talk is scheduled");
        c.kette_weiter("mentor_vorschlag", p, "offen");
        assert!(c.mensch.story_folgen.iter().any(|x| x.0 == "kb_neuling"), "after the follow-up talk the scene");
    }

    #[test]
    fn jedes_thema_hat_texte() {
        // every theme's openings, answers, reactions and outcomes have a text that composes without leftovers
        let var = BTreeMap::new();
        for (tid, th) in themen() {
            for (teil, l) in [("e", "eroeffnung"), ("a", "antwort"), ("r", "reaktion"), ("x", "ausgang")] {
                for x in th[l].as_array().into_iter().flatten() {
                    let s = text(&format!("pool.{tid}.{teil}.{}", x["id"].as_str().unwrap()), "neutral", "t", &var);
                    assert!(!s.is_empty() && !s.contains("{b."), "{tid}.{teil}.{}: „{s}“", x["id"]);
                }
            }
        }
    }
}
