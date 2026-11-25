use super::*;

pub(super) fn akademie(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // Akademie-Konzept: overview, U19, U17, mentors; with a real second team its tab comes last (farm.rs)
    let zweite = c.zweite_mannschaft().is_some();
    let mut reiter = vec![crate::inhalt::t("reiter.uebersicht"), crate::inhalt::t("ui.wort.gross_u19"), crate::inhalt::t("ui.wort.gross_u17"), crate::inhalt::t("ui.jugend.reiter.mentoren")];
    if zweite { reiter.push(crate::inhalt::t("ui.game.zweite_mannschaft")); }
    tabs(ui, v, &reiter);
    match v.tab { 0 => { akademie_uebersicht(ui, v, c); return; } 3 => { mentoren_seite(ui, v, c); return; } 4 if zweite => { zweite_mannschaft_tab(ui, v, c); return; } _ => {} }
    let k = if v.tab == 1 { 1 } else { 0 };
    let g = if k == 1 { 19 } else { 17 };
    let sq = c.youth_squad(g);
    let lw = cbreite() * 0.58;
    let area = Rect::new(x0() + 4.0, Y0, lw, hoehe() - Y0 - 16.0);
    ui.card(area);
    ui.title(&crate::tf!("ui.pages.spieler", c.club().name, if k == 1 { crate::inhalt::t("ui.wort.gross_u19") } else { crate::inhalt::t("ui.wort.gross_u17") }, sq.len()), area.x + 14.0, area.y + 32.0, 22.0, Theme::text());
    let vis = ((area.h - 60.0) / 40.0) as usize;
    let start = scroll_rows(ui, v, area, sq.len(), vis);
    for (j, &p) in sq.iter().skip(start).take(vis).enumerate() {
        let pl = c.world.players[p].clone();
        let r = Rect::new(area.x + 6.0, area.y + 48.0 + j as f32 * 40.0, area.w - 12.0, 36.0);
        if j % 2 == 1 { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::erhoehung(), 0.6)); }
        ui.pos_tag(r.x + 6.0, r.y + 18.0, pos_de(&pl.pos));
        ui.player_photo(r.x + 56.0, r.y + 18.0, 13.0, pl.id, &pl.name);
        let nm = ui.fit(&pl.name, 17.0, 160.0);
        ui.text_semi(&nm, r.x + 76.0, r.y + 24.0, 17.0, Theme::text());
        ui.text(&crate::tf!("ui.wort.alter_kurz", pl.age), r.x + 240.0, r.y + 24.0, 16.0, Theme::text2());
        ui.pill(r.x + 310.0, r.y + 18.0, pl.ovr, 34.0);
        let (pa, pb) = c.pot_range(p);
        ui.text(&crate::tf!("ui.pages.pot", pa, pb), r.x + 336.0, r.y + 24.0, 15.0, Theme::gold());
        entwicklung_pfeil_mit(ui, r.x + 422.0, r.y + 18.0, c.ent_erwartung(p), false);
        if ui.button(Rect::new(r.x + r.w - 350.0, r.y + 3.0, 124.0, 30.0), crate::inhalt::t(if pl.ps.gast { "ui.jugend.gast.beenden" } else { "ui.jugend.gast.knopf" }), pl.ps.gast) {
            match c.gast_setzen(p, !pl.ps.gast) { Ok(t) => v.toast(&t, true), Err(e) => v.toast(&e, false) }
        }
        if pl.ps.mentor.is_some() { ui.text("M", r.x + 470.0, r.y + 24.0, 15.0, Theme::gold()); } // roh
        if ui.button(Rect::new(r.x + r.w - 220.0, r.y + 3.0, 120.0, 30.0), crate::inhalt::t("ui.pages.zu_den_profis"), false) { c.promote(p); v.toast(&crate::tf!("ui.game.rueckt_zu_den_profis_auf", pl.name), true); }
        if ui.button(Rect::new(r.x + r.w - 92.0, r.y + 3.0, 86.0, 30.0), crate::inhalt::t("ui.pages.abgeben"), false) { c.release(p); }
    }
    let rt = Rect::new(x0() + lw + 18.0, Y0, cbreite() - lw - 14.0, hoehe() - Y0 - 16.0);
    ui.card_titled(rt, &c.youth[k].name.clone(), Theme::akzent());
    let yl = c.youth[k].clone();
    let mut y = rt.y + 50.0;
    // a click on a rival shows its real academy squad (FC-26 wish), scoutable and signable
    if let Some(row) = v.jugend_verein.filter(|&r| r < yl.rows.len() && !yl.rows[r].own) {
        let r = &yl.rows[row];
        ui.text_semi(&r.name, rt.x + 14.0, y, 16.0, Theme::akzent());
        if ui.button(Rect::new(rt.x + rt.w - 110.0, y - 20.0, 96.0, 26.0), crate::inhalt::t("karrieren.zurueck"), false) { v.jugend_verein = None; }
        y += 26.0;
        let mut kader: Vec<(usize, &crate::game::YSpieler)> = r.kader.iter().enumerate().collect();
        kader.sort_by(|a, b| (b.1.pot).partial_cmp(&a.1.pot).unwrap());
        let mut aktion = None;
        for (i, s) in kader {
            if y > rt.y + rt.h - 20.0 { break; }
            ui.pos_tag(rt.x + 14.0, y - 6.0, pos_de(&s.pos));
            ui.text(&ui.fit(&s.name, 14.0, rt.w * 0.4), rt.x + 62.0, y, 14.0, Theme::text());
            // the potential as a range: an estimate, the scouting of foreign youths is not exact
            ui.text(&crate::tf!("ui.jugend.fremd.zeile", s.age, format!("{:.0}", s.ovr), format!("{:.0}", s.pot - 4.0), format!("{:.0}", s.pot + 4.0), s.tore), rt.x + rt.w * 0.45, y, 13.0, Theme::text2());
            if ui.button(Rect::new(rt.x + rt.w - 100.0, y - 17.0, 86.0, 24.0), crate::inhalt::t("ui.jugend.fremd.holen"), false) { aktion = Some(i); }
            y += 26.0;
        }
        if let Some(i) = aktion { match c.jugend_verpflichten(k, row, i) { Ok(t) => v.toast(&t, true), Err(e) => if !e.is_empty() { v.toast(&e, false) } } }
        return;
    }
    for (i, r) in yl.rows.iter().enumerate().take(12) {
        if !r.own && ui.hit(Rect::new(rt.x + 6.0, y - 18.0, rt.w - 12.0, 26.0)) { v.jugend_verein = Some(i); }
        if r.own { rrect(rt.x + 6.0, y - 18.0, rt.w - 12.0, 26.0, 4.0, alpha(Theme::akzent(), 0.15)); }
        ui.text(&format!("{}", i + 1), rt.x + 14.0, y, 16.0, Theme::text2());
        let nm = ui.fit(&r.name, 16.0, rt.w - 170.0);
        ui.text(&nm, rt.x + 40.0, y, 16.0, Theme::text());
        ui.text_right(&format!("{}  {:+}  {}", r.p, r.gf - r.ga, r.pts), rt.x + rt.w - 14.0, y, 16.0, Theme::text());
        y += 28.0;
    }
    // the league's top scorers, own and rival academies together
    let mut tj: Vec<(String, String, u16)> = yl.rows.iter().filter(|r| !r.own).flat_map(|r| r.kader.iter().filter(|s| s.tore > 0).map(move |s| (s.name.clone(), r.name.clone(), s.tore))).collect();
    let g = if k == 0 { 17 } else { 19 };
    tj.extend(c.youth_squad(g).into_iter().filter(|&p| c.world.players[p].goals > 0).map(|p| (c.world.players[p].name.clone(), c.club().name.clone(), c.world.players[p].goals as u16)));
    tj.sort_by(|a, b| b.2.cmp(&a.2));
    if !tj.is_empty() {
        ui.text_semi(crate::inhalt::t("ui.jugend.fremd.torjaeger"), rt.x + 14.0, y + 10.0, 15.0, Theme::akzent());
        for (n, cl, t) in tj.iter().take(3) { y += 22.0; ui.text(&ui.fit(&format!("{t} · {n} ({cl})"), 14.0, rt.w - 28.0), rt.x + 14.0, y + 10.0, 14.0, Theme::text()); }
        y += 24.0;
    }
    ui.text_semi(crate::inhalt::t("spielbericht.letztes"), rt.x + 14.0, y + 20.0, 15.0, Theme::text2());
    ui.paragraph(if yl.last.is_empty() { crate::inhalt::t("ui.pages.noch_nicht_gespielt_sonntags") } else { &yl.last }, rt.x + 14.0, y + 42.0, 15.0, rt.w - 28.0, Theme::text());
}

/// Akademie overview: youth strength with its four parts, youth reputation, the training guests and who is ready.
fn akademie_uebersicht(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let jugend_farbe = hex(0x2dd4bf);
    let j = c.jugendstaerke(c.own);
    let lw = cbreite() * 0.42;
    let l = Rect::new(x0() + 4.0, Y0, lw, 330.0);
    ui.card(l);
    ui.text_semi(crate::inhalt::t("ui.jugend.staerke"), l.x + 16.0, l.y + 30.0, 16.0, Theme::text2());
    ui.title(&format!("{:.1}", j.wert), l.x + 16.0, l.y + 86.0, 54.0, jugend_farbe);
    for s in 0..5 { draw_poly(l.x + 140.0 + s as f32 * 24.0, l.y + 72.0, 5, 10.0, -90.0, if s < c.jugend_ruf() as usize { Theme::gold() } else { Theme::ueberlagerung() }); }
    ui.text(crate::inhalt::t("ui.jugend.ruf"), l.x + 130.0, l.y + 56.0, 13.0, Theme::gedaempft());
    let mut y = l.y + 124.0;
    for (key, w, g) in [("ui.jugend.teil.anlagen", j.anlagen, "staerke.anlagen"), ("ui.jugend.teil.stab", j.stab, "staerke.stab"), ("ui.jugend.teil.region", j.region, "staerke.region"), ("ui.jugend.teil.tradition", j.tradition, "staerke.tradition")] {
        ui.text(crate::inhalt::t(key), l.x + 16.0, y + 4.0, 15.0, Theme::text());
        ui.bar(l.x + 160.0, y - 6.0, l.w - 260.0, 10.0, w / 10.0, jugend_farbe);
        ui.text_right(&format!("{:.1} · {:.0} %", w, crate::regeln::f("jugendakademie", g) * 100.0), l.x + l.w - 16.0, y + 4.0, 14.0, Theme::text2());
        y += 34.0;
    }
    if let Some(b) = j.bekannt { ui.text(&crate::tf!("ui.jugend.bekannt", b), l.x + 16.0, y + 6.0, 13.0, Theme::gedaempft()); }
    // the scouting tournament, once a season
    let schon = c.sichtung_saison == c.world.season + 1;
    if ui.button(Rect::new(l.x + l.w - 250.0, l.y + 12.0, 234.0, 32.0), crate::inhalt::t(if schon { "ui.jugend.sichtung.erledigt" } else { "ui.jugend.sichtung.knopf" }), !schon) && !schon {
        match c.sichtungsturnier() { Ok(t) => v.toast(&t, true), Err(e) => v.toast(&e, false) }
    }
    // right: training guests and who is ready for the first team
    let r = Rect::new(x0() + lw + 16.0, Y0, cbreite() - lw - 12.0, hoehe() - Y0 - 16.0);
    ui.card(r);
    let jung: Vec<usize> = c.world.clubs[c.own].squad.iter().copied().filter(|&p| c.world.players[p].youth > 0).collect();
    let gaeste: Vec<usize> = jung.iter().copied().filter(|&p| c.world.players[p].ps.gast).collect();
    ui.text_semi(&crate::tf!("ui.jugend.gaeste", gaeste.len(), crate::regeln::f("jugendakademie", "hochziehen.gast_hoechstens")), r.x + 16.0, r.y + 30.0, 16.0, Theme::text());
    let mut y = r.y + 58.0;
    for &p in &gaeste { let pl = &c.world.players[p]; ui.pos_tag(r.x + 16.0, y - 5.0, pos_de(&pl.pos)); ui.text(&format!("{} · {:.0}", pl.name, pl.ovr), r.x + 60.0, y, 15.0, Theme::text2()); y += 24.0; }
    y += 14.0;
    ui.text_semi(crate::inhalt::t("ui.jugend.talente"), r.x + 16.0, y, 16.0, Theme::text());
    y += 12.0;
    let mut beste = jung.clone();
    beste.sort_by(|a, b| c.world.players[*b].pot.partial_cmp(&c.world.players[*a].pot).unwrap());
    for &p in beste.iter().take(((r.y + r.h - y - 20.0) / 34.0).max(0.0) as usize) {
        let pl = c.world.players[p].clone();
        let rr = Rect::new(r.x + 8.0, y, r.w - 16.0, 32.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
        ui.pos_tag(rr.x + 6.0, rr.y + 16.0, pos_de(&pl.pos));
        ui.text_semi(&ui.fit(&pl.name, 15.0, 200.0), rr.x + 50.0, rr.y + 21.0, 15.0, Theme::text());
        ui.text(&format!("{} · {}", pl.age, if pl.youth == 17 { crate::inhalt::t("ui.wort.gross_u17") } else { crate::inhalt::t("ui.wort.gross_u19") }), rr.x + 260.0, rr.y + 21.0, 14.0, Theme::text2());
        ui.pill(rr.x + 360.0, rr.y + 15.0, pl.ovr, 34.0);
        let (pa, pb) = c.pot_range(p);
        let st = (((pa + pb) / 2.0 - 50.0) / 9.0).round().clamp(1.0, 5.0) as usize;
        for s in 0..5 { draw_poly(rr.x + 400.0 + s as f32 * 16.0, rr.y + 16.0, 5, 6.0, -90.0, if s < st { Theme::gold() } else { Theme::ueberlagerung() }); }
        ui.text(crate::jugend::stufe_wort((pa + pb) / 2.0), rr.x + 490.0, rr.y + 21.0, 13.0, Theme::text2());
        if pl.ent.merkmale.iter().any(|m| m == "spaetzuender" || m == "fruehentwickler") { let m = pl.ent.merkmale.iter().find(|m| *m == "spaetzuender" || *m == "fruehentwickler").unwrap(); ui.text(crate::inhalt::t(&format!("ui.jugend.merkmal.{m}")), rr.x + 680.0, rr.y + 21.0, 13.0, Theme::gedaempft()); }
        if ui.hit(rr) { v.modal = Modal::Player(p); }
        y += 34.0;
    }
    // below left: the youth tables in short
    let t = Rect::new(x0() + 4.0, Y0 + 342.0, lw, hoehe() - Y0 - 358.0);
    ui.card(t);
    let mut y = t.y + 30.0;
    for k in 0..2 {
        let yl = &c.youth[k];
        let platz = yl.rows.iter().position(|r| r.own).map(|x| x + 1).unwrap_or(0);
        ui.text_semi(&yl.name, t.x + 16.0, y, 16.0, Theme::text());
        ui.text_right(&crate::tf!("ui.jugend.platz", platz, yl.rows.len()), t.x + t.w - 16.0, y, 15.0, jugend_farbe);
        y += 22.0;
        if !yl.last.is_empty() { let h = ui.paragraph(&yl.last, t.x + 16.0, y, 13.0, t.w - 32.0, Theme::text2()); y += h; }
        y += 18.0;
    }
}

/// Mentors: the pairs with their fit, and the co-trainer's proposals on request.
fn mentoren_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    ui.title(crate::inhalt::t("ui.jugend.reiter.mentoren"), area.x + 16.0, area.y + 34.0, 22.0, Theme::text());
    if ui.button(Rect::new(area.x + area.w - 300.0, area.y + 12.0, 284.0, 32.0), crate::inhalt::t("ui.jugend.mentor.vorschlagen"), true) {
        let n = c.mentor_vorschlaege();
        v.toast(&if n == 0 { crate::inhalt::t("ui.jugend.mentor.keine").to_string() } else { crate::tf!("ui.jugend.mentor.neu", n) }, n > 0);
    }
    ui.paragraph(crate::inhalt::t("ui.jugend.mentor.regel"), area.x + 16.0, area.y + 62.0, 13.0, area.w - 32.0, Theme::gedaempft());
    let sq = c.squad();
    let paare: Vec<(usize, usize)> = sq.iter().filter_map(|&s| c.world.players[s].ps.mentor.and_then(|m| c.pid(m)).map(|m| (m, s))).collect();
    let mut y = area.y + 100.0;
    if paare.is_empty() { ui.text(crate::inhalt::t("ui.jugend.mentor.leer"), area.x + 16.0, y + 20.0, 15.0, Theme::text2()); }
    let mut weg = None;
    for (m, s) in paare {
        let (pa, gr) = c.mentor_paar(m, s);
        let (mn, sn) = (c.world.players[m].name.clone(), c.world.players[s].name.clone());
        let r = Rect::new(area.x + 10.0, y, area.w - 20.0, 56.0);
        rrect(r.x, r.y, r.w, r.h, 8.0, alpha(hex(0x2dd4bf), 0.08));
        ui.player_photo(r.x + 26.0, r.y + 28.0, 16.0, c.world.players[m].id, &mn);
        ui.text_semi(&mn, r.x + 50.0, r.y + 24.0, 16.0, Theme::text());
        ui.text("→", r.x + 250.0, r.y + 24.0, 18.0, Theme::gedaempft()); // roh
        ui.player_photo(r.x + 296.0, r.y + 28.0, 16.0, c.world.players[s].id, &sn);
        ui.text_semi(&sn, r.x + 320.0, r.y + 24.0, 16.0, Theme::text());
        ui.text(&gr.iter().map(|g| crate::inhalt::t(g)).collect::<Vec<_>>().join(" "), r.x + 50.0, r.y + 46.0, 13.0, Theme::text2());
        ui.bar(r.x + 540.0, r.y + 20.0, 160.0, 8.0, pa, if pa >= 0.6 { Theme::gut() } else if pa >= 0.35 { Theme::achtung() } else { Theme::problem() });
        ui.text(&format!("{:.0} %", pa * 100.0), r.x + 710.0, r.y + 28.0, 14.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 140.0, r.y + 12.0, 128.0, 30.0), crate::inhalt::t("ui.jugend.mentor.trennen"), false) { weg = Some(s); }
        y += 64.0;
    }
    if let Some(s) = weg { c.world.players[s].ps.mentor = None; }
}

pub(super) fn jugendnetz(ui: &mut Ui, v: &mut View, c: &mut Career) {
    c.netz_an();
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.seiten.netz_je_land"), Theme::akzent());
    let netz = c.mgr.netz.clone();
    let mut sorted = netz.clone();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (k, (land, st, inv)) in sorted.iter().enumerate() {
        let yy = l.y + 62.0 + k as f32 * 34.0;
        if yy > l.y + l.h - 14.0 { break; }
        ui.text(&ui.fit(&crate::nationen::anzeige(land), 16.0, 130.0), l.x + 14.0, yy, 16.0, Theme::text());
        ui.bar(l.x + 150.0, yy - 10.0, 120.0, 8.0, st / 100.0, if *st >= 35.0 { Theme::gut() } else if *st >= 20.0 { Theme::achtung() } else { Theme::ueberlagerung() });
        if let Some(i) = ui.segment(Rect::new(l.x + 290.0, yy - 22.0, l.w - 304.0, 28.0), &[crate::inhalt::t("ui.seiten.kein_geld"), crate::inhalt::t("ui.wort.wenig"), crate::inhalt::t("ui.wort.mittel"), crate::inhalt::t("ui.wort.viel")], *inv as usize) { c.netz_stufe(land, i as u8); }
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.talente_ab_netzstaerke_20_sichtbar"), Theme::gold());
    for (k, t) in c.talente().into_iter().enumerate() {
        let yy = r.y + 62.0 + k as f32 * 40.0;
        if yy > r.y + r.h - 14.0 { break; }
        ui.pos_tag(r.x + 14.0, yy - 5.0, pos_de(&t.pos));
        ui.text(&crate::tf!("ui.wort.name_alter_land", t.name, t.alter, crate::nationen::anzeige(&t.land)), r.x + 58.0, yy, 15.0, Theme::text());
        ui.text(&crate::tf!("ui.seiten.potenzial", stufe_wort(t.ovr + 20.0), stufe_wort(t.pot)), r.x + 14.0, yy + 18.0, 13.0, Theme::text2());
        let laeuft = c.mgr.anwerbung.iter().any(|a| a.0 == t.key);
        if laeuft { ui.text_right(crate::inhalt::t("ui.seiten.wird_angeworben"), r.x + r.w - 14.0, yy, 14.0, Theme::achtung()); }
        else if ui.button(Rect::new(r.x + r.w - 140.0, yy - 18.0, 126.0, 28.0), crate::inhalt::t("jn.ui.anwerben"), false) { match c.anwerben(&t) { Ok(()) => v.toast(crate::inhalt::t("ui.seiten.anwerbung_laeuft_28_tage"), true), Err(e) => v.toast(&e, false) } }
    }
}
