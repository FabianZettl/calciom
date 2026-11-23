//! Präsidium-Konzept (konzepte/co-trainer-und-praesidium.md Teil 5, 23.11.2026): the club type sets how strict the board is
//! (a world club fires faster than a family club), three people with their own stance and mood, the escalation stage in
//! words, and side goals per season that count at its end. Numbers in assets/regeln/praesidium.toml.
use crate::game::*;
use crate::mensch::zufall;
use crate::regeln::{f, wert};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Person { pub name: String, pub rolle: u8, pub haltung: String }
/// A side goal of the season: key, the season it belongs to.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Nebenziel { pub key: String, pub season: u32 }

pub const TYPEN: [&str; 5] = [crate::id("weltklub"), crate::id("spitzenklub"), crate::id("etabliert"), crate::id("aufstrebend"), crate::id("familienklub")];
const ZIELE: [&str; 4] = [crate::id("eigengewaechse"), crate::id("gehaltsquote"), crate::id("tore"), crate::id("jung")];

impl Career {
    /// The club type from reputation and league tier (Konzept 5.2).
    pub fn vereinstyp(&self) -> &'static str {
        let (rep, stufe) = (self.club().rep, self.world.leagues[self.league()].stufe.max(1));
        TYPEN.iter().copied().find(|t| { let w = &wert("praesidium", "typ")[*t]; rep >= w["ruf_ab"].as_f64().unwrap_or(0.0) as f32 && stufe as u64 <= w["stufe"].as_u64().unwrap_or(9) }).unwrap_or(crate::id("familienklub"))
    }
    /// Below this board confidence the ultimatum comes, by club type (a patient board 5 lower).
    pub fn ultimatum_schwelle(&self) -> f32 {
        f("praesidium", &format!("typ.{}.ultimatum", self.vereinstyp())) - if self.vision() == crate::id("geduldig") { 5.0 } else { 0.0 }
    }
    /// The escalation stage 0 Ruhig, 1 Beobachtend, 2 Unruhig, 3 Krise, 4 Ultimatum.
    pub fn eskalation(&self) -> usize {
        if self.ultimatum.is_some() { return 4; }
        let v = self.mensch.vorstand;
        let s = wert("praesidium", &format!("typ.{}.stufen", self.vereinstyp())).as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect::<Vec<_>>()).unwrap_or_default();
        s.iter().position(|&g| v >= g).unwrap_or(3)
    }
    /// The three people of the board, made once per club (names from the country's players, a stance each).
    pub fn praesidium_an(&mut self) {
        let cid = self.club().id;
        if self.praesidium.len() == 3 && self.praesidium_verein == cid { return; }
        let nat = self.world.land_nat(self.league());
        let namen: Vec<&String> = self.world.players.iter().filter(|p| p.db && p.nat == nat).map(|p| &p.name).take(300).collect();
        let haltungen = wert("praesidium", "personen.haltungen").as_array().unwrap().iter().filter_map(|x| x.as_str()).collect::<Vec<_>>();
        self.praesidium = (0..3u8).map(|r| {
            let s = format!("praesidium:{cid}:{r}");
            let vor = namen.get((zufall(&[&s, "v"]) * namen.len().max(1) as f32) as usize).and_then(|n| n.split_whitespace().next()).unwrap_or("Klaus"); // roh
            let nach = namen.get((zufall(&[&s, "n"]) * namen.len().max(1) as f32) as usize).and_then(|n| n.split_whitespace().last()).unwrap_or("Becker"); // roh
            // the president carries the club's vision, the others draw their own
            let haltung = if r == 0 { self.vision().to_string() } else { haltungen[(zufall(&[&s, crate::id("h")]) * haltungen.len() as f32) as usize % haltungen.len()].to_string() };
            Person { name: format!("{vor} {nach}"), rolle: r, haltung }
        }).collect();
        self.praesidium_verein = cid;
    }
    /// One person's mood 0–100: the board's confidence, seen through his role and stance.
    pub fn person_stimmung(&self, p: &Person) -> f32 {
        let v = self.mensch.vorstand;
        // the table only counts once a few matches are played
        let gespielt = self.world.table(self.league()).iter().find(|r| r.club == self.own).map_or(0, |r| r.p);
        let ziel = self.club().goal_pos as f32;
        let rank = if gespielt >= 5 { self.world.rank_of(self.own) as f32 } else { ziel };
        let kasse = if self.club().balance >= 0.0 { 4.0 } else { -8.0 };
        let mut m = v + match p.rolle { 1 => (ziel - rank) * 1.5, 2 => kasse, _ => 0.0 };
        m += match p.haltung.as_str() {
            "ehrgeizig" => if rank > ziel { -6.0 } else { 2.0 },
            "sparsam" => kasse,
            "geduldig" => 5.0,
            "investor" => (ziel - rank) * 1.0,
            "fanpraesident" => (self.fans - 50.0) / 5.0,
            _ => 0.0,
        };
        m.clamp(0.0, 100.0)
    }
    /// The side goals of a new season: two from the catalogue, drawn by the stances of the board.
    pub fn nebenziele_setzen(&mut self) {
        let season = self.world.season;
        if self.nebenziele.iter().any(|z| z.season == season) { return; }
        self.praesidium_an();
        let cid = self.club().id.to_string();
        let mut gewicht: Vec<f32> = ZIELE.iter().map(|z| match *z {
            "eigengewaechse" => if self.praesidium.iter().any(|p| p.haltung == "traditionell") { 3.0 } else { 1.0 },
            "gehaltsquote" => if self.praesidium.iter().any(|p| p.haltung == "sparsam") { 3.0 } else { 1.0 },
            "tore" => if self.praesidium.iter().any(|p| p.haltung == "fanpraesident") { 3.0 } else { 1.0 },
            _ => if self.praesidium.iter().any(|p| p.haltung == "investor") { 3.0 } else { 1.0 },
        }).collect();
        for k in 0..f("praesidium", "nebenziele.anzahl") as usize {
            let summe: f32 = gewicht.iter().sum();
            if summe <= 0.0 { break; }
            let mut x = zufall(&["nebenziel", &cid, &season.to_string(), &k.to_string()]) * summe; // roh
            let i = gewicht.iter().position(|&g| { x -= g; x < 0.0 }).unwrap_or(0);
            gewicht[i] = 0.0;
            self.nebenziele.push(Nebenziel { key: ZIELE[i].into(), season });
        }
    }
    /// Whether a side goal is met now, and the line that says how far.
    pub fn nebenziel_stand(&self, key: &str) -> (bool, String) {
        let n = |k: &str| f("praesidium", &format!("nebenziele.{k}"));
        match key {
            "eigengewaechse" => {
                let cid = self.club().id;
                let x = self.squad().iter().filter(|&&p| self.world.eigen.get(&self.world.players[p].id) == Some(&cid) && self.world.players[p].apps >= 5).count();
                (x as f32 >= n(crate::id("eigengewaechse")), crate::tf!("ui.pr.ziel.eigengewaechse", x, n(crate::id("eigengewaechse"))))
            }
            "gehaltsquote" => {
                let q = (self.world.wage_bill(self.own) / self.umsatz_prognose().max(1.0)) as f32;
                (q <= n(crate::id("gehaltsquote")), crate::tf!("ui.pr.ziel.gehaltsquote", format!("{:.0}", q * 100.0), format!("{:.0}", n(crate::id("gehaltsquote")) * 100.0)))
            }
            "tore" => {
                let (t, s) = self.world.table(self.league()).iter().find(|r| r.club == self.own).map(|r| (r.gf, r.p)).unwrap_or((0, 0));
                let q = t as f32 / s.max(1) as f32;
                (q >= n("tore_je_spiel"), crate::tf!("ui.pr.ziel.tore", format!("{q:.1}"), format!("{:.1}", n("tore_je_spiel"))))
            }
            _ => {
                let a = self.xi.iter().map(|&p| self.world.players[p].age as f32).sum::<f32>() / self.xi.len().max(1) as f32;
                (a <= n(crate::id("altersschnitt")), crate::tf!("ui.pr.ziel.jung", format!("{a:.1}"), format!("{:.1}", n(crate::id("altersschnitt")))))
            }
        }
    }
    /// Season end: each side goal counts for the board's confidence, with its reason.
    pub fn nebenziele_abrechnen(&mut self) {
        let season = self.world.season;
        let ziele: Vec<String> = self.nebenziele.iter().filter(|z| z.season == season).map(|z| z.key.clone()).collect();
        for k in ziele {
            let (ok, zeile) = self.nebenziel_stand(&k);
            let w = f("praesidium", if ok { "nebenziele.erfuellt" } else { "nebenziele.verfehlt" });
            let grund = format!("{}: {zeile}", crate::inhalt::t(&format!("ui.pr.ziel.{k}.name")));
            self.klima(0.0, 0.0, w, &grund);
        }
        self.nebenziele.retain(|z| z.season > season);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vereinstyp_personen_nebenziele() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 7);
        let mut c = Career::new(w, 0, "Test");
        let streng = |c: &Career| c.ultimatum_schwelle();
        // a big club's board is stricter than a small one's
        c.world.clubs[c.own].rep = 95.0;
        let gross = streng(&c);
        c.world.clubs[c.own].rep = 10.0;
        let klein = streng(&c);
        if c.world.leagues[c.league()].stufe <= 1 { assert!(gross > klein, "{gross} > {klein}"); }
        c.praesidium_an();
        assert_eq!(c.praesidium.len(), 3);
        assert!(c.praesidium.iter().all(|p| !p.name.is_empty()));
        for p in c.praesidium.clone() { let m = c.person_stimmung(&p); assert!((0.0..=100.0).contains(&m)); }
        c.nebenziele_setzen();
        assert_eq!(c.nebenziele.len(), 2);
        assert_ne!(c.nebenziele[0].key, c.nebenziele[1].key);
        let g0 = c.mensch.gruende.len();
        c.nebenziele_abrechnen();
        assert!(c.nebenziele.is_empty() && c.mensch.gruende.len() >= g0 + 2, "side goals count at season end, each with its reason");
        assert!(c.eskalation() <= 4);
    }
}
