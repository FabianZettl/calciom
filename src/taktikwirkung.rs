//! Taktik-Konzept Teil 3.3/3.4 (konzepte/taktiksystem.md): what the own settings do to the expected goals of both sides
//! in the simulation, with the explainable counter rules (fast forwards behind a high line, pressing against a weak
//! build-up, tired legs) and their sentences for the match report. All together at most ±15 % (taktik.cap).
use crate::game::Career;

fn k(p: &str) -> f32 { crate::regeln::f("taktikwirkung", p) }
/// The mean of one ability (entwicklung.rs NAMEN index) over some players; 0 without values.
fn mittel(c: &Career, ps: &[usize], i: usize) -> f32 {
    let v: Vec<f32> = ps.iter().filter_map(|&p| c.world.players[p].attrs.get(i).copied()).collect();
    if v.is_empty() { 0.0 } else { v.iter().sum::<f32>() / v.len() as f32 }
}

impl Career {
    /// (own goals factor, opponent goals factor, sentences of the counter rules) for today's own match against `gegner`.
    pub fn taktik_wirkung(&self, gegner: usize) -> (f32, f32, Vec<String>) {
        let t = &self.tactic;
        let (mut fe, mut fg) = (1.0f32, 1.0f32);
        let mut saetze = vec![];
        let elf = &self.xi;
        let pos = |p: usize| self.world.players[p].pos.clone();
        let (geg_elf, _) = self.world.lineup(gegner, 0, &[]);
        let geg: Vec<usize> = geg_elf.iter().map(|x| x.1).collect();
        let ballbesitz = t.angriff == 1;
        if ballbesitz { fe *= 1.0 - k("ballbesitz.eigen"); fg *= 1.0 - k("ballbesitz.gegner"); }
        // long balls: more attacks, a little worse; with a real target man better
        if t.aufbau == 0 {
            let ziel = elf.iter().filter(|&&p| pos(p) == "CF").map(|&p| self.world.players[p].attrs.get(7).copied().unwrap_or(0.0)).fold(0.0, f32::max); // roh
            fe *= 1.0 + if ziel >= k("lang.zielspieler_ab") { k("lang.mit_zielspieler") } else { k("lang.ohne") };
            fg *= 1.0 + k("lang.gegner");
        }
        let pressing = t.pressing == 0 && t.abwehr == 0;
        if pressing { fe *= 1.0 + k("pressing.eigen"); fg *= 1.0 + k("pressing.konter"); }
        let stufe = (t.linie as f32 - 5.5) / 2.25;
        fg *= 1.0 + k("linie.je_stufe") * stufe;
        // counter rules (Konzept 3.4), each with its sentence
        let tempo = 10;
        let ivs: Vec<usize> = elf.iter().copied().filter(|&p| pos(p) == "CB").collect();
        let stuermer: Vec<usize> = geg.iter().copied().filter(|&p| matches!(self.world.players[p].pos.as_str(), "CF" | "LWF" | "RWF" | "SS")).collect();
        let schnellster = stuermer.iter().filter_map(|&p| self.world.players[p].attrs.get(tempo).copied()).fold(0.0, f32::max);
        let langsamster = ivs.iter().filter_map(|&p| self.world.players[p].attrs.get(tempo).copied()).fold(f32::MAX, f32::min);
        if stufe >= 0.6 && schnellster > 0.0 && langsamster < f32::MAX && schnellster >= langsamster + k("regel.tempo_abstand") {
            fg *= 1.0 + k("regel.raum_hinter_kette");
            saetze.push(crate::tf!("ui.tw.raum_hinter_kette", self.world.clubs[gegner].name.clone()));
        }
        let mitte = |v: &[usize]| v.iter().copied().filter(|&p| matches!(self.world.players[p].pos.as_str(), "DMF" | "CMF")).collect::<Vec<_>>();
        let (eig_m, geg_m) = (mitte(elf), mitte(&geg));
        if pressing && !geg_m.is_empty() && !eig_m.is_empty() {
            let druck = (mittel(self, &eig_m, 18) + mittel(self, &eig_m, 16)) / 2.0;
            let aufbau = mittel(self, &geg_m, 4);
            if aufbau > 0.0 && aufbau < druck - k("regel.aufbau_abstand") { fe *= 1.0 + k("regel.pressing_gegen_schwach"); saetze.push(crate::inhalt::t("ui.tw.pressing_zerreisst").to_string()); }
            else if aufbau > druck + k("regel.aufbau_abstand") { fg *= 1.0 + k("regel.pressing_gegen_stark"); saetze.push(crate::inhalt::t("ui.tw.aus_pressing_befreit").to_string()); }
        }
        let fit = elf.iter().map(|&p| self.world.players[p].fit).sum::<f32>() / elf.len().max(1) as f32;
        if pressing && fit < k("regel.muede_unter") { fe *= 1.0 - k("regel.muede_eigen"); fg *= 1.0 + k("regel.muede_gegner"); saetze.push(crate::inhalt::t("ui.tw.muede_beine").to_string()); }
        let cap = k(crate::id("cap"));
        (fe.clamp(1.0 - cap, 1.0 + cap), fg.clamp(1.0 - cap, 1.0 + cap), saetze)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn taktik_bleibt_im_deckel() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 7);
        let mut c = Career::new(w, 0, "Test");
        let g = (c.own + 1) % c.world.clubs.len();
        for (angriff, aufbau, pressing, linie) in [(0u8, 0u8, 0u8, 10), (1, 1, 1, 1), (0, 1, 0, 5), (1, 0, 1, 8)] {
            c.tactic.angriff = angriff; c.tactic.aufbau = aufbau; c.tactic.pressing = pressing; c.tactic.abwehr = pressing; c.tactic.linie = linie;
            let (fe, fg, _) = c.taktik_wirkung(g);
            assert!((0.85..=1.15).contains(&fe) && (0.85..=1.15).contains(&fg), "{fe} {fg}");
        }
        // a high line lets the opponent score more than a deep one
        c.tactic.linie = 10; let hoch = c.taktik_wirkung(g).1;
        c.tactic.linie = 1; let tief = c.taktik_wirkung(g).1;
        assert!(hoch > tief);
    }
}

#[cfg(test)]
mod plan_b_tests {
    use crate::game::Career;
    #[test]
    fn plan_b_greift_im_spiel() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 7);
        let mut c = Career::new(w, 0, "Test");
        while c.today_match().is_none() { c.end_day().unwrap(); }
        c.plan_b = crate::game::PlanB { an: true, minute: 46, wenn: 1, mentalitaet: 1 };
        let mut gegriffen = 0;
        for _ in 0..40 { let r = c.prepare_match(None); if r.reason.contains(&crate::tf!("ui.planb.bericht", 46, crate::inhalt::t("ui.live.ment.2"))) { gegriffen += 1; assert_eq!(r.hg as usize, r.events.iter().filter(|e| e.side == 0 && e.kind <= 1).count()); } }
        assert!(gegriffen > 0, "Plan B never kicked in");
    }
}
