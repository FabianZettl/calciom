use super::*;

thread_local! { static TAG: std::cell::Cell<Option<(u32, usize)>> = const { std::cell::Cell::new(None) }; }
/// Events of the world the club is not part of (assets/regeln/kalender.toml): name on a given date.
fn welt_termin(y: i32, m: u32, d: u32) -> Option<(String, bool)> {
    crate::regeln::wert_oder("kalender", "termin").and_then(|v| v.as_array().cloned()).unwrap_or_default().iter().find_map(|t| {
        let dat = |k: &str| -> Option<(i32, u32, u32)> { let a = t[k].as_array()?; Some((a.first()?.as_i64()? as i32, a.get(1)?.as_u64()? as u32, a.get(2)?.as_u64()? as u32)) };
        let (von, bis) = (dat("von")?, dat("bis")?);
        ((y, m, d) >= von && (y, m, d) <= bis).then(|| (t["name"].as_str().unwrap_or("").to_string(), (y, m, d) == von))
    })
}

// ================================================================ Kalender
pub(super) fn kalender(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("ui.vorb.reiter_monat"), crate::inhalt::t("ui.vorb.reiter")]);
    if v.tab == 1 { return vorbereitung_seite(ui, v, c); }
    let (_, cm, _) = date::ymd(c.world.season, c.world.day);
    if v.cal_month < 0 { v.cal_month = cm as i32; }
    let m = v.cal_month as u32;
    if ui.button(Rect::new(x0() + cbreite() - 260.0, Y0 - 44.0, 40.0, 32.0), "<", false) { v.cal_month = if m == 1 { 12 } else { m as i32 - 1 }; }
    if ui.button(Rect::new(x0() + cbreite() - 40.0, Y0 - 44.0, 40.0, 32.0), ">", false) { v.cal_month = if m == 12 { 1 } else { m as i32 + 1 }; }
    let mut first = None;
    for d in 0..date::SEASON_DAYS { let (_, mm, dd) = date::ymd(c.world.season, d); if mm == m && dd == 1 { first = Some(d); break; } }
    let Some(f0) = first else { return };
    let (yy, _, _) = date::ymd(c.world.season, f0);
    // the month sits between the arrows, not over the tab pills (Fabian 23.11.)
    let mt = format!("{} {}", date::month_name(m), yy);
    let mw = ui.measure_title(&mt, 24.0);
    ui.title(&mt, x0() + cbreite() - 110.0 - mw / 2.0, Y0 - 20.0, 24.0, Theme::text());
    let cw = (cbreite() - 6.0 * 8.0) / 7.0;
    let ch = (hoehe() - Y0 - 70.0) / 6.0 - 8.0;
    for (i, wdn) in [crate::inhalt::t("training.tag.0"), crate::inhalt::t("training.tag.1"), crate::inhalt::t("training.tag.2"), crate::inhalt::t("training.tag.3"), crate::inhalt::t("training.tag.4"), crate::inhalt::t("training.tag.5"), crate::inhalt::t("training.tag.6")].iter().enumerate() {
        ui.text_semi(wdn, x0() + 4.0 + i as f32 * (cw + 8.0), Y0 + 12.0, 16.0, Theme::text2());
    }
    let off = date::weekday(f0);
    let fx = c.own_fixtures();
    for d in f0..date::SEASON_DAYS {
        let (_, mm, dd) = date::ymd(c.world.season, d);
        if mm != m { break; }
        let cell = off + (dd - 1) as usize;
        let r = Rect::new(x0() + 4.0 + (cell % 7) as f32 * (cw + 8.0), Y0 + 22.0 + (cell / 7) as f32 * (ch + 8.0), cw, ch);
        let window = WINDOWS.iter().any(|(a, b)| d >= *a && d <= *b);
        let intl = INTL_BREAKS.iter().any(|(a, b)| d >= *a && d <= *b);
        rrect(r.x, r.y, r.w, r.h, 6.0, if d == c.world.day { alpha(Theme::akzent(), 0.18) } else if d < c.world.day { Theme::grund_tief() } else { Theme::flaeche() });
        // a match day wears its competition's colour (Fabian 25.11.: which match is it, at a glance)
        if let Some((wo, _)) = fx.iter().find(|x| x.1.day == d) {
            let col = wo_look(c, *wo).c1;
            rrect(r.x, r.y, r.w, r.h, 6.0, alpha(col, 0.20));
            draw_rectangle(r.x, r.y + 6.0, 5.0, r.h - 12.0, col);
        }
        // the world's events we take no part in: dimmed, with their name on the first day
        let (y, mo, dt) = date::ymd(c.world.season, d);
        if let Some((name, erster)) = welt_termin(y, mo, dt) {
            draw_rectangle(r.x, r.y, r.w, 4.0, alpha(Theme::gedaempft(), 0.6));
            if erster && !fx.iter().any(|x| x.1.day == d) { ui.text(&ui.fit(&name, 12.0, r.w - 20.0), r.x + 10.0, r.y + r.h - 26.0, 12.0, Theme::gedaempft()); }
        }
        let gala = [("ballon_gala", "ui.kal.ballon"), ("best_gala", "ui.kal.best")].iter().find(|(k, _)| crate::regeln::f("ligapreise", &format!("termine.{k}")) as u32 == d).map(|x| x.1);
        if let Some(g) = gala { ui.text(&ui.fit(crate::inhalt::t(g), 12.0, r.w - 20.0), r.x + 10.0, r.y + r.h - 26.0, 12.0, Theme::gold()); }
        if intl && INTL_BREAKS.iter().any(|(a, _)| *a == d) { ui.text(&ui.fit(crate::inhalt::t("ui.pages.laenderspielpause"), 12.0, r.w - 20.0), r.x + 10.0, r.y + r.h - 26.0, 12.0, hex(0xB48CFF)); }
        // planned talks
        let abs = date::SEASON_DAYS * c.world.season + d;
        if let Some((_, p, art)) = c.gespraech_termine.iter().find(|x| x.0 == abs) { ui.text(&ui.fit(&crate::tf!(if *art == 1 { "ui.kal.zelle_vertrag" } else { "ui.kal.zelle_gespraech" }, c.world.players[*p].name.clone()), 12.0, r.w - 20.0), r.x + 10.0, r.y + 40.0, 12.0, Theme::akzent()); }
        // a future day can get a talk: click it
        if d > c.world.day && ui.hit(r) { TAG.with(|x| x.set(Some((d, 0)))); }
        if window { draw_rectangle(r.x, r.y + r.h - 4.0, r.w, 4.0, Theme::gold()); }
        if intl { draw_rectangle(r.x, r.y + r.h - 4.0, r.w, 4.0, hex(0xB48CFF)); }
        ui.text_semi(&format!("{dd}"), r.x + 10.0, r.y + 24.0, 18.0, if d == c.world.day { Theme::akzent() } else { Theme::text2() });
        if let Some((wo, f)) = fx.iter().find(|x| x.1.day == d) {
            ui.wettbewerb_logo_auf(r.x + r.w - 16.0, r.y + 17.0, 10.0, c.comp_cid(*wo), Some(wo_look(c, *wo).c1));
            let home = f.h == c.own;
            let opp = if home { f.a } else { f.h };
            let (c1, c2) = club_color(c, opp);
            let _ = c2;
            ui.club_crest(r.x + 26.0, r.y + 48.0, 15.0, c.world.clubs[opp].id, c1, &c.world.clubs[opp].short);
            let nm = ui.fit(&c.world.clubs[opp].name, 15.0, r.w - 56.0);
            ui.text(&nm, r.x + 48.0, r.y + 46.0, 15.0, Theme::text());
            let what = match fx.iter().find(|x| x.1.day == d).map(|x| x.0) { Some(Wo::Pokal(ci)) => c.world.cups[ci].name.clone(), _ => if home { crate::inhalt::t("zentrale.k.heim").into() } else { crate::inhalt::t("zentrale.k.auswaerts").into() } };
            let what = ui.fit(&what, 14.0, r.w - 56.0);
            ui.text(&what, r.x + 48.0, r.y + 63.0, 14.0, if matches!(fx.iter().find(|x| x.1.day == d).map(|x| x.0), Some(Wo::Pokal(_))) { Theme::gold() } else { Theme::gedaempft() });
            if f.hg < 0 { ui.text_right(&crate::ligaregeln::zeit_text(f.zeit), r.x + r.w - 10.0, r.y + ch - 8.0, 14.0, Theme::text2()); }
            if f.hg >= 0 {
                let (gf, ga) = if home { (f.hg, f.ag) } else { (f.ag, f.hg) };
                let col = if gf > ga { Theme::gut() } else if gf < ga { Theme::problem() } else { Theme::achtung() };
                ui.text_semi(&format!("{}:{}", f.hg, f.ag), r.x + 10.0, r.y + ch - 8.0, 17.0, col);
            }
        } else if let Some(tm) = c.termine.iter().find(|tm| tm.art == 0 && tm.tag == d) {
            // a friendly (Vorbereitung): opponent, „Testspiel“, the score when played
            let g = tm.gegner;
            let (c1, _) = club_color(c, g);
            ui.club_crest(r.x + 26.0, r.y + 48.0, 15.0, c.world.clubs[g].id, c1, &c.world.clubs[g].short);
            let nm = ui.fit(&c.world.clubs[g].name, 15.0, r.w - 56.0);
            ui.text(&nm, r.x + 48.0, r.y + 46.0, 15.0, Theme::text());
            ui.text(crate::inhalt::t("ui.vorb.testspiel"), r.x + 48.0, r.y + 63.0, 14.0, hex(0x2CC5B5));
            if let Some((a, b)) = tm.ergebnis { ui.text_semi(&format!("{a}:{b}"), r.x + 10.0, r.y + ch - 8.0, 17.0, Theme::text2()); }
        } else if let Some(tm) = c.termine.iter().find(|tm| tm.art == 2 && d >= tm.tag && d < tm.tag + tm.tage) {
            // a training camp: a bar over its days, the place on the first
            draw_rectangle(r.x, r.y, r.w, 5.0, Theme::gold());
            if d == tm.tag {
                let name = crate::vorbereitung::lager_orte().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["name"].as_str()).unwrap_or("");
                ui.paragraph(&ui.fit(name, 14.0, (r.w - 20.0) * 2.0), r.x + 10.0, r.y + 46.0, 14.0, r.w - 20.0, Theme::text());
            }
        } else if let Some(tm) = c.termine.iter().find(|tm| tm.art == 1 && d >= tm.tag && d < tm.tag + crate::vorbereitung::turniere().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["tage"].as_u64()).unwrap_or(5) as u32) {
            // a trip with its tournament: a bar over all its days, the name on the first
            draw_rectangle(r.x, r.y, r.w, 5.0, hex(0x2CC5B5));
            if d == tm.tag {
                let name = crate::vorbereitung::turniere().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["name"].as_str()).unwrap_or("");
                ui.paragraph(&ui.fit(name, 14.0, (r.w - 20.0) * 2.0), r.x + 10.0, r.y + 46.0, 14.0, r.w - 20.0, Theme::text());
            }
        } else if c.plan[date::weekday(d)] != 3 || true {
            ui.text(units_t()[c.plan[date::weekday(d)] as usize], r.x + 10.0, r.y + ch - 10.0, 13.0, Theme::gedaempft());
        }
    }
    // planning a talk for the chosen day: a player and the kind of talk
    if let Some((tag, wahl)) = TAG.with(|x| x.get()) {
        let sq = c.squad();
        if !sq.is_empty() {
            let p = sq[wahl % sq.len()];
            let bar = Rect::new(x0() + 520.0, hoehe() - 48.0, cbreite() - 520.0, 40.0);
            rrect(bar.x, bar.y, bar.w, bar.h, 8.0, Theme::erhoehung());
            ui.text_semi(&crate::tf!("ui.kal.planen", date::long(c.world.season, tag)), bar.x + 12.0, bar.y + 26.0, 14.0, Theme::text());
            let px = bar.x + 250.0;
            if ui.button(Rect::new(px, bar.y + 6.0, 28.0, 28.0), "<", false) { TAG.with(|x| x.set(Some((tag, wahl + sq.len() - 1)))); }
            ui.text_center(&ui.fit(&c.world.players[p].name, 14.0, 180.0), px + 124.0, bar.y + 26.0, 14.0, Theme::text());
            if ui.button(Rect::new(px + 220.0, bar.y + 6.0, 28.0, 28.0), ">", false) { TAG.with(|x| x.set(Some((tag, wahl + 1)))); }
            let abs = date::SEASON_DAYS * c.world.season + tag;
            for (k, (art, key)) in [(0u8, "ui.kal.knopf_gespraech"), (1u8, "ui.kal.knopf_vertrag")].iter().enumerate() {
                if ui.button(Rect::new(px + 260.0 + k as f32 * 150.0, bar.y + 6.0, 140.0, 28.0), crate::inhalt::t(key), false) {
                    c.gespraech_termine.retain(|x| !(x.0 == abs && x.1 == p));
                    c.gespraech_termine.push((abs, p, *art));
                    v.toast(&crate::tf!("ui.kal.geplant", c.world.players[p].name.clone(), date::long(c.world.season, tag)), true);
                    TAG.with(|x| x.set(None));
                }
            }
            if ui.button(Rect::new(bar.x + bar.w - 36.0, bar.y + 6.0, 28.0, 28.0), "×", false) { TAG.with(|x| x.set(None)); } // roh
        }
    }
    let ly = hoehe() - 26.0;
    draw_rectangle(x0() + 8.0, ly - 8.0, 20.0, 4.0, Theme::gold()); ui.text(crate::inhalt::t("termin.transferfenster"), x0() + 34.0, ly, 15.0, Theme::text2());
    draw_rectangle(x0() + 168.0, ly - 8.0, 20.0, 4.0, hex(0xB48CFF)); ui.text(crate::inhalt::t("ui.pages.laenderspielpause"), x0() + 194.0, ly, 15.0, Theme::text2());
    draw_rectangle(x0() + 368.0, ly - 8.0, 20.0, 4.0, hex(0x2CC5B5)); ui.text(crate::inhalt::t("ui.vorb.reiter"), x0() + 394.0, ly, 15.0, Theme::text2());
}
