//! The content of the Python CalcioM, copied unchanged into assets/inhalt/ (same folder layout, list in QUELLEN.txt):
//! text blocks de and en, the content pool (themes, catalogue, occasions), world texts, scenarios, story and config.
//! Read at runtime from the app folder; nothing is rewritten or shortened.
use std::path::{Path, PathBuf};

/// assets/inhalt next to the program (or in the project folder when started from there).
pub fn ordner() -> PathBuf {
    let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default();
    [PathBuf::from("assets/inhalt"), exe.join("../../assets/inhalt"), PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/inhalt"))]
        .into_iter().find(|p| p.is_dir()).unwrap_or_else(|| PathBuf::from("assets/inhalt"))
}
/// A JSON or TOML file as a JSON value.
pub fn lesen(p: &Path) -> Result<serde_json::Value, String> {
    let t = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
    if p.extension().map_or(false, |e| e == "toml") {
        toml::from_str::<toml::Value>(&t).map_err(|e| format!("{}: {e}", p.display())).map(|v| serde_json::to_value(v).unwrap())
    } else {
        serde_json::from_str(&t).map_err(|e| format!("{}: {e}", p.display()))
    }
}
/// Entries of a file: every value that is no object or list (texts, numbers, switches).
pub fn eintraege(v: &serde_json::Value) -> usize {
    match v { serde_json::Value::Object(o) => o.values().map(eintraege).sum(), serde_json::Value::Array(a) => a.iter().map(eintraege).sum(), _ => 1 }
}
/// The copied files (relative paths, as in QUELLEN.txt).
pub fn quellen() -> Vec<String> {
    std::fs::read_to_string(ordner().join("QUELLEN.txt")).unwrap_or_default().lines().skip(1).filter(|z| !z.trim().is_empty()).map(String::from).collect()
}

// ---------------------------------------------------------------- language (Einstellungen → Sprache)
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
use std::sync::OnceLock;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sprache { De, En, Es, Fr }
/// The order of the choice in Einstellungen and of the text columns: German, English, Spanish, French (Fabian 26.11.).
pub const SPRACHEN: [Sprache; 4] = [Sprache::De, Sprache::En, Sprache::Es, Sprache::Fr];
impl Sprache {
    pub fn code(self) -> &'static str { ["de", "en", "es", "fr"][self as usize] } // roh
}
static SPRACHE: AtomicU8 = AtomicU8::new(0);
pub fn sprache() -> Sprache { SPRACHEN[SPRACHE.load(Relaxed) as usize % 4] }
pub fn sprache_setzen(s: Sprache) { SPRACHE.store(s as u8, Relaxed) }
/// "de", "es", "fr" → that language, anything else → English (Fabian 23.11.: English first, also the default without a
/// choice; stored in sprache.txt and in CALCIOM_SPRACHE).
pub fn sprache_aus(s: &str) -> Sprache { SPRACHEN.into_iter().find(|x| x.code() == s.trim()).unwrap_or(Sprache::En) }

type Texte = HashMap<String, String>;
/// Texts of a file as dotted key paths (list entries by index); only texts, no numbers.
fn flach(v: &serde_json::Value, pfad: &str, out: &mut Texte) {
    let unter = |k: &str| if pfad.is_empty() { k.to_string() } else { format!("{pfad}.{k}") };
    match v {
        serde_json::Value::Object(o) => for (k, w) in o { flach(w, &unter(k), out) },
        serde_json::Value::Array(a) => for (i, w) in a.iter().enumerate() { flach(w, &unter(&i.to_string()), out) },
        serde_json::Value::String(s) => { out.insert(pfad.to_string(), s.clone()); }
        _ => {}
    }
}
/// Every file of pesm/core/texts: name (without .json) → (German, English). The English side of de gets the keys
/// of texts/en/nachtrag.json that the original English file lacks.
fn alle() -> &'static BTreeMap<String, (Texte, Texte, Texte, Texte)> {
    static T: OnceLock<BTreeMap<String, (Texte, Texte, Texte, Texte)>> = OnceLock::new();
    T.get_or_init(|| {
        let dir = ordner().join("pesm/core/texts");
        let mut m = BTreeMap::new();
        for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.extension().map_or(true, |x| x != "json") { continue; }
            let (mut de, mut en, mut es, mut fr) = (Texte::new(), Texte::new(), Texte::new(), Texte::new());
            if let Ok(v) = lesen(&p) { flach(&v, "", &mut de); }
            // Spanish and French (Fabian 26.11.) sit next to English: texts/es, texts/fr; missing files fall back to English
            for (sp, t) in [("en", &mut en), ("es", &mut es), ("fr", &mut fr)] { if let Ok(v) = lesen(&dir.join(sp).join(p.file_name().unwrap())) { flach(&v, "", t); } } // roh
            m.insert(p.file_stem().unwrap().to_string_lossy().to_string(), (de, en, es, fr));
        }
        if let (Some((_, en, _, _)), Ok(v)) = (m.get_mut("de"), lesen(&ordner().join("texts/en/nachtrag.json"))) {
            let mut n = Texte::new();
            flach(&v, "", &mut n);
            for (k, t) in n { en.entry(k).or_insert(t); }
        }
        m
    })
}
/// A text of pesm/core/texts/<datei>.json by its key (dotted path, list entries by index) in that language.
/// Missing or empty in English: the German text. Missing in both: "" – nothing is shown, never the key.
pub fn text(sprache: Sprache, datei: &str, schluessel: &str) -> String {
    let Some((de, en, es, fr)) = alle().get(datei.trim_end_matches(".json")) else { return String::new() };
    let eigen = match sprache { Sprache::Es => es.get(schluessel), Sprache::Fr => fr.get(schluessel), _ => None };
    eigen.filter(|t| !t.trim().is_empty()).cloned().unwrap_or_else(|| waehlen(sprache, de.get(schluessel), en.get(schluessel)))
}
fn waehlen(sprache: Sprache, de: Option<&String>, en: Option<&String>) -> String {
    let gut = |t: Option<&String>| t.filter(|t| !t.trim().is_empty()).cloned();
    (if sprache != Sprache::De { gut(en) } else { None }).or_else(|| gut(de)).unwrap_or_default()
}
/// Every text by key in both languages: the code's texts (assets/texte/ui.de.json, ui.en.json) and the original's
/// (pesm/core/texts, de.json first, then the files by name; English from texts/en and the nachtrag). Built once.
fn schluessel_tabelle() -> &'static HashMap<String, [&'static str; 4]> {
    static T: OnceLock<HashMap<String, [&'static str; 4]>> = OnceLock::new();
    T.get_or_init(|| {
        let leak = |s: &String| -> &'static str { Box::leak(s.clone().into_boxed_str()) };
        let mut m: HashMap<String, [&'static str; 4]> = HashMap::new();
        let a = alle();
        for (_, (de, en, es, fr)) in a.get("de").into_iter().chain(a.iter().filter(|(n, _)| n.as_str() != "de").map(|(_, x)| x)).map(|x| ((), x)) {
            let z = |t: &Texte, k: &String| t.get(k).map(leak).unwrap_or("");
            for (k, d) in de { m.entry(k.clone()).or_insert_with(|| [leak(d), z(en, k), z(es, k), z(fr, k)]); }
        }
        let (mut de, mut en, mut es, mut fr) = (Texte::new(), Texte::new(), Texte::new(), Texte::new());
        flach(&serde_json::from_str(include_str!("../assets/texte/ui.de.json")).expect("ui.de.json"), "", &mut de);
        flach(&serde_json::from_str(include_str!("../assets/texte/ui.en.json")).expect("ui.en.json"), "", &mut en);
        flach(&serde_json::from_str(include_str!("../assets/texte/ui.es.json")).expect("ui.es.json"), "", &mut es);
        flach(&serde_json::from_str(include_str!("../assets/texte/ui.fr.json")).expect("ui.fr.json"), "", &mut fr);
        for (k, d) in &de { m.insert(k.clone(), [leak(d), en.get(k).map(leak).unwrap_or(""), es.get(k).map(leak).unwrap_or(""), fr.get(k).map(leak).unwrap_or("")]); }
        m
    })
}
/// The text of a key in the chosen language; English missing or empty → German; unknown key → "" (never the key).
pub fn t(key: &str) -> &'static str { t_in(sprache(), key) }
/// How many variants `basis.0`, `basis.1` … exist (`basis` itself counts as variant 0 when there is no `.0`).
pub fn varianten(basis: &str) -> usize {
    let start = if schluessel_tabelle().contains_key(&format!("{basis}.0")) { 0 } else { 1 };
    let mut n = start;
    while schluessel_tabelle().contains_key(&format!("{basis}.{n}")) { n += 1; }
    n.max(1)
}
/// The key of variant `i`: `basis.i`, or `basis` itself for variant 0 when the texts start at `.1`.
pub fn variante_key(basis: &str, i: usize) -> String { if i == 0 && !schluessel_tabelle().contains_key(&format!("{basis}.0")) { basis.to_string() } else { format!("{basis}.{i}") } }
/// The text of a key in a given language (same rules as `t`).
pub fn t_in(sprache: Sprache, key: &str) -> &'static str {
    let Some(z) = schluessel_tabelle().get(key) else { return "" };
    // the chosen language, else English (Spanish and French are translated from it), else German
    [sprache as usize, 1, 0].into_iter().map(|i| z[i]).find(|x| !x.trim().is_empty()).unwrap_or(z[0])
}
/// A text with placeholders %1, %2 … filled in (from the highest number down, so %1 never eats %10).
pub fn tf(key: &str, werte: &[String]) -> String { tf_in(sprache(), key, werte) }
pub fn tf_in(sprache: Sprache, key: &str, werte: &[String]) -> String {
    let mut s = t_in(sprache, key).to_string();
    for (i, w) in werte.iter().enumerate().rev() { s = s.replace(&format!("%{}", i + 1), w); }
    s
}
/// One of the code's own text files (assets/texte/<name>.json and <name>.en.json) in the chosen language; in English
/// the German values fill every gap, so a missing English text shows the German one.
pub fn eigen(name: &str) -> &'static serde_json::Value {
    static T: OnceLock<HashMap<&'static str, [serde_json::Value; 4]>> = OnceLock::new();
    let m = T.get_or_init(|| {
        fn mischen(de: &serde_json::Value, en: &serde_json::Value) -> serde_json::Value {
            match (de, en) {
                (serde_json::Value::Object(d), serde_json::Value::Object(e)) => serde_json::Value::Object(d.iter().map(|(k, v)| (k.clone(), e.get(k).map_or(v.clone(), |w| mischen(v, w)))).collect()),
                (serde_json::Value::Array(d), serde_json::Value::Array(e)) => serde_json::Value::Array(d.iter().enumerate().map(|(i, v)| e.get(i).map_or(v.clone(), |w| mischen(v, w))).collect()),
                (serde_json::Value::String(d), serde_json::Value::String(e)) => serde_json::Value::String(if e.trim().is_empty() { d.clone() } else { e.clone() }),
                (d, _) => d.clone(),
            }
        }
        // Spanish and French over English over German, so every gap shows the next language
        let paar = |de: &str, en: &str, es: &str, fr: &str| {
            let d: serde_json::Value = serde_json::from_str(de).expect("Textdatei");
            let lies = |t: &str| serde_json::from_str(t).unwrap_or(serde_json::Value::Null);
            let e = mischen(&d, &lies(en));
            let (s, f) = (mischen(&e, &lies(es)), mischen(&e, &lies(fr)));
            [d, e, s, f]
        };
        HashMap::from([
            ("mensch", paar(include_str!("../assets/texte/mensch.json"), include_str!("../assets/texte/mensch.en.json"), include_str!("../assets/texte/mensch.es.json"), include_str!("../assets/texte/mensch.fr.json"))),
            ("presse", paar(include_str!("../assets/texte/presse.json"), include_str!("../assets/texte/presse.en.json"), include_str!("../assets/texte/presse.es.json"), include_str!("../assets/texte/presse.fr.json"))),
            ("spielerprofil", paar(include_str!("../assets/texte/spielerprofil.json"), include_str!("../assets/texte/spielerprofil.en.json"), include_str!("../assets/texte/spielerprofil.es.json"), include_str!("../assets/texte/spielerprofil.fr.json"))),
            ("farm", paar(include_str!("../assets/texte/farm.json"), include_str!("../assets/texte/farm.en.json"), include_str!("../assets/texte/farm.es.json"), include_str!("../assets/texte/farm.fr.json"))),
            ("handbuch", paar(include_str!("../assets/texte/handbuch.json"), include_str!("../assets/texte/handbuch.en.json"), include_str!("../assets/texte/handbuch.es.json"), include_str!("../assets/texte/handbuch.fr.json"))),
            ("handbuch_texte", paar(include_str!("../assets/texte/handbuch_texte.json"), include_str!("../assets/texte/handbuch_texte.en.json"), include_str!("../assets/texte/handbuch_texte.es.json"), include_str!("../assets/texte/handbuch_texte.fr.json"))),
        ])
    });
    let p = m.get(name).unwrap_or_else(|| panic!("Textdatei {name} fehlt")); // roh
    &p[sprache() as usize]
}
/// A data value that is also shown (the sender of a message, the subject of a reason, a placeholder manager): its text
/// from the explicit list `ui.anzeige.<value>`; any other value (a name) is shown as it is.
pub fn anzeige(wert: &str) -> &str {
    let k = format!("ui.anzeige.{wert}");
    match schluessel_tabelle().get(&k) { Some(_) => t(&k), None => wert }
}
/// `tf!("key", a, b)`: the text of `key` with %1 = a, %2 = b (anything that implements Display).
#[macro_export]
macro_rules! tf { ($k:expr $(, $a:expr)* $(,)?) => { $crate::inhalt::tf($k, &[$(($a).to_string()),*]) } }

#[cfg(test)]
mod tests {
    use super::*;
    const PYTHON: &str = "../pesm/pesm-ml-raus";

    /// Counting test: every source file of the Python version is here with at least as many entries.
    #[test]
    fn zaehltest_python_gegen_rust() {
        let q = quellen();
        assert!(q.len() >= 545, "{} Dateien in QUELLEN.txt", q.len());
        let (mut py, mut rs, mut fehlt) = (0usize, 0usize, vec![]);
        for rel in &q {
            // real data moved into the pack (src/pakete.rs) counts from there; the program holds a neutral stand-in
            let im_paket = rel.starts_with("pesm/core/config/").then(|| crate::pakete::echt_ordner().map(|d| d.join(Path::new(rel).file_name().unwrap()))).flatten().filter(|p| p.is_file());
            let hier = lesen(&im_paket.unwrap_or_else(|| ordner().join(rel))).unwrap_or_else(|e| panic!("{e}"));
            rs += eintraege(&hier);
            let quelle = Path::new(PYTHON).join(rel);
            if !quelle.exists() { continue; } // another computer: only the copy is checked
            let n = eintraege(&lesen(&quelle).unwrap());
            py += n;
            if eintraege(&hier) < n { fehlt.push(format!("{rel}: Python {n}, Rust {}", eintraege(&hier))); }
        }
        assert!(fehlt.is_empty(), "in Rust fehlt etwas:\n{}", fehlt.join("\n"));
        eprintln!("Zähltest: {} Dateien, Python {py} Einträge, Rust {rs} Einträge", q.len());
    }
    /// The English pool texts have the keys of the German ones, and no pool text is empty.
    #[test]
    fn texte_deutsch_und_englisch() {
        let de = ordner().join("pesm/core/texts");
        let mut befund = vec![];
        fn schluessel(v: &serde_json::Value, pfad: &str, out: &mut Vec<String>, leer: &mut Vec<String>) {
            match v {
                serde_json::Value::Object(o) => for (k, w) in o { schluessel(w, &format!("{pfad}.{k}"), out, leer) },
                serde_json::Value::String(s) => { out.push(pfad.to_string()); if s.trim().is_empty() { leer.push(pfad.to_string()); } }
                _ => out.push(pfad.to_string()),
            }
        }
        for e in std::fs::read_dir(&de).unwrap().flatten() {
            let p = e.path();
            if p.extension().map_or(true, |x| x != "json") { continue; }
            let en = de.join("en").join(p.file_name().unwrap());
            if !en.exists() { continue; }
            let (mut kd, mut ke, mut leer) = (vec![], vec![], vec![]);
            schluessel(&lesen(&p).unwrap(), "", &mut kd, &mut leer);
            schluessel(&lesen(&en).unwrap(), "", &mut ke, &mut leer);
            kd.sort(); ke.sort();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            let stamm = name.trim_end_matches(".json");
            // keys the original English lacks: texts/en/nachtrag.json gives them English (de.json: all 44 of them)
            let alle_fehlenden = kd.iter().filter(|k| ke.binary_search(k).is_err()).count();
            let nur_de: Vec<&String> = kd.iter().filter(|k| ke.binary_search(k).is_err())
                .filter(|k| { let k = &k[1..]; text(Sprache::En, stamm, k) == text(Sprache::De, stamm, k) }).collect();
            if stamm == "de" {
                assert_eq!(alle_fehlenden, 44, "de.json: Schlüssel ohne Englisch im Original");
                assert!(nur_de.is_empty(), "de.json: ohne Englisch auch nach nachtrag.json: {nur_de:?}");
                assert_eq!(text(Sprache::En, "de", "kachel.in_tagen.eins"), "in %1 day");
                assert_eq!(text(Sprache::De, "de", "kachel.in_tagen.eins"), "in %1 Tag");
            }
            // like pesm/core/inhalt/pruefung.py: strict for the content pool; elsewhere the original has gaps, listed only
            let mut b = vec![];
            if !nur_de.is_empty() { b.push(format!("{name}: {} Schlüssel ohne Englisch, z. B. {}", nur_de.len(), nur_de[0])); }
            if !leer.is_empty() { b.push(format!("{name}: leerer Text bei {}", leer[0])); }
            if name.starts_with("pool_") { befund.extend(b); } else { for x in b { eprintln!("Hinweis (wie im Original): {x}"); } }
        }
        assert!(befund.is_empty(), "{}", befund.join("\n"));
    }
    /// Fallback: English missing or empty → the German text; missing in both (or an unknown file) → "" (nothing is
    /// drawn, never the key). The UI table keeps unknown texts as they are and German mode changes nothing.
    #[test]
    fn sprache_rueckfall() {
        let (d, e, leer) = ("Zentrale".to_string(), "Hub".to_string(), " ".to_string());
        assert_eq!(waehlen(Sprache::En, Some(&d), Some(&e)), "Hub");
        assert_eq!(waehlen(Sprache::En, Some(&d), Some(&leer)), "Zentrale");
        assert_eq!(waehlen(Sprache::En, None, None), "");
        assert_eq!(text(Sprache::En, "de", "bereich.zentrale"), "Hub");
        assert_eq!(t_in(Sprache::En, "bereich.zentrale"), "Hub", "keys of the original");
        assert_eq!(t_in(Sprache::De, "bereich.zentrale"), "Zentrale");
        assert_eq!(t_in(Sprache::En, "gibt.es.nicht"), "", "unknown key: nothing, never the key");
        // a club called "Training" stays "Training": names are placeholder values and never translated
        let k = schluessel_tabelle().iter().find(|(k, v)| k.starts_with("ui.") && v[0] == "%1").map(|(k, _)| k.clone())
            .or_else(|| schluessel_tabelle().iter().find(|(k, v)| k.starts_with("ui.") && v[0].contains("%1")).map(|(k, _)| k.clone())).unwrap();
        assert!(tf_in(Sprache::En, &k, &["Training".into()]).contains("Training"));
        assert!(!tf_in(Sprache::En, &k, &["Training".into()]).contains("%1"));
    }
    /// ABDECKUNG.md has one row per file of the Python version (tools/abdeckung.py); a new or removed file there
    /// means the list is out of date. Skipped without the Python version on this machine.
    #[test]
    fn abdeckung_vollstaendig() {
        let py = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Projects/pesm/pesm-ml-raus/pesm");
        if !py.is_dir() { return; }
        let liste = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ABDECKUNG.md")).unwrap();
        let mut fehlt = vec![];
        let mut stapel = vec![py.clone()];
        while let Some(d) = stapel.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                let n = e.file_name().to_string_lossy().to_string();
                if p.is_dir() { if n != "__pycache__" && n != "tests" && !(n == "demo" && d.ends_with("ui")) { stapel.push(p); } continue; }
                if !n.ends_with(".py") { continue; }
                let rel = p.strip_prefix(&py).unwrap().to_string_lossy().to_string();
                if !liste.contains(&format!("| `{rel}` |")) { fehlt.push(rel); }
            }
        }
        assert!(fehlt.is_empty(), "ABDECKUNG.md fehlt: {fehlt:?} (python3 -I tools/abdeckung.py)");
    }
    /// 100 % of the fixed texts come from the text files, and every other word in the code is marked as an identifier
    /// (crate::id) or a raw name (`// roh`). One set of rules: tools/kennungen.py (it also marks new identifiers).
    #[test]
    fn sprache_abdeckung() {
        let dir = env!("CARGO_MANIFEST_DIR");
        let Ok(o) = std::process::Command::new("python3").args(["-I", &format!("{dir}/tools/kennungen.py"), "--nur-zaehlen"]).output() else { eprintln!("python3 fehlt: Prüfung übersprungen"); return };
        let aus = String::from_utf8_lossy(&o.stdout).to_string();
        assert!(o.status.success() && aus.trim_end().ends_with("{'id': 0, 'text': 0}"), "feste Texte ohne Schlüssel oder Wörter ohne id():\n{aus}");
    }
    /// No logic on translated texts: no comparison with the result of t(...), no t(t(...)), no value of ui.*.json that is
    /// itself a key.
    #[test]
    fn keine_logik_auf_texten() {
        let mut fehler = vec![];
        for e in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap().flatten() {
            if e.path().ends_with("inhalt.rs") { continue; }
            let t = std::fs::read_to_string(e.path()).unwrap_or_default();
            for (nr, z) in t.split("\n#[cfg(test)]").next().unwrap().lines().enumerate() {
                let z: String = z.split("//").next().unwrap_or("").chars().filter(|c| !c.is_whitespace()).collect();
                let schlecht = ["==crate::inhalt::t(", "!=crate::inhalt::t(", "==&crate::inhalt::t(", "!=&crate::inhalt::t(", "inhalt::t(crate::inhalt::t(", "inhalt::t(t(", ".contains(crate::inhalt::t(", ".contains(&crate::inhalt::t(", "==crate::tf!(", "!=crate::tf!("]
                    .iter().any(|m| z.contains(m))
                    || z.split("crate::inhalt::t(\"").skip(1).any(|r| { let nach = r.splitn(2, "\")").nth(1).unwrap_or(""); nach.starts_with("==") || nach.starts_with("!=") });
                if schlecht { fehler.push(format!("{}:{}", e.path().file_name().unwrap().to_string_lossy(), nr + 1)); }
            }
        }
        for name in ["ui.de.json", "ui.en.json"] {
            let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{}/assets/texte/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()).unwrap();
            for (k, w) in v.as_object().unwrap() {
                let w = w.as_str().unwrap_or("");
                if schluessel_tabelle().contains_key(w) { fehler.push(format!("{name} {k}: Wert ist selbst ein Schlüssel ({w})")); }
            }
        }
        assert!(fehler.is_empty(), "Logik auf übersetzten Texten:\n{}", fehler.join("\n"));
    }
    /// Every key in the code has a German text; ui.de.json and ui.en.json have the same keys, none empty, and the same
    /// placeholders; the own text files have the same structure and placeholders in both languages.
    #[test]
    fn schluessel_und_platzhalter() {
        let platz = |s: &str| { let mut v: Vec<String> = vec![]; let c: Vec<char> = s.chars().collect();
            for i in 0..c.len() { if c[i] == '%' && c.get(i + 1).map_or(false, |x| x.is_ascii_digit()) { v.push(format!("%{}", c[i + 1])); } if c[i] == '{' { if let Some(e) = c[i..].iter().position(|&x| x == '}') { v.push(c[i..i + e + 1].iter().collect()); } } }
            v.sort(); v };
        let (mut de, mut en) = (Texte::new(), Texte::new());
        flach(&serde_json::from_str(include_str!("../assets/texte/ui.de.json")).unwrap(), "", &mut de);
        flach(&serde_json::from_str(include_str!("../assets/texte/ui.en.json")).unwrap(), "", &mut en);
        let mut fehler = vec![];
        for (k, d) in &de {
            match en.get(k) { None => fehler.push(format!("{k}: kein Englisch")), Some(e) if e.trim().is_empty() && !d.trim().is_empty() => fehler.push(format!("{k}: Englisch leer")),
                Some(e) => if platz(d) != platz(e) { fehler.push(format!("{k}: Platzhalter {:?} / {:?}", platz(d), platz(e))) } }
        }
        for k in en.keys() { if !de.contains_key(k) { fehler.push(format!("{k}: nur Englisch")); } }
        for name in ["mensch", "presse", "spielerprofil", "farm", "handbuch", "handbuch_texte"] {
            let d = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(format!("{}/assets/texte/{name}.json", env!("CARGO_MANIFEST_DIR"))).unwrap()).unwrap();
            let e = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(format!("{}/assets/texte/{name}.en.json", env!("CARGO_MANIFEST_DIR"))).unwrap()).unwrap();
            let (mut fd, mut fe) = (Texte::new(), Texte::new());
            flach(&d, "", &mut fd); flach(&e, "", &mut fe);
            for (k, x) in &fd { match fe.get(k) { None => fehler.push(format!("{name}.{k}: kein Englisch")), Some(y) => if platz(x) != platz(y) { fehler.push(format!("{name}.{k}: Platzhalter")) } } }
        }
        // every key the code uses exists
        for e in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap().flatten() {
            if e.path().ends_with("inhalt.rs") { continue; } // its doc comments show tf!("key", …) as an example
            let t = std::fs::read_to_string(e.path()).unwrap_or_default();
            for teil in t.split("inhalt::t(\"").skip(1).chain(t.split("tf!(\"").skip(1)) {
                let k = teil.split('"').next().unwrap();
                if t_in(Sprache::De, k).is_empty() && !de.get(k).map_or(false, |x| x.is_empty()) { fehler.push(format!("Schlüssel ohne Text: {k}")); }
            }
        }
        assert!(fehler.is_empty(), "{} Befunde:\n{}", fehler.len(), fehler.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
    }

}
