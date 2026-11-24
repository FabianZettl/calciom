use super::*;

// ================================================================ player profile modal
pub fn player_modal(ui: &mut Ui, v: &mut View, c: &mut Career, p: usize) {
    let r = Rect::new(breite() / 2.0 - 380.0, 110.0, 760.0, 640.0);
    rrect(r.x, r.y, r.w, r.h, 10.0, Theme::ueberlagerung());
    let pl = c.world.players[p].clone();
    let club = if pl.team >= 0 { pl.team as usize } else { c.own };
    let (c1, c2) = club_color(c, club);
    rrect(r.x, r.y, r.w, 130.0, 10.0, c1);
    draw_rectangle(r.x, r.y + 100.0, r.w, 30.0, alpha(Theme::ueberlagerung(), 0.9));
    ui.player_photo(r.x + 80.0, r.y + 72.0, 48.0, pl.id, &pl.name);
    ui.title(&pl.name, r.x + 150.0, r.y + 52.0, 34.0, c2);
    let clubname = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else if pl.team == -1 { crate::inhalt::t("ui.wort.vereinslos").into() } else { crate::inhalt::t("ui.pages.karriereende").into() };
    ui.pos_tag(r.x + 150.0, r.y + 78.0, pos_de(&pl.pos));
    ui.text_semi(&crate::tf!("ui.pages.jahre_verein", pl.age, clubname), r.x + 194.0, r.y + 84.0, 18.0, c2);
    let (lo, hi) = c.ovr_range(p);
    let (pa, pb) = c.pot_range(p);
    let mut y = r.y + 170.0;
    let items = [
        (crate::inhalt::t("ui.pages.staerke"), if hi - lo <= 1.0 { format!("{:.0}", pl.ovr) } else { format!("{lo}–{hi}") }),
        (crate::id("POTENZIAL"), format!("{pa}–{pb}")),
        (crate::id("MARKTWERT"), money(pl.value)),
        (crate::id("GEHALT"), money(pl.wage)),
        (crate::id("VERTRAG"), crate::tf!("zentrale.bis", date::START_YEAR as u32 + pl.contract_end + 1)),
    ];
    for (i, (k, val)) in items.iter().enumerate() {
        let x = r.x + 24.0 + i as f32 * 146.0;
        ui.text_semi(k, x, y, 13.0, Theme::gedaempft());
        ui.title(val, x, y + 30.0, 24.0, if i == 0 { Theme::skala((lo + hi) / 2.0) } else { Theme::text() });
    }
    y += 70.0;
    let own = pl.team == c.own as i32;
    let unser = c.gehoert_uns(p);
    if unser || c.know_of(p) >= 0.6 {
        for (i, (k, val, col)) in [(crate::inhalt::t("uebersicht.fitness"), pl.fit, Theme::gut()), (crate::inhalt::t("kader.moral"), pl.morale, Theme::akzent()), (crate::inhalt::t("zentrale.k.form"), (pl.form + 6.0) / 12.0 * 100.0, Theme::achtung())].iter().enumerate() {
            let yy = y + i as f32 * 30.0;
            ui.text(k, r.x + 24.0, yy + 12.0, 16.0, Theme::text2());
            ui.bar(r.x + 110.0, yy + 2.0, 260.0, 8.0, val / 100.0, *col);
        }
        if pl.ps.an {
            let ty = crate::mensch::typen(&pl);
            let typ = ty.first().map(|t| crate::mensch::typ_name(t)).unwrap_or(crate::inhalt::t("gs.kontext.form.mittel"));
            ui.text(&crate::tf!("profil.kurz_text", typ, crate::mensch::rolle_name(pl.ps.rolle)), r.x + 400.0, y + 12.0, 16.0, Theme::text());
        } else {
            let pers = [crate::inhalt::t("ui.pages.sachlich_und_fair"), crate::inhalt::t("ui.pages.hat_den_blick_fuers_geld"), crate::inhalt::t("ui.pages.treu_und_loyal"), crate::inhalt::t("ui.pages.sehr_ehrgeizig"), crate::id("heimatverbunden")][pl.persona as usize];
            ui.text(&crate::tf!("pm.zeile.persoenlichkeit", pers), r.x + 400.0, y + 12.0, 16.0, Theme::text());
        }
        ui.text(&crate::tf!("ui.pages.ins", ideas_t()[c.tactic.idea()], fit_text(c.system_fit(p))), r.x + 400.0, y + 42.0, 16.0, Theme::text());
        if pl.inj > 0 { ui.text(&crate::tf!("ui.pages.verletzt_tage", pl.inj_name, pl.inj), r.x + 400.0, y + 72.0, 16.0, Theme::problem()); }
    } else {
        // what the knowledge shows (scouting.toml sterne.stufen): stars, then 30 % / 60 % / all abilities, character, traits
        let st = c.sicht(p);
        let (lo2, hi2) = c.ovr_range(p);
        let mut zeilen = vec![];
        if st >= 1 { zeilen.push(crate::tf!("ui.pages.einschaetzung_bis", crate::scouting::sterne_text(crate::scouting::sterne(lo2)), crate::scouting::sterne_text(crate::scouting::sterne(hi2)))); }
        if st >= 2 && pl.attrs.len() == 25 {
            let anteil = crate::mensch::arr("scouting", "sterne.einzelwerte_anteil")[st].as_f64().unwrap();
            let n = (25.0 * anteil).round() as usize;
            let mut idx: Vec<usize> = (0..25).collect();
            idx.sort_by(|a, b| pl.attrs[*b].partial_cmp(&pl.attrs[*a]).unwrap());
            let w: Vec<String> = idx.iter().take(n.min(6)).map(|&k| format!("{} {:.0}", crate::pesdb::attr_de_t()[k], pl.attrs[k])).collect();
            zeilen.push(crate::tf!("ui.pages.bekannt_von_25_werten", n, w.join(", ")));
        }
        if st >= 3 && pl.ps.an { if let Some(t) = crate::mensch::typen(&pl).first() { zeilen.push(crate::tf!("pm.zeile.persoenlichkeit", crate::mensch::typ_name(t))); } }
        if st >= 5 { zeilen.push(crate::tf!("ui.pages.verletzungsanfaellig", if pl.inj > 0 || pl.age >= 31 { crate::inhalt::t("ui.pages.eher_ja") } else { crate::inhalt::t("ui.wort.nein") })); }
        if zeilen.is_empty() { zeilen.push(crate::inhalt::t("ui.pages.ueber_ihn_weisst_du_noch").into()); }
        for (k, z) in zeilen.iter().enumerate() { ui.text(&ui.fit(z, 16.0, r.w - 48.0), r.x + 24.0, y + 12.0 + k as f32 * 24.0, 16.0, Theme::text2()); }
    }
    y += 100.0;
    if own && pl.ps.an {
        let mut tr = pl.ps.treiber.clone();
        tr.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        let words: Vec<String> = tr.iter().take(4).filter(|x| x.1.abs() >= 0.5).map(|x| format!("{} {}", crate::mensch::treiber_name(&x.0), if x.1 > 0.0 { crate::id("hebt") } else { crate::inhalt::t("ui.pages.drueckt") })).collect();
        let mut st: Vec<(usize, f32)> = (0..9).map(|i| (i, crate::mensch::cw(&pl, i))).collect();
        st.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let stark = crate::tf!("ui.pages.ausgepraegt", crate::mensch::wert_name(st[0].0), crate::mensch::wert_name(st[1].0));
        ui.text(&if words.is_empty() { crate::inhalt::t("ui.pages.nichts_drueckt_auf_seine_stimmung").to_string() } else { crate::tf!("ui.pages.was_ihn_bewegt", words.join(", ")) }, r.x + 24.0, y, 15.0, Theme::text2());
        ui.text(&crate::tf!("ui.pages.vertrauen_in_dich", stark, match c.vertrauen(p) as i32 { 0..=29 => "gering", 30..=59 => "mittel", _ => crate::inhalt::t("r.ui.demo.gross") }), r.x + 24.0, y + 22.0, 15.0, Theme::text2());
    }
    y += 44.0;
    ui.text_semi(crate::inhalt::t("ui.wort.gross_saison"), r.x + 24.0, y, 14.0, Theme::gedaempft());
    ui.text(&crate::tf!("ui.pages.spiele_tore_vorlagen_note", pl.apps, pl.goals, pl.assists, if pl.rn > 0 { format!("{:.2}", pl.avg_rating()) } else { "–".into() }), r.x + 24.0, y + 26.0, 18.0, Theme::text());
    for (j, rt) in pl.last.iter().enumerate() { ui.note_pill(r.x + 50.0 + j as f32 * 52.0, y + 56.0, *rt); }
    y += 90.0;
    if c.know_of(p) < 1.0 && !own {
        let rep = c.report_text(p);
        ui.paragraph(&rep, r.x + 24.0, y, 15.0, r.w - 48.0, Theme::text2());
    }
    let by = r.y + r.h - 60.0;
    if own {
        if ui.button(Rect::new(r.x + 24.0, by, 180.0, 40.0), crate::inhalt::t("aktion.verlaengern"), false) { match c.open_neg(p) { Ok(i) => { v.modal = Modal::Neg(i); return; } Err(e) => v.toast(&e, false) } }
        if ui.button(Rect::new(r.x + 216.0, by, 180.0, 40.0), if pl.listed { crate::inhalt::t("kue.btn.liste_aus") } else { crate::inhalt::t("ui.pages.auf_transferliste") }, pl.listed) { c.world.players[p].listed = !pl.listed; }
        ui.text(&crate::tf!("liga.nummer", pl.no), r.x + 610.0, by - 26.0, 15.0, Theme::text2());
        if ui.button(Rect::new(r.x + 660.0, by - 44.0, 26.0, 26.0), "−", false) { if let Err(e) = c.rueckennummer(p, -1) { v.toast(&e, false); } }
        if ui.button(Rect::new(r.x + 690.0, by - 44.0, 26.0, 26.0), "+", false) { if let Err(e) = c.rueckennummer(p, 1) { v.toast(&e, false); } }
        if pl.leihe.as_ref().map_or(false, |l| l.von != c.own && l.option > 0.0) {
            if ui.button_gold(Rect::new(r.x + 24.0, by - 50.0, 260.0, 40.0), &crate::tf!("ui.pages.kaufoption", money(pl.leihe.as_ref().unwrap().option))) { match c.kaufoption(p) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.kaufoption_gezogen"), true), Err(e) => v.toast(&e, false) } }
        } else if pl.leihe.is_none() && ui.button(Rect::new(r.x + 24.0, by - 50.0, 180.0, 40.0), crate::inhalt::t(if pl.ps.leihliste { "ui.leihe.von_liste" } else { "ui.leihe.auf_leihliste" }), pl.ps.leihliste) {
            if pl.ps.leihliste { c.world.players[p].ps.leihliste = false; } else { match c.leihe_abgeben(p) { Ok(()) => v.toast(crate::inhalt::t("ui.leihe.auf_liste"), true), Err(e) => v.toast(&e, false) } }
        }
        // first or extended squad
        if ui.button(Rect::new(r.x + 24.0, by - 94.0, 260.0, 34.0), crate::inhalt::t(if pl.ps.erweitert { "ui.kader.in_erste" } else { "ui.kader.in_erweitert" }), pl.ps.erweitert) { c.world.players[p].ps.erweitert = !pl.ps.erweitert; }
        if let Some((pos, bis)) = &pl.ps.umschulung {
            ui.text(&crate::tf!("ui.pages.lernt_noch_tage", pos_de(pos), bis.saturating_sub(c.abs())), r.x + 300.0, by - 26.0, 15.0, Theme::text2());
            if ui.button(Rect::new(r.x + 640.0, by - 44.0, 120.0, 28.0), crate::inhalt::t("ui.personal.abbrechen"), false) { c.umschulung_abbrechen(p); }
        }
        else {
            ui.text(crate::inhalt::t("ui.pages.umschulen"), r.x + 300.0, by - 26.0, 14.0, Theme::gedaempft());
            for (k, pos) in [crate::id("GK"), crate::id("CB"), crate::id("LB"), crate::id("RB"), crate::id("DMF"), crate::id("CMF"), crate::id("AMF"), crate::id("LMF"), crate::id("RMF"), crate::id("LWF"), crate::id("RWF"), crate::id("SS"), crate::id("CF")].iter().enumerate() {
                if *pos == pl.pos { continue; }
                if ui.button(Rect::new(r.x + 380.0 + k as f32 * 28.0, by - 44.0, 26.0, 26.0), &pos_de(pos)[..pos_de(pos).char_indices().nth(2).map(|x| x.0).unwrap_or(pos_de(pos).len())], false) { match c.umschulen(p, pos) { Ok(t) => v.toast(&crate::tf!("ui.pages.umschulung_dauert_tage", t), true), Err(e) => v.toast(&e, false) } }
            }
        }
        if ui.button(Rect::new(r.x + 408.0, by, 160.0, 40.0), crate::inhalt::t("aka.entlassen.ja"), false) { if c.squad().len() > 18 { c.release(p); v.toast(crate::inhalt::t("ui.pages.vertrag_aufgeloest"), true); v.modal = Modal::None; return; } else { v.toast(crate::inhalt::t("ui.pages.der_kader_ist_zu_klein"), false); } }
    } else if pl.team != -2 && !unser {
        if ui.button_gold(Rect::new(r.x + 24.0, by, 200.0, 40.0), crate::inhalt::t("ui.wort.gross_verhandeln")) { match c.open_neg(p) { Ok(i) => { v.modal = Modal::Neg(i); return; } Err(e) => v.toast(&e, false) } }
        // a loan request: duration and buy terms, then the club answers (Fabian 23.11.)
        if pl.team >= 0 {
            const MONATE: [u32; 4] = [6, 12, 18, 24];
            let (m, k) = v.leih_wahl;
            if let Some(i) = ui.segment(Rect::new(r.x + 236.0, by - 46.0, 300.0, 34.0), &[crate::inhalt::t("ui.leihe.m6"), crate::inhalt::t("ui.leihe.m12"), crate::inhalt::t("ui.leihe.m18"), crate::inhalt::t("ui.leihe.m24")], m) { v.leih_wahl.0 = i; }
            if let Some(i) = ui.segment(Rect::new(r.x + 546.0, by - 46.0, 330.0, 34.0), &[crate::inhalt::t("ui.leihe.ohne_kauf"), crate::inhalt::t("ui.leihe.kaufoption"), crate::inhalt::t("ui.leihe.kaufpflicht")], k) { v.leih_wahl.1 = i; }
            if ui.button(Rect::new(r.x + 236.0, by, 200.0, 40.0), crate::inhalt::t("ui.leihe.anfragen"), false) { match c.leihanfrage(p, MONATE[m], k as u8) { Ok(t) => { v.toast(&t, true); v.modal = Modal::None; return; } Err(e) => v.toast(&e, false) } }
        }
    }
    if ui.button(Rect::new(r.x + r.w - 160.0, by, 136.0, 40.0), crate::inhalt::t("aktion.schliessen"), false) || is_key_pressed(KeyCode::Escape) { v.modal = Modal::None; }
    if ui.button(Rect::new(r.x + r.w - 160.0, r.y + 12.0, 136.0, 30.0), crate::inhalt::t("ui.pages.ganze_seite"), false) { v.sel_player = Some(p); v.modal = Modal::None; v.go(Page::Spielerprofil); }
}
