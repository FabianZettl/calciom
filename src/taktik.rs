//! Paket 4: tactics and match day in depth. Opponent report (core/gegner/bericht.py with gegner.toml), tactic verdict
//! and position suitability (taktikurteil.py / taktikeignung.py, heuristics of the original), set pieces, penalty
//! taker, captain, saved tactics, shirt numbers and the analysis after a match.
use crate::game::*;
use crate::regeln::f;
use crate::world::*;
use serde::{Deserialize, Serialize};

/// Indices into the 25 abilities (pesdb::attr_de_t()).
const BALLKONTROLLE: usize = 1;
const DRIBBLING: usize = 2;
const FLACHPASS: usize = 4;
const ABSCHLUSS: usize = 6;
const KOPFBALL: usize = 7;
const STANDARDS: usize = 8;
const EFFET: usize = 9;
const SCHNELLIGKEIT: usize = 10;
const ANTRITT: usize = 11;
const SCHUSSKRAFT: usize = 12;
const SPRUNGKRAFT: usize = 13;
const AUSDAUER: usize = 16;

/// Set pieces and saved tactics (Paket 4).
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Standards { pub elfer: Option<u32>, pub freistoss: Option<u32>, pub ecken: Option<u32>, pub kapitaen: Option<u32>, pub vorlagen: Vec<(String, usize, crate::game::Tactic)>,
    /// Role per slot of the starting eleven (rollen.json ids) and instruction per player (anweisungen_t()).
    pub rollen: Vec<(usize, String)>, pub anweisungen: Vec<(u32, u8)> }

pub fn anweisungen_t() -> [&'static str; 4] { [crate::inhalt::t("programm.intensitaet.normal"), crate::inhalt::t("ui.taktik.nach_vorn"), crate::inhalt::t("taktik.spielweise.schalter.7.1"), crate::inhalt::t("ui.taktik.pressen")] }
/// The roles of a position (config/taktik/rollen.json positionen), with German names.
pub fn rollen_fuer(slot: &str) -> Vec<(&'static str, &'static str)> {
    let i = crate::pesdb::pos_index(slot) as u64;
    crate::mensch::arr("rollen", "rollen").iter().filter(|r| r["positionen"].as_array().map_or(false, |a| a.iter().any(|x| x.as_u64() == Some(i)))).map(|r| { let id = r["id"].as_str().unwrap(); (id, rolle_de(id)) }).collect()
}
pub fn rolle_de(id: &str) -> &'static str {
    match id { "ball_playing_defender" => crate::inhalt::t("ui.taktik.spielstarker_verteidiger"), "stopper" => crate::inhalt::t("ui.taktik.vorstopper"), "cover_defender" => crate::inhalt::t("ui.taktik.absichernder_verteidiger"), "defensive_fullback" => crate::inhalt::t("taktik.rolle.defensive_fullback"), "attacking_fullback" => crate::inhalt::t("taktik.rolle.attacking_fullback"),
        "inverted_fullback" => crate::inhalt::t("taktik.rolle.inverted_fullback"), "holding_midfielder" => crate::inhalt::t("ui.taktik.abraeumer"), "deep_lying_playmaker" => crate::inhalt::t("taktik.rolle.deep_lying_playmaker"), "ball_winning_midfielder" => crate::inhalt::t("en2.typ.balleroberer"), "box_to_box" => "Box-to-Box", "advanced_playmaker" => crate::inhalt::t("ui.taktik.spielmacher"),
        "shadow_striker" => crate::inhalt::t("aka.pos.hs"), "traditional_winger" => crate::inhalt::t("ui.taktik.klassischer_fluegel"), "inverted_winger" => crate::inhalt::t("ui.taktik.inverser_fluegel"), "inside_forward" => crate::inhalt::t("ui.taktik.inside_forward"), "wide_playmaker" => crate::inhalt::t("ui.taktik.fluegelspielmacher"), "advanced_forward" => crate::inhalt::t("ui.taktik.tiefe_spitze"),
        "target_forward" => crate::inhalt::t("en2.typ.zielspieler"), "deep_lying_forward" => crate::inhalt::t("ui.taktik.mitspielender_stuermer"), "false_nine" => crate::inhalt::t("taktik.rolle.false_nine"), _ => crate::inhalt::t("attributgruppe.Torwart") }
}
/// Ability index for the names of rollen.json / rollenwissen.json.
fn attr_index(n: &str) -> Option<usize> {
    Some(match n { "abschluss" => 6, "ausdauer" => 16, "balleroberung" => 18, "ballkontrolle" => 1, "defensivgeist" => 17, "dribbling" => 2, "flachpass" => 4, "steilpass" => 5, "koerperkontakt" => 14, "kopfball" => 7, "offensivgeist" => 0, "reflexe" => 23, "schnelligkeit" => 10, "antritt" => 11, "aggressivitaet" => 19, "schusskraft" => 12, _ => return None })
}
/// Hard minimums of a role the player misses (taktikeignung.py luecken): (ability, has, needs).
pub fn rollen_luecken(p: &Player, rolle: &str) -> Vec<(&'static str, f32, f32)> {
    if p.attrs.len() != 25 { return vec![]; }
    let Some(m) = crate::regeln::wert_oder("rollenwissen", &format!("rollen.{rolle}.mindest")).and_then(|v| v.as_object()) else { return vec![] };
    m.iter().filter_map(|(n, s)| { let i = attr_index(n)?; let s = s.as_f64()? as f32; (p.attrs[i] < s).then_some((crate::pesdb::attr_de_t()[i], p.attrs[i], s)) }).collect()
}

fn faehig(p: &Player, idx: &[usize]) -> Option<f32> { if p.attrs.len() != 25 { return None; } Some(idx.iter().map(|&i| p.attrs[i]).sum::<f32>() / idx.len() as f32) }
fn mittel(v: Vec<Option<f32>>) -> Option<f32> { let w: Vec<f32> = v.into_iter().flatten().collect(); if w.is_empty() { None } else { Some(w.iter().sum::<f32>() / w.len() as f32) } }

/// Key figures of a squad from the abilities (bericht.py merkmale): pace of the wide players and centre-backs,
/// aerial duels, technique in midfield, finishing, set pieces.
pub fn merkmale(w: &World, c: usize) -> Vec<(&'static str, Option<f32>)> {
    let sq: Vec<&Player> = w.clubs[c].squad.iter().map(|&p| &w.players[p]).filter(|p| p.youth == 0 && p.inj == 0).collect();
    let beste = |pos: &[&str], n: usize| -> Vec<&Player> { let mut v: Vec<&Player> = sq.iter().copied().filter(|p| pos.contains(&p.pos.as_str())).collect(); v.sort_by(|a, b| b.ovr.partial_cmp(&a.ovr).unwrap()); v.truncate(n); v };
    let aussen = beste(&[crate::id("LB"), crate::id("RB"), crate::id("LMF"), crate::id("RMF"), crate::id("LWF"), crate::id("RWF")], 4);
    let iv = beste(&[crate::id("CB")], 3);
    let zentrum = beste(&[crate::id("DMF"), crate::id("CMF"), crate::id("AMF")], 4);
    let sturm = beste(&[crate::id("SS"), crate::id("CF"), crate::id("LWF"), crate::id("RWF")], 3);
    let mut alle: Vec<&Player> = sq.clone();
    alle.sort_by(|a, b| b.ovr.partial_cmp(&a.ovr).unwrap());
    vec![
        ("tempo_aussen", mittel(aussen.iter().map(|p| faehig(p, &[SCHNELLIGKEIT, ANTRITT])).collect())),
        ("tempo_abwehr", mittel(iv.iter().map(|p| faehig(p, &[SCHNELLIGKEIT, ANTRITT])).collect())),
        (crate::id("luftduelle"), mittel(iv.iter().map(|p| faehig(p, &[SPRUNGKRAFT, KOPFBALL])).collect())),
        ("technik_zentrum", mittel(zentrum.iter().map(|p| faehig(p, &[BALLKONTROLLE, DRIBBLING, FLACHPASS])).collect())),
        (crate::id("abschluss"), mittel(sturm.iter().map(|p| faehig(p, &[ABSCHLUSS, SCHUSSKRAFT])).collect())),
        (crate::id("standards"), mittel(alle.iter().take(5).map(|p| faehig(p, &[STANDARDS, EFFET])).collect())),
        ("luft_angriff", mittel(sturm.iter().map(|p| faehig(p, &[SPRUNGKRAFT, KOPFBALL])).collect())),
    ]
}
pub fn bereich_wort(b: &str) -> &'static str {
    match b { "tempo_aussen" => crate::inhalt::t("ui.taktik.tempo_auf_aussen"), "tempo_abwehr" => crate::inhalt::t("ui.taktik.tempo_in_der_innenverteidigung"), "luftduelle" => crate::inhalt::t("ui.taktik.kopfballstaerke_hinten"), "technik_zentrum" => crate::inhalt::t("gb.bereich.technik_zentrum"), "abschluss" => crate::inhalt::t("ui.taktik.abschlussstaerke"), "standards" => crate::inhalt::t("attribut.Standardausfuehrung"), _ => crate::inhalt::t("ui.taktik.kopfballstaerke_vorne") }
}
/// Where the opponent is vulnerable (bericht.py schwaechen): clearly below the own squad (3 points) or below the
/// absolute limits for aerial duels and pace of the centre-backs (66) is a weakness, clearly above a strength.
pub fn schwaechen(w: &World, gegner: usize, eigen: usize) -> Vec<(&'static str, bool, f32)> {
    let (g, e) = (merkmale(w, gegner), merkmale(w, eigen));
    let ab = f("gegner", "schwelle.abstand");
    let mut aus = vec![];
    for (i, (bereich, gw)) in g.iter().enumerate() {
        let Some(gw) = *gw else { continue };
        let abstand = e[i].1.map(|r| gw - r);
        let abs_schwach = (*bereich == crate::id("luftduelle") && gw < f("gegner", "schwelle.luft_schwach")) || (*bereich == "tempo_abwehr" && gw < f("gegner", "schwelle.iv_tempo_schwach"));
        if (abs_schwach && !abstand.map_or(false, |a| a >= ab)) || abstand.map_or(false, |a| a <= -ab) { aus.push((*bereich, true, abstand.unwrap_or(0.0))); }
        else if abstand.map_or(false, |a| a >= ab) { aus.push((*bereich, false, abstand.unwrap())); }
    }
    aus.sort_by(|a, b| (!a.1).cmp(&!b.1).then(b.2.abs().partial_cmp(&a.2.abs()).unwrap()));
    aus
}

impl Career {
    /// The opponent report before a match (gegner/bericht.py): strength, form, key players, strengths and weaknesses, a plan.
    pub fn gegnerbericht(&self) -> Option<(String, Vec<String>)> {
        let f_ = self.next_match()?;
        let g = if f_.h == self.own { f_.a } else { f_.h };
        let gn = self.world.clubs[g].name.clone();
        let mut zeilen = vec![];
        let st = crate::mensch::kaderstaerke(&self.world, g) - crate::mensch::kaderstaerke(&self.world, self.own);
        zeilen.push(crate::tf!("scouting.staerke", if st >= 3.0 { crate::inhalt::t("ui.taktik.klar_staerker_als_wir") } else if st >= 1.0 { crate::inhalt::t("ui.taktik.etwas_staerker") } else if st <= -3.0 { crate::inhalt::t("ui.taktik.klar_schwaecher_als_wir") } else if st <= -1.0 { crate::inhalt::t("ui.taktik.etwas_schwaecher") } else { crate::inhalt::t("ui.world.auf_augenhoehe") }));
        let table = self.world.table(self.world.clubs[g].league);
        if let Some(r) = table.iter().find(|r| r.club == g) { zeilen.push(crate::tf!("ui.taktik.form_platz", r.form.iter().collect::<String>(), table.iter().position(|x| x.club == g).unwrap() + 1)); }
        let stil = match self.world.clubs[g].style { -1 => crate::inhalt::t("ui.taktik.steht_tief_und_kontert"), 1 => crate::inhalt::t("ui.taktik.spielt_offensiv_nach_vorn"), _ => crate::inhalt::t("ui.taktik.spielt_ausgewogen") };
        zeilen.push(crate::tf!("ui.taktik.spielweise", gn, stil));
        let mut beste: Vec<usize> = self.world.clubs[g].squad.iter().copied().filter(|&p| self.world.players[p].youth == 0 && self.world.players[p].available()).collect();
        beste.sort_by(|a, b| (self.world.players[*b].ovr + self.world.players[*b].goals as f32).partial_cmp(&(self.world.players[*a].ovr + self.world.players[*a].goals as f32)).unwrap());
        let n = f("gegner", "schluessel.anzahl") as usize;
        zeilen.push(crate::tf!("ui.taktik.schluesselspieler", beste.iter().take(n).map(|&p| { let pl = &self.world.players[p]; format!("{} ({}{})", pl.name, pos_de(&pl.pos), if pl.goals as f32 >= f("gegner", "schluessel.torjaeger_ab") { crate::tf!("ui.taktik.tore", pl.goals) } else { String::new() }) }).collect::<Vec<_>>().join(", ")));
        let sw = schwaechen(&self.world, g, self.own);
        let schw: Vec<&str> = sw.iter().filter(|x| x.1).map(|x| bereich_wort(x.0)).collect();
        let stark: Vec<&str> = sw.iter().filter(|x| !x.1).map(|x| bereich_wort(x.0)).collect();
        if !schw.is_empty() { zeilen.push(crate::tf!("ui.taktik.verwundbar", schw.join(", "))); }
        if !stark.is_empty() { zeilen.push(crate::tf!("ui.taktik.vorsicht", stark.join(", "))); }
        // the plan (bericht.py empfehlung): attack the weakest area, protect against the strongest
        let plan = match sw.iter().find(|x| x.1).map(|x| x.0) {
            Some("tempo_abwehr") => crate::inhalt::t("ui.taktik.steilpaesse_hinter_die_abwehr_die"),
            Some("luftduelle") => crate::inhalt::t("ui.taktik.flanken_und_standards_in_den"),
            Some("tempo_aussen") => crate::inhalt::t("ui.taktik.ueber_die_fluegel_kommen_auf"),
            Some("technik_zentrum") => crate::inhalt::t("ui.taktik.im_zentrum_frueh_stoeren_dort"),
            Some(_) => crate::inhalt::t("ui.taktik.geduldig_aufbauen_und_die_luecken"),
            None => if st >= 3.0 { crate::inhalt::t("ui.taktik.kompakt_stehen_und_auf_konter") } else { crate::inhalt::t("ui.taktik.das_eigene_spiel_durchziehen") },
        };
        zeilen.push(crate::tf!("ui.taktik.plan", plan));
        Some((gn, zeilen))
    }
    /// The opponent's key players (strength and goals), as the report names them.
    pub fn schluesselspieler(&self) -> Vec<usize> {
        let Some(f_) = self.next_match() else { return vec![] };
        let g = if f_.h == self.own { f_.a } else { f_.h };
        let mut beste: Vec<usize> = self.world.clubs[g].squad.iter().copied().filter(|&p| self.world.players[p].youth == 0 && self.world.players[p].available()).collect();
        beste.sort_by(|a, b| (self.world.players[*b].ovr + self.world.players[*b].goals as f32).partial_cmp(&(self.world.players[*a].ovr + self.world.players[*a].goals as f32)).unwrap());
        beste.truncate(f("gegner", "schluessel.anzahl") as usize);
        beste
    }
    /// The day before a match the assistant sends the report.
    pub fn gegnerbericht_senden(&mut self) {
        if let Some((gn, z)) = self.gegnerbericht() {
            let co = self.co_name.clone();
            self.msg(0, &co, &crate::tf!("ui.pages.gegnerbericht", gn), &z.join(" · "), Action::None);
            self.meldung_art(crate::pages::Art::Gegner, self.schluesselspieler());
        }
    }
    /// Suitability of a player for a slot (taktikeignung.py): natural (main position or level 2), usable (level 1 or the
    /// same line), poor.
    pub fn eignung(&self, p: usize, slot: &str) -> u8 {
        let pl = &self.world.players[p];
        if pl.pos == slot { return 0; }
        if pl.pp.len() == 13 { let lv = pl.pp[crate::pesdb::pos_index(slot) as usize]; if lv == 2 { return 0; } if lv == 1 { return 1; } }
        if part_of(&pl.pos) == part_of(slot) { 1 } else { 2 }
    }
    /// The assistant's verdict on the tactic (taktikurteil.py heuristics): what the squad can carry and what not.
    pub fn taktikurteil(&self) -> Vec<String> {
        let mut v = vec![];
        let xi: Vec<(String, usize, f32)> = self.current_xi();
        let schlecht: Vec<String> = xi.iter().filter(|(s, p, _)| self.eignung(*p, s) == 2).map(|(s, p, _)| crate::tf!("ui.taktik.als", self.world.players[*p].name, pos_de(s))).collect();
        if !schlecht.is_empty() { v.push(crate::tf!("ui.taktik.ungewohnte_positionen", schlecht.join(", "))); }
        let m = merkmale(&self.world, self.own);
        let get = |k: &str| m.iter().find(|x| x.0 == k).and_then(|x| x.1);
        let ausdauer = mittel(xi.iter().map(|(_, p, _)| faehig(&self.world.players[*p], &[AUSDAUER])).collect());
        if self.tactic.pressing >= 1 && ausdauer.map_or(false, |a| a < 68.0) { v.push(crate::inhalt::t("ui.taktik.hohes_pressing_kostet_kraft_die").into()); }
        if self.tactic.linie >= f("gegner", "schwelle.linie_hoch") as i32 && get("tempo_abwehr").map_or(false, |t| t < f("gegner", "schwelle.iv_tempo_schwach")) { v.push(crate::inhalt::t("ui.taktik.die_hohe_abwehrlinie_ist_riskant").into()); }
        if self.tactic.aufbau == 0 && get("technik_zentrum").map_or(false, |t| t < 68.0) { v.push(crate::inhalt::t("ui.taktik.kurzpassspiel_verlangt_technik_im_zentru").into()); }
        if get("tempo_aussen").map_or(false, |t| t >= f("gegner", "schwelle.tempo_hoch")) && self.tactic.breite == 0 { v.push(crate::inhalt::t("ui.taktik.wir_haben_schnelle_aussen_breiter").into()); }
        for (k, (slot, p, _)) in xi.iter().enumerate() {
            let Some(r) = self.std.rollen.iter().find(|x| x.0 == k).map(|x| x.1.clone()) else { continue };
            let _ = slot;
            if let Some((n, ist, soll)) = rollen_luecken(&self.world.players[*p], &r).first() { v.push(crate::tf!("ui.taktik.als_die_rolle_verlangt", self.world.players[*p].name, rolle_de(&r), n, format!("{:.0}", ist), format!("{:.0}", soll))); }
        }
        if v.is_empty() { v.push(crate::inhalt::t("ui.taktik.die_taktik_passt_zum_kader").into()); }
        v
    }
    /// Set-piece takers: the best by the matching abilities, unless the coach chose someone (Paket 4).
    pub fn schuetze(&self, art: u8) -> Option<usize> {
        let gewaehlt = match art { 0 => self.std.elfer, 1 => self.std.freistoss, 2 => self.std.ecken, _ => self.std.kapitaen };
        let im_team = |p: usize| self.xi.contains(&p);
        if let Some(id) = gewaehlt { if let Some(p) = self.pid(id) { if im_team(p) || art == 3 { return Some(p); } } }
        if art == 3 { return self.kapitaen(); }
        let idx: &[usize] = match art { 0 => &[ABSCHLUSS, SCHUSSKRAFT], 1 => &[STANDARDS, SCHUSSKRAFT], _ => &[STANDARDS, EFFET] };
        self.xi.iter().copied().max_by(|&a, &b| faehig(&self.world.players[a], idx).unwrap_or(self.world.players[a].ovr).partial_cmp(&faehig(&self.world.players[b], idx).unwrap_or(self.world.players[b].ovr)).unwrap())
    }
    pub fn schuetze_waehlen(&mut self, art: u8) {
        // cycles through the starting eleven
        let xi = self.xi.clone();
        if xi.is_empty() { return; }
        let jetzt = self.schuetze(art);
        let i = jetzt.and_then(|p| xi.iter().position(|&x| x == p)).map(|i| (i + 1) % xi.len()).unwrap_or(0);
        let id = Some(self.world.players[xi[i]].id);
        match art { 0 => self.std.elfer = id, 1 => self.std.freistoss = id, 2 => self.std.ecken = id, _ => { self.std.kapitaen = id; self.kapitaen_setzen(xi[i]); } }
    }
    /// A captain chosen by the coach needs no leadership threshold (hierarchie.json).
    pub fn kapitaen_setzen(&mut self, p: usize) {
        for q in self.world.clubs[self.own].squad.clone() { if self.world.players[q].ps.rolle == 0 { self.world.players[q].ps.rolle = 1; } }
        self.world.players[p].ps.rolle = 0;
        let n = self.world.players[p].name.clone();
        self.moral_wirkung(p, 3.0, 0, crate::inhalt::t("ui.taktik.zum_kapitaen_ernannt"));
        self.grund(crate::inhalt::t("bereich.mannschaft"), "Kapitän", 0.0, &crate::tf!("ui.taktik.traegt_die_binde", n)); // roh
    }
    pub fn vorlage_speichern(&mut self) -> Result<(), String> {
        if self.std.vorlagen.len() >= 5 { return Err(crate::inhalt::t("ui.taktik.hoechstens_fuenf_vorlagen_loesche_erst").into()); }
        let n = crate::tf!("ui.taktik.vorlage", self.std.vorlagen.len() + 1, crate::world::formation_slots(self.formation).iter().filter(|s| **s != crate::id("GK")).count());
        self.std.vorlagen.push((n, self.formation, self.tactic.clone()));
        Ok(())
    }
    pub fn vorlage_laden(&mut self, i: usize) { if let Some((_, f_, t)) = self.std.vorlagen.get(i).cloned() { self.formation = f_; self.tactic = t; self.auto_lineup(); } }
    /// Change a shirt number (1…99, unique in the squad); FL27 gets it with the squad (PlayerAssignment).
    pub fn rueckennummer(&mut self, p: usize, d: i32) {
        let belegt: Vec<u8> = self.world.clubs[self.own].squad.iter().filter(|&&q| q != p).map(|&q| self.world.players[q].no).collect();
        let mut n = self.world.players[p].no as i32;
        for _ in 0..99 { n = (n + d - 1).rem_euclid(99) + 1; if !belegt.contains(&(n as u8)) { break; } }
        self.world.players[p].no = n as u8;
    }
    /// Analysis after an own match (beobachter / taktikanalyse): grades per line, best and weakest, what to work on.
    pub fn analyse(&self, res: &MatchResult) -> String {
        let eigen: Vec<(usize, f32)> = res.ratings.iter().copied().filter(|r| self.world.players[r.0].team == self.own as i32).collect();
        if eigen.is_empty() { return String::new(); }
        let linie = |t: usize| { let v: Vec<f32> = eigen.iter().filter(|r| part_of(&self.world.players[r.0].pos) == t).map(|r| r.1).collect(); if v.is_empty() { None } else { Some(v.iter().sum::<f32>() / v.len() as f32) } };
        let namen = [crate::inhalt::t("gb2.sp.teil.tor"), crate::inhalt::t("achse.Abwehr"), crate::inhalt::t("gb2.vgl.mittel"), crate::inhalt::t("bericht.achse.angriff")];
        let teile: Vec<String> = (0..4).filter_map(|t| linie(t).map(|n| format!("{} {:.1}", namen[t], n))).collect();
        let best = eigen.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        let schwach = eigen.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        let home = res.h == self.own;
        let (sf, sg) = if home { res.shots } else { (res.shots.1, res.shots.0) };
        let ballbesitz = if home { res.possession } else { 100 - res.possession };
        let fazit = if sf < sg && ballbesitz < 45 { crate::inhalt::t("ui.taktik.wir_hatten_zu_wenig_vom") } else if sf > sg + 5 { crate::inhalt::t("ui.taktik.viele_abschluesse_an_der_chancenverwertu") } else { crate::inhalt::t("ui.taktik.ein_ausgeglichenes_spiel") };
        crate::tf!("ui.taktik.analyse_noten_je_mannschaftsteil_bester", teile.join(", "), self.world.players[best.0].name, format!("{:.1}", best.1), self.world.players[schwach.0].name, format!("{:.1}", schwach.1), sf, sg, ballbesitz, fazit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gegnerbericht_und_urteil() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        let (_, z) = c.gegnerbericht().expect("a next match");
        assert!(z.iter().any(|l| l.starts_with("Plan:")));
        assert!(!c.taktikurteil().is_empty());
        let p = c.xi[3];
        c.rueckennummer(p, 1);
        let nos: Vec<u8> = c.world.clubs[0].squad.iter().map(|&q| c.world.players[q].no).collect();
        assert_eq!(nos.iter().filter(|&&n| n == c.world.players[p].no).count(), 1, "numbers stay unique");
        c.vorlage_speichern().unwrap();
        c.tactic.pressing = 2;
        c.vorlage_laden(0);
        assert_ne!(c.tactic.pressing, 2);
        assert!(c.schuetze(0).is_some());
    }
}
