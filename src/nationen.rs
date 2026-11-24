//! Nationality (Player.bin bit 233): names, the country of a league and the squad rules (assets/regeln/kaderregeln.toml).
use crate::game::*;
use crate::world::World;

/// Is a nationality European (PES numbering: the European countries lie between 190 and 310)? A simplification of EU rules.
pub fn europaeisch(n: u16) -> bool { (190..=310).contains(&n) }

/// Name of a nationality: from the real data (304 players, 46 countries) or a league country; "" when unknown.
pub fn name(w: &World, n: u16) -> String {
    if n == 0 { return String::new(); }
    static T: std::sync::OnceLock<std::collections::HashMap<u16, String>> = std::sync::OnceLock::new();
    let t = T.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/regeln/nationen.json")).expect("nationen.json");
        v.as_object().map(|m| m.iter().filter_map(|(k, x)| Some((k.parse().ok()?, x.as_str()?.to_string()))).collect()).unwrap_or_default()
    });
    let x = if let Some(x) = t.get(&n) { x.clone() } else { (0..w.leagues.len()).find(|&l| w.land_nat(l) == n && !w.leagues[l].land.is_empty()).map(|l| w.leagues[l].land.clone()).unwrap_or_default() };
    x
}
/// The name for the screen (English in the English interface); `name` stays the key for lookups.
pub fn name_anzeige(w: &World, n: u16) -> String { anzeige(&name(w, n)) }
/// A country name (German, as the lists keep it) in the interface language: English from the Natural Earth names
/// (assets/geo/laender.json) plus the home nations; unknown names stay as they are.
pub fn anzeige(de: &str) -> String {
    if crate::inhalt::sprache() != crate::inhalt::Sprache::En { return de.to_string(); }
    static T: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();
    let t = T.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/geo/laender.json")).unwrap_or_default();
        let mut m: std::collections::HashMap<String, String> = v["laender"].as_array().into_iter().flatten()
            .filter_map(|l| Some((l["de"].as_str()?.to_string(), l["en"].as_str()?.to_string()))).collect();
        for (d, e) in [("Schottland", "Scotland"), ("Nordirland", "Northern Ireland"), ("Wales", "Wales"), ("England", "England"), ("Südkorea", "South Korea"), ("Elfenbeinküste", "Ivory Coast"), ("Tschechien", "Czechia"), ("Bosnien-Herzegowina", "Bosnia and Herzegovina")] { m.insert(d.into(), e.into()); } // roh
        m
    });
    t.get(de).cloned().unwrap_or_else(|| de.to_string())
}

impl World {
    /// Leagues whose country the world file does not name („Land der A-League Men“) get the name of the nationality most
    /// of their players have, when it is known (nationen.json); cups of that country follow.
    pub fn laender_benennen(&mut self) {
        let vorsilbe = crate::inhalt::t("ui.pesdb.land_der").split("%1").next().unwrap_or("").to_string();
        if vorsilbe.trim().is_empty() { return; }
        for l in 0..self.leagues.len() {
            let alt = self.leagues[l].land.clone();
            if !alt.starts_with(&vorsilbe) { continue; }
            let n = self.land_nat(l);
            // the nationality's name, else the country most of its clubs are listed in at Wikidata (assets/geo)
            let liste = crate::regeln::wert_oder("kaderregeln", "liga_land").and_then(|m| m.get(&self.leagues[l].name)).and_then(|v| v.as_str()).map(|x| x.to_string());
            let neu = liste.unwrap_or_else(|| { let x = name(self, n); if x.is_empty() || x.starts_with(&vorsilbe) { crate::geo::liga_land_sicher(self, l).unwrap_or_default() } else { x } });
            if neu.is_empty() || neu.starts_with(&vorsilbe) { continue; }
            for k in 0..self.leagues.len() { if self.leagues[k].land == alt { self.leagues[k].land = neu.clone(); } }
            for cu in self.cups.iter_mut() { if cu.country == alt { cu.country = neu.clone(); if cu.kind == 0 { cu.name = crate::cups::pokalname(&cu.key, &neu); } } }
        }
    }
    /// The nationality most players of a league have: the league's country.
    pub fn land_nat(&self, l: usize) -> u16 {
        let mut z: std::collections::HashMap<u16, usize> = Default::default();
        for &c in &self.leagues[l].teams { for &p in &self.clubs[c].squad { let n = self.players[p].nat; if n > 0 { *z.entry(n).or_default() += 1; } } }
        z.into_iter().max_by_key(|x| x.1).map(|x| x.0).unwrap_or(0)
    }
}

impl Career {
    /// Squad rule check for a signing of `p` by club `c` (kaderregeln.toml): None when it is allowed, else the reason.
    pub fn kaderregel_verstoss(&self, c: usize, p: usize) -> Option<String> {
        if !self.kaderregeln { return None; }
        let l = self.world.clubs[c].league;
        let land = self.world.leagues[l].land.clone();
        let regel = |k: &str| crate::regeln::wert_oder("kaderregeln", &format!("land.{land}.{k}")).and_then(|v| v.as_f64()).map(|x| x as usize);
        let n = self.world.players[p].nat;
        if n == 0 { return None; }
        let kader: Vec<u16> = self.world.clubs[c].squad.iter().filter(|&&x| x != p && self.world.players[x].youth == 0).map(|&x| self.world.players[x].nat).collect();
        if let Some(max) = regel(crate::id("nicht_europaeer_max")) {
            if !europaeisch(n) && kader.iter().filter(|&&x| x > 0 && !europaeisch(x)).count() >= max { return Some(crate::tf!("ui.kaderregel.nicht_europaeer", max, land)); }
        }
        if let Some(max) = regel(crate::id("auslaender_max")) {
            let heim = self.world.land_nat(l);
            if n != heim && kader.iter().filter(|&&x| x > 0 && x != heim).count() >= max { return Some(crate::tf!("ui.kaderregel.auslaender", max, land)); }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn laender_ohne_namen_bekommen_einen() {
        let d = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&d) { return; }
        let w = crate::world::World::from_pesdb(&d, 7).unwrap();
        let vorsilbe = crate::inhalt::t("ui.pesdb.land_der").split("%1").next().unwrap().to_string();
        let rest: Vec<String> = w.leagues.iter().filter(|l| l.land.starts_with(&vorsilbe)).map(|l| l.land.clone()).collect();
        let alle = w.leagues.len();
        eprintln!("ohne Namen: {} von {}: {:?}", rest.len(), alle, rest);
        assert!(w.leagues.iter().any(|l| l.land == "Österreich" || l.land == "Schweden" || l.land == "Australien"), "bekannte Länder benannt");
    }

    use super::*;

    #[test]
    fn nationalitaet_aus_player_bin() {
        let d = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&d) { return; }
        let w = World::from_pesdb(&d, 7).unwrap();
        // every player with real data has the matching nationality number (Messi 144 Argentina)
        let messi = w.players.iter().find(|p| p.id == 7511).unwrap();
        assert_eq!((messi.nat, name(&w, messi.nat).as_str()), (144, "Argentinien"));
        let eng = w.leagues.iter().position(|l| l.key == "ENGLAND_D1_LEAGUE").unwrap();
        assert_eq!(w.land_nat(eng), 204, "England");
        assert!(europaeisch(204) && !europaeisch(144));
    }
}
