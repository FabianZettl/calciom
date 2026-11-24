//! Reasons of an injury (pesm/core/entwicklung/medizin_gruende.py): load, tiredness, age, match share, proneness, history
//! and bad luck, from the same quantities the risk uses; and the physio's estimate of the time out. No reason is made up
//! from the result: chance stays visible as bad luck.
use crate::game::*;
use crate::inhalt::t;

/// Smaller contributions are not worth mentioning.
const MINDEST: f32 = 0.05;

impl Career {
    /// The physio's report for an injury of `p` (`spiel`: in a match): the main reasons, then the estimate of the days out.
    pub fn verletzung_gruende(&mut self, p: usize, spiel: bool, tage: u16) -> String {
        let abs = self.abs();
        let pl = &self.world.players[p];
        let mut us: Vec<(f32, String)> = vec![];
        if abs < pl.ent.rueckkehr_bis { us.push((0.5, t("bg.entscheidung.verl_rueckkehr").into())); }
        let hart = pl.program == 2 || self.hard_streak >= 2;
        if hart { us.push((if pl.program == 2 && self.hard_streak >= 2 { 0.6 } else { 0.35 }, t("bg.entscheidung.verl_belastung").into())); }
        let m = 1.5 * (100.0 - pl.fit) / 100.0;
        if m >= MINDEST && pl.fit < 92.0 { us.push((m, crate::tf!("bg.entscheidung.verl_muedigkeit", format!("{:.0}", pl.fit)))); }
        if pl.age > 29 { us.push(((pl.age as f32 - 29.0) * 0.08, crate::tf!("bg.entscheidung.verl_alter", pl.age))); }
        if spiel {
            let spiele = self.results.len().max(1) as f32;
            let anteil = (pl.apps as f32 / spiele).min(1.0);
            if 0.4 * anteil >= MINDEST && pl.apps >= 3 { us.push((0.4 * anteil, crate::tf!("bg.entscheidung.verl_spieldichte", format!("{:.0}", anteil * 100.0)))); }
        }
        if crate::pages::robustheit(pl).is_some_and(|r| r < 45.0) { us.push((0.3, t("bg.entscheidung.verl_anfaellig").into())); }
        let vorher = pl.ent.ausfaelle.iter().filter(|&&d| d + 365 > abs).count();
        if vorher > 0 { us.push((0.12 * vorher.min(5) as f32, crate::tf!("bg.entscheidung.verl_vorgeschichte", vorher))); }
        us.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        us.truncate(3);
        us.push((0.0, t("bg.zufall.pech").into()));
        // the estimate: a better medical team narrows the span (Python: prognose_streuung of the department)
        let q = self.medizinqualitaet();
        let streu = (0.5 - 0.3 * q / 100.0).clamp(0.15, 0.5);
        let von = ((tage as f32) * (1.0 - streu / 2.0)).round().max(1.0) as u16;
        let bis = ((tage as f32) * (1.0 + streu / 2.0)).round() as u16;
        let name = if pl.inj_name.is_empty() { t("en4.ursache.verletzung").to_string() } else { pl.inj_name.clone() };
        let mut dauer = vec![crate::tf!("bg.entscheidung.dauer_art", name, (tage as f32 * 0.6).round(), (tage as f32 * 1.5).round(), von, bis)];
        dauer.push(t(if streu >= 0.4 { "bg.entscheidung.dauer_spanne_weit" } else if streu <= 0.25 { "bg.entscheidung.dauer_spanne_eng" } else { "bg.entscheidung.dauer_spanne_mittel" }).into());
        let kuerzer = (1.0 - self.medizin_faktor()) * 100.0;
        if kuerzer >= 2.0 { dauer.push(crate::tf!("bg.entscheidung.dauer_abteilung_kuerzer", format!("{kuerzer:.0}"))); }
        self.world.players[p].ent.ausfaelle.push(abs);
        self.world.players[p].ent.ausfaelle.retain(|&d| d + 730 > abs);
        format!("{}\n{}\n{}", t("med.gruende.kopf"), us.iter().map(|u| format!("· {}", u.1)).collect::<Vec<_>>().join("\n"), dauer.join(" ")) // roh
    }
}


// ---------------------------------------------------------------- medical department v1 (core/entwicklung/medizin.py, medizin.toml)
fn mk(k: &str) -> f32 { crate::regeln::f("medizin", k) }
const PHASEN: [&str; 4] = [crate::id("behandlung"), crate::id("aufbau"), crate::id("mannschaftstraining"), crate::id("einsatzbereit")];

/// The state of a current injury: total days, days left, phase (index, rest in it), relapse factor when playing now.
pub struct Stand { pub gesamt: u16, pub rest: u16, pub phase: usize, pub phase_rest: u16, pub rueckfall: f32 }

/// Phase of the rehab from the total and the days left (medizin.py reha_phase): bounds by shares, every phase a minimum
/// of days (very short injuries skip phases).
pub fn reha_phase(gesamt: u16, rest: u16) -> (usize, u16) {
    let ph = crate::regeln::wert("medizin", "reha.phasen").as_array().unwrap();
    let g = gesamt.max(1) as f32;
    let vergangen = (g - rest as f32).clamp(0.0, g);
    let mind: f32 = ph.iter().map(|p| p["mindest"].as_f64().unwrap_or(0.0) as f32).sum();
    let mut grenzen = vec![];
    let mut t = 0.0;
    for p in ph {
        let (a, m) = (p["anteil"].as_f64().unwrap_or(0.25) as f32, p["mindest"].as_f64().unwrap_or(0.0) as f32);
        t += if g >= mind { m.max(g * a) } else { g * a };
        grenzen.push(t);
    }
    let skal = g / grenzen[grenzen.len() - 1];
    for x in grenzen.iter_mut() { *x *= skal; }
    for (i, b) in grenzen.iter().enumerate() {
        if vergangen < b - 1e-6 || i == grenzen.len() - 1 { return (i, (b - vergangen).round().max(0.0) as u16); }
    }
    (ph.len() - 1, 0)
}

/// Strain light (medizin.py ampel): points 0…1 from tiredness, minutes and match density of 14 days, proneness; drivers biggest first.
pub fn ampel(kondition: f32, belastung: f32, minuten: f32, spiele: f32, anfaellig: f32) -> (f32, u8, Vec<(&'static str, f32)>) {
    let muede = (0.5 * (100.0 - kondition) / 60.0 + 0.5 * belastung / 100.0).clamp(0.0, 1.0);
    let teile = [(crate::id("muedigkeit"), muede), (crate::id("minuten"), (minuten / mk("ampel.minuten_voll")).clamp(0.0, 1.0)), (crate::id("dichte"), (spiele / mk("ampel.dichte_voll")).clamp(0.0, 1.0)), (crate::id("anfaellig"), ((anfaellig - 0.6) / 0.9).clamp(0.0, 1.0))];
    let mut b: Vec<(&'static str, f32)> = teile.iter().map(|(n, v)| (*n, mk(&format!("ampel.gewichte.{n}")) * v)).collect(); // roh
    let punkte: f32 = b.iter().map(|x| x.1).sum();
    b.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
    (punkte, if punkte >= mk("ampel.rot_ab") { 2 } else if punkte >= mk("ampel.gelb_ab") { 1 } else { 0 }, b)
}

impl Career {
    /// Matches of the own club in the last 14 days (league and cups).
    pub fn spiele_14(&self) -> usize {
        let (d, own) = (self.world.day, self.own);
        let rein = |f: &crate::world::Fixture| (f.h == own || f.a == own) && f.hg >= 0 && f.day < d && f.day + mk("ampel.fenster_tage") as u32 >= d;
        self.world.leagues.iter().map(|l| l.fixtures.iter().filter(|f| rein(f)).count()).sum::<usize>() + self.world.cups.iter().map(|c| c.fx.iter().filter(|x| rein(&x.f)).count()).sum::<usize>()
    }
    /// The light of one player: (points, 0 green / 1 yellow / 2 red, drivers).
    pub fn ampel_von(&self, p: usize) -> (f32, u8, Vec<(&'static str, f32)>) {
        let pl = &self.world.players[p];
        let n = self.spiele_14();
        let minuten: f32 = pl.ps.einsatz.iter().rev().take(n).sum::<f32>() * 90.0;
        let belastung = match pl.program { 1 => 60.0, 2 => 90.0, _ => 35.0 } + if self.hard_streak >= 2 { 10.0 } else { 0.0 };
        // proneness as a factor around 1 (0.6 robust … 1.5 prone) from the robustness of the profile
        let anf = crate::pages::robustheit(pl).map(|r| 1.5 - 0.9 * r / 100.0).unwrap_or(1.0);
        // the history counts too: every injury of the last twelve months adds (changeable in medizin.toml [ampel])
        let jahr = self.world.season * 365 + self.world.day;
        let zuletzt = pl.profil.verletzungen.iter().filter(|v| jahr.saturating_sub(v.0 * 365 + v.1) <= 365).count() as f32;
        let anf = anf * (1.0 + (mk("ampel.historie_je_verletzung") * zuletzt).min(mk("ampel.historie_hoechstens")));
        ampel(pl.fit, belastung, minuten, n as f32, anf)
    }
    /// The current injury of an own player.
    pub fn verletzung_stand(&self, p: usize) -> Option<Stand> {
        let pl = &self.world.players[p];
        if pl.inj == 0 { return None; }
        let gesamt = pl.ent.inj_gesamt.max(pl.inj);
        let (phase, phase_rest) = reha_phase(gesamt, pl.inj);
        let rueckfall = crate::regeln::wert("medizin", "reha.rueckfall")[PHASEN[phase]].as_f64().unwrap_or(1.0) as f32;
        Some(Stand { gesamt, rest: pl.inj, phase, phase_rest, rueckfall })
    }
    /// The physio's estimate (von, bis) in days around the truth; a better medical team narrows it (medizin.py prognose).
    pub fn prognose(&self, p: usize) -> (u16, u16) {
        let pl = &self.world.players[p];
        let s = (0.5 - 0.35 * self.medizinqualitaet() / 100.0).clamp(0.15, 0.5) * (1.0 - mk("prognose.diagnostik_je_stufe") * (self.wi.stufen[4].max(1) - 1) as f32);
        let versatz = (crate::mensch::zufall(&[crate::id("prognose"), &pl.id.to_string(), &pl.ent.inj_beginn.to_string()]) - 0.5) * 2.0 * mk("prognose.verschiebung") * s;
        let t = pl.inj as f32;
        let von = (t * (1.0 - s + versatz)).round().max(1.0) as u16;
        (von, ((t * (1.0 + s + versatz)).round() as u16).max(von + 1))
    }
    /// Daily: notes the total of a new injury, relapses of those back too early, the fading effect of an injection.
    pub fn medizin_tag(&mut self) {
        let abs = self.abs();
        for p in self.world.clubs[self.own].squad.clone() {
            let pl = &mut self.world.players[p];
            if pl.inj > 0 && (pl.ent.inj_gesamt == 0 || pl.inj > pl.ent.inj_gesamt) { pl.ent.inj_gesamt = pl.inj; pl.ent.inj_beginn = abs; }
            if pl.inj == 0 { pl.ent.inj_gesamt = 0; }
            if pl.ent.fitm_bis > abs { pl.fit = pl.fit.min(100.0 - mk("fitspritze.kondition_abzug")); }
            // relapse: an extra daily risk while the factor runs (rueckfall_basis × factor)
            if pl.inj == 0 && pl.ent.rueck_bis > abs {
                let r = crate::mensch::zufall(&[crate::id("rueckfall"), &pl.id.to_string(), &abs.to_string()]);
                if r < mk("reha.rueckfall_basis") * pl.ent.rueck_faktor {
                    let tage = ((pl.ent.rueck_tage.max(3) as f32) * (1.0 + mk("reha.rueckfall_verlaengerung"))).round() as u16;
                    pl.inj = tage;
                    pl.ent.rueck_bis = 0;
                    let n = pl.name.clone();
                    let iname = pl.inj_name.clone();
                    let text = format!("{}\n\n{}", crate::tf!("ui.game.hat_sich_verletzt_er_faellt", n, iname, tage), t("bg.entscheidung.verl_rueckfall"));
                    self.msg(1, t("med.vg.absender"), &crate::tf!("ui.game.verletzung", n), &text, Action::None);
                    self.meldung_art(crate::pages::Art::Verletzung, vec![p]);
                }
            }
        }
    }
    fn rueckfall_merken(&mut self, p: usize, faktor: f32, tage: u32) {
        let abs = self.abs();
        let pl = &mut self.world.players[p];
        pl.ent.rueck_bis = abs + tage;
        pl.ent.rueck_faktor = faktor;
        pl.ent.rueck_tage = pl.ent.inj_gesamt.max(pl.inj);
    }
    /// „Play him now“: the injury ends, the relapse factor of the current phase holds for `rueckfall_tage` days.
    pub fn frueh_einsetzen(&mut self, p: usize) -> Result<String, String> {
        let st = self.verletzung_stand(p).ok_or_else(|| t("ui.wirtschaft.das_angebot_gibt_es_nicht").to_string())?;
        let tage = mk("reha.rueckfall_tage") as u32;
        let mut g = vec![crate::tf!("bg.entscheidung.arzt_frueh", t(&format!("med.phase.{}", PHASEN[st.phase])), st.rest)]; // roh
        if st.rueckfall > 1.0 { self.rueckfall_merken(p, st.rueckfall, tage); g.push(crate::tf!("bg.entscheidung.arzt_rueckfall", format!("{:.1}", st.rueckfall), tage)); }
        self.world.players[p].inj = 0;
        self.world.players[p].ent.inj_gesamt = 0;
        Ok(g.join(" "))
    }
    /// Rehab programme: shorter by a share (more with the medical centre), costs money once per injury, then a slightly higher relapse risk.
    pub fn reha_wirkung(&self, p: usize) -> Option<(u16, u16, u32, f64)> {
        let st = self.verletzung_stand(p)?;
        if (st.rest as f32) < mk("reha_programm.mindest_ausfall_tage") || self.world.players[p].ent.reha_bei == self.world.players[p].ent.inj_beginn + 1 { return None; }
        let anteil = mk("reha_programm.verkuerzung_anteil") + mk("reha_programm.verkuerzung_je_stufe_reha") * (self.wi.stufen[4].max(1) - 1) as f32;
        let v = ((st.rest as f32 * anteil).round() as u16).clamp(1, mk("reha_programm.verkuerzung_hoechstens_tage") as u16);
        let neu = st.rest - v;
        let wochen = (neu as u32).div_ceil(7).min(mk("reha_programm.kosten_wochen_hoechstens") as u32);
        Some((v, neu, wochen, wochen as f64 * mk("reha_programm.kosten_je_woche") as f64 * self.skala()))
    }
    pub fn reha_programm(&mut self, p: usize) -> Result<String, String> {
        let (v, neu, _, kosten) = self.reha_wirkung(p).ok_or_else(|| t("ui.wirtschaft.das_angebot_gibt_es_nicht").to_string())?;
        self.book(t("med.akt.reha"), -kosten);
        let rt = mk("reha_programm.rueckfall_tage") as u32;
        let pl = &mut self.world.players[p];
        pl.inj = neu;
        pl.ent.reha_bei = pl.ent.inj_beginn + 1;
        self.rueckfall_merken(p, mk("reha_programm.rueckfall_faktor"), neu as u32 + rt);
        Ok(format!("{} {}", crate::tf!("bg.entscheidung.arzt_reha", v, crate::world::money(kosten).trim_end_matches(" €")), crate::tf!("bg.entscheidung.arzt_rueckfall", format!("{:.1}", mk("reha_programm.rueckfall_faktor")), rt)))
    }
    fn fits_saison(&self, p: usize) -> u8 { let e = &self.world.players[p].ent; if e.fits.0 == self.world.season { e.fits.1 } else { 0 } }
    /// Injection: only up to 28 days left, at most 3 a season; fit at once, relapse ×3 for 21 days, condition and form down,
    /// from the second of a season a chance of a lasting point less in pace or stamina.
    pub fn fitspritze_moeglich(&self, p: usize) -> bool { self.verletzung_stand(p).is_some_and(|s| (s.rest as f32) <= mk("fitspritze.nur_bis_tage")) && (self.fits_saison(p) as f32) < mk("fitspritze.limit_saison") }
    pub fn fitspritze(&mut self, p: usize) -> Result<String, String> {
        if !self.fitspritze_moeglich(p) { return Err(t("ui.wirtschaft.das_angebot_gibt_es_nicht").into()); }
        let n = self.fits_saison(p) + 1;
        let abs = self.abs();
        let (rf, rt, lt) = (mk("fitspritze.rueckfall_faktor"), mk("fitspritze.rueckfall_tage") as u32, mk("fitspritze.leistung_tage") as u32);
        self.rueckfall_merken(p, rf, rt);
        let season = self.world.season;
        let pl = &mut self.world.players[p];
        pl.inj = 0;
        pl.ent.inj_gesamt = 0;
        pl.ent.fits = (season, n);
        pl.ent.fitm_bis = abs + lt;
        pl.form = (pl.form - mk("fitspritze.form_abzug")).max(-5.0);
        let mut g = vec![crate::tf!("bg.entscheidung.arzt_fit", n, mk("fitspritze.limit_saison")), crate::tf!("bg.entscheidung.arzt_rueckfall", format!("{rf:.1}"), rt), crate::tf!("bg.entscheidung.arzt_fit_leistung", lt)];
        if n >= 2 && crate::mensch::zufall(&[crate::id("dauerschaden"), &pl.id.to_string(), &abs.to_string()]) < mk("fitspritze.dauerschaden_je_nutzung") * (n - 1) as f32 && pl.attrs.len() == 25 {
            let felder = [10usize, 13, 11, 16];
            let i = felder[(crate::mensch::zufall(&[crate::id("dauerfeld"), &pl.id.to_string(), &abs.to_string()]) * 4.0) as usize % 4];
            pl.attrs[i] = (pl.attrs[i] - mk("fitspritze.dauerschaden_punkte")).max(40.0);
            pl.ovr = crate::pesdb::ovr(&pl.attrs, crate::pesdb::pos_index(&pl.pos)).clamp(30.0, 99.0);
            g.push(crate::tf!("bg.entscheidung.arzt_dauerschaden", n));
        }
        Ok(g.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reha_phasen_und_eingriffe() {
        assert_eq!(reha_phase(20, 20).0, 0);
        assert_eq!(reha_phase(20, 1).0, 3);
        assert!(reha_phase(20, 10).0 >= 1);
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let p = c.squad()[1];
        c.world.players[p].inj = 30;
        c.medizin_tag();
        let (v, b) = c.prognose(p);
        assert!(v < b && v >= 1);
        assert!(!c.fitspritze_moeglich(p), "too long for an injection");
        let (kurz, neu, _, kosten) = c.reha_wirkung(p).unwrap();
        assert!(kurz >= 1 && neu < 30 && kosten > 0.0);
        c.reha_programm(p).unwrap();
        assert!(c.reha_wirkung(p).is_none(), "once per injury");
        c.world.players[p].inj = 10;
        assert!(c.fitspritze_moeglich(p));
        c.fitspritze(p).unwrap();
        assert_eq!(c.world.players[p].inj, 0);
        assert!(c.world.players[p].ent.rueck_bis > c.abs());
        let (_, f, d) = c.ampel_von(p);
        assert!(f <= 2 && d.len() == 4);
    }
    #[test]
    fn gruende_aus_der_lage() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let p = c.squad()[0];
        c.world.players[p].fit = 60.0;
        c.world.players[p].age = 33;
        let g = c.verletzung_gruende(p, false, 14);
        assert!(g.contains("60") && g.contains("33"), "{g}");
        assert!(g.contains(t("bg.zufall.pech")), "bad luck always stays visible");
        // the second injury within a year names the history
        let g2 = c.verletzung_gruende(p, false, 7);
        assert!(!g2.is_empty() && c.world.players[p].ent.ausfaelle.len() == 2);
    }
}
