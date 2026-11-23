//! Live writing of the career's pesdb (as in the Python version): after every day and every change that matters to
//! the game, PES M writes abilities, positions, age, club, shirt numbers and line-ups (everything fl27_vorbereiten
//! writes) into <Karriere>/pesdb and, once the career was loaded into FL27, into its livecpk copy. Changes in quick
//! succession are collected: one write after RUHE of calm. Only changed files are written, atomically, read back and
//! rolled back on a difference; the original world is never touched.
use crate::game::*;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

/// Calm before a write: ten changes within a second give one write.
pub const RUHE: Duration = Duration::from_secs(2);

impl Career {
    /// Fingerprint of everything the write covers: per player id, club, youth, shirt number, age, positions and
    /// abilities, and the own line-up.
    pub fn db_fingerabdruck(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for p in &self.world.players {
            (p.id, p.team, p.youth, p.no, p.age, &p.pp).hash(&mut h);
            for a in &p.attrs { a.to_bits().hash(&mut h); }
            (crate::mensch::pfeil(p.form), p.style, p.skills).hash(&mut h);
        }
        (&self.xi, self.formation).hash(&mut h);
        h.finish()
    }
    /// Notice a change since the last look (called about once a second and after every day).
    pub fn db_beobachten(&mut self, jetzt: Instant) {
        if !self.db_live || self.fl27_welt.is_empty() { return; }
        let f = self.db_fingerabdruck();
        if f != self.db_abdruck { self.db_abdruck = f; self.db_unruhe = Some(jetzt); }
    }
    /// After RUHE without a new change: write. Returns the text for the person when something was written.
    pub fn db_tick(&mut self, jetzt: Instant) -> Option<Result<String, String>> {
        let t = self.db_unruhe?;
        if jetzt.duration_since(t) < RUHE { return None; }
        self.db_unruhe = None;
        Some(self.db_schreiben())
    }
    /// One write now (also before the match start). Only changed files; the livecpk copy follows when it exists.
    pub fn db_schreiben(&mut self) -> Result<String, String> {
        self.db_schreibvorgaenge += 1;
        let (n, dateien) = self.fl27_schreiben()?;
        if !dateien.is_empty() && !self.fl27_cpk.is_empty() {
            crate::fl27::spiegeln(&self.karriere_dir(), &crate::fl27::sider(), &self.fl27_cpk, &dateien)?;
        }
        let uhr = crate::fl27::uhrzeit();
        if !dateien.is_empty() { self.db_zuletzt = crate::tf!("ui.live.zuletzt_geschrieben_spieler_geaendert", uhr, n); }
        // FL27 reads its database at start: say so once while it runs
        if !dateien.is_empty() && crate::fl27::fl27_pid().is_some() && !self.db_hinweis {
            self.db_hinweis = true;
            return Ok(crate::inhalt::t("ui.live.aenderungen_gespeichert_fl27_uebernimmt_").into());
        }
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::fl27;
    use crate::game::*;
    use crate::world::*;
    use std::time::{Duration, Instant};

    /// A career on MyWorld in a temporary HOME, with the original backup and the career copy like after the start.
    fn karriere(tmp: &std::path::Path) -> Option<(Career, std::path::PathBuf)> {
        let welt = std::path::PathBuf::from("/mnt/GAMES/sp-football-life-2026/drive_c/Games/SP Football Life 2027/SiderAddons/livecpk/_FL26MyWorld");
        if !fl27::echte_welt(&welt) { return None; }
        std::env::set_var("PESM_KARRIEREN", tmp.join("karrieren"));
        let w = World::from_pesdb(&welt.join("common/etc/pesdb"), 3).unwrap();
        let own = w.clubs.iter().position(|c| c.name == "FC Porto").unwrap();
        let mut c = Career::new(w, own, "Test");
        c.fl27_welt = welt.display().to_string();
        let k = c.karriere_dir();
        let orig = k.join("original").join(fl27::welt_name(&welt)).join("common/etc/pesdb");
        std::fs::create_dir_all(&orig).unwrap();
        for e in std::fs::read_dir(welt.join("common/etc/pesdb")).unwrap() { let e = e.unwrap(); std::fs::copy(e.path(), orig.join(e.file_name())).unwrap(); }
        Some((c, welt))
    }
    fn sha(p: &std::path::Path) -> String { fl27::sha256_hex(&std::fs::read(p).unwrap()) }

    #[test]
    fn testmodus_nur_ersatzprozess() {
        let _g = crate::game::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(fl27::fl27_pid(), None, "the real FL27 is never seen in tests");
        let mut kind = std::process::Command::new("sleep").arg("30").spawn().unwrap();
        std::env::set_var("PESM_FL27_ERSATZ_PID", kind.id().to_string());
        assert_eq!(fl27::fl27_pid(), Some(kind.id()));
        let t = fl27::fl27_beenden();
        let _ = kind.wait();
        std::env::remove_var("PESM_FL27_ERSATZ_PID");
        assert!(t.unwrap().contains("Ersatzprozess"));
        let echt = std::path::Path::new(fl27::PREFIX).join("drive_c/Games/SP Football Life 2027/SiderAddons/livecpk/PESMTest/x.bin");
        assert!(std::panic::catch_unwind(|| fl27::atomic_write(&echt, b"x")).is_err(), "the guard stops a write in the real livecpk");
        assert!(!echt.exists());
    }
    #[test]
    fn live_mitschreiben() {
        let _g = crate::game::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = std::env::temp_dir().join(format!("pesm-live-{}", std::process::id()));
        let Some((mut c, welt)) = karriere(&tmp) else { return };
        let original = sha(&welt.join("common/etc/pesdb/Player.bin"));
        let pesdb = fl27::karriere_pesdb(&c.karriere_dir());
        let t0 = Instant::now();
        c.db_beobachten(t0);
        assert!(c.db_tick(t0 + Duration::from_secs(3)).unwrap().is_ok(), "first write makes the copy");
        // a player develops: after the day his new value is in the career's Player.bin
        let p = c.squad()[0];
        c.world.players[p].attrs[0] = (c.world.players[p].attrs[0] + 3.0).min(99.0);
        let soll = c.world.players[p].attrs[0].round() as u8;
        c.end_day().unwrap();
        c.db_beobachten(t0 + Duration::from_secs(10));
        assert!(c.db_tick(t0 + Duration::from_secs(13)).is_some());
        let pb = crate::pesdb::PlayerBin::load(&pesdb.join("Player.bin")).unwrap();
        assert_eq!(pb.attrs(pb.index[&c.world.players[p].id])[0], soll, "the new value is in the copy");
        // ten changes within one second are one write
        let vorher = c.db_schreibvorgaenge;
        for i in 0..10 { c.world.players[p].no = 50 + i; c.db_beobachten(t0 + Duration::from_millis(20_000 + i as u64 * 100)); assert!(c.db_tick(t0 + Duration::from_millis(20_000 + i as u64 * 100)).is_none()); }
        assert!(c.db_tick(t0 + Duration::from_millis(23_000)).is_some());
        assert_eq!(c.db_schreibvorgaenge, vorher + 1, "one write for ten changes");
        // a move to the second team is in PlayerAssignment.bin right after the calm
        let z = c.zweite_mannschaft().expect("FC Porto B");
        let q = *c.world.clubs[c.own].squad.iter().filter(|&&q| c.world.players[q].age <= 22 && c.world.players[q].youth == 0).last().unwrap();
        c.mannschaft_wechseln(q).unwrap();
        c.db_beobachten(t0 + Duration::from_secs(30));
        assert!(c.db_tick(t0 + Duration::from_secs(33)).unwrap().is_ok());
        let (_, a) = fl27::asg_laden(&pesdb).unwrap();
        let id = c.world.players[q].id;
        assert_eq!(a.iter().filter(|x| x.pid == id).map(|x| x.team).collect::<Vec<_>>(), vec![c.world.clubs[z].id]);
        assert!(c.db_zuletzt.contains("Spieler geändert"));
        // nothing changed: nothing written
        c.db_beobachten(t0 + Duration::from_secs(40));
        assert!(c.db_tick(t0 + Duration::from_secs(43)).is_none());
        assert_eq!(sha(&welt.join("common/etc/pesdb/Player.bin")), original, "the original stays untouched");
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::remove_var("PESM_KARRIEREN");
    }
}
