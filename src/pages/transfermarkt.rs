use super::*;

// ================================================================ Transfermarkt
/// Search profiles (suchprofil.rs): editor on the left, saved profiles and presets, the hits on the right; a scout can take the profile along.
fn suchprofile_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::suchprofil::{ATTRIBUTE, POSITIONEN};
    let t = crate::inhalt::t;
    let lw = cbreite() * 0.42;
    let r = Rect::new(x0() + 4.0, Y0, lw, hoehe() - Y0 - 16.0);
    ui.card_titled(r, t("sup.neu"), Theme::akzent());
    let k = &mut v.sp_edit;
    let mut y = r.y + 56.0;
    ui.text_semi(t("sup.pos"), r.x + 14.0, y + 4.0, 14.0, Theme::text2());
    y += 12.0;
    let bw = (r.w - 28.0 - 12.0 * 6.0) / 13.0;
    for (i, ps) in POSITIONEN.iter().enumerate() {
        let br = Rect::new(r.x + 14.0 + i as f32 * (bw + 6.0), y, bw, 28.0);
        let an = k.pos.iter().any(|x| x == ps);
        rrect(br.x, br.y, br.w, br.h, 6.0, if an { Theme::teil(ps) } else { alpha(Theme::teil(ps), 0.12) });
        ui.text_center(pos_de(ps), br.x + br.w / 2.0, br.y + 19.0, 13.0, if an { hex(0xffffff) } else { Theme::teil(ps) });
        if ui.hit(br) { if an { k.pos.retain(|x| x != ps); } else { k.pos.push(ps.to_string()); } }
    }
    y += 44.0;
    // the number criteria as sliders; 0 = any
    let egal = t("sup.egal");
    let zeilen: [(&str, i32, i32, String); 7] = [
        (t("sup.alter_min"), k.alter_min as i32, 40, if k.alter_min == 0 { egal.into() } else { k.alter_min.to_string() }),
        (t("sup.alter_max"), k.alter_max as i32, 40, if k.alter_max == 0 { egal.into() } else { k.alter_max.to_string() }),
        (t("sup.staerke_min"), k.staerke_min as i32, 95, if k.staerke_min == 0 { egal.into() } else { k.staerke_min.to_string() }),
        (t("sup.potenzial_min"), k.pot_min as i32, 99, if k.pot_min == 0 { egal.into() } else { k.pot_min.to_string() }),
        (t("sup.vertrag"), k.vertrag_monate as i32, 36, if k.vertrag_monate == 0 { egal.into() } else { crate::tf!("sup.monate", k.vertrag_monate) }),
        (t("sup.wert_max"), k.wert_max as i32, 200, if k.wert_max == 0 { egal.into() } else { crate::tf!("sup.mio", k.wert_max) }),
        (t("sup.gehalt_max"), k.gehalt_max as i32, 30, if k.gehalt_max == 0 { egal.into() } else { crate::tf!("sup.mio", k.gehalt_max) }),
    ];
    for (i, (n, wert, max, txt)) in zeilen.iter().enumerate() {
        ui.text(n, r.x + 14.0, y + 14.0, 15.0, Theme::text2());
        ui.text_right(txt, r.x + r.w - 14.0, y + 14.0, 15.0, Theme::text());
        if let Some(x) = ui.slider(Rect::new(r.x + 200.0, y + 4.0, r.w - 300.0, 14.0), *wert, 0, *max) {
            let x = x as u32;
            match i { 0 => k.alter_min = x as u8, 1 => k.alter_max = x as u8, 2 => k.staerke_min = x as u8, 3 => k.pot_min = x as u8, 4 => k.vertrag_monate = x as u8, 5 => k.wert_max = x, _ => k.gehalt_max = x }
        }
        y += 34.0;
    }
    ui.text(t("sup.fuss"), r.x + 14.0, y + 20.0, 15.0, Theme::text2());
    if let Some(s) = ui.segment(Rect::new(r.x + 200.0, y + 2.0, r.w - 214.0, 28.0), &[egal, t("sup.fuss.rechts"), t("sup.fuss.links")], k.fuss as usize) { k.fuss = s as u8; }
    y += 42.0;
    // one ability minimum: the name cycles on click
    ui.text(t("sup.werte"), r.x + 14.0, y + 14.0, 15.0, Theme::text2());
    y += 22.0;
    let (ai, am) = k.attr.first().copied().unwrap_or((0, 0));
    let an = crate::inhalt::t(&format!("faehigkeit.{}", crate::entwicklung::NAMEN[ATTRIBUTE[ai as usize % ATTRIBUTE.len()]])); // roh
    if ui.button(Rect::new(r.x + 14.0, y, 180.0, 28.0), an, false) { let n = ((ai as usize + 1) % ATTRIBUTE.len()) as u8; k.attr = vec![(n, am)]; }
    ui.text_right(&if am == 0 { egal.to_string() } else { am.to_string() }, r.x + r.w - 14.0, y + 20.0, 15.0, Theme::text());
    if let Some(x) = ui.slider(Rect::new(r.x + 200.0, y + 8.0, r.w - 300.0, 14.0), am as i32, 0, 95) { k.attr = if x == 0 { vec![] } else { vec![(ai, x as u8)] }; }
    y += 46.0;
    let kk = k.clone();
    if ui.button(Rect::new(r.x + 14.0, y, 200.0, 32.0), t("sup.speichern"), true) {
        let mut p = kk.clone();
        if p.name.is_empty() || c.profil_vorlagen().iter().any(|x| x.name == p.name) { p.name = crate::tf!("sup.name_vorgabe", c.suchprofile.len() + 1); }
        c.suchprofile.retain(|x| x.name != p.name);
        c.suchprofile.push(p.clone());
        if c.suchprofile.len() > 8 { c.suchprofile.remove(0); }
        v.sp_edit.name = p.name.clone();
        v.toast(&crate::tf!("sup.ok.gespeichert", p.name), true);
    }
    if ui.button(Rect::new(r.x + 224.0, y, 220.0, 32.0), t("sup.beauftragen"), false) {
        let mut p = kk.clone();
        if p.name.is_empty() { p.name = crate::tf!("sup.name_vorgabe", c.suchprofile.len() + 1); }
        match c.profil_beauftragen(&p, 42) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
    }
    // saved profiles and presets as chips
    let r2 = Rect::new(x0() + lw + 16.0, Y0, cbreite() - lw - 12.0, 140.0);
    ui.card_titled(r2, t("sup.meine"), Theme::gold());
    let mut x = r2.x + 14.0;
    let mut yy = r2.y + 54.0;
    let mut weg = None;
    let alle: Vec<(bool, crate::suchprofil::Suchprofil)> = c.suchprofile.iter().cloned().map(|p| (true, p)).chain(c.profil_vorlagen().into_iter().map(|p| (false, p))).collect();
    for (i, (eigen, p)) in alle.iter().enumerate() {
        let w = ui.measure(&p.name, 14.0) + 28.0 + if *eigen { 22.0 } else { 0.0 };
        if x + w > r2.x + r2.w - 14.0 { x = r2.x + 14.0; yy += 38.0; }
        let br = Rect::new(x, yy, w, 30.0);
        let an = v.sp_edit == *p;
        rrect(br.x, br.y, br.w, br.h, 15.0, if an { Theme::akzent() } else if *eigen { Theme::ueberlagerung() } else { Theme::erhoehung() });
        ui.text(&p.name, br.x + 14.0, br.y + 20.0, 14.0, if an { Theme::auf_akzent() } else { Theme::text() });
        if *eigen {
            let xr = Rect::new(br.x + br.w - 26.0, br.y + 4.0, 22.0, 22.0);
            ui.text_center("×", xr.x + 11.0, xr.y + 16.0, 15.0, if an { Theme::auf_akzent() } else { Theme::gedaempft() }); // roh
            if ui.hit(xr) { weg = Some(i); continue; }
        }
        if ui.hit(br) { v.sp_edit = p.clone(); }
        x += w + 8.0;
    }
    if let Some(i) = weg { let n = c.suchprofile[i].name.clone(); c.suchprofile.remove(i); v.toast(&crate::tf!("sup.ok.geloescht", n), true); }
    if alle.is_empty() { ui.paragraph(t("sup.keine"), r2.x + 14.0, r2.y + 60.0, 15.0, r2.w - 28.0, Theme::text2()); }
    // hits, cached per profile and day
    let r3 = Rect::new(r2.x, r2.y + r2.h + 10.0, r2.w, hoehe() - r2.y - r2.h - 26.0);
    ui.card_titled(r3, t("sup.treffer"), Theme::akzent());
    if v.sp_treffer.0 != v.sp_edit || v.sp_treffer.1 != c.abs() { v.sp_treffer = (v.sp_edit.clone(), c.abs(), c.profil_treffer(&v.sp_edit, 200)); }
    let liste = v.sp_treffer.2.clone();
    if liste.is_empty() { ui.paragraph(t("sup.keine_treffer"), r3.x + 14.0, r3.y + 60.0, 15.0, r3.w - 28.0, Theme::text2()); return; }
    let area = Rect::new(r3.x, r3.y + 44.0, r3.w, r3.h - 48.0);
    let vis = ((area.h - 8.0) / 34.0) as usize;
    let start = scroll_rows(ui, v, area, liste.len(), vis);
    for (k, &p) in liste.iter().skip(start).take(vis).enumerate() {
        let pl = &c.world.players[p];
        let rr = Rect::new(area.x + 6.0, area.y + k as f32 * 34.0, area.w - 12.0, 32.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
        let cy = rr.y + 22.0;
        ui.player_photo(rr.x + 16.0, rr.y + 16.0, 12.0, pl.id, &pl.name);
        ui.text_semi(&ui.fit(&pl.name, 16.0, 168.0), rr.x + 34.0, cy, 16.0, Theme::text());
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { t("ui.wort.vereinslos").into() };
        let cw = rr.w - 270.0 - 210.0 - 10.0;
        if cw > 40.0 { ui.text(&ui.fit(&club, 14.0, cw), rr.x + 210.0, cy, 14.0, Theme::text2()); }
        ui.text(&pl.age.to_string(), rr.x + rr.w - 270.0, cy, 15.0, Theme::text());
        ui.pos_tag(rr.x + rr.w - 238.0, cy - 5.0, pos_de(&pl.pos));
        strength_cell(ui, c, p, rr.x + rr.w - 150.0, cy - 6.0);
        ui.text_right(&money(pl.value), rr.x + rr.w - 8.0, cy, 15.0, Theme::text());
        if ui.hit(rr) { v.sel_player = Some(p); v.go(Page::Spielerprofil); return; }
    }
}

/// The numbers the free search can sort by (per 90 or a quote).
const MARKT_ZAHLEN: [(&str, fn(&crate::statistik::Stat) -> Option<f32>); 8] = [
    (crate::id("einfluss"), crate::statistik::einfluss90),
    (crate::id("xg"), |s| crate::statistik::pro90(s, crate::statistik::XG)), (crate::id("xa"), |s| crate::statistik::pro90(s, crate::statistik::XA)),
    (crate::id("schluesselpaesse"), |s| crate::statistik::pro90(s, crate::statistik::SCHLUESSEL)), ("prog_paesse", |s| crate::statistik::pro90(s, crate::statistik::PROG_PAESSE)),
    (crate::id("dribblings"), |s| crate::statistik::pro90(s, crate::statistik::DRIBB)), (crate::id("tacklings"), |s| crate::statistik::pro90(s, crate::statistik::TACKL)),
    (crate::id("abgefangen"), |s| crate::statistik::pro90(s, crate::statistik::ABGEF)),
];
/// Whose extended numbers we see (Konzept Spielerstatistiken 8.1): own league, detailed leagues, players known from 40.
pub fn markt_erweitert(c: &Career, p: usize) -> bool {
    let pl = &c.world.players[p];
    if pl.team < 0 { return false; }
    let l = c.world.clubs[pl.team as usize].league;
    l == c.league() || c.know_of(p) >= 0.40 || c.analyst_ligen().contains(&l)
}

/// How well a player plays a PES position: 2 natural (his main position counts), 1 learned, 0 not.
fn pos_stufe(p: &crate::world::Player, k: usize) -> u8 {
    if crate::pesdb::pos_index(&p.pos) as usize == k { return 2; }
    p.pp.get(k).copied().unwrap_or(0)
}
pub(super) fn transfermarkt(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // order (Fabian 24.11.): search, transfer list, loan list, free agents, search profiles
    tabs(ui, v, &[crate::inhalt::t("menu.transfermarkt.reiter.0"), crate::inhalt::t("gl.chip.transferliste"), crate::inhalt::t("ui.leihe.leihliste"), crate::inhalt::t("gl.chip.vertragslos"), crate::inhalt::t("sup.meine")]);
    if v.tab == 4 { suchprofile_seite(ui, v, c); return; }
    let w4 = (cbreite() - 36.0) / 4.0;
    let y = Y0;
    kpi(ui, Rect::new(x0() + 4.0, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_transferbudget"), &money(c.transfer_budget), crate::inhalt::t("gl.bk.transfer.unter"), Theme::gold());
    kpi(ui, Rect::new(x0() + 16.0 + w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_kader"), &format!("{} / 40", c.squad().len()), crate::inhalt::t("gl.bk.kader.unter"), Theme::akzent());
    kpi(ui, Rect::new(x0() + 28.0 + 2.0 * w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_gehaltssumme"), &money(c.world.wage_bill(c.own)), crate::inhalt::t("gl.bk.gehaelter.unter"), Theme::gut());
    kpi(ui, Rect::new(x0() + 40.0 + 3.0 * w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_kontostand"), &money(c.club().balance), crate::inhalt::t("bereich.verein"), Theme::gold());
    let y = y + 82.0;
    let wtxt = match c.in_window() { Some((_, b)) => crate::tf!("tv.fenster.frist", b - c.world.day, date::short(c.world.season, b)), None => { let next = WINDOWS.iter().find(|(a, _)| *a > c.world.day).map(|(a, _)| date::short(c.world.season, *a)).unwrap_or_else(|| crate::inhalt::t("ui.pages.sommer").into()); crate::tf!("ui.pages.transferfenster_geschlossen_es_oeffnet_a", next) } };
    ui.card(Rect::new(x0() + 4.0, y, cbreite(), 38.0));
    ui.title(&wtxt, x0() + 18.0, y + 27.0, 20.0, Theme::text());
    let y = y + 46.0;
    let si = Rect::new(x0() + 4.0, y, 330.0, 36.0);
    if ui.hit(si) { v.market_focus = true; } else if ui.clicked { v.market_focus = false; }
    let focus = v.market_focus;
    ui.input(si, &mut v.market_text, focus, crate::inhalt::t("sup.suche.platz"));
    // the filters as dropdowns in one row and one checkbox (Fabian 24.11.: "dropdowns and checkboxes instead of so many
    // bubbles"); the list gets the room of the two rows of buttons there were before
    const ALTER: [i32; 7] = [40, 19, 21, 23, 25, 29, 32];
    const WERT: [i32; 8] = [0, 1, 5, 10, 25, 50, 100, 200];
    let alle = crate::inhalt::t("ui.wort.alle");
    let fx = x0() + 344.0;
    let cbw = 200.0;
    let dw = ((x0() + cbreite() - cbw - 10.0 - fx) / 6.0 - 8.0).max(90.0);
    let feld = |k: usize| Rect::new(fx + k as f32 * (dw + 8.0), y, dw, 36.0);
    let teile = [crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("ui.wort.gross_tw"), crate::inhalt::t("ui.wort.gross_abw"), crate::inhalt::t("ui.wort.gross_mit"), crate::inhalt::t("ui.wort.gross_ang")];
    let felder = vec![
        Auswahl { r: feld(0), titel: crate::inhalt::t("ui.tm.teil").into(), items: teile.iter().map(|x| x.to_string()).collect(), sel: v.market_pos },
        Auswahl { r: feld(1), titel: crate::inhalt::t("ui.tm.position").into(), items: std::iter::once(alle.to_string()).chain(crate::pesdb::POS.iter().map(|p| crate::world::pos_de(p).to_string())).collect(), sel: v.market_genau },
        Auswahl { r: feld(2), titel: crate::inhalt::t("ui.tm.liga").into(), items: vec![crate::inhalt::t("ui.pages.alle_ligen").into(), crate::inhalt::t("ui.pages.eigene_liga").into()], sel: v.market_scope },
        Auswahl { r: feld(3), titel: crate::inhalt::t("ui.wort.gross_alter").into(), items: ALTER.iter().map(|&a| if a == 40 { alle.to_string() } else { format!("≤ {a}") }).collect(), sel: ALTER.iter().position(|&a| a == v.market_age).unwrap_or(0) },
        Auswahl { r: feld(4), titel: crate::inhalt::t("ui.pages.wert_mio").into(), items: WERT.iter().map(|&w| if w == 0 { alle.to_string() } else { format!("≤ {w}") }).collect(), sel: WERT.iter().position(|&w| w == v.market_price).unwrap_or(0) },
        Auswahl { r: feld(5), titel: crate::inhalt::t("ui.tm.zahl").into(), items: std::iter::once(crate::inhalt::t("ui.markt.zahl_keine").to_string()).chain(MARKT_ZAHLEN.iter().map(|z| crate::inhalt::t(&format!("ui.stat.{}", z.0)).to_string())).collect(), sel: v.market_stat },
    ];
    if let Some((f, i)) = auswahl_vorher(ui, v, &felder) {
        match f { 0 => v.market_pos = i, 1 => v.market_genau = i, 2 => v.market_scope = i, 3 => v.market_age = ALTER[i], 4 => v.market_price = WERT[i], _ => v.market_stat = i }
        v.scroll = 0.0; v.scrolls.clear();
    }
    // checkbox: only players who play the chosen position well (natural)
    let cb = Rect::new(x0() + cbreite() - cbw, y, cbw, 36.0);
    let an = v.market_gut == 1;
    rrect(cb.x + 6.0, cb.y + 9.0, 18.0, 18.0, 4.0, if an { Theme::akzent() } else { Theme::flaeche() });
    rrect_line(cb.x + 6.0, cb.y + 9.0, 18.0, 18.0, 4.0, if an { Theme::akzent() } else { Theme::linie() });
    if an { draw_line(cb.x + 10.0, cb.y + 18.0, cb.x + 14.0, cb.y + 22.0, 2.0, hex(0xffffff)); draw_line(cb.x + 14.0, cb.y + 22.0, cb.x + 21.0, cb.y + 13.0, 2.0, hex(0xffffff)); }
    ui.text(&ui.fit(crate::inhalt::t("ui.pages.kann_gut_spielen"), 13.0, cbw - 34.0), cb.x + 30.0, cb.y + 23.0, 13.0, Theme::text());
    if ui.hit(cb) { v.market_gut = 1 - v.market_gut.min(1); v.scroll = 0.0; v.scrolls.clear(); }
    let key = format!("{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}", v.tab, v.market_pos, v.market_scope, v.market_age, v.market_price, v.market_text.to_lowercase(), c.abs(), c.world.players.len(), v.market_stat, v.market_genau, v.market_gut);
    if key != v.market_key {
        v.market_key = key;
        let q = v.market_text.to_lowercase();
        let ol = c.league();
        let mut list: Vec<usize> = (0..c.world.players.len()).filter(|&i| {
            let p = &c.world.players[i];
            if p.team == c.own as i32 || p.team == -2 || p.youth > 0 { return false; }
            if v.tab == 1 && !p.listed { return false; }
            if v.tab == 2 && (p.leihe.is_some() || c.status_von(i).map_or(true, |s| s.status != crate::kimarkt::Status::Leihliste)) { return false; }
            if v.tab == 3 && p.team != -1 { return false; }
            if v.tab != 3 && p.team < 0 { return false; }
            let part = part_of(&p.pos);
            if v.market_pos > 0 && part + 1 != v.market_pos { return false; }
            if v.market_scope == 1 && (p.team < 0 || c.world.clubs[p.team as usize].league != ol) { return false; }
            if p.age as i32 > v.market_age || (v.market_price > 0 && p.value > v.market_price as f64 * 1e6) { return false; }
            if v.market_genau > 0 { let k = v.market_genau - 1; if pos_stufe(p, k) < 1 + v.market_gut as u8 { return false; } }
            if !q.is_empty() && !p.name.to_lowercase().contains(&q) && !(p.team >= 0 && c.world.clubs[p.team as usize].name.to_lowercase().contains(&q)) { return false; }
            true
        }).collect();
        if v.market_stat > 0 {
            let f = MARKT_ZAHLEN[v.market_stat - 1].1;
            let mut mit: Vec<(usize, f32)> = list.into_iter().filter(|&p| markt_erweitert(c, p)).filter_map(|p| { let (st, _) = crate::statistik::saison_von(&c.world, p); (crate::statistik::get(&st, crate::statistik::MIN) >= 900).then_some(())?; f(&st).map(|x| (p, x)) }).collect();
            mit.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            list = mit.into_iter().map(|x| x.0).collect();
        } else {
            list.sort_by(|a, b| { let (x, y) = (c.ovr_range(*a), c.ovr_range(*b)); (y.0 + y.1).partial_cmp(&(x.0 + x.1)).unwrap() });
        }
        list.truncate(2000);
        v.market_cache = list;
    }
    let mut list = v.market_cache.clone();
    sortieren(v, 20, &mut list, |p, col| {
        let pl = &c.world.players[p];
        match col {
            0 => Sk::S(pl.name.to_lowercase()),
            1 => Sk::S(if pl.team >= 0 { c.world.clubs[pl.team as usize].name.to_lowercase() } else { String::new() }),
            2 => Sk::N(pl.age as f32), 3 => Sk::N(-(crate::pesdb::pos_index(&pl.pos) as f32)),
            4 => { let (a, b) = c.ovr_range(p); Sk::N(a + b) }
            5 => Sk::N(pl.value as f32), _ => Sk::N(c.moneyball(p) as f32),
        }
    });
    let y = y + 46.0;
    let lw = cbreite() - 300.0;
    let area = Rect::new(x0() + 4.0, y, lw, hoehe() - y - 16.0);
    ui.card(area);
    sort_head(ui, v, 20, area.x, area.y + 22.0, &[(crate::inhalt::t("ui.wort.gross_name"), 16.0), (crate::inhalt::t("ui.wort.gross_verein"), 250.0), (crate::inhalt::t("ui.wort.gross_alter"), 470.0), (crate::inhalt::t("ui.wort.gross_pos"), 530.0), (crate::inhalt::t("ui.pages.staerke"), 590.0), (crate::inhalt::t("ui.wort.gross_marktwert"), 680.0), (crate::inhalt::t("ui.wort.gross_preis_leist"), 800.0)]);
    let vis = ((area.h - 40.0) / 36.0) as usize;
    let start = scroll_rows(ui, v, area, list.len(), vis);
    ui.text_right(&crate::tf!("zentrale.k.spieler.viele", list.len()), x0() + cbreite() - 14.0, Y0 - 20.0, 16.0, Theme::text2());
    for (k, &p) in list.iter().skip(start).take(vis).enumerate() {
        let pl = &c.world.players[p];
        let r = Rect::new(area.x + 4.0, area.y + 34.0 + k as f32 * 36.0, area.w - 8.0, 34.0);
        let sel = v.sel_player == Some(p);
        if sel { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::akzent(), 0.14)); rrect_line(r.x, r.y, r.w, r.h, 4.0, Theme::akzent()); } else if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::erhoehung()); }
        draw_rectangle(r.x, r.y + 4.0, 3.0, r.h - 8.0, Theme::teil(&pl.pos));
        let cy = r.y + 23.0;
        ui.player_photo(r.x + 22.0, r.y + r.h / 2.0, 12.0, pl.id, &pl.name);
        let nm = ui.fit(&pl.name, 17.0, 196.0);
        ui.text_semi(&nm, r.x + 40.0, cy, 17.0, Theme::text());
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
        let cl = ui.fit(&club, 16.0, 210.0);
        ui.text(&cl, r.x + 248.0, cy, 16.0, Theme::text2());
        ui.text(&format!("{}", pl.age), r.x + 476.0, cy, 16.0, Theme::text());
        ui.pos_tag(r.x + 526.0, cy - 5.0, pos_de(&pl.pos));
        strength_cell(ui, c, p, r.x + 612.0, cy - 6.0);
        ui.text(&money(pl.value), r.x + 680.0, cy, 16.0, Theme::text());
        if v.market_stat > 0 {
            let (st, hoch) = crate::statistik::saison_von(&c.world, p);
            if let Some(x) = MARKT_ZAHLEN[v.market_stat - 1].1(&st) { ui.text(&format!("{x:.2}{}", if hoch { " ~" } else { "" }), r.x + 820.0, cy, 16.0, Theme::akzent()); }
        } else {
            let mb = c.moneyball(p);
            ui.text(&format!("{:+.0}", mb), r.x + 820.0, cy, 16.0, if mb >= 3.0 { Theme::gut() } else if mb <= -3.0 { Theme::problem() } else { Theme::text2() });
        }
        // labels (Konzept 5.1): over budget, would not come, on the shortlist, report there
        let schild = if c.merkliste.contains(&p) { Some((crate::inhalt::t("ui.markt.schild.merkliste"), Theme::akzent())) }
            else if let Some(k) = c.machbar(p, false) { Some((crate::inhalt::t(k), Theme::problem())) }
            else if c.know_of(p) >= crate::regeln::f("scouting", "wissen.bericht_ab") / 100.0 { Some((crate::inhalt::t("ui.markt.schild.bericht"), Theme::gut())) } else { None };
        if let Some((txt, col)) = schild { ui.text_right(&ui.fit(txt, 13.0, r.w - 900.0), r.x + r.w - 10.0, cy, 13.0, col); }
        if ui.hit(r) { v.sel_player = Some(p); }
    }
    // detail panel
    let dr = Rect::new(x0() + lw + 16.0, y - 0.0, 288.0 - 4.0, hoehe() - y - 16.0);
    ui.card(dr);
    let Some(p) = v.sel_player.filter(|&p| p < c.world.players.len()) else { ui.paragraph(crate::inhalt::t("ui.pages.waehle_einen_spieler_aus_der"), dr.x + 14.0, dr.y + 30.0, 16.0, dr.w - 28.0, Theme::text2()); auswahl_zeichnen(ui, v, &felder); return; };
    let pl = c.world.players[p].clone();
    ui.player_photo(dr.x + dr.w - 40.0, dr.y + 34.0, 24.0, pl.id, &pl.name);
    let nm = ui.fit(&pl.name, 24.0, dr.w - 84.0);
    ui.title(&nm, dr.x + 14.0, dr.y + 34.0, 24.0, Theme::text());
    ui.pos_tag(dr.x + 14.0, dr.y + 54.0, pos_de(&pl.pos));
    ui.text(&crate::tf!("zentrale.k.jahre", pl.age), dr.x + 56.0, dr.y + 59.0, 16.0, Theme::text2());
    let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
    let cl = ui.fit(&club, 16.0, dr.w - 28.0);
    ui.text(&cl, dr.x + 14.0, dr.y + 82.0, 16.0, Theme::text2());
    let (lo, hi) = c.ovr_range(p);
    let (pa, pb) = c.pot_range(p);
    ui.text_semi(crate::inhalt::t("ui.pages.staerke"), dr.x + 14.0, dr.y + 112.0, 13.0, Theme::gedaempft());
    ui.title(&if hi - lo <= 1.0 { format!("{:.0}", (lo + hi) / 2.0) } else { format!("{lo}–{hi}") }, dr.x + 14.0, dr.y + 140.0, 26.0, Theme::skala((lo + hi) / 2.0));
    ui.text_semi(crate::inhalt::t("ui.wort.gross_potenzial"), dr.x + 150.0, dr.y + 112.0, 13.0, Theme::gedaempft());
    ui.title(&format!("{pa}–{pb}"), dr.x + 150.0, dr.y + 140.0, 26.0, Theme::gold());
    ui.text(&crate::tf!("scouting.wissen_prozent", format!("{:.0}", c.know_of(p) * 100.0)), dr.x + 14.0, dr.y + 162.0, 14.0, Theme::text2());
    ui.bar(dr.x + 110.0, dr.y + 155.0, dr.w - 124.0, 6.0, c.know_of(p), Theme::akzent());
    ui.text(&crate::tf!("ui.pages.wert_gehalt", money(pl.value), money(pl.wage)), dr.x + 14.0, dr.y + 186.0, 15.0, Theme::text());
    ui.text(&crate::tf!("ui.pages.vertrag_bis_preis", date::START_YEAR as u32 + pl.contract_end + 1, if pl.team >= 0 { money(c.ask_price(p)) } else { crate::inhalt::t("iq.wechsel.frei").into() }), dr.x + 14.0, dr.y + 206.0, 15.0, Theme::text());
    // Dafür / Dagegen
    let best_own = c.squad().iter().filter(|&&q| part_of(&c.world.players[q].pos) == part_of(&pl.pos)).map(|&q| c.world.players[q].ovr).fold(0.0f32, f32::max);
    let mid = (lo + hi) / 2.0;
    // why his club lets him go (kimarkt.rs: transfer or loan list and its reason)
    let liste = c.status_von(p).filter(|s| matches!(s.status, crate::kimarkt::Status::Transferliste | crate::kimarkt::Status::Leihliste)).map(|s| {
        let cn = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { String::new() };
        let g = if s.grund.is_empty() { String::new() } else { crate::tf!(&format!("ui.km.grund.{}", s.grund), pl.name.clone(), cn) }; // roh
        format!("{} {g} ", crate::inhalt::t(if s.status == crate::kimarkt::Status::Leihliste { "tf.liste.leihe" } else { "tf.liste.verkauf" }))
    }).unwrap_or_default();
    // the scout's verdict (scoutbericht.rs) once he knows enough
    let urteil = if c.know_of(p) >= crate::regeln::f("scouting", "wissen.bericht_ab") / 100.0 { format!("{} ", crate::inhalt::t(&format!("ui.sb.urteil.{}", c.scout_urteil(p).urteil))) } else { String::new() };
    let pro = urteil + &liste + &if mid > best_own { crate::tf!("ui.pages.verstaerkt_die_position_wertung_gegen", format!("{:.0}", mid), format!("{:.0}", best_own)) } else if pl.age <= 21 && pb > best_own { crate::inhalt::t("ui.pages.junges_talent_mit_luft_nach").to_string() } else { crate::tf!("ui.pages.passt_ins", ideas_t()[c.tactic.idea()], fit_text(c.system_fit(p))) };
    let mut con = vec![];
    if c.ask_price(p) > c.transfer_budget { con.push(crate::inhalt::t("ui.pages.sprengt_das_transferbudget").to_string()); }
    if c.player_rep(p) - c.club().rep > 25.0 { con.push(crate::inhalt::t("ui.pages.haelt_deinen_verein_fuer_zu").into()); }
    if pl.age >= 31 { con.push(crate::inhalt::t("ui.pages.schon_ueber_30_baut_bald").into()); }
    if c.know_of(p) < 0.5 { con.push(crate::inhalt::t("ui.pages.noch_wenig_bekannt_scout_schicken").into()); }
    let con = if con.is_empty() { crate::inhalt::t("tf.verf.keine_bedenken").to_string() } else { con.join(" ") };
    // the buttons hang from the card's bottom; a short card (lower windows) shows „Dafür“ and „Dagegen“ in two lines each
    let unten = (dr.h - 92.0).min(382.0);
    if unten >= 380.0 {
        let r1 = Rect::new(dr.x + 10.0, dr.y + 220.0, dr.w - 20.0, 74.0);
        rrect(r1.x, r1.y, r1.w, r1.h, 6.0, alpha(Theme::gut(), 0.1)); rrect_line(r1.x, r1.y, r1.w, r1.h, 6.0, alpha(Theme::gut(), 0.5));
        ui.text_semi(crate::inhalt::t("ui.pages.dafuer"), r1.x + 10.0, r1.y + 18.0, 13.0, Theme::gut());
        ui.paragraph(&pro, r1.x + 10.0, r1.y + 38.0, 15.0, r1.w - 20.0, Theme::text());
        let r2 = Rect::new(dr.x + 10.0, dr.y + 300.0, dr.w - 20.0, 74.0);
        rrect(r2.x, r2.y, r2.w, r2.h, 6.0, alpha(Theme::problem(), 0.1)); rrect_line(r2.x, r2.y, r2.w, r2.h, 6.0, alpha(Theme::problem(), 0.5));
        ui.text_semi(crate::inhalt::t("ui.wort.gross_dagegen"), r2.x + 10.0, r2.y + 18.0, 13.0, Theme::problem());
        ui.paragraph(&con, r2.x + 10.0, r2.y + 38.0, 15.0, r2.w - 20.0, Theme::text());
    } else {
        // one line each, the whole text when the mouse is over it (text_passend)
        ui.text_passend(&crate::tf!("ui.pages.dafuer_zeile", &pro), dr.x + 14.0, dr.y + (unten - 28.0).max(226.0), 13.0, dr.w - 28.0, 1, Theme::gut());
        ui.text_passend(&crate::tf!("ui.pages.dagegen_zeile", &con), dr.x + 14.0, dr.y + (unten - 10.0).max(244.0), 13.0, dr.w - 28.0, 1, Theme::problem());
    }
    // from 1 January a player with an expiring contract can be signed for the summer, without a fee (termine.rs)
    let vv = c.vorvertrag_moeglich(p);
    // a player on his club's loan list: ask for a loan right here (12 months, no buy clause; the player card offers
    // every term), the transfer talks stay one click away on his card (Fabian 24.11.: the loan list had no loan button)
    let leihbar = c.status_von(p).is_some_and(|s| s.status == crate::kimarkt::Status::Leihliste) && c.world.players[p].leihe.is_none();
    if leihbar {
        if ui.button_gold(Rect::new(dr.x + 10.0, dr.y + unten, dr.w - 20.0, 42.0), crate::inhalt::t("ui.leihe.anfragen")) {
            match c.leihanfrage(p, 12, 0) { Ok(t) => { v.toast(&t, true); ui.play(crate::id("angebot")); } Err(e) => { v.toast(&e, false); ui.play(crate::id("fehler")); } }
        }
    } else if ui.button_gold(Rect::new(dr.x + 10.0, dr.y + unten, dr.w - 20.0, 42.0), if vv { crate::inhalt::t("vv.knopf") } else { crate::inhalt::t("ui.wort.gross_verhandeln") }) {
        match if vv { c.vorvertrag_oeffnen(p) } else { c.open_neg(p) } { Ok(i) => { v.modal = Modal::Neg(i); ui.play(crate::id("angebot")); } Err(e) => { v.toast(&e, false); ui.play(crate::id("fehler")); } }
    }
    // a watch assignment with a free scout (auftraege.rs)
    let watched = c.auftraege.iter().any(|x| !x.fertig && x.spieler == Some(p));
    if ui.button(Rect::new(dr.x + 10.0, dr.y + unten + 48.0, (dr.w - 28.0) / 2.0, 32.0), if watched { crate::inhalt::t("ui.scout.laeuft") } else { crate::inhalt::t("ui.scout.schicken") }, false) && !watched {
        let belegt: Vec<String> = c.auftraege.iter().filter(|x| !x.fertig).map(|x| x.scout.clone()).collect();
        match c.scouts.iter().find(|s| !belegt.contains(&s.name)).map(|s| s.name.clone()) {
            Some(scout) => { let x = crate::auftraege::Auftrag { art: crate::auftraege::BEOBACHTEN, scout, spieler: Some(p), ..Default::default() }; match c.auftrag_anlegen(x, 21) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
            None => v.toast(crate::inhalt::t("ui.pages.alle_scouts_haben_schon_einen"), false),
        }
    }
    if ui.button(Rect::new(dr.x + 18.0 + (dr.w - 28.0) / 2.0, dr.y + unten + 48.0, (dr.w - 28.0) / 2.0, 32.0), crate::inhalt::t("aktion.profil"), false) { v.modal = Modal::Player(p); }
    auswahl_zeichnen(ui, v, &felder);
}

/// Transfers › Merkliste: the players put on the watch list with "Beobachten" (profile).
pub(super) fn merkliste(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(x0() + 4.0, Y0 - 40.0, cbreite(), hoehe() - Y0 + 24.0);
    ui.card_titled(area, &crate::tf!("ui.pages.merkliste_spieler", c.merkliste.len()), Theme::akzent());
    if c.merkliste.is_empty() { ui.paragraph(crate::inhalt::t("ui.pages.noch_niemand_auf_der_merkliste"), area.x + 14.0, area.y + 60.0, 16.0, area.w - 28.0, Theme::text2()); return; }
    let mut weg = None;
    for (k, &p) in c.merkliste.clone().iter().enumerate() {
        let y = area.y + 50.0 + k as f32 * 44.0;
        if y > area.y + area.h - 44.0 { break; }
        let r = Rect::new(area.x + 8.0, y, area.w - 16.0, 40.0);
        if k % 2 == 1 { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::erhoehung(), 0.6)); }
        let pl = &c.world.players[p];
        ui.pos_tag(r.x + 8.0, r.y + 20.0, pos_de(&pl.pos));
        ui.player_photo(r.x + 62.0, r.y + 20.0, 15.0, pl.id, &pl.name);
        ui.text_semi(&ui.fit(&pl.name, 17.0, 260.0), r.x + 84.0, r.y + 26.0, 17.0, Theme::text());
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
        ui.text(&crate::tf!("as.ent.talent", pl.age, ui.fit(&club, 15.0, 240.0)), r.x + 360.0, r.y + 26.0, 15.0, Theme::text2());
        let (lo, hi) = c.ovr_range(p);
        ui.text_semi(&if hi - lo <= 1.0 { format!("{lo:.0}") } else { format!("{lo:.0}–{hi:.0}") }, r.x + 720.0, r.y + 26.0, 16.0, Theme::skala((lo + hi) / 2.0));
        ui.text(&money(pl.value), r.x + 800.0, r.y + 26.0, 15.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 220.0, r.y + 6.0, 100.0, 28.0), crate::inhalt::t("aktion.profil"), false) { v.sel_player = Some(p); v.go(Page::Spielerprofil); return; }
        if ui.button(Rect::new(r.x + r.w - 110.0, r.y + 6.0, 100.0, 28.0), crate::inhalt::t("karrieren.entfernen.ja"), false) { weg = Some(p); }
    }
    if let Some(p) = weg { c.merkliste.retain(|&x| x != p); }
}
