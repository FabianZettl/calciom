//! The AI clubs' transfers with reasons (konzepte/transfers-und-verhandlungen.md, Teil 1): every club has a profile
//! (prestige, two philosophies, network), a squad plan per position group and at most three open needs; players get a
//! status at their club (transfer list, loan list, …) with a reason; a move only happens when the buyer has a need,
//! the seller wants or has to sell and the player is willing. Every move carries its chain of reasons into the news.
//! Detailed clubs (own league and first divisions) trade daily in the window; the rest stays with the simple market.
use crate::game::*;
use crate::mensch::zufall;
use crate::world::*;
use serde::{Deserialize, Serialize};

/// Position groups of the squad plan: goal, centre-back, full-back, central midfield, wide/attacking, striker.
pub const GRUPPEN: [&str; 6] = [crate::id("tw"), crate::id("iv"), crate::id("av"), crate::id("zm"), crate::id("off"), crate::id("st")];
/// (regulars, back-ups) per group (konzept 1.2 b).
const SOLL: [(usize, usize); 6] = [(1, 1), (2, 2), (2, 1), (3, 2), (3, 2), (2, 1)];

pub fn gruppe(pos: &str) -> usize {
    match pos { "GK" => 0, "CB" => 1, "LB" | "RB" => 2, "DMF" | "CMF" => 3, "LMF" | "RMF" | "AMF" | "LWF" | "RWF" => 4, _ => 5 }
}

/// A player's standing at his club (konzept 1.5).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Status { Unverkaeuflich, Stamm, Abgebbar, Transferliste, Leihliste }

/// The status of one player with its reason key (ui.km.grund.*), set by the weekly review.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarktStatus { pub p: usize, pub status: Status, pub grund: String }

/// The two philosophies of a club (konzept 1.2 a), main first.
pub const PHILOSOPHIEN: [&str; 6] = [crate::id("starkaeufer"), crate::id("ausbildung"), crate::id("weiterverkauf"), crate::id("erfahrung"), crate::id("jugend"), crate::id("sparfuchs")];

impl Career {
    /// Detailed clubs (konzept 1.8, about 20 %): own league, first divisions and second divisions; not second teams.
    fn detailliert(&self, c: usize) -> bool { let l = &self.world.leagues[self.world.clubs[c].league]; (l.detail <= 1 || l.key.contains("D2_LEAGUE")) && self.world.erste_von(c).is_none() }

    /// Prestige 0–100 of a club: its reputation.
    pub fn vereins_prestige(&self, c: usize) -> f32 { self.world.clubs[c].rep }

    /// Main and side philosophy from prestige, money, age of the squad and the league's country; fixed per club.
    pub fn philosophie(&self, c: usize) -> (&'static str, &'static str) {
        let cl = &self.world.clubs[c];
        let sq: Vec<&Player> = cl.squad.iter().map(|&p| &self.world.players[p]).collect();
        let alter = sq.iter().map(|p| p.age as f32).sum::<f32>() / sq.len().max(1) as f32;
        let land = &self.world.leagues[cl.league].land;
        let arm = cl.balance < self.world.revenue(c) * 0.1;
        let z = zufall(&["2026", crate::id("philosophie"), &cl.id.to_string()]);
        let mut p: Vec<&'static str> = vec![];
        if cl.rep >= 75.0 && !arm { p.push(PHILOSOPHIEN[0]); }
        if [crate::id("Niederlande"), crate::id("Portugal"), crate::id("Belgien"), crate::id("Österreich"), crate::id("Netherlands"), crate::id("Belgium"), crate::id("Austria")].contains(&land.as_str()) || z < 0.2 { p.push(PHILOSOPHIEN[2]); }
        if alter <= 25.0 || (cl.rep >= 40.0 && cl.rep < 70.0 && z > 0.6) { p.push(PHILOSOPHIEN[1]); }
        if cl.rep < 40.0 { p.push(PHILOSOPHIEN[3]); }
        if arm { p.push(PHILOSOPHIEN[5]); }
        if z > 0.85 { p.push(PHILOSOPHIEN[4]); }
        p.dedup();
        let erste = p.first().copied().unwrap_or(if cl.rep >= 60.0 { PHILOSOPHIEN[0] } else { PHILOSOPHIEN[3] });
        let zweite = p.get(1).copied().unwrap_or(if erste == PHILOSOPHIEN[5] { PHILOSOPHIEN[3] } else { PHILOSOPHIEN[5] });
        (erste, zweite)
    }

    /// How well club `c` knows a player of club `von` (konzept 1.2 a network): own league 1, own country 0.8, rest 0.15.
    fn bekannt(&self, c: usize, von: usize) -> f32 {
        let (a, b) = (self.world.clubs[c].league, self.world.clubs[von].league);
        if a == b { 1.0 } else if !self.world.leagues[a].land.is_empty() && self.world.leagues[a].land == self.world.leagues[b].land { 0.8 } else { 0.15 }
    }

    /// The squad of a club sorted per group (strongest first), without loanees out.
    fn gruppen(&self, c: usize) -> [Vec<usize>; 6] {
        let mut g: [Vec<usize>; 6] = Default::default();
        for &p in &self.world.clubs[c].squad { let pl = &self.world.players[p]; if pl.youth == 0 { g[gruppe(&pl.pos)].push(p); } }
        for v in g.iter_mut() { v.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap()); }
        g
    }

    /// The open needs of a club (konzept 1.2 c): (group, urgency 0–100, reason key), the most urgent first, at most
    /// 2 (top clubs), 3 or 4 (clubs in crisis). In winter only urgencies from 70 count.
    pub fn bedarf(&self, c: usize) -> Vec<(usize, f32, &'static str)> {
        let g = self.gruppen(c);
        let season = self.world.season;
        let stamm: Vec<f32> = g.iter().enumerate().flat_map(|(i, v)| v.iter().take(SOLL[i].0).map(|&p| self.world.players[p].ovr).collect::<Vec<_>>()).collect();
        let schnitt = stamm.iter().sum::<f32>() / stamm.len().max(1) as f32;
        let mut out = vec![];
        for (i, v) in g.iter().enumerate() {
            let (s, e) = SOLL[i];
            let gesund: Vec<usize> = v.iter().copied().filter(|&p| self.world.players[p].inj < 60).collect();
            if gesund.len() < s { out.push((i, 95.0, crate::id("leer"))); continue; }
            if v.len() < s + e { out.push((i, 60.0, crate::id("leer"))); continue; }
            if let Some(&best) = gesund.get(s - 1) { if self.world.players[best].ovr < schnitt - 5.0 { out.push((i, 65.0, crate::id("qualitaet"))); continue; } }
            if v.iter().take(s).any(|&p| self.world.players[p].inj >= 60) { out.push((i, 70.0, crate::id("ausfall"))); continue; }
            if v.iter().take(s).any(|&p| self.world.players[p].contract_end <= season && self.world.day >= 150) { out.push((i, 50.0, crate::id("vertrag"))); continue; }
            if v.iter().take(s).any(|&p| self.world.players[p].age >= 32) && !v.iter().skip(s).any(|&p| self.world.players[p].age <= 26) { out.push((i, 40.0, crate::id("alter"))); continue; }
            // reinforcement (konzept 1.2 c, Gelegenheit/Ziel): the weakest regular of the group is below the club's level
            if let Some(&w) = v.get(s - 1) { if self.world.players[w].ovr < self.world.clubs[c].level && self.world.clubs[c].balance > 0.0 { out.push((i, 35.0, crate::id("verstaerkung"))); } }
        }
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let winter = self.in_window().map_or(false, |(a, _)| a > 100);
        if winter { out.retain(|x| x.1 >= 70.0); }
        let max = if self.world.clubs[c].rep >= 80.0 { 2 } else if self.world.clubs[c].balance < 0.0 { 4 } else { 3 };
        out.truncate(max);
        out
    }

    /// Weekly review of every detailed AI club (konzept 1.5): status and reason per player, at most 15 % of a squad on
    /// the transfer or loan list. The transfer list also sets `listed` (asking price 1.0 instead of 1.3).
    pub fn marktstatus_pruefen(&mut self) {
        let season = self.world.season;
        let mut neu = vec![];
        for c in 0..self.world.clubs.len() {
            if c == self.own || !self.detailliert(c) { continue; }
            let g = self.gruppen(c);
            let n = self.world.clubs[c].squad.len();
            let mut liste: Vec<(usize, Status, &'static str, f32)> = vec![];
            let mut gehalt: Vec<usize> = self.world.clubs[c].squad.clone();
            gehalt.sort_by(|a, b| self.world.players[*b].wage.partial_cmp(&self.world.players[*a].wage).unwrap());
            let mut stark: Vec<usize> = self.world.clubs[c].squad.clone();
            stark.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
            for (i, v) in g.iter().enumerate() {
                for (rang, &p) in v.iter().enumerate() {
                    let pl = &self.world.players[p];
                    if pl.leihe.is_some() { continue; }
                    let top = stark.iter().position(|&x| x == p).unwrap_or(99) < 3;
                    let (status, grund, gewicht): (Status, &'static str, f32) =
                        if top && pl.contract_end > season { (Status::Unverkaeuflich, "", 0.0) }
                        else if rang < SOLL[i].0 { (Status::Stamm, "", 0.0) }
                        else if pl.age <= 22 && pl.pot >= pl.ovr + 4.0 && pl.apps < 5 && self.world.day > 60 { (Status::Leihliste, crate::id("praxis"), 3.0) }
                        else if rang >= SOLL[i].0 + SOLL[i].1 && pl.age >= 24 { (Status::Transferliste, crate::id("kein_platz"), 3.0) }
                        else if pl.age >= 31 && pl.contract_end <= season { (Status::Transferliste, crate::id("alter_vertrag"), 2.0) }
                        else if gehalt.iter().position(|&x| x == p).unwrap_or(99) + 8 < stark.iter().position(|&x| x == p).unwrap_or(0) { (Status::Transferliste, crate::id("zu_teuer"), 1.5) }
                        else { (Status::Abgebbar, "", 0.0) };
                    liste.push((p, status, grund, gewicht));
                }
            }
            // gemäßigt: at most 15 % of the squad on a list, the strongest reasons first
            let max = ((n as f32) * 0.15).ceil() as usize;
            let mut auf: Vec<&mut (usize, Status, &'static str, f32)> = liste.iter_mut().filter(|x| matches!(x.1, Status::Transferliste | Status::Leihliste)).collect();
            auf.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
            for x in auf.into_iter().skip(max) { x.1 = Status::Abgebbar; x.2 = ""; }
            for (p, s, gr, _) in liste {
                self.world.players[p].listed = s == Status::Transferliste;
                neu.push(MarktStatus { p, status: s, grund: gr.to_string() });
            }
        }
        self.marktstatus = neu;
    }

    /// Fabian 22.11.: the family sits at the table up to 19 (konzept 2.8): a type per player (ambitious father,
    /// caring mother, down-to-earth family, family as adviser), fixed by his id.
    pub fn familie_typ(&self, p: usize) -> Option<&'static str> {
        if self.world.players[p].age > 19 { return None; }
        let t = [crate::id("vater"), crate::id("mutter"), crate::id("bodenstaendig"), crate::id("berater")];
        Some(t[(zufall(&["2026", crate::id("familie"), &self.world.players[p].id.to_string()]) * 4.0) as usize % 4])
    }
    /// What the family's wishes add to the chance in the player talk: the father likes a big name, the mother a club
    /// of his own country, the down-to-earth family a real chance to play; the family as adviser looks at money only.
    pub fn familie_bonus(&self, p: usize) -> f32 {
        let Some(t) = self.familie_typ(p) else { return 0.0 };
        let pl = &self.world.players[p];
        match t {
            "vater" => ((self.world.clubs[self.own].rep - 50.0) / 200.0).clamp(-0.15, 0.15),
            "mutter" => if pl.nat != 0 && self.world.land_nat(self.league()) == pl.nat { 0.1 } else { -0.1 },
            "bodenstaendig" => { let g = gruppe(&pl.pos); let besser = self.squad().iter().filter(|&&x| gruppe(&self.world.players[x].pos) == g && self.world.players[x].ovr > pl.ovr).count(); if besser < 3 { 0.1 } else { -0.05 } }
            _ => 0.0,
        }
    }

    pub fn status_von(&self, p: usize) -> Option<&MarktStatus> { self.marktstatus.iter().find(|x| x.p == p) }

    /// Fit 0–100 of a candidate for a need of club `c` (konzept 1.3): 30 % strength for the role, 20 % philosophy,
    /// 20 % value for money, 15 % feasibility, 15 % how well the club knows him. None when a hard limit blocks.
    pub fn passung(&self, c: usize, gr: usize, p: usize, preis: f64, budget: f64) -> Option<f32> {
        let g = self.gruppen(c);
        let ziel = g[gr].iter().take(SOLL[gr].0).map(|&x| self.world.players[x].ovr).fold(0.0f32, f32::max).max(self.world.clubs[c].level);
        self.passung_mit(c, p, preis, budget, ziel, self.philosophie(c).0, self.status_von(p).map(|s| s.status))
    }
    /// passung with what is the same for every candidate of one need worked out once (speed: one day of the window
    /// looks at thousands of players).
    fn passung_mit(&self, c: usize, p: usize, preis: f64, budget: f64, ziel: f32, ph: &str, st: Option<Status>) -> Option<f32> {
        let pl = &self.world.players[p];
        let von = pl.team;
        if von < 0 || von as usize == c || von as usize == self.own || pl.youth > 0 || pl.leihe.is_some() || pl.inj >= 28 { return None; }
        if self.abs().saturating_sub(pl.joined) < 150 && pl.joined > 0 { return None; }
        if preis > budget { return None; }
        if self.kaderregel_verstoss(c, p).is_some() { return None; }
        let ruf = self.player_rep(p);
        if ruf > self.vereins_prestige(c) + 20.0 && pl.age < 31 { return None; }
        let von = von as usize;
        // a top-5 player never goes to a clearly weaker club of the same league
        if self.world.clubs[von].league == self.world.clubs[c].league && self.world.clubs[von].rep > self.world.clubs[c].rep + 5.0 {
            let mut st = self.world.clubs[von].squad.clone();
            st.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
            if st.iter().take(5).any(|&x| x == p) { return None; }
        }
        let staerke = ((pl.ovr - (ziel - 5.0)) / 10.0).clamp(0.0, 1.0);
        let philo = match ph {
            "starkaeufer" => if (24..=29).contains(&pl.age) { 1.0 } else { 0.4 },
            "ausbildung" | "jugend" => if pl.age <= 21 && pl.pot >= pl.ovr + 5.0 { 1.0 } else { 0.3 },
            "weiterverkauf" => (self.moneyball(p) / 10.0 + 0.5).clamp(0.0, 1.0),
            "erfahrung" => if (27..=32).contains(&pl.age) { 1.0 } else { 0.4 },
            _ => if pl.contract_end <= self.world.season { 1.0 } else { 0.4 },
        };
        let geld = (1.0 - preis / budget.max(1.0)).clamp(0.0, 1.0) as f32;
        let status = match st { Some(Status::Transferliste) => 1.0, Some(Status::Abgebbar) | None => 0.6, Some(Status::Leihliste) => 0.5, Some(Status::Stamm) => 0.2, Some(Status::Unverkaeuflich) => 0.0 };
        let machbar = (1.0 - (ruf - self.vereins_prestige(c)).max(0.0) / 40.0) * status;
        let kennt = self.bekannt(c, von);
        Some(100.0 * (0.30 * staerke + 0.20 * philo + 0.20 * geld + 0.15 * machbar + 0.15 * kennt))
    }

    /// Asking price of the seller by status (konzept 1.5): transfer list 85 %, open 100 %, regular 130 % of the value.
    fn ki_forderung(&self, p: usize, st: Option<Status>) -> f64 {
        let f = match st { Some(Status::Transferliste) => 0.85, Some(Status::Stamm) => 1.3, Some(Status::Unverkaeuflich) => 3.0, _ => 1.0 };
        (self.world.players[p].value * f / 50_000.0).round() * 50_000.0
    }

    /// One day of the AI market in the window: every detailed AI club with a need looks at its best candidate (only
    /// in its field of view; unknown players of small leagues often stay hidden) and buys when seller and player agree.
    pub fn ki_markt_tag(&mut self) {
        if self.in_window().is_none() { return; }
        let abs = self.abs();
        if self.marktstatus.is_empty() || (abs % 7 == 0) { self.marktstatus_pruefen(); }
        let tag = abs.to_string();
        let status: std::collections::HashMap<usize, Status> = self.marktstatus.iter().map(|m| (m.p, m.status)).collect();
        // the players per position group, strongest first (one sort a day instead of a full scan per look)
        let mut nach_gruppe: [Vec<usize>; 6] = Default::default();
        for (p, pl) in self.world.players.iter().enumerate() { if pl.team >= 0 && pl.youth == 0 { nach_gruppe[gruppe(&pl.pos)].push(p); } }
        for v in nach_gruppe.iter_mut() { v.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap()); }
        let kaeufer: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.detailliert(c)).collect();
        let mut wechsel = 0;
        for c in kaeufer {
            // not every club acts every day: spread over the window (a look every three days or so)
            if zufall(&["2026", crate::id("marktblick"), &c.to_string(), &tag]) > 0.35 { continue; }
            for (gr, dringend, bedarf_grund) in self.bedarf(c) {
            let budget = (self.world.clubs[c].balance * 0.5 + self.world.revenue(c) * 0.1).max(0.0);
            let g = self.gruppen(c);
            let ziel = g[gr].iter().take(SOLL[gr].0).map(|&x| self.world.players[x].ovr).fold(0.0f32, f32::max).max(self.world.clubs[c].level);
            let ph = self.philosophie(c).0;
            let mut best: Option<(f32, usize, f64)> = None;
            for &p in &nach_gruppe[gr] {
                let pl = &self.world.players[p];
                if pl.ovr < ziel - 8.0 { break; } // sorted strongest first
                if pl.team < 0 { continue; }
                let von = pl.team as usize;
                // field of view: what a club does not know it does not see (except the transfer list of the world)
                if !pl.listed && zufall(&["2026", crate::id("blick"), &c.to_string(), &pl.id.to_string()]) > self.bekannt(c, von) { continue; }
                let st = status.get(&p).copied();
                let preis = self.ki_forderung(p, st);
                if let Some(s) = self.passung_mit(c, p, preis, budget, ziel, ph, st) { if best.map_or(true, |b| s > b.0) { best = Some((s, p, preis)); } }
            }
            let Some((s, p, preis)) = best else { continue };
            if s < 45.0 || dringend < 30.0 { continue; }
            let von = self.world.players[p].team as usize;
            self.ki_wechsel(p, von, c, preis, bedarf_grund);
            wechsel += 1;
            break; // one signing per look; the next need comes at the next look (domino stays calm)
            }
            if wechsel >= 400 { break; }
        }
        self.ki_leihen_tag(abs);
        if abs % 7 == 3 { self.ki_anwerben(abs); }
    }

    /// Fabian 22.11.: AI clubs may also court the user's players directly (konzept 2, question 2). Once a week in the
    /// window a detailed club with a need in a player's group and at least his club's prestige approaches one player
    /// who is open to a move (willingness ≥ 55): that counts as an offer for his willingness (unrest), with news and a
    /// note in the inbox, so the user can talk to him.
    fn ki_anwerben(&mut self, abs: u32) {
        let own_rep = self.world.clubs[self.own].rep;
        let woche = (abs / 7).to_string();
        let mut kand: Vec<usize> = self.squad().into_iter().filter(|&p| { let pl = &self.world.players[p]; pl.ps.an && pl.ps.wb >= 55.0 && pl.leihe.is_none() && pl.youth == 0 }).collect();
        kand.sort_by(|a, b| self.world.players[*b].ps.wb.partial_cmp(&self.world.players[*a].ps.wb).unwrap());
        for p in kand {
            if zufall(&["2026", crate::id("anwerben"), &p.to_string(), &woche]) > 0.3 { continue; }
            let gr = gruppe(&self.world.players[p].pos);
            let wert = self.world.players[p].value;
            let werber = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.detailliert(c) && self.world.clubs[c].rep >= own_rep - 5.0 && self.world.clubs[c].balance * 0.5 + self.world.revenue(c) * 0.1 >= wert)
                .filter(|&c| self.bedarf(c).iter().any(|b| b.0 == gr)).max_by(|a, b| self.world.clubs[*a].rep.partial_cmp(&self.world.clubs[*b].rep).unwrap());
            let Some(c) = werber else { continue };
            self.world.players[p].ps.angebote.push(abs);
            let (n, cn) = (self.world.players[p].name.clone(), self.world.clubs[c].name.clone());
            self.news(&crate::tf!("ui.km.anwerben", cn, n), crate::inhalt::t("ui.km.anwerben_text"));
            let co = self.co_name.clone();
            self.msg(1, &co, &crate::tf!("ui.km.anwerben", cn, n), &crate::tf!("ui.km.anwerben_post", n, cn), Action::None);
            self.grund(&n, crate::id("anwerben"), 1.0, &crate::tf!("ui.km.anwerben", cn, n));
            break; // one a week
        }
    }

    /// Loan list players of detailed AI clubs go on loan (konzept 1.6): to a club of the same country where they will
    /// probably play (at least the level of its regulars − 3), a partner club first; fee 15 % of the value.
    fn ki_leihen_tag(&mut self, abs: u32) {
        let liste: Vec<(usize, String)> = self.marktstatus.iter().filter(|m| m.status == Status::Leihliste).map(|m| (m.p, m.grund.clone())).collect();
        let season = self.world.season;
        for (p, grund) in liste {
            if zufall(&["2026", crate::id("leihblick"), &p.to_string(), &abs.to_string()]) > 0.2 { continue; }
            let pl = &self.world.players[p];
            if pl.team < 0 || pl.leihe.is_some() { continue; }
            let von = pl.team as usize;
            if von == self.own { continue; }
            let land = self.world.leagues[self.world.clubs[von].league].land.clone();
            let gr = gruppe(&pl.pos);
            let ovr = pl.ovr;
            let partner: Vec<usize> = self.world.koops.iter().filter(|k| k.haupt == von).map(|k| k.partner).collect();
            let mut best: Option<(f32, usize)> = None;
            for n in 0..self.world.clubs.len() {
                if n == von || n == self.own || self.world.clubs[n].squad.len() >= 30 { continue; }
                let gleich = !land.is_empty() && self.world.leagues[self.world.clubs[n].league].land == land;
                if !gleich && !partner.contains(&n) { continue; }
                let g = self.gruppen(n);
                let stamm = g[gr].iter().take(SOLL[gr].0).map(|&x| self.world.players[x].ovr).sum::<f32>() / SOLL[gr].0.max(1) as f32;
                if ovr < stamm - 3.0 || self.world.clubs[n].level > self.world.clubs[von].level { continue; }
                let s = -(ovr - stamm).abs() + if partner.contains(&n) { 10.0 } else { 0.0 };
                if best.map_or(true, |b| s > b.0) { best = Some((s, n)); }
            }
            let Some((_, nach)) = best else { continue };
            let gebuehr = (self.world.players[p].value * 0.15 / 10_000.0).round() * 10_000.0;
            self.world.clubs[nach].balance -= gebuehr;
            self.world.clubs[von].balance += gebuehr;
            self.world.clubs[von].squad.retain(|&x| x != p);
            self.world.clubs[nach].squad.push(p);
            let pl = &mut self.world.players[p];
            pl.team = nach as i32;
            pl.listed = false;
            pl.leihe = Some(crate::transfer::Leihe { von, bis: season, anteil: 0.7, option: 0.0, spielzeit: true, grund: grund.clone() });
            self.marktstatus.retain(|x| x.p != p);
            let (nm, vn, nn) = (self.world.players[p].name.clone(), self.world.clubs[von].name.clone(), self.world.clubs[nach].name.clone());
            let warum = if grund.is_empty() { String::new() } else { crate::tf!(&format!("ui.km.grund.{grund}"), nm, vn) };
            self.news(&crate::tf!("ui.km.leihe", vn, nm, nn), &warum);
        }
    }

    /// Carries out an AI move and writes the news with its chain of reasons (need of the buyer, status at the seller).
    fn ki_wechsel(&mut self, p: usize, von: usize, nach: usize, fee: f64, bedarf_grund: &str) {
        let status_grund = self.status_von(p).map(|s| s.grund.clone()).unwrap_or_default();
        self.weiterverkauf_abrechnen(p, von, fee);
        self.world.clubs[von].squad.retain(|&x| x != p);
        self.world.clubs[von].balance += fee;
        self.world.clubs[nach].squad.push(p);
        self.world.clubs[nach].balance -= fee;
        let lv = self.world.leagues[self.world.clubs[nach].league].level;
        let (season, abs) = (self.world.season, self.abs());
        let pl = &mut self.world.players[p];
        pl.team = nach as i32;
        pl.contract_end = season + contract_years(pl.age) - 1;
        pl.wage = fair_wage(pl, lv);
        pl.joined = abs;
        pl.listed = false;
        self.marktstatus.retain(|x| x.p != p);
        let (nm, bn, sn, pos) = (self.world.players[p].name.clone(), self.world.clubs[nach].name.clone(), self.world.clubs[von].name.clone(), pos_de(&self.world.players[p].pos).to_string());
        let mut warum = crate::inhalt::t(&format!("ui.km.bedarf.{bedarf_grund}")).to_string();
        if !status_grund.is_empty() { warum = format!("{warum} {}", crate::tf!(&format!("ui.km.grund.{status_grund}"), nm, sn)); }
        self.news(&crate::tf!("ui.km.wechsel", bn, pos, nm, sn, money(fee)), &warum);
        self.world.refresh_levels();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn karriere() -> Career {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(0);
        Career::new(w, own, "Test")
    }

    #[test]
    fn bedarf_status_und_gruende() {
        let mut c = karriere();
        // a club that loses both keepers has an urgent need for a goalkeeper
        let k = (0..c.world.clubs.len()).find(|&k| k != c.own && c.detailliert(k)).unwrap();
        let tw: Vec<usize> = c.world.clubs[k].squad.iter().copied().filter(|&p| c.world.players[p].pos == "GK").collect();
        for &p in &tw { c.world.players[p].inj = 90; }
        let b = c.bedarf(k);
        assert_eq!(b.first().map(|x| (x.0, x.2)), Some((0, "leer")), "{b:?}");
        assert!(b.len() <= 4);
        // the weekly review: lists with reasons, at most 15 % of a squad
        c.marktstatus_pruefen();
        for k in 0..c.world.clubs.len() {
            if k == c.own || !c.detailliert(k) { continue; }
            let n = c.world.clubs[k].squad.len();
            let auf = c.marktstatus.iter().filter(|s| c.world.players[s.p].team == k as i32 && matches!(s.status, Status::Transferliste | Status::Leihliste)).count();
            assert!(auf as f32 <= (n as f32 * 0.15).ceil(), "{auf} von {n}");
        }
        assert!(c.marktstatus.iter().all(|s| !matches!(s.status, Status::Transferliste | Status::Leihliste) || !s.grund.is_empty()), "jede Liste hat einen Grund");
    }

    #[test]
    fn familie_bis_19() {
        let mut c = karriere();
        let p = c.world.players.iter().position(|x| x.team >= 0 && x.team != c.own as i32).unwrap();
        c.world.players[p].age = 18;
        let t = c.familie_typ(p).unwrap();
        assert!(["vater", "mutter", "bodenstaendig", "berater"].contains(&t));
        assert!(c.familie_bonus(p).abs() <= 0.15);
        c.world.players[p].age = 20;
        assert!(c.familie_typ(p).is_none() && c.familie_bonus(p) == 0.0, "ab 20 ohne Familie");
    }

    #[test]
    fn ki_wirbt_wechselwillige_an() {
        let mut c = karriere();
        let sq = c.squad();
        for &p in &sq { c.world.players[p].ps.wb = 90.0; c.world.players[p].ps.an = true; }
        let vorher = c.inbox.len();
        for _ in 0..31 { c.ki_markt_tag(); c.world.day += 1; }
        let umworben = sq.iter().filter(|&&p| !c.world.players[p].ps.angebote.is_empty()).count();
        assert!(umworben >= 1 && umworben <= 5, "einmal je Woche: {umworben}");
        assert!(c.inbox.len() > vorher);
        assert!(sq.iter().all(|&p| c.world.players[p].team == c.own as i32), "Anwerben ist kein Kauf");
    }

    #[test]
    fn wechsel_mit_gruenden_im_fenster() {
        let mut c = karriere();
        let vorher: Vec<i32> = c.world.players.iter().map(|p| p.team).collect();
        for _ in 0..31 { c.ki_markt_tag(); c.world.day += 1; }
        let bewegt = c.world.players.iter().enumerate().filter(|(i, p)| p.team != vorher[*i] && p.team >= 0 && vorher[*i] >= 0).count();
        assert!(bewegt > 0, "im Sommerfenster wechselt jemand");
        assert!(c.world.players.iter().enumerate().all(|(i, p)| !(vorher[i] == c.own as i32 && p.team != c.own as i32)), "KI kauft nie direkt beim Nutzer");
        assert!(c.feed.iter().any(|n| !n.text.is_empty()), "jede Meldung hat einen Grund");
    }
}

#[cfg(test)]
mod menge {
    use super::*;
    /// Measures how many players the first-division clubs sign in one summer window on the real world (konzept 1.8:
    /// 6–10 incl. loans and youth). Run with `cargo test --release wechselmenge -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn wechselmenge() {
        let d = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&d) { return; }
        let w = World::from_pesdb(&d, 7).unwrap();
        let own = w.clubs.iter().position(|c| c.id == 396).unwrap();
        let mut c = Career::new(w, own, "Test");
        let vorher: Vec<i32> = c.world.players.iter().map(|p| p.team).collect();
        let t0 = std::time::Instant::now();
        while c.in_window().is_some() || c.world.day < 5 { c.ki_markt_tag(); c.world.day += 1; if c.world.day > 40 { break; } }
        let tage = c.world.day;
        let mut zu: std::collections::HashMap<usize, usize> = Default::default();
        for (i, p) in c.world.players.iter().enumerate() { if p.team >= 0 && p.team != vorher[i] { *zu.entry(p.team as usize).or_default() += 1; } }
        let d1: Vec<usize> = (0..c.world.clubs.len()).filter(|&k| c.world.leagues[c.world.clubs[k].league].key.contains("D1_LEAGUE")).collect();
        let summe: usize = d1.iter().map(|k| zu.get(k).copied().unwrap_or(0)).sum();
        eprintln!("{tage} Tage, {:.1} s, Erstligisten {}, Zugänge je Verein {:.2}", t0.elapsed().as_secs_f32(), d1.len(), summe as f32 / d1.len() as f32);
    }
}
