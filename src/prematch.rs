//! Before the match (pesm/core/spieltag/prematch.py, werte.py team_reaktion/auswertung, spieltag.toml [teamgespraech]):
//! the expectation (must-win, favourite, open, underdog, derby) from places, home, form, the season goal and absences,
//! with a recommended goal, tone and focus; the team talk moves every starter's morale, and after the final whistle the
//! goal set is judged (met: the team's morale up, missed: down).
use crate::game::*;
use crate::inhalt::t;
use crate::regeln::f;
use serde::{Deserialize, Serialize};

pub const ZIELE: [&str; 3] = [crate::id("sieg"), crate::id("punkt"), crate::id("einsatz")];
pub const TOENE: [&str; 5] = [crate::id("warm"), crate::id("sachlich"), crate::id("bestimmt"), crate::id("hart"), crate::id("ausweichend")];
pub const FOKUS: [&str; 2] = [crate::id("gegner"), crate::id("eigen")];
pub const EINORDNUNG: [&str; 5] = [crate::id("pflichtsieg"), crate::id("favorit"), crate::id("offen"), crate::id("aussenseiter"), crate::id("derby")];

/// The talk given before today's match: (abs day, goal, strength 0 favourite / 1 open / 2 underdog).
#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug, PartialEq)]
pub struct Vorgabe { pub tag: u32, pub ziel: u8, pub staerke: u8 }

fn k(x: &str) -> f32 { f("spieltag", &format!("teamgespraech.{x}")) } // roh

pub struct Erwartung { pub einordnung: usize, pub ziel: usize, pub ton: usize, pub fokus: usize, pub gruende: Vec<&'static str> }

impl Career {
    fn form_von(&self, club: usize) -> i32 {
        let mut spiele: Vec<(u32, i32)> = vec![];
        for l in &self.world.leagues { for x in l.fixtures.iter().filter(|x| x.hg >= 0 && (x.h == club || x.a == club)) { let (g, a) = if x.h == club { (x.hg, x.ag) } else { (x.ag, x.hg) }; spiele.push((x.day, (g - a).signum() as i32)); } }
        spiele.sort();
        spiele.iter().rev().take(5).map(|x| x.1).sum()
    }
    /// The expectation of today's match (prematch.py erwartung).
    pub fn spiel_erwartung(&self) -> Option<Erwartung> {
        let fx = self.fixture(self.today_match()?);
        let heim = fx.h == self.own;
        let geg = if heim { fx.a } else { fx.h };
        let mut gruende = vec![];
        let gleiche_liga = self.world.clubs[geg].league == self.league();
        let (pe, pg) = (self.world.rank_of(self.own) as i32, self.world.rank_of(geg) as i32);
        // the table says something only after a few matches (Tabellensperre); before that the squads' strength counts
        let gespielt = self.world.table(self.league()).iter().find(|r| r.club == self.own).map(|r| r.p).unwrap_or(0);
        let abstand = if gleiche_liga && gespielt >= 5 { pg - pe } else { ((self.club().level - self.world.clubs[geg].level) * 1.5).round() as i32 };
        gruende.push(if abstand > 2 { crate::id("abstand_vorn") } else if abstand < -2 { crate::id("abstand_hinten") } else { crate::id("abstand_eng") });
        let mut punkte = abstand + if heim { 3 } else { -3 };
        gruende.push(if heim { crate::id("heim") } else { crate::id("auswaerts") });
        let fd = self.form_von(self.own) - self.form_von(geg);
        punkte += fd;
        if fd >= 2 { gruende.push(crate::id("form_besser")); } else if fd <= -2 { gruende.push(crate::id("form_schlechter")); }
        let aus = self.squad().iter().filter(|&&p| !self.world.players[p].available()).count() as i32;
        if aus > 2 { punkte -= aus - 2; gruende.push(crate::id("ausfaelle")); }
        let anspruch = gleiche_liga && gespielt >= 5 && pe > self.club().goal_pos as i32;
        gruende.push(if anspruch { crate::id("ziel_verfehlt") } else { crate::id("ziel_erreicht") });
        let n = self.world.leagues[self.league()].teams.len() as i32;
        let schwach = gleiche_liga && gespielt >= 5 && pg >= n - 2;
        let derby = is_derby(&self.club().name, &self.world.clubs[geg].name).is_some();
        let e = if derby { 4 } else if punkte >= 8 || (schwach && heim && (anspruch || abstand >= 6)) { 0 } else if punkte >= 3 { 1 } else if punkte <= -5 { 3 } else { 2 };
        let (mut ziel, ton, fokus) = [(0, 2, 1), (0, 1, 1), (1, 1, 1), (1, 0, 0), (2, 2, 1)][e];
        if e == 2 && anspruch && heim { ziel = 0; }
        Some(Erwartung { einordnung: e, ziel, ton, fokus, gruende })
    }
    /// Strength for the talk's context: 0 favourite, 1 open, 2 underdog.
    fn staerke_von(e: usize) -> u8 { match e { 0 | 1 => 0, 3 => 2, _ => 1 } }
    /// The team talk before the match: every starter reacts by character, the tone's fit to the goal, focus and run.
    pub fn teamgespraech_vor(&mut self, ziel: usize, ton: usize, fokus: usize) -> Result<String, String> {
        let abs = self.abs();
        if self.ansprache_vor.is_some_and(|v| v.tag == abs) { return Err(t("pm.schon_gesprochen").into()); }
        let e = self.spiel_erwartung().ok_or_else(|| t("ui.pages.heute_ist_kein_spiel").to_string())?;
        let staerke = Self::staerke_von(e.einordnung);
        let serie: Vec<char> = self.results.iter().rev().take(k("serie_laenge") as usize).filter_map(|r| r.chars().next()).collect();
        let lang = serie.len() >= k("serie_laenge") as usize;
        let s = if lang && serie.iter().all(|&c| c == 'S') { 1 } else if lang && serie.iter().all(|&c| c == 'N') { -1 } else { 0 };
        let sq = if self.xi.is_empty() { self.squad().into_iter().take(11).collect() } else { self.xi.clone() };
        let mut summe = 0.0;
        let mut je = vec![];
        for &p in &sq {
            let pl = &self.world.players[p];
            let chs: Vec<f32> = (0..10).map(|i| crate::mensch::cw(pl, i)).collect();
            let mut d = crate::mensch::ton_aenderung(TOENE[ton], &chs, pl.morale, self.vertrauen(p)) * self.mgr_faktor(0);
            let ehrgeizig = chs[crate::mensch::EHRGEIZ] >= 60.0;
            if ziel == 0 { d += if staerke == 2 { k("ziel_unrealistisch") } else if staerke == 0 { k("ziel_realistisch") } else { 0.0 } + if ehrgeizig { 1.0 } else { 0.0 }; }
            if ziel == 1 { d += if staerke == 0 && ehrgeizig { k("ziel_unterfordert") } else if staerke == 2 { k("ziel_realistisch") } else { 0.0 }; }
            if (fokus == 0 && staerke == 2) || (fokus == 1 && staerke == 0) { d += k("fokus_passt"); }
            if s < 0 && ton <= 1 { d += k("serie_trost"); }
            if s < 0 && ton == 3 { d -= k("serie_trost"); }
            if s > 0 && (ton == 2 || ton == 3) { d += k("serie_euphorie"); }
            summe += d;
            je.push((p, d));
        }
        for (p, d) in je { self.moral_wirkung(p, d / 2.0, 0, t("pm.grund")); }
        self.ansprache_vor = Some(Vorgabe { tag: abs, ziel: ziel as u8, staerke });
        let mittel = if sq.is_empty() { 0.0 } else { summe / sq.len() as f32 };
        Ok(t(if mittel >= 2.0 { "ui.rest3.die_ansprache_kam_an" } else if mittel <= -2.0 { "ui.rest3.die_ansprache_ging_daneben" } else { "ui.rest3.die_ansprache_verpuffte" }).into())
    }
    /// After the own match: was the goal of the talk met? The team's morale follows (werte.py auswertung).
    pub fn teamgespraech_auswertung(&mut self, gf: u8, ga: u8) {
        let Some(v) = self.ansprache_vor.take() else { return };
        if v.tag != self.abs() || v.ziel == 2 { return; }
        let ok = if v.ziel == 0 { gf > ga } else { gf >= ga };
        let mut d = if ok { k("auswertung_erfuellt") } else { k("auswertung_verfehlt") };
        if !ok && v.staerke == 2 { d = (d / 2.0).trunc(); } else if !ok && v.staerke == 0 { d -= k("auswertung_gegner_schwach") - 1.0; }
        let grund = t(if ok { "pm.erfuellt" } else { "pm.verfehlt" });
        for p in self.xi.clone() { self.moral_wirkung(p, d, 0, grund); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ansprache_vor_dem_spiel() {
        let w = crate::world::World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        while c.today_match().is_none() { c.end_day().unwrap(); }
        let e = c.spiel_erwartung().unwrap();
        assert!(e.einordnung < 5 && !e.gruende.is_empty());
        c.auto_lineup();
        let m0: f32 = c.xi.iter().map(|&p| c.world.players[p].morale).sum();
        c.teamgespraech_vor(e.ziel, 0, e.fokus).unwrap();
        assert!(c.teamgespraech_vor(e.ziel, 0, e.fokus).is_err(), "one talk per match");
        let _ = m0;
        c.teamgespraech_auswertung(2, 0);
        assert!(c.ansprache_vor.is_none());
    }
}
