//! Paket 3f: transfers with drama, loans and purchase options (Mechanik-Übergabe §9; core/transfers/werte.py,
//! verhandlung.py, drama.py). The market value stays the world's (sim/transfers.py, calibrated); asking price,
//! sale readiness, offers for own players, drama, medical, aftermath, strike and loans follow the core.
use crate::game::*;
use crate::mensch::{cw, zufall, LOYAL};
use crate::world::*;
use serde::{Deserialize, Serialize};

/// drama.py KONF (Startwerte of the original).
pub const DRAMA_TAG: f32 = 0.035;
pub const DEADLINE_FAKTOR: f32 = 4.0;
pub const GATE: f32 = 0.09;
pub const GATE_DEADLINE: f32 = 3.0;
pub const HIJACK_TAGE: u32 = 4;
pub const MED_BEFUND: f32 = 0.07;
pub const MED_DURCHFALL: f32 = 0.28;
pub const MED_AUSFALL_JE_TAG: f32 = 0.012;
pub const MED_ALTER_AB: f32 = 31.0;
pub const MED_ALTER_JE_JAHR: f32 = 0.08;
pub const MED_DECKEL: f32 = 0.7;
pub const MED_NACHLASS_CHANCE: f32 = 0.62;
pub const STREIK_AB: f32 = 60.0;
/// Influence on the dressing room by role (drama.py EINFLUSS).
pub const EINFLUSS: [f32; 7] = [100.0, 85.0, 75.0, 45.0, 25.0, 10.0, 5.0];
/// Loans (§9): fee 15 % of the market value, the borrowing club pays 70 % of the wage.
pub const LEIH_GEBUEHR: f64 = 0.15;
pub const LEIH_ANTEIL: f32 = 0.7;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Leihe { pub von: usize, pub bis: u32, pub anteil: f32, pub option: f64, #[serde(default)] pub spielzeit: bool, #[serde(default)] pub grund: String, /// Strength when the loan began (monthly report). 
#[serde(default)] pub start: f32 }

/// Medical risk and verdict (drama.py medizin_risiko / medizin_urteil / medizin_abschlag).
pub fn medizin(ausfall_tage: u16, alter: u8, u: f32) -> (&'static str, f32, f32) {
    let p = (MED_BEFUND * (1.0 + ausfall_tage as f32 * MED_AUSFALL_JE_TAG) * (1.0 + (alter as f32 - MED_ALTER_AB).max(0.0) * MED_ALTER_JE_JAHR)).min(MED_DECKEL);
    let urteil = if u < p * MED_DURCHFALL { crate::id("durchgefallen") } else if u < p { crate::id("auffaellig") } else { crate::id("bestanden") };
    (urteil, p, (0.04 + 0.5 * p).min(0.2))
}

impl Career {
    /// Role of a player in his club's squad by rating: 0 star/regular (best 11), 1 rotation (12–18), 2 reserve.
    fn kaderrolle(&self, p: usize) -> u8 {
        let pl = &self.world.players[p];
        if pl.team < 0 { return 3; }
        let rang = self.world.clubs[pl.team as usize].squad.iter().filter(|&&q| self.world.players[q].ovr > pl.ovr).count();
        if rang < 11 { 0 } else if rang < 18 { 1 } else { 2 }
    }
    /// Readiness of the club to sell (own decision from §9: listed 0.9, wish to leave 0.8, by squad role 0.3/0.5/0.7;
    /// one of its three best with more than two years left is not for sale).
    pub fn verkaufsbereitschaft(&self, p: usize) -> f32 {
        let pl = &self.world.players[p];
        if pl.team < 0 { return 1.0; }
        if pl.listed { return 0.9; }
        if pl.ps.wunsch { return 0.8; }
        let rang = self.world.clubs[pl.team as usize].squad.iter().filter(|&&q| self.world.players[q].ovr > pl.ovr).count();
        if rang < 3 && pl.contract_end >= self.world.season + 2 { return 0.05; }
        [0.3, 0.5, 0.7, 1.0][self.kaderrolle(p) as usize]
    }
    /// Asking fee (werte.py:75): market value × role (Stamm 1.25, Rotation 1.05, Reserve 0.85), × 0.7 at ≤ 6 months left.
    pub fn abloese_forderung(&self, p: usize) -> f64 {
        let pl = &self.world.players[p];
        if pl.team < 0 { return 0.0; }
        let rest = self.monate_rest(p);
        pl.value * [1.25, 1.05, 0.85, 0.0][self.kaderrolle(p) as usize] * if rest <= 6.0 { 0.7 } else { 1.0 }
    }
    /// The club's price target (verhandlung.py:108): demand · (1 + (0.5 − readiness)·0.3), × 1.06 when upset with us,
    /// × 1.1 for a regular of a thin squad.
    pub fn ask_price(&self, p: usize) -> f64 {
        let pl = &self.world.players[p];
        if pl.team < 0 || pl.team == self.own as i32 { return 0.0; }
        let c = pl.team as usize;
        let mut v = self.abloese_forderung(p) * (1.0 + (0.5 - self.verkaufsbereitschaft(p)) as f64 * 0.3);
        if self.mensch.vereine.iter().any(|x| x.0 == c && x.1 < 0.0) { v *= 1.06; }
        if self.world.clubs[c].squad.len() <= 20 && self.kaderrolle(p) == 0 { v *= 1.1; }
        round_money(v.max(50000.0))
    }

    // ------------------------------------------------------------ offers for own players (§9 Verkauf)
    /// Weekly in an open window: at most 2 offers, only from rating 55, 85–125 % of the market value (+10 % with a wish to leave), 7 days to answer.
    pub fn angebote_woche(&mut self) {
        self.ausstieg_woche();
        self.ki_ausstieg_woche();
        let abs = self.abs();
        for m in self.inbox.iter_mut() { if !m.done && matches!(m.action, Action::TransferOffer { .. }) && abs >= abs_day(m.season, m.day) + 7 { m.done = true; m.body += crate::inhalt::t("ui.transfer.die_frist_ist_abgelaufen"); } }
        let woche = (abs / 7).to_string();
        let mut n = 0;
        for p in self.squad() {
            if n >= 2 { break; }
            let pl = &self.world.players[p];
            if pl.ovr < 55.0 || pl.youth > 0 || pl.leihe.is_some() { continue; }
            let chance = if pl.listed { 0.5 } else if pl.ps.wunsch { 0.4 } else if pl.ovr >= self.club().level { 0.08 } else { 0.03 };
            let id = pl.id.to_string();
            if zufall(&["2026", crate::id("angebot"), &id, &woche]) >= chance { continue; }
            let (ovr, value, wunsch) = (pl.ovr, pl.value, pl.ps.wunsch);
            let cands: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.erste_von(c).is_none() && (self.world.clubs[c].level - ovr).abs() <= 4.0 && self.world.clubs[c].balance > value).collect();
            if cands.is_empty() { continue; }
            let c = cands[(zufall(&["2026", "angebot_verein", &id, &woche]) * cands.len() as f32) as usize % cands.len()];
            let f = 0.85 + 0.4 * zufall(&["2026", "angebot_hoehe", &id, &woche]) as f64 + if wunsch { 0.1 } else { 0.0 };
            let fee = round_money(value * f);
            // on the list with a minimum price: lower offers are turned down without asking
            if self.world.players[p].listed && self.world.players[p].profil.mindestpreis > fee { continue; }
            let (nm, cn) = (self.world.players[p].name.clone(), self.world.clubs[c].name.clone());
            let ampel = if fee >= value * 1.1 { crate::inhalt::t("ui.transfer.der_preis_ist_gut") } else if fee >= value * 0.95 { crate::inhalt::t("ui.transfer.der_preis_ist_fair") } else { crate::inhalt::t("ui.transfer.das_ist_unter_wert") };
            let wort = if wunsch { crate::inhalt::t("ui.transfer.er_selbst_will_wechseln") } else { "" };
            let body = format!("{}\n\n{}", crate::tf!("ui.transfer.bietet_fuer_sein_marktwert_liegt", cn, money(fee), nm, money(value), ampel, wort), self.haltung_text(p, c));
            self.msg(2, &cn, &crate::tf!("fenster.angebot.titel", nm), &body, Action::TransferOffer { pid: p, club: c, fee });
            self.world.players[p].ps.angebote.push(abs);
            // medien.toml [fremd]: from 2 Mio. the papers pick it up as a rumour
            // the papers only pick it up when it is a story: a key player, a big club or big money, and not again within weeks
            let r = |k: &str| crate::regeln::f("medien", &format!("geruecht_angebot.{k}"));
            let pl = &self.world.players[p];
            let wichtig = if pl.ovr >= self.club().level { r(crate::id("stammspieler")) } else { 0.0 } + if self.world.clubs[c].level >= crate::regeln::f("medien", "relevanz.grosser_verein_ab") { r("grosser_verein") } else { 0.0 } + if fee >= self.world.revenue(self.own) * 0.1 { r("grosse_summe") } else { 0.0 };
            let neulich = self.me.geruechte.iter().any(|g| g.0 == p && abs < g.1 + r("sperre_tage") as u32);
            if fee >= 2_000_000.0 && !neulich && zufall(&["geruecht_angebot", &id, &woche]) < r("basis") + wichtig { self.geruecht_mit(p, true, Some(c)); } // roh
            n += 1;
        }
    }

    // ------------------------------------------------------------ drama during talks (drama.py)
    pub fn deadline(&self) -> bool { self.in_window().map_or(false, |(_, b)| self.world.day + 1 >= b) }
    pub fn drama_tag(&mut self) {
        let abs = self.abs();
        let fak = if self.deadline() { DEADLINE_FAKTOR } else { 1.0 };
        for i in 0..self.negs.len() {
            let n = self.negs[i].clone();
            if n.stage >= 2 || n.renew || n.haken != 0 { continue; }
            let id = self.world.players[n.pid].id.to_string();
            let u = zufall(&["2026", crate::id("drama"), &id, &abs.to_string()]);
            if u >= DRAMA_TAG * fak { continue; }
            let art = (zufall(&["2026", "drama_art", &id, &abs.to_string()]) * 3.0) as u8;
            let name = self.world.players[n.pid].name.clone();
            match art {
                0 => {
                    let nn = &mut self.negs[i];
                    nn.state = NegState::Waiting;
                    nn.reply_day = abs + 2;
                    nn.text = crate::tf!("ui.transfer.der_berater_von_laesst_sich", name);
                }
                1 => {
                    let rival = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.clubs[c].rep >= self.club().rep - 5.0 && self.world.players[n.pid].team != c as i32)
                        .min_by(|&a, &b| (self.world.clubs[a].rep - self.club().rep).abs().partial_cmp(&(self.world.clubs[b].rep - self.club().rep).abs()).unwrap());
                    let Some(r) = rival else { continue };
                    let rn = self.world.clubs[r].name.clone();
                    self.negs[i].haken = 3;
                    self.negs[i].haken_bis = abs + HIJACK_TAGE;
                    let opts = vec![(crate::id("ueberbieten").to_string(), crate::inhalt::t("ui.transfer.wir_legen_10_drauf").to_string()), (crate::id("verzichten").to_string(), crate::inhalt::t("ui.transfer.dann_soll_er_gehen").to_string())];
                    self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.transfer.graetscht_dazwischen", rn), &crate::tf!("ui.transfer.will_auch_und_bietet_mehr", rn, name, HIJACK_TAGE), Action::Wahl { art: crate::id("hijack").into(), bezug: vec![n.pid], optionen: opts });
                }
                _ if n.stage == 0 && n.round >= 2 => {
                    let nn = &mut self.negs[i];
                    nn.stage = 2;
                    nn.state = NegState::Rejected;
                    nn.text = crate::tf!("ui.transfer.der_verein_von_zieht_sich", name);
                    let t = nn.text.clone();
                    self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.transfer.aus_fuer", name), &t, Action::None);
                }
                _ => {}
            }
        }
        // whoever does not answer decides against the deal (drama.py STANDARD)
        for i in 0..self.negs.len() {
            if self.negs[i].haken != 0 && self.negs[i].stage < 2 && abs >= self.negs[i].haken_bis {
                let p = self.negs[i].pid;
                let art = ["", crate::id("medizin"), crate::id("nachforderung"), crate::id("hijack")][self.negs[i].haken as usize];
                let key = ["", crate::id("abbrechen"), crate::id("ablehnen"), crate::id("verzichten")][self.negs[i].haken as usize];
                for m in self.inbox.iter_mut() { if !m.done && matches!(&m.action, Action::Wahl { art: a, bezug, .. } if a == art && bezug.first() == Some(&p)) { m.done = true; m.body += crate::inhalt::t("ui.transfer.keine_antwort_die_frist_ist"); } }
                self.drama_antwort(art, p, key);
            }
        }
    }
    /// Gate before the signature: medical (own buys), then a last-minute demand (9 %, 3× on deadline day).
    /// Returns true when the contract may be signed now.
    pub fn vor_unterschrift(&mut self, i: usize) -> bool {
        let n = self.negs[i].clone();
        if n.renew { return true; }
        let p = n.pid;
        let id = self.world.players[p].id.to_string();
        let name = self.world.players[p].name.clone();
        let abs = self.abs();
        if !n.med {
            self.negs[i].med = true;
            let pl = &self.world.players[p];
            let (urteil, risiko, abschlag) = medizin(pl.inj, pl.age, zufall(&["2026", crate::id("medizin"), &id, &abs.to_string()]));
            if urteil != crate::id("bestanden") {
                self.negs[i].haken = 1;
                self.negs[i].haken_bis = abs + 3;
                self.negs[i].abschlag = abschlag as f64;
                let text = if urteil == crate::id("durchgefallen") { crate::tf!("ui.transfer.ist_beim_medizincheck_durchgefallen_risi", name, format!("{:.0}", risiko * 100.0)) } else { crate::tf!("ui.transfer.beim_medizincheck_von_gibt_es", name, format!("{:.0}", risiko * 100.0)) };
                let opts = vec![(crate::id("nachverhandeln").to_string(), crate::tf!("ui.transfer.wir_wollen_nachlass", format!("{:.0}", abschlag * 100.0))), (crate::id("trotzdem").to_string(), crate::inhalt::t("ui.transfer.wir_holen_ihn_trotzdem").to_string()), (crate::id("abbrechen").to_string(), crate::inhalt::t("ui.transfer.wir_lassen_es").to_string())];
                self.msg(2, crate::inhalt::t("med.vg.absender"), &crate::tf!("ui.transfer.medizincheck", name), &crate::tf!("ui.transfer.du_hast_3_tage", text), Action::Wahl { art: crate::id("medizin").into(), bezug: vec![p], optionen: opts });
                self.negs[i].state = NegState::Waiting;
                self.negs[i].reply_day = u32::MAX;
                return false;
            }
        }
        if !n.gate {
            self.negs[i].gate = true;
            let fak = if self.deadline() { GATE_DEADLINE } else { 1.0 };
            if zufall(&["2026", crate::id("gate"), &id, &abs.to_string()]) < GATE * fak {
                self.negs[i].haken = 2;
                self.negs[i].haken_bis = abs + 3;
                let opts = vec![(crate::id("zahlen").to_string(), crate::inhalt::t("ui.transfer.wir_zahlen_10_mehr_gehalt").to_string()), (crate::id("verhandeln").to_string(), crate::inhalt::t("ui.transfer.wir_verhandeln_noch_einmal").to_string()), (crate::id("ablehnen").to_string(), crate::inhalt::t("ui.transfer.nein_es_bleibt_beim_vereinbarten").to_string())];
                self.msg(2, crate::inhalt::t("meldung.absender.berater"), &crate::tf!("ui.transfer.nachforderung_im_fall", name), &crate::tf!("ui.transfer.kurz_vor_der_unterschrift_will", name), Action::Wahl { art: crate::id("nachforderung").into(), bezug: vec![p], optionen: opts });
                self.negs[i].state = NegState::Waiting;
                self.negs[i].reply_day = u32::MAX;
                return false;
            }
        }
        true
    }
    /// The answer to a drama choice; the deal goes on or ends.
    pub fn drama_antwort(&mut self, art: &str, p: usize, key: &str) -> String {
        let Some(i) = self.negs.iter().position(|n| n.pid == p && n.stage < 2) else { return crate::inhalt::t("ui.transfer.die_verhandlung_ist_schon_vorbei").into() };
        let name = self.world.players[p].name.clone();
        let abs = self.abs();
        self.negs[i].haken = 0;
        self.negs[i].state = NegState::Counter;
        let weiter = match (art, key) {
            ("medizin", "nachverhandeln") => {
                if zufall(&["2026", crate::id("nachlass"), &p.to_string(), &abs.to_string()]) < MED_NACHLASS_CHANCE { let a = self.negs[i].abschlag; self.negs[i].fee = round_money(self.negs[i].fee * (1.0 - a)); true } else { false }
            }
            ("medizin", "trotzdem") | ("nachforderung", "zahlen") => { if art == crate::id("nachforderung") { self.negs[i].wage = round_money(self.negs[i].wage * 1.1); } true }
            ("nachforderung", "verhandeln") => zufall(&["2026", crate::id("nachverhandeln"), &p.to_string(), &abs.to_string()]) < 0.5,
            ("hijack", "ueberbieten") => { self.negs[i].fee = round_money(self.negs[i].fee * 1.1); self.transfer_budget >= self.negs[i].fee }
            _ => false,
        };
        if !weiter {
            let n = &mut self.negs[i];
            n.stage = 2;
            n.state = NegState::Rejected;
            n.text = crate::tf!("ui.transfer.der_wechsel_von_ist_geplatzt", name);
            self.news(&crate::tf!("ui.transfer.wechsel_von_geplatzt", name), crate::inhalt::t("ui.transfer.auf_den_letzten_metern_ist"));
            return crate::inhalt::t("ui.transfer.der_wechsel_ist_geplatzt").into();
        }
        if art == crate::id("hijack") { self.negs[i].text = crate::tf!("ui.transfer.wir_haben_nachgelegt_abloese", money(self.negs[i].fee)); return crate::inhalt::t("ui.transfer.wir_bleiben_im_rennen").into(); }
        let v = self.negs[i].wage;
        self.neg_einig(i, v);
        crate::inhalt::t("ui.transfer.es_geht_weiter").into()
    }
    /// Aftermath of a signing (§9 step 7): finds his place +2 or needs weeks −1.5; up to 2 rivals on his position
    /// (at most 8 weaker) −2 each; at most 5 morale points in sum.
    pub fn nachspiel(&mut self, p: usize) {
        self.psyche_an();
        let id = self.world.players[p].id.to_string();
        let (pos, ovr, name) = (self.world.players[p].pos.clone(), self.world.players[p].ovr, self.world.players[p].name.clone());
        if zufall(&["2026", crate::id("anschluss"), &id]) < 0.5 { self.moral_wirkung(p, 2.0, 0, crate::inhalt::t("ui.transfer.findet_schnell_anschluss_im_neuen")); } else { self.moral_wirkung(p, -1.5, 0, crate::inhalt::t("ui.transfer.braucht_wochen_um_anzukommen")); }
        let mut rivals: Vec<usize> = self.squad().into_iter().filter(|&q| q != p && self.world.players[q].pos == pos && self.world.players[q].ovr >= ovr - 8.0).collect();
        rivals.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
        let mut summe = 0.0;
        for q in rivals.into_iter().take(2) {
            if summe >= 5.0 { break; }
            self.moral_wirkung(q, -2.0, 0, &crate::tf!("ui.transfer.neue_konkurrenz_auf_seiner_position", name));
            summe += 2.0;
        }
    }

    // ------------------------------------------------------------ strike of a player who wants to leave (drama.py streik)
    /// A refused offer for a player with a wish to leave (readiness ≥ 60) raises his protest: unrest → training strike → match strike.
    pub fn streik_pruefen(&mut self, p: usize) {
        if self.world.players[p].ps.wb < STREIK_AB { return; }
        let stufe = (self.world.players[p].ps.streik + 1).min(3);
        self.world.players[p].ps.streik = stufe;
        let einfluss = EINFLUSS[self.world.players[p].ps.rolle.min(6) as usize];
        let d = -einfluss / 20.0 * stufe as f32;
        self.mensch.teamgeist = (self.mensch.teamgeist + d).clamp(0.0, 100.0);
        let name = self.world.players[p].name.clone();
        self.grund(crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("charakter.teamgeist"), d, &crate::tf!("ui.transfer.protestiert_gegen_den_geplatzten_wechsel", name));
        let (t, b) = match stufe { 1 => (crate::inhalt::t("dr.ui.streik.stufe.1"), crate::tf!("ui.transfer.ist_sauer_ueber_das_abgelehnte", name)), 2 => ("Trainingsstreik", crate::tf!("ui.transfer.erscheint_nicht_mehr_zum_training", name)), _ => ("Einsatzstreik", crate::tf!("ui.transfer.weigert_sich_zu_spielen_bis", name)) }; // roh
        self.msg(1, &self.co_name.clone(), &crate::tf!("diagnose.schalter", t, name), &b, Action::None);
        if stufe >= 2 { let n2 = name.clone(); self.news(&crate::tf!("ui.transfer.streikt", n2), crate::inhalt::t("ui.transfer.der_spieler_will_den_verein")); }
    }
    pub fn streik_tag(&mut self) {
        for p in self.squad() {
            let pl = &mut self.world.players[p];
            if pl.ps.streik == 0 { continue; }
            if !pl.ps.wunsch { pl.ps.streik = 0; continue; }
            if pl.ps.streik >= 3 { pl.susp = pl.susp.max(1); }
            if pl.ps.streik >= 2 { pl.fit = (pl.fit - 0.5).max(60.0); }
        }
    }

    // ------------------------------------------------------------ loans (§9: fee 15 %, wage share 70 %, purchase option)
    pub fn leihe_holen(&mut self, p: usize) -> Result<(), String> {
        if self.in_window().is_none() { return Err(crate::inhalt::t("ui.farm.das_transferfenster_ist_geschlossen").into()); }
        let pl = &self.world.players[p];
        if pl.team < 0 || pl.team == self.own as i32 { return Err(crate::inhalt::t("ui.transfer.nur_spieler_anderer_vereine_lassen").into()); }
        if pl.leihe.is_some() { return Err(crate::inhalt::t("ui.farm.er_ist_schon_verliehen").into()); }
        if self.kaderrolle(p) == 0 { return Err(crate::tf!("ui.transfer.ist_bei_seinem_verein_stammspieler", pl.name)); }
        let fee = round_money(pl.value * LEIH_GEBUEHR);
        if fee > self.transfer_budget { return Err(crate::inhalt::t("ui.transfer.die_leihgebuehr_sprengt_das_transferbudg").into()); }
        let von = pl.team as usize;
        let option = round_money(pl.value);
        let (name, vn) = (pl.name.clone(), self.world.clubs[von].name.clone());
        self.world.clubs[von].squad.retain(|&x| x != p);
        self.world.clubs[von].balance += fee;
        self.book(crate::inhalt::t("ui.transfer.leihgebuehren"), -fee);
        self.transfer_budget -= fee;
        let season = self.world.season;
        let abs = self.abs();
        let pl = &mut self.world.players[p];
        pl.team = self.own as i32;
        pl.leihe = Some(Leihe { von, bis: season, anteil: LEIH_ANTEIL, option, spielzeit: false, grund: String::new(), start: 0.0 });
        pl.know = 1.0;
        pl.joined = abs;
        self.world.clubs[self.own].squad.push(p);
        self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("vhd.titel.loan", name), &crate::tf!("ui.transfer.kommt_bis_saisonende_von_leihgebuehr", name, vn, money(fee), money(option)), Action::None);
        self.nachspiel(p);
        self.world.refresh_levels();
        Ok(())
    }
    /// Puts an own player on the loan list (Fabian 23.11.: like the transfer list): clubs make offers, the person chooses.
    pub fn leihe_abgeben(&mut self, p: usize) -> Result<(), String> {
        let pl = &self.world.players[p];
        if pl.team != self.own as i32 || pl.leihe.is_some() { return Err(crate::inhalt::t("ui.farm.das_geht_nur_mit_eigenen").into()); }
        self.world.players[p].ps.leihliste = true;
        Ok(())
    }
    /// Loan an own player to `c` on these terms (fee to us, wage share the borrower pays, a promise of play time).
    pub fn leihe_an(&mut self, p: usize, c: usize, fee: f64, anteil: f32, spielzeit: bool) -> Result<(), String> {
        if self.in_window().is_none() { return Err(crate::inhalt::t("ui.farm.das_transferfenster_ist_geschlossen").into()); }
        let pl = &self.world.players[p];
        if pl.team != self.own as i32 || pl.leihe.is_some() { return Err(crate::inhalt::t("ui.farm.das_geht_nur_mit_eigenen").into()); }
        let season = self.world.season;
        let (name, cn, start) = (pl.name.clone(), self.world.clubs[c].name.clone(), pl.ovr);
        self.world.clubs[self.own].squad.retain(|&x| x != p);
        self.world.clubs[c].squad.push(p);
        self.world.clubs[c].balance -= fee;
        if fee > 0.0 { self.book(crate::inhalt::t("ui.transfer.leihgebuehren"), fee); }
        self.xi.retain(|&x| x != p);
        let own = self.own;
        let pl = &mut self.world.players[p];
        pl.team = c as i32;
        pl.ps.leihliste = false;
        pl.leihe = Some(Leihe { von: own, bis: season, anteil, option: 0.0, spielzeit, grund: String::new(), start });
        self.news(&crate::tf!("ui.transfer.wird_verliehen", name), &crate::tf!("ui.transfer.spielt_bis_saisonende_fuer", name, cn));
        self.world.refresh_levels();
        Ok(())
    }
    /// Loan offers for the players on the own loan list (in a window, every few days): the more a player promises,
    /// the more clubs ask; each offer names the club, the role he would have, the fee and the wage share.
    pub fn leihangebote_tag(&mut self) {
        if self.in_window().is_none() { return; }
        let abs = self.abs();
        let r = |k: &str| crate::regeln::f("transfers", &format!("leihe.{k}"));
        for m in self.inbox.iter_mut() { if !m.done && m.action_art() == Some(crate::id("leihe")) && abs >= abs_day(m.season, m.day) + r("frist_tage") as u32 { m.done = true; m.body += crate::inhalt::t("ui.transfer.die_frist_ist_abgelaufen"); } }
        let liste: Vec<usize> = self.squad().into_iter().filter(|&p| self.world.players[p].ps.leihliste).collect();
        let land = crate::cups::country(&self.world.leagues[self.league()]);
        for p in liste {
            let pl = &self.world.players[p];
            let (ovr, pot, age, value, id) = (pl.ovr, pl.pot, pl.age, pl.value, pl.id);
            if self.inbox.iter().filter(|m| !m.done && m.action_art() == Some(crate::id("leihe")) && matches!(&m.action, Action::Wahl { bezug, .. } if bezug.first() == Some(&p))).count() >= r("offen_hoechstens") as usize { continue; }
            // interest: young with room to grow and already good enough for a lower club draw more clubs
            let reiz = ((ovr - 55.0) / 20.0).clamp(0.0, 1.0) * 0.6 + ((pot - ovr) / 15.0).clamp(0.0, 1.0) * 0.3 + if age <= 23 { 0.1 } else { 0.0 };
            if zufall(&["leihangebot", &id.to_string(), &abs.to_string()]) > r("chance_basis") + r("chance_reiz") * reiz { continue; } // roh
            // clubs of the own country first; when none fits his level, clubs abroad
            let passt = |c: usize, inland: bool| c != self.own && self.world.erste_von(c).is_none() && self.world.clubs[c].level <= ovr + 3.0 && self.world.clubs[c].level >= ovr - 12.0
                && (crate::cups::country(&self.world.leagues[self.world.clubs[c].league]) == land) == inland
                && !self.inbox.iter().any(|m| !m.done && m.action_art() == Some(crate::id("leihe")) && matches!(&m.action, Action::Wahl { art, bezug, .. } if bezug.first() == Some(&p) && art.starts_with(&format!("leihe:{c}:"))));
            let mut cands: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| passt(c, true)).collect();
            if cands.is_empty() { cands = (0..self.world.clubs.len()).filter(|&c| passt(c, false)).collect(); }
            if cands.is_empty() { continue; }
            let c = cands[(zufall(&["leihverein", &id.to_string(), &abs.to_string()]) * cands.len() as f32) as usize % cands.len()]; // roh
            let stamm = ovr >= self.world.clubs[c].level - 1.0;
            let anteil = (r("anteil_von") + (r("anteil_bis") - r("anteil_von")) * reiz).min(1.0);
            let fee = round_money(value * (r("gebuehr_von") + (r("gebuehr_bis") - r("gebuehr_von")) * reiz) as f64);
            let (nm, cn, ln) = (self.world.players[p].name.clone(), self.world.clubs[c].name.clone(), self.world.leagues[self.world.clubs[c].league].name.clone());
            let rolle = crate::inhalt::t(if stamm { "ui.leihe.rolle_stamm" } else { "ui.leihe.rolle_rotation" });
            let body = crate::tf!("ui.leihe.angebot_text", cn.clone(), ln, nm.clone(), rolle, money(fee), format!("{:.0}", anteil * 100.0));
            let opts = vec![(crate::id("ja").to_string(), crate::inhalt::t("ui.leihe.annehmen").to_string()), (crate::id("nein").to_string(), crate::inhalt::t("ui.leihe.ablehnen").to_string())];
            self.msg(2, &cn, &crate::tf!("ui.leihe.angebot_titel", nm), &body, Action::Wahl { art: format!("leihe:{c}:{}:{}:{}", fee as u64, (anteil * 100.0) as u32, stamm as u8), bezug: vec![p], optionen: opts });
        }
    }
    pub fn leihe_antwort(&mut self, sub: &str, p: usize, key: &str) -> String {
        if key != "ja" { return crate::inhalt::t("ui.leihe.abgelehnt").into(); }
        let t: Vec<&str> = sub.split(':').collect();
        let (c, fee, anteil, stamm) = (t.first().and_then(|x| x.parse().ok()).unwrap_or(usize::MAX), t.get(1).and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0), t.get(2).and_then(|x| x.parse::<f32>().ok()).unwrap_or(50.0) / 100.0, t.get(3) == Some(&"1"));
        if c >= self.world.clubs.len() { return String::new(); }
        match self.leihe_an(p, c, fee, anteil, stamm) { Ok(()) => crate::tf!("ui.leihe.ok", self.world.players[p].name.clone(), self.world.clubs[c].name.clone()), Err(e) => e }
    }
    /// The monthly report of the players we have out on loan: games, goals, grade and how they grew.
    pub fn leih_bericht(&mut self) {
        let weg: Vec<usize> = (0..self.world.players.len()).filter(|&p| self.world.players[p].leihe.as_ref().map_or(false, |l| l.von == self.own) && self.world.players[p].team != self.own as i32 && self.world.players[p].team >= 0).collect();
        if weg.is_empty() { return; }
        let zeilen: Vec<String> = weg.iter().map(|&p| {
            let pl = &self.world.players[p];
            let l = pl.leihe.as_ref().unwrap();
            let entw = if l.start > 0.0 { pl.ovr - l.start } else { 0.0 };
            let note = if pl.rn > 0 { format!("{:.1}", pl.avg_rating()) } else { "–".into() };
            let urteil = crate::inhalt::t(if pl.apps < 3 { "ui.leihe.kaum_spielzeit" } else if entw >= 1.0 { "ui.leihe.waechst" } else { "ui.leihe.spielt" });
            crate::tf!("ui.leihe.bericht_zeile", pl.name.clone(), self.world.clubs[pl.team as usize].name.clone(), pl.apps, pl.goals, note, format!("{entw:+.0}"), urteil)
        }).collect();
        let karte: Vec<(usize, f32)> = weg.iter().map(|&p| (p, self.world.players[p].ovr)).collect();
        self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), crate::inhalt::t("ui.leihe.bericht_titel"), &zeilen.join(crate::id("\n")), Action::None);
        self.meldung_karte(crate::pages::Art::Talent, crate::id("entwicklung"), karte.into_iter().take(6).collect());
    }
    /// Draw the purchase option of a loaned player.
    pub fn kaufoption(&mut self, p: usize) -> Result<(), String> {
        let Some(l) = self.world.players[p].leihe.clone() else { return Err(crate::inhalt::t("ui.transfer.er_ist_nicht_ausgeliehen").into()) };
        if l.von == self.own || l.option <= 0.0 { return Err(crate::inhalt::t("ui.transfer.es_gibt_keine_kaufoption").into()); }
        if l.option > self.transfer_budget { return Err(crate::inhalt::t("ui.transfer.die_kaufoption_sprengt_das_transferbudge").into()); }
        self.world.clubs[l.von].balance += l.option;
        self.book(crate::inhalt::t("ui.game.abloesen_kauf"), -l.option);
        self.transfer_budget -= l.option;
        let season = self.world.season;
        let pl = &mut self.world.players[p];
        pl.leihe = None;
        pl.contract_end = season + contract_years(pl.age) - 1;
        let n = pl.name.clone();
        self.news(&crate::tf!("ui.transfer.bleibt_fest", n), &crate::tf!("ui.transfer.die_kaufoption_ist_gezogen", money(l.option)));
        Ok(())
    }
    /// Season end: loaned players go back.
    pub fn leihen_ende(&mut self) {
        let season = self.world.season;
        for p in 0..self.world.players.len() {
            let Some(l) = self.world.players[p].leihe.clone() else { continue };
            if l.bis > season { continue; }
            let t = self.world.players[p].team;
            if t >= 0 { self.world.clubs[t as usize].squad.retain(|&x| x != p); }
            self.world.clubs[l.von].squad.push(p);
            self.xi.retain(|&x| x != p);
            let pl = &mut self.world.players[p];
            pl.team = l.von as i32;
            pl.leihe = None;
        }
    }
    /// The share of the wage bill the own club does not pay because of loans (in and out).
    pub fn leih_entlastung(&self) -> f64 {
        let mut v = 0.0;
        for pl in &self.world.players {
            let Some(l) = &pl.leihe else { continue };
            if pl.team == self.own as i32 && l.von != self.own { v += pl.wage * (1.0 - l.anteil as f64); }
            if l.von == self.own && pl.team != self.own as i32 { v -= pl.wage * (1.0 - l.anteil as f64); }
        }
        v
    }
    pub fn treue_wort(&self, p: usize) -> &'static str { if cw(&self.world.players[p], LOYAL) >= 60.0 { crate::id("treu") } else { crate::inhalt::t("ui.transfer.offen_fuer_neues") } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn medizin_wie_vorlage() {
        let (u, p, a) = medizin(0, 25, 0.5);
        assert_eq!(u, "bestanden");
        assert!((p - 0.07).abs() < 1e-6);
        assert!((a - 0.075).abs() < 1e-6);
        assert_eq!(medizin(0, 25, 0.01).0, "durchgefallen");
        assert_eq!(medizin(0, 25, 0.05).0, "auffaellig");
        assert!(medizin(60, 35, 0.5).1 > 0.15, "injured and old players carry more risk");
    }
    #[test]
    fn mindestpreis_haelt_billige_angebote_ab() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        let sq = c.squad();
        for &p in &sq { c.world.players[p].listed = true; c.world.players[p].profil.mindestpreis = c.world.players[p].value * 10.0; }
        for _ in 0..30 { c.angebote_woche(); for _ in 0..7 { c.world.day += 1; } }
        assert!(!c.inbox.iter().any(|m| matches!(m.action, Action::TransferOffer { .. })), "no offer below the minimum price reaches the person");
        for &p in &sq { c.world.players[p].profil.mindestpreis = 0.0; }
        for _ in 0..30 { c.angebote_woche(); for _ in 0..7 { c.world.day += 1; } }
        assert!(c.inbox.iter().any(|m| matches!(m.action, Action::TransferOffer { .. })), "without a minimum price offers come");
    }
    #[test]
    fn doppeltes_senden_wirkt_einmal() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        let other = (1..c.world.clubs.len()).find(|&k| c.world.clubs[k].squad.len() > 22).unwrap();
        let p = *c.world.clubs[other].squad.iter().min_by(|a, b| c.world.players[**a].ovr.partial_cmp(&c.world.players[**b].ovr).unwrap()).unwrap();
        let i = c.open_neg(p).unwrap();
        c.negs[i].fee = round_money(c.negs[i].ask * 0.93);
        c.submit_neg(i);
        let (runde, geduld, text) = (c.negs[i].ph.as_ref().unwrap().runde, c.negs[i].ph.as_ref().unwrap().geduld, c.negs[i].text.clone());
        if c.negs[i].state == NegState::Counter && c.negs[i].stage == 0 {
            c.submit_neg(i);
            assert_eq!((c.negs[i].ph.as_ref().unwrap().runde, c.negs[i].ph.as_ref().unwrap().geduld, c.negs[i].text.clone()), (runde, geduld, text));
        }
        // a renewal opens only 18 months before the end of the contract
        let own = c.squad()[0];
        c.world.players[own].contract_end = c.world.season + 4;
        assert!(c.open_neg(own).is_err());
    }
    #[test]
    fn leihe_hin_und_zurueck() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        c.transfer_budget = 1e9;
        let other = (1..c.world.clubs.len()).find(|&k| c.world.clubs[k].squad.len() > 22).unwrap();
        let p = *c.world.clubs[other].squad.iter().min_by(|a, b| c.world.players[**a].ovr.partial_cmp(&c.world.players[**b].ovr).unwrap()).unwrap();
        c.leihe_holen(p).unwrap();
        assert_eq!(c.world.players[p].team, 0);
        assert!(c.leih_entlastung() > 0.0);
        c.leihen_ende();
        assert_eq!(c.world.players[p].team, other as i32, "back at the end of the season");
        assert!(c.world.players[p].leihe.is_none());
    }
}
