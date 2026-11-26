use super::*;

/// Start ending FL27 in its own thread (SIGTERM waits up to 10 s; the interface keeps running).
pub fn fl27_beenden_starten(v: &mut View) {
    if v.fl27_ende.is_some() { return; }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || { let _ = tx.send(crate::fl27::fl27_beenden()); });
    v.fl27_ende = Some(rx);
    v.fl27_ende_wunsch = false;
    v.toast(crate::inhalt::t("ui.pages.fl27_wird_beendet"), true);
}
/// "FL27 beenden": with the reader running the result is saved first (match day page), else at once.
pub fn fl27_beenden_klick(v: &mut View) {
    if v.fl27.is_some() { v.fl27_ende_wunsch = true; if v.page != Page::Spieltag { v.go(Page::Spieltag); } } else { fl27_beenden_starten(v); }
}
/// Every frame: is FL27 running, live writing of the pesdb, the ending's answer, the automatic end.
pub fn fl27_und_db(v: &mut View, c: &mut Career) {
    let t = get_time() as f32;
    if t - v.fl27_pruef_t > 2.0 {
        v.fl27_pruef_t = t;
        v.fl27_laeuft = crate::fl27::fl27_pid().is_some();
        if v.fl27_laeuft { v.fl27_gesehen = true; }
        // FL27 is gone after a match (ended, closed or crashed), or CalcioM starts and old presets are still there: undo them.
        // Between writing the presets and FL27 appearing nothing happens (it hasn't been seen yet).
        else if (v.fl27_gesehen || !v.fl27_erster_blick) && v.fl27_ende.is_none() && v.fl27.is_none() {
            v.fl27_gesehen = false;
            if let Some(s) = c.vorgaben_aufraeumen() { v.toast(&s, true); }
        }
        v.fl27_erster_blick = true;
    }
    // nothing is written into the game folder while FL27 is being ended
    if v.fl27_ende.is_none() {
        let jetzt = std::time::Instant::now();
        if t - v.db_blick_t > 1.0 { v.db_blick_t = t; c.db_beobachten(jetzt); }
        match c.db_tick(jetzt) { Some(Ok(s)) if !s.is_empty() => v.toast(&s, true), Some(Err(e)) if !crate::fl27::testmodus() => v.toast(&crate::tf!("ui.pages.datenbank_nicht_geschrieben", e), false), _ => {} } // previews have no original database to write
    }
    if v.fl27_ende_wunsch && v.fl27.is_none() { fl27_beenden_starten(v); }
    if let Some(Ok(r)) = v.fl27_ende.as_ref().map(|rx| rx.try_recv()) {
        v.fl27_ende = None;
        v.fl27_pruef_t = -10.0;
        match r {
            Ok(s) => { let e = v.fl27_uebernommen.take().map(|x| crate::tf!("ui.pages.uebernommen", x)).unwrap_or_else(|| crate::inhalt::t("ui.pages.kein_ergebnis_uebernommen_bitte_von").into()); v.toast(&format!("{s}{e}"), true); }
            Err(e) => v.toast(&e, false),
        }
    }
    if let Some(a) = v.fl27_auto_t { if t >= a && v.fl27.is_none() { v.fl27_auto_t = None; if v.fl27_laeuft { fl27_beenden_starten(v); } } }
}
