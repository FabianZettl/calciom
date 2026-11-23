//! Paket 3e: negotiation as a round model (core/verhandlung/runden.py, Mechanik-Übergabe §7) and the contract demand
//! of own players (§8, gespraeche/werte.py:122/169). A phase is the talk with one counterpart (club, then player);
//! "Wert" always rises for the counterpart (fee when buying, wage for the player). Randomness: mensch::zufall.
use serde::{Deserialize, Serialize};

pub const MAX_RUNDEN: u8 = 8;
pub const ABKUEHL_TAGE: u32 = 14;
pub const ABKUEHL_LETZT: u32 = 21;
pub const GEDULD_WARN: f32 = 25.0;
pub const NICHT_ERNST: f32 = 0.30;
pub const DRUCK_RISIKO: f32 = 0.12;
pub const DRUCK_PLUS: f32 = 0.12;
pub const FRIST_TAGE: u32 = 7;
pub const ZEIT_MAX: u8 = 2;
pub const ZEIT_TAGE: u32 = 2;

/// (flex, konz, geduld, kosten, zone lo/hi, letzt lo/hi, bedenk) per character (runden.py CHARAKTER).
pub fn charakter(name: &str) -> (f32, f32, f32, f32, (f32, f32), (f32, f32), f32) {
    match name {
        "pragmatiker" => (0.14, 0.35, 100.0, 8.0, (0.25, 0.80), (0.65, 0.97), 0.05),
        "hardliner" => (0.04, 0.12, 70.0, 12.0, (0.10, 0.55), (0.12, 0.60), 0.10),
        _ => (0.09, 0.25, 90.0, 9.0, (0.20, 0.70), (0.45, 0.92), 0.07),
    }
}
pub fn charakter_wort(name: &str) -> &'static str {
    match name { "pragmatiker" => "kompromissbereit", "hardliner" => "knallhart", _ => crate::id("fair") }
}
/// Tone and action: (chance, patience cost, risk, relation) — runden.py TON / HANDLUNG.
pub fn toene_t() -> [(&'static str, &'static str, f32, f32); 3] { [("sachlich", crate::inhalt::t("an.ton.sachlich"), 0.01, 0.0), ("herzlich", crate::inhalt::t("an.ton.foerdernd"), 0.03, -2.0), ("bestimmt", crate::inhalt::t("gs.ton.bestimmt"), 0.0, 2.0)] } // roh: ids
pub fn handlungen_t() -> [(&'static str, &'static str, f32, f32, f32, f32); 4] { [
    (crate::id("neutral"), crate::inhalt::t("ui.verhandlung.nur_das_angebot"), 0.0, 0.0, 0.0, 0.0),
    (crate::id("erklaeren"), crate::inhalt::t("ui.verhandlung.gut_erklaeren"), 0.04, 0.0, 0.0, 0.0),
    (crate::id("gefuehl"), crate::inhalt::t("ui.verhandlung.aufs_gefuehl_setzen"), 0.05, 1.0, 0.0, 0.0),
    (crate::id("druck"), crate::inhalt::t("dlg.absicht.druck"), DRUCK_PLUS, 6.0, DRUCK_RISIKO, -1.0),
] }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Phase {
    pub ziel: f64,
    pub schmerz: f64,
    pub skala: f64,
    pub charakter: String,
    pub beziehung: f32,
    pub lage: f32,
    pub runde: u8,
    pub geduld: f32,
    pub gegen: Option<f64>,
    pub letztes_wort: Option<f64>,
    pub bedenk: Option<f64>,
    pub zeit: u8,
}
/// The answer of the counterpart.
#[derive(Clone, Debug, PartialEq)]
pub enum Reaktion { Annehmen(f64), Gegen(f64), LetztesWort(f64), Ablehnen(&'static str), Bedenkzeit, Abbruch(&'static str) }
/// Side effects for the caller: mood of the player (−3 not serious, −4 last offer refused, −8 pressure burst), relation.
#[derive(Clone, Debug, Default)]
pub struct Folgen { pub stimmung: f32, pub beziehung: f32, pub abkuehl: u32 }

/// Pain limit from target, character, relation (−3…3) and position of the counterpart (−1 needy … +1 strong).
pub fn schmerz_aus(ziel: f64, ch: &str, beziehung: f32, lage: f32) -> f64 {
    let f = (charakter(ch).0 * (1.0 + 0.1 * beziehung - 0.2 * lage)).max(0.01);
    ziel * (1.0 - f as f64)
}
impl Phase {
    pub fn neu(ziel: f64, ch: &str, beziehung: f32, lage: f32) -> Phase {
        Phase { ziel, schmerz: schmerz_aus(ziel, ch, beziehung, lage), skala: ziel.max(1.0), charakter: ch.into(), beziehung, lage, runde: 0,
            geduld: charakter(ch).2 + 4.0 * beziehung, gegen: None, letztes_wort: None, bedenk: None, zeit: 0 }
    }
    /// The counterpart's current target: it comes closer each round.
    pub fn ziel_akt(&self, runde: u8) -> f64 { self.ziel + (self.schmerz - self.ziel) * (charakter(&self.charakter).1 as f64 * runde as f64).min(1.0) }
    fn anteil(&self, wert: f64, r: f64) -> f32 { let d = r - self.schmerz; if d == 0.0 { 1.0 } else { ((wert - self.schmerz) / d).clamp(0.0, 1.0) as f32 } }
    /// Chance that an offer is accepted (the preview shows the same number that is rolled). vor = 1 before sending.
    pub fn chance(&self, wert: f64, letztes: bool, ton: usize, handlung: usize, vor: u8, bonus: f32) -> f32 {
        let c = charakter(&self.charakter);
        if wert < self.schmerz { return 0.0; }
        if letztes {
            if wert >= self.ziel { return 1.0; }
            let (lo, hi) = c.5;
            return (lo + (hi - lo) * self.anteil(wert, self.ziel) + 0.04 * self.beziehung + 0.03 * self.lage + bonus).clamp(0.0, 0.97);
        }
        let za = self.ziel_akt(self.runde + vor);
        if wert >= za { return 1.0; }
        let (lo, hi) = c.4;
        let plus = toene_t()[ton].2 + handlungen_t()[handlung].2 + 0.03 * self.beziehung + 0.04 * self.lage + bonus;
        ((lo + (hi - lo) * self.anteil(wert, za)) * (0.5 + 0.5 * self.geduld / 100.0) + plus).clamp(0.0, 0.95)
    }
    /// Send an offer (runden.py senden). `w(k)` gives the k-th fixed random number of this move.
    pub fn senden(&mut self, wert: f64, letztes: bool, ton: usize, handlung: usize, bonus: f32, w: &dyn Fn(u32) -> f32) -> (Reaktion, Folgen) {
        let c = charakter(&self.charakter);
        let mut fo = Folgen::default();
        self.runde += 1;
        let (dg, risiko0, dbez) = (toene_t()[ton].3 + handlungen_t()[handlung].3, handlungen_t()[handlung].4, handlungen_t()[handlung].5);
        if handlungen_t()[handlung].0 == crate::id("druck") {
            let risiko = risiko0 + 0.08 * (-self.beziehung).max(0.0) + if self.charakter == crate::id("hardliner") { 0.08 } else { 0.0 };
            if w(1) < risiko {
                self.geduld = (self.geduld - 45.0).max(0.0);
                fo.stimmung = -8.0;
                fo.beziehung = -1.0;
                if self.geduld <= 0.0 || w(2) < 0.5 { fo.abkuehl = ABKUEHL_TAGE; return (Reaktion::Abbruch("druck_geplatzt"), fo); }
                return (Reaktion::Ablehnen("druck_geplatzt"), fo);
            }
        }
        fo.beziehung += dbez;
        if letztes {
            let pr = self.chance(wert, true, ton, 0, 0, bonus);
            if w(3) < pr { return (Reaktion::Annehmen(wert), fo); }
            fo.stimmung -= 4.0;
            fo.beziehung -= 1.0;
            fo.abkuehl = ABKUEHL_LETZT;
            return (Reaktion::Abbruch(if wert < self.schmerz { "letztes_unter_schmerz" } else { "letztes_nein" }), fo);
        }
        let pr = self.chance(wert, false, ton, handlung, 0, bonus);
        self.geduld = (self.geduld - (c.3 + dg).max(0.0) * (1.0 + 3.0 * ((self.schmerz - wert) / self.skala).max(0.0) as f32)).max(0.0);
        let ernst = ((self.schmerz - wert) / self.skala) as f32 > NICHT_ERNST;
        if ernst { self.geduld = (self.geduld - 35.0).max(0.0); fo.stimmung -= 3.0; }
        if pr >= 1.0 || w(4) < pr { return (Reaktion::Annehmen(wert), fo); }
        if wert >= self.schmerz && self.zeit < 2 && w(5) < c.6 && handlungen_t()[handlung].0 != crate::id("druck") {
            self.bedenk = Some(wert);
            return (Reaktion::Bedenkzeit, fo);
        }
        if self.geduld <= 0.0 || self.runde >= MAX_RUNDEN {
            fo.abkuehl = ABKUEHL_TAGE;
            if ernst { fo.beziehung -= 1.0; }
            return (Reaktion::Abbruch(if ernst { "nicht_ernst" } else { "geduld_leer" }), fo);
        }
        let g = self.ziel_akt(self.runde);
        if ernst || (wert < self.schmerz && w(6) < 0.25) { return (Reaktion::Ablehnen(if ernst { "nicht_ernst" } else { "zu_wenig" }), fo); }
        self.gegen = Some(g);
        if self.geduld < GEDULD_WARN { self.letztes_wort = Some(g); return (Reaktion::LetztesWort(g), fo); }
        (Reaktion::Gegen(g), fo)
    }
    /// After a day of thinking: the waiting offer is accepted with its chance, else a counter-offer.
    pub fn nach_bedenkzeit(&mut self, w: f32) -> Reaktion {
        let Some(wert) = self.bedenk.take() else { return Reaktion::Gegen(self.ziel_akt(self.runde)) };
        self.zeit += 1;
        if w < self.chance(wert, false, 0, 0, 0, 0.0) { return Reaktion::Annehmen(wert); }
        let g = self.ziel_akt(self.runde);
        self.gegen = Some(g);
        Reaktion::Gegen(g)
    }
}

/// Contract demand of an own player (werte.py:122): base × clamp(1 + (Ego − 0.5)·0.3 + (50 − Moral)/100·0.3
/// − (Loyalität − 0.5)·0.15 − Beziehung/100·0.1 + max(0, 50 − Vertrauen)/100·0.2 + (Gier − 40)/100·0.15, 0.7, 1.6).
pub fn forderung(basis: f64, ego: f32, moral: f32, loyal: f32, beziehung: f32, vertrauen: f32, gier: f32) -> f64 {
    let f = 1.0 + (ego / 100.0 - 0.5) * 0.3 + (50.0 - moral) / 100.0 * 0.3 - (loyal / 100.0 - 0.5) * 0.15 - beziehung / 100.0 * 0.1 + (50.0 - vertrauen).max(0.0) / 100.0 * 0.2 + (gier - 40.0) / 100.0 * 0.15;
    basis * f.clamp(0.7, 1.6) as f64
}
/// Wish wage. Fabian 23.11.: the old value · 0.12 (werte.py:93) was half of what stars already earn, so the wish now
/// comes from the same wage curve as all wages (fair_wage in the target league) times the role, and never sinks far
/// below the current wage: who earns 16 M does not ask for 8 M.
pub fn wunschgehalt(pl: &crate::world::Player, liga: f32, rolle: u8) -> f64 {
    let r = match rolle { 0..=2 => 1.15, 3 => 1.0, 4 => 0.9, _ => 0.8 };
    (crate::world::fair_wage(pl, liga) * r).max(pl.wage * aufschlag(pl.age)).max(20000.0)
}
/// What a player wants over his current wage: young ones more, from 30 a little less, from 33 clearly less.
pub fn aufschlag(alter: u8) -> f64 { if alter <= 25 { 1.15 } else if alter <= 29 { 1.08 } else if alter <= 32 { 0.95 } else { 0.8 } }
/// Wish length (werte.py): from 0 years 3, from 24 5, from 29 3, from 32 1.
pub fn wunschlaufzeit(alter: u8) -> u32 { if alter >= 32 { 1 } else if alter >= 29 { 3 } else if alter >= 24 { 5 } else { 3 } }
/// Verdict on a renewal offer (werte.py:169): ratio ≥ 1 accept, ≥ 0.88 counter, ≥ 0.65 refuse, below insulted; veto at morale < 25 and ratio < 1.15.
pub fn bewertung(verhaeltnis: f64, moral: f32) -> &'static str {
    if moral < 25.0 && verhaeltnis < 1.15 { return crate::id("veto"); }
    if verhaeltnis >= 1.0 { crate::id("annahme") } else if verhaeltnis >= 0.88 { crate::id("gegenforderung") } else if verhaeltnis >= 0.65 { crate::id("ablehnung") } else { crate::id("beleidigt") }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runden_wie_vorlage() {
        let mut ph = Phase::neu(1_000_000.0, "fair", 0.0, 0.0);
        assert!((ph.schmerz - 910_000.0).abs() < 1.0);
        assert_eq!(ph.chance(900_000.0, false, 0, 0, 1, 0.0), 0.0, "under the pain limit never");
        assert_eq!(ph.chance(1_000_000.0, false, 0, 0, 1, 0.0), 1.0);
        // the counterpart comes closer each round
        assert!(ph.ziel_akt(2) < ph.ziel_akt(1));
        let (r, fo) = ph.senden(500_000.0, false, 0, 0, 0.0, &|_| 0.99);
        assert_eq!(r, Reaktion::Ablehnen("nicht_ernst"));
        assert_eq!(fo.stimmung, -3.0);
        assert!(ph.geduld < 90.0 - 35.0);
        let mut h = Phase::neu(1_000_000.0, "hardliner", 0.0, 0.0);
        let mut p = Phase::neu(1_000_000.0, "pragmatiker", 0.0, 0.0);
        assert!(h.chance(980_000.0, false, 0, 0, 1, 0.0) < p.chance(980_000.0, false, 0, 0, 1, 0.0), "a hardliner gives less");
        let (r, _) = h.senden(990_000.0, false, 0, 0, 0.0, &|_| 0.99);
        assert!(matches!(r, Reaktion::Gegen(_)), "{r:?}");
        let (r, _) = p.senden(1_000_000.0, false, 0, 0, 0.0, &|_| 0.99);
        assert_eq!(r, Reaktion::Annehmen(1_000_000.0));
    }
    // ports of tests/test_verhandlung_runden.py with the same expectations
    use std::cell::RefCell;
    fn mk(ch: &str, bez: f32, lage: f32) -> Phase { let mut p = Phase::neu(100.0, ch, bez, lage); p.skala = 100.0; p }
    const CH: [&str; 3] = ["pragmatiker", "fair", "hardliner"];
    #[test]
    fn nie_unter_schmerzgrenze() {
        for seed in 0..300u64 {
            let r = RefCell::new(crate::world::Rng(seed + 1));
            let mut ph = mk(CH[(seed % 3) as usize], 0.0, 0.0);
            for i in 0..6 {
                if ph.letztes_wort.is_some() { break; }
                let w = ph.schmerz - 1.0 - i as f64;
                let (re, fo) = ph.senden(w, i % 2 == 1, 0, 0, 0.0, &|_| r.borrow_mut().f());
                assert!(!matches!(re, Reaktion::Annehmen(_)));
                if matches!(re, Reaktion::Abbruch(_)) { assert!(fo.abkuehl > 0); break; }
            }
        }
    }
    #[test]
    fn ablehnung_und_nicht_ernst() {
        let mut ph = mk("hardliner", 0.0, 0.0);
        let (re, fo) = ph.senden(10.0, false, 0, 0, 0.0, &|_| 0.5);
        // Python checks the reason only: with a hardliner the patience may already be gone (then it is a breakdown)
        assert!(matches!(re, Reaktion::Ablehnen("nicht_ernst") | Reaktion::Abbruch("nicht_ernst")), "{re:?}");
        assert!(fo.stimmung < 0.0);
    }
    #[test]
    fn abbruch_wegen_geduld() {
        let r = RefCell::new(crate::world::Rng(3));
        let mut ph = mk("hardliner", 0.0, 0.0);
        let mut ende = false;
        for _ in 0..12 {
            if ph.letztes_wort.is_some() { ende = true; break; }
            let (re, fo) = ph.senden(10.0, false, 0, 0, 0.0, &|_| r.borrow_mut().f());
            if matches!(re, Reaktion::Abbruch(_)) { assert!(fo.abkuehl > 0); ende = true; break; }
        }
        assert!(ende);
    }
    #[test]
    fn letztes_angebot_je_charakter() {
        let mut quote = [0.0f32; 2];
        for (k, ch) in ["pragmatiker", "hardliner"].iter().enumerate() {
            let mut ok = 0;
            for seed in 0..400u64 {
                let r = RefCell::new(crate::world::Rng(seed + 11));
                let mut ph = mk(ch, 0.0, 0.0);
                let w = ph.schmerz + 2.0;
                let (re, fo) = ph.senden(w, true, 0, 0, 0.0, &|_| r.borrow_mut().f());
                if matches!(re, Reaktion::Annehmen(_)) { ok += 1; } else { assert_eq!(fo.abkuehl, ABKUEHL_LETZT); assert!(fo.beziehung < 0.0); }
            }
            quote[k] = ok as f32 / 400.0;
        }
        assert!(quote[0] > quote[1] + 0.3, "{quote:?}");
        let mut ph = mk("fair", 0.0, 0.0);
        assert_eq!(ph.senden(100.0, true, 0, 0, 0.0, &|_| 0.99).0, Reaktion::Annehmen(100.0));
    }
    #[test]
    fn vorschau_stimmt_mit_reaktion_im_mittel() {
        for ch in CH {
            for w in [80.0, 88.0, 94.0] {
                for (ton, h) in [(0usize, 0usize), (1, 1)] {
                    let pr = mk(ch, 1.0, 0.0).chance(w, false, ton, h, 1, 0.0);
                    let n = 1500;
                    let mut acc = 0;
                    for seed in 0..n {
                        let r = RefCell::new(crate::world::Rng(seed as u64 * 7 + 5));
                        let mut ph = mk(ch, 1.0, 0.0);
                        acc += matches!(ph.senden(w, false, ton, h, 0.0, &|_| r.borrow_mut().f()).0, Reaktion::Annehmen(_)) as u32;
                    }
                    let d = 0.05 + 0.1 * charakter(ch).6;
                    assert!((acc as f32 / n as f32 - pr).abs() <= d, "{ch} {w} {ton}: {} vs {pr}", acc as f32 / n as f32);
                }
            }
        }
    }
    #[test]
    fn viele_verhandlungen_ohne_endlosschleife() {
        for ch in CH {
            let (mut einig, n) = (0, 600);
            for seed in 0..n {
                let r = RefCell::new(crate::world::Rng(seed as u64 + 101));
                let (bez, lage) = { let mut x = r.borrow_mut(); (x.int(-2, 2) as f32, x.range(-1.0, 1.0)) };
                let mut ph = mk(ch, bez, lage);
                let start = r.borrow_mut().range(0.6, 0.9) as f64 * 100.0;
                let mut k = 0;
                loop {
                    k += 1;
                    assert!(k < 40);
                    if ph.bedenk.is_some() { let u = r.borrow_mut().f(); if matches!(ph.nach_bedenkzeit(u), Reaktion::Annehmen(_)) { einig += 1; break; } continue; }
                    if ph.letztes_wort.is_some() { einig += 1; break; }
                    let w = match ph.gegen { None => start + (k - 1) as f64 * 4.0, Some(g) => (start + k as f64 * 4.0 + g) / 2.0 };
                    match ph.senden(w.min(100.0), false, 0, 0, 0.0, &|_| r.borrow_mut().f()).0 { Reaktion::Annehmen(_) => { einig += 1; break; } Reaktion::Abbruch(_) => break, _ => {} }
                }
            }
            assert!(einig as f32 / n as f32 > 0.3, "{ch}: {einig}");
        }
    }
    #[test]
    fn vertrag_forderung_und_urteil() {
        let neutral = forderung(1_000_000.0, 50.0, 50.0, 50.0, 0.0, 50.0, 40.0);
        assert!((neutral - 1_000_000.0).abs() < 1.0);
        assert!(forderung(1_000_000.0, 80.0, 30.0, 30.0, 0.0, 20.0, 70.0) > neutral, "vain, unhappy, greedy players ask more");
        assert_eq!(bewertung(1.0, 50.0), "annahme");
        assert_eq!(bewertung(0.9, 50.0), "gegenforderung");
        assert_eq!(bewertung(0.7, 50.0), "ablehnung");
        assert_eq!(bewertung(0.5, 50.0), "beleidigt");
        assert_eq!(bewertung(1.1, 20.0), "veto");
    }
}
#[cfg(test)]
mod forderung_tests {
    /// Fabian 23.11.: who earns 16.2 M must not ask for 8.4 M; the demand follows the current wage.
    #[test]
    fn forderung_am_gehalt() {
        let w = crate::world::World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = crate::game::Career::new(w, 1, "Test");
        let mut v: Vec<usize> = (0..c.world.players.len()).filter(|&p| c.world.players[p].team >= 0).collect();
        v.sort_by(|&a, &b| c.world.players[b].wage.total_cmp(&c.world.players[a].wage));
        let mut sq = c.squad();
        sq.extend(v.into_iter().step_by(500).take(40));
        for p in sq {
            let (f, pl) = (c.forderung(p), &c.world.players[p]);
            assert!(f >= pl.wage * 0.7 && f <= pl.wage.max(50000.0) * 2.5, "{} earns {} and asks {f}", pl.name, pl.wage);
        }
    }
}
