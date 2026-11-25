//! Spielerstatistiken (konzepte/spielerstatistiken.md, Teil 3, 23.11.2026): every detailed match leaves a trace of actions
//! per player (passes, runs, duels, shots with xG, saves …). Team values are the sums of the players; every goal is a shot
//! with a shooter and an xG; the goalkeeper's saves plus goals against are the opponent's shots on target. Numbers in
//! assets/regeln/statistik.toml.
use crate::world::{MatchResult, Rng, World};
use std::sync::OnceLock;

// ---------------------------------------------------------------- the numbers kept per player and match
pub const MIN: usize = 0;
pub const SPIELE: usize = 1;
pub const TORE: usize = 2;
pub const SCHUESSE: usize = 3;
pub const AUFS_TOR: usize = 4;
/// Thousandths of a goal.
pub const XG: usize = 5;
/// Thousandths of a goal.
pub const XA: usize = 6;
pub const VORLAGEN: usize = 7;
pub const SCHLUESSEL: usize = 8;
pub const PAESSE: usize = 9;
pub const PAESSE_OK: usize = 10;
pub const PROG_PAESSE: usize = 11;
pub const PROG_LAEUFE: usize = 12;
pub const DRIBB: usize = 13;
pub const DRIBB_OK: usize = 14;
pub const FLANKEN: usize = 15;
pub const FLANKEN_OK: usize = 16;
pub const STRAFRAUM: usize = 17;
pub const TACKL: usize = 18;
pub const TACKL_OK: usize = 19;
pub const ABGEF: usize = 20;
pub const KOPF: usize = 21;
pub const KOPF_OK: usize = 22;
pub const KLAER: usize = 23;
pub const BLOCKS: usize = 24;
pub const FOULS: usize = 25;
pub const BALLVERL: usize = 26;
pub const GELB: usize = 27;
pub const ROT: usize = 28;
pub const PARADEN: usize = 29;
pub const GEGENTORE: usize = 30;
/// Thousandths of a goal (post-shot xG of the shots on target he faced).
pub const PSXG: usize = 31;
pub const EIGENTORE: usize = 32;
pub const GEBLOCKT: usize = 33;
pub const N: usize = 34;
/// Text keys (ui.stat.<key>) in the order of the indices.
pub const KEYS: [&str; N] = [crate::id("minuten"), crate::id("spiele"), crate::id("tore"), crate::id("schuesse"), "aufs_tor", crate::id("xg"), crate::id("xa"), crate::id("vorlagen"), crate::id("schluesselpaesse"), crate::id("paesse"), "paesse_ok",
    "prog_paesse", "prog_laeufe", crate::id("dribblings"), "dribblings_ok", crate::id("flanken"), "flanken_ok", crate::id("strafraum"), crate::id("tacklings"), "tacklings_ok", crate::id("abgefangen"),
    crate::id("kopfball"), "kopfball_ok", crate::id("klaerungen"), crate::id("blocks"), crate::id("fouls"), crate::id("ballverluste"), crate::id("gelb"), crate::id("rot"), crate::id("paraden"), crate::id("gegentore"), crate::id("psxg"), crate::id("eigentore"), crate::id("geblockt")];

pub type Stat = Vec<u32>;
/// One shot of a match: minute, side, shooter, xG in thousandths, outcome (0 wide, 1 saved, 2 goal, 3 blocked).
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Schuss { pub min: u8, pub side: u8, pub pid: usize, pub xg: u16, pub aus: u8 }
pub const DANEBEN: u8 = 0;
pub const GEHALTEN: u8 = 1;
pub const TOR: u8 = 2;
pub const GEBLOCKT_S: u8 = 3;
/// Value i of a statistic (0 when it was never filled).
pub fn get(s: &Stat, i: usize) -> u32 { s.get(i).copied().unwrap_or(0) }
pub fn add(a: &mut Stat, b: &Stat) {
    if a.len() < N { a.resize(N, 0); }
    for (i, x) in b.iter().enumerate().take(N) { a[i] += x; }
}

// ---------------------------------------------------------------- families of actions (statistik.toml `familien`)
pub const F_PAESSE: usize = 0;
pub const F_PROGP: usize = 1;
pub const F_PROGL: usize = 2;
pub const F_DRIBB: usize = 3;
pub const F_FLANK: usize = 4;
pub const F_SCHUSS: usize = 5;
pub const F_STRAF: usize = 6;
pub const F_SCHLUESSEL: usize = 7;
pub const F_TACKL: usize = 8;
pub const F_ABGEF: usize = 9;
pub const F_KOPF: usize = 10;
pub const F_KLAER: usize = 11;
pub const F_BLOCKS: usize = 12;
pub const F_FOULS: usize = 13;
pub const F_BALLV: usize = 14;
const NF: usize = 15;

struct Art { anteil: f32, xg: f32, block: f32, tor: f32, schuetze: usize, vorbereiter: Option<usize> }
struct Cfg {
    rate: [f32; NF],
    profil: Vec<(String, [f32; NF])>,
    werte: Vec<Vec<usize>>,
    exp: f32, fmin: f32, fmax: f32, spielweise: f32,
    erfolg: Vec<Option<(f32, f32, f32)>>,
    arten: Vec<Art>,
    xg_sd: f32, xg_min: f32, xg_max: f32, tor_je10: f32, k: f32, weitere_min: f32, team_sd: f32, vorlage_quote: f32, ziel_f: f32,
    poss_exp: f32, tech_basis: f32, stil_off: [f32; NF], stil_def: [f32; NF],
}
fn cfg() -> &'static Cfg {
    static C: OnceLock<Cfg> = OnceLock::new();
    C.get_or_init(|| {
        let w = |k: &str| crate::regeln::wert("statistik", k);
        let fam: Vec<String> = w(crate::id("familien")).as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        let idx = |n: &str| fam.iter().position(|x| x == n).unwrap_or_else(|| panic!("statistik: Familie {n}")); // roh
        let zahl = |v: &serde_json::Value| v.as_f64().unwrap_or(0.0) as f32;
        let mut rate = [0.0; NF];
        for (k, v) in w(crate::id("teamrate")).as_object().unwrap() { rate[idx(k)] = zahl(v); }
        let profil = w(crate::id("profil")).as_object().unwrap().iter().map(|(g, v)| {
            let mut a = [0.0; NF];
            for (i, x) in v.as_array().unwrap().iter().enumerate().take(NF) { a[i] = zahl(x); }
            (g.clone(), a)
        }).collect();
        let mut werte = vec![vec![]; NF];
        for (k, v) in w(crate::id("familienwerte")).as_object().unwrap() { werte[idx(k)] = v.as_array().unwrap().iter().filter_map(|x| x.as_u64().map(|x| x as usize)).collect(); }
        let mut erfolg = vec![None; NF];
        for (k, v) in w(crate::id("erfolg")).as_object().unwrap() { let a = v.as_array().unwrap(); erfolg[idx(k)] = Some((zahl(&a[0]), zahl(&a[1]), zahl(&a[2]))); }
        let c = w(crate::id("chancen"));
        let arten = c["arten"].as_array().unwrap().iter().map(|a| Art {
            anteil: zahl(&a["anteil"]), xg: zahl(&a["xg"]), block: zahl(&a["block"]), tor: zahl(&a["tor"]),
            schuetze: idx(a["schuetze"].as_str().unwrap()),
            vorbereiter: a["vorbereiter"].as_str().filter(|s| !s.is_empty()).map(idx),
        }).collect();
        let stil = |k: &str| { let mut a = [1.0; NF]; for (n, v) in w(crate::id("lage"))[k].as_object().unwrap() { a[idx(n)] = zahl(v); } a };
        Cfg {
            rate, profil, werte,
            exp: zahl(&w(crate::id("verteilung"))["exponent"]), fmin: zahl(&w(crate::id("verteilung"))["faktor_min"]), fmax: zahl(&w(crate::id("verteilung"))["faktor_max"]), spielweise: zahl(&w(crate::id("verteilung"))["spielweise"]),
            erfolg, arten,
            xg_sd: zahl(&c["xg_streuung"]), xg_min: zahl(&c["xg_min"]), xg_max: zahl(&c["xg_max"]), tor_je10: zahl(&c["aufs_tor_je_10"]),
            k: zahl(&c["psxg_k"]), weitere_min: zahl(&c["weitere_schuesse_min"]), team_sd: zahl(&c["xg_streuung_team"]), vorlage_quote: zahl(&c["vorlage_quote"]), ziel_f: zahl(&c["xg_ziel_faktor"]),
            poss_exp: zahl(&w(crate::id("lage"))["ballbesitz_exp"]), tech_basis: zahl(&w(crate::id("lage"))["technik_basis"]),
            stil_off: stil(crate::id("stil_offensiv")), stil_def: stil(crate::id("stil_defensiv")),
        }
    })
}

/// One player in the match: index, side, minutes, role profile, family values.
struct Sp { p: usize, side: u8, min: f32, prof: [f32; NF], wert: [f32; NF], gk: bool }

fn familienwerte(w: &World, p: usize) -> [f32; NF] {
    let c = cfg();
    let pl = &w.players[p];
    let mut a = [pl.ovr; NF];
    if pl.attrs.len() == 25 { for f in 0..NF { let ix = &c.werte[f]; if !ix.is_empty() { a[f] = ix.iter().map(|&i| pl.attrs[i]).sum::<f32>() / ix.len() as f32; } } }
    a
}

/// The role profile of a player's position group times his own playing style: a fixed factor per player and family
/// (lognormal, mean 1), so players of one position differ the way real ones do, and the same ones stay dribblers or crossers.
fn profil_von(w: &World, p: usize) -> [f32; NF] {
    let c = cfg();
    let gr = crate::scoutbericht::gruppe(&w.players[p].pos);
    let mut prof = c.profil.iter().find(|x| x.0 == gr).map(|x| x.1).unwrap_or([1.0; NF]);
    let mut rng = Rng(w.players[p].id as u64 * 0x2545_F491 + 0x51ED);
    for x in prof.iter_mut() { *x *= (rng.gauss(c.spielweise) - c.spielweise * c.spielweise / 2.0).exp(); }
    prof
}
/// A lognormal xG around `mittel` (mean kept), clamped.
fn xg_ziehen(rng: &mut Rng, mittel: f32) -> f32 {
    let c = cfg();
    (mittel * (rng.gauss(c.xg_sd)).exp() / (c.xg_sd * c.xg_sd / 2.0).exp()).clamp(c.xg_min, c.xg_max)
}
fn binomial(rng: &mut Rng, n: u32, p: f32) -> u32 { (0..n).filter(|_| rng.chance(p)).count() as u32 }

/// The statistics of a played detailed match (both sides). Empty for summary-level matches (no minutes).
/// Goals stay as the match drew them; each becomes a shot with an xG, and shots without a goal fill the team's expected
/// goals, so over a league goals ≈ xG while good finishers beat their xG (they are picked as scorers more often).
pub fn spiel(w: &World, res: &MatchResult, rng: &mut Rng) -> (Vec<(usize, Stat)>, Vec<Schuss>) {
    if res.minutes.is_empty() { return (vec![], vec![]); }
    let mut schuesse: Vec<Schuss> = vec![];
    // when a player was on the pitch: from kick-off or his substitution to the end, his substitution off or a red card
    let fenster = |p: usize| -> (u8, u8) {
        let ein = res.events.iter().find(|e| e.kind == 5 && e.pid2 == Some(p)).map_or(1, |e| e.min.max(1));
        let aus = res.events.iter().filter(|e| (e.kind == 5 || e.kind == 3) && e.pid == p).map(|e| e.min).min().unwrap_or(90);
        (ein, aus.max(ein))
    };
    let c = cfg();
    let side_of = |p: usize| -> u8 {
        if res.xi_h.iter().any(|x| x.1 == p) { 0 } else if res.xi_a.iter().any(|x| x.1 == p) { 1 }
        else { res.events.iter().find(|e| e.kind == 5 && e.pid2 == Some(p)).map_or(0, |e| e.side) }
    };
    let sp: Vec<Sp> = res.minutes.iter().map(|&(p, m)| {
        Sp { p, side: side_of(p), min: m as f32, prof: profil_von(w, p), wert: familienwerte(w, p), gk: w.players[p].pos == "GK" }
    }).collect();
    let mut st: Vec<Stat> = sp.iter().map(|_| vec![0; N]).collect();
    for (i, s) in sp.iter().enumerate() { st[i][MIN] = s.min as u32; st[i][SPIELE] = 1; }
    // minutes-weighted team means per family
    let mittel = |side: u8, f: usize| -> f32 {
        let (mut s, mut n) = (0.0, 0.0);
        for x in sp.iter().filter(|x| x.side == side && !x.gk) { s += x.wert[f] * x.min; n += x.min; }
        if n > 0.0 { s / n } else { 70.0 }
    };
    let mw: [[f32; NF]; 2] = [std::array::from_fn(|f| mittel(0, f)), std::array::from_fn(|f| mittel(1, f))];
    let anteil = |x: &Sp, f: usize| -> f32 {
        let m = mw[x.side as usize][f].max(1.0);
        let r = if f == F_BALLV { m / x.wert[f].max(1.0) } else { x.wert[f] / m };
        x.prof[f] * r.powf(c.exp).clamp(c.fmin, c.fmax) * x.min / 90.0
    };
    let summe = |side: u8, f: usize| -> f32 { sp.iter().filter(|x| x.side == side).map(|x| anteil(x, f)).sum::<f32>().max(1e-6) };
    let poss = [res.possession.clamp(20, 80) as f32 / 50.0, (100 - res.possession.clamp(20, 80)) as f32 / 50.0];
    let stil = [w.clubs[res.h].style, w.clubs[res.a].style];
    let tech = [(mw[0][F_PAESSE] / c.tech_basis).min(1.1), (mw[1][F_PAESSE] / c.tech_basis).min(1.1)];
    let erfolg = |f: usize, v: f32, gegen: f32| -> f32 { let (b, a, bb) = c.erfolg[f].unwrap(); (b + a * (v - 70.0) / 10.0 + bb * (v - gegen) / 10.0).clamp(0.05, 0.97) };
    // mass actions, once per match: each player Poisson(team rate × his share)
    for side in 0..2u8 {
        let s = side as usize;
        let o = 1 - s;
        for f in [F_PAESSE, F_PROGP, F_PROGL, F_DRIBB, F_FLANK, F_STRAF, F_TACKL, F_ABGEF, F_KLAER, F_FOULS, F_BALLV] {
            let mut rate = c.rate[f];
            rate *= match f {
                F_PAESSE | F_PROGP => poss[s].powf(c.poss_exp),
                F_PROGL | F_DRIBB | F_STRAF => poss[s].sqrt(),
                F_TACKL | F_ABGEF | F_KLAER => poss[o],
                _ => 1.0,
            };
            if f == F_PAESSE { rate *= 0.8 + 0.2 * tech[s]; }
            rate *= match stil[s] { 1 => c.stil_off[f], -1 => c.stil_def[f], _ => 1.0 };
            let sum = summe(side, f);
            for (i, x) in sp.iter().enumerate().filter(|(_, x)| x.side == side) {
                let n = rng.poisson(rate * anteil(x, f) / sum);
                if n == 0 { continue; }
                let v = x.wert[f];
                match f {
                    F_PAESSE => { st[i][PAESSE] += n; st[i][PAESSE_OK] += binomial(rng, n, erfolg(F_PAESSE, v, mw[o][F_TACKL])); }
                    F_PROGP => { st[i][PROG_PAESSE] += n; st[i][PAESSE] += n; st[i][PAESSE_OK] += binomial(rng, n, erfolg(F_PROGP, v, mw[o][F_ABGEF])); }
                    F_PROGL => st[i][PROG_LAEUFE] += n,
                    F_DRIBB => { st[i][DRIBB] += n; st[i][DRIBB_OK] += binomial(rng, n, erfolg(F_DRIBB, v, mw[o][F_TACKL])); }
                    F_FLANK => { st[i][FLANKEN] += n; st[i][FLANKEN_OK] += binomial(rng, n, erfolg(F_FLANK, v, mw[o][F_KLAER])); }
                    F_STRAF => st[i][STRAFRAUM] += n,
                    F_TACKL => { st[i][TACKL] += n; st[i][TACKL_OK] += binomial(rng, n, erfolg(F_TACKL, v, mw[o][F_DRIBB])); }
                    F_ABGEF => st[i][ABGEF] += n,
                    F_KLAER => st[i][KLAER] += n,
                    F_FOULS => st[i][FOULS] += n,
                    _ => st[i][BALLVERL] += n,
                }
            }
        }
    }
    // aerial duels are shared: every duel has a winner on one side and a loser on the other
    let duelle = rng.poisson(2.0 * c.rate[F_KOPF] * (1.2 - 0.2 * (tech[0] + tech[1]) / 2.0));
    let gew = |side: u8| -> Vec<f32> { sp.iter().map(|x| if x.side == side { anteil(x, F_KOPF) } else { 0.0 }).collect() };
    let (g0, g1) = (gew(0), gew(1));
    if g0.iter().sum::<f32>() > 0.0 && g1.iter().sum::<f32>() > 0.0 {
        for _ in 0..duelle {
            let (i, j) = (rng.weighted(&g0), rng.weighted(&g1));
            st[i][KOPF] += 1; st[j][KOPF] += 1;
            let p = erfolg(F_KOPF, sp[i].wert[F_KOPF], sp[j].wert[F_KOPF]);
            if rng.chance(p) { st[i][KOPF_OK] += 1; } else { st[j][KOPF_OK] += 1; }
        }
    }
    // cards from the match events
    for e in &res.events { if let Some(i) = sp.iter().position(|x| x.p == e.pid) { match e.kind { 2 => st[i][GELB] += 1, 3 => st[i][ROT] += 1, 1 => st[i][EIGENTORE] += 1, _ => {} } } }
    // the goalkeeper of each side: the keeper with most minutes
    let tw = |side: u8| sp.iter().enumerate().filter(|(_, x)| x.side == side && x.gk).max_by(|a, b| a.1.min.partial_cmp(&b.1.min).unwrap()).map(|(i, _)| i);
    let keeper = [tw(0), tw(1)];
    // chances: the goals first, then the shots without a goal until the side's expected goals are reached
    for side in 0..2u8 {
        let s = side as usize;
        let o = 1 - s;
        let ziel = c.ziel_f * (if side == 0 { res.lh } else { res.la }) * (1.0 + rng.gauss(c.team_sd)).max(0.2);
        let mut xg_summe = 0.0f32;
        let wahl = |rng: &mut Rng, f: usize, ohne: Option<usize>| -> Option<usize> {
            let wv: Vec<f32> = sp.iter().enumerate().map(|(i, x)| if x.side == side && Some(i) != ohne && !x.gk { anteil(x, f) } else { 0.0 }).collect();
            (wv.iter().sum::<f32>() > 0.0).then(|| rng.weighted(&wv))
        };
        for e in res.events.iter().filter(|e| e.side == side && e.kind == 0) {
            let Some(i) = sp.iter().position(|x| x.p == e.pid) else { continue };
            // the kind of chance by its share, its xG and how well it suits the scorer
            let wa: Vec<f32> = c.arten.iter().map(|a| a.anteil * a.xg * sp[i].prof[a.schuetze].max(0.05)).collect();
            let art = &c.arten[rng.weighted(&wa)];
            // a goal tends to come from a better chance: of three draws the one picked by its size
            let k: Vec<f32> = (0..3).map(|_| xg_ziehen(rng, art.xg)).collect();
            let xg = k[rng.weighted(&k)];
            xg_summe += xg;
            st[i][SCHUESSE] += 1; st[i][AUFS_TOR] += 1; st[i][TORE] += 1; st[i][XG] += (xg * 1000.0) as u32;
            schuesse.push(Schuss { min: e.min, side, pid: e.pid, xg: (xg * 1000.0) as u16, aus: TOR });
            if let Some(j) = e.pid2.and_then(|a| sp.iter().position(|x| x.p == a)) { st[j][VORLAGEN] += 1; st[j][SCHLUESSEL] += 1; st[j][XA] += (xg * 1000.0) as u32; }
            if let Some(g) = keeper[o] { st[g][GEGENTORE] += 1; st[g][PSXG] += ((xg * c.k).min(0.98) * 1000.0) as u32; }
        }
        // own goals of the other side count against this side's keeper
        if let Some(g) = keeper[s] { st[g][GEGENTORE] += res.events.iter().filter(|e| e.side as usize == o && e.kind == 1).count() as u32; }
        let mindest = rng.poisson(c.weitere_min);
        let wa: Vec<f32> = c.arten.iter().map(|a| a.anteil).collect();
        let mut n = 0;
        while (xg_summe < ziel || n < mindest) && n < 40 {
            n += 1;
            let art = &c.arten[rng.weighted(&wa)];
            let Some(i) = wahl(rng, art.schuetze, None) else { break };
            let xg = xg_ziehen(rng, art.xg);
            xg_summe += xg;
            st[i][SCHUESSE] += 1; st[i][XG] += (xg * 1000.0) as u32;
            let (ein, aus) = fenster(sp[i].p);
            let min = rng.int(ein as i32, aus as i32) as u8;
            schuesse.push(Schuss { min, side, pid: sp[i].p, xg: (xg * 1000.0) as u16, aus: DANEBEN });
            if let Some(vf) = art.vorbereiter { if rng.chance(c.vorlage_quote) { if let Some(j) = wahl(rng, vf, Some(i)) { st[j][SCHLUESSEL] += 1; st[j][XA] += (xg * 1000.0) as u32; } } }
            if rng.chance(art.block) {
                st[i][GEBLOCKT] += 1;
                let wb: Vec<f32> = sp.iter().map(|x| if x.side as usize == o && !x.gk { x.prof[F_BLOCKS] * x.min / 90.0 } else { 0.0 }).collect();
                if wb.iter().sum::<f32>() > 0.0 { let b = rng.weighted(&wb); st[b][BLOCKS] += 1; }
                schuesse.last_mut().unwrap().aus = GEBLOCKT_S;
                continue;
            }
            let fin = sp[i].wert[F_SCHUSS];
            if rng.chance((art.tor + c.tor_je10 * (fin - 70.0) / 10.0).clamp(0.1, 0.8)) {
                st[i][AUFS_TOR] += 1;
                schuesse.last_mut().unwrap().aus = GEHALTEN;
                if let Some(g) = keeper[o] { st[g][PARADEN] += 1; st[g][PSXG] += ((xg * c.k).min(0.98) * 1000.0) as u32; }
            }
        }
    }
    schuesse.sort_by_key(|x| x.min);
    (sp.iter().map(|x| x.p).zip(st).collect(), schuesse)
}

/// Konzept 5.2: a player of a summary-level league has no match trace; his season is projected when someone looks:
/// minutes from his place in the squad (regular 88 % of the matches, rotation 50 %, else 15 %) and the team's matches,
/// every number from his role profile and values as a rate per 90, with a fixed noise per player and season (the same
/// number every time, larger with few minutes). Quotes from the success model against an average opponent.
pub fn hochrechnen(w: &World, p: usize) -> Stat { hochrechnen_saison(w, p, None) }
/// Konzept 5.4: the season before the career, estimated the same way over a whole season (no real figures exist), so
/// scouting has numbers from day one. Never stored: drawn with a fixed noise each time it is looked at.
pub fn vorsaison_geschaetzt(w: &World, p: usize) -> Stat {
    let l = if w.players[p].team >= 0 { w.clubs[w.players[p].team as usize].league } else { return vec![0; N] };
    if l >= w.leagues.len() { return vec![0; N]; }
    let n = w.leagues[l].teams.len().saturating_sub(1) as f32 * 2.0;
    hochrechnen_saison(w, p, Some(n))
}
fn hochrechnen_saison(w: &World, p: usize, spiele_vorgabe: Option<f32>) -> Stat {
    let c = cfg();
    let pl = &w.players[p];
    let mut s = vec![0u32; N];
    if pl.team < 0 { return s; }
    let club = pl.team as usize;
    let l = w.clubs[club].league;
    if l >= w.leagues.len() { return s; }
    let spiele = spiele_vorgabe.unwrap_or_else(|| w.leagues[l].fixtures.iter().filter(|f| f.hg >= 0 && (f.h == club || f.a == club)).count() as f32);
    let mut kader: Vec<usize> = w.clubs[club].squad.iter().copied().filter(|&q| w.players[q].youth == 0).collect();
    kader.sort_by(|a, b| w.players[*b].ovr.partial_cmp(&w.players[*a].ovr).unwrap());
    let rang = kader.iter().position(|&q| q == p).unwrap_or(kader.len());
    // one keeper plays: the best; the others hardly
    let quote = if pl.pos == "GK" { if kader.iter().find(|&&q| w.players[q].pos == "GK") == Some(&p) { 0.95 } else { 0.05 } } else if rang < 12 { 0.88 } else if rang < 17 { 0.5 } else { 0.15 };
    let einsaetze = (spiele * quote).round();
    let min = einsaetze * if quote >= 0.8 { 84.0 } else { 55.0 };
    if min <= 0.0 { return s; }
    let prof = profil_von(w, p);
    let wert = familienwerte(w, p);
    let mittel = 70.0f32.max(crate::statistik::liga_staerke(w, l) - 2.0);
    let wf = |f: usize| -> f32 { let r = if f == F_BALLV { mittel / wert[f].max(1.0) } else { wert[f] / mittel }; r.powf(c.exp).clamp(c.fmin, c.fmax) };
    let mut rng = Rng(pl.id as u64 * 0x9E37 + w.season as u64 * 0x85EB + spiele_vorgabe.is_some() as u64 * 0x5151);
    let rausch = |rng: &mut Rng| (1.0 + rng.gauss(0.25 / (min / 90.0).sqrt().max(0.3))).max(0.0);
    let je90 = |f: usize| c.rate[f] * prof[f] * wf(f) / 10.5;
    let mut zahl = |rng: &mut Rng, f: usize| -> u32 { (je90(f) * min / 90.0 * rausch(rng)).round() as u32 };
    let erfolg = |f: usize| -> f32 { let (b, a, _) = c.erfolg[f].unwrap(); (b + a * (wert[f] - 70.0) / 10.0).clamp(0.05, 0.97) };
    s[MIN] = min as u32;
    s[SPIELE] = einsaetze as u32;
    s[PAESSE] = zahl(&mut rng, F_PAESSE); s[PAESSE_OK] = (s[PAESSE] as f32 * erfolg(F_PAESSE)) as u32;
    s[PROG_PAESSE] = zahl(&mut rng, F_PROGP); s[PROG_LAEUFE] = zahl(&mut rng, F_PROGL);
    s[DRIBB] = zahl(&mut rng, F_DRIBB); s[DRIBB_OK] = (s[DRIBB] as f32 * erfolg(F_DRIBB)) as u32;
    s[FLANKEN] = zahl(&mut rng, F_FLANK); s[FLANKEN_OK] = (s[FLANKEN] as f32 * erfolg(F_FLANK)) as u32;
    s[STRAFRAUM] = zahl(&mut rng, F_STRAF);
    s[TACKL] = zahl(&mut rng, F_TACKL); s[TACKL_OK] = (s[TACKL] as f32 * erfolg(F_TACKL)) as u32;
    s[ABGEF] = zahl(&mut rng, F_ABGEF); s[KLAER] = zahl(&mut rng, F_KLAER); s[FOULS] = zahl(&mut rng, F_FOULS); s[BALLVERL] = zahl(&mut rng, F_BALLV);
    s[KOPF] = zahl(&mut rng, F_KOPF); s[KOPF_OK] = (s[KOPF] as f32 * erfolg(F_KOPF)) as u32;
    // shots and xG: the team's expected goals spread by the shot profile (an eleven sums to about 10,5 profile units)
    let team_xg = 1.35;
    let xg = team_xg * prof[F_SCHUSS] * wf(F_SCHUSS) / 10.5 * min / 90.0 * rausch(&mut rng);
    s[XG] = (xg * 1000.0) as u32;
    s[SCHUESSE] = (xg / 0.105).round() as u32;
    s[AUFS_TOR] = (s[SCHUESSE] as f32 * (0.36 + c.tor_je10 * (wert[F_SCHUSS] - 70.0) / 10.0).clamp(0.2, 0.6)) as u32;
    s[TORE] = pl.goals.max((xg * (1.0 + 0.04 * (wert[F_SCHUSS] - 70.0) / 10.0) * rausch(&mut rng)).round() as u16) as u32;
    let xa = team_xg * prof[F_SCHLUESSEL] * wf(F_SCHLUESSEL) / 10.5 * 0.75 * min / 90.0 * rausch(&mut rng);
    s[XA] = (xa * 1000.0) as u32;
    s[SCHLUESSEL] = (xa / 0.105).round() as u32;
    s[VORLAGEN] = pl.assists.max((xa * rausch(&mut rng)).round() as u16) as u32;
    s[GELB] = if spiele_vorgabe.is_some() { (min / 90.0 * 0.18 * prof[F_FOULS] * rausch(&mut rng)).round() as u32 } else { pl.yellow as u32 };
    if spiele_vorgabe.is_some() {
        s[TORE] = (xg * (1.0 + 0.04 * (wert[F_SCHUSS] - 70.0) / 10.0) * rausch(&mut rng)).round() as u32;
        s[VORLAGEN] = (xa * rausch(&mut rng)).round() as u32;
    }
    if pl.pos == "GK" {
        let gegen = 1.35 * min / 90.0;
        s[GEGENTORE] = (gegen * rausch(&mut rng)).round() as u32;
        s[PSXG] = (gegen * 1000.0) as u32;
        s[PARADEN] = (gegen * 2.1 * rausch(&mut rng)).round() as u32;
    }
    s
}

/// Imported careers (altimport.rs, Fabian 24.11.): the old matches left season totals (games, goals, assists, grades)
/// but no trace. The missing games in a detailed league get numbers like the projection (role profile, values, fixed
/// noise), scaled to the real games, with the real goals and assists. Returns the number of players filled.
pub fn nachtragen(w: &mut World) -> usize {
    let mut n = 0;
    for p in 0..w.players.len() {
        let mut neu = false;
        for i in 0..w.players[p].profil.ligen.len() {
            let ls = w.players[p].profil.ligen[i].clone();
            let l = ls.l as usize;
            if l >= w.leagues.len() || w.leagues[l].detail > 1 { continue; }
            let fehlt = (ls.apps as u32).saturating_sub(get(&ls.st, SPIELE));
            if fehlt == 0 { continue; }
            let voll = w.leagues[l].teams.len().saturating_sub(1) as f32 * 2.0;
            let mut s = hochrechnen_saison(w, p, Some(voll));
            if s.len() < N { s = vec![0; N]; }
            let f = fehlt as f32 / get(&s, SPIELE).max(1) as f32;
            for x in s.iter_mut() { *x = (*x as f32 * f).round() as u32; }
            if s[MIN] == 0 { s[MIN] = fehlt * 70; }
            s[SPIELE] = fehlt;
            s[TORE] = (ls.goals as u32).saturating_sub(get(&ls.st, TORE));
            s[VORLAGEN] = (ls.assists as u32).saturating_sub(get(&ls.st, VORLAGEN));
            s[SCHUESSE] = s[SCHUESSE].max(s[TORE]);
            s[AUFS_TOR] = s[AUFS_TOR].clamp(s[TORE], s[SCHUESSE]);
            for (ok, alle) in [(PAESSE_OK, PAESSE), (DRIBB_OK, DRIBB), (FLANKEN_OK, FLANKEN), (TACKL_OK, TACKL), (KOPF_OK, KOPF)] { s[ok] = s[ok].min(s[alle]); }
            add(&mut w.players[p].profil.ligen[i].st, &s);
            add(&mut w.players[p].profil.saison, &s);
            neu = true;
        }
        n += neu as usize;
    }
    n
}

/// Konzept 2.5: Einfluss per 90 in goals — danger created (npxG, xA, progression) plus danger prevented (won duels,
/// interceptions, blocks, saves above expectation) minus danger given away (ball losses, fouls). None under 90 minutes.
pub fn einfluss90(s: &Stat) -> Option<f32> {
    let m = get(s, MIN) as f32;
    if m < 90.0 { return None; }
    let g = |i| get(s, i) as f32;
    let e = |k: &str| crate::regeln::f("statistik", &format!("einfluss.{k}"));
    let wert = g(XG) / 1000.0 + e(crate::id("xa")) * g(XA) / 1000.0 + e("prog_pass") * g(PROG_PAESSE) + e("prog_lauf") * g(PROG_LAEUFE)
        + e(crate::id("tackling")) * g(TACKL_OK) + e(crate::id("abgefangen")) * g(ABGEF) + e(crate::id("block")) * g(BLOCKS)
        + (g(PSXG) / 1000.0 - g(GEGENTORE)) - e(crate::id("ballverlust")) * g(BALLVERL) - e(crate::id("foul")) * g(FOULS);
    Some(wert * 90.0 / m)
}

/// A count per 90 minutes (xG-type values in goals); None under 90 minutes.
pub fn pro90(s: &Stat, i: usize) -> Option<f32> { je90(s, i) }
fn je90(s: &Stat, i: usize) -> Option<f32> { let m = get(s, MIN) as f32; (m >= 90.0).then(|| get(s, i) as f32 * if i == XG || i == XA || i == PSXG { 0.001 } else { 1.0 } * 90.0 / m) }
fn quote(s: &Stat, a: usize, b: usize, mindest: u32) -> Option<f32> { (get(s, b) >= mindest).then(|| get(s, a) as f32 / get(s, b) as f32 * 100.0) }
/// One key number of a position group: text key (ui.stat.*), its value per 90 or as a quote, whether it is a quote.
pub type Kern = (&'static str, fn(&Stat) -> Option<f32>, bool);
/// Konzept Spielerstatistiken 7.1/8.2: the six numbers that count for a position group, the core one first.
pub fn kernzahlen(gr: &str) -> Vec<Kern> {
    match gr {
        "tw" => vec![("psxg_minus_gt", |s| je90(s, PSXG).map(|x| x - je90(s, GEGENTORE).unwrap_or(0.0)), false), (crate::id("paraden"), |s| je90(s, PARADEN), false),
            ("paraden_quote", |s| (get(s, PARADEN) + get(s, GEGENTORE) >= 15).then(|| get(s, PARADEN) as f32 / (get(s, PARADEN) + get(s, GEGENTORE)) as f32 * 100.0), true),
            (crate::id("passquote"), |s| quote(s, PAESSE_OK, PAESSE, 30), true), (crate::id("einfluss"), einfluss90, false)],
        "iv" => vec![("kopfball_quote", |s| quote(s, KOPF_OK, KOPF, 30), true), (crate::id("abgefangen"), |s| je90(s, ABGEF), false), (crate::id("tacklings"), |s| je90(s, TACKL), false),
            (crate::id("klaerungen"), |s| je90(s, KLAER), false), ("prog_paesse", |s| je90(s, PROG_PAESSE), false), (crate::id("einfluss"), einfluss90, false)],
        "av" => vec![("prog_laeufe", |s| je90(s, PROG_LAEUFE), false), (crate::id("flanken"), |s| je90(s, FLANKEN), false), (crate::id("xa"), |s| je90(s, XA), false),
            (crate::id("tacklings"), |s| je90(s, TACKL), false), (crate::id("abgefangen"), |s| je90(s, ABGEF), false), (crate::id("einfluss"), einfluss90, false)],
        "dm" | "zm" => vec![("prog_paesse", |s| je90(s, PROG_PAESSE), false), (crate::id("passquote"), |s| quote(s, PAESSE_OK, PAESSE, 30), true), (crate::id("paesse"), |s| je90(s, PAESSE), false),
            (crate::id("tacklings"), |s| je90(s, TACKL), false), (crate::id("abgefangen"), |s| je90(s, ABGEF), false), (crate::id("einfluss"), einfluss90, false)],
        "om" | "fl" => vec![(crate::id("xa"), |s| je90(s, XA), false), (crate::id("schluesselpaesse"), |s| je90(s, SCHLUESSEL), false), (crate::id("xg"), |s| je90(s, XG), false),
            (crate::id("dribblings"), |s| je90(s, DRIBB), false), ("prog_laeufe", |s| je90(s, PROG_LAEUFE), false), (crate::id("einfluss"), einfluss90, false)],
        _ => vec![(crate::id("xg"), |s| je90(s, XG), false), (crate::id("schuesse"), |s| je90(s, SCHUESSE), false), (crate::id("strafraum"), |s| je90(s, STRAFRAUM), false),
            (crate::id("xa"), |s| je90(s, XA), false), ("kopfball_quote", |s| quote(s, KOPF_OK, KOPF, 20), true), (crate::id("einfluss"), einfluss90, false)],
    }
}
/// The comparison group of a position group in a league: league figures of the detailed matches (at least 900 minutes,
/// earlier in the season 45 % of the most-used player's), or the projections of a results-only league.
pub fn liga_vergleich(w: &World, l: usize, gr: &str) -> Vec<Stat> { liga_vergleich_spieler(w, l, gr).0.into_iter().map(|x| x.1).collect() }
/// The same group with the player of each figure, and the minutes it took to be in it.
pub fn liga_vergleich_spieler(w: &World, l: usize, gr: &str) -> (Vec<(usize, Stat)>, u32) {
    if l >= w.leagues.len() { return (vec![], 0); }
    let alle: Vec<(usize, Stat)> = if w.leagues[l].detail == 2 {
        w.leagues[l].teams.iter().flat_map(|&t| w.clubs[t].squad.iter().copied()).filter(|&q| crate::scoutbericht::gruppe(&w.players[q].pos) == gr).map(|q| (q, hochrechnen(w, q))).collect()
    } else {
        w.players.iter().enumerate().filter(|(_, q)| crate::scoutbericht::gruppe(&q.pos) == gr).filter_map(|(i, q)| q.profil.ligen.iter().find(|x| x.l as usize == l).map(|x| (i, x.st.clone()))).collect()
    };
    let hoechst = alle.iter().map(|s| get(&s.1, MIN)).max().unwrap_or(0);
    let schwelle = 900.min((hoechst as f32 * 0.45) as u32).max(90);
    (alle.into_iter().filter(|s| get(&s.1, MIN) >= schwelle).collect(), schwelle)
}
/// Percentile 0–100 of `wert` among the group's values (`weniger`: fewer is better). None with fewer than five.
pub fn perzentil(gruppe: &[Stat], f: fn(&Stat) -> Option<f32>, wert: f32, weniger: bool) -> Option<f32> {
    let v: Vec<f32> = gruppe.iter().filter_map(f).collect();
    if v.len() < 5 { return None; }
    let unter = v.iter().filter(|&&x| if weniger { x > wert } else { x < wert }).count() as f32 + 0.5 * v.iter().filter(|&&x| x == wert).count() as f32;
    Some((unter / v.len() as f32 * 100.0).clamp(0.0, 100.0))
}
/// A player's season figures as the scouts see them: his league figures, or projected for a results-only league.
pub fn saison_von(w: &World, p: usize) -> (Stat, bool) {
    let pl = &w.players[p];
    let l = if pl.team >= 0 { w.clubs[pl.team as usize].league } else { usize::MAX };
    if l < w.leagues.len() && w.leagues[l].detail == 2 && pl.profil.saison.is_empty() { return (hochrechnen(w, p), true); }
    (if l < w.leagues.len() { pl.in_liga(l).st } else { pl.profil.saison.clone() }, false)
}

/// Konzept 7.3 "Finde einen wie …": the players whose key numbers (per 90, scaled by the spread of all candidates) lie
/// closest to the model's, with similarity 0–100. Only candidates with the same position group and at least 450 minutes.
pub fn aehnliche(w: &World, vorbild: usize, kandidaten: &[usize], n: usize) -> Vec<(usize, f32)> {
    let gr = crate::scoutbericht::gruppe(&w.players[vorbild].pos);
    let kern = kernzahlen(gr);
    let vek = |p: usize| -> Option<Vec<f32>> {
        let (st, _) = saison_von(w, p);
        if get(&st, MIN) < 450 { return None; }
        kern.iter().map(|k| k.1(&st)).collect()
    };
    let Some(v0) = vek(vorbild) else { return vec![] };
    let alle: Vec<(usize, Vec<f32>)> = kandidaten.iter().copied().filter(|&q| q != vorbild && crate::scoutbericht::gruppe(&w.players[q].pos) == gr).filter_map(|q| vek(q).map(|v| (q, v))).collect();
    if alle.len() < 3 { return vec![]; }
    let sd: Vec<f32> = (0..v0.len()).map(|i| { let m = alle.iter().map(|x| x.1[i]).sum::<f32>() / alle.len() as f32; (alle.iter().map(|x| (x.1[i] - m).powi(2)).sum::<f32>() / alle.len() as f32).sqrt().max(1e-3) }).collect();
    let mut r: Vec<(usize, f32)> = alle.iter().map(|(q, v)| {
        let d = (0..v.len()).map(|i| ((v[i] - v0[i]) / sd[i]).powi(2) * if i == 0 { 2.0 } else { 1.0 }).sum::<f32>().sqrt();
        (*q, (100.0 * (-d / 3.0).exp()).clamp(0.0, 100.0))
    }).collect();
    r.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    r.truncate(n);
    r
}

/// A league's strength: the mean level of its clubs (Konzept 6.3 uses the regulars' role strength; the club level is its
/// cheap stand-in). shortcut: club level instead of regulars' role strength, switch once a weekly league figure exists.
pub fn liga_staerke(w: &World, l: usize) -> f32 {
    let t = &w.leagues[l].teams;
    if t.is_empty() { return 70.0; }
    t.iter().map(|&c| w.clubs[c].level).sum::<f32>() / t.len() as f32
}
/// Konzept 6.3: a value from league `von` as it would be in league `nach`. `art`: 0 attack (β 0,25), 1 progression (0,18),
/// 2 defence (0,08), 3 goalkeeper (0,15), 4 a quote in percent (−3 points per 10 strength points).
pub fn umrechnen(w: &World, wert: f32, von: usize, nach: usize, art: u8) -> f32 {
    if von >= w.leagues.len() || nach >= w.leagues.len() || von == nach { return wert; }
    let d = liga_staerke(w, von) - liga_staerke(w, nach);
    let beta = |k: &str| crate::regeln::f("statistik", &format!("umrechnung.{k}"));
    match art {
        4 => (wert + beta(crate::id("quote_je_10")) * d / 10.0).clamp(0.0, 100.0),
        _ => wert * (beta([crate::id("angriff"), crate::id("aufbau"), crate::id("abwehr"), crate::id("torwart")][art as usize]) * d / 10.0).exp(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nachtragen_aus_summen() {
        let mut w = crate::world::testwelt(7);
        let p = w.clubs[0].squad.iter().copied().find(|&q| w.players[q].pos == "CF").unwrap();
        let l = w.clubs[0].league;
        w.leagues[l].detail = 0;
        w.players[p].profil.ligen.push(crate::world::LigaStat { l: l as u16, apps: 20, goals: 9, assists: 3, rsum: 140.0, rn: 20, st: vec![] });
        assert_eq!(nachtragen(&mut w), 1);
        let s = &w.players[p].profil.ligen[0].st;
        assert!(get(s, SPIELE) == 20 && get(s, TORE) == 9 && get(s, VORLAGEN) == 3 && get(s, SCHUESSE) >= 9 && get(s, PAESSE) > 0 && get(s, MIN) > 900, "{s:?}");
        assert_eq!(&w.players[p].profil.saison, s);
        assert_eq!(nachtragen(&mut w), 0, "only once");
    }
    /// Fabian 24.11.: club figures per competition: the team's sums, the opponent's sums mirror the other club.
    #[test]
    fn vereinsstatistik() {
        let mut w = crate::world::testwelt(11);
        let r = w.play(0, 1, 0, None, None);
        w.wettbewerb_zaehlen(0, &r);
        if r.stat.is_empty() { return; }
        let v = |c: usize| w.vstat.iter().find(|x| x.club == c && x.l == 0).unwrap().clone();
        let (h, a) = (v(0), v(1));
        assert_eq!((h.spiele, a.spiele), (1, 1));
        assert_eq!(get(&h.eigen, TORE), r.hg as u32);
        assert_eq!(h.eigen, a.gegen, "the opponent's sums are the other club's own");
        assert_eq!(h.besitz + a.besitz, 100);
        assert_eq!(h.gross, a.gross_gegen);
        assert_eq!(get(&h.eigen, XG), r.summe(0, XG));
    }
    #[test]
    fn aehnlichkeit() {
        let mut w = crate::world::testwelt(11);
        for k in 0..120 { let (h, a) = (k % 18, (k * 5 + 1) % 18); if h != a { let r = w.play(h, a, 0, None, None); w.apply_stats(&r); for (p, st) in &r.stat { let l = w.clubs[w.players[*p].team.max(0) as usize].league as u16; let x = &mut w.players[*p].profil.ligen; if x.is_empty() { x.push(crate::world::LigaStat { l, ..Default::default() }); } add(&mut x[0].st, st); } } }
        let alle: Vec<usize> = (0..w.players.len()).collect();
        let st = alle.iter().copied().find(|&p| w.players[p].pos == "CF" && get(&saison_von(&w, p).0, MIN) >= 450).unwrap();
        let r = aehnliche(&w, st, &alle, 10);
        assert!(!r.is_empty() && r.iter().all(|&(q, s)| crate::scoutbericht::gruppe(&w.players[q].pos) == "st" && (0.0..=100.0).contains(&s)), "a striker's look-alikes are strikers");
    }
    #[test]
    fn hochrechnung_ist_fest() {
        let w = crate::world::testwelt(3);
        let p = w.clubs[0].squad[3];
        assert_eq!(hochrechnen(&w, p), hochrechnen(&w, p), "the same number every time");
    }
    #[test]
    fn umrechnung_auf_andere_liga() {
        let w = crate::world::testwelt(3);
        let n = w.leagues.len();
        let (stark, schwach) = (0..n).fold((0, 0), |(a, b), l| (if liga_staerke(&w, l) > liga_staerke(&w, a) { l } else { a }, if liga_staerke(&w, l) < liga_staerke(&w, b) { l } else { b }));
        assert_eq!(umrechnen(&w, 0.5, stark, stark, 0), 0.5);
        if liga_staerke(&w, stark) > liga_staerke(&w, schwach) {
            assert!(umrechnen(&w, 0.5, schwach, stark, 0) < 0.5, "numbers from a weaker league count less");
            assert!(umrechnen(&w, 80.0, schwach, stark, 4) < 80.0);
        }
    }
    #[test]
    fn stimmigkeit_und_liga_xg() {
        let mut w = crate::world::testwelt(11);
        let (mut tore, mut xg, mut schuesse, mut spiele) = (0u32, 0u32, 0u32, 0u32);
        for k in 0..400 {
            let (h, a) = (k % 18, (k * 7 + 3) % 18);
            if h == a { continue; }
            let res = w.play(h, a, 0, None, None);
            assert!(!res.stat.is_empty());
            assert_eq!(res.schuesse.iter().filter(|x| x.aus == TOR).count(), res.events.iter().filter(|e| e.kind == 0).count(), "every goal is a shot");
            for side in 0..2u8 {
                let seite = |p: usize| if res.xi_h.iter().any(|x| x.1 == p) { 0 } else if res.xi_a.iter().any(|x| x.1 == p) { 1 } else { res.events.iter().find(|e| e.kind == 5 && e.pid2 == Some(p)).map_or(0, |e| e.side) };
                let eigen: Vec<&(usize, Stat)> = res.stat.iter().filter(|(p, _)| seite(*p) == side).collect();
                let t: u32 = eigen.iter().map(|x| get(&x.1, TORE)).sum();
                let og = res.events.iter().filter(|e| e.side == side && e.kind == 1).count() as u32;
                let team = if side == 0 { res.hg } else { res.ag } as u32;
                assert_eq!(t + og, team, "goals of a side = its players' goals + own goals of the other side");
                let v: u32 = eigen.iter().map(|x| get(&x.1, VORLAGEN)).sum();
                assert!(v <= t);
                let (s, at) = (eigen.iter().map(|x| get(&x.1, SCHUESSE)).sum::<u32>(), eigen.iter().map(|x| get(&x.1, AUFS_TOR)).sum::<u32>());
                assert!(at <= s && t <= at);
                // the other keeper: saves + goals = shots on target of this side (own goals extra)
                let gegner: Vec<&(usize, Stat)> = res.stat.iter().filter(|(p, _)| w.players[*p].pos == "GK" && !eigen.iter().any(|e| e.0 == *p)).collect();
                if let Some(g) = gegner.iter().max_by_key(|x| get(&x.1, MIN)) { assert_eq!(get(&g.1, PARADEN) + get(&g.1, GEGENTORE), at + og); }
                tore += t; xg += eigen.iter().map(|x| get(&x.1, XG)).sum::<u32>(); schuesse += s;
            }
            // aerial duels are shared
            let kopf: u32 = res.stat.iter().map(|x| get(&x.1, KOPF)).sum();
            let gew: u32 = res.stat.iter().map(|x| get(&x.1, KOPF_OK)).sum();
            assert_eq!(kopf, 2 * gew);
            spiele += 1;
        }
        let q = tore as f32 / (xg as f32 / 1000.0);
        let s = schuesse as f32 / (2 * spiele) as f32;
        eprintln!("Tore/xG {q:.3}, Schüsse je Team {s:.1}");
        assert!((0.9..=1.1).contains(&q), "goals / xG {q:.2}");
        assert!((9.0..=17.0).contains(&s), "shots per team {s:.1}");
    }
}
#[cfg(test)]
mod noten_mittel {
    #[test]
    #[ignore]
    fn notenschnitt_messen() {
        let mut w = crate::world::testwelt(5);
        let (mut s, mut n) = (0.0, 0);
        for k in 0..300 { let r = w.play(k % 18, (k + 5) % 18, 0, None, None); for x in &r.ratings { s += x.1; n += 1; } }
        eprintln!("Notenschnitt {:.3}", s / n as f32);
    }
}
