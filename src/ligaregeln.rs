//! League rules from daten/ligaregeln.toml (a ligaregeln.toml in the career folder wins): promotion and relegation,
//! play-offs, relegation ties, playouts, split seasons, rounds, second-team limits and the European places.
//! Country: [[land]] by pes_land (= fl26world country=), else by Competition.bin key (schluessel), else the country of
//! the league below, else the country name. League: [[land.liga]] by schluessel, else by stufe. Missing → [standard].
use crate::game::Career;
use crate::world::{Fixture, Row, World};
use serde::Deserialize;
use std::sync::{Arc, OnceLock};

#[derive(Deserialize, Default, Clone, Debug)]
pub struct Datei { #[serde(default)] pub standard: Standard, #[serde(default)] pub uefa: Uefa, #[serde(default)] pub land: Vec<Land> }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Standard { #[serde(default)] pub absteiger_nach_vereinen: Vec<Groesse>, pub runden: Option<usize>, #[serde(default)] pub zweite_mannschaften: Zweite }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Groesse { pub ab_vereine: usize, pub absteiger: usize }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Zweite { pub erlaubt: Option<bool>, pub hoechste_stufe: Option<u8>, #[serde(default)] pub erkennung: Vec<String> }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Uefa { #[serde(default)] pub leistungsplaetze_eps: usize }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Land {
    #[serde(default)] pub code: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub pes_land: u32,
    #[serde(default)] pub verband: String,
    pub zweite_mannschaften: Option<Zweite>,
    pub europa: Option<Plaetze>,
    pub kontinental: Option<Plaetze>,
    #[serde(default)] pub liga: Vec<Liga>,
}
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Plaetze { #[serde(default)] pub plaetze: Vec<Platz> }
#[derive(Deserialize, Clone, Debug)]
pub struct Platz { pub wettbewerb: String, pub plaetze: Vec<Ort>, #[serde(default)] pub phase: String }
/// A place: a table position, or "pokal" / "ligapokal".
#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Ort { Platz(usize), Pokal(String) }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Liga {
    pub stufe: u8,
    #[serde(default)] pub name: String,
    pub schluessel: Option<String>,
    pub vereine: Option<usize>,
    pub staffeln: Option<usize>,
    pub runden: Option<usize>,
    pub aufsteiger_direkt: Option<usize>,
    pub absteiger_direkt: Option<usize>,
    pub sicher: Option<bool>,
    pub playoffs: Option<Playoffs>,
    pub relegation: Option<Relegation>,
    pub playout: Option<Playout>,
    pub split: Option<Split>,
}
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Playoffs { #[serde(default)] pub plaetze: Vec<usize>, #[serde(default)] pub aufsteiger: usize, #[serde(default)] pub format: String, pub ziel: Option<String> }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Relegation { pub platz: usize, #[serde(default)] pub gegen: String, #[serde(default)] pub format: String }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Playout { #[serde(default)] pub plaetze: Vec<usize>, #[serde(default)] pub absteiger: usize, #[serde(default)] pub format: String }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Split { pub nach_spieltagen: usize, #[serde(default)] pub gruppen: Vec<Gruppe>, #[serde(default)] pub punkte: String }
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Gruppe { #[serde(default)] pub name: String, pub plaetze: [usize; 2] }

/// The rules of one league, with the standard filled in.
#[derive(Clone, Debug, Default)]
pub struct Regel {
    pub auf: usize,
    pub ab: usize,
    pub runden: usize,
    pub playoffs: Option<Playoffs>,
    pub relegation: Option<Relegation>,
    pub playout: Option<Playout>,
    pub split: Option<Split>,
    /// false: estimated in the file, or no entry at all (standard rules) → "Regel geschätzt".
    pub sicher: bool,
    pub aus_datei: bool,
    /// The file's club count, only to compare with the world.
    pub vereine: Option<usize>,
}

/// One move at the season end: `ab` goes from `oben` down to `ziel` (its regional group), `auf` from `unten` up; `grund` direkt, Play-off, Relegation, Playout.
#[derive(Clone, Debug)]
pub struct Wechsel { pub ab: usize, pub auf: usize, pub oben: usize, pub unten: usize, pub grund: &'static str, pub ziel: usize, pub geschaetzt: bool }

/// A club name for the region lists: without club words (FC, SV …) and without year numbers.
fn name_ohne_zahlen(n: &str) -> String { crate::farm::kern(n).split(' ').filter(|w| !w.chars().all(|c| c.is_ascii_digit())).collect::<Vec<_>>().join(" ") }
/// A tie not decided yet (while planning the decision matches).
pub const OFFEN: usize = usize::MAX;

/// assets/regionen.json (a copy in the data folder adds to it): regional group name → clubs of that region.
pub fn regionen() -> &'static Vec<(String, Vec<String>)> {
    static R: OnceLock<Vec<(String, Vec<String>)>> = OnceLock::new();
    R.get_or_init(|| {
        let mut v: Vec<(String, Vec<String>)> = vec![];
        let mut lesen = |t: &str| if let Ok(j) = serde_json::from_str::<serde_json::Value>(t) { if let Some(o) = j["staffeln"].as_object() { for (k, w) in o {
            let namen: Vec<String> = w.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            match v.iter_mut().find(|x| &x.0 == k) { Some(x) => x.1.extend(namen), None => v.push((k.clone(), namen)) }
        } } };
        lesen(include_str!("../assets/regionen.json"));
        if !cfg!(test) { if let Ok(t) = std::fs::read_to_string(crate::game::save_dir().join(crate::id("../regionen.json"))) { lesen(&t); } }
        v
    })
}
pub fn eingebaut() -> Arc<Datei> {
    static D: OnceLock<Arc<Datei>> = OnceLock::new();
    D.get_or_init(|| Arc::new(toml::from_str(include_str!("../daten/ligaregeln.toml")).unwrap_or_else(|e| panic!("daten/ligaregeln.toml: {e}")))).clone() // roh
}
/// The career folder's ligaregeln.toml if there is one and it reads, else the built-in file (with the error).
pub fn laden(karriere: &std::path::Path) -> (Arc<Datei>, Option<String>) {
    let p = karriere.join("ligaregeln.toml");
    let Ok(t) = std::fs::read_to_string(&p) else { return (eingebaut(), None) };
    match toml::from_str::<Datei>(&t) { Ok(d) => (Arc::new(d), None), Err(e) => (eingebaut(), Some(format!("{}: {e}", p.display()))) }
}

impl World {
    fn unter(&self, l: usize) -> impl Iterator<Item = usize> + '_ {
        let k = self.leagues[l].key.clone();
        (0..self.leagues.len()).filter(move |&u| self.leagues[u].ueber.as_deref() == Some(k.as_str()))
    }
    pub fn ueber_index(&self, l: usize) -> Option<usize> { let k = self.leagues[l].ueber.as_deref()?; self.leagues.iter().position(|o| o.key == k) }
    /// Tier of a league; a base league without its own line is one above the league below it.
    pub fn stufe_von(&self, l: usize) -> u8 {
        let s = self.leagues[l].stufe;
        if s > 0 { return s; }
        self.unter(l).map(|u| self.leagues[u].stufe).find(|&s| s > 1).map_or(0, |s| s - 1)
    }
    pub fn regel_land(&self, l: usize) -> Option<&Land> {
        let (d, lg) = (&*self.ligaregeln, &self.leagues[l]);
        let nr = if lg.land_nr != 0 { lg.land_nr } else { self.unter(l).map(|u| self.leagues[u].land_nr).find(|&n| n != 0).unwrap_or(0) };
        d.land.iter().find(|x| nr != 0 && x.pes_land == nr)
            .or_else(|| d.land.iter().find(|x| x.liga.iter().any(|g| g.schluessel.as_deref() == Some(lg.key.as_str()))))
            .or_else(|| d.land.iter().find(|x| !lg.land.is_empty() && x.name == lg.land))
    }
    fn regel_liga(&self, l: usize) -> Option<&Liga> {
        let land = self.regel_land(l)?;
        let key = self.leagues[l].key.as_str();
        let st = self.stufe_von(l);
        land.liga.iter().find(|g| g.schluessel.as_deref() == Some(key)).or_else(|| land.liga.iter().find(|g| st > 0 && g.stufe == st))
    }
    /// Direct relegations: the file, else by size ([standard]); a league with nothing below relegates nobody.
    fn ab_von(&self, l: usize) -> usize {
        if self.unter(l).next().is_none() { return 0; }
        let n = self.leagues[l].teams.len();
        self.regel_liga(l).and_then(|g| g.absteiger_direkt)
            .unwrap_or_else(|| self.ligaregeln.standard.absteiger_nach_vereinen.iter().find(|s| n >= s.ab_vereine).map_or(0, |s| s.absteiger))
    }
    pub fn regel(&self, l: usize) -> Regel {
        let g = self.regel_liga(l);
        let auf = g.and_then(|g| g.aufsteiger_direkt).unwrap_or_else(|| self.ueber_index(l).map_or(0, |u| self.ab_von(u)));
        let runden = g.and_then(|g| g.runden).or(self.ligaregeln.standard.runden).unwrap_or(2).max(1);
        Regel {
            auf, ab: self.ab_von(l), runden,
            playoffs: g.and_then(|g| g.playoffs.clone()), relegation: g.and_then(|g| g.relegation.clone()), playout: g.and_then(|g| g.playout.clone()), split: g.and_then(|g| g.split.clone()),
            sicher: g.and_then(|g| g.sicher).unwrap_or(false), aus_datei: g.is_some(), vereine: g.and_then(|g| g.vereine),
        }
    }
    /// The highest tier a second team of this league may reach (0: none, not in the league system).
    pub fn zweite_hoechste_stufe(&self, l: usize) -> u8 {
        let s = &self.ligaregeln.standard.zweite_mannschaften;
        let z = self.regel_land(l).and_then(|x| x.zweite_mannschaften.as_ref());
        if !z.and_then(|z| z.erlaubt).or(s.erlaubt).unwrap_or(true) { return 0; }
        z.and_then(|z| z.hoechste_stufe).or(s.hoechste_stufe).unwrap_or(0)
    }
    /// Name parts that mark a second team: "Jong " is a prefix, everything else a suffix (" II", " B", " U23").
    pub fn zweite_namensteile(&self) -> Vec<(bool, String)> {
        self.ligaregeln.standard.zweite_mannschaften.erkennung.iter().map(|e| if e.ends_with(' ') { (true, e.clone()) } else { (false, format!(" {}", e.trim_start())) }).collect()
    }

    /// The split of a league when its groups cover every club: the last group reaches the end of the table (a world
    /// with more or fewer clubs than the file). Title rounds of only the best (Liguilla, Apertura/Clausura) don't
    /// cover everyone and are left out: those leagues play their normal schedule.
    pub fn split_fuer(&self, l: usize) -> Option<Split> {
        let r = self.regel(l);
        let mut s = r.split?;
        let n = self.leagues[l].teams.len();
        let ende = s.gruppen.last()?.plaetze[1];
        if ende == r.vereine.unwrap_or(n) || ende > n { s.gruppen.last_mut()?.plaetze[1] = n; }
        let mut bis = 0;
        for g in &s.gruppen { if g.plaetze[0] != bis + 1 || g.plaetze[1] < g.plaetze[0] { return None; } bis = g.plaetze[1]; }
        (bis == n).then_some(s)
    }
    /// Rounds of the season (before a split) and the rounds of the longest split group after it.
    fn runden_plan(&self, l: usize) -> (usize, usize) {
        let n = self.leagues[l].teams.len();
        let r = self.regel(l);
        match &self.split_fuer(l) {
            Some(s) => (s.nach_spieltagen, s.gruppen.iter().map(|g| 2 * (g.plaetze[1] + 1).saturating_sub(g.plaetze[0]).saturating_sub(1)).max().unwrap_or(0)),
            None => (r.runden * n.saturating_sub(1), 0),
        }
    }
    pub fn spieltage(&self, l: usize) -> Vec<u32> {
        let (a, b) = self.runden_plan(l);
        let n = self.leagues[l].teams.len();
        let no_winter = ["England", "Schottland", "Türkei"].contains(&self.leagues[l].land.as_str()); // roh
        if n < 2 { return vec![]; }
        World::match_days(a + b + self.endrunde_tage(l), n <= 18 && !no_winter)
    }
    /// The season's fixtures: `runden` round robins (home and away alternate per round robin), or up to the split.
    pub fn make_fixtures(&mut self, l: usize) {
        self.profil_cache.clear();
        self.leagues[l].basis.clear(); // an imported table base belongs to its season only
        let mut ts = self.leagues[l].teams.clone();
        let mut r = crate::world::Rng(self.rng.next());
        for i in (1..ts.len()).rev() { let j = (r.next() % (i as u64 + 1)) as usize; ts.swap(i, j); }
        let days = self.spieltage(l);
        let basis = self.runden_plan(l).0;
        let mut fx: Vec<Fixture> = rundenplan(&ts, basis).into_iter().map(|(rd, h, a)| Fixture { day: days[rd], h, a, hg: -1, ag: -1 }).collect();
        fx.sort_by_key(|f| f.day);
        self.leagues[l].fixtures = fx;
        self.leagues[l].gruppen.clear();
        self.leagues[l].meister = None;
    }
    /// Split seasons: when every match before the split is played, the table is cut into its groups, the points
    /// change by the rule, and each group plays a double round robin on the days kept free. Returns the leagues split.
    pub fn split_pruefen(&mut self) -> Vec<usize> {
        let mut out = vec![];
        for l in 0..self.leagues.len() {
            let lg = &self.leagues[l];
            if !lg.gruppen.is_empty() || lg.fixtures.is_empty() || lg.fixtures.iter().any(|f| f.hg < 0) { continue; }
            let Some(s) = self.split_fuer(l) else { continue };
            let days = self.spieltage(l);
            let rest = days.get(s.nach_spieltagen..).unwrap_or(&[]).to_vec();
            // an old schedule without free days (or days already past): no split this season
            if rest.is_empty() || rest[0] <= self.day { continue; }
            let table = self.table(l);
            let gruppen = split_gruppen(&table, &s);
            for (club, alt, neu) in split_punkte(&table, &s) { if alt != neu { self.leagues[l].deduct.push((club, alt - neu)); } }
            for g in &gruppen { for (rd, h, a) in rundenplan(g, 2 * g.len().saturating_sub(1)) { if let Some(&d) = rest.get(rd) { self.leagues[l].fixtures.push(Fixture { day: d, h, a, hg: -1, ag: -1 }); } } }
            self.leagues[l].fixtures.sort_by_key(|f| f.day);
            self.leagues[l].gruppen = gruppen;
            out.push(l);
        }
        out
    }

    /// A title round of only the best (Liguilla, Apertura/Clausura finals): split groups in the file that don't cover
    /// every club. Its size: the end of the last group.
    pub fn endrunde(&self, l: usize) -> Option<usize> {
        let s = self.regel(l).split?;
        if self.split_fuer(l).is_some() { return None; }
        let k = s.gruppen.last()?.plaetze[1].min(self.leagues[l].teams.len());
        (k >= 2).then_some(k)
    }
    /// Two legs per tie in this country's title round (assets/regeln/endrunden.toml, by country code).
    pub fn endrunde_hin_rueck(&self, l: usize) -> bool {
        let d: toml::Value = toml::from_str(include_str!("../assets/regeln/endrunden.toml")).expect("endrunden.toml");
        let code = self.regel_land(l).map(|x| x.code.clone()).unwrap_or_default();
        d.get(&code).or_else(|| d.get("standard")).and_then(|x| x.get("hin_rueck")).and_then(|x| x.as_bool()).unwrap_or(true)
    }
    /// Rounds and match days the title round needs (kept free at the end of the schedule).
    fn endrunde_tage(&self, l: usize) -> usize {
        let Some(k) = self.endrunde(l) else { return 0 };
        let runden = (k as f32).log2().ceil() as usize;
        runden * if self.endrunde_hin_rueck(l) { 2 } else { 1 }
    }
    /// After the last match day of the main round: the best of the table start the title round, a competition of
    /// its own (cup kind 4) on the days kept free: best against worst, byes for the best when the number is no
    /// power of two. Returns the leagues whose title round starts today.
    pub fn endrunden_pruefen(&mut self) -> Vec<usize> {
        let mut out = vec![];
        for l in 0..self.leagues.len() {
            let lg = &self.leagues[l];
            if lg.meister.is_some() || lg.fixtures.is_empty() || lg.fixtures.iter().any(|f| f.hg < 0) { continue; }
            let Some(k) = self.endrunde(l) else { continue };
            if self.cups.iter().any(|c| c.kind == 4 && c.key == lg.key) { continue; }
            let teil: Vec<usize> = self.table(l).iter().take(k).map(|r| r.club).collect();
            let a = self.runden_plan(l).0;
            let mut days: Vec<u32> = self.spieltage(l).get(a..).unwrap_or(&[]).to_vec();
            // a schedule made before the title rounds (older save): a match every fourth day from now
            let noetig = self.endrunde_tage(l);
            if days.len() < noetig || days.first().map_or(true, |&d| d <= self.day) { days = (1..=noetig as u32).map(|i| self.day + 4 * i).collect(); }
            let name = crate::tf!("ui.ligaregeln.endrunde", lg.name);
            self.cups.push(crate::cups::Cup { name, key: lg.key.clone(), kind: 4, country: String::new(), teams: teil.clone(), fx: vec![], stage: 0, alive: vec![], winner: None, days });
            let ci = self.cups.len() - 1;
            self.endrunde_runde(ci, teil, 0);
            out.push(l);
        }
        out
    }
    /// One round of a title round: the clubs in seed order (table order), the best get byes up to a power of two,
    /// then best against worst. Two legs: the worse placed at home first; one match: at the better placed club,
    /// the final on neutral ground.
    pub fn endrunde_runde(&mut self, ci: usize, teil: Vec<usize>, r: usize) {
        let Some(l) = self.leagues.iter().position(|x| x.key == self.cups[ci].key) else { return };
        let legs = if self.endrunde_hin_rueck(l) { 2 } else { 1 };
        let runden = (self.cups[ci].teams.len() as f32).log2().ceil() as usize;
        let groesse = 1usize << (runden - r);
        let freilos = groesse.saturating_sub(teil.len());
        let (frei, spielen) = teil.split_at(freilos.min(teil.len()));
        self.cups[ci].alive = frei.to_vec();
        let final_ = groesse == 2;
        let stage = (if legs == 2 { 34 - runden + r } else if final_ { 34 } else { 24 - runden + r }) as u8;
        self.cups[ci].stage = stage;
        let days = self.cups[ci].days.clone();
        let heute = self.day;
        let tag = |i: usize| *days.get(i).or(days.last()).unwrap_or(&heute);
        let mut busy = self.busy_days();
        for i in 0..spielen.len() / 2 {
            let (a, b) = (spielen[i], spielen[spielen.len() - 1 - i]);
            if legs == 2 {
                self.add_fx(ci, &mut busy, b, a, tag(2 * r), stage, 1, false);
                self.add_fx(ci, &mut busy, a, b, tag(2 * r + 1), stage, 2, false);
            } else {
                self.add_fx(ci, &mut busy, a, b, tag(r), stage, 0, final_);
            }
        }
    }
    /// The champion: the winner of the title round, else the first of the table.
    pub fn meister(&self, l: usize) -> Option<usize> { self.leagues[l].meister.or_else(|| self.table(l).first().map(|r| r.club)) }
    /// Two legs (the better placed `a` at home second); level on aggregate: penalties, the stronger side slightly favoured.
    /// A tie already decided this season (played as real matches) is taken as it is; while planning, a tie becomes
    /// two real matches and stays OFFEN until both are played.
    pub fn duell(&mut self, a: usize, b: usize) -> usize {
        if a == OFFEN || b == OFFEN { return OFFEN; }
        if let Some(x) = self.entscheid.iter().find(|x| (x.0 == a && x.1 == b) || (x.0 == b && x.1 == a)) { return x.2; }
        if let Some((oben, art)) = self.planen { return self.duell_planen(a, b, oben, art); }
        self.neutral = false;
        let r1 = self.play(b, a, 2, None, None);
        let r2 = self.play(a, b, 2, None, None);
        let (ga, gb) = (r1.ag as i32 + r2.hg as i32, r1.hg as i32 + r2.ag as i32);
        if ga != gb { return if ga > gb { a } else { b }; }
        let p = (0.5 + crate::cups::PENS_STRENGTH * (r2.sh - r2.sa)).clamp(0.35, 0.65);
        if self.rng.chance(p) { a } else { b }
    }
    fn duell_planen(&mut self, a: usize, b: usize, oben: usize, art: &'static str) -> usize {
        let key = self.leagues[oben].key.clone();
        let name = crate::tf!("ui.entscheid.name", self.leagues[oben].name, crate::inhalt::t(&format!("ui.entscheid.{art}")));
        let ci = match self.cups.iter().position(|c| c.kind == 5 && c.name == name) {
            Some(i) => i,
            None => { self.cups.push(crate::cups::Cup { name, key, kind: 5, country: String::new(), teams: vec![], fx: vec![], stage: 33, alive: vec![], winner: None, days: vec![] }); self.cups.len() - 1 }
        };
        let paar = |x: &crate::cups::CupFx| (x.f.h == a && x.f.a == b) || (x.f.h == b && x.f.a == a);
        if !self.cups[ci].fx.iter().any(paar) {
            // two legs after the league season, the better placed `a` at home second
            let mut busy = self.busy_days();
            let t = self.day.max(crate::world::LEAGUE_END) + 4;
            self.cups[ci].teams.extend([a, b]);
            self.add_fx(ci, &mut busy, b, a, t, 33, 1, false);
            self.add_fx(ci, &mut busy, a, b, t + 4, 33, 2, false);
            return OFFEN;
        }
        if self.cups[ci].fx.iter().filter(|x| paar(x)).any(|x| x.f.hg < 0) { return OFFEN; }
        let w = self.winner_of(ci, 33, a, b);
        self.entscheid.push((a, b, w));
        w
    }
    /// Every day once all leagues are played: the play-offs, playouts and relegation ties the season end needs become
    /// real matches (calendar, statistics, playable in FL27), round by round; the season end then takes their winners.
    pub fn entscheidungen_planen(&mut self) {
        if self.leagues.iter().any(|l| l.fixtures.iter().any(|f| f.hg < 0)) { return; }
        let tables: Vec<Vec<Row>> = (0..self.leagues.len()).map(|l| self.table(l)).collect();
        self.planen = Some((0, ""));
        let _ = self.wechsel_rechnen(&tables);
        self.planen = None;
    }
    /// A knock-out among clubs in table order: best against worst, each tie over two legs (unknown formats too).
    pub fn ko_runde(&mut self, teil: &[usize]) -> Option<usize> {
        let mut v = teil.to_vec();
        while v.len() > 1 {
            let mut next = vec![];
            if v.len() % 2 == 1 { next.push(v.remove(0)); } // odd: the best has a bye
            for i in 0..v.len() / 2 { let (a, b) = (v[i], v[v.len() - 1 - i]); next.push(self.duell(a, b)); }
            next.sort_by_key(|c| teil.iter().position(|x| x == c));
            v = next;
        }
        v.first().copied()
    }
    fn tauschen(&mut self, w: &Wechsel) {
        self.leagues[w.oben].teams.retain(|&x| x != w.ab);
        self.leagues[w.unten].teams.retain(|&x| x != w.auf);
        self.leagues[w.oben].teams.push(w.auf);
        self.leagues[w.ziel].teams.push(w.ab);
        self.clubs[w.ab].league = w.ziel;
        self.clubs[w.auf].league = w.oben;
        // leaving one of several regional groups: remember it, a later relegation brings the club back there
        if self.unter(w.oben).nth(1).is_some() { self.clubs[w.auf].heimat = Some(self.leagues[w.unten].key.clone()); }
        if self.unter(w.oben).nth(1).is_some() { self.clubs[w.ab].heimat = Some(self.leagues[w.ziel].key.clone()); }
    }
    /// The regional group a club goes down into: the group it played in before, its first team's group, the
    /// group of its region (assets/regionen.json, by its or its first team's name). Nothing fits: `sonst`, and
    /// true for "Staffel geschätzt".
    pub fn region_fuer(&self, c: usize, gruppen: &[usize], sonst: usize) -> (usize, bool) {
        if gruppen.len() < 2 { return (sonst, false); }
        let e = self.erste_von(c);
        let hat = |k: &str| gruppen.iter().copied().find(|&u| self.leagues[u].key == k);
        if let Some(u) = [Some(c), e].into_iter().flatten().find_map(|x| self.clubs[x].heimat.as_deref().and_then(hat)) { return (u, false); }
        let namen: Vec<String> = [Some(c), e].into_iter().flatten().map(|x| name_ohne_zahlen(&self.clubs[x].name)).collect();
        for (staffel, vereine) in regionen() {
            if !vereine.iter().any(|v| namen.contains(&name_ohne_zahlen(v))) { continue; }
            if let Some(u) = gruppen.iter().copied().find(|&u| self.leagues[u].name == *staffel) { return (u, false); }
        }
        (sonst, true)
    }
    /// Clubs playing in one of several groups of a tier get that group as home (start of a world, older saves).
    pub fn heimat_setzen(&mut self) {
        for o in 0..self.leagues.len() {
            let gruppen: Vec<usize> = self.unter(o).collect();
            if gruppen.len() < 2 { continue; }
            for u in gruppen { for c in self.leagues[u].teams.clone() { if self.clubs[c].heimat.is_none() { self.clubs[c].heimat = Some(self.leagues[u].key.clone()); } } }
        }
    }
    /// Season end over the final tables: direct moves, playouts, play-offs and relegation ties per league pair,
    /// second teams skipped (the next club moves up). Moves the clubs and returns what happened.
    pub fn saisonwechsel(&mut self, tables: &[Vec<Row>]) -> Vec<Wechsel> {
        let out = self.wechsel_rechnen(tables);
        for w in &out { self.tauschen(w); }
        self.entscheid.clear();
        out
    }
    fn art(&mut self, oben: usize, art: &'static str) { if self.planen.is_some() { self.planen = Some((oben, art)); } }
    fn wechsel_rechnen(&mut self, tables: &[Vec<Row>]) -> Vec<Wechsel> {
        let mut out = vec![];
        for oben in 0..self.leagues.len() {
            let unten: Vec<usize> = self.unter(oben).collect();
            if unten.is_empty() { continue; }
            let ro = self.regel(oben);
            let to: Vec<usize> = tables[oben].iter().map(|r| r.club).collect();
            let mut ab: Vec<(usize, &'static str)> = to.iter().rev().take(ro.ab).map(|&c| (c, crate::id("direkt"))).collect();
            if let Some(po) = &ro.playout {
                let p: Vec<usize> = po.plaetze.iter().filter_map(|&x| to.get(x.wrapping_sub(1)).copied()).filter(|c| !ab.iter().any(|d| d.0 == *c)).collect();
                self.art(oben, crate::id("playout"));
                for i in 0..(p.len() / 2).min(po.absteiger.max(1)) {
                    let (a, b) = (p[i], p[p.len() - 1 - i]);
                    let sieger = self.duell(a, b);
                    ab.push((if sieger == a { b } else { a }, "Playout")); // roh
                }
            }
            let mut auf: Vec<(usize, usize, &'static str)> = vec![];
            let mut gegner: Option<(usize, usize)> = None;
            for &u in &unten {
                let ru = self.regel(u);
                let order: Vec<usize> = tables[u].iter().map(|r| r.club).filter(|&c| self.darf_aufsteigen(c, oben)).collect();
                let k = ru.auf.min(order.len());
                auf.extend(order[..k].iter().map(|&c| (c, u, crate::id("direkt"))));
                let mut i = k;
                if let Some(po) = ru.playoffs.as_ref().filter(|p| p.ziel.is_none()) {
                    let teil: Vec<usize> = order[i..].iter().take(po.plaetze.len()).copied().collect();
                    i += teil.len();
                    if teil.len() >= 2 {
                        self.art(u, crate::id("playoff"));
                        let s = self.ko_runde(&teil).unwrap();
                        // with a relegation tie above, the play-off winner plays it; else it goes up when the rule says so
                        if ro.relegation.is_some() { if gegner.is_none() { gegner = Some((s, u)); } } else if po.aufsteiger > 0 { auf.push((s, u, crate::id("Play-off"))); }
                    }
                }
                if gegner.is_none() && ro.relegation.is_some() { if let Some(&c) = order.get(i) { gegner = Some((c, u)); } }
            }
            let n = ab.len().min(auf.len());
            for j in 0..n { let (ziel, geschaetzt) = self.region_fuer(ab[j].0, &unten, auf[j].1); out.push(Wechsel { ab: ab[j].0, auf: auf[j].0, oben, unten: auf[j].1, grund: if auf[j].2 == crate::id("direkt") { ab[j].1 } else { auf[j].2 }, ziel, geschaetzt }); }
            if let (Some(rel), Some((g, u))) = (&ro.relegation, gegner) {
                if let Some(&c) = to.get(rel.platz.wrapping_sub(1)) {
                    self.art(oben, crate::id("relegation"));
                    if !ab[..n].iter().any(|d| d.0 == c) && self.duell(c, g) == g { let (ziel, geschaetzt) = self.region_fuer(c, &unten, u); out.push(Wechsel { ab: c, auf: g, oben, unten: u, grund: "Relegation", ziel, geschaetzt }); } // roh
                }
            }
        }
        out
    }

    /// Next season's places in the UEFA competitions by the file: places in order, a place already taken (or a cup
    /// winner already in) passes to the next league place; `eps` countries (codes) get one more CL place.
    pub fn uefa_plaetze(&self, ranks: &std::collections::HashMap<String, Vec<usize>>, pokal: &std::collections::HashMap<String, usize>, eps: &[String], out: &mut [Vec<usize>; 3], taken: &mut std::collections::HashSet<usize>) {
        for (l, land) in self.uefa_ligen() {
            let rang: Vec<usize> = ranks.get(&self.leagues[l].key).map(|v| v.iter().copied().filter(|&c| self.erste_von(c).is_none()).collect()).unwrap_or_default();
            let cup = pokal.get(&self.leagues[l].land).copied().filter(|&c| self.erste_von(c).is_none());
            let naechster = |taken: &std::collections::HashSet<usize>| rang.iter().copied().find(|c| !taken.contains(c));
            let mut add = |t: usize, c: Option<usize>, taken: &mut std::collections::HashSet<usize>| if let Some(c) = c { if taken.insert(c) { out[t].push(c); } };
            for p in &land.europa.as_ref().unwrap().plaetze {
                let Some(t) = [crate::id("cl"), crate::id("el"), crate::id("uecl")].iter().position(|w| *w == p.wettbewerb) else { continue };
                for o in &p.plaetze {
                    let c = match o { Ort::Platz(n) => rang.get(n.wrapping_sub(1)).copied(), Ort::Pokal(s) if s == "pokal" => cup, Ort::Pokal(_) => None }.filter(|c| !taken.contains(c));
                    let c = c.or_else(|| naechster(taken));
                    add(t, c, taken);
                }
            }
            if eps.contains(&land.code) { let c = naechster(taken); add(0, c, taken); }
        }
    }
    /// The top league of every UEFA country with European places in the file.
    pub fn uefa_ligen(&self) -> Vec<(usize, &Land)> {
        let mut v: Vec<(usize, &Land)> = vec![];
        for l in 0..self.leagues.len() {
            if self.stufe_von(l) != 1 { continue; }
            let Some(land) = self.regel_land(l) else { continue };
            if land.verband != "UEFA" || land.europa.as_ref().map_or(true, |e| e.plaetze.is_empty()) || v.iter().any(|x| x.1.code == land.code) { continue; }
            v.push((l, land));
        }
        v
    }
    /// Performance places (EPS): the countries with the best average European points of this season.
    pub fn eps_berechnen(&self) -> Vec<String> {
        let mut v: Vec<(String, f32)> = vec![];
        for (l, land) in self.uefa_ligen() {
            let name = &self.leagues[l].land;
            let pts: f32 = self.coef.iter().filter(|x| &x.0 == name && x.1 == self.season).map(|x| x.2).sum();
            let n = self.cups.iter().filter(|c| (1..=3).contains(&c.kind)).flat_map(|c| c.teams.iter()).filter(|&&t| &self.leagues[self.clubs[t].league].land == name).count();
            if n > 0 { v.push((land.code.clone(), pts / n as f32)); }
        }
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        v.into_iter().take(self.ligaregeln.uefa.leistungsplaetze_eps).map(|x| x.0).collect()
    }
}

/// The points of each club at the split: (club, before, after).
pub fn split_punkte(table: &[Row], s: &Split) -> Vec<(usize, i32, i32)> {
    let in_gruppe = |i: usize| s.gruppen.iter().any(|g| i + 1 >= g.plaetze[0] && i + 1 <= g.plaetze[1]);
    table.iter().enumerate().map(|(i, r)| {
        let neu = if !in_gruppe(i) { r.pts } else { match s.punkte.as_str() { "halbiert_aufgerundet" => (r.pts + 1).div_euclid(2), "halbiert_abgerundet" => r.pts.div_euclid(2), "neu" => 0, _ => r.pts } };
        (r.club, r.pts, neu)
    }).collect()
}
pub fn split_gruppen(table: &[Row], s: &Split) -> Vec<Vec<usize>> {
    s.gruppen.iter().map(|g| table.iter().skip(g.plaetze[0].saturating_sub(1)).take((g.plaetze[1] + 1).saturating_sub(g.plaetze[0])).map(|r| r.club).collect()).collect()
}
/// Circle method: (round, home, away) for `runden` rounds; every second round robin swaps home and away.
pub fn rundenplan(ts: &[usize], runden: usize) -> Vec<(usize, usize, usize)> {
    let n = ts.len();
    if n < 2 { return vec![]; }
    let half = n - 1;
    let mut arr = ts.to_vec();
    let mut paare: Vec<Vec<(usize, usize)>> = vec![];
    for round in 0..half {
        // home and away swap every round for each slot: a club moving along one side then alternates (with (round + i) it
        // kept the same side round after round, Fabian 23.11.: six away matches in a row)
        paare.push((0..n / 2).map(|i| { let (a, b) = (arr[i], arr[n - 1 - i]); if round % 2 == 0 { (a, b) } else { (b, a) } }).collect());
        let last = arr.pop().unwrap();
        arr.insert(1, last);
    }
    (0..runden).flat_map(|rd| paare[rd % half].iter().map(move |&(h, a)| if (rd / half) % 2 == 0 { (rd, h, a) } else { (rd, a, h) }).collect::<Vec<_>>()).collect()
}

impl Career {
    /// Career start and load: the league rules (the career folder's own file wins) and corrections of the second-team
    /// list (zweite_mannschaften.json in the career folder: {"paare": {"second": "first", "not a second team": ""}}).
    pub fn ligaregeln_laden(&mut self) {
        let dir = self.karriere_dir();
        let (d, fehler) = laden(&dir);
        self.world.ligaregeln = d;
        if let Some(e) = fehler { self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), crate::inhalt::t("ui.ligaregeln.ligaregeln_aus_der_karriere_nicht"), &crate::tf!("ui.ligaregeln.es_gelten_die_eingebauten_regeln", e), crate::game::Action::None); }
        let Ok(t) = std::fs::read_to_string(dir.join("zweite_mannschaften.json")) else { return };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) else { return };
        let Some(o) = v["paare"].as_object() else { return };
        let finde = |n: &str| self.world.clubs.iter().position(|c| c.name == n);
        let paare: Vec<(Option<usize>, Option<usize>)> = o.iter().map(|(k, w)| (finde(k), w.as_str().and_then(|w| finde(w)))).collect();
        for (z, e) in paare {
            let Some(z) = z else { continue };
            self.world.zweite.retain(|x| x.0 != z);
            if let Some(e) = e { self.world.zweite.push((z, e)); }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn heim_und_auswaerts_wechseln_sich_ab() {
        // no club plays more than two home or two away matches in a row in a double round robin
        for n in [10usize, 18, 20] {
            let ts: Vec<usize> = (0..n).collect();
            let plan = super::rundenplan(&ts, 2 * (n - 1));
            for t in 0..n {
                let folge: Vec<bool> = (0..2 * (n - 1)).filter_map(|rd| plan.iter().find(|x| x.0 == rd && (x.1 == t || x.2 == t)).map(|x| x.1 == t)).collect();
                let mut serie = 1; let mut max = 1;
                for k in 1..folge.len() { if folge[k] == folge[k - 1] { serie += 1; max = max.max(serie); } else { serie = 1; } }
                assert!(max <= 2, "{n} clubs, club {t}: {max} in a row {folge:?}");
                assert_eq!(folge.iter().filter(|x| **x).count(), n - 1, "half at home");
            }
        }
    }

    use super::*;
    fn welt() -> World { let mut w = World::load(include_str!("../assets/world.json"), 5); w.zweite.clear(); w }
    fn liga(w: &World, k: &str) -> usize { w.leagues.iter().position(|l| l.key == k).unwrap() }
    fn als(w: &mut World, l: usize, land: &str, stufe: u8, ueber: Option<usize>) {
        w.leagues[l].land = land.into();
        w.leagues[l].stufe = stufe;
        w.leagues[l].ueber = ueber.map(|u| w.leagues[u].key.clone());
    }
    /// A final table in the order of the team list: club j gets 100 − j points.
    fn ordnen(w: &mut World, l: usize) -> Vec<usize> {
        let t = w.leagues[l].teams.clone();
        w.leagues[l].fixtures.clear();
        w.leagues[l].deduct = t.iter().enumerate().map(|(j, &c)| (c, j as i32 - 100)).collect();
        t
    }
    fn deutschland() -> (World, [usize; 3]) {
        let mut w = welt();
        let (bl, zwei, drei) = (liga(&w, "GERMANY_D1_LEAGUE"), liga(&w, "FL_011_LEAGUE"), liga(&w, "FL_049_LEAGUE"));
        als(&mut w, zwei, "Deutschland", 2, Some(bl));
        als(&mut w, drei, "Deutschland", 3, Some(zwei));
        (w, [bl, zwei, drei])
    }
    fn tabellen(w: &World) -> Vec<Vec<Row>> { (0..w.leagues.len()).map(|l| w.table(l)).collect() }

    #[test]
    fn relegation_wird_echt_gespielt() {
        let (mut w, [bl, zwei, _]) = deutschland();
        let t1 = ordnen(&mut w, bl);
        let t2 = ordnen(&mut w, zwei);
        for l in 0..w.leagues.len() { w.leagues[l].fixtures.clear(); }
        w.day = crate::world::LEAGUE_END;
        w.entscheidungen_planen();
        // the 16th of the Bundesliga against the 3rd of the 2. Bundesliga: two real matches, nothing decided yet
        let ci = w.cups.iter().position(|c| c.kind == 5 && c.key == w.leagues[bl].key).expect("Relegation als Wettbewerb");
        assert_eq!(w.cups[ci].fx.len(), 2);
        assert!(w.cups[ci].fx.iter().all(|x| [t1[15], t2[2]].contains(&x.f.h) && x.f.day > crate::world::LEAGUE_END));
        w.entscheidungen_planen();
        assert_eq!(w.cups[ci].fx.len(), 2, "nie doppelt angesetzt");
        // play the days; the season end takes the winner of the real matches
        for d in w.day..crate::date::SEASON_DAYS { w.day = d; w.cup_day(d, OFFEN); w.entscheidungen_planen(); }
        assert!(w.cups[ci].fx.iter().all(|x| x.f.hg >= 0));
        let sieger = w.winner_of(ci, 33, t1[15], t2[2]);
        let wechsel = w.saisonwechsel(&tabellen(&w));
        let rel = wechsel.iter().find(|x| x.grund == "Relegation");
        assert_eq!(rel.is_some(), sieger == t2[2], "Relegation folgt dem echten Ergebnis");
        assert!(w.entscheid.is_empty());
    }

    #[test]
    fn datei_laedt() {
        let d = eingebaut();
        assert_eq!(d.land.len(), 195);
        assert_eq!(d.land.iter().map(|l| l.liga.len()).sum::<usize>(), 306);
    }
    #[test]
    fn bundesliga_relegation_und_absteiger() {
        let (mut w, [bl, zwei, _]) = deutschland();
        let r = w.regel(bl);
        assert_eq!((r.ab, r.relegation.as_ref().map(|x| x.platz)), (2, Some(16)));
        assert_eq!(w.regel(zwei).auf, 2);
        let (tb, tz) = (ordnen(&mut w, bl), ordnen(&mut w, zwei));
        let t = tabellen(&w);
        let wechsel = w.saisonwechsel(&t);
        assert!(w.clubs[tb[16]].league == zwei && w.clubs[tb[17]].league == zwei, "17. und 18. steigen ab");
        assert!(w.clubs[tz[0]].league == bl && w.clubs[tz[1]].league == bl, "1. und 2. steigen auf");
        assert!((w.clubs[tb[15]].league == bl) != (w.clubs[tz[2]].league == bl), "16. gegen 3.: genau einer spielt oben");
        assert!(w.clubs[tb[14]].league == bl, "der 15. bleibt");
        assert_eq!(w.leagues[bl].teams.len(), 18);
        if w.clubs[tz[2]].league == bl { assert!(wechsel.iter().any(|x| x.grund == "Relegation" && x.auf == tz[2])); }
    }
    #[test]
    fn zweite_mannschaft_als_meister_steigt_nicht_auf() {
        let (mut w, [bl, zwei, drei]) = deutschland();
        let (tb, td) = (ordnen(&mut w, bl), ordnen(&mut w, drei));
        ordnen(&mut w, zwei);
        w.zweite = vec![(td[0], tb[0])];
        assert!(!w.darf_aufsteigen(td[0], zwei), "kein Aufstieg in die 2. Bundesliga");
        let t = tabellen(&w);
        w.saisonwechsel(&t);
        assert_eq!(w.clubs[td[0]].league, drei, "der Meister bleibt in der 3. Liga");
        assert!(w.clubs[td[1]].league == zwei && w.clubs[td[2]].league == zwei, "2. und 3. rücken auf");
    }
    #[test]
    fn belgien_teilt_nach_30_spieltagen() {
        let mut w = welt();
        let b = liga(&w, "BELGIUM_D1_LEAGUE");
        let weg: Vec<usize> = w.leagues[b].teams.split_off(16);
        let ziel = liga(&w, "FL_062_LEAGUE");
        for c in weg { w.clubs[c].league = ziel; w.leagues[ziel].teams.push(c); }
        w.day = 0;
        w.make_fixtures(b);
        assert_eq!(w.leagues[b].fixtures.len(), 30 * 8, "30 Spieltage bis zur Teilung");
        // results: the first club of each pairing in the team list wins, the table spreads out
        let teams = w.leagues[b].teams.clone();
        for f in w.leagues[b].fixtures.iter_mut() { let (ih, ia) = (teams.iter().position(|&x| x == f.h).unwrap(), teams.iter().position(|&x| x == f.a).unwrap()); (f.hg, f.ag) = if ih < ia { (2, 0) } else if (ih + ia) % 3 == 0 { (1, 1) } else { (0, 1) }; }
        let vorher = w.table(b);
        assert_eq!(w.split_pruefen(), vec![b]);
        let g = &w.leagues[b].gruppen;
        assert_eq!(g.iter().map(|x| x.len()).collect::<Vec<_>>(), vec![6, 6, 4]);
        assert_eq!(g[0], vorher[..6].iter().map(|r| r.club).collect::<Vec<_>>(), "Plätze 1–6");
        assert_eq!(g[2], vorher[12..].iter().map(|r| r.club).collect::<Vec<_>>(), "Plätze 13–16");
        let nachher = w.table(b);
        for r in &vorher { assert_eq!(nachher.iter().find(|x| x.club == r.club).unwrap().pts, (r.pts + 1) / 2, "halbiert, aufgerundet"); }
        assert_eq!(w.leagues[b].fixtures.len(), 240 + 30 + 30 + 12, "Doppelrunde in jeder Gruppe");
        assert!(w.leagues[b].fixtures[240..].iter().all(|f| g.iter().any(|x| x.contains(&f.h) && x.contains(&f.a))), "nur innerhalb der Gruppen");
        assert_eq!(nachher[..6].iter().map(|r| r.club).collect::<std::collections::HashSet<_>>(), g[0].iter().copied().collect(), "die Meisterrunde steht oben");
    }
    #[test]
    fn liga_ohne_eintrag_bekommt_den_standard() {
        let mut w = welt();
        let (a, b) = (liga(&w, "FL_062_LEAGUE"), liga(&w, "FL_074_LEAGUE"));
        als(&mut w, a, "Atlantis", 1, None);
        als(&mut w, b, "Atlantis", 2, Some(a));
        let (ra, rb) = (w.regel(a), w.regel(b));
        assert!(!ra.aus_datei && !ra.sicher && ra.playoffs.is_none() && ra.relegation.is_none() && ra.split.is_none());
        assert_eq!((w.leagues[a].teams.len(), ra.ab, ra.runden), (16, 2, 2), "16 Vereine: 2 Absteiger, Doppelrunde");
        assert_eq!((rb.auf, rb.ab), (2, 0), "so viele Aufsteiger wie oben absteigen; darunter keine Liga");
    }
    #[test]
    fn mexiko_endrunde_bestimmt_den_meister() {
        let mut w = welt();
        let l = liga(&w, "FL_062_LEAGUE");
        als(&mut w, l, "Mexiko", 1, None);
        assert_eq!(w.endrunde(l), Some(8), "Liguilla der besten 8");
        assert!(w.endrunde_hin_rueck(l), "Mexiko: Hin- und Rückspiel");
        w.make_fixtures(l);
        assert_eq!(w.leagues[l].fixtures.len(), 15 * 8, "Liga MX: eine einfache Runde");
        let letzter = w.leagues[l].fixtures.iter().map(|f| f.day).max().unwrap();
        for f in w.leagues[l].fixtures.iter_mut() { (f.hg, f.ag) = if f.h < f.a { (2, 1) } else { (0, 0) }; }
        w.day = letzter;
        assert_eq!(w.endrunden_pruefen(), vec![l]);
        assert!(w.endrunden_pruefen().is_empty(), "nur einmal je Saison");
        let ci = w.cups.iter().position(|c| c.kind == 4).expect("eigener Wettbewerb Endrunde");
        let top: Vec<usize> = w.table(l).iter().take(8).map(|r| r.club).collect();
        assert_eq!(w.cups[ci].teams, top);
        // the own club (the first of the table) plays its quarter-final in two single matches after the main round
        let own = top[0];
        let mut c = crate::game::Career::new(w, own, "Test");
        let eigene: Vec<u32> = c.world.cups[ci].fx.iter().filter(|x| x.f.h == own || x.f.a == own).map(|x| x.f.day).collect();
        assert_eq!(eigene.len(), 2, "Viertelfinale mit Hin- und Rückspiel");
        assert!(eigene[0] != eigene[1] && eigene.iter().all(|&d| d > letzter), "eigene Termine nach der Hauptrunde");
        assert_eq!(crate::cups::stage_name(c.world.cups[ci].stage, 4), "Viertelfinale");
        c.world.day = eigene[0];
        assert!(matches!(c.today_match(), Some((crate::game::Wo::Pokal(x), _)) if x == ci), "am Spieltag steht die Liguilla-Partie");
        // the rounds follow one another until a champion is found
        let mut runden = 0;
        while c.world.cups[ci].winner.is_none() && runden < 10 {
            for x in c.world.cups[ci].fx.iter_mut().filter(|x| x.f.hg < 0) { (x.f.hg, x.f.ag) = if x.f.h < x.f.a { (2, 0) } else { (1, 1) }; }
            c.world.cup_progress();
            runden += 1;
        }
        let m = c.world.cups[ci].winner.expect("ein Sieger");
        assert_eq!(c.world.leagues[l].meister, Some(m), "der Sieger der Endrunde ist Meister");
        assert!(top.contains(&m));
        let stufen: std::collections::BTreeSet<u8> = c.world.cups[ci].fx.iter().map(|x| x.stage).collect();
        assert_eq!(stufen.len(), 3, "Viertelfinale, Halbfinale, Finale");
        assert_eq!(c.world.cups[ci].fx.len(), 2 * (4 + 2 + 1), "alles mit Hin- und Rückspiel");
        let ar = liga(&c.world, "ARGENTINA_D1_LEAGUE");
        assert_eq!(c.world.endrunde(ar), Some(8), "Argentinien: Endrunde der besten 8");
        assert!(!c.world.endrunde_hin_rueck(ar), "Argentinien: Einzelspiele");
    }
    #[test]
    fn abgestiegene_zweite_kommt_in_ihre_regionalliga() {
        let (mut w, [bl, zwei, drei]) = deutschland();
        let (west, bayern) = (liga(&w, "FL_060_LEAGUE"), liga(&w, "FL_061_LEAGUE"));
        als(&mut w, west, "Deutschland", 4, Some(drei));
        als(&mut w, bayern, "Deutschland", 4, Some(drei));
        w.leagues[west].name = "Regionalliga West".into();
        w.leagues[bayern].name = "Regionalliga Bayern".into();
        let fcb = w.clubs.iter().position(|c| crate::farm::kern(&c.name) == crate::farm::kern("Bayern München")).expect("Bayern München in der Probewelt");
        let td = ordnen(&mut w, drei);
        for l in [bl, zwei, west, bayern] { ordnen(&mut w, l); }
        let letzter = *td.last().unwrap();
        w.zweite = vec![(letzter, fcb)];
        let t = tabellen(&w);
        let wechsel = w.saisonwechsel(&t);
        assert!(wechsel.iter().any(|x| x.ab == letzter), "der Letzte steigt ab");
        assert_eq!(w.clubs[letzter].league, bayern, "Bayerns zweite Mannschaft geht in die Regionalliga Bayern, nicht in den Westen");
        assert_eq!(w.clubs[letzter].heimat.as_deref(), Some(w.leagues[bayern].key.as_str()));
    }
    #[test]
    fn ohne_region_heisst_es_staffel_geschaetzt() {
        let (mut w, [_, _, drei]) = deutschland();
        let (a, b) = (liga(&w, "FL_060_LEAGUE"), liga(&w, "FL_061_LEAGUE"));
        als(&mut w, a, "Deutschland", 4, Some(drei));
        als(&mut w, b, "Deutschland", 4, Some(drei));
        let fremd = w.leagues[liga(&w, "BRAZIL_D1_LEAGUE")].teams[0];
        assert_eq!(w.region_fuer(fremd, &[a, b], b), (b, true), "keine Zuordnung: Staffel geschätzt");
        let c = w.leagues[a].teams[0];
        w.clubs[c].heimat = None;
        w.heimat_setzen();
        assert_eq!(w.region_fuer(c, &[a, b], b), (a, false), "wer in einer Staffel spielt, hat sie als Heimat");
    }
    /// Every tier with several groups in the FL worlds: each club of the league above has a group from the data,
    /// or the move says "Staffel geschätzt". Germany, Austria, Spain, Italy, England and France have no gaps.
    #[test]
    fn jede_staffel_hat_zuordnung_oder_hinweis() {
        for welt in ["_FL26Megapack", "_FL26-27-Def", "_FL26MyWorld"] {
            let p = std::path::PathBuf::from("/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk").join(welt);
            if !crate::fl27::echte_welt(&p) { continue; }
            let w = World::from_pesdb(&p.join("common/etc/pesdb"), 3).unwrap();
            for o in 0..w.leagues.len() {
                let gruppen: Vec<usize> = w.unter(o).collect();
                if gruppen.len() < 2 { continue; }
                let land = w.leagues[gruppen[0]].land.clone();
                let geschaetzt: Vec<String> = w.leagues[o].teams.iter().filter(|&&c| w.region_fuer(c, &gruppen, gruppen[0]).1).map(|&c| w.clubs[c].name.clone()).collect();
                let pflicht = ["Deutschland", "Österreich", "Spanien", "Italien", "England", "Frankreich"].iter().any(|x| land.contains(x)) || gruppen.iter().any(|&g| w.leagues[g].name.contains("(AUT)"));
                if pflicht { assert!(geschaetzt.is_empty(), "{welt} {}: ohne Zuordnung {:?}", w.leagues[o].name, geschaetzt); }
                eprintln!("{welt} {} → {} Staffeln, {} von {} Vereinen geschätzt", w.leagues[o].name, gruppen.len(), geschaetzt.len(), w.leagues[o].teams.len());
            }
        }
    }
    #[test]
    fn england_pokalsieger_schon_dabei() {
        let w = welt();
        let pl = liga(&w, "ENGLAND_D1_LEAGUE");
        let r = w.leagues[pl].teams.clone();
        let ranks = [(w.leagues[pl].key.clone(), r.clone())].into_iter().collect();
        let pokal = [("England".to_string(), r[2])].into_iter().collect();
        let (mut out, mut taken) = ([vec![], vec![], vec![]], Default::default());
        w.uefa_plaetze(&ranks, &pokal, &[], &mut out, &mut taken);
        let eng = |t: usize| out[t].iter().copied().filter(|c| r.contains(c)).collect::<Vec<_>>();
        assert_eq!(eng(0), r[..5].to_vec(), "1.–5. in die Champions League");
        assert_eq!(eng(1), vec![r[5], r[6]], "der Pokalplatz geht an den 6., der 6. Platz an den 7.");
        assert_eq!(eng(2), vec![r[7]], "kein Ligapokal: der nächste Ligaplatz");
    }
}
