//! Nationalteams (konzepte/nationalteams.md, 23.11.2026), the core: every nation with enough players has an A-team and a
//! coach with a philosophy (assets/regeln/nationaltrainer.toml, real coaches, else neutral values). On the first day of
//! each international break the coach nominates (Teil 3.2: strength, form, playing time, loyalty, youth, league, each
//! weighted by his philosophy), the nations play two matches, caps and goals count, travel costs fitness, and the own
//! club feels it: first call-ups, dropped regulars, injuries.
use crate::world::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Nationalteam {
    pub nat: u16,
    /// The current squad (player indices) and the earlier ones, newest last (at most 6).
    pub kader: Vec<usize>,
    pub frueher: Vec<Vec<usize>>,
    pub spiele: u16,
    pub siege: u16,
    pub remis: u16,
    /// The U21 squad of the last international break (package rest, 23.11.): the best young ones not in the A-team.
    #[serde(default)]
    pub u21: Vec<usize>,
}

/// A coach's philosophy (0–1 each, Teil 2.2) and his name.
#[derive(Clone, Debug)]
pub struct Philosophie { pub name: String, pub treue: f32, pub form: f32, pub spielzeit: f32, pub jugend: f32, pub liga: f32, pub disziplin: f32 }

/// The A-team coach of a nation from nationaltrainer.toml (matched by the country name in nationen.json), else neutral.
pub fn trainer(w: &World, nat: u16) -> Philosophie {
    let name = crate::nationen::name(w, nat);
    let t = crate::regeln::wert("nationaltrainer", crate::id("trainer"));
    let codes = crate::regeln::wert("nationaltrainer", crate::id("codes"));
    let code = codes.as_object().and_then(|m| m.iter().find(|(_, v)| v.as_str() == Some(name.as_str())).map(|(k, _)| k.clone()));
    let e = code.and_then(|c| t.as_array()?.iter().find(|x| x["code"].as_str() == Some(c.as_str()) && x["team"].as_str() == Some(crate::id("A"))).cloned());
    let r = |k: &str| e.as_ref().and_then(|x| x[k].as_f64()).unwrap_or(50.0) as f32 / 100.0;
    Philosophie {
        name: e.as_ref().and_then(|x| x["name"].as_str().map(str::to_string)).unwrap_or_default(),
        treue: r(crate::id("treue")), form: r("form_blick"), spielzeit: r("spielzeit_pflicht"), jugend: r(crate::id("jugendmut")), liga: r("liga_anspruch"), disziplin: r(crate::id("disziplin")),
    }
}

fn k(s: &str) -> f32 { crate::regeln::f("nationaltrainer", &format!("pes_m.{s}")) }

/// The confederation of a nationality. shortcut: by the PES number ranges (Europe 190–310 is confirmed, the others are
/// estimated from the known numbers); replace with nationen.toml once it exists.
pub fn verband(n: u16) -> &'static str {
    match n { 190..=310 => "UEFA", 141..=189 => "CONMEBOL", 100..=140 => "CONCACAF", 40..=99 => "CAF", 311.. => "OFC", _ => crate::id("AFC") }
}
/// What a tournament left for one player of the own club: player, nation, stage reached (0 group … 5 won), goals.
pub type TurnierSpur = (usize, u16, u8, u16);
/// A Nations League group: its league (0 A … 3 D), the nations and the matches (goals −1 until played).
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct NlGruppe { pub liga: u8, pub nats: Vec<u16>, pub spiele: Vec<NlSpiel> }
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct NlSpiel { pub runde: u8, pub h: u16, pub a: u16, pub hg: i16, pub ag: i16 }
/// A finished tournament for the history: name, year, winner, runner-up.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Turnier { pub name: String, pub jahr: i32, pub sieger: u16, pub zweiter: u16 }

impl World {
    /// FIFA-style points of a nation (Elo, Teil 4.4): from the team's strength when it has none yet.
    pub fn fifa_punkte(&self, n: u16) -> f32 {
        if let Some(&p) = self.nat_punkte.get(&n) { return p; }
        1000.0 + (self.nat_staerke(n) - 65.0) * 25.0
    }
    /// A nation's strength: mean overall of the best eleven of its squad (or its pool).
    pub fn nat_staerke(&self, n: u16) -> f32 {
        let mut v: Vec<f32> = match self.nationalteams.iter().find(|t| t.nat == n) { Some(t) if !t.kader.is_empty() => t.kader.iter().map(|&p| self.players[p].ovr).collect(), _ => self.players.iter().filter(|p| p.nat == n && p.team >= 0).map(|p| p.ovr).collect() };
        v.sort_by(|a, b| b.partial_cmp(a).unwrap());
        v.truncate(11);
        if v.is_empty() { 50.0 } else { v.iter().sum::<f32>() / v.len() as f32 }
    }
    /// One international match: the score, the ranking update (Elo with weight `wichtig`), caps, goals, fatigue.
    fn laenderspiel(&mut self, a: u16, b: u16, wichtig: f32, spur: &mut Vec<(usize, u16)>) -> (u16, u16) {
        let (sa, sb) = (self.nat_staerke(a), self.nat_staerke(b));
        let (lh, la) = World::rates(sa, sb, 0, 0);
        let (ga, gb) = (self.rng.poisson(lh * 0.9).min(7) as u16, self.rng.poisson(la * 0.9).min(7) as u16);
        let (pa, pb) = (self.fifa_punkte(a), self.fifa_punkte(b));
        let erw = 1.0 / (10f32.powf(-(pa - pb) / 600.0) + 1.0);
        let w = if ga > gb { 1.0 } else if ga == gb { 0.5 } else { 0.0 };
        self.nat_punkte.insert(a, pa + wichtig * (w - erw));
        self.nat_punkte.insert(b, pb + wichtig * ((1.0 - w) - (1.0 - erw)));
        for (n, tore) in [(a, ga), (b, gb)] {
            let Some(ti) = self.nationalteams.iter().position(|t| t.nat == n) else { continue };
            let mut kader = self.nationalteams[ti].kader.clone();
            kader.sort_by(|x, y| self.players[*y].ovr.partial_cmp(&self.players[*x].ovr).unwrap());
            let spieler: Vec<usize> = kader.into_iter().take(14).collect();
            for &p in &spieler { let pl = &mut self.players[p]; pl.profil.laenderspiele += 1; pl.fit = (pl.fit - k("fitness_je_spiel")).max(20.0); }
            let wg: Vec<f32> = spieler.iter().map(|&p| match part_of(&self.players[p].pos) { 0 => 0.0, 1 => 0.4, 2 => 1.2, _ => 3.0 }).collect();
            for _ in 0..tore { if wg.iter().sum::<f32>() <= 0.0 { break; } let p = spieler[self.rng.weighted(&wg)]; self.players[p].profil.nat_tore += 1; spur.push((p, 1)); }
        }
        (ga, gb)
    }
    /// The big tournaments (Teil 4.2) on their day: WM, EM, Copa, Afrika-, Asien-, Gold Cup, OFC (nationalwettbewerbe.toml,
    /// format gruppen_ko, by rhythm). Qualification by ranking within the confederation, groups of four seeded by pots,
    /// the best go to the knock-out rounds. Returns the tournaments played today with the own club's traces.
    /// shortcut: qualification by ranking instead of qualifying groups, U21/U23/Olympia not yet; build the qualifiers after.
    pub fn turniere_tag(&mut self) -> Vec<(String, Vec<TurnierSpur>, u16)> {
        let (jahr, monat, tag) = crate::date::ymd(self.season, self.day);
        let liste = crate::regeln::wert("nationalwettbewerbe", crate::id("wettbewerb")).as_array().cloned().unwrap_or_default();
        let mut out = vec![];
        for w in liste {
            if w["format"].as_str() != Some("gruppen_ko") { continue; }
            let kz = w["kuerzel"].as_str().unwrap_or("");
            if kz.starts_with(crate::id("u2")) || kz == "olympia" { continue; }
            let (erst, rh) = (w["erstes_jahr"].as_i64().unwrap_or(0) as i32, w["rhythmus"].as_i64().unwrap_or(4).max(1) as i32);
            if jahr < erst || (jahr - erst) % rh != 0 { continue; }
            let m0: u32 = w["monate"].as_array().and_then(|a| a.first()).and_then(|x| x.as_str()).and_then(|x| x.parse().ok()).unwrap_or(6);
            if monat != m0 || tag != k("turnier_starttag") as u32 { continue; }
            let vb = w["verband"].as_str().unwrap_or(crate::id("FIFA"));
            let mut teiln: Vec<u16> = self.nationalteams.iter().map(|t| t.nat).filter(|&n| vb == "FIFA" || verband(n) == vb).collect();
            teiln.sort_by(|a, b| self.fifa_punkte(*b).partial_cmp(&self.fifa_punkte(*a)).unwrap());
            let soll = w["teams"].as_u64().unwrap_or(16) as usize;
            teiln.truncate(soll.min(teiln.len() / 4 * 4));
            if teiln.len() < 4 { continue; }
            let name = w["name"].as_str().unwrap_or(kz).to_string();
            let (sieger, zweiter, spur) = self.turnier_spielen(&teiln);
            let mut eigen: Vec<TurnierSpur> = vec![];
            for (p, n, stufe) in spur.iter().copied() {
                let tore = self.turnier_tore.get(&p).copied().unwrap_or(0);
                eigen.push((p, n, stufe, tore));
            }
            self.turnier_tore.clear();
            self.turniere.push(Turnier { name: name.clone(), jahr, sieger, zweiter });
            out.push((name, eigen, sieger));
        }
        out
    }
    /// Plays one tournament: groups of four (pots by ranking), top two and the best thirds into the knock-out rounds.
    fn turnier_spielen(&mut self, teiln: &[u16]) -> (u16, u16, Vec<(usize, u16, u8)>) {
        let g = teiln.len() / 4;
        let mut gruppen: Vec<Vec<u16>> = vec![vec![]; g];
        for (i, &n) in teiln.iter().enumerate() { let topf = i / g; let j = if topf % 2 == 0 { i % g } else { g - 1 - i % g }; gruppen[j].push(n); }
        let mut stufe: std::collections::HashMap<u16, u8> = teiln.iter().map(|&n| (n, 0u8)).collect();
        let mut spur: Vec<(usize, u16)> = vec![];
        let mut dritte: Vec<(u16, i32, i32)> = vec![];
        let mut weiter: Vec<u16> = vec![];
        for gr in &gruppen {
            let mut tab: Vec<(u16, i32, i32)> = gr.iter().map(|&n| (n, 0, 0)).collect();
            for i in 0..gr.len() { for j in i + 1..gr.len() {
                let (a, b) = self.laenderspiel(gr[i], gr[j], k("elo_turnier"), &mut spur);
                tab[i].2 += a as i32 - b as i32; tab[j].2 += b as i32 - a as i32;
                if a > b { tab[i].1 += 3; } else if a < b { tab[j].1 += 3; } else { tab[i].1 += 1; tab[j].1 += 1; }
            } }
            tab.sort_by(|x, y| (y.1, y.2).cmp(&(x.1, x.2)));
            weiter.extend(tab.iter().take(2).map(|x| x.0));
            if let Some(&d) = tab.get(2) { dritte.push(d); }
        }
        let mut ko = 1;
        while ko * 2 <= weiter.len() + dritte.len() && ko * 2 <= 32 { ko *= 2; }
        dritte.sort_by(|x, y| (y.1, y.2).cmp(&(x.1, x.2)));
        let fehlen = ko.saturating_sub(weiter.len());
        weiter.extend(dritte.iter().take(fehlen).map(|x| x.0));
        weiter.truncate(ko);
        weiter.sort_by(|a, b| self.fifa_punkte(*b).partial_cmp(&self.fifa_punkte(*a)).unwrap());
        let mut runde = 1u8;
        let mut zweiter = 0;
        while weiter.len() > 1 {
            for &n in &weiter { stufe.insert(n, runde); }
            let mut naechste = vec![];
            for i in 0..weiter.len() / 2 {
                let (a, b) = (weiter[i], weiter[weiter.len() - 1 - i]);
                let (x, y) = self.laenderspiel(a, b, k("elo_turnier"), &mut spur);
                let a_weiter = if x != y { x > y } else { self.rng.chance(0.5 + (self.nat_staerke(a) - self.nat_staerke(b)) * 0.01) };
                if weiter.len() == 2 { zweiter = if a_weiter { b } else { a }; }
                naechste.push(if a_weiter { a } else { b });
            }
            weiter = naechste;
            runde += 1;
        }
        let sieger = weiter[0];
        stufe.insert(sieger, 5);
        if zweiter > 0 { stufe.insert(zweiter, 4); }
        // the tournament boost (Teil 5.5): market value by how far the nation got, everyone becomes known
        let schub = crate::regeln::wert("nationaltrainer", "pes_m.turnier_schub").as_array().cloned().unwrap_or_default();
        let mut spieler: Vec<(usize, u16, u8)> = vec![];
        for (&n, &st) in &stufe {
            let kader = self.nationalteams.iter().find(|t| t.nat == n).map(|t| t.kader.clone()).unwrap_or_default();
            let f = schub.get(st.min(5) as usize).and_then(|x| x.as_f64()).unwrap_or(1.0);
            for p in kader { self.players[p].value *= f; self.players[p].know = self.players[p].know.max(k("wissen_nach_turnier") / 100.0); spieler.push((p, n, st)); }
        }
        for (p, _) in spur { *self.turnier_tore.entry(p).or_default() += 1; }
        (sieger, zweiter, spieler)
    }
    /// The players of each nationality that could be called up: club players, not youth, not injured beyond the break.
    fn pools(&self) -> std::collections::HashMap<u16, Vec<usize>> {
        let mut m: std::collections::HashMap<u16, Vec<usize>> = Default::default();
        for (p, pl) in self.players.iter().enumerate() {
            if pl.nat == 0 || pl.team < 0 || pl.youth != 0 || pl.inj as f32 > k("verletzt_bis") { continue; }
            m.entry(pl.nat).or_default().push(p);
        }
        m
    }
    /// Teil 3.2: the nomination value 0–100 of a player for his nation's coach.
    pub fn nominierungswert(&self, p: usize, pool: &[usize], ph: &Philosophie, team: Option<&Nationalteam>) -> f32 {
        let pl = &self.players[p];
        let rang = pool.iter().filter(|&&q| self.players[q].ovr > pl.ovr).count() as f32;
        let staerke = 100.0 * (1.0 - rang / pool.len().max(1) as f32);
        let form = if pl.last.is_empty() { 50.0 } else { ((pl.last.iter().sum::<f32>() / pl.last.len() as f32 - 6.0) / 2.0 * 100.0).clamp(0.0, 100.0) };
        let spiele = self.leagues.get(self.clubs[pl.team as usize].league).map_or(0, |l| l.fixtures.iter().filter(|f| f.hg >= 0 && (f.h == pl.team as usize || f.a == pl.team as usize)).count());
        let spielzeit = if spiele < 3 { 50.0 } else { (pl.apps as f32 / spiele as f32 * 100.0).min(100.0) };
        let treue = team.map_or(0.0, |t| {
            let zuletzt = if t.kader.contains(&p) { 40.0 } else { 0.0 };
            let kern = if t.frueher.iter().rev().take(6).filter(|k| k.contains(&p)).count() >= 4 { 40.0 } else { 0.0 };
            zuletzt + kern
        }) + (pl.profil.laenderspiele as f32 / 3.0).min(20.0);
        let jugend = if pl.age <= 23 && pl.pot - pl.ovr >= 6.0 { ((pl.pot - 70.0) * 4.0).clamp(0.0, 100.0) } else { 0.0 };
        let liga = { let l = self.clubs[pl.team as usize].league; self.leagues.get(l).map_or(30.0, |lg| (100.0 - 25.0 * (lg.stufe.max(1) - 1) as f32).max(20.0)) };
        let g = [0.40, 0.05 + 0.20 * ph.form, 0.03 + 0.17 * ph.spielzeit, 0.05 + 0.25 * ph.treue, 0.12 * ph.jugend, 0.10 * ph.liga];
        let n = g[0] * staerke + g[1] * form + g[2] * spielzeit + g[3] * treue.min(100.0) + g[4] * jugend + g[5] * liga;
        let zufall = (crate::mensch::zufall(&[crate::id("nominierung"), &pl.id.to_string(), &self.season.to_string(), &self.day.to_string()]) - 0.5) * 6.0;
        (n / g.iter().sum::<f32>() + zufall).clamp(0.0, 100.0)
    }
    /// Teil 3.3: the squad of a nation: 3 keepers, then per part the highest values (defence 8, midfield 7, attack 5).
    fn kader_waehlen(&self, pool: &[usize], ph: &Philosophie, team: Option<&Nationalteam>) -> Vec<usize> {
        let mut wert: Vec<(usize, f32)> = pool.iter().map(|&p| (p, self.nominierungswert(p, pool, ph, team))).collect();
        wert.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let mut kader = vec![];
        for (teil, n) in [(0usize, 3usize), (1, 8), (2, 7), (3, 5)] {
            kader.extend(wert.iter().filter(|x| part_of(&self.players[x.0].pos) == teil).take(n).map(|x| x.0));
        }
        kader
    }
    /// The first day of an international break: every nation with enough players nominates and plays two matches.
    /// Returns per own-club-relevant player what happened: (player, first call-up, called up, dropped, injured days, goals).
    pub fn laenderspielfenster(&mut self, day: u32) -> Vec<(usize, bool, bool, bool, u16, u16)> {
        if !INTL_BREAKS.iter().any(|(a, _)| *a == day) { return vec![]; }
        let pools = self.pools();
        let mut nats: Vec<u16> = pools.iter().filter(|(_, v)| v.len() >= k("pool_mindestens") as usize).map(|(n, _)| *n).collect();
        nats.sort();
        let mut out: Vec<(usize, bool, bool, bool, u16, u16)> = vec![];
        let mut staerke: Vec<(u16, f32)> = vec![];
        for &n in &nats {
            let ph = trainer(self, n);
            let alt = self.nationalteams.iter().find(|t| t.nat == n).cloned();
            let kader = self.kader_waehlen(&pools[&n], &ph, alt.as_ref());
            let vorher: Vec<usize> = alt.as_ref().map(|t| t.kader.clone()).unwrap_or_default();
            for &p in &kader { out.push((p, self.players[p].profil.laenderspiele == 0, true, false, 0, 0)); }
            for &p in vorher.iter().filter(|p| !kader.contains(p)) { out.push((p, false, false, true, 0, 0)); }
            let mut elf: Vec<f32> = kader.iter().map(|&p| self.players[p].ovr).collect();
            elf.sort_by(|a, b| b.partial_cmp(a).unwrap());
            staerke.push((n, elf.iter().take(11).sum::<f32>() / 11.0f32.min(elf.len().max(1) as f32)));
            // the U21: age up to 21, not in the A squad, by strength and potential
            let mut jung: Vec<usize> = pools[&n].iter().copied().filter(|&p| self.players[p].age <= k("u21_alter") as u8 && !kader.contains(&p)).collect();
            jung.sort_by(|a, b| (self.players[*b].ovr + self.players[*b].pot * 0.5).partial_cmp(&(self.players[*a].ovr + self.players[*a].pot * 0.5)).unwrap());
            jung.truncate(k("u21_kader") as usize);
            for &p in &jung { let pl = &mut self.players[p]; pl.fit = (pl.fit - k("fitness_je_spiel")).max(20.0); }
            match self.nationalteams.iter_mut().find(|t| t.nat == n) {
                Some(t) => { let alt = std::mem::replace(&mut t.kader, kader); t.frueher.push(alt); if t.frueher.len() > 6 { t.frueher.remove(0); } t.u21 = jung; }
                None => self.nationalteams.push(Nationalteam { nat: n, kader, u21: jung, ..Default::default() }),
            }
        }
        // the Nations League in the autumn windows (Fabian 24.11., Teil 4): UEFA nations play their group's matchdays
        let fenster = INTL_BREAKS.iter().position(|(a, _)| *a == day).unwrap_or(9);
        let st: std::collections::HashMap<u16, f32> = staerke.iter().copied().collect();
        if fenster == 0 && (self.nl_saison != self.season + 1 || self.nl.is_empty()) { self.nl_auslosen(&nats); }
        let mut in_nl: Vec<u16> = vec![];
        if fenster < 3 {
            for runde in [fenster * 2, fenster * 2 + 1] {
                for g in 0..self.nl.len() {
                    for s_i in 0..self.nl[g].spiele.len() {
                        let sp = self.nl[g].spiele[s_i].clone();
                        if sp.runde as usize != runde || sp.hg >= 0 { continue; }
                        let (Some(&sa), Some(&sb)) = (st.get(&sp.h), st.get(&sp.a)) else { continue };
                        let (ga, gb) = self.laenderspiel_spielen(sp.h, sa, sp.a, sb, &mut out);
                        let x = &mut self.nl[g].spiele[s_i];
                        (x.hg, x.ag) = (ga as i16, gb as i16);
                        in_nl.extend([sp.h, sp.a]);
                    }
                }
            }
        }
        // the others: two matches against a nation of similar strength (the neighbour in the strength order)
        staerke.retain(|x| !in_nl.contains(&x.0));
        staerke.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for _runde in 0..2usize {
            let mut i = 0;
            while i + 1 < staerke.len() {
                let ((na, sa), (nb, sb)) = (staerke[i], staerke[i + 1]);
                self.laenderspiel_spielen(na, sa, nb, sb, &mut out);
                i += 2;
            }
        }
        out
    }
    /// One international between `na` and `nb` (strengths `sa`, `sb`): goals, ranking points, caps, scorers, injuries.
    fn laenderspiel_spielen(&mut self, na: u16, sa: f32, nb: u16, sb: f32, out: &mut Vec<(usize, bool, bool, bool, u16, u16)>) -> (u16, u16) {
        let (lh, la) = World::rates(sa, sb, 0, 0);
        let (ga, gb) = (self.rng.poisson(lh * 0.9).min(7) as u16, self.rng.poisson(la * 0.9).min(7) as u16);
        // the ranking moves with every match (friendly weight)
        let (pa, pb) = (self.fifa_punkte(na), self.fifa_punkte(nb));
        let erw = 1.0 / (10f32.powf(-(pa - pb) / 600.0) + 1.0);
        let ww = if ga > gb { 1.0 } else if ga == gb { 0.5 } else { 0.0 };
        self.nat_punkte.insert(na, pa + k("elo_fenster") * (ww - erw));
        self.nat_punkte.insert(nb, pb + k("elo_fenster") * (erw - ww));
        for (n, tore, gegen) in [(na, ga, gb), (nb, gb, ga)] {
            let Some(ti) = self.nationalteams.iter().position(|t| t.nat == n) else { continue };
            let t = &mut self.nationalteams[ti];
            t.spiele += 1;
            if tore > gegen { t.siege += 1; } else if tore == gegen { t.remis += 1; }
            let mut kader = t.kader.clone();
            kader.sort_by(|a, b| self.players[*b].ovr.partial_cmp(&self.players[*a].ovr).unwrap());
            // the eleven and three substitutes play, goals for the attack
            let spieler: Vec<usize> = kader.iter().take(14).copied().collect();
            for &p in &spieler {
                let pl = &mut self.players[p];
                pl.profil.laenderspiele += 1;
                pl.fit = (pl.fit - k("fitness_je_spiel")).max(20.0);
                if pl.profil.laenderspiele <= 10 { pl.value *= 1.0 + k("wert_je_spiel") as f64; }
            }
            let w: Vec<f32> = spieler.iter().map(|&p| match part_of(&self.players[p].pos) { 0 => 0.0, 1 => 0.4, 2 => 1.2, _ => 3.0 }).collect();
            for _ in 0..tore {
                if w.iter().sum::<f32>() <= 0.0 { break; }
                let p = spieler[self.rng.weighted(&w)];
                self.players[p].profil.nat_tore += 1;
                if let Some(x) = out.iter_mut().find(|x| x.0 == p && x.2) { x.5 += 1; }
            }
            for &p in &spieler {
                if self.rng.chance(k("verletzung_je_spiel")) {
                    let days = [7u16, 14, 28][(self.rng.next() % 3) as usize];
                    self.players[p].inj = self.players[p].inj.max(days);
                    self.players[p].inj_name = injury_names(days)[0].to_string();
                    let e = (self.season, self.day, self.players[p].inj_name.clone(), days);
                    self.players[p].profil.verletzungen.push(e);
                    if let Some(x) = out.iter_mut().find(|x| x.0 == p && x.2) { x.4 = days; }
                }
            }
        }
        (ga, gb)
    }
    /// The Nations League draw (Fabian 24.11.): UEFA nations by ranking into leagues A, B, C of 16 and D for the rest,
    /// groups of four (three at the end) drawn from pots, six matchdays in the three autumn windows.
    fn nl_auslosen(&mut self, nats: &[u16]) {
        let mut uefa: Vec<u16> = nats.iter().copied().filter(|&n| verband(n) == "UEFA").collect();
        uefa.sort_by(|a, b| self.fifa_punkte(*b).partial_cmp(&self.fifa_punkte(*a)).unwrap());
        self.nl.clear();
        for (li, liga) in uefa.chunks(16).enumerate() {
            let n_gr = liga.len().div_ceil(4).max(1);
            let mut gruppen: Vec<Vec<u16>> = vec![vec![]; n_gr];
            // pots: each run of n_gr nations spread over the groups, the order turned every pot (snake)
            for (k, topf) in liga.chunks(n_gr).enumerate() {
                for (j, &n) in topf.iter().enumerate() { let g = if k % 2 == 0 { j } else { n_gr - 1 - j }; gruppen[g.min(n_gr - 1)].push(n); }
            }
            for g in gruppen.into_iter().filter(|g| g.len() >= 2) {
                let paare: &[&[(usize, usize)]] = if g.len() >= 4 { &[&[(0, 1), (2, 3)], &[(0, 2), (1, 3)], &[(3, 0), (1, 2)], &[(1, 0), (3, 2)], &[(2, 0), (3, 1)], &[(0, 3), (2, 1)]] }
                    else if g.len() == 3 { &[&[(0, 1)], &[(1, 2)], &[(2, 0)], &[(1, 0)], &[(2, 1)], &[(0, 2)]] } else { &[&[(0, 1)], &[(1, 0)]] };
                let spiele = paare.iter().enumerate().flat_map(|(r, ps)| ps.iter().map(move |&(h, a)| (r, h, a))).map(|(r, h, a)| NlSpiel { runde: r as u8, h: g[h], a: g[a], hg: -1, ag: -1 }).collect();
                self.nl.push(NlGruppe { liga: li.min(3) as u8, nats: g.into_iter().take(4).collect(), spiele });
            }
        }
        self.nl_saison = self.season + 1;
    }
    /// A Nations League group's table: (nation, played, won, drawn, lost, goals for, against, points), best first.
    pub fn nl_tabelle(&self, g: &NlGruppe) -> Vec<(u16, u8, u8, u8, u8, i16, i16, u8)> {
        let mut t: Vec<(u16, u8, u8, u8, u8, i16, i16, u8)> = g.nats.iter().map(|&n| (n, 0, 0, 0, 0, 0, 0, 0)).collect();
        for sp in g.spiele.iter().filter(|x| x.hg >= 0) {
            for (n, a, b) in [(sp.h, sp.hg, sp.ag), (sp.a, sp.ag, sp.hg)] {
                let Some(r) = t.iter_mut().find(|x| x.0 == n) else { continue };
                r.1 += 1; r.5 += a; r.6 += b;
                if a > b { r.2 += 1; r.7 += 3; } else if a == b { r.3 += 1; r.7 += 1; } else { r.4 += 1; }
            }
        }
        t.sort_by_key(|x| (std::cmp::Reverse(x.7), std::cmp::Reverse(x.5 - x.6), std::cmp::Reverse(x.5)));
        t
    }
    /// A player's standing with his national coach (Teil 3.4): 0 set (core), 1 in the squad, 2 on the list, 3 outside.
    pub fn nat_status(&self, p: usize) -> Option<u8> {
        let pl = &self.players[p];
        let t = self.nationalteams.iter().find(|t| t.nat == pl.nat)?;
        if t.frueher.iter().rev().take(6).filter(|k| k.contains(&p)).count() >= 4 && t.kader.contains(&p) { return Some(0); }
        if t.kader.contains(&p) || t.frueher.last().map_or(false, |k| k.contains(&p)) { return Some(1); }
        let schwaechster = t.kader.iter().filter(|&&q| part_of(&self.players[q].pos) == part_of(&pl.pos)).map(|&q| self.players[q].ovr).fold(f32::MAX, f32::min);
        Some(if pl.ovr >= schwaechster - k("zettel_abstand") { 2 } else { 3 })
    }
}

impl crate::game::Career {
    /// What the break means for the own club: one message with the called-up, first call-ups (+morale), dropped players
    /// (−morale, a reason to talk), injuries and goals; the opponents' and world's moves stay in the news.
    pub fn laenderspiel_folgen(&mut self, f: &[(usize, bool, bool, bool, u16, u16)]) {
        let own = self.own as i32;
        let mut zeilen = vec![];
        let meine: Vec<(usize, bool, bool, bool, u16, u16)> = f.iter().copied().filter(|x| self.world.players[x.0].team == own).collect();
        for (p, erst, dabei, raus, verletzt, tore) in meine {
            let n = self.world.players[p].name.clone();
            let land = crate::nationen::name_anzeige(&self.world, self.world.players[p].nat);
            if dabei {
                let mut z = crate::tf!(if erst { "ui.nt.zeile.erst" } else { "ui.nt.zeile.dabei" }, n.clone(), land.clone(), self.world.players[p].profil.laenderspiele);
                if tore > 0 { z = format!("{z} {}", crate::tf!("ui.nt.zeile.tore", tore)); }
                if verletzt > 0 { z = format!("{z} {}", crate::tf!("ui.nt.zeile.verletzt", verletzt)); }
                zeilen.push(z);
                let m = &mut self.world.players[p].morale;
                *m = (*m + if erst { 5.0 } else { 2.0 }).min(95.0);
                if erst { self.news(&crate::tf!("ui.nt.erst_titel", n.clone(), land.clone()), &crate::tf!("ui.nt.erst_text", n, land)); }
            } else if raus {
                zeilen.push(crate::tf!("ui.nt.zeile.raus", n, land));
                let m = &mut self.world.players[p].morale;
                *m = (*m - 4.0).max(5.0);
            }
        }
        if zeilen.is_empty() { return; }
        self.msg(1, crate::inhalt::t("ui.nt.absender"), crate::inhalt::t("ui.nt.titel"), &zeilen.join(crate::id("\n")), crate::game::Action::None);
    }
}

impl crate::game::Career {
    /// After a tournament: the winner in the news, and one message how the own players fared (heroes come back worth more).
    pub fn turnier_folgen(&mut self, name: &str, spur: &[TurnierSpur], sieger: u16) {
        let land = crate::nationen::name_anzeige(&self.world, sieger);
        self.news(&crate::tf!("ui.nt.turnier.sieger_titel", land.clone(), name.to_string()), &crate::tf!("ui.nt.turnier.sieger_text", land, name.to_string()));
        let own = self.own as i32;
        let zeilen: Vec<String> = spur.iter().filter(|x| self.world.players[x.0].team == own).map(|&(p, n, st, tore)| {
            crate::tf!("ui.nt.turnier.zeile", self.world.players[p].name.clone(), crate::nationen::name_anzeige(&self.world, n), crate::inhalt::t(&format!("ui.nt.stufe.{}", st.min(5))), tore)
        }).collect();
        let meine: Vec<TurnierSpur> = spur.iter().copied().filter(|x| self.world.players[x.0].team == own).collect();
        for (p, _, st, _) in meine {
            let m = &mut self.world.players[p].morale;
            *m = (*m + if st >= 4 { 6.0 } else if st >= 2 { 2.0 } else { -2.0 }).clamp(5.0, 95.0);
        }
        if zeilen.is_empty() { return; }
        self.msg(1, crate::inhalt::t("ui.nt.absender"), &crate::tf!("ui.nt.turnier.titel", name.to_string()), &zeilen.join(crate::id("\n")), crate::game::Action::None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Fabian 24.11.: the Nations League: leagues of 16, groups of four, six matchdays, a table.
    #[test]
    fn nations_league_auslosung() {
        let mut w = crate::world::testwelt(4);
        let nats: Vec<u16> = (190..226).collect();
        w.nl_auslosen(&nats);
        assert_eq!(w.nl.iter().filter(|g| g.liga == 0).count(), 4, "league A: four groups");
        assert!(w.nl.iter().all(|g| g.spiele.iter().map(|x| x.runde).max() == Some(5)), "six matchdays");
        let g = w.nl[0].clone();
        assert!(g.nats.iter().all(|n| g.spiele.iter().filter(|x| x.h == *n || x.a == *n).count() == 6), "everyone plays six");
        let mut g2 = g.clone();
        for sp in g2.spiele.iter_mut() { (sp.hg, sp.ag) = (1, 0); }
        let t = w.nl_tabelle(&g2);
        assert_eq!(t.iter().map(|x| x.7 as u32).sum::<u32>(), 12 * 3);
    }
    #[test]
    fn nominierung_und_laenderspiele() {
        let mut w = crate::world::testwelt(7);
        // the demo world has no nationalities: give the clubs' players two nations
        for (i, p) in w.players.iter_mut().enumerate() { p.nat = if i % 2 == 0 { 210 } else { 208 }; }
        let tag = INTL_BREAKS[0].0;
        let r = w.laenderspielfenster(tag);
        assert!(w.nationalteams.len() >= 2);
        for t in &w.nationalteams {
            assert_eq!(t.kader.iter().filter(|&&p| w.players[p].pos == "GK").count(), 3, "three keepers");
            assert!(t.kader.len() <= 23 && t.kader.iter().all(|&p| w.players[p].nat == t.nat));
            assert_eq!(t.spiele, 2);
            // the U21: young, not in the A squad, of the nation
            assert!(t.u21.iter().all(|&p| w.players[p].age <= 21 && !t.kader.contains(&p) && w.players[p].nat == t.nat));
        }
        assert!(w.nationalteams.iter().any(|t| !t.u21.is_empty()), "U21 squads are picked");
        assert!(r.iter().any(|x| x.1), "first call-ups are reported");
        assert!(w.players.iter().any(|p| p.profil.laenderspiele > 0));
        // the strongest of a nation is called up
        let besten = w.players.iter().enumerate().filter(|(_, p)| p.nat == 210 && p.team >= 0 && p.youth == 0 && p.inj == 0 && p.pos != "GK").max_by(|a, b| a.1.ovr.partial_cmp(&b.1.ovr).unwrap()).unwrap().0;
        let t = w.nationalteams.iter().find(|t| t.nat == 210).unwrap();
        assert!(t.kader.iter().any(|&p| w.players[p].ovr >= w.players[besten].ovr), "the strongest (or one as strong) is called up");
        // a small tournament: four nations, one winner and one runner-up, ranking points move
        for (i, p) in w.players.iter_mut().enumerate() { p.nat = [210, 208, 204, 200][i % 4]; }
        w.laenderspielfenster(INTL_BREAKS[1].0);
        let p0 = w.fifa_punkte(210);
        let (sieger, zweiter, spur) = w.turnier_spielen(&[210, 208, 204, 200]);
        assert!([210, 208, 204, 200].contains(&sieger) && sieger != zweiter && zweiter != 0);
        assert!(spur.iter().any(|x| x.2 == 5));
        assert!(w.fifa_punkte(210) != p0);

    }
}
