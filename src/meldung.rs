//! Squad registration for the European competitions (Fabian 24.11.): list A of at most 25 players, list B free for
//! own players up to 21 (second team and youth too); whoever is not registered cannot play there. The list is set
//! before the first match (the assistant registers the best if nobody does) and in the winter window up to three
//! players may change. Numbers in assets/regeln/uefa.toml [meldung].
use crate::game::*;
use serde::{Deserialize, Serialize};

fn f(k: &str) -> f32 { crate::regeln::f("uefa", &format!("meldung.{k}")) }

/// A registration: the competition's key, the season, list A, and how many players were changed in winter.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Meldung { pub key: String, pub season: u32, pub liste_a: Vec<usize>, pub nachgemeldet: u8 }

impl Career {
    /// The European competitions we play this season (cup indices).
    pub fn europa_wettbewerbe(&self) -> Vec<usize> { (0..self.world.cups.len()).filter(|&k| (1..=3).contains(&self.world.cups[k].kind) && self.world.cups[k].teams.contains(&self.own)).collect() }
    /// List B: own players up to the age limit, from the first team, the second team or the academy; free to play.
    pub fn liste_b(&self, p: usize) -> bool {
        let pl = &self.world.players[p];
        let z = self.zweite_mannschaft();
        pl.age as f32 <= f("liste_b_alter") && (pl.team == self.own as i32 || z.is_some_and(|z| pl.team == z as i32))
    }
    /// The registration for competition `ci` this season, made by the assistant with the 25 best when there is none.
    pub fn meldung(&mut self, ci: usize) -> &Meldung {
        let (key, season) = (self.world.cups[ci].key.clone(), self.world.season);
        if !self.meldungen.iter().any(|m| m.key == key && m.season == season) {
            let mut kader: Vec<usize> = self.squad().into_iter().filter(|&p| !self.liste_b(p)).collect();
            kader.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
            kader.truncate(f("liste_a") as usize);
            self.meldungen.push(Meldung { key: key.clone(), season, liste_a: kader, nachgemeldet: 0 });
        }
        self.meldungen.retain(|m| m.season + 1 >= season);
        self.meldungen.iter().find(|m| m.key == key && m.season == season).unwrap()
    }
    pub fn gemeldet(&mut self, ci: usize, p: usize) -> bool { self.liste_b(p) || self.meldung(ci).liste_a.contains(&p) }
    /// Whether the list may change now: before the competition's first match, or in the winter window (at most three).
    pub fn meldung_offen(&mut self, ci: usize) -> Result<bool, String> {
        let erster = self.world.cups[ci].fx.iter().filter(|x| x.f.h == self.own || x.f.a == self.own).map(|x| x.f.day).min().unwrap_or(u32::MAX);
        let winter = crate::world::WINDOWS[1];
        if self.world.day < erster { return Ok(false); }
        if self.world.day >= winter.0 && self.world.day <= winter.1 {
            let n = self.meldung(ci).nachgemeldet;
            if (n as f32) < f("nachmeldungen") { return Ok(true); }
            return Err(crate::inhalt::t("ui.meldung.winter_voll").into());
        }
        Err(crate::inhalt::t("ui.meldung.geschlossen").into())
    }
    /// Register or take off a player (list A); a winter change counts once.
    pub fn melden(&mut self, ci: usize, p: usize) -> Result<String, String> {
        if self.liste_b(p) { return Err(crate::inhalt::t("ui.meldung.liste_b_frei").into()); }
        let winter = self.meldung_offen(ci)?;
        let max = f("liste_a") as usize;
        let (key, season) = (self.world.cups[ci].key.clone(), self.world.season);
        let _ = self.meldung(ci);
        let m = self.meldungen.iter_mut().find(|m| m.key == key && m.season == season).unwrap();
        let n = self.world.players[p].name.clone();
        if let Some(i) = m.liste_a.iter().position(|&x| x == p) { m.liste_a.remove(i); return Ok(crate::tf!("ui.meldung.abgemeldet", n)); }
        if m.liste_a.len() >= max { return Err(crate::tf!("ui.meldung.voll", max)); }
        m.liste_a.push(p);
        if winter { m.nachgemeldet += 1; }
        Ok(crate::tf!("ui.meldung.gemeldet", n))
    }
    /// Before a European match: players not registered leave the eleven; the assistant fills the places and says so.
    pub fn meldung_pruefen(&mut self, ci: usize) {
        if !(1..=3).contains(&self.world.cups[ci].kind) { return; }
        let raus: Vec<usize> = self.xi.clone().into_iter().filter(|&p| !self.gemeldet(ci, p)).collect();
        if raus.is_empty() { return; }
        // the unregistered are kept out like a suspension for the choice, then given back their state
        let alle: Vec<usize> = self.squad().into_iter().filter(|&p| !self.gemeldet(ci, p)).collect();
        let vorher: Vec<u8> = alle.iter().map(|&p| self.world.players[p].susp).collect();
        for &p in &alle { self.world.players[p].susp = self.world.players[p].susp.max(1); }
        self.auto_lineup();
        for (k, &p) in alle.iter().enumerate() { self.world.players[p].susp = vorher[k]; }
        let namen: Vec<String> = raus.iter().map(|&p| self.world.players[p].name.clone()).collect();
        let co = self.co_name.clone();
        let wb = self.world.cups[ci].name.clone();
        self.msg(1, &crate::tf!("ui.pool.von_co", co), &crate::tf!("ui.meldung.nicht_gemeldet_titel", wb.clone()), &crate::tf!("ui.meldung.nicht_gemeldet", namen.join(", "), wb), Action::None);
    }
}

#[cfg(test)]
mod tests {
    use crate::game::*;
    /// Fabian 24.11.: 25 on list A, the young ones free, nobody unregistered plays, changes only at the right time.
    #[test]
    fn kadermeldung() {
        let mut c = Career::new(crate::world::testwelt(3), 2, "Test");
        let ci = 0;
        c.world.cups[ci].kind = 1;
        if !c.world.cups[ci].teams.contains(&c.own) { c.world.cups[ci].teams.push(c.own); }
        let a = c.meldung(ci).liste_a.clone();
        assert!(a.len() <= 25 && a.iter().all(|&p| !c.liste_b(p)));
        // an unregistered regular in the eleven is replaced before the match
        let draussen = c.squad().into_iter().find(|&p| !c.gemeldet(ci, p));
        if let Some(p) = draussen {
            c.xi = vec![p];
            c.meldung_pruefen(ci);
            assert!(!c.xi.contains(&p));
        }
        // before the first match changes are free, the list stays within 25
        c.world.day = 0;
        if let Some(&p) = a.first() { assert!(c.melden(ci, p).is_ok()); assert!(c.melden(ci, p).is_ok()); }
    }
}
