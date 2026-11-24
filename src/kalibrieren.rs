//! The calibration command (Statistik Teil 9, 21): `pesm --kalibrieren <seasons> [<league name>]` plays whole seasons of
//! the built-in world and compares the league's figures with the targets in statistik_ziele.toml, as a traffic light.
use crate::game::*;
use crate::statistik as sx;

pub fn lauf(saisons: usize, liga: Option<&str>) -> String {
    let w = crate::world::World::load(include_str!("../assets/world.json"), 11);
    let l = liga.and_then(|n| w.leagues.iter().position(|x| x.name == n)).unwrap_or(0);
    let own = w.leagues[l].teams.first().copied().unwrap_or(0);
    let mut c = Career::new(w, own, "Kalibrierung"); // roh
    // the figures of the running season, taken each day; when the season starts again they fall, so the last ones count
    let schnitt = |c: &Career| {
        let (mut sp, mut to, mut he, mut re) = (0usize, 0usize, 0usize, 0usize);
        for f in c.world.leagues[l].fixtures.iter().filter(|f| f.hg >= 0) {
            sp += 1; to += (f.hg + f.ag) as usize;
            if f.hg > f.ag { he += 1 } else if f.hg == f.ag { re += 1 }
        }
        let mut st = vec![0u64; sx::N];
        for p in &c.world.players { if let Some(x) = p.profil.ligen.iter().find(|x| x.l as usize == l) { for i in 0..sx::N.min(x.st.len()) { st[i] += x.st[i] as u64; } } }
        (sp, to, he, re, st)
    };
    let (mut spiele, mut tore, mut heim, mut remis) = (0usize, 0usize, 0usize, 0usize);
    let mut st = vec![0u64; sx::N];
    let mut letzt = schnitt(&c);
    let mut fertig = 0;
    while fertig < saisons {
        if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
        c.ultimatum = None;
        c.fired = false;
        if c.end_day().is_err() { break; }
        let jetzt = schnitt(&c);
        if jetzt.0 < letzt.0 {
            spiele += letzt.0; tore += letzt.1; heim += letzt.2; remis += letzt.3;
            for i in 0..sx::N { st[i] += letzt.4[i]; }
            fertig += 1;
        }
        letzt = jetzt;
    }
    let teams = (spiele * 2).max(1) as f64;
    let werte: Vec<(&str, f64)> = vec![
        ("tore_je_spiel", tore as f64 / spiele.max(1) as f64),
        (crate::id("heimsieg"), heim as f64 / spiele.max(1) as f64),
        (crate::id("remis"), remis as f64 / spiele.max(1) as f64),
        ("xg_je_tor", st[sx::XG] as f64 / 1000.0 / tore.max(1) as f64),
        ("schuesse_je_team", st[sx::SCHUESSE] as f64 / teams),
        ("aufs_tor_quote", st[sx::AUFS_TOR] as f64 / st[sx::SCHUESSE].max(1) as f64),
        (crate::id("passquote"), st[sx::PAESSE_OK] as f64 / st[sx::PAESSE].max(1) as f64),
        ("paesse_je_team", st[sx::PAESSE] as f64 / teams),
        ("tacklings_je_team", st[sx::TACKL] as f64 / teams),
        ("gelb_je_team", st[sx::GELB] as f64 / teams),
    ];
    let mut out = format!("Kalibrierung: {} · {saisons} Saison(en) · {spiele} Spiele\n", c.world.leagues[l].name); // roh
    for (k, v) in werte {
        let z = crate::regeln::wert("statistik_ziele", k).as_array().cloned().unwrap_or_default();
        let (a, b) = (z.first().and_then(|x| x.as_f64()).unwrap_or(0.0), z.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0));
        let ampel = if v >= a && v <= b { "GRÜN" } else if v >= a * 0.85 && v <= b * 1.15 { "GELB" } else { "ROT " }; // roh
        out += &format!("{ampel}  {k:<20} {v:>9.3}   Ziel {a}–{b}\n"); // roh
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn eine_saison_kalibrieren() { eprintln!("{}", super::lauf(1, None)); }
}
