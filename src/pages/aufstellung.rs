use super::*;

pub(super) fn aufstellung(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("gb2.reiter.taktik"), crate::inhalt::t("ui.pages.standards_und_vorlagen"), crate::inhalt::t("bericht.gegner"), crate::inhalt::t("ui.pages.rollen_und_anweisungen"), crate::inhalt::t("ui.t9.reiter")]);
    if v.tab == 4 { fl27_taktik_seite(ui, c); return; }
    if v.tab == 3 { rollen_seite(ui, v, c); return; }
    if v.tab == 1 { standards_seite(ui, v, c); return; }
    if v.tab == 2 { gegner_seite(ui, v, c); return; }
    // left: Meine Taktik
    let lt = Rect::new(x0() + 4.0, Y0, 280.0, hoehe() - Y0 - 16.0);
    ui.card(lt);
    ui.title(crate::inhalt::t("tk.kompakt.meine"), lt.x + 14.0, lt.y + 30.0, 22.0, Theme::text());
    ui.text(crate::inhalt::t("ui.pages.gilt_fuer_alle_phasen"), lt.x + 14.0, lt.y + 50.0, 14.0, Theme::text2());
    let mut y = lt.y + 66.0;
    ui.text_semi(crate::inhalt::t("ui.wort.gross_formation"), lt.x + 14.0, y + 10.0, 14.0, Theme::text2());
    if let Some(f) = ui.segment(Rect::new(lt.x + 10.0, y + 18.0, lt.w - 20.0, 30.0), &FORMATIONS, c.formation) {
        c.formation = f;
        c.frei.clear();
        c.auto_lineup();
        c.world.clubs[c.own].fam = (c.world.clubs[c.own].fam - 2.5).max(0.0);
    }
    y += 58.0;
    // with the ball and against it (Fabian 24.11., like The Gaffer's three formations): "=" keeps the base formation
    for (k, titel) in [crate::inhalt::t("ui.taktik.mit_ball"), crate::inhalt::t("ui.taktik.gegen_ball")].iter().enumerate() {
        let wert = if k == 0 { c.formation_ball } else { c.formation_gegen };
        ui.text(titel, lt.x + 14.0, y + 19.0, 13.0, Theme::text2());
        let name = wert.map(|f| FORMATIONS[f].to_string()).unwrap_or_else(|| crate::inhalt::t("ui.taktik.wie_grund").to_string());
        let n = FORMATIONS.len() + 1;
        let idx = wert.map_or(0, |f| f + 1);
        let neu = if ui.button(Rect::new(lt.x + lt.w * 0.45, y + 2.0, 28.0, 26.0), "<", false) { Some((idx + n - 1) % n) } else if ui.button(Rect::new(lt.x + lt.w - 42.0, y + 2.0, 28.0, 26.0), ">", false) { Some((idx + 1) % n) } else { None }; // roh
        ui.text_center(&ui.fit(&name, 13.0, lt.w * 0.4 - 70.0), lt.x + lt.w * 0.45 + 28.0 + (lt.w * 0.55 - 84.0) / 2.0, y + 19.0, 13.0, if wert.is_some() { Theme::akzent() } else { Theme::text() });
        if let Some(i) = neu { let f = if i == 0 { None } else { Some(i - 1) }; if k == 0 { c.formation_ball = f; } else { c.formation_gegen = f; } }
        y += 30.0;
    }
    y += 4.0;
    // the PES M tactic (taktik9.rs): a playing idea, nine sliders in football words, four switches; every change is
    // translated at once into the game's values (tab "How FL27 plays it")
    let mut neu = false;
    if !c.t9.aktiv { c.t9 = crate::taktik9::Taktik9::aus_tactic(&c.tactic); neu = true; }
    ui.text_semi(crate::inhalt::t("ui.t9.idee"), lt.x + 14.0, y + 10.0, 14.0, Theme::text2());
    let n_ideen = crate::taktik9::IDEEN.len();
    if ui.button(Rect::new(lt.x + 10.0, y + 18.0, 30.0, 28.0), "<", false) { c.t9 = crate::taktik9::Taktik9::aus_idee((c.t9.idee as usize + n_ideen - 1) % n_ideen); neu = true; } // roh
    if ui.button(Rect::new(lt.x + lt.w - 40.0, y + 18.0, 30.0, 28.0), ">", false) { c.t9 = crate::taktik9::Taktik9::aus_idee((c.t9.idee as usize + 1) % n_ideen); neu = true; } // roh
    let idee = crate::inhalt::t(&format!("ui.t9.idee.{}", crate::taktik9::IDEEN[c.t9.idee as usize % n_ideen]));
    let idee = if c.t9.angepasst { crate::tf!("ui.t9.angepasst", idee) } else { idee.to_string() };
    ui.text_center(&ui.fit(&idee, 15.0, lt.w - 100.0), lt.x + lt.w / 2.0, y + 37.0, 15.0, Theme::akzent());
    y += 54.0;
    for (k, name) in crate::taktik9::REGLER.iter().enumerate() {
        let val = c.t9.regler[k];
        ui.text(crate::inhalt::t(&format!("ui.t9.r.{name}")), lt.x + 14.0, y + 12.0, 13.0, Theme::text());
        ui.text_right(&ui.fit(crate::inhalt::t(&format!("ui.t9.r.{name}.{val}")), 12.0, lt.w * 0.5), lt.x + lt.w - 14.0, y + 12.0, 12.0, Theme::akzent());
        if let Some(nv) = ui.slider(Rect::new(lt.x + 14.0, y + 18.0, lt.w - 28.0, 12.0), val as i32, 1, 5) { c.t9.regler[k] = nv as u8; c.t9.angepasst = true; neu = true; }
        y += 29.0;
    }
    let sw = (lt.w - 34.0) / 2.0;
    for (k, name) in crate::taktik9::SCHALTER.iter().enumerate() {
        let br = Rect::new(lt.x + 14.0 + (k % 2) as f32 * (sw + 6.0), y + 4.0 + (k / 2) as f32 * 32.0, sw, 28.0);
        if ui.button(br, &ui.fit(crate::inhalt::t(&format!("ui.t9.s.{name}")), 12.0, sw - 8.0), c.t9.schalter[k]) { c.t9.schalter[k] = !c.t9.schalter[k]; c.t9.angepasst = true; neu = true; }
    }
    y += 74.0;
    if neu { c.taktik9_anwenden(); }
    let fam = c.club().fam;
    // how well the team knows the system (the playing idea itself is chosen above)
    ui.text(&crate::tf!("ui.pages.eingespielt", format!("{:.0}", fam)), lt.x + 14.0, y + 16.0, 14.0, Theme::text());
    ui.bar(lt.x + 14.0, y + 24.0, lt.w - 28.0, 6.0, fam / 100.0, Theme::akzent());
    // middle: pitch
    let mid_x = lt.x + lt.w + 14.0;
    let mw = 560.0;
    let xi = c.current_xi();
    let st = c.team_strength();
    ui.title(crate::inhalt::t("aufstellung.startelf"), mid_x, Y0 + 22.0, 22.0, Theme::text());
    ui.text_semi(&crate::tf!("ui.pages.staerke_2", format!("{:.0}", st)), mid_x + 90.0, Y0 + 22.0, 18.0, Theme::gut());
    let mut co = c.co_lineup;
    if ui.button(Rect::new(mid_x + mw - 210.0, Y0 + 2.0, 210.0, 30.0), if co { crate::inhalt::t("ui.pages.co_trainer_stellt_auf") } else { crate::inhalt::t("ui.pages.co_trainer_stellt_auf_2") }, co) { co = !co; c.co_lineup = co; if co { c.auto_lineup(); } }
    let pitch = Rect::new(mid_x, Y0 + 40.0, mw, hoehe() - Y0 - 126.0);
    // the co-trainer's criteria, freely combined (Fabian 23.11.): a click toggles one, the line-up follows at once
    {
        let cw = (mw - 6.0 * 6.0) / 7.0;
        let y = pitch.y + pitch.h + 52.0;
        for b in 0..7u8 {
            let an = c.aufstellung_kriterien & (1 << b) != 0;
            if ui.button(Rect::new(mid_x + b as f32 * (cw + 6.0), y, cw, 28.0), &ui.fit(crate::inhalt::t(&format!("ui.kriterium.{b}")), 13.0, cw - 8.0), an) {
                let neu = c.aufstellung_kriterien ^ (1 << b);
                c.aufstellung_kriterien = if neu == 0 { 1 } else { neu };
                c.auto_lineup();
            }
        }
    }
    rrect(pitch.x, pitch.y, pitch.w, pitch.h, 8.0, hex(0x1E5A35));
    ui.rahmen(pitch);
    for i in 0..8 { if i % 2 == 0 { draw_rectangle(pitch.x + 6.0, pitch.y + 6.0 + i as f32 * (pitch.h - 12.0) / 8.0, pitch.w - 12.0, (pitch.h - 12.0) / 8.0, hex(0x226440)); } }
    let lc = Color::new(1.0, 1.0, 1.0, 0.35);
    draw_rectangle_lines(pitch.x + 6.0, pitch.y + 6.0, pitch.w - 12.0, pitch.h - 12.0, 2.0, lc);
    draw_line(pitch.x + 6.0, pitch.y + pitch.h / 2.0, pitch.x + pitch.w - 6.0, pitch.y + pitch.h / 2.0, 2.0, lc);
    draw_circle_lines(pitch.x + pitch.w / 2.0, pitch.y + pitch.h / 2.0, 60.0, 2.0, lc);
    draw_rectangle_lines(pitch.x + pitch.w / 2.0 - 120.0, pitch.y + 6.0, 240.0, 80.0, 2.0, lc);
    draw_rectangle_lines(pitch.x + pitch.w / 2.0 - 120.0, pitch.y + pitch.h - 86.0, 240.0, 80.0, 2.0, lc);
    // which formation the pitch shows: kick-off (editable), with the ball, against the ball (read only)
    let ansicht = v.taktik_ansicht.min(2);
    let alt = match ansicht { 1 => c.formation_ball, 2 => c.formation_gegen, _ => None };
    let spots = alt.map(crate::world::formation_spots).unwrap_or_else(|| c.eigene_spots());
    let alt_slots = alt.map(crate::world::formation_slots);
    // the three formations (kick-off, with the ball, against the ball) to look at on the pitch
    if let Some(i) = ui.segment(Rect::new(pitch.x + 10.0, pitch.y + 8.0, 320.0, 26.0), &[crate::inhalt::t("ui.taktik.anstoss"), crate::inhalt::t("ui.taktik.mit_ball"), crate::inhalt::t("ui.taktik.gegen_ball")], v.taktik_ansicht as usize) { v.taktik_ansicht = i as u8; }
    let mut swap: Option<(usize, usize)> = None;
    // free placement (Fabian 22.11.): hold a card and drag it; a short click still picks and swaps
    let zug_ziel = |m: Vec2| (((m.x - pitch.x - 40.0) / (pitch.w - 80.0)).clamp(0.02, 0.98), ((pitch.y + pitch.h - 50.0 - m.y) / (pitch.h - 100.0)).clamp(0.12, 0.95));
    let gezogen = v.tafel_zug.filter(|(_, s)| ui.mouse.distance(*s) > 8.0);
    // drag and drop (Fabian 23.11.): a card dropped on another card swaps the two, on a bench row it swaps with that
    // player, anywhere else on the pitch it moves the spot
    let karten: Vec<Rect> = spots.iter().map(|&(sx, sy)| { let (cx, cy) = (pitch.x + 40.0 + sx * (pitch.w - 80.0), pitch.y + pitch.h - 50.0 - sy * (pitch.h - 100.0)); Rect::new(cx - 46.0, cy - 40.0, 92.0, 96.0) }).collect();
    let bank_rect = { let rx = mid_x + mw + 14.0; Rect::new(rx, Y0, breite() - 18.0 - rx, hoehe() - Y0 - 16.0) };
    let mut auf_bank: Option<usize> = None;
    if let Some((i, _)) = gezogen {
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(j) = karten.iter().position(|k| k.contains(ui.mouse)).filter(|&j| j != i) { swap = Some((i, j)); }
            else if bank_rect.contains(ui.mouse) { auf_bank = Some(i); }
            else if i > 0 {
                let mut f: Vec<(f32, f32)> = spots.to_vec();
                f[i] = zug_ziel(ui.mouse);
                c.frei = f;
                c.co_lineup = false;
            }
            v.tafel_zug = None;
            v.pick = None;
        }
    } else if is_mouse_button_released(MouseButton::Left) { v.tafel_zug = None; }
    for (i, (slot, p, e)) in xi.iter().enumerate() {
        let (sx, sy) = spots[i];
        let mut cx = pitch.x + 40.0 + sx * (pitch.w - 80.0);
        let mut cy = pitch.y + pitch.h - 50.0 - sy * (pitch.h - 100.0);
        if gezogen.map_or(false, |g| g.0 == i) && i > 0 { cx = ui.mouse.x; cy = ui.mouse.y; }
        if ansicht == 0 && is_mouse_button_pressed(MouseButton::Left) && Rect::new(cx - 46.0, cy - 40.0, 92.0, 96.0).contains(ui.mouse) { v.tafel_zug = Some((i, ui.mouse)); }
        let r = Rect::new(cx - 46.0, cy - 40.0, 92.0, 96.0);
        let pl = &c.world.players[*p];
        let sel = v.pick == Some(i);
        rrect(r.x, r.y, r.w, r.h, 6.0, if sel { alpha(Theme::akzent(), 0.35) } else { Color::new(0.07, 0.08, 0.09, 0.92) });
        if ui.hover(r) { rrect_line(r.x, r.y, r.w, r.h, 6.0, Theme::akzent()); }
        rrect(r.x + 4.0, r.y + 4.0, 22.0, 18.0, 3.0, Theme::akzent());
        ui.text_center(&format!("{}", pl.no), r.x + 15.0, r.y + 18.0, 14.0, hex(0x10141c));
        rrect(r.x + r.w - 30.0, r.y + 4.0, 26.0, 18.0, 3.0, Theme::stufe(*e));
        ui.text_center(&format!("{:.0}", e), r.x + r.w - 17.0, r.y + 18.0, 14.0, hex(0x10141c));
        ui.player_photo(cx, r.y + 32.0, 14.0, pl.id, &pl.name);
        draw_circle_lines(cx, r.y + 32.0, 14.0, 2.0, Theme::akzent());
        let last = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
        let nm = ui.fit(&last, 15.0, 86.0);
        ui.text_center(&nm, cx, r.y + 60.0, 15.0, hex(0xffffff));
        let slot = alt_slots.map(|s| s[i]).unwrap_or(slot);
        let tag = pos_de(slot);
        ui.text_center(tag, cx, r.y + 74.0, 12.0, if pos_penalty(&pl.pos, slot) > 0.0 { Theme::achtung() } else { Theme::teil(slot) });
        // fitness, morale and form of the last five (Fabian 23.11.)
        ui.text(&format!("{:.0}%", pl.fit), r.x + 6.0, r.y + 89.0, 11.0, if pl.fit > 75.0 { Theme::gut() } else if pl.fit > 60.0 { Theme::achtung() } else { Theme::problem() });
        ui.smiley(cx, r.y + 85.0, 6.0, pl.morale);
        if let Some(f) = form5(pl) { ui.text_right(&format!("{f:.1}"), r.x + r.w - 6.0, r.y + 89.0, 11.0, Theme::note(f)); }
        draw_rectangle(r.x + 8.0, r.y + r.h - 4.0, (r.w - 16.0) * pl.fit / 100.0, 2.0, if pl.fit > 75.0 { Theme::gut() } else { Theme::achtung() });
        if gezogen.is_none() && ui.hit(r) {
            if let Some(a) = v.pick { if a != i { swap = Some((a, i)); } v.pick = None; } else { v.pick = Some(i); }
        }
    }
    if c.frei.len() == 11 && ui.button(Rect::new(pitch.x + pitch.w - 200.0, pitch.y + 10.0, 190.0, 28.0), crate::inhalt::t("ui.taktik.frei_zurueck"), false) { c.frei.clear(); c.auto_lineup(); }
    ui.text(&crate::tf!("ui.pages.abwehrlinie_kompaktheit_pressing", c.tactic.linie, c.tactic.kompakt, if c.tactic.breite == 0 { crate::inhalt::t("ui.wort.breit") } else { crate::inhalt::t("ui.wort.eng") }, if c.tactic.pressing == 0 { crate::inhalt::t("ui.wort.hart") } else { crate::inhalt::t("ui.wort.ruhig") }), mid_x + 8.0, pitch.y + pitch.h + 22.0, 15.0, Theme::text2());
    ui.text(crate::inhalt::t("ui.taktik.frei_hinweis"), mid_x + 8.0, pitch.y + pitch.h + 42.0, 14.0, Theme::gedaempft());
    let mut cur: Vec<usize> = xi.iter().map(|x| x.1).collect();
    if let Some((a, b)) = swap { cur.swap(a, b); c.xi = cur.clone(); c.co_lineup = false; }
    // a bench player dropped on a card comes in for that player
    let bank_gezogen = v.bank_zug.filter(|(_, s)| ui.mouse.distance(*s) > 8.0);
    if is_mouse_button_released(MouseButton::Left) {
        if let Some((p, _)) = bank_gezogen {
            if let Some(j) = karten.iter().position(|k| k.contains(ui.mouse)) { if j < cur.len() && c.world.players[p].available() { cur[j] = p; c.xi = cur.clone(); c.co_lineup = false; } }
        }
    }
    // right: bench and co-trainer
    let rx = mid_x + mw + 14.0;
    let rw = breite() - 18.0 - rx;
    let rr = Rect::new(rx, Y0, rw, hoehe() - Y0 - 16.0);
    ui.card(rr);
    ui.title(crate::inhalt::t("ui.pages.bank_und_reserve"), rr.x + 12.0, rr.y + 28.0, 20.0, Theme::text());
    // bench and reserve apart, each by keeper, defence, midfield and attack, as cards like on the pitch (Fabian 24.11.):
    // the bench is the best available keeper and the best others up to the matchday bench size, the reserve the rest
    let others: Vec<usize> = c.squad().into_iter().filter(|p| !cur.contains(p)).collect();
    let bank_n = crate::regeln::f_oder("kaderregeln", "spieltag.bank", 9.0) as usize;
    let mut frei: Vec<usize> = others.iter().copied().filter(|&p| c.world.players[p].available()).collect();
    frei.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
    let mut bank: Vec<usize> = frei.iter().copied().filter(|&p| c.world.players[p].pos == "GK").take(1).collect(); // roh
    for &p in &frei { if bank.len() >= bank_n { break; } if !bank.contains(&p) { bank.push(p); } }
    let reserve: Vec<usize> = others.iter().copied().filter(|p| !bank.contains(p)).collect();
    let ordnen = |v: &mut Vec<usize>| v.sort_by(|a, b| { let (p, q) = (&c.world.players[*a], &c.world.players[*b]); crate::world::part_of(&p.pos).cmp(&crate::world::part_of(&q.pos)).then(q.ovr.partial_cmp(&p.ovr).unwrap()) });
    let (mut bank, mut reserve) = (bank, reserve);
    ordnen(&mut bank); ordnen(&mut reserve);
    // the layout as rows: a section title, a line label, or a row of cards
    enum Zeile { Titel(String), Teil(usize), Karten(Vec<usize>) }
    let (kw, kh) = (84.0f32, 88.0f32);
    let spalten = (((rr.w - 30.0) / (kw + 6.0)) as usize).max(1);
    let mut zeilen: Vec<Zeile> = vec![];
    for (titel, liste) in [(crate::tf!("ui.taktik.bank", bank.len()), &bank), (crate::tf!("ui.taktik.reserve", reserve.len()), &reserve)] {
        zeilen.push(Zeile::Titel(titel));
        for teil in 0..4usize {
            let g: Vec<usize> = liste.iter().copied().filter(|&p| crate::world::part_of(&c.world.players[p].pos) == teil).collect();
            if g.is_empty() { continue; }
            zeilen.push(Zeile::Teil(teil));
            for ch in g.chunks(spalten) { zeilen.push(Zeile::Karten(ch.to_vec())); }
        }
    }
    let hoehe_von = |z: &Zeile| match z { Zeile::Titel(_) => 30.0, Zeile::Teil(_) => 20.0, Zeile::Karten(_) => kh + 6.0 };
    let gesamt: f32 = zeilen.iter().map(hoehe_von).sum();
    let liste = Rect::new(rr.x, rr.y + 44.0, rr.w, rr.h - 230.0 - 50.0);
    let start = scroll_schritt(ui, v, liste, (gesamt / 10.0).ceil() as usize, (liste.h / 10.0) as usize, 6.0) as f32 * 10.0;
    let mut yy = liste.y - start;
    let teile = [crate::inhalt::t("ui.taktik.teil.0"), crate::inhalt::t("ui.taktik.teil.1"), crate::inhalt::t("ui.taktik.teil.2"), crate::inhalt::t("ui.taktik.teil.3")];
    for z in &zeilen {
        let h = hoehe_von(z);
        let sichtbar = yy >= liste.y - 2.0 && yy + h <= liste.y + liste.h + 2.0;
        match z {
            Zeile::Titel(t) if sichtbar => { ui.text_semi(t, rr.x + 12.0, yy + 20.0, 15.0, Theme::akzent()); }
            Zeile::Teil(k) if sichtbar => { let col = Theme::teil(crate::pesdb::POS[[0usize, 1, 5, 12][*k]]); draw_rectangle(rr.x + 12.0, yy + 6.0, 4.0, 12.0, col); ui.text(teile[*k], rr.x + 22.0, yy + 16.0, 12.0, col); }
            Zeile::Karten(ps) if sichtbar => {
                for (k, &p) in ps.iter().enumerate() {
                    let pl = &c.world.players[p];
                    let r = Rect::new(rr.x + 12.0 + k as f32 * (kw + 6.0), yy, kw, kh);
                    let cx = r.x + r.w / 2.0;
                    rrect(r.x, r.y, r.w, r.h, 6.0, Color::new(0.07, 0.08, 0.09, 0.92));
                    if ui.hover(r) { rrect_line(r.x, r.y, r.w, r.h, 6.0, Theme::akzent()); }
                    rrect(r.x + r.w - 28.0, r.y + 4.0, 24.0, 16.0, 3.0, Theme::stufe(pl.ovr));
                    ui.text_center(&format!("{:.0}", pl.ovr), r.x + r.w - 16.0, r.y + 17.0, 13.0, hex(0x10141c));
                    ui.player_photo(cx, r.y + 28.0, 13.0, pl.id, &pl.name);
                    draw_circle_lines(cx, r.y + 28.0, 13.0, 2.0, Theme::teil(&pl.pos));
                    let last = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
                    ui.text_center(&ui.fit(&last, 13.0, kw - 6.0), cx, r.y + 55.0, 13.0, if pl.available() { hex(0xffffff) } else { Theme::problem() });
                    ui.text_center(pos_de(&pl.pos), cx, r.y + 68.0, 11.0, Theme::teil(&pl.pos));
                    ui.text(&format!("{:.0}%", pl.fit), r.x + 5.0, r.y + 82.0, 10.0, if pl.fit > 75.0 { Theme::gut() } else if pl.fit > 60.0 { Theme::achtung() } else { Theme::problem() });
                    ui.smiley(cx, r.y + 78.0, 5.0, pl.morale);
                    if let Some(f) = form5(pl) { ui.text_right(&format!("{f:.1}"), r.x + r.w - 5.0, r.y + 82.0, 10.0, Theme::note(f)); }
                    if let Some(i) = auf_bank.filter(|_| r.contains(ui.mouse)) { if i < cur.len() && pl.available() { cur[i] = p; c.xi = cur.clone(); c.co_lineup = false; } }
                    if ui.hit(r) { v.bank_zug = Some((p, ui.mouse)); }
                    // a click without moving: as before (swap with the picked card, else the profile)
                    if is_mouse_button_released(MouseButton::Left) && bank_gezogen.is_none() && v.bank_zug.map_or(false, |b| b.0 == p) && r.contains(ui.mouse) {
                        if let Some(a) = v.pick { if a < cur.len() { cur[a] = p; c.xi = cur.clone(); c.co_lineup = false; } v.pick = None; } else { v.modal = Modal::Player(p); }
                    }
                }
            }
            _ => {}
        }
        yy += h;
    }
    if is_mouse_button_released(MouseButton::Left) { v.bank_zug = None; }
    // the dragged bench player follows the mouse
    if let Some((p, _)) = bank_gezogen {
        let pl = &c.world.players[p];
        let g = Rect::new(ui.mouse.x - 70.0, ui.mouse.y - 16.0, 140.0, 32.0);
        rrect(g.x, g.y, g.w, g.h, 6.0, alpha(Theme::akzent(), 0.9));
        ui.player_photo(g.x + 16.0, g.y + 16.0, 11.0, pl.id, &pl.name);
        let nm = ui.fit(pl.name.split_whitespace().last().unwrap_or(&pl.name), 14.0, g.w - 40.0);
        ui.text(&nm, g.x + 32.0, g.y + 21.0, 14.0, Theme::auf_akzent());
    }
    let cy = rr.y + rr.h - 230.0;
    draw_rectangle(rr.x + 10.0, cy, rr.w - 20.0, 1.0, Theme::kontur());
    ui.text_semi(&crate::tf!("ui.pages.co_trainer_rat", c.co_name), rr.x + 12.0, cy + 24.0, 14.0, Theme::akzent());
    let op = c.co_opinion();
    let mut ty = cy + 46.0;
    for l in ui.wrap(&op, 15.0, rr.w - 24.0).into_iter().take(8) { ui.text(&l, rr.x + 12.0, ty, 15.0, Theme::text()); ty += 19.0; }
    if ui.button(Rect::new(rr.x + 12.0, rr.y + rr.h - 44.0, rr.w - 24.0, 34.0), crate::inhalt::t("ui.pages.vorschlag_uebernehmen"), false) { c.auto_lineup(); v.toast(crate::inhalt::t("ui.pages.aufstellung_vom_co_trainer_uebernommen"), true); }
}

fn standards_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let r = Rect::new(x0() + 4.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0);
    ui.card_titled(r, crate::inhalt::t("ui.pages.standards_und_kapitaen"), Theme::akzent());
    for (k, n) in [crate::inhalt::t("gb.schluessel.standard_elfmeter"), crate::inhalt::t("gb.schluessel.standard_freistoss_lang"), crate::inhalt::t("beob.stat.ecken"), "Kapitän"].iter().enumerate() { // roh
        let yy = r.y + 70.0 + k as f32 * 56.0;
        ui.text_semi(n, r.x + 14.0, yy, 18.0, Theme::text());
        let wer = c.schuetze(k as u8).map(|p| c.world.players[p].name.clone()).unwrap_or("–".into());
        ui.text(&wer, r.x + 150.0, yy, 18.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 140.0, yy - 20.0, 126.0, 30.0), crate::inhalt::t("ui.pages.wechseln"), false) { c.schuetze_waehlen(k as u8); }
    }
    ui.paragraph(crate::inhalt::t("ui.pages.ohne_eigene_wahl_uebernimmt_der"), r.x + 14.0, r.y + 320.0, 15.0, r.w - 28.0, Theme::gedaempft());
    let r2 = Rect::new(x0() + cbreite() * 0.5 + 10.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0);
    ui.card_titled(r2, crate::inhalt::t("ui.wort.taktik_vorlagen"), Theme::gold());
    // Plan B (Taktik-Konzept 1.7): from a minute on, when the score says so, the mentality changes by itself
    let (r, py) = (r2, r2.y + 112.0 + c.std.vorlagen.len() as f32 * 46.0);
    {
        ui.text_semi(crate::inhalt::t("ui.planb.titel"), r.x + 14.0, py, 18.0, Theme::akzent());
        let an = c.plan_b.an;
        if ui.button(Rect::new(r.x + r.w - 140.0, py - 20.0, 126.0, 30.0), crate::inhalt::t(if an { "ui.planb.an" } else { "ui.planb.aus" }), an) { c.plan_b.an = !an; }
        if let Some(s) = ui.segment(Rect::new(r.x + 14.0, py + 14.0, r.w - 28.0, 28.0), &[crate::inhalt::t("ui.planb.wenn.0"), crate::inhalt::t("ui.planb.wenn.1"), crate::inhalt::t("ui.planb.wenn.2")], c.plan_b.wenn as usize) { c.plan_b.wenn = s as u8; }
        if let Some(s) = ui.segment(Rect::new(r.x + 14.0, py + 50.0, r.w - 28.0, 28.0), &[crate::inhalt::t("ui.live.ment.0"), crate::inhalt::t("ui.live.ment.1"), crate::inhalt::t("ui.live.ment.2")], (c.plan_b.mentalitaet + 1) as usize) { c.plan_b.mentalitaet = s as i8 - 1; }
        ui.text(&crate::tf!("ui.planb.ab", c.plan_b.minute), r.x + 14.0, py + 104.0, 15.0, Theme::text());
        if let Some(m) = ui.slider(Rect::new(r.x + 140.0, py + 92.0, r.w - 170.0, 16.0), c.plan_b.minute as i32, 46, 85) { c.plan_b.minute = m as u8; }
    }
    if ui.button(Rect::new(r2.x + 14.0, r2.y + 50.0, 260.0, 32.0), crate::inhalt::t("ui.pages.aktuelle_taktik_ablegen"), false) { match c.vorlage_speichern() { Ok(()) => v.toast(crate::inhalt::t("ui.pages.abgelegt"), true), Err(e) => v.toast(&e, false) } }
    let mut del = None;
    for (i, (n, _, _)) in c.std.vorlagen.clone().iter().enumerate() {
        let yy = r2.y + 110.0 + i as f32 * 46.0;
        ui.text(n, r2.x + 14.0, yy, 17.0, Theme::text());
        if ui.button(Rect::new(r2.x + r2.w - 250.0, yy - 20.0, 110.0, 30.0), crate::inhalt::t("ui.pages.laden"), false) { c.vorlage_laden(i); v.toast(crate::inhalt::t("ui.pages.taktik_geladen"), true); }
        if ui.button(Rect::new(r2.x + r2.w - 130.0, yy - 20.0, 116.0, 30.0), crate::inhalt::t("md.ui.mail.loeschen"), false) { del = Some(i); }
    }
    if let Some(i) = del { c.std.vorlagen.remove(i); }
    let yy = (r2.y + 360.0).max(r2.y + 112.0 + c.std.vorlagen.len() as f32 * 46.0 + 130.0);
    ui.text_semi(crate::inhalt::t("ui.pages.urteil_des_co_trainers"), r2.x + 14.0, yy, 16.0, Theme::akzent());
    for (k, l) in c.taktikurteil().iter().enumerate().take(4) { ui.paragraph(l, r2.x + 14.0, yy + 26.0 + k as f32 * 44.0, 15.0, r2.w - 28.0, Theme::text2()); }
}
/// "How FL27 plays it" (Konzept 2.3): every game value with its setting, the reason, and whether PES M writes it into
/// the game's database (Tactics.bin) or only the simulation knows it.
fn fl27_taktik_seite(ui: &mut Ui, c: &mut Career) {
    let t = crate::inhalt::t;
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(r, t("ui.t9.reiter"), Theme::akzent());
    ui.paragraph(t("ui.t9.erklaerung"), r.x + 14.0, r.y + 60.0, 14.0, r.w - 28.0, Theme::text2());
    let mut tac = c.tactic.clone();
    let zeilen = crate::taktik9::uebersetzen(&c.t9, &mut tac, c.zielspieler_in_elf());
    let (x1, x2, x3) = (r.x + 14.0, r.x + r.w * 0.24, r.x + r.w * 0.42);
    let mut y = r.y + 110.0;
    for (k, kopf) in [t("ui.t9.k.wert"), t("ui.t9.k.einstellung"), t("ui.t9.k.grund")].iter().enumerate() { ui.text_semi(kopf, [x1, x2, x3][k], y, 13.0, Theme::gedaempft()); }
    ui.text_semi(t("ui.t9.k.stand"), r.x + r.w - 220.0, y, 13.0, Theme::gedaempft());
    y += 14.0;
    for z in &zeilen {
        y += 34.0;
        if y > r.y + r.h - 160.0 { break; }
        rrect(r.x + 8.0, y - 22.0, r.w - 16.0, 30.0, 6.0, Theme::erhoehung());
        ui.text(t(&format!("ui.t9.n.{}", z.name)), x1, y - 2.0, 15.0, Theme::text());
        ui.text_semi(&z.wert, x2, y - 2.0, 15.0, Theme::akzent());
        ui.text(&ui.fit(&z.grund, 14.0, r.w * 0.4), x3, y - 2.0, 14.0, Theme::text2());
        // written into the game, or only in PES M's simulation until the research finds the field
        let (col, txt) = if z.geschrieben { (Theme::gut(), t("ui.t9.stand.geschrieben")) } else { (Theme::achtung(), t("ui.t9.stand.simulation")) };
        draw_circle(r.x + r.w - 214.0, y - 7.0, 6.0, col);
        ui.text(txt, r.x + r.w - 200.0, y - 2.0, 13.0, Theme::text2());
    }
    // what only PES M knows
    let nur: Vec<&str> = crate::taktik9::SCHALTER.iter().enumerate().filter(|(k, _)| c.t9.schalter[*k] && *k >= 2).map(|(_, n)| crate::inhalt::t(&format!("ui.t9.s.{n}"))).collect();
    if !nur.is_empty() { ui.text(&crate::tf!("ui.t9.nur_simulation", nur.join(", ")), r.x + 14.0, r.y + r.h - 24.0, 13.0, Theme::text2()); }
    // the advanced instructions, two in attack and two in defence (the game's list, as The Gaffer has it); they act in
    // PES M's simulation, the place in FL27's database is still unknown
    let ay = r.y + r.h - 150.0;
    ui.text_semi(t("ui.t9.anweisungen"), r.x + 14.0, ay, 14.0, Theme::akzent());
    let bw = (r.w - 28.0 - 3.0 * 12.0) / 4.0;
    for k in 0..4 {
        let liste: Vec<(u8, &str)> = if k < 2 { crate::taktik9::ANWEISUNGEN_ANGRIFF.to_vec() } else { crate::taktik9::ANWEISUNGEN_ABWEHR.to_vec() };
        let x = r.x + 14.0 + k as f32 * (bw + 12.0);
        ui.text(&crate::tf!(if k < 2 { "ui.t9.anw_angriff" } else { "ui.t9.anw_abwehr" }, k % 2 + 1), x, ay + 26.0, 12.0, Theme::gedaempft());
        let id = c.anweisungen[k];
        let pos = liste.iter().position(|x| x.0 == id).map_or(0, |i| i + 1);
        let name = if pos == 0 { t("ui.t9.anw_keine").to_string() } else { t(&format!("ui.t9.anw.{}", liste[pos - 1].1)).to_string() };
        let n = liste.len() + 1;
        let neu = if ui.button(Rect::new(x, ay + 36.0, 28.0, 30.0), "<", false) { Some((pos + n - 1) % n) } else if ui.button(Rect::new(x + bw - 28.0, ay + 36.0, 28.0, 30.0), ">", false) { Some((pos + 1) % n) } else { None }; // roh
        ui.text_center(&ui.fit(&name, 13.0, bw - 66.0), x + bw / 2.0, ay + 56.0, 13.0, if pos > 0 { Theme::akzent() } else { Theme::text2() });
        if let Some(i) = neu {
            let wahl = if i == 0 { 0 } else { liste[i - 1].0 };
            // the same instruction twice makes no sense: the other slot is cleared
            if wahl != 0 { for j in 0..4 { if j != k && c.anweisungen[j] == wahl { c.anweisungen[j] = 0; } } }
            c.anweisungen[k] = wahl;
        }
    }
    ui.text(t("ui.t9.anw_hinweis"), r.x + 14.0, ay + 92.0, 12.0, Theme::gedaempft());
}
