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
            self.feiern.push(Feier { art: crate::id("meilenstein").into(), titel, text, spieler: None, club: Some(own), stimmen: vec![] });
        }
        let titel_jetzt = self.world.titles.iter().filter(|x| x.club == self.own).count();
        if titel_jetzt > self.titel_gefeiert {
            for x in self.world.titles.iter().filter(|x| x.club == self.own).skip(self.titel_gefeiert).cloned().collect::<Vec<_>>() {
                let own = self.own;
                self.feiern.push(Feier { art: crate::id("titel").into(), titel: x.comp.clone(), text: crate::tf!("ui.feier.titel_text", self.club().name, x.comp, crate::date::season_label(x.season)), spieler: None, club: Some(own), stimmen: vec![] });
            }
            self.titel_gefeiert = titel_jetzt;
        }
    }

    /// A signing for the own club, celebrated full screen: photo, crest, number and what the scout said.
    pub fn feier_neuzugang(&mut self, p: usize, von: &str, abloese: f64) {
        let pl = &self.world.players[p];
        let text = crate::tf!("ui.feier.neuzugang_text", pl.name, von, crate::world::money(abloese), crate::world::pos_de(&pl.pos), pl.no);
        let titel = crate::tf!("ui.feier.neuzugang", pl.name);
        let own = self.own;
        let stimmen = self.feier_stimmen(p, false);
        self.feiern.push(Feier { art: crate::id("neuzugang").into(), titel, text, spieler: Some(p), club: Some(own), stimmen });
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
        assert_eq!(liste().len(), 10, "zehn Meilensteine");
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
}
