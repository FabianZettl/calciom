//! Transfer AI for the whole world (docs/intern/TRANSFER-KI.md, Fabian 24.11.): corridors between leagues (tier,
//! distance, language, historical flows, money leagues), a player's market radius, and flow draws for the summary
//! leagues. Numbers in assets/regeln/korridore.toml.
use crate::game::*;
use crate::world::World;

fn k(x: &str) -> f32 { crate::regeln::f("korridore", x) }

/// The nationality of a league's country (World::land_nat counts all its players): cached per day, as it is asked
/// for every candidate a club looks at.
fn liga_nat(w: &World, l: usize) -> u16 {
    thread_local! { static C: std::cell::RefCell<((u32, u32), std::collections::HashMap<usize, u16>)> = Default::default(); }
    C.with(|c| {
        let mut c = c.borrow_mut();
        if c.0 != (w.season, w.day) { *c = ((w.season, w.day), Default::default()); }
        *c.1.entry(l).or_insert_with(|| w.land_nat(l))
    })
}
/// Tier and country of a league, cached per day (strength and names are read for every candidate).
fn liga_info(w: &World, l: usize) -> (i32, String) {
    thread_local! { static C: std::cell::RefCell<((u32, u32), std::collections::HashMap<usize, (i32, String)>)> = Default::default(); }
    C.with(|c| {
        let mut c = c.borrow_mut();
        if c.0 != (w.season, w.day) { *c = ((w.season, w.day), Default::default()); }
        c.1.entry(l).or_insert_with(|| (stufe_roh(w, l), land_roh(w, l))).clone()
    })
}

/// The country of a league: its land, else its name without the trailing number ("Spanien 2" → "Spanien").
pub fn land(w: &World, l: usize) -> String { liga_info(w, l).1 }
fn land_roh(w: &World, l: usize) -> String {
    let lg = &w.leagues[l];
    if !lg.land.is_empty() { return lg.land.clone(); }
    lg.name.trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ').to_string()
}
/// The transfer window of league `l` open on season day `tag` (Teil 2: windows per country, korridore.toml [fenster]):
/// its first and last day, else None.
pub fn fenster(w: &World, l: usize, tag: u32) -> Option<(u32, u32)> {
    let eigen = crate::regeln::wert_oder("korridore", &format!("fenster.{}", land(w, l))).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| Some((x[0].as_u64()? as u32, x[1].as_u64()? as u32))).collect::<Vec<_>>());
    let liste = eigen.unwrap_or_else(|| crate::world::WINDOWS.to_vec());
    liste.into_iter().find(|&(a, b)| tag >= a && tag <= b)
}
/// Tier 0 (A) … 3 (D) of a league by its strength.
pub fn stufe(w: &World, l: usize) -> i32 { liga_info(w, l).0 }
fn stufe_roh(w: &World, l: usize) -> i32 {
    let s = crate::statistik::liga_staerke(w, l);
    if s >= k("stufen.a") { 0 } else if s >= k("stufen.b") { 1 } else if s >= k("stufen.c") { 2 } else { 3 }
}
pub fn land_info(land: &str) -> (String, String) {
    thread_local! { static C: std::cell::RefCell<std::collections::HashMap<String, (String, String)>> = Default::default(); }
    if let Some(x) = C.with(|c| c.borrow().get(land).cloned()) { return x; }
    let x = land_info_roh(land);
    C.with(|c| c.borrow_mut().insert(land.to_string(), x.clone()));
    x
}
fn land_info_roh(land: &str) -> (String, String) {
    // countries missing from the table (or still named "Land der …") count as unknown, never as an error
    let a = crate::regeln::wert_oder("korridore", &format!("land.{land}")).and_then(|v| v.as_array().cloned()).unwrap_or_default();
    (a.first().and_then(|x| x.as_str()).unwrap_or("?").to_string(), a.get(1).and_then(|x| x.as_str()).unwrap_or("?").to_string()) // roh
}
/// The weight of the corridor from league `von` to league `nach` for a player of `alter` (0…≈2).
pub fn korridor(w: &World, von: usize, nach: usize, alter: u8) -> f32 {
    // a club looks at thousands of players a day: one value per league pair (and age side of the money leagues), per day
    thread_local! { static C: std::cell::RefCell<((u32, u32), std::collections::HashMap<(usize, usize, bool), f32>)> = Default::default(); }
    let alt = alter as f32 >= k("gewicht.geld_ab_alter");
    C.with(|c| {
        let mut c = c.borrow_mut();
        if c.0 != (w.season, w.day) { *c = ((w.season, w.day), Default::default()); }
        *c.1.entry((von, nach, alt)).or_insert_with(|| korridor_roh(w, von, nach, alter))
    })
}
fn korridor_roh(w: &World, von: usize, nach: usize, alter: u8) -> f32 {
    let (sv, sn) = (stufe(w, von), stufe(w, nach));
    let idx = ((sv - sn) + 3).clamp(0, 6) as usize; // up a tier = target tier smaller
    let mut g = crate::regeln::wert("korridore", "gewicht.stufe").as_array().and_then(|a| a.get(idx)).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
    let (lv, ln) = (land(w, von), land(w, nach));
    let ((kv, sv_), (kn, sn_)) = (land_info(&lv), land_info(&ln));
    g *= if lv == ln { k("gewicht.land_gleich") } else if kv == kn && kv != "?" { k("gewicht.kontinent_gleich") } else { k("gewicht.kontinent_anders") };
    if lv != ln && sv_ == sn_ && sv_ != "?" { g *= k("gewicht.sprache"); }
    let paare = crate::regeln::wert("korridore", "strom_paare.paare");
    if paare.as_array().into_iter().flatten().any(|p| p[0].as_str() == Some(lv.as_str()) && p[1].as_str() == Some(ln.as_str())) { g *= k("gewicht.strom"); }
    let geld = crate::regeln::wert("korridore", "geld.laender");
    if alter as f32 >= k("gewicht.geld_ab_alter") && geld.as_array().into_iter().flatten().any(|x| x.as_str() == Some(ln.as_str())) { g *= k("gewicht.geld"); }
    g
}
/// The market radius of a player for a target league: 1 inside, a fraction at its edge, 0 outside.
pub fn radius(w: &World, p: usize, nach: usize) -> f32 {
    let pl = &w.players[p];
    if pl.team < 0 { return 1.0; }
    let von = w.clubs[pl.team as usize].league;
    let (lv, ln) = (land(w, von), land(w, nach));
    let ((kv, _), (kn, _)) = (land_info(&lv), land_info(&ln));
    let geld = crate::regeln::wert("korridore", "geld.laender").as_array().into_iter().flatten().any(|x| x.as_str() == Some(ln.as_str()));
    let heimat = pl.nat != 0 && liga_nat(w, nach) == pl.nat;
    if pl.ovr >= k("radius.weltklasse_ab") { return if stufe(w, nach) <= 1 || geld && pl.age >= 30 { 1.0 } else { 0.1 }; }
    if heimat { return 1.0; }
    if pl.ovr >= k("radius.gut_ab") { return if kv == kn || geld { 1.0 } else { 0.4 }; }
    if pl.age as f32 >= k("radius.alt_ab") || pl.ovr < 65.0 { return if lv == ln && stufe(w, nach) >= stufe(w, von) { 1.0 } else { 0.05 }; }
    if lv == ln { 1.0 } else if kv == kn { 0.5 } else { 0.15 }
}
/// Corridor × radius, with the exceptions (back home, or 34+ without a club): the factor on a club's fit score.
pub fn faktor(w: &World, p: usize, kaeufer: usize) -> f32 {
    let pl = &w.players[p];
    let nach = w.clubs[kaeufer].league;
    if pl.team < 0 { return if pl.age >= 34 { 1.0 } else { radius(w, p, nach) }; }
    if pl.nat != 0 && liga_nat(w, nach) == pl.nat { return 1.0; }
    let g = korridor(w, w.clubs[pl.team as usize].league, nach, pl.age) * radius(w, p, nach);
    if g < k("gewicht.mindestens") { 0.0 } else { g.min(1.5) }
}

impl Career {
    /// Flow draws for the summary leagues (the detailed ones search themselves): each window day a share of the league's
    /// planned moves, buyer at random, a player from a corridor league in the buyer's strength band.
    pub fn strom_tag(&mut self) {
        let abs = self.abs();
        for l in 0..self.world.leagues.len() {
            if self.world.leagues[l].detail <= 1 || l == self.league() { continue; }
            // each league buys in its own country's window
            let Some((a, b)) = fenster(&self.world, l, self.world.day) else { continue };
            let tage = (b - a + 1) as f32;
            let winter = (100..250).contains(&a);
            let teams = self.world.leagues[l].teams.clone();
            if teams.is_empty() { continue; }
            let plan = teams.len() as f32 * k("strom.je_team") * if winter { k("strom.winter") } else { 1.0 };
            // today's share: whole moves plus one more by chance
            let heute = plan / tage;
            let n = heute.floor() as usize + (crate::mensch::zufall(&["strom", &l.to_string(), &abs.to_string()]) < heute.fract()) as usize; // roh
            for i in 0..n {
                let s = format!("{l}|{abs}|{i}");
                let kaeufer = teams[(crate::mensch::zufall(&["strom_k", &s]) * teams.len() as f32) as usize % teams.len()]; // roh
                let niveau = self.world.clubs[kaeufer].level;
                // a handful of candidates from anywhere, weighted by the corridor; the best of them
                let mut best: Option<(f32, usize)> = None;
                for j in 0..40 {
                    let p = (crate::mensch::zufall(&["strom_p", &s, &j.to_string()]) * self.world.players.len() as f32) as usize % self.world.players.len(); // roh
                    let pl = &self.world.players[p];
                    if pl.team < 0 || pl.team as usize == kaeufer || pl.youth > 0 || pl.leihe.is_some() || (pl.ovr - niveau).abs() > 5.0 { continue; }
                    if self.world.players[p].team == self.own as i32 || self.verkauf_gesperrt(p).is_some() { continue; }
                    let g = faktor(&self.world, p, kaeufer);
                    if g <= 0.0 { continue; }
                    let wert = g * (1.0 - (pl.ovr - niveau).abs() / 10.0);
                    if best.map_or(true, |x| wert > x.0) { best = Some((wert, p)); }
                }
                let Some((_, p)) = best else { continue };
                let von = self.world.players[p].team as usize;
                let fee = crate::world::round_money(self.world.players[p].value * 0.9);
                self.strom_wechsel(p, von, kaeufer, fee);
            }
        }
    }
    /// A quiet move between AI clubs of the summary leagues: squads, balances, the transfer log; no news.
    fn strom_wechsel(&mut self, p: usize, von: usize, nach: usize, fee: f64) {
        let abs = self.abs();
        self.world.clubs[von].squad.retain(|&x| x != p);
        self.world.clubs[nach].squad.push(p);
        self.world.clubs[von].balance += fee;
        self.world.clubs[nach].balance -= fee;
        let pl = &mut self.world.players[p];
        pl.team = nach as i32;
        pl.joined = abs;
        pl.listed = false;
        self.strom_log.push((p, von, nach, fee));
    }
}

#[cfg(test)]
mod tests {
    use crate::game::*;
    /// Teil 2: each country its own window; Saudi Arabia shops until early September, Germany not in October.
    #[test]
    fn fenster_je_land() {
        let w = crate::world::testwelt(2);
        let l = |n: &str| w.leagues.iter().position(|x| x.name == n).unwrap();
        let (de1, sau) = (l("Deutschland 1"), l("Saudi-Arabien 1"));
        assert!(super::fenster(&w, de1, 10).is_some() && super::fenster(&w, de1, 35).is_none());
        assert!(super::fenster(&w, sau, 35).is_some(), "the Saudi window runs into September");
        assert!(super::fenster(&w, de1, 160).is_some() && super::fenster(&w, de1, 200).is_none());
    }
    #[test]
    fn korridore_und_radius() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 2);
        let l = |n: &str| w.leagues.iter().position(|x| x.name == n).unwrap();
        let (de1, de2, isl, sau) = (l("Deutschland 1"), l("Deutschland 2"), l("Island 1"), l("Saudi-Arabien 1"));
        // the normal path: same country, a tier up
        assert!(super::korridor(&w, de2, de1, 25) > super::korridor(&w, de1, isl, 25));
        // money leagues pull older players
        assert!(super::korridor(&w, de1, sau, 31) > super::korridor(&w, de1, sau, 22));
        assert!(super::stufe(&w, de1) <= super::stufe(&w, isl));
    }
    /// Ten seasons of the world: where the moves go (run: cargo test --release zehn_jahre -- --ignored --nocapture).
    #[test]
    #[ignore]
    fn zehn_jahre() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 8);
        let mut c = Career::new(w, 3, "Test");
        let jahre: u32 = std::env::var("CALCIOM_JAHRE").ok().and_then(|x| x.parse().ok()).unwrap_or(10);
        let mut alle: Vec<(usize, usize, usize, f64, u8)> = vec![];
        for _ in 0..jahre * crate::date::SEASON_DAYS {
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            c.ultimatum = None; c.fired = false; c.press = None;
            let n0 = c.welt_tag.len();
            if c.end_day().is_err() { break; }
            for x in c.welt_tag.iter().skip(n0).chain(c.strom_log.iter()) { alle.push((x.0, c.world.clubs[x.1].league, c.world.clubs[x.2].league, x.3, c.world.players[x.0].age)); }
        }
        let n = alle.len().max(1) as f32;
        let inland = alle.iter().filter(|x| super::land(&c.world, x.1) == super::land(&c.world, x.2)).count() as f32;
        let tief = alle.iter().filter(|x| super::stufe(&c.world, x.2) - super::stufe(&c.world, x.1) > 2).count();
        eprintln!("{} Wechsel in {jahre} Jahren · im eigenen Land {:.0} % · mehr als zwei Stufen runter {tief}", alle.len(), inland / n * 100.0);
        for s in 0..4 { let v: Vec<&(usize, usize, usize, f64, u8)> = alle.iter().filter(|x| super::stufe(&c.world, x.2) == s).collect(); if !v.is_empty() { eprintln!("Stufe {}: {} Wechsel, Ablöse im Mittel {:.1} Mio., Alter {:.1}", ["A", "B", "C", "D"][s as usize], v.len(), v.iter().map(|x| x.3).sum::<f64>() / v.len() as f64 / 1e6, v.iter().map(|x| x.4 as f32).sum::<f32>() / v.len() as f32); } } // roh
    }
}
