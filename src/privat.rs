//! Private life, block 1 (Fabian 24.11., docs/intern/PRIVATLEBEN.md): the coach's stress with its reasons, his home
//! (rent or buy, move or commute), the monthly statement with tax, lifestyle, partner and children, and a private
//! calendar. Stress reaches the game: from 70 team talks work weaker. Numbers in assets/regeln/privat.toml.
use crate::game::*;
use crate::mensch::zufall;
use serde::{Deserialize, Serialize};

fn f(k: &str) -> f32 { crate::regeln::f("privat", k) }
fn liste(k: &str) -> Vec<f32> { crate::regeln::wert("privat", k).as_array().map(|a| a.iter().filter_map(|x| x.as_f64()).map(|x| x as f32).collect()).unwrap_or_default() }

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Privat {
    pub an: bool,
    pub stress: f32,
    /// (abs day, text key, change) newest last.
    pub gruende: Vec<(u32, String, f32)>,
    /// 1 small, 2 mid, 3 large; bought or rented; the club whose town it is in; commuting from there.
    pub wohnung: u8,
    pub gekauft: bool,
    pub wohnort: usize,
    pub pendeln: bool,
    pub lebensstil: u8,
    pub partner: String,
    pub beziehung: f32,
    /// Season day of the anniversary.
    pub jahrestag: u32,
    pub kinder: Vec<(String, u32)>,
    pub termine: Vec<Termin>,
    pub auszuege: Vec<Auszug>,
    pub erschoepft_gemeldet: u32,
    // block 2: car (0 none … 3 sports car) and motorbike, flight class (0 economy, 1 business, 2 private), staff
    // (housekeeper, cook, driver), personal sponsors (name, a month, until abs day), languages learned and a course
    pub auto: u8,
    pub motorrad: bool,
    pub flug: u8,
    pub personal: [bool; 3],
    pub sponsoren: Vec<(String, f64, u32)>,
    pub sprachen: Vec<String>,
    pub kurs: Option<(String, u32)>,
    pub besuche: Vec<(usize, u32)>,
}
/// A weekly private appointment: its kind (familie, sport, freunde), weekday 0 = Monday, until which abs day, and the
/// single days it was called off.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Termin { pub art: String, pub wochentag: u8, pub bis: u32, pub abgesagt: Vec<u32> }
/// A monthly statement: salary, tax, home, lifestyle, what stays.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Auszug { pub abs: u32, pub gehalt: f64, pub steuer: f64, pub wohnen: f64, pub stil: f64, pub netto: f64, #[serde(default)] pub extras: f64, #[serde(default)] pub sponsoren: f64 }

pub const ARTEN: [&str; 3] = ["familie", "sport", "freunde"]; // roh

impl Career {
    fn privat_an(&mut self) {
        if self.privat.an { return; }
        let id = self.club().id.to_string();
        let p = &mut self.privat;
        p.an = true;
        (p.stress, p.wohnung, p.wohnort, p.lebensstil) = (f("stress.start"), 2, self.own, 2);
        let sprache = self.club_sprache();
        self.privat.sprachen = vec![sprache];
        if zufall(&[crate::id("partner"), &id]) < f("familie.partner_chance") {
            let namen = self.namen_von(self.world.land_nat(self.world.clubs[self.own].league));
            let vor = ["Anna", "Laura", "Sofia", "Lea", "Marie", "Julia", "Clara", "Emma", "Lena", "Mia"]; // roh
            let n = (zufall(&["partner_name", &id]) * vor.len() as f32) as usize % vor.len();
            let _ = namen;
            self.privat.partner = vor[n].to_string();
            self.privat.beziehung = f("familie.start_beziehung");
            self.privat.jahrestag = (zufall(&[crate::id("jahrestag"), &id]) * 364.0) as u32;
        }
    }
    /// A stress change with its reason (small daily drifts are summed into one reason per day and key).
    pub fn stress(&mut self, d: f32, key: &str) {
        let abs = self.abs();
        let p = &mut self.privat;
        p.stress = (p.stress + d).clamp(0.0, 100.0);
        match p.gruende.iter_mut().find(|g| g.0 == abs && g.1 == key) { Some(g) => g.2 += d, None => p.gruende.push((abs, key.to_string(), d)) }
        p.gruende.retain(|g| abs < g.0 + 14);
        if p.gruende.len() > 40 { p.gruende.remove(0); }
    }
    /// Distance in km between two clubs' towns (0 when unknown).
    fn km(&self, a: usize, b: usize) -> f32 {
        let (Some(x), Some(y)) = (crate::geo::lage(&self.world, a), crate::geo::lage(&self.world, b)) else { return 0.0 };
        let (la1, la2) = (x.1.to_radians(), y.1.to_radians());
        let dl = (y.0 - x.0).to_radians();
        6371.0 * (la1.sin() * la2.sin() + la1.cos() * la2.cos() * dl.cos()).clamp(-1.0, 1.0).acos()
    }
    /// The day in private life: stress from matches, rest, commuting and the relationship; the calendar; a move after a
    /// change of club; the anniversary; now and then a family moment.
    pub fn privat_tag(&mut self) {
        self.privat_an();
        let abs = self.abs();
        // the match of today is played before the day ends: read it from the season's games
        let heute_spiel = self.spiele.last().filter(|x| x.abs == abs).map(|x| (x.tore, x.gegentore));
        let spieltag = heute_spiel.is_some() || self.today_match().is_some();
        if spieltag { self.stress(f("stress.spieltag"), "spieltag"); } else { self.stress(f("stress.ruhetag"), "ruhetag"); }
        if let Some((a, b)) = heute_spiel { if a < b { self.stress(f("stress.niederlage"), "niederlage"); } else if a > b { self.stress(f("stress.sieg"), "sieg"); } }
        if self.privat.pendeln { let km = self.km(self.privat.wohnort, self.own); self.stress(f("stress.pendeln_je_100km") * km / 100.0, crate::id("pendeln")); }
        if !self.privat.partner.is_empty() && self.privat.beziehung < 30.0 { self.stress(f("stress.partner_schlecht"), "partner"); }
        let stil = self.privat.lebensstil as f32 - 2.0;
        if stil > 0.0 { self.stress(f("stress.lebensstil_je_stufe") * stil, crate::id("lebensstil")); }
        // the calendar rests on matchdays and in holidays; a kept appointment eases
        let wt = crate::date::weekday(self.world.day) as u8;
        self.privat.termine.retain(|t| abs <= t.bis);
        let heute: Vec<String> = self.privat.termine.iter().filter(|t| t.wochentag == wt && !t.abgesagt.contains(&abs)).map(|t| t.art.clone()).collect();
        // the summer break is the holiday: the calendar rests
        if !spieltag && self.world.day <= crate::world::LEAGUE_END {
            for art in heute {
                self.stress(f("stress.termin"), &format!("termin_{art}"));
                if art == "familie" && !self.privat.partner.is_empty() { self.privat.beziehung = (self.privat.beziehung + 1.0).min(100.0); }
            }
        }
        // a new club far away: move or commute (asked once)
        if self.privat.wohnort != self.own && !self.privat.pendeln && !self.inbox.iter().any(|m| !m.done && m.action_art() == Some(crate::id("privat"))) {
            let km = self.km(self.privat.wohnort, self.own);
            if km < f("wohnen.pendeln_ab_km") { self.privat.wohnort = self.own; } else {
                let opts = vec![(crate::id("umziehen").to_string(), crate::tf!("ui.privat.umziehen", crate::world::money(f("wohnen.umzug") as f64))), (crate::id("pendeln").to_string(), crate::tf!("ui.privat.pendeln", format!("{km:.0}")))];
                self.msg(1, crate::inhalt::t("ui.privat.absender"), crate::inhalt::t("ui.privat.umzug_titel"), &crate::tf!("ui.privat.umzug_text", self.world.clubs[self.own].name.clone(), format!("{km:.0}")), Action::Wahl { art: "privat:umzug".into(), bezug: vec![], optionen: opts }); // roh
            }
        }
        // the anniversary
        if !self.privat.partner.is_empty() && self.world.day == self.privat.jahrestag {
            let opts = vec![(crate::id("feiern").to_string(), crate::inhalt::t("ui.privat.feiern").to_string()), (crate::id("arbeiten").to_string(), crate::inhalt::t("ui.privat.vergessen").to_string())];
            self.msg(1, &self.privat.partner.clone(), crate::inhalt::t("ui.privat.jahrestag_titel"), &crate::tf!("ui.privat.jahrestag_text", self.privat.partner.clone()), Action::Wahl { art: "privat:jahrestag".into(), bezug: vec![], optionen: opts }); // roh
        }
        // once a week a family moment, now and then the news of a child
        if crate::date::weekday(self.world.day) == 6 && !self.privat.partner.is_empty() {
            let s = abs.to_string();
            if self.privat.beziehung > 70.0 && zufall(&[crate::id("kind"), &s]) < f("familie.kind_chance") {
                let n = ["Paul", "Ella", "Noah", "Mila", "Ben", "Lina"][(zufall(&["kindname", &s]) * 6.0) as usize % 6].to_string(); // roh
                self.privat.kinder.push((n.clone(), abs));
                self.stress(-8.0, crate::id("kind"));
                self.feiern.push(crate::meilensteine::Feier { art: "privat".into(), titel: crate::tf!("ui.privat.kind_titel", n.clone()), text: crate::tf!("ui.privat.kind_text", self.privat.partner.clone(), n), spieler: None, club: Some(self.own), stimmen: vec![] }); // roh
            } else if zufall(&[crate::id("familie"), &s]) < f("familie.ereignis_je_woche") {
                let opts = vec![(crate::id("ja").to_string(), crate::inhalt::t("ui.privat.zeit_ja").to_string()), (crate::id("nein").to_string(), crate::inhalt::t("ui.privat.zeit_nein").to_string())];
                let v = self.vkey("ui.privat.zeit_text", &s);
                self.msg(1, &self.privat.partner.clone(), crate::inhalt::t("ui.privat.zeit_titel"), &crate::tf!(&v, self.privat.partner.clone()), Action::Wahl { art: "privat:zeit".into(), bezug: vec![], optionen: opts }); // roh
            }
        }
        // block 2: staff takes work off the coach's hands, a driver half the commute; a flight class on away days
        let hilfe = self.privat.personal.iter().filter(|x| **x).count() as f32;
        if hilfe > 0.0 { self.stress(f("block2.personal_stress") * hilfe, crate::id("personal")); }
        if self.privat.personal[2] && self.privat.pendeln { let km = self.km(self.privat.wohnort, self.own); self.stress(-0.5 * f("stress.pendeln_je_100km") * km / 100.0, crate::id("fahrer")); }
        if let Some(x) = self.spiele.last().filter(|x| x.abs == abs && !x.heim) {
            let _ = x;
            let flug = self.privat.flug as usize;
            self.stress(liste("block2.flug_stress").get(flug).copied().unwrap_or(2.0), crate::id("reise"));
            self.mgr_money -= liste("block2.flug_kosten").get(flug).copied().unwrap_or(0.0) as f64;
        }
        // a language course: weekly lessons, the language learned at the end
        if let Some((code, bis)) = self.privat.kurs.clone() {
            if crate::date::weekday(self.world.day) == 2 && !spieltag { self.stress(f("block2.kurs_stress"), "kurs"); }
            if abs >= bis {
                self.privat.kurs = None;
                if !self.privat.sprachen.contains(&code) { self.privat.sprachen.push(code.clone()); }
                self.msg(1, crate::inhalt::t("ui.privat.absender"), crate::inhalt::t("ui.privat.sprache_titel"), &crate::tf!("ui.privat.sprache_text", crate::inhalt::t(&format!("ui.sprache.{code}"))), Action::None);
            }
        }
        // a star coach gets personal sponsors: an offer now and then (from reputation 75)
        if crate::date::weekday(self.world.day) == 0 && self.mgr_rep >= f("block2.sponsor_ab_ruf") && self.privat.sponsoren.len() < 3 && zufall(&[crate::id("sponsor"), &abs.to_string()]) < f("block2.sponsor_chance") {
            let marken = ["Uhrenhaus Lindner", "Autohaus Berg", "Sportmarke Kestrel", "Kaffee Aurora", "Bank Nordlicht", "Mode Alvarez"]; // roh
            let name = marken[(zufall(&["sponsor_name", &abs.to_string()]) * marken.len() as f32) as usize % marken.len()].to_string();
            let betrag = crate::world::round_money(f("block2.sponsor_basis") as f64 * (self.mgr_rep as f64 / 75.0).powi(3));
            let opts = vec![(crate::id("ja").to_string(), crate::inhalt::t("ui.privat.sponsor_ja").to_string()), (crate::id("nein").to_string(), crate::inhalt::t("ui.privat.sponsor_nein").to_string())];
            self.msg(1, &name, &crate::tf!("ui.privat.sponsor_titel", name.clone()), &crate::tf!("ui.privat.sponsor_text", name.clone(), crate::world::money(betrag)), Action::Wahl { art: format!("privat:sponsor|{name}|{}", betrag as u64), bezug: vec![], optionen: opts });
        }
        // exhausted: a warning, at most once a month
        if self.privat.stress >= f("stress.erschoepft_ab") && abs >= self.privat.erschoepft_gemeldet + 30 {
            self.privat.erschoepft_gemeldet = abs;
            let co = self.co_name.clone();
            self.msg(1, &crate::tf!("ui.pool.von_co", co), crate::inhalt::t("ui.privat.erschoepft_titel"), crate::inhalt::t("ui.privat.erschoepft_text"), Action::None);
        }
    }
    pub fn privat_antwort(&mut self, sub: &str, key: &str) -> String {
        match (sub, key) {
            ("umzug", "umziehen") => {
                let kosten = f("wohnen.umzug") as f64;
                // a bought home is sold for 90 %
                if self.privat.gekauft { self.mgr_money += self.wohnung_kaufpreis(self.privat.wohnort) * 0.9; self.privat.gekauft = false; }
                self.mgr_money -= kosten;
                self.privat.wohnort = self.own;
                self.stress(5.0, crate::id("umzug"));
                crate::inhalt::t("ui.privat.umgezogen").into()
            }
            ("umzug", _) => { self.privat.pendeln = true; crate::inhalt::t("ui.privat.pendelt").into() }
            ("jahrestag", "feiern") => { self.privat.beziehung = (self.privat.beziehung + f("familie.jahrestag")).min(100.0); self.stress(-3.0, crate::id("familie")); self.mgr_money -= 2000.0; crate::inhalt::t("ui.privat.gefeiert").into() }
            ("jahrestag", _) => { self.privat.beziehung = (self.privat.beziehung + f("familie.jahrestag_vergessen")).max(0.0); crate::inhalt::t("ui.privat.vergessen_folge").into() }
            ("zeit", "ja") => { self.privat.beziehung = (self.privat.beziehung + f("familie.zeit_ja")).min(100.0); self.stress(f("familie.zeit_stress"), "familie"); crate::inhalt::t("ui.privat.zeit_ja_folge").into() } // roh
            ("zeit", _) => { self.privat.beziehung = (self.privat.beziehung + f("familie.zeit_nein")).max(0.0); crate::inhalt::t("ui.privat.zeit_nein_folge").into() }
            (x, "ja") if x.starts_with("sponsor|") => { // roh
                let t: Vec<&str> = x.split('|').collect();
                let (name, betrag) = (t.get(1).copied().unwrap_or_default().to_string(), t.get(2).and_then(|b| b.parse::<f64>().ok()).unwrap_or(0.0));
                let bis = self.abs() + 365;
                self.privat.sponsoren.push((name.clone(), betrag, bis));
                // a weekly shoot for a year: a little stress every week (the calendar)
                self.termin_anlegen("sponsor", 3, 13); // roh
                crate::tf!("ui.privat.sponsor_ok", name)
            }
            (x, _) if x.starts_with("sponsor|") => crate::inhalt::t("ui.privat.sponsor_abgelehnt").into(),
            _ => String::new(),
        }
    }
    /// The language of the own club's country (korridore.toml [land]).
    pub fn club_sprache(&self) -> String { crate::transferki::land_info(&crate::preise::liga_land(&self.world, self.league())).1 }
    /// A language course for the club's language: weekly lessons for `block2.kurs_wochen` weeks.
    pub fn sprachkurs(&mut self) -> Result<String, String> {
        let code = self.club_sprache();
        if code == "?" || self.privat.sprachen.contains(&code) { return Err(crate::inhalt::t("ui.privat.sprache_kann").into()); }
        let kosten = f("block2.kurs_kosten") as f64;
        self.mgr_money -= kosten;
        self.privat.kurs = Some((code.clone(), self.abs() + 7 * f("block2.kurs_wochen") as u32));
        Ok(crate::tf!("ui.privat.kurs_ok", crate::inhalt::t(&format!("ui.sprache.{code}")), f("block2.kurs_wochen") as u32))
    }
    /// Talks in a language the players hear as foreign work a little weaker until the coach has learned it.
    pub fn sprach_faktor(&self) -> f32 { if self.privat.an && !self.privat.sprachen.contains(&self.club_sprache()) && self.club_sprache() != "?" { f("block2.fremdsprache_faktor") } else { 1.0 } }
    /// A home visit (a car makes it possible, once a month per player): morale and the relation to the coach rise.
    pub fn hausbesuch(&mut self, p: usize) -> Result<String, String> {
        let abs = self.abs();
        if self.privat.auto == 0 && !self.privat.motorrad { return Err(crate::inhalt::t("ui.privat.besuch_ohne_auto").into()); }
        if self.privat.besuche.iter().any(|x| x.0 == p && abs < x.1 + 30) { return Err(crate::inhalt::t("ui.privat.besuch_schon").into()); }
        self.privat.besuche.retain(|x| abs < x.1 + 30);
        self.privat.besuche.push((p, abs));
        let n = self.world.players[p].name.clone();
        let w = f("block2.besuch_moral") * if self.privat.auto >= 2 { 1.2 } else { 1.0 };
        self.moral_wirkung(p, w, 0, &crate::tf!("ui.privat.besuch_grund", n.clone()));
        self.beziehung(p, f("block2.besuch_beziehung"), &crate::tf!("ui.privat.besuch_grund", n.clone()));
        self.stress(1.0, crate::id("besuch"));
        Ok(crate::tf!("ui.privat.besuch_ok", n))
    }
    /// Monthly rent of home size `stufe` (1–3) in the town of `club`.
    pub fn miete(&self, stufe: u8, club: usize) -> f64 { (liste("wohnen.miete").get(stufe.saturating_sub(1) as usize).copied().unwrap_or(6000.0) * (self.world.clubs[club].level / 75.0).max(0.5)) as f64 }
    pub fn wohnung_kaufpreis(&self, club: usize) -> f64 { self.miete(self.privat.wohnung, club) * 12.0 * f("wohnen.kauf_jahre") as f64 }
    /// Change the home: a size to rent, or buy the current one (when the money is there).
    pub fn wohnung_waehlen(&mut self, stufe: u8, kaufen: bool) -> Result<String, String> {
        if kaufen {
            let preis = self.wohnung_kaufpreis(self.privat.wohnort);
            if self.mgr_money < preis { return Err(crate::tf!("ui.privat.zu_teuer", crate::world::money(preis))); }
            self.mgr_money -= preis;
            self.privat.gekauft = true;
            return Ok(crate::tf!("ui.privat.gekauft", crate::world::money(preis)));
        }
        if self.privat.gekauft { self.mgr_money += self.wohnung_kaufpreis(self.privat.wohnort) * 0.9; self.privat.gekauft = false; }
        self.privat.wohnung = stufe.clamp(1, 3);
        self.stress(3.0, crate::id("umzug"));
        Ok(crate::tf!("ui.privat.gemietet", crate::world::money(self.miete(self.privat.wohnung, self.privat.wohnort))))
    }
    /// The month's statement: salary, tax by the club's country, home and lifestyle; the rest goes to the account.
    pub fn privat_monat(&mut self) {
        self.privat_an();
        let gehalt = self.mgr_wage / 12.0;
        let land = crate::preise::liga_land(&self.world, self.league());
        let satz = crate::regeln::wert_oder("privat", &format!("steuer.{land}")).and_then(|v| v.as_f64()).unwrap_or(f("steuer.standard") as f64);
        let steuer = gehalt * satz;
        let wohnen = if self.privat.gekauft { 0.0 } else { self.miete(self.privat.wohnung, self.privat.wohnort) };
        let stil = liste("lebensstil.kosten").get(self.privat.lebensstil.saturating_sub(1) as usize).copied().unwrap_or(4000.0) as f64;
        // block 2: car, motorbike and staff cost every month; personal sponsors pay (taxed like the salary)
        let autos = liste("block2.auto_kosten");
        let extras = autos.get(self.privat.auto as usize).copied().unwrap_or(0.0) as f64 + if self.privat.motorrad { f("block2.motorrad_kosten") as f64 } else { 0.0 }
            + self.privat.personal.iter().zip(liste("block2.personal_kosten")).filter(|x| *x.0).map(|x| x.1 as f64).sum::<f64>();
        let abs = self.abs();
        self.privat.sponsoren.retain(|x| abs <= x.2);
        let sponsoren = self.privat.sponsoren.iter().map(|x| x.1).sum::<f64>() * (1.0 - satz);
        let netto = gehalt - steuer - wohnen - stil - extras + sponsoren;
        // the salary was booked in full already (game.rs): only the rest moves the account
        self.mgr_money += sponsoren - steuer - wohnen - stil - extras;
        let a = Auszug { abs: self.abs(), gehalt, steuer, wohnen, stil, netto, extras, sponsoren };
        let m = crate::world::money;
        let mut text = crate::tf!("ui.privat.auszug_text", m(gehalt), format!("{:.0}", satz * 100.0), m(steuer), m(wohnen), m(stil), m(netto), m(self.mgr_money));
        if extras > 0.0 || sponsoren > 0.0 { text = format!("{text} {}", crate::tf!("ui.privat.auszug_extra", m(extras), m(sponsoren))); }
        self.privat.auszuege.push(a);
        if self.privat.auszuege.len() > 24 { self.privat.auszuege.remove(0); }
        self.msg(3, crate::inhalt::t("ui.privat.bank"), &crate::tf!("ui.privat.auszug_titel", crate::date::long(self.world.season, self.world.day)), &text, Action::None);
    }
    /// A weekly appointment from tomorrow on, for up to three months.
    pub fn termin_anlegen(&mut self, art: &str, wochentag: u8, wochen: u32) {
        let bis = self.abs() + wochen.min(13) * 7;
        self.privat.termine.push(Termin { art: art.into(), wochentag: wochentag % 7, bis, abgesagt: vec![] });
    }
    /// Call off the next date of appointment `i`.
    pub fn termin_absagen(&mut self, i: usize) {
        let abs = self.abs();
        let Some(t) = self.privat.termine.get(i) else { return };
        let heute = crate::date::weekday(self.world.day) as u32;
        let tage = (t.wochentag as u32 + 7 - heute) % 7;
        let naechster = abs + if tage == 0 { 7 } else { tage };
        self.privat.termine[i].abgesagt.push(naechster);
        if self.privat.termine[i].art == "familie" && !self.privat.partner.is_empty() { self.privat.beziehung = (self.privat.beziehung - 3.0).max(0.0); } // roh
    }
    /// The factor on team talks from the coach's stress (Fabian 24.11.: from 70 they work 10 % weaker).
    pub fn stress_faktor(&self) -> f32 { if self.privat.stress >= f("stress.ansprache_ab") { f("stress.ansprache_faktor") } else { 1.0 } }
}

#[cfg(test)]
mod tests {
    use crate::game::*;
    /// Fabian 24.11.: stress with reasons and its effect on talks, the statement with tax, the calendar resting on matchdays.
    #[test]
    fn privatleben_block1() {
        let mut c = Career::new(crate::world::testwelt(4), 2, "Test");
        c.privat_tag();
        assert!(c.privat.an && !c.privat.gruende.is_empty());
        let geld = c.mgr_money;
        c.privat_monat();
        let a = c.privat.auszuege.last().unwrap().clone();
        assert!(a.steuer > 0.0 && c.mgr_money < geld && (a.gehalt - a.steuer - a.wohnen - a.stil - a.netto).abs() < 1.0);
        c.privat.stress = 80.0;
        assert!(c.stress_faktor() < 1.0);
        c.privat.stress = 40.0;
        assert_eq!(c.stress_faktor(), 1.0);
        // an appointment eases stress on a free day, not on a matchday; one called off does not count
        let wt = crate::date::weekday(c.world.day) as u8;
        c.termin_anlegen("sport", wt, 4); // roh
        let vorher = c.privat.stress;
        if c.today_match().is_none() { c.privat_tag(); assert!(c.privat.stress < vorher); }
        c.termin_absagen(0);
        assert_eq!(c.privat.termine[0].abgesagt.len(), 1);
        // renting bigger costs more, buying needs the money
        assert!(c.miete(3, c.own) > c.miete(1, c.own));
        c.mgr_money = 0.0;
        assert!(c.wohnung_waehlen(2, true).is_err());
        // block 2: a home visit needs a car and works once a month; a sponsor deal pays and adds a weekly shoot
        let p = c.squad()[3];
        assert!(c.hausbesuch(p).is_err());
        c.privat.auto = 1;
        let m0 = c.world.players[p].morale;
        assert!(c.hausbesuch(p).is_ok());
        assert!(c.world.players[p].morale >= m0 && c.hausbesuch(p).is_err());
        let n = c.privat.termine.len();
        assert!(!c.privat_antwort("sponsor|Test|20000", "ja").is_empty()); // roh
        assert_eq!(c.privat.sponsoren.len(), 1);
        assert_eq!(c.privat.termine.len(), n + 1);
        // an unknown club language costs a little on talks until learned
        let code = c.club_sprache();
        c.privat.sprachen.retain(|x| *x != code);
        if code != "?" { assert!(c.sprach_faktor() < 1.0); c.privat.sprachen.push(code); assert_eq!(c.sprach_faktor(), 1.0); } // roh
    }
}
