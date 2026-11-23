//! Ehrungen übers Jahr (Punkt 6): port of pesm/fl/sim/ehrungen.py in a lean form. Team and player of the week (own
//! league, balanced by position, at most three per club; own players +1.5 morale, „Dauergast“ at 3, 5, 8, 12 picks),
//! player and coach of the month, season awards (player, top scorer, young player, fair play, coach) and the Ballon d'Or.
//! Numbers from the original's config/ehrungen.json, texts from its config/texte/ehrungen.json (both copied unchanged;
//! texts as keys in ui.de.json and ui.en.json).
use crate::game::*;
use crate::mensch::zufall;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

fn datei(name: &str) -> &'static Value {
    static K: OnceLock<Value> = OnceLock::new();
    static T: OnceLock<Value> = OnceLock::new();
    let lesen = |rel: &str| crate::inhalt::lesen(&crate::inhalt::ordner().join(rel)).unwrap_or(Value::Null);
    if name == "konfig" { K.get_or_init(|| lesen("pesm/fl/sim/config/ehrungen.json")) } else { T.get_or_init(|| lesen("pesm/fl/sim/config/texte/ehrungen.json")) }
}
fn k(pfad: &str) -> &'static Value { pfad.split('.').fold(datei(crate::id("konfig")), |v, p| &v[p]) }
fn kf(pfad: &str) -> f32 { k(pfad).as_f64().unwrap_or(0.0) as f32 }
/// A sentence from the original's parts (texts in ui.de.json / ui.en.json as ehrung.teile.<key>.<part>.<variant>):
/// one variant of each part by seed, joined, placeholders filled.
fn teile(key: &str, samen: &str, var: &[(&str, String)]) -> String {
    let mut s: Vec<&str> = vec![];
    for i in 0.. {
        let l: Vec<&str> = (0..).map(|j| crate::inhalt::t(&format!("ehrung.teile.{key}.{i}.{j}"))).take_while(|x| !x.is_empty()).collect();
        if l.is_empty() { break; }
        let j = (zufall(&[crate::id("ehrung"), key, samen, &i.to_string()]) * l.len() as f32) as usize % l.len();
        s.push(l[j]);
    }
    fuellen(&s.join(" "), var)
}
fn fuellen(s: &str, var: &[(&str, String)]) -> String { var.iter().fold(s.to_string(), |a, (k, v)| a.replace(&format!("{{{k}}}"), v)) }
fn titel(key: &str, var: &[(&str, String)]) -> String { fuellen(crate::inhalt::t(&format!("ehrung.t.{key}")), var) }

/// One award: season, kind (week, month, season, ballon …), the player (id) or the club, the text.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Ehrung { pub season: u32, pub art: String, pub pid: Option<u32>, pub club: Option<usize>, pub titel: String }

impl Career {
    fn ehrung(&mut self, art: &str, pid: Option<usize>, club: Option<usize>, titel_: String, grund: String, moral: f32) {
        let season = self.world.season;
        self.ehrungen.push(Ehrung { season, art: art.to_string(), pid: pid.map(|p| self.world.players[p].id), club, titel: titel_.clone() });
        let eigen = pid.map_or(false, |p| self.world.players[p].team == self.own as i32) || club == Some(self.own);
        if eigen {
            if let Some(p) = pid { if moral > 0.0 { self.moral_wirkung(p, moral, 0, &titel_); } }
            self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), &titel_, &grund, Action::None);
            // the winner as a picture card with his average grade
            if let Some(p) = pid { let pl = &self.world.players[p]; let n = if pl.rn > 0 { pl.rsum / pl.rn as f32 } else { 0.0 }; self.meldung_karte(crate::pages::Art::Meilenstein, crate::id("ehrung"), vec![(p, n)]); }
        } else { self.news(&titel_, &grund); }
    }
    /// Monday: team and player of the week from the grades of the own league's matches of the last seven days.
    pub fn ehrungen_woche(&mut self) {
        if crate::date::weekday(self.world.day) != 1 { return; }
        let l = self.league();
        let noten = std::mem::take(&mut self.world.noten_woche);
        // best grade per player in the own league
        let mut beste: Vec<(usize, f32, u8)> = vec![];
        for (p, r, g) in noten {
            let t = self.world.players[p].team;
            if t < 0 || self.world.clubs[t as usize].league != l { continue; }
            match beste.iter_mut().find(|x| x.0 == p) { Some(x) => if r > x.1 { x.1 = r; x.2 = g; }, None => beste.push((p, r, g)) }
        }
        if beste.len() < 11 { return; }
        beste.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        // 4-3-3 balanced by position, at most club_cap per club
        let cap = kf("week.club_cap") as usize;
        let mut elf: Vec<usize> = vec![];
        for (teil, n) in [(0usize, 1usize), (1, 4), (2, 3), (3, 3)] {
            let mut genommen = 0;
            for x in &beste {
                if genommen >= n { break; }
                let pl = &self.world.players[x.0];
                if crate::world::part_of(&pl.pos) != teil || elf.contains(&x.0) { continue; }
                if elf.iter().filter(|&&q| self.world.players[q].team == pl.team).count() >= cap { continue; }
                elf.push(x.0);
                genommen += 1;
            }
        }
        let meine: Vec<String> = elf.iter().filter(|&&p| self.world.players[p].team == self.own as i32).map(|&p| self.world.players[p].name.clone()).collect();
        for &p in &elf {
            let id = self.world.players[p].id;
            let n = { let e = self.elf_der_woche.iter_mut().find(|x| x.0 == id); match e { Some(x) => { x.1 += 1; x.1 } None => { self.elf_der_woche.push((id, 1)); 1 } } };
            if self.world.players[p].team == self.own as i32 {
                self.moral_wirkung(p, kf("week.morale"), 0, crate::inhalt::t("ui.ehrung.elf_der_woche"));
                if k("week.dg_steps").as_array().map_or(false, |a| a.iter().any(|x| x.as_u64() == Some(n as u64))) {
                    let name = self.world.players[p].name.clone();
                    self.news(&titel("dg_news", &[(crate::id("name"), name.clone())]), &crate::tf!("ui.ehrung.dauergast", name, n));
                }
            }
        }
        let lg = self.world.leagues[l].name.clone();
        let wer = beste[0].0;
        let (wn, wc) = (self.world.players[wer].name.clone(), self.world.players[wer].team.max(0) as usize);
        let cn = self.world.clubs[wc].name.clone();
        self.letzte_elf = elf.iter().map(|&p| self.world.players[p].id).collect();
        if !meine.is_empty() {
            self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), &crate::tf!("ui.ehrung.elf_titel", lg), &format!("{} {}", titel("week_news", &[(crate::id("lg"), lg.clone()), (crate::id("n"), meine.len().to_string()), (crate::id("names"), meine.join(", "))]), titel("week_reason", &[(crate::id("f"), "4-3-3".into())])), Action::None);
        }
        self.ehrungen.push(Ehrung { season: self.world.season, art: "woche".into(), pid: Some(self.world.players[wer].id), club: Some(wc), titel: crate::tf!("ui.ehrung.spieler_der_woche", wn, cn) });
    }
    /// First day of a month: player and coach of the month in the own league (from the month's grades and table).
    pub fn ehrungen_monat(&mut self) {
        let (_, m, d) = crate::date::ymd(self.world.season, self.world.day);
        if d != 1 { return; }
        let l = self.league();
        let noten = std::mem::take(&mut self.world.noten_monat);
        let vormonat = crate::date::month_name(if m == 1 { 12 } else { m - 1 }).to_string();
        let lg = self.world.leagues[l].name.clone();
        let mut je: Vec<(usize, f32, u16, u16)> = vec![];
        for (p, r, g) in noten {
            let t = self.world.players[p].team;
            if t < 0 || self.world.clubs[t as usize].league != l { continue; }
            match je.iter_mut().find(|x| x.0 == p) { Some(x) => { x.1 += r; x.2 += 1; x.3 += g as u16; } None => je.push((p, r, 1, g as u16)) }
        }
        let gw = kf("goal_weight");
        let best = je.iter().filter(|x| x.2 as f32 >= kf("min_apps_month")).max_by(|a, b| (a.1 / a.2 as f32 + gw * a.3 as f32).partial_cmp(&(b.1 / b.2 as f32 + gw * b.3 as f32)).unwrap()).copied();
        if let Some((p, _, _, g)) = best {
            let (n, c) = (self.world.players[p].name.clone(), self.world.players[p].team.max(0) as usize);
            let cn = self.world.clubs[c].name.clone();
            let tt = titel("player_month", &[(crate::id("lg"), lg.clone()), (crate::id("m"), vormonat.clone()), (crate::id("name"), n), (crate::id("club"), cn)]);
            let grund = teile("m_spieler", &format!("{}|{m}", self.world.season), &[(crate::id("m"), vormonat.clone()), (crate::id("g"), if g > 0 { crate::tf!("ui.ehrung.dazu_treffer", g) } else { String::new() })]);
            self.ehrung(crate::id("monat"), Some(p), Some(c), tt, grund, kf("effects.morale_award"));
        }
        // coach of the month: most points per match over the month (at least two matches)
        let (von, bis) = (self.world.day.saturating_sub(31), self.world.day);
        let mut pkt: Vec<(usize, u32, u32)> = vec![];
        for f in self.world.leagues[l].fixtures.iter().filter(|f| f.hg >= 0 && f.day >= von && f.day < bis) {
            for (c, a, b) in [(f.h, f.hg, f.ag), (f.a, f.ag, f.hg)] {
                let p = if a > b { 3 } else if a == b { 1 } else { 0 };
                match pkt.iter_mut().find(|x| x.0 == c) { Some(x) => { x.1 += p; x.2 += 1; } None => pkt.push((c, p, 1)) }
            }
        }
        if let Some(&(c, p, n)) = pkt.iter().filter(|x| x.2 >= 2).max_by(|a, b| (a.1 as f32 / a.2 as f32).partial_cmp(&(b.1 as f32 / b.2 as f32)).unwrap()) {
            let mg = if c == self.own { self.coach.clone() } else { self.world.clubs[c].manager.clone() };
            let cn = self.world.clubs[c].name.clone();
            let tt = titel("coach_month", &[(crate::id("lg"), lg), (crate::id("m"), vormonat.clone()), (crate::id("name"), mg), (crate::id("club"), cn.clone())]);
            let grund = teile("m_trainer", &format!("{}|{m}|t", self.world.season), &[(crate::id("m"), vormonat), (crate::id("club"), cn), (crate::id("pts"), p.to_string()), (crate::id("n"), n.to_string())]);
            self.ehrung("trainer_monat", None, Some(c), tt, grund, 0.0);
        }
    }
    /// End of a season, besides the player of the season and the top scorer (ki.rs ehrungen_saison): young player and
    /// fair play of the own league, and the Ballon d'Or.
    pub fn ehrungen_saisonende(&mut self) {
        let l = self.league();
        let lg = self.world.leagues[l].name.clone();
        let spieler: Vec<usize> = self.world.leagues[l].teams.iter().flat_map(|&t| self.world.clubs[t].squad.clone()).collect();
        let min = kf("min_apps_season") as u16;
        let s = self.world.season.to_string();
        let wertung = |c: &Career, p: usize| { let pl = &c.world.players[p]; pl.avg_rating() + kf("goal_weight") * pl.goals as f32 / pl.apps.max(1) as f32 };
        if let Some(p) = spieler.iter().copied().filter(|&p| self.world.players[p].age <= 21 && self.world.players[p].rn >= min).max_by(|a, b| wertung(self, *a).partial_cmp(&wertung(self, *b)).unwrap()) {
            let pl = self.world.players[p].clone();
            let cn = self.world.clubs[pl.team.max(0) as usize].name.clone();
            let tt = titel("young_season", &[(crate::id("lg"), lg.clone()), (crate::id("name"), pl.name.clone()), (crate::id("club"), cn)]);
            let grund = teile("s_jug", &s, &[(crate::id("age"), pl.age.to_string()), (crate::id("n"), pl.apps.to_string())]);
            self.ehrung(crate::id("talent"), Some(p), Some(pl.team.max(0) as usize), tt, grund, kf("effects.morale_award"));
        }
        if let Some(&c) = self.world.leagues[l].teams.iter().min_by_key(|&&t| self.world.clubs[t].squad.iter().map(|&p| self.world.players[p].yellow as u32).sum::<u32>()) {
            let cn = self.world.clubs[c].name.clone();
            self.ehrung(crate::id("fair"), None, Some(c), titel("fair_club", &[(crate::id("club"), cn)]), titel("fair_club_why", &[]), 0.0);
        }
        // Ballon d'Or: the world's best by grade and goals (at least min_apps) plus titles of the season
        let season = self.world.season;
        let titel_pkt = |c: &Career, club: usize| -> f32 {
            c.world.titles.iter().filter(|t| t.season == season && t.club == club).map(|t| if c.world.leagues.iter().any(|x| x.name == t.comp) { kf("ballon.title.league") } else { kf("ballon.title.cup") }).sum()
        };
        let mut kand: Vec<(usize, f32)> = (0..self.world.players.len()).filter(|&p| self.world.players[p].team >= 0 && self.world.players[p].rn >= kf("ballon.min_apps") as u16 && self.world.players[p].ovr >= 78.0)
            .map(|p| { let pl = &self.world.players[p]; (p, pl.avg_rating() + kf("goal_weight") * pl.goals as f32 / pl.apps.max(1) as f32 + 0.1 * titel_pkt(self, pl.team as usize) + pl.ovr / 100.0) }).collect();
        kand.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        if let Some(&(p, _)) = kand.first() {
            let pl = self.world.players[p].clone();
            let cn = self.world.clubs[pl.team as usize].name.clone();
            let drei: Vec<String> = kand.iter().take(3).map(|x| self.world.players[x.0].name.clone()).collect();
            self.ehrung(crate::id("ballon"), Some(p), Some(pl.team as usize), crate::tf!("ui.ehrung.ballon_titel", pl.name, cn), crate::tf!("ui.ehrung.ballon_text", drei.join(", ")), kf("effects.morale_award"));
        }
        self.elf_der_woche.clear();
    }
    /// The honours of a player (for the profile): count per kind, newest first.
    pub fn ehrungen_von(&self, p: usize) -> Vec<&Ehrung> {
        let id = self.world.players[p].id;
        self.ehrungen.iter().rev().filter(|e| e.pid == Some(id) && e.art != "woche").collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn woche_monat_saison() {
        let w = crate::world::World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let own = w.clubs.iter().position(|c| c.name.contains("Arsenal")).unwrap_or(0); // roh
        let mut c = Career::new(w, own, "Test"); // roh
        for _ in 0..70 {
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            c.ultimatum = None;
            c.end_day().unwrap();
        }
        assert!(c.letzte_elf.len() == 11, "a team of the week: {:?}", c.letzte_elf.len());
        assert!(c.ehrungen.iter().any(|e| e.art == "woche"), "player of the week");
        assert!(c.ehrungen.iter().any(|e| e.art == "monat") && c.ehrungen.iter().any(|e| e.art == "trainer_monat"), "awards of the month");
        let t = c.ehrungen.iter().find(|e| e.art == "monat").unwrap();
        assert!(!t.titel.contains('{'), "{}", t.titel);
        // the Ballon d'Or needs 15 graded matches: play on to late in the season first
        for _ in 0..130 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } c.ultimatum = None; c.end_day().unwrap(); }
        c.ehrungen_saisonende();
        assert!(c.ehrungen.iter().any(|e| e.art == "ballon"), "Ballon d'Or");
        assert!(c.ehrungen.iter().any(|e| e.art == "fair"));
    }
}
