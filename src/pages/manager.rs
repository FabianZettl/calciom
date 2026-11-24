use super::*;

pub(super) fn manager_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    c.manager_an();
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("uebersicht.faehigkeiten"), Theme::akzent());
    for i in 0..7 {
        let yy = l.y + 64.0 + i as f32 * 40.0;
        ui.text(crate::rest3::werte_de_t()[i], l.x + 14.0, yy, 17.0, Theme::text());
        ui.bar(l.x + 220.0, yy - 10.0, l.w - 300.0, 10.0, c.mgr_wert(i) / 100.0, Theme::akzent());
        ui.text_right(&format!("{:.0}", c.mgr_wert(i)), l.x + l.w - 14.0, yy, 17.0, Theme::text2());
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.zustand_und_stil"), Theme::gold());
    for (i, (n, w)) in [(crate::inhalt::t("ui.seiten.energie"), c.mgr.energie), (crate::inhalt::t("ui.seiten.stress"), c.mgr.stress), (crate::inhalt::t("manager.wert.motivation"), c.mgr.motivation)].iter().enumerate() {
        let yy = r.y + 64.0 + i as f32 * 40.0;
        ui.text(n, r.x + 14.0, yy, 17.0, Theme::text());
        ui.bar(r.x + 160.0, yy - 10.0, r.w - 240.0, 10.0, w / 100.0, if (i == 1) == (*w > 60.0) { Theme::problem() } else { Theme::gut() });
        ui.text_right(&format!("{:.0}", w), r.x + r.w - 14.0, yy, 17.0, Theme::text2());
    }
    ui.text(&crate::tf!("gm.mg.hero.stil", c.stil().map(|i| crate::rest3::stile_de_t()[i]).unwrap_or(crate::inhalt::t("ui.seiten.noch_nicht_erkennbar"))), r.x + 14.0, r.y + 200.0, 18.0, Theme::text());
    ui.paragraph(crate::inhalt::t("ui.seiten.ab_stress_70_oder_energie"), r.x + 14.0, r.y + 236.0, 15.0, r.w - 28.0, Theme::text2());
    if ui.button(Rect::new(r.x + 14.0, r.y + 300.0, 200.0, 36.0), crate::inhalt::t("ui.seiten.freizeit_nehmen"), false) { match c.freizeit() { Ok(()) => v.toast(crate::inhalt::t("ui.seiten.durchatmen"), true), Err(e) => v.toast(&e, false) } }
}

pub(super) fn tagebuch(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(area, crate::inhalt::t("ui.seiten.tagebuch_jede_wirkung_mit_ihrem"), Theme::akzent());
    let vis = ((area.h - 70.0) / 28.0) as usize;
    let n = c.mensch.gruende.len();
    let start = scroll_rows(ui, v, area, n, vis);
    for (k, g) in c.mensch.gruende.iter().rev().skip(start).take(vis).enumerate() {
        let yy = area.y + 64.0 + k as f32 * 28.0;
        ui.text(&date::short(g.abs / date::SEASON_DAYS, g.abs % date::SEASON_DAYS), area.x + 14.0, yy, 15.0, Theme::gedaempft());
        ui.text(&ui.fit(&crate::tf!("profil.kurz_text", crate::inhalt::anzeige(&g.bezug), g.was), 15.0, 300.0), area.x + 110.0, yy, 15.0, Theme::text());
        ui.text(&ui.fit(&g.grund, 15.0, area.w - 560.0), area.x + 430.0, yy, 15.0, Theme::text2());
        if g.wert != 0.0 { ui.text_right(&format!("{:+.1}", g.wert), area.x + area.w - 14.0, yy, 15.0, if g.wert > 0.0 { Theme::gut() } else { Theme::problem() }); }
    }
}
