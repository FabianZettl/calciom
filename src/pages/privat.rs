use super::*;

/// Trainer › Private life (Fabian 24.11.): stress with its reasons, the account and the last statement, the home and
/// lifestyle, the family with the next anniversary, the weekly calendar.
pub(super) fn privat_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    tabs(ui, v, &[t("ui.privat.reiter.alltag"), t("ui.privat.reiter.mehr")]);
    if v.tab == 1 { return mehr_seite(ui, v, c); }
    let w3 = (cbreite() - 24.0) / 3.0;
    let h = hoehe() - Y0 - 16.0;
    let spalte = |k: usize| x0() + 4.0 + k as f32 * (w3 + 12.0);
    // stress
    let r = Rect::new(spalte(0), Y0, w3, h * 0.55);
    let s = c.privat.stress;
    let farbe = if s >= 70.0 { Theme::problem() } else if s >= 45.0 { Theme::achtung() } else { Theme::gut() };
    ui.card_titled(r, t("ui.privat.stress"), farbe);
    ui.title(&format!("{s:.0}"), r.x + 18.0, r.y + 100.0, 48.0, farbe);
    ui.bar(r.x + 110.0, r.y + 76.0, r.w - 130.0, 14.0, s / 100.0, farbe);
    let satz = if s >= 70.0 { t("ui.privat.stress_hoch") } else if s >= 45.0 { t("ui.privat.stress_mittel") } else { t("ui.privat.stress_niedrig") };
    ui.text(&ui.fit(satz, 14.0, r.w - 130.0), r.x + 110.0, r.y + 112.0, 14.0, Theme::text2());
    // the reasons of the last two weeks, summed per kind
    let mut summe: Vec<(String, f32)> = vec![];
    for g in &c.privat.gruende { match summe.iter_mut().find(|x| x.0 == g.1) { Some(x) => x.1 += g.2, None => summe.push((g.1.clone(), g.2)) } }
    summe.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
    let mut y = r.y + 150.0;
    for (k, w) in summe.iter().take(((r.h - 160.0) / 24.0) as usize) {
        let text = t(&format!("ui.privat.grund.{}", k.split('_').next().unwrap_or(k)));
        ui.text(text, r.x + 18.0, y, 14.0, Theme::text());
        ui.text_right(&format!("{w:+.0}").replace('-', "−"), r.x + r.w - 18.0, y, 14.0, if *w > 0.0 { Theme::problem() } else { Theme::gut() });
        y += 24.0;
    }
    // the account
    let r = Rect::new(spalte(0), Y0 + h * 0.55 + 12.0, w3, h * 0.45 - 12.0);
    ui.card_titled(r, t("ui.privat.konto"), Theme::gold());
    ui.title(&money(c.mgr_money), r.x + 18.0, r.y + 96.0, 34.0, Theme::text());
    if let Some(a) = c.privat.auszuege.last() {
        let zeilen = [(t("ui.privat.gehalt"), a.gehalt, Theme::gut()), (t("ui.privat.steuer"), -a.steuer, Theme::problem()), (t("ui.privat.wohnen"), -a.wohnen, Theme::problem()), (t("ui.privat.lebensstil"), -a.stil, Theme::problem()), (t("ui.privat.bleibt"), a.netto, Theme::text())];
        let mut y = r.y + 130.0;
        for (n, w, f) in zeilen { ui.text(n, r.x + 18.0, y, 14.0, Theme::text2()); ui.text_right(&money(w), r.x + r.w - 18.0, y, 14.0, f); y += 22.0; }
    } else { ui.text(t("ui.privat.noch_kein_auszug"), r.x + 18.0, r.y + 130.0, 14.0, Theme::text2()); }
    // home and lifestyle
    let r = Rect::new(spalte(1), Y0, w3, h * 0.55);
    ui.card_titled(r, t("ui.privat.wohnen"), Theme::akzent());
    let ort = c.world.clubs[c.privat.wohnort.min(c.world.clubs.len() - 1)].name.clone();
    let art = t(&format!("ui.privat.wohnung.{}", c.privat.wohnung.clamp(1, 3)));
    ui.text_semi(&ui.fit(&crate::tf!("ui.privat.wohnt", art, ort), 16.0, r.w - 36.0), r.x + 18.0, r.y + 66.0, 16.0, Theme::text());
    let kosten = if c.privat.gekauft { t("ui.privat.eigentum").to_string() } else { crate::tf!("ui.privat.miete_monat", money(c.miete(c.privat.wohnung, c.privat.wohnort))) };
    ui.text(&kosten, r.x + 18.0, r.y + 90.0, 14.0, Theme::text2());
    if c.privat.pendeln { ui.text(t("ui.privat.pendelt_hinweis"), r.x + 18.0, r.y + 112.0, 14.0, Theme::achtung()); }
    let bw = (r.w - 48.0) / 3.0;
    for k in 1..=3u8 {
        let b = Rect::new(r.x + 18.0 + (k - 1) as f32 * (bw + 6.0), r.y + 128.0, bw, 34.0);
        if ui.button(b, t(&format!("ui.privat.wohnung.{k}")), c.privat.wohnung == k && !c.privat.gekauft) && c.privat.wohnung != k {
            match c.wohnung_waehlen(k, false) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
        }
    }
    if !c.privat.gekauft && ui.button(Rect::new(r.x + 18.0, r.y + 170.0, r.w - 36.0, 34.0), &crate::tf!("ui.privat.kaufen", money(c.wohnung_kaufpreis(c.privat.wohnort))), false) {
        match c.wohnung_waehlen(c.privat.wohnung, true) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
    }
    ui.text_semi(t("ui.privat.lebensstil"), r.x + 18.0, r.y + 236.0, 15.0, Theme::text());
    let stufen: Vec<String> = (1..=5).map(|k| k.to_string()).collect();
    let refs: Vec<&str> = stufen.iter().map(|x| x.as_str()).collect();
    if let Some(k) = ui.segment(Rect::new(r.x + 18.0, r.y + 248.0, r.w - 36.0, 34.0), &refs, c.privat.lebensstil.saturating_sub(1) as usize) { c.privat.lebensstil = k as u8 + 1; }
    ui.text(&ui.fit(t(&format!("ui.privat.stil.{}", c.privat.lebensstil.clamp(1, 5))), 13.0, r.w - 36.0), r.x + 18.0, r.y + 302.0, 13.0, Theme::text2());
    // family
    let r = Rect::new(spalte(1), Y0 + h * 0.55 + 12.0, w3, h * 0.45 - 12.0);
    ui.card_titled(r, t("ui.privat.familie"), hex(0xdb2777));
    if c.privat.partner.is_empty() { ui.text(t("ui.privat.single"), r.x + 18.0, r.y + 70.0, 15.0, Theme::text2()); } else {
        ui.avatar(r.x + 40.0, r.y + 84.0, 22.0, &c.privat.partner.clone());
        ui.text_semi(&c.privat.partner, r.x + 72.0, r.y + 80.0, 16.0, Theme::text());
        ui.bar(r.x + 72.0, r.y + 90.0, r.w - 100.0, 8.0, c.privat.beziehung / 100.0, if c.privat.beziehung >= 60.0 { Theme::gut() } else if c.privat.beziehung >= 30.0 { Theme::achtung() } else { Theme::problem() });
        let bis = (c.privat.jahrestag + crate::date::SEASON_DAYS - c.world.day) % crate::date::SEASON_DAYS;
        ui.text(&crate::tf!("ui.privat.demnaechst_jahrestag", bis), r.x + 18.0, r.y + 130.0, 14.0, Theme::text2());
        let mut y = r.y + 158.0;
        for (n, seit) in &c.privat.kinder { ui.text(&crate::tf!("ui.privat.kind", n, (c.abs().saturating_sub(*seit)) / 365), r.x + 18.0, y, 14.0, Theme::text()); y += 22.0; }
    }
    // calendar
    let r = Rect::new(spalte(2), Y0, w3, h);
    ui.card_titled(r, t("ui.privat.kalender"), Theme::gut());
    ui.text(&ui.fit(t("ui.privat.kalender_hinweis"), 13.0, r.w - 36.0), r.x + 18.0, r.y + 62.0, 13.0, Theme::text2());
    let mut y = r.y + 92.0;
    let mut weg = None;
    for (i, tm) in c.privat.termine.iter().enumerate() {
        ui.text_semi(&format!("{} · {}", t(&format!("ui.privat.termin.{}", tm.art)), crate::date::wd_long(tm.wochentag as u32 + 2)), r.x + 18.0, y, 15.0, Theme::text());
        ui.text(&crate::tf!("ui.privat.bis", crate::date::short(c.world.season, c.world.day + tm.bis.saturating_sub(c.abs()))), r.x + 18.0, y + 20.0, 12.0, Theme::gedaempft());
        if ui.button(Rect::new(r.x + r.w - 128.0, y - 16.0, 110.0, 30.0), t("ui.privat.absagen"), false) { weg = Some(i); }
        y += 50.0;
    }
    if let Some(i) = weg { c.termin_absagen(i); v.toast(t("ui.privat.abgesagt"), true); }
    // new weekly appointments: three kinds, the weekday picked, three months long
    y = r.y + r.h - 130.0;
    ui.text_semi(t("ui.privat.neu"), r.x + 18.0, y, 15.0, Theme::text());
    let tage: Vec<&str> = (0..7).map(|k| crate::date::wd((k + 2) as u32)).collect();
    if let Some(k) = ui.segment(Rect::new(r.x + 18.0, y + 12.0, r.w - 36.0, 30.0), &tage, v.leih_wahl.0.min(6)) { v.leih_wahl.0 = k; }
    let bw = (r.w - 48.0) / 3.0;
    for (k, art) in crate::privat::ARTEN.iter().enumerate() {
        if ui.button(Rect::new(r.x + 18.0 + k as f32 * (bw + 6.0), y + 52.0, bw, 34.0), t(&format!("ui.privat.termin.{art}")), false) {
            c.termin_anlegen(art, v.leih_wahl.0.min(6) as u8, 13);
            v.toast(t("ui.privat.angelegt"), true);
        }
    }
}

/// Block 2: car and motorbike (they make home visits possible), flights, staff, personal sponsors, languages.
fn mehr_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let w3 = (cbreite() - 24.0) / 3.0;
    let h = hoehe() - Y0 - 16.0;
    let spalte = |k: usize| x0() + 4.0 + k as f32 * (w3 + 12.0);
    let kosten = |k: &str, i: usize| crate::regeln::wert("privat", &format!("block2.{k}")).as_array().and_then(|a| a.get(i)).and_then(|x| x.as_f64()).unwrap_or(0.0);
    // car
    let r = Rect::new(spalte(0), Y0, w3, h * 0.5 - 6.0);
    ui.card_titled(r, t("ui.privat.auto"), Theme::akzent());
    let autos: Vec<&str> = (0..4).map(|k| t(&format!("ui.privat.auto.{k}"))).collect();
    if let Some(k) = ui.segment(Rect::new(r.x + 16.0, r.y + 60.0, r.w - 32.0, 34.0), &autos, c.privat.auto as usize) { c.privat.auto = k as u8; }
    ui.text(&crate::tf!("ui.privat.im_monat", money(kosten("auto_kosten", c.privat.auto as usize))), r.x + 16.0, r.y + 116.0, 14.0, Theme::text2());
    if ui.button(Rect::new(r.x + 16.0, r.y + 130.0, r.w - 32.0, 34.0), t("ui.privat.motorrad"), c.privat.motorrad) { c.privat.motorrad = !c.privat.motorrad; }
    ui.paragraph(t("ui.privat.auto_hinweis"), r.x + 16.0, r.y + 190.0, 13.0, r.w - 32.0, Theme::text2());
    // flights
    let r = Rect::new(spalte(0), Y0 + h * 0.5 + 6.0, w3, h * 0.5 - 6.0);
    ui.card_titled(r, t("ui.privat.reisen"), Theme::gold());
    let fl: Vec<&str> = (0..3).map(|k| t(&format!("ui.privat.flug.{k}"))).collect();
    if let Some(k) = ui.segment(Rect::new(r.x + 16.0, r.y + 60.0, r.w - 32.0, 34.0), &fl, c.privat.flug as usize) { c.privat.flug = k as u8; }
    ui.paragraph(&crate::tf!("ui.privat.flug_hinweis", money(kosten("flug_kosten", c.privat.flug as usize)), format!("{:.0}", kosten("flug_stress", c.privat.flug as usize))), r.x + 16.0, r.y + 120.0, 14.0, r.w - 32.0, Theme::text2());
    // staff
    let r = Rect::new(spalte(1), Y0, w3, h * 0.5 - 6.0);
    ui.card_titled(r, t("ui.privat.personal"), Theme::gut());
    for k in 0..3 {
        let b = Rect::new(r.x + 16.0, r.y + 60.0 + k as f32 * 44.0, r.w - 32.0, 36.0);
        let titel = format!("{} · {}", t(&format!("ui.privat.personal.{k}")), crate::tf!("ui.privat.im_monat", money(kosten("personal_kosten", k))));
        if ui.button(b, &titel, c.privat.personal[k]) { c.privat.personal[k] = !c.privat.personal[k]; }
    }
    ui.paragraph(t("ui.privat.personal_hinweis"), r.x + 16.0, r.y + 210.0, 13.0, r.w - 32.0, Theme::text2());
    // sponsors
    let r = Rect::new(spalte(1), Y0 + h * 0.5 + 6.0, w3, h * 0.5 - 6.0);
    ui.card_titled(r, t("ui.privat.sponsoren"), hex(0xa78bfa));
    if c.privat.sponsoren.is_empty() { ui.paragraph(&crate::tf!("ui.privat.sponsoren_leer", format!("{:.0}", c.mgr_rep)), r.x + 16.0, r.y + 64.0, 14.0, r.w - 32.0, Theme::text2()); }
    for (k, (n, b, bis)) in c.privat.sponsoren.iter().enumerate() {
        let y = r.y + 70.0 + k as f32 * 30.0;
        ui.text(n, r.x + 16.0, y, 15.0, Theme::text());
        ui.text_right(&format!("{} · {}", crate::tf!("ui.privat.im_monat", money(*b)), crate::tf!("ui.privat.bis", crate::date::short(c.world.season, c.world.day + bis.saturating_sub(c.abs())))), r.x + r.w - 16.0, y, 13.0, Theme::text2());
    }
    // languages
    let r = Rect::new(spalte(2), Y0, w3, h);
    ui.card_titled(r, t("ui.privat.sprachen"), hex(0x0d9488));
    let mut y = r.y + 70.0;
    for code in &c.privat.sprachen { ui.text_semi(t(&format!("ui.sprache.{code}")), r.x + 16.0, y, 16.0, Theme::text()); y += 28.0; }
    let verein = c.club_sprache();
    y += 10.0;
    let satz = if c.privat.sprachen.contains(&verein) { crate::tf!("ui.privat.sprache_kann_verein", t(&format!("ui.sprache.{verein}"))) } else { crate::tf!("ui.privat.sprache_verein", t(&format!("ui.sprache.{verein}"))) };
    ui.paragraph(&satz, r.x + 16.0, y, 14.0, r.w - 32.0, Theme::text2());
    if let Some((code, bis)) = c.privat.kurs.clone() {
        ui.text(&crate::tf!("ui.privat.kurs_laeuft", t(&format!("ui.sprache.{code}")), bis.saturating_sub(c.abs()) / 7), r.x + 16.0, y + 60.0, 14.0, Theme::achtung());
    } else if !c.privat.sprachen.contains(&verein) && verein != "?" && ui.button(Rect::new(r.x + 16.0, y + 44.0, r.w - 32.0, 36.0), &crate::tf!("ui.privat.kurs_knopf", t(&format!("ui.sprache.{verein}"))), false) {
        match c.sprachkurs() { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
    }
}
