//! The rule numbers of the Python original (core/**/*.toml, core/config/*.json), copied unchanged into
//! assets/regeln/. Code asks for a number by file and dotted path; a missing number is a bug (test `alle_zahlen_da`).
use std::sync::OnceLock;

const DATEIEN: &[(&str, &str)] = &[
    (crate::id("entwicklung"), include_str!("../assets/regeln/entwicklung.toml")),
    (crate::id("gespraeche"), include_str!("../assets/regeln/gespraeche.toml")),
    (crate::id("vereinswelt"), include_str!("../assets/regeln/vereinswelt.toml")),
    (crate::id("laufbahn"), include_str!("../assets/regeln/laufbahn.toml")),
    (crate::id("manager"), include_str!("../assets/regeln/manager.toml")),
    (crate::id("medien"), include_str!("../assets/regeln/medien.toml")),
    (crate::id("transfers"), include_str!("../assets/regeln/transfers.toml")),
    (crate::id("scouting"), include_str!("../assets/regeln/scouting.toml")),
    (crate::id("assistent"), include_str!("../assets/regeln/regeln.toml")),
    (crate::id("akademie"), include_str!("../assets/regeln/akademie.toml")),
    (crate::id("jugend"), include_str!("../assets/regeln/jugend.toml")),
    (crate::id("jugendakademie"), include_str!("../assets/regeln/jugendakademie.toml")),
    (crate::id("positionsrollen"), include_str!("../assets/regeln/positionsrollen.toml")),
    ("ki_trainer", include_str!("../assets/regeln/ki_trainer.toml")),
    (crate::id("einheiten"), include_str!("../assets/regeln/einheiten.toml")),
    (crate::id("kabine"), include_str!("../assets/regeln/kabine.toml")),
    (crate::id("konflikte"), include_str!("../assets/regeln/konflikte.toml")),
    (crate::id("personal"), include_str!("../assets/regeln/personal.toml")),
    (crate::id("gegner"), include_str!("../assets/regeln/gegner.toml")),
    ("budget_start", include_str!("../assets/regeln/budget_start.toml")),
    (crate::id("farm"), include_str!("../assets/regeln/farm.toml")),
    (crate::id("medizin"), include_str!("../assets/regeln/medizin.toml")),
    (crate::id("praemien"), include_str!("../assets/regeln/praemien.toml")),
    (crate::id("spieltag"), include_str!("../assets/regeln/spieltag.toml")),
    (crate::id("ligaprofile"), include_str!("../assets/regeln/ligaprofile.toml")),
    (crate::id("namen"), include_str!("../assets/regeln/namen.toml")),
    (crate::id("entwicklung2"), include_str!("../assets/regeln/entwicklung2.toml")),
    (crate::id("entwicklung3"), include_str!("../assets/regeln/entwicklung3.toml")),
    ("entwicklung_pesm", include_str!("../assets/regeln/entwicklung_pesm.toml")),
    ("live_spiel", include_str!("../assets/regeln/live_spiel.toml")),
    (crate::id("posteingang"), include_str!("../assets/regeln/posteingang.toml")),
    (crate::id("drosselung"), include_str!("../assets/regeln/drosselung.toml")),
    (crate::id("spielgefuehl"), include_str!("../assets/regeln/spielgefuehl.toml")),
    (crate::id("klauseln"), include_str!("../assets/regeln/klauseln.toml")),
    (crate::id("meilensteine"), include_str!("../assets/regeln/meilensteine.toml")),
    (crate::id("kaderregeln"), include_str!("../assets/regeln/kaderregeln.toml")),
    (crate::id("vorbereitung"), include_str!("../assets/regeln/vorbereitung.toml")),
    (crate::id("turniere"), include_str!("../assets/regeln/turniere.toml")),
    (crate::id("berater"), include_str!("../assets/regeln/berater.toml")),
    (crate::id("trainingslager"), include_str!("../assets/regeln/trainingslager.toml")),
    (crate::id("wettbewerbslook"), include_str!("../assets/regeln/wettbewerbslook.toml")),
];
const JSON: &[(&str, &str)] = &[
    (crate::id("persoenlichkeit"), include_str!("../assets/regeln/persoenlichkeit.json")),
    (crate::id("hierarchie"), include_str!("../assets/regeln/hierarchie.json")),
    (crate::id("klima"), include_str!("../assets/regeln/klima.json")),
    (crate::id("rollen"), include_str!("../assets/regeln/rollen.json")),
    (crate::id("rollenwissen"), include_str!("../assets/regeln/rollenwissen.json")),
    (crate::id("profilseite"), include_str!("../assets/regeln/profilseite.json")),
    (crate::id("kalibrierung"), include_str!("../assets/regeln/kalibrierung.json")),
    (crate::id("merkmale"), include_str!("../assets/regeln/merkmale.json")),
    (crate::id("texte"), include_str!("../assets/texte/mensch.json")),
];

fn alle() -> &'static Vec<(&'static str, serde_json::Value)> {
    static R: OnceLock<Vec<(&'static str, serde_json::Value)>> = OnceLock::new();
    R.get_or_init(|| {
        let mut v: Vec<(&str, serde_json::Value)> = DATEIEN.iter().map(|(n, t)| {
            let tv: toml::Value = toml::from_str(t).unwrap_or_else(|e| panic!("{n}.toml: {e}")); // roh
            (*n, serde_json::to_value(tv).unwrap())
        }).collect();
        v.extend(JSON.iter().map(|(n, t)| (*n, serde_json::from_str(t).unwrap_or_else(|e| panic!("{n}.json: {e}"))))); // roh
        // Paket 6: a copy in the data folder (never the game folder) overrides the built-in values; broken copies are ignored and reported
        if !cfg!(test) {
            for (n, w) in v.iter_mut() {
                let Some((pfad, toml_art)) = eigene_datei(n) else { continue };
                let Ok(text) = std::fs::read_to_string(&pfad) else { continue };
                let neu: Result<serde_json::Value, String> = if toml_art { toml::from_str::<toml::Value>(&text).map_err(|e| e.to_string()).map(|t| serde_json::to_value(t).unwrap()) } else { serde_json::from_str(&text).map_err(|e| e.to_string()) };
                match neu { Ok(x) => mischen(w, x), Err(e) => FEHLER.lock().unwrap().push(format!("{}: {e}", pfad.display())) }
            }
        }
        v
    })
}

static FEHLER: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
fn mischen(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) { (serde_json::Value::Object(x), serde_json::Value::Object(y)) => { for (k, w) in y { mischen(x.entry(k).or_insert(serde_json::Value::Null), w); } } (a, b) => *a = b }
}
/// The folder for the person's own rule files (next to the careers, never in the game folder).
pub fn eigene_regeln() -> std::path::PathBuf { crate::game::save_dir().join(crate::id("../regeln")) }
fn quelle(n: &str) -> Option<(&'static str, bool)> {
    DATEIEN.iter().find(|x| x.0 == n).map(|x| (x.1, true)).or_else(|| JSON.iter().find(|x| x.0 == n).map(|x| (x.1, false)))
}
fn eigene_datei(n: &str) -> Option<(std::path::PathBuf, bool)> { let (_, t) = quelle(n)?; let p = eigene_regeln().join(format!("{n}.{}", if t { crate::id("toml") } else { crate::id("json") })); p.exists().then_some((p, t)) }
/// Every rule file with whether the person has an own copy.
pub fn dateien() -> Vec<(String, bool)> { DATEIEN.iter().chain(JSON.iter()).filter(|x| x.0 != crate::id("texte")).map(|x| (x.0.to_string(), eigene_datei(x.0).is_some())).collect() }
/// Copy a built-in rule file into the own folder to edit it (works from the next start).
pub fn kopie_anlegen(n: &str) -> Result<std::path::PathBuf, String> {
    let (text, t) = quelle(n).ok_or(crate::inhalt::t("ui.regeln.unbekannte_regeldatei"))?;
    let d = eigene_regeln();
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let p = d.join(format!("{n}.{}", if t { crate::id("toml") } else { crate::id("json") }));
    if p.exists() { return Err(crate::inhalt::t("ui.regeln.die_kopie_gibt_es_schon").into()); }
    std::fs::write(&p, text).map_err(|e| e.to_string())?;
    Ok(p)
}
pub fn fehler() -> Vec<String> { FEHLER.lock().map(|x| x.clone()).unwrap_or_default() }
/// The value at `datei` / `a.b.c` (array items by number).
pub fn wert(datei: &str, pfad: &str) -> &'static serde_json::Value {
    let mut v = &alle().iter().find(|x| x.0 == datei).unwrap_or_else(|| panic!("Regeldatei {datei} fehlt")).1; // roh
    for teil in pfad.split('.') {
        v = match v { serde_json::Value::Array(a) => &a[teil.parse::<usize>().unwrap()], _ => &v[teil] };
    }
    if v.is_null() { panic!("Regelwert {datei}:{pfad} fehlt"); } // roh
    v
}
/// A value of assets/texte/mensch.json in the chosen language (crate::inhalt::eigen).
pub fn text_wert(pfad: &str) -> &'static serde_json::Value { pfad.split('.').fold(crate::inhalt::eigen("mensch"), |v, p| match v { serde_json::Value::Array(a) => &a[p.parse::<usize>().unwrap_or(0)], _ => &v[p] }) }
/// A text from assets/texte/mensch.json.
pub fn t(pfad: &str) -> &'static str { pfad.split('.').fold(crate::inhalt::eigen("mensch"), |v, p| match v { serde_json::Value::Array(a) => &a[p.parse::<usize>().unwrap_or(0)], _ => &v[p] }).as_str().unwrap_or_else(|| panic!("Text {pfad} fehlt")) } // roh
/// A number from the rules.
pub fn f(datei: &str, pfad: &str) -> f32 { wert(datei, pfad).as_f64().unwrap_or_else(|| panic!("{datei}:{pfad} ist keine Zahl")) as f32 } // roh
/// A value that may be missing.
pub fn wert_oder(datei: &str, pfad: &str) -> Option<&'static serde_json::Value> {
    let mut v = &alle().iter().find(|x| x.0 == datei)?.1;
    for teil in pfad.split('.') { v = &v[teil]; }
    (!v.is_null()).then_some(v)
}
/// A number that may be missing (then `ersatz`).
pub fn f_oder(datei: &str, pfad: &str, ersatz: f32) -> f32 {
    let mut v = &alle().iter().find(|x| x.0 == datei).unwrap().1;
    for teil in pfad.split('.') { v = &v[teil]; }
    v.as_f64().map(|x| x as f32).unwrap_or(ersatz)
}

#[cfg(test)]
mod tests {
    #[test]
    fn alle_zahlen_da() {
        assert_eq!(super::f("entwicklung", "moral.rate"), 0.12);
        assert_eq!(super::f("klima", "je_takt"), 8.0);
        assert_eq!(super::f("vereinswelt", "praesidium.ultimatum_tage"), 60.0);
        assert_eq!(super::f("persoenlichkeit", "zufall.grenze"), 12.0);
        assert_eq!(super::f("laufbahn", "gehalt.basis"), 250000.0);
        let mut a = serde_json::json!({"x": {"y": 1, "z": 2}});
        super::mischen(&mut a, serde_json::json!({"x": {"y": 5}}));
        assert_eq!(a, serde_json::json!({"x": {"y": 5, "z": 2}}), "an own copy overrides only what it names");
        // the handbook describes this build, not the old flow of the Python version
        let hb = include_str!("../assets/texte/handbuch_texte.json");
        for alt in ["einlesen", "Einlesen", "vorgemerkt", "Lehrg", "Fitspritze", "Sandbox", "Preset", "Meister-Liga", "Spielstand einl"] { assert!(!hb.contains(alt), "Handbuch erwähnt noch „{alt}“"); }
        let h: serde_json::Value = serde_json::from_str(include_str!("../assets/texte/handbuch.json")).unwrap();
        let t: serde_json::Value = serde_json::from_str(hb).unwrap();
        for k in h["kapitel"].as_array().unwrap() { let id = k["id"].as_str().unwrap(); assert!(t[format!("handbuch.{id}.titel")].is_string(), "Kapitel {id} ohne Text"); }
    }
}
