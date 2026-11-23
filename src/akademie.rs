//! Paket 3p: academy and U23 (Mechanik-Übergabe §19, akademie.toml): Akademie 16–17 (youth 17), U23 18–22
//! (youth 19), places, yearly intake, weekly development, promotion proposals and parents asking for play time.
use crate::game::*;
use crate::mensch::zufall;
use crate::regeln::f;
use crate::world::*;

impl Career {
    fn a(k: &str) -> f32 { f("akademie", k) }
    /// Places: Akademie 20, U23 22, +2 per level of the youth centre above 1, at most 40 together.
    pub fn akademie_plaetze(&self) -> (usize, usize) {
        let plus = (Self::a("kader.ausbau_je_stufe") * (self.wi.stufen[3].max(1) as f32 - 1.0)) as usize;
        let u23 = if self.zweite_mannschaft().is_some() { 0 } else { Self::a("kader.u23_plaetze") as usize + plus };
        (Self::a("kader.akademie_plaetze") as usize + plus, u23)
    }
    /// The yearly intake in July: 3–6 players aged 16–17, strength 34–52, potential 55–90 (+4 per youth level),
    /// 60 000 € a year for 3 years (vereinswelt.toml [jugend], akademie.toml [aufnahme]).
    pub fn akademie_jahrgang(&mut self) { self.talenttag(); }
    /// The week of the own youth: play share and grade of the U23 and academy for the development per ability
    /// (entwicklung.rs, which runs right after), the parents' question after weeks without matches.
    pub fn akademie_woche(&mut self) {
        let mut u23: Vec<usize> = self.world.clubs[self.own].squad.clone().into_iter().filter(|&p| self.world.players[p].youth == 19).collect();
        u23.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
        let mittel = if u23.is_empty() { 50.0 } else { u23.iter().map(|&p| self.world.players[p].ovr).sum::<f32>() / u23.len() as f32 };
        let spielt = u23.len() >= Self::a("kader.u23_mindestens") as usize;
        let ohne_wochen = Self::a("ohne_perspektive.wochen_mindestens") as u32;
        // at most one parents' question a week; the others wait for the next week (Fabian 22.11.: too many at once)
        let mut eltern_gefragt = false;
        // Konzept Akademie 4.2: homesickness in the first six months for talents from abroad; a countryman in the
        // academy or a mentor halves it
        let land = self.world.land_nat(self.league());
        let abs = self.abs();
        for p in self.world.clubs[self.own].squad.clone() {
            let pl = &self.world.players[p];
            if pl.youth == 0 || pl.nat == 0 || pl.nat == land || abs.saturating_sub(pl.joined) > Self::a("heimweh.tage") as u32 { continue; }
            let landsmann = self.world.clubs[self.own].squad.iter().any(|&q| q != p && self.world.players[q].nat == pl.nat);
            let halb = landsmann || pl.ps.mentor.is_some();
            let m = &mut self.world.players[p].morale;
            *m = (*m - Self::a("heimweh.je_woche") * if halb { 0.5 } else { 1.0 }).max(10.0);
        }
        for p in self.world.clubs[self.own].squad.clone() {
            let pl = &self.world.players[p];
            if pl.youth == 0 { continue; }
            let akademie = pl.youth == 17;
            let stamm = u23.iter().take(11).any(|&q| q == p) || pl.ps.u23_stamm;
            let quote = if akademie || !spielt { 0.0 } else if stamm { 1.0 } else { 0.3 };
            let note = if quote > 0.0 { 6.0 + (pl.ovr - mittel) / 10.0 } else { 6.0 };
            // the growth itself runs per ability in entwicklung.rs; the academy gives it the play share and the grade
            self.world.players[p].ent.u23 = Some((quote, note));
            // U23 without matches for 4 weeks: the parents ask (Paket 3p, §20 parents with effect)
            if !akademie && !stamm {
                let pl = &mut self.world.players[p];
                if !(eltern_gefragt && pl.ps.ohne_spiel + 1 == ohne_wochen) { pl.ps.ohne_spiel += 1; }
                if pl.ps.ohne_spiel == ohne_wochen && !eltern_gefragt {
                    eltern_gefragt = true;
                    let n = pl.name.clone();
                    let opts = vec![(crate::id("spielzeit").to_string(), crate::inhalt::t("ui.akademie.er_bekommt_seine_spiele_in").to_string()), (crate::id("geduld").to_string(), crate::inhalt::t("ui.akademie.er_muss_sich_gedulden").to_string()), (crate::id("leihe").to_string(), crate::inhalt::t("ui.akademie.wir_verleihen_ihn_damit_er").to_string())];
                    self.msg(1, &crate::tf!("ui.akademie.eltern_von", n), &crate::tf!("ui.akademie.die_eltern_von_fragen_nach", n), &crate::tf!("ui.akademie.spielt_seit_wochen_nicht_seine", n), Action::Wahl { art: crate::id("eltern").into(), bezug: vec![p], optionen: opts });
                }
            } else { self.world.players[p].ps.ohne_spiel = 0; }
        }
        self.befoerderung_vorschlaege();
    }
    /// Promotion proposal (akademie.toml [befoerderung]): strength at most 4 below the weakest regular, U23 player,
    /// grade 6.3 or better, at most 40 professionals.
    fn befoerderung_vorschlaege(&mut self) {
        let abs = self.abs();
        if abs % 28 != 0 { return; }
        let profis: Vec<usize> = self.world.clubs[self.own].squad.clone().into_iter().filter(|&p| self.world.players[p].youth == 0).collect();
        if profis.len() as f32 >= Self::a("befoerderung.kader_hoechstens") { return; }
        let schwach = self.xi.iter().map(|&p| self.world.players[p].ovr).fold(f32::MAX, f32::min);
        for p in self.world.clubs[self.own].squad.clone() {
            let pl = &self.world.players[p];
            if pl.youth != 19 || pl.ovr < schwach - Self::a("befoerderung.abstand_profi") || pl.ps.befoerdert_gefragt { continue; }
            let n = pl.name.clone();
            self.world.players[p].ps.befoerdert_gefragt = true;
            let opts = vec![(crate::id("befoerdern").to_string(), crate::inhalt::t("ui.akademie.er_kommt_zu_den_profis").to_string()), (crate::id("abwarten").to_string(), crate::inhalt::t("ui.akademie.noch_ein_bisschen_u23").to_string())];
            self.msg(1, crate::inhalt::t("meldung.absender.jugendleiter"), &crate::tf!("ui.akademie.ist_bereit_fuer_die_profis", n), &crate::tf!("ui.akademie.ist_in_der_u23_kaum", n), Action::Wahl { art: crate::id("befoerderung").into(), bezug: vec![p], optionen: opts });
        }
    }
    pub fn akademie_antwort(&mut self, art: &str, p: usize, key: &str) -> String {
        match (art, key) {
            ("befoerderung", "befoerdern") => { self.promote(p); crate::inhalt::t("ui.akademie.er_gehoert_jetzt_zum_profikader").into() }
            ("eltern", "spielzeit") => { self.world.players[p].ps.u23_stamm = true; self.moral_wirkung(p, 3.0, 0, crate::inhalt::t("ui.akademie.zusage_an_die_eltern_er")); crate::inhalt::t("ui.akademie.die_eltern_sind_beruhigt").into() }
            ("eltern", "geduld") => { self.moral_wirkung(p, -2.0, 0, crate::inhalt::t("ui.akademie.die_eltern_sind_enttaeuscht")); self.beziehung(p, -2.0, crate::inhalt::t("ui.akademie.die_eltern_sind_enttaeuscht")); crate::inhalt::t("ui.akademie.die_eltern_sind_enttaeuscht_2").into() }
            ("eltern", "leihe") => match self.leihe_abgeben(p) { Ok(()) => crate::inhalt::t("ui.leihe.auf_liste").into(), Err(e) => e },
            _ => crate::inhalt::t("ui.akademie.notiert").into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jahrgang_und_entwicklung() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        let vor = c.world.clubs[c.own].squad.clone().iter().filter(|&&p| c.world.players[p].youth > 0).count();
        c.akademie_jahrgang();
        let nach = c.world.clubs[c.own].squad.clone().iter().filter(|&&p| c.world.players[p].youth > 0).count();
        assert!(nach <= 40 && nach >= vor);
        let jung: Vec<usize> = c.world.clubs[c.own].squad.clone().into_iter().filter(|&p| c.world.players[p].youth > 0).collect();
        let s0: f32 = jung.iter().map(|&p| c.world.players[p].ovr).sum();
        for _ in 0..20 { c.world.day += 7; c.akademie_woche(); c.entwicklung_woche(); c.world.develop_week(c.own); }
        let s1: f32 = jung.iter().map(|&p| c.world.players[p].ovr).sum();
        assert!(s1 > s0, "the youth grows: {s0} → {s1}");
        assert_eq!(c.akademie_plaetze().0 >= 20, true);
    }
}
