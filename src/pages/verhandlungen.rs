use super::*;

/// Instalments and add-on of a fee, two short rows with what they mean in money.
fn zahlweise_zeilen(ui: &mut Ui, w: &mut crate::klauseln::Wahl, x: f32, y: f32, fee: f64) {
    let t = crate::inhalt::t;
    ui.text(t("ui.zahlweise.raten"), x, y + 20.0, 14.0, Theme::text2());
    if let Some(k) = ui.segment(Rect::new(x + 156.0, y, 300.0, 28.0), &[t("ui.zahlweise.sofort"), t("ui.zahlweise.zwei"), t("ui.zahlweise.drei")], w.raten as usize) { w.raten = k as u8; }
    ui.text(t("ui.zahlweise.bonus"), x, y + 56.0, 14.0, Theme::text2());
    if let Some(k) = ui.segment(Rect::new(x + 156.0, y + 36.0, 300.0, 28.0), &[t("ui.klausel.keine"), "+10 %", "+20 %"], w.bonus as usize) { w.bonus = k as u8; }
    let teile = w.raten as f64 + 1.0;
    let b = [0.0, 0.10, 0.20][w.bonus.min(2) as usize];
    let satz = if w.raten == 0 && w.bonus == 0 { String::new() } else { crate::tf!("ui.zahlweise.satz", money(fee / teile), w.raten + 1, money(fee * b), crate::regeln::f("klauseln", "bonus.einsaetze")) };
    if !satz.is_empty() { ui.text_passend(&satz, x + 470.0, y + 16.0, 13.0, 220.0, 3, Theme::text2()); }
}

// ================================================================ Verhandlungen
pub(super) fn verhandlungen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("chat.laufend"), crate::inhalt::t("chat.abschluss.titel")]);
    let list: Vec<usize> = (0..c.negs.len()).filter(|&i| (c.negs[i].stage < 2) == (v.tab == 0)).collect();
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    if list.is_empty() { ui.title(if v.tab == 0 { crate::inhalt::t("ui.pages.keine_laufenden_verhandlungen") } else { crate::inhalt::t("transfer.nichts_abgeschlossen") }, area.x + 20.0, area.y + 46.0, 24.0, Theme::text2()); ui.text(crate::inhalt::t("ui.pages.im_transfermarkt_einen_spieler_waehlen"), area.x + 20.0, area.y + 76.0, 17.0, Theme::gedaempft()); }
    for (k, &i) in list.iter().enumerate().take(12) {
        let n = &c.negs[i];
        let pl = &c.world.players[n.pid];
        let r = Rect::new(area.x + 8.0, area.y + 10.0 + k as f32 * 66.0, area.w - 16.0, 60.0);
        rrect(r.x, r.y, r.w, r.h, 6.0, if ui.hover(r) { Theme::ueberlagerung() } else { Theme::erhoehung() });
        ui.pos_tag(r.x + 10.0, r.y + 22.0, pos_de(&pl.pos));
        ui.player_photo(r.x + 62.0, r.y + 22.0, 15.0, pl.id, &pl.name);
        ui.text_semi(&pl.name, r.x + 84.0, r.y + 27.0, 19.0, Theme::text());
        let stage = if n.renew { crate::inhalt::t("r.be.mock.system.art.vertrag") } else if n.stage == 0 { crate::inhalt::t("ui.pages.gespraech_mit_dem_verein") } else if n.stage == 1 { crate::inhalt::t("tf.phase.spieler") } else { crate::inhalt::t("chat.abschluss.titel") };
        ui.text(stage, r.x + 54.0, r.y + 48.0, 15.0, Theme::text2());
        let (st, col) = match n.state { NegState::Waiting => (crate::inhalt::t("ui.pages.wartet_auf_antwort"), Theme::achtung()), NegState::Counter => (crate::inhalt::t("ui.pages.du_bist_dran"), Theme::akzent()), NegState::Accepted => (crate::inhalt::t("gs.status.einig"), Theme::gut()), NegState::Rejected => (crate::id("gescheitert"), Theme::problem()) };
        ui.text_semi(st, r.x + 340.0, r.y + 27.0, 17.0, col);
        let t = ui.fit(&n.text, 15.0, r.w - 520.0);
        ui.text(&t, r.x + 340.0, r.y + 48.0, 15.0, Theme::text2());
        if ui.hit(r) { v.modal = Modal::Neg(i); }
    }
}

pub fn neg_modal(ui: &mut Ui, v: &mut View, c: &mut Career, i: usize) {
    if i >= c.negs.len() { v.modal = Modal::None; return; }
    // the talk as a chat on the left (Fabian 23.11.), the offer on the right
    c.neg_verlauf_pflegen(i);
    let ganz = Rect::new(breite() / 2.0 - 570.0, 60.0, 1140.0, 724.0);
    rrect(ganz.x, ganz.y, ganz.w, ganz.h, 10.0, Theme::ueberlagerung());
    neg_chat(ui, v, c, i, Rect::new(ganz.x + 12.0, ganz.y + 12.0, 456.0, ganz.h - 24.0));
    let r = Rect::new(ganz.x + 480.0, ganz.y, 660.0, ganz.h);
    let n = c.negs[i].clone();
    let pl = c.world.players[n.pid].clone();
    ui.title(&crate::tf!("vhd.titel.buy", pl.name), r.x + 24.0, r.y + 44.0, 28.0, Theme::text());
    let stage = if n.verkauf { crate::inhalt::t("ui.pages.verkauf_abloese_mit_dem_kaeufer") } else if n.renew { crate::inhalt::t("r.be.mock.system.art.vertrag") } else if n.stage == 0 { crate::inhalt::t("ui.pages.schritt_1_von_2_abloese") } else if n.stage == 1 { crate::inhalt::t("ui.pages.schritt_2_von_2_vertrag") } else { crate::inhalt::t("chat.abschluss.titel") };
    ui.text_semi(stage, r.x + 24.0, r.y + 74.0, 18.0, Theme::akzent());
    let art = n.ph.as_ref().map(|p| crate::verhandlung::charakter_wort(&p.charakter)).unwrap_or(crate::id("fair"));
    let geduld = n.ph.as_ref().map(|p| p.geduld).unwrap_or(100.0);
    let gw = if geduld >= 60.0 { crate::inhalt::t("ui.pages.viel_geduld") } else if geduld >= 25.0 { crate::inhalt::t("ui.pages.wird_ungeduldig") } else { crate::inhalt::t("ui.pages.kaum_noch_geduld") };
    ui.text(&crate::tf!("ui.pages.gegenseite_runde_von", art, gw, n.ph.as_ref().map(|p| p.runde).unwrap_or(0), crate::verhandlung::MAX_RUNDEN), r.x + 24.0, r.y + 98.0, 16.0, Theme::text2());
    let bubble = Rect::new(r.x + 24.0, r.y + 114.0, r.w - 48.0, 70.0);
    rrect(bubble.x, bubble.y, bubble.w, bubble.h, 10.0, Theme::erhoehung());
    ui.paragraph(&n.text, bubble.x + 14.0, bubble.y + 28.0, 17.0, bubble.w - 28.0, Theme::text());
    let editable = n.state == NegState::Counter && n.stage < 2;
    let mut y = r.y + 214.0;
    // the player's side: who sits at the table and, with people skills or a scout, what matters to him (berater.rs)
    if n.stage >= 1 && !n.verkauf {
        ui.text(&ui.fit(&c.berater_zeile(n.pid), 14.0, r.w - 48.0), r.x + 24.0, r.y + 204.0, 14.0, Theme::text2());
        if let Some(h) = c.prioritaet_hinweis(n.pid) { ui.text(&ui.fit(&h, 14.0, r.w - 48.0), r.x + 24.0, r.y + 220.0, 14.0, Theme::akzent()); }
        y += 24.0;
    }
    if n.stage == 0 {
        let sp = if n.verkauf { 0.0 } else { c.preis_spanne(n.pid) as f64 };
        let fo = n.ask.max(n.counter);
        let ft = if sp <= 0.001 { money(fo) } else { crate::tf!("stufe.spanne", money(round_money(fo * (1.0 - sp))), money(round_money(fo * (1.0 + sp)))) };
        ui.text(&crate::tf!("ui.pages.marktwert_preisvorstellung", money(pl.value), ft), r.x + 24.0, y, 17.0, Theme::text2());
        y += 34.0;
        ui.text_semi(crate::inhalt::t("ui.pages.dein_angebot"), r.x + 24.0, y, 18.0, Theme::text());
        ui.title(&money(n.fee), r.x + 200.0, y + 4.0, 28.0, Theme::gold());
        if editable {
            let base = n.ask.max(1.0);
            let t = ((n.fee / base) * 50.0).round() as i32;
            let (lo, hi) = if n.verkauf { (50, 100) } else { (25, 100) };
            if let Some(nt) = ui.slider(Rect::new(r.x + 24.0, y + 24.0, r.w - 48.0, 18.0), t, lo, hi) { c.negs[i].fee = (base * nt as f64 / 50.0 / 50000.0).round() * 50000.0; }
            if n.counter > 0.0 && ui.button(Rect::new(r.x + 24.0, y + 54.0, 260.0, 34.0), crate::inhalt::t("gs.ui.gegen_annehmen"), false) { c.neg_annehmen(i); }
        }
    } else {
        ui.text(&crate::tf!("ui.pages.bisher_forderung_etwa", if n.renew { money(pl.wage) } else { "–".into() }, money(n.need.max(n.counter))), r.x + 24.0, y, 17.0, Theme::text2());
        y += 34.0;
        ui.text_semi(crate::inhalt::t("af.gehalt"), r.x + 24.0, y, 18.0, Theme::text());
        ui.title(&money(n.wage), r.x + 200.0, y + 4.0, 28.0, Theme::gold());
        if editable {
            let base = n.need.max(1.0);
            let t = ((n.wage / base) * 50.0).round() as i32;
            if let Some(nt) = ui.slider(Rect::new(r.x + 24.0, y + 24.0, r.w - 48.0, 18.0), t, 25, 100) { c.negs[i].wage = (base * nt as f64 / 50.0 / 5000.0).round() * 5000.0; }
            ui.text(crate::inhalt::t("gs.folge.label.laufzeit"), r.x + 24.0, y + 74.0, 17.0, Theme::text());
            if let Some(s) = ui.segment(Rect::new(r.x + 120.0, y + 54.0, 300.0, 30.0), &["1", "2", "3", "4", "5"], n.years as usize - 1) { c.negs[i].years = s as u32 + 1; }
            if n.counter > 0.0 && ui.button(Rect::new(r.x + 440.0, y + 54.0, 196.0, 30.0), crate::inhalt::t("ui.pages.forderung_annehmen"), false) { c.neg_annehmen(i); }
        }
    }
    if editable {
        // tone and action at the table (runden.py TON / HANDLUNG) and the chance the offer is taken
        let ty = r.y + 360.0;
        ui.text(crate::inhalt::t("ton.titel"), r.x + 24.0, ty + 20.0, 16.0, Theme::text2());
        let tn: Vec<&str> = crate::verhandlung::toene_t().iter().map(|t| t.1).collect();
        if let Some(k) = ui.segment(Rect::new(r.x + 120.0, ty, 420.0, 30.0), &tn, n.ton as usize) { c.negs[i].ton = k as u8; }
        ui.text(crate::inhalt::t("gb.trainer.auftreten_titel"), r.x + 24.0, ty + 62.0, 16.0, Theme::text2());
        let hn: Vec<&str> = crate::verhandlung::handlungen_t().iter().map(|t| t.1).collect();
        if let Some(k) = ui.segment(Rect::new(r.x + 120.0, ty + 42.0, 516.0, 30.0), &hn, n.handlung as usize) { c.negs[i].handlung = k as u8; }
        if ui.button(Rect::new(r.x + 24.0, ty + 86.0, 200.0, 30.0), if n.letzt { crate::inhalt::t("ui.pages.letztes_angebot_ja") } else { crate::inhalt::t("ui.pages.als_letztes_angebot") }, n.letzt) { c.negs[i].letzt = !n.letzt; }
        if n.zeit < crate::verhandlung::ZEIT_MAX && ui.button(Rect::new(r.x + 232.0, ty + 86.0, 150.0, 30.0), crate::inhalt::t("ui.pages.zeit_erbitten"), false) { match c.neg_zeit(i) { Ok(()) => v.toast(crate::inhalt::t("vhd.art.zeit"), true), Err(e) => v.toast(&e, false) } }
        // clauses (Punkt 6): for the player release, bonuses and raise; in a sale sell-on and buy-back
        let ky = ty + 132.0;
        let t = crate::inhalt::t;
        if n.stage == 1 && !n.verkauf {
            let mut w = n.kl;
            ui.text(t("ui.klausel.ausstieg"), r.x + 24.0, ky + 20.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 150.0, ky, 230.0, 28.0), &[t("ui.klausel.keine"), "1,5×", "2×", "3×"], w.ausstieg as usize) { w.ausstieg = k as u8; }
            ui.text(t("ui.klausel.steigerung_kurz"), r.x + 396.0, ky + 20.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 486.0, ky, 150.0, 28.0), &["0 %", "5 %", "10 %"], w.steigerung as usize) { w.steigerung = k as u8; }
            let stufen = [t("ui.klausel.keine"), t("ui.klausel.klein"), t("ui.klausel.mittel"), t("ui.klausel.hoch")];
            ui.text(t("ui.klausel.tor"), r.x + 24.0, ky + 56.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 150.0, ky + 36.0, 230.0, 28.0), &stufen, w.tor as usize) { w.tor = k as u8; }
            ui.text(t("ui.klausel.einsatz_kurz"), r.x + 396.0, ky + 56.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 486.0, ky + 36.0, 150.0, 28.0), &stufen[..3], w.einsatz.min(2) as usize) { w.einsatz = k as u8; }
            ui.text(t("ui.klausel.titelpraemie"), r.x + 24.0, ky + 92.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 150.0, ky + 72.0, 230.0, 28.0), &stufen, w.titel as usize) { w.titel = k as u8; }
            if w != n.kl { c.negs[i].kl = w; }
            if c.will_ausstieg(n.pid) && w.ausstieg == 0 { ui.text_passend(t("ui.klausel.will_ausstieg"), r.x + 396.0, ky + 84.0, 13.0, 240.0, 2, Theme::achtung()); }
            // an argument with the next offer (berater.rs): believable and fitting his priorities, or it costs patience
            ui.text(t("ui.argument.titel"), r.x + 24.0, ky + 128.0, 14.0, Theme::text2());
            let namen: Vec<&str> = (0..7).map(|k| t(&format!("ui.argument.{k}"))).collect();
            if let Some(k) = ui.segment(Rect::new(r.x + 150.0, ky + 108.0, 486.0, 28.0), &namen, n.argument as usize) { c.negs[i].argument = k as u8; }
            if !n.gefragt && ui.button(Rect::new(r.x + 24.0, ky + 142.0, 260.0, 26.0), t("ui.argument.frage"), false) {
                match c.neg_frage(i) { Ok(x) => { c.negs[i].verlauf.push((0, t("ui.chat.frage").to_string())); c.negs[i].verlauf.push((1, x)); v.neg_tipp = get_time(); } Err(e) => v.toast(&e, false) }
            }
            if !n.renew && ui.button(Rect::new(r.x + 296.0, ky + 142.0, 340.0, 26.0), t(if n.tausch { "ui.argument.tausch_an" } else { "ui.argument.tausch" }), n.tausch) { c.negs[i].tausch = !n.tausch; }
        } else if n.verkauf {
            let mut w = n.kl;
            ui.text(t("ui.klausel.weiterverkauf"), r.x + 24.0, ky + 20.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 180.0, ky, 260.0, 28.0), &["0 %", "10 %", "20 %", "30 %"], w.weiterverkauf as usize) { w.weiterverkauf = k as u8; }
            ui.text(t("ui.klausel.rueckkauf"), r.x + 24.0, ky + 56.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 180.0, ky + 36.0, 260.0, 28.0), &[t("ui.klausel.keine"), "1,3×", "1,6×"], w.rueckkauf as usize) { w.rueckkauf = k as u8; }
            zahlweise_zeilen(ui, &mut w, r.x + 24.0, ky + 72.0, n.fee);
            if w != n.kl { c.negs[i].kl = w; }
        } else if n.stage == 0 {
            // buying: pay in instalments, or with an add-on after matches (Fabian 24.11.)
            let mut w = n.kl;
            zahlweise_zeilen(ui, &mut w, r.x + 24.0, ky, n.fee);
            if w != n.kl { c.negs[i].kl = w; }
        }
        if let Some(ph) = &n.ph {
            let wert = if n.verkauf { -n.fee } else if n.stage == 0 { n.fee } else { n.wage } + c.klausel_kv(&c.negs[i]);
            let bonus = if n.stage == 0 { c.scout_bonus(n.pid) } else { 0.0 };
            let ch = ph.chance(wert, n.letzt, n.ton as usize, n.handlung as usize, 1, bonus);
            let wort = if ch >= 0.99 { crate::inhalt::t("ui.wort.sicher") } else if ch >= 0.6 { crate::inhalt::t("ui.wort.gut") } else if ch >= 0.3 { crate::inhalt::t("ui.wort.offen") } else if ch > 0.0 { crate::inhalt::t("ui.wort.gering") } else { crate::inhalt::t("ui.wort.keine") };
            ui.text_semi(&crate::tf!("ui.pages.aussicht_auf_zusage", wort, format!("{:.0}", ch * 100.0)), r.x + 396.0, ty + 106.0, 17.0, if ch >= 0.6 { Theme::gut() } else if ch >= 0.3 { Theme::achtung() } else { Theme::problem() });
        }
    }
    let by = r.y + r.h - 64.0;
    let tippt = get_time() - v.neg_tipp < NEG_TIPP_S;
    if editable && !tippt && ui.button_gold(Rect::new(r.x + 24.0, by, 280.0, 44.0), crate::inhalt::t("ui.pages.angebot_abschicken")) { c.submit_neg(i); v.neg_tipp = get_time(); ui.play(crate::id("bestaetigt")); }
    if n.stage < 2 && ui.button(Rect::new(r.x + 316.0, by, 150.0, 44.0), crate::inhalt::t("fenster.pk.unterbrechen"), false) { c.negs[i].stage = 2; c.negs[i].state = NegState::Rejected; c.negs[i].text = crate::inhalt::t("ui.pages.du_hast_die_gespraeche_beendet").into(); }
    if ui.button(Rect::new(r.x + r.w - 160.0, by, 136.0, 44.0), crate::inhalt::t("aktion.schliessen"), false) || is_key_pressed(KeyCode::Escape) { v.modal = Modal::None; }
}

/// How long the other side "types" before its answer shows (seconds).
const NEG_TIPP_S: f64 = 1.6;
/// The chat of a negotiation: our offers on the right in the club colour, the other side on the left with its face,
/// the newest at the bottom; while they "type", three dots instead of their answer.
fn neg_chat(ui: &mut Ui, v: &mut View, c: &Career, i: usize, r: Rect) {
    let n = &c.negs[i];
    rrect(r.x, r.y, r.w, r.h, 10.0, Theme::flaeche());
    let pl = &c.world.players[n.pid];
    let gegen = if n.verkauf { c.world.clubs[n.kaeufer].name.clone() } else if n.stage == 0 && pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { c.berater_name(n.pid) };
    ui.text_semi(&ui.fit(&gegen, 18.0, r.w - 80.0), r.x + 58.0, r.y + 30.0, 18.0, Theme::text());
    ui.player_photo(r.x + 30.0, r.y + 24.0, 16.0, pl.id, &pl.name);
    let tippt = get_time() - v.neg_tipp < NEG_TIPP_S;
    ui.text(if tippt { crate::inhalt::t("ui.chat.tippt") } else { crate::inhalt::t("ui.chat.online") }, r.x + 58.0, r.y + 48.0, 13.0, if tippt { Theme::akzent() } else { Theme::gut() });
    draw_rectangle(r.x + 10.0, r.y + 60.0, r.w - 20.0, 1.0, Theme::kontur());
    // lay out from the bottom: the newest above the input line
    let mut zeilen: Vec<(u8, String)> = n.verlauf.clone();
    if tippt { if let Some(pos) = zeilen.iter().rposition(|x| x.0 == 1) { if pos + 1 == zeilen.len() { zeilen.pop(); } } }
    let bw = r.w * 0.78;
    let mut y = r.y + r.h - 16.0;
    if tippt {
        let br = Rect::new(r.x + 14.0, y - 40.0, 70.0, 34.0);
        rrect(br.x, br.y, br.w, br.h, 14.0, Theme::erhoehung());
        for k in 0..3 { let hop = ((get_time() * 6.0 - k as f64).sin().max(0.0) * 4.0) as f32; draw_circle(br.x + 20.0 + k as f32 * 15.0, br.y + 18.0 - hop, 4.0, Theme::text2()); }
        y -= 48.0;
    }
    for (wer, txt) in zeilen.iter().rev() {
        let z = ui.wrap(txt, 15.0, bw - 24.0);
        let h = 16.0 + z.len() as f32 * 20.0;
        if y - h < r.y + 66.0 { break; }
        let x = if *wer == 0 { r.x + r.w - 14.0 - bw } else { r.x + 14.0 };
        let (bg, fg) = match wer { 0 => (Theme::verein(), Theme::verein_auf()), 1 => (Theme::erhoehung(), Theme::text()), _ => (alpha(Theme::gold(), 0.2), Theme::text()) };
        rrect(x, y - h, bw, h, 14.0, bg);
        let mut ty = y - h + 22.0;
        for l in &z { ui.text(l, x + 12.0, ty, 15.0, fg); ty += 20.0; }
        y -= h + 10.0;
    }
}

// ================================================================ Verträge
pub(super) fn vertraege(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("reiter.uebersicht"), crate::inhalt::t("menu.vertraege.reiter.1")]);
    let mut sq = c.squad();
    if v.tab == 1 { sq.retain(|&p| c.world.players[p].contract_end <= c.world.season); }
    sq.sort_by_key(|&p| c.world.players[p].contract_end);
    sortieren(v, 30, &mut sq, |p, col| {
        let pl = &c.world.players[p];
        match col {
            0 => Sk::S(pl.name.to_lowercase()), 1 => Sk::N(pl.age as f32), 2 => Sk::N(pl.ovr), 3 => Sk::N(pl.wage as f32),
            4 => Sk::N(-(pl.contract_end as f32)), _ => Sk::S(personas_t()[pl.persona as usize].to_string()),
        }
    });
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    sort_head(ui, v, 30, area.x, area.y + 24.0, &[(crate::inhalt::t("ui.wort.gross_name"), 60.0), (crate::inhalt::t("ui.wort.gross_alter"), 330.0), (crate::inhalt::t("ui.pages.staerke"), 400.0), (crate::inhalt::t("ui.wort.gross_gehalt"), 480.0), (crate::inhalt::t("ui.pages.vertrag_bis_2"), 640.0), (crate::inhalt::t("ui.wort.gross_berater"), 780.0)]);
    let vis = ((area.h - 44.0) / 40.0) as usize;
    let start = scroll_rows(ui, v, area, sq.len(), vis);
    for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
        let pl = c.world.players[p].clone();
        let r = Rect::new(area.x + 4.0, area.y + 36.0 + k as f32 * 40.0, area.w - 8.0, 38.0);
        if k % 2 == 1 { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::erhoehung(), 0.6)); }
        let cy = r.y + 25.0;
        ui.pos_tag(r.x + 10.0, cy - 5.0, pos_de(&pl.pos));
        ui.player_photo(r.x + 60.0, cy - 5.0, 13.0, pl.id, &pl.name);
        let nm = ui.fit(&pl.name, 17.0, 240.0);
        ui.text_semi(&nm, r.x + 80.0, cy, 17.0, Theme::text());
        ui.text(&format!("{}", pl.age), r.x + 336.0, cy, 17.0, Theme::text());
        ui.pill(r.x + 420.0, cy - 6.0, pl.ovr, 36.0);
        ui.text(&money(pl.wage), r.x + 480.0, cy, 17.0, Theme::text());
        ui.text(&format!("{}", date::START_YEAR as u32 + pl.contract_end + 1), r.x + 640.0, cy, 17.0, if pl.contract_end <= c.world.season { Theme::achtung() } else { Theme::text() });
        ui.text(personas_t()[pl.persona as usize], r.x + 780.0, cy, 16.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 300.0, r.y + 4.0, 140.0, 30.0), crate::inhalt::t("ui.pages.verlaengern"), false) {
            match c.open_neg(p) { Ok(i) => v.modal = Modal::Neg(i), Err(e) => v.toast(&e, false) }
        }
        if ui.button(Rect::new(r.x + r.w - 150.0, r.y + 4.0, 140.0, 30.0), if pl.listed { crate::inhalt::t("ui.pages.von_liste") } else { crate::inhalt::t("gl.chip.transferliste") }, pl.listed) { c.world.players[p].listed = !pl.listed; }
    }
}
