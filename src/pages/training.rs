use super::*;

// ================================================================ Training
pub(super) fn training(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("menu.training.reiter.0"), crate::inhalt::t("gm.tr.kz.einzel"), crate::inhalt::t("ui.main.trainerteam")]);
    if v.tab == 2 { trainerteam(ui, v, c); return; }
    let y = Y0;
    let w4 = (cbreite() - 36.0) / 4.0;
    let days = c.plan.iter().filter(|&&u| u >= 2).count();
    let tired = c.squad().iter().filter(|&&p| c.world.players[p].fit < 60.0).count();
    let progs = c.squad().iter().filter(|&&p| c.world.players[p].program > 0).count();
    kpi(ui, Rect::new(x0() + 4.0, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_trainingstage"), &format!("{days}"), crate::inhalt::t("gm.tr.kz.tage.unter"), Theme::gut());
    kpi(ui, Rect::new(x0() + 16.0 + w4, y, w4, 92.0), crate::inhalt::t("ui.pages.hohe_belastung"), &format!("{tired}"), if tired == 0 { crate::inhalt::t("gm.tr.kz.belastung.ok") } else { crate::inhalt::t("ui.pages.spieler_unter_60_fitness") }, if tired == 0 { Theme::gut() } else { Theme::achtung() });
    kpi(ui, Rect::new(x0() + 28.0 + 2.0 * w4, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_einzelprogramme"), &crate::tf!("ui.pages.6", progs), crate::inhalt::t("ui.pages.fokus_oder_extraschicht"), Theme::akzent());
    kpi(ui, Rect::new(x0() + 40.0 + 3.0 * w4, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_eingespielt"), &format!("{:.0} %", c.club().fam), crate::inhalt::t("ui.pages.taktiktraining_hilft"), Theme::akzent());
    let y = y + 106.0;
    let mut co = c.co_training;
    rrect(x0() + 4.0, y, 120.0, 24.0, 12.0, Theme::erhoehung());
    ui.text(if co { crate::inhalt::t("ui.pages.co_trainer_plant") } else { crate::inhalt::t("modus.manuell") }, x0() + 14.0, y + 17.0, 14.0, Theme::text2());
    if ui.button(Rect::new(x0() + 134.0, y - 2.0, 230.0, 28.0), if co { crate::inhalt::t("ui.pages.selbst_planen") } else { crate::inhalt::t("ui.pages.an_den_co_trainer_abgeben") }, false) { co = !co; c.co_training = co; if co { c.co_wochenplan(); v.tp_zelle = None; } }
    // what the plan does for development: intensity × repetition (entwicklung.rs plan_faktor), and the legend of the list
    ui.text(&if v.tab == 0 { crate::tf!("ui.ent.plan_wirkung", format!("{:.2}", c.plan_faktor())) } else { crate::inhalt::t("ui.ent.legende").to_string() }, x0() + 380.0, y + 17.0, 14.0, Theme::gedaempft());
    if v.tab == 0 {
        trainingswoche(ui, v, c, y + 36.0);
    } else {
        let sq = c.squad();
        let area = Rect::new(x0() + 4.0, y + 36.0, cbreite(), hoehe() - y - 52.0);
        ui.card(area);
        let vis = ((area.h - 10.0) / 40.0) as usize;
        let start = scroll_rows(ui, v, area, sq.len(), vis);
        let used = sq.iter().filter(|&&p| c.world.players[p].program > 0).count();
        for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
            let r = Rect::new(area.x + 6.0, area.y + 6.0 + k as f32 * 40.0, area.w - 12.0, 36.0);
            let (name, pos, age, ovr, pot, prog) = { let pl = &c.world.players[p]; (pl.name.clone(), pl.pos.clone(), pl.age, pl.ovr, pl.pot, pl.program) };
            ui.pos_tag(r.x + 6.0, r.y + 18.0, pos_de(&pos));
            ui.text_semi(&name, r.x + 50.0, r.y + 24.0, 17.0, Theme::text());
            ui.text(&crate::tf!("kue.jahre", age), r.x + 290.0, r.y + 24.0, 16.0, Theme::text2());
            ui.pill(r.x + 360.0, r.y + 18.0, ovr, 36.0);
            ui.text(&crate::tf!("aka.talent.potenzial", format!("{:.0}", pot)), r.x + 390.0, r.y + 24.0, 15.0, Theme::text2());
            let opts: &[&str] = if age <= 23 { &[crate::inhalt::t("ui.pages.kein_programm"), crate::inhalt::t("ui.pages.fokus"), crate::inhalt::t("ui.pages.extraschicht")] } else { &[crate::inhalt::t("ui.pages.kein_programm"), crate::inhalt::t("ui.pages.fokus")] };
            if let Some(s) = ui.segment(Rect::new(r.x + 500.0, r.y + 3.0, 300.0, 30.0), opts, prog as usize) {
                if s == 0 || prog > 0 || used < 6 { c.world.players[p].program = s as u8; } else { v.toast(crate::inhalt::t("ui.pages.hoechstens_sechs_einzelprogramme_gleichz"), false); }
            }
            // focus group of the programme (×1,7 on its abilities, ×0,95 on the rest); a click goes to the next group
            if prog == 1 {
                let fg = crate::entwicklung::fokusgruppen();
                let name = crate::entwicklung::fokus_von(&c.world.players[p]);
                let auto = c.world.players[p].ent.fokus == 0;
                let txt = crate::tf!(if auto { "ui.ent.fokus_auto" } else { "ui.ent.fokus_gewaehlt" }, crate::inhalt::t(&format!("ui.ent.fokus.{name}")));
                if ui.button(Rect::new(r.x + 810.0, r.y + 3.0, 116.0, 30.0), &ui.fit(&txt, 14.0, 108.0), false) { let e = &mut c.world.players[p].ent; e.fokus = (e.fokus + 1) % (fg.len() as u8 + 1); }
            } else if prog == 2 { ui.text(&ui.fit(crate::inhalt::t("ui.ent.extraschicht"), 13.0, 120.0), r.x + 812.0, r.y + 24.0, 13.0, Theme::gedaempft()); }
            // load as a traffic light (entwicklung.toml belastung.zonen) and the expected development per season
            let last = c.world.players[p].ps.last;
            let zone = crate::mensch::arr("entwicklung", "belastung.zonen").iter().position(|z| last < z[1].as_f64().unwrap_or(101.0) as f32).unwrap_or(3);
            let zc = [Theme::gedaempft(), Theme::gut(), Theme::achtung(), Theme::problem()][zone.min(3)];
            draw_circle(r.x + 940.0, r.y + 18.0, 6.0, zc);
            ui.text(crate::inhalt::t(&format!("ui.ent.zone.{zone}")), r.x + 952.0, r.y + 23.0, 13.0, Theme::text2());
            entwicklung_pfeil(ui, r.x + 1040.0, r.y + 18.0, c.ent_erwartung(p));
        }
    }
}

/// Colour of a training group.
fn tp_farbe(g: &str) -> Color {
    match g { "technik" => Theme::akzent(), "standards" => Theme::gold(), "defensive" => Theme::problem(), "athletik" => Theme::achtung(), "torwart" => hex(0xB48CFF), "taktik" => Theme::gut(), _ => Theme::gedaempft() }
}

/// The training week: seven days of three sessions; a click on a session and then on a unit fills it, a click on the day
/// type changes it, the intensity chip cycles schonend / normal / hart (trainingsplan.rs).
fn trainingswoche(ui: &mut Ui, v: &mut View, c: &mut Career, y: f32) {
    use crate::trainingsplan as tp;
    c.wochenplan_an();
    let frei = !c.co_training;
    let pw = 330.0;
    let gw = cbreite() - pw - 12.0;
    let grid = Rect::new(x0() + 4.0, y, gw, 64.0 + 3.0 * 70.0 + 40.0);
    ui.card(grid);
    let lx = 104.0;
    let dw = (gw - lx - 12.0) / 7.0;
    let tage: Vec<&str> = (0..7).map(|d| crate::inhalt::t(&format!("training.tag.{d}"))).collect();
    let heute = date::weekday(c.world.day);
    for (s, k) in [crate::id("vormittag"), crate::id("nachmittag"), crate::id("abend")].iter().enumerate() {
        ui.text_semi(crate::inhalt::t(&format!("tp.slot.{k}")), grid.x + 12.0, grid.y + 64.0 + s as f32 * 70.0 + 38.0, 14.0, Theme::text2());
    }
    let lasten = tp::tageslast(&c.wochenplan, true);
    let mut geaendert = false;
    for d in 0..7 {
        let x = grid.x + lx + d as f32 * dw;
        ui.text_semi(tage[d], x + 6.0, grid.y + 24.0, 16.0, if d == heute { Theme::akzent() } else { Theme::text() });
        let typ = c.wochenplan.tage[d].typ;
        let tr = Rect::new(x + 4.0, grid.y + 32.0, dw - 8.0, 24.0);
        rrect(tr.x, tr.y, tr.w, tr.h, 12.0, alpha(if typ == 1 { Theme::gold() } else if typ == 3 { Theme::gedaempft() } else if typ == 2 { Theme::gut() } else { Theme::akzent() }, 0.18));
        ui.text_center(&ui.fit(crate::inhalt::t(&format!("tp.typ.{}", tp::TYPEN[typ as usize])), 13.0, tr.w - 8.0), tr.x + tr.w / 2.0, tr.y + 17.0, 13.0, Theme::text());
        if frei && ui.hit(tr) { tp::typ_setzen(&mut c.wochenplan.tage[d], (typ + 1) % 4); geaendert = true; }
        for s in 0..tp::SLOTS {
            let r = Rect::new(x + 4.0, grid.y + 64.0 + s as f32 * 70.0, dw - 8.0, 64.0);
            let sel = v.tp_zelle == Some((d, s));
            let geht = typ != 3 && !(typ == 1 && s > 0);
            match c.wochenplan.tage[d].slots[s].clone() {
                Some(z) => {
                    let col = tp_farbe(tp::gruppe(&z.einheit));
                    rrect(r.x, r.y, r.w, r.h, 6.0, alpha(col, 0.14));
                    draw_rectangle(r.x, r.y + 6.0, 3.0, r.h - 12.0, col);
                    ui.text_passend(tp::name(&z.einheit), r.x + 10.0, r.y + 20.0, 14.0, r.w - 14.0, 2, Theme::text());
                    let ch = Rect::new(r.x + 8.0, r.y + r.h - 22.0, (r.w - 16.0).min(86.0), 18.0);
                    rrect(ch.x, ch.y, ch.w, ch.h, 9.0, alpha([Theme::gut(), Theme::akzent(), Theme::problem()][z.stufe.min(2) as usize], 0.25));
                    ui.text_center(crate::inhalt::t(&format!("tp.intensitaet.{}", tp::STUFEN[z.stufe.min(2) as usize])), ch.x + ch.w / 2.0, ch.y + 13.0, 12.0, Theme::text());
                    if frei && ui.hit(ch) {
                        if let Some(z) = c.wochenplan.tage[d].slots[s].as_mut() { z.stufe = (z.stufe + 1) % 3; }
                        geaendert = true;
                    }
                }
                None => {
                    rrect_line(r.x, r.y, r.w, r.h, 6.0, if geht { Theme::kontur() } else { alpha(Theme::kontur(), 0.3) });
                    if geht && frei { ui.text_center("+", r.x + r.w / 2.0, r.y + r.h / 2.0 + 8.0, 22.0, Theme::gedaempft()); } // roh
                }
            }
            if sel { rrect_line(r.x - 1.0, r.y - 1.0, r.w + 2.0, r.h + 2.0, 6.0, Theme::akzent()); }
            if frei && geht && ui.hit(r) { v.tp_zelle = if sel { None } else { Some((d, s)) }; }
        }
        // the day's load
        let l = lasten[d];
        let by = grid.y + 64.0 + 3.0 * 70.0 + 6.0;
        ui.bar(x + 6.0, by, dw - 12.0, 6.0, (l / 12.0).clamp(0.0, 1.0), if l <= 0.0 { Theme::gut() } else if l <= 4.0 { Theme::akzent() } else if l <= 8.0 { Theme::achtung() } else { Theme::problem() });
        let stufe = if l <= 0.0 { crate::id("erholt") } else if l <= 3.0 { crate::id("leicht") } else if l <= 7.0 { crate::id("mittel") } else { crate::id("hoch") };
        ui.text(crate::inhalt::t(&format!("tp.last.{stufe}")), x + 6.0, by + 22.0, 12.0, Theme::gedaempft());
    }
    // what the week brings: the abilities it trains most
    let wr = Rect::new(x0() + 4.0, grid.y + grid.h + 12.0, gw, hoehe() - grid.y - grid.h - 28.0);
    ui.card(wr);
    ui.text_semi(crate::inhalt::t("tp.ui.wirkung"), wr.x + 14.0, wr.y + 26.0, 16.0, Theme::text());
    let pk = tp::punkte(&c.wochenplan);
    let mut rang: Vec<(usize, f32)> = pk.iter().copied().enumerate().filter(|x| x.1 > 0.0).collect();
    rang.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let maxp = rang.first().map_or(1.0, |x| x.1);
    let zeilen = (((wr.h - 44.0) / 24.0) as usize).max(1);
    let spw = (wr.w - 28.0) / 2.0;
    for (k, (i, p)) in rang.iter().take(zeilen * 2).enumerate() {
        let (cx, cy) = (wr.x + 14.0 + (k / zeilen) as f32 * spw, wr.y + 52.0 + (k % zeilen) as f32 * 24.0);
        ui.text(&ui.fit(crate::inhalt::t(&format!("faehigkeit.{}", crate::entwicklung::NAMEN[*i])), 14.0, 150.0), cx, cy, 14.0, Theme::text2());
        ui.bar(cx + 160.0, cy - 9.0, spw - 220.0, 8.0, p / maxp, Theme::akzent());
    }
    if rang.is_empty() { ui.text(crate::inhalt::t("ui.tp.bericht.nichts"), wr.x + 14.0, wr.y + 54.0, 14.0, Theme::gedaempft()); }
    // right: the units for the chosen session, else the ready-made weeks and the load of the week
    let pr = Rect::new(x0() + gw + 16.0, y, pw - 4.0, hoehe() - y - 16.0);
    ui.card(pr);
    let mut yy = pr.y + 26.0;
    if let Some((d, s)) = v.tp_zelle.filter(|_| frei) {
        let typ = c.wochenplan.tage[d].typ;
        ui.text_semi(&format!("{} · {}", tage[d], crate::inhalt::t(&format!("tp.slot.{}", [crate::id("vormittag"), crate::id("nachmittag"), crate::id("abend")][s]))), pr.x + 14.0, yy, 16.0, Theme::text());
        yy += 12.0;
        let mut kat = tp::katalog();
        kat.sort_by_key(|id| (tp::gruppe(id), tp::name(id)));
        let mut hover = None;
        for id in kat {
            let r = Rect::new(pr.x + 8.0, yy, pr.w - 16.0, 26.0);
            let geht = tp::passt(typ, s, id);
            let an = c.wochenplan.tage[d].slots[s].as_ref().map_or(false, |z| z.einheit == id);
            if an { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::akzent(), 0.14)); } else if geht && ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::ueberlagerung()); hover = Some(id); }
            draw_circle(r.x + 10.0, r.y + 13.0, 4.0, tp_farbe(tp::gruppe(id)));
            ui.text(tp::name(id), r.x + 22.0, r.y + 18.0, 14.0, if geht { Theme::text() } else { Theme::gedaempft() });
            if ui.hit(r) {
                if geht { c.wochenplan.tage[d].slots[s] = Some(crate::trainingsplan::Zelle { einheit: id.into(), stufe: 1 }); geaendert = true; } else { v.toast(crate::inhalt::t("tp.fehler.plan"), false); }
            }
            yy += 27.0;
        }
        yy += 6.0;
        if c.wochenplan.tage[d].slots[s].is_some() && ui.button(Rect::new(pr.x + 8.0, yy, pr.w - 16.0, 30.0), crate::inhalt::t("tp.ui.einheit_entfernen"), false) { c.wochenplan.tage[d].slots[s] = None; geaendert = true; }
        yy += 42.0;
        let zeig = hover.or(c.wochenplan.tage[d].slots[s].as_ref().map(|z| z.einheit.as_str()).and_then(|e| tp::katalog().into_iter().find(|x| *x == e)));
        if let Some(id) = zeig { ui.paragraph(crate::inhalt::t(&format!("tp.einheit.{id}.text")), pr.x + 14.0, yy, 13.0, pr.w - 28.0, Theme::text2()); }
    } else {
        ui.text_semi(crate::inhalt::t("tp.ui.vorlagen"), pr.x + 14.0, yy, 16.0, Theme::text());
        yy += 14.0;
        for (id, w) in tp::werk() {
            let r = Rect::new(pr.x + 8.0, yy, pr.w - 16.0, 30.0);
            if frei { if ui.button(r, crate::inhalt::t(&format!("tp.vorlage.{id}.name")), c.wochenplan == w) { c.wochenplan = w; geaendert = true; v.toast(crate::inhalt::t("tp.ok.uebernommen"), true); } }
            else { ui.text(crate::inhalt::t(&format!("tp.vorlage.{id}.name")), r.x + 6.0, r.y + 20.0, 14.0, Theme::text2()); }
            yy += 36.0;
        }
        yy += 10.0;
        let sq = c.squad();
        let start = sq.iter().map(|&p| c.world.players[p].ps.last).sum::<f32>() / sq.len().max(1) as f32;
        let (_, zone) = tp::verlauf(&c.wochenplan, start);
        ui.text_semi(crate::inhalt::t("tp.ui.belastung"), pr.x + 14.0, yy, 16.0, Theme::text());
        draw_circle(pr.x + 20.0, yy + 22.0, 6.0, [Theme::gut(), Theme::gut(), Theme::achtung(), Theme::problem()][zone]);
        yy += 16.0;
        let h = ui.paragraph(crate::inhalt::t(&format!("tp.zone.{}", tp::zone_name(zone))), pr.x + 34.0, yy + 10.0, 14.0, pr.w - 48.0, Theme::text2());
        yy += h.max(20.0) + 16.0;
        if zone >= 2 { ui.paragraph(crate::inhalt::t(if zone == 3 { "tp.hinweis.belastung_rot" } else { "tp.hinweis.belastung_gelb" }), pr.x + 14.0, yy, 13.0, pr.w - 28.0, Theme::gedaempft()); }
        ui.paragraph(crate::inhalt::t(if frei { "ui.tp.waehlen" } else { "ui.tp.co_plan" }), pr.x + 14.0, pr.y + pr.h - 50.0, 13.0, pr.w - 28.0, Theme::gedaempft());
    }
    if geaendert { c.plan_uebernehmen(); }
}

pub(super) fn entwicklung(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(area, crate::inhalt::t("ui.seiten.entwicklung_der_spieler"), Theme::akzent());
    let sq = c.squad();
    let vis = ((area.h - 70.0) / 28.0) as usize;
    let start = scroll_rows(ui, v, area, sq.len(), vis);
    for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
        let pl = &c.world.players[p];
        let yy = area.y + 64.0 + k as f32 * 28.0;
        let (a, b) = c.pot_range(p);
        let phase = ["", crate::inhalt::t("en3.phase.aufschwung"), crate::inhalt::t("en3.betreff.formtief"), crate::inhalt::t("en3.betreff.krise")][pl.ps.phase.min(3) as usize];
        ui.player_photo(area.x + 24.0, yy - 5.0, 10.0, pl.id, &pl.name);
        ui.text(&pl.name, area.x + 40.0, yy, 15.0, Theme::text());
        ui.text(&crate::tf!("ui.seiten.jahre_potenzial", pl.age, stufe_wort(pl.ovr), stufe_wort((a + b) / 2.0)), area.x + 260.0, yy, 15.0, Theme::text2());
        let extra = [phase.to_string(), pl.ps.umschulung.as_ref().map(|u| crate::tf!("ui.seiten.lernt", pos_de(&u.0))).unwrap_or_default(), if pl.ps.mentor.is_some() { crate::inhalt::t("ui.seiten.hat_einen_mentor").into() } else { String::new() }, if pl.ps.last >= 70.0 { crate::inhalt::t("r.ui.demo.ueberlastet").into() } else { String::new() }].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(", ");
        ui.text_right(&extra, area.x + area.w - 14.0, yy, 15.0, Theme::achtung());
    }
}
