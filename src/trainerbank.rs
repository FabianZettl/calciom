//! The coach bank of the original (pesm/core/trainer/bank.py, service.py; data config/scouting/trainerbank.json and
//! trainer_zuordnung.json): real coaches for the clubs of the world, matched by tolerant club names; coaches without a
//! club form the pool. A fired AI coach goes to the pool (after a lock of `sperre_tage`), a vacant post is filled from
//! the pool (fame near the club's size, the same country preferred), otherwise an invented coach comes.
use crate::world::World;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Lowest similarity of two normalized club names (bank.py SCHWELLE).
const SCHWELLE: f32 = 0.84;
/// Days a fired coach waits before he is in the pool again (trainer.toml karussell.sperre_tage).
pub const SPERRE_TAGE: u32 = 28;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Trainer {
    pub name: String,
    pub nat: String,
    /// Country and league of the post in the bank (where he worked last).
    pub land: String,
    pub stil: String,
    pub persona: String,
    pub tags: Vec<String>,
    /// Playing idea and pressing from the bank profile ("ballbesitz", "konter", "pragmatisch"; "hoch", "tief"); "" unknown.
    pub idee: String,
    pub pressing: String,
    /// Fame 1–5.
    pub fame: u8,
    /// In office since ("MM/YYYY" from the bank, or "S<season>" for a hire in the career).
    pub seit: String,
    /// From the bank (true) or invented (false).
    pub echt: bool,
    /// Pool only: free from this absolute day on.
    #[serde(default)]
    pub frei_ab: u32,
    /// His favourite formation and the one he switches to now and then, (code, depth, width) × 11 (Fabian 22.11.); he
    /// takes them along to a new club.
    #[serde(default)]
    pub formation: Vec<(u8, u8, u8)>,
    #[serde(default)]
    pub ausweich: Vec<(u8, u8, u8)>,
}

impl Trainer {
    /// The AI style it brings (-1 defensive, 0 balanced, 1 attacking): pressing high or attacking tags → 1,
    /// counter or defensive tags or deep pressing → -1.
    pub fn stil_wert(&self) -> i8 {
        let tag = |t: &str| self.tags.iter().any(|x| x == t);
        if self.pressing == "hoch" || tag(crate::id("offensiv")) || tag(crate::id("gegenpressing")) { 1 }
        else if self.idee == "konter" || self.pressing == "tief" || tag(crate::id("defensivorientiert")) || tag(crate::id("kompakt")) { -1 }
        else { 0 }
    }
}

struct Bank { trainer: Vec<(String, Trainer, bool)>, manuell: HashMap<String, Option<String>> }

fn bank() -> &'static Bank {
    static B: OnceLock<Bank> = OnceLock::new();
    B.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(&crate::pakete::oder("trainerbank.json", include_str!("../assets/inhalt/pesm/core/config/scouting/trainerbank.json"))).expect("trainerbank.json");
        let s = |x: &serde_json::Value| x.as_str().unwrap_or("").to_string();
        let trainer = v["trainer"].as_array().map(|a| a.iter().map(|e| {
            let t = Trainer {
                name: s(&e["name"]), nat: s(&e["nat"]), land: s(&e["land"]), stil: s(&e["style"]), persona: s(&e["persona"]),
                tags: e["tags_id"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
                idee: s(&e["profil"]["idee"]), pressing: s(&e["profil"]["pressing"]), fame: e["fame"].as_u64().unwrap_or(1).clamp(1, 5) as u8,
                seit: s(&e["since"]), echt: true, frei_ab: 0, formation: vec![], ausweich: vec![],
            };
            (s(&e["club"]), t, e["status"].as_str() == Some(crate::id("gesucht")))
        }).collect()).unwrap_or_default();
        let z: serde_json::Value = serde_json::from_str(&crate::pakete::oder("trainer_zuordnung.json", include_str!("../assets/inhalt/pesm/core/config/scouting/trainer_zuordnung.json"))).expect("trainer_zuordnung.json");
        let manuell = z["zuordnung"].as_object().map(|m| m.iter().map(|(k, v)| (k.clone(), v.as_str().map(String::from))).collect()).unwrap_or_default();
        Bank { trainer, manuell }
    })
}

const ZUSATZ: &[&str] = &[crate::id("fc"), crate::id("sc"), crate::id("sv"), crate::id("ssv"), crate::id("vfl"), crate::id("vfb"), crate::id("dsc"), crate::id("tsg"), crate::id("fk"), crate::id("cf"), crate::id("cd"), crate::id("ud"), crate::id("rc"), crate::id("rcd"), crate::id("ac"), crate::id("as"), crate::id("afc"), crate::id("sk"), crate::id("ssc"), crate::id("us"), crate::id("fsv"), crate::id("kfc"), crate::id("krc"), crate::id("ca"), crate::id("sd"), crate::id("ad"), crate::id("fsc"), crate::id("bk"), crate::id("if"), crate::id("ik"), crate::id("ks"), crate::id("cs"), crate::id("sl"), crate::id("gd"), crate::id("cp"), crate::id("og"), crate::id("rsc"), crate::id("sco"), crate::id("ogc"), crate::id("uc"), crate::id("vv"), crate::id("fv"), crate::id("nec"), crate::id("gnk"), crate::id("de"), crate::id("la"), crate::id("el"), crate::id("kv"), crate::id("ksv"), crate::id("kaa"), crate::id("jk"), crate::id("bc"), crate::id("futbol"), crate::id("calcio"), crate::id("royale"), crate::id("sad"), crate::id("cfc"), crate::id("balompie"), crate::id("le"), "1", "04", "05", "09", "46", "48", "1848", "1860", "1899", "1900", "1903", "1907"];
const RESERVE: [&str; 8] = [crate::id("b"), crate::id("ii"), crate::id("u23"), crate::id("u21"), crate::id("u19"), crate::id("jong"), "2", crate::id("reserves")];

fn falten(c: char) -> &'static str {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a", 'ç' | 'ć' | 'č' => "c", 'ď' | 'đ' => "d", 'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => crate::id("e"),
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => "i", 'ł' | 'ľ' | 'ĺ' => "l", 'ñ' | 'ń' | 'ň' => "n", 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => crate::id("o"),
        'ř' | 'ŕ' => "r", 'ś' | 'š' | 'ş' | 'ș' => "s", 'ß' => "ss", 'ť' | 'ţ' | 'ț' => "t", 'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' => "u", 'ý' | 'ÿ' => "y", 'ź' | 'ż' | 'ž' => crate::id("z"),
        _ => "",
    }
}

/// bank.py norm(): without accents, lower case, club additions (FC, SC …) and years dropped, a few spellings unified.
pub fn norm(name: &str) -> String {
    let mut s = String::new();
    for c in name.to_lowercase().chars() {
        let f = falten(c);
        if !f.is_empty() { s.push_str(f); } else if c.is_ascii_alphanumeric() || c == ' ' { s.push(c); } else if c == '&' || c == '-' || !c.is_ascii() || c.is_ascii_punctuation() { s.push(' '); }
    }
    let ersatz = |t: &str| -> String { match t { "muenchen" | "munchen" => "munchen", "internazionale" => "inter", "milan" => "mailand", "roma" => "rom", "athens" => "athen", "wolves" => "wolverhampton", _ => t }.to_string() };
    let jahr = |t: &str| t.len() == 4 && t.chars().all(|c| c.is_ascii_digit()) && (t.starts_with("18") || t.starts_with("19") || (t.starts_with("20") && t.as_bytes()[2] <= b'2'));
    let teile: Vec<String> = s.split_whitespace().filter(|t| !ZUSATZ.contains(t) && !jahr(t)).map(ersatz).collect();
    let n = teile.join(" ");
    if n.is_empty() { s.split_whitespace().collect::<Vec<_>>().join(" ") } else { n }
}

/// Ratcliff/Obershelp similarity like Python's difflib.SequenceMatcher.ratio() (no junk heuristic; names are short).
pub fn ratio(a: &str, b: &str) -> f32 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.is_empty() && b.is_empty() { return 1.0; }
    fn treffer(a: &[char], b: &[char]) -> usize {
        let (mut bi, mut bj, mut bk) = (0, 0, 0);
        let mut vor = vec![0usize; b.len() + 1];
        for i in 0..a.len() {
            let mut neu = vec![0usize; b.len() + 1];
            for j in 0..b.len() {
                if a[i] == b[j] { neu[j + 1] = vor[j] + 1; if neu[j + 1] > bk { bk = neu[j + 1]; bi = i + 1 - bk; bj = j + 1 - bk; } }
            }
            vor = neu;
        }
        if bk == 0 { return 0; }
        bk + treffer(&a[..bi], &b[..bj]) + treffer(&a[bi + bk..], &b[bj + bk..])
    }
    2.0 * treffer(&a, &b) as f32 / (a.len() + b.len()) as f32
}

/// bank.py aehnlich(): 1.0 for equal normalized names, 0 when only one side is a reserve team, 0.88/0.9 when the
/// shorter name is part of the longer one (at most two words more), otherwise the sequence ratio.
pub fn aehnlich(a: &str, b: &str) -> f32 {
    let (na, nb) = (norm(a), norm(b));
    aehnlich_norm(&na, &nb)
}
fn aehnlich_norm(na: &str, nb: &str) -> f32 {
    if na.is_empty() || nb.is_empty() { return 0.0; }
    if na == nb { return 1.0; }
    let (ta, tb): (HashSet<&str>, HashSet<&str>) = (na.split(' ').collect(), nb.split(' ').collect());
    if ta.symmetric_difference(&tb).any(|t| RESERVE.contains(t)) { return 0.0; }
    let (klein, gross) = if ta.len() <= tb.len() { (&ta, &tb) } else { (&tb, &ta) };
    let mut teil = 0.0;
    if klein.is_subset(gross) && gross.len() <= klein.len() + 2 && klein.iter().map(|t| t.len()).sum::<usize>() >= 4 { teil = if klein.len() >= 2 { 0.9 } else { 0.88 }; }
    // a cheap bound first: the ratio can not exceed 2·min/(a+b)
    let (la, lb) = (na.chars().count(), nb.chars().count());
    if teil == 0.0 && 2.0 * la.min(lb) as f32 / ((la + lb) as f32) < SCHWELLE { return 0.0; }
    ratio(na, nb).max(teil)
}

/// bank.py zuordnen(): {club index: bank index}; every entry and every club at most once, the best pairs first.
/// Manual corrections (trainer_zuordnung.json) come first: a name forces the pair, null blocks the entry.
pub fn zuordnen(namen: &[(usize, String)]) -> HashMap<usize, usize> {
    let b = bank();
    let mut aus = HashMap::new();
    let mut belegt = HashSet::new();
    let nach_name: HashMap<&str, usize> = namen.iter().map(|(c, n)| (n.as_str(), *c)).collect();
    for (i, (club, _, _)) in b.trainer.iter().enumerate() {
        if let Some(ziel) = b.manuell.get(club) {
            belegt.insert(i);
            if let Some(c) = ziel.as_deref().and_then(|z| nach_name.get(z)) { aus.insert(*c, i); }
        }
    }
    let bank_norm: Vec<String> = b.trainer.iter().map(|(club, _, _)| norm(club)).collect();
    let mut paare = vec![];
    for (c, name) in namen {
        if aus.contains_key(c) { continue; }
        let n = norm(name);
        for (i, bn) in bank_norm.iter().enumerate() {
            if belegt.contains(&i) { continue; }
            let s = aehnlich_norm(&n, bn);
            if s >= SCHWELLE { paare.push((s, *c, i)); }
        }
    }
    paare.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap().then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
    for (_, c, i) in paare {
        if aus.contains_key(&c) || belegt.contains(&i) { continue; }
        aus.insert(c, i);
        belegt.insert(i);
    }
    aus
}

impl World {
    /// Once per world (service.py erstausstattung): coaches from the bank for the clubs found, the rest of the bank
    /// (not "Trainer gesucht") into the pool. Clubs without a match keep their coach. `ausser` keeps the own club out.
    pub fn trainer_ausstatten(&mut self, ausser: Option<usize>) {
        if self.trainer_an { return; }
        self.trainer_an = true;
        let namen: Vec<(usize, String)> = self.clubs.iter().enumerate().filter(|(c, _)| Some(*c) != ausser).map(|(c, cl)| (c, cl.name.clone())).collect();
        let zu = zuordnen(&namen);
        let b = bank();
        let vergeben: HashSet<usize> = zu.values().copied().collect();
        for (&c, &i) in &zu {
            let (_, t, gesucht) = &b.trainer[i];
            if *gesucht { continue; }
            let cl = &mut self.clubs[c];
            cl.manager = t.name.clone();
            cl.style = t.stil_wert();
            cl.trainer = Some(t.clone());
        }
        self.trainer_pool = b.trainer.iter().enumerate().filter(|(i, (_, _, g))| !vergeben.contains(i) && !*g).map(|(_, (_, t, _))| t.clone()).collect();
    }

    /// A fired AI coach leaves his club; a bank coach goes to the pool and is free again after SPERRE_TAGE.
    pub fn trainer_entlassen(&mut self, c: usize, heute: u32) {
        if let Some(mut t) = self.clubs[c].trainer.take() {
            t.frei_ab = heute + SPERRE_TAGE;
            self.trainer_pool.push(t);
        }
    }

    /// service.py _kandidat(): the free pool coach whose fame is nearest to the club's size (1 + 4·reputation), the same
    /// country as the club's league preferred, a foreign one only now and then. None when nobody fits.
    pub fn trainer_kandidat(&self, c: usize, heute: u32, zufall: impl Fn(&str) -> f32) -> Option<usize> {
        let cl = &self.clubs[c];
        let ziel = 1.0 + (4.0 * cl.rep / 100.0).round();
        let land = self.leagues[cl.league].land.to_lowercase();
        let mut best: Option<(f32, usize)> = None;
        for (i, p) in self.trainer_pool.iter().enumerate() {
            if p.frei_ab > heute { continue; }
            let ab = (p.fame as f32 - ziel).abs();
            if ab > 1.0 { continue; }
            let gleich = !land.is_empty() && p.land.to_lowercase() == land;
            if !land.is_empty() && !gleich && zufall(&format!("fremd|{c}|{heute}|{}", p.name)) >= 0.15 { continue; }
            let s = -ab + if gleich { 1.5 } else { 0.0 } + 0.3 * zufall(&format!("kand|{c}|{heute}|{}", p.name));
            if best.map_or(true, |(b, _)| s > b) { best = Some((s, i)); }
        }
        best.map(|x| x.1)
    }

    /// A pool coach takes over club `c` (service.py _verpflichten); returns his name.
    pub fn trainer_verpflichten(&mut self, c: usize, i: usize, saison: u32) -> String {
        let mut t = self.trainer_pool.remove(i);
        t.frei_ab = 0;
        t.seit = format!("S{saison}");
        let cl = &mut self.clubs[c];
        cl.manager = t.name.clone();
        cl.style = t.stil_wert();
        cl.trainer = Some(t);
        cl.manager.clone()
    }
}

impl World {
    /// Every coach gets his two formations once: a coach at a club those of the club's records in the world (they are the
    /// club's real formations), a coach in the pool a formation of his playing idea from the world's formations; when both
    /// records are the same, the fallback is the formation of another club whose coach has the same idea.
    pub fn trainer_formationen(&mut self, pesdb: &std::path::Path) {
        if !self.clubs.iter().filter_map(|c| c.trainer.as_ref()).chain(self.trainer_pool.iter()).any(|t| t.formation.is_empty()) { return; }
        let alle = crate::fl27::alle_formationen(pesdb);
        if alle.is_empty() { return; }
        let idee_von = |c: usize, w: &World| w.clubs[c].trainer.as_ref().map(|t| t.idee.clone()).unwrap_or_default();
        // formations by playing idea: (idea, formation) of every club with a coach
        let mut nach_idee: Vec<(String, Vec<(u8, u8, u8)>)> = vec![];
        for c in 0..self.clubs.len() { if let Some(f) = alle.get(&self.clubs[c].id).and_then(|v| v.first()) { nach_idee.push((idee_von(c, self), f.clone())); } }
        let wahl = |idee: &str, nicht: &[(u8, u8, u8)], samen: &str| -> Vec<(u8, u8, u8)> {
            let name_nicht = crate::fl27::formation_name(nicht);
            let k: Vec<&Vec<(u8, u8, u8)>> = nach_idee.iter().filter(|x| x.0 == idee && crate::fl27::formation_name(&x.1) != name_nicht).map(|x| &x.1).collect();
            let k = if k.is_empty() { nach_idee.iter().filter(|x| crate::fl27::formation_name(&x.1) != name_nicht).map(|x| &x.1).collect() } else { k };
            if k.is_empty() { return nicht.to_vec(); }
            k[(crate::mensch::zufall(&[crate::id("formation"), samen]) * k.len() as f32) as usize % k.len()].clone()
        };
        for c in 0..self.clubs.len() {
            let id = self.clubs[c].id;
            let Some(t) = self.clubs[c].trainer.as_ref() else { continue };
            if !t.formation.is_empty() { continue; }
            let (name, idee) = (t.name.clone(), t.idee.clone());
            let v = alle.get(&id).cloned().unwrap_or_default();
            // a club without a record in the world: a formation of his idea, so nobody stays without one
            let fav = v.first().cloned().unwrap_or_else(|| wahl(&idee, &[], &format!("{name}|1")));
            let aus = v.get(1).filter(|f| crate::fl27::formation_name(f) != crate::fl27::formation_name(&fav)).cloned().unwrap_or_else(|| wahl(&idee, &fav, &name));
            let t = self.clubs[c].trainer.as_mut().unwrap();
            t.formation = fav;
            t.ausweich = aus;
        }
        for i in 0..self.trainer_pool.len() {
            if !self.trainer_pool[i].formation.is_empty() { continue; }
            let (name, idee) = (self.trainer_pool[i].name.clone(), self.trainer_pool[i].idee.clone());
            let fav = wahl(&idee, &[], &format!("{name}|1"));
            let aus = wahl(&idee, &fav, &format!("{name}|2"));
            self.trainer_pool[i].formation = fav;
            self.trainer_pool[i].ausweich = aus;
        }
    }
    /// The formation a club's coach chooses for a match: his favourite, against a clearly stronger opponent often the
    /// more defensive fallback, otherwise now and then the fallback too. None without a coach or formations.
    pub fn formation_wahl(&self, c: usize, gegner: usize, tag: u32) -> Option<Vec<(u8, u8, u8)>> {
        let t = self.clubs[c].trainer.as_ref()?;
        if t.formation.len() != 11 { return None; }
        if t.ausweich.len() != 11 { return Some(t.formation.clone()); }
        let abwehr = |f: &[(u8, u8, u8)]| f.iter().filter(|x| (1..=3).contains(&x.0)).count() as f32 + f.iter().filter(|x| x.0 != 0).map(|x| 47.0 - x.1 as f32).sum::<f32>() / 400.0;
        let defensiver = abwehr(&t.ausweich) > abwehr(&t.formation);
        let unterschied = crate::mensch::kaderstaerke(self, gegner) - crate::mensch::kaderstaerke(self, c);
        let chance = if defensiver && unterschied > 4.0 { 0.6 } else if !defensiver && unterschied < -4.0 { 0.4 } else { 0.15 };
        let z = crate::mensch::zufall(&[crate::id("formationswahl"), &self.clubs[c].id.to_string(), &tag.to_string()]);
        Some(if z < chance { t.ausweich.clone() } else { t.formation.clone() })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn trainer_haben_zwei_formationen_und_nehmen_sie_mit() {
        let d = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&d) { return; }
        let mut w = crate::world::World::from_pesdb(&d, 7).unwrap();
        w.trainer_ausstatten(None);
        w.trainer_formationen(&d);
        let mit: Vec<usize> = (0..w.clubs.len()).filter(|&c| w.clubs[c].trainer.as_ref().map_or(false, |t| t.formation.len() == 11)).collect();
        assert!(mit.len() > 100, "{} Vereine mit Trainerformation", mit.len());
        let c = mit[0];
        let t = w.clubs[c].trainer.clone().unwrap();
        assert_eq!(t.ausweich.len(), 11);
        assert!(crate::fl27::formation_name(&t.formation).split('-').map(|x| x.parse::<usize>().unwrap()).sum::<usize>() == 10, "{}", crate::fl27::formation_name(&t.formation));
        // he chooses between the two, over many matches both come up
        let namen: std::collections::HashSet<String> = (0..200).filter_map(|tag| w.formation_wahl(c, mit[1], tag)).map(|f| crate::fl27::formation_name(&f)).collect();
        assert!(!namen.is_empty() && namen.len() <= 2);
        // fired and hired elsewhere: the formations go along
        w.trainer_entlassen(c, 0);
        let i = w.trainer_pool.iter().position(|x| x.name == t.name).unwrap();
        let ziel = mit[2];
        w.trainer_verpflichten(ziel, i, 0);
        assert_eq!(w.clubs[ziel].trainer.as_ref().unwrap().formation, t.formation);
    }

    use super::*;

    #[test]
    fn namen_wie_im_original() {
        assert_eq!(norm("FC Bayern München"), "bayern munchen");
        assert_eq!(norm("Borussia Dortmund 09"), "borussia dortmund");
        assert!((ratio("abcd", "bcde") - 0.75).abs() < 1e-6, "difflib: 2·3/8");
        assert!(aehnlich("Genoa", "Genoa Calcio") >= SCHWELLE);
        assert_eq!(aehnlich("Real Sociedad", "Real Sociedad B"), 0.0);
        assert!((aehnlich("Ajax", "Ajax Cape Town") - 0.88).abs() < 1e-6, "wie im Original: Teilname zählt 0,88");
        assert!(aehnlich("Sunderland AFC", "Sunderland") >= SCHWELLE);
    }

    #[test]
    fn zuordnung_mit_handkorrektur() {
        let namen = vec![(0, "Lille OSC".to_string()), (1, "Athletic Club".to_string()), (2, "RSC Anderlecht".to_string()), (3, "Phantasia United".to_string())];
        let zu = zuordnen(&namen);
        let b = bank();
        assert_eq!(b.trainer[zu[&0]].0, "LOSC Lille");
        assert_eq!(b.trainer[zu[&1]].0, "Athletic Bilbao");
        assert_eq!(b.trainer[zu[&2]].1.name, "Vítor Bruno");
        assert!(!zu.contains_key(&3));
    }

    #[test]
    fn echte_welt_bekommt_echte_trainer() {
        let d = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&d) { return; }
        let mut w = World::from_pesdb(&d, 7).unwrap();
        let own = w.clubs.iter().position(|c| c.id == 396).unwrap();
        w.trainer_ausstatten(Some(own));
        let mit = w.clubs.iter().filter(|c| c.trainer.is_some()).count();
        assert!(mit >= 150, "nur {mit} Vereine mit echtem Trainer");
        assert!(w.clubs[own].trainer.is_none(), "der eigene Verein bekommt keinen Bank-Trainer");
        assert!(!w.trainer_pool.is_empty());
        let namen: HashSet<&str> = w.clubs.iter().filter_map(|c| c.trainer.as_ref().map(|t| t.name.as_str())).collect();
        assert_eq!(namen.len(), mit, "jeder Bank-Trainer höchstens einmal");
        // fired and re-hired: the pool gets him back only after the lock
        let c = w.clubs.iter().position(|c| c.trainer.is_some()).unwrap();
        let name = w.clubs[c].manager.clone();
        w.trainer_entlassen(c, 100);
        assert!(w.trainer_pool.iter().any(|t| t.name == name && t.frei_ab == 100 + SPERRE_TAGE));
        let i = w.trainer_pool.iter().position(|t| t.name == name).unwrap();
        assert_ne!(w.trainer_kandidat(c, 100, |_| 0.0), Some(i), "gesperrt");
    }
}
