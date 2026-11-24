//! Rest of Paket 3: the manager as a person (manager.toml, §12), half-time talk and mind games (§6, kabine.toml),
//! the youth network (jugend.toml, §19) and applying for vacant coaching jobs (laufbahn.toml [bewerbung], §12).
use crate::game::*;
use crate::mensch::{arr, zufall};
use crate::regeln::{f, wert};
use crate::world::*;
use serde::{Deserialize, Serialize};

pub const WERTE: [&str; 7] = [crate::id("menschenfuehrung"), crate::id("taktikwissen"), crate::id("autoritaet"), crate::id("medienwirkung"), crate::id("nachwuchsarbeit"), crate::id("fitnesswissen"), crate::id("verhandlungsgeschick")];
pub fn werte_de_t() -> [&'static str; 7] { [crate::inhalt::t("manager.wert.menschenfuehrung"), crate::inhalt::t("manager.wert.taktikwissen"), crate::inhalt::t("manager.wert.autoritaet"), crate::inhalt::t("manager.wert.medienwirkung"), crate::inhalt::t("manager.wert.nachwuchsarbeit"), crate::inhalt::t("manager.wert.fitnesswissen"), crate::inhalt::t("manager.wert.verhandlungsgeschick")] }
pub const STILE: [&str; 4] = [crate::id("hart"), crate::id("fuersorglich"), crate::id("diplomatisch"), crate::id("kumpelhaft")];
pub fn stile_de_t() -> [&'static str; 4] { [crate::inhalt::t("tp.last.hoch"), crate::inhalt::t("ui.rest3.fuersorglich"), crate::inhalt::t("ui.rest3.diplomatisch"), crate::inhalt::t("ui.rest3.kumpelhaft")] }
pub const PERSONA: [&str; 4] = [crate::id("traditionell"), crate::id("ehrgeizig"), crate::id("sparsam"), crate::id("geduldig")];
pub const SCHRITTE: [&str; 4] = [crate::id("projekt"), crate::id("spielstil"), crate::id("rueckfrage"), crate::id("reaktion")];

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Manager {
    pub an: bool,
    pub werte: [f32; 7],
    pub energie: f32,
    pub stress: f32,
    pub motivation: f32,
    pub stil: [f32; 4],
    /// Free-time slots used this week.
    pub frei: u8,
    /// Youth network: (country, strength, investment level 0…3), contacts (country, strength), running recruitments (talent key, until).
    pub netz: Vec<(String, f32, u8)>,
    pub kontakte: Vec<(String, f32)>,
    pub anwerbung: Vec<(String, u32)>,
    /// Application: (club, step 0…3, summed answer effects).
    pub bewerbung: Option<(usize, u8, f32)>,
    /// Running talks about the coach's own contract with a new club (jobverhandlung.rs).
    pub verh: Option<crate::jobverhandlung::JobVerh>,
}

/// A fictive U17 talent of a region (jugend.toml): visible from network 20, fixed per country and month.
#[derive(Clone)]
pub struct Talent { pub key: String, pub name: String, pub land: String, pub alter: u8, pub ovr: f32, pub pot: f32, pub pos: String, pub wert: f64 }

impl Career {
    fn m(k: &str) -> f32 { f("manager", k) }
    pub fn manager_an(&mut self) {
        if self.mgr.an { return; }
        for (i, w) in WERTE.iter().enumerate() { self.mgr.werte[i] = Self::m(&format!("start.werte.{w}")); }
        self.mgr.energie = Self::m("start.zustand.energie");
        self.mgr.stress = Self::m("start.zustand.stress");
        self.mgr.motivation = Self::m("start.zustand.motivation");
        self.mgr.an = true;
    }
    /// A skill of the manager after exhaustion (stress ≥ 70 up to −8, energy < 30 up to −6; manager.toml [abzug]).
    pub fn mgr_wert(&self, i: usize) -> f32 {
        if !self.mgr.an { return 50.0; }
        let a = |k: &str| Self::m(&format!("abzug.{k}"));
        let s = ((self.mgr.stress - a("stress_ab")) / (100.0 - a("stress_ab"))).clamp(0.0, 1.0) * a("stress_max");
        let e = ((a("energie_unter") - self.mgr.energie) / a("energie_unter")).clamp(0.0, 1.0) * a("energie_max");
        (self.mgr.werte[i] - s - e).clamp(0.0, 100.0)
    }
    /// Weekly: the job costs stress and energy, motivation drifts to the middle, the style fades (0.97).
    pub fn manager_woche(&mut self) {
        self.manager_an();
        let w = |k: &str| Self::m(&format!("woche.{k}"));
        self.mgr.stress = (self.mgr.stress + w(crate::id("stress"))).clamp(0.0, 100.0);
        self.mgr.energie = (self.mgr.energie + w(crate::id("energie"))).clamp(0.0, 100.0);
        self.mgr.motivation += (50.0 - self.mgr.motivation) * w("motivation_zu_mitte");
        let ab = Self::m("stil.abklingen");
        for s in self.mgr.stil.iter_mut() { *s *= ab; }
        self.mgr.frei = 0;
    }
    /// Free time (manager.toml woche.zeitplaetze 3 a week): own decision, each slot −10 stress, +10 energy.
    pub fn freizeit(&mut self) -> Result<(), String> {
        self.manager_an();
        if self.mgr.frei as f32 >= Self::m("woche.zeitplaetze") { return Err(crate::inhalt::t("ui.rest3.diese_woche_ist_keine_freizeit").into()); }
        self.mgr.frei += 1;
        self.mgr.stress = (self.mgr.stress - 10.0).max(0.0);
        self.mgr.energie = (self.mgr.energie + 10.0).min(100.0);
        Ok(())
    }
    /// Behaviour collects style points (manager.toml [verhalten.*]).
    pub fn stil_punkte(&mut self, gruppe: &str, art: &str) {
        self.manager_an();
        if let Some(m) = crate::regeln::wert_oder("manager", &format!("verhalten.{gruppe}.{art}")).and_then(|v| v.as_object().cloned()) {
            for (k, v) in m { if let Some(i) = STILE.iter().position(|s| *s == k) { self.mgr.stil[i] += v.as_f64().unwrap_or(0.0) as f32; } }
        }
    }
    /// The recognisable style (at least 3 points and 35 % of all), else None.
    pub fn stil(&self) -> Option<usize> {
        let sum: f32 = self.mgr.stil.iter().sum();
        let (i, v) = self.mgr.stil.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap())?;
        (sum >= Self::m("stil.mindest_summe") && *v / sum >= Self::m("stil.mindest_anteil")).then_some(i)
    }
    /// Effect factor of a skill: 50 neutral, ±10 % at 0/100 (own decision: how the person's skills act on the game).
    pub fn mgr_faktor(&self, i: usize) -> f32 { 1.0 + 0.2 * (self.mgr_wert(i) - 50.0) / 100.0 }
    /// After an own match: following the assistant's line-up raises tactical knowledge (+0.3).
    pub fn manager_nach_spiel(&mut self) {
        self.manager_an();
        if self.co_lineup { self.mgr.werte[1] = (self.mgr.werte[1] + Self::m("taktik.folgt_rat")).min(100.0); }
    }

    // ------------------------------------------------------------ half-time talk and mind games (§6)
    /// After an own match: what did the coach say at half-time? (kabine.toml halbzeit_passung by the score at the break)
    /// The tones of the half-time talk (key, words).
    pub fn halbzeit_optionen() -> Vec<(String, String)> {
         [(crate::id("warm"), crate::inhalt::t("ui.rest3.ruhig_und_aufbauend")), (crate::id("sachlich"), crate::inhalt::t("ui.rest3.sachlich_mit_klaren_anweisungen")), (crate::id("bestimmt"), crate::inhalt::t("ui.rest3.bestimmt_jetzt_mehr")), (crate::id("hart"), crate::inhalt::t("ui.rest3.laut_und_hart")), (crate::id("lob"), crate::inhalt::t("ui.rest3.lob_fuer_die_erste_haelfte"))].iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }
    /// The score at the break as the fit key of kabine.toml (rueckstand, fuehrung, remis).
    pub fn halbzeit_stand(gf: u8, ga: u8) -> &'static str { if gf < ga { crate::id("rueckstand") } else if gf > ga { crate::id("fuehrung") } else { crate::id("remis") } }
    /// The score at the break of a simulated match from its events (own goals first).
    pub fn halbzeit_tore(&self, res: &MatchResult) -> (u8, u8) {
        let home = res.h == self.own;
        let (mut gf, mut ga) = (0, 0);
        for e in res.events.iter().filter(|e| e.min <= 45 && (e.kind == 0 || e.kind == 1)) {
            let unser = (e.side == 0) == home;
            let tor_fuer_uns = if e.kind == 0 { unser } else { !unser };
            if tor_fuer_uns { gf += 1; } else { ga += 1; }
        }
        (gf, ga)
    }
    /// Tone effect as in the conversation engine plus the fit to the score; the team's morale gets half of the mean (ansprachen.py:30).
    pub fn halbzeit_antwort(&mut self, stand: &str, ton: &str) -> String {
        let pass = crate::regeln::f_oder("kabine", &format!("halbzeit_passung.{stand}.{ton}"), 0.0);
        let mut summe = 0.0;
        let sq: Vec<usize> = self.xi.clone();
        for &p in &sq {
            let pl = &self.world.players[p];
            let chs: Vec<f32> = (0..10).map(|i| crate::mensch::cw(pl, i)).collect();
            summe += crate::mensch::ton_aenderung(ton, &chs, 50.0, self.vertrauen(p)) * self.mgr_faktor(0) + pass;
        }
        let mittel = if sq.is_empty() { 0.0 } else { summe / sq.len() as f32 };
        self.hz_wirkung = mittel;
        for &p in &sq { self.moral_wirkung(p, mittel / 2.0, 0, &crate::tf!("ui.rest3.halbzeitansprache", ton)); }
        let tendenz = f("kabine", "ansprache.form_schwelle");
        if mittel >= tendenz { for &p in &sq { self.world.players[p].form += 0.5; } }
        if mittel <= -tendenz { for &p in &sq { self.world.players[p].form -= 0.5; } }
        self.stil_punkte(crate::id("ton"), match ton { "warm" => "einfuehlsam", "sachlich" => "sachlich", "hart" => "hart", "bestimmt" => "bestimmt", _ => crate::id("offen") });
        let urteil = if mittel >= 2.0 { crate::inhalt::t("ui.rest3.die_ansprache_kam_an") } else if mittel <= -2.0 { crate::inhalt::t("ui.rest3.die_ansprache_ging_daneben") } else { crate::inhalt::t("ui.rest3.die_ansprache_verpuffte") };
        // what it brought, in numbers (Fabian 23.11.): the team's morale, the form, and a push for the one it reached most
        let mut s = format!("{urteil} {}", crate::tf!("ans.wirkung", format!("{:+.1}", mittel / 2.0).replace('.', ",")));
        if mittel >= tendenz { s = format!("{s} {}", crate::inhalt::t("ans.form_plus")); } else if mittel <= -tendenz { s = format!("{s} {}", crate::inhalt::t("ans.form_minus")); }
        if mittel >= 2.0 {
            if let Some(&p) = sq.iter().max_by(|a, b| crate::mensch::cw(&self.world.players[**a], crate::mensch::EHRGEIZ).partial_cmp(&crate::mensch::cw(&self.world.players[**b], crate::mensch::EHRGEIZ)).unwrap()) {
                self.world.players[p].form += 1.0;
                s = format!("{s} {}", crate::tf!("ans.push", self.world.players[p].name.clone()));
            }
        }
        s
    }
    /// One player addressed after the match: praise or criticism, judged against his grade (spieltag.toml note_*).
    pub fn spieler_ansprechen(&mut self, p: usize, lob: bool, note: f32) -> String {
        let k = |x: &str| f("spieltag", &format!("teamgespraech.{x}"));
        let stark = note >= f("spieltag", "bericht.note_spieler_des_spiels");
        let schwach = note <= f("spieltag", "bericht.note_schwach_spieler");
        let d = match (lob, stark, schwach) {
            (true, true, _) => 3.0 + k("note_verdient"),
            (true, _, true) => k("note_hohl"),
            (true, _, _) => 1.5,
            (false, true, _) => k("note_unfair"),
            (false, _, true) => k("note_verdient") + 0.5,
            (false, _, _) => -1.5,
        };
        let n = self.world.players[p].name.clone();
        self.moral_wirkung(p, d, 0, crate::inhalt::t(if lob { "ans.lob" } else { "ans.kritik" }));
        self.beziehung(p, d / 2.0, crate::inhalt::t(if lob { "ans.lob" } else { "ans.kritik" }));
        crate::tf!("ans.einzeln", n, format!("{d:+.0}"))
    }
    /// Mind games the day before a match: statement × balance of strength (favourite, open, underdog) → morale and media.
    pub fn mind_games_frage(&mut self) {
        let Some(f_) = self.next_match() else { return };
        // not before every match (Fabian 23.11.: the same question again and again): at most every ten days
        let abs = self.abs();
        if self.mensch.co_sperre.iter().any(|x| x.0 == "mind" && abs < x.1) { return; } // roh
        self.mensch.co_sperre.retain(|x| x.0 != "mind"); // roh
        self.mensch.co_sperre.push(("mind".into(), abs + 10)); // roh
        let g = if f_.h == self.own { f_.a } else { f_.h };
        let d = crate::mensch::kaderstaerke(&self.world, self.own) - crate::mensch::kaderstaerke(&self.world, g);
        let lage = if d >= 2.0 { crate::id("favorit") } else if d <= -2.0 { crate::id("aussenseiter") } else { crate::id("offen") };
        let gn = self.world.clubs[g].name.clone();
        // the opponent in the words (Fabian 24.11.: "the same every time, the same for every opponent"): their coach,
        // their best player, both places; title, question and the three answers each from their own bag
        let trainer = { let m = self.world.clubs[g].manager.clone(); if m.is_empty() { gn.clone() } else { m } };
        let star = self.world.clubs[g].squad.iter().copied().max_by(|a, b| self.world.players[*a].ovr.partial_cmp(&self.world.players[*b].ovr).unwrap()).map(|p| self.world.players[p].name.clone()).unwrap_or_else(|| gn.clone());
        let (rg, ro) = (self.world.rank_of(g).to_string(), self.world.rank_of(self.own).to_string());
        let kt = self.vkey("ui.mind.titel", &gn);
        let titel = crate::tf!(&kt, gn.clone());
        let kx = self.vkey(&format!("ui.mind.text.{lage}"), &gn);
        let text = crate::tf!(&kx, gn.clone(), trainer.clone(), star.clone(), rg, ro);
        let mut opts = vec![];
        for key in [crate::id("respekt"), crate::id("selbstbewusst"), crate::id("sticheln")] {
            let k = self.vkey(&format!("ui.mind.opt.{key}"), &gn);
            opts.push((key.to_string(), crate::tf!(&k, gn.clone(), trainer.clone(), star.clone())));
        }
        self.msg(4, crate::inhalt::t("ui.medien.pressestelle"), &titel, &text, Action::Wahl { art: format!("mind:{lage}"), bezug: vec![], optionen: opts });
    }
    pub fn mind_antwort(&mut self, lage: &str, key: &str) -> String {
        let mo = crate::regeln::f_oder("kabine", &format!("mind.{key}.moral.{lage}"), 0.0);
        let me = crate::regeln::f_oder("kabine", &format!("mind.{key}.medien.{lage}"), 0.0) * self.mgr_faktor(3);
        for p in self.xi.clone() { self.moral_wirkung(p, mo, 0, &crate::tf!("ui.rest3.mind_games", key)); }
        if me != 0.0 { self.klima(0.0, me, 0.0, &crate::tf!("ui.rest3.mind_games", key)); }
        self.stil_punkte(crate::id("presse"), if key == crate::id("sticheln") { crate::id("angriff") } else if key == crate::id("respekt") { crate::id("ruhig") } else { crate::id("locker") });
        crate::inhalt::t("ui.rest3.gesagt_ist_gesagt").into()
    }

    // ------------------------------------------------------------ youth network (§19, jugend.toml)
    fn j(k: &str) -> f32 { f("jugend", k) }
    pub fn netz_an(&mut self) {
        if !self.mgr.netz.is_empty() { return; }
        let heim = crate::cups::country(&self.world.leagues[self.league()]).unwrap_or_default();
        let scout: Vec<String> = self.scouts.iter().filter_map(|s| s.league.and_then(|l| crate::cups::country(&self.world.leagues[l]))).collect();
        let mut laender: Vec<String> = self.world.leagues.iter().filter_map(|l| crate::cups::country(&l)).collect();
        laender.sort();
        laender.dedup();
        for l in laender {
            let s = if l == heim { Self::j("netz.start_heimat") } else if scout.contains(&l) { Self::j("netz.start_scout") } else { Self::j("netz.start_fremd") };
            self.mgr.netz.push((l, s, 0));
        }
    }
    fn jugendleiter(&self) -> f32 { self.stab.iter().filter(|s| s.rolle == 4).map(|s| s.f).fold(0.0, f32::max) }
    /// Monthly: investment and youth leader grow a region, without care it decays 0.6; contacts grow; costs are booked.
    pub fn netz_monat(&mut self) {
        self.netz_an();
        let jl = self.jugendleiter() / 100.0 * self.mgr_faktor(4);
        let kosten = arr("jugend", "netz.invest_kosten");
        let zuw = arr("jugend", "netz.invest_zuwachs");
        let mut summe = 0.0;
        for r in self.mgr.netz.iter_mut() {
            let st = r.2 as usize;
            summe += kosten[st].as_f64().unwrap();
            let plus = zuw[st].as_f64().unwrap() as f32 + if st > 0 { Self::j("netz.leiter_zuwachs") * jl } else { 0.0 };
            r.1 = (r.1 + if st > 0 { plus } else { -Self::j("netz.verfall") }).clamp(0.0, 100.0);
        }
        for k in self.mgr.kontakte.iter_mut() { k.1 = (k.1 + Self::j("kontakt.zuwachs") * jl.max(0.8)).min(100.0); }
        if summe > 0.0 { self.book(crate::inhalt::t("ui.main.jugendnetz"), -summe); }
    }
    pub fn netz_stufe(&mut self, land: &str, stufe: u8) { if let Some(r) = self.mgr.netz.iter_mut().find(|r| r.0 == land) { r.2 = stufe.min(3); } }
    pub fn kontakt_knuepfen(&mut self, land: &str) -> Result<(), String> {
        if self.mgr.kontakte.len() as f32 >= Self::j("kontakt.hoechstens") { return Err(crate::inhalt::t("ui.rest3.mehr_kontakte_gehen_nicht").into()); }
        if self.mgr.kontakte.iter().any(|k| k.0 == land) { return Err(crate::inhalt::t("ui.rest3.dort_gibt_es_schon_einen").into()); }
        self.book(crate::inhalt::t("ui.main.jugendnetz"), -(Self::j("kontakt.kosten") as f64));
        self.mgr.kontakte.push((land.to_string(), Self::j("kontakt.start")));
        Ok(())
    }
    /// The talents a region shows this month (from network 20), at most liste_hoechstens over all regions.
    pub fn talente(&self) -> Vec<Talent> {
        let monat = (self.abs() / 30).to_string();
        let mut v = vec![];
        let pos = [crate::id("GK"), crate::id("CB"), crate::id("LB"), crate::id("RB"), crate::id("DMF"), crate::id("CMF"), crate::id("AMF"), crate::id("LMF"), crate::id("RMF"), crate::id("LWF"), crate::id("RWF"), crate::id("CF")];
        for (land, netz, _) in &self.mgr.netz {
            if *netz < Self::j("netz.sicht_ab") { continue; }
            for i in 0..3 {
                let key = format!("{land}|{monat}|{i}");
                let z = |k: &str| zufall(&["2026", crate::id("talent"), &key, k]);
                let ovr = 36.0 + z(crate::id("o")) * 14.0 + netz / 20.0;
                let pot = (ovr + 15.0 + z(crate::id("p")) * 25.0).min(92.0);
                let name = format!("{} {}", [crate::inhalt::t("ui.rest3.ilyas"), crate::inhalt::t("ui.rest3.mattia"), crate::inhalt::t("ui.rest3.kofi"), crate::inhalt::t("ui.rest3.yannick"), crate::inhalt::t("ui.rest3.sami"), crate::inhalt::t("ui.rest3.oskar"), crate::inhalt::t("ui.rest3.rui"), crate::inhalt::t("ui.rest3.lev"), crate::inhalt::t("ui.rest3.dario"), crate::inhalt::t("ui.game.anton")][(z(crate::id("v")) * 10.0) as usize % 10], [crate::inhalt::t("ui.rest3.varga"), crate::inhalt::t("ui.rest3.okoro"), crate::inhalt::t("ui.rest3.silva"), crate::inhalt::t("ui.akademie.novak"), crate::inhalt::t("ui.game.haas"), crate::inhalt::t("ui.rest3.moreau"), crate::inhalt::t("ui.rest3.ricci"), crate::inhalt::t("ui.rest3.berg"), crate::inhalt::t("ui.rest3.petrov"), crate::inhalt::t("ui.rest3.lund")][(z(crate::id("n")) * 10.0) as usize % 10]);
                let (alter, posi) = (15 + (z(crate::id("a")) * 3.0) as u8 % 3, pos[(z(crate::id("x")) * 12.0) as usize % 12].to_string());
                v.push(Talent { key: key.clone(), name, land: land.clone(), alter, ovr, pot, pos: posi, wert: round_money(10_000.0 + (pot as f64 - 55.0).max(0.0) * 15_000.0) });
            }
        }
        v.truncate(Self::j("spieler.liste_hoechstens") as usize);
        v
    }
    /// Recruit a talent (from network 35): 28 days, at most 2 at once; chance 0.25 + 0.45·network + 0.15·contact + 0.2·(own − their reputation).
    pub fn anwerben(&mut self, t: &Talent) -> Result<(), String> {
        let netz = self.mgr.netz.iter().find(|r| r.0 == t.land).map(|r| r.1).unwrap_or(0.0);
        if netz < Self::j("netz.anwerben_ab") { return Err(crate::tf!("ui.rest3.das_netz_in_ist_noch", t.land, format!("{:.0}", Self::j("netz.anwerben_ab")))); }
        if self.mgr.anwerbung.len() as f32 >= Self::j("anwerbung.gleichzeitig") { return Err(crate::inhalt::t("ui.rest3.es_laufen_schon_zwei_anwerbungen").into()); }
        if self.mgr.anwerbung.iter().any(|a| a.0 == t.key) { return Err(crate::inhalt::t("ui.rest3.laeuft_schon").into()); }
        let abs = self.abs();
        self.mgr.anwerbung.push((t.key.clone(), abs + Self::j("anwerbung.dauer_tage") as u32));
        Ok(())
    }
    /// Daily: finished recruitments decide with the fixed chance; a success joins the academy and costs 25 % of his value (at least 10 000 €).
    pub fn anwerbung_tag(&mut self) {
        let abs = self.abs();
        let fertig: Vec<String> = self.mgr.anwerbung.iter().filter(|a| abs >= a.1).map(|a| a.0.clone()).collect();
        self.mgr.anwerbung.retain(|a| abs < a.1);
        for key in fertig {
            let land = key.split('|').next().unwrap_or("").to_string();
            let monat: u32 = key.split('|').nth(1).and_then(|m| m.parse().ok()).unwrap_or(0);
            let netz = self.mgr.netz.iter().find(|r| r.0 == land).map(|r| r.1).unwrap_or(0.0) / 100.0;
            let kontakt = self.mgr.kontakte.iter().find(|k| k.0 == land).map(|k| k.1).unwrap_or(0.0) / 100.0;
            let a = |k: &str| Self::j(&format!("anwerbung.{k}"));
            let chance = a(crate::id("basis")) + a("netz_gewicht") * netz + a("kontakt_gewicht") * kontakt + a("ruf_gewicht") * (self.club().rep - 50.0) / 100.0;
            // rebuild the talent of that month
            let jetzt = self.abs();
            let t = { let mut c2 = self.talente(); c2.retain(|t| t.key == key); c2.pop() }.or_else(|| { let _ = (monat, jetzt); None });
            let Some(t) = t else { continue };
            if zufall(&["2026", crate::id("anwerbung"), &key]) < chance {
                let ent = (t.wert * a(crate::id("entschaedigung")) as f64).max(a("mindest_entschaedigung") as f64);
                self.book(crate::inhalt::t("ui.rest3.abloesen_jugend"), -ent);
                let (attrs, pp) = self.world.template_attrs(&t.pos, t.ovr);
                let id = self.world.new_id();
                let season = self.world.season;
                let pl = Player { nat: 0, id, name: t.name.clone(), pos: t.pos.clone(), age: t.alter, ovr: t.ovr, pot: t.pot, talent: 1.0, no: 0, team: self.own as i32, youth: 17, form: 0.0, fit: 100.0, gf: 0.0, morale: 60.0, inj: 0, inj_name: String::new(), susp: 0,
                    wage: f("akademie", "aufnahme.gehalt") as f64, contract_end: season + 3, value: t.wert, apps: 0, goals: 0, assists: 0, rsum: 0.0, rn: 0, yellow: 0, last: vec![], know: 1.0,
                    listed: false, persona: 0, unhappy_days: 0, benched: 0, promise: 0, program: 0, joined: abs, talk_day: 0, ps: Default::default(), leihe: None, ent: Default::default(), attrs, pp, style: 0, skills: 0, db: false, profil: Default::default() };
                let idx = self.world.players.len();
                self.world.players.push(pl);
                self.world.clubs[self.own].squad.push(idx);
                self.msg(1, crate::inhalt::t("meldung.absender.jugendleiter"), &crate::tf!("ui.rest3.kommt_zu_uns", t.name), &crate::tf!("ui.rest3.jahre_wechselt_in_unsere_akademie", t.name, t.alter, t.land, money(ent)), Action::None);
            } else {
                self.msg(1, crate::inhalt::t("meldung.absender.jugendleiter"), &crate::tf!("ui.rest3.sagt_ab", t.name), crate::inhalt::t("ui.rest3.seine_familie_hat_sich_fuer"), Action::None);
            }
        }
    }

    // ------------------------------------------------------------ applying for a vacant job (laufbahn.toml [bewerbung])
    /// Vacant posts (interim coaches), at most motor.vakanzen_hoechstens, that fit the coach's reputation.
    pub fn vakanzen(&self) -> Vec<usize> {
        let rt = self.mgr_rep / 100.0;
        let luft = f("laufbahn", "interesse.ruf_luft");
        let mut v: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.clubs[c].manager == "Interimstrainer" && self.world.clubs[c].rep / 100.0 <= (rt + luft).min(self.lizenz_grenze())).collect(); // roh
        v.sort_by(|a, b| self.world.clubs[*b].rep.partial_cmp(&self.world.clubs[*a].rep).unwrap());
        v.truncate(f("laufbahn", "motor.vakanzen_hoechstens") as usize);
        v
    }
    /// The coach market (Fabian 23.11.): every vacant post and every coach on a hot seat, with whether we may apply
    /// (reputation and licence) and why not: (club, vacant, reachable, reason key).
    pub fn trainermarkt(&self) -> Vec<(usize, bool, bool, &'static str)> {
        let rt = self.mgr_rep / 100.0;
        let luft = f("laufbahn", "interesse.ruf_luft");
        let mut v: Vec<(usize, bool, bool, &'static str)> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.erste_von(c).is_none() && self.world.leagues[self.world.clubs[c].league].detail <= 1).filter_map(|c| {
            let cl = &self.world.clubs[c];
            let frei = cl.manager == "Interimstrainer"; // roh
            // a hot seat: far below the goal after a few matches, or the board's patience nearly used up
            let gespielt = self.world.table(cl.league).iter().find(|r| r.club == c).map(|r| r.p).unwrap_or(0);
            let wackelt = !frei && ((gespielt >= 6 && self.world.rank_of(c) as i32 - cl.goal_pos as i32 >= 4) || (cl.pressure > 0.0 && cl.pressure >= 0.7 * cl.patience));
            if !frei && !wackelt { return None; }
            let r = cl.rep / 100.0;
            let (ok, grund) = if r > self.lizenz_grenze() { (false, crate::id("lizenz")) } else if r > rt + luft { (false, crate::id("ruf")) } else if !frei { (false, crate::id("besetzt")) } else { (true, "") };
            Some((c, frei, ok, grund))
        }).collect();
        v.sort_by(|a, b| (b.1, b.2).cmp(&(a.1, a.2)).then(self.world.clubs[b.0].rep.partial_cmp(&self.world.clubs[a.0].rep).unwrap()));
        v.truncate(40);
        v
    }
    pub fn persona_von(&self, c: usize) -> &'static str { PERSONA[(zufall(&["2026", crate::id("persona"), &self.world.clubs[c].id.to_string()]) * 4.0) as usize % 4] }
    /// Start an application: four steps (project, playing style, question, reaction) as questions in the inbox.
    pub fn bewerben(&mut self, c: usize) -> Result<(), String> {
        if self.mgr.bewerbung.is_some() { return Err(crate::inhalt::t("ui.rest3.es_laeuft_schon_eine_bewerbung").into()); }
        // applying while in a job gets out: the own board takes it badly, the papers write about it
        if !self.fired {
            let (cn, zn) = (self.club().name.clone(), self.world.clubs[c].name.clone());
            self.klima(-2.0, -1.0, -5.0, crate::inhalt::t("tm.bewerbung_grund"));
            self.news(&crate::tf!("tm.bewerbung_news", self.coach.clone(), zn), &crate::tf!("tm.bewerbung_news_text", cn));
        }
        self.mgr.bewerbung = Some((c, 0, 0.0));
        self.bewerbung_schritt();
        Ok(())
    }
    fn bewerbung_schritt(&mut self) {
        let Some((c, s, _)) = self.mgr.bewerbung else { return };
        let schritt = SCHRITTE[s as usize];
        let opts: Vec<(String, String)> = wert("laufbahn", &format!("bewerbung.wirkung.{schritt}")).as_object().unwrap().keys().map(|k| (k.clone(), match k.as_str() {
            "entwicklung" => crate::inhalt::t("ui.rest3.ich_will_den_verein_entwickeln"), "erfolg" => crate::inhalt::t("ui.rest3.ich_will_sofort_erfolg"), "stabilitaet" => crate::inhalt::t("ui.rest3.ich_bringe_ruhe_und_stabilitaet"), "jugend" => crate::inhalt::t("ui.rest3.ich_setze_auf_die_eigene"), "neuaufbau" => crate::inhalt::t("ui.rest3.ich_baue_die_mannschaft_neu"),
            "eigener" => crate::inhalt::t("ui.rest3.ich_bringe_meinen_eigenen_stil"), "anpassen" => crate::inhalt::t("ui.rest3.ich_passe_mich_dem_kader"), "ueberzeugen" => crate::inhalt::t("ui.rest3.ich_ueberzeuge_die_spieler_von"),
            "ehrlich" => crate::inhalt::t("ui.rest3.ehrlich_antworten"), "ausweichend" => crate::inhalt::t("ui.rest3.ausweichend_antworten"), "versprechen" => crate::inhalt::t("ui.rest3.grosse_versprechen_machen"),
            "loyal" => crate::inhalt::t("ui.rest3.ich_bleibe_dem_verein_treu"), "offen" => crate::inhalt::t("ui.rest3.ich_bin_offen_fuer_alles"), _ => crate::inhalt::t("ui.rest3.ich_wiegle_ab"),
        }.to_string())).collect();
        let n = self.world.clubs[c].name.clone();
        let text = [crate::inhalt::t("ui.rest3.erzaehlen_sie_uns_von_ihrem"), crate::inhalt::t("ui.rest3.welchen_fussball_wollen_sie_spielen"), crate::inhalt::t("ui.rest3.eine_rueckfrage_des_praesidiums"), crate::inhalt::t("ui.rest3.wie_reagieren_sie_auf_gegenwind")][s as usize];
        self.msg(5, &n, &crate::tf!("ui.rest3.bewerbung_bei_von_4", n, s + 1), text, Action::Wahl { art: format!("bewerbung:{c}:{s}"), bezug: vec![], optionen: opts });
    }
    /// One answer: its effect for the club's persona (−1…+1); after the fourth: verdict (Zusage from 0.62, hinhalten from 0.42).
    pub fn bewerbung_antwort(&mut self, key: &str) -> String {
        let Some((c, s, sum)) = self.mgr.bewerbung else { return crate::inhalt::t("ui.rest3.die_bewerbung_ist_vorbei").into() };
        let persona = self.persona_von(c);
        let w = crate::regeln::f_oder("laufbahn", &format!("bewerbung.wirkung.{}.{key}.{persona}", SCHRITTE[s as usize]), 0.0);
        let mut sum = sum + w;
        if s == 3 && zufall(&["2026", crate::id("vorfall"), &c.to_string(), &self.abs().to_string()]) < 0.33 { sum -= f("laufbahn", "bewerbung.vorfall.wirkung"); }
        if s < 3 { self.mgr.bewerbung = Some((c, s + 1, sum)); self.bewerbung_schritt(); return crate::inhalt::t("ui.rest3.weiter_geht_s").into(); }
        self.mgr.bewerbung = None;
        let b = |k: &str| f("laufbahn", &format!("bewerbung.{k}"));
        let rt = self.mgr_rep / 100.0;
        let passung = (1.0 - (self.world.clubs[c].rep / 100.0 - rt).abs() * 2.0).clamp(0.0, 1.0);
        let argumente = ((sum / 4.0) + 1.0) / 2.0;
        let wert = b("w_passung") * passung + b("w_argumente") * argumente + b("w_ruf") * rt;
        let n = self.world.clubs[c].name.clone();
        if wert >= b("zusage_ab") {
            let wage = self.trainergehalt(c);
            self.msg(5, &n, &crate::tf!("lb.bewerbung.zusage.betreff", n), &crate::tf!("ui.rest3.will_dich_als_trainer_im", n, money(wage)), Action::JobOffer { club: c, wage });
            crate::inhalt::t("ui.rest3.zusage").into()
        } else if wert >= b("hinhalten_ab") {
            crate::inhalt::t("ui.rest3.der_verein_will_sich_noch").into()
        } else { crate::tf!("ui.rest3.sagt_ab_2", n) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manager_netz_bewerbung() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.manager_an();
        let e0 = c.mgr_wert(0);
        for _ in 0..12 { c.manager_woche(); }
        assert!(c.mgr.stress > 70.0 && c.mgr_wert(0) < e0, "exhaustion lowers the skills");
        c.freizeit().unwrap();
        c.stil_punkte("ton", "hart");
        c.stil_punkte("ton", "hart");
        c.stil_punkte("ton", "hart");
        assert_eq!(c.stil(), Some(0));
        c.netz_an();
        let heim = c.mgr.netz.iter().find(|r| r.1 >= 25.0).map(|r| r.0.clone()).unwrap();
        assert!(c.talente().iter().any(|t| t.land == heim));
        c.netz_stufe(&heim, 3);
        for _ in 0..3 { c.netz_monat(); }
        let t = c.talente().into_iter().find(|t| t.land == heim).unwrap();
        c.anwerben(&t).unwrap();
        // a vacancy, then four answers
        c.world.clubs[5].manager = "Interimstrainer".into();
        c.mgr_rep = 90.0;
        c.bewerben(5).unwrap();
        for _ in 0..4 { let k = c.inbox.iter().find_map(|m| if let Action::Wahl { art, optionen, .. } = &m.action { art.starts_with("bewerbung").then(|| optionen[0].0.clone()) } else { None }).unwrap(); c.inbox.retain(|m| !matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("bewerbung"))); c.bewerbung_antwort(&k); }
        assert!(c.mgr.bewerbung.is_none());
    }
}
