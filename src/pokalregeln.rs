//! National cups by daten/pokalregeln.toml (Fabian 24.11.: "the FA Cup really has every English club, the DFB-Pokal
//! goes down to the Regionalliga"): who enters which round (world tiers, else amateur clubs with real names from
//! daten/amateure.toml), qualifying rounds among amateurs played on the side, byes, home right for the lower club,
//! two-legged semi-finals, penalties without extra time, a group stage.
//! Amateur clubs exist only in the cups: World::amateure, indices from AMA on (World::club), no players, no market.
use crate::world::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// Amateur clubs sit behind every real club index.
pub const AMA: usize = 1 << 24;

#[derive(Deserialize, Default)]
struct Datei { #[serde(default)] pokal: Vec<Pokal> }
#[derive(Deserialize, Default, Clone)]
pub struct Pokal {
    pub land: String,
    pub name: String,
    #[serde(default)] pub art: String,
    #[serde(default)] pub heimrecht: String,
    #[serde(default)] pub ablauf: Vec<RundeDef>,
}
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct RundeDef {
    #[serde(default)] pub name: String,
    #[serde(default)] pub neu: Vec<String>,
    #[serde(default)] pub quali: bool,
    #[serde(default)] pub hin_rueck: bool,
    #[serde(default)] pub elfmeter: bool,
    #[serde(default)] pub tiefste: bool,
    /// two pots: the higher half of the field (tier, then strength) drawn against the lower half (DFB-Pokal round 1)
    #[serde(default)] pub toepfe: bool,
    #[serde(default)] pub gruppen: usize,
    #[serde(default)] pub spiele: usize,
}

/// The plan of one cup this season (saved with the cup).
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Lauf {
    pub runden: Vec<RundeDef>,
    /// clubs entering per round
    pub zugang: Vec<Vec<usize>>,
    /// first day slot (Cup::days) per round
    pub slot: Vec<usize>,
    pub runde: usize,
    /// 0 draw, 1 the lower club at home, 2 the stronger club at home
    pub heim: u8,
    pub ligapokal: bool,
    /// stages decided on penalties without extra time
    pub ohne_vl: Vec<u8>,
    pub gruppen: Vec<Vec<usize>>,
    /// clubs of a qualifying round waiting for its day
    pub wartend: Vec<usize>,
    /// played qualifying rounds: name, clubs
    pub quali: Vec<(String, usize)>,
    pub ko_nr: u8,
}

pub fn pokale() -> &'static [Pokal] {
    static D: std::sync::OnceLock<Vec<Pokal>> = std::sync::OnceLock::new();
    D.get_or_init(|| toml::from_str::<Datei>(include_str!("../daten/pokalregeln.toml")).unwrap_or_else(|e| panic!("daten/pokalregeln.toml: {e}")).pokal) // roh
}
fn amateur_liste() -> &'static BTreeMap<String, Vec<(String, u8)>> {
    static D: std::sync::OnceLock<BTreeMap<String, Vec<(String, u8)>>> = std::sync::OnceLock::new();
    D.get_or_init(|| toml::from_str(include_str!("../daten/amateure.toml")).unwrap_or_else(|e| panic!("daten/amateure.toml: {e}"))) // roh
}
/// The round of a plan for `n` clubs left: the last one for 2, the one before for 4, …
fn lauf_def(c: &crate::cups::Cup, n: usize) -> RundeDef {
    let l = c.lauf.as_ref().unwrap();
    let zurueck = n.trailing_zeros() as usize;
    l.runden.len().checked_sub(zurueck).and_then(|i| l.runden.get(i)).cloned().unwrap_or_default()
}
fn norm(s: &str) -> String { s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect() }

impl World {
    pub fn ist_amateur(&self, t: usize) -> bool { t >= AMA }
    /// A club of the world or an amateur club of the cups.
    pub fn club(&self, t: usize) -> &Club { if t >= AMA { &self.amateure[t - AMA].0 } else { &self.clubs[t] } }
    /// Tier of a club (amateurs: their real tier).
    pub fn pokal_stufe(&self, t: usize) -> u8 { if t >= AMA { self.amateure[t - AMA].1 } else { self.leagues[self.clubs[t].league].stufe.max(1) } }

    fn amateur_neu(&mut self, name: &str, stufe: u8, land: &str, basis: (usize, u8, f32, f32)) -> usize {
        let (liga, unten, level, stat) = basis;
        let mut c = self.clubs[self.leagues[liga].teams[0]].clone();
        let i = self.amateure.len();
        let ab = (stufe.saturating_sub(unten)).max(1) as f32 * 3.5 + self.rng.gauss(1.5);
        c.id = u32::MAX - i as u32;
        c.name = name.to_string();
        c.short = name.split_whitespace().filter(|w| w.chars().next().map_or(false, |x| x.is_uppercase())).map(|w| w.chars().next().unwrap()).take(3).collect::<String>();
        if c.short.len() < 2 { c.short = name.chars().filter(|x| x.is_alphanumeric()).take(3).collect::<String>().to_uppercase(); }
        c.league = liga;
        c.level = (level - ab).max(20.0);
        c.static_str = (stat - ab).max(20.0);
        c.style = 0;
        c.squad = vec![];
        c.balance = 0.0;
        c.rep = 5.0;
        c.trainer = None;
        c.heimat = None;
        c.c1 = 0x6b7280 + (i as u32 % 7) * 0x0a0a0a;
        let _ = land;
        self.amateure.push((c, stufe));
        AMA + i
    }

    /// Creates one national cup by its rules (or all first teams of the country without a plan); None below 8 clubs.
    pub fn pokal_anlegen(&mut self, land: &str, name: String, key: String, regel: Option<&Pokal>, euro: &HashSet<usize>, busy: &mut HashSet<(usize, u32)>) -> Option<usize> {
        // the cup's own random numbers (season and country), so hundreds of amateur clubs leave the world's draws alone
        let welt_rng = self.rng.clone();
        self.rng = Rng(welt_rng.0 ^ land.bytes().fold(self.season as u64 * 0x9E37_79B9, |h, b| h.rotate_left(5) ^ b as u64));
        let ci = self.pokal_anlegen_mit(land, name, key, regel, euro, busy);
        self.rng = welt_rng;
        ci
    }
    fn pokal_anlegen_mit(&mut self, land: &str, name: String, key: String, regel: Option<&Pokal>, euro: &HashSet<usize>, busy: &mut HashSet<(usize, u32)>) -> Option<usize> {
        // the country's first teams by tier
        let mut stufen: BTreeMap<u8, Vec<usize>> = BTreeMap::new();
        for l in self.leagues.iter().filter(|l| l.land == land) { stufen.entry(l.stufe.max(1)).or_default().extend(l.teams.iter().copied().filter(|&t| self.erste_von(t).is_none())); }
        let alle: Vec<usize> = stufen.values().flatten().copied().collect();
        if alle.len() < 8 { return None; }
        for v in stufen.values_mut() { v.sort_by(|a, b| self.clubs[*b].level.partial_cmp(&self.clubs[*a].level).unwrap()); }
        // amateurs: the lowest tier of the world as their measure
        let (&unten, lt) = stufen.iter().next_back().unwrap();
        let liga = self.clubs[lt[0]].league;
        let level = lt.iter().map(|&t| self.clubs[t].level).sum::<f32>() / lt.len() as f32;
        let stat = lt.iter().map(|&t| self.clubs[t].static_str).sum::<f32>() / lt.len() as f32;
        let welt: HashSet<String> = self.clubs.iter().map(|c| norm(&c.name)).collect();
        let mut pool: BTreeMap<u8, Vec<String>> = BTreeMap::new();
        for (n, s) in amateur_liste().get(land).map(|v| v.as_slice()).unwrap_or(&[]) {
            if !welt.contains(&norm(n)) { pool.entry(*s).or_default().push(n.clone()); }
        }
        for v in pool.values_mut() { self.shuffle_pub(v); }
        let mut runden: Vec<RundeDef> = regel.map(|r| r.ablauf.clone()).unwrap_or_default();
        if runden.is_empty() { runden.push(RundeDef { neu: vec![crate::id("alle").into()], ..Default::default() }); }
        let mut zugang: Vec<Vec<usize>> = vec![vec![]; runden.len()];
        let mut genommen: HashSet<usize> = HashSet::new();
        // explicit picks first, "rest" last
        for pass in 0..2 {
            for (k, r) in runden.iter().enumerate() {
                for sel in &r.neu {
                    let teile: Vec<&str> = sel.split(':').collect();
                    if (teile.get(2) == Some(&"rest")) != (pass == 1) { continue; } // roh
                    let mut neu: Vec<usize> = vec![];
                    match teile[0] {
                        "alle" => neu = alle.clone(), // roh
                        "beste" => { let mut v = alle.clone(); v.sort_by(|a, b| self.clubs[*b].level.partial_cmp(&self.clubs[*a].level).unwrap()); neu = v.into_iter().filter(|t| !genommen.contains(t)).take(teile.get(1).and_then(|x| x.parse().ok()).unwrap_or(0)).collect(); } // roh
                        "stufe" | "ab" | "landespokal" => { // roh
                            let s: u8 = teile.get(1).and_then(|x| x.parse().ok()).unwrap_or(1);
                            let arg = teile.get(2).copied().unwrap_or("");
                            if teile[0] == "landespokal" { // roh
                                // K regional cup winners: a share p of them third-tier clubs outside the top four, else amateurs
                                let p: f32 = teile.get(2).and_then(|x| x.parse().ok()).unwrap_or(0.25);
                                let mut dritte: Vec<usize> = stufen.get(&3).cloned().unwrap_or_default().into_iter().filter(|t| !genommen.contains(t)).collect();
                                self.shuffle_pub(&mut dritte);
                                for _ in 0..s {
                                    if !dritte.is_empty() && self.rng.chance(p) { neu.push(dritte.pop().unwrap()); continue; }
                                    let st = pool.keys().copied().find(|&x| x > unten && pool[&x].len() > 0);
                                    if let Some(st) = st { let n = pool.get_mut(&st).unwrap().pop().unwrap(); neu.push(self.amateur_neu(&n, st, land, (liga, unten, level, stat))); }
                                }
                            } else if teile[0] == "ab" { // roh
                                let k: usize = arg.parse().unwrap_or(0);
                                let mut namen: Vec<(String, u8)> = pool.iter().filter(|(&x, _)| x >= s).flat_map(|(&x, v)| v.iter().map(move |n| (n.clone(), x))).collect();
                                self.shuffle_pub(&mut namen);
                                for (n, st) in namen.into_iter().take(k) { pool.get_mut(&st).unwrap().retain(|x| *x != n); neu.push(self.amateur_neu(&n, st, land, (liga, unten, level, stat))); }
                            } else if let Some(v) = stufen.get(&s) {
                                let frei: Vec<usize> = v.iter().copied().filter(|t| !genommen.contains(t)).collect();
                                neu = match arg {
                                    "europa" => frei.into_iter().filter(|t| euro.contains(t)).collect(), // roh
                                    "ohne_europa" => frei.into_iter().filter(|t| !euro.contains(t)).collect(), // roh
                                    "" | "rest" => frei,
                                    x if x.starts_with('-') => { let k: usize = x[1..].parse().unwrap_or(0); frei[frei.len().saturating_sub(k)..].to_vec() }
                                    x => { let k: usize = x.parse().unwrap_or(0); frei.into_iter().take(k).collect() }
                                };
                            } else {
                                // no league of this tier in the world: amateur clubs of that tier
                                let k = match arg { "" | "rest" => usize::MAX, x => x.trim_start_matches('-').parse().unwrap_or(usize::MAX) };
                                let namen: Vec<String> = pool.get_mut(&s).map(|v| { let n = v.len().min(k); v.split_off(v.len() - n) }).unwrap_or_default();
                                for n in namen { neu.push(self.amateur_neu(&n, s, land, (liga, unten, level, stat))); }
                            }
                        }
                        _ => {}
                    }
                    for t in neu { if genommen.insert(t) { zugang[k].push(t); } }
                }
            }
        }
        let teams: Vec<usize> = zugang.iter().flatten().copied().collect();
        if teams.len() < 8 { return None; }
        let mut slot = vec![];
        let mut n = 0;
        for r in &runden { slot.push(n); n += if r.gruppen > 0 { r.spiele.max(1) } else { 1 }; }
        // comp.py cup_days: Tuesdays between day 24 and 296 outside the breaks, spread, the final on the last one
        let tue: Vec<u32> = (24..=296).filter(|d| d % 7 == 3 && !INTL_BREAKS.iter().any(|(a, b)| d >= a && d <= b)).collect();
        let n = n.max((teams.len() as f32).log2().ceil() as usize) + 1; // one spare day for a play-in round
        let mut days: Vec<u32> = if n > 1 { (0..n - 1).map(|i| tue[i * (tue.len() - 2) / (n - 1).max(1)]).collect() } else { vec![] };
        days.push(*tue.last().unwrap());
        let heim = match regel.map(|r| r.heimrecht.as_str()) { Some("unterklassig") => 1, Some("gesetzt") => 2, _ => 0 }; // roh
        let ligapokal = regel.map_or(false, |r| r.art == crate::id("ligapokal"));
        let ci = self.cups.len();
        let lauf = Lauf { runden, zugang, slot, heim, ligapokal, ..Default::default() };
        self.cups.push(crate::cups::Cup { name, key, kind: 0, country: land.to_string(), teams, fx: vec![], stage: 20, alive: vec![], winner: None, days, lauf: Some(Box::new(lauf)) });
        self.runde_ansetzen(ci, 0, vec![], busy);
        Some(ci)
    }
    pub fn shuffle_pub<T>(&mut self, v: &mut [T]) { for i in (1..v.len()).rev() { let j = (self.rng.next() % (i as u64 + 1)) as usize; v.swap(i, j); } }

    /// Round k of a cup with a plan: the clubs still in plus the ones entering now; a qualifying round waits for its
    /// day, a group stage is drawn, a knock-out round is paired (the strongest get byes: one when the number is odd,
    /// after the last entry as many as needed for a power of two).
    pub fn runde_ansetzen(&mut self, ci: usize, k: usize, mut teams: Vec<usize>, busy: &mut HashSet<(usize, u32)>) {
        let Some(lauf) = self.cups[ci].lauf.as_deref() else { return };
        if let Some(z) = lauf.zugang.get(k) { for &t in z { if !teams.contains(&t) { teams.push(t); } } }
        let r = lauf.runden.get(k).cloned().unwrap_or_default();
        let spaeter = lauf.zugang.iter().skip(k + 1).any(|z| !z.is_empty());
        let neu: Vec<usize> = lauf.zugang.get(k).cloned().unwrap_or_default();
        let tag = |w: &World, j: usize| { let d = &w.cups[ci].days; d[j.min(d.len() - 1)] };
        let d0 = tag(self, lauf.slot.get(k).copied().unwrap_or(lauf.slot.last().map_or(0, |s| s + k + 1 - lauf.slot.len())));
        // never before the last match of the round before (a play-in round or a second leg can push it)
        let d0 = d0.max(self.cups[ci].fx.iter().map(|x| x.f.day + 7).max().unwrap_or(0));
        let (heim, slot0) = (lauf.heim, lauf.slot.get(k).copied().unwrap_or(0));
        self.cups[ci].lauf.as_mut().unwrap().runde = k;
        if teams.len() <= 1 { if let Some(&w) = teams.first() { self.cup_won(ci, w); } return; }
        if r.quali { self.cups[ci].lauf.as_mut().unwrap().wartend = teams; return; }
        teams.sort_by(|a, b| self.club(*b).level.partial_cmp(&self.club(*a).level).unwrap());
        if r.gruppen > 0 {
            // pots by strength, snake into the groups; within a group every club meets `spiele` others (circle method)
            let g = r.gruppen.min(teams.len() / 2).max(1);
            let mut gr: Vec<Vec<usize>> = vec![vec![]; g];
            for (i, &t) in teams.iter().enumerate() { let row = i / g; let j = if row % 2 == 0 { i % g } else { g - 1 - i % g }; gr[j].push(t); }
            for (gi, grp) in gr.iter().enumerate() {
                let mut m = grp.clone();
                self.shuffle_pub(&mut m);
                if m.len() % 2 == 1 { m.push(usize::MAX); }
                let n = m.len();
                for rd in 0..r.spiele.max(1).min(n - 1) {
                    for i in 0..n / 2 {
                        let (a, b) = (m[i], m[n - 1 - i]);
                        if a == usize::MAX || b == usize::MAX { continue; }
                        let (h, x) = if (rd + i + gi) % 2 == 0 { (a, b) } else { (b, a) };
                        let d = tag(self, slot0 + rd);
                        self.add_fx(ci, busy, h, x, d, rd as u8, 0, false);
                    }
                    let last = m.pop().unwrap();
                    m.insert(1, last);
                }
            }
            self.cups[ci].stage = 0;
            self.cups[ci].alive = vec![];
            self.cups[ci].lauf.as_mut().unwrap().gruppen = gr;
            return;
        }
        let n = teams.len();
        let s = if spaeter { (n + 1) / 2 } else { ((n + 1) / 2).next_power_of_two() };
        let m = n - s;
        // after the last entry the rounds are counted from the final back (a play-in round shifts no semi-final)
        let r = if !spaeter && n.is_power_of_two() { lauf_def(&self.cups[ci], n) } else { r };
        let byes: Vec<usize> = teams.drain(..n - 2 * m).collect();
        // pairs: new entrants against the lowest clubs (Copa del Rey: the Supercopa four), else drawn
        let mut paare: Vec<(usize, usize)> = vec![];
        if r.tiefste {
            let (mut oben, mut rest): (Vec<usize>, Vec<usize>) = teams.iter().partition(|t| neu.contains(t));
            rest.sort_by_key(|&t| std::cmp::Reverse(self.pokal_stufe(t)));
            while let (Some(a), true) = (oben.pop(), !rest.is_empty()) { paare.push((a, rest.remove(0))); }
            teams = oben.into_iter().chain(rest).collect();
        }
        if r.toepfe {
            teams.sort_by(|a, b| (self.pokal_stufe(*a), -self.club(*a).level as i32).cmp(&(self.pokal_stufe(*b), -self.club(*b).level as i32)));
            let mut unten = teams.split_off(teams.len() / 2);
            self.shuffle_pub(&mut unten);
            paare.extend(teams.iter().copied().zip(unten.iter().copied()));
            teams.clear();
        }
        self.shuffle_pub(&mut teams);
        paare.extend(teams.chunks(2).filter(|p| p.len() == 2).map(|p| (p[0], p[1])));
        let finale = m == 1 && byes.is_empty();
        let ko_nr = self.cups[ci].lauf.as_ref().unwrap().ko_nr;
        let stage = if finale { 34 } else if r.hin_rueck { 33 } else { (20 + ko_nr).min(29) };
        {
            let l = self.cups[ci].lauf.as_mut().unwrap();
            if !finale && !r.hin_rueck { l.ko_nr += 1; }
            if r.elfmeter && !l.ohne_vl.contains(&stage) { l.ohne_vl.push(stage); }
        }
        for (a, b) in paare {
            // home right: the lower club (higher tier number) or the stronger club, else as drawn
            let (h, x) = match heim {
                1 if self.pokal_stufe(b) > self.pokal_stufe(a) => (b, a),
                2 if self.club(b).level > self.club(a).level => (b, a),
                _ => (a, b),
            };
            if r.hin_rueck && !finale { self.add_fx(ci, busy, x, h, d0, stage, 1, false); self.add_fx(ci, busy, h, x, d0 + 7, stage, 2, false); } else { self.add_fx(ci, busy, h, x, d0, stage, 0, finale); }
        }
        self.cups[ci].stage = stage;
        self.cups[ci].alive = byes;
    }

    /// A qualifying round among amateurs on its day: drawn, every tie decided at once (90 minutes, then penalties).
    pub fn quali_tag(&mut self, day: u32) {
        let mut busy: Option<HashSet<(usize, u32)>> = None;
        for ci in 0..self.cups.len() {
            let Some(l) = self.cups[ci].lauf.as_deref() else { continue };
            if l.wartend.is_empty() || self.cups[ci].winner.is_some() { continue; }
            let k = l.runde;
            let d = self.cups[ci].days[l.slot.get(k).copied().unwrap_or(0).min(self.cups[ci].days.len() - 1)];
            if day < d { continue; }
            let spaeter = l.zugang.iter().skip(k + 1).any(|z| !z.is_empty());
            let name = l.runden.get(k).map(|r| r.name.clone()).unwrap_or_default();
            let mut teams = std::mem::take(&mut self.cups[ci].lauf.as_mut().unwrap().wartend);
            let n = teams.len();
            teams.sort_by(|a, b| self.club(*b).level.partial_cmp(&self.club(*a).level).unwrap());
            let s = if spaeter { (n + 1) / 2 } else { ((n + 1) / 2).next_power_of_two() };
            let mut weiter: Vec<usize> = teams.drain(..n - 2 * (n - s)).collect();
            self.shuffle_pub(&mut teams);
            for p in teams.chunks(2) {
                let (lh, la) = World::rates(self.club(p[0]).static_str, self.club(p[1]).static_str, 0, 0);
                let (hg, ag) = (self.rng.poisson(lh), self.rng.poisson(la));
                weiter.push(if hg > ag || (hg == ag && self.rng.chance(0.5)) { p[0] } else { p[1] });
            }
            self.cups[ci].lauf.as_mut().unwrap().quali.push((name, n));
            let busy = busy.get_or_insert_with(|| self.busy_days());
            self.runde_ansetzen(ci, k + 1, weiter, busy);
        }
    }

    /// A cup match with an amateur club: static strengths, the real club's league profile.
    pub fn spiel_amateur(&mut self, h: usize, a: usize) -> MatchResult {
        let (sh, sa) = (self.club(h).static_str, self.club(a).static_str);
        let (mut lh, mut la) = World::rates(sh, sa, self.club(h).style, self.club(a).style);
        let echt = if self.ist_amateur(h) { a } else { h };
        let (ph, pa, remis) = if self.ist_amateur(echt) { (HOME_GOALS, AWAY_GOALS, 1.0) } else { self.liga_profil(echt) };
        lh *= ph / HOME_GOALS;
        la *= pa / AWAY_GOALS;
        if std::mem::take(&mut self.neutral) { let m = (ph + pa) / 2.0; lh *= m / ph; la *= m / pa; }
        let cells = World::pmf(lh, la);
        let w: Vec<f32> = cells.iter().map(|c| if c.0 == c.1 { c.2 * remis } else { c.2 }).collect();
        let i = self.rng.weighted(&w);
        MatchResult { h, a, hg: cells[i].0 as u8, ag: cells[i].1 as u8, sh, sa, lh, la, ..Default::default() }
    }

    /// After a group stage: the top two of every group and the best third-placed clubs up to a power of two.
    pub fn gruppen_weiter(&self, ci: usize) -> Vec<usize> {
        let Some(l) = self.cups[ci].lauf.as_deref() else { return vec![] };
        let tab = self.cup_table(ci);
        let mut weiter = vec![];
        let mut dritte = vec![];
        for g in &l.gruppen {
            let rows: Vec<&Row> = tab.iter().filter(|r| g.contains(&r.club)).collect();
            weiter.extend(rows.iter().take(2).map(|r| r.club));
            dritte.extend(rows.iter().skip(2).map(|r| (r.pts, r.gf - r.ga, r.gf, r.club)));
        }
        let ziel = weiter.len().next_power_of_two();
        dritte.sort_by(|a, b| (b.0, b.1, b.2).cmp(&(a.0, a.1, a.2)));
        weiter.extend(dritte.into_iter().take(ziel - weiter.len()).map(|x| x.3));
        weiter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dateien_lesbar() {
        assert!(pokale().iter().any(|p| p.name == "DFB-Pokal" && !p.ablauf.is_empty()));
        assert!(amateur_liste()["England"].len() > 600);
    }
    /// One season with the real FL world: fields as in the rules, every cup ends with a winner, no club twice a day,
    /// amateur clubs never in the honours.
    #[test]
    fn echte_pokale_eine_saison() {
        let p = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&p) { return; }
        let mut w = World::from_pesdb(&p, 5).unwrap();
        let groesse = |w: &World, n: &str| w.cups.iter().find(|c| c.name == n).map_or(0, |c| c.teams.len());
        for c in w.cups.iter().filter(|c| c.kind == 0) { println!("{} ({}): {} Vereine, {} Amateure", c.name, c.country, c.teams.len(), c.teams.iter().filter(|&&t| t >= AMA).count()); }
        assert_eq!(groesse(&w, "DFB-Pokal"), 64);
        assert_eq!(groesse(&w, "FA Cup"), 747);
        assert_eq!(groesse(&w, "EFL Cup"), 92);
        assert_eq!(groesse(&w, "Coppa Italia"), 44);
        let dfb = w.cups.iter().find(|c| c.name == "DFB-Pokal").unwrap();
        assert!(dfb.fx.iter().all(|x| w.pokal_stufe(x.f.h) >= w.pokal_stufe(x.f.a)), "the lower club at home");
        assert!(dfb.fx.iter().all(|x| !(w.ist_amateur(x.f.h) && w.ist_amateur(x.f.a))), "round 1: two pots, amateurs only against professionals");
        for s in 0..2 {
            for day in 0..crate::date::SEASON_DAYS {
                w.day = day;
                for l in 0..w.leagues.len() {
                    for fi in 0..w.leagues[l].fixtures.len() {
                        let f = w.leagues[l].fixtures[fi];
                        if f.day == day && f.hg < 0 { let r = w.play(f.h, f.a, 2, None, None); w.apply_result(l, fi, &r); }
                    }
                }
                w.cup_day(day, usize::MAX);
            }
            let offen: Vec<_> = w.cups.iter().filter(|c| c.kind <= 3 && c.winner.is_none()).map(|c| (c.name.clone(), c.stage, c.fx.iter().filter(|x| x.f.hg < 0).count())).collect();
            for c in w.cups.iter().filter(|c| c.winner.is_none() && c.kind <= 3) { println!("OFFEN {} days {:?} fx {:?}", c.name, c.days, c.fx.iter().map(|x| (x.stage, x.leg, x.f.day, x.f.hg)).collect::<Vec<_>>()); }
            assert!(offen.is_empty(), "season {s}: {offen:?}");
            let mut seen = HashSet::new();
            for l in &w.leagues { for f in &l.fixtures { seen.insert((f.h, f.day)); seen.insert((f.a, f.day)); } }
            for c in &w.cups { for x in &c.fx { assert!(seen.insert((x.f.h, x.f.day)) && seen.insert((x.f.a, x.f.day)), "{}: club plays twice on day {}", c.name, x.f.day); } }
            for c in w.cups.iter().filter(|c| c.kind == 0 && c.teams.len() > 2) {
                let w0 = c.winner.unwrap();
                let runden: Vec<u8> = { let mut v: Vec<u8> = c.fx.iter().map(|x| x.stage).collect(); v.dedup(); v };
                println!("{}: Sieger {} ({}. Liga), Runden {:?}, Quali {:?}", c.name, w.club(w0).name, w.pokal_stufe(w0), runden, c.lauf.as_ref().map(|l| l.quali.len()));
                let ama: Vec<(u8, usize)> = runden.iter().map(|&st| (st, c.fx.iter().filter(|x| x.stage == st && x.leg <= 1).filter(|x| w.ist_amateur(x.f.h) || w.ist_amateur(x.f.a)).count())).collect();
                println!("   Spiele mit Amateuren je Runde: {ama:?}");
                let fin: Vec<_> = c.fx.iter().filter(|x| x.stage == 34).collect();
                assert_eq!(fin.len(), 1, "{}: one final", c.name);
            }
            assert!(w.titles.iter().all(|t| t.club < AMA));
            let uecl = w.cups.iter().find(|c| c.kind == 3).unwrap();
            assert!(uecl.teams.iter().all(|&t| uecl.fx.iter().filter(|x| x.stage < 10 && (x.f.h == t || x.f.a == t)).count() == 6), "Conference League: six league-phase matches");
            if s == 1 {
                for n in ["Supercopa de España", "Supercoppa Italiana", "DFL-Supercup", "Johan Cruijff Schaal"] { assert!(w.cups.iter().any(|c| c.name == n && c.winner.is_some()), "{n}"); }
                assert_eq!(w.cups.iter().find(|c| c.name == "Supercopa de España").unwrap().teams.len(), 4);
            }
            let q = w.euro_qualify();
            assert!(q.iter().flatten().all(|&t| t < AMA));
            w.season_honours();
            w.season += 1;
            for l in 0..w.leagues.len() { w.make_fixtures(l); }
            w.setup_cups(Some(q));
        }
    }
}
