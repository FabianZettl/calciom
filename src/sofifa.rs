//! Real contract data from SoFIFA (Fabian 24.11.): potential, wage, market value, contract end, release clause and loans,
//! taken over once at the start of a career from the German SoFIFA export of the FCFL Stats Converter
//! (`ID;Name;Vollständiger Name;…;Vertrag;…;Leihende;Lohn;Wert;Freigabeklausel;…`).
//! Source (changeable, PAKETE.md): the newest full export in CalcioM's folder `sofifa/` or in the converter's `ea-base` /
//! `ea-updates` folders; the converter is not called, its files are only read.
use crate::trainerbank::norm;
use crate::world::World;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Default, Debug)]
pub struct Zeile {
    pub id: u32, pub name: String, pub voll: String, pub nat: String, pub verein: String, pub vertrag: String,
    pub alter: u8, pub ovr: f32, pub pot: f32, pub leihe_bis: Option<(i32, u32, u32)>, pub lohn: f64, pub wert: f64, pub klausel: f64,
    /// Emotion 1 very calm … 3 normal … 5 explosive (0 unknown); growth profile, see `kurve`.
    pub emotion: u8, pub wachstum: u8,
}

const EMOTION: [&str; 5] = ["Very calm", "Calm", "Normal", "Hot temper", "Explosive"]; // roh
const WACHSTUM: [&str; 5] = ["Explosive Start", "Late Bloomer", "Meteoric Peak", "Consistent Performer", "Complete Technician"]; // roh

/// Factor of the growth profile on the yearly change at this age (EA FC career mode, own figures): growth up to 26, decline after.
/// 1 Explosive Start grows early, 2 Late Bloomer late, 3 Meteoric Peak high and short, 4 Consistent Performer keeps his level longer,
/// 5 Complete Technician declines a little slower; 0 Default = 1.
pub fn kurve(profil: u8, alter: f32) -> f32 {
    let (jung, mitte, spaet, abbau) = match profil { 1 => (1.3, 1.0, 0.7, 1.1), 2 => (0.7, 1.1, 1.4, 0.85), 3 => (1.15, 1.15, 1.0, 1.3), 4 => (1.0, 1.0, 1.0, 0.7), 5 => (1.0, 1.0, 1.0, 0.85), _ => return 1.0 };
    if alter > 26.0 { abbau } else if alter <= 21.0 { jung } else if alter <= 23.0 { mitte } else { spaet }
}

/// Fields of a line split at `;`, with "…" quotes ("" inside a quote is one quote).
fn felder(l: &str) -> Vec<String> {
    let (mut v, mut f, mut q) = (vec![], String::new(), false);
    let mut it = l.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '"' if q && it.peek() == Some(&'"') => { f.push('"'); it.next(); }
            '"' => q = !q,
            ';' if !q => v.push(std::mem::take(&mut f)),
            _ => f.push(c),
        }
    }
    v.push(f);
    v
}

/// "€65K", "€1.2M", "€500" → euros.
pub fn geld(s: &str) -> f64 {
    let t = s.trim().trim_start_matches('€');
    let (z, f) = match t.chars().last() { Some('K') => (&t[..t.len() - 1], 1e3), Some('M') => (&t[..t.len() - 1], 1e6), Some('B') => (&t[..t.len() - 1], 1e9), _ => (t, 1.0) };
    z.parse::<f64>().map(|x| x * f).unwrap_or(0.0)
}

/// "71-1" (live change behind the rating) → 71.
fn zahl(s: &str) -> f32 { s.split(|c: char| !c.is_ascii_digit()).next().and_then(|x| x.parse().ok()).unwrap_or(0.0) }

fn datum(s: &str) -> Option<(i32, u32, u32)> {
    let mut t = s.trim().splitn(3, '.');
    let d = t.next()?.parse().ok()?;
    let m = t.next()?.parse().ok()?;
    Some((t.next()?.get(..4)?.parse().ok()?, m, d))
}

/// The rows of one export; None when it is no full German SoFIFA export (an update without contract columns is left out).
pub fn lesen(text: &str) -> Option<Vec<Zeile>> {
    let mut zeilen = text.lines();
    let kopf = felder(zeilen.next()?.trim_start_matches('\u{feff}'));
    let sp = |n: &str| kopf.iter().position(|k| k == n);
    let s: Vec<usize> = ["ID", "Name", "Vollständiger Name", "Nationalität", "Verein", "Vertrag", "Alter", "Gesamtbewertung", "Potenzial", "Leihende", "Lohn", "Wert", "Freigabeklausel"] // roh
        .iter().map(|n| sp(n)).collect::<Option<_>>()?;
    let (em, wa) = (sp("Emotion"), sp("Growth profile")); // roh
    Some(zeilen.filter_map(|l| {
        let f = felder(l);
        let opt = |i: Option<usize>, liste: &[&str]| i.and_then(|i| f.get(i)).and_then(|x| liste.iter().position(|n| *n == x.trim())).map_or(0, |k| k as u8 + 1);
        let g = |i: usize| f.get(s[i]).map(|x| x.trim()).unwrap_or("");
        Some(Zeile {
            id: g(0).parse().ok()?, name: g(1).into(), voll: g(2).into(), nat: g(3).into(), verein: g(4).into(), vertrag: g(5).into(),
            alter: g(6).parse().unwrap_or(0), ovr: zahl(g(7)), pot: zahl(g(8)), leihe_bis: datum(g(9)),
            lohn: geld(g(10)), wert: geld(g(11)), klausel: geld(g(12)), emotion: opt(em, &EMOTION), wachstum: opt(wa, &WACHSTUM),
        })
    }).collect())
}

/// Folders searched for exports: CalcioM's own `sofifa/` and the converter's data folder.
pub fn ordner() -> Vec<PathBuf> {
    let conv = std::env::var("APPDATA").map(|a| PathBuf::from(a).join("FL-Stats-Converter")) // roh
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share/fl-stats-converter")); // roh
    vec![crate::fl27::datenordner().join(crate::id("sofifa")), conv.join("ea-base"), conv.join("ea-updates")] // roh
}

/// The newest data: every full export found, oldest first (date in the name, else the file's day), newer rows replace older ones per SoFIFA id.
/// (file name of the newest, rows)
pub fn laden_aus(ordner: &[PathBuf]) -> Option<(String, Vec<Zeile>)> {
    let datum_im_namen = |p: &Path| -> (u32, u32, u32) {
        let n = p.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_default();
        let z: Vec<char> = n.chars().collect();
        (0..z.len().saturating_sub(9)).find_map(|i| {
            let s: String = z[i..i + 10].iter().collect();
            if !(s.is_ascii() && s.starts_with("20") && &s[4..5] == "-" && &s[7..8] == "-") { return None; }
            Some((s[0..4].parse().ok()?, s[5..7].parse().ok()?, s[8..10].parse().ok()?))
        }).unwrap_or((0, 0, 0))
    };
    let mut dateien: Vec<_> = ordner.iter().flat_map(|o| std::fs::read_dir(o).into_iter().flatten().flatten().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case(crate::id("csv"))))
        .map(|p| (datum_im_namen(&p), std::fs::metadata(&p).and_then(|m| m.modified()).ok(), p)).collect();
    dateien.sort();
    let mut alle: HashMap<u32, Zeile> = HashMap::new();
    let mut neueste = None;
    for (_, _, p) in dateien {
        let Some(z) = std::fs::read_to_string(&p).ok().and_then(|t| lesen(&t)) else { continue };
        for r in z { alle.insert(r.id, r); }
        neueste = p.file_name().map(|x| x.to_string_lossy().to_string());
    }
    let mut v: Vec<Zeile> = alle.into_values().collect();
    v.sort_by_key(|r| r.id);
    Some((neueste?, v))
}

/// The assignments made by hand in the converter (assign-*.json in its data folder: player id → SoFIFA id), all worlds together.
pub fn fest() -> HashMap<u32, u32> {
    let Some(d) = ordner().get(1).and_then(|p| p.parent().map(Path::to_path_buf)) else { return HashMap::new() };
    std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path())
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("assign-") && n.to_string_lossy().ends_with(".json"))) // roh
        .filter_map(|p| serde_json::from_str::<HashMap<String, String>>(&std::fs::read_to_string(p).ok()?).ok())
        .flatten().filter_map(|(k, v)| Some((k.parse().ok()?, v.parse().ok()?))).collect()
}

fn teile(s: &str) -> Vec<String> { norm(s).split_whitespace().map(String::from).collect() }
/// Same club: one name's words all in the other's ("Bayer Leverkusen" / "Bayer 04 Leverkusen").
fn gleicher_verein(a: &[String], b: &[String]) -> bool { !a.is_empty() && !b.is_empty() && (a.iter().all(|x| b.contains(x)) || b.iter().all(|x| a.contains(x))) }

/// The club a loaned player belongs to ("30.06.2027 RC Lens" → "RC Lens").
pub fn stammverein(r: &Zeile) -> Option<&str> { r.leihe_bis.and(r.vertrag.split_once(' ').map(|x| x.1.trim())).filter(|x| !x.is_empty()) }

/// Player index → row index: the name must fit (whole name, or surname + first initial, or all words in the full name),
/// age within a year; club, loan club and nation add. Taken only when one row is clearly best, each row at most once.
/// `fest`: player id → SoFIFA id chosen by hand in the FCFL Stats Converter (always first, when the age fits).
pub fn zuordnen(w: &World, rows: &[Zeile], fest: &HashMap<u32, u32>) -> HashMap<usize, usize> {
    let nach_id: HashMap<u32, usize> = rows.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
    let mut nach_name: HashMap<String, Vec<usize>> = HashMap::new();
    let rt: Vec<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)> = rows.iter().map(|r| (teile(&r.name), teile(&r.voll), teile(&r.verein), stammverein(r).map(teile).unwrap_or_default())).collect();
    for (i, t) in rt.iter().enumerate() {
        for n in t.0.iter().chain(&t.1).filter(|x| x.len() > 1) { let e = nach_name.entry(n.clone()).or_default(); if e.last() != Some(&i) { e.push(i); } }
    }
    let vt: Vec<Vec<String>> = w.clubs.iter().map(|c| teile(&c.name)).collect();
    // rows per world club, for other spellings in the same club (Del Prato / Delprato)
    let mut nach_verein: HashMap<&[String], Vec<usize>> = HashMap::new();
    for (i, t) in rt.iter().enumerate() { nach_verein.entry(&t.2[..]).or_default().push(i); }
    let im_verein: Vec<Vec<usize>> = vt.iter().map(|c| nach_verein.iter().filter(|(k, _)| gleicher_verein(c, k)).flat_map(|(_, v)| v.iter().copied()).collect()).collect();
    let mut beste: Vec<(i32, usize, usize)> = vec![];
    for (p, pl) in w.players.iter().enumerate() {
        if !pl.db || pl.youth > 0 || pl.team == -2 { continue; }
        if let Some(&i) = fest.get(&pl.id).and_then(|x| nach_id.get(x)) { if (pl.age as i32 - rows[i].alter as i32).abs() <= 2 { beste.push((100, p, i)); continue; } }
        let n = teile(&pl.name);
        let Some(letzt) = n.last() else { continue };
        let land = crate::nationen::name(w, pl.nat);
        let mut s: Vec<(i32, usize, i32)> = nach_name.get(letzt).into_iter().flatten().map(|&i| {
            let (r, t) = (&rows[i], &rt[i]);
            let vorn = |x: &[String]| x.first().zip(n.first()).is_some_and(|(a, b)| a.starts_with(b.as_str()) || b.starts_with(a.as_str()) || (a.len() >= 3 && b.len() >= 3 && a[..3] == b[..3]));
            let alle_in = n.iter().all(|x| t.1.contains(x) || t.0.contains(x));
            let mut sc = if n == t.0 || n == t.1 { 4 } else if (n.len() > 1 && (vorn(&t.0) || vorn(&t.1))) || alle_in { 2 } else { 0 };
            let d = (pl.age as i32 - r.alter as i32).abs();
            sc += if d <= 1 { 2 } else if d == 2 { 0 } else { -4 };
            if pl.team >= 0 && (gleicher_verein(&vt[pl.team as usize], &t.2) || gleicher_verein(&vt[pl.team as usize], &t.3)) { sc += 4; }
            if !land.is_empty() && land == r.nat { sc += 1; }
            (sc, i, sc - if d <= 1 { 2 } else if d == 2 { 0 } else { -4 })
        }).collect();
        s.sort_by(|a, b| b.cmp(a));
        // a player who moved (club does not fit): enough when he is the only one of that name and age
        let einzig = s.iter().filter(|x| x.2 >= 2).count() == 1;
        if let Some(&(sc, i, _)) = s.first() { if (sc >= 6 || (einzig && sc >= 4)) && s.get(1).is_none_or(|x| x.0 < sc) { beste.push((sc, p, i)); continue; } }
        if pl.team < 0 { continue; }
        let voll = n.join(" ");
        let mut a: Vec<(f32, usize)> = im_verein[pl.team as usize].iter().filter(|&&i| (pl.age as i32 - rows[i].alter as i32).abs() <= 1)
            .map(|&i| (crate::trainerbank::ratio(&voll, &rt[i].0.join(" ")).max(crate::trainerbank::ratio(&voll, &rt[i].1.join(" "))), i)).filter(|x| x.0 >= 0.85).collect();
        a.sort_by(|x, y| y.0.total_cmp(&x.0));
        if let Some(&(_, i)) = a.first() { if a.len() == 1 || a[1].0 < a[0].0 { beste.push((5, p, i)); } }
    }
    beste.sort_by(|a, b| b.cmp(a));
    let mut belegt = std::collections::HashSet::new();
    beste.into_iter().filter(|x| belegt.insert(x.2)).map(|x| (x.1, x.2)).collect()
}

/// Abs day of a date (season 0 starts on 1 Aug of date::START_YEAR; 365-day seasons).
fn abs_tag(y: i32, m: u32, d: u32) -> u32 {
    const MLEN: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let s = (y - crate::date::START_YEAR - if m < 8 { 1 } else { 0 }).max(0) as u32;
    let mut t = 0;
    let mut mm = 8;
    while mm != m { t += MLEN[mm as usize - 1]; mm = mm % 12 + 1; }
    s * crate::date::SEASON_DAYS + t + d.saturating_sub(1)
}
/// Last season (index) of a contract that ends in year y, month m.
fn letzte_saison(y: i32, m: u32) -> u32 { (y - crate::date::START_YEAR - if m < 8 { 1 } else { 0 }).max(0) as u32 }

pub struct Bericht { pub quelle: String, pub gefunden: usize, pub spieler: usize, pub leihen: usize }

impl World {
    /// Takes the SoFIFA values over into the world (once, at the start of a career). Club incomes follow the new wage bill.
    pub fn sofifa_uebernehmen(&mut self, quelle: &str, rows: &[Zeile]) -> Bericht {
        let zu = zuordnen(self, rows, &fest());
        let lohn_vor: Vec<f64> = (0..self.clubs.len()).map(|c| self.clubs[c].squad.iter().map(|&p| self.players[p].wage).sum()).collect();
        let vt: Vec<Vec<String>> = self.clubs.iter().map(|c| teile(&c.name)).collect();
        let mut leihen = 0;
        for (&p, &i) in &zu {
            let r = &rows[i];
            let pl = &mut self.players[p];
            pl.profil.sofifa = r.id;
            (pl.profil.emotion, pl.profil.wachstum) = (r.emotion, r.wachstum);
            pl.pot = (pl.ovr + (r.pot - r.ovr).max(0.0)).min(99.0);
            if r.lohn > 0.0 { pl.wage = ((r.lohn * 52.0 / 1000.0).round() * 1000.0).max(1000.0); } // SoFIFA wages are per week
            if r.klausel > 0.0 { pl.profil.klauseln.ausstieg = r.klausel; }
            if r.leihe_bis.is_none() {
                if let Some(bis) = r.vertrag.rsplit('~').next().and_then(|x| x.trim().parse::<i32>().ok()) { pl.contract_end = letzte_saison(bis, 6); }
                continue;
            }
            let (y, m, d) = r.leihe_bis.unwrap();
            let Some(st) = stammverein(r).map(teile) else { continue };
            let Some(von) = (0..vt.len()).find(|&k| gleicher_verein(&vt[k], &st)) else { continue };
            if von as i32 == pl.team || pl.team < 0 { continue; }
            let ende_juni = m == 6 && d == 30;
            pl.leihe = Some(crate::transfer::Leihe { von, bis: letzte_saison(y, m), anteil: crate::transfer::LEIH_ANTEIL, option: 0.0, spielzeit: false, grund: String::new(), start: pl.ovr, bis_tag: if ende_juni { 0 } else { abs_tag(y, m, d) }, pflicht: false });
            leihen += 1;
        }
        // market value: the model keeps moving it (form, age, contract), SoFIFA sets its level per player
        for p in 0..self.players.len() { self.players[p].profil.wert_faktor = 0.0; }
        self.update_values();
        for (&p, &i) in &zu {
            let v = self.players[p].value;
            if rows[i].wert > 0.0 && v > 0.0 { self.players[p].profil.wert_faktor = (rows[i].wert / v).clamp(0.2, 5.0) as f32; }
        }
        self.update_values();
        for (c, vor) in lohn_vor.into_iter().enumerate() {
            let nach: f64 = self.clubs[c].squad.iter().map(|&p| self.players[p].wage).sum();
            if vor <= 0.0 || nach <= 0.0 { continue; }
            let f = nach / vor;
            let cl = &mut self.clubs[c];
            for x in [&mut cl.tv, &mut cl.gate, &mut cl.comm, &mut cl.opex, &mut cl.balance] { *x *= f; }
        }
        Bericht { quelle: quelle.into(), gefunden: zu.len(), spieler: self.players.iter().filter(|p| p.db && p.youth == 0 && p.team != -2).count(), leihen }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zeile_lesen() {
        let t = "\u{feff}ID;Name;Vollständiger Name;Nationalität;Verein;Vertrag;Alter;Gesamtbewertung;Potenzial;Leihende;Lohn;Wert;Freigabeklausel;Größe\n\
                 69957;J. Elordi;Juan Manuel Elordi;Argentinien;Independiente Rivadavia;31.12.2026 Racing Club;31;70;70;31.12.2026;€11K;€1.2M;€0;\"176cm 5'9\"\"\"\n";
        let r = &lesen(t).unwrap()[0];
        assert_eq!((r.id, r.lohn, r.wert, r.leihe_bis), (69957, 11000.0, 1.2e6, Some((2026, 12, 31))));
        assert_eq!(stammverein(r), Some("Racing Club"));
        assert_eq!((r.emotion, r.wachstum), (0, 0), "columns missing");
        let t2 = t.replace("Freigabeklausel;", "Freigabeklausel;Emotion;Growth profile;").replace("€0;\"", "€0;Hot temper;Late Bloomer;\"");
        let r2 = &lesen(&t2).unwrap()[0];
        assert_eq!((r2.emotion, r2.wachstum), (4, 2));
        assert!(kurve(2, 19.0) < 1.0 && kurve(2, 25.0) > 1.0 && kurve(4, 31.0) < 1.0 && kurve(0, 30.0) == 1.0);
        assert_eq!(letzte_saison(2029, 6), 2);
        assert_eq!(letzte_saison(2026, 12), 0);
        assert_eq!(abs_tag(2026, 8, 1), 0);
        assert_eq!(abs_tag(2027, 6, 30), 333);
        assert!(lesen("ID;Name;Gesamtbewertung\n1;x;70\n").is_none(), "update without contracts");
    }
    /// Hit rate against the real world: CALCIOM_SOFIFA=<csv> cargo test --release sofifa_trefferquote -- --ignored --nocapture
    #[test]
    #[ignore]
    fn sofifa_trefferquote() {
        let csv = std::env::var("CALCIOM_SOFIFA").expect("CALCIOM_SOFIFA");
        let rows = lesen(&std::fs::read_to_string(&csv).unwrap()).unwrap();
        let welt = std::env::var("CALCIOM_WELT_PFAD").map(PathBuf::from).unwrap_or_else(|_| crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb"));
        let mut w = World::from_pesdb(&welt, 7).unwrap();
        let b = w.sofifa_uebernehmen(&csv, &rows);
        println!("{} of {} players ({:.1} %), {} loans, {} SoFIFA rows", b.gefunden, b.spieler, 100.0 * b.gefunden as f32 / b.spieler as f32, b.leihen, rows.len());
        let mut je: Vec<(String, usize, usize)> = w.leagues.iter().map(|l| {
            let ps: Vec<_> = l.teams.iter().flat_map(|&c| w.clubs[c].squad.iter()).filter(|&&p| w.players[p].db && w.players[p].youth == 0).collect();
            (format!("{} ({})", l.name, l.key), ps.iter().filter(|&&&p| w.players[p].profil.sofifa > 0).count(), ps.len())
        }).collect();
        je.sort_by_key(|x| std::cmp::Reverse(x.2));
        for (n, a, b) in je.iter().take(60) { println!("{a:>4}/{b:<4} {:>5.1} %  {n}", 100.0 * *a as f32 / (*b).max(1) as f32); }
        let fehl: Vec<_> = w.players.iter().filter(|p| p.db && p.youth == 0 && p.team >= 0 && p.profil.sofifa == 0 && p.ovr >= 78.0).take(40).map(|p| format!("{} ({}, {})", p.name, w.clubs[p.team as usize].name, p.age)).collect();
        let genutzt: std::collections::HashSet<u32> = w.players.iter().map(|p| p.profil.sofifa).collect();
        let offen: Vec<_> = rows.iter().filter(|r| !genutzt.contains(&r.id) && r.ovr >= 75.0).take(60).map(|r| format!("{} / {} ({}, {})", r.name, r.voll, r.verein, r.alter)).collect();
        println!("SoFIFA rows left: {} of {}; rating 75+: {}", rows.len() - genutzt.len() + 1, rows.len(), offen.join("; "));
        for k in [crate::id("Bayern"), crate::id("Sunderland")] { if let Some(c) = w.clubs.iter().position(|c| c.name.contains(k)) {
            println!("{}: tv {:.0} M, balance {:.0} M", w.clubs[c].name, w.clubs[c].tv / 1e6, w.clubs[c].balance / 1e6);
            for &p in w.clubs[c].squad.iter().take(6) { let x = &w.players[p]; println!("  {} {} ovr {} pot {} wage {:.0} k value {:.1} M until {} clause {:.0} M loan {:?}", x.name, x.age, x.ovr, x.pot, x.wage / 1e3, x.value / 1e6, x.contract_end, x.profil.klauseln.ausstieg / 1e6, x.leihe.as_ref().map(|l| (w.clubs[l.von].name.clone(), l.bis, l.bis_tag))); }
        } }
        if let Ok(j) = std::env::var("CALCIOM_VERGLEICH") {
            let v: HashMap<String, String> = serde_json::from_str(&std::fs::read_to_string(j).unwrap()).unwrap();
            let rn: HashMap<u32, &Zeile> = rows.iter().map(|r| (r.id, r)).collect();
            let (mut gleich, mut anders, mut nur_conv, mut nur_wir, mut bsp) = (0, 0, 0, 0, vec![]);
            for p in w.players.iter().filter(|p| p.db) {
                match (v.get(&p.id.to_string()).and_then(|x| x.parse::<u32>().ok()), p.profil.sofifa) {
                    (Some(a), b) if a == b => gleich += 1,
                    (Some(a), 0) => { nur_conv += 1; if bsp.len() < 25 && rn.contains_key(&a) { bsp.push(format!("+ {} {} ({}) -> {} {} ({})", p.name, p.age, if p.team >= 0 { w.clubs[p.team as usize].name.as_str() } else { "-" }, rn[&a].voll, rn[&a].alter, rn[&a].verein)); } }
                    (Some(a), b) => { anders += 1; if rn.contains_key(&a) && rn.contains_key(&b) { bsp.push(format!("! {} {} -> conv {} {} ({}), wir {} {} ({})", p.name, p.age, rn[&a].voll, rn[&a].alter, rn[&a].verein, rn[&b].voll, rn[&b].alter, rn[&b].verein)); } }
                    (None, b) if b > 0 => nur_wir += 1,
                    _ => {}
                }
            }
            println!("compare: same {gleich}, different {anders}, only converter {nur_conv}, only CalcioM {nur_wir}");
            for b in bsp.iter().take(60) { println!("  {b}"); }
        }
        println!("not found, rating 78+: {}", fehl.join("; "));
    }
}
