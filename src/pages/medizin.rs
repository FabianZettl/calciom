use super::*;

// ================================================================ Medizin
pub(super) fn medizin(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("gl.kz.verletzte"), crate::inhalt::t("vorschau.belastung")]);
    let mut sq = c.squad();
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    if v.tab == 0 {
        sq.retain(|&p| c.world.players[p].inj > 0);
        if sq.is_empty() { ui.title(crate::inhalt::t("pk.karte.keine_ausfaelle"), area.x + 20.0, area.y + 50.0, 28.0, Theme::gut()); ui.text(crate::inhalt::t("ui.pages.der_mannschaftsarzt_meldet_keine_ausfael"), area.x + 20.0, area.y + 80.0, 18.0, Theme::text2()); }
        // one row per injured player: rehab phase, the physio's estimate and the doctor's options (medizin.rs)
        let mut meldung = None;
        for (k, &p) in sq.iter().enumerate() {
            let y = area.y + 16.0 + k as f32 * 104.0;
            if y + 96.0 > area.y + area.h { break; }
            let Some(st) = c.verletzung_stand(p) else { continue };
            let (von, bis) = c.prognose(p);
            let colw = (area.w - 616.0 - 300.0 - 20.0).clamp(160.0, 380.0);
            let pl = &c.world.players[p];
            ui.pos_tag(area.x + 16.0, y + 22.0, pos_de(&pl.pos));
            ui.player_photo(area.x + 70.0, y + 22.0, 18.0, pl.id, &pl.name);
            ui.text_semi(&pl.name, area.x + 96.0, y + 20.0, 20.0, Theme::text());
            ui.text(&pl.inj_name, area.x + 96.0, y + 44.0, 16.0, Theme::problem());
            ui.text(&crate::tf!("med.hinweis.prognose", crate::tf!("med.dauer.tage", von, bis)), area.x + 300.0, y + 20.0, 16.0, Theme::text());
            ui.text_passend(&crate::tf!("med.hinweis.phase", crate::inhalt::t(&format!("med.phase.{}", ["behandlung", "aufbau", "mannschaftstraining", "einsatzbereit"][st.phase])), st.phase_rest), area.x + 300.0, y + 44.0, 15.0, colw, 1, Theme::text2()); // roh
            // the four phases as a bar
            let bw = colw;
            for i in 0..4 {
                let x = area.x + 300.0 + i as f32 * (bw / 4.0);
                draw_rectangle(x, y + 58.0, bw / 4.0 - 4.0, 8.0, if i < st.phase { Theme::gut() } else if i == st.phase { Theme::achtung() } else { Theme::ueberlagerung() });
            }
            let bx = area.x + area.w - 16.0;
            let risiko = if st.rueckfall > 1.0 { crate::tf!("med.hinweis.rueckfall_frueh", format!("{:.1}", st.rueckfall), crate::regeln::f("medizin", "reha.rueckfall_tage")) } else { crate::inhalt::t("med.hinweis.frueh_ohne").to_string() };
            if ui.button(Rect::new(bx - 600.0, y + 6.0, 190.0, 32.0), crate::inhalt::t("ui.med.einsetzen"), false) { meldung = Some((p, c.frueh_einsetzen(p))); }
            ui.text_passend(&risiko, bx - 600.0, y + 58.0, 13.0, 190.0, 2, Theme::gedaempft());
            if let Some((_, _, _, kosten)) = c.reha_wirkung(p) {
                if ui.button(Rect::new(bx - 400.0, y + 6.0, 190.0, 32.0), crate::inhalt::t("med.akt.reha"), false) { meldung = Some((p, c.reha_programm(p))); }
                ui.text_passend(&crate::tf!("ui.med.kosten", money(kosten)), bx - 400.0, y + 58.0, 13.0, 190.0, 2, Theme::gedaempft());
            }
            if c.fitspritze_moeglich(p) {
                if ui.button(Rect::new(bx - 200.0, y + 6.0, 190.0, 32.0), crate::inhalt::t("med.akt.fitspritze"), false) { meldung = Some((p, c.fitspritze(p))); }
                ui.text_passend(&crate::tf!("med.hinweis.fit_rueckfall", format!("{:.0}", crate::regeln::f("medizin", "fitspritze.rueckfall_faktor")), crate::regeln::f("medizin", "fitspritze.rueckfall_tage")), bx - 200.0, y + 58.0, 13.0, 190.0, 2, Theme::gedaempft());
            }
            draw_rectangle(area.x + 14.0, y + 96.0, area.w - 28.0, 1.0, Theme::kontur());
        }
        if let Some((p, r)) = meldung {
            match r {
                Ok(text) => { let n = c.world.players[p].name.clone(); c.msg(3, crate::inhalt::t("med.vg.absender"), &crate::tf!("ui.med.entscheidung", n), &text, Action::None); v.toast(&text, true); }
                Err(e) => v.toast(&e, false),
            }
        }
    } else {
        sq.sort_by(|a, b| c.world.players[*a].fit.partial_cmp(&c.world.players[*b].fit).unwrap());
        let vis = ((area.h - 20.0) / 36.0) as usize;
        let start = scroll_rows(ui, v, area, sq.len(), vis);
        for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
            let pl = &c.world.players[p];
            let y = area.y + 14.0 + k as f32 * 36.0;
            ui.pos_tag(area.x + 16.0, y + 14.0, pos_de(&pl.pos));
            ui.player_photo(area.x + 66.0, y + 14.0, 13.0, pl.id, &pl.name);
            ui.text(&pl.name, area.x + 86.0, y + 20.0, 17.0, Theme::text());
            let col = if pl.fit >= 80.0 { Theme::gut() } else if pl.fit >= 60.0 { Theme::achtung() } else { Theme::problem() };
            ui.bar(area.x + 320.0, y + 10.0, 400.0, 10.0, pl.fit / 100.0, col);
            ui.text(&format!("{:.0} %", pl.fit), area.x + 730.0, y + 20.0, 16.0, col);
            ui.text(if pl.fit < 60.0 { crate::inhalt::t("ui.pages.pause_empfohlen") } else if pl.fit < 80.0 { crate::inhalt::t("ui.wort.belastet") } else { crate::inhalt::t("ui.wort.frisch") }, area.x + 820.0, y + 20.0, 16.0, Theme::text2());
            // strain light: colour and the biggest driver (medizin.py ampel)
            let (_, farbe, treiber) = c.ampel_von(p);
            let fc = [Theme::gut(), Theme::achtung(), Theme::problem()][farbe as usize];
            draw_circle(area.x + 1010.0, y + 14.0, 8.0, fc);
            ui.text(&format!("{} · {}", crate::inhalt::t(&format!("med.ampel.{}", ["gruen", "gelb", "rot"][farbe as usize])), crate::inhalt::t(&format!("med.ampel.{}", treiber[0].0))), area.x + 1026.0, y + 20.0, 16.0, Theme::text2()); // roh
        }
    }
}
