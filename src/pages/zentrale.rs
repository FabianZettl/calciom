use super::*;

/// „Was jetzt?“ (Spielgefühl Teil 4, Maßnahme 1): the one thing to do now, why, until when, and a button that goes there.
fn was_jetzt_karte(ui: &mut Ui, v: &mut View, c: &Career, r: Rect) {
    let w = c.was_jetzt();
    let farbe = if w.vorrang { Theme::problem() } else { Theme::akzent() };
    ui.card_titled(r, crate::inhalt::t("ui.wj.titel"), farbe);
    if !w.frist.is_empty() {
        let fw = ui.measure_semi(&w.frist, 13.0) + 16.0;
        rrect(r.x + r.w - fw - 12.0, r.y + 7.0, fw, 22.0, 11.0, alpha(hex(0xffffff), 0.25));
        ui.text_semi(&w.frist, r.x + r.w - fw - 4.0, r.y + 23.0, 13.0, on(farbe));
    }
    let kw = if w.knopf.is_empty() { 0.0 } else { (r.w * 0.36).min(190.0) };
    ui.title(&ui.fit(&w.titel, 20.0, r.w - kw - 36.0), r.x + 16.0, r.y + 66.0, 20.0, Theme::text());
    ui.text_passend(&w.grund, r.x + 16.0, r.y + 90.0, 14.0, r.w - kw - 36.0, 2, Theme::text2());
    if kw > 0.0 && ui.button(Rect::new(r.x + r.w - kw - 12.0, r.y + 58.0, kw, 36.0), &w.knopf, w.vorrang) {
        if let Some(p) = w.ziel { v.go(p); }
        if w.msg.is_some() { v.sel_msg = w.msg; }
    }
}
/// Saisonreise (Maßnahme 2): the stages of the season on a band, today's marker, the season goal as a bar.
fn saisonreise_karte(ui: &mut Ui, c: &Career, r: Rect) {
    ui.card_titled(r, crate::inhalt::t("ui.reise.titel"), Theme::akzent());
    let (etappen, letzt, (ziel, platz, teams)) = c.saisonreise();
    let (bx, bw, by) = (r.x + 22.0, r.w - 44.0, r.y + 56.0);
    let pos = |d: u32| bx + bw * (d.min(letzt) as f32 / letzt as f32);
    rrect(bx, by - 2.0, bw, 4.0, 2.0, Theme::linie());
    rrect(bx, by - 2.0, pos(c.world.day) - bx, 4.0, 2.0, Theme::verein());
    for (k, (key, tag)) in etappen.iter().enumerate() {
        let x = pos(*tag);
        let erreicht = c.world.day >= *tag;
        draw_circle(x, by, 5.0, if erreicht { Theme::verein() } else { Theme::flaeche() });
        draw_circle_lines(x, by, 5.0, 1.5, if erreicht { Theme::verein() } else { Theme::gedaempft() });
        // labels alternate above and below the band so neighbours never overlap
        let ly = if k % 2 == 0 { by + 18.0 } else { by - 9.0 };
        let l = crate::inhalt::t(key);
        let lw = ui.measure(l, 11.0);
        ui.text(l, (x - lw / 2.0).clamp(r.x + 6.0, r.x + r.w - lw - 6.0), ly, 11.0, if erreicht { Theme::text2() } else { Theme::gedaempft() });
    }
    let hx = pos(c.world.day);
    draw_triangle(vec2(hx - 6.0, by - 12.0), vec2(hx + 6.0, by - 12.0), vec2(hx, by - 4.0), Theme::problem());
    // season goal: full when the place is reached, else how far up the table there is still to go
    let anteil = if platz <= ziel { 1.0 } else { (teams.saturating_sub(platz)) as f32 / (teams.saturating_sub(ziel)).max(1) as f32 };
    let col = if anteil >= 1.0 { Theme::gut() } else if anteil >= 0.6 { Theme::achtung() } else { Theme::problem() };
    // the goal on its own line under the stage labels: text on the left, bar on the right
    let gy = r.y + r.h - 16.0;
    let zt = crate::tf!("ui.reise.ziel", ziel, platz);
    let tw = ui.measure(&zt, 12.0) + 12.0;
    ui.text(&zt, r.x + 16.0, gy + 3.0, 12.0, Theme::text2());
    rrect(r.x + 16.0 + tw, gy - 4.0, r.w - 32.0 - tw, 7.0, 3.5, Theme::ueberlagerung());
    rrect(r.x + 16.0 + tw, gy - 4.0, (r.w - 32.0 - tw) * anteil.clamp(0.02, 1.0), 7.0, 3.5, col);
}
/// Geführte erste Woche (Maßnahme 9): five steps with ticks, the count, and a button to hide it.
fn erste_woche_karte(ui: &mut Ui, c: &mut Career, r: Rect) {
    let n = c.erste_woche.iter().filter(|x| **x).count();
    ui.card_titled(r, &crate::tf!("profil.kurz_text", crate::inhalt::t("ui.woche.titel"), crate::tf!("ui.woche.stand", n)), Theme::gold());
    for (i, key) in crate::spielgefuehl::SCHRITTE.iter().enumerate() {
        let y = r.y + 50.0 + i as f32 * 12.5;
        let fertig = c.erste_woche[i];
        if fertig { draw_circle(r.x + 22.0, y - 4.0, 4.5, Theme::gut()); } else { draw_circle_lines(r.x + 22.0, y - 4.0, 4.5, 1.2, Theme::gedaempft()); }
        ui.text(crate::inhalt::t(key), r.x + 34.0, y, 11.5, if fertig { Theme::gedaempft() } else { Theme::text() });
    }
    if ui.button(Rect::new(r.x + r.w - 92.0, r.y + r.h - 30.0, 80.0, 22.0), crate::inhalt::t("ui.woche.aus"), false) { c.erste_woche_aus = true; }
}

pub(super) fn zentrale(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // Spielgefühl Teil 4: the one thing to do now, the season journey and, while it runs, the guided first week
    let kopf_h = 112.0;
    let woche = c.erste_woche_offen();
    let bw = cbreite();
    // the manager's photo (Fabian 23.11.): a click picks a PNG, kept in the career folder
    let fw = 104.0;
    managerfoto(ui, v, c, Rect::new(x0() + 4.0, Y0, fw, kopf_h));
    let bw = bw - fw - 12.0;
    let (w1, w2) = if woche { (bw * 0.4, bw * 0.34) } else { (bw * 0.5 - 6.0, bw * 0.5 - 6.0) };
    let hx = crate::pages::x0() + fw + 12.0;
    was_jetzt_karte(ui, v, c, Rect::new(hx + 4.0, Y0, w1, kopf_h));
    saisonreise_karte(ui, c, Rect::new(hx + 16.0 + w1, Y0, w2, kopf_h));
    if woche { erste_woche_karte(ui, c, Rect::new(hx + 28.0 + w1 + w2, Y0, bw - w1 - w2 - 24.0, kopf_h)); }
    let y0 = Y0 + kopf_h + 12.0;
    zentrale_dashboard(ui, v, c, Rect::new(x0() + 4.0, y0, cbreite(), hoehe() - y0 - 16.0));
}

static FOTO_DIALOG: std::sync::Mutex<Option<std::sync::mpsc::Receiver<Option<std::path::PathBuf>>>> = std::sync::Mutex::new(None);
thread_local! { static FOTO: std::cell::RefCell<Option<(std::time::SystemTime, Texture2D)>> = const { std::cell::RefCell::new(None) }; }
fn managerfoto(ui: &mut Ui, v: &mut View, c: &mut Career, r: Rect) {
    let datei = c.karriere_dir().join("manager.png");
    let fertig = FOTO_DIALOG.lock().unwrap().as_ref().and_then(|rx| rx.try_recv().ok());
    if let Some(g) = fertig {
        *FOTO_DIALOG.lock().unwrap() = None;
        if let Some(p) = g {
            match std::fs::read(&p).ok().filter(|b| Image::from_file_with_format(b, Some(ImageFormat::Png)).is_ok()) {
                Some(b) => { let _ = std::fs::create_dir_all(c.karriere_dir()); let _ = std::fs::write(&datei, b); FOTO.with(|f| *f.borrow_mut() = None); }
                None => v.toast(crate::inhalt::t("ui.manager.nur_png"), false),
            }
        }
    }
    ui.card(r);
    let zeit = std::fs::metadata(&datei).and_then(|m| m.modified()).ok();
    let tex = zeit.and_then(|z| FOTO.with(|f| {
        let mut f = f.borrow_mut();
        if f.as_ref().map_or(true, |x| x.0 != z) { *f = std::fs::read(&datei).ok().and_then(|b| Image::from_file_with_format(&b, Some(ImageFormat::Png)).ok()).map(|i| { let t = Texture2D::from_image(&i); t.set_filter(FilterMode::Linear); (z, t) }); }
        f.as_ref().map(|x| x.1.clone())
    }));
    let bild = Rect::new(r.x + 8.0, r.y + 8.0, r.w - 16.0, r.h - 30.0);
    match tex {
        Some(t) => {
            // centre crop to the box's shape
            let (tw, th) = (t.width(), t.height());
            let ziel = bild.w / bild.h;
            let (sw, sh) = if tw / th > ziel { (th * ziel, th) } else { (tw, tw / ziel) };
            draw_texture_ex(&t, bild.x, bild.y, WHITE, DrawTextureParams { dest_size: Some(vec2(bild.w, bild.h)), source: Some(Rect::new((tw - sw) / 2.0, (th - sh) / 2.0, sw, sh)), ..Default::default() });
        }
        None => {
            rrect(bild.x, bild.y, bild.w, bild.h, 6.0, Theme::erhoehung());
            ui.paragraph(crate::inhalt::t("ui.manager.foto_waehlen"), bild.x + 6.0, bild.y + bild.h / 2.0 - 8.0, 12.0, bild.w - 12.0, Theme::gedaempft());
        }
    }
    ui.text_center(&ui.fit(&c.coach, 13.0, r.w - 8.0), r.x + r.w / 2.0, r.y + r.h - 8.0, 13.0, Theme::text());
    if ui.hit(bild) && FOTO_DIALOG.lock().unwrap().is_none() { *FOTO_DIALOG.lock().unwrap() = Some(crate::fl27::bild_dialog()); }
}

/// The dashboard of the Zentrale (Fabian 23.11.: too empty): table around us, last and next match, form of our players
/// (best and worst), season figures, the league's best players and scorers. Every player opens his profile.
fn zentrale_dashboard(ui: &mut Ui, v: &mut View, c: &mut Career, r: Rect) {
    let t = crate::inhalt::t;
    let w3 = (r.w - 24.0) / 3.0;
    let spalte = |k: usize| r.x + k as f32 * (w3 + 12.0);
    let zeilen_h = 26.0;
    let form = |p: usize| { let pl = &c.world.players[p]; let l: Vec<f32> = pl.last.iter().rev().take(3).copied().collect(); if l.is_empty() { None } else { Some(l.iter().sum::<f32>() / l.len() as f32) } };
    let mut klick: Option<usize> = None;
    // column 1: table around us and the matches
    let l = c.league();
    let tab = c.world.table(l);
    let pos = tab.iter().position(|x| x.club == c.own).unwrap_or(0);
    let h1 = (r.h * 0.58).max(250.0);
    let k1 = Rect::new(spalte(0), r.y, w3, h1);
    ui.card_titled(k1, &crate::tf!("ui.zd.tabelle", c.world.leagues[l].name.clone()), Theme::akzent());
    let n = ((k1.h - 56.0) / zeilen_h) as usize;
    let ab = pos.saturating_sub(n / 2).min(tab.len().saturating_sub(n));
    for (k, row) in tab.iter().enumerate().skip(ab).take(n) {
        let y = k1.y + 58.0 + (k - ab) as f32 * zeilen_h;
        if row.club == c.own { rrect(k1.x + 6.0, y - 18.0, k1.w - 12.0, 24.0, 4.0, alpha(Theme::akzent(), 0.16)); }
        let cl = &c.world.clubs[row.club];
        ui.text(&format!("{}", k + 1), k1.x + 14.0, y, 14.0, Theme::text2());
        ui.club_crest(k1.x + 48.0, y - 5.0, 8.0, cl.id, hex(cl.c1), &cl.short);
        ui.text(&ui.fit(&cl.name, 14.0, k1.w - 170.0), k1.x + 62.0, y, 14.0, Theme::text());
        ui.text_right(&format!("{}  {:+}  {}", row.p, row.gf - row.ga, row.pts), k1.x + k1.w - 14.0, y, 14.0, Theme::text());
    }
    let k2 = Rect::new(spalte(0), r.y + h1 + 12.0, w3, r.h - h1 - 12.0);
    ui.card_titled(k2, t("ui.zd.spiele"), Theme::gold());
    let mut y = k2.y + 58.0;
    if let Some((wo, f)) = c.own_fixtures().into_iter().find(|x| x.1.hg < 0) {
        let g = if f.h == c.own { f.a } else { f.h };
        let cl = &c.world.clubs[g];
        ui.club_crest(k2.x + 26.0, y - 5.0, 12.0, cl.id, hex(cl.c1), &cl.short);
        ui.text_semi(&ui.fit(&crate::tf!("ui.zd.naechstes", cl.name.clone(), date::short(c.world.season, f.day)), 15.0, k2.w - 60.0), k2.x + 46.0, y, 15.0, Theme::text());
        // the competition with its logo (Fabian 24.11.)
        y += 24.0;
        let name = match wo { Wo::Liga(l) => c.world.leagues[l].name.clone(), Wo::Pokal(k) => c.world.cups[k].name.clone() };
        let lx = if ui.wettbewerb_logo_auf(k2.x + 26.0, y - 5.0, 10.0, c.comp_cid(wo), Some(wo_look(c, wo).c1)) { k2.x + 46.0 } else { k2.x + 14.0 };
        ui.text(&ui.fit(&name, 14.0, k2.w - 60.0), lx, y, 14.0, Theme::text2());
        y += 30.0;
    }
    for s in c.spiele.iter().rev().take(((k2.y + k2.h - y) / 24.0).max(0.0) as usize) {
        let cl = &c.world.clubs[s.gegner];
        let col = if s.tore > s.gegentore { Theme::gut() } else if s.tore < s.gegentore { Theme::problem() } else { Theme::achtung() };
        rrect(k2.x + 14.0, y - 14.0, 40.0, 18.0, 4.0, col);
        ui.text_center(&format!("{}:{}", s.tore, s.gegentore), k2.x + 34.0, y, 13.0, hex(0xffffff));
        ui.text(&ui.fit(&format!("{} {}", if s.heim { t("ui.zd.gegen") } else { "@" }, cl.name), 14.0, k2.w - 80.0), k2.x + 62.0, y, 14.0, Theme::text()); // roh
        y += 24.0;
    }
    // column 2: our players' form, best and worst, and the season in figures
    let mut sq: Vec<(usize, f32)> = c.squad().into_iter().filter_map(|p| form(p).map(|f| (p, f))).collect();
    sq.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let h2 = (r.h * 0.58).max(250.0);
    let k3 = Rect::new(spalte(1), r.y, w3, h2);
    ui.card_titled(k3, t("ui.zd.form"), Theme::gut());
    let mut y = k3.y + 58.0;
    let zeile = |ui: &mut Ui, x: f32, y: f32, w: f32, p: usize, wert: String, farbe: Color| -> bool {
        let pl = &c.world.players[p];
        let rr = Rect::new(x + 6.0, y - 18.0, w - 12.0, 24.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
        ui.player_photo(x + 22.0, y - 5.0, 10.0, pl.id, &pl.name);
        ui.pos_tag(x + 38.0, y - 5.0, pos_de(&pl.pos));
        ui.text(&ui.fit(&pl.name, 14.0, w - 154.0), x + 82.0, y, 14.0, Theme::text());
        ui.text_right(&wert, x + w - 14.0, y, 14.0, farbe);
        ui.hit(rr)
    };
    if sq.is_empty() { ui.text(t("ui.zd.noch_keine_noten"), k3.x + 14.0, y, 14.0, Theme::text2()); }
    ui.text_semi(t("ui.zd.beste_form"), k3.x + 14.0, y, 12.0, Theme::gedaempft()); y += 22.0;
    let platz = (((k3.h - 100.0) / zeilen_h) as usize / 2).max(2);
    for &(p, f) in sq.iter().take(platz) { if zeile(ui, k3.x, y, k3.w, p, format!("{f:.1}"), Theme::gut()) { klick = Some(p); } y += zeilen_h; }
    y += 6.0;
    ui.text_semi(t("ui.zd.schwaechste_form"), k3.x + 14.0, y, 12.0, Theme::gedaempft()); y += 22.0;
    for &(p, f) in sq.iter().rev().take(platz.min(sq.len().saturating_sub(platz))) { if y > k3.y + k3.h - 8.0 { break; } if zeile(ui, k3.x, y, k3.w, p, format!("{f:.1}"), Theme::problem()) { klick = Some(p); } y += zeilen_h; }
    let k4 = Rect::new(spalte(1), r.y + h2 + 12.0, w3, r.h - h2 - 12.0);
    ui.card_titled(k4, t("ui.zd.saison"), Theme::akzent());
    let own = tab.iter().find(|x| x.club == c.own);
    let spiele = own.map_or(0, |x| x.p).max(1) as f32;
    let torjaeger = c.squad().into_iter().max_by_key(|&p| c.world.players[p].goals);
    let vorlagen = c.squad().into_iter().max_by_key(|&p| c.world.players[p].assists);
    let schnitt = { let v: Vec<f32> = c.squad().iter().filter(|&&p| c.world.players[p].rn > 0).map(|&p| c.world.players[p].avg_rating()).collect(); if v.is_empty() { 0.0 } else { v.iter().sum::<f32>() / v.len() as f32 } };
    let werte: Vec<(String, String)> = vec![
        (t("ui.zd.tore_spiel").into(), format!("{:.1}", own.map_or(0, |x| x.gf) as f32 / spiele)),
        (t("ui.zd.gegentore_spiel").into(), format!("{:.1}", own.map_or(0, |x| x.ga) as f32 / spiele)),
        (t("ui.zd.punkte_spiel").into(), format!("{:.2}", own.map_or(0, |x| x.pts) as f32 / spiele)),
        (t("ui.zd.notenschnitt").into(), if schnitt > 0.0 { format!("{schnitt:.2}") } else { "–".into() }),
        (t("ui.zd.torjaeger").into(), torjaeger.filter(|&p| c.world.players[p].goals > 0).map(|p| format!("{} ({})", c.world.players[p].name, c.world.players[p].goals)).unwrap_or("–".into())),
        (t("ui.zd.vorlagen").into(), vorlagen.filter(|&p| c.world.players[p].assists > 0).map(|p| format!("{} ({})", c.world.players[p].name, c.world.players[p].assists)).unwrap_or("–".into())),
    ];
    let mut y = k4.y + 58.0;
    for (n, w) in werte { if y > k4.y + k4.h - 6.0 { break; } ui.text(&n, k4.x + 14.0, y, 14.0, Theme::text2()); ui.text_right(&ui.fit(&w, 14.0, k4.w * 0.55), k4.x + k4.w - 14.0, y, 14.0, Theme::text()); y += 24.0; }
    // column 3: the league's best players and scorers
    // league figures only (youth, second-team and cup games stay out)
    let mut alle: Vec<(usize, crate::world::LigaStat)> = c.world.players.iter().enumerate().filter_map(|(p, pl)| pl.profil.ligen.iter().find(|x| x.l as usize == l).map(|x| (p, x.clone()))).collect();
    alle.sort_unstable_by_key(|x| x.0);
    let mut beste: Vec<(usize, f32)> = alle.iter().filter(|x| x.1.rn >= 3).map(|x| (x.0, x.1.note())).collect();
    beste.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let k5 = Rect::new(spalte(2), r.y, w3, h1);
    ui.card_titled(k5, t("ui.zd.liga_beste"), Theme::gold());
    let mut y = k5.y + 58.0;
    if beste.is_empty() { ui.text(t("ui.zd.noch_keine_noten"), k5.x + 14.0, y, 14.0, Theme::text2()); }
    for &(p, n) in beste.iter().take(((k5.h - 60.0) / zeilen_h) as usize) {
        let pl = &c.world.players[p];
        let cl = &c.world.clubs[pl.team.max(0) as usize];
        ui.club_crest(k5.x + k5.w - 70.0, y - 5.0, 7.0, cl.id, hex(cl.c1), &cl.short);
        if zeile(ui, k5.x, y, k5.w - 20.0, p, format!("{n:.2}"), if pl.team == c.own as i32 { Theme::gold() } else { Theme::text() }) { klick = Some(p); }
        y += zeilen_h;
    }
    let mut tore: Vec<(usize, u16)> = alle.iter().filter(|x| x.1.goals > 0).map(|x| (x.0, x.1.goals)).collect();
    tore.sort_by_key(|x| std::cmp::Reverse(x.1));
    let k6 = Rect::new(spalte(2), r.y + h1 + 12.0, w3, r.h - h1 - 12.0);
    ui.card_titled(k6, t("ui.zd.liga_tore"), Theme::problem());
    let mut y = k6.y + 58.0;
    if tore.is_empty() { ui.text(t("statistik.keine_tore"), k6.x + 14.0, y, 14.0, Theme::text2()); }
    for &(p, g) in tore.iter().take(((k6.h - 60.0) / zeilen_h) as usize) { if zeile(ui, k6.x, y, k6.w, p, format!("{g}"), Theme::text()) { klick = Some(p); } y += zeilen_h; }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
}
