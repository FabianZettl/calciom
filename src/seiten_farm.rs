// ---------------------------------------------------------------- second team and farm teams (farm.rs)
/// Two lists side by side: first team and second team, a button moves a player across (no fee, at any time).
fn zweite_mannschaft_tab(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let Some(z) = c.zweite_mannschaft() else { ui.empty(Rect::new(X0 + 4.0, Y0, cbreite(), 200.0), crate::inhalt::t("ui.seiten_farm.dein_verein_hat_keine_zweite")); return };
    let w2 = (cbreite() - 12.0) / 2.0;
    let (ueber, hoechstens) = c.world.altersregel(z);
    let mut wechsel: Option<usize> = None;
    for (k, club) in [c.own, z].into_iter().enumerate() {
        let r = Rect::new(X0 + 4.0 + k as f32 * (w2 + 12.0), Y0, w2, hoehe() - Y0 - 16.0);
        let lg = c.world.leagues[c.world.clubs[club].league].name.clone();
        let mut sq: Vec<usize> = c.world.clubs[club].squad.iter().copied().filter(|&p| c.world.players[p].youth == 0).collect();
        sq.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
        let titel = crate::tf!("ui.seiten_farm.spieler", c.world.clubs[club].name, lg, sq.len());
        ui.card_titled(r, &titel, if k == 0 { Theme::akzent() } else { Theme::gold() });
        if k == 1 { ui.text(&crate::tf!("ui.seiten_farm.altersregel_hoechstens_spieler_aelter_al", hoechstens, ueber), r.x + 14.0, r.y + 58.0, 13.0, Theme::gedaempft()); }
        let vis = ((r.h - 80.0) / 30.0) as usize;
        for (j, &p) in sq.iter().take(vis).enumerate() {
            let pl = &c.world.players[p];
            let y = r.y + 72.0 + j as f32 * 30.0;
            let rr = Rect::new(r.x + 8.0, y, r.w - 16.0, 28.0);
            if j % 2 == 1 { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::erhoehung(), 0.6)); }
            ui.pos_tag(rr.x + 22.0, rr.y + 14.0, pos_de(&pl.pos));
            ui.player_photo(rr.x + 62.0, rr.y + 14.0, 12.0, pl.id, &pl.name);
            let nm = ui.fit(&pl.name, 15.0, rr.w - 280.0);
            ui.text_semi(&nm, rr.x + 80.0, rr.y + 19.0, 15.0, Theme::text());
            ui.text(&crate::tf!("ui.wort.alter_kurz", pl.age), rr.x + rr.w - 190.0, rr.y + 19.0, 14.0, if k == 1 && pl.age > ueber { Theme::achtung() } else { Theme::text2() });
            ui.pill(rr.x + rr.w - 128.0, rr.y + 14.0, pl.ovr, 34.0);
            if !c.world.spielberechtigt(p, z) && k == 1 { ui.text(crate::inhalt::t("ui.wort.gesperrt"), rr.x + rr.w - 260.0, rr.y + 19.0, 12.0, Theme::problem()); }
            if ui.button(Rect::new(rr.x + rr.w - 96.0, rr.y + 2.0, 92.0, 24.0), if k == 0 { crate::inhalt::t("ui.seiten_farm.nach_unten") } else { crate::inhalt::t("ui.seiten_farm.nach_oben") }, false) { wechsel = Some(p); }
            if ui.hit(Rect::new(rr.x, rr.y, rr.w - 100.0, rr.h)) { v.modal = Modal::Player(p); }
        }
    }
    if let Some(p) = wechsel { match c.mannschaft_wechseln(p) { Ok(t) => v.toast(&t, true), Err(e) => v.toast(&e, false) } }
}

/// The page "Kooperationen" (Verein): the farm teams with contract, loans and buttons, the search and the applications.
fn kooperationen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let wl = cbreite() * 0.5 - 6.0;
    let l = Rect::new(X0 + 4.0, Y0 - 40.0, wl, hoehe() - Y0 + 24.0);
    let hoechstens = crate::regeln::f("farm", "koop.hoechstens") as usize;
    let eigene: Vec<usize> = (0..c.world.koops.len()).filter(|&i| c.world.koops[i].haupt == c.own).collect();
    ui.card_titled(l, &crate::tf!("ui.seiten_farm.unsere_farmteams_von", eigene.len(), hoechstens), Theme::akzent());
    if eigene.is_empty() { ui.paragraph(crate::inhalt::t("ui.seiten_farm.noch_kein_partnerverein_ein_farmteam"), l.x + 14.0, l.y + 60.0, 16.0, l.w - 28.0, Theme::text2()); }
    let mut aktion: Option<(usize, u8)> = None;
    let mut rueck: Option<usize> = None;
    let mut y = l.y + 52.0;
    for &ki in &eigene {
        let k = c.world.koops[ki].clone();
        let pc = &c.world.clubs[k.partner];
        let r = Rect::new(l.x + 10.0, y, l.w - 20.0, 270.0);
        rrect(r.x, r.y, r.w, r.h, 8.0, Theme::erhoehung());
        ui.club_crest(r.x + 30.0, r.y + 32.0, 20.0, pc.id, hex(pc.c1), &pc.short);
        ui.title(&ui.fit(&pc.name, 24.0, r.w - 80.0), r.x + 62.0, r.y + 32.0, 24.0, Theme::text());
        ui.text(&crate::tf!("ui.seiten_farm.staerke", c.world.leagues[pc.league].name, format!("{:.0}", pc.level)), r.x + 62.0, r.y + 54.0, 15.0, Theme::text2());
        let bei = c.leihen_bei(k.partner);
        let z = [crate::tf!("ui.seiten_farm.vertrag_bis_im_jahr", date::season_label(k.bis), money(k.betrag)),
            crate::tf!("ui.seiten_farm.leihplaetze_von", bei.len(), k.plaetze, if k.min_leihen > 0 { crate::tf!("ui.seiten_farm.zugesagt_mindestens", k.min_leihen) } else { String::new() }),
            if k.bleiben { crate::inhalt::t("ui.seiten_farm.spielzeit_zusage_leihspieler_bleiben_die").into() } else { crate::inhalt::t("ui.seiten_farm.leihspieler_koennen_zurueckgerufen_werde").into() }];
        for (j, t) in z.iter().enumerate() { ui.text(t, r.x + 14.0, r.y + 84.0 + j as f32 * 22.0, 15.0, Theme::text()); }
        for (j, &p) in bei.iter().take(3).enumerate() {
            let yy = r.y + 158.0 + j as f32 * 26.0;
            let pl = &c.world.players[p];
            let t = ui.fit(&format!("{} · {}", pl.name, pl.leihe.as_ref().map(|x| x.grund.clone()).unwrap_or_default()), 14.0, r.w - 150.0);
            ui.text(&t, r.x + 14.0, yy + 16.0, 14.0, Theme::text2());
            if ui.button(Rect::new(r.x + r.w - 124.0, yy, 112.0, 22.0), crate::inhalt::t("ui.seiten.zurueckrufen"), false) { rueck = Some(p); }
        }
        if ui.button(Rect::new(r.x + 14.0, r.y + r.h - 40.0, 140.0, 30.0), crate::inhalt::t("ui.pages.verlaengern"), false) { aktion = Some((ki, 0)); }
        if ui.button(Rect::new(r.x + 164.0, r.y + r.h - 40.0, 140.0, 30.0), crate::inhalt::t("ui.seiten_farm.kuendigen"), false) { aktion = Some((ki, 1)); }
        y += r.h + 10.0;
    }
    if let Some((ki, a)) = aktion { if a == 0 { c.koop_verlaengern(ki); v.toast(crate::inhalt::t("ui.seiten_farm.um_eine_saison_verlaengert"), true); } else { let t = c.koop_kuendigen(ki); v.toast(&t, true); } }
    if let Some(p) = rueck { match c.leihe_zurueckrufen(p) { Ok(t) => v.toast(&t, true), Err(e) => v.toast(&e, false) } }

    // right: search and applications
    let r = Rect::new(X0 + cbreite() * 0.5 + 10.0, Y0 - 40.0, wl, hoehe() - Y0 + 24.0);
    ui.card_titled(r, crate::inhalt::t("ui.seiten_farm.partnerverein_suchen"), Theme::gold());
    let mut y = r.y + 56.0;
    if let Some(s) = c.koop_suche.clone() {
        ui.text_semi(&crate::tf!("ui.seiten_farm.die_suche_laeuft_bis", date::long(s.bis / date::SEASON_DAYS, s.bis % date::SEASON_DAYS)), r.x + 14.0, y, 17.0, Theme::text());
        ui.text(&crate::tf!("ui.seiten_farm.bis_stufen_unter_uns_leihplaetze", if s.nur_land { crate::inhalt::t("ui.seiten_farm.eigenes_land") } else { crate::inhalt::t("ui.seiten_farm.alle_laender") }, s.stufe_von, s.stufe_bis, s.plaetze, money(s.max_betrag)), r.x + 14.0, y + 24.0, 14.0, Theme::text2());
        y += 50.0;
    } else if eigene.len() < hoechstens {
        let f = &mut v.koop_form;
        if f.3 <= 0.0 { f.3 = (c.koop_rahmen() / 50000.0).round() * 50000.0; }
        let zeilen: [(String, String); 4] = [(crate::inhalt::t("verein.land").into(), if f.0 { crate::inhalt::t("ui.seiten_farm.nur_unser_land").into() } else { crate::inhalt::t("ui.seiten_farm.alle_laender").into() }), (crate::inhalt::t("ui.seiten_farm.ligastufe").into(), crate::tf!("ui.seiten_farm.bis_stufen_unter_uns", f.1, f.2)), (crate::inhalt::t("ui.seiten_farm.leihplaetze").into(), f.4.to_string()), (crate::inhalt::t("ui.seiten_farm.jahresbetrag_hoechstens").into(), money(f.3))];
        for (j, (t, w)) in zeilen.iter().enumerate() {
            let yy = y + j as f32 * 36.0;
            ui.text(t, r.x + 14.0, yy + 20.0, 15.0, Theme::text2());
            ui.text_semi(w, r.x + 210.0, yy + 20.0, 15.0, Theme::text());
            let (minus, plus) = (Rect::new(r.x + r.w - 84.0, yy, 32.0, 28.0), Rect::new(r.x + r.w - 46.0, yy, 32.0, 28.0));
            let (m, p) = (ui.button(minus, "−", false), ui.button(plus, "+", false));
            match j {
                0 => if m || p { f.0 = !f.0; },
                1 => { if m && f.2 > f.1 { f.2 -= 1; } else if m && f.1 > 1 { f.1 -= 1; } if p && f.2 < 4 { f.2 += 1; } }
                2 => { if m && f.4 > 1 { f.4 -= 1; } if p && f.4 < 8 { f.4 += 1; } }
                _ => { if m && f.3 > 100000.0 { f.3 -= 100000.0; } if p { f.3 += 100000.0; } }
            }
        }
        y += 4.0 * 36.0 + 6.0;
        let ueber = v.koop_form.3 > c.koop_rahmen();
        ui.text(&if ueber { crate::tf!("ui.seiten_farm.ueber_dem_rahmen_von_das", money(c.koop_rahmen())) } else { crate::tf!("ui.seiten_farm.im_rahmen_von_im_jahr", money(c.koop_rahmen())) }, r.x + 14.0, y + 14.0, 14.0, if ueber { Theme::achtung() } else { Theme::gedaempft() });
        y += 26.0;
        if ui.button_gold(Rect::new(r.x + 14.0, y, 260.0, 38.0), crate::inhalt::t("ui.seiten_farm.suche_starten_14_tage")) {
            let f = v.koop_form;
            match c.koop_suche_starten(crate::farm::KoopSuche { bis: 0, nur_land: f.0, stufe_von: f.1, stufe_bis: f.2, plaetze: f.4, max_betrag: f.3 }) { Ok(t) => v.toast(&t, true), Err(e) => v.toast(&e, false) }
        }
        y += 52.0;
    } else { ui.text(&crate::tf!("ui.farm.mehr_als_farmteams_gleichzeitig_gehen", hoechstens), r.x + 14.0, y + 10.0, 15.0, Theme::text2()); y += 36.0; }
    ui.text_semi(&crate::tf!("ui.seiten_farm.bewerbungen", c.koop_angebote.len()), r.x + 14.0, y + 12.0, 16.0, Theme::text());
    y += 22.0;
    if c.koop_angebote.is_empty() { ui.text(crate::inhalt::t("ui.seiten_farm.noch_keine_bewerbung_sie_kommen"), r.x + 14.0, y + 18.0, 14.0, Theme::gedaempft()); }
    let mut antwort: Option<(u32, &str)> = None;
    let n_a = c.koop_angebote.len();
    for (j, a) in c.koop_angebote.clone().into_iter().enumerate() {
        if y > r.y + r.h - 150.0 { ui.text(&crate::tf!("ui.seiten_farm.weitere_im_posteingang", n_a - j), r.x + 14.0, y + 16.0, 14.0, Theme::gedaempft()); break; }
        let cl = &c.world.clubs[a.club];
        let b = Rect::new(r.x + 10.0, y, r.w - 20.0, 140.0);
        rrect(b.x, b.y, b.w, b.h, 8.0, Theme::erhoehung());
        ui.club_crest(b.x + 26.0, b.y + 26.0, 16.0, cl.id, hex(cl.c1), &cl.short);
        ui.text_semi(&ui.fit(&cl.name, 17.0, b.w - 200.0), b.x + 52.0, b.y + 22.0, 17.0, Theme::text());
        ui.text(&crate::tf!("ui.seiten_farm.staerke", c.world.leagues[cl.league].name, format!("{:.0}", cl.level)), b.x + 52.0, b.y + 42.0, 13.0, Theme::text2());
        ui.text_right(&crate::tf!("zentrale.bis", date::long(a.frist / date::SEASON_DAYS, a.frist % date::SEASON_DAYS)), b.x + b.w - 12.0, b.y + 22.0, 13.0, Theme::gedaempft());
        ui.text(&crate::tf!("ui.seiten_farm.fordert_im_jahr_mindestens_leihspieler", money(a.betrag), a.min_leihen, if a.bleiben { crate::inhalt::t("ui.wort.ja") } else { crate::inhalt::t("ui.wort.nein") }), b.x + 14.0, b.y + 66.0, 14.0, Theme::text());
        ui.text(&ui.fit(&format!("„{}“", a.grund), 14.0, b.w - 28.0), b.x + 14.0, b.y + 86.0, 14.0, Theme::text2());
        if !a.antwort.is_empty() { ui.text(&ui.fit(&a.antwort, 13.0, b.w - 28.0), b.x + 14.0, b.y + 104.0, 13.0, Theme::akzent()); }
        let bw = (b.w - 40.0) / 3.0;
        if ui.button(Rect::new(b.x + 10.0, b.y + b.h - 30.0, bw, 24.0), crate::inhalt::t("lb.aktion.annehmen"), false) { antwort = Some((a.id, crate::id("annehmen"))); }
        if ui.button(Rect::new(b.x + 20.0 + bw, b.y + b.h - 30.0, bw, 24.0), crate::inhalt::t("ui.farm.nachverhandeln"), false) { antwort = Some((a.id, crate::id("nachverhandeln"))); }
        if ui.button(Rect::new(b.x + 30.0 + 2.0 * bw, b.y + b.h - 30.0, bw, 24.0), crate::inhalt::t("as.ablehnen"), false) { antwort = Some((a.id, crate::id("ablehnen"))); }
        y += b.h + 8.0;
    }
    if let Some((id, k)) = antwort { let t = c.koop_antwort(id, k, None); v.toast(&t, true); }
}
