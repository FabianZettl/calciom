use super::*;

// ================================================================ Posteingang
pub(super) fn posteingang(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // filter by kind: all, must decide, then the kind groups of postkarten.rs (Art::gruppe)
    let filters = [crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("gm.in.kz.muss"), crate::inhalt::t("menu.posteingang.reiter.6"), crate::inhalt::t("zentrale.k.kader"), crate::inhalt::t("bereich.transfers"), crate::inhalt::t("ui.anzeige.Präsidium"), crate::inhalt::t("profil.teil.8"), crate::inhalt::t("md.ui.posteingang.ungelesen")];
    tabs(ui, v, &filters);
    v.msg_filter = v.tab;
    let list: Vec<usize> = (0..c.inbox.len()).filter(|&i| {
        let m = &c.inbox[i];
        // after 120 days a message is only in „Alle“ (medien.toml posteingang.bestand_tage)
        let frisch = abs_day(m.season, m.day) + 120 > c.abs();
        match v.msg_filter { 0 => true, 1 => m.action != Action::None && !m.done && !matches!(m.action, Action::Report { .. }), 7 => !m.read, g => frisch && art_von(m).gruppe() == g }
    }).collect();
    let lw = 400.0;
    let area = Rect::new(x0() + 4.0, Y0, lw, hoehe() - Y0 - 16.0);
    ui.card(area);
    // Posteingang, five questions (1) and (2): one sentence on what waits, and this week's progress as a bar
    let abs = c.abs();
    let braucht = |m: &Msg| m.action != Action::None && !matches!(m.action, Action::Report { .. });
    let offen = c.inbox.iter().filter(|m| braucht(m) && !m.done).count();
    let heute = c.inbox.iter().filter(|m| braucht(m) && !m.done && m.pflicht > 0 && m.frist <= abs).count();
    let woche: Vec<&Msg> = c.inbox.iter().filter(|m| braucht(m) && abs_day(m.season, m.day) + 7 > abs).collect();
    let erledigt = woche.iter().filter(|m| m.done).count();
    let satz = if offen == 0 { crate::inhalt::t("ui.post.kopf_nichts").to_string() } else { crate::tf!(if offen == 1 { "ui.post.kopf_eins" } else { "ui.post.kopf" }, offen, heute) };
    ui.text_semi(&ui.fit(&satz, 15.0, area.w - 28.0), area.x + 14.0, area.y + 24.0, 15.0, if heute > 0 { Theme::problem() } else { Theme::text() });
    let anteil = if woche.is_empty() { 1.0 } else { erledigt as f32 / woche.len() as f32 };
    rrect(area.x + 14.0, area.y + 34.0, area.w - 28.0, 6.0, 3.0, Theme::ueberlagerung());
    rrect(area.x + 14.0, area.y + 34.0, (area.w - 28.0) * anteil.max(0.02), 6.0, 3.0, Theme::gut());
    ui.text(&crate::tf!("ui.post.kopf_woche", erledigt, woche.len()), area.x + 14.0, area.y + 56.0, 12.0, Theme::gedaempft());
    let oben = 64.0;
    let vis = ((area.h - oben - 10.0) / 60.0) as usize;
    let start = scroll_rows(ui, v, area, list.len(), vis);
    if list.is_empty() { ui.title(crate::inhalt::t("pe.hub.leer"), area.x + 16.0, area.y + oben + 40.0, 22.0, Theme::text2()); }
    for (k, &i) in list.iter().skip(start).take(vis).enumerate() {
        let r = Rect::new(area.x + 6.0, area.y + oben + 6.0 + k as f32 * 60.0, area.w - 22.0, 56.0);
        meldung_zeile(ui, c, r, &c.inbox[i], v.sel_msg == Some(i));
        if ui.hit(r) { v.sel_msg = Some(i); v.post_folgen = None; }
    }
    let rd = Rect::new(x0() + lw + 18.0, Y0, cbreite() - lw - 14.0, hoehe() - Y0 - 16.0);
    ui.card(rd);
    let Some(i) = v.sel_msg.filter(|&i| i < c.inbox.len()) else {
        ui.text(crate::inhalt::t("meldung.waehlen"), rd.x + 20.0, rd.y + 40.0, 19.0, Theme::text2());
        return;
    };
    c.inbox[i].read = true;
    meldung_zeichnen(ui, v, c, rd, i);
}
