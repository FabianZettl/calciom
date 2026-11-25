use super::*;

pub(super) fn kader(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // the list first, portraits as the alternative (Fabian 24.11.)
    tabs(ui, v, &[crate::inhalt::t("kalender.ansicht.liste"), crate::inhalt::t("ui.pages.portraets"), crate::inhalt::t("bericht.statistik")]);
    if v.tab != 1 { kader_list(ui, v, c); return; }
    let all = c.squad();
    // Kennzahlen-Leiste in words
    let n = all.len().max(1) as f32;
    let avg_age = all.iter().map(|&p| c.world.players[p].age as f32).sum::<f32>() / n;
    let mood = all.iter().map(|&p| c.world.players[p].morale).sum::<f32>() / n;
    let inj = all.iter().filter(|&&p| c.world.players[p].inj > 0).count();
    let facts = crate::tf!("ui.pages.spieler_im_schnitt_jahre_stimmung", all.len(), format!("{:.0}", avg_age), mood_words(mood), if inj == 0 { crate::inhalt::t("ui.pages.alle_fit").to_string() } else { crate::tf!("zentrale.k.verletzt.viele", inj) });
    ui.text_right(&facts, x0() + cbreite(), Y0 - 20.0, 16.0, Theme::text2());
    let filters = [crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("aka.pos.tw"), crate::inhalt::t("achse.Abwehr"), crate::inhalt::t("gb2.vgl.mittel"), crate::inhalt::t("bericht.achse.angriff"), crate::inhalt::t("pille.verletzt")];
    if let Some(f) = ui.segment(Rect::new(x0() + 4.0, Y0, 640.0, 36.0), &filters, v.market_pos.min(5)) { v.market_pos = f; v.scroll = 0.0; v.scrolls.clear(); }
    let sq: Vec<usize> = all.into_iter().filter(|&p| { let pl = &c.world.players[p]; match v.market_pos { 1..=4 => part_of(&pl.pos) + 1 == v.market_pos, 5 => pl.inj > 0, _ => true } }).collect();
    let gw = cbreite() - 300.0;
    let ch = 156.0;
    // as many columns of at least 182 as fit, then the cards share the whole width (no gap before „Im Blick“ on wide windows)
    let cols = ((gw + 12.0) / 194.0).max(1.0) as usize;
    let cw = (gw + 12.0) / cols as f32 - 12.0;
    let rows = ((hoehe() - Y0 - 60.0) / (ch + 12.0)) as usize;
    let area = Rect::new(x0(), Y0 + 46.0, gw, hoehe() - Y0 - 60.0);
    let start = scroll_rows(ui, v, area, (sq.len() + cols - 1) / cols, rows) * cols;
    if sq.is_empty() { ui.empty(area, crate::inhalt::t("ui.pages.hier_ist_gerade_niemand_gutes")); }
    for (k, &p) in sq.iter().skip(start).take(rows * cols).enumerate() {
        let pl = &c.world.players[p];
        let r = Rect::new(x0() + 4.0 + (k % cols) as f32 * (cw + 12.0), Y0 + 50.0 + (k / cols) as f32 * (ch + 12.0), cw, ch);
        let sel = v.sel_player == Some(p);
        ui.card(r);
        if sel { rrect(r.x, r.y, r.w, 4.0, 2.0, Theme::verein()); }
        if ui.hover(r) { rrect(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0, 14.0, alpha(Theme::verein(), 0.08)); }
        ui.player_photo_nr(r.x + r.w / 2.0, r.y + 46.0, 30.0, pl.id, &pl.name, pl.no, Theme::verein());
        ui.smiley(r.x + r.w / 2.0 + 26.0, r.y + 66.0, 10.0, pl.morale);
        if pl.inj > 0 { rrect(r.x + 10.0, r.y + 10.0, 22.0, 22.0, 11.0, hex(0xf472b6)); ui.text_center("+", r.x + 21.0, r.y + 27.0, 20.0, hex(0xffffff)); }
        if pl.susp > 0 { rrect(r.x + 12.0, r.y + 10.0, 14.0, 20.0, 3.0, Theme::problem()); }
        let last = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
        let nm = ui.fit(&last, 20.0, r.w - 16.0);
        let w = measure_text(&nm, Some(&ui.head), 20, 1.0).width;
        ui.title(&nm, r.x + r.w / 2.0 - w / 2.0, r.y + 104.0, 20.0, Theme::text());
        ui.pos_tag(r.x + 12.0, r.y + 126.0, pos_de(&pl.pos));
        ui.text(&crate::tf!("ui.wort.alter_kurz", pl.age), r.x + 52.0, r.y + 131.0, 14.0, Theme::text2());
        ui.pill(r.x + r.w - 28.0, r.y + 126.0, pl.ovr, 36.0);
        zuschlag_text(ui, r.x + r.w - 72.0, r.y + 131.0, &pl);
        ui.bar(r.x + 12.0, r.y + ch - 12.0, r.w - 24.0, 5.0, pl.fit / 100.0, if pl.fit >= 80.0 { Theme::gut() } else if pl.fit >= 60.0 { Theme::achtung() } else { Theme::problem() });
        if ui.hit(r) { if sel { v.modal = Modal::Player(p); } else { v.sel_player = Some(p); } }
    }
    // detail card
    let dr = Rect::new(x0() + gw + 12.0, Y0, cbreite() - gw - 8.0, hoehe() - Y0 - 14.0);
    ui.card_titled(dr, crate::inhalt::t("sc.mentor.kachel"), Theme::verein());
    let Some(p) = v.sel_player.filter(|&p| c.world.players[p].team == c.own as i32) else {
        ui.empty(Rect::new(dr.x, dr.y + 36.0, dr.w, dr.h - 36.0), crate::inhalt::t("ui.pages.tipp_auf_ein_trikot_dann"));
        return;
    };
    let pl = c.world.players[p].clone();
    ui.player_photo_nr(dr.x + 60.0, dr.y + 90.0, 36.0, pl.id, &pl.name, pl.no, Theme::verein());
    ui.smiley(dr.x + 92.0, dr.y + 116.0, 13.0, pl.morale);
    let nm = ui.fit(&pl.name, 21.0, dr.w - 120.0);
    ui.title(&nm, dr.x + 108.0, dr.y + 80.0, 21.0, Theme::text());
    ui.pos_tag(dr.x + 108.0, dr.y + 97.0, pos_de(&pl.pos));
    ui.text(&crate::tf!("ui.wort.jahre_alt", pl.age), dr.x + 152.0, dr.y + 102.0, 15.0, Theme::text2());
    let lines = [
        crate::tf!("ui.pages.er_ist", mood_words(pl.morale)),
        if pl.inj > 0 { crate::tf!("ui.pages.faellt_noch_tage_aus", pl.inj, pl.inj_name) } else if pl.fit < 70.0 { crate::inhalt::t("ui.pages.wirkt_muede_eine_pause_taete").into() } else { crate::inhalt::t("ui.pages.koerperlich_frisch").into() },
        if pl.benched >= 3 { crate::inhalt::t("ui.pages.wartet_seit_wochen_auf_einsaetze").into() } else if pl.apps > 0 { crate::tf!("ui.pages.spiele_tore_in_dieser_saison", pl.apps, pl.goals) } else { crate::inhalt::t("ui.pages.noch_ohne_einsatz_in_dieser").into() },
        if pl.contract_end <= c.world.season { crate::inhalt::t("ui.pages.sein_vertrag_laeuft_im_sommer").into() } else { crate::tf!("ui.pages.vertrag_bis", date::START_YEAR as u32 + pl.contract_end + 1) },
    ];
    let mut y = dr.y + 160.0;
    for l in lines { y = ui.paragraph(&l, dr.x + 18.0, y, 16.0, dr.w - 36.0, Theme::text()) + 6.0; }
    ui.pill(dr.x + dr.w - 40.0, dr.y + 90.0, pl.ovr, 44.0);
    if crate::world::form_zuschlag(&pl) != 0.0 {
        zuschlag_text(ui, dr.x + dr.w - 50.0, dr.y + 122.0, &pl);
        ui.text_right(crate::inhalt::t("ui.wellen.zuschlag_hinweis"), dr.x + dr.w - 56.0, dr.y + 122.0, 12.0, Theme::gedaempft());
    }
    if ui.button_gold(Rect::new(dr.x + 16.0, dr.y + dr.h - 58.0, dr.w - 32.0, 44.0), crate::inhalt::t("ui.pages.profil_oeffnen")) { v.modal = Modal::Player(p); }
}

fn kader_list(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let mut sq = c.squad();
    let liste = 10 + if v.tab == 0 { 1 } else { 2 };
    sortieren(v, liste, &mut sq, |p, col| {
        let pl = &c.world.players[p];
        let n = |x: f32| Sk::N(x);
        match (v.tab == 0, col) {
            (_, 0) => n(-(pl.no as f32)),
            (_, 1) => Sk::S(pl.name.to_lowercase()),
            (_, 2) => n(-(crate::pesdb::pos_index(&pl.pos) as f32)),
            (true, 3) => n(pl.age as f32), (true, 4) => n(pl.ovr), (true, 5) => n(pl.pot), (true, 6) => n(pl.form),
            (true, 7) => n(pl.fit), (true, 8) => n(pl.morale), (true, 9) => n(pl.value as f32), (true, _) => n(-(pl.contract_end as f32)),
            (false, 3) => n(pl.apps as f32), (false, 4) => n(pl.goals as f32), (false, 5) => n(pl.assists as f32), (false, 6) => n(pl.avg_rating()),
            (false, 7) => n(if pl.last.is_empty() { 0.0 } else { pl.last.iter().sum::<f32>() / pl.last.len() as f32 }),
            (false, 8) => n(pl.yellow as f32), (false, _) => n(pl.wage as f32),
        }
    });
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    let head: Vec<(&str, f32)> = if v.tab == 0 {
        vec![(crate::inhalt::t("ui.kader.kopf.nr"), 14.0), (crate::inhalt::t("ui.kader.kopf.name"), 60.0), (crate::inhalt::t("ui.kader.kopf.pos"), 330.0), (crate::inhalt::t("ui.kader.kopf.alter"), 390.0), (crate::inhalt::t("ui.pages.staerke"), 450.0), (crate::inhalt::t("ui.kader.kopf.potenzial"), 520.0), (crate::inhalt::t("ui.kader.kopf.form"), 620.0), (crate::inhalt::t("ui.kader.kopf.fitness"), 680.0), (crate::inhalt::t("ui.kader.kopf.moral"), 800.0), (crate::inhalt::t("ui.kader.kopf.marktwert"), 920.0), (crate::inhalt::t("ui.kader.kopf.vertrag"), 1050.0)]
    } else {
        vec![(crate::inhalt::t("ui.kader.kopf.nr"), 14.0), (crate::inhalt::t("ui.kader.kopf.name"), 60.0), (crate::inhalt::t("ui.kader.kopf.pos"), 330.0), (crate::inhalt::t("ui.kader.kopf.spiele"), 400.0), (crate::inhalt::t("ui.kader.kopf.tore"), 480.0), (crate::inhalt::t("ui.kader.kopf.vorlagen"), 550.0), (crate::inhalt::t("ui.pages.note"), 650.0), (crate::inhalt::t("ui.pages.letzte_fuenf"), 740.0), (crate::inhalt::t("ui.kader.kopf.gelb"), 960.0), (crate::inhalt::t("ui.kader.kopf.gehalt"), 1030.0)]
    };
    sort_head(ui, v, liste, area.x, area.y + 26.0, &head);
    let vis = ((area.h - 44.0) / 38.0) as usize;
    let start = scroll_rows(ui, v, area, sq.len(), vis);
    for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
        let pl = &c.world.players[p];
        let r = Rect::new(area.x + 4.0, area.y + 40.0 + k as f32 * 38.0, area.w - 8.0, 36.0);
        if k % 2 == 1 { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::erhoehung(), 0.6)); }
        if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::ueberlagerung()); }
        draw_rectangle(r.x, r.y + 4.0, 3.0, r.h - 8.0, Theme::teil(&pl.pos));
        let cy = r.y + 24.0;
        ui.text_semi(&format!("{}", pl.no), r.x + 12.0, cy, 17.0, Theme::text2());
        ui.player_photo(r.x + 52.0, r.y + 18.0, 14.0, pl.id, &pl.name);
        let nm = ui.fit(&pl.name, 18.0, 210.0);
        ui.text_semi(&nm, r.x + 72.0, cy, 18.0, Theme::text());
        if pl.inj > 0 { ui.text_semi("+", r.x + 300.0, cy, 20.0, Theme::problem()); }
        if pl.susp > 0 { rrect(r.x + 300.0, cy - 13.0, 10.0, 14.0, 2.0, Theme::problem()); }
        if pl.listed { ui.text_semi(crate::inhalt::t("ui.wort.buchstabe_t"), r.x + 314.0, cy, 15.0, Theme::gold()); }
        ui.pos_tag(r.x + 326.0, cy - 5.0, pos_de(&pl.pos));
        if v.tab == 0 {
            ui.text(&format!("{}", pl.age), r.x + 396.0, cy, 17.0, Theme::text());
            ui.pill(r.x + 470.0, cy - 6.0, pl.ovr, 38.0);
            zuschlag_text(ui, r.x + 493.0, cy - 1.0, &pl);
            let stars = ((pl.pot - 50.0) / 10.0).clamp(0.5, 5.0);
            for s in 0..5 { let full = (s as f32) < stars; draw_poly(r.x + 528.0 + s as f32 * 16.0, cy - 6.0, 5, 6.0, -90.0, if full { Theme::gold() } else { Theme::ueberlagerung() }); }
            // the grades of the last five games as bars (Fabian 24.11.), the arrow while there are none
            if pl.last.is_empty() { form_arrow(ui, r.x + 626.0, cy - 6.0, pl.form); }
            for (j, rt) in pl.last.iter().rev().take(5).rev().enumerate() { let h = ((rt - 3.0) / 7.0 * 24.0).clamp(3.0, 24.0); draw_rectangle(r.x + 604.0 + j as f32 * 13.0, cy + 2.0 - h, 10.0, h, Theme::note(*rt)); }
            ui.bar(r.x + 680.0, cy - 10.0, 90.0, 8.0, pl.fit / 100.0, if pl.fit >= 80.0 { Theme::gut() } else if pl.fit >= 60.0 { Theme::achtung() } else { Theme::problem() });
            ui.text(&format!("{:.0}", pl.fit), r.x + 776.0, cy, 15.0, Theme::text2());
            ui.bar(r.x + 800.0, cy - 10.0, 90.0, 8.0, pl.morale / 100.0, if pl.morale >= 60.0 { Theme::gut() } else if pl.morale >= 40.0 { Theme::achtung() } else { Theme::problem() });
            ui.text(&money(pl.value), r.x + 920.0, cy, 17.0, Theme::text());
            ui.text(&crate::tf!("zentrale.bis", date::START_YEAR as u32 + pl.contract_end + 1), r.x + 1050.0, cy, 17.0, if pl.contract_end <= c.world.season { Theme::achtung() } else { Theme::text() });
        } else {
            ui.text(&format!("{}", pl.apps), r.x + 410.0, cy, 17.0, Theme::text());
            ui.text(&format!("{}", pl.goals), r.x + 490.0, cy, 17.0, Theme::text());
            ui.text(&format!("{}", pl.assists), r.x + 570.0, cy, 17.0, Theme::text());
            if pl.rn > 0 { ui.note_pill(r.x + 676.0, cy - 6.0, pl.avg_rating()); } else { ui.text("–", r.x + 670.0, cy, 17.0, Theme::gedaempft()); }
            for (j, rt) in pl.last.iter().enumerate() { let h = (rt - 3.0) / 7.0 * 24.0; draw_rectangle(r.x + 744.0 + j as f32 * 26.0, cy + 2.0 - h, 18.0, h, Theme::note(*rt)); }
            ui.text(&format!("{}", pl.yellow), r.x + 970.0, cy, 17.0, Theme::achtung());
            ui.text(&money(pl.wage), r.x + 1030.0, cy, 17.0, Theme::text());
        }
        if ui.hit(r) { v.modal = Modal::Player(p); }
    }
}

/// Squad › Registration (Fabian 24.11.): for each European competition list A (at most 25) to tick, list B (own
/// players up to 21) free; when the list can change and how many winter changes are left.
thread_local! { static PLUS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
/// Squad registration (Fabian 25.11.: smarter, grouped, sorted by shirt number, list B explained, "+" for the second
/// team and the academy): three cards side by side, list A, list B and not registered; "+" swaps the third card for
/// the candidates of the second team and the academy, who get a free shirt number when registered.
pub(super) fn meldung_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let wb = c.europa_wettbewerbe();
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    if wb.is_empty() { ui.card_titled(r, t("ui.meldung.seite"), Theme::akzent()); ui.text(t("ui.meldung.kein_europa"), r.x + 16.0, r.y + 64.0, 15.0, Theme::text2()); return; }
    let namen: Vec<String> = wb.iter().map(|&k| c.world.cups[k].name.clone()).collect();
    let refs: Vec<&str> = namen.iter().map(|x| x.as_str()).collect();
    tabs(ui, v, &refs);
    let ci = wb[v.tab.min(wb.len() - 1)];
    let lk = wo_look(c, Wo::Pokal(ci));
    let _ = wettbewerb_banner(ui, Rect::new(r.x, r.y, r.w, 46.0), &lk, c.comp_cid(Wo::Pokal(ci)), &namen[v.tab.min(wb.len() - 1)]);
    let m = c.meldung(ci).clone();
    let offen = c.meldung_offen(ci);
    let max = crate::regeln::f("uefa", "meldung.liste_a") as usize;
    let alter_b = crate::regeln::f("uefa", "meldung.liste_b_alter") as u8;
    let status = match &offen { Ok(false) => t("ui.meldung.offen").to_string(), Ok(true) => crate::tf!("ui.meldung.winter_offen", crate::regeln::f("uefa", "meldung.nachmeldungen") as u8 - m.nachgemeldet), Err(e) => e.clone() };
    // the explanation box
    let info = Rect::new(r.x, r.y + 56.0, r.w, 64.0);
    rrect(info.x, info.y, info.w, info.h, 8.0, alpha(lk.c1, 0.10));
    draw_rectangle(info.x, info.y, 4.0, info.h, lk.c1);
    ui.paragraph(&crate::tf!("ui.meldung.erklaerung", max, alter_b), info.x + 16.0, info.y + 22.0, 14.0, info.w - 300.0, Theme::text());
    ui.text_right(&status, info.x + info.w - 14.0, info.y + 38.0, 14.0, if offen.is_ok() { Theme::gut() } else { Theme::text2() });
    // the three groups
    let ueber = |c: &Career, p: usize| c.world.players[p].age > alter_b;
    let mut a_liste: Vec<usize> = m.liste_a.iter().copied().chain(m.extra.iter().copied().filter(|&p| ueber(c, p))).collect();
    let mut b_liste: Vec<usize> = c.squad().into_iter().filter(|&p| c.liste_b(p)).chain(m.extra.iter().copied().filter(|&p| !ueber(c, p))).collect();
    let mut nicht: Vec<usize> = c.squad().into_iter().filter(|&p| !c.liste_b(p) && !m.liste_a.contains(&p)).collect();
    for l in [&mut a_liste, &mut b_liste, &mut nicht] { l.sort_by_key(|&p| (c.world.players[p].no == 0, c.world.players[p].no)); }
    let gy = info.y + info.h + 12.0;
    let gw = (r.w - 24.0) / 3.0;
    let gh = r.y + r.h - gy;
    let plus = PLUS.with(|x| x.get());
    let kandidaten = if plus { c.meldung_kandidaten(ci) } else { vec![] };
    let gruppen: [(String, &Vec<usize>, Color, u8); 3] = [
        (crate::tf!("ui.meldung.gruppe_a", a_liste.len(), max), &a_liste, if a_liste.len() <= max { lk.c1 } else { Theme::problem() }, 0),
        (crate::tf!("ui.meldung.gruppe_b", alter_b), &b_liste, Theme::gut(), 1),
        (if plus { t("ui.meldung.gruppe_plus").to_string() } else { t("ui.meldung.gruppe_nicht").to_string() }, if plus { &kandidaten } else { &nicht }, if plus { Theme::akzent() } else { Theme::gedaempft() }, 2),
    ];
    let mut aktion: Option<(usize, bool)> = None;
    for (g, (titel, liste, farbe, art)) in gruppen.iter().enumerate() {
        let gr = Rect::new(r.x + g as f32 * (gw + 12.0), gy, gw, gh);
        ui.card(gr);
        rrect(gr.x, gr.y, gr.w, 40.0, 10.0, *farbe);
        draw_rectangle(gr.x, gr.y + 30.0, gr.w, 10.0, *farbe);
        ui.text_semi(&titel.to_uppercase(), gr.x + 14.0, gr.y + 26.0, 14.0, on(*farbe));
        if g == 2 && ui.button(Rect::new(gr.x + gr.w - 118.0, gr.y + 6.0, 108.0, 28.0), if plus { t("ui.meldung.zurueck") } else { t("ui.meldung.plus") }, false) { PLUS.with(|x| x.set(!plus)); }
        if liste.is_empty() { ui.paragraph(t(if *art == 2 && plus { "ui.meldung.keine_kandidaten" } else { "ui.meldung.leer" }), gr.x + 14.0, gr.y + 66.0, 13.0, gr.w - 28.0, Theme::text2()); }
        let zeile = 34.0;
        let platz = ((gr.h - 52.0) / zeile) as usize;
        for (k, &p) in liste.iter().take(platz).enumerate() {
            let pl = &c.world.players[p];
            let rr = Rect::new(gr.x + 8.0, gr.y + 48.0 + k as f32 * zeile, gr.w - 16.0, zeile - 4.0);
            if k % 2 == 0 { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, Theme::erhoehung()); }
            rrect(rr.x + 6.0, rr.y + 4.0, 30.0, 22.0, 5.0, alpha(*farbe, 0.85));
            ui.text_center(&if pl.no > 0 { pl.no.to_string() } else { "–".into() }, rr.x + 21.0, rr.y + 20.0, 14.0, on(*farbe));
            ui.pos_tag(rr.x + 44.0, rr.y + 6.0, pos_de(&pl.pos));
            ui.text(&ui.fit(&pl.name, 14.0, rr.w - 190.0), rr.x + 86.0, rr.y + 20.0, 14.0, Theme::text());
            ui.text_right(&format!("{} · {:.0}", crate::tf!("ui.meldung.alter", pl.age), pl.ovr), rr.x + rr.w - 40.0, rr.y + 20.0, 12.0, Theme::text2());
            let extra = m.extra.contains(&p);
            // list A players and extras can be taken off, the not registered and the candidates registered; list B is free
            let knopf = match art { 0 => Some("−"), 1 if extra => Some("−"), 2 => Some("+"), _ => None }; // roh
            if let Some(kn) = knopf { if ui.button(Rect::new(rr.x + rr.w - 32.0, rr.y + 2.0, 28.0, 26.0), kn, false) { aktion = Some((p, extra || (plus && *art == 2))); } }
        }
        if liste.len() > platz { ui.text(&crate::tf!("ui.meldung.weitere", liste.len() - platz), gr.x + 14.0, gr.y + gr.h - 10.0, 12.0, Theme::gedaempft()); }
    }
    if let Some((p, ist_extra)) = aktion {
        let r = if ist_extra { c.extra_melden(ci, p) } else { c.melden(ci, p) };
        match r { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
    }
}
