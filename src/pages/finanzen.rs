use super::*;

// ================================================================ Finanzen
/// Finanzen › Budget (Fabian 23.11.): the three pots and moving money between them.
fn budget_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    c.wirtschaft_an();
    let w3 = (cbreite() - 24.0) / 3.0;
    let werte = [(c.transfer_budget, c.topf_frei(0), Theme::gold()), (c.gehalt_topf(), c.topf_frei(1), Theme::gut()), (c.infra(), c.topf_frei(2), Theme::akzent())];
    for (k, (summe, frei, farbe)) in werte.iter().enumerate() {
        let r = Rect::new(x0() + 4.0 + k as f32 * (w3 + 12.0), Y0, w3, 150.0);
        ui.card_titled(r, t(&format!("ui.topf.{k}")), *farbe);
        ui.title(&money(*summe), r.x + 14.0, r.y + 86.0, 30.0, Theme::text());
        ui.text(&crate::tf!(&format!("ui.topf.frei.{k}"), money(*frei)), r.x + 14.0, r.y + 116.0, 15.0, Theme::text2());
        if k == 1 { ui.text(&crate::tf!("ui.topf.gehaelter_laufen", money(c.world.wage_bill(c.own))), r.x + 14.0, r.y + 138.0, 13.0, Theme::gedaempft()); }
    }
    // moving: from, to, amount as a share of what the source can give
    let r = Rect::new(x0() + 4.0, Y0 + 162.0, cbreite(), 230.0);
    ui.card_titled(r, t("ui.topf.schieben"), Theme::akzent());
    let namen = [t("ui.topf.0"), t("ui.topf.1"), t("ui.topf.2")];
    let (von, nach, anteil) = v.topf_wahl;
    ui.text(t("ui.topf.von"), r.x + 14.0, r.y + 66.0, 15.0, Theme::text2());
    if let Some(i) = ui.segment(Rect::new(r.x + 70.0, r.y + 46.0, 420.0, 32.0), &namen, von) { v.topf_wahl.0 = i; }
    ui.text(t("ui.topf.nach"), r.x + 520.0, r.y + 66.0, 15.0, Theme::text2());
    if let Some(i) = ui.segment(Rect::new(r.x + 576.0, r.y + 46.0, 420.0, 32.0), &namen, nach) { v.topf_wahl.1 = i; }
    let stufen = [5u32, 10, 25, 50, 100];
    let st: Vec<String> = stufen.iter().map(|s| format!("{s} %")).collect();
    let sr: Vec<&str> = st.iter().map(|s| s.as_str()).collect();
    if let Some(i) = ui.segment(Rect::new(r.x + 70.0, r.y + 96.0, 420.0, 32.0), &sr, anteil) { v.topf_wahl.2 = i; }
    let betrag = crate::world::round_money(c.topf_frei(von as u8) * stufen[anteil.min(4)] as f64 / 100.0);
    ui.text(&crate::tf!("ui.topf.betrag", money(betrag)), r.x + 520.0, r.y + 118.0, 16.0, Theme::text());
    if ui.button_gold(Rect::new(r.x + 14.0, r.y + 146.0, 260.0, 42.0), t("ui.topf.ausfuehren")) {
        match c.budget_schieben(von as u8, nach as u8, betrag) { Ok(s) => { c.grund("Präsidium", t("ui.topf.reiter"), 0.0, &s); v.toast(&s, true); } Err(e) => v.toast(&e, false) } // roh
    }
    ui.paragraph(t("ui.topf.hinweis"), r.x + 290.0, r.y + 150.0, 13.0, r.w - 310.0, Theme::text2());
}
/// Bank loans (kredit.rs): frame and rate, a new loan by amount and term, the running loans with early repayment.
fn kredite_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let w = cbreite() * 0.5 - 6.0;
    let r = Rect::new(x0() + 4.0, Y0, w, hoehe() - Y0 - 16.0);
    ui.card_titled(r, t("kr.neu"), Theme::gold());
    let rahmen = c.kredit_rahmen();
    ui.text(&crate::tf!("kr.rahmen", money(rahmen)), r.x + 14.0, r.y + 62.0, 18.0, Theme::text());
    ui.paragraph(t("kr.hinweis"), r.x + 14.0, r.y + 88.0, 14.0, r.w - 28.0, Theme::text2());
    let anteil = v.kr.0.clamp(10, 100);
    ui.text(t("kr.betrag"), r.x + 14.0, r.y + 170.0, 15.0, Theme::text2());
    if let Some(x) = ui.slider(Rect::new(r.x + 150.0, r.y + 160.0, r.w - 300.0, 14.0), anteil, 10, 100) { v.kr.0 = x; }
    let betrag = round_money(rahmen * anteil as f64 / 100.0);
    ui.text_right(&money(betrag), r.x + r.w - 14.0, r.y + 170.0, 16.0, Theme::text());
    ui.text(t("kr.laufzeit"), r.x + 14.0, r.y + 214.0, 15.0, Theme::text2());
    let jl: Vec<String> = (1..=5).map(|j| if j == 1 { t("kr.ein_jahr").to_string() } else { crate::tf!("ui.post.jahre", j) }).collect();
    let jr: Vec<&str> = jl.iter().map(|s| s.as_str()).collect();
    if let Some(s) = ui.segment(Rect::new(r.x + 150.0, r.y + 196.0, r.w - 164.0, 28.0), &jr, (v.kr.1 as usize).clamp(1, 5) - 1) { v.kr.1 = s as u32 + 1; }
    let jahre = v.kr.1.clamp(1, 5);
    let zins = c.kredit_zins(jahre);
    let rate = Career::kredit_rate(betrag, zins, jahre * 12);
    ui.text(&crate::tf!("kr.konditionen", crate::kredit::prozent(zins), money(rate), money(rate * (jahre * 12) as f64 - betrag)), r.x + 14.0, r.y + 260.0, 16.0, Theme::text());
    if ui.button(Rect::new(r.x + 14.0, r.y + 280.0, 240.0, 34.0), t("kr.aufnehmen"), true) && betrag > 0.0 {
        match c.kredit_aufnehmen(betrag, jahre) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
    }
    let r2 = Rect::new(x0() + w + 16.0, Y0, w, hoehe() - Y0 - 16.0);
    ui.card_titled(r2, t("kr.laufend"), Theme::akzent());
    if c.kredite.is_empty() { ui.text(t("kr.keine"), r2.x + 14.0, r2.y + 62.0, 16.0, Theme::text2()); }
    let mut weg = None;
    for (i, k) in c.kredite.iter().enumerate() {
        let yy = r2.y + 64.0 + i as f32 * 70.0;
        ui.text_semi(&crate::tf!("kr.zeile", money(k.betrag), crate::kredit::prozent(k.zins)), r2.x + 14.0, yy, 17.0, Theme::text());
        ui.text(&crate::tf!("kr.rest", money(k.rest), money(k.rate)), r2.x + 14.0, yy + 24.0, 15.0, Theme::text2());
        if ui.button(Rect::new(r2.x + r2.w - 170.0, yy - 18.0, 156.0, 30.0), t("kr.abloesen"), false) { weg = Some(i); }
    }
    if let Some(i) = weg { match c.kredit_abloesen(i) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
}

pub(super) fn finanzen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // sponsors, stadium and fans are main tabs of their own now (the wrapper pages choose the view)
    if v.page == Page::Finanzen { tabs(ui, v, &[crate::inhalt::t("as.reiter.ueberblick"), crate::inhalt::t("ui.pages.tickets_und_merchandising"), crate::inhalt::t("kr.reiter"), crate::inhalt::t("ui.topf.reiter")]); }
    if v.page == Page::Finanzen && v.tab == 2 { kredite_seite(ui, v, c); return; }
    if v.page == Page::Finanzen && v.tab == 3 { budget_seite(ui, v, c); return; }
    c.wirtschaft_an();
    let w4 = (cbreite() - 36.0) / 4.0;
    let y = Y0;
    let rev = c.umsatz_prognose();
    let wages = c.world.wage_bill(c.own);
    // the four money figures head the overview; the stadium tabs need the room for their settings
    let zeigen = !matches!(v.tab, 1 | 3);
    if zeigen {
        kpi(ui, Rect::new(x0() + 4.0, y, w4, 96.0), crate::inhalt::t("ui.wort.gross_kontostand"), &money(c.club().balance), crate::inhalt::t("ui.wort.aktuell"), if c.club().balance >= 0.0 { Theme::gut() } else { Theme::problem() });
        kpi(ui, Rect::new(x0() + 16.0 + w4, y, w4, 96.0), crate::inhalt::t("ui.wort.gross_transferbudget"), &money(c.transfer_budget), crate::inhalt::t("ui.pages.zum_ausgeben"), Theme::gold());
        kpi(ui, Rect::new(x0() + 28.0 + 2.0 * w4, y, w4, 96.0), crate::inhalt::t("ui.wort.gross_gehaltssumme"), &money(wages), &crate::tf!("ui.pages.des_umsatzes", format!("{:.0}", wages / rev * 100.0)), Theme::akzent());
        kpi(ui, Rect::new(x0() + 40.0 + 3.0 * w4, y, w4, 96.0), crate::inhalt::t("ui.wort.gross_umsatz"), &money(rev), crate::inhalt::t("welt.pro_jahr"), Theme::akzent());
    }
    let y = if zeigen { y + 110.0 } else { y };
    if v.tab == 0 {
        let r = Rect::new(x0() + 4.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r, crate::inhalt::t("ui.pages.buchungen_dieser_saison"), Theme::akzent());
        let mut yy = r.y + 56.0;
        if c.book.is_empty() { ui.text(crate::inhalt::t("ui.pages.noch_keine_buchungen_monatsabschluss_am"), r.x + 14.0, yy, 16.0, Theme::text2()); }
        for (n, val) in c.book.iter() {
            ui.text(n, r.x + 14.0, yy, 18.0, Theme::text());
            ui.text_right(&money(*val), r.x + r.w - 14.0, yy, 18.0, if *val >= 0.0 { Theme::gut() } else { Theme::problem() });
            yy += 30.0;
        }
        let sum: f64 = c.book.iter().map(|b| b.1).sum();
        if !c.book.is_empty() {
            draw_rectangle(r.x + 14.0, yy - 14.0, r.w - 28.0, 1.0, Theme::kontur());
            ui.text_semi(crate::inhalt::t("ui.pages.saldo"), r.x + 14.0, yy + 10.0, 18.0, Theme::text());
            ui.text_right(&money(sum), r.x + r.w - 14.0, yy + 10.0, 18.0, if sum >= 0.0 { Theme::gut() } else { Theme::problem() });
        }
        // where the money comes from (Fabian 23.11.): a year's income by source, then the balance over time
        let mix = c.einnahmen_mix();
        let summe: f64 = mix.iter().map(|x| x.1).sum::<f64>().max(1.0);
        let rm = Rect::new(x0() + cbreite() * 0.5 + 10.0, y, cbreite() * 0.5 - 6.0, 70.0 + mix.len() as f32 * 30.0);
        ui.card_titled(rm, crate::inhalt::t("vb.herkunft"), Theme::gold());
        for (k, (n, w)) in mix.iter().enumerate() {
            let yy = rm.y + 60.0 + k as f32 * 30.0;
            ui.text(n, rm.x + 14.0, yy, 15.0, Theme::text());
            ui.bar(rm.x + 200.0, yy - 10.0, rm.w - 360.0, 10.0, (*w / summe) as f32, Theme::gold());
            ui.text_right(&format!("{} · {:.0} %", money(*w), w / summe * 100.0), rm.x + rm.w - 14.0, yy, 14.0, Theme::text2());
        }
        let r2 = Rect::new(x0() + cbreite() * 0.5 + 10.0, rm.y + rm.h + 10.0, cbreite() * 0.5 - 6.0, hoehe() - rm.y - rm.h - 26.0);
        ui.card_titled(r2, crate::inhalt::t("ui.pages.kontostand_im_verlauf"), Theme::akzent());
        let hist = &c.balance_hist;
        let maxv = hist.iter().fold(1.0f64, |m, x| m.max(x.abs()));
        let bw = ((r2.w - 40.0) / hist.len().max(1) as f32).min(48.0);
        let base = r2.y + r2.h / 2.0 + 20.0;
        draw_line(r2.x + 20.0, base, r2.x + r2.w - 20.0, base, 1.0, Theme::kontur());
        for (i, b) in hist.iter().enumerate() {
            let h = (*b / maxv) as f32 * (r2.h / 2.0 - 60.0);
            let x = r2.x + 20.0 + i as f32 * bw;
            if h >= 0.0 { draw_rectangle(x + 2.0, base - h, bw - 4.0, h, Theme::akzent()); } else { draw_rectangle(x + 2.0, base, bw - 4.0, -h, Theme::problem()); }
        }
        ui.text(crate::inhalt::t("ui.pages.ein_balken_je_monat"), r2.x + 20.0, r2.y + r2.h - 16.0, 14.0, Theme::gedaempft());
    } else if v.tab == 1 {
        use crate::stadionbetrieb as sb;
        betrieb_kacheln(ui, c, y);
        let y = y + 104.0;
        let w2 = cbreite() * 0.5 - 6.0;
        let r = Rect::new(x0() + 4.0, y, w2, hoehe() - y - 16.0);
        ui.card_titled(r, crate::inhalt::t("vw.wi.preise.titel"), Theme::gold());
        let rh = ((r.h - 56.0) / 4.0).min(118.0);
        for (k, p) in [sb::P_SITZ, sb::P_STEH, sb::P_DAUER, sb::P_GAST].into_iter().enumerate() { stellschraube(ui, v, c, p, Rect::new(r.x + 14.0, r.y + 52.0 + k as f32 * rh, r.w - 28.0, rh)); }
        let r2 = Rect::new(x0() + cbreite() * 0.5 + 10.0, y, w2, hoehe() - y - 16.0);
        ui.card_titled(r2, crate::inhalt::t("vw.wi.betrieb.titel"), Theme::gold());
        for (k, p) in [sb::CATERING, sb::MERCH, sb::SICHERHEIT, sb::WERBUNG].into_iter().enumerate() { stellschraube(ui, v, c, p, Rect::new(r2.x + 14.0, r2.y + 52.0 + k as f32 * rh, r2.w - 28.0, rh)); }
    } else if v.tab == 2 {
        let r = Rect::new(x0() + 4.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
        // the pyramid (Fabian 24.11.): an investor above all, then main sponsor, kit, stadium name, two secondary
        // sponsors and four boards; open places say so
        ui.card_titled(r, crate::inhalt::t("ui.sponsor.pyramide"), Theme::gold());
        let mut yy = r.y + 58.0;
        let zeile = (r.h - 70.0) / (11.0 + c.wi.investor.is_some() as u8 as f32);
        if let Some(inv) = &c.wi.investor {
            rrect(r.x + 10.0, yy - 18.0, r.w - 20.0, zeile - 4.0, 6.0, alpha(Theme::gold(), 0.18));
            ui.text_semi(&format!("{}: {}", crate::inhalt::t("ui.sponsor.investor"), inv.name), r.x + 18.0, yy, 16.0, Theme::text());
            ui.text_right(&format!("{} · {} %", money(inv.summe), inv.anteil), r.x + r.w - 18.0, yy, 14.0, Theme::text2());
            yy += zeile;
        }
        for art in 0..crate::wirtschaft::SPONSOR_ARTEN.len() as u8 {
            let laufend: Vec<crate::wirtschaft::Sponsor> = c.wi.vertraege.iter().filter(|s| s.art == art).cloned().collect();
            for k in 0..crate::wirtschaft::plaetze(art) {
                let einzug = [0.0, 0.0, 0.0, 24.0, 48.0][art as usize];
                let art_t = crate::wirtschaft::sponsor_de_t()[art as usize];
                match laufend.get(k) {
                    Some(sp) => {
                        ui.text_semi(&ui.fit(&format!("{art_t}: {}", sp.name), 16.0, r.w * 0.55 - einzug), r.x + 18.0 + einzug, yy, 16.0, Theme::text());
                        ui.text_right(&crate::tf!("ui.pages.im_jahr_bis_bonus_bei", money(sp.jahr), date::season_label(sp.bis), money(sp.bonus)), r.x + r.w - 18.0, yy, 13.0, Theme::text2());
                    }
                    None => {
                        ui.text(&format!("{art_t}: {}", crate::inhalt::t("ui.sponsor.frei")), r.x + 18.0 + einzug, yy, 15.0, Theme::gedaempft());
                    }
                }
                yy += zeile;
            }
        }
        let r2 = Rect::new(x0() + cbreite() * 0.5 + 10.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r2, crate::inhalt::t("as.reiter.angebote"), Theme::akzent());
        let mut yy = r2.y + 60.0;
        if c.wi.angebote.is_empty() { ui.paragraph(crate::inhalt::t("ui.pages.neue_angebote_kommen_am_monatsende"), r2.x + 14.0, yy, 16.0, r2.w - 28.0, Theme::text2()); }
        let abs = c.abs();
        for (i, sp) in c.wi.angebote.clone().into_iter().enumerate() {
            if yy > r2.y + r2.h - 60.0 { break; }
            ui.text_semi(&format!("{}: {}", crate::wirtschaft::sponsor_de_t()[sp.art as usize], sp.name), r2.x + 14.0, yy, 18.0, Theme::text());
            ui.text(&crate::tf!("ui.pages.im_jahr_bis_noch_tage", money(sp.jahr), date::season_label(sp.bis), sp.frist.saturating_sub(abs)), r2.x + 14.0, yy + 24.0, 15.0, Theme::text2());
            if ui.button(Rect::new(r2.x + r2.w - 150.0, yy - 14.0, 136.0, 32.0), crate::inhalt::t("lb.aktion.annehmen"), false) { match c.sponsor_annehmen(i) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.vertrag_unterschrieben"), true), Err(e) => v.toast(&e, false) } break; }
            // one try to get more: the chance as a word; a no withdraws the offer (vorgaenge.py)
            if !sp.versucht {
                let mut weg = false;
                for (k, a) in crate::wirtschaftsvorgaenge::AUFSCHLAG.into_iter().enumerate() {
                    let ch = c.sponsor_chance(i, a);
                    let br = Rect::new(r2.x + r2.w - 14.0 - (2 - k) as f32 * 236.0, yy + 36.0, 226.0, 28.0);
                    if ui.button(br, &crate::tf!("sb.mehr", a, crate::wirtschaftsvorgaenge::aussicht(ch)), false) {
                        match c.sponsor_verhandeln(i, a) { Ok((ja, t)) => { v.toast(&t, ja); weg = !ja; } Err(e) => v.toast(&e, false) }
                    }
                }
                if weg { break; }
            } else { ui.text_right(crate::inhalt::t("sb.nachverhandelt"), r2.x + r2.w - 14.0, yy + 56.0, 14.0, Theme::gedaempft()); }
            yy += 90.0;
        }
    } else if v.tab == 4 {
        let r = Rect::new(x0() + 4.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r, crate::inhalt::t("gs.thema.fans_stimmung"), Theme::akzent());
        let teile = [(crate::inhalt::t("ui.pages.grundstimmung"), c.fans), (crate::inhalt::t("bereich.transfers"), c.me.fan_transfer), (crate::inhalt::t("ui.pages.preise"), c.me.fan_wirtschaft), (crate::inhalt::t("ui.medien.fanaktionen"), c.me.fan_aktion)];
        let mut yy = r.y + 64.0;
        ui.title(&format!("{:.0}", c.fans_eff()), r.x + 14.0, yy + 10.0, 40.0, if c.fans_eff() >= 55.0 { Theme::gut() } else if c.fans_eff() >= 40.0 { Theme::achtung() } else { Theme::problem() });
        ui.text(c.protest_wort(c.protest_stufe()), r.x + 110.0, yy + 4.0, 18.0, Theme::text());
        yy += 52.0;
        for (n, w) in teile { ui.text(n, r.x + 14.0, yy, 17.0, Theme::text2()); ui.text_right(&format!("{:+.1}", w), r.x + r.w - 14.0, yy, 17.0, if w >= 0.0 { Theme::gut() } else { Theme::problem() }); yy += 30.0; }
        ui.paragraph(crate::inhalt::t("ui.pages.die_grundstimmung_folgt_serien_tabelle"), r.x + 14.0, yy + 10.0, 15.0, r.w - 28.0, Theme::gedaempft());
        let r2 = Rect::new(x0() + cbreite() * 0.5 + 10.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r2, crate::inhalt::t("ui.medien.fanaktionen"), Theme::gold());
        let abs = c.abs();
        for (i, (id, name, kosten, fans, medien, _)) in c.fanaktionen().into_iter().enumerate() {
            let yy = r2.y + 64.0 + i as f32 * 62.0;
            ui.text_semi(&name, r2.x + 14.0, yy, 18.0, Theme::text());
            ui.text(&crate::tf!("ui.pages.fans_medien", money(kosten), format!("{:+.0}", fans), format!("{:+.0}", medien)), r2.x + 14.0, yy + 22.0, 15.0, Theme::text2());
            if let Some(x) = c.me.aktion_sperre.iter().find(|x| x.0 == id) { ui.text_right(&crate::tf!("ui.pages.wieder_in_tagen", x.1 - abs), r2.x + r2.w - 14.0, yy, 15.0, Theme::gedaempft()); }
            else if ui.button(Rect::new(r2.x + r2.w - 140.0, yy - 18.0, 126.0, 32.0), crate::inhalt::t("ui.pages.ausrichten"), false) { match c.fanaktion(i) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.die_fans_freuen_sich"), true), Err(e) => v.toast(&e, false) } }
        }
    } else {
        let r = Rect::new(x0() + 4.0, y, cbreite() * 0.58 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r, crate::inhalt::t("menu.stadion_und_anlagen.titel"), Theme::akzent());
        let abs = c.abs();
        for a in 0..5 {
            let yy = r.y + 70.0 + a as f32 * 74.0;
            ui.text_semi(crate::wirtschaft::anlagen_de_t()[a], r.x + 14.0, yy, 19.0, Theme::text());
            for k in 0..5 { draw_circle(r.x + 300.0 + k as f32 * 22.0, yy - 6.0, 7.0, if k < c.wi.stufen[a] as usize { Theme::gold() } else { Theme::ueberlagerung() }); }
            let wirkung = match a {
                0 => crate::tf!("ui.pages.plaetze", format!("{:.0}", c.kapazitaet())),
                1 => crate::tf!("ui.pages.merchandising_im_monat", money(c.merch_posten().0)),
                2 => crate::tf!("ui.pages.entwicklung", format!("{:.0}", (c.training_faktor() - 1.0) * 100.0)),
                3 => crate::tf!("ui.pages.jahrgaenge_staerke", format!("{:.0}", c.jugend_bonus())),
                _ => crate::tf!("ui.pages.verletzungsrisiko", format!("{:.0}", (1.0 - c.medizin_faktor()) * 100.0)),
            };
            ui.text(&wirkung, r.x + 14.0, yy + 24.0, 15.0, Theme::text2());
            if let Some(b) = c.wi.bau.iter().find(|b| b.0 == a) {
                ui.text_right(&crate::tf!("ui.pages.im_bau_stufe_fertig_in", b.1, b.2.saturating_sub(abs)), r.x + r.w - 14.0, yy + 26.0, 15.0, Theme::achtung());
            } else if c.wi.stufen[a] < 5 {
                ui.text_right(&crate::tf!("ui.pages.naechste_stufe", money(c.ausbau_kosten(a))), r.x + r.w - 14.0, yy + 26.0, 15.0, Theme::text2());
                if ui.button(Rect::new(r.x + r.w - 150.0, yy - 20.0, 136.0, 32.0), crate::inhalt::t("vw.aktion.ausbauen"), false) { match c.ausbauen(a) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.ausbau_begonnen"), true), Err(e) => v.toast(&e, false) } }
            }
        }
        ui.text(&crate::tf!("ui.pages.betrieb_und_unterhalt_im_monat", money(c.betrieb_monat()), money(c.tv_monat())), r.x + 14.0, r.y + r.h - 20.0, 15.0, Theme::gedaempft());
        // a new arena (neubau.rs): the button with its cost, or how long the building still takes
        let ny = r.y + r.h - 70.0;
        if c.neubau_laeuft() { ui.text(&crate::tf!("nb.laeuft", c.neubau.fertig.saturating_sub(c.abs())), r.x + 14.0, ny + 20.0, 16.0, Theme::achtung()); }
        else if c.neubau_faktor() > 1.0 { ui.text(crate::inhalt::t("nb.offen"), r.x + 14.0, ny + 20.0, 16.0, Theme::gut()); }
        else if ui.button(Rect::new(r.x + 14.0, ny, 360.0, 32.0), &crate::tf!("nb.knopf", money(c.neubau_kosten())), false) {
            match c.neubau_starten() { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
        }
        let r2 = Rect::new(x0() + cbreite() * 0.58 + 10.0, y, cbreite() * 0.42 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r2, crate::inhalt::t("vw.wi.stadion.bereiche"), Theme::gold());
        let s = c.spiel_neutral();
        let zeilen = [(crate::inhalt::t("vw.wi.z.atmosphaere"), format!("{} · {}", crate::stadionbetrieb::wort(crate::id("atm"), s.atmosphaere, &[30.0, 50.0, 70.0, 85.0]), crate::stadionbetrieb::wort(crate::id("heim"), s.heimvorteil, &[1.0, 2.0, 3.5]))),
            (crate::inhalt::t("vw.wi.z.sicherheit"), crate::stadionbetrieb::wort(crate::id("sich"), s.sicherheit(), &[35.0, 60.0, 80.0]).to_string()),
            (crate::inhalt::t("vw.wi.z.auslastung"), crate::tf!("vw.wi.z.zuschauer", format!("{:.0}", s.zuschauer), format!("{:.0}", s.kapazitaet)))];
        for (k, (n, w)) in zeilen.iter().enumerate() {
            let yy = r2.y + 64.0 + k as f32 * 28.0;
            ui.text(n, r2.x + 14.0, yy, 16.0, Theme::text2());
            ui.text_right(w, r2.x + r2.w - 14.0, yy, 16.0, Theme::text());
        }
        let y0 = r2.y + 64.0 + 3.0 * 28.0;
        let rh = ((r2.y + r2.h - y0 - 8.0) / 3.0).min(118.0);
        for (k, p) in crate::stadionbetrieb::BAULICH.into_iter().enumerate() { stellschraube(ui, v, c, p, Rect::new(r2.x + 14.0, y0 + k as f32 * rh, r2.w - 28.0, rh)); }
    }
}

/// Key numbers of the stadium operation (wirtschaft.py zentrale): filling, atmosphere, fans' mood, security, next home match.
fn betrieb_kacheln(ui: &mut Ui, c: &Career, y: f32) {
    use crate::stadionbetrieb as sb;
    let s = c.spiel_neutral();
    let w5 = (cbreite() - 48.0) / 5.0;
    let nb = c.wi.heimspiele.last().map(|h| crate::tf!("sb.letztes", format!("{}", h.1), format!("{:.0}", h.2 * 100.0))).unwrap_or_default();
    kpi(ui, Rect::new(x0() + 4.0, y, w5, 92.0), crate::inhalt::t("vw.wi.z.auslastung"), &format!("{:.0} %", s.auslastung * 100.0), &crate::tf!("vw.wi.z.zuschauer", format!("{:.0}", s.zuschauer), format!("{:.0}", s.kapazitaet)), Theme::akzent());
    kpi(ui, Rect::new(x0() + 16.0 + w5, y, w5, 92.0), crate::inhalt::t("vw.wi.z.atmosphaere"), sb::wort(crate::id("atm"), s.atmosphaere, &[30.0, 50.0, 70.0, 85.0]), sb::wort(crate::id("heim"), s.heimvorteil, &[1.0, 2.0, 3.5]), Theme::akzent());
    let tend = crate::inhalt::t(if s.stimmung_ziel > 0.5 { "vw.wi.w.tendenz.0" } else if s.stimmung_ziel < -0.5 { "vw.wi.w.tendenz.2" } else { "vw.wi.w.tendenz.1" });
    kpi(ui, Rect::new(x0() + 28.0 + 2.0 * w5, y, w5, 92.0), crate::inhalt::t("vw.wi.z.stimmung"), sb::wort(crate::id("stim"), c.fans_eff() as f64, &[25.0, 45.0, 65.0, 85.0]), tend, if s.stimmung_ziel < -0.5 { Theme::problem() } else { Theme::gut() });
    let risiko = if s.risiko > crate::regeln::f("vereinswelt", "wirtschaft.risiko_strafe_ab") as f64 { crate::inhalt::t("vw.wi.z.risiko_hoch") } else { crate::inhalt::t("vw.wi.z.risiko_ok") };
    kpi(ui, Rect::new(x0() + 40.0 + 3.0 * w5, y, w5, 92.0), crate::inhalt::t("vw.wi.z.sicherheit"), sb::wort(crate::id("sich"), s.sicherheit(), &[35.0, 60.0, 80.0]), risiko, Theme::akzent());
    let e = c.einst();
    let monat = c.monat_netto(&e, c.wi.stufen[0]);
    kpi(ui, Rect::new(x0() + 52.0 + 4.0 * w5, y, w5, 92.0), crate::inhalt::t("vw.wi.z.ergebnis"), &money(monat), &nb, if monat >= 0.0 { Theme::gut() } else { Theme::problem() });
}

/// One setting of the stadium operation: name, hint, the levels as a segment; hovering a level shows what it changes.
fn stellschraube(ui: &mut Ui, v: &mut View, c: &mut Career, p: usize, r: Rect) {
    use crate::stadionbetrieb as sb;
    let name = sb::POSTEN[p];
    let e = c.einst();
    let n = sb::stufenzahl(p);
    ui.text_semi(crate::inhalt::t(&format!("vw.wi.posten.{name}")), r.x, r.y + 14.0, 17.0, Theme::text()); // roh
    let s = c.spiel_neutral();
    let preis = match p { sb::P_SITZ => Some(s.preise[0]), sb::P_STEH => Some(s.preise[1]), sb::P_DAUER => Some(s.preise[2]), sb::P_GAST => Some(s.preise[3]), _ => None };
    if let Some(x) = preis { ui.text_right(&crate::tf!("vw.wi.preis_je_karte", money(x)), r.x + r.w, r.y + 14.0, 15.0, Theme::gold()); }
    let labels: Vec<&str> = (0..n).map(|k| crate::inhalt::t(&format!("vw.wi.{name}.{k}"))).collect(); // roh
    let seg = Rect::new(r.x, r.y + 26.0, r.w, 32.0);
    if let Some(k) = ui.segment(seg, &labels, e[p] as usize) {
        match c.einst_setzen(p, k as u8) { Ok(kosten) => v.toast(&if kosten > 0.0 { crate::tf!("sb.umgebaut", money(kosten)) } else { crate::inhalt::t("vw.ok.wirtschaft").to_string() }, true), Err(x) => v.toast(&x, false) }
    }
    // the hovered level: its effect on the month, the crowd and the mood; otherwise the hint
    let gap = 6.0;
    let cw = (seg.w - gap * (n - 1) as f32) / n as f32;
    let (mx, my) = mouse_position();
    let hov = (0..n).find(|&k| { let x0 = seg.x + k as f32 * (cw + gap); mx >= x0 && mx <= x0 + cw && my >= seg.y && my <= seg.y + seg.h });
    match hov.filter(|&k| k != e[p] as usize) {
        Some(k) => {
            let (d, a, b) = c.vergleich(p, k as u8);
            let mut t = crate::tf!("sb.wirkung", format!("{}{}", if d >= 0.0 { "+" } else { "" }, money(d)), format!("{:.0}", b.zuschauer - a.zuschauer), format!("{:+.1}", b.stimmung_ziel - a.stimmung_ziel), format!("{:+.0}", b.atmosphaere - a.atmosphaere));
            let einmal = c.umbau_kosten(p, k as u8);
            if einmal > 0.0 { t = format!("{t} · {}", crate::tf!("vw.wi.v.einmal", money(einmal))); }
            ui.text_passend(&t, r.x, r.y + 78.0, 14.0, r.w, 2, if d >= 0.0 { Theme::gut() } else { Theme::achtung() });
        }
        None => { ui.text_passend(crate::inhalt::t(&format!("vw.wi.hinweis.{name}")), r.x, r.y + 78.0, 14.0, r.w, 2, Theme::text2()); } // roh
    }
}


pub(super) fn stadion(ui: &mut Ui, v: &mut View, c: &mut Career) { v.tab = 3; finanzen(ui, v, c); }
pub(super) fn marken(ui: &mut Ui, v: &mut View, c: &mut Career) { v.tab = 2; finanzen(ui, v, c); }
pub(super) fn fans_seite(ui: &mut Ui, v: &mut View, c: &mut Career) { v.tab = 4; finanzen(ui, v, c); }
