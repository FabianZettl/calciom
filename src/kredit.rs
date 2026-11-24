//! Bank loans (own design after Anstoss: the classic house bank): the frame follows the yearly income and the board's
//! confidence, the rate the club's standing (cash, board, term); repaid in monthly instalments (annuity), early repayment
//! costs a small fee. Half of a loan goes to the transfer budget, as the board agrees to spend it.
use crate::game::*;
use crate::inhalt::t;
use crate::world::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Kredit { pub betrag: f64, pub rest: f64, pub zins: f64, pub monate: u32, pub rate: f64, pub beginn: u32 }

const RAHMEN_ANTEIL: f64 = 0.3;
/// A percentage with one decimal in the language's way (5,0 in German).
pub fn prozent(x: f64) -> String { let s = format!("{x:.1}"); if crate::inhalt::sprache() == crate::inhalt::Sprache::En { s } else { s.replace('.', ",") } }
const VORZEITIG_GEBUEHR: f64 = 0.02;

impl Career {
    /// How much the bank still lends: 30 % of a year's income, more with a confident board, minus what is owed.
    pub fn kredit_rahmen(&self) -> f64 {
        let offen: f64 = self.kredite.iter().map(|k| k.rest).sum();
        round_money((self.umsatz_prognose() * RAHMEN_ANTEIL * (0.5 + self.mensch.vorstand as f64 / 100.0) - offen).max(0.0))
    }
    /// Yearly rate in percent: 4 % base, half a point a year of term, more in debt or with a doubting board.
    pub fn kredit_zins(&self, jahre: u32) -> f64 {
        4.0 + 0.5 * jahre as f64 + if self.club().balance < 0.0 { 3.0 } else { 0.0 } + if self.mensch.vorstand < 35.0 { 2.0 } else { 0.0 }
    }
    pub fn kredit_rate(betrag: f64, zins: f64, monate: u32) -> f64 {
        let r = zins / 100.0 / 12.0;
        if r <= 0.0 { betrag / monate.max(1) as f64 } else { betrag * r / (1.0 - (1.0 + r).powi(-(monate as i32))) }
    }
    pub fn kredit_aufnehmen(&mut self, betrag: f64, jahre: u32) -> Result<String, String> {
        let betrag = round_money(betrag);
        if betrag <= 0.0 || betrag > self.kredit_rahmen() + 1.0 { return Err(t("kr.zu_viel").into()); }
        let jahre = jahre.clamp(1, 5);
        let zins = self.kredit_zins(jahre);
        let monate = jahre * 12;
        let rate = Self::kredit_rate(betrag, zins, monate);
        self.kredite.push(Kredit { betrag, rest: betrag, zins, monate, rate, beginn: self.abs() });
        self.book(t("kr.auszahlung"), betrag);
        self.transfer_budget += betrag * 0.5;
        // a board does not like a coach who borrows while in the red
        if self.club().balance - betrag < 0.0 { self.klima(0.0, 0.0, -2.0, t("kr.auszahlung")); }
        Ok(crate::tf!("kr.ok", money(betrag), prozent(zins), money(rate)))
    }
    /// Monthly: interest and repayment of every loan.
    pub fn kredite_monat(&mut self) {
        let mut zinsen = 0.0;
        let mut tilgung = 0.0;
        for k in self.kredite.iter_mut() {
            let z = k.rest * k.zins / 100.0 / 12.0;
            let ti = (k.rate - z).min(k.rest).max(0.0);
            k.rest -= ti;
            zinsen += z;
            tilgung += ti;
        }
        self.kredite.retain(|k| k.rest > 1.0);
        if zinsen > 0.0 { self.book(t("kr.zinsen"), -zinsen); }
        if tilgung > 0.0 { self.book(t("kr.tilgung"), -tilgung); }
    }
    /// Repay a loan at once: the rest plus a 2 % fee.
    pub fn kredit_abloesen(&mut self, i: usize) -> Result<String, String> {
        let Some(k) = self.kredite.get(i).cloned() else { return Err(t("kr.zu_viel").into()) };
        let summe = k.rest * (1.0 + VORZEITIG_GEBUEHR);
        if self.club().balance < summe { return Err(t("tv.drama.rueckmeldung.budget").into()); }
        self.book(t("kr.tilgung"), -k.rest);
        self.book(t("kr.zinsen"), -k.rest * VORZEITIG_GEBUEHR);
        self.kredite.remove(i);
        Ok(crate::tf!("kr.abgeloest", money(summe)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kredit_aufnehmen_und_tilgen() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.wirtschaft_an();
        let rahmen = c.kredit_rahmen();
        assert!(rahmen > 0.0);
        assert!(c.kredit_aufnehmen(rahmen * 2.0, 2).is_err(), "not above the frame");
        let k0 = c.club().balance;
        c.kredit_aufnehmen(rahmen / 2.0, 1).unwrap();
        assert!(c.club().balance > k0);
        for _ in 0..12 { c.kredite_monat(); }
        assert!(c.kredite.is_empty(), "repaid after a year");
        // the twelve instalments cost more than the loan (interest)
        assert!(c.club().balance < k0);
    }
}
