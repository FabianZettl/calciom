//! A new stadium (own design after Anstoss and the EA manager): from facility level 3 the club can build a new arena
//! with half again as many seats; it costs per seat, takes two and a half years, can be paid with a bank loan, and on
//! opening the fans celebrate and a naming-rights partner is found again.
use crate::game::*;
use crate::inhalt::t;
use crate::world::*;
use serde::{Deserialize, Serialize};

const MEHR_PLAETZE: f64 = 0.5;
const KOSTEN_JE_PLATZ: f64 = 7_000.0;
const BAUZEIT_TAGE: u32 = 30 * 30;
const AB_STUFE: u8 = 3;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Neubau { pub fertig: u32, pub kosten: f64, pub faktor: f64 }

impl Career {
    /// Capacity factor of a new stadium that is open (1 without).
    pub fn neubau_faktor(&self) -> f64 { if self.neubau.faktor > 0.0 && self.neubau.fertig > 0 && self.abs() >= self.neubau.fertig { self.neubau.faktor } else { 1.0 } }
    pub fn neubau_laeuft(&self) -> bool { self.neubau.fertig > self.abs() }
    /// Cost of a new arena: the new seats in the club's money scale.
    pub fn neubau_kosten(&self) -> f64 { round_money(self.kapazitaet() * MEHR_PLAETZE * KOSTEN_JE_PLATZ * self.skala().sqrt()) }
    pub fn neubau_moeglich(&self) -> Result<(), String> {
        if self.neubau_faktor() > 1.0 || self.neubau_laeuft() { return Err(t("nb.schon").into()); }
        if self.wi.stufen[0] < AB_STUFE { return Err(crate::tf!("nb.stufe", AB_STUFE)); }
        Ok(())
    }
    pub fn neubau_starten(&mut self) -> Result<String, String> {
        self.neubau_moeglich()?;
        let k = self.neubau_kosten();
        if self.club().balance - k < -f64::from(crate::regeln::f("vereinswelt", "budget.schuldengrenze_anteil")) * self.umsatz_prognose() { return Err(t("nb.geld").into()); }
        self.book(t("nb.buchung"), -k);
        self.neubau = Neubau { fertig: self.abs() + BAUZEIT_TAGE, kosten: k, faktor: 1.0 + MEHR_PLAETZE };
        self.klima(3.0, 2.0, 0.0, t("nb.buchung"));
        let cn = self.club().name.clone();
        self.news(&crate::tf!("nb.news", cn), &crate::tf!("nb.news_text", money(k)));
        Ok(crate::tf!("nb.ok", money(k)))
    }
    /// Daily: the opening.
    pub fn neubau_tag(&mut self) {
        if self.neubau.fertig == 0 || self.abs() != self.neubau.fertig { return; }
        let cn = self.club().name.clone();
        self.klima(6.0, 3.0, 3.0, t("nb.eroeffnung"));
        let plaetze = self.kapazitaet() * self.neubau.faktor;
        self.msg(0, t("vw.figur.praesident"), t("nb.eroeffnung"), &crate::tf!("nb.eroeffnung_text", cn.clone(), format!("{plaetze:.0}")), Action::None);
        self.news(&crate::tf!("nb.eroeffnung_news", cn), &crate::tf!("nb.eroeffnung_text", self.club().name.clone(), format!("{plaetze:.0}")));
        // the naming rights are for sale again
        self.wi.vertraege.retain(|s| s.art != 2);
        self.wi.angebot_monat = u32::MAX;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn neues_stadion() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.wirtschaft_an();
        c.wi.stufen[0] = 3;
        c.world.clubs[c.own].balance = 1e10;
        let k0 = c.kapazitaet() * c.neubau_faktor();
        c.neubau_starten().unwrap();
        assert!(c.neubau_starten().is_err());
        c.world.day += 1;
        for _ in 0..BAUZEIT_TAGE { c.world.day = (c.world.day + 1) % crate::date::SEASON_DAYS; if c.world.day == 0 { c.world.season += 1; } c.neubau_tag(); }
        assert!(c.kapazitaet() > k0 * 1.4);
    }
}
