//! Club members and the annual general meeting (own design after Anstoss; vereinswelt.toml verein.mitglieder_anteil):
//! the members are a share of the fan base, more when the fans are happy; they pay a yearly fee (booked monthly); every
//! summer the meeting judges the season (place against the goal, money, mood) and the coach's speech moves board and fans.
use crate::game::*;
use crate::inhalt::t;
use crate::mensch::zufall;
use crate::regeln::f;
use crate::world::*;

/// Yearly fee per member in euros (before the club's money scale, like a ticket).
const BEITRAG_JAHR: f64 = 60.0;

impl Career {
    /// Members: a share of the fan base, between 70 % and 130 % of it by the fans' mood.
    pub fn mitglieder(&self) -> f64 { (self.fanbasis() * f("vereinswelt", "verein.mitglieder_anteil") as f64 * (0.7 + 0.6 * self.fans_eff() as f64 / 100.0)).round() }
    pub fn beitraege_monat(&self) -> f64 { self.mitglieder() * BEITRAG_JAHR * self.skala().sqrt() / 12.0 }
    /// The meeting's view of the season just ended: 0…100 (goal reached, money, fans' mood).
    pub fn versammlung_zustimmung(&self, rang: usize, ziel: usize, saldo: f64) -> f64 {
        let ziel_teil = if rang <= ziel { 30.0 + 4.0 * (ziel - rang) as f64 } else { 30.0 - 8.0 * (rang - ziel) as f64 };
        let geld = if saldo >= 0.0 { 15.0 } else { -15.0 };
        (35.0 + ziel_teil + geld + (self.fans_eff() as f64 - 50.0) * 0.4).clamp(5.0, 98.0)
    }
    /// Summer: the invitation with the season's figures and the three ways to speak.
    pub fn versammlung_einladen(&mut self, rang: usize, ziel: usize, saldo: f64) {
        let z = self.versammlung_zustimmung(rang, ziel, saldo);
        let m = self.mitglieder();
        let opts = vec![
            (crate::id("bilanz").to_string(), t("mv.opt.bilanz").to_string()),
            (crate::id("ziele").to_string(), t("mv.opt.ziele").to_string()),
            (crate::id("sparen").to_string(), t("mv.opt.sparen").to_string()),
        ];
        let text = crate::tf!("mv.text", format!("{m:.0}"), rang, ziel, money(saldo), crate::wirtschaftsvorgaenge::aussicht(z));
        self.msg(0, t("vw.figur.praesident"), &crate::tf!("mv.titel", crate::date::season_label(self.world.season)), &text, Action::Wahl { art: format!("mv:{}", z.round()), bezug: vec![], optionen: opts });
    }
    /// The vote after the speech: approval (Entlastung) or a vote of no confidence moves board and fans.
    pub fn versammlung_antwort(&mut self, sub: &str, key: &str) -> String {
        let mut z: f64 = sub.parse().unwrap_or(50.0);
        let (fans, vorstand) = match key {
            "ziele" => { z += 8.0; (3.0, -1.0) }
            "sparen" => { z -= 4.0; (-3.0, 4.0) }
            _ => (0.0, 1.0),
        };
        let r = zufall(&[crate::id("versammlung"), &self.abs().to_string()]) as f64 * 20.0 - 10.0;
        let prozent = (z + r).clamp(3.0, 99.0);
        let entlastet = prozent >= 50.0;
        self.klima(fans as f32, 0.0, vorstand as f32 + if entlastet { 3.0 } else { -6.0 }, t("mv.grund"));
        // a promise of big goals is taken at its word: next season's goal one place higher
        if key == crate::id("ziele") { let g = self.world.clubs[self.own].goal_pos; self.world.clubs[self.own].goal_pos = g.saturating_sub(1).max(1); }
        crate::tf!(if entlastet { "mv.entlastet" } else { "mv.misstrauen" }, format!("{prozent:.0}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versammlung_und_beitraege() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.wirtschaft_an();
        assert!(c.mitglieder() > 1000.0 && c.beitraege_monat() > 0.0);
        assert!(c.versammlung_zustimmung(1, 3, 1e6) > c.versammlung_zustimmung(12, 3, -1e6));
        c.versammlung_einladen(2, 3, 1e6);
        let i = c.inbox.iter().position(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("mv:"))).unwrap();
        let v0 = c.mensch.vorstand;
        c.answer(i, true, 0);
        assert!(c.mensch.vorstand != v0);
    }
}
