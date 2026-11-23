//! Probe tool for the in-game test of docs/FORSCHUNG-TAKTIK.md: show and change Tactics.bin / TacticsFormation.bin of one
//! team in a pesdb folder (a career copy, never the original world). Every change makes a backup next to the file first.
//!   pesm --taktik-zeigen <pesdb> <team-id>
//!   pesm --taktik-setzen <pesdb> <team-id> <satz 0|1> <bit> <breite> <wert>
//!   pesm --formation-kopieren <pesdb> <team-id> <von-team-id>
use crate::fl27::{pack, unpack};
use std::path::Path;

fn lesen(p: &Path) -> Result<([u8; 8], Vec<u8>), String> { unpack(&std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?) }
fn schreiben(p: &Path, kopf: [u8; 8], d: &[u8]) -> Result<(), String> {
    // the backup goes next to the pesdb folder, not into it (everything in pesdb is copied into the game at match start)
    let ordner = p.parent().and_then(|x| x.parent()).unwrap_or(Path::new(".")).join(crate::id("taktik-sicherungen"));
    std::fs::create_dir_all(&ordner).map_err(|e| e.to_string())?;
    let name = format!("{}.{}", p.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_default(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|x| x.as_secs()).unwrap_or(0));
    std::fs::copy(p, ordner.join(name)).map_err(|e| e.to_string())?;
    crate::fl27::atomic_write(p, &pack(kopf, d))
}
fn u32_at(d: &[u8], o: usize) -> u32 { u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]) }
/// The two Tactics.bin records of a team: (offset, id, packed).
fn saetze(t: &[u8], team: u32) -> Vec<(usize, u32, u32)> { (0..t.len() / 12).map(|i| i * 12).filter(|&o| u32_at(t, o + 4) == team).map(|o| (o, u32_at(t, o), u32_at(t, o + 8))).collect() }

pub fn zeigen(pesdb: &Path, team: u32) -> Result<String, String> {
    let (_, t) = lesen(&pesdb.join("Tactics.bin"))?;
    let (_, f) = lesen(&pesdb.join("TacticsFormation.bin"))?;
    let mut out = String::new();
    for (i, (_, id, v)) in saetze(&t, team).into_iter().enumerate() {
        out += &format!("Satz {i}: Kennung {id}, gepackt {v:#010x}, Halbbytes 0/1/2/3 = {}/{}/{}/{}, Bits 24–31 = {:08b}\n", v & 15, (v >> 4) & 15, (v >> 8) & 15, (v >> 12) & 15, v >> 24); // roh
        for o in (0..f.len() / 12).map(|k| k * 12).filter(|&o| u32_at(&f, o) == id) {
            let platz = u16::from_le_bytes([f[o + 10], f[o + 11]]);
            if platz & 15 == 0 { out += &format!("  Formation {}: ", platz >> 4); } // roh
            out += &format!("{}({},{}) ", u32_at(&f, o + 4), f[o + 8], f[o + 9]);
            if platz & 15 == 10 { out += crate::id("\n"); }
        }
    }
    if out.is_empty() { return Err(format!("Team {team} nicht in Tactics.bin")); } // roh
    Ok(out)
}

pub fn setzen(pesdb: &Path, team: u32, satz: usize, bit: u32, breite: u32, wert: u32) -> Result<String, String> {
    if breite == 0 || bit + breite > 32 || wert >= (1u64 << breite) as u32 { return Err("Bit, Breite oder Wert ungültig".into()); } // roh
    let p = pesdb.join("Tactics.bin");
    let (kopf, mut t) = lesen(&p)?;
    let s = saetze(&t, team);
    let &(o, _, alt) = s.get(satz).ok_or("Satz nicht vorhanden")?; // roh
    let maske = (((1u64 << breite) - 1) as u32) << bit;
    let neu = (alt & !maske) | (wert << bit);
    t[o + 8..o + 12].copy_from_slice(&neu.to_le_bytes());
    schreiben(&p, kopf, &t)?;
    Ok(format!("Team {team} Satz {satz}: {alt:#010x} → {neu:#010x}")) // roh
}

/// Copies the three formation sets of `von` onto `team` (positions and places; the start eleven stays in PlayerAssignment).
pub fn formation_kopieren(pesdb: &Path, team: u32, von: u32) -> Result<String, String> {
    let (_, t) = lesen(&pesdb.join("Tactics.bin"))?;
    let p = pesdb.join("TacticsFormation.bin");
    let (kopf, mut f) = lesen(&p)?;
    let (Some(&(_, ziel, _)), Some(&(_, quelle, _))) = (saetze(&t, team).first(), saetze(&t, von).first()) else { return Err("Team nicht gefunden".into()) }; // roh
    let q: Vec<(u16, [u8; 6])> = (0..f.len() / 12).map(|k| k * 12).filter(|&o| u32_at(&f, o) == quelle).map(|o| (u16::from_le_bytes([f[o + 10], f[o + 11]]), [f[o + 4], f[o + 5], f[o + 6], f[o + 7], f[o + 8], f[o + 9]])).collect();
    let mut n = 0;
    let orte: Vec<usize> = (0..f.len() / 12).map(|k| k * 12).filter(|&o| u32_at(&f, o) == ziel).collect();
    for o in orte {
        let platz = u16::from_le_bytes([f[o + 10], f[o + 11]]);
        if let Some((_, w)) = q.iter().find(|x| x.0 == platz) { f[o + 4..o + 10].copy_from_slice(w); n += 1; }
    }
    schreiben(&p, kopf, &f)?;
    Ok(format!("{n} Plätze von Team {von} auf Team {team} kopiert")) // roh
}

pub fn kommando(args: &[String]) -> Option<Result<String, String>> {
    let a = |i: usize| args.get(i).cloned().unwrap_or_default();
    let n = |i: usize| a(i).parse::<u32>().unwrap_or(u32::MAX);
    let i = args.iter().position(|x| x.starts_with("--taktik-") || x == "--formation-kopieren")?; // roh
    let p = Path::new(&args[i + 1]).to_path_buf();
    Some(match args[i].as_str() {
        "--taktik-zeigen" => zeigen(&p, n(i + 2)), // roh
        "--taktik-setzen" => setzen(&p, n(i + 2), n(i + 3) as usize, n(i + 4), n(i + 5), n(i + 6)), // roh
        _ => formation_kopieren(&p, n(i + 2), n(i + 3)),
    })
}
