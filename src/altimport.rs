//! Import of a career of the Python version (Fabian 22.11.: the Sunderland career of spring 2027). tools/alt_import.py
//! reads the old welt.db into JSON; this builds the world from the old career's database, then takes over the date,
//! the coach, the players' state, the clubs' money, the league tables (the imported base plus the matches since) and
//! the domestic cups with their fixtures. Leagues and cups the old career did not play (no fixtures there) and the
//! European competitions are played by CalcioM up to the import day. The old career is only read.
use crate::cups::{Cup, CupFx};
use crate::game::Career;
use crate::world::{Fixture, World};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Builds the career; `alt` is the old career folder (its pesdb is the world), `welt` the FL27 world folder.
pub fn importieren(json: &Value, alt: &Path, welt: &Path) -> Result<Career, String> {
    let w = World::from_pesdb(&alt.join(crate::id("pesdb")), 7).map_err(|e| format!("pesdb: {e}"))?; // roh
    let own_id = json["own"].as_u64().ok_or("own fehlt")? as u32; // roh
    let own = w.clubs.iter().position(|c| c.id == own_id).ok_or("eigener Verein nicht in der Welt")?; // roh
    let coach = json["coach"].as_str().unwrap_or("Trainer"); // roh
    let mut c = Career::new(w, own, coach);
    let tag = json["day"].as_u64().ok_or("day fehlt")? as u32; // roh
    let club_ix: HashMap<u32, usize> = c.world.clubs.iter().enumerate().map(|(i, k)| (k.id, i)).collect();
    let spieler_ix: HashMap<u32, usize> = c.world.players.iter().enumerate().map(|(i, p)| (p.id, i)).collect();
    let n = |v: &Value| v.as_f64().unwrap_or(0.0);

    // players: state, season figures, contract
    for s in json["spieler"].as_array().into_iter().flatten() {
        let Some(&p) = s["id"].as_u64().and_then(|id| spieler_ix.get(&(id as u32))) else { continue };
        let pl = &mut c.world.players[p];
        pl.form = n(&s["form"]) as f32;
        pl.fit = n(&s["fit"]) as f32;
        pl.morale = n(&s["morale"]) as f32;
        pl.inj = n(&s["inj"]) as u16;
        pl.susp = n(&s["susp"]) as u8;
        pl.yellow = n(&s["yc"]) as u8;
        pl.goals = n(&s["goals"]) as u16;
        pl.assists = n(&s["assists"]) as u16;
        pl.apps = n(&s["apps"]) as u16;
        pl.rsum = n(&s["rsum"]) as f32;
        pl.rn = n(&s["rated"]) as u16;
        if let Some(e) = s["contract_end"].as_i64() { pl.contract_end = e.max(0) as u32; }
        if s["wage"].as_f64().unwrap_or(0.0) > 0.0 { pl.wage = n(&s["wage"]); }
        if let Some(no) = s["no"].as_u64() { if no > 0 && no < 100 { pl.no = no as u8; } }
    }
    // clubs: money and familiarity
    if let Some(v) = json["vereine"].as_object() {
        for (id, x) in v {
            let Some(&k) = id.parse::<u32>().ok().and_then(|i| club_ix.get(&i)) else { continue };
            let cl = &mut c.world.clubs[k];
            if let Some(b) = x["balance"].as_f64() { cl.balance = b; }
            for (f, z) in [(crate::id("tv"), &mut cl.tv), (crate::id("gate"), &mut cl.gate), (crate::id("comm"), &mut cl.comm), (crate::id("opex"), &mut cl.opex)] { if let Some(b) = x[f].as_f64() { *z = b; } }
            if let Some(f) = x["fam"].as_f64() { cl.fam = f as f32; }
        }
    }
    let fx_von = |f: &Value| -> Option<Fixture> {
        Some(Fixture { day: f["day"].as_u64()? as u32, h: *club_ix.get(&(f["h"].as_u64()? as u32))?, a: *club_ix.get(&(f["a"].as_u64()? as u32))?, hg: f["hg"].as_i64()? as i8, ag: f["ag"].as_i64()? as i8, zeit: 0 })
    };
    // leagues with an old schedule: its fixtures and the imported base table; the others are played up to the day
    let mut nachholen: Vec<usize> = vec![];
    for l in 0..c.world.leagues.len() {
        let key = c.world.leagues[l].key.clone();
        let alt_lg = &json["ligen"][&key];
        let fx: Vec<Fixture> = alt_lg["fixtures"].as_array().into_iter().flatten().filter_map(|f| fx_von(f)).collect();
        if fx.is_empty() { nachholen.push(l); continue; }
        let teams = c.world.leagues[l].teams.clone();
        c.world.leagues[l].fixtures = fx.into_iter().filter(|f| teams.contains(&f.h) && teams.contains(&f.a)).collect();
        c.world.leagues[l].basis = alt_lg["basis"].as_object().into_iter().flatten().filter_map(|(id, r)| {
            let k = *club_ix.get(&id.parse::<u32>().ok()?)?;
            let z: Vec<i32> = r.as_array()?.iter().map(|x| x.as_i64().unwrap_or(0) as i32).collect();
            (z.len() == 7 && teams.contains(&k)).then(|| (k, [z[0], z[1], z[2], z[3], z[4], z[5], z[6]]))
        }).collect();
    }
    // domestic cups with an old schedule: their rounds as they were (the last round with one match is the final)
    for ci in 0..c.world.cups.len() {
        if c.world.cups[ci].kind != 0 { continue; }
        let key = c.world.cups[ci].key.clone();
        let alt_fx: Vec<&Value> = json["pokale"][&key]["fixtures"].as_array().into_iter().flatten().collect();
        if alt_fx.is_empty() { continue; }
        let mut runden: HashMap<u64, usize> = HashMap::new();
        for f in &alt_fx { *runden.entry(f["rnd"].as_u64().unwrap_or(1)).or_default() += 1; }
        let mut fx = vec![];
        let mut teams = vec![];
        for f in &alt_fx {
            let Some(x) = fx_von(f) else { continue };
            let rnd = f["rnd"].as_u64().unwrap_or(1);
            let stage = if runden[&rnd] == 1 { 34 } else { (19 + rnd).min(33) as u8 };
            let pens = f["pens"].as_array().map(|p| (p[0].as_u64().unwrap_or(0) as u8, p[1].as_u64().unwrap_or(0) as u8));
            for t in [x.h, x.a] { if !teams.contains(&t) { teams.push(t); } }
            fx.push(CupFx { f: x, stage, leg: 0, neutral: stage == 34, pens, et: pens.is_some() });
        }
        if fx.is_empty() { continue; }
        let stage = fx.iter().map(|x| x.stage).max().unwrap_or(20);
        // days of the coming rounds: the old ones where known, else one round a week after the import day
        let mut days: Vec<u32> = (1..=14u64).map(|r| alt_fx.iter().find(|f| f["rnd"].as_u64() == Some(r)).and_then(|f| f["day"].as_u64()).map(|d| d as u32).unwrap_or(0)).collect();
        let mut naechster = tag + 4;
        for d in days.iter_mut() { if *d == 0 { *d = naechster.min(crate::date::SEASON_DAYS - 2); naechster += 7; } }
        let cup = &mut c.world.cups[ci];
        cup.teams = teams;
        cup.fx = fx;
        cup.stage = stage;
        cup.alive = vec![];
        cup.days = days;
        cup.winner = None;
        let fin = cup.fx.iter().find(|x| x.stage == 34 && x.f.hg >= 0).cloned();
        if let Some(x) = fin { let w = c.world.winner_of(ci, 34, x.f.h, x.f.a); c.world.cups[ci].winner = Some(w); c.world.cups[ci].alive = vec![w]; }
    }
    // Europe: the own club plays there only if it did in the old career; else a club of its league that is not in Europe
    // takes its place (the other European places come from the database, as at every career start)
    let europa: Vec<u32> = json["europa"].as_array().into_iter().flatten().filter_map(|x| x.as_u64()).map(|x| x as u32).collect();
    if !europa.is_empty() && !europa.contains(&own_id) {
        let liga = c.world.clubs[own].league;
        let drin = |w: &World, k: usize| w.cups.iter().any(|cu| (1..=3).contains(&cu.kind) && cu.teams.contains(&k));
        let ersatz = c.world.table(liga).iter().map(|r| r.club).find(|&k| k != own && !drin(&c.world, k));
        for cu in c.world.cups.iter_mut().filter(|cu| (1..=3).contains(&cu.kind)) {
            let Some(e) = ersatz else { cu.teams.retain(|&k| k != own); continue };
            for t in cu.teams.iter_mut().chain(cu.alive.iter_mut()) { if *t == own { *t = e; } }
            for x in cu.fx.iter_mut() { if x.f.h == own { x.f.h = e; } if x.f.a == own { x.f.a = e; } }
        }
    }
    if let Some(b) = json["budget"].as_f64().filter(|b| *b > 0.0) { c.transfer_budget = b; }
    // everything else up to the day: the leagues without an old schedule, the other cups and Europe
    for d in 0..tag {
        c.world.day = d;
        for &l in &nachholen {
            let level = c.world.leagues[l].detail.max(1);
            for fi in 0..c.world.leagues[l].fixtures.len() {
                let f = c.world.leagues[l].fixtures[fi];
                if f.day != d || f.hg >= 0 { continue; }
                let res = c.world.play(f.h, f.a, level, None, None);
                c.world.apply_result(l, fi, &res);
            }
        }
        c.world.cup_day(d, usize::MAX);
    }
    c.world.day = tag;
    Ok(c)
}

/// The command line: `pesm --import-alt <json> <alte-karriere> <welt-ordner>` builds and saves the career with its own
/// database copy (the old career's pesdb) for FL27.
pub fn kommando(json_pfad: &str, alt: &str, welt: &str) -> Result<String, String> {
    let json: Value = serde_json::from_str(&std::fs::read_to_string(json_pfad).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let (alt, welt) = (Path::new(alt), Path::new(welt));
    let mut c = importieren(&json, alt, welt)?;
    // a name of its own: the existing careers of the club stay untouched
    let name = crate::fl27::cpk_name(&c.club().name.clone(), &crate::fl27::sider().join(crate::id("livecpk")));
    if crate::game::save_dir().join(&name).exists() { return Err(format!("{name} gibt es schon")); } // roh
    c.save_name = name;
    c.fl27_welt = welt.display().to_string();
    let s2 = welt.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_else(crate::fl27::sider);
    crate::fl27::original_sichern(&c.karriere_dir(), &s2, welt)?;
    crate::fl27::karriere_pesdb_anlegen(&c.karriere_dir(), welt)?;
    // the career's database is the old career's (its transfers, development and squads)
    let ziel = crate::fl27::karriere_pesdb(&c.karriere_dir());
    for e in std::fs::read_dir(alt.join(crate::id("pesdb"))).map_err(|e| e.to_string())?.flatten() {
        std::fs::copy(e.path(), ziel.join(e.file_name())).map_err(|x| format!("{}: {x}", e.path().display()))?;
    }
    let _ = c.fl27_cpk();
    c.save()?;
    Ok(format!("{} angelegt: {}, {}", c.save_name, c.club().name, crate::date::long(c.world.season, c.world.day))) // roh
}

#[cfg(test)]
mod tests {
    /// Look at an imported career (CALCIOM_ALT_SAVE=<…json>): run with --ignored --nocapture.
    #[test]
    #[ignore]
    fn import_ansehen() {
        let Ok(p) = std::env::var("CALCIOM_ALT_SAVE") else { return };
        let c = crate::game::load(std::path::Path::new(&p)).unwrap();
        eprintln!("{} {} Tag {} Kasse {:.0}", c.coach, c.club().name, crate::date::long(c.world.season, c.world.day), c.club().balance);
        let l = c.league();
        for (i, r) in c.world.table(l).iter().enumerate() { eprintln!("{:2}. {:28} {:2} {:2}-{:2}-{:2} {:3}:{:3} {:3}", i + 1, c.world.clubs[r.club].name, r.p, r.w, r.d, r.l, r.gf, r.ga, r.pts); }
        for (wo, f) in c.own_fixtures().iter().filter(|x| x.1.hg < 0).take(4) { eprintln!("naechstes: {:?} Tag {} {} - {}", matches!(wo, crate::game::Wo::Pokal(_)), f.day, c.world.clubs[f.h].name, c.world.clubs[f.a].name); }
        for cu in c.world.cups.iter().filter(|x| x.teams.contains(&c.own)) { eprintln!("Pokal {} Stufe {} Sieger {:?} offen {}", cu.name, cu.stage, cu.winner.map(|w| c.world.clubs[w].name.clone()), cu.fx.iter().filter(|x| x.f.hg < 0).count()); }
        let mut sq = c.squad(); sq.sort_by_key(|&p| std::cmp::Reverse(c.world.players[p].goals));
        for &p in sq.iter().take(5) { let pl = &c.world.players[p]; eprintln!("{} Tore {} Spiele {} Note {:.2} Vertrag {}", pl.name, pl.goals, pl.apps, pl.avg_rating(), pl.contract_end); }
        let offen: usize = c.world.leagues.iter().map(|lg| lg.fixtures.iter().filter(|f| f.hg < 0 && f.day < c.world.day).count()).sum();
        let offen_c: usize = c.world.cups.iter().map(|cu| cu.fx.iter().filter(|f| f.f.hg < 0 && f.f.day < c.world.day).count()).sum();
        eprintln!("vergangene offene Ligaspiele {offen}, Pokalspiele {offen_c}");
        // CALCIOM_ALT_WEITER=1: play on into the next season (no panic, the season end takes the imported table)
        if std::env::var("CALCIOM_ALT_WEITER").is_ok() {
            let mut c = c;
            let s0 = c.world.season;
            for _ in 0..120 {
                if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
                for i in 0..c.inbox.len() { if !c.inbox[i].done && matches!(c.inbox[i].action, crate::game::Action::Wahl { .. }) { c.answer(i, true, 0); } }
                c.ultimatum = None;
                if c.end_day().is_err() { break; }
                if c.world.season > s0 && c.world.day > 5 { break; }
            }
            assert!(c.world.leagues.iter().all(|l| l.basis.is_empty()), "Basis gilt nur eine Saison");
            eprintln!("weiter: Saison {} Tag {} Liga {} Titel {:?}", c.world.season, c.world.day, c.world.leagues[c.league()].name, c.world.titles.iter().filter(|t| t.season == s0).map(|t| format!("{} {}", t.comp, c.world.clubs[t.club].name)).take(6).collect::<Vec<_>>());
        }
    }
}

/// `pesm --neue-karriere <welt> <team-id> <trainer>`: a fresh career for tests (e.g. the tactics probe), with its own
/// database copy and CPK name; existing careers stay untouched.
pub fn neue_karriere(welt: &str, team: &str, trainer: &str) -> Result<String, String> {
    let welt = Path::new(welt);
    let w = World::from_pesdb(&welt.join("common/etc/pesdb"), 7).map_err(|e| format!("pesdb: {e}"))?;
    let id: u32 = team.parse().map_err(|_| "Team-ID fehlt".to_string())?; // roh
    let own = w.clubs.iter().position(|c| c.id == id).ok_or("Verein nicht in der Welt")?; // roh
    let mut c = Career::new(w, own, trainer);
    let name = crate::fl27::cpk_name(&c.club().name.clone(), &crate::fl27::sider().join(crate::id("livecpk")));
    if crate::game::save_dir().join(&name).exists() { return Err(format!("{name} gibt es schon")); } // roh
    c.save_name = name;
    c.fl27_welt = welt.display().to_string();
    let s2 = welt.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_else(crate::fl27::sider);
    crate::fl27::original_sichern(&c.karriere_dir(), &s2, welt)?;
    crate::fl27::karriere_pesdb_anlegen(&c.karriere_dir(), welt)?;
    let _ = c.fl27_cpk();
    c.save()?;
    let gegner = c.next_match().map(|f| { let g = if f.h == c.own { f.a } else { f.h }; format!("{} ({})", c.world.clubs[g].name, c.world.clubs[g].id) }).unwrap_or_default();
    Ok(format!("{} angelegt: {}, nächster Gegner {gegner}", c.save_name, c.club().name)) // roh
}
