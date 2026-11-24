use super::*;

pub(super) fn uebersicht(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let lw = cbreite() * 0.62;
    // the banner (Fabian 23.11.): our stadium, a greeting by the real time of day, table place and squad strength
    let ban = Rect::new(x0() + 4.0, Y0 - 40.0, cbreite(), 140.0);
    {
        let cl = c.club().clone();
        let farbe = { let a = ui.wappen_farbe(cl.id).map(hex).unwrap_or(hex(cl.c1)); if crate::ui::kontrast(a, WHITE) >= 3.0 { a } else { crate::ui::lesbar(a, BLACK, 1.0) } };
        rrect(ban.x, ban.y, ban.w, ban.h, 10.0, farbe);
        let bild = ui.stadion_bild(ban, cl.id, &cl.name);
        gradient_h(ban.x, ban.y, ban.w, ban.h, alpha(farbe, if bild { 0.92 } else { 1.0 }), alpha(farbe, if bild { 0.15 } else { 0.7 }));
        draw_circle(ban.x + 70.0, ban.y + 70.0, 50.0, alpha(WHITE, 0.92));
        ui.club_crest(ban.x + 70.0, ban.y + 70.0, 42.0, cl.id, hex(cl.c1), &cl.short);
        let gruss = match ortsstunde() { Some(h) if (5..12).contains(&h) => "ui.ueb.gruss.morgen", Some(h) if (12..18).contains(&h) => "ui.ueb.gruss.tag", Some(h) if (18..23).contains(&h) => "ui.ueb.gruss.abend", Some(_) => "ui.ueb.gruss.nacht", None => "ui.ueb.gruss.tag" }; // roh
        let weiss = hex(0xffffff);
        ui.title(&ui.fit(&crate::tf!(gruss, c.coach.clone()), 32.0, ban.w - 200.0), ban.x + 140.0, ban.y + 56.0, 32.0, weiss);
        let tab = c.world.table(c.league());
        let platz = tab.iter().position(|x| x.club == c.own).map(|i| i + 1).unwrap_or(0);
        let gespielt = tab.iter().any(|x| x.p > 0);
        let chips = [
            crate::tf!("ui.ueb.chip_verein", cl.name.clone(), c.world.leagues[c.league()].name.clone()),
            if gespielt { crate::tf!("ui.ueb.chip_platz", platz) } else { crate::inhalt::t("ui.ueb.chip_vor_saison").to_string() },
            crate::tf!("ui.ueb.chip_staerke", format!("{:.0}", crate::mensch::kaderstaerke(&c.world, c.own)), stufe_wort(crate::mensch::kaderstaerke(&c.world, c.own))),
            crate::tf!("ui.ueb.chip_datum", date::long(c.world.season, c.world.day)),
        ];
        let mut cx = ban.x + 140.0;
        for ch in chips {
            let w = ui.measure(&ch, 14.0) + 20.0;
            if cx + w > ban.x + ban.w - 12.0 { break; }
            rrect(cx, ban.y + 80.0, w, 26.0, 13.0, alpha(BLACK, 0.35));
            ui.text(&ch, cx + 10.0, ban.y + 98.0, 14.0, weiss);
            cx += w + 8.0;
        }
    }
    let oben = ban.h + 12.0;
    // next match
    let r = Rect::new(x0() + 4.0, Y0 - 40.0 + oben, lw, 260.0);
    ui.card_titled(r, crate::inhalt::t("zentrale.spiel_naechstes"), Theme::akzent());
    let table = c.world.table(c.league());
    let form_of = |club: usize| table.iter().find(|x| x.club == club).map(|x| x.form.clone()).unwrap_or_default();
    if let Some(f) = c.next_match() {
        let (h, a) = (f.h, f.a);
        for (k, club) in [h, a].iter().enumerate() {
            let cx = if k == 0 { r.x + 120.0 } else { r.x + r.w - 120.0 };
            let cl = &c.world.clubs[*club];
            ui.club_crest(cx, r.y + 94.0, 40.0, cl.id, hex(cl.c1), &cl.short);
            let nm = ui.fit(&cl.name, 22.0, 220.0);
            let w = measure_text(&nm, Some(&ui.head), 22, 1.0).width;
            ui.title(&nm, cx - w / 2.0, r.y + 160.0, 22.0, Theme::text());
            let fm = form_of(*club);
            form_squares(ui, cx - fm.len() as f32 * 13.0, r.y + 172.0, &fm);
        }
        let w = measure_text(crate::inhalt::t("ui.wort.gross_gegen"), Some(&ui.head), 34, 1.0).width;
        ui.title(crate::inhalt::t("ui.wort.gross_gegen"), r.x + r.w / 2.0 - w / 2.0, r.y + 102.0, 34.0, Theme::verein());
        let when = if f.day == c.world.day { crate::id("heute").to_string() } else { crate::tf!("zentrale.in_tagen.viele", f.day - c.world.day) };
        ui.text_center(&crate::tf!("ui.pages.anstoss_15_30", date::long(c.world.season, f.day), when), r.x + r.w / 2.0, r.y + 130.0, 16.0, Theme::text2());
        let label = if c.today_match().is_some() { crate::inhalt::t("pm.zum_spiel") } else { crate::inhalt::t("ui.pages.elf_vorbereiten") };
        if ui.button_gold(Rect::new(r.x + r.w / 2.0 - 130.0, r.y + 204.0, 260.0, 44.0), label) { v.go(if c.today_match().is_some() { Page::Spieltag } else { Page::Aufstellung }); }
    } else {
        ui.empty(Rect::new(r.x, r.y + 40.0, r.w, r.h - 40.0), crate::inhalt::t("ui.pages.die_saison_ist_gespielt_die"));
    }
    // short table
    let rt = Rect::new(x0() + lw + 16.0, Y0 - 40.0 + oben, cbreite() - lw - 12.0, 260.0);
    ui.card_titled(rt, &c.world.leagues[c.league()].name.clone(), Theme::akzent());
    let me = table.iter().position(|x| x.club == c.own).unwrap_or(0);
    let start = me.saturating_sub(2).min(table.len().saturating_sub(5));
    let played = table.iter().any(|x| x.p > 0);
    for (k, row) in table.iter().enumerate().skip(start).take(5) {
        let y = rt.y + 62.0 + (k - start) as f32 * 33.0;
        if row.club == c.own { rrect(rt.x + 8.0, y - 20.0, rt.w - 16.0, 30.0, 8.0, if Theme::dark() { Theme::verein_tief() } else { Theme::verein_weich() }); }
        ui.text_semi(&if played { format!("{}", k + 1) } else { "–".into() }, rt.x + 18.0, y, 16.0, Theme::text2());
        let cl = &c.world.clubs[row.club];
        ui.club_crest(rt.x + 52.0, y - 5.0, 11.0, cl.id, hex(cl.c1), &cl.short);
        let nm = ui.fit(&cl.name, 16.0, rt.w - 150.0);
        ui.text(&nm, rt.x + 70.0, y, 16.0, Theme::text());
        ui.text_semi(&format!("{}", row.pts), rt.x + rt.w - 24.0, y, 16.0, Theme::text());
    }
    if ui.button(Rect::new(rt.x + 16.0, rt.y + rt.h - 44.0, rt.w - 32.0, 32.0), crate::inhalt::t("liga.zur_tabelle"), false) { v.go(Page::Tabellen); }
    // tasks of the day
    let y2 = Y0 - 40.0 + oben + 272.0;
    let rk = Rect::new(x0() + 4.0, y2, lw, hoehe() - y2 - 14.0);
    ui.card_titled(rk, crate::inhalt::t("ui.pages.aufgaben_des_tages"), hex(0xfb923c));
    let mut tasks: Vec<(String, Page, Color)> = vec![];
    if c.fired { tasks.push((crate::inhalt::t("ui.pages.du_bist_ohne_verein_angebote").into(), Page::Posteingang, Theme::problem())); }
    if c.today_match().is_some() { tasks.push((crate::inhalt::t("ui.pages.heute_ist_spieltag").into(), Page::Spieltag, Theme::verein())); }
    if c.press.is_some() { tasks.push((crate::inhalt::t("vorbereitung.pk_offen").into(), Page::Presse, hex(0x38bdf8))); }
    let dec = c.open_decisions();
    if dec > 0 { tasks.push((crate::tf!("ui.pages.entscheidungen_im_posteingang", dec), Page::Posteingang, Theme::achtung())); }
    let negs = c.negs.iter().filter(|n| n.state == NegState::Counter && n.stage < 2).count();
    if negs > 0 { tasks.push((crate::tf!("ui.pages.verhandlungen_warten_auf_dein_angebot", negs), Page::Verhandlungen, hex(0xfb923c))); }
    let sad = c.squad().iter().filter(|&&p| c.world.players[p].morale < 40.0).count();
    if sad > 0 { tasks.push((crate::tf!("ui.pages.spieler_sind_unzufrieden", sad), Page::Kabine, hex(0xa78bfa))); }
    let inj = c.squad().iter().filter(|&&p| c.world.players[p].inj > 0).count();
    if inj > 0 { tasks.push((crate::tf!("ui.pages.spieler_fallen_verletzt_aus", inj), Page::Medizin, hex(0xf472b6))); }
    if tasks.is_empty() { ui.empty(Rect::new(rk.x, rk.y + 36.0, rk.w, rk.h - 36.0), crate::inhalt::t("hub.jetzt.tag")); }
    for (k, (t, pg, col)) in tasks.iter().take(5).enumerate() {
        let rr = Rect::new(rk.x + 12.0, rk.y + 48.0 + k as f32 * 48.0, rk.w - 24.0, 42.0);
        rrect(rr.x, rr.y, rr.w, rr.h, 12.0, if ui.hover(rr) { Theme::ueberlagerung() } else { Theme::erhoehung() });
        rrect(rr.x, rr.y, 6.0, rr.h, 3.0, *col);
        ui.text_semi(t, rr.x + 20.0, rr.y + 27.0, 18.0, Theme::text());
        ui.text_right("»", rr.x + rr.w - 14.0, rr.y + 28.0, 20.0, Theme::text2());
        if ui.hit(rr) { v.go(*pg); }
    }
    // the squad at a glance below the tasks: fitness, morale, injured and suspended, then the players in form
    let ky = rk.y + 60.0 + tasks.len().min(5).max(1) as f32 * 48.0;
    if ky + 150.0 < rk.y + rk.h {
        let sq = c.squad();
        let n = sq.len().max(1) as f32;
        let fit = sq.iter().map(|&p| c.world.players[p].fit).sum::<f32>() / n;
        let mor = sq.iter().map(|&p| c.world.players[p].morale).sum::<f32>() / n;
        let verl = sq.iter().filter(|&&p| c.world.players[p].inj > 0).count();
        let gesp = sq.iter().filter(|&&p| c.world.players[p].susp > 0).count();
        ui.text_semi(&crate::inhalt::t("ui.ueb.kader_blick").to_uppercase(), rk.x + 14.0, ky, 13.0, Theme::akzent());
        let w4 = (rk.w - 28.0 - 24.0) / 4.0;
        for (i, (titel, wert, farbe)) in [(crate::inhalt::t("ui.ueb.fitness"), format!("{fit:.0} %"), if fit > 75.0 { Theme::gut() } else { Theme::achtung() }), (crate::inhalt::t("ui.ueb.moral"), format!("{mor:.0}"), if mor > 55.0 { Theme::gut() } else { Theme::achtung() }), (crate::inhalt::t("ui.ueb.verletzt"), verl.to_string(), if verl == 0 { Theme::gut() } else { Theme::problem() }), (crate::inhalt::t("ui.ueb.gesperrt"), gesp.to_string(), if gesp == 0 { Theme::gut() } else { Theme::problem() })].into_iter().enumerate() {
            let b = Rect::new(rk.x + 14.0 + i as f32 * (w4 + 8.0), ky + 10.0, w4, 56.0);
            rrect(b.x, b.y, b.w, b.h, 8.0, Theme::erhoehung());
            ui.text(titel, b.x + 10.0, b.y + 18.0, 12.0, Theme::text2());
            ui.title(&wert, b.x + 10.0, b.y + 46.0, 24.0, farbe);
        }
        let mut form: Vec<(usize, f32)> = sq.iter().filter_map(|&p| form5(&c.world.players[p]).map(|f| (p, f))).collect();
        form.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let fy = ky + 92.0;
        if !form.is_empty() {
            ui.text_semi(&crate::inhalt::t("ui.ueb.in_form").to_uppercase(), rk.x + 14.0, fy, 13.0, Theme::akzent());
            for (k, (p, f)) in form.iter().take(((rk.y + rk.h - fy - 16.0) / 26.0).max(0.0) as usize).take(5).enumerate() {
                let pl = &c.world.players[*p];
                let y = fy + 24.0 + k as f32 * 26.0;
                ui.player_photo(rk.x + 26.0, y - 5.0, 10.0, pl.id, &pl.name);
                ui.pos_tag(rk.x + 44.0, y - 5.0, pos_de(&pl.pos));
                ui.text(&ui.fit(&pl.name, 14.0, rk.w - 164.0), rk.x + 88.0, y, 14.0, Theme::text());
                ui.text_right(&format!("{f:.1}"), rk.x + rk.w - 16.0, y, 14.0, Theme::note(*f));
                if ui.hit(Rect::new(rk.x + 12.0, y - 18.0, rk.w - 24.0, 24.0)) { v.modal = Modal::Player(*p); }
            }
        }
    }
    // the last results, newest first, then the co-trainer
    let rl = Rect::new(x0() + lw + 16.0, y2, cbreite() - lw - 12.0, 150.0);
    ui.card_titled(rl, crate::inhalt::t("ui.ueb.letzte_spiele"), Theme::akzent());
    if c.spiele.is_empty() { ui.text(crate::inhalt::t("ui.ueb.noch_keine_spiele"), rl.x + 14.0, rl.y + 60.0, 14.0, Theme::gedaempft()); }
    for (k, s) in c.spiele.iter().rev().take(4).enumerate() {
        let y = rl.y + 58.0 + k as f32 * 22.0;
        let farbe = if s.tore > s.gegentore { Theme::gut() } else if s.tore == s.gegentore { Theme::achtung() } else { Theme::problem() };
        rrect(rl.x + 12.0, y - 13.0, 40.0, 18.0, 4.0, farbe);
        ui.text_center(&format!("{}:{}", s.tore, s.gegentore), rl.x + 32.0, y + 1.0, 13.0, hex(0x10141c));
        let g = &c.world.clubs[s.gegner];
        ui.text(&ui.fit(&format!("{} {}", if s.heim { crate::inhalt::t("ui.ueb.gegen") } else { "@" }, g.name), 14.0, rl.w - 80.0), rl.x + 62.0, y + 1.0, 14.0, Theme::text()); // roh
    }
    let y3 = rl.y + rl.h + 12.0;
    let rc = Rect::new(x0() + lw + 16.0, y3, cbreite() - lw - 12.0, hoehe() - y3 - 14.0);
    let op = c.co_opinion();
    let co = c.co_name.clone();
    bubble(ui, rc, &co, crate::id("Co-Trainer"), &op);
}
