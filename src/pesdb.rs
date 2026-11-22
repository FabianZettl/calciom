// FL27 database (pesdb) read and written directly, without the Python version (Übergabe "Daten und Datenbank" §0-§3, §7):
// Player.bin (312-byte records, bit fields LSB-first), Team.bin, Competition.bin, CompetitionEntry.bin, PlayerAssignment.bin,
// the overall-rating model (ovr_weights_fl27.json) and the selection of leagues, clubs and players (fl/sim/welt.py import_pesdb).
use crate::fl27::{self, Asg};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub const REC: usize = 312;
pub const POS: [&str; 13] = [crate::id("GK"), crate::id("CB"), crate::id("LB"), crate::id("RB"), crate::id("DMF"), crate::id("CMF"), crate::id("LMF"), crate::id("RMF"), crate::id("AMF"), crate::id("LWF"), crate::id("RWF"), crate::id("SS"), crate::id("CF")];
/// The 25 abilities in the order of the rating model (plbin.ATTR_BITS / AB), with their bit offset.
pub const ATTRS: [(&str, u32); 25] = [ // roh
    ("OffensiveAwareness", 370), ("BallControl", 281), ("Dribbling", 352), ("TightPossession", 416), ("LowPass", 263), ("LoftedPass", 402), // roh
    ("Finishing", 396), ("Heading", 288), ("PlaceKicking", 250), ("Curl", 332), ("Speed", 306), ("Acceleration", 344), ("KickingPower", 358), // roh
    ("Jump", 294), ("PhysicalContact", 390), ("Balance", 376), ("Stamina", 338), ("DefensiveAwareness", 275), ("BallWinning", 312), // roh
    ("Aggression", 384), ("GKAwareness", 326), ("GKCatching", 364), ("GKClearing", 269), ("GKReflexes", 320), ("GKReach", 300), // roh
]; // roh
pub fn attr_de_t() -> [&'static str; 25] { [crate::inhalt::t("ui.pesdb.torinstinkt"), crate::inhalt::t("attribut.Ballkontrolle"), crate::inhalt::t("attribut.dribbling"), crate::inhalt::t("ui.pesdb.ballfuehrung_eng"), crate::inhalt::t("ui.pesdb.flaches_passspiel"), crate::inhalt::t("ui.pesdb.hohes_passspiel"), crate::inhalt::t("attribut.abschluss"), crate::inhalt::t("attribut.Kopfball"),
    crate::inhalt::t("attribut.Standardausfuehrung"), crate::inhalt::t("attribut.Effet"), crate::inhalt::t("attribut.schnelligkeit"), crate::inhalt::t("attribut.antritt"), crate::inhalt::t("attribut.Schusskraft"), crate::inhalt::t("attribut.sprungkraft"), crate::inhalt::t("ui.pesdb.koerpereinsatz"), crate::inhalt::t("attribut.balance"), crate::inhalt::t("attribut.ausdauer"), crate::inhalt::t("ui.pesdb.defensivbewusstsein"),
    crate::inhalt::t("faehigkeit.balleroberung"), crate::inhalt::t("attribut.Aggressivitaet"), crate::inhalt::t("ui.pesdb.torwartgespuer"), crate::inhalt::t("achse.Fangsicherheit"), crate::inhalt::t("ui.pesdb.abwehr_mit_faeusten"), crate::inhalt::t("achse.Reflexe"), crate::inhalt::t("achse.Reichweite")] }
/// Position levels (plbin.POS_BITS): (low bit, high bit); level = lo + 2·hi; GK has only the high bit.
pub const POS_BITS: [(Option<u32>, u32); 13] = [(None, 351), (Some(468), 469), (Some(318), 319), (Some(474), 475), (Some(414), 415), (Some(456), 457), (Some(466), 467),
    (Some(460), 461), (Some(464), 465), (Some(472), 473), (Some(476), 477), (Some(478), 479), (Some(470), 471)];
pub const STYLES: [&str; 22] = ["Balanced", "Goal Poacher", "Dummy Runner", "Fox in the Box", "Prolific Winger", "Classic No. 10", "Hole Player", "Box-to-Box", // roh
    "Anchor Man", "The Destroyer", "Extra Frontman", "Offensive Full-back", "Defensive Full-back", "Target Man", "Creative Playmaker", "Build Up", // roh
    "Offensive Goalkeeper", "Defensive Goalkeeper", "Roaming Flank", "Cross Specialist", "Orchestrator", "Full-back Finisher"]; // roh
/// Special abilities (35, plbin.SKILL_BITS) then COM styles (7, COM_BITS): one bit each; index = bit in Player::skills.
pub const SKILLS: [(&str, u32); 42] = [("Scissors Feint", 519), ("Flip Flap", 485), ("Marseille Turn", 497), ("Sombrero", 482), ("Cut Behind & Turn", 528), // roh
    ("Scotch Move", 525), ("Heading", 495), ("Long Range Drive", 527), ("Acrobatic Finishing", 524), ("Heel Trick", 505), ("First-time Shot", 511), // roh
    ("One-touch Pass", 507), ("Weighted Pass", 484), ("Pinpoint Crossing", 483), ("Outside Curler", 493), ("Low Lofted Pass", 488), ("GK Low Punt", 490), // roh
    ("Long Throw", 521), ("GK Long Throw", 522), ("Gamesmanship", 491), ("Man Marking", 504), ("Track Back", 517), ("Acrobatic Clear", 530), ("Captaincy", 492), // roh
    ("Fighting Spirit", 486), ("Penalty Specialist", 501), ("Chip Shot Control", 506), ("Step On Ball Control", 500), ("Dipping Shot", 494), ("Rising Shots", 499), // roh
    ("Interception", 503), ("Double Touch", 523), ("Cross Over Turn", 287), ("Long Range Shooting", 518), ("Through Passing", 487), // roh
    ("Trickster", 489), ("Mazing Run", 531), ("Speeding Bullet", 526), ("Incisive Run", 510), ("Long Ball Expert", 529), ("Early Cross", 447), ("Long Ranger", 520)]; // roh
const NAME_OFFS: [usize; 3] = [68, 251, 129]; // name, its copy, shirt name (61 bytes UTF-8 each)
const NAME_LEN: usize = 61;

// ---------------------------------------------------------------- rating model (Übergabe §1.6)
static WEIGHTS: std::sync::OnceLock<HashMap<u8, Vec<f32>>> = std::sync::OnceLock::new();
fn weights() -> &'static HashMap<u8, Vec<f32>> {
    WEIGHTS.get_or_init(|| {
        let v: HashMap<String, Vec<f32>> = serde_json::from_str(include_str!("../assets/ovr_weights_fl27.json")).expect("ovr_weights_fl27.json");
        v.into_iter().filter_map(|(k, w)| Some((k.parse().ok()?, w))).collect()
    })
}
/// Weights of a position (convert.wvec: missing positions fall back 6→5, 7→5, 11→12, else 5).
pub fn wvec(pos: u8) -> &'static [f32] {
    let w = weights();
    w.get(&pos).or_else(|| w.get(&match pos { 6 | 7 => 5, 11 => 12, p => p })).or_else(|| w.get(&5)).unwrap()
}
pub fn pos_index(pos: &str) -> u8 { POS.iter().position(|p| *p == pos).unwrap_or(5) as u8 }
/// ovr(pos) = Σ w[pos][i]·attr[i] + w[pos][25] (convert.ovr_model; PES M rounds it for display).
pub fn ovr(attrs: &[f32], pos: u8) -> f32 {
    let w = wvec(pos);
    attrs.iter().zip(w).map(|(a, w)| a * w).sum::<f32>() + w[25]
}
/// Moves the abilities so the model's rating changes by `delta`: each ability that counts for the position moves in
/// proportion to its weight (Σ w·Δa = delta). Abilities stay in 40..99. Like convert.fit_overall, but continuous.
pub fn shift(attrs: &mut [f32], pos: u8, delta: f32) {
    let w = wvec(pos);
    let s: f32 = w[..25].iter().filter(|x| **x > 0.0).map(|x| x * x).sum();
    if s <= 0.0 { return; }
    for (a, wi) in attrs.iter_mut().zip(w) { if *wi > 0.0 { *a = (*a + delta * wi / s).clamp(40.0, 99.0); } }
}

// ---------------------------------------------------------------- Player.bin
pub struct PlayerBin { pub head: [u8; 8], pub raw: Vec<u8>, pub data: Vec<u8>, pub index: HashMap<u32, usize> }
impl PlayerBin {
    pub fn load(p: &Path) -> Result<PlayerBin, String> {
        let raw = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
        let (head, data) = fl27::unpack(&raw)?;
        if data.len() % REC != 0 { return Err(crate::inhalt::t("ui.pesdb.player_bin_satzlaenge_passt_nicht").into()); }
        let index = (0..data.len() / REC).map(|i| (u32::from_le_bytes(data[i * REC + 8..i * REC + 12].try_into().unwrap()), i)).collect();
        Ok(PlayerBin { head, raw, data, index })
    }
    pub fn n(&self) -> usize { self.data.len() / REC }
    pub fn id(&self, i: usize) -> u32 { u32::from_le_bytes(self.data[i * REC + 8..i * REC + 12].try_into().unwrap()) }
    pub fn bits(&self, i: usize, off: u32, w: u32) -> u32 {
        let b = i * REC * 8 + off as usize;
        let o = b >> 3;
        let mut v = [0u8; 4];
        let end = (o + 4).min(self.data.len());
        v[..end - o].copy_from_slice(&self.data[o..end]);
        (u32::from_le_bytes(v) >> (b & 7)) & ((1u32 << w) - 1)
    }
    pub fn set_bits(&mut self, i: usize, off: u32, w: u32, v: u32) {
        let b = i * REC * 8 + off as usize;
        let o = b >> 3;
        let mut cur = [0u8; 4];
        cur.copy_from_slice(&self.data[o..o + 4]);
        let mask = ((1u32 << w) - 1) << (b & 7);
        let n = (u32::from_le_bytes(cur) & !mask) | ((v << (b & 7)) & mask);
        self.data[o..o + 4].copy_from_slice(&n.to_le_bytes());
    }
    pub fn attrs(&self, i: usize) -> [u8; 25] { let mut a = [0u8; 25]; for (k, (_, off)) in ATTRS.iter().enumerate() { a[k] = self.bits(i, *off, 6) as u8 + 40; } a }
    pub fn set_attrs(&mut self, i: usize, a: &[f32]) { for (k, (_, off)) in ATTRS.iter().enumerate() { self.set_bits(i, *off, 6, (a[k].round().clamp(40.0, 99.0) as u32) - 40); } }
    pub fn position(&self, i: usize) -> u8 { self.bits(i, 434, 4) as u8 }
    pub fn set_position(&mut self, i: usize, p: u8) { self.set_bits(i, 434, 4, p as u32); }
    pub fn age(&self, i: usize) -> u8 { self.bits(i, 408, 6) as u8 + 15 }
    pub fn set_age(&mut self, i: usize, a: u8) { self.set_bits(i, 408, 6, (a.clamp(15, 78) - 15) as u32); }
    pub fn height(&self, i: usize) -> u8 { (self.bits(i, 216, 8) + 100).min(255) as u8 }
    /// Height (216, 8 bit + 100), weight (256, 7 bit + 30), foot (514, 1 = left), weak foot use and accuracy (454 and 462,
    /// 2 bit + 1): flconv/plbin.py. Injury resistance and form are not decoded.
    pub fn koerper(&self, i: usize) -> [u8; 5] { [self.height(i), (self.bits(i, 256, 7) + 30) as u8, self.bits(i, 514, 1) as u8, self.bits(i, 454, 2) as u8 + 1, self.bits(i, 462, 2) as u8 + 1] }
    pub fn positions(&self, i: usize) -> [u8; 13] {
        let mut out = [0u8; 13];
        for (k, (lo, hi)) in POS_BITS.iter().enumerate() { out[k] = (lo.map(|l| self.bits(i, l, 1)).unwrap_or(0) + 2 * self.bits(i, *hi, 1)) as u8; }
        out
    }
    pub fn set_positions(&mut self, i: usize, lv: &[u8]) {
        for ((lo, hi), v) in POS_BITS.iter().zip(lv) { let v = (*v).min(2) as u32; if let Some(l) = lo { self.set_bits(i, *l, 1, v & 1); } self.set_bits(i, *hi, 1, v >> 1); }
    }
    /// Nationality (bit 233, 9 bits): found by matching the real data of 304 players (46 countries, every country one
    /// value, every value one country); the numbers are fl26world.txt's country numbers (210 Germany, 204 England).
    pub fn nat(&self, i: usize) -> u16 { self.bits(i, 233, 9) as u16 }
    pub fn set_nat(&mut self, i: usize, n: u16) { self.set_bits(i, 233, 9, n as u32); }
    pub fn style(&self, i: usize) -> u8 { self.bits(i, 155, 5) as u8 }
    pub fn skills(&self, i: usize) -> u64 { SKILLS.iter().enumerate().fold(0u64, |m, (k, (_, o))| m | ((self.bits(i, *o, 1) as u64) << k)) }
    /// Playing style and special abilities (learned or lost through the development paths, entwicklung.rs).
    pub fn set_style_skills(&mut self, i: usize, style: u8, skills: u64) {
        self.set_bits(i, 155, 5, (style as u32).min(STYLES.len() as u32 - 1));
        for (k, (_, o)) in SKILLS.iter().enumerate() { self.set_bits(i, *o, 1, ((skills >> k) & 1) as u32); }
    }
    pub fn name(&self, i: usize) -> String {
        let o = i * REC + NAME_OFFS[0];
        String::from_utf8_lossy(self.data[o..o + NAME_LEN].split(|b| *b == 0).next().unwrap_or(&[])).to_string()
    }
    pub fn set_name(&mut self, i: usize, text: &str) {
        let mut b = text.as_bytes()[..text.len().min(NAME_LEN - 1)].to_vec();
        while std::str::from_utf8(&b).is_err() { b.pop(); } // never cut inside a multi-byte character
        for off in NAME_OFFS { let o = i * REC + off; self.data[o..o + NAME_LEN].fill(0); self.data[o..o + b.len()].copy_from_slice(&b); }
    }
    /// A new record: a copy of `template` with a new id (the rest is set by the caller).
    pub fn add(&mut self, template: usize, id: u32) -> usize {
        let rec = self.data[template * REC..(template + 1) * REC].to_vec();
        let i = self.n();
        self.data.extend_from_slice(&rec);
        self.data[i * REC + 8..i * REC + 12].copy_from_slice(&id.to_le_bytes());
        self.index.insert(id, i);
        i
    }
    /// A new player of our own (youth, regen): the record is copied from a template chosen by the new id among
    /// `templates` (several players of the position, so new players don't share one look), then everything the
    /// format lets us set is set: id, name, position and levels, abilities, age, height, weight, foot, playing style
    /// and special abilities. shortcut: face, nationality and birth date are not decoded (Übergabe §1.1), they stay
    /// those of the template; set them once the fields are known.
    pub fn new_player(&mut self, templates: &[usize], id: u32, name: &str, pos: u8, pp: &[u8], attrs: &[f32], age: u8, style: u8, skills: u64) -> usize {
        let h = (id as u64).wrapping_mul(2654435761) >> 7;
        let t = if templates.is_empty() { 0 } else { templates[(h % templates.len() as u64) as usize] };
        let i = self.add(t, id);
        self.set_name(i, name);
        self.set_position(i, pos);
        self.set_positions(i, pp);
        self.set_attrs(i, attrs);
        self.set_age(i, age);
        // body: a typical height of the position ± 6 cm, weight from the height, left foot more often on the left side (estimated)
        let base: u32 = [190, 187, 178, 178, 182, 180, 176, 176, 177, 175, 175, 177, 184][pos.min(12) as usize];
        let cm = base + (h % 13) as u32 - 6;
        self.set_bits(i, 216, 8, cm - 100);
        self.set_bits(i, 256, 7, cm - 104 + ((h >> 4) % 9) as u32 - 4 - 30);
        let left = matches!(pos, 2 | 6 | 9);
        self.set_bits(i, 514, 1, ((h >> 8) % 100 < if left { 60 } else { 20 }) as u32);
        self.set_bits(i, 155, 5, (style as u32).min(STYLES.len() as u32 - 1));
        for (k, (_, o)) in SKILLS.iter().enumerate() { self.set_bits(i, *o, 1, ((skills >> k) & 1) as u32); }
        i
    }
    /// Container bytes; an unchanged file keeps its original bytes (byte-equal round trip).
    pub fn bytes(&self) -> Vec<u8> { container(&self.raw, self.head, &self.data) }
}
/// Writes a container: unchanged content keeps the file's original bytes, changed content is packed anew (zlib level 9).
pub fn container(raw: &[u8], head: [u8; 8], data: &[u8]) -> Vec<u8> {
    if fl27::unpack(raw).map(|x| x.1 == data).unwrap_or(false) { raw.to_vec() } else { fl27::pack(head, data) }
}

// ---------------------------------------------------------------- the world from a pesdb folder
pub struct DbPlayer { pub id: u32, pub nat: u16, pub name: String, pub pos: String, pub age: u8, pub ovr: f32, pub no: u8, pub attrs: Vec<f32>, pub pp: Vec<u8>, pub style: u8, pub skills: u64, pub koerper: [u8; 5] }
pub struct DbTeam { pub id: u32, pub name: String, pub squad: Vec<DbPlayer> }
pub struct DbLeague { pub id: u16, pub key: String, pub teams: Vec<usize>, pub name: String, pub land: String, pub stufe: u8, pub ueber: Option<String>, pub auf: u8, pub ab: u8, pub pokal: Option<String>, pub land_nr: u32 }
pub struct DbWorld { pub cids: HashMap<String, u16>, pub leagues: Vec<DbLeague>, pub teams: Vec<DbTeam>, pub pool: Vec<DbPlayer>, pub cups: Vec<(String, Vec<usize>)>, pub euro: [Vec<usize>; 3] }

fn rd(dir: &Path, name: &str) -> Result<Vec<u8>, String> { fl27::pesdb_datei(dir, name) }
fn txt(b: &[u8]) -> String { String::from_utf8_lossy(b.split(|x| *x == 0).next().unwrap_or(&[])).trim().to_string() }

/// Names, countries and the pyramid of the leagues: <Welt>/fl26world.txt for FL leagues (country groups by its
/// `country` number, `above`/`cup` checked against league lines and tournament ids), wettbewerbsnamen.json for the
/// fixed keys, "England 1" only as the last resort. A country is named after a fixed league of the same
/// Competition.bin country byte, otherwise "Land der <Liga der ersten Stufe>" (never an invented name).
fn liga_meta(dir: &Path, comp_bytes: &[u8], comps: &[(u16, String)], ligen: &mut [DbLeague]) {
    let fl = dir.ancestors().nth(3).and_then(|w| fl27::datei_ci(w, "fl26world.txt")).and_then(|p| std::fs::read_to_string(p).ok()).map(|t| fl27::fl26world(&t)).unwrap_or_default();
    let region = fl27::regionen(comp_bytes);
    let turnier = rd(dir, "CompetitionRegulation.bin").map(|d| fl27::turnier_ids(&d)).unwrap_or_default();
    let key_of: HashMap<u16, &str> = comps.iter().map(|(c, k)| (*c, k.as_str())).collect();
    // a number of the league space → competition id: a league line first, then a tournament id
    let cid_von = |n: u32| fl.iter().find(|l| l.n == n).map(|l| l.cid).or_else(|| turnier.iter().find(|(_, t)| **t as u32 == n).map(|(c, _)| *c));
    // a country byte names a country only when it is unambiguous: one fixed country and at most one fl26world country number
    let mut region_land: HashMap<u8, Option<String>> = HashMap::new();
    for (c, k) in comps { if let (Some(l), Some(r)) = (crate::world::fest_land(k), region.get(c)) { let e = region_land.entry(*r).or_insert(Some(l.clone())); if e.as_deref() != Some(l.as_str()) { *e = None; } } }
    let mut byte_laender: HashMap<u8, HashSet<u32>> = HashMap::new();
    for f in &fl { if let Some(r) = region.get(&f.cid) { byte_laender.entry(*r).or_default().insert(f.country); } }
    let eindeutig = |r: &u8| -> Option<String> { if byte_laender.get(r).map_or(0, |s| s.len()) > 1 { return None; } region_land.get(r).cloned().flatten() };
    let land_von_nummer = |country: u32| -> String {
        let gruppe: Vec<&fl27::FlLiga> = fl.iter().filter(|l| l.country == country).collect();
        let namen: HashSet<String> = gruppe.iter().filter_map(|l| region.get(&l.cid).and_then(|r| eindeutig(r))).collect();
        if namen.len() == 1 && gruppe.iter().all(|l| region.get(&l.cid).map_or(true, |r| !region_land.contains_key(r) || eindeutig(r).is_some())) { return namen.into_iter().next().unwrap(); }
        gruppe.iter().min_by_key(|l| l.tier).map(|l| crate::tf!("ui.pesdb.land_der", l.name)).unwrap_or_default()
    };
    for l in ligen.iter_mut() {
        if let Some(f) = fl.iter().find(|f| f.cid == l.id) {
            l.name = f.name.clone();
            l.land = land_von_nummer(f.country);
            l.stufe = f.tier;
            l.land_nr = f.country;
            l.ueber = f.above.and_then(cid_von).and_then(|c| key_of.get(&c)).map(|k| k.to_string());
            (l.auf, l.ab) = (f.promote, f.demote);
            l.pokal = f.cup.and_then(cid_von).and_then(|c| key_of.get(&c)).filter(|k| k.ends_with(crate::id("CUP"))).map(|k| k.to_string());
        } else {
            l.name = crate::world::echter_name(&l.key).unwrap_or_else(|| crate::world::league_name(&l.key));
            l.land = crate::world::fest_land(&l.key).or_else(|| region.get(&l.id).and_then(|r| eindeutig(r))).unwrap_or_default();
            l.stufe = crate::world::fest_stufe(&l.key);
        }
        if l.name.starts_with("FL ") { l.name = if l.land.is_empty() { crate::tf!("ui.pesdb.liga", l.id) } else { crate::tf!("ui.pesdb.liga_2", l.id, l.land) }; } // roh
    }
}

/// Reads clubs of the leagues with squads, and the pool of players without a league club (welt.py import_pesdb):
/// a league ends in _LEAGUE, does not start with UEFA_/AFC_, has at least 10 teams with at least 11 players each;
/// every player belongs to one league club only (the first one in league order).
pub fn read_world(dir: &Path) -> Result<DbWorld, String> {
    let team = rd(dir, "Team.bin")?;
    let mut names: HashMap<u32, String> = HashMap::new();
    for r in team.chunks_exact(1532) {
        let id = u32::from_le_bytes(r[8..12].try_into().unwrap());
        let n = txt(&r[368..438]);
        names.insert(id, if n.is_empty() { txt(&r[158..228]) } else { n });
    }
    let comp_bytes = rd(dir, "Competition.bin")?;
    let comps = fl27::competitions(&comp_bytes);
    let ce = rd(dir, "CompetitionEntry.bin")?;
    let mut comp_teams: HashMap<u16, Vec<u32>> = HashMap::new();
    for r in ce.chunks_exact(12) {
        let t = u32::from_le_bytes(r[0..4].try_into().unwrap());
        // 16-bit competition id: (b & 0xff) | (b >> 24) << 8 of the u32 at +8 (byte 1 of it is the slot)
        let c = r[8] as u16 | (r[11] as u16) << 8;
        if names.contains_key(&t) { comp_teams.entry(c).or_default().push(t); }
    }
    let pb = PlayerBin::load(&dir.join("Player.bin"))?;
    let (_, asg) = fl27::asg_laden(dir)?;
    let mut squads: HashMap<u32, Vec<(u32, u8)>> = HashMap::new();
    for a in &asg { if pb.index.contains_key(&a.pid) { squads.entry(a.team).or_default().push((a.pid, a.nummer() as u8)); } }
    let mk = |pid: u32, no: u8| -> DbPlayer {
        let i = pb.index[&pid];
        let a = pb.attrs(i);
        let p = pb.position(i).min(12);
        let attrs: Vec<f32> = a.iter().map(|x| *x as f32).collect();
        DbPlayer { id: pid, nat: pb.nat(i), name: pb.name(i), pos: POS[p as usize].into(), age: pb.age(i), ovr: ovr(&attrs, p), no, pp: pb.positions(i).to_vec(), style: pb.style(i), skills: pb.skills(i), koerper: pb.koerper(i), attrs }
    };
    let squad_ok = |t: u32| squads.get(&t).map(|s| s.len() >= 11).unwrap_or(false);
    let mut out = DbWorld { cids: comps.iter().map(|(c, k)| (k.clone(), *c)).collect(), leagues: vec![], teams: vec![], pool: vec![], cups: vec![], euro: [vec![], vec![], vec![]] };
    let mut taken: HashSet<u32> = HashSet::new();
    let mut team_idx: HashMap<u32, usize> = HashMap::new();
    for (cid, key) in &comps {
        let ts = comp_teams.get(cid).cloned().unwrap_or_default();
        if !key.ends_with("_LEAGUE") || key.starts_with("UEFA_") || key.starts_with("AFC_") || ts.len() < 10 || !ts.iter().all(|t| squad_ok(*t)) { continue; }
        // a club in two leagues stays in the first; a league left with fewer than 10 clubs is not a league
        let ts: Vec<u32> = ts.into_iter().filter(|t| !team_idx.contains_key(t)).collect();
        if ts.len() < 10 { continue; }
        let mut lt = vec![];
        for t in ts {
            let mut sq: Vec<DbPlayer> = squads[&t].iter().filter(|(p, _)| taken.insert(*p)).map(|(p, no)| mk(*p, *no)).collect();
            sq.sort_by(|a, b| b.ovr.partial_cmp(&a.ovr).unwrap());
            team_idx.insert(t, out.teams.len());
            lt.push(out.teams.len());
            out.teams.push(DbTeam { id: t, name: names[&t].clone(), squad: sq });
        }
        out.leagues.push(DbLeague { id: *cid, key: key.clone(), teams: lt, name: String::new(), land: String::new(), stufe: 0, ueber: None, auf: 3, ab: 3, pokal: None, land_nr: 0 });
    }
    liga_meta(dir, &comp_bytes, &comps, &mut out.leagues);
    // cups (Übergabe §3.4, welt.py import_pesdb): name ends in CUP, not one of sim.json cups.exclude, at least 8 teams from the leagues
    const EXCLUDE: [&str; 7] = [crate::id("WORLD"), crate::id("NATIONS"), crate::id("ASIA"), crate::id("SUPER"), crate::id("CUSTOM"), crate::id("UEFA"), crate::id("CLUB")];
    for (cid, key) in &comps {
        if !key.ends_with(crate::id("CUP")) || EXCLUDE.iter().any(|b| key.contains(b)) { continue; }
        let ts: Vec<usize> = comp_teams.get(cid).map(|v| v.iter().filter_map(|t| team_idx.get(t).copied()).collect()).unwrap_or_default();
        if ts.len() >= 8 { out.cups.push((key.clone(), ts)); }
    }
    // European entries of the database for the first season (uefa.json db_comps: CL 2, EL 3, ECL 174; uefa.py _db_entries)
    for (k, cid) in [2u16, 3, 174].iter().enumerate() {
        out.euro[k] = comp_teams.get(cid).map(|v| v.iter().filter_map(|t| team_idx.get(t).copied()).collect()).unwrap_or_default();
    }
    let known: HashSet<u32> = squads.values().flatten().map(|x| x.0).collect();
    for i in 0..pb.n() { let id = pb.id(i); if !known.contains(&id) { out.pool.push(mk(id, 0)); } }
    Ok(out)
}

/// A world folder (Übergabe §7): from any path upwards the folder X whose parent is "livecpk" and grandparent "SiderAddons".
/// Returns (game folder, world folder).
pub fn ableiten(p: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut cur = Some(p);
    while let Some(c) = cur {
        let par = c.parent()?;
        if par.file_name()?.to_str()?.eq_ignore_ascii_case(crate::id("livecpk")) && par.parent()?.file_name()?.to_str()?.eq_ignore_ascii_case(crate::id("SiderAddons")) {
            return Some((par.parent()?.parent()?.to_path_buf(), c.to_path_buf()));
        }
        cur = c.parent();
    }
    None
}
/// A world is a livecpk folder with Team.bin and Player.bin (and the other required files), whose name does not start with PESM.
pub fn pruefen(welt: &Path) -> Result<(), String> {
    let name = welt.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.to_uppercase().starts_with(crate::id("PESM")) { return Err(crate::inhalt::t("ui.pesdb.das_ist_ein_ordner_von").into()); }
    let d = welt.join("common/etc/pesdb");
    for f in ["Team.bin", "Player.bin", "PlayerAssignment.bin", "Competition.bin", "CompetitionEntry.bin"] {
        if !d.join(f).exists() { return Err(crate::tf!("ui.pesdb.in_diesem_ordner_fehlt_die", f)); }
    }
    Ok(())
}
/// All worlds in the game's livecpk folder.
pub fn welten(s: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(s.join(crate::id("livecpk"))).map(|d| d.flatten().map(|e| e.path()).filter(|p| pruefen(p).is_ok()).collect()).unwrap_or_default();
    v.sort();
    v
}

#[cfg(test)]
mod t {
    use super::*;
    fn def() -> Option<PathBuf> { let p = fl27::sider().join("livecpk/_FL26-27-Def"); fl27::echte_welt(&p.join("common/etc/pesdb")).then_some(p) }

    #[test]
    fn rating_model_and_round_trip() {
        let Some(w) = def() else { return };
        let dir = w.join("common/etc/pesdb");
        let pb = PlayerBin::load(&dir.join("Player.bin")).unwrap();
        // the model within 1 point of the Python version's value for 50 known players
        let refs: Vec<(u32, f32, String, u8)> = serde_json::from_str(include_str!("../tests_data/ovr50.json")).unwrap();
        for (id, want, pos, age) in &refs {
            let i = pb.index[id];
            let a: Vec<f32> = pb.attrs(i).iter().map(|x| *x as f32).collect();
            let got = ovr(&a, pb.position(i));
            assert!((got.round() - want).abs() <= 1.0, "{id}: {got} vs {want}");
            assert_eq!((POS[pb.position(i) as usize], pb.age(i)), (pos.as_str(), *age));
            assert_eq!(pb.positions(i)[pb.position(i) as usize], 2);
        }
        // every file read and written back unchanged is byte-equal
        for f in ["Player.bin", "PlayerAssignment.bin", "Team.bin", "Competition.bin", "CompetitionEntry.bin", "CompetitionRegulation.bin", "Tactics.bin", "TacticsFormation.bin", "Coach.bin"] {
            let raw = std::fs::read(dir.join(f)).unwrap();
            let (h, d) = fl27::unpack(&raw).unwrap();
            assert_eq!(container(&raw, h, &d), raw, "{f}");
        }
        assert_eq!(pb.bytes(), pb.raw);
        // set and read back: abilities, age, position levels, name, a new record
        let mut pb2 = PlayerBin::load(&dir.join("Player.bin")).unwrap();
        let i = pb2.index[&refs[0].0];
        let mut a: Vec<f32> = pb2.attrs(i).iter().map(|x| *x as f32).collect();
        shift(&mut a, pb2.position(i), 2.0);
        pb2.set_attrs(i, &a);
        pb2.set_age(i, 30);
        let cbs: Vec<usize> = (0..pb2.n()).filter(|&k| pb2.position(k) == 1).take(50).collect();
        let j = pb2.new_player(&cbs, 9_000_001, "Jonas Krüger", 5, &[0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0], &a, 17, 0, 0);
        let j2 = pb2.new_player(&cbs, 9_000_002, "Ole Brandt", 1, &[0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], &a, 18, 0, 0);
        assert_ne!(pb2.data[j * REC + 12..j * REC + 68], pb2.data[j2 * REC + 12..j2 * REC + 68]); // two new players, two looks
        assert_eq!((pb2.style(j), pb2.skills(j), pb2.position(j), pb2.age(j2)), (0, 0, 5, 18));
        assert!((150..=210).contains(&pb2.height(j)));
        let back = fl27::unpack(&pb2.bytes()).unwrap().1;
        assert_eq!(back, pb2.data);
        assert_eq!((pb2.age(i), pb2.name(j), pb2.id(j), pb2.positions(j)[5]), (30, "Jonas Krüger".to_string(), 9_000_001, 2));
        let got = ovr(&pb2.attrs(i).iter().map(|x| *x as f32).collect::<Vec<_>>(), pb2.position(i));
        assert!((got - refs[0].1).abs() <= 3.5, "{got}");
    }

    #[test]
    fn megapack_16_bit() {
        // the FL_MegaPack has 726 competitions: ids above 255 must not merge leagues (Competition.bin byte 5 | byte 7 << 8)
        let w = std::path::PathBuf::from("/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk/_FL26Megapack");
        if !fl27::echte_welt(&w) { return; }
        let p = w.join("common/etc/pesdb");
        let d = read_world(&p).unwrap();
        let mut seen = HashSet::new();
        for l in &d.leagues { assert!(l.teams.len() >= 10, "{} has {} clubs", l.key, l.teams.len()); for &t in &l.teams { assert!(seen.insert(t), "club in two leagues"); } }
        assert!(d.leagues.iter().any(|l| l.id > 255), "leagues with 16-bit ids are read");
        let w = crate::world::World::from_pesdb(&p, 3).unwrap();
        let own = w.leagues[0].teams[0];
        let mut c = crate::game::Career::new(w, own, "Test");
        c.mensch.vertrag_bis = u32::MAX;
        for _ in 0..crate::date::SEASON_DAYS {
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            c.ultimatum = None;
            c.end_day().unwrap();
        }
        assert_eq!(c.world.season, 1, "a whole season ran");
        // profile page: body from Player.bin, a station per season, the scout view
        let k = d.teams[0].squad[0].koerper;
        assert!((150..=210).contains(&k[0]) && (50..=110).contains(&k[1]) && k[2] <= 1 && (1..=4).contains(&k[3]) && (1..=4).contains(&k[4]), "{k:?}");
        let own_p = c.squad()[0];
        assert!(c.world.players.iter().any(|p| p.profil.laufbahn.iter().any(|s| s.0 == 0 && s.2 > 0)), "the finished season is a station");
        assert!(crate::pages::bekannte_werte(&c, own_p).iter().all(|x| *x), "own players: every value");
        let fremd = (0..c.world.players.len()).find(|&p| c.world.players[p].team >= 0 && c.world.players[p].team != c.own as i32 && c.world.players[p].attrs.len() == 25).unwrap();
        c.world.players[fremd].know = 0.0;
        assert!(crate::pages::bekannte_werte(&c, fremd).iter().all(|x| !*x), "nothing known without scouting");
        c.world.players[fremd].know = 0.5;
        let n = crate::pages::bekannte_werte(&c, fremd).iter().filter(|x| **x).count();
        assert!(n > 0 && n < 25, "part known: {n}");
        assert!(crate::pages::aehnliche(&c, fremd).iter().all(|&q| c.world.players[q].pos == c.world.players[fremd].pos && q != fremd));
        assert!(crate::pages::robustheit(&c.world.players[fremd]).map_or(false, |r| (1.0..=99.0).contains(&r)));
        // ours everywhere: a player loaned to another club and an academy player show all 25 values, radar and robustness
        let sq = c.squad();
        let (leih, aka) = (sq[1], sq[2]);
        let ziel = c.world.players[fremd].team as usize;
        c.world.clubs[c.own].squad.retain(|&x| x != leih);
        c.world.clubs[ziel].squad.push(leih);
        c.world.players[leih].team = ziel as i32;
        c.world.players[leih].know = 0.0;
        c.world.players[leih].leihe = Some(crate::transfer::Leihe { von: c.own, bis: c.world.season, anteil: 0.5, option: 0.0, spielzeit: false, grund: String::new() });
        c.world.players[aka].youth = 19;
        for q in [leih, aka] {
            assert!(c.gehoert_uns(q) && c.know_of(q) >= 1.0, "ours");
            assert!(crate::pages::bekannte_werte(&c, q).iter().all(|x| *x), "all 25 values (and so the radar)");
            assert!(crate::pages::robustheit(&c.world.players[q]).is_some());
            assert_eq!(c.ovr_range(q).0, c.ovr_range(q).1, "exact strength");
        }
        assert!(!c.gehoert_uns(fremd));
    }
    #[test]
    fn ligennamen_aus_der_welt() {
        let base = std::path::PathBuf::from("/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk");
        let my = base.join("_FL26MyWorld");
        if fl27::echte_welt(&my) {
            let d = read_world(&my.join("common/etc/pesdb")).unwrap();
            let l = d.leagues.iter().find(|l| l.key == "FL_011_LEAGUE").expect("FL_011");
            assert_eq!((l.name.as_str(), l.land.as_str(), l.stufe), ("2. Bundesliga", "Deutschland", 2));
            assert_eq!(l.ueber.as_deref(), Some("GERMANY_D1_LEAGUE"), "above=50 is a tournament id, not SPAIN_D1_LEAGUE");
            let l3 = d.leagues.iter().find(|l| l.key == "FL_049_LEAGUE").expect("FL_049");
            assert_eq!(l3.ueber.as_deref(), Some("FL_011_LEAGUE"));
            if let Some(a) = d.leagues.iter().find(|l| l.key == "FL_076_LEAGUE") { assert_eq!(a.land, "Land der ADMIRAL Bundesliga"); }
        }
        if let Some(w) = def() {
            let d = read_world(&w.join("common/etc/pesdb")).unwrap();
            let l = d.leagues.iter().find(|l| l.key == "ENGLAND_D1_LEAGUE").unwrap();
            assert_eq!((l.name.as_str(), l.land.as_str()), ("Premier League", "England"));
        }
        // one country name ↔ one fl26world country number, in every world; known samples per world
        let pruef = |welt: &std::path::Path, proben: &[(&str, &str)]| {
            let d = read_world(&welt.join("common/etc/pesdb")).unwrap();
            let fl = fl27::fl26world(&std::fs::read_to_string(fl27::datei_ci(welt, "fl26world.txt").unwrap()).unwrap());
            let mut nummern: HashMap<String, HashSet<u32>> = HashMap::new();
            for l in &d.leagues { if let Some(f) = fl.iter().find(|f| f.cid == l.id) { nummern.entry(l.land.clone()).or_default().insert(f.country); } }
            for (land, n) in &nummern { assert_eq!(n.len(), 1, "{}: {land} steht für mehrere Länder {n:?}", welt.display()); }
            for (liga, land) in proben { assert_eq!(d.leagues.iter().find(|l| l.name == *liga).map(|l| l.land.as_str()), Some(*land), "{liga}"); }
        };
        if fl27::echte_welt(&my) { pruef(&my, &[("3. Liga", "Deutschland"), ("Sky Bet League One", "England"), ("Trendyol 1. Lig", "Türkei")]); }
        if let Some(w) = def() { pruef(&w, &[("EFL League One", "England"), ("Russian First League", "Russland"), ("Meiji Yasuda J2 League", "Japan")]); }
        let mp = std::path::PathBuf::from("/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk/_FL26Megapack");
        if fl27::echte_welt(&mp) { pruef(&mp, &[("Regionalliga Bayern", "Deutschland"), ("Scottish League Two", "Schottland"), ("Serie C Girone A", "Italien")]); }
        if fl27::echte_welt(&mp) {
            let d = read_world(&mp.join("common/etc/pesdb")).unwrap();
            for l in &d.leagues { assert!(!l.name.starts_with("FL ") && l.name != format!("Liga {}", l.id) && !l.name.starts_with(&format!("Liga {} (", l.id)), "{} heißt {}", l.key, l.name); assert!(!l.land.is_empty(), "{} ohne Land", l.key); }
        }
    }
    #[test]
    fn teamsuche() {
        let base = std::path::PathBuf::from("/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk");
        if let Some(w) = def() {
            let w = crate::world::World::from_pesdb(&w.join("common/etc/pesdb"), 7).unwrap();
            let idx = w.such_index();
            let t = w.suche(&idx, "sunder", None, None);
            assert!(t.first().map(|&c| w.clubs[c].name.contains("Sunderland")).unwrap_or(false), "sunder → Sunderland");
            let t0 = std::time::Instant::now();
            for q in ["m", "mu", "mun", "munc"] { w.suche(&idx, q, None, None); }
            assert!(t0.elapsed().as_millis() < 200, "the search keeps up with typing");
        }
        let my = base.join("_FL26MyWorld");
        if fl27::echte_welt(&my) {
            let w = crate::world::World::from_pesdb(&my.join("common/etc/pesdb"), 7).unwrap();
            let idx = w.such_index();
            let t = w.suche(&idx, "bundesliga", None, None);
            let bl = w.leagues.iter().position(|l| l.key == "GERMANY_D1_LEAGUE").unwrap();
            assert!(w.leagues[bl].teams.iter().all(|c| t.contains(c)), "bundesliga finds every Bundesliga club");
            let de = w.suche(&idx, "bundesliga", Some("Deutschland"), None);
            assert!(de.iter().all(|&c| w.leagues[w.clubs[c].league].land == "Deutschland") && !de.is_empty());
            if let Some(b) = w.clubs.iter().position(|c| c.name.contains("München")) { assert!(w.suche(&idx, "munchen", None, None).contains(&b), "ohne Akzent"); }
        }
    }
    #[test]
    fn whole_world_counts() {
        let Some(w) = def() else { return };
        pruefen(&w).unwrap();
        let (spiel, welt) = ableiten(&w.join("common/etc/pesdb")).unwrap();
        assert!(welt.ends_with("_FL26-27-Def") && spiel.ends_with("SP Football Life 2027"));
        let d = read_world(&w.join("common/etc/pesdb")).unwrap();
        assert_eq!(d.leagues.len(), 99);
        assert_eq!(d.teams.len(), 1643);
        assert_eq!(d.teams.iter().map(|t| t.squad.len()).sum::<usize>(), 48777);
        assert!(d.teams.iter().any(|t| t.id == 396 && t.name.contains("Sunderland")));
        assert!(d.cups.iter().any(|c| c.0 == "ENGLAND_D1_CUP" && c.1.len() >= 8), "{:?}", d.cups.iter().map(|c| &c.0).collect::<Vec<_>>());
        assert!(d.euro[0].len() >= 20, "{}", d.euro[0].len());
    }
}
