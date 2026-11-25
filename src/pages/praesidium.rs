use super::*;

// ================================================================ Präsidium
pub(super) fn praesidium(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let y = Y0 - 40.0;
    let r = Rect::new(x0() + 4.0, y, 420.0, 330.0);
    ui.card_titled(r, crate::inhalt::t("sc.pv.vertrauen"), Theme::akzent());
    let conf = c.confidence();
    let col = if conf >= 65.0 { Theme::gut() } else if conf >= 35.0 { Theme::achtung() } else { Theme::problem() };
    let (cx, cy) = (r.x + r.w / 2.0, r.y + 180.0);
    for i in 0..60 {
        let a = std::f32::consts::PI * (1.0 + i as f32 / 59.0);
        let on = (i as f32 / 59.0) <= conf / 100.0;
        draw_circle(cx + a.cos() * 120.0, cy + a.sin() * 120.0, 6.0, if on { col } else { Theme::ueberlagerung() });
    }
    ui.title(&format!("{:.0}", conf), cx - 30.0, cy - 10.0, 52.0, col);
    ui.text_center(c.confidence_label(), cx, cy + 30.0, 20.0, Theme::text());
    if let Some(dl) = c.ultimatum { ui.text_center(&crate::tf!("ui.pages.ultimatum_noch_tage", dl.saturating_sub(c.abs())), cx, cy + 64.0, 18.0, Theme::problem()); }
    let r2 = Rect::new(x0() + 438.0, y, cbreite() - 434.0, 160.0);
    ui.card_titled(r2, crate::inhalt::t("zentrale.k.ziel"), Theme::gold());
    ui.title(&c.club().goal_kind, r2.x + 14.0, r2.y + 70.0, 34.0, Theme::text());
    let rank = c.world.rank_of(c.own);
    ui.text(&crate::tf!("ui.pages.mindestens_platz_aktuell_platz", c.club().goal_pos, rank), r2.x + 14.0, r2.y + 100.0, 19.0, if rank <= c.club().goal_pos { Theme::gut() } else { Theme::achtung() });
    let risiko = c.entlassungsrisiko();
    let rw = crate::inhalt::t(if risiko >= 0.6 { "ui.risiko.hoch" } else if risiko >= 0.3 { "ui.risiko.mittel" } else { "ui.risiko.gering" });
    ui.text(&crate::tf!("ui.pages.entlassungsgefahr_vom_ueberschuss_gehen_", rw, format!("{:.0}", c.budget_anteil() * 100.0)), r2.x + 14.0, r2.y + 130.0, 16.0, Theme::text2());
    let r3 = Rect::new(x0() + 438.0, y + 172.0, cbreite() - 434.0, 158.0);
    ui.card_titled(r3, crate::inhalt::t("ui.pages.umfeld"), Theme::akzent());
    for (i, (n, val)) in [("Fans", c.fans), (crate::inhalt::t("story.groesse.medienbild"), c.media)].iter().enumerate() { // roh
        let yy = r3.y + 60.0 + i as f32 * 46.0;
        ui.text(n, r3.x + 14.0, yy, 18.0, Theme::text());
        ui.bar(r3.x + 150.0, yy - 10.0, r3.w - 240.0, 10.0, val / 100.0, if *val >= 55.0 { Theme::gut() } else if *val >= 35.0 { Theme::achtung() } else { Theme::problem() });
        ui.text_right(&format!("{:.0}", val), r3.x + r3.w - 14.0, yy, 18.0, Theme::text());
    }
    ui.text_right(&crate::tf!("mv.zeile", format!("{:.0}", c.mitglieder())), r3.x + r3.w - 14.0, r3.y + r3.h - 8.0, 15.0, Theme::text2());
    // the club's reputation over the seasons (geschichten.rs), as a line under the surroundings
    if !c.geschichten.tags.is_empty() {
        let n: Vec<&str> = c.geschichten.tags.iter().map(|k| crate::inhalt::t(&format!("gw.ruf.{k}"))).collect(); // roh
        ui.text(&crate::tf!("gw.ruf_zeile", n.join(" · ")), r3.x + 14.0, r3.y + r3.h - 8.0, 15.0, Theme::gold());
    }
    // the three people of the board, the club type and the stage, and the season's side goals (praesidium.rs)
    c.praesidium_an();
    let pw = (cbreite() * 0.62 - 24.0) / 3.0;
    let stufe = c.eskalation();
    for (k, p) in c.praesidium.clone().iter().enumerate() {
        let r = Rect::new(x0() + 4.0 + k as f32 * (pw + 12.0), y + 344.0, pw, 140.0);
        ui.card(r);
        ui.text_semi(crate::inhalt::t(&format!("ui.pr.rolle.{}", p.rolle)), r.x + 14.0, r.y + 24.0, 13.0, Theme::gedaempft());
        ui.text_semi(&ui.fit(&p.name, 18.0, r.w - 28.0), r.x + 14.0, r.y + 50.0, 18.0, Theme::text());
        ui.text(crate::inhalt::t(&format!("ui.pr.haltung.{}", p.haltung)), r.x + 14.0, r.y + 72.0, 14.0, Theme::text2());
        let m = c.person_stimmung(p);
        let col = if m >= 60.0 { Theme::gut() } else if m >= 40.0 { Theme::achtung() } else { Theme::problem() };
        ui.bar(r.x + 14.0, r.y + 96.0, r.w - 70.0, 10.0, m / 100.0, col);
        ui.text_right(&format!("{m:.0}"), r.x + r.w - 14.0, r.y + 106.0, 16.0, col);
        ui.text(crate::inhalt::t(&format!("ui.pr.stimmung.{}", if m >= 60.0 { 0 } else if m >= 40.0 { 1 } else { 2 })), r.x + 14.0, r.y + 130.0, 13.0, Theme::text2());
    }
    let rz = Rect::new(x0() + 4.0 + 3.0 * (pw + 12.0), y + 344.0, cbreite() - 3.0 * (pw + 12.0), 140.0);
    ui.card(rz);
    ui.text_semi(&crate::tf!("ui.pr.typ_zeile", crate::inhalt::t(&format!("ui.pr.typ.{}", c.vereinstyp())), crate::inhalt::t(&format!("ui.pr.stufe.{stufe}"))), rz.x + 14.0, rz.y + 24.0, 14.0, [Theme::gut(), Theme::text(), Theme::achtung(), Theme::problem(), Theme::problem()][stufe]);
    ui.text_semi(crate::inhalt::t("ui.pr.nebenziele"), rz.x + 14.0, rz.y + 52.0, 13.0, Theme::gedaempft());
    let season = c.world.season;
    let ziele: Vec<String> = c.nebenziele.iter().filter(|z| z.season == season).map(|z| z.key.clone()).collect();
    for (k, z) in ziele.iter().enumerate() {
        let (ok, zeile) = c.nebenziel_stand(z);
        let yy = rz.y + 76.0 + k as f32 * 28.0;
        draw_circle(rz.x + 22.0, yy - 5.0, 6.0, if ok { Theme::gut() } else { Theme::achtung() });
        ui.text_semi(crate::inhalt::t(&format!("ui.pr.ziel.{z}.name")), rz.x + 36.0, yy, 15.0, Theme::text());
        ui.text_right(&ui.fit(&zeile, 13.0, rz.w * 0.55), rz.x + rz.w - 14.0, yy, 13.0, Theme::text2());
    }
    let r4 = Rect::new(x0() + 4.0, y + 496.0, cbreite(), (hoehe() - y - 512.0).max(100.0));
    ui.card_titled(r4, crate::inhalt::t("ui.pages.so_denkt_das_praesidium"), Theme::akzent());
    // requests to the board: transfer funds, wage room, a facility grant
    for (k, key) in ["ui.bitte.knopf_transfer", "ui.bitte.knopf_gehalt", "ui.bitte.knopf_ausbau"].iter().enumerate() { // roh
        let b = Rect::new(r4.x + r4.w - 3.0 * 196.0 - 8.0 + k as f32 * 196.0, r4.y + 6.0, 188.0, 28.0);
        if ui.button(b, crate::inhalt::t(key), false) { match c.vorstand_bitte(k as u8) { Ok(t) => { let gut = !t.is_empty() && c.bitten.last().map_or(false, |x| x.2 > 0.0); c.grund("Präsidium", crate::inhalt::t("ui.bitte.titel"), 0.0, &t); v.toast(&t, gut); } Err(e) => v.toast(&e, false) } } // roh
    }
    let gr: Vec<_> = c.mensch.gruende.iter().rev().filter(|g| g.bezug == "Präsidium").take(((r4.h - 70.0) / 26.0).max(1.0) as usize).cloned().collect(); // roh
    if gr.is_empty() {
        ui.paragraph(crate::inhalt::t("ui.pages.nach_jedem_spiel_schaut_das"), r4.x + 14.0, r4.y + 56.0, 18.0, r4.w - 28.0, Theme::text2());
    }
    for (k, g) in gr.iter().enumerate() {
        let yy = r4.y + 60.0 + k as f32 * 26.0;
        ui.text(&date::short(g.abs / date::SEASON_DAYS, g.abs % date::SEASON_DAYS), r4.x + 14.0, yy, 16.0, Theme::gedaempft());
        ui.text(&g.grund, r4.x + 110.0, yy, 17.0, Theme::text());
        ui.text_right(&format!("{:+.0}", g.wert), r4.x + r4.w - 14.0, yy, 17.0, if g.wert >= 0.0 { Theme::gut() } else { Theme::problem() });
    }
}

pub(super) fn ziele(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.seiten.saisonziel_des_praesidiums"), Theme::gold());
    ui.title(&c.club().goal_kind.clone(), l.x + 14.0, l.y + 80.0, 34.0, Theme::text());
    let rank = c.world.rank_of(c.own);
    ui.text(&crate::tf!("ui.pages.mindestens_platz_aktuell_platz", c.club().goal_pos, rank), l.x + 14.0, l.y + 112.0, 18.0, if rank <= c.club().goal_pos { Theme::gut() } else { Theme::achtung() });
    ui.paragraph(crate::inhalt::t("ui.seiten.das_ziel_richtet_sich_nach"), l.x + 14.0, l.y + 150.0, 15.0, l.w - 28.0, Theme::text2());
    if !c.mensch.ziel_angenommen {
        if ui.button_gold(Rect::new(l.x + 14.0, l.y + 250.0, 220.0, 38.0), crate::inhalt::t("ui.seiten.ziel_annehmen")) { c.mensch.ziel_angenommen = true; c.grund("Präsidium", crate::inhalt::t("zentrale.k.ziel"), 0.0, crate::inhalt::t("ui.seiten.ziel_angenommen")); v.toast(crate::inhalt::t("ui.seiten.ziel_angenommen"), true); } // roh
    } else { ui.text(crate::inhalt::t("ui.seiten.du_hast_das_ziel_angenommen"), l.x + 14.0, l.y + 270.0, 16.0, Theme::gut()); }
    ui.text(&crate::tf!("ui.seiten.vision_des_praesidiums", crate::inhalt::t(&format!("tm.persona.{}", c.vision()))), l.x + 14.0, l.y + 320.0, 16.0, Theme::text2());
    // milestones below the promises: seal when reached, otherwise progress (meilensteine.toml)
    // 19 milestones in two columns: the card takes the room it needs, the promises keep the rest
    let n_ms = crate::meilensteine::liste().len();
    let hm = (70.0 + n_ms.div_ceil(2) as f32 * 40.0).min(r.h * 0.78);
    let m = Rect::new(r.x, r.y + r.h - hm, r.w, hm);
    let r = Rect::new(r.x, r.y, r.w, r.h - hm - 12.0);
    ui.card_titled(m, crate::inhalt::t("ui.meilenstein.titel"), Theme::gold());
    for (k, (id, zaehlt, ziel)) in crate::meilensteine::liste().iter().enumerate() {
        let (sp, zeile) = (k % 2, k / 2);
        let bw = (m.w - 42.0) / 2.0;
        let (x, y) = (m.x + 14.0 + sp as f32 * (bw + 14.0), m.y + 58.0 + zeile as f32 * ((m.h - 70.0) / n_ms.div_ceil(2).max(1) as f32));
        let erreicht = c.meilensteine.iter().any(|x| &x.0 == id);
        let stand = c.meilenstein_stand(zaehlt).min(*ziel);
        draw_circle(x + 12.0, y - 5.0, 11.0, if erreicht { Theme::gold() } else { alpha(Theme::gedaempft(), 0.3) });
        if erreicht { ui.text_center("✓", x + 12.0, y + 1.0, 14.0, hex(0x1b1f27)); }
        ui.text_semi(crate::inhalt::t(&format!("ui.meilenstein.{id}")), x + 30.0, y, 15.0, if erreicht { Theme::text() } else { Theme::text2() });
        ui.bar(x + 30.0, y + 8.0, bw - 110.0, 6.0, stand / ziel.max(1.0), if erreicht { Theme::gold() } else { Theme::akzent() });
        ui.text_right(&crate::tf!("ui.meilenstein.fortschritt", format!("{stand:.0}"), format!("{ziel:.0}")), x + bw, y + 14.0, 12.0, Theme::gedaempft());
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.oeffentliche_zusagen"), Theme::akzent());
    let z: Vec<(String, String)> = c.me.zusagen.iter().map(|z| (if z.0 == crate::id("saisonziel") { crate::inhalt::t("ui.seiten.saisonziel_erreichen").to_string() } else { crate::tf!("ui.seiten.bleibt", z.1.map(|p| c.world.players[p].name.clone()).unwrap_or_default()) }, crate::inhalt::t(&format!("ui.zusage.stand.{}", [crate::id("offen"), crate::id("gehalten"), crate::id("gebrochen")][z.4.min(2) as usize])).to_string())).collect();
    if z.is_empty() { ui.text(crate::inhalt::t("ui.seiten.keine_zusagen_gegeben_in_der"), r.x + 14.0, r.y + 60.0, 16.0, Theme::text2()); }
    zeilen(ui, r, &z, 60.0);
}
