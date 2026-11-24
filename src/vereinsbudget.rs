//! The money a club of its size really has (pesm/core/wirtschaft/budget_start.py klasse / budget_nach_klasse; Fabian
//! 23.11.: Bayern far above 60 Mio.): the club class from its size and league tier gives the yearly revenue (a share of
//! the top-league average), known big clubs use their real revenue, the transfer budget is a share of it. At the start the
//! board and the fans of a big club expect more: their mood starts lower than at a small club.
use crate::game::*;
use crate::regeln::{f, wert};
use crate::world::round_money;

fn b(k: &str) -> f64 { f("budget_start", k) as f64 }

impl Career {
    /// Position on the class ladder 0…5 and the revenue share of the top-league average, between the rungs.
    pub fn klasse(&self) -> (f64, f64) {
        let stufe = self.world.leagues[self.league()].stufe.clamp(1, 3);
        let kl = wert("budget_start", &format!("liga.stufe_{stufe}.klassen")); // roh
        let (von, bis) = (kl[0].as_f64().unwrap_or(0.0), kl[1].as_f64().unwrap_or(1.0));
        // size counts absolutely and within the league: the smallest club of a top league is a small top-league club
        let teams = &self.world.leagues[self.league()].teams;
        let besser = teams.iter().filter(|&&t| self.world.clubs[t].level > self.club().level).count() as f64;
        let rel = if teams.len() > 1 { 1.0 - besser / (teams.len() - 1) as f64 } else { 0.5 };
        let x = von + (bis - von) * (0.25 * self.prestige().clamp(0.0, 1.0) + 0.75 * rel * rel);
        let ks = wert("budget_start", crate::id("klasse")).as_array().unwrap();
        let i = (x as usize).min(ks.len() - 2);
        let fr = x - i as f64;
        let (a, c) = (ks[i]["umsatz_rel"].as_f64().unwrap_or(1.0), ks[i + 1]["umsatz_rel"].as_f64().unwrap_or(1.0));
        (x, a * (c / a).powf(fr))
    }
    /// The real-world revenue of the club in euros: the known figure of a big club, else the class estimate.
    pub fn umsatz_echt(&self) -> f64 {
        let name = self.club().name.clone();
        let echt = wert("budget_start", crate::id("umsatz_echt")).as_object().and_then(|o| o.iter().find(|(n, _)| name.contains(n.as_str())).and_then(|(_, v)| v.as_f64()));
        match echt { Some(m) => m * 1e6, None => self.klasse().1 * b("budget.referenz_umsatz") }
    }
    /// The infrastructure pot (Fabian 23.11.): facilities are paid from it.
    pub fn infra(&self) -> f64 { self.infra_topf.unwrap_or_else(|| round_money(self.umsatz_prognose() * f("vereinswelt", "toepfe.infra_anteil") as f64)) }
    /// The wage pot: the yearly wage sum the club allows (share of the revenue plus what the board added or was moved).
    pub fn gehalt_topf(&self) -> f64 { self.world.wage_bill(self.own) + self.gehalt_frei() }
    /// What each pot can give away: transfers (minus running bids), wages (never below the wages paid), infrastructure.
    pub fn topf_frei(&self, t: u8) -> f64 { match t { 0 => self.transfer_frei(), 1 => self.gehalt_frei(), _ => self.infra().max(0.0) } }
    /// Moves money between the pots (0 transfers, 1 wages per year, 2 infrastructure). Up to `frei_bis` of the whole
    /// budget at once on the coach's word; more needs the board (full, half or no, by confidence and form).
    pub fn budget_schieben(&mut self, von: u8, nach: u8, betrag: f64) -> Result<String, String> {
        let t = |k: &str| f("vereinswelt", &format!("toepfe.{k}"));
        if von == nach || betrag <= 0.0 { return Err(crate::inhalt::t("ui.topf.nichts").into()); }
        if betrag > self.topf_frei(von) + 1.0 { return Err(crate::inhalt::t(if von == 1 { "ui.topf.gehalt_belegt" } else { "ui.topf.zu_wenig" }).into()); }
        let gesamt = self.transfer_budget.max(0.0) + self.gehalt_topf() + self.infra();
        let mut x = betrag;
        let mut zusatz = String::new();
        if betrag > gesamt * t("frei_bis") as f64 {
            let bereit = self.mensch.vorstand / 100.0 * 0.7 + self.vereins_ppg().unwrap_or(1.3) / 3.0 * 0.3;
            if bereit < t("vorstand_halb_ab") { return Err(crate::inhalt::t("ui.topf.vorstand_nein").into()); }
            if bereit < t("vorstand_ja_ab") { x = round_money(betrag / 2.0); zusatz = crate::inhalt::t("ui.topf.vorstand_halb").into(); }
        }
        match von { 0 => self.transfer_budget -= x, 1 => self.gehalt_plus -= x, _ => self.infra_topf = Some(self.infra() - x) }
        match nach { 0 => self.transfer_budget += x, 1 => self.gehalt_plus += x, _ => self.infra_topf = Some(self.infra() + x) }
        let namen = [crate::inhalt::t("ui.topf.0"), crate::inhalt::t("ui.topf.1"), crate::inhalt::t("ui.topf.2")];
        Ok(format!("{}{zusatz}", crate::tf!("ui.topf.verschoben", crate::world::money(x), namen[von.min(2) as usize], namen[nach.min(2) as usize])))
    }
    /// Transfer budget the club's size gives: share of the revenue minus the reserve.
    pub fn transferbudget_echt(&self) -> f64 { round_money(self.umsatz_echt() * b("budget.transferanteil") * (1.0 - b("budget.reserve"))) }
    /// Where a year's money comes from (forecast): TV, sponsors, match days, advertising, merchandise, members, prizes so far.
    pub fn einnahmen_mix(&self) -> Vec<(&'static str, f64)> {
        let t = crate::inhalt::t;
        let heim = (self.world.leagues[self.league()].teams.len().max(2) - 1) as f64;
        let s = self.spiel_neutral();
        let praemien: f64 = self.book.iter().filter(|b| b.1 > 0.0 && (b.0.contains(t("vw.art.praemie_wettbewerb")) || b.0.contains(t("ui.game.siegpraemien")) || b.0.contains(t("ui.game.praemien_europapokal")))).map(|b| b.1).sum();
        vec![(t("vw.art.tv"), self.tv_monat() * 12.0), (t("vw.art.sponsoren"), self.sponsoren_jahr()), (t("vb.spieltag"), heim * (s.ein[0] + s.ein[1] + s.ein[2] + s.ein[3])),
            (t("vw.art.werbung"), heim * s.ein[4]), (t("vw.art.merchandising"), self.merch_posten().0 * 12.0), (t("mv.beitraege"), self.beitraege_monat() * 12.0), (t("vb.praemien"), praemien)]
    }
    /// At the start of a job: the budget by the club's size, and the mood of board and fans by its expectations.
    pub fn start_erwartung(&mut self) {
        self.transfer_budget = self.transfer_budget.max(self.transferbudget_echt());
        let p = self.prestige() as f32;
        self.mensch.vorstand = 58.0 - 18.0 * p;
        self.fans = 62.0 - 12.0 * p;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grosser_verein_mehr_geld_und_mehr_druck() {
        let w = crate::world::World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let gross = w.clubs.iter().position(|c| c.name.contains("Bayern")).unwrap();
        let liga = w.clubs[gross].league;
        let klein = *w.leagues[liga].teams.iter().min_by(|a, c| w.clubs[**a].level.partial_cmp(&w.clubs[**c].level).unwrap()).unwrap();
        let g = Career::new(w.clone(), gross, "T");
        let k = Career::new(w, klein, "T");
        assert!(g.transfer_budget > 100e6, "Bayern: {}", g.transfer_budget);
        assert!(g.transfer_budget > 3.0 * k.transfer_budget, "{} {} {:?} {:?} {}", g.transfer_budget, k.transfer_budget, k.klasse(), g.klasse(), k.club().name);
        assert!(g.mensch.vorstand < k.mensch.vorstand && g.fans < k.fans, "a big club is harder at the start");
    }
}
