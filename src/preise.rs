//! Awards (Fabian 24.11.): the Ballon d'Or night in October with its nominees in September, The Best FIFA awards in
//! December with a shortlist in November and a vote of national coaches, captains, press and fans (5/3/1 points),
//! and every league's own awards at the season's end (assets/regeln/ligapreise.toml). All ranked on the season that
//! just ended; the pages show the live race of the running one with the same numbers.
use crate::game::*;
use crate::mensch::zufall;
use serde::{Deserialize, Serialize};

/// A given award: the season it is for, its key ("ballon", "best_spieler", "liga:<league>:<id>" …), the winner
/// (a player, or a club for coaches and teams) and the club, and the points of the vote.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Preis {
    pub season: u32,
    pub art: String,
    pub spieler: Option<usize>,
    pub club: usize,
    pub punkte: u32,
}

/// Ballon d'Or night: categories and how many are nominated (France Football 2026: 30, 10, 10, Müller by goals, 5, 5).
pub const BALLON: [(&str, usize); 6] = [("ballon", 30), ("kopa", 10), ("yashin", 10), ("mueller", 10), ("cruyff", 5), ("team", 5)]; // roh
/// The Best FIFA awards: player (shortlist of 11, three finalists), keeper, coach, FIFA 11 and fair play.
pub const BEST: [(&str, usize); 5] = [("best_spieler", 11), ("best_torwart", 5), ("best_trainer", 5), ("best_elf", 11), ("best_fairplay", 3)]; // roh

fn tag(k: &str) -> u32 { crate::regeln::f("ligapreise", &format!("termine.{k}")) as u32 }

/// Club of the coach: the manager's name (ours is the career's).
pub fn trainer_name(c: &Career, club: usize) -> String { if club == c.own { c.coach.clone() } else { c.world.clubs[club].manager.clone() } }

impl Career {
    /// The race for any award: players or clubs with their score, best first. Covers ballon, kopa, yashin, mueller
    /// (players, ehrungen::award_rennen), best_spieler and best_torwart (the same as ballon and yashin), cruyff and
    /// best_trainer (coaches: titles and how far above the expected place), team (titles and points per game),
    /// best_fairplay (fewest cards per game).
    pub fn rennen(&self, kat: &str, anteil: f32) -> Vec<(usize, f32)> {
        match kat {
            "best_spieler" => self.award_rennen(crate::id("ballon"), anteil),
            "best_torwart" => self.award_rennen(crate::id("yashin"), anteil),
            "cruyff" | "best_trainer" => self.vereins_rennen(0, None),
            "team" => self.vereins_rennen(1, None),
            "best_fairplay" => self.vereins_rennen(2, None),
            "best_elf" => self.beste_elf(anteil).into_iter().map(|p| (p, 0.0)).collect(),
            _ => self.award_rennen(kat, anteil),
        }
    }
    /// Clubs of the leagues with played matches (only in `liga` if given): 0 coach, 1 team, 2 fair play.
    pub fn vereins_rennen(&self, art: u8, liga: Option<usize>) -> Vec<(usize, f32)> {
        let season = self.world.season;
        let mut v = vec![];
        for l in 0..self.world.leagues.len() {
            if liga.is_some_and(|x| x != l) { continue; }
            let tab = self.world.table(l);
            let n = tab.len();
            if n < 4 || tab.iter().all(|r| r.p == 0) { continue; }
            // the place the squad's strength promised
            let mut nach_staerke: Vec<usize> = tab.iter().map(|r| r.club).collect();
            nach_staerke.sort_by(|a, b| self.world.clubs[*b].level.partial_cmp(&self.world.clubs[*a].level).unwrap());
            // a league without single figures has no cards per player: estimated (Fabian 24.11.)
            let detail = tab.iter().any(|r| self.world.clubs[r.club].squad.iter().any(|&p| self.world.players[p].in_liga(l).rn > 0));
            for (ist, r) in tab.iter().enumerate() {
                let titel = self.world.titles.iter().filter(|t| t.season == season && t.club == r.club).count() as f32;
                let erw = nach_staerke.iter().position(|&x| x == r.club).unwrap_or(ist) as f32;
                let ppg = r.pts as f32 / r.p.max(1) as f32;
                let s = match art {
                    0 => titel * 2.0 + (erw - ist as f32) / n as f32 * 4.0 + ppg,
                    1 => titel * 3.0 + ppg + self.world.clubs[r.club].level / 100.0,
                    _ => -self.gelbe_je_spiel(r.club, l, detail),
                };
                v.push((r.club, s));
            }
        }
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        v
    }
    /// Yellow cards per game of a club: counted where its league is played in detail, else estimated from a typical
    /// 1.8 a game, the coach's style (attacking sides foul less) and a fixed spread per club and season.
    pub fn gelbe_je_spiel(&self, club: usize, l: usize, detail: bool) -> f32 {
        let spiele = self.world.leagues[l].fixtures.iter().filter(|f| f.hg >= 0 && (f.h == club || f.a == club)).count().max(1) as f32;
        if detail { return self.world.clubs[club].squad.iter().map(|&p| self.world.players[p].yellow as f32).sum::<f32>() / spiele; }
        let streu = zufall(&[crate::id("gelbe"), &self.world.clubs[club].id.to_string(), &self.world.season.to_string()]) - 0.5;
        (1.8 - 0.15 * self.world.clubs[club].style as f32 + streu * 0.9).max(0.6)
    }
    /// FIFA 11 in a 4-3-3: keeper, two centre-backs, two full-backs, three midfielders, three forwards.
    pub fn beste_elf(&self, anteil: f32) -> Vec<usize> {
        let alle = self.award_rennen(crate::id("best_alle"), anteil);
        let mut elf = vec![];
        for (gruppen, n) in [(&[0usize][..], 1), (&[1], 2), (&[2], 2), (&[3], 3), (&[4, 5], 3)] {
            elf.extend(alle.iter().filter(|x| gruppen.contains(&crate::kimarkt::gruppe(&self.world.players[x.0].pos))).take(n).map(|x| x.0));
        }
        elf
    }
    /// The voters of the world: one per nation of the top `n` by FIFA points in each group; fans add `fans` voters.
    /// A captain is the best player of his national squad and never votes for himself.
    pub fn waehler_welt(&self, gruppen: &[&'static str], n: usize, fans: usize) -> Vec<Waehler> {
        let mut nat: Vec<u16> = self.world.nationalteams.iter().map(|t| t.nat).filter(|&x| x > 0).collect();
        nat.sort_by(|a, b| self.world.fifa_punkte(*b).partial_cmp(&self.world.fifa_punkte(*a)).unwrap());
        nat.truncate(n);
        let mut v = vec![];
        for &g in gruppen {
            for &x in &nat {
                let selbst = (g == "kapitaen").then(|| self.world.nationalteams.iter().find(|t| t.nat == x).and_then(|t| t.kader.iter().copied().max_by(|a, b| self.world.players[*a].ovr.partial_cmp(&self.world.players[*b].ovr).unwrap()))).flatten(); // roh
                v.push(Waehler { gruppe: g, nat: x, selbst });
            }
        }
        for i in 0..fans { v.push(Waehler { gruppe: "fans", nat: (i as u16) % 7, selbst: None }); } // roh
        // without national teams (small test worlds) a plain panel of each group
        if nat.is_empty() { for &g in gruppen { for _ in 0..40 { v.push(Waehler { gruppe: g, nat: 0, selbst: None }); } } }
        v
    }
    /// A vote (Fabian 24.11.: by the regulations, not by grades): every voter sees the candidates' merit through his
    /// own eyes (his group's spread), likes a countryman, never names himself, and gives `punkte` to his top places.
    /// Points per candidate, best first.
    pub fn abstimmen(&self, kand: &[(usize, f32)], salz: &str, punkte: &[u32], waehler: &[Waehler], spieler: bool) -> Vec<(usize, u32)> {
        if kand.is_empty() { return vec![]; }
        let spanne = (kand[0].1 - kand[kand.len() - 1].1).abs().max(0.1);
        let mut erg: Vec<(usize, u32)> = kand.iter().map(|x| (x.0, 0)).collect();
        for (wi, w) in waehler.iter().enumerate() {
            let streu = match w.gruppe { "presse" => 0.2, "trainer" => 0.25, "kapitaen" | "spieler" => 0.3, _ => 0.5 }; // roh
            let mut sicht: Vec<(usize, f32)> = kand.iter().enumerate().filter(|(_, x)| Some(x.0) != w.selbst).map(|(i, x)| {
                let land = if spieler { self.world.players[x.0].nat } else { self.world.land_nat(self.world.clubs[x.0].league) };
                let heim = if w.nat > 0 && land == w.nat { 0.15 * spanne } else { 0.0 };
                (i, x.1 + heim + (zufall(&[salz, w.gruppe, &wi.to_string(), &x.0.to_string()]) - 0.5) * 2.0 * streu * spanne)
            }).collect();
            sicht.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            for (rang, pk) in punkte.iter().enumerate() { if let Some(s) = sicht.get(rang) { erg[s.0].1 += pk; } }
        }
        erg.sort_by_key(|x| std::cmp::Reverse(x.1));
        erg
    }
    /// The vote of an award night: Ballon d'Or by one journalist per nation (top 100, ten places 15…1), The Best by
    /// national coaches, captains, press and fans (5/3/1).
    pub fn preis_abstimmung(&self, kat: &str, kand: &[(usize, f32)], season: u32) -> Vec<(usize, u32)> {
        let salz = format!("{season}{kat}");
        if BALLON.iter().any(|x| x.0 == kat) {
            let w = self.waehler_welt(&["presse"], 100, 0); // roh
            self.abstimmen(kand, &salz, &[15, 12, 10, 8, 7, 5, 4, 3, 2, 1], &w, ist_spieler_preis(kat))
        } else {
            let w = self.waehler_welt(&["trainer", "kapitaen", "presse"], 100, 100); // roh
            self.abstimmen(kand, &salz, &[5, 3, 1], &w, ist_spieler_preis(kat))
        }
    }
    /// A few voters' first choices with the reason in words, for the result page: (voter, his pick, reason key).
    pub fn wahl_beispiele(&self, kat: &str, kand: &[(usize, f32)], season: u32, n: usize) -> Vec<(String, usize, &'static str)> {
        let spieler = ist_spieler_preis(kat);
        let gruppen: &[&'static str] = if BALLON.iter().any(|x| x.0 == kat) { &["presse"] } else { &["trainer", "kapitaen", "presse"] }; // roh
        let w = self.waehler_welt(gruppen, 100, 0);
        let mut v = vec![];
        for (k, x) in w.iter().enumerate().filter(|(_, x)| x.nat > 0).step_by((w.len() / n.max(1)).max(1)).take(n) {
            let erg = self.abstimmen(kand, &format!("{season}{kat}"), &[1], std::slice::from_ref(x), spieler);
            let Some(&(p, _)) = erg.iter().find(|e| e.1 > 0) else { continue };
            let land = if spieler { self.world.players[p].nat } else { self.world.land_nat(self.world.clubs[p].league) };
            let club = if spieler { self.world.players[p].team.max(0) as usize } else { p };
            let grund = if land == x.nat { "heimat" } else if self.world.titles.iter().any(|t| t.club == club && t.season + 1 >= season) { "titel" } else if spieler && self.turnier_glanz(p) > 0.0 { "turnier" } else if spieler && crate::kimarkt::gruppe(&self.world.players[p].pos) >= 4 { "tore" } else { "zahlen" }; // roh
            let wer = crate::tf!(&format!("ui.preis.waehler.{}", x.gruppe), crate::nationen::name(&self.world, x.nat));
            let _ = k;
            v.push((wer, p, grund));
        }
        v
    }
    /// Glory from a national tournament of this or the last year: the winners' players more, the runners-up less.
    pub fn turnier_glanz(&self, p: usize) -> f32 {
        let n = self.world.players[p].nat;
        let jahr = crate::date::START_YEAR + self.world.season as i32;
        self.world.turniere.iter().filter(|t| t.jahr == jahr || t.jahr == jahr - 1).map(|t| if t.sieger == n { 0.3 } else if t.zweiter == n { 0.12 } else { 0.0 }).fold(0.0, f32::max)
    }
    /// Starting values for the season before the career (Fabian 24.11.: the first award night needs a past season):
    /// the estimated last season of each player (role, strength, club), his club's standing and tournament glory.
    pub fn start_rennen(&self, kat: &str) -> Vec<(usize, f32)> {
        if !ist_spieler_preis(kat) {
            let mut v: Vec<(usize, f32)> = (0..self.world.clubs.len()).filter(|&c| self.world.clubs[c].league < self.world.leagues.len()).map(|c| {
                let cl = &self.world.clubs[c];
                (c, cl.level / 10.0 + cl.rep / 20.0 + (zufall(&["start_verein", &cl.id.to_string()]) - 0.5)) // roh
            }).collect();
            v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            return v;
        }
        let mut v: Vec<(usize, f32)> = (0..self.world.players.len()).filter_map(|p| {
            let pl = &self.world.players[p];
            if pl.team < 0 || pl.youth != 0 { return None; }
            let passt = match kat { "ballon" | "best_spieler" => pl.ovr >= 80.0, "kopa" => pl.age <= 21 && pl.ovr >= 68.0, "yashin" | "best_torwart" => pl.pos == "GK" && pl.ovr >= 78.0, "mueller" => !matches!(pl.pos.as_str(), "GK" | "CB") && pl.ovr >= 74.0, _ => pl.ovr >= 78.0 }; // roh
            if !passt { return None; }
            let s = crate::statistik::vorsaison_geschaetzt(&self.world, p);
            let apps = crate::statistik::get(&s, crate::statistik::SPIELE).max(1) as f32;
            let tore = crate::statistik::get(&s, crate::statistik::TORE) as f32;
            let cl = &self.world.clubs[pl.team as usize];
            let streu = zufall(&["start_note", &pl.id.to_string()]) - 0.5;
            let note = 6.3 + (pl.ovr - 72.0) * 0.04 + (cl.level - 75.0) * 0.02 + streu * 0.35;
            let glanz = ((cl.rep - 70.0) / 40.0).clamp(0.0, 1.0) * 0.3 + self.turnier_glanz(p);
            Some((p, match kat { "mueller" => tore + note / 100.0, "yashin" | "best_torwart" => note + pl.ovr / 100.0, _ => note + 0.8 * tore / apps + glanz + pl.ovr / 100.0 })) // roh
        }).collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        v
    }
    fn preis_geben(&mut self, art: &str, wer: usize, spieler: bool, punkte: u32, season: u32) {
        let club = if spieler { self.world.players[wer].team.max(0) as usize } else { wer };
        self.preise.push(Preis { season, art: art.into(), spieler: spieler.then_some(wer), club, punkte });
    }
    /// Season end: the nominee lists of the coming Ballon d'Or night and The Best, from the season just played.
    pub fn preise_saisonende(&mut self) {
        let season = self.world.season;
        // the Ballon d'Or of next autumn is for this season: its candidates and merit now, before the figures reset
        self.preis_kand.retain(|x| !BALLON.iter().any(|b| b.0 == x.0));
        for (kat, n) in BALLON.iter() {
            let mut k = self.rennen(kat, 1.0);
            k.truncate(*n);
            if !k.is_empty() { self.preis_kand.push((kat.to_string(), k)); }
        }
        self.preis_kand_saison = season + 1;
        self.liga_preise(season);
    }
    /// The Best covers the calendar year: half the season before (its Ballon merit, or the starting values) and half
    /// the running autumn; the shortlist on its day.
    fn best_kandidaten(&mut self) {
        for (kat, n) in BEST.iter() {
            let mut k = self.best_rennen(kat);
            k.truncate(*n);
            self.preis_kand.retain(|x| x.0 != *kat);
            if !k.is_empty() { self.preis_kand.push((kat.to_string(), k)); }
        }
    }
    /// The Best's merit over the calendar year: half the season before, half the running autumn.
    pub fn best_rennen(&self, kat: &str) -> Vec<(usize, f32)> {
        let anteil = (self.world.day as f32 / crate::world::LEAGUE_END as f32).clamp(0.1, 1.0);
        let jetzt = self.rennen(kat, anteil);
        let vorher_kat = match kat { "best_spieler" => "ballon", "best_torwart" => "yashin", "best_trainer" => "cruyff", _ => "" }; // roh
        if vorher_kat.is_empty() { return jetzt; }
        let vorher: std::collections::HashMap<usize, f32> = self.preis_kand.iter().find(|x| x.0 == vorher_kat).map(|x| x.1.iter().copied().collect()).unwrap_or_else(|| self.start_rennen(kat).into_iter().take(200).collect());
        // who shone only in one half gets the weakest mark of the other half for it
        let boden = vorher.values().copied().fold(f32::MAX, f32::min);
        let boden_jetzt = jetzt.iter().map(|x| x.1).fold(boden, f32::min);
        let mut k: Vec<(usize, f32)> = jetzt.iter().map(|&(p, x)| (p, 0.5 * x + 0.5 * vorher.get(&p).copied().unwrap_or(boden.min(x)))).collect();
        k.extend(vorher.iter().filter(|(p, _)| !jetzt.iter().any(|x| x.0 == **p)).map(|(&p, &x)| (p, 0.5 * x + 0.5 * boden_jetzt.min(x))));
        k.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        k
    }
    /// The candidates of the next award night as the bookmakers see them today: the nominees once they are out, the
    /// starting values before the first night, else the running season (Ballon) or the calendar year (The Best).
    pub fn vorschau(&self, kat: &str) -> Vec<(usize, f32)> {
        let d = self.world.day;
        if BEST.iter().any(|x| x.0 == kat) {
            if d >= tag("best_shortlist") && d <= tag("best_gala") { if let Some(k) = self.preis_kand.iter().find(|x| x.0 == kat) { return k.1.clone(); } }
            return self.best_rennen(kat);
        }
        if d <= tag("ballon_gala") {
            if self.preis_kand_saison == self.world.season { if let Some(k) = self.preis_kand.iter().find(|x| x.0 == kat) { return k.1.clone(); } }
            if self.world.season == 0 || self.preis_kand_saison < self.world.season { return self.start_rennen(kat); }
        }
        let anteil = (d as f32 / crate::world::LEAGUE_END as f32).clamp(0.1, 1.0);
        self.rennen(kat, anteil)
    }
    /// Every league's own awards (ligapreise.toml), given at the season's end; ours as a message, a win of ours as a moment.
    pub fn liga_preise(&mut self, season: u32) {
        for l in 0..self.world.leagues.len() {
            if !self.world.leagues[l].teams.iter().any(|&t| self.world.clubs[t].squad.iter().any(|&p| self.world.players[p].in_liga(l).rn > 0)) { continue; }
            let mut zeilen = vec![];
            for a in liga_preisliste(&liga_land(&self.world, l)) {
                // the top scorer by goals; the rest voted by the league's journalists, players and coaches (Fabian 24.11.)
                let rennen = self.liga_rennen(l, &a, 1.0);
                let Some(&(_, _, ist_spieler)) = rennen.first() else { continue };
                let wer = if a.art == "tore" { rennen[0].0 } else {
                    let kand: Vec<(usize, f32)> = rennen.iter().take(10).map(|x| (x.0, x.1)).collect();
                    let nat = self.world.land_nat(l);
                    let gruppe: &'static str = match a.stimme.as_str() { "profis" => "spieler", "presse" => "presse", _ => "trainer" }; // roh
                    let w: Vec<Waehler> = ["presse", "spieler", gruppe].iter().flat_map(|&g| (0..20).map(move |_| Waehler { gruppe: g, nat, selbst: None })).collect(); // roh
                    self.abstimmen(&kand, &format!("{season}liga{l}{}", a.id), &[5, 3, 1], &w, ist_spieler).first().map(|x| x.0).unwrap_or(rennen[0].0)
                };
                let art = format!("liga:{l}:{}", a.id);
                self.preis_geben(&art, wer, ist_spieler, 0, season);
                if l != self.league() { continue; }
                let n = if ist_spieler { self.world.players[wer].name.clone() } else { trainer_name(self, wer) };
                let club = if ist_spieler { self.world.players[wer].team.max(0) as usize } else { wer };
                zeilen.push(format!("{}: {n} ({})", a.name(), self.world.clubs[club].name));
                if club == self.own { self.feiern.push(crate::meilensteine::Feier { art: "gala".into(), titel: a.name(), text: crate::tf!("ui.preis.liga_sieg", n, a.name()), spieler: ist_spieler.then_some(wer), club: Some(club), stimmen: vec![] }); } // roh
            }
            if !zeilen.is_empty() {
                let ln = self.world.leagues[l].name.clone();
                self.msg(0, &ln, &crate::tf!("ui.preis.liga_titel", ln.clone()), &zeilen.join(crate::id("\n")), Action::None);
            }
        }
    }
    /// The race for one league award in league `l`, best first: (player or club, score, whether it is a player).
    /// Only what was played in this league counts (its own figures per player).
    pub fn liga_rennen(&self, l: usize, a: &LigaPreis, anteil: f32) -> Vec<(usize, f32, bool)> {
        if a.art == "trainer" { return self.vereins_rennen(0, Some(l)).into_iter().map(|x| (x.0, x.1, false)).collect(); }
        let land = self.world.land_nat(l);
        // a share of the league's games; the top scorer needs none, the young ones half
        let min = (crate::regeln::f("ligapreise", "min_spiele_anteil") * anteil * self.world.leagues[l].teams.len().saturating_sub(1) as f32 * 2.0) as u16;
        let min = match a.art.as_str() { "tore" => 1, "jung" => (min / 2).max(1), _ => min.max(1) }; // roh
        let mut v: Vec<(usize, f32, bool)> = self.world.leagues[l].teams.iter().flat_map(|&t| self.world.clubs[t].squad.iter().copied()).filter_map(|p| {
            let pl = &self.world.players[p];
            let ls = pl.in_liga(l);
            if ls.rn < min || (a.eigene && pl.nat != land) { return None; }
            let tore = crate::regeln::f("ligapreise", "tor_gewicht") * ls.goals as f32 / ls.apps.max(1) as f32;
            let w = match a.art.as_str() {
                "tore" if ls.goals > 0 => ls.goals as f32 + ls.note() / 100.0, // roh
                "jung" if pl.age <= a.alter => ls.note() + tore + pl.pot / 200.0, // roh
                "torwart" if pl.pos == "GK" => ls.note(), // roh
                "spieler" => ls.note() + tore + pl.ovr / 100.0 + if a.stimme == "presse" { ls.goals as f32 * 0.01 } else { 0.0 }, // roh
                _ => return None,
            };
            Some((p, w, true))
        }).collect();
        v.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
        v
    }
    /// The award calendar, daily: nominees (8 Sep), Ballon d'Or night (26 Oct), The Best shortlist (6 Nov) and night (16 Dec).
    pub fn preise_tag(&mut self) {
        let d = self.world.day;
        let s = self.world.season;
        // no season played yet (the first autumn of a career): the nominees from the starting values
        if d == tag("ballon_nominierung") {
            if self.preis_kand_saison != s || !self.preis_kand.iter().any(|x| x.0 == "ballon") { // roh
                self.preis_kand.retain(|x| !BALLON.iter().any(|b| b.0 == x.0));
                for (kat, n) in BALLON.iter() { let mut k = self.start_rennen(kat); k.truncate(*n); if !k.is_empty() { self.preis_kand.push((kat.to_string(), k)); } }
                self.preis_kand_saison = s;
            }
            self.nominierungen(&BALLON.map(|x| x.0), "ui.preis.ballon_nominiert");
        }
        if d == tag("best_shortlist") { self.best_kandidaten(); self.nominierungen(&["best_spieler"], "ui.preis.best_shortlist"); } // roh
        if d == tag("ballon_gala") && self.preis_kand_saison == s { self.preisnacht(&BALLON.map(|x| x.0), s, "ui.preis.ballon_nacht"); }
        if d == tag("best_gala") && self.preis_kand.iter().any(|x| x.0 == "best_spieler") { self.preisnacht(&BEST.map(|x| x.0), s, "ui.preis.best_nacht"); } // roh
    }
    fn nominierungen(&mut self, kats: &[&str], titel: &str) {
        let mut eigene = vec![];
        for kat in kats {
            let Some((_, k)) = self.preis_kand.iter().find(|x| x.0 == *kat) else { continue };
            for &(w, _) in k {
                let club = if ist_spieler_preis(kat) { self.world.players[w].team } else { w as i32 };
                if club == self.own as i32 { eigene.push(format!("{} ({})", if ist_spieler_preis(kat) { self.world.players[w].name.clone() } else { self.world.clubs[w].name.clone() }, crate::inhalt::t(&format!("ui.gala.{kat}")))); }
            }
        }
        let text = if eigene.is_empty() { crate::inhalt::t("ui.preis.keine_eigenen").to_string() } else { crate::tf!("ui.preis.eigene_nominiert", eigene.join(", ")) };
        self.news(crate::inhalt::t(titel), &text);
        if !eigene.is_empty() { self.msg(0, crate::inhalt::t("ui.preis.absender"), crate::inhalt::t(titel), &text, Action::None); }
    }
    fn preisnacht(&mut self, kats: &[&str], season: u32, titel: &str) {
        for kat in kats {
            let Some((_, k)) = self.preis_kand.iter().find(|x| x.0 == *kat).cloned() else { continue };
            let spieler = ist_spieler_preis(kat);
            if *kat == "best_elf" { for &(p, _) in &k { self.preis_geben(kat, p, true, 0, season); } continue; } // roh
            // goals decide the Müller trophy, fewest cards the fair play award; the rest is voted
            let erg = if *kat == "mueller" || *kat == "best_fairplay" { k.iter().map(|x| (x.0, 0)).collect() } else { self.preis_abstimmung(kat, &k, season) }; // roh
            let Some(&(w, pk)) = erg.first() else { continue };
            self.preis_geben(kat, w, spieler, pk, season);
            let wer = if spieler { self.world.players[w].name.clone() } else if *kat == "team" || *kat == "best_fairplay" { self.world.clubs[w].name.clone() } else { trainer_name(self, w) }; // roh
            let club = if spieler { self.world.players[w].team.max(0) as usize } else { w };
            let name = crate::inhalt::t(&format!("ui.gala.{kat}")).to_string();
            let text = crate::tf!("ui.gala.text", wer.clone(), self.world.clubs[club].name.clone(), name.clone());
            self.news(&crate::tf!("ui.gala.news", name.clone(), wer), &text);
            if club == self.own || *kat == "ballon" || *kat == "best_spieler" { self.feiern.push(crate::meilensteine::Feier { art: "gala".into(), titel: name, text, spieler: spieler.then_some(w), club: Some(club), stimmen: vec![] }); } // roh
        }
        self.msg(0, crate::inhalt::t("ui.preis.absender"), crate::inhalt::t(titel), crate::inhalt::t("ui.preis.nacht_text"), Action::None);
    }
    /// The winners of an award by season, newest first.
    pub fn preistraeger(&self, art: &str) -> Vec<&Preis> { let mut v: Vec<&Preis> = self.preise.iter().filter(|p| p.art == art).collect(); v.sort_by_key(|p| std::cmp::Reverse(p.season)); v }
}

/// The UEFA five-year ranking of the countries (Fabian 24.11.): per country the points of each of the last five
/// seasons (a season's points divided by the country's clubs in Europe that season), the starting value fading by a
/// fifth with every season played. Best first: (country, five seasons oldest first, total).
pub fn uefa_laender(w: &crate::world::World) -> Vec<(String, Vec<f32>, f32)> {
    let gespielt = (w.season as f32 + 1.0).min(5.0);
    let mut laender: Vec<String> = crate::regeln::wert("uefa", "land").as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default();
    for x in &w.coef { if !laender.contains(&x.0) { laender.push(x.0.clone()); } }
    let mut v: Vec<(String, Vec<f32>, f32)> = laender.into_iter().map(|land| {
        let start = crate::regeln::wert_oder("uefa", &format!("land.{land}")).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
        let je: Vec<f32> = (0..5).map(|k| {
            let s = w.season as i64 - 4 + k as i64;
            if s < 0 { return start / 5.0; }
            let pts: f32 = w.coef.iter().filter(|x| x.0 == land && x.1 as i64 == s).map(|x| x.2).sum();
            let n = w.club_coef.iter().filter(|x| x.1 as i64 == s && crate::cups::country(&w.leagues[w.clubs[x.0].league]).as_deref() == Some(land.as_str())).count();
            if n > 0 { pts / n as f32 } else { 0.0 }
        }).collect();
        let _ = gespielt;
        let sum = je.iter().sum();
        (land, je, sum)
    }).collect();
    v.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    v
}
/// The UEFA club ranking: the clubs' points of the last five seasons plus a starting value from reputation for the
/// seasons before the career. Best first: (club, total).
pub fn uefa_vereine(w: &crate::world::World) -> Vec<(usize, f32)> {
    let (ab, je) = (crate::regeln::f("uefa", "verein.ab"), crate::regeln::f("uefa", "verein.je_punkt"));
    let vor = (4.0 - w.season as f32).max(0.0) / 5.0;
    let laender = crate::regeln::wert("uefa", "land");
    let mut v: Vec<(usize, f32)> = (0..w.clubs.len()).filter_map(|c| {
        let l = w.clubs[c].league;
        let land = crate::cups::country(&w.leagues[l])?;
        if laender.get(&land).is_none() || w.erste_von(c).is_some() { return None; }
        let jetzt: f32 = w.club_coef.iter().filter(|x| x.0 == c).map(|x| x.2).sum();
        let start = ((w.clubs[c].rep - ab).max(0.0) * je) * vor;
        let t = jetzt + start;
        (t > 0.0).then_some((c, t))
    }).collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    v
}
/// A voter: his group (presse, trainer, kapitaen, spieler, fans), his country and, for a captain, himself.
#[derive(Clone, Debug)]
pub struct Waehler { pub gruppe: &'static str, pub nat: u16, pub selbst: Option<usize> }

/// Win odds of the first candidates (decimal, with the bookmaker's margin): the merit through a soft maximum.
pub fn quoten(k: &[(usize, f32)]) -> Vec<f32> {
    if k.is_empty() { return vec![]; }
    let n = k.len().min(20);
    let spanne = (k[0].1 - k[n - 1].1).abs().max(0.05);
    let e: Vec<f32> = k[..n].iter().map(|x| ((x.1 - k[0].1) / (spanne / 4.0)).exp()).collect();
    let sum: f32 = e.iter().sum();
    e.iter().map(|x| (0.9 / (x / sum)).clamp(1.05, 501.0)).collect()
}
/// The period an award is for, in words: the season before the night (Ballon d'Or), the calendar year (The Best), the
/// season played (league awards).
pub fn zeitraum(kat: &str, season: u32) -> String {
    let j = crate::date::START_YEAR + season as i32;
    if BALLON.iter().any(|x| x.0 == kat) { format!("{}/{:02}", j - 1, j % 100) } else if BEST.iter().any(|x| x.0 == kat) { j.to_string() } else { crate::date::season_label(season) }
}
/// Whether the award goes to a player (else to a club or its coach).
pub fn ist_spieler_preis(kat: &str) -> bool { !matches!(kat, "cruyff" | "team" | "best_trainer" | "best_fairplay") }

/// One league award of ligapreise.toml.
#[derive(Clone, Debug)]
pub struct LigaPreis { pub id: String, pub art: String, pub alter: u8, pub eigene: bool, pub stimme: String, pub name: Option<String> }
impl LigaPreis {
    /// The award's real name, or the generic one in the person's language.
    pub fn name(&self) -> String { self.name.clone().unwrap_or_else(|| crate::inhalt::t(&format!("ui.ligapreis.{}", self.art)).to_string()) }
}
/// The country of league `l` by its rules (German names, as in ligapreise.toml), else the league's own field.
pub fn liga_land(w: &crate::world::World, l: usize) -> String { w.regel_land(l).map(|x| x.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| w.leagues[l].land.clone()) }
/// The awards of a country's league (the standard set where the country has none listed).
pub fn liga_preisliste(land: &str) -> Vec<LigaPreis> {
    let w = crate::regeln::wert_oder("ligapreise", &format!("land.{land}")).filter(|v| v.is_array()).unwrap_or_else(|| crate::regeln::wert("ligapreise", "standard")); // roh
    w.as_array().map(|a| a.iter().map(|x| LigaPreis {
        id: x["id"].as_str().unwrap_or_default().into(),
        art: x["art"].as_str().unwrap_or_default().into(),
        alter: x["alter"].as_u64().unwrap_or(21) as u8,
        eigene: x["eigene"].as_bool().unwrap_or(false),
        stimme: x["stimme"].as_str().unwrap_or_default().into(),
        name: x["name"].as_str().map(|s| s.to_string()),
    }).collect()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Fabian 24.11.: the five-year ranking starts from the estimated past and adds what is played.
    #[test]
    fn uefa_wertung() {
        let mut w = crate::world::testwelt(2);
        let vorher = uefa_laender(&w);
        assert!(vorher.first().is_some_and(|x| x.0 == "England"), "{:?}", vorher.first()); // roh
        let c = (0..w.clubs.len()).find(|&k| crate::cups::country(&w.leagues[w.clubs[k].league]).as_deref() == Some("Deutschland")).unwrap(); // roh
        w.coef.push(("Deutschland".into(), 0, 10.0)); // roh
        w.club_coef.push((c, 0, 10.0));
        let de = |v: &Vec<(String, Vec<f32>, f32)>| v.iter().find(|x| x.0 == "Deutschland").unwrap().2; // roh
        assert!(de(&uefa_laender(&w)) > de(&vorher));
        assert!(uefa_vereine(&w).iter().any(|x| x.0 == c));
    }
    /// Fabian 24.11.: nominees at the season's end, the vote at the gala, league awards with their real names.
    #[test]
    fn preise_jahr() {
        let mut c = Career::new(crate::world::testwelt(5), 2, "Test");
        let sq = c.squad();
        let l = c.league() as u16;
        for (i, &p) in sq.iter().enumerate() {
            let pl = &mut c.world.players[p];
            pl.rn = 30; pl.apps = 30; pl.rsum = 30.0 * (6.5 + i as f32 * 0.05); pl.goals = i as u16;
            pl.profil.ligen.push(crate::world::LigaStat { l, apps: 30, goals: i as u16, rsum: pl.rsum, rn: 30, ..Default::default() });
        }
        c.preise_saisonende();
        assert!(c.preis_kand.iter().any(|x| x.0 == "ballon" && !x.1.is_empty()), "next autumn's Ballon d'Or nominees from this season"); // roh
        let kand = c.preis_kand.iter().find(|x| x.0 == "ballon").unwrap().1.clone(); // roh
        let w = c.waehler_welt(&["trainer", "kapitaen", "presse"], 100, 100); // roh
        let erg = c.abstimmen(&kand, "t", &[5, 3, 1], &w, true);
        assert_eq!(erg.iter().map(|x| x.1).sum::<u32>(), w.len() as u32 * 9, "every voter gives 5+3+1");
        // a countryman gets a little more from his own journalist
        let p0 = kand[kand.len() / 2].0;
        let heim = vec![Waehler { gruppe: "presse", nat: c.world.players[p0].nat.max(1), selbst: None }; 50]; // roh
        let fremd = vec![Waehler { gruppe: "presse", nat: 9999, selbst: None }; 50]; // roh
        if c.world.players[p0].nat > 0 { assert!(c.abstimmen(&kand, "h", &[1], &heim, true).iter().find(|x| x.0 == p0).unwrap().1 >= c.abstimmen(&kand, "h", &[1], &fremd, true).iter().find(|x| x.0 == p0).unwrap().1); }
        // a captain never names himself
        let selbst = vec![Waehler { gruppe: "kapitaen", nat: 0, selbst: Some(kand[0].0) }; 20]; // roh
        assert_eq!(c.abstimmen(&kand, "s", &[5, 3, 1], &selbst, true).iter().find(|x| x.0 == kand[0].0).unwrap().1, 0);
        c.world.season += 1;
        c.world.day = tag("ballon_gala");
        c.preise_tag();
        assert!(c.preistraeger("ballon").first().is_some_and(|p| p.season == c.world.season)); // roh
        // a new career has a first award night too: nominees from the starting values
        let mut d = Career::new(crate::world::testwelt(5), 2, "Test");
        d.world.day = tag("ballon_nominierung");
        d.preise_tag();
        assert!(d.preis_kand.iter().any(|x| x.0 == "ballon" && !x.1.is_empty()), "first autumn: nominees from the season before"); // roh
        d.world.day = tag("best_shortlist");
        d.preise_tag();
        assert!(d.preis_kand.iter().any(|x| x.0 == "best_spieler")); // roh
        assert!(quoten(&d.vorschau("ballon")).windows(2).all(|q| q[0] <= q[1]), "the favourite has the shortest odds"); // roh
        // the league's own awards: a real name for a known country, the generic set elsewhere
        assert!(liga_preisliste("Deutschland").iter().any(|a| a.name() == "Torjägerkanone"));
        assert!(liga_preisliste("Nirgendwo").iter().all(|a| a.name.is_none()) && !liga_preisliste("Nirgendwo").is_empty());
        assert!(c.preise.iter().any(|p| p.art.starts_with(&format!("liga:{}:", c.league()))), "the own league gave its awards");
    }
}
