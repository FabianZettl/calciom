//! AI pass photo for players promoted from the own academy (Fabian 26.11., concept "Passfotos für erfundene Spieler").
//! The drawn pass photo (passfoto.rs, skin, hair and beard from the in-game face) is the template; stable-diffusion.cpp
//! paints it as a real-looking photo (img2img). The result is a 128×128 DXT5 portrait like FL27's own: kept in the
//! career (passfotos/<id>.dds) and copied into the career's livecpk (common/render/symbol/player/<id>.dds), where FL27
//! and CalcioM's photo index both find it. Program and model are installed by hand in <data folder>/kifoto/.
use crate::game::Career;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};

/// The studio grey behind the template; the background the finished photo is cut free from.
const GRUND: [u8; 3] = [200, 202, 206];
/// Size the model paints at, and FL27's portrait size.
const MALEN: u32 = 768;
const PORTRAET: u32 = 128;
/// How far the model may stray from the template (0 keeps it, 1 ignores it).
const STAERKE: &str = "0.75"; // probes 26.11.: 0.6 stays a drawing, 0.75 a photo that keeps skin, hair and beard
const NEGATIV: &str = "cartoon, 3d render, drawing, painting, toy, plastic, text, watermark, blurry, deformed"; // roh

pub fn ordner() -> PathBuf { crate::fl27::datenordner().join(crate::id("kifoto")) }
/// The program (sd-cli) and the model (first .safetensors/.gguf in the folder), when both are there.
pub fn werkzeug() -> Option<(PathBuf, PathBuf)> {
    let o = ordner();
    let sd = [crate::id("sd-cli"), crate::id("sd-cli.exe")].iter().map(|n| o.join(n)).find(|p| p.is_file())?;
    let mut modelle: Vec<PathBuf> = std::fs::read_dir(&o).ok()?.flatten().map(|e| e.path())
        .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("safetensors" | "gguf"))).collect();
    modelle.sort();
    Some((sd, modelle.into_iter().next()?))
}
/// The one-off download (Einstellungen › Features, optional): the image program from its GitHub release and a smaller
/// copy of SDXL-Lightning (ByteDance, openrail++) by the Hugging Face user mzwing (Fabian 26.11.; checked 26.11.: it
/// paints the same faces as our own q4_0 of the original). Both checked by SHA-256; change source and sum together.
pub const MODELL_URL: &str = "https://huggingface.co/mzwing/SDXL-Lightning-GGUF/resolve/main/sdxl_lightning_4step.q4_0.gguf"; // roh
const MODELL_SHA: &str = "33cc25b6f5dd121ecf3ad28e35eb38cb21427b7794c53527ceafe2dc3b109f4f"; // roh
const MODELL_DATEI: &str = "sdxl_lightning_4step.q4_0.gguf"; // roh
pub const MODELL_GROESSE: u64 = 2_584_858_432;
#[cfg(windows)]
const PROGRAMM: (&str, &str) = ("https://github.com/leejet/stable-diffusion.cpp/releases/download/master-1079-a3c49b1/sd-master-a3c49b1-bin-win-vulkan-x64.zip", "60991b744e21add68625c27386e3a6dbcfc72929a3101880cb4863f5f1e6c013"); // roh
#[cfg(not(windows))]
const PROGRAMM: (&str, &str) = ("https://github.com/leejet/stable-diffusion.cpp/releases/download/master-1079-a3c49b1/sd-master-a3c49b1-bin-Linux-Ubuntu-26.04-x86_64-vulkan.zip", "a7353599e38047bc52294e15a75d8443fbd0f95159e6a10d02c5c52b3104dbff"); // roh

/// The running download: None idle, Some("") busy, Some(text) the error of the last try.
static LADEN: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
pub fn laedt() -> bool { LADEN.lock().unwrap().as_deref() == Some("") }
pub fn ladefehler() -> Option<String> { LADEN.lock().unwrap().clone().filter(|e| !e.is_empty()) }
/// Bytes of the model downloaded so far.
pub fn geladen() -> u64 { std::fs::metadata(ordner().join(format!("{MODELL_DATEI}.part"))).map_or(0, |m| m.len()) }
/// Program and model into the kifoto folder, in the background. A broken download resumes on the next try.
pub fn installieren() {
    if laedt() { return; }
    *LADEN.lock().unwrap() = Some(String::new());
    std::thread::spawn(|| {
        let r = (|| -> Result<(), String> {
            let o = ordner();
            std::fs::create_dir_all(&o).map_err(|e| e.to_string())?;
            let laden = |url: &str, ziel: &Path| -> Result<(), String> {
                let ok = std::process::Command::new("curl").args(["-fsSL", "--retry", "3", "-C", "-", "-o"]).arg(ziel).arg(url).status().map_err(|e| format!("curl: {e}"))?; // roh
                if ok.success() { Ok(()) } else { Err(format!("Download fehlgeschlagen: {url}")) } // roh
            };
            if !o.join(crate::id("sd-cli")).is_file() && !o.join(crate::id("sd-cli.exe")).is_file() {
                let zip = o.join(crate::id("programm.zip"));
                let _ = std::fs::remove_file(&zip);
                laden(PROGRAMM.0, &zip)?;
                if crate::fl27::sha_file(&zip)? != PROGRAMM.1 { let _ = std::fs::remove_file(&zip); return Err("Prüfsumme des Programms stimmt nicht".into()); } // roh
                // shortcut: unzip / bsdtar (Linux) and tar (Windows 10+) unpack the zip; a zip crate if a system lacks both
                let aus = if cfg!(windows) { std::process::Command::new("tar").arg("-xf").arg(&zip).arg("-C").arg(&o).status() } // roh
                    else { std::process::Command::new("unzip").args(["-o", "-q"]).arg(&zip).arg("-d").arg(&o).status().or_else(|_| std::process::Command::new("bsdtar").arg("-xf").arg(&zip).arg("-C").arg(&o).status()) }; // roh
                if !aus.map_or(false, |s| s.success()) { return Err("Programm konnte nicht entpackt werden".into()); } // roh
                let _ = std::fs::remove_file(&zip);
            }
            let ziel = o.join(MODELL_DATEI);
            if !ziel.is_file() {
                let teil = o.join(format!("{MODELL_DATEI}.part"));
                laden(MODELL_URL, &teil)?;
                if sha_datei(&teil)? != MODELL_SHA { let _ = std::fs::remove_file(&teil); return Err("Prüfsumme des Modells stimmt nicht".into()); } // roh
                std::fs::rename(&teil, &ziel).map_err(|e| e.to_string())?;
            }
            Ok(())
        })();
        if let Err(e) = &r { crate::fl27::log(&format!("KI-Passfotos: {e}")); } // roh
        *LADEN.lock().unwrap() = r.err();
    });
}
/// SHA-256 of a big file without reading it into memory.
fn sha_datei(p: &Path) -> Result<String, String> {
    use sha2::Digest;
    use std::io::Read;
    let mut f = std::fs::File::open(p).map_err(|e| e.to_string())?;
    let (mut h, mut puffer) = (sha2::Sha256::new(), vec![0u8; 1 << 20]);
    loop { let n = f.read(&mut puffer).map_err(|e| e.to_string())?; if n == 0 { break; } h.update(&puffer[..n]); }
    Ok(h.finalize().iter().map(|x| format!("{x:02x}")).collect())
}
/// Program and model off the computer again.
pub fn entfernen() { let _ = std::fs::remove_dir_all(ordner()); }

fn karriere_datei(c: &Career, id: u32) -> PathBuf { c.karriere_dir().join(crate::id("passfotos")).join(format!("{id}.dds")) }
fn spiel_datei(cpk: &str, id: u32) -> PathBuf { crate::fl27::sider().join(crate::id("livecpk")).join(cpk).join(crate::id("common/render/symbol/player")).join(format!("{id}.dds")) }
/// Does this player have an AI photo of his own?
pub fn hat_foto(c: &Career, id: u32) -> bool { karriere_datei(c, id).is_file() }

/// The career's AI photos into its livecpk folder (missing or changed ones), when the career is loaded into the game.
pub fn ins_spiel(karriere: &Path, cpk: &str) {
    for e in std::fs::read_dir(karriere.join(crate::id("passfotos"))).into_iter().flatten().flatten() {
        let Ok(b) = std::fs::read(e.path()) else { continue };
        let ziel = spiel_datei(cpk, 0).with_file_name(e.file_name());
        if std::fs::read(&ziel).ok().as_deref() != Some(&b[..]) {
            let _ = std::fs::create_dir_all(ziel.parent().unwrap());
            let _ = crate::fl27::atomic_write(&ziel, &b);
        }
    }
}

thread_local! { static LAUF: std::cell::RefCell<Option<(u32, Receiver<Result<Vec<u8>, String>>)>> = const { std::cell::RefCell::new(None) }; }

/// Once per frame, after passfoto::vorbereiten: takes the next waiting player, or picks up a finished photo.
/// Returns the id and the game file of a photo that just got finished (the UI then shows it).
pub fn takt(c: &mut Career) -> Option<(u32, PathBuf)> {
    LAUF.with(|lauf| {
        let mut lauf = lauf.borrow_mut();
        if let Some((id, rx)) = lauf.as_ref() {
            let id = *id;
            let erg = match rx.try_recv() { Err(TryRecvError::Empty) => return None, Ok(e) => e, Err(TryRecvError::Disconnected) => Err("abgebrochen".into()) }; // roh
            *lauf = None;
            c.ki_fotos.retain(|&x| x != id); // a failed photo is not tried again and again; "New photo" asks again
            return match erg.and_then(|dds| {
                let k = karriere_datei(c, id);
                std::fs::create_dir_all(k.parent().unwrap()).map_err(|e| e.to_string())?;
                crate::fl27::atomic_write(&k, &dds)?;
                let s = spiel_datei(&c.fl27_cpk(), id);
                std::fs::create_dir_all(s.parent().unwrap()).map_err(|e| e.to_string())?;
                crate::fl27::atomic_write(&s, &dds)?;
                Ok(s)
            }) {
                Ok(s) => { crate::fl27::log(&format!("KI-Passfoto fertig: {id}")); Some((id, s)) } // roh
                Err(e) => { crate::fl27::log(&format!("KI-Passfoto {id} fehlgeschlagen: {e}")); None } // roh
            };
        }
        let &id = c.ki_fotos.first()?;
        let (sd, modell) = werkzeug()?;
        let Some(p) = c.world.players.iter().position(|p| p.id == id) else { c.ki_fotos.remove(0); return None };
        let pl = &c.world.players[p];
        let club = if pl.team >= 0 { pl.team as usize } else { c.own };
        let (k1, k2) = crate::pages::club_color(c, club);
        let vorlage = crate::passfoto::vorlage(id, k1, k2, GRUND)?; // rendered next frame when not ready
        // shortcut: the nation in the UI language (German names mostly work for the model), English names would fit it better
        // two years older than he is: the model paints teenagers too young (Fabian 26.11.: 16-year-olds looked 12)
        let prompt = format!("professional studio headshot photo of a {} year old football player from {}, {}, front view, head and shoulders, wearing a plain football shirt without logos, plain light grey background, sharp focus, natural skin, realistic",
            pl.age + 2, crate::nationen::name_anzeige(&c.world, pl.nat), beschreibung(crate::gesichter::aussehen_von(id))); // roh
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(id as u64, |d| d.as_nanos() as u64 % 1_000_000_007); // "New photo" gives a new face
        let (tx, rx) = channel();
        let ordner = std::env::temp_dir().join(format!("calciom-kifoto-{id}"));
        std::thread::spawn(move || { let _ = tx.send(malen(&sd, &modell, vorlage, &prompt, seed, &ordner)); });
        *lauf = Some((id, rx));
        None
    })
}

/// Skin, hair and beard of the in-game face in words: the template alone loses them at the strength a photo needs
/// (probe 26.11.: a dark-skinned, bearded template came out lighter and shaven).
fn beschreibung(l: Option<crate::gesichter::Aussehen>) -> String {
    let Some(l) = l else { return "short hair, clean shaven".into() }; // roh
    let haut = ["very fair skin", "fair skin", "light olive skin", "olive skin", "brown skin", "dark brown skin"][(l.haut.clamp(1, 6) - 1) as usize]; // roh
    let farbe = ["black", "dark brown", "brown", "auburn", "light brown", "dark blond", "blond"][(l.haar_stufe.clamp(1, 7) - 1) as usize]; // roh
    let haar = match l.stil { 0 => "shaved head".to_string(), 4 => format!("short {farbe} afro hair"), _ => format!("short {farbe} hair") }; // roh
    format!("{haut}, {haar}, {}", if l.bart { "full beard" } else { "clean shaven" }) // roh
}

/// One photo: template → model (img2img) → cut free from the grey → 128×128 DXT5 DDS.
fn malen(sd: &Path, modell: &Path, vorlage: image::RgbaImage, prompt: &str, seed: u64, ordner: &Path) -> Result<Vec<u8>, String> {
    std::fs::create_dir_all(ordner).map_err(|e| e.to_string())?;
    // the template square (the photo's shape is 194×219), studio grey around it
    let seite = vorlage.width().max(vorlage.height());
    let mut quadrat = image::RgbaImage::from_pixel(seite, seite, image::Rgba([GRUND[0], GRUND[1], GRUND[2], 255]));
    image::imageops::overlay(&mut quadrat, &vorlage, ((seite - vorlage.width()) / 2) as i64, (seite - vorlage.height()) as i64);
    // head and shoulders only (Fabian 26.11.: closer, from the shoulders up): the upper middle of the square, so the
    // model also paints the face bigger and sharper
    let a = seite * 70 / 100;
    let ausschnitt = image::imageops::crop_imm(&quadrat, (seite - a) / 2, seite * 5 / 100, a, a).to_image();
    let gross = image::imageops::resize(&ausschnitt, MALEN, MALEN, image::imageops::FilterType::CatmullRom);
    let (ein, aus) = (ordner.join("vorlage.png"), ordner.join("foto.png")); // roh
    gross.save(&ein).map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(sd);
    cmd.current_dir(sd.parent().unwrap_or(Path::new(".")))
        .arg("-m").arg(modell).arg("--init-img").arg(&ein).arg("-o").arg(&aus) // roh
        .args(["-p", prompt, "-n", NEGATIV]) // roh
        .args(["--strength", STAERKE, "--steps", "4", "--cfg-scale", "1", "--sampling-method", "euler", "-W"]).arg(MALEN.to_string()).arg("-H").arg(MALEN.to_string()) // roh
        .arg("--seed").arg(seed.to_string()); // roh
    #[cfg(unix)]
    cmd.env("LD_LIBRARY_PATH", sd.parent().unwrap_or(Path::new("."))); // the release zip keeps its libraries next to the program
    let out = cmd.output().map_err(|e| format!("{}: {e}", sd.display()))?;
    if !out.status.success() || !aus.is_file() { return Err(String::from_utf8_lossy(&out.stderr).lines().rev().take(3).collect::<Vec<_>>().join(" | ")); }
    let foto = image::open(&aus).map_err(|e| e.to_string())?.to_rgba8();
    let _ = std::fs::remove_dir_all(ordner);
    Ok(dds_dxt5(&freistellen(&image::imageops::resize(&foto, PORTRAET, PORTRAET, image::imageops::FilterType::Lanczos3))))
}

/// The background cut free like FL27's portraits: the studio grey reached from the top and the sides becomes
/// transparent. The model paints it as a soft gradient, so the fill goes step by step (each pixel grey and close to the
/// one it came from), not against one colour. Grey inside the player (a grey kit) stays: it is not reached.
fn freistellen(b: &image::RgbaImage) -> image::RgbaImage {
    let (w, h) = b.dimensions();
    let grau = |p: &image::Rgba<u8>| { let (mx, mn) = (p.0[..3].iter().max().unwrap(), p.0[..3].iter().min().unwrap()); mx - mn <= 22 && *mn >= 80 };
    let nah = |p: &image::Rgba<u8>, q: &image::Rgba<u8>| (0..3).map(|i| (p.0[i] as i32 - q.0[i] as i32).abs()).sum::<i32>() <= 18;
    let mut draussen = vec![false; (w * h) as usize];
    // not from the bottom edge: the shirt runs into it
    let mut stapel: Vec<(u32, u32, image::Rgba<u8>)> = (0..w).map(|x| (x, 0)).chain((0..h).flat_map(|y| [(0, y), (w - 1, y)]))
        .map(|(x, y)| (x, y, *b.get_pixel(x, y))).collect();
    while let Some((x, y, von)) = stapel.pop() {
        let i = (y * w + x) as usize;
        let p = b.get_pixel(x, y);
        if draussen[i] || !grau(p) || !nah(p, &von) { continue; }
        draussen[i] = true;
        if x > 0 { stapel.push((x - 1, y, *p)); }
        if y > 0 { stapel.push((x, y - 1, *p)); }
        if x + 1 < w { stapel.push((x + 1, y, *p)); }
        if y + 1 < h { stapel.push((x, y + 1, *p)); }
    }
    image::RgbaImage::from_fn(w, h, |x, y| {
        let mut p = *b.get_pixel(x, y);
        if draussen[(y * w + x) as usize] { p.0[3] = 0; return p; }
        // a soft edge next to the background
        let rand = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h && draussen[(ny as u32 * w + nx as u32) as usize]
        });
        if rand { p.0[3] = 170; }
        p
    })
}

/// A DDS file with one DXT5 picture, no mipmaps (FL27's portraits: 128×128, 16 512 bytes).
#[allow(deprecated)]
pub fn dds_dxt5(b: &image::RgbaImage) -> Vec<u8> {
    let (w, h) = b.dimensions();
    let mut d = Vec::with_capacity(128 + (w * h) as usize);
    let u = |d: &mut Vec<u8>, v: u32| d.extend_from_slice(&v.to_le_bytes());
    d.extend_from_slice(b"DDS ");
    u(&mut d, 124);
    u(&mut d, 0x1 | 0x2 | 0x4 | 0x1000 | 0x80000); // caps, height, width, pixel format, linear size
    u(&mut d, h);
    u(&mut d, w);
    u(&mut d, w * h); // DXT5: one byte per pixel
    u(&mut d, 0); // depth
    u(&mut d, 0); // mipmaps
    for _ in 0..11 { u(&mut d, 0); }
    u(&mut d, 32);
    u(&mut d, 0x4); // pixel format: FourCC
    d.extend_from_slice(b"DXT5");
    for _ in 0..5 { u(&mut d, 0); }
    u(&mut d, 0x1000); // caps: texture
    for _ in 0..4 { u(&mut d, 0); }
    image::codecs::dxt::DxtEncoder::new(&mut d).encode(b.as_raw(), w, h, image::codecs::dxt::DxtVariant::DXT5).expect("width and height are multiples of 4");
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Cut free real model outputs: `KIFOTO_PROBE=<folder> cargo test --release kifoto_probe -- --ignored` turns every
    /// out*.png there into <name>.dds and <name>.cut.png.
    #[test]
    #[ignore]
    fn kifoto_probe() {
        let d = std::path::PathBuf::from(std::env::var("KIFOTO_PROBE").unwrap());
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if !n.starts_with("out") || !n.ends_with(".png") || n.ends_with(".cut.png") { continue; }
            let f = image::open(e.path()).unwrap().to_rgba8();
            let k = freistellen(&image::imageops::resize(&f, PORTRAET, PORTRAET, image::imageops::FilterType::Lanczos3));
            std::fs::write(d.join(n.replace(".png", ".dds")), dds_dxt5(&k)).unwrap();
            k.save(d.join(n.replace(".png", ".cut.png"))).unwrap();
        }
    }
    /// The real download into CALCIOM_KARRIEREN/kifoto (network, 2.7 GB): `cargo test --release kifoto_download -- --ignored`.
    #[test]
    #[ignore]
    fn kifoto_download() {
        installieren();
        while laedt() { std::thread::sleep(std::time::Duration::from_millis(500)); }
        assert_eq!(ladefehler(), None);
        assert!(werkzeug().is_some());
    }
    #[test]
    fn pruefsumme_einer_grossen_datei() {
        let d = std::env::temp_dir().join(format!("kifoto-sha-{}", std::process::id()));
        let b: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(&d, &b).unwrap();
        assert_eq!(sha_datei(&d).unwrap(), crate::fl27::sha256_hex(&b));
        let _ = std::fs::remove_file(&d);
    }
    #[test]
    fn beschreibung_aus_dem_gesicht() {
        let l = crate::gesichter::Aussehen { haut: 6, haar: [5, 4, 3], haar_stufe: 1, stil: 4, bart: true };
        assert_eq!(beschreibung(Some(l)), "dark brown skin, short black afro hair, full beard");
        assert_eq!(beschreibung(Some(crate::gesichter::Aussehen { haut: 1, haar_stufe: 7, stil: 1, bart: false, ..l })), "very fair skin, short blond hair, clean shaven");
        assert_eq!(beschreibung(Some(crate::gesichter::Aussehen { stil: 0, ..l })), "dark brown skin, shaved head, full beard");
    }
    #[test]
    fn portraet_wie_im_spiel() {
        // a dark "player" in the middle of the grey: the grey around becomes transparent, the player and the grey shirt
        // inside him stay
        let b = image::RgbaImage::from_fn(PORTRAET, PORTRAET, |x, y| {
            let mitte = (40..88).contains(&x) && y >= 30;
            let hemd = (56..72).contains(&x) && y >= 90;
            if hemd { image::Rgba([GRUND[0], GRUND[1], GRUND[2], 255]) } else if mitte { image::Rgba([90, 60, 40, 255]) } else { image::Rgba([GRUND[0], GRUND[1], GRUND[2], 255]) }
        });
        let f = freistellen(&b);
        assert_eq!(f.get_pixel(5, 5).0[3], 0);
        assert_eq!(f.get_pixel(64, 50).0[3], 255);
        assert_eq!(f.get_pixel(64, 100).0[3], 255, "grey inside the player stays");
        let d = dds_dxt5(&f);
        assert_eq!(d.len(), 16_512, "the size of FL27's own portraits");
        // CalcioM reads it back like every other portrait
        let z = image::load_from_memory_with_format(&d, image::ImageFormat::Dds).unwrap().to_rgba8();
        assert_eq!(z.dimensions(), (PORTRAET, PORTRAET));
        assert!(z.get_pixel(5, 5).0[3] < 20 && z.get_pixel(64, 50).0[3] > 230);
    }
}
