//! Handy-Ansicht (konzepte/mobile-ansicht.md, 23.11.2026): the career from a phone browser in the home network.
//! A small web server in its own thread (axum on tokio) hands every request to the main loop through a channel; the
//! main loop answers with the same functions the PC buttons use (handy/ansicht.rs builds the views). Safety (Teil 4):
//! private sender addresses only, host and origin checked, pairing with a 6-digit code, one random key per phone as an
//! HttpOnly cookie, only its SHA-256 stored, at most three phones, a log of every command.
pub mod ansicht;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Path, Request, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn r(k: &str) -> f32 { crate::regeln::f("handy", k) }

/// What the server asks of the main loop: a page view or a command; the answer goes back on `antwort`.
pub enum Auftrag { Seite(String), Befehl(Value), Bild(String, u32) }
/// The main loop's answer: JSON, or the bytes of a PNG picture.
pub enum Antwort { Json(Value), Png(Vec<u8>) }
pub struct Bote { pub auftrag: Auftrag, pub antwort: tokio::sync::oneshot::Sender<Antwort> }

/// The pairing state the PC shows (code, valid until) and the server checks.
pub struct Kopplung { pub code: String, pub bis: Instant, pub fehler: u32, pub sperre_bis: Option<Instant>, pub aus: bool }
impl Kopplung {
    fn neu() -> Kopplung { Kopplung { code: neuer_code(), bis: Instant::now() + Duration::from_secs(r("kopplung.gueltig_s") as u64), fehler: 0, sperre_bis: None, aus: false } }
    /// The code for the PC: a fresh one when the old one ran out.
    pub fn aktuell(&mut self) -> String { if Instant::now() >= self.bis { self.code = neuer_code(); self.bis = Instant::now() + Duration::from_secs(r("kopplung.gueltig_s") as u64); } self.code.clone() }
}
fn neuer_code() -> String {
    // PESM_HANDY_CODE: a fixed code for the automatic phone test, never set in normal play
    if let Ok(c) = std::env::var("PESM_HANDY_CODE") { return c; }
    let n: u32 = rand::random::<u32>() % 1_000_000; format!("{n:06}")
}

/// A paired phone: hash of its key, a name, when it was paired and last seen (seconds since 1970).
#[derive(Serialize, Deserialize, Clone)]
pub struct Geraet { pub hash: String, pub name: String, pub seit: u64, pub zuletzt: u64 }

fn jetzt() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }
fn hash(s: &str) -> String { use sha2::{Digest, Sha256}; Sha256::digest(s.as_bytes()).iter().map(|b| format!("{b:02x}")).collect() }
/// The folder of the device list and the log: next to the careers (PESM_HANDY_DIR for the test only).
fn ordner() -> std::path::PathBuf { std::env::var("PESM_HANDY_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| crate::game::save_dir()) }
fn geraete_pfad() -> std::path::PathBuf { ordner().join("handy_geraete.json") }
pub fn geraete() -> Vec<Geraet> {
    let max_alter = (r("geraete.abmelden_tage") * 86400.0) as u64;
    let v: Vec<Geraet> = std::fs::read_to_string(geraete_pfad()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    v.into_iter().filter(|g| jetzt().saturating_sub(g.zuletzt) < max_alter).collect()
}
pub fn geraete_speichern(v: &[Geraet]) { let _ = std::fs::create_dir_all(ordner()); let _ = std::fs::write(geraete_pfad(), serde_json::to_string_pretty(v).unwrap_or_default()); }
fn log(zeile: &str) {
    let p = ordner().join("handy.log");
    let alt = std::fs::read_to_string(&p).unwrap_or_default();
    let mut z: Vec<&str> = alt.lines().collect();
    let neu = format!("{} {zeile}", jetzt());
    z.push(&neu);
    let n = z.len().saturating_sub(r("log.zeilen") as usize);
    let _ = std::fs::write(&p, z[n..].join("\n") + "\n");
}

/// Private addresses only (Teil 4.1): 10/8, 172.16/12, 192.168/16, 169.254/16, 127/8, fc00::/7, fe80::/10, ::1.
pub fn privat(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(a) => a.is_private() || a.is_loopback() || a.is_link_local(),
        IpAddr::V6(a) => { let s = a.segments()[0]; a.is_loopback() || (s & 0xfe00) == 0xfc00 || (s & 0xffc0) == 0xfe80 || a.to_ipv4_mapped().map_or(false, |v| v.is_private() || v.is_loopback()) }
    }
}
/// The host a request names must be this PC (its own addresses, localhost), never a foreign name (DNS rebinding).
fn host_ok(host: &str, eigene: &[String]) -> bool {
    let h = if let Some(r) = host.strip_prefix('[') { r.split(']').next().unwrap_or("") } else { host.rsplit_once(':').filter(|(_, p)| p.chars().all(|c| c.is_ascii_digit())).map(|x| x.0).unwrap_or(host) };
    h == "localhost" || h == "127.0.0.1" || h == "::1" || eigene.iter().any(|e| e == h) // roh
}

#[derive(Clone)]
struct Zustand { bote: Arc<Mutex<mpsc::Sender<Bote>>>, stand: Arc<AtomicU64>, kopplung: Arc<Mutex<Kopplung>>, eigene: Arc<Vec<String>>, zuletzt: Arc<AtomicU64> }

/// The running server as the main loop holds it.
pub struct Handy { pub rx: mpsc::Receiver<Bote>, pub stand: Arc<AtomicU64>, pub kopplung: Arc<Mutex<Kopplung>>, pub zuletzt: Arc<AtomicU64>, pub adresse: String, stop: Option<tokio::sync::oneshot::Sender<()>> }
impl Drop for Handy { fn drop(&mut self) { if let Some(s) = self.stop.take() { let _ = s.send(()); } } }
impl Handy {
    /// A phone asked something within the last minute.
    pub fn verbunden(&self) -> bool { jetzt().saturating_sub(self.zuletzt.load(Ordering::Relaxed)) < 60 }
    pub fn code(&self) -> String { self.kopplung.lock().map(|mut k| k.aktuell()).unwrap_or_default() }
}

pub fn port() -> u16 { std::env::var("PESM_HANDY_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(r("server.port") as u16) }
fn schalter() -> std::path::PathBuf { crate::game::save_dir().join("../handy.txt") } // roh
/// "Handy-Zugang" in the settings (off unless switched on).
pub fn an() -> bool { std::fs::read_to_string(schalter()).map_or(false, |s| s.trim() == "an") } // roh
pub fn setzen(an: bool) { let _ = std::fs::create_dir_all(crate::game::save_dir()); let _ = std::fs::write(schalter(), if an { "an" } else { "aus" }); FEHLER.store(false, Ordering::Relaxed); } // roh
static FEHLER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn start_fehlgeschlagen() -> bool { FEHLER.load(Ordering::Relaxed) }
pub fn merke_fehler() { FEHLER.store(true, Ordering::Relaxed); }
static FINGER: AtomicU64 = AtomicU64::new(0);
/// What the PC shows of the running server (address, current code, a phone seen lately), set by the main loop each frame.
static INFO: Mutex<Option<(String, String, bool)>> = Mutex::new(None);
pub fn info_setzen(i: Option<(String, String, bool)>) { if let Ok(mut x) = INFO.lock() { *x = i; } }
pub fn info() -> Option<(String, String, bool)> { INFO.lock().ok().and_then(|x| x.clone()) }
/// Whether the career changed since the last frame (day, messages), for the phone's nudge.
pub fn fingerabdruck_neu(fp: u64) -> bool { FINGER.swap(fp, Ordering::Relaxed) != fp }
/// Starts the server in its own thread; Err when the port is taken.
pub fn starten() -> Result<Handy, String> {
    let (tx, rx) = mpsc::channel::<Bote>();
    let stand = Arc::new(AtomicU64::new(1));
    let kopplung = Arc::new(Mutex::new(Kopplung::neu()));
    let zuletzt = Arc::new(AtomicU64::new(0));
    let mut eigene: Vec<String> = local_ip_address::list_afinet_netifas().map(|v| v.into_iter().map(|x| x.1.to_string()).collect()).unwrap_or_default();
    if let Ok(ip) = local_ip_address::local_ip() { eigene.push(ip.to_string()); }
    let adresse = format!("http://{}:{}/", local_ip_address::local_ip().map(|i| i.to_string()).unwrap_or_else(|_| "127.0.0.1".into()), port()); // roh
    let z = Zustand { bote: Arc::new(Mutex::new(tx)), stand: stand.clone(), kopplung: kopplung.clone(), eigene: Arc::new(eigene), zuletzt: zuletzt.clone() };
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let listener = std::net::TcpListener::bind(("0.0.0.0", port())).map_err(|e| e.to_string())?; // roh
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let Ok(rt) = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build() else { return };
        rt.block_on(async move {
            let Ok(l) = tokio::net::TcpListener::from_std(listener) else { return };
            let app = Router::new()
                .route("/", get(|| async { Html(include_str!("../../web/handy.html")) }))
                .route("/koppeln", post(koppeln))
                .route("/schrift/kopf.ttf", get(|| async { ([(header::CONTENT_TYPE, "font/ttf")], &include_bytes!("../../assets/fonts/BarlowCondensed-ExtraBoldItalic.ttf")[..]) })) // roh
                .route("/schrift/text.ttf", get(|| async { ([(header::CONTENT_TYPE, "font/ttf")], &include_bytes!("../../assets/fonts/BarlowSemiCondensed-Medium.ttf")[..]) })) // roh
                .route("/schrift/halb.ttf", get(|| async { ([(header::CONTENT_TYPE, "font/ttf")], &include_bytes!("../../assets/fonts/BarlowSemiCondensed-SemiBold.ttf")[..]) })) // roh
                .route("/api/stand", get(|s: State<Zustand>, h: HeaderMap| async move { mit_geraet(&s, &h, Auftrag::Seite("stand".into())).await })) // roh
                .route("/api/seite/{name}", get(|s: State<Zustand>, h: HeaderMap, Path(n): Path<String>| async move { mit_geraet(&s, &h, Auftrag::Seite(n)).await }))
                .route("/api/befehl", post(|s: State<Zustand>, h: HeaderMap, Json(b): Json<Value>| async move { mit_geraet(&s, &h, Auftrag::Befehl(b)).await }))
                .route("/api/ws", get(ws))
                .route("/img/{art}/{id}", get(|s: State<Zustand>, h: HeaderMap, Path((a, i)): Path<(String, u32)>| async move { mit_geraet(&s, &h, Auftrag::Bild(a, i)).await }))
                .layer(middleware::from_fn_with_state(z.clone(), wache))
                .with_state(z);
            let _ = axum::serve(l, app.into_make_service_with_connect_info::<SocketAddr>()).with_graceful_shutdown(async { let _ = stop_rx.await; }).await;
        });
    });
    Ok(Handy { rx, stand, kopplung, zuletzt, adresse, stop: Some(stop_tx) })
}

/// The guard in front of everything: private sender, our own host name, and for POST and WebSocket the same origin.
async fn wache(State(z): State<Zustand>, ConnectInfo(von): ConnectInfo<SocketAddr>, req: Request, next: Next) -> Response {
    if !privat(von.ip()) { return (StatusCode::FORBIDDEN, "").into_response(); }
    let host = req.headers().get(header::HOST).and_then(|h| h.to_str().ok()).unwrap_or("").to_string();
    if !host_ok(&host, &z.eigene) { return (StatusCode::FORBIDDEN, "").into_response(); }
    let ws = req.headers().get(header::UPGRADE).is_some();
    if req.method() == Method::POST || ws {
        if let Some(o) = req.headers().get(header::ORIGIN).and_then(|h| h.to_str().ok()) { if o != format!("http://{host}") { return (StatusCode::FORBIDDEN, "").into_response(); } } // roh
    }
    next.run(req).await
}

/// The phone's key from its cookie, checked against the stored hashes; updates "last seen".
fn geraet_von(h: &HeaderMap) -> Option<String> {
    let c = h.get(header::COOKIE)?.to_str().ok()?;
    let k = c.split(';').filter_map(|x| x.trim().strip_prefix("pesm_geraet=")).next()?.to_string(); // roh
    let hs = hash(&k);
    let mut v = geraete();
    let g = v.iter_mut().find(|g| g.hash == hs)?;
    let name = g.name.clone();
    if jetzt() - g.zuletzt > 300 { g.zuletzt = jetzt(); geraete_speichern(&v); }
    Some(name)
}

async fn mit_geraet(z: &Zustand, h: &HeaderMap, a: Auftrag) -> Response {
    let Some(name) = geraet_von(h) else { return (StatusCode::UNAUTHORIZED, Json(json!({"koppeln": true}))).into_response(); }; // roh
    z.zuletzt.store(jetzt(), Ordering::Relaxed);
    if let Auftrag::Befehl(b) = &a { log(&format!("{name} {b}")); }
    let (tx, rx) = tokio::sync::oneshot::channel();
    if z.bote.lock().map(|b| b.send(Bote { auftrag: a, antwort: tx })).is_err() { return (StatusCode::SERVICE_UNAVAILABLE, "").into_response(); }
    match tokio::time::timeout(Duration::from_secs(10), rx).await {
        Ok(Ok(Antwort::Json(v))) => Json(v).into_response(),
        Ok(Ok(Antwort::Png(b))) => if b.is_empty() { StatusCode::NOT_FOUND.into_response() } else { ([(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "private, max-age=86400")], b).into_response() }, // roh
        _ => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"fehler": crate::inhalt::t("ui.handy.pc_beschaeftigt")}))).into_response(), // roh
    }
}

#[derive(Deserialize)]
struct KoppelnDaten { code: String, name: Option<String> }
/// Pairing: the 6-digit code from the PC; 5 misses lock for a minute, 20 switch pairing off until it is turned on again.
async fn koppeln(State(z): State<Zustand>, Json(d): Json<KoppelnDaten>) -> Response {
    let ok = {
        let Ok(mut k) = z.kopplung.lock() else { return StatusCode::SERVICE_UNAVAILABLE.into_response() };
        if k.aus { return (StatusCode::FORBIDDEN, Json(json!({"fehler": crate::inhalt::t("ui.handy.kopplung_aus")}))).into_response(); } // roh
        if k.sperre_bis.map_or(false, |s| Instant::now() < s) { return (StatusCode::TOO_MANY_REQUESTS, Json(json!({"fehler": crate::inhalt::t("ui.handy.gesperrt")}))).into_response(); } // roh
        let gut = Instant::now() < k.bis && d.code.trim() == k.code;
        if !gut {
            k.fehler += 1;
            if k.fehler as f32 >= r("kopplung.fehler_aus") { k.aus = true; }
            else if k.fehler % r("kopplung.fehler_sperre") as u32 == 0 { k.sperre_bis = Some(Instant::now() + Duration::from_secs(r("kopplung.sperre_s") as u64)); }
        } else { k.fehler = 0; k.code = neuer_code(); }
        gut
    };
    if !ok { log("kopplung fehlgeschlagen"); return (StatusCode::UNAUTHORIZED, Json(json!({"fehler": crate::inhalt::t("ui.handy.code_falsch")}))).into_response(); } // roh
    let schluessel: String = (0..32).map(|_| format!("{:02x}", rand::random::<u8>())).collect();
    let mut v = geraete();
    if v.len() >= r("geraete.hoechstens") as usize { v.sort_by_key(|g| g.zuletzt); v.remove(0); }
    let name = d.name.filter(|n| !n.trim().is_empty()).map(|n| n.chars().take(30).collect()).unwrap_or_else(|| crate::inhalt::t("ui.handy.geraet").to_string());
    v.push(Geraet { hash: hash(&schluessel), name: name.clone(), seit: jetzt(), zuletzt: jetzt() });
    geraete_speichern(&v);
    log(&format!("gekoppelt {name}"));
    let cookie = format!("pesm_geraet={schluessel}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}", (r("geraete.abmelden_tage") * 86400.0) as u64); // roh
    ([(header::SET_COOKIE, cookie)], Json(json!({"ok": true}))).into_response() // roh
}

/// The nudge line: the phone hears the state number whenever it changes and reloads its page.
async fn ws(State(z): State<Zustand>, h: HeaderMap, up: WebSocketUpgrade) -> Response {
    if geraet_von(&h).is_none() { return StatusCode::UNAUTHORIZED.into_response(); }
    up.on_upgrade(move |s| anstoss(s, z.stand.clone()))
}
async fn anstoss(mut s: WebSocket, stand: Arc<AtomicU64>) {
    let mut alt = 0;
    loop {
        let n = stand.load(Ordering::Relaxed);
        if n != alt { alt = n; if s.send(Message::Text(json!({"stand": n}).to_string().into())).await.is_err() { return; } } // roh
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nur_heimnetz_und_eigener_host() {
        for a in ["10.0.0.5", "172.16.3.1", "192.168.1.20", "169.254.1.1", "127.0.0.1", "::1", "fd00::1", "fe80::1"] { assert!(privat(a.parse().unwrap()), "{a}"); }
        for a in ["8.8.8.8", "172.32.0.1", "2001:db8::1", "100.64.0.1"] { assert!(!privat(a.parse().unwrap()), "{a}"); }
        let eigene = vec!["192.168.1.10".to_string()];
        for h in ["localhost:8320", "127.0.0.1:8320", "192.168.1.10:8320", "192.168.1.10", "[::1]:8320"] { assert!(host_ok(h, &eigene), "{h}"); }
        for h in ["evil.example:8320", "192.168.1.11:8320", "attacker.com"] { assert!(!host_ok(h, &eigene), "{h}"); }
        let c = neuer_code();
        assert_eq!(c.len(), 6);
        assert!(c.chars().all(|x| x.is_ascii_digit()));
        assert_eq!(hash("x").len(), 64);
    }

    /// One raw HTTP request to the test server; returns (status, headers and body as text).
    fn anfrage(port: u16, roh: &str) -> (u16, String) {
        use std::io::{Read, Write};
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        s.write_all(roh.as_bytes()).unwrap();
        let mut out = String::new();
        let _ = s.read_to_string(&mut out);
        (out.split_whitespace().nth(1).and_then(|x| x.parse().ok()).unwrap_or(0), out)
    }
    #[test]
    fn koppeln_und_befehle_ueber_den_server() {
        let dir = std::env::temp_dir().join(format!("pesm_handy_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        unsafe { std::env::set_var("PESM_HANDY_DIR", &dir); std::env::set_var("PESM_HANDY_PORT", "18420"); }
        let port = 18420;
        let mut h = starten().unwrap();
        let code = h.code();
        // the main loop stand-in answers every request
        let rx = std::mem::replace(&mut h.rx, mpsc::channel().1);
        std::thread::spawn(move || { while let Ok(b) = rx.recv() { let v = match b.auftrag { Auftrag::Seite(n) => Antwort::Json(json!({"seite": n})), Auftrag::Befehl(v) => Antwort::Json(json!({"ok": true, "befehl": v})), Auftrag::Bild(..) => Antwort::Png(vec![]) }; let _ = b.antwort.send(v); } });
        std::thread::sleep(Duration::from_millis(300));
        let host = format!("Host: 127.0.0.1:{port}\r\nConnection: close\r\n");
        // the page is there, the data needs a paired phone
        assert_eq!(anfrage(port, &format!("GET / HTTP/1.1\r\n{host}\r\n")).0, 200);
        assert_eq!(anfrage(port, &format!("GET /api/seite/kader HTTP/1.1\r\n{host}\r\n")).0, 401);
        // a foreign host name and a foreign origin are refused
        assert_eq!(anfrage(port, &format!("GET / HTTP/1.1\r\nHost: evil.example\r\nConnection: close\r\n\r\n")).0, 403);
        let body = format!("{{\"code\":\"{code}\",\"name\":\"Testhandy\"}}");
        assert_eq!(anfrage(port, &format!("POST /koppeln HTTP/1.1\r\n{host}Origin: http://evil.example\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len())).0, 403);
        // a wrong code, then the right one: a cookie comes back
        let falsch = "{\"code\":\"000000x\"}";
        assert_eq!(anfrage(port, &format!("POST /koppeln HTTP/1.1\r\n{host}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{falsch}", falsch.len())).0, 401);
        let (st, antwort) = anfrage(port, &format!("POST /koppeln HTTP/1.1\r\n{host}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()));
        assert_eq!(st, 200, "{antwort}");
        let cookie = antwort.lines().find_map(|l| l.strip_prefix("set-cookie: ")).unwrap().split(';').next().unwrap().to_string();
        assert!(antwort.contains("HttpOnly") && antwort.contains("SameSite=Strict"));
        let (st, a) = anfrage(port, &format!("GET /api/seite/kader HTTP/1.1\r\n{host}Cookie: {cookie}\r\n\r\n"));
        assert!(st == 200 && a.contains("kader"), "{a}");
        // only the hash is stored, never the key
        let gespeichert = std::fs::read_to_string(dir.join("handy_geraete.json")).unwrap();
        assert!(!gespeichert.contains(cookie.trim_start_matches("pesm_geraet=")) && gespeichert.contains("Testhandy"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
