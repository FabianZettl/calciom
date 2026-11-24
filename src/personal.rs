//! Paket 3o: training, development and staff (Mechanik-Übergabe §18; core/personal/personal.toml,
//! entwicklung.toml [belastung] [stab], entwicklung3 phases). The staff replaces the fixed training level 5 of 10.
use crate::game::*;
use crate::mensch::{arr, zufall};
use crate::regeln::f;
use crate::world::*;
use serde::{Deserialize, Serialize};

pub const ROLLEN: [&str; 5] = [crate::id("trainer"), "co_trainer", crate::id("arzt"), crate::id("physio"), crate::id("jugendleiter")];
pub fn rollen_de_t() -> [&'static str; 5] { [crate::inhalt::t("taktik.board.coach"), crate::inhalt::t("meldung.absender.co_trainer"), crate::inhalt::t("med.vg.absender"), crate::inhalt::t("pm.rolle1.physio"), crate::inhalt::t("meldung.absender.jugendleiter")] }
pub fn bereiche_t() -> [&'static str; 5] { [crate::inhalt::t("gb2.sp.teil.tor"), crate::inhalt::t("achse.Abwehr"), crate::inhalt::t("gb2.vgl.mittel"), crate::inhalt::t("bericht.achse.angriff"), crate::inhalt::t("pm.bereich.alle")] }
const VORNAMEN: [&str; 14] = ["Uwe", "Carla", "Dimitri", "Sanne", "Hakan", "Bruno", "Elif", "Marek", "Nora", "Ivo", "Ruth", "Gaspard", "Timo", "Lucía"]; // roh
const NACHNAMEN: [&str; 14] = ["Lorenz", "Bakker", "Ilić", "Engel", "Demir", "Rossi", "Kowalski", "Strand", "Pohl", "Marín", "Adler", "Fontaine", "Wolff", "Kaya"]; // roh

#[derive(Serialize, Deserialize, Clone)]
pub struct Stab { pub rolle: u8, pub name: String, pub f: f32, pub bereich: u8, pub gehalt: f64, pub bis: u32 }

fn gehalt(rolle: u8, faehigkeit: f32) -> f64 {
    let basis = crate::regeln::f_oder("personal", &format!("gehalt.basis.{}", ROLLEN[rolle as usize]), 60000.0) as f64;
    round_money(basis * (1.0 + f("personal", "gehalt.je_punkt") as f64 * (faehigkeit as f64 - 50.0)).max(0.4))
}
/// Load of a training unit of this build (entwicklung.toml [belastung] einheiten: frei, regeneration, leicht, normal, taktik, intensiv).
pub fn einheit_last(u: u8) -> f32 { [0.0, -1.0, 2.0, 4.0, 3.0, 8.0][u.min(5) as usize] }

impl Career {
    pub fn stab_an(&mut self) {
        if !self.stab.is_empty() { return; }
        let k = self.kraft() as f32;
        let st = arr("vereinswelt", "staff.start_staerke");
        let s0 = st[0].as_f64().unwrap() as f32 + (st[1].as_f64().unwrap() - st[0].as_f64().unwrap()) as f32 * k;
        let abs = self.abs();
        let co = self.co_name.clone();
        for (i, (rolle, bereich)) in [(0u8, 4u8), (0, 4), (1, 4), (2, 4), (3, 4), (4, 4)].into_iter().enumerate() {
            let name = if rolle == 1 { co.clone() } else { self.echter_stab(i).unwrap_or_else(|| self.stab_name(&format!("start{i}"))) };
            let fz = (s0 + (zufall(&["2026", crate::id("stab"), &self.club().id.to_string(), &i.to_string()]) - 0.5) * 10.0).clamp(20.0, 95.0).round();
            self.stab.push(Stab { rolle, name, f: fz, bereich, gehalt: gehalt(rolle, fz), bis: abs + 2 * crate::date::SEASON_DAYS });
        }
        self.echte_namen_einsetzen();
    }
    /// The real person for starting slot `i` (two coaches, co-trainer, doctor, physio, youth) from vereinsfuehrung.toml.
    fn echter_stab(&self, i: usize) -> Option<String> {
        let club = self.club().name.clone();
        let (rolle, n) = [("fitness_coach", 0), ("goalkeeping_coach", 0), ("assistant_coach", 0), ("doctor", 0), ("physio", 0), ("academy_head", 0)][i.min(5)]; // roh
        let mut v = crate::vereinsfuehrung::personen(&club, rolle);
        if v.is_empty() && i == 5 { v = crate::vereinsfuehrung::personen(&club, "youth_coach"); } // roh
        if v.is_empty() && i == 1 { v = crate::vereinsfuehrung::personen(&club, "analyst"); } // roh
        v.into_iter().nth(n)
    }
    /// Saves from before the real names (Fabian 24.11.): the invented starting staff and co-trainer get the club's real
    /// people; staff hired later keep their names.
    pub fn echte_namen_einsetzen(&mut self) {
        let co_echt = crate::vereinsfuehrung::personen(&self.club().name, "assistant_coach").into_iter().next(); // roh
        for i in 0..6 {
            let alt = self.stab_name(&format!("start{i}"));
            let neu = if i == 2 { co_echt.clone() } else { self.echter_stab(i) };
            if let Some(neu) = neu {
                for s in self.stab.iter_mut().filter(|s| s.name == alt) { s.name = neu.clone(); }
            }
        }
        if let Some(co) = co_echt {
            if self.co_erfunden() { let alt = self.co_name.clone(); for s in self.stab.iter_mut().filter(|s| s.name == alt) { s.name = co.clone(); } self.co_name = co; }
        }
    }
    /// The co-trainer still has one of the five invented starting names.
    fn co_erfunden(&self) -> bool {
        ["ui.game.thomas_brueck", "ui.game.mehmet_ayd_n", "ui.game.rafael_ortega", "ui.game.sven_lindqvist", "ui.game.marco_ferri"].iter() // roh
            .any(|k| [crate::inhalt::Sprache::De, crate::inhalt::Sprache::En].iter().any(|&s| crate::inhalt::t_in(s, k) == self.co_name))
    }
    fn stab_name(&self, k: &str) -> String {
        let z = |x: &str| zufall(&["2026", crate::id("stabname"), k, x]);
        format!("{} {}", VORNAMEN[(z(crate::id("v")) * 14.0) as usize % 14], NACHNAMEN[(z(crate::id("n")) * 14.0) as usize % 14])
    }
    /// Applicants of this month per role (personal.toml markt.bewerber_je_monat), fixed per month.
    pub fn stab_bewerber(&self) -> Vec<Stab> {
        let monat = (self.abs() / 30).to_string();
        let rep = self.club().rep;
        let mut v = vec![];
        for (r, rolle) in ROLLEN.iter().enumerate() {
            let n = crate::regeln::f_oder("personal", &format!("markt.bewerber_je_monat.{rolle}"), 2.0) as usize;
            for i in 0..n.min(3) {
                let k = format!("{monat}|{rolle}|{i}");
                let fz = (30.0 + rep * 0.4 + zufall(&["2026", crate::id("stabbewerber"), &k]) * 30.0).clamp(20.0, 95.0).round();
                let bereich = if r == 0 { (zufall(&["2026", crate::id("stabbereich"), &k]) * 5.0) as u8 % 5 } else { 4 };
                v.push(Stab { rolle: r as u8, name: self.stab_name(&k), f: fz, bereich, gehalt: gehalt(r as u8, fz), bis: 0 });
            }
        }
        v
    }
    pub fn stab_einstellen(&mut self, i: usize) -> Result<(), String> {
        let mut s = self.stab_bewerber().swap_remove(i);
        let max = crate::regeln::f_oder("personal", &format!("markt.stab_max.{}", ROLLEN[s.rolle as usize]), 2.0) as usize;
        if self.stab.iter().filter(|x| x.rolle == s.rolle).count() >= max { return Err(crate::tf!("ui.personal.mehr_als_sind_nicht_vorgesehen", max, rollen_de_t()[s.rolle as usize])); }
        if self.stab.iter().any(|x| x.name == s.name) { return Err(crate::inhalt::t("ui.personal.er_arbeitet_schon_fuer_uns").into()); }
        s.bis = self.abs() + 2 * crate::date::SEASON_DAYS;
        self.book(crate::inhalt::t("ui.personal.handgeld_personal"), -(s.gehalt * f("personal", "markt.handgeld_anteil") as f64));
        self.grund(&s.name.clone(), rollen_de_t()[s.rolle as usize], s.f, &crate::tf!("ui.personal.eingestellt_im_jahr", money(s.gehalt)));
        if s.rolle == 1 && !self.stab.iter().any(|x| x.rolle == 1) { self.co_name = s.name.clone(); }
        self.stab.push(s);
        Ok(())
    }
    pub fn stab_entlassen(&mut self, i: usize) {
        let s = self.stab.remove(i);
        let rest = s.bis.saturating_sub(self.abs()) as f64 / crate::date::SEASON_DAYS as f64;
        self.book(crate::inhalt::t("ui.personal.abfindungen"), -(s.gehalt * rest * f("personal", "markt.abfindung_anteil") as f64));
        if s.rolle == 1 { if let Some(n) = self.stab.iter().filter(|x| x.rolle == 1).max_by(|a, b| a.f.partial_cmp(&b.f).unwrap()).map(|x| x.name.clone()) { self.co_name = n; } }
        self.grund(&s.name, rollen_de_t()[s.rolle as usize], 0.0, crate::inhalt::t("aka.entlassen.ja"));
    }
    /// Training quality 0…100: trainers of an area weigh 1.0, general trainers 0.6, assistants 0.4 (personal.toml [wirkung]).
    pub fn trainingsqualitaet(&self) -> f32 {
        let (mut s, mut g) = (0.0, 0.0);
        for x in &self.stab {
            let w = match (x.rolle, x.bereich) { (0, 4) => f("personal", "wirkung.allgemein_gewicht"), (0, _) => f("personal", "wirkung.bereich_gewicht"), (1, _) => f("personal", "wirkung.co_gewicht"), _ => 0.0 };
            s += w * x.f;
            g += w;
        }
        if g == 0.0 { 50.0 } else { s / g }
    }
    /// Medical quality 0…100: doctor 0.35, physio 0.2 (rehab and sports science are not staffed here, they count 0).
    pub fn medizinqualitaet(&self) -> f32 {
        let best = |r: u8| self.stab.iter().filter(|x| x.rolle == r).map(|x| x.f).fold(0.0f32, f32::max);
        let w = |k: &str| f("personal", &format!("medizin.posten_gewicht.{k}"));
        // a post that is not staffed counts as quality 0 (personal.toml [medizin]); rehab and sports science are not offered here
        w(crate::id("arzt")) * best(2) + w(crate::id("physio")) * best(3)
    }
    /// Training level 1…10 for the world's development step, from the training quality.
    pub fn trainingsstufe(&self) -> f32 { 1.0 + 9.0 * self.trainingsqualitaet() / 100.0 }
    pub fn stab_monat(&mut self) {
        let g: f64 = self.stab.iter().map(|s| s.gehalt).sum();
        if g > 0.0 { self.book(crate::inhalt::t("ui.main.trainerteam"), -g / 12.0); }
        let abs = self.abs();
        let aus: Vec<String> = self.stab.iter().filter(|s| s.bis > 0 && abs >= s.bis).map(|s| s.name.clone()).collect();
        self.stab.retain(|s| s.bis == 0 || abs < s.bis);
        for n in aus { self.msg(1, &n, &crate::tf!("ui.personal.verlaesst_das_trainerteam", n), crate::inhalt::t("ui.game.sein_vertrag_ist_ausgelaufen"), Action::None); }
        if let Some(c) = self.stab.iter().filter(|x| x.rolle == 1).map(|x| x.f).reduce(f32::max) { self.mensch.co_staerke = c; }
    }

    // ------------------------------------------------------------ load (E1), phases and leaps (E3), retraining
    /// Daily load: b + unit load − 12 % · b (entwicklung.toml [belastung]); an individual programme adds 2.
    pub fn belastung_tag(&mut self) {
        let u = self.plan[crate::date::weekday(self.world.day)];
        let (abbau, einzel) = (f("entwicklung", "belastung.abbau"), f("entwicklung", "belastung.einzeltraining"));
        for p in self.squad() {
            let pl = &mut self.world.players[p];
            if !pl.ps.an { continue; }
            let last = einheit_last(u) + if pl.program > 0 { einzel } else { 0.0 };
            pl.ps.last = (pl.ps.last + last - abbau * pl.ps.last).max(0.0);
        }
    }
    /// Weekly phases of the own players (entwicklung3): Aufschwung, Formtief, Krise from the events of the last weeks,
    /// and the development leap of young regulars.
    /// Retraining to a new position: 120 days + 6 per year over 21, a distant position × 1.8 (§18); a position he already
    /// plays a little (B) goes 40 % faster. From 30 a player takes it as a step down (morale). Changeable, 24.11.
    pub fn umschulen(&mut self, p: usize, pos: &str) -> Result<u32, String> {
        let pl = &self.world.players[p];
        if pl.pos == pos { return Err(crate::inhalt::t("ui.personal.das_ist_schon_seine_position").into()); }
        let fern = part_of(&pl.pos) != part_of(pos);
        let teils = pl.pp.len() == 13 && pl.pp[crate::pesdb::pos_index(pos) as usize] >= 1;
        let tage = ((120.0 + 6.0 * (pl.age as f32 - 21.0).max(0.0)) * if fern { 1.8 } else { 1.0 } * if teils { 0.6 } else { 1.0 }) as u32;
        let abs = self.abs();
        let (n, alt) = (pl.name.clone(), pl.age >= 30);
        self.world.players[p].ps.umschulung = Some((pos.to_string(), abs + tage));
        self.world.players[p].ps.umschulung_start = Some(abs);
        self.grund(&n, crate::inhalt::t("menu.entwicklung.reiter.3"), tage as f32, &crate::tf!("ui.personal.lernt_die_position", pos_de(pos)));
        if alt { self.moral_wirkung(p, -4.0, 0, &crate::tf!("ui.personal.umschulung_alt", pos_de(pos))); }
        Ok(tage)
    }
    /// Stop a running retraining; what was learned up to halfway stays.
    pub fn umschulung_abbrechen(&mut self, p: usize) {
        if let Some((pos, _)) = self.world.players[p].ps.umschulung.take() {
            let n = self.world.players[p].name.clone();
            self.grund(&n, crate::inhalt::t("menu.entwicklung.reiter.3"), 0.0, &crate::tf!("ui.personal.umschulung_abgebrochen", pos_de(&pos)));
        }
    }
    pub fn umschulung_tag(&mut self) {
        let abs = self.abs();
        for p in self.squad() {
            let Some((pos, bis)) = self.world.players[p].ps.umschulung.clone() else { continue };
            // halfway: he can already play there now and then (B), and the coach says so once
            let pi = crate::pesdb::pos_index(&pos) as usize;
            let start = self.world.players[p].ps.umschulung_start.unwrap_or(abs);
            if self.world.players[p].ps.umschulung_start.is_none() { self.world.players[p].ps.umschulung_start = Some(abs); }
            if abs < bis && abs >= start + (bis.saturating_sub(start)) / 2 && self.world.players[p].pp.len() == 13 && self.world.players[p].pp[pi] == 0 {
                self.world.players[p].pp[pi] = 1;
                let n = self.world.players[p].name.clone();
                let k = self.vkey("ui.personal.umschulung_halb", &n);
                self.msg(1, &self.co_name.clone(), &crate::tf!("ui.personal.umschulung_halb_titel", n.clone()), &crate::tf!(&k, n, pos_de(&pos)), Action::None);
            }
            if abs < bis { continue; }
            let pl = &mut self.world.players[p];
            pl.ps.umschulung = None;
            pl.ps.umschulung_start = None;
            if pl.pp.len() == 13 { pl.pp[crate::pesdb::pos_index(&pos) as usize] = 2; }
            pl.pos = pos.clone();
            let n = pl.name.clone();
            self.msg(1, &self.co_name.clone(), &crate::tf!("ui.personal.umschulung_fertig", n), &crate::tf!("ui.personal.beherrscht_jetzt_die_position", n, pos_de(&pos)), Action::None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stab_und_belastung() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.stab_an();
        assert_eq!(c.stab.len(), 6);
        let q = c.trainingsqualitaet();
        assert!((20.0..=95.0).contains(&q));
        assert!((1.0..=10.0).contains(&c.trainingsstufe()));
        // load: a week of intensive units drives the load up, rest brings it down
        let p = c.squad()[0];
        c.plan = [5; 7];
        for _ in 0..7 { c.belastung_tag(); c.world.day += 1; }
        let hoch = c.world.players[p].ps.last;
        assert!(hoch > 40.0, "{hoch}");
        c.plan = [1; 7];
        for _ in 0..7 { c.belastung_tag(); c.world.day += 1; }
        assert!(c.world.players[p].ps.last < hoch);
        let ziel = if c.world.players[p].pos == "CF" { "GK" } else { "CF" };
        let tage = c.umschulen(p, ziel).unwrap();
        assert!(tage >= 60, "{tage}");
        // halfway he may play there now and then (B), at the end it is his position
        c.world.day += tage / 2 + 1;
        c.umschulung_tag();
        if c.world.players[p].pp.len() == 13 { assert!(c.world.players[p].pp[crate::pesdb::pos_index(ziel) as usize] >= 1); }
        c.world.day += tage;
        c.umschulung_tag();
        assert_eq!(c.world.players[p].pos, ziel);
        assert!(c.stab_bewerber().len() >= 5);
    }
}
