//! Scouting-Konzept Teil 2 und 4 (konzepte/scouting.md, 23.11.2026): what counts per position (assets/regeln/positionsrollen.toml:
//! kern / rolle / plus / egal), the role strength, strengths and weaknesses only from the values that matter, measured against
//! the regulars of the own league, the squad effect (rank at his position, first eleven before and after, who loses time),
//! five part scores and one clear verdict out of six. Role choice per position (2.3) is not built yet: every role value counts.
use crate::game::Career;
use crate::regeln::{f, wert};

fn g(p: &str) -> f32 { f("positionsrollen", p) }
/// The position group of a position (key in positionsrollen.toml).
pub fn gruppe(pos: &str) -> &'static str {
    wert("positionsrollen", "gruppe").as_object().unwrap().iter().find(|(_, v)| v["pos"].as_array().map_or(false, |a| a.iter().any(|x| x == pos))).map(|(k, _)| k.as_str()).unwrap_or(crate::id("zm"))
}
/// Importance of an ability in a group: 3 kern, 2 rolle, 1 plus, 0 egal (abilities named nowhere count as plus).
pub fn wichtig(gr: &str, i: usize) -> u8 {
    let n = crate::entwicklung::NAMEN[i];
    let v = &wert("positionsrollen", "gruppe")[gr];
    let hat = |k: &str| v[k].as_array().map_or(false, |a| a.iter().any(|x| x == n));
    if hat(crate::id("kern")) { 3 } else if hat(crate::id("rolle")) { 2 } else if hat(crate::id("egal")) { 0 } else { 1 }
}
fn gewicht(w: u8) -> f32 { [g("gewicht.egal"), g("gewicht.plus"), g("gewicht.rolle"), g("gewicht.kern")][w as usize] }

/// One strength or weakness: ability, distance to the yardstick, big, only "probably" (the knowledge is too thin).
#[derive(Clone, Debug)]
pub struct Merkmal { pub i: usize, pub abstand: f32, pub gross: bool, pub wohl: bool }

#[derive(Clone, Debug)]
pub struct Bericht {
    pub gruppe: &'static str,
    pub staerke: f32,
    /// Kader besser, System, Preis-Leistung, Zukunft, Risiko (0–100) and the points.
    pub teil: [f32; 5],
    pub punkte: f32,
    /// 0 Sofort holen, 1 Holen, 2 Holen wenn der Preis stimmt, 3 Gute Ergänzung, 4 Beobachten, 5 Nicht holen.
    pub urteil: usize,
    pub ko: Option<&'static str>,
    /// Own players of the group with their role strength, best first (the candidate not included) and his rank among them.
    pub rang: Vec<(usize, f32)>,
    pub platz: usize,
    pub startelf: (f32, f32),
    pub verlierer: Vec<usize>,
    pub staerken: Vec<Merkmal>,
    pub schwaechen: Vec<Merkmal>,
}

impl Career {
    /// Role strength (Konzept 2.4): weighted mean of the abilities, kern ×3, rolle ×2, plus ×0,5, egal 0, minus the position penalty.
    pub fn rollen_staerke(&self, p: usize, gr: &str) -> f32 {
        let pl = &self.world.players[p];
        if pl.attrs.len() != 25 { return pl.ovr; }
        let (mut s, mut n) = (0.0, 0.0);
        for i in 0..25 { let w = gewicht(wichtig(gr, i)); s += pl.attrs[i] * w; n += w; }
        let mut v = if n > 0.0 { s / n } else { pl.ovr };
        // position level: natural in the group no penalty, learnt −4, not at all −12
        let in_gruppe = gruppe(&pl.pos) == gr;
        if !in_gruppe {
            let stufe = wert("positionsrollen", "gruppe")[gr]["pos"].as_array().unwrap().iter().filter_map(|x| x.as_str()).map(|x| pl.pp.get(crate::pesdb::pos_index(x) as usize).copied().unwrap_or(0)).max().unwrap_or(0);
            v += if stufe >= 2 { 0.0 } else if stufe == 1 { g("gewicht.gelernt") } else { g("gewicht.fremd") };
        }
        v
    }
    /// The yardstick: mean abilities of the regulars of this group in the own league (best by overall, as many as an eleven uses).
    pub fn massstab(&self, gr: &str) -> Vec<f32> {
        let je = match gr { "tw" | "dm" | "om" | "hs" | "st" => 1, _ => 2 };
        let (mut summe, mut n) = (vec![0.0f32; 25], 0.0f32);
        for &t in &self.world.leagues[self.league()].teams {
            let mut v: Vec<usize> = self.world.clubs[t].squad.iter().copied().filter(|&p| gruppe(&self.world.players[p].pos) == gr && self.world.players[p].attrs.len() == 25).collect();
            v.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
            for p in v.into_iter().take(je) { for i in 0..25 { summe[i] += self.world.players[p].attrs[i]; } n += 1.0; }
        }
        if n == 0.0 { return vec![60.0; 25]; }
        summe.into_iter().map(|x| x / n).collect()
    }
    /// Strengths and weaknesses only from the values that count for the group (Konzept 4.5): egal never, plus only as strength.
    pub fn staerken_schwaechen(&self, p: usize, gr: &str, mass: &[f32]) -> (Vec<Merkmal>, Vec<Merkmal>) {
        let pl = &self.world.players[p];
        if pl.attrs.len() != 25 { return (vec![], vec![]); }
        let unsicher = if self.gehoert_uns(p) { 0.0 } else { self.spanne(p) };
        let (st, gr_s) = (g("schwelle.staerke"), g("schwelle.gross"));
        let (mut plus, mut minus) = (vec![], vec![]);
        for i in 0..25 {
            let w = wichtig(gr, i);
            if w == 0 { continue; }
            let d = pl.attrs[i] - mass[i];
            let wohl = d.abs() < st + unsicher;
            if d >= st { plus.push(Merkmal { i, abstand: d, gross: d >= gr_s, wohl }); }
            else if d <= -st && w >= 2 { minus.push(Merkmal { i, abstand: d, gross: w == 3 && d <= -gr_s, wohl }); }
        }
        plus.sort_by(|a, b| b.abstand.partial_cmp(&a.abstand).unwrap());
        minus.sort_by(|a, b| a.abstand.partial_cmp(&b.abstand).unwrap());
        plus.truncate(g("schwelle.staerken_hoechstens") as usize);
        minus.truncate(g("schwelle.schwaechen_hoechstens") as usize);
        (plus, minus)
    }
    /// The whole report of a candidate from the own point of view (Konzept 4.2–4.5).
    pub fn scout_urteil(&self, p: usize) -> Bericht {
        let pl = &self.world.players[p];
        let gr = gruppe(&pl.pos);
        let staerke = self.rollen_staerke(p, gr);
        let mass = self.massstab(gr);
        let (staerken, schwaechen) = self.staerken_schwaechen(p, gr, &mass);
        let mut rang: Vec<(usize, f32)> = self.squad().into_iter().filter(|&q| q != p && gruppe(&self.world.players[q].pos) == gr).map(|q| (q, self.rollen_staerke(q, gr))).collect();
        rang.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let platz = rang.iter().filter(|x| x.1 > staerke).count();
        // part scores
        let kader = match rang.first() {
            None => 90.0,
            Some(&(_, beste)) => { let d = staerke - beste; (60.0 + d * 20.0 / 3.0).clamp(0.0, 100.0) }
        };
        let system = (40.0 + 60.0 * self.system_fit(p)).clamp(0.0, 100.0);
        let preis = (60.0 + self.moneyball(p) * 5.0 + if pl.team < 0 { 20.0 } else { 0.0 }).clamp(0.0, 100.0);
        let luft = (pl.pot - pl.ovr).max(0.0);
        let zukunft = (if pl.age <= 23 { 65.0 } else if pl.age >= 31 { 15.0 } else { 45.0 } + luft * 2.0).clamp(0.0, 100.0);
        let unsicher = if self.gehoert_uns(p) { 0.0 } else { self.spanne(p) };
        let risiko = (unsicher * 5.0 + if pl.age >= 31 { 25.0 } else { 0.0 } + if pl.inj > 0 { 20.0 } else { 0.0 }).clamp(0.0, 100.0);
        let gw: Vec<f32> = wert("positionsrollen", "urteil.gewichte").as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect();
        let teil = [kader, system, preis, zukunft, risiko];
        let punkte = gw[0] * kader + gw[1] * system + gw[2] * preis + gw[3] * zukunft - gw[4] * risiko + gw[4] * 50.0;
        // knock-out reasons: a core value far under the league's regulars
        let ko = if pl.attrs.len() == 25 && (0..25).any(|i| wichtig(gr, i) == 3 && pl.attrs[i] < mass[i] - 10.0) { Some("ui.sb.ko.kern") } else { None };
        let u = |k: &str| g(&format!("urteil.{k}"));
        let mut urteil = if ko.is_some() { 5 }
            else if punkte >= u(crate::id("sofort")) && kader >= 80.0 { 0 }
            else if punkte >= u(crate::id("holen")) { 1 }
            else if punkte >= u(crate::id("preis")) && preis < 55.0 { 2 }
            else if punkte >= u(crate::id("ergaenzung")) && kader >= 30.0 { 3 }
            else if unsicher >= 4.0 || zukunft >= 70.0 { 4 }
            else { 5 };
        // weaker than the own second choice: never "Holen" (Konzept Teil 9, Punkt 3)
        if urteil <= 2 && rang.get(1).map_or(false, |x| staerke < x.1) { urteil = 3; }
        // first eleven before and after: the candidate takes the place of the weakest of his group in it
        let elf: Vec<f32> = self.xi.iter().map(|&q| self.rollen_staerke(q, gruppe(&self.world.players[q].pos))).collect();
        let vor = elf.iter().sum::<f32>() / elf.len().max(1) as f32;
        let mut verlierer = vec![];
        let mut nach = vor;
        if let Some((k, &q)) = self.xi.iter().enumerate().filter(|(_, &q)| gruppe(&self.world.players[q].pos) == gr).min_by(|a, b| elf[a.0].partial_cmp(&elf[b.0]).unwrap()) {
            if staerke > elf[k] { nach = vor + (staerke - elf[k]) / elf.len().max(1) as f32; verlierer.push(q); }
        }
        Bericht { gruppe: gr, staerke, teil, punkte, urteil, ko, rang, platz, startelf: (vor, nach), verlierer, staerken, schwaechen }
    }
    /// The report as text for the inbox: verdict, at our club, strengths and weaknesses, certainty.
    pub fn urteil_text(&self, p: usize) -> String {
        let b = self.scout_urteil(p);
        let t = crate::inhalt::t;
        let name = |q: usize| self.world.players[q].name.clone();
        let mut z = vec![format!("{} {}", t(&format!("ui.sb.urteil.{}", b.urteil)), t(&format!("ui.sb.satz.{}", b.urteil)))];
        if let Some(k) = b.ko { z.push(t(k).to_string()); }
        let rolle = match b.platz { 0 if b.rang.first().map_or(true, |x| b.staerke - x.1 > 2.0) => "stamm", 0 => "herausforderer", 1 => crate::id("rotation"), _ if self.world.players[p].age <= 20 => "talent", _ => crate::id("ersatz") };
        let konk = b.rang.iter().take(3).map(|(q, s)| format!("{} ({s:.0})", name(*q))).collect::<Vec<_>>().join(", ");
        z.push(crate::tf!("ui.sb.bei_uns", b.platz + 1, t(&format!("ui.sb.rolle.{rolle}")), format!("{:.0}", b.staerke), if konk.is_empty() { t("ui.sb.niemand").to_string() } else { konk }));
        z.push(crate::tf!("ui.sb.startelf", format!("{:.1}", b.startelf.0), format!("{:.1}", b.startelf.1)));
        if !b.verlierer.is_empty() { z.push(crate::tf!("ui.sb.verliert", b.verlierer.iter().map(|&q| name(q)).collect::<Vec<_>>().join(", "))); }
        let wort = |m: &Merkmal, art: &str| format!("{}{} ({:+.0})", if m.wohl { t("ui.sb.wohl") } else { "" }, t(&format!("ui.sb.{art}.{}", crate::entwicklung::NAMEN[m.i])), m.abstand);
        if !b.staerken.is_empty() { z.push(crate::tf!("ui.sb.staerken", b.staerken.iter().map(|m| wort(m, crate::id("plus"))).collect::<Vec<_>>().join(" · "))); }
        z.push(if b.schwaechen.is_empty() { t("ui.sb.keine_schwaeche").to_string() } else { crate::tf!("ui.sb.schwaechen", b.schwaechen.iter().map(|m| wort(m, crate::id("minus"))).collect::<Vec<_>>().join(" · ")) });
        let sicher = if self.gehoert_uns(p) { 0 } else { let s = self.spanne(p); if s <= 2.0 { 0 } else if s <= 4.0 { 1 } else { 2 } };
        z.push(t(&format!("ui.sb.sicher.{sicher}")).to_string());
        z.join(crate::id("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn staerken_schwaechen_nur_relevant() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 7);
        let mut c = Career::new(w, 0, "Test");
        let idx = |n: &str| crate::entwicklung::NAMEN.iter().position(|x| *x == n).unwrap();
        let gk: Vec<usize> = (20..25).collect();
        let mut berichte = 0;
        // the built-in world has no single abilities: give every player the template of his position, with some spread
        for p in 0..c.world.players.len() {
            if c.world.players[p].attrs.len() == 25 { continue; }
            let (pos, ovr) = (c.world.players[p].pos.clone(), c.world.players[p].ovr);
            let pi = crate::pesdb::pos_index(&pos);
            let mut a: Vec<f32> = (0..25).map(|i| 50.0 + ((p * 13 + i * 7) % 31) as f32).collect();
            for _ in 0..3 { let d = ovr - crate::pesdb::ovr(&a, pi); crate::pesdb::shift(&mut a, pi, d); }
            let mut pp = vec![0u8; 13]; pp[pi as usize] = 2;
            for (i, x) in a.iter_mut().enumerate() { *x = (*x + ((p * 31 + i * 17) % 21) as f32 - 10.0).clamp(30.0, 99.0); }
            c.world.players[p].attrs = a; c.world.players[p].pp = pp;
        }
        // a thousand random players of the world, each with random knowledge
        for k in 0..1000usize {
            let p = (k * 7919) % c.world.players.len();
            if c.world.players[p].attrs.len() != 25 || c.world.players[p].team < 0 { continue; }
            c.world.players[p].know = (k % 10) as f32 / 10.0;
            let pos = c.world.players[p].pos.clone();
            let gr = gruppe(&pos);
            let b = c.scout_urteil(p);
            berichte += 1;
            assert!(b.urteil <= 5 && !c.urteil_text(p).is_empty());
            for m in &b.schwaechen {
                if pos != "GK" { assert!(!gk.contains(&m.i), "{pos}: GK-Wert als Schwäche"); }
                if pos == "GK" { assert!(gk.contains(&m.i) || wichtig(gr, m.i) >= 2, "TW: Feldwert {} als Schwäche", m.i); }
                if pos == "CB" { assert!(m.i != idx("abschluss") && m.i != idx("effet"), "IV: Abschluss/Effet als Schwäche"); }
                if pos == "CF" { assert!(m.i != idx("defensivgeist"), "ST: Defensivgeist als Schwäche"); }
                assert!(wichtig(gr, m.i) >= 2, "Plus- oder Egal-Wert als Schwäche");
            }
            for m in &b.staerken { assert!(wichtig(gr, m.i) >= 1, "Egal-Wert als Stärke"); }
            // never "Holen" when weaker than the own second choice
            if b.rang.get(1).map_or(false, |x| b.staerke < x.1) { assert!(b.urteil >= 3); }
        }
        assert!(berichte > 500, "{berichte}");
    }
}
