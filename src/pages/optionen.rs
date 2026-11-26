use super::*;

// ================================================================ Optionen
pub(super) fn optionen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // Fabian 23.11.: one area per topic instead of everything on one card
    let t = crate::inhalt::t;
    tabs(ui, v, &[t("ui.einst.reiter.spielstand"), t("ui.einst.reiter.anzeige"), t("ui.einst.reiter.klang"), t("ui.einst.reiter.spiel"), t("ui.einst.reiter.fl27"), t("ui.einst.reiter.features"), t("ui.einst.reiter.handy")]);
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    match v.tab {
        0 => {
            ui.card_titled(r, t("ui.einst.reiter.spielstand"), Theme::akzent());
    ui.text(&crate::tf!("ui.pages.gespeichert_wird_in", c.save_path().display()), r.x + 16.0, r.y + 64.0, 16.0, Theme::text2());
    ui.text(crate::inhalt::t("ui.pages.autosave"), r.x + 16.0, r.y + 100.0, 16.0, Theme::text2());
    if let Some(a) = ui.segment(Rect::new(r.x + 16.0, r.y + 112.0, 300.0, 36.0), &[crate::inhalt::t("ui.pages.autosave.montags"), crate::inhalt::t("ui.pages.autosave.taeglich"), crate::inhalt::t("ui.pages.autosave.monatlich")], c.autosave as usize) { c.autosave = a as u8; let _ = c.save(); }
    if ui.button_gold(Rect::new(r.x + 16.0, r.y + 164.0, 240.0, 44.0), crate::inhalt::t("ui.pages.jetzt_sichern")) { match c.save() { Ok(_) => v.toast(crate::inhalt::t("ui.pages.gesichert"), true), Err(e) => v.toast(&crate::tf!("ui.pages.fehler_beim_sichern", e), false) } }
            // rename, second copy, back to the start screen (also to load another career)
            ui.text_semi(t("ui.spielstand.name"), r.x + 16.0, r.y + 250.0, 17.0, Theme::text());
            let nr = Rect::new(r.x + 16.0, r.y + 262.0, 360.0, 40.0);
            let mut name = v.name_neu.clone().unwrap_or_else(|| c.save_name.clone());
            if ui.hit(nr) && v.name_neu.is_none() { v.name_neu = Some(c.save_name.clone()); }
            ui.input(nr, &mut name, v.name_neu.is_some(), "");
            if v.name_neu.is_some() { v.name_neu = Some(name.clone()); }
            if ui.button(Rect::new(r.x + 390.0, r.y + 264.0, 180.0, 36.0), t("ui.spielstand.umbenennen"), false) {
                match c.umbenennen(&name) { Ok(()) => { v.name_neu = None; v.toast(&crate::tf!("ui.spielstand.umbenannt", c.save_name.clone()), true); } Err(e) => v.toast(&e, false) }
            }
            let laeuft = KOPIE.lock().unwrap().is_some();
            if ui.button(Rect::new(r.x + 16.0, r.y + 320.0, 260.0, 40.0), t(if laeuft { "ui.spielstand.kopie_laeuft" } else { "ui.spielstand.kopie" }), laeuft) && !laeuft {
                match c.kopie() { Ok((n, rx)) => { *KOPIE.lock().unwrap() = Some(rx); v.toast(&crate::tf!("ui.spielstand.kopie_angelegt", n), true); } Err(e) => v.toast(&e, false) }
            }
            if laeuft {
                use std::sync::atomic::Ordering::Relaxed;
                let (f, g) = (crate::fl27::FORT_FERTIG.load(Relaxed), crate::fl27::FORT_GESAMT.load(Relaxed));
                if g > 0 { ui.bar(r.x + 290.0, r.y + 334.0, 300.0, 10.0, f as f32 / g as f32, Theme::akzent()); }
                let fertig = KOPIE.lock().unwrap().as_ref().and_then(|rx| rx.try_recv().ok());
                if let Some(e) = fertig { *KOPIE.lock().unwrap() = None; match e { Ok(_) => v.toast(t("ui.spielstand.kopie_fertig"), true), Err(e) => v.toast(&e, false) } }
            }
            ui.paragraph(t("ui.spielstand.kopie_hinweis"), r.x + 16.0, r.y + 384.0, 13.0, 600.0, Theme::text2());
            if ui.button_gold(Rect::new(r.x + 16.0, r.y + 430.0, 320.0, 44.0), t("ui.spielstand.zum_start")) { v.zum_start = true; }
            // logs for a bug report in one archive, the folder opened right away
            if ui.button(Rect::new(r.x + 360.0, r.y + 434.0, 280.0, 36.0), t("ui.diagnose.knopf"), false) {
                match crate::fl27::diagnose_paket() { Ok(p) => { v.toast(&crate::tf!("ui.diagnose.fertig", p.display()), true); if let Some(d) = p.parent() { crate::update::oeffnen(&d.display().to_string()); } } Err(e) => v.toast(&e, false) }
            }
            ui.paragraph(t("ui.spielstand.zum_start_hinweis"), r.x + 16.0, r.y + 494.0, 13.0, 600.0, Theme::text2());
            // a slim world backup for new careers (only the database), off unless switched on
            let schlank = crate::fl27::sicherung_schlank();
            if ui.button(Rect::new(r.x + 16.0, r.y + 520.0, 420.0, 34.0), t("ui.sicherung.schlank"), schlank) { crate::fl27::sicherung_schlank_setzen(!schlank); }
            ui.paragraph(t("ui.sicherung.schlank_hinweis"), r.x + 450.0, r.y + 528.0, 13.0, 460.0, Theme::text2());
        }
        1 => {
            ui.card_titled(r, t("ui.einst.reiter.anzeige"), Theme::akzent());
    ui.text_semi(crate::inhalt::t("ui.pages.thema"), r.x + 16.0, r.y + 64.0, 17.0, Theme::text());
    if let Some(t) = ui.segment(Rect::new(r.x + 16.0, r.y + 82.0, 420.0, 38.0), &[crate::inhalt::t("ui.pages.sonntag_hell"), crate::inhalt::t("ui.pages.flutlicht_dunkel")], v.dark as usize) {
        v.dark = t == 1;
        let _ = std::fs::create_dir_all(save_dir());
        let _ = std::fs::write(theme_file(), if v.dark { crate::id("flutlicht") } else { crate::id("sonntag") });
    }
    // on a matchday the competition's colours (Fabian 24.11.), on other days light or dark as chosen
    ui.text_semi(crate::inhalt::t("ui.thema.spieltag"), r.x + 16.0, r.y + 154.0, 17.0, Theme::text());
    if let Some(t) = ui.segment(Rect::new(r.x + 16.0, r.y + 172.0, 580.0, 38.0), &[crate::inhalt::t("ui.thema.immer_gleich"), crate::inhalt::t("ui.thema.wettbewerb")], c.thema_wettbewerb as usize) { c.thema_wettbewerb = t == 1; }
    ui.text_semi(crate::inhalt::t("optionen.sprache"), r.x + 16.0, r.y + 474.0, 17.0, Theme::text());
    // Fabian 26.11.: Spanish and French as well; their own names, so everyone finds theirs
    if let Some(t) = ui.segment(Rect::new(r.x + 16.0, r.y + 492.0, 580.0, 38.0), &[crate::inhalt::t("sprache.de"), crate::inhalt::t("sprache.en"), "Español", "Français"], crate::inhalt::sprache() as usize) { // roh
        let s = crate::inhalt::SPRACHEN[t];
        crate::inhalt::sprache_setzen(s);
        let _ = std::fs::create_dir_all(save_dir());
        let _ = std::fs::write(sprache_file(), s.code());
    }
    ui.text_semi(crate::inhalt::t("ui.akzent.titel"), r.x + 16.0, r.y + 244.0, 17.0, Theme::text());
    // crest, default blue, or an own colour (Fabian 23.11.: let users adjust it themselves)
    let modus = if c.akzent_wappen { 0 } else if eigener_akzent().is_some() { 2 } else { 1 };
    if let Some(t) = ui.segment(Rect::new(r.x + 16.0, r.y + 262.0, 580.0, 38.0), &[crate::inhalt::t("ui.akzent.wappen"), crate::inhalt::t("ui.akzent.standard"), crate::inhalt::t("ui.akzent.eigene")], modus) {
        c.akzent_wappen = t == 0;
        if t == 1 { akzent_setzen(None); } else if t == 2 && eigener_akzent().is_none() { akzent_setzen(Some(0x38bdf8)); }
    }
    if modus == 2 || (!c.akzent_wappen && eigener_akzent().is_some()) {
        const FARBEN: [u32; 12] = [0x38bdf8, 0x2563eb, 0x7c3aed, 0xdb2777, 0xdc2626, 0xea580c, 0xd97706, 0x65a30d, 0x16a34a, 0x0d9488, 0x475569, 0x111827];
        for (k, &f) in FARBEN.iter().enumerate() {
            let b = Rect::new(r.x + 16.0 + k as f32 * 36.0, r.y + 314.0, 30.0, 30.0);
            rrect(b.x, b.y, b.w, b.h, 15.0, hex(f));
            if eigener_akzent() == Some(f) { rrect_line(b.x - 3.0, b.y - 3.0, b.w + 6.0, b.h + 6.0, 18.0, Theme::text()); }
            if ui.hit(b) { akzent_setzen(Some(f)); }
        }
        let hr = Rect::new(r.x + 460.0, r.y + 310.0, 160.0, 38.0);
        let mut h = v.name_neu.clone().filter(|s| s.starts_with('#')).unwrap_or_else(|| format!("#{:06x}", eigener_akzent().unwrap_or(0)));
        // a click starts an empty field (Fabian 23.11.: typing or pasting onto the old code gave nothing valid)
        if ui.hit(hr) { h = "#".into(); v.name_neu = Some(h.clone()); }
        ui.input(hr, &mut h, v.name_neu.as_deref().map_or(false, |s| s.starts_with('#')), crate::id("#rrggbb"));
        if v.name_neu.as_deref().map_or(false, |s| s.starts_with('#')) {
            h = format!("#{}", h.chars().filter(|c| c.is_ascii_hexdigit()).take(6).collect::<String>());
            if let Ok(x) = u32::from_str_radix(&h[1..], 16).ok().filter(|_| h.len() == 7).ok_or(()) { akzent_setzen(Some(x)); v.name_neu = None; } else { v.name_neu = Some(h); }
        }
    }
    // the UI scale (75–150 %), saved for every start
    ui.text_semi(crate::inhalt::t("ui.skala.titel"), r.x + 16.0, r.y + 384.0, 17.0, Theme::text());
    let stufen = [0.75f32, 0.9, 1.0, 1.15, 1.3, 1.5];
    let namen: Vec<String> = stufen.iter().map(|s| format!("{:.0} %", s * 100.0)).collect();
    let nr: Vec<&str> = namen.iter().map(|s| s.as_str()).collect();
    let jetzt = stufen.iter().position(|s| (s - crate::ui::ui_skala()).abs() < 0.01).unwrap_or(2);
    if let Some(k) = ui.segment(Rect::new(r.x + 16.0, r.y + 402.0, 580.0, 38.0), &nr, jetzt) {
        crate::ui::ui_skala_setzen(stufen[k]);
        let _ = std::fs::create_dir_all(save_dir());
        let _ = std::fs::write(skala_file(), format!("{}", stufen[k]));
    }
        }
        2 => {
            ui.card_titled(r, t("ui.einst.reiter.klang"), Theme::akzent());
    ui.text_semi(crate::inhalt::t("ui.klang.titel"), r.x + 16.0, r.y + 64.0, 17.0, Theme::text());
    let stufen = [crate::id("aus"), crate::id("leise"), crate::id("normal")];
    let jetzt = stufen.iter().position(|s| (klang_wert(s) - ui.lautstaerke).abs() < 0.01).unwrap_or(1);
    if let Some(k) = ui.segment(Rect::new(r.x + 16.0, r.y + 78.0, 380.0, 34.0), &[crate::inhalt::t("ui.klang.aus"), crate::inhalt::t("ui.klang.leise"), crate::inhalt::t("ui.klang.normal")], jetzt) {
        ui.lautstaerke = klang_wert(stufen[k]);
        let _ = std::fs::create_dir_all(save_dir());
        let _ = std::fs::write(klang_file(), stufen[k]);
        ui.play(crate::id("bestaetigt"));
    }
    // background music (musik.rs): a radio stream or YouTube playlist, pasted from the clipboard; paused during a match
    {
        let url = crate::musik::url();
        let (mpv, ytdlp) = crate::musik::programme();
        ui.text_semi(crate::inhalt::t("ui.musik.titel"), r.x + 16.0, r.y + 140.0, 17.0, Theme::text());
        if ui.button(Rect::new(r.x + 16.0, r.y + 154.0, 300.0, 34.0), crate::inhalt::t("ui.musik.einfuegen"), false) {
            let neu = macroquad::miniquad::window::clipboard_get().unwrap_or_default().trim().to_string();
            if neu.starts_with(crate::id("http")) { crate::musik::setzen(&neu); ui.play(crate::id("bestaetigt")); } else { v.toast(crate::inhalt::t("ui.musik.keine_url"), false); }
        }
        if ui.button(Rect::new(r.x + 328.0, r.y + 154.0, 120.0, 34.0), crate::inhalt::t("ui.klang.aus"), url.is_empty()) { crate::musik::setzen(""); }
        if !url.is_empty() && ui.button(Rect::new(r.x + 460.0, r.y + 154.0, 150.0, 34.0), crate::inhalt::t("ui.musik.weiter"), false) { crate::musik::weiter(); }
        let zeile = if !mpv { crate::inhalt::t("ui.musik.ohne_mpv").to_string() }
            else if url.is_empty() { crate::inhalt::t("ui.musik.hinweis").to_string() }
            else if crate::musik::ist_youtube(&url) && !ytdlp { crate::inhalt::t("ui.musik.ohne_ytdlp").to_string() }
            else { crate::tf!("ui.musik.spielt", ui.fit(&url, 13.0, r.w - 500.0)) };
        ui.paragraph(&zeile, r.x + 624.0, r.y + 156.0, 13.0, r.w - 640.0, if mpv { Theme::text2() } else { Theme::achtung() });
        // music volume, saved for every start (Fabian 24.11.)
        ui.text_semi(crate::inhalt::t("ui.musik.lautstaerke"), r.x + 16.0, r.y + 222.0, 17.0, Theme::text());
        let laut = crate::musik::lautstaerke().round() as i32;
        if let Some(n) = ui.slider(Rect::new(r.x + 24.0, r.y + 236.0, 360.0, 24.0), laut, 0, 100) { crate::musik::lautstaerke_setzen(n as f32); }
        ui.text(&format!("{laut} %"), r.x + 400.0, r.y + 254.0, 15.0, Theme::text2());
    }
        }
        3 => {
            ui.card_titled(r, t("ui.einst.reiter.spiel"), Theme::akzent());
    // FC-26 wish: what AI clubs may do with our players
    if ui.button(Rect::new(r.x + 16.0, r.y + 280.0, 360.0, 34.0), crate::inhalt::t(if c.ohne_ki_vorvertraege { "ui.einst.ki_vv_aus" } else { "ui.einst.ki_vv_an" }), !c.ohne_ki_vorvertraege) { c.ohne_ki_vorvertraege = !c.ohne_ki_vorvertraege; }
    if ui.button(Rect::new(r.x + 390.0, r.y + 280.0, 360.0, 34.0), crate::inhalt::t(if c.ohne_ki_klauseln { "ui.einst.ki_kl_aus" } else { "ui.einst.ki_kl_an" }), !c.ohne_ki_klauseln) { c.ohne_ki_klauseln = !c.ohne_ki_klauseln; }
    // market-value era: the game's values or an inflated modern market (prices and values; wages stay)
    ui.text_semi(crate::inhalt::t("ui.aera.titel"), r.x + 16.0, r.y + 344.0, 17.0, Theme::text());
    let stufen = [100u32, 150, 220];
    let jetzt = stufen.iter().position(|&s| s == if c.marktwert_aera == 0 { 100 } else { c.marktwert_aera }).unwrap_or(0);
    if let Some(i) = ui.segment(Rect::new(r.x + 16.0, r.y + 358.0, 560.0, 34.0), &[crate::inhalt::t("ui.aera.klassisch"), crate::inhalt::t("ui.aera.heute"), crate::inhalt::t("ui.aera.modern")], jetzt) {
        c.marktwert_aera = stufen[i];
        crate::world::WERT_AERA.store(stufen[i], std::sync::atomic::Ordering::Relaxed);
        c.world.update_values();
    }
    ui.text_semi(crate::inhalt::t("ui.tempo.titel"), r.x + 16.0, r.y + 214.0, 17.0, Theme::text());
    if let Some(k) = ui.segment(Rect::new(r.x + 16.0, r.y + 228.0, 560.0, 34.0), &[crate::inhalt::t("ui.tempo.sofort"), crate::inhalt::t("ui.tempo.zuegig"), crate::inhalt::t("ui.tempo.realistisch")], c.verhandlungstempo.min(2) as usize) { c.verhandlungstempo = k as u8; }
    ui.text_semi(crate::inhalt::t("ui.kaderregel.titel"), r.x + 16.0, r.y + 64.0, 17.0, Theme::text());
    if let Some(k) = ui.segment(Rect::new(r.x + 16.0, r.y + 78.0, 300.0, 34.0), &[crate::inhalt::t("ui.kaderregel.an"), crate::inhalt::t("ui.kaderregel.aus")], (!c.kaderregeln) as usize) { c.kaderregeln = k == 0; }
    ui.text_semi(crate::inhalt::t("ui.pages.co_trainer_uebernimmt"), r.x + 16.0, r.y + 144.0, 17.0, Theme::text());
    let mut x = r.x + 16.0;
    for (i, (label, on)) in [(crate::inhalt::t("aktion.aufstellung"), c.co_lineup), (crate::inhalt::t("status.training"), c.co_training), (crate::inhalt::t("md.ui.presse.verlauf"), c.co_press)].iter().enumerate() {
        if ui.button(Rect::new(x, r.y + 158.0, 220.0, 36.0), &format!("{} {}", label, if *on { "✓" } else { "" }), *on) {
            match i { 0 => c.co_lineup = !on, 1 => c.co_training = !on, _ => c.co_press = !on }
        }
        x += 232.0;
    }
        }
        4 => {
    let r2 = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(r2, crate::inhalt::t("ui.pages.football_life_2027"), Theme::gold());
    let mut on = c.fl27_erlaubt;
    if ui.button(Rect::new(r2.x + 16.0, r2.y + 48.0, 240.0, 34.0), if on { crate::inhalt::t("ui.pages.spielstart_erlaubt") } else { crate::inhalt::t("ui.pages.spielstart_erlaubt_2") }, on) { on = !on; c.fl27_erlaubt = on; }
    let live = c.db_live;
    if ui.button(Rect::new(r2.x + 268.0, r2.y + 48.0, 290.0, 34.0), if live { crate::inhalt::t("ui.pages.datenbank_live_mitschreiben") } else { crate::inhalt::t("ui.pages.datenbank_live_mitschreiben_2") }, live) { c.db_live = !live; }
    // BlankState222's start_stamina module (installed by the user): PES M writes the start fitness for it
    if ui.button(Rect::new(r2.x + 450.0, r2.y + 90.0, 300.0, 34.0), crate::inhalt::t(if c.direktstart { "ui.fl27.direktstart_an" } else { "ui.fl27.direktstart_aus" }), c.direktstart) { c.direktstart = !c.direktstart; }
    if ui.button(Rect::new(r2.x + 16.0, r2.y + 90.0, 420.0, 34.0), crate::inhalt::t(if c.start_stamina { "ui.fl27.start_stamina_an" } else { "ui.fl27.start_stamina_aus" }), c.start_stamina) { c.start_stamina = !c.start_stamina; }
    if ui.button(Rect::new(r2.x + 450.0, r2.y + 90.0, 300.0, 34.0), crate::inhalt::t(if c.direktstart { "ui.fl27.direktstart_an" } else { "ui.fl27.direktstart_aus" }), c.direktstart) { c.direktstart = !c.direktstart; }
    let auto = c.fl27_auto_ende;
    if ui.button(Rect::new(r2.x + 570.0, r2.y + 48.0, 320.0, 34.0), if auto { crate::inhalt::t("ui.pages.fl27_nach_dem_abpfiff_beenden") } else { crate::inhalt::t("ui.pages.fl27_nach_dem_abpfiff_beenden_2") }, auto) { c.fl27_auto_ende = !auto; }
    // right column under „Rückbau“: the career database after the match, and presets still lying in the game folder
    let db_aus = c.db_nach_spiel_aus;
    if ui.button(Rect::new(r2.x + r2.w - 260.0, r2.y + 90.0, 244.0, 34.0), &ui.fit(crate::inhalt::t(if db_aus { "ui.fl27.db_nach_spiel_aus_an" } else { "ui.fl27.db_nach_spiel_aus" }), 14.0, 230.0), db_aus) { c.db_nach_spiel_aus = !db_aus; }
    let liegen: Vec<&str> = crate::fl27::VORGABEN.iter().copied().filter(|n| crate::fl27::sider().join(crate::id("flw")).join(n).exists()).collect();
    if !liegen.is_empty() && !v.fl27_laeuft {
        ui.paragraph(&crate::tf!("ui.fl27.vorgaben_liegen", liegen.len()), r2.x + r2.w - 260.0, r2.y + 148.0, 13.0, 244.0, Theme::achtung());
        if ui.button(Rect::new(r2.x + r2.w - 260.0, r2.y + 186.0, 244.0, 34.0), crate::inhalt::t("ui.fl27.vorgaben_entfernen"), false) {
            match crate::fl27::vorgaben_schreiben(&crate::fl27::sider(), None, None, None, None) { Ok(_) => { c.vorgaben_gesetzt.clear(); v.toast(crate::inhalt::t("ui.fl27.vorgaben_zurueck"), true) } Err(e) => v.toast(&e, false) }
        }
    }
    let s = crate::fl27::sider();
    let ok = |b: bool| if b { crate::id("ok") } else { crate::id("fehlt") };
    let welt = c.welt().join("common/etc/pesdb/PlayerAssignment.bin").exists();
    let ini = std::fs::read_to_string(s.join("sider.ini")).unwrap_or_default();
    let zeile = !c.fl27_cpk.is_empty() && ini.lines().any(|l| l.trim_start().starts_with("cpk.root") && l.contains(&format!("\\{}\"", c.fl27_cpk)));
    let module = crate::fl27::MODULE.iter().filter(|(rel, _)| std::fs::read_to_string(s.join(rel)).map(|t| t.contains("PES M")).unwrap_or(false)).count(); // roh
    let lines = [
        crate::tf!("ui.pages.spielordner_und_welt_datenbank", ok(welt)),
        crate::tf!("ui.pages.karriere_zeile_in_sider_ini", if zeile { crate::inhalt::t("ui.wort.aktiv") } else { crate::inhalt::t("ui.pages.noch_nicht_eingerichtet") }, module),
        if cfg!(windows) { crate::tf!("ui.pages.speicher_direkt", crate::fl27::freischalten_hinweis()) } else { crate::tf!("ui.pages.speicher_helfer", if crate::fl27::helfer_pfad().exists() { crate::inhalt::t("ui.wort.gebaut") } else { crate::inhalt::t("ui.wort.fehlt") }, crate::fl27::freischalten_hinweis()) },
        crate::tf!("ui.pages.original_sicherung", if c.karriere_dir().join("original/inhalt.json").exists() { crate::inhalt::t("ui.wort.vorhanden") } else { crate::inhalt::t("ui.pages.wird_beim_ersten_start_angelegt") }),
        if c.db_zuletzt.is_empty() { crate::inhalt::t("ui.pages.zuletzt_geschrieben_noch_nicht").to_string() } else { c.db_zuletzt.clone() },
    ];
    let mut y = r2.y + 134.0;
    for l in lines { y = ui.paragraph(&l, r2.x + 16.0, y, 15.0, r2.w - 300.0, Theme::text2()) + 2.0; }
    if ui.button(Rect::new(r2.x + r2.w - 260.0, r2.y + 48.0, 244.0, 34.0), crate::inhalt::t("ui.pages.rueckbau_karriere_aus_fl27"), false) {
        let k = c.karriere_dir();
        let cpk = c.fl27_cpk.clone();
        let r = crate::fl27::sider_ini_anwenden(&k, &s, |t| Ok(crate::fl27::sider_ini_rueckbau(t, &cpk))).and_then(|_| crate::fl27::vorgaben_schreiben(&s, None, None, None, None));
        match r { Ok(_) => { c.vorgaben_gesetzt.clear(); v.toast(crate::inhalt::t("ui.pages.karriere_zeile_und_vorgaben_entfernt"), true) } Err(e) => v.toast(&e, false) }
    }
        }
        5 => {
            // Fabian 25.11.: whole feature areas on or off, also in a running save; off = the world runs on with defaults
            ui.card_titled(r, t("ui.einst.reiter.features"), Theme::akzent());
            ui.paragraph(t("ui.features.hinweis"), r.x + 16.0, r.y + 56.0, 13.0, r.w - 32.0, Theme::text2());
            for (i, f) in crate::game::FEATURES.iter().enumerate() {
                let y = r.y + 100.0 + i as f32 * 56.0;
                let an = c.feature(f);
                if let Some(k) = ui.segment(Rect::new(r.x + 16.0, y, 180.0, 34.0), &[t("ui.features.an"), t("ui.features.aus")], (!an) as usize) {
                    if (k == 0) != an { if an { c.aus.push(f.to_string()); } else { c.aus.retain(|a| a != f); } }
                }
                ui.text_semi(t(&format!("ui.features.{f}")), r.x + 212.0, y + 14.0, 16.0, Theme::text());
                ui.text(t(&format!("ui.features.{f}.info")), r.x + 212.0, y + 32.0, 12.0, Theme::text2());
            }
        }
        _ => {
            ui.card_titled(r, t("ui.einst.reiter.handy"), Theme::akzent());
    // phone access (konzepte/mobile-ansicht.md): switch, address, pairing code with QR, paired phones
    {
        let hr = Rect::new(r.x + 16.0, r.y + 50.0, 364.0, 60.0);
        let an = crate::handy::an();
        if ui.button(Rect::new(hr.x, hr.y + 14.0, 170.0, 34.0), crate::inhalt::t(if an { "ui.handy.an" } else { "ui.handy.aus" }), an) { crate::handy::setzen(!an); }
        if let Some((adr, code, verbunden)) = crate::handy::info() {
            ui.text_semi(&code, hr.x + 184.0, hr.y + 40.0, 26.0, Theme::akzent());
            if verbunden { ui.text(crate::inhalt::t("ui.handy.verbunden"), hr.x + 290.0, hr.y + 38.0, 13.0, Theme::gut()); }
            let url = format!("{adr}?c={code}");
            ui.text(&crate::tf!("ui.handy.hinweis", adr.clone()), r.x + 16.0, r.y + 130.0, 12.0, Theme::text2());
            if let Ok(q) = qrcode::QrCode::new(url.as_bytes()) {
                let farben = q.to_colors();
                let n = q.width();
                let px = 3.0;
                let (qx, qy) = (r.x + 420.0, r.y + 60.0);
                draw_rectangle(qx - 6.0, qy - 6.0, n as f32 * px + 12.0, n as f32 * px + 12.0, hex(0xffffff));
                for (i, f) in farben.iter().enumerate() { if *f == qrcode::Color::Dark { draw_rectangle(qx + (i % n) as f32 * px, qy + (i / n) as f32 * px, px, px, hex(0x000000)); } }
            }
        }
        let g = crate::handy::geraete();
        if !g.is_empty() {
            ui.text(&crate::tf!("ui.handy.geraete", g.iter().map(|x| x.name.clone()).collect::<Vec<_>>().join(", ")), hr.x, hr.y + 4.0, 12.0, Theme::text2());
            if ui.button(Rect::new(hr.x + 184.0 + 0.0, hr.y - 30.0, 180.0, 26.0), crate::inhalt::t("ui.handy.entfernen"), false) { crate::handy::geraete_speichern(&[]); }
        }
    }
        }
    }
}
/// The career copy running in the background (Einstellungen › Spielstand).
static KOPIE: std::sync::Mutex<Option<std::sync::mpsc::Receiver<Result<usize, String>>>> = std::sync::Mutex::new(None);

pub(super) fn handbuch(ui: &mut Ui, v: &mut View, _c: &mut Career) {
    let h = crate::inhalt::eigen("handbuch");
    let kap: Vec<String> = h["kapitel"].as_array().map(|a| a.iter().filter_map(|x| x["id"].as_str().map(String::from)).collect()).unwrap_or_default();
    let (kap, t) = (&kap, crate::inhalt::eigen("handbuch_texte"));
    let tx = |k: &str| t[k].as_str().unwrap_or("").to_string();
    let l = Rect::new(x0() + 4.0, Y0, 300.0, hoehe() - Y0 - 16.0);
    ui.card_titled(l, crate::inhalt::t("ui.seiten.kapitel"), Theme::akzent());
    for (i, k) in kap.iter().enumerate() {
        let r = Rect::new(l.x + 6.0, l.y + 48.0 + i as f32 * 30.0, l.w - 12.0, 28.0);
        if v.tab == i { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::akzent(), 0.15)); }
        ui.text(&tx(&format!("handbuch.{k}.titel")), r.x + 8.0, r.y + 20.0, 15.0, Theme::text());
        if ui.hit(r) { v.tab = i; }
    }
    let r = Rect::new(x0() + 316.0, Y0, cbreite() - 312.0, hoehe() - Y0 - 16.0);
    let k = kap.get(v.tab).cloned().unwrap_or_default();
    ui.card_titled(r, &tx(&format!("handbuch.{k}.titel")), Theme::gold());
    let mut yy = r.y + 64.0;
    for teil in [crate::id("kurz"), crate::id("was"), crate::id("ablauf"), crate::id("wirkung"), crate::id("mehr"), crate::id("tipps")] {
        let s = tx(&format!("handbuch.{k}.{teil}"));
        if s.is_empty() { continue; }
        if teil != crate::id("kurz") { ui.text_semi(&tx(&format!("handbuch.abschnitt.{teil}")), r.x + 14.0, yy, 15.0, Theme::akzent()); yy += 22.0; }
        let n = ui.wrap(&s, 15.0, r.w - 28.0).len() as f32;
        ui.paragraph(&s, r.x + 14.0, yy, 15.0, r.w - 28.0, Theme::text());
        yy += n * 21.0 + 12.0;
        if yy > r.y + r.h - 20.0 { break; }
    }
}

pub(super) fn regeln_seite(ui: &mut Ui, v: &mut View, _c: &mut Career) {
    let ordner = crate::regeln::eigene_regeln();
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.seiten.regeldateien"), Theme::akzent());
    ui.paragraph(&crate::tf!("ui.seiten.alle_zahlen_stehen_in_regeldateien", ordner.display()), l.x + 14.0, l.y + 60.0, 15.0, l.w - 28.0, Theme::text2());
    for (i, (n, eigen)) in crate::regeln::dateien().iter().enumerate() {
        let yy = l.y + 150.0 + i as f32 * 26.0;
        if yy > l.y + l.h - 10.0 { break; }
        ui.text(n, l.x + 14.0, yy, 15.0, Theme::text());
        if *eigen { ui.text_right(crate::inhalt::t("ui.wort.angepasst"), l.x + l.w - 150.0, yy, 14.0, Theme::achtung()); }
        else if ui.button(Rect::new(l.x + l.w - 130.0, yy - 18.0, 116.0, 24.0), crate::inhalt::t("ui.seiten.anpassen"), false) { match crate::regeln::kopie_anlegen(n) { Ok(p) => v.toast(&crate::tf!("ui.seiten.kopie", p.display()), true), Err(e) => v.toast(&e, false) } }
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.so_geht_s"), Theme::gold());
    ui.paragraph(crate::inhalt::t("ui.seiten.anpassen_legt_eine_kopie_der"), r.x + 14.0, r.y + 60.0, 15.0, r.w - 28.0, Theme::text2());
    for (k, f) in crate::regeln::fehler().iter().enumerate() { ui.paragraph(f, r.x + 14.0, r.y + 200.0 + k as f32 * 40.0, 15.0, r.w - 28.0, Theme::problem()); }
}

pub(super) fn profi(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("pm.f.diagnose"), Theme::akzent());
    let t = TAGESZEIT.lock().map(|x| x.clone()).unwrap_or_default();
    let mittel = if t.is_empty() { 0.0 } else { t.iter().sum::<f32>() / t.len() as f32 };
    let max = t.iter().cloned().fold(0.0, f32::max);
    let groesse = std::fs::metadata(c.save_path()).map(|m| m.len()).unwrap_or(0);
    let z = vec![(crate::inhalt::t("ui.seiten.tagesschritt_im_mittel").to_string(), crate::tf!("ui.seiten.ms", format!("{:.0}", mittel))), (crate::inhalt::t("ui.seiten.laengster_tagesschritt").into(), crate::tf!("ui.seiten.ms", format!("{:.0}", max))), (crate::inhalt::t("ui.seiten.gemessene_tage").into(), t.len().to_string()),
        (crate::inhalt::t("quelle.spielstand").into(), crate::tf!("spielstand.groesse", format!("{:.1}", groesse as f64 / 1e6))), (crate::inhalt::t("ui.seiten.spieler_in_der_welt").into(), c.world.players.len().to_string()), (crate::inhalt::t("gm.zt.z.meldungen").into(), c.inbox.len().to_string()),
        (crate::inhalt::t("ui.seiten.gruende_im_tagebuch").into(), c.mensch.gruende.len().to_string()), (crate::inhalt::t("reiter.beziehungen").into(), c.mensch.beziehungen.len().to_string()), (crate::inhalt::t("ui.seiten.angepasste_regeldateien").into(), crate::regeln::dateien().iter().filter(|x| x.1).count().to_string())];
    zeilen(ui, l, &z, 64.0);
    ui.card_titled(r, crate::id("FL27"), Theme::gold()); // roh: product name
    let z = vec![(crate::inhalt::t("ticker.kat.welt").to_string(), if c.fl27_welt.is_empty() { crate::inhalt::t("ui.main.eingebaute_probewelt_2").into() } else { c.fl27_welt.clone() }), (crate::inhalt::t("ui.seiten.spiel_starten_erlaubt").into(), if c.fl27_erlaubt { crate::id("ja").into() } else { crate::id("nein").into() }), (crate::inhalt::t("ui.seiten.karriere_ordner_im_spiel").into(), c.fl27_cpk.clone()), (crate::inhalt::t("ui.seiten.in_fl27_gespielte_spiele").into(), c.fl27_spiele.len().to_string())];
    zeilen(ui, r, &z, 64.0);
}
