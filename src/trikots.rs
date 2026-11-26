//! Kits from the FL27 kitserver (SiderAddons/modules/common/kits.lua 1.9), read only, for the kit choice before the
//! match (Fabian 26.11.). Nothing of the kits is ever copied into CalcioM or its repo; they are read where they are.
//!
//! content/kits/map.txt: "team id, \"folder\"". The team folder's order.ini lists the kit folders; in a friendly
//! (tournament 65535, which the direct start is) the kitserver loads every [section], skips folders without a
//! config.txt, and kit number n in flw/direktmatch.txt is the n-th loaded entry. Goalkeepers it picks itself.
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct Trikot {
    /// The line of order.ini ("p1", "uefa\\p1-uefa").
    pub name: String,
    /// ShirtColor1, ShirtColor2, ShortsColor, SocksColor as 0xRRGGBB.
    pub farben: [u32; 4],
    /// The kit texture (KitFile=… + .ftex in the kit folder), when it is there.
    pub textur: Option<PathBuf>,
}

/// The kit folder of a team, from map.txt.
fn team_ordner(kits: &Path, team: u32) -> Option<PathBuf> {
    let map = std::fs::read_to_string(kits.join("map.txt")).ok()?;
    let pfad = map.lines().find_map(|z| {
        let (id, p) = z.split('#').next()?.split_once(',')?;
        (id.trim().parse::<u32>().ok()? == team).then(|| p.trim().trim_matches('"').to_string())
    })?;
    teil_pfad(kits, &pfad)
}
/// "a\\b" under `base`, each part found regardless of case (the files come from Windows).
fn teil_pfad(base: &Path, p: &str) -> Option<PathBuf> {
    p.split(['\\', '/']).filter(|s| !s.is_empty()).try_fold(base.to_path_buf(), |b, s| crate::fl27::datei_ci(&b, s))
}
fn farbe(cfg: &str, key: &str) -> Option<u32> {
    cfg.lines().find_map(|z| {
        let (k, v) = z.split(';').next()?.split_once('=')?;
        (k.trim() == key).then(|| u32::from_str_radix(v.trim().trim_start_matches('#'), 16).ok())?
    })
}

/// The kits of an FL27 team in the kitserver's order; empty when the kitserver has none for it.
pub fn trikots(sider: &Path, team: u32) -> Vec<Trikot> {
    let kits = sider.join(crate::id("content")).join(crate::id("kits"));
    let Some(ordner) = team_ordner(&kits, team) else { return vec![] };
    let Some(order) = crate::fl27::datei_ci(&ordner, "order.ini").and_then(|p| std::fs::read(p).ok()) else { return vec![] };
    String::from_utf8_lossy(&order).trim_start_matches('\u{feff}').lines().filter_map(|z| {
        let z = z.split([';', '#']).next()?.split_whitespace().next()?;
        if z.starts_with('[') { return None; }
        let d = teil_pfad(&ordner, z)?;
        let cfg = String::from_utf8_lossy(&std::fs::read(crate::fl27::datei_ci(&d, "config.txt")?).ok()?).into_owned();
        let f = |k, alt| farbe(&cfg, k).unwrap_or(alt);
        let s1 = f("ShirtColor1", 0x808080);
        let textur = cfg.lines().find_map(|l| l.split(';').next()?.trim().strip_prefix(crate::id("KitFile=")).map(|v| v.trim().to_string()))
            .filter(|v| !v.is_empty()).and_then(|v| crate::fl27::datei_ci(&d, &format!("{v}.ftex")));
        Some(Trikot { name: z.to_string(), farben: [s1, f("ShirtColor2", s1), f("ShortsColor", s1), f("SocksColor", s1)], textur })
    }).collect()
}

/// The same, read once per team (map.txt has thousands of lines) from the chosen game folder.
pub fn von_team(team: u32) -> Vec<Trikot> {
    static CACHE: std::sync::Mutex<Option<std::collections::HashMap<u32, Vec<Trikot>>>> = std::sync::Mutex::new(None);
    CACHE.lock().unwrap().get_or_insert_with(Default::default).entry(team).or_insert_with(|| trikots(&crate::fl27::sider(), team)).clone()
}
/// A Fox Engine texture (.ftex) as RGBA, the largest mip level not wider than `max`. Formats seen in the kits:
/// 0 = BGRA8, 3 = BC2, 4 = BC3, 11 = BC7 (checked by eye 26.11. on Arsenal, Liverpool, Dortmund).
/// Mip table at 0x40, 16 bytes each: offset, size, packed size, index, file, chunk count; each chunk 8 bytes
/// (packed size u16, size u16, offset u32 with the top bit = stored raw), else zlib.
pub fn ftex_rgba(d: &[u8], max: u32) -> Option<(u32, u32, Vec<u8>)> {
    let u16_ = |o: usize| d.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let u32_ = |o: usize| d.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    if d.get(0..4)? != b"FTEX" { return None; }
    let (fmt, w0, h0, mips) = (u16_(8)?, u16_(10)? as u32, u16_(12)? as u32, *d.get(16)? as usize);
    let i = (0..mips.max(1)).find(|&i| (w0 >> i).max(1) <= max || i + 1 == mips.max(1))?;
    let (w, h) = ((w0 >> i).max(1), (h0 >> i).max(1));
    let e = 0x40 + 16 * i;
    let (off, n) = (u32_(e)? as usize, u16_(e + 14)? as usize);
    let mut roh = vec![];
    for c in 0..n {
        let k = off + 8 * c;
        let (gr, o) = (u16_(k)? as usize, u32_(k + 4)?);
        let a = off + (o & 0x7fff_ffff) as usize;
        let teil = d.get(a..a + gr)?;
        if o & 0x8000_0000 != 0 { roh.extend_from_slice(teil); } else { std::io::Read::read_to_end(&mut flate2::read::ZlibDecoder::new(teil), &mut roh).ok()?; }
    }
    let (wu, hu) = (w as usize, h as usize);
    let mut px = vec![0u32; wu * hu];
    let rgba = match fmt {
        0 => roh.get(..wu * hu * 4)?.chunks_exact(4).flat_map(|p| [p[2], p[1], p[0], p[3]]).collect(),
        3 | 4 | 11 => {
            match fmt { 3 => texture2ddecoder::decode_bc2(&roh, wu, hu, &mut px), 4 => texture2ddecoder::decode_bc3(&roh, wu, hu, &mut px), _ => texture2ddecoder::decode_bc7(&roh, wu, hu, &mut px) }.ok()?;
            px.iter().flat_map(|p| { let [b, g, r, a] = p.to_le_bytes(); [r, g, b, a] }).collect()
        }
        _ => return None,
    };
    Some((w, h, rgba))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trikotliste_wie_der_kitserver() {
        let s = std::env::temp_dir().join(format!("calciom-trikots-{}", std::process::id()));
        let k = s.join("content/kits");
        let t = k.join("Liga/Verein");
        for (d, farbe) in [("p1", "#ff0000"), ("P2", "#00ff00"), ("uefa/p1-uefa", "#0000ff")] {
            std::fs::create_dir_all(t.join(d)).unwrap();
            std::fs::write(t.join(d).join("config.txt"), format!("KitFile=k1\nShirtColor1={farbe}   ; Hemd\nShortsColor=#ffffff\n")).unwrap();
        }
        std::fs::write(t.join("p1/k1.ftex"), b"FTEX").unwrap();
        std::fs::write(k.join("map.txt"), "# Karte\n7, \"Liga\\Verein\"\n").unwrap();
        // BOM, CRLF, comments, a folder that is missing, a section
        std::fs::write(t.join("order.ini"), "\u{feff}p1\r\np2 ; zweites\r\np9\r\n\r\n[uefa]\r\nuefa\\p1-uefa\r\n").unwrap();
        let v = trikots(&s, 7);
        assert_eq!(v.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["p1", "p2", "uefa\\p1-uefa"]);
        assert_eq!(v[0].farben, [0xff0000, 0xff0000, 0xffffff, 0xff0000]);
        assert!(v[0].textur.is_some() && v[1].textur.is_none());
        assert!(trikots(&s, 8).is_empty());
        std::fs::remove_dir_all(&s).ok();
    }
    #[test]
    fn ftex_roh_und_gepackt() {
        // 4×4 BGRA8, two mips: the 4×4 level stored raw, the 2×2 level zlib-packed
        let mut d = b"FTEX".to_vec();
        d.extend([0u8; 4]);
        d.extend(0u16.to_le_bytes()); d.extend(4u16.to_le_bytes()); d.extend(4u16.to_le_bytes()); d.extend(1u16.to_le_bytes());
        d.push(2);
        d.resize(0x60, 0);
        let gross: Vec<u8> = (0..16).flat_map(|_| [1u8, 2, 3, 255]).collect();
        let klein: Vec<u8> = (0..4).flat_map(|_| [9u8, 8, 7, 255]).collect();
        let mut z = flate2::write::ZlibEncoder::new(vec![], flate2::Compression::default());
        std::io::Write::write_all(&mut z, &klein).unwrap();
        let z = z.finish().unwrap();
        for (i, (daten, roh)) in [(&gross, true), (&z, false)].into_iter().enumerate() {
            let off = d.len() as u32;
            let e = 0x40 + 16 * i;
            d[e..e + 4].copy_from_slice(&off.to_le_bytes());
            d[e + 14..e + 16].copy_from_slice(&1u16.to_le_bytes());
            d.extend((daten.len() as u16).to_le_bytes()); d.extend(0u16.to_le_bytes());
            d.extend((8u32 | if roh { 0x8000_0000 } else { 0 }).to_le_bytes());
            d.extend(daten.iter());
        }
        assert_eq!(ftex_rgba(&d, 4).map(|x| (x.0, x.2[..4].to_vec())), Some((4, vec![3, 2, 1, 255])));
        assert_eq!(ftex_rgba(&d, 2).map(|x| (x.0, x.2[..4].to_vec())), Some((2, vec![7, 8, 9, 255])));
    }
}
