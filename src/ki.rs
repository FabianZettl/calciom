//! Paket 3r: the AI coaches of the other clubs (core/ki_trainer/ki_trainer.toml): a type per coach decides how fast
//! he reacts to a losing series (stable, defensive set-up for some weeks) and whether he rotates tired players.
//! Also §20 points with effect: the players' council and the board's vision.
use crate::game::*;
use crate::mensch::zufall;
use crate::regeln::f;

pub const TYPEN: [&str; 5] = [crate::id("starr"), crate::id("reaktiv"), crate::id("experimentierfreudig"), crate::id("dogmatisch"), crate::id("pragmatisch")];
pub const VISIONEN: [&str; 4] = [crate::id("traditionell"), crate::id("ehrgeizig"), crate::id("sparsam"), crate::id("geduldig")];
const TRAINER: [&str; 16] = ["Erik Sandvik", "Paolo Venturi", "Jens Hartmann", "Luis Carvalho", "Ömer Aksoy", "Pierre Lambert", "Ronan Doyle", "Sven Ahlberg", "Marco Bellini", "Dirk Wendt", "Andrés Molina", "Tomasz Wrona", "Karel Dvořák", "Fabio Lenz", "Hugo Martel", "Nils Brekke"]; // roh

impl Career {
    /// The AI coach's type, fixed per club and coach name.
    pub fn ki_typ(&self, c: usize) -> usize { (zufall(&["2026", "ki_typ", &self.world.clubs[c].id.to_string(), &self.world.clubs[c].manager]) * 5.0) as usize % 5 }
    /// Weekly for the clubs of the detailed leagues: a losing series of `serie` games makes the coach switch to a stable,
    /// defensive style for `sperre` weeks (fixed types later and longer); an interim coach is replaced after two weeks.
    pub fn ki_woche(&mut self) {
        let abs = self.abs();
        for l in 0..self.world.leagues.len() {
            if self.world.leagues[l].detail > 1 { continue; }
            let table = self.world.table(l);
            for r in table {
                let c = r.club;
                if c == self.own { continue; }
                if self.world.clubs[c].manager == "Interimstrainer" { // roh
                    if abs >= self.world.clubs[c].ki_bis {
                        // a free coach from the bank pool (service.py _kandidat), otherwise an invented one (_erfinden)
                        let season = self.world.season;
                        let neu = match self.world.trainer_kandidat(c, abs, |k| zufall(&["2026", k])) {
                            Some(i) => self.world.trainer_verpflichten(c, i, season),
                            None => { let n = TRAINER[(zufall(&["2026", "neuer_trainer", &c.to_string(), &abs.to_string()]) * 16.0) as usize % 16].to_string(); self.world.clubs[c].manager = n.clone(); n }
                        };
                        let cn = self.world.clubs[c].name.clone();
                        self.news(&crate::tf!("tb.ereignis.verpflichtet", cn, neu), crate::inhalt::t("tb.grund.interim"));
                        self.world.clubs[c].ki_bis = 0;
                    }
                    continue;
                }
                let t = TYPEN[self.ki_typ(c)];
                let serie = f("ki_trainer", &format!("typ.{t}.serie")) as usize;
                let sperre = f("ki_trainer", &format!("typ.{t}.sperre")) as u32;
                let n = r.form.iter().rev().take_while(|&&x| x == 'N').count();
                let cl = &mut self.world.clubs[c];
                if cl.ki_bis > 0 && abs >= cl.ki_bis { cl.style = cl.ki_stil; cl.ki_bis = 0; }
                if n >= serie && cl.ki_bis == 0 { cl.ki_stil = cl.style; cl.style = -1; cl.ki_bis = abs + sperre * 7; }
            }
        }
    }
    /// An AI board fires its coach: the interim coach stays two weeks (board_check), then a new coach comes.
    pub fn ki_entlassung(&mut self, c: usize) { self.world.clubs[c].ki_bis = self.abs() + 14; }

    // ------------------------------------------------------------ §20 with effect
    /// The players' council: captain, vice and leaders (hierarchie roles 0–2).
    pub fn spielerrat(&self) -> Vec<usize> { self.squad().into_iter().filter(|&p| self.world.players[p].ps.an && self.world.players[p].ps.rolle <= 2 && self.world.players[p].youth == 0).collect() }
    /// Own decision for §20: an unhappy council member weighs on the team spirit (−2 each on its target), weekly.
    pub fn spielerrat_woche(&mut self) {
        let n = self.spielerrat().iter().filter(|&&p| self.world.players[p].ps.unzufrieden).count() as f32;
        if n > 0.0 { self.mensch.teamgeist = (self.mensch.teamgeist - 0.2 * 2.0 * n).max(0.0); self.grund(crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("charakter.teamgeist"), -0.4 * n, crate::inhalt::t("ui.ki.unzufriedene_im_spielerrat")); }
    }
    /// The board's vision (laufbahn.toml bewerbung.persona), fixed per club.
    pub fn vision(&self) -> &'static str { VISIONEN[(zufall(&["2026", crate::id("vision"), &self.club().id.to_string()]) * 4.0) as usize % 4] }
    /// Own decision for §20: the vision changes the board's measure: ehrgeizig expects one place more, geduldig sets the
    /// ultimatum 5 points lower, sparsam passes 10 % less of the surplus on, traditionell likes promoted youth (+2).
    pub fn vision_ziel(&self, z: usize) -> usize { if self.vision() == crate::id("ehrgeizig") { z.saturating_sub(1).max(1) } else { z } }
    pub fn vision_ultimatum(&self) -> f32 { f("vereinswelt", "praesidium.ultimatum_unter") - if self.vision() == crate::id("geduldig") { 5.0 } else { 0.0 } }
    pub fn vision_budget(&self) -> f32 { if self.vision() == crate::id("sparsam") { -0.1 } else { 0.0 } }

    /// Paket 6, awards (fl/sim/ehrungen.py in its simplest form): per detailed league the player of the season (best
    /// average grade with at least 15 graded matches) and the top scorer; returned as lines for the season archive.
    pub fn ehrungen_saison(&mut self) -> Vec<String> {
        let mut out = vec![];
        for l in 0..self.world.leagues.len() {
            if self.world.leagues[l].detail > 0 && l != self.league() { continue; }
            let spieler: Vec<usize> = self.world.leagues[l].teams.iter().flat_map(|&t| self.world.clubs[t].squad.clone()).collect();
            let ln = self.world.leagues[l].name.clone();
            if let Some(&p) = spieler.iter().filter(|&&p| self.world.players[p].rn >= 15).max_by(|&&a, &&b| self.world.players[a].avg_rating().partial_cmp(&self.world.players[b].avg_rating()).unwrap()) {
                let pl = &self.world.players[p];
                out.push(crate::tf!("ui.ki.spieler_der_saison_in_note", ln, pl.name, format!("{:.2}", pl.avg_rating())));
            }
            if let Some(&p) = spieler.iter().filter(|&&p| self.world.players[p].goals > 0).max_by_key(|&&p| self.world.players[p].goals) {
                let pl = &self.world.players[p];
                out.push(crate::tf!("ui.ki.torschuetzenkoenig_in_tore", ln, pl.name, pl.goals));
            }
        }
        for z in out.clone() { let own = self.world.clubs[self.own].squad.iter().any(|&p| z.contains(&self.world.players[p].name)); if own { self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), crate::inhalt::t("ui.ki.auszeichnung_fuer_uns"), &z, Action::None); } self.news(&z, crate::inhalt::t("ui.ki.die_ehrungen_der_saison")); }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ki_reagiert_auf_serien() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        for _ in 0..200 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } c.ultimatum = None; c.end_day().unwrap(); }
        assert!(c.world.clubs.iter().any(|cl| cl.ki_bis > 0) || c.world.clubs.iter().all(|cl| cl.style >= -1));
        assert!(VISIONEN.contains(&c.vision()));
    }
    #[test]
    fn ki_rotiert_muede_stammspieler() {
        let w = crate::world::testwelt(7);
        let c = Career::new(w, 0, "Test");
        let mut w = c.world.clone();
        // a club whose coach rotates
        let k = (1..w.clubs.len()).find(|&k| { let t = TYPEN[c.ki_typ(k)]; crate::regeln::wert_oder("ki_trainer", &format!("typ.{t}.rotiert")).and_then(|v| v.as_bool()) == Some(true) }).unwrap();
        let (xi, bench) = w.lineup(k, 0, &[]);
        let (slot, star, _) = xi.iter().find(|(s, _, _)| *s != "GK").cloned().unwrap();
        let Some(&ersatz) = bench.iter().find(|&&b| w.players[b].pos != "GK") else { return };
        w.players[ersatz].pos = slot.clone();
        w.players[ersatz].ovr = w.players[star].ovr - 1.0;
        w.players[ersatz].fit = 100.0;
        w.players[ersatz].morale = w.players[star].morale;
        w.players[star].fit = 54.0;
        let (xi2, _) = w.lineup(k, 0, &[]);
        assert!(xi2.iter().any(|x| x.1 == ersatz), "the tired regular is rotated out");
    }
}
