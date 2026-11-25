//! Career milestones and reward moments (SPIELGEFUEHL.md measures 3 and 4): ten milestones with progress and a seal,
//! and full-screen moments for a milestone, a title, a new licence and a signing. The milestones are PES M's own
//! (the Python version only had an empty tab), the numbers are in assets/regeln/meilensteine.toml.
use crate::game::*;
use serde::{Deserialize, Serialize};

/// A reward moment waiting to be shown full screen (shown once, then gone; not saved).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Feier {
    pub art: String,
    pub titel: String,
    pub text: String,
    pub spieler: Option<usize>,
    pub club: Option<usize>,
    /// Voices on a big signing or renewal (Fabian 24.11.): (who, what they say), shown under the text.
    #[serde(default)]
    pub stimmen: Vec<(String, String)>,
    /// An upcoming big match (Fabian 25.11.): the opponent and the competition (cup or not, index), for crests and colours.
    #[serde(default)]
    pub gegner: Option<usize>,
    #[serde(default)]
    pub wo: Option<(bool, usize)>,
}

/// (id, kind of count, target) of the milestones, from meilensteine.toml.
pub fn liste() -> Vec<(String, String, f32)> {
    crate::mensch::arr("meilensteine", "meilenstein").iter().filter_map(|m| Some((m["id"].as_str()?.to_string(), m["zaehlt"].as_str()?.to_string(), m["ziel"].as_f64()? as f32))).collect()
}

impl Career {
    /// The current count of a milestone kind: wins, matches, longest unbeaten run, clean sheets (own matches), titles,
    /// seasons as a coach, own players up to 19 with a first-team match, licence level (0 c … 3 pro).
    pub fn meilenstein_stand(&self, zaehlt: &str) -> f32 {
        let sp = &self.spiele;
        match zaehlt {
            "siege" => sp.iter().filter(|x| x.tore > x.gegentore).count() as f32,
            "spiele" => sp.len() as f32,
            "serie" => { let (mut n, mut best) = (0, 0); for x in sp { if x.tore >= x.gegentore { n += 1; best = best.max(n); } else { n = 0; } } best as f32 }
            "zu_null" => sp.iter().filter(|x| x.gegentore == 0).count() as f32,
            "titel" => self.world.titles.iter().filter(|x| x.club == self.own).count() as f32,
            "saisons" => (self.abs().saturating_sub(self.mgr_since) / crate::date::SEASON_DAYS) as f32 + self.history.len() as f32,
            "talente" => self.squad().iter().filter(|&&p| self.world.players[p].age <= 19 && self.world.players[p].apps > 0).count() as f32,
            "lizenz" => crate::mensch::LIZENZEN.iter().position(|s| *s == self.lizenz()).unwrap_or(0) as f32,
            "kantersiege" => sp.iter().filter(|x| x.tore >= x.gegentore + 4).count() as f32,
            "derbysiege" => sp.iter().filter(|x| x.derby && x.tore > x.gegentore).count() as f32,
            "europa" => sp.iter().filter(|x| self.world.cups.iter().any(|c| (1..=3).contains(&c.kind) && c.name == x.wettbewerb)).count() as f32,
            "europa_siege" => sp.iter().filter(|x| x.tore > x.gegentore && self.world.cups.iter().any(|c| (1..=3).contains(&c.kind) && c.name == x.wettbewerb)).count() as f32,
            "tore" => sp.iter().map(|x| x.tore as f32).sum(),
            _ => 0.0,
        }
    }

    /// After every day: newly reached milestones are kept with their day, announced in the inbox and celebrated;
    /// a new own title is celebrated too.
    pub fn meilensteine_pruefen(&mut self) {
        let abs = self.abs();
        for (id, zaehlt, ziel) in liste() {
            if self.meilensteine.iter().any(|m| m.0 == id) { continue; }
            if self.meilenstein_stand(&zaehlt) < ziel { continue; }
            self.meilensteine.push((id.clone(), abs));
            let titel = crate::inhalt::t(&format!("ui.meilenstein.{id}")).to_string();
            let text = crate::tf!("ui.meilenstein.erreicht_text", titel, self.club().name);
            let co = self.co_name.clone();
            self.msg(0, &co, &crate::tf!("ui.meilenstein.erreicht", titel), &text, Action::None);
            self.meldung_art(crate::pages::Art::Meilenstein, vec![]);
            let own = self.own;
            self.feiern.push(Feier { art: crate::id("meilenstein").into(), titel, text, spieler: None, club: Some(own), stimmen: vec![], ..Default::default() });
        }
        self.spieler_meilensteine();
        // a big match today (Fabian 25.11.): a derby or a final is announced once, full screen, with both crests
        if let Some(t) = self.today_match() {
            let key = format!("ank:{}", self.abs()); // roh
            if !self.meilensteine.iter().any(|m| m.0 == key) {
                let f = self.fixture(t);
                let gegner = if f.h == self.own { f.a } else { f.h };
                let (own_n, opp_n) = (self.club().name.clone(), self.world.clubs[gegner].name.clone());
                let derby = crate::game::is_derby(&own_n, &opp_n);
                let finale = matches!(t.0, Wo::Pokal(ci) if self.world.cups[ci].fx[t.1].stage == 34 && self.world.cups[ci].kind != 6);
                if derby.is_some() || finale {
                    self.meilensteine.push((key, self.abs()));
                    let comp = self.comp_name(t);
                    let (art, titel) = if finale { (crate::id("finale"), crate::tf!("ui.feier.finale_titel", comp.clone())) } else { (crate::id("derby"), derby.map(|d| d.to_string()).unwrap_or_default()) };
                    let vk = self.vkey(&format!("ui.feier.{art}_text"), &opp_n);
                    let text = crate::tf!(&vk, own_n, opp_n, comp);
                    let wo = match t.0 { Wo::Liga(l) => (false, l), Wo::Pokal(ci) => (true, ci) };
                    let own = self.own;
                    self.feiern.push(Feier { art: art.into(), titel, text, spieler: None, club: Some(own), gegner: Some(gegner), wo: Some(wo), ..Default::default() });
                }
            }
        }
        let titel_jetzt = self.world.titles.iter().filter(|x| x.club == self.own).count();
        if titel_jetzt > self.titel_gefeiert {
            for x in self.world.titles.iter().filter(|x| x.club == self.own).skip(self.titel_gefeiert).cloned().collect::<Vec<_>>() {
                let own = self.own;
                self.feiern.push(Feier { art: crate::id("titel").into(), titel: x.comp.clone(), text: crate::tf!("ui.feier.titel_text", self.club().name, x.comp, crate::date::season_label(x.season)), spieler: None, club: Some(own), stimmen: vec![], ..Default::default() });
            }
            self.titel_gefeiert = titel_jetzt;
        }
    }

    /// A player's count for a player milestone at the club: games and goals for the club (finished seasons in his
    /// career list plus this season), caps, a hat-trick this week.
    pub fn spieler_stand(&self, p: usize, zaehlt: &str) -> u32 {
        let pl = &self.world.players[p];
        let club = &self.club().name;
        let frueher = |i: usize| pl.profil.laufbahn.iter().filter(|x| &x.1 == club).map(|x| if i == 0 { x.2 as u32 } else { x.3 as u32 }).sum::<u32>();
        match zaehlt {
            "spiele" => frueher(0) + pl.apps as u32,
            "tore" => frueher(1) + pl.goals as u32,
            "laenderspiele" => pl.profil.laenderspiele as u32,
            "hattrick" => self.world.noten_woche.iter().any(|x| x.0 == p && x.2 >= 3) as u32,
            _ => 0,
        }
    }
    /// Player milestones of the own squad (meilensteine.toml [[spieler]]): debut, first goal, hat-trick, 50/100/200/300
    /// games, 25/50/100 goals, first cap; an inbox note with his banner, a boost for his morale, the big ones
    /// celebrated full screen. The first check only marks what was reached before.
    pub fn spieler_meilensteine(&mut self) {
        let liste: Vec<(String, String, u32, bool)> = crate::mensch::arr("meilensteine", "spieler").iter().filter_map(|m| Some((m["id"].as_str()?.to_string(), m["zaehlt"].as_str()?.to_string(), m["ziel"].as_u64()? as u32, m["feier"].as_bool().unwrap_or(false)))).collect();
        let erste = self.spieler_ms.is_none();
        let mut erreicht = self.spieler_ms.take().unwrap_or_default();
        let abs = self.abs();
        for p in self.squad() {
            let id = self.world.players[p].id;
            for (ms, zaehlt, ziel, feier) in &liste {
                if erreicht.iter().any(|x| x.0 == id && &x.1 == ms) || self.spieler_stand(p, zaehlt) < *ziel { continue; }
                erreicht.push((id, ms.clone(), abs));
                // a hat-trick or a brace counts again in another week; the career ones once
                if erste { continue; }
                let name = self.world.players[p].name.clone();
                let titel = crate::tf!(&format!("ui.sms.{ms}.titel"), name.clone());
                let vk = self.vkey(&format!("ui.sms.{ms}.text"), &format!("{id}"));
                let text = crate::tf!(&vk, name.clone(), self.club().name.clone(), self.spieler_stand(p, zaehlt));
                self.world.players[p].morale = (self.world.players[p].morale + crate::regeln::f("meilensteine", "spieler_moral")).min(95.0);
                let co = self.co_name.clone();
                self.msg(0, &co, &titel, &text, Action::Report { pid: p });
                self.meldung_art(crate::pages::Art::Meilenstein, vec![p]);
                // a milestone of the Skill Tree list gives a point at once (skilltree.toml punkte.meilenstein_sp)
                if crate::regeln::wert("skilltree", "punkte.meilenstein_sp").as_array().into_iter().flatten().any(|x| x.as_str() == Some(ms.as_str())) { self.st_punkt(p, &titel); }
                if *feier { let own = self.own; self.feiern.push(Feier { art: crate::id("meilenstein").into(), titel, text, spieler: Some(p), club: Some(own), stimmen: vec![], ..Default::default() }); }
            }
        }
        // a hat-trick is weekly: forgotten once the week's grades are gone, so the next one counts again
        erreicht.retain(|x| x.1 != crate::id("hattrick") || abs < x.2 + 8);
        self.spieler_ms = Some(erreicht);
    }
    /// A signing for the own club, celebrated full screen: photo, crest, number and what the scout said.
    pub fn feier_neuzugang(&mut self, p: usize, von: &str, abloese: f64) {
        let pl = &self.world.players[p];
        let text = crate::tf!("ui.feier.neuzugang_text", pl.name, von, crate::world::money(abloese), crate::world::pos_de(&pl.pos), pl.no);
        let titel = crate::tf!("ui.feier.neuzugang", pl.name);
        let own = self.own;
        let stimmen = self.feier_stimmen(p, false);
        self.feiern.push(Feier { art: crate::id("neuzugang").into(), titel, text, spieler: Some(p), club: Some(own), stimmen, ..Default::default() });
    }
    /// Fans, a transfer reporter and the captain on a signing or renewal of a player at least at the club's level;
    /// each line from its own shuffle bag. None for smaller deals.
    pub fn feier_stimmen(&mut self, p: usize, verlaengerung: bool) -> Vec<(String, String)> {
        if self.world.players[p].ovr < self.club().level { return vec![]; }
        let art = if verlaengerung { "vertrag" } else { "neuzugang" }; // roh
        let n = self.world.players[p].name.clone();
        let mut z = |c: &mut Career, wer: String, k: &str| { let v = c.variante(&format!("feier_{art}_{k}"), 4, &n); (wer, crate::tf!(&format!("ui.feier.stimme.{art}.{k}.{v}"), n.clone())) };
        let mut out = vec![z(self, crate::inhalt::t("ui.feier.stimme.fans").into(), crate::id("fans"))];
        let blatt = self.blatt(0);
        if !blatt.is_empty() { out.push(z(self, blatt, crate::id("presse"))); }
        if let Some(k) = self.kapitaen().filter(|&k| k != p) { let kn = self.world.players[k].name.clone(); out.push(z(self, kn, crate::id("kapitaen"))); }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fabian 24.11.: a big signing gets voices (fans, press, captain), a small one none; no voice twice in a row.
    #[test]
    fn grosse_transfers_mit_stimmen() {
        let mut c = Career::new(crate::world::testwelt(3), 2, "Test");
        let lvl = c.club().level;
        let gross = (0..c.world.players.len()).find(|&p| c.world.players[p].ovr >= lvl + 2.0).unwrap();
        let klein = (0..c.world.players.len()).find(|&p| c.world.players[p].ovr < lvl - 10.0).unwrap();
        let a = c.feier_stimmen(gross, false);
        assert!(!a.is_empty() && a.iter().all(|(w, t)| !w.is_empty() && !t.is_empty() && !t.contains("ui.")), "{a:?}");
        assert!(c.feier_stimmen(klein, true).is_empty());
        let b = c.feier_stimmen(gross, false);
        assert_ne!(a[0].1, b[0].1, "the fans don't say the same twice");
    }
    #[test]
    fn meilensteine_mit_fortschritt_und_feier() {
        let w = crate::world::testwelt(7);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(0);
        let mut c = Career::new(w, own, "Test");
        assert_eq!(liste().len(), 19, "19 Meilensteine");
        c.meilensteine_pruefen();
        let vorher = c.meilensteine.len();
        // a first win: the milestone comes once, with a message and a reward moment
        c.spiele.push(crate::spielgefuehl::SpielZeile { abs: 1, season: 0, wettbewerb: String::new(), gegner: (own + 1) % c.world.clubs.len(), heim: true, tore: 2, gegentore: 0, ..Default::default() });
        c.meilensteine_pruefen();
        assert!(c.meilensteine.iter().any(|m| m.0 == "erster_sieg"));
        assert!(c.meilensteine.len() > vorher);
        assert!(c.feiern.iter().any(|f| f.art == "meilenstein"));
        let n = c.meilensteine.len();
        c.meilensteine_pruefen();
        assert_eq!(c.meilensteine.len(), n, "nie doppelt");
        // a title is celebrated once
        c.world.titles.push(crate::cups::Title { season: 0, comp: "Pokal".into(), club: own });
        c.feiern.clear();
        c.meilensteine_pruefen();
        assert_eq!(c.feiern.iter().filter(|f| f.art == "titel").count(), 1);
        c.meilensteine_pruefen();
        assert_eq!(c.feiern.iter().filter(|f| f.art == "titel").count(), 1);
    }
    /// Fabian 24.11.: player milestones: what was reached before the career stays quiet, a debut and a hat-trick come
    /// with a message, a hat-trick full screen, and a second one in another week counts again.
    #[test]
    fn spieler_meilensteine() {
        let mut c = Career::new(crate::world::testwelt(7), 0, "Test");
        let p = c.squad()[3];
        c.world.players[p].apps = 0;
        c.world.players[p].profil.laufbahn.clear();
        c.spieler_meilensteine();
        assert!(c.spieler_ms.is_some());
        let name = c.world.players[p].name.clone();
        let zahl = |c: &Career| c.inbox.iter().filter(|m| m.title.contains(&name)).count();
        let n0 = zahl(&c);
        c.world.players[p].apps = 1;
        c.spieler_meilensteine();
        assert_eq!(zahl(&c), n0 + 1, "debut");
        c.spieler_meilensteine();
        assert_eq!(zahl(&c), n0 + 1, "once");
        c.world.noten_woche.push((p, 8.5, 3));
        c.spieler_meilensteine();
        assert!(c.feiern.iter().any(|f| f.spieler == Some(p)), "hat-trick celebrated");
        c.world.noten_woche.clear();
        c.world.day += 10;
        c.spieler_meilensteine();
        c.world.noten_woche.push((p, 8.0, 3));
        let n = zahl(&c);
        c.spieler_meilensteine();
        assert_eq!(zahl(&c), n + 1, "a new week, a new hat-trick");
    }
}
