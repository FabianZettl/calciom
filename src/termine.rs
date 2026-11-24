//! Agreed moves of the future (pesm/fl/sim/termine.py): a pre-contract with a player whose contract ends this season
//! (from 1 January, outside the window), carried out on the deadline at the season's change; other clubs sign our
//! expiring players the same way. Each entry has its reason.
use crate::game::*;
use crate::inhalt::t;
use crate::mensch::zufall;
use crate::world::*;
use serde::{Deserialize, Serialize};

/// 1 January in days from 1 August (the half of the season from which pre-contracts are allowed).
pub const AB_TAG: u32 = 153;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Vorvertrag { pub pid: usize, pub von: usize, pub nach: usize, pub wage: f64, pub years: u32, pub tag: u32 }

impl Career {
    /// May we offer `p` a pre-contract? His contract ends this season, it is 1 January or later, he plays elsewhere.
    pub fn vorvertrag_moeglich(&self, p: usize) -> bool {
        let pl = &self.world.players[p];
        pl.team >= 0 && pl.team != self.own as i32 && pl.youth == 0 && pl.contract_end <= self.world.season && self.world.day >= AB_TAG
            && !self.vorvertraege.iter().any(|v| v.pid == p)
    }
    /// Opens the talks with the player directly (no fee, the club has no say); the result is a pre-contract.
    pub fn vorvertrag_oeffnen(&mut self, p: usize) -> Result<usize, String> {
        if !self.vorvertrag_moeglich(p) { return Err(t("vv.nicht_moeglich").into()); }
        if self.negs.iter().any(|n| n.pid == p && n.stage < 2) { return Err(t("ui.game.es_laeuft_schon_eine_verhandlung").into()); }
        if self.player_rep(p) - self.club().rep > 45.0 { return Err(crate::tf!("ui.game.winkt_ab_dein_verein_ist", self.world.players[p].name.clone())); }
        // like a free agent: straight to the player's phase
        let team = self.world.players[p].team;
        self.world.players[p].team = -1;
        let r = self.open_neg(p);
        self.world.players[p].team = team;
        let i = r?;
        self.negs[i].vorvertrag = true;
        self.negs[i].text = crate::tf!("vv.gespraech", self.world.players[p].name.clone());
        Ok(i)
    }
    /// The pre-contract is signed: the move happens at the season's change.
    pub fn vorvertrag_abschliessen(&mut self, n: &Neg) {
        let p = n.pid;
        let von = self.world.players[p].team.max(0) as usize;
        self.vorvertraege.push(Vorvertrag { pid: p, von, nach: self.own, wage: n.wage, years: n.years, tag: self.abs() });
        let (nm, vn) = (self.world.players[p].name.clone(), self.world.clubs[von].name.clone());
        self.msg(2, t("ui.game.transferabteilung"), &crate::tf!("vv.titel", nm.clone()), &crate::tf!("vv.text", nm.clone(), vn.clone(), money(n.wage), n.years), Action::None);
        let own = self.club().name.clone();
        self.news(&crate::tf!("vv.news", nm, own), &crate::tf!("vv.news_text", vn));
    }
    /// Weekly from 1 January: other clubs sign our expiring players to pre-contracts (better players are wanted more;
    /// a player who wants to stay or is in renewal talks does not sign).
    pub fn vorvertraege_woche(&mut self) {
        if self.world.day < AB_TAG { return; }
        let season = self.world.season;
        let woche = (self.abs() / 7).to_string();
        // a warning first (FC-26 wish): from now on other clubs may sign him for the summer; extend or let it happen
        let abs = self.abs();
        for p in self.squad() {
            let pl = &self.world.players[p];
            if pl.contract_end > season || pl.youth > 0 || pl.leihe.is_some() || self.vorvertraege.iter().any(|v| v.pid == p) { continue; }
            let id = pl.id;
            if self.mensch.story_erzaehlt.iter().any(|x| x.0 == "warn_vv" && x.2 == id && x.1 / crate::date::SEASON_DAYS == season) { continue; } // roh
            self.mensch.story_erzaehlt.push(("warn_vv".into(), abs, id)); // roh
            let nm = pl.name.clone();
            let opts = vec![("verlaengern".to_string(), t("vv.warn.verlaengern").to_string()), ("lassen".to_string(), t("vv.warn.lassen").to_string())]; // roh
            self.msg(1, t("ui.game.transferabteilung"), &crate::tf!("vv.warn.titel", nm.clone()), &crate::tf!("vv.warn.text", nm), Action::Wahl { art: "vvwarn:x".into(), bezug: vec![p], optionen: opts }); // roh
        }
        if self.ohne_ki_vorvertraege { return; }
        for p in self.squad() {
            let pl = &self.world.players[p];
            if pl.contract_end > season || pl.youth > 0 || pl.leihe.is_some() || self.vorvertraege.iter().any(|v| v.pid == p) || self.negs.iter().any(|n| n.pid == p && n.stage < 2) { continue; }
            let chance = 0.04 + 0.12 * ((pl.ovr - self.club().level + 8.0) / 16.0).clamp(0.0, 1.0) + if pl.ps.wunsch { 0.1 } else { 0.0 };
            if zufall(&[crate::id("vorvertrag"), &pl.id.to_string(), &woche]) as f32 >= chance { continue; }
            let ovr = pl.ovr;
            let cands: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.erste_von(c).is_none() && (self.world.clubs[c].level - ovr).abs() <= 5.0).collect();
            if cands.is_empty() { continue; }
            let nach = cands[(zufall(&[crate::id("vorvertrag_verein"), &pl.id.to_string(), &woche]) * cands.len() as f32) as usize % cands.len()];
            let lv = self.world.leagues[self.world.clubs[nach].league].level;
            let wage = fair_wage(&self.world.players[p], lv);
            self.vorvertraege.push(Vorvertrag { pid: p, von: self.own, nach, wage, years: contract_years(self.world.players[p].age), tag: self.abs() });
            let (nm, cn) = (self.world.players[p].name.clone(), self.world.clubs[nach].name.clone());
            self.msg(1, t("ui.game.transferabteilung"), &crate::tf!("vv.weg_titel", nm.clone()), &crate::tf!("vv.weg_text", nm.clone(), cn.clone()), Action::None);
            self.news(&crate::tf!("vv.news", nm, cn), &crate::tf!("vv.news_text", self.club().name.clone()));
            self.moral_wirkung(p, -3.0, 0, t("vv.weg_grund"));
        }
    }
    /// At the season's change, before contracts run out: every pre-contract is carried out (if he is still where he was).
    pub fn vorvertraege_vollziehen(&mut self) {
        let alle = std::mem::take(&mut self.vorvertraege);
        let season = self.world.season;
        for v in alle {
            let pl = &self.world.players[v.pid];
            if pl.team != v.von as i32 || pl.team == -2 { continue; }
            self.world.clubs[v.von].squad.retain(|&x| x != v.pid);
            self.world.clubs[v.nach].squad.push(v.pid);
            self.xi.retain(|&x| x != v.pid);
            let abs = self.abs();
            let pl = &mut self.world.players[v.pid];
            pl.team = v.nach as i32;
            pl.wage = v.wage;
            // the new contract counts from the coming season
            pl.contract_end = season + v.years;
            pl.joined = abs;
            pl.listed = false;
            let nm = pl.name.clone();
            if v.nach == self.own {
                self.world.players[v.pid].know = 1.0;
                let vn = self.world.clubs[v.von].name.clone();
                self.msg(2, t("ui.game.transferabteilung"), &crate::tf!("vv.da_titel", nm.clone()), &crate::tf!("vv.da_text", nm, vn.clone()), Action::None);
                self.feier_neuzugang(v.pid, &vn, 0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vorvertrag_vom_januar_bis_zum_sommer() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        let andere = (0..c.world.clubs.len()).find(|&x| x != c.own && !c.world.clubs[x].squad.is_empty()).unwrap();
        let p = c.world.clubs[andere].squad[0];
        c.world.players[p].contract_end = c.world.season;
        assert!(!c.vorvertrag_moeglich(p), "not before 1 January");
        c.world.day = AB_TAG;
        assert!(c.vorvertrag_moeglich(p));
        let n = Neg { pid: p, renew: false, stage: 1, round: 0, ask: 0.0, fee: 0.0, counter: 0.0, wage: 1e6, need: 1e6, years: 3, state: NegState::Counter, reply_day: 0, ends: 0, text: String::new(), ph: None, ton: 0, handlung: 0, letzt: false, argument: 0, argumente: vec![], gefragt: false, tausch: false, haken: 0, haken_bis: 0, abschlag: 0.0, med: false, gate: false, verkauf: false, kaeufer: 0, gesendet: String::new(), kl: Default::default(), zeit: 0, vorvertrag: true, liegt_bis: 0, verlauf: vec![] };
        c.vorvertrag_abschliessen(&n);
        assert_eq!(c.world.players[p].team, andere as i32, "he stays until the summer");
        c.vorvertraege_vollziehen();
        assert_eq!(c.world.players[p].team, c.own as i32);
        assert!(c.world.clubs[c.own].squad.contains(&p) && !c.world.clubs[andere].squad.contains(&p));
    }
}
