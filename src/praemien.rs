//! Prize money (pesm/core/wirtschaft/praemien.py, praemien.toml): the domestic cup per round played (doubling each round),
//! promotion bonus and relegation parachute, and the league's TV money by final place on top of the monthly money.
//! Amounts are shares of the reference (`bezug_mio` = average income of a top league) times the club's reference: the
//! average income of the top league of its country in the club's money scale.
use crate::game::*;
use crate::inhalt::t;
use crate::regeln::f;
use crate::world::*;

fn p(k: &str) -> f64 { f("praemien", k) as f64 }

impl Career {
    /// Average income of the top league of the own country, in the club's money scale.
    pub fn praemie_referenz(&self) -> f64 {
        let land = self.world.leagues[self.league()].land.clone();
        let oben: Vec<usize> = self.world.leagues.iter().filter(|l| l.land == land && l.stufe <= 1).flat_map(|l| l.teams.iter().copied()).collect();
        let schnitt = if oben.is_empty() { self.world.revenue(self.own) } else { oben.iter().map(|&c| self.world.revenue(c)).sum::<f64>() / oben.len() as f64 };
        let massstab = self.umsatz_prognose() / self.world.revenue(self.own).max(1.0);
        schnitt * massstab
    }
    fn praemie_buchen(&mut self, wb: &str, grund: &str, betrag: f64, zusatz: String) {
        let betrag = (betrag / 1000.0).round() * 1000.0;
        if betrag <= 0.0 { return; }
        self.book(t("vw.art.praemie_wettbewerb"), betrag);
        let w = t(&format!("vw.praemie.wb.{wb}")); // roh
        self.msg(3, t("vw.figur.finanz"), &crate::tf!("vw.praemie.betreff", w), &crate::tf!(&format!("vw.praemie.text.{grund}"), w, money(betrag), zusatz), Action::None); // roh
    }
    /// After an own match in a domestic cup: the prize of the round (the number of own matches played in it).
    pub fn pokal_praemie(&mut self, ci: usize) {
        let own = self.own;
        let cup = &self.world.cups[ci];
        if cup.kind != 0 { return; }
        let runde = cup.fx.iter().filter(|x| (x.f.h == own || x.f.a == own) && x.f.hg >= 0).count() as i32;
        if runde == 0 { return; }
        let mio = p("pokal.basis_mio") * p("pokal.faktor_je_runde").powi(runde.min(p("pokal.runden_max") as i32) - 1);
        let betrag = mio / p("pokal.bezug_mio") * self.praemie_referenz();
        self.praemie_buchen(crate::id("pokal"), crate::id("pokal"), betrag, runde.to_string());
    }
    /// Promotion bonus or relegation parachute when the own league's tier changed (1 = top).
    pub fn auf_abstieg_geld(&mut self, alt: u8, neu: u8) {
        if neu < alt {
            let b = p("aufstieg.anteil") * self.praemie_referenz();
            self.praemie_buchen(crate::id("aufstieg"), crate::id("aufstieg"), b, String::new());
        } else if neu > alt {
            let b = p("abstieg.fallschirm_anteil") * self.praemie_referenz() / p("abstieg.jahre").max(1.0);
            self.praemie_buchen(crate::id("abstieg"), crate::id("abstieg"), b, String::new());
        }
    }
    /// How unevenly the league shares its TV money (first to last).
    pub fn tv_spreizung(&self) -> f64 {
        let land = self.world.leagues[self.league()].land.to_lowercase();
        let tab = crate::regeln::wert("praemien", "liga_tv.liga");
        tab.as_object().and_then(|o| o.iter().find(|(n, _)| land.contains(n.as_str())).and_then(|(_, v)| v["spreizung"].as_f64())).unwrap_or(p("liga_tv.standard.spreizung"))
    }
    /// Season end: the TV money's extra for a good final place (never negative), Jahresbasis × Faktor × (Gewicht − 1).
    pub fn tv_zuschlag_saison(&mut self, platz: usize) {
        let l = self.league();
        let n = self.world.leagues[l].teams.len();
        if n < 2 || platz == 0 { return; }
        let s = self.tv_spreizung();
        let gewicht = 2.0 / (1.0 + s) * (1.0 + (s - 1.0) * (n - platz) as f64 / (n - 1) as f64);
        let basis = (f("vereinswelt", "einnahmen.tv_basis") as f64 + f("vereinswelt", "einnahmen.tv_je_ruf") as f64 * self.kraft()) * self.skala();
        let z = ((gewicht - 1.0) * basis * p("liga_tv.zuschlag_faktor")).max(0.0);
        if z >= 1000.0 {
            self.book(t("vw.art.tv"), (z / 1000.0).round() * 1000.0);
            let wort = t(if s <= 1.3 { "vw.tv.gleich" } else if s <= 1.9 { "vw.tv.maessig" } else { "vw.tv.stark" });
            self.msg(3, t("vw.figur.finanz"), t("pr.tv_titel"), &format!("{} {wort}", crate::tf!("pr.tv_text", platz, money(z))), Action::None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pokal_verdoppelt_je_runde() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.wirtschaft_an();
        let r = c.praemie_referenz();
        assert!(r > 0.0);
        let k0 = c.club().balance;
        c.auf_abstieg_geld(2, 1);
        assert!((c.club().balance - k0 - 0.06 * r).abs() < 1000.0, "promotion: 6 % of the reference");
        let k1 = c.club().balance;
        c.tv_zuschlag_saison(1);
        assert!(c.club().balance > k1, "the champion gets the TV extra");
        let k2 = c.club().balance;
        c.tv_zuschlag_saison(c.world.leagues[c.league()].teams.len());
        assert_eq!(c.club().balance, k2, "the last pays nothing back");
    }
}
