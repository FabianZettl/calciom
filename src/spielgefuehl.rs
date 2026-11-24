//! Spielgefühl, Teil 4 (docs/intern/SPIELGEFUEHL.md, Maßnahmen 1, 2 und 9): the one thing to do now with its deadline
//! and a reason, the season as a journey with today's marker and the season goal as a bar, and the guided first week
//! (five steps with a small reward, can be switched off). Numbers in assets/regeln/spielgefuehl.toml.
use crate::game::*;
use crate::pages::Page;
use crate::world::MatchResult;

fn r(k: &str) -> f32 { crate::regeln::f("spielgefuehl", k) }
fn t(k: &str) -> &'static str { crate::inhalt::t(k) }

/// The most important thing now: what, why (one sentence), until when, the button and where it leads. `vorrang`: it can't
/// wait, so the „Weiter“ button leads there instead of starting the next day.
pub struct WasJetzt { pub titel: String, pub grund: String, pub frist: String, pub knopf: String, pub ziel: Option<Page>, pub msg: Option<usize>, pub vorrang: bool }

/// One weekly point of the club's history (Teil 3, „Verlauf statt Zahl“): table place, strength of the eleven, team
/// spirit, mean morale of the squad, balance.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct VerlaufPunkt { pub abs: u32, pub platz: u8, pub staerke: f32, pub teamgeist: f32, pub moral: f32, pub kasse: f64 }

/// One own match for the season and all-time figures (Teil 3): when, competition, opponent, home or away, score, derby.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct SpielZeile { pub abs: u32, pub season: u32, pub wettbewerb: String, pub gegner: usize, pub heim: bool, pub tore: u8, pub gegentore: u8, pub derby: bool }
/// A player's season in one competition (own players): matches, minutes, goals, assists, cards, sum and count of grades.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct SpielerWb { pub pid: u32, pub wettbewerb: String, pub spiele: u16, pub minuten: u16, pub tore: u16, pub vorlagen: u16, pub gelb: u8, pub rot: u8, pub notensumme: f32, pub noten: u16 }

pub const SCHRITTE: [&str; 5] = ["ui.woche.s0", "ui.woche.s1", "ui.woche.s2", "ui.woche.s3", "ui.woche.s4"];

impl Career {
    pub fn was_jetzt(&self) -> WasJetzt {
        let abs = self.abs();
        let w = |titel: String, grund: String, frist: String, knopf: &str, ziel: Page, msg: Option<usize>, vorrang: bool| WasJetzt { titel, grund, frist, knopf: knopf.into(), ziel: Some(ziel), msg, vorrang };
        let frist = |f: u32| if f <= abs { t("ui.wj.frist.heute").to_string() } else { crate::tf!("ui.wj.frist.tage", f - abs) };
        if self.fired { return w(t("ui.wj.entlassen.titel").into(), t("ui.wj.entlassen.grund").into(), String::new(), t("ui.wj.entlassen.knopf"), Page::Posteingang, None, true); }
        if self.press.is_some() { return w(t("ui.wj.presse.titel").into(), t("ui.wj.presse.grund").into(), t("ui.wj.frist.heute").into(), t("ui.wj.presse.knopf"), Page::Presse, None, true); }
        // open decisions by deadline: a Muss due today can't wait
        let offen = |pflicht: u8| self.inbox.iter().enumerate().filter(|(_, m)| !m.done && m.pflicht == pflicht && m.action != Action::None && !matches!(m.action, Action::Report { .. })).min_by_key(|(_, m)| m.frist);
        if let Some((i, m)) = offen(2).filter(|(_, m)| m.frist <= abs) { return w(m.title.clone(), t("ui.wj.muss.grund").into(), frist(m.frist), t("ui.wj.muss.knopf"), Page::Posteingang, Some(i), true); }
        if let Some(fi) = self.today_match() {
            let f = self.fixture(fi);
            let g = if f.h == self.own { f.a } else { f.h };
            return w(crate::tf!("ui.wj.spiel.titel", self.world.clubs[g].name), t("ui.wj.spiel.grund").into(), t("ui.wj.frist.heute").into(), t("ui.wj.spiel.knopf"), Page::Spieltag, None, false);
        }
        if let Some((i, m)) = offen(2) { return w(m.title.clone(), t("ui.wj.muss.grund").into(), frist(m.frist), t("ui.wj.muss.knopf"), Page::Posteingang, Some(i), false); }
        let negs = self.negs.iter().filter(|n| n.state == NegState::Counter && n.stage < 2).count();
        if negs > 0 { return w(crate::tf!("ui.wj.verhandlung.titel", negs), t("ui.wj.verhandlung.grund").into(), String::new(), t("ui.wj.verhandlung.knopf"), Page::Verhandlungen, None, false); }
        if let Some((i, m)) = offen(1) { return w(m.title.clone(), t("ui.wj.soll.grund").into(), frist(m.frist), t("ui.wj.soll.knopf"), Page::Posteingang, Some(i), false); }
        let d = self.world.day;
        if let Some(&(_, bis)) = crate::world::WINDOWS.iter().find(|(von, bis)| *von <= d && d <= *bis && bis - d <= r("fenster_warnung_tage") as u32) {
            return w(t("ui.wj.fenster.titel").into(), crate::tf!("ui.wj.fenster.grund", bis - d), crate::tf!("ui.wj.frist.tage", bis - d), t("ui.wj.fenster.knopf"), Page::Transfermarkt, None, false);
        }
        if let Some(f) = self.next_match() {
            let g = if f.h == self.own { f.a } else { f.h };
            return w(crate::tf!("ui.wj.vorbereitung.titel", self.world.clubs[g].name), crate::tf!("ui.wj.vorbereitung.grund", f.day.saturating_sub(d)), crate::tf!("ui.wj.frist.tage", f.day.saturating_sub(d)), t("ui.wj.vorbereitung.knopf"), Page::Vorbereitung, None, false);
        }
        WasJetzt { titel: t("ui.wj.ruhe.titel").into(), grund: t("ui.wj.ruhe.grund").into(), frist: String::new(), knopf: String::new(), ziel: None, msg: None, vorrang: false }
    }
    /// The season as a journey: stages (label, day) from the own league's schedule, the last day, and the season goal
    /// (goal place, current place, number of teams).
    pub fn saisonreise(&self) -> (Vec<(&'static str, u32)>, u32, (usize, usize, usize)) {
        let l = self.league();
        let mut tage: Vec<u32> = self.world.leagues[l].fixtures.iter().filter(|f| f.h == self.own || f.a == self.own).map(|f| f.day).collect();
        tage.sort();
        let (erst, letzt) = (tage.first().copied().unwrap_or(14), tage.last().copied().unwrap_or(300));
        let mitte = tage.get(tage.len() / 2).copied().unwrap_or((erst + letzt) / 2);
        let winter = crate::world::WINDOWS[1].0;
        let etappen = vec![("ui.reise.start", 0), ("ui.reise.hinrunde", erst), ("ui.reise.rueckrunde", mitte), ("ui.reise.winter", winter), ("ui.reise.ende", letzt)];
        let mut e = etappen;
        e.sort_by_key(|x| x.1);
        (e, letzt.max(1), (self.club().goal_pos, self.world.rank_of(self.own), self.world.leagues[l].teams.len()))
    }
    /// Every Monday a point of the history (kept for three seasons).
    pub fn verlauf_tag(&mut self) {
        if crate::date::weekday(self.world.day) != 1 { return; }
        let sq = self.squad();
        let moral = if sq.is_empty() { 50.0 } else { sq.iter().map(|&p| self.world.players[p].morale).sum::<f32>() / sq.len() as f32 };
        let pkt = VerlaufPunkt { abs: self.abs(), platz: self.world.rank_of(self.own) as u8, staerke: self.team_strength(), teamgeist: self.mensch.teamgeist, moral, kasse: self.club().balance };
        self.verlauf.push(pkt);
        if self.verlauf.len() > 160 { self.verlauf.remove(0); }
    }
    /// After an own match: the match line, each player's competition figures and grade (Teil 3).
    pub fn statistik_buchen(&mut self, res: &MatchResult, wettbewerb: &str, derby: bool) {
        let heim = res.h == self.own;
        let (tore, gegentore) = if heim { (res.hg, res.ag) } else { (res.ag, res.hg) };
        let gegner = if heim { res.a } else { res.h };
        self.spiele.push(SpielZeile { abs: self.abs(), season: self.world.season, wettbewerb: wettbewerb.to_string(), gegner, heim, tore, gegentore, derby });
        let eigen = |c: &Career, p: usize| c.world.players[p].team == c.own as i32;
        let mut zeile = |c: &mut Career, p: usize| -> usize {
            let id = c.world.players[p].id;
            match c.spieler_saison.iter().position(|x| x.pid == id && x.wettbewerb == wettbewerb) {
                Some(i) => i,
                None => { c.spieler_saison.push(SpielerWb { pid: id, wettbewerb: wettbewerb.to_string(), ..Default::default() }); c.spieler_saison.len() - 1 }
            }
        };
        for &(p, m) in &res.minutes { if eigen(self, p) { let i = zeile(self, p); self.spieler_saison[i].spiele += 1; self.spieler_saison[i].minuten += m as u16; } }
        for e in &res.events {
            if eigen(self, e.pid) {
                let i = zeile(self, e.pid);
                match e.kind { 0 => self.spieler_saison[i].tore += 1, 2 => self.spieler_saison[i].gelb += 1, 3 => self.spieler_saison[i].rot += 1, _ => {} }
            }
            if e.kind == 0 { if let Some(a) = e.pid2.filter(|&a| eigen(self, a)) { let i = zeile(self, a); self.spieler_saison[i].vorlagen += 1; } }
        }
        for &(p, r) in &res.ratings {
            if !eigen(self, p) { continue; }
            let i = zeile(self, p);
            self.spieler_saison[i].notensumme += r;
            self.spieler_saison[i].noten += 1;
            let id = self.world.players[p].id;
            self.noten_verlauf.entry(id).or_default().push(r);
        }
    }
    /// At the end of a season the per-competition figures and the grade curves start again (the match lines stay).
    pub fn statistik_saisonende(&mut self) { self.spieler_saison.clear(); self.noten_verlauf.clear(); }
    /// The points of the running season.
    pub fn verlauf_saison(&self) -> Vec<VerlaufPunkt> {
        let ab = abs_day(self.world.season, 0);
        self.verlauf.iter().filter(|p| p.abs >= ab).cloned().collect()
    }
    /// Is the guided first week still shown? (not switched off, not all done, within its days)
    pub fn erste_woche_offen(&self) -> bool {
        !self.erste_woche_aus && !self.erste_woche.iter().all(|x| *x) && self.abs() < self.erste_woche_ab + r("erste_woche.tage") as u32
    }
    /// A step of the first week done: the board sees the new coach settle in (a small plus in trust, more when all are done).
    pub fn erste_woche_schritt(&mut self, i: usize) {
        if !self.erste_woche_offen() || self.erste_woche[i] { return; }
        self.erste_woche[i] = true;
        let g = crate::tf!("ui.woche.grund", t(SCHRITTE[i]));
        self.klima(0.0, 0.0, r("erste_woche.belohnung_schritt"), &g);
        if self.erste_woche.iter().all(|x| *x) { self.klima(r("erste_woche.belohnung_fertig"), 0.0, r("erste_woche.belohnung_fertig"), t("ui.woche.fertig")); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn karriere() -> Career {
        let w = crate::world::testwelt(7);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(0); // roh
        Career::new(w, own, "Test") // roh
    }
    #[test]
    fn was_jetzt_reihenfolge_reise_und_erste_woche() {
        let mut c = karriere();
        let w = c.was_jetzt();
        assert!(!w.titel.is_empty() && !w.grund.is_empty());
        // a press conference can't wait: the „Weiter“ button leads there
        c.press = Some(serde_json::from_value(serde_json::json!({"q": "", "answers": [], "post": false})).unwrap());
        assert!(c.was_jetzt().vorrang && c.was_jetzt().ziel == Some(Page::Presse));
        c.press = None;
        let (e, letzt, (ziel, platz, teams)) = c.saisonreise();
        assert_eq!(e.len(), 5);
        assert!(e.windows(2).all(|x| x[0].1 <= x[1].1) && letzt > 100 && ziel >= 1 && platz >= 1 && teams >= 10);
        let v0 = c.mensch.vorstand;
        for i in 0..5 { c.erste_woche_schritt(i); }
        assert!(!c.erste_woche_offen(), "all five steps done");
        assert!(c.mensch.vorstand > v0, "the board likes a coach who settles in");
        // the history gets a point every Monday
        for _ in 0..15 { c.ultimatum = None; c.end_day().unwrap(); if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } }
        let v = c.verlauf_saison();
        assert!((2..=3).contains(&v.len()), "{} weekly points in 15 days", v.len());
        assert!(v.iter().all(|p| p.platz >= 1 && p.staerke > 0.0));
        // match lines and the players' season per competition
        assert!(!c.spiele.is_empty(), "the own matches are recorded");
        let tore: u32 = c.spiele.iter().map(|x| x.tore as u32).sum();
        let spieler_tore: u32 = c.spieler_saison.iter().map(|x| x.tore as u32).sum();
        assert!(spieler_tore <= tore && (tore == 0 || spieler_tore > 0), "players' goals {spieler_tore} fit the team's {tore} (own goals of the opponent count only for the team)");
        assert!(c.spieler_saison.iter().any(|x| x.spiele > 0 && x.minuten > 0 && x.noten > 0));
        assert!(c.noten_verlauf.values().any(|v| !v.is_empty()));
        c.erste_woche_schritt(0);
        let v1 = c.mensch.vorstand;
        c.erste_woche_schritt(0);
        assert_eq!(c.mensch.vorstand, v1, "no reward twice");
    }
}

#[cfg(test)]
mod karrieren_tests {
    /// Punkt 6, Karrieren verwalten: the list card, three rotating backups, deleting into the bin (never for good).
    /// Runs in the test folder of PESM_KARRIEREN (never the person's careers).
    #[test]
    fn sicherungen_und_papierkorb() {
        let _g = crate::game::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!("pesm-karr-{}", std::process::id()));
        let vorher = std::env::var_os("PESM_KARRIEREN");
        std::env::set_var("PESM_KARRIEREN", &tmp);
        let w = crate::world::testwelt(7);
        let mut c = crate::game::Career::new(w, 0, "Test"); // roh
        c.save_name = "Sicherungstest".into(); // roh
        for _ in 0..5 { c.save_mit_sicherung().unwrap(); }
        let p = c.save_path();
        assert!(crate::game::save_info(&p).map_or(false, |i| i["verein"].as_str() == Some(c.club().name.as_str())));
        assert_eq!(crate::game::sicherungen(&p).len(), 3, "three backups, the oldest drops out");
        crate::game::karriere_wegwerfen(&p).unwrap();
        assert!(!p.exists() && !crate::game::list_saves().contains(&p), "gone from the list");
        let korb = tmp.join("papierkorb");
        assert!(std::fs::read_dir(&korb).unwrap().flatten().any(|e| e.path().join("Sicherungstest.json").exists()), "kept in the bin");
        // only the test's own folder is removed, and the folder of the other tests comes back
        match vorher { Some(v) => std::env::set_var("PESM_KARRIEREN", v), None => std::env::remove_var("PESM_KARRIEREN") }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
