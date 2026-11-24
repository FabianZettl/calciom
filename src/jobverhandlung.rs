//! Negotiating the coach's own contract with a new club (pesm/core/laufbahn/gespraech.py, laufbahn.toml [verhandlung]):
//! wage, contract length, transfer budget and the season goal. Every demand above the offer costs the club's goodwill
//! (weighted by its board's persona); within its tolerance it agrees, up to twice the tolerance it makes a counter-offer
//! (halfway), beyond that or after three rounds it withdraws.
use crate::game::*;
use crate::inhalt::t;
use crate::regeln::f;
use crate::world::*;
use serde::{Deserialize, Serialize};

/// The points with an effect: wage, years, transfer budget, season goal (a higher place number is the more modest goal).
pub const PUNKTE: [&str; 4] = [crate::id("gehalt"), crate::id("laufzeit"), crate::id("budget"), crate::id("ziel")];

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct JobVerh { pub club: usize, pub angebot: [f64; 4], pub punkte: [f64; 4], pub max: [f64; 4], pub interesse: f64, pub persona: String, pub runde: u8 }

fn v(k: &str) -> f64 { f("laufbahn", &format!("verhandlung.{k}")) as f64 } // roh
fn arr2(k: &str) -> (f64, f64) { let a = crate::mensch::arr("laufbahn", &format!("verhandlung.{k}")); (a[0].as_f64().unwrap_or(1.0), a[1].as_f64().unwrap_or(0.0)) } // roh

impl JobVerh {
    fn gewicht(&self, i: usize) -> f64 {
        let n = PUNKTE[i];
        v(&format!("gewicht.{n}")) * crate::regeln::wert("laufbahn", "verhandlung.persona_gewicht")[self.persona.as_str()][n].as_f64().unwrap_or(1.0)
    }
    /// How far the demand is above what the club wants to give (0 and below: satisfied).
    pub fn unzufriedenheit(&self) -> f64 {
        (0..4).map(|i| {
            let span = match i { 1 => 1.0, 3 => (self.max[3] - self.angebot[3]).max(1.0), _ => (self.max[i] - self.angebot[i]).max(1.0) };
            self.gewicht(i) * ((self.punkte[i] - self.angebot[i]) / span).clamp(-0.6, 3.0)
        }).sum()
    }
    pub fn toleranz(&self) -> f64 { v("toleranz_basis") + v("toleranz_je_interesse") * self.interesse }
    /// 0 satisfied, 1 hesitant, 2 refusing.
    pub fn stimmung(&self) -> usize { let (u, t) = (self.unzufriedenheit(), self.toleranz()); if u <= t { 0 } else if u <= t * v("gegenvorschlag_bis") { 1 } else { 2 } }
}

impl Career {
    /// Open the talks about a job offer: the offer as the club makes it, its limits from its interest in us.
    pub fn job_verhandeln(&mut self, club: usize, wage: f64) {
        let interesse = (0.5 + (self.mgr_rep as f64 - self.world.clubs[club].rep as f64) / 100.0).clamp(0.1, 1.0);
        let budget = round_money(self.world.revenue(club) * 0.22);
        let ziel = self.world.clubs[club].goal_pos as f64;
        let (g0, g1) = arr2("gehalt_max_faktor");
        let (b0, b1) = arr2("budget_max_faktor");
        let lz = crate::mensch::arr("laufbahn", "verhandlung.laufzeit");
        let jahre = 2.0_f64.clamp(lz[0].as_f64().unwrap_or(1.0), lz[1].as_f64().unwrap_or(4.0));
        let angebot = [wage, jahre, budget, ziel];
        let j = JobVerh { club, angebot, punkte: angebot, max: [wage * (g0 + g1 * interesse), lz[1].as_f64().unwrap_or(4.0), budget * (b0 + b1 * interesse), ziel + (1.0 + 2.0 * interesse).round()], interesse, persona: self.persona_von(club).to_string(), runde: 0 };
        self.mgr.verh = Some(j);
        self.job_nachricht(None);
    }
    /// The talks as a message: offer and demand per point, the club's mood, the options.
    fn job_nachricht(&mut self, vorspann: Option<String>) {
        let Some(j) = self.mgr.verh.clone() else { return };
        let fmt = |i: usize, x: f64| match i { 0 | 2 => money(x), 1 => crate::tf!("ui.post.jahre", x as u32), _ => crate::tf!("jv.platz", x as u32) };
        let mut zeilen: Vec<String> = vorspann.into_iter().collect();
        for i in 0..4 {
            let n = t(&format!("lb.punkt.{}", PUNKTE[i])); // roh
            zeilen.push(if (j.punkte[i] - j.angebot[i]).abs() < 1e-6 { format!("{n}: {}", fmt(i, j.angebot[i])) } else { crate::tf!("jv.zeile", n, fmt(i, j.angebot[i]), fmt(i, j.punkte[i])) });
        }
        zeilen.push(format!("{} · {}", t(["lb.stimmung.zufrieden", "lb.stimmung.zoegerlich", "lb.stimmung.ablehnend"][j.stimmung()]), crate::tf!("lb.verhandlung.unter", j.runde + 1, v("runden_hoechstens") as u32)));
        let opts = vec![
            (crate::id("gehalt").to_string(), crate::tf!("jv.mehr", t("lb.punkt.gehalt"))),
            (crate::id("budget").to_string(), crate::tf!("jv.mehr", t("lb.punkt.budget"))),
            (crate::id("ziel").to_string(), t("jv.ziel").to_string()),
            (crate::id("laufzeit").to_string(), t("jv.laufzeit").to_string()),
            (crate::id("abgeben").to_string(), t("lb.aktion.abgeben").to_string()),
            (crate::id("annehmen").to_string(), t("lb.aktion.einig").to_string()),
        ];
        let cn = self.world.clubs[j.club].name.clone();
        self.msg(5, &cn, &crate::tf!("jv.titel", cn.clone()), &zeilen.join(crate::id("\n")), Action::Wahl { art: crate::id("jobverh").into(), bezug: vec![], optionen: opts });
    }
    /// One move in the talks (mensch.rs wahl „jobverh“).
    pub fn job_antwort(&mut self, key: &str) -> String {
        let Some(mut j) = self.mgr.verh.clone() else { return t("ui.rest3.die_bewerbung_ist_vorbei").into() };
        let schritt = |i: usize, j: &JobVerh| match i { 0 => (j.angebot[0] * v("schritt_gehalt") / 10_000.0).round().max(1.0) * 10_000.0, 2 => (j.angebot[2] * v("schritt_budget") / 100_000.0).round().max(1.0) * 100_000.0, _ => 1.0 };
        match key {
            "gehalt" | "budget" | "ziel" | "laufzeit" => {
                let i = PUNKTE.iter().position(|x| *x == key).unwrap_or(0);
                let neu = j.punkte[i] + schritt(i, &j);
                if neu > j.max[i] * 1.5 + 1e-6 || (i == 1 && neu > j.max[1]) { self.mgr.verh = Some(j); self.job_nachricht(Some(t("jv.grenze").into())); return t("jv.grenze").into(); }
                j.punkte[i] = neu;
                self.mgr.verh = Some(j);
                self.job_nachricht(None);
                t("jv.notiert").into()
            }
            "annehmen" => { j.punkte = j.angebot; self.job_abschluss(j) }
            _ => {
                j.runde += 1;
                match j.stimmung() {
                    0 => self.job_abschluss(j),
                    s if s == 2 || j.runde as f64 >= v("runden_hoechstens") => {
                        self.mgr.verh = None;
                        crate::tf!("jv.abbruch", self.world.clubs[j.club].name.clone())
                    }
                    _ => {
                        // counter-offer: halfway on every demand above the offer; concessions are kept
                        for i in 0..4 { if j.punkte[i] > j.angebot[i] { j.angebot[i] = if i == 1 || i == 3 { (j.angebot[i] + ((j.punkte[i] - j.angebot[i]) / 2.0).floor()).round() } else { round_money(j.angebot[i] + (j.punkte[i] - j.angebot[i]) / 2.0) }; } }
                        j.punkte = j.angebot;
                        self.mgr.verh = Some(j);
                        self.job_nachricht(Some(t("lb.verhandlung.gegenvorschlag").into()));
                        t("lb.verhandlung.gegenvorschlag").into()
                    }
                }
            }
        }
    }
    /// Agreement: the switch with the negotiated wage, budget, goal and contract length.
    fn job_abschluss(&mut self, j: JobVerh) -> String {
        self.mgr.verh = None;
        self.switch_club(j.club, j.punkte[0]);
        self.transfer_budget = j.punkte[2];
        self.world.clubs[j.club].goal_pos = j.punkte[3].max(1.0) as usize;
        self.mensch.vertrag_bis = self.abs() + (j.punkte[1] * crate::date::SEASON_DAYS as f64) as u32;
        self.ziel_auffrischen();
        crate::tf!("jv.einig", self.world.clubs[j.club].name.clone(), money(j.punkte[0]), j.punkte[1] as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eigener_vertrag_verhandeln() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let club = (0..c.world.clubs.len()).find(|&x| x != c.own).unwrap();
        c.job_verhandeln(club, 1_000_000.0);
        assert_eq!(c.mgr.verh.as_ref().unwrap().stimmung(), 0, "the offer itself satisfies the club");
        c.job_antwort("gehalt");
        assert!(c.mgr.verh.as_ref().unwrap().punkte[0] > 1_000_000.0);
        // a modest demand is accepted on handing it in
        c.job_antwort("abgeben");
        assert!(c.own == club || c.mgr.verh.is_some(), "accepted or a counter-offer");
        // greed ends the talks
        let mut d = Career::new(crate::world::testwelt(7), 0, "Test");
        d.job_verhandeln(club, 1_000_000.0);
        for _ in 0..12 { d.job_antwort("gehalt"); d.job_antwort("budget"); }
        let r = d.job_antwort("abgeben");
        assert!(d.mgr.verh.is_none() || r == t("lb.verhandlung.gegenvorschlag"));
    }
}
