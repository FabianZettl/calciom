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
/// A club's matches of the season (Fabian 26.11., roadmap, after his example): left the list by month with time,
/// venue, result and competition; right the chosen match with scorers, cards and grades, and the earlier meetings.
pub fn spielplan_modal(ui: &mut Ui, v: &mut View, c: &Career, k: usize) {
    let t = crate::inhalt::t;
    let w = (cbreite() - 40.0).min(1240.0);
    let r = Rect::new(x0() + cbreite() / 2.0 - w / 2.0, Y0 - 40.0, w, hoehe() - Y0 + 20.0);
    let (farbe, _) = club_color(c, k);
    ui.card_titled(r, &crate::tf!("ui.vk.spielplan_titel", c.world.clubs[k].name.clone()), farbe);
    let fx = spiele_des_vereins(c, k);
    // the chosen match: by click, else the last one played, else the first
    let wahl = v.spiel_wahl.and_then(|(d, h, a)| fx.iter().position(|x| x.f.day == d && x.f.h == h && x.f.a == a))
        .or_else(|| fx.iter().rposition(|x| x.f.hg >= 0)).unwrap_or(0);
    let lw = (r.w * 0.58).floor();
    let zh = 26.0;
    // rows: a month heading before the first match of each month
    let mut zeilen: Vec<Option<usize>> = vec![];
    let mut monat = 0;
    for (i, x) in fx.iter().enumerate() {
        let m = crate::date::ymd(c.world.season, x.f.day).1;
        if m != monat { zeilen.push(None); monat = m; }
        zeilen.push(Some(i));
    }
    let liste = Rect::new(r.x, r.y + 50.0, lw, r.h - 110.0);
    let sichtbar = (liste.h / zh) as usize;
    let start = scroll_rows(ui, v, liste, zeilen.len(), sichtbar);
    for (n, z) in zeilen.iter().skip(start).take(sichtbar).enumerate() {
        let y = r.y + 74.0 + n as f32 * zh;
        let Some(i) = *z else {
            let i = zeilen.iter().skip(start + n).find_map(|z| *z).unwrap_or(0);
            let (yr, m, _) = crate::date::ymd(c.world.season, fx[i].f.day);
            ui.text_semi(&format!("{} {yr}", crate::date::month_name(m)), r.x + 16.0, y, 14.0, Theme::text()); // roh
            continue;
        };
        let x = &fx[i];
        let zr = Rect::new(r.x + 8.0, y - 17.0, lw - 16.0, zh - 2.0);
        if i == wahl { rrect(zr.x, zr.y, zr.w, zr.h, 6.0, Theme::ueberlagerung()); }
        if ui.hit(zr) { v.spiel_wahl = Some((x.f.day, x.f.h, x.f.a)); }
        let heim = x.f.h == k;
        let geg = if heim { x.f.a } else { x.f.h };
        ui.text(&crate::date::short(c.world.season, x.f.day), r.x + 16.0, y, 13.0, Theme::text2());
        ui.text(&crate::ligaregeln::zeit_text(x.f.zeit), r.x + 100.0, y, 13.0, Theme::text2());
        ui.club_crest(r.x + 158.0, y - 4.0, 8.0, c.world.club(geg).id, club_color(c, geg).0, &c.world.club(geg).short);
        ui.text(&ui.fit(&c.world.club(geg).name, 14.0, lw * 0.30), r.x + 172.0, y, 14.0, Theme::text());
        let ort = if x.neutral { t("ui.vk.neutral") } else if heim { t("ui.vk.heim") } else { t("ui.vk.auswaerts") };
        ui.text_center(ort, r.x + lw * 0.30 + 192.0, y, 13.0, Theme::text2());
        if x.f.hg >= 0 {
            let (g, gg) = if heim { (x.f.hg, x.f.ag) } else { (x.f.ag, x.f.hg) };
            let gewonnen = x.pens.map(|(ph, pa)| if heim { ph > pa } else { pa > ph });
            let col = match gewonnen { Some(true) => Theme::gut(), Some(false) => Theme::problem(), None if g > gg => Theme::gut(), None if g == gg => Theme::achtung(), None => Theme::problem() };
            draw_circle(r.x + lw * 0.30 + 222.0, y - 4.0, 5.0, col);
            ui.text(&format!("{}{g}–{gg}", if x.pens.is_some() { t("ui.vk.elfmeter_kurz") } else { "" }), r.x + lw * 0.30 + 232.0, y, 13.0, Theme::text()); // roh
        }
        ui.text(&ui.fit(&x.wettbewerb, 12.0, lw * 0.70 - 290.0), r.x + lw * 0.30 + 282.0, y, 12.0, Theme::gedaempft());
    }
    if let Some(x) = fx.get(wahl) { spiel_detail(ui, c, x, Rect::new(r.x + lw + 8.0, r.y + 50.0, r.w - lw - 24.0, r.h - 110.0)); }
    if ui.button(Rect::new(r.x + r.w - 160.0, r.y + r.h - 52.0, 140.0, 40.0), t("aktion.schliessen"), false) || is_key_pressed(KeyCode::Escape) { v.modal = Modal::None; v.spiel_wahl = None; }
}
/// One match of a club's list: the fixture, its competition with round and leg, neutral ground and penalties.
pub struct VereinsSpiel { pub f: crate::world::Fixture, pub wettbewerb: String, pub neutral: bool, pub pens: Option<(u8, u8)> }
pub fn spiele_des_vereins(c: &Career, k: usize) -> Vec<VereinsSpiel> {
    let mine = |f: &crate::world::Fixture| f.h == k || f.a == k;
    let l = c.world.clubs[k].league;
    let mut fx: Vec<VereinsSpiel> = c.world.leagues[l].fixtures.iter().filter(|f| mine(f)).map(|f| VereinsSpiel { f: *f, wettbewerb: c.world.leagues[l].name.clone(), neutral: false, pens: None }).collect();
    for cu in &c.world.cups {
        for x in cu.fx.iter().filter(|x| mine(&x.f)) {
            let paare = cu.fx.iter().filter(|y| y.stage == x.stage && y.leg <= 1).count();
            let mut w = format!("{} · {}", cu.name, crate::cups::runden_name(x.stage, cu.kind, paare)); // roh
            if x.leg == 1 { w = format!("{w} · {}", crate::inhalt::t("ui.vk.hinspiel")); } else if x.leg == 2 { w = format!("{w} · {}", crate::inhalt::t("ui.vk.rueckspiel")); } // roh
            fx.push(VereinsSpiel { f: x.f, wettbewerb: w, neutral: x.neutral, pens: x.pens });
        }
    }
    fx.sort_by_key(|x| (x.f.day, x.f.zeit));
    fx
}
/// The right side of the match page: score, scorers and cards of both sides, the grades, earlier meetings.
fn spiel_detail(ui: &mut Ui, c: &Career, x: &VereinsSpiel, r: Rect) {
    let t = crate::inhalt::t;
    rrect(r.x, r.y, r.w, r.h, 10.0, Theme::ueberlagerung());
    let (h, a) = (x.f.h, x.f.a);
    ui.text_center(&format!("{} · {}", crate::date::short(c.world.season, x.f.day), x.wettbewerb), r.x + r.w / 2.0, r.y + 24.0, 13.0, Theme::text2()); // roh
    let my = r.y + 66.0;
    ui.club_crest(r.x + r.w / 2.0 - 46.0, my - 6.0, 16.0, c.world.club(h).id, club_color(c, h).0, &c.world.club(h).short);
    ui.club_crest(r.x + r.w / 2.0 + 46.0, my - 6.0, 16.0, c.world.club(a).id, club_color(c, a).0, &c.world.club(a).short);
    ui.text_semi(&ui.fit(&c.world.club(h).name, 16.0, r.w / 2.0 - 80.0), r.x + 12.0, my, 16.0, Theme::text());
    let an = ui.fit(&c.world.club(a).name, 16.0, r.w / 2.0 - 80.0);
    ui.text_semi(&an, r.x + r.w - 12.0 - ui.measure_semi(&an, 16.0), my, 16.0, Theme::text());
    let stand = if x.f.hg >= 0 { format!("{}–{}", x.f.hg, x.f.ag) } else { crate::ligaregeln::zeit_text(x.f.zeit) }; // roh
    ui.title(&stand, r.x + r.w / 2.0 - ui.measure_semi(&stand, 22.0) / 2.0, my + 2.0, 22.0, Theme::text());
    if let Some((ph, pa)) = x.pens { ui.text_center(&crate::tf!("ui.vk.elfmeter", ph, pa), r.x + r.w / 2.0, my + 24.0, 12.0, Theme::text2()); }
    // a match known only by its score gets a report made up to fit it (Fabian 26.11.: like every other match, no mark)
    let sim = || c.world.bericht_simulieren(c.world.season, x.f.day, h, a, x.f.hg.max(0) as u8, x.f.ag.max(0) as u8);
    let mut b = c.world.berichte.iter().rev().find(|b| b.season == c.world.season && b.day == x.f.day && b.h == h && b.a == a).cloned()
        .or_else(|| (x.f.hg >= 0).then(sim));
    // light matches keep scorers but no grades: those come from the same made-up report
    if let Some(b) = b.as_mut().filter(|b| b.noten.is_empty()) { b.noten = sim().noten; }
    let mut y = my + 46.0;
    match b.as_ref() {
        Some(b) => {
            // goals and cards per side (an own goal counts for the other side)
            let name = |p: usize| c.world.players.get(p).map(|pl| pl.name.clone()).unwrap_or_default();
            let zeile = |e: &crate::world::MatchEvent| -> Option<(u8, String)> {
                let seite = if e.kind == 1 { 1 - e.side } else { e.side };
                let s = match e.kind { 0 => format!("{} {}'", name(e.pid), e.min), 1 => format!("{} {}' ({})", name(e.pid), e.min, t("ui.vk.eigentor")), 2 => format!("▮ {} {}'", name(e.pid), e.min), 3 => format!("▮▮ {} {}'", name(e.pid), e.min), _ => return None }; // roh
                Some((seite, s))
            };
            let zl: Vec<(u8, String, bool)> = b.ev.iter().filter_map(|e| zeile(e).map(|(s, z)| (s, z, e.kind >= 2))).collect();
            let mut yy = [y, y];
            for (s, z, karte) in zl.iter().take(16) {
                let xx = if *s == 0 { r.x + 12.0 } else { r.x + r.w / 2.0 + 6.0 };
                ui.text(&ui.fit(z, 12.0, r.w / 2.0 - 18.0), xx, yy[*s as usize], 12.0, if *karte { Theme::achtung() } else { Theme::text() });
                yy[*s as usize] += 17.0;
            }
            y = yy[0].max(yy[1]) + 10.0;
            // the grades, best first (only detailed matches have them)
            if !b.noten.is_empty() {
                ui.text_semi(t("ui.vk.noten"), r.x + 12.0, y, 13.0, Theme::text2());
                y += 20.0;
                let platz = ((r.y + r.h - 120.0 - y) / 18.0).max(0.0) as usize;
                // home on the left, away on the right, each best first
                // shortcut: the side comes from the player's club today, a player sold since then is left out
                for (seite, club) in [h, a].into_iter().enumerate() {
                    let mut n: Vec<&(usize, f32)> = b.noten.iter().filter(|(p, _)| c.world.players.get(*p).is_some_and(|pl| pl.team == club as i32)).collect();
                    n.sort_by(|p, q| q.1.partial_cmp(&p.1).unwrap_or(std::cmp::Ordering::Equal));
                    let xx = r.x + 12.0 + seite as f32 * (r.w / 2.0);
                    for (i, (p, note)) in n.iter().take(platz).enumerate() {
                        let yy = y + i as f32 * 18.0;
                        ui.text(&ui.fit(&name(*p), 12.0, r.w / 2.0 - 60.0), xx, yy, 12.0, Theme::text());
                        ui.text(&format!("{note:.1}"), xx + r.w / 2.0 - 48.0, yy, 12.0, if *note >= 7.0 { Theme::gut() } else if *note < 6.0 { Theme::problem() } else { Theme::text2() }); // roh
                    }
                }
            }
        }
        None => {}
    }
    // earlier meetings of the two clubs: this season's other matches and last season's
    let mut frueher: Vec<(u32, u32, usize, u8, u8)> = c.world.berichte.iter().filter(|b| (b.h == h && b.a == a || b.h == a && b.a == h) && !(b.season == c.world.season && b.day == x.f.day)).map(|b| (b.season, b.day, b.h, b.hg, b.ag)).collect();
    frueher.sort();
    let fy = r.y + r.h - 92.0;
    ui.text_semi(t("ui.vk.fruehere"), r.x + 12.0, fy, 13.0, Theme::text2());
    if frueher.is_empty() { ui.text(t("ui.vk.keine_frueheren"), r.x + 12.0, fy + 20.0, 12.0, Theme::gedaempft()); }
    for (i, (s, d, hh, hg, ag)) in frueher.iter().rev().take(3).enumerate() {
        let gegen = if *hh == h { a } else { h };
        ui.text(&format!("{}  {} {}  {hg}–{ag}", crate::date::short(*s, *d), c.world.club(*hh).short, c.world.club(gegen).short), r.x + 12.0, fy + 20.0 + i as f32 * 18.0, 12.0, Theme::text()); // roh
    }
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
            let cl = c.world.club(*club);
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
/// A gold trophy at (cx, cy), height h (Fabian 25.11.: football motifs instead of the sun).
fn pokal(cx: f32, cy: f32, h: f32, glanz: f32) {
    let g = Theme::gold();
    let s = h / 120.0;
    draw_circle(cx, cy - 20.0 * s, 34.0 * s, g);
    draw_rectangle(cx - 34.0 * s, cy - 54.0 * s, 68.0 * s, 34.0 * s, g);
    draw_circle_lines(cx - 40.0 * s, cy - 32.0 * s, 13.0 * s, 5.0 * s, g);
    draw_circle_lines(cx + 40.0 * s, cy - 32.0 * s, 13.0 * s, 5.0 * s, g);
    draw_rectangle(cx - 6.0 * s, cy + 12.0 * s, 12.0 * s, 22.0 * s, g);
    draw_rectangle(cx - 26.0 * s, cy + 32.0 * s, 52.0 * s, 10.0 * s, g);
    draw_rectangle(cx - 32.0 * s, cy + 42.0 * s, 64.0 * s, 14.0 * s, hex(0xb45309));
    // the shine running over the cup
    let x = cx - 30.0 * s + (glanz % 1.0) * 60.0 * s;
    draw_rectangle(x, cy - 52.0 * s, 6.0 * s, 60.0 * s, alpha(WHITE, 0.35));
}
/// A medal on its ribbon.
fn medaille(cx: f32, cy: f32, r: f32, band: Color) {
    draw_triangle(vec2(cx - r * 0.9, cy - r * 2.2), vec2(cx - r * 0.1, cy - r * 2.2), vec2(cx - r * 0.1, cy - r * 0.6), band);
    draw_triangle(vec2(cx + r * 0.9, cy - r * 2.2), vec2(cx + r * 0.1, cy - r * 2.2), vec2(cx + r * 0.1, cy - r * 0.6), alpha(band, 0.8));
    draw_circle(cx, cy, r, Theme::gold());
    draw_circle_lines(cx, cy, r * 0.78, 3.0, hex(0xb45309));
    draw_poly(cx, cy, 5, r * 0.42, -90.0, hex(0xb45309));
}
/// A shirt in the club colours with a number.
fn trikot(ui: &Ui, cx: f32, cy: f32, w: f32, c1: Color, c2: Color, nr: u8) {
    let h = w * 1.05;
    draw_rectangle(cx - w * 0.32, cy - h * 0.42, w * 0.64, h * 0.9, c1);
    draw_triangle(vec2(cx - w * 0.32, cy - h * 0.42), vec2(cx - w * 0.55, cy - h * 0.18), vec2(cx - w * 0.32, cy - h * 0.05), c1);
    draw_triangle(vec2(cx + w * 0.32, cy - h * 0.42), vec2(cx + w * 0.55, cy - h * 0.18), vec2(cx + w * 0.32, cy - h * 0.05), c1);
    draw_rectangle(cx - w * 0.32, cy - h * 0.42, w * 0.64, h * 0.06, c2);
    draw_circle(cx, cy - h * 0.42, w * 0.09, Theme::flaeche());
    ui.title(&nr.to_string(), cx - ui.measure_title(&nr.to_string(), w * 0.42) / 2.0, cy + h * 0.2, w * 0.42, on(c1));
}
/// Floodlights: two masts with beams towards the middle.
fn flutlicht(r: Rect, t: f32) {
    for (x, seite) in [(r.x + 40.0, 1.0f32), (r.x + r.w - 40.0, -1.0)] {
        draw_rectangle(x - 3.0, r.y + 30.0, 6.0, r.h - 40.0, alpha(WHITE, 0.25));
        draw_rectangle(x - 16.0, r.y + 18.0, 32.0, 16.0, alpha(WHITE, 0.85));
        let flackern = 0.10 + 0.04 * (t * 2.0 + x).sin();
        draw_triangle(vec2(x, r.y + 26.0), vec2(x + seite * r.w * 0.55, r.y + r.h), vec2(x + seite * r.w * 0.20, r.y + r.h), alpha(WHITE, flackern));
    }
}
/// Confetti in the club colours, falling and turning, seeded per piece.
fn konfetti(r: Rect, t: f32, farben: [Color; 3]) {
    for k in 0..70 {
        let s = (k as f32 * 12.9898).sin() * 43758.547;
        let fx = s - s.floor();
        let speed = 50.0 + 70.0 * ((k * 7 % 11) as f32 / 11.0);
        let y = r.y + ((t * speed + k as f32 * 37.0) % (r.h + 40.0)) - 20.0;
        let x = r.x + fx * r.w + 12.0 * (t * 2.0 + k as f32).sin();
        let w = 6.0 + 4.0 * (t * 3.0 + k as f32).cos().abs();
        draw_rectangle(x, y, w, 4.0, farben[k % 3]);
    }
}

/// Full-screen moments (Fabian 25.11.: more spectacular, fitting a football manager, no sun): three levels.
/// Title and award: club band, trophy or golden ball, confetti. Big match ahead (derby, final): floodlights, both
/// crests, the competition's colour. Signing and new contract: the shirt with his number and a stamp. Milestone and
/// licence: a medal on a ribbon.
pub fn feier_overlay(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let Some(f) = c.feiern.first().cloned() else { return };
    if !v.feier_ton { ui.play(if matches!(f.art.as_str(), "meilenstein" | "lizenz" | "privat") { crate::id("meilenstein") } else { crate::id("titel") }); v.feier_ton = true; }
    let (w, h) = (breite(), hoehe());
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.80));
    let t = get_time() as f32;
    let art = f.art.as_str();
    let gross = matches!(art, "titel" | "gala" | "siegerehrung");
    let spiel = matches!(art, "derby" | "finale");
    let deal = matches!(art, "neuzugang" | "vertrag");
    let hoch = (if gross { 560.0 } else { 520.0 }) + if f.stimmen.is_empty() { 0.0 } else { 30.0 * f.stimmen.len() as f32 + 20.0 };
    let r = Rect::new(w / 2.0 - 400.0, h / 2.0 - hoch / 2.0, 800.0, hoch);
    // the colours: own club, or the competition for a big match
    let own = f.club.unwrap_or(c.own);
    let (k1, k2) = club_color(c, own);
    let band = match f.wo { Some((pokal, i)) if spiel => wo_look(c, if pokal { Wo::Pokal(i) } else { Wo::Liga(i) }).c1, _ => k1 };
    // appearing: the card drops in from a little above over a quarter second
    if v.feier_start <= 0.0 || v.feier_start > t { v.feier_start = t; }
    let auf = ((t - v.feier_start) * 4.0).clamp(0.0, 1.0);
    let r = Rect::new(r.x, r.y - 40.0 * (1.0 - auf), r.w, r.h);
    rrect(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 20.0, if gross { Theme::gold() } else { band });
    rrect(r.x, r.y, r.w, r.h, 18.0, Theme::flaeche());
    let kopf = Rect::new(r.x, r.y, r.w, 250.0);
    rrect(kopf.x, kopf.y, kopf.w, kopf.h, 18.0, band);
    draw_rectangle(kopf.x, kopf.y + kopf.h - 18.0, kopf.w, 18.0, band);
    // a pitch pattern on the band
    for k in 0..8 { if k % 2 == 0 { draw_rectangle(kopf.x + k as f32 * kopf.w / 8.0, kopf.y + 18.0, kopf.w / 8.0, kopf.h - 18.0, alpha(BLACK, 0.06)); } }
    draw_circle_lines(kopf.x + kopf.w / 2.0, kopf.y + kopf.h, 90.0, 3.0, alpha(WHITE, 0.18));
    let (cx, cy) = (r.x + r.w / 2.0, r.y + 128.0);
    if gross {
        konfetti(r, t, [k1, k2, Theme::gold()]);
        if art == "gala" {
            // the golden ball: a ball in gold with its pentagon
            draw_circle(cx, cy, 70.0, Theme::gold());
            draw_poly(cx, cy, 5, 22.0, -90.0 + t * 20.0, hex(0xb45309));
            for k in 0..5 { let a = (t * 20.0 - 90.0 + k as f32 * 72.0).to_radians(); draw_line(cx + 22.0 * a.cos(), cy + 22.0 * a.sin(), cx + 62.0 * a.cos(), cy + 62.0 * a.sin(), 3.0, hex(0xb45309)); }
            if let Some(p) = f.spieler { ui.player_photo(cx + 120.0, cy + 30.0, 46.0, c.world.players[p].id, &c.world.players[p].name.clone()); }
        } else {
            pokal(cx, cy + 10.0, 170.0 * (1.0 + 0.03 * (t * 2.5).sin()), t * 0.4);
            let k = &c.world.clubs[own];
            ui.club_crest(cx - 170.0, cy + 10.0, 44.0, k.id, hex(k.c1), &k.short.clone());
            // the ceremony shows the beaten finalist on the right, a little smaller
            let (rk, rr) = match f.gegner { Some(g) if art == "siegerehrung" => (&c.world.clubs[g], 34.0), _ => (k, 44.0) };
            ui.club_crest(cx + 170.0, cy + 10.0, rr, rk.id, hex(rk.c1), &rk.short.clone());
        }
    } else if spiel {
        flutlicht(kopf, t);
        let ko = &c.world.clubs[own];
        ui.club_crest(cx - 150.0, cy, 64.0, ko.id, hex(ko.c1), &ko.short.clone());
        if let Some(g) = f.gegner { let kg = &c.world.clubs[g]; ui.club_crest(cx + 150.0, cy, 64.0, kg.id, hex(kg.c1), &kg.short.clone()); }
        let vs = 1.0 + 0.06 * (t * 4.0).sin();
        ui.title("VS", cx - ui.measure_title("VS", 54.0 * vs) / 2.0, cy + 20.0, 54.0 * vs, on(band)); // roh
    } else if deal {
        let nr = f.spieler.map(|p| c.world.players[p].no).filter(|n| *n > 0).unwrap_or(10);
        // a white outline so the shirt stands out on a band of the same colour
        trikot(ui, cx + 110.0, cy + 6.0, 162.0, WHITE, WHITE, nr);
        trikot(ui, cx + 110.0, cy + 6.0, 150.0, k1, k2, nr);
        if let Some(p) = f.spieler { draw_circle(cx - 110.0, cy, 74.0, alpha(WHITE, 0.9)); ui.player_photo(cx - 110.0, cy, 68.0, c.world.players[p].id, &c.world.players[p].name.clone()); }
        // the stamp, turned a little
        let stempel = crate::inhalt::t(if art == "vertrag" { "ui.feier.stempel_vertrag" } else { "ui.feier.stempel_neu" });
        let sw = ui.measure_title(stempel, 26.0) + 24.0;
        rrect(cx - sw / 2.0, cy + 70.0, sw, 40.0, 6.0, alpha(Theme::gold(), 0.95));
        ui.title(stempel, cx - sw / 2.0 + 12.0, cy + 100.0, 26.0, hex(0x1b1f27));
    } else {
        medaille(cx, cy + 40.0, 58.0 * (1.0 + 0.04 * (t * 3.0).sin()), k2);
        if let Some(p) = f.spieler { ui.player_photo(cx + 140.0, cy + 30.0, 44.0, c.world.players[p].id, &c.world.players[p].name.clone()); }
    }
    let art_t = match art { "neuzugang" => crate::inhalt::t("ui.feier.art.neuzugang"), "titel" => crate::inhalt::t("ui.feier.art.titel"), "lizenz" => crate::inhalt::t("ui.feier.art.lizenz"), "gala" => crate::inhalt::t("ui.feier.art.gala"), "vertrag" => crate::inhalt::t("ui.feier.art.vertrag"), "derby" => crate::inhalt::t("ui.feier.art.derby"), "finale" => crate::inhalt::t("ui.feier.art.finale"), "siegerehrung" => crate::inhalt::t("ui.feier.art.siegerehrung"), _ => crate::inhalt::t("ui.meilenstein.titel") };
    ui.text_center(&art_t.to_uppercase(), cx, r.y + 286.0, 15.0, if gross { Theme::gold() } else { farbe_lesbar(band) });
    let size = if ui.measure_title(&f.titel, 42.0) > r.w - 60.0 { 30.0 } else { 42.0 };
    let tw = ui.measure_title(&f.titel, size);
    ui.title(&f.titel, cx - tw / 2.0, r.y + 334.0, size, Theme::text());
    let mut y = r.y + 372.0;
    for z in ui.wrap(&f.text, 17.0, r.w - 100.0).into_iter().take(4) { ui.text_center(&z, cx, y, 17.0, Theme::text2()); y += 24.0; }
    y += 14.0;
    for (wer, was) in &f.stimmen {
        let ww = ui.measure(wer, 15.0) + 12.0;
        ui.text(wer, r.x + 50.0, y, 15.0, Theme::gold());
        let was = if art == "siegerehrung" { was.clone() } else { format!("\"{was}\"") }; // scorers, not quotes
        ui.text(&ui.fit(&was, 15.0, r.w - 100.0 - ww), r.x + 50.0 + ww, y, 15.0, Theme::text());
        y += 30.0;
    }
    let weiter = ui.button_gold(Rect::new(cx - 120.0, r.y + r.h - 70.0, 240.0, 48.0), crate::inhalt::t(if spiel { "ui.feier.los" } else { "ui.feier.weiter" }));
    if weiter || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
        c.feiern.remove(0);
        v.feier_ton = false;
        v.feier_start = 0.0;
    }
}

/// Yes to the direct start: on for this career with everything it needs, then the guide (or the error page).
fn direktstart_einschalten(v: &mut View, c: &mut Career) {
    c.direktstart = true;
    // the yes covers the launch from CalcioM too: nothing more to allow on match day
    c.fl27_erlaubt = true;
    c.fl27_bestaetigt = true;
    // quick start (Fabian 26.11., "the game changer"): FL27 stays open and waits at the side selection
    c.schnellstart = true;
    c.fl27_auto_ende = false;
    let mut st = crate::game::Fl27Start { c: &mut *c, assets: crate::game::fl27_assets(), log: vec![] };
    let ok = st.direktstart_bauen();
    v.fl27_log = st.log;
    // the real reason on the error page (Fabian 26.11.: the Sunderland career only said "files not found")
    v.ds_grund = ok.as_ref().err().cloned();
    v.ds_frage = if matches!(ok, Ok(true)) { 2 } else { 3 };
}
/// The one-time question after the update with the direct start (Fabian 26.11.): yes builds the whole way at once
/// (flow files from the player's game, module, sider.ini line), then a short guide of what happens from now on.
pub fn direktstart_frage(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let (w, h) = (breite(), hoehe());
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.75));
    let r = Rect::new(w / 2.0 - 380.0, (h / 2.0 - 230.0).max(20.0), 760.0, 460.0);
    rrect(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 20.0, Theme::gold());
    rrect(r.x, r.y, r.w, r.h, 18.0, Theme::flaeche());
    let (x, bw) = (r.x + 32.0, r.w - 64.0);
    let titel = ["ui.ds.frage.titel", "ui.ds.fertig.titel", "ui.ds.fehler.titel", "ui.ds.ohne.titel"][(v.ds_frage as usize).clamp(1, 4) - 1]; // roh
    ui.title(&ui.fit(crate::inhalt::t(titel), 30.0, bw), x, r.y + 60.0, 30.0, Theme::text());
    let mut y = r.y + 100.0;
    match v.ds_frage {
        1 => {
            ui.paragraph(crate::inhalt::t("ui.ds.frage.text"), x, y, 17.0, bw, Theme::text2());
            let bt = Rect::new(x, r.y + r.h - 76.0, bw / 2.0 - 8.0, 48.0);
            if ui.button_gold(bt, crate::inhalt::t("ui.ds.ja")) {
                direktstart_einschalten(v, c);
                let _ = std::fs::write(direktstart_file(), "ja"); // roh
            }
            if ui.button(Rect::new(bt.x + bw / 2.0 + 8.0, bt.y + 7.0, bw / 2.0 - 8.0, 34.0), crate::inhalt::t("ui.ds.nein"), false) {
                let _ = std::fs::write(direktstart_file(), "nein"); // roh
                v.ds_frage = 0;
            }
        }
        // a career without the direct start (Fabian 26.11.): what it misses; later again, unless "only simulate"
        4 => {
            ui.paragraph(crate::inhalt::t("ui.ds.ohne.text"), x, y, 17.0, bw, Theme::text2());
            let bt = Rect::new(x, r.y + r.h - 124.0, bw / 2.0 - 8.0, 48.0);
            if ui.button_gold(bt, crate::inhalt::t("ui.ds.ohne.ja")) { direktstart_einschalten(v, c); }
            if ui.button(Rect::new(bt.x + bw / 2.0 + 8.0, bt.y + 7.0, bw / 2.0 - 8.0, 34.0), crate::inhalt::t("ui.ds.ohne.spaeter"), false) { v.ds_frage = 0; }
            if ui.button(Rect::new(x, r.y + r.h - 62.0, bw, 34.0), crate::inhalt::t("ui.ds.ohne.nie"), false) { c.ds_nicht_erinnern = true; v.ds_frage = 0; }
        }
        2 => {
            for (i, k) in ["ui.ds.schritt.1", "ui.ds.schritt.2", "ui.ds.schritt.3", "ui.ds.schritt.4", "ui.ds.schritt.5"].iter().enumerate() { // roh
                draw_circle(x + 13.0, y + 11.0, 13.0, Theme::gold());
                ui.text_center(&(i + 1).to_string(), x + 13.0, y + 17.0, 15.0, Theme::flaeche());
                let zeilen = ui.wrap(crate::inhalt::t(k), 16.0, bw - 44.0);
                for (j, z) in zeilen.iter().enumerate() { ui.text(z, x + 40.0, y + 16.0 + j as f32 * 21.0, 16.0, Theme::text()); }
                y += 21.0 * zeilen.len().max(1) as f32 + 14.0;
            }
            ui.paragraph(crate::inhalt::t("ui.ds.luecken"), x, y + 10.0, 15.0, bw, Theme::text2());
            if ui.button_gold(Rect::new(x, r.y + r.h - 76.0, bw, 48.0), crate::inhalt::t("ui.ds.ok")) { v.ds_frage = 0; }
        }
        _ => {
            match &v.ds_grund {
                Some(g) => { ui.paragraph(&crate::tf!("ui.ds.fehler.grund", g.clone()), x, y, 17.0, bw, Theme::text2()); }
                None => { ui.paragraph(crate::inhalt::t("ui.ds.fehler.text"), x, y, 17.0, bw, Theme::text2()); }
            }
            if ui.button_gold(Rect::new(x, r.y + r.h - 76.0, bw, 48.0), crate::inhalt::t("ui.ds.ok")) { v.ds_frage = 0; }
        }
    }
}

/// "The app is working" (Fabian 26.11.): three pulsing dots and a word. `gross`: a card in the middle that covers the
/// screen (loading), else a small badge at the bottom left (saving in the background, the game goes on).
pub fn arbeitet(ui: &mut Ui, text: &str, gross: bool) {
    let t = get_time() as f32;
    let (w, h) = (breite(), hoehe());
    let (bx, by, bw, bh) = if gross { (w / 2.0 - 180.0, h / 2.0 - 50.0, 360.0, 100.0) } else { (x0() + 12.0, h - 52.0, 190.0, 40.0) };
    if gross { draw_rectangle(0.0, 0.0, w, h, alpha(Theme::grund(), 0.85)); }
    rrect(bx, by, bw, bh, 12.0, Theme::erhoehung());
    let mitte_y = by + bh / 2.0;
    for i in 0..3 {
        let phase = ((t * 3.0 - i as f32 * 0.6).sin() + 1.0) / 2.0;
        draw_circle(bx + 22.0 + i as f32 * 14.0, mitte_y, 3.0 + phase * 2.5, alpha(Theme::akzent(), 0.4 + 0.6 * phase));
    }
    ui.text_semi(text, bx + 72.0, mitte_y + 6.0, if gross { 18.0 } else { 15.0 }, Theme::text());
}
