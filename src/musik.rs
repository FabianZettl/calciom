//! Background music (Einstellungen → Musik): a radio stream URL or a YouTube playlist, played by mpv in the
//! background (mpv opens YouTube links through yt-dlp by itself). Paused while a match runs (FL27 or the simulation).
//! The URL lives in musik.txt beside klang.txt; empty means no music.
#[cfg(unix)]
use std::io::Write;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};

static URL: Mutex<Option<String>> = Mutex::new(None);
fn datei() -> std::path::PathBuf { crate::game::save_dir().join(crate::id("../musik.txt")) }
/// The stored URL (read once from musik.txt, then kept in memory).
pub fn url() -> String { URL.lock().unwrap().get_or_insert_with(|| std::fs::read_to_string(datei()).unwrap_or_default().trim().to_string()).clone() }
/// Settings: store a new URL (empty = no music).
pub fn setzen(u: &str) {
    let _ = std::fs::create_dir_all(crate::game::save_dir());
    let _ = std::fs::write(datei(), u);
    *URL.lock().unwrap() = Some(u.to_string());
}
/// A YouTube link with list= (also a "watch?v=…&list=…" copied from a playing video) becomes the playlist link,
/// so mpv plays the whole list and not only the one song.
pub fn abspiel_url(url: &str) -> String {
    if !ist_youtube(url) { return url.to_string(); }
    match url.split(['?', '&']).find_map(|t| t.strip_prefix(crate::id("list="))) {
        Some(id) if !id.is_empty() => format!("https://www.youtube.com/playlist?list={id}"),
        _ => url.to_string(),
    }
}
pub fn ist_youtube(url: &str) -> bool { url.contains(crate::id("youtube.com")) || url.contains(crate::id("youtu.be")) }
/// Whether mpv and yt-dlp are installed (looked up once).
pub fn programme() -> (bool, bool) {
    static P: OnceLock<(bool, bool)> = OnceLock::new();
    *P.get_or_init(|| {
        let hat = |p: &str| Command::new(p).arg(crate::id("--version")).stdout(Stdio::null()).stderr(Stdio::null()).status().map(|s| s.success()).unwrap_or(false);
        (hat(crate::id("mpv")), hat(crate::id("yt-dlp")))
    })
}

/// mpv volume at full music (0–100).
const VOLL: f32 = 45.0;
/// Seconds for a fade down or up.
const FADE: f32 = 2.5;

/// "Next song" pressed (options or the title popup); the next frame sends it to mpv.
pub static WEITER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn weiter() { WEITER.store(true, std::sync::atomic::Ordering::Relaxed); }

#[derive(Default)]
pub struct Musik {
    kind: Option<Child>,
    url: String,
    #[cfg(unix)]
    ipc: Option<std::io::BufReader<std::os::unix::net::UnixStream>>,
    /// Current loudness 0…1 of VOLL (faded towards 0 during a match), the last value sent to mpv, paused at 0.
    pegel: f32,
    gesendet: i32,
    pausiert: bool,
}

impl Musik {
    fn sock() -> std::path::PathBuf { std::env::temp_dir().join(format!("pesm-musik-{}.sock", std::process::id())) }

    /// Once per frame (dt in seconds): start or stop mpv for the stored URL, fade down and pause while a match runs,
    /// resume and fade up after it. Returns the new title when the song or stream title changes.
    pub fn takt(&mut self, url: &str, spiel_laeuft: bool, dt: f32) -> Option<String> {
        if url != self.url { self.stopp(); self.url = url.to_string(); }
        if self.url.is_empty() { return None; }
        if self.kind.as_mut().map(|k| k.try_wait().ok().flatten().is_some()).unwrap_or(false) { return None; } // mpv ended (bad URL): no restart loop
        if self.kind.is_none() {
            let mut cmd = Command::new(crate::id("mpv"));
            cmd.args([crate::id("--no-video"), crate::id("--really-quiet"), crate::id("--no-terminal"), crate::id("--loop-playlist=inf"), "--ytdl-format=bestaudio/best"])
                .arg(format!("--volume={}", if spiel_laeuft { 0.0 } else { VOLL }))
                // Fabian 23.11.: playlists in random order
                .args(ist_youtube(&self.url).then_some(crate::id("--shuffle")))
                .arg(format!("--input-ipc-server={}", Self::sock().display()))
                .arg(abspiel_url(&self.url)).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
            // mpv dies with PES M, also when the window is simply closed
            #[cfg(target_os = "linux")]
            unsafe { cmd.pre_exec(|| { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM); Ok(()) }); }
            self.kind = cmd.spawn().ok();
            self.pegel = if spiel_laeuft { 0.0 } else { 1.0 };
            self.gesendet = (self.pegel * VOLL) as i32;
            self.pausiert = false;
        }
        #[cfg(unix)]
        return self.steuern(spiel_laeuft, dt);
        #[cfg(not(unix))]
        { let _ = (spiel_laeuft, dt); None }
    }

    #[cfg(unix)]
    fn steuern(&mut self, spiel_laeuft: bool, dt: f32) -> Option<String> {
        use std::io::BufRead;
        if self.ipc.is_none() {
            // the socket is there a moment after the start; until then try again next frame
            let s = std::os::unix::net::UnixStream::connect(Self::sock()).ok()?;
            s.set_nonblocking(true).ok()?;
            let mut r = std::io::BufReader::new(s);
            let _ = writeln!(r.get_mut(), "{{\"command\":[\"observe_property\",1,\"media-title\"]}}");
            self.ipc = Some(r);
        }
        let ipc = self.ipc.as_mut()?;
        if WEITER.swap(false, std::sync::atomic::Ordering::Relaxed) { let _ = writeln!(ipc.get_mut(), "{{\"command\":[\"playlist-next\",\"force\"]}}"); }
        let ziel = if spiel_laeuft { 0.0 } else { 1.0 };
        if !spiel_laeuft && self.pausiert && writeln!(ipc.get_mut(), "{{\"command\":[\"set_property\",\"pause\",false]}}").is_ok() { self.pausiert = false; }
        let schritt = dt.min(0.1) / FADE;
        self.pegel = if self.pegel < ziel { (self.pegel + schritt).min(ziel) } else { (self.pegel - schritt).max(ziel) };
        let vol = (self.pegel * VOLL).round() as i32;
        if vol != self.gesendet && writeln!(ipc.get_mut(), "{{\"command\":[\"set_property\",\"volume\",{}]}}", vol).is_ok() { self.gesendet = vol; }
        if spiel_laeuft && self.pegel <= 0.0 && !self.pausiert && writeln!(ipc.get_mut(), "{{\"command\":[\"set_property\",\"pause\",true]}}").is_ok() { self.pausiert = true; }
        // events from mpv: the title of the song (YouTube) or of the stream (radio's ICY title)
        let mut titel = None;
        let mut zeile = String::new();
        loop {
            zeile.clear();
            match ipc.read_line(&mut zeile) {
                Ok(0) => { self.ipc = None; break; }
                Ok(_) => if let Some(t) = titel_aus(&zeile) { titel = Some(t); },
                Err(_) => break, // WouldBlock: nothing more this frame
            }
        }
        titel
    }

    fn stopp(&mut self) {
        #[cfg(unix)]
        { self.ipc = None; }
        if let Some(mut k) = self.kind.take() { let _ = k.kill(); let _ = k.wait(); }
        let _ = std::fs::remove_file(Self::sock());
    }
}

/// The new title from one line of mpv's IPC output (a media-title change), None for anything else.
fn titel_aus(zeile: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(zeile).ok()?;
    if v["event"] != "property-change" || v["name"] != "media-title" { return None; }
    let t = v["data"].as_str()?.trim();
    (!t.is_empty() && !t.starts_with(crate::id("http")) && !t.contains(crate::id("watch?v="))).then(|| t.to_string())
}

impl Drop for Musik { fn drop(&mut self) { self.stopp(); } }

#[cfg(test)]
mod tests {
    #[test]
    fn playlist_aus_link() {
        assert_eq!(super::abspiel_url("https://www.youtube.com/watch?v=abc&list=PL123&index=4"), "https://www.youtube.com/playlist?list=PL123");
        assert_eq!(super::abspiel_url("https://youtu.be/abc?list=PL9"), "https://www.youtube.com/playlist?list=PL9");
        assert_eq!(super::abspiel_url("https://www.youtube.com/watch?v=abc"), "https://www.youtube.com/watch?v=abc");
        assert_eq!(super::abspiel_url("https://radio.example/stream?list=x"), "https://radio.example/stream?list=x");
    }

    #[test]
    fn titel_aus_mpv_zeile() {
        assert_eq!(super::titel_aus(r#"{"event":"property-change","id":1,"name":"media-title","data":"Song - Band"}"#).as_deref(), Some("Song - Band"));
        assert_eq!(super::titel_aus(r#"{"event":"property-change","id":1,"name":"media-title","data":"https://x/stream.mp3"}"#), None);
        assert_eq!(super::titel_aus(r#"{"data":true,"request_id":0,"error":"success"}"#), None);
    }

    /// Real mpv (silent through MPV_HOME with ao=null): title event, fade down to pause, resume. cargo test -- --ignored musik
    #[test]
    #[ignore]
    fn fade_und_pause_mit_mpv() {
        let d = std::env::temp_dir().join(format!("pesm-mpv-test-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("mpv.conf"), "ao=null\n").unwrap();
        std::env::set_var("MPV_HOME", &d);
        let datei = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/sfx/titel.wav");
        let mut m = super::Musik::default();
        let mut titel = None;
        let mut frames = |m: &mut super::Musik, spiel: bool, n: usize, titel: &mut Option<String>| for _ in 0..n {
            if let Some(t) = m.takt(datei.to_str().unwrap(), spiel, 0.05) { *titel = Some(t); }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        frames(&mut m, false, 40, &mut titel);
        assert_eq!(titel.as_deref(), Some("titel.wav"));
        assert_eq!(m.gesendet, 45);
        frames(&mut m, true, 30, &mut titel); // 1.5 s of 2.5: still fading
        assert!(m.gesendet > 0 && m.gesendet < 45 && !m.pausiert, "{}", m.gesendet);
        frames(&mut m, true, 30, &mut titel);
        assert!(m.gesendet == 0 && m.pausiert);
        frames(&mut m, false, 60, &mut titel);
        assert!(m.gesendet == 45 && !m.pausiert);
    }
}
