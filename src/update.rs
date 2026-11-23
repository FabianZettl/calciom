//! Update notice (Fabian 23.11.): once at start a background thread asks GitHub for the newest release of
//! FabianZettl/pesm (curl, built into Windows 10+ and Linux); a newer one shows a banner on the start screen with a
//! button to its page. Nothing is downloaded or installed, and without network nothing happens.
use std::sync::OnceLock;

/// The version of this build, as tagged on GitHub.
pub const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"), "-beta");
const API: &str = "https://api.github.com/repos/FabianZettl/pesm/releases?per_page=1";

/// The newest release (tag, page) when it is newer than this build.
pub static NEUER: OnceLock<Option<(String, String)>> = OnceLock::new();

pub fn pruefen() {
    if std::env::var("PESM_KARRIEREN").is_ok() { return; } // tests and previews stay offline
    std::thread::spawn(|| {
        let r = std::process::Command::new(crate::id("curl")).args([crate::id("-s"), crate::id("-m"), "8", crate::id("-H"), "User-Agent: pesm", API]).output().ok().filter(|o| o.status.success()); // roh
        let neu = r.and_then(|o| {
            let v: serde_json::Value = serde_json::from_slice(&o.stdout).ok()?;
            let e = v.get(0)?;
            let tag = e["tag_name"].as_str()?.to_string();
            neuer(&tag, VERSION).then(|| (tag, e["html_url"].as_str().unwrap_or("https://github.com/FabianZettl/pesm/releases").to_string()))
        });
        let _ = NEUER.set(neu);
    });
}

/// "v0.1.10-beta" is newer than "v0.1.9-beta": the numbers compared one by one.
pub fn neuer(a: &str, b: &str) -> bool {
    let z = |s: &str| s.trim_start_matches('v').split(|c: char| !c.is_ascii_digit()).filter(|x| !x.is_empty()).map(|x| x.parse::<u32>().unwrap_or(0)).collect::<Vec<_>>();
    z(a) > z(b)
}

/// Opens a web page in the person's browser.
pub fn oeffnen(url: &str) {
    #[cfg(windows)] let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    #[cfg(not(windows))] let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

#[cfg(test)]
mod tests {
    #[test]
    fn versionen_vergleichen() {
        assert!(super::neuer("v0.1.10-beta", "v0.1.9-beta"));
        assert!(super::neuer("v0.2.0-beta", "v0.1.1-beta"));
        assert!(!super::neuer("v0.1.1-beta", "v0.1.1-beta"));
        assert!(!super::neuer("v0.1.0-beta", "v0.1.1-beta"));
    }
}
