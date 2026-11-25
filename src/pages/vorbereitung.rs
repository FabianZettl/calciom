use super::*;

/// The opponent report (Fabian 24.11.: "stats, stats, stats; how they line up, how they play, what the assistant
/// would do, and do it at once"): their probable eleven, both sides' season numbers, the report, the suggestions.
pub(super) fn gegner_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    let (Some(g), Some((gn, zeilen))) = (c.naechster_gegner(), c.gegnerbericht()) else {
        ui.card_titled(r, t("gegner.bericht"), Theme::akzent()); ui.text(t("vorbereitung.kein_spiel"), r.x + 14.0, r.y + 64.0, 18.0, Theme::text2()); return;
    };
    let (gc, _) = club_color(c, g);
    let (uc, _) = club_color(c, c.own);
    let sw = r.w * 0.36;
    let schluessel = c.schluesselspieler();
    // left: their probable eleven with faces, by line; key players ringed in gold; under it the danger men as cards
    let lr = Rect::new(r.x, r.y, sw, r.h);
    ui.card_titled(lr, &gn, gc);
    ui.club_crest(lr.x + lr.w - 28.0, lr.y + 22.0, 14.0, c.world.clubs[g].id, gc, &c.world.clubs[g].short);
    let trainer = c.world.clubs[g].manager.clone();
    if !trainer.is_empty() { ui.text(&ui.fit(&crate::tf!("ui.duell.trainer", trainer), 14.0, lr.w - 28.0), lr.x + 14.0, lr.y + 62.0, 14.0, Theme::text2()); }
    let feld = Rect::new(lr.x + 12.0, lr.y + 74.0, lr.w - 24.0, (lr.h * 0.58).max(300.0));
    rrect(feld.x, feld.y, feld.w, feld.h, 8.0, hex(0x1e5a35));
    for i in 0..6 { if i % 2 == 0 { draw_rectangle(feld.x, feld.y + i as f32 * feld.h / 6.0, feld.w, feld.h / 6.0, hex(0x226440)); } }
    let lc = Color::new(1.0, 1.0, 1.0, 0.35);
    draw_rectangle_lines(feld.x + 6.0, feld.y + 6.0, feld.w - 12.0, feld.h - 12.0, 1.5, lc);
    draw_line(feld.x + 6.0, feld.y + feld.h / 2.0, feld.x + feld.w - 6.0, feld.y + feld.h / 2.0, 1.5, lc);
    draw_circle_lines(feld.x + feld.w / 2.0, feld.y + feld.h / 2.0, feld.h * 0.12, 1.5, lc);
    let elf = c.world.lineup(g, 0, &[]).0;
    for teil in 0..4u8 {
        let reihe: Vec<&(String, usize, f32)> = elf.iter().filter(|x| crate::world::part_of(&x.0) == teil as usize).collect();
        let y = feld.y + feld.h * [0.88, 0.66, 0.40, 0.15][teil as usize];
        for (k, (slot, p, _)) in reihe.iter().enumerate() {
            let x = feld.x + feld.w * (k as f32 + 1.0) / (reihe.len() as f32 + 1.0);
            let pl = &c.world.players[*p];
            let gold = schluessel.contains(p);
            ui.player_photo(x, y, 17.0, pl.id, &pl.name);
            draw_circle_lines(x, y, 17.0, if gold { 3.0 } else { 2.0 }, if gold { Theme::gold() } else { Theme::teil(slot) });
            rrect(x + 8.0, y - 22.0, 22.0, 14.0, 3.0, Theme::stufe(pl.ovr));
            ui.text_center(&format!("{:.0}", pl.ovr), x + 19.0, y - 11.0, 11.0, hex(0x10141c));
            let kurz = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
            ui.text_center(&ui.fit(&kurz, 11.0, feld.w / (reihe.len() as f32 + 1.0) - 4.0), x, y + 30.0, 11.0, hex(0xffffff));
        }
    }
    // the danger men: up to three cards with photo, position, goals and assists
    let dy = feld.y + feld.h + 14.0;
    ui.text_semi(t("ui.gv.gefahr"), lr.x + 14.0, dy + 6.0, 13.0, Theme::gedaempft());
    let dw = (lr.w - 28.0 - 16.0) / 3.0;
    for (k, &p) in schluessel.iter().take(3).enumerate() {
        let pl = &c.world.players[p];
        let kr = Rect::new(lr.x + 14.0 + k as f32 * (dw + 8.0), dy + 14.0, dw, (lr.y + lr.h - dy - 26.0).max(80.0));
        rrect(kr.x, kr.y, kr.w, kr.h, 8.0, Theme::erhoehung());
        rrect_line(kr.x, kr.y, kr.w, kr.h, 8.0, alpha(Theme::gold(), 0.6));
        ui.player_photo(kr.x + kr.w / 2.0, kr.y + 28.0, 20.0, pl.id, &pl.name);
        ui.text_center(&ui.fit(pl.name.split_whitespace().last().unwrap_or(&pl.name), 13.0, kr.w - 8.0), kr.x + kr.w / 2.0, kr.y + 64.0, 13.0, Theme::text());
        ui.pos_tag(kr.x + kr.w / 2.0 - 18.0, kr.y + 80.0, pos_de(&pl.pos));
        if kr.h > 100.0 { ui.text_center(&crate::tf!("ui.duell.tore_vorlagen", pl.goals, pl.assists), kr.x + kr.w / 2.0, kr.y + 104.0, 11.0, Theme::text2()); }
    }
    // middle: the numbers them against us, then their last five results
    let mr = Rect::new(r.x + sw + 12.0, r.y, r.w - 2.0 * sw - 24.0, r.h);
    let zr = Rect::new(mr.x, mr.y, mr.w, mr.h * 0.62);
    ui.card_titled(zr, t("ui.gv.zahlen"), Theme::akzent());
    let (zg, zu) = (c.verein_zahlen(g), c.verein_zahlen(c.own));
    ui.club_crest(zr.x + zr.w * 0.38, zr.y + 66.0, 12.0, c.world.clubs[g].id, gc, &c.world.clubs[g].short);
    ui.club_crest(zr.x + zr.w * 0.72, zr.y + 66.0, 12.0, c.world.clubs[c.own].id, uc, &c.world.clubs[c.own].short);
    let namen = ["ui.gv.tore", "ui.gv.gegentore", "ui.gv.punkte", "ui.gv.xg", "ui.gv.schuesse", "ui.gv.passquote", "ui.gv.tacklings"];
    let zh = ((zr.h - 120.0) / 7.0).clamp(30.0, 46.0);
    for (k, key) in namen.iter().enumerate() {
        let y = zr.y + 96.0 + k as f32 * zh;
        ui.text(&ui.fit(t(key), 13.0, zr.w * 0.3 - 20.0), zr.x + 14.0, y, 13.0, Theme::text2());
        let max = zg[k].max(zu[k]).max(0.01);
        let bw = zr.w * 0.3;
        for (j, (wert, col)) in [(zg[k], gc), (zu[k], uc)].into_iter().enumerate() {
            let x = zr.x + zr.w * (0.3 + 0.34 * j as f32);
            ui.bar(x, y + 8.0, bw - 10.0, 8.0, wert / max, col);
            ui.text(&if k == 5 { format!("{wert:.0} %") } else { format!("{wert:.2}") }, x, y + 4.0, 13.0, Theme::text());
        }
    }
    ui.text(t("ui.gv.je_spiel"), zr.x + 14.0, zr.y + zr.h - 14.0, 12.0, Theme::gedaempft());
    let er = Rect::new(mr.x, mr.y + zr.h + 12.0, mr.w, mr.h - zr.h - 12.0);
    ui.card_titled(er, t("ui.duell.letzte"), gc);
    let l = c.world.clubs[g].league;
    let mut fx: Vec<crate::world::Fixture> = c.world.leagues[l].fixtures.iter().filter(|f| f.hg >= 0 && (f.h == g || f.a == g)).cloned().collect();
    fx.sort_by_key(|f| std::cmp::Reverse(f.day));
    let mut siege = 0;
    for (i, f) in fx.iter().take(5).enumerate() {
        let y = er.y + 70.0 + i as f32 * 26.0;
        if y > er.y + er.h - 8.0 { break; }
        let heim = f.h == g;
        let (gg, ga) = if heim { (f.hg, f.ag) } else { (f.ag, f.hg) };
        if gg > ga { siege += 1; }
        let geg = if heim { f.a } else { f.h };
        let col = if gg > ga { Theme::gut() } else if gg == ga { Theme::achtung() } else { Theme::problem() };
        rrect(er.x + 14.0, y - 14.0, 40.0, 19.0, 4.0, col);
        ui.text_center(&format!("{gg}:{ga}"), er.x + 34.0, y + 1.0, 13.0, hex(0xffffff));
        ui.club_crest(er.x + 72.0, y - 4.0, 8.0, c.world.clubs[geg].id, club_color(c, geg).0, &c.world.clubs[geg].short);
        ui.text(&ui.fit(&format!("{} {}", if heim { t("ui.duell.gegen") } else { "@" }, c.world.clubs[geg].name), 13.0, er.w - 110.0), er.x + 88.0, y + 1.0, 13.0, Theme::text()); // roh
    }
    // right: the assistant's assessment in his words, then his suggestions to apply
    let rr = Rect::new(r.x + r.w - sw, r.y, sw, r.h);
    ui.card_titled(rr, &crate::tf!("ui.gv.co_titel", c.co_name.clone()), Theme::gold());
    ui.figur(rr.x + 40.0, rr.y + 86.0, 26.0, &c.co_name.clone(), crate::ui::Rolle::Stab);
    let einschaetzung = c.co_einschaetzung(g, siege.min(fx.len().min(5)), &zeilen);
    let mut y = rr.y + 70.0;
    for satz in &einschaetzung {
        let n = ui.text_passend(&crate::tf!("ui.duell.zitat", satz.clone()), rr.x + 78.0, y, 14.0, rr.w - 92.0, 3, Theme::text());
        y += 10.0 + 19.0 * n as f32;
    }
    y = y.max(rr.y + 130.0) + 6.0;
    draw_rectangle(rr.x + 14.0, y, rr.w - 28.0, 1.0, Theme::kontur());
    ui.text_semi(t("ui.gv.vorschlaege"), rr.x + 14.0, y + 22.0, 13.0, Theme::gedaempft());
    y += 30.0;
    let vs = c.gegner_vorschlaege();
    if vs.is_empty() { ui.paragraph(t("ui.gv.nichts"), rr.x + 14.0, y + 16.0, 14.0, rr.w - 28.0, Theme::text2()); }
    for (text, s) in vs {
        if y > rr.y + rr.h - 70.0 { break; }
        let n = ui.text_passend(&text, rr.x + 14.0, y + 16.0, 13.0, rr.w - 28.0, 3, Theme::text());
        let by = y + 20.0 + 17.0 * n as f32;
        if ui.button(Rect::new(rr.x + 14.0, by, 150.0, 28.0), t("ui.gv.umsetzen"), true) { let m = c.vorschlag_umsetzen(s); v.toast(&m, true); }
        y = by + 40.0;
    }
}
// ================================================================ Vorbereitung (Punkt 14)
/// Friendlies, trips with tournaments and team events: plan them, see them, enter the score of one played in FL27.
pub(super) fn vorbereitung_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let (bw, bh) = ((cbreite() - 12.0) / 2.0, (hoehe() - Y0 - 28.0) / 2.0);
    let felder = [Rect::new(x0() + 4.0, Y0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0, bw, bh), Rect::new(x0() + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
    // dates: friendlies with simulate / play in FL27 (default simulate), trips; results
    let r = felder[0];
    ui.card_titled(r, t("ui.vorb.termine"), Theme::akzent());
    if c.lager_laeuft().is_some() && ui.button(Rect::new(r.x + r.w - 230.0, r.y + 4.0, 220.0, 28.0), t("ui.lager.vorspulen"), true) { match c.lager_vorspulen() { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
    let mut ix: Vec<usize> = (0..c.termine.len()).collect();
    ix.sort_by_key(|&i| (c.termine[i].ergebnis.is_some(), c.termine[i].tag));
    if ix.is_empty() { ui.paragraph(t("ui.vorb.keine_termine"), r.x + 14.0, r.y + 64.0, 15.0, r.w - 28.0, Theme::text2()); }
    let mut y = r.y + 50.0;
    for i in ix {
        if y + 34.0 > r.y + r.h - 6.0 { break; }
        let tm = c.termine[i].clone();
        let was = if tm.art == 2 { crate::vorbereitung::lager_orte().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["name"].as_str()).unwrap_or("").to_string() } else if tm.art == 0 { crate::tf!("ui.vorb.testspiel_gegen", c.world.clubs[tm.gegner].name) } else { crate::vorbereitung::turniere().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["name"].as_str()).unwrap_or("").to_string() };
        ui.text(&date::short(c.world.season, tm.tag), r.x + 14.0, y + 20.0, 14.0, Theme::text2());
        ui.text(&ui.fit(&was, 15.0, r.w * 0.42), r.x + 90.0, y + 20.0, 15.0, Theme::text());
        let rx = r.x + r.w * 0.6;
        if let Some(_) = tm.ergebnis { ui.text(&ui.fit(&tm.text, 13.0, r.w * 0.38), rx, y + 20.0, 13.0, Theme::text2()); }
        else if tm.art == 0 && tm.selbst && tm.tag <= c.world.day {
            // played in FL27: the score by hand
            let (a, b) = v.vorb_tore;
            if ui.button(Rect::new(rx, y + 2.0, 26.0, 26.0), "+", false) { v.vorb_tore.0 = (a + 1).min(9); }
            ui.text_center(&format!("{a}:{b}"), rx + 46.0, y + 20.0, 16.0, Theme::text());
            if ui.button(Rect::new(rx + 66.0, y + 2.0, 26.0, 26.0), "+", false) { v.vorb_tore.1 = (b + 1).min(9); }
            if ui.button(Rect::new(rx + 98.0, y + 2.0, r.x + r.w - rx - 112.0, 26.0), t("ui.vorb.eintragen"), true) { c.testspiel_eintragen(i, a, b); v.vorb_tore = (0, 0); }
        } else if tm.art == 0 {
            if let Some(k) = ui.segment(Rect::new(rx, y + 2.0, r.x + r.w - rx - 14.0, 26.0), &[t("ui.vorb.simulieren"), t("ui.vorb.selbst")], tm.selbst as usize) { c.termine[i].selbst = k == 1; c.testspiel_als_spieltag(i); }
        } else { ui.text(t("ui.vorb.geplant"), rx, y + 20.0, 13.0, Theme::gedaempft()); }
        y += 34.0;
    }
    // ask for a friendly: a free day, then one of three opponents
    let r = felder[1];
    ui.card_titled(r, t("ui.vorb.anfragen"), Theme::gut());
    let tage = c.freie_tage(5, 1);
    if tage.is_empty() { ui.text(t("ui.vorb.kein_tag"), r.x + 14.0, r.y + 64.0, 15.0, Theme::text2()); } else {
        v.vorb_tag = v.vorb_tag.min(tage.len() - 1);
        let namen: Vec<String> = tage.iter().map(|d| date::short(c.world.season, *d)).collect();
        let nr: Vec<&str> = namen.iter().map(|s| s.as_str()).collect();
        if let Some(k) = ui.segment(Rect::new(r.x + 14.0, r.y + 48.0, r.w - 28.0, 30.0), &nr, v.vorb_tag) { v.vorb_tag = k; }
        ui.text(t("ui.vorb.abstand"), r.x + 14.0, r.y + 100.0, 13.0, Theme::gedaempft());
        for (k, g) in c.testspiel_gegner().into_iter().enumerate() {
            let b = Rect::new(r.x + 14.0, r.y + 114.0 + k as f32 * 42.0, r.w - 28.0, 36.0);
            let label = crate::tf!("ui.vorb.gegner_zeile", c.world.clubs[g].name, stufe_wort(crate::mensch::kaderstaerke(&c.world, g)));
            if ui.button(b, &label, false) { match c.testspiel_anfragen(g, tage[v.vorb_tag]) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
        }
    }
    // trips with a small tournament (turniere.toml: real and invented names)
    let r = felder[2];
    ui.card_titled(r, t("ui.vorb.reisen"), Theme::gold());
    if let Some(k) = ui.segment(Rect::new(r.x + r.w - 330.0, r.y + 6.0, 316.0, 26.0), &[t("ui.vorb.reisen"), t("ui.lager.titel")], v.lager_reiter as usize) { v.lager_reiter = k as u8; }
    if v.lager_reiter == 1 { lager_karte(ui, v, c, r); } else {
    let mut y = r.y + 50.0;
    for tu in crate::vorbereitung::turniere() {
        if y + 40.0 > r.y + r.h - 6.0 { break; }
        let id = tu["id"].as_str().unwrap_or("");
        let tage = tu["tage"].as_u64().unwrap_or(5) as u32;
        let kennz = if tu["echt"].as_bool().unwrap_or(false) { t("ui.vorb.echt") } else { t("ui.vorb.erfunden") };
        ui.text(&ui.fit(tu["name"].as_str().unwrap_or(id), 15.0, r.w * 0.55), r.x + 14.0, y + 16.0, 15.0, Theme::text());
        ui.text(&ui.fit(&crate::tf!("ui.vorb.reise_zeile", kennz, tu["region"].as_str().unwrap_or(""), tage, tu["teams"].as_u64().unwrap_or(4), money(tu["kosten"].as_f64().unwrap_or(0.0))), 12.0, r.w * 0.62), r.x + 14.0, y + 33.0, 12.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 124.0, y + 4.0, 110.0, 30.0), t("ui.vorb.buchen"), false) {
            match c.freie_tage(1, tage).first() { Some(&d) => match c.reise_buchen(id, d) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }, None => v.toast(t("ui.vorb.kein_tag"), false) }
        }
        y += 42.0;
    }
    }
    // team events
    let r = felder[3];
    ui.card_titled(r, t("ui.vorb.teamevents"), hex(0xa78bfa));
    ui.text(&crate::tf!("ui.vorb.summand", format!("{:+.1}", c.event_summand())), r.x + 14.0, r.y + 58.0, 13.0, Theme::gedaempft());
    let abs = c.abs();
    let mut y = r.y + 70.0;
    for e in crate::vorbereitung::events() {
        if y + 40.0 > r.y + r.h - 6.0 { break; }
        let id = e["id"].as_str().unwrap_or("");
        let frei = c.teamevents.iter().find(|x| x.0 == id).map(|x| x.1 + e["sperre"].as_u64().unwrap_or(30) as u32).unwrap_or(0);
        ui.text(t(&format!("ui.vorb.event.{id}")), r.x + 14.0, y + 16.0, 15.0, Theme::text());
        let zeile = if abs < frei { crate::tf!("ui.vorb.event_gesperrt", frei - abs) } else { crate::tf!("ui.vorb.event_zeile", money(e["kosten"].as_f64().unwrap_or(0.0)), format!("{:.0}", e["teamgeist"].as_f64().unwrap_or(0.0))) };
        ui.text(&ui.fit(&zeile, 12.0, r.w * 0.6), r.x + 14.0, y + 33.0, 12.0, Theme::text2());
        if abs >= frei && ui.button(Rect::new(r.x + r.w - 124.0, y + 4.0, 110.0, 30.0), t("ui.vorb.ausrichten"), false) {
            match c.teamevent(id) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
        }
        y += 42.0;
    }
}


/// Training camps (Konzept Teil 4): the places that fit now with costs, the length, booking on the next free days.
fn lager_karte(ui: &mut Ui, v: &mut View, c: &mut Career, r: Rect) {
    use crate::inhalt::t;
    let laengen = [3u32, 5, 7, 10];
    let namen: Vec<String> = laengen.iter().map(|n| crate::tf!("ui.lager.tage", n)).collect();
    let nr: Vec<&str> = namen.iter().map(|s| s.as_str()).collect();
    let sel = laengen.iter().position(|&n| n == v.lager_tage).unwrap_or(2);
    if let Some(k) = ui.segment(Rect::new(r.x + 14.0, r.y + 44.0, r.w - 28.0, 28.0), &nr, sel) { v.lager_tage = laengen[k]; }
    let tage = laengen[sel];
    let mut y = r.y + 82.0;
    let orte = c.lager_passend();
    if orte.is_empty() { ui.text(t("ui.lager.keine"), r.x + 14.0, y + 16.0, 15.0, Theme::text2()); }
    for o in orte {
        if y + 40.0 > r.y + r.h - 6.0 { break; }
        let id = o["id"].as_str().unwrap_or("");
        ui.text(&ui.fit(o["name"].as_str().unwrap_or(id), 15.0, r.w * 0.58), r.x + 14.0, y + 16.0, 15.0, Theme::text());
        let klima = t(&format!("ui.lager.klima.{}", o["klima"].as_str().unwrap_or("mild")));
        let zeile = crate::tf!("ui.lager.zeile", klima, o["plaetze"].as_u64().unwrap_or(3), o["ruhe"].as_u64().unwrap_or(3), money(c.lager_kosten(id, tage)));
        ui.text(&ui.fit(&zeile, 12.0, r.w * 0.62), r.x + 14.0, y + 33.0, 12.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 124.0, y + 4.0, 110.0, 30.0), t("ui.vorb.buchen"), false) {
            match c.freie_tage(1, tage).first() { Some(&d) => match c.lager_buchen(id, d, tage) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }, None => v.toast(t("ui.vorb.kein_tag"), false) }
        }
        y += 42.0;
    }
}

pub(super) fn vorbereitung(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    match c.gegnerbericht() {
        Some((gn, z)) => { ui.card_titled(l, &crate::tf!("ui.seiten.naechster_gegner", gn), Theme::akzent()); for (k, x) in z.iter().enumerate() { ui.paragraph(x, l.x + 14.0, l.y + 64.0 + k as f32 * 44.0, 15.0, l.w - 28.0, Theme::text()); } }
        None => { ui.card_titled(l, crate::inhalt::t("ui.seiten.kein_spiel_in_sicht"), Theme::akzent()); }
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.startelf_und_zustand"), Theme::gold());
    let z: Vec<(String, String)> = c.current_xi().iter().map(|(s, p, _)| { let pl = &c.world.players[*p]; (format!("{} {}", pos_de(s), pl.name), crate::tf!("ui.seiten.fitness", if crate::mensch::pfeil(pl.form) >= 3 { crate::inhalt::t("gs.kontext.form.gut") } else if crate::mensch::pfeil(pl.form) <= 1 { crate::inhalt::t("ui.seiten.ausser_form") } else { crate::inhalt::t("ui.wort.normal") }, format!("{:.0}", pl.fit))) }).collect();
    zeilen(ui, r, &z, 64.0);
}
