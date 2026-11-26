//! Career start from a Master League save of FL27 (Fabian 26.11.): the save is read once and only read, never changed.
//! Container and field positions come from the Python version (pesm/adapters/pes/spielstand, layout
//! sp-football-life-2027.toml). The positions of the blocks move with the world (MyWorld saves are larger), so every
//! block is found by its shape: the clock, the club records, the player table, the competitions, the fixtures, the
//! game plans and the own season statistics.
use crate::game::Career;
use crate::world::{Fixture, World};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;

// ---------------------------------------------------------------------------------------------------------------
// Container: six areas, each XOR-ed with a key stream (MT19937 seeded from the header and the game key)

const KEY_FL26: &str = "9061d866437724f892bab87121c76063f0919a7ded4780de51f5ddd108fe3284f5099200b23e889feb244305587600229bfeecf6500029d3427550b9ecd2f675"; // roh
const KEY_FL27: &str = "cef798b8718231b5325dfbc7301ba4adf84fd4d85c4b0e6b4f3eac61f22815c90c43105597c13f00d3674f3b63731f9d28e479b733331366b87109925e902c05"; // roh

fn hex(s: &str) -> Vec<u8> { (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect() }

struct Mt { s: [u32; 624], i: usize }
impl Mt {
    /// init_by_array with the grundwert 19650218, the first word set to 0x80000000 (standard MT19937).
    fn new(feld: &[u32]) -> Mt {
        let mut mt = [0u32; 624];
        mt[0] = 19650218;
        for i in 1..624 { mt[i] = 1812433253u32.wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_add(i as u32); }
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..624.max(feld.len()) {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1664525)).wrapping_add(feld[j]).wrapping_add(j as u32);
            i += 1; j += 1;
            if i >= 624 { mt[0] = mt[623]; i = 1; }
            if j >= feld.len() { j = 0; }
        }
        for _ in 0..623 {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1566083941)).wrapping_sub(i as u32);
            i += 1;
            if i >= 624 { mt[0] = mt[623]; i = 1; }
        }
        mt[0] = 0x8000_0000;
        Mt { s: mt, i: 624 }
    }
    fn next(&mut self) -> u32 {
        if self.i >= 624 {
            for k in 0..624 {
                let y = (self.s[k] & 0x8000_0000) | (self.s[(k + 1) % 624] & 0x7fff_ffff);
                self.s[k] = self.s[(k + 397) % 624] ^ (y >> 1) ^ if y & 1 == 1 { 0x9908_b0df } else { 0 };
            }
            self.i = 0;
        }
        let mut y = self.s[self.i];
        self.i += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }
}

/// XOR with the key stream (encrypting and decrypting are the same).
fn entschluesseln(daten: &[u8], key: &[u8]) -> Vec<u8> {
    let feld: Vec<u32> = key.chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
    let mut mt = Mt::new(&feld);
    let woerter = daten.len().div_ceil(4);
    let s: [u32; 4] = [mt.next(), mt.next(), mt.next(), mt.next()];
    let mut t: Vec<u32> = Vec::with_capacity(woerter);
    for _ in 0..woerter { t.push(mt.next()); }
    let mut out = daten.to_vec();
    out.resize(woerter * 4, 0);
    let mut r = s;
    for i in 0..woerter {
        let m = if i < 4 {
            let m = t[i] ^ r[0] ^ r[1] ^ r[2] ^ r[3];
            r = [r[1].rotate_left(17), r[2].rotate_left(11), r[3].rotate_left(7), t[i].rotate_left(19)];
            m
        } else {
            t[i] ^ t[i - 4].rotate_left(22) ^ t[i - 3].rotate_left(5) ^ t[i - 2].rotate_left(26) ^ t[i - 1].rotate_left(19)
        };
        let w = u32::from_le_bytes(out[i * 4..i * 4 + 4].try_into().unwrap()) ^ m;
        out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
    out.truncate(daten.len());
    out
}

fn laufschluessel(kopf: &[u8], spiel: &[u8]) -> Vec<u8> {
    let salz = &kopf[256..320];
    let gedreht: Vec<u8> = spiel.chunks(8).flat_map(|c| c.iter().rev().copied()).collect();
    let kk: Vec<u8> = salz.iter().zip(&gedreht).map(|(a, b)| a ^ b).collect();
    let mut klar = entschluesseln(&kopf[..320], &kk);
    klar[256..320].copy_from_slice(salz);
    let mut lauf = klar[..64].to_vec();
    for k in 64..320 { lauf[k % 64] ^= klar[k]; }
    lauf
}
fn bereich(lauf: &[u8], v: u64) -> Vec<u8> { lauf.chunks(8).flat_map(|c| (u64::from_le_bytes(c.try_into().unwrap()) ^ v).to_le_bytes()).collect() }

fn text(b: &[u8]) -> String { String::from_utf8_lossy(b.split(|&x| x == 0).next().unwrap_or(&[])).to_string() }
/// A readable name (UTF-8, at least two printable characters) at `o`, at most `n` bytes.
fn name_bei(d: &[u8], o: usize, n: usize) -> Option<String> {
    let roh = d.get(o..o + n)?.split(|&x| x == 0).next()?;
    let t = std::str::from_utf8(roh).ok()?;
    (t.chars().count() >= 2 && t.chars().all(|c| !c.is_control())).then(|| t.to_string())
}

/// Version text and the game data area of a Master League save.
pub fn oeffnen(roh: &[u8]) -> Result<(String, Vec<u8>), String> {
    if roh.len() < 528 { return Err("zu kurz für einen Spielstand".into()); } // roh
    for key in [KEY_FL27, KEY_FL26] {
        let lauf = laufschluessel(&roh[..320], &hex(key));
        let kopf = entschluesseln(&roh[320..528], &bereich(&lauf, 208));
        let l = |k: usize| u32::from_le_bytes(kopf[0x40 + 4 * k..0x44 + 4 * k].try_into().unwrap()) as usize;
        let (data, logo, desc, serial) = (l(0), l(1), l(2), l(3));
        if 528usize.saturating_add(desc).saturating_add(logo).saturating_add(data).saturating_add(2 * serial) != roh.len() { continue; }
        if text(&kopf[0x90..0xB0]) != "ML" { return Err("kein Meister-Liga-Spielstand".into()); } // roh
        let ab = 528 + desc + logo;
        return Ok((text(&kopf[0xB0..0xD0]), entschluesseln(&roh[ab..ab + data], &bereich(&lauf, 2))));
    }
    Err("Spielstand nicht lesbar (Schlüssel oder Datei unvollständig)".into()) // roh
}

// ---------------------------------------------------------------------------------------------------------------
// What the save holds

#[derive(Clone, Debug, Default)]
pub struct MlSpieler { pub alter: u8, pub pos: u8, pub gehalt: f64, pub ende: (u16, u8), pub f15: [u8; 15], pub kondition: u8, pub form: u8, pub ausfall: u8 }
#[derive(Clone, Debug, Default)]
pub struct MlVerein { pub id: u32, pub name: String, pub kader: Vec<(u32, u8)>, pub tabelle: Vec<(u16, [i32; 7])>, pub trainer: String, pub elf: Vec<u32> }
#[derive(Clone, Debug, Default)]
pub struct MlSpiel { pub datum: (i32, u32, u32), pub comp: u16, pub gespielt: bool, pub h: u32, pub a: u32, pub hg: u8, pub ag: u8, pub spieler: Vec<(u32, u8, u8, u8)> }
#[derive(Clone, Debug, Default)]
pub struct MlStand {
    pub datum: (i32, u32, u32),
    pub verein: u32,
    pub vereine: Vec<MlVerein>,
    pub spieler: HashMap<u32, MlSpieler>,
    pub comps: HashMap<u16, String>,
    pub spiele: Vec<MlSpiel>,
    /// Transfer and wage budget of the own club in euro.
    pub budget: Option<(f64, f64)>,
    /// Own squad this season: player → (appearances, goals, assists, rated games, rating sum).
    pub saison: HashMap<u32, (u16, u16, u16, u16, f32)>,
}
impl MlStand {
    pub fn eigener(&self) -> Option<&MlVerein> { self.vereine.iter().find(|v| v.id == self.verein) }
}

fn u16_(d: &[u8], o: usize) -> u16 { d.get(o..o + 2).map(|b| u16::from_le_bytes(b.try_into().unwrap())).unwrap_or(0) }
fn u32_(d: &[u8], o: usize) -> u32 { d.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).unwrap_or(0) }
fn gueltig(y: i32, m: u32, t: u32) -> bool { (2000..=2200).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&t) }

const VEREIN: usize = 1680; // club record
const SPIELER: usize = 156; // career player record
const WETTB: usize = 788; // competition record
const ZEILE: usize = 596; // fixture row
const PLAN: usize = 600; // game plan row
/// Competition table → fixture table, the same in FL27 and MyWorld saves (measured 26.11.).
const WETTB_ZU_SPIELPLAN: usize = 0x46B3FA;
/// Own season statistics → budget and → first player table stream, the same in both saves.
const STAT_ZU_BUDGET: usize = 0x77A98;
const STAT_ZU_STROM: usize = 0x53A16C;
/// The 15 abilities in the career record ([byte, bit], 6 bits, value + 40) and their index in pesdb::ATTRS.
const F15: [(usize, u32, usize); 15] = [(0, 0, 8), (0, 6, 9), (1, 4, 21), (2, 2, 22), (3, 0, 23), (4, 0, 24), (4, 6, 10), (5, 4, 14), (6, 2, 15), (7, 0, 12), (8, 0, 11), (8, 6, 13), (9, 4, 16), (11, 0, 3), (12, 0, 19)];

/// The clock (day of year, year twice, 365/366, date, own club): exactly one hit.
fn uhr(d: &[u8]) -> Result<(usize, (i32, u32, u32), u32), String> {
    let mut funde = vec![];
    for o in (0..d.len().saturating_sub(16)).step_by(4) {
        let n = u32_(d, o);
        if n != 365 && n != 366 || o < 4 { continue; }
        let at = o + 4;
        let (y, m, t) = (u16_(d, at) as i32, d[at + 2] as u32, d[at + 3] as u32);
        if !(2018..=2199).contains(&y) || u16_(d, at - 6) as i32 != y || u32_(d, at + 4) != 0 || !gueltig(y, m, t) || u16_(d, at - 8) > 366 { continue; }
        let v = u32_(d, at + 8) >> 14;
        if (1..262143).contains(&v) { funde.push((at, (y, m, t), v)); }
    }
    if funde.len() != 1 { return Err(format!("Karriere-Uhr nicht eindeutig ({} Treffer)", funde.len())); } // roh
    Ok(funde[0])
}

fn verein_satz(d: &[u8], o: usize) -> Option<MlVerein> {
    if o + VEREIN > d.len() { return None; }
    let id = u32_(d, o) >> 14;
    let name = name_bei(d, o + 4, 64)?;
    let groesse = d[o + 1062] as usize;
    if !(1..262143).contains(&id) || groesse > 40 { return None; }
    let kader: Vec<(u32, u8)> = (0..40).map(|p| (u32_(d, o + 336 + 8 * p), d[o + 666 + 2 * p])).filter(|x| x.0 != 0).collect();
    let mut tabelle = vec![];
    for k in 0..16 {
        let t = o + 748 + 20 * k;
        let nr = u16_(d, t);
        if nr == 0 || nr == 0xFFFF { break; }
        let b = |i: usize| d[t + 2 + i] as i32;
        // [p, w, d, l, gf, ga, pts]
        tabelle.push((nr & 1023, [b(1), b(2), b(3), b(4), u32_(d, t + 8) as i32, u32_(d, t + 12) as i32, b(0)]));
    }
    Some(MlVerein { id, name, kader, tabelle, trainer: String::new(), elf: vec![] })
}

/// All club records around the own one (contiguous, 1680 bytes each).
fn vereine(d: &[u8], own: u32) -> Result<(Vec<MlVerein>, Vec<usize>), String> {
    let start = (0..d.len().saturating_sub(VEREIN)).step_by(4).find(|&o| u32_(d, o) >> 14 == own && verein_satz(d, o).is_some_and(|v| v.kader.len() == d[o + 1062] as usize))
        .ok_or("Vereinssatz des eigenen Vereins nicht gefunden")?; // roh
    let mut a = start;
    while a >= VEREIN && verein_satz(d, a - VEREIN).is_some() { a -= VEREIN; }
    let (mut v, mut lagen) = (vec![], vec![]);
    let mut o = a;
    while let Some(x) = verein_satz(d, o) { v.push(x); lagen.push(o); o += VEREIN; }
    Ok((v, lagen))
}

/// Concatenated zlib streams from the first header that unpacks at or after `ab` (gap ≤ 16 bytes between them).
fn zlib_stroeme(d: &[u8], ab: usize) -> Vec<u8> {
    let kopf = |o: usize| d.get(o) == Some(&0x78) && matches!(d.get(o + 1), Some(0x01 | 0x9c | 0xda));
    let mut out = vec![];
    let mut pos = (ab..d.len().saturating_sub(2)).find(|&o| kopf(o));
    while let Some(p) = pos {
        let mut z = flate2::read::ZlibDecoder::new(&d[p..]);
        let mut teil = vec![];
        if z.read_to_end(&mut teil).is_err() { if out.is_empty() { pos = (p + 1..d.len().saturating_sub(2)).find(|&o| kopf(o)); continue; } break; }
        out.extend_from_slice(&teil);
        let ende = p + z.total_in() as usize;
        pos = (ende..(ende + 17).min(d.len().saturating_sub(2))).find(|&o| kopf(o));
    }
    out
}

fn spielertabelle(inhalt: &[u8]) -> HashMap<u32, MlSpieler> {
    let mut m = HashMap::new();
    let n = inhalt.len().saturating_sub(8) / SPIELER;
    for i in 0..n {
        let r = &inhalt[8 + i * SPIELER..8 + (i + 1) * SPIELER];
        let id = u32_(r, 28);
        if id == 0 || m.contains_key(&id) { continue; }
        let mut f15 = [0u8; 15];
        for (k, &(by, bit, _)) in F15.iter().enumerate() { f15[k] = ((u32_(r, by) >> bit) & 63) as u8 + 40; }
        let w = u16_(r, 144);
        m.insert(id, MlSpieler { alter: (r[10] >> 2) & 63, pos: (r[13] >> 3) & 15, gehalt: u32_(r, 108) as f64 * 100.0, ende: (u16_(r, 124), r[126]), f15,
            kondition: (w & 127) as u8, form: ((w >> 7) & 7) as u8, ausfall: r[140] });
    }
    m
}

/// Every run of at least `mindestens` readable names in steps of `schritt` (aligned to 4): start and length.
fn namenslaeufe(d: &[u8], schritt: usize, mindestens: usize, n: usize) -> Vec<(usize, usize)> {
    let mut laeufe = vec![];
    let mut o = 0;
    while o + schritt < d.len() {
        if d[o].is_ascii_alphabetic() && (o < schritt || name_bei(d, o - schritt, n).is_none()) && name_bei(d, o, n).is_some() {
            let mut k = 0;
            while name_bei(d, o + k * schritt, n).is_some() { k += 1; }
            if k >= mindestens { laeufe.push((o, k)); }
        }
        o += 4;
    }
    laeufe
}

/// The fixture table: the longest run of rows with a valid date in steps of 596 bytes.
fn spielplan_lage(d: &[u8]) -> Option<(usize, usize)> {
    let ok = |o: usize| o + 4 <= d.len() && gueltig(u16_(d, o) as i32, d[o + 2] as u32, d[o + 3] as u32);
    let mut best: Option<(usize, usize)> = None;
    for o in 0..d.len().saturating_sub(ZEILE * 3) {
        if !ok(o) || !ok(o + ZEILE) || !ok(o + 2 * ZEILE) || (o >= ZEILE && ok(o - ZEILE)) { continue; }
        let mut k = 0;
        while ok(o + k * ZEILE) { k += 1; }
        if best.is_none_or(|b| k > b.1) { best = Some((o, k)); }
    }
    best.filter(|b| b.1 >= 50)
}

/// Reads everything the career start needs. Never writes.
pub fn lesen(pfad: &Path) -> Result<MlStand, String> {
    let roh = std::fs::read(pfad).map_err(|e| format!("{}: {e}", pfad.display()))?;
    let (_, d) = oeffnen(&roh)?;
    let (_, datum, verein) = uhr(&d)?;
    let (mut vs, _) = vereine(&d, verein)?;
    let own_kader: HashSet<u32> = vs.iter().find(|v| v.id == verein).map(|v| v.kader.iter().map(|x| x.0).collect()).unwrap_or_default();
    // own season statistics: 44-byte records starting with the own players' ids
    let mut stat: Option<usize> = None;
    for o in (0..d.len().saturating_sub(44 * 10)).step_by(4) {
        if own_kader.contains(&u32_(&d, o)) && (o < 44 || !own_kader.contains(&u32_(&d, o - 44))) && (1..10).all(|k| own_kader.contains(&u32_(&d, o + 44 * k))) { stat = Some(o); break; }
    }
    let mut saison = HashMap::new();
    let mut budget = None;
    if let Some(s) = stat {
        for i in 0..40 {
            let o = s + 44 * i;
            let id = u32_(&d, o);
            if id == 0 || id == u32::MAX { continue; }
            let w = |k: usize| u16_(&d, o + 4 + 2 * k);
            saison.insert(id, (w(1), w(2), w(3), w(4), w(5) as f32 / 10.0));
        }
        let b = s + STAT_ZU_BUDGET;
        let (t, g) = (u32_(&d, b) as f64 * 100.0, u32_(&d, b + 20) as f64 * 100.0);
        if t <= 1e9 && g <= 1e9 { budget = Some((t, g)); }
    }
    // the player table: the zlib streams behind the statistics (found from the clock if that fails)
    let mut tabelle = stat.map(|s| spielertabelle(&zlib_stroeme(&d, s + STAT_ZU_STROM))).unwrap_or_default();
    if tabelle.len() < 1000 { tabelle = spielertabelle(&zlib_stroeme(&d, uhr(&d)?.0)); }
    if tabelle.len() < 1000 { return Err("Spielertabelle nicht gefunden".into()); } // roh
    // game plans: one row per club (the record's row number at +652): coach name and the eleven
    let lagen = vereine(&d, verein)?.1;
    let zeile = |l: usize| u32_(&d, l + 652) as usize;
    let max_zeile = lagen.iter().map(|&l| zeile(l)).max().unwrap_or(0);
    // the table that holds every club's row and a valid eleven in the own row (11 different squad places)
    let elf_ok = |o: usize| { let e = &d[o + 536..o + 547]; e.iter().all(|&p| p < 40) && e.iter().collect::<HashSet<_>>().len() == 11 };
    let own_zeile = vs.iter().zip(&lagen).find(|(v, _)| v.id == verein).map(|(_, &l)| zeile(l)).unwrap_or(0);
    if let Some((g, n)) = namenslaeufe(&d, PLAN, vs.len().min(200), 60).into_iter().find(|&(g, n)| n > max_zeile && elf_ok(g + own_zeile * PLAN)) {
        for (v, &l) in vs.iter_mut().zip(&lagen) {
            let z = zeile(l);
            if z >= n { continue; }
            let o = g + z * PLAN;
            v.trainer = name_bei(&d, o, 60).unwrap_or_default();
            let kader: Vec<u32> = (0..40).map(|p| u32_(&d, l + 336 + 8 * p)).collect();
            v.elf = d[o + 536..o + 547].iter().filter(|&&p| (p as usize) < 40).map(|&p| kader[p as usize]).filter(|&x| x != 0).collect();
        }
    }
    // competitions and fixtures
    let mut comps = HashMap::new();
    let mut spiele = vec![];
    if let Some((sp, n)) = spielplan_lage(&d) {
        if sp >= WETTB_ZU_SPIELPLAN {
            let w0 = sp - WETTB_ZU_SPIELPLAN;
            for i in 0..1024 {
                let o = w0 + i * WETTB;
                if o + WETTB > sp { break; }
                if let Some(nm) = name_bei(&d, o, 64) { comps.entry(u16_(&d, o - 2) & 1023).or_insert(nm); }
            }
        }
        for i in 0..n {
            let o = sp + i * ZEILE;
            let datum = (u16_(&d, o) as i32, d[o + 2] as u32, d[o + 3] as u32);
            let gespielt = d[o - 1] & 0x40 != 0;
            let mut spieler = vec![];
            if gespielt {
                for (seite, ab) in [(0u8, 28usize), (1, 300)] {
                    for k in 0..17 {
                        let e = o + ab + 16 * k;
                        if u16_(&d, e) == 0xFFFF { continue; }
                        let id = u32_(&d, e + 4);
                        if id != 0 { spieler.push((id, d[e + 8], d[e + 9], ((u32_(&d, e + 12) >> 8) & 63) as u8 | seite << 7)); }
                    }
                }
            }
            spiele.push(MlSpiel { datum, comp: u16_(&d, o - 4) & 1023, gespielt, h: u32_(&d, o + 12) >> 14, a: u32_(&d, o + 16) >> 14, hg: d[o + 20], ag: d[o + 23], spieler });
        }
    }
    Ok(MlStand { datum, verein, vereine: vs, spieler: tabelle, comps, spiele, budget, saison })
}

// ---------------------------------------------------------------------------------------------------------------
// The career from the save

/// The world folder whose database knows the most players of the save (MyWorld saves need MyWorld).
pub fn welt_waehlen(st: &MlStand) -> Option<(std::path::PathBuf, usize)> {
    crate::pesdb::welten(&crate::fl27::sider()).into_iter().filter_map(|w| {
        let pb = crate::pesdb::PlayerBin::load(&w.join("common/etc/pesdb/Player.bin")).ok()?;
        Some((w, st.spieler.keys().filter(|id| pb.index.contains_key(id)).count()))
    }).max_by_key(|x| x.1)
}

/// Master League saves of FL27 on this computer, newest first: in the game's save folder and folders next to it whose
/// name starts with "save" (renamed copies). Linux: inside the Wine prefix of the game folder; Windows: Documents.
pub fn gefundene() -> Vec<std::path::PathBuf> {
    let mut wurzeln = vec![];
    if let Some(dc) = crate::fl27::spielordner().ancestors().find(|a| a.ends_with(crate::id("drive_c"))) {
        for u in std::fs::read_dir(dc.join(crate::id("users"))).into_iter().flatten().flatten() { wurzeln.push(u.path().join("Documents/SPGAME/Football Life/2027")); } // roh
    }
    if let Ok(h) = std::env::var(crate::id("USERPROFILE")) { wurzeln.push(Path::new(&h).join("Documents/SPGAME/Football Life/2027")); } // roh
    let ml = |n: &str| n.len() == 10 && n.starts_with("ML") && n[2..].bytes().all(|b| b.is_ascii_digit()); // roh
    let mut v: Vec<(std::time::SystemTime, std::path::PathBuf)> = vec![];
    for w in wurzeln {
        for d in std::fs::read_dir(&w).into_iter().flatten().flatten().filter(|d| d.file_name().to_string_lossy().starts_with(crate::id("save")) && d.path().is_dir()) {
            for f in std::fs::read_dir(d.path()).into_iter().flatten().flatten().filter(|f| ml(&f.file_name().to_string_lossy())) {
                // the same folder under two user names (Wine links them): once
                let echt = std::fs::canonicalize(f.path()).unwrap_or(f.path());
                if let Ok(t) = f.metadata().and_then(|m| m.modified()) { if !v.iter().any(|x| x.1 == echt) { v.push((t, echt)); } }
            }
        }
    }
    v.sort_by(|a, b| b.0.cmp(&a.0));
    v.into_iter().map(|x| x.1).collect()
}

/// Club and date of a save for its button ("Sunderland AFC · 09.05.2027"); None when it cannot be read.
pub fn kurz(p: &Path) -> Option<String> {
    let (_, d) = oeffnen(&std::fs::read(p).ok()?).ok()?;
    let (_, (y, m, t), v) = uhr(&d).ok()?;
    let name = vereine(&d, v).ok()?.0.into_iter().find(|x| x.id == v)?.name;
    Some(format!("{name} · {t:02}.{m:02}.{y}"))
}

/// The season day of a date (1 August = day 0); 29 February counts as 28 February.
fn saisontag(season: u32, (y, m, t): (i32, u32, u32)) -> Option<u32> {
    let t = if m == 2 && t == 29 { 28 } else { t };
    (0..crate::date::SEASON_DAYS).find(|&d| crate::date::ymd(season, d) == (y, m, t))
}

/// Form arrow of the game (0 very bad … 4 top) as CalcioM form, in the middle of each step (mensch::pfeil).
const FORM: [f32; 5] = [-5.0, -2.3, 0.0, 2.3, 5.0];

/// Builds the career on the world `welt`: date, coach, squads of every club, players (age, position, contract, wage,
/// the 15 abilities the save holds, the other 10 moved by the same amount, fitness, form, injury), coaches of the
/// other clubs, league tables and fixtures, the season's player figures and the own budget. Cups and Europe are played
/// by CalcioM up to the day (their rounds are not read).
pub fn aufbauen(st: &MlStand, welt: &Path) -> Result<Career, String> {
    let w = World::from_pesdb(&welt.join("common/etc/pesdb"), 7).map_err(|e| format!("pesdb: {e}"))?; // roh
    let own = w.clubs.iter().position(|c| c.id == st.verein).ok_or("eigener Verein nicht in der Welt")?; // roh
    let trainer = st.eigener().map(|v| v.trainer.clone()).filter(|t| !t.is_empty()).unwrap_or_else(|| crate::inhalt::t("ui.main.fabian_trainer").to_string());
    let mut c = Career::new(w, own, &trainer);
    let (y, m, _) = st.datum;
    let season = ((if m >= 8 { y } else { y - 1 }) - crate::date::START_YEAR).max(0) as u32;
    c.world.season = season;
    let tag = saisontag(season, st.datum).ok_or("Datum des Spielstands passt in keine Saison")?; // roh
    let club_ix: HashMap<u32, usize> = c.world.clubs.iter().enumerate().map(|(i, k)| (k.id, i)).collect();
    let mut spieler_ix: HashMap<u32, usize> = c.world.players.iter().enumerate().map(|(i, p)| (p.id, i)).collect();

    // players the game made itself (not in the database): new players with the save's abilities
    for v in st.vereine.iter().filter(|v| club_ix.contains_key(&v.id)) {
        for &(id, _) in &v.kader {
            if spieler_ix.contains_key(&id) { continue; }
            let Some(s) = st.spieler.get(&id) else { continue };
            let k = club_ix[&v.id];
            let key = format!("ml:{id}"); // roh
            let nat = c.jugend_nat(k, &key);
            let name = c.name_aus(&c.namen_von(nat), &key);
            let pos = crate::pesdb::POS[(s.pos as usize).min(12)].to_string();
            let (attrs, pp) = c.world.template_attrs(&pos, 65.0);
            let neu = c.world.new_id();
            let mut p = c.world.players[0].clone();
            p.id = neu; p.name = name; p.pos = pos; p.nat = nat; p.attrs = attrs; p.pp = pp; p.db = false;
            p.youth = 0; p.leihe = None; p.style = 0; p.skills = 0; p.apps = 0; p.goals = 0; p.assists = 0; p.rsum = 0.0; p.rn = 0; p.last = vec![];
            spieler_ix.insert(id, c.world.players.len());
            c.world.players.push(p);
        }
    }
    // squads: every club of the save as it is there; who is in no squad of a known club is without a club
    let mut team_neu: HashMap<usize, (usize, u8)> = HashMap::new();
    for v in &st.vereine {
        let Some(&k) = club_ix.get(&v.id) else { continue };
        for &(id, no) in &v.kader { if let Some(&p) = spieler_ix.get(&id) { team_neu.insert(p, (k, no)); } }
    }
    let bekannt: HashSet<usize> = st.vereine.iter().filter_map(|v| club_ix.get(&v.id).copied()).collect();
    for k in &bekannt { let jugend: Vec<usize> = c.world.clubs[*k].squad.iter().copied().filter(|&p| c.world.players[p].youth > 0).collect(); c.world.clubs[*k].squad = jugend; }
    for (i, p) in c.world.players.iter_mut().enumerate() {
        if let Some(&(k, no)) = team_neu.get(&i) { p.team = k as i32; p.youth = 0; p.leihe = None; if (1..100).contains(&no) { p.no = no; } }
        else if p.team >= 0 && bekannt.contains(&(p.team as usize)) && p.youth == 0 { p.team = -1; p.leihe = None; }
    }
    let mut zuordnung: Vec<(usize, usize)> = team_neu.iter().map(|(&p, &(k, _))| (k, p)).collect();
    zuordnung.sort();
    for (k, p) in zuordnung { c.world.clubs[k].squad.push(p); }
    // clubs the save does not have lose the players it puts elsewhere
    for k in 0..c.world.clubs.len() { let sq = std::mem::take(&mut c.world.clubs[k].squad); c.world.clubs[k].squad = sq.into_iter().filter(|&p| c.world.players[p].team == k as i32).collect(); }

    // the players' state
    for (id, s) in &st.spieler {
        let Some(&i) = spieler_ix.get(id) else { continue };
        let lg = c.world.players[i].team.max(0) as usize;
        let lvl = c.world.leagues[c.world.clubs[lg].league].level;
        let p = &mut c.world.players[i];
        if (15..=55).contains(&s.alter) { p.age = s.alter; }
        if s.pos <= 12 { p.pos = crate::pesdb::POS[s.pos as usize].to_string(); }
        if s.gehalt > 0.0 { p.wage = s.gehalt; }
        let (ej, em) = s.ende;
        if ej != 0xFFFF && ej >= 2000 { p.contract_end = ((ej as i32 - crate::date::START_YEAR - if em <= 8 { 1 } else { 0 }).max(season as i32)) as u32; }
        if (1..=100).contains(&s.kondition) { p.fit = s.kondition as f32; }
        p.form = FORM[(s.form as usize).min(4)];
        p.inj = s.ausfall as u16;
        if p.attrs.len() == 25 {
            // the save holds 15 abilities; the other ten move by the average change of those 15
            let diff: f32 = F15.iter().enumerate().map(|(k, f)| s.f15[k] as f32 - p.attrs[f.2]).sum::<f32>() / 15.0;
            let mit: HashSet<usize> = F15.iter().map(|f| f.2).collect();
            for a in 0..25 { if !mit.contains(&a) { p.attrs[a] = (p.attrs[a] + diff).clamp(40.0, 99.0); } }
            for (k, f) in F15.iter().enumerate() { p.attrs[f.2] = s.f15[k] as f32; }
            p.ovr = crate::pesdb::ovr(&p.attrs, crate::pesdb::pos_index(&p.pos)).clamp(30.0, 99.0);
            p.pot = p.pot.max(p.ovr);
        }
        p.value = crate::world::market_value(p, season, lvl);
    }
    // coaches of the clubs
    for v in &st.vereine { if let Some(&k) = club_ix.get(&v.id) { if !v.trainer.is_empty() && k != own { c.world.clubs[k].manager = v.trainer.clone(); } } }

    // this season's matches (1 August to 31 July)
    let in_saison = |s: &MlSpiel| saisontag(season, s.datum).is_some();
    let mut nachholen = vec![];
    for l in 0..c.world.leagues.len() {
        let teams: HashSet<usize> = c.world.leagues[l].teams.iter().copied().collect();
        // the competition whose table holds this league's clubs and the fewest others
        let mut zahl: HashMap<u16, (i32, i32)> = HashMap::new();
        for v in &st.vereine {
            let drin = club_ix.get(&v.id).is_some_and(|k| teams.contains(k));
            for (nr, _) in &v.tabelle { let z = zahl.entry(*nr).or_default(); if drin { z.0 += 1; } else { z.1 += 1; } }
        }
        let Some((&nr, _)) = zahl.iter().filter(|(_, z)| z.0 * 2 >= teams.len() as i32).max_by_key(|(n, z)| (z.0 - z.1, std::cmp::Reverse(**n))) else { nachholen.push(l); continue };
        let alt = c.world.leagues[l].fixtures.clone();
        let fx: Vec<Fixture> = st.spiele.iter().filter(|s| s.comp == nr && in_saison(s)).filter_map(|s| {
            let (h, a) = (*club_ix.get(&s.h)?, *club_ix.get(&s.a)?);
            if !teams.contains(&h) || !teams.contains(&a) { return None; }
            let day = saisontag(season, s.datum)?;
            let zeit = alt.iter().find(|f| f.h == h && f.a == a).map(|f| f.zeit).unwrap_or(0);
            Some(if s.gespielt { Fixture { day, h, a, hg: s.hg as i8, ag: s.ag as i8, zeit } } else { Fixture { day, h, a, hg: -1, ag: -1, zeit } })
        }).collect();
        // the table as in the save: what the fixtures do not hold comes in as a base
        let tab: HashMap<usize, [i32; 7]> = st.vereine.iter().filter_map(|v| Some((*club_ix.get(&v.id)?, v.tabelle.iter().find(|t| t.0 == nr)?.1))).filter(|(k, _)| teams.contains(k)).collect();
        if fx.is_empty() { c.world.leagues[l].fixtures.retain(|f| f.day >= tag); } else { c.world.leagues[l].fixtures = fx; }
        c.world.leagues[l].basis = vec![];
        let gerechnet: HashMap<usize, crate::world::Row> = c.world.table(l).into_iter().map(|r| (r.club, r)).collect();
        c.world.leagues[l].basis = tab.iter().filter_map(|(&k, z)| {
            let r = gerechnet.get(&k)?;
            let b = [z[0] - r.p as i32, z[1] - r.w as i32, z[2] - r.d as i32, z[3] - r.l as i32, z[4] - r.gf, z[5] - r.ga, z[6] - r.pts];
            b.iter().any(|&x| x != 0).then_some((k, b))
        }).collect();
    }
    // the season's figures of every player from the matches, the own squad from its statistics
    for s in st.spiele.iter().filter(|s| s.gespielt && in_saison(s)) {
        for &(id, tore, vorlagen, note) in &s.spieler {
            let Some(&i) = spieler_ix.get(&id) else { continue };
            let p = &mut c.world.players[i];
            p.apps += 1; p.goals += tore as u16; p.assists += vorlagen as u16;
            let n = (note & 63) as f32 / 5.0;
            if n > 0.0 { p.rsum += n; p.rn += 1; }
        }
    }
    for (id, &(apps, goals, assists, rn, rsum)) in &st.saison {
        let Some(&i) = spieler_ix.get(id) else { continue };
        let p = &mut c.world.players[i];
        (p.apps, p.goals, p.assists, p.rn, p.rsum) = (apps, goals, assists, rn, rsum);
    }
    // Europe: the own club plays there only if it does in the save (a competition with clubs of several countries);
    // else a club of its league that is not in Europe takes its place, as in the import of the Python career
    let land = |id: &u32| club_ix.get(id).map(|&k| c.world.leagues[c.world.clubs[k].league].land.clone());
    let international = |nr: u16| st.vereine.iter().filter(|v| v.tabelle.iter().any(|t| t.0 == nr)).filter_map(|v| land(&v.id)).collect::<HashSet<_>>().len() > 1;
    if !st.eigener().is_some_and(|v| v.tabelle.iter().any(|t| international(t.0))) {
        let liga = c.world.clubs[own].league;
        let drin = |w: &World, k: usize| w.cups.iter().any(|cu| (1..=3).contains(&cu.kind) && cu.teams.contains(&k));
        let ersatz = c.world.table(liga).iter().map(|r| r.club).find(|&k| k != own && !drin(&c.world, k));
        for cu in c.world.cups.iter_mut().filter(|cu| (1..=3).contains(&cu.kind)) {
            let Some(e) = ersatz else { cu.teams.retain(|&k| k != own); continue };
            for t in cu.teams.iter_mut().chain(cu.alive.iter_mut()) { if *t == own { *t = e; } }
            for x in cu.fx.iter_mut() { if x.f.h == own { x.f.h = e; } if x.f.a == own { x.f.a = e; } }
        }
    }
    if let Some((t, _)) = st.budget.filter(|b| b.0 > 0.0) { c.transfer_budget = t; }
    // leagues the save did not match, the cups and Europe: played by CalcioM up to the day
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
    c.auto_lineup();
    Ok(c)
}

/// A short look at the save before the start: club, date, coach, place in the league, squad, what comes over.
pub fn vorschau(st: &MlStand, welt: &Path, treffer: usize) -> String {
    let v = st.eigener();
    let (y, m, t) = st.datum;
    let platz = v.and_then(|v| v.tabelle.first()).map(|(nr, z)| {
        let mut alle: Vec<(i32, i32, i32, u32)> = st.vereine.iter().filter_map(|x| x.tabelle.iter().find(|e| e.0 == *nr).map(|e| (e.1[6], e.1[4] - e.1[5], e.1[4], x.id))).collect();
        alle.sort_by(|a, b| b.cmp(a));
        let rang = alle.iter().position(|a| a.3 == st.verein).map(|i| i + 1).unwrap_or(0);
        crate::tf!("ui.mlimport.platz", rang, st.comps.get(nr).cloned().unwrap_or_default(), z[0], z[6])
    }).unwrap_or_default();
    crate::tf!("ui.mlimport.vorschau", v.map(|v| v.name.clone()).unwrap_or_default(), format!("{t:02}.{m:02}.{y}"), v.map(|v| v.trainer.clone()).unwrap_or_default(), platz,
        v.map(|v| v.kader.len()).unwrap_or(0), st.vereine.len(), st.spieler.len(), crate::fl27::welt_name(welt), treffer)
}

/// Builds and saves the career with its own database copy, like a new career (the save stays as it is).
pub fn karriere_anlegen(st: &MlStand, welt: &Path) -> Result<Career, String> {
    let mut c = aufbauen(st, welt)?;
    // a name free in the game and among the careers: existing careers of the club stay untouched
    let lc = crate::fl27::sider().join(crate::id("livecpk"));
    let erster = crate::fl27::cpk_name(&c.club().name.clone(), &lc);
    let stamm = &erster[..erster.len() - 2];
    let frei = |n: &str| !lc.join(n).exists() && !crate::game::save_dir().join(n).exists() && !crate::game::save_dir().join(format!("{n}.json")).exists(); // roh
    c.save_name = (1..=99).map(|n| format!("{stamm}{n:02}")).find(|n| frei(n)).ok_or("kein freier Karrierename")?; // roh
    c.fl27_welt = welt.display().to_string();
    Ok(c)
}

/// `calciom --import-ml <save> [<welt>]`: the career on the command line (tests and checks).
pub fn kommando(save: &str, welt: Option<&str>) -> Result<String, String> {
    let st = lesen(Path::new(save))?;
    let welt = match welt { Some(w) => std::path::PathBuf::from(w), None => welt_waehlen(&st).ok_or("keine Welt gefunden")?.0 }; // roh
    let c = karriere_anlegen(&st, &welt)?;
    let s2 = welt.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_else(crate::fl27::sider);
    crate::fl27::original_sichern(&c.karriere_dir(), &s2, &welt)?;
    crate::fl27::karriere_pesdb_anlegen(&c.karriere_dir(), &welt)?;
    let mut c = c;
    let _ = c.fl27_cpk();
    c.save()?;
    Ok(format!("{} angelegt: {}, {}, Trainer {}", c.save_name, c.club().name, crate::date::long(c.world.season, c.world.day), c.coach)) // roh
}

#[cfg(test)]
mod tests {
    use super::*;
    /// The MT19937 of the container is the standard one (reference output of init_by_array {0x123, 0x234, 0x345, 0x456}).
    #[test]
    fn mt_wie_referenz() {
        let mut m = Mt::new(&[0x123, 0x234, 0x345, 0x456]);
        assert_eq!(m.next(), 1067595299);
        assert_eq!(m.next(), 955945823);
    }
    #[test]
    fn xor_zweimal_ist_original() {
        let k = hex(KEY_FL27);
        let d: Vec<u8> = (0..1001u32).map(|i| (i * 7) as u8).collect();
        assert_eq!(entschluesseln(&entschluesseln(&d, &k), &k), d);
    }
    /// Saves in the Wine prefix: the save folder and its renamed copies, the same folder under a linked user name once.
    #[cfg(unix)]
    #[test]
    fn gefundene_im_prefix() {
        let _g = crate::game::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let t = std::env::temp_dir().join(format!("calciom-mlsuche-{}", std::process::id()));
        let doc = t.join("drive_c/users/steamuser/Documents/SPGAME/Football Life/2027");
        for (d, f) in [("save", "ML00000001"), ("save_alt", "ML00000000"), ("save", "EDIT00000000"), ("andere", "ML00000002")] {
            std::fs::create_dir_all(doc.join(d)).unwrap();
            std::fs::write(doc.join(d).join(f), b"x").unwrap();
        }
        std::os::unix::fs::symlink("steamuser", t.join("drive_c/users/fabian")).unwrap();
        std::fs::create_dir_all(t.join("drive_c/Games/FL")).unwrap();
        let vorher = std::env::var("CALCIOM_FL27_SPIEL").ok();
        std::env::set_var("CALCIOM_FL27_SPIEL", t.join("drive_c/Games/FL"));
        let mut n: Vec<String> = gefundene().iter().map(|p| p.file_name().unwrap().to_string_lossy().to_string()).collect();
        match vorher { Some(v) => std::env::set_var("CALCIOM_FL27_SPIEL", v), None => std::env::remove_var("CALCIOM_FL27_SPIEL") }
        let _ = std::fs::remove_dir_all(&t);
        n.sort();
        assert_eq!(n, ["ML00000000", "ML00000001"]);
    }
    #[test]
    fn saisontage() {
        assert_eq!(saisontag(0, (2026, 8, 1)), Some(0));
        assert_eq!(saisontag(0, (2027, 7, 31)), Some(364));
        assert_eq!(saisontag(0, (2027, 8, 1)), None);
    }
    /// A real save (CALCIOM_ML_SAVE=<file>, read only): run with --ignored --nocapture.
    #[test]
    #[ignore]
    fn echter_spielstand() {
        let Ok(p) = std::env::var("CALCIOM_ML_SAVE") else { return };
        let t0 = std::time::Instant::now();
        let st = lesen(Path::new(&p)).unwrap();
        eprintln!("{:?} Verein {} Vereine {} Spieler {} Wettbewerbe {} Spiele {} ({} gespielt) Budget {:?} Saison {} in {:.1?}", st.datum, st.verein, st.vereine.len(), st.spieler.len(), st.comps.len(), st.spiele.len(), st.spiele.iter().filter(|s| s.gespielt).count(), st.budget, st.saison.len(), t0.elapsed());
        let v = st.eigener().unwrap();
        eprintln!("{} Trainer {} Kader {} Elf {} Tabelle {:?}", v.name, v.trainer, v.kader.len(), v.elf.len(), v.tabelle);
        assert!(st.vereine.len() > 100 && st.spieler.len() > 1000 && !v.kader.is_empty());
        if std::env::var("CALCIOM_ML_BAUEN").is_err() { return; }
        let (welt, n) = welt_waehlen(&st).unwrap();
        eprintln!("{}", vorschau(&st, &welt, n));
        let c = aufbauen(&st, &welt).unwrap();
        eprintln!("{} {} Trainer {} Kasse {:.0} Budget {:.0}", c.club().name, crate::date::long(c.world.season, c.world.day), c.coach, c.club().balance, c.transfer_budget);
        for (i, r) in c.world.table(c.league()).iter().enumerate() { eprintln!("{:2}. {:28} {:2} {:2}-{:2}-{:2} {:3}:{:3} {:3}", i + 1, c.world.clubs[r.club].name, r.p, r.w, r.d, r.l, r.gf, r.ga, r.pts); }
        let mut sq = c.squad(); sq.sort_by_key(|&p| std::cmp::Reverse(c.world.players[p].goals));
        for &p in sq.iter().take(8) { let pl = &c.world.players[p]; eprintln!("{} {} {} Alter {} Stärke {:.0} Tore {} Spiele {} Vertrag {} Gehalt {:.0}", pl.no, pl.name, pl.pos, pl.age, pl.ovr, pl.goals, pl.apps, pl.contract_end, pl.wage); }
        // every club of a league: its row in the career is one of its rows in the save
        let (mut gleich, mut alle) = (0, 0);
        for v in &st.vereine {
            let Some(k) = c.world.clubs.iter().position(|x| x.id == v.id) else { continue };
            if v.tabelle.is_empty() { continue; }
            let Some(r) = c.world.table(c.world.clubs[k].league).into_iter().find(|r| r.club == k) else { continue };
            alle += 1;
            if v.tabelle.iter().any(|(_, z)| (z[0], z[6], z[4], z[5]) == (r.p as i32, r.pts, r.gf, r.ga)) { gleich += 1; } else if alle - gleich <= 10 { eprintln!("  anders: {} in {}: {:?} Karriere {} {}", v.name, c.world.leagues[c.world.clubs[k].league].name, v.tabelle, r.p, r.pts); }
        }
        let frei = c.world.players.iter().filter(|p| p.team == -1).count();
        let kader: Vec<usize> = c.world.clubs.iter().map(|k| k.squad.len()).collect();
        eprintln!("Tabellenzeilen wie im Spielstand: {gleich}/{alle}; vereinslos {frei}; Kader {}..{}", kader.iter().min().unwrap(), kader.iter().max().unwrap());
        // the save's own table and the career's match
        let (nr, z) = v.tabelle[0];
        let r = c.world.table(c.league()).into_iter().find(|r| r.club == c.own).unwrap();
        eprintln!("Wettbewerb {nr}: Spielstand {:?}, Karriere {} Spiele {} Punkte", z, r.p, r.pts);
        assert_eq!((r.p as i32, r.pts), (z[0], z[6]), "the own table row is the save's");
    }
    /// The career's first database write (CALCIOM_ML_KARRIERE=<career .json> made by --import-ml, game folder a copy via
    /// CALCIOM_FL27_SPIEL): every club's squad of the save is in its PlayerAssignment.bin.
    #[test]
    #[ignore]
    fn karriere_schreiben() {
        let Ok(p) = std::env::var("CALCIOM_ML_KARRIERE") else { return };
        let mut c = crate::game::load(Path::new(&p)).unwrap();
        let (n, dateien) = c.fl27_schreiben().unwrap();
        let (_, a) = crate::fl27::asg_laden(&crate::fl27::karriere_pesdb(&c.karriere_dir())).unwrap();
        let (mut gleich, mut alle) = (0, 0);
        for k in &c.world.clubs {
            if k.squad.is_empty() { continue; }
            alle += 1;
            let mut soll: Vec<u32> = k.squad.iter().map(|&p| c.world.players[p].id).collect();
            let mut ist: Vec<u32> = a.iter().filter(|x| x.team == k.id).map(|x| x.pid).collect();
            soll.sort(); ist.sort();
            if soll == ist { gleich += 1; }
        }
        eprintln!("geschrieben {n} {dateien:?}; Kader wie in der Karriere: {gleich}/{alle}");
        let own = c.world.clubs[c.own].id;
        // injured and suspended players of today's two clubs leave the game's squad for the match (asg_abgleichen)
        assert!(c.squad().iter().filter(|&&p| c.world.players[p].available()).all(|&p| a.iter().any(|x| x.team == own && x.pid == c.world.players[p].id)), "own squad in the database");
    }

}
