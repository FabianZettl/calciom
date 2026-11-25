//! Skill Tree for players (Fabian 24./25.11., konzepte/skilltree.md): good performances fill a bar, a full bar is a
//! Skill Point; points buy nodes on a tree by the player's role (three branches, four tiers) and a Positions branch.
//! The tree is only a boost on top: regular training and development stay as they are. Attribute nodes add +2 over
//! four weeks, skills and play styles are trained (a learning goal with weekly hits) and then written to Player.bin
//! like every skill, positions use the retraining idea, quicker. Numbers in assets/regeln/skilltree.toml.
use crate::game::*;
use crate::regeln::wert;
use serde::{Deserialize, Serialize};

fn f(k: &str) -> f32 { crate::regeln::f("skilltree", k) }

/// A player's tree state: account, bar, points this season, bought nodes, running boosts, the learning goal, the log.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Baum {
    pub konto: u8,
    pub balken: f32,
    pub saison: u32,
    pub saison_sp: u8,
    pub boost_saison: f32,
    pub knoten: Vec<String>,
    /// Attribute boosts still to come: (ability index, points left).
    pub boost: Vec<(usize, f32)>,
    /// The learning goal: (node id, hits so far, abs day started).
    pub lernziel: Option<(String, u8, u32)>,
    /// A position being learned through the tree: (position, abs day ready).
    pub pos_lernt: Option<(String, u32)>,
    pub stil_seit: Option<u32>,
    pub log: Vec<(u32, String, i8)>,
    /// Who spends the points (Konzept 6.1): 0 not chosen yet (the five best: me, the rest: the assistant), 1 me, 2 the assistant.
    pub vergabe: u8,
    /// The assistant's suggestion (node id, abs day); with "assistant" he buys it at the next weekly run.
    pub vorschlag: Option<(String, u32)>,
}

/// One node: id, branch index (99 = Positions), tier 1–4, kind (A, S, P, E, X, L learn, M master, H main position),
/// what it gives, cost, prerequisites (ability, minimum).
#[derive(Clone, Debug, PartialEq)]
pub struct Knoten { pub id: String, pub ast: usize, pub stufe: u8, pub art: char, pub name: String, pub werte: Vec<(usize, f32)>, pub skill: Option<String>, pub kosten: u8, pub voraus: Vec<(usize, u8)> }

fn attr_index(n: &str) -> Option<usize> { crate::pesdb::ATTRS.iter().position(|a| a.0 == n) }
pub fn skill_index(n: &str) -> Option<usize> { crate::pesdb::SKILLS.iter().position(|a| a.0 == n) }
pub fn stil_index(n: &str) -> Option<usize> { crate::pesdb::STYLES.iter().position(|a| *a == n) }

/// The tree key of a position (gk, cb, fb, cm, am, cf).
pub fn baum_von(pos: &str) -> String {
    wert("skilltree", "baum").as_object().and_then(|m| m.iter().find(|(_, b)| b["positionen"].as_array().is_some_and(|a| a.iter().any(|x| x.as_str() == Some(pos)))).map(|(k, _)| k.clone())).unwrap_or_else(|| crate::id("cm").into())
}
pub fn aeste(baum: &str) -> Vec<String> { wert("skilltree", &format!("baum.{baum}.aeste")).as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }

/// "A:LowPass+2:1", "S:Interception:2:DefensiveAwareness>=72,BallWinning>=70", "X:Wall:GKReflexes+1,S=…:4".
fn lesen(id: String, ast: usize, k: usize, s: &str) -> Knoten {
    let teile: Vec<&str> = s.split(':').collect();
    let art = teile[0].chars().next().unwrap_or('A');
    let stufe = if k >= 6 { 4 } else { (k / 2 + 1) as u8 };
    let (name, inhalt, kosten, voraus) = match art {
        'A' => (String::new(), teile.get(1).copied().unwrap_or(""), teile.get(2), teile.get(3)),
        'X' => (teile.get(1).unwrap_or(&"").to_string(), teile.get(2).copied().unwrap_or(""), teile.get(3), teile.get(4)),
        _ => (teile.get(1).unwrap_or(&"").to_string(), "", teile.get(2), teile.get(3)),
    };
    let mut werte = vec![];
    let mut skill = if matches!(art, 'S' | 'P') { Some(name.clone()) } else { None };
    for t in inhalt.split(',').filter(|t| !t.is_empty()) {
        if let Some(sk) = t.strip_prefix(crate::id("S=")) { skill = Some(sk.to_string()); continue; }
        if let Some((a, n)) = t.split_once('+') { if let Some(i) = attr_index(a) { werte.push((i, n.parse().unwrap_or(1.0))); } }
    }
    let voraus = voraus.map(|v| v.split(',').filter_map(|x| { let (a, n) = x.split_once(">=")?; Some((attr_index(a)?, n.parse().ok()?)) }).collect()).unwrap_or_default();
    Knoten { id, ast, stufe, art, name, werte, skill, kosten: kosten.and_then(|k| k.parse().ok()).unwrap_or(1), voraus }
}

/// The nodes of a role tree.
pub fn knoten(baum: &str) -> Vec<Knoten> {
    let mut v = vec![];
    for (a, ast) in aeste(baum).iter().enumerate() {
        for (k, s) in wert("skilltree", &format!("baum.{baum}.{ast}")).as_array().into_iter().flatten().filter_map(|x| x.as_str()).enumerate() {
            v.push(lesen(format!("{baum}.{ast}.{k}"), a, k, s));
        }
    }
    v
}
fn verwandt(a: &str, b: &str) -> bool { wert("skilltree", "regeln.verwandt").as_array().into_iter().flatten().any(|p| { let p: Vec<&str> = p.as_array().into_iter().flatten().filter_map(|x| x.as_str()).collect(); p.len() == 2 && ((p[0] == a && p[1] == b) || (p[0] == b && p[1] == a)) }) }

impl Career {
    /// The Positions branch of a player: learn (related positions he cannot play), master (learned ones), main position
    /// (natural ones besides his own).
    pub fn pos_knoten(&self, p: usize) -> Vec<Knoten> {
        let pl = &self.world.players[p];
        let mut v = vec![];
        for (i, pos) in crate::pesdb::POS.iter().enumerate() {
            if *pos == pl.pos || (*pos == "GK") != (pl.pos == "GK") { continue; } // roh
            let st = pl.pp.get(i).copied().unwrap_or(0);
            let (art, stufe, kosten) = match st { 0 if verwandt(&pl.pos, pos) => ('L', 1, 2), 1 if verwandt(&pl.pos, pos) => ('M', 2, 3), 2 => ('H', 3, 2), _ => continue };
            v.push(Knoten { id: format!("pos.{art}.{pos}"), ast: 99, stufe, art, name: pos.to_string(), werte: vec![], skill: None, kosten, voraus: vec![] });
        }
        v
    }
    /// Whether node `k` can be bought for player `p` now, else the reason in football words.
    pub fn knoten_frei(&self, p: usize, k: &Knoten) -> Result<u8, String> {
        let pl = &self.world.players[p];
        let b = &pl.st;
        let t = |s: &str| crate::inhalt::t(s).to_string();
        if b.knoten.contains(&k.id) { return Err(t("ui.st.grund.gekauft")); }
        if k.art == 'E' { return Err(t("ui.st.grund.spaeter")); }
        // tier: two nodes of the tier below in the same branch
        if k.ast != 99 && k.stufe > 1 {
            let baum = baum_von(&pl.pos);
            let unten = knoten(&baum).into_iter().filter(|x| x.ast == k.ast && x.stufe == k.stufe - 1 && b.knoten.contains(&x.id)).count();
            if unten < 2 { return Err(crate::tf!("ui.st.grund.stufe", k.stufe - 1)); }
        }
        if k.art == 'X' && b.knoten.iter().any(|x| knoten(&baum_von(&pl.pos)).iter().any(|y| &y.id == x && y.art == 'X')) { return Err(t("ui.st.grund.signature")); }
        for &(a, min) in &k.voraus { if (pl.attrs.get(a).copied().unwrap_or(0.0) as u8) < min { return Err(crate::tf!("ui.st.grund.braucht", crate::pesdb::attr_de_t()[a], min, pl.attrs.get(a).copied().unwrap_or(0.0) as u8)); } }
        if matches!(k.art, 'A' | 'X') && !k.werte.is_empty() {
            if pl.age as f32 > f("regeln.attribute_bis_alter") && k.art == 'A' { return Err(t("ui.st.grund.zu_alt")); }
            if pl.ovr >= pl.pot + f("regeln.ueber_potenzial") { return Err(t("ui.st.grund.potenzial")); }
            let koerper = wert("skilltree", "regeln.koerper").as_array().cloned().unwrap_or_default();
            if pl.age as f32 > f("regeln.koerper_bis_alter") && k.werte.iter().any(|(a, _)| koerper.iter().any(|x| x.as_str() == Some(crate::pesdb::ATTRS[*a].0))) { return Err(t("ui.st.grund.tempo")); }
            if b.boost_saison >= f("regeln.boost_saison_max") { return Err(t("ui.st.grund.saison_voll")); }
        }
        if let Some(s) = &k.skill {
            if k.art == 'P' {
                if stil_index(s) == Some(pl.style as usize) { return Err(t("ui.st.grund.hat_schon")); }
                if b.stil_seit.is_some_and(|x| self.world.season < x + f("regeln.stil_saisons") as u32) { return Err(t("ui.st.grund.stil_frisch")); }
            } else if let Some(i) = skill_index(s) {
                if pl.skills >> i & 1 == 1 { return Err(t("ui.st.grund.hat_schon")); }
                if pl.skills.count_ones() >= f("regeln.skills_max") as u32 && k.art == 'S' { return Err(t("ui.st.grund.zehn")); }
            }
            if b.lernziel.is_some() { return Err(t("ui.st.grund.lernt")); }
        }
        if k.ast == 99 {
            if b.pos_lernt.is_some() || pl.ps.umschulung.is_some() { return Err(t("ui.st.grund.lernt_pos")); }
            if k.art == 'L' && b.knoten.iter().filter(|x| x.starts_with(crate::id("pos.L."))).count() >= f("regeln.pos_max") as usize { return Err(t("ui.st.grund.pos_max")); }
            if k.art == 'M' && pl.age as f32 > f("regeln.master_bis_alter") { return Err(t("ui.st.grund.master_alt")); }
        }
        let kosten = k.kosten + if k.art == 'A' && k.werte.iter().any(|(a, _)| pl.attrs.get(*a).copied().unwrap_or(0.0) >= f("regeln.teuer_ab")) { 1 } else { 0 };
        if b.konto < kosten { return Err(crate::tf!("ui.st.grund.punkte", kosten)); }
        Ok(kosten)
    }
    /// Buys a node: attribute boosts queue up over four weeks, skills and styles become the learning goal, positions
    /// start learning; the log gets the reason.
    pub fn knoten_kaufen(&mut self, p: usize, k: &Knoten) -> Result<String, String> {
        let kosten = self.knoten_frei(p, k)?;
        let abs = self.abs();
        let halb = self.world.players[p].age as f32 >= f("regeln.halb_ab_alter");
        let n = self.world.players[p].name.clone();
        let tage_pos = {
            let pl = &self.world.players[p];
            ((28.0 + 3.0 * (pl.age as f32 - 21.0).max(0.0)) * f("regeln.pos_faktor")) as u32
        };
        let b = &mut self.world.players[p].st;
        b.konto -= kosten;
        b.knoten.push(k.id.clone());
        for &(a, w) in &k.werte { let w = if halb && k.art == 'A' { (w / 2.0).max(1.0) } else { w }; b.boost.push((a, w)); b.boost_saison += w; }
        if k.skill.is_some() { b.lernziel = Some((k.id.clone(), 0, abs)); if k.art == 'P' { b.stil_seit = Some(self.world.season); } }
        let text = match k.art {
            'L' => { b.pos_lernt = Some((k.name.clone(), abs + tage_pos)); crate::tf!("ui.st.log.pos", n, crate::world::pos_de(&k.name), tage_pos / 7 + 1) }
            'M' => { b.pos_lernt = Some((k.name.clone(), abs + tage_pos)); crate::tf!("ui.st.log.master", n, crate::world::pos_de(&k.name)) }
            'H' => { crate::tf!("ui.st.log.haupt", n, crate::world::pos_de(&k.name)) }
            _ => crate::tf!("ui.st.log.kauf", n, self.knoten_name(k)),
        };
        if k.art == 'H' { self.world.players[p].pos = k.name.clone(); }
        let b = &mut self.world.players[p].st;
        b.log.push((abs, text.clone(), -(kosten as i8)));
        if b.log.len() > 30 { b.log.remove(0); }
        Ok(text)
    }
    /// The display name of a node.
    pub fn knoten_name(&self, k: &Knoten) -> String {
        match k.art {
            'A' => k.werte.iter().map(|(a, w)| format!("{} +{w:.0}", crate::pesdb::attr_de_t()[*a])).collect::<Vec<_>>().join(", "),
            'L' => crate::tf!("ui.st.k.lernen", crate::world::pos_de(&k.name)),
            'M' => crate::tf!("ui.st.k.meistern", crate::world::pos_de(&k.name)),
            'H' => crate::tf!("ui.st.k.haupt", crate::world::pos_de(&k.name)),
            _ => k.name.clone(),
        }
    }
    /// Skill Points after an own match: the bar fills by the grade, more for the man of the match, a big competition
    /// and a stronger opponent; the player's professionalism and ambition count a little; season limit by age, account
    /// at most 6.
    pub fn st_nach_spiel(&mut self, res: &crate::world::MatchResult, faktor_wb: f32) {
        if !self.feature("skilltree") { return; } // roh
        let side = if res.h == self.own { 0 } else if res.a == self.own { 1 } else { return };
        let gegner = if side == 0 { res.a } else { res.h };
        let diff = crate::mensch::kaderstaerke(&self.world, gegner) - crate::mensch::kaderstaerke(&self.world, self.own);
        let stark = if diff >= f("punkte.staerker_ab") { f("punkte.faktor_staerker") } else if diff <= -f("punkte.staerker_ab") { f("punkte.faktor_schwaecher") } else { 1.0 };
        let eigen: Vec<usize> = (if side == 0 { &res.xi_h } else { &res.xi_a }).iter().map(|x| x.1).chain(res.events.iter().filter(|e| e.kind == 5 && e.side == side).filter_map(|e| e.pid2)).collect();
        let best = res.ratings.iter().filter(|x| eigen.contains(&x.0)).max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|x| x.0);
        let gn = self.world.clubs[gegner].name.clone();
        for &(p, note) in res.ratings.iter().filter(|x| eigen.contains(&x.0)) {
            let min = res.minutes.iter().find(|x| x.0 == p).map(|x| x.1).unwrap_or(90);
            if (min as f32) < f("punkte.min_minuten") || !self.gehoert_uns(p) { continue; }
            let mut plus = wert("skilltree", "punkte.balken").as_array().into_iter().flatten().filter_map(|x| Some((x[0].as_f64()? as f32, x[1].as_f64()? as f32))).filter(|x| note >= x.0).map(|x| x.1).fold(0.0, f32::max);
            if best == Some(p) { plus += f("punkte.spieler_des_spiels"); }
            if plus <= 0.0 { continue; }
            let pl = &self.world.players[p];
            let charakter = 0.85 + 0.3 * (crate::mensch::cw(pl, crate::mensch::PROF) + crate::mensch::cw(pl, crate::mensch::EHRGEIZ)) / 200.0;
            let grund = if best == Some(p) { crate::tf!("ui.st.log.motm", format!("{note:.1}"), gn.clone()) } else { crate::tf!("ui.st.log.note", format!("{note:.1}"), gn.clone()) };
            self.st_balken(p, plus * faktor_wb * stark * charakter, &grund);
        }
    }
    fn st_grenze(&self, p: usize) -> u8 {
        let a = self.world.players[p].age as i64;
        wert("skilltree", "punkte.grenze_alter").as_array().into_iter().flatten().find(|x| x[0].as_i64().is_some_and(|b| a <= b)).and_then(|x| x[1].as_u64()).unwrap_or(2) as u8
    }
    fn st_saison(&mut self, p: usize) {
        let s = self.world.season;
        let b = &mut self.world.players[p].st;
        if b.saison != s { b.saison = s; b.saison_sp = 0; b.boost_saison = 0.0; }
    }
    /// Fills the bar; a full bar is one Skill Point (within the season limit and the account limit).
    pub fn st_balken(&mut self, p: usize, plus: f32, grund: &str) {
        if !self.feature("skilltree") { return; } // roh
        self.st_saison(p);
        let (grenze, max) = (self.st_grenze(p), f("punkte.konto_max") as u8);
        let abs = self.abs();
        let b = &mut self.world.players[p].st;
        if b.konto >= max || b.saison_sp >= grenze { return; }
        b.balken += plus;
        while b.balken >= 100.0 && b.konto < max && b.saison_sp < grenze {
            b.balken -= 100.0;
            b.konto += 1;
            b.saison_sp += 1;
            b.log.push((abs, crate::tf!("ui.st.log.punkt", grund), 1));
        }
        if b.konto >= max || b.saison_sp >= grenze { b.balken = b.balken.min(99.0); }
        if b.log.len() > 30 { b.log.remove(0); }
    }
    /// A Skill Point at once (milestones, achievements), within the same limits.
    pub fn st_punkt(&mut self, p: usize, grund: &str) {
        if !self.feature("skilltree") { return; } // roh
        self.st_saison(p);
        let (grenze, max) = (self.st_grenze(p), f("punkte.konto_max") as u8);
        let abs = self.abs();
        let b = &mut self.world.players[p].st;
        if b.konto >= max || b.saison_sp >= grenze { return; }
        b.konto += 1;
        b.saison_sp += 1;
        b.log.push((abs, crate::tf!("ui.st.log.punkt", grund), 1));
    }
    /// Whether the assistant spends this player's points: chosen per player, else the five best stay with the coach.
    pub fn st_assistent(&self, p: usize) -> bool {
        match self.world.players[p].st.vergabe {
            1 => false,
            2 => true,
            _ => { let mut sq = self.squad(); sq.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap()); !sq.iter().take(5).any(|&q| q == p) }
        }
    }
    /// The assistant's choice: the cheapest buyable node, the branch he has started first, abilities that count most
    /// at his position before skills.
    pub fn st_vorschlag_waehlen(&self, p: usize) -> Option<Knoten> {
        let pl = &self.world.players[p];
        let baum = baum_von(&pl.pos);
        let mut alle = knoten(&baum);
        alle.extend(self.pos_knoten(p));
        let w = crate::pesdb::wvec(crate::pesdb::pos_index(&pl.pos));
        let gekauft_im_ast = |a: usize| alle.iter().filter(|k| k.ast == a && pl.st.knoten.contains(&k.id)).count();
        alle.iter().filter(|k| k.ast != 99 && self.knoten_frei(p, k).is_ok()).max_by(|a, b| {
            let wert = |k: &Knoten| gekauft_im_ast(k.ast) as f32 * 10.0 + k.werte.iter().map(|(i, x)| w.get(*i).copied().unwrap_or(0.0) * x * 20.0).sum::<f32>() + if k.skill.is_some() { 2.0 } else { 0.0 } - k.kosten as f32;
            wert(a).partial_cmp(&wert(b)).unwrap()
        }).cloned()
    }
    /// The weekly round of suggestions (Konzept 6.1): with "assistant" last week's suggestion is bought now, unless the
    /// coach changed something; then everyone with points gets a new suggestion, all of them in one message.
    pub fn st_vorschlaege(&mut self) {
        if !self.feature("skilltree") { return; } // roh
        let abs = self.abs();
        let mut zeilen = vec![];
        for p in self.squad() {
            if self.st_assistent(p) {
                if let Some((kid, _)) = self.world.players[p].st.vorschlag.take() {
                    let baum = baum_von(&self.world.players[p].pos);
                    if let Some(k) = knoten(&baum).into_iter().chain(self.pos_knoten(p)).find(|k| k.id == kid) {
                        if let Ok(t) = self.knoten_kaufen(p, &k) { zeilen.push(crate::tf!("ui.st.co.erledigt", t)); }
                    }
                }
            }
            if self.world.players[p].st.konto == 0 { self.world.players[p].st.vorschlag = None; continue; }
            if let Some(k) = self.st_vorschlag_waehlen(p) {
                let neu = self.world.players[p].st.vorschlag.as_ref().map(|v| &v.0) != Some(&k.id);
                if neu {
                    let n = self.world.players[p].name.clone();
                    zeilen.push(crate::tf!(if self.st_assistent(p) { "ui.st.co.plan_auto" } else { "ui.st.co.plan" }, n, self.knoten_name(&k), k.kosten));
                    self.world.players[p].st.vorschlag = Some((k.id.clone(), abs));
                }
            }
        }
        if zeilen.is_empty() { return; }
        let co = self.co_name.clone();
        let text = format!("{}\n\n{}", crate::inhalt::t("ui.st.co.einleitung"), zeilen.iter().map(|z| format!("• {z}")).collect::<Vec<_>>().join(crate::id("\n")));
        self.msg(1, &co, crate::inhalt::t("ui.st.co.titel"), &text, Action::None);
    }
    /// Once a week: attribute boosts come in (+0.5 a week each), learning goals get their weekly chance, a learned
    /// position is ready at its day. An injured player pauses.
    pub fn st_woche(&mut self) {
        if !self.feature("skilltree") { return; } // roh
        let abs = self.abs();
        let je = f("regeln.boost_je_woche");
        for p in self.squad() {
            if self.world.players[p].inj > 0 { continue; }
            let id = self.world.players[p].id;
            let mut neu_ovr = false;
            {
                let pl = &mut self.world.players[p];
                for (a, rest) in pl.st.boost.iter_mut() { let d = rest.min(je); if let Some(x) = pl.attrs.get_mut(*a) { *x = (*x + d).min(99.0); } *rest -= d; neu_ovr = true; }
                pl.st.boost.retain(|x| x.1 > 0.01);
                if neu_ovr && pl.attrs.len() == 25 { pl.ovr = crate::pesdb::ovr(&pl.attrs, crate::pesdb::pos_index(&pl.pos)).clamp(30.0, 99.0); }
            }
            if let Some((kid, treffer, seit)) = self.world.players[p].st.lernziel.clone() {
                let wurf = crate::mensch::zufall(&[crate::id("skilltree"), &id.to_string(), &(abs / 7).to_string(), &kid]);
                let t = treffer + (wurf < f("regeln.lern_chance")) as u8;
                if t as f32 >= f("regeln.lern_treffer") {
                    let baum = baum_von(&self.world.players[p].pos);
                    let k = knoten(&baum).into_iter().find(|x| x.id == kid);
                    let pl = &mut self.world.players[p];
                    pl.st.lernziel = None;
                    if let Some(s) = k.as_ref().and_then(|k| k.skill.clone()) {
                        if k.as_ref().is_some_and(|k| k.art == 'P') { if let Some(i) = stil_index(&s) { pl.style = i as u8; } }
                        else if let Some(i) = skill_index(&s) { pl.skills |= 1 << i; }
                        let (n, wochen) = (pl.name.clone(), (abs.saturating_sub(seit)) / 7 + 1);
                        let text = crate::tf!("ui.st.log.gelernt", n.clone(), s.clone(), wochen);
                        pl.st.log.push((abs, text.clone(), 0));
                        let co = self.co_name.clone();
                        self.msg(0, &co, &crate::tf!("ui.st.gelernt_titel", n, s), &text, Action::Report { pid: p });
                    }
                } else { self.world.players[p].st.lernziel = Some((kid, t, seit)); }
            }
            if let Some((pos, bis)) = self.world.players[p].st.pos_lernt.clone() {
                if abs >= bis {
                    let pl = &mut self.world.players[p];
                    let i = crate::pesdb::pos_index(&pos) as usize;
                    if pl.pp.len() == 13 { pl.pp[i] = (pl.pp[i] + 1).min(2); }
                    pl.st.pos_lernt = None;
                    let n = pl.name.clone();
                    let text = crate::tf!(if pl.pp.get(i) == Some(&2) { "ui.st.log.pos_natuerlich" } else { "ui.st.log.pos_fertig" }, n.clone(), crate::world::pos_de(&pos));
                    pl.st.log.push((abs, text.clone(), 0));
                    let co = self.co_name.clone();
                    self.msg(0, &co, &crate::tf!("ui.st.pos_titel", n), &text, Action::Report { pid: p });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Every tree reads: three branches of seven nodes, every skill, style and ability name known.
    #[test]
    fn baeume_lesbar() {
        for b in ["gk", "cb", "fb", "cm", "am", "cf"] {
            let k = knoten(b);
            assert_eq!(k.len(), 21, "{b}");
            for x in &k {
                assert!(x.kosten >= 1 && x.kosten <= 4 && (1..=4).contains(&x.stufe));
                if let Some(s) = &x.skill { assert!(if x.art == 'P' { stil_index(s).is_some() } else { skill_index(s).is_some() }, "{b}: {s}"); }
                if x.art == 'A' { assert!(!x.werte.is_empty(), "{}", x.id); }
            }
        }
        assert_eq!(baum_von("RB"), "fb");
    }
    /// Konzept T6: a suggestion for every player with points; with "assistant" it is bought the week after.
    #[test]
    fn co_trainer_vorschlag() {
        let mut c = Career::new(crate::world::testwelt(7), 0, "Test");
        let p = c.squad()[4];
        c.world.players[p].attrs = vec![70.0; 25];
        c.world.players[p].age = 22;
        c.world.players[p].pot = c.world.players[p].ovr + 10.0;
        c.world.players[p].st.konto = 2;
        c.world.players[p].st.vergabe = 2;
        c.st_vorschlaege();
        assert!(c.world.players[p].st.vorschlag.is_some(), "suggestion made");
        assert!(c.inbox.iter().any(|m| m.title == crate::inhalt::t("ui.st.co.titel")));
        c.world.day += 7;
        c.st_vorschlaege();
        assert!(!c.world.players[p].st.knoten.is_empty(), "bought by the assistant");
    }
    /// Konzept T1/T2: points come from grades, never above the account or the season limit; tiers open after two
    /// nodes below; an attribute node comes in over four weeks; a skill is trained, then set.
    #[test]
    fn punkte_kauf_und_lernen() {
        let mut c = Career::new(crate::world::testwelt(7), 0, "Test");
        let p = *c.squad().iter().find(|&&p| c.world.players[p].pos == "CB" && c.world.players[p].attrs.len() == 25).unwrap_or(&c.squad()[0]);
        c.world.players[p].age = 22;
        if c.world.players[p].attrs.len() != 25 { c.world.players[p].attrs = vec![70.0; 25]; }
        c.world.players[p].inj = 0;
        c.world.players[p].pot = c.world.players[p].ovr + 10.0;
        for _ in 0..30 { c.st_balken(p, 40.0, "test"); }
        let b = &c.world.players[p].st;
        assert!(b.konto <= 6 && b.saison_sp <= 7, "{} {}", b.konto, b.saison_sp);
        let baum = baum_von(&c.world.players[p].pos);
        let alle = knoten(&baum);
        let stufe2 = alle.iter().find(|k| k.ast == 0 && k.stufe == 2).unwrap().clone();
        assert!(c.knoten_frei(p, &stufe2).is_err(), "tier 2 closed");
        for k in alle.iter().filter(|k| k.ast == 0 && k.stufe == 1).cloned().collect::<Vec<_>>() { c.knoten_kaufen(p, &k).unwrap(); }
        let vorher = c.world.players[p].attrs.clone();
        assert!(c.knoten_frei(p, &stufe2).is_err() || stufe2.werte.is_empty(), "season limit of +4 reached");
        c.world.players[p].st.boost_saison = 0.0;
        let r = c.knoten_frei(p, &stufe2); assert!(r.is_ok() || c.world.players[p].st.konto < stufe2.kosten, "tier 2 open: {:?} {}", r, c.world.players[p].pos);
        for _ in 0..5 { c.world.day += 7; c.st_woche(); }
        let plus: f32 = c.world.players[p].attrs.iter().zip(&vorher).map(|(a, b)| a - b).sum();
        assert!((plus - 4.0).abs() < 0.01, "two +2 nodes in full: {plus}");
        // a skill node: learned in some weeks, then the bit is set
        c.world.players[p].st.konto = 6;
        if let Some(sk) = alle.iter().find(|k| k.art == 'S' && c.knoten_frei(p, k).is_ok()).cloned() {
            c.knoten_kaufen(p, &sk).unwrap();
            for _ in 0..30 { c.world.day += 7; c.st_woche(); }
            let i = skill_index(sk.skill.as_ref().unwrap()).unwrap();
            assert_eq!(c.world.players[p].skills >> i & 1, 1, "skill learned");
        }
    }
}

// ---------------------------------------------------------------- the manager's tree (Konzept Teil 5)
/// A manager node: id, branch, tier, kind (W value, K trick, T tactic, X signature), name, the value index for W, cost.
#[derive(Clone, Debug)]
pub struct MKnoten { pub id: String, pub ast: usize, pub stufe: u8, pub art: char, pub name: String, pub wert: Option<usize>, pub kosten: u8 }
pub fn m_aeste() -> Vec<String> { wert("skilltree", "manager.aeste").as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }
pub fn m_knoten() -> Vec<MKnoten> {
    let mut v = vec![];
    for (a, ast) in m_aeste().iter().enumerate() {
        for (k, s) in wert("skilltree", &format!("manager.{ast}")).as_array().into_iter().flatten().filter_map(|x| x.as_str()).enumerate() {
            let t: Vec<&str> = s.split(':').collect();
            let art = t[0].chars().next().unwrap_or('K');
            let name = t.get(1).unwrap_or(&"").to_string();
            let wert = (art == 'W').then(|| crate::rest3::WERTE.iter().position(|w| *w == name)).flatten();
            v.push(MKnoten { id: format!("m.{ast}.{k}"), ast: a, stufe: if k >= 6 { 4 } else { (k / 2 + 1) as u8 }, art, name, wert, kosten: t.get(2).and_then(|x| x.parse().ok()).unwrap_or(1) });
        }
    }
    v
}
fn mf(k: &str) -> f32 { crate::regeln::f("skilltree", &format!("manager.{k}")) }

impl Career {
    /// Coaching Points (Konzept 5.1): within the season limit and the account limit, with the reason in the log.
    pub fn cp_geben(&mut self, n: u8, grund: &str) {
        let s = self.world.season;
        let abs = self.abs();
        let b = &mut self.mst;
        if b.saison != s { b.saison = s; b.saison_sp = 0; }
        let n = n.min((mf("saison_max") as u8).saturating_sub(b.saison_sp)).min((mf("konto_max") as u8).saturating_sub(b.konto));
        if n == 0 { return; }
        b.konto += n;
        b.saison_sp += n;
        b.log.push((abs, crate::tf!("ui.mst.log.punkte", n, grund), n as i8));
        if b.log.len() > 30 { b.log.remove(0); }
    }
    /// The highest tier the licence allows (C 1, B 2, A 3, Pro 4).
    pub fn m_stufe_max(&self) -> u8 { crate::mensch::LIZENZEN.iter().position(|s| *s == self.lizenz()).unwrap_or(0) as u8 + 1 }
    pub fn m_frei(&self, k: &MKnoten) -> Result<u8, String> {
        let t = |s: &str| crate::inhalt::t(s).to_string();
        if self.mst.knoten.contains(&k.id) { return Err(t("ui.st.grund.gekauft")); }
        if k.art != 'W' { return Err(t("ui.mst.grund.spaeter")); }
        if k.stufe > self.m_stufe_max() { return Err(crate::tf!("ui.mst.grund.lizenz", k.stufe)); }
        if k.stufe > 1 && m_knoten().iter().filter(|x| x.ast == k.ast && x.stufe == k.stufe - 1 && self.mst.knoten.contains(&x.id)).count() < 2 { return Err(crate::tf!("ui.st.grund.stufe", k.stufe - 1)); }
        if let Some(i) = k.wert { if self.mgr.werte[i] >= mf("wert_max") { return Err(t("ui.mst.grund.max")); } }
        if self.mst.konto < k.kosten { return Err(crate::tf!("ui.st.grund.punkte", k.kosten)); }
        Ok(k.kosten)
    }
    pub fn m_kaufen(&mut self, k: &MKnoten) -> Result<String, String> {
        let kosten = self.m_frei(k)?;
        self.manager_an();
        if let Some(i) = k.wert { self.mgr.werte[i] = (self.mgr.werte[i] + mf("wert_plus")).min(mf("wert_max")); }
        let text = crate::tf!("ui.mst.log.kauf", k.wert.map(|i| crate::rest3::werte_de_t()[i]).unwrap_or(""), format!("{:.0}", k.wert.map(|i| self.mgr.werte[i]).unwrap_or(0.0)));
        let abs = self.abs();
        self.mst.konto -= kosten;
        self.mst.knoten.push(k.id.clone());
        self.mst.log.push((abs, text.clone(), -(kosten as i8)));
        Ok(text)
    }
}

#[cfg(test)]
mod m_tests {
    use super::*;
    /// Konzept T5: account at most 5, season at most 8, the licence caps the tier, a value node adds +3.
    #[test]
    fn manager_baum() {
        let mut c = Career::new(crate::world::testwelt(7), 0, "Test");
        c.manager_an();
        for _ in 0..10 { c.cp_geben(2, "test"); }
        assert_eq!(c.mst.konto, 5);
        let k = m_knoten().into_iter().find(|k| k.art == 'W' && k.stufe == 1).unwrap();
        let vorher = c.mgr.werte[k.wert.unwrap()];
        c.m_kaufen(&k).unwrap();
        assert!((c.mgr.werte[k.wert.unwrap()] - vorher - 3.0).abs() < 0.01 || vorher > 82.0);
        let hoch = m_knoten().into_iter().find(|k| k.stufe > c.m_stufe_max()).unwrap();
        assert!(c.m_frei(&hoch).is_err());
        assert_eq!(m_knoten().len(), 35);
    }
}
