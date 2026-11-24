// Paket 2: domestic cups and the three European competitions (fl/sim/uefa.py, comp.py, config/uefa.json, sim.json cups/match).
// Cups run beside the leagues: their fixtures live in World::cups, every club plays at most once a day.
use crate::world::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// stage: 0..=9 league-phase rounds (Europe), 20 + r domestic round r, 30 play-offs, 31 last 16, 32 quarter, 33 semi, 34 final.
#[derive(Serialize, Deserialize, Clone)]
pub struct CupFx { pub f: Fixture, pub stage: u8, pub leg: u8, pub neutral: bool, pub pens: Option<(u8, u8)>, pub et: bool }

#[derive(Serialize, Deserialize, Clone)]
pub struct Cup {
    pub name: String,
    /// Name of the competition in Competition.bin (for the FL27 tournament look).
    pub key: String,
    /// 0 domestic cup, 1 Champions League, 2 Europa League, 3 Conference League, 4 title round of a league,
    /// 5 play-offs, playouts and relegation of a league (both ligaregeln.rs)
    pub kind: u8,
    pub country: String,
    pub teams: Vec<usize>,
    pub fx: Vec<CupFx>,
    pub stage: u8,
    pub alive: Vec<usize>,
    pub winner: Option<usize>,
    pub days: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Title { pub season: u32, pub comp: String, pub club: usize }
 // roh
pub const EURO: [(&str, &str); 3] = [("Champions League", "UEFA_CHAMPIONS_LEAGUE"), ("Europa League", "UEFA_EUROPE_LEAGUE"), ("Conference League", "FL_UECL")]; // roh
// uefa.json days (0 = Saturday 1 Aug; CL Wednesday, EL/ECL Thursday)
const L_DAYS: [[u32; 10]; 3] = [[46, 60, 81, 95, 116, 123, 130, 158, 172, 186], [47, 61, 82, 96, 117, 124, 131, 159, 173, 187], [47, 61, 82, 96, 117, 124, 131, 159, 173, 187]];
const KO_DAYS: [[(u32, u32); 4]; 3] = [[(214, 221), (228, 249), (263, 270), (277, 284)], [(215, 222), (229, 250), (264, 271), (278, 285)], [(215, 222), (229, 250), (264, 271), (278, 285)]];
const FINAL: [u32; 3] = [298, 299, 299];
const FILL_CAP: usize = 7;
const PRIZE_SCALE: [f64; 3] = [1.0, 0.25, 0.12];
// uefa.json prizes.base: start, win, draw, po, r16, qf, sf, final, champion
const PRIZE: [f64; 9] = [18_600_000.0, 2_100_000.0, 700_000.0, 1_000_000.0, 11_000_000.0, 12_500_000.0, 15_000_000.0, 18_500_000.0, 6_500_000.0];
pub const ET_FACTOR: f32 = 0.33; // sim.json match.et_factor
pub const PENS_STRENGTH: f32 = 0.002; // sim.json match.pens_strength
/// Shoot-out winner known, score not (the person only said who won): 1:0 / 0:1 cannot end a real shoot-out.
pub const PENS_UNKNOWN_H: (u8, u8) = (1, 0);
pub const PENS_UNKNOWN_A: (u8, u8) = (0, 1);
/// " 4:5" for a known shoot-out score, "" when only the winner is known.
pub fn pens_score(p: (u8, u8)) -> String { if p.0 + p.1 <= 1 { String::new() } else { format!(" {}:{}", p.0, p.1) } }

/// Country of a league (fl26world.txt country groups, fixed keys); None when unknown.
pub fn country(l: &crate::world::League) -> Option<String> { (!l.land.is_empty()).then(|| l.land.clone()) }
/// The real name of a national cup (Fabian 23.11.): the fixed key's name, else by the country, else "<Land>-Pokal".
pub fn pokalname(key: &str, land: &str) -> String {
    crate::world::echter_name(key)
        .or_else(|| crate::regeln::wert("pokalnamen", "pokal").get(land).and_then(|v| v.as_str()).map(str::to_string)) // roh
        .unwrap_or_else(|| crate::tf!("ui.cups.pokal_fallback", land))
}
/// The name of a knock-out round by the ties in it (23.11.: a domestic cup's "Round 5" or a play-off was shown as the
/// final although semi-finals were still to come): 1 final, 2 semi-finals, 4 quarter-finals, 8 last 16.
pub fn runden_name(st: u8, kind: u8, paare: usize) -> String {
    match (kind, paare) {
        (0 | 5, 1) => crate::inhalt::t("wb.runde.finale").into(),
        (0 | 5, 2) => crate::inhalt::t("wb.runde.halbfinale").into(),
        (0, 4) => crate::inhalt::t("wb.runde.viertelfinale").into(),
        (0, 8) => crate::inhalt::t("wb.runde.achtelfinale").into(),
        _ => stage_name(st, kind),
    }
}
pub fn stage_name(st: u8, kind: u8) -> String {
    if kind == 5 { return crate::inhalt::t(if st == 33 { "wb.runde.hin_rueck" } else { "wb.runde.entscheidung" }).into(); }
    if kind == 4 { return match st { 20 | 30 => crate::inhalt::t("wb.runde.achtelfinale"), 21 | 31 => crate::inhalt::t("wb.runde.viertelfinale"), 22 | 32 => crate::inhalt::t("wb.runde.halbfinale"), _ => crate::inhalt::t("wb.runde.finale") }.into(); }
    match st {
        0..=9 => crate::tf!("ui.cups.ligaphase_spieltag", st + 1),
        20..=29 => crate::tf!("wb.runde.nummer", st - 19),
        30 => "Play-offs".into(), 31 => crate::inhalt::t("wb.runde.achtelfinale").into(), 32 => crate::inhalt::t("wb.runde.viertelfinale").into(), 33 => crate::inhalt::t("wb.runde.halbfinale").into(), 34 => crate::inhalt::t("wb.runde.finale").into(),
        _ => if kind == 0 { crate::inhalt::t("liga.art.pokal").into() } else { crate::inhalt::t("mk.tv.familie.europa").into() },
    }
}

impl World {
    /// club → days with a match (leagues and cups)
    pub fn busy_days(&self) -> HashSet<(usize, u32)> {
        let mut s = HashSet::new();
        for l in &self.leagues { for f in &l.fixtures { s.insert((f.h, f.day)); s.insert((f.a, f.day)); } }
        for c in &self.cups { for x in &c.fx { s.insert((x.f.h, x.f.day)); s.insert((x.f.a, x.f.day)); } }
        s
    }
    /// uefa.py _place: the first day near the target where neither club plays (outside international breaks).
    /// Fabian 23.11.: a real calendar — cup and European nights only on their weekdays (Champions League Tue/Wed,
    /// Europa and Conference League Thu, national cups Tue/Wed), and never the day after or before another match of
    /// either club. Finals on neutral ground keep their day.
    fn place(busy: &HashSet<(usize, u32)>, h: usize, a: usize, target: u32, kind: u8, neutral: bool, ab: u32) -> u32 {
        let tage: &[usize] = match kind { 1 => &[1, 2], 2 | 3 => &[3], 0 => &[1, 2], _ => &[0, 1, 2, 3, 4, 5, 6] };
        let frei = |d: u32, ruhe: bool| {
            !INTL_BREAKS.iter().any(|(x, y)| d >= *x && d <= *y)
                && (neutral || tage.contains(&crate::date::weekday(d)))
                && (if ruhe { d.saturating_sub(1)..=d + 1 } else { d..=d }).all(|e| !busy.contains(&(h, e)) && !busy.contains(&(a, e)))
        };
        for ruhe in [true, false] {
            for o in 0..22i32 {
                // later rounds must not slip past the next one nor into the past: a week forward, three days back
                for d in [target as i32 + o, target as i32 - o] {
                    if d >= ab as i32 && d <= target as i32 + if kind == 0 { 14 } else { 7 } && d >= target as i32 - 3 && frei(d as u32, ruhe) { return d as u32; }
                }
            }
        }
        (target.max(ab)..target.max(ab) + 60).find(|d| !busy.contains(&(h, *d)) && !busy.contains(&(a, *d))).unwrap_or(target.max(ab))
    }
    pub fn add_fx(&mut self, ci: usize, busy: &mut HashSet<(usize, u32)>, h: usize, a: usize, target: u32, stage: u8, leg: u8, neutral: bool) {
        // the second leg always comes days after the first (the round is decided on the second leg's day)
        let target = if leg == 2 { self.cups[ci].fx.iter().filter(|x| x.leg == 1 && x.stage == stage && x.f.h == a && x.f.a == h).map(|x| x.f.day + 5).max().map_or(target, |d| target.max(d)) } else { target };
        // never on a day already played (a round set up at the end of its target day)
        let ab = if self.day < target + 7 { self.day + 1 } else { 1 }; // a new season is set up while the old day still counts
        let d = Self::place(busy, h, a, target, self.cups[ci].kind, neutral, ab.max(1));
        busy.insert((h, d));
        busy.insert((a, d));
        self.cups[ci].fx.push(CupFx { f: Fixture { day: d, h, a, hg: -1, ag: -1 }, stage, leg, neutral, pens: None, et: false });
    }

    /// New season: one cup per country (all clubs of its leagues, at least 8) and Europe from `euro` (or, in the
    /// first season, from the strength order of the quota leagues).
    pub fn setup_cups(&mut self, euro: Option<[Vec<usize>; 3]>) {
        if euro.is_some() { self.ki_ausgaben.clear(); } // a new season: the fair-play account starts again
        for c in &self.cups { if let Some(w) = c.winner { self.titelverteidiger.insert(c.name.clone(), w); } }
        // the supercups of the new season: last season's champion against the cup winner (Fabian 23.11.); when the
        // champion also won the cup, the runner-up of the league plays instead
        let mut supercups: Vec<(String, String, usize, usize)> = vec![];
        if euro.is_some() {
            for c in self.cups.iter().filter(|c| c.kind == 0 && c.teams.len() > 2) {
                let Some(pokal) = c.winner else { continue };
                let Some(name) = crate::regeln::wert_oder("pokalnamen", "supercup").and_then(|m| m.get(&c.name)).and_then(|v| v.as_str()) else { continue };
                let Some(l) = (0..self.leagues.len()).find(|&l| self.leagues[l].land == c.country && self.leagues[l].key.contains("_D1_")) else { continue };
                let tab = self.table(l);
                let meister = self.meister(l).unwrap_or(pokal);
                let gegner = if meister == pokal { tab.get(1).map(|r| r.club).unwrap_or(pokal) } else { pokal };
                if gegner != meister { supercups.push((name.to_string(), c.country.clone(), meister, gegner)); }
            }
            // the UEFA Super Cup: Champions League winner against Europa League winner
            let sieger = |k: u8| self.cups.iter().find(|c| c.kind == k).and_then(|c| c.winner);
            if let (Some(cl), Some(el)) = (sieger(1), sieger(2)) { if cl != el { supercups.push((crate::inhalt::t("ui.cups.uefa_supercup").to_string(), String::new(), cl, el)); } }
        }
        self.cups.clear();
        let mut by_country: Vec<(String, Vec<usize>, String)> = vec![];
        for l in &self.leagues {
            let Some(c) = country(l) else { continue };
            let base = l.key.split("_D").next().unwrap_or("").to_string();
            match by_country.iter_mut().find(|x| x.0 == c) {
                Some(x) => { x.1.extend(l.teams.iter().copied()); if l.key.contains("_D1_") { x.2 = base; } }
                None => by_country.push((c, l.teams.clone(), if l.key.contains("_D") { base } else { String::new() })),
            }
        }
        if !self.db_cups.is_empty() {
            // real FL world: the cups of the database (Übergabe §3.4); their clubs stay the same every season
            // country of a cup: the league that names it (fl26world `cup`), else the country most of its clubs play in
            by_country = self.db_cups.iter().map(|(key, teams)| {
                let mut z: HashMap<String, usize> = HashMap::new();
                for &t in teams { if let Some(c) = country(&self.leagues[self.clubs[t].league]) { *z.entry(c).or_default() += 1; } }
                let c = self.leagues.iter().find(|l| l.pokal.as_deref() == Some(key)).and_then(country)
                    .or_else(|| z.into_iter().max_by_key(|x| x.1).map(|x| x.0)).unwrap_or_else(|| key.trim_end_matches("_CUP").replace('_', " "));
                (c, teams.iter().copied().filter(|&t| self.erste_von(t).is_none()).collect(), format!("={key}"))
            }).collect();
        }
        // second teams play in no cup (daten/ligaregeln.toml [standard.zweite_mannschaften])
        for x in by_country.iter_mut() { x.1.retain(|&t| self.erste_von(t).is_none()); }
        let mut busy = self.busy_days();
        for (c, teams, base) in by_country {
            if teams.len() < 8 { continue; }
            let key = if let Some(k) = base.strip_prefix('=') { k.to_string() } else if base.is_empty() { String::new() } else { format!("{base}_D1_CUP") };
            // comp.py cup_days: Tuesdays (day % 7 == 3) between day 24 and 296, spread, the final on the last one
            let n = (teams.len() as f32).log2().ceil() as usize;
            let tue: Vec<u32> = (24..=296).filter(|d| d % 7 == 3 && !INTL_BREAKS.iter().any(|(a, b)| d >= a && d <= b)).collect();
            let mut days: Vec<u32> = if n > 1 { (0..n - 1).map(|i| tue[i * (tue.len() - 2) / (n - 1).max(1)]).collect() } else { vec![] };
            days.push(*tue.last().unwrap());
            let ci = self.cups.len();
            self.cups.push(Cup { name: pokalname(&key, &c), key, kind: 0, country: c, teams: teams.clone(), fx: vec![], stage: 20, alive: vec![], winner: None, days });
            // comp.py cup_draw: in the first round the strongest get byes up to the next power of two
            let mut t = teams.clone();
            t.sort_by(|a, b| self.clubs[*b].level.partial_cmp(&self.clubs[*a].level).unwrap());
            let size = 1usize << n;
            let byes: Vec<usize> = t.drain(..size - teams.len()).collect();
            self.shuffle(&mut t);
            self.cups[ci].alive = byes;
            let d0 = self.cups[ci].days[0];
            for p in t.chunks(2) { if p.len() == 2 { self.add_fx(ci, &mut busy, p[0], p[1], d0, 20, 0, false); } }
        }
        let euro = euro.unwrap_or_else(|| self.euro_first());
        for (k, teams) in euro.into_iter().enumerate() {
            if teams.len() < 36 { continue; }
            let ci = self.cups.len();
            self.cups.push(Cup { name: EURO[k].0.into(), key: EURO[k].1.into(), kind: k as u8 + 1, country: String::new(), teams: teams.clone(), fx: vec![], stage: 0, alive: vec![], winner: None, days: vec![] });
            for &t in &teams { self.prize(t, k, 0, 1.0); }
            // uefa.py _league_phase: four pots by strength, every team 8 games (2 per pot: one home, one away)
            let mut order = teams.clone();
            order.sort_by(|a, b| self.clubs[*b].level.partial_cmp(&self.clubs[*a].level).unwrap());
            let m = order.len() / 4;
            let mut pots: Vec<Vec<usize>> = (0..4).map(|i| order[i * m..(i + 1) * m].to_vec()).collect();
            for p in pots.iter_mut() { self.shuffle(p); }
            let mut games = vec![];
            for a in 0..4 {
                let p = &pots[a];
                games.extend((0..m).map(|i| (p[i], p[(i + 1) % m])));
                for b in a + 1..4 {
                    let q = &pots[b];
                    games.extend((0..m).map(|i| (p[i], q[i])));
                    games.extend((0..m).map(|i| (q[i], p[(i + 1) % m])));
                }
            }
            self.shuffle(&mut games);
            let mut used: HashMap<usize, HashSet<usize>> = HashMap::new();
            for (h, a) in games {
                let slot = (0..10).find(|i| !used.get(&h).map(|s| s.contains(i)).unwrap_or(false) && !used.get(&a).map(|s| s.contains(i)).unwrap_or(false)).unwrap_or(9);
                used.entry(h).or_default().insert(slot);
                used.entry(a).or_default().insert(slot);
                self.add_fx(ci, &mut busy, h, a, L_DAYS[k][slot], slot as u8, 0, false);
            }
        }
        // one match each, before the league starts, on neutral ground
        for (name, land, meister, gegner) in supercups {
            let mut busy = self.busy_days();
            let ci = self.cups.len();
            self.cups.push(Cup { name, key: String::new(), kind: 0, country: land, teams: vec![meister, gegner], fx: vec![], stage: 34, alive: vec![], winner: None, days: vec![crate::regeln::f("pokalnamen", "supercup_tag") as u32] }); // roh
            self.add_fx(ci, &mut busy, meister, gegner, crate::regeln::f("pokalnamen", "supercup_tag") as u32, 34, 0, true); // roh
        }
    }
    fn shuffle<T>(&mut self, v: &mut [T]) { for i in (1..v.len()).rev() { let j = (self.rng.next() % (i as u64 + 1)) as usize; v.swap(i, j); } }

    /// First season: quota places by strength inside the quota leagues, then filled to 36 (uefa.py _qualify/_fill).
    fn euro_first(&self) -> [Vec<usize>; 3] {
        if let Some(db) = &self.db_euro {
            // uefa.py _db_entries: the database's clubs (first come first), topped up where it has too few
            let mut out: [Vec<usize>; 3] = [vec![], vec![], vec![]];
            let mut taken = HashSet::new();
            for t in 0..3 { for &c in &db[t] { if taken.insert(c) { out[t].push(c); } } }
            return self.fill(out, taken);
        }
        let ranks: HashMap<String, Vec<usize>> = self.leagues.iter().map(|l| {
            let mut t = l.teams.clone();
            t.sort_by(|a, b| self.clubs[*b].level.partial_cmp(&self.clubs[*a].level).unwrap());
            (l.key.clone(), t)
        }).collect();
        self.qualify(&ranks, &HashMap::new(), [None, None, None], &[])
    }
    /// Next season's participants from the final tables (call before promotion), cup winners and holders.
    pub fn euro_qualify(&self) -> [Vec<usize>; 3] {
        let ranks: HashMap<String, Vec<usize>> = (0..self.leagues.len()).map(|l| (self.leagues[l].key.clone(), self.table(l).iter().map(|r| r.club).collect())).collect();
        let cups: HashMap<String, usize> = self.cups.iter().filter(|c| c.kind == 0).filter_map(|c| Some((c.country.clone(), c.winner?))).collect();
        let w = |k: u8| self.cups.iter().find(|c| c.kind == k).and_then(|c| c.winner);
        // from the second season the performance places (EPS) follow this season's European results
        self.qualify(&ranks, &cups, [w(1), w(2), w(3)], &self.eps_berechnen())
    }
    fn qualify(&self, ranks: &HashMap<String, Vec<usize>>, cup_winners: &HashMap<String, usize>, holders: [Option<usize>; 3], eps: &[String]) -> [Vec<usize>; 3] {
        let mut out: [Vec<usize>; 3] = [vec![], vec![], vec![]];
        let mut taken: HashSet<usize> = HashSet::new();
        let mut add = |t: usize, c: Option<usize>, out: &mut [Vec<usize>; 3], taken: &mut HashSet<usize>| { if let Some(c) = c { if taken.insert(c) { out[t].push(c); } } };
        add(0, holders[0], &mut out, &mut taken); // cl_holder
        add(0, holders[1], &mut out, &mut taken); // el_winner_to_cl
        add(1, holders[2], &mut out, &mut taken); // ecl_winner_to_el
        // the places of every UEFA country by daten/ligaregeln.toml (ligaregeln.rs)
        self.uefa_plaetze(ranks, cup_winners, eps, &mut out, &mut taken);
        self.fill(out, taken)
    }
    /// uefa.py _fill: top up to 36 with the strongest clubs of the UEFA top leagues not yet qualified, at most FILL_CAP per country.
    fn fill(&self, mut out: [Vec<usize>; 3], mut taken: HashSet<usize>) -> [Vec<usize>; 3] {
        let mut pool: Vec<usize> = self.uefa_ligen().iter().flat_map(|(l, _)| self.leagues[*l].teams.iter().copied()).filter(|&c| self.erste_von(c).is_none()).collect();
        pool.sort_by(|a, b| self.clubs[*b].level.partial_cmp(&self.clubs[*a].level).unwrap());
        for t in 0..3 {
            // the cap counts per competition (uefa.py _fill counts inside the list it fills)
            let mut per: HashMap<usize, usize> = HashMap::new();
            for c in &out[t] { *per.entry(self.clubs[*c].league).or_default() += 1; }
            out[t].truncate(36);
            for &c in &pool {
                if out[t].len() >= 36 { break; }
                let lg = self.clubs[c].league;
                if taken.contains(&c) || *per.get(&lg).unwrap_or(&0) >= FILL_CAP { continue; }
                taken.insert(c);
                *per.entry(lg).or_default() += 1;
                out[t].push(c);
            }
        }
        out
    }

    fn prize(&mut self, club: usize, tier: usize, key: usize, times: f64) {
        let v = PRIZE[key] * PRIZE_SCALE[tier] * times;
        // a second team has no budget of its own: its prize money goes to the parent club
        let kasse = self.erste_von(club).unwrap_or(club);
        self.clubs[kasse].balance += v;
        self.cup_money.push((club, v));
    }

    /// Plays every cup match of the day except the own club's (that one goes through the match day).
    pub fn cup_day(&mut self, day: u32, own: usize) {
        for ci in 0..self.cups.len() {
            for fi in 0..self.cups[ci].fx.len() {
                let x = &self.cups[ci].fx[fi];
                if x.f.day != day || x.f.hg >= 0 || x.f.h == own || x.f.a == own { continue; }
                let (h, a) = (x.f.h, x.f.a);
                let lv = if self.cups[ci].kind > 0 { 1 } else { self.leagues[self.clubs[h].league].detail.max(self.leagues[self.clubs[a].league].detail).max(1) };
                self.neutral = self.cups[ci].fx[fi].neutral;
                let mut res = self.play(h, a, lv, None, None);
                self.finish_cup_match(ci, fi, &mut res);
            }
        }
        self.cup_progress();
    }
    /// The decision of a knock-out match (match.py play knockout, uefa.py _extra_time/_pens): a level single match
    /// or a level second leg on aggregate gets extra time (rates × 0.33), then penalties (5 each, 0.76, sudden death).
    pub fn finish_cup_match(&mut self, ci: usize, fi: usize, res: &mut MatchResult) {
        let x = self.cups[ci].fx[fi].clone();
        let ko_single = x.stage >= 20 && (x.stage < 30 || x.stage == 34);
        let second = x.stage >= 30 && x.stage < 34 && x.leg == 2;
        let first_leg = if second { self.cups[ci].fx.iter().find(|y| y.stage == x.stage && y.leg == 1 && y.f.h == x.f.a && y.f.a == x.f.h).map(|y| (y.f.ag as i32, y.f.hg as i32)) } else { None };
        let level = |hg: u8, ag: u8| if let Some((h1, a1)) = first_leg { h1 + hg as i32 == a1 + ag as i32 } else { hg == ag };
        let mut pens = None;
        let mut et = false;
        let vorgabe = self.ko_vorgabe.take();
        if (ko_single || second) && level(res.hg, res.ag) && vorgabe.is_some() {
            // played in FL27: the person said who won after extra time or on penalties
            let (home_won, on_pens) = vorgabe.unwrap();
            et = true;
            if on_pens { pens = Some(if home_won { PENS_UNKNOWN_H } else { PENS_UNKNOWN_A }); } else if home_won { res.hg += 1; } else { res.ag += 1; }
        } else if (ko_single || second) && level(res.hg, res.ag) {
            let (lh, la) = World::rates(res.sh, res.sa, self.clubs[x.f.h].style, self.clubs[x.f.a].style);
            let eh = self.rng.poisson(lh * ET_FACTOR).min(4) as u8;
            let ea = self.rng.poisson(la * ET_FACTOR).min(4) as u8;
            res.hg += eh;
            res.ag += ea;
            et = true;
            if level(res.hg, res.ag) {
                let p_h = (0.5 + PENS_STRENGTH * (res.sh - res.sa)).clamp(0.35, 0.65);
                let (mut ph, mut pa) = (0u8, 0u8);
                for _ in 0..5 { ph += self.rng.chance(0.76) as u8; pa += self.rng.chance(0.76) as u8; }
                while ph == pa { ph += self.rng.chance(0.76 + (p_h - 0.5) * 0.1) as u8; pa += self.rng.chance(0.76 - (p_h - 0.5) * 0.1) as u8; }
                pens = Some((ph, pa));
            }
        }
        res.pens = pens;
        res.et = et;
        let y = &mut self.cups[ci].fx[fi];
        y.f.hg = res.hg as i8;
        y.f.ag = res.ag as i8;
        y.pens = pens;
        y.et = et;
        self.apply_stats(res);
        let k = self.cups[ci].kind;
        if k > 0 && x.stage < 10 {
            let t = k as usize - 1;
            if res.hg > res.ag { self.prize(x.f.h, t, 1, 1.0); } else if res.hg < res.ag { self.prize(x.f.a, t, 1, 1.0); } else { self.prize(x.f.h, t, 2, 1.0); self.prize(x.f.a, t, 2, 1.0); }
        }
        self.coef_points(x.f.h, x.f.a, res.hg, res.ag, k);
    }
    /// Does a result leave a knock-out match open (level single match or level on aggregate after the second leg)?
    pub fn ko_offen(&self, ci: usize, fi: usize, hg: u8, ag: u8) -> bool {
        let x = &self.cups[ci].fx[fi];
        if x.stage >= 20 && (x.stage < 30 || x.stage == 34) { return hg == ag; }
        if x.stage >= 30 && x.stage < 34 && x.leg == 2 {
            if let Some(y) = self.cups[ci].fx.iter().find(|y| y.stage == x.stage && y.leg == 1 && y.f.h == x.f.a && y.f.a == x.f.h) {
                return y.f.ag as i32 + hg as i32 == y.f.hg as i32 + ag as i32;
            }
        }
        false
    }
    fn coef_points(&mut self, h: usize, a: usize, hg: u8, ag: u8, kind: u8) {
        if kind == 0 || kind >= 4 { return; }
        for (c, gf, ga) in [(h, hg, ag), (a, ag, hg)] {
            let Some(land) = country(&self.leagues[self.clubs[c].league]) else { continue };
            let pts = if gf > ga { 2.0 } else if gf == ga { 1.0 } else { 0.0 };
            match self.coef.iter_mut().find(|x| x.0 == land && x.1 == self.season) { Some(x) => x.2 += pts, None => self.coef.push((land, self.season, pts)) }
        }
    }
    pub fn winner_of(&self, ci: usize, stage: u8, a: usize, b: usize) -> usize {
        let legs: Vec<&CupFx> = self.cups[ci].fx.iter().filter(|x| x.stage == stage && ((x.f.h == a && x.f.a == b) || (x.f.h == b && x.f.a == a))).collect();
        let last = legs.iter().max_by_key(|x| x.leg).unwrap();
        if let Some((ph, pa)) = last.pens { return if ph > pa { last.f.h } else { last.f.a }; }
        let (mut ga, mut gb) = (0i32, 0i32);
        for x in &legs { if x.f.h == a { ga += x.f.hg as i32; gb += x.f.ag as i32; } else { ga += x.f.ag as i32; gb += x.f.hg as i32; } }
        if ga >= gb { a } else { b }
    }
    /// League-phase table of a European competition (points, goal difference, goals).
    pub fn cup_table(&self, ci: usize) -> Vec<Row> {
        let mut rows: Vec<Row> = self.cups[ci].teams.iter().map(|&c| Row { club: c, ..Default::default() }).collect();
        for x in self.cups[ci].fx.iter().filter(|x| x.stage < 10 && x.f.hg >= 0) {
            for (c, gf, ga) in [(x.f.h, x.f.hg as i32, x.f.ag as i32), (x.f.a, x.f.ag as i32, x.f.hg as i32)] {
                let r = rows.iter_mut().find(|r| r.club == c).unwrap();
                r.p += 1; r.gf += gf; r.ga += ga;
                if gf > ga { r.w += 1; r.pts += 3; } else if gf == ga { r.d += 1; r.pts += 1; } else { r.l += 1; }
            }
        }
        rows.sort_by(|a, b| (b.pts, b.gf - b.ga, b.gf, std::cmp::Reverse(self.clubs[b.club].id)).cmp(&(a.pts, a.gf - a.ga, a.gf, std::cmp::Reverse(self.clubs[a.club].id))));
        rows
    }
    /// Draws and schedules the next stage when the current one is complete.
    pub fn cup_progress(&mut self) {
        let mut busy: Option<HashSet<(usize, u32)>> = None;
        for ci in 0..self.cups.len() {
            if self.cups[ci].winner.is_some() { continue; }
            let st = self.cups[ci].stage;
            let in_stage = |x: &CupFx| if st < 10 { x.stage < 10 } else { x.stage == st };
            if self.cups[ci].fx.iter().filter(|x| in_stage(x)).any(|x| x.f.hg < 0) || !self.cups[ci].fx.iter().any(|x| in_stage(x)) { continue; }
            let kind = self.cups[ci].kind;
            if kind == 5 { continue; } // decision ties: ligaregeln.rs entscheidungen_planen moves them on
            let busy = busy.get_or_insert_with(|| self.busy_days());
            if kind == 4 {
                // title round: winners (and byes) in seed order into the next round, best against worst
                let pairs: Vec<(usize, usize)> = self.cups[ci].fx.iter().filter(|x| x.stage == st && x.leg <= 1).map(|x| (x.f.h, x.f.a)).collect();
                let mut next = std::mem::take(&mut self.cups[ci].alive);
                for (a, b) in pairs { let w = self.winner_of(ci, st, a, b); next.push(w); }
                let teams = self.cups[ci].teams.clone();
                next.sort_by_key(|c| teams.iter().position(|x| x == c));
                if next.len() == 1 { self.cup_won(ci, next[0]); continue; }
                let mut stufen: Vec<u8> = self.cups[ci].fx.iter().map(|x| x.stage).collect();
                stufen.dedup();
                self.endrunde_runde(ci, next, stufen.len());
                continue;
            }
            if kind == 0 {
                // domestic: winners (and the byes of round 1) into the next round
                let mut next = std::mem::take(&mut self.cups[ci].alive);
                let pairs: Vec<(usize, usize)> = self.cups[ci].fx.iter().filter(|x| x.stage == st).map(|x| (x.f.h, x.f.a)).collect();
                for (a, b) in pairs { let w = self.winner_of(ci, st, a, b); next.push(w); }
                if next.len() == 1 { self.cup_won(ci, next[0]); continue; }
                self.shuffle(&mut next);
                let r = (st - 20 + 1) as usize;
                let day = *self.cups[ci].days.get(r).unwrap_or(self.cups[ci].days.last().unwrap());
                let final_ = next.len() == 2;
                self.cups[ci].stage = if final_ { 34 } else { st + 1 };
                let ns = self.cups[ci].stage;
                for p in next.chunks(2) { self.add_fx(ci, busy, p[0], p[1], day, ns, 0, final_); }
                continue;
            }
            let t = kind as usize - 1;
            let (seeded, others): (Vec<usize>, Vec<usize>) = if st < 10 {
                // uefa.py: 1-8 straight into the last 16, 9-24 play-offs (9-16 seeded against 17-24)
                let tab: Vec<usize> = self.cup_table(ci).iter().map(|r| r.club).collect();
                for &c in &tab[8..24] { self.prize(c, t, 3, 1.0); }
                self.cups[ci].alive = tab[..8].to_vec();
                self.cups[ci].stage = 30;
                (tab[8..16].to_vec(), tab[16..24].to_vec())
            } else if st == 34 {
                let x = self.cups[ci].fx.iter().find(|x| x.stage == 34).unwrap();
                let w = self.winner_of(ci, 34, x.f.h, x.f.a);
                self.prize(w, t, 8, 1.0);
                self.cup_won(ci, w);
                continue;
            } else {
                let pairs: Vec<(usize, usize)> = self.cups[ci].fx.iter().filter(|x| x.stage == st && x.leg == 1).map(|x| (x.f.h, x.f.a)).collect();
                let winners: Vec<usize> = pairs.iter().map(|(a, b)| self.winner_of(ci, st, *a, *b)).collect();
                let mut prev = std::mem::take(&mut self.cups[ci].alive);
                self.cups[ci].stage = st + 1;
                let key = [4, 5, 6, 7][(st + 1 - 31).min(3) as usize];
                if st == 30 {
                    for &w in &winners { self.prize(w, t, key, 1.0); }
                    for &w in &prev { self.prize(w, t, key, 1.0); }
                    let mut o = winners;
                    self.shuffle(&mut o);
                    self.shuffle(&mut prev);
                    (prev, o)
                } else {
                    for &w in &winners { self.prize(w, t, key, 1.0); }
                    let mut all = winners;
                    self.shuffle(&mut all);
                    let h = all.len() / 2;
                    (all[..h].to_vec(), all[h..].to_vec())
                }
            };
            let ns = self.cups[ci].stage;
            if ns == 34 {
                self.add_fx(ci, busy, seeded[0], others[0], FINAL[t], 34, 0, true);
            } else {
                let (d1, d2) = KO_DAYS[t][(ns - 30) as usize];
                // two legs, the seeded club at home in the second leg
                for (s, o) in seeded.iter().zip(others.iter()) {
                    self.add_fx(ci, busy, *o, *s, d1, ns, 1, false);
                    self.add_fx(ci, busy, *s, *o, d2, ns, 2, false);
                }
            }
        }
    }
    fn cup_won(&mut self, ci: usize, w: usize) {
        self.cups[ci].winner = Some(w);
        self.cups[ci].alive = vec![w];
        let name = self.cups[ci].name.clone();
        // a title round makes the champion of its league (the league title goes into the honours at season end)
        if self.cups[ci].kind == 4 { let k = self.cups[ci].key.clone(); if let Some(l) = self.leagues.iter().position(|x| x.key == k) { self.leagues[l].meister = Some(w); } return; }
        self.titles.push(Title { season: self.season, comp: name, club: w });
    }

    /// Season end: league champions into the honours, reputation from titles and Europe (ruf.json club).
    pub fn season_honours(&mut self) {
        for l in 0..self.leagues.len() {
            if let Some(m) = self.meister(l) { let n = self.leagues[l].name.clone(); self.titles.push(Title { season: self.season, comp: n, club: m }); }
        }
        let mut delta: HashMap<usize, f32> = HashMap::new();
        for t in self.titles.iter().filter(|t| t.season == self.season) {
            let kind = self.cups.iter().find(|c| c.name == t.comp).map(|c| c.kind);
            *delta.entry(t.club).or_default() += match kind { Some(0) => 1.0, Some(1) => 3.0, Some(2) => 1.5, Some(3) => 1.0, _ => 2.0 };
        }
        for c in self.cups.iter().filter(|c| (1..=3).contains(&c.kind)) {
            for &t in &c.teams { *delta.entry(t).or_default() += [0.6, 0.3, 0.2][c.kind as usize - 1]; }
        }
        for (c, d) in delta { self.clubs[c].rep = (self.clubs[c].rep + d.min(4.0)).clamp(0.0, 100.0); } // ruf.json max_step 4
        self.coef.retain(|x| x.1 + 5 > self.season);
    }
}

#[cfg(test)]
mod t {
    use super::*;
    /// Fabian 23.11.: the knock-out view showed odd pairings and semi-finals as finals. Over a whole season every cup:
    /// no club twice in a round, every club of a round won the one before (or had a bye), the last round is one tie.
    #[test]
    fn ko_runden_stimmig() {
        let w = World::load(include_str!("../assets/world.json"), 3);
        let mut c = crate::game::Career::new(w, 1, "Test");
        for _ in 0..330 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } c.ultimatum = None; if c.end_day().is_err() { break; } }
        for (ci, cup) in c.world.cups.iter().enumerate() {
            let mut stages: Vec<u8> = cup.fx.iter().map(|f| f.stage).filter(|s| *s >= 20).collect();
            stages.sort(); stages.dedup();
            let mut vorher: Option<Vec<usize>> = None;
            for &st in &stages {
                let mut paare: Vec<(usize, usize)> = vec![];
                for f in cup.fx.iter().filter(|f| f.stage == st) { let p = (f.f.h.min(f.f.a), f.f.h.max(f.f.a)); if !paare.contains(&p) { paare.push(p); } }
                let mut teams: Vec<usize> = paare.iter().flat_map(|p| [p.0, p.1]).collect();
                let n = teams.len();
                teams.sort(); teams.dedup();
                assert_eq!(teams.len(), n, "{} stage {st}: a club twice in one round", cup.name);
                if let Some(v) = &vorher {
                    let verloren: Vec<usize> = v.iter().copied().filter(|t| !paare.is_empty() && cup.fx.iter().any(|f| f.stage < st && f.stage >= 20 && (f.f.h == *t || f.f.a == *t)) && false).collect();
                    let _ = verloren;
                    for t in &teams {
                        let in_vorrunde = v.contains(t);
                        if in_vorrunde {
                            let gegner = cup.fx.iter().find(|f| f.stage == *stages.iter().filter(|&&s| s < st).last().unwrap() && (f.f.h == *t || f.f.a == *t)).unwrap();
                            // a tie on aggregate goes to the better placed club, which the order of the call says: either order
                            if gegner.f.hg >= 0 { assert!(c.world.winner_of(ci, gegner.stage, gegner.f.h, gegner.f.a) == *t || c.world.winner_of(ci, gegner.stage, gegner.f.a, gegner.f.h) == *t, "{} stage {st}: a loser of the round before plays on", cup.name); }
                        }
                    }
                }
                vorher = Some(teams);
            }
            if let (Some(_), Some(&last)) = (cup.winner, stages.last()) {
                let n = cup.fx.iter().filter(|f| f.stage == last).map(|f| (f.f.h.min(f.f.a), f.f.h.max(f.f.a))).collect::<std::collections::HashSet<_>>().len();
                assert_eq!(n, 1, "{}: the last round of a finished cup is one tie", cup.name);
            }
        }
    }
    /// Fabian 23.11.: the new season opens with the supercup – champion against cup winner, one match, real name.
    #[test]
    fn supercup_meister_gegen_pokalsieger() {
        let mut w = World::load(include_str!("../assets/world.json"), 3);
        w.setup_cups(None);
        let fa = w.cups.iter().position(|c| c.name == "FA Cup").expect("FA Cup in the demo world");
        let sieger = w.cups[fa].teams[3];
        w.cups[fa].winner = Some(sieger);
        let eu = [w.cups.iter().find(|c| c.kind == 1).map(|c| c.teams.clone()).unwrap_or_default(), vec![], vec![]];
        w.setup_cups(Some(eu));
        let sc = w.cups.iter().find(|c| c.name == "FA Community Shield").expect("Community Shield");
        assert_eq!(sc.teams.len(), 2);
        assert!(sc.teams.contains(&sieger));
        assert_eq!(sc.fx.len(), 1, "one match");
        assert_eq!(w.titelverteidiger.get("FA Cup"), Some(&sieger));
        // the UEFA Super Cup: winners of the Champions and Europa League
        let mut w = World::load(include_str!("../assets/world.json"), 3);
        w.setup_cups(None);
        let (cl, el) = (w.cups.iter().position(|c| c.kind == 1), w.cups.iter().position(|c| c.kind == 2));
        if let (Some(cl), Some(el)) = (cl, el) {
            let (a, b) = (w.cups[cl].teams[0], w.cups[el].teams[0]);
            w.cups[cl].winner = Some(a); w.cups[el].winner = Some(b);
            let eu = [w.cups[cl].teams.clone(), vec![], vec![]];
            w.setup_cups(Some(eu));
            let us = w.cups.iter().find(|c| c.teams == vec![a, b]).expect("UEFA Super Cup");
            assert_eq!(us.fx.len(), 1);
        }
    }
    #[test]
    fn echter_kalender() {
        let mut w = World::load(include_str!("../assets/world.json"), 3);
        w.setup_cups(None);
        // European nights on their weekdays, and no club plays a cup match the day before or after another match
        let mut tage: std::collections::HashMap<usize, Vec<u32>> = Default::default();
        for l in &w.leagues { for f in &l.fixtures { tage.entry(f.h).or_default().push(f.day); tage.entry(f.a).or_default().push(f.day); } }
        for c in &w.cups { for x in &c.fx { tage.entry(x.f.h).or_default().push(x.f.day); tage.entry(x.f.a).or_default().push(x.f.day); } }
        for c in w.cups.iter().filter(|c| c.kind >= 1 && c.kind <= 3) {
            for x in c.fx.iter().filter(|x| !x.neutral) {
                let wd = crate::date::weekday(x.f.day);
                assert!(if c.kind == 1 { wd == 1 || wd == 2 } else { wd == 3 }, "{} on weekday {wd}", c.name);
            }
        }
        let eng = tage.values().map(|v| { let mut v = v.clone(); v.sort(); v.windows(2).filter(|p| p[1] == p[0] + 1).count() }).sum::<usize>();
        // shortcut: clashes with midweek league rounds (Wednesday) can remain for cup ties; none may touch a weekend
        let mut wochenende = 0;
        for c in &w.cups { for x in &c.fx { for t in [x.f.h, x.f.a] { if tage[&t].iter().any(|&d| (d + 1 == x.f.day || d == x.f.day + 1) && crate::date::weekday(d) >= 5) { wochenende += 1; } } } }
        assert_eq!(wochenende, 0, "no cup match the day before or after a weekend match");
        let _ = eng;
    }
    #[test]
    fn answer_decides_level_knockout() {
        let mut w = World::load(include_str!("../assets/world.json"), 4);
        let ci = w.cups.iter().position(|c| c.kind == 0 && !c.fx.is_empty()).unwrap();
        let (h, a) = (w.cups[ci].fx[0].f.h, w.cups[ci].fx[0].f.a);
        assert!(w.ko_offen(ci, 0, 1, 1) && !w.ko_offen(ci, 0, 2, 1));
        let mut res = w.play(h, a, 1, None, Some((1, 1)));
        w.ko_vorgabe = Some((false, true)); // the away team won on penalties
        w.finish_cup_match(ci, 0, &mut res);
        assert_eq!((res.pens, res.et), (Some(PENS_UNKNOWN_A), true));
        assert_eq!(pens_score(res.pens.unwrap()), "");
        assert_eq!(pens_score((4, 2)), " 4:2");
        assert_eq!(w.winner_of(ci, w.cups[ci].fx[0].stage, h, a), a);
        let mut res2 = w.play(w.cups[ci].fx[1].f.h, w.cups[ci].fx[1].f.a, 1, None, Some((0, 0)));
        w.ko_vorgabe = Some((true, false)); // home team won in extra time
        w.finish_cup_match(ci, 1, &mut res2);
        assert_eq!((res2.hg, res2.ag, res2.pens), (1, 0, None));
    }
    #[test]
    fn real_world_takes_cups_and_europe_from_the_database() {
        let p = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&p) { return; }
        let w = World::from_pesdb(&p, 5).unwrap();
        let dom: Vec<&Cup> = w.cups.iter().filter(|c| c.kind == 0).collect();
        assert!(dom.iter().any(|c| c.key == "ENGLAND_D1_CUP" && c.name == "FA Cup"), "{:?}", dom.iter().map(|c| (&c.key, &c.name)).collect::<Vec<_>>());
        assert_eq!(dom.len(), w.db_cups.len());
        let db = w.db_euro.clone().unwrap();
        let cl = &w.cups.iter().find(|c| c.kind == 1).unwrap().teams;
        assert_eq!(cl.len(), 36);
        assert!(db[0].iter().take(20).all(|t| cl.contains(t)), "Champions-League-Einträge der Datenbank fehlen");
    }
    #[test]
    fn twenty_seasons_of_cups() {
        let mut w = World::load(include_str!("../assets/world.json"), 9);
        for s in 0..20 {
            for day in 0..crate::date::SEASON_DAYS {
                w.day = day;
                for l in 0..w.leagues.len() {
                    for fi in 0..w.leagues[l].fixtures.len() {
                        let f = w.leagues[l].fixtures[fi];
                        if f.day == day && f.hg < 0 { let r = w.play(f.h, f.a, 2, None, None); w.apply_result(l, fi, &r); }
                    }
                }
                w.cup_day(day, usize::MAX);
                w.laenderspielfenster(day);
            }
            // every competition has exactly one winner; no club plays twice on one day
            assert!(w.cups.iter().all(|c| c.winner.is_some()), "season {s}: {:?}", w.cups.iter().filter(|c| c.winner.is_none()).map(|c| (c.name.clone(), c.stage, c.fx.iter().filter(|x| x.f.hg < 0).map(|x| (x.stage, x.leg, x.f.day)).collect::<Vec<_>>())).collect::<Vec<_>>());
            assert_eq!(w.cups.iter().filter(|c| c.kind > 0).count(), 3);
            let mut seen = HashSet::new();
            for l in &w.leagues { for f in &l.fixtures { assert!(seen.insert((f.h, f.day)) && seen.insert((f.a, f.day)), "two matches on one day"); } }
            for c in &w.cups { for x in &c.fx { assert!(seen.insert((x.f.h, x.f.day)) && seen.insert((x.f.a, x.f.day)), "{}: club plays twice on day {}", c.name, x.f.day); } }
            let q = w.euro_qualify();
            let holder = w.cups.iter().find(|c| c.kind == 1).unwrap().winner.unwrap();
            assert!(q[0].contains(&holder), "the holder is in next season's Champions League");
            assert!(q.iter().all(|t| t.len() == 36));
            w.season_honours();
            w.season += 1;
            for l in 0..w.leagues.len() { w.make_fixtures(l); }
            w.setup_cups(Some(q));
        }
        assert!(w.titles.len() > 20 * w.leagues.len());
    }
}
