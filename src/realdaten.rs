//! Real club and player data up to summer 2026 (pesm/fl/sim/realdaten.py; data config/realdaten/vereine.json and
//! spieler.json, copied unchanged). Read only, never copied into a save: founding year, town, stadium, record players,
//! league titles, legends, rivals and titles of 145 clubs; career stations and titles of 305 players.
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Default)]
pub struct Verein {
    pub gruendung: Option<u32>,
    pub ort: String,
    pub stadion: String,
    /// (name, games) and (name, goals).
    pub rekordspieler: Option<(String, u32)>,
    pub rekordtorschuetze: Option<(String, u32)>,
    pub meister: u32,
    /// (name, player id in the database if known).
    pub legenden: Vec<(String, Option<u32>)>,
    /// (club, name of the duel).
    pub rivalen: Vec<(String, String)>,
    pub europa_frueher: String,
    /// Kind of title ("meister", "pokal", "cl" …) → years (season end).
    pub titel: Vec<(String, Vec<u32>)>,
}

pub struct Station { pub verein: String, pub von: u32, pub bis: Option<u32>, pub spiele: Option<u32>, pub tore: Option<u32> }
pub struct Spieler { pub stationen: Vec<Station>, pub titel: Vec<(String, Vec<u32>)> }

pub struct Daten { pub stand: String, vereine: HashMap<String, Verein>, norm: Vec<(String, String)>, spieler: HashMap<u32, Spieler> }

pub fn daten() -> &'static Daten {
    static D: OnceLock<Daten> = OnceLock::new();
    D.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/inhalt/pesm/fl/sim/config/realdaten/vereine.json")).expect("vereine.json");
        let s = |x: &serde_json::Value| x.as_str().unwrap_or("").to_string();
        let paar = |x: &serde_json::Value| x.as_array().and_then(|a| Some((a.first()?.as_str()?.to_string(), a.get(1)?.as_u64()? as u32)));
        let titel = |x: &serde_json::Value| -> Vec<(String, Vec<u32>)> { x.as_object().map(|m| m.iter().map(|(k, v)| (k.clone(), v.as_array().map(|a| a.iter().filter_map(|y| y.as_u64().map(|y| y as u32)).collect()).unwrap_or_default())).collect()).unwrap_or_default() };
        let mut vereine = HashMap::new();
        for (name, d) in v["vereine"].as_object().into_iter().flatten() {
            vereine.insert(name.clone(), Verein {
                gruendung: d["g"].as_u64().map(|x| x as u32), ort: s(&d["ort"]), stadion: s(&d["st"]),
                rekordspieler: paar(&d["rs"]), rekordtorschuetze: paar(&d["rt"]), meister: d["lg"].as_u64().unwrap_or(0) as u32,
                legenden: d["leg"].as_array().map(|a| a.iter().map(|l| (s(&l["name"]), l["pid"].as_u64().map(|x| x as u32))).collect()).unwrap_or_default(),
                rivalen: d["riv"].as_array().map(|a| a.iter().filter_map(|r| Some((r.get(0)?.as_str()?.to_string(), r.get(1)?.as_str()?.to_string()))).collect()).unwrap_or_default(),
                europa_frueher: s(&d["euro_hist"]), titel: titel(&d["titel"]),
            });
        }
        let norm = vereine.keys().map(|n| (crate::trainerbank::norm(n), n.clone())).collect();
        let sp: serde_json::Value = serde_json::from_str(include_str!("../assets/inhalt/pesm/fl/sim/config/realdaten/spieler.json")).expect("spieler.json");
        let spieler = sp["spieler"].as_object().into_iter().flatten().filter_map(|(k, e)| Some((k.parse().ok()?, Spieler {
            stationen: e["stationen"].as_array().map(|a| a.iter().map(|x| Station {
                verein: { let w = s(&x["w"]); if w.is_empty() { s(&x["v"]) } else { w } },
                von: x["von"].as_u64().unwrap_or(0) as u32, bis: x["bis"].as_u64().map(|y| y as u32),
                spiele: x["spiele"].as_u64().map(|y| y as u32), tore: x["tore"].as_u64().map(|y| y as u32),
            }).collect()).unwrap_or_default(),
            titel: titel(&e["titel"]),
        }))).collect();
        Daten { stand: s(&v["stand"]), vereine, norm, spieler }
    })
}

/// The real data of a club by its name in the world: exact, otherwise the same normalized name (FC, years and
/// accents ignored, like the coach bank).
pub fn verein(name: &str) -> Option<&'static Verein> {
    let d = daten();
    if let Some(v) = d.vereine.get(name) { return Some(v); }
    let n = crate::trainerbank::norm(name);
    d.norm.iter().find(|(k, _)| *k == n).and_then(|(_, orig)| d.vereine.get(orig))
}

/// The real career of a player (database id) or None.
pub fn spieler(id: u32) -> Option<&'static Spieler> { daten().spieler.get(&id) }

/// The name of the duel between two clubs from the real data (both directions), or None.
pub fn derby(a: &str, b: &str) -> Option<&'static str> {
    let passt = |x: &str, y: &str| x == y || crate::trainerbank::norm(x) == crate::trainerbank::norm(y);
    for (eins, zwei) in [(a, b), (b, a)] {
        if let Some(v) = verein(eins) {
            if let Some((_, label)) = v.rivalen.iter().find(|(c, _)| passt(c, zwei)) { return Some(label.as_str()); }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vereine_und_spieler_aus_den_echtdaten() {
        let mu = verein("Manchester United").unwrap();
        assert_eq!((mu.gruendung, mu.stadion.as_str(), mu.meister), (Some(1878), "Old Trafford", 20));
        assert_eq!(mu.rekordspieler.as_ref().unwrap(), &("Bobby Charlton".to_string(), 758));
        // tolerant name: the world may write "FC Bayern München"
        assert!(verein("FC Bayern München").is_some());
        assert_eq!(derby("Liverpool FC", "Manchester United"), Some("Nordwest-Derby"));
        assert_eq!(derby("Sunderland AFC", "Newcastle United"), Some("Tyne-Wear-Derby"));
        assert!(derby("Sunderland AFC", "Arsenal FC").is_none());
        let messi = spieler(7511).unwrap();
        assert_eq!(messi.stationen[0].verein, "FC Barcelona");
        assert!(messi.titel.iter().any(|(k, j)| k == "Champions League" && j.len() == 4));
    }
}
