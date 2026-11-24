use super::*;

/// The way into a match before kick-off (Fabian 23.11., like Anstoss and Fussball Manager): the stadium and its city,
/// the two teams side by side, then the team talk with single players; "Weiter" leads to kick-off.
fn vorlauf_seite(ui: &mut Ui, v: &mut View, c: &mut Career, mv: &mut MatchView, body: Rect) {
    let t = crate::inhalt::t;
    let (h, a) = (mv.res.h, mv.res.a);
    let opp = if h == c.own { a } else { h };
    let schritte = [t("ui.vorlauf.stadion"), t("ui.vorlauf.duell"), t("ui.vorlauf.ansprache"), t("gl.anpfiff")];
    for (k, s) in schritte.iter().enumerate() {
        let x = body.x + k as f32 * 170.0;
        rrect(x, body.y, 160.0, 28.0, 14.0, if k == mv.vorlauf as usize { Theme::akzent() } else { Theme::erhoehung() });
        ui.text_center(s, x + 80.0, body.y + 19.0, 14.0, if k == mv.vorlauf as usize { Theme::auf_akzent() } else { Theme::text2() });
    }
    if ui.button(Rect::new(body.x + body.w - 180.0, body.y - 2.0, 180.0, 32.0), t("ui.vorlauf.direkt"), false) { mv.vorlauf = 3; return; }
    let r = Rect::new(body.x, body.y + 40.0, body.w, body.h - 96.0);
    match mv.vorlauf {
        0 => {
            // the stadium: picture, name, city, capacity; the map of the league with the home club marked
            let hc = c.world.clubs[h].clone();
            let rd = crate::realdaten::verein(&hc.name);
            let links = Rect::new(r.x, r.y, r.w * 0.62, r.h);
            ui.card(links);
            if !ui.stadion_bild(Rect::new(links.x + 8.0, links.y + 8.0, links.w - 16.0, links.h - 110.0), hc.id, &hc.name) {
                rrect(links.x + 8.0, links.y + 8.0, links.w - 16.0, links.h - 110.0, 8.0, club_color(c, h).0);
                ui.club_crest(links.x + links.w / 2.0, links.y + (links.h - 110.0) / 2.0, 60.0, hc.id, club_color(c, h).0, &hc.short);
            }
            let stadion = rd.map(|x| x.stadion.clone()).filter(|s| !s.is_empty()).unwrap_or_else(|| crate::tf!("ui.vorlauf.stadion_von", hc.name.clone()));
            let ort = rd.map(|x| x.ort.clone()).unwrap_or_default();
            ui.title(&ui.fit(&stadion, 30.0, links.w - 32.0), links.x + 16.0, links.y + links.h - 62.0, 30.0, Theme::text());
            let kap = if h == c.own { format!(" · {}", crate::tf!("ui.vorlauf.plaetze", format!("{:.0}", c.kapazitaet()))) } else { String::new() };
            ui.text(&format!("{ort}{kap}"), links.x + 16.0, links.y + links.h - 30.0, 17.0, Theme::text2());
            let rechts = Rect::new(r.x + r.w * 0.62 + 12.0, r.y, r.w * 0.38 - 12.0, r.h);
            ui.card(rechts);
            let teams = c.world.leagues[c.world.clubs[h].league].teams.clone();
            let (mut zoom, mut off) = (v.karte_zoom, v.karte_off);
            let _ = crate::geo::karte(ui, Rect::new(rechts.x + 8.0, rechts.y + 8.0, rechts.w - 16.0, rechts.h - 16.0), &c.world, &teams, Some(h), &mut zoom, &mut off);
            v.karte_zoom = zoom; v.karte_off = off;
        }
        1 => {
            // the duel (Fabian 24.11.: "still very empty"): per side coach, form, last results and key players; in the
            // middle the record, the odds and the voices before the match (pundit, press, one interview per side)
            let tab = c.world.table(c.league());
            let staerke = |c: &Career, club: usize| { let elf: Vec<(String, usize, f32)> = if club == c.own { c.current_xi() } else { c.world.lineup(club, 0, &[]).0 }; (elf.iter().map(|x| x.2).sum::<f32>() / elf.len().max(1) as f32, elf) };
            let (sh, elf_h) = staerke(c, h);
            let (sa, elf_a) = staerke(c, a);
            let schluessel = |c: &Career, elf: &[(String, usize, f32)]| { let mut v: Vec<usize> = elf.iter().map(|x| x.1).collect(); v.sort_by(|x, y| { let (p, q) = (&c.world.players[*x], &c.world.players[*y]); (q.ovr + q.goals as f32 * 0.6).partial_cmp(&(p.ovr + p.goals as f32 * 0.6)).unwrap() }); v.truncate(3); v };
            let (key_h, key_a) = (schluessel(c, &elf_h), schluessel(c, &elf_a));
            if mv.duell.is_empty() { mv.duell = c.duell_stimmen(h, a, sh, sa, key_h[0], key_a[0]); }
            let sw = r.w * 0.3;
            for (k, (club, st, keys)) in [(h, sh, &key_h), (a, sa, &key_a)].into_iter().enumerate() {
                let kr = Rect::new(if k == 0 { r.x } else { r.x + r.w - sw }, r.y, sw, r.h);
                let (farbe, _) = club_color(c, club);
                ui.card_titled(kr, &c.world.clubs[club].name.clone(), farbe);
                let form = tab.iter().find(|x| x.club == club).map(|x| x.form.clone()).unwrap_or_default();
                form_squares(ui, kr.x + kr.w - 14.0 - form.len() as f32 * 26.0, kr.y + 44.0, &form);
                ui.text(&ui.fit(&crate::tf!("ui.duell.trainer", c.world.clubs[club].manager.clone()), 15.0, kr.w - 28.0), kr.x + 14.0, kr.y + 82.0, 15.0, Theme::text());
                ui.text(&crate::tf!("ui.vorlauf.staerke_platz", format!("{st:.0}"), c.world.rank_of(club)), kr.x + 14.0, kr.y + 104.0, 14.0, Theme::text2());
                // the last five league results
                ui.text_semi(t("ui.duell.letzte"), kr.x + 14.0, kr.y + 136.0, 13.0, Theme::gedaempft());
                let l = c.world.clubs[club].league;
                let mut fx: Vec<crate::world::Fixture> = c.world.leagues[l].fixtures.iter().filter(|f| f.hg >= 0 && (f.h == club || f.a == club)).cloned().collect();
                fx.sort_by_key(|f| std::cmp::Reverse(f.day));
                for (i, f) in fx.iter().take(5).enumerate() {
                    let y = kr.y + 158.0 + i as f32 * 24.0;
                    let heim = f.h == club;
                    let (g, gg) = if heim { (f.hg, f.ag) } else { (f.ag, f.hg) };
                    let geg = c.world.clubs[if heim { f.a } else { f.h }].name.clone();
                    let col = if g > gg { Theme::gut() } else if g == gg { Theme::achtung() } else { Theme::problem() };
                    rrect(kr.x + 14.0, y - 13.0, 40.0, 18.0, 4.0, col);
                    ui.text_center(&format!("{g}:{gg}"), kr.x + 34.0, y + 1.0, 13.0, hex(0xffffff));
                    ui.text(&ui.fit(&format!("{} {geg}", if heim { t("ui.duell.gegen") } else { "@" }), 14.0, kr.w - 80.0), kr.x + 62.0, y + 1.0, 14.0, Theme::text()); // roh
                }
                // the key players
                ui.text_semi(t("ui.duell.schluessel"), kr.x + 14.0, kr.y + 300.0, 13.0, Theme::gedaempft());
                for (i, &p) in keys.iter().enumerate() {
                    let y = kr.y + 330.0 + i as f32 * 44.0;
                    if y > kr.y + kr.h - 20.0 { break; }
                    let pl = &c.world.players[p];
                    ui.player_photo(kr.x + 32.0, y, 17.0, pl.id, &pl.name);
                    ui.text(&ui.fit(&pl.name, 15.0, kr.w - 130.0), kr.x + 58.0, y - 2.0, 15.0, Theme::text());
                    ui.pos_tag(kr.x + 58.0, y + 12.0, pos_de(&pl.pos));
                    ui.text(&crate::tf!("ui.duell.tore_vorlagen", pl.goals, pl.assists), kr.x + 102.0, y + 16.0, 12.0, Theme::text2());
                    ui.pill(kr.x + kr.w - 30.0, y, pl.ovr, 32.0);
                }
            }
            // the middle: record, odds, voices
            let m = Rect::new(r.x + sw + 12.0, r.y, r.w - 2.0 * sw - 24.0, r.h);
            ui.card_titled(m, t("ui.vorlauf.duell"), Theme::akzent());
            let b = c.spiele.iter().filter(|x| x.gegner == opp).fold((0, 0, 0), |s, x| if x.tore > x.gegentore { (s.0 + 1, s.1, s.2) } else if x.tore == x.gegentore { (s.0, s.1 + 1, s.2) } else { (s.0, s.1, s.2 + 1) });
            ui.text_center(&crate::tf!("ui.vorlauf.bilanz", b.0, b.1, b.2), m.x + m.w / 2.0, m.y + 62.0, 14.0, Theme::text2());
            // odds from the strengths (home advantage included), three made-up bookmakers with a little spread
            let (ph, px, pa) = crate::pages::wahrscheinlichkeiten(sh, sa);
            ui.text_semi(t("ui.duell.quoten"), m.x + 14.0, m.y + 92.0, 13.0, Theme::gedaempft());
            let spalte = (m.w - 28.0 - 110.0) / 3.0;
            for (j, kopf) in ["1", "X", "2"].iter().enumerate() { ui.text_center(kopf, m.x + 124.0 + spalte * (j as f32 + 0.5), m.y + 92.0, 13.0, Theme::gedaempft()); } // roh
            for (i, (name, marge, farbe)) in [("BetLine", 0.93, hex(0x1f8a4c)), ("Quotenhaus", 0.91, hex(0xc2410c)), ("GoalOdds", 0.94, hex(0x1d4ed8))].iter().enumerate() { // roh
                let y = m.y + 104.0 + i as f32 * 30.0;
                rrect(m.x + 14.0, y, 100.0, 24.0, 5.0, *farbe);
                ui.text_center(name, m.x + 64.0, y + 17.0, 13.0, hex(0xffffff));
                let schwank = (crate::mensch::zufall(&[name, &c.abs().to_string()]) - 0.5) * 0.04;
                for (j, p) in [ph, px, pa].iter().enumerate() {
                    let q = (marge + if j == 1 { 0.0 } else { schwank }) / p.max(0.02);
                    rrect(m.x + 124.0 + spalte * j as f32 + 4.0, y, spalte - 8.0, 24.0, 5.0, Theme::erhoehung());
                    ui.text_center(&format!("{q:.2}"), m.x + 124.0 + spalte * (j as f32 + 0.5), y + 17.0, 14.0, Theme::text());
                }
            }
            ui.text(t("ui.duell.quoten_hinweis"), m.x + 14.0, m.y + 210.0, 11.0, Theme::gedaempft());
            // the voices, written once per match
            let mut y = m.y + 238.0;
            let etiketten = [t("ui.duell.tv"), t("ui.duell.presse_label"), &c.world.players[key_h[0]].name.clone(), &c.world.players[key_a[0]].name.clone()];
            for (k, text) in mv.duell.iter().enumerate() {
                if y > m.y + m.h - 40.0 { break; }
                ui.text_semi(&ui.fit(etiketten[k.min(3)], 13.0, m.w - 28.0), m.x + 14.0, y, 13.0, if k < 2 { Theme::akzent() } else { club_color(c, if k == 2 { h } else { a }).0 });
                let n = ui.text_passend(&crate::tf!("ui.duell.zitat", text.clone()), m.x + 14.0, y + 20.0, 14.0, m.w - 28.0, 4, Theme::text());
                y += 30.0 + 20.0 * n as f32;
            }
        }
        _ => {
            // the team talk on the left, single players on the right: praise or a push (at most two)
            let w2 = (r.w - 12.0) / 2.0;
            let lr = Rect::new(r.x, r.y, w2, r.h);
            ui.card_titled(lr, t("ui.vorlauf.ansprache"), Theme::akzent());
            teamgespraech_karte(ui, v, c, Rect::new(lr.x + 14.0, lr.y + 50.0, lr.w - 28.0, lr.h - 64.0), opp);
            let rr = Rect::new(r.x + w2 + 12.0, r.y, w2, r.h);
            ui.card_titled(rr, t("ui.vorlauf.einzeln"), Theme::gold());
            ui.paragraph(t("ui.vorlauf.einzeln_hinweis"), rr.x + 14.0, rr.y + 58.0, 13.0, rr.w - 28.0, Theme::text2());
            for (i, (_, p, _)) in c.current_xi().iter().enumerate() {
                let y = rr.y + 96.0 + i as f32 * 30.0;
                if y > rr.y + rr.h - 20.0 { break; }
                let pl = c.world.players[*p].clone();
                ui.player_photo(rr.x + 26.0, y + 10.0, 11.0, pl.id, &pl.name);
                ui.text(&ui.fit(&pl.name, 15.0, rr.w * 0.45), rr.x + 44.0, y + 15.0, 15.0, Theme::text());
                ui.smiley(rr.x + rr.w * 0.6, y + 10.0, 6.0, pl.morale);
                if mv.angesprochen.contains(p) { ui.text_right(t("ui.vorlauf.angesprochen"), rr.x + rr.w - 14.0, y + 15.0, 13.0, Theme::gut()); continue; }
                if mv.angesprochen.len() >= 2 { continue; }
                let note = form5(&pl).unwrap_or(6.5);
                if ui.button(Rect::new(rr.x + rr.w - 170.0, y, 74.0, 24.0), t("ans.lob"), false) { let w = c.spieler_ansprechen(*p, true, note); mv.wirkung.push(w.clone()); mv.angesprochen.push(*p); v.toast(&w, true); }
                if ui.button(Rect::new(rr.x + rr.w - 88.0, y, 74.0, 24.0), t("ui.vorlauf.anstacheln"), false) { let w = c.spieler_ansprechen(*p, false, note); mv.wirkung.push(w.clone()); mv.angesprochen.push(*p); v.toast(&w, true); }
            }
        }
    }
    let by = body.y + body.h - 48.0;
    if mv.vorlauf > 0 && ui.button(Rect::new(body.x, by, 160.0, 44.0), t("karrieren.zurueck"), false) { mv.vorlauf -= 1; }
    if ui.button_gold(Rect::new(body.x + body.w - 220.0, by, 220.0, 44.0), t("ui.wort.gross_weiter")) { mv.vorlauf += 1; }
}

/// Win, draw and loss chance of the home side from both elevens' strengths (the home side gets about four points).
pub fn wahrscheinlichkeiten(heim: f32, gast: f32) -> (f32, f32, f32) {
    let d = (heim + 4.0 - gast) / 10.0;
    let remis = (0.29 - 0.08 * d.abs()).clamp(0.14, 0.29);
    let h = (1.0 - remis) / (1.0 + (-d * 1.6).exp());
    (h, remis, 1.0 - remis - h)
}

/// The team talk before the match (prematch.rs): the expectation, goal, tone and focus preset by the recommendation, once per match.
fn teamgespraech_karte(ui: &mut Ui, v: &mut View, c: &mut Career, r: Rect, opp: usize) {
    use crate::prematch::{EINORDNUNG, FOKUS, TOENE, ZIELE};
    let Some(e) = c.spiel_erwartung() else { return };
    let abs = c.abs();
    if v.pm_wahl.0 != abs { v.pm_wahl = (abs, [e.ziel, e.ton, e.fokus]); }
    draw_rectangle(r.x, r.y, r.w, 1.0, Theme::kontur());
    ui.text_semi(crate::inhalt::t("pm.titel"), r.x, r.y + 26.0, 17.0, Theme::akzent());
    let drei = ["favorit", "favorit", "offen", "aussenseiter", "offen"][e.einordnung]; // roh
    let mut txt = crate::tf!(&format!("sp.vor.einordnung.{drei}"), c.world.clubs[opp].name.clone()); // roh
    if e.einordnung == 0 || e.einordnung == 4 { txt = format!("{} {txt}", crate::inhalt::t(&format!("pm.einordnung.{}", EINORDNUNG[e.einordnung]))); } // roh
    let mut y = r.y + 38.0 + 22.0 * ui.text_passend(&txt, r.x, r.y + 52.0, 14.0, r.w, 2, Theme::text2()) as f32;
    let schon = c.ansprache_vor.is_some_and(|x| x.tag == abs);
    let reihen: [(&str, Vec<&str>); 3] = [("pm.ziel", ZIELE.iter().map(|z| crate::inhalt::t(&format!("sp.ziel.{z}"))).collect()), ("pm.ton", TOENE.iter().map(|z| crate::inhalt::t(&format!("sp.ton.vor.{z}"))).collect()), ("pm.fokus", FOKUS.iter().map(|z| crate::inhalt::t(&format!("sp.fokus.{z}"))).collect())]; // roh
    for (k, (titel, opts)) in reihen.iter().enumerate() {
        ui.text(crate::inhalt::t(titel), r.x, y + 20.0, 14.0, Theme::text2());
        if !schon { if let Some(s) = ui.segment(Rect::new(r.x + 80.0, y, r.w - 80.0, 28.0), opts, v.pm_wahl.1[k]) { v.pm_wahl.1[k] = s; } } else { ui.text(opts[v.pm_wahl.1[k].min(opts.len() - 1)], r.x + 80.0, y + 20.0, 14.0, Theme::text()); }
        y += 34.0;
    }
    if schon { ui.text(crate::inhalt::t("pm.schon_gesprochen"), r.x, y + 18.0, 14.0, Theme::gedaempft()); return; }
    if ui.button(Rect::new(r.x, y + 2.0, 220.0, 32.0), crate::inhalt::t("pm.knopf"), true) {
        let w = v.pm_wahl.1;
        match c.teamgespraech_vor(w[0], w[1], w[2]) { Ok(m) => v.toast(&m, true), Err(x) => v.toast(&x, false) }
    }
}

pub(super) fn spieltag(ui: &mut Ui, v: &mut View, c: &mut Career) {
    if v.fl27.is_none() && !v.fl27_vorschau { v.fl27_ansicht = None; v.fl27_seq = None; v.fl27_einblendung = None; }
    if v.mv.is_none() {
        let Some(fi) = c.today_match() else {
            ui.title(crate::inhalt::t("ui.pages.heute_ist_kein_spiel"), x0() + 10.0, Y0 + 40.0, 30.0, Theme::text());
            if let Some(f) = c.next_match() { ui.text(&crate::tf!("ui.pages.naechstes_spiel_gegen", date::long(c.world.season, f.day), c.world.clubs[if f.h == c.own { f.a } else { f.h }].name), x0() + 10.0, Y0 + 76.0, 19.0, Theme::text2()); }
            if let Some(r) = c.last_result.clone() { match_report(ui, c, &r, Rect::new(x0() + 4.0, Y0 + 110.0, cbreite(), hoehe() - Y0 - 126.0)); }
            return;
        };
        let f = c.fixture(fi);
        v.mv = Some(MatchView { res: MatchResult { h: f.h, a: f.a, ..Default::default() }, minute: 0.0, speed: 1.0, phase: 0, ended_at: 0.0, manual: (0, 0), manual_open: false, halbzeit: 0, wirkung: vec![], nach_gesagt: false, angesprochen: vec![], vorlauf: std::env::var("PESM_VORLAUF").ok().and_then(|x| x.parse().ok()).unwrap_or(0), eingriff: Default::default(), vor_pause: 1.0, duell: vec![], parallel: vec![] });
    }
    let mut mv = v.mv.take().unwrap();
    let (h, a) = (mv.res.h, mv.res.a);
    // scoreboard
    let sb = Rect::new(x0() + 4.0, Y0 - 42.0, cbreite(), 110.0);
    ui.card(sb);
    // the home team's stadium behind the scoreboard, dimmed so the names stay readable
    if ui.stadion_bild(sb, c.world.clubs[h].id, &c.world.clubs[h].name) { draw_rectangle(sb.x, sb.y, sb.w, sb.h, alpha(Theme::flaeche(), 0.72)); }
    let (c1, c2) = club_color(c, h);
    let (d1, d2) = club_color(c, a);
    let _ = (c2, d2);
    ui.club_crest(sb.x + 70.0, sb.y + 55.0, 34.0, c.world.clubs[h].id, c1, &c.world.clubs[h].short);
    ui.club_crest(sb.x + sb.w - 70.0, sb.y + 55.0, 34.0, c.world.clubs[a].id, d1, &c.world.clubs[a].short);
    ui.title(&c.world.clubs[h].name, sb.x + 120.0, sb.y + 58.0, 26.0, Theme::text());
    let an = c.world.clubs[a].name.clone();
    let aw = measure_text(&an, Some(&ui.head), 26, 1.0).width;
    ui.title(&an, sb.x + sb.w - 120.0 - aw, sb.y + 58.0, 26.0, Theme::text());
    let shown_goals = |side: u8| mv.res.events.iter().filter(|e| (e.kind == 0 && e.side == side || e.kind == 1 && e.side == side) && (e.min as f32) <= mv.minute).count();
    let (sh, sa) = if mv.phase == 0 { (0, 0) } else { (shown_goals(0), shown_goals(1)) };
    let score = crate::tf!("rueckblick.tore_wert", sh, sa);
    let sw = measure_text(&score, Some(&ui.head), 56, 1.0).width;
    ui.title(&score, sb.x + sb.w / 2.0 - sw / 2.0, sb.y + 72.0, 56.0, Theme::text());
    let clock = match mv.phase { 0 => crate::inhalt::t("vorbereitung.checkliste").to_string(), 1 => format!("{}'", mv.minute.min(90.0) as u32), _ => crate::inhalt::t("ui.pages.abpfiff").to_string() };
    ui.text_center(&clock, sb.x + sb.w / 2.0, sb.y + 104.0, 18.0, Theme::akzent());
    // live: expected goals and shots so far, from the match's real shots (statistik.rs)
    if mv.phase >= 1 && !mv.res.schuesse.is_empty() {
        let bis = |side: u8| mv.res.schuesse.iter().filter(|x| x.side == side && (x.min as f32) <= mv.minute).fold((0.0f32, 0u32), |a, x| (a.0 + x.xg as f32 / 1000.0, a.1 + 1));
        let (h0, a0) = (bis(0), bis(1));
        let t = crate::tf!("ui.stat.live_xg", format!("{:.2}", h0.0), format!("{:.2}", a0.0), h0.1, a0.1);
        ui.text(&t, sb.x + sb.w / 2.0 + 44.0, sb.y + 104.0, 14.0, Theme::text2());
    }
    // derby: a ribbon with the duel's name (real rivals first) and the own record against this opponent
    if let Some(derby) = crate::game::is_derby(&c.world.clubs[h].name, &c.world.clubs[a].name) {
        let geg = if h == c.own { a } else { h };
        let b = c.spiele.iter().filter(|x| x.gegner == geg).fold((0, 0, 0), |s, x| if x.tore > x.gegentore { (s.0 + 1, s.1, s.2) } else if x.tore == x.gegentore { (s.0, s.1 + 1, s.2) } else { (s.0, s.1, s.2 + 1) });
        let txt = format!("{} · {}", crate::tf!("ui.derby.band", derby), crate::tf!("ui.derby.bilanz", b.0, b.1, b.2));
        let tw = ui.measure_semi(&txt, 15.0) + 36.0;
        let puls = 0.85 + 0.15 * (get_time() as f32 * 2.5).sin();
        rrect(sb.x + sb.w / 2.0 - tw / 2.0, sb.y - 14.0, tw, 28.0, 14.0, alpha(hex(0xd62839), puls));
        ui.text_center(&txt, sb.x + sb.w / 2.0, sb.y + 5.0, 15.0, hex(0xffffff));
    }
    let comp = c.today_match().map(|t| c.comp_name(t)).unwrap_or_else(|| c.world.leagues[c.league()].name.clone());
    let cid = c.comp_cid(c.today_match().map(|t| t.0).unwrap_or(Wo::Liga(c.league())));
    let wo = c.today_match().map(|t| t.0).unwrap_or(Wo::Liga(c.league()));
    let lx = if ui.wettbewerb_logo_auf(sb.x + 130.0, sb.y + 85.0, 10.0, cid, Some(wo_look(c, wo).c1)) { sb.x + 146.0 } else { sb.x + 120.0 };
    ui.text(&format!("{} · {}", comp, date::long(c.world.season, c.world.day)), lx, sb.y + 90.0, 15.0, Theme::text2());
    let body = Rect::new(x0() + 4.0, sb.y + sb.h + 12.0, cbreite(), hoehe() - sb.y - sb.h - 28.0);
    match mv.phase {
        0 if mv.vorlauf < 3 && v.fl27.is_none() => {
            vorlauf_seite(ui, v, c, &mut mv, body);
            v.mv = Some(mv);
            return;
        }
        0 => {
            // the live view of a match in FL27 lies over this page; the cards below it take no clicks meanwhile
            let live_an = v.fl27_ansicht.is_some() && v.fl27_ko.is_none();
            let klick = ui.clicked;
            if live_an { ui.clicked = false; }
            // it also covers the page banner: its own scoreboard says what the banner would
            let live_flaeche = Rect::new(x0() + 4.0, 62.0, cbreite(), hoehe() - 62.0 - 16.0);
            // opponent report and kickoff choices
            let opp = if h == c.own { a } else { h };
            let r = Rect::new(body.x, body.y, body.w * 0.5 - 6.0, body.h);
            ui.card_titled(r, crate::inhalt::t("gegner.bericht"), Theme::akzent());
            let (oxi, _) = c.world.lineup(opp, 0, &[]);
            let ostr = oxi.iter().map(|x| x.2).sum::<f32>() / oxi.len().max(1) as f32;
            let mine = c.team_strength();
            let (sh_, sa_) = if h == c.own { (mine, ostr) } else { (ostr, mine) };
            let (lh, la) = World::rates(sh_, sa_, if h == c.own { c.tactic.style() } else { c.world.clubs[h].style }, if a == c.own { c.tactic.style() } else { c.world.clubs[a].style });
            let (ph, pd, pa) = World::win_probs(lh, la);
            ui.text(&crate::tf!("ui.pages.staerke_gegen_deine_trainer", format!("{:.0}", ostr), format!("{:.0}", mine), c.world.clubs[opp].manager), r.x + 14.0, r.y + 58.0, 17.0, Theme::text());
            ui.text(&crate::tf!("ui.pages.platz_spielweise", c.world.rank_of(opp), [crate::inhalt::t("ui.wort.defensiv"), crate::inhalt::t("ui.wort.ausgewogen"), crate::inhalt::t("ui.wort.offensiv")][(c.world.clubs[opp].style + 1) as usize]), r.x + 14.0, r.y + 84.0, 17.0, Theme::text2());
            let bw = r.w - 28.0;
            let y = r.y + 110.0;
            rrect(r.x + 14.0, y, bw * ph, 26.0, 4.0, Theme::gut());
            rrect(r.x + 14.0 + bw * ph, y, bw * pd, 26.0, 0.0, Theme::achtung());
            rrect(r.x + 14.0 + bw * (ph + pd), y, bw * pa, 26.0, 4.0, Theme::problem());
            ui.text(&crate::tf!("ui.pages.heimsieg_remis_auswaertssieg", format!("{:.0}", ph * 100.0), format!("{:.0}", pd * 100.0), format!("{:.0}", pa * 100.0)), r.x + 14.0, y + 48.0, 15.0, Theme::text2());
            ui.text_semi(crate::inhalt::t("ui.pages.gefaehrlichste_spieler"), r.x + 14.0, y + 86.0, 16.0, Theme::text2());
            let mut top: Vec<(String, usize, f32)> = oxi.clone();
            top.sort_by(|x, y| y.2.partial_cmp(&x.2).unwrap());
            for (k, (slot, p, e)) in top.iter().take(5).enumerate() {
                let yy = y + 112.0 + k as f32 * 30.0;
                ui.pos_tag(r.x + 14.0, yy - 5.0, pos_de(slot));
                ui.player_photo(r.x + 66.0, yy - 5.0, 11.0, c.world.players[*p].id, &c.world.players[*p].name);
                ui.text(&c.world.players[*p].name, r.x + 84.0, yy, 17.0, Theme::text());
                ui.pill(r.x + r.w - 40.0, yy - 6.0, *e, 36.0);
                zuschlag_text(ui, r.x + r.w - 82.0, yy - 1.0, &c.world.players[*p]);
            }
            let r2 = Rect::new(body.x + body.w * 0.5 + 6.0, body.y, body.w * 0.5 - 6.0, body.h);
            ui.card_titled(r2, crate::inhalt::t("gl.anpfiff"), Theme::gold());
            ui.text(&crate::tf!("ui.pages.deine_startelf", format!("{:.0}", mine), FORMATIONS[c.formation], [crate::inhalt::t("ui.wort.defensiv"), crate::inhalt::t("ui.wort.ausgewogen"), crate::inhalt::t("ui.wort.offensiv")][c.tactic.haltung as usize]), r2.x + 14.0, r2.y + 58.0, 17.0, Theme::text());
            if ui.button(Rect::new(r2.x + 14.0, r2.y + 74.0, 200.0, 32.0), crate::inhalt::t("pm.aufstellung.aendern"), false) { v.mv = Some(mv); v.go(Page::Aufstellung); return; }
            if c.press.is_some() { ui.text(crate::inhalt::t("ui.pages.die_pressekonferenz_vor_dem_spiel"), r2.x + 14.0, r2.y + 126.0, 15.0, Theme::achtung()); }
            if ui.button(Rect::new(r2.x + 14.0, r2.y + 142.0, (r2.w - 40.0) / 2.0, 50.0), crate::inhalt::t("ui.pages.spielen_fl27"), false) { mv.manual_open = true; }
            if ui.button_gold(Rect::new(r2.x + 26.0 + (r2.w - 40.0) / 2.0, r2.y + 142.0, (r2.w - 40.0) / 2.0, 50.0), crate::inhalt::t("dm.knopf.simulieren")) {
                c.undo_merken();
                c.press = None;
                mv.res = c.prepare_match(None);
                mv.parallel = c.parallel_spielen();
                mv.phase = 1;
                mv.eingriff.intro = get_time() as f32;
                ui.play(crate::id("anpfiff"));
            }
            if !mv.manual_open && !live_an { teamgespraech_karte(ui, v, c, Rect::new(r2.x + 14.0, r2.y + 210.0, r2.w - 28.0, r2.h - 224.0), opp); }
            if !mv.manual_open {
                if live_an { ui.clicked = klick; crate::livespiel::zeichnen(ui, v, c, h, a, live_flaeche); halbzeit_fl27(ui, v, c, h, a, live_flaeche); }
                v.mv = Some(mv);
                return;
            }
            // FL27: start via Lutris, then the memory reader brings the result back (Modul-Paket FL27-Anbindung §5)
            let mut y = r2.y + 206.0;
            if !c.fl27_erlaubt {
                ui.paragraph(crate::inhalt::t("ui.pages.der_spielstart_aus_pes_m"), r2.x + 14.0, y, 15.0, r2.w - 28.0, Theme::text2());
                if ui.button(Rect::new(r2.x + 14.0, y + 50.0, 240.0, 34.0), crate::inhalt::t("ui.pages.spielstart_erlauben"), false) { c.fl27_erlaubt = true; }
            } else if !c.fl27_bestaetigt {
                ui.paragraph(crate::inhalt::t("ui.pages.die_karriere_wird_in_den"), r2.x + 14.0, y, 15.0, r2.w - 28.0, Theme::text2());
                if ui.button(Rect::new(r2.x + 14.0, y + 70.0, 300.0, 34.0), crate::inhalt::t("ui.pages.verstanden_nicht_mehr_fragen"), false) { c.fl27_bestaetigt = true; }
            } else if v.fl27.is_none() {
                if ui.button_gold(Rect::new(r2.x + 14.0, y, r2.w - 28.0, 44.0), crate::inhalt::t("ui.pages.match_in_fl27_starten")) {
                    let mut st = Fl27Start { c: &mut *c, assets: fl27_assets(), log: vec![] };
                    let r = crate::fl27::match_starten(&mut st);
                    v.fl27_log = st.log;
                    match r {
                        Ok(()) => { let tf = c.fl27_team_fn(); v.fl27 = Some(crate::fl27::beobachten(tf)); v.fl27_live = None; v.fl27_status = crate::inhalt::t("ui.pages.warte_auf_fl27").into(); v.fl27_t0 = get_time() as f32; ui.play(crate::id("bestaetigt")); }
                        Err(e) => { v.fl27_log.push(e.clone()); v.toast(&e, false); ui.play(crate::id("fehler")); }
                    }
                }
                y += 52.0;
                for l in v.fl27_log.iter().rev().take(3) { let t = ui.fit(l, 14.0, r2.w - 28.0); ui.text(&t, r2.x + 14.0, y + 14.0, 14.0, Theme::text2()); y += 18.0; }
            } else {
                for l in c.fl27_ansage() { y = ui.paragraph(&format!("• {l}"), r2.x + 14.0, y + 14.0, 15.0, r2.w - 28.0, Theme::text()) - 8.0; }
                // §5.9: FL27 must be running within 240 s
                if v.fl27_status == "FL27 läuft nicht." && get_time() as f32 - v.fl27_t0 > 240.0 { v.fl27_status = crate::inhalt::t("ui.pages.fl27_ist_nach_240_s").into(); } // roh
                let live = v.fl27_live.map(|(h, a)| crate::tf!("ui.pages.stand", h, a)).unwrap_or_default();
                ui.text_semi(&format!("{}{live}", v.fl27_status), r2.x + 14.0, y + 26.0, 15.0, Theme::akzent());
                if ui.button(Rect::new(r2.x + r2.w - 170.0, y + 8.0, 156.0, 28.0), crate::inhalt::t("ui.pages.mitlesen_beenden"), false) { v.fl27 = None; }
                if v.fl27_laeuft && v.fl27_ende.is_none() && !v.fl27_ende_wunsch && ui.button(Rect::new(r2.x + r2.w - 340.0, y + 8.0, 160.0, 28.0), crate::inhalt::t("ui.main.fl27_beenden"), false) { fl27_beenden_klick(v); }
                if v.fl27_ende_wunsch {
                    let q = Rect::new(r2.x + 10.0, y + 44.0, r2.w - 20.0, 92.0);
                    rrect(q.x, q.y, q.w, q.h, 10.0, Theme::ueberlagerung());
                    ui.paragraph(crate::inhalt::t("ui.pages.die_partie_laeuft_noch_trotzdem"), q.x + 14.0, q.y + 22.0, 15.0, q.w - 28.0, Theme::text());
                    if ui.button(Rect::new(q.x + 14.0, q.y + 50.0, 200.0, 32.0), crate::inhalt::t("ui.pages.trotzdem_beenden"), false) { v.fl27 = None; }
                    if ui.button(Rect::new(q.x + 224.0, q.y + 50.0, 140.0, 32.0), crate::inhalt::t("fenster.pk.unterbrechen"), false) { v.fl27_ende_wunsch = false; }
                }
            }
            if v.fl27.is_none() && v.fl27_laeuft && v.fl27_ende.is_none() && c.fl27_bestaetigt && ui.button(Rect::new(r2.x + r2.w - 174.0, r2.y + 8.0, 160.0, 28.0), crate::inhalt::t("ui.main.fl27_beenden"), false) { fl27_beenden_klick(v); }
            // reader messages
            let mut fertig = None;
            if let Some(b) = &v.fl27 {
                while let Ok(m) = b.rx.try_recv() {
                    match m {
                        crate::fl27::Meldung::Status(t) => v.fl27_status = t,
                        crate::fl27::Meldung::Live(h, a) => v.fl27_live = Some((h, a)),
                        crate::fl27::Meldung::Ansicht(m) => v.fl27_ansicht = Some(*m),
                        crate::fl27::Meldung::Fertig(d) => { fertig = Some(d); v.fl27_live = None; }
                    }
                }
            }
            if let Some(d) = fertig { if c.fl27_ko_offen(&d) { v.fl27_ko = Some(KoOffen::Doc(d)); v.fl27 = None; ui.play(crate::id("angebot")); } else { v.fl27_ko_apply = Some((KoOffen::Doc(d), None)); } }
            // a level knock-out match from FL27: who won after extra time or on penalties? (no answer = PES M decides)
            if v.fl27_ko.is_some() {
                let q = Rect::new(r2.x + 10.0, r2.y + 196.0, r2.w - 20.0, 250.0);
                rrect(q.x, q.y, q.w, q.h, 14.0, Theme::ueberlagerung());
                ui.title(crate::inhalt::t("ui.pages.unentschieden_wer_ist_weitergekommen"), q.x + 14.0, q.y + 30.0, 22.0, Theme::text());
                ui.text(crate::inhalt::t("ui.pages.sag_pes_m_wie_es"), q.x + 14.0, q.y + 54.0, 15.0, Theme::text2());
                let (hn, an) = (c.world.clubs[h].name.clone(), c.world.clubs[a].name.clone());
                let opts = [(crate::tf!("ui.pages.in_der_verlaengerung", hn), Some((true, false))), (crate::tf!("ui.pages.im_elfmeterschiessen", hn), Some((true, true))),
                    (crate::tf!("ui.pages.in_der_verlaengerung", an), Some((false, false))), (crate::tf!("ui.pages.im_elfmeterschiessen", an), Some((false, true))), (crate::inhalt::t("ui.pages.weiss_ich_nicht_pes_m").to_string(), None)];
                for (k, (label, val)) in opts.iter().enumerate() {
                    let bw = (q.w - 38.0) / 2.0;
                    let r = Rect::new(q.x + 14.0 + (k % 2) as f32 * (bw + 10.0), q.y + 72.0 + (k / 2) as f32 * 46.0, if k == 4 { q.w - 28.0 } else { bw }, 38.0);
                    let label = ui.fit(label, 16.0, r.w - 16.0);
                    if ui.button(r, &label, k < 4) { let p = v.fl27_ko.take().unwrap(); v.fl27_ko_apply = Some((p, *val)); }
                }
            }
            if let Some((p, vorgabe)) = v.fl27_ko_apply.take() {
                c.press = None;
                let aus_fl27 = matches!(p, KoOffen::Doc(_));
                let r = match p {
                    KoOffen::Doc(d) => c.apply_fl27(&d, vorgabe),
                    KoOffen::Hand(res) => { c.world.ko_vorgabe = vorgabe; c.apply_own_match(res); c.world.ko_vorgabe = None; Ok(crate::inhalt::t("ui.pages.ergebnis_uebernommen").to_string()) }
                };
                match r {
                    Ok(t) => {
                        v.toast(&t, true);
                        if let Some(r) = &c.last_result { v.fl27_uebernommen = Some(format!("{} {}:{} {}", c.world.clubs[r.h].name, r.hg, r.ag, c.world.clubs[r.a].name)); }
                        // optional: end FL27 30 s after the final whistle, the protocol is saved by now
                        if aus_fl27 && c.fl27_auto_ende { v.fl27_auto_t = Some(get_time() as f32 + 30.0); }
                        mv.res = c.last_result.clone().unwrap_or_default();
                        mv.phase = 2;
                        mv.ended_at = get_time() as f32;
                        mv.minute = 90.0;
                        v.fl27 = None;
                        ui.play(crate::id("tagesabschluss"));
                        v.mv = Some(mv);
                        return;
                    }
                    Err(e) => v.fl27_status = e,
                }
            }
            ui.text_semi(crate::inhalt::t("ui.pages.ohne_mitlesen_ergebnis_von_hand"), r2.x + 14.0, r2.y + r2.h - 104.0, 15.0, Theme::text2());
            let yy = r2.y + r2.h - 92.0;
            for (k, side) in [(0usize, h), (1usize, a)] {
                let x = r2.x + 14.0 + k as f32 * 200.0;
                ui.text(&c.world.clubs[side].short, x, yy + 24.0, 18.0, Theme::text());
                let val = if k == 0 { mv.manual.0 } else { mv.manual.1 };
                if ui.button(Rect::new(x + 50.0, yy, 36.0, 36.0), "−", false) { if k == 0 { mv.manual.0 = val.saturating_sub(1); } else { mv.manual.1 = val.saturating_sub(1); } }
                ui.title(&format!("{val}"), x + 98.0, yy + 30.0, 30.0, Theme::text());
                if ui.button(Rect::new(x + 124.0, yy, 36.0, 36.0), "+", false) { if k == 0 { mv.manual.0 = (val + 1).min(9); } else { mv.manual.1 = (val + 1).min(9); } }
            }
            if v.fl27_ko.is_none() && ui.button(Rect::new(r2.x + 420.0, yy, 220.0, 36.0), crate::inhalt::t("ui.pages.ergebnis_uebernehmen"), false) {
                let res = c.prepare_match(Some(mv.manual));
                let ko = matches!(c.today_match(), Some((Wo::Pokal(ci), i)) if c.world.ko_offen(ci, i, res.hg, res.ag));
                if ko { v.fl27_ko = Some(KoOffen::Hand(res)); } else { v.fl27_ko_apply = Some((KoOffen::Hand(res), None)); }
            }
            if live_an { ui.clicked = klick; crate::livespiel::zeichnen(ui, v, c, h, a, live_flaeche); halbzeit_fl27(ui, v, c, h, a, live_flaeche); }
        }
        1 => {
            let before = mv.minute;
            // the clock stands at the break until the half-time talk is given (Fabian 22.11.: on the ticker, not in the inbox)
            // PESM_INTRO=<seconds> holds the cut-scene at that moment (preview)
            let t_intro = std::env::var("PESM_INTRO").ok().and_then(|x| x.parse::<f32>().ok()).unwrap_or(get_time() as f32 - mv.eingriff.intro);
            let im_intro = (mv.eingriff.intro > 0.0 || std::env::var("PESM_INTRO").is_ok()) && t_intro < INTRO_DAUER;
            // Fabian 23.11.: slower – at normal speed one half takes about a minute (0.75 match minutes per second)
            if mv.halbzeit != 1 && !im_intro { mv.minute += get_frame_time() * 0.75 * mv.speed; }
            if mv.halbzeit == 0 && before < 45.0 && mv.minute >= 45.0 { mv.minute = 45.0; mv.halbzeit = 1; }
            for e in mv.res.events.iter().filter(|e| (e.min as f32) > before && (e.min as f32) <= mv.minute) { match e.kind { 0 | 1 => ui.play(crate::id("tor")), 2 | 3 => ui.play(crate::id("karte")), 4 => ui.play(crate::id("verletzung")), _ => {} } }
            // the radar: both teams as dots, the ball moving with the match's real situations (Fabian 23.11.)
            let rh = (body.w * 0.62 * 0.42).min(body.h * 0.5);
            radar(ui, c, &mv, Rect::new(body.x, body.y, body.w * 0.62, rh));
            let r = Rect::new(body.x, body.y + rh + 10.0, body.w * 0.62, body.h - rh - 10.0);
            ui.card_titled(r, crate::inhalt::t("ui.wort.live_ticker"), Theme::problem());
            let shown: Vec<&MatchEvent> = mv.res.events.iter().filter(|e| (e.min as f32) <= mv.minute).collect();
            let mut y = r.y + 56.0;
            // what the half-time talk brought, on top
            for w in &mv.wirkung { let hh = ui.text_passend(w, r.x + 16.0, y, 15.0, r.w - 32.0, 2, Theme::akzent()) as f32; y += 20.0 * hh.max(1.0) + 6.0; }
            for e in shown.iter().rev() {
                if y > r.y + r.h - 40.0 { break; }
                // goals and red cards as big cards, like the mobile app (Fabian 23.11.)
                if matches!(e.kind, 0 | 1 | 3) {
                    let team = if e.side == 0 { h } else { a };
                    let (tc, _) = club_color(c, team);
                    let kr = Rect::new(r.x + 12.0, y - 18.0, r.w - 24.0, 78.0);
                    rrect(kr.x, kr.y, kr.w, kr.h, 12.0, if e.kind == 3 { alpha(Theme::problem(), 0.16) } else { alpha(tc, 0.18) });
                    draw_rectangle(kr.x, kr.y + 8.0, 5.0, kr.h - 16.0, if e.kind == 3 { Theme::problem() } else { tc });
                    ui.title(&format!("{}'", e.min), kr.x + 18.0, kr.y + 48.0, 30.0, Theme::text());
                    let pn = c.world.players[e.pid].name.clone();
                    ui.player_photo(kr.x + 108.0, kr.y + 39.0, 26.0, c.world.players[e.pid].id, &pn);
                    let kopf = match e.kind { 0 => crate::inhalt::t("tk.tor"), 1 => crate::inhalt::t("tk.eigentor"), _ => crate::inhalt::t("tk.rot") };
                    ui.title(kopf, kr.x + 146.0, kr.y + 30.0, 24.0, if e.kind == 3 { Theme::problem() } else { Theme::text() });
                    let unter = match e.kind { 0 => format!("{pn}{}", e.pid2.map(|p| crate::tf!("ui.pages.vorlage", c.world.players[p].name)).unwrap_or_default()), _ => pn.clone() };
                    ui.text(&ui.fit(&unter, 16.0, kr.w - 220.0), kr.x + 146.0, kr.y + 56.0, 16.0, Theme::text2());
                    ui.club_crest(kr.x + kr.w - 34.0, kr.y + 39.0, 18.0, c.world.clubs[team].id, tc, &c.world.clubs[team].short);
                    y += 90.0;
                    continue;
                }
                let team = if e.side == 0 { h } else { a };
                let pn = c.world.players[e.pid].name.clone();
                let (txt, col) = match e.kind {
                    0 => (crate::tf!(&ticker_key("ui.pages.tor_fuer", e), c.world.clubs[team].name, pn, e.pid2.map(|p| crate::tf!("ui.pages.vorlage", c.world.players[p].name)).unwrap_or_default()), Theme::gut()),
                    1 => (crate::tf!("ui.pages.eigentor_von", pn), Theme::problem()),
                    2 => (crate::tf!(&ticker_key("ui.pages.gelbe_karte_fuer", e), pn), Theme::achtung()),
                    3 => (crate::tf!(&ticker_key("ui.pages.rote_karte_muss_runter", e), pn), Theme::problem()),
                    4 => (crate::tf!(&ticker_key("ui.pages.verletzt_sich", e), pn), Theme::problem()),
                    5 => (crate::tf!(&ticker_key("ui.pages.wechsel_bei_fuer", e), c.world.clubs[team].short, e.pid2.map(|p| c.world.players[p].name.clone()).unwrap_or_default(), pn), Theme::text2()),
                    _ => (crate::tf!(&ticker_key("ui.pages.chance_fuer_scheitert", e), c.world.clubs[team].short, pn), Theme::text2()),
                };
                // every line in the colour of its club (from the crest), so the ticker reads at a glance
                let (tc, _) = club_color(c, team);
                rrect(r.x + 8.0, y - 20.0, r.w - 16.0, 28.0, 6.0, alpha(tc, 0.10));
                draw_rectangle(r.x + 8.0, y - 20.0, 4.0, 28.0, tc);
                ui.text_semi(&format!("{}'", e.min), r.x + 16.0, y, 18.0, tc);
                ui.player_photo(r.x + 72.0, y - 6.0, 11.0, c.world.players[e.pid].id, &pn);
                let t = ui.fit(&txt, 18.0, r.w - 104.0);
                ui.text(&t, r.x + 90.0, y, 18.0, col);
                y += 30.0;
            }
            let r2 = Rect::new(body.x + body.w * 0.62 + 12.0, body.y, body.w * 0.38 - 12.0, body.h);
            ui.card_titled(r2, crate::inhalt::t("bericht.achse.tempo"), Theme::akzent());
            // pause (Fabian 23.11.): the clock stands, grades and interventions stay usable; P toggles it too
            let sp = [0.0, 1.0, 2.0, 4.0, 12.0];
            if is_key_pressed(KeyCode::P) && mv.minute < 90.0 { if mv.speed == 0.0 { mv.speed = mv.vor_pause.max(1.0); } else { mv.vor_pause = mv.speed; mv.speed = 0.0; } }
            let sel = sp.iter().position(|s| *s == mv.speed).unwrap_or(1);
            if let Some(s) = ui.segment(Rect::new(r2.x + 14.0, r2.y + 44.0, r2.w - 28.0, 34.0), &[crate::inhalt::t("ui.live.pause"), crate::inhalt::t("programm.intensitaet.normal"), crate::inhalt::t("ui.pages.schnell"), crate::inhalt::t("ui.pages.sehr_schnell"), crate::inhalt::t("ui.live.turbo")], sel) { if sp[s] == 0.0 { mv.vor_pause = mv.speed; } mv.speed = sp[s]; }
            if mv.speed == 0.0 && mv.minute < 90.0 { ui.text_right(crate::inhalt::t("ui.live.pausiert"), r.x + r.w - 16.0, r.y + 30.0, 14.0, hex(0xffffff)); }
            if mv.halbzeit == 1 {
                let (gf, ga) = c.halbzeit_tore(&mv.res);
                if let Some(wahl) = halbzeit_karte(ui, Rect::new(r.x + 16.0, r.y + 52.0, r.w - 32.0, 300.0), gf, ga) {
                    if let Some(k) = wahl {
                        let t = c.halbzeit_antwort(Career::halbzeit_stand(gf, ga), &k);
                        mv.wirkung.push(t);
                        // the talk acts on the second half: a talk that landed pushes the own side, one that missed slows it
                        let side = if mv.res.h == c.own { 0u8 } else { 1 };
                        let w = (c.hz_wirkung / 2.0 * 0.03).clamp(-0.08, 0.08);
                        let (fe, fg) = match mv.eingriff.mentalitaet { -1 => (0.85, 0.88), 1 => (1.12, 1.10), _ => (1.0, 1.0) };
                        let elf = aktuelle_elf(&mv, side);
                        c.world.rest_neu(&mut mv.res, 45, side, &elf, fe * (1.0 + w), fg * (1.0 - w / 2.0));
                    }
                    mv.halbzeit = 2;
                }
            }
            if ui.button(Rect::new(r2.x + 14.0, r2.y + 90.0, r2.w - 28.0, 40.0), crate::inhalt::t("ui.pages.zum_abpfiff_springen"), false) { mv.minute = 91.0; mv.halbzeit = 2; }
            ui.bar(r2.x + 14.0, r2.y + 140.0, r2.w - 28.0, 6.0, mv.minute / 90.0, Theme::akzent());
            // three menus on the right (Fabian 24.11.): board and interventions, the live table, the other matches
            let reiter = [crate::inhalt::t("ui.live.reiter.eingriff"), crate::inhalt::t("ui.live.reiter.tabelle"), crate::inhalt::t("ui.live.reiter.parallel")];
            if let Some(i) = ui.segment(Rect::new(r2.x + 14.0, r2.y + 154.0, r2.w - 28.0, 30.0), &reiter, mv.eingriff.reiter as usize) { mv.eingriff.reiter = i as u8; }
            let unten = Rect::new(r2.x + 14.0, r2.y + 194.0, r2.w - 28.0, r2.h - 206.0);
            match mv.eingriff.reiter {
                1 => live_tabelle(ui, c, &mv, unten),
                2 => parallel_liste(ui, c, &mv, unten),
                _ => {
                    // the live board and the interventions (Fabian 23.11.)
                    let bh = (unten.w * 0.62).min(240.0).min(unten.h * 0.3);
                    live_board(ui, c, &mv, Rect::new(unten.x, unten.y, unten.w, bh));
                    if mv.minute < 90.0 { eingriffe(ui, v, c, &mut mv, Rect::new(unten.x, unten.y + 10.0 + bh, unten.w, unten.h - 10.0 - bh)); }
                }
            }
            // a goal: a big flash over the ticker in the scoring club's colour (Fabian 23.11.: more animation)
            let tore = mv.res.events.iter().filter(|e| e.kind <= 1 && (e.min as f32) <= mv.minute).count();
            if tore > mv.eingriff.tor_gesehen { mv.eingriff.tor_gesehen = tore; mv.eingriff.tor_t = get_time() as f32; }
            let seit = get_time() as f32 - mv.eingriff.tor_t;
            if mv.eingriff.tor_t > 0.0 && seit < 1.6 {
                if let Some(e) = mv.res.events.iter().rev().find(|e| e.kind <= 1 && (e.min as f32) <= mv.minute) {
                    let team = if e.side == 0 { h } else { a };
                    let (tc, _) = club_color(c, team);
                    let k = (seit / 0.25).min(1.0);
                    let aus = if seit > 1.2 { (1.6 - seit) / 0.4 } else { 1.0 };
                    let hgt = 150.0 * k;
                    let fr = Rect::new(body.x, body.y + body.h * 0.35 - hgt / 2.0, body.w * 0.62, hgt);
                    draw_rectangle(fr.x, fr.y, fr.w, fr.h, alpha(tc, 0.92 * aus));
                    let txt = crate::inhalt::t("tk.tor");
                    let gr = 40.0 + 50.0 * k;
                    ui.title(txt, fr.x + fr.w / 2.0 - ui.measure_title(txt, gr) / 2.0, fr.y + fr.h / 2.0 + gr * 0.2, gr, alpha(hex(0xffffff), aus));
                    let pn = c.world.players[e.pid].name.clone();
                    ui.text_center(&pn, fr.x + fr.w / 2.0, fr.y + fr.h / 2.0 + gr * 0.2 + 34.0, 22.0, alpha(hex(0xffffff), aus * k));
                }
            }
            // the walk-out: both crests come in from the sides, the competition between them, then kick-off
            if im_intro { match_intro(ui, c, &mv, body, t_intro); }
            if mv.minute >= 91.0 {
                mv.phase = 2;
                mv.ended_at = get_time() as f32;
                let res = mv.res.clone();
                c.apply_own_match(res);
                if let Some(r) = c.last_result.clone() { mv.res = r; } // with extra time and penalties of a cup match
                ui.play(crate::id("abpfiff"));
                let (eig, geg) = if mv.res.h == c.own { (mv.res.hg, mv.res.ag) } else { (mv.res.ag, mv.res.hg) };
                if eig > geg { ui.play(crate::id("sieg")); } else if eig < geg { ui.play(crate::id("niederlage")); }
            }
        }
        _ => {
            // after the match (Fabian 24.11.): the report with the talks, and the matchday in the look of the duel
            let schritte = [crate::inhalt::t("ui.nach.bericht"), crate::inhalt::t("ui.nach.spieltag")];
            for (k, s) in schritte.iter().enumerate() {
                let x = body.x + k as f32 * 170.0;
                let an = k == mv.eingriff.nach_reiter as usize;
                rrect(x, body.y, 160.0, 28.0, 14.0, if an { Theme::akzent() } else { Theme::erhoehung() });
                ui.text_center(s, x + 80.0, body.y + 19.0, 14.0, if an { Theme::auf_akzent() } else { Theme::text2() });
                if ui.hit(Rect::new(x, body.y, 160.0, 28.0)) { mv.eingriff.nach_reiter = k as u8; }
            }
            let body = Rect::new(body.x, body.y + 40.0, body.w, body.h - 40.0);
            if mv.eingriff.nach_reiter == 1 {
                if mv.eingriff.nach_stimmen.is_empty() { mv.eingriff.nach_stimmen = c.nach_stimmen(&mv.res); }
                nach_spieltag(ui, v, c, &mv, body);
                v.mv = Some(mv);
                return;
            }
            match_report(ui, c, &mv.res, Rect::new(body.x, body.y, body.w * 0.66, body.h));
            // guide §3.4: a short win banner with confetti in the club colour
            let won = if mv.res.h == c.own { mv.res.hg > mv.res.ag } else { mv.res.ag > mv.res.hg };
            let age = get_time() as f32 - mv.ended_at;
            if won && age < 1.6 {
                let a = (1.6 - age).min(1.0);
                for k in 0..80 {
                    let fx = ((k * 7919) % 1000) as f32 / 1000.0;
                    let fy = ((k * 104729) % 1000) as f32 / 1000.0;
                    let x = x0() + fx * cbreite() + (age * 3.0 + k as f32).sin() * 12.0;
                    let y = 200.0 + (fy * 120.0 + age * 380.0) % 700.0;
                    let col = if k % 3 == 0 { hex(0xffffff) } else if k % 3 == 1 { Theme::verein() } else { Theme::gold() };
                    draw_rectangle(x, y, 7.0, 4.0, alpha(col, a));
                }
                let r = Rect::new(x0() + cbreite() / 2.0 - 220.0, 330.0, 440.0, 90.0);
                rrect(r.x, r.y, r.w, r.h, 18.0, alpha(Theme::verein(), a));
                let t = crate::id("SIEG!");
                let w = measure_text(t, Some(&ui.head), 64, 1.0).width;
                ui.title(t, r.x + r.w / 2.0 - w / 2.0, r.y + 66.0, 64.0, alpha(Theme::verein_auf(), a));
            }
            let r2 = Rect::new(body.x + body.w * 0.66 + 12.0, body.y, body.w * 0.34 - 12.0, body.h);
            if c.press.is_some() { press_card(ui, v, c, r2); } else {
                ui.card_titled(r2, crate::inhalt::t("ux.abschnitt.nach"), Theme::akzent());
                let mut y = ui.paragraph(&mv.res.reason, r2.x + 14.0, r2.y + 60.0, 15.0, r2.w - 28.0, Theme::text()) + 8.0;
                // the talk after the match: the tone, and what it brought (Fabian 23.11.)
                let (gf, ga) = if mv.res.h == c.own { (mv.res.hg, mv.res.ag) } else { (mv.res.ag, mv.res.hg) };
                ui.text_semi(crate::inhalt::t("ans.nach_titel"), r2.x + 14.0, y + 14.0, 15.0, Theme::akzent());
                y += 24.0;
                if !mv.nach_gesagt {
                    let toene = [("lob", "sp.ton.nach.lob"), ("warm", "sp.ton.nach.warm"), ("sachlich", "sp.ton.nach.sachlich"), ("bestimmt", "sp.ton.nach.bestimmt"), ("hart", "sp.ton.nach.hart")]; // roh
                    let namen: Vec<&str> = toene.iter().map(|x| crate::inhalt::t(x.1)).collect();
                    if let Some(k) = ui.segment(Rect::new(r2.x + 14.0, y, r2.w - 28.0, 30.0), &namen, 99) {
                        let t = c.halbzeit_antwort(Career::halbzeit_stand(gf as u8, ga as u8), toene[k].0);
                        mv.wirkung.push(t);
                        mv.nach_gesagt = true;
                    }
                    y += 40.0;
                }
                for w in mv.wirkung.iter().rev().take(2) { let n = ui.text_passend(w, r2.x + 14.0, y + 14.0, 14.0, r2.w - 28.0, 2, Theme::text2()) as f32; y += 18.0 * n.max(1.0) + 6.0; }
                // single players: the best and the weakest of ours, praise or criticism
                let eigen: Vec<(usize, f32)> = { let mut v: Vec<(usize, f32)> = mv.res.ratings.iter().copied().filter(|r| c.world.players[r.0].team == c.own as i32).collect(); v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); let n = v.len(); if n > 4 { v.drain(2..n - 2); } v };
                ui.text_semi(crate::inhalt::t("ans.einzeln_titel"), r2.x + 14.0, y + 14.0, 15.0, Theme::akzent());
                y += 24.0;
                for (p, note) in eigen {
                    if y > r2.y + r2.h - 130.0 { break; }
                    let pl = &c.world.players[p];
                    ui.player_photo(r2.x + 26.0, y + 14.0, 11.0, pl.id, &pl.name);
                    ui.text(&ui.fit(&pl.name, 14.0, r2.w - 230.0), r2.x + 44.0, y + 19.0, 14.0, Theme::text());
                    ui.note_pill(r2.x + r2.w - 186.0, y + 13.0, note);
                    if !mv.angesprochen.contains(&p) {
                        if ui.button(Rect::new(r2.x + r2.w - 160.0, y + 2.0, 70.0, 24.0), crate::inhalt::t("ans.lob"), false) { let t = c.spieler_ansprechen(p, true, note); mv.wirkung.push(t); mv.angesprochen.push(p); }
                        if ui.button(Rect::new(r2.x + r2.w - 84.0, y + 2.0, 70.0, 24.0), crate::inhalt::t("ans.kritik"), false) { let t = c.spieler_ansprechen(p, false, note); mv.wirkung.push(t); mv.angesprochen.push(p); }
                    }
                    y += 30.0;
                }
                if ui.button_gold(Rect::new(r2.x + 14.0, r2.y + r2.h - 64.0, r2.w - 28.0, 48.0), crate::inhalt::t("ui.wort.gross_weiter")) { v.go(Page::Uebersicht); return; }
                if c.undo.is_some() && ui.button(Rect::new(r2.x + 14.0, r2.y + r2.h - 112.0, r2.w - 28.0, 38.0), crate::inhalt::t("undo.spiel"), false) { c.rueckgaengig(); v.mv = None; v.toast(crate::inhalt::t("undo.ok"), true); return; }
            }
        }
    }
    v.mv = Some(mv);
}

/// The matchday after our match (Fabian 24.11.): the table with its movement, the day's results, the players of the
/// match and of the matchday, and the voices on our result.
fn nach_spieltag(ui: &mut Ui, v: &mut View, c: &Career, mv: &MatchView, r: Rect) {
    let t = crate::inhalt::t;
    let l = c.league();
    let tag = c.world.day;
    // the places before today, to show who moved
    let mut z: std::collections::HashMap<usize, (i32, i32, i32)> = c.world.leagues[l].teams.iter().map(|&x| (x, (0, 0, 0))).collect();
    for f in c.world.leagues[l].fixtures.iter().filter(|f| f.hg >= 0 && f.day < tag) {
        for (x, g, gg) in [(f.h, f.hg as i32, f.ag as i32), (f.a, f.ag as i32, f.hg as i32)] { let e = z.entry(x).or_insert((0, 0, 0)); e.0 += if g > gg { 3 } else if g == gg { 1 } else { 0 }; e.1 += g - gg; e.2 += g; }
    }
    let mut vor: Vec<(usize, (i32, i32, i32))> = z.into_iter().collect();
    vor.sort_by(|a, b| b.1.0.cmp(&a.1.0).then(b.1.1.cmp(&a.1.1)).then(b.1.2.cmp(&a.1.2)));
    let platz_vor = |club: usize| vor.iter().position(|x| x.0 == club).map(|i| i + 1).unwrap_or(0) as i32;
    let sw = r.w * 0.34;
    // the table
    let tr = Rect::new(r.x, r.y, sw, r.h);
    ui.card_titled(tr, &c.world.leagues[l].name.clone(), Theme::akzent());
    let tab = c.world.table(l);
    let zh = ((tr.h - 60.0) / tab.len().max(1) as f32).clamp(14.0, 26.0);
    for (i, row) in tab.iter().enumerate() {
        let y = tr.y + 50.0 + i as f32 * zh;
        if y + zh > tr.y + tr.h { break; }
        if row.club == c.own { rrect(tr.x + 6.0, y, tr.w - 12.0, zh - 2.0, 4.0, alpha(Theme::akzent(), 0.15)); }
        ui.text(&format!("{}", i + 1), tr.x + 12.0, y + zh * 0.72, 12.0, Theme::text2());
        let d = platz_vor(row.club) - (i as i32 + 1);
        // moved up, down or stayed: a drawn arrow (the fonts have no arrow glyphs)
        let (ax, ay) = (tr.x + 36.0, y + zh * 0.45);
        if d > 0 { draw_triangle(vec2(ax - 5.0, ay + 3.0), vec2(ax + 5.0, ay + 3.0), vec2(ax, ay - 4.0), Theme::gut()); }
        else if d < 0 { draw_triangle(vec2(ax - 5.0, ay - 3.0), vec2(ax + 5.0, ay - 3.0), vec2(ax, ay + 4.0), Theme::problem()); }
        else { draw_circle(ax, ay, 2.0, Theme::gedaempft()); }
        ui.club_crest(tr.x + 56.0, y + zh * 0.45, zh * 0.36, c.world.clubs[row.club].id, club_color(c, row.club).0, &c.world.clubs[row.club].short);
        ui.text(&ui.fit(&c.world.clubs[row.club].name, 12.0, tr.w - 150.0), tr.x + 74.0, y + zh * 0.72, 12.0, Theme::text());
        ui.text_right(&format!("{:+}", row.gf - row.ga), tr.x + tr.w - 44.0, y + zh * 0.72, 12.0, Theme::text2());
        ui.text_right(&format!("{}", row.pts), tr.x + tr.w - 12.0, y + zh * 0.72, 12.0, Theme::text());
    }
    // the day's results
    let mr = Rect::new(r.x + sw + 12.0, r.y, r.w - 2.0 * sw - 24.0, r.h);
    ui.card_titled(mr, t("ui.nach.ergebnisse"), Theme::verein());
    let mut spiele: Vec<(usize, usize, &MatchResult)> = vec![(mv.res.h, mv.res.a, &mv.res)];
    spiele.extend(mv.parallel.iter().map(|(h, a, res)| (*h, *a, res)));
    for (i, (h, a, res)) in spiele.iter().enumerate() {
        let y = mr.y + 52.0 + i as f32 * 40.0;
        if y + 36.0 > mr.y + mr.h * 0.55 { break; }
        let unser = i == 0;
        rrect(mr.x + 8.0, y, mr.w - 16.0, 36.0, 8.0, if unser { alpha(Theme::akzent(), 0.14) } else { Theme::erhoehung() });
        ui.club_crest(mr.x + 26.0, y + 18.0, 11.0, c.world.clubs[*h].id, club_color(c, *h).0, &c.world.clubs[*h].short);
        ui.text(&ui.fit(&c.world.clubs[*h].name, 13.0, mr.w * 0.34), mr.x + 44.0, y + 23.0, 13.0, Theme::text());
        ui.title(&format!("{}:{}", res.hg, res.ag), mr.x + mr.w / 2.0 - 18.0, y + 27.0, 22.0, Theme::text());
        ui.text_right(&ui.fit(&c.world.clubs[*a].name, 13.0, mr.w * 0.34), mr.x + mr.w - 44.0, y + 23.0, 13.0, Theme::text());
        ui.club_crest(mr.x + mr.w - 26.0, y + 18.0, 11.0, c.world.clubs[*a].id, club_color(c, *a).0, &c.world.clubs[*a].short);
    }
    // the voices on our result
    let mut y = mr.y + mr.h * 0.58;
    ui.text_semi(t("ui.nach.stimmen"), mr.x + 14.0, y, 13.0, Theme::gedaempft());
    y += 20.0;
    for (k, s) in mv.eingriff.nach_stimmen.iter().enumerate() {
        if y > mr.y + mr.h - 40.0 { break; }
        ui.text_semi([t("ui.duell.tv"), t("ui.duell.presse_label"), t("ui.nach.fans")][k.min(2)], mr.x + 14.0, y, 12.0, Theme::akzent());
        let n = ui.text_passend(&crate::tf!("ui.duell.zitat", s.clone()), mr.x + 14.0, y + 18.0, 14.0, mr.w - 28.0, 3, Theme::text());
        y += 26.0 + 19.0 * n as f32;
    }
    // players of the match and of the matchday
    let pr = Rect::new(r.x + r.w - sw, r.y, sw, r.h);
    ui.card_titled(pr, t("ui.nach.spieler"), Theme::gold());
    let mut alle: Vec<(usize, f32)> = spiele.iter().flat_map(|x| x.2.ratings.iter().copied()).collect();
    alle.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let des_spiels = mv.res.ratings.iter().copied().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let mut y = pr.y + 52.0;
    if let Some((p, n)) = des_spiels {
        ui.text_semi(t("ui.nach.des_spiels"), pr.x + 14.0, y + 12.0, 13.0, Theme::gedaempft());
        let pl = &c.world.players[p];
        ui.player_photo(pr.x + 44.0, y + 52.0, 28.0, pl.id, &pl.name);
        ui.title(&ui.fit(&pl.name, 22.0, pr.w - 150.0), pr.x + 84.0, y + 50.0, 22.0, Theme::text());
        if pl.team >= 0 { ui.text(&c.world.clubs[pl.team as usize].name.clone(), pr.x + 84.0, y + 72.0, 13.0, Theme::text2()); }
        ui.note_pill(pr.x + pr.w - 50.0, y + 44.0, n);
        y += 104.0;
    }
    ui.text_semi(t("ui.nach.des_spieltags"), pr.x + 14.0, y + 12.0, 13.0, Theme::gedaempft());
    y += 24.0;
    for (p, n) in alle.iter().take(8) {
        if y > pr.y + pr.h - 40.0 { break; }
        let pl = &c.world.players[*p];
        ui.player_photo(pr.x + 28.0, y + 12.0, 12.0, pl.id, &pl.name);
        ui.pos_tag(pr.x + 46.0, y + 12.0, pos_de(&pl.pos));
        ui.text(&ui.fit(&pl.name, 13.0, pr.w - 170.0), pr.x + 90.0, y + 17.0, 13.0, Theme::text());
        if pl.team >= 0 { ui.club_crest(pr.x + pr.w - 74.0, y + 12.0, 9.0, c.world.clubs[pl.team as usize].id, club_color(c, pl.team as usize).0, &c.world.clubs[pl.team as usize].short); }
        ui.note_pill(pr.x + pr.w - 40.0, y + 6.0, *n);
        y += 30.0;
    }
    if ui.button_gold(Rect::new(pr.x + 14.0, pr.y + pr.h - 58.0, pr.w - 28.0, 44.0), t("ui.wort.gross_weiter")) { v.go(Page::Uebersicht); }
}

fn match_report(ui: &mut Ui, c: &Career, res: &MatchResult, r: Rect) {
    ui.card_titled(r, crate::inhalt::t("bericht.titel"), Theme::akzent());
    let mut y = r.y + 54.0;
    if res.et { ui.text_center(&crate::tf!("ui.pages.nach_verlaengerung", res.pens.map(|p| crate::tf!("ui.pages.komma_ie", crate::cups::pens_score(p))).unwrap_or_default()), r.x + r.w / 2.0, r.y + 22.0, 15.0, Theme::gold()); }
    // expected goals from the players' shots (statistik.rs), when the match has statistics
    if !res.stat.is_empty() {
        let (xh, xa) = (res.summe(0, crate::statistik::XG) as f32 / 1000.0, res.summe(1, crate::statistik::XG) as f32 / 1000.0);
        ui.text_center(crate::inhalt::t("ui.stat.xg"), r.x + r.w / 2.0, y, 15.0, Theme::text2());
        ui.text(&format!("{xh:.2}"), r.x + 16.0, y, 17.0, Theme::text());
        ui.text_right(&format!("{xa:.2}"), r.x + r.w - 16.0, y, 17.0, Theme::text());
        let bw = r.w / 2.0 - 90.0;
        let tot = (xh + xa).max(0.01);
        rrect(r.x + r.w / 2.0 - 70.0 - bw * xh / tot, y + 6.0, bw * xh / tot, 6.0, 3.0, Theme::akzent());
        rrect(r.x + r.w / 2.0 + 70.0, y + 6.0, bw * xa / tot, 6.0, 3.0, Theme::problem());
        y += 22.0;
        // the xG over the match in 18 sections of five minutes, both sides cumulated, goals as dots
        let (gx, gw, gh) = (r.x + 16.0, r.w - 32.0, 46.0);
        let kum = |side: u8| -> Vec<f32> { let mut a = 0.0; (0..=18).map(|k| { if k > 0 { a += res.schuesse.iter().filter(|x| x.side == side && ((x.min as usize).saturating_sub(1) / 5).min(17) == k - 1).map(|x| x.xg as f32 / 1000.0).sum::<f32>(); } a }).collect() };
        let (kh, ka) = (kum(0), kum(1));
        let top = kh.last().copied().unwrap_or(0.0).max(ka.last().copied().unwrap_or(0.0)).max(1.0);
        draw_line(gx, y + gh, gx + gw, y + gh, 1.0, alpha(Theme::text(), 0.15));
        draw_line(gx + gw / 2.0, y, gx + gw / 2.0, y + gh, 1.0, alpha(Theme::text(), 0.1));
        for (v, col) in [(&kh, Theme::akzent()), (&ka, Theme::problem())] {
            for k in 0..18 { draw_line(gx + gw * k as f32 / 18.0, y + gh - gh * v[k] / top, gx + gw * (k + 1) as f32 / 18.0, y + gh - gh * v[k + 1] / top, 2.0, col); }
        }
        for x in res.schuesse.iter().filter(|x| x.aus == crate::statistik::TOR) {
            let v = if x.side == 0 { &kh } else { &ka };
            let k = ((x.min as usize).saturating_sub(1) / 5).min(17) + 1;
            draw_circle(gx + gw * k as f32 / 18.0, y + gh - gh * v[k] / top, 4.0, if x.side == 0 { Theme::akzent() } else { Theme::problem() });
        }
        y += gh + 18.0;
    }
    // did the tactic work? (Konzept Taktik 5.3)
    for (z, ok) in c.taktik_bilanz(res) {
        ui.text(if ok { "+" } else { "–" }, r.x + 16.0, y + 4.0, 16.0, if ok { Theme::gut() } else { Theme::problem() });
        ui.text(&ui.fit(&z, 14.0, r.w - 50.0), r.x + 32.0, y + 4.0, 14.0, Theme::text2());
        y += 20.0;
    }
    if !res.stat.is_empty() { y += 14.0; }
    for (label, hv, av) in [(crate::inhalt::t("beob.stat.schuesse"), res.shots.0 as f32, res.shots.1 as f32), (crate::inhalt::t("ui.pages.aufs_tor"), res.on_target.0 as f32, res.on_target.1 as f32), (crate::inhalt::t("ui.pages.ballbesitz"), res.possession as f32, 100.0 - res.possession as f32)] {
        ui.text_center(label, r.x + r.w / 2.0, y, 15.0, Theme::text2());
        ui.text(&format!("{hv:.0}"), r.x + 16.0, y, 17.0, Theme::text());
        ui.text_right(&format!("{av:.0}"), r.x + r.w - 16.0, y, 17.0, Theme::text());
        let tot = (hv + av).max(1.0);
        let bw = r.w / 2.0 - 90.0;
        rrect(r.x + r.w / 2.0 - 70.0 - bw * hv / tot, y + 6.0, bw * hv / tot, 6.0, 3.0, Theme::akzent());
        rrect(r.x + r.w / 2.0 + 70.0, y + 6.0, bw * av / tot, 6.0, 3.0, Theme::problem());
        y += 34.0;
    }
    let goals: Vec<String> = res.events.iter().filter(|e| e.kind <= 1).map(|e| format!("{}' {}{}", e.min, c.world.players[e.pid].name, if e.kind == 1 { crate::inhalt::t("ui.pages.et") } else { "" })).collect();
    ui.text(&crate::tf!("ui.pages.tore", if goals.is_empty() { crate::inhalt::t("ui.wort.keine").into() } else { goals.join(", ") }), r.x + 16.0, y + 6.0, 15.0, Theme::text2());
    y += 30.0;
    ui.text_semi(crate::inhalt::t("bericht.noten"), r.x + 16.0, y + 8.0, 16.0, Theme::text());
    let col_w = r.w / 2.0 - 20.0;
    for (k, side) in [res.h, res.a].iter().enumerate() {
        let mut rs: Vec<(usize, f32)> = res.ratings.iter().copied().filter(|(p, _)| { let t = c.world.players[*p].team; t == *side as i32 || res.minutes.iter().any(|m| m.0 == *p) && (if k == 0 { res.xi_h.iter().any(|x| x.1 == *p) } else { res.xi_a.iter().any(|x| x.1 == *p) }) }).collect();
        rs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rs.dedup_by_key(|x| x.0);
        let x = r.x + 16.0 + k as f32 * (col_w + 10.0);
        ui.text_semi(&c.world.clubs[*side].short, x, y + 34.0, 15.0, Theme::akzent());
        for (j, (p, rt)) in rs.iter().take(14).enumerate() {
            let yy = y + 58.0 + j as f32 * 25.0;
            if yy > r.y + r.h - 10.0 { break; }
            ui.player_photo(x + 9.0, yy - 5.0, 9.0, c.world.players[*p].id, &c.world.players[*p].name);
            let nm = ui.fit(&c.world.players[*p].name, 15.0, col_w - 92.0);
            ui.text(&nm, x + 22.0, yy, 15.0, Theme::text());
            ui.note_pill(x + col_w - 30.0, yy - 5.0, *rt);
        }
    }
}

/// The half-time talk as a card over the ticker or the live view: the score at the break and the tones (keys 1–5).
/// Returns Some(Some(tone)) for a talk, Some(None) for none, None while waiting.
fn halbzeit_karte(ui: &mut Ui, r: Rect, gf: u8, ga: u8) -> Option<Option<String>> {
    let opts = Career::halbzeit_optionen();
    let r = Rect::new(r.x, r.y, r.w, 92.0 + opts.len() as f32 * 44.0 + 50.0);
    rrect(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, 12.0, Theme::akzent());
    rrect(r.x, r.y, r.w, r.h, 11.0, Theme::flaeche());
    ui.title(&crate::tf!("ui.rest3.halbzeit_beim", gf, ga), r.x + 18.0, r.y + 36.0, 26.0, Theme::text());
    ui.text(crate::inhalt::t("ui.halbzeit.frage"), r.x + 18.0, r.y + 62.0, 16.0, Theme::text2());
    let tasten = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5];
    for (i, (k, w)) in opts.iter().enumerate() {
        let b = Rect::new(r.x + 18.0, r.y + 80.0 + i as f32 * 44.0, r.w - 36.0, 38.0);
        if ui.button(b, &format!("{}  {w}", i + 1), false) || is_key_pressed(tasten[i.min(4)]) { return Some(Some(k.clone())); }
    }
    if ui.button(Rect::new(r.x + 18.0, r.y + r.h - 46.0, r.w - 36.0, 34.0), crate::inhalt::t("ui.halbzeit.ohne"), false) { return Some(None); }
    None
}

/// The half-time talk in the FL27 live view: from minute 45 of the first half, once per match.
fn halbzeit_fl27(ui: &mut Ui, v: &mut View, c: &mut Career, h: usize, a: usize, r: Rect) {
    let Some(m) = v.fl27_ansicht.clone() else { return };
    let key = (c.world.day, h, a);
    if v.halbzeit_fl27 == Some(key) || m.minute < 45 || m.period > 3 { return; }
    let (gf, ga) = if h == c.own { (m.score[0], m.score[1]) } else { (m.score[1], m.score[0]) };
    let k = Rect::new(r.x + r.w / 2.0 - 320.0, r.y + 140.0, 640.0, 300.0);
    if let Some(wahl) = halbzeit_karte(ui, k, gf as u8, ga as u8) {
        if let Some(t) = wahl { let txt = c.halbzeit_antwort(Career::halbzeit_stand(gf as u8, ga as u8), &t); v.toast(&txt, true); }
        v.halbzeit_fl27 = Some(key);
    }
}
