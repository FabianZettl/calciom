//! Paket 3m and 3n: the inbox as cases (Mechanik-Übergabe §16: Muss/Soll/Info, rating, deadlines, threads, weekly
//! world summary) and the assistant coach (§17: recommendations, relation, taking over Soll cases at the deadline).
use crate::game::*;
use crate::mensch::{arr, zufall};
use crate::regeln::f;
use crate::world::*;

pub fn stufen_t() -> [&'static str; 4] { [crate::inhalt::t("es.stufe.hintergrund"), crate::inhalt::t("story.prioritaet.gruen"), crate::inhalt::t("chat.dringend"), crate::inhalt::t("es.stufe.eil")] }
pub fn pflicht_t() -> [&'static str; 3] { [crate::inhalt::t("md.ui.prio.1"), crate::inhalt::t("ui.vorgang.soll"), crate::inhalt::t("ui.vorgang.muss")] }

/// A text variant for the co-trainer's advice: by day and player, so it changes (Fabian 23.11.: never the same words).
fn zufall_v(basis: &str, abs: u32, p: usize) -> usize { let n = crate::inhalt::varianten(basis); (crate::mensch::zufall(&["co_rat", basis, &abs.to_string(), &p.to_string()]) * n as f32) as usize % n } // roh

impl Career {
    /// Rate a new message (einstufung.toml / pflicht.toml): Muss, Soll or Info, the level, the deadline and the thread.
    pub fn einstufen(&mut self, m: &mut Msg) {
        let abs = self.abs();
        let (pflicht, pid) = match &m.action {
            // stories are answered by the person or stay unanswered (−3 mood after 14 days), the assistant does not take them
            Action::Wahl { art, bezug, .. } if art.starts_with("story:") => (0, bezug.first().copied()),
            Action::Wahl { art, bezug, .. } => (if [crate::id("konflikt"), crate::id("disziplin"), crate::id("medizin"), crate::id("nachforderung"), crate::id("hijack")].iter().any(|k| art.starts_with(k)) { 2 } else { 1 }, bezug.first().copied()),
            Action::JobOffer { .. } | Action::Graduate { .. } => (2, None),
            Action::TransferOffer { pid, .. } | Action::Expiring { pid } | Action::Complaint { pid } => (1, Some(*pid)),
            Action::Report { pid } => (0, Some(*pid)),
            Action::None => (0, None),
        };
        m.pflicht = pflicht;
        m.frist = abs + match pflicht { 2 => f("medien", "posteingang.frist_tage") as u32, 1 => 5, _ => 0 };
        m.faden = pid.map(|p| format!("{}|{}", self.world.players[p].id, m.cat)).unwrap_or_default();
        let eil = ["Ultimatum", "entlassen", "Medizincheck", "grätscht", "Streik", "geplatzt", "will weg", "Trainingsstreik", "Einsatzstreik", "berät über deine Zukunft"].iter().any(|k| m.title.contains(k)); // roh
        let stamm = pid.map_or(false, |p| self.world.players[p].team == self.own as i32 && (self.world.players[p].ps.rolle <= 3 || self.xi.contains(&p)));
        let mut stufe = if eil { 3 } else { match pflicht { 2 => 2, 1 => 1, _ => 0 } };
        if stamm && stufe < 3 && pflicht > 0 { stufe += 1; }
        if matches!(m.action, Action::TransferOffer { fee, .. } if fee >= 30_000_000.0) && stufe < 3 { stufe += 1; }
        // at most 3 flash messages a day, at most 4 in full priority (medien.toml posteingang.je_tag_hoechstens)
        let heute: Vec<&Msg> = self.inbox.iter().filter(|x| x.day == m.day && x.season == m.season).collect();
        if stufe == 3 && heute.iter().filter(|x| x.stufe == 3).count() >= 3 { stufe = 2; }
        if stufe >= 2 && heute.iter().filter(|x| x.stufe >= 2).count() as f32 >= f("medien", "posteingang.je_tag_hoechstens") { stufe = 1; }
        m.stufe = stufe;
    }
    /// Daily: outdated cases go to the archive, the assistant answers Soll cases at their deadline (cotrainer.py),
    /// Muss cases are never taken over: they escalate.
    pub fn posteingang_tag(&mut self) {
        let abs = self.abs();
        let own = self.own as i32;
        for m in self.inbox.iter_mut() {
            if m.done { continue; }
            let weg = match &m.action {
                Action::TransferOffer { pid, .. } | Action::Complaint { pid } | Action::Expiring { pid } => self.world.players[*pid].team != own,
                Action::Wahl { art, bezug, .. } if !art.starts_with("medizin") && !art.starts_with("nachforderung") && !art.starts_with("hijack") => bezug.iter().any(|&p| self.world.players[p].team != own),
                _ => false,
            };
            if weg { m.done = true; m.read = true; m.body += crate::inhalt::t("ui.vorgang.ueberholt_er_ist_nicht_mehr"); }
        }
        // Fabian 23.11.: nothing stays open for ever. A case left unanswered runs out at its deadline (a week for those
        // without one); the assistant answers it. A must-case first escalates and gets three more days.
        let faellig: Vec<usize> = (0..self.inbox.len()).filter(|&i| {
            let m = &self.inbox[i];
            let offen = !m.done && m.action != Action::None && !matches!(m.action, Action::Report { .. });
            let frist = if m.frist > 0 { m.frist } else { abs_day(m.season, m.day) + 7 } + if m.pflicht == 2 { 3 } else { 0 };
            offen && abs >= frist
        }).collect();
        for m in self.inbox.iter_mut() { if !m.done && m.pflicht == 2 && m.frist > 0 && abs >= m.frist && m.stufe < 3 { m.stufe = 3; m.title = crate::tf!("ui.vorgang.eskaliert", m.title); } }
        for i in faellig { self.cotrainer_uebernimmt(i); }
    }
    /// The assistant answers a Soll case with his standard line: calm and neutral (cotrainer.py); conversations cost
    /// −1 relation to the player, at most −3 per player in 30 days.
    /// The person hands a case to the assistant (delegate): he answers it at once, in his calm way.
    pub fn delegieren(&mut self, i: usize) { if i < self.inbox.len() && !self.inbox[i].done { self.cotrainer_uebernimmt(i); } }
    fn cotrainer_uebernimmt(&mut self, i: usize) {
        let a = self.inbox[i].action.clone();
        match a {
            Action::Wahl { art, bezug, optionen } if art.starts_with("gespraech") => {
                let thema = art.split_once(':').map(|x| x.1).unwrap_or("");
                // the first answer with a warm or matter-of-fact tone
                let opts = arr("gespraeche", &format!("thema.{thema}.optionen"));
                let k = optionen.iter().position(|o| opts.iter().any(|x| x["id"].as_str() == Some(&o.0) && matches!(x["ton"].as_str(), Some("warm") | Some("sachlich")) && x["versprechen"].is_null())).unwrap_or(0);
                self.answer(i, true, k as u8);
                let p = bezug[0];
                let abs = self.abs();
                let n = self.world.players[p].name.clone();
                let schon: f32 = self.mensch.gruende.iter().filter(|g| g.bezug == n && g.grund.starts_with("Der Co-Trainer") && abs < g.abs + 30).map(|g| g.wert).sum(); // roh
                if schon > -3.0 { self.beziehung(p, -1.0 * (1.0 - 0.3 * (self.co_staerke() - 50.0) / 50.0), crate::inhalt::t("ui.vorgang.der_co_trainer_hat_das")); }
                self.inbox[i].body += crate::inhalt::t("ui.vorgang.vom_co_trainer_beantwortet");
            }
            Action::TransferOffer { .. } => { self.answer(i, false, 0); self.inbox[i].body += crate::inhalt::t("ui.vorgang.ohne_antwort_abgelaufen"); }
            // a job offer runs out, an expiring contract is put off, a complaint gets the matter-of-fact answer, a graduate is kept
            Action::JobOffer { .. } => { self.answer(i, false, 0); self.inbox[i].body += crate::inhalt::t("ui.vorgang.ohne_antwort_abgelaufen"); }
            Action::Expiring { .. } => { self.answer(i, true, 2); self.inbox[i].body += crate::inhalt::t("ui.vorgang.vom_co_trainer_beantwortet"); }
            Action::Complaint { .. } => { self.answer(i, true, 1); self.inbox[i].body += crate::inhalt::t("ui.vorgang.vom_co_trainer_beantwortet"); }
            Action::Graduate { .. } => { self.answer(i, true, 0); self.inbox[i].body += crate::inhalt::t("ui.vorgang.vom_co_trainer_beantwortet"); }
            Action::Wahl { .. } => {
                // the calm answer: the first matter-of-fact option, else the first friendly one, else the first
                let antw = crate::pages::antworten(&self.inbox[i]);
                let k = antw.iter().find(|a| a.3 == 1).or_else(|| antw.iter().find(|a| a.3 == 0)).map(|a| a.1).unwrap_or(0);
                self.answer(i, true, k);
                self.inbox[i].body += crate::inhalt::t("ui.vorgang.vom_co_trainer_beantwortet");
            }
            _ => { self.inbox[i].done = true; self.inbox[i].body += crate::inhalt::t("ui.vorgang.erledigt_durch_den_co_trainer"); }
        }
    }
    // ------------------------------------------------------------ assistant coach (§17, assistent/regeln.toml)
    pub fn co_staerke(&self) -> f32 { self.mensch.co_staerke.max(1.0) }
    /// Relation coach – assistant: start 55, accepted +2, refused −2 (fading over 60 days), +1/−1 per win/loss of the last five.
    pub fn co_beziehung(&self) -> f32 {
        let abs = self.abs();
        let heilt = f("assistent", "beziehung.heilt_tage");
        let s: f32 = self.mensch.co_eintraege.iter().map(|(d, w)| w * (1.0 - (abs.saturating_sub(*d)) as f32 / heilt).max(0.0)).sum();
        let res: f32 = self.results.iter().rev().take(5).map(|r| if r.starts_with('S') { f("assistent", "beziehung.sieg") } else if r.starts_with('N') { f("assistent", "beziehung.niederlage") } else { 0.0 }).sum();
        (f("assistent", "beziehung.start") + s + res).clamp(0.0, 100.0)
    }
    /// Weekly recommendations (from 3 own matches): form, play time, talents, unhappy players, long injuries, gaps.
    pub fn cotrainer_woche(&mut self) {
        let r = |k: &str| f("assistent", k);
        if (self.results.len() as f32) < r("min_spiele") { return; }
        let abs = self.abs();
        let sperre = r("ablauf.tage") as u32;
        self.mensch.co_sperre.retain(|x| abs < x.1);
        let co = self.co_name.clone();
        let mut raete: Vec<(String, usize, String, bool)> = vec![];
        for p in self.squad() {
            let pl = &self.world.players[p];
            if pl.youth > 0 || !pl.ps.an { continue; }
            let anteil = if pl.ps.einsatz.is_empty() { 0.0 } else { pl.ps.einsatz.iter().sum::<f32>() / pl.ps.einsatz.len() as f32 };
            let note = if pl.last.is_empty() { 6.5 } else { pl.last.iter().sum::<f32>() / pl.last.len() as f32 };
            let n = pl.name.clone();
            if (note >= r("form.gut_note") || pl.form >= r("form.gut_form")) && anteil < r("form.wenig_anteil") && pl.inj == 0 { raete.push((crate::id("bringen").into(), p, crate::tf!(&crate::inhalt::variante_key("ui.vorgang.ist_in_form_spielt_aber", zufall_v("ui.vorgang.ist_in_form_spielt_aber", abs, p)), n), false)); }
            else if (note <= r("form.schwach_note") || pl.form <= r("form.tief_form")) && anteil >= r("form.stamm_anteil") && !pl.last.is_empty() { raete.push((crate::id("rausnehmen").into(), p, crate::tf!(&crate::inhalt::variante_key("ui.vorgang.steckt_im_formtief_setz_ihn", zufall_v("ui.vorgang.steckt_im_formtief_setz_ihn", abs, p)), n), false)); }
            else if pl.age as f32 <= r("jugend.max_alter") && pl.pot - pl.ovr >= r("jugend.pot_luecke") && anteil < r("jugend.wenig_anteil") { raete.push((crate::id("talent").into(), p, crate::tf!(&crate::inhalt::variante_key("ui.vorgang.hat_grosses_potenzial_gib_ihm", zufall_v("ui.vorgang.hat_grosses_potenzial_gib_ihm", abs, p)), n), false)); }
            else if pl.morale < r("stimmung.unzufrieden") { raete.push((crate::id("gespraech").into(), p, crate::tf!(&crate::inhalt::variante_key("ui.vorgang.ist_unzufrieden_rede_mit_ihm", zufall_v("ui.vorgang.ist_unzufrieden_rede_mit_ihm", abs, p)), n, if pl.morale < r("stimmung.sehr") { crate::inhalt::t("ui.vorgang.sehr_sogar") } else { "" }), pl.morale < r("stimmung.sehr"))); }
            else if pl.inj as f32 >= r("verletzt.lang_tage") { raete.push((crate::id("verletzt").into(), p, crate::tf!(&crate::inhalt::variante_key("ui.vorgang.faellt_lange_aus_tage_wir", zufall_v("ui.vorgang.faellt_lange_aus_tage_wir", abs, p)), n, pl.inj), false)); }
        }
        let bereit = |teil: usize| self.squad().iter().filter(|&&p| self.world.players[p].available() && self.world.players[p].youth == 0 && part_of(&self.world.players[p].pos) == teil).count() as f32;
        for (teil, k, wort) in [(0usize, "luecken.tor", crate::inhalt::t("aka.pos.tw")), (1, "luecken.abwehr", crate::inhalt::t("ui.vorgang.abwehrspieler")), (2, "luecken.mittelfeld", crate::inhalt::t("ui.vorgang.mittelfeldspieler")), (3, "luecken.angriff", crate::inhalt::t("aka.pos.st"))] {
            if bereit(teil) < r(k) { raete.push((crate::id("luecke").into(), usize::MAX, crate::tf!(&crate::inhalt::variante_key("ui.vorgang.uns_gehen_die_aus_wir", zufall_v("ui.vorgang.uns_gehen_die_aus_wir", abs, teil)), wort), true)); }
        }
        let mut n = 0;
        for (art, p, text, dringend) in raete {
            if n >= 3 { break; }
            let key = format!("{art}|{p}");
            if self.mensch.co_sperre.iter().any(|x| x.0 == key) { continue; }
            self.mensch.co_sperre.push((key, abs + sperre));
            let opts = vec![(crate::id("annehmen").to_string(), crate::inhalt::t("ui.vorgang.einverstanden").to_string()), (crate::id("ablehnen").to_string(), crate::inhalt::t("ui.vorgang.nein_ich_sehe_das_anders").to_string())];
            let bezug = if p == usize::MAX { vec![] } else { vec![p] };
            self.msg(1, &co, &crate::tf!("ui.vorgang.rat_vom_co_trainer", if dringend { crate::inhalt::t("ui.vorgang.dringend") } else { "" }), &text, Action::Wahl { art: format!("rat:{art}:{}", dringend as u8), bezug, optionen: opts });
            n += 1;
        }
    }
    pub fn rat_antwort(&mut self, sub: &str, bezug: &[usize], key: &str) -> String {
        let (art, dringend) = sub.split_once(':').unwrap_or((sub, "0"));
        let faktor = if dringend == "1" { f("assistent", "beziehung.hoch_faktor") } else { 1.0 };
        let abs = self.abs();
        let ja = key == crate::id("annehmen");
        let w = if ja { f("assistent", "beziehung.angenommen") } else { f("assistent", "beziehung.abgelehnt") } * faktor;
        self.mensch.co_eintraege.push((abs, w));
        self.mensch.co_eintraege.retain(|x| abs < x.0 + 60);
        let co = self.co_name.clone();
        self.grund(&co, crate::inhalt::t("ui.vorgang.beziehung_zum_co_trainer"), w, if ja { crate::inhalt::t("ui.vorgang.rat_angenommen") } else { crate::inhalt::t("ui.vorgang.rat_abgelehnt") });
        if !ja { return crate::inhalt::t("ui.vorgang.er_nimmt_es_hin").into(); }
        let Some(&p) = bezug.first() else { return crate::inhalt::t("ui.akademie.notiert").into() };
        match art {
            "bringen" | "talent" => {
                if !self.xi.contains(&p) {
                    let teil = part_of(&self.world.players[p].pos);
                    if let Some(k) = (0..self.xi.len()).filter(|&k| part_of(&self.world.players[self.xi[k]].pos) == teil).min_by(|&a, &b| self.world.players[self.xi[a]].ovr.partial_cmp(&self.world.players[self.xi[b]].ovr).unwrap()) { self.xi[k] = p; }
                }
                crate::inhalt::t("ui.vorgang.er_steht_in_der_naechsten").into()
            }
            "rausnehmen" => { self.xi.retain(|&x| x != p); self.auto_lineup_fill(); crate::inhalt::t("ui.vorgang.er_sitzt_erst_einmal_draussen").into() }
            "gespraech" => { self.anliegen(p, crate::id("form"), true); crate::inhalt::t("ui.vorgang.ein_gespraech_steht_an").into() }
            _ => crate::inhalt::t("ui.akademie.notiert").into(),
        }
    }
    /// Fill a short starting eleven with the best available players of the formation.
    pub fn auto_lineup_fill(&mut self) {
        if self.xi.len() >= 11 { return; }
        let keep = self.xi.clone();
        let (xi, _) = self.world.lineup_slots(self.own, self.eigene_slots(), &keep);
        self.xi = xi.iter().map(|x| x.1).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aufstellung_nach_kriterien() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        c.auto_lineup();
        let alter = |c: &Career| c.xi.iter().map(|&p| c.world.players[p].age as f32).sum::<f32>() / 11.0;
        let a0 = alter(&c);
        c.aufstellung_kriterien = 1 | 1 << 6;
        c.auto_lineup();
        assert!(alter(&c) <= a0, "young players criterion makes the eleven younger");
        c.aufstellung_kriterien = 1 << 2;
        let p = c.xi[3];
        c.world.players[p].fit = 30.0;
        c.auto_lineup();
        assert!(!c.xi.contains(&p), "fitness alone leaves the exhausted out");
    }
    #[test]
    fn soll_uebernimmt_der_co_trainer() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        let p = c.squad()[5];
        c.anliegen(p, "spielzeit", true);
        let i = c.inbox.iter().position(|m| matches!(m.action, Action::Wahl { .. })).unwrap();
        assert_eq!(c.inbox[i].pflicht, 1);
        let b0 = c.world.players[p].ps.bez;
        for _ in 0..6 { c.world.day += 1; c.posteingang_tag(); }
        let i = c.inbox.iter().position(|m| matches!(m.action, Action::Wahl { .. })).unwrap();
        assert!(c.inbox[i].done, "taken over at the deadline");
        assert!(c.inbox[i].body.contains("Co-Trainer"));
        let _ = b0;
        assert!((c.co_beziehung() - 55.0).abs() < 1e-3);
        c.rat_antwort("bringen:0", &[p], "annehmen");
        assert!(c.co_beziehung() > 55.0);
    }
}
