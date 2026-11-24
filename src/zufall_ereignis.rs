//! Random events (ideas of other FL tools, Fabian 23.11.): a rich owner buys a club in the own league and spends, and
//! the supporters boycott the home end after a bad run until the coach meets them. Numbers in zufall_ereignis.toml.
use crate::game::*;
use crate::regeln::f;
use crate::world::money;

fn k(s: &str) -> f32 { f("zufall_ereignis", s) }

impl Career {
    /// Monthly: maybe a takeover in the own league, maybe a boycott.
    pub fn zufall_ereignisse_monat(&mut self) {
        if self.fired { return; }
        let abs = self.abs();
        // a takeover: at most once a season, a club of the own league below the top gets a rich owner
        let season = self.world.season;
        if !self.mensch.story_erzaehlt.iter().any(|x| x.0 == "uebernahme" && x.1 / crate::date::SEASON_DAYS == season) // roh
            && crate::mensch::zufall(&["uebernahme", &abs.to_string()]) < k("uebernahme_chance") { // roh
            let teams = self.world.leagues[self.league()].teams.clone();
            let mut kand: Vec<usize> = teams.into_iter().filter(|&t| t != self.own).collect();
            kand.sort_by(|a, b| self.world.clubs[*a].level.partial_cmp(&self.world.clubs[*b].level).unwrap());
            if let Some(&t) = kand.get(kand.len() / 3) {
                let namen = crate::regeln::wert("laufbahn", "erfunden.investoren").as_array().cloned().unwrap_or_default();
                let inv = namen.get((abs as usize) % namen.len().max(1)).and_then(|x| x.as_str()).unwrap_or("").to_string();
                let geld = (self.world.revenue(t) * k("uebernahme_umsatz") as f64).round();
                self.world.clubs[t].balance += geld;
                self.mensch.story_erzaehlt.push(("uebernahme".into(), abs, self.world.clubs[t].id)); // roh
                let cn = self.world.clubs[t].name.clone();
                self.news(&crate::tf!("ui.ze.uebernahme_titel", cn.clone(), inv.clone()), &crate::tf!("ui.ze.uebernahme_text", inv, cn, money(geld)));
            }
        }
        // a boycott: the mood of the fans very low and no win in the last matches
        let res: Vec<char> = self.results.iter().map(|r| r.chars().next().unwrap_or('U')).collect();
        let sieglos = res.iter().rev().take_while(|&&x| x != 'S').count();
        if self.boykott_bis <= abs && self.fans < k("boykott_fans_unter") && sieglos as f32 >= k("boykott_sieglos") {
            self.boykott_bis = abs + k("boykott_tage") as u32;
            let opts = vec![("treffen".to_string(), crate::inhalt::t("ui.ze.treffen").to_string()), ("nein".to_string(), crate::inhalt::t("ui.ze.nicht_treffen").to_string())]; // roh
            self.msg(5, crate::inhalt::t("ui.storys.fanvertreter"), crate::inhalt::t("ui.ze.boykott_titel"), &crate::tf!("ui.ze.boykott_text", sieglos), Action::Wahl { art: "boykott:x".into(), bezug: vec![], optionen: opts }); // roh
            let cn = self.club().name.clone();
            self.news(&crate::tf!("ui.ze.boykott_news", cn), "");
        }
    }
    pub fn boykott_antwort(&mut self, key: &str) -> String {
        if key == "treffen" {
            self.boykott_bis = self.abs();
            self.klima(k("treffen_fans"), 0.0, 0.0, crate::inhalt::t("ui.ze.boykott_titel"));
            crate::inhalt::t("ui.ze.getroffen").into()
        } else { crate::inhalt::t("ui.ze.nicht_getroffen").into() }
    }
    /// The share of ticket income left during a boycott (1.0 without).
    pub fn boykott_faktor(&self) -> f64 { if self.abs() < self.boykott_bis { k("boykott_tickets") as f64 } else { 1.0 } }
}

#[cfg(test)]
mod tests {
    use crate::game::*;
    #[test]
    fn boykott_und_treffen() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        let d = 10.0 - c.fans; c.klima(d, 0.0, 0.0, "Test");
        c.results = vec!["N 0:1".into(); 6];
        c.zufall_ereignisse_monat();
        assert!(c.boykott_faktor() < 1.0, "boycott after a bad run with angry fans");
        let i = c.inbox.iter().position(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("boykott"))).unwrap();
        c.answer(i, true, 0);
        assert_eq!(c.boykott_faktor(), 1.0, "meeting the fans ends it");
    }
}
