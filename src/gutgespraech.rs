//! Players who talk when nothing is wrong (Fabian 23.11., ideas of other FL career tools): an in-form player says he
//! is ready, a prospect asks what it takes, the captain reports the mood after a good run, a young player with few
//! minutes asks for games or a loan once a season. And the technical director brings one recommendation a month.
//! Texts come from bags (no repeats), every talk has its own quiet time.
use crate::game::*;
use crate::regeln::f;
use crate::world::money;

fn k(s: &str) -> f32 { f("gutgespraech", s) }

impl Career {
    /// Weekly (Mondays): at most one of these talks.
    pub fn gutgespraech_woche(&mut self) {
        if crate::date::weekday(self.world.day) != 0 || self.fired || self.abs() < self.ruhe_bis { return; }
        let abs = self.abs();
        let season = self.world.season;
        let frei = |c: &Career, art: &str, p: usize| !c.mensch.story_erzaehlt.iter().any(|x| x.0 == art && x.2 == c.world.players[p].id && (x.1 / crate::date::SEASON_DAYS == season || abs < x.1 + k("sperre_tage") as u32));
        let sq: Vec<usize> = self.squad().into_iter().filter(|&p| self.world.players[p].youth == 0).collect();
        let xi = self.xi.clone();
        let res: Vec<char> = self.results.iter().map(|r| r.chars().next().unwrap_or('U')).collect();
        let ungeschlagen = res.iter().rev().take_while(|&&x| x != 'N').count();
        let mut kandidat: Option<(&'static str, usize)> = None;
        for &p in &sq {
            let pl = &self.world.players[p];
            let form = crate::pages::form5_pub(pl).unwrap_or(6.5);
            if form >= k("bereit_note") && !xi.contains(&p) && frei(self, "gut_bereit", p) { kandidat = Some(("gut_bereit", p)); break; }
            if pl.age <= 21 && pl.pot >= pl.ovr + k("talent_luecke") && frei(self, "gut_talent", p) && self.world.day > 40 { kandidat = Some(("gut_talent", p)); break; }
            if pl.age <= 22 && pl.apps < 3 && self.world.day > 80 && frei(self, "gut_minuten", p) { kandidat = Some(("gut_minuten", p)); break; }
        }
        if kandidat.is_none() && ungeschlagen as f32 >= k("serie_kapitaen") {
            if let Some(&kap) = sq.iter().find(|&&p| self.world.players[p].ps.rolle == 0) { if frei(self, "gut_kapitaen", kap) { kandidat = Some(("gut_kapitaen", kap)); } }
        }
        let Some((art, p)) = kandidat else { return };
        if crate::mensch::zufall(&["gutgespraech", art, &abs.to_string()]) > k("chance") { return; } // roh
        self.mensch.story_erzaehlt.push((art.to_string(), abs, self.world.players[p].id));
        let n = self.world.players[p].name.clone();
        let basis = format!("ui.gut.{art}");
        let v = self.variante(&format!("gut:{art}"), crate::inhalt::varianten(&format!("{basis}.text")), &n);
        let text = crate::tf!(&crate::inhalt::variante_key(&format!("{basis}.text"), v), n.clone(), ungeschlagen.to_string());
        let opts: Vec<(String, String)> = ["a", "b"].iter().map(|o| (o.to_string(), crate::inhalt::t(&format!("{basis}.opt.{o}")).to_string())).collect(); // roh
        self.msg(1, &n, &crate::tf!(&format!("{basis}.titel"), n.clone()), &text, Action::Wahl { art: format!("gut:{art}"), bezug: vec![p], optionen: opts });
    }
    pub fn gut_antwort(&mut self, art: &str, p: usize, key: &str) -> String {
        let (moral, text) = match (art, key) {
            ("gut_bereit", "a") => { if !self.xi.contains(&p) { self.xi.push(p); } (k("moral_ja"), "ui.gut.gut_bereit.a") }
            ("gut_minuten", "b") => { let _ = self.leihe_abgeben(p); (k("moral_ja") / 2.0, "ui.gut.gut_minuten.b") }
            (_, "a") => (k("moral_ja"), "ui.gut.antwort.a"),
            _ => (k("moral_nein"), "ui.gut.antwort.b"),
        };
        self.moral_wirkung(p, moral, 0, crate::inhalt::t("ui.gut.grund"));
        crate::inhalt::t(text).into()
    }
    /// Monthly: the technical director's one recommendation from the squad as it stands; agreeing carries it out.
    pub fn sportdirektor_monat(&mut self) {
        if self.fired { return; }
        let sq: Vec<usize> = self.squad().into_iter().filter(|&p| self.world.players[p].youth == 0).collect();
        let season = self.world.season;
        let lohn = self.world.wage_bill(self.own) / sq.len().max(1) as f64;
        // the first thing that fits, in this order: an expiring regular, a young one without minutes, an expensive
        // bench player, a thin position
        let ende = sq.iter().copied().filter(|&p| self.world.players[p].contract_end <= season && self.world.players[p].ovr >= self.team_strength() - 2.0).max_by(|a, b| self.world.players[*a].ovr.partial_cmp(&self.world.players[*b].ovr).unwrap());
        let jung = sq.iter().copied().find(|&p| { let pl = &self.world.players[p]; pl.age <= 21 && pl.apps < 2 && pl.leihe.is_none() && !pl.ps.leihliste && self.world.day > 60 });
        let teuer = sq.iter().copied().find(|&p| { let pl = &self.world.players[p]; pl.age >= 29 && pl.apps < 3 && pl.wage > lohn * 1.4 && !pl.listed && self.world.day > 60 });
        let (art, p, text) = if let Some(p) = ende { (crate::id("verlaengern"), p, crate::tf!("ui.sd.verlaengern", self.world.players[p].name.clone())) }
            else if let Some(p) = jung { (crate::id("leihen"), p, crate::tf!("ui.sd.leihen", self.world.players[p].name.clone())) }
            else if let Some(p) = teuer { (crate::id("liste"), p, crate::tf!("ui.sd.liste", self.world.players[p].name.clone(), money(self.world.players[p].wage))) }
            else { return };
        // the same recommendation about the same player not again for a while
        let abs = self.abs();
        let id = self.world.players[p].id;
        if self.mensch.story_erzaehlt.iter().any(|x| x.0 == format!("sd:{art}") && x.2 == id && abs < x.1 + k("sd_sperre_tage") as u32) { return; } // roh
        self.mensch.story_erzaehlt.push((format!("sd:{art}"), abs, id)); // roh
        let opts = vec![("ja".to_string(), crate::inhalt::t("ui.sd.ja").to_string()), ("nein".to_string(), crate::inhalt::t("ui.sd.nein").to_string())]; // roh
        self.msg(2, crate::inhalt::t("ui.sd.absender"), crate::inhalt::t("ui.sd.titel"), &text, Action::Wahl { art: format!("sd:{art}"), bezug: vec![p], optionen: opts });
    }
    pub fn sd_antwort(&mut self, art: &str, p: usize, key: &str) -> String {
        if key != "ja" { return crate::inhalt::t("ui.sd.abgelehnt").into(); }
        match art {
            "verlaengern" => match self.open_neg(p) { Ok(_) => crate::inhalt::t("ui.sd.verhandlung_offen").into(), Err(e) => e },
            "leihen" => match self.leihe_abgeben(p) { Ok(()) => crate::inhalt::t("ui.leihe.auf_liste").into(), Err(e) => e },
            _ => match self.verkauf_gesperrt(p) { Some(g) => g, None => { self.world.players[p].listed = true; crate::inhalt::t("ui.sd.gelistet").into() } }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::*;
    /// A good talk comes, is answered, and the same talk does not come back for that player in the season.
    #[test]
    fn gute_gespraeche_ohne_wiederholung() {
        let w = crate::world::testwelt(5);
        let mut c = Career::new(w, 1, "Test");
        let mut zahl = std::collections::HashMap::<String, usize>::new();
        for _ in 0..200 {
            c.world.day += 1;
            let n = c.inbox.len();
            c.gutgespraech_woche();
            for m in c.inbox.iter().skip(n) { if let Action::Wahl { art, bezug, .. } = &m.action { *zahl.entry(format!("{art}|{bezug:?}")).or_default() += 1; } }
        }
        assert!(zahl.values().all(|&n| n == 1), "a talk twice: {zahl:?}");
        c.sportdirektor_monat();
    }
}
