//! Trainingsplan (pesm/core/trainingsplan/service.py): a week of seven days with three sessions each (morning, afternoon,
//! evening), every session a unit of einheiten.toml with its intensity. From the plan come the day unit the rest of the game
//! reads (`Career.plan`: fitness, injuries, load, morale), how much the week trains each ability (entwicklung.rs) and the
//! co-trainer's weekly training report. The Einzel slot of the original stays with the individual programmes (tab 2).
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::game::{Action, Career};
use crate::regeln::{f, wert};

pub const SLOTS: usize = 3;
pub const TYPEN: [&str; 4] = [crate::id("training"), crate::id("spieltag"), crate::id("regeneration"), crate::id("frei")];
pub const STUFEN: [&str; 3] = [crate::id("schonend"), crate::id("normal"), crate::id("hart")];

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Debug)]
pub struct Zelle { pub einheit: String, pub stufe: u8 }
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Debug)]
pub struct Tag { pub typ: u8, pub slots: [Option<Zelle>; SLOTS] }
/// The week by weekday (Monday first); empty in a save from before the plan.
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Debug)]
pub struct Woche { pub tage: Vec<Tag> }

fn num(v: &Value) -> f32 { v.as_f64().unwrap_or(0.0) as f32 }
pub fn einheit(id: &str) -> &'static Value { &wert("einheiten", "einheit")[id] }
fn r(k: &str) -> f32 { f("einheiten", &format!("raster.{k}")) }
fn nutzen(stufe: u8) -> f32 { num(&wert("einheiten", "raster.intensitaeten")[STUFEN[stufe.min(2) as usize]]["nutzen"]) }
fn last_faktor(stufe: u8) -> f32 { num(&wert("einheiten", "raster.intensitaeten")[STUFEN[stufe.min(2) as usize]]["last"]) }

/// The group units a session can hold (the Einzel units of the original need the Einzel slot).
pub fn katalog() -> Vec<&'static str> {
    wert("einheiten", "einheit").as_object().unwrap().iter().filter(|(_, e)| e["art"] == "gruppe").map(|(k, _)| k.as_str()).collect()
}
pub fn name(id: &str) -> &'static str { crate::inhalt::t(&format!("tp.einheit.{id}.name")) }
pub fn gruppe(id: &str) -> &'static str { einheit(id)["gruppe"].as_str().unwrap_or("") }

/// Whether a unit fits this slot of a day of this type (service.py pruefen): nothing on a free day; on match day only the
/// first slot and a light unit; on a recovery day only units up to the rehab load.
pub fn passt(typ: u8, slot: usize, id: &str) -> bool {
    let last = num(&einheit(id)["last"]);
    match typ { 3 => false, 1 => slot == 0 && last <= r("spieltag_max_last"), 2 => last <= r("reha_max_last"), _ => true }
}
/// Sets a day's type and takes out the sessions that no longer fit.
pub fn typ_setzen(t: &mut Tag, typ: u8) {
    t.typ = typ;
    for (i, s) in t.slots.iter_mut().enumerate() { if s.as_ref().map_or(false, |z| !passt(typ, i, &z.einheit)) { *s = None; } }
}

/// The ready-made weeks of einheiten.toml (first three slots): (id, week).
pub fn werk() -> Vec<(String, Woche)> {
    crate::mensch::arr("einheiten", "vorlage").iter().map(|v| {
        let tage = v["tage"].as_array().unwrap().iter().map(|t| {
            let typ = TYPEN.iter().position(|x| t["typ"] == *x).unwrap_or(0) as u8;
            let mut slots: [Option<Zelle>; SLOTS] = Default::default();
            for (i, s) in t["slots"].as_array().unwrap().iter().take(SLOTS).enumerate() {
                if let Some(id) = s.as_str().filter(|x| !x.is_empty() && einheit(x).is_object()) { slots[i] = Some(Zelle { einheit: id.into(), stufe: 1 }); }
            }
            Tag { typ, slots }
        }).collect();
        (v["id"].as_str().unwrap_or("").to_string(), Woche { tage })
    }).collect()
}

/// Load points per day (unit × intensity, 60 minutes); with `spiel` the match day adds its match load.
pub fn tageslast(w: &Woche, spiel: bool) -> Vec<f32> {
    w.tage.iter().map(|t| t.slots.iter().flatten().map(|z| num(&einheit(&z.einheit)["last"]) * last_faktor(z.stufe)).sum::<f32>()
        + if spiel && t.typ == 1 { r("spieltag_last") } else { 0.0 }).collect()
}
/// The day unit of the rest of the game (game.rs units: frei, Regeneration, leicht, normal, Taktik, intensiv) by the
/// thresholds of service.py hebel. Own decision: the match itself is not counted, the match day carries its own load.
pub fn tageseinheiten(w: &Woche) -> [u8; 7] {
    let mut out = [0u8; 7];
    for (d, (t, last)) in w.tage.iter().zip(tageslast(w, false)).enumerate().take(7) {
        let zellen: Vec<&Zelle> = t.slots.iter().flatten().collect();
        let taktik = !zellen.is_empty() && zellen.iter().all(|z| gruppe(&z.einheit) == "taktik");
        out[d] = if t.typ == 3 || zellen.is_empty() { 0 } else if last <= 0.0 { 1 } else if last <= 2.0 { 2 } else if taktik && last <= 4.0 { 4 } else if last <= 6.0 { 3 } else { 5 };
    }
    out
}

/// Plan points per ability for the week: basis × main/side share × intensity × repetition (service.py wirkung_spieler,
/// without the player's own factors: age, talent and position work in the development model).
pub fn punkte(w: &Woche) -> [f32; 25] {
    let mut out = [0.0f32; 25];
    let mut n: std::collections::HashMap<&str, i32> = Default::default();
    for z in w.tage.iter().flat_map(|t| t.slots.iter().flatten()) {
        let e = einheit(&z.einheit);
        let wdh = n.entry(z.einheit.as_str()).or_insert(0);
        let fak = num(&e["basis"]) * nutzen(z.stufe) * r(crate::id("wiederholung")).powi(*wdh);
        *wdh += 1;
        for (feld, anteil) in [(crate::id("haupt"), 1.0), (crate::id("neben"), r("neben_anteil"))] {
            for a in e[feld].as_array().into_iter().flatten() {
                if let Some(i) = crate::entwicklung::NAMEN.iter().position(|x| Some(*x) == a.as_str()) { out[i] += fak * anteil; }
            }
        }
    }
    out
}
/// Growth factor per ability from the week (einheiten.toml [wirkung]).
pub fn faktoren(w: &Woche) -> Vec<f32> {
    let k = |x: &str| f("einheiten", &format!("wirkung.{x}"));
    punkte(w).iter().map(|p| k(crate::id("ungeuebt")) + (k(crate::id("geuebt")) - k(crate::id("ungeuebt"))) * (p / k("voll_ab")).min(1.0)).collect()
}

/// Load over the week (the decay of entwicklung.toml belastung) from a start value, and the zone of its peak (0 low … 3 overloaded).
pub fn verlauf(w: &Woche, start: f32) -> (Vec<f32>, usize) {
    let abbau = f("entwicklung", "belastung.abbau");
    let mut b = start;
    let v: Vec<f32> = tageslast(w, true).into_iter().map(|l| { b = (b + l - abbau * b).clamp(0.0, 100.0); b }).collect();
    let spitze = v.iter().cloned().fold(0.0, f32::max);
    let zone = crate::mensch::arr("entwicklung", "belastung.zonen").iter().position(|z| spitze < num(&z[1])).unwrap_or(3);
    (v, zone)
}
pub fn zone_name(z: usize) -> &'static str { [crate::id("niedrig"), crate::id("normal"), crate::id("hoch"), crate::id("ueberlastet")][z.min(3)] }

/// A week from the old day units (saves from before the plan): the closest sessions for each.
pub fn aus_einheiten(plan: &[u8; 7]) -> Woche {
    let z = |id: &str, stufe: u8| Some(Zelle { einheit: id.into(), stufe });
    let tage = plan.iter().map(|&u| match u {
        0 => Tag { typ: 3, slots: [None, None, None] },
        1 => Tag { typ: 2, slots: [z(crate::id("regeneration"), 1), None, None] },
        2 => Tag { typ: 0, slots: [z(crate::id("passspiel"), 0), None, None] },
        4 => Tag { typ: 0, slots: [z(crate::id("taktikschulung"), 2), z(crate::id("videoanalyse"), 1), None] },
        5 => Tag { typ: 0, slots: [z(crate::id("ausdauerlauf"), 2), z(crate::id("zweikampf"), 1), z(crate::id("torschuss"), 1)] },
        _ => Tag { typ: 0, slots: [z(crate::id("passspiel"), 1), z(crate::id("taktikschulung"), 1), None] },
    }).collect();
    Woche { tage }
}

impl Career {
    /// The plan of this career (built from the old day units once).
    pub fn wochenplan_an(&mut self) {
        if self.wochenplan.tage.len() != 7 { self.wochenplan = aus_einheiten(&self.plan); }
    }
    /// Takes the plan over into the day units.
    pub fn plan_uebernehmen(&mut self) {
        self.wochenplan_an();
        self.plan = tageseinheiten(&self.wochenplan);
    }
    /// The co-trainer's week from the fixtures of the next seven days: match day, recovery the day after, set pieces and
    /// tactics the day before, the other days in rotation; a tired squad gets lighter work.
    pub fn co_wochenplan(&mut self) {
        let heute = self.world.day;
        let spiele: Vec<u32> = self.own_fixtures().into_iter().map(|x| x.1.day).filter(|&d| d + 1 >= heute && d <= heute + 8).collect();
        let sq = self.squad();
        let fit = sq.iter().map(|&p| self.world.players[p].fit).sum::<f32>() / sq.len().max(1) as f32;
        let mude = fit < f("einheiten", "co.muede_unter");
        let z = |id: &str, stufe: u8| Some(Zelle { einheit: id.into(), stufe });
        let rot: [[&str; 3]; 4] = [[crate::id("passspiel"), crate::id("stellungsspiel"), crate::id("torwarttraining")], [crate::id("torschuss"), crate::id("offensivspiel"), crate::id("kraftraum")], [crate::id("schnelligkeit"), crate::id("spielaufbau"), crate::id("torwarttraining")], [crate::id("zweikampf"), crate::id("fluegelspiel"), crate::id("videoanalyse")]];
        let mut tage: Vec<Tag> = vec![Tag::default(); 7];
        let mut k = 0;
        for d in heute..heute + 7 {
            let wd = crate::date::weekday(d);
            tage[wd] = if spiele.contains(&d) { Tag { typ: 1, slots: [None, None, None] } }
                else if d > 0 && spiele.contains(&(d - 1)) { Tag { typ: 2, slots: [z(crate::id("regeneration"), 1), None, None] } }
                else if spiele.contains(&(d + 1)) { Tag { typ: 0, slots: [z(crate::id("taktikschulung"), 1), z(if d % 2 == 0 { crate::id("freistoesse") } else { crate::id("eckenvariationen") }, 0), None] } }
                else if mude && k % 2 == 1 { k += 1; Tag { typ: 2, slots: [z(crate::id("regeneration"), 1), z(crate::id("videoanalyse"), 1), None] } }
                else {
                    let r = rot[(d as usize / 7 + k) % 4];
                    k += 1;
                    Tag { typ: 0, slots: [z(r[0], if mude { 0 } else { 1 }), z(r[1], 1), z(r[2], 0)] }
                };
        }
        self.wochenplan = Woche { tage };
        self.plan = tageseinheiten(&self.wochenplan);
    }

    /// The co-trainer's report of the week: what was trained, the load, who gained most in the trained abilities, what
    /// the week left out and who got hurt in training.
    pub fn trainingsbericht(&mut self) {
        self.wochenplan_an();
        let w = self.wochenplan.clone();
        let mut gruppen: Vec<(&str, usize)> = vec![];
        for z in w.tage.iter().flat_map(|t| t.slots.iter().flatten()) {
            let g = gruppe(&z.einheit);
            if let Some(x) = gruppen.iter_mut().find(|x| x.0 == g) { x.1 += 1; } else { gruppen.push((g, 1)); }
        }
        let co = self.co_name.clone();
        let titel = crate::inhalt::t("ui.tp.bericht.titel");
        if gruppen.is_empty() { self.msg(1, &co, titel, crate::inhalt::t("ui.tp.bericht.nichts"), Action::None); return; }
        gruppen.sort_by(|a, b| b.1.cmp(&a.1));
        let mut body = vec![crate::tf!("ui.tp.bericht.einheiten", gruppen.iter().map(|(g, n)| format!("{} {n}", crate::inhalt::t(&format!("tp.gruppe.{g}")))).collect::<Vec<_>>().join(", "))];
        let start = { let sq = self.squad(); sq.iter().map(|&p| self.world.players[p].ps.last).sum::<f32>() / sq.len().max(1) as f32 };
        let (_, zone) = verlauf(&w, start);
        body.push(crate::tf!("ui.tp.bericht.last", crate::inhalt::t(&format!("tp.zone.{}", zone_name(zone)))));
        let mut gew = std::mem::take(&mut self.tr_gewinn);
        gew.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        gew.retain(|x| x.1 > 0.0);
        if !gew.is_empty() { body.push(crate::tf!("ui.tp.bericht.profit", gew.iter().take(5).map(|(p, x)| format!("{} (+{:.2})", self.world.players[*p].name, x)).collect::<Vec<_>>().join(", "))); }
        // the areas a field player needs that the week did not touch
        let fk = faktoren(&w);
        let mut luecken: Vec<&str> = vec![];
        for (g, felder) in crate::entwicklung::fokusgruppen() .iter().map(|(n, v)| (n.clone(), v.clone())).collect::<Vec<_>>().iter() {
            if felder.iter().all(|&i| fk[i] <= f("einheiten", "wirkung.ungeuebt") + 0.01) { luecken.push(crate::inhalt::t(&format!("ui.ent.fokus.{g}"))); }
        }
        if !luecken.is_empty() { body.push(crate::tf!("ui.tp.bericht.luecke", luecken.join(", "))); }
        if self.tr_verletzt > 0 { body.push(crate::tf!("ui.tp.bericht.verletzt", self.tr_verletzt)); }
        self.tr_verletzt = 0;
        self.msg(1, &co, titel, &body.join(crate::id("\n")), Action::None);
        self.meldung_karte(crate::pages::Art::Talent, crate::id("entwicklung"), gew.into_iter().take(6).collect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plan_lenkt_und_belastet() {
        let werk = werk();
        assert!(werk.len() >= 5 && werk.iter().all(|w| w.1.tage.len() == 7));
        let std = &werk.iter().find(|w| w.0 == "standard").unwrap().1;
        let intensiv = &werk.iter().find(|w| w.0 == "intensivwoche").unwrap().1;
        let erhol = &werk.iter().find(|w| w.0 == "regenerationswoche").unwrap().1;
        // an intensive week is hard days, a recovery week light ones
        assert!(tageseinheiten(intensiv).iter().filter(|&&u| u == 5).count() >= 2, "{:?}", tageseinheiten(intensiv));
        assert!(tageseinheiten(erhol).iter().all(|&u| u <= 2), "{:?}", tageseinheiten(erhol));
        assert!(verlauf(intensiv, 30.0).1 > verlauf(erhol, 30.0).1);
        // the standard week moves the trained abilities up and the rest down, about evenly over the field abilities
        let fk = faktoren(std);
        let feld: Vec<f32> = (0..20).map(|i| fk[i]).collect();
        let mittel = feld.iter().sum::<f32>() / feld.len() as f32;
        assert!((0.9..=1.15).contains(&mittel), "Mittel {mittel} {fk:?}");
        let i = |n: &str| crate::entwicklung::NAMEN.iter().position(|x| *x == n).unwrap();
        assert!(fk[i("flachpass")] > fk[i("dribbling")]);
        // the slots follow the day type
        assert!(!passt(3, 0, "passspiel") && !passt(1, 1, "videoanalyse") && passt(1, 0, "videoanalyse") && !passt(2, 0, "ausdauerlauf"));
        // an old save's units come back as the same day units
        let alt = [3, 3, 2, 3, 4, 0, 0];
        assert_eq!(tageseinheiten(&aus_einheiten(&alt)), alt);
    }
}
