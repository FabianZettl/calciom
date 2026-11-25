//! Paket 3j: finances, sponsors, stadium and facilities of the own club (Mechanik-Übergabe §13, vereinswelt.toml).
//! Other clubs keep the world's money (sim/geld.py: tv, gate, comm per club).
use crate::game::*;
use crate::mensch::{arr, zufall};
use crate::regeln::f;
use crate::world::*;
use serde::{Deserialize, Serialize};

pub const ANLAGEN: [&str; 5] = [crate::id("stadion"), crate::id("fanshop"), crate::id("training"), crate::id("jugend"), crate::id("medizin")];
pub fn anlagen_de_t() -> [&'static str; 5] { [crate::inhalt::t("verein.stadion"), crate::inhalt::t("vw.merch.fanshop"), crate::inhalt::t("ui.wirtschaft.trainingszentrum"), crate::inhalt::t("ui.wirtschaft.jugendzentrum"), crate::inhalt::t("ui.wirtschaft.medizinische_abteilung")] }
/// The sponsor pyramid (Fabian 24.11.): main (shirt) sponsor, kit supplier, stadium name, two secondary sponsors and four
/// small partners on the boards; the places per kind in vereinswelt.toml [sponsor.plaetze].
pub const SPONSOR_ARTEN: [&str; 5] = [crate::id("trikot"), crate::id("ausruester"), crate::id("stadion"), crate::id("neben"), crate::id("bande")];
pub fn sponsor_de_t() -> [&'static str; 5] { [crate::inhalt::t("mk.rolle.trikot"), crate::inhalt::t("mk.rolle.ausruester"), crate::inhalt::t("ui.wirtschaft.stadionname"), crate::inhalt::t("ui.sponsor.neben"), crate::inhalt::t("ui.sponsor.bande")] }
/// Places of a sponsor kind in the pyramid.
pub fn plaetze(art: u8) -> usize { crate::regeln::wert_oder("vereinswelt", &format!("sponsor.plaetze.{}", SPONSOR_ARTEN[art as usize])).and_then(|x| x.as_u64()).unwrap_or(1) as usize } // roh
/// Brand names of a kind (vereinswelt.toml [sponsor.marken]); the big brands (`gross_<kind>`) only ask big clubs.
fn marken(art: u8, gross: bool) -> Vec<String> {
    let k = SPONSOR_ARTEN[art as usize];
    let liste = |n: &str| crate::regeln::wert_oder("vereinswelt", &format!("sponsor.marken.{n}")).and_then(|x| x.as_array().cloned()).unwrap_or_default().iter().filter_map(|x| x.as_str().map(String::from)).collect::<Vec<String>>(); // roh
    let mut v = liste(k);
    // a big club's boards go to national brands, not to the bakery round the corner
    if gross { let g = liste(&format!("gross_{k}")); if g.len() >= 5 { return g; } v.extend(g); }
    v
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sponsor { pub art: u8, pub name: String, pub jahr: f64, pub bis: u32, pub bonus: f64, pub frist: u32, #[serde(default)] pub versucht: bool }

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Wirtschaft {
    pub an: bool,
    /// Facility levels 1…5 (stadium, fan shop, training, youth, medical).
    pub stufen: [u8; 5],
    /// Building work: (facility, target level, abs day it is finished).
    pub bau: Vec<(usize, u8, u32)>,
    pub vertraege: Vec<Sponsor>,
    pub angebote: Vec<Sponsor>,
    pub angebot_monat: u32,
    /// Economic power, refreshed every month (rank over all clubs is costly to compute every frame).
    pub kraft: f64,
    /// Scale of all money values (budget_start.py massstab): the formulas of vereinswelt.toml are made for a small
    /// club; the scale brings the revenue to wages / quota. Changes at most 25 % per season.
    pub massstab: f64,
    /// Operating income booked so far this season (snapshot) and the last three months (for the monthly calibration).
    pub ein_stand: f64,
    pub ein_monate: Vec<f64>,
    /// Levels of the stadium settings (stadionbetrieb.rs POSTEN; u8::MAX or missing = standard, the seat price is `ticket`).
    pub einst: Vec<u8>,
    /// The last home matches: (abs day, spectators, filling, net result).
    pub heimspiele: Vec<(u32, u32, f32, f64)>,
    /// Economy cases already opened (wirtschaftsvorgaenge.rs keys), planned bookings (month = abs/30, amount), last month's result.
    pub wv: Vec<String>,
    pub wv_plan: Vec<(u32, f64)>,
    pub wv_vormonat: Option<f64>,
    /// The investor above the pyramid (Fabian 24.11.) and the abs day of the last request.
    pub investor: Option<Investor>,
    pub investor_frage: u32,
}
/// An investor or owner: name, kind (0 state fund, 1 US private equity, 2 Asian group, 3 local entrepreneur), the money
/// he brings, his share in percent, since when.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Investor { pub name: String, pub typ: u8, pub summe: f64, pub anteil: u8, pub seit: u32 }

fn v(k: &str) -> f64 { f("vereinswelt", k) as f64 }
fn b(k: &str) -> f64 { f("budget_start", k) as f64 }
fn norm(x: f64, a: f64, z: f64) -> f64 { ((x - a) / (z - a)).clamp(0.0, 1.0) }

impl Career {
    /// Economic power = rank of squad strength among all clubs (0…1) to the power 3 (vereinswelt.toml ruf_potenz).
    pub fn kraft(&self) -> f64 { if self.wi.an { self.wi.kraft } else { self.kraft_neu() } }
    fn kraft_neu(&self) -> f64 {
        let own = crate::mensch::kaderstaerke(&self.world, self.own);
        let n = self.world.clubs.len().max(2);
        let unter = (0..n).filter(|&c| crate::mensch::kaderstaerke(&self.world, c) < own).count();
        ((unter as f64 / (n - 1) as f64).clamp(0.0, 1.0)).powf(v("verein.ruf_potenz"))
    }
    /// Money scale (1.0 = the plain values of vereinswelt.toml).
    pub fn skala(&self) -> f64 { if self.wi.massstab > 0.0 { self.wi.massstab } else { 1.0 } }
    /// Club size 0…1 (budget_start.py prestige): mean of the best 16 (62…84) 0.45, squad value (2 M…1 bn, log) 0.35, best player (65…92) 0.2.
    pub fn prestige(&self) -> f64 {
        let mut o: Vec<f64> = self.world.clubs[self.own].squad.iter().filter(|&&p| self.world.players[p].youth == 0).map(|&p| self.world.players[p].ovr as f64).collect();
        if o.is_empty() { return 0.0; }
        o.sort_by(|a, c| c.partial_cmp(a).unwrap());
        let n = o.len().min(16);
        let mw: f64 = self.world.clubs[self.own].squad.iter().map(|&p| self.world.players[p].value).sum();
        let st = norm(o[..n].iter().sum::<f64>() / n as f64, b("prestige.staerke.0"), b("prestige.staerke.1"));
        let sp = norm(o[0], b("prestige.spitze.0"), b("prestige.spitze.1"));
        let m = norm(mw.max(1.0).log10(), b("prestige.marktwert.0").log10(), b("prestige.marktwert.1").log10());
        b("prestige.gewicht.staerke") * st + b("prestige.gewicht.marktwert") * m + b("prestige.gewicht.spitze") * sp
    }
    /// Yearly revenue of an average club of this power at scale f (budget_start.py umsatz_formel): match days at
    /// fan mood 50 and normal prices, 1.7 home games a month, merchandising, TV without place bonus, sponsors.
    fn umsatz_formel(&self, f: f64) -> f64 {
        let k = self.kraft();
        let stufe = 1.0 + (k * 3.0).floor().min(2.0);
        let kap = (v("verein.kapazitaet_basis") + v("verein.kapazitaet_je_ruf") * k) * (1.0 + v("anlagen.kapazitaet_je_stufe") * (stufe - 1.0));
        // like Vereinswelt._kalibrieren: the club's real mood and its real number of league home games
        let aus = (v("einnahmen.auslastung_basis") + v("einnahmen.auslastung_je_fan") * self.fans_eff() as f64 / 100.0).clamp(0.25, 1.0);
        let preis = (v("einnahmen.ticketpreis") + v("einnahmen.ticketpreis_je_ruf") * k) * f.sqrt();
        let heim = (self.world.leagues[self.league()].teams.len().max(2) - 1) as f64;
        let spieltag = kap * aus * preis * heim;
        let merch = self.fanbasis() * v("einnahmen.merch_je_fan_monat") * f * (1.0 + v("einnahmen.merch_je_fanshopstufe") * (stufe - 1.0)) * (0.7 + 0.6 * self.fans_eff() as f64 / 100.0) * 12.0;
        let tv = (v("einnahmen.tv_basis") + v("einnahmen.tv_je_ruf") * k) * f;
        let sp: f64 = SPONSOR_ARTEN.iter().enumerate().map(|(i, a)| v(&format!("sponsor.{a}")) * (1.0 + v("sponsor.je_ruf") * k) * f * plaetze(i as u8) as f64).sum();
        spieltag + merch + tv + sp
    }
    /// The scale that brings the formula revenue to wages / quota (quota 0.62 → 0.57 by prestige, corridor 0.55…0.62).
    pub fn massstab_ziel(&self) -> f64 {
        let gehalt = self.world.wage_bill(self.own);
        let p = self.prestige();
        let q = b("quote.mitte.0") + (b("quote.mitte.1") - b("quote.mitte.0")) * p;
        let ziel = (gehalt / q).clamp(gehalt / b("quote.korridor.1"), gehalt / b("quote.korridor.0"));
        let (mut lo, mut hi) = (b("budget.faktor_grenzen.0"), b("budget.faktor_grenzen.1"));
        for _ in 0..40 { let m = (lo * hi).sqrt(); if self.umsatz_formel(m) < ziel { lo = m; } else { hi = m; } }
        (lo * hi).sqrt()
    }
    fn betriebseinnahmen(&self) -> f64 { self.book.iter().filter(|(n, v)| *v > 0.0 && !n.starts_with("Ablösen") && !n.starts_with("Leih")).map(|x| x.1).sum() } // roh
    /// Monthly, like Vereinswelt._kalibrieren on the real bookings: the income of the last three months, as a year,
    /// against wages / quota; the scale moves at most 10 % a month (own decision, so the first season is right too).
    pub fn massstab_monat(&mut self) {
        let jetzt = self.betriebseinnahmen();
        let monat = if jetzt >= self.wi.ein_stand { jetzt - self.wi.ein_stand } else { jetzt };
        self.wi.ein_stand = jetzt;
        if monat <= 0.0 { return; }
        // a rolling year of months, each kept at scale 1 (income ÷ the scale it was earned with): lumpy months (sponsor
        // money, season tickets) and older months at another scale no longer make the calibration swing
        self.wi.ein_monate.push(monat / self.skala());
        if self.wi.ein_monate.len() > 12 { self.wi.ein_monate.remove(0); }
        if self.wi.ein_monate.len() < 2 { return; }
        let jahr_basis = self.wi.ein_monate.iter().sum::<f64>() / self.wi.ein_monate.len() as f64 * 12.0;
        let gehalt = self.world.wage_bill(self.own);
        let q = b("quote.mitte.0") + (b("quote.mitte.1") - b("quote.mitte.0")) * self.prestige();
        if gehalt <= 0.0 || jahr_basis <= 0.0 { return; }
        let ziel = gehalt / q / jahr_basis;
        let s = self.skala();
        // the first half year may step ±25 %: the start value from the formula can be far off and must not carry a whole season
        let schritt = if self.wi.ein_monate.len() < 6 { 0.25 } else { 0.1 };
        self.wi.massstab = ziel.clamp(s * (1.0 - schritt), s * (1.0 + schritt)).clamp(b("budget.faktor_grenzen.0"), b("budget.faktor_grenzen.1"));
    }
    /// Season change: the scale follows its target by at most 25 % (budget.jahr_schritt).
    /// Season change: the book starts anew, so does the income snapshot.
    pub fn massstab_saison(&mut self) { self.wi.ein_stand = 0.0; self.wi.kraft = self.kraft_neu(); }
    pub fn wirtschaft_an(&mut self) {
        if self.wi.an { return; }
        let k = self.kraft_neu();
        self.wi.kraft = k;
        self.wi.an = true;
        self.wi.massstab = self.massstab_ziel();
        self.wi.an = false;
        let s = (1.0 + (k * 3.0).round()).clamp(1.0, 3.0) as u8;
        self.wi.stufen = [s; 5];
        // own decision: the club starts with running contracts of the three kinds (middle offer, 2 years)
        // the pyramid starts full except the last two board places (offers for them come in the first months)
        for art in 0..SPONSOR_ARTEN.len() as u8 { for i in 0..plaetze(art).saturating_sub(if art == 4 { 2 } else { 0 }) { let mut o = self.sponsor_angebot(art, i, 9999); o.bis = self.world.season + 1; self.wi.vertraege.push(o); } }
        self.wi.an = true;
    }
    fn sponsor_angebot(&self, art: u8, i: usize, monat: u32) -> Sponsor {
        let z = |x: &str| zufall(&[crate::id("vw"), crate::id("sponsor"), &monat.to_string(), &art.to_string(), &i.to_string(), x]) as f64;
        let basis = v(&format!("sponsor.{}", SPONSOR_ARTEN[art as usize])) * (1.0 + v("sponsor.je_ruf") * self.kraft()) * self.skala();
        let lz = arr("vereinswelt", "sponsor.laufzeiten");
        let jahre = lz[(z(crate::id("l")) * lz.len() as f64) as usize % lz.len()].as_u64().unwrap() as u32;
        let jahr = round_money(basis * (0.85 + 0.3 * z(crate::id("h"))));
        // a brand not yet with the club (running contracts and open offers)
        let liste: Vec<String> = marken(art, self.kraft() >= v("sponsor.gross_ab")).into_iter().filter(|n| !self.wi.vertraege.iter().chain(self.wi.angebote.iter()).any(|s| &s.name == n)).collect();
        let name = if liste.is_empty() { crate::inhalt::t("ui.sponsor.regional").to_string() } else { liste[(z(crate::id("n")) * liste.len() as f64) as usize % liste.len()].clone() };
        Sponsor { art, name, jahr, bis: self.world.season + jahre - 1, bonus: round_money(jahr * v("sponsor.bonus_anteil")), frist: self.abs() + v("sponsor.frist_tage") as u32, versucht: false }
    }
    pub fn kapazitaet(&self) -> f64 { (v("verein.kapazitaet_basis") + v("verein.kapazitaet_je_ruf") * self.kraft()) * (1.0 + v("anlagen.kapazitaet_je_stufe") * (self.wi.stufen[0].max(1) as f64 - 1.0)) * self.neubau_faktor() }
    pub fn fanbasis(&self) -> f64 { v("verein.fanbasis") + v("verein.fanbasis_je_ruf") * self.kraft() }
    pub fn ticket_price(&self) -> f64 { (v("einnahmen.ticketpreis") + v("einnahmen.ticketpreis_je_ruf") * self.kraft()) * self.skala().sqrt() * arr("vereinswelt", "einnahmen.preisstufen")[self.ticket as usize].as_f64().unwrap() }
    pub fn capacity(&self) -> f64 { self.kapazitaet() }
    /// Share of seats sold in a league match (vereinswelt.toml einnahmen).
    pub fn attendance(&self) -> f64 { self.spiel_neutral().auslastung }
    pub fn merch_month(&self) -> f64 { self.fanbasis() * v("einnahmen.merch_je_fan_monat") * self.skala() * (1.0 + v("einnahmen.merch_je_fanshopstufe") * (self.wi.stufen[1].max(1) as f64 - 1.0)) * (0.7 + 0.6 * self.fans_eff() as f64 / 100.0) }
    pub fn tv_monat(&self) -> f64 {
        let table = self.world.table(self.league());
        let lage = table.iter().position(|r| r.club == self.own).map(|i| 1.0 - i as f64 / (table.len().max(2) - 1) as f64).unwrap_or(0.5);
        (v("einnahmen.tv_basis") + v("einnahmen.tv_je_ruf") * self.kraft()) * self.skala() / 12.0 * (1.0 + v("einnahmen.tv_platzbonus") * lage)
    }
    pub fn betrieb_monat(&self) -> f64 {
        let k = self.kraft();
        // upkeep only for the levels above 1 (werte.py unterhalt_monat)
        let stufen: f64 = self.wi.stufen.iter().map(|&s| (s as f64 - 1.0).max(0.0)).sum();
        (v("ausgaben.betrieb_monat") + v("ausgaben.betrieb_monat_je_ruf") * k + (v("ausgaben.unterhalt_je_stufe_monat") + v("ausgaben.unterhalt_je_ruf") * k) * stufen) * self.skala()
    }
    pub fn sponsoren_jahr(&self) -> f64 { self.wi.vertraege.iter().map(|s| s.jahr).sum() }
    /// Projected yearly income (for the wage limit and the debt limit).
    pub fn umsatz_prognose(&self) -> f64 {
        let heim = (self.world.leagues[self.league()].teams.len().max(2) - 1) as f64;
        self.tv_monat() * 12.0 + self.merch_posten().0 * 12.0 + self.sponsoren_jahr() + heim * self.spiel_neutral().ein.iter().sum::<f64>()
    }
    /// Can the club take on a new yearly wage? Wages at most 62 % of the projected income; debts beyond 15 % block (vereinswelt.toml [budget]).
    pub fn rahmen_pruefen(&self, neues_gehalt: f64, altes: f64) -> Result<(), String> {
        let u = self.umsatz_prognose();
        if self.club().balance < -v("budget.schuldengrenze_anteil") * u { return Err(crate::inhalt::t("ui.wirtschaft.die_schulden_sind_zu_hoch").into()); }
        let summe = self.world.wage_bill(self.own) - altes + neues_gehalt;
        if summe > v("budget.gehaltsquote") * u { return Err(crate::tf!("ui.wirtschaft.das_gehalt_sprengt_den_rahmen", money(v("budget.gehaltsquote") * u))); }
        Ok(())
    }
    /// Monthly bookings of the own club (§13) and sponsor offers, building work.
    pub fn wirtschaft_monat(&mut self) -> f64 {
        self.wirtschaft_an();
        self.wi.kraft = self.kraft_neu();
        self.massstab_monat();
        let tv = self.tv_monat();
        let sp = self.sponsoren_jahr() / 12.0;
        let (m, mk) = self.merch_posten();
        let vb = self.vip_betrieb();
        let b = self.betrieb_monat();
        self.book(crate::inhalt::t("vw.mix.medien"), tv);
        self.book(crate::inhalt::t("menu.marken.reiter.0"), sp);
        self.book(crate::inhalt::t("ui.pages.merchandising"), m);
        // the club shop's three best-selling shirts of the month, and a share of the shop's income for transfers
        // (ideas of other FL tools, 23.11.): fame of the player, his goals, a new signing sells extra
        {
            let abs = self.abs();
            let mut sq: Vec<(usize, f32)> = self.squad().into_iter().filter(|&p| self.world.players[p].youth == 0).map(|p| {
                let pl = &self.world.players[p];
                (p, pl.ovr + pl.goals as f32 * 1.5 + if abs < pl.joined + 60 { 6.0 } else { 0.0 } + crate::mensch::zufall(&["trikot", &pl.id.to_string(), &(abs / 30).to_string()]) * 4.0) // roh
            }).collect();
            sq.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let top: Vec<String> = sq.iter().take(3).map(|x| { let pl = &self.world.players[x.0]; format!("{} #{}", pl.name, pl.no) }).collect();
            let anteil = crate::regeln::f("vereinswelt", "merch.transfer_anteil") as f64;
            let plus = crate::world::round_money(m * anteil);
            if plus > 0.0 { self.transfer_budget += plus; }
            if !top.is_empty() { let vk = self.vkey("ui.shop.text", ""); self.msg(0, crate::inhalt::t("ui.shop.absender"), crate::inhalt::t("ui.shop.titel"), &crate::tf!(&vk, top.join(", "), crate::world::money(m), crate::world::money(plus)), crate::game::Action::None); }
        }
        let beitrag = self.beitraege_monat();
        self.book(crate::inhalt::t("mv.beitraege"), beitrag);
        if mk > 0.0 { self.book(crate::inhalt::t("vw.art.merchandising_kosten"), -mk); }
        if vb > 0.0 { self.book(crate::inhalt::t("vw.art.vip_betrieb"), -vb); }
        self.book(crate::inhalt::t("ui.wirtschaft.betrieb_und_anlagen"), -b);
        // sponsor offers: 2 per kind and month, fixed per month (Random("vw|sponsor|<Monat>")), 45 days to decide
        let monat = self.abs() / 30;
        let abs = self.abs();
        self.wi.angebote.retain(|o| abs < o.frist);
        if self.wi.angebot_monat != monat {
            self.wi.angebot_monat = monat;
            // free places of the pyramid get offers, a place whose contract ends this season from January on; several
            // times a season, never while offers for that kind are still open
            let mut neu: Vec<usize> = vec![];
            for art in 0..SPONSOR_ARTEN.len() as u8 {
                let season = self.world.season;
                let laufend = self.wi.vertraege.iter().filter(|s| s.art == art && (s.bis > season || s.bis == season && self.world.day < 153)).count();
                if laufend >= plaetze(art) || self.wi.angebote.iter().any(|o| o.art == art) { continue; }
                if zufall(&[crate::id("sponsor"), crate::id("anfrage"), &monat.to_string(), &art.to_string()]) as f64 > v("sponsor.anfrage_chance") { continue; }
                for i in 0..f("vereinswelt", "sponsor.angebote_je_art") as usize { let o = self.sponsor_angebot(art, i, monat); neu.push(self.wi.angebote.len()); self.wi.angebote.push(o); }
            }
            for i in neu { self.sponsor_anfrage_melden(i); }
        }
        // building work finished
        let fertig: Vec<(usize, u8, u32)> = self.wi.bau.iter().copied().filter(|b| abs >= b.2).collect();
        self.wi.bau.retain(|b| abs < b.2);
        for (a, z, _) in fertig {
            self.wi.stufen[a] = z;
            self.msg(3, crate::inhalt::t("ui.wirtschaft.bauabteilung"), &crate::tf!("ui.wirtschaft.fertig", anlagen_de_t()[a]), &crate::tf!("ui.wirtschaft.ist_auf_stufe_ausgebaut", anlagen_de_t()[a], z), Action::None);
            if a == 0 { self.klima(f("vereinswelt", "wirtschaft.ausbau_stimmung"), 0.0, 0.0, crate::inhalt::t("ui.wirtschaft.der_stadionausbau_ist_fertig")); }
        }
        tv + sp + m + beitrag - mk - vb - b
    }
    /// Season end: contracts that ran out end; reaching the season goal pays the bonuses.
    pub fn sponsoren_saisonende(&mut self, ziel_erreicht: bool) {
        let season = self.world.season;
        if ziel_erreicht { let b: f64 = self.wi.vertraege.iter().map(|s| s.bonus).sum(); if b > 0.0 { self.book(crate::inhalt::t("ui.wirtschaft.sponsoren_bonus"), b); } }
        let aus: Vec<String> = self.wi.vertraege.iter().filter(|s| s.bis <= season).map(|s| format!("{} ({})", s.name, sponsor_de_t()[s.art as usize])).collect();
        self.wi.vertraege.retain(|s| s.bis > season);
        if !aus.is_empty() { self.msg(3, crate::inhalt::t("ui.wirtschaft.marketing"), crate::inhalt::t("ui.wirtschaft.sponsorenvertraege_laufen_aus"), &crate::tf!("ui.wirtschaft.ausgelaufen_neue_angebote_kommen_in", aus.join(", ")), Action::None); }
    }
    pub fn sponsor_annehmen(&mut self, i: usize) -> Result<(), String> {
        let Some(o) = self.wi.angebote.get(i).cloned() else { return Err(crate::inhalt::t("ui.wirtschaft.das_angebot_gibt_es_nicht").into()) };
        if self.wi.vertraege.iter().filter(|s| s.art == o.art && s.bis > self.world.season).count() >= plaetze(o.art) { return Err(crate::inhalt::t("ui.wirtschaft.fuer_diese_art_laeuft_schon").into()); }
        // the contract that ends this season gives way to the new partner
        let season = self.world.season;
        if let Some(k) = self.wi.vertraege.iter().position(|s| s.art == o.art && s.bis == season) { if self.wi.vertraege.iter().filter(|s| s.art == o.art).count() >= plaetze(o.art) { self.wi.vertraege.remove(k); } }
        let voll = self.wi.vertraege.iter().filter(|s| s.art == o.art && s.bis > season).count() + 1 >= plaetze(o.art);
        if voll { self.wi.angebote.retain(|x| x.art != o.art); } else { self.wi.angebote.retain(|x| x.name != o.name); }
        self.grund(crate::inhalt::t("ui.wirtschaft.marketing"), crate::inhalt::t("vg.rolle.sponsor"), 0.0, &crate::tf!("ui.wirtschaft.im_jahr_bis", o.name, money(o.jahr), crate::date::season_label(o.bis)));
        self.wi.vertraege.push(o);
        Ok(())
    }
    /// Cost of the next level of a facility: base · level² (stadium +10 % per level built on).
    pub fn ausbau_kosten(&self, a: usize) -> f64 {
        let z = self.wi.stufen[a] as f64 + 1.0;
        let basis = v(&format!("anlagen.kosten.{}", ANLAGEN[a]));
        basis * self.skala() * z * z * if a == 0 { 1.0 + v("anlagen.kosten_stufe_anstieg") * (z - 1.0) } else { 1.0 }
    }
    pub fn ausbauen(&mut self, a: usize) -> Result<(), String> {
        self.wirtschaft_an();
        if self.wi.stufen[a] as f32 >= f("vereinswelt", "anlagen.stufen_max") { return Err(crate::inhalt::t("ui.wirtschaft.die_hoechste_stufe_ist_erreicht").into()); }
        if self.wi.bau.iter().any(|b| b.0 == a) { return Err(crate::inhalt::t("ui.wirtschaft.hier_wird_schon_gebaut").into()); }
        let k = self.ausbau_kosten(a);
        if self.club().balance - k < -v("budget.schuldengrenze_anteil") * self.umsatz_prognose() { return Err(crate::inhalt::t("tv.drama.rueckmeldung.budget").into()); }
        // paid from the infrastructure pot (Fabian 23.11.)
        if k > self.infra() + 1.0 { return Err(crate::tf!("ui.topf.infra_reicht_nicht", crate::world::money(self.infra()), crate::world::money(k))); }
        self.infra_topf = Some(self.infra() - k);
        let dauer = (v(&format!("anlagen.dauer.{}", ANLAGEN[a])) * 30.4) as u32;
        let z = self.wi.stufen[a] + 1;
        self.book(crate::inhalt::t("vw.art.ausbau"), -k);
        self.wi.bau.push((a, z, self.abs() + dauer));
        self.grund(crate::inhalt::t("bereich.verein"), anlagen_de_t()[a], z as f32, &crate::tf!("ui.wirtschaft.ausbau_auf_stufe_begonnen_kosten", z, money(k)));
        Ok(())
    }
    /// Development factor of the training centre (+5 % per level above 1).
    pub fn training_faktor(&self) -> f32 { 1.0 + f("vereinswelt", "anlagen.training_je_stufe") * (self.wi.stufen[2].max(1) as f32 - 1.0) }
    /// Injury factor of the medical department (−4 % per level above 1).
    pub fn medizin_faktor(&self) -> f32 { 1.0 - f("vereinswelt", "anlagen.medizin_je_stufe") * (self.wi.stufen[4].max(1) as f32 - 1.0) }
    /// Extra strength and potential of youth intakes (+4 per level above 1).
    pub fn jugend_bonus(&self) -> f32 { f("vereinswelt", "anlagen.jugend_je_stufe") * (self.wi.stufen[3].max(1) as f32 - 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Three seasons for a big club, an average club and the weakest club of the same league (stand-in for a promoted one),
    // one test each so they run side by side.
    #[test]
    fn kasse_drei_saisons_gross() { kasse_drei_saisons(0) }
    #[test]
    fn kasse_drei_saisons_mitte() { kasse_drei_saisons(1) }
    #[test]
    fn kasse_drei_saisons_schwach() { kasse_drei_saisons(2) }
    fn kasse_drei_saisons(welcher: usize) {
        let w0 = crate::world::testwelt(7);
        let l = w0.clubs[0].league;
        let mut t = w0.leagues[l].teams.clone();
        t.sort_by(|a, b| crate::mensch::kaderstaerke(&w0, *b).partial_cmp(&crate::mensch::kaderstaerke(&w0, *a)).unwrap());
        for (art, own) in [("Großverein", t[0]), ("Durchschnitt", t[t.len() / 2]), ("Schwächster", t[t.len() - 1])].into_iter().skip(welcher).take(1) {
            let mut c = Career::new(w0.clone(), own, "Test");
            c.mensch.vertrag_bis = u32::MAX; // this test is about money, not about the coach's contract
            c.wi.investor_frage = u32::MAX / 2; // and about the running money: no investor comes in
            let k0 = c.club().balance;
            let mut vorher = k0;
            let mut posten: Vec<(String, f64)> = vec![];
            let mut umsatz = 0.0;
            for _ in 0..3 * crate::date::SEASON_DAYS {
                if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
                for i in 0..c.inbox.len() { if !c.inbox[i].done && matches!(c.inbox[i].action, Action::Wahl { .. } | Action::TransferOffer { .. }) { c.answer(i, false, 0); } }
                c.ultimatum = None;
                assert!(!c.fired, "{art}: entlassen");
                // like a person: expiring contracts are renewed, so the wage bill stays what it is
                let season = c.world.season;
                for p in c.world.clubs[c.own].squad.clone() { let pl = &mut c.world.players[p]; if pl.youth == 0 { pl.contract_end = pl.contract_end.max(season + 2); } }
                if c.world.day == crate::date::SEASON_DAYS - 1 {
                    let jetzt = c.club().balance;
                    // promotion bonus or relegation parachute (praemien.rs) come on top of a normal season
                    assert!((jetzt - vorher).abs() <= 0.45 * c.umsatz_prognose().max(1.0) * 1.5 + 0.12 * c.praemie_referenz(), "{art} ({}): one season {} → {}; {}", c.club().name, money(vorher), money(jetzt), c.book.iter().map(|(n, v)| format!("{n} {}", money(*v))).collect::<Vec<_>>().join(", "));
                    vorher = jetzt;
                    for (n, v) in &c.book { if let Some(x) = posten.iter_mut().find(|x| x.0 == *n) { x.1 += v; } else { posten.push((n.clone(), *v)); } } umsatz += c.umsatz_prognose(); }
                c.end_day().unwrap();
            }
            let k1 = c.club().balance;
            eprintln!("{art} ({}): Kasse {} → {}, Umsatz/Jahr {}, Posten 3 Jahre: {}", c.club().name, money(k0), money(k1), money(umsatz / 3.0),
                posten.iter().map(|(n, v)| format!("{n} {}", money(*v))).collect::<Vec<_>>().join(", "));
            // a reasonable frame: in three seasons neither more than one year's income won nor lost
            let jahr = umsatz / 3.0;
            // plus what promotion bonus and relegation parachute (praemien.rs) may bring on top
            assert!((k1 - k0).abs() <= jahr + 0.16 * c.praemie_referenz(), "{art}: {} → {} bei {} Umsatz", money(k0), money(k1), money(jahr));
        }
    }
    #[test]
    fn geld_wie_vorlage() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.wirtschaft_an();
        let k = c.kraft();
        assert!((0.0..=1.0).contains(&k));
        assert!((c.kapazitaet() - (8000.0 + 55000.0 * k) * (1.0 + 0.12 * (c.wi.stufen[0] as f64 - 1.0))).abs() < 1.0);
        assert_eq!(c.wi.vertraege.len(), (0..5u8).map(|a| plaetze(a)).sum::<usize>() - 2, "the pyramid with two free board places");
        let s = c.wi.stufen[2];
        assert!((c.ausbau_kosten(2) - 800000.0 * c.skala() * ((s + 1) as f64).powi(2)).abs() < 1.0, "cost base·level², in the club's money scale");
        let vorher = c.club().balance;
        c.infra_topf = Some(1e12); // the pot is tested in game.rs toepfe
        c.ausbauen(2).ok();
        assert!(c.club().balance < vorher);
        for _ in 0..200 { c.world.day += 1; if c.world.day % 30 == 0 { c.wirtschaft_monat(); } }
        assert_eq!(c.wi.stufen[2], s + 1, "finished after five months");
        // the same month gives the same offers
        assert_eq!(c.sponsor_angebot(0, 0, 3).jahr, c.sponsor_angebot(0, 0, 3).jahr);
    }
}
