//! The coach the person creates at the start (Fabian 23.11., like FM): birth date, nationality, previous knowledge, own
//! playing career and preferred style. Knowledge and playing career count as experience and reputation, so a former pro
//! or an experienced coach starts with a higher licence; the style sets the first tactic. A career can also start
//! without a club: offers come, limited to chosen leagues, the leagues of the own country or the whole database.
use crate::game::*;
use crate::inhalt::t;
use crate::world::money;
use serde::{Deserialize, Serialize};

/// Experience months and reputation points of each level of previous knowledge and of the own playing career.
pub const VORWISSEN_MONATE: [f32; 5] = [0.0, 6.0, 18.0, 30.0, 48.0];
pub const VORWISSEN_RUF: [f32; 5] = [0.0, 3.0, 8.0, 14.0, 22.0];
pub const KARRIERE_RUF: [f32; 5] = [0.0, 2.0, 5.0, 10.0, 18.0];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Trainerprofil { pub tag: u8, pub monat: u8, pub jahr: u16, pub nation: String, pub vorwissen: u8, pub karriere: u8, pub stil: u8 }
impl Default for Trainerprofil { fn default() -> Self { Trainerprofil { tag: 1, monat: 1, jahr: 1985, nation: String::new(), vorwissen: 0, karriere: 0, stil: 0 } } }

impl Trainerprofil {
    pub fn monate(&self) -> f32 { VORWISSEN_MONATE[self.vorwissen.min(4) as usize] }
    pub fn ruf(&self) -> f32 { VORWISSEN_RUF[self.vorwissen.min(4) as usize] + KARRIERE_RUF[self.karriere.min(4) as usize] }
    pub fn alter(&self) -> u32 { (crate::date::START_YEAR as u32).saturating_sub(self.jahr as u32) }
    /// The licence the background gives at the start (laufbahn.toml [lizenz], with the base reputation of a new coach).
    pub fn start_lizenz(&self) -> &'static str {
        let ruf = crate::regeln::f("laufbahn", "ruf.basis") + self.ruf();
        crate::mensch::LIZENZEN.iter().copied().filter(|s| { let a = crate::mensch::arr("laufbahn", &format!("lizenz.{s}")); ruf >= a[0].as_f64().unwrap_or(0.0) as f32 && self.monate() >= a[1].as_f64().unwrap_or(0.0) as f32 }).last().unwrap_or(crate::id("c"))
    }
}

impl Career {
    /// Takes the created coach over: reputation from the background, the first tactic from the style.
    pub fn trainer_uebernehmen(&mut self, p: Trainerprofil) {
        self.mgr_rep = (self.mgr_rep + p.ruf()).min(90.0);
        let tk = &mut self.tactic;
        match p.stil {
            1 => { tk.haltung = 0; tk.pressing = 0; tk.linie = 4; } // counter
            2 => { tk.pressing = 2; tk.linie = 7; tk.haltung = 2; } // pressing
            3 => { tk.haltung = 0; tk.kompakt = 8; tk.linie = 3; } // compact
            4 => { tk.haltung = 2; tk.breite = 2; } // wing play
            _ => { tk.haltung = 1; tk.aufbau = 0; } // possession
        }
        self.trainerprofil = p;
    }
    /// The leagues offers may come from (empty: all).
    pub fn angebot_erlaubt(&self, c: usize) -> bool { self.angebot_ligen.is_empty() || self.angebot_ligen.contains(&self.world.clubs[c].league) }
    /// Without a club: offers from the allowed leagues, as far as reputation and licence reach (the weakest clubs first if
    /// nothing fits, so a newcomer always gets a chance).
    pub fn angebote_ohne_verein(&mut self) {
        let grenze = self.lizenz_grenze();
        let rep = self.mgr_rep;
        let mut c: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.erste_von(c).is_none() && self.angebot_erlaubt(c) && self.world.clubs[c].rep <= rep + 5.0 && self.world.clubs[c].rep / 100.0 <= grenze && !self.mensch.job_sperre.iter().any(|x| x.0 == c)).collect();
        if c.is_empty() {
            c = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.erste_von(c).is_none() && self.angebot_erlaubt(c)).collect();
            c.sort_by(|a, b| self.world.clubs[*a].rep.partial_cmp(&self.world.clubs[*b].rep).unwrap());
            c.truncate(8);
        }
        for _ in 0..3 {
            if c.is_empty() { break; }
            let i = self.world.rng.int(0, c.len() as i32 - 1) as usize;
            let club = c.swap_remove(i);
            let wage = (self.world.revenue(club) * 0.004).max(120000.0);
            let n = self.world.clubs[club].name.clone();
            self.msg(5, &n, &crate::tf!("transfer.angebot_von", n.clone()), &crate::tf!("ui.game.bietet_dir_den_trainerposten_an", n, money(wage)), Action::JobOffer { club, wage });
        }
    }
    /// A career that starts without a club: the coach waits for offers.
    pub fn ohne_verein_starten(&mut self) {
        self.fired = true;
        self.inbox.clear();
        self.history.clear();
        self.msg(5, t("tp.absender"), t("tp.ohne_titel"), t("tp.ohne_text"), Action::None);
        self.angebote_ohne_verein();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hintergrund_gibt_lizenz_und_angebote() {
        let p = Trainerprofil { vorwissen: 4, karriere: 4, ..Default::default() };
        assert_ne!(p.start_lizenz(), "c", "an experienced former pro starts higher");
        assert_eq!(Trainerprofil::default().start_lizenz(), "c");
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let liga = c.world.clubs[5].league;
        c.angebot_ligen = vec![liga];
        c.trainer_uebernehmen(p);
        c.ohne_verein_starten();
        let angebote: Vec<usize> = c.inbox.iter().filter_map(|m| if let Action::JobOffer { club, .. } = m.action { Some(club) } else { None }).collect();
        assert!(!angebote.is_empty() && angebote.iter().all(|&k| c.world.clubs[k].league == liga), "offers only from the chosen league");
        assert!(c.erfahrung_monate() >= 48.0);
    }
}
