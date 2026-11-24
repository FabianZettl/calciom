use super::*;

pub(super) fn trainer(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("menu.trainerprofil.reiter.4"), crate::inhalt::t("tm.reiter")]);
    if v.tab == 1 { trainermarkt(ui, v, c); return; }
    ui.title(&c.coach.to_uppercase(), x0() + 8.0, Y0 - 14.0, 30.0, Theme::text());
    let y = Y0;
    let w3 = (cbreite() - 24.0) / 3.0;
    kpi(ui, Rect::new(x0() + 4.0, y, w3, 96.0), crate::inhalt::t("ui.wort.gross_ruf"), &format!("{:.0}", c.mgr_rep), if c.mgr_rep >= 70.0 { crate::inhalt::t("ui.pages.grosser_name") } else if c.mgr_rep >= 50.0 { crate::inhalt::t("ui.pages.bekannter_name") } else if c.mgr_rep >= 30.0 { crate::inhalt::t("ui.pages.solider_ruf") } else { crate::inhalt::t("ui.pages.kaum_bekannt") }, Theme::akzent());
    kpi(ui, Rect::new(x0() + 16.0 + w3, y, w3, 96.0), crate::inhalt::t("ui.wort.gross_gehalt"), &money(c.mgr_wage), crate::inhalt::t("welt.pro_jahr"), Theme::gold());
    kpi(ui, Rect::new(x0() + 28.0 + 2.0 * w3, y, w3, 96.0), crate::inhalt::t("ui.pages.eigenes_konto"), &money(c.mgr_money), crate::inhalt::t("ui.wort.angespart"), Theme::gut());
    let r = Rect::new(x0() + 4.0, y + 110.0, cbreite() * 0.5 - 6.0, hoehe() - y - 126.0);
    ui.card_titled(r, crate::inhalt::t("menu.trainerprofil.reiter.4"), Theme::akzent());
    let mut yy = r.y + 56.0;
    // coaching licence (laufbahn.toml [lizenz]): level, the clubs it is good for, what is missing for the next one
    ui.text_semi(&format!("{}: {}", crate::inhalt::t("lb.lizenz.titel"), crate::inhalt::t(&format!("lb.lizenz.{}", c.lizenz()))), r.x + 14.0, yy, 18.0, Theme::akzent());
    ui.text(&crate::tf!("lb.lizenz.reicht", format!("{:.0}", c.lizenz_grenze() * 100.0)), r.x + 14.0, yy + 24.0, 15.0, Theme::text2());
    let naechste = match c.lizenz_naechste() {
        Some((n, ruf, monate)) if ruf > 0.0 || monate > 0.0 => format!("{} {}: {}", crate::inhalt::t("lb.lizenz.naechste"), crate::inhalt::t(&format!("lb.lizenz.{n}")), crate::tf!("lb.lizenz.fehlt", ruf, monate)),
        Some(_) => crate::inhalt::t("lb.lizenz.bereit").to_string(),
        None => crate::inhalt::t("lb.lizenz.hoechste").to_string(),
    };
    ui.text(&naechste, r.x + 14.0, yy + 46.0, 15.0, Theme::text2());
    yy += 84.0;
    if c.history.is_empty() { ui.text(&crate::tf!("ui.pages.seit_bei", date::season_label(0), c.club().name), r.x + 14.0, yy, 18.0, Theme::text2()); }
    for h in c.history.iter().rev().take(14) { ui.text(h, r.x + 14.0, yy, 18.0, Theme::text()); yy += 28.0; }
    let r2 = Rect::new(x0() + cbreite() * 0.5 + 10.0, y + 110.0, cbreite() * 0.5 - 6.0, hoehe() - y - 126.0);
    ui.card_titled(r2, crate::inhalt::t("menu.trainerprofil.reiter.5"), Theme::gold());
    let offers: Vec<usize> = (0..c.inbox.len()).filter(|&i| matches!(c.inbox[i].action, Action::JobOffer { .. }) && !c.inbox[i].done).collect();
    if offers.is_empty() { ui.text(crate::inhalt::t("ui.pages.im_moment_liegt_kein_angebot"), r2.x + 14.0, r2.y + 56.0, 17.0, Theme::text2()); }
    for (k, &i) in offers.iter().take(5).enumerate() {
        let yy = r2.y + 50.0 + k as f32 * 60.0;
        let t = ui.fit(&c.inbox[i].title, 18.0, r2.w - 180.0);
        ui.text_semi(&t, r2.x + 14.0, yy + 20.0, 18.0, Theme::text());
        if ui.button(Rect::new(r2.x + r2.w - 150.0, yy, 136.0, 32.0), crate::inhalt::t("es.eil.ansehen"), false) { v.sel_msg = Some(i); v.go(Page::Posteingang); }
    }
    // vacant posts: apply in four steps (laufbahn.toml [bewerbung])
    let vy = r2.y + r2.h * 0.5;
    ui.text_semi(crate::inhalt::t("ui.pages.freie_posten"), r2.x + 14.0, vy, 14.0, Theme::text2());
    let vak = c.vakanzen();
    if vak.is_empty() { ui.text(crate::inhalt::t("ui.pages.gerade_ist_kein_passender_posten"), r2.x + 14.0, vy + 28.0, 16.0, Theme::text2()); }
    for (k, &cl) in vak.iter().enumerate() {
        let yy = vy + 20.0 + k as f32 * 46.0;
        ui.text(&crate::tf!("ui.pages.praesidium", c.world.clubs[cl].name, c.persona_von(cl)), r2.x + 14.0, yy + 20.0, 16.0, Theme::text());
        if ui.button(Rect::new(r2.x + r2.w - 150.0, yy, 136.0, 32.0), crate::inhalt::t("lb.antwort.bewerben"), false) { match c.bewerben(cl) { Ok(()) => { v.toast(crate::inhalt::t("ui.pages.bewerbung_laeuft_fragen_im_posteingang"), true); } Err(e) => v.toast(&e, false) } }
    }
}
