//! Stories, club reputation and the world's memory for the own club (pesm/fl/sim/geschichten.py; values
//! assets/story/welt/werte.json, texts as building blocks assets/story/welt/texte.json, unchanged from the original).
//! Weekly: running arcs get an update every three weeks, a new one starts (fairy tale of a promoted club, title race,
//! relegation fight, coach under pressure, a talent's breakthrough, the rivals' duel); at the season's end the arcs close
//! and the reputation tags (Ausbildungsverein, Spektakel-Klub, Bollwerk …) grow or fade; now and then the papers remember.
use crate::game::*;
use crate::mensch::zufall;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

fn daten() -> &'static (Value, Value) {
    static D: OnceLock<(Value, Value)> = OnceLock::new();
    D.get_or_init(|| (serde_json::from_str(include_str!("../assets/story/welt/werte.json")).unwrap(), serde_json::from_str(include_str!("../assets/story/welt/texte.json")).unwrap()))
}
fn g(pfad: &str) -> f64 { pfad.split('.').fold(&daten().0, |v, k| &v[k]).as_f64().unwrap_or(0.0) }
const STIMMEN: [&str; 3] = [crate::id("presse"), crate::id("vorstand"), crate::id("fans")];

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Bogen { pub key: String, pub start: u32, pub last: u32, pub stimme: u8, pub pid: usize, pub gegner: usize, pub tag: u32 }

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Geschichten {
    pub arcs: Vec<Bogen>,
    pub done: Vec<(u32, String)>,
    pub ruf: Vec<(String, f32)>,
    pub tags: Vec<String>,
    pub seen: Vec<String>,
    pub promoted: bool,
    pub relegated: bool,
    pub bounced: bool,
    pub rel_last: bool,
    /// Per finished season: (season, tier, champion, cup winner).
    pub hist: Vec<(u32, u8, bool, bool)>,
    pub mem_last: u32,
    pub mem_used: Vec<String>,
}

/// One text from the building blocks: one piece per part (the voice's own parts if it has them), facts filled in.
pub fn sagen(key: &str, fakten: &[(&str, String)], stimme: Option<&str>, seed: &str) -> String {
    let d = &daten().1[key];
    let teile = stimme.and_then(|s| d["ton"][s].as_array()).or(d["teile"].as_array());
    let Some(teile) = teile else { return String::new() };
    let mut out = String::new();
    for (i, t) in teile.iter().enumerate() {
        let alt = t.as_array().map(|a| a.as_slice()).unwrap_or(std::slice::from_ref(t));
        if alt.is_empty() { continue; }
        let k = (zufall(&[crate::id("geschichte"), key, seed, &i.to_string()]) * alt.len() as f32) as usize % alt.len();
        let s = alt[k].as_str().unwrap_or("");
        if s.is_empty() { continue; }
        if !out.is_empty() && !s.starts_with([',', '.', ';', ':', '!', '?', '…', ')']) { out.push(' '); }
        out.push_str(s);
    }
    for (n, w) in fakten { out = out.replace(&format!("{{{n}}}"), w); }
    out.replace("  ", " ").trim().to_string()
}

fn zahl(n: i32) -> String { daten().0["zahlen"].as_array().and_then(|z| z.get(n as usize)).and_then(|x| x.as_str()).map(|s| s.to_string()).unwrap_or(n.to_string()) }
fn punkte(n: i32) -> String { if n == 1 { crate::inhalt::t("gw.ein_punkt").into() } else { crate::tf!("gw.punkte", zahl(n)) } }

struct Lage { pos: usize, n: usize, pts: i32, frac: f64, spiele: u32, rows: Vec<crate::world::Row> }

impl Career {
    fn gw_lage(&self) -> Option<Lage> {
        let rows = self.world.table(self.league());
        let n = rows.len();
        let idx = rows.iter().position(|r| r.club == self.own)?;
        if n < 6 { return None; }
        let total = 2 * (n - 1);
        Some(Lage { pos: idx + 1, n, pts: rows[idx].pts, frac: rows[idx].p as f64 / total as f64, spiele: rows[idx].p, rows })
    }
    fn gw_fakten(&self, l: &Lage) -> Vec<(&'static str, String)> {
        vec![(crate::id("club"), self.club().name.clone()), (crate::id("platz"), crate::tf!("jv.platz", l.pos)), (crate::id("liga"), self.world.leagues[self.league()].name.clone()), (crate::id("spiele"), l.spiele.to_string())]
    }
    /// One story as a message from the press, the board or the fans (and a line in the news).
    fn gw_senden(&mut self, art: &str, fakten: &[(&str, String)], stimme: u8) {
        let seed = format!("{}|{}", self.abs(), art);
        let s = STIMMEN[stimme as usize % 3];
        let text = sagen(art, fakten, Some(s), &seed);
        if text.is_empty() { return; }
        let betreff = { let b = sagen(&format!("{art}_betreff"), fakten, None, &seed); if b.is_empty() { text.split('.').next().unwrap_or("").to_string() } else { b } };
        let von = crate::inhalt::t(&format!("gw.von.{s}")); // roh
        self.msg(4, von, &betreff, &text, Action::None);
        self.news(&betreff, &text);
    }
    fn gw_erkennen(&self, k: &str, l: &Lage) -> Option<Vec<(&'static str, String)>> {
        let mut f = self.gw_fakten(l);
        match k {
            "maerchen" => (self.geschichten.promoted && l.frac >= g("maerchen.min_anteil") && l.pos <= 3.max((l.n as f64 * g("maerchen.spitze_anteil")).ceil() as usize)).then_some(f),
            "titel" => {
                if l.pos > 2 || l.frac < g("titel.min_anteil") { return None; }
                let gap = l.rows[0].pts - l.rows[1].pts;
                if gap as f64 > g("titel.max_abstand") { return None; }
                f.push((crate::id("luecke"), if l.pos == 1 { crate::tf!("gw.vor_verfolger", punkte(gap)) } else if gap > 0 { crate::tf!("gw.hinter_fuehrer", punkte(gap)) } else { crate::inhalt::t("gw.punktgleich_fuehrer").into() }));
                Some(f)
            }
            "abstieg" => {
                let zone = g("abstieg.zone") as usize;
                if l.frac < g("abstieg.min_anteil") || l.pos < l.n - zone { return None; }
                let gap = l.rows[l.n - zone - 1].pts - l.pts;
                f.push((crate::id("luecke"), if gap > 0 { crate::tf!("gw.vom_ufer", punkte(gap)) } else { crate::inhalt::t("gw.punktgleich_ufer").into() }));
                Some(f)
            }
            "druck" => (self.mensch.vorstand as f64 <= g("druck.board_max") && l.frac >= 0.15 && l.pos as f64 > self.club().goal_pos as f64 + g("druck.platz_ueber_ziel")).then_some(f),
            _ => None,
        }
    }
    fn gw_stimme(&self, k: &str) -> u8 { (zufall(&[crate::id("stimme"), k, &self.abs().to_string()]) * 3.0) as u8 % 3 }
    /// Weekly (on the original's weekday, from the third week): updates, a new story, the memory.
    pub fn geschichten_woche(&mut self) {
        if self.world.day < 14 || self.world.day % 7 != g(crate::id("wochentag")) as u32 { return; }
        let Some(l) = self.gw_lage() else { return };
        let (abs, season) = (self.abs(), self.world.season);
        let verlauf = g("verlauf_tage") as u32;
        // running arcs
        for a in self.geschichten.arcs.clone() {
            let k = a.key.clone();
            if let Some(rest) = k.strip_prefix(crate::id("rivale:")) {
                let _ = rest;
                if let Some(x) = self.spiele.iter().rev().find(|x| x.gegner == a.gegner && x.abs >= a.tag) {
                    let (gh, ga) = (x.tore, x.gegentore);
                    let art = if gh > ga { crate::id("rivale_sieg") } else if gh < ga { crate::id("rivale_niederlage") } else { crate::id("rivale_remis") };
                    let mut f = self.gw_fakten(&l);
                    f.push((crate::id("gegner"), self.world.clubs[a.gegner].name.clone()));
                    f.push((crate::id("ergebnis"), format!("{gh}:{ga}")));
                    self.gw_senden(art, &f, self.gw_stimme(&k));
                    self.geschichten.arcs.retain(|b| b.key != k);
                }
                continue;
            }
            if let Some(p) = k.strip_prefix(crate::id("talent:")).and_then(|x| x.parse::<usize>().ok()) {
                if abs >= a.last + verlauf && self.world.players.get(p).is_some_and(|pl| pl.team == self.own as i32) {
                    let mut f = self.gw_fakten(&l);
                    f.push((crate::id("spieler"), self.world.players[p].name.clone()));
                    f.push((crate::id("alter"), self.world.players[p].age.to_string()));
                    self.gw_senden(crate::id("talent_verlauf"), &f, a.stimme);
                    if let Some(b) = self.geschichten.arcs.iter_mut().find(|b| b.key == k) { b.last = abs; }
                }
                continue;
            }
            match self.gw_erkennen(&k, &l) {
                Some(f) => if abs >= a.last + verlauf { self.gw_senden(&format!("{k}_verlauf"), &f, a.stimme); if let Some(b) = self.geschichten.arcs.iter_mut().find(|b| b.key == k) { b.last = abs; } }, // roh
                None => {
                    let f = self.gw_fakten(&l);
                    if k == crate::id("druck") && self.mensch.vorstand as f64 >= g("druck.befreit_board") { self.gw_senden(crate::id("druck_befreit"), &f, self.gw_stimme(&k)); self.geschichten.arcs.retain(|b| b.key != k); }
                    else if k == crate::id("abstieg") && l.frac >= 0.8 && l.pos < l.n - g("abstieg.zone") as usize { self.gw_senden(crate::id("abstieg_gerettet"), &f, self.gw_stimme(&k)); self.geschichten.arcs.retain(|b| b.key != k); }
                }
            }
        }
        // a new story (at most one a week, at most max_arcs at once)
        if self.geschichten.arcs.len() < g("max_arcs") as usize {
            let erledigt = |s: &Self, k: &str| s.geschichten.done.iter().any(|d| d.0 == season && d.1 == k);
            let mut neu = false;
            for k in [crate::id("maerchen"), crate::id("titel"), crate::id("abstieg"), crate::id("druck")] {
                if erledigt(self, k) || self.geschichten.arcs.iter().any(|a| a.key == k) { continue; }
                if let Some(f) = self.gw_erkennen(k, &l) {
                    let v = self.gw_stimme(k);
                    self.gw_senden(&format!("{k}_start"), &f, v); // roh
                    self.geschichten.arcs.push(Bogen { key: k.into(), start: abs, last: abs, stimme: v, ..Default::default() });
                    self.geschichten.done.push((season, k.into()));
                    neu = true;
                    break;
                }
            }
            if !neu {
                // a talent: young, enough matches, a strong average
                let best = self.squad().into_iter().filter(|&p| { let pl = &self.world.players[p]; pl.age as f64 <= g("talent.max_alter") && pl.apps as f64 >= g("talent.min_einsaetze") && pl.rn > 0 && (pl.rsum / pl.rn as f32) as f64 >= g("talent.min_note") })
                    .max_by(|a, b| (self.world.players[*a].rsum / self.world.players[*a].rn.max(1) as f32).partial_cmp(&(self.world.players[*b].rsum / self.world.players[*b].rn.max(1) as f32)).unwrap());
                if let Some(p) = best.filter(|p| !erledigt(self, &format!("talent:{p}"))) { // roh
                    let key = format!("talent:{p}"); // roh
                    let v = self.gw_stimme(&key);
                    let mut f = self.gw_fakten(&l);
                    f.push((crate::id("spieler"), self.world.players[p].name.clone()));
                    f.push((crate::id("alter"), self.world.players[p].age.to_string()));
                    self.gw_senden(crate::id("talent_start"), &f, v);
                    self.geschichten.arcs.push(Bogen { key: key.clone(), start: abs, last: abs, stimme: v, pid: p, ..Default::default() });
                    self.geschichten.done.push((season, key));
                } else if let Some(f_) = self.next_match().filter(|f_| f_.day <= self.world.day + g("rivale.tage_vorher") as u32) {
                    // the rivals' duel: a derby with a clear balance
                    let geg = if f_.h == self.own { f_.a } else { f_.h };
                    let bilanz = self.spiele.iter().filter(|x| x.gegner == geg).fold((0, 0, 0), |s, x| if x.tore > x.gegentore { (s.0 + 1, s.1, s.2) } else if x.tore == x.gegentore { (s.0, s.1 + 1, s.2) } else { (s.0, s.1, s.2 + 1) });
                    let key = format!("rivale:{}", abs_day(season, f_.day)); // roh
                    if is_derby(&self.club().name, &self.world.clubs[geg].name).is_some() && (bilanz.0 as i32 - bilanz.2 as i32).abs() as f64 >= g("rivale.min_diff") && !self.geschichten.arcs.iter().any(|a| a.key.starts_with(crate::id("rivale:"))) {
                        let mut f = self.gw_fakten(&l);
                        f.push((crate::id("gegner"), self.world.clubs[geg].name.clone()));
                        f.push((crate::id("bilanz"), crate::tf!("gw.bilanz", bilanz.0, bilanz.1, bilanz.2)));
                        let v = self.gw_stimme(&key);
                        self.gw_senden(crate::id("rivale_start"), &f, v);
                        self.geschichten.arcs.push(Bogen { key, start: abs, last: abs, stimme: v, gegner: geg, tag: abs_day(season, f_.day), ..Default::default() });
                    }
                }
            }
        }
        self.gw_gedaechtnis(&l);
    }
    /// Now and then the papers remember: a title, a cup, a promotion or relegation one, two, three or five years ago.
    fn gw_gedaechtnis(&mut self, l: &Lage) {
        let abs = self.abs();
        if abs < self.geschichten.mem_last + g("gedaechtnis.abstand_tage") as u32 { return; }
        let s = self.world.season;
        let jahre: Vec<u32> = daten().0["gedaechtnis"]["jahre"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64()).map(|x| x as u32).collect()).unwrap_or_default();
        let mut cands = vec![];
        let hist = self.geschichten.hist.clone();
        for (i, h) in hist.iter().enumerate() {
            let ago = s.saturating_sub(h.0);
            if !jahre.contains(&ago) { continue; }
            if h.2 { cands.push((crate::id("gedaechtnis_titel"), ago, h.0)); }
            if h.3 { cands.push((crate::id("gedaechtnis_pokal"), ago, h.0)); }
            if i > 0 && hist[i - 1].1 != h.1 { cands.push((if h.1 < hist[i - 1].1 { crate::id("gedaechtnis_aufstieg") } else { crate::id("gedaechtnis_abstieg") }, ago, h.0)); }
        }
        cands.retain(|x| !self.geschichten.mem_used.contains(&format!("{}:{}", x.0, x.2)));
        if cands.is_empty() { return; }
        let (art, ago, hs) = cands[(zufall(&[crate::id("mem"), &abs.to_string()]) * cands.len() as f32) as usize % cands.len()];
        let mut f = self.gw_fakten(l);
        f.push((crate::id("jahre"), if ago == 1 { crate::inhalt::t("gw.einem_jahr").into() } else { crate::tf!("gw.jahren", zahl(ago as i32)) }));
        f.push((crate::id("jahre_c"), if ago == 1 { crate::inhalt::t("gw.ein_jahr").into() } else { crate::tf!("gw.jahre", { let z = zahl(ago as i32); let mut c = z.chars(); c.next().map(|x| x.to_uppercase().collect::<String>() + c.as_str()).unwrap_or(z) }) }));
        self.gw_senden(art, &f, 0);
        self.geschichten.mem_used.push(format!("{art}:{hs}"));
        if self.geschichten.mem_used.len() > 30 { self.geschichten.mem_used.remove(0); }
        self.geschichten.mem_last = abs;
    }
    /// Season end: open arcs close, the season goes into the memory, the reputation grows or fades.
    pub fn geschichten_saisonende(&mut self) {
        let Some(l) = self.gw_lage() else { return };
        let abgestiegen = l.pos + (self.world.leagues[self.league()].ab as usize) > l.n;
        for a in std::mem::take(&mut self.geschichten.arcs) {
            let mut f = self.gw_fakten(&l);
            let ende = match a.key.as_str() {
                "maerchen" => Some(if l.pos <= 3.max((l.n as f64 * g("maerchen.spitze_anteil")).ceil() as usize) { "maerchen_gut" } else { "maerchen_schlecht" }), // roh
                "titel" => Some(if l.pos == 1 { "titel_gewonnen" } else { "titel_verpasst" }), // roh
                "abstieg" => Some(if !abgestiegen { "abstieg_gerettet" } else { "abstieg_abgestiegen" }), // roh
                "druck" => Some(if self.mensch.vorstand as f64 >= g("druck.befreit_board") { "druck_befreit" } else { "druck_bleibt" }), // roh
                k if k.starts_with(crate::id("talent:")) && self.world.players.get(a.pid).is_some_and(|p| p.team == self.own as i32) => {
                    f.push((crate::id("spieler"), self.world.players[a.pid].name.clone()));
                    f.push((crate::id("alter"), self.world.players[a.pid].age.to_string()));
                    Some(crate::id("talent_ende"))
                }
                _ => None,
            };
            if let Some(e) = ende { let v = self.gw_stimme(e); self.gw_senden(e, &f, v); }
        }
        let season = self.world.season;
        let meister = l.pos == 1;
        let pokal = self.world.titles.iter().any(|t| t.club == self.own && t.season == season && self.world.cups.iter().any(|c| c.kind == 0 && c.name == t.comp));
        self.geschichten.hist.push((season, self.world.leagues[self.league()].stufe, meister, pokal));
        if self.geschichten.hist.len() > 12 { self.geschichten.hist.remove(0); }
        self.geschichten.done.retain(|d| d.0 + 2 > season);
        // the reputation (geschichten.py _ruf): points fade, the season adds, from `aktiv` a tag is the club's name
        let r = |k: &str| g(&format!("ruf.{k}")); // roh
        for x in self.geschichten.ruf.iter_mut() { x.1 *= r(crate::id("verfall")) as f32; }
        let row = &l.rows[l.pos - 1];
        let p_ = row.p.max(1) as f64;
        let jugend = self.squad().iter().filter(|&&p| self.world.players[p].age <= 21 && self.world.players[p].apps >= 10).count() as f64;
        let mut add: Vec<(&str, f32)> = vec![];
        if jugend >= r("jugend_min") { add.push((crate::id("ausbildung"), if jugend >= r("jugend_min") + 2.0 { 1.5 } else { 1.0 })); }
        if (row.gf + row.ga) as f64 / p_ >= r("tore_pro_spiel") { add.push((crate::id("spektakel"), 1.0)); }
        if row.ga as f64 / p_ <= r("gegentore_pro_spiel") { add.push((crate::id("bollwerk"), 1.0)); }
        if self.geschichten.bounced { add.push((crate::id("stehaufmaennchen"), 2.0)); }
        if meister { add.push((crate::id("titelsammler"), 1.5)); } else if pokal { add.push((crate::id("titelsammler"), 1.0)); }
        if self.geschichten.promoted || self.geschichten.relegated { add.push((crate::id("fahrstuhl"), 0.7)); }
        for (k, v) in add { if let Some(x) = self.geschichten.ruf.iter_mut().find(|x| x.0 == k) { x.1 += v; } else { self.geschichten.ruf.push((k.into(), v)); } }
        let mut akt: Vec<(String, f32)> = self.geschichten.ruf.iter().filter(|x| x.1 as f64 >= r(crate::id("aktiv"))).cloned().collect();
        akt.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        self.geschichten.tags = akt.into_iter().take(2).map(|x| x.0).collect();
        for k in self.geschichten.tags.clone() {
            let eff = daten().0["ruf"]["fans"][k.as_str()].as_f64().unwrap_or(0.0) as f32;
            if eff != 0.0 { self.klima(eff, 0.0, 0.0, crate::inhalt::t(&format!("gw.ruf.{k}"))); } // roh
            if !self.geschichten.seen.contains(&k) {
                self.geschichten.seen.push(k.clone());
                let f = self.gw_fakten(&l);
                let seed = format!("{}|ruf", self.abs());
                let text = format!("{} {}", sagen(&format!("ruf_{k}"), &f, None, &seed), sagen(crate::id("ruf_grund"), &f, None, &seed)); // roh
                let betreff = crate::tf!("gw.ruf_betreff", self.club().name.clone(), crate::inhalt::t(&format!("gw.ruf.{k}"))); // roh
                self.msg(4, crate::inhalt::t("gw.von.presse"), &betreff, &text, Action::None);
                self.news(&betreff, &text);
            }
        }
    }
    /// After the league change of the season: promoted, relegated, back up straight after going down.
    pub fn geschichten_stufe(&mut self, alt: u8, neu: u8) {
        let g_ = &mut self.geschichten;
        g_.promoted = neu < alt;
        g_.relegated = neu > alt;
        g_.bounced = g_.promoted && g_.rel_last;
        g_.rel_last = g_.relegated;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn baukasten_und_ruf() {
        let t = sagen("maerchen_start", &[("club", "Testverein".into()), ("liga", "Liga".into()), ("platz", "Platz 2".into()), ("spiele", "9".into())], Some("fans"), "x");
        assert!(t.contains("Testverein") || t.contains("Platz 2"), "{t}");
        assert!(!t.contains('{'));
        let w = crate::world::World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        for _ in 0..120 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } c.ultimatum = None; c.end_day().unwrap(); }
        c.geschichten_saisonende();
        assert_eq!(c.geschichten.hist.len(), 1);
    }
}
