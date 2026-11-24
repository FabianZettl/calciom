//! The board escalates step by step (own addition between warning and ultimatum): when confidence falls below the crisis
//! line, the president calls a crisis meeting with three ways out (a plan with a points target in the next four league
//! matches, demanding backing, admitting mistakes); the press smells blood and names successors when an ultimatum is set.
//! A change of club gets its story: the farewell, the first days, the contracts waiting on the desk.
use crate::game::*;
use crate::inhalt::t;
use crate::mensch::zufall;
use serde::{Deserialize, Serialize};

const KRISE_UNTER: f32 = 32.0;
const PLAN_SPIELE: usize = 4;
const PLAN_PUNKTE: i32 = 7;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Krise { pub saison: u32, pub gehalten: bool, pub plan: Option<(usize, i32)> }

impl Career {
    /// After each own match: the crisis meeting when confidence is low, the plan's check after four league matches.
    pub fn krise_pruefen(&mut self) {
        if self.fired { return; }
        let season = self.world.season;
        if let Some((start, ziel)) = self.krise.plan {
            if self.results.len() >= start + PLAN_SPIELE {
                let punkte: i32 = self.results[start..].iter().take(PLAN_SPIELE).map(|r| if r.starts_with('S') { 3 } else if r.starts_with('U') { 1 } else { 0 }).sum();
                self.krise.plan = None;
                let ok = punkte >= ziel;
                self.klima(0.0, 0.0, if ok { 6.0 } else { -6.0 }, t("kr2.plan"));
                self.msg(5, t("meldung.absender.praesident"), t(if ok { "kr2.plan_ok_titel" } else { "kr2.plan_nein_titel" }), &crate::tf!(if ok { "kr2.plan_ok" } else { "kr2.plan_nein" }, punkte, ziel), Action::None);
            }
        }
        if self.mensch.vorstand >= KRISE_UNTER || (self.krise.gehalten && self.krise.saison == season) || self.ultimatum.is_some() || self.abs() < self.mgr_since + 45 || self.abs() < self.ruhe_bis { return; }
        self.krise = Krise { saison: season, gehalten: true, plan: None };
        let opts = vec![(crate::id("plan").to_string(), crate::tf!("kr2.opt.plan", PLAN_PUNKTE, PLAN_SPIELE)), (crate::id("rueckhalt").to_string(), t("kr2.opt.rueckhalt").to_string()), (crate::id("fehler").to_string(), t("kr2.opt.fehler").to_string())];
        let (rang, ziel) = (self.world.rank_of(self.own), self.club().goal_pos);
        self.msg(5, t("meldung.absender.praesident"), t("kr2.titel"), &crate::tf!("kr2.text", rang, ziel), Action::Wahl { art: crate::id("krise").into(), bezug: vec![], optionen: opts });
        let cn = self.club().name.clone();
        self.news(&crate::tf!("kr2.news", cn), t("kr2.news_text"));
    }
    pub fn krise_antwort(&mut self, key: &str) -> String {
        match key {
            "plan" => { self.krise.plan = Some((self.results.len(), PLAN_PUNKTE)); self.klima(0.0, 0.0, 3.0, t("kr2.plan")); crate::tf!("kr2.a_plan", PLAN_PUNKTE, PLAN_SPIELE) }
            "rueckhalt" => {
                let chance = (self.mgr_rep / 100.0 * 0.8 + 0.1).clamp(0.1, 0.85);
                if zufall(&[crate::id("rueckhalt"), &self.abs().to_string()]) < chance { self.klima(0.0, 1.0, 5.0, t("kr2.opt.rueckhalt")); t("kr2.a_rueckhalt_ja").into() }
                else { self.klima(0.0, -3.0, -5.0, t("kr2.opt.rueckhalt")); t("kr2.a_rueckhalt_nein").into() }
            }
            _ => { self.klima(1.0, 2.0, 2.0, t("kr2.opt.fehler")); t("kr2.a_fehler").into() }
        }
    }
    /// When the board sets an ultimatum, the papers name possible successors (free coaches of the coaches' bank).
    pub fn nachfolger_geruechte(&mut self) {
        let rep = self.club().rep;
        let mut namen: Vec<String> = self.world.trainer_pool.iter().map(|tr| tr.name.clone()).collect();
        if namen.is_empty() { return; }
        let s = (zufall(&[crate::id("nachfolger"), &self.abs().to_string()]) * namen.len() as f32) as usize % namen.len();
        namen.rotate_left(s);
        namen.truncate(2);
        let cn = self.club().name.clone();
        let _ = rep;
        self.news(&crate::tf!("kr2.nachfolger", cn), &crate::tf!("kr2.nachfolger_text", namen.join(crate::id(" und "))));
    }
    /// The story of a change of club: farewell from the old, the first days, the contracts on the desk.
    pub fn wechsel_geschichte(&mut self, alt: usize, war_entlassen: bool) {
        let (an, nn, coach) = (self.world.clubs[alt].name.clone(), self.club().name.clone(), self.coach.clone());
        if !war_entlassen {
            self.news(&crate::tf!("kr2.abschied", coach.clone(), an.clone()), &crate::tf!("kr2.abschied_text", an.clone(), nn.clone()));
            self.msg(4, t("gw.von.fans"), &crate::tf!("kr2.abschied_fans_titel", an.clone()), &crate::tf!("kr2.abschied_fans", an), Action::None);
        }
        let op = self.co_opinion();
        let co = self.co_name.clone();
        self.msg(1, &co, t("kr2.erste_tage"), &op, Action::None);
        let n = self.squad().iter().filter(|&&p| self.world.players[p].contract_end <= self.world.season).count();
        if n > 0 { self.msg(2, t("ui.game.transferabteilung"), t("kr2.vertraege_titel"), &crate::tf!("kr2.vertraege", n), Action::None); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn krisensitzung_und_plan() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.mgr_since = 0;
        c.world.day = 100;
        c.mensch.vorstand = 25.0;
        c.krise_pruefen();
        let i = c.inbox.iter().position(|m| matches!(&m.action, Action::Wahl { art, .. } if art == "krise")).expect("crisis meeting");
        c.answer(i, true, 0);
        assert!(c.krise.plan.is_some());
        for _ in 0..4 { c.results.push("S x".into()); }
        let v = c.mensch.vorstand;
        c.krise_pruefen();
        assert!(c.krise.plan.is_none() && c.mensch.vorstand > v, "plan met");
        c.krise_pruefen();
        assert_eq!(c.inbox.iter().filter(|m| matches!(&m.action, Action::Wahl { art, .. } if art == "krise")).count(), 1, "once a season");
    }
    #[test]
    fn rueckgaengig_stellt_genau_her() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let vorher = serde_json::to_string(&c).unwrap();
        c.undo_merken();
        for _ in 0..1 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } c.end_day().unwrap(); }
        assert!(c.rueckgaengig());
        assert_eq!(serde_json::to_string(&c).unwrap(), vorher, "the state is exactly the one before");
        assert!(!c.rueckgaengig(), "one step back only");
    }
}
