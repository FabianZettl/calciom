use super::*;

pub(super) const INTRO_DAUER: f32 = 2.6;
/// The cut-scene before kick-off: stadium in the dark, both crests slide in, "VS", the competition, then "Anpfiff!".
pub(super) fn match_intro(ui: &mut Ui, c: &Career, mv: &MatchView, r: Rect, t: f32) {
    let aus = if t > INTRO_DAUER - 0.4 { (INTRO_DAUER - t) / 0.4 } else { 1.0 };
    draw_rectangle(r.x, r.y, r.w, r.h, alpha(hex(0x0b0f17), 0.94 * aus));
    let (h, a) = (mv.res.h, mv.res.a);
    let ein = |d: f32| { let x = ((t - d) / 0.5).clamp(0.0, 1.0); 1.0 - (1.0 - x).powi(3) };
    let (ch, _) = club_color(c, h);
    let (ca, _) = club_color(c, a);
    let my = r.y + r.h * 0.42;
    let lx = r.x + r.w * 0.27 - (1.0 - ein(0.1)) * r.w * 0.4;
    let rx = r.x + r.w * 0.73 + (1.0 - ein(0.25)) * r.w * 0.4;
    draw_circle(lx, my, 86.0, alpha(ch, 0.35 * aus));
    draw_circle(rx, my, 86.0, alpha(ca, 0.35 * aus));
    ui.club_crest(lx, my, 64.0, c.world.clubs[h].id, ch, &c.world.clubs[h].short);
    ui.club_crest(rx, my, 64.0, c.world.clubs[a].id, ca, &c.world.clubs[a].short);
    let wcol = alpha(hex(0xffffff), aus);
    for (n, x) in [(&c.world.clubs[h].name, lx), (&c.world.clubs[a].name, rx)] { ui.text_center(n, x, my + 110.0, 24.0, alpha(hex(0xffffff), aus * ein(0.5))); }
    let vs = crate::id("VS");
    let g = 60.0 * ein(0.6);
    if g > 1.0 { ui.title(vs, r.x + r.w / 2.0 - ui.measure_title(vs, g) / 2.0, my + g * 0.35, g, wcol); }
    if t > 1.5 {
        let k = ((t - 1.5) / 0.3).clamp(0.0, 1.0);
        let anpfiff = crate::inhalt::t("gl.anpfiff");
        let gr = 30.0 + 20.0 * k;
        ui.title(anpfiff, r.x + r.w / 2.0 - ui.measure_title(anpfiff, gr) / 2.0, r.y + r.h * 0.82, gr, alpha(Theme::gold(), aus * k));
    }
}

/// The own eleven on the pitch at the shown minute: starting eleven, substitutions done, red cards gone.
pub(super) fn aktuelle_elf(mv: &MatchView, side: u8) -> Vec<usize> {
    let mut elf: Vec<usize> = (if side == 0 { &mv.res.xi_h } else { &mv.res.xi_a }).iter().map(|x| x.1).collect();
    for e in mv.res.events.iter().filter(|e| e.side == side && (e.min as f32) <= mv.minute) {
        match e.kind { 5 => { if let (Some(i), Some(on)) = (elf.iter().position(|&p| p == e.pid), e.pid2) { elf[i] = on; } } 3 => elf.retain(|&p| p != e.pid), _ => {} }
    }
    elf
}

/// Where a position stands on the board (0..1 along the pitch from the own goal, 0..1 across).
fn brett_platz(pos: &str) -> (f32, f32) {
    match pos { "GK" => (0.05, 0.5), "CB" => (0.2, 0.5), "LB" => (0.25, 0.12), "RB" => (0.25, 0.88), "DMF" => (0.36, 0.5), "CMF" => (0.46, 0.5), "LMF" => (0.5, 0.14), "RMF" => (0.5, 0.86),
        "AMF" => (0.6, 0.5), "LWF" => (0.7, 0.16), "RWF" => (0.7, 0.84), "SS" => (0.72, 0.5), _ => (0.8, 0.5) }
}

/// The live board: both elevens move with the play, the ball goes where the next chance or goal comes from.
pub(super) fn live_board(ui: &mut Ui, c: &Career, mv: &MatchView, r: Rect) {
    rrect(r.x, r.y, r.w, r.h, 8.0, hex(0x2E7D32));
    for k in 0..8 { if k % 2 == 0 { draw_rectangle(r.x + k as f32 * r.w / 8.0, r.y, r.w / 8.0, r.h, alpha(hex(0xffffff), 0.04)); } }
    let lc = alpha(hex(0xffffff), 0.7);
    draw_rectangle_lines(r.x + 4.0, r.y + 4.0, r.w - 8.0, r.h - 8.0, 1.5, lc);
    draw_line(r.x + r.w / 2.0, r.y + 4.0, r.x + r.w / 2.0, r.y + r.h - 4.0, 1.5, lc);
    draw_circle_lines(r.x + r.w / 2.0, r.y + r.h / 2.0, r.h * 0.14, 1.5, lc);
    for gx in [r.x + 4.0, r.x + r.w - 4.0 - r.w * 0.14] { draw_rectangle_lines(gx, r.y + r.h * 0.25, r.w * 0.14, r.h * 0.5, 1.5, lc); }
    let t = get_time() as f32;
    let m = mv.minute;
    // who attacks: the side of the next goal or chance within three minutes, else the possession with a slow swing
    let naechstes = mv.res.events.iter().find(|e| matches!(e.kind, 0 | 1 | 6) && (e.min as f32) > m && (e.min as f32) <= m + 3.0);
    let ball_heim = (mv.res.possession.max(1) as f32 / 100.0 - 0.5) + 0.35 * (m * 0.9 + t * 0.35).sin();
    let angriff = match naechstes { Some(e) => if e.side == 0 { 1.0 } else { -1.0 }, None => ball_heim.clamp(-1.0, 1.0) };
    let naehe = naechstes.map_or(0.0, |e| 1.0 - ((e.min as f32 - m) / 3.0).clamp(0.0, 1.0));
    for side in 0..2u8 {
        let xi = if side == 0 { &mv.res.xi_h } else { &mv.res.xi_a };
        let elf = aktuelle_elf(mv, side);
        let club = if side == 0 { mv.res.h } else { mv.res.a };
        let (mut col, c2) = club_color(c, club);
        // the away side takes its second colour (or white) when both shirts look alike
        if side == 1 { let (h1, _) = club_color(c, mv.res.h); if (h1.r - col.r).abs() + (h1.g - col.g).abs() + (h1.b - col.b).abs() < 0.45 { col = if (h1.r - c2.r).abs() + (h1.g - c2.g).abs() + (h1.b - c2.b).abs() >= 0.45 { c2 } else { hex(0xffffff) }; } }
        let dir = if side == 0 { 1.0 } else { -1.0 };
        let vor = angriff * dir;
        for (i, &p) in elf.iter().enumerate() {
            let pos = xi.iter().find(|x| x.1 == p).map(|x| x.0.clone()).unwrap_or_else(|| c.world.players[p].pos.clone());
            let gleich: Vec<usize> = elf.iter().copied().filter(|&q| xi.iter().find(|x| x.1 == q).map(|x| x.0.clone()).unwrap_or_else(|| c.world.players[q].pos.clone()) == pos).collect();
            let k = gleich.iter().position(|&q| q == p).unwrap_or(0) as f32;
            let n = gleich.len() as f32;
            let (mut px, mut py) = brett_platz(&pos);
            if n > 1.0 { py = (py - 0.18 * (n - 1.0) / 2.0 + 0.18 * k).clamp(0.08, 0.92); }
            if pos != "GK" { px += 0.09 * vor + 0.012 * (t * 1.3 + i as f32 * 1.7).sin(); py += 0.02 * (t * 0.9 + i as f32 * 2.3).cos(); }
            let x = if side == 0 { px } else { 1.0 - px };
            let (sx, sy) = (r.x + 8.0 + x * (r.w - 16.0), r.y + 8.0 + py.clamp(0.04, 0.96) * (r.h - 16.0));
            draw_circle(sx, sy, 7.5, col);
            draw_circle_lines(sx, sy, 7.5, 1.5, if club == c.own { Theme::gold() } else { hex(0x1a1a1a) });
            let _ = p;
        }
    }
    // the ball
    let bx = 0.5 + angriff * (0.18 + 0.22 * naehe) + 0.05 * (t * 1.9).sin();
    let by = 0.5 + 0.28 * (t * 0.7 + m).sin() * (1.0 - naehe * 0.7);
    draw_circle(r.x + 8.0 + bx.clamp(0.03, 0.97) * (r.w - 16.0), r.y + 8.0 + by * (r.h - 16.0), 4.0, hex(0xffffff));
    // a goal in the last minute and a half: big on the board
    if let Some(e) = mv.res.events.iter().rev().find(|e| e.kind <= 1 && (e.min as f32) <= m && m - (e.min as f32) < 1.5) {
        let club = if e.side == 0 { mv.res.h } else { mv.res.a };
        let (col, _) = club_color(c, club);
        let a = 0.6 + 0.4 * (t * 8.0).sin().abs();
        rrect(r.x + r.w / 2.0 - 70.0, r.y + r.h / 2.0 - 26.0, 140.0, 52.0, 12.0, alpha(col, a));
        let txt = crate::inhalt::t("tk.tor");
        ui.title(txt, r.x + r.w / 2.0 - ui.measure_title(txt, 34.0) / 2.0, r.y + r.h / 2.0 + 14.0, 34.0, hex(0xffffff));
    }
}

/// Interventions: mentality, a shout, a substitution; each draws the rest of the match again from this minute (world.rs rest_neu).
/// A player's grade as it stands at `minute`: the events so far count at once, the rest of the final grade grows with the
/// clock, so at the final whistle it is the match grade.
pub fn live_note(res: &MatchResult, p: usize, minute: f32) -> f32 {
    let Some(&(_, ende)) = res.ratings.iter().find(|x| x.0 == p) else { return 6.0 };
    let wert = |e: &MatchEvent| -> f32 { match e.kind { 0 if e.pid == p => 0.9, 0 if e.pid2 == Some(p) => 0.5, 1 if e.pid == p => -1.2, 2 if e.pid == p => -0.2, 3 if e.pid == p => -1.4, _ => 0.0 } };
    let bisher: f32 = res.events.iter().filter(|e| e.min as f32 <= minute).map(wert).sum();
    let alle: f32 = res.events.iter().map(wert).sum();
    let anteil = (minute / 90.0).clamp(0.0, 1.0);
    (6.3 + bisher + (ende - 6.3 - alle) * anteil).clamp(3.0, 10.0)
}
/// Stamina in the match: fitness at kick-off minus what his minutes on the pitch so far have cost.
pub fn live_kraft(res: &MatchResult, side: u8, p: usize, minute: f32, fit: f32) -> f32 {
    let rein = res.events.iter().find(|e| e.side == side && e.kind == 5 && e.pid2 == Some(p) && e.min as f32 <= minute).map_or(0.0, |e| e.min as f32);
    (fit - crate::world::FATIGUE_PER_MATCH * 1.4 * (minute.min(90.0) - rein).max(0.0) / 90.0).clamp(0.0, 100.0)
}
/// The live table of our league: points before today, plus today's matches at the score of this minute (Fabian 24.11.).
pub(super) fn live_tabelle(ui: &mut Ui, c: &Career, mv: &MatchView, r: Rect) {
    let l = c.league();
    let tag = c.world.day;
    let mut z: std::collections::HashMap<usize, (i32, i32, i32)> = c.world.leagues[l].teams.iter().map(|&t| (t, (0, 0, 0))).collect();
    let mut buchen = |h: usize, a: usize, hg: i32, ag: i32| {
        for (t, f, g) in [(h, hg, ag), (a, ag, hg)] { let e = z.entry(t).or_insert((0, 0, 0)); e.0 += if f > g { 3 } else if f == g { 1 } else { 0 }; e.1 += f - g; e.2 += f; }
    };
    for f in c.world.leagues[l].fixtures.iter().filter(|f| f.hg >= 0 && f.day < tag) { buchen(f.h, f.a, f.hg as i32, f.ag as i32); }
    let stand = |res: &MatchResult| { let t = |s: u8| res.events.iter().filter(|e| (e.min as f32) <= mv.minute && ((e.kind == 0 && e.side == s) || (e.kind == 1 && e.side != s))).count() as i32; (t(0), t(1)) };
    let heute: Vec<(usize, usize, (i32, i32))> = mv.parallel.iter().map(|(h, a, res)| (*h, *a, stand(res))).chain(std::iter::once((mv.res.h, mv.res.a, stand(&mv.res)))).collect();
    for (h, a, (hg, ag)) in &heute { if c.world.leagues[l].teams.contains(h) { buchen(*h, *a, *hg, *ag); } }
    let mut rang: Vec<(usize, (i32, i32, i32))> = z.into_iter().collect();
    rang.sort_by(|a, b| b.1.0.cmp(&a.1.0).then(b.1.1.cmp(&a.1.1)).then(b.1.2.cmp(&a.1.2)));
    let zh = ((r.h - 10.0) / rang.len().max(1) as f32).clamp(14.0, 24.0);
    for (i, (t, (pkt, td, _))) in rang.iter().enumerate() {
        let y = r.y + i as f32 * zh;
        if y + zh > r.y + r.h { break; }
        let spielt = heute.iter().any(|x| x.0 == *t || x.1 == *t);
        if *t == c.own { rrect(r.x, y, r.w, zh - 2.0, 4.0, alpha(Theme::akzent(), 0.15)); }
        ui.text(&format!("{}", i + 1), r.x + 6.0, y + zh * 0.72, 12.0, Theme::text2());
        ui.club_crest(r.x + 34.0, y + zh * 0.45, zh * 0.36, c.world.clubs[*t].id, club_color(c, *t).0, &c.world.clubs[*t].short);
        ui.text(&ui.fit(&c.world.clubs[*t].name, 12.0, r.w - 150.0), r.x + 52.0, y + zh * 0.72, 12.0, if spielt { Theme::text() } else { Theme::text2() });
        ui.text_right(&format!("{td:+}"), r.x + r.w - 44.0, y + zh * 0.72, 12.0, Theme::text2());
        ui.text_right(&format!("{pkt}"), r.x + r.w - 6.0, y + zh * 0.72, 12.0, Theme::text());
    }
}
/// The other matches of our league today, at the score of this minute, with the last scorer.
pub(super) fn parallel_liste(ui: &mut Ui, c: &Career, mv: &MatchView, r: Rect) {
    if mv.parallel.is_empty() { ui.paragraph(crate::inhalt::t("ui.live.keine_parallel"), r.x, r.y + 20.0, 14.0, r.w, Theme::text2()); return; }
    for (i, (h, a, res)) in mv.parallel.iter().enumerate() {
        let y = r.y + i as f32 * 52.0;
        if y + 48.0 > r.y + r.h { break; }
        let tore: Vec<&MatchEvent> = res.events.iter().filter(|e| e.kind <= 1 && (e.min as f32) <= mv.minute).collect();
        let hg = tore.iter().filter(|e| (e.kind == 0) == (e.side == 0)).count();
        let ag = tore.len() - hg;
        let frisch = tore.last().map_or(false, |e| mv.minute - e.min as f32 <= 3.0);
        rrect(r.x, y, r.w, 46.0, 8.0, if frisch { alpha(Theme::gut(), 0.18) } else { Theme::erhoehung() });
        ui.club_crest(r.x + 18.0, y + 16.0, 10.0, c.world.clubs[*h].id, club_color(c, *h).0, &c.world.clubs[*h].short);
        ui.text(&ui.fit(&c.world.clubs[*h].name, 13.0, r.w * 0.36), r.x + 34.0, y + 21.0, 13.0, Theme::text());
        ui.title(&format!("{hg}:{ag}"), r.x + r.w / 2.0 - 18.0, y + 26.0, 22.0, Theme::text());
        ui.text_right(&ui.fit(&c.world.clubs[*a].name, 13.0, r.w * 0.36), r.x + r.w - 34.0, y + 21.0, 13.0, Theme::text());
        ui.club_crest(r.x + r.w - 18.0, y + 16.0, 10.0, c.world.clubs[*a].id, club_color(c, *a).0, &c.world.clubs[*a].short);
        if let Some(e) = tore.last() { ui.text_center(&ui.fit(&format!("{}' {}", e.min, c.world.players[e.pid].name), 11.0, r.w - 20.0), r.x + r.w / 2.0, y + 41.0, 11.0, Theme::text2()); }
    }
}

pub(super) fn eingriffe(ui: &mut Ui, v: &mut View, c: &mut Career, mv: &mut MatchView, r: Rect) {
    let side = if mv.res.h == c.own { 0u8 } else if mv.res.a == c.own { 1 } else { return };
    let m = mv.minute.floor() as u8;
    let mut neu = false;
    let mut zuruf = (1.0f32, 1.0f32);
    ui.text_semi(crate::inhalt::t("ui.live.mentalitaet"), r.x, r.y + 14.0, 13.0, Theme::gedaempft());
    let opts = [crate::inhalt::t("ui.live.ment.0"), crate::inhalt::t("ui.live.ment.1"), crate::inhalt::t("ui.live.ment.2")];
    if let Some(i) = ui.segment(Rect::new(r.x, r.y + 20.0, r.w, 30.0), &opts, (mv.eingriff.mentalitaet + 1) as usize) {
        mv.eingriff.mentalitaet = i as i8 - 1;
        v.toast(&crate::tf!("ui.live.log.ment", m, opts[i]), true);
        neu = true;
    }
    ui.text_semi(crate::inhalt::t("ui.live.zurufe"), r.x, r.y + 70.0, 13.0, Theme::gedaempft());
    let frei = mv.minute - mv.eingriff.zuruf_min >= 10.0 || mv.eingriff.zuruf_min == 0.0;
    let bw = (r.w - 16.0) / 3.0;
    for (k, key) in [crate::id("antreiben"), crate::id("beruhigen"), crate::id("konzentration"), crate::id("fluegel"), crate::id("pressing"), crate::id("zeit")].iter().enumerate() {
        let br = Rect::new(r.x + (k % 3) as f32 * (bw + 8.0), r.y + 76.0 + (k / 3) as f32 * 36.0, bw, 30.0);
        if ui.button(br, &ui.fit(crate::inhalt::t(&format!("ui.live.zuruf.{key}")), 13.0, bw - 8.0), false) {
            if !frei { v.toast(crate::inhalt::t("ui.live.zuruf.warten"), false); continue; }
            mv.eingriff.zuruf_min = mv.minute.max(0.1);
            // a shout works for about a quarter of an hour: its effect on the rest is scaled to that
            let anteil = (15.0 / (90.0 - mv.minute).max(15.0)).min(1.0);
            // own goals factor, opponent's goals factor (changeable): wings +attack, high press +both, time-wasting −both
            zuruf = match k { 0 => (1.0 + 0.10 * anteil, 1.0 + 0.04 * anteil), 1 => (1.0, 1.0 - 0.06 * anteil), 2 => (1.0 - 0.02 * anteil, 1.0 - 0.08 * anteil), 3 => (1.0 + 0.06 * anteil, 1.0 + 0.02 * anteil), 4 => (1.0 + 0.08 * anteil, 1.0 + 0.05 * anteil), _ => (1.0 - 0.08 * anteil, 1.0 - 0.10 * anteil) };
            v.toast(&crate::tf!("ui.live.log.zuruf", m, crate::inhalt::t(&format!("ui.live.zuruf.{key}"))), true);
            neu = true;
        }
    }
    // substitution: a click on a player on the pitch, then on one from the bench
    let wechsel = mv.res.events.iter().filter(|e| e.side == side && e.kind == 5 && (e.min as f32) <= mv.minute).count();
    ui.text_semi(&crate::tf!("ui.live.wechsel", wechsel, 5), r.x, r.y + 162.0, 13.0, Theme::gedaempft());
    let elf = aktuelle_elf(mv, side);
    let gespielt: Vec<usize> = mv.res.events.iter().filter(|e| e.side == side && e.kind == 5).flat_map(|e| [Some(e.pid), e.pid2]).flatten().collect();
    let bank: Vec<usize> = c.squad().into_iter().filter(|p| !elf.contains(p) && !gespielt.contains(p) && c.world.players[*p].inj == 0 && c.world.players[*p].susp == 0).collect();
    let spw = (r.w - 8.0) / 2.0;
    let zh = ((r.h - 176.0) / 11.0).clamp(15.0, 22.0);
    let platz = ((r.h - 172.0) / zh).max(0.0) as usize;
    for (k, &p) in elf.iter().take(platz).enumerate() {
        let rr = Rect::new(r.x, r.y + 170.0 + k as f32 * zh, spw, zh - 2.0);
        let sel = mv.eingriff.raus == Some(p);
        if sel { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::problem(), 0.2)); } else if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
        let pl = &c.world.players[p];
        // live grade and stamina so a substitution can be judged (Fabian 23.11.)
        let (note, kraft) = (live_note(&mv.res, p, mv.minute), live_kraft(&mv.res, side, p, mv.minute, pl.fit));
        let rechts = 64.0;
        ui.pos_tag(rr.x + 2.0, rr.y + zh * 0.5, pos_de(&pl.pos));
        ui.text(&ui.fit(&pl.name, 12.0, spw - rechts - 50.0), rr.x + 44.0, rr.y + zh * 0.7, 12.0, Theme::text());
        ui.text_right(&format!("{note:.1}"), rr.x + spw - 36.0, rr.y + zh * 0.7, 12.0, Theme::note(note));
        ui.text_right(&format!("{kraft:.0}%"), rr.x + spw - 4.0, rr.y + zh * 0.7, 11.0, if kraft < 60.0 { Theme::problem() } else if kraft < 75.0 { Theme::achtung() } else { Theme::text2() });
        if ui.hit(rr) { mv.eingriff.raus = if sel { None } else { Some(p) }; }
    }
    let mut bank_s = bank.clone();
    bank_s.sort_by(|a, b| { let (p, q) = (&c.world.players[*a], &c.world.players[*b]); crate::pesdb::pos_index(&p.pos).cmp(&crate::pesdb::pos_index(&q.pos)).then(q.ovr.partial_cmp(&p.ovr).unwrap()) });
    for (k, &p) in bank_s.iter().take(platz).enumerate() {
        let rr = Rect::new(r.x + spw + 8.0, r.y + 170.0 + k as f32 * zh, spw, zh - 2.0);
        let aktiv = mv.eingriff.raus.is_some() && wechsel < 5;
        if aktiv && ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::gut(), 0.2)); }
        let pl = &c.world.players[p];
        ui.pos_tag(rr.x + 2.0, rr.y + zh * 0.5, pos_de(&pl.pos));
        ui.text(&ui.fit(&pl.name, 12.0, spw - 114.0), rr.x + 44.0, rr.y + zh * 0.7, 12.0, if aktiv { Theme::text() } else { Theme::text2() });
        ui.text_right(&format!("{:.0} · {:.0}%", pl.ovr, pl.fit), rr.x + spw - 4.0, rr.y + zh * 0.7, 11.0, Theme::text2());
        if aktiv && ui.hit(rr) {
            let raus = mv.eingriff.raus.take().unwrap();
            mv.res.events.push(MatchEvent { min: m.max(1), side, kind: 5, pid: raus, pid2: Some(p) });
            v.toast(&crate::tf!("ui.live.log.wechsel", m, pl.name.clone(), c.world.players[raus].name.clone()), true);
            ui.play(crate::id("klick"));
            neu = true;
        }
    }
    if neu {
        let elf = aktuelle_elf(&MatchView { minute: m as f32, ..mv.clone() }, side);
        let (fe, fg) = match mv.eingriff.mentalitaet { -1 => (0.85, 0.88), 1 => (1.12, 1.10), _ => (1.0, 1.0) };
        c.world.rest_neu(&mut mv.res, m.max(1), side, &elf, fe * zuruf.0, fg * zuruf.1);
    }
}

// ================================================================ Spieltag
/// The ticker's wording of an event: a variant fixed by minute and player, so a line keeps its words while it stays
/// on screen, and the same kind of event reads differently across a match (Fabian 23.11.: no repeats).
pub(super) fn ticker_key(basis: &str, e: &MatchEvent) -> String { let n = crate::inhalt::varianten(basis); crate::inhalt::variante_key(basis, (e.min as usize * 7 + e.pid * 13) % n) }
/// A deterministic 0..1 per minute slice and salt (the radar's movement).
fn rz(c: &Career, mv: &MatchView, k: u32, salz: &str) -> f32 { crate::mensch::zufall(&["radar", salz, &k.to_string(), &mv.res.h.to_string(), &c.abs().to_string()]) } // roh
/// The live radar: a pitch from above, both elevens on their spots, the ball moving between the players; chances and
/// goals of the match pull the ball to the goal at their minute. shortcut: the movement is drawn from the events, not
/// simulated position by position; a real position model would come from the section simulation.
pub(super) fn radar(ui: &mut Ui, c: &Career, mv: &MatchView, r: Rect) {
    ui.card(r);
    let p = Rect::new(r.x + 12.0, r.y + 12.0, r.w - 24.0, r.h - 24.0);
    rrect(p.x, p.y, p.w, p.h, 6.0, hex(0x1e5a35));
    for i in 0..10 { if i % 2 == 0 { draw_rectangle(p.x + i as f32 * p.w / 10.0, p.y, p.w / 10.0, p.h, hex(0x226440)); } }
    let lc = Color::new(1.0, 1.0, 1.0, 0.4);
    draw_rectangle_lines(p.x, p.y, p.w, p.h, 2.0, lc);
    draw_line(p.x + p.w / 2.0, p.y, p.x + p.w / 2.0, p.y + p.h, 1.5, lc);
    draw_circle_lines(p.x + p.w / 2.0, p.y + p.h / 2.0, p.h * 0.15, 1.5, lc);
    for s in [0.0, 1.0] { let x = p.x + s * (p.w - p.w * 0.16); draw_rectangle_lines(x, p.y + p.h * 0.22, p.w * 0.16, p.h * 0.56, 1.5, lc); }
    let m = mv.minute.min(90.0);
    // where the ball is: towards a goal at a shot's minute, else wandering between the thirds
    let schuss = mv.res.events.iter().filter(|e| matches!(e.kind, 0 | 1 | 6)).find(|e| (e.min as f32 - m).abs() < 0.6);
    let k = (m * 2.0) as u32;
    let f = m * 2.0 - k as f32;
    let punkt = |k: u32| (0.15 + 0.7 * rz(c, mv, k, crate::id("x")), 0.12 + 0.76 * rz(c, mv, k, crate::id("y")));
    let (a, b) = (punkt(k), punkt(k + 1));
    let mut ball = (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f);
    if let Some(e) = schuss {
        let ziel = if (e.side == 0) != (e.kind == 1) { 0.97 } else { 0.03 };
        let t = 1.0 - ((e.min as f32 - m).abs() / 0.6);
        ball = (ball.0 + (ziel - ball.0) * t, ball.1 + (0.5 - ball.1) * t);
    }
    // the players: formation spots, pulled a little towards the ball (home left to right)
    for (side, farbe) in [(0u8, club_color(c, mv.res.h).0), (1u8, club_color(c, mv.res.a).0)] {
        let elf = aktuelle_elf(mv, side);
        let spots = crate::world::formation_spots(if (side == 0) == (mv.res.h == c.own) { c.formation } else { 0 });
        for (i, _) in elf.iter().enumerate().take(11) {
            let (sx, sy) = spots[i.min(10)];
            let mut x = 0.04 + sy * 0.9;
            if side == 1 { x = 1.0 - x; }
            let mut y = sx;
            x += (ball.0 - x) * 0.12;
            y += (ball.1 - y) * 0.08;
            draw_circle(p.x + x * p.w, p.y + y * p.h, 6.5, crate::ui::lesbar(farbe, hex(0x1e5a35), 2.0));
            draw_circle_lines(p.x + x * p.w, p.y + y * p.h, 6.5, 1.5, WHITE);
        }
    }
    draw_circle(p.x + ball.0 * p.w, p.y + ball.1 * p.h, 4.5, WHITE);
    draw_circle_lines(p.x + ball.0 * p.w, p.y + ball.1 * p.h, 4.5, 1.0, BLACK);
    // a goal: the net flashes for a moment
    if let Some(e) = mv.res.events.iter().filter(|e| matches!(e.kind, 0 | 1)).find(|e| m - e.min as f32 >= 0.0 && m - (e.min as f32) < 1.0) {
        let links = !((e.side == 0) != (e.kind == 1));
        let x = if links { p.x } else { p.x + p.w - 10.0 };
        draw_rectangle(x, p.y + p.h * 0.4, 10.0, p.h * 0.2, alpha(Theme::gold(), 0.5 + 0.5 * (get_time() as f32 * 8.0).sin().abs()));
        ui.title(crate::inhalt::t("tk.tor"), p.x + p.w / 2.0 - 40.0, p.y + 34.0, 30.0, WHITE);
    }
}
