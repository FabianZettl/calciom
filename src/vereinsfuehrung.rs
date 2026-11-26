//! Real club leadership and staff (assets/vereinsfuehrung.toml, collected from Wikipedia 23.11., 153 clubs). Names with
//! `sicher = false` are old or disputed: they are only used when no confirmed name for the role exists. Nothing here
//! is stored in the save, so running careers get the names on the next start too.
use std::collections::HashMap;
use std::sync::OnceLock;

/// (role, name, sure) per club, keyed by a normalised club name.
fn tabelle() -> &'static HashMap<String, Vec<(String, String, bool)>> {
    static T: OnceLock<HashMap<String, Vec<(String, String, bool)>>> = OnceLock::new();
    T.get_or_init(|| {
        let v: toml::Value = toml::from_str(include_str!("../assets/vereinsfuehrung.toml")).unwrap_or(toml::Value::Integer(0));
        let mut m = HashMap::new();
        for (name, c) in v.get("clubs").and_then(|x| x.as_table()).into_iter().flatten() {
            let leute: Vec<(String, String, bool)> = c.get("people").and_then(|x| x.as_array()).into_iter().flatten().filter_map(|p| {
                Some((p.get("role")?.as_str()?.to_string(), p.get("name")?.as_str()?.to_string(), p.get("sicher").and_then(|x| x.as_bool()).unwrap_or(false)))
            }).collect();
            for n in [Some(name.as_str()), c.get("name_local").and_then(|x| x.as_str())].into_iter().flatten() { m.entry(norm(n)).or_insert_with(|| leute.clone()); }
        }
        m
    })
}

/// Lower case, no accents, no club-form words (FC, CF, AFC …), German and English city names alike.
pub fn norm(s: &str) -> String {
    let s: String = s.to_lowercase().chars().map(|c| match c {
        'ä' | 'á' | 'à' | 'â' | 'ã' => 'a', 'ö' | 'ó' | 'ò' | 'ô' | 'õ' | 'ø' => 'o', 'ü' | 'ú' | 'ù' | 'û' => 'u', 'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'ı' => 'i', 'ç' => 'c', 'ñ' => 'n', 'ş' => 's', 'ğ' => 'g', '.' | '-' | '\'' | '&' => ' ', c => c }).collect();
    let s = s.replace(crate::id("munchen"), crate::id("munich")).replace(crate::id("koln"), crate::id("cologne")).replace(crate::id("ß"), crate::id("ss"));
    let weg = ["fc", "cf", "afc", "f c", "sc", "ac", "sv", "tsg", "vfl", "vfb", "ssc", "as", "ss", "cd", "rc", "rcd", "sd", "ud", "ca", "sl", "fk", "sk", "club", "de", "1", "04", "05", "09", "1899", "1846", "1848", "1860", "1907", "1910", "1913", "and", "hove"]; // roh
    s.split_whitespace().filter(|w| !weg.contains(w)).collect::<Vec<_>>().join(" ")
}

/// The person of a club in one of the roles (first match wins), confirmed names before disputed ones.
pub fn person(club: &str, rollen: &[&str]) -> Option<String> {
    let leute = tabelle().get(&norm(club))?;
    for sicher in [true, false] {
        for r in rollen { if let Some(p) = leute.iter().find(|p| p.0 == *r && p.2 == sicher) { return Some(p.1.clone()); } }
    }
    None
}
/// All people of a club in a role, confirmed first.
pub fn personen(club: &str, rolle: &str) -> Vec<String> {
    let Some(leute) = tabelle().get(&norm(club)) else { return vec![] };
    let mut v: Vec<&(String, String, bool)> = leute.iter().filter(|p| p.0 == rolle).collect();
    v.sort_by_key(|p| !p.2);
    v.into_iter().map(|p| p.1.clone()).collect()
}
/// Inbox sender role (posteingang.toml [von]) to the roles of the list.
pub fn amt(rolle: &str) -> &'static [&'static str] {
    match rolle {
        "praesident" => &["president", "chairman"], // roh
        "sportdirektor" => &["sporting_director"], // roh
        "finanzen" => &["finance_director", "ceo"], // roh
        "arzt" => &["doctor"], // roh
        "jugendtrainer" => &["academy_head", "youth_coach"], // roh
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    /// Which listed clubs find no club of a real world: CALCIOM_WELT=<world> cargo test --release abgleich -- --ignored --nocapture
    #[test]
    #[ignore]
    fn abgleich_mit_welt() {
        let Ok(d) = std::env::var("CALCIOM_WELT") else { return };
        let w = crate::world::World::from_pesdb(std::path::Path::new(&d), 1).unwrap();
        let welt: std::collections::HashSet<String> = w.clubs.iter().map(|c| super::norm(&c.name)).collect();
        let fehlt: Vec<&String> = super::tabelle().keys().filter(|k| !welt.contains(*k)).collect();
        let treffer = w.clubs.iter().filter(|c| super::tabelle().contains_key(&super::norm(&c.name))).count();
        eprintln!("{treffer} Vereine der Welt mit Namen; Schlüssel ohne Verein: {fehlt:?}");
    }
    #[test]
    fn namen_gefunden() {
        assert!(super::tabelle().len() >= 150);
        assert_eq!(super::norm("Bayern München"), super::norm("Bayern Munich"));
        assert!(super::person("Bayern München", &["president"]).is_some());
        assert!(super::person("Arsenal", &["president", "chairman"]).is_some());
        assert!(super::person("Gibt es nicht", &["president"]).is_none()); // roh
    }
}
