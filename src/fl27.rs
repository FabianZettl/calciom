// FL27-Anbindung (Modul-Paket "FL27-Anbindung", Stand 2026-11-20): memory reader for the live match statistics,
// pesdb container and PlayerAssignment writing, career LiveCPK, sider.ini, Sider presets and the match start via Lutris.
// Pure std + flate2/sha2/serde_json and the Coaching Zone library (Punkt 4), no crate:: imports: the read-only helper
// binary (src/bin/calciom-speicher.rs) includes it too.
#![allow(dead_code)]
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(unix)]
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use pesm_coaching_zone as cz;

// ---------------------------------------------------------------- environment (§0)
pub const PREFIX: &str = crate::id("/mnt/GAMES/sp-football-life-2026");
pub const WELT_ROOT: &str = "_FL26MyWorld";
pub const DIRECT_ROOT: &str = "CalcioM_DirectMatch";
pub const LUTRIS_ID: u32 = 7;
pub const EXE: &str = "FL_2027.exe";

/// CalcioM's own data folder (log, chosen game folder): Windows %APPDATA%\calciom, else ~/.local/share/calciom;
/// tests and previews use their CALCIOM_KARRIEREN folder.
pub fn datenordner() -> PathBuf {
    if let Ok(d) = std::env::var("CALCIOM_KARRIEREN") { return PathBuf::from(d); }
    match std::env::var("APPDATA") {
        Ok(a) => neu_oder_alt(PathBuf::from(&a).join(crate::id("calciom")), PathBuf::from(a).join(concat!("pe", "sm"))),
        Err(_) => neu_oder_alt(heim().join(".local/share/calciom"), heim().join(concat!(".local/share/pe", "sm-rust"))), // roh
    }
}
fn heim() -> PathBuf { PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())) }
/// The new folder, or the old one of PES M while it is still there and the new one is not (the move at the start
/// failed, e.g. a file of it open on Windows): old careers and settings are read where they are, nothing is copied.
pub fn neu_oder_alt(neu: PathBuf, alt: PathBuf) -> PathBuf { if !neu.exists() && alt.is_dir() { alt } else { neu } }
/// The careers folder (game::save_dir and the memory helper): next to the settings, old or new name as above.
pub fn karrieren_ordner() -> PathBuf {
    if let Ok(d) = std::env::var("CALCIOM_KARRIEREN") { return PathBuf::from(d); }
    neu_oder_alt(heim().join(".local/share/calciom"), heim().join(concat!(".local/share/pe", "sm-rust"))).join(crate::id("karrieren")) // roh
}
static GEWAEHLT: std::sync::Mutex<Option<Option<PathBuf>>> = std::sync::Mutex::new(None);
/// The game folder the person chose (beta 23.11.: the search did not find every install), kept in spielordner.txt.
fn gewaehlt() -> Option<PathBuf> {
    GEWAEHLT.lock().unwrap().get_or_insert_with(|| std::fs::read_to_string(datenordner().join("spielordner.txt")).ok().map(|s| PathBuf::from(s.trim())).filter(|p| p.join("SiderAddons").is_dir())).clone() // roh
}
pub fn spielordner_setzen(p: &Path) {
    let _ = std::fs::create_dir_all(datenordner());
    let _ = std::fs::write(datenordner().join("spielordner.txt"), p.display().to_string()); // roh
    *GEWAEHLT.lock().unwrap() = Some(Some(p.to_path_buf()));
}
/// From any path the person pastes or picks (game folder, SiderAddons, livecpk or a world): the game folder.
pub fn spielordner_aus(p: &Path) -> Option<PathBuf> {
    p.ancestors().find(|a| a.join("SiderAddons").join("livecpk").is_dir()).map(Path::to_path_buf) // roh
}
/// The system's file dialog for a PNG picture in the background (manager photo); None when cancelled.
pub fn bild_dialog() -> std::sync::mpsc::Receiver<Option<PathBuf>> { datei_dialog(true) }
/// The system's file dialog in the background: a PNG picture, or (`png` false) any file such as a Master League save.
pub fn datei_dialog(png: bool) -> std::sync::mpsc::Receiver<Option<PathBuf>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let (win, zen, kde) = if png { ("PNG|*.png", "--file-filter=*.png", "*.png") } else { ("*|*.*", "--file-filter=*", "*") }; // roh
        #[cfg(windows)]
        let aus = std::process::Command::new("powershell").args(["-NoProfile", "-STA", "-Command", &format!("Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.OpenFileDialog; $f.Filter = '{win}'; if ($f.ShowDialog() -eq 'OK') {{ $f.FileName }}")]).output(); // roh
        #[cfg(not(windows))]
        let aus = { let _ = win; std::process::Command::new("zenity").args(["--file-selection", zen]).output().or_else(|_| std::process::Command::new("kdialog").args(["--getopenfilename", ".", kde]).output()) }; // roh
        let _ = tx.send(aus.ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|s| !s.is_empty()).map(PathBuf::from));
    });
    rx
}
/// The system's folder dialog in the background (Windows: PowerShell, Linux: zenity or kdialog); None when cancelled.
pub fn ordner_dialog() -> std::sync::mpsc::Receiver<Option<PathBuf>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        #[cfg(windows)]
        let aus = std::process::Command::new("powershell").args(["-NoProfile", "-STA", "-Command", "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = 'SP Football Life 2027'; if ($f.ShowDialog() -eq 'OK') { $f.SelectedPath }"]).output(); // roh
        #[cfg(not(windows))]
        let aus = std::process::Command::new("zenity").args(["--file-selection", "--directory"]).output().or_else(|_| std::process::Command::new("kdialog").arg("--getexistingdirectory").output()); // roh
        let p = aus.ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|s| !s.is_empty()).map(PathBuf::from);
        let _ = tx.send(p);
    });
    rx
}
/// Game folder G (CALCIOM_FL27_SPIEL overrides it, for tests).
pub fn spielordner() -> PathBuf {
    if let Ok(p) = std::env::var("CALCIOM_FL27_SPIEL") { return PathBuf::from(p); }
    if let Some(p) = gewaehlt() { return p; }
    #[cfg(windows)]
    { static ORT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new(); return ORT.get_or_init(spielordner_suchen).clone(); }
    #[allow(unreachable_code)]
    PathBuf::from(PREFIX).join(crate::inhalt::t("ui.fl27.drive_c_games_sp_football"))
}
/// Windows: the FL27 install next to CalcioM (any folder above it), else on any drive under the usual places.
#[cfg(windows)]
fn spielordner_suchen() -> PathBuf {
    const NAME: &str = "SP Football Life 2027"; // roh
    let passt = |d: &Path| d.join(EXE).is_file();
    let neben = std::env::current_exe().ok().into_iter().flat_map(|e| e.ancestors().skip(1).map(Path::to_path_buf).collect::<Vec<_>>());
    for d in neben { for k in [d.clone(), d.join(NAME), d.join(crate::id("Games")).join(NAME)] { if passt(&k) { return k; } } }
    for lw in b'C'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\", lw as char));
        if !root.is_dir() { continue; }
        for k in [root.join(NAME), root.join(crate::id("Games")).join(NAME), root.join("Program Files (x86)").join(NAME), root.join("Program Files").join(NAME)] { if passt(&k) { return k; } } // roh
        // one level deeper: X:\<any folder>\SP Football Life 2027
        for e in std::fs::read_dir(&root).into_iter().flatten().flatten() { let k = e.path().join(NAME); if passt(&k) { return k; } }
    }
    PathBuf::from("C:\\Games").join(NAME)
}
pub fn sider() -> PathBuf { spielordner().join(crate::id("SiderAddons")) }
pub fn welt_ordner(s: &Path) -> PathBuf { s.join(crate::id("livecpk")).join(WELT_ROOT) }

/// Career CPK name (Übergabe Daten und Datenbank §8.1): "CalcioM" + club name (accents dropped, ASCII letters and digits,
/// at most 30) + the first free number 01..99 in the livecpk folder.
pub fn cpk_name(club: &str, livecpk: &Path) -> String {
    let base: String = club.chars().filter_map(|c| {
        let b = match c { 'ä' | 'á' | 'à' | 'â' | 'ã' | 'å' => 'a', 'Ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Å' => 'A', 'ö' | 'ó' | 'ò' | 'ô' | 'õ' | 'ø' => 'o', 'Ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ø' => 'O',
            'ü' | 'ú' | 'ù' | 'û' => 'u', 'Ü' | 'Ú' | 'Ù' | 'Û' => 'U', 'é' | 'è' | 'ê' | 'ë' => 'e', 'É' | 'È' | 'Ê' | 'Ë' => 'E', 'í' | 'ì' | 'î' | 'ï' | 'ı' => 'i', 'Í' | 'Ì' | 'Î' | 'Ï' | 'İ' => 'I',
            'ñ' => 'n', 'Ñ' => 'N', 'ç' => 'c', 'Ç' => 'C', 'ş' | 'š' => 's', 'Ş' | 'Š' => 'S', 'ğ' => 'g', 'Ğ' => 'G', 'ž' => 'z', 'Ž' => 'Z', 'č' | 'ć' => 'c', 'Č' | 'Ć' => 'C', x => x };
        b.is_ascii_alphanumeric().then_some(b)
    }).take(30).collect();
    (1..=99).map(|n| format!("CalcioM{base}{n:02}")).find(|n| !livecpk.join(n).exists()).unwrap_or_else(|| format!("CalcioM{base}99"))
}

/// Our own career folders: those from before the rename (26.11.2026) start with the old short name, new ones with CalcioM.
pub fn eigener_cpk(n: &str) -> bool { let n = n.to_ascii_lowercase(); n.starts_with(crate::id("pesm")) || n.starts_with(crate::id("calciom")) }
/// Our marker in other mods' Lua files: installs from before the rename carry the old name.
pub fn von_uns(t: &str) -> bool { t.contains(concat!("PES", " M")) || t.contains("CalcioM") }

/// First start after the rename to CalcioM (26.11.2026): the old data folders move to the new names, so careers,
/// log, chosen game folder and picture cache stay. On Linux the old name stays as a link to the new folder (an older
/// build still finds everything). Nothing happens when the new folder already exists or a test folder is set.
pub fn umzug() {
    if std::env::var(crate::id("CALCIOM_KARRIEREN")).is_ok() { return; }
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()));
    let mut paare = vec![(home.join(concat!(".local/share/pesm", "-rust")), home.join(".local/share/calciom"))];
    if let Ok(a) = std::env::var("APPDATA") { paare.push((PathBuf::from(&a).join(concat!("pe", "sm")), PathBuf::from(a).join("calciom"))); }
    let cache = std::env::var("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|_| home.join(crate::id(".cache")));
    paare.push((cache.join(concat!("pe", "sm")), cache.join("calciom")));
    for (alt, neu) in paare { umziehen(&alt, &neu); }
    let d = datenordner();
    umziehen(&d.join(concat!("pesm", ".log")), &d.join("calciom.log"));
}
fn umziehen(alt: &Path, neu: &Path) {
    if neu.exists() || !alt.exists() || alt.is_symlink() { return; }
    if std::fs::rename(alt, neu).is_ok() && neu.is_dir() {
        #[cfg(unix)] let _ = std::os::unix::fs::symlink(neu, alt);
    }
}

/// The old direct start in the game folder (before the rename): its folder, module and flow files go, its sider.ini
/// lines are switched off with ";". The caller sets up the new names right after, so nothing runs twice.
pub fn alte_namen_weg(s: &Path) -> Result<(), String> {
    let (root, modul) = (concat!("PE", "SM_DirectMatch"), concat!("pesm", "_direktmatch.lua"));
    if std::fs::read_to_string(s.join("sider.ini")).is_ok_and(|t| alte_zeilen_aus(&t, root, modul) != t) {
        sider_ini_anwenden(s, s, |t| Ok(alte_zeilen_aus(t, root, modul)))?;
    }
    let _ = std::fs::remove_dir_all(s.join(crate::id("livecpk")).join(root));
    let _ = std::fs::remove_file(s.join(crate::id("modules")).join(modul));
    let _ = std::fs::remove_dir_all(s.join(crate::id("content")).join(concat!("pesm", "-direktmatch")));
    Ok(())
}
fn alte_zeilen_aus(t: &str, root: &str, modul: &str) -> String {
    t.split_inclusive('\n').map(|l| {
        let r = l.trim_start();
        let alt = !r.starts_with(';') && !r.starts_with('#') && (r.contains(&format!("\\livecpk\\{root}\"")) || (r.starts_with("lua.module") && r.contains(&format!("\"{modul}\""))));
        if alt { format!("{};{r}", indent_of(l)) } else { l.to_string() }
    }).collect()
}

/// Tests with the real world: a missing world folder fails the test, unless CALCIOM_OHNE_SPIEL=1 says on purpose that
/// they run without the game (then they are reported as skipped, never silently green).
#[cfg(test)]
pub fn echte_welt(p: &Path) -> bool {
    if p.exists() { return true; }
    if std::env::var("CALCIOM_OHNE_SPIEL").is_ok() { eprintln!("ÜBERSPRUNGEN (CALCIOM_OHNE_SPIEL): {} fehlt", p.display()); return false; }
    panic!("Welt-Ordner fehlt: {}. Ohne das Spiel mit CALCIOM_OHNE_SPIEL=1 testen, dann werden diese Tests als übersprungen gemeldet.", p.display());
}

// ---------------------------------------------------------------- files
pub fn sha256_hex(b: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}
pub fn sha_file(p: &Path) -> Result<String, String> { std::fs::read(p).map(|b| sha256_hex(&b)).map_err(|e| format!("{}: {e}", p.display())) }

/// Atomic write (temp file + rename), then read back and compare.
/// Test mode (cargo test, --screenshot, --self-test, --start-test): never the real FL27 process, never the real game folder.
pub fn testmodus() -> bool { cfg!(test) || std::env::var_os("CALCIOM_TESTMODUS").is_some() }
/// Guard: in test mode, writing or deleting below the real game prefix aborts.
pub fn wache(p: &Path) {
    if testmodus() && p.starts_with(PREFIX) { panic!("Testmodus: CalcioM würde in den echten Spielordner schreiben: {}", p.display()); }
}
pub fn atomic_write(p: &Path, b: &[u8]) -> Result<(), String> {
    wache(p);
    if let Some(d) = p.parent() { std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?; }
    let tmp = p.with_extension(format!("{}.calciom-tmp", p.extension().and_then(|e| e.to_str()).unwrap_or("")));
    std::fs::write(&tmp, b).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, p).map_err(|e| format!("{}: {e}", p.display()))?;
    match std::fs::read(p) { Ok(back) if back == b => Ok(()), _ => Err(format!("{}: nach dem Schreiben nicht gleich", p.display())) }
}

/// Progress of the long career-start steps for the loading bar (beta 23.11.: Windows showed "Not Responding" while
/// ~46 000 Megapack files were copied and hashed in the foreground): phase (0 counting, 1 copying, 2 checking,
/// 3 career database), files done, files in all.
pub static FORT_PHASE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static FORT_FERTIG: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static FORT_GESAMT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
fn fort(phase: usize, gesamt: usize) {
    use std::sync::atomic::Ordering::Relaxed;
    FORT_PHASE.store(phase, Relaxed); FORT_FERTIG.store(0, Relaxed); FORT_GESAMT.store(gesamt, Relaxed);
}
fn dateien_zaehlen(d: &Path) -> usize {
    std::fs::read_dir(d).map(|r| r.flatten().map(|e| match e.file_type() { Ok(t) if t.is_dir() => dateien_zaehlen(&e.path()), Ok(t) if t.is_file() => 1, _ => 0 }).sum()).unwrap_or(0)
}
/// One line into the log file users can send along (Windows %APPDATA%\calciom\calciom.log, else next to the careers).
pub fn log(zeile: &str) {
    let datei = datenordner().join("calciom.log");
    let _ = std::fs::create_dir_all(datei.parent().unwrap_or(Path::new(".")));
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&datei) { let _ = writeln!(f, "{} {zeile}", zeit()); }
}
/// Diagnostics for bug reports (Einstellungen › Spielstand): calciom.log, sider.log, sider.ini and versions in one archive
/// next to the log; the user name in paths is replaced, no save goes in. Returns the archive.
pub fn diagnose_paket() -> Result<PathBuf, String> {
    let d = datenordner();
    let tmp = d.join(format!("calciom-diagnose-{}", zeit()));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let nutzer: Vec<String> = ["USERNAME", "USER"].iter().filter_map(|k| std::env::var(k).ok()).filter(|n| n.len() >= 2).collect();
    let sauber = |t: String| nutzer.iter().fold(t, |t, n| t.replace(n.as_str(), "<user>"));
    let g = spielordner();
    let quellen = [(d.join("calciom.log"), "calciom.log"), (sider().join("sider.log"), "sider.log"), (g.join("sider.log"), "sider-spiel.log"), (sider().join("sider.ini"), "sider.ini")]; // roh
    for (q, n) in quellen { if let Ok(b) = std::fs::read(&q) { let _ = std::fs::write(tmp.join(n), sauber(String::from_utf8_lossy(&b).into_owned())); } }
    let info = format!("CalcioM {}\nOS {} {}\nSpielordner {}\nWelten {}\n", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH, g.display(), std::fs::read_dir(sider().join("livecpk")).map(|r| r.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect::<Vec<_>>().join(", ")).unwrap_or_default()); // roh
    std::fs::write(tmp.join("versionen.txt"), sauber(info)).map_err(|e| e.to_string())?; // roh
    #[cfg(windows)]
    let (ziel, st) = { let z = tmp.with_extension("zip"); let st = still(std::process::Command::new("powershell").args(["-NoProfile", "-Command", &format!("Compress-Archive -Path '{}\\*' -DestinationPath '{}' -Force", tmp.display(), z.display())])).status(); (z, st) }; // roh
    #[cfg(not(windows))]
    let (ziel, st) = { let z = tmp.with_extension("tar.gz"); let st = std::process::Command::new("tar").arg("czf").arg(&z).arg("-C").arg(&tmp).arg(".").status(); (z, st) }; // roh
    let _ = std::fs::remove_dir_all(&tmp);
    match st { Ok(s) if s.success() && ziel.exists() => Ok(ziel), _ => Err(format!("{}: Archiv nicht erstellt", ziel.display())) } // roh
}
/// A whole folder copied (career copy), with the progress counters.
pub fn ordner_kopieren(from: &Path, to: &Path) -> Result<usize, String> {
    fort(1, dateien_zaehlen(from));
    let mut files = vec![];
    copy_dir(from, to, &mut files, Path::new(""))?;
    Ok(files.len())
}
fn copy_dir(from: &Path, to: &Path, files: &mut Vec<PathBuf>, rel: &Path) -> Result<(), String> {
    wache(to);
    std::fs::create_dir_all(to.join(rel)).map_err(|e| e.to_string())?;
    for e in std::fs::read_dir(from.join(rel)).map_err(|e| format!("{}: {e}", from.join(rel).display()))? {
        let e = e.map_err(|e| e.to_string())?;
        let r = rel.join(e.file_name());
        let ft = e.file_type().map_err(|e| e.to_string())?;
        if ft.is_dir() { copy_dir(from, to, files, &r)?; } else if ft.is_file() {
            // an unreadable picture (e.g. a damaged file on disk) is left out; everything else must copy
            if let Err(e) = std::fs::copy(from.join(&r), to.join(&r)) {
                let bild = r.components().any(|c| c.as_os_str().eq_ignore_ascii_case("render"));
                if !bild { return Err(format!("{}: {e}", r.display())); }
                let _ = std::fs::remove_file(to.join(&r));
                eprintln!("übersprungen (nicht lesbar): {}: {e}", from.join(&r).display()); // roh
                continue;
            }
            files.push(r);
            FORT_FERTIG.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    Ok(())
}

/// YYYYmmdd_HHMMSS (UTC) for backup names.
fn zeit() -> String {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (d, r) = (t.div_euclid(86400), t.rem_euclid(86400));
    let z = d + 719468; let era = z.div_euclid(146097); let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153; let day = doy - (153 * mp + 2) / 5 + 1; let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}{m:02}{day:02}_{:02}{:02}{:02}", r / 3600, r / 60 % 60, r % 60)
}
fn stamp() -> String {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    format!("{t}")
}

// ---------------------------------------------------------------- pesdb container (§2.1)
/// 16-byte header (3 bytes id, "WESYS", u32 packed length, u32 unpacked length) + zlib data.
pub fn unpack(raw: &[u8]) -> Result<([u8; 8], Vec<u8>), String> {
    if raw.len() < 16 || !raw[3..8].eq_ignore_ascii_case(b"WESYS") { return Err("kein pesdb-Container (WESYS fehlt)".into()); }
    let mut head = [0u8; 8];
    head.copy_from_slice(&raw[..8]);
    let mut out = vec![];
    flate2::read::ZlibDecoder::new(&raw[16..]).read_to_end(&mut out).map_err(|e| format!("zlib: {e}"))?;
    let want = u32::from_le_bytes(raw[12..16].try_into().unwrap()) as usize;
    if want != out.len() { return Err(format!("Länge {} statt {}", out.len(), want)); }
    Ok((head, out))
}
pub fn pack(head: [u8; 8], data: &[u8]) -> Vec<u8> {
    let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(9));
    enc.write_all(data).unwrap();
    let z = enc.finish().unwrap();
    let mut out = head.to_vec();
    out.extend_from_slice(&(z.len() as u32).to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&z);
    out
}

// ---------------------------------------------------------------- PlayerAssignment.bin (§2.3)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Asg { pub nr: u32, pub pid: u32, pub team: u32, pub k: u32 }
const PLATZ_MASK: u32 = 63 << 10;
impl Asg {
    pub fn platz(&self) -> u32 { (self.k >> 10) & 63 }
    pub fn nummer(&self) -> u32 { (self.k & 0x7f) + 1 }
}
pub fn asg_parse(d: &[u8]) -> Vec<Asg> {
    d.chunks_exact(16).map(|c| {
        let u = |i: usize| u32::from_le_bytes(c[i..i + 4].try_into().unwrap());
        Asg { nr: u(0), pid: u(4), team: u(8), k: u(12) }
    }).collect()
}
pub fn asg_bytes(a: &[Asg]) -> Vec<u8> {
    a.iter().flat_map(|x| [x.nr, x.pid, x.team, x.k].into_iter().flat_map(|v| v.to_le_bytes())).collect()
}
fn squad_order(a: &[Asg], team: u32) -> Vec<usize> {
    let mut v: Vec<usize> = (0..a.len()).filter(|&i| a[i].team == team).collect();
    v.sort_by_key(|&i| (a[i].platz(), i));
    v
}
/// Lineup: the wanted ids on places 0, 1, … (starting XI in formation order, then bench), everybody else behind in the
/// previous order. Only bits 10..15 change. Refuses players outside the club, duplicates and squads over 64.
/// Set-piece takers of a team in PlayerAssignment.bin: bits 16..21 of a record, exactly one player of the team each
/// (read from the FL27 world 24.11.: 21 captain, 20 penalties, 16/17 free kicks, 18/19 corners; which of each pair is
/// short/long or left/right is not confirmed, CalcioM has one taker for both). `takers`: free kick, corner, penalty,
/// captain; None or a player not in the team leaves that flag as it is.
pub fn schuetzen_setzen(a: &mut [Asg], team: u32, takers: [Option<u32>; 4]) {
    for (bits, pid) in [(&[16u32, 17][..], takers[0]), (&[18, 19][..], takers[1]), (&[20][..], takers[2]), (&[21][..], takers[3])] {
        let Some(pid) = pid else { continue };
        if !a.iter().any(|x| x.team == team && x.pid == pid) { continue; }
        for x in a.iter_mut().filter(|x| x.team == team) {
            for &b in bits { if x.pid == pid { x.k |= 1 << b; } else { x.k &= !(1 << b); } }
        }
    }
}
pub fn set_lineup(a: &mut [Asg], team: u32, order: &[u32]) -> Result<(), String> {
    let sq = squad_order(a, team);
    if sq.len() > 64 { return Err(format!("Kader {team} hat {} Spieler (höchstens 64)", sq.len())); }
    let mut seen = HashSet::new();
    for p in order {
        if !seen.insert(*p) { return Err(format!("Spieler {p} doppelt")); }
        if !sq.iter().any(|&i| a[i].pid == *p) { return Err(format!("Spieler {p} ist nicht im Verein {team}")); }
    }
    let mut folge: Vec<usize> = order.iter().map(|p| *sq.iter().find(|&&i| a[i].pid == *p).unwrap()).collect();
    folge.extend(sq.iter().copied().filter(|i| !order.contains(&a[*i].pid)));
    for (platz, i) in folge.into_iter().enumerate() { a[i].k = (a[i].k & !PLATZ_MASK) | ((platz as u32) << 10); }
    Ok(())
}
/// Transfer: the record at the old club gets the new team id; number = wish if free, else the first free one;
/// place = end of the new squad. No old record: new record with nr = max + 1. New team 0 (free agent): record removed.
pub fn transfer(a: &mut Vec<Asg>, pid: u32, from: u32, to: u32, wish: u32) -> Result<(), String> {
    let old = if from == 0 { None } else { a.iter().position(|x| x.pid == pid && x.team == from) };
    if to == 0 {
        if let Some(i) = old { a.remove(i); }
        return Ok(());
    }
    // already listed at the new club (a second record in the world, e.g. on loan or doubled in the data): keep that
    // one and drop the old record, so the squads stay as the career says (live writing failed here, 22.11.)
    if a.iter().any(|x| x.pid == pid && x.team == to) {
        if let Some(i) = old { a.remove(i); }
        return Ok(());
    }
    let used: HashSet<u32> = a.iter().filter(|x| x.team == to).map(|x| x.nummer()).collect();
    let no = if (1..=128).contains(&wish) && !used.contains(&wish) { wish } else { (1..=128).find(|n| !used.contains(n)).ok_or("keine Rückennummer frei")? };
    let platz = a.iter().filter(|x| x.team == to).map(|x| x.platz() + 1).max().unwrap_or(0).min(63);
    let k_new = |k: u32| (k & !0x7f & !PLATZ_MASK) | (no - 1) | (platz << 10);
    match old {
        Some(i) => { a[i].team = to; a[i].k = k_new(a[i].k); }
        None => {
            let nr = a.iter().map(|x| x.nr).max().unwrap_or(0) + 1;
            a.push(Asg { nr, pid, team: to, k: k_new(0) });
        }
    }
    Ok(())
}
/// Team of a side: the majority of the player ids' teams (§1.4.7).
pub fn team_majority(a: &[Asg], pids: &[u32], clubs: &HashSet<u32>) -> u32 {
    let mut c: HashMap<u32, u32> = HashMap::new();
    for x in a { if pids.contains(&x.pid) && (clubs.is_empty() || clubs.contains(&x.team)) { *c.entry(x.team).or_default() += 1; } }
    c.into_iter().max_by_key(|(t, n)| (*n, *t)).map(|x| x.0).unwrap_or(0)
}

// ---------------------------------------------------------------- competitions, stadium (§4)
/// Competition.bin: 36-byte records, 16-bit id = byte 5 | byte 7 << 8 (the FL_MegaPack has 726), name at +8.
pub fn competitions(d: &[u8]) -> Vec<(u16, String)> {
    d.chunks_exact(36).map(|r| (r[5] as u16 | (r[7] as u16) << 8, String::from_utf8_lossy(r[8..].split(|b| *b == 0).next().unwrap_or(&[])).to_string())).collect()
}
/// Competition.bin byte 3 marks the country: GERMANY_D1_LEAGUE and FL_011_LEAGUE (2. Bundesliga) share 176, England 16 (measured on MyWorld).
pub fn regionen(d: &[u8]) -> HashMap<u16, u8> { d.chunks_exact(36).map(|r| (r[5] as u16 | (r[7] as u16) << 8, r[3])).collect() }
/// One `league <n> key=wert … name=<Rest>` line of <Welt>/fl26world.txt. `above` and `cup` are numbers of the league
/// space (league lines and tournament ids of CompetitionRegulation), never competition ids.
#[derive(Clone, Debug, Default)]
pub struct FlLiga { pub n: u32, pub cid: u16, pub country: u32, pub tier: u8, pub above: Option<u32>, pub promote: u8, pub demote: u8, pub cup: Option<u32>, pub name: String }
pub fn fl26world(text: &str) -> Vec<FlLiga> {
    text.lines().filter_map(|z| {
        let rest = z.strip_prefix("league ")?;
        let (vorn, name) = rest.split_once(" name=").unwrap_or((rest, ""));
        let mut t = vorn.split_whitespace();
        let mut l = FlLiga { n: t.next()?.parse().ok()?, name: name.trim().to_string(), ..Default::default() };
        for kv in t {
            let Some((k, v)) = kv.split_once('=') else { continue };
            let Ok(x) = v.parse::<u32>() else { continue };
            match k { "cid" => l.cid = x as u16, "country" => l.country = x, "tier" => l.tier = x as u8, "above" => l.above = Some(x), "promote" => l.promote = x as u8, "demote" => l.demote = x as u8, "cup" => l.cup = Some(x), _ => {} }
        }
        (l.cid > 0).then_some(l)
    }).collect()
}
/// A file in `dir` whatever its case (game folders come from Windows).
pub fn datei_ci(dir: &Path, name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).find(|p| p.file_name().and_then(|n| n.to_str()).map(|n| n.eq_ignore_ascii_case(name)).unwrap_or(false))
}
/// Crests and competition logos of a career (Übergabe Daten §5.1–5.3, fl/bilder.py): file names only, read once.
/// Order: the career's world, the other active cpk.root folders of sider.ini in their order, the other livecpk folders;
/// the first folder with a file wins, inside a folder the best crest variant. assets/wappen comes after (ui.rs).
#[derive(Default, Debug)]
pub struct Bilder { pub verein: HashMap<u32, Quelle>, pub emblem: HashMap<u16, Quelle>, pub spieler: HashMap<u32, Quelle>, pub stadion: HashMap<u32, Quelle> }
/// Where a picture is: a loose file, or an entry of a CPK archive (archive, (offset, size, unpacked size)).
#[derive(Clone, Debug, PartialEq)]
pub enum Quelle { Datei(PathBuf), Cpk(PathBuf, (u64, u64, u64)) }
impl Quelle {
    /// The picture's bytes (PNG or DDS), unpacked.
    pub fn lesen(&self) -> Option<Vec<u8>> { match self { Quelle::Datei(p) => std::fs::read(p).ok(), Quelle::Cpk(a, e) => crate::cpk::lesen(a, *e).ok() } }
    pub fn datei(&self) -> Option<&Path> { match self { Quelle::Datei(p) => Some(p), _ => None } }
}
pub const WAPPEN_VARIANTEN: [&str; 9] = ["r_l", "r_w_l", "r_b_l", "r", "r_w", "r_b", "r_ll", "r_w_ll", "r_b_ll"];
/// A folder below `base` by parts, whatever their case.
fn ordner_ci(base: &Path, teile: &[&str]) -> Option<PathBuf> { teile.iter().try_fold(base.to_path_buf(), |p, t| datei_ci(&p, t).filter(|x| x.is_dir())) }
/// Active `cpk.root` folders of sider.ini in their order (commented lines do not count).
pub fn aktive_wurzeln(sider: &Path) -> Vec<PathBuf> {
    let ini = std::fs::read_to_string(sider.join("sider.ini")).unwrap_or_default();
    let mut v: Vec<PathBuf> = vec![];
    for z in ini.lines() {
        let Some(r) = z.trim_start().strip_prefix("cpk.root") else { continue };
        let r = r.trim_start().trim_start_matches('=').split(';').next().unwrap_or("").trim().trim_matches('"').replace('\\', "/");
        let p = sider.join(r.trim_start_matches("./"));
        if p.is_dir() && !v.contains(&p) { v.push(p); }
    }
    v
}
/// Kind, id and rank of a picture by its path (lower case): crest e_<id>_<variant>, logo emb_<id>_l, photo <id>;
/// .png before .dds, the better crest variant first (fl/bilder.py _schluessel_datei).
fn bild_schluessel(n: &str) -> Option<(u8, u32, usize)> {
    let (ordner, datei) = n.rsplit_once('/').unwrap_or(("", n));
    let (stamm, dds) = if let Some(x) = datei.strip_suffix(".png") { (x, 0) } else if let Some(x) = datei.strip_suffix(".dds") { (x, 1) } else { return None };
    let stamm = stamm.trim_end_matches(".png").trim_end_matches(".dds"); // "emb_0117_w_l.png.png"
    // the leading number after the prefix; unknown variants come after the known ones (.dds as well as .png)
    let nummer = |s: &str| -> Option<(u32, String)> { let n: String = s.chars().take_while(|c| c.is_ascii_digit()).collect(); Some((n.parse().ok()?, s[n.len()..].trim_start_matches('_').to_string())) };
    if ordner.ends_with("symbol/flag") || ordner == "flag" {
        let (id, var) = nummer(stamm.strip_prefix("e_")?)?;
        let rang = WAPPEN_VARIANTEN.iter().position(|v| *v == var).unwrap_or(WAPPEN_VARIANTEN.len() + usize::from(!var.is_empty()));
        return Some((0, id, rang * 2 + dds));
    }
    if ordner.ends_with("symbol/emblemlc") || ordner == "emblemlc" {
        let (id, var) = nummer(stamm.strip_prefix("emb_")?)?;
        let rang = ["l", "ll", "w_l", "w_ll", ""].iter().position(|v| *v == var).unwrap_or(5);
        return Some((1, id, rang * 2 + dds));
    }
    if ordner.ends_with("symbol/player") || ordner == "player" { return Some((2, stamm.parse().ok()?, dds)); }
    None
}
impl Bilder {
    /// Take the pictures of one source; inside it the best rank, against earlier sources nothing (the first source wins).
    fn aufnehmen(&mut self, eintraege: impl Iterator<Item = (String, Quelle)>) {
        let mut best: HashMap<(u8, u32), (usize, Quelle)> = HashMap::new();
        for (n, q) in eintraege {
            let Some((art, id, rang)) = bild_schluessel(&n) else { continue };
            if best.get(&(art, id)).map_or(true, |x| rang < x.0) { best.insert((art, id), (rang, q)); }
        }
        for ((art, id), (_, q)) in best {
            match art { 0 => { self.verein.entry(id).or_insert(q); } 1 => { self.emblem.entry(id as u16).or_insert(q); } _ => { self.spieler.entry(id).or_insert(q); } }
        }
    }
}
/// Crests, logos and player photos of a career (Übergabe Daten §5, fl/bilder.py index_bauen): the world folder and the
/// active cpk.root folders of sider.ini, then the game's CPK archives in Data/ and download/ (a later archive wins),
/// then the other livecpk folders. File names only; nothing is written.
pub fn bilder_index(welt: &Path, sider: &Path) -> Bilder {
    let mut vorn = vec![welt.to_path_buf()];
    for p in aktive_wurzeln(sider) { if !vorn.iter().any(|o| same(o, &p)) { vorn.push(p); } }
    let mut rest: Vec<PathBuf> = std::fs::read_dir(sider.join("livecpk")).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()).unwrap_or_default();
    rest.sort();
    rest.retain(|p| !vorn.iter().any(|o| same(o, p)));
    let ordner = |o: &Path| -> Vec<(String, Quelle)> {
        let mut v = vec![];
        for teil in ["flag", "emblemLc", "player"] {
            let Some(d) = ordner_ci(o, &["common", "render", "symbol", teil]) else { continue };
            for f in std::fs::read_dir(&d).into_iter().flatten().flatten() {
                v.push((format!("symbol/{}/{}", teil.to_lowercase(), f.file_name().to_string_lossy().to_lowercase()), Quelle::Datei(f.path())));
            }
        }
        v
    };
    let mut b = Bilder::default();
    for o in &vorn { b.aufnehmen(ordner(o).into_iter()); }
    let spiel = sider.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let mut archive: Vec<PathBuf> = vec![];
    for sub in ["Data", "download"] {
        let mut v: Vec<PathBuf> = std::fs::read_dir(spiel.join(sub)).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |e| e.eq_ignore_ascii_case("cpk"))).collect()).unwrap_or_default();
        v.sort();
        archive.extend(v);
    }
    for a in archive.iter().rev() {
        // an unreadable archive is skipped
        let Ok(c) = cz::cpk::Cpk::open(a) else { continue };
        b.aufnehmen(c.files.into_iter().filter(|(n, _)| n.contains("render/symbol/")).map(|(n, e)| (n, Quelle::Cpk(a.clone(), (e.offset, e.size, e.extract_size)))));
    }
    for o in &rest { b.aufnehmen(ordner(o).into_iter()); }
    b.stadion = stadien_index(sider);
    b
}
/// Stadium pictures: content/stadiums/map_teams.txt names each team's stadium folder; its preview
/// common/render/thumbnail/stadium/stNNN.dds is the picture (first entry of a team; nothing is written).
pub fn stadien_index(sider: &Path) -> HashMap<u32, Quelle> {
    let root = sider.join("content").join("stadiums");
    let Ok(mt) = std::fs::read_to_string(root.join("map_teams.txt")) else { return HashMap::new() };
    let mut bild: HashMap<String, Option<PathBuf>> = HashMap::new();
    let mut out = HashMap::new();
    for z in mt.trim_start_matches('\u{feff}').lines() {
        let t: Vec<&str> = z.split('#').next().unwrap_or("").split(',').map(|x| x.trim()).collect();
        let (Some(id), Some(pfad)) = (t.first().and_then(|x| x.parse::<u32>().ok()), t.get(3).filter(|p| !p.is_empty())) else { continue };
        if out.contains_key(&id) { continue; }
        let b = bild.entry(pfad.to_string()).or_insert_with(|| {
            let d = pfad.split(['\\', '/']).try_fold(root.clone(), |p, s| datei_ci(&p, s))?;
            let t = ordner_ci(&d, &["common", "render", "thumbnail", "stadium"])?;
            let mut v: Vec<PathBuf> = std::fs::read_dir(t).ok()?.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |e| e.eq_ignore_ascii_case("dds") || e.eq_ignore_ascii_case("png"))).collect();
            v.sort();
            v.into_iter().next()
        });
        if let Some(p) = b { out.insert(id, Quelle::Datei(p.clone())); }
    }
    out
}
fn same(a: &Path, b: &Path) -> bool { a.canonicalize().ok().zip(b.canonicalize().ok()).map_or(a == b, |(x, y)| x == y) }
/// CompetitionRegulation.bin: records of 2352 bytes, u16 tournament id at +2, competition id at +8; the first one counts.
/// The competition id stays one byte here even in the FL_MegaPack (no byte layout gives more than 261 distinct ids
/// over its 1 268 records): competitions from id 256 have no regulation of their own.
pub fn turnier_ids(d: &[u8]) -> HashMap<u16, u16> {
    let mut out = HashMap::new();
    for r in d.chunks_exact(2352) { out.entry(r[8] as u16).or_insert(u16::from_le_bytes([r[2], r[3]])); }
    out
}
/// Tournament ids a map_competitions.txt (scoreboards, referee …) has an entry for: "tid, path  # comment". An entry
/// whose path starts with another world's folder (`fremd`, e.g. "_FL26Megapack\A-League Men" in a MyWorld career)
/// belongs to that world's ids and is left out.
pub fn karten_tids(text: &str, fremd: &[String]) -> HashSet<u16> {
    text.trim_start_matches('\u{feff}').lines().filter_map(|z| {
        let (id, pfad) = z.split('#').next()?.split_once(',')?;
        let erster = pfad.trim().split(['\\', '/']).next().unwrap_or("");
        if fremd.iter().any(|f| f.eq_ignore_ascii_case(erster)) { return None; }
        id.trim().parse().ok()
    }).collect()
}
/// Tournament ids Entrance.lua (walk-out scenes) has a branch for: every "tid == N" outside a "--" comment.
pub fn entrance_tids(lua: &str) -> HashSet<u16> {
    lua.lines().flat_map(|z| z.split("--").next().unwrap_or("").split("tid == ").skip(1).filter_map(|s| s.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()).collect::<Vec<u16>>()).collect()
}
/// The tournament id for flw/wettbewerb.txt: the first candidate (closest first: exact, same kind, same country) that
/// any of the content maps knows; none known: the first candidate (the modules then use their exhibition look).
pub fn turnier_waehlen(kandidaten: &[u16], karten: &[HashSet<u16>]) -> Option<u16> {
    kandidaten.iter().find(|t| karten.iter().any(|k| k.contains(t))).or(kandidaten.first()).copied()
}
/// content/stadiums/map_teams.txt: "Team-ID, Stadion-ID, Name, Pfad  # Kommentar"; the first entry of the team.
pub fn heimstadion(text: &str, team: u32) -> Option<String> {
    text.lines().find_map(|z| {
        let t: Vec<&str> = z.split('#').next().unwrap_or("").split(',').map(|x| x.trim()).collect();
        (t.len() >= 4 && t[0].parse::<u32>().ok() == Some(team) && !t[3].is_empty()).then(|| t[3].to_string())
    })
}

pub const WETTER: [&str; 4] = ["sunny", "cloudy", "rainy", "snowy"];
pub const TAGESZEIT: [&str; 2] = ["tag", "nacht"];
/// Writes S/flw/{wettbewerb,wetter,stadion}.txt (UTF-8, CRLF, atomic). None removes the file (the modules choose).
pub fn vorgaben_schreiben(s: &Path, turnier: Option<u16>, wetter: Option<&str>, tageszeit: Option<&str>, stadion: Option<&str>) -> Result<(), String> { vorgaben_schreiben_mit(s, turnier, wetter, tageszeit, stadion, None) }
/// The same plus the match rules for the LivePatch module (spielregeln.txt: extra time and penalties on or off,
/// the first leg's score as a note; idea after BlankState222's two_legs, written anew).
pub fn vorgaben_schreiben_mit(s: &Path, turnier: Option<u16>, wetter: Option<&str>, tageszeit: Option<&str>, stadion: Option<&str>, regeln: Option<String>) -> Result<(), String> {
    if let Some(w) = wetter { if !WETTER.contains(&w) { return Err(format!("Wetter {w:?} unbekannt")); } }
    if let Some(t) = tageszeit { if !TAGESZEIT.contains(&t) { return Err(format!("Tageszeit {t:?} unbekannt")); } }
    let o = s.join("flw");
    let set = |name: &str, text: Option<String>| -> Result<(), String> {
        let p = o.join(name);
        match text {
            Some(t) => atomic_write(&p, t.as_bytes()),
            None => { wache(&p); if p.exists() { std::fs::remove_file(&p).map_err(|e| e.to_string())?; } Ok(()) }
        }
    };
    set("wettbewerb.txt", turnier.map(|t| format!("{t}\r\n")))?;
    let wz: String = [("wetter", wetter), ("tageszeit", tageszeit)].iter().filter_map(|(k, v)| v.map(|v| format!("{k}={v}\r\n"))).collect();
    set("wetter.txt", (!wz.is_empty()).then_some(wz))?;
    set("stadion.txt", stadion.filter(|x| !x.is_empty()).map(|x| format!("{x}\r\n")))?;
    set("spielregeln.txt", regeln)?; // roh
    Ok(())
}

/// The three preset files of a match start (CommonLib.lua pesm_wettbewerb, stadiums.lua pesm_stadion, weather.lua): without
/// the file every module falls back to the game's normal choice by teams and home side.
pub const VORGABEN: [&str; 4] = ["wettbewerb.txt", "stadion.txt", "wetter.txt", "spielregeln.txt"]; // roh
/// The presets now in S/flw with their SHA-256: what CalcioM wrote, so only that is undone later.
pub fn vorgaben_merken(s: &Path) -> Vec<(String, String)> {
    VORGABEN.iter().filter_map(|n| std::fs::read(s.join("flw").join(n)).ok().map(|b| (n.to_string(), sha256_hex(&b)))).collect()
}
/// Undo after a match: removes the presets CalcioM wrote and nobody changed since. Changed files stay and are returned.
pub fn vorgaben_zuruecksetzen(s: &Path, gesetzt: &[(String, String)]) -> Result<Vec<String>, String> {
    let mut geaendert = vec![];
    for (n, sha) in gesetzt {
        let p = s.join("flw").join(n);
        let Ok(b) = std::fs::read(&p) else { continue };
        if sha256_hex(&b) == *sha { wache(&p); std::fs::remove_file(&p).map_err(|e| format!("{n}: {e}"))?; } else { geaendert.push(n.clone()); }
    }
    Ok(geaendert)
}

// ---------------------------------------------------------------- sider.ini (§3.3, §3.4, §4 Direktstart)
fn root_line(line: &str, name: &str) -> Option<bool> {
    let t = line.trim_start();
    let (active, rest) = if t.starts_with('#') || t.starts_with(';') { (false, t[1..].trim_start()) } else { (true, t) };
    (rest.starts_with("cpk.root") && rest.contains(&format!("\\livecpk\\{name}\""))).then_some(active)
}
fn uncomment(line: &str) -> String {
    let i = line.len() - line.trim_start().len();
    format!("{}{}", &line[..i], &line[i + 1..])
}
fn indent_of(l: &str) -> String { l.chars().take_while(|c| c.is_whitespace()).collect() }
fn root_line_of(l: &str, name: &str, indent: &str) -> String { format!("{indent}cpk.root = \".\\livecpk\\{name}\"") }
/// sider.ini for a career (Übergabe Daten und Datenbank §8.1, idempotent): active CPK lines of other CalcioM careers are
/// commented out with ";"; the career's own line is removed and set again as the first active line, directly below the
/// active CalcioM_DirectMatch line (without it: before the first cpk.root line); a commented world line is switched on,
/// a missing one is added directly below the career line. With `mit_direktstart` a missing CalcioM_DirectMatch line becomes
/// the first root (FL27 spec §4). Line endings and indentation are kept; a correct file stays byte-equal.
pub fn sider_ini_einrichten(text: &str, cpk: &str, welt: &str, andere: &[String], mit_direktstart: bool) -> Result<String, String> {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let trailing = text.ends_with('\n');
    let mut lines: Vec<String> = text.split(eol).map(|s| s.to_string()).collect();
    if trailing { lines.pop(); }
    let find = |lines: &Vec<String>, n: &str| lines.iter().position(|l| root_line(l, n).is_some());
    for n in andere.iter().filter(|n| n.as_str() != cpk) {
        for l in lines.iter_mut() { if root_line(l, n) == Some(true) { let i = indent_of(l); *l = format!("{i};{}", l.trim_start()); } }
    }
    // any other active CalcioM career folder (the list can miss one whose name was folded, e.g. „München“ → „Munchen“)
    for l in lines.iter_mut() {
        let t = l.trim_start();
        let Some(n) = t.strip_prefix("cpk.root").and_then(|r| r.split("\\livecpk\\").nth(1)).map(|r| r.trim_end().trim_end_matches('"')) else { continue };
        if crate::fl27::eigener_cpk(n) && n != DIRECT_ROOT && n != cpk { let i = indent_of(l); *l = format!("{i};{}", l.trim_start()); }
    }
    if mit_direktstart {
        match find(&lines, DIRECT_ROOT) {
            Some(i) if root_line(&lines[i], DIRECT_ROOT) == Some(false) => lines[i] = uncomment(&lines[i]),
            Some(_) => {}
            None => {
                let first = lines.iter().position(|l| root_line_any(l)).ok_or("keine cpk.root-Zeile in sider.ini")?;
                let indent = indent_of(&lines[first]);
                lines.insert(first, root_line_of("", DIRECT_ROOT, &indent));
                lines.insert(first, format!("{indent};--- CalcioM Direktstart (Stufe 1) ---"));
            }
        }
    }
    // own line: where it belongs
    let want_at = |lines: &Vec<String>| -> Option<usize> {
        find(lines, DIRECT_ROOT).filter(|&d| root_line(&lines[d], DIRECT_ROOT) == Some(true)).map(|d| d + 1)
            .or_else(|| lines.iter().position(|l| root_line_any(l)))
    };
    let own: Vec<usize> = (0..lines.len()).filter(|&i| root_line(&lines[i], cpk).is_some()).collect();
    let correct = own.len() == 1 && root_line(&lines[own[0]], cpk) == Some(true) && {
        let mut t = lines.clone(); t.remove(own[0]); want_at(&t) == Some(own[0])
    };
    if !correct {
        for i in own.into_iter().rev() { lines.remove(i); }
        let at = want_at(&lines).ok_or("keine cpk.root-Zeile in sider.ini")?;
        let indent = indent_of(&lines[at.min(lines.len() - 1)]);
        lines.insert(at, root_line_of("", cpk, &indent));
    }
    match find(&lines, welt) {
        Some(w) if root_line(&lines[w], welt) == Some(false) => lines[w] = uncomment(&lines[w]),
        Some(_) => {}
        None => {
            let c = find(&lines, cpk).unwrap();
            let indent = indent_of(&lines[c]);
            lines.insert(c + 1, root_line_of("", welt, &indent));
        }
    }
    // the world above the career would win with its own database (Sider: the first root wins): move it below
    if let (Some(w), Some(c)) = (find(&lines, welt), find(&lines, cpk)) {
        if w < c { let l = lines.remove(w); let c = find(&lines, cpk).unwrap(); lines.insert(c + 1, l); }
    }
    let mut out = lines.join(eol);
    if trailing { out.push_str(eol); }
    Ok(out)
}
/// Switches add-on folders on again that earlier versions switched off: active in the sider.ini from before the career
/// (`vorher`), commented out now, with pesdb files but no Player.bin (boots, derbies, a cup, stadiums).
pub fn zusaetze_wieder_an(text: &str, vorher: &str, s: &Path) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let name = |l: &str| l.split("\\livecpk\\").nth(1).map(|r| r.trim_end().trim_end_matches('"').to_string()); // roh
    let aktiv: HashSet<String> = vorher.lines().filter(|l| { let t = l.trim_start(); t.starts_with("cpk.root") }).filter_map(name).collect(); // roh
    text.split(eol).map(|l| match name(l) {
        Some(n) if aktiv.contains(&n) && root_line(l, &n) == Some(false) && !eigener_cpk(&n) && n != DIRECT_ROOT && {
            let db = s.join("livecpk").join(&n).join("common/etc/pesdb"); // roh
            db.is_dir() && !db.join("Player.bin").exists() // roh
        } => uncomment(l),
        _ => l.to_string(),
    }).collect::<Vec<_>>().join(eol)
}
fn root_line_any(l: &str) -> bool { let t = l.trim_start(); let t = t.strip_prefix(';').or_else(|| t.strip_prefix('#')).unwrap_or(t).trim_start(); t.starts_with("cpk.root") }
/// Comments out the career's active line (after the match, if the person wants the career database off); the next match
/// start writes it active again (sider_ini_einrichten).
pub fn sider_ini_abschalten(text: &str, cpk: &str) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    text.split(eol).map(|l| if root_line(l, cpk) == Some(true) { format!("{};{}", indent_of(l), l.trim_start()) } else { l.to_string() }).collect::<Vec<_>>().join(eol)
}
/// sider.ini as without CalcioM (Fabian 26.11.: FL27 started by hand must open its main menu with its own database):
/// the career's folder, the direct start's folder and CalcioM's Lua modules are switched off with ";". Everything else
/// stays; the next match start from CalcioM switches them on again.
pub fn sider_ini_ohne_calciom(text: &str, cpk: &str) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let unser = |l: &str| {
        let t = l.trim_start();
        if t.starts_with(';') || t.starts_with('#') { return false; }
        root_line(l, cpk) == Some(true) || root_line(l, DIRECT_ROOT) == Some(true)
            || (t.starts_with("lua.module") && (t.contains("calciom_direktmatch.lua") || t.contains("pesm-livepatch.lua"))) // roh
    };
    text.split(eol).map(|l| if unser(l) { format!("{};{}", indent_of(l), l.trim_start()) } else { l.to_string() }).collect::<Vec<_>>().join(eol)
}
/// Removes the career's line; everything else stays.
pub fn sider_ini_rueckbau(text: &str, cpk: &str) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    text.split(eol).filter(|l| root_line(l, cpk).is_none()).collect::<Vec<_>>().join(eol)
}

// ---------------------------------------------------------------- original backup, career pesdb, loading (§2.4, §3)
/// Backs up the world folder and sider.ini once to <karriere>/original/ with a SHA-256 list (inhalt.json). Never overwrites.
pub fn welt_name(welt: &Path) -> String { welt.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| WELT_ROOT.into()) }
pub fn original_sichern(karriere: &Path, s: &Path, welt: &Path) -> Result<(), String> {
    let wn = welt_name(welt);
    let dest = karriere.join("original");
    if dest.join("inhalt.json").exists() { return original_pruefen(karriere); }
    let mut files = vec![];
    fort(0, 0);
    let n = dateien_zaehlen(welt);
    log(&format!("Sicherung der Welt {} beginnt: {n} Dateien", welt.display()));
    fort(1, n);
    // slim (option, off by default, Fabian 24.11.): only the database CalcioM writes and restores, not the pictures
    let teil = if sicherung_schlank() { Path::new("common/etc/pesdb") } else { Path::new("") };
    copy_dir(welt, &dest.join(&wn), &mut files, teil)?;
    std::fs::copy(s.join("sider.ini"), dest.join("sider.ini")).map_err(|e| format!("sider.ini: {e}"))?;
    fort(2, files.len());
    let mut list = serde_json::Map::new();
    for f in &files { let r = Path::new(&wn).join(f); list.insert(r.display().to_string(), sha_file(&dest.join(&r))?.into()); FORT_FERTIG.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
    list.insert("sider.ini".into(), sha_file(&dest.join("sider.ini"))?.into());
    let txt = serde_json::to_string_pretty(&list).unwrap();
    atomic_write(&dest.join("inhalt.json"), txt.as_bytes())?;
    // the sums were just made from the copy: checking them again at the next start would read every file a second time
    atomic_write(&dest.join("geprueft.stamp"), sha256_hex(txt.as_bytes()).as_bytes())?;
    log(&format!("Sicherung fertig: {} Dateien", files.len()));
    Ok(())
}
/// Whether new careers back up only the world's database (`sicherung_schlank.txt` next to the careers; the settings
/// switch writes it). Off by default: the full world copy stays the safe way back.
pub fn sicherung_schlank() -> bool { schlank_datei().exists() }
pub fn sicherung_schlank_setzen(an: bool) { let f = schlank_datei(); if an { let _ = std::fs::write(&f, b"1"); } else { let _ = std::fs::remove_file(&f); } }
/// Next to the careers folder (the same place as game::save_dir; fl27.rs is shared with calciom-speicher, which has no game).
fn schlank_datei() -> PathBuf {
    let k = karrieren_ordner();
    k.join("../sicherung_schlank.txt") // roh
}
pub fn original_pruefen(karriere: &Path) -> Result<(), String> {
    let dest = karriere.join("original");
    let txt = std::fs::read_to_string(dest.join("inhalt.json")).map_err(|_| "Sicherung der Original-Welt fehlt".to_string())?;
    let list: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&txt).map_err(|e| e.to_string())?;
    // the full hash pass runs once per manifest (tens of thousands of files froze every match start); after that a
    // stamp with the manifest's hash allows a cheap existence check. Deleting geprueft.stamp forces the full pass again.
    let stempel = dest.join("geprueft.stamp");
    let marke = sha256_hex(txt.as_bytes());
    if std::fs::read_to_string(&stempel).ok().as_deref() == Some(marke.as_str()) {
        return match list.keys().find(|f| !dest.join(f).is_file()) { Some(f) => Err(format!("Sicherung beschädigt: {f} fehlt")), None => Ok(()) };
    }
    fort(2, list.len());
    for (f, h) in &list {
        FORT_FERTIG.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if sha_file(&dest.join(f)).ok().as_deref() != h.as_str() { return Err(format!("Sicherung beschädigt: {f}")); }
    }
    atomic_write(&stempel, marke.as_bytes())
}
pub fn karriere_pesdb(karriere: &Path) -> PathBuf { karriere.join("pesdb") }
/// The career works on its own copy of the pesdb (made from the original backup, never from the game folder).
pub fn karriere_pesdb_anlegen(karriere: &Path, welt: &Path) -> Result<(), String> {
    let to = karriere_pesdb(karriere);
    if to.join("PlayerAssignment.bin").exists() { return Ok(()); }
    fort(3, 0);
    let mut files = vec![];
    copy_dir(&karriere.join("original").join(welt_name(welt)).join("common/etc/pesdb"), &to, &mut files, Path::new(""))
}
pub fn asg_laden(pesdb: &Path) -> Result<([u8; 8], Vec<Asg>), String> {
    let raw = std::fs::read(pesdb.join("PlayerAssignment.bin")).map_err(|e| format!("PlayerAssignment.bin: {e}"))?;
    let (h, d) = unpack(&raw)?;
    Ok((h, asg_parse(&d)))
}
pub fn pesdb_datei(pesdb: &Path, name: &str) -> Result<Vec<u8>, String> {
    let raw = std::fs::read(pesdb.join(name)).map_err(|e| format!("{name}: {e}"))?;
    if raw.len() > 8 && raw[3..8].eq_ignore_ascii_case(b"WESYS") { Ok(unpack(&raw)?.1) } else { Ok(raw) }
}
/// Writes PlayerAssignment.bin of the career copy: backup (the last five kept), atomic write, read back and compare
/// every record with the intended list, otherwise the backup goes back.
/// Writes Player.bin (when given) and PlayerAssignment.bin of the career copy: backup of both (the last five kept),
/// atomic writes, read back and compared; on a difference both backups go back.
pub fn pesdb_schreiben(karriere: &Path, player: Option<&[u8]>, head: [u8; 8], a: Option<&[Asg]>) -> Result<(), String> {
    let pesdb = karriere_pesdb(karriere);
    let res = (|| {
        if let Some(b) = player { atomic_write(&pesdb.join("Player.bin.neu"), b)?; }
        match a { Some(a) => asg_schreiben(karriere, head, a)?, None => { sichern(karriere)?; } }
        if player.is_some() { std::fs::rename(pesdb.join("Player.bin.neu"), pesdb.join("Player.bin")).map_err(|e| e.to_string())?; }
        if let Some(b) = player { if std::fs::read(pesdb.join("Player.bin")).ok().as_deref() != Some(b) { return Err("Player.bin weicht nach dem Schreiben ab".to_string()); } }
        Ok(())
    })();
    if res.is_err() {
        let _ = std::fs::remove_file(pesdb.join("Player.bin.neu"));
        let mut old: Vec<PathBuf> = std::fs::read_dir(karriere.join("pesdb-sicherungen")).map(|d| d.flatten().map(|e| e.path()).collect()).unwrap_or_default();
        old.sort();
        if let Some(last) = old.last() { for f in ["Player.bin", "PlayerAssignment.bin"] { let _ = std::fs::copy(last.join(f), pesdb.join(f)); } }
    }
    res
}
/// Backup of Player.bin and PlayerAssignment.bin of the career copy before a write (the last five kept).
fn sichern(karriere: &Path) -> Result<PathBuf, String> {
    let pesdb = karriere_pesdb(karriere);
    let sich = karriere.join("pesdb-sicherungen");
    let dir = sich.join(stamp());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for f in ["Player.bin", "PlayerAssignment.bin"] { let _ = std::fs::copy(pesdb.join(f), dir.join(f)); }
    let mut old: Vec<PathBuf> = std::fs::read_dir(&sich).map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect()).unwrap_or_default();
    old.sort();
    while old.len() > 5 { let _ = std::fs::remove_dir_all(old.remove(0)); }
    Ok(dir)
}
/// Live writing: the changed files of the career copy also into its livecpk folder, when the career was loaded there
/// once (S/livecpk/<cpk>/common/etc/pesdb); atomic, compared by SHA-256, the previous file back on a difference.
pub fn spiegeln(karriere: &Path, s: &Path, cpk: &str, dateien: &[&str]) -> Result<(), String> {
    let ziel = s.join("livecpk").join(cpk).join("common/etc/pesdb");
    if !ziel.is_dir() { return Ok(()); }
    for f in dateien {
        let b = std::fs::read(karriere_pesdb(karriere).join(f)).map_err(|e| format!("{f}: {e}"))?;
        let p = ziel.join(f);
        let vorher = std::fs::read(&p).ok();
        atomic_write(&p, &b)?;
        if std::fs::read(&p).map(|x| sha256_hex(&x)).ok() != Some(sha256_hex(&b)) {
            if let Some(v) = vorher { let _ = atomic_write(&p, &v); }
            return Err(format!("{f} im Karriere-Ordner von FL27 weicht nach dem Schreiben ab"));
        }
    }
    Ok(())
}
pub fn asg_schreiben(karriere: &Path, head: [u8; 8], a: &[Asg]) -> Result<(), String> {
    let pesdb = karriere_pesdb(karriere);
    let dir = sichern(karriere)?;
    let p = pesdb.join("PlayerAssignment.bin");
    let res = atomic_write(&p, &pack(head, &asg_bytes(a))).and_then(|_| {
        let (_, back) = asg_laden(&pesdb)?;
        if back == a { Ok(()) } else { Err("PlayerAssignment.bin weicht nach dem Schreiben ab".into()) }
    });
    if res.is_err() { let _ = std::fs::copy(dir.join("PlayerAssignment.bin"), &p); }
    res
}
/// Puts the career's pesdb into S/livecpk/PESM_Karriere (copy of all pesdb files plus character0 of the world),
/// checks every copied file by SHA-256 and rolls back on a difference. The game must be closed.
// ---------------------------------------------------------------- formation (TacticsFormation.bin, docs/FORSCHUNG-TAKTIK.md)
fn le32(d: &[u8], o: usize) -> u32 { u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]) }
/// The formation id the game uses for a team: the first of its two Tactics.bin records (confirmed in game 22.11.).
fn formation_kennung(pesdb: &Path, team: u32) -> Result<u32, String> {
    let (_, t) = unpack(&std::fs::read(pesdb.join("Tactics.bin")).map_err(|e| format!("Tactics.bin: {e}"))?)?;
    (0..t.len() / 12).map(|i| i * 12).find(|&o| le32(&t, o + 4) == team).map(|o| le32(&t, o)).ok_or_else(|| format!("Team {team} nicht in Tactics.bin")) // roh
}
/// Position codes (0 GK … 12 CF) of the formation the game uses for a team, in the order of its places 0..10.
pub fn formation_codes(pesdb: &Path, team: u32) -> Result<Vec<u8>, String> {
    let k = formation_kennung(pesdb, team)?;
    let (_, f) = unpack(&std::fs::read(pesdb.join("TacticsFormation.bin")).map_err(|e| format!("TacticsFormation.bin: {e}"))?)?;
    let mut v: Vec<(u16, u8)> = (0..f.len() / 12).map(|i| i * 12).filter(|&o| le32(&f, o) == k).map(|o| (u16::from_le_bytes([f[o + 10], f[o + 11]]), le32(&f, o + 4) as u8)).filter(|x| x.0 >> 4 == 0).collect();
    v.sort();
    Ok(v.into_iter().map(|x| x.1).collect())
}
/// Writes a formation for a team: place i gets (position code, depth 0..47, width 0..104) in all three sets, so the
/// game shows it whatever set it starts with. Returns whether the file changed.
pub fn formation_schreiben(pesdb: &Path, team: u32, plaetze: &[(u8, u8, u8)]) -> Result<bool, String> { formation_schreiben_saetze(pesdb, team, [plaetze, plaetze, plaetze]) }
/// The same with its own places per set (Fabian 24.11., The Gaffer's three formations: 0 kick-off, 1 and 2 with and
/// against the ball – which of 1/2 is which is not confirmed in the game yet).
pub fn formation_schreiben_saetze(pesdb: &Path, team: u32, saetze: [&[(u8, u8, u8)]; 3]) -> Result<bool, String> {
    if saetze.iter().any(|plaetze| plaetze.len() != 11 || plaetze.iter().any(|p| p.0 > 12 || p.1 > 47 || p.2 > 104)) { return Err("Formation ungültig".into()); } // roh
    let k = formation_kennung(pesdb, team)?;
    let p = pesdb.join("TacticsFormation.bin");
    let (kopf, mut f) = unpack(&std::fs::read(&p).map_err(|e| format!("TacticsFormation.bin: {e}"))?)?;
    let orte: Vec<usize> = (0..f.len() / 12).map(|i| i * 12).filter(|&o| le32(&f, o) == k).collect();
    let alt = f.clone();
    for o in orte {
        let w = u16::from_le_bytes([f[o + 10], f[o + 11]]);
        let (platz, satz) = ((w & 15) as usize, ((w >> 4) as usize).min(2));
        if platz >= 11 { continue; }
        let (c, t, q) = saetze[satz][platz];
        f[o + 4..o + 8].copy_from_slice(&(c as u32).to_le_bytes());
        f[o + 8] = t;
        f[o + 9] = q;
    }
    if f == alt { return Ok(false); }
    atomic_write(&p, &pack(kopf, &f))?;
    Ok(true)
}
/// Sliders of a team in the game (Tactics.bin, first record; confirmed in game 22.11.: shown value = half-byte + 1):
/// compactness (half-byte 0), support range (1) and defensive line (2), each 1..10; None leaves a value as it is.
pub fn regler_schreiben(pesdb: &Path, team: u32, kompakt: Option<u8>, unterstuetzung: Option<u8>, linie: Option<u8>) -> Result<bool, String> {
    let p = pesdb.join("Tactics.bin");
    let (kopf, mut t) = unpack(&std::fs::read(&p).map_err(|e| format!("Tactics.bin: {e}"))?)?;
    let o = (0..t.len() / 12).map(|i| i * 12).find(|&o| le32(&t, o + 4) == team).ok_or_else(|| format!("Team {team} nicht in Tactics.bin"))?; // roh
    let alt = le32(&t, o + 8);
    let mut v = alt;
    for (sh, w) in [(0u32, kompakt), (4, unterstuetzung), (8, linie)] {
        if let Some(w) = w { v = (v & !(15 << sh)) | (((w.clamp(1, 10) - 1) as u32) << sh); }
    }
    if v == alt { return Ok(false); }
    t[o + 8..o + 12].copy_from_slice(&v.to_le_bytes());
    atomic_write(&p, &pack(kopf, &t))?;
    Ok(true)
}
/// Set 0 of a team's formation record `rec` (0 the one the game uses, 1 the second record): (code, depth, width) × 11.
pub fn formation_satz(pesdb: &Path, team: u32, rec: usize) -> Option<Vec<(u8, u8, u8)>> {
    let (_, t) = unpack(&std::fs::read(pesdb.join("Tactics.bin")).ok()?).ok()?;
    let k = (0..t.len() / 12).map(|i| i * 12).filter(|&o| le32(&t, o + 4) == team).nth(rec).map(|o| le32(&t, o))?;
    let (_, f) = unpack(&std::fs::read(pesdb.join("TacticsFormation.bin")).ok()?).ok()?;
    formation_aus(&f, k)
}
fn formation_aus(f: &[u8], k: u32) -> Option<Vec<(u8, u8, u8)>> {
    let mut v: Vec<(u16, (u8, u8, u8))> = (0..f.len() / 12).map(|i| i * 12).filter(|&o| le32(f, o) == k).map(|o| (u16::from_le_bytes([f[o + 10], f[o + 11]]), (le32(f, o + 4) as u8, f[o + 8], f[o + 9]))).filter(|x| x.0 >> 4 == 0).collect();
    v.sort_by_key(|x| x.0);
    (v.len() == 11).then(|| v.into_iter().map(|x| x.1).collect())
}
/// Every team's formation (both records) of a world: team id → [record 0, record 1].
pub fn alle_formationen(pesdb: &Path) -> std::collections::HashMap<u32, Vec<Vec<(u8, u8, u8)>>> {
    let mut out: std::collections::HashMap<u32, Vec<Vec<(u8, u8, u8)>>> = Default::default();
    let (Some(t), Some(f)) = (std::fs::read(pesdb.join("Tactics.bin")).ok().and_then(|b| unpack(&b).ok()), std::fs::read(pesdb.join("TacticsFormation.bin")).ok().and_then(|b| unpack(&b).ok())) else { return out };
    for o in (0..t.1.len() / 12).map(|i| i * 12) {
        if let Some(x) = formation_aus(&f.1, le32(&t.1, o)) { out.entry(le32(&t.1, o + 4)).or_default().push(x); }
    }
    out
}
/// The name of a formation as the game shows it („4-2-1-3“): the defenders, the midfield in rows by depth (gap of 4
/// or more), a second striker as a row of his own, the wingers and the centre forward together in front.
pub fn formation_name(p: &[(u8, u8, u8)]) -> String {
    let abwehr = p.iter().filter(|x| (1..=3).contains(&x.0)).count();
    let mut mitte: Vec<u8> = p.iter().filter(|x| (4..=8).contains(&x.0)).map(|x| x.1).collect();
    mitte.sort();
    let mut reihen: Vec<usize> = vec![abwehr];
    let mut vorher: Option<u8> = None;
    for d in mitte { if vorher.map_or(true, |v| d >= v + 4) { reihen.push(1); } else { *reihen.last_mut().unwrap() += 1; } vorher = Some(d); }
    let hs = p.iter().filter(|x| x.0 == 11).count();
    if hs > 0 { reihen.push(hs); }
    let vorn = p.iter().filter(|x| matches!(x.0, 9 | 10 | 12)).count();
    if vorn > 0 { reihen.push(vorn); }
    reihen.iter().filter(|&&n| n > 0).map(|n| n.to_string()).collect::<Vec<_>>().join("-")
}
/// The places in the order the world's data use: keeper, centre-backs, right back, left back, defensive, central, right,
/// left midfield, attacking midfield, right, left wing, second striker, centre forward; on the same role right before left.
pub fn formation_ordnen<T>(p: &mut [(u8, u8, u8, T)]) {
    let rang = |c: u8| [0u8, 1, 3, 2, 4, 5, 7, 6, 8, 10, 9, 11, 12].iter().position(|&x| x == c).unwrap_or(13);
    p.sort_by_key(|x| (rang(x.0), std::cmp::Reverse(x.2)));
}
/// The eleven in the order of the game's formation: each place takes the player whose CalcioM slot (or position) fits
/// its code best, so nobody ends up on the wrong place when the formation itself is the world's (opponents).
pub fn nach_formation(codes: &[u8], elf: &[(u8, u32)]) -> Vec<u32> {
    let gruppe = |c: u8| match c { 0 => 0, 1..=3 => 1, 4..=8 => 2, _ => 3 };
    let seite = |c: u8| -> i32 { match c { 2 | 6 | 9 => -1, 3 | 7 | 10 => 1, _ => 0 } };
    let mut frei: Vec<(u8, u32)> = elf.to_vec();
    let mut out = vec![0u32; codes.len()];
    let mut offen: Vec<usize> = vec![];
    for (i, &c) in codes.iter().enumerate() {
        if let Some(j) = frei.iter().position(|x| x.0 == c) { out[i] = frei.remove(j).1; } else { offen.push(i); }
    }
    for i in offen {
        let c = codes[i];
        let j = (0..frei.len()).min_by_key(|&j| { let x = frei[j].0; ((gruppe(x) as i32 - gruppe(c) as i32).abs() * 10 + (seite(x) - seite(c)).abs(), j) });
        if let Some(j) = j { out[i] = frei.remove(j).1; }
    }
    out
}
/// Every file of the career's pesdb that differs in the game's copy is written again (the live writing keeps only
/// Player.bin and PlayerAssignment.bin current; formation and tactics changes would otherwise never reach the game).
/// Returns the names of the files written.
pub fn karriere_abgleichen(karriere: &Path, s: &Path, cpk: &str) -> Result<Vec<String>, String> {
    let dst = s.join("livecpk").join(cpk).join("common/etc/pesdb");
    wache(&dst);
    let mut out = vec![];
    for e in std::fs::read_dir(karriere_pesdb(karriere)).map_err(|e| e.to_string())?.flatten() {
        if !e.path().is_file() { continue; }
        let b = std::fs::read(e.path()).map_err(|x| format!("{}: {x}", e.path().display()))?;
        let ziel = dst.join(e.file_name());
        if std::fs::read(&ziel).ok().as_deref() != Some(&b[..]) { atomic_write(&ziel, &b)?; out.push(e.file_name().to_string_lossy().to_string()); }
    }
    Ok(out)
}
pub fn karriere_laden(karriere: &Path, s: &Path, welt: &Path, cpk: &str) -> Result<(), String> {
    if fl27_pid().is_some() { return Err("Bitte FL27 erst beenden.".into()); }
    let root = s.join("livecpk").join(cpk);
    wache(&root);
    let backup = karriere.join("cpk-vorher").join(stamp());
    if root.exists() { let mut f = vec![]; copy_dir(&root, &backup, &mut f, Path::new(""))?; }
    let result = (|| -> Result<(), String> {
        let src = karriere_pesdb(karriere);
        let dst = root.join("common/etc/pesdb");
        std::fs::create_dir_all(&dst).map_err(|e| e.to_string())?;
        for e in std::fs::read_dir(&src).map_err(|e| e.to_string())? {
            let e = e.map_err(|e| e.to_string())?;
            let b = std::fs::read(e.path()).map_err(|e| e.to_string())?;
            atomic_write(&dst.join(e.file_name()), &b)?;
        }
        let ch = root.join("common/character0");
        // a world read straight from the game's CPKs has no unpacked character0 (Reddit 25.11.: "character/0 missing"); the
        // game then keeps its own, nothing to copy
        let quelle = welt.join("common/character0");
        if !ch.exists() && quelle.is_dir() { let mut f = vec![]; copy_dir(&quelle, &ch, &mut f, Path::new(""))?; }
        Ok(())
    })();
    if let Err(e) = result {
        let _ = std::fs::remove_dir_all(&root);
        if backup.exists() { let mut f = vec![]; let _ = copy_dir(&backup, &root, &mut f, Path::new("")); }
        return Err(e);
    }
    Ok(())
}
/// sider.ini: backup, career line and Direktstart in, write, read back; on a difference the backup goes back.
pub fn sider_ini_anwenden(karriere: &Path, s: &Path, f: impl Fn(&str) -> Result<String, String>) -> Result<bool, String> {
    if fl27_pid().is_some() { return Err("Bitte FL27 erst beenden.".into()); }
    let p = s.join("sider.ini");
    let raw = std::fs::read(&p).map_err(|e| format!("sider.ini: {e}"))?;
    let text = String::from_utf8_lossy(&raw).to_string();
    let neu = f(&text)?;
    if neu == text { return Ok(false); }
    let _ = karriere;
    let b = s.join(format!("sider.ini.bak-calciom-{}", zeit())); // Übergabe §8.1: backup next to sider.ini
    wache(&b);
    std::fs::write(&b, &raw).map_err(|e| e.to_string())?;
    if let Err(e) = atomic_write(&p, neu.as_bytes()) { let _ = std::fs::write(&p, &raw); return Err(e); }
    Ok(true)
}

/// The three Lua modules carry a small CalcioM addition (§4). Missing addition: if the file is exactly the known
/// original (SHA-256), it is backed up and replaced by the patched copy; a changed module is only reported.
pub const MODULE: [(&str, &str); 3] = [("modules/lib/CommonLib.lua", "CommonLib"), ("modules/common/weather.lua", "weather"), ("modules/common/stadiums.lua", "stadiums")];
pub fn module_pruefen(s: &Path, assets: &Path, karriere: &Path, reparieren: bool) -> Vec<String> {
    let mut out = vec![];
    for (rel, name) in MODULE {
        let p = s.join(rel);
        let Ok(b) = std::fs::read(&p) else { out.push(format!("{rel} fehlt")); continue };
        if crate::fl27::von_uns(&String::from_utf8_lossy(&b)) { continue; }
        let orig = std::fs::read_to_string(assets.join(format!("module/{name}.lua.orig.sha256"))).unwrap_or_default();
        // beta 23.11.: other packs ship a different stadiums.lua; the public release has no copy of the original at all
        // (export.sh): the addition goes in at its anchors either way, the copy is only the last resort
        let angepasst = (name == "stadiums").then(|| stadion_einsetzen(&String::from_utf8_lossy(&b))).flatten(); // roh
        if sha256_hex(&b) != orig.trim() && angepasst.is_none() { out.push(format!("{rel}: Ergänzung fehlt und die Datei weicht ab; die Partie startet normal, nur ohne PES-M-Vorgabe")); continue; }
        if !reparieren { out.push(format!("{rel}: Ergänzung fehlt")); continue; }
        let sich = karriere.join("sicherungen").join(format!("{name}.lua.{}", stamp()));
        let neu = match angepasst { Some(t) => Ok(t.into_bytes()), None => std::fs::read(assets.join(format!("module/{name}.lua"))).map_err(|e| e.to_string()) };
        let ok = std::fs::create_dir_all(sich.parent().unwrap()).and_then(|_| std::fs::write(&sich, &b)).is_ok()
            && neu.and_then(|n| atomic_write(&p, &n)).is_ok();
        out.push(if ok { format!("{rel}: Ergänzung wieder eingesetzt") } else { format!("{rel}: Ergänzung konnte nicht eingesetzt werden") });
    }
    out
}
/// The CalcioM stadium preset put into any stadiums.lua at its anchors: the function before get_new_stadium_path and
/// the exhibition branch (tid 65535) in both places that pick the stadium. None when an anchor is missing.
pub fn stadion_einsetzen(t: &str) -> Option<String> {
    let nl = if t.contains("\r\n") { "\r\n" } else { "\n" };
    let funk = ["-- CalcioM: Stadion-Vorgabe fuer Freundschaftsspiele aus flw\\stadion.txt.", "local function pesm_stadion(ctx)", "    local f = io.open((ctx.sider_dir or \"\") .. \"flw\\\\stadion.txt\", \"r\")", "    if not f then return nil end",
        "    local t = trim(f:read(\"*l\") or \"\")", "    f:close()", "    if t == \"\" then return nil end", "    local s = string.lower(t)", "    for i, row in ipairs(all_stadiums_map) do",
        "        if string.lower(row[2] or \"\") == s or string.lower(row[3] or \"\") == s then", "            return i", "        end", "    end", "    log(\"CalcioM: Stadion-Vorgabe nicht gefunden: \" .. t)", "    return nil", "end", "", ""].join(nl); // roh
    let anker = "local function get_new_stadium_path(ctx)"; // roh
    if t.matches(anker).count() != 1 || !t.contains("trim(") { return None; } // roh
    // the two places: "if tid == 65535 then" followed by the manual-selection line
    let zeilen: Vec<&str> = t.split(nl).collect();
    let orte: Vec<usize> = (0..zeilen.len().saturating_sub(1)).filter(|&i| zeilen[i].trim() == "if tid == 65535 then" && zeilen[i + 1].trim().starts_with("if manual_selection_status == \"On\" and manual_stad_idx then")).collect(); // roh
    if orte.len() != 2 { return None; }
    let mut out: Vec<String> = vec![];
    for (i, z) in zeilen.iter().enumerate() {
        let e = &z[..z.len() - z.trim_start().len()];
        if *z == anker { out.extend(funk.split(nl).map(str::to_string).take(funk.split(nl).count() - 1)); }
        if i == orte[0] {
            let e2 = &zeilen[i + 1][..zeilen[i + 1].len() - zeilen[i + 1].trim_start().len()];
            out.push(z.to_string());
            for x in ["local pesm_idx = pesm_stadion(ctx)", "if pesm_idx then"] { out.push(format!("{e2}{x}")); } // roh
            for x in ["stadium_path = all_stadiums_map[pesm_idx][3]", "custom_name = all_stadiums_map[pesm_idx][2]", "custom_st_id = all_stadiums_map[pesm_idx][1]"] { out.push(format!("{e2}\t{x}")); } // roh
            continue;
        }
        if i == orte[0] + 1 { out.push(z.replacen("if manual_selection_status", "elseif manual_selection_status", 1)); continue; } // roh
        if i == orte[1] {
            out.push(format!("{e}if tid == 65535 and pesm_stadion(ctx) then")); // roh
            for x in ["local pesm_idx = pesm_stadion(ctx)", "custom_st_id = all_stadiums_map[pesm_idx][1]", "custom_name = all_stadiums_map[pesm_idx][2]", "custom_path = all_stadiums_map[pesm_idx][3]", "stadium_switched = true", "log(\"Switching to stadium with ID \" .. custom_st_id .. \" (exhibition mode, CalcioM Vorgabe)\")"] { out.push(format!("{e}\t{x}")); } // roh
            out.push(format!("{e}elseif tid == 65535 then")); // roh
            continue;
        }
        out.push(z.to_string());
    }
    Some(out.join(nl))
}
/// Direktstart stage 1: replacement flow files and the 2-KB intro placeholder in S/livecpk/CalcioM_DirectMatch.
/// Without the templates (they are not in the public repository) the step is skipped: false.
pub fn direktstart_einrichten(s: &Path) -> Result<bool, String> {
    alte_namen_weg(s)?;
    let d = s.join("livecpk").join(DIRECT_ROOT);
    for dst in ["common/script/flow/PreStartup.json", "common/script/flow/Intro/MenuPreTitle.json"] { // roh
        if std::fs::read(d.join(dst)).ok().as_deref() != Some(FLOW_LADEN.as_bytes()) { atomic_write(&d.join(dst), FLOW_LADEN.as_bytes())?; }
    }
    // the module writes its switch (main menu or straight to the side selection) into Edit/Load at every start; Lua
    // cannot make folders, and the renamed root lacked it, so FL27 landed on the main menu (Fabian 26.11.)
    std::fs::create_dir_all(d.join("common/script/flow/Edit/Load")).map_err(|e| e.to_string())?; // roh
    // an old mod's short film replaced the game's intro even on a normal start (fl27-direktstart 26.11.): it goes
    let _ = std::fs::remove_file(d.join("movie/intro/FL2027.usm")); // roh
    Ok(true)
}
/// Stage 1: a flow step that goes straight on to loading the system data (no intro, no title screen).
const FLOW_LADEN: &str = "{\n\t\"events\": {\n\t\t\"proceed\": {\n\t\t\t\"to\": \"Intro/ProcSystemDataLoad\",\n\t\t\t\"go\": \"forward\",\n\t\t\t\"clear\": \"all\",\n\t\t\t\"fadeStart\": \"\",\n\t\t\t\"fadeEnd\": \"do\",\n\t\t\t\"bgType\": \"none\"\n\t\t}\n\t},\n\t\"data\": {\n\t\t\"type\": \"process\",\n\t\t\"bgId\": \"\",\n\t\t\"dialog\": \"\"\n\t},\n\t\"binFile\": {\n\t\t\"files\": []\n\t}\n}\n"; // roh

/// Direktstart stage 2 (fl27-direktstart, Fabian 26.11.): the module calciom_direktmatch.lua and the flow files it puts
/// in only for a direct start (side selection → match day screen without team choice, result menus skipped).
/// The flow files are the game's own, so they are never shipped: they are built here from the originals in the
/// player's Data/dt16_all.cpk with only the targets of FLOW_UMLEITUNGEN changed. False when that is not possible.
pub fn direktmatch_einrichten(s: &Path, assets: &Path) -> Result<bool, String> {
    let Ok(lua) = std::fs::read(assets.join("direktmatch/calciom_direktmatch.lua")) else { return Ok(false) };
    let Some(spiel) = s.parent() else { return Ok(false) };
    let Ok(cpk) = pesm_coaching_zone::cpk::Cpk::open(&spiel.join("Data/dt16_all.cpk")) else { return Ok(false) }; // roh
    let mut neu = vec![];
    for (f, quelle, evs) in FLOW_UMLEITUNGEN {
        let Ok(roh) = cpk.read(&format!("common/script/flow/{quelle}.json")) else { return Ok(false) }; // roh
        let roh = if roh.len() > 16 && &roh[3..8] == b"WESYS" { unpack(&roh)?.1 } else { roh };
        let mut t = String::from_utf8(roh).map_err(|e| format!("{f}: {e}"))?;
        for (ev, to) in evs.iter() { t = flow_umleiten(&t, ev, to).ok_or(format!("{f}: {ev}"))?; }
        neu.push((f, t));
    }
    let ziel = s.join("modules").join("calciom_direktmatch.lua");
    if std::fs::read(&ziel).ok().as_deref() != Some(&lua[..]) { atomic_write(&ziel, &lua)?; }
    // the module takes every file in the folder: old copies of dropped ones must go (replay hang, half-time menu back on 26.11.)
    for alt in ["Match/MatchReplay.json", "Match/MatchPreHalfTime.json"] { let _ = std::fs::remove_file(s.join("content/calciom-direktmatch/flow").join(alt)); } // roh
    for (f, t) in neu {
        let d = s.join("content/calciom-direktmatch/flow").join(format!("{f}.json")); // roh
        if std::fs::read(&d).ok().as_deref() != Some(t.as_bytes()) { atomic_write(&d, t.as_bytes())?; }
    }
    Ok(true)
}
/// GigaPack's fl26match.lua (another mod): its statistics file call starts a 5 s thread that reads a match pointer
/// unchecked and crashes FL27 at half-time or the end in the direct start (fl27-direktstart 26.11.). With direct start
/// on, that one call is commented out and the original kept next to it; true when the file was changed now.
pub fn fl26match_statistik_aus(s: &Path) -> Result<bool, String> {
    let p = s.join("modules/fl26match.lua"); // roh
    let Ok(t) = std::fs::read_to_string(&p) else { return Ok(false) };
    let aktiv = |z: &str| z.contains("\"fl26_match_statsfile\"") && !z.trim_start().starts_with("--"); // roh
    if !t.split_inclusive('\n').any(aktiv) { return Ok(false); }
    let sich = s.join("modules/fl26match.lua.pesm-original"); // roh
    if !sich.exists() { atomic_write(&sich, t.as_bytes())?; }
    let neu: String = t.split_inclusive('\n').map(|z| if aktiv(z) { let e = z.len() - z.trim_start().len(); format!("{}-- {}", &z[..e], &z[e..]) } else { z.to_string() }).collect();
    atomic_write(&p, neu.as_bytes())?;
    Ok(true)
}
/// Direct start off: fl26match.lua back as it was before CalcioM touched it.
pub fn fl26match_wiederherstellen(s: &Path) -> Result<(), String> {
    let sich = s.join("modules/fl26match.lua.pesm-original"); // roh
    let Ok(t) = std::fs::read(&sich) else { return Ok(()) };
    atomic_write(&s.join("modules/fl26match.lua"), &t)?; // roh
    std::fs::remove_file(&sich).map_err(|e| e.to_string())
}
/// The only changes to the game's flow (fl27-direktstart/ablauf/README.md): file written, original it comes from,
/// events and their new target. Schnell/* only count with schnell=1 (quick start: back to the side selection).
pub const FLOW_UMLEITUNGEN: [(&str, &str, &[(&str, &str)]); 6] = [
    ("Exhibition/ExhibiSideSetting", "Exhibition/ExhibiSideSetting", &[("proceed", "Exhibition/ExhibiChangeCompeDesignKind")]), // roh
    ("Match/MatchIdle", "Match/MatchIdle", &[("02_match_result", "Match/MatchPreResult")]), // roh
    ("Match/MatchPreResult", "Match/MatchPreResult", &[("proceed", "Match/MatchEnd")]), // roh
    ("Schnell/MatchEnd", "Match/MatchEnd", &[("01_discontinue", "Exhibition/ExhibiFlowInit"), ("04_postExhibition", "Exhibition/ExhibiFlowInit")]), // roh
    ("Schnell/ExhibiFlowEnd", "Exhibition/ExhibiFlowEnd", &[("proceed", "Exhibition/ExhibiFlowInit")]), // roh
    // without the edit data (Fabian 26.11.: EDIT00000000 overwrote every change of the career database): after the
    // install step straight on, past Edit/Load/EditDataLoadProcess; the module takes it only with edit=0
    ("Ohne/ProcInstall", "Intro/ProcInstall", &[("proceed", "Exhibition/ExhibiFlowInit")]), // roh
];
/// The flow text with the "to" of `events.<ev>` set to `to`, everything else byte for byte as it was; None when the
/// event is missing or the result differs from the original in anything but that target.
pub fn flow_umleiten(t: &str, ev: &str, to: &str) -> Option<String> {
    let e = t.find("\"events\"")?;
    let k = e + t[e..].find(&format!("\"{ev}\""))?;
    let z = k + t[k..].find("\"to\"")?;
    let a = z + 4 + t[z + 4..].find('"')? + 1;
    let b = a + t[a..].find('"')?;
    let neu = format!("{}{to}{}", &t[..a], &t[b..]);
    let mut soll: serde_json::Value = serde_json::from_str(t).ok()?;
    *soll.get_mut("events")?.get_mut(ev)?.get_mut("to")? = to.into(); // roh
    (serde_json::from_str::<serde_json::Value>(&neu).ok()? == soll).then_some(neu)
}
/// The line of flw/direktmatch.txt the module reads at the game's start: "heim gast kit_h kit_a minuten menue" (kits:
/// the kitserver's numbers, trikots.rs) with
/// the rules: vl/em (extra time, penalties), finale=1 or mi=52 (semi-final: the mods' final look and full stands),
/// ast=1 (the players press "Kick-off" themselves; the side selection stays for the controllers anyway).
/// The module marks it "# benutzt" after the start, so the next plain start goes to the main menu.
pub fn direktmatch_zeile(heim: u32, gast: u32, kits: (u8, u8), minuten: u8, vl: bool, em: bool, runde: Option<u8>, schnell: bool) -> String {
    let r = match runde { Some(53) => " finale=1".to_string(), Some(m) => format!(" mi={m}"), None => String::new() }; // roh
    let s = if schnell { " schnell=1" } else { "" }; // roh
    // edit=0: FL27 starts without its edit data, so the career database is what it plays with (Fabian 26.11.)
    format!("{heim} {gast} {} {} {minuten} menue vl={} em={}{r} ast=1 edit=0{s}", kits.0, kits.1, vl as u8, em as u8) // roh
}
/// Quick start: FL27 waits at the side selection, so its window comes to the front once the next match is written.
pub fn fenster_nach_vorne() {
    #[cfg(windows)]
    { let _ = std::process::Command::new("powershell").args(["-NoProfile", "-Command", "(New-Object -ComObject WScript.Shell).AppActivate((Get-Process FL_2027).Id)"]).status(); } // roh
    #[cfg(unix)]
    {
        // Hyprland first, then any X11 desktop; neither there: the player switches by hand
        let hypr = std::process::Command::new("hyprctl").args(["dispatch", "focuswindow", "class:(?i).*fl_2027.*"]).output().is_ok_and(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).contains("No such")); // roh
        if !hypr { let _ = std::process::Command::new("xdotool").args(["search", "--class", "fl_2027", "windowactivate"]).status(); } // roh
    }
}
pub fn direktmatch_schreiben(s: &Path, zeile: &str) -> Result<(), String> { atomic_write(&s.join("flw").join("direktmatch.txt"), format!("{zeile}\r\n").as_bytes()) }

// ---------------------------------------------------------------- process and start (§5)
/// FL_2027.exe by its command line; with several, the one with the most memory (statm field 2).
#[cfg(windows)]
pub fn fl27_pid() -> Option<u32> {
    if testmodus() { return None; }
    tasklist(&format!("IMAGENAME eq {EXE}")).into_iter().next()
}
/// A Windows command without a console window popping up.
#[cfg(windows)]
fn still(c: &mut std::process::Command) -> &mut std::process::Command { use std::os::windows::process::CommandExt; c.creation_flags(0x0800_0000) }
/// Process ids that `tasklist /FI <filter>` lists (CSV: "name","pid",...).
#[cfg(windows)]
fn tasklist(filter: &str) -> Vec<u32> {
    still(std::process::Command::new("tasklist").args(["/FI", filter, "/FO", "CSV", "/NH"])).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().filter_map(|l| l.split("\",\"").nth(1)?.parse().ok()).collect()).unwrap_or_default()
}
#[cfg(windows)]
fn lebt(pid: u32) -> bool { !tasklist(&format!("PID eq {pid}")).is_empty() }
#[cfg(unix)]
pub fn fl27_pid() -> Option<u32> {
    if testmodus() { return ersatz_pid(); }
    let mut best: Option<(u64, u32)> = None;
    for e in std::fs::read_dir("/proc").ok()?.flatten() {
        let Some(pid) = e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else { continue };
        let Ok(cmd) = std::fs::read(format!("/proc/{pid}/cmdline")) else { continue };
        let cmd = String::from_utf8_lossy(&cmd);
        if !cmd.trim_end_matches('\0').ends_with(EXE) { continue; }
        let mem = std::fs::read_to_string(format!("/proc/{pid}/statm")).ok().and_then(|s| s.split_whitespace().nth(1).and_then(|x| x.parse().ok())).unwrap_or(0);
        if best.map(|b| mem > b.0).unwrap_or(true) { best = Some((mem, pid)); }
    }
    best.map(|b| b.1)
}
#[cfg(unix)]
/// Test mode: only the substitute process named in CALCIOM_FL27_ERSATZ_PID; a real FL_2027.exe there aborts.
fn ersatz_pid() -> Option<u32> {
    let pid: u32 = std::env::var("CALCIOM_FL27_ERSATZ_PID").ok()?.parse().ok()?;
    let cmd = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    if String::from_utf8_lossy(&cmd).trim_end_matches('\0').ends_with(EXE) { panic!("Testmodus: PID {pid} ist das echte FL27"); }
    lebt(pid).then_some(pid)
}
/// Running and not a zombie.
#[cfg(unix)]
fn lebt(pid: u32) -> bool { std::fs::read_to_string(format!("/proc/{pid}/stat")).map_or(false, |s| !s.contains(") Z ")) }
/// The Lutris game of FL27: Wine prefix and Wine version from its game configuration.
#[derive(Debug, PartialEq)]
pub struct LutrisSpiel { pub prefix: PathBuf, pub wine: String }
/// Reads `prefix`, `working_dir`, `exe` (section game) and `version` (section wine) of a Lutris game .yml.
pub fn lutris_yml(text: &str) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let (mut teil, mut out) = (String::new(), (None, None, None, None));
    for z in text.lines() {
        if !z.starts_with(' ') && z.trim_end().ends_with(':') { teil = z.trim_end().trim_end_matches(':').to_string(); continue; }
        let Some(r) = z.strip_prefix("  ").filter(|r| !r.starts_with(' ')) else { continue };
        let Some((k, v)) = r.split_once(':') else { continue };
        let v = Some(v.trim().trim_matches('\'').trim_matches('"').to_string());
        match (teil.as_str(), k.trim()) { ("game", "prefix") => out.0 = v, ("game", "working_dir") => out.1 = v, ("game", "exe") => out.2 = v, ("wine", "version") => out.3 = v, _ => {} }
    }
    out
}
/// The Lutris configuration of FL27 (game LUTRIS_ID).
/// shortcut: found by the game folder among ~/.local/share/lutris/games/*.yml, not through pga.db (no SQLite reader); read pga.db once one exists.
pub fn lutris_spiel() -> Result<LutrisSpiel, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = PathBuf::from(home).join(".local/share/lutris/games");
    let spiel = spielordner();
    for e in std::fs::read_dir(&dir).map_err(|e| format!("Lutris-Spiele: {e}"))?.flatten() {
        let Ok(t) = std::fs::read_to_string(e.path()) else { continue };
        let (prefix, wd, exe, wine) = lutris_yml(&t);
        let passt = wd.as_deref().map(|w| Path::new(w) == spiel).unwrap_or(false) || exe.as_deref().map(|x| Path::new(x).starts_with(&spiel)).unwrap_or(false);
        if let (true, Some(p)) = (passt, prefix) { return Ok(LutrisSpiel { prefix: p.into(), wine: wine.unwrap_or_else(|| "system".into()) }); }
    }
    Err("Die Lutris-Konfiguration von FL27 ist nicht zu finden.".into())
}
/// Does a process environment (/proc/<pid>/environ) run in this Wine prefix?
pub fn im_prefix(environ: &[u8], prefix: &Path) -> bool {
    environ.split(|b| *b == 0).filter_map(|e| std::str::from_utf8(e).ok()).filter_map(|e| e.strip_prefix("WINEPREFIX=")).any(|p| {
        let (a, b) = (Path::new(p.trim_end_matches('/')), prefix);
        a == b || a.canonicalize().ok().zip(b.canonicalize().ok()).map_or(false, |(x, y)| x == y)
    })
}
/// End FL27: only FL_2027.exe (command line) in the game's prefix. SIGTERM, up to 10 s; else SIGKILL and
/// `wineserver -k` in that prefix so no Wine remains hang. Lutris and other programs stay open; nothing is written.
#[cfg(windows)]
pub fn fl27_beenden() -> Result<String, String> {
    let pid = fl27_pid().ok_or("FL27 läuft nicht.")?;
    let kill = |hart: bool| { let mut c = std::process::Command::new("taskkill"); c.args(["/PID", &pid.to_string()]); if hart { c.arg("/F"); } still(&mut c).status().map(|_| ()).map_err(|e| e.to_string()) };
    kill(false)?;
    for _ in 0..40 { if !lebt(pid) { return Ok("FL27 beendet.".into()); } std::thread::sleep(std::time::Duration::from_millis(250)); }
    kill(true)?;
    Ok("FL27 beendet (hart).".into())
}
#[cfg(unix)]
pub fn fl27_beenden() -> Result<String, String> {
    // test mode: only the substitute process, no prefix and no wineserver
    let lu = if testmodus() { None } else { Some(lutris_spiel()?) };
    let pid = fl27_pid().ok_or("FL27 läuft nicht.")?;
    if let Some(lu) = &lu {
        let env = std::fs::read(format!("/proc/{pid}/environ")).map_err(|e| format!("Prozess {pid}: {e}"))?;
        if !im_prefix(&env, &lu.prefix) { return Err(format!("FL_2027.exe läuft nicht im Prefix {}; CalcioM beendet es nicht.", lu.prefix.display())); }
    }
    let lebt = || lebt(pid);
    let signal = |s: &str| std::process::Command::new("kill").args([s, &pid.to_string()]).status().map(|_| ()).map_err(|e| e.to_string());
    signal("-TERM")?;
    for _ in 0..40 { if !lebt() { return Ok(if lu.is_none() { "Ersatzprozess beendet (Testmodus)." } else { "FL27 beendet." }.into()); } std::thread::sleep(std::time::Duration::from_millis(250)); }
    signal("-KILL")?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    let Some(lu) = lu else { return Ok("Ersatzprozess beendet (Testmodus).".into()) };
    let ws = if lu.wine == "system" { PathBuf::from("wineserver") } else {
        let p = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share/lutris/runners/wine").join(&lu.wine).join("bin/wineserver");
        if p.exists() { p } else { PathBuf::from("wineserver") }
    };
    let _ = std::process::Command::new(ws).arg("-k").env("WINEPREFIX", &lu.prefix).status();
    Ok("FL27 beendet (hart, Wine-Reste im Prefix aufgeräumt).".into())
}
/// Starts FL27 with sider: on Windows the pack's launcher, on Linux through Lutris.
pub fn spiel_starten() -> Result<&'static str, String> {
    if testmodus() { return Err("Testmodus: FL27 wird nicht gestartet.".into()); }
    #[cfg(windows)]
    {
        let g = spielordner();
        // beta 23.11.: the pack's launcher asks for admin rights (error 740, CreateProcess cannot elevate); then the
        // start goes through Windows' own elevation (UAC prompt) with PowerShell Start-Process -Verb RunAs
        let los = |p: PathBuf| {
            let dir = p.parent().unwrap_or(&g).to_path_buf();
            match still(std::process::Command::new(&p).current_dir(&dir)).spawn() {
                Ok(_) => Ok(()),
                Err(e) if e.raw_os_error() == Some(740) => {
                    log(&format!("{} braucht Administratorrechte, Start mit Erhöhung", p.display()));
                    let q = |x: &Path| x.display().to_string().replace('\'', "''");
                    let st = still(std::process::Command::new("powershell").args(["-NoProfile", "-Command", &format!("Start-Process -FilePath '{}' -WorkingDirectory '{}' -Verb RunAs", q(&p), q(&dir))])).status(); // roh
                    if st.map(|s| s.success()).unwrap_or(false) { Ok(()) } else { Err(format!("{}: {e}", p.display())) }
                }
                Err(e) => Err(format!("{}: {e}", p.display())),
            }
        };
        // the launcher brings up sider and the game together; if it fails, sider and the game directly
        let start = g.join("FL 2027 start.exe");
        if start.is_file() {
            match los(start) { Ok(()) => return Ok("FL27 startet über FL 2027 start.exe …"), Err(e) => log(&format!("Startprogramm: {e}, versuche sider.exe")) }
        }
        // else sider first, the game shortly after (sider.ini has no start.game, so sider alone would end without the game)
        let sider_exe = [sider().join("sider.exe"), g.join("sider.exe")].into_iter().find(|p| p.is_file()).ok_or("sider.exe nicht gefunden")?;
        los(sider_exe)?;
        let exe = [g.join(EXE), sider().join(EXE)].into_iter().find(|p| p.is_file()).ok_or(format!("{EXE} nicht gefunden"))?;
        std::thread::sleep(std::time::Duration::from_millis(1500));
        los(exe)?;
        return Ok("FL27 startet über sider.exe …");
    }
    #[allow(unreachable_code)]
    lutris_starten().map(|_| "FL27 startet über Lutris …")
}
pub fn lutris_starten() -> Result<(), String> {
    if testmodus() { return Err("Testmodus: FL27 wird nicht gestartet.".into()); }
    std::process::Command::new("setsid").args(["lutris", &format!("lutris:rungameid/{LUTRIS_ID}")]).current_dir(spielordner())
        .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().map(|_| ()).map_err(|e| format!("Lutris lässt sich nicht starten: {e}"))
}

/// The steps of "Match starten", each one stops with a clear text (§5). Implemented by the career; faked in tests.
pub trait Ablauf {
    fn erlaubt(&self) -> bool;
    fn bestaetigt(&self) -> bool;
    fn speichern(&mut self) -> Result<(), String>;
    fn spiel_laeuft(&self) -> bool;
    fn sicherung(&mut self) -> Result<(), String>;
    fn vorbereiten(&mut self) -> Result<(), String>;
    fn laden(&mut self) -> Result<(), String>;
    fn konfiguration(&mut self) -> Result<(), String>;
    fn vorgaben(&mut self) -> Result<(), String>;
    fn start(&mut self) -> Result<(), String>;
    /// FL27 is open: can it take the career's changes without a restart (LivePatch module on, this career active, no new players)?
    fn live_bereit(&mut self) -> Result<(), String>;
    /// FL27 is open: the match teams as patch files, the person presses F7 and starts the match in the game.
    fn live_patch(&mut self) -> Result<(), String>;
}
pub fn match_starten(a: &mut dyn Ablauf) -> Result<(), String> {
    if !a.erlaubt() { return Err("Der Spielstart ist ausgeschaltet (Einstellungen › Spielstart erlaubt).".into()); }
    if !a.bestaetigt() { return Err("Bitte zuerst bestätigen, dass die Karriere in den Spielordner geladen wird.".into()); }
    a.speichern()?;
    // FL27 stays open between matches (Fabian 23.11.): database and patches for the running game, no restart
    if a.spiel_laeuft() {
        a.live_bereit()?;
        a.vorbereiten()?;
        a.laden()?;
        a.vorgaben()?;
        return a.live_patch();
    }
    a.sicherung()?;
    a.vorbereiten()?;
    a.laden()?;
    a.konfiguration()?;
    a.vorgaben()?;
    a.start()
}

// ---------------------------------------------------------------- memory reader (§1)
pub const REC: usize = 0x2134;
pub const SEITE: usize = 0x59650;
pub const PLAETZE: usize = 40;
/// FL_2027.exe FUN_1408ca920 (0x1408ca920): kinds 0..=0xb0 (`kind < 0xb1`), value at record + 8 + kind·0x24.
pub const ARTEN: usize = 0xb1;
/// FL_2027.exe FUN_1408ca920 mode 5 (the total the stats getters use) adds sections 0..7; section 8 stays out
/// (modes 0/1 = sections 0-2 / 3-5 are the halves, modes 2/3/4 = sections 6/7/8 alone).
pub const ABSCHNITTE_SUMME: usize = 8;
pub const KOPF: [u32; 2] = [65533, 65534];
pub const STARTELF: u8 = 11;
pub const GOAL: u8 = 0; pub const OWNGOAL: u8 = 1; pub const ASSIST: u8 = 4; pub const SHOOT: u8 = 6; pub const SHOOT_INFRAME: u8 = 17;
pub const SHOOT_PK: u8 = 21; pub const PASS: u8 = 22; pub const PASS_OK: u8 = 23; pub const CENTERING: u8 = 30; pub const TACKLE_OK: u8 = 50;
pub const FOUL: u8 = 55; pub const OFFSIDE: u8 = 56; pub const CARD_Y: u8 = 57; pub const CARD_R: u8 = 58; pub const FK: u8 = 61; pub const CK: u8 = 63;
pub const BALLTOUCH: u8 = 67; pub const CUTPASS: u8 = 78; pub const GKSAVE: u8 = 133; pub const LAUF: u8 = 173;
pub const STAT: [(&str, u8); 14] = [("schuesse", SHOOT), ("schuesse_aufs_tor", SHOOT_INFRAME), ("paesse", PASS), ("paesse_erfolgreich", PASS_OK),
    ("flanken", CENTERING), ("fouls", FOUL), ("abseits", OFFSIDE), ("freistoesse", FK), ("ecken", CK), ("paraden", GKSAVE),
    ("abgefangene_paesse", CUTPASS), ("zweikaempfe_gewonnen", TACKLE_OK), ("gelb", CARD_Y), ("rot", CARD_R)];
const GB: u64 = 1 << 30;

/// Read-only access to the game's memory: writable regions and bytes. Exchangeable so the tests run without the game.
pub trait Mem: Send {
    fn regions(&mut self) -> Vec<(u64, u64)>;
    fn read(&mut self, adr: u64, n: usize) -> Option<Vec<u8>>;
    fn alive(&mut self) -> bool;
    /// The helper searches itself (one round trip instead of gigabytes over the pipe).
    fn bloecke(&mut self) -> Option<Vec<u64>> { None }
}
#[cfg(unix)]
pub struct ProcMem { pub pid: u32, file: std::fs::File }
#[cfg(unix)]
impl ProcMem {
    pub fn open(pid: u32) -> std::io::Result<ProcMem> { Ok(ProcMem { pid, file: std::fs::File::open(format!("/proc/{pid}/mem"))? }) }
}
#[cfg(unix)]
impl Mem for ProcMem {
    fn regions(&mut self) -> Vec<(u64, u64)> {
        std::fs::read_to_string(format!("/proc/{}/maps", self.pid)).unwrap_or_default().lines().filter_map(|l| {
            let mut it = l.split_whitespace();
            let (range, perm) = (it.next()?, it.next()?);
            if !perm.starts_with("rw") { return None; }
            let (a, b) = range.split_once('-')?;
            Some((u64::from_str_radix(a, 16).ok()?, u64::from_str_radix(b, 16).ok()?))
        }).collect()
    }
    fn read(&mut self, adr: u64, n: usize) -> Option<Vec<u8>> {
        let mut b = vec![0u8; n];
        self.file.read_exact_at(&mut b, adr).ok().map(|_| b)
    }
    fn alive(&mut self) -> bool { Path::new(&format!("/proc/{}", self.pid)).exists() }
}

fn u32_at(b: &[u8], i: usize) -> u32 { u32::from_le_bytes(b[i..i + 4].try_into().unwrap()) }

/// Our memory access seen as the Coaching Zone's (it reads through `&self`); regions over 1 GB stay out as before.
struct CzMem<'a>(std::cell::RefCell<&'a mut dyn Mem>);
impl cz::process::Memory for CzMem<'_> {
    fn read(&self, addr: u64, len: usize) -> Option<Vec<u8>> { self.0.borrow_mut().read(addr, len) }
    fn regions(&self) -> Vec<cz::process::Region> {
        self.0.borrow_mut().regions().into_iter().filter(|(lo, hi)| hi - lo <= GB).map(|(lo, hi)| cz::process::Region { start: lo, len: hi - lo }).collect()
    }
}
/// All statistics blocks (address of the first home record), found by the Coaching Zone: three records in a row with the
/// home head, the away head one side further, a plausible player id; the head's upper 16 bits may vary between matches.
pub fn bloecke(m: &mut dyn Mem) -> Vec<u64> {
    if let Some(b) = m.bloecke() { return b; }
    cz::stats::find_blocks(&CzMem(std::cell::RefCell::new(m)))
}
/// One reading of the block at `basis` through the Coaching Zone (both sides, 40 places each).
pub fn schnappschuss(m: &mut dyn Mem, basis: u64) -> Option<cz::stats::Snapshot> { cz::stats::Snapshot::read(&CzMem(std::cell::RefCell::new(m)), basis) }
/// The reading in CalcioM's form: per place the kinds with any value. The Coaching Zone reads kinds 0..0xAF; kind 0xB0
/// (the game knows it, `kind < 0xb1`) is used nowhere and stays out (wish in PAKETE.md).
pub fn stand(s: &cz::stats::Snapshot) -> Stand {
    let seite = |side: usize| s.sides[side].iter().enumerate().filter_map(|(pl, x)| x.as_ref().map(|x| Spieler {
        platz: pl as u8, pid: x.id,
        w: (0..cz::stats::KINDS).filter_map(|k| { let v: [u32; 9] = std::array::from_fn(|ab| x.get(k, ab)); v.iter().any(|y| *y > 0).then_some((k as u8, v)) }).collect(),
    })).collect();
    Stand { s: [seite(0), seite(1)] }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spieler { pub platz: u8, pub pid: u32, pub w: Vec<(u8, [u32; 9])> }
impl Spieler {
    pub fn art(&self, k: u8) -> [u32; 9] { self.w.iter().find(|x| x.0 == k).map(|x| x.1).unwrap_or([0; 9]) }
    pub fn sum(&self, k: u8) -> u32 { self.art(k)[..ABSCHNITTE_SUMME].iter().sum() }
    pub fn im_spiel(&self) -> bool { !self.w.is_empty() }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Stand { pub s: [Vec<Spieler>; 2] }

pub fn lesen(m: &mut dyn Mem, basis: u64) -> Option<Stand> { schnappschuss(m, basis).map(|s| stand(&s)) }
pub fn summe(sp: &[Spieler], k: u8) -> u32 { sp.iter().map(|x| x.sum(k)).sum() }
pub fn kennung(st: &Stand) -> (Vec<u32>, Vec<u32>) {
    let f = |v: &Vec<Spieler>| v.iter().filter(|x| x.platz < STARTELF).map(|x| x.pid).collect();
    (f(&st.s[0]), f(&st.s[1]))
}
pub fn umfang(st: &Stand) -> u32 { st.s.iter().map(|v| summe(v, PASS) + summe(v, BALLTOUCH)).sum() }
/// Goals (incl. free kicks and penalties) plus the opponent's own goals.
pub fn endstand(st: &Stand) -> (u32, u32) {
    (summe(&st.s[0], GOAL) + summe(&st.s[1], OWNGOAL), summe(&st.s[1], GOAL) + summe(&st.s[0], OWNGOAL))
}
/// Simple xG: penalty 0.76, on target 0.12, off target 0.04. shortcut: no shot location, upgrade when positions are found.
pub fn xg(sp: &[Spieler]) -> f32 {
    let pk = summe(sp, SHOOT_PK) as f32;
    let auf = (summe(sp, SHOOT_INFRAME) as f32 - pk).max(0.0);
    let neben = (summe(sp, SHOOT) as f32 - auf - pk).max(0.0);
    ((pk * 0.76 + auf * 0.12 + neben * 0.04) * 100.0).round() / 100.0
}
/// (minute, stoppage) from the section (0 = 0-15 … 5 = 75-90, 6 = stoppage); share 0..1 inside the section, else the middle.
pub fn minute(ab: u8, anteil: Option<f64>) -> (u8, u8) {
    let a = anteil.unwrap_or(0.5).clamp(0.0, 0.99);
    match ab {
        0..=5 => (15 * ab + (15.0 * a) as u8 + 1, 0),
        6 => (90, 1 + (4.0 * a) as u8),
        _ => (90 + 15 * (ab - 7) + (15.0 * a) as u8 + 1, 0),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvArt { Tor { vorlage: Option<u32>, eigentor: bool }, Karte { rot: bool }, Wechsel { raus: Option<u32> } }
#[derive(Clone, Debug, PartialEq)]
pub struct Ev { pub min: u8, pub plus: u8, pub seite: u8, pub pid: u32, pub art: EvArt, pub konfidenz: u8, pub unsicher: bool }
#[derive(Clone, Debug)]
pub struct SpielDoc { pub spiel_id: String, pub home: u32, pub away: u32, pub hg: u32, pub ag: u32, pub ev: Vec<Ev>, pub st: Stand, pub xg: [f32; 2] }
impl SpielDoc {
    pub fn team(&self, k: u8) -> [u32; 2] { [summe(&self.st.s[0], k), summe(&self.st.s[1], k)] }
    /// The match file (schema 1 of pesm/fl/beobachter/schema.json, reduced to what the reader knows).
    pub fn json(&self) -> serde_json::Value {
        use serde_json::json;
        let ev: Vec<serde_json::Value> = self.ev.iter().enumerate().map(|(i, e)| {
            let mut v = json!({"seq": i + 1, "quelle": "speicher", "min": e.min, "plus": e.plus, "seite": if e.seite == 0 { "h" } else { "a" }, "konfidenz": e.konfidenz, "unsicher": e.unsicher});
            match &e.art {
                EvArt::Tor { vorlage, eigentor } => { v["typ"] = "tor".into(); v["pid"] = e.pid.into(); v["vorlage"] = json!(vorlage); v["eigentor"] = (*eigentor).into(); }
                EvArt::Karte { rot } => { v["typ"] = "karte".into(); v["pid"] = e.pid.into(); v["farbe"] = if *rot { "rot" } else { "gelb" }.into(); }
                EvArt::Wechsel { raus } => { v["typ"] = "wechsel".into(); v["rein"] = e.pid.into(); v["raus"] = json!(raus); }
            }
            v
        }).collect();
        let mut werte = serde_json::Map::new();
        for (n, k) in STAT { werte.insert(n.into(), json!(self.team(k))); }
        werte.insert("xg".into(), json!(self.xg));
        let mut spieler = serde_json::Map::new();
        for (side, v) in self.st.s.iter().enumerate() {
            for x in v {
                let mut d = json!({"seite": if side == 0 { "h" } else { "a" }, "platz": x.platz, "im_spiel": x.im_spiel(), "tore": x.sum(GOAL), "vorlagen": x.sum(ASSIST), "eigentore": x.sum(OWNGOAL)});
                for (n, k) in STAT { if x.sum(k) > 0 { d[n] = x.sum(k).into(); } }
                spieler.insert(x.pid.to_string(), d);
            }
        }
        json!({"schema": 1, "spiel_id": self.spiel_id, "quelle": "fl27", "status": "beendet",
            "meta": {"home": self.home, "away": self.away, "modul": "speicher"},
            "endstand": {"home": self.hg, "away": self.ag, "quelle": "speicher", "konfidenz": 100},
            "aufstellung": {"home": kennung(&self.st).0, "away": kennung(&self.st).1, "quelle": "speicher"},
            "ereignisse": ev, "statistik": {"quelle": "speicher", "werte": werte, "spieler": spieler, "warn": ["Ballbesitz fehlt (nicht im Speicherblock)", "xG einfach (ohne Schussort)"]}})
    }
}

type Key = (u8, u32, u8, u8, u32);
/// Events from a state; anteile[(side, pid, kind, section, n)] = share inside the section (from following the match)
/// makes the minute exact; subs from following the match, without it the subs appear without "raus".
pub fn doc(st: &Stand, spiel_id: String, home: u32, away: u32, anteile: &HashMap<Key, f64>, wechsel: Option<&Vec<(u8, Option<u32>, u32, u8)>>) -> SpielDoc {
    let vorkommen = |side: u8, art: u8| -> Vec<(u8, Option<f64>, u32, u32)> {
        let mut out = vec![];
        for x in &st.s[side as usize] {
            for (ab, anz) in x.art(art).iter().enumerate().take(ABSCHNITTE_SUMME) {
                for n in 0..*anz { out.push((ab as u8, anteile.get(&(side, x.pid, art, ab as u8, n)).copied(), x.pid, n)); }
            }
        }
        out.sort_by(|a, b| (a.0, a.1.unwrap_or(0.5)).partial_cmp(&(b.0, b.1.unwrap_or(0.5))).unwrap());
        out
    };
    let mut liste: Vec<(u8, Option<f64>, u8, u32, EvArt)> = vec![];
    for side in 0..2u8 {
        let gegner = 1 - side;
        let mut vorl = vorkommen(side, ASSIST);
        for (ab, a, pid, _) in vorkommen(side, GOAL) {
            let v = vorl.iter().position(|x| x.0 == ab).map(|i| vorl.remove(i).2);
            liste.push((ab, a, side, pid, EvArt::Tor { vorlage: v, eigentor: false }));
        }
        for (ab, a, pid, _) in vorkommen(gegner, OWNGOAL) { liste.push((ab, a, side, pid, EvArt::Tor { vorlage: None, eigentor: true })); }
        let rote: HashSet<u32> = vorkommen(side, CARD_R).into_iter().map(|x| x.2).collect();
        let mut gelbe: HashMap<u32, u32> = HashMap::new();
        for (ab, a, pid, _) in vorkommen(side, CARD_Y) {
            let n = gelbe.entry(pid).or_default();
            *n += 1;
            if !(*n == 2 && rote.contains(&pid)) { liste.push((ab, a, side, pid, EvArt::Karte { rot: false })); } // yellow-red: the 2nd yellow is the red
        }
        for (ab, a, pid, _) in vorkommen(side, CARD_R) { liste.push((ab, a, side, pid, EvArt::Karte { rot: true })); }
    }
    let auto: Vec<(u8, Option<u32>, u32, u8)> = (0..2u8).flat_map(|s| st.s[s as usize].iter().filter(|x| x.platz >= STARTELF && x.im_spiel())
        .map(move |x| (s, None, x.pid, x.w.iter().filter_map(|(_, v)| v.iter().position(|y| *y > 0)).min().unwrap_or(0) as u8))).collect();
    for (s, raus, rein, ab) in wechsel.cloned().unwrap_or(auto) { liste.push((ab, None, s, rein, EvArt::Wechsel { raus })); }
    liste.sort_by(|a, b| (a.0, a.1.unwrap_or(0.5)).partial_cmp(&(b.0, b.1.unwrap_or(0.5))).unwrap());
    let ev = liste.into_iter().map(|(ab, a, side, pid, art)| {
        let (min, plus) = minute(ab, a);
        let konfidenz = match &art { EvArt::Wechsel { raus } => if raus.is_some() { 80 } else { 50 }, _ => if a.is_some() { 100 } else { 60 } };
        Ev { min, plus, seite: side, pid, art, konfidenz, unsicher: a.is_none() }
    }).collect();
    let (hg, ag) = endstand(st);
    SpielDoc { spiel_id, home, away, hg, ag, ev, xg: [xg(&st.s[0]), xg(&st.s[1])], st: st.clone() }
}

/// Follows the running match in a 2-second beat (§1.4).
pub struct Mitlauf {
    finder: Box<dyn FnMut() -> Result<Option<Box<dyn Mem>>, String> + Send>,
    team_fn: Box<dyn Fn(&[u32]) -> u32 + Send>,
    m: Option<Box<dyn Mem>>,
    pub basis: Option<u64>,
    pub st: Option<Stand>,
    kenn: Option<(Vec<u32>, Vec<u32>)>,
    pub fehler: Option<String>,
    suche_ab: f64,
    start: Option<f64>,
    gesehen: HashMap<Key, f64>,
    ab_start: HashMap<u8, f64>,
    wechsel: Vec<(u8, Option<u32>, u32, u8)>,
    /// The Coaching Zone's live tracker: events, minute, grades and places for the live view.
    pub tracker: cz::live::Tracker,
    /// FL27's process ended since the last beat: a match that ends now was broken off, not whistled (Discord 26.11.).
    pub abgebrochen: bool,
}
pub const SUCHE_S: f64 = 30.0;
impl Mitlauf {
    pub fn new(finder: Box<dyn FnMut() -> Result<Option<Box<dyn Mem>>, String> + Send>, team_fn: Box<dyn Fn(&[u32]) -> u32 + Send>) -> Mitlauf {
        Mitlauf { finder, team_fn, m: None, basis: None, st: None, kenn: None, fehler: None, suche_ab: 0.0, start: None, gesehen: HashMap::new(), ab_start: HashMap::new(), wechsel: vec![], tracker: cz::live::Tracker::new(), abgebrochen: false }
    }
    fn neu(&mut self) { self.start = None; self.gesehen.clear(); self.ab_start.clear(); self.wechsel.clear(); }
    fn lesen(&mut self, jetzt: f64) -> Option<cz::stats::Snapshot> {
        if self.m.as_mut().is_some_and(|m| !m.alive()) { self.abgebrochen = true; }
        if self.m.as_mut().map(|m| !m.alive()).unwrap_or(true) {
            self.basis = None;
            match (self.finder)() { Ok(m) => { self.m = m; self.fehler = None; } Err(e) => { self.m = None; self.fehler = Some(e); } }
        }
        let m = self.m.as_mut()?;
        if let Some(b) = self.basis { if let Some(s) = schnappschuss(m.as_mut(), b) { return Some(s); } }
        if jetzt < self.suche_ab { return None; }
        self.suche_ab = jetzt + SUCHE_S; // at most one search (about 4 s of work) every 30 s
        let bl = bloecke(m.as_mut());
        if bl.is_empty() { self.basis = None; return None; }
        let mut pick = bl[0];
        if let (Some(k), Some(alt)) = (&self.kenn, &self.st) {
            for b in &bl { if let Some(s) = lesen(m.as_mut(), *b) { if &kennung(&s) == k && umfang(&s) >= umfang(alt) { pick = *b; break; } } }
        }
        self.basis = Some(pick);
        schnappschuss(m.as_mut(), pick)
    }
    /// The live view of the match being followed (score, minute, players with grades, events), from the Coaching Zone.
    pub fn ansicht(&self) -> Option<cz::live::MatchView> { self.st.as_ref()?; self.tracker.view(None) }
    pub fn live(&self) -> Option<(u32, u32)> { self.st.as_ref().map(endstand) }
    pub fn takt(&mut self, jetzt: f64) -> Option<SpielDoc> {
        let snap = self.lesen(jetzt);
        let Some(st) = snap.as_ref().map(stand) else {
            let fertig = self.st.is_some().then(|| self.abschluss(jetzt));
            self.st = None;
            self.kenn = None;
            self.tracker = cz::live::Tracker::new();
            return fertig;
        };
        if let Some(s) = snap { self.tracker.update(s, std::time::Instant::now()); }
        let mut fertig = None;
        if let Some(alt) = &self.st {
            if Some(kennung(&st)) != self.kenn || umfang(&st) < umfang(alt) { fertig = Some(self.abschluss(jetzt)); self.st = None; }
        }
        if self.st.is_none() {
            self.neu();
            self.kenn = Some(kennung(&st));
            self.start = (umfang(&st) == 0).then_some(jetzt); // joined mid-match: minutes only from the sections
        }
        let alt = self.st.take();
        self.vergleichen(alt.as_ref(), &st, jetzt);
        self.st = Some(st);
        fertig
    }
    fn vergleichen(&mut self, alt: Option<&Stand>, neu: &Stand, jetzt: f64) {
        for side in 0..2u8 {
            for x in &neu.s[side as usize] {
                let x0 = alt.and_then(|a| a.s[side as usize].iter().find(|y| y.pid == x.pid));
                for (art, v) in &x.w {
                    let v0 = x0.map(|y| y.art(*art)).unwrap_or([0; 9]);
                    for ab in 0..9 {
                        if v[ab] > 0 { self.ab_start.entry(ab as u8).or_insert(jetzt); }
                        for n in v0[ab]..v[ab] { self.gesehen.entry((side, x.pid, *art, ab as u8, n)).or_insert(jetzt); }
                    }
                }
                if let Some(a) = alt { if x.platz >= STARTELF && x.im_spiel() && x0.map(|y| !y.im_spiel()).unwrap_or(true) { self.eingewechselt(side, x, a, neu); } }
            }
        }
    }
    /// out = the only player of the side whose run value (kind 173) stood still since the last beat, else unknown.
    fn eingewechselt(&mut self, side: u8, x: &Spieler, alt: &Stand, neu: &Stand) {
        let vorher: HashMap<u32, u32> = alt.s[side as usize].iter().map(|y| (y.pid, y.sum(LAUF))).collect();
        let weg: HashSet<u32> = self.wechsel.iter().filter(|w| w.0 == side).filter_map(|w| w.1).chain(neu.s[side as usize].iter().filter(|y| y.sum(CARD_R) > 0).map(|y| y.pid)).collect();
        let steht: Vec<u32> = neu.s[side as usize].iter().filter(|y| y.im_spiel() && y.pid != x.pid && !weg.contains(&y.pid) && vorher.get(&y.pid) == Some(&y.sum(LAUF))).map(|y| y.pid).collect();
        let ab = x.w.iter().filter_map(|(_, v)| v.iter().position(|y| *y > 0)).min().unwrap_or(0) as u8;
        self.wechsel.push((side, if steht.len() == 1 { Some(steht[0]) } else { None }, x.pid, ab));
    }
    fn anteile(&self) -> HashMap<Key, f64> {
        if self.start.is_none() { return HashMap::new(); }
        let mut anf: Vec<(u8, f64)> = self.ab_start.iter().map(|(a, t)| (*a, *t)).collect();
        anf.sort_by(|a, b| a.0.cmp(&b.0));
        let mut l: Vec<f64> = anf.windows(2).filter(|w| w[1].0 == w[0].0 + 1 && w[1].1 > w[0].1).map(|w| w[1].1 - w[0].1).collect();
        if l.is_empty() { return HashMap::new(); }
        l.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let dauer = l[l.len() / 2];
        self.gesehen.iter().filter_map(|(k, t)| self.ab_start.get(&k.3).map(|s| (*k, (t - s) / dauer))).collect()
    }
    /// The match read so far as finished (no match read yet: nothing).
    pub fn jetzt_fertig(&mut self, jetzt: f64) -> Option<SpielDoc> { self.st.is_some().then(|| self.abschluss(jetzt)) }
    fn abschluss(&mut self, jetzt: f64) -> SpielDoc {
        let st = self.st.clone().unwrap();
        let home = (self.team_fn)(&st.s[0].iter().map(|x| x.pid).collect::<Vec<_>>());
        let away = (self.team_fn)(&st.s[1].iter().map(|x| x.pid).collect::<Vec<_>>());
        let id = format!("{}-{home}-{away}", (self.start.unwrap_or(jetzt) as u64));
        let w = self.wechsel.clone();
        doc(&st, id, home, away, &self.anteile(), Some(&w))
    }
}

// ---------------------------------------------------------------- helper process (read-only, cap_sys_ptrace)
/// Talks to the small helper binary (calciom-speicher, `setcap cap_sys_ptrace+ep`) over JSON lines: it can only
/// search blocks and read bytes, never write. CalcioM itself stays unprivileged.
pub struct DienstMem { pid: u32, child: std::process::Child, inp: std::process::ChildStdin, out: BufReader<std::process::ChildStdout> }
/// The local clock as HH:MM (no time-zone crate: the system's own command).
pub fn uhrzeit() -> String {
    #[cfg(windows)]
    let o = still(std::process::Command::new("cmd").args(["/C", "time", "/t"])).output();
    #[cfg(not(windows))]
    let o = std::process::Command::new("date").arg("+%H:%M").output();
    o.ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}
pub fn helfer_pfad() -> PathBuf { std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("calciom-speicher"))).unwrap_or_else(|| PathBuf::from("calciom-speicher")) }
impl DienstMem {
    pub fn start(pid: u32) -> Result<DienstMem, String> {
        let p = helfer_pfad();
        if !p.exists() { return Err(format!("Speicher-Helfer fehlt ({})", p.display())); }
        let mut child = std::process::Command::new(&p).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null())
            .spawn().map_err(|e| format!("Speicher-Helfer: {e}"))?;
        let inp = child.stdin.take().unwrap();
        let out = BufReader::new(child.stdout.take().unwrap());
        let mut d = DienstMem { pid, child, inp, out };
        let r = d.frage(serde_json::json!({"op": "offen", "pid": pid}))?;
        if r["ok"] != true { return Err(r["fehler"].as_str().unwrap_or("Speicherzugriff nicht freigeschaltet").to_string()); }
        Ok(d)
    }
    fn frage(&mut self, q: serde_json::Value) -> Result<serde_json::Value, String> {
        writeln!(self.inp, "{q}").map_err(|e| e.to_string())?;
        let mut line = String::new();
        self.out.read_line(&mut line).map_err(|e| e.to_string())?;
        serde_json::from_str(&line).map_err(|e| e.to_string())
    }
}
impl Drop for DienstMem { fn drop(&mut self) { let _ = self.child.kill(); let _ = self.child.wait(); } }
impl Mem for DienstMem {
    fn regions(&mut self) -> Vec<(u64, u64)> { vec![] }
    fn read(&mut self, adr: u64, n: usize) -> Option<Vec<u8>> {
        let r = self.frage(serde_json::json!({"op": "lesen", "pid": self.pid, "adresse": adr, "laenge": n})).ok()?;
        let h = r["daten"].as_str()?;
        (0..h.len() / 2).map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).ok()).collect()
    }
    fn alive(&mut self) -> bool { Path::new(&format!("/proc/{}", self.pid)).exists() }
    fn bloecke(&mut self) -> Option<Vec<u64>> {
        let r = self.frage(serde_json::json!({"op": "bloecke", "pid": self.pid})).ok()?;
        r["bloecke"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64()).collect())
    }
}
/// Windows reads FL27's memory directly through the Coaching Zone (OpenProcess/ReadProcessMemory).
#[cfg(windows)]
pub struct WinMem(cz::process::GameProcess);
#[cfg(windows)]
impl Mem for WinMem {
    fn regions(&mut self) -> Vec<(u64, u64)> { cz::process::Memory::regions(&self.0).into_iter().map(|r| (r.start, r.start + r.len)).collect() }
    fn read(&mut self, adr: u64, n: usize) -> Option<Vec<u8>> { cz::process::Memory::read(&self.0, adr, n) }
    fn alive(&mut self) -> bool { cz::process::is_alive(&self.0) }
}
#[cfg(windows)]
pub fn freischalten_hinweis() -> String { "Unter Windows nicht nötig. Verweigert Windows den Zugriff, CalcioM als derselbe Benutzer wie FL27 oder als Administrator starten.".into() }
#[cfg(windows)]
pub fn finder() -> Result<Option<Box<dyn Mem>>, String> {
    if testmodus() { return Ok(None); }
    match cz::process::GameProcess::find() { Ok(Some(g)) => Ok(Some(Box::new(WinMem(g)))), Ok(None) => Ok(None), Err(e) => Err(e.to_string()) }
}
/// What the person runs once so the helper may read FL27's memory.
#[cfg(unix)]
pub fn freischalten_hinweis() -> String { format!("Einmal im Terminal ausführen: sudo setcap cap_sys_ptrace+ep \"{}\"", helfer_pfad().display()) }
/// The real finder: the game's process, read directly if allowed, else through the helper; without rights a clear message.
#[cfg(unix)]
pub fn finder() -> Result<Option<Box<dyn Mem>>, String> {
    let Some(pid) = fl27_pid() else { return Ok(None) };
    if let Ok(mut m) = ProcMem::open(pid) {
        if let Some(&(lo, _)) = m.regions().first() { if m.read(lo, 4).is_some() { return Ok(Some(Box::new(m))); } }
    }
    DienstMem::start(pid).map(|d| Some(Box::new(d) as Box<dyn Mem>))
        .map_err(|e| format!("Speicherzugriff nicht freigeschaltet: {e}. {}", freischalten_hinweis()))
}

/// Runs a Mitlauf in its own thread (beat 2 s) and reports to the interface.
/// Abgebrochen: FL27 was closed during the match; the figures so far, for the person to count or discard.
pub enum Meldung { Status(String), Live(u32, u32), Ansicht(Box<cz::live::MatchView>), Fertig(Box<SpielDoc>), Abgebrochen(Box<SpielDoc>) }
pub struct Beobachter { pub rx: std::sync::mpsc::Receiver<Meldung>, stop: std::sync::Arc<std::sync::atomic::AtomicBool>, abschluss: std::sync::Arc<std::sync::atomic::AtomicBool> }
impl Beobachter {
    /// "Confirm result" (Fabian 24.11.: the final whistle is not always seen): the match as it stands is finished now.
    pub fn jetzt_abschliessen(&self) { self.abschluss.store(true, std::sync::atomic::Ordering::Relaxed); }
}
impl Drop for Beobachter { fn drop(&mut self) { self.stop.store(true, std::sync::atomic::Ordering::Relaxed); } }
pub fn beobachten(team_fn: Box<dyn Fn(&[u32]) -> u32 + Send>) -> Beobachter {
    let (tx, rx) = std::sync::mpsc::channel();
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let s2 = stop.clone();
    let abschluss = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let a2 = abschluss.clone();
    std::thread::spawn(move || {
        let mut ml = Mitlauf::new(Box::new(finder), team_fn);
        let t0 = std::time::Instant::now();
        let mut last = String::new();
        while !s2.load(std::sync::atomic::Ordering::Relaxed) {
            let mut doc = ml.takt(t0.elapsed().as_secs_f64());
            let weg = std::mem::take(&mut ml.abgebrochen);
            // confirmed by hand: the figures so far are the result (once; the page drops the reader after booking)
            let mut weg = weg;
            if doc.is_none() && a2.swap(false, std::sync::atomic::Ordering::Relaxed) { doc = ml.jetzt_fertig(t0.elapsed().as_secs_f64()); weg = false; }
            let status = if let Some(e) = &ml.fehler { e.clone() } else if ml.m.is_none() { "FL27 läuft nicht.".into() } else if ml.basis.is_none() { "FL27 läuft, warte auf den Anstoß …".into() } else { "Partie läuft, CalcioM liest mit.".into() };
            if status != last { let _ = tx.send(Meldung::Status(status.clone())); last = status; }
            if let Some((h, a)) = ml.live() { let _ = tx.send(Meldung::Live(h, a)); }
            if let Some(a) = ml.ansicht() { let _ = tx.send(Meldung::Ansicht(Box::new(a))); }
            if let Some(d) = doc { if tx.send(if weg { Meldung::Abgebrochen(Box::new(d)) } else { Meldung::Fertig(Box::new(d)) }).is_err() { break; } }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });
    Beobachter { rx, stop, abschluss }
}

// ---------------------------------------------------------------- tests without the game (§7.1)
#[cfg(test)]
mod t {
    /// Rename CalcioM → CalcioM: old career folders still count as ours, the old direct start goes, the data folder moves.
    #[test]
    fn alte_namen_werden_uebernommen() {
        assert!(eigener_cpk(concat!("PE", "SMChelseaFC01")) && eigener_cpk("CalcioMChelseaFC01") && !eigener_cpk("_FL26Megapack"));
        assert!(von_uns(concat!("-- PES", " M: Stadion")) && von_uns("-- CalcioM: Stadion") && !von_uns("-- GigaPack"));
        let s = std::env::temp_dir().join(format!("calciom-umzug-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&s);
        let (root, modul) = (concat!("PE", "SM_DirectMatch"), concat!("pesm", "_direktmatch.lua"));
        std::fs::create_dir_all(s.join("livecpk").join(root).join("common")).unwrap();
        std::fs::create_dir_all(s.join("modules")).unwrap();
        std::fs::write(s.join("modules").join(modul), "x").unwrap();
        let ini = format!("[sider]\r\n\tcpk.root = \".\\livecpk\\{root}\"\r\n\tcpk.root = \".\\livecpk\\_FL26Megapack\"\r\n\tlua.module = \"{modul}\"\r\n\t;lua.module = \"{modul}\"\r\n\tlua.module = \"a.lua\"\r\n");
        std::fs::write(s.join("sider.ini"), &ini).unwrap();
        alte_namen_weg(&s).unwrap();
        let neu = std::fs::read_to_string(s.join("sider.ini")).unwrap();
        assert_eq!(neu, ini.replace(&format!("\tcpk.root = \".\\livecpk\\{root}"), &format!("\t;cpk.root = \".\\livecpk\\{root}")).replacen(&format!("\tlua.module = \"{modul}"), &format!("\t;lua.module = \"{modul}"), 1));
        assert!(!s.join("livecpk").join(root).exists() && !s.join("modules").join(modul).exists());
        alte_namen_weg(&s).unwrap();
        assert_eq!(std::fs::read_to_string(s.join("sider.ini")).unwrap(), neu, "a second run changes nothing");
        // data folder: moved, old name left as a link, an existing new folder is never overwritten
        let (alt, ziel) = (s.join("alt"), s.join("neu"));
        std::fs::create_dir_all(alt.join("karrieren")).unwrap();
        umziehen(&alt, &ziel);
        assert!(ziel.join("karrieren").is_dir());
        #[cfg(unix)] assert!(alt.is_symlink() && alt.join("karrieren").is_dir());
        std::fs::create_dir_all(s.join("alt2")).unwrap();
        umziehen(&s.join("alt2"), &ziel);
        assert!(s.join("alt2").is_dir() && !s.join("alt2").is_symlink());
        // a move that failed (Windows: a file still open): the old folder is read where it is, until the new one exists
        let (n3, a3) = (s.join("neu3"), s.join("alt3"));
        std::fs::create_dir_all(&a3).unwrap();
        assert_eq!(neu_oder_alt(n3.clone(), a3.clone()), a3);
        std::fs::create_dir_all(&n3).unwrap();
        assert_eq!(neu_oder_alt(n3.clone(), a3), n3);
        let _ = std::fs::remove_dir_all(&s);
    }
    #[test]
    fn fl27_beenden_nur_im_prefix() {
        let yml = "game:\n  exe: /g/FL_2027.bat\n  prefix: /mnt/GAMES/sp-football-life-2026\n  working_dir: /g\nsystem:\n  env:\n    X: '1'\nwine:\n  version: system\n";
        assert_eq!(lutris_yml(yml), (Some("/mnt/GAMES/sp-football-life-2026".into()), Some("/g".into()), Some("/g/FL_2027.bat".into()), Some("system".into())));
        let p = Path::new("/mnt/GAMES/sp-football-life-2026");
        assert!(im_prefix(b"A=1\0WINEPREFIX=/mnt/GAMES/sp-football-life-2026\0B=2\0", p));
        assert!(!im_prefix(b"WINEPREFIX=/mnt/GAMES/Crimson Desert\0", p), "other Wine games are left alone");
        assert!(!im_prefix(b"HOME=/x\0", p), "without WINEPREFIX nothing is ended");
        if let Ok(l) = lutris_spiel() { assert_eq!(l.prefix, PathBuf::from("/mnt/GAMES/sp-football-life-2026"), "read from the Lutris game configuration"); }
    }
    #[test]
    fn wappen_namen_mit_varianten_und_dds() {
        assert_eq!(bild_schluessel("common/render/symbol/emblemlc/emb_0117_w_l.png.png").map(|x| (x.0, x.1)), Some((1, 117)), "doppelte Endung, Variante w_l");
        assert_eq!(bild_schluessel("common/render/symbol/emblemlc/emb_0037_l.dds").map(|x| (x.0, x.1)), Some((1, 37)), "Logo als .dds");
        assert_eq!(bild_schluessel("common/render/symbol/flag/e_002653_r_ll.dds").map(|x| (x.0, x.1)), Some((0, 2653)));
        assert_eq!(bild_schluessel("common/render/symbol/flag/e_002653.dds").map(|x| (x.0, x.1)), Some((0, 2653)), "Wappen ohne Variante");
        let (a, b) = (bild_schluessel("symbol/flag/e_000001_r_l.png").unwrap().2, bild_schluessel("symbol/flag/e_000001_zz.png").unwrap().2);
        assert!(a < b, "bekannte Varianten vor unbekannten");
    }
    #[test]
    fn wappen_und_logos_aus_der_welt() {
        let w = PathBuf::from("/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk/_FL26Megapack");
        if !echte_welt(&w) { return; }
        let b = bilder_index(&w, w.parent().unwrap().parent().unwrap());
        let e = b.verein.get(&71581).and_then(|q| q.datei()).expect("e_071581 in the world folder");
        assert!(e.starts_with(&w) && e.ends_with("e_071581_r_l.png"), "best variant first: {}", e.display());
        assert!(b.emblem.get(&130).and_then(|q| q.datei()).expect("emb_0130").ends_with("emb_0130_l.png"));
        assert!(!b.verein.contains_key(&999_999_999), "a club without a file falls back to assets/wappen or the drawn crest");
        assert!(b.verein.len() > 100 && b.emblem.len() > 10);
    }

    use super::*;
    /// The diagnostics archive is made without the user name in it.
    #[test]
    fn diagnose_ohne_nutzernamen() {
        if std::env::var("CALCIOM_KARRIEREN").is_err() { return; }
        log(&format!("Test {}", std::env::var("USER").unwrap_or_default()));
        let p = diagnose_paket().unwrap();
        let aus = std::process::Command::new("tar").arg("xzOf").arg(&p).output().unwrap();
        let t = String::from_utf8_lossy(&aus.stdout);
        assert!(t.contains("Test <user>") && t.contains("CalcioM"), "{t}");
        let _ = std::fs::remove_file(p);
    }
    /// Beta 23.11.: another pack's stadiums.lua (here: ours without the addition, with other comments) gets the preset
    /// at its anchors, and the result is valid Lua (luac -p, when installed).
    #[test]
    fn stadion_vorgabe_in_fremde_datei() {
        // the module files are not in the public repository
        let Ok(a) = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fl27/module/stadiums.lua")) else { return };
        let nl = "\r\n";
        let z: Vec<&str> = a.split(nl).collect();
        let von = z.iter().position(|l| l.starts_with("-- CalcioM: Stadion-Vorgabe")).unwrap();
        let bis = z.iter().position(|l| l.starts_with("local function get_new_stadium_path")).unwrap();
        let mut o: Vec<String> = z[..von].iter().chain(z[bis..].iter()).map(|s| s.to_string()).collect();
        let k = o.iter().position(|l| l.trim() == "local pesm_idx = pesm_stadion(ctx)").unwrap();
        o.drain(k..k + 6);
        o[k] = o[k].replacen("elseif manual_selection_status", "if manual_selection_status", 1);
        let k = o.iter().position(|l| l.trim() == "if tid == 65535 and pesm_stadion(ctx) then").unwrap();
        o.drain(k..k + 8);
        o.insert(k, "\t\tif tid == 65535 then".into());
        o.insert(1, "-- another pack's comment".into());
        let fremd = o.join(nl);
        assert!(!fremd.contains("pesm_stadion"));
        let neu = stadion_einsetzen(&fremd).expect("anchors found");
        assert_eq!(neu.matches("pesm_stadion(ctx)").count(), 4);
        if std::process::Command::new("luac5.1").arg("-v").output().is_ok() {
            let d = std::env::temp_dir().join(format!("calciom-stadion-{}.lua", std::process::id()));
            std::fs::write(&d, &neu).unwrap();
            let ok = std::process::Command::new("luac5.1").args(["-p"]).arg(&d).status().unwrap().success();
            let _ = std::fs::remove_file(&d);
            assert!(ok, "invalid Lua");
        }
        assert!(stadion_einsetzen("print(1)").is_none(), "no anchors: leave it");
    }


    struct Fake { base: u64, data: Vec<u8>, alive: bool }
    impl Mem for Fake {
        fn regions(&mut self) -> Vec<(u64, u64)> { vec![(self.base, self.base + self.data.len() as u64)] }
        fn read(&mut self, adr: u64, n: usize) -> Option<Vec<u8>> {
            let o = adr.checked_sub(self.base)? as usize;
            (o + n <= self.data.len()).then(|| self.data[o..o + n].to_vec())
        }
        fn alive(&mut self) -> bool { self.alive }
    }
    fn put(d: &mut [u8], o: usize, v: u32) { d[o..o + 4].copy_from_slice(&v.to_le_bytes()); }
    /// A block at `at` with 14 players per side; set(side, place, kind, section, value)
    fn block(d: &mut [u8], at: usize, ids: u32) {
        for side in 0..2 { for pl in 0..14 { let o = at + side * SEITE + pl * REC; put(d, o, KOPF[side]); put(d, o + 4, ids + (side * 100 + pl) as u32); } }
    }
    fn set(d: &mut [u8], at: usize, side: usize, pl: usize, k: usize, ab: usize, v: u32) { put(d, at + side * SEITE + pl * REC + 8 + k * 36 + ab * 4, v); }
    fn mem(blocks: &[usize]) -> Fake {
        let mut d = vec![0u8; 0x200000];
        for (i, b) in blocks.iter().enumerate() { block(&mut d, *b, 1000 * (i as u32 + 1)); }
        Fake { base: 0x10000, data: d, alive: true }
    }

    #[test]
    fn finds_blocks_and_walks_back() {
        let mut m = mem(&[0x2000]);
        // a header in the middle of the block (place 5) is found and walked back to place 0
        let b = bloecke(&mut m);
        assert_eq!(b, vec![0x10000 + 0x2000]);
        let mut m = mem(&[0x2000, 0x2000 + 2 * SEITE + 0x4000]);
        assert_eq!(bloecke(&mut m).len(), 2);
    }

    #[test]
    fn score_cards_stats() {
        let mut m = mem(&[0x2000]);
        let at = 0x2000;
        set(&mut m.data, at, 0, 9, GOAL as usize, 0, 1);      // home goal
        set(&mut m.data, at, 0, 8, GOAL as usize, 8, 1);      // section 8 (shoot-out) is not in the game's total (0x1408ca920 mode 5)
        set(&mut m.data, at, 0, 3, 176, 0, 1);                // kind 176 exists in the game (kind < 0xb1), the Coaching Zone reads 0..175
        set(&mut m.data, at, 1, 3, OWNGOAL as usize, 2, 1);   // away own goal counts for home
        set(&mut m.data, at, 1, 4, CARD_Y as usize, 1, 1);    // yellow-red: 2 yellows + 1 red
        set(&mut m.data, at, 1, 4, CARD_Y as usize, 4, 1);
        set(&mut m.data, at, 1, 4, CARD_R as usize, 4, 1);
        set(&mut m.data, at, 0, 9, SHOOT as usize, 0, 3);
        set(&mut m.data, at, 0, 9, SHOOT_INFRAME as usize, 0, 2);
        set(&mut m.data, at, 0, 7, SHOOT as usize, 3, 1);
        let st = lesen(&mut m, 0x10000 + at as u64).unwrap();
        assert_eq!(endstand(&st), (2, 0));
        assert_eq!(summe(&st.s[0], 176), 0, "kind 0xB0 is not read (unused; wish for the Coaching Zone in PAKETE.md)");
        let d = doc(&st, "x".into(), 1, 2, &HashMap::new(), None);
        let karten: Vec<&Ev> = d.ev.iter().filter(|e| matches!(e.art, EvArt::Karte { .. })).collect();
        assert_eq!(karten.len(), 2);
        assert_eq!(karten.iter().filter(|e| e.art == EvArt::Karte { rot: true }).count(), 1);
        assert_eq!(d.team(SHOOT), [4, 0]);
        assert!((d.xg[0] - 0.32).abs() < 1e-4, "{}", d.xg[0]);
        assert!(d.ev.iter().any(|e| e.art == EvArt::Tor { vorlage: None, eigentor: true } && e.seite == 0));
    }

    #[test]
    fn follows_match_minutes_subs_and_end() {
        let shared = std::sync::Arc::new(std::sync::Mutex::new(mem(&[0x2000])));
        let at = 0x2000;
        struct Shared(std::sync::Arc<std::sync::Mutex<Fake>>);
        impl Mem for Shared {
            fn regions(&mut self) -> Vec<(u64, u64)> { self.0.lock().unwrap().regions() }
            fn read(&mut self, a: u64, n: usize) -> Option<Vec<u8>> { self.0.lock().unwrap().read(a, n) }
            fn alive(&mut self) -> bool { self.0.lock().unwrap().alive }
        }
        let s2 = shared.clone();
        // like the real finder: no process, no memory
        let mut ml = Mitlauf::new(Box::new(move || Ok(s2.lock().unwrap().alive.then(|| Box::new(Shared(s2.clone())) as Box<dyn Mem>))), Box::new(|_| 7));
        let w = |f: &dyn Fn(&mut Vec<u8>)| f(&mut shared.lock().unwrap().data);
        // kick-off: everything 0, all starters run
        for pl in 0..11 { w(&|d| { set(d, at, 0, pl, LAUF as usize, 0, 1); set(d, at, 1, pl, LAUF as usize, 0, 1); }); }
        w(&|d| set(d, at, 0, 0, PASS as usize, 0, 0));
        assert!(ml.takt(0.0).is_none());
        // goal after 40 s in section 0, section 1 starts after 90 s
        w(&|d| { set(d, at, 0, 9, GOAL as usize, 0, 1); set(d, at, 0, 2, PASS as usize, 0, 5); });
        assert!(ml.takt(40.0).is_none());
        // a sub: bench player 12 gets values, only starter 6 of home stands still on kind 173
        w(&|d| { for pl in 0..11 { if pl != 6 { set(d, at, 0, pl, LAUF as usize, 1, 2); } set(d, at, 1, pl, LAUF as usize, 1, 2); } set(d, at, 0, 12, LAUF as usize, 1, 1); set(d, at, 0, 2, PASS as usize, 1, 3); });
        assert!(ml.takt(90.0).is_none());
        w(&|d| set(d, at, 0, 2, PASS as usize, 2, 3));
        assert!(ml.takt(180.0).is_none());
        // "Confirm result" before the whistle is seen: the score and figures as they stand
        let jetzt = ml.jetzt_fertig(181.0).expect("a match is read");
        assert_eq!((jetzt.hg, jetzt.ag), (1, 0));
        // values fall back to 0: new match, the old one is closed exactly once
        w(&|d| { for side in 0..2 { for pl in 0..14 { for k in 0..ARTEN { for ab in 0..9 { set(d, at, side, pl, k, ab, 0); } } } } });
        let d = ml.takt(200.0).expect("closed");
        assert_eq!((d.hg, d.ag), (1, 0));
        let tor = d.ev.iter().find(|e| matches!(e.art, EvArt::Tor { .. })).unwrap();
        assert!(tor.min < 15 && !tor.unsicher, "{tor:?}");
        let sub = d.ev.iter().find(|e| matches!(e.art, EvArt::Wechsel { .. })).unwrap();
        assert_eq!(sub.art, EvArt::Wechsel { raus: Some(1006) });
        assert_eq!(sub.pid, 1012);
        assert!(ml.takt(202.0).is_none());
        assert!(!ml.abgebrochen, "the game ran on: a whistled match, not a broken-off one");
        // a next match is under way and FL27 is closed: its figures come once, marked as broken off (Discord 26.11.)
        w(&|d| set(d, at, 0, 9, GOAL as usize, 0, 1));
        assert!(ml.takt(203.0).is_none());
        shared.lock().unwrap().alive = false;
        let d = ml.takt(204.0).expect("the broken-off match");
        assert!(ml.abgebrochen && (d.hg, d.ag) == (1, 0));
        ml.abgebrochen = false;
        assert!(ml.takt(206.0).is_none());
    }

    #[test]
    fn kopf_mit_anderen_oberen_bytes_und_live_ansicht() {
        // FL27 writes the head as 0x0707FFFD in some matches (Coaching Zone, stats.rs): the block is still found and read
        let mut m = mem(&[]);
        let at = 0x2000;
        for side in 0..2 { for pl in 0..14 { let o = at + side * SEITE + pl * REC; put(&mut m.data, o, 0x0707_0000 | KOPF[side]); put(&mut m.data, o + 4, 500 + (side * 100 + pl) as u32); } }
        assert_eq!(bloecke(&mut m), vec![0x10000 + at as u64]);
        set(&mut m.data, at, 0, 9, GOAL as usize, 1, 1);
        assert_eq!(endstand(&lesen(&mut m, 0x10000 + at as u64).unwrap()), (1, 0));
        // the Coaching Zone's tracker in the Mitlauf gives the live view: score, the goal as an event, grades
        let shared = std::sync::Arc::new(std::sync::Mutex::new(mem(&[at])));
        struct Shared(std::sync::Arc<std::sync::Mutex<Fake>>);
        impl Mem for Shared {
            fn regions(&mut self) -> Vec<(u64, u64)> { self.0.lock().unwrap().regions() }
            fn read(&mut self, a: u64, n: usize) -> Option<Vec<u8>> { self.0.lock().unwrap().read(a, n) }
            fn alive(&mut self) -> bool { self.0.lock().unwrap().alive }
        }
        let s2 = shared.clone();
        let mut ml = Mitlauf::new(Box::new(move || Ok(Some(Box::new(Shared(s2.clone())) as Box<dyn Mem>))), Box::new(|_| 7));
        for pl in 0..11 { for side in 0..2 { set(&mut shared.lock().unwrap().data, at, side, pl, LAUF as usize, 0, 1); } }
        assert!(ml.takt(0.0).is_none());
        set(&mut shared.lock().unwrap().data, at, 0, 9, GOAL as usize, 0, 1);
        set(&mut shared.lock().unwrap().data, at, 0, 9, SHOOT as usize, 0, 1);
        assert!(ml.takt(2.0).is_none());
        let a = ml.ansicht().expect("live view");
        assert_eq!(a.score, [1, 0]);
        assert!(a.events.iter().any(|e| e.kind == cz::live::EventKind::Goal && e.side == 0 && e.slot == 9), "{:?}", a.events);
        assert!(a.players.iter().any(|p| p.id == 1009 && p.goals == 1 && p.rating.unwrap_or(0.0) > 6.0));
    }

    #[test]
    fn no_rights_is_a_message() {
        let mut ml = Mitlauf::new(Box::new(|| Err("Speicherzugriff nicht freigeschaltet".into())), Box::new(|_| 0));
        assert!(ml.jetzt_fertig(0.0).is_none(), "nothing read, nothing to confirm");
        assert!(ml.takt(0.0).is_none());
        assert!(ml.fehler.as_deref().unwrap().contains("nicht freigeschaltet"));
    }

    /// Discord 24.11.: the set-piece takers from CalcioM land in PlayerAssignment.bin, one player per flag, other teams untouched.
    #[test]
    fn schuetzen_in_player_assignment() {
        let mut a = vec![Asg { nr: 1, pid: 10, team: 5, k: 0x3f << 16 }, Asg { nr: 2, pid: 11, team: 5, k: 3 }, Asg { nr: 3, pid: 20, team: 6, k: 0x3f << 16 }];
        schuetzen_setzen(&mut a, 5, [Some(11), None, Some(11), Some(99)]);
        assert_eq!((a[1].k >> 16) & 0x3f, 0b010011, "free kicks and penalty to 11");
        assert_eq!((a[0].k >> 16) & 0x3f, 0b101100, "10 keeps corners and captain (99 is not in the team)");
        assert_eq!(a[1].k & 0x7f, 3, "shirt number untouched");
        assert_eq!(a[2].k >> 16, 0x3f, "other team untouched");
    }
    #[test]
    fn pesdb_lineup_transfer() {
        let a0 = vec![Asg { nr: 1, pid: 10, team: 5, k: (0 << 10) | 0 }, Asg { nr: 2, pid: 11, team: 5, k: (1 << 10) | 8 }, Asg { nr: 3, pid: 12, team: 5, k: (2 << 10) | 9 | 0x10000 }, Asg { nr: 4, pid: 20, team: 6, k: 0 }];
        let raw = pack(*b"\x01\x02\x03WESYS", &asg_bytes(&a0));
        let (h, d) = unpack(&raw).unwrap();
        assert_eq!(asg_parse(&d), a0);
        assert_eq!(pack(h, &d), raw);
        let mut a = a0.clone();
        set_lineup(&mut a, 5, &[12, 10]).unwrap();
        assert_eq!((a[2].platz(), a[0].platz(), a[1].platz()), (0, 1, 2));
        assert_eq!((a[2].nummer(), a[2].k & 0x10000), (10, 0x10000)); // numbers and flags stay
        let before = a.clone();
        assert!(set_lineup(&mut a, 5, &[20]).is_err());
        assert!(set_lineup(&mut a, 5, &[10, 10]).is_err());
        assert_eq!(a, before);
        transfer(&mut a, 11, 5, 6, 1).unwrap();             // number 1 is taken at team 6 -> first free is 2
        let x = a.iter().find(|x| x.pid == 11).unwrap();
        assert_eq!((x.team, x.nummer(), x.platz()), (6, 2, 1));
        transfer(&mut a, 99, 0, 6, 7).unwrap();
        assert_eq!(a.last().unwrap().nr, 5);
        transfer(&mut a, 99, 6, 0, 0).unwrap();
        assert!(a.iter().all(|x| x.pid != 99));
        let clubs: HashSet<u32> = [5, 6].into();
        assert_eq!(team_majority(&a, &[10, 12, 11], &clubs), 5);
    }

    #[test]
    fn welt_ueber_der_karriere_wird_verschoben() {
        // Fabian 22.11.: the world above the career and a second career active - the world's database won, Bischof was missing
        let ini = "[sider]\r\n\tcpk.root = \".\\livecpk\\_FL26Megapack\"\r\n\tcpk.root = \".\\livecpk\\CalcioM_DirectMatch\"\r\n\tcpk.root = \".\\livecpk\\PESMSunderlandAFC01\"\r\n\tcpk.root = \".\\livecpk\\PESMBayernMunchen01\"\r\n\tcpk.root = \".\\livecpk\\boot-root\"\r\n";
        let neu = sider_ini_einrichten(ini, "PESMSunderlandAFC01", "_FL26Megapack", &["PESMBayernMünchen01".into()], true).unwrap();
        let aktiv: Vec<&str> = neu.split("\r\n").filter(|l| l.trim_start().starts_with("cpk.root")).collect();
        assert!(aktiv[0].contains("CalcioM_DirectMatch") && aktiv[1].contains("PESMSunderlandAFC01") && aktiv[2].contains("_FL26Megapack"), "{neu}");
        assert!(!aktiv.iter().any(|l| l.contains("PESMBayernMunchen01")), "andere Karriere aus: {neu}");
        assert_eq!(sider_ini_einrichten(&neu, "PESMSunderlandAFC01", "_FL26Megapack", &[], true).unwrap(), neu, "bleibt gleich");
    }
    #[test]
    fn formation_schreiben_und_elf_einsortieren() {
        // the eleven onto the places of a 4-2-1-3 by their positions
        let codes = [0u8, 1, 1, 3, 2, 4, 4, 8, 10, 9, 12];
        let elf = [(12u8, 11), (0, 1), (2, 5), (3, 4), (1, 2), (1, 3), (4, 6), (5, 7), (8, 8), (9, 10), (10, 9)];
        let o = nach_formation(&codes, &elf);
        assert_eq!(formation_name(&[(0, 3, 52), (1, 10, 63), (1, 10, 41), (3, 12, 89), (2, 12, 15), (4, 18, 64), (4, 18, 40), (8, 28, 53), (10, 37, 88), (9, 37, 16), (12, 43, 52)]), "4-2-1-3", "wie im Spiel");
        assert_eq!(o[0], 1, "Torwart auf Platz 0");
        assert_eq!(o[10], 11, "Mittelstürmer vorn");
        assert_eq!(o[4], 5, "Linksverteidiger links");
        assert!(o[5] == 6 || o[6] == 6 || o[6] == 7 || o[5] == 7, "Sechser im Mittelfeld");
        // own places in the order of the world's data: RB after the centre-backs, RM after the central midfield
        let mut f442 = vec![(0u8, 3u8, 52u8, 1u32), (3, 12, 88, 2), (1, 10, 64, 3), (1, 10, 40, 4), (2, 12, 16, 5), (7, 28, 88, 6), (5, 26, 64, 7), (5, 26, 40, 8), (6, 28, 16, 9), (12, 43, 64, 10), (12, 43, 40, 11)];
        formation_ordnen(&mut f442);
        assert_eq!(f442.iter().map(|x| x.3).collect::<Vec<_>>(), vec![1, 3, 4, 2, 5, 7, 8, 6, 9, 10, 11]);
        // a formation written into a copy of the world reads back the same
        let welt = sider().join("livecpk/_FL26MyWorld/common/etc/pesdb");
        if !welt.join("TacticsFormation.bin").exists() { return; }
        let d = std::env::temp_dir().join(format!("calciom-formation-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        for n in ["Tactics.bin", "TacticsFormation.bin"] { std::fs::copy(welt.join(n), d.join(n)).unwrap(); }
        let p442: Vec<(u8, u8, u8)> = vec![(0, 3, 52), (3, 12, 89), (1, 10, 63), (1, 10, 41), (2, 12, 15), (7, 30, 87), (5, 20, 64), (5, 20, 40), (6, 30, 18), (12, 43, 66), (12, 43, 38)];
        assert!(formation_schreiben(&d, 102, &p442).unwrap());
        assert_eq!(formation_codes(&d, 102).unwrap(), p442.iter().map(|x| x.0).collect::<Vec<_>>());
        assert!(!formation_schreiben(&d, 102, &p442).unwrap(), "zweites Mal unverändert");
        // sliders: compactness 2 and line 8 in the game = half-bytes 1 and 7
        assert!(regler_schreiben(&d, 102, Some(2), None, Some(8)).unwrap());
        let (_, t) = unpack(&std::fs::read(d.join("Tactics.bin")).unwrap()).unwrap();
        let o = (0..t.len() / 12).map(|i| i * 12).find(|&o| le32(&t, o + 4) == 102).unwrap();
        let v = le32(&t, o + 8);
        assert_eq!((v & 15, (v >> 8) & 15), (1, 7));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn fremde_welt_wird_abgeschaltet() {
        // a career on MyWorld while the Megapack line is active above it: the Megapack line goes out, MyWorld comes in below
        let ini = "[sider]\r\n\tcpk.root = \".\\livecpk\\_FL26Megapack\"\r\n\tcpk.root = \".\\livecpk\\CalcioM_DirectMatch\"\r\n\tcpk.root = \".\\livecpk\\PESMChelseaFC01\"\r\n\t#cpk.root = \".\\livecpk\\_FL26MyWorld\"\r\n";
        let neu = sider_ini_einrichten(ini, "PESMChelseaFC01", "_FL26MyWorld", &["_FL26Megapack".into()], true).unwrap();
        let aktiv: Vec<&str> = neu.split("\r\n").filter(|l| l.trim_start().starts_with("cpk.root")).collect();
        assert!(!aktiv.iter().any(|l| l.contains("_FL26Megapack")), "{neu}");
        let c = aktiv.iter().position(|l| l.contains("PESMChelseaFC01")).unwrap();
        let w = aktiv.iter().position(|l| l.contains("_FL26MyWorld")).unwrap();
        assert!(c < w, "{neu}");
    }

    #[test]
    fn wettbewerb_aus_den_karten() {
        // real lines of FL27's scoreboards map and Entrance.lua (24.11.)
        let sb = "\u{feff}# format: tournament_ID, scoreboard_path\r\n65535, Exhibición\\eFootball\r\n50, Germany\\Bundesliga\r\n#23, _FL26-27-Def\\_packs\\ENGC   # off\r\n11, _FL26Megapack\\A-League Men   # FL26 Mod Studio\r\n5, UEFA\\UEFA Europa League 21\\DAZN\r\n";
        let sb = karten_tids(sb, &["_FL26Megapack".into()]);
        assert!(sb.contains(&50) && sb.contains(&5) && !sb.contains(&23) && !sb.contains(&11), "{sb:?}");
        let ent = entrance_tids("\tif tid == 2 or tid == 1026 then\n\telseif tid == 23 then\n\t--elseif tid == 112 then\n");
        assert_eq!(ent, [2, 1026, 23].into_iter().collect());
        let k = [sb, ent];
        assert_eq!(turnier_waehlen(&[23, 17], &k), Some(23), "FA Cup keeps its walk-out, not the league look");
        assert_eq!(turnier_waehlen(&[186, 5, 2], &k), Some(5), "Conference League without a look: Europa League");
        assert_eq!(turnier_waehlen(&[11, 50], &k), Some(50), "2. Bundesliga on MyWorld: tid 11 is the Megapack's A-League");
        assert_eq!(turnier_waehlen(&[99], &k), Some(99));
        assert_eq!(turnier_waehlen(&[], &k), None);
    }

    #[test]
    fn presets_and_sider_ini() {
        let mut reg = vec![0u8; 2352 * 3];
        reg[2..4].copy_from_slice(&17u16.to_le_bytes()); reg[8] = 9;
        reg[2352 + 2..2352 + 4].copy_from_slice(&23u16.to_le_bytes()); reg[2352 + 8] = 9;
        reg[4704 + 2..4704 + 4].copy_from_slice(&50u16.to_le_bytes()); reg[4704 + 8] = 39;
        let t = turnier_ids(&unpack(&pack(*b"xyzWESYS", &reg)).unwrap().1);
        assert_eq!((t[&9], t[&39]), (17, 50));
        let mt = "#STADIUM\r\n396, 009, Stadium of Light, England\\Stadium of Light\t\t# Sunderland\r\n4071, 009, Vitality Stadium, England\\Dean Court  # B\r\n";
        assert_eq!(heimstadion(mt, 396).as_deref(), Some("England\\Stadium of Light"));
        // t2-ecom 26.11.: add-ons with a few pesdb files that an older version switched off come back on, worlds stay off
        let d = std::env::temp_dir().join(format!("calciom-zusatz-{}", std::process::id()));
        for (n, f) in [("boot-root", "Boots.bin"), ("_FL26Megapack", "Player.bin"), ("Mods", "Glove.bin")] { std::fs::create_dir_all(d.join("livecpk").join(n).join("common/etc/pesdb")).unwrap(); std::fs::write(d.join("livecpk").join(n).join("common/etc/pesdb").join(f), b"x").unwrap(); }
        let vorher = "\tcpk.root = \".\\livecpk\\boot-root\"\r\n\tcpk.root = \".\\livecpk\\_FL26Megapack\"\r\n\t;cpk.root = \".\\livecpk\\Mods\"";
        let jetzt = "\t;cpk.root = \".\\livecpk\\boot-root\"\r\n\t;cpk.root = \".\\livecpk\\_FL26Megapack\"\r\n\t;cpk.root = \".\\livecpk\\Mods\"";
        assert_eq!(zusaetze_wieder_an(jetzt, vorher, &d), "\tcpk.root = \".\\livecpk\\boot-root\"\r\n\t;cpk.root = \".\\livecpk\\_FL26Megapack\"\r\n\t;cpk.root = \".\\livecpk\\Mods\"");
        let _ = std::fs::remove_dir_all(&d);
        assert_eq!(heimstadion(mt, 1), None);
        let dir = std::env::temp_dir().join(format!("calciom-vorgaben-{}", std::process::id()));
        vorgaben_schreiben(&dir, Some(17), Some("rainy"), Some("nacht"), Some("England\\Stadium of Light")).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("flw/wetter.txt")).unwrap(), "wetter=rainy\r\ntageszeit=nacht\r\n");
        assert!(vorgaben_schreiben(&dir, None, Some("hagel"), None, None).is_err());
        // the match rules for the LivePatch module (two legs): written with the presets, removed with them
        vorgaben_schreiben_mit(&dir, None, None, None, None, Some("verlaengerung=0\r\nelfmeter=0\r\nhinspiel=2:1\r\n".into())).unwrap();
        assert!(std::fs::read_to_string(dir.join("flw/spielregeln.txt")).unwrap().contains("hinspiel=2:1"));
        vorgaben_schreiben(&dir, None, None, None, None).unwrap();
        // undo after the match: only what CalcioM wrote and nobody changed
        vorgaben_schreiben(&dir, Some(17), Some("rainy"), Some("nacht"), Some("England\\Stadium of Light")).unwrap();
        let gesetzt = vorgaben_merken(&dir);
        assert_eq!(gesetzt.len(), 3, "der Start setzt drei Dateien");
        assert_eq!(vorgaben_zuruecksetzen(&dir, &gesetzt).unwrap(), Vec::<String>::new());
        assert!(VORGABEN.iter().all(|n| !dir.join("flw").join(n).exists()), "Beenden entfernt genau diese");
        vorgaben_schreiben(&dir, Some(17), Some("rainy"), Some("nacht"), Some("England\\Stadium of Light")).unwrap();
        let gesetzt = vorgaben_merken(&dir);
        std::fs::write(dir.join("flw/stadion.txt"), "Deutschland\\Allianz Arena\r\n").unwrap();
        std::fs::write(dir.join("flw/fremd.txt"), "x").unwrap();
        assert_eq!(vorgaben_zuruecksetzen(&dir, &gesetzt).unwrap(), vec!["stadion.txt".to_string()], "eine fremd geänderte Datei bleibt");
        assert!(dir.join("flw/stadion.txt").exists() && dir.join("flw/fremd.txt").exists() && !dir.join("flw/wetter.txt").exists());
        let ini = "[cpk]\r\ncpk.root = \".\\livecpk\\PESM_Bayern\"\r\ncpk.root = \".\\livecpk\\_FL26Megapack\"\r\n";
        // after the match: as without CalcioM (career, direct start, our modules off; other mods untouched), twice the same
        let mit = "[sider]\r\n\tcpk.root = \".\\livecpk\\CalcioM_DirectMatch\"\r\n\tcpk.root = \".\\livecpk\\PESMBayern01\"\r\n\tcpk.root = \".\\livecpk\\_FL26Megapack\"\r\n\tlua.module = \"calciom_direktmatch.lua\"\r\n\tlua.module = \"pesm-livepatch.lua\"\r\n\tlua.module = \"fl26match.lua\"\r\n";
        let ohne = sider_ini_ohne_calciom(mit, "PESMBayern01");
        assert_eq!(ohne, mit.replace("\tcpk.root = \".\\livecpk\\CalcioM", "\t;cpk.root = \".\\livecpk\\CalcioM").replace("\tcpk.root = \".\\livecpk\\PESMB", "\t;cpk.root = \".\\livecpk\\PESMB").replace("\tlua.module = \"calciom", "\t;lua.module = \"calciom").replace("\tlua.module = \"pesm-", "\t;lua.module = \"pesm-"));
        assert_eq!(sider_ini_ohne_calciom(&ohne, "PESMBayern01"), ohne);
        let aus = sider_ini_abschalten(ini, "PESM_Bayern");
        assert!(aus.contains(";cpk.root = \".\\livecpk\\PESM_Bayern\"") && aus.contains("\r\ncpk.root = \".\\livecpk\\_FL26Megapack\"") && aus.ends_with("\r\n"));
        assert!(sider_ini_einrichten(&aus, "PESM_Bayern", "_FL26Megapack", &[], false).unwrap().contains("\ncpk.root = \".\\livecpk\\PESM_Bayern\""), "der nächste Start schaltet sie wieder ein");
        vorgaben_schreiben(&dir, None, None, None, None).unwrap();
        assert!(!dir.join("flw/wettbewerb.txt").exists() && !dir.join("flw/stadion.txt").exists());
        let _ = std::fs::remove_dir_all(&dir);
        let ini = "[sider]\r\n\t;--- CalcioM Direktstart (Stufe 1) ---\r\n\tcpk.root = \".\\livecpk\\CalcioM_DirectMatch\"\r\n\tcpk.root = \".\\livecpk\\boot-root\"\r\n\t#cpk.root = \".\\livecpk\\_FL26MyWorld\"\r\n";
        let other = "\tcpk.root = \".\\livecpk\\PESMArsenalFC01\"\r\n";
        let ini2 = ini.replace("\tcpk.root = \".\\livecpk\\boot-root\"\r\n", &format!("{other}\tcpk.root = \".\\livecpk\\boot-root\"\r\n"));
        let neu = sider_ini_einrichten(&ini2, "PESMSunderlandAFC01", WELT_ROOT, &["PESMArsenalFC01".into()], true).unwrap();
        let lines: Vec<&str> = neu.split("\r\n").collect();
        assert_eq!(lines[3], "\tcpk.root = \".\\livecpk\\PESMSunderlandAFC01\"");
        assert_eq!(lines[4], "\t;cpk.root = \".\\livecpk\\PESMArsenalFC01\"");
        assert_eq!(lines[6], "\tcpk.root = \".\\livecpk\\_FL26MyWorld\"");
        assert!(neu.ends_with("\r\n") && !neu.replace("\r\n", "").contains('\n'));
        assert_eq!(sider_ini_einrichten(&neu, "PESMSunderlandAFC01", WELT_ROOT, &["PESMArsenalFC01".into()], true).unwrap(), neu); // stays byte-equal
        assert_eq!(sider_ini_rueckbau(&neu, "PESMSunderlandAFC01"), ini2.replace("#cpk.root", "cpk.root").replace("\tcpk.root = \".\\livecpk\\PESMArsenalFC01", "\t;cpk.root = \".\\livecpk\\PESMArsenalFC01"));
        // a misplaced own line moves up; a missing world line comes directly below the career line
        let ini3 = "\tcpk.root = \".\\livecpk\\boot-root\"\n\tcpk.root = \".\\livecpk\\PESMSunderlandAFC01\"\n";
        let n3 = sider_ini_einrichten(ini3, "PESMSunderlandAFC01", WELT_ROOT, &[], false).unwrap();
        assert_eq!(n3, "\tcpk.root = \".\\livecpk\\PESMSunderlandAFC01\"\n\tcpk.root = \".\\livecpk\\_FL26MyWorld\"\n\tcpk.root = \".\\livecpk\\boot-root\"\n");
        assert_eq!(cpk_name("Bayern München", Path::new("/nonexistent")), "CalcioMBayernMunchen01");
        assert!(zeit().len() == 15);
    }

    #[test]
    fn start_order() {
        struct F { log: Vec<&'static str>, ok: bool, laeuft: bool, neustart: bool }
        impl Ablauf for F {
            fn erlaubt(&self) -> bool { self.ok }
            fn bestaetigt(&self) -> bool { true }
            fn speichern(&mut self) -> Result<(), String> { self.log.push("speichern"); Ok(()) }
            fn spiel_laeuft(&self) -> bool { self.laeuft }
            fn sicherung(&mut self) -> Result<(), String> { self.log.push("sicherung"); Ok(()) }
            fn vorbereiten(&mut self) -> Result<(), String> { self.log.push("vorbereiten"); Ok(()) }
            fn laden(&mut self) -> Result<(), String> { self.log.push("laden"); Ok(()) }
            fn konfiguration(&mut self) -> Result<(), String> { self.log.push("konfiguration"); Ok(()) }
            fn vorgaben(&mut self) -> Result<(), String> { self.log.push("vorgaben"); Ok(()) }
            fn start(&mut self) -> Result<(), String> { self.log.push("start"); Ok(()) }
            fn live_bereit(&mut self) -> Result<(), String> { self.log.push("live_bereit"); if self.neustart { Err("Bitte FL27 neu starten.".into()) } else { Ok(()) } }
            fn live_patch(&mut self) -> Result<(), String> { self.log.push("live_patch"); Ok(()) }
        }
        let mut f = F { log: vec![], ok: true, laeuft: false, neustart: false };
        match_starten(&mut f).unwrap();
        assert_eq!(f.log, ["speichern", "sicherung", "vorbereiten", "laden", "konfiguration", "vorgaben", "start"]);
        let mut f = F { log: vec![], ok: false, laeuft: false, neustart: false };
        assert!(match_starten(&mut f).is_err() && f.log.is_empty());
        let mut f = F { log: vec![], ok: true, laeuft: true, neustart: false };
        match_starten(&mut f).unwrap();
        assert_eq!(f.log, ["speichern", "live_bereit", "vorbereiten", "laden", "vorgaben", "live_patch"]);
        let mut f = F { log: vec![], ok: true, laeuft: true, neustart: true };
        assert!(match_starten(&mut f).unwrap_err().contains("neu starten") && f.log == ["speichern", "live_bereit"]);
    }
}

#[cfg(test)]
mod echt {
    use super::*;
    /// Read-only against the real world database, when it exists on this computer.
    #[test]
    fn real_pesdb() {
        let p = welt_ordner(&sider()).join("common/etc/pesdb");
        if !echte_welt(&p) { return; }
        let raw = std::fs::read(p.join("PlayerAssignment.bin")).unwrap();
        let (h, d) = unpack(&raw).unwrap();
        let a = asg_parse(&d);
        assert_eq!(asg_bytes(&a), d);
        assert_eq!(unpack(&pack(h, &d)).unwrap().1, d);
        let comps = competitions(&pesdb_datei(&p, "Competition.bin").unwrap());
        let t = turnier_ids(&pesdb_datei(&p, "CompetitionRegulation.bin").unwrap());
        let id = |n: &str| comps.iter().find(|c| c.1 == n).map(|c| c.0).unwrap();
        assert_eq!(t[&id("ENGLAND_D1_LEAGUE")], 17);
        assert_eq!(t[&id("GERMANY_D1_LEAGUE")], 50);
        assert_eq!(t[&id("UEFA_CHAMPIONS_LEAGUE")], 2);
        // places 0 are mostly keepers, the Sunderland squad has its players (spec §2.3, §7.3)
        let sq: Vec<&Asg> = a.iter().filter(|x| x.team == 396).collect();
        assert!(sq.len() >= 20 && sq.iter().any(|x| x.pid == 132157));
        let mt = std::fs::read_to_string(sider().join("content/stadiums/map_teams.txt")).unwrap_or_default();
        assert_eq!(heimstadion(&mt, 396).as_deref(), Some("England\\Stadium of Light"));
        eprintln!("{} assignments, {} competitions", a.len(), comps.len());
    }
}

#[cfg(test)]
mod pruefung_tests {
    use super::*;
    /// The slim backup copies only common/etc/pesdb (and sider.ini), the full one everything.
    #[test]
    fn schlanke_sicherung() {
        let t = std::env::temp_dir().join(format!("calciom-schlank-{}", std::process::id()));
        let welt = t.join("Welt");
        std::fs::create_dir_all(welt.join("common/etc/pesdb")).unwrap();
        std::fs::create_dir_all(welt.join("common/render/face")).unwrap();
        std::fs::write(welt.join("common/etc/pesdb/Player.bin"), b"db").unwrap();
        std::fs::write(welt.join("common/render/face/1.dds"), b"bild").unwrap();
        let s = t.join("sider");
        std::fs::create_dir_all(&s).unwrap();
        std::fs::write(s.join("sider.ini"), b"x").unwrap();
        let mut files = vec![];
        copy_dir(&welt, &t.join("k/Welt"), &mut files, Path::new("common/etc/pesdb")).unwrap();
        assert!(t.join("k/Welt/common/etc/pesdb/Player.bin").exists() && !t.join("k/Welt/common/render").exists());
        assert_eq!(files, vec![PathBuf::from("common/etc/pesdb/Player.bin")]);
        let _ = std::fs::remove_dir_all(&t);
    }
    #[test]
    fn original_pruefen_mit_stempel() {
        let k = std::env::temp_dir().join(format!("pesm-original-{}", std::process::id()));
        let o = k.join("original");
        std::fs::create_dir_all(&o).unwrap();
        std::fs::write(o.join("a.bin"), b"abc").unwrap();
        let list = serde_json::json!({"a.bin": sha256_hex(b"abc")});
        std::fs::write(o.join("inhalt.json"), list.to_string()).unwrap();
        assert!(original_pruefen(&k).is_ok());
        assert!(o.join("geprueft.stamp").exists(), "the full pass leaves its stamp");
        // with the stamp only existence counts; a missing file is still caught
        std::fs::write(o.join("a.bin"), b"xyz").unwrap();
        assert!(original_pruefen(&k).is_ok());
        std::fs::remove_file(o.join("a.bin")).unwrap();
        assert!(original_pruefen(&k).is_err());
        // without the stamp the hashes count again
        std::fs::write(o.join("a.bin"), b"xyz").unwrap();
        std::fs::remove_file(o.join("geprueft.stamp")).unwrap();
        assert!(original_pruefen(&k).is_err());
        let _ = std::fs::remove_dir_all(&k);
    }
}
#[cfg(test)]
mod anweisungen_forschung {
    use super::*;
    /// Research (Fabian 23.11.: individual instructions): which bits of Tactics.bin and TacticsFormation.bin vary.
    /// cargo test --release anweisungen_forschung -- --ignored --nocapture
    #[test]
    #[ignore]
    fn bits_zeigen() {
        let pesdb = sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        let (_, t) = unpack(&std::fs::read(pesdb.join("Tactics.bin")).unwrap()).unwrap();
        let (_, f) = unpack(&std::fs::read(pesdb.join("TacticsFormation.bin")).unwrap()).unwrap();
        eprintln!("Tactics {} bytes ({} records), Formation {} bytes ({} records)", t.len(), t.len() / 12, f.len(), f.len() / 12);
        let mut bits = [0u32; 32];
        for o in (0..t.len() / 12).map(|i| i * 12) { let v = le32(&t, o + 8); for b in 0..32 { if v >> b & 1 == 1 { bits[b] += 1; } } }
        eprintln!("Tactics u32@8 bit set counts: {bits:?}");
        let mut b4 = [0u32; 32]; let mut b10 = [0u32; 16];
        for o in (0..f.len() / 12).map(|i| i * 12) {
            let v = le32(&f, o + 4); for b in 0..32 { if v >> b & 1 == 1 { b4[b] += 1; } }
            let w = u16::from_le_bytes([f[o + 10], f[o + 11]]); for b in 0..16 { if w >> b & 1 == 1 { b10[b] += 1; } }
        }
        eprintln!("Formation u32@4 bits: {b4:?}");
        eprintln!("Formation u16@10 bits: {b10:?}");
        // the first team's records in full
        let team = le32(&t, 4);
        for o in (0..t.len() / 12).map(|i| i * 12).filter(|&o| le32(&t, o + 4) == team) {
            let k = le32(&t, o);
            eprintln!("team {team} key {k} sliders {:032b}", le32(&t, o + 8));
            for p in (0..f.len() / 12).map(|i| i * 12).filter(|&p| le32(&f, p) == k) { eprintln!("  {:02x?}", &f[p..p + 12]); }
        }
    }
}
