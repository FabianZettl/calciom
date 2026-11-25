//! The economy as cases in the inbox, ported from pesm/core/wirtschaft/vorgaenge.py: stadium expansion when the ground is
//! full, sponsor renegotiation (on the sponsors page), shirt launch and the shop's slump after a star sale, fan protest
//! and the monthly finance report. Every roll is fixed per case key (same situation, same outcome).
use crate::game::*;
use crate::inhalt::t;
use crate::mensch::zufall;
use crate::regeln::f;
use crate::world::*;

fn v(k: &str) -> f64 { f("vereinswelt", k) as f64 }
fn runden(x: f64, auf: f64) -> f64 { (x / auf).round() * auf }
/// Roll in 0…100, fixed per key.
fn wurf(key: &str) -> f64 { zufall(&[crate::id("wv"), key]) as f64 * 100.0 }
/// A chance as a word (never the number).
pub fn aussicht(p: f64) -> &'static str { t(if p >= 85.0 { "vhd.wort.sehr_gut" } else if p >= 60.0 { "vhd.wort.gut" } else if p >= 35.0 { "vhd.wort.offen" } else if p >= 15.0 { "vhd.wort.gering" } else { "vhd.wort.kaum" }) }
/// Linear between the points (x rising), constant outside (transfers/chance.py _stueck).
fn stueck(x: f64, p: &[(f64, f64)]) -> f64 {
    if x <= p[0].0 { return p[0].1; }
    for w in p.windows(2) { if x <= w[1].0 { return w[0].1 + (w[1].1 - w[0].1) * (x - w[0].0) / (w[1].0 - w[0].0); } }
    p[p.len() - 1].1
}
// KONFIG of vorgaenge.py (start values, not calibrated)
const AUSLASTUNG_AB: f64 = 85.0;
const WARTEN_AB: f64 = 95.0;
const SPARSAM_KOSTEN: f64 = 0.8;
const SPARSAM_DAUER: f64 = 1.4;
const UEBER_VOLL: (f64, f64) = (15.0, 0.20);
const UEBER_SPARSAM: (f64, f64) = (40.0, 0.25);
pub const AUFSCHLAG: [u32; 2] = [10, 20];
const TRIKOT_MONATE: [u32; 3] = [7, 8, 9];
const MERCH_DAUER: u32 = 3;
const GROSS: (f64, f64) = (0.7, 0.40);
const DEZENT: (f64, f64) = (0.2, 0.15);
const VERKAUF_AB: f64 = 0.04;
const ERSATZ_KOSTEN: f64 = 0.3;
const ERSATZ_REST: f64 = 0.4;
const DIALOG_KOSTEN: f64 = 25_000.0;
const SENKEN_STIMMUNG: f32 = 3.0;
const HALTEN_STIMMUNG: f32 = -3.0;
const DIALOG_STIMMUNG: f32 = 4.0;
const PREIS_STIMMUNG: f32 = -2.0;
const WERBUNG_STIMMUNG: f32 = -1.0;
const GRENZE_STIMMUNG: f32 = 15.0;

impl Career {
    /// A case key is new once; true the first time.
    fn wv_neu(&mut self, key: &str) -> bool {
        if self.wi.wv.iter().any(|k| k == key) { return false; }
        self.wi.wv.push(key.to_string());
        if self.wi.wv.len() > 200 { self.wi.wv.remove(0); }
        true
    }
    fn leistbar(&self, kosten: f64) -> bool { self.club().balance - kosten >= -v("budget.schuldengrenze_anteil") * self.umsatz_prognose() }
    /// The fans' mood of the cases: signed points that fade with the monthly fan step (fan_aktion).
    fn wv_stimmung(&mut self, d: f32, grund: &str) {
        if d == 0.0 { return; }
        self.me.fan_aktion = (self.me.fan_aktion + d).clamp(-GRENZE_STIMMUNG, GRENZE_STIMMUNG);
        self.grund(crate::id("Fans"), t("story.groesse.fanstimmung"), d, grund);
    }
    fn umbau_skala(&self) -> f64 { let a = crate::mensch::arr("vereinswelt", "wirtschaft.umbau_ruf"); let (x, y) = (a[0].as_f64().unwrap_or(0.4), a[1].as_f64().unwrap_or(1.6)); (x + (y - x) * self.kraft()) * self.skala() }
    /// Average filling of the last five home matches in percent (the neutral forecast before the first one).
    pub fn auslastung_schnitt(&self) -> f64 {
        let h: Vec<f32> = self.wi.heimspiele.iter().rev().take(5).map(|x| x.2).collect();
        if h.is_empty() { self.spiel_neutral().auslastung * 100.0 } else { h.iter().sum::<f32>() as f64 / h.len() as f64 * 100.0 }
    }

    /// Monthly: the cases that the situation opens (stadium, shirt launch, protest, report) and the planned bookings.
    pub fn wv_monat(&mut self, einnahmen: f64, ausgaben: f64) {
        let monat = self.abs() / 30;
        let plan: Vec<f64> = self.wi.wv_plan.iter().filter(|p| p.0 <= monat).map(|p| p.1).collect();
        self.wi.wv_plan.retain(|p| p.0 > monat);
        for b in plan { self.book(t("vw.art.merch_kampagne"), b); }
        self.wv_stadion();
        self.investor_pruefen();
        self.wv_trikot();
        self.wv_protest();
        self.wv_bericht(einnahmen, ausgaben);
    }

    // ---------- 1. stadium
    fn wv_stadion(&mut self) {
        let st = self.wi.stufen[0];
        if st as f32 >= f("vereinswelt", "anlagen.stufen_max") || self.wi.bau.iter().any(|b| b.0 == 0) || self.wi.heimspiele.len() < 3 { return; }
        let ausl = self.auslastung_schnitt();
        if ausl < AUSLASTUNG_AB { return; }
        let key = format!("stadion|{}|{}", self.world.season, st + 1); // roh
        if !self.wv_neu(&key) { return; }
        let kosten = self.ausbau_kosten(0);
        let dauer = v("anlagen.dauer.stadion");
        let kap0 = self.kapazitaet();
        let kap1 = kap0 / (1.0 + v("anlagen.kapazitaet_je_stufe") * (st.max(1) as f64 - 1.0)) * (1.0 + v("anlagen.kapazitaet_je_stufe") * st as f64);
        let sp_kosten = runden(kosten * SPARSAM_KOSTEN, 10_000.0);
        let sp_dauer = (dauer * SPARSAM_DAUER).round();
        let opts = vec![
            (crate::id("voll").to_string(), format!("{} ({}, {}, {})", t("wv.stadion.opt.voll"), crate::tf!("wv.folge.kosten", money(kosten)), crate::tf!("wv.folge.bauzeit", dauer), crate::tf!("wv.folge.risiko", UEBER_VOLL.0, (UEBER_VOLL.1 * 100.0) as i32))),
            (crate::id("sparsam").to_string(), format!("{} ({}, {}, {})", t("wv.stadion.opt.sparsam"), crate::tf!("wv.folge.kosten", money(sp_kosten)), crate::tf!("wv.folge.bauzeit", sp_dauer), crate::tf!("wv.folge.risiko", UEBER_SPARSAM.0, (UEBER_SPARSAM.1 * 100.0) as i32))),
            (crate::id("warten").to_string(), t("wv.stadion.opt.warten").to_string()),
        ];
        let text = crate::tf!("wv.stadion.text", format!("{ausl:.0}"), format!("{kap0:.0}"), format!("{kap1:.0}"), st + 1);
        self.msg(1, t("vw.figur.praesident"), &crate::tf!("wv.stadion.titel", st + 1), &text, Action::Wahl { art: format!("wv:{key}"), bezug: vec![], optionen: opts }); // roh
    }
    fn wv_stadion_antwort(&mut self, key: &str, o: &str) -> String {
        if o == crate::id("warten") {
            if self.auslastung_schnitt() >= WARTEN_AB { self.wv_stimmung(-2.0, t("wv.folge.fans.warteliste")); }
            return t("wv.stadion.ergebnis.warten").into();
        }
        if self.wi.bau.iter().any(|b| b.0 == 0) { return t("ui.wirtschaft.hier_wird_schon_gebaut").into(); }
        let sparsam = o == crate::id("sparsam");
        let (p, ueber) = if sparsam { UEBER_SPARSAM } else { UEBER_VOLL };
        let mut kosten = if sparsam { runden(self.ausbau_kosten(0) * SPARSAM_KOSTEN, 10_000.0) } else { self.ausbau_kosten(0) };
        let monate = v("anlagen.dauer.stadion") * if sparsam { SPARSAM_DAUER } else { 1.0 };
        let teurer = wurf(&format!("{key}|{o}")) < p;
        if teurer { kosten = runden(kosten * (1.0 + ueber), 10_000.0); }
        if !self.leistbar(kosten) { return t("tv.drama.rueckmeldung.budget").into(); }
        let z = self.wi.stufen[0] + 1;
        let fertig = self.abs() + (monate.round() * 30.4) as u32;
        self.book(t("vw.art.ausbau"), -kosten);
        self.wi.bau.push((0, z, fertig));
        if sparsam { self.wv_stimmung(-1.5, t("wv.folge.fans.baustelle")); }
        let sd = crate::date::SEASON_DAYS;
        crate::tf!(if teurer { "wv.stadion.ergebnis.teurer" } else { "wv.stadion.ergebnis.ok" }, money(kosten), crate::date::long(fertig / sd, fertig % sd))
    }

    // ---------- 2. sponsors (renegotiation from the sponsors page)
    /// Chance in percent that the sponsor goes along with `aufschlag` percent more (vorgaenge.py sponsor_chance).
    pub fn sponsor_chance(&self, i: usize, aufschlag: u32) -> f64 {
        let Some(s) = self.wi.angebote.get(i) else { return 0.0 };
        let markt = v(&format!("sponsor.{}", crate::wirtschaft::SPONSOR_ARTEN[s.art as usize])) * (1.0 + v("sponsor.je_ruf") * self.kraft()) * self.skala(); // roh
        let verh = s.jahr * (1.0 + aufschlag as f64 / 100.0) / markt.max(1.0);
        let cv = stueck(verh, &[(0.9, 92.0), (1.1, 65.0), (1.3, 30.0), (1.6, 6.0), (2.0, 0.0)]) + (self.media as f64 - 50.0) * 0.2 + (self.fans_eff() as f64 - 50.0) * 0.1;
        cv.clamp(0.0, 100.0).round()
    }
    /// One try per offer: success raises the amount (the offer stays open), failure withdraws it.
    pub fn sponsor_verhandeln(&mut self, i: usize, aufschlag: u32) -> Result<(bool, String), String> {
        let Some(s) = self.wi.angebote.get(i).cloned() else { return Err(t("ui.wirtschaft.das_angebot_gibt_es_nicht").into()) };
        if s.versucht || !AUFSCHLAG.contains(&aufschlag) { return Err(t("ui.wirtschaft.das_angebot_gibt_es_nicht").into()); }
        let c = self.sponsor_chance(i, aufschlag);
        let erfolg = wurf(&format!("sponsor|{}|{}|{}|{aufschlag}", s.name, s.art, s.frist)) < c; // roh
        if erfolg {
            let neu = runden(s.jahr * (1.0 + aufschlag as f64 / 100.0), 10_000.0);
            let o = &mut self.wi.angebote[i];
            o.bonus = runden(o.bonus * neu / o.jahr.max(1.0), 1_000.0);
            o.jahr = neu;
            o.versucht = true;
            Ok((true, crate::tf!("wv.sponsor.ergebnis.verhandelt_ja", s.name, money(neu))))
        } else {
            self.wi.angebote.remove(i);
            Ok((false, crate::tf!("wv.sponsor.ergebnis.verhandelt_nein", s.name)))
        }
    }

    // ---------- 3. merchandise
    fn wv_trikot(&mut self) {
        let (_, m, _) = crate::date::ymd(self.world.season, self.world.day);
        if !TRIKOT_MONATE.contains(&m) { return; }
        let key = format!("merch|trikot|{}", self.world.season); // roh
        if !self.wv_neu(&key) { return; }
        let mm = self.merch_posten().0;
        let fak = 0.6 + 0.8 * self.fans_eff() as f64 / 100.0;
        let opt = |o: &str, (k, g): (f64, f64)| { let gew = mm * g * fak * MERCH_DAUER as f64; let kosten = runden(mm * k, 5000.0); (o.to_string(), format!("{} ({}, {}, {})", t(&format!("wv.merch.trikot.opt.{o}")), crate::tf!("wv.folge.kosten", money(kosten)), crate::tf!("wv.folge.merch.erwartet", money(gew), MERCH_DAUER), crate::tf!("wv.folge.saldo", money(gew - kosten)))) }; // roh
        let opts = vec![opt(crate::id("gross"), GROSS), opt(crate::id("dezent"), DEZENT), (crate::id("auslassen").to_string(), t("wv.merch.trikot.opt.auslassen").to_string())];
        self.msg(3, t("vw.figur.finanz"), t("wv.merch.trikot.titel"), &crate::tf!("wv.merch.trikot.text", money(mm)), Action::Wahl { art: format!("wv:{key}"), bezug: vec![], optionen: opts }); // roh
    }
    /// After a sale: the shop loses the shirts of that name (from 4 % of a year's income).
    pub fn wv_verkauf(&mut self, fee: f64, name: &str) {
        let jahr = self.umsatz_prognose().max(1.0);
        if fee / jahr < VERKAUF_AB { return; }
        let key = format!("merch|verkauf|{}|{}", self.abs(), name); // roh
        if !self.wv_neu(&key) { return; }
        let mm = self.merch_posten().0;
        let anteil = (fee / jahr * 1.5).clamp(0.05, 0.25);
        let verlust = mm * anteil * 2.0;
        let ersatz = runden(mm * ERSATZ_KOSTEN, 5000.0);
        let opts = vec![
            (crate::id("ersatz").to_string(), format!("{} ({}, {})", t("wv.merch.verkauf.opt.ersatz"), crate::tf!("wv.folge.kosten", money(ersatz)), crate::tf!("wv.folge.merch.verlust", money(verlust * ERSATZ_REST)))),
            (crate::id("aussitzen").to_string(), format!("{} ({})", t("wv.merch.verkauf.opt.aussitzen"), crate::tf!("wv.folge.merch.verlust", money(verlust)))),
        ];
        self.msg(3, t("vw.figur.finanz"), t("wv.merch.verkauf.titel"), &crate::tf!("wv.merch.verkauf.text", money(fee), format!("{:.0}", anteil * 100.0)), Action::Wahl { art: format!("wv:{key}|{verlust:.0}|{ersatz:.0}"), bezug: vec![], optionen: opts }); // roh
    }
    fn wv_merch_antwort(&mut self, teile: &[&str], o: &str) -> String {
        let monat = self.abs() / 30;
        if teile.get(1) == Some(&crate::id("trikot")) {
            if o == crate::id("auslassen") { return t("wv.merch.ergebnis.auslassen").into(); }
            let (k, g) = if o == crate::id("gross") { GROSS } else { DEZENT };
            let mm = self.merch_posten().0;
            let kosten = runden(mm * k, 5000.0);
            if !self.leistbar(kosten) { return t("tv.drama.rueckmeldung.budget").into(); }
            let gew = mm * g * (0.6 + 0.8 * self.fans_eff() as f64 / 100.0) * MERCH_DAUER as f64;
            self.book(t("vw.art.merch_kampagne"), -kosten);
            for n in 1..=MERCH_DAUER { self.wi.wv_plan.push((monat + n, gew / MERCH_DAUER as f64)); }
            return crate::tf!("wv.merch.ergebnis.trikot", money(gew), MERCH_DAUER);
        }
        let n = teile.len();
        let verlust: f64 = teile.get(n.saturating_sub(2)).and_then(|x| x.parse().ok()).unwrap_or(0.0);
        let ersatz: f64 = teile.last().and_then(|x| x.parse().ok()).unwrap_or(0.0);
        let rest = if o == crate::id("ersatz") {
            if !self.leistbar(ersatz) { return t("tv.drama.rueckmeldung.budget").into(); }
            self.book(t("vw.art.merch_kampagne"), -ersatz);
            verlust * ERSATZ_REST
        } else { verlust };
        // the slump spreads over two months
        for k in 1..=2 { self.wi.wv_plan.push((monat + k, -rest / 2.0)); }
        crate::tf!(if o == crate::id("ersatz") { "wv.merch.ergebnis.ersatz" } else { "wv.merch.ergebnis.aussitzen" }, money(rest))
    }

    // ---------- 4. ticket prices / fan protest
    fn wv_protest(&mut self) {
        let st = self.fans_eff();
        if st >= f("vereinswelt", "fans.protest_unter") { return; }
        let key = format!("protest|{}|{}", self.world.season, self.abs() / 30); // roh
        if !self.wv_neu(&key) { return; }
        let preis = self.ticket;
        let ursache = if preis >= 2 { crate::id("preis") } else { crate::id("lage") };
        let schwer = st < f("vereinswelt", "fans.unruhe_unter");
        let mut opts = vec![];
        if preis > 0 {
            let (d, _, _) = self.vergleich(crate::stadionbetrieb::P_SITZ, preis - 1);
            opts.push((crate::id("preise_senken").to_string(), format!("{} ({})", t("wv.protest.opt.preise_senken"), crate::tf!("wv.folge.einnahmen.monat", money(d)))));
        }
        let kosten = runden(DIALOG_KOSTEN * self.umbau_skala(), 5000.0);
        let c = self.dialog_chance(schwer);
        opts.push((crate::id("dialog").to_string(), format!("{} ({}, {})", t("wv.protest.opt.dialog"), crate::tf!("wv.folge.kosten", money(kosten)), crate::tf!("wv.folge.chance", aussicht(c)))));
        opts.push((crate::id("halten").to_string(), format!("{} ({})", t("wv.protest.opt.halten"), t("wv.folge.fans.runter"))));
        let gp = self.spiel_neutral().preise[0];
        let vk = self.vkey(&format!("wv.protest.text.{ursache}"), "");
        self.msg(if schwer { 0 } else { 1 }, t("wv.figur.fans"), t(&format!("wv.protest.titel.{ursache}")), &crate::tf!(&vk, format!("{st:.0}"), money(gp)), Action::Wahl { art: format!("wv:{key}|{kosten:.0}|{}", schwer as u8), bezug: vec![], optionen: opts }); // roh
    }
    fn dialog_chance(&self, schwer: bool) -> f64 { (45.0 + self.mensch.vorstand as f64 * 0.3 + if self.ticket < 2 { 15.0 } else { 0.0 } - if schwer { 15.0 } else { 0.0 }).clamp(0.0, 100.0) }
    fn wv_protest_antwort(&mut self, key: &str, teile: &[&str], o: &str) -> String {
        if o == crate::id("preise_senken") {
            if self.ticket == 0 { return t("wv.folge.nichts").into(); }
            let (d, _, _) = self.vergleich(crate::stadionbetrieb::P_SITZ, self.ticket - 1);
            self.ticket -= 1;
            self.wv_stimmung(SENKEN_STIMMUNG, t("wv.protest.opt.preise_senken"));
            return crate::tf!("wv.protest.ergebnis.preise_senken", money(d.abs()));
        }
        if o == crate::id("dialog") {
            let kosten: f64 = teile.get(teile.len().saturating_sub(2)).and_then(|x| x.parse().ok()).unwrap_or(0.0);
            let schwer = teile.last() == Some(&"1");
            if !self.leistbar(kosten) { return t("tv.drama.rueckmeldung.budget").into(); }
            self.book(t("vw.art.fanaktion"), -kosten);
            let erfolg = wurf(&format!("{key}|dialog")) < self.dialog_chance(schwer); // roh
            if erfolg { self.wv_stimmung(DIALOG_STIMMUNG, t("wv.protest.opt.dialog")); } else { self.klima(0.0, -3.0, 0.0, t("wv.protest.opt.dialog")); }
            return t(if erfolg { "wv.protest.ergebnis.dialog_ja" } else { "wv.protest.ergebnis.dialog_nein" }).into();
        }
        self.wv_stimmung(HALTEN_STIMMUNG, t("wv.protest.opt.halten"));
        self.klima(0.0, -2.0, 0.0, t("wv.protest.opt.halten"));
        t("wv.protest.ergebnis.halten").into()
    }

    // ---------- 5. finance report
    fn wv_bericht(&mut self, ein: f64, aus: f64) {
        let kasse = self.club().balance;
        let saldo = ein - aus;
        let grenze = v("budget.schuldengrenze_anteil") * self.umsatz_prognose();
        let warnung = if kasse < -grenze { crate::id("gesperrt") } else if kasse < 0.0 { crate::id("schulden") } else if kasse < aus * 0.5 { crate::id("knapp") } else { "" };
        let (_, m, _) = crate::date::ymd(self.world.season, self.world.day.saturating_sub(1));
        let mname = crate::date::month_name(m);
        let mut text = crate::tf!("wv.bericht.text", mname, money(ein), money(aus), money(saldo));
        if let Some(vor) = self.wi.wv_vormonat { text = format!("{text} {}: {}{}.", t("wv.k.vormonat"), if saldo - vor >= 0.0 { "+" } else { "" }, money(saldo - vor)); } // roh
        self.wi.wv_vormonat = Some(saldo);
        let titel = crate::tf!("wv.bericht.titel", mname);
        if warnung.is_empty() { self.msg(3, t("vw.figur.finanz"), &titel, &text, Action::None); return; }
        text = format!("{text} {}", crate::tf!(&format!("wv.bericht.warnung.{warnung}"), money(kasse), money(grenze))); // roh
        let e = self.einst();
        let mut opts = vec![];
        for (p, was) in [(crate::stadionbetrieb::P_SITZ, crate::id("preise_anheben")), (crate::stadionbetrieb::WERBUNG, crate::id("werbung_mehr"))] {
            if (e[p] as usize) + 1 < crate::stadionbetrieb::stufenzahl(p) {
                let (d, _, _) = self.vergleich(p, e[p] + 1);
                opts.push((was.to_string(), format!("{} ({}, {})", t(&format!("wv.bericht.opt.{was}")), crate::tf!("wv.folge.einnahmen.monat", money(d)), t("wv.folge.fans.runter")))); // roh
            }
        }
        opts.push((crate::id("kenntnis").to_string(), t("wv.bericht.opt.kenntnis").to_string()));
        self.msg(if warnung == crate::id("knapp") { 1 } else { 0 }, t("vw.figur.finanz"), &titel, &text, Action::Wahl { art: format!("wv:bericht|{}", self.abs()), bezug: vec![], optionen: opts }); // roh
    }
    fn wv_bericht_antwort(&mut self, o: &str) -> String {
        let e = self.einst();
        let (p, st) = match o { "preise_anheben" => (crate::stadionbetrieb::P_SITZ, PREIS_STIMMUNG), "werbung_mehr" => (crate::stadionbetrieb::WERBUNG, WERBUNG_STIMMUNG), _ => return t("wv.bericht.ergebnis.kenntnis").into() }; // roh
        if let Err(x) = self.einst_setzen(p, e[p] + 1) { return x; }
        self.wv_stimmung(st, t(&format!("wv.bericht.opt.{o}"))); // roh
        t(&format!("wv.bericht.ergebnis.{o}")).into() // roh
    }

    // ---------- sponsor requests and the investor (Fabian 24.11.: the pyramid lives)
    /// A new sponsor offer as an inbox case: accept, ask for more, or leave it for later (it stays on the sponsors page).
    pub fn sponsor_anfrage_melden(&mut self, i: usize) {
        if !self.feature("sponsoren") { return; } // roh
        let Some(o) = self.wi.angebote.get(i).cloned() else { return };
        let art = crate::wirtschaft::SPONSOR_ARTEN[o.art as usize];
        let jahre = o.bis + 1 - self.world.season.min(o.bis);
        let vk = self.vkey(&format!("ui.sponsor.anfrage.text.{art}"), &o.name);
        let text = crate::tf!(&vk, o.name.clone(), money(o.jahr), jahre, money(o.bonus));
        let opts = vec![(crate::id("annehmen").to_string(), crate::tf!("ui.sponsor.opt.annehmen", money(o.jahr))), (crate::id("mehr").to_string(), crate::tf!("ui.sponsor.opt.mehr", crate::wirtschaftsvorgaenge::aussicht(self.sponsor_chance(i, 10)))), (crate::id("spaeter").to_string(), t("ui.sponsor.opt.spaeter").to_string())];
        self.msg(3, t("ui.wirtschaft.marketing"), &crate::tf!(&format!("ui.sponsor.anfrage.titel.{art}"), o.name.clone()), &text, Action::Wahl { art: format!("wv:sponsor|{}", o.name), bezug: vec![], optionen: opts }); // roh
        self.meldung_art(crate::pages::Art::Angebot, vec![]);
    }
    fn wv_sponsor_antwort(&mut self, name: &str, o: &str) -> String {
        let Some(i) = self.wi.angebote.iter().position(|x| x.name == name) else { return t("ui.wirtschaft.das_angebot_gibt_es_nicht").into() };
        match o {
            "annehmen" => match self.sponsor_annehmen(i) { Ok(()) => t("ui.pages.vertrag_unterschrieben").into(), Err(e) => e },
            "mehr" => match self.sponsor_verhandeln(i, 10) { Ok((_, txt)) => txt, Err(e) => e },
            _ => t("ui.sponsor.bleibt_offen").into(),
        }
    }
    /// Now and then an investor asks to come in (rarely, more often for a big or a struggling club); German clubs only
    /// up to 49 % (50+1).
    pub fn investor_pruefen(&mut self) {
        if !self.feature("sponsoren") { return; } // roh
        if self.wi.investor.is_some() { return; }
        let abs = self.abs();
        if self.wi.investor_frage > 0 && abs < self.wi.investor_frage + v("investor.pause_tage") as u32 { return; }
        if abs < v("investor.ab_tag") as u32 { return; }
        let monat = abs / 30;
        let not = self.club().balance < 0.0;
        let chance = v("investor.chance_monat") * (0.4 + self.kraft() + if not { 0.6 } else { 0.0 });
        if zufall(&[crate::id("investor"), &self.club().id.to_string(), &monat.to_string()]) as f64 > chance { return; }
        self.wi.investor_frage = abs;
        let z = |x: &str| zufall(&[crate::id("investor"), x, &monat.to_string(), &self.club().id.to_string()]) as f64;
        let typ = (z(crate::id("typ")) * 4.0) as u8 % 4;
        let namen = crate::regeln::wert_oder("vereinswelt", &format!("investor.namen.t{typ}")).and_then(|x| x.as_array().cloned()).unwrap_or_default(); // roh
        let name = namen.get((z("name") * namen.len().max(1) as f64) as usize % namen.len().max(1)).and_then(|x| x.as_str()).unwrap_or("Capital Partners").to_string(); // roh
        let deutsch = self.world.leagues[self.league()].land == crate::id("Deutschland");
        let anteil = if deutsch { 49 } else { [51u8, 75, 100][(z(crate::id("anteil")) * 3.0) as usize % 3] };
        let summe = round_money(self.umsatz_prognose() * v(&format!("investor.faktor_t{typ}")) * (0.8 + 0.4 * z(crate::id("summe"))) * anteil as f64 / 100.0);
        let inv = crate::wirtschaft::Investor { name: name.clone(), typ, summe, anteil, seit: abs };
        let vk = self.vkey(&format!("ui.investor.text.t{typ}"), &name);
        let text = format!("{} {}", crate::tf!(&vk, name.clone(), money(summe), anteil), t(if deutsch { "ui.investor.fuenfzig_plus_eins" } else { "ui.investor.folgen" }));
        let opts = vec![(crate::id("annehmen").to_string(), crate::tf!("ui.investor.opt.annehmen", money(summe))), (crate::id("mehr").to_string(), t("ui.investor.opt.mehr").to_string()), (crate::id("ablehnen").to_string(), t("ui.investor.opt.ablehnen").to_string())];
        let key = format!("investor|{}|{}|{}|{}|{}", typ, summe, anteil, abs, name); // roh
        self.msg(1, t("vw.figur.praesident"), &crate::tf!("ui.investor.titel", name), &text, Action::Wahl { art: format!("wv:{key}"), bezug: vec![], optionen: opts }); // roh
        self.meldung_art(crate::pages::Art::Angebot, vec![]);
    }
    fn wv_investor_antwort(&mut self, teile: &[&str], o: &str) -> String {
        let (Some(typ), Some(summe), Some(anteil), Some(seit), Some(name)) = (teile.get(1).and_then(|x| x.parse::<u8>().ok()), teile.get(2).and_then(|x| x.parse::<f64>().ok()), teile.get(3).and_then(|x| x.parse::<u8>().ok()), teile.get(4).and_then(|x| x.parse::<u32>().ok()), teile.get(5)) else { return t("ui.akademie.notiert").into() };
        if self.wi.investor.is_some() { return t("ui.investor.schon").into(); }
        let mut summe = summe;
        if o == "mehr" {
            // more money: a fixed roll; a no ends the talks
            if wurf(&format!("investor|mehr|{seit}")) > v("investor.mehr_chance") * 100.0 { self.news(&crate::tf!("ui.investor.news.weg_titel", name.to_string()), &crate::tf!("ui.investor.news.weg", name.to_string())); return crate::tf!("ui.investor.weg", name.to_string()); }
            summe = round_money(summe * 1.15);
        }
        if o == "ablehnen" {
            let fans = if typ <= 1 { v("investor.ablehnen_fans") as f32 } else { 1.0 };
            self.klima(fans, 1.0, -v("investor.ablehnen_vorstand") as f32, &crate::tf!("ui.investor.grund_abgelehnt", name.to_string()));
            return crate::tf!("ui.investor.abgelehnt", name.to_string());
        }
        // in: the money, higher goals, the fans react by kind (a state fund splits them, a local entrepreneur is welcome)
        let club = self.own;
        self.world.clubs[club].balance += summe;
        self.transfer_budget += round_money(summe * v("investor.transfer_anteil"));
        let ziel = self.world.clubs[club].goal_pos;
        self.world.clubs[club].goal_pos = ziel.saturating_sub(v("investor.ziel_besser") as usize).max(1);
        let fans = [-6.0f32, -4.0, -2.0, 3.0][typ.min(3) as usize];
        self.klima(fans, 2.0, 6.0, &crate::tf!("ui.investor.grund_eingestiegen", name.to_string()));
        self.wi.investor = Some(crate::wirtschaft::Investor { name: name.to_string(), typ, summe, anteil, seit: self.abs() });
        self.news(&crate::tf!("ui.investor.news.titel", name.to_string(), self.club().name.clone()), &crate::tf!(&format!("ui.investor.news.t{typ}"), name.to_string(), money(summe), anteil));
        crate::tf!("ui.investor.eingestiegen", name.to_string(), money(summe), self.world.clubs[club].goal_pos)
    }
    /// The answer to an economy case (mensch.rs wahl, art „wv:<key>“).
    pub fn wv_antwort(&mut self, key: &str, o: &str) -> String {
        let teile: Vec<&str> = key.split('|').collect();
        match teile[0] {
            "stadion" => self.wv_stadion_antwort(key, o),
            "merch" => self.wv_merch_antwort(&teile, o),
            "protest" => self.wv_protest_antwort(key, &teile, o),
            "bericht" => self.wv_bericht_antwort(o),
            "sponsor" => self.wv_sponsor_antwort(teile.get(1).copied().unwrap_or(""), o),
            "investor" => self.wv_investor_antwort(&teile, o),
            _ => t("ui.akademie.notiert").into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn karriere() -> Career {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.wirtschaft_an();
        c
    }
    /// Fabian 24.11.: the sponsor pyramid fills its free places through inbox requests; an investor comes in with money
    /// and higher goals.
    #[test]
    fn pyramide_und_investor() {
        let mut c = karriere();
        let frei = |c: &Career| 4 - c.wi.vertraege.iter().filter(|s| s.art == 4).count();
        assert_eq!(frei(&c), 2);
        for _ in 0..12 { c.world.day += 30; c.wirtschaft_monat(); if c.wi.angebote.iter().any(|o| o.art == 4) { break; } }
        let i = c.inbox.iter().rposition(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("wv:sponsor|"))).expect("sponsor request in the inbox");
        let Action::Wahl { art, .. } = c.inbox[i].action.clone() else { unreachable!() };
        let name = art.trim_start_matches("wv:sponsor|").to_string();
        let a = c.wi.angebote.iter().find(|o| o.name == name).unwrap().art;
        let vorher = c.wi.vertraege.iter().filter(|s| s.art == a).count();
        c.wahl(&art, &[], "annehmen");
        assert_eq!(c.wi.vertraege.iter().filter(|s| s.art == a).count(), vorher + 1);
        assert!(c.wi.vertraege.iter().filter(|s| s.art == a).count() <= crate::wirtschaft::plaetze(a));
        // the investor: money in, goal one or two places higher
        let (geld, ziel) = (c.club().balance, c.club().goal_pos);
        c.wahl("wv:investor|3|5000000|51|200|Familie Brenner Holding", &[], "annehmen");
        assert!(c.wi.investor.is_some() && c.club().balance >= geld + 5_000_000.0 - 1.0);
        assert!(c.club().goal_pos < ziel || ziel == 1);
    }
    #[test]
    fn stadion_voll_bringt_ausbaufrage() {
        let mut c = karriere();
        for k in 0..5 { c.wi.heimspiele.push((k, 50_000, 0.97, 0.0)); }
        c.wv_stadion();
        let i = c.inbox.iter().position(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("wv:stadion"))).expect("Ausbaufrage");
        // asked once per level and season
        c.wv_stadion();
        assert_eq!(c.inbox.iter().filter(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("wv:stadion"))).count(), 1);
        c.world.clubs[c.own].balance = 1e9;
        let k0 = c.club().balance;
        c.answer(i, true, 0);
        assert!(c.club().balance < k0 && c.wi.bau.iter().any(|b| b.0 == 0), "{} / {}", c.inbox[i].body, c.inbox.len());
    }
    #[test]
    fn sponsor_einmal_nachverhandeln() {
        let mut c = karriere();
        c.wi.vertraege.clear();
        c.wi.angebot_monat = u32::MAX;
        c.wirtschaft_monat();
        assert!(!c.wi.angebote.is_empty());
        assert!(c.sponsor_chance(0, 10) >= c.sponsor_chance(0, 20));
        let alt = c.wi.angebote[0].jahr;
        let n = c.wi.angebote.len();
        match c.sponsor_verhandeln(0, 10).unwrap() {
            (true, _) => { assert!(c.wi.angebote[0].jahr > alt); assert!(c.sponsor_verhandeln(0, 10).is_err(), "only one try"); }
            (false, _) => assert_eq!(c.wi.angebote.len(), n - 1),
        }
    }
    #[test]
    fn protest_und_bericht() {
        let mut c = karriere();
        c.fans = 20.0;
        c.ticket = 2;
        c.wv_protest();
        let i = c.inbox.iter().position(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("wv:protest"))).expect("Protest");
        c.answer(i, true, 0);
        assert_eq!(c.ticket, 1, "prices one level down");
        c.world.clubs[c.own].balance = -1.0;
        c.wv_bericht(100.0, 200.0);
        assert!(c.inbox.iter().any(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("wv:bericht"))));
    }
}
