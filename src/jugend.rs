//! Akademie-Konzept (konzepte/akademie.md, 23.11.2026): youth strength 1–10 for every club, the yearly Talenttag, graduates
//! of the AI clubs (real players first), mentors only for the young with the co-trainer's proposals, training guests with
//! the first team, the closing round for those turning 19, the academy factors of the development and the monthly report.
//! Numbers in assets/regeln/jugendakademie.toml.
use crate::game::*;
use crate::mensch::{cw, zufall, EGO, FUEHRUNG, PROF, TEMPERAMENT};
use crate::regeln::{f, wert};
use crate::world::*;

fn k(p: &str) -> f32 { f("jugendakademie", p) }
fn money_txt(v: f64) -> String { crate::world::money(v) }
pub(crate) fn reihe(p: &str) -> Vec<f32> { wert("jugendakademie", p).as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect()).unwrap_or_default() }
/// A normal draw from two keyed uniform draws (Box-Muller), so a year group stays the same on reload.
fn normal(schluessel: &str, m: f32, sd: f32) -> f32 {
    let u1 = zufall(&[schluessel, "n1"]).max(1e-6); // roh
    let u2 = zufall(&[schluessel, "n2"]); // roh
    m + sd * (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
}

/// The word for an estimated potential (Konzept 3.4).
pub fn stufe_wort(pot: f32) -> &'static str {
    let i = [88.0, 82.0, 76.0, 70.0].iter().position(|&g| pot >= g).unwrap_or(4);
    crate::inhalt::t(&format!("ui.jugend.stufe.{i}"))
}

/// The parts of a club's youth strength (all 1–10) and the value itself.
#[derive(Clone, Copy, Debug, Default)]
pub struct Jugend { pub anlagen: f32, pub stab: f32, pub region: f32, pub tradition: f32, pub wert: f32, pub bekannt: Option<f32> }

impl Career {
    /// Youth strength (Konzept 1.2): 0,35·Anlagen + 0,25·Stab + 0,20·Region + 0,20·Tradition; a known academy of [bekannt] sets the start.
    pub fn jugendstaerke(&self, club: usize) -> Jugend {
        let cl = &self.world.clubs[club];
        let rep = cl.rep;
        let lin = |p: &str| { let r = reihe(p); r[0] + r[1] * rep };
        let own = club == self.own;
        let anlagen = if own { 2.0 * self.wi.stufen[3].max(1) as f32 } else { lin("staerke.ki_anlagen") }.clamp(1.0, 10.0);
        let stab = if own { self.trainingsqualitaet() / 10.0 } else { lin("staerke.ki_stab") }.clamp(1.0, 10.0);
        let land = &self.world.leagues[cl.league].land;
        let region = wert("jugendakademie", "region").get(land.as_str()).and_then(|x| x.as_f64()).map(|x| x as f32).unwrap_or(k("staerke.region_standard"));
        let eigene = cl.squad.iter().filter(|&&p| self.world.eigen.get(&self.world.players[p].id) == Some(&cl.id)).count() as f32;
        let tradition = (lin("staerke.tradition_basis") + k("staerke.tradition_je_eigengewaechs") * eigene).clamp(1.0, 10.0);
        let formel = k("staerke.anlagen") * anlagen + k("staerke.stab") * stab + k("staerke.region") * region + k("staerke.tradition") * tradition;
        let bekannt = wert("jugendakademie", "bekannt").as_object().and_then(|o| o.iter().find(|(n, _)| cl.name.contains(n.as_str()))).and_then(|x| x.1.as_f64()).map(|x| x as f32);
        let g = k("staerke.bekannt_gewicht");
        let wert = bekannt.map_or(formel, |b| g * b + (1.0 - g) * formel).clamp(1.0, 10.0);
        Jugend { anlagen, stab, region, tradition, wert, bekannt }
    }
    /// The youth reputation in stars 1–5.
    pub fn jugend_ruf(&self) -> u8 { ((self.jugendstaerke(self.own).wert / 2.0).round() as u8).clamp(1, 5) }

    /// A new player of a club built from the template of the position (talent day, graduates).
    fn nachwuchs_anlegen(&mut self, club: usize, name: String, pos: &str, alter: u8, ovr: f32, pot: f32, nat: u16, youth: u8) -> usize {
        let (attrs, pp) = self.world.template_attrs(pos, ovr);
        let mut p = self.world.players[0].clone();
        p.id = self.world.new_id();
        p.attrs = attrs; p.pp = pp; p.db = false; p.skills = 0; p.style = 0; p.name = name; p.pos = pos.into(); p.age = alter; p.ovr = ovr; p.pot = pot.max(ovr + 1.0); p.talent = 1.0;
        p.nat = nat; p.team = club as i32; p.youth = youth; p.no = 0; p.know = if club == self.own { 1.0 } else { 0.1 };
        p.inj = 0; p.inj_name.clear(); p.susp = 0; p.form = 0.0; p.fit = 100.0; p.morale = 60.0; p.last.clear(); p.listed = false; p.program = 0;
        p.apps = 0; p.goals = 0; p.assists = 0; p.rsum = 0.0; p.rn = 0; p.yellow = 0; p.leihe = None; p.ps = Default::default(); p.ent = Default::default(); p.profil = Default::default();
        p.unhappy_days = 0; p.benched = 0; p.promise = 0; p.joined = 0; p.talk_day = 0; p.value = 0.0;
        p.contract_end = self.world.season + if youth > 0 { k("jahrgang.jahre") as u32 } else { 3 };
        p.wage = if youth > 0 { k("jahrgang.gehalt") as f64 } else { fair_wage(&p, self.world.leagues[self.world.clubs[club].league].level) };
        p.persona = (zufall(&["persona", &p.id.to_string()]) * 5.0) as u8 % 5; // roh
        let i = self.world.players.len();
        self.world.players.push(p);
        self.world.clubs[club].squad.push(i);
        i
    }
    /// The country of a new young player of a club (Fabian 24.11.: a small club almost only home-grown, a big one now and
    /// then someone from abroad): home, or by a chance growing with the reputation a country weighted by how often it plays in
    /// the club's league (only countries with enough real names).
    pub(crate) fn jugend_nat(&self, club: usize, schluessel: &str) -> u16 {
        let l = self.world.clubs[club].league;
        let heim = self.world.land_nat(l);
        let rep = (self.world.clubs[club].rep / 100.0).clamp(0.0, 1.0);
        let h = reihe("herkunft.ausland");
        if zufall(&[schluessel, crate::id("herkunft")]) >= h[0] + h[1] * rep * rep { return heim; }
        let mut alle: std::collections::HashMap<u16, usize> = Default::default();
        for p in &self.world.players { *alle.entry(p.nat).or_default() += 1; }
        let mut liga: std::collections::BTreeMap<u16, usize> = Default::default();
        for &t in &self.world.leagues[l].teams { for &p in &self.world.clubs[t].squad { let n = self.world.players[p].nat; if n > 0 && n != heim && alle[&n] >= k("herkunft.namen_mindestens") as usize { *liga.entry(n).or_default() += 1; } } }
        let summe: usize = liga.values().sum();
        if summe == 0 { return heim; }
        let mut x = zufall(&[schluessel, crate::id("land")]) * summe as f32;
        liga.iter().find(|(_, &z)| { x -= z as f32; x < 0.0 }).map_or(heim, |(&n, _)| n)
    }
    /// First and last names of real players of a nation (for new players of that country); empty without any.
    /// First and last names to combine (Fabian 23.11.: realistic, a large pool): every player of the nationality, else
    /// every player of the own league's clubs, else a built-in list. Duplicates removed.
    pub(crate) fn namen_von(&self, nat: u16) -> (Vec<String>, Vec<String>) {
        let teilen = |it: &mut dyn Iterator<Item = &crate::world::Player>| -> (Vec<String>, Vec<String>) {
            let (mut a, mut b): (Vec<String>, Vec<String>) = (vec![], vec![]);
            for p in it {
                let mut t: Vec<&str> = p.name.split_whitespace().collect();
                // "Chase Jr" is no surname: drop suffixes and initials at the end
                while t.len() > 2 && t.last().map_or(false, |x| [crate::id("Jr"), crate::id("Jr."), crate::id("Junior"), crate::id("Sr"), crate::id("Sr."), crate::id("II"), crate::id("III")].contains(x) || x.chars().count() <= 2) { t.pop(); }
                if t.len() >= 2 && t[t.len() - 1].chars().count() > 2 && ![crate::id("Jr"), crate::id("Jr."), crate::id("Junior"), crate::id("Sr"), crate::id("Sr."), crate::id("II"), crate::id("III")].contains(&t[t.len() - 1]) && t[0].chars().count() > 2 && !t[0].ends_with('.') { a.push(t[0].to_string()); b.push(t[t.len() - 1].to_string()); }
            }
            a.sort(); a.dedup(); b.sort(); b.dedup();
            (a, b)
        };
        let v = teilen(&mut self.world.players.iter().filter(|p| nat > 0 && p.nat == nat));
        if v.0.len() >= 25 && v.1.len() >= 25 { return v; }
        let liga: std::collections::HashSet<usize> = self.world.leagues[self.league()].teams.iter().copied().collect();
        teilen(&mut self.world.players.iter().filter(|p| p.team >= 0 && liga.contains(&(p.team as usize))))
    }
    /// A name nobody in the world has yet (up to 40 tries with a fresh draw), remembered so it does not come again.
    pub(crate) fn name_aus(&self, namen: &(Vec<String>, Vec<String>), schluessel: &str) -> String {
        thread_local! { static VERGEBEN: std::cell::RefCell<(usize, std::collections::HashSet<String>)> = Default::default(); }
        let (a, b) = namen;
        let vor = ["Luca", "Noah", "Jonas", "Leon", "Elias", "Finn", "Milan", "Emil", "Mats", "Malik", "Theo", "Arda", "Ben", "Paul", "Felix", "Jakob", "Lukas", "Max", "Julian", "Tim", "Niklas", "Moritz", "David", "Samuel", "Tom", "Jan", "Nils", "Leo", "Henri", "Mika"]; // roh
        let nach = ["Becker", "Kramer", "Hoffmann", "Keller", "Brandt", "Vogel", "Novak", "Lindner", "Arslan", "Roth", "Seidel", "Kuhn", "Schulz", "Wagner", "Bauer", "Richter", "Klein", "Wolf", "Neumann", "Schwarz", "Zimmermann", "Braun", "Krüger", "Hartmann", "Lange", "Werner", "Krause", "Lehmann", "Köhler", "Maier"]; // roh
        VERGEBEN.with(|v| {
            let mut v = v.borrow_mut();
            // only the players added since the last look (a new career starts from scratch)
            let n = self.world.players.len();
            if v.0 > n { *v = Default::default(); }
            let neu: Vec<String> = self.world.players[v.0..].iter().map(|p| p.name.clone()).collect();
            v.1.extend(neu);
            v.0 = n;
            let mut name = String::new();
            for k in 0..40 {
                let salz = format!("{schluessel}:{k}");
                name = if a.is_empty() || b.is_empty() {
                    format!("{} {}", vor[(zufall(&[&salz, "v"]) * vor.len() as f32) as usize % vor.len()], nach[(zufall(&[&salz, "n"]) * nach.len() as f32) as usize % nach.len()]) // roh
                } else {
                    format!("{} {}", a[(zufall(&[&salz, "v"]) * a.len() as f32) as usize % a.len()], b[(zufall(&[&salz, "n"]) * b.len() as f32) as usize % b.len()]) // roh
                };
                if !v.1.contains(&name) { break; }
            }
            v.1.insert(name.clone());
            name
        })
    }

    /// The Talenttag (Konzept 3.1): the new U17 year group in July, size and quality from the youth strength, now and then a
    /// jewel; one message with every card, a jewel gets its own from the academy head.
    pub fn talenttag(&mut self) -> Vec<usize> {
        let js = self.jugendstaerke(self.own).wert;
        let stufe = self.wi.stufen[3].max(1) as f32;
        let s = format!("talenttag:{}:{}", self.club().id, self.world.season);
        let zr = reihe("jahrgang.zufall");
        let mut n = (k("jahrgang.groesse_basis") + stufe + zr[0] + (zufall(&[&s, "groesse"]) * (zr[1] - zr[0] + 1.0)).floor()).max(0.0) as usize; // roh
        let u17 = self.world.clubs[self.own].squad.iter().filter(|&&p| self.world.players[p].youth == 17).count();
        n = n.min(self.akademie_plaetze().0.saturating_sub(u17));
        if n == 0 { return vec![]; }
        let mut namen: std::collections::HashMap<u16, (Vec<String>, Vec<String>)> = Default::default();
        let (gw, pt, jp, juwel) = (reihe("jahrgang.gw"), reihe("jahrgang.pot"), reihe("jahrgang.juwel_pot"), reihe("jahrgang.juwel"));
        let tw = reihe("jahrgang.torhueter");
        let torhueter = (tw[0] + (zufall(&[&s, "tw"]) * (tw[1] - tw[0] + 1.0)).floor()) as usize; // roh
        let felder = [[crate::id("CB"), crate::id("LB"), crate::id("RB"), crate::id("CB")], [crate::id("DMF"), crate::id("CMF"), crate::id("AMF"), crate::id("CMF")], [crate::id("CF"), crate::id("LWF"), crate::id("RWF"), crate::id("SS")]];
        let mut neu = vec![];
        let mut zeilen = vec![];
        let mut juwelen = vec![];
        for i in 0..n {
            let sk = format!("{s}:{i}");
            let alter = if zufall(&[&sk, "alter"]) < k("jahrgang.alter_15_anteil") { 15 } else { 16 }; // roh
            let pos = if i < torhueter { "GK".to_string() } else { let g = felder[(zufall(&[&sk, "teil"]) * 3.0) as usize % 3]; g[(zufall(&[&sk, "pos"]) * 4.0) as usize % 4].to_string() }; // roh
            let mut ovr = normal(&format!("{sk}:gw"), gw[0] + gw[1] * js, gw[2]).clamp(gw[3], gw[4]);
            let ist_juwel = zufall(&[&sk, "juwel"]) < juwel[0] + juwel[1] * js; // roh
            let mut pot = if ist_juwel { normal(&format!("{sk}:jp"), jp[0], jp[1]).min(jp[2]) } else { normal(&format!("{sk}:pot"), pt[0] + pt[1] * js, pt[2]).clamp(pt[3], pt[4]) };
            let mz = zufall(&[&sk, "merkmal"]); // roh
            let merkmal = if mz < k("jahrgang.spaetzuender") { Some("spaetzuender") } else if mz < k("jahrgang.spaetzuender") + k("jahrgang.fruehentwickler") { Some("fruehentwickler") } else { None }; // roh
            if merkmal == Some("fruehentwickler") { ovr += 3.0; pot -= 2.0; } // roh
            if merkmal == Some("spaetzuender") { ovr -= 2.0; } // roh
            let nat = self.jugend_nat(self.own, &sk);
            let name = self.name_aus(namen.entry(nat).or_insert_with(|| self.namen_von(nat)), &sk);
            let p = self.nachwuchs_anlegen(self.own, name, &pos, alter, ovr, pot.max(ovr + 2.0), nat, 17);
            if let Some(m) = merkmal { self.world.players[p].ent.merkmale.push(m.into()); }
            self.world.eigen.insert(self.world.players[p].id, self.club().id);
            let pl = &self.world.players[p];
            let (pa, pb) = self.pot_range(p);
            zeilen.push(format!("{}{}, {}, {} – {} ({pa:.0}–{pb:.0}){}", if ist_juwel { crate::inhalt::t("ui.jugend.juwel.kurz") } else { "" }, pl.name, pl.age, crate::world::pos_de(&pl.pos), stufe_wort((pa + pb) / 2.0), merkmal.map(|m| format!(" ({})", crate::inhalt::t(&format!("ui.jugend.merkmal.{m}")))).unwrap_or_default())); // roh
            if ist_juwel { juwelen.push(p); }
            neu.push(p);
        }
        let leiter = crate::inhalt::t("meldung.absender.jugendleiter");
        let text = format!("{}\n\n{}", crate::tf!("ui.jugend.talenttag.text", n, format!("{js:.1}")), zeilen.join(crate::id("\n")));
        self.msg(1, leiter, crate::inhalt::t("ui.jugend.talenttag.titel"), &text, Action::None);
        let karte: Vec<(usize, f32)> = neu.iter().map(|&p| (p, self.world.players[p].pot)).collect();
        self.meldung_karte(crate::pages::Art::Talent, crate::id("entwicklung"), karte.into_iter().take(6).collect());
        for p in juwelen {
            let pl = &self.world.players[p];
            let (n, pos) = (pl.name.clone(), crate::world::pos_de(&pl.pos).to_string());
            self.msg(1, leiter, &crate::tf!("ui.jugend.juwel.titel", n), &crate::tf!("ui.jugend.juwel.text", n, pos), Action::None);
            self.meldung_art(crate::pages::Art::Talent, vec![p]);
        }
        neu
    }

    /// Graduates of the AI clubs once a year (Konzept 6.3): number and quality from the youth strength; first the club's own
    /// real young players, then unattached young players of its country, only the rest is generated. Each is marked home-grown.
    pub fn absolventen_jahr(&mut self) {
        let alter_bis = k("absolventen.alter_bis") as u8;
        let (gw, pt, lb) = (reihe("absolventen.gw"), reihe("absolventen.pot"), reihe("absolventen.liga_bonus"));
        let mut frei: std::collections::HashMap<u16, Vec<usize>> = Default::default();
        for (i, p) in self.world.players.iter().enumerate() { if p.team == -1 && p.age <= alter_bis { frei.entry(p.nat).or_default().push(i); } }
        let mut namen: std::collections::HashMap<u16, (Vec<String>, Vec<String>)> = Default::default();
        let season = self.world.season;
        for c in 0..self.world.clubs.len() {
            if c == self.own || self.world.clubs[c].squad.is_empty() { continue; }
            let js = self.jugendstaerke(c).wert;
            let cid = self.world.clubs[c].id;
            let s = format!("absolventen:{cid}:{season}");
            let mut n = (zufall(&[&s, "n"]) + js / k("absolventen.anzahl_teiler")).round() as usize; // roh
            n = n.min((k("absolventen.kader_hoechstens") as usize).saturating_sub(self.world.clubs[c].squad.len()));
            // the club's own real young players come up first
            let eigene: Vec<usize> = self.world.clubs[c].squad.iter().copied().filter(|&p| { let pl = &self.world.players[p]; pl.db && pl.age <= alter_bis && !self.world.eigen.contains_key(&pl.id) }).take(n).collect();
            for &p in &eigene { self.world.eigen.insert(self.world.players[p].id, cid); }
            let mut rest = n.saturating_sub(eigene.len());
            let l = self.world.clubs[c].league;
            let nat = self.world.land_nat(l);
            while rest > 0 {
                let Some(p) = frei.get_mut(&nat).and_then(|v| v.pop()) else { break };
                if self.world.players[p].team != -1 { continue; }
                let lv = self.world.leagues[l].level;
                let pl = &mut self.world.players[p];
                pl.team = c as i32; pl.contract_end = season + 3; pl.wage = fair_wage(pl, lv);
                self.world.clubs[c].squad.push(p);
                self.world.eigen.insert(self.world.players[p].id, cid);
                rest -= 1;
            }
            if rest == 0 { continue; }
            let bonus = match self.world.leagues[l].stufe { 1 => lb[0], 2 => lb[1], _ => 0.0 };
            for i in 0..rest {
                let sk = format!("{s}:{i}");
                let nat = self.jugend_nat(c, &sk);
                let nm = namen.entry(nat).or_insert_with(|| self.namen_von(nat)).clone();
                let ovr = normal(&format!("{sk}:gw"), gw[0] + gw[1] * js + bonus, gw[2]).clamp(38.0, 75.0);
                let pot = normal(&format!("{sk}:pot"), pt[0] + pt[1] * js, pt[2]).clamp(ovr + 2.0, 92.0);
                let pos = ["GK", "CB", "CB", "LB", "RB", "DMF", "CMF", "AMF", "LWF", "RWF", "CF"][(zufall(&[&sk, "pos"]) * 11.0) as usize % 11]; // roh
                let alter = 17 + (zufall(&[&sk, "alter"]) * 2.0) as u8 % 2; // roh
                let name = self.name_aus(&nm, &sk);
                let p = self.nachwuchs_anlegen(c, name, pos, alter, ovr, pot, nat, 0);
                self.world.eigen.insert(self.world.players[p].id, cid);
            }
        }
    }

    // ------------------------------------------------------------ scouting tournament (Konzept 3.2)
    /// Once a season the club holds a scouting tournament: 4–10 youngsters of the region show themselves, 0–2 of them really
    /// good; the best three can be taken into the U17 straight away, the others stay unattached (the pool).
    pub fn sichtungsturnier(&mut self) -> Result<String, String> {
        let season = self.world.season;
        if self.sichtung_saison == season + 1 { return Err(crate::inhalt::t("ui.jugend.sichtung.schon").into()); }
        let stufe = self.wi.stufen[3].max(1) as f32;
        let kosten = (k("sichtung.kosten_basis") + k("sichtung.kosten_je_stufe") * stufe) as f64;
        if self.club().balance < kosten { return Err(crate::inhalt::t("ui.jugend.sichtung.geld").into()); }
        self.sichtung_saison = season + 1;
        self.book(crate::inhalt::t("ui.jugend.sichtung.titel"), -kosten);
        let js = self.jugendstaerke(self.own).wert;
        let s = format!("sichtung:{}:{season}", self.club().id);
        let n = 4 + (zufall(&[&s, "n"]) * 7.0) as usize; // roh
        let nat = self.world.land_nat(self.league());
        let namen = self.namen_von(nat);
        let mut talente = vec![];
        for i in 0..n {
            let sk = format!("{s}:{i}");
            let gut = zufall(&[&sk, "gut"]) < k("sichtung.gut_chance"); // roh
            let ovr = normal(&format!("{sk}:gw"), k("sichtung.gw") + 1.2 * js + if gut { 6.0 } else { 0.0 }, 5.0).clamp(35.0, 64.0);
            let pot = normal(&format!("{sk}:pot"), k("sichtung.pot") + 2.0 * js + if gut { 12.0 } else { 0.0 }, 7.0).clamp(ovr + 2.0, 92.0);
            let pos = ["GK", "CB", "LB", "RB", "DMF", "CMF", "AMF", "LWF", "RWF", "CF"][(zufall(&[&sk, "pos"]) * 10.0) as usize % 10]; // roh
            let alter = 15 + (zufall(&[&sk, "alter"]) * 2.0) as u8 % 2; // roh
            let name = self.name_aus(&namen, &sk);
            let p = self.nachwuchs_anlegen(self.own, name, pos, alter, ovr, pot, nat, 17);
            self.release(p);
            self.world.players[p].know = 0.6;
            talente.push(p);
        }
        talente.sort_by(|a, b| self.world.players[*b].pot.partial_cmp(&self.world.players[*a].pot).unwrap());
        let zeilen: Vec<String> = talente.iter().map(|&p| { let pl = &self.world.players[p]; let (pa, pb) = self.pot_range(p); format!("{}, {}, {} – {} ({pa:.0}–{pb:.0})", pl.name, pl.age, crate::world::pos_de(&pl.pos), stufe_wort((pa + pb) / 2.0)) }).collect();
        let mut opts: Vec<(String, String)> = talente.iter().take(3).map(|&p| (format!("{p}"), crate::tf!("ui.jugend.sichtung.nehmen", self.world.players[p].name.clone()))).collect();
        // Konzept 3.2: a trial first — a few days with us, then the estimate is much sharper
        opts.extend(talente.iter().take(3).map(|&p| (format!("probe:{p}"), crate::tf!("ui.jugend.probe.knopf", self.world.players[p].name.clone()))));
        opts.push((crate::id("keinen").into(), crate::inhalt::t("ui.jugend.sichtung.keinen").into()));
        let text = format!("{}\n\n{}", crate::tf!("ui.jugend.sichtung.text", n, money_txt(kosten)), zeilen.join(crate::id("\n")));
        self.msg(1, crate::inhalt::t("meldung.absender.jugendleiter"), crate::inhalt::t("ui.jugend.sichtung.titel"), &text, Action::Wahl { art: crate::id("sichtung").into(), bezug: talente.iter().take(3).copied().collect(), optionen: opts });
        Ok(crate::tf!("ui.jugend.sichtung.ok", n))
    }
    pub fn sichtung_antwort(&mut self, key: &str) -> String {
        if let Some(p) = key.strip_prefix(crate::id("probe:")).and_then(|x| x.parse::<usize>().ok()) { return self.probetraining(p).unwrap_or_else(|e| e); }
        let Ok(p) = key.parse::<usize>() else { return crate::inhalt::t("ui.akademie.notiert").into() };
        if p >= self.world.players.len() || self.world.players[p].team != -1 { return crate::inhalt::t("ui.jugend.sichtung.weg").into(); }
        let pl = &mut self.world.players[p];
        pl.team = self.own as i32; pl.youth = 17; pl.know = 1.0; pl.contract_end = self.world.season + k("jahrgang.jahre") as u32; pl.wage = k("jahrgang.gehalt") as f64;
        self.world.clubs[self.own].squad.push(p);
        self.world.eigen.insert(self.world.players[p].id, self.club().id);
        crate::tf!("ui.jugend.sichtung.aufgenommen", self.world.players[p].name.clone())
    }

    // ------------------------------------------------------------ trial training (Konzept 3.2)
    /// Invites a talent for a trial of a few days (cost, at most 3 at once); its end sharpens the estimate.
    pub fn probetraining(&mut self, p: usize) -> Result<String, String> {
        if p >= self.world.players.len() || self.world.players[p].team != -1 { return Err(crate::inhalt::t("ui.jugend.sichtung.weg").into()); }
        if self.probetrainings.len() >= k("probe.hoechstens") as usize { return Err(crate::inhalt::t("ui.jugend.probe.voll").into()); }
        if self.probetrainings.iter().any(|x| x.0 == p) { return Err(crate::inhalt::t("ui.jugend.probe.schon").into()); }
        let kosten = k("probe.kosten") as f64;
        self.book(crate::inhalt::t("ui.jugend.probe.buchung"), -kosten);
        let bis = self.abs() + k("probe.tage") as u32;
        self.probetrainings.push((p, bis));
        Ok(crate::tf!("ui.jugend.probe.ok", self.world.players[p].name.clone(), k("probe.tage")))
    }
    /// Daily: finished trials report back — knowledge +30, family warmer when it went well, and the choice to take him.
    pub fn probetraining_tag(&mut self) {
        let abs = self.abs();
        let (fertig, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.probetrainings).into_iter().partition(|x| x.1 <= abs);
        self.probetrainings = rest;
        for (p, _) in fertig {
            if self.world.players[p].team != -1 { continue; }
            let k2 = (self.world.players[p].know + k("probe.wissen") / 100.0).min(0.97);
            self.world.players[p].know = k2;
            let (pa, pb) = self.pot_range(p);
            let gut = (pa + pb) / 2.0 >= k("probe.gut_ab");
            let n = self.world.players[p].name.clone();
            let text = crate::tf!(if gut { "ui.jugend.probe.gut" } else { "ui.jugend.probe.mittel" }, n.clone(), pa, pb, stufe_wort((pa + pb) / 2.0));
            let opts = vec![(format!("{p}"), crate::tf!("ui.jugend.sichtung.nehmen", n.clone())), (crate::id("keinen").into(), crate::inhalt::t("ui.jugend.probe.nein").into())];
            self.msg(1, crate::inhalt::t("meldung.absender.jugendleiter"), &crate::tf!("ui.jugend.probe.titel", n), &text, Action::Wahl { art: crate::id("sichtung").into(), bezug: vec![p], optionen: opts });
        }
    }

    // ------------------------------------------------------------ school (Konzept 4.3, kept light)
    /// Once a month for the own players up to 18: travelling (first-team trips, national youth) lets school wobble,
    /// a boarding school (academy level 3+) keeps it steady; a wobbling one recovers or breaks off.
    pub fn schule_monat(&mut self) {
        let heute = self.world.heute();
        let internat = self.wi.stufen[3] >= k("schule.internat_ab") as u8;
        for p in self.world.clubs[self.own].squad.clone() {
            let pl = &self.world.players[p];
            if pl.age > 18 { continue; }
            let id = pl.id;
            let reist = pl.profil.oben + 30 > heute + 1 && pl.profil.oben > 0 || pl.profil.laenderspiele > 0;
            let st = self.schule.get(&id).copied().unwrap_or(0);
            let z = zufall(&["schule", &id.to_string(), &(self.abs() / 28).to_string()]); // roh
            let neu = match st {
                0 if reist && z < k("schule.wackeln") * if internat { 0.2 } else { 1.0 } => 1,
                1 if reist && z < k("schule.abbruch") => 2,
                1 if !reist && z < k("schule.erholen") => 0,
                x => x,
            };
            if neu == st { continue; }
            self.schule.insert(id, neu);
            let n = self.world.players[p].name.clone();
            let titel = crate::tf!(&format!("ui.jugend.schule.{neu}"), n.clone());
            if neu == 2 && cw(&self.world.players[p], PROF) < 60.0 { self.world.players[p].morale = (self.world.players[p].morale - 3.0).max(5.0); }
            self.msg(6, crate::inhalt::t("meldung.absender.jugendleiter"), &titel, &crate::tf!(&format!("ui.jugend.schule.{neu}.text"), n), Action::None);
        }
    }

    // ------------------------------------------------------------ mentors (Konzept 5.3)
    fn darf_mentor(&self, m: usize) -> bool {
        let pl = &self.world.players[m];
        pl.youth == 0 && pl.age as f32 >= k("mentor.mentor_ab") && cw(pl, FUEHRUNG) >= k("mentor.fuehrung_ab") && cw(pl, PROF) >= k("mentor.prof_ab") && cw(pl, EGO) <= k("mentor.ego_bis")
            && self.world.clubs[self.own].squad.iter().filter(|&&s| self.world.players[s].ps.mentor == Some(pl.id)).count() < k("mentor.schuetzlinge") as usize
    }
    fn darf_schuetzling(&self, s: usize) -> bool {
        let pl = &self.world.players[s];
        pl.age as f32 <= k("mentor.schuetzling_bis") && pl.ps.mentor.is_none() && (pl.youth != 17 || pl.ps.gast)
    }
    /// Fit of a pair (0–1) and its reasons: 0,35·position + 0,20·country + 0,25·personality + 0,20·what he can learn.
    pub fn mentor_paar(&self, m: usize, s: usize) -> (f32, Vec<&'static str>) {
        let (mp, sp) = (&self.world.players[m], &self.world.players[s]);
        let mut gruende = vec![];
        let pos = if mp.pos == sp.pos { gruende.push("ui.jugend.grund.position"); 1.0 } else if crate::entwicklung::bereich(crate::pesdb::pos_index(&mp.pos) as usize) == crate::entwicklung::bereich(crate::pesdb::pos_index(&sp.pos) as usize) { 0.6 } else { 0.2 };
        let land = if mp.nat == sp.nat { gruende.push("ui.jugend.grund.land"); 1.0 } else { 0.3 };
        let (tm, ts) = (cw(mp, TEMPERAMENT), cw(sp, TEMPERAMENT));
        let mut pers = (cw(mp, PROF) / 100.0 + (1.0 - (cw(mp, EGO) / 100.0))) / 2.0;
        if tm < 40.0 && ts > 65.0 { pers *= 1.2; gruende.push("ui.jugend.grund.ruhig"); }
        if tm > 65.0 && ts > 65.0 { pers *= 0.5; }
        let lernen = if mp.attrs.len() == 25 && sp.attrs.len() == 25 {
            let w = crate::pesdb::wvec(crate::pesdb::pos_index(&sp.pos));
            let haupt: Vec<usize> = (0..25).filter(|&i| w[i] > 0.0).collect();
            haupt.iter().filter(|&&i| mp.attrs[i] - sp.attrs[i] >= 8.0).count() as f32 / haupt.len().max(1) as f32
        } else { 0.5 };
        if lernen >= 0.5 { gruende.push("ui.jugend.grund.lernen"); }
        ((0.35 * pos + 0.20 * land + 0.25 * pers.min(1.0) + 0.20 * lernen).clamp(0.0, 1.0), gruende)
    }
    /// The co-trainer's proposals: at most two new pairs, best fit first (monthly, or on the mentor page).
    pub fn mentor_vorschlaege(&mut self) -> usize {
        let sq = self.world.clubs[self.own].squad.clone();
        let mut paare: Vec<(f32, usize, usize)> = vec![];
        for &s in sq.iter().filter(|&&s| self.darf_schuetzling(s)) {
            for &m in sq.iter().filter(|&&m| m != s && self.darf_mentor(m)) {
                if (self.world.players[m].age as f32) < self.world.players[s].age as f32 + k("mentor.abstand") { continue; }
                // a mentor must be better than his pupil, or there is nothing to pass on
                if self.world.players[m].ovr < self.world.players[s].ovr + k("mentor.staerker_um") { continue; }
                let (p, _) = self.mentor_paar(m, s);
                if p >= k("mentor.passung_ab") { paare.push((p, m, s)); }
            }
        }
        paare.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let (mut genutzt, mut n) = (vec![], 0);
        let co = self.co_name.clone();
        for (p, m, s) in paare {
            if n >= k("mentor.vorschlaege") as usize || genutzt.contains(&m) || genutzt.contains(&s) { continue; }
            genutzt.extend([m, s]);
            n += 1;
            let (mn, sn) = (self.world.players[m].name.clone(), self.world.players[s].name.clone());
            let gruende = self.mentor_paar(m, s).1.iter().map(|g| crate::inhalt::t(g)).collect::<Vec<_>>().join(" ");
            let opts = vec![(crate::id("ja").to_string(), crate::inhalt::t("ui.jugend.mentor.ja").to_string()), (crate::id("nein").to_string(), crate::inhalt::t("ui.jugend.mentor.nein").to_string())];
            self.msg(1, &co, &crate::tf!("ui.jugend.mentor.titel", mn, sn), &crate::tf!("ui.jugend.mentor.text", mn, sn, gruende, format!("{:.0}", p * 100.0)), Action::Wahl { art: crate::id("mentor").into(), bezug: vec![s, m], optionen: opts });
        }
        n
    }
    pub fn mentor_antwort(&mut self, s: usize, m: usize, key: &str) -> String {
        if key != "ja" { return crate::inhalt::t("ui.jugend.mentor.abgelehnt").into(); }
        if self.world.players[s].team != self.own as i32 || self.world.players[m].team != self.own as i32 { return crate::inhalt::t("ui.mensch.er_ist_nicht_mehr_bei").into(); }
        let mp = &self.world.players[m];
        let chance = k("mentor.ja_basis") + k("mentor.ja_fuehrung") * cw(mp, FUEHRUNG) / 100.0;
        let mn = mp.name.clone();
        if mp.morale < k("mentor.moral_nein") || zufall(&["mentor_ja", &mp.id.to_string(), &self.abs().to_string()]) >= chance { // roh
            return crate::tf!("ui.jugend.mentor.will_nicht", mn);
        }
        self.world.players[s].ps.mentor = Some(self.world.players[m].id);
        let sn = self.world.players[s].name.clone();
        self.moral_wirkung(s, 2.0, 0, &crate::tf!("ui.jugend.mentor.grund", mn));
        crate::tf!("ui.jugend.mentor.ok", mn, sn)
    }

    // ------------------------------------------------------------ training guests, closing round (Konzept 5.4)
    pub fn gast_setzen(&mut self, p: usize, an: bool) -> Result<String, String> {
        if !an { self.world.players[p].ps.gast = false; return Ok(crate::inhalt::t("ui.jugend.gast.zurueck").into()); }
        let jetzt = self.world.clubs[self.own].squad.iter().filter(|&&q| self.world.players[q].ps.gast).count();
        if jetzt >= k("hochziehen.gast_hoechstens") as usize { return Err(crate::tf!("ui.jugend.gast.voll", k("hochziehen.gast_hoechstens"))); }
        self.world.players[p].ps.gast = true;
        self.moral_wirkung(p, k("hochziehen.gast_moral"), 0, crate::inhalt::t("ui.jugend.gast.grund"));
        Ok(crate::tf!("ui.jugend.gast.ok", self.world.players[p].name.clone()))
    }
    /// Development factor of an academy player (Konzept 5.2): youth coaches, facilities, level against his league, guest.
    pub fn akademie_faktor(&self, p: usize) -> f32 {
        let pl = &self.world.players[p];
        if pl.youth == 0 { return 1.0; }
        let trainer = 1.0 + k("faktoren.trainer_hebel") * (self.trainingsqualitaet() - 50.0) / 50.0;
        let a = reihe("faktoren.anlagen");
        let anlagen = a[0] + a[1] * self.wi.stufen[3].max(1) as f32;
        let yk = if pl.youth == 17 { 0 } else { 1 };
        let gegner: Vec<f32> = self.youth[yk].rows.iter().filter(|r| !r.own).map(|r| r.str_).collect();
        let schnitt = if gegner.is_empty() { pl.ovr } else { gegner.iter().sum::<f32>() / gegner.len() as f32 };
        let abst = k("faktoren.niveau_abstand");
        let niveau = if pl.ps.gast { k("hochziehen.gast_faktor") } else if pl.ovr >= schnitt + abst { k("faktoren.unterfordert") } else if pl.ovr <= schnitt - abst { k("faktoren.ueberfordert") } else { 1.0 };
        trainer * anlagen * niveau
    }
    /// Season start: U17 → U19 by birth year; who turns 19 comes to the closing round (Profis, Leihe, Freigabe); mentorships end at 22.
    pub fn jugend_saisonwechsel(&mut self) {
        let ab = k("hochziehen.abschluss_alter") as u8;
        let ohne_zweite = self.world.zweite_von(self.own).is_none();
        let leiter = crate::inhalt::t("meldung.absender.jugendleiter");
        for p in self.world.clubs[self.own].squad.clone() {
            let pl = &mut self.world.players[p];
            if pl.youth == 17 && pl.age >= 17 { pl.youth = 19; }
            // Konzept 4.3: at 18 the school is done — a small note and morale for a good finish
            if pl.age == 18 && self.schule.get(&pl.id).copied().unwrap_or(0) == 0 && pl.youth > 0 {
                pl.morale = (pl.morale + 2.0).min(95.0);
                let n = pl.name.clone();
                self.schule.insert(self.world.players[p].id, 3);
                let vk = self.vkey("ui.jugend.schule.abschluss.text", "");
                self.msg(6, leiter, &crate::tf!("ui.jugend.schule.abschluss", n.clone()), &crate::tf!(&vk, n), Action::None);
                continue;
            }
            let pl = &mut self.world.players[p];
            if pl.ps.mentor.is_some() && pl.age as f32 > k("mentor.schuetzling_bis") {
                pl.ps.mentor = None;
                let n = pl.name.clone();
                self.msg(1, leiter, &crate::tf!("ui.jugend.mentor.ende", n), &crate::tf!("ui.jugend.mentor.ende_text", n), Action::None);
                continue;
            }
            let pl = &self.world.players[p];
            if pl.youth == 19 && pl.age >= ab && ohne_zweite {
                let n = pl.name.clone();
                let opts = vec![(crate::id("profi").to_string(), crate::inhalt::t("ui.jugend.abschluss.profi").to_string()), (crate::id("leihe").to_string(), crate::inhalt::t("ui.jugend.abschluss.leihe").to_string()), (crate::id("frei").to_string(), crate::inhalt::t("ui.jugend.abschluss.frei").to_string())];
                let (pa, pb) = self.pot_range(p);
                self.msg(1, leiter, &crate::tf!("ui.jugend.abschluss.titel", n), &crate::tf!("ui.jugend.abschluss.text", n, format!("{:.0}", pl.ovr), pa, pb), Action::Wahl { art: crate::id("abschluss").into(), bezug: vec![p], optionen: opts });
            }
        }
        self.absolventen_jahr();
    }
    pub fn abschluss_antwort(&mut self, p: usize, key: &str) -> String {
        if self.world.players[p].team != self.own as i32 { return crate::inhalt::t("ui.mensch.er_ist_nicht_mehr_bei").into(); }
        match key {
            "profi" => { self.promote(p); crate::inhalt::t("ui.akademie.er_gehoert_jetzt_zum_profikader").into() }
            "leihe" => match self.leihe_abgeben(p) { Ok(()) => crate::inhalt::t("ui.leihe.auf_liste").into(), Err(e) => e },
            _ => { self.release(p); crate::inhalt::t("ui.jugend.abschluss.frei_ok").into() }
        }
    }

    /// The monthly academy report (Konzept 8.2): winners, worries, table places, who is ready for the first team.
    pub fn akademie_bericht(&mut self) {
        self.schule_monat();
        let jung: Vec<usize> = self.world.clubs[self.own].squad.iter().copied().filter(|&p| self.world.players[p].youth > 0).collect();
        if jung.is_empty() { return; }
        let summe = |p: usize| self.world.players[p].ent.verlauf.iter().rev().take(4).map(|w| w.gruppen.iter().sum::<f32>()).sum::<f32>();
        let mut v: Vec<(usize, f32)> = jung.iter().map(|&p| (p, summe(p))).collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let name = |p: usize| self.world.players[p].name.clone();
        let mut body = vec![crate::tf!("ui.jugend.bericht.staerke", format!("{:.1}", self.jugendstaerke(self.own).wert), self.jugend_ruf())];
        let gew: Vec<String> = v.iter().filter(|x| x.1 > 0.0).take(3).map(|x| name(x.0)).collect();
        if !gew.is_empty() { body.push(crate::tf!("ui.jugend.bericht.gewinner", gew.join(", "))); }
        let sorgen: Vec<String> = jung.iter().filter(|&&p| self.world.players[p].morale < 40.0).take(2).map(|&p| name(p)).chain(v.iter().rev().filter(|x| x.1 < 0.0).take(1).map(|x| name(x.0))).collect();
        if !sorgen.is_empty() { body.push(crate::tf!("ui.jugend.bericht.sorgen", sorgen.join(", "))); }
        for k in 0..2 {
            if let Some(pl) = self.youth[k].rows.iter().position(|r| r.own) { body.push(crate::tf!("ui.jugend.bericht.tabelle", self.youth[k].name.clone(), pl + 1)); }
        }
        let schwach = self.xi.iter().map(|&p| self.world.players[p].ovr).fold(f32::MAX, f32::min);
        let bereit: Vec<String> = jung.iter().filter(|&&p| self.world.players[p].ovr >= schwach - f("akademie", "befoerderung.abstand_profi")).map(|&p| name(p)).collect();
        if !bereit.is_empty() { body.push(crate::tf!("ui.jugend.bericht.bereit", bereit.join(", "))); }
        self.msg(1, crate::inhalt::t("meldung.absender.jugendleiter"), crate::inhalt::t("ui.jugend.bericht.titel"), &body.join(crate::id("\n")), Action::None);
        self.meldung_karte(crate::pages::Art::Talent, crate::id("entwicklung"), v.into_iter().take(6).collect());
    }
}

impl Career {
    /// Saves from before the youth spread (Fabian 24.11.: an old academy where nearly everyone is 66): the own
    /// academy players are drawn once more like a talent day's group (rating by age, potential with few talents).
    /// True when it happened.
    pub fn jugend_neu_wuerfeln(&mut self) -> bool {
        let z = self.zweite_mannschaft();
        let jung: Vec<usize> = (0..self.world.players.len()).filter(|&p| { let pl = &self.world.players[p]; pl.youth > 0 && (pl.team == self.own as i32 || z.is_some_and(|z| pl.team == z as i32)) }).collect();
        if jung.len() < 5 { return false; }
        let mut zahl: std::collections::HashMap<i32, usize> = Default::default();
        for &p in &jung { *zahl.entry(self.world.players[p].ovr.round() as i32).or_default() += 1; }
        if zahl.values().copied().max().unwrap_or(0) * 10 < jung.len() * 6 { return false; }
        let js = self.jugendstaerke(self.own).wert;
        let (gw, pt, jp, juwel) = (reihe("jahrgang.gw"), reihe("jahrgang.pot"), reihe("jahrgang.juwel_pot"), reihe("jahrgang.juwel"));
        for p in jung {
            let age = self.world.players[p].age;
            let pos = self.world.players[p].pos.clone();
            let r = &mut self.world.rng;
            let ovr = (gw[0] + gw[1] * js + (age as f32 - 15.5) * 3.0 + r.gauss(gw[2])).clamp(gw[3], gw[4] + 6.0);
            let pot = if r.f() < juwel[0] + juwel[1] * js { (jp[0] + r.gauss(jp[1])).min(jp[2]) } else { (pt[0] + pt[1] * js + r.gauss(pt[2])).clamp(pt[3], pt[4]) }.max(ovr + 3.0);
            let (attrs, pp) = self.world.template_attrs(&pos, ovr);
            let pl = &mut self.world.players[p];
            (pl.ovr, pl.pot, pl.attrs, pl.pp) = (ovr, pot, attrs, pp);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Fabian 24.11.: an old save with every academy player at 66 gets a spread once, and only once.
    #[test]
    fn alte_akademie_neu_gewuerfelt() {
        let mut c = Career::new(crate::world::testwelt(7), 0, "Test");
        let jung: Vec<usize> = (0..c.world.players.len()).filter(|&p| c.world.players[p].youth > 0 && c.world.players[p].team == c.own as i32).collect();
        if jung.len() < 5 { return; }
        for &p in &jung { c.world.players[p].ovr = 66.0; }
        assert!(c.jugend_neu_wuerfeln());
        let v: Vec<f32> = jung.iter().map(|&p| c.world.players[p].ovr).collect();
        assert!(v.iter().any(|&x| (x - 66.0).abs() > 1.0), "{v:?}");
        assert!(!c.jugend_neu_wuerfeln(), "a spread academy stays as it is");
    }
    #[test]
    fn jugend_hat_streuung() {
        let w = crate::world::testwelt(7);
        let c = Career::new(w, 0, "Test");
        let v: Vec<f32> = c.world.clubs[c.own].squad.iter().copied().filter(|&p| c.world.players[p].youth > 0).map(|p| c.world.players[p].ovr).collect();
        if v.len() >= 5 {
            let m = v.iter().sum::<f32>() / v.len() as f32;
            let sd = (v.iter().map(|x| (x - m).powi(2)).sum::<f32>() / v.len() as f32).sqrt();
            assert!(sd > 2.0, "youth ratings spread: {v:?}");
        }
        // few talents: most potentials stay below 80 (Fabian 24.11.: nearly all were 80–95); over several worlds,
        // one academy alone swings between 5 and 10 of 30 with the draw
        let mut pot: Vec<f32> = vec![];
        for seed in 1..=6 {
            let c = Career::new(crate::world::testwelt(seed), 0, "Test");
            pot.extend(c.world.clubs[c.own].squad.iter().copied().filter(|&p| c.world.players[p].youth > 0).map(|p| c.world.players[p].pot));
        }
        assert!(pot.iter().filter(|&&x| x >= 80.0).count() * 3 <= pot.len(), "potentials: {pot:?}");
    }
    #[test]
    fn jugend_herkunft_nach_ruf() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        // the built-in world has no countries: two thirds home (1), the rest from country 2
        for (i, p) in c.world.players.iter_mut().enumerate() { p.nat = if i % 3 == 0 { 2 } else { 1 }; }
        let heim = c.world.land_nat(c.world.clubs[0].league);
        assert_eq!(heim, 1);
        let mut anteil = |rep: f32| { c.world.clubs[0].rep = rep; (0..400).filter(|i| c.jugend_nat(0, &format!("h{i}")) != heim).count() as f32 / 400.0 };
        let (klein, gross) = (anteil(0.0), anteil(100.0));
        assert!(klein < 0.1 && gross > 0.2 && gross < 0.5, "foreigners small club {klein}, big club {gross}");
    }
    #[test]
    fn jugendnamen_wiederholen_sich_nicht() {
        let w = crate::world::testwelt(7);
        let c = Career::new(w, 0, "Test");
        let namen = c.namen_von(c.world.land_nat(c.league()));
        let v: Vec<String> = (0..200).map(|i| c.name_aus(&namen, &format!("t{i}"))).collect();
        let mut u = v.clone(); u.sort(); u.dedup();
        assert_eq!(u.len(), v.len(), "no name twice");
        assert!(v.iter().all(|n| !c.world.players.iter().any(|p| &p.name == n)), "nobody in the world has it already");
    }
    #[test]
    fn jugendstaerke_talenttag_absolventen_mentor() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        // every club has a youth strength 1–10, a known academy sits high
        for cl in 0..c.world.clubs.len() { let j = c.jugendstaerke(cl); assert!((1.0..=10.0).contains(&j.wert), "{:?}", j); }
        // the talent day brings a U17 year group of 15/16-year-olds, all home-grown
        let neu = c.talenttag();
        assert!(!neu.is_empty());
        for &p in &neu { let pl = &c.world.players[p]; assert!(pl.youth == 17 && (15..=16).contains(&pl.age) && pl.pot > pl.ovr); assert!(c.world.eigen.contains_key(&pl.id)); }
        assert!(c.talenttag().len() <= c.akademie_plaetze().0, "places cap the group");
        // graduates: strong youth brings more over five years than weak youth, ids stay unique
        let mut nach_staerke = [(0usize, 0usize); 2];
        for _ in 0..5 {
            let vor: std::collections::HashSet<u32> = c.world.eigen.keys().copied().collect();
            let js: Vec<f32> = (0..c.world.clubs.len()).map(|x| c.jugendstaerke(x).wert).collect();
            for cl in 0..c.world.clubs.len() { let sq = c.world.clubs[cl].squad.clone(); for p in sq.into_iter().skip(24) { c.world.players[p].team = -2; c.world.clubs[cl].squad.retain(|&x| x != p); } }
            c.absolventen_jahr();
            for (cl, &j) in js.iter().enumerate() {
                if cl == c.own { continue; }
                let n = c.world.clubs[cl].squad.iter().filter(|&&p| { let id = c.world.players[p].id; !vor.contains(&id) && c.world.eigen.get(&id) == Some(&c.world.clubs[cl].id) }).count();
                let g = if j >= 6.0 { 1 } else { 0 };
                nach_staerke[g].0 += n; nach_staerke[g].1 += 1;
            }
            c.world.season += 1;
        }
        let schnitt = |g: usize| nach_staerke[g].0 as f32 / nach_staerke[g].1.max(1) as f32;
        assert!(nach_staerke[0].1 == 0 || nach_staerke[1].1 == 0 || schnitt(1) > schnitt(0), "stark {} schwach {}", schnitt(1), schnitt(0));
        let ids: std::collections::HashSet<u32> = c.world.players.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), c.world.players.len(), "no duplicate ids");
        // mentors: only the young as protégés, the fit is 0–1
        let sq = c.squad();
        let alt = sq.iter().copied().max_by_key(|&p| c.world.players[p].age).unwrap();
        let jung = sq.iter().copied().min_by_key(|&p| c.world.players[p].age).unwrap();
        let (p, _) = c.mentor_paar(alt, jung);
        assert!((0.0..=1.0).contains(&p));
        c.mentor_vorschlaege();
        // proposals only for the young, and the mentor is always the better player
        for m in c.inbox.iter().filter_map(|m| match &m.action { Action::Wahl { art, bezug, .. } if art == "mentor" => Some(bezug.clone()), _ => None }) {
            let (s, mt) = (&c.world.players[m[0]], &c.world.players[m[1]]);
            assert!(s.age <= 21 && mt.ovr > s.ovr, "{} ({}, {}) as pupil of {} ({})", s.name, s.age, s.ovr, mt.name, mt.ovr);
        }
        // the scouting tournament: once a season, talents in the pool, one taken into the U17
        c.world.clubs[c.own].balance = 1e9;
        let n_vor = c.world.players.len();
        assert!(c.sichtungsturnier().is_ok());
        assert!(c.sichtungsturnier().is_err(), "only once a season");
        let neu: Vec<usize> = (n_vor..c.world.players.len()).collect();
        assert!((4..=10).contains(&neu.len()) && neu.iter().all(|&p| c.world.players[p].team == -1));
        let wahl = neu.iter().copied().max_by(|a, b| c.world.players[*a].pot.partial_cmp(&c.world.players[*b].pot).unwrap()).unwrap();
        c.sichtung_antwort(&wahl.to_string());
        assert!(c.world.players[wahl].team == c.own as i32 && c.world.players[wahl].youth == 17);
        // the academy factor stays in a sensible band
        for &p in &neu { let f = c.akademie_faktor(p); assert!((0.6..=1.5).contains(&f), "{f}"); }
    }
}
