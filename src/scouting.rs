//! Paket 3g: scouting with real scouts, assignments and search profiles (Mechanik-Übergabe §10, scouting.toml).
//! Knowledge of a player is `Player::know` (0…1 = Wissen 0…100).
use crate::game::*;
use crate::mensch::zufall;
use crate::regeln::f;
use crate::world::*;
use serde::{Deserialize, Serialize};

pub fn faehigkeiten_t() -> [&'static str; 4] { [crate::inhalt::t("sc.scout.f.urteil"), crate::inhalt::t("profil.potenzial"), crate::inhalt::t("pm.f.menschenkenntnis"), crate::inhalt::t("menu.jugendnetzwerk.reiter.0")] }
pub fn gruppen_t() -> [&'static str; 5] { [crate::inhalt::t("gb2.sp.teil.tor"), crate::inhalt::t("achse.Abwehr"), crate::inhalt::t("gb2.vgl.mittel"), crate::inhalt::t("bericht.achse.angriff"), crate::inhalt::t("profil.jugend")] }
pub const TYPEN: [&str; 4] = [crate::id("vorsichtig"), crate::id("euphorisch"), crate::id("nuechtern"), crate::id("kritisch")];
pub fn typ_wort_t() -> [&'static str; 4] { [crate::inhalt::t("ui.scouting.vorsichtig"), crate::inhalt::t("vw.wi.w.stim.4"), crate::inhalt::t("ui.scouting.nuechtern"), crate::inhalt::t("md.ui.journalist.stil.kritisch")] }
pub const DAUER: [u32; 4] = [7, 14, 21, 28];
pub const ALTER_MAX: [u8; 4] = [99, 21, 25, 29];

/// A scout as a person (scouting/scoutperson.py).
#[derive(Serialize, Deserialize, Clone)]
pub struct Person {
    pub f: [f32; 4],
    pub spezial: u8,
    pub typ: u8,
    pub gehalt: f64,
    pub bis: u32,
    pub vertrauen: f32,
    /// Running assignment: until abs day, max age of the search profile.
    pub auftrag_bis: u32,
    pub alter_max: u8,
    /// The search profile of the running assignment (suchprofil.rs).
    #[serde(default)]
    pub profil: Option<crate::suchprofil::Suchprofil>,
    /// His task in the staff (Konzept 3.1): 0 regional scout, 2 data analyst.
    #[serde(default)]
    pub aufgabe: u8,
    /// How often his estimates held after 8 weeks (Konzept 4.3 "Trefferquote").
    #[serde(default)]
    pub treffer: u16,
    #[serde(default)]
    pub fehler: u16,
    /// Reports to check after 8 weeks: (player, estimate, abs day).
    pub pruefen: Vec<(usize, f32, u32)>,
}
pub fn gehalt(fs: &[f32; 4]) -> f64 {
    let m = fs.iter().sum::<f32>() / 4.0;
    (f("scouting", "scout.gehalt_basis") + f("scouting", "scout.gehalt_je_punkt") * (m - 50.0)).max(20000.0) as f64
}
/// Reach by club reputation level 1…5 (scouting.toml [reichweite]).
pub fn stufe(rep: f32) -> usize { ((rep / 20.0) as usize).min(4) }
/// Stars 0…5 in half steps from rating 55 to 90 ([sterne]).
pub fn sterne(v: f32) -> f32 { (((v - f("scouting", "sterne.von")) / (f("scouting", "sterne.bis") - f("scouting", "sterne.von")) * 10.0).round() / 2.0).clamp(0.0, 5.0) }
/// "3½ Sterne": the fonts have no star glyph, so stars are written as a number (pictures use Ui::sterne).
pub fn sterne_text(s: f32) -> String { let g = s.floor() as usize; let h = if s - g as f32 >= 0.5 { "½" } else { "" }; let n = if g == 0 { h.to_string() } else { format!("{g}{h}") }; format!("{n} {}", if g <= 1 && h.is_empty() { crate::inhalt::t("ui.scouting.stern") } else { crate::inhalt::t("ui.scouting.sterne") }) }

impl Career {
    pub fn person(&mut self, s: usize) -> &mut Person {
        if self.scouts[s].p.is_none() {
            let q = self.scouts[s].q as f32;
            let fs = [40.0 + q * 4.0, 38.0 + q * 4.0, 45.0 + q * 2.0, 40.0 + q * 3.0];
            let abs = self.abs();
            self.scouts[s].p = Some(Person { f: fs, spezial: (s % 5) as u8, typ: 2, gehalt: gehalt(&fs), bis: abs + 2 * crate::date::SEASON_DAYS, vertrauen: f("scouting", "vertrauen.start"), auftrag_bis: 0, alter_max: 99, profil: None, pruefen: vec![], treffer: 0, fehler: 0, aufgabe: 0 });
        }
        self.scouts[s].p.as_mut().unwrap()
    }
    pub fn scout_stufe(&self) -> usize { stufe(self.club().rep) }
    pub fn scouts_max(&self) -> usize { crate::mensch::arr("scouting", "reichweite.scouts_max")[self.scout_stufe()].as_u64().unwrap() as usize }
    /// Five applicants a month, fixed per month (scout.bewerber_je_monat).
    pub fn bewerber(&self) -> Vec<(String, Person)> {
        let monat = (self.abs() / 30).to_string();
        let vn = ["Lars", "Diego", "Kofi", "Henrik", "Paolo", "Jonas", "Malik", "Tomás", "Yusuf", "Pierre", "Andrej", "Liam"]; // roh
        let nn = ["Brandt", "Okafor", "Lindgren", "Moretti", "Duarte", "Nowak", "Keller", "Haddad", "Ferreira", "Søndergaard", "Petrov", "Walsh"]; // roh
        let rep = self.club().rep;
        (0..f("scouting", "scout.bewerber_je_monat") as usize).map(|i| {
            let z = |k: &str| zufall(&["2026", crate::id("scout"), &monat, &i.to_string(), k]);
            let basis = 35.0 + rep * 0.35;
            let fs = [0, 1, 2, 3].map(|k| (basis + z(&format!("f{k}")) * 30.0).clamp(20.0, 95.0).round());
            let name = format!("{} {}", vn[(z(crate::id("v")) * vn.len() as f32) as usize % vn.len()], nn[(z(crate::id("n")) * nn.len() as f32) as usize % nn.len()]);
            (name, Person { f: fs, spezial: (z(crate::id("s")) * 5.0) as u8 % 5, typ: (z(crate::id("t")) * 4.0) as u8 % 4, gehalt: gehalt(&fs), bis: 0, vertrauen: f("scouting", "vertrauen.start"), auftrag_bis: 0, alter_max: 99, profil: None, pruefen: vec![], treffer: 0, fehler: 0, aufgabe: 0 })
        }).collect()
    }
    pub fn scout_einstellen(&mut self, i: usize) -> Result<(), String> {
        if self.scouts.len() >= self.scouts_max() { return Err(crate::tf!("ui.scouting.mehr_als_scouts_erlaubt_der", self.scouts_max())); }
        let (name, mut p) = self.bewerber().swap_remove(i);
        if self.scouts.iter().any(|s| s.name == name) { return Err(crate::inhalt::t("ui.personal.er_arbeitet_schon_fuer_uns").into()); }
        p.bis = self.abs() + f("scouting", "scout.vertrag_jahre") as u32 * crate::date::SEASON_DAYS;
        let q = ((p.f[0] + p.f[1]) / 20.0).round().clamp(1.0, 10.0) as u8;
        let g = p.gehalt;
        self.scouts.push(Scout { name: name.clone(), q, league: Some(self.league()), pos: 0, watch: None, seen: 0, p: Some(p) });
        self.grund(&name, crate::inhalt::t("ui.pages.scout_eingestellt"), 0.0, &crate::tf!("tv.klausel.gehalt", money(g)));
        Ok(())
    }
    pub fn scout_entlassen(&mut self, s: usize) {
        let rest = self.person(s).bis.saturating_sub(self.abs()) as f64 / crate::date::SEASON_DAYS as f64;
        let g = self.person(s).gehalt;
        let n = self.scouts[s].name.clone();
        self.book(crate::inhalt::t("ui.personal.abfindungen"), -(g * rest * 0.5));
        self.scouts.remove(s);
        self.grund(&n, crate::inhalt::t("ui.pages.scout_entlassen"), 0.0, crate::inhalt::t("ui.scouting.vertrag_aufgeloest_halbe_restsumme_als"));
    }
    /// Start an assignment: region (league), position group, duration, max age; costs per week by knowledge of the region.
    /// Can we pay him? His club's asking price within the transfer budget (a free agent always).
    pub fn bezahlbar(&self, p: usize) -> bool { self.world.players[p].team < 0 || self.ask_price(p) <= self.transfer_budget.max(0.0) }
    pub fn auftrag(&mut self, s: usize, dauer: u32, alter_max: u8) -> Result<(), String> {
        let abs = self.abs();
        let laufend = self.scouts.iter().filter(|x| x.p.as_ref().map_or(false, |p| p.auftrag_bis > abs)).count();
        if laufend >= self.auftraege_max() { return Err(crate::tf!("ui.scouting.mehr_als_auftraege_gleichzeitig_erlaubt", self.auftraege_max())); }
        let p = self.person(s);
        p.auftrag_bis = abs + dauer;
        p.alter_max = alter_max;
        self.erste_woche_schritt(3);
        Ok(())
    }
    /// Konzept 1.1: what PES M knows about a player without any scouting — the most of reputation (star players are
    /// known), league (own league, same country, top-5) and own club. scouting.toml [grundwissen].
    pub fn grundwissen(&self, p: usize) -> f32 {
        let pl = &self.world.players[p];
        if pl.team < 0 { return 0.0; }
        let g = |k: &str| f("scouting", &format!("grundwissen.{k}"));
        let cl = &self.world.clubs[pl.team as usize];
        let ruf = pl.ovr * g("ruf_ovr") + cl.rep * (1.0 - g("ruf_ovr"));
        let ruf_w = crate::mensch::arr("scouting", "grundwissen.ruf_ab").iter().zip(crate::mensch::arr("scouting", "grundwissen.ruf_wissen").iter())
            .find(|(ab, _)| ruf >= ab.as_f64().unwrap() as f32).map_or(0.0, |(_, w)| w.as_f64().unwrap() as f32);
        let stamm = pl.ovr >= cl.level - g("stamm_unter_niveau");
        let l = cl.league;
        let liga_w = if l == self.league() { if stamm { g("eigene_liga_stamm") } else { g("eigene_liga_ersatz") } }
            else if l < self.world.leagues.len() && (self.world.leagues[l].land == self.world.leagues[self.league()].land || ["ENGLAND_D1_LEAGUE", "SPAIN_D1_LEAGUE", "GERMANY_D1_LEAGUE", "ITALY_D1_LEAGUE", "FRANCE_D1_LEAGUE"].contains(&self.world.leagues[l].key.as_str())) {
                if stamm { g("nah_stamm") } else { g("nah_ersatz") }
            } else { 0.0 };
        ruf_w.max(liga_w) / 100.0
    }
    /// Konzept 1.1, once a month: the base knowledge follows transfers and form, and knowledge above it fades by 2 a
    /// month once nobody has watched the player for 6 months — never below the base, never below 40 once it reached 75.
    pub fn wissen_monat(&mut self) {
        let abs = self.abs();
        let g = |k: &str| f("scouting", &format!("verfall.{k}"));
        for p in 0..self.world.players.len() {
            if self.world.players[p].team < 0 || self.gehoert_uns(p) { continue; }
            let grund = self.grundwissen(p);
            let id = self.world.players[p].id;
            let (zuletzt, hoch) = self.wissen_info.get(&id).copied().unwrap_or((0, false));
            let k = self.world.players[p].know;
            let mut neu = k.max(grund);
            if k > grund && abs.saturating_sub(zuletzt) >= g("nach_tagen") as u32 {
                let boden = grund.max(if hoch { g("boden_nach_genau") / 100.0 } else { 0.0 });
                neu = (k - g("je_monat") / 100.0).max(boden).max(grund);
            }
            self.world.players[p].know = neu;
        }
    }
    /// A scout looked at the player: remember when, and whether he was ever known exactly.
    pub fn wissen_gesehen(&mut self, p: usize) {
        let (id, k, abs) = (self.world.players[p].id, self.world.players[p].know, self.abs());
        let e = self.wissen_info.entry(id).or_insert((0, false));
        e.0 = abs;
        e.1 |= k >= f("scouting", "verfall.genau_ab") / 100.0;
    }
    /// Half span of the rating estimate (scouting.toml [bericht]): 12 at knowledge 0 down to 1, × judgement and personality.
    pub fn spanne(&self, p: usize) -> f32 {
        let k = self.know_of(p);
        let (mx, mn) = (f("scouting", "bericht.spanne_max"), f("scouting", "bericht.spanne_min"));
        let urteil = self.scouts.iter().filter_map(|s| s.p.as_ref()).map(|x| x.f[0]).fold(50.0f32, f32::max);
        let typ = self.scouts.iter().filter_map(|s| s.p.as_ref()).max_by(|a, b| a.f[0].partial_cmp(&b.f[0]).unwrap()).map(|x| x.typ).unwrap_or(2);
        let sp = crate::regeln::f_oder("scouting", &format!("persoenlichkeit.{}.spanne", TYPEN[typ as usize]), 1.0);
        (mx - (mx - mn) * k) * (1.0 + f("scouting", "bericht.urteil_wirkung") * (0.5 - urteil / 100.0)) * sp
    }
    /// The scout's colour (scoutperson.py:26): the middle moves by his bias (euphorisch +3, vorsichtig −2, kritisch −1.5),
    /// at most bias_anteil of the half span, so the range still holds the true value.
    fn scout_bias(&self, halb: f32) -> f32 {
        let typ = self.scouts.iter().filter_map(|s| s.p.as_ref()).max_by(|a, b| a.f[0].partial_cmp(&b.f[0]).unwrap()).map(|x| x.typ).unwrap_or(2);
        let b = crate::regeln::f_oder("scouting", &format!("persoenlichkeit.{}.bias", TYPEN[typ as usize]), 0.0);
        let a = f("scouting", "persoenlichkeit.bias_anteil");
        b.clamp(-a * halb, a * halb).round()
    }
    /// What the knowledge shows (scouting.toml sterne.stufen 5/40/60/75/92): 0 nothing, 1 stars with range,
    /// 2 30 % of the abilities, 3 60 % and character, 4 all abilities, 5 traits and injury proneness.
    pub fn sicht(&self, p: usize) -> usize {
        let w = self.know_of(p) * 100.0;
        crate::mensch::arr("scouting", "sterne.stufen").iter().filter(|x| w >= x.as_f64().unwrap() as f32).count()
    }
    /// Range of the price expectation in talks (scouting.toml [vorsprung]): ±25 % from 40, ±12 % from 60, exact from 80, rough ±35 % before.
    pub fn preis_spanne(&self, p: usize) -> f32 {
        let w = self.know_of(p) * 100.0;
        let st = crate::mensch::arr("scouting", "vorsprung.stufen");
        let sp = crate::mensch::arr("scouting", "vorsprung.spanne");
        let mut r = f("scouting", "vorsprung.grob");
        for i in 0..st.len() { if w >= st[i].as_f64().unwrap() as f32 { r = sp[i].as_f64().unwrap() as f32; } }
        r
    }
    pub fn auftraege_max(&self) -> usize { crate::mensch::arr("scouting", "reichweite.auftraege_max")[self.scout_stufe()].as_u64().unwrap() as usize }
    /// Foreign countries a club of this level may scout (regionen_max), the world scout from level 3.
    pub fn gebiet_erlaubt(&self, s: usize, l: Option<usize>) -> bool {
        let st = self.scout_stufe();
        let Some(l) = l else { return st + 1 >= f("scouting", "reichweite.weltscout_ab_stufe") as usize };
        let land = |l: usize| crate::cups::country(&self.world.leagues[l]).unwrap_or_default();
        let heim = land(self.league());
        if land(l) == heim { return true; }
        let max = crate::mensch::arr("scouting", "reichweite.regionen_max")[st].as_u64().unwrap() as usize;
        let mut fremd: Vec<String> = self.scouts.iter().enumerate().filter(|(i, x)| *i != s && x.league.map_or(false, |k| land(k) != heim)).map(|(_, x)| land(x.league.unwrap())).collect();
        fremd.sort();
        fremd.dedup();
        fremd.contains(&land(l)) || fremd.len() < max
    }
    pub fn ovr_range(&self, p: usize) -> (f32, f32) {
        let pl = &self.world.players[p];
        if self.know_of(p) >= 0.999 { return (pl.ovr.round(), pl.ovr.round()); }
        let s = self.spanne(p);
        let off = (zufall(&["2026", crate::id("schaetzung"), &pl.id.to_string()]) - 0.5) * s * (1.0 - f("scouting", "persoenlichkeit.bias_anteil")) + self.scout_bias(s);
        ((pl.ovr + off - s).round().max(30.0), (pl.ovr + off + s).round().min(99.0))
    }
    pub fn pot_range(&self, p: usize) -> (f32, f32) {
        let pl = &self.world.players[p];
        let own = self.gehoert_uns(p);
        // the staff does not know a young player's potential for sure either ([sterne] pa_unsicher_*)
        let s = if own { if (pl.age as f32) <= f("scouting", "sterne.pa_unsicher_alter") && (pl.apps as f32) < f("scouting", "sterne.pa_unsicher_einsaetze") { f("scouting", "sterne.pa_unsicher_punkte") } else { 0.0 } } else { self.spanne(p) * f("scouting", "sterne.pa_scout_faktor") };
        let off = (zufall(&["2026", "schaetzung_pot", &pl.id.to_string()]) - 0.5) * s;
        ((pl.pot + off - s).max(pl.ovr - 2.0).round(), (pl.pot + off + s).min(99.0).round())
    }
    /// Weekly work of the scouts: knowledge growth (4 per day × skill, region 0.9, video 0.5), reports from 40,
    /// assignment results, salaries and expenses, trust checks after 8 weeks.
    pub fn scouting_week(&mut self) {
        let abs = self.abs();
        self.analyst_woche();
        if abs % 28 < 7 { self.wissen_monat(); }
        let je_tag = f("scouting", "wissen.je_tag");
        let own_league = self.league();
        for s in 0..self.scouts.len() {
            let (league, pos, watch) = (self.scouts[s].league, self.scouts[s].pos, self.scouts[s].watch);
            let pe = self.person(s).clone();
            self.book(crate::inhalt::t("profil.teil.8"), -(pe.gehalt / 52.0));
            let spez = |pl: &Player| { let g = if pl.age <= 19 { 4 } else { match part_of(&pl.pos) { 0 => 0, 1 => 1, 2 => 2, _ => 3 } }; if g == pe.spezial as usize { f("scouting", "scout.spezial_passend") } else { f("scouting", "scout.spezial_fremd") } };
            if let Some(w) = watch {
                let pl = &self.world.players[w];
                let gain = je_tag * 7.0 * pe.f[0] / 100.0 * spez(pl) / 100.0 + if pl.apps > 0 { f("scouting", "wissen.live_gewinn") / 100.0 } else { 0.0 };
                let k = (self.world.players[w].know + gain).min(1.0);
                self.world.players[w].know = k;
                self.wissen_gesehen(w);
                if k >= f("scouting", "wissen.genau_ab") / 100.0 {
                    self.scouts[s].watch = None;
                    self.scout_bericht(s, w, crate::inhalt::t("ui.scouting.bericht"));
                }
            }
            // a world scout (no fixed region) works everywhere from level 3; he sees the league of the week
            let l = match league { Some(l) => l, None if self.gebiet_erlaubt(s, None) => (zufall(&["2026", "weltscout", &s.to_string(), &(abs / 7).to_string()]) * self.world.leagues.len() as f32) as usize % self.world.leagues.len(), None => continue };
            let aktiv = pe.auftrag_bis > abs;
            if pe.auftrag_bis > 0 && !aktiv { continue; }
            let ferne = if l == own_league { 0 } else { 1 };
            if aktiv { let sp = crate::mensch::arr("scouting", "einsatz.spesen_woche")[ferne].as_f64().unwrap(); self.book(crate::inhalt::t("profil.teil.8"), -sp); }
            let heimat = if l == own_league { f("scouting", "wissen.heimat_faktor") } else { crate::mensch::arr("scouting", "reichweite.ausland")[self.scout_stufe()].as_f64().unwrap() as f32 };
            let mut cands: Vec<usize> = vec![];
            for &t in &self.world.leagues[l].teams {
                if t == self.own { continue; }
                for &p in &self.world.clubs[t].squad {
                    let pl = &self.world.players[p];
                    let ok = match pos { 1 => pl.pos == "GK", 2 => part_of(&pl.pos) == 1, 3 => part_of(&pl.pos) == 2, 4 => part_of(&pl.pos) == 3, _ => true } && pl.age <= pe.alter_max && (!aktiv || pe.profil.as_ref().map_or(true, |k| self.profil_passt(p, k, false)));
                    if ok && pl.know < 0.95 { cands.push(p); }
                }
            }
            let woche = (abs / 7).to_string();
            let per = 3 + (zufall(&["2026", crate::id("sichtung"), &s.to_string(), &woche]) * 6.0) as usize;
            let mut gesehen = vec![];
            for j in 0..per.min(cands.len()) {
                let i = (zufall(&["2026", crate::id("sichtung"), &s.to_string(), &woche, &j.to_string()]) * cands.len() as f32) as usize % cands.len();
                let p = cands.swap_remove(i);
                let pl = &self.world.players[p];
                let gain = je_tag * 7.0 * pe.f[0] / 100.0 * heimat * f("scouting", "wissen.video_faktor") * spez(pl) / 100.0 * (1.0 + pe.f[3] / 200.0);
                self.world.players[p].know = (self.world.players[p].know + gain).min(1.0);
                self.scouts[s].seen += 1;
                self.wissen_gesehen(p);
                gesehen.push(p);
            }
            if aktiv && pe.auftrag_bis <= abs + 7 {
                // assignment ends this week: the best hits as reports (bericht.treffer_hoechstens)
                // only players the transfer budget can pay (Fabian 23.11.)
                let mut best: Vec<(usize, f32)> = gesehen.iter().chain(cands.iter()).copied().filter(|&p| self.world.players[p].know >= f("scouting", "wissen.bericht_ab") / 100.0 && self.bezahlbar(p))
                    .map(|p| { let pl = &self.world.players[p]; (p, pl.ovr - self.club().level + self.moneyball(p) * 0.5 + if pl.age <= 21 { (pl.pot - pl.ovr) * 0.3 } else { 0.0 }) }).collect();
                best.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                best.dedup_by_key(|x| x.0);
                for (p, _) in best.into_iter().take(3) { self.scout_bericht(s, p, crate::inhalt::t("ui.scouting.auftrag_erledigt")); }
            } else if let Some(&p) = gesehen.iter().filter(|&&p| self.world.players[p].know >= f("scouting", "wissen.eindruck_ab") / 100.0 && self.bezahlbar(p)).max_by(|&&a, &&b| (self.world.players[a].ovr + self.moneyball(a) * 0.5).partial_cmp(&(self.world.players[b].ovr + self.moneyball(b) * 0.5)).unwrap()) {
                if self.world.players[p].ovr + self.moneyball(p) * 0.5 > self.club().level - 4.0 { self.scout_bericht(s, p, if self.moneyball(p) >= 4.0 { crate::inhalt::t("ui.scouting.geheimtipp") } else { crate::inhalt::t("ui.scouting.bericht") }); }
            }
            // trust: checked after 8 weeks, hit (within 3 points) +3, miss −4 ([vertrauen])
            let mut v = self.person(s).vertrauen;
            let pr = std::mem::take(&mut self.person(s).pruefen);
            let mut rest = vec![];
            for (p, est, d) in pr {
                if abs < d + 56 { rest.push((p, est, d)); continue; }
                let ok = (self.world.players[p].ovr - est).abs() <= 3.0;
                v = (v + if ok { 3.0 } else { -4.0 }).clamp(0.0, 100.0);
                if ok { self.person(s).treffer += 1; } else { self.person(s).fehler += 1; }
            }
            let pe = self.person(s);
            pe.pruefen = rest;
            pe.vertrauen = v;
        }
        let abs = self.abs();
        let ablauf: Vec<usize> = (0..self.scouts.len()).filter(|&s| self.scouts[s].p.as_ref().map_or(false, |p| p.bis > 0 && abs >= p.bis)).collect();
        for s in ablauf.into_iter().rev() {
            let n = self.scouts[s].name.clone();
            self.scouts.remove(s);
            self.msg(6, &n, &crate::tf!("ui.scouting.verlaesst_uns", n), crate::inhalt::t("ui.game.sein_vertrag_ist_ausgelaufen"), Action::None);
        }
    }
    fn scout_bericht(&mut self, s: usize, p: usize, art: &str) {
        // the verdict of the scouting concept (scoutbericht.rs) under the short estimate
        let body = format!("{}\n\n{}", self.report_text(p), self.urteil_text(p));
        let (lo, hi) = self.ovr_range(p);
        let abs = self.abs();
        self.person(s).pruefen.push((p, (lo + hi) / 2.0, abs));
        let (n2, sn) = (self.world.players[p].name.clone(), self.scouts[s].name.clone());
        self.msg(6, &sn, &format!("{art}{n2}"), &body, Action::Report { pid: p });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grundwissen_und_verfall() {
        let w = World::load(include_str!("../assets/world.json"), 7);
        let mut c = Career::new(w, 0, "Test");
        // the own league is known, a star more than a bench player of the same club
        let eigene: Vec<usize> = c.world.leagues[c.league()].teams.iter().filter(|&&t| t != c.own).flat_map(|&t| c.world.clubs[t].squad.clone()).collect();
        assert!(eigene.iter().all(|&p| c.grundwissen(p) >= 0.35));
        let p = eigene[0];
        c.world.players[p].know = 0.9;
        c.wissen_gesehen(p);
        c.world.day += 200;
        c.wissen_monat();
        let k = c.world.players[p].know;
        assert!(k < 0.9 && k >= 0.40f32.max(c.grundwissen(p)), "fades, but not below 40 once known exactly: {k}");
    }
    #[test]
    fn sterne_und_spanne() {
        assert_eq!(sterne(55.0), 0.0);
        assert_eq!(sterne(90.0), 5.0);
        assert_eq!(sterne(72.5), 2.5);
        assert!((gehalt(&[50.0; 4]) - 40000.0).abs() < 1.0);
        assert!((gehalt(&[60.0; 4]) - 65000.0).abs() < 1.0);
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        let p = c.world.clubs[1].squad[0];
        c.world.players[p].know = 0.0;
        let (a, b) = c.ovr_range(p);
        c.world.players[p].know = 0.9;
        let (x, y) = c.ovr_range(p);
        assert!(b - a > y - x, "more knowledge, narrower range");
        assert!(c.bewerber().len() == 5);
        // colour of the scout: euphorisch +3, at most 40 % of the half span (scoutperson.py:26)
        let pe = Person { f: [99.0, 50.0, 50.0, 50.0], spezial: 0, typ: 1, gehalt: 1.0, bis: 0, vertrauen: 50.0, auftrag_bis: 0, alter_max: 99, profil: None, pruefen: vec![], treffer: 0, fehler: 0, aufgabe: 0 };
        c.scouts.push(Scout { name: "E".into(), q: 9, league: None, pos: 0, watch: None, seen: 0, p: Some(pe) });
        assert_eq!(c.scout_bias(10.0), 3.0);
        assert_eq!(c.scout_bias(5.0), 2.0);
        assert_eq!(c.sicht(p), 4, "knowledge 90: all abilities, not yet the traits");
        assert!((c.preis_spanne(p) - 0.0).abs() < 1e-6);
        c.scouts.pop();
        c.scout_einstellen(0).ok();
        for _ in 0..10 { c.scouting_week(); }
    }
}
