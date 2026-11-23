//! The experts' season preview (own design after the EA manager's pre-season tips): before the season the papers rank the
//! league by squad strength (favourite, relegation candidates, our predicted place); at the season's end the forecast is
//! held against the table, and beating it lifts fans and media, falling short costs a little.
use crate::game::*;
use crate::inhalt::t;

impl Career {
    /// The league ranked by squad strength (strongest first).
    pub fn experten_rangliste(&self) -> Vec<usize> {
        let mut v = self.world.leagues[self.league()].teams.clone();
        v.sort_by(|a, b| self.world.clubs[*b].level.partial_cmp(&self.world.clubs[*a].level).unwrap());
        v
    }
    /// At the season's start: the tip in the papers and a message; the predicted place is kept.
    pub fn experten_tipp(&mut self) {
        let r = self.experten_rangliste();
        if r.len() < 6 { return; }
        let platz = r.iter().position(|&c| c == self.own).map(|i| i + 1).unwrap_or(0);
        self.experten_platz = Some((self.world.season, platz));
        let n = r.len();
        let unten: Vec<String> = r[n - 3..].iter().map(|&c| self.world.clubs[c].name.clone()).collect();
        let text = crate::tf!("ep.text", self.world.clubs[r[0]].name.clone(), self.world.clubs[r[1]].name.clone(), unten.join(", "), platz);
        let ln = self.world.leagues[self.league()].name.clone();
        self.news(&crate::tf!("ep.titel", ln.clone()), &text);
        self.msg(4, t("ui.medien.pressestelle"), &crate::tf!("ep.titel", ln), &text, Action::None);
    }
    /// At the season's end: the table against the forecast.
    pub fn experten_bilanz(&mut self, rang: usize) {
        let Some((s, platz)) = self.experten_platz.take() else { return };
        if s != self.world.season || platz == 0 || rang == 0 { return; }
        let d = platz as i32 - rang as i32;
        let (key, fans, medien) = if d >= 3 { ("ep.besser", 3.0, 3.0) } else if d <= -3 { ("ep.schlechter", -2.0, -2.0) } else { ("ep.wie_erwartet", 0.0, 0.0) };
        if fans != 0.0 { self.klima(fans, medien, 0.0, t("ep.grund")); }
        let cn = self.club().name.clone();
        self.news(&crate::tf!("ep.bilanz_titel", cn), &crate::tf!(key, platz, rang));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tipp_und_bilanz() {
        let w = crate::world::World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        c.experten_tipp();
        let (_, p) = c.experten_platz.unwrap();
        assert!(p >= 1);
        let f0 = c.fans;
        c.experten_bilanz(p.saturating_sub(5).max(1));
        assert!(p < 6 || c.fans > f0, "beating the forecast lifts the fans");
    }
}
