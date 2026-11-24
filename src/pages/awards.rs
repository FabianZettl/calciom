use super::*;
use crate::preise::{ist_spieler_preis, trainer_name, BALLON, BEST};

/// World › Awards (Fabian 24.11.): the Ballon d'Or night and The Best as their own section, in gala colours.
pub(super) fn awards_seite(ui: &mut Ui, v: &mut View, c: &mut Career) { gala_seite(ui, v, c, &BALLON.map(|x| x.0), ("ballon_nominierung", "ballon_gala")); }
pub(super) fn the_best_seite(ui: &mut Ui, v: &mut View, c: &mut Career) { gala_seite(ui, v, c, &BEST.map(|x| x.0), ("best_shortlist", "best_gala")); }

const GOLD: u32 = 0xD4AF37;
fn medaille(i: usize) -> Color { [hex(GOLD), hex(0xC9D1DC), hex(0xCD7F32)][i.min(2)] }
/// A dark card with a gold rim and slow rays: the award night's look.
fn gala_karte(r: Rect) {
    rrect(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, 14.0, alpha(hex(GOLD), 0.9));
    rrect(r.x, r.y, r.w, r.h, 12.0, hex(0x14110b));
    rrect(r.x, r.y, r.w, 90.0_f32.min(r.h), 12.0, alpha(hex(GOLD), 0.10));
}

fn gala_seite(ui: &mut Ui, v: &mut View, c: &mut Career, kats: &[&str], termine: (&str, &str)) {
    let t = crate::inhalt::t;
    let namen: Vec<&str> = kats.iter().map(|k| t(&format!("ui.gala.kurz.{k}"))).collect();
    tabs(ui, v, &namen);
    let kat = kats[v.tab.min(kats.len() - 1)];
    let tag = |k: &str| crate::regeln::f("ligapreise", &format!("termine.{k}")) as u32;
    let (tn, tg) = (tag(termine.0), tag(termine.1));
    let jahr = if c.world.day > tg { c.world.season + 1 } else { c.world.season };
    let h = hoehe() - Y0 - 16.0;
    // the hero: the trophy's name in gold, rays, the two dates
    let kopf = Rect::new(x0() + 4.0, Y0, cbreite(), 112.0);
    gala_karte(kopf);
    let zeit = get_time() as f32;
    let (sx, sy) = (kopf.x + kopf.w - 90.0, kopf.y + 56.0);
    for k in 0..20 { let a = zeit * 0.25 + k as f32 * std::f32::consts::TAU / 20.0; draw_line(sx + a.cos() * 30.0, sy + a.sin() * 30.0, sx + a.cos() * 52.0, sy + a.sin() * 52.0, 3.0, alpha(hex(GOLD), 0.5)); }
    draw_circle(sx, sy, 26.0, hex(GOLD));
    ui.text_center(&format!("{}", crate::date::START_YEAR + jahr as i32), sx, sy + 6.0, 14.0, hex(0x14110b));
    ui.title(&t(&format!("ui.gala.{kat}")).to_uppercase(), kopf.x + 24.0, kopf.y + 52.0, 34.0, hex(GOLD));
    let s_n = crate::date::long(jahr, tn);
    let s_g = crate::date::long(jahr, tg);
    ui.text(&crate::tf!("ui.preis.termine", s_n, s_g), kopf.x + 26.0, kopf.y + 84.0, 15.0, hex(0xEDE3C8));
    let y0 = Y0 + 124.0;
    let hh = h - 124.0;
    let lw = (cbreite() - 12.0) * 0.56;
    let (l, r) = (Rect::new(x0() + 4.0, y0, lw, hh), Rect::new(x0() + 16.0 + lw, y0, cbreite() - 12.0 - lw, hh));
    // left: the bookmakers' view of the next night (Fabian 24.11.: an estimate with odds, not a final order)
    gala_karte(l);
    let titel_l = if c.world.day <= tg { crate::tf!("ui.preis.favoriten", crate::preise::zeitraum(kat, jahr)) } else { crate::tf!("ui.preis.favoriten", crate::preise::zeitraum(kat, jahr)) };
    ui.text_semi(&titel_l, l.x + 18.0, l.y + 30.0, 18.0, hex(GOLD));
    let rennen: Vec<(usize, f32)> = rennen_heute(c, kat);
    if rennen.is_empty() { ui.text(t("ui.award.leer"), l.x + 18.0, l.y + 70.0, 15.0, hex(0xEDE3C8)); }
    if kat == "best_elf" { elf_zeichnen(ui, v, c, Rect::new(l.x + 10.0, l.y + 44.0, l.w - 20.0, l.h - 54.0), &rennen.iter().map(|x| x.0).collect::<Vec<_>>()); }
    else { rennen_zeichnen(ui, v, c, l, kat, &rennen); }
    // right: the vote once the night is over, else how the winner is chosen and what the experts say; past winners
    gala_karte(r);
    let kand = c.preis_kand.iter().find(|x| x.0 == kat).map(|x| x.1.clone()).unwrap_or_default();
    let vergeben = c.preise.iter().any(|p| p.art == kat && p.season == c.world.season) && c.world.day >= tg;
    let mut y = r.y + 30.0;
    if vergeben && !kand.is_empty() {
        ui.text_semi(t("ui.preis.ergebnis"), r.x + 18.0, y, 18.0, hex(GOLD));
        y += 14.0;
        let (erg, beispiele) = ergebnis_heute(c, kat, &kand);
        let zeile = ((r.h * 0.42) / erg.len().max(1) as f32).clamp(18.0, 26.0);
        for (i, &(w, pk)) in erg.iter().enumerate() {
            y += zeile;
            if y > r.y + r.h * 0.48 { break; }
            if i == 0 && kat != "best_elf" { rrect(r.x + 8.0, y - zeile + 5.0, r.w - 16.0, zeile - 2.0, 5.0, alpha(hex(GOLD), 0.25)); } // roh
            let (name, club) = wer(c, kat, w);
            let cl = &c.world.clubs[club];
            if club == c.own { rrect(r.x + 8.0, y - zeile + 5.0, r.w - 16.0, zeile - 2.0, 5.0, alpha(Theme::akzent(), 0.25)); }
            ui.club_crest(r.x + 24.0, y - 5.0, 7.0, cl.id, hex(cl.c1), &cl.short);
            ui.text(&ui.fit(&format!("{}. {name} ({})", i + 1, cl.short), 14.0, r.w - 110.0), r.x + 38.0, y, 14.0, hex(0xEDE3C8));
            if pk > 0 { ui.text_right(&crate::tf!("ui.preis.punkte", pk), r.x + r.w - 16.0, y, 13.0, hex(GOLD)); }
        }
        // how some voters chose, in words
        y = r.y + r.h * 0.5;
        ui.text_semi(t("ui.preis.stimmen"), r.x + 18.0, y, 15.0, hex(GOLD));
        for (wer_, p, grund) in beispiele {
            y += 20.0;
            let (name, _) = wer(c, kat, p);
            ui.text(&ui.fit(&format!("{wer_}: {name} · {}", t(&format!("ui.preis.grund.{grund}"))), 13.0, r.w - 36.0), r.x + 18.0, y, 13.0, hex(0xEDE3C8));
        }
    } else {
        ui.text_semi(t("ui.preis.so_gewaehlt"), r.x + 18.0, y, 18.0, hex(GOLD));
        let regel = if crate::preise::BALLON.iter().any(|x| x.0 == kat) { t("ui.preis.regel_ballon") } else { t("ui.preis.regel_best") };
        y += 10.0;
        for z in ui.wrap(regel, 14.0, r.w - 36.0).into_iter().take(4) { y += 19.0; ui.text(&z, r.x + 18.0, y, 14.0, hex(0xEDE3C8)); }
        y += 18.0;
        let woche = (c.world.day / 7) as usize;
        for (i, &(w, _)) in rennen.iter().take(2).enumerate() {
            if !ist_spieler_preis(kat) || kat == "best_elf" { break; }
            let pl = &c.world.players[w];
            let cn = c.world.clubs[pl.team.max(0) as usize].name.clone();
            let (apps, goals, note) = zahlen(c, w);
            let txt = crate::tf!(&format!("ui.award.stimme.{}.{}", stimmen_art(kat), (i + woche) % 4), pl.name.clone(), cn, goals, apps, dez2(note), i + 1);
            ui.player_photo(r.x + 34.0, y + 22.0, 18.0, pl.id, &pl.name.clone());
            let mut yy = y + 12.0;
            for z in ui.wrap(&txt, 14.0, r.w - 80.0).into_iter().take(3) { ui.text(&z, r.x + 62.0, yy, 14.0, hex(0xEDE3C8)); yy += 18.0; }
            y = yy.max(y + 50.0) + 8.0;
        }
    }
    // past winners
    let fruehere: Vec<crate::preise::Preis> = c.preistraeger(kat).into_iter().take(5).cloned().collect();
    let mut y = r.y + r.h * 0.74;
    ui.text_semi(t("ui.preis.fruehere"), r.x + 18.0, y, 16.0, hex(GOLD));
    if fruehere.is_empty() { ui.text(t("ui.preis.noch_keine"), r.x + 18.0, y + 24.0, 14.0, hex(0x9c9178)); }
    for p in fruehere.iter().filter(|p| kat != "best_elf" || p.season == fruehere[0].season).take(5) {
        y += 22.0;
        let (name, club) = match p.spieler { Some(s) => (c.world.players[s].name.clone(), p.club), None => wer(c, kat, p.club) };
        ui.text(&crate::preise::zeitraum(kat, p.season), r.x + 18.0, y, 13.0, hex(0x9c9178));
        ui.text(&ui.fit(&format!("{name} · {}", c.world.clubs[club].short), 14.0, r.w - 110.0), r.x + 90.0, y, 14.0, hex(0xEDE3C8));
    }
}

/// The vote of the night and a few voters' reasons, once per award and day.
fn ergebnis_heute(c: &Career, kat: &str, kand: &[(usize, f32)]) -> (Vec<(usize, u32)>, Vec<(String, usize, &'static str)>) {
    static E: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<(String, u32), (Vec<(usize, u32)>, Vec<(String, usize, &'static str)>)>>> = std::sync::LazyLock::new(Default::default);
    let mut g = E.lock().unwrap_or_else(|e| e.into_inner());
    let key = (kat.to_string(), c.abs());
    if let Some(x) = g.get(&key) { return x.clone(); }
    let s = c.world.season;
    let erg = if kat == "mueller" || kat == "best_fairplay" || kat == "best_elf" { kand.iter().map(|x| (x.0, 0)).collect() } else { c.preis_abstimmung(kat, kand, s) }; // roh
    let bsp = if ist_spieler_preis(kat) && kat != "best_elf" && kat != "mueller" { c.wahl_beispiele(kat, kand, s, 5) } else { vec![] }; // roh
    g.insert(key.clone(), (erg, bsp));
    g[&key].clone()
}
/// The race of today, worked out once per award and day (the world-wide estimate is too dear for every frame).
fn rennen_heute(c: &Career, kat: &str) -> Vec<(usize, f32)> {
    let mut g = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if g.0 != c.abs() { *g = (c.abs(), Default::default(), Default::default()); }
    if let Some(v) = g.1.get(kat) { return v.clone(); }
    let v: Vec<(usize, f32)> = c.vorschau(kat).into_iter().take(20).collect();
    g.1.insert(kat.to_string(), v.clone());
    v
}
/// Games, goals and grade of a player for the awards (estimated where his league has no single figures), per day.
fn zahlen(c: &Career, p: usize) -> (u16, u16, f32) {
    let mut g = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if g.0 != c.abs() { *g = (c.abs(), Default::default(), Default::default()); }
    if let Some(&z) = g.2.get(&p) { return z; }
    let ppg = c.punkte_je_spiel();
    let z = crate::ehrungen::award_zahlen(&c.world, p, &ppg).unwrap_or((0, 0, 0.0));
    g.2.insert(p, z);
    z
}
type Zwischen = (u32, std::collections::HashMap<String, Vec<(usize, f32)>>, std::collections::HashMap<usize, (u16, u16, f32)>);
static CACHE: std::sync::LazyLock<std::sync::Mutex<Zwischen>> = std::sync::LazyLock::new(|| std::sync::Mutex::new((u32::MAX, Default::default(), Default::default())));
/// The name shown for a winner or nominee: a player, a coach (by his club) or a club.
fn wer(c: &Career, kat: &str, w: usize) -> (String, usize) {
    if ist_spieler_preis(kat) { (c.world.players[w].name.clone(), c.world.players[w].team.max(0) as usize) }
    else if kat == "cruyff" || kat == "best_trainer" { (trainer_name(c, w), w) }
    else { (c.world.clubs[w].name.clone(), w) }
}
fn stimmen_art(kat: &str) -> &str { match kat { "best_spieler" => "ballon", "best_torwart" => "yashin", k => k } } // roh

/// The race: a podium for the first three (big photos or crests), then rows down to 20.
fn rennen_zeichnen(ui: &mut Ui, v: &mut View, c: &Career, l: Rect, kat: &str, rennen: &[(usize, f32)]) {
    let quoten = crate::preise::quoten(rennen);
    let quote = |i: usize| quoten.get(i).map(|q| format!("{q:.2}")).unwrap_or_default();
    let bw = (l.w - 56.0) / 3.0;
    for (j, &(w, _)) in rennen.iter().take(3).enumerate() {
        let pos = [1usize, 0, 2][j];
        let hx = l.x + 18.0 + pos as f32 * (bw + 10.0);
        let hy = l.y + 50.0 + if j == 0 { 0.0 } else { 18.0 };
        let hh = 150.0 - if j == 0 { 0.0 } else { 18.0 };
        rrect(hx, hy, bw, hh, 10.0, alpha(medaille(j), 0.14));
        rrect_line(hx, hy, bw, hh, 10.0, medaille(j));
        let (name, club) = wer(c, kat, w);
        let cl = &c.world.clubs[club];
        let mx = hx + bw / 2.0;
        if ist_spieler_preis(kat) { ui.player_photo(mx, hy + 48.0, 36.0, c.world.players[w].id, &name); ui.club_crest(mx + 30.0, hy + 72.0, 11.0, cl.id, hex(cl.c1), &cl.short); }
        else { ui.club_crest(mx, hy + 48.0, 34.0, cl.id, hex(cl.c1), &cl.short); }
        draw_circle(hx + 18.0, hy + 18.0, 12.0, medaille(j));
        ui.text_right(&quote(j), hx + bw - 10.0, hy + 24.0, 18.0, medaille(j));
        ui.text_center(&format!("{}", j + 1), hx + 18.0, hy + 23.0, 15.0, hex(0x14110b));
        if club == c.own { rrect_line(hx - 3.0, hy - 3.0, bw + 6.0, hh + 6.0, 12.0, Theme::akzent()); }
        ui.text_center(&ui.fit(&name, 15.0, bw - 12.0), mx, hy + hh - 30.0, 15.0, hex(0xEDE3C8));
        ui.text_center(&ui.fit(&kennzahl(c, kat, w), 13.0, bw - 12.0), mx, hy + hh - 10.0, 13.0, medaille(j));
        if ui.hit(Rect::new(hx, hy, bw, hh)) && ist_spieler_preis(kat) { v.modal = Modal::Player(w); }
    }
    let start = l.y + 226.0;
    let zeile = ((l.y + l.h - start) / 17.0).clamp(18.0, 26.0);
    for (i, &(w, _)) in rennen.iter().enumerate().skip(3) {
        let y = start + (i - 3) as f32 * zeile;
        if y > l.y + l.h - 8.0 { break; }
        let (name, club) = wer(c, kat, w);
        let cl = &c.world.clubs[club];
        let rr = Rect::new(l.x + 8.0, y - zeile + 6.0, l.w - 16.0, zeile - 2.0);
        if club == c.own { rrect(rr.x, rr.y, rr.w, rr.h, 5.0, alpha(Theme::akzent(), 0.22)); }
        ui.text(&format!("{}.", i + 1), l.x + 18.0, y, 14.0, hex(0x9c9178));
        ui.club_crest(l.x + 58.0, y - 5.0, 8.0, cl.id, hex(cl.c1), &cl.short);
        ui.text(&ui.fit(&format!("{name} ({})", cl.short), 14.0, l.w - 330.0), l.x + 74.0, y, 14.0, hex(0xEDE3C8));
        ui.text_right(&kennzahl(c, kat, w), l.x + l.w - 86.0, y, 13.0, hex(0x9c9178));
        ui.text_right(&quote(i), l.x + l.w - 18.0, y, 15.0, hex(GOLD));
        if ui.hit(rr) && ist_spieler_preis(kat) { v.modal = Modal::Player(w); }
    }
}
/// The figures behind a place: games, goals, grade for players; points per game for clubs; cards for fair play.
fn kennzahl(c: &Career, kat: &str, w: usize) -> String {
    if ist_spieler_preis(kat) { let (a, g, n) = zahlen(c, w); return crate::tf!("ui.preis.kennzahl", a, g, dez2(n)); }
    if kat == "best_fairplay" { let l = c.world.clubs[w].league; let d = c.world.leagues[l].teams.iter().any(|&t| c.world.clubs[t].squad.iter().any(|&p| c.world.players[p].in_liga(l).rn > 0)); crate::tf!("ui.preis.karten", dez2(c.gelbe_je_spiel(w, l, d))) } else { crate::tf!("ui.preis.titel_zahl", c.world.titles.iter().filter(|t| t.season == c.world.season && t.club == w).count()) } // roh
}
/// FIFA 11 as a 4-3-3 on the dark card: keeper at the bottom, forwards at the top.
fn elf_zeichnen(ui: &mut Ui, v: &mut View, c: &Career, r: Rect, elf: &[usize]) {
    let reihen: [&[usize]; 4] = [&[0], &[3, 1, 2, 4], &[5, 6, 7], &[8, 9, 10]];
    draw_rectangle_lines(r.x + 20.0, r.y + 10.0, r.w - 40.0, r.h - 20.0, 2.0, alpha(hex(GOLD), 0.35));
    draw_line(r.x + 20.0, r.y + r.h / 2.0, r.x + r.w - 20.0, r.y + r.h / 2.0, 2.0, alpha(hex(GOLD), 0.35));
    for (ri, reihe) in reihen.iter().enumerate() {
        let y = r.y + r.h - 50.0 - ri as f32 * (r.h - 90.0) / 3.0;
        for (k, &i) in reihe.iter().enumerate() {
            let Some(&p) = elf.get(i) else { continue };
            let x = r.x + r.w * (k as f32 + 1.0) / (reihe.len() as f32 + 1.0);
            let pl = &c.world.players[p];
            ui.player_photo(x, y - 8.0, 24.0, pl.id, &pl.name.clone());
            if pl.team == c.own as i32 { draw_circle_lines(x, y - 8.0, 27.0, 3.0, Theme::akzent()); }
            ui.text_center(&ui.fit(&pl.name, 13.0, 120.0), x, y + 30.0, 13.0, hex(0xEDE3C8));
            if ui.hit(Rect::new(x - 30.0, y - 36.0, 60.0, 70.0)) { v.modal = Modal::Player(p); }
        }
    }
}

/// Competitions › League awards (Fabian 24.11.): every league's own awards by their real names, the race today and
/// last season's winners; given at the season's end.
pub(super) fn liga_preise_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let gruppen: Vec<(String, Vec<usize>)> = crate::pages::ligen_gruppen(c, |_| true);
    let l0 = v.table_league.filter(|&x| x < 1000).unwrap_or(c.league());
    let felder = crate::pages::wettbewerb_felder(c, &gruppen, l0, Rect::new(x0() + 4.0 + cbreite() - 480.0, Y0 - 40.0, 470.0, 36.0));
    if let Some((f, i)) = crate::pages::auswahl_vorher(ui, v, &felder) { v.table_league = Some(crate::pages::wahl_umsetzen(&gruppen, l0, f, i)); }
    let l = v.table_league.filter(|&x| x < 1000).unwrap_or(c.league());
    let lgk = c.world.leagues[l].key.clone();
    let lk = crate::pages::wettbewerb_look(c, &lgk, 9, &c.world.leagues[l].land.clone());
    let _ = crate::pages::wettbewerb_banner(ui, Rect::new(x0() + 4.0, Y0 - 44.0, cbreite(), 46.0), &lk, c.world.cids.get(&lgk).copied(), &c.world.leagues[l].name.clone());
    let preise = crate::preise::liga_preisliste(&crate::preise::liga_land(&c.world, l));
    let anteil = (c.world.day as f32 / crate::world::LEAGUE_END as f32).clamp(0.1, 1.0);
    let n = preise.len().max(1);
    let spalten = if n > 4 { 3 } else { 2.min(n) };
    let zeilen = n.div_ceil(spalten);
    let w = (cbreite() - 12.0 * (spalten as f32 - 1.0)) / spalten as f32;
    let hh = (hoehe() - Y0 - 16.0 - 12.0 * (zeilen as f32 - 1.0)) / zeilen as f32;
    for (k, a) in preise.iter().enumerate() {
        let r = Rect::new(x0() + 4.0 + (k % spalten) as f32 * (w + 12.0), Y0 + 8.0 + (k / spalten) as f32 * (hh + 12.0), w, hh - 8.0);
        gala_karte(r);
        ui.title(&ui.fit(&a.name(), 20.0, r.w - 30.0), r.x + 16.0, r.y + 32.0, 20.0, hex(GOLD));
        ui.text(&ui.fit(t(&format!("ui.ligapreis.erkl.{}", a.art)), 13.0, r.w - 30.0), r.x + 16.0, r.y + 52.0, 13.0, hex(0x9c9178));
        let rennen = c.liga_rennen(l, a, anteil);
        let zahl_von = |wer_: usize, wert: f32, spieler: bool| if a.art == "tore" { format!("{}", wert as u32) } else if spieler { dez2(c.world.players[wer_].in_liga(l).note()) } else { String::new() }; // roh
        // the first three on a podium (Fabian 24.11.), the rest as rows
        let bw = (r.w - 48.0) / 3.0;
        for (j, &(wer_, wert, spieler)) in rennen.iter().take(3).enumerate() {
            let pos = [1usize, 0, 2][j];
            let hx = r.x + 16.0 + pos as f32 * (bw + 8.0);
            let hy = r.y + 66.0 + if j == 0 { 0.0 } else { 12.0 };
            let ph = 112.0 - if j == 0 { 0.0 } else { 12.0 };
            rrect(hx, hy, bw, ph, 8.0, alpha(medaille(j), 0.14));
            rrect_line(hx, hy, bw, ph, 8.0, medaille(j));
            let (name, club) = if spieler { (c.world.players[wer_].name.clone(), c.world.players[wer_].team.max(0) as usize) } else { (trainer_name(c, wer_), wer_) };
            let cl = &c.world.clubs[club];
            let mx = hx + bw / 2.0;
            if spieler { ui.player_photo(mx, hy + 32.0, 22.0, c.world.players[wer_].id, &name); ui.club_crest(mx + 20.0, hy + 46.0, 8.0, cl.id, hex(cl.c1), &cl.short); }
            else { ui.club_crest(mx, hy + 32.0, 22.0, cl.id, hex(cl.c1), &cl.short); }
            draw_circle(hx + 13.0, hy + 13.0, 9.0, medaille(j));
            ui.text_center(&format!("{}", j + 1), hx + 13.0, hy + 17.0, 12.0, hex(0x14110b));
            if club == c.own { rrect_line(hx - 3.0, hy - 3.0, bw + 6.0, ph + 6.0, 10.0, Theme::akzent()); }
            ui.text_center(&ui.fit(&name, 13.0, bw - 8.0), mx, hy + ph - 24.0, 13.0, hex(0xEDE3C8));
            ui.text_center(&zahl_von(wer_, wert, spieler), mx, hy + ph - 7.0, 13.0, medaille(j));
            if spieler && ui.hit(Rect::new(hx, hy, bw, ph)) { v.modal = Modal::Player(wer_); }
        }
        let mut y = r.y + 206.0;
        for (i, &(wer_, wert, spieler)) in rennen.iter().enumerate().skip(3).take(((r.h - 236.0) / 22.0).max(0.0) as usize) {
            let (name, club) = if spieler { (c.world.players[wer_].name.clone(), c.world.players[wer_].team.max(0) as usize) } else { (trainer_name(c, wer_), wer_) };
            let cl = &c.world.clubs[club];
            ui.text(&format!("{}.", i + 1), r.x + 16.0, y, 14.0, hex(0x9c9178));
            ui.club_crest(r.x + 48.0, y - 5.0, 8.0, cl.id, hex(cl.c1), &cl.short);
            if club == c.own { rrect(r.x + 8.0, y - 17.0, r.w - 16.0, 22.0, 5.0, alpha(Theme::akzent(), 0.25)); }
            ui.text(&ui.fit(&name, 14.0, r.w - 140.0), r.x + 62.0, y, 14.0, hex(0xEDE3C8));
            ui.text_right(&zahl_von(wer_, wert, spieler), r.x + r.w - 16.0, y, 14.0, hex(GOLD));
            y += 22.0;
        }
        if rennen.is_empty() { ui.text(t("ui.award.leer"), r.x + 16.0, y, 13.0, hex(0x9c9178)); }
        // last winner
        let art = format!("liga:{l}:{}", a.id);
        if let Some(p) = c.preistraeger(&art).first() {
            let name = match p.spieler { Some(s) => c.world.players[s].name.clone(), None => trainer_name(c, p.club) };
            ui.text(&ui.fit(&crate::tf!("ui.preis.zuletzt", crate::date::season_label(p.season), name), 13.0, r.w - 30.0), r.x + 16.0, r.y + r.h - 14.0, 13.0, hex(0x9c9178));
        }
    }
}

/// Competitions › Rankings of the clubs and countries (Fabian 24.11.): the UEFA five-year ranking of the countries and
/// the clubs, and a world club ranking by strength.
pub(super) fn uefa_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    tabs(ui, v, &[t("ui.uefa.laender"), t("ui.uefa.vereine"), t("ui.uefa.welt")]);
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(r);
    let zeile = 26.0;
    let n = ((r.h - 70.0) / zeile) as usize;
    let start = scroll_rows(ui, v, r, 200, n);
    match v.tab {
        0 => {
            let liste = crate::preise::uefa_laender(&c.world);
            let jahr = crate::date::START_YEAR + c.world.season as i32;
            let kopf: Vec<String> = (0..5).map(|k| format!("{}/{:02}", jahr - 4 + k as i32, (jahr - 3 + k as i32) % 100)).collect();
            ui.text(t("ui.uefa.land"), r.x + 60.0, r.y + 30.0, 13.0, Theme::gedaempft());
            for (k, h) in kopf.iter().enumerate() { ui.text_right(h, r.x + r.w * (0.45 + k as f32 * 0.09), r.y + 30.0, 13.0, Theme::gedaempft()); }
            ui.text_right(t("ui.uefa.summe"), r.x + r.w - 20.0, r.y + 30.0, 13.0, Theme::gedaempft());
            let eigen = crate::cups::country(&c.world.leagues[c.league()]).unwrap_or_default();
            for (i, (land, je, sum)) in liste.iter().enumerate().skip(start).take(n) {
                let y = r.y + 60.0 + (i - start) as f32 * zeile;
                if *land == eigen { rrect(r.x + 8.0, y - 18.0, r.w - 16.0, zeile - 2.0, 5.0, alpha(Theme::akzent(), 0.16)); }
                ui.text(&format!("{}.", i + 1), r.x + 20.0, y, 14.0, if i < 4 { Theme::gold() } else { Theme::text2() });
                ui.text(land, r.x + 60.0, y, 15.0, Theme::text());
                for (k, x) in je.iter().enumerate() { ui.text_right(&format!("{x:.1}"), r.x + r.w * (0.45 + k as f32 * 0.09), y, 14.0, Theme::text2()); }
                ui.text_right(&format!("{sum:.1}"), r.x + r.w - 20.0, y, 15.0, Theme::text());
            }
        }
        _ => {
            let liste: Vec<(usize, f32)> = if v.tab == 1 { crate::preise::uefa_vereine(&c.world) } else {
                let mut l: Vec<(usize, f32)> = (0..c.world.clubs.len()).filter(|&k| c.world.erste_von(k).is_none() && c.world.clubs[k].league < c.world.leagues.len()).map(|k| (k, c.world.clubs[k].level)).collect();
                l.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                l.truncate(200);
                l
            };
            ui.text(if v.tab == 1 { t("ui.uefa.verein_hinweis") } else { t("ui.uefa.welt_hinweis") }, r.x + 20.0, r.y + 30.0, 13.0, Theme::gedaempft());
            for (i, &(k, w)) in liste.iter().enumerate().skip(start).take(n) {
                let y = r.y + 60.0 + (i - start) as f32 * zeile;
                let cl = &c.world.clubs[k];
                let rr = Rect::new(r.x + 8.0, y - 18.0, r.w - 16.0, zeile - 2.0);
                if k == c.own { rrect(rr.x, rr.y, rr.w, rr.h, 5.0, alpha(Theme::akzent(), 0.16)); }
                ui.text(&format!("{}.", i + 1), r.x + 20.0, y, 14.0, if i < 3 { Theme::gold() } else { Theme::text2() });
                ui.club_crest(r.x + 70.0, y - 5.0, 9.0, cl.id, hex(cl.c1), &cl.short);
                ui.text(&ui.fit(&cl.name, 15.0, r.w * 0.4), r.x + 90.0, y, 15.0, Theme::text());
                ui.text(&ui.fit(&c.world.leagues[cl.league].name, 13.0, r.w * 0.3), r.x + r.w * 0.5, y, 13.0, Theme::text2());
                ui.text_right(&format!("{w:.1}"), r.x + r.w - 20.0, y, 15.0, Theme::text());
                if ui.hit(rr) { v.kontext = None; v.sel_club = Some(k); v.go(Page::Vereine); }
            }
        }
    }
}
