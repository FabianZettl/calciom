//! Real data lives in a pack, not in the program (Fabian 26.11.: a store release ships without it, the pack's source is
//! shared on Discord): club leadership, coaches, competition and cup names, partner clubs, journalists. Without the pack
//! the program uses its neutral built-in files. Pack folder: <data folder>/pakete/echte-daten, else the one next to the
//! program (assets/pakete/echte-daten). CALCIOM_OHNE_ECHTE_DATEN switches it off for a test of the store release.
use std::path::PathBuf;

pub const ECHT: &str = crate::id("echte-daten");

/// Where the real-data pack is, if it is there and not switched off.
pub fn echt_ordner() -> Option<PathBuf> {
    if std::env::var(crate::id("CALCIOM_OHNE_ECHTE_DATEN")).is_ok() { return None; }
    let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default();
    [crate::fl27::datenordner().join(crate::id("pakete")).join(ECHT), PathBuf::from(crate::id("assets/pakete")).join(ECHT), exe.join(crate::id("../../assets/pakete")).join(ECHT), PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/pakete")).join(ECHT)]
        .into_iter().find(|p| p.join(crate::id("calciom-paket.toml")).is_file())
}
/// A file of the real-data pack (path inside the pack), or None: then the neutral built-in one counts.
pub fn echt(rel: &str) -> Option<String> {
    let d = echt_ordner()?;
    // shortcut: no checksums yet, the pack is trusted like the program; the mod store adds them (docs/intern/konzepte/MODSTORE.md)
    if rel.contains("..") { return None; }
    std::fs::read_to_string(d.join(rel)).ok()
}
/// The pack's file, else the neutral text compiled into the program.
pub fn oder(rel: &str, neutral: &'static str) -> std::borrow::Cow<'static, str> { echt(rel).map(std::borrow::Cow::Owned).unwrap_or(std::borrow::Cow::Borrowed(neutral)) }

#[cfg(test)]
mod tests {
    /// The neutral built-in files read without the pack: nothing real in them, and the code that reads them gets
    /// valid, empty data (Fabian 26.11.: a release without real data must run).
    #[test]
    fn neutral_ohne_paket() {
        let t: serde_json::Value = serde_json::from_str(include_str!("../assets/inhalt/pesm/core/config/scouting/trainerbank.json")).unwrap();
        assert!(t["trainer"].as_array().unwrap().is_empty());
        let v: toml::Value = toml::from_str(include_str!("../assets/vereinsfuehrung.toml")).unwrap();
        assert!(v["clubs"].as_table().unwrap().is_empty());
        let n: toml::Value = toml::from_str(include_str!("../assets/regeln/nationaltrainer.toml")).unwrap();
        assert!(n["trainer"].as_array().unwrap().iter().all(|x| x["name"].as_str() == Some("")) && n.get("pes_m").is_some());
        let p: toml::Value = toml::from_str(include_str!("../assets/regeln/pokalnamen.toml")).unwrap();
        assert!(p["pokal"].as_table().unwrap().is_empty() && p.get("gala_tag").is_some());
        assert!(include_str!("../assets/wettbewerbsnamen.json").trim() == "{}");
        // and the pack is there in this repository, with the real names
        assert!(super::echt(crate::id("vereinsfuehrung.toml")).is_some_and(|t| t.len() > 1000));
    }
}
