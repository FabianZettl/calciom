//! Live-Ansicht eines Spiels in FL27 (Punkt 4): what the Coaching Zone reads from the game's memory, on the desktop in
//! columns after its phone page (web/phone.html): scoreboard in the team colours, both line-ups with places, grades,
//! form arrow and fitness, the ticker with big cards (and „Alles“), flow per 15 minutes, team figures, a player table
//! and sub ideas. New goals, cards and missed penalties come up as a card with a sound.
use crate::game::Career;
use crate::pages::{club_color, form_arrow, View};
use crate::ui::*;
use crate::world::pos_de;
use macroquad::prelude::*;
use pesm_coaching_zone::live::{Event, EventKind, MatchView, PlayerView};
use std::collections::HashMap;

fn r(k: &str) -> f32 { crate::regeln::f("live_spiel", k) }
fn t(k: &str) -> &'static str { crate::inhalt::t(k) }
const ABSCHNITT: [&str; 9] = ["live.abschnitt.1", "live.abschnitt.2", "live.abschnitt.3", "live.abschnitt.4", "live.abschnitt.5", "live.abschnitt.6",
    "live.abschnitt.7", "live.abschnitt.8", "live.abschnitt.9"];

/// FL27 player id → PES M player, for the two teams of the match.
fn index(c: &Career, h: usize, a: usize) -> HashMap<u32, usize> {
    [h, a].iter().flat_map(|&k| c.world.clubs[k].squad.iter().map(|&p| (c.world.players[p].id, p))).collect()
}
fn spieler(m: &MatchView, side: usize, slot: usize) -> Option<&PlayerView> { m.players.iter().find(|p| p.side == side && p.slot == slot) }
/// The name PES M knows for a place, else what the Coaching Zone has.
fn name(c: &Career, idx: &HashMap<u32, usize>, m: &MatchView, side: usize, slot: usize) -> String {
    match spieler(m, side, slot) { Some(p) => idx.get(&p.id).map(|&q| c.world.players[q].name.clone()).unwrap_or_else(|| p.name.clone()), None => "?".into() }
}

/// The ticker line of an event in plain German (the Coaching Zone's own texts are English).
pub fn ereignis_text(c: &Career, idx: &HashMap<u32, usize>, m: &MatchView, e: &Event) -> String {
    let n = name(c, idx, m, e.side, e.slot);
    let o = |s: usize| name(c, idx, m, e.side, s);
    match e.kind {
        EventKind::Goal => match (e.detail, e.other) {
            (1, _) => crate::tf!("live.ev.tor_elfmeter", n),
            (2, _) => crate::tf!("live.ev.tor_freistoss", n),
            (_, Some(v)) => crate::tf!("live.ev.tor_vorlage", n, o(v)),
            _ => crate::tf!("live.ev.tor", n),
        },
        EventKind::OwnGoal => crate::tf!("live.ev.eigentor", n),
        EventKind::Yellow => crate::tf!("live.ev.gelb", n),
        EventKind::SecondYellow => crate::tf!("live.ev.gelbrot", n),
        EventKind::Red => crate::tf!("live.ev.rot", n),
        EventKind::Sub => match e.other { Some(x) => crate::tf!("live.ev.wechsel", n, o(x)), None => crate::tf!("live.ev.wechsel_rein", n) },
        EventKind::PenaltyMissed => crate::tf!("live.ev.elfmeter_vorbei", n),
        EventKind::Chance => match e.other { Some(k) => crate::tf!("live.ev.chance", n, name(c, idx, m, 1 - e.side, k)), None => crate::tf!("live.ev.chance_kurz", n) },
        EventKind::ShotWide => crate::tf!("live.ev.vorbei", n),
        EventKind::Corner => crate::tf!("live.ev.ecke", n),
        EventKind::FreeKick => crate::tf!("live.ev.freistoss", n),
        EventKind::Offside => crate::tf!("live.ev.abseits", n),
    }
}

/// A new goal, card or missed penalty: title, sound and colour of its card (None for the rest).
fn einblendung_art(k: &EventKind) -> Option<(&'static str, &'static str, Color)> {
    match k {
        EventKind::Goal => Some(("live.pop.tor", crate::id("tor"), Theme::gut())),
        EventKind::OwnGoal => Some(("live.pop.eigentor", crate::id("tor"), Theme::problem())),
        EventKind::Yellow => Some(("live.pop.gelb", crate::id("karte"), hex(0xf5d90a))),
        EventKind::SecondYellow => Some(("live.pop.gelbrot", crate::id("karte"), hex(0xf97316))),
        EventKind::Red => Some(("live.pop.rot", crate::id("karte"), Theme::problem())),
        EventKind::PenaltyMissed => Some(("live.pop.elfmeter", crate::id("elfmeter_vorbei"), Theme::achtung())),
        _ => None,
    }
}

/// Sub ideas for the own side: who should come off and why (yellow card, grade, form), and the best bench player for
/// that place. Each idea: (off, on, reason).
pub fn wechselideen(c: &Career, idx: &HashMap<u32, usize>, m: &MatchView, side: usize) -> Vec<(usize, usize, String)> {
    let mut raus: Vec<(f32, &PlayerView, String)> = vec![];
    for p in m.players.iter().filter(|p| p.side == side && p.on_pitch && !p.sent_off) {
        let Some(&q) = idx.get(&p.id) else { continue };
        let pl = &c.world.players[q];
        if pl.pos == crate::id("GK") { continue; }
        let n = pl.name.clone();
        if p.yellow > 0 && m.minute as f32 >= r("gelb_ab_minute") { raus.push((3.0, p, crate::tf!("live.grund.gelb", n))); }
        else if let Some(note) = p.rating.filter(|x| *x < r("note_schwach") && m.minute as f32 >= r("note_ab_minute")) {
            raus.push((2.0 + (r("note_schwach") - note), p, crate::tf!("live.grund.note", n, format!("{note:.1}"))));
        } else if crate::mensch::pfeil(pl.form) <= 1 && m.minute as f32 >= r("form_ab_minute") { raus.push((1.0, p, crate::tf!("live.grund.form", n))); }
    }
    raus.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let mut frei: Vec<usize> = m.players.iter().filter(|p| p.side == side && p.slot >= 11 && !p.on_pitch && !p.came_on && !p.went_off && !p.sent_off)
        .filter_map(|p| idx.get(&p.id).copied()).collect();
    let mut out = vec![];
    for (_, p, grund) in raus {
        let Some(&q) = idx.get(&p.id) else { continue };
        let pos = c.world.players[q].pos.clone();
        let Some((k, _)) = frei.iter().enumerate().filter(|(_, &b)| c.world.players[b].pos != crate::id("GK")).max_by(|x, y| c.world.eff(*x.1, &pos).partial_cmp(&c.world.eff(*y.1, &pos)).unwrap()) else { break };
        out.push((q, frei.remove(k), grund));
        if out.len() >= r("ideen_max") as usize { break; }
    }
    out
}

fn note_chip(ui: &Ui, x: f32, y: f32, n: Option<f32>) {
    let Some(n) = n else { ui.text_center("–", x + 18.0, y + 15.0, 14.0, Theme::gedaempft()); return };
    let col = Theme::note(n);
    rrect(x, y, 36.0, 20.0, 6.0, col);
    ui.text_center(&format!("{n:.1}"), x + 18.0, y + 15.0, 14.0, on(col));
}

/// One player row of height `zh` (19..28): shirt colour, photo, place, name with goals, cards and sub arrows, form arrow,
/// fitness before the kick-off, grade.
fn zeile(ui: &Ui, c: &Career, idx: &HashMap<u32, usize>, p: &PlayerView, farbe: Color, x: f32, y: f32, w: f32, zh: f32) {
    let q = idx.get(&p.id).copied();
    let (nm, pos) = q.map(|q| (c.world.players[q].name.clone(), pos_de(&c.world.players[q].pos))).unwrap_or((p.name.clone(), "?"));
    let cy = y + zh / 2.0;
    let g = (zh * 0.55).clamp(12.0, 15.0);
    draw_rectangle(x, cy - zh * 0.42, 4.0, zh * 0.84, farbe);
    ui.player_photo(x + 20.0, cy, zh * 0.42, p.id, &nm);
    let mut nx = x + 36.0;
    let pw = ui.measure(pos, 12.0) + 10.0;
    rrect(nx, cy - 9.0, pw, 18.0, 5.0, Theme::erhoehung());
    ui.text_center(pos, nx + pw / 2.0, cy + 4.0, 12.0, Theme::text2());
    nx += pw + 6.0;
    let rechts = x + w - 40.0;
    let n = ui.fit(&nm, g, (rechts - 120.0 - nx).max(20.0));
    ui.text(&n, nx, cy + g * 0.35, g, if p.sent_off { Theme::gedaempft() } else { Theme::text() });
    let mut mx = nx + ui.measure(&n, g) + 6.0;
    if p.goals > 0 { ui.text_semi(&format!("{}×", p.goals), mx, cy + 4.0, 13.0, Theme::gut()); mx += 22.0; }
    if p.yellow > 0 { rrect(mx, cy - 6.0, 9.0, 13.0, 2.0, hex(0xf5d90a)); mx += 13.0; }
    if p.red > 0 || p.sent_off { rrect(mx, cy - 6.0, 9.0, 13.0, 2.0, Theme::problem()); mx += 13.0; }
    if p.came_on { draw_triangle(vec2(mx, cy + 4.0), vec2(mx + 10.0, cy + 4.0), vec2(mx + 5.0, cy - 5.0), Theme::gut()); mx += 14.0; }
    if p.went_off { draw_triangle(vec2(mx, cy - 5.0), vec2(mx + 10.0, cy - 5.0), vec2(mx + 5.0, cy + 4.0), Theme::problem()); }
    if let Some(q) = q {
        let pl = &c.world.players[q];
        form_arrow(ui, rechts - 84.0, cy, pl.form);
        // fitness before the kick-off: FL27's live fitness is not read yet (wish for the Coaching Zone)
        let k = (pl.fit / 100.0).clamp(0.0, 1.0);
        let kc = if pl.fit >= 60.0 { Theme::gut() } else if pl.fit >= 30.0 { Theme::achtung() } else { Theme::problem() };
        rrect(rechts - 64.0, cy - 3.0, 40.0, 6.0, 3.0, Theme::ueberlagerung());
        rrect(rechts - 64.0, cy - 3.0, 40.0 * k, 6.0, 3.0, kc);
    }
    note_chip(ui, x + w - 38.0, cy - 10.0, p.rating);
}

fn mannschaft(ui: &mut Ui, c: &Career, idx: &HashMap<u32, usize>, m: &MatchView, side: usize, club: usize, rr: Rect) {
    let (c1, _) = club_color(c, club);
    ui.card_titled(rr, &c.world.clubs[club].name, c1);
    ui.text(t("live.legende"), rr.x + 16.0, rr.y + 54.0, 13.0, Theme::gedaempft());
    let mut auf: Vec<&PlayerView> = m.players.iter().filter(|p| p.side == side && (p.on_pitch || (p.slot < 11 && !p.went_off && !p.sent_off && !p.came_on))).collect();
    auf.sort_by_key(|p| (p.slot >= 11, p.slot));
    let weg: Vec<&PlayerView> = m.players.iter().filter(|p| p.side == side && !auf.iter().any(|a| a.slot == p.slot) && (p.went_off || p.came_on || p.sent_off)).collect();
    // rows shrink down to 19 units so the whole eleven fits; who was subbed on and off follows when there is room
    let mut y = rr.y + 62.0;
    let frei = rr.y + rr.h - y - 8.0;
    let n = auf.len() + if weg.is_empty() { 0 } else { weg.len() + 1 };
    let zh = (frei / n.max(1) as f32).clamp(19.0, 28.0);
    let platz = (frei / zh).floor().max(1.0) as usize;
    for p in auf.iter().take(platz) { zeile(ui, c, idx, p, c1, rr.x + 10.0, y, rr.w - 20.0, zh); y += zh; }
    if !weg.is_empty() && auf.len() + 1 < platz {
        ui.text_semi(t("live.bank"), rr.x + 16.0, y + zh * 0.7, 13.0, Theme::gedaempft());
        y += zh;
        for p in weg.iter().take(platz - auf.len() - 1) { zeile(ui, c, idx, p, alpha(c1, 0.4), rr.x + 10.0, y, rr.w - 20.0, zh); y += zh; }
    }
}

fn ticker(ui: &mut Ui, v: &mut View, c: &Career, idx: &HashMap<u32, usize>, m: &MatchView, h: usize, a: usize, rr: Rect) {
    ui.card_titled(rr, t("live.ticker"), Theme::akzent());
    if let Some(i) = ui.segment(Rect::new(rr.x + rr.w - 330.0, rr.y + 6.0, 320.0, 24.0), &[t("live.wichtiges"), t("live.alles"), t("live.tabelle")], v.fl27_reiter) { v.fl27_reiter = i; }
    if v.fl27_reiter == 2 { return tabelle(ui, c, idx, m, h, a, rr); }
    let ev: Vec<&Event> = m.events.iter().rev().filter(|e| v.fl27_reiter == 1 || e.key).collect();
    if ev.is_empty() { ui.text(t("live.noch_nichts"), rr.x + 16.0, rr.y + 64.0, 15.0, Theme::text2()); return; }
    let mut y = rr.y + 46.0;
    for e in ev {
        let gross = einblendung_art(&e.kind).is_some() || e.kind == EventKind::Sub;
        let hh = if gross { 54.0 } else { 28.0 };
        if y + hh > rr.y + rr.h - 6.0 { break; }
        let (col, _) = club_color(c, if e.side == 0 { h } else { a });
        if gross {
            rrect(rr.x + 10.0, y, rr.w - 20.0, hh - 6.0, 10.0, Theme::erhoehung());
            draw_rectangle(rr.x + 10.0, y, 5.0, hh - 6.0, einblendung_art(&e.kind).map(|x| x.2).unwrap_or(col));
            ui.title(&format!("{}'", e.minute), rr.x + 24.0, y + 32.0, 24.0, Theme::akzent());
            ui.text(&ereignis_text(c, idx, m, e), rr.x + 84.0, y + 22.0, 16.0, Theme::text());
            ui.text(&c.world.clubs[if e.side == 0 { h } else { a }].name, rr.x + 84.0, y + 40.0, 13.0, Theme::text2());
        } else {
            ui.text_semi(&format!("{}'", e.minute), rr.x + 24.0, y + 18.0, 14.0, Theme::gedaempft());
            draw_circle(rr.x + 70.0, y + 13.0, 4.0, col);
            ui.text(&ereignis_text(c, idx, m, e), rr.x + 84.0, y + 18.0, 14.0, Theme::text2());
        }
        y += hh;
    }
}

fn tabelle(ui: &mut Ui, c: &Career, idx: &HashMap<u32, usize>, m: &MatchView, h: usize, a: usize, rr: Rect) {
    let x = rr.x + 14.0;
    let w = rr.w - 28.0;
    let spalten = [(t("live.tab.tore"), 0.52), (t("live.tab.vorlagen"), 0.6), (t("live.tab.schuesse"), 0.68), (t("live.tab.paesse"), 0.78), (t("live.tab.note"), 0.9)];
    ui.text_semi(t("live.tab.spieler"), x, rr.y + 58.0, 13.0, Theme::gedaempft());
    for (s, f) in spalten { ui.text_semi(s, x + w * f, rr.y + 58.0, 13.0, Theme::gedaempft()); }
    let mut liste: Vec<&PlayerView> = m.players.iter().filter(|p| p.rating.is_some()).collect();
    liste.sort_by(|p, q| q.rating.partial_cmp(&p.rating).unwrap());
    let mut y = rr.y + 66.0;
    for p in liste {
        if y + 24.0 > rr.y + rr.h - 4.0 { break; }
        let (col, _) = club_color(c, if p.side == 0 { h } else { a });
        draw_rectangle(x, y + 4.0, 4.0, 16.0, col);
        let nm = idx.get(&p.id).map(|&q| c.world.players[q].name.clone()).unwrap_or_else(|| p.name.clone());
        ui.text(&ui.fit(&nm, 14.0, w * 0.5 - 14.0), x + 10.0, y + 17.0, 14.0, Theme::text());
        for (k, v) in [p.goals, p.assists, p.shots].iter().enumerate() { ui.text(&v.to_string(), x + w * spalten[k].1, y + 17.0, 14.0, Theme::text2()); }
        ui.text(&format!("{}/{}", p.passes_ok, p.passes), x + w * 0.78, y + 17.0, 14.0, Theme::text2());
        note_chip(ui, x + w * 0.9 - 4.0, y + 1.0, p.rating);
        y += 24.0;
    }
}

fn verlauf(ui: &mut Ui, c: &Career, m: &MatchView, h: usize, a: usize, rr: Rect) {
    ui.card_titled(rr, t("live.verlauf"), Theme::akzent());
    ui.text(t("live.verlauf_sub"), rr.x + 16.0, rr.y + 54.0, 13.0, Theme::gedaempft());
    let (hc, _) = club_color(c, h);
    let (ac, _) = club_color(c, a);
    let max = m.flow.iter().flat_map(|f| f.touches).max().unwrap_or(1).max(1) as f32;
    let zeilen = m.flow.len().max(1) as f32;
    let zh = ((rr.h - 70.0) / zeilen).min(24.0);
    for (i, f) in m.flow.iter().enumerate() {
        let y = rr.y + 66.0 + i as f32 * zh;
        ui.text(t(ABSCHNITT[f.period.min(8)]), rr.x + 16.0, y + zh * 0.7, 12.0, Theme::text2());
        let bx = rr.x + 120.0;
        let bw = rr.w - 210.0;
        rrect(bx, y + 3.0, bw * f.touches[0] as f32 / max, zh * 0.3, 2.0, hc);
        rrect(bx, y + 3.0 + zh * 0.35, bw * f.touches[1] as f32 / max, zh * 0.3, 2.0, ac);
        if f.shots[0] + f.shots[1] > 0 { ui.text_right(&crate::tf!("live.schuesse_kurz", f.shots[0], f.shots[1]), rr.x + rr.w - 14.0, y + zh * 0.7, 12.0, Theme::text2()); }
    }
}

fn zahlen(ui: &mut Ui, c: &Career, m: &MatchView, h: usize, a: usize, rr: Rect) {
    ui.card_titled(rr, t("live.st.titel"), Theme::akzent());
    let s = |f: &dyn Fn(&PlayerView) -> u32| -> [u32; 2] { [0, 1].map(|side| m.players.iter().filter(|p| p.side == side).map(f).sum()) };
    let (sch, auf, pa, pok, fo, pr) = (s(&|p| p.shots), s(&|p| p.on_target), s(&|p| p.passes), s(&|p| p.passes_ok), s(&|p| p.fouls), s(&|p| p.saves));
    let quote = [0, 1].map(|i| if pa[i] > 0 { (pok[i] * 100 / pa[i]) as f32 } else { 0.0 });
    let reihen: [(&str, [f32; 2], bool); 6] = [(t("live.st.schuesse"), sch.map(|x| x as f32), false), (t("live.st.aufs_tor"), auf.map(|x| x as f32), false),
        (t("live.st.xg"), m.xg, true), (t("live.st.passquote"), quote, false), (t("live.st.fouls"), fo.map(|x| x as f32), false), (t("live.st.paraden"), pr.map(|x| x as f32), false)];
    let (hc, _) = club_color(c, h);
    let (ac, _) = club_color(c, a);
    let zh = ((rr.h - 72.0) / 6.0).min(30.0);
    for (i, (l, w, dez)) in reihen.iter().enumerate() {
        let y = rr.y + 42.0 + i as f32 * zh;
        let f = |x: f32| if *dez { format!("{x:.2}") } else { format!("{x:.0}") };
        ui.text_semi(&f(w[0]), rr.x + 16.0, y + 16.0, 15.0, Theme::text());
        ui.text_right(&f(w[1]), rr.x + rr.w - 16.0, y + 16.0, 15.0, Theme::text());
        ui.text_center(l, rr.x + rr.w / 2.0, y + 12.0, 13.0, Theme::text2());
        let sum = (w[0] + w[1]).max(0.01);
        let bx = rr.x + 70.0;
        let bw = rr.w - 140.0;
        rrect(bx, y + 17.0, bw * w[0] / sum, 4.0, 2.0, hc);
        rrect(bx + bw * w[0] / sum, y + 17.0, bw * w[1] / sum, 4.0, 2.0, ac);
    }
    ui.text(&ui.fit(t("live.st.hinweis"), 12.0, rr.w - 32.0), rr.x + 16.0, rr.y + rr.h - 10.0, 12.0, Theme::gedaempft());
}

fn ideen(ui: &mut Ui, c: &Career, idx: &HashMap<u32, usize>, m: &MatchView, h: usize, rr: Rect) {
    ui.card_titled(rr, t("live.wechselideen"), Theme::akzent());
    let side = if h == c.own { 0 } else { 1 };
    let limit = if m.subs.limit[side] > 0 { m.subs.limit[side] as usize } else { r("wechsel_max") as usize };
    let gemacht = m.events.iter().filter(|e| e.kind == EventKind::Sub && e.side == side).count();
    let rest = limit.saturating_sub(gemacht);
    ui.text(&crate::tf!("live.wechsel_rest", rest, limit), rr.x + 16.0, rr.y + 54.0, 13.0, Theme::gedaempft());
    if rest == 0 { ui.text(t("live.keine_wechsel"), rr.x + 16.0, rr.y + 82.0, 15.0, Theme::text2()); return; }
    let l = wechselideen(c, idx, m, side);
    if l.is_empty() { ui.text(t("live.keine_ideen"), rr.x + 16.0, rr.y + 82.0, 15.0, Theme::text2()); }
    let mut y = rr.y + 66.0;
    for (raus, rein, grund) in l.iter().take(rest) {
        if y + 46.0 > rr.y + rr.h - 30.0 { break; }
        rrect(rr.x + 10.0, y, rr.w - 20.0, 42.0, 8.0, Theme::erhoehung());
        ui.text_semi(&crate::tf!("live.idee", c.world.players[*raus].name, c.world.players[*rein].name), rr.x + 20.0, y + 17.0, 14.0, Theme::text());
        ui.text(grund, rr.x + 20.0, y + 35.0, 12.0, Theme::text2());
        y += 48.0;
    }
    ui.text(t("live.wechsel_hinweis"), rr.x + 16.0, rr.y + rr.h - 12.0, 12.0, Theme::gedaempft());
}

/// The whole live view over the match day page (it covers the cards below it; clicks there are off meanwhile).
pub fn zeichnen(ui: &mut Ui, v: &mut View, c: &Career, h: usize, a: usize, rr: Rect) {
    let Some(m) = v.fl27_ansicht.clone() else { return };
    let idx = index(c, h, a);
    draw_rectangle(rr.x - 4.0, rr.y - 4.0, rr.w + 8.0, rr.h + 8.0, Theme::grund());
    // scoreboard in the team colours
    let sb = Rect::new(rr.x, rr.y, rr.w, 118.0);
    ui.card(sb);
    let (hc, _) = club_color(c, h);
    let (ac, _) = club_color(c, a);
    rrect(sb.x, sb.y, 10.0, sb.h, 4.0, hc);
    rrect(sb.x + sb.w - 10.0, sb.y, 10.0, sb.h, 4.0, ac);
    ui.club_crest(sb.x + 60.0, sb.y + 52.0, 30.0, c.world.clubs[h].id, hc, &c.world.clubs[h].short);
    ui.club_crest(sb.x + sb.w - 60.0, sb.y + 52.0, 30.0, c.world.clubs[a].id, ac, &c.world.clubs[a].short);
    let nb = sb.w / 2.0 - 220.0;
    ui.title(&ui.fit(&c.world.clubs[h].name, 24.0, nb), sb.x + 100.0, sb.y + 58.0, 24.0, Theme::text());
    let an = ui.fit(&c.world.clubs[a].name, 24.0, nb);
    ui.title(&an, sb.x + sb.w - 100.0 - ui.measure_title(&an, 24.0), sb.y + 58.0, 24.0, Theme::text());
    let stand = format!("{} : {}", m.score[0], m.score[1]);
    ui.title(&stand, sb.x + sb.w / 2.0 - ui.measure_title(&stand, 52.0) / 2.0, sb.y + 66.0, 52.0, Theme::text());
    ui.text_center(&format!("{}' · {}", m.minute, t(ABSCHNITT[m.period.min(8)])), sb.x + sb.w / 2.0, sb.y + 90.0, 15.0, Theme::akzent());
    // ball touches of this period (FL27 keeps no possession)
    let bw = 360.0_f32.min(sb.w * 0.3);
    let bx = sb.x + sb.w / 2.0 - bw / 2.0;
    rrect(bx, sb.y + 98.0, bw * m.touch_share as f32 / 100.0, 6.0, 3.0, hc);
    rrect(bx + bw * m.touch_share as f32 / 100.0, sb.y + 98.0, bw * (100 - m.touch_share.min(100)) as f32 / 100.0, 6.0, 3.0, ac);
    ui.text(&crate::tf!("live.ballkontakte", m.touch_share, 100 - m.touch_share.min(100)), sb.x + 100.0, sb.y + 104.0, 12.0, Theme::gedaempft());
    ui.text(&if v.fl27_vorschau { t("live.demo").to_string() } else { v.fl27_status.clone() }, sb.x + 100.0, sb.y + 22.0, 13.0, Theme::gedaempft());
    if ui.button(Rect::new(sb.x + sb.w - 230.0, sb.y + 8.0, 156.0, 26.0), t("ui.pages.mitlesen_beenden"), false) { v.fl27 = None; v.fl27_ansicht = None; v.fl27_vorschau = false; return; }
    // three columns, and below them flow, team figures and sub ideas
    let top = sb.y + sb.h + 10.0;
    let unten = (rr.h * 0.3).clamp(190.0, 270.0);
    let oh = rr.y + rr.h - top - unten - 10.0;
    let sw = (rr.w - 20.0) * 0.3;
    let mw = rr.w - 20.0 - 2.0 * sw;
    mannschaft(ui, c, &idx, &m, 0, h, Rect::new(rr.x, top, sw, oh));
    ticker(ui, v, c, &idx, &m, h, a, Rect::new(rr.x + sw + 10.0, top, mw, oh));
    mannschaft(ui, c, &idx, &m, 1, a, Rect::new(rr.x + sw + mw + 20.0, top, sw, oh));
    let uy = top + oh + 10.0;
    verlauf(ui, c, &m, h, a, Rect::new(rr.x, uy, sw, unten));
    zahlen(ui, c, &m, h, a, Rect::new(rr.x + sw + 10.0, uy, mw, unten));
    ideen(ui, c, &idx, &m, h, Rect::new(rr.x + sw + mw + 20.0, uy, sw, unten));
    // new goals, cards and missed penalties: a card with a sound (old events when joining late stay quiet)
    let neu: Vec<&Event> = match v.fl27_seq { None => vec![], Some(s) => m.events.iter().filter(|e| e.seq > s).collect() };
    if let Some(e) = neu.iter().rev().find(|e| einblendung_art(&e.kind).is_some()) {
        let (titel, klang, _) = einblendung_art(&e.kind).unwrap();
        ui.play(klang);
        v.fl27_einblendung = Some((e.kind.clone(), titel, ereignis_text(c, &idx, &m, e), e.side, crate::ui::jetzt()));
    }
    v.fl27_seq = Some(m.last_seq.max(v.fl27_seq.unwrap_or(0)));
    if let Some((k, titel, text, side, seit)) = v.fl27_einblendung.clone() {
        let alter = crate::ui::jetzt() - seit;
        if alter > r("einblendung_sekunden") as f64 { v.fl27_einblendung = None; return; }
        let col = einblendung_art(&k).map(|x| x.2).unwrap_or(Theme::akzent());
        let ew = 520.0_f32.min(rr.w - 40.0);
        let er = Rect::new(rr.x + rr.w / 2.0 - ew / 2.0, sb.y + sb.h + 40.0, ew, 120.0);
        ui.rahmen(er);
        rrect(er.x - 2.0, er.y - 2.0, er.w + 4.0, er.h + 4.0, 16.0, col);
        rrect(er.x, er.y, er.w, er.h, 14.0, Theme::flaeche());
        ui.title(t(titel), er.x + 24.0, er.y + 52.0, 40.0, col);
        ui.text(&text, er.x + 24.0, er.y + 82.0, 17.0, Theme::text());
        ui.text(&c.world.clubs[if side == 0 { h } else { a }].name, er.x + 24.0, er.y + 104.0, 14.0, Theme::text2());
    }
}

/// Made-up numbers for previews and the window check (PESM_LIVE=1): the next own match in minute 67, 2:1, with a goal,
/// a yellow card, a sub and a missed penalty. Never used with a real match.
pub fn beispiel(c: &Career) -> Option<MatchView> {
    use pesm_coaching_zone::live::{FlowRow, SubsInfo};
    let f = c.fixture(c.today_match()?);
    let mut players = vec![];
    for (side, club) in [(0usize, f.h), (1usize, f.a)] {
        let (xi, _) = c.world.lineup(club, 0, &[]);
        let bank: Vec<usize> = c.world.clubs[club].squad.iter().copied().filter(|p| !xi.iter().any(|x| x.1 == *p)).take(5).collect();
        for (slot, p) in xi.iter().map(|x| x.1).chain(bank).enumerate() {
            let pl = &c.world.players[p];
            let (on, came, went) = match (side, slot) { (0, 7) => (false, false, true), (0, 11) => (true, true, false), (_, s) => (s < 11, false, false) };
            let rating = (on || went).then(|| 6.0 + ((pl.id % 17) as f32 - 6.0) / 10.0 + if (side, slot) == (0, 9) { 1.1 } else { 0.0 });
            players.push(PlayerView { side, slot, id: pl.id, name: pl.name.clone(), full_name: pl.name.clone(), number: Some(pl.no as u32), position: None, ovr: None,
                form: None, fitness: None, role: None, starter: slot < 11, on_pitch: on, came_on: came, went_off: went, sent_off: false, rating,
                goals: if (side, slot) == (0, 9) { 2 } else if (side, slot) == (1, 10) { 1 } else { 0 }, assists: 0, shots: if on { (pl.id % 3) as u32 } else { 0 },
                on_target: 0, passes: 20 + pl.id % 30, passes_ok: 16 + pl.id % 20, touches: 30, duels_won: 2, interceptions: 1, fouls: pl.id % 2, yellow: if (side, slot) == (1, 4) { 1 } else { 0 },
                red: 0, saves: if slot == 0 { 3 } else { 0 } });
        }
    }
    let ev = |seq: u32, minute: u32, side: usize, kind: EventKind, slot: usize, other: Option<usize>| Event { seq, minute, side, kind: kind.clone(), slot, other, detail: 0, key: kind.key(), text: String::new() };
    let events = vec![ev(1, 12, 0, EventKind::Goal, 9, Some(8)), ev(2, 23, 1, EventKind::Corner, 6, None), ev(3, 31, 1, EventKind::Yellow, 4, None),
        ev(4, 38, 1, EventKind::Goal, 10, None), ev(5, 52, 0, EventKind::PenaltyMissed, 10, None), ev(6, 58, 0, EventKind::Sub, 11, Some(7)),
        ev(7, 61, 0, EventKind::ShotWide, 9, None), ev(8, 64, 0, EventKind::Goal, 9, None)];
    let flow = (0..5).map(|i| FlowRow { period: i, label: String::new(), shots: [(i as u32 * 7) % 4, (i as u32 * 5) % 3], touches: [60 + i as u32 * 9 % 40, 50 + i as u32 * 13 % 35] }).collect();
    Some(MatchView { home: String::new(), away: String::new(), colors: [None, None], team_ids: [None, None], score: [2, 1], period: 4, minute: 67, period_label: String::new(),
        minute_tracked: true, touch_share: 56, team_stats: vec![], xg: [1.84, 0.71], players, events, flow, last_seq: 8,
        subs: SubsInfo { available: false, limit: [5, 5], from_memory: false }, subs_pending: vec![] })
}
