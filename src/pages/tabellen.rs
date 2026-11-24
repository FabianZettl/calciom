use super::*;

/// Zone of a table place: 0 CL, 1 EL, 2 Conference League, 3 promotion, 4 play-offs, 5 relegation / playout, 6 relegated.
fn tabellen_zone(pl: usize, rows: usize, ab: usize, rel: usize, pout: &[usize], auf: usize, po: &[usize], euro: &[(usize, usize)]) -> Option<u8> {
    if ab > 0 && pl + ab > rows { return Some(6); }
    if pl == rel || pout.contains(&pl) { return Some(5); }
    if let Some(e) = euro.iter().find(|e| e.0 == pl) { return Some(e.1.min(2) as u8); }
    if pl <= auf { return Some(3); }
    if po.contains(&pl) { return Some(4); }
    None
}
fn zone_farbe(z: u8) -> Color {
    match z { 0 => hex(0x1E6BFF), 1 => hex(0xF28C28), 2 => hex(0x2EB872), 3 => hex(0x18A058), 4 => hex(0x7C5CFF), 5 => Theme::achtung(), _ => Theme::problem() }
}

// ================================================================ Tabellen
pub(super) fn tabellen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // no list on the left any more (Fabian 22.11.): country and competition are chosen in the banner, the page has the
    // whole width for table, scorers, form and news
    let gruppen = wettbewerb_gruppen(c);
    let l = v.table_league.unwrap_or(c.league());
    let x = x0() + 4.0;
    let tw = cbreite();
    let felder = wettbewerb_felder(c, &gruppen, l, Rect::new(x + tw - 480.0, Y0 - 40.0, 470.0, 36.0));
    if let Some((f, i)) = auswahl_vorher(ui, v, &felder) { v.table_league = Some(wahl_umsetzen(&gruppen, l, f, i)); v.league_scroll = 0; }
    let l = v.table_league.unwrap_or(c.league());
    if l >= 1000 && l - 1000 < c.world.cups.len() { cup_view(ui, c, l - 1000, x, tw); auswahl_zeichnen(ui, v, &felder); return; }
    let lg = &c.world.leagues[l];
    let lk = wettbewerb_look(c, &lg.key, 9, &lg.land);
    let lx = wettbewerb_banner(ui, Rect::new(x, Y0 - 44.0, tw, 46.0), &lk, c.world.cids.get(&lg.key).copied(), &lg.name);
    // zones from daten/ligaregeln.toml: up, play-offs, relegation tie, playout, down; Europe for a top league
    let regel = c.world.regel(l);
    let oben = c.world.ueber_index(l);
    let (auf, ab) = (if oben.is_some() { regel.auf } else { 0 }, regel.ab);
    let po: Vec<usize> = regel.playoffs.as_ref().map(|p| p.plaetze.clone()).unwrap_or_default();
    let rel = regel.relegation.as_ref().map_or(0, |r| r.platz);
    let pout: Vec<usize> = regel.playout.as_ref().map(|p| p.plaetze.clone()).unwrap_or_default();
    let euro: Vec<(usize, usize)> = c.world.uefa_ligen().iter().find(|x| x.0 == l).map(|(_, land)| land.europa.as_ref().unwrap().plaetze.iter().flat_map(|p| {
        let t = [crate::id("cl"), crate::id("el"), crate::id("uecl")].iter().position(|w| *w == p.wettbewerb).unwrap_or(2);
        p.plaetze.iter().filter_map(move |o| if let crate::ligaregeln::Ort::Platz(n) = o { Some((*n, t)) } else { None })
    }).collect()).unwrap_or_default();
    let hinweis = match lg.detail { 0 => crate::inhalt::t("ui.pages.deine_liga_jedes_spiel_mit"), 1 => crate::inhalt::t("ui.pages.detailliert_torschuetzen_und_verletzunge"), _ => crate::inhalt::t("ui.pages.nur_ergebnisse") };
    let hx = (x + 260.0).max(lx + ui.measure_title(&lg.name, 26.0) + 16.0);
    let hinweis = ui.fit(hinweis, 15.0, (x + tw - 500.0 - hx).max(60.0));
    ui.text(&hinweis, hx, Y0 - 14.0, 15.0, lk.text2);
    // estimated in daten/ligaregeln.toml (sicher = false) or no entry there: the standard rules
    if !regel.sicher && hx + ui.measure(&hinweis, 15.0) + 200.0 < x + tw - 490.0 { ui.text(crate::inhalt::t("ui.pages.regel_geschaetzt"), hx + ui.measure(&hinweis, 15.0), Y0 - 14.0, 15.0, Theme::achtung()); }
    let table = c.world.table(l);
    let rows = table.len();
    let area = Rect::new(x, Y0 + 6.0, tw * 0.64, hoehe() - Y0 - 22.0);
    ui.card(area);
    draw_rectangle(area.x + 10.0, area.y, area.w - 20.0, 3.0, lk.akzent_lesbar);
    player_row_head(ui, area.x, area.y + 22.0, &[("#", 10.0), (crate::inhalt::t("ui.wort.gross_verein"), 70.0), (crate::inhalt::t("ui.wort.gross_sp"), 330.0), (crate::inhalt::t("ui.wort.buchstabe_s"), 370.0), (crate::inhalt::t("ui.wort.buchstabe_u"), 400.0), (crate::inhalt::t("ui.wort.buchstabe_n"), 430.0), (crate::inhalt::t("ui.wort.gross_tore"), 470.0), (crate::inhalt::t("ui.wort.gross_pkt"), 540.0), (crate::inhalt::t("ui.wort.gross_form"), 590.0)]);
    // rows keep a readable height (26 to 32); a long table (league phase, 36 teams) scrolls instead of being squeezed
    let rh = ((area.h - 36.0) / rows as f32).clamp(26.0, 32.0);
    let vis = ((area.h - 36.0) / rh) as usize;
    let start = scroll_rows(ui, v, area, rows, vis);
    let mut zonen: Vec<u8> = vec![];
    for (i, r) in table.iter().enumerate().skip(start).take(vis) {
        let y = area.y + 30.0 + (i - start) as f32 * rh;
        let rr = Rect::new(area.x + 4.0, y, area.w - 8.0, rh - 2.0);
        if r.club == c.own { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::akzent(), 0.16)); }
        let pl = i + 1;
        // the zone of the place (Fabian 23.11.: plainly visible who goes to Europe, up, into the play-offs or down)
        let zone = tabellen_zone(pl, rows, ab, rel, &pout, auf, &po, &euro);
        if let Some(z) = zone {
            let col = zone_farbe(z);
            if !zonen.contains(&z) { zonen.push(z); }
            if r.club != c.own { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(col, 0.09)); }
            draw_rectangle(rr.x, rr.y + 1.0, 5.0, rr.h - 2.0, col);
        }
        // a line between two zones
        if i + 1 < rows && zone != tabellen_zone(pl + 1, rows, ab, rel, &pout, auf, &po, &euro) && i + 1 < start + vis { draw_rectangle(rr.x, rr.y + rh - 2.0, rr.w, 1.0, alpha(Theme::text2(), 0.35)); }
        let ty = y + rh * 0.7;
        let fs = (rh * 0.55).min(17.0);
        ui.text(&format!("{}", i + 1), area.x + 12.0, ty, fs, Theme::text2());
        let (c1, _) = club_color(c, r.club);
        let k = &c.world.clubs[r.club];
        ui.club_crest(area.x + 52.0, y + rh / 2.0 - 1.0, rh * 0.4, k.id, c1, &k.short);
        let sperre = oben.map_or(false, |o| !c.world.darf_aufsteigen(r.club, o));
        let nm = ui.fit(&c.world.clubs[r.club].name, fs, if sperre { 170.0 } else { 250.0 });
        ui.text_semi(&nm, area.x + 70.0, ty, fs, Theme::text());
        if sperre { let w = ui.measure_semi(&nm, fs); ui.text(crate::inhalt::t("ui.pages.kein_aufstieg"), area.x + 78.0 + w, ty, 12.0, Theme::text2()); }
        for (val, cx) in [(r.p as i32, 330.0), (r.w as i32, 370.0), (r.d as i32, 400.0), (r.l as i32, 430.0)] { ui.text(&format!("{val}"), area.x + cx, ty, fs, Theme::text()); }
        ui.text(&format!("{}:{}", r.gf, r.ga), area.x + 470.0, ty, fs, Theme::text());
        ui.text_semi(&format!("{}", r.pts), area.x + 540.0, ty, fs, Theme::text());
        for (j, ch) in r.form.iter().enumerate() { let col = match ch { 'S' => Theme::gut(), 'N' => Theme::problem(), _ => Theme::achtung() }; rrect(area.x + 590.0 + j as f32 * 14.0, y + rh * 0.25, 10.0, rh * 0.45, 2.0, col); }
        if ui.hit(rr) && r.club != c.own { /* club profile: not in this version */ }
    }
    // legend of the zones shown, under the table
    zonen.sort();
    let mut lx = area.x + 12.0;
    for z in zonen {
        let col = zone_farbe(z);
        let t = crate::inhalt::t(&format!("ui.tabelle.zone.{z}"));
        draw_rectangle(lx, area.y + area.h - 18.0, 12.0, 10.0, col);
        ui.text(t, lx + 16.0, area.y + area.h - 8.0, 13.0, Theme::text2());
        lx += ui.measure(t, 13.0) + 34.0;
    }
    // right: scorers and scorer points side by side, the form table and the league's news (Fabian 22.11.)
    let rx = x + tw * 0.64 + 12.0;
    let rw = tw * 0.36 - 12.0;
    let hoch = hoehe() - Y0 - 22.0;
    // only goals and assists of this league (Fabian 23.11.)
    let spieler: Vec<usize> = (0..c.world.players.len()).filter(|&p| c.world.players[p].profil.ligen.iter().any(|x| x.l as usize == l)).collect();
    let r1 = Rect::new(rx, Y0 + 6.0, rw, hoch * 0.42);
    ui.card_titled(r1, crate::inhalt::t("statistik.torjaeger_hinweis"), lk.c1);
    let halb = (r1.w - 36.0) / 2.0;
    for (seite, titel) in [crate::inhalt::t("ui.liga.tore"), crate::inhalt::t("ui.liga.scorer")].iter().enumerate() {
        let sx = r1.x + 14.0 + seite as f32 * (halb + 8.0);
        ui.text_semi(&titel.to_uppercase(), sx, r1.y + 56.0, 12.0, Theme::gedaempft());
        let wert = |p: usize| { let x = c.world.players[p].in_liga(l); if seite == 0 { x.goals as u32 } else { (x.goals + x.assists) as u32 } };
        let mut l2: Vec<usize> = spieler.iter().copied().filter(|&p| wert(p) > 0).collect();
        l2.sort_by_key(|&p| std::cmp::Reverse(wert(p)));
        if l2.is_empty() { ui.text(crate::inhalt::t("statistik.keine_tore"), sx, r1.y + 80.0, 13.0, Theme::text2()); }
        let n = ((r1.h - 76.0) / 24.0) as usize;
        for (j, &p) in l2.iter().take(n).enumerate() {
            let pl = &c.world.players[p];
            let yy = r1.y + 80.0 + j as f32 * 24.0;
            if pl.team == c.own as i32 { rrect(sx - 4.0, yy - 16.0, halb + 8.0, 22.0, 5.0, alpha(Theme::akzent(), 0.16)); }
            ui.player_photo(sx + 9.0, yy - 5.0, 9.0, pl.id, &pl.name);
            let cl = &c.world.clubs[pl.team.max(0) as usize];
            ui.club_crest(sx + 26.0, yy - 5.0, 6.0, cl.id, hex(cl.c1), &cl.short);
            ui.text(&ui.fit(&pl.name, 13.0, halb - 70.0), sx + 36.0, yy, 13.0, if pl.team == c.own as i32 { Theme::gold() } else { Theme::text() });
            ui.text_right(&format!("{}", wert(p)), sx + halb, yy, 14.0, lk.akzent_lesbar);
        }
    }
    // the form table: points from the last five matches
    let r2 = Rect::new(rx, r1.y + r1.h + 10.0, rw, hoch * 0.3);
    ui.card_titled(r2, crate::inhalt::t("ui.liga.form"), lk.c1);
    let mut form: Vec<(usize, i32, Vec<char>)> = table.iter().map(|r| (r.club, r.form.iter().map(|ch| match ch { 'S' => 3, 'U' => 1, _ => 0 }).sum(), r.form.clone())).collect();
    form.sort_by_key(|x| std::cmp::Reverse(x.1));
    let n = ((r2.h - 56.0) / 24.0) as usize;
    for (j, (k, pkt, f)) in form.iter().take(n).enumerate() {
        let yy = r2.y + 60.0 + j as f32 * 24.0;
        let cl = &c.world.clubs[*k];
        ui.club_crest(r2.x + 22.0, yy - 5.0, 8.0, cl.id, hex(cl.c1), &cl.short);
        ui.text(&ui.fit(&cl.name, 13.0, r2.w - 190.0), r2.x + 36.0, yy, 13.0, if *k == c.own { Theme::gold() } else { Theme::text() });
        for (q, ch) in f.iter().enumerate() { let col = match ch { 'S' => Theme::gut(), 'N' => Theme::problem(), _ => Theme::achtung() }; rrect(r2.x + r2.w - 120.0 + q as f32 * 14.0, yy - 11.0, 10.0, 12.0, 2.0, col); }
        ui.text_right(&format!("{pkt}"), r2.x + r2.w - 14.0, yy, 14.0, Theme::text2());
    }
    // news of this league: the newspaper items naming one of its clubs, newest first
    let r3 = Rect::new(rx, r2.y + r2.h + 10.0, rw, Y0 + 6.0 + hoch - (r2.y + r2.h + 10.0));
    ui.card_titled(r3, crate::inhalt::t("ui.liga.news"), lk.c1);
    let namen: Vec<String> = lg.teams.iter().map(|&t| c.world.clubs[t].name.clone()).collect();
    let mut yy = r3.y + 56.0;
    let mut gezeigt = 0;
    for nw in c.feed.iter().filter(|nw| namen.iter().any(|n| nw.head.contains(n.as_str()) || nw.text.contains(n.as_str()))) {
        if yy > r3.y + r3.h - 30.0 { break; }
        ui.text(&date::short(nw.season, nw.day), r3.x + 14.0, yy, 12.0, Theme::gedaempft());
        yy = ui.paragraph(&ui.fit(&nw.head, 13.0, (r3.w - 28.0) * 2.0), r3.x + 14.0, yy + 16.0, 13.0, r3.w - 28.0, Theme::text()) + 8.0;
        gezeigt += 1;
    }
    // the league's own news when the papers say little: the leader and the last matchday's results
    if gezeigt < 3 && yy < r3.y + r3.h - 40.0 {
        if let Some(erster) = c.world.table(l).first().filter(|r| r.p > 0) {
            ui.text_passend(&crate::tf!("ui.liga.fuehrer", c.world.clubs[erster.club].name.clone(), erster.pts, erster.p), r3.x + 14.0, yy + 4.0, 13.0, r3.w - 28.0, 1, Theme::text());
            yy += 24.0;
            gezeigt += 1;
        }
        if let Some(tag) = lg.fixtures.iter().filter(|f| f.hg >= 0).map(|f| f.day).max() {
            ui.text(&crate::tf!("ui.liga.zuletzt", date::short(c.world.season, tag)), r3.x + 14.0, yy + 4.0, 12.0, Theme::gedaempft());
            yy += 20.0;
            for f in lg.fixtures.iter().filter(|f| f.day == tag && f.hg >= 0) {
                if yy > r3.y + r3.h - 24.0 { break; }
                ui.text_passend(&format!("{} – {} {}:{}", c.world.clubs[f.h].short, c.world.clubs[f.a].short, f.hg, f.ag), r3.x + 14.0, yy + 4.0, 13.0, r3.w - 28.0, 1, Theme::text2());
                yy += 20.0;
                gezeigt += 1;
            }
        }
    }
    if gezeigt == 0 { ui.text(crate::inhalt::t("ui.liga.keine_news"), r3.x + 14.0, yy, 13.0, Theme::text2()); }
    auswahl_zeichnen(ui, v, &felder);
}

/// A cup or European competition: league-phase table (Europe) and the knock-out rounds with results.
static CUP_REITER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
/// The latest results and the next fixtures of a competition (Fabian 23.11.: always on the schedule); the own club's
/// matches highlighted, the latest stage of play first.
fn cup_spiele_kasten(ui: &mut Ui, c: &Career, ci: usize, r: Rect, lk: &Look) {
    let cup = &c.world.cups[ci];
    ui.card_titled(r, crate::inhalt::t("ui.cup.spiele"), lk.c1);
    let mut gespielt: Vec<&crate::cups::CupFx> = cup.fx.iter().filter(|f| f.f.hg >= 0).collect();
    gespielt.sort_by_key(|f| std::cmp::Reverse(f.f.day));
    let mut offen: Vec<&crate::cups::CupFx> = cup.fx.iter().filter(|f| f.f.hg < 0).collect();
    offen.sort_by_key(|f| f.f.day);
    let n = (((r.h - 110.0) / 2.0) / 20.0).max(1.0) as usize;
    let mut y = r.y + 54.0;
    for (titel, liste) in [(crate::inhalt::t("ui.cup.letzte"), gespielt), (crate::inhalt::t("ui.cup.naechste"), offen)] {
        ui.text_semi(titel, r.x + 14.0, y, 14.0, Theme::akzent());
        y += 22.0;
        if liste.is_empty() { ui.text(crate::inhalt::t("ui.cup.keine"), r.x + 14.0, y, 14.0, Theme::gedaempft()); y += 22.0; }
        // the own matches first, then the rest of that day
        let mut l = liste.clone();
        l.sort_by_key(|f| !(f.f.h == c.own || f.f.a == c.own));
        let tag = l.first().map(|f| f.f.day);
        let mut auswahl: Vec<&&crate::cups::CupFx> = l.iter().filter(|f| f.f.h == c.own || f.f.a == c.own).take(2).collect();
        auswahl.extend(liste.iter().filter(|f| Some(f.f.day) == tag && !(f.f.h == c.own || f.f.a == c.own)));
        for f in auswahl.into_iter().take(n) {
            let mine = f.f.h == c.own || f.f.a == c.own;
            let res = if f.f.hg < 0 { date::short(c.world.season, f.f.day) } else { format!("{}:{}", f.f.hg, f.f.ag) };
            let t = format!("{} – {}", c.world.club(f.f.h).short, c.world.club(f.f.a).short);
            ui.text(&ui.fit(&t, 14.0, r.w - 100.0), r.x + 14.0, y, 14.0, if mine { Theme::gold() } else { Theme::text() });
            ui.text_right(&res, r.x + r.w - 14.0, y, 14.0, Theme::text2());
            y += 20.0;
        }
        y += 14.0;
    }
}
fn cup_view(ui: &mut Ui, c: &Career, ci: usize, x: f32, tw: f32) {
    let cup = &c.world.cups[ci];
    let land = if cup.country.is_empty() { c.world.leagues.iter().find(|l| l.key == cup.key).map(|l| l.land.clone()).unwrap_or_default() } else { cup.country.clone() };
    let lk = wettbewerb_look(c, &cup.key, cup.kind, &land);
    let nach_name = wettbewerb_banner(ui, Rect::new(x, Y0 - 44.0, tw, 46.0), &lk, c.world.cids.get(&cup.key).copied(), &cup.name);
    let status = match cup.winner { Some(w) => crate::tf!("ui.pages.sieger", c.world.club(w).name), None => crate::tf!("ui.pages.laeuft", if cup.stage < 10 { crate::inhalt::t("ui.pages.ligaphase").to_string() } else { crate::cups::stage_name(cup.stage, cup.kind) }) };
    // more about the competition (Fabian 23.11.): title holder, clubs in it, where the final is played
    let mut info = vec![status];
    if let Some(&t) = c.world.titelverteidiger.get(&cup.name) { info.push(crate::tf!("ui.cup.titelverteidiger", c.world.clubs[t].name.clone())); }
    info.push(crate::tf!("ui.cup.teilnehmer", cup.teams.len()));
    if let Some(o) = crate::regeln::wert("pokalnamen", "finale").get(&cup.name).and_then(|x| x.as_str()) { info.push(crate::tf!("ui.cup.finalort", o)); } // roh
    let status = info.join(" · ");
    ui.text(&ui.fit(&status, 15.0, (x + tw - 500.0 - nach_name - 16.0).max(60.0)), nach_name + 16.0, Y0 - 14.0, 15.0, lk.text2);
    // league phase and knock-out rounds in two tabs (Fabian 23.11.), the matches box always on the right
    let liga = (1..=3).contains(&cup.kind);
    let mut reiter = if liga { CUP_REITER.load(std::sync::atomic::Ordering::Relaxed) } else { 1 };
    let mut top = Y0 + 6.0;
    if liga {
        if let Some(i) = ui.segment(Rect::new(x, Y0 + 6.0, 360.0, 30.0), &[crate::inhalt::t("ui.pages.ligaphase"), crate::inhalt::t("wb.liste.runden")], reiter) { reiter = i; CUP_REITER.store(i, std::sync::atomic::Ordering::Relaxed); }
        top += 40.0;
    }
    let kw = (tw * 0.30).max(280.0);
    cup_spiele_kasten(ui, c, ci, Rect::new(x + tw - kw, top, kw, hoehe() - top - 16.0), &lk);
    let (rx, rw) = (x, tw - kw - 12.0);
    if reiter == 0 {
        // the detailed league-phase table: played, won, drawn, lost, goals, difference, points, form
        let area = Rect::new(rx, top, rw, hoehe() - top - 16.0);
        ui.card_titled(area, crate::inhalt::t("ui.pages.ligaphase"), lk.c1);
        let tab = c.world.cup_table(ci);
        let rh = ((area.h - 70.0) / tab.len().max(1) as f32).min(24.0);
        let spalten = [(crate::inhalt::t("ui.cup.sp"), 0.50), (crate::inhalt::t("ui.cup.s"), 0.57), (crate::inhalt::t("ui.cup.u"), 0.63), (crate::inhalt::t("ui.cup.n"), 0.69), (crate::inhalt::t("ui.cup.tore"), 0.76), (crate::inhalt::t("ui.cup.diff"), 0.84), (crate::inhalt::t("ui.cup.pkt"), 0.91)];
        for (t, f) in spalten { ui.text_center(t, area.x + area.w * f, area.y + 58.0, 12.0, Theme::gedaempft()); }
        for (i, r) in tab.iter().enumerate() {
            let y = area.y + 66.0 + i as f32 * rh;
            if r.club == c.own { rrect(area.x + 4.0, y, area.w - 8.0, rh - 1.0, 3.0, alpha(Theme::akzent(), 0.16)); }
            let zone = if i < 8 { Theme::gut() } else if i < 24 { Theme::achtung() } else { Theme::problem() };
            draw_rectangle(area.x + 4.0, y + 2.0, 3.0, rh - 4.0, zone);
            let fs = (rh * 0.66).min(15.0);
            ui.text(&format!("{}", i + 1), area.x + 12.0, y + rh * 0.75, fs, Theme::text2());
            let k = c.world.club(r.club);
            ui.club_crest(area.x + 44.0, y + rh / 2.0, rh * 0.42, k.id, hex(k.c1), &k.short);
            ui.text(&ui.fit(&k.name, fs, area.w * 0.44 - 60.0), area.x + 58.0, y + rh * 0.75, fs, Theme::text());
            let werte = [r.p.to_string(), r.w.to_string(), r.d.to_string(), r.l.to_string(), format!("{}:{}", r.gf, r.ga), format!("{:+}", r.gf - r.ga), r.pts.to_string()];
            for (k2, (_, f)) in spalten.iter().enumerate() { ui.text_center(&werte[k2], area.x + area.w * f, y + rh * 0.75, fs, if k2 == 6 { Theme::text() } else { Theme::text2() }); }
            let form: String = r.form.iter().rev().take(5).collect();
            ui.text_right(&form, area.x + area.w - 10.0, y + rh * 0.75, fs * 0.9, Theme::gedaempft());
        }
        let _ = nach_name;
        return;
    }
    let rr = Rect::new(rx, top, rw, hoehe() - top - 16.0);
    ui.card_titled(rr, crate::inhalt::t("wb.liste.runden"), lk.c1);
    let mut stages: Vec<u8> = cup.fx.iter().map(|f| f.stage).filter(|s| *s >= 20).collect();
    stages.sort();
    stages.dedup();
    stages.reverse();
    let mut y = rr.y + 54.0;
    if stages.is_empty() { ui.text(crate::inhalt::t("ui.pages.die_k_o_runden_folgen"), rr.x + 14.0, y, 15.0, Theme::text2()); }
    // from the last 16 on: a bracket, one half left, the other right, the final in the middle (Fabian 22.11.)
    if turnierbaum(ui, c, ci, Rect::new(rr.x + 8.0, rr.y + 44.0, rr.w - 16.0, rr.h - 52.0), &lk) { return; }
    for st in stages {
        if y > rr.y + rr.h - 30.0 { break; }
        let n = cup.fx.iter().filter(|f| f.stage == st).map(|f| (f.f.h.min(f.f.a), f.f.h.max(f.f.a))).collect::<std::collections::HashSet<_>>().len();
        ui.text_semi(&crate::cups::runden_name(st, cup.kind, n), rr.x + 14.0, y, 15.0, Theme::akzent());
        y += 22.0;
        for f in cup.fx.iter().filter(|f| f.stage == st) {
            if y > rr.y + rr.h - 10.0 { break; }
            let res = if f.f.hg < 0 { date::short(c.world.season, f.f.day) } else { format!("{}:{}{}{}", f.f.hg, f.f.ag, if f.et { crate::inhalt::t("ui.pages.kurz_nv") } else { "" }, f.pens.map(|p| crate::tf!("ui.pages.kurz_ie", crate::cups::pens_score(p))).unwrap_or_default()) };
            let t = format!("{} – {}", c.world.club(f.f.h).short, c.world.club(f.f.a).short);
            let mine = f.f.h == c.own || f.f.a == c.own;
            ui.text(&t, rr.x + 14.0, y, 15.0, if mine { Theme::gold() } else { Theme::text() });
            ui.text_right(&res, rr.x + rr.w - 14.0, y, 15.0, Theme::text2());
            y += 20.0;
        }
        y += 8.0;
    }
}

/// One tie of a knock-out round: the two clubs (None = not decided yet), the score text and the winner.
#[derive(Clone)]
struct Paarung { a: Option<usize>, b: Option<usize>, text: String, sieger: Option<usize> }

/// The bracket of a cup from the first round with at most eight ties: the rounds as columns from both sides towards
/// the final in the middle; ties of a round sit next to the tie of the next round their winners met in, rounds not
/// drawn yet as empty boxes. False when no round is small enough yet (then the list is shown).
fn turnierbaum(ui: &mut Ui, c: &Career, ci: usize, r: Rect, lk: &Look) -> bool {
    let cup = &c.world.cups[ci];
    let mut stages: Vec<u8> = cup.fx.iter().map(|f| f.stage).filter(|s| *s >= 20).collect();
    stages.sort();
    stages.dedup();
    let paare = |st: u8| -> Vec<Paarung> {
        let mut v: Vec<Paarung> = vec![];
        for f in cup.fx.iter().filter(|f| f.stage == st) {
            let (h, a) = (f.f.h, f.f.a);
            if v.iter().any(|p| (p.a == Some(h) && p.b == Some(a)) || (p.a == Some(a) && p.b == Some(h))) { continue; }
            let legs: Vec<&crate::cups::CupFx> = cup.fx.iter().filter(|x| x.stage == st && ((x.f.h == h && x.f.a == a) || (x.f.h == a && x.f.a == h))).collect();
            let gespielt = legs.iter().all(|x| x.f.hg >= 0);
            let text = if !gespielt { legs.iter().find(|x| x.f.hg < 0).map(|x| date::short(c.world.season, x.f.day)).unwrap_or_default() } else {
                let (mut ta, mut tb) = (0i32, 0i32);
                for x in &legs { if x.f.h == h { ta += x.f.hg as i32; tb += x.f.ag as i32; } else { ta += x.f.ag as i32; tb += x.f.hg as i32; } }
                let pens = legs.iter().find_map(|x| x.pens).map(|p| crate::tf!("ui.pages.kurz_ie", crate::cups::pens_score(p)).trim().to_string()).unwrap_or_default();
                format!("{ta}:{tb}{}", if pens.is_empty() { String::new() } else { format!(" {pens}") })
            };
            let sieger = if gespielt { Some(c.world.winner_of(ci, st, h, a)) } else { None };
            v.push(Paarung { a: Some(h), b: Some(a), text, sieger });
        }
        v
    };
    // the bracket starts at a round of 2, 4 or 8 ties after which every round halves (23.11.: a preliminary round
    // with byes, e.g. 2 ties among 18 clubs, was taken as the semi-finals and the next round shown as the final)
    let zahl: Vec<usize> = stages.iter().map(|&st| paare(st).len()).collect();
    let Some(erste) = (0..stages.len()).find(|&i| {
        let n = zahl[i];
        n > 0 && n <= 8 && n.is_power_of_two() && zahl[i + 1..].iter().enumerate().all(|(k, &m)| m == n >> (k + 1))
    }) else { return false };
    let n0 = paare(stages[erste]).len().next_power_of_two();
    let runden = (n0 as f32).log2() as usize + 1;
    let mut baum: Vec<Vec<Paarung>> = (0..runden).map(|k| stages.get(erste + k).map(|&st| paare(st)).unwrap_or_default()).collect();
    // empty boxes for rounds still to come, then order every round after the next one (backwards from the final)
    for k in 0..runden { let soll = n0 >> k; while baum[k].len() < soll { baum[k].push(Paarung { a: None, b: None, text: String::new(), sieger: None }); } }
    for k in (0..runden - 1).rev() {
        let mut neu: Vec<Paarung> = vec![];
        let mut rest = baum[k].clone();
        for p in &baum[k + 1] {
            for t in [p.a, p.b] {
                let pos = t.and_then(|t| rest.iter().position(|x| x.sieger == Some(t) || x.a == Some(t) || x.b == Some(t)));
                if let Some(i) = pos { neu.push(rest.remove(i)); } else if !rest.is_empty() { neu.push(rest.remove(0)); }
            }
        }
        neu.extend(rest);
        baum[k] = neu;
    }
    // columns: rounds 0..runden-2 on the left (first half of each), the final in the middle, the right half mirrored
    let spalten = 2 * (runden - 1) + 1;
    let sw = r.w / spalten as f32;
    let bw = (sw - 10.0).min(150.0);
    let bh = 40.0;
    let mitte_y = |k: usize, i: usize, n: usize| -> f32 { let h = r.h - 30.0; r.y + 24.0 + h * (i as f32 + 0.5) / n as f32 };
    let kasten = |ui: &mut Ui, x: f32, y: f32, p: &Paarung, finale: bool| {
        let br = Rect::new(x - bw / 2.0, y - bh / 2.0, bw, bh);
        rrect(br.x, br.y, br.w, br.h, 6.0, if finale { alpha(lk.c1, 0.9) } else { Theme::erhoehung() });
        if finale { rrect_line(br.x, br.y, br.w, br.h, 6.0, lk.akzent); }
        let farbe = if finale { lk.text } else { Theme::text() };
        for (j, t) in [p.a, p.b].iter().enumerate() {
            let yy = br.y + 14.0 + j as f32 * 18.0;
            match t {
                Some(k) => {
                    let cl = c.world.club(*k);
                    ui.club_crest(br.x + 11.0, yy - 4.0, 7.0, cl.id, hex(cl.c1), &cl.short);
                    let raus = p.sieger.is_some() && p.sieger != Some(*k);
                    let col = if *k == c.own { Theme::gold() } else if raus { alpha(farbe, 0.45) } else { farbe };
                    ui.text(&ui.fit(&cl.short, 13.0, bw - 30.0), br.x + 22.0, yy, 13.0, col);
                }
                None => { ui.text("–", br.x + 22.0, yy, 13.0, alpha(farbe, 0.4)); }
            }
        }
        if !p.text.is_empty() { ui.text_right(&ui.fit(&p.text, 11.0, bw * 0.5), br.x + br.w - 6.0, br.y + 22.0, 11.0, if finale { lk.text2 } else { Theme::text2() }); }
    };
    let linie = alpha(Theme::text2(), 0.45);
    for k in 0..runden {
        let n = baum[k].len();
        if k == runden - 1 {
            let x = r.x + sw * (runden - 1) as f32 + sw / 2.0;
            ui.text_center(&crate::cups::stage_name(34, cup.kind), x, r.y + 12.0, 13.0, lk.akzent_lesbar);
            if let Some(w) = baum[k][0].sieger { let cl = c.world.club(w); ui.club_crest(x, r.y + r.h / 2.0 - 70.0, 26.0, cl.id, hex(cl.c1), &cl.short); ui.text_center(&crate::tf!("ui.pages.sieger", cl.name), x, r.y + r.h / 2.0 - 34.0, 13.0, Theme::gold()); }
            kasten(ui, x, (r.y + 24.0 + r.y + r.h - 6.0) / 2.0, &baum[k][0], true);
            continue;
        }
        let haelfte = n / 2;
        let name = stages.get(erste + k).map(|&st| crate::cups::runden_name(st, cup.kind, n)).unwrap_or_default();
        for seite in 0..2 {
            let spalte = if seite == 0 { k } else { spalten - 1 - k };
            let x = r.x + sw * spalte as f32 + sw / 2.0;
            ui.text_center(&ui.fit(&name, 12.0, sw), x, r.y + 12.0, 12.0, Theme::gedaempft());
            for i in 0..haelfte {
                let p = &baum[k][seite * haelfte + i];
                let y = mitte_y(k, i, haelfte);
                // connector towards the next round
                let nx = if seite == 0 { x + sw / 2.0 } else { x - sw / 2.0 };
                let kante = if seite == 0 { x + bw / 2.0 } else { x - bw / 2.0 };
                draw_line(kante, y, nx, y, 1.5, linie);
                let ny = if k + 2 == runden { (r.y + 24.0 + r.y + r.h - 6.0) / 2.0 } else { mitte_y(k + 1, i / 2, (haelfte / 2).max(1)) };
                draw_line(nx, y, nx, ny, 1.5, linie);
                kasten(ui, x, y, p, false);
            }
        }
    }
    true
}
