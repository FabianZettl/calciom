//! Faces of new players (youth, regens, Fabian 24.11.). FL27 finds a player's appearance (head, skin, hair: a 56-byte
//! record of PlayerAppearance.bin) by his id, so a player PES M adds has none and wears the default look. fl26regen
//! (FL Mod Studio, modules/fl26regen.dll) reads "faceapp <to>:<from>" lines in modules/fl26world.txt at the game's start
//! and copies the appearance record of `from` to `to` in memory (fl26regen.c fl26_regen_copy/apply_copies, at most
//! 4096 copies). PES M picks `from` once per player (World.gesichter): a real player of his nationality, now and then
//! one of a country his family could come from (jugendakademie.toml [gesicht]). The line appears once he plays for a
//! player PES M writes into the career's Player.bin (academy players too: FL27 shows them, 24.11.), bound to his id.
use crate::game::Career;
use crate::regeln::{f, wert};
use crate::world::Rng;
use std::collections::HashSet;
use std::path::Path;

const ANFANG: &str = crate::id("# PES M faces begin (written by PES M at every match start)");
const ENDE: &str = crate::id("# PES M faces end");
const KOPIEN_MAX: usize = 4096; // fl26regen.c MAX_COPY

/// Ids in "newfaces"/"newfaces3d" lines: the world builder's own new players, who have no record of their own to copy.
pub fn newfaces(text: &str) -> HashSet<u32> {
    let mut s = HashSet::new();
    for l in text.lines() {
        let Some(rest) = l.trim().strip_prefix(crate::id("newfaces")) else { continue };
        let rest = rest.strip_prefix(crate::id("3d")).unwrap_or(rest);
        for t in rest.split_whitespace() {
            let mut it = t.splitn(2, '-').map(|x| x.parse::<u32>().ok());
            let (Some(Some(a)), b) = (it.next(), it.next()) else { continue };
            let b = b.flatten().unwrap_or(a);
            s.extend(a..=b.min(a + 200_000));
        }
    }
    s
}

/// The text with PES M's block replaced (or appended, or removed when there are no pairs), in the file's line ending.
pub fn block_setzen(text: &str, paare: &[(u32, u32)]) -> String {
    let nl = if text.contains(crate::id("\r\n")) { crate::id("\r\n") } else { crate::id("\n") };
    let mut aus: Vec<&str> = vec![];
    let mut drin = false;
    for l in text.lines() {
        let l = l.trim_end_matches('\r');
        if l == ANFANG { drin = true; continue; }
        if l == ENDE { drin = false; continue; }
        if !drin { aus.push(l); }
    }
    while aus.last().is_some_and(|l| l.is_empty()) { aus.pop(); }
    let mut s = aus.join(nl);
    if !paare.is_empty() {
        s.push_str(nl); s.push_str(ANFANG); s.push_str(nl);
        for c in paare.chunks(16) {
            s.push_str(crate::id("faceapp"));
            for (to, from) in c { s.push_str(&format!(" {to}:{from}")); }
            s.push_str(nl);
        }
        s.push_str(ENDE);
    }
    s.push_str(nl);
    s
}

/// Nationality of the face: mostly the player's own, with chance [gesicht] andere one of his country's origins (weighted).
pub fn herkunft(nat: u16, r: &mut Rng) -> u16 {
    let liste: Vec<(u16, f32)> = wert("jugendakademie", "gesicht.herkunft").get(nat.to_string()).and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| Some((x.get(0)?.as_u64()? as u16, x.get(1)?.as_f64()? as f32))).collect()).unwrap_or_default();
    let summe: f32 = liste.iter().map(|x| x.1).sum();
    if summe <= 0.0 || !r.chance(f("jugendakademie", "gesicht.andere")) { return nat; }
    let mut z = r.f() * summe;
    for (n, w) in &liste { if z < *w { return *n; } z -= w; }
    nat
}

/// Ids of the world's own Player.bin (players FL27 knows, with an appearance of their own unless listed in newfaces);
/// read once per world and session.
fn welt_ids(welt: &Path) -> HashSet<u32> {
    static C: std::sync::Mutex<Option<(std::path::PathBuf, std::sync::Arc<HashSet<u32>>)>> = std::sync::Mutex::new(None);
    let mut g = C.lock().unwrap();
    if let Some((p, s)) = g.as_ref() { if p == welt { return (**s).clone(); } }
    let s: HashSet<u32> = crate::pesdb::PlayerBin::load(&welt.join(crate::id("common/etc/pesdb/Player.bin"))).map(|pb| pb.index.keys().copied().collect()).unwrap_or_default();
    *g = Some((welt.to_path_buf(), std::sync::Arc::new(s.clone())));
    s
}

const APPEARANCE: &str = crate::id("common/character0/model/character/appearance/PlayerAppearance.bin");
/// Ids whose appearance has a beard (FacialHairType ≠ 0), from the game's PlayerAppearance.bin (download/*.cpk wins over
/// Data/*.cpk, as the game loads them; FL27: download/data_s2627.cpk). On disk 60 bytes a record: u32 id + the 56 bytes
/// that follow the id and 8 zero bytes in memory, so FacialHairType (memory bit 371, 5 bits) is disk bit 307.
/// Read once per session; not found: empty (no filter).
/// Ids with a real face model (Asset/model/character/face/real/<id>/ in a livecpk folder or a CPK of the game). Their
/// appearance record carries only body and skin, the head is the model, so a copy of it looks like the default face
/// (Elijah Alker, 24.11.). Read once per session.
pub fn echte_gesichter(spiel: &Path) -> HashSet<u32> {
    static C: std::sync::OnceLock<HashSet<u32>> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        let mut s = HashSet::new();
        let real = crate::id("asset/model/character/face/real/");
        for e in std::fs::read_dir(spiel.join(crate::id("SiderAddons/livecpk"))).into_iter().flatten().flatten() {
            for d in std::fs::read_dir(e.path().join(crate::id("Asset/model/character/face/real"))).into_iter().flatten().flatten() {
                if let Some(id) = d.file_name().to_str().and_then(|n| n.parse().ok()) { s.insert(id); }
            }
        }
        for ordner in [crate::id("download"), crate::id("Data")] {
            for c in std::fs::read_dir(spiel.join(ordner)).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case(crate::id("cpk")))) {
                let Ok(cpk) = pesm_coaching_zone::cpk::Cpk::open(&c) else { continue };
                for n in cpk.files.keys() {
                    if let Some(id) = n.strip_prefix(real).and_then(|r| r.split('/').next()).and_then(|x| x.parse().ok()) { s.insert(id); }
                }
            }
        }
        s
    }).clone()
}
/// The game's PlayerAppearance.bin records (unpacked), read once per session; empty when not found.
fn aussehen(spiel: &Path) -> &'static [u8] {
    static C: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        for ordner in [crate::id("download"), crate::id("Data")] {
            let mut cpks: Vec<_> = std::fs::read_dir(spiel.join(ordner)).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case(crate::id("cpk")))).collect();
            cpks.sort();
            for c in cpks.iter().rev() {
                let Ok(cpk) = pesm_coaching_zone::cpk::Cpk::open(c) else { continue };
                if !cpk.has(APPEARANCE) { continue; }
                let Ok(b) = cpk.read(APPEARANCE) else { continue };
                let Ok((_, d)) = crate::fl27::unpack(&b) else { continue };
                return d;
            }
        }
        vec![]
    })
}
pub fn baertige(spiel: &Path) -> HashSet<u32> { baertige_aus(aussehen(spiel)) }
/// Ids that have an appearance record of their own (only these can be copied).
pub fn mit_aussehen(spiel: &Path) -> HashSet<u32> { aussehen(spiel).chunks_exact(60).map(|r| u32::from_le_bytes(r[..4].try_into().unwrap())).collect() }
pub fn baertige_aus(d: &[u8]) -> HashSet<u32> {
    d.chunks_exact(60).filter(|r| (u64::from_le_bytes(r[38..46].try_into().unwrap()) >> 3) & 31 != 0).map(|r| u32::from_le_bytes(r[..4].try_into().unwrap())).collect()
}

/// A player's look from his appearance record (layout: PES Edit Face AI docs/appearance-record.md, checked against
/// 1,879 player photos 25.11.): skin 1 (fair) … 6 (very dark); hair colour as the game's 6-bit RGB and its nearest
/// preset step 1 (black) … 7 (blond); hair style (HairStyle: 0 bald, 4 afro); beard (FacialHairType ≠ 0).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Aussehen { pub haut: u8, pub haar: [u8; 3], pub haar_stufe: u8, pub stil: u8, pub bart: bool }
impl Aussehen { pub fn glatze(&self) -> bool { self.stil == 0 } }
/// The game's seven hair colour presets (HairColourType 1–7), 6-bit RGB.
const HAAR_STUFEN: [[u8; 3]; 7] = [[5, 4, 3], [8, 5, 3], [11, 7, 4], [15, 8, 4], [14, 10, 7], [18, 14, 10], [33, 28, 21]];
/// One bit field of a disk record (lowest bit first; disk bit = memory bit − 64).
fn feld(r: &[u8], bit: usize, breite: u32) -> u8 {
    let b = bit / 8;
    let mut x = [0u8; 4];
    let n = (r.len() - b).min(4);
    x[..n].copy_from_slice(&r[b..b + n]);
    ((u32::from_le_bytes(x) >> (bit % 8)) & ((1 << breite) - 1)) as u8
}
fn satz_aussehen(r: &[u8]) -> Aussehen {
    let rgb = [feld(r, 366, 6), feld(r, 372, 6), feld(r, 378, 6)];
    let stufe = feld(r, 460, 3);
    // the RGB is the colour itself; 53 records hold none (0/0/0) and keep their preset
    let haar = if rgb == [0, 0, 0] && (1..=7).contains(&stufe) { HAAR_STUFEN[stufe as usize - 1] } else { rgb };
    let abstand = |s: &[u8; 3]| (0..3).map(|k| (s[k] as i32 - haar[k] as i32).pow(2)).sum::<i32>();
    let haar_stufe = (1..=7u8).min_by_key(|&k| abstand(&HAAR_STUFEN[k as usize - 1])).unwrap();
    Aussehen { haut: feld(r, 296, 3).clamp(1, 6), haar, haar_stufe, stil: feld(r, 384, 3), bart: feld(r, 307, 5) != 0 }
}
pub fn aussehen_aus(d: &[u8]) -> std::collections::HashMap<u32, Aussehen> {
    d.chunks_exact(60).map(|r| (u32::from_le_bytes(r[..4].try_into().unwrap()), satz_aussehen(r))).collect()
}
/// New player → the real player whose face he wears (World.gesichter of the open career), for the drawn pictures.
static SPENDER: std::sync::Mutex<std::collections::BTreeMap<u32, u32>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
/// Tells the pictures which faces the open career's new players wear (called each frame; copies only on a change).
pub fn spender_setzen(m: &std::collections::BTreeMap<u32, u32>) {
    if let Ok(mut g) = SPENDER.lock() { if *g != *m { *g = m.clone(); } }
}
/// The look of a player in FL27: his own record, for a new player the record of the face he wears; None without one
/// (no game folder, or a player FL27 has no appearance for). The game's file is read once per session.
pub fn aussehen_von(id: u32) -> Option<Aussehen> {
    static C: std::sync::OnceLock<std::collections::HashMap<u32, Aussehen>> = std::sync::OnceLock::new();
    let m = C.get_or_init(|| aussehen_aus(aussehen(&crate::fl27::spielordner())));
    let q = SPENDER.lock().ok().and_then(|g| g.get(&id).copied()).unwrap_or(id);
    m.get(&q).copied()
}

impl Career {
    /// Chooses the missing faces and writes the "faceapp" block into the game's modules/fl26world.txt (once a backup
    /// fl26world.txt.vor-pesm). No fl26regen file: nothing to do. FL27 reads it at its next start.
    pub fn gesichter_schreiben(&mut self) {
        if crate::fl27::testmodus() { return; } // tests never write the real game folder (gesichter_im_text tests the parts)
        let datei = crate::fl27::sider().join(crate::id("modules/fl26world.txt"));
        let _ = self.gesichter_in(&datei);
    }
    pub fn gesichter_in(&mut self, datei: &Path) -> Result<bool, String> {
        let Ok(text) = std::fs::read_to_string(datei) else { return Ok(false) };
        let welt = welt_ids(&self.welt());
        let spiel = crate::fl27::spielordner();
        let mut ohne = newfaces(&text);
        ohne.extend(echte_gesichter(&spiel));
        // only players with a record of their own (the world's newfaces players have none; the modules file may name another world's)
        let app = mit_aussehen(&spiel);
        if !app.is_empty() { ohne.extend(welt.iter().filter(|i| !app.contains(i))); }
        let paare = self.gesichter_waehlen(&welt, &ohne, &baertige(&spiel));
        let neu = block_setzen(&text, &paare);
        if neu == text { return Ok(false); }
        let sicherung = datei.with_extension(crate::id("txt.vor-pesm"));
        if !sicherung.exists() { std::fs::copy(datei, &sicherung).map_err(|e| format!("{}: {e}", sicherung.display()))?; }
        crate::fl27::atomic_write(datei, neu.as_bytes())?;
        Ok(true)
    }
    /// Pairs (new player, face) of every new first-team player; a missing face is chosen now and kept. A young player
    /// (up to [gesicht] bartlos_bis) gets a face without a beard while there is one (`bart`: ids with a beard).
    pub fn gesichter_waehlen(&mut self, welt: &HashSet<u32>, ohne: &HashSet<u32>, bart: &HashSet<u32>) -> Vec<(u32, u32)> {
        if welt.is_empty() { return vec![]; }
        let jung = f("jugendakademie", "gesicht.jung_bis") as u8;
        let bartlos_bis = f("jugendakademie", "gesicht.bartlos_bis") as u8;
        // a face chosen before a source became unfit (real face model, no record) is chosen again
        self.world.gesichter.retain(|_, q| !ohne.contains(q));
        let neue: Vec<usize> = (0..self.world.players.len()).filter(|&i| { let p = &self.world.players[i]; p.team != -2 && p.attrs.len() == 25 && !welt.contains(&p.id) }).collect();
        for &i in &neue {
            let (id, nat, alter) = (self.world.players[i].id, self.world.players[i].nat, self.world.players[i].age);
            if self.world.gesichter.contains_key(&id) { continue; }
            // own little random stream per player: choosing faces never moves the world's dice
            let mut r = Rng((id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let soll = herkunft(nat, &mut r);
            let glatt = alter <= bartlos_bis;
            let quelle = |n: u16, nur_jung: bool| -> Vec<u32> { self.world.players.iter().filter(|q| q.team != -2 && welt.contains(&q.id) && !ohne.contains(&q.id) && (n == 0 || q.nat == n) && (!nur_jung || q.age <= jung) && (!glatt || !bart.contains(&q.id))).map(|q| q.id).collect() };
            let mut v = quelle(soll, true);
            if v.is_empty() { v = quelle(soll, false); }
            if v.is_empty() && soll != nat { v = quelle(nat, true); }
            if v.is_empty() { v = quelle(0, true); }
            if v.is_empty() { continue; }
            v.sort_unstable();
            self.world.gesichter.insert(id, v[(r.next() % v.len() as u64) as usize]);
        }
        let mut paare: Vec<(u32, u32)> = neue.iter().filter_map(|&i| { let id = self.world.players[i].id; self.world.gesichter.get(&id).map(|&q| (id, q)) }).collect();
        paare.sort_unstable();
        // shortcut: past fl26regen's 4096 copies the rest keep the default look; raise it there if a world ever needs more
        paare.truncate(KOPIEN_MAX);
        paare
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn newfaces_und_block() {
        let ids = newfaces("newfaces 10-12 15\r\nnewfaces3d 20\r\nleague 1 x\r\n");
        assert_eq!(ids, [10, 11, 12, 15, 20].into_iter().collect());
        let t = "# fl26world 1\r\nnewfaces 10-12\r\n";
        let a = block_setzen(t, &[(900, 5), (901, 6)]);
        assert!(a.contains("faceapp 900:5 901:6\r\n") && a.starts_with(t));
        // written again: the block is replaced, not doubled; no pairs: the file is as before
        let b = block_setzen(&a, &[(900, 5)]);
        assert_eq!(b.matches(ANFANG).count(), 1);
        assert!(!b.contains("901:6"));
        assert_eq!(block_setzen(&b, &[]), t);
    }
    #[test]
    fn bart_aus_dem_satz() {
        // FacialHairType 5 in record 1 (disk bit 307 = byte 38 bit 3), none in record 2
        let mut d = vec![0u8; 120];
        d[..4].copy_from_slice(&11u32.to_le_bytes()); d[60..64].copy_from_slice(&12u32.to_le_bytes());
        d[38] = 5 << 3;
        assert_eq!(baertige_aus(&d), [11].into_iter().collect());
    }
    #[test]
    fn aussehen_aus_dem_satz() {
        // skin 6 (disk bit 296 = byte 37), hair RGB 33/28/21 (bits 366/372/378), step 7 (bit 460), HairStyle 0 (bald)
        let mut r = vec![0u8; 60];
        r[..4].copy_from_slice(&5u32.to_le_bytes());
        let mut setze = |bit: usize, w: u32, v: u64| { let mut x = u64::from_le_bytes(r[bit / 8..bit / 8 + 8].try_into().unwrap_or([0; 8])); x |= (v & ((1 << w) - 1)) << (bit % 8); r[bit / 8..bit / 8 + 8].copy_from_slice(&x.to_le_bytes()); };
        setze(296, 3, 6); setze(366, 6, 33); setze(372, 6, 28); setze(378, 6, 21); setze(307, 5, 2);
        r[57] |= 7 << 4;
        let a = aussehen_aus(&r)[&5];
        assert_eq!(a, Aussehen { haut: 6, haar: [33, 28, 21], haar_stufe: 7, stil: 0, bart: true });
        assert!(a.glatze());
        // no RGB: the preset of its step; a hair style: not bald
        let mut r2 = r.clone(); r2[45] &= 0b0011_1111; r2[46] = 0; r2[47] = 0; r2[48] = 1;
        let b = aussehen_aus(&r2)[&5];
        assert_eq!((b.haar, b.haar_stufe, b.glatze()), ([33, 28, 21], 7, false));
    }
    #[test]
    fn aussehen_im_echten_spiel() {
        // read only; Haaland fair and blond, Koulibaly very dark, black hair, bald (25.11.)
        if !crate::fl27::spielordner().join(crate::id("download")).is_dir() { return; }
        let h = aussehen_von(133543).unwrap();
        assert_eq!((h.haut, h.haar_stufe, h.haar), (1, 7, [33, 28, 21]));
        let k = aussehen_von(102757).unwrap();
        assert_eq!((k.haut, k.haar_stufe, k.glatze()), (6, 1, true));
        // a new player wears his face's look
        spender_setzen(&[(9_000_777, 133543)].into_iter().collect());
        assert_eq!(aussehen_von(9_000_777), Some(h));
    }
    #[test]
    fn bart_im_echten_spiel() {
        // read only; FL27 24.11.: 26,395 records, 11,751 without a beard
        let spiel = crate::fl27::spielordner();
        if !spiel.join(crate::id("download")).is_dir() { return; }
        let n = baertige(&spiel).len();
        assert!((10_000..20_000).contains(&n), "{n}");
        // Elijah Alker's first face 165090 has a real face model (F4L facepack)
        let e = echte_gesichter(&spiel);
        assert!(e.contains(&165090), "{}", e.len());
        assert_eq!(mit_aussehen(&spiel).len(), 26_395);
        eprintln!("real face models: {}", e.len());
    }
    #[test]
    fn herkunft_meist_eigene_manchmal_andere() {
        let mut r = Rng(7);
        let n = 4000;
        let andere = (0..n).filter(|_| herkunft(210, &mut r) != 210).count() as f32 / n as f32;
        assert!((0.08..0.16).contains(&andere), "{andere}");
        // a country without origins keeps its own
        assert!((0..200).all(|_| herkunft(144, &mut r) == 144));
    }
    #[test]
    fn gesichter_im_text() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let welt: HashSet<u32> = c.world.players.iter().map(|p| p.id).collect();
        let nat = c.world.players[0].nat;
        let herkunft_liste = wert("jugendakademie", "gesicht.herkunft").get(nat.to_string()).is_some();
        // a new player of the first team, one in the academy, one retired
        let mut p = c.world.players[0].clone();
        p.id = 9_000_001; p.youth = 0; p.attrs = vec![60.0; 25]; // only players with abilities go into Player.bin
        let mut jugend = p.clone(); jugend.id = 9_000_002; jugend.youth = 19;
        let mut weg = p.clone(); weg.id = 9_000_003; weg.team = -2;
        c.world.players.extend([p, jugend, weg]);
        let ohne: HashSet<u32> = [c.world.players[1].id].into_iter().collect();
        let paare = c.gesichter_waehlen(&welt, &ohne, &HashSet::new());
        assert_eq!(paare.iter().map(|x| x.0).collect::<Vec<_>>(), vec![9_000_001, 9_000_002], "first team and academy, not the retired one");
        let (to, from) = paare[0];
        assert_eq!(to, 9_000_001);
        assert!(welt.contains(&from) && !ohne.contains(&from));
        let fnat = c.world.players.iter().find(|q| q.id == from).unwrap().nat;
        assert!(fnat == nat || herkunft_liste, "face of the same nationality ({fnat} vs {nat})");
        // the face is kept, also when the academy player is promoted
        let i = c.world.players.iter().position(|p| p.id == 9_000_002).unwrap();
        c.world.players[i].youth = 0;
        let paare2 = c.gesichter_waehlen(&welt, &ohne, &HashSet::new());
        assert_eq!(paare2, paare);
        // a young new player never gets a bearded face while a beardless one is there
        let mut bart: HashSet<u32> = welt.clone();
        let glatt = c.world.players.iter().find(|q| welt.contains(&q.id) && q.nat == nat && q.age <= 24 && !ohne.contains(&q.id)).map(|q| q.id).unwrap();
        bart.remove(&glatt);
        let mut p = c.world.players[0].clone(); p.id = 9_000_004; p.youth = 19; p.age = 17; p.attrs = vec![60.0; 25];
        c.world.players.push(p);
        let paare3 = c.gesichter_waehlen(&welt, &ohne, &bart);
        let neu = paare3.iter().find(|x| x.0 == 9_000_004).unwrap().1;
        assert!(!bart.contains(&neu) || !herkunft_liste, "beardless face for a 17-year-old");
        // the block for fl26world.txt names the new ids
        let b = block_setzen("x\n", &paare3);
        for id in [9_000_001, 9_000_002, 9_000_004] { assert!(b.contains(&format!(" {id}:")), "{id}"); }
    }
}
