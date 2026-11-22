//! Vorbereitung (Punkt 14, mit Fabians Antworten): friendlies (simulate by default, or played in FL27 by the person with
//! the score entered by hand), trips with small tournaments of 2 to 4 teams (assets/regeln/turniere.toml, real and
//! invented names, editable), team events with rare, moderate scandals at the team night and the end-of-season trip,
//! and the return from the summer holiday by professionalism with the event „Übergewicht“.
//! Numbers in assets/regeln/vorbereitung.toml.
use crate::game::*;
use crate::mensch::{arr, cw, zufall, PROF};
use crate::world::money;
use serde::{Deserialize, Serialize};

fn f(k: &str) -> f32 { crate::regeln::f("vorbereitung", k) }
fn t(k: &str) -> &'static str { crate::inhalt::t(k) }

/// A date of the preparation: a friendly (art 0) or a trip with a tournament (art 1, `turnier` = its id).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Termin {
    pub tag: u32,
    pub art: u8,
    pub gegner: usize,
    pub turnier: String,
    /// Played in FL27 by the person (score by hand) instead of simulated.
    pub selbst: bool,
    pub ergebnis: Option<(u8, u8)>,
    pub text: String,
    /// Length of a training camp (art 2, `turnier` = its place in trainingslager.toml).
    pub tage: u32,
    /// What happened in the camp, for its report.
    pub bericht: Vec<String>,
}

/// The tournaments and trips of turniere.toml.
pub fn turniere() -> &'static Vec<serde_json::Value> { arr("turniere", "turnier") }
pub fn events() -> &'static Vec<serde_json::Value> { arr("vorbereitung", "event") }
/// The training camp places of trainingslager.toml.
pub fn lager_orte() -> &'static Vec<serde_json::Value> { arr("trainingslager", "ort") }
fn lg(k: &str) -> f32 { crate::regeln::f("trainingslager", &format!("regeln.{k}")) }
fn ort(id: &str) -> Option<&'static serde_json::Value> { lager_orte().iter().find(|x| x["id"].as_str() == Some(id)) }
fn zahl(o: &serde_json::Value, k: &str) -> f32 { o[k].as_f64().unwrap_or(0.0) as f32 }

impl Career {
    /// Is the day free for a friendly or the start of a trip? No own match that day, 2 days before or 1 day after.
    pub fn tag_frei(&self, d: u32, dauer: u32) -> bool {
        let (vor, nach) = (f("testspiel.abstand_davor") as u32, f("testspiel.abstand_danach") as u32);
        let belegt = |x: u32| self.own_fixtures().iter().any(|(_, fx)| fx.day + nach >= x && x + vor >= fx.day) || self.termine.iter().any(|tm| tm.ergebnis.is_none() && x >= tm.tag && x < tm.tag + tm.tage.max(1));
        d > self.world.day && d < crate::date::SEASON_DAYS && (0..dauer.max(1)).all(|i| !belegt(d + i))
    }
    /// The next free days for a friendly (from the day after tomorrow, at most n).
    pub fn freie_tage(&self, n: usize, dauer: u32) -> Vec<u32> { (self.world.day + 2..self.world.day + 60).filter(|&d| self.tag_frei(d, dauer)).take(n).collect() }
    /// Three opponents for a friendly: weaker, about as strong, stronger (by squad strength, nearby leagues first).
    pub fn testspiel_gegner(&self) -> Vec<usize> {
        let lv = self.club().level;
        let mut v: Vec<usize> = (0..self.world.clubs.len()).filter(|&k| k != self.own && self.world.erste_von(k).is_none()).collect();
        let pick = |v: &Vec<usize>, ziel: f32, s: &str| v.iter().copied().min_by(|a, b| {
            let da = (self.world.clubs[*a].level - ziel).abs() + zufall(&[s, &self.world.clubs[*a].id.to_string(), &self.world.day.to_string()]);
            let db = (self.world.clubs[*b].level - ziel).abs() + zufall(&[s, &self.world.clubs[*b].id.to_string(), &self.world.day.to_string()]);
            da.partial_cmp(&db).unwrap()
        });
        let mut out = vec![];
        for (d, s) in [(-6.0, crate::id("a")), (0.0, crate::id("b")), (5.0, crate::id("c"))] { if let Some(k) = pick(&v, lv + d, s) { out.push(k); v.retain(|&x| x != k); } }
        out
    }
    /// Ask a club for a friendly on day d: a stronger club says no more often. Ok(text) or Err(text).
    pub fn testspiel_anfragen(&mut self, gegner: usize, d: u32) -> Result<String, String> {
        if !self.tag_frei(d, 1) { return Err(t("ui.vorb.tag_belegt").into()); }
        let diff = (self.world.clubs[gegner].level - self.club().level).max(0.0);
        let chance = f("testspiel.zusage") - f("testspiel.zusage_je_punkt") * diff;
        let n = self.world.clubs[gegner].name.clone();
        if zufall(&[crate::id("testspiel"), &self.world.clubs[gegner].id.to_string(), &d.to_string()]) >= chance { return Err(crate::tf!("ui.vorb.absage", n)); }
        self.termine.push(Termin { tag: d, art: 0, gegner, ..Default::default() });
        Ok(crate::tf!("ui.vorb.zusage", n, crate::date::short(self.world.season, d)))
    }
    /// Book a trip with its tournament from day d: the costs go out now, the travel load comes on the first day.
    pub fn reise_buchen(&mut self, id: &str, d: u32) -> Result<String, String> {
        let Some(tu) = turniere().iter().find(|x| x["id"].as_str() == Some(id)) else { return Err(t("ui.vorb.unbekannt").into()) };
        let tage = tu["tage"].as_u64().unwrap_or(5) as u32;
        if !self.tag_frei(d, tage) { return Err(t("ui.vorb.tag_belegt").into()); }
        let kosten = tu["kosten"].as_f64().unwrap_or(0.0);
        if self.club().balance < kosten { return Err(crate::tf!("ui.vorb.zu_teuer", money(kosten))); }
        self.book(t("ui.vorb.buchung"), -kosten);
        self.termine.push(Termin { tag: d, art: 1, turnier: id.to_string(), ..Default::default() });
        let tname = tu["name"].as_str().unwrap_or(id);
        Ok(crate::tf!("ui.vorb.gebucht", tname, crate::date::short(self.world.season, d)))
    }
    /// A friendly against `gegner`: the own eleven plays; it counts for nothing (no table, no scorers), it costs
    /// fitness, gives familiarity and a little morale to those who play. Returns (own goals, goals against).
    fn testspiel_spielen(&mut self, gegner: usize, heim: bool) -> (u8, u8) {
        let own = Some((self.own, self.formation, self.xi.clone(), self.tactic.style()));
        let (h, a) = if heim { (self.own, gegner) } else { (gegner, self.own) };
        let res = self.world.play(h, a, 0, own, None);
        let grund = t("ui.vorb.testspiel");
        // the half-time eleven (Konzept 2.3, 11 changes): the starters play the first half, the bench the second
        let mut zweite: Vec<usize> = self.squad().into_iter().filter(|p| !self.xi.contains(p) && self.world.players[*p].inj == 0).collect();
        zweite.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
        zweite.truncate(11);
        let mut einsatz: Vec<(usize, f32)> = res.minutes.iter().filter(|(p, _)| self.world.players[*p].team == self.own as i32).map(|&(p, m)| (p, (m as f32).min(45.0))).collect();
        einsatz.extend(zweite.into_iter().map(|p| (p, 45.0)));
        for (p, m) in einsatz {
            self.gf_dazu(p, f("grundfitness.testspiel_je_90") * m / 90.0);
            self.world.players[p].fit = (self.world.players[p].fit - f("testspiel.belastung") * m / 90.0).max(20.0);
            self.moral_wirkung(p, f("testspiel.moral_spielzeit"), 0, grund);
        }
        self.world.clubs[self.own].fam = (self.world.clubs[self.own].fam + f("testspiel.eingespielt")).min(100.0);
        if heim { (res.hg, res.ag) } else { (res.ag, res.hg) }
    }
    /// Daily: today's friendly is simulated (unless the person plays it in FL27), a trip starts with its travel load
    /// and its tournament is played on its days (semi-finals and a final, or one match with two teams).
    pub fn vorbereitung_tag(&mut self) {
        self.testspiel_anfragen_tag();
        // the second fitness test shortly before the first league match shows what the pre-season brought
        if self.world.day + 2 == crate::world::LEAGUE_START { self.leistungstest(); }
        let d = self.world.day;
        for i in 0..self.termine.len() {
            let tm = self.termine[i].clone();
            if tm.art == 2 { if tm.ergebnis.is_none() && d >= tm.tag && d < tm.tag + tm.tage { self.lager_tag(i); } continue; }
            if tm.tag != d || tm.ergebnis.is_some() { continue; }
            if tm.art == 0 {
                if tm.selbst { continue; } // the person plays it in FL27 and enters the score in the calendar
                let (g, ga) = self.testspiel_spielen(tm.gegner, true);
                let n = self.world.clubs[tm.gegner].name.clone();
                self.termine[i].ergebnis = Some((g, ga));
                self.termine[i].text = crate::tf!("ui.vorb.test_ergebnis", self.club().name.clone(), n, g, ga);
                let tx = self.termine[i].text.clone();
                self.news(&tx, t("ui.vorb.zaehlt_nicht"));
            } else {
                let Some(tu) = turniere().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).cloned() else { continue };
                // the journey: every player of the squad loses fitness; the team spirit of the trip
                for p in self.squad() { self.world.players[p].fit = (self.world.players[p].fit - tu["reise"].as_f64().unwrap_or(5.0) as f32).max(30.0); }
                self.event_summand_dazu(f("reise.teamgeist"));
                let teams = (tu["teams"].as_u64().unwrap_or(4) as usize).clamp(2, 4);
                let lv = self.club().level;
                let mut gegner: Vec<usize> = (0..self.world.clubs.len()).filter(|&k| k != self.own && self.world.erste_von(k).is_none() && (self.world.clubs[k].level - lv).abs() < 6.0).collect();
                gegner.sort_by(|a, b| zufall(&[&tm.turnier, &self.world.clubs[*a].id.to_string(), &d.to_string()]).partial_cmp(&zufall(&[&tm.turnier, &self.world.clubs[*b].id.to_string(), &d.to_string()])).unwrap());
                gegner.truncate(teams - 1);
                let (mut sieg, mut zeilen) = (true, vec![]);
                // two teams: one match; three: we play the two others in turn; four: semi-final and final (or match for third)
                let spiele: Vec<usize> = if teams == 4 { vec![gegner[0]] } else { gegner.clone() };
                let mut punkte = 0;
                for &g in &spiele {
                    let (a, b) = self.testspiel_spielen(g, false);
                    zeilen.push(format!("{} {a}:{b}", self.world.clubs[g].short));
                    if a < b { sieg = false; }
                    punkte += if a > b { 3 } else if a == b { 1 } else { 0 };
                }
                if teams == 4 {
                    let fin = if sieg { gegner[1] } else { gegner[2] };
                    let (a, b) = self.testspiel_spielen(fin, false);
                    zeilen.push(format!("{} {a}:{b}", self.world.clubs[fin].short));
                    sieg = sieg && a >= b;
                } else if teams == 3 { sieg = punkte >= 4; }
                self.world.clubs[self.own].fam = (self.world.clubs[self.own].fam + f("reise.eingespielt") * zeilen.len() as f32).min(100.0);
                let antritt = tu["antritt"].as_f64().unwrap_or(0.0);
                if antritt > 0.0 { self.book(t("ui.vorb.antritt"), antritt); }
                let name = tu["name"].as_str().unwrap_or("").to_string();
                self.termine[i].ergebnis = Some((sieg as u8, zeilen.len() as u8));
                self.termine[i].text = crate::tf!(if sieg { "ui.vorb.turnier_sieg" } else { "ui.vorb.turnier_ende" }, name, zeilen.join(", "));
                let tx = self.termine[i].text.clone();
                let region = tu["region"].as_str().unwrap_or("");
                self.msg(0, &name, &tx, &crate::tf!("ui.vorb.reise_text", region, money(antritt)), Action::None);
            }
        }
    }
    /// The breaks of the season in which other clubs ask for friendlies: (first day, last day, international break).
    fn pausen() -> Vec<(u32, u32, bool)> {
        let mut v = vec![(1, crate::world::LEAGUE_START - 1, false), (crate::world::WINTER_BREAK.0, crate::world::WINTER_BREAK.1, false)];
        v.extend(crate::world::INTL_BREAKS.iter().map(|&(a, b)| (a, b, true)));
        v
    }
    /// On the first day of a break: 3 to 8 clubs (fewer in an international break) ask for a friendly on a free day of
    /// it, each as a choice in the inbox with a deadline of 5 to 10 days; an unanswered one lapses.
    fn testspiel_anfragen_tag(&mut self) {
        let (d, abs) = (self.world.day, self.abs());
        let s = format!("{}:{d}", self.world.season);
        // lapsed requests
        for m in self.inbox.iter_mut() { if !m.done && m.frist > 0 && m.frist <= abs && matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("testanfrage:")) { m.done = true; m.read = true; } }
        let Some(&(_, bis, intl)) = Self::pausen().iter().find(|p| p.0 == d) else { return };
        let fa = if intl { f("anfragen.faktor_laenderspiel") } else { 1.0 };
        let n = ((f("anfragen.min") + (f("anfragen.max") - f("anfragen.min")) * zufall(&[crate::id("anfragen"), &s])) * fa).round() as usize;
        let lv = self.club().level;
        let mut kand: Vec<usize> = (0..self.world.clubs.len()).filter(|&k| k != self.own && self.world.erste_von(k).is_none() && self.world.clubs[k].level > lv - f("anfragen.staerke_unter") && self.world.clubs[k].level < lv + f("anfragen.staerke_ueber")).collect();
        kand.sort_by(|a, b| zufall(&[&s, &self.world.clubs[*a].id.to_string()]).partial_cmp(&zufall(&[&s, &self.world.clubs[*b].id.to_string()])).unwrap());
        let mut belegt: Vec<u32> = vec![];
        for g in kand.into_iter().take(n) {
            let tage: Vec<u32> = (d + 2..=bis).filter(|&x| self.tag_frei(x, 1) && !belegt.contains(&x)).collect();
            if tage.is_empty() { break; }
            let tag = tage[(zufall(&[&s, crate::id("tag"), &self.world.clubs[g].id.to_string()]) * tage.len() as f32) as usize % tage.len()];
            belegt.push(tag);
            let frist = (f("anfragen.frist_min") + (f("anfragen.frist_max") - f("anfragen.frist_min")) * zufall(&[&s, crate::id("frist"), &self.world.clubs[g].id.to_string()])).round() as u32;
            let frist = frist.min(tag.saturating_sub(d + 1).max(1));
            let n = self.world.clubs[g].name.clone();
            let opts = vec![(crate::id("zusagen").to_string(), t("ui.vorb.anfrage_zusagen").to_string()), (crate::id("absagen").to_string(), t("ui.vorb.anfrage_absagen").to_string())];
            let body = crate::tf!("ui.vorb.anfrage_text", n, crate::date::long(self.world.season, tag), crate::stufe_wort(self.world.clubs[g].level), frist);
            self.msg(1, &n, &crate::tf!("ui.vorb.anfrage_titel", n), &body, Action::Wahl { art: format!("testanfrage:{g}:{tag}"), bezug: vec![], optionen: opts });
            self.inbox[0].frist = abs + frist;
        }
    }
    /// The answer to a request: yes puts the friendly into the calendar when the day is still free.
    pub fn testanfrage_antwort(&mut self, sub: &str, key: &str) -> String {
        let mut it = sub.split(':').filter_map(|x| x.parse::<u32>().ok());
        let (Some(g), Some(tag)) = (it.next(), it.next()) else { return t("ui.vorb.unbekannt").into() };
        let n = self.world.clubs[g as usize].name.clone();
        if key != crate::id("zusagen") { return crate::tf!("ui.vorb.anfrage_abgesagt", n); }
        if !self.tag_frei(tag, 1) { return t("ui.vorb.tag_belegt").into(); }
        self.termine.push(Termin { tag, art: 0, gegner: g as usize, ..Default::default() });
        crate::tf!("ui.vorb.zusage", n, crate::date::short(self.world.season, tag))
    }
    /// The camp types that fit now: summer during the pre-season, winter in the winter break, short and closed camps
    /// always; the marketing trip in the pre-season for big clubs only.
    pub fn lager_passend(&self) -> Vec<&'static serde_json::Value> {
        let d = self.world.day;
        let sommer = d < crate::world::LEAGUE_START + 7;
        let winter = d + 14 >= crate::world::WINTER_BREAK.0 && d <= crate::world::WINTER_BREAK.1;
        let p = self.prestige() as f32;
        lager_orte().iter().filter(|o| match o["art"].as_str().unwrap_or("") { "sommer" => sommer, "winter" => winter, "marketing" => sommer && p >= zahl(o, "prestige_ab"), _ => true }).collect()
    }
    /// The estimate of a camp: people × days × cost per day + travel (Konzept 4.5).
    pub fn lager_kosten(&self, id: &str, tage: u32) -> f64 {
        let Some(o) = ort(id) else { return 0.0 };
        let personen = self.squad().len() as f64 + lg(crate::id("stab")) as f64;
        personen * tage as f64 * zahl(o, "kosten_tag") as f64 + zahl(o, "reise_kosten") as f64
    }
    /// Book a camp from day d for `tage` days: the board with little confidence allows cheap ones only (Konzept 4.5).
    pub fn lager_buchen(&mut self, id: &str, d: u32, tage: u32) -> Result<String, String> {
        let Some(o) = ort(id) else { return Err(t("ui.vorb.unbekannt").into()) };
        if !self.tag_frei(d, tage) { return Err(t("ui.vorb.tag_belegt").into()); }
        let kosten = self.lager_kosten(id, tage);
        if self.mensch.vorstand < lg("vorstand_knapp") && kosten > lg("grenze_knapp") as f64 { return Err(crate::tf!("ui.lager.vorstand_nein", money(kosten))); }
        if self.club().balance < kosten { return Err(crate::tf!("ui.vorb.zu_teuer", money(kosten))); }
        if kosten > 0.0 { self.book(t("ui.lager.buchung"), -kosten); }
        self.termine.push(Termin { tag: d, art: 2, turnier: id.to_string(), tage, ..Default::default() });
        let name = o["name"].as_str().unwrap_or(id);
        Ok(crate::tf!("ui.lager.gebucht", name, crate::date::short(self.world.season, d), tage, money(kosten)))
    }
    /// A day of a camp: the start (team spirit „Lager“, room question), the training days (familiarity by pitches and
    /// quiet, height, jetlag after a long flight), a media day in the middle, one curfew story at most, and the report
    /// with its effects on the last day. The marketing trip pays and brings fans, but trains less.
    fn lager_tag(&mut self, i: usize) {
        let tm = self.termine[i].clone();
        let Some(o) = ort(&tm.turnier) else { return };
        let d = self.world.day;
        let k = d - tm.tag;
        let name = o["name"].as_str().unwrap_or("").to_string();
        let reise = zahl(o, crate::id("reisetage")) as u32;
        if k == 0 {
            let tg = (lg("teamgeist_basis") + lg("teamgeist_je_tag") * tm.tage as f32).min(lg("teamgeist_max"));
            self.event_summand_dazu(tg);
            self.termine[i].bericht.push(crate::tf!("ui.lager.b_teamgeist", format!("{tg:+.1}")));
            let opts = vec![(crate::id("kapitaen").to_string(), t("ui.lager.zimmer_kapitaen").to_string()), (crate::id("nationen").to_string(), t("ui.lager.zimmer_nationen").to_string()), (crate::id("einzel").to_string(), t("ui.lager.zimmer_einzel").to_string()), (crate::id("los").to_string(), t("ui.lager.zimmer_los").to_string())];
            let co = self.co_name.clone();
            self.msg(1, &co, &crate::tf!("ui.lager.zimmer_titel", name), t("ui.lager.zimmer_text"), Action::Wahl { art: crate::id("lager:zimmer").into(), bezug: vec![], optionen: opts });
        }
        let marketing = o["art"].as_str() == Some(crate::id("marketing"));
        if k >= reise && k < tm.tage.saturating_sub(reise.min(1)) {
            // a training day
            let mut f = 1.0 + 0.05 * (zahl(o, crate::id("plaetze")) - 3.0) + 0.05 * (zahl(o, crate::id("ruhe")) - 3.0);
            if o["klima"].as_str() == Some(crate::id("hoehe")) { f *= lg("hoehe_faktor"); }
            if marketing { f *= lg("marketing_eingespielt"); }
            let plus = lg("eingespielt_tag") * f;
            self.gf_an();
            for p in self.squad() { self.gf_dazu(p, crate::regeln::f("vorbereitung", "grundfitness.lager_je_tag") * f); }
            self.world.clubs[self.own].fam = (self.world.clubs[self.own].fam + plus).min(100.0);
            if zahl(o, crate::id("zeit")) >= lg("jetlag_ab") && k < reise + 2 {
                for p in self.squad() { self.world.players[p].fit = (self.world.players[p].fit - lg("jetlag_kondition")).max(40.0); }
            }
        }
        if k == tm.tage / 2 && tm.tage >= 5 {
            let opts = vec![(crate::id("offen").to_string(), t("ui.lager.medien_offen").to_string()), (crate::id("knapp").to_string(), t("ui.lager.medien_knapp").to_string())];
            let co = self.co_name.clone();
            self.msg(1, &co, &crate::tf!("ui.lager.medien_titel", name), t("ui.lager.medien_text"), Action::Wahl { art: crate::id("lager:medien").into(), bezug: vec![], optionen: opts });
        }
        // one curfew story at most: the least professional player, now and then
        if k > reise && !tm.bericht.iter().any(|b| b.starts_with('!')) {
            let s = format!("{}:{}", self.world.season, tm.tag);
            if let Some(p) = self.squad().into_iter().filter(|&p| cw(&self.world.players[p], PROF) <= 40.0).min_by(|a, b| cw(&self.world.players[*a], PROF).partial_cmp(&cw(&self.world.players[*b], PROF)).unwrap()) {
                if zufall(&[crate::id("zapfenstreich"), &s, &k.to_string()]) < 0.18 {
                    let n = self.world.players[p].name.clone();
                    let opts = self.optionen_fuer(crate::id("disziplin"), &format!("{}:{d}", self.world.players[p].id));
                    let co = self.co_name.clone();
                    self.msg(1, &co, &crate::tf!("ui.lager.zapfen_titel", n), &crate::tf!("ui.lager.zapfen_text", n), Action::Wahl { art: crate::id("disziplin:zapfenstreich").into(), bezug: vec![p], optionen: opts });
                    self.termine[i].bericht.push(format!("!{}", crate::tf!("ui.lager.b_zapfen", n)));
                }
            }
        }
        if k + 1 == tm.tage {
            // the end: marketing money and fans, the report
            if marketing {
                let m = zahl(o, crate::id("marketing")) as f64;
                let geld = lg("marketing_antritt") as f64 * m * self.skala();
                self.book(t("ui.lager.antritt"), geld);
                self.klima(lg("marketing_fans") * m as f32, lg("marketing_fans") * m as f32, 0.0, &name);
                self.termine[i].bericht.push(crate::tf!("ui.lager.b_marketing", money(geld)));
            }
            let zeilen: Vec<String> = self.termine[i].bericht.iter().map(|b| b.trim_start_matches('!').to_string()).collect();
            let text = crate::tf!("ui.lager.bericht_text", name, tm.tage, format!("{:.0}", self.club().fam), zeilen.join(crate::id("\n")));
            self.termine[i].ergebnis = Some((1, tm.tage as u8));
            self.termine[i].text = crate::tf!("ui.lager.vorbei", name);
            let co = self.co_name.clone();
            self.msg(1, &co, &crate::tf!("ui.lager.bericht_titel", name), &text, Action::None);
        }
    }
    /// The answer to a camp question (rooms, media day).
    pub fn lager_antwort(&mut self, sub: &str, key: &str) -> String {
        let sq = self.squad();
        match (sub, key) {
            ("zimmer", "kapitaen") => { if let Some(k) = self.kapitaen() { for &p in &sq { if p != k && self.world.players[p].joined + 60 > self.abs() { self.beziehung(p, 10.0, t("ui.lager.zimmer_grund")); } } } self.event_summand_dazu(0.5); t("ui.lager.a_kapitaen").into() }
            ("zimmer", "einzel") => { let stars: Vec<usize> = sq.iter().copied().filter(|&p| cw(&self.world.players[p], crate::mensch::EGO) >= 65.0).collect(); for p in stars { self.moral_wirkung(p, 2.0, 0, t("ui.lager.zimmer_grund")); } self.event_summand_dazu(-1.0); t("ui.lager.a_einzel").into() }
            ("zimmer", "los") => { let g = if zufall(&[crate::id("los"), &self.abs().to_string()]) < 0.5 { 1.0 } else { -0.5 }; self.event_summand_dazu(g); t(if g > 0.0 { "ui.lager.a_los_gut" } else { "ui.lager.a_los_schlecht" }).into() }
            ("zimmer", _) => t("ui.lager.a_nationen").into(),
            ("medien", "offen") => { self.klima(0.0, 2.0, 0.0, t("ui.lager.medien_grund")); t("ui.lager.a_offen").into() }
            _ => t("ui.lager.a_knapp").into(),
        }
    }
    /// Base fitness on for the own squad: from age and professionalism (Konzept 3.3), once per player.
    pub fn gf_an(&mut self) {
        let g = |k: &str| f(&format!("grundfitness.{k}"));
        for p in self.world.clubs[self.own].squad.clone() {
            let pl = &self.world.players[p];
            if pl.gf > 0.0 { continue; }
            let v = g(crate::id("start")) + g("start_je_prof") * (cw(pl, PROF) - 50.0) - g("start_je_jahr") * (pl.age as f32 - g("start_alter_ab")).max(0.0);
            self.world.players[p].gf = v.clamp(40.0, 95.0);
        }
    }
    /// Base fitness moves by `d` (above 85 a gain counts half).
    pub fn gf_dazu(&mut self, p: usize, d: f32) {
        let pl = &mut self.world.players[p];
        if pl.gf <= 0.0 { return; }
        let d = if d > 0.0 && pl.gf > 85.0 { d * 0.5 } else { d };
        pl.gf = (pl.gf + d).clamp(20.0, 100.0);
    }
    /// The training day's effect on base fitness: endurance work builds it, a free or recovery day in the season lets it slip.
    pub fn gf_training(&mut self, einheit: usize) {
        self.gf_an();
        let g = |k: &str| f(&format!("grundfitness.{k}"));
        let basis = crate::mensch::arr("vorbereitung", "grundfitness.basis").get(einheit).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
        let saison = self.world.day >= crate::world::LEAGUE_START;
        for p in self.squad() {
            if self.world.players[p].inj > 0 { continue; }
            if basis > 0.0 { self.gf_dazu(p, g("je_basispunkt") * basis); } else if saison { self.gf_dazu(p, -g("abbau_tag")); }
        }
    }
    /// The highest condition base fitness allows (Konzept 3.3), None before base fitness started.
    pub fn kondition_max(&self, p: usize) -> Option<f32> {
        let gf = self.world.players[p].gf;
        (gf > 0.0).then(|| f("grundfitness.kondition_basis") + f("grundfitness.kondition_je_gf") * gf)
    }
    /// The fitness test (Konzept 3.4) on the first pre-season day and before the first league match: base fitness per
    /// player (exact with a good medical and fitness staff, else in words) and one line of the assistant coach.
    pub fn leistungstest(&mut self) {
        self.gf_an();
        let g = |k: &str| f(&format!("grundfitness.{k}"));
        let genau = self.medizinqualitaet() >= g("test_genau_ab");
        let mut sq = self.squad();
        sq.sort_by(|a, b| self.world.players[*b].gf.partial_cmp(&self.world.players[*a].gf).unwrap());
        let wort = |v: f32| t(if v >= g(crate::id("topfit")) { "ui.gf.topfit" } else if v >= g(crate::id("soll")) { "ui.gf.gut" } else if v >= 55.0 { "ui.gf.mittel" } else { "ui.gf.schwach" });
        let zeilen: Vec<String> = sq.iter().map(|&p| { let pl = &self.world.players[p]; format!("{}: {}", pl.name, if genau { format!("{:.0}", pl.gf) } else { wort(pl.gf).to_string() }) }).collect();
        let unter = sq.iter().filter(|&&p| self.world.players[p].gf < g(crate::id("soll"))).count();
        let bester = sq.first().map(|&p| self.world.players[p].name.clone()).unwrap_or_default();
        let co_zeile = if unter == 0 { crate::tf!("ui.gf.co_alle", bester) } else { crate::tf!("ui.gf.co_unter", unter, bester) };
        let co = self.co_name.clone();
        self.msg(1, &co, t("ui.gf.titel"), &format!("{co_zeile}\n{}", zeilen.join(crate::id("\n"))), Action::None);
        self.meldung_art(crate::pages::Art::Allgemein, sq.iter().take(3).copied().collect());
    }
    /// The score of a friendly the person played in FL27.
    pub fn testspiel_eintragen(&mut self, i: usize, g: u8, ga: u8) {
        let Some(tm) = self.termine.get(i).cloned() else { return };
        if tm.ergebnis.is_some() { return; }
        for p in self.squad().into_iter().filter(|p| self.xi.contains(p)) { self.world.players[p].fit = (self.world.players[p].fit - f("testspiel.belastung")).max(20.0); }
        self.world.clubs[self.own].fam = (self.world.clubs[self.own].fam + f("testspiel.eingespielt")).min(100.0);
        let n = self.world.clubs[tm.gegner].name.clone();
        self.termine[i].ergebnis = Some((g, ga));
        self.termine[i].text = crate::tf!("ui.vorb.test_ergebnis", self.club().name.clone(), n, g, ga);
    }
    /// The team events' share of the team spirit: their points fade with a half-life of 30 days, at most ±8.
    pub fn event_summand(&self) -> f32 {
        let abs = self.abs() as f32;
        let hw = f("teamgeist.halbwertszeit");
        self.event_punkte.iter().map(|(d, p)| p * 0.5f32.powf((abs - *d as f32) / hw)).sum::<f32>().clamp(-f("teamgeist.deckel"), f("teamgeist.deckel"))
    }
    fn event_summand_dazu(&mut self, p: f32) {
        let abs = self.abs();
        self.event_punkte.retain(|x| abs < x.0 + 180);
        self.event_punkte.push((abs, p));
    }
    /// Hold a team event: costs, team spirit; at the team night and the season trip now and then a moderate scandal.
    pub fn teamevent(&mut self, id: &str) -> Result<String, String> {
        let Some(e) = events().iter().find(|x| x["id"].as_str() == Some(id)).cloned() else { return Err(t("ui.vorb.unbekannt").into()) };
        let abs = self.abs();
        if let Some(seit) = self.teamevents.iter().find(|x| x.0 == id).map(|x| x.1) {
            let frei = seit + e["sperre"].as_u64().unwrap_or(30) as u32;
            if abs < frei { return Err(crate::tf!("ui.vorb.event_gesperrt", frei - abs)); }
        }
        self.teamevents.retain(|x| x.0 != id);
        self.teamevents.push((id.to_string(), abs));
        self.book(t("ui.vorb.buchung_event"), -e["kosten"].as_f64().unwrap_or(0.0));
        let mut tg = e["teamgeist"].as_f64().unwrap_or(0.0) as f32;
        let name = t(&format!("ui.vorb.event.{id}"));
        if zufall(&[crate::id("skandal"), id, &abs.to_string()]) < e["skandal"].as_f64().unwrap_or(0.0) as f32 {
            // the one who overdoes it: little professionalism, a hot temper
            let wer = self.squad().into_iter().max_by(|a, b| {
                let s = |p: usize| cw(&self.world.players[p], crate::mensch::TEMPERAMENT) - cw(&self.world.players[p], PROF) + 20.0 * zufall(&[crate::id("skandal_wer"), &self.world.players[p].id.to_string(), &abs.to_string()]);
                s(*a).partial_cmp(&s(*b)).unwrap()
            });
            tg *= 0.5;
            self.event_summand_dazu(tg);
            if let Some(p) = wer {
                let n = self.world.players[p].name.clone();
                let g = crate::tf!("ui.vorb.skandal_grund", name);
                self.moral_wirkung(p, f("skandal.moral"), 0, &g);
                self.klima(f("skandal.fans"), f("skandal.medien"), f("skandal.vorstand"), &g);
                self.news(&crate::tf!("ui.vorb.skandal_titel", n, name), &crate::tf!("ui.vorb.skandal_text", n));
                return Ok(crate::tf!("ui.vorb.skandal_ok", name, n));
            }
        }
        self.event_summand_dazu(tg);
        Ok(crate::tf!("ui.vorb.event_ok", name))
    }
    /// After the summer holiday: each player comes back fit by his professionalism; a careless one now and then with
    /// overweight (more fitness lost, a little morale).
    pub fn urlaub_rueckkehr(&mut self) {
        self.gf_an();
        let s = self.world.season.to_string();
        for p in self.squad() {
            let prof = cw(&self.world.players[p], PROF) / 100.0;
            let mut fit = 100.0 - f("urlaub.tage") * f("urlaub.verlust_je_tag") * (1.3 - prof);
            self.gf_dazu(p, -f("urlaub.tage") * f("urlaub.verlust_je_tag") * (1.3 - prof));
            let id = self.world.players[p].id.to_string();
            if zufall(&[crate::id("uebergewicht"), &id, &s]) < f("urlaub.basis") + (1.0 - prof) * f("urlaub.je_unprofi") {
                fit -= f("urlaub.extra_verlust");
                self.gf_dazu(p, -f("grundfitness.uebergewicht"));
                let n = self.world.players[p].name.clone();
                self.moral_wirkung(p, f("urlaub.moral"), 0, t("ui.vorb.uebergewicht_grund"));
                let co = self.co_name.clone();
                self.msg(1, &co, &crate::tf!("ui.vorb.uebergewicht_titel", n), &crate::tf!("ui.vorb.uebergewicht_text", n), Action::None);
            }
            self.world.players[p].fit = fit.clamp(40.0, 100.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn karriere() -> Career {
        let w = crate::world::World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(0); // roh
        Career::new(w, own, "Test") // roh
    }
    #[test]
    fn trainingslager_kostet_wirkt_und_berichtet() {
        let mut c = karriere();
        assert!(c.lager_passend().iter().any(|o| o["art"] == "sommer"), "im Sommer gibt es Sommerlager");
        let d = c.freie_tage(1, 7)[0];
        let k0 = c.club().balance;
        let kosten = c.lager_kosten("alpen", 7);
        c.lager_buchen("alpen", d, 7).unwrap();
        assert!((k0 - c.club().balance - kosten).abs() < 1.0 && kosten > 0.0);
        assert!(c.lager_buchen("sportschule", d + 2, 3).is_err(), "Tage sind belegt");
        let (fam0, tg0) = (c.club().fam, c.event_summand());
        for x in d..d + 7 { c.world.day = x; c.vorbereitung_tag(); }
        assert!(c.club().fam > fam0, "Eingespieltheit steigt");
        assert!(c.event_summand() > tg0, "Teamgeist „Lager“");
        assert!(c.termine.iter().any(|t| t.art == 2 && t.ergebnis.is_some()));
        assert!(c.inbox.iter().any(|m| m.title.contains("Alpen") && m.action == Action::None), "Lagerbericht");
        assert!(c.inbox.iter().any(|m| matches!(&m.action, Action::Wahl { art, .. } if art == "lager:zimmer")));
        // the board with little confidence stops an expensive camp
        c.mensch.vorstand = 20.0;
        let d2 = c.freie_tage(1, 10)[0];
        assert!(c.lager_buchen("alpen", d2, 10).is_err());
        assert!(c.lager_antwort("zimmer", "los").len() > 5);
    }

    #[test]
    fn grundfitness_urlaub_aufbau_deckel_test() {
        let mut c = karriere();
        c.gf_an();
        let p = c.squad()[0];
        let g0 = c.world.players[p].gf;
        assert!((40.0..=95.0).contains(&g0));
        c.urlaub_rueckkehr();
        let g1 = c.world.players[p].gf;
        assert!(g1 < g0, "Urlaub senkt die Grundfitness {g0} → {g1}");
        // intensive units build it up, the condition stays under its cap
        for _ in 0..20 { c.gf_training(5); }
        assert!(c.world.players[p].gf > g1);
        c.world.players[p].gf = 50.0;
        c.world.players[p].fit = 95.0;
        c.world.recover_all();
        assert!(c.world.players[p].fit <= 95.0);
        c.world.players[p].fit = 60.0;
        for _ in 0..30 { c.world.recover_all(); }
        assert!(c.world.players[p].fit <= c.kondition_max(p).unwrap() + 0.01, "Deckel 60 + 0,4 · 50 = 80");
        c.inbox.clear();
        c.leistungstest();
        assert!(c.inbox.iter().any(|m| m.title == crate::inhalt::t("ui.gf.titel")));
    }

    #[test]
    fn anfragen_in_der_winterpause() {
        let mut c = karriere();
        c.world.day = crate::world::WINTER_BREAK.0;
        c.vorbereitung_tag();
        let anfr: Vec<(String, u32)> = c.inbox.iter().filter_map(|m| match &m.action { Action::Wahl { art, .. } if art.starts_with("testanfrage:") => Some((art.clone(), m.frist)), _ => None }).collect();
        assert!((3..=8).contains(&anfr.len()), "{} Anfragen", anfr.len());
        assert!(anfr.iter().all(|a| a.1 > c.abs()), "mit Frist");
        // yes puts it into the calendar, inside the break
        let r = c.wahl(&anfr[0].0, &[], "zusagen");
        assert_eq!(c.termine.len(), 1, "{r}");
        assert!(c.termine[0].tag <= crate::world::WINTER_BREAK.1);
        // unanswered ones lapse after their deadline
        c.world.day += 11;
        c.vorbereitung_tag();
        assert!(c.inbox.iter().filter(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("testanfrage:"))).all(|m| m.done || m.frist > c.abs()));
    }

    #[test]
    fn testspiel_reise_event_urlaub() {
        let mut c = karriere();
        // a friendly counts for nothing in the table, costs fitness, gives familiarity
        let d = c.freie_tage(1, 1)[0];
        let g = c.testspiel_gegner();
        assert_eq!(g.len(), 3);
        let mut ok = false;
        for &x in &g { if c.testspiel_anfragen(x, d).is_ok() { ok = true; break; } }
        assert!(ok, "someone says yes");
        let tabelle: Vec<i32> = c.world.table(c.league()).iter().map(|r| r.pts).collect();
        let fam0 = c.club().fam;
        while c.world.day < d { c.ultimatum = None; c.end_day().unwrap(); if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } }
        c.vorbereitung_tag();
        assert!(c.termine[0].ergebnis.is_some(), "simulated by default");
        assert!(c.club().fam > fam0);
        assert_eq!(tabelle, c.world.table(c.league()).iter().map(|r| r.pts).collect::<Vec<_>>(), "no table points");
        // a trip with a tournament of four: costs, travel, two matches
        let k0 = c.club().balance;
        let d2 = c.freie_tage(1, 10)[0];
        c.reise_buchen("usa_tour", d2).unwrap();
        assert!(c.club().balance < k0);
        c.world.day = d2;
        c.vorbereitung_tag();
        assert!(c.termine[1].ergebnis.map_or(false, |e| e.1 == 2), "semi-final and final: {:?}", c.termine[1].text);
        // team events: team spirit summand, the same event not twice in its lock time; scandals rare
        let s0 = c.event_summand();
        c.teamevent("grillabend").unwrap();
        assert!(c.event_summand() > s0);
        assert!(c.teamevent("grillabend").is_err());
        let skandale = (0..400).filter(|i| { c.teamevents.clear(); c.world.day = (i % 300) as u32; c.world.season += (i / 300) as u32; c.teamevent("abschlussfahrt").map_or(false, |t| t.contains('!')) }).count();
        assert!((10..=80).contains(&skandale), "{skandale} scandals in 400 trips");
        // holiday: the professional comes back fitter; overweight happens to careless players more often
        c.urlaub_rueckkehr();
        let sq = c.squad();
        let profi = *sq.iter().max_by(|a, b| cw(&c.world.players[**a], PROF).partial_cmp(&cw(&c.world.players[**b], PROF)).unwrap()).unwrap();
        assert!(c.world.players[profi].fit >= 85.0, "{}", c.world.players[profi].fit);
    }
}
