//! Search profiles (pesm/core/scouting/suchprofil.py): criteria for finding players, at once from the market or as a scout's
//! assignment. A criterion at 0 (or empty) does not restrict. Presets: talent under 22, free soon, set-piece specialist,
//! cover for the weakest position of the own squad.
use crate::game::*;
use crate::world::*;
use serde::{Deserialize, Serialize};

/// Abilities a minimum can be set for (index into the 25 values, names as in entwicklung.rs NAMEN).
pub const ATTRIBUTE: [usize; 14] = [10, 11, 12, 6, 2, 1, 4, 5, 7, 13, 18, 16, 14, 8];
pub const POSITIONEN: [&str; 13] = [crate::id("GK"), crate::id("CB"), crate::id("LB"), crate::id("RB"), crate::id("DMF"), crate::id("CMF"), crate::id("LMF"), crate::id("RMF"), crate::id("AMF"), crate::id("LWF"), crate::id("RWF"), crate::id("SS"), crate::id("CF")];

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Debug)]
#[serde(default)]
pub struct Suchprofil {
    pub name: String,
    pub pos: Vec<String>,
    pub alter_min: u8,
    pub alter_max: u8,
    pub staerke_min: u8,
    pub pot_min: u8,
    /// Contract left at most so many months (free agents always pass).
    pub vertrag_monate: u8,
    /// Market value and yearly wage at most (Mio. €, 0: any).
    pub wert_max: u32,
    pub gehalt_max: u32,
    /// 0 any, 1 right, 2 left.
    pub fuss: u8,
    /// (index into ATTRIBUTE, minimum).
    pub attr: Vec<(u8, u8)>,
}

impl Career {
    /// Does player `p` fit `k`? `geschaetzt`: rating and potential as the market sees them (our knowledge), else the truth (a scout on site).
    pub fn profil_passt(&self, p: usize, k: &Suchprofil, geschaetzt: bool) -> bool {
        let pl = &self.world.players[p];
        if !k.pos.is_empty() && !k.pos.iter().any(|x| *x == pl.pos) { return false; }
        if pl.attrs.len() == 25 { for &(i, m) in &k.attr { if pl.attrs[ATTRIBUTE[i as usize % ATTRIBUTE.len()]] < m as f32 { return false; } } } else if !k.attr.is_empty() { return false; }
        if (k.alter_min > 0 && pl.age < k.alter_min) || (k.alter_max > 0 && pl.age > k.alter_max) { return false; }
        let (st, pot) = if geschaetzt { let (a, b) = self.ovr_range(p); let (x, y) = self.pot_range(p); ((a + b) / 2.0, (x + y) as f32 / 2.0) } else { (pl.ovr, pl.pot) };
        if k.staerke_min > 0 && st < k.staerke_min as f32 { return false; }
        if k.pot_min > 0 && pot < k.pot_min as f32 { return false; }
        if k.fuss > 0 && pl.profil.koerper[2] != k.fuss - 1 && pl.profil.koerper[0] > 0 { return false; }
        if k.vertrag_monate > 0 && pl.team >= 0 && self.monate_rest(p) > k.vertrag_monate as f32 { return false; }
        if k.wert_max > 0 && pl.value > k.wert_max as f64 * 1e6 { return false; }
        if k.gehalt_max > 0 && pl.wage > k.gehalt_max as f64 * 1e6 { return false; }
        true
    }
    /// The presets: talent under 22, free soon, set-piece specialist, cover for the weakest position of the own squad.
    pub fn profil_vorlagen(&self) -> Vec<Suchprofil> {
        let t = crate::inhalt::t;
        let mut v = vec![
            Suchprofil { name: t("sup.vorlage.talent_u21").into(), alter_max: 21, pot_min: 75, ..Default::default() },
            Suchprofil { name: t("sup.vorlage.abloesefrei").into(), vertrag_monate: 6, staerke_min: 65, ..Default::default() },
            Suchprofil { name: t("sup.vorlage.standardspezialist").into(), pos: vec![crate::id("AMF").into(), crate::id("CMF").into()], attr: vec![(13, 75)], ..Default::default() },
        ];
        // the weakest position by its best player (only positions the squad fills at all)
        let sq = self.squad();
        if let Some((pos, best)) = POSITIONEN.iter().filter_map(|&ps| sq.iter().filter(|&&p| self.world.players[p].pos == ps).map(|&p| self.world.players[p].ovr).fold(None, |m: Option<f32>, x| Some(m.map_or(x, |m| m.max(x)))).map(|b| (ps, b))).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
            v.push(Suchprofil { name: crate::tf!("sup.ersatz", pos_de(pos)), pos: vec![pos.into()], staerke_min: (best + 2.0).round() as u8, ..Default::default() });
        }
        v
    }
    /// Hits of a profile on the market (not own players, not youth), best first by estimated rating.
    pub fn profil_treffer(&self, k: &Suchprofil, max: usize) -> Vec<usize> {
        let mut l: Vec<usize> = (0..self.world.players.len()).filter(|&p| { let pl = &self.world.players[p]; pl.team != self.own as i32 && pl.team != -2 && pl.youth == 0 && (pl.team >= 0 || pl.team == -1) && self.profil_passt(p, k, true) }).collect();
        l.sort_by(|a, b| { let (x, y) = (self.ovr_range(*a), self.ovr_range(*b)); (y.0 + y.1).partial_cmp(&(x.0 + x.1)).unwrap() });
        l.truncate(max);
        l
    }
    /// Sends the first free scout with the profile (an assignment of `dauer` days).
    pub fn profil_beauftragen(&mut self, k: &Suchprofil, dauer: u32) -> Result<String, String> {
        let abs = self.abs();
        let s = (0..self.scouts.len()).find(|&s| self.scouts[s].p.as_ref().map_or(true, |p| p.auftrag_bis <= abs)).ok_or_else(|| crate::inhalt::t("sup.kein_scout").to_string())?;
        self.auftrag(s, dauer, if k.alter_max > 0 { k.alter_max } else { 99 })?;
        self.person(s).profil = Some(k.clone());
        Ok(crate::tf!("sup.ok.beauftragt", self.scouts[s].name.clone(), k.name.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_filtern() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let c = Career::new(w, 0, "Test");
        let v = c.profil_vorlagen();
        assert!(v.len() >= 3);
        let jung = c.profil_treffer(&v[0], 50);
        assert!(jung.iter().all(|&p| c.world.players[p].age <= 21));
        let alle = c.profil_treffer(&Suchprofil::default(), 100_000).len();
        assert!(c.profil_treffer(&Suchprofil { pos: vec!["GK".into()], ..Default::default() }, 100_000).len() < alle, "a criterion narrows");
    }
}
