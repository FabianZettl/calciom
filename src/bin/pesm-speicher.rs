// Read-only memory helper for PES M (FL27-Anbindung §1.1). The person gives it the read right once
// (capability cap_sys_ptrace, see the settings page of PES M for the exact command).
// It answers JSON lines on stdin/stdout and can only search statistics blocks and read bytes, never write.
/// An identifier in the code (see src/main.rs).
pub const fn id(s: &'static str) -> &'static str { s }
#[path = "../fl27.rs"]
mod fl27;
#[path = "../cpk.rs"]
mod cpk;
#[path = "../inhalt.rs"]
#[allow(dead_code)]
mod inhalt;
#[cfg(unix)]
use std::io::{BufRead, Write};

/// Windows reads FL27 directly from PES M; the helper is only needed on Linux.
#[cfg(windows)]
fn main() {}
#[cfg(unix)]
fn main() {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    let mut mem: Option<fl27::ProcMem> = None;
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let q: serde_json::Value = serde_json::from_str(&line).unwrap_or_default();
        let pid = q["pid"].as_u64().unwrap_or(0) as u32;
        if mem.as_ref().map(|m| m.pid != pid).unwrap_or(true) {
            mem = fl27::ProcMem::open(pid).ok();
        }
        use fl27::Mem;
        let r = match (q["op"].as_str(), mem.as_mut()) {
            (_, None) => serde_json::json!({"ok": false, "fehler": inhalt::t("ui.speicher.prozess_nicht_lesbar")}),
            (Some("offen"), Some(m)) => {
                let ok = m.regions().first().map(|r| m.read(r.0, 4).is_some()).unwrap_or(false);
                if ok { serde_json::json!({"ok": true}) } else { serde_json::json!({"ok": false, "fehler": inhalt::t("ui.speicher.leserecht_fehlt")}) }
            }
            (Some("bloecke"), Some(m)) => serde_json::json!({"ok": true, "bloecke": fl27::bloecke(m)}),
            (Some("lesen"), Some(m)) => {
                let n = (q["laenge"].as_u64().unwrap_or(0) as usize).min(4 << 20);
                match m.read(q["adresse"].as_u64().unwrap_or(0), n) {
                    Some(b) => serde_json::json!({"ok": true, "daten": b.iter().map(|x| format!("{x:02x}")).collect::<String>()}),
                    None => serde_json::json!({"ok": false, "fehler": inhalt::t("ui.speicher.nicht_lesbar")}),
                }
            }
            _ => serde_json::json!({"ok": false, "fehler": crate::id("unbekannt")}),
        };
        if writeln!(out, "{r}").is_err() || out.flush().is_err() { break; }
    }
}
