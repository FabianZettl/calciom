use super::*;

/// The card „So hat es gewirkt“ after a talk: arrows for morale, trust, relationship, team spirit and the teammates it reached.
pub fn wirkung_karte(ui: &mut Ui, c: &mut Career) {
    let Some(w) = c.wirkung.clone() else { return };
    if crate::ui::jetzt() - w.seit > crate::regeln::f("entwicklung_pesm", "wellen.anzeige_sekunden") as f64 { c.wirkung = None; return; }
    let n = w.andere.len().min(6);
    let r = Rect::new(breite() - 440.0, hoehe() - 200.0 - n as f32 * 30.0, 420.0, 160.0 + n as f32 * 30.0);
    rrect(r.x, r.y, r.w, r.h, 12.0, Theme::flaeche());
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, Theme::akzent());
    let pl = &c.world.players[w.pid];
    ui.player_photo(r.x + 34.0, r.y + 36.0, 20.0, pl.id, &pl.name);
    ui.title(crate::inhalt::t("ui.wellen.titel"), r.x + 64.0, r.y + 32.0, 20.0, Theme::text());
    ui.text(&ui.fit(&w.grund, 13.0, r.w - 80.0), r.x + 64.0, r.y + 52.0, 13.0, Theme::text2());
    let zahl = |d: f32| { let s = format!("{d:+.1}").replace('-', "−"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { s.replace('.', ",") } else { s } };
    let pfeil = |ui: &Ui, x: f32, y: f32, name: &str, d: f32| {
        let col = if d > 0.05 { Theme::gut() } else if d < -0.05 { Theme::problem() } else { Theme::gedaempft() };
        ui.text(name, x, y, 14.0, Theme::text2());
        let (ax, ay) = (x + 102.0, y - 5.0);
        if d > 0.05 { draw_triangle(vec2(ax - 6.0, ay + 4.0), vec2(ax + 6.0, ay + 4.0), vec2(ax, ay - 6.0), col); }
        else if d < -0.05 { draw_triangle(vec2(ax - 6.0, ay - 4.0), vec2(ax + 6.0, ay - 4.0), vec2(ax, ay + 6.0), col); }
        else { draw_rectangle(ax - 6.0, ay - 1.0, 12.0, 2.0, col); }
        ui.text(&zahl(d), x + 114.0, y, 14.0, col);
    };
    pfeil(ui, r.x + 18.0, r.y + 84.0, crate::inhalt::t("ui.wellen.moral"), w.moral);
    pfeil(ui, r.x + 218.0, r.y + 84.0, crate::inhalt::t("ui.wellen.vertrauen"), w.vertrauen);
    pfeil(ui, r.x + 18.0, r.y + 108.0, crate::inhalt::t("ui.wellen.beziehung"), w.beziehung);
    pfeil(ui, r.x + 218.0, r.y + 108.0, crate::inhalt::t("ui.wellen.teamgeist"), w.teamgeist);
    if n == 0 { ui.text(crate::inhalt::t("ui.wellen.niemand"), r.x + 18.0, r.y + 140.0, 13.0, Theme::gedaempft()); }
    for (k, (q, d, g)) in w.andere.iter().take(6).enumerate() {
        let y = r.y + 136.0 + k as f32 * 30.0;
        let ql = &c.world.players[*q];
        ui.player_photo(r.x + 30.0, y + 6.0, 11.0, ql.id, &ql.name);
        ui.text(&ui.fit(&format!("{}: {}", ql.name, g), 13.0, r.w - 120.0), r.x + 48.0, y + 11.0, 13.0, Theme::text());
        ui.text_right(&zahl(*d), r.x + r.w - 14.0, y + 11.0, 13.0, if *d >= 0.0 { Theme::gut() } else { Theme::problem() });
    }
    if ui.hit(r) { c.wirkung = None; }
}

// ================================================================ Suche als Overlay und Lesezeichen (Punkt 6)
/// Every page reachable from the menu, with its name (for the search and the bookmarks).
pub fn alle_seiten(c: &Career) -> Vec<(Page, String)> {
    let mut v: Vec<(Page, String)> = vec![];
    for (_, haupt) in MENU.iter() {
        for h in haupt.iter() { for (p, n) in reiter(*h, c) { if !v.iter().any(|x| x.0 == p) { v.push((p, n.to_string())); } } }
    }
    v
}
/// The search over the page: players, clubs and pages whose name holds the text; Enter opens the first, Esc closes.
/// Empty, it shows the bookmarks.
pub fn suche_overlay(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let r = Rect::new(breite() / 2.0 - 300.0, 70.0, 600.0, 520.0);
    ui.card(r);
    ui.tippen(&mut v.suche_text, 60);
    let feld = Rect::new(r.x + 16.0, r.y + 16.0, r.w - 32.0, 44.0);
    rrect(feld.x, feld.y, feld.w, feld.h, 22.0, Theme::erhoehung());
    let cursor = if (crate::ui::jetzt() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
    let leer = v.suche_text.is_empty();
    ui.text(&if leer { crate::inhalt::t("ui.suche.hinweis").to_string() } else { format!("{}{cursor}", v.suche_text) }, feld.x + 20.0, feld.y + 29.0, 18.0, if leer { Theme::gedaempft() } else { Theme::text() });
    let seiten = alle_seiten(c);
    if v.suche_fuer != v.suche_text {
        v.suche_fuer = v.suche_text.clone();
        let q = v.suche_text.to_lowercase();
        let mut t: Vec<(u8, usize)> = vec![];
        if q.chars().count() >= 2 {
            t.extend(seiten.iter().enumerate().filter(|(_, x)| x.1.to_lowercase().contains(&q)).map(|(i, _)| (2u8, i)).take(4));
            t.extend(c.world.clubs.iter().enumerate().filter(|(_, x)| x.name.to_lowercase().contains(&q)).map(|(i, _)| (1u8, i)).take(5));
            let mut sp: Vec<usize> = c.world.players.iter().enumerate().filter(|(_, x)| x.team >= -1 && x.name.to_lowercase().contains(&q)).map(|(i, _)| i).collect();
            sp.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
            t.extend(sp.into_iter().take(8).map(|i| (0u8, i)));
        }
        v.suche_treffer = t;
    }
    let mut oeffnen: Option<(u8, usize)> = None;
    let mut y = r.y + 76.0;
    if leer {
        ui.text_semi(crate::inhalt::t("ui.suche.lesezeichen"), r.x + 20.0, y + 6.0, 14.0, Theme::gedaempft());
        y += 16.0;
        let marken: Vec<usize> = c.lesezeichen.iter().filter_map(|m| seiten.iter().position(|s| format!("{:?}", s.0) == *m)).collect();
        if marken.is_empty() { ui.paragraph(crate::inhalt::t("ui.suche.keine_lesezeichen"), r.x + 20.0, y + 20.0, 15.0, r.w - 40.0, Theme::text2()); }
        for i in marken { let z = Rect::new(r.x + 12.0, y, r.w - 24.0, 34.0); if zeile_suche(ui, z, &seiten[i].1, crate::inhalt::t("ui.suche.seite")) { oeffnen = Some((2, i)); } y += 38.0; }
    } else if v.suche_treffer.is_empty() {
        ui.text(crate::inhalt::t("ui.suche.nichts"), r.x + 20.0, y + 20.0, 15.0, Theme::text2());
    }
    for &(art, i) in v.suche_treffer.clone().iter() {
        if y + 38.0 > r.y + r.h - 8.0 { break; }
        let z = Rect::new(r.x + 12.0, y, r.w - 24.0, 34.0);
        let (name, wo) = match art {
            0 => { let pl = &c.world.players[i]; (pl.name.clone(), if pl.team >= 0 { format!("{} · {}", crate::world::pos_de(&pl.pos), c.world.clubs[pl.team as usize].name) } else { crate::inhalt::t("sc.profil.vereinslos").to_string() }) }
            1 => (c.world.clubs[i].name.clone(), crate::inhalt::t("ui.suche.verein").to_string()),
            _ => (seiten[i].1.clone(), crate::inhalt::t("ui.suche.seite").to_string()),
        };
        if zeile_suche(ui, z, &name, &wo) { oeffnen = Some((art, i)); }
        y += 38.0;
    }
    if ui.key_enter { oeffnen = oeffnen.or(v.suche_treffer.first().copied()); }
    if let Some((art, i)) = oeffnen {
        v.modal = Modal::None;
        v.suche_text.clear();
        match art { 0 => v.modal = Modal::Player(i), 1 => { v.sel_club = Some(i); v.go(Page::Vereine); } _ => v.go(seiten[i].0) }
        return;
    }
    if is_key_pressed(KeyCode::Escape) || (ui.clicked && !r.contains(ui.mouse)) { v.modal = Modal::None; v.suche_text.clear(); }
}
fn zeile_suche(ui: &mut Ui, z: Rect, name: &str, wo: &str) -> bool {
    rrect(z.x, z.y, z.w, z.h, 8.0, if ui.hover(z) { Theme::ueberlagerung() } else { Theme::flaeche() });
    ui.text_semi(&ui.fit(name, 16.0, z.w * 0.55), z.x + 12.0, z.y + 23.0, 16.0, Theme::text());
    ui.text_right(&ui.fit(wo, 13.0, z.w * 0.4), z.x + z.w - 12.0, z.y + 22.0, 13.0, Theme::text2());
    ui.hit(z)
}

/// The right-click menu of a club in a table, a cup or a list (Fabian 24.11.): profile, squad, fixtures, and the
/// opponent report when it is our next opponent.
pub fn verein_kontext_menue(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let Some((k, pos)) = v.verein_kontext else { return };
    let t = crate::inhalt::t;
    let naechster = c.naechster_gegner() == Some(k);
    let mut punkte: Vec<(u8, &str)> = vec![(0, t("ui.vk.profil")), (1, t("ui.vk.kader")), (2, t("ui.vk.spielplan"))];
    if naechster { punkte.push((3, t("ui.vk.gegnerbericht"))); }
    let w = 240.0;
    let r = Rect::new(pos.x.min(breite() - w - 8.0), pos.y.min(hoehe() - 40.0 * punkte.len() as f32 - 50.0), w, 44.0 + 36.0 * punkte.len() as f32);
    ui.card(r);
    let (farbe, _) = club_color(c, k);
    ui.club_crest(r.x + 22.0, r.y + 22.0, 11.0, c.world.clubs[k].id, farbe, &c.world.clubs[k].short);
    ui.text_semi(&ui.fit(&c.world.clubs[k].name, 15.0, w - 56.0), r.x + 40.0, r.y + 27.0, 15.0, Theme::text());
    let mut wahl = None;
    for (i, (id, txt)) in punkte.iter().enumerate() {
        let z = Rect::new(r.x + 6.0, r.y + 40.0 + i as f32 * 36.0, w - 12.0, 32.0);
        rrect(z.x, z.y, z.w, z.h, 6.0, if ui.hover(z) { Theme::ueberlagerung() } else { Theme::flaeche() });
        ui.text(txt, z.x + 10.0, z.y + 21.0, 15.0, Theme::text());
        if ui.hit(z) { wahl = Some(*id); }
    }
    match wahl {
        Some(0) | Some(1) => { v.verein_kontext = None; v.sel_club = Some(k); v.go(Page::Vereine); }
        Some(2) => { v.verein_kontext = None; v.modal = Modal::Spielplan(k); }
        Some(_) => { v.verein_kontext = None; v.go(Page::Aufstellung); v.tab = 2; }
        None => if (ui.clicked && !r.contains(ui.mouse)) || is_key_pressed(KeyCode::Escape) { v.verein_kontext = None; },
    }
}
/// A club's fixtures of the season, league and cups, with results so far.
pub fn spielplan_modal(ui: &mut Ui, v: &mut View, c: &Career, k: usize) {
    let t = crate::inhalt::t;
    let r = Rect::new(x0() + cbreite() / 2.0 - 360.0, Y0 - 40.0, 720.0, hoehe() - Y0 + 20.0);
    let (farbe, _) = club_color(c, k);
    ui.card_titled(r, &crate::tf!("ui.vk.spielplan_titel", c.world.clubs[k].name.clone()), farbe);
    let mine = |f: &crate::world::Fixture| f.h == k || f.a == k;
    let l = c.world.clubs[k].league;
    let mut fx: Vec<(String, crate::world::Fixture)> = c.world.leagues[l].fixtures.iter().filter(|f| mine(f)).map(|f| (c.world.leagues[l].name.clone(), *f)).collect();
    for cu in &c.world.cups { fx.extend(cu.fx.iter().filter(|x| mine(&x.f)).map(|x| (cu.name.clone(), x.f))); }
    fx.sort_by_key(|x| x.1.day);
    let zh = 26.0;
    let sichtbar = ((r.h - 110.0) / zh) as usize;
    let start = scroll_rows(ui, v, Rect::new(r.x, r.y + 50.0, r.w, r.h - 110.0), fx.len(), sichtbar);
    for (i, (wett, f)) in fx.iter().skip(start).take(sichtbar).enumerate() {
        let y = r.y + 74.0 + i as f32 * zh;
        let heim = f.h == k;
        let geg = if heim { f.a } else { f.h };
        ui.text(&crate::date::short(c.world.season, f.day), r.x + 16.0, y, 13.0, Theme::text2());
        ui.text(&ui.fit(wett, 12.0, 150.0), r.x + 100.0, y, 12.0, Theme::gedaempft());
        ui.club_crest(r.x + 272.0, y - 4.0, 8.0, c.world.clubs[geg].id, club_color(c, geg).0, &c.world.clubs[geg].short);
        ui.text(&ui.fit(&format!("{} {}", if heim { t("ui.duell.gegen") } else { "@" }, c.world.clubs[geg].name), 14.0, 300.0), r.x + 288.0, y, 14.0, Theme::text()); // roh
        if f.hg >= 0 {
            let (g, gg) = if heim { (f.hg, f.ag) } else { (f.ag, f.hg) };
            let col = if g > gg { Theme::gut() } else if g == gg { Theme::achtung() } else { Theme::problem() };
            rrect(r.x + r.w - 70.0, y - 14.0, 48.0, 20.0, 5.0, col);
            ui.text_center(&format!("{g}:{gg}"), r.x + r.w - 46.0, y + 1.0, 13.0, hex(0xffffff));
        }
    }
    if ui.button(Rect::new(r.x + r.w - 160.0, r.y + r.h - 52.0, 140.0, 40.0), t("aktion.schliessen"), false) || is_key_pressed(KeyCode::Escape) { v.modal = Modal::None; }
}

/// The right-click menu of a player (Punkt 6): profile, watch list, negotiate (or renew for an own player).
pub fn kontext_menue(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let Some((p, pos)) = v.kontext else { return };
    let eigen = c.world.players[p].team == c.own as i32;
    let gemerkt = c.merkliste.contains(&p);
    let t = crate::inhalt::t;
    let punkte: Vec<(u8, &str)> = vec![(0, t("ui.kontext.profil")), (1, if gemerkt { t("ui.kontext.merkliste_weg") } else { t("ui.kontext.merkliste") }), (2, if eigen { t("ui.kontext.verlaengern") } else { t("ui.kontext.verhandeln") })];
    let w = 230.0;
    let r = Rect::new(pos.x.min(breite() - w - 8.0), pos.y.min(hoehe() - 40.0 * punkte.len() as f32 - 50.0), w, 40.0 + 36.0 * punkte.len() as f32);
    ui.card(r);
    let pl = &c.world.players[p];
    ui.text_semi(&ui.fit(&pl.name, 15.0, w - 24.0), r.x + 12.0, r.y + 24.0, 15.0, Theme::text());
    let mut wahl = None;
    for (k, (id, txt)) in punkte.iter().enumerate() {
        let z = Rect::new(r.x + 6.0, r.y + 36.0 + k as f32 * 36.0, w - 12.0, 32.0);
        rrect(z.x, z.y, z.w, z.h, 6.0, if ui.hover(z) { Theme::ueberlagerung() } else { Theme::flaeche() });
        ui.text(txt, z.x + 10.0, z.y + 21.0, 15.0, Theme::text());
        if ui.hit(z) { wahl = Some(*id); }
    }
    match wahl {
        Some(0) => { v.kontext = None; v.modal = Modal::Player(p); }
        Some(1) => { v.kontext = None; if gemerkt { c.merkliste.retain(|&x| x != p); } else { c.merkliste.push(p); } v.toast(if gemerkt { t("ui.kontext.ist_weg") } else { t("ui.post.auf_merkliste") }, true); }
        Some(_) => { v.kontext = None; match c.open_neg(p) { Ok(k) => v.modal = Modal::Neg(k), Err(e) => v.toast(&e, false) } }
        None => if (ui.clicked && !r.contains(ui.mouse)) || is_key_pressed(KeyCode::Escape) { v.kontext = None; },
    }
}

/// A reward moment full screen (SPIELGEFUEHL.md measure 3): a pulsing gold seal with the player's photo or the crest,
/// the title, one paragraph and "Weiter" (Enter, space or a click). The first frame plays the title jingle.
/// The draw as a ceremony: the balls come out one after the other, the ties appear, ours in gold; a click closes.
pub fn auslosung_overlay(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let Some(&(ci, st)) = c.auslosungen.first() else { return };
    if ci >= c.world.cups.len() { c.auslosungen.remove(0); return; }
    let t = get_time() as f32;
    if v.auslosung_t0 <= 0.0 { v.auslosung_t0 = t; ui.play(crate::id("meilenstein")); }
    let dt = t - v.auslosung_t0;
    let cup = &c.world.cups[ci];
    let mut paare: Vec<(usize, usize)> = vec![];
    for f in cup.fx.iter().filter(|f| f.stage == st) { if !paare.iter().any(|p| (p.0 == f.f.h && p.1 == f.f.a) || (p.0 == f.f.a && p.1 == f.f.h)) { paare.push((f.f.h, f.f.a)); } }
    let (w, h) = (breite(), hoehe());
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.8));
    let r = Rect::new(w / 2.0 - 420.0, 60.0, 840.0, h - 120.0);
    rrect(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 20.0, Theme::gold());
    rrect(r.x, r.y, r.w, r.h, 18.0, Theme::flaeche());
    let runde = crate::cups::runden_name(st, cup.kind, paare.len());
    ui.text_center(&crate::inhalt::t("ui.losung.titel").to_uppercase(), r.x + r.w / 2.0, r.y + 40.0, 15.0, Theme::gold());
    ui.title(&ui.fit(&format!("{} · {runde}", cup.name), 30.0, r.w - 40.0), r.x + 24.0, r.y + 80.0, 30.0, Theme::text());
    // the bowl: balls moving in a circle, fewer as ties come out
    let (bx, by) = (r.x + r.w - 110.0, r.y + 150.0);
    draw_circle_lines(bx, by, 60.0, 3.0, alpha(Theme::text2(), 0.6));
    let gezogen = ((dt / 0.9) as usize).min(paare.len());
    for k in 0..(paare.len() - gezogen) * 2 { let a = t * 1.7 + k as f32 * 0.9; draw_circle(bx + a.cos() * (20.0 + (k % 3) as f32 * 12.0), by + a.sin() * (20.0 + (k % 3) as f32 * 12.0), 9.0, alpha(WHITE, 0.9)); }
    let zeilen = ((r.h - 170.0) / 34.0).max(1.0) as usize;
    let spalten = if paare.len() > zeilen { 2 } else { 1 };
    let sw = (r.w - 240.0) / spalten as f32;
    for (k, (a, b)) in paare.iter().take(gezogen).enumerate() {
        let (sp, z) = (k / zeilen, k % zeilen);
        let x = r.x + 24.0 + sp as f32 * sw;
        let y = r.y + 130.0 + z as f32 * 34.0;
        let unser = *a == c.own || *b == c.own;
        if unser { rrect(x - 6.0, y - 4.0, sw - 12.0, 30.0, 8.0, alpha(Theme::gold(), 0.25)); }
        for (j, club) in [*a, *b].iter().enumerate() {
            let cl = &c.world.clubs[*club];
            let xx = x + j as f32 * (sw / 2.0);
            ui.club_crest(xx + 10.0, y + 11.0, 9.0, cl.id, hex(cl.c1), &cl.short);
            ui.text(&ui.fit(&cl.name, 14.0, sw / 2.0 - 40.0), xx + 24.0, y + 16.0, 14.0, if *club == c.own { Theme::gold() } else { Theme::text() });
        }
        ui.text_center("–", x + sw / 2.0 - 8.0, y + 16.0, 14.0, Theme::text2()); // roh
    }
    let fertig = gezogen >= paare.len();
    if ui.button_gold(Rect::new(r.x + r.w / 2.0 - 120.0, r.y + r.h - 60.0, 240.0, 44.0), crate::inhalt::t(if fertig { "ui.losung.weiter" } else { "ui.losung.schneller" })) {
        if fertig { c.auslosungen.remove(0); v.auslosung_t0 = 0.0; } else { v.auslosung_t0 -= 1000.0; }
    }
}
pub fn feier_overlay(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let Some(f) = c.feiern.first().cloned() else { return };
    if !v.feier_ton { ui.play(if f.art == crate::id("meilenstein") || f.art == crate::id("lizenz") { crate::id("meilenstein") } else { crate::id("titel") }); v.feier_ton = true; }
    let (w, h) = (breite(), hoehe());
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.78));
    let t = get_time() as f32;
    let r = Rect::new(w / 2.0 - 380.0, h / 2.0 - 250.0, 760.0, 500.0);
    rrect(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 20.0, Theme::gold());
    rrect(r.x, r.y, r.w, r.h, 18.0, Theme::flaeche());
    let (cx, cy) = (r.x + r.w / 2.0, r.y + 130.0);
    // rays that turn slowly, a breathing halo, the seal
    for k in 0..16 {
        let a = t * 0.4 + k as f32 * std::f32::consts::TAU / 16.0;
        draw_line(cx + a.cos() * 78.0, cy + a.sin() * 78.0, cx + a.cos() * 104.0, cy + a.sin() * 104.0, 4.0, alpha(Theme::gold(), 0.55));
    }
    let puls = 1.0 + 0.05 * (t * 3.0).sin();
    draw_circle(cx, cy, 74.0 * puls, alpha(Theme::gold(), 0.25));
    draw_circle(cx, cy, 62.0, Theme::gold());
    match (f.spieler, f.club) {
        (Some(p), _) => ui.player_photo(cx, cy, 54.0, c.world.players[p].id, &c.world.players[p].name.clone()),
        (None, Some(cl)) => { let k = &c.world.clubs[cl]; ui.club_crest(cx, cy, 50.0, k.id, hex(k.c1), &k.short.clone()); }
        _ => {}
    }
    let art = match f.art.as_str() { "neuzugang" => crate::inhalt::t("ui.feier.art.neuzugang"), "titel" => crate::inhalt::t("ui.feier.art.titel"), "lizenz" => crate::inhalt::t("ui.feier.art.lizenz"), "gala" => crate::inhalt::t("ui.feier.art.gala"), "vertrag" => crate::inhalt::t("ui.feier.art.vertrag"), _ => crate::inhalt::t("ui.meilenstein.titel") };
    ui.text_center(&art.to_uppercase(), cx, r.y + 258.0, 15.0, Theme::gold());
    let size = if ui.measure_title(&f.titel, 40.0) > r.w - 60.0 { 30.0 } else { 40.0 };
    let tw = ui.measure_title(&f.titel, size);
    ui.title(&f.titel, cx - tw / 2.0, r.y + 304.0, size, Theme::text());
    let mut y = r.y + 340.0;
    for z in ui.wrap(&f.text, 17.0, r.w - 100.0).into_iter().take(4) { ui.text_center(&z, cx, y, 17.0, Theme::text2()); y += 24.0; }
    let weiter = ui.button_gold(Rect::new(cx - 120.0, r.y + r.h - 70.0, 240.0, 48.0), crate::inhalt::t("ui.feier.weiter"));
    if weiter || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
        c.feiern.remove(0);
        v.feier_ton = false;
    }
}
