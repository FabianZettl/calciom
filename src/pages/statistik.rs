use super::*;

/// Tab „Entwicklung“: history per group with the reasons of the last weeks, phase, focus, traits and progress paths (entwicklung.rs).
/// One row of the statistics tab: text key, how the value is made, fewer is better.
#[derive(Clone, Copy)]
enum StatWert { Zahl(usize), Tausendstel(usize), Quote(usize, usize), JeSchuss, ToreMinusXg, PsxgMinusGt, Einfluss }
const STAT_ZEILEN: [(&str, &[(&str, StatWert, bool)]); 4] = [
    (crate::id("angriff"), &[(crate::id("einfluss"), StatWert::Einfluss, false), (crate::id("tore"), StatWert::Zahl(crate::statistik::TORE), false), (crate::id("xg"), StatWert::Tausendstel(crate::statistik::XG), false), ("tore_minus_xg", StatWert::ToreMinusXg, false),
        (crate::id("schuesse"), StatWert::Zahl(crate::statistik::SCHUESSE), false), ("aufs_tor", StatWert::Zahl(crate::statistik::AUFS_TOR), false), ("xg_je_schuss", StatWert::JeSchuss, false),
        (crate::id("vorlagen"), StatWert::Zahl(crate::statistik::VORLAGEN), false), (crate::id("xa"), StatWert::Tausendstel(crate::statistik::XA), false), (crate::id("schluesselpaesse"), StatWert::Zahl(crate::statistik::SCHLUESSEL), false),
        (crate::id("strafraum"), StatWert::Zahl(crate::statistik::STRAFRAUM), false)]),
    (crate::id("aufbau"), &[(crate::id("paesse"), StatWert::Zahl(crate::statistik::PAESSE), false), (crate::id("passquote"), StatWert::Quote(crate::statistik::PAESSE_OK, crate::statistik::PAESSE), false),
        ("prog_paesse", StatWert::Zahl(crate::statistik::PROG_PAESSE), false), ("prog_laeufe", StatWert::Zahl(crate::statistik::PROG_LAEUFE), false),
        (crate::id("dribblings"), StatWert::Zahl(crate::statistik::DRIBB), false), ("dribbling_quote", StatWert::Quote(crate::statistik::DRIBB_OK, crate::statistik::DRIBB), false),
        (crate::id("flanken"), StatWert::Zahl(crate::statistik::FLANKEN), false), ("flanken_quote", StatWert::Quote(crate::statistik::FLANKEN_OK, crate::statistik::FLANKEN), false),
        (crate::id("ballverluste"), StatWert::Zahl(crate::statistik::BALLVERL), true)]),
    (crate::id("abwehr"), &[(crate::id("tacklings"), StatWert::Zahl(crate::statistik::TACKL), false), ("tackling_quote", StatWert::Quote(crate::statistik::TACKL_OK, crate::statistik::TACKL), false),
        (crate::id("abgefangen"), StatWert::Zahl(crate::statistik::ABGEF), false), (crate::id("blocks"), StatWert::Zahl(crate::statistik::BLOCKS), false), (crate::id("klaerungen"), StatWert::Zahl(crate::statistik::KLAER), false),
        (crate::id("kopfball"), StatWert::Zahl(crate::statistik::KOPF), false), ("kopfball_quote", StatWert::Quote(crate::statistik::KOPF_OK, crate::statistik::KOPF), false),
        (crate::id("fouls"), StatWert::Zahl(crate::statistik::FOULS), true), (crate::id("gelb"), StatWert::Zahl(crate::statistik::GELB), true), (crate::id("rot"), StatWert::Zahl(crate::statistik::ROT), true)]),
    (crate::id("torwart"), &[(crate::id("einfluss"), StatWert::Einfluss, false), (crate::id("gegentore"), StatWert::Zahl(crate::statistik::GEGENTORE), true), (crate::id("paraden"), StatWert::Zahl(crate::statistik::PARADEN), false),
        ("paraden_quote", StatWert::Quote(crate::statistik::PARADEN, crate::statistik::AUFS_TOR), false), ("psxg_minus_gt", StatWert::PsxgMinusGt, false)]),
];
impl StatWert {
    /// The season value and whether it can be shown per 90 (counts) — quotes and per-shot values stay as they are.
    fn wert(self, s: &crate::statistik::Stat) -> Option<(f32, bool)> {
        use crate::statistik::get;
        let g = |i| get(s, i) as f32;
        Some(match self {
            StatWert::Zahl(i) => (g(i), true),
            StatWert::Tausendstel(i) => (g(i) / 1000.0, true),
            // quotes only from 30 attempts on (Konzept 6.1); the keepers' save rate counts the shots on target they faced
            StatWert::Quote(a, b) => {
                let n = if a == crate::statistik::PARADEN { g(crate::statistik::PARADEN) + g(crate::statistik::GEGENTORE) } else { g(b) };
                if n < if a == crate::statistik::PARADEN { 15.0 } else { 30.0 } { return None; }
                (g(a) / n * 100.0, false)
            }
            StatWert::JeSchuss => { if g(crate::statistik::SCHUESSE) < 10.0 { return None; } (g(crate::statistik::XG) / 1000.0 / g(crate::statistik::SCHUESSE), false) }
            StatWert::ToreMinusXg => (g(crate::statistik::TORE) - g(crate::statistik::XG) / 1000.0, false),
            StatWert::PsxgMinusGt => (g(crate::statistik::PSXG) / 1000.0 - g(crate::statistik::GEGENTORE), false),
            StatWert::Einfluss => (crate::statistik::einfluss90(s)?, false),
        })
    }
}
impl StatWert {
    /// The value as compared: counts per 90 (from 90 minutes on), the rest as they are.
    fn p90(self, s: &crate::statistik::Stat) -> Option<f32> {
        let m = crate::statistik::get(s, crate::statistik::MIN) as f32;
        let (v, j) = self.wert(s)?;
        if !j { return Some(v); }
        (m >= 90.0).then(|| v * 90.0 / m)
    }
}
/// Percentile 0–100 of `eigen` among `werte` (`weniger`: fewer is better): the share he is better than, ties half.
fn pz_von(werte: &[f32], eigen: f32, weniger: bool) -> f32 {
    let unter = werte.iter().filter(|&&x| if weniger { x > eigen } else { x < eigen }).count() as f32 + 0.5 * werte.iter().filter(|&&x| x == eigen).count() as f32;
    (unter / werte.len().max(1) as f32 * 100.0).clamp(0.0, 100.0)
}
/// The open ranking window: player * 1024 + area * 32 + row + 1, 0 when closed.
static STAT_FENSTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static STAT_VORSAISON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static STAT_UMGERECHNET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static STAT_AEHNLICH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static STAT_RAD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn stat_rad_an() { STAT_RAD.store(true, std::sync::atomic::Ordering::Relaxed); }
pub fn stat_fenster_offen(p: usize) -> bool { let f = STAT_FENSTER.load(std::sync::atomic::Ordering::Relaxed); f > 0 && (f - 1) / 1024 == p }
pub fn stat_fenster_auf(p: usize, b: usize, z: usize) { STAT_FENSTER.store(p * 1024 + b * 32 + z + 1, std::sync::atomic::Ordering::Relaxed); }

/// Statistik Teil 10.4: the percentile wheel. One wedge per number, its length the percentile against the comparison
/// group, coloured by area; rings at 25/50/75.
fn perzentil_rad(ui: &mut Ui, r: Rect, werte: &[(String, usize, f32, usize)]) -> Option<usize> {
    let mut geklickt = None;
    let n = werte.len().max(1);
    let (cx, cy) = (r.x + r.w / 2.0, r.y + 40.0 + (r.h - 40.0) / 2.0);
    let rmax = ((r.w * 0.8).min(r.h - 130.0) / 2.0).max(60.0);
    // fixed area colours, so the wheel reads the same in every club colour
    let farbe = |b: usize| [hex(0xE0564A), hex(0x3B8FDA), hex(0x2EA866), hex(0xD9A520)][b.min(3)];
    let keil = |a0: f32, a1: f32, rad: f32, col: Color| {
        let schritte = 10;
        for k in 0..schritte {
            let (u, v) = (a0 + (a1 - a0) * k as f32 / schritte as f32, a0 + (a1 - a0) * (k + 1) as f32 / schritte as f32);
            draw_triangle(vec2(cx, cy), vec2(cx + rad * u.cos(), cy + rad * u.sin()), vec2(cx + rad * v.cos(), cy + rad * v.sin()), col);
        }
    };
    let schritt = std::f32::consts::TAU / n as f32;
    for (i, (name, b, pz, _)) in werte.iter().enumerate() {
        let a0 = -std::f32::consts::FRAC_PI_2 + i as f32 * schritt + 0.02;
        let a1 = a0 + schritt - 0.04;
        keil(a0, a1, rmax, alpha(Theme::text(), 0.06));
        keil(a0, a1, rmax * (pz / 100.0).clamp(0.04, 1.0), alpha(farbe(*b), 0.85));
        let am = (a0 + a1) / 2.0;
        let (lx, ly) = (cx + (rmax + 34.0) * am.cos(), cy + (rmax + 30.0) * am.sin());
        ui.text_center(&ui.fit(name, 13.0, 130.0), lx, ly, 13.0, Theme::text());
        ui.text_center(&format!("{pz:.0}"), lx, ly + 16.0, 13.0, farbe(*b));
        // a click on the name opens the league ranking of this number
        let lr = Rect::new(lx - 70.0, ly - 16.0, 140.0, 38.0);
        if ui.hover(lr) { rrect_line(lr.x, lr.y, lr.w, lr.h, 6.0, alpha(farbe(*b), 0.6)); }
        if ui.hit(lr) { geklickt = Some(i); }
    }
    for q in [0.25, 0.5, 0.75, 1.0] { draw_circle_lines(cx, cy, rmax * q, 1.0, alpha(Theme::text(), 0.15)); }
    geklickt
}

/// Statistics tab (konzepte/spielerstatistiken.md Teil 10.1): context, then per area the season value, per 90 and the
/// percentile against the same position group of his league. Extended numbers for the own league, own players and players
/// the scouts know well; others show only the basics.
pub(super) fn statistik_reiter(ui: &mut Ui, v: &mut View, c: &Career, p: usize, own: bool, cy: f32, ch: f32) {
    use crate::statistik::{get, MIN, SPIELE};
    let pl = &c.world.players[p];
    let vor = STAT_VORSAISON.load(std::sync::atomic::Ordering::Relaxed);
    let liga = if pl.team >= 0 { c.world.clubs[pl.team as usize].league } else { usize::MAX };
    // a summary-level league has no match trace: projected numbers, marked so (Konzept 5.2)
    let hoch = !vor && pl.profil.saison.is_empty() && liga < c.world.leagues.len() && c.world.leagues[liga].detail == 2;
    let geschaetzt = if hoch { crate::statistik::hochrechnen(&c.world, p) } else { vec![] };
    let vor_geschaetzt = vor && pl.profil.vorsaison.is_empty() && c.world.season == 0;
    let geschaetzt = if vor_geschaetzt { crate::statistik::vorsaison_geschaetzt(&c.world, p) } else { geschaetzt };
    let st = if vor_geschaetzt || hoch { &geschaetzt } else if vor { &pl.profil.vorsaison } else { &pl.profil.saison };
    // the table shows the league figures, the same the percentile ranks (Fabian 24.11.: value and percentile disagreed,
    // the table had all competitions); results-only leagues and last season keep their own figures
    let st_liga: crate::statistik::Stat = if vor || hoch { st.clone() } else { pl.in_liga(liga).st };
    let st: &crate::statistik::Stat = if liga < c.world.leagues.len() && !vor && !hoch { &st_liga } else { st };
    let erweitert = own || crate::pages::markt_erweitert(c, p);
    let kopf = Rect::new(x0() + 4.0, cy, cbreite(), 64.0);
    ui.card(kopf);
    let min = get(st, MIN);
    let ampel = if min >= 1800 { Theme::gut() } else if min >= 900 { Theme::achtung() } else { Theme::problem() };
    draw_circle(kopf.x + 22.0, kopf.y + 24.0, 6.0, ampel);
    // the two lines stop before the switches on the right (Fabian 24.11.: the pills lay over the minutes)
    let platz = (kopf.w - 1010.0 - 52.0).max(120.0);
    let kontext = crate::tf!("ui.stat.kontext", min, get(st, SPIELE), if liga < c.world.leagues.len() { c.world.leagues[liga].name.clone() } else { "–".into() });
    ui.text(&ui.fit(&kontext, 16.0, platz), kopf.x + 36.0, kopf.y + 29.0, 16.0, Theme::text());
    let quelle = crate::inhalt::t(if vor_geschaetzt { "ui.stat.vorsaison_geschaetzt" } else if hoch { "ui.stat.quelle_hochgerechnet" } else if min == 0 { "ui.stat.keine_daten" } else if min < 900 { "ui.stat.wackeln" } else { "ui.stat.quelle_simuliert" });
    ui.text(&ui.fit(quelle, 13.0, platz), kopf.x + 36.0, kopf.y + 52.0, 13.0, Theme::gedaempft());
    if ui.hover(Rect::new(kopf.x + 30.0, kopf.y + 10.0, platz, 50.0)) { *ui.tooltip.borrow_mut() = Some(format!("{kontext}\n{quelle}")); }
    if let Some(i) = ui.segment(Rect::new(kopf.x + kopf.w - 300.0, kopf.y + 14.0, 286.0, 32.0), &[crate::inhalt::t("ui.saison.diese_saison"), crate::inhalt::t("ui.stat.vorsaison")], vor as usize) {
        STAT_VORSAISON.store(i == 1, std::sync::atomic::Ordering::Relaxed);
    }
    // Konzept 6.3: "what would that be with us?" for a player of another league
    let fremd = liga < c.world.leagues.len() && liga != c.league();
    let um = fremd && STAT_UMGERECHNET.load(std::sync::atomic::Ordering::Relaxed);
    if fremd {
        let d = crate::statistik::liga_staerke(&c.world, liga) - crate::statistik::liga_staerke(&c.world, c.league());
        let br = Rect::new(kopf.x + kopf.w - 600.0, kopf.y + 14.0, 286.0, 32.0);
        if ui.button(br, &crate::tf!("ui.stat.umrechnen", format!("{:+.0}", d).replace('-', "−")), um) { STAT_UMGERECHNET.store(!um, std::sync::atomic::Ordering::Relaxed); }
    }
    // Konzept 7.3: "Finde einen wie ihn" — the look-alikes by numbers among players whose numbers we see
    let aehnlich = STAT_AEHNLICH.load(std::sync::atomic::Ordering::Relaxed);
    // one switch for the three views: numbers, percentile wheel, look-alikes
    let rad = STAT_RAD.load(std::sync::atomic::Ordering::Relaxed);
    let ansicht = if aehnlich { 2 } else { rad as usize };
    if let Some(i) = ui.segment(Rect::new(kopf.x + kopf.w - 1010.0, kopf.y + 14.0, 396.0, 32.0), &[crate::inhalt::t("ui.stat.zahlen"), crate::inhalt::t("ui.stat.rad"), crate::inhalt::t("ui.stat.wie_ihn")], ansicht) {
        STAT_RAD.store(i == 1, std::sync::atomic::Ordering::Relaxed);
        STAT_AEHNLICH.store(i == 2, std::sync::atomic::Ordering::Relaxed);
    }
    if aehnlich {
        STAT_FENSTER.store(0, std::sync::atomic::Ordering::Relaxed);
        let r = Rect::new(x0() + 4.0, cy + 76.0, cbreite(), ch - 76.0);
        ui.card_titled(r, &crate::tf!("ui.stat.wie_titel", pl.name.clone()), Theme::akzent());
        // computed once per player and day, not every frame (Fabian 24.11.: the page hung and was hard to leave)
        static CACHE: std::sync::Mutex<Option<((usize, u32), Vec<(usize, f32)>)>> = std::sync::Mutex::new(None);
        let key = (p, c.abs());
        let mut g = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if g.as_ref().map(|x| x.0) != Some(key) {
            let kand: Vec<usize> = (0..c.world.players.len()).filter(|&q| !c.gehoert_uns(q) && crate::pages::markt_erweitert(c, q)).collect();
            *g = Some((key, crate::statistik::aehnliche(&c.world, p, &kand, 15)));
        }
        let liste = g.as_ref().map(|x| x.1.clone()).unwrap_or_default();
        drop(g);
        if liste.is_empty() { ui.paragraph(crate::inhalt::t("ui.stat.wie_leer"), r.x + 14.0, r.y + 60.0, 15.0, r.w - 28.0, Theme::text2()); return; }
        // the best three as cards (Fabian 24.11.: "still loveless"): photo, club, age, value, the similarity, and his key
        // numbers per 90 next to the model's, so the likeness shows
        let gr = crate::scoutbericht::gruppe(&pl.pos);
        let kern = crate::statistik::kernzahlen(gr);
        let (vorbild, _) = crate::statistik::saison_von(&c.world, p);
        let kw = (r.w - 28.0 - 24.0) / 3.0;
        let kh = 300.0f32.min(r.h * 0.6);
        for (k, &(q, sim)) in liste.iter().take(3).enumerate() {
            let pq = &c.world.players[q];
            let kr = Rect::new(r.x + 14.0 + k as f32 * (kw + 12.0), r.y + 54.0, kw, kh);
            let club = if pq.team >= 0 { pq.team as usize } else { c.own };
            let (farbe, _) = crate::pages::club_color(c, club);
            rrect(kr.x, kr.y, kr.w, kr.h, 12.0, Theme::erhoehung());
            rrect(kr.x, kr.y, kr.w, 70.0, 12.0, alpha(farbe, 0.85));
            if ui.hover(kr) { rrect_line(kr.x, kr.y, kr.w, kr.h, 12.0, Theme::akzent()); }
            ui.player_photo(kr.x + 44.0, kr.y + 52.0, 32.0, pq.id, &pq.name);
            ui.title(&ui.fit(&pq.name, 20.0, kr.w - 170.0), kr.x + 86.0, kr.y + 32.0, 20.0, hex(0xffffff));
            if pq.team >= 0 { ui.club_crest(kr.x + kr.w - 30.0, kr.y + 30.0, 16.0, c.world.clubs[club].id, farbe, &c.world.clubs[club].short); }
            ui.text(&ui.fit(&if pq.team >= 0 { c.world.clubs[club].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").to_string() }, 13.0, kr.w - 140.0), kr.x + 86.0, kr.y + 54.0, 13.0, alpha(hex(0xffffff), 0.85));
            ui.pos_tag(kr.x + 86.0, kr.y + 84.0, pos_de(&pq.pos));
            ui.text(&format!("{} · {}", crate::tf!("ui.wort.jahre_alt", pq.age), money(pq.value)), kr.x + 130.0, kr.y + 89.0, 13.0, Theme::text2());
            ui.title(&format!("{sim:.0} %"), kr.x + kr.w - 76.0, kr.y + 94.0, 24.0, Theme::akzent());
            ui.text_right(crate::inhalt::t("ui.stat.aehnlich"), kr.x + kr.w - 12.0, kr.y + 108.0, 11.0, Theme::gedaempft());
            let (sq, _) = crate::statistik::saison_von(&c.world, q);
            let mut y = kr.y + 132.0;
            for (key, f, quote) in kern.iter().take(6) {
                let (a, b) = (f(&sq), f(&vorbild));
                let fmt = |x: Option<f32>| x.map(|v| if *quote { format!("{v:.0} %") } else { format!("{v:.2}") }).unwrap_or_else(|| "–".into());
                ui.text(&ui.fit(crate::inhalt::t(&format!("ui.stat.{key}")), 12.0, kr.w * 0.5), kr.x + 12.0, y, 12.0, Theme::text2());
                ui.text_right(&fmt(a), kr.x + kr.w * 0.78, y, 13.0, Theme::text());
                ui.text_right(&fmt(b), kr.x + kr.w - 12.0, y, 12.0, Theme::gedaempft());
                y += 22.0;
                if y > kr.y + kr.h - 40.0 { break; }
            }
            ui.text_right(&crate::tf!("ui.stat.vorbild_spalte", pl.name.split_whitespace().last().unwrap_or("")), kr.x + kr.w - 12.0, kr.y + 124.0 - 4.0, 10.0, Theme::gedaempft());
            if let Some(m) = c.machbar(q, false) { ui.text(&ui.fit(crate::inhalt::t(m), 12.0, kr.w - 24.0), kr.x + 12.0, kr.y + kr.h - 14.0, 12.0, Theme::problem()); }
            if ui.hit(kr) { v.sel_player = Some(q); v.modal = crate::pages::Modal::Player(q); }
        }
        let mut y = r.y + 54.0 + kh + 34.0;
        for (q, sim) in liste.into_iter().skip(3) {
            let pq = &c.world.players[q];
            let rr = Rect::new(r.x + 8.0, y - 20.0, r.w - 16.0, 30.0);
            if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
            ui.text(&format!("{sim:.0} %"), r.x + 16.0, y, 15.0, Theme::akzent());
            ui.pos_tag(r.x + 80.0, y - 5.0, pos_de(&pq.pos));
            ui.text(&ui.fit(&format!("{} · {}", pq.name, pq.age), 15.0, r.w * 0.35 - 44.0), r.x + 124.0, y, 15.0, Theme::text());
            let club = if pq.team >= 0 { c.world.clubs[pq.team as usize].name.clone() } else { String::new() };
            ui.text(&ui.fit(&club, 14.0, r.w * 0.25), r.x + r.w * 0.42, y, 14.0, Theme::text2());
            ui.text(&money(pq.value), r.x + r.w * 0.70, y, 14.0, Theme::text());
            if let Some(k) = c.machbar(q, false) { ui.text_right(crate::inhalt::t(k), r.x + r.w - 16.0, y, 13.0, Theme::problem()); }
            if ui.hit(rr) { v.sel_player = Some(q); v.modal = crate::pages::Modal::Player(q); }
            y += 32.0;
            if y > r.y + r.h - 10.0 { break; }
        }
        return;
    }
    let um_art = |b: usize, w: StatWert| -> u8 { if matches!(w, StatWert::Quote(..)) { 4 } else { [0u8, 1, 2, 3][b] } };
    let um_f = |b: usize, w: StatWert, x: f32| -> f32 { if um && !matches!(w, StatWert::Zahl(crate::statistik::GELB) | StatWert::Zahl(crate::statistik::ROT)) { crate::statistik::umrechnen(&c.world, x, liga, c.league(), um_art(b, w)) } else { x } };
    // the comparison group: same position group in his league, at least 900 league minutes (earlier in the season: at
    // least 45 % of the most-used player's minutes, so the bars mean something from the first weeks)
    let gr = crate::scoutbericht::gruppe(&pl.pos);
    let (gruppe, schwelle) = if liga < c.world.leagues.len() && !vor { crate::statistik::liga_vergleich_spieler(&c.world, liga, gr) } else { (vec![], 0) };
    let liga_name = if liga < c.world.leagues.len() { c.world.leagues[liga].name.clone() } else { "–".into() };
    let bereiche: Vec<usize> = if gr == crate::id("tw") { vec![3, 1] } else { vec![0, 1, 2] };
    // always say what he is compared with (Fabian 24.11.)
    let vergleich = if gruppe.len() >= 5 { crate::tf!("ui.stat.vergleich", gruppe.len(), crate::inhalt::t(&format!("ui.stat.gruppe_lang.{gr}")), liga_name.clone(), schwelle) }
        else if vor { crate::inhalt::t("ui.stat.vergleich_vorsaison").to_string() } else { crate::inhalt::t("ui.stat.vergleich_zu_klein").to_string() };
    ui.text(&ui.fit(&vergleich, 14.0, cbreite() - 24.0), x0() + 16.0, cy + 92.0, 14.0, Theme::text2());
    let oben = cy + 104.0;
    // the ranking window is modal: everything under it gets no mouse until it closes
    // over the whole profile below the tabs, so the rankings have room
    let fr = { let w = (cbreite() - 80.0).min(900.0); let y = Y0 + 36.0; Rect::new(x0() + 4.0 + (cbreite() - w) / 2.0, y, w, (cy + ch - y).min(720.0)) };
    let fenster = STAT_FENSTER.load(std::sync::atomic::Ordering::Relaxed);
    let offen = (fenster > 0 && (fenster - 1) / 1024 == p).then(|| ((fenster - 1) % 1024 / 32, (fenster - 1) % 32));
    let (maus, klick) = (ui.mouse, ui.clicked);
    if offen.is_some() { ui.mouse = vec2(-1.0e4, -1.0e4); ui.clicked = false; }
    let mut oeffnen: Option<(usize, usize)> = None;
    if STAT_RAD.load(std::sync::atomic::Ordering::Relaxed) {
        let r = Rect::new(x0() + 4.0, oben, cbreite(), ch - (oben - cy));
        ui.card_titled(r, &crate::tf!("ui.stat.rad_titel", pl.name.clone()), Theme::akzent());
        let mut werte: Vec<(String, usize, f32, usize)> = vec![];
        for &b in &bereiche {
            if !erweitert && b != 0 { continue; }
            let schon = werte.len();
            for (z, &(key, wert, weniger)) in STAT_ZEILEN[b].1.iter().enumerate() {
                if !erweitert && !matches!(key, "tore" | "vorlagen" | "gelb" | "rot") { continue; }
                if matches!(key, "gelb" | "rot") { continue; }
                let Some(eigen) = wert.p90(&st_liga) else { continue };
                let w: Vec<f32> = gruppe.iter().filter_map(|s| wert.p90(&s.1)).collect();
                if w.len() < 5 { continue; }
                // at most six per area, so every area shows on the wheel
                if werte.len() - schon < 6 { werte.push((crate::inhalt::t(&format!("ui.stat.{key}")).to_string(), b, pz_von(&w, eigen, weniger), b * 32 + z)); }
            }
        }
        if werte.len() < 3 { ui.paragraph(crate::inhalt::t("ui.stat.rad_leer"), r.x + 14.0, r.y + 62.0, 15.0, r.w - 28.0, Theme::text2()); }
        else if let Some(i) = perzentil_rad(ui, r, &werte) { oeffnen = Some((werte[i].3 / 32, werte[i].3 % 32)); }
    } else {
    let spalten = if bereiche.len() == 3 { 3 } else { 2 };
    let bw = (cbreite() - 12.0 * (spalten as f32 - 1.0)) / spalten as f32;
    for (k, &b) in bereiche.iter().enumerate() {
        let r = Rect::new(x0() + 4.0 + k as f32 * (bw + 12.0), oben, bw, ch - (oben - cy));
        let (name, zeilen) = STAT_ZEILEN[b];
        ui.card_titled(r, crate::inhalt::t(&format!("ui.stat.bereich.{name}")), [Theme::problem(), Theme::akzent(), Theme::verein(), Theme::gold()][b]);
        if !erweitert && b != 0 {
            ui.paragraph(crate::inhalt::t("ui.stat.nur_basis"), r.x + 14.0, r.y + 62.0, 14.0, r.w - 28.0, Theme::text2());
            continue;
        }
        player_row_head(ui, r.x, r.y + 52.0, &[("", 14.0), (crate::inhalt::t("ui.stat.saison"), r.w * 0.47), (crate::inhalt::t("ui.stat.pro90"), r.w * 0.61), (crate::inhalt::t("ui.stat.perzentil"), r.w * 0.74)]);
        let mut y = r.y + 80.0;
        for (z, &(key, wert, weniger)) in zeilen.iter().enumerate() {
            if !erweitert && !matches!(key, "tore" | "vorlagen" | "gelb" | "rot") { continue; }
            if y > r.y + r.h - 12.0 { break; }
            // a click on the row opens the league ranking of this number
            let zr = Rect::new(r.x + 6.0, y - 18.0, r.w - 12.0, 25.0);
            if gruppe.len() >= 5 {
                if ui.hover(zr) { rrect(zr.x, zr.y, zr.w, zr.h, 4.0, Theme::ueberlagerung()); }
                if ui.hit(zr) { oeffnen = Some((b, z)); }
            }
            ui.text(&ui.fit(crate::inhalt::t(&format!("ui.stat.{key}")), 14.0, r.w * 0.45 - 16.0), r.x + 14.0, y, 14.0, Theme::text());
            // an "i" with the one-sentence explanation of the number (Statistik Teil 4, catalogue)
            let erkl = crate::inhalt::t(&format!("ui.stat.erkl.{key}"));
            if !erkl.is_empty() {
                let iw = ui.measure(&ui.fit(crate::inhalt::t(&format!("ui.stat.{key}")), 14.0, r.w * 0.45 - 16.0), 14.0);
                let ix = r.x + 22.0 + iw;
                draw_circle_lines(ix, y - 5.0, 6.0, 1.0, Theme::gedaempft());
                ui.text_center("i", ix, y - 1.0, 10.0, Theme::gedaempft()); // roh
                if Rect::new(r.x + 10.0, y - 16.0, r.w * 0.45, 22.0).contains(ui.mouse) { *ui.tooltip.borrow_mut() = Some(erkl.to_string()); }
            }
            let Some((v, je90)) = wert.wert(st) else { ui.text("–", r.x + r.w * 0.47, y, 14.0, Theme::gedaempft()); y += 25.0; continue };
            let v = um_f(b, wert, v);
            let fmt = |x: f32| if matches!(wert, StatWert::Quote(..)) { format!("{x:.0} %") } else if matches!(wert, StatWert::Zahl(_)) && !je90 { format!("{x:.0}") } else if x.fract() == 0.0 && matches!(wert, StatWert::Zahl(_)) { format!("{x:.0}") } else { format!("{x:.2}") };
            ui.text(&fmt(v), r.x + r.w * 0.47, y, 14.0, Theme::text());
            let m = get(st, MIN) as f32;
            if je90 && m >= 90.0 { ui.text(&format!("{:.2}", v * 90.0 / m), r.x + r.w * 0.61, y, 14.0, Theme::text2()); }
            // percentile against the group (league figures only)
            if let Some(eigen) = wert.p90(&st_liga) {
                let werte: Vec<f32> = gruppe.iter().filter_map(|s| wert.p90(&s.1)).collect();
                if werte.len() >= 5 {
                    let pz = pz_von(&werte, eigen, weniger);
                    let bx = r.x + r.w * 0.74;
                    let bl = r.w * 0.26 - 44.0;
                    rrect(bx, y - 10.0, bl, 10.0, 3.0, alpha(Theme::text(), 0.08));
                    rrect(bx, y - 10.0, bl * pz / 100.0, 10.0, 3.0, pz_farbe(pz));
                    ui.text_right(&format!("{pz:.0}"), r.x + r.w - 12.0, y, 13.0, Theme::text2());
                }
            }
            y += 25.0;
        }
    }
    }
    ui.mouse = maus;
    ui.clicked = klick;
    if let Some((b, z)) = oeffnen { STAT_FENSTER.store(p * 1024 + b * 32 + z + 1, std::sync::atomic::Ordering::Relaxed); }
    if let Some((b, z)) = offen {
        let zu = rangliste_fenster(ui, v, c, p, fr, b, z, &gruppe, wert_eigen(b, z, &st_liga), (gr, &liga_name, schwelle));
        if zu { STAT_FENSTER.store(0, std::sync::atomic::Ordering::Relaxed); }
    }
}

fn wert_eigen(b: usize, z: usize, st: &crate::statistik::Stat) -> Option<f32> { STAT_ZEILEN[b].1[z].1.p90(st) }
fn pz_farbe(pz: f32) -> Color { if pz >= 70.0 { Theme::gut() } else if pz >= 35.0 { Theme::achtung() } else { Theme::problem() } }

/// The league ranking of one number (Fabian 24.11.): who he is compared with, the top three of the league, and the players
/// next to him in the ranking. Returns true when it closes.
#[allow(clippy::too_many_arguments)]
fn rangliste_fenster(ui: &mut Ui, v: &mut View, c: &Career, p: usize, r: Rect, b: usize, z: usize, gruppe: &[(usize, crate::statistik::Stat)], eigen: Option<f32>, (gr, liga, schwelle): (&str, &str, u32)) -> bool {
    let (key, wert, weniger) = STAT_ZEILEN[b].1[z];
    let quote = !matches!(wert, StatWert::Zahl(_) | StatWert::Tausendstel(_));
    let fmt = |x: f32| if matches!(wert, StatWert::Quote(..)) { format!("{x:.0} %") } else { format!("{x:.2}") };
    rrect(r.x - 8.0, r.y - 8.0, r.w + 16.0, r.h + 16.0, 20.0, alpha(hex(0x000000), 0.35));
    // texts measure against the window, not the cards under it
    let rahmen_vorher = std::mem::take(&mut *ui.rahmen.borrow_mut());
    ui.card_titled(r, &crate::tf!("ui.stat.fenster_titel", crate::inhalt::t(&format!("ui.stat.{key}"))), [Theme::problem(), Theme::akzent(), Theme::verein(), Theme::gold()][b]);
    // best first; ties by index so the order stays put from frame to frame
    let mut rang: Vec<(usize, f32)> = gruppe.iter().filter_map(|(q, s)| Some((*q, wert.p90(s)?))).collect();
    rang.sort_by(|a, b| (if weniger { a.1.partial_cmp(&b.1) } else { b.1.partial_cmp(&a.1) }).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0)));
    let werte: Vec<f32> = rang.iter().map(|x| x.1).collect();
    let mut y = r.y + 62.0;
    let art = crate::inhalt::t(if quote { "ui.stat.fenster_wie_sie_sind" } else { "ui.stat.fenster_pro90" });
    let gruppe_text = crate::tf!("ui.stat.fenster_gruppe", rang.len(), crate::inhalt::t(&format!("ui.stat.gruppe_lang.{gr}")), liga.to_string(), schwelle, art);
    ui.text(&ui.fit(&gruppe_text, 14.0, r.w - 32.0), r.x + 16.0, y, 14.0, Theme::text2());
    y += 24.0;
    let name = c.world.players[p].name.clone();
    let platz = rang.iter().position(|x| x.0 == p);
    let er = match eigen {
        Some(e) if rang.len() >= 5 => match platz {
            Some(i) => crate::tf!("ui.stat.fenster_er", name, fmt(e), format!("{:.0}", pz_von(&werte, e, weniger)), i + 1, rang.len()),
            None => crate::tf!("ui.stat.fenster_ausser", name, fmt(e), format!("{:.0}", pz_von(&werte, e, weniger))),
        },
        _ => crate::inhalt::t("ui.stat.fenster_kein_wert").to_string(),
    };
    ui.text_semi(&ui.fit(&er, 16.0, r.w - 32.0), r.x + 16.0, y, 16.0, Theme::text());
    y += 22.0;
    ui.text(&ui.fit(crate::inhalt::t("ui.stat.fenster_hinweis"), 13.0, r.w - 32.0), r.x + 16.0, y, 13.0, Theme::gedaempft());
    y += 30.0;
    let mut zu = false;
    if rang.len() < 5 {
        ui.paragraph(crate::inhalt::t("ui.stat.fenster_zu_wenig"), r.x + 16.0, y, 15.0, r.w - 32.0, Theme::text2());
    } else {
        // one row: rank, position, name, club, value, percentile bar
        let reihe = |ui: &mut Ui, y: f32, platz: Option<usize>, q: usize, x: f32| -> bool {
            let pq = &c.world.players[q];
            let rr = Rect::new(r.x + 10.0, y - 21.0, r.w - 20.0, 30.0);
            if q == p { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, alpha(Theme::akzent(), 0.18)); } else if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, Theme::ueberlagerung()); }
            ui.text_right(&platz.map_or("–".to_string(), |i| format!("{}.", i + 1)), r.x + 52.0, y, 15.0, Theme::text2());
            ui.pos_tag(r.x + 62.0, y - 5.0, pos_de(&pq.pos));
            ui.text(&ui.fit(&pq.name, 15.0, r.w * 0.30), r.x + 108.0, y, 15.0, Theme::text());
            let club = if pq.team >= 0 { c.world.clubs[pq.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").to_string() };
            ui.text(&ui.fit(&club, 14.0, r.w * 0.26), r.x + 108.0 + r.w * 0.31, y, 14.0, Theme::text2());
            ui.text_right(&fmt(x), r.x + r.w * 0.76, y, 15.0, Theme::text());
            let pz = pz_von(&werte, x, weniger);
            let (bx, bl) = (r.x + r.w * 0.79, r.w * 0.21 - 56.0);
            rrect(bx, y - 10.0, bl, 10.0, 3.0, alpha(Theme::text(), 0.08));
            rrect(bx, y - 10.0, bl * pz / 100.0, 10.0, 3.0, pz_farbe(pz));
            ui.text_right(&format!("{pz:.0}"), r.x + r.w - 18.0, y, 14.0, Theme::text2());
            q != p && ui.hit(rr)
        };
        let mut gewaehlt = None;
        ui.text_semi(crate::inhalt::t("ui.stat.fenster_top3"), r.x + 16.0, y, 14.0, Theme::gedaempft());
        ui.text_right(crate::inhalt::t("ui.stat.perzentil"), r.x + r.w - 18.0, y, 12.0, Theme::gedaempft());
        y += 30.0;
        for (i, &(q, x)) in rang.iter().enumerate().take(3) { if reihe(ui, y, Some(i), q, x) { gewaehlt = Some(q); } y += 32.0; }
        y += 14.0;
        // the neighbours: him in the middle, two better, two worse (he is put in by value when he is not in the group)
        let mut liste: Vec<(Option<usize>, usize, f32)> = rang.iter().enumerate().map(|(i, &(q, x))| (Some(i), q, x)).collect();
        let mitte = match (platz, eigen) {
            (Some(i), _) => Some(i),
            (None, Some(e)) => { let k = rang.iter().filter(|x| if weniger { x.1 < e } else { x.1 > e }).count(); liste.insert(k, (None, p, e)); Some(k) }
            _ => None,
        };
        if let Some(m) = mitte {
            // five around him, after the top three when he is up there himself
            let ab = 3.min(liste.len().saturating_sub(5));
            let ende = (m.saturating_sub(2).max(ab) + 5).min(liste.len());
            let start = ende.saturating_sub(5).max(ab);
            ui.text_semi(crate::inhalt::t(if m >= start { "ui.stat.fenster_nachbarn" } else { "ui.stat.fenster_danach" }), r.x + 16.0, y, 14.0, Theme::gedaempft());
            y += 30.0;
            for &(pl, q, x) in &liste[start..ende] {
                if y > r.y + r.h - 56.0 { break; }
                if reihe(ui, y, pl, q, x) { gewaehlt = Some(q); }
                y += 32.0;
            }
        }
        if let Some(q) = gewaehlt { v.sel_player = Some(q); v.modal = crate::pages::Modal::Player(q); zu = true; }
    }
    if ui.button(Rect::new(r.x + r.w - 156.0, r.y + r.h - 50.0, 140.0, 38.0), crate::inhalt::t("aktion.schliessen"), false) { zu = true; }
    if is_key_pressed(KeyCode::Escape) || (ui.clicked && !r.contains(ui.mouse)) { ui.clicked = false; zu = true; }
    // clicks inside the window go nowhere else
    if r.contains(ui.mouse) { ui.clicked = false; }
    *ui.rahmen.borrow_mut() = rahmen_vorher;
    zu
}

pub(super) fn entwicklung_reiter(ui: &mut Ui, c: &Career, p: usize, cy: f32, ch: f32) {
    use crate::entwicklung::{GRUPPEN, PHASEN};
    let pl = &c.world.players[p];
    let (l, r) = (Rect::new(x0() + 4.0, cy, cbreite() * 0.62 - 6.0, ch), Rect::new(x0() + cbreite() * 0.62 + 10.0, cy, cbreite() * 0.38 - 6.0, ch));
    ui.card_titled(l, crate::inhalt::t("ui.ent.verlauf"), Theme::akzent());
    let e = &pl.ent;
    if !c.gehoert_uns(p) || !e.an || e.verlauf.is_empty() {
        ui.paragraph(crate::inhalt::t(if c.gehoert_uns(p) { "ui.ent.noch_kein_verlauf" } else { "ui.ent.nur_eigene" }), l.x + 14.0, l.y + 64.0, 15.0, l.w - 28.0, Theme::text2());
        ui.card_titled(r, crate::inhalt::t("ui.ent.stand"), Theme::gold());
        return;
    }
    let farben = [hex(0xE04A4A), hex(0xE0A526), hex(0x3BA3DA), hex(0x8E44AD), hex(0x27AE60)];
    let gruppe_name = |g: usize| crate::inhalt::t(&format!("ui.ent.gruppe.{}", GRUPPEN[g]));
    let zahl = |x: f32| { let s = format!("{x:+.1}"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { s.replace('.', ",") } else { s } };
    let torwart = pl.pos == "GK";
    // cumulative change per group, one line each (a field player's goalkeeping abilities hardly move and are left out)
    let mut summe = [0.0f32; 5];
    let reihen: Vec<[f32; 5]> = e.verlauf.iter().map(|w| { for g in 0..5 { summe[g] += w.gruppen[g]; } summe }).collect();
    let zeigen: Vec<usize> = (0..5).filter(|&g| torwart || g != 4).collect();
    let (gx, gy, gw, gh) = (l.x + 50.0, l.y + 60.0, l.w - 70.0, (l.h * 0.34).clamp(110.0, 200.0));
    let lo = reihen.iter().flat_map(|x| zeigen.iter().map(move |&g| x[g])).fold(-0.3f32, f32::min);
    let hi = reihen.iter().flat_map(|x| zeigen.iter().map(move |&g| x[g])).fold(0.3f32, f32::max);
    let yv = |v: f32| gy + gh - (v - lo) / (hi - lo) * gh;
    rrect(gx, gy, gw, gh, 4.0, Theme::erhoehung());
    draw_line(gx, yv(0.0), gx + gw, yv(0.0), 1.0, Theme::kontur());
    ui.text_right(&zahl(hi), gx - 6.0, gy + 10.0, 12.0, Theme::gedaempft());
    ui.text_right(&zahl(lo), gx - 6.0, gy + gh, 12.0, Theme::gedaempft());
    let n = reihen.len();
    let xv = |i: usize| gx + 6.0 + if n > 1 { i as f32 / (n - 1) as f32 * (gw - 12.0) } else { 0.0 };
    for &g in &zeigen {
        for i in 1..n { draw_line(xv(i - 1), yv(reihen[i - 1][g]), xv(i), yv(reihen[i][g]), 2.0, farben[g]); }
        if let Some(last) = reihen.last() { draw_circle(xv(n - 1), yv(last[g]), 3.0, farben[g]); }
    }
    // legend with the change over the shown weeks
    let mut x = gx;
    for &g in &zeigen {
        draw_rectangle(x, gy + gh + 14.0, 12.0, 4.0, farben[g]);
        let t = crate::tf!("ui.ent.legende_gruppe", gruppe_name(g), zahl(summe[g]));
        ui.text(&t, x + 16.0, gy + gh + 21.0, 13.0, Theme::text2());
        x += ui.measure(&t, 13.0) + 34.0;
    }
    // the last weeks: their biggest change and its reasons
    let mut y = gy + gh + 52.0;
    ui.text_semi(crate::inhalt::t("ui.ent.letzte_wochen"), l.x + 14.0, y, 14.0, Theme::gedaempft());
    y += 24.0;
    for w in e.verlauf.iter().rev().take(crate::regeln::f("entwicklung_pesm", "anzeige.gruende_wochen") as usize) {
        if y > l.y + l.h - 12.0 { break; }
        let g = zeigen.iter().copied().max_by(|a, b| w.gruppen[*a].abs().partial_cmp(&w.gruppen[*b].abs()).unwrap()).unwrap();
        let gruende: Vec<&str> = w.gruende.iter().map(|k| crate::inhalt::t(&format!("ui.ent.grund.{}", k.replace('+', "_plus").replace('-', "_minus")))).filter(|s| !s.is_empty()).take(3).collect();
        let warum = if gruende.is_empty() { crate::inhalt::t(&format!("en3.phase.{}", PHASEN[w.phase as usize])).to_string() } else { gruende.join(", ") };
        let n_gruppe = crate::entwicklung::GRUPPE.iter().filter(|&&x| x == g).count() as f32;
        let wert = w.gruppen[g] * n_gruppe;
        // a small weekly change keeps two decimals, so it doesn't read as „+0,0“
        let text = if wert.abs() < 0.1 { let s = format!("{wert:+.2}"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { s.replace('.', ",") } else { s } } else { zahl(wert) };
        let zeile = crate::tf!("ui.ent.zeile", text, gruppe_name(g), warum);
        ui.text(&date::short(c.world.season, w.tag % date::SEASON_DAYS), l.x + 14.0, y, 14.0, Theme::text2());
        ui.text(&ui.fit(&zeile, 15.0, l.w - 120.0), l.x + 100.0, y, 15.0, if w.gruppen[g] >= 0.0 { Theme::text() } else { Theme::problem() });
        y += 22.0;
    }
    // state: phase, expected development, focus, load, traits, progress paths
    ui.card_titled(r, crate::inhalt::t("ui.ent.stand"), Theme::gold());
    let abs = c.abs();
    let phase = crate::inhalt::t(&format!("en3.phase.{}", PHASEN[e.phase as usize]));
    let phase = if (1..=3).contains(&e.phase) && e.bis > abs { crate::tf!("ui.ent.phase_bis", phase, (e.bis - abs) / 7 + 1) } else { phase.to_string() };
    let fokus = crate::entwicklung::fokus_von(pl);
    let z: Vec<(String, String)> = vec![
        (crate::inhalt::t("ui.ent.phase").into(), phase),
        (crate::inhalt::t("ui.ent.fokus").into(), if fokus.is_empty() { crate::inhalt::t("ui.ent.kein_fokus").into() } else { crate::inhalt::t(&format!("ui.ent.fokus.{fokus}")).into() }),
        (crate::inhalt::t("ui.ent.belastung").into(), format!("{:.0}", pl.ps.last)),
    ];
    zeilen(ui, r, &z, 64.0);
    let mut y = r.y + 64.0 + z.len() as f32 * 30.0;
    ui.text(crate::inhalt::t("ui.ent.erwartet"), r.x + 14.0, y, 15.0, Theme::text2());
    entwicklung_pfeil(ui, r.x + 220.0, y - 5.0, c.ent_erwartung(p));
    y += 30.0;
    ui.text_semi(crate::inhalt::t("ui.ent.merkmale"), r.x + 14.0, y, 14.0, Theme::gedaempft());
    y += 10.0;
    let mut x = r.x + 14.0;
    for m in &e.merkmale {
        let t = crate::inhalt::t(&format!("mk.{m}"));
        let w = ui.measure(t, 14.0) + 20.0;
        if x + w > r.x + r.w - 14.0 { x = r.x + 14.0; y += 30.0; }
        rrect(x, y, w, 24.0, 12.0, alpha(Theme::akzent(), 0.16));
        ui.text(t, x + 10.0, y + 17.0, 14.0, Theme::text());
        x += w + 8.0;
    }
    y += 46.0;
    ui.text_semi(crate::inhalt::t("ui.ent.wege"), r.x + 14.0, y, 14.0, Theme::gedaempft());
    y += 20.0;
    let bis = crate::regeln::f("entwicklung2", "wege.treffer_bis");
    if e.wege.is_empty() { ui.text(crate::inhalt::t("ui.ent.keine_wege"), r.x + 14.0, y + 4.0, 14.0, Theme::text2()); }
    for (id, t, _) in &e.wege {
        if y > r.y + r.h - 20.0 { break; }
        ui.text(&ui.fit(crate::inhalt::t(&format!("en2.weg.{id}")), 15.0, r.w * 0.5), r.x + 14.0, y + 4.0, 15.0, Theme::text());
        if *t as f32 >= bis { ui.text_right(crate::inhalt::t("ui.ent.erreicht"), r.x + r.w - 14.0, y + 4.0, 14.0, Theme::gut()); }
        else { ui.bar(r.x + r.w * 0.55, y - 4.0, r.w * 0.45 - 14.0, 8.0, *t as f32 / bis, Theme::akzent()); }
        y += 26.0;
    }
}
