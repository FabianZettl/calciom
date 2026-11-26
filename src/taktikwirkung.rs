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
    /// Konzept Taktik 5.3: did the tactic work? Intention → measured value → verdict, from the match statistics.
    pub fn taktik_bilanz(&self, res: &crate::world::MatchResult) -> Vec<(String, bool)> {
        use crate::statistik::*;
        if res.stat.is_empty() { return vec![]; }
        let side = if res.h == self.own { 0 } else if res.a == self.own { 1 } else { return vec![] };
        let t = &self.tactic;
        let b = |k: &str| crate::regeln::f("taktikwirkung", &format!("bilanz.{k}"));
        let mut v = vec![];
        if t.angriff == 1 {
            let bb = if side == 0 { res.possession } else { 100 - res.possession } as f32;
            v.push((crate::tf!("ui.tb.ballbesitz", format!("{bb:.0}"), b(crate::id("ballbesitz"))), bb >= b(crate::id("ballbesitz"))));
        } else {
            let x = res.summe(side, PROG_LAEUFE) as f32;
            v.push((crate::tf!("ui.tb.konter", x, b(crate::id("konter"))), x >= b(crate::id("konter"))));
        }
        if t.breite == 0 { let x = res.summe(side, FLANKEN) as f32; v.push((crate::tf!("ui.tb.fluegel", x, b(crate::id("flanken"))), x >= b(crate::id("flanken")))); }
        if t.pressing == 0 { let x = (res.summe(side, TACKL) + res.summe(side, ABGEF)) as f32; v.push((crate::tf!("ui.tb.pressing", x, b(crate::id("pressing"))), x >= b(crate::id("pressing")))); }
        if t.haltung == 0 { let x = res.summe(1 - side, AUFS_TOR) as f32; v.push((crate::tf!("ui.tb.sicher", x, b("gegen_aufs_tor")), x <= b("gegen_aufs_tor"))); }
        v
    }
    /// Konzept Taktik 5.1: the note for FL27 — what to set in the game's tactics before kick-off (at most 6 lines).
    pub fn taktikzettel(&self) -> Vec<String> {
        let t = &self.tactic;
        let w = |k: &str| crate::inhalt::t(k).to_string();
        let mut z = vec![crate::tf!("ui.tz.formation", crate::world::FORMATIONS[self.formation])];
        z.push(crate::tf!("ui.tz.stil", w(if t.angriff == 0 { "ui.tz.konter" } else { "ui.tz.ballbesitz" }), w(if t.pressing == 0 { "ui.tz.pressing_vorne" } else { "ui.tz.pressing_ruhig" }), t.linie, t.kompakt));
        z.push(w(if t.breite == 0 { "ui.tz.breit" } else { "ui.tz.eng" }));
        let name = |p: Option<usize>| p.map(|p| self.world.players[p].name.clone()).unwrap_or_else(|| "–".into());
        z.push(crate::tf!("ui.tz.rollen", name(self.kapitaen()), name(self.schuetze(0))));
        if self.plan_b.an { z.push(w("ui.tz.planb")); }
        z.truncate(6);
        z
    }

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
        // the CalcioM sliders and switches that the game has no field for (taktik9.rs, spielideen.toml [wirkung])
        {
            let w = |x: &str| crate::regeln::f("spielideen", &format!("wirkung.{x}"));
            let st = |n: &str| self.t9.r(n) as f32 - 3.0;
            fe *= 1.0 + w("risiko_eigen") * st("risiko"); // roh
            fg *= 1.0 + w("risiko_gegner") * st("risiko") + w("haerte_gegner") * st("haerte"); // roh
            let technik = mittel(self, elf, 2);
            let frei = st("freiheit"); // roh
            if frei > 0.0 { fe *= 1.0 + w("freiheit_eigen") * frei * if technik >= w("freiheit_technik_ab") { 1.0 } else { -1.0 }; }
            if self.t9.s("abseitsfalle") { fg *= 1.0 + w("abseitsfalle_gegner"); } // roh
            if self.t9.s("absichern") { fg *= 1.0 + w("absichern_gegner"); fe *= 1.0 + w("absichern_eigen"); } // roh
            // advanced instructions and the formations with and without the ball (Fabian 24.11.)
            for &id in &self.anweisungen {
                let k = crate::taktik9::anweisung_key(id);
                if k.is_empty() { continue; }
                if let Some(a) = crate::regeln::wert_oder("spielideen", &format!("anweisung.{k}")).and_then(|v| v.as_array().cloned()) {
                    fe *= 1.0 + a.first().and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                    fg *= 1.0 + a.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                }
            }
            let teil = |f: usize, t: usize| crate::world::formation_slots(f).iter().filter(|s| crate::world::part_of(s) == t).count() as f32;
            if let Some(fb) = self.formation_ball { fe *= 1.0 + crate::regeln::f("spielideen", "anweisung.mit_ball_je_angreifer") * (teil(fb, 3) - teil(self.formation, 3)); }
            if let Some(fg2) = self.formation_gegen { fg *= 1.0 + crate::regeln::f("spielideen", "anweisung.gegen_ball_je_verteidiger") * (teil(fg2, 1) - teil(self.formation, 1)); }
        }
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
        // the coach's own style (trainerprofil.rs): when the tactics are his kind of football, he gets it across
        let passt = match self.trainerprofil.stil {
            // the switches as on the tactics board: angriff 0 counter, pressing 0 hard, breite 0 wide
            1 => t.angriff == 0 && t.haltung <= 1,                  // counter
            2 => t.pressing == 0 && t.linie >= 6,                   // pressing
            3 => t.kompakt >= 7 && t.linie <= 4,                    // compact
            4 => t.breite == 0,                                     // wing play
            _ => t.angriff == 1 || t.aufbau == 1,                   // possession
        };
        if passt {
            fe *= 1.0 + k("trainerstil.eigen");
            if matches!(self.trainerprofil.stil, 1 | 3) { fg *= 1.0 - k("trainerstil.gegner"); }
            saetze.push(crate::inhalt::t("ui.tw.trainerstil").to_string());
        }
        let cap = k(crate::id("cap"));
        (fe.clamp(1.0 - cap, 1.0 + cap), fg.clamp(1.0 - cap, 1.0 + cap), saetze)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn taktik_bleibt_im_deckel() {
        let w = crate::world::testwelt(7);
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
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        while c.today_match().is_none() { c.end_day().unwrap(); }
        c.plan_b = crate::game::PlanB { an: true, minute: 46, wenn: 1, mentalitaet: 1 };
        let mut gegriffen = 0;
        for _ in 0..40 { let r = c.prepare_match(None); if r.reason.contains(&crate::tf!("ui.planb.bericht", 46, crate::inhalt::t("ui.live.ment.2"))) { gegriffen += 1; assert_eq!(r.hg as usize, r.events.iter().filter(|e| e.side == 0 && e.kind <= 1).count()); } }
        assert!(gegriffen > 0, "Plan B never kicked in");
    }
}
