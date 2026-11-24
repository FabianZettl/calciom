use super::*;

pub(super) fn trainerteam(ui: &mut Ui, v: &mut View, c: &mut Career) {
    c.stab_an();
    // only the people employed; applicants are on the staff market (Personalmarkt)
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(r, crate::inhalt::t("ui.pages.unser_trainerteam"), Theme::akzent());
    ui.text(&crate::tf!("ui.pages.trainingsqualitaet_medizin_neue_leute_fi", format!("{:.0}", c.trainingsqualitaet()), format!("{:.0}", c.medizinqualitaet())), r.x + 14.0, r.y + 56.0, 16.0, Theme::text2());
    let spalte = (r.w - 28.0) / 2.0;
    for i in 0..c.stab.len() {
        if i >= c.stab.len() { break; }
        let (sx, yy) = (r.x + 14.0 + (i % 2) as f32 * spalte, r.y + 96.0 + (i / 2) as f32 * 52.0);
        if yy > r.y + r.h - 30.0 { break; }
        let s = c.stab[i].clone();
        ui.text_semi(&s.name, sx, yy, 17.0, Theme::text());
        let bereich = if s.rolle == 0 { format!(" ({})", crate::personal::bereiche_t()[s.bereich as usize % 5]) } else { String::new() };
        let jahre = s.bis.saturating_sub(c.abs()) / crate::date::SEASON_DAYS;
        ui.text(&crate::tf!("ui.pages.koennen_im_jahr_vertrag_noch", crate::personal::rollen_de_t()[s.rolle as usize % 5], bereich, format!("{:.0}", s.f), money(s.gehalt), jahre.max(1), if jahre <= 1 { crate::inhalt::t("vertraege.jahr") } else { crate::inhalt::t("gl.kd.alter.unter") }), sx, yy + 20.0, 14.0, Theme::text2());
        if ui.button(Rect::new(sx + spalte - 130.0, yy - 14.0, 106.0, 28.0), crate::inhalt::t("aka.entlassen.ja"), false) { c.stab_entlassen(i); v.toast(crate::inhalt::t("aka.entlassen.ja"), true); break; }
    }
}

/// The staff market: applicants and free people for every role, with their values, salary and a hire button.
pub(super) fn personalmarkt(ui: &mut Ui, v: &mut View, c: &mut Career) {
    c.stab_an();
    let w = (cbreite() - 24.0) / 3.0;
    let karte = |k: usize, titel: &str, ui: &mut Ui| { let r = Rect::new(x0() + 4.0 + k as f32 * (w + 10.0), Y0, w, hoehe() - Y0 - 16.0); ui.card_titled(r, titel, if k == 1 { Theme::gold() } else { Theme::akzent() }); r };
    // name, value lines and salary on the left; the right column stays free for the button
    let zeile = |ui: &mut Ui, r: Rect, yy: f32, name: &str, werte: &[String], gehalt: f64| -> bool {
        ui.text_semi(&ui.fit(name, 17.0, r.w - 140.0), r.x + 14.0, yy, 17.0, Theme::text());
        for (k, w) in werte.iter().chain(std::iter::once(&crate::tf!("uebersicht.gehalt_jahr", money(gehalt)))).enumerate() {
            ui.text(&ui.fit(w, 14.0, r.w - 140.0), r.x + 14.0, yy + 20.0 + 18.0 * k as f32, 14.0, Theme::text2());
        }
        ui.button(Rect::new(r.x + r.w - 120.0, yy - 14.0, 106.0, 28.0), crate::inhalt::t("menu.staff.reiter.1"), false)
    };
    let bewerber = c.stab_bewerber();
    let spalten: [(&str, &[u8]); 2] = [(crate::inhalt::t("ui.pages.trainer_und_co_trainer"), &[0, 1]), (crate::inhalt::t("ui.pages.aerzte_physios_jugendleiter"), &[2, 3, 4])];
    for (k, (titel, rollen)) in [(0usize, spalten[0]), (2, spalten[1])] {
        let r = karte(k, titel, ui);
        let mut yy = r.y + 64.0;
        for (i, s) in bewerber.iter().enumerate().filter(|(_, s)| rollen.contains(&s.rolle)) {
            if yy > r.y + r.h - 50.0 { break; }
            let bereich = if s.rolle == 0 { format!(" ({})", crate::personal::bereiche_t()[s.bereich as usize % 5]) } else { String::new() };
            if zeile(ui, r, yy, &s.name, &[crate::tf!("ui.pages.koennen", crate::personal::rollen_de_t()[s.rolle as usize % 5], bereich, format!("{:.0}", s.f))], s.gehalt) {
                match c.stab_einstellen(i) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.eingestellt"), true), Err(e) => v.toast(&e, false) }
                return;
            }
            yy += 62.0;
        }
    }
    let r = karte(1, crate::inhalt::t("scouting.scouts"), ui);
    let mut yy = r.y + 64.0;
    for (i, (name, pe)) in c.bewerber().into_iter().enumerate() {
        if yy > r.y + r.h - 50.0 { break; }
        let w: Vec<String> = crate::scouting::faehigkeiten_t().iter().zip(pe.f.iter()).map(|(n, x)| format!("{n} {x:.0}")).collect();
        if zeile(ui, r, yy, &name, &[w[..2].join(" · "), w[2..].join(" · ")], pe.gehalt) {
            match c.scout_einstellen(i) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.scout_eingestellt"), true), Err(e) => v.toast(&e, false) }
            return;
        }
        yy += 80.0;
    }
}

// ================================================================ Trainerprofil
/// The coach market: vacant posts and hot seats with league, place, board, budget and goal; apply where we may.
pub(super) fn trainermarkt(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(r, t("tm.titel"), Theme::akzent());
    let hinweis = if c.fired { t("tm.hinweis_frei") } else { t("tm.hinweis_im_amt") };
    ui.text(hinweis, r.x + 14.0, r.y + 56.0, 14.0, Theme::text2());
    let markt = c.trainermarkt();
    if markt.is_empty() { ui.text(t("ui.pages.gerade_ist_kein_passender_posten"), r.x + 14.0, r.y + 90.0, 16.0, Theme::text2()); return; }
    let kopf = [(t("ui.wort.gross_verein"), 60.0), (t("tm.liga"), 360.0), (t("tm.platz"), 580.0), (t("tm.praesidium"), 650.0), (t("tm.budget"), 800.0), (t("tm.ziel"), 940.0)];
    for (n, x) in kopf { ui.text(n, r.x + x, r.y + 86.0, 13.0, Theme::gedaempft()); }
    let area = Rect::new(r.x, r.y + 96.0, r.w, r.h - 100.0);
    let vis = ((area.h - 8.0) / 44.0) as usize;
    let start = scroll_rows(ui, v, area, markt.len(), vis);
    for (k, &(cl, frei, ok, grund)) in markt.iter().skip(start).take(vis).enumerate() {
        let y = area.y + 6.0 + k as f32 * 44.0;
        let k_ = &c.world.clubs[cl];
        if ui.hover(Rect::new(r.x + 6.0, y, r.w - 12.0, 40.0)) { rrect(r.x + 6.0, y, r.w - 12.0, 40.0, 6.0, Theme::erhoehung()); }
        let (c1, _) = club_color(c, cl);
        ui.club_crest(r.x + 30.0, y + 20.0, 15.0, k_.id, c1, &k_.short);
        ui.text_semi(&ui.fit(&k_.name, 16.0, 280.0), r.x + 60.0, y + 26.0, 16.0, Theme::text());
        ui.text(&ui.fit(&c.world.leagues[k_.league].name, 14.0, 210.0), r.x + 360.0, y + 26.0, 14.0, Theme::text2());
        ui.text(&c.world.rank_of(cl).to_string(), r.x + 580.0, y + 26.0, 14.0, Theme::text());
        ui.text(t(&format!("tm.persona.{}", c.persona_von(cl))), r.x + 650.0, y + 26.0, 14.0, Theme::text2()); // roh
        ui.text(&money(round_money(c.world.revenue(cl) * 0.22)), r.x + 800.0, y + 26.0, 14.0, Theme::text());
        ui.text(&crate::tf!("jv.platz", k_.goal_pos), r.x + 940.0, y + 26.0, 14.0, Theme::text2());
        let bx = Rect::new(r.x + r.w - 190.0, y + 5.0, 176.0, 30.0);
        if ok {
            if ui.button(bx, t("lb.antwort.bewerben"), true) { match c.bewerben(cl) { Ok(()) => v.toast(t("ui.pages.bewerbung_laeuft_fragen_im_posteingang"), true), Err(e) => v.toast(&e, false) } }
        } else {
            let w = if !frei { t("tm.wackelt") } else { t(&format!("tm.grund.{grund}")) }; // roh
            ui.text_right(w, r.x + r.w - 20.0, y + 26.0, 13.0, if frei { Theme::gedaempft() } else { Theme::achtung() });
        }
    }
}
