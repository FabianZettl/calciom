//! PESM-LivePatch (Übergabe 23.11.2026, ~/Projects/pesm/pes-management/uebergabe/pesm-livepatch, ported from livepatch.py):
//! changes of Player.bin / PlayerAssignment / Tactics reach the RUNNING game without a restart. PES M writes small patch
//! files next to the game; the Sider module pesm-livepatch.lua writes them into the game's memory when the person presses
//! F7 in the Sider overlay. Live: the 25 attributes, squad order (line-up), shirt numbers, both formations of a team. Not
//! live (restart): skills, styles, positions, names, new or moved players.
//!
//! Folder <Sider>/content/pesm-livepatch/:
//!   players.txt  "COUNT n" + per field "pid byte_offset bit_shift 7 new" (7-bit attribute fields of the 380-byte in-game block)
//!   teams.txt    "LOC rank hex" (a team NOT patched, its first formation: the module finds the team array with it), then per
//!                team "T rank pid …" (squad in line-up order), "N rank number …", "F rank 1|2 hex" (3 sets × (11 codes + 11 × depth, width)).
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const MAX_ZEILEN: usize = 400_000;

/// Attribute (index as pesdb::ATTRS) → (byte offset, bit shift) of its 7-bit field in the 380-byte block.
pub const F7: [(u32, u32); 25] = [(0x03, 0), (0x06, 5), (0x05, 6), (0x08, 0), (0x09, 6), (0x0A, 5), (0x08, 7), (0x0C, 0), (0x0E, 5), (0x10, 0), (0x14, 7), (0x18, 7), (0x18, 0),
    (0x19, 6), (0x15, 6), (0x16, 5), (0x1A, 5), (0x04, 0), (0x0C, 7), (0x0D, 6), (0x04, 7), (0x10, 7), (0x11, 6), (0x12, 5), (0x14, 0)];

/// <Sider>/content/pesm-livepatch for a database folder below <Sider>/livecpk/…, else None.
pub fn live_dir(pesdb: &Path) -> Option<PathBuf> {
    let mut p = pesdb;
    while p.file_name().map_or(true, |n| n != "livecpk") { p = p.parent()?; }
    Some(p.parent()?.join(crate::id("content")).join(crate::id("pesm-livepatch")))
}

fn zeilen(p: &Path) -> Vec<String> { std::fs::read_to_string(p).map(|t| t.lines().map(str::to_string).collect()).unwrap_or_default() }
fn speichern(p: &Path, z: &[String]) -> Result<(), String> {
    std::fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    crate::fl27::atomic_write(p, (z.join(crate::id("\n")) + crate::id("\n")).as_bytes())
}

/// Changed attributes {pid: [(attribute index, new value)]} into players.txt; `anzahl` = players in Player.bin.
/// Some(lines) written, None nothing to do, Some(0) too much (a restart is needed).
pub fn add_players(pesdb: &Path, aenderungen: &[(u32, Vec<(usize, u8)>)], anzahl: usize) -> Result<Option<usize>, String> {
    let Some(d) = live_dir(pesdb) else { return Ok(None) };
    let datei = d.join("players.txt");
    let mut behalten: Vec<((String, String, String), String)> = vec![];
    for z in zeilen(&datei) {
        let w: Vec<&str> = z.split_whitespace().collect();
        if w.len() == 5 { behalten.push(((w[0].into(), w[1].into(), w[2].into()), z.clone())); }
    }
    let mut neu = 0;
    for (pid, att) in aenderungen {
        for &(i, v) in att {
            let Some(&(off, sh)) = F7.get(i) else { continue };
            let k = (pid.to_string(), off.to_string(), sh.to_string());
            let z = format!("{pid} {off} {sh} 7 {v}");
            match behalten.iter_mut().find(|x| x.0 == k) { Some(x) => x.1 = z, None => behalten.push((k, z)) }
            neu += 1;
        }
    }
    if neu == 0 { return Ok(None); }
    if behalten.len() > MAX_ZEILEN { return Ok(Some(0)); }
    let mut out = vec![format!("COUNT {anzahl}")];
    out.extend(behalten.into_iter().map(|x| x.1));
    speichern(&datei, &out)?;
    Ok(Some(out.len() - 1))
}

fn u32le(b: &[u8], o: usize) -> u32 { u32::from_le_bytes(b[o..o + 4].try_into().unwrap()) }

/// rank of each team (order of Team.bin), the tactics keys per team, and the formation sets per key.
#[allow(clippy::type_complexity)]
fn team_tabellen(pesdb: &Path) -> Result<(HashMap<u32, usize>, HashMap<u32, Vec<u32>>, HashMap<u32, HashMap<u16, HashMap<u16, (u8, u8, u8)>>>), String> {
    let tm = crate::fl27::pesdb_datei(pesdb, "Team.bin")?;
    let rang: HashMap<u32, usize> = (0..tm.len() / 1532).map(|i| (u32le(&tm, i * 1532 + 8), i)).collect();
    let t = crate::fl27::pesdb_datei(pesdb, "Tactics.bin")?;
    let f = crate::fl27::pesdb_datei(pesdb, "TacticsFormation.bin")?;
    let mut kens: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut o = 0;
    while o + 12 <= t.len() { kens.entry(u32le(&t, o + 4)).or_default().push(u32le(&t, o)); o += 12; }
    let mut sets: HashMap<u32, HashMap<u16, HashMap<u16, (u8, u8, u8)>>> = HashMap::new();
    let mut o = 0;
    while o + 12 <= f.len() {
        let (k, c, d, q, pl) = (u32le(&f, o), u32le(&f, o + 4), f[o + 8], f[o + 9], u16::from_le_bytes([f[o + 10], f[o + 11]]));
        sets.entry(k).or_default().entry(pl >> 4).or_default().insert(pl & 15, (c as u8, d, q));
        o += 12;
    }
    Ok((rang, kens, sets))
}
fn block(sets: &HashMap<u32, HashMap<u16, HashMap<u16, (u8, u8, u8)>>>, k: u32) -> Option<Vec<u8>> {
    let s = sets.get(&k)?;
    if (0..3u16).any(|i| s.get(&i).map_or(true, |v| v.len() < 11)) { return None; }
    let mut out = vec![];
    for i in 0..3u16 {
        let v = &s[&i];
        for j in 0..11u16 { out.push(v[&j].0); }
        for j in 0..11u16 { out.push(v[&j].1); out.push(v[&j].2); }
    }
    Some(out)
}
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

/// Squad order, shirt numbers and formations of team `tid` as they are in the files now into teams.txt (a newer entry
/// replaces the team's older one). false when it cannot be patched (unknown team, no locator).
pub fn add_team(pesdb: &Path, tid: u32) -> Result<bool, String> {
    let Some(d) = live_dir(pesdb) else { return Ok(false) };
    let (rang, kens, sets) = team_tabellen(pesdb)?;
    let Some(&r) = rang.get(&tid) else { return Ok(false) };
    let pa = crate::fl27::pesdb_datei(pesdb, "PlayerAssignment.bin")?;
    let mut sq: Vec<(u32, u32, u32)> = crate::fl27::asg_parse(&pa).into_iter().filter(|a| a.team == tid).map(|a| ((a.k >> 10) & 63, a.pid, (a.k & 127) + 1)).collect();
    sq.sort();
    sq.truncate(40);
    let mut mein = vec![format!("T {r} {}", sq.iter().map(|x| x.1.to_string()).collect::<Vec<_>>().join(" ")), format!("N {r} {}", sq.iter().map(|x| x.2.to_string()).collect::<Vec<_>>().join(" "))];
    for (i, &k) in kens.get(&tid).map(|v| v.as_slice()).unwrap_or(&[]).iter().take(2).enumerate() {
        if let Some(b) = block(&sets, k) { mein.push(format!("F {r} {} {}", i + 1, hex(&b))); }
    }
    let datei = d.join("teams.txt");
    let alt: Vec<String> = zeilen(&datei).into_iter().filter(|z| matches!(z.chars().next(), Some('T' | 'N' | 'F')) && z.split_whitespace().nth(1) != Some(&r.to_string())).collect();
    let mut raenge: std::collections::HashSet<usize> = alt.iter().filter_map(|z| z.split_whitespace().nth(1)?.parse().ok()).collect();
    raenge.insert(r);
    // the locator: a team not patched whose first formation is unique among all formations
    let mut anzahl: HashMap<Vec<u8>, usize> = HashMap::new();
    for ks in kens.values() { for &k in ks { if let Some(b) = block(&sets, k) { *anzahl.entry(b).or_default() += 1; } } }
    let mut teams: Vec<&u32> = kens.keys().collect();
    teams.sort();
    let loc = teams.into_iter().find_map(|t2| {
        let rr = *rang.get(t2)?;
        if raenge.contains(&rr) { return None; }
        let b = block(&sets, kens[t2][0])?;
        (anzahl[&b] == 1).then(|| format!("LOC {rr} {}", hex(&b)))
    });
    let Some(loc) = loc else { return Ok(false) };
    let mut out = vec![loc];
    out.extend(alt);
    out.extend(mein);
    speichern(&datei, &out)?;
    Ok(true)
}

/// Puts the module into <Sider>/modules and its line into sider.ini (once; the ini is backed up by the caller's routine).
pub fn modul_einrichten(sider: &Path, assets: &Path) -> Result<bool, String> {
    let quelle = assets.join(crate::id("livepatch")).join("pesm-livepatch.lua");
    let Ok(lua) = std::fs::read(&quelle) else { return Ok(false) };
    let ziel = sider.join(crate::id("modules")).join("pesm-livepatch.lua");
    if std::fs::read(&ziel).ok().as_deref() != Some(&lua[..]) { crate::fl27::atomic_write(&ziel, &lua)?; }
    Ok(true)
}
/// The sider.ini text with the module line (after the first lua.module line), unchanged when it is there.
pub fn ini_mit_modul(t: &str) -> String {
    if t.lines().any(|z| z.contains("pesm-livepatch.lua") && !z.trim_start().starts_with(';')) { return t.to_string(); }
    let mut out = vec![];
    let mut drin = false;
    for z in t.lines() {
        out.push(z.to_string());
        if !drin && z.trim_start().starts_with("lua.module") {
            let einzug: String = z.chars().take_while(|c| c.is_whitespace()).collect();
            out.push(format!("{einzug}lua.module = \"pesm-livepatch.lua\""));
            drin = true;
        }
    }
    if !drin { out.push("lua.module = \"pesm-livepatch.lua\"".into()); } // roh
    out.join(if t.contains(crate::id("\r\n")) { crate::id("\r\n") } else { crate::id("\n") }) + if t.ends_with('\n') { if t.contains(crate::id("\r\n")) { crate::id("\r\n") } else { crate::id("\n") } } else { "" }
}

/// Marker for "the running game cannot take this in": new or moved players since the game started. PES M deletes it
/// when it starts FL27 itself (the game then loads the files as they are).
/// shortcut: a game started outside PES M keeps an old marker (one restart too many), compare with the process start time if that bothers.
pub fn neustart_merken(pesdb: &Path) -> Result<(), String> {
    let Some(d) = live_dir(pesdb) else { return Ok(()) };
    speichern(&d.join("neustart.txt"), &[crate::id("neu").into()])
}
pub fn neustart_loeschen(pesdb: &Path) { if let Some(d) = live_dir(pesdb) { let _ = std::fs::remove_file(d.join("neustart.txt")); } }
pub fn neustart_noetig(pesdb: &Path) -> bool { live_dir(pesdb).map_or(false, |d| d.join("neustart.txt").exists()) }

/// Can the running game take the career's patches: the module is on in sider.ini and the career is the first active
/// PES M database (else the game runs another career or world and the patches would land in the wrong database).
pub fn ini_passt(t: &str, cpk: &str) -> bool {
    let aktiv = |z: &&str| { let z = z.trim_start(); !z.starts_with(';') && !z.starts_with('#') };
    let modul = t.lines().filter(aktiv).any(|z| z.trim_start().starts_with("lua.module") && z.contains("pesm-livepatch.lua"));
    let erste = t.lines().filter(aktiv).filter_map(|z| z.trim_start().strip_prefix("cpk.root")?.split(crate::id("\\livecpk\\")).nth(1).map(|r| r.trim_end().trim_end_matches('"').to_string()))
        .find(|n| n.starts_with(crate::id("PESM")) && n != crate::fl27::DIRECT_ROOT);
    modul && erste.as_deref() == Some(cpk)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wesys(p: &Path, d: &[u8]) {
        use std::io::Write;
        let mut e = flate2::write::ZlibEncoder::new(vec![], flate2::Compression::default());
        e.write_all(d).unwrap();
        let z = e.finish().unwrap();
        let mut out = b"\x00\x00\x01WESYS".to_vec();
        out.extend((z.len() as u32).to_le_bytes());
        out.extend((d.len() as u32).to_le_bytes());
        out.extend(z);
        std::fs::write(p, out).unwrap();
    }
    #[test]
    fn wie_livepatch_py() {
        let root = std::env::temp_dir().join(format!("pesm-livepatch-{}", std::process::id()));
        let db = root.join("SiderAddons/livecpk/x/common/etc/pesdb");
        std::fs::create_dir_all(&db).unwrap();
        let tids = [10u32, 20, 30, 40];
        wesys(&db.join("Team.bin"), &tids.iter().flat_map(|&t| { let mut v = vec![0u8; 8]; v.extend(t.to_le_bytes()); v.extend(vec![0u8; 1532 - 12]); v }).collect::<Vec<_>>());
        let mut pa = vec![];
        for &t in &tids { for slot in 0..14u32 { for x in [0, t * 100 + slot, t, (slot << 10) | (slot + 5)] { pa.extend(x.to_le_bytes()); } } }
        wesys(&db.join("PlayerAssignment.bin"), &pa);
        let (mut tac, mut fm) = (vec![], vec![]);
        for (n, &t) in tids.iter().enumerate() {
            for j in 0..2u32 {
                let k = 1 + n as u32 * 2 + j;
                for x in [k, t, 0] { tac.extend(x.to_le_bytes()); }
                for s in 0..3u16 { for i in 0..11u16 {
                    fm.extend(k.to_le_bytes()); fm.extend((if i == 0 { 0u32 } else { 1 }).to_le_bytes());
                    fm.push((3 + i as u32 + k) as u8); fm.push(50 + s as u8); fm.extend((s * 16 + i).to_le_bytes());
                } }
            }
        }
        wesys(&db.join("Tactics.bin"), &tac);
        wesys(&db.join("TacticsFormation.bin"), &fm);
        assert_eq!(live_dir(&db), Some(root.join("SiderAddons/content/pesm-livepatch")));
        assert_eq!(live_dir(Path::new("/nowhere/pesdb")), None);
        assert!(add_team(&db, 20).unwrap());
        let teams = std::fs::read_to_string(live_dir(&db).unwrap().join("teams.txt")).unwrap();
        let z: Vec<&str> = teams.lines().collect();
        assert!(z[0].starts_with("LOC ") && !z[0].starts_with("LOC 1 "));
        let t: Vec<&str> = z.iter().find(|l| l.starts_with("T 1 ")).unwrap().split_whitespace().collect();
        assert_eq!(&t[2..4], &["2000", "2001"]);
        assert_eq!(t.len(), 16);
        let n: Vec<&str> = z.iter().find(|l| l.starts_with("N 1 ")).unwrap().split_whitespace().collect();
        assert_eq!(&n[2..4], &["6", "7"]);
        assert!(z.iter().filter(|l| l.starts_with("F ")).all(|l| l.split_whitespace().nth(3).unwrap().len() == 198));
        assert!(add_team(&db, 20).unwrap() && add_team(&db, 30).unwrap());
        let teams = std::fs::read_to_string(live_dir(&db).unwrap().join("teams.txt")).unwrap();
        assert_eq!(teams.lines().filter(|l| l.starts_with("T ")).count(), 2);
        let loc: usize = teams.lines().next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
        assert!(loc != 1 && loc != 2);
        assert_eq!(add_players(&db, &[(7, vec![(10, 80), (16, 70)])], 1000).unwrap(), Some(2));
        assert_eq!(add_players(&db, &[(7, vec![(10, 81)]), (8, vec![(13, 60)])], 1000).unwrap(), Some(3));
        let p = std::fs::read_to_string(live_dir(&db).unwrap().join("players.txt")).unwrap();
        assert!(p.starts_with("COUNT 1000") && p.contains("7 20 7 7 81") && !p.contains("7 20 7 7 80"));
        assert_eq!(add_players(&db, &[], 1000).unwrap(), None);
        assert!(ini_mit_modul("[x]\n\tlua.module = \"a.lua\"\n").contains("\tlua.module = \"pesm-livepatch.lua\""));
        let ini = "\tcpk.root = \".\\livecpk\\PESM_DirectMatch\"\n\t;cpk.root = \".\\livecpk\\PESMArsenal01\"\n\tcpk.root = \".\\livecpk\\PESMChelseaFC01\"\n\tlua.module = \"pesm-livepatch.lua\"\n";
        assert!(ini_passt(ini, "PESMChelseaFC01") && !ini_passt(ini, "PESMArsenal01") && !ini_passt(&ini.replace("\tlua", "\t;lua"), "PESMChelseaFC01"));
        assert!(!neustart_noetig(&db));
        neustart_merken(&db).unwrap();
        assert!(neustart_noetig(&db));
        neustart_loeschen(&db);
        assert!(!neustart_noetig(&db));
        let _ = std::fs::remove_dir_all(&root);
    }
}
