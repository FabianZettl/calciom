//! Scouting assignments (konzepte/scouting.md Teil 3, 23.11.2026): player search, watching one player, exploring a
//! league, a second opinion, finding a successor. Scouts propose only players the club can afford and who would come
//! (3.3); a fresh search brings its first proposals after 3–5 days. Numbers in assets/regeln/scouting.toml [auftrag].
use crate::game::*;
use crate::mensch::zufall;
use crate::regeln::f;
use crate::world::*;
use serde::{Deserialize, Serialize};

pub const SUCHE: u8 = 0;
pub const BEOBACHTEN: u8 = 1;
pub const GEBIET: u8 = 2;
pub const ZWEITE: u8 = 3;
pub const NACHFOLGER: u8 = 4;
pub const ARTEN: [&str; 5] = [crate::id("suche"), crate::id("beobachten"), crate::id("gebiet"), crate::id("zweite"), crate::id("nachfolger")];

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Auftrag {
    pub id: u32,
    pub art: u8,
    /// The scout by name (scouts come and go, their index changes).
    pub scout: String,
    /// Position group (scoutbericht.rs gruppe) for searches.
    pub gruppe: String,
    pub alter_min: u8,
    pub alter_max: u8,
    /// The league to work in (None: the scout's region).
    pub liga: Option<usize>,
    /// The player of a watch, second opinion or successor assignment.
    pub spieler: Option<usize>,
    /// Show players up to 15 % over budget in their own section.
    pub knapp: bool,
    pub start: u32,
    pub ende: u32,
    /// Players being looked at right now (at most `auftrag.im_blick`).
    pub im_blick: Vec<usize>,
    /// Proposals with their points (at most 6, the better replace the worse).
    pub vorschlaege: Vec<(usize, f32)>,
    /// Looked at and not good enough, or already proposed once.
    pub gesehen: Vec<usize>,
    pub fertig: bool,
    /// Minimum role strength (successor: his own minus 3).
    #[serde(default)]
    pub staerke_min: f32,
}

fn a(k: &str) -> f32 { f("scouting", &format!("auftrag.{k}")) }

impl Career {
    /// The club's wage room: the target wage share of the expected turnover minus the current wage bill.
    pub fn gehalt_frei(&self) -> f64 {
        (self.umsatz_prognose() * a(crate::id("gehaltsquote")) as f64 + self.gehalt_plus - self.world.wage_bill(self.own)).max(0.0)
    }
    /// Free transfer budget: the budget minus the fees of running negotiations.
    pub fn transfer_frei(&self) -> f64 {
        (self.transfer_budget - self.negs.iter().map(|n| n.fee).sum::<f64>()).max(0.0)
    }
    /// Konzept 3.3: can we do it? None if yes, else the reason's text key. `knapp`: up to 15 % over budget counts.
    pub fn machbar(&self, p: usize, knapp: bool) -> Option<&'static str> {
        let pl = &self.world.players[p];
        let rand = if knapp { 1.0 + a("knapp_drueber") as f64 } else { 1.0 };
        if pl.team >= 0 && self.ask_price(p) * (1.0 + self.preis_spanne(p) as f64) > self.transfer_frei() * rand { return Some("ui.auftrag.grund.budget"); }
        let wunsch = crate::verhandlung::wunschgehalt(pl, self.world.leagues[self.league()].level, 2);
        if wunsch > self.gehalt_frei() * rand { return Some("ui.auftrag.grund.gehalt"); }
        // would he come? our reputation against his (players' reputation like the base knowledge)
        let ruf = pl.ovr * f("scouting", "grundwissen.ruf_ovr") + if pl.team >= 0 { self.world.clubs[pl.team as usize].rep * (1.0 - f("scouting", "grundwissen.ruf_ovr")) } else { 0.0 };
        if self.club().rep < ruf - a("prestige_luecke") { return Some("ui.auftrag.grund.kommt_nicht"); }
        if pl.team >= 0 && pl.joined > 0 && self.abs().saturating_sub(pl.joined) < a("frisch_gewechselt") as u32 { return Some("ui.auftrag.grund.frisch"); }
        if pl.inj as f32 >= a("verletzt_ab") { return Some("ui.auftrag.grund.verletzt"); }
        None
    }
    pub fn auftraege_laufend(&self) -> usize { self.auftraege.iter().filter(|x| !x.fertig).count() }
    /// Starts an assignment: one per scout, at most as many as the club level allows.
    pub fn auftrag_anlegen(&mut self, mut x: Auftrag, tage: u32) -> Result<String, String> {
        if self.auftraege_laufend() >= self.auftraege_max() { return Err(crate::tf!("ui.scouting.mehr_als_auftraege_gleichzeitig_erlaubt", self.auftraege_max())); }
        if !self.scouts.iter().any(|s| s.name == x.scout) { return Err(crate::inhalt::t("sup.kein_scout").into()); }
        if self.auftraege.iter().any(|y| !y.fertig && y.scout == x.scout) { return Err(crate::tf!("ui.auftrag.scout_belegt", x.scout.clone())); }
        if matches!(x.art, BEOBACHTEN | ZWEITE | NACHFOLGER) && x.spieler.is_none() { return Err(crate::inhalt::t("ui.auftrag.spieler_fehlt").into()); }
        if x.art == NACHFOLGER {
            let p = x.spieler.unwrap();
            let pl = &self.world.players[p];
            x.gruppe = crate::scoutbericht::gruppe(&pl.pos).to_string();
            x.alter_max = pl.age.saturating_sub(1).max(17);
            x.staerke_min = self.rollen_staerke(p, &x.gruppe) - a("nachfolger_abstand");
        }
        self.auftrag_id += 1;
        x.id = self.auftrag_id;
        x.start = self.abs();
        x.ende = self.abs() + tage;
        let t = crate::tf!("ui.auftrag.gestartet", x.scout.clone(), crate::inhalt::t(&format!("ui.auftrag.art.{}", ARTEN[x.art as usize])));
        self.auftraege.push(x);
        Ok(t)
    }
    fn scout_index(&self, name: &str) -> Option<usize> { self.scouts.iter().position(|s| s.name == name) }
    /// The leagues an assignment searches: its own league, else the scout's region, else the own league.
    fn auftrag_ligen(&self, x: &Auftrag) -> Vec<usize> {
        if let Some(l) = x.liga { return vec![l]; }
        let s = self.scout_index(&x.scout).and_then(|s| self.scouts[s].league);
        vec![s.unwrap_or(self.league())]
    }
    /// Who fits a search: group, age, strength floor, not ours, affordable (or just over with `knapp`).
    fn auftrag_passt(&self, x: &Auftrag, p: usize) -> bool {
        let pl = &self.world.players[p];
        if pl.youth != 0 || pl.team < 0 || self.gehoert_uns(p) { return false; }
        if !x.gruppe.is_empty() && crate::scoutbericht::gruppe(&pl.pos) != x.gruppe { return false; }
        if (x.alter_min > 0 && pl.age < x.alter_min) || (x.alter_max > 0 && pl.age > x.alter_max) { return false; }
        if x.staerke_min > 0.0 && self.rollen_staerke(p, &x.gruppe) < x.staerke_min - 2.0 { return false; }
        self.machbar(p, x.knapp).is_none()
    }
    /// One day of all assignments (Konzept 3.4): discover, look at, propose; summary message once a day.
    pub fn auftraege_tag(&mut self) {
        let abs = self.abs();
        let mut neu: Vec<(usize, usize)> = vec![];
        for k in 0..self.auftraege.len() {
            if self.auftraege[k].fertig { continue; }
            let x = self.auftraege[k].clone();
            let Some(s) = self.scout_index(&x.scout) else { self.auftraege[k].fertig = true; continue };
            let pe = self.person(s).clone();
            // a week of expenses, by distance
            if (abs - x.start) % 7 == 0 && abs > x.start {
                let fern = if self.auftrag_ligen(&x).iter().any(|&l| l != self.league()) { 1 } else { 0 };
                let sp = crate::mensch::arr("scouting", "einsatz.spesen_woche")[fern].as_f64().unwrap();
                self.book(crate::inhalt::t("profil.teil.8"), -sp);
            }
            let heim = self.scouts[s].league.map_or(false, |l| self.auftrag_ligen(&x).contains(&l));
            let tempo = a("je_tag") / 100.0 * pe.f[0] / 70.0 * if heim { 1.0 } else { a("fremd_faktor") };
            match x.art {
                BEOBACHTEN | ZWEITE => {
                    let p = x.spieler.unwrap();
                    let k2 = (self.world.players[p].know + tempo * if x.art == ZWEITE { 0.5 } else { 1.0 }).min(1.0);
                    self.world.players[p].know = k2;
                    self.wissen_gesehen(p);
                    let dauer = if x.art == ZWEITE { a("zweite_tage") as u32 } else { u32::MAX };
                    if k2 >= f("scouting", "wissen.genau_ab") / 100.0 || abs >= x.ende || abs >= x.start + dauer {
                        if x.art == ZWEITE { self.zweite_meinung(s, p); } else { self.bericht_von(s, p, crate::inhalt::t("ui.scouting.bericht")); }
                        self.auftraege[k].fertig = true;
                    }
                }
                GEBIET => {
                    if abs >= x.ende {
                        self.gebiet_abschluss(s, x.liga.unwrap_or(self.league()));
                        self.auftraege[k].fertig = true;
                    }
                }
                _ => {
                    // discover: 1 + network/40 a day (home region ×1,5), the hidden good ones preferred
                    let n = ((1.0 + pe.f[3] / 40.0) * if heim { 1.5 } else { 1.0 }).round() as usize;
                    if self.auftraege[k].im_blick.len() < a("im_blick") as usize {
                        let pool: Vec<usize> = self.auftrag_ligen(&x).iter().flat_map(|&l| self.world.leagues[l].teams.clone()).filter(|&t| t != self.own)
                            .flat_map(|t| self.world.clubs[t].squad.clone()).filter(|p| !x.gesehen.contains(p) && !x.im_blick.contains(p) && !x.vorschlaege.iter().any(|v| v.0 == *p)).filter(|&p| self.auftrag_passt(&x, p)).collect();
                        let niveau = self.club().level;
                        let mut w: Vec<f32> = pool.iter().map(|&p| {
                            let st = self.rollen_staerke(p, crate::scoutbericht::gruppe(&self.world.players[p].pos));
                            let unbekannt = 1.0 - self.grundwissen(p);
                            ((st - niveau) / 5.0).exp().min(20.0) * (1.0 + unbekannt * (0.5 + pe.f[0] / 100.0)) * (0.5 + zufall(&[crate::id("auftrag"), &x.id.to_string(), &p.to_string()]))
                        }).collect();
                        for _ in 0..n.min(pool.len()) {
                            if w.iter().sum::<f32>() <= 0.0 { break; }
                            let i = self.world.rng.weighted(&w);
                            w[i] = 0.0;
                            self.auftraege[k].im_blick.push(pool[i]);
                        }
                    }
                    // look at those in view; at the report threshold judge them
                    for p in self.auftraege[k].im_blick.clone() {
                        let k2 = (self.world.players[p].know + tempo).min(1.0);
                        self.world.players[p].know = k2;
                        self.wissen_gesehen(p);
                        if k2 < f("scouting", "wissen.bericht_ab") / 100.0 { continue; }
                        let y = &mut self.auftraege[k];
                        y.im_blick.retain(|&q| q != p);
                        y.gesehen.push(p);
                        let b = self.scout_urteil(p);
                        if b.urteil > 4 { continue; }
                        let y = &mut self.auftraege[k];
                        if y.vorschlaege.len() >= 6 {
                            let (i, schwach) = y.vorschlaege.iter().enumerate().min_by(|a, b| a.1 .1.partial_cmp(&b.1 .1).unwrap()).map(|(i, v)| (i, v.1)).unwrap();
                            if b.punkte <= schwach { continue; }
                            y.vorschlaege.remove(i);
                        }
                        y.vorschlaege.push((p, b.punkte));
                        neu.push((s, p));
                    }
                    // a proposal whose verdict fell to "Nicht holen" with more knowledge drops out
                    let alt = self.auftraege[k].vorschlaege.clone();
                    let bleiben: Vec<(usize, f32)> = alt.into_iter().filter(|&(p, _)| self.scout_urteil(p).urteil <= 4).collect();
                    self.auftraege[k].vorschlaege = bleiben;
                    if abs >= x.ende { self.auftraege[k].fertig = true; }
                }
            }
        }
        if !neu.is_empty() { self.vorschlaege_melden(&neu); }
    }
    /// One message a day for all new proposals (Konzept Teil 7), "Sofort holen" first.
    fn vorschlaege_melden(&mut self, neu: &[(usize, usize)]) {
        let mut v: Vec<(usize, usize, usize)> = neu.iter().map(|&(s, p)| (self.scout_urteil(p).urteil, s, p)).collect();
        v.sort();
        let sofort = v.iter().filter(|x| x.0 == 0).count();
        let zeilen: Vec<String> = v.iter().map(|&(u, s, p)| {
            let pl = &self.world.players[p];
            let club = if pl.team >= 0 { self.world.clubs[pl.team as usize].name.clone() } else { String::new() };
            crate::tf!("ui.auftrag.zeile", pl.name.clone(), club, pl.age, crate::inhalt::t(&format!("ui.sb.urteil.{u}")), self.scouts[s].name.clone())
        }).collect();
        let titel = if sofort > 0 { crate::tf!("ui.auftrag.neu_sofort", v.len(), sofort) } else { crate::tf!("ui.auftrag.neu", v.len()) };
        let von = self.scouts[v[0].1].name.clone();
        let p = v[0].2;
        for &(_, s, q) in &v { let (lo, hi) = self.ovr_range(q); let abs = self.abs(); self.person(s).pruefen.push((q, (lo + hi) / 2.0, abs)); }
        self.msg(if sofort > 0 { 1 } else { 6 }, &von, &titel, &zeilen.join(crate::id("\n")), Action::Report { pid: p });
    }
    /// A full report of one player from scout `s` (inbox, with the report card).
    pub fn bericht_von(&mut self, s: usize, p: usize, art: &str) {
        let body = format!("{}\n\n{}", self.report_text(p), self.urteil_text(p));
        let (lo, hi) = self.ovr_range(p);
        let abs = self.abs();
        self.person(s).pruefen.push((p, (lo + hi) / 2.0, abs));
        let (n2, sn) = (self.world.players[p].name.clone(), self.scouts[s].name.clone());
        self.msg(6, &sn, &format!("{art}{n2}"), &body, Action::Report { pid: p });
    }
    /// Konzept 3.2 "Zweite Meinung": another scout's view, his personality moves the points; the difference is shown.
    fn zweite_meinung(&mut self, s: usize, p: usize) {
        let b = self.scout_urteil(p);
        let typ = self.person(s).typ as usize % crate::scouting::TYPEN.len();
        let bias = crate::regeln::f_oder("scouting", &format!("persoenlichkeit.{}.bias", crate::scouting::TYPEN[typ]), 0.0);
        let urteil = self.urteil_stufe(&b, bias);
        let (n, sn) = (self.world.players[p].name.clone(), self.scouts[s].name.clone());
        let satz = if urteil == b.urteil { crate::inhalt::t("ui.auftrag.zweite.gleich").to_string() } else if urteil < b.urteil { crate::inhalt::t("ui.auftrag.zweite.besser").to_string() } else { crate::inhalt::t("ui.auftrag.zweite.schlechter").to_string() };
        let body = crate::tf!("ui.auftrag.zweite.text", sn.clone(), n.clone(), crate::inhalt::t(&format!("ui.sb.urteil.{urteil}")), crate::inhalt::t(&format!("ui.sb.urteil.{}", b.urteil)), satz);
        self.msg(6, &sn, &crate::tf!("ui.auftrag.zweite.titel", n), &body, Action::Report { pid: p });
    }
    /// Konzept 3.2 "Gebiet erkunden": the league's regulars get better known (+15) and the ten most interesting are named.
    fn gebiet_abschluss(&mut self, s: usize, l: usize) {
        let mut alle: Vec<(usize, f32)> = vec![];
        for t in self.world.leagues[l].teams.clone() {
            if t == self.own { continue; }
            let lvl = self.world.clubs[t].level;
            for p in self.world.clubs[t].squad.clone() {
                if self.world.players[p].youth != 0 { continue; }
                if self.world.players[p].ovr >= lvl - 1.0 { let k = &mut self.world.players[p].know; *k = (*k + a("gebiet_plus") / 100.0).min(1.0); self.wissen_gesehen(p); }
                if self.machbar(p, false).is_none() { alle.push((p, self.rollen_staerke(p, crate::scoutbericht::gruppe(&self.world.players[p].pos)) - self.club().level + self.moneyball(p) * 0.5)); }
            }
        }
        alle.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let zeilen: Vec<String> = alle.iter().take(10).map(|&(p, _)| { let pl = &self.world.players[p]; format!("{} ({}, {}, {})", pl.name, pos_de(&pl.pos), pl.age, self.world.clubs[pl.team as usize].name) }).collect();
        let sn = self.scouts[s].name.clone();
        let p = alle.first().map(|x| x.0);
        self.msg(6, &sn, &crate::tf!("ui.auftrag.gebiet.titel", self.world.leagues[l].name.clone()), &crate::tf!("ui.auftrag.gebiet.text", zeilen.join(crate::id("\n"))), p.map_or(Action::None, |p| Action::Report { pid: p }));
    }
}

impl Career {
    /// The data analyst (Konzept 3.1, 5.2): a scout with that task, from club level 3.
    pub fn analyst(&self) -> Option<usize> { self.scouts.iter().position(|s| s.p.as_ref().map_or(false, |p| p.aufgabe == 2)) }
    pub fn analyst_erlaubt(&self) -> bool { self.scout_stufe() + 1 >= a("analyst_ab_stufe") as usize }
    /// The leagues whose extended numbers we see (Konzept Spielerstatistiken 8.1): every detailed league, and with an
    /// analyst 3 more results-only leagues (strongest first), 3 more from judgement 60 and from 75.
    pub fn analyst_ligen(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.world.leagues.len()).filter(|&l| self.world.leagues[l].detail < 2).collect();
        // bought data packages count like the analyst's leagues for their season
        v.extend(self.datenpakete.iter().filter(|x| x.1 == self.world.season && x.0 < self.world.leagues.len()).map(|x| x.0));
        if let Some(s) = self.analyst() {
            let u = self.scouts[s].p.as_ref().map_or(50.0, |p| p.f[0]);
            let n = 3 + if u >= 60.0 { 3 } else { 0 } + if u >= 75.0 { 3 } else { 0 };
            let mut rest: Vec<usize> = (0..self.world.leagues.len()).filter(|&l| self.world.leagues[l].detail == 2 && self.world.leagues[l].teams.len() >= 8).collect();
            rest.sort_by(|x, y| crate::statistik::liga_staerke(&self.world, *y).partial_cmp(&crate::statistik::liga_staerke(&self.world, *x)).unwrap());
            v.extend(rest.into_iter().take(n));
        }
        v
    }
    /// The price of a league's data package for a season (Statistik 8.1): by the league's strength, in the club's money scale.
    pub fn datenpaket_preis(&self, l: usize) -> f64 {
        crate::world::round_money(crate::regeln::f("scouting", "datenpaket.basis") as f64 * (crate::statistik::liga_staerke(&self.world, l) as f64 / 70.0).powi(3) * self.skala())
    }
    pub fn datenpaket_kaufen(&mut self, l: usize) -> Result<String, String> {
        let s = self.world.season;
        if self.datenpakete.contains(&(l, s)) { return Err(crate::inhalt::t("ui.daten.schon").into()); }
        let preis = self.datenpaket_preis(l);
        if preis > self.club().balance.max(0.0) + self.transfer_budget { return Err(crate::inhalt::t("tv.drama.rueckmeldung.budget").into()); }
        self.book(crate::inhalt::t("ui.daten.buchung"), -preis);
        self.datenpakete.push((l, s));
        Ok(crate::tf!("ui.daten.gekauft", self.world.leagues[l].name.clone(), money(preis)))
    }
    /// Konzept 5.2 "Auffällige Zahlen", once a week: performance against price in the analyst's leagues, ten players with a
    /// sentence each; a hit lifts the knowledge to 15 ("heard of"). A weak analyst looks at goals more than at xG and falls
    /// for hot streaks more often.
    pub fn analyst_woche(&mut self) {
        let Some(s) = self.analyst() else { return };
        let u = self.scouts[s].p.as_ref().map_or(50.0, |p| p.f[0]);
        let gut = ((u - 40.0) / 40.0).clamp(0.0, 1.0);
        let eigene = self.league();
        let ligen = self.analyst_ligen();
        let mut treffer: Vec<(usize, f32, f32)> = vec![];
        for &l in &ligen {
            if l == eigene { continue; }
            for &t in &self.world.leagues[l].teams {
                for &p in &self.world.clubs[t].squad {
                    let pl = &self.world.players[p];
                    if pl.youth != 0 || self.gehoert_uns(p) { continue; }
                    let (st, _) = crate::statistik::saison_von(&self.world, p);
                    let min = crate::statistik::get(&st, crate::statistik::MIN) as f32;
                    if min < a("analyst_min") { continue; }
                    let Some(e) = crate::statistik::einfluss90(&st) else { continue };
                    let tore90 = crate::statistik::get(&st, crate::statistik::TORE) as f32 * 90.0 / min;
                    let xg90 = crate::statistik::get(&st, crate::statistik::XG) as f32 / 1000.0 * 90.0 / min;
                    // the weak analyst swaps xG for goals in the attack part
                    let roh = e + (1.0 - gut) * (tore90 - xg90);
                    let um = crate::statistik::umrechnen(&self.world, roh, l, eigene, 0);
                    let geschrumpft = (um * min + a("analyst_mittel") * 600.0) / (min + 600.0);
                    let jung = (23.0 - pl.age as f32).max(0.0) * 0.05;
                    let preis = (pl.value.max(1e5) as f32).log10();
                    treffer.push((p, geschrumpft + jung - a("analyst_preisgewicht") * (preis - 6.0), um));
                }
            }
        }
        treffer.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
        let zehn: Vec<(usize, f32, f32)> = treffer.into_iter().filter(|x| self.machbar(x.0, true).is_none()).take(10).collect();
        if zehn.is_empty() { return; }
        let mut zeilen = vec![];
        for &(p, _, um) in &zehn {
            let k = self.world.players[p].know.max(a("analyst_wissen") / 100.0);
            self.world.players[p].know = k;
            let pl = &self.world.players[p];
            let l = self.world.clubs[pl.team as usize].league;
            zeilen.push(crate::tf!("ui.analyst.zeile", pl.name.clone(), pos_de(&pl.pos), pl.age, self.world.clubs[pl.team as usize].name.clone(), self.world.leagues[l].name.clone(), format!("{um:.2}"), money(pl.value)));
        }
        let n = self.scouts[s].name.clone();
        self.analyst_liste = zehn.iter().map(|x| x.0).collect();
        self.msg(6, &n, crate::inhalt::t("ui.analyst.titel"), &format!("{}\n{}", crate::inhalt::t("ui.analyst.einleitung"), zeilen.join(crate::id("\n"))), Action::Report { pid: zehn[0].0 });
    }
}

impl Career {
    /// Konzept 4.4 "Kaderlücke": per position group 0 green (well covered), 1 yellow (thin, old or contract ending),
    /// 2 red (empty or clearly weaker than the league's regulars).
    pub fn kaderluecke(&self, gr: &str) -> u8 {
        let sq: Vec<usize> = self.squad().into_iter().filter(|&p| crate::scoutbericht::gruppe(&self.world.players[p].pos) == gr).collect();
        let Some(&beste) = sq.iter().max_by(|a, b| self.rollen_staerke(**a, gr).partial_cmp(&self.rollen_staerke(**b, gr)).unwrap()) else { return 2 };
        let mass = self.massstab(gr);
        let liga = if mass.len() == 25 { (0..25).map(|i| mass[i] * if crate::scoutbericht::wichtig(gr, i) >= 2 { 1.0 } else { 0.0 }).sum::<f32>() / (0..25).filter(|&i| crate::scoutbericht::wichtig(gr, i) >= 2).count().max(1) as f32 } else { 70.0 };
        let st = self.rollen_staerke(beste, gr);
        if st < liga - a("luecke_rot") { return 2; }
        let bl = &self.world.players[beste];
        let braucht = match gr { "iv" | "av" | "zm" | "fl" => 3, _ => 2 };
        if sq.len() < braucht || bl.age >= 31 || bl.contract_end <= self.world.season { 1 } else { 0 }
    }
    /// Konzept 5.4: once a week the shortlist's state — transfer list, contract ending, other clubs after him.
    pub fn merkliste_woche(&mut self) {
        let mut zeilen = vec![];
        for &p in &self.merkliste {
            let pl = &self.world.players[p];
            if pl.team == self.own as i32 { continue; }
            let mut was = vec![];
            if pl.listed { was.push(crate::inhalt::t("ui.merk.transferliste")); }
            if pl.team >= 0 && pl.contract_end <= self.world.season { was.push(crate::inhalt::t("ui.merk.vertrag")); }
            if self.anbahnung.iter().any(|x| x.p == p) { was.push(crate::inhalt::t("ui.merk.konkurrenz")); }
            if pl.inj > 0 { was.push(crate::inhalt::t("ui.merk.verletzt")); }
            if !was.is_empty() { zeilen.push(format!("{}: {}", pl.name, was.join(", "))); }
        }
        if zeilen.is_empty() { return; }
        let p = self.merkliste[0];
        self.msg(6, crate::inhalt::t("ui.merk.absender"), crate::inhalt::t("ui.merk.titel"), &zeilen.join(crate::id("\n")), Action::Report { pid: p });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analyst_meldet_auffaellige_zahlen() {
        let w = World::load(include_str!("../assets/world.json"), 7);
        let mut c = Career::new(w, 0, "Test");
        c.transfer_budget = 5e7;
        let s = 0;
        c.person(s).aufgabe = 2;
        assert_eq!(c.analyst(), Some(s));
        for _ in 0..60 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } let _ = c.end_day(); }
        c.analyst_woche();
        assert!(!c.analyst_liste.is_empty(), "a list of standout numbers");
        assert!(c.analyst_liste.iter().all(|&p| !c.gehoert_uns(p) && c.world.players[p].know >= 0.15));
    }
    #[test]
    fn suche_liefert_machbare_vorschlaege() {
        let w = World::load(include_str!("../assets/world.json"), 7);
        let mut c = Career::new(w, 0, "Test");
        c.transfer_budget = 8e6;
        let scout = c.scouts[0].name.clone();
        let x = Auftrag { art: SUCHE, scout, gruppe: "av".into(), alter_max: 30, ..Default::default() };
        c.auftrag_anlegen(x, 14).unwrap();
        for _ in 0..6 { c.auftraege_tag(); c.world.day += 1; }
        let v = &c.auftraege[0].vorschlaege;
        assert!(!v.is_empty(), "a search brings proposals within days");
        for &(p, _) in v {
            assert!(c.machbar(p, false).is_none(), "only affordable players");
            assert_eq!(crate::scoutbericht::gruppe(&c.world.players[p].pos), "av");
        }
        // a second assignment for the same scout is refused
        let x2 = Auftrag { art: SUCHE, scout: c.scouts[0].name.clone(), ..Default::default() };
        assert!(c.auftrag_anlegen(x2, 14).is_err());
        // a tiny budget leaves no proposal over it
        c.transfer_budget = 1.0;
        assert!(c.world.players.iter().enumerate().filter(|(_, p)| p.team >= 0 && p.team != c.own as i32 && p.youth == 0).take(400).all(|(p, _)| c.machbar(p, false).is_some() || c.world.players[p].team < 0 || c.ask_price(p) * (1.0 + c.preis_spanne(p) as f64) <= 1.0));
    }
}
