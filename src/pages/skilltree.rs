use super::*;
use crate::skilltree::{knoten, Knoten};

thread_local! { static WAHL: std::cell::RefCell<Option<(usize, String)>> = const { std::cell::RefCell::new(None) }; }

const GOLD: u32 = 0xfbbf24;
/// The colour of a branch: the three role branches in their own tones, Positions in pitch green.
fn ast_farbe(a: usize) -> Color { hex([0x3b82f6, 0xef4444, 0xa855f7, 0x22c55e][a.min(3)]) }

/// Skill Tree tab of the player profile (Fabian 25.11.: like a ball, the player in the middle of the net, the skills
/// going outwards): rings are the tiers, the role branches and Positions are the four sectors, nodes sit on the
/// rings; the panel on the right shows the account, the bar and the chosen node with its reason and the buy button.
pub(super) fn skilltree_reiter(ui: &mut Ui, v: &mut View, c: &mut Career, p: usize, cy: f32, ch: f32) {
    let t = |k: &str| crate::inhalt::t(k);
    let pl = c.world.players[p].clone();
    let area = Rect::new(x0() + 4.0, cy, cbreite(), ch);
    if !c.feature("skilltree") { ui.text(crate::inhalt::t("ui.features.ist_aus"), area.x + 16.0, area.y + 40.0, 16.0, Theme::text2()); return; } // roh
    ui.card(area);
    let panel_w = 380.0;
    let feld = Rect::new(area.x, area.y, area.w - panel_w - 12.0, area.h);
    let (mx, my) = (feld.x + feld.w / 2.0, feld.y + feld.h / 2.0 + 6.0);
    let rr = ((feld.h - 40.0) / 2.0).min(feld.w / 2.0 - 30.0);
    let ring = |k: u8| rr * (0.24 + 0.165 * k as f32);
    let zeit = get_time() as f32;
    // the ball: pentagon in the middle, hexagon net rings and seams to the edge
    let ink = alpha(Theme::text(), 0.10);
    draw_circle(mx, my, rr, alpha(Theme::text(), 0.03));
    draw_circle_lines(mx, my, rr, 2.0, alpha(Theme::text(), 0.16));
    for k in 1..=4u8 { let r = ring(k); for s in 0..6 { let (a0, a1) = (s as f32 * std::f32::consts::TAU / 6.0 + 0.26, (s + 1) as f32 * std::f32::consts::TAU / 6.0 + 0.26); draw_line(mx + r * a0.cos(), my + r * a0.sin(), mx + r * a1.cos(), my + r * a1.sin(), 1.0, ink); } }
    for s in 0..10 { let a = s as f32 * std::f32::consts::TAU / 10.0 - 1.57; draw_line(mx + ring(1) * 0.7 * a.cos(), my + ring(1) * 0.7 * a.sin(), mx + rr * a.cos(), my + rr * a.sin(), 1.0, ink); }
    // the four sectors: role branches top-left, top-right, bottom-right; Positions bottom-left
    let baum = crate::skilltree::baum_von(&pl.pos);
    let aeste = crate::skilltree::aeste(&baum);
    let mut alle: Vec<Knoten> = knoten(&baum);
    alle.extend(c.pos_knoten(p));
    let mitte_von = |ast: usize| -> f32 { [-2.36, -0.79, 0.79, 2.36][if ast == 99 { 3 } else { ast.min(2) }] };
    let mut orte: Vec<(Knoten, f32, f32)> = vec![];
    for (ast, _) in aeste.iter().enumerate().chain(std::iter::once((99usize, &String::new()))) {
        for stufe in 1..=4u8 {
            let hier: Vec<&Knoten> = alle.iter().filter(|k| k.ast == ast && k.stufe == stufe).collect();
            let n = hier.len();
            for (i, k) in hier.into_iter().enumerate() {
                // the whole quarter of the ball: tiers 1 and 3 wide, tier 2 a little narrower, like the seams of a ball
                let breite = if stufe % 2 == 1 { 1.25 } else { 0.85 };
                let spread = if n <= 1 { 0.0 } else { (i as f32 - (n - 1) as f32 / 2.0) * breite / (n - 1) as f32 };
                let a = mitte_von(ast) + spread;
                orte.push((k.clone(), mx + ring(stufe) * a.cos(), my + ring(stufe) * a.sin()));
            }
        }
    }
    // lines: tier to tier inside a branch, the first tier to the player
    for (k, x, y) in &orte {
        let col = if k.ast == 99 { ast_farbe(3) } else { ast_farbe(k.ast) };
        let gekauft = pl.st.knoten.contains(&k.id);
        if k.stufe == 1 { draw_line(mx, my, *x, *y, 2.0, alpha(col, if gekauft { 0.8 } else { 0.25 })); }
        for (k2, x2, y2) in orte.iter().filter(|o| o.0.ast == k.ast && o.0.stufe == k.stufe + 1) {
            let an = gekauft && pl.st.knoten.contains(&k2.id);
            draw_line(*x, *y, *x2, *y2, 2.0, alpha(col, if an { 0.8 } else { 0.18 }));
        }
    }
    // the branch names on the outer ring
    for (ast, name) in aeste.iter().enumerate().chain(std::iter::once((99usize, &crate::id("positions").to_string()))) {
        let a = mitte_von(ast);
        let (x, y) = (mx + (rr + 30.0) * a.cos(), my + (rr + 30.0) * a.sin());
        let txt = t(&format!("ui.st.ast.{name}"));
        let w = ui.measure_semi(txt, 14.0) + 16.0;
        let col = ast_farbe(if ast == 99 { 3 } else { ast });
        rrect(x - w / 2.0, y - 12.0, w, 24.0, 12.0, col);
        ui.text_center(txt, x, y + 5.0, 14.0, hex(0xffffff));
    }
    // the player in the middle
    draw_circle(mx, my, ring(1) * 0.62, Theme::teil(&pl.pos));
    ui.player_photo(mx, my - 4.0, ring(1) * 0.5, pl.id, &pl.name);
    ui.text_center(&ui.fit(&pl.name, 13.0, ring(1) * 1.2), mx, my + ring(1) * 0.62 + 16.0, 13.0, Theme::text());
    // the nodes
    let mut gewaehlt = WAHL.with(|w| w.borrow().clone().filter(|x| x.0 == p).map(|x| x.1));
    for (k, x, y) in &orte {
        let r = if k.stufe == 4 { 21.0 } else { 17.0 };
        let gekauft = pl.st.knoten.contains(&k.id);
        let lernt = pl.st.lernziel.as_ref().is_some_and(|l| l.0 == k.id) || (k.ast == 99 && pl.st.pos_lernt.as_ref().is_some_and(|l| l.0 == k.name));
        let frei = c.knoten_frei(p, k).is_ok();
        let col = if k.ast == 99 { ast_farbe(3) } else { ast_farbe(k.ast) };
        let puls = if lernt { 0.5 + 0.5 * (zeit * 3.0).sin() } else { 0.0 };
        draw_circle(*x, *y, r + 3.0, if gewaehlt.as_deref() == Some(&k.id) { Theme::text() } else { Theme::grund() });
        draw_circle(*x, *y, r, if gekauft { col } else if lernt { alpha(col, 0.4 + 0.4 * puls) } else { alpha(Theme::gedaempft(), 0.25) });
        if frei { draw_circle_lines(*x, *y, r + 1.5, 3.0, hex(GOLD)); }
        // the assistant's suggestion: a dashed ring
        if pl.st.vorschlag.as_ref().is_some_and(|v| v.0 == k.id) { for s in 0..12 { if s % 2 == 0 { let (a0, a1) = (s as f32 * 0.5236 + zeit * 0.5, (s + 1) as f32 * 0.5236 + zeit * 0.5); draw_line(x + (r + 7.0) * a0.cos(), y + (r + 7.0) * a0.sin(), x + (r + 7.0) * a1.cos(), y + (r + 7.0) * a1.sin(), 2.5, Theme::akzent()); } } }
        let farbe_z = if gekauft || lernt { hex(0xffffff) } else { Theme::text2() };
        match k.art {
            // the signature node is the ball's pentagon
            'X' => draw_poly(*x, *y, 5, r * 0.55, -90.0, farbe_z),
            'L' | 'M' | 'H' => ui.text_center(pos_de(&k.name), *x, *y + 5.0, 12.0, farbe_z),
            _ => ui.text_center(match k.art { 'A' => "+", 'S' => "S", 'P' => "P", _ => "E" }, *x, *y + 6.0, 16.0, farbe_z), // roh
        }
        // the FL27 light: green in the game, blue CalcioM only
        let gruen = !matches!(k.art, 'E');
        draw_circle(*x + r * 0.7, *y - r * 0.7, 4.5, if gruen { hex(0x22c55e) } else { hex(0x3b82f6) });
        if frei { draw_circle(*x - r * 0.75, *y + r * 0.75, 8.0, hex(GOLD)); ui.text_center(&k.kosten.to_string(), *x - r * 0.75, *y + r * 0.75 + 4.0, 11.0, hex(0x1b1f27)); }
        if ui.hover(Rect::new(x - r, y - r, 2.0 * r, 2.0 * r)) && is_mouse_button_pressed(MouseButton::Left) { gewaehlt = Some(k.id.clone()); WAHL.with(|w| *w.borrow_mut() = Some((p, k.id.clone()))); }
    }
    // the panel: coins, bar, season, the chosen node, the log
    let pr = Rect::new(area.x + area.w - panel_w - 4.0, area.y + 12.0, panel_w - 12.0, area.h - 24.0);
    rrect(pr.x, pr.y, pr.w, pr.h, 10.0, alpha(hex(GOLD), 0.08));
    let mut y = pr.y + 34.0;
    for i in 0..pl.st.konto.min(6) { draw_circle(pr.x + 24.0 + i as f32 * 18.0, y - 6.0, 11.0, hex(GOLD)); draw_circle_lines(pr.x + 24.0 + i as f32 * 18.0, y - 6.0, 11.0, 2.0, hex(0xb45309)); }
    ui.text_semi(&crate::tf!("ui.st.konto", pl.st.konto), pr.x + 34.0 + pl.st.konto.min(6) as f32 * 18.0, y, 18.0, Theme::text());
    y += 30.0;
    ui.bar(pr.x + 14.0, y, pr.w - 28.0, 10.0, pl.st.balken / 100.0, hex(GOLD));
    y += 26.0;
    let grenze = crate::regeln::wert("skilltree", "punkte.grenze_alter").as_array().into_iter().flatten().find(|x| x[0].as_i64().is_some_and(|b| pl.age as i64 <= b)).and_then(|x| x[1].as_u64()).unwrap_or(2);
    ui.text(&crate::tf!("ui.st.balken", format!("{:.0}", pl.st.balken), pl.st.saison_sp, grenze), pr.x + 14.0, y, 13.0, Theme::text2());
    y += 14.0;
    // who spends the points: the coach or his assistant (Konzept 6.1)
    let auto = c.st_assistent(p);
    if let Some(i) = ui.segment(Rect::new(pr.x + 14.0, y, pr.w - 28.0, 30.0), &[t("ui.st.vergabe.ich"), t("ui.st.vergabe.co")], auto as usize) { c.world.players[p].st.vergabe = if i == 1 { 2 } else { 1 }; }
    y += 50.0;
    if let Some((kid, tr, _)) = &pl.st.lernziel { if let Some(k) = alle.iter().find(|x| &x.id == kid) { ui.text(&crate::tf!("ui.st.lernt", c.knoten_name(k), tr, crate::regeln::f("skilltree", "regeln.lern_treffer") as u8), pr.x + 14.0, y, 13.0, Theme::akzent()); y += 20.0; } }
    if let Some((pos, bis)) = &pl.st.pos_lernt { ui.text(&crate::tf!("ui.st.lernt_pos", pos_de(pos), bis.saturating_sub(c.abs()) / 7 + 1), pr.x + 14.0, y, 13.0, Theme::akzent()); y += 20.0; }
    y += 10.0;
    match gewaehlt.and_then(|g| alle.iter().find(|k| k.id == g).cloned()) {
        Some(k) => {
            let art = t(&format!("ui.st.art.{}", k.art));
            ui.text_semi(&art.to_uppercase(), pr.x + 14.0, y, 12.0, Theme::gedaempft());
            ui.title(&ui.fit(&c.knoten_name(&k), 22.0, pr.w - 28.0), pr.x + 14.0, y + 28.0, 22.0, Theme::text());
            y += 44.0;
            // effect in figures: abilities now and after, the rating at his position
            if !k.werte.is_empty() {
                let mut neu = pl.attrs.clone();
                for (a, w) in &k.werte { if let Some(x) = neu.get_mut(*a) { *x = (*x + w).min(99.0); } ui.text(&format!("{} {:.0} → {:.0}", crate::pesdb::attr_de_t()[*a], pl.attrs.get(*a).copied().unwrap_or(0.0), (pl.attrs.get(*a).copied().unwrap_or(0.0) + w).min(99.0)), pr.x + 14.0, y, 14.0, Theme::text2()); y += 20.0; }
                if pl.attrs.len() == 25 { let pi = crate::pesdb::pos_index(&pl.pos); ui.text(&crate::tf!("ui.st.ovr", pos_de(&pl.pos), format!("{:.0}", crate::pesdb::ovr(&pl.attrs, pi)), format!("{:.0}", crate::pesdb::ovr(&neu, pi))), pr.x + 14.0, y, 14.0, Theme::text()); y += 20.0; }
            }
            if let Some(s) = &k.skill { ui.text(&crate::tf!(if k.art == 'P' { "ui.st.wirkung_stil" } else { "ui.st.wirkung_skill" }, s), pr.x + 14.0, y, 14.0, Theme::text2()); y += 20.0; }
            if k.art == 'X' { y = ui.paragraph(t("ui.st.signature_spaeter"), pr.x + 14.0, y, 13.0, pr.w - 28.0, Theme::gedaempft()); }
            let gruen = k.art != 'E';
            draw_circle(pr.x + 20.0, y + 2.0, 5.0, if gruen { hex(0x22c55e) } else { hex(0x3b82f6) });
            ui.text(t(if gruen { "ui.st.fl27_ja" } else { "ui.st.fl27_nein" }), pr.x + 32.0, y + 7.0, 13.0, Theme::text2());
            y += 26.0;
            match c.knoten_frei(p, &k) {
                Ok(kosten) => {
                    if ui.button_gold(Rect::new(pr.x + 14.0, y, pr.w - 28.0, 40.0), &crate::tf!("ui.st.kaufen", kosten)) {
                        match c.knoten_kaufen(p, &k) { Ok(txt) => v.toast(&txt, true), Err(e) => v.toast(&e, false) }
                    }
                    y += 52.0;
                }
                Err(grund) => { y = ui.paragraph(&grund, pr.x + 14.0, y + 14.0, 14.0, pr.w - 28.0, Theme::achtung()) + 8.0; }
            }
        }
        None => { y = ui.paragraph(t("ui.st.hinweis"), pr.x + 14.0, y, 14.0, pr.w - 28.0, Theme::text2()) + 8.0; }
    }
    // the log, newest first
    ui.text_semi(&t("ui.st.verlauf").to_uppercase(), pr.x + 14.0, y + 10.0, 12.0, Theme::gedaempft());
    y += 30.0;
    for (abs, txt, d) in pl.st.log.iter().rev() {
        if y > pr.y + pr.h - 16.0 { break; }
        let _ = abs;
        let col = if *d > 0 { hex(GOLD) } else { Theme::text2() };
        for z in ui.wrap(txt, 12.0, pr.w - 28.0).into_iter().take(2) { ui.text(&z, pr.x + 14.0, y, 12.0, col); y += 15.0; }
        y += 4.0;
    }
}

thread_local! { static M_WAHL: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) }; }
/// The manager's Skill Tree (Konzept 8.3): five branches round the coach like the ball's five pentagon sides
/// (Tactics blue, People green, Development teal, Market orange, Media purple); the licence line says how high he may
/// climb; the panel shows the Coaching Points, the chosen node and the log.
pub(super) fn trainerbaum(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::skilltree::{m_aeste, m_knoten};
    let t = |k: &str| crate::inhalt::t(k);
    if !c.feature("skilltree") { ui.text(crate::inhalt::t("ui.features.ist_aus"), x0() + 20.0, Y0 + 40.0, 16.0, Theme::text2()); return; } // roh
    c.manager_an();
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    let panel_w = 380.0;
    let feld = Rect::new(area.x, area.y, area.w - panel_w - 12.0, area.h);
    let (mx, my) = (feld.x + feld.w / 2.0, feld.y + feld.h / 2.0 + 6.0);
    let rr = ((feld.h - 100.0) / 2.0).min(feld.w / 2.0 - 40.0);
    let ring = |k: u8| rr * (0.27 + 0.16 * k as f32);
    let farben = [0x3b82f6u32, 0x22c55e, 0x14b8a6, 0xf97316, 0xa855f7];
    let ink = alpha(Theme::text(), 0.10);
    draw_circle(mx, my, rr, alpha(Theme::text(), 0.03));
    draw_circle_lines(mx, my, rr, 2.0, alpha(Theme::text(), 0.16));
    for k in 1..=4u8 { let r = ring(k); for s in 0..5 { let (a0, a1) = (s as f32 * 1.2566 - 1.5708, (s + 1) as f32 * 1.2566 - 1.5708); draw_line(mx + r * a0.cos(), my + r * a0.sin(), mx + r * a1.cos(), my + r * a1.sin(), 1.0, ink); } }
    // the licence: the rings above it are faded
    let max = c.m_stufe_max();
    if max < 4 { draw_circle_lines(mx, my, (ring(max) + ring(max + 1)) / 2.0, 3.0, alpha(Theme::achtung(), 0.6)); }
    let alle = m_knoten();
    let mitte = |a: usize| a as f32 * 1.2566 - 1.5708 + 0.6283;
    let mut orte = vec![];
    for k in &alle {
        let n = alle.iter().filter(|x| x.ast == k.ast && x.stufe == k.stufe).count();
        let i = alle.iter().filter(|x| x.ast == k.ast && x.stufe == k.stufe).position(|x| x.id == k.id).unwrap_or(0);
        let a = mitte(k.ast) + if n <= 1 { 0.0 } else { (i as f32 - 0.5) * 0.52 };
        orte.push((k.clone(), mx + ring(k.stufe) * a.cos(), my + ring(k.stufe) * a.sin()));
    }
    for (k, x, y) in &orte {
        let col = hex(farben[k.ast % 5]);
        let gekauft = c.mst.knoten.contains(&k.id);
        if k.stufe == 1 { draw_line(mx, my, *x, *y, 2.0, alpha(col, if gekauft { 0.8 } else { 0.25 })); }
        for (k2, x2, y2) in orte.iter().filter(|o| o.0.ast == k.ast && o.0.stufe == k.stufe + 1) { draw_line(*x, *y, *x2, *y2, 2.0, alpha(col, if gekauft && c.mst.knoten.contains(&k2.id) { 0.8 } else { 0.18 })); }
    }
    for (a, name) in m_aeste().iter().enumerate() {
        let w = mitte(a);
        let (x, y) = (mx + (rr + 22.0) * w.cos(), my + (rr + 22.0) * w.sin());
        let txt = t(&format!("ui.mst.ast.{name}"));
        let bw = ui.measure_semi(txt, 14.0) + 16.0;
        rrect(x - bw / 2.0, y - 12.0, bw, 24.0, 12.0, hex(farben[a % 5]));
        ui.text_center(txt, x, y + 5.0, 14.0, hex(0xffffff));
    }
    draw_circle(mx, my, ring(1) * 0.5, Theme::verein());
    ui.text_center(&ui.fit(&c.coach.clone(), 14.0, ring(1) * 0.9), mx, my + 5.0, 14.0, on(Theme::verein()));
    let mut gewaehlt = M_WAHL.with(|w| w.borrow().clone());
    for (k, x, y) in &orte {
        let r = if k.stufe == 4 { 21.0 } else { 17.0 };
        let gekauft = c.mst.knoten.contains(&k.id);
        let frei = c.m_frei(k).is_ok();
        let col = hex(farben[k.ast % 5]);
        let blass = k.stufe > max;
        draw_circle(*x, *y, r + 3.0, if gewaehlt.as_deref() == Some(&k.id) { Theme::text() } else { Theme::grund() });
        draw_circle(*x, *y, r, if gekauft { col } else { alpha(Theme::gedaempft(), if blass { 0.12 } else { 0.25 }) });
        if frei { draw_circle_lines(*x, *y, r + 1.5, 3.0, hex(GOLD)); }
        let fz = if gekauft { hex(0xffffff) } else { Theme::text2() };
        if k.art == 'X' { draw_poly(*x, *y, 5, r * 0.55, -90.0, fz); } else { ui.text_center(match k.art { 'W' => "+", 'T' => "T", _ => "K" }, *x, *y + 6.0, 16.0, fz); } // roh
        draw_circle(*x + r * 0.7, *y - r * 0.7, 4.5, hex(0x3b82f6));
        if ui.hover(Rect::new(x - r, y - r, 2.0 * r, 2.0 * r)) && is_mouse_button_pressed(MouseButton::Left) { gewaehlt = Some(k.id.clone()); M_WAHL.with(|w| *w.borrow_mut() = Some(k.id.clone())); }
    }
    // the panel
    let pr = Rect::new(area.x + area.w - panel_w - 4.0, area.y + 12.0, panel_w - 12.0, area.h - 24.0);
    rrect(pr.x, pr.y, pr.w, pr.h, 10.0, alpha(hex(GOLD), 0.08));
    let mut y = pr.y + 34.0;
    for i in 0..c.mst.konto.min(5) { draw_circle(pr.x + 24.0 + i as f32 * 18.0, y - 6.0, 11.0, hex(GOLD)); draw_circle_lines(pr.x + 24.0 + i as f32 * 18.0, y - 6.0, 11.0, 2.0, hex(0xb45309)); }
    ui.text_semi(&crate::tf!("ui.mst.konto", c.mst.konto), pr.x + 34.0 + c.mst.konto.min(5) as f32 * 18.0, y, 18.0, Theme::text());
    y += 28.0;
    ui.text(&crate::tf!("ui.mst.saison", c.mst.saison_sp, crate::regeln::f("skilltree", "manager.saison_max") as u8), pr.x + 14.0, y, 13.0, Theme::text2());
    y += 22.0;
    let lz = crate::inhalt::t(&format!("lb.lizenz.{}", c.lizenz()));
    y = ui.paragraph(&crate::tf!("ui.mst.lizenz", lz, max), pr.x + 14.0, y, 13.0, pr.w - 28.0, Theme::achtung()) + 14.0;
    match gewaehlt.and_then(|g| alle.iter().find(|k| k.id == g).cloned()) {
        Some(k) => {
            ui.text_semi(&t(&format!("ui.mst.art.{}", k.art)).to_uppercase(), pr.x + 14.0, y, 12.0, Theme::gedaempft());
            let name = k.wert.map(|i| crate::rest3::werte_de_t()[i].to_string()).unwrap_or_else(|| k.name.clone());
            ui.title(&ui.fit(&name, 22.0, pr.w - 28.0), pr.x + 14.0, y + 28.0, 22.0, Theme::text());
            y += 46.0;
            if let Some(i) = k.wert { ui.text(&format!("{:.0} → {:.0}", c.mgr.werte[i], (c.mgr.werte[i] + crate::regeln::f("skilltree", "manager.wert_plus")).min(crate::regeln::f("skilltree", "manager.wert_max"))), pr.x + 14.0, y, 15.0, Theme::text2()); y += 24.0; }
            match c.m_frei(&k) {
                Ok(kosten) => { if ui.button_gold(Rect::new(pr.x + 14.0, y, pr.w - 28.0, 40.0), &crate::tf!("ui.st.kaufen", kosten)) { match c.m_kaufen(&k) { Ok(x) => v.toast(&x, true), Err(e) => v.toast(&e, false) } } y += 52.0; }
                Err(g) => { y = ui.paragraph(&g, pr.x + 14.0, y + 10.0, 14.0, pr.w - 28.0, Theme::achtung()) + 8.0; }
            }
        }
        None => { y = ui.paragraph(t("ui.mst.hinweis"), pr.x + 14.0, y, 14.0, pr.w - 28.0, Theme::text2()) + 8.0; }
    }
    ui.text_semi(&t("ui.st.verlauf").to_uppercase(), pr.x + 14.0, y + 10.0, 12.0, Theme::gedaempft());
    y += 30.0;
    for (_, txt, d) in c.mst.log.iter().rev() {
        if y > pr.y + pr.h - 16.0 { break; }
        for z in ui.wrap(txt, 12.0, pr.w - 28.0).into_iter().take(2) { ui.text(&z, pr.x + 14.0, y, 12.0, if *d > 0 { hex(GOLD) } else { Theme::text2() }); y += 15.0; }
        y += 4.0;
    }
}
