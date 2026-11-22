//! The game's CPK archives (Data/*.cpk, download/*.cpk), read only. Table of contents and CRILAYLA unpacking come from
//! the Coaching Zone (`pesm_coaching_zone::cpk`, Punkt 4); here only what it doesn't do: one entry read by its place
//! (so the picture index keeps no open archives) and the 16-byte WESYS header removed. The archive is never written.
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// A file from an archive by its entry (offset, size, unpacked size).
pub fn lesen(pfad: &Path, (off, gr, voll): (u64, u64, u64)) -> Result<Vec<u8>, String> {
    if gr > 64 << 20 { return Err(crate::inhalt::t("ui.cpk.eintrag_zu_gross").into()); }
    let mut f = std::fs::File::open(pfad).map_err(|e| e.to_string())?;
    let mut roh = vec![0u8; gr as usize];
    f.seek(SeekFrom::Start(off)).map_err(|e| e.to_string())?;
    f.read_exact(&mut roh).map_err(|e| e.to_string())?;
    if voll > gr { roh = pesm_coaching_zone::cpk::crilayla(&roh).map_err(|e| e.to_string())?; }
    if roh.len() > 16 && &roh[3..8] == b"WESYS" { return crate::fl27::unpack(&roh).map(|x| x.1); }
    Ok(roh)
}

#[cfg(test)]
mod tests {
    #[test]
    fn archive_des_spiels_lesen() {
        let s = crate::fl27::spielordner();
        if !s.join("Data").is_dir() { return; }
        // the player photos of FL27 are .dds pictures inside the Data archives (dt14_all.cpk holds about 26 000)
        let sider = s.join("SiderAddons");
        let b = crate::fl27::bilder_index(&crate::fl27::welt_ordner(&sider), &sider);
        assert!(b.spieler.len() > 20_000, "{} photos", b.spieler.len());
        assert!(b.verein.len() > 1_000, "{} crests", b.verein.len());
        let (_, q) = b.spieler.iter().find(|(_, q)| matches!(q, crate::fl27::Quelle::Cpk(..))).unwrap();
        let bytes = q.lesen().expect("read from the archive");
        assert_eq!(&bytes[0..4], b"DDS ");
        assert!(image::load_from_memory(&bytes).is_ok(), "the DDS reader decodes it");
    }
}
