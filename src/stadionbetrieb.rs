//! Stadium operation of the own club, ported from pesm/core/vereinswelt/wirtschaft.py: the eleven settings (prices of
//! seats, terrace, season tickets and away block, size of away block, terrace and boxes, catering, merchandise, security,
//! advertising), the demand of one home match by area, atmosphere, security risk, fines and the target of the fans' mood.
//! Numbers from vereinswelt.toml [wirtschaft] and [posten.*].
use crate::game::*;
use crate::mensch::{arr, zufall};
use crate::regeln::f;
use crate::world::*;

pub const POSTEN: [&str; 11] = [crate::id("preis_sitz"), crate::id("preis_steh"), crate::id("preis_dauer"), crate::id("preis_gast"), crate::id("gaeste"), crate::id("steh"), crate::id("vip"), crate::id("catering"), crate::id("merch"), crate::id("sicherheit"), crate::id("werbung")];
/// Index of each setting in `POSTEN`.
pub const P_SITZ: usize = 0;
pub const P_STEH: usize = 1;
pub const P_DAUER: usize = 2;
pub const P_GAST: usize = 3;
pub const GAESTE: usize = 4;
pub const STEH: usize = 5;
pub const VIP: usize = 6;
pub const CATERING: usize = 7;
pub const MERCH: usize = 8;
pub const SICHERHEIT: usize = 9;
pub const WERBUNG: usize = 10;
/// The structural settings (one-off building costs) are shown on the stadium tab, the rest on the tickets tab.
pub const BAULICH: [usize; 3] = [STEH, VIP, GAESTE];

fn v(k: &str) -> f64 { f("vereinswelt", k) as f64 }
fn a(k: &str, i: usize) -> f64 { arr("vereinswelt", k).get(i).and_then(|x| x.as_f64()).unwrap_or(0.0) }
fn lin(k: &str, x: f64) -> f64 { a(k, 0) + (a(k, 1) - a(k, 0)) * x }
fn klemme(x: f64, lo: f64, hi: f64) -> f64 { x.clamp(lo, hi) }

/// What kind of match: league, early cup round, European knockout, final.
#[derive(Clone, Copy, PartialEq)]
pub enum Art { Liga, Pokal, EuropaKo, Finale }
/// The day's circumstances of a home match (wirtschaft.py kontext).
#[derive(Clone, Copy)]
pub struct Ctx { pub gegner_ruf: f64, pub form: f64, pub wetter: usize, pub art: Art }
pub const NEUTRAL: Ctx = Ctx { gegner_ruf: 0.5, form: 7.5, wetter: 2, art: Art::Liga };

/// One home match: spectators by area (season ticket, seat, terrace, away, boxes), money in and out, mood numbers.
#[derive(Clone, Default)]
pub struct Spiel {
    pub kapazitaet: f64,
    pub bereiche: [f64; 5],
    pub zuschauer: f64,
    pub auslastung: f64,
    /// tickets, season tickets, boxes, catering, advertising
    pub ein: [f64; 5],
    /// catering costs, security, fines
    pub aus: [f64; 3],
    pub atmosphaere: f64,
    pub risiko: f64,
    pub heimvorteil: f64,
    pub stimmung_ziel: f64,
    /// seat, terrace, season ticket (per match), away, box
    pub preise: [f64; 5],
}
impl Spiel {
    pub fn netto(&self) -> f64 { self.ein.iter().sum::<f64>() - self.aus.iter().sum::<f64>() }
    pub fn sicherheit(&self) -> f64 { 100.0 - self.risiko }
}

/// Number of levels of a setting.
pub fn stufenzahl(p: usize) -> usize {
    if p == P_SITZ { return arr("vereinswelt", "einnahmen.preisstufen").len(); }
    crate::regeln::wert("vereinswelt", crate::id("posten"))[POSTEN[p]].as_object().and_then(|o| o.values().find_map(|x| x.as_array().map(|y| y.len()))).unwrap_or(1)
}
pub fn standard(p: usize) -> u8 { v(&format!("posten.{}.standard", POSTEN[p])) as u8 } // roh
fn posten(p: usize, feld: &str, stufe: u8) -> f64 { a(&format!("posten.{}.{feld}", POSTEN[p]), stufe as usize) } // roh

/// Weather of a day (0 rain or snow … 3 sun): fixed per date, worse in winter.
pub fn wetter(season: u32, day: u32) -> usize {
    let (_, m, _) = crate::date::ymd(season, day);
    let gew = arr("vereinswelt", if [11, 12, 1, 2, 3].contains(&m) { "wirtschaft.wetter_winter" } else { "wirtschaft.wetter_sommer" });
    let r = zufall(&[crate::id("wetter"), &season.to_string(), &day.to_string()]) as f64;
    let mut s = 0.0;
    for (i, g) in gew.iter().enumerate() { s += g.as_f64().unwrap_or(0.0); if r <= s { return i; } }
    gew.len() - 1
}

impl Career {
    /// The level of each setting; the seat price is the old price level (`ticket`).
    pub fn einst(&self) -> [u8; 11] {
        let mut e = [0u8; 11];
        for (p, x) in e.iter_mut().enumerate() { *x = self.wi.einst.get(p).copied().filter(|&s| s != u8::MAX).unwrap_or_else(|| standard(p)); }
        e[P_SITZ] = self.ticket;
        e
    }
    fn betrag_skala(&self) -> f64 { (0.2 + 0.8 * self.kraft()) * self.skala() }
    /// Seat price at level "normal".
    pub fn grundpreis(&self) -> f64 { (v("einnahmen.ticketpreis") + v("einnahmen.ticketpreis_je_ruf") * self.kraft()) * self.skala().sqrt() }

    /// One home match with the given settings (wirtschaft.py spiel).
    pub fn spiel_rechnen(&self, e: &[u8; 11], ctx: Ctx, stadion: u8) -> Spiel {
        let ruf = self.kraft();
        let kap = (v("verein.kapazitaet_basis") + v("verein.kapazitaet_je_ruf") * ruf) * (1.0 + v("anlagen.kapazitaet_je_stufe") * (stadion.max(1) as f64 - 1.0)) * self.neubau_faktor();
        let el = v("wirtschaft.elastizitaet");
        let sk = self.betrag_skala();
        let kopf = self.skala().sqrt(); // money per spectator scales like the ticket price
        let (steh_a, vip_a, gast_a) = (posten(STEH, "anteil", e[STEH]), posten(VIP, "anteil", e[VIP]), posten(GAESTE, "anteil", e[GAESTE])); // roh
        let sitz_a = (1.0 - steh_a - vip_a - gast_a).max(0.3);
        let fan = klemme(self.fans_eff() as f64, 0.0, 100.0);
        let f_sitz = a("einnahmen.preisstufen", e[P_SITZ] as usize);
        let f_steh = posten(P_STEH, "faktoren", e[P_STEH]); // roh
        let f_gast = posten(P_GAST, "faktoren", e[P_GAST]); // roh
        let f_dauer = posten(P_DAUER, "faktoren", e[P_DAUER]); // roh
        let ref_dauer = posten(P_DAUER, "faktoren", standard(P_DAUER)); // roh
        let wet = a("wirtschaft.wetter", ctx.wetter);
        let ext = lin("wirtschaft.gegner", ctx.gegner_ruf) * lin("wirtschaft.form", ctx.form / 15.0) * wet;
        let basis = v("einnahmen.auslastung_basis") + v("einnahmen.auslastung_je_fan") * fan / 100.0;
        let base = self.grundpreis();
        let quote = match ctx.art {
            Art::Finale => Some((v("spielart.finale"), false)),
            Art::EuropaKo => Some((v("spielart.europa_ko"), false)),
            Art::Pokal => Some((v("spielart.pokal_frueh"), false)),
            Art::Liga => (ctx.gegner_ruf >= v("spielart.topspiel_ab_gegnerruf")).then(|| (v("spielart.topspiel"), true)),
        };
        let fuellung = |atm_f: f64| -> [f64; 5] {
            let n = basis * ext * atm_f;
            let sitz_cap = kap * sitz_a;
            let dauer = sitz_cap * klemme((v("wirtschaft.dauer_basis") + v("wirtschaft.dauer_je_fan") * fan / 100.0) * (f_dauer / ref_dauer).powf(-el), 0.0, v("wirtschaft.dauer_grenze"));
            let mut z = [dauer * 0.97, (sitz_cap - dauer) * klemme(n * f_sitz.powf(-el), 0.0, 1.0), kap * steh_a * klemme(n * v("wirtschaft.steh_nachfrage") * f_steh.powf(-el), 0.0, 1.0),
                kap * gast_a * klemme((0.5 + 0.5 * ctx.gegner_ruf) * f_gast.powf(-el), 0.0, 1.0), kap * vip_a * klemme((0.25 + 0.6 * ruf + 0.2 * (fan - 50.0) / 100.0) * ext.sqrt(), 0.05, 1.0)];
            let s: f64 = z.iter().sum();
            // the kind of match sets the filling (early cup 52 %, top match at least 92 %, European knockout 98 %, final 100 %)
            if let Some((q, min)) = quote { if s > 0.0 && (!min || s < q * kap) { let k = q * kap / s; for x in z.iter_mut() { *x *= k; } } }
            z
        };
        let atmo = |ausl: f64| klemme(35.0 + 80.0 * steh_a + 25.0 * ausl + 0.25 * (fan - 50.0) + v("wirtschaft.atmosphaere_je_stadionstufe") * (stadion.max(1) as f64 - 1.0)
            + posten(WERBUNG, "atmosphaere", e[WERBUNG]) - 40.0 * vip_a - 30.0 * gast_a, 0.0, 100.0); // roh
        let z = fuellung(1.0);
        let atm = atmo(z.iter().sum::<f64>() / kap);
        // a full terrace makes more noise, and that draws spectators again
        let z = fuellung(1.0 + 0.04 * (atm - 50.0) / 50.0);
        let summe: f64 = z.iter().sum();
        let ausl = (summe / kap).min(1.0);
        let atm = atmo(ausl);
        let ein = [z[1] * base * f_sitz + z[2] * base * v("wirtschaft.steh_preis") * f_steh + z[3] * base * f_gast, z[0] * base * f_dauer, z[4] * base * v("wirtschaft.vip_preis"),
            summe * posten(CATERING, "ausgabe", e[CATERING]) * kopf, posten(WERBUNG, "je_spiel", e[WERBUNG]) * sk]; // roh
        let risiko = klemme(10.0 + 60.0 * steh_a + 30.0 * ausl + 80.0 * gast_a - posten(SICHERHEIT, "abzug", e[SICHERHEIT]), 0.0, 100.0); // roh
        let ab = v("wirtschaft.risiko_strafe_ab");
        let strafe = ((risiko - ab) / (100.0 - ab)).max(0.0) * v("wirtschaft.strafe_je_spiel") * sk;
        let aus = [summe * posten(CATERING, "kosten", e[CATERING]) * kopf, posten(SICHERHEIT, "je_spiel", e[SICHERHEIT]) * sk, strafe]; // roh
        let index = (f_sitz + f_steh) / 2.0;
        let stimmung = 0.12 * (atm - 50.0) + if index > 1.0 { -12.0 * (index - 1.0) } else { -5.0 * (index - 1.0) } + posten(CATERING, "zufriedenheit", e[CATERING]) + posten(WERBUNG, "stimmung", e[WERBUNG]); // roh
        let g = v("wirtschaft.stimmung_grenze");
        Spiel { kapazitaet: kap, bereiche: z, zuschauer: summe, auslastung: ausl, ein, aus, atmosphaere: atm.round(), risiko: risiko.round(), heimvorteil: ((atm - 50.0) * 0.08 * 10.0).round() / 10.0,
            stimmung_ziel: (klemme(stimmung, -g, g) * 10.0).round() / 10.0, preise: [base * f_sitz, base * v("wirtschaft.steh_preis") * f_steh, base * f_dauer, base * f_gast, base * v("wirtschaft.vip_preis")] }
    }
    /// An average home match with today's settings.
    pub fn spiel_neutral(&self) -> Spiel { let e = self.einst(); self.spiel_rechnen(&e, NEUTRAL, self.wi.stufen[0]) }
    /// The day's circumstances of today's home match: opponent, form of the last five, weather, kind of match.
    pub fn ctx_heute(&self) -> Ctx {
        let Some((t, i)) = self.today_match() else { return NEUTRAL };
        let fx = self.fixture((t, i));
        let gegner = if fx.h == self.own { fx.a } else { fx.h };
        let art = match t {
            Wo::Liga(_) => Art::Liga,
            Wo::Pokal(ci) => {
                let cup = &self.world.cups[ci];
                let st = cup.fx[i].stage;
                if st == 34 || cup.fx[i].neutral { Art::Finale } else if (1..=3).contains(&cup.kind) && st >= 30 { Art::EuropaKo } else { Art::Pokal }
            }
        };
        Ctx { gegner_ruf: (self.world.clubs[gegner].rep / 100.0) as f64, form: self.form_punkte(), wetter: wetter(self.world.season, self.world.day), art }
    }
    /// Points of the last five matches, scaled to five matches; without matches the neutral 7.5.
    pub fn form_punkte(&self) -> f64 {
        let l: Vec<&String> = self.results.iter().rev().take(5).collect();
        if l.is_empty() { return NEUTRAL.form; }
        l.iter().map(|r| if r.starts_with(crate::id("S")) { 3.0 } else if r.starts_with(crate::id("U")) { 1.0 } else { 0.0 }).sum::<f64>() * 5.0 / l.len() as f64
    }
    /// Books a home match: every income and cost as its own line; remembers the crowd.
    pub fn heimspiel_buchen(&mut self) {
        let e = self.einst();
        let mut s = self.spiel_rechnen(&e, self.ctx_heute(), self.wi.stufen[0]);
        // a fan boycott empties the home end: less ticket income and fewer people
        let b = self.boykott_faktor();
        if b < 1.0 { s.ein[0] *= b; s.zuschauer = (s.zuschauer as f64 * b) as _; }
        for (i, k) in [crate::id("tickets"), crate::id("dauerkarten"), crate::id("vip"), crate::id("catering"), crate::id("werbung")].iter().enumerate() { if s.ein[i] > 0.0 { self.book(crate::inhalt::t(&format!("vw.art.{k}")), s.ein[i]); } } // roh
        for (i, k) in [crate::id("cateringkosten"), crate::id("sicherheit"), crate::id("verbandsstrafe")].iter().enumerate() { if s.aus[i] > 0.0 { self.book(crate::inhalt::t(&format!("vw.art.{k}")), -s.aus[i]); } } // roh
        if s.aus[2] > 0.0 { self.grund(crate::inhalt::t("bereich.verein"), crate::inhalt::t("vw.art.verbandsstrafe"), 0.0, &crate::tf!("sb.strafe", money(s.aus[2]))); }
        self.wi.heimspiele.push((self.abs(), s.zuschauer as u32, s.auslastung as f32, s.netto()));
        if self.wi.heimspiele.len() > 40 { self.wi.heimspiele.remove(0); }
    }
    /// Merchandise of a month: (income, costs) with the range setting.
    pub fn merch_posten(&self) -> (f64, f64) {
        let e = self.einst();
        (self.merch_month() * posten(MERCH, "faktor", e[MERCH]), posten(MERCH, "monat", e[MERCH]) * self.betrag_skala()) // roh
    }
    pub fn vip_betrieb(&self) -> f64 { let e = self.einst(); posten(VIP, "monat", e[VIP]) * self.betrag_skala() } // roh
    /// Matchdays and merchandise of an average month (wirtschaft.py monat), net.
    pub fn monat_netto(&self, e: &[u8; 11], stadion: u8) -> f64 {
        let s = self.spiel_rechnen(e, NEUTRAL, stadion);
        let m = self.merch_month() * posten(MERCH, "faktor", e[MERCH]) - posten(MERCH, "monat", e[MERCH]) * self.betrag_skala(); // roh
        s.netto() * v("wirtschaft.heimspiele_je_monat") + m - posten(VIP, "monat", e[VIP]) * self.betrag_skala() // roh
    }
    /// One-off cost of switching a setting to `stufe` (rounded to 5,000).
    pub fn umbau_kosten(&self, p: usize, stufe: u8) -> f64 {
        let e = self.einst();
        let einmal = &crate::regeln::wert("vereinswelt", crate::id("posten"))[POSTEN[p]][crate::id("einmal")];
        if e[p] == stufe || einmal.is_null() { return 0.0; }
        let k = einmal[stufe as usize].as_f64().unwrap_or(0.0) * lin("wirtschaft.umbau_ruf", self.kraft()) * self.skala();
        (k / 5000.0).round() * 5000.0
    }
    /// Change of the monthly result and the match numbers before and after a switch.
    pub fn vergleich(&self, p: usize, stufe: u8) -> (f64, Spiel, Spiel) {
        let e = self.einst();
        let mut n = e;
        n[p] = stufe;
        (self.monat_netto(&n, self.wi.stufen[0]) - self.monat_netto(&e, self.wi.stufen[0]), self.spiel_rechnen(&e, NEUTRAL, self.wi.stufen[0]), self.spiel_rechnen(&n, NEUTRAL, self.wi.stufen[0]))
    }
    pub fn einst_setzen(&mut self, p: usize, stufe: u8) -> Result<f64, String> {
        if p >= POSTEN.len() || stufe as usize >= stufenzahl(p) { return Err(crate::inhalt::t("ui.wirtschaft.das_angebot_gibt_es_nicht").into()); }
        let k = self.umbau_kosten(p, stufe);
        if k > 0.0 && self.club().balance - k < -v("budget.schuldengrenze_anteil") * self.umsatz_prognose() { return Err(crate::inhalt::t("tv.drama.rueckmeldung.budget").into()); }
        if k > 0.0 { self.book(crate::inhalt::t("vw.art.umbau"), -k); }
        if p == P_SITZ { self.ticket = stufe; } else {
            if self.wi.einst.len() < POSTEN.len() { self.wi.einst.resize(POSTEN.len(), u8::MAX); }
            self.wi.einst[p] = stufe;
        }
        Ok(k)
    }
}

/// A 0–100 number as a word of the game (wirtschaft.py wort): `grenzen` are the lower bounds from level 1.
pub fn wort(art: &str, wert: f64, grenzen: &[f64]) -> &'static str { crate::inhalt::t(&format!("vw.wi.w.{art}.{}", grenzen.iter().filter(|&&g| wert >= g).count())) } // roh

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heimspiel_wie_vorlage() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.wirtschaft_an();
        let e = c.einst();
        let s = c.spiel_rechnen(&e, NEUTRAL, 1);
        assert!(s.zuschauer > 0.0 && s.zuschauer <= s.kapazitaet * 1.0001, "{} / {}", s.zuschauer, s.kapazitaet);
        // dearer seats: fewer people, unhappier fans
        let mut d = e;
        d[P_SITZ] = 2;
        let t = c.spiel_rechnen(&d, NEUTRAL, 1);
        assert!(t.bereiche[1] < s.bereiche[1] && t.stimmung_ziel < s.stimmung_ziel);
        // a big terrace: more atmosphere, more risk
        let mut d = e;
        d[STEH] = 3;
        let t = c.spiel_rechnen(&d, NEUTRAL, 1);
        assert!(t.atmosphaere > s.atmosphaere && t.risiko > s.risiko);
        // the final is sold out
        let f = c.spiel_rechnen(&e, Ctx { art: Art::Finale, ..NEUTRAL }, 1);
        assert!((f.zuschauer / f.kapazitaet - 1.0).abs() < 0.01);
        // switching a building setting costs money once, prices cost nothing
        assert!(c.umbau_kosten(VIP, 2) > 0.0 && c.umbau_kosten(P_STEH, 0) == 0.0);
        let k0 = c.club().balance;
        c.einst_setzen(VIP, 1).unwrap();
        assert!(c.club().balance < k0 && c.einst()[VIP] == 1);
        assert_eq!(stufenzahl(P_SITZ), 3);
        assert_eq!(stufenzahl(P_STEH), 5);
    }
}
