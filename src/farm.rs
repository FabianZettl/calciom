//! Second teams and farm teams (own rules, assets/regeln/farm.toml; no Python original).
//! A second team is a club of its own in its real league; moving a player between first and second team only changes
//! his club, so FL27 gets an ordinary PlayerAssignment.bin entry. Team.bin has no field for the parent club: second
//! teams are recognised by name (suffix " II", " B", " 2", " Reserves", " Amateure" or prefix "Jong ", the rest must be a
//! club of the same country) and by assets/zweite_mannschaften.json. Farm teams are cooperations (World::koops).
use crate::game::*;
use crate::date;
use crate::mensch::zufall;
use crate::regeln::{f, wert_oder};
use crate::world::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


/// Club words left out when names are compared (assets/regeln/namen.toml vereinswoerter).
fn klubwort(w: &str) -> bool { crate::mensch::arr("namen", "vereinswoerter").iter().any(|x| x.as_str() == Some(w)) }
pub fn kern(n: &str) -> String { falten(n).split_whitespace().filter(|w| !klubwort(w)).collect::<Vec<_>>().join(" ") }
fn tx(k: &str) -> String {
    let t = crate::inhalt::eigen("farm");
    k.split('.').fold(t, |v, p| &v[p]).as_str().unwrap_or("").to_string()
}
fn fz(k: &str) -> f32 { f("farm", k) }
/// Second teams without a suffix: assets/zweite_mannschaften.json, plus the same file in the data folder (the person's additions).
fn eigene_liste() -> HashMap<String, String> {
    let mut m: HashMap<String, String> = HashMap::new();
    let lesen = |t: &str, m: &mut HashMap<String, String>| { if let Ok(v) = serde_json::from_str::<serde_json::Value>(t) { if let Some(o) = v["paare"].as_object() { for (k, w) in o { if let Some(w) = w.as_str() { m.insert(k.clone(), w.to_string()); } } } } };
    lesen(&crate::pakete::oder("zweite_mannschaften.json", include_str!("../assets/zweite_mannschaften.json")), &mut m);
    if let Ok(t) = std::fs::read_to_string(crate::game::save_dir().join(crate::id("../zweite_mannschaften.json"))) { lesen(&t, &mut m); }
    m
}

/// A farm-team cooperation: contract until the end of season `bis`, yearly amount, loan places, the promises.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Koop {
    pub haupt: usize,
    pub partner: usize,
    pub bis: u32,
    pub betrag: f64,
    pub plaetze: u8,
    /// Promised: at least this many loan players.
    pub min_leihen: u8,
    /// Promised (Spielzeit-Zusage): loan players stay for the whole season, no recall.
    pub bleiben: bool,
    #[serde(default)]
    pub verstoesse: u8,
}
/// The person's search for a farm team.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KoopSuche { pub bis: u32, pub nur_land: bool, pub stufe_von: u8, pub stufe_bis: u8, pub plaetze: u8, pub max_betrag: f64 }
/// An application (or a request of a small club by itself), answered in the inbox or on the page.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KoopAngebot {
    pub id: u32,
    pub club: usize,
    pub betrag: f64,
    pub min_leihen: u8,
    pub bleiben: bool,
    pub grund: String,
    pub frist: u32,
    pub plaetze: u8,
    #[serde(default)]
    pub phase: Option<crate::verhandlung::Phase>,
    #[serde(default)]
    pub antwort: String,
}

impl World {
    pub fn heute(&self) -> u32 { abs_day(self.season, self.day) }
    /// Recognise second teams: (second, first) pairs and unclear cases ("bitte prüfen").
    pub fn zweite_erkennen(&self) -> (Vec<(usize, usize)>, Vec<(usize, String)>) {
        let n = self.clubs.len();
        let land = |c: usize| self.leagues[self.clubs[c].league].land.clone();
        let kerne: Vec<String> = self.clubs.iter().map(|c| kern(&c.name)).collect();
        let liste = eigene_liste();
        let teile = self.zweite_namensteile();
        let finde = |rest: &str, c: usize| -> Result<Option<usize>, String> {
            let k = kern(rest);
            if k.is_empty() { return Ok(None); }
            let gleich: Vec<usize> = (0..n).filter(|&o| o != c && land(o) == land(c) && (self.clubs[o].name == rest || kerne[o] == k)).collect();
            if gleich.len() == 1 { return Ok(Some(gleich[0])); }
            if gleich.len() > 1 { return Err(crate::tf!("ui.farm.mehrere_moegliche_hauptvereine", gleich.iter().map(|&o| self.clubs[o].name.clone()).collect::<Vec<_>>().join(", "))); }
            let anfang: Vec<usize> = (0..n).filter(|&o| o != c && land(o) == land(c) && kerne[o].starts_with(&format!("{k} "))).collect();
            match anfang.len() { 0 => Ok(None), 1 => Ok(Some(anfang[0])), _ => Err(crate::tf!("ui.farm.mehrere_moegliche_hauptvereine", anfang.iter().map(|&o| self.clubs[o].name.clone()).collect::<Vec<_>>().join(", "))) }
        };
        let (mut paare, mut unklar) = (vec![], vec![]);
        for c in 0..n {
            let name = self.clubs[c].name.as_str();
            if let Some(ziel) = liste.get(name) {
                match finde(ziel, c) { Ok(Some(e)) => paare.push((c, e)), Ok(None) => unklar.push((c, crate::tf!("ui.farm.hauptverein_ist_in_dieser_welt", ziel))), Err(e) => unklar.push((c, e)) }
                continue;
            }
            let Some(rest) = teile.iter().find_map(|(vorn, z)| if *vorn { name.strip_prefix(z.as_str()) } else { name.strip_suffix(z.as_str()) }) else { continue };
            match finde(rest.trim(), c) { Ok(Some(e)) => paare.push((c, e)), Ok(None) => {}, Err(e) => unklar.push((c, e)) }
        }
        // a first team cannot be a second team itself
        let zweite: Vec<usize> = paare.iter().map(|x| x.0).collect();
        paare.retain(|x| !zweite.contains(&x.1));
        (paare, unklar)
    }
    pub fn erste_von(&self, c: usize) -> Option<usize> { self.zweite.iter().find(|x| x.0 == c).map(|x| x.1) }
    pub fn zweite_von(&self, c: usize) -> Option<usize> { self.zweite.iter().find(|x| x.1 == c).map(|x| x.0) }
    /// A second team never goes up into the league of its first team or higher, nor above its country's highest tier
    /// for second teams (daten/ligaregeln.toml; 0 = not in the league system). Others always may.
    pub fn darf_aufsteigen(&self, c: usize, upper: usize) -> bool {
        let Some(e) = self.erste_von(c) else { return true };
        let el = self.clubs[e].league;
        if el == upper { return false; }
        let (su, se) = (self.stufe_von(upper), self.stufe_von(el));
        if self.leagues[upper].land == self.leagues[el].land && su > 0 && se > 0 && su <= se { return false; }
        let h = self.zweite_hoechste_stufe(self.clubs[c].league);
        h > 0 && (su == 0 || su >= h)
    }
    /// The k clubs that go up, in table order, without the second teams that may not.
    pub fn aufsteiger(&self, order: &[usize], k: usize, upper: usize) -> Vec<usize> { order.iter().copied().filter(|&c| self.darf_aufsteigen(c, upper)).take(k).collect() }
    /// Season end: a second team in the league of its first team (first relegated, or second promoted there) goes one tier
    /// down; its place in that league goes to the next club of the lower league's table that may go up. `tabellen`: the
    /// final order of every league. Returns (second team, league it left, league it joins, club that took its place).
    pub fn zweite_trennen(&mut self, tabellen: &[Vec<usize>]) -> Vec<(usize, usize, usize, usize, bool)> {
        let mut out = vec![];
        for (z, e) in self.zweite.clone() {
            let l = self.clubs[z].league;
            if l != self.clubs[e].league { continue; }
            let key = self.leagues[l].key.clone();
            let gruppen: Vec<usize> = (0..self.leagues.len()).filter(|&u| self.leagues[u].ueber.as_deref() == Some(key.as_str())).collect();
            let Some(&erste) = gruppen.first() else { continue };
            // several regional groups below: the second team's region (ligaregeln.rs region_fuer)
            let (unten, geschaetzt) = self.region_fuer(z, &gruppen, erste);
            let Some(&k) = tabellen[unten].iter().find(|&&c| self.clubs[c].league == unten && self.darf_aufsteigen(c, l)) else { continue };
            self.leagues[l].teams.retain(|&x| x != z);
            self.leagues[unten].teams.retain(|&x| x != k);
            self.leagues[l].teams.push(k);
            self.leagues[unten].teams.push(z);
            self.clubs[z].league = unten;
            self.clubs[k].league = l;
            out.push((z, l, unten, k, geschaetzt));
        }
        out
    }
    /// Who played for the first team this weekend may not play for the second team.
    pub fn spielberechtigt(&self, p: usize, c: usize) -> bool {
        let o = self.players[p].profil.oben;
        self.erste_von(c).is_none() || o == 0 || self.heute() + 1 > o + fz("zweite.sperre_tage") as u32
    }
    /// The country's age rule for second teams: (older than, at most so many).
    pub fn altersregel(&self, c: usize) -> (u8, usize) {
        let land = &self.leagues[self.clubs[c].league].land;
        let v = wert_oder("farm", &format!("zweite.alter.{land}")).unwrap_or_else(|| crate::regeln::wert("farm", "zweite.alter.standard"));
        (v[0].as_u64().unwrap_or(23) as u8, v[1].as_u64().unwrap_or(5) as usize)
    }
    pub fn partner_von(&self, haupt: usize) -> Vec<usize> { self.koops.iter().filter(|k| k.haupt == haupt).map(|k| k.partner).collect() }
    pub fn koop_index(&self, haupt: usize, partner: usize) -> Option<usize> { self.koops.iter().position(|k| k.haupt == haupt && k.partner == partner) }
    /// The type of an AI coach (ki_trainer.toml typ), fixed per club and coach.
    pub fn ki_typ(&self, c: usize) -> &'static str {
        [crate::id("starr"), crate::id("reaktiv"), crate::id("experimentierfreudig"), crate::id("dogmatisch"), crate::id("pragmatisch")][(zufall(&["2026", "ki_typ", &self.clubs[c].id.to_string(), &self.clubs[c].manager]) * 5.0) as usize % 5]
    }
    /// Second teams and the start cooperations of assets/kooperationen.json (by team id, both clubs in the world).
    pub fn farm_einrichten(&mut self) {
        self.zweite = self.zweite_erkennen().0;
        let v: serde_json::Value = serde_json::from_str(&crate::pakete::oder("kooperationen.json", include_str!("../assets/kooperationen.json"))).expect("kooperationen.json");
        let idx = |id: u64| self.clubs.iter().position(|c| c.id as u64 == id);
        let mut k = vec![];
        for p in v["paare"].as_array().unwrap() {
            if let (Some(h), Some(pa)) = (idx(p["haupt"].as_u64().unwrap_or(0)), idx(p["partner"].as_u64().unwrap_or(0))) {
                k.push(Koop { haupt: h, partner: pa, bis: self.season + fz("koop.jahre") as u32 - 1, betrag: fz("koop.betrag_basis") as f64, plaetze: fz("koop.plaetze") as u8, min_leihen: 0, bleiben: false, verstoesse: 0 });
            }
        }
        self.koops = k;
        self.farm_an = true;
    }
    /// Move a player between two clubs of one organisation (no fee, no talks): squads, club and line-ups follow.
    pub fn umziehen(&mut self, p: usize, von: usize, nach: usize) {
        self.clubs[von].squad.retain(|&x| x != p);
        if !self.clubs[nach].squad.contains(&p) { self.clubs[nach].squad.push(p); }
        self.players[p].team = nach as i32;
        self.players[p].youth = 0;
    }
    /// AI clubs use their second team the same way (monthly): surplus young players down, the best young ones up.
    pub fn ki_zweite_monat(&mut self, own: usize) {
        for (z, e) in self.zweite.clone() {
            if e == own || z == own { continue; }
            let ziel = fz("zweite.ki_erste_ziel") as usize;
            let jung = fz("zweite.ki_alter_hoechstens") as u8;
            let mut erste: Vec<usize> = self.clubs[e].squad.iter().copied().filter(|&p| self.players[p].youth == 0 && self.players[p].leihe.is_none()).collect();
            erste.sort_by(|a, b| self.players[*b].ovr.partial_cmp(&self.players[*a].ovr).unwrap());
            for &p in erste.iter().skip(ziel) { if self.players[p].age <= jung && self.clubs[z].squad.len() < fz("zweite.zweite_max") as usize { self.umziehen(p, e, z); } }
            // the best young player below who beats the weakest regular goes up
            let schwach = erste.iter().take(18).map(|&p| self.players[p].ovr).fold(f32::MAX, f32::min);
            let best = self.clubs[z].squad.iter().copied().filter(|&p| self.players[p].age <= jung && self.players[p].leihe.is_none()).max_by(|a, b| self.players[*a].ovr.partial_cmp(&self.players[*b].ovr).unwrap());
            if let Some(b) = best { if self.players[b].ovr > schwach && self.clubs[e].squad.len() < fz("zweite.erste_max") as usize { self.umziehen(b, z, e); } }
        }
    }
    /// The loan reason in football words.
    pub fn leihgrund(&self, p: usize, partner: usize) -> String {
        let pl = &self.players[p];
        let ort = self.clubs[partner].name.split_whitespace().filter(|w| !klubwort(&falten(w))).collect::<Vec<_>>().join(" ");
        let k = if pl.inj_name.is_empty() == false && pl.fit < 80.0 { crate::id("verletzt") } else if pl.age <= 21 { crate::id("jung") } else if pl.benched >= 3 { crate::id("bank") } else { crate::id("sonst") };
        tx(&format!("leihgrund.{k}")).replace("%1", &ort)
    }
    /// Loan a player of `haupt` to its partner (fee-free, for the season); AI and person use this.
    pub fn an_partner(&mut self, p: usize, haupt: usize, partner: usize, spielzeit: bool) {
        let grund = self.leihgrund(p, partner);
        let season = self.season;
        self.umziehen(p, haupt, partner);
        self.players[p].leihe = Some(crate::transfer::Leihe { von: haupt, bis: season, anteil: crate::transfer::LEIH_ANTEIL, option: 0.0, spielzeit, grund, start: 0.0, bis_tag: 0, pflicht: false });
    }
}

impl Career {
    pub fn zweite_mannschaft(&self) -> Option<usize> { self.world.zweite_von(self.own) }
    /// Send a player between first and second team, at any time, without fee or talks. Limits: squad sizes and the
    /// country's age rule for the second team. Who played up this weekend stays eligible only for the first team.
    pub fn mannschaft_wechseln(&mut self, p: usize) -> Result<String, String> {
        let z = self.zweite_mannschaft().ok_or(crate::inhalt::t("ui.farm.der_verein_hat_keine_zweite"))?;
        let pl = &self.world.players[p];
        if pl.leihe.as_ref().is_some_and(|l| l.von != self.own) { return Err(crate::inhalt::t("ui.farm.leihspieler_bleiben_in_der_ersten").into()); }
        let (von, nach) = if pl.team == self.own as i32 { (self.own, z) } else if pl.team == z as i32 { (z, self.own) } else { return Err(crate::inhalt::t("ui.farm.er_spielt_in_keiner_unserer").into()) };
        let profis = |c: usize| self.world.clubs[c].squad.iter().filter(|&&q| self.world.players[q].youth == 0).count();
        let (max_n, min_v) = if nach == z { (fz("zweite.zweite_max"), fz("zweite.erste_min")) } else { (fz("zweite.erste_max"), fz("zweite.zweite_min")) };
        if profis(nach) >= max_n as usize { return Err(crate::tf!("ui.farm.hat_schon_spieler_mehr_gehen", self.world.clubs[nach].name, max_n)); }
        if profis(von) <= min_v as usize { return Err(crate::tf!("ui.farm.braucht_mindestens_spieler", self.world.clubs[von].name, min_v)); }
        if nach == z {
            let (ueber, hoechstens) = self.world.altersregel(z);
            let alte = self.world.clubs[z].squad.iter().filter(|&&q| self.world.players[q].age > ueber).count();
            if pl.age > ueber && alte >= hoechstens { return Err(crate::tf!("ui.farm.in_der_zweiten_mannschaft_duerfen", hoechstens, ueber)); }
        }
        let n = pl.name.clone();
        self.world.umziehen(p, von, nach);
        self.xi.retain(|&x| x != p);
        self.world.refresh_levels();
        Ok(if nach == z { crate::tf!("ui.farm.spielt_jetzt_in_der_zweiten", n) } else { crate::tf!("ui.farm.gehoert_jetzt_zur_ersten_mannschaft", n) })
    }
    /// Once a week in the window (Fabian 24.11.: the second team runs its own transfers): when our second team has
    /// fewer pros than it needs, its coach signs young free agents of its level himself and tells us who came.
    pub fn zweite_markt_woche(&mut self) {
        let Some(z) = self.zweite_mannschaft() else { return };
        if self.in_window().is_none() { return; }
        let soll = fz("zweite.zweite_min") as usize + 2;
        let profis = self.world.clubs[z].squad.iter().filter(|&&q| self.world.players[q].youth == 0).count();
        if profis >= soll { return; }
        let (ueber, _) = self.world.altersregel(z);
        let lvl = self.world.clubs[z].level;
        let mut frei: Vec<usize> = (0..self.world.players.len()).filter(|&p| { let pl = &self.world.players[p]; pl.team == -1 && pl.age <= ueber && pl.ovr <= lvl + 4.0 && pl.ovr >= lvl - 10.0 }).collect();
        frei.sort_by(|a, b| { let k = |p: usize| self.world.players[p].ovr + (self.world.players[p].pot - self.world.players[p].ovr) * 0.5; k(*b).partial_cmp(&k(*a)).unwrap() });
        let (abs, season, liga) = (self.abs(), self.world.season, self.world.leagues[self.world.clubs[z].league].level);
        let mut neu = vec![];
        for p in frei.into_iter().take(soll - profis) {
            let pl = &mut self.world.players[p];
            pl.team = z as i32;
            pl.joined = abs;
            pl.contract_end = season + crate::world::contract_years(pl.age);
            pl.wage = crate::world::fair_wage(pl, liga);
            self.world.clubs[z].squad.push(p);
            neu.push(format!("{} ({}, {}, {:.0})", self.world.players[p].name, crate::world::pos_de(&self.world.players[p].pos), self.world.players[p].age, self.world.players[p].ovr));
        }
        if neu.is_empty() { return; }
        let zn = self.world.clubs[z].name.clone();
        self.msg(2, &zn, &crate::tf!("ui.farm.zweite_holt_titel", zn.clone(), neu.len()), &crate::tf!("ui.farm.zweite_holt", zn, neu.join(crate::id(", "))), Action::None);
    }
    /// The academy hands its graduates to the second team instead of an invented U23.
    pub fn u23_an_zweite(&mut self) {
        let Some(z) = self.zweite_mannschaft() else { return };
        for p in self.world.clubs[self.own].squad.clone() {
            if self.world.players[p].youth == 19 { self.world.umziehen(p, self.own, z); }
        }
    }
    fn eigene_koops(&self) -> Vec<usize> { (0..self.world.koops.len()).filter(|&i| self.world.koops[i].haupt == self.own).collect() }
    /// Loans currently at the partner.
    pub fn leihen_bei(&self, partner: usize) -> Vec<usize> {
        self.world.clubs[partner].squad.iter().copied().filter(|&p| self.world.players[p].leihe.as_ref().is_some_and(|l| l.von == self.own)).collect()
    }
    /// Loan to a partner: faster, without fee, the partner nearly always says yes (koop.annahme), within the places.
    pub fn leihe_an_partner(&mut self, p: usize, partner: usize, spielzeit: bool) -> Result<String, String> {
        if self.in_window().is_none() { return Err(crate::inhalt::t("ui.farm.das_transferfenster_ist_geschlossen").into()); }
        let ki = self.world.koop_index(self.own, partner).ok_or(crate::inhalt::t("ui.farm.mit_diesem_verein_gibt_es"))?;
        let pl = &self.world.players[p];
        if pl.team != self.own as i32 && Some(pl.team as usize) != self.zweite_mannschaft() { return Err(crate::inhalt::t("ui.farm.das_geht_nur_mit_eigenen").into()); }
        if pl.leihe.is_some() { return Err(crate::inhalt::t("ui.farm.er_ist_schon_verliehen").into()); }
        if self.leihen_bei(partner).len() >= self.world.koops[ki].plaetze as usize { return Err(crate::tf!("ui.farm.alle_leihplaetze_bei_sind_belegt", self.world.koops[ki].plaetze, self.world.clubs[partner].name)); }
        let id = pl.id.to_string();
        if zufall(&["2026", crate::id("partnerleihe"), &id, &self.abs().to_string()]) >= fz("koop.annahme") { return Err(crate::tf!("ui.farm.hat_fuer_ihn_gerade_keinen", self.world.clubs[partner].name)); }
        let von = pl.team as usize;
        self.world.an_partner(p, von, partner, spielzeit);
        self.world.players[p].leihe.as_mut().unwrap().von = self.own;
        self.xi.retain(|&x| x != p);
        self.world.refresh_levels();
        let (n, g) = (self.world.players[p].name.clone(), self.world.players[p].leihe.as_ref().unwrap().grund.clone());
        self.news(&crate::tf!("ui.farm.wechselt_auf_leihbasis_zu", n, self.world.clubs[partner].name), &g);
        Ok(crate::tf!("ui.farm.ist_bis_saisonende_verliehen", n, g))
    }
    /// Call a player back from the partner. With the promise "loan players stay" this breaks the contract.
    pub fn leihe_zurueckrufen(&mut self, p: usize) -> Result<String, String> {
        let l = self.world.players[p].leihe.clone().ok_or(crate::inhalt::t("ui.farm.er_ist_nicht_verliehen"))?;
        let t = self.world.players[p].team;
        if l.von != self.own || t < 0 { return Err(crate::inhalt::t("ui.farm.das_geht_nur_mit_eigenen_2").into()); }
        let partner = t as usize;
        let ki = self.world.koop_index(self.own, partner).ok_or(crate::inhalt::t("ui.farm.zurueckrufen_geht_nur_beim_partnerverein"))?;
        self.world.umziehen(p, partner, self.own);
        self.world.players[p].leihe = None;
        self.world.refresh_levels();
        let n = self.world.players[p].name.clone();
        if self.world.koops[ki].bleiben { self.koop_ende(ki, crate::id("zurueck")); }
        Ok(crate::tf!("ui.farm.ist_zurueck", n))
    }
    /// The partner ends the cooperation (reason key of farm.json kuendigung).
    pub fn koop_ende(&mut self, ki: usize, grund: &str) {
        let k = self.world.koops.remove(ki);
        let pn = self.world.clubs[k.partner].name.clone();
        let n = self.leihen_bei(k.partner).len();
        let t = tx(&format!("kuendigung.{grund}")).replace("%1", &pn).replace("%2", &k.min_leihen.to_string()).replace("%3", &n.to_string());
        self.msg(5, &pn, &crate::tf!("ui.farm.beendet_die_kooperation", pn), &t, Action::None);
        self.news(&crate::tf!("tb.ereignis.entlassen", pn, self.club().name), &t);
    }
    pub fn koop_schliessen(&mut self, partner: usize, betrag: f64, min_leihen: u8, bleiben: bool, plaetze: u8) -> Result<String, String> {
        if self.eigene_koops().len() >= fz("koop.hoechstens") as usize { return Err(crate::tf!("ui.farm.mehr_als_farmteams_gleichzeitig_gehen", fz("koop.hoechstens"))); }
        if self.world.koop_index(self.own, partner).is_some() { return Err(crate::inhalt::t("ui.farm.mit_diesem_verein_gibt_es_2").into()); }
        let bis = self.world.season + fz("koop.jahre") as u32 - 1;
        self.world.koops.push(Koop { haupt: self.own, partner, bis, betrag, plaetze, min_leihen, bleiben, verstoesse: 0 });
        let pn = self.world.clubs[partner].name.clone();
        self.news(&crate::tf!("ui.farm.und_arbeiten_zusammen", self.club().name, pn), &crate::tf!("ui.farm.wird_farmteam_vertrag_bis", pn, date::season_label(bis)));
        Ok(crate::tf!("ui.farm.die_kooperation_mit_steht_im", pn, money(betrag), date::season_label(bis)))
    }
    pub fn koop_verlaengern(&mut self, ki: usize) { self.world.koops[ki].bis += 1; }
    /// We end the cooperation: the loan players stay until the season ends.
    pub fn koop_kuendigen(&mut self, ki: usize) -> String { let k = self.world.koops.remove(ki); crate::tf!("ui.farm.die_kooperation_mit_ist_beendet", self.world.clubs[k.partner].name) }
    /// The frame the board allows without asking (koop.rahmen_anteil of the yearly revenue).
    pub fn koop_rahmen(&self) -> f64 { self.world.revenue(self.own) * fz("koop.rahmen_anteil") as f64 }
    /// Start the 14-day search. Above the frame the board must approve (trust koop.vorstand_freigabe).
    pub fn koop_suche_starten(&mut self, mut s: KoopSuche) -> Result<String, String> {
        if self.eigene_koops().len() >= fz("koop.hoechstens") as usize { return Err(crate::tf!("ui.farm.mehr_als_farmteams_gleichzeitig_gehen", fz("koop.hoechstens"))); }
        if self.koop_suche.is_some() { return Err(crate::inhalt::t("ui.farm.die_suche_laeuft_schon").into()); }
        let mut t = String::new();
        if s.max_betrag > self.koop_rahmen() {
            if self.confidence() < fz("koop.vorstand_freigabe") { return Err(crate::tf!("ui.farm.das_praesidium_gibt_mehr_als", money(self.koop_rahmen()))); }
            t = crate::inhalt::t("ui.farm.das_praesidium_gibt_das_hoehere").into();
        }
        s.bis = self.abs() + fz("koop.suche_tage") as u32;
        self.koop_suche = Some(s);
        Ok(crate::tf!("ui.farm.die_suche_laeuft_14_tage", t))
    }
    /// How much a club wants to be our farm team (0 = not at all), and the sentence why.
    pub fn koop_interesse(&self, c: usize) -> (f32, String, String) {
        let i = |k: &str| fz(&format!("koop.interesse.{k}"));
        let (own, cl) = (&self.world.clubs[self.own], &self.world.clubs[c]);
        let abstand = own.level - cl.level;
        if abstand <= 0.0 || abstand >= i("abstand_zu_gross") { return (0.0, String::new(), String::new()); }
        let mut w = if abstand >= i("abstand_von") && abstand <= i("abstand_bis") { 1.0 } else { 0.5 };
        let klamm = cl.balance < self.world.revenue(c) * 0.05;
        if klamm { w *= i("klamm_faktor"); }
        if self.world.koops.iter().any(|k| k.partner == c) { w *= i("fremde_koop"); }
        let land = |x: usize| self.world.leagues[self.world.clubs[x].league].land.clone();
        if land(c) != land(self.own) { w *= i("anderes_land"); }
        let trainer = self.world.ki_typ(c) == crate::id("experimentierfreudig");
        if trainer { w *= i("junger_trainer"); }
        // squad gap: the group with the fewest players
        let gruppe = |pos: &str| match pos { "GK" => "tor", "CB" => "abwehr", "LB" | "RB" => "aussen", "DMF" | "CMF" | "AMF" => "mitte", "LMF" | "RMF" | "LWF" | "RWF" => "fluegel", _ => crate::id("sturm") };
        let soll = [(crate::id("tor"), 3), (crate::id("abwehr"), 4), (crate::id("aussen"), 4), (crate::id("mitte"), 6), (crate::id("fluegel"), 4), (crate::id("sturm"), 3)];
        let luecke = soll.iter().map(|(g, n)| (*g, *n as i32 - cl.squad.iter().filter(|&&p| gruppe(&self.world.players[p].pos) == *g).count() as i32)).max_by_key(|x| x.1).unwrap();
        if luecke.1 > 0 { w *= i(crate::id("luecke")); }
        let mut satz = if luecke.1 > 0 { tx(&format!("luecke.{}", luecke.0)) } else { tx("grund.abstand") };
        if klamm { satz = crate::tf!("ui.farm.text", satz, tx("grund.klamm")); } else if trainer { satz = crate::tf!("ui.farm.text", satz, tx("grund.trainer")); }
        (w, satz, luecke.0.to_string())
    }
    /// The demand of an applicant: yearly amount (more when short of money), minimum loan players, the play-time promise.
    fn koop_forderung(&self, c: usize, plaetze: u8) -> (f64, u8, bool) {
        let cl = &self.world.clubs[c];
        let klamm = cl.balance < self.world.revenue(c) * 0.05;
        let z = |k: &str| zufall(&["2026", crate::id("koopforderung"), &cl.id.to_string(), &self.world.season.to_string(), k]) as f64;
        let betrag = (fz("koop.betrag_basis") as f64 * self.skala() * (0.8 + 0.5 * z(crate::id("b"))) * if klamm { fz("koop.interesse.klamm_faktor") as f64 } else { 1.0 } / 25000.0).round() * 25000.0;
        let min = (1.0 + z(crate::id("n")) * 2.0).floor() as u8;
        (betrag, min.min(plaetze.max(1)), z(crate::id("s")) < 0.5)
    }
    fn kandidat(&self, c: usize, s: Option<&KoopSuche>) -> bool {
        let own_l = &self.world.leagues[self.league()];
        let l = &self.world.leagues[self.world.clubs[c].league];
        if c == self.own || Some(c) == self.zweite_mannschaft() || self.world.erste_von(c).is_some() || self.world.koop_index(self.own, c).is_some() { return false; }
        if self.koop_angebote.iter().any(|a| a.club == c) { return false; }
        match s {
            Some(s) => (!s.nur_land || l.land == own_l.land) && l.stufe >= own_l.stufe + s.stufe_von && l.stufe <= own_l.stufe + s.stufe_bis,
            None => l.land == own_l.land && l.stufe > own_l.stufe,
        }
    }
    fn angebot_aufnehmen(&mut self, c: usize, plaetze: u8, satz: String, von_sich_aus: bool) {
        let (betrag, min_leihen, bleiben) = self.koop_forderung(c, plaetze);
        let id = self.koop_angebote.iter().map(|a| a.id).max().unwrap_or(0) + 1 + self.abs();
        let frist = self.abs() + fz("koop.frist_tage") as u32;
        let cn = self.world.clubs[c].name.clone();
        let lg = self.world.leagues[self.world.clubs[c].league].name.clone();
        self.koop_angebote.push(KoopAngebot { id, club: c, betrag, min_leihen, bleiben, grund: satz.clone(), frist, plaetze, phase: None, antwort: String::new() });
        let titel = tx(if von_sich_aus { "anfrage_titel" } else { "bewerbung_titel" }).replace("%1", &cn);
        let body = crate::tf!("ui.farm.staerke_verlangt_im_jahr_mindestens", cn, lg, format!("{:.0}", self.world.clubs[c].level), money(betrag), min_leihen, if bleiben { crate::inhalt::t("ui.farm.und_die_zusage_dass_sie") } else { "" }, satz, date::long(self.world.season, self.world.day + fz("koop.frist_tage") as u32));
        let opts = vec![(crate::id("annehmen").to_string(), crate::inhalt::t("lb.aktion.annehmen").to_string()), (crate::id("nachverhandeln").to_string(), crate::inhalt::t("ui.farm.nachverhandeln").to_string()), (crate::id("ablehnen").to_string(), crate::inhalt::t("as.ablehnen").to_string())];
        self.msg(5, &cn, &titel, &body, Action::Wahl { art: format!("koop:{id}"), bezug: vec![], optionen: opts });
        if let Some(m) = self.inbox.first_mut() { m.frist = frist; }
    }
    /// Answer to an application: accept (contract), renegotiate (round model of the transfers, at most koop.runden rounds) or decline.
    pub fn koop_antwort(&mut self, id: u32, key: &str, wert: Option<f64>) -> String {
        let Some(i) = self.koop_angebote.iter().position(|a| a.id == id) else { return crate::inhalt::t("ui.farm.die_bewerbung_ist_nicht_mehr").into() };
        let a = self.koop_angebote[i].clone();
        let cn = self.world.clubs[a.club].name.clone();
        match key {
            "annehmen" => match self.koop_schliessen(a.club, a.betrag, a.min_leihen, a.bleiben, a.plaetze) {
                Ok(t) => { self.koop_angebote.remove(i); self.erledigen(id); t }
                Err(e) => e,
            },
            "ablehnen" => {
                self.koop_angebote.remove(i);
                self.erledigen(id);
                self.vereinsbeziehung(a.club, fz("koop.ablehnen_beziehung"));
                crate::tf!("ui.farm.ist_enttaeuscht_nimmt_es_aber", cn)
            }
            _ => {
                let mut ph = a.phase.clone().unwrap_or_else(|| crate::verhandlung::Phase::neu(a.betrag, if self.world.clubs[a.club].balance < 0.0 { crate::id("hardliner") } else { crate::id("pragmatiker") }, 0.0, -0.5));
                let angebot = wert.unwrap_or((ph.gegen.unwrap_or(a.betrag) * 0.85 / 25000.0).round() * 25000.0);
                let s = format!("{id}|{}", ph.runde);
                let w = |k: u32| zufall(&["2026", crate::id("koopverh"), &s, &k.to_string()]);
                let (r, _) = ph.senden(angebot, false, 0, 1, 0.0, &w);
                use crate::verhandlung::Reaktion::*;
                let t = match r {
                    Annehmen(x) => { self.koop_angebote[i].betrag = x; return self.koop_antwort(id, crate::id("annehmen"), None); }
                    Gegen(g) | LetztesWort(g) => { self.koop_angebote[i].betrag = (g / 25000.0).round() * 25000.0; crate::tf!("ui.farm.kommt_entgegen_im_jahr", cn, money(self.koop_angebote[i].betrag)) }
                    Bedenkzeit => crate::tf!("ui.farm.denkt_ueber_nach", cn, money(angebot)),
                    Ablehnen(_) => crate::tf!("ui.farm.lehnt_ab", cn, money(angebot)),
                    Abbruch(_) => { self.koop_angebote.remove(i); self.erledigen(id); return crate::tf!("gs.ui.partner_bricht_ab", cn); }
                };
                if ph.runde >= fz("koop.runden") as u8 { self.koop_angebote.remove(i); self.erledigen(id); return crate::tf!("ui.farm.mehr_runden_gibt_es_nicht", t, cn); }
                self.koop_angebote[i].phase = Some(ph);
                self.koop_angebote[i].antwort = t.clone();
                t
            }
        }
    }
    fn erledigen(&mut self, id: u32) { let art = format!("koop:{id}"); for m in self.inbox.iter_mut() { if matches!(&m.action, Action::Wahl { art: a, .. } if *a == art) { m.done = true; m.read = true; } } }
    /// Daily: applications during the search, deadlines, spontaneous requests, payments and promises, AI loans.
    pub fn koop_tag(&mut self) {
        let abs = self.abs();
        self.koop_angebote.retain(|a| a.frist >= abs);
        if let Some(s) = self.koop_suche.clone() {
            let laufend = self.koop_angebote.len() < fz("koop.bewerbungen_hoechstens") as usize;
            // clubs that would ask for much more than the person's maximum do not apply (0: no maximum)
            let zu_teuer = |c: usize| s.max_betrag > 0.0 && self.koop_forderung(c, s.plaetze).0 > s.max_betrag * 1.25;
            let mut kand: Vec<(f32, usize, String)> = (0..self.world.clubs.len()).filter(|&c| self.kandidat(c, Some(&s)) && !zu_teuer(c)).map(|c| { let (w, satz, _) = self.koop_interesse(c); (w, c, satz) }).filter(|x| x.0 > 0.0).collect();
            kand.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.cmp(&b.1)));
            let mut neu = 0;
            if laufend {
                for (w, c, satz) in kand.iter().take(40) {
                    if zufall(&["2026", crate::id("koopbewerbung"), &self.world.clubs[*c].id.to_string(), &abs.to_string()]) < fz("koop.bewerbung_chance") * w && self.koop_angebote.len() < fz("koop.bewerbungen_hoechstens") as usize {
                        self.angebot_aufnehmen(*c, s.plaetze, satz.clone(), false);
                        neu += 1;
                    }
                }
            }
            // the search always brings an application when suitable clubs exist: on the last day the most interested applies
            if abs >= s.bis {
                if neu == 0 && self.koop_angebote.is_empty() { if let Some((_, c, satz)) = kand.first().cloned() { self.angebot_aufnehmen(c, s.plaetze, satz, false); } }
                self.koop_suche = None;
            }
        }
        // small clubs ask by themselves (rarer), weekly
        if abs % 7 == 0 && self.eigene_koops().len() < fz("koop.hoechstens") as usize && self.koop_suche.is_none() && zufall(&["2026", crate::id("koopanfrage"), &abs.to_string()]) < fz("koop.anfrage_chance") {
            let kand: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| self.kandidat(c, None) && self.koop_interesse(c).0 > 0.8).collect();
            if !kand.is_empty() {
                let c = kand[(zufall(&["2026", "koopanfrage_wer", &abs.to_string()]) * kand.len() as f32) as usize % kand.len()];
                let satz = self.koop_interesse(c).1;
                self.angebot_aufnehmen(c, fz("koop.plaetze") as u8, satz, true);
            }
        }
        // monthly payment; promised loan players after each window
        let day = self.world.day;
        if day % 30 == 29 {
            for ki in self.eigene_koops().into_iter().rev() {
                let b = self.world.koops[ki].betrag / 12.0;
                let p = self.world.koops[ki].partner;
                if self.club().balance < 0.0 { self.world.koops[ki].verstoesse += 1; if self.world.koops[ki].verstoesse as f32 >= fz("koop.zahlung_verstoesse") { self.koop_ende(ki, crate::id("zahlung")); continue; } }
                else { self.book(crate::inhalt::t("ui.farm.kooperationen"), -b); self.world.clubs[p].balance += b; }
            }
        }
        if WINDOWS.iter().any(|w| day == w.1 + 1) {
            for ki in self.eigene_koops().into_iter().rev() {
                if (self.leihen_bei(self.world.koops[ki].partner).len() as u8) < self.world.koops[ki].min_leihen { self.koop_ende(ki, crate::id("wenig")); }
            }
        }
        // AI clubs with a cooperation loan to their partner first (a few days into each window)
        if WINDOWS.iter().any(|w| day == w.0 + 3) {
            for k in self.world.koops.clone() {
                if k.haupt == self.own || k.partner == self.own { continue; }
                let mut sq: Vec<usize> = self.world.clubs[k.haupt].squad.iter().copied().filter(|&p| self.world.players[p].youth == 0 && self.world.players[p].leihe.is_none()).collect();
                sq.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
                let n = fz("koop.ki_leihen") as usize;
                let weg: Vec<usize> = sq.into_iter().skip(20).filter(|&p| self.world.players[p].age <= 21).take(n).collect();
                for p in weg { self.world.an_partner(p, k.haupt, k.partner, false); }
            }
        }
        if day % 30 == 29 { self.world.ki_zweite_monat(self.own); }
    }
    /// Season end: contracts run out (AI clubs extend some), AI clubs find partners.
    pub fn koop_saison(&mut self) {
        let season = self.world.season;
        for ki in (0..self.world.koops.len()).rev() {
            let k = self.world.koops[ki].clone();
            if k.bis >= season { continue; }
            if k.haupt == self.own { let pn = self.world.clubs[k.partner].name.clone(); self.world.koops.remove(ki); self.msg(5, &pn, &crate::tf!("ui.farm.kooperation_mit_ausgelaufen", pn), &tx("kuendigung.ablauf").replace("%1", &pn), Action::None); }
            else if zufall(&["2026", crate::id("kiverl"), &ki.to_string(), &season.to_string()]) < fz("koop.ki_verlaengern") { self.world.koops[ki].bis += fz("koop.jahre") as u32; }
            else { self.world.koops.remove(ki); }
        }
        for c in 0..self.world.clubs.len() {
            if c == self.own || self.world.clubs[c].level < fz("koop.ki_ab_staerke") || !self.world.partner_von(c).is_empty() || self.world.erste_von(c).is_some() { continue; }
            if zufall(&["2026", crate::id("kikoop"), &self.world.clubs[c].id.to_string(), &season.to_string()]) >= fz("koop.ki_chance") { continue; }
            let (land, st) = (self.world.leagues[self.world.clubs[c].league].land.clone(), self.world.leagues[self.world.clubs[c].league].stufe);
            let p = (0..self.world.clubs.len()).filter(|&o| o != self.own && o != c && self.world.erste_von(o).is_none() && !self.world.koops.iter().any(|k| k.partner == o) && { let l = &self.world.leagues[self.world.clubs[o].league]; l.land == land && l.stufe > st && l.stufe <= st + 2 })
                .max_by(|a, b| self.world.clubs[*a].level.partial_cmp(&self.world.clubs[*b].level).unwrap());
            if let Some(p) = p { self.world.koops.push(Koop { haupt: c, partner: p, bis: season + fz("koop.jahre") as u32, betrag: fz("koop.betrag_basis") as f64, plaetze: fz("koop.plaetze") as u8, min_leihen: 0, bleiben: false, verstoesse: 0 }); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fl27;
    fn welt(pfad: &str) -> Option<(std::path::PathBuf, World)> {
        let w = std::path::PathBuf::from(pfad);
        if !fl27::echte_welt(&w) { return None; }
        let p = w.join("common/etc/pesdb");
        Some((p.clone(), World::from_pesdb(&p, 3).unwrap()))
    }
    /// Fabian 24.11.: our second team fills a thin squad itself; our partner club asks about every loan-list player.
    #[test]
    fn zweite_kauft_selbst_und_partner_fragt() {
        let mut w = crate::world::testwelt(4);
        let own = 2;
        let z = (0..w.clubs.len()).find(|&c| c != own && w.clubs[c].level < w.clubs[own].level - 8.0).unwrap();
        w.zweite.push((z, own));
        let mut c = Career::new(w, own, "Test");
        c.world.day = 3;
        for p in c.world.clubs[z].squad.split_off(10) { c.world.players[p].team = -1; }
        c.zweite_markt_woche();
        let profis = c.world.clubs[z].squad.iter().filter(|&&q| c.world.players[q].youth == 0).count();
        assert!(profis > 10, "the second team signed players itself");
        assert!(c.inbox.iter().any(|m| m.from == c.world.clubs[z].name), "and told us");
        // a partner club asks about a loan-list player, every time a fitting one is listed
        let partner = (0..c.world.clubs.len()).find(|&k| k != own && k != z && c.world.clubs[k].level < c.world.clubs[own].level - 3.0 && c.world.clubs[k].squad.len() < 30).unwrap();
        c.world.koops.push(Koop { haupt: own, partner, bis: c.world.season + 2, betrag: 0.0, plaetze: 3, min_leihen: 0, bleiben: false, verstoesse: 0 });
        let p = *c.squad().iter().find(|&&p| c.world.players[p].ovr >= c.world.clubs[partner].level - 5.0 && c.world.players[p].leihe.is_none()).unwrap();
        c.leihe_abgeben(p).unwrap();
        c.leihangebote_tag();
        let m = c.inbox.iter().find(|m| matches!(&m.action, Action::Wahl { art, bezug, .. } if bezug == &vec![p] && art.starts_with(&format!("leihe:{partner}:0:")))).expect("the partner asks, without a fee");
        assert!(m.body.contains(&c.world.clubs[partner].name));
        let n = c.inbox.len();
        c.leihangebote_tag();
        assert!(c.inbox.iter().filter(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with(&format!("leihe:{partner}:")))).count() == 1, "not again while the offer is open ({n})");
    }
    const MEGA: &str = "/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk/_FL26Megapack";
    const MY: &str = "/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk/_FL26MyWorld";
    fn club(w: &World, n: &str) -> usize { w.clubs.iter().position(|c| c.name == n).unwrap_or_else(|| panic!("{n} fehlt")) }

    #[test]
    fn zweite_mannschaften_erkennen() {
        if let Some((_, w)) = welt(MEGA) {
            assert_eq!(w.erste_von(club(&w, "VfB Stuttgart II")), Some(club(&w, "VfB Stuttgart")));
            assert_eq!(w.erste_von(club(&w, "FC Bayern München II")), Some(club(&w, "Bayern München")), "club words left out");
            assert_eq!(w.erste_von(club(&w, "Jong Ajax")), Some(club(&w, "Ajax Amsterdam")), "prefix Jong, name start");
            let willem = club(&w, "Willem II Tilburg");
            assert!(w.erste_von(willem).is_none() && w.zweite_von(willem).is_none(), "Willem II Tilburg is no second team");
            if let Some(c) = w.clubs.iter().position(|c| c.name == "Real Madrid Castilla") { assert!(w.erste_von(c).is_some(), "from zweite_mannschaften.json"); }
            assert!(w.koop_index(club(&w, "Bayern München"), club(&w, "SpVgg Unterhaching")).is_some(), "start cooperation by team id");
        }
        if let Some((_, w)) = welt(MY) { assert_eq!(w.erste_von(club(&w, "FC Porto B")), Some(club(&w, "FC Porto"))); }
    }
    #[test]
    fn hin_und_her_in_player_assignment() {
        let Some((pesdb, w)) = welt(MY) else { return };
        let (porto, b) = (club(&w, "FC Porto"), club(&w, "FC Porto B"));
        let mut c = Career::new(w, porto, "Test");
        let p = *c.world.clubs[porto].squad.iter().filter(|&&p| c.world.players[p].age <= 22 && c.world.players[p].youth == 0).last().expect("young player");
        let known: std::collections::HashSet<u32> = crate::pesdb::PlayerBin::load(&pesdb.join("Player.bin")).unwrap().index.keys().copied().collect();
        let (_, mut a) = fl27::asg_laden(&pesdb).unwrap();
        asg_abgleichen(&c.world, &mut a, &known, &[]).unwrap();
        let id = c.world.players[p].id;
        let team_von = |a: &Vec<fl27::Asg>| a.iter().filter(|x| x.pid == id).map(|x| x.team).collect::<Vec<_>>();
        let summe = c.world.wage_bill(porto);
        c.mannschaft_wechseln(p).unwrap();
        assert_eq!(c.world.players[p].team, b as i32);
        assert!((c.world.wage_bill(porto) - summe).abs() < 1.0 && c.world.wage_bill(b) == 0.0, "no budget of its own: a player costs the same up and down");
        assert!(c.gehoert_uns(p), "the second team is ours");
        assert!(asg_abgleichen(&c.world, &mut a, &known, &[]).unwrap() >= 1);
        assert_eq!(team_von(&a), vec![c.world.clubs[b].id], "down: one entry at the second team");
        c.mannschaft_wechseln(p).unwrap();
        asg_abgleichen(&c.world, &mut a, &known, &[]).unwrap();
        assert_eq!(team_von(&a), vec![c.world.clubs[porto].id], "and back up");
        // injured before a match: out of the FL27 squad, fit again: back with his shirt number (Fabian 23.11.)
        let nr = a.iter().find(|x| x.pid == id).unwrap().nummer();
        c.world.players[p].inj = 5;
        asg_abgleichen(&c.world, &mut a, &known, &[porto]).unwrap();
        assert!(team_von(&a).is_empty(), "injured: not in the squad");
        c.world.players[p].inj = 0;
        asg_abgleichen(&c.world, &mut a, &known, &[porto]).unwrap();
        assert_eq!(team_von(&a), vec![c.world.clubs[porto].id]);
        assert_eq!(a.iter().find(|x| x.pid == id).unwrap().nummer(), nr, "same shirt number");
        // a number changed in CalcioM reaches PlayerAssignment.bin (Fabian 24.11.)
        let t = c.world.clubs[porto].id;
        let frei = (1..=99u8).find(|&n| !a.iter().any(|x| x.team == t && x.nummer() == n as u32)).unwrap();
        c.world.players[p].no = frei;
        asg_abgleichen(&c.world, &mut a, &known, &[porto]).unwrap();
        assert_eq!(a.iter().find(|x| x.pid == id).unwrap().nummer(), frei as u32, "new shirt number in FL27");
        // who played up this weekend may not play below
        c.world.players[p].profil.oben = c.world.heute() + 1;
        assert!(!c.world.spielberechtigt(p, b) && c.world.spielberechtigt(p, porto));
        // the second team runs through our books as its own post
        c.mensch.vertrag_bis = u32::MAX;
        c.mannschaft_wechseln(p).unwrap();
        for _ in 0..31 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } c.ultimatum = None; c.end_day().unwrap(); }
        assert!(c.book.iter().any(|x| x.0 == "Zweite Mannschaft" && x.1 != 0.0), "post Zweite Mannschaft in the finances {:?} {}", c.book, c.world.day);
    }
    #[test]
    fn zweite_nach_abstieg_der_ersten_eine_stufe_tiefer() {
        let Some((_, mut w)) = welt(MEGA) else { return };
        let (vfb, ii) = (club(&w, "VfB Stuttgart"), club(&w, "VfB Stuttgart II"));
        let (oben, drei) = (w.clubs[vfb].league, w.clubs[ii].league);
        // the first team is relegated into the league of its second team
        let x = *w.leagues[drei].teams.iter().find(|&&c| c != ii).unwrap();
        w.leagues[oben].teams.retain(|&c| c != vfb);
        w.leagues[drei].teams.retain(|&c| c != x);
        w.leagues[oben].teams.push(x);
        w.leagues[drei].teams.push(vfb);
        w.clubs[vfb].league = drei;
        w.clubs[x].league = oben;
        let tabellen: Vec<Vec<usize>> = (0..w.leagues.len()).map(|l| w.table(l).iter().map(|r| r.club).collect()).collect();
        let m = w.zweite_trennen(&tabellen);
        assert_eq!(m.len(), 1, "one move");
        let (z, von, nach, k, _) = m[0];
        assert_eq!((z, von), (ii, drei));
        assert_eq!(w.clubs[ii].league, nach);
        assert_eq!(w.leagues[nach].ueber.as_deref(), Some(w.leagues[drei].key.as_str()), "one tier below");
        assert_eq!(w.clubs[k].league, drei, "the next of the lower table takes the place");
        assert_eq!(k, *tabellen[nach].iter().find(|&&c| c != ii && w.erste_von(c).map_or(true, |e| w.clubs[e].league != drei)).unwrap());
        assert!(w.zweite_trennen(&tabellen).is_empty(), "nothing left to separate");
    }
    #[test]
    fn zweite_steigt_nicht_in_die_liga_der_ersten_auf() {
        let Some((_, w)) = welt(MY) else { return };
        let (porto, b) = (club(&w, "FC Porto"), club(&w, "FC Porto B"));
        let upper = w.clubs[porto].league;
        assert_eq!(w.leagues[w.clubs[b].league].ueber.as_deref(), Some(w.leagues[upper].key.as_str()), "Porto B plays one tier below");
        let rest: Vec<usize> = w.leagues[w.clubs[b].league].teams.iter().copied().filter(|&x| x != b).take(3).collect();
        let order: Vec<usize> = std::iter::once(b).chain(rest.iter().copied()).collect();
        assert_eq!(w.aufsteiger(&order, 3, upper), rest, "the second team stays below, the next one goes up");
    }
    #[test]
    fn partnerleihe_suche_und_kuendigung() {
        let Some((_, w)) = welt(MY) else { return };
        let (chelsea, stras) = (club(&w, "Chelsea FC"), club(&w, "RC Strasbourg Alsace"));
        let mut c = Career::new(w, chelsea, "Test");
        c.mensch.vertrag_bis = u32::MAX;
        c.world.day = 5;
        let mut sq = c.world.clubs[chelsea].squad.clone();
        sq.sort_by(|a, b| c.world.players[*a].ovr.partial_cmp(&c.world.players[*b].ovr).unwrap());
        let p = sq.iter().copied().find(|&p| c.leihe_an_partner(p, stras, false).is_ok()).expect("the partner says yes");
        let l = c.world.players[p].leihe.clone().unwrap();
        assert!(c.world.players[p].team == stras as i32 && l.von == chelsea && !l.grund.is_empty(), "loan at the partner with a reason");
        assert!(c.gehoert_uns(p));
        // the promise "loan players stay" broken by a recall: the partner ends the cooperation
        let ki = c.world.koop_index(chelsea, stras).unwrap();
        c.world.koops[ki].bleiben = true;
        c.leihe_zurueckrufen(p).unwrap();
        assert!(c.world.koop_index(chelsea, stras).is_none(), "cancelled");
        assert!(c.inbox.iter().any(|m| m.title.contains("beendet die Kooperation")));
        // a 14-day search brings an application, accepting makes a contract, at most two at a time
        let msg = c.koop_suche_starten(KoopSuche { bis: 0, nur_land: true, stufe_von: 1, stufe_bis: 2, plaetze: 3, max_betrag: 0.0 }).unwrap();
        assert!(msg.contains("14 Tage"));
        for _ in 0..15 { c.world.day += 1; c.koop_tag(); }
        assert!(c.koop_suche.is_none() && !c.koop_angebote.is_empty(), "at least one application");
        assert!(c.inbox.iter().any(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("koop:"))), "as a case in the inbox");
        let a = c.koop_angebote[0].clone();
        let t = c.wahl(&format!("koop:{}", a.id), &[], "annehmen");
        assert!(t.contains("steht"), "{t}");
        let k = &c.world.koops[c.world.koop_index(chelsea, a.club).unwrap()];
        assert!(k.bis >= c.world.season && k.betrag > 0.0, "a contract with term and yearly amount");
        let frei: Vec<usize> = (0..c.world.clubs.len()).filter(|&x| x != chelsea && x != a.club && c.world.koop_index(chelsea, x).is_none()).take(2).collect();
        c.koop_schliessen(frei[0], 1.0, 0, false, 2).unwrap();
        assert!(c.koop_schliessen(frei[1], 1.0, 0, false, 2).is_err(), "no more than two");
    }
}

