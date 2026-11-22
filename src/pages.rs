// The manager's pages, drawn into the content area right of the sidebar (layout after the original's screenshots).
use crate::date;
use crate::game::*;
use crate::ui::*;
use crate::world::*;
use macroquad::prelude::*;

pub const X0: f32 = 214.0;
/// Content width right of the side bar (follows breite()).
pub fn cbreite() -> f32 { crate::ui::breite() - X0 - 18.0 }
pub const Y0: f32 = 244.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Page {
    Uebersicht, Posteingang, Kalender, Kader, Aufstellung, Kabine, Training, Medizin, Praesidium, Finanzen, Trainer,
    Transfermarkt, Verhandlungen, Vertraege, Scouting, Akademie, Tabellen, Presse, Spieltag, Optionen,
    Ziele, Manager, Stadion, Personalmarkt, Geschichte, Tagebuch, Klima, Marken, Kaderplanung, Vergleich, Entwicklung, Vorbereitung, Analyse, Zeitung, Fans, Ranglisten, Vereine, Vereinswelt, Portal, Handbuch, Regeln, Profi, Spielerprofil, Zentrale, Kooperationen, ZweiteMannschaft, Jugendnetz, Merkliste, Trainerteam,
}

/// The side menu: group → main pages → their tabs. Every page keeps its id (links from the inbox, ticker and so on
/// open the right tab); a menu item opens the tab last used on that main page.
pub type Haupt = (&'static str, &'static [(Page, &'static str)]);
pub const MENU: [(&str, &[Haupt]); 7] = [
    ("ui.pages.buero", &[("bereich.zentrale", &[(Page::Zentrale, "bereich.zentrale"), (Page::Uebersicht, "reiter.uebersicht")]), ("zentrale.k.posteingang", &[(Page::Posteingang, "zentrale.k.posteingang")]), ("gl.kopf.ergebnisse.alle", &[(Page::Kalender, "gl.kopf.ergebnisse.alle")])]),
    ("ui.pages.mannschaft", &[("zentrale.k.kader", &[(Page::Kader, "zentrale.k.kader"), (Page::ZweiteMannschaft, "ui.game.zweite_mannschaft"), (Page::Kaderplanung, "aktion.kaderplanung"), (Page::Vergleich, "menu.spielervergleich.titel")]),
        ("gb2.reiter.taktik", &[(Page::Aufstellung, "aktion.aufstellung"), (Page::Vorbereitung, "ui.pages.spielvorbereitung"), (Page::Analyse, "menu.analyse.titel")]),
        ("vg.pg.dim.kabine", &[(Page::Kabine, "vg.pg.dim.kabine")]),
        ("status.training", &[(Page::Training, "status.training"), (Page::Entwicklung, "profil.entwicklung_verlauf"), (Page::Medizin, "kachel.medizin")])]),
    ("ui.pages.markt", &[("bereich.transfers", &[(Page::Transfermarkt, "ux.abschnitt.markt"), (Page::Verhandlungen, "ux.abschnitt.verhandlungen"), (Page::Vertraege, "as.reiter.vertraege"), (Page::Merkliste, "aka.pille.merkliste")]), ("profil.teil.8", &[(Page::Scouting, "profil.teil.8")])]),
    ("ui.pages.jugend", &[("aka.gruppe.akademie", &[(Page::Akademie, "aka.gruppe.akademie"), (Page::Jugendnetz, "ui.main.jugendnetz")])]),
    ("ui.pages.verein", &[("klima.praesidium", &[(Page::Praesidium, "klima.praesidium"), (Page::Ziele, "aktion.ziele"), (Page::Klima, "ui.main.vereinsklima")]),
        ("bereich.finanzen", &[(Page::Finanzen, "bereich.finanzen"), (Page::Stadion, "menu.stadion_und_anlagen.titel"), (Page::Marken, "ui.main.marken_und_sponsoren"), (Page::Kooperationen, "ui.farm.kooperationen")]),
        ("md.ui.chip.personal", &[(Page::Trainerteam, "ui.main.trainerteam"), (Page::Personalmarkt, "ui.main.personalmarkt")]),
        ("taktik.board.coach", &[(Page::Trainer, "aktion.trainerprofil"), (Page::Manager, "gm.mg.hero"), (Page::Tagebuch, "gm.tb.hero")])]),
    ("ui.pages.welt", &[("zentrale.wettbewerbe", &[(Page::Tabellen, "ui.pages.tabellen"), (Page::Ranglisten, "ux.abschnitt.rankings"), (Page::Geschichte, "menu.vereinsgeschichte.titel")]),
        ("menu.vereine_und_spieler.titel", &[(Page::Vereine, "suche.ueberschrift"), (Page::Vereinswelt, "ui.main.vereinswelt"), (Page::Portal, "md.pool.typ.portal")])]),
    ("ui.pages.medien", &[("zentrale.k.medien", &[(Page::Presse, "feed.art.journalist"), (Page::Zeitung, "md.pool.typ.zeitung"), (Page::Fans, "menu.fans_und_stimmung.titel")])]),
];
/// Settings (bottom left, no menu item) with their tabs.
pub const EINSTELLUNGEN: Haupt = ("bereich.system", &[(Page::Optionen, "menu.optionen.titel"), (Page::Handbuch, "ui.main.handbuch"), (Page::Regeln, "menu.regeln_und_anpassung.titel"), (Page::Profi, "menu.profi_details.titel")]);
/// The main page a page belongs to: (group index or 7 for the settings, the main page).
pub fn haupt_von(p: Page) -> Option<(usize, Haupt)> {
    for (g, (_, hs)) in MENU.iter().enumerate() { for h in hs.iter() { if h.1.iter().any(|x| x.0 == p) { return Some((g, *h)); } } }
    EINSTELLUNGEN.1.iter().any(|x| x.0 == p).then_some((7, EINSTELLUNGEN))
}
/// The tabs shown for a main page (the second team only when the club has one).
pub fn reiter(h: Haupt, c: &Career) -> Vec<(Page, &'static str)> { h.1.iter().copied().filter(|x| x.0 != Page::ZweiteMannschaft || c.zweite_mannschaft().is_some()).map(|(p, k)| (p, crate::inhalt::t(k))).collect() }
/// Counters at a menu item and its tab: unread mail, open negotiations.
pub fn zaehler(p: Page, c: &Career) -> usize {
    match p { Page::Posteingang => c.unread(), Page::Verhandlungen => c.negs.iter().filter(|n| n.state == crate::game::NegState::Counter && n.stage < 2).count(), _ => 0 }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Modal { None, Player(usize), Neg(usize), Suche }

pub struct MatchView {
    pub res: MatchResult,
    pub minute: f32,
    pub speed: f32,
    pub phase: u8, // 0 pre, 1 live, 2 post
    pub ended_at: f32,
    pub manual: (u8, u8),
    pub manual_open: bool,
    /// Half-time talk on the ticker: 0 not yet, 1 waiting at the break, 2 done.
    pub halbzeit: u8,
}

pub struct View {
    /// World map: zoom and offset (geo.rs).
    pub karte_zoom: f32,
    pub karte_off: (f32, f32),
    /// The jingle of the reward moment on screen was played.
    pub feier_ton: bool,
    pub page: Page,
    pub back: Vec<Page>,
    pub fwd: Vec<Page>,
    pub modal: Modal,
    pub tab: usize,
    pub scroll: f32,
    /// Scroll position of each list on the open page (by its place), and the list whose bar is being dragged.
    pub scrolls: std::collections::HashMap<(i32, i32), f32>,
    pub ziehen: Option<(i32, i32)>,
    /// Consequences of the answers of the open message, played out once: ((message, inbox size, day), per answer).
    pub post_folgen: Option<((usize, usize, u32), Vec<Vec<(String, f32)>>)>,
    pub sel_msg: Option<usize>,
    pub msg_filter: usize,
    /// Vorbereitung: trips (0) or training camps (1), and the camp length chosen.
    pub lager_reiter: u8,
    pub lager_tage: u32,
    /// The match (day, home, away) whose half-time talk in the FL27 live view is done.
    pub halbzeit_fl27: Option<(u32, usize, usize)>,
    pub sel_player: Option<usize>,
    pub market_pos: usize,
    pub market_age: i32,
    pub market_price: i32,
    pub market_scope: usize,
    pub market_text: String,
    pub market_focus: bool,
    /// Search overlay (Punkt 6): the text, its hits (0 player, 1 club, 2 page + index), the text they were made for.
    pub suche_text: String,
    /// Preparation page: chosen free day for a friendly, the score being entered for one played in FL27.
    pub vorb_tag: usize,
    pub vorb_tore: (u8, u8),
    pub suche_treffer: Vec<(u8, usize)>,
    pub suche_fuer: String,
    pub market_cache: Vec<usize>,
    pub market_key: String,
    pub table_league: Option<usize>,
    pub cal_month: i32,
    pub pick: Option<usize>,
    pub toast: Option<(String, f32, bool)>,
    pub mv: Option<MatchView>,
    pub sel_neg: Option<usize>,
    pub changed_at: f32,
    pub hud_seen: Vec<String>,
    pub league_scroll: usize,
    pub sel_club: Option<usize>,
    /// Right-click menu (Punkt 6): the player and where the menu opens.
    pub kontext: Option<(usize, Vec2)>,
    pub dark: bool,
    pub fl27: Option<crate::fl27::Beobachter>,
    pub fl27_status: String,
    pub fl27_live: Option<(u32, u32)>,
    pub fl27_log: Vec<String>,
    pub fl27_t0: f32,
    /// A knock-out match from FL27 that ended level, waiting for the person's answer.
    pub fl27_ko: Option<KoOffen>,
    pub fl27_ko_apply: Option<(KoOffen, Option<(bool, bool)>)>,
    /// Farm-team search form: own country only, tiers below from/to, yearly amount at most, loan places.
    pub koop_form: (bool, u8, u8, f64, u8),
    /// FL27 running (checked every 2 s), the ending in its own thread, the wish to end (waits for the result), the
    /// question while a match runs, the automatic end time, the result taken over from FL27; last live-write look.
    pub fl27_laeuft: bool,
    /// Live view of the match in FL27 (Punkt 4, from the Coaching Zone), its tab (0 key events, 1 all, 2 player table),
    /// the last event seen (None = joined, old events stay quiet), the card of a new goal or card, preview data only.
    pub fl27_ansicht: Option<pesm_coaching_zone::live::MatchView>,
    pub fl27_reiter: usize,
    pub fl27_seq: Option<u32>,
    pub fl27_einblendung: Option<(pesm_coaching_zone::live::EventKind, &'static str, String, usize, f64)>,
    pub fl27_vorschau: bool,
    pub fl27_pruef_t: f32,
    /// FL27 was seen running since the presets were written; the first look of this session has happened.
    pub fl27_gesehen: bool,
    pub fl27_erster_blick: bool,
    pub fl27_ende: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub fl27_ende_wunsch: bool,
    pub fl27_auto_t: Option<f32>,
    pub fl27_uebernommen: Option<String>,
    pub db_blick_t: f32,
    /// The tab last open on each main page (menu label → page).
    pub reiter_merk: std::collections::HashMap<&'static str, Page>,
}
pub enum KoOffen { Doc(Box<crate::fl27::SpielDoc>), Hand(MatchResult) }
fn klang_file() -> std::path::PathBuf { save_dir().join(crate::id("../klang.txt")) }
/// The stored volume (klang.txt: aus, leise, normal); default leise (SPIELGEFUEHL.md: „Standard leise“).
pub fn klang_lesen() -> f32 { klang_wert(std::fs::read_to_string(klang_file()).unwrap_or_default().trim()) }
fn klang_wert(s: &str) -> f32 { match s { "aus" => 0.0, "normal" => 0.6, _ => 0.3 } }
fn theme_file() -> std::path::PathBuf { save_dir().join(crate::id("../thema.txt")) }
/// Einstellungen → Sprache: "de" or "en", beside thema.txt (read at start in main.rs).
pub fn sprache_file() -> std::path::PathBuf { save_dir().join(crate::id("../sprache.txt")) }
impl View {
    pub fn new() -> View {
        View { karte_zoom: 1.0, karte_off: (0.0, 0.0), feier_ton: false, page: Page::Uebersicht, back: vec![], fwd: vec![], modal: Modal::None, tab: 0, scroll: 0.0, scrolls: Default::default(), ziehen: None, post_folgen: None, sel_msg: None, msg_filter: 0, lager_reiter: 0, lager_tage: 7, halbzeit_fl27: None, sel_player: None,
            market_pos: 0, market_age: 40, market_price: 200, market_scope: 0, market_text: String::new(), market_focus: false, suche_text: String::new(), vorb_tag: 0, vorb_tore: (0, 0), suche_treffer: vec![], suche_fuer: String::new(), market_cache: vec![], market_key: String::new(),
            table_league: None, cal_month: -1, pick: None, toast: None, mv: None, sel_neg: None, changed_at: 0.0, hud_seen: vec![], league_scroll: 0, sel_club: None, kontext: None,
            fl27: None, fl27_status: String::new(), fl27_live: None, fl27_log: vec![], fl27_t0: 0.0, fl27_ko: None, fl27_ko_apply: None, koop_form: (true, 1, 2, 0.0, 3), fl27_laeuft: false, fl27_ansicht: None, fl27_reiter: 0, fl27_seq: None, fl27_einblendung: None, fl27_vorschau: false, fl27_pruef_t: -10.0, fl27_gesehen: false, fl27_erster_blick: false, fl27_ende: None, fl27_ende_wunsch: false, fl27_auto_t: None, fl27_uebernommen: None, db_blick_t: 0.0, reiter_merk: Default::default(),
            dark: std::env::var("PESM_THEME").map(|t| t == crate::id("flutlicht")).unwrap_or_else(|_| std::fs::read_to_string(theme_file()).map(|t| t.trim() == crate::id("flutlicht")).unwrap_or(false)) }
    }
    pub fn go(&mut self, p: Page) {
        if p != self.page { self.back.push(self.page); self.fwd.clear(); }
        self.page = p;
        self.tab = 0;
        self.scroll = 0.0; self.scrolls.clear();
        self.changed_at = get_time() as f32;
    }
    pub fn toast(&mut self, s: &str, ok: bool) { self.toast = Some((s.to_string(), get_time() as f32, ok)); }
}

/// The rows shown of a list of `total` rows with room for `visible`, scrolled to `scroll`: every row is reachable.
pub fn fenster(total: usize, visible: usize, scroll: usize) -> std::ops::Range<usize> {
    let start = scroll.min(total.saturating_sub(visible));
    start..(start + visible).min(total)
}
/// A list that may not fit: mouse wheel, a visible scroll bar at the right edge that can be dragged, and its own
/// position per list (by where it sits on the page) while the page stays open. Returns the first row to draw.
fn scroll_rows(ui: &Ui, v: &mut View, area: Rect, total: usize, visible: usize) -> usize { scroll_schritt(ui, v, area, total, visible, 3.0) }
/// The same with its own step per wheel notch (blocks of different height move one at a time).
fn scroll_schritt(ui: &Ui, v: &mut View, area: Rect, total: usize, visible: usize, schritt: f32) -> usize {
    let key = (area.x as i32, area.y as i32);
    let max = total.saturating_sub(visible) as f32;
    let mut s = *v.scrolls.get(&key).unwrap_or(&v.scroll);
    if area.contains(ui.mouse) && ui.wheel != 0.0 { s -= ui.wheel.signum() * schritt; }
    if max > 0.0 {
        let track = Rect::new(area.x + area.w - 9.0, area.y + 4.0, 6.0, area.h - 8.0);
        let th = (track.h * visible as f32 / total as f32).max(28.0);
        let greif = Rect::new(track.x - 8.0, track.y, track.w + 12.0, track.h);
        if is_mouse_button_down(MouseButton::Left) && (v.ziehen == Some(key) || (v.ziehen.is_none() && greif.contains(ui.mouse))) {
            v.ziehen = Some(key);
            s = (ui.mouse.y - track.y - th / 2.0) / (track.h - th).max(1.0) * max;
        }
        let ty = track.y + (track.h - th) * (s / max).clamp(0.0, 1.0);
        rrect(track.x, track.y, track.w, track.h, 3.0, alpha(Theme::text(), 0.07));
        rrect(track.x, ty, track.w, th, 3.0, alpha(Theme::text(), if v.ziehen == Some(key) { 0.55 } else { 0.32 }));
    }
    if !is_mouse_button_down(MouseButton::Left) { v.ziehen = None; }
    s = s.clamp(0.0, max).round();
    v.scrolls.insert(key, s);
    fenster(total, visible, s as usize).start
}

fn tabs(ui: &mut Ui, v: &mut View, items: &[&str]) {
    let mut x = X0 + 4.0;
    for (i, t) in items.iter().enumerate() {
        let w = ui.measure_semi(t, 19.0) + 20.0;
        let r = Rect::new(x, Y0 - 42.0, w, 32.0);
        let h = ui.hover(r);
        if i == v.tab { rrect(r.x, r.y, r.w, r.h, 12.0, Theme::verein()); rrect(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, 10.0, if Theme::dark() { Theme::verein_tief() } else { Theme::verein_weich() }); } else if h { rrect(r.x, r.y, r.w, r.h, 12.0, Theme::erhoehung()); }
        ui.text_semi(t, x + 10.0, Y0 - 20.0, 19.0, if i == v.tab || h { Theme::text() } else { Theme::text2() });
        if ui.hit(r) && v.tab != i { v.tab = i; v.scroll = 0.0; v.scrolls.clear(); ui.play(crate::id("klick")); }
        x += w + 8.0;
    }
}

fn kpi(ui: &Ui, r: Rect, label: &str, value: &str, sub: &str, edge: Color) {
    ui.card(r);
    draw_rectangle(r.x, r.y, r.w, 3.0, edge);
    ui.text_semi(label, r.x + 16.0, r.y + 28.0, 15.0, Theme::text2());
    if r.h < 90.0 { ui.title(value, r.x + 16.0, r.y + 58.0, 28.0, Theme::text()); return; } // compact: no sub line
    ui.title(value, r.x + 16.0, r.y + 62.0, 30.0, Theme::text());
    ui.text(sub, r.x + 16.0, r.y + 84.0, 15.0, Theme::gedaempft());
}

/// Expected development of the overall per season (entwicklung.rs ent_erwartung) as an arrow with the number.
fn entwicklung_pfeil(ui: &Ui, x: f32, y: f32, d: f32) { entwicklung_pfeil_mit(ui, x, y, d, true) }
fn entwicklung_pfeil_mit(ui: &Ui, x: f32, y: f32, d: f32, mit_zahl: bool) {
    let g = crate::mensch::arr("entwicklung_pesm", "anzeige.pfeil");
    let stufe = g.iter().filter(|x| d >= x.as_f64().unwrap_or(0.0) as f32).count();
    let col = [Theme::problem(), Theme::achtung(), Theme::gedaempft(), Theme::gut(), Theme::gut()][stufe.min(4)];
    let winkel = [90.0f32, 45.0, 0.0, -45.0, -90.0][stufe.min(4)].to_radians();
    let (dx, dy) = (winkel.cos(), winkel.sin());
    let (sx, sy, ex, ey) = (x - dx * 8.0, y - dy * 8.0, x + dx * 8.0, y + dy * 8.0);
    draw_line(sx, sy, ex, ey, 3.0, col);
    let (nx, ny) = (-dy, dx);
    draw_triangle(vec2(ex + dx * 5.0, ey + dy * 5.0), vec2(ex + nx * 5.0, ey + ny * 5.0), vec2(ex - nx * 5.0, ey - ny * 5.0), col);
    if !mit_zahl { return; }
    let zahl = format!("{d:+.1}");
    let zahl = if crate::inhalt::sprache() == crate::inhalt::Sprache::De { zahl.replace('.', ",") } else { zahl };
    ui.text(&crate::tf!("ui.ent.je_saison", zahl), x + 16.0, y + 5.0, 13.0, Theme::text2());
}

/// The form bonus next to a strength („+2“ green, „−1“ red); nothing at 0. It works in the match, not for good.
fn zuschlag_text(ui: &Ui, x: f32, y: f32, p: &Player) {
    let z = crate::world::form_zuschlag(p);
    if z != 0.0 { ui.text(&format!("{z:+.0}").replace('-', "−"), x, y, 14.0, if z > 0.0 { Theme::gut() } else { Theme::problem() }); }
}
/// The card „So hat es gewirkt“ after a talk: arrows for morale, trust, relationship, team spirit and the teammates it reached.
pub fn wirkung_karte(ui: &mut Ui, c: &mut Career) {
    let Some(w) = c.wirkung.clone() else { return };
    if crate::ui::jetzt() - w.seit > crate::regeln::f("entwicklung_pesm", "wellen.anzeige_sekunden") as f64 { c.wirkung = None; return; }
    let n = w.andere.len().min(6);
    let r = Rect::new(breite() - 440.0, hoehe() - 200.0 - n as f32 * 30.0, 420.0, 160.0 + n as f32 * 30.0);
    rrect(r.x, r.y, r.w, r.h, 12.0, Theme::flaeche());
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.5, Theme::akzent());
    let pl = &c.world.players[w.pid];
    ui.player_photo(r.x + 34.0, r.y + 36.0, 20.0, pl.id, &pl.name);
    ui.title(crate::inhalt::t("ui.wellen.titel"), r.x + 64.0, r.y + 32.0, 20.0, Theme::text());
    ui.text(&ui.fit(&w.grund, 13.0, r.w - 80.0), r.x + 64.0, r.y + 52.0, 13.0, Theme::text2());
    let zahl = |d: f32| { let s = format!("{d:+.1}").replace('-', "−"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { s.replace('.', ",") } else { s } };
    let pfeil = |ui: &Ui, x: f32, y: f32, name: &str, d: f32| {
        let col = if d > 0.05 { Theme::gut() } else if d < -0.05 { Theme::problem() } else { Theme::gedaempft() };
        ui.text(name, x, y, 14.0, Theme::text2());
        let (ax, ay) = (x + 102.0, y - 5.0);
        if d > 0.05 { draw_triangle(vec2(ax - 6.0, ay + 4.0), vec2(ax + 6.0, ay + 4.0), vec2(ax, ay - 6.0), col); }
        else if d < -0.05 { draw_triangle(vec2(ax - 6.0, ay - 4.0), vec2(ax + 6.0, ay - 4.0), vec2(ax, ay + 6.0), col); }
        else { draw_rectangle(ax - 6.0, ay - 1.0, 12.0, 2.0, col); }
        ui.text(&zahl(d), x + 114.0, y, 14.0, col);
    };
    pfeil(ui, r.x + 18.0, r.y + 84.0, crate::inhalt::t("ui.wellen.moral"), w.moral);
    pfeil(ui, r.x + 218.0, r.y + 84.0, crate::inhalt::t("ui.wellen.vertrauen"), w.vertrauen);
    pfeil(ui, r.x + 18.0, r.y + 108.0, crate::inhalt::t("ui.wellen.beziehung"), w.beziehung);
    pfeil(ui, r.x + 218.0, r.y + 108.0, crate::inhalt::t("ui.wellen.teamgeist"), w.teamgeist);
    if n == 0 { ui.text(crate::inhalt::t("ui.wellen.niemand"), r.x + 18.0, r.y + 140.0, 13.0, Theme::gedaempft()); }
    for (k, (q, d, g)) in w.andere.iter().take(6).enumerate() {
        let y = r.y + 136.0 + k as f32 * 30.0;
        let ql = &c.world.players[*q];
        ui.player_photo(r.x + 30.0, y + 6.0, 11.0, ql.id, &ql.name);
        ui.text(&ui.fit(&format!("{}: {}", ql.name, g), 13.0, r.w - 120.0), r.x + 48.0, y + 11.0, 13.0, Theme::text());
        ui.text_right(&zahl(*d), r.x + r.w - 14.0, y + 11.0, 13.0, if *d >= 0.0 { Theme::gut() } else { Theme::problem() });
    }
    if ui.hit(r) { c.wirkung = None; }
}

pub(crate) fn form_arrow(ui: &Ui, x: f32, y: f32, f: f32) {
    // the form arrow of the game, 0…4 (entwicklung.toml pfeil_grenzen): up from 3, down to 1
    let pf = crate::mensch::pfeil(f);
    let f = if pf >= 3 { 1.0 } else if pf <= 1 { -1.0 } else { 0.0 };
    let (c, s) = if f > 0.8 { (Theme::gut(), "▲") } else if f < -0.8 { (Theme::problem(), "▼") } else { (Theme::achtung(), "►") };
    let _ = s;
    if f > 0.8 { draw_triangle(vec2(x, y + 5.0), vec2(x + 12.0, y + 5.0), vec2(x + 6.0, y - 5.0), c); }
    else if f < -0.8 { draw_triangle(vec2(x, y - 5.0), vec2(x + 12.0, y - 5.0), vec2(x + 6.0, y + 5.0), c); }
    else { draw_triangle(vec2(x, y - 6.0), vec2(x, y + 6.0), vec2(x + 11.0, y), c); }
    let _ = ui;
}

pub fn club_color(c: &Career, club: usize) -> (Color, Color) {
    let cl = &c.world.clubs[club];
    (hex(cl.c1), hex(cl.c2))
}

fn player_row_head(ui: &Ui, x: f32, y: f32, cols: &[(&str, f32)]) {
    for (t, cx) in cols { ui.text_semi(t, x + cx, y, 14.0, Theme::gedaempft()); }
}

fn strength_cell(ui: &Ui, c: &Career, p: usize, cx: f32, cy: f32) {
    let (lo, hi) = c.ovr_range(p);
    if hi - lo <= 1.0 { ui.pill(cx, cy, (lo + hi) / 2.0, 38.0); } else {
        let col = Theme::skala((lo + hi) / 2.0);
        rrect(cx - 30.0, cy - 11.0, 60.0, 22.0, 11.0, alpha(col, 0.25));
        rrect_line(cx - 30.0, cy - 11.0, 60.0, 22.0, 11.0, col);
        ui.text_center(&format!("{}–{}", lo, hi), cx, cy + 6.0, 15.0, Theme::text());
    }
}

// ================================================================ dispatch
pub fn draw_page(ui: &mut Ui, v: &mut View, c: &mut Career) {
    match v.page {
        Page::Uebersicht => uebersicht(ui, v, c),
        Page::Posteingang => posteingang(ui, v, c),
        Page::Kalender => kalender(ui, v, c),
        Page::Kader => kader(ui, v, c),
        Page::Aufstellung => aufstellung(ui, v, c),
        Page::Kabine => kabine(ui, v, c),
        Page::Training => training(ui, v, c),
        Page::Medizin => medizin(ui, v, c),
        Page::Praesidium => praesidium(ui, v, c),
        Page::Finanzen => finanzen(ui, v, c),
        Page::Trainer => trainer(ui, v, c),
        Page::Transfermarkt => transfermarkt(ui, v, c),
        Page::Verhandlungen => verhandlungen(ui, v, c),
        Page::Vertraege => vertraege(ui, v, c),
        Page::Scouting => scouting(ui, v, c),
        Page::Akademie => akademie(ui, v, c),
        Page::Tabellen => tabellen(ui, v, c),
        Page::Presse => presse(ui, v, c),
        Page::Spieltag => spieltag(ui, v, c),
        Page::Optionen => optionen(ui, v, c),
        Page::Ziele => ziele(ui, v, c),
        Page::Manager => manager_seite(ui, v, c),
        Page::Stadion => stadion(ui, v, c),
        Page::Personalmarkt => personalmarkt(ui, v, c),
        Page::Geschichte => geschichte(ui, v, c),
        Page::Tagebuch => tagebuch(ui, v, c),
        Page::Klima => klima_seite(ui, v, c),
        Page::Marken => marken(ui, v, c),
        Page::Kaderplanung => kaderplanung(ui, v, c),
        Page::Vergleich => vergleich(ui, v, c),
        Page::Entwicklung => entwicklung(ui, v, c),
        Page::Vorbereitung => vorbereitung(ui, v, c),
        Page::Analyse => analyse_seite(ui, v, c),
        Page::Zeitung => zeitung(ui, v, c),
        Page::Fans => fans_seite(ui, v, c),
        Page::Ranglisten => ranglisten(ui, v, c),
        Page::Vereine => vereine(ui, v, c),
        Page::Vereinswelt => vereinswelt(ui, v, c),
        Page::Portal => portal(ui, v, c),
        Page::Handbuch => handbuch(ui, v, c),
        Page::Regeln => regeln_seite(ui, v, c),
        Page::Profi => profi(ui, v, c),
        Page::Spielerprofil => spielerprofil(ui, v, c),
        Page::Zentrale => zentrale(ui, v, c),
        Page::Kooperationen => kooperationen(ui, v, c),
        Page::ZweiteMannschaft => zweite_mannschaft_tab(ui, v, c),
        Page::Jugendnetz => jugendnetz(ui, v, c),
        Page::Merkliste => merkliste(ui, v, c),
        Page::Trainerteam => trainerteam(ui, v, c),
    }
}

// ================================================================ Übersicht
fn form_squares(ui: &Ui, x: f32, y: f32, form: &[char]) {
    for (j, ch) in form.iter().enumerate() {
        let col = match ch { 'S' => Theme::gut(), 'N' => Theme::problem(), _ => Theme::gedaempft() };
        rrect(x + j as f32 * 26.0, y, 22.0, 22.0, 5.0, col);
        ui.text_center(&ch.to_string(), x + j as f32 * 26.0 + 11.0, y + 16.0, 14.0, hex(0xffffff));
    }
}

/// A speech-bubble card with an avatar (guide §3.5).
fn bubble(ui: &mut Ui, r: Rect, who: &str, role: &str, text: &str) {
    ui.card(r);
    ui.avatar(r.x + 36.0, r.y + 36.0, 22.0, who);
    ui.text_semi(who, r.x + 68.0, r.y + 32.0, 18.0, Theme::text());
    ui.text(role, r.x + 68.0, r.y + 50.0, 14.0, Theme::text2());
    let b = Rect::new(r.x + 16.0, r.y + 66.0, r.w - 32.0, r.h - 80.0);
    rrect(b.x, b.y, b.w, b.h, 14.0, if Theme::dark() { Theme::erhoehung() } else { Theme::verein_weich() });
    draw_triangle(vec2(b.x + 20.0, b.y), vec2(b.x + 34.0, b.y), vec2(b.x + 24.0, b.y - 10.0), if Theme::dark() { Theme::erhoehung() } else { Theme::verein_weich() });
    let mut y = b.y + 24.0;
    for l in ui.wrap(text, 16.0, b.w - 28.0) { if y > b.y + b.h - 6.0 { break; } ui.text(&l, b.x + 14.0, y, 16.0, Theme::text()); y += 21.0; }
}

/// Übersicht = Spieltag-Board (guide §2, §7): next match big, short table, today's tasks, co-trainer hint.
fn uebersicht(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let lw = cbreite() * 0.62;
    // next match
    let r = Rect::new(X0 + 4.0, Y0 - 40.0, lw, 300.0);
    ui.card_titled(r, crate::inhalt::t("zentrale.spiel_naechstes"), Theme::akzent());
    let table = c.world.table(c.league());
    let form_of = |club: usize| table.iter().find(|x| x.club == club).map(|x| x.form.clone()).unwrap_or_default();
    if let Some(f) = c.next_match() {
        let (h, a) = (f.h, f.a);
        for (k, club) in [h, a].iter().enumerate() {
            let cx = if k == 0 { r.x + 120.0 } else { r.x + r.w - 120.0 };
            let cl = &c.world.clubs[*club];
            ui.club_crest(cx, r.y + 104.0, 46.0, cl.id, hex(cl.c1), &cl.short);
            let nm = ui.fit(&cl.name, 22.0, 220.0);
            let w = measure_text(&nm, Some(&ui.head), 22, 1.0).width;
            ui.title(&nm, cx - w / 2.0, r.y + 178.0, 22.0, Theme::text());
            let fm = form_of(*club);
            form_squares(ui, cx - fm.len() as f32 * 13.0, r.y + 192.0, &fm);
        }
        let w = measure_text(crate::inhalt::t("ui.wort.gross_gegen"), Some(&ui.head), 34, 1.0).width;
        ui.title(crate::inhalt::t("ui.wort.gross_gegen"), r.x + r.w / 2.0 - w / 2.0, r.y + 112.0, 34.0, Theme::verein());
        let when = if f.day == c.world.day { crate::id("heute").to_string() } else { crate::tf!("zentrale.in_tagen.viele", f.day - c.world.day) };
        ui.text_center(&crate::tf!("ui.pages.anstoss_15_30", date::long(c.world.season, f.day), when), r.x + r.w / 2.0, r.y + 140.0, 16.0, Theme::text2());
        let label = if c.today_match().is_some() { crate::inhalt::t("pm.zum_spiel") } else { crate::inhalt::t("ui.pages.elf_vorbereiten") };
        if ui.button_gold(Rect::new(r.x + r.w / 2.0 - 130.0, r.y + 236.0, 260.0, 46.0), label) { v.go(if c.today_match().is_some() { Page::Spieltag } else { Page::Aufstellung }); }
    } else {
        ui.empty(Rect::new(r.x, r.y + 40.0, r.w, r.h - 40.0), crate::inhalt::t("ui.pages.die_saison_ist_gespielt_die"));
    }
    // short table
    let rt = Rect::new(X0 + lw + 16.0, Y0 - 40.0, cbreite() - lw - 12.0, 300.0);
    ui.card_titled(rt, &c.world.leagues[c.league()].name.clone(), Theme::akzent());
    let me = table.iter().position(|x| x.club == c.own).unwrap_or(0);
    let start = me.saturating_sub(2).min(table.len().saturating_sub(6));
    let played = table.iter().any(|x| x.p > 0);
    for (k, row) in table.iter().enumerate().skip(start).take(6) {
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
    let y2 = Y0 + 274.0;
    let rk = Rect::new(X0 + 4.0, y2, lw, hoehe() - y2 - 14.0);
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
    // co-trainer hint
    let rc = Rect::new(X0 + lw + 16.0, y2, cbreite() - lw - 12.0, hoehe() - y2 - 14.0);
    let op = c.co_opinion();
    let co = c.co_name.clone();
    bubble(ui, rc, &co, crate::id("Co-Trainer"), &op);
}

// ================================================================ Posteingang
fn posteingang(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // filter by kind: all, must decide, then the kind groups of postkarten.rs (Art::gruppe)
    let filters = [crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("gm.in.kz.muss"), crate::inhalt::t("menu.posteingang.reiter.6"), crate::inhalt::t("zentrale.k.kader"), crate::inhalt::t("bereich.transfers"), crate::inhalt::t("ui.anzeige.Präsidium"), crate::inhalt::t("profil.teil.8"), crate::inhalt::t("md.ui.posteingang.ungelesen")];
    tabs(ui, v, &filters);
    v.msg_filter = v.tab;
    let list: Vec<usize> = (0..c.inbox.len()).filter(|&i| {
        let m = &c.inbox[i];
        // after 120 days a message is only in „Alle“ (medien.toml posteingang.bestand_tage)
        let frisch = abs_day(m.season, m.day) + 120 > c.abs();
        match v.msg_filter { 0 => true, 1 => m.action != Action::None && !m.done && !matches!(m.action, Action::Report { .. }), 7 => !m.read, g => frisch && art_von(m).gruppe() == g }
    }).collect();
    let lw = 400.0;
    let area = Rect::new(X0 + 4.0, Y0, lw, hoehe() - Y0 - 16.0);
    ui.card(area);
    // Posteingang, five questions (1) and (2): one sentence on what waits, and this week's progress as a bar
    let abs = c.abs();
    let braucht = |m: &Msg| m.action != Action::None && !matches!(m.action, Action::Report { .. });
    let offen = c.inbox.iter().filter(|m| braucht(m) && !m.done).count();
    let heute = c.inbox.iter().filter(|m| braucht(m) && !m.done && m.pflicht > 0 && m.frist <= abs).count();
    let woche: Vec<&Msg> = c.inbox.iter().filter(|m| braucht(m) && abs_day(m.season, m.day) + 7 > abs).collect();
    let erledigt = woche.iter().filter(|m| m.done).count();
    let satz = if offen == 0 { crate::inhalt::t("ui.post.kopf_nichts").to_string() } else { crate::tf!(if offen == 1 { "ui.post.kopf_eins" } else { "ui.post.kopf" }, offen, heute) };
    ui.text_semi(&ui.fit(&satz, 15.0, area.w - 28.0), area.x + 14.0, area.y + 24.0, 15.0, if heute > 0 { Theme::problem() } else { Theme::text() });
    let anteil = if woche.is_empty() { 1.0 } else { erledigt as f32 / woche.len() as f32 };
    rrect(area.x + 14.0, area.y + 34.0, area.w - 28.0, 6.0, 3.0, Theme::ueberlagerung());
    rrect(area.x + 14.0, area.y + 34.0, (area.w - 28.0) * anteil.max(0.02), 6.0, 3.0, Theme::gut());
    ui.text(&crate::tf!("ui.post.kopf_woche", erledigt, woche.len()), area.x + 14.0, area.y + 56.0, 12.0, Theme::gedaempft());
    let oben = 64.0;
    let vis = ((area.h - oben - 10.0) / 60.0) as usize;
    let start = scroll_rows(ui, v, area, list.len(), vis);
    if list.is_empty() { ui.title(crate::inhalt::t("pe.hub.leer"), area.x + 16.0, area.y + oben + 40.0, 22.0, Theme::text2()); }
    for (k, &i) in list.iter().skip(start).take(vis).enumerate() {
        let r = Rect::new(area.x + 6.0, area.y + oben + 6.0 + k as f32 * 60.0, area.w - 22.0, 56.0);
        meldung_zeile(ui, c, r, &c.inbox[i], v.sel_msg == Some(i));
        if ui.hit(r) { v.sel_msg = Some(i); v.post_folgen = None; }
    }
    let rd = Rect::new(X0 + lw + 18.0, Y0, cbreite() - lw - 14.0, hoehe() - Y0 - 16.0);
    ui.card(rd);
    let Some(i) = v.sel_msg.filter(|&i| i < c.inbox.len()) else {
        ui.text(crate::inhalt::t("meldung.waehlen"), rd.x + 20.0, rd.y + 40.0, 19.0, Theme::text2());
        return;
    };
    c.inbox[i].read = true;
    meldung_zeichnen(ui, v, c, rd, i);
}

// ================================================================ Kalender
fn kalender(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("ui.vorb.reiter_monat"), crate::inhalt::t("ui.vorb.reiter")]);
    if v.tab == 1 { return vorbereitung_seite(ui, v, c); }
    let (_, cm, _) = date::ymd(c.world.season, c.world.day);
    if v.cal_month < 0 { v.cal_month = cm as i32; }
    let m = v.cal_month as u32;
    if ui.button(Rect::new(X0 + cbreite() - 220.0, Y0 - 44.0, 40.0, 32.0), "<", false) { v.cal_month = if m == 1 { 12 } else { m as i32 - 1 }; }
    if ui.button(Rect::new(X0 + cbreite() - 40.0, Y0 - 44.0, 40.0, 32.0), ">", false) { v.cal_month = if m == 12 { 1 } else { m as i32 + 1 }; }
    let mut first = None;
    for d in 0..date::SEASON_DAYS { let (_, mm, dd) = date::ymd(c.world.season, d); if mm == m && dd == 1 { first = Some(d); break; } }
    let Some(f0) = first else { return };
    let (yy, _, _) = date::ymd(c.world.season, f0);
    ui.title(&format!("{} {}", date::month_name(m), yy), X0 + 10.0, Y0 - 16.0, 28.0, Theme::text());
    let cw = (cbreite() - 6.0 * 8.0) / 7.0;
    let ch = (hoehe() - Y0 - 70.0) / 6.0 - 8.0;
    for (i, wdn) in [crate::inhalt::t("training.tag.0"), crate::inhalt::t("training.tag.1"), crate::inhalt::t("training.tag.2"), crate::inhalt::t("training.tag.3"), crate::inhalt::t("training.tag.4"), crate::inhalt::t("training.tag.5"), crate::inhalt::t("training.tag.6")].iter().enumerate() {
        ui.text_semi(wdn, X0 + 4.0 + i as f32 * (cw + 8.0), Y0 + 12.0, 16.0, Theme::text2());
    }
    let off = date::weekday(f0);
    let fx = c.own_fixtures();
    for d in f0..date::SEASON_DAYS {
        let (_, mm, dd) = date::ymd(c.world.season, d);
        if mm != m { break; }
        let cell = off + (dd - 1) as usize;
        let r = Rect::new(X0 + 4.0 + (cell % 7) as f32 * (cw + 8.0), Y0 + 22.0 + (cell / 7) as f32 * (ch + 8.0), cw, ch);
        let window = WINDOWS.iter().any(|(a, b)| d >= *a && d <= *b);
        let intl = INTL_BREAKS.iter().any(|(a, b)| d >= *a && d <= *b);
        rrect(r.x, r.y, r.w, r.h, 6.0, if d == c.world.day { alpha(Theme::akzent(), 0.18) } else if d < c.world.day { Theme::grund_tief() } else { Theme::flaeche() });
        if window { draw_rectangle(r.x, r.y + r.h - 4.0, r.w, 4.0, Theme::gold()); }
        if intl { draw_rectangle(r.x, r.y + r.h - 4.0, r.w, 4.0, hex(0xB48CFF)); }
        ui.text_semi(&format!("{dd}"), r.x + 10.0, r.y + 24.0, 18.0, if d == c.world.day { Theme::akzent() } else { Theme::text2() });
        if let Some((wo, f)) = fx.iter().find(|x| x.1.day == d) {
            ui.wettbewerb_logo(r.x + r.w - 16.0, r.y + 17.0, 10.0, c.comp_cid(*wo));
            let home = f.h == c.own;
            let opp = if home { f.a } else { f.h };
            let (c1, c2) = club_color(c, opp);
            let _ = c2;
            ui.club_crest(r.x + 26.0, r.y + 48.0, 15.0, c.world.clubs[opp].id, c1, &c.world.clubs[opp].short);
            let nm = ui.fit(&c.world.clubs[opp].name, 15.0, r.w - 56.0);
            ui.text(&nm, r.x + 48.0, r.y + 46.0, 15.0, Theme::text());
            let what = match fx.iter().find(|x| x.1.day == d).map(|x| x.0) { Some(Wo::Pokal(ci)) => c.world.cups[ci].name.clone(), _ => if home { crate::inhalt::t("zentrale.k.heim").into() } else { crate::inhalt::t("zentrale.k.auswaerts").into() } };
            let what = ui.fit(&what, 14.0, r.w - 56.0);
            ui.text(&what, r.x + 48.0, r.y + 63.0, 14.0, if matches!(fx.iter().find(|x| x.1.day == d).map(|x| x.0), Some(Wo::Pokal(_))) { Theme::gold() } else { Theme::gedaempft() });
            if f.hg >= 0 {
                let (gf, ga) = if home { (f.hg, f.ag) } else { (f.ag, f.hg) };
                let col = if gf > ga { Theme::gut() } else if gf < ga { Theme::problem() } else { Theme::achtung() };
                ui.text_semi(&format!("{}:{}", f.hg, f.ag), r.x + 10.0, r.y + ch - 8.0, 17.0, col);
            }
        } else if let Some(tm) = c.termine.iter().find(|tm| tm.art == 0 && tm.tag == d) {
            // a friendly (Vorbereitung): opponent, „Testspiel“, the score when played
            let g = tm.gegner;
            let (c1, _) = club_color(c, g);
            ui.club_crest(r.x + 26.0, r.y + 48.0, 15.0, c.world.clubs[g].id, c1, &c.world.clubs[g].short);
            let nm = ui.fit(&c.world.clubs[g].name, 15.0, r.w - 56.0);
            ui.text(&nm, r.x + 48.0, r.y + 46.0, 15.0, Theme::text());
            ui.text(crate::inhalt::t("ui.vorb.testspiel"), r.x + 48.0, r.y + 63.0, 14.0, hex(0x2CC5B5));
            if let Some((a, b)) = tm.ergebnis { ui.text_semi(&format!("{a}:{b}"), r.x + 10.0, r.y + ch - 8.0, 17.0, Theme::text2()); }
        } else if let Some(tm) = c.termine.iter().find(|tm| tm.art == 2 && d >= tm.tag && d < tm.tag + tm.tage) {
            // a training camp: a bar over its days, the place on the first
            draw_rectangle(r.x, r.y, r.w, 5.0, Theme::gold());
            if d == tm.tag {
                let name = crate::vorbereitung::lager_orte().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["name"].as_str()).unwrap_or("");
                ui.paragraph(&ui.fit(name, 14.0, (r.w - 20.0) * 2.0), r.x + 10.0, r.y + 46.0, 14.0, r.w - 20.0, Theme::text());
            }
        } else if let Some(tm) = c.termine.iter().find(|tm| tm.art == 1 && d >= tm.tag && d < tm.tag + crate::vorbereitung::turniere().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["tage"].as_u64()).unwrap_or(5) as u32) {
            // a trip with its tournament: a bar over all its days, the name on the first
            draw_rectangle(r.x, r.y, r.w, 5.0, hex(0x2CC5B5));
            if d == tm.tag {
                let name = crate::vorbereitung::turniere().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["name"].as_str()).unwrap_or("");
                ui.paragraph(&ui.fit(name, 14.0, (r.w - 20.0) * 2.0), r.x + 10.0, r.y + 46.0, 14.0, r.w - 20.0, Theme::text());
            }
        } else if c.plan[date::weekday(d)] != 3 || true {
            ui.text(units_t()[c.plan[date::weekday(d)] as usize], r.x + 10.0, r.y + ch - 10.0, 13.0, Theme::gedaempft());
        }
    }
    let ly = hoehe() - 26.0;
    draw_rectangle(X0 + 8.0, ly - 8.0, 20.0, 4.0, Theme::gold()); ui.text(crate::inhalt::t("termin.transferfenster"), X0 + 34.0, ly, 15.0, Theme::text2());
    draw_rectangle(X0 + 168.0, ly - 8.0, 20.0, 4.0, hex(0xB48CFF)); ui.text(crate::inhalt::t("ui.pages.laenderspielpause"), X0 + 194.0, ly, 15.0, Theme::text2());
    draw_rectangle(X0 + 368.0, ly - 8.0, 20.0, 4.0, hex(0x2CC5B5)); ui.text(crate::inhalt::t("ui.vorb.reiter"), X0 + 394.0, ly, 15.0, Theme::text2());
}

// ================================================================ Kader (Porträt-Raster) and Kabine
fn mood_words(m: f32) -> &'static str {
    if m >= 70.0 { crate::inhalt::t("ui.pages.bester_laune") } else if m >= 55.0 { crate::inhalt::t("ui.wort.zufrieden") } else if m >= 40.0 { crate::inhalt::t("ui.wort.durchwachsen") } else { crate::inhalt::t("ui.wort.unzufrieden") }
}

fn kader(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("ui.pages.portraets"), crate::inhalt::t("kalender.ansicht.liste"), crate::inhalt::t("bericht.statistik")]);
    if v.tab > 0 { kader_list(ui, v, c); return; }
    let all = c.squad();
    // Kennzahlen-Leiste in words
    let n = all.len().max(1) as f32;
    let avg_age = all.iter().map(|&p| c.world.players[p].age as f32).sum::<f32>() / n;
    let mood = all.iter().map(|&p| c.world.players[p].morale).sum::<f32>() / n;
    let inj = all.iter().filter(|&&p| c.world.players[p].inj > 0).count();
    let facts = crate::tf!("ui.pages.spieler_im_schnitt_jahre_stimmung", all.len(), format!("{:.0}", avg_age), mood_words(mood), if inj == 0 { crate::inhalt::t("ui.pages.alle_fit").to_string() } else { crate::tf!("zentrale.k.verletzt.viele", inj) });
    ui.text_right(&facts, X0 + cbreite(), Y0 - 20.0, 16.0, Theme::text2());
    let filters = [crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("aka.pos.tw"), crate::inhalt::t("achse.Abwehr"), crate::inhalt::t("gb2.vgl.mittel"), crate::inhalt::t("bericht.achse.angriff"), crate::inhalt::t("pille.verletzt")];
    if let Some(f) = ui.segment(Rect::new(X0 + 4.0, Y0, 640.0, 36.0), &filters, v.market_pos.min(5)) { v.market_pos = f; v.scroll = 0.0; v.scrolls.clear(); }
    let sq: Vec<usize> = all.into_iter().filter(|&p| { let pl = &c.world.players[p]; match v.market_pos { 1..=4 => part_of(&pl.pos) + 1 == v.market_pos, 5 => pl.inj > 0, _ => true } }).collect();
    let gw = cbreite() - 300.0;
    let ch = 156.0;
    // as many columns of at least 182 as fit, then the cards share the whole width (no gap before „Im Blick“ on wide windows)
    let cols = ((gw + 12.0) / 194.0).max(1.0) as usize;
    let cw = (gw + 12.0) / cols as f32 - 12.0;
    let rows = ((hoehe() - Y0 - 60.0) / (ch + 12.0)) as usize;
    let area = Rect::new(X0, Y0 + 46.0, gw, hoehe() - Y0 - 60.0);
    let start = scroll_rows(ui, v, area, (sq.len() + cols - 1) / cols, rows) * cols;
    if sq.is_empty() { ui.empty(area, crate::inhalt::t("ui.pages.hier_ist_gerade_niemand_gutes")); }
    for (k, &p) in sq.iter().skip(start).take(rows * cols).enumerate() {
        let pl = &c.world.players[p];
        let r = Rect::new(X0 + 4.0 + (k % cols) as f32 * (cw + 12.0), Y0 + 50.0 + (k / cols) as f32 * (ch + 12.0), cw, ch);
        let sel = v.sel_player == Some(p);
        ui.card(r);
        if sel { rrect(r.x, r.y, r.w, 4.0, 2.0, Theme::verein()); }
        if ui.hover(r) { rrect(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0, 14.0, alpha(Theme::verein(), 0.08)); }
        ui.player_photo_nr(r.x + r.w / 2.0, r.y + 46.0, 30.0, pl.id, &pl.name, pl.no, Theme::verein());
        ui.smiley(r.x + r.w / 2.0 + 26.0, r.y + 66.0, 10.0, pl.morale);
        if pl.inj > 0 { rrect(r.x + 10.0, r.y + 10.0, 22.0, 22.0, 11.0, hex(0xf472b6)); ui.text_center("+", r.x + 21.0, r.y + 27.0, 20.0, hex(0xffffff)); }
        if pl.susp > 0 { rrect(r.x + 12.0, r.y + 10.0, 14.0, 20.0, 3.0, Theme::problem()); }
        let last = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
        let nm = ui.fit(&last, 20.0, r.w - 16.0);
        let w = measure_text(&nm, Some(&ui.head), 20, 1.0).width;
        ui.title(&nm, r.x + r.w / 2.0 - w / 2.0, r.y + 104.0, 20.0, Theme::text());
        ui.pos_tag(r.x + 12.0, r.y + 126.0, pos_de(&pl.pos));
        ui.text(&crate::tf!("ui.wort.alter_kurz", pl.age), r.x + 52.0, r.y + 131.0, 14.0, Theme::text2());
        ui.pill(r.x + r.w - 28.0, r.y + 126.0, pl.ovr, 36.0);
        zuschlag_text(ui, r.x + r.w - 72.0, r.y + 131.0, &pl);
        ui.bar(r.x + 12.0, r.y + ch - 12.0, r.w - 24.0, 5.0, pl.fit / 100.0, if pl.fit >= 80.0 { Theme::gut() } else if pl.fit >= 60.0 { Theme::achtung() } else { Theme::problem() });
        if ui.hit(r) { if sel { v.modal = Modal::Player(p); } else { v.sel_player = Some(p); } }
    }
    // detail card
    let dr = Rect::new(X0 + gw + 12.0, Y0, cbreite() - gw - 8.0, hoehe() - Y0 - 14.0);
    ui.card_titled(dr, crate::inhalt::t("sc.mentor.kachel"), Theme::verein());
    let Some(p) = v.sel_player.filter(|&p| c.world.players[p].team == c.own as i32) else {
        ui.empty(Rect::new(dr.x, dr.y + 36.0, dr.w, dr.h - 36.0), crate::inhalt::t("ui.pages.tipp_auf_ein_trikot_dann"));
        return;
    };
    let pl = c.world.players[p].clone();
    ui.player_photo_nr(dr.x + 60.0, dr.y + 90.0, 36.0, pl.id, &pl.name, pl.no, Theme::verein());
    ui.smiley(dr.x + 92.0, dr.y + 116.0, 13.0, pl.morale);
    let nm = ui.fit(&pl.name, 21.0, dr.w - 120.0);
    ui.title(&nm, dr.x + 108.0, dr.y + 80.0, 21.0, Theme::text());
    ui.text(&crate::tf!("finanzen.zeile", pos_de(&pl.pos), pl.age), dr.x + 108.0, dr.y + 102.0, 15.0, Theme::text2());
    let lines = [
        crate::tf!("ui.pages.er_ist", mood_words(pl.morale)),
        if pl.inj > 0 { crate::tf!("ui.pages.faellt_noch_tage_aus", pl.inj, pl.inj_name) } else if pl.fit < 70.0 { crate::inhalt::t("ui.pages.wirkt_muede_eine_pause_taete").into() } else { crate::inhalt::t("ui.pages.koerperlich_frisch").into() },
        if pl.benched >= 3 { crate::inhalt::t("ui.pages.wartet_seit_wochen_auf_einsaetze").into() } else if pl.apps > 0 { crate::tf!("ui.pages.spiele_tore_in_dieser_saison", pl.apps, pl.goals) } else { crate::inhalt::t("ui.pages.noch_ohne_einsatz_in_dieser").into() },
        if pl.contract_end <= c.world.season { crate::inhalt::t("ui.pages.sein_vertrag_laeuft_im_sommer").into() } else { crate::tf!("ui.pages.vertrag_bis", date::START_YEAR as u32 + pl.contract_end + 1) },
    ];
    let mut y = dr.y + 160.0;
    for l in lines { y = ui.paragraph(&l, dr.x + 18.0, y, 16.0, dr.w - 36.0, Theme::text()) + 6.0; }
    ui.pill(dr.x + dr.w - 40.0, dr.y + 90.0, pl.ovr, 44.0);
    if crate::world::form_zuschlag(&pl) != 0.0 {
        zuschlag_text(ui, dr.x + dr.w - 50.0, dr.y + 122.0, &pl);
        ui.text_right(crate::inhalt::t("ui.wellen.zuschlag_hinweis"), dr.x + dr.w - 56.0, dr.y + 122.0, 12.0, Theme::gedaempft());
    }
    if ui.button_gold(Rect::new(dr.x + 16.0, dr.y + dr.h - 58.0, dr.w - 32.0, 44.0), crate::inhalt::t("ui.pages.profil_oeffnen")) { v.modal = Modal::Player(p); }
}

/// Kabine (guide §7): mood bar, unhappy / happy columns with reason chips, hierarchy pyramid.
fn kabine(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("chat.stimmung"), crate::inhalt::t("ui.pages.hierarchie")]);
    let sq = c.squad();
    let level = c.club().level;
    if v.tab == 0 {
        let happy: Vec<usize> = sq.iter().copied().filter(|&p| c.world.players[p].morale >= 50.0).collect();
        let sad: Vec<usize> = sq.iter().copied().filter(|&p| c.world.players[p].morale < 50.0).collect();
        let share = happy.len() as f32 / sq.len().max(1) as f32;
        let head = if share >= 0.8 { crate::inhalt::t("ui.pages.die_kabine_ist_gut_gelaunt") } else if share >= 0.6 { crate::inhalt::t("ui.pages.die_stimmung_ist_ordentlich") } else if share >= 0.4 { crate::inhalt::t("ui.pages.die_kabine_ist_angespannt") } else { crate::inhalt::t("ui.pages.in_der_kabine_brodelt_es") };
        ui.title(head, X0 + 8.0, Y0 + 26.0, 28.0, Theme::text());
        let bw = cbreite() - 8.0;
        rrect(X0 + 4.0, Y0 + 40.0, bw, 12.0, 6.0, Theme::problem());
        rrect(X0 + 4.0 + bw * (1.0 - share), Y0 + 40.0, bw * share, 12.0, 6.0, Theme::gut());
        ui.text(&crate::tf!("kabine.neu.unzufrieden", sad.len()), X0 + 8.0, Y0 + 72.0, 15.0, Theme::problem());
        ui.text_right(&crate::tf!("kabine.neu.zufrieden", happy.len()), X0 + cbreite() - 4.0, Y0 + 72.0, 15.0, Theme::gut());
        let colw = (cbreite() - 12.0) / 2.0;
        let today = c.abs();
        for (k, list) in [&sad, &happy].iter().enumerate() {
            let r = Rect::new(X0 + 4.0 + k as f32 * (colw + 12.0), Y0 + 86.0, colw, hoehe() - Y0 - 100.0);
            ui.card_titled(r, if k == 0 { crate::inhalt::t("ui.pages.braucht_ein_gespraech") } else { crate::inhalt::t("status.zufrieden") }, if k == 0 { Theme::problem() } else { Theme::gut() });
            let vis = ((r.h - 46.0) / 62.0) as usize;
            if list.is_empty() { ui.empty(Rect::new(r.x, r.y + 36.0, r.w, r.h - 36.0), if k == 0 { crate::inhalt::t("ui.pages.keiner_murrt_weiter_so") } else { crate::inhalt::t("ui.pages.gerade_ist_niemand_richtig_zufrieden") }); }
            for (j, &p) in list.iter().take(vis).enumerate() {
                let pl = c.world.players[p].clone();
                let y = r.y + 44.0 + j as f32 * 62.0;
                ui.player_photo_nr(r.x + 34.0, y + 28.0, 20.0, pl.id, &pl.name, pl.no, Theme::verein());
                ui.smiley(r.x + 52.0, y + 42.0, 8.0, pl.morale);
                ui.text_semi(&pl.name, r.x + 70.0, y + 22.0, 17.0, Theme::text());
                // Grund-Chips: Spielzeit, Gehalt, Rang
                let mut chips: Vec<(&str, Color)> = vec![];
                if pl.benched >= 3 { chips.push((crate::inhalt::t("hierarchie.teil.spielzeit"), Theme::problem())); } else if pl.apps > 0 { chips.push((crate::id("spielt"), Theme::gut())); }
                let fair = fair_wage(&pl, c.world.leagues[c.league()].level);
                if pl.wage < fair * 0.8 { chips.push((crate::inhalt::t("profil.gehalt"), Theme::achtung())); }
                if pl.ovr >= level + 3.0 { chips.push((crate::inhalt::t("ui.pages.leistungstraeger"), hex(0xa78bfa))); } else if pl.ovr < level - 6.0 { chips.push((crate::inhalt::t("pille.ergaenzung"), Theme::gedaempft())); }
                let mut cx = r.x + 70.0;
                for (t, col) in chips {
                    let w = ui.measure_semi(t, 13.0) + 18.0;
                    rrect(cx, y + 32.0, w, 20.0, 10.0, alpha(col, 0.18));
                    ui.text_semi(t, cx + 9.0, y + 46.0, 13.0, col);
                    cx += w + 6.0;
                }
                if k == 0 {
                    let can = today >= pl.talk_day + 7 || pl.talk_day == 0;
                    if can && ui.button(Rect::new(r.x + r.w - 130.0, y + 12.0, 116.0, 32.0), crate::inhalt::t("story.ansicht.gespraech"), false) {
                        // entwicklung/kabine.py:87: an encouraging word, 30 % now and 40 % after 3 days
                        c.kabinen_gespraech(p, crate::id("ermutigen"));
                        v.toast(&crate::tf!("ui.pages.fuehlt_sich_gehoert", pl.name), true);
                    }
                    if !can { ui.text_right(crate::inhalt::t("vg.ausgang.kurz.abgeschlossen"), r.x + r.w - 14.0, y + 32.0, 14.0, Theme::text2()); }
                }
            }
        }
    } else {
        // Hierarchie-Pyramide: captain on top, then leaders, regulars, the rest (rank by strength and age)
        let mut ranked = sq.clone();
        ranked.sort_by(|a, b| {
            let w = |p: usize| { let x = &c.world.players[p]; x.ovr + (x.age as f32 - 24.0).clamp(0.0, 8.0) * 0.8 };
            w(*b).partial_cmp(&w(*a)).unwrap()
        });
        let tiers: [(&str, usize); 4] = [("Kapitän", 1), (crate::inhalt::t("hierarchie.rolle.fuehrungsspieler"), 3), (crate::inhalt::t("ui.pages.stammkraefte"), 7), (crate::inhalt::t("ui.pages.ergaenzung_und_talente"), 12)]; // roh
        let mut idx = 0;
        let mut y = Y0 + 10.0;
        for (t, n) in tiers {
            let take: Vec<usize> = ranked.iter().skip(idx).take(n).copied().collect();
            idx += n;
            let rowh = 112.0;
            let total = take.len() as f32 * 92.0;
            let x0 = X0 + cbreite() / 2.0 - total / 2.0;
            let pw = total + 40.0;
            rrect(X0 + cbreite() / 2.0 - pw / 2.0, y, pw, rowh - 10.0, 18.0, alpha(Theme::verein(), 0.10 + 0.06 * (4 - n.min(4)) as f32));
            ui.text_semi(t, X0 + cbreite() / 2.0 - pw / 2.0 + 14.0, y + 18.0, 13.0, Theme::text2());
            for (j, &p) in take.iter().enumerate() {
                let pl = &c.world.players[p];
                let cx = x0 + 46.0 + j as f32 * 92.0;
                ui.player_photo_nr(cx, y + 50.0, 22.0, pl.id, &pl.name, pl.no, if t == "Kapitän" { Theme::gold() } else { Theme::verein() }); // roh
                ui.smiley(cx + 18.0, y + 64.0, 7.0, pl.morale);
                let last = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
                let nm = ui.fit(&last, 14.0, 88.0);
                ui.text_center(&nm, cx, y + 92.0, 14.0, Theme::text());
                if ui.hit(Rect::new(cx - 40.0, y + 20.0, 80.0, 80.0)) { v.modal = Modal::Player(p); }
            }
            y += rowh;
        }
    }
}

fn kader_list(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let sq = c.squad();
    let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    let head: Vec<(&str, f32)> = if v.tab == 1 {
        vec![(crate::id("NR"), 14.0), (crate::id("NAME"), 60.0), (crate::id("POS."), 330.0), (crate::id("ALTER"), 390.0), (crate::inhalt::t("ui.pages.staerke"), 450.0), (crate::id("POTENZIAL"), 520.0), (crate::id("FORM"), 620.0), (crate::id("FITNESS"), 680.0), (crate::id("MORAL"), 800.0), (crate::id("MARKTWERT"), 920.0), (crate::id("VERTRAG"), 1050.0)]
    } else {
        vec![(crate::id("NR"), 14.0), (crate::id("NAME"), 60.0), (crate::id("POS."), 330.0), (crate::id("SPIELE"), 400.0), (crate::id("TORE"), 480.0), (crate::id("VORLAGEN"), 550.0), (crate::inhalt::t("ui.pages.note"), 650.0), (crate::inhalt::t("ui.pages.letzte_fuenf"), 740.0), (crate::id("GELB"), 960.0), (crate::id("GEHALT"), 1030.0)]
    };
    player_row_head(ui, area.x, area.y + 26.0, &head);
    let vis = ((area.h - 44.0) / 38.0) as usize;
    let start = scroll_rows(ui, v, area, sq.len(), vis);
    for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
        let pl = &c.world.players[p];
        let r = Rect::new(area.x + 4.0, area.y + 40.0 + k as f32 * 38.0, area.w - 8.0, 36.0);
        if k % 2 == 1 { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::erhoehung(), 0.6)); }
        if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::ueberlagerung()); }
        draw_rectangle(r.x, r.y + 4.0, 3.0, r.h - 8.0, Theme::teil(&pl.pos));
        let cy = r.y + 24.0;
        ui.text_semi(&format!("{}", pl.no), r.x + 12.0, cy, 17.0, Theme::text2());
        ui.player_photo(r.x + 52.0, r.y + 18.0, 14.0, pl.id, &pl.name);
        let nm = ui.fit(&pl.name, 18.0, 210.0);
        ui.text_semi(&nm, r.x + 72.0, cy, 18.0, Theme::text());
        if pl.inj > 0 { ui.text_semi("+", r.x + 300.0, cy, 20.0, Theme::problem()); }
        if pl.susp > 0 { rrect(r.x + 300.0, cy - 13.0, 10.0, 14.0, 2.0, Theme::problem()); }
        if pl.listed { ui.text_semi(crate::inhalt::t("ui.wort.buchstabe_t"), r.x + 314.0, cy, 15.0, Theme::gold()); }
        ui.pos_tag(r.x + 326.0, cy - 5.0, pos_de(&pl.pos));
        if v.tab == 1 {
            ui.text(&format!("{}", pl.age), r.x + 396.0, cy, 17.0, Theme::text());
            ui.pill(r.x + 470.0, cy - 6.0, pl.ovr, 38.0);
            zuschlag_text(ui, r.x + 493.0, cy - 1.0, &pl);
            let stars = ((pl.pot - 50.0) / 10.0).clamp(0.5, 5.0);
            for s in 0..5 { let full = (s as f32) < stars; draw_poly(r.x + 528.0 + s as f32 * 16.0, cy - 6.0, 5, 6.0, -90.0, if full { Theme::gold() } else { Theme::ueberlagerung() }); }
            form_arrow(ui, r.x + 626.0, cy - 6.0, pl.form);
            ui.bar(r.x + 680.0, cy - 10.0, 90.0, 8.0, pl.fit / 100.0, if pl.fit >= 80.0 { Theme::gut() } else if pl.fit >= 60.0 { Theme::achtung() } else { Theme::problem() });
            ui.text(&format!("{:.0}", pl.fit), r.x + 776.0, cy, 15.0, Theme::text2());
            ui.bar(r.x + 800.0, cy - 10.0, 90.0, 8.0, pl.morale / 100.0, if pl.morale >= 60.0 { Theme::gut() } else if pl.morale >= 40.0 { Theme::achtung() } else { Theme::problem() });
            ui.text(&money(pl.value), r.x + 920.0, cy, 17.0, Theme::text());
            ui.text(&crate::tf!("zentrale.bis", date::START_YEAR as u32 + pl.contract_end + 1), r.x + 1050.0, cy, 17.0, if pl.contract_end <= c.world.season { Theme::achtung() } else { Theme::text() });
        } else {
            ui.text(&format!("{}", pl.apps), r.x + 410.0, cy, 17.0, Theme::text());
            ui.text(&format!("{}", pl.goals), r.x + 490.0, cy, 17.0, Theme::text());
            ui.text(&format!("{}", pl.assists), r.x + 570.0, cy, 17.0, Theme::text());
            if pl.rn > 0 { ui.note_pill(r.x + 676.0, cy - 6.0, pl.avg_rating()); } else { ui.text("–", r.x + 670.0, cy, 17.0, Theme::gedaempft()); }
            for (j, rt) in pl.last.iter().enumerate() { let h = (rt - 3.0) / 7.0 * 24.0; draw_rectangle(r.x + 744.0 + j as f32 * 26.0, cy + 2.0 - h, 18.0, h, Theme::note(*rt)); }
            ui.text(&format!("{}", pl.yellow), r.x + 970.0, cy, 17.0, Theme::achtung());
            ui.text(&money(pl.wage), r.x + 1030.0, cy, 17.0, Theme::text());
        }
        if ui.hit(r) { v.modal = Modal::Player(p); }
    }
}

// ================================================================ Aufstellung
fn aufstellung(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("gb2.reiter.taktik"), crate::inhalt::t("ui.pages.standards_und_vorlagen"), crate::inhalt::t("bericht.gegner"), crate::inhalt::t("ui.pages.rollen_und_anweisungen")]);
    if v.tab == 3 { rollen_seite(ui, v, c); return; }
    if v.tab == 1 { standards_seite(ui, v, c); return; }
    if v.tab == 2 { gegner_seite(ui, c); return; }
    // left: Meine Taktik
    let lt = Rect::new(X0 + 4.0, Y0, 280.0, hoehe() - Y0 - 16.0);
    ui.card(lt);
    ui.title(crate::inhalt::t("tk.kompakt.meine"), lt.x + 14.0, lt.y + 30.0, 22.0, Theme::text());
    ui.text(crate::inhalt::t("ui.pages.gilt_fuer_alle_phasen"), lt.x + 14.0, lt.y + 50.0, 14.0, Theme::text2());
    let mut y = lt.y + 66.0;
    ui.text_semi(crate::inhalt::t("ui.wort.gross_formation"), lt.x + 14.0, y + 10.0, 14.0, Theme::text2());
    if let Some(f) = ui.segment(Rect::new(lt.x + 10.0, y + 18.0, lt.w - 20.0, 30.0), &FORMATIONS, c.formation) {
        c.formation = f;
        c.auto_lineup();
        c.world.clubs[c.own].fam = (c.world.clubs[c.own].fam - 2.5).max(0.0);
    }
    y += 60.0;
    ui.text_semi(crate::inhalt::t("ui.wort.gross_grundhaltung"), lt.x + 14.0, y + 10.0, 14.0, Theme::text2());
    if let Some(s) = ui.segment(Rect::new(lt.x + 10.0, y + 18.0, lt.w - 20.0, 28.0), &[crate::inhalt::t("taktik.schreiben.anweisung_art.3"), crate::inhalt::t("tm.mentalitaet.ausgewogen"), crate::inhalt::t("tm.mentalitaet.offensiv")], c.tactic.haltung as usize) { c.tactic.haltung = s as u8; }
    y += 58.0;
    let rows: [(&str, [&str; 2], u8); 5] = [(crate::inhalt::t("bericht.achse.angriff"), [crate::inhalt::t("gb.stilfeld.offensivstil.0"), crate::inhalt::t("spielbericht.ballbesitz")], c.tactic.angriff), (crate::inhalt::t("taktik.wert.aufbau"), [crate::inhalt::t("tk.kompakt.k.aufbau.0"), crate::inhalt::t("tk.kompakt.k.aufbau.1")], c.tactic.aufbau), (crate::inhalt::t("bericht.breite"), [crate::inhalt::t("tk.kompakt.k.angriffsbereich.0"), crate::inhalt::t("tk.kompakt.k.angriffsbereich.1")], c.tactic.breite), (crate::inhalt::t("achse.Abwehr"), [crate::inhalt::t("tk.kompakt.k.defensivstil.0"), crate::inhalt::t("tk.kompakt.k.defensivstil.1")], c.tactic.abwehr), (crate::inhalt::t("bericht.pressing"), [crate::inhalt::t("programm.intensitaet.hart"), crate::inhalt::t("ton.ruhig")], c.tactic.pressing)];
    for (i, (name, opts, val)) in rows.iter().enumerate() {
        ui.text(name, lt.x + 14.0, y + 20.0, 16.0, Theme::text());
        if let Some(s) = ui.segment(Rect::new(lt.x + 96.0, y + 2.0, lt.w - 106.0, 26.0), opts, *val as usize) {
            match i { 0 => c.tactic.angriff = s as u8, 1 => c.tactic.aufbau = s as u8, 2 => c.tactic.breite = s as u8, 3 => c.tactic.abwehr = s as u8, _ => c.tactic.pressing = s as u8 }
        }
        y += 34.0;
    }
    for (name, val, lo, hi) in [("Abwehrlinie", c.tactic.linie, "tief", "hoch"), (crate::inhalt::t("taktik.wert.kompaktheit"), c.tactic.kompakt, "locker", "kompakt")] { // roh
        ui.text(name, lt.x + 14.0, y + 18.0, 16.0, Theme::text());
        ui.text_right(&format!("{val}"), lt.x + lt.w - 14.0, y + 18.0, 16.0, Theme::text());
        if let Some(nv) = ui.slider(Rect::new(lt.x + 50.0, y + 24.0, lt.w - 100.0, 16.0), val, 1, 10) { if name == "Abwehrlinie" { c.tactic.linie = nv; } else { c.tactic.kompakt = nv; } } // roh
        ui.text(lo, lt.x + 14.0, y + 37.0, 13.0, Theme::gedaempft());
        ui.text_right(hi, lt.x + lt.w - 14.0, y + 37.0, 13.0, Theme::gedaempft());
        y += 50.0;
    }
    let fam = c.club().fam;
    // low windows (760 high, 21:9): the game idea in one line, the explanation left out, so nothing runs out of the card
    if lt.y + lt.h - y < 112.0 {
        ui.text_semi(&crate::tf!("ui.pages.spielidee_kurz", ideas_t()[c.tactic.idea()]), lt.x + 14.0, y + 16.0, 15.0, Theme::akzent());
        ui.text(&crate::tf!("ui.pages.eingespielt", format!("{:.0}", fam)), lt.x + 14.0, y + 38.0, 14.0, Theme::text());
        ui.bar(lt.x + 14.0, y + 46.0, lt.w - 28.0, 6.0, fam / 100.0, Theme::akzent());
    } else {
        ui.text_semi(crate::inhalt::t("ui.wort.gross_spielidee"), lt.x + 14.0, y + 14.0, 14.0, Theme::text2());
        ui.title(ideas_t()[c.tactic.idea()], lt.x + 14.0, y + 40.0, 22.0, Theme::akzent());
        ui.text(crate::inhalt::t("ui.pages.daraus_folgt_wer_ins_system"), lt.x + 14.0, y + 60.0, 14.0, Theme::gedaempft());
        ui.text(&crate::tf!("ui.pages.eingespielt", format!("{:.0}", fam)), lt.x + 14.0, y + 86.0, 16.0, Theme::text());
        ui.bar(lt.x + 14.0, y + 94.0, lt.w - 28.0, 6.0, fam / 100.0, Theme::akzent());
    }
    // middle: pitch
    let mid_x = lt.x + lt.w + 14.0;
    let mw = 560.0;
    let xi = c.current_xi();
    let st = c.team_strength();
    ui.title(crate::inhalt::t("aufstellung.startelf"), mid_x, Y0 + 22.0, 22.0, Theme::text());
    ui.text_semi(&crate::tf!("ui.pages.staerke_2", format!("{:.0}", st)), mid_x + 90.0, Y0 + 22.0, 18.0, Theme::gut());
    let mut co = c.co_lineup;
    if ui.button(Rect::new(mid_x + mw - 210.0, Y0 + 2.0, 210.0, 30.0), if co { crate::inhalt::t("ui.pages.co_trainer_stellt_auf") } else { crate::inhalt::t("ui.pages.co_trainer_stellt_auf_2") }, co) { co = !co; c.co_lineup = co; if co { c.auto_lineup(); } }
    let pitch = Rect::new(mid_x, Y0 + 40.0, mw, hoehe() - Y0 - 90.0);
    rrect(pitch.x, pitch.y, pitch.w, pitch.h, 8.0, hex(0x1E5A35));
    ui.rahmen(pitch);
    for i in 0..8 { if i % 2 == 0 { draw_rectangle(pitch.x + 6.0, pitch.y + 6.0 + i as f32 * (pitch.h - 12.0) / 8.0, pitch.w - 12.0, (pitch.h - 12.0) / 8.0, hex(0x226440)); } }
    let lc = Color::new(1.0, 1.0, 1.0, 0.35);
    draw_rectangle_lines(pitch.x + 6.0, pitch.y + 6.0, pitch.w - 12.0, pitch.h - 12.0, 2.0, lc);
    draw_line(pitch.x + 6.0, pitch.y + pitch.h / 2.0, pitch.x + pitch.w - 6.0, pitch.y + pitch.h / 2.0, 2.0, lc);
    draw_circle_lines(pitch.x + pitch.w / 2.0, pitch.y + pitch.h / 2.0, 60.0, 2.0, lc);
    draw_rectangle_lines(pitch.x + pitch.w / 2.0 - 120.0, pitch.y + 6.0, 240.0, 80.0, 2.0, lc);
    draw_rectangle_lines(pitch.x + pitch.w / 2.0 - 120.0, pitch.y + pitch.h - 86.0, 240.0, 80.0, 2.0, lc);
    let spots = formation_spots(c.formation);
    let mut swap: Option<(usize, usize)> = None;
    for (i, (slot, p, e)) in xi.iter().enumerate() {
        let (sx, sy) = spots[i];
        let cx = pitch.x + 40.0 + sx * (pitch.w - 80.0);
        let cy = pitch.y + pitch.h - 50.0 - sy * (pitch.h - 100.0);
        let r = Rect::new(cx - 46.0, cy - 40.0, 92.0, 82.0);
        let pl = &c.world.players[*p];
        let sel = v.pick == Some(i);
        rrect(r.x, r.y, r.w, r.h, 6.0, if sel { alpha(Theme::akzent(), 0.35) } else { Color::new(0.07, 0.08, 0.09, 0.92) });
        if ui.hover(r) { rrect_line(r.x, r.y, r.w, r.h, 6.0, Theme::akzent()); }
        rrect(r.x + 4.0, r.y + 4.0, 22.0, 18.0, 3.0, Theme::akzent());
        ui.text_center(&format!("{}", pl.no), r.x + 15.0, r.y + 18.0, 14.0, hex(0x10141c));
        rrect(r.x + r.w - 30.0, r.y + 4.0, 26.0, 18.0, 3.0, Theme::stufe(*e));
        ui.text_center(&format!("{:.0}", e), r.x + r.w - 17.0, r.y + 18.0, 14.0, hex(0x10141c));
        ui.player_photo(cx, r.y + 32.0, 14.0, pl.id, &pl.name);
        draw_circle_lines(cx, r.y + 32.0, 14.0, 2.0, Theme::akzent());
        let last = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
        let nm = ui.fit(&last, 15.0, 86.0);
        ui.text_center(&nm, cx, r.y + 60.0, 15.0, hex(0xffffff));
        let tag = pos_de(slot);
        ui.text_center(tag, cx, r.y + 74.0, 12.0, if pos_penalty(&pl.pos, slot) > 0.0 { Theme::achtung() } else { Theme::teil(slot) });
        draw_rectangle(r.x + 8.0, r.y + r.h - 4.0, (r.w - 16.0) * pl.fit / 100.0, 2.0, if pl.fit > 75.0 { Theme::gut() } else { Theme::achtung() });
        if ui.hit(r) {
            if let Some(a) = v.pick { if a != i { swap = Some((a, i)); } v.pick = None; } else { v.pick = Some(i); }
        }
    }
    ui.text(&crate::tf!("ui.pages.abwehrlinie_kompaktheit_pressing", c.tactic.linie, c.tactic.kompakt, if c.tactic.breite == 0 { crate::inhalt::t("ui.wort.breit") } else { crate::inhalt::t("ui.wort.eng") }, if c.tactic.pressing == 0 { crate::inhalt::t("ui.wort.hart") } else { crate::inhalt::t("ui.wort.ruhig") }), mid_x + 8.0, pitch.y + pitch.h + 22.0, 15.0, Theme::text2());
    ui.text(crate::inhalt::t("ui.pages.spieler_anklicken_dann_einen_zweiten"), mid_x + 8.0, pitch.y + pitch.h + 42.0, 14.0, Theme::gedaempft());
    let mut cur: Vec<usize> = xi.iter().map(|x| x.1).collect();
    if let Some((a, b)) = swap { cur.swap(a, b); c.xi = cur.clone(); c.co_lineup = false; }
    // right: bench and co-trainer
    let rx = mid_x + mw + 14.0;
    let rw = breite() - 18.0 - rx;
    let rr = Rect::new(rx, Y0, rw, hoehe() - Y0 - 16.0);
    ui.card(rr);
    ui.title(crate::inhalt::t("ui.pages.bank_und_reserve"), rr.x + 12.0, rr.y + 28.0, 20.0, Theme::text());
    let others: Vec<usize> = c.squad().into_iter().filter(|p| !cur.contains(p)).collect();
    // every player of bench and reserve: a scrollable list above the assistant's advice
    let liste = Rect::new(rr.x, rr.y + 44.0, rr.w, rr.h - 230.0 - 50.0);
    let vis = ((liste.h / 32.0) as usize).max(1);
    let start = scroll_rows(ui, v, liste, others.len(), vis);
    let mut yy = rr.y + 44.0;
    for &p in others.iter().skip(start).take(vis) {
        let pl = &c.world.players[p];
        let r = Rect::new(rr.x + 6.0, yy, rr.w - 24.0, 30.0);
        if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::erhoehung()); }
        ui.pos_tag(r.x + 4.0, r.y + 15.0, pos_de(&pl.pos));
        ui.player_photo(r.x + 54.0, r.y + 15.0, 12.0, pl.id, &pl.name);
        let nm = ui.fit(&pl.name, 16.0, r.w - 136.0);
        ui.text(&nm, r.x + 72.0, r.y + 21.0, 16.0, if pl.available() { Theme::text() } else { Theme::problem() });
        ui.pill(r.x + r.w - 22.0, r.y + 15.0, pl.ovr, 32.0);
        zuschlag_text(ui, r.x + r.w - 60.0, r.y + 20.0, &pl);
        if ui.hit(r) {
            if let Some(a) = v.pick { if a < cur.len() { cur[a] = p; c.xi = cur.clone(); c.co_lineup = false; } v.pick = None; } else { v.modal = Modal::Player(p); }
        }
        yy += 32.0;
    }
    let cy = rr.y + rr.h - 230.0;
    draw_rectangle(rr.x + 10.0, cy, rr.w - 20.0, 1.0, Theme::kontur());
    ui.text_semi(&crate::tf!("ui.pages.co_trainer_rat", c.co_name), rr.x + 12.0, cy + 24.0, 14.0, Theme::akzent());
    let op = c.co_opinion();
    let mut ty = cy + 46.0;
    for l in ui.wrap(&op, 15.0, rr.w - 24.0).into_iter().take(8) { ui.text(&l, rr.x + 12.0, ty, 15.0, Theme::text()); ty += 19.0; }
    if ui.button(Rect::new(rr.x + 12.0, rr.y + rr.h - 44.0, rr.w - 24.0, 34.0), crate::inhalt::t("ui.pages.vorschlag_uebernehmen"), false) { c.auto_lineup(); v.toast(crate::inhalt::t("ui.pages.aufstellung_vom_co_trainer_uebernommen"), true); }
}

// ================================================================ Training
fn training(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("menu.training.reiter.0"), crate::inhalt::t("gm.tr.kz.einzel"), crate::inhalt::t("ui.main.trainerteam")]);
    if v.tab == 2 { trainerteam(ui, v, c); return; }
    let y = Y0;
    let w4 = (cbreite() - 36.0) / 4.0;
    let days = c.plan.iter().filter(|&&u| u >= 2).count();
    let tired = c.squad().iter().filter(|&&p| c.world.players[p].fit < 60.0).count();
    let progs = c.squad().iter().filter(|&&p| c.world.players[p].program > 0).count();
    kpi(ui, Rect::new(X0 + 4.0, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_trainingstage"), &format!("{days}"), crate::inhalt::t("gm.tr.kz.tage.unter"), Theme::gut());
    kpi(ui, Rect::new(X0 + 16.0 + w4, y, w4, 92.0), crate::inhalt::t("ui.pages.hohe_belastung"), &format!("{tired}"), if tired == 0 { crate::inhalt::t("gm.tr.kz.belastung.ok") } else { crate::inhalt::t("ui.pages.spieler_unter_60_fitness") }, if tired == 0 { Theme::gut() } else { Theme::achtung() });
    kpi(ui, Rect::new(X0 + 28.0 + 2.0 * w4, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_einzelprogramme"), &crate::tf!("ui.pages.6", progs), crate::inhalt::t("ui.pages.fokus_oder_extraschicht"), Theme::akzent());
    kpi(ui, Rect::new(X0 + 40.0 + 3.0 * w4, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_eingespielt"), &format!("{:.0} %", c.club().fam), crate::inhalt::t("ui.pages.taktiktraining_hilft"), Theme::akzent());
    let y = y + 106.0;
    let mut co = c.co_training;
    rrect(X0 + 4.0, y, 120.0, 24.0, 12.0, Theme::erhoehung());
    ui.text(if co { crate::inhalt::t("ui.pages.co_trainer_plant") } else { crate::inhalt::t("modus.manuell") }, X0 + 14.0, y + 17.0, 14.0, Theme::text2());
    if ui.button(Rect::new(X0 + 134.0, y - 2.0, 230.0, 28.0), if co { crate::inhalt::t("ui.pages.selbst_planen") } else { crate::inhalt::t("ui.pages.an_den_co_trainer_abgeben") }, false) { co = !co; c.co_training = co; }
    // what the plan does for development: intensity × repetition (entwicklung.rs plan_faktor), and the legend of the list
    ui.text(&if v.tab == 0 { crate::tf!("ui.ent.plan_wirkung", format!("{:.2}", c.plan_faktor())) } else { crate::inhalt::t("ui.ent.legende").to_string() }, X0 + 380.0, y + 17.0, 14.0, Theme::gedaempft());
    if v.tab == 0 {
        let y = y + 36.0;
        let dw = (cbreite() - 6.0 * 8.0) / 7.0;
        for d in 0..7 {
            let r = Rect::new(X0 + 4.0 + d as f32 * (dw + 8.0), y, dw, 76.0);
            ui.card(r);
            let today = date::weekday(c.world.day) == d;
            ui.text_semi([crate::inhalt::t("training.tag.0"), crate::inhalt::t("training.tag.1"), crate::inhalt::t("training.tag.2"), crate::inhalt::t("training.tag.3"), crate::inhalt::t("training.tag.4"), crate::inhalt::t("training.tag.5"), crate::inhalt::t("training.tag.6")][d], r.x + 10.0, r.y + 22.0, 17.0, if today { Theme::akzent() } else { Theme::text() });
            let u = c.plan[d] as usize;
            let (fit, inj, dev, _) = unit_effect(u as u8);
            ui.text(units_t()[u], r.x + 10.0, r.y + 46.0, 14.0, Theme::text2());
            ui.text(&crate::tf!("ui.pages.fitness_risiko", format!("{:+.1}", fit), if inj >= 2.0 { crate::inhalt::t("ui.wort.hoch") } else if inj >= 1.0 { crate::inhalt::t("ui.wort.normal") } else { crate::inhalt::t("ui.wort.niedrig") }), r.x + 10.0, r.y + 66.0, 12.0, if dev > 1.0 { Theme::achtung() } else { Theme::gedaempft() });
        }
        let mut yy = y + 90.0;
        for d in 0..7 {
            let wd = [crate::inhalt::t("training.tag.0"), crate::inhalt::t("training.tag.1"), crate::inhalt::t("training.tag.2"), crate::inhalt::t("training.tag.3"), crate::inhalt::t("training.tag.4"), crate::inhalt::t("training.tag.5"), crate::inhalt::t("training.tag.6")][d];
            ui.text_semi(wd, X0 + 10.0, yy + 22.0, 17.0, Theme::text());
            if !c.co_training {
                if let Some(u) = ui.segment(Rect::new(X0 + 120.0, yy + 2.0, cbreite() - 120.0, 30.0), &units_t(), c.plan[d] as usize) { c.plan[d] = u as u8; }
            } else { ui.text(units_t()[c.plan[d] as usize], X0 + 130.0, yy + 22.0, 17.0, Theme::text2()); }
            yy += 38.0;
        }
        ui.text(crate::inhalt::t("ui.pages.intensive_einheiten_foerdern_die_entwick"), X0 + 10.0, yy + 18.0, 14.0, Theme::gedaempft());
    } else {
        let sq = c.squad();
        let area = Rect::new(X0 + 4.0, y + 36.0, cbreite(), hoehe() - y - 52.0);
        ui.card(area);
        let vis = ((area.h - 10.0) / 40.0) as usize;
        let start = scroll_rows(ui, v, area, sq.len(), vis);
        let used = sq.iter().filter(|&&p| c.world.players[p].program > 0).count();
        for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
            let r = Rect::new(area.x + 6.0, area.y + 6.0 + k as f32 * 40.0, area.w - 12.0, 36.0);
            let (name, pos, age, ovr, pot, prog) = { let pl = &c.world.players[p]; (pl.name.clone(), pl.pos.clone(), pl.age, pl.ovr, pl.pot, pl.program) };
            ui.pos_tag(r.x + 6.0, r.y + 18.0, pos_de(&pos));
            ui.text_semi(&name, r.x + 50.0, r.y + 24.0, 17.0, Theme::text());
            ui.text(&crate::tf!("kue.jahre", age), r.x + 290.0, r.y + 24.0, 16.0, Theme::text2());
            ui.pill(r.x + 360.0, r.y + 18.0, ovr, 36.0);
            ui.text(&crate::tf!("aka.talent.potenzial", format!("{:.0}", pot)), r.x + 390.0, r.y + 24.0, 15.0, Theme::text2());
            let opts: &[&str] = if age <= 23 { &[crate::inhalt::t("ui.pages.kein_programm"), crate::inhalt::t("ui.pages.fokus"), crate::inhalt::t("ui.pages.extraschicht")] } else { &[crate::inhalt::t("ui.pages.kein_programm"), crate::inhalt::t("ui.pages.fokus")] };
            if let Some(s) = ui.segment(Rect::new(r.x + 500.0, r.y + 3.0, 300.0, 30.0), opts, prog as usize) {
                if s == 0 || prog > 0 || used < 6 { c.world.players[p].program = s as u8; } else { v.toast(crate::inhalt::t("ui.pages.hoechstens_sechs_einzelprogramme_gleichz"), false); }
            }
            // focus group of the programme (×1,7 on its abilities, ×0,95 on the rest); a click goes to the next group
            if prog == 1 {
                let fg = crate::entwicklung::fokusgruppen();
                let name = crate::entwicklung::fokus_von(&c.world.players[p]);
                let auto = c.world.players[p].ent.fokus == 0;
                let txt = crate::tf!(if auto { "ui.ent.fokus_auto" } else { "ui.ent.fokus_gewaehlt" }, crate::inhalt::t(&format!("ui.ent.fokus.{name}")));
                if ui.button(Rect::new(r.x + 810.0, r.y + 3.0, 116.0, 30.0), &ui.fit(&txt, 14.0, 108.0), false) { let e = &mut c.world.players[p].ent; e.fokus = (e.fokus + 1) % (fg.len() as u8 + 1); }
            } else if prog == 2 { ui.text(&ui.fit(crate::inhalt::t("ui.ent.extraschicht"), 13.0, 120.0), r.x + 812.0, r.y + 24.0, 13.0, Theme::gedaempft()); }
            // load as a traffic light (entwicklung.toml belastung.zonen) and the expected development per season
            let last = c.world.players[p].ps.last;
            let zone = crate::mensch::arr("entwicklung", "belastung.zonen").iter().position(|z| last < z[1].as_f64().unwrap_or(101.0) as f32).unwrap_or(3);
            let zc = [Theme::gedaempft(), Theme::gut(), Theme::achtung(), Theme::problem()][zone.min(3)];
            draw_circle(r.x + 940.0, r.y + 18.0, 6.0, zc);
            ui.text(crate::inhalt::t(&format!("ui.ent.zone.{zone}")), r.x + 952.0, r.y + 23.0, 13.0, Theme::text2());
            entwicklung_pfeil(ui, r.x + 1040.0, r.y + 18.0, c.ent_erwartung(p));
        }
    }
}

// ================================================================ Medizin
fn medizin(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("gl.kz.verletzte"), crate::inhalt::t("vorschau.belastung")]);
    let mut sq = c.squad();
    let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    if v.tab == 0 {
        sq.retain(|&p| c.world.players[p].inj > 0);
        if sq.is_empty() { ui.title(crate::inhalt::t("pk.karte.keine_ausfaelle"), area.x + 20.0, area.y + 50.0, 28.0, Theme::gut()); ui.text(crate::inhalt::t("ui.pages.der_mannschaftsarzt_meldet_keine_ausfael"), area.x + 20.0, area.y + 80.0, 18.0, Theme::text2()); }
        for (k, &p) in sq.iter().enumerate() {
            let pl = &c.world.players[p];
            let y = area.y + 20.0 + k as f32 * 56.0;
            ui.pos_tag(area.x + 16.0, y + 18.0, pos_de(&pl.pos));
            ui.player_photo(area.x + 68.0, y + 18.0, 15.0, pl.id, &pl.name);
            ui.text_semi(&pl.name, area.x + 90.0, y + 24.0, 20.0, Theme::text());
            ui.text(&pl.inj_name, area.x + 340.0, y + 24.0, 18.0, Theme::problem());
            ui.text(&crate::tf!("ui.pages.noch_tage_zurueck_am", pl.inj, date::short(c.world.season, (c.world.day + pl.inj as u32).min(364))), area.x + 620.0, y + 24.0, 17.0, Theme::text2());
            ui.bar(area.x + 60.0, y + 34.0, 900.0, 4.0, 1.0 - pl.inj as f32 / 120.0, Theme::problem());
        }
    } else {
        sq.sort_by(|a, b| c.world.players[*a].fit.partial_cmp(&c.world.players[*b].fit).unwrap());
        let vis = ((area.h - 20.0) / 36.0) as usize;
        let start = scroll_rows(ui, v, area, sq.len(), vis);
        for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
            let pl = &c.world.players[p];
            let y = area.y + 14.0 + k as f32 * 36.0;
            ui.pos_tag(area.x + 16.0, y + 14.0, pos_de(&pl.pos));
            ui.player_photo(area.x + 66.0, y + 14.0, 13.0, pl.id, &pl.name);
            ui.text(&pl.name, area.x + 86.0, y + 20.0, 17.0, Theme::text());
            let col = if pl.fit >= 80.0 { Theme::gut() } else if pl.fit >= 60.0 { Theme::achtung() } else { Theme::problem() };
            ui.bar(area.x + 320.0, y + 10.0, 400.0, 10.0, pl.fit / 100.0, col);
            ui.text(&format!("{:.0} %", pl.fit), area.x + 730.0, y + 20.0, 16.0, col);
            ui.text(if pl.fit < 60.0 { crate::inhalt::t("ui.pages.pause_empfohlen") } else if pl.fit < 80.0 { crate::inhalt::t("ui.wort.belastet") } else { crate::inhalt::t("ui.wort.frisch") }, area.x + 820.0, y + 20.0, 16.0, Theme::text2());
        }
    }
}

fn standards_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let r = Rect::new(X0 + 4.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0);
    ui.card_titled(r, crate::inhalt::t("ui.pages.standards_und_kapitaen"), Theme::akzent());
    for (k, n) in [crate::inhalt::t("gb.schluessel.standard_elfmeter"), crate::inhalt::t("gb.schluessel.standard_freistoss_lang"), crate::inhalt::t("beob.stat.ecken"), "Kapitän"].iter().enumerate() { // roh
        let yy = r.y + 70.0 + k as f32 * 56.0;
        ui.text_semi(n, r.x + 14.0, yy, 18.0, Theme::text());
        let wer = c.schuetze(k as u8).map(|p| c.world.players[p].name.clone()).unwrap_or("–".into());
        ui.text(&wer, r.x + 150.0, yy, 18.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 140.0, yy - 20.0, 126.0, 30.0), crate::inhalt::t("ui.pages.wechseln"), false) { c.schuetze_waehlen(k as u8); }
    }
    ui.paragraph(crate::inhalt::t("ui.pages.ohne_eigene_wahl_uebernimmt_der"), r.x + 14.0, r.y + 320.0, 15.0, r.w - 28.0, Theme::gedaempft());
    let r2 = Rect::new(X0 + cbreite() * 0.5 + 10.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0);
    ui.card_titled(r2, crate::inhalt::t("ui.wort.taktik_vorlagen"), Theme::gold());
    if ui.button(Rect::new(r2.x + 14.0, r2.y + 50.0, 260.0, 32.0), crate::inhalt::t("ui.pages.aktuelle_taktik_ablegen"), false) { match c.vorlage_speichern() { Ok(()) => v.toast(crate::inhalt::t("ui.pages.abgelegt"), true), Err(e) => v.toast(&e, false) } }
    let mut del = None;
    for (i, (n, _, _)) in c.std.vorlagen.clone().iter().enumerate() {
        let yy = r2.y + 110.0 + i as f32 * 46.0;
        ui.text(n, r2.x + 14.0, yy, 17.0, Theme::text());
        if ui.button(Rect::new(r2.x + r2.w - 250.0, yy - 20.0, 110.0, 30.0), crate::inhalt::t("ui.pages.laden"), false) { c.vorlage_laden(i); v.toast(crate::inhalt::t("ui.pages.taktik_geladen"), true); }
        if ui.button(Rect::new(r2.x + r2.w - 130.0, yy - 20.0, 116.0, 30.0), crate::inhalt::t("md.ui.mail.loeschen"), false) { del = Some(i); }
    }
    if let Some(i) = del { c.std.vorlagen.remove(i); }
    let yy = r2.y + 360.0;
    ui.text_semi(crate::inhalt::t("ui.pages.urteil_des_co_trainers"), r2.x + 14.0, yy, 16.0, Theme::akzent());
    for (k, l) in c.taktikurteil().iter().enumerate().take(4) { ui.paragraph(l, r2.x + 14.0, yy + 26.0 + k as f32 * 44.0, 15.0, r2.w - 28.0, Theme::text2()); }
}
fn gegner_seite(ui: &mut Ui, c: &mut Career) {
    let r = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    match c.gegnerbericht() {
        Some((gn, z)) => {
            ui.card_titled(r, &crate::tf!("ui.pages.gegnerbericht", gn), Theme::akzent());
            for (k, l) in z.iter().enumerate() { ui.paragraph(l, r.x + 14.0, r.y + 64.0 + k as f32 * 46.0, 18.0, r.w - 28.0, if l.starts_with("Plan") { Theme::gold() } else { Theme::text() }); } // roh
        }
        None => { ui.card_titled(r, crate::inhalt::t("gegner.bericht"), Theme::akzent()); ui.text(crate::inhalt::t("vorbereitung.kein_spiel"), r.x + 14.0, r.y + 64.0, 18.0, Theme::text2()); }
    }
}
fn trainerteam(ui: &mut Ui, v: &mut View, c: &mut Career) {
    c.stab_an();
    // only the people employed; applicants are on the staff market (Personalmarkt)
    let r = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(r, crate::inhalt::t("ui.pages.unser_trainerteam"), Theme::akzent());
    ui.text(&crate::tf!("ui.pages.trainingsqualitaet_medizin_neue_leute_fi", format!("{:.0}", c.trainingsqualitaet()), format!("{:.0}", c.medizinqualitaet())), r.x + 14.0, r.y + 56.0, 16.0, Theme::text2());
    let spalte = (r.w - 28.0) / 2.0;
    for i in 0..c.stab.len() {
        if i >= c.stab.len() { break; }
        let (sx, yy) = (r.x + 14.0 + (i % 2) as f32 * spalte, r.y + 96.0 + (i / 2) as f32 * 52.0);
        if yy > r.y + r.h - 30.0 { break; }
        let s = c.stab[i].clone();
        ui.text_semi(&s.name, sx, yy, 17.0, Theme::text());
        let bereich = if s.rolle == 0 { format!(" ({})", crate::personal::bereiche_t()[s.bereich as usize % 5]) } else { String::new() };
        let jahre = s.bis.saturating_sub(c.abs()) / crate::date::SEASON_DAYS;
        ui.text(&crate::tf!("ui.pages.koennen_im_jahr_vertrag_noch", crate::personal::rollen_de_t()[s.rolle as usize % 5], bereich, format!("{:.0}", s.f), money(s.gehalt), jahre.max(1), if jahre <= 1 { crate::inhalt::t("vertraege.jahr") } else { crate::inhalt::t("gl.kd.alter.unter") }), sx, yy + 20.0, 14.0, Theme::text2());
        if ui.button(Rect::new(sx + spalte - 130.0, yy - 14.0, 106.0, 28.0), crate::inhalt::t("aka.entlassen.ja"), false) { c.stab_entlassen(i); v.toast(crate::inhalt::t("aka.entlassen.ja"), true); break; }
    }
}

/// The staff market: applicants and free people for every role, with their values, salary and a hire button.
fn personalmarkt(ui: &mut Ui, v: &mut View, c: &mut Career) {
    c.stab_an();
    let w = (cbreite() - 24.0) / 3.0;
    let karte = |k: usize, titel: &str, ui: &mut Ui| { let r = Rect::new(X0 + 4.0 + k as f32 * (w + 10.0), Y0, w, hoehe() - Y0 - 16.0); ui.card_titled(r, titel, if k == 1 { Theme::gold() } else { Theme::akzent() }); r };
    // name, value lines and salary on the left; the right column stays free for the button
    let zeile = |ui: &mut Ui, r: Rect, yy: f32, name: &str, werte: &[String], gehalt: f64| -> bool {
        ui.text_semi(&ui.fit(name, 17.0, r.w - 140.0), r.x + 14.0, yy, 17.0, Theme::text());
        for (k, w) in werte.iter().chain(std::iter::once(&crate::tf!("uebersicht.gehalt_jahr", money(gehalt)))).enumerate() {
            ui.text(&ui.fit(w, 14.0, r.w - 140.0), r.x + 14.0, yy + 20.0 + 18.0 * k as f32, 14.0, Theme::text2());
        }
        ui.button(Rect::new(r.x + r.w - 120.0, yy - 14.0, 106.0, 28.0), crate::inhalt::t("menu.staff.reiter.1"), false)
    };
    let bewerber = c.stab_bewerber();
    let spalten: [(&str, &[u8]); 2] = [(crate::inhalt::t("ui.pages.trainer_und_co_trainer"), &[0, 1]), (crate::inhalt::t("ui.pages.aerzte_physios_jugendleiter"), &[2, 3, 4])];
    for (k, (titel, rollen)) in [(0usize, spalten[0]), (2, spalten[1])] {
        let r = karte(k, titel, ui);
        let mut yy = r.y + 64.0;
        for (i, s) in bewerber.iter().enumerate().filter(|(_, s)| rollen.contains(&s.rolle)) {
            if yy > r.y + r.h - 50.0 { break; }
            let bereich = if s.rolle == 0 { format!(" ({})", crate::personal::bereiche_t()[s.bereich as usize % 5]) } else { String::new() };
            if zeile(ui, r, yy, &s.name, &[crate::tf!("ui.pages.koennen", crate::personal::rollen_de_t()[s.rolle as usize % 5], bereich, format!("{:.0}", s.f))], s.gehalt) {
                match c.stab_einstellen(i) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.eingestellt"), true), Err(e) => v.toast(&e, false) }
                return;
            }
            yy += 62.0;
        }
    }
    let r = karte(1, crate::inhalt::t("scouting.scouts"), ui);
    let mut yy = r.y + 64.0;
    for (i, (name, pe)) in c.bewerber().into_iter().enumerate() {
        if yy > r.y + r.h - 50.0 { break; }
        let w: Vec<String> = crate::scouting::faehigkeiten_t().iter().zip(pe.f.iter()).map(|(n, x)| format!("{n} {x:.0}")).collect();
        if zeile(ui, r, yy, &name, &[w[..2].join(" · "), w[2..].join(" · ")], pe.gehalt) {
            match c.scout_einstellen(i) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.scout_eingestellt"), true), Err(e) => v.toast(&e, false) }
            return;
        }
        yy += 80.0;
    }
}

// ================================================================ Präsidium
fn praesidium(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    let y = Y0 - 40.0;
    let r = Rect::new(X0 + 4.0, y, 420.0, 330.0);
    ui.card_titled(r, crate::inhalt::t("sc.pv.vertrauen"), Theme::akzent());
    let conf = c.confidence();
    let col = if conf >= 65.0 { Theme::gut() } else if conf >= 35.0 { Theme::achtung() } else { Theme::problem() };
    let (cx, cy) = (r.x + r.w / 2.0, r.y + 180.0);
    for i in 0..60 {
        let a = std::f32::consts::PI * (1.0 + i as f32 / 59.0);
        let on = (i as f32 / 59.0) <= conf / 100.0;
        draw_circle(cx + a.cos() * 120.0, cy + a.sin() * 120.0, 6.0, if on { col } else { Theme::ueberlagerung() });
    }
    ui.title(&format!("{:.0}", conf), cx - 30.0, cy - 10.0, 52.0, col);
    ui.text_center(c.confidence_label(), cx, cy + 30.0, 20.0, Theme::text());
    if let Some(dl) = c.ultimatum { ui.text_center(&crate::tf!("ui.pages.ultimatum_noch_tage", dl.saturating_sub(c.abs())), cx, cy + 64.0, 18.0, Theme::problem()); }
    let r2 = Rect::new(X0 + 438.0, y, cbreite() - 434.0, 160.0);
    ui.card_titled(r2, crate::inhalt::t("zentrale.k.ziel"), Theme::gold());
    ui.title(&c.club().goal_kind, r2.x + 14.0, r2.y + 70.0, 34.0, Theme::text());
    let rank = c.world.rank_of(c.own);
    ui.text(&crate::tf!("ui.pages.mindestens_platz_aktuell_platz", c.club().goal_pos, rank), r2.x + 14.0, r2.y + 100.0, 19.0, if rank <= c.club().goal_pos { Theme::gut() } else { Theme::achtung() });
    let risiko = c.entlassungsrisiko();
    let rw = if risiko >= 0.6 { crate::id("hoch") } else if risiko >= 0.3 { crate::inhalt::t("bg.groesse.mittel") } else { crate::id("gering") };
    ui.text(&crate::tf!("ui.pages.entlassungsgefahr_vom_ueberschuss_gehen_", rw, format!("{:.0}", c.budget_anteil() * 100.0)), r2.x + 14.0, r2.y + 130.0, 16.0, Theme::text2());
    let r3 = Rect::new(X0 + 438.0, y + 172.0, cbreite() - 434.0, 158.0);
    ui.card_titled(r3, crate::inhalt::t("ui.pages.umfeld"), Theme::akzent());
    for (i, (n, val)) in [("Fans", c.fans), (crate::inhalt::t("story.groesse.medienbild"), c.media)].iter().enumerate() { // roh
        let yy = r3.y + 60.0 + i as f32 * 46.0;
        ui.text(n, r3.x + 14.0, yy, 18.0, Theme::text());
        ui.bar(r3.x + 150.0, yy - 10.0, r3.w - 240.0, 10.0, val / 100.0, if *val >= 55.0 { Theme::gut() } else if *val >= 35.0 { Theme::achtung() } else { Theme::problem() });
        ui.text_right(&format!("{:.0}", val), r3.x + r3.w - 14.0, yy, 18.0, Theme::text());
    }
    let r4 = Rect::new(X0 + 4.0, y + 344.0, cbreite(), (hoehe() - y - 360.0).max(120.0));
    ui.card_titled(r4, crate::inhalt::t("ui.pages.so_denkt_das_praesidium"), Theme::akzent());
    let gr: Vec<_> = c.mensch.gruende.iter().rev().filter(|g| g.bezug == "Präsidium").take(((r4.h - 70.0) / 26.0).max(1.0) as usize).cloned().collect(); // roh
    if gr.is_empty() {
        ui.paragraph(crate::inhalt::t("ui.pages.nach_jedem_spiel_schaut_das"), r4.x + 14.0, r4.y + 56.0, 18.0, r4.w - 28.0, Theme::text2());
    }
    for (k, g) in gr.iter().enumerate() {
        let yy = r4.y + 60.0 + k as f32 * 26.0;
        ui.text(&date::short(g.abs / date::SEASON_DAYS, g.abs % date::SEASON_DAYS), r4.x + 14.0, yy, 16.0, Theme::gedaempft());
        ui.text(&g.grund, r4.x + 110.0, yy, 17.0, Theme::text());
        ui.text_right(&format!("{:+.0}", g.wert), r4.x + r4.w - 14.0, yy, 17.0, if g.wert >= 0.0 { Theme::gut() } else { Theme::problem() });
    }
}

// ================================================================ Finanzen
fn finanzen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // sponsors, stadium and fans are main tabs of their own now (the wrapper pages choose the view)
    if v.page == Page::Finanzen { tabs(ui, v, &[crate::inhalt::t("as.reiter.ueberblick"), crate::inhalt::t("ui.pages.tickets_und_merchandising")]); }
    c.wirtschaft_an();
    let w4 = (cbreite() - 36.0) / 4.0;
    let y = Y0;
    let rev = c.umsatz_prognose();
    let wages = c.world.wage_bill(c.own);
    kpi(ui, Rect::new(X0 + 4.0, y, w4, 96.0), crate::inhalt::t("ui.wort.gross_kontostand"), &money(c.club().balance), crate::inhalt::t("ui.wort.aktuell"), if c.club().balance >= 0.0 { Theme::gut() } else { Theme::problem() });
    kpi(ui, Rect::new(X0 + 16.0 + w4, y, w4, 96.0), crate::inhalt::t("ui.wort.gross_transferbudget"), &money(c.transfer_budget), crate::inhalt::t("ui.pages.zum_ausgeben"), Theme::gold());
    kpi(ui, Rect::new(X0 + 28.0 + 2.0 * w4, y, w4, 96.0), crate::inhalt::t("ui.wort.gross_gehaltssumme"), &money(wages), &crate::tf!("ui.pages.des_umsatzes", format!("{:.0}", wages / rev * 100.0)), Theme::akzent());
    kpi(ui, Rect::new(X0 + 40.0 + 3.0 * w4, y, w4, 96.0), crate::inhalt::t("ui.wort.gross_umsatz"), &money(rev), crate::inhalt::t("welt.pro_jahr"), Theme::akzent());
    let y = y + 110.0;
    if v.tab == 0 {
        let r = Rect::new(X0 + 4.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
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
        let r2 = Rect::new(X0 + cbreite() * 0.5 + 10.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
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
        let r = Rect::new(X0 + 4.0, y, cbreite() * 0.5 - 6.0, 300.0);
        ui.card_titled(r, crate::inhalt::t("vw.preis.titel"), Theme::gold());
        if let Some(s) = ui.segment(Rect::new(r.x + 14.0, r.y + 44.0, r.w - 28.0, 34.0), &[crate::inhalt::t("as.prio.niedrig"), crate::inhalt::t("programm.intensitaet.normal"), crate::inhalt::t("belastung.hoch")], c.ticket as usize) { c.ticket = s as u8; }
        let att = c.attendance();
        let lines = [
            crate::tf!("ui.pages.preis_je_karte_schnitt", money(c.ticket_price() as f64)),
            crate::tf!("ui.pages.stadion_plaetze", format!("{:.0}", c.capacity())),
            crate::tf!("ui.pages.erwartete_auslastung_zuschauer", format!("{:.0}", att * 100.0), format!("{:.0}", att * c.capacity())),
            crate::tf!("ui.pages.einnahmen_je_heimspiel", money(c.gate_per_match())),
            if c.ticket == 2 { crate::inhalt::t("ui.pages.hohe_preise_aergern_die_fans") } else if c.ticket == 0 { crate::inhalt::t("ui.pages.niedrige_preise_freuen_die_fans") } else { crate::inhalt::t("ui.pages.normale_preise_die_fans_sind") }.to_string(),
        ];
        for (i, l) in lines.iter().enumerate() { ui.text(l, r.x + 14.0, r.y + 112.0 + i as f32 * 32.0, 18.0, if i == 4 { Theme::text2() } else { Theme::text() }); }
        let r2 = Rect::new(X0 + cbreite() * 0.5 + 10.0, y, cbreite() * 0.5 - 6.0, 300.0);
        ui.card_titled(r2, crate::inhalt::t("ui.pages.merchandising"), Theme::gold());
        if let Some(s) = ui.segment(Rect::new(r2.x + 14.0, r2.y + 44.0, r2.w - 28.0, 34.0), &[crate::inhalt::t("vw.preis.stufe.0"), crate::inhalt::t("programm.intensitaet.normal"), crate::inhalt::t("vw.wi.preis_sitz.2")], c.merch as usize) { c.merch = s as u8; }
        ui.text(&crate::tf!("ui.pages.einnahmen_pro_monat", money(c.merch_month())), r2.x + 14.0, r2.y + 112.0, 18.0, Theme::text());
        ui.text(&crate::tf!("ui.pages.fanstimmung_wirkt_mit_je_besser", format!("{:.0}", c.fans)), r2.x + 14.0, r2.y + 144.0, 18.0, Theme::text2());
        ui.text(&crate::tf!("ui.pages.fans_im_umfeld_fanshop_stufe", format!("{:.0}", c.fanbasis()), c.wi.stufen[1]), r2.x + 14.0, r2.y + 176.0, 18.0, Theme::text2());
    } else if v.tab == 2 {
        let r = Rect::new(X0 + 4.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r, crate::inhalt::t("ui.pages.laufende_vertraege"), Theme::gold());
        let mut yy = r.y + 60.0;
        if c.wi.vertraege.is_empty() { ui.text(crate::inhalt::t("ui.pages.kein_sponsor_unter_vertrag"), r.x + 14.0, yy, 17.0, Theme::text2()); }
        for sp in c.wi.vertraege.clone() {
            ui.text_semi(&format!("{}: {}", crate::wirtschaft::sponsor_de_t()[sp.art as usize], sp.name), r.x + 14.0, yy, 18.0, Theme::text());
            ui.text(&crate::tf!("ui.pages.im_jahr_bis_bonus_bei", money(sp.jahr), date::season_label(sp.bis), money(sp.bonus)), r.x + 14.0, yy + 24.0, 15.0, Theme::text2());
            yy += 62.0;
        }
        let r2 = Rect::new(X0 + cbreite() * 0.5 + 10.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r2, crate::inhalt::t("as.reiter.angebote"), Theme::akzent());
        let mut yy = r2.y + 60.0;
        if c.wi.angebote.is_empty() { ui.paragraph(crate::inhalt::t("ui.pages.neue_angebote_kommen_am_monatsende"), r2.x + 14.0, yy, 16.0, r2.w - 28.0, Theme::text2()); }
        let abs = c.abs();
        for (i, sp) in c.wi.angebote.clone().into_iter().enumerate() {
            if yy > r2.y + r2.h - 60.0 { break; }
            ui.text_semi(&format!("{}: {}", crate::wirtschaft::sponsor_de_t()[sp.art as usize], sp.name), r2.x + 14.0, yy, 18.0, Theme::text());
            ui.text(&crate::tf!("ui.pages.im_jahr_bis_noch_tage", money(sp.jahr), date::season_label(sp.bis), sp.frist.saturating_sub(abs)), r2.x + 14.0, yy + 24.0, 15.0, Theme::text2());
            if ui.button(Rect::new(r2.x + r2.w - 150.0, yy - 14.0, 136.0, 32.0), crate::inhalt::t("lb.aktion.annehmen"), false) { match c.sponsor_annehmen(i) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.vertrag_unterschrieben"), true), Err(e) => v.toast(&e, false) } break; }
            yy += 62.0;
        }
    } else if v.tab == 4 {
        let r = Rect::new(X0 + 4.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
        ui.card_titled(r, crate::inhalt::t("gs.thema.fans_stimmung"), Theme::akzent());
        let teile = [(crate::inhalt::t("ui.pages.grundstimmung"), c.fans), (crate::inhalt::t("bereich.transfers"), c.me.fan_transfer), (crate::inhalt::t("ui.pages.preise"), c.me.fan_wirtschaft), (crate::inhalt::t("ui.medien.fanaktionen"), c.me.fan_aktion)];
        let mut yy = r.y + 64.0;
        ui.title(&format!("{:.0}", c.fans_eff()), r.x + 14.0, yy + 10.0, 40.0, if c.fans_eff() >= 55.0 { Theme::gut() } else if c.fans_eff() >= 40.0 { Theme::achtung() } else { Theme::problem() });
        ui.text(c.protest_wort(c.protest_stufe()), r.x + 110.0, yy + 4.0, 18.0, Theme::text());
        yy += 52.0;
        for (n, w) in teile { ui.text(n, r.x + 14.0, yy, 17.0, Theme::text2()); ui.text_right(&format!("{:+.1}", w), r.x + r.w - 14.0, yy, 17.0, if w >= 0.0 { Theme::gut() } else { Theme::problem() }); yy += 30.0; }
        ui.paragraph(crate::inhalt::t("ui.pages.die_grundstimmung_folgt_serien_tabelle"), r.x + 14.0, yy + 10.0, 15.0, r.w - 28.0, Theme::gedaempft());
        let r2 = Rect::new(X0 + cbreite() * 0.5 + 10.0, y, cbreite() * 0.5 - 6.0, hoehe() - y - 16.0);
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
        let r = Rect::new(X0 + 4.0, y, cbreite(), hoehe() - y - 16.0);
        ui.card_titled(r, crate::inhalt::t("menu.stadion_und_anlagen.titel"), Theme::akzent());
        let abs = c.abs();
        for a in 0..5 {
            let yy = r.y + 70.0 + a as f32 * 74.0;
            ui.text_semi(crate::wirtschaft::anlagen_de_t()[a], r.x + 14.0, yy, 19.0, Theme::text());
            for k in 0..5 { draw_circle(r.x + 300.0 + k as f32 * 22.0, yy - 6.0, 7.0, if k < c.wi.stufen[a] as usize { Theme::gold() } else { Theme::ueberlagerung() }); }
            let wirkung = match a {
                0 => crate::tf!("ui.pages.plaetze", format!("{:.0}", c.kapazitaet())),
                1 => crate::tf!("ui.pages.merchandising_im_monat", money(c.merch_month())),
                2 => crate::tf!("ui.pages.entwicklung", format!("{:.0}", (c.training_faktor() - 1.0) * 100.0)),
                3 => crate::tf!("ui.pages.jahrgaenge_staerke", format!("{:.0}", c.jugend_bonus())),
                _ => crate::tf!("ui.pages.verletzungsrisiko", format!("{:.0}", (1.0 - c.medizin_faktor()) * 100.0)),
            };
            ui.text(&wirkung, r.x + 14.0, yy + 24.0, 15.0, Theme::text2());
            if let Some(b) = c.wi.bau.iter().find(|b| b.0 == a) {
                ui.text_right(&crate::tf!("ui.pages.im_bau_stufe_fertig_in", b.1, b.2.saturating_sub(abs)), r.x + r.w - 14.0, yy, 16.0, Theme::achtung());
            } else if c.wi.stufen[a] < 5 {
                ui.text_right(&crate::tf!("ui.pages.naechste_stufe", money(c.ausbau_kosten(a))), r.x + r.w - 170.0, yy, 16.0, Theme::text2());
                if ui.button(Rect::new(r.x + r.w - 150.0, yy - 20.0, 136.0, 32.0), crate::inhalt::t("vw.aktion.ausbauen"), false) { match c.ausbauen(a) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.ausbau_begonnen"), true), Err(e) => v.toast(&e, false) } }
            }
        }
        ui.text(&crate::tf!("ui.pages.betrieb_und_unterhalt_im_monat", money(c.betrieb_monat()), money(c.tv_monat())), r.x + 14.0, r.y + r.h - 20.0, 15.0, Theme::gedaempft());
    }
}

// ================================================================ Trainerprofil
fn trainer(ui: &mut Ui, v: &mut View, c: &mut Career) {
    ui.title(&c.coach.to_uppercase(), X0 + 8.0, Y0 - 14.0, 30.0, Theme::text());
    let y = Y0;
    let w3 = (cbreite() - 24.0) / 3.0;
    kpi(ui, Rect::new(X0 + 4.0, y, w3, 96.0), crate::inhalt::t("ui.wort.gross_ruf"), &format!("{:.0}", c.mgr_rep), if c.mgr_rep >= 70.0 { crate::inhalt::t("ui.pages.grosser_name") } else if c.mgr_rep >= 50.0 { crate::inhalt::t("ui.pages.bekannter_name") } else if c.mgr_rep >= 30.0 { crate::inhalt::t("ui.pages.solider_ruf") } else { crate::inhalt::t("ui.pages.kaum_bekannt") }, Theme::akzent());
    kpi(ui, Rect::new(X0 + 16.0 + w3, y, w3, 96.0), crate::inhalt::t("ui.wort.gross_gehalt"), &money(c.mgr_wage), crate::inhalt::t("welt.pro_jahr"), Theme::gold());
    kpi(ui, Rect::new(X0 + 28.0 + 2.0 * w3, y, w3, 96.0), crate::inhalt::t("ui.pages.eigenes_konto"), &money(c.mgr_money), crate::inhalt::t("ui.wort.angespart"), Theme::gut());
    let r = Rect::new(X0 + 4.0, y + 110.0, cbreite() * 0.5 - 6.0, hoehe() - y - 126.0);
    ui.card_titled(r, crate::inhalt::t("menu.trainerprofil.reiter.4"), Theme::akzent());
    let mut yy = r.y + 56.0;
    // coaching licence (laufbahn.toml [lizenz]): level, the clubs it is good for, what is missing for the next one
    ui.text_semi(&format!("{}: {}", crate::inhalt::t("lb.lizenz.titel"), crate::inhalt::t(&format!("lb.lizenz.{}", c.lizenz()))), r.x + 14.0, yy, 18.0, Theme::akzent());
    ui.text(&crate::tf!("lb.lizenz.reicht", format!("{:.0}", c.lizenz_grenze() * 100.0)), r.x + 14.0, yy + 24.0, 15.0, Theme::text2());
    let naechste = match c.lizenz_naechste() {
        Some((n, ruf, monate)) if ruf > 0.0 || monate > 0.0 => format!("{} {}: {}", crate::inhalt::t("lb.lizenz.naechste"), crate::inhalt::t(&format!("lb.lizenz.{n}")), crate::tf!("lb.lizenz.fehlt", ruf, monate)),
        Some(_) => crate::inhalt::t("lb.lizenz.bereit").to_string(),
        None => crate::inhalt::t("lb.lizenz.hoechste").to_string(),
    };
    ui.text(&naechste, r.x + 14.0, yy + 46.0, 15.0, Theme::text2());
    yy += 84.0;
    if c.history.is_empty() { ui.text(&crate::tf!("ui.pages.seit_bei", date::season_label(0), c.club().name), r.x + 14.0, yy, 18.0, Theme::text2()); }
    for h in c.history.iter().rev().take(14) { ui.text(h, r.x + 14.0, yy, 18.0, Theme::text()); yy += 28.0; }
    let r2 = Rect::new(X0 + cbreite() * 0.5 + 10.0, y + 110.0, cbreite() * 0.5 - 6.0, hoehe() - y - 126.0);
    ui.card_titled(r2, crate::inhalt::t("menu.trainerprofil.reiter.5"), Theme::gold());
    let offers: Vec<usize> = (0..c.inbox.len()).filter(|&i| matches!(c.inbox[i].action, Action::JobOffer { .. }) && !c.inbox[i].done).collect();
    if offers.is_empty() { ui.text(crate::inhalt::t("ui.pages.im_moment_liegt_kein_angebot"), r2.x + 14.0, r2.y + 56.0, 17.0, Theme::text2()); }
    for (k, &i) in offers.iter().take(5).enumerate() {
        let yy = r2.y + 50.0 + k as f32 * 60.0;
        let t = ui.fit(&c.inbox[i].title, 18.0, r2.w - 180.0);
        ui.text_semi(&t, r2.x + 14.0, yy + 20.0, 18.0, Theme::text());
        if ui.button(Rect::new(r2.x + r2.w - 150.0, yy, 136.0, 32.0), crate::inhalt::t("es.eil.ansehen"), false) { v.sel_msg = Some(i); v.go(Page::Posteingang); }
    }
    // vacant posts: apply in four steps (laufbahn.toml [bewerbung])
    let vy = r2.y + r2.h * 0.5;
    ui.text_semi(crate::inhalt::t("ui.pages.freie_posten"), r2.x + 14.0, vy, 14.0, Theme::text2());
    let vak = c.vakanzen();
    if vak.is_empty() { ui.text(crate::inhalt::t("ui.pages.gerade_ist_kein_passender_posten"), r2.x + 14.0, vy + 28.0, 16.0, Theme::text2()); }
    for (k, &cl) in vak.iter().enumerate() {
        let yy = vy + 20.0 + k as f32 * 46.0;
        ui.text(&crate::tf!("ui.pages.praesidium", c.world.clubs[cl].name, c.persona_von(cl)), r2.x + 14.0, yy + 20.0, 16.0, Theme::text());
        if ui.button(Rect::new(r2.x + r2.w - 150.0, yy, 136.0, 32.0), crate::inhalt::t("lb.antwort.bewerben"), false) { match c.bewerben(cl) { Ok(()) => { v.toast(crate::inhalt::t("ui.pages.bewerbung_laeuft_fragen_im_posteingang"), true); } Err(e) => v.toast(&e, false) } }
    }
}

// ================================================================ Transfermarkt
fn transfermarkt(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("menu.transfermarkt.reiter.0"), crate::inhalt::t("gl.chip.transferliste"), crate::inhalt::t("gl.chip.vertragslos")]);
    let w4 = (cbreite() - 36.0) / 4.0;
    let y = Y0;
    kpi(ui, Rect::new(X0 + 4.0, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_transferbudget"), &money(c.transfer_budget), crate::inhalt::t("gl.bk.transfer.unter"), Theme::gold());
    kpi(ui, Rect::new(X0 + 16.0 + w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_kader"), &format!("{} / 40", c.squad().len()), crate::inhalt::t("gl.bk.kader.unter"), Theme::akzent());
    kpi(ui, Rect::new(X0 + 28.0 + 2.0 * w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_gehaltssumme"), &money(c.world.wage_bill(c.own)), crate::inhalt::t("gl.bk.gehaelter.unter"), Theme::gut());
    kpi(ui, Rect::new(X0 + 40.0 + 3.0 * w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_kontostand"), &money(c.club().balance), crate::inhalt::t("bereich.verein"), Theme::gold());
    let y = y + 82.0;
    let wtxt = match c.in_window() { Some((_, b)) => crate::tf!("tv.fenster.frist", b - c.world.day, date::short(c.world.season, b)), None => { let next = WINDOWS.iter().find(|(a, _)| *a > c.world.day).map(|(a, _)| date::short(c.world.season, *a)).unwrap_or_else(|| crate::inhalt::t("ui.pages.sommer").into()); crate::tf!("ui.pages.transferfenster_geschlossen_es_oeffnet_a", next) } };
    ui.card(Rect::new(X0 + 4.0, y, cbreite(), 38.0));
    ui.title(&wtxt, X0 + 18.0, y + 27.0, 20.0, Theme::text());
    let y = y + 46.0;
    let si = Rect::new(X0 + 4.0, y, 330.0, 36.0);
    if ui.hit(si) { v.market_focus = true; } else if ui.clicked { v.market_focus = false; }
    let focus = v.market_focus;
    ui.input(si, &mut v.market_text, focus, crate::inhalt::t("sup.suche.platz"));
    if let Some(s) = ui.segment(Rect::new(X0 + 344.0, y, 300.0, 36.0), &[crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("ui.wort.gross_tw"), crate::inhalt::t("ui.wort.gross_abw"), crate::inhalt::t("ui.wort.gross_mit"), crate::inhalt::t("ui.wort.gross_ang")], v.market_pos) { v.market_pos = s; v.scroll = 0.0; v.scrolls.clear(); }
    if let Some(s) = ui.segment(Rect::new(X0 + 654.0, y, 220.0, 36.0), &[crate::inhalt::t("ui.pages.alle_ligen"), crate::inhalt::t("ui.pages.eigene_liga")], v.market_scope) { v.market_scope = s; v.scroll = 0.0; v.scrolls.clear(); }
    ui.text(&crate::tf!("ui.pages.alter_bis", v.market_age), X0 + 890.0, y + 14.0, 14.0, Theme::text2());
    if let Some(a) = ui.slider(Rect::new(X0 + 890.0, y + 20.0, 120.0, 14.0), v.market_age, 16, 40) { v.market_age = a; v.scroll = 0.0; v.scrolls.clear(); }
    ui.text(&crate::tf!("ui.pages.wert_bis_mio", money(v.market_price as f64 * 1e6)), X0 + 1030.0, y + 14.0, 14.0, Theme::text2());
    if let Some(a) = ui.slider(Rect::new(X0 + 1030.0, y + 20.0, 120.0, 14.0), v.market_price, 1, 200) { v.market_price = a; v.scroll = 0.0; v.scrolls.clear(); }
    let key = format!("{}|{}|{}|{}|{}|{}|{}|{}", v.tab, v.market_pos, v.market_scope, v.market_age, v.market_price, v.market_text.to_lowercase(), c.abs(), c.world.players.len());
    if key != v.market_key {
        v.market_key = key;
        let q = v.market_text.to_lowercase();
        let ol = c.league();
        let mut list: Vec<usize> = (0..c.world.players.len()).filter(|&i| {
            let p = &c.world.players[i];
            if p.team == c.own as i32 || p.team == -2 || p.youth > 0 { return false; }
            if v.tab == 1 && !p.listed { return false; }
            if v.tab == 2 && p.team != -1 { return false; }
            if v.tab != 2 && p.team < 0 { return false; }
            let part = part_of(&p.pos);
            if v.market_pos > 0 && part + 1 != v.market_pos { return false; }
            if v.market_scope == 1 && (p.team < 0 || c.world.clubs[p.team as usize].league != ol) { return false; }
            if p.age as i32 > v.market_age || p.value > v.market_price as f64 * 1e6 { return false; }
            if !q.is_empty() && !p.name.to_lowercase().contains(&q) && !(p.team >= 0 && c.world.clubs[p.team as usize].name.to_lowercase().contains(&q)) { return false; }
            true
        }).collect();
        list.sort_by(|a, b| { let (x, y) = (c.ovr_range(*a), c.ovr_range(*b)); (y.0 + y.1).partial_cmp(&(x.0 + x.1)).unwrap() });
        list.truncate(2000);
        v.market_cache = list;
    }
    let list = v.market_cache.clone();
    let y = y + 46.0;
    let lw = cbreite() - 300.0;
    let area = Rect::new(X0 + 4.0, y, lw, hoehe() - y - 16.0);
    ui.card(area);
    player_row_head(ui, area.x, area.y + 22.0, &[(crate::inhalt::t("ui.wort.gross_name"), 16.0), (crate::inhalt::t("ui.wort.gross_verein"), 250.0), (crate::inhalt::t("ui.wort.gross_alter"), 470.0), (crate::inhalt::t("ui.wort.gross_pos"), 530.0), (crate::inhalt::t("ui.pages.staerke"), 590.0), (crate::inhalt::t("ui.wort.gross_marktwert"), 680.0), (crate::inhalt::t("ui.wort.gross_preis_leist"), 800.0)]);
    let vis = ((area.h - 40.0) / 36.0) as usize;
    let start = scroll_rows(ui, v, area, list.len(), vis);
    ui.text_right(&crate::tf!("zentrale.k.spieler.viele", list.len()), X0 + cbreite() - 14.0, Y0 - 20.0, 16.0, Theme::text2());
    for (k, &p) in list.iter().skip(start).take(vis).enumerate() {
        let pl = &c.world.players[p];
        let r = Rect::new(area.x + 4.0, area.y + 34.0 + k as f32 * 36.0, area.w - 8.0, 34.0);
        let sel = v.sel_player == Some(p);
        if sel { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::akzent(), 0.14)); rrect_line(r.x, r.y, r.w, r.h, 4.0, Theme::akzent()); } else if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::erhoehung()); }
        draw_rectangle(r.x, r.y + 4.0, 3.0, r.h - 8.0, Theme::teil(&pl.pos));
        let cy = r.y + 23.0;
        ui.player_photo(r.x + 22.0, r.y + r.h / 2.0, 12.0, pl.id, &pl.name);
        let nm = ui.fit(&pl.name, 17.0, 196.0);
        ui.text_semi(&nm, r.x + 40.0, cy, 17.0, Theme::text());
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
        let cl = ui.fit(&club, 16.0, 210.0);
        ui.text(&cl, r.x + 248.0, cy, 16.0, Theme::text2());
        ui.text(&format!("{}", pl.age), r.x + 476.0, cy, 16.0, Theme::text());
        ui.pos_tag(r.x + 526.0, cy - 5.0, pos_de(&pl.pos));
        strength_cell(ui, c, p, r.x + 612.0, cy - 6.0);
        ui.text(&money(pl.value), r.x + 680.0, cy, 16.0, Theme::text());
        let mb = c.moneyball(p);
        ui.text(&format!("{:+.0}", mb), r.x + 820.0, cy, 16.0, if mb >= 3.0 { Theme::gut() } else if mb <= -3.0 { Theme::problem() } else { Theme::text2() });
        if ui.hit(r) { v.sel_player = Some(p); }
    }
    // detail panel
    let dr = Rect::new(X0 + lw + 16.0, y - 0.0, 288.0 - 4.0, hoehe() - y - 16.0);
    ui.card(dr);
    let Some(p) = v.sel_player.filter(|&p| p < c.world.players.len()) else { ui.paragraph(crate::inhalt::t("ui.pages.waehle_einen_spieler_aus_der"), dr.x + 14.0, dr.y + 30.0, 16.0, dr.w - 28.0, Theme::text2()); return; };
    let pl = c.world.players[p].clone();
    ui.player_photo(dr.x + dr.w - 40.0, dr.y + 34.0, 24.0, pl.id, &pl.name);
    let nm = ui.fit(&pl.name, 24.0, dr.w - 84.0);
    ui.title(&nm, dr.x + 14.0, dr.y + 34.0, 24.0, Theme::text());
    ui.pos_tag(dr.x + 14.0, dr.y + 54.0, pos_de(&pl.pos));
    ui.text(&crate::tf!("zentrale.k.jahre", pl.age), dr.x + 56.0, dr.y + 59.0, 16.0, Theme::text2());
    let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
    let cl = ui.fit(&club, 16.0, dr.w - 28.0);
    ui.text(&cl, dr.x + 14.0, dr.y + 82.0, 16.0, Theme::text2());
    let (lo, hi) = c.ovr_range(p);
    let (pa, pb) = c.pot_range(p);
    ui.text_semi(crate::inhalt::t("ui.pages.staerke"), dr.x + 14.0, dr.y + 112.0, 13.0, Theme::gedaempft());
    ui.title(&if hi - lo <= 1.0 { format!("{:.0}", (lo + hi) / 2.0) } else { format!("{lo}–{hi}") }, dr.x + 14.0, dr.y + 140.0, 26.0, Theme::skala((lo + hi) / 2.0));
    ui.text_semi(crate::inhalt::t("ui.wort.gross_potenzial"), dr.x + 150.0, dr.y + 112.0, 13.0, Theme::gedaempft());
    ui.title(&format!("{pa}–{pb}"), dr.x + 150.0, dr.y + 140.0, 26.0, Theme::gold());
    ui.text(&crate::tf!("scouting.wissen_prozent", format!("{:.0}", c.know_of(p) * 100.0)), dr.x + 14.0, dr.y + 162.0, 14.0, Theme::text2());
    ui.bar(dr.x + 110.0, dr.y + 155.0, dr.w - 124.0, 6.0, c.know_of(p), Theme::akzent());
    ui.text(&crate::tf!("ui.pages.wert_gehalt", money(pl.value), money(pl.wage)), dr.x + 14.0, dr.y + 186.0, 15.0, Theme::text());
    ui.text(&crate::tf!("ui.pages.vertrag_bis_preis", date::START_YEAR as u32 + pl.contract_end + 1, if pl.team >= 0 { money(c.ask_price(p)) } else { crate::inhalt::t("iq.wechsel.frei").into() }), dr.x + 14.0, dr.y + 206.0, 15.0, Theme::text());
    // Dafür / Dagegen
    let best_own = c.squad().iter().filter(|&&q| part_of(&c.world.players[q].pos) == part_of(&pl.pos)).map(|&q| c.world.players[q].ovr).fold(0.0f32, f32::max);
    let mid = (lo + hi) / 2.0;
    let pro = if mid > best_own { crate::tf!("ui.pages.verstaerkt_die_position_wertung_gegen", format!("{:.0}", mid), format!("{:.0}", best_own)) } else if pl.age <= 21 && pb > best_own { crate::inhalt::t("ui.pages.junges_talent_mit_luft_nach").to_string() } else { crate::tf!("ui.pages.passt_ins", ideas_t()[c.tactic.idea()], fit_text(c.system_fit(p))) };
    let mut con = vec![];
    if c.ask_price(p) > c.transfer_budget { con.push(crate::inhalt::t("ui.pages.sprengt_das_transferbudget").to_string()); }
    if c.player_rep(p) - c.club().rep > 25.0 { con.push(crate::inhalt::t("ui.pages.haelt_deinen_verein_fuer_zu").into()); }
    if pl.age >= 31 { con.push(crate::inhalt::t("ui.pages.schon_ueber_30_baut_bald").into()); }
    if c.know_of(p) < 0.5 { con.push(crate::inhalt::t("ui.pages.noch_wenig_bekannt_scout_schicken").into()); }
    let con = if con.is_empty() { crate::inhalt::t("tf.verf.keine_bedenken").to_string() } else { con.join(" ") };
    // the buttons hang from the card's bottom; a short card (lower windows) shows „Dafür“ and „Dagegen“ in two lines each
    let unten = (dr.h - 92.0).min(382.0);
    if unten >= 380.0 {
        let r1 = Rect::new(dr.x + 10.0, dr.y + 220.0, dr.w - 20.0, 74.0);
        rrect(r1.x, r1.y, r1.w, r1.h, 6.0, alpha(Theme::gut(), 0.1)); rrect_line(r1.x, r1.y, r1.w, r1.h, 6.0, alpha(Theme::gut(), 0.5));
        ui.text_semi(crate::inhalt::t("ui.pages.dafuer"), r1.x + 10.0, r1.y + 18.0, 13.0, Theme::gut());
        ui.paragraph(&pro, r1.x + 10.0, r1.y + 38.0, 15.0, r1.w - 20.0, Theme::text());
        let r2 = Rect::new(dr.x + 10.0, dr.y + 300.0, dr.w - 20.0, 74.0);
        rrect(r2.x, r2.y, r2.w, r2.h, 6.0, alpha(Theme::problem(), 0.1)); rrect_line(r2.x, r2.y, r2.w, r2.h, 6.0, alpha(Theme::problem(), 0.5));
        ui.text_semi(crate::inhalt::t("ui.wort.gross_dagegen"), r2.x + 10.0, r2.y + 18.0, 13.0, Theme::problem());
        ui.paragraph(&con, r2.x + 10.0, r2.y + 38.0, 15.0, r2.w - 20.0, Theme::text());
    } else {
        // one line each, the whole text when the mouse is over it (text_passend)
        ui.text_passend(&crate::tf!("ui.pages.dafuer_zeile", &pro), dr.x + 14.0, dr.y + (unten - 28.0).max(226.0), 13.0, dr.w - 28.0, 1, Theme::gut());
        ui.text_passend(&crate::tf!("ui.pages.dagegen_zeile", &con), dr.x + 14.0, dr.y + (unten - 10.0).max(244.0), 13.0, dr.w - 28.0, 1, Theme::problem());
    }
    if ui.button_gold(Rect::new(dr.x + 10.0, dr.y + unten, dr.w - 20.0, 42.0), crate::inhalt::t("ui.wort.gross_verhandeln")) {
        match c.open_neg(p) { Ok(i) => { v.modal = Modal::Neg(i); ui.play(crate::id("angebot")); } Err(e) => { v.toast(&e, false); ui.play(crate::id("fehler")); } }
    }
    let watched = c.scouts.iter().any(|s| s.watch == Some(p));
    if ui.button(Rect::new(dr.x + 10.0, dr.y + unten + 48.0, (dr.w - 28.0) / 2.0, 32.0), if watched { crate::inhalt::t("pille.beobachtet") } else { crate::inhalt::t("aktion.beobachten") }, false) && !watched {
        if let Some(s) = c.scouts.iter_mut().find(|s| s.watch.is_none()) { s.watch = Some(p); let n = s.name.clone(); v.toast(&crate::tf!("ui.pages.beobachtet_ihn_ab_jetzt", n), true); } else { v.toast(crate::inhalt::t("ui.pages.alle_scouts_haben_schon_einen"), false); }
    }
    if ui.button(Rect::new(dr.x + 18.0 + (dr.w - 28.0) / 2.0, dr.y + unten + 48.0, (dr.w - 28.0) / 2.0, 32.0), crate::inhalt::t("aktion.profil"), false) { v.modal = Modal::Player(p); }
}

// ================================================================ Verhandlungen
fn verhandlungen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("chat.laufend"), crate::inhalt::t("chat.abschluss.titel")]);
    let list: Vec<usize> = (0..c.negs.len()).filter(|&i| (c.negs[i].stage < 2) == (v.tab == 0)).collect();
    let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
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
    let r = Rect::new(breite() / 2.0 - 330.0, 60.0, 660.0, 724.0);
    rrect(r.x, r.y, r.w, r.h, 10.0, Theme::ueberlagerung());
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
        } else if n.verkauf {
            let mut w = n.kl;
            ui.text(t("ui.klausel.weiterverkauf"), r.x + 24.0, ky + 20.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 180.0, ky, 260.0, 28.0), &["0 %", "10 %", "20 %", "30 %"], w.weiterverkauf as usize) { w.weiterverkauf = k as u8; }
            ui.text(t("ui.klausel.rueckkauf"), r.x + 24.0, ky + 56.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 180.0, ky + 36.0, 260.0, 28.0), &[t("ui.klausel.keine"), "1,3×", "1,6×"], w.rueckkauf as usize) { w.rueckkauf = k as u8; }
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
    if editable && ui.button_gold(Rect::new(r.x + 24.0, by, 280.0, 44.0), crate::inhalt::t("ui.pages.angebot_abschicken")) { c.submit_neg(i); ui.play(crate::id("bestaetigt")); v.toast(crate::inhalt::t("ui.pages.angebot_abgeschickt"), true); }
    if n.stage < 2 && ui.button(Rect::new(r.x + 316.0, by, 150.0, 44.0), crate::inhalt::t("fenster.pk.unterbrechen"), false) { c.negs[i].stage = 2; c.negs[i].state = NegState::Rejected; c.negs[i].text = crate::inhalt::t("ui.pages.du_hast_die_gespraeche_beendet").into(); }
    if ui.button(Rect::new(r.x + r.w - 160.0, by, 136.0, 44.0), crate::inhalt::t("aktion.schliessen"), false) || is_key_pressed(KeyCode::Escape) { v.modal = Modal::None; }
}

// ================================================================ Verträge
fn vertraege(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("reiter.uebersicht"), crate::inhalt::t("menu.vertraege.reiter.1")]);
    let mut sq = c.squad();
    if v.tab == 1 { sq.retain(|&p| c.world.players[p].contract_end <= c.world.season); }
    sq.sort_by_key(|&p| c.world.players[p].contract_end);
    let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    player_row_head(ui, area.x, area.y + 24.0, &[(crate::inhalt::t("ui.wort.gross_name"), 60.0), (crate::inhalt::t("ui.wort.gross_alter"), 330.0), (crate::inhalt::t("ui.pages.staerke"), 400.0), (crate::inhalt::t("ui.wort.gross_gehalt"), 480.0), (crate::inhalt::t("ui.pages.vertrag_bis_2"), 640.0), (crate::inhalt::t("ui.wort.gross_berater"), 780.0)]);
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

// ================================================================ Scouting
fn scouting(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("ui.pages.scouts_und_auftraege"), crate::inhalt::t("menu.scout_verwaltung.reiter.1"), crate::inhalt::t("as.reiter.berichte")]);
    if v.tab == 0 {
        let mut y = Y0;
        let abs = c.abs();
        let n = c.scouts.len();
        for s in 0..n {
            if s >= c.scouts.len() { break; }
            let r = Rect::new(X0 + 4.0, y, cbreite(), 150.0);
            ui.card(r);
            let sc = c.scouts[s].clone();
            ui.title(&sc.name, r.x + 16.0, r.y + 34.0, 24.0, Theme::text());
            if let Some(pe) = &sc.p {
                let fw: Vec<String> = (0..4).map(|k| format!("{} {}", crate::scouting::faehigkeiten_t()[k], if pe.f[k] >= 70.0 { crate::id("stark") } else if pe.f[k] >= 50.0 { crate::id("solide") } else { crate::id("schwach") })).collect();
                ui.text(&crate::tf!("ui.pages.fachgebiet", fw.join(", "), crate::scouting::gruppen_t()[pe.spezial as usize % 5], crate::scouting::typ_wort_t()[pe.typ as usize % 4]), r.x + 16.0, r.y + 58.0, 15.0, Theme::text2());
                let vt = if pe.vertrauen >= 60.0 { crate::inhalt::t("md.pool.tag.verlässlich") } else if pe.vertrauen >= 40.0 { crate::inhalt::t("zentrale.k.keins") } else { crate::inhalt::t("ui.pages.lag_oft_daneben") }; // roh: key
                ui.text_right(&crate::tf!("ui.pages.im_jahr_urteil", money(pe.gehalt), if pe.auftrag_bis > abs { crate::tf!("ui.pages.auftrag_noch_tage", pe.auftrag_bis - abs) } else { crate::inhalt::t("ui.pages.kein_auftrag").into() }, vt), r.x + r.w - 16.0, r.y + 34.0, 15.0, Theme::text2());
            } else {
                ui.text(&crate::tf!("ui.pages.spieler_gesichtet", sc.seen), r.x + 16.0, r.y + 58.0, 16.0, Theme::text2());
            }
            ui.text_semi(crate::inhalt::t("ui.wort.gross_gebiet"), r.x + 16.0, r.y + 92.0, 14.0, Theme::text2());
            let ln = sc.league.map(|l| c.world.leagues[l].name.clone()).unwrap_or(crate::id("weltweit").into());
            if ui.button(Rect::new(r.x + 16.0, r.y + 102.0, 36.0, 32.0), "<", false) { let n = c.world.leagues.len(); let mut l = sc.league.map(|l| (l + n - 1) % n).unwrap_or(0); for _ in 0..n { if c.gebiet_erlaubt(s, Some(l)) { break; } l = (l + n - 1) % n; } c.scouts[s].league = Some(l); }
            ui.text_center(&ui.fit(&ln, 17.0, 190.0), r.x + 150.0, r.y + 124.0, 17.0, Theme::text());
            if ui.button(Rect::new(r.x + 250.0, r.y + 102.0, 36.0, 32.0), ">", false) { let n = c.world.leagues.len(); let mut l = sc.league.map(|l| (l + 1) % n).unwrap_or(0); for _ in 0..n { if c.gebiet_erlaubt(s, Some(l)) { break; } l = (l + 1) % n; } c.scouts[s].league = Some(l); }
            if c.gebiet_erlaubt(s, None) && ui.button(Rect::new(r.x + 16.0, r.y + 66.0, 120.0, 22.0), crate::inhalt::t("ui.wort.weltweit"), sc.league.is_none()) { c.scouts[s].league = None; }
            ui.text_semi(crate::inhalt::t("ui.wort.gross_position"), r.x + 300.0, r.y + 92.0, 14.0, Theme::text2());
            if let Some(p) = ui.segment(Rect::new(r.x + 300.0, r.y + 102.0, 250.0, 32.0), &[crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("ui.wort.gross_tw"), crate::inhalt::t("ui.wort.gross_abw"), crate::inhalt::t("ui.wort.gross_mit"), crate::inhalt::t("ui.wort.gross_ang")], sc.pos as usize) { c.scouts[s].pos = p as u8; }
            ui.text_semi(crate::inhalt::t("ui.wort.gross_alter"), r.x + 562.0, r.y + 92.0, 14.0, Theme::text2());
            let am = sc.p.as_ref().map(|p| crate::scouting::ALTER_MAX.iter().position(|&a| a == p.alter_max).unwrap_or(0)).unwrap_or(0);
            if let Some(k) = ui.segment(Rect::new(r.x + 562.0, r.y + 102.0, 220.0, 32.0), &[crate::inhalt::t("ui.wort.alle"), crate::inhalt::t("ui.pages.bis_21"), crate::inhalt::t("ui.pages.bis_25"), crate::inhalt::t("ui.pages.bis_29")], am) { if let Some(p) = c.scouts[s].p.as_mut() { p.alter_max = crate::scouting::ALTER_MAX[k]; } }
            ui.text_semi(crate::inhalt::t("ui.wort.gross_auftrag"), r.x + 796.0, r.y + 92.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 796.0, r.y + 102.0, 250.0, 32.0), &[crate::inhalt::t("ui.pages.1_wo"), crate::inhalt::t("ui.pages.2_wo"), crate::inhalt::t("ui.pages.3_wo"), crate::inhalt::t("ui.pages.4_wo")], 9) { let am = sc.p.as_ref().map(|p| p.alter_max).unwrap_or(99); match c.auftrag(s, crate::scouting::DAUER[k], am) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.auftrag_erteilt"), true), Err(e) => v.toast(&e, false) } }
            let w = sc.watch.map(|p| crate::tf!("ui.pages.beobachtet", c.world.players[p].name, format!("{:.0}", c.world.players[p].know * 100.0))).unwrap_or(crate::inhalt::t("ui.pages.beobachtet_niemanden_im_transfermarkt_wa").into());
            ui.text(&ui.fit(&w, 15.0, 300.0), r.x + r.w - 330.0, r.y + 58.0, 15.0, Theme::text2());
            if ui.button(Rect::new(r.x + r.w - 140.0, r.y + 102.0, 124.0, 32.0), crate::inhalt::t("aka.entlassen.ja"), false) { c.scout_entlassen(s); v.toast(crate::inhalt::t("ui.pages.scout_entlassen"), true); break; }
            y += 160.0;
            if y > hoehe() - 170.0 { break; }
        }
        if y <= hoehe() - 120.0 {
            let r = Rect::new(X0 + 4.0, y, cbreite(), 110.0);
            ui.card_titled(r, crate::inhalt::t("ui.pages.so_arbeiten_scouts"), Theme::akzent());
            ui.paragraph(&crate::tf!("ui.pages.ein_scout_lernt_je_tag", c.scouts_max()), r.x + 14.0, r.y + 56.0, 16.0, r.w - 28.0, Theme::text2());
        }
    } else if v.tab == 1 {
        let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
        ui.card(area);
        ui.text(&crate::tf!("ui.pages.jeden_monat_stellen_sich_fuenf", c.scouts.len(), c.scouts_max()), area.x + 16.0, area.y + 30.0, 17.0, Theme::text2());
        for (k, (name, pe)) in c.bewerber().into_iter().enumerate() {
            let r = Rect::new(area.x + 8.0, area.y + 50.0 + k as f32 * 76.0, area.w - 16.0, 68.0);
            rrect(r.x, r.y, r.w, r.h, 6.0, Theme::erhoehung());
            ui.text_semi(&name, r.x + 14.0, r.y + 26.0, 19.0, Theme::text());
            let fw: Vec<String> = (0..4).map(|i| format!("{} {:.0}", crate::scouting::faehigkeiten_t()[i], pe.f[i])).collect();
            ui.text(&crate::tf!("ui.pages.fachgebiet", fw.join(" · "), crate::scouting::gruppen_t()[pe.spezial as usize % 5], crate::scouting::typ_wort_t()[pe.typ as usize % 4]), r.x + 14.0, r.y + 50.0, 15.0, Theme::text2());
            ui.text_right(&crate::tf!("uebersicht.gehalt_jahr", money(pe.gehalt)), r.x + r.w - 160.0, r.y + 38.0, 17.0, Theme::text());
            if ui.button(Rect::new(r.x + r.w - 140.0, r.y + 18.0, 124.0, 32.0), crate::inhalt::t("menu.staff.reiter.1"), false) { match c.scout_einstellen(k) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.scout_eingestellt"), true), Err(e) => v.toast(&e, false) } }
        }
    } else {
        let reps: Vec<usize> = (0..c.inbox.len()).filter(|&i| c.inbox[i].cat == 6).collect();
        let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
        ui.card(area);
        if reps.is_empty() { ui.text(crate::inhalt::t("ui.pages.noch_keine_berichte_die_scouts"), area.x + 20.0, area.y + 40.0, 18.0, Theme::text2()); }
        let vis = ((area.h - 10.0) / 70.0) as usize;
        let start = scroll_rows(ui, v, area, reps.len(), vis);
        for (k, &i) in reps.iter().skip(start).take(vis).enumerate() {
            let m = c.inbox[i].clone();
            let r = Rect::new(area.x + 8.0, area.y + 8.0 + k as f32 * 70.0, area.w - 16.0, 64.0);
            rrect(r.x, r.y, r.w, r.h, 6.0, if ui.hover(r) { Theme::ueberlagerung() } else { Theme::erhoehung() });
            ui.text_semi(&m.title, r.x + 14.0, r.y + 24.0, 18.0, if m.title.starts_with("Geheimtipp") { Theme::gold() } else { Theme::text() }); // roh
            let t = ui.fit(&m.body, 15.0, r.w - 28.0);
            ui.text(&t, r.x + 14.0, r.y + 48.0, 15.0, Theme::text2());
            if ui.hit(r) { if let Action::Report { pid } = m.action { v.modal = Modal::Player(pid); c.inbox[i].read = true; } }
        }
    }
}

fn akademie(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // with a second team there is no invented U23: the second team takes its place (farm.rs)
    let zweite = c.zweite_mannschaft().is_some();
    tabs(ui, v, &[if zweite { crate::inhalt::t("ui.game.zweite_mannschaft") } else { crate::id("U23") }, crate::inhalt::t("aka.gruppe.akademie")]);
    if v.tab == 0 && zweite { zweite_mannschaft_tab(ui, v, c); return; }
    let k = if v.tab == 0 { 1 } else { 0 };
    let g = if k == 1 { 19 } else { 17 };
    let sq = c.youth_squad(g);
    let lw = cbreite() * 0.58;
    let area = Rect::new(X0 + 4.0, Y0, lw, hoehe() - Y0 - 16.0);
    ui.card(area);
    ui.title(&crate::tf!("ui.pages.spieler", c.club().name, if k == 1 { crate::inhalt::t("ui.wort.gross_u19") } else { crate::inhalt::t("ui.wort.gross_u17") }, sq.len()), area.x + 14.0, area.y + 32.0, 22.0, Theme::text());
    let vis = ((area.h - 60.0) / 40.0) as usize;
    let start = scroll_rows(ui, v, area, sq.len(), vis);
    for (j, &p) in sq.iter().skip(start).take(vis).enumerate() {
        let pl = c.world.players[p].clone();
        let r = Rect::new(area.x + 6.0, area.y + 48.0 + j as f32 * 40.0, area.w - 12.0, 36.0);
        if j % 2 == 1 { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::erhoehung(), 0.6)); }
        ui.pos_tag(r.x + 6.0, r.y + 18.0, pos_de(&pl.pos));
        ui.player_photo(r.x + 56.0, r.y + 18.0, 13.0, pl.id, &pl.name);
        let nm = ui.fit(&pl.name, 17.0, 160.0);
        ui.text_semi(&nm, r.x + 76.0, r.y + 24.0, 17.0, Theme::text());
        ui.text(&crate::tf!("ui.wort.alter_kurz", pl.age), r.x + 240.0, r.y + 24.0, 16.0, Theme::text2());
        ui.pill(r.x + 310.0, r.y + 18.0, pl.ovr, 34.0);
        let (pa, pb) = c.pot_range(p);
        ui.text(&crate::tf!("ui.pages.pot", pa, pb), r.x + 336.0, r.y + 24.0, 15.0, Theme::gold());
        entwicklung_pfeil_mit(ui, r.x + 422.0, r.y + 18.0, c.ent_erwartung(p), false);
        if ui.button(Rect::new(r.x + r.w - 220.0, r.y + 3.0, 120.0, 30.0), crate::inhalt::t("ui.pages.zu_den_profis"), false) { c.promote(p); v.toast(&crate::tf!("ui.game.rueckt_zu_den_profis_auf", pl.name), true); }
        if ui.button(Rect::new(r.x + r.w - 92.0, r.y + 3.0, 86.0, 30.0), crate::inhalt::t("ui.pages.abgeben"), false) { c.release(p); }
    }
    let rt = Rect::new(X0 + lw + 18.0, Y0, cbreite() - lw - 14.0, hoehe() - Y0 - 16.0);
    ui.card_titled(rt, &c.youth[k].name.clone(), Theme::akzent());
    let yl = c.youth[k].clone();
    let mut y = rt.y + 50.0;
    for (i, r) in yl.rows.iter().enumerate().take(12) {
        if r.own { rrect(rt.x + 6.0, y - 18.0, rt.w - 12.0, 26.0, 4.0, alpha(Theme::akzent(), 0.15)); }
        ui.text(&format!("{}", i + 1), rt.x + 14.0, y, 16.0, Theme::text2());
        let nm = ui.fit(&r.name, 16.0, rt.w - 170.0);
        ui.text(&nm, rt.x + 40.0, y, 16.0, Theme::text());
        ui.text_right(&format!("{}  {:+}  {}", r.p, r.gf - r.ga, r.pts), rt.x + rt.w - 14.0, y, 16.0, Theme::text());
        y += 28.0;
    }
    ui.text_semi(crate::inhalt::t("spielbericht.letztes"), rt.x + 14.0, y + 20.0, 15.0, Theme::text2());
    ui.paragraph(if yl.last.is_empty() { crate::inhalt::t("ui.pages.noch_nicht_gespielt_sonntags") } else { &yl.last }, rt.x + 14.0, y + 42.0, 15.0, rt.w - 28.0, Theme::text());
}

// ================================================================ Tabellen
fn tabellen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let l = v.table_league.unwrap_or(c.league());
    let lw = 230.0;
    let la = Rect::new(X0 + 4.0, Y0 - 40.0, lw, hoehe() - Y0 + 24.0);
    ui.card(la);
    // the list: Europe and the own country's cup first, then all leagues, then the other cups (index 1000 + cup)
    let own_country = crate::cups::country(&c.world.leagues[c.league()]);
    let mut entries: Vec<usize> = c.world.cups.iter().enumerate().filter(|(_, x)| x.kind > 0 || Some(&x.country) == own_country.as_ref()).map(|(i, _)| 1000 + i).collect();
    entries.extend(0..c.world.leagues.len());
    entries.extend(c.world.cups.iter().enumerate().filter(|(_, x)| x.kind == 0 && Some(&x.country) != own_country.as_ref()).map(|(i, _)| 1000 + i));
    let n = entries.len();
    let vis = ((la.h - 10.0) / 30.0) as usize;
    if la.contains(ui.mouse) && ui.wheel != 0.0 { v.league_scroll = (v.league_scroll as i32 - ui.wheel.signum() as i32 * 2).clamp(0, (n.saturating_sub(vis)) as i32) as usize; }
    for (k, &i) in entries.iter().skip(v.league_scroll).take(vis).enumerate() {
        let r = Rect::new(la.x + 4.0, la.y + 6.0 + k as f32 * 30.0, la.w - 8.0, 28.0);
        let sel = i == l;
        if sel { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::akzent(), 0.18)); } else if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::erhoehung()); }
        let (name, mine, dot) = if i >= 1000 { let cup = &c.world.cups[i - 1000]; (cup.name.clone(), cup.teams.contains(&c.own), true) } else { (c.world.leagues[i].name.clone(), i == c.league(), c.world.leagues[i].detail <= 1) };
        let key = if i >= 1000 { &c.world.cups[i - 1000].key } else { &c.world.leagues[i].key };
        let tx = if ui.wettbewerb_logo(r.x + 14.0, r.y + 14.0, 10.0, c.world.cids.get(key).copied()) { r.x + 30.0 } else { r.x + 10.0 };
        let name = ui.fit(&name, 16.0, r.w - 20.0 - (tx - r.x));
        ui.text(&name, tx, r.y + 20.0, 16.0, if sel { Theme::akzent() } else if mine { Theme::gold() } else { Theme::text() });
        if dot { draw_circle(r.x + r.w - 12.0, r.y + 14.0, 3.0, if i >= 1000 { Theme::gold() } else { Theme::gut() }); }
        if ui.hit(r) { v.table_league = Some(i); }
    }
    let x = X0 + lw + 18.0;
    let tw = cbreite() - lw - 14.0;
    if l >= 1000 && l - 1000 < c.world.cups.len() { cup_view(ui, c, l - 1000, x, tw); return; }
    let lg = &c.world.leagues[l];
    let lx = if ui.wettbewerb_logo(x + 16.0, Y0 - 22.0, 17.0, c.world.cids.get(&lg.key).copied()) { x + 42.0 } else { x };
    ui.title(&lg.name, lx, Y0 - 12.0, 28.0, Theme::text());
    // zones from daten/ligaregeln.toml: up, play-offs, relegation tie, playout, down; Europe for a top league
    let regel = c.world.regel(l);
    let oben = c.world.ueber_index(l);
    let (auf, ab) = (if oben.is_some() { regel.auf } else { 0 }, regel.ab);
    let po: Vec<usize> = regel.playoffs.as_ref().map(|p| p.plaetze.clone()).unwrap_or_default();
    let rel = regel.relegation.as_ref().map_or(0, |r| r.platz);
    let pout: Vec<usize> = regel.playout.as_ref().map(|p| p.plaetze.clone()).unwrap_or_default();
    let euro: Vec<(usize, usize)> = c.world.uefa_ligen().iter().find(|x| x.0 == l).map(|(_, land)| land.europa.as_ref().unwrap().plaetze.iter().flat_map(|p| {
        let t = [crate::id("cl"), crate::id("el"), crate::id("uecl")].iter().position(|w| *w == p.wettbewerb).unwrap_or(2);
        p.plaetze.iter().filter_map(move |o| if let crate::ligaregeln::Ort::Platz(n) = o { Some((*n, t)) } else { None })
    }).collect()).unwrap_or_default();
    let hinweis = match lg.detail { 0 => crate::inhalt::t("ui.pages.deine_liga_jedes_spiel_mit"), 1 => crate::inhalt::t("ui.pages.detailliert_torschuetzen_und_verletzunge"), _ => crate::inhalt::t("ui.pages.nur_ergebnisse") };
    let hx = (x + 260.0).max(lx + ui.measure_title(&lg.name, 28.0) + 16.0);
    ui.text(hinweis, hx, Y0 - 10.0, 15.0, Theme::text2());
    // estimated in daten/ligaregeln.toml (sicher = false) or no entry there: the standard rules
    if !regel.sicher { ui.text(crate::inhalt::t("ui.pages.regel_geschaetzt"), hx + ui.measure(hinweis, 15.0), Y0 - 10.0, 15.0, Theme::achtung()); }
    let table = c.world.table(l);
    let rows = table.len();
    let area = Rect::new(x, Y0 + 6.0, tw * 0.64, hoehe() - Y0 - 22.0);
    ui.card(area);
    player_row_head(ui, area.x, area.y + 22.0, &[("#", 10.0), (crate::inhalt::t("ui.wort.gross_verein"), 70.0), (crate::inhalt::t("ui.wort.gross_sp"), 330.0), (crate::inhalt::t("ui.wort.buchstabe_s"), 370.0), (crate::inhalt::t("ui.wort.buchstabe_u"), 400.0), (crate::inhalt::t("ui.wort.buchstabe_n"), 430.0), (crate::inhalt::t("ui.wort.gross_tore"), 470.0), (crate::inhalt::t("ui.wort.gross_pkt"), 540.0), (crate::inhalt::t("ui.wort.gross_form"), 590.0)]);
    // rows keep a readable height (26 to 32); a long table (league phase, 36 teams) scrolls instead of being squeezed
    let rh = ((area.h - 36.0) / rows as f32).clamp(26.0, 32.0);
    let vis = ((area.h - 36.0) / rh) as usize;
    let start = scroll_rows(ui, v, area, rows, vis);
    for (i, r) in table.iter().enumerate().skip(start).take(vis) {
        let y = area.y + 30.0 + (i - start) as f32 * rh;
        let rr = Rect::new(area.x + 4.0, y, area.w - 8.0, rh - 2.0);
        if r.club == c.own { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::akzent(), 0.16)); }
        let pl = i + 1;
        let zone = if ab > 0 && i + ab >= rows { Some(Theme::problem()) } else if pl == rel || pout.contains(&pl) { Some(Theme::achtung()) } else if pl <= auf || pl == 1 { Some(hex(0xD9B44A)) }
            else if po.contains(&pl) { Some(Theme::akzent()) } else { euro.iter().find(|e| e.0 == pl).map(|e| if e.1 == 0 { Theme::akzent() } else { Theme::gut() }) };
        if let Some(z) = zone { draw_rectangle(rr.x, rr.y + 2.0, 3.0, rr.h - 4.0, z); }
        let ty = y + rh * 0.7;
        let fs = (rh * 0.55).min(17.0);
        ui.text(&format!("{}", i + 1), area.x + 12.0, ty, fs, Theme::text2());
        let (c1, c2) = club_color(c, r.club);
        draw_circle(area.x + 52.0, y + rh / 2.0 - 1.0, rh * 0.32, c1);
        draw_circle_lines(area.x + 52.0, y + rh / 2.0 - 1.0, rh * 0.32, 1.5, c2);
        let sperre = oben.map_or(false, |o| !c.world.darf_aufsteigen(r.club, o));
        let nm = ui.fit(&c.world.clubs[r.club].name, fs, if sperre { 170.0 } else { 250.0 });
        ui.text_semi(&nm, area.x + 70.0, ty, fs, Theme::text());
        if sperre { let w = ui.measure_semi(&nm, fs); ui.text(crate::inhalt::t("ui.pages.kein_aufstieg"), area.x + 78.0 + w, ty, 12.0, Theme::text2()); }
        for (val, cx) in [(r.p as i32, 330.0), (r.w as i32, 370.0), (r.d as i32, 400.0), (r.l as i32, 430.0)] { ui.text(&format!("{val}"), area.x + cx, ty, fs, Theme::text()); }
        ui.text(&format!("{}:{}", r.gf, r.ga), area.x + 470.0, ty, fs, Theme::text());
        ui.text_semi(&format!("{}", r.pts), area.x + 540.0, ty, fs, Theme::text());
        for (j, ch) in r.form.iter().enumerate() { let col = match ch { 'S' => Theme::gut(), 'N' => Theme::problem(), _ => Theme::achtung() }; rrect(area.x + 590.0 + j as f32 * 14.0, y + rh * 0.25, 10.0, rh * 0.45, 2.0, col); }
        if ui.hit(rr) && r.club != c.own { /* club profile: not in this version */ }
    }
    // right: top scorers or last results
    let rx = x + tw * 0.64 + 12.0;
    let rr = Rect::new(rx, Y0 + 6.0, tw * 0.36 - 12.0, hoehe() - Y0 - 22.0);
    ui.card_titled(rr, if lg.detail <= 1 { crate::inhalt::t("statistik.torjaeger_hinweis") } else { crate::inhalt::t("verein.letzte") }, Theme::gold());
    let mut y = rr.y + 54.0;
    if lg.detail <= 1 {
        let mut scorers: Vec<usize> = lg.teams.iter().flat_map(|&t| c.world.clubs[t].squad.iter().copied()).filter(|&p| c.world.players[p].goals > 0 && c.world.players[p].youth == 0).collect();
        scorers.sort_by_key(|&p| std::cmp::Reverse(c.world.players[p].goals));
        if scorers.is_empty() { ui.text(crate::inhalt::t("statistik.keine_tore"), rr.x + 14.0, y, 16.0, Theme::text2()); }
        for &p in scorers.iter().take(15) {
            let pl = &c.world.players[p];
            ui.player_photo(rr.x + 24.0, y - 5.0, 10.0, pl.id, &pl.name);
            let nm = ui.fit(&pl.name, 16.0, rr.w - 90.0);
            ui.text(&nm, rr.x + 40.0, y, 16.0, Theme::text());
            ui.text_right(&format!("{}", pl.goals), rr.x + rr.w - 14.0, y, 16.0, Theme::gold());
            y += 26.0;
        }
    } else {
        let mut played: Vec<&Fixture> = lg.fixtures.iter().filter(|f| f.hg >= 0).collect();
        played.reverse();
        for f in played.iter().take(16) {
            let t = format!("{} – {}", c.world.clubs[f.h].short, c.world.clubs[f.a].short);
            ui.text(&t, rr.x + 14.0, y, 16.0, Theme::text());
            ui.text_right(&format!("{}:{}", f.hg, f.ag), rr.x + rr.w - 14.0, y, 16.0, Theme::text());
            y += 26.0;
        }
    }
}

/// A cup or European competition: league-phase table (Europe) and the knock-out rounds with results.
fn cup_view(ui: &mut Ui, c: &Career, ci: usize, x: f32, tw: f32) {
    let cup = &c.world.cups[ci];
    let lx = if ui.wettbewerb_logo(x + 16.0, Y0 - 22.0, 17.0, c.world.cids.get(&cup.key).copied()) { x + 42.0 } else { x };
    ui.title(&cup.name, lx, Y0 - 12.0, 28.0, Theme::text());
    let status = match cup.winner { Some(w) => crate::tf!("ui.pages.sieger", c.world.clubs[w].name), None => crate::tf!("ui.pages.laeuft", if cup.stage < 10 { crate::inhalt::t("ui.pages.ligaphase").to_string() } else { crate::cups::stage_name(cup.stage, cup.kind) }) };
    ui.text(&status, x + 300.0, Y0 - 10.0, 15.0, Theme::text2());
    let mut rx = x;
    let mut rw = tw;
    if (1..=3).contains(&cup.kind) {
        let area = Rect::new(x, Y0 + 6.0, tw * 0.5, hoehe() - Y0 - 22.0);
        ui.card_titled(area, crate::inhalt::t("ui.pages.ligaphase"), Theme::gold());
        let tab = c.world.cup_table(ci);
        let rh = ((area.h - 44.0) / tab.len().max(1) as f32).min(22.0);
        for (i, r) in tab.iter().enumerate() {
            let y = area.y + 44.0 + i as f32 * rh;
            if r.club == c.own { rrect(area.x + 4.0, y, area.w - 8.0, rh - 1.0, 3.0, alpha(Theme::akzent(), 0.16)); }
            let zone = if i < 8 { Theme::gut() } else if i < 24 { Theme::achtung() } else { Theme::problem() };
            draw_rectangle(area.x + 4.0, y + 2.0, 3.0, rh - 4.0, zone);
            let fs = (rh * 0.7).min(15.0);
            ui.text(&format!("{}", i + 1), area.x + 12.0, y + rh * 0.75, fs, Theme::text2());
            let nm = ui.fit(&c.world.clubs[r.club].name, fs, area.w - 140.0);
            ui.text(&nm, area.x + 40.0, y + rh * 0.75, fs, Theme::text());
            ui.text_right(&format!("{}  {:+}  {}", r.p, r.gf - r.ga, r.pts), area.x + area.w - 12.0, y + rh * 0.75, fs, Theme::text());
        }
        rx = x + tw * 0.5 + 12.0;
        rw = tw * 0.5 - 12.0;
    }
    let rr = Rect::new(rx, Y0 + 6.0, rw, hoehe() - Y0 - 22.0);
    ui.card_titled(rr, crate::inhalt::t("wb.liste.runden"), Theme::gold());
    let mut stages: Vec<u8> = cup.fx.iter().map(|f| f.stage).filter(|s| *s >= 20).collect();
    stages.sort();
    stages.dedup();
    stages.reverse();
    let mut y = rr.y + 54.0;
    if stages.is_empty() { ui.text(crate::inhalt::t("ui.pages.die_k_o_runden_folgen"), rr.x + 14.0, y, 15.0, Theme::text2()); }
    for st in stages {
        if y > rr.y + rr.h - 30.0 { break; }
        ui.text_semi(&crate::cups::stage_name(st, cup.kind), rr.x + 14.0, y, 15.0, Theme::akzent());
        y += 22.0;
        for f in cup.fx.iter().filter(|f| f.stage == st) {
            if y > rr.y + rr.h - 10.0 { break; }
            let res = if f.f.hg < 0 { date::short(c.world.season, f.f.day) } else { format!("{}:{}{}{}", f.f.hg, f.f.ag, if f.et { crate::inhalt::t("ui.pages.kurz_nv") } else { "" }, f.pens.map(|p| crate::tf!("ui.pages.kurz_ie", crate::cups::pens_score(p))).unwrap_or_default()) };
            let t = format!("{} – {}", c.world.clubs[f.f.h].short, c.world.clubs[f.f.a].short);
            let mine = f.f.h == c.own || f.f.a == c.own;
            ui.text(&t, rr.x + 14.0, y, 15.0, if mine { Theme::gold() } else { Theme::text() });
            ui.text_right(&res, rr.x + rr.w - 14.0, y, 15.0, Theme::text2());
            y += 20.0;
        }
        y += 8.0;
    }
}

// ================================================================ Presse
fn presse(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("zentrale.k.schlagzeilen"), crate::inhalt::t("vorbereitung.pk")]);
    if v.tab == 1 || c.press.is_some() && v.tab == 0 && false { }
    if v.tab == 1 {
        press_card(ui, v, c, Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0));
        return;
    }
    if c.press.is_some() {
        let r = Rect::new(X0 + 4.0, Y0, cbreite(), 60.0);
        rrect(r.x, r.y, r.w, r.h, 8.0, alpha(Theme::achtung(), 0.12));
        ui.title(crate::inhalt::t("ui.pages.die_presse_wartet_auf_dich"), r.x + 16.0, r.y + 38.0, 24.0, Theme::achtung());
        if ui.button(Rect::new(r.x + r.w - 200.0, r.y + 12.0, 184.0, 36.0), crate::inhalt::t("ui.pages.zur_konferenz"), true) { v.tab = 1; ui.play(crate::id("pressefrage")); }
    }
    let y0 = if c.press.is_some() { Y0 + 72.0 } else { Y0 };
    let cols = 3;
    let cw = (cbreite() - 24.0) / 3.0;
    let ch = 170.0;
    let rows = ((hoehe() - y0 - 16.0) / (ch + 12.0)) as usize;
    let area = Rect::new(X0, y0, cbreite(), hoehe() - y0);
    let start = scroll_rows(ui, v, area, (c.feed.len() + cols - 1) / cols, rows) * cols;
    if c.feed.is_empty() { ui.text(crate::inhalt::t("md.ui.leer.schlagzeilen"), X0 + 10.0, y0 + 30.0, 18.0, Theme::text2()); }
    for (k, n) in c.feed.iter().skip(start).take(rows * cols).enumerate() {
        let r = Rect::new(X0 + 4.0 + (k % cols) as f32 * (cw + 12.0), y0 + (k / cols) as f32 * (ch + 12.0), cw, ch);
        rrect(r.x, r.y, r.w, r.h, 6.0, hex(0xE9E4D8));
        ui.title(&n.paper.to_uppercase(), r.x + 14.0, r.y + 26.0, 18.0, hex(0x1A1A1A));
        ui.text_right(&date::short(n.season, n.day), r.x + r.w - 12.0, r.y + 24.0, 13.0, hex(0x555555));
        draw_rectangle(r.x + 12.0, r.y + 34.0, r.w - 24.0, 2.0, hex(0x1A1A1A));
        let mut y = r.y + 60.0;
        for l in ui.wrap(&n.head, 20.0, r.w - 28.0).into_iter().take(2) { ui.text_semi(&l, r.x + 14.0, y, 20.0, hex(0x111111)); y += 22.0; }
        for l in ui.wrap(&n.text, 14.0, r.w - 28.0).into_iter().take(4) { ui.text(&l, r.x + 14.0, y + 4.0, 14.0, hex(0x333333)); y += 18.0; }
    }
}

pub fn press_card(ui: &mut Ui, _v: &mut View, c: &mut Career, r: Rect) {
    use crate::inhalt::t;
    ui.card_titled(r, t("vorbereitung.pk"), Theme::akzent());
    let Some(pq) = c.press.clone() else {
        ui.text(t("ui.pages.gerade_steht_keine_pressekonferenz_an"), r.x + 16.0, r.y + 60.0, 17.0, Theme::text2());
        return;
    };
    // the journalist as a person: drawn face, name, outlet, style and attitude (medien.rs, medienpool.json)
    let j = pq.journalist as usize;
    let (jn, stil, haltung) = c.me.journalisten.get(j).cloned().unwrap_or_else(|| (t("ui.medien.pressestelle").to_string(), 0, 0.0));
    let blatt = c.blatt(j);
    let stil_t = t(&format!("md.ui.journalist.stil.{}", crate::medien::STILE[(stil as usize).min(3)]));
    let halt_t = t(if haltung > 15.0 { "md.ui.journalist.haltung.gut" } else if haltung < -15.0 { "md.ui.journalist.haltung.schlecht" } else { "md.ui.journalist.haltung.neutral" });
    let (fx, fy, fr) = (r.x + 64.0, r.y + 100.0, 44.0);
    ui.figur(fx, fy, fr, &jn, crate::ui::Rolle::Presse);
    ui.text_semi(&jn, r.x + 124.0, r.y + 84.0, 20.0, Theme::text());
    ui.text(&ui.fit(&format!("{} · {}", if blatt.is_empty() { "–".to_string() } else { blatt.clone() }, stil_t), 15.0, r.w * 0.45), r.x + 124.0, r.y + 106.0, 15.0, Theme::text2());
    ui.text(halt_t, r.x + 124.0, r.y + 128.0, 14.0, if haltung > 15.0 { Theme::gut() } else if haltung < -15.0 { Theme::problem() } else { Theme::gedaempft() });
    // the needles: what the answer moves (media, board, fans, dressing room), current values
    let zeiger = [(t("ui.presse.medien"), c.media), (t("ui.presse.vorstand"), c.mensch.vorstand), (t("ui.presse.fans"), c.fans), (t("ui.presse.kabine"), c.mensch.teamgeist)];
    let zx = r.x + r.w * 0.58;
    for (i, (n, w)) in zeiger.iter().enumerate() {
        let y = r.y + 70.0 + i as f32 * 20.0;
        ui.text(n, zx, y, 13.0, Theme::text2());
        ui.bar(zx + 110.0, y - 9.0, r.w * 0.42 - 170.0, 8.0, (*w / 100.0).clamp(0.0, 1.0), Theme::skala(40.0 + *w * 0.59));
        ui.text_right(&format!("{w:.0}"), r.x + r.w - 16.0, y, 13.0, Theme::text());
    }
    // the question as a speech bubble from the journalist
    let qy = r.y + 156.0;
    let zeilen = ui.wrap(&pq.q, 19.0, r.w - 64.0);
    let qh = 22.0 + zeilen.len() as f32 * 25.0;
    rrect(r.x + 16.0, qy, r.w - 32.0, qh, 12.0, Theme::erhoehung());
    draw_triangle(vec2(fx - 8.0, fy + fr - 2.0), vec2(fx + 14.0, fy + fr - 2.0), vec2(fx + 2.0, qy + 1.0), Theme::erhoehung());
    let mut yy = qy + 28.0;
    for z in &zeilen { ui.text(z, r.x + 32.0, yy, 19.0, Theme::text()); yy += 25.0; }
    // answers as tone cards: tone, the answer, its effect
    let mut y = qy + qh + 12.0;
    let spalten = if r.w > 900.0 { 2 } else { 1 };
    let kw = (r.w - 32.0 - (spalten as f32 - 1.0) * 10.0) / spalten as f32;
    let mut picked = None;
    for (i, (a, ton)) in pq.answers.iter().enumerate() {
        let (sx, sy) = (r.x + 16.0 + (i % spalten) as f32 * (kw + 10.0), y + (i / spalten) as f32 * 82.0);
        let kr = Rect::new(sx, sy, kw, 74.0);
        let hover = ui.hover(kr);
        rrect(kr.x, kr.y, kr.w, kr.h, 10.0, if hover { alpha(Theme::akzent(), 0.18) } else { Theme::erhoehung() });
        let key = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4][i.min(3)];
        let ton_n = t(&format!("md.ui.ton.{}", crate::medien::TOENE[(*ton % 10) as usize % crate::medien::TOENE.len()]));
        ui.text_semi(&format!("{}  {}", i + 1, ton_n), kr.x + 12.0, kr.y + 20.0, 14.0, Theme::akzent());
        ui.paragraph(&ui.fit(a, 15.0, (kr.w - 24.0) * 2.0), kr.x + 12.0, kr.y + 42.0, 15.0, kr.w - 24.0, Theme::text());
        let (m, b, f, mo) = tone_effect(*ton % 10);
        let eff = crate::tf!("ui.pages.medien_praesidium_fans_kabine", format!("{:+}", m), format!("{:+}", b), format!("{:+}", f), format!("{:+}", mo));
        ui.text_right(&eff, kr.x + kr.w - 12.0, kr.y + 20.0, 12.0, Theme::gedaempft());
        if ui.hit(kr) || is_key_pressed(key) { picked = Some(i); }
    }
    y += ((pq.answers.len() + spalten - 1) / spalten) as f32 * 82.0;
    if ui.button(Rect::new(r.x + 16.0, (y + 4.0).min(r.y + r.h - 44.0), 220.0, 34.0), t("ui.pages.nicht_hingehen"), false) { c.answer_press(None); return; }
    if let Some(i) = picked { c.answer_press(Some(i)); ui.play(crate::id("bestaetigt")); }
}

// ================================================================ Optionen
fn optionen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let r = Rect::new(X0 + 4.0, Y0 - 40.0, cbreite(), 340.0);
    ui.card_titled(r, crate::inhalt::t("menu.spielstand_und_karriere.reiter.0"), Theme::akzent());
    ui.text(&crate::tf!("ui.pages.gespeichert_wird_in", c.save_path().display()), r.x + 16.0, r.y + 60.0, 16.0, Theme::text2());
    ui.text(crate::inhalt::t("ui.pages.pes_m_sichert_jeden_montag"), r.x + 16.0, r.y + 86.0, 16.0, Theme::text2());
    if ui.button_gold(Rect::new(r.x + 16.0, r.y + 110.0, 240.0, 44.0), crate::inhalt::t("ui.pages.jetzt_sichern")) { match c.save() { Ok(_) => v.toast(crate::inhalt::t("ui.pages.gesichert"), true), Err(e) => v.toast(&crate::tf!("ui.pages.fehler_beim_sichern", e), false) } }
    ui.text_semi(crate::inhalt::t("ui.pages.thema"), r.x + 760.0, r.y + 60.0, 17.0, Theme::text());
    if let Some(t) = ui.segment(Rect::new(r.x + 760.0, r.y + 74.0, 380.0, 38.0), &[crate::inhalt::t("ui.pages.sonntag_hell"), crate::inhalt::t("ui.pages.flutlicht_dunkel")], v.dark as usize) {
        v.dark = t == 1;
        let _ = std::fs::create_dir_all(save_dir());
        let _ = std::fs::write(theme_file(), if v.dark { crate::id("flutlicht") } else { crate::id("sonntag") });
    }
    ui.text_semi(crate::inhalt::t("optionen.sprache"), r.x + 760.0, r.y + 140.0, 17.0, Theme::text());
    let en = crate::inhalt::sprache() == crate::inhalt::Sprache::En;
    if let Some(t) = ui.segment(Rect::new(r.x + 760.0, r.y + 154.0, 380.0, 38.0), &[crate::inhalt::t("sprache.de"), crate::inhalt::t("sprache.en")], en as usize) {
        crate::inhalt::sprache_setzen(if t == 1 { crate::inhalt::Sprache::En } else { crate::inhalt::Sprache::De });
        let _ = std::fs::create_dir_all(save_dir());
        let _ = std::fs::write(sprache_file(), if t == 1 { crate::id("en") } else { crate::id("de") });
    }
    ui.text_semi(crate::inhalt::t("ui.klang.titel"), r.x + 300.0, r.y + 110.0, 17.0, Theme::text());
    let stufen = [crate::id("aus"), crate::id("leise"), crate::id("normal")];
    let jetzt = stufen.iter().position(|s| (klang_wert(s) - ui.lautstaerke).abs() < 0.01).unwrap_or(1);
    if let Some(k) = ui.segment(Rect::new(r.x + 300.0, r.y + 124.0, 380.0, 34.0), &[crate::inhalt::t("ui.klang.aus"), crate::inhalt::t("ui.klang.leise"), crate::inhalt::t("ui.klang.normal")], jetzt) {
        ui.lautstaerke = klang_wert(stufen[k]);
        let _ = std::fs::create_dir_all(save_dir());
        let _ = std::fs::write(klang_file(), stufen[k]);
        ui.play(crate::id("bestaetigt"));
    }
    ui.text_semi(crate::inhalt::t("ui.kaderregel.titel"), r.x + 16.0, r.y + 266.0, 17.0, Theme::text());
    if let Some(k) = ui.segment(Rect::new(r.x + 16.0, r.y + 280.0, 300.0, 34.0), &[crate::inhalt::t("ui.kaderregel.an"), crate::inhalt::t("ui.kaderregel.aus")], (!c.kaderregeln) as usize) { c.kaderregeln = k == 0; }
    ui.text_semi(crate::inhalt::t("ui.akzent.titel"), r.x + 760.0, r.y + 210.0, 17.0, Theme::text());
    if let Some(t) = ui.segment(Rect::new(r.x + 760.0, r.y + 224.0, 380.0, 38.0), &[crate::inhalt::t("ui.akzent.wappen"), crate::inhalt::t("ui.akzent.standard")], (!c.akzent_wappen) as usize) { c.akzent_wappen = t == 0; }
    ui.text_semi(crate::inhalt::t("ui.pages.co_trainer_uebernimmt"), r.x + 16.0, r.y + 190.0, 17.0, Theme::text());
    let mut x = r.x + 16.0;
    for (i, (label, on)) in [(crate::inhalt::t("aktion.aufstellung"), c.co_lineup), (crate::inhalt::t("status.training"), c.co_training), (crate::inhalt::t("md.ui.presse.verlauf"), c.co_press)].iter().enumerate() {
        if ui.button(Rect::new(x, r.y + 206.0, 220.0, 36.0), &format!("{} {}", label, if *on { "✓" } else { "" }), *on) {
            match i { 0 => c.co_lineup = !on, 1 => c.co_training = !on, _ => c.co_press = !on }
        }
        x += 232.0;
    }
    let r2 = Rect::new(X0 + 4.0, Y0 + 314.0, cbreite(), hoehe() - Y0 - 330.0);
    ui.card_titled(r2, crate::inhalt::t("ui.pages.football_life_2027"), Theme::gold());
    let mut on = c.fl27_erlaubt;
    if ui.button(Rect::new(r2.x + 16.0, r2.y + 48.0, 240.0, 34.0), if on { crate::inhalt::t("ui.pages.spielstart_erlaubt") } else { crate::inhalt::t("ui.pages.spielstart_erlaubt_2") }, on) { on = !on; c.fl27_erlaubt = on; }
    let live = c.db_live;
    if ui.button(Rect::new(r2.x + 268.0, r2.y + 48.0, 290.0, 34.0), if live { crate::inhalt::t("ui.pages.datenbank_live_mitschreiben") } else { crate::inhalt::t("ui.pages.datenbank_live_mitschreiben_2") }, live) { c.db_live = !live; }
    let auto = c.fl27_auto_ende;
    if ui.button(Rect::new(r2.x + 570.0, r2.y + 48.0, 320.0, 34.0), if auto { crate::inhalt::t("ui.pages.fl27_nach_dem_abpfiff_beenden") } else { crate::inhalt::t("ui.pages.fl27_nach_dem_abpfiff_beenden_2") }, auto) { c.fl27_auto_ende = !auto; }
    // right column under „Rückbau“: the career database after the match, and presets still lying in the game folder
    let db_aus = c.db_nach_spiel_aus;
    if ui.button(Rect::new(r2.x + r2.w - 260.0, r2.y + 90.0, 244.0, 34.0), &ui.fit(crate::inhalt::t(if db_aus { "ui.fl27.db_nach_spiel_aus_an" } else { "ui.fl27.db_nach_spiel_aus" }), 14.0, 230.0), db_aus) { c.db_nach_spiel_aus = !db_aus; }
    let liegen: Vec<&str> = crate::fl27::VORGABEN.iter().copied().filter(|n| crate::fl27::sider().join(crate::id("flw")).join(n).exists()).collect();
    if !liegen.is_empty() && !v.fl27_laeuft {
        ui.paragraph(&crate::tf!("ui.fl27.vorgaben_liegen", liegen.len()), r2.x + r2.w - 260.0, r2.y + 148.0, 13.0, 244.0, Theme::achtung());
        if ui.button(Rect::new(r2.x + r2.w - 260.0, r2.y + 186.0, 244.0, 34.0), crate::inhalt::t("ui.fl27.vorgaben_entfernen"), false) {
            match crate::fl27::vorgaben_schreiben(&crate::fl27::sider(), None, None, None, None) { Ok(_) => { c.vorgaben_gesetzt.clear(); v.toast(crate::inhalt::t("ui.fl27.vorgaben_zurueck"), true) } Err(e) => v.toast(&e, false) }
        }
    }
    let s = crate::fl27::sider();
    let ok = |b: bool| if b { crate::id("ok") } else { crate::id("fehlt") };
    let welt = c.welt().join("common/etc/pesdb/PlayerAssignment.bin").exists();
    let ini = std::fs::read_to_string(s.join("sider.ini")).unwrap_or_default();
    let zeile = !c.fl27_cpk.is_empty() && ini.lines().any(|l| l.trim_start().starts_with("cpk.root") && l.contains(&format!("\\{}\"", c.fl27_cpk)));
    let module = crate::fl27::MODULE.iter().filter(|(rel, _)| std::fs::read_to_string(s.join(rel)).map(|t| t.contains("PES M")).unwrap_or(false)).count(); // roh
    let lines = [
        crate::tf!("ui.pages.spielordner_und_welt_datenbank", ok(welt)),
        crate::tf!("ui.pages.karriere_zeile_in_sider_ini", if zeile { crate::inhalt::t("ui.wort.aktiv") } else { crate::inhalt::t("ui.pages.noch_nicht_eingerichtet") }, module),
        crate::tf!("ui.pages.speicher_helfer", if crate::fl27::helfer_pfad().exists() { crate::inhalt::t("ui.wort.gebaut") } else { crate::inhalt::t("ui.wort.fehlt") }, crate::fl27::freischalten_hinweis()),
        crate::tf!("ui.pages.original_sicherung", if c.karriere_dir().join("original/inhalt.json").exists() { crate::inhalt::t("ui.wort.vorhanden") } else { crate::inhalt::t("ui.pages.wird_beim_ersten_start_angelegt") }),
        if c.db_zuletzt.is_empty() { crate::inhalt::t("ui.pages.zuletzt_geschrieben_noch_nicht").to_string() } else { c.db_zuletzt.clone() },
    ];
    let mut y = r2.y + 108.0;
    for l in lines { y = ui.paragraph(&l, r2.x + 16.0, y, 15.0, r2.w - 300.0, Theme::text2()) + 2.0; }
    if ui.button(Rect::new(r2.x + r2.w - 260.0, r2.y + 48.0, 244.0, 34.0), crate::inhalt::t("ui.pages.rueckbau_karriere_aus_fl27"), false) {
        let k = c.karriere_dir();
        let cpk = c.fl27_cpk.clone();
        let r = crate::fl27::sider_ini_anwenden(&k, &s, |t| Ok(crate::fl27::sider_ini_rueckbau(t, &cpk))).and_then(|_| crate::fl27::vorgaben_schreiben(&s, None, None, None, None));
        match r { Ok(_) => { c.vorgaben_gesetzt.clear(); v.toast(crate::inhalt::t("ui.pages.karriere_zeile_und_vorgaben_entfernt"), true) } Err(e) => v.toast(&e, false) }
    }
}

// ================================================================ Spieltag
fn spieltag(ui: &mut Ui, v: &mut View, c: &mut Career) {
    if v.fl27.is_none() && !v.fl27_vorschau { v.fl27_ansicht = None; v.fl27_seq = None; v.fl27_einblendung = None; }
    if v.mv.is_none() {
        let Some(fi) = c.today_match() else {
            ui.title(crate::inhalt::t("ui.pages.heute_ist_kein_spiel"), X0 + 10.0, Y0 + 40.0, 30.0, Theme::text());
            if let Some(f) = c.next_match() { ui.text(&crate::tf!("ui.pages.naechstes_spiel_gegen", date::long(c.world.season, f.day), c.world.clubs[if f.h == c.own { f.a } else { f.h }].name), X0 + 10.0, Y0 + 76.0, 19.0, Theme::text2()); }
            if let Some(r) = c.last_result.clone() { match_report(ui, c, &r, Rect::new(X0 + 4.0, Y0 + 110.0, cbreite(), hoehe() - Y0 - 126.0)); }
            return;
        };
        let f = c.fixture(fi);
        v.mv = Some(MatchView { res: MatchResult { h: f.h, a: f.a, ..Default::default() }, minute: 0.0, speed: 1.0, phase: 0, ended_at: 0.0, manual: (0, 0), manual_open: false, halbzeit: 0 });
    }
    let mut mv = v.mv.take().unwrap();
    let (h, a) = (mv.res.h, mv.res.a);
    // scoreboard
    let sb = Rect::new(X0 + 4.0, Y0 - 42.0, cbreite(), 110.0);
    ui.card(sb);
    // the home team's stadium behind the scoreboard, dimmed so the names stay readable
    if ui.stadion_bild(sb, c.world.clubs[h].id) { draw_rectangle(sb.x, sb.y, sb.w, sb.h, alpha(Theme::flaeche(), 0.72)); }
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
    let lx = if ui.wettbewerb_logo(sb.x + 130.0, sb.y + 85.0, 10.0, cid) { sb.x + 146.0 } else { sb.x + 120.0 };
    ui.text(&format!("{} · {}", comp, date::long(c.world.season, c.world.day)), lx, sb.y + 90.0, 15.0, Theme::text2());
    let body = Rect::new(X0 + 4.0, sb.y + sb.h + 12.0, cbreite(), hoehe() - sb.y - sb.h - 28.0);
    match mv.phase {
        0 => {
            // the live view of a match in FL27 lies over this page; the cards below it take no clicks meanwhile
            let live_an = v.fl27_ansicht.is_some() && v.fl27_ko.is_none();
            let klick = ui.clicked;
            if live_an { ui.clicked = false; }
            // it also covers the page banner: its own scoreboard says what the banner would
            let live_flaeche = Rect::new(X0 + 4.0, 62.0, cbreite(), hoehe() - 62.0 - 16.0);
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
                c.press = None;
                mv.res = c.prepare_match(None);
                mv.phase = 1;
                ui.play(crate::id("anpfiff"));
            }
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
                        Ok(()) => { let tf = c.fl27_team_fn(); v.fl27 = Some(crate::fl27::beobachten(tf)); v.fl27_status = crate::inhalt::t("ui.pages.warte_auf_fl27").into(); v.fl27_t0 = get_time() as f32; ui.play(crate::id("bestaetigt")); }
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
                        crate::fl27::Meldung::Fertig(d) => fertig = Some(d),
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
            if mv.halbzeit != 1 { mv.minute += get_frame_time() * 7.5 * mv.speed; }
            if mv.halbzeit == 0 && before < 45.0 && mv.minute >= 45.0 { mv.minute = 45.0; mv.halbzeit = 1; }
            for e in mv.res.events.iter().filter(|e| (e.min as f32) > before && (e.min as f32) <= mv.minute) { match e.kind { 0 | 1 => ui.play(crate::id("tor")), 2 | 3 => ui.play(crate::id("karte")), 4 => ui.play(crate::id("verletzung")), _ => {} } }
            let r = Rect::new(body.x, body.y, body.w * 0.62, body.h);
            ui.card_titled(r, crate::inhalt::t("ui.wort.live_ticker"), Theme::problem());
            let shown: Vec<&MatchEvent> = mv.res.events.iter().filter(|e| (e.min as f32) <= mv.minute).collect();
            let mut y = r.y + 56.0;
            for e in shown.iter().rev().take(14) {
                let team = if e.side == 0 { h } else { a };
                let pn = c.world.players[e.pid].name.clone();
                let (txt, col) = match e.kind {
                    0 => (crate::tf!("ui.pages.tor_fuer", c.world.clubs[team].name, pn, e.pid2.map(|p| crate::tf!("ui.pages.vorlage", c.world.players[p].name)).unwrap_or_default()), Theme::gut()),
                    1 => (crate::tf!("ui.pages.eigentor_von", pn), Theme::problem()),
                    2 => (crate::tf!("ui.pages.gelbe_karte_fuer", pn), Theme::achtung()),
                    3 => (crate::tf!("ui.pages.rote_karte_muss_runter", pn), Theme::problem()),
                    4 => (crate::tf!("ui.pages.verletzt_sich", pn), Theme::problem()),
                    5 => (crate::tf!("ui.pages.wechsel_bei_fuer", c.world.clubs[team].short, e.pid2.map(|p| c.world.players[p].name.clone()).unwrap_or_default(), pn), Theme::text2()),
                    _ => (crate::tf!("ui.pages.chance_fuer_scheitert", c.world.clubs[team].short, pn), Theme::text2()),
                };
                ui.text_semi(&format!("{}'", e.min), r.x + 16.0, y, 18.0, Theme::akzent());
                ui.player_photo(r.x + 72.0, y - 6.0, 11.0, c.world.players[e.pid].id, &pn);
                let t = ui.fit(&txt, 18.0, r.w - 104.0);
                ui.text(&t, r.x + 90.0, y, 18.0, col);
                y += 30.0;
            }
            let r2 = Rect::new(body.x + body.w * 0.62 + 12.0, body.y, body.w * 0.38 - 12.0, body.h);
            ui.card_titled(r2, crate::inhalt::t("bericht.achse.tempo"), Theme::akzent());
            let sp = [1.0, 2.0, 4.0];
            let sel = sp.iter().position(|s| *s == mv.speed).unwrap_or(0);
            if let Some(s) = ui.segment(Rect::new(r2.x + 14.0, r2.y + 44.0, r2.w - 28.0, 34.0), &[crate::inhalt::t("programm.intensitaet.normal"), crate::inhalt::t("ui.pages.schnell"), crate::inhalt::t("ui.pages.sehr_schnell")], sel) { mv.speed = sp[s]; }
            if mv.halbzeit == 1 {
                let (gf, ga) = c.halbzeit_tore(&mv.res);
                if let Some(wahl) = halbzeit_karte(ui, Rect::new(r.x + 16.0, r.y + 52.0, r.w - 32.0, 300.0), gf, ga) {
                    if let Some(k) = wahl { let t = c.halbzeit_antwort(Career::halbzeit_stand(gf, ga), &k); v.toast(&t, true); }
                    mv.halbzeit = 2;
                }
            }
            if ui.button(Rect::new(r2.x + 14.0, r2.y + 90.0, r2.w - 28.0, 40.0), crate::inhalt::t("ui.pages.zum_abpfiff_springen"), false) { mv.minute = 91.0; mv.halbzeit = 2; }
            ui.bar(r2.x + 14.0, r2.y + 150.0, r2.w - 28.0, 8.0, mv.minute / 90.0, Theme::akzent());
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
            match_report(ui, c, &mv.res, Rect::new(body.x, body.y, body.w * 0.66, body.h));
            // guide §3.4: a short win banner with confetti in the club colour
            let won = if mv.res.h == c.own { mv.res.hg > mv.res.ag } else { mv.res.ag > mv.res.hg };
            let age = get_time() as f32 - mv.ended_at;
            if won && age < 1.6 {
                let a = (1.6 - age).min(1.0);
                for k in 0..80 {
                    let fx = ((k * 7919) % 1000) as f32 / 1000.0;
                    let fy = ((k * 104729) % 1000) as f32 / 1000.0;
                    let x = X0 + fx * cbreite() + (age * 3.0 + k as f32).sin() * 12.0;
                    let y = 200.0 + (fy * 120.0 + age * 380.0) % 700.0;
                    let col = if k % 3 == 0 { hex(0xffffff) } else if k % 3 == 1 { Theme::verein() } else { Theme::gold() };
                    draw_rectangle(x, y, 7.0, 4.0, alpha(col, a));
                }
                let r = Rect::new(X0 + cbreite() / 2.0 - 220.0, 330.0, 440.0, 90.0);
                rrect(r.x, r.y, r.w, r.h, 18.0, alpha(Theme::verein(), a));
                let t = crate::id("SIEG!");
                let w = measure_text(t, Some(&ui.head), 64, 1.0).width;
                ui.title(t, r.x + r.w / 2.0 - w / 2.0, r.y + 66.0, 64.0, alpha(Theme::verein_auf(), a));
            }
            let r2 = Rect::new(body.x + body.w * 0.66 + 12.0, body.y, body.w * 0.34 - 12.0, body.h);
            if c.press.is_some() { press_card(ui, v, c, r2); } else {
                ui.card_titled(r2, crate::inhalt::t("ux.abschnitt.nach"), Theme::akzent());
                ui.paragraph(&mv.res.reason, r2.x + 14.0, r2.y + 60.0, 17.0, r2.w - 28.0, Theme::text());
                if ui.button_gold(Rect::new(r2.x + 14.0, r2.y + r2.h - 64.0, r2.w - 28.0, 48.0), crate::inhalt::t("ui.wort.gross_weiter")) { v.go(Page::Uebersicht); return; }
            }
        }
    }
    v.mv = Some(mv);
}

fn match_report(ui: &mut Ui, c: &Career, res: &MatchResult, r: Rect) {
    ui.card_titled(r, crate::inhalt::t("bericht.titel"), Theme::akzent());
    let mut y = r.y + 54.0;
    if res.et { ui.text_center(&crate::tf!("ui.pages.nach_verlaengerung", res.pens.map(|p| crate::tf!("ui.pages.komma_ie", crate::cups::pens_score(p))).unwrap_or_default()), r.x + r.w / 2.0, r.y + 22.0, 15.0, Theme::gold()); }
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

// ================================================================ player profile modal
pub fn player_modal(ui: &mut Ui, v: &mut View, c: &mut Career, p: usize) {
    let r = Rect::new(breite() / 2.0 - 380.0, 110.0, 760.0, 640.0);
    rrect(r.x, r.y, r.w, r.h, 10.0, Theme::ueberlagerung());
    let pl = c.world.players[p].clone();
    let club = if pl.team >= 0 { pl.team as usize } else { c.own };
    let (c1, c2) = club_color(c, club);
    rrect(r.x, r.y, r.w, 130.0, 10.0, c1);
    draw_rectangle(r.x, r.y + 100.0, r.w, 30.0, alpha(Theme::ueberlagerung(), 0.9));
    ui.player_photo(r.x + 80.0, r.y + 72.0, 48.0, pl.id, &pl.name);
    ui.title(&pl.name, r.x + 150.0, r.y + 52.0, 34.0, c2);
    let clubname = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else if pl.team == -1 { crate::inhalt::t("ui.wort.vereinslos").into() } else { crate::inhalt::t("ui.pages.karriereende").into() };
    ui.text_semi(&crate::tf!("ui.pages.jahre", pos_de(&pl.pos), pl.age, clubname), r.x + 150.0, r.y + 84.0, 18.0, c2);
    let (lo, hi) = c.ovr_range(p);
    let (pa, pb) = c.pot_range(p);
    let mut y = r.y + 170.0;
    let items = [
        (crate::inhalt::t("ui.pages.staerke"), if hi - lo <= 1.0 { format!("{:.0}", pl.ovr) } else { format!("{lo}–{hi}") }),
        (crate::id("POTENZIAL"), format!("{pa}–{pb}")),
        (crate::id("MARKTWERT"), money(pl.value)),
        (crate::id("GEHALT"), money(pl.wage)),
        (crate::id("VERTRAG"), crate::tf!("zentrale.bis", date::START_YEAR as u32 + pl.contract_end + 1)),
    ];
    for (i, (k, val)) in items.iter().enumerate() {
        let x = r.x + 24.0 + i as f32 * 146.0;
        ui.text_semi(k, x, y, 13.0, Theme::gedaempft());
        ui.title(val, x, y + 30.0, 24.0, if i == 0 { Theme::skala((lo + hi) / 2.0) } else { Theme::text() });
    }
    y += 70.0;
    let own = pl.team == c.own as i32;
    let unser = c.gehoert_uns(p);
    if unser || c.know_of(p) >= 0.6 {
        for (i, (k, val, col)) in [(crate::inhalt::t("uebersicht.fitness"), pl.fit, Theme::gut()), (crate::inhalt::t("kader.moral"), pl.morale, Theme::akzent()), (crate::inhalt::t("zentrale.k.form"), (pl.form + 6.0) / 12.0 * 100.0, Theme::achtung())].iter().enumerate() {
            let yy = y + i as f32 * 30.0;
            ui.text(k, r.x + 24.0, yy + 12.0, 16.0, Theme::text2());
            ui.bar(r.x + 110.0, yy + 2.0, 260.0, 8.0, val / 100.0, *col);
        }
        if pl.ps.an {
            let ty = crate::mensch::typen(&pl);
            let typ = ty.first().map(|t| crate::mensch::typ_name(t)).unwrap_or(crate::inhalt::t("gs.kontext.form.mittel"));
            ui.text(&crate::tf!("profil.kurz_text", typ, crate::mensch::rolle_name(pl.ps.rolle)), r.x + 400.0, y + 12.0, 16.0, Theme::text());
        } else {
            let pers = [crate::inhalt::t("ui.pages.sachlich_und_fair"), crate::inhalt::t("ui.pages.hat_den_blick_fuers_geld"), crate::inhalt::t("ui.pages.treu_und_loyal"), crate::inhalt::t("ui.pages.sehr_ehrgeizig"), crate::id("heimatverbunden")][pl.persona as usize];
            ui.text(&crate::tf!("pm.zeile.persoenlichkeit", pers), r.x + 400.0, y + 12.0, 16.0, Theme::text());
        }
        ui.text(&crate::tf!("ui.pages.ins", ideas_t()[c.tactic.idea()], fit_text(c.system_fit(p))), r.x + 400.0, y + 42.0, 16.0, Theme::text());
        if pl.inj > 0 { ui.text(&crate::tf!("ui.pages.verletzt_tage", pl.inj_name, pl.inj), r.x + 400.0, y + 72.0, 16.0, Theme::problem()); }
    } else {
        // what the knowledge shows (scouting.toml sterne.stufen): stars, then 30 % / 60 % / all abilities, character, traits
        let st = c.sicht(p);
        let (lo2, hi2) = c.ovr_range(p);
        let mut zeilen = vec![];
        if st >= 1 { zeilen.push(crate::tf!("ui.pages.einschaetzung_bis", crate::scouting::sterne_text(crate::scouting::sterne(lo2)), crate::scouting::sterne_text(crate::scouting::sterne(hi2)))); }
        if st >= 2 && pl.attrs.len() == 25 {
            let anteil = crate::mensch::arr("scouting", "sterne.einzelwerte_anteil")[st].as_f64().unwrap();
            let n = (25.0 * anteil).round() as usize;
            let mut idx: Vec<usize> = (0..25).collect();
            idx.sort_by(|a, b| pl.attrs[*b].partial_cmp(&pl.attrs[*a]).unwrap());
            let w: Vec<String> = idx.iter().take(n.min(6)).map(|&k| format!("{} {:.0}", crate::pesdb::attr_de_t()[k], pl.attrs[k])).collect();
            zeilen.push(crate::tf!("ui.pages.bekannt_von_25_werten", n, w.join(", ")));
        }
        if st >= 3 && pl.ps.an { if let Some(t) = crate::mensch::typen(&pl).first() { zeilen.push(crate::tf!("pm.zeile.persoenlichkeit", crate::mensch::typ_name(t))); } }
        if st >= 5 { zeilen.push(crate::tf!("ui.pages.verletzungsanfaellig", if pl.inj > 0 || pl.age >= 31 { crate::inhalt::t("ui.pages.eher_ja") } else { crate::inhalt::t("ui.wort.nein") })); }
        if zeilen.is_empty() { zeilen.push(crate::inhalt::t("ui.pages.ueber_ihn_weisst_du_noch").into()); }
        for (k, z) in zeilen.iter().enumerate() { ui.text(&ui.fit(z, 16.0, r.w - 48.0), r.x + 24.0, y + 12.0 + k as f32 * 24.0, 16.0, Theme::text2()); }
    }
    y += 100.0;
    if own && pl.ps.an {
        let mut tr = pl.ps.treiber.clone();
        tr.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        let words: Vec<String> = tr.iter().take(4).filter(|x| x.1.abs() >= 0.5).map(|x| format!("{} {}", crate::mensch::treiber_name(&x.0), if x.1 > 0.0 { crate::id("hebt") } else { crate::inhalt::t("ui.pages.drueckt") })).collect();
        let mut st: Vec<(usize, f32)> = (0..9).map(|i| (i, crate::mensch::cw(&pl, i))).collect();
        st.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let stark = crate::tf!("ui.pages.ausgepraegt", crate::mensch::wert_name(st[0].0), crate::mensch::wert_name(st[1].0));
        ui.text(&if words.is_empty() { crate::inhalt::t("ui.pages.nichts_drueckt_auf_seine_stimmung").to_string() } else { crate::tf!("ui.pages.was_ihn_bewegt", words.join(", ")) }, r.x + 24.0, y, 15.0, Theme::text2());
        ui.text(&crate::tf!("ui.pages.vertrauen_in_dich", stark, match c.vertrauen(p) as i32 { 0..=29 => "gering", 30..=59 => "mittel", _ => crate::inhalt::t("r.ui.demo.gross") }), r.x + 24.0, y + 22.0, 15.0, Theme::text2());
    }
    y += 44.0;
    ui.text_semi(crate::inhalt::t("ui.wort.gross_saison"), r.x + 24.0, y, 14.0, Theme::gedaempft());
    ui.text(&crate::tf!("ui.pages.spiele_tore_vorlagen_note", pl.apps, pl.goals, pl.assists, if pl.rn > 0 { format!("{:.2}", pl.avg_rating()) } else { "–".into() }), r.x + 24.0, y + 26.0, 18.0, Theme::text());
    for (j, rt) in pl.last.iter().enumerate() { ui.note_pill(r.x + 50.0 + j as f32 * 52.0, y + 56.0, *rt); }
    y += 90.0;
    if c.know_of(p) < 1.0 && !own {
        let rep = c.report_text(p);
        ui.paragraph(&rep, r.x + 24.0, y, 15.0, r.w - 48.0, Theme::text2());
    }
    let by = r.y + r.h - 60.0;
    if own {
        if ui.button(Rect::new(r.x + 24.0, by, 180.0, 40.0), crate::inhalt::t("aktion.verlaengern"), false) { match c.open_neg(p) { Ok(i) => { v.modal = Modal::Neg(i); return; } Err(e) => v.toast(&e, false) } }
        if ui.button(Rect::new(r.x + 216.0, by, 180.0, 40.0), if pl.listed { crate::inhalt::t("kue.btn.liste_aus") } else { crate::inhalt::t("ui.pages.auf_transferliste") }, pl.listed) { c.world.players[p].listed = !pl.listed; }
        ui.text(&crate::tf!("liga.nummer", pl.no), r.x + 610.0, by - 26.0, 15.0, Theme::text2());
        if ui.button(Rect::new(r.x + 660.0, by - 44.0, 26.0, 26.0), "−", false) { c.rueckennummer(p, -1); }
        if ui.button(Rect::new(r.x + 690.0, by - 44.0, 26.0, 26.0), "+", false) { c.rueckennummer(p, 1); }
        if pl.leihe.as_ref().map_or(false, |l| l.von != c.own && l.option > 0.0) {
            if ui.button_gold(Rect::new(r.x + 24.0, by - 50.0, 260.0, 40.0), &crate::tf!("ui.pages.kaufoption", money(pl.leihe.as_ref().unwrap().option))) { match c.kaufoption(p) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.kaufoption_gezogen"), true), Err(e) => v.toast(&e, false) } }
        } else if pl.leihe.is_none() && ui.button(Rect::new(r.x + 24.0, by - 50.0, 180.0, 40.0), crate::inhalt::t("aktion.verleihen"), false) { match c.leihe_abgeben(p) { Ok(()) => { v.toast(crate::inhalt::t("ui.pages.verliehen_bis_saisonende"), true); v.modal = Modal::None; return; } Err(e) => v.toast(&e, false) } }
        if let Some((pos, bis)) = &pl.ps.umschulung { ui.text(&crate::tf!("ui.pages.lernt_noch_tage", pos_de(pos), bis.saturating_sub(c.abs())), r.x + 300.0, by - 26.0, 15.0, Theme::text2()); }
        else {
            ui.text(crate::inhalt::t("ui.pages.umschulen"), r.x + 300.0, by - 26.0, 14.0, Theme::gedaempft());
            for (k, pos) in [crate::id("GK"), crate::id("CB"), crate::id("LB"), crate::id("RB"), crate::id("DMF"), crate::id("CMF"), crate::id("AMF"), crate::id("LMF"), crate::id("RMF"), crate::id("LWF"), crate::id("RWF"), crate::id("SS"), crate::id("CF")].iter().enumerate() {
                if *pos == pl.pos { continue; }
                if ui.button(Rect::new(r.x + 380.0 + k as f32 * 28.0, by - 44.0, 26.0, 26.0), &pos_de(pos)[..pos_de(pos).char_indices().nth(2).map(|x| x.0).unwrap_or(pos_de(pos).len())], false) { match c.umschulen(p, pos) { Ok(t) => v.toast(&crate::tf!("ui.pages.umschulung_dauert_tage", t), true), Err(e) => v.toast(&e, false) } }
            }
        }
        if ui.button(Rect::new(r.x + 408.0, by, 160.0, 40.0), crate::inhalt::t("aka.entlassen.ja"), false) { if c.squad().len() > 18 { c.release(p); v.toast(crate::inhalt::t("ui.pages.vertrag_aufgeloest"), true); v.modal = Modal::None; return; } else { v.toast(crate::inhalt::t("ui.pages.der_kader_ist_zu_klein"), false); } }
    } else if pl.team != -2 && !unser {
        if ui.button_gold(Rect::new(r.x + 24.0, by, 200.0, 40.0), crate::inhalt::t("ui.wort.gross_verhandeln")) { match c.open_neg(p) { Ok(i) => { v.modal = Modal::Neg(i); return; } Err(e) => v.toast(&e, false) } }
        if pl.team >= 0 && ui.button(Rect::new(r.x + 236.0, by, 160.0, 40.0), crate::inhalt::t("cr.leihen"), false) { match c.leihe_holen(p) { Ok(()) => { v.toast(crate::inhalt::t("ui.pages.leihe_perfekt"), true); v.modal = Modal::None; return; } Err(e) => v.toast(&e, false) } }
    }
    if ui.button(Rect::new(r.x + r.w - 160.0, by, 136.0, 40.0), crate::inhalt::t("aktion.schliessen"), false) || is_key_pressed(KeyCode::Escape) { v.modal = Modal::None; }
    if ui.button(Rect::new(r.x + r.w - 160.0, r.y + 12.0, 136.0, 30.0), crate::inhalt::t("ui.pages.ganze_seite"), false) { v.sel_player = Some(p); v.modal = Modal::None; v.go(Page::Spielerprofil); }
}

fn stadion(ui: &mut Ui, v: &mut View, c: &mut Career) { v.tab = 3; finanzen(ui, v, c); }
fn marken(ui: &mut Ui, v: &mut View, c: &mut Career) { v.tab = 2; finanzen(ui, v, c); }
fn fans_seite(ui: &mut Ui, v: &mut View, c: &mut Career) { v.tab = 4; finanzen(ui, v, c); }
/// Start ending FL27 in its own thread (SIGTERM waits up to 10 s; the interface keeps running).
pub fn fl27_beenden_starten(v: &mut View) {
    if v.fl27_ende.is_some() { return; }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || { let _ = tx.send(crate::fl27::fl27_beenden()); });
    v.fl27_ende = Some(rx);
    v.fl27_ende_wunsch = false;
    v.toast(crate::inhalt::t("ui.pages.fl27_wird_beendet"), true);
}
/// "FL27 beenden": with the reader running the result is saved first (match day page), else at once.
pub fn fl27_beenden_klick(v: &mut View) {
    if v.fl27.is_some() { v.fl27_ende_wunsch = true; if v.page != Page::Spieltag { v.go(Page::Spieltag); } } else { fl27_beenden_starten(v); }
}
/// Every frame: is FL27 running, live writing of the pesdb, the ending's answer, the automatic end.
pub fn fl27_und_db(v: &mut View, c: &mut Career) {
    let t = get_time() as f32;
    if t - v.fl27_pruef_t > 2.0 {
        v.fl27_pruef_t = t;
        v.fl27_laeuft = crate::fl27::fl27_pid().is_some();
        if v.fl27_laeuft { v.fl27_gesehen = true; }
        // FL27 is gone after a match (ended, closed or crashed), or PES M starts and old presets are still there: undo them.
        // Between writing the presets and FL27 appearing nothing happens (it hasn't been seen yet).
        else if (v.fl27_gesehen || !v.fl27_erster_blick) && v.fl27_ende.is_none() && v.fl27.is_none() {
            v.fl27_gesehen = false;
            if let Some(s) = c.vorgaben_aufraeumen() { v.toast(&s, true); }
        }
        v.fl27_erster_blick = true;
    }
    // nothing is written into the game folder while FL27 is being ended
    if v.fl27_ende.is_none() {
        let jetzt = std::time::Instant::now();
        if t - v.db_blick_t > 1.0 { v.db_blick_t = t; c.db_beobachten(jetzt); }
        match c.db_tick(jetzt) { Some(Ok(s)) if !s.is_empty() => v.toast(&s, true), Some(Err(e)) => v.toast(&crate::tf!("ui.pages.datenbank_nicht_geschrieben", e), false), _ => {} }
    }
    if v.fl27_ende_wunsch && v.fl27.is_none() { fl27_beenden_starten(v); }
    if let Some(Ok(r)) = v.fl27_ende.as_ref().map(|rx| rx.try_recv()) {
        v.fl27_ende = None;
        v.fl27_pruef_t = -10.0;
        match r {
            Ok(s) => { let e = v.fl27_uebernommen.take().map(|x| crate::tf!("ui.pages.uebernommen", x)).unwrap_or_else(|| crate::inhalt::t("ui.pages.kein_ergebnis_uebernommen_bitte_von").into()); v.toast(&format!("{s}{e}"), true); }
            Err(e) => v.toast(&e, false),
        }
    }
    if let Some(a) = v.fl27_auto_t { if t >= a && v.fl27.is_none() { v.fl27_auto_t = None; if v.fl27_laeuft { fl27_beenden_starten(v); } } }
}
/// The player a message is from or about (talks, contracts, injuries, offers), for its photo.
fn msg_spieler(c: &Career, m: &Msg) -> Option<usize> {
    let p = match &m.action {
        Action::TransferOffer { pid, .. } | Action::Complaint { pid } | Action::Expiring { pid } | Action::Report { pid } | Action::Graduate { pid } => Some(*pid),
        Action::Wahl { bezug, .. } => bezug.first().copied(),
        _ => c.squad().into_iter().find(|&p| { let n = &c.world.players[p].name; !n.is_empty() && (m.title.contains(n.as_str()) || &m.from == n) }),
    };
    p.filter(|&p| p < c.world.players.len())
}
/// Transfers › Merkliste: the players put on the watch list with "Beobachten" (profile).
fn merkliste(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(X0 + 4.0, Y0 - 40.0, cbreite(), hoehe() - Y0 + 24.0);
    ui.card_titled(area, &crate::tf!("ui.pages.merkliste_spieler", c.merkliste.len()), Theme::akzent());
    if c.merkliste.is_empty() { ui.paragraph(crate::inhalt::t("ui.pages.noch_niemand_auf_der_merkliste"), area.x + 14.0, area.y + 60.0, 16.0, area.w - 28.0, Theme::text2()); return; }
    let mut weg = None;
    for (k, &p) in c.merkliste.clone().iter().enumerate() {
        let y = area.y + 50.0 + k as f32 * 44.0;
        if y > area.y + area.h - 44.0 { break; }
        let r = Rect::new(area.x + 8.0, y, area.w - 16.0, 40.0);
        if k % 2 == 1 { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::erhoehung(), 0.6)); }
        let pl = &c.world.players[p];
        ui.pos_tag(r.x + 8.0, r.y + 20.0, pos_de(&pl.pos));
        ui.player_photo(r.x + 62.0, r.y + 20.0, 15.0, pl.id, &pl.name);
        ui.text_semi(&ui.fit(&pl.name, 17.0, 260.0), r.x + 84.0, r.y + 26.0, 17.0, Theme::text());
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
        ui.text(&crate::tf!("as.ent.talent", pl.age, ui.fit(&club, 15.0, 240.0)), r.x + 360.0, r.y + 26.0, 15.0, Theme::text2());
        let (lo, hi) = c.ovr_range(p);
        ui.text_semi(&if hi - lo <= 1.0 { format!("{lo:.0}") } else { format!("{lo:.0}–{hi:.0}") }, r.x + 720.0, r.y + 26.0, 16.0, Theme::skala((lo + hi) / 2.0));
        ui.text(&money(pl.value), r.x + 800.0, r.y + 26.0, 15.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 220.0, r.y + 6.0, 100.0, 28.0), crate::inhalt::t("aktion.profil"), false) { v.sel_player = Some(p); v.go(Page::Spielerprofil); return; }
        if ui.button(Rect::new(r.x + r.w - 110.0, r.y + 6.0, 100.0, 28.0), crate::inhalt::t("karrieren.entfernen.ja"), false) { weg = Some(p); }
    }
    if let Some(p) = weg { c.merkliste.retain(|&x| x != p); }
}
include!("seiten.rs");
include!("seiten_farm.rs");
include!("postkarten.rs");

#[cfg(test)]
mod scroll_t {
    /// A squad of 40 in the portrait grid (4 a row, 3 rows fit) and a bench of 12 (8 fit): scrolling shows every one.
    #[test]
    fn nach_dem_scrollen_sind_alle_zu_sehen() {
        for (total, vis) in [(10usize, 3usize), (12, 8), (40, 12), (5, 9), (0, 4)] {
            let mut gesehen = vec![false; total];
            for s in 0..=total { for i in super::fenster(total, vis, s) { gesehen[i] = true; } }
            assert!(gesehen.iter().all(|&g| g), "{total} Einträge, {vis} sichtbar");
            assert!(super::fenster(total, vis, 999).end == total, "ganz unten ist der letzte Eintrag");
        }
    }
}

// ================================================================ Suche als Overlay und Lesezeichen (Punkt 6)
/// Every page reachable from the menu, with its name (for the search and the bookmarks).
pub fn alle_seiten(c: &Career) -> Vec<(Page, String)> {
    let mut v: Vec<(Page, String)> = vec![];
    for (_, haupt) in MENU.iter() {
        for h in haupt.iter() { for (p, n) in reiter(*h, c) { if !v.iter().any(|x| x.0 == p) { v.push((p, n.to_string())); } } }
    }
    v
}
/// The search over the page: players, clubs and pages whose name holds the text; Enter opens the first, Esc closes.
/// Empty, it shows the bookmarks.
pub fn suche_overlay(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let r = Rect::new(breite() / 2.0 - 300.0, 70.0, 600.0, 520.0);
    ui.card(r);
    for ch in ui.typed.clone() { v.suche_text.push(ch); }
    if ui.key_back { v.suche_text.pop(); }
    let feld = Rect::new(r.x + 16.0, r.y + 16.0, r.w - 32.0, 44.0);
    rrect(feld.x, feld.y, feld.w, feld.h, 22.0, Theme::erhoehung());
    let cursor = if (crate::ui::jetzt() * 2.0) as i64 % 2 == 0 { "|" } else { "" };
    let leer = v.suche_text.is_empty();
    ui.text(&if leer { crate::inhalt::t("ui.suche.hinweis").to_string() } else { format!("{}{cursor}", v.suche_text) }, feld.x + 20.0, feld.y + 29.0, 18.0, if leer { Theme::gedaempft() } else { Theme::text() });
    let seiten = alle_seiten(c);
    if v.suche_fuer != v.suche_text {
        v.suche_fuer = v.suche_text.clone();
        let q = v.suche_text.to_lowercase();
        let mut t: Vec<(u8, usize)> = vec![];
        if q.chars().count() >= 2 {
            t.extend(seiten.iter().enumerate().filter(|(_, x)| x.1.to_lowercase().contains(&q)).map(|(i, _)| (2u8, i)).take(4));
            t.extend(c.world.clubs.iter().enumerate().filter(|(_, x)| x.name.to_lowercase().contains(&q)).map(|(i, _)| (1u8, i)).take(5));
            let mut sp: Vec<usize> = c.world.players.iter().enumerate().filter(|(_, x)| x.team >= -1 && x.name.to_lowercase().contains(&q)).map(|(i, _)| i).collect();
            sp.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
            t.extend(sp.into_iter().take(8).map(|i| (0u8, i)));
        }
        v.suche_treffer = t;
    }
    let mut oeffnen: Option<(u8, usize)> = None;
    let mut y = r.y + 76.0;
    if leer {
        ui.text_semi(crate::inhalt::t("ui.suche.lesezeichen"), r.x + 20.0, y + 6.0, 14.0, Theme::gedaempft());
        y += 16.0;
        let marken: Vec<usize> = c.lesezeichen.iter().filter_map(|m| seiten.iter().position(|s| format!("{:?}", s.0) == *m)).collect();
        if marken.is_empty() { ui.paragraph(crate::inhalt::t("ui.suche.keine_lesezeichen"), r.x + 20.0, y + 20.0, 15.0, r.w - 40.0, Theme::text2()); }
        for i in marken { let z = Rect::new(r.x + 12.0, y, r.w - 24.0, 34.0); if zeile_suche(ui, z, &seiten[i].1, crate::inhalt::t("ui.suche.seite")) { oeffnen = Some((2, i)); } y += 38.0; }
    } else if v.suche_treffer.is_empty() {
        ui.text(crate::inhalt::t("ui.suche.nichts"), r.x + 20.0, y + 20.0, 15.0, Theme::text2());
    }
    for &(art, i) in v.suche_treffer.clone().iter() {
        if y + 38.0 > r.y + r.h - 8.0 { break; }
        let z = Rect::new(r.x + 12.0, y, r.w - 24.0, 34.0);
        let (name, wo) = match art {
            0 => { let pl = &c.world.players[i]; (pl.name.clone(), if pl.team >= 0 { format!("{} · {}", crate::world::pos_de(&pl.pos), c.world.clubs[pl.team as usize].name) } else { crate::inhalt::t("sc.profil.vereinslos").to_string() }) }
            1 => (c.world.clubs[i].name.clone(), crate::inhalt::t("ui.suche.verein").to_string()),
            _ => (seiten[i].1.clone(), crate::inhalt::t("ui.suche.seite").to_string()),
        };
        if zeile_suche(ui, z, &name, &wo) { oeffnen = Some((art, i)); }
        y += 38.0;
    }
    if ui.key_enter { oeffnen = oeffnen.or(v.suche_treffer.first().copied()); }
    if let Some((art, i)) = oeffnen {
        v.modal = Modal::None;
        v.suche_text.clear();
        match art { 0 => v.modal = Modal::Player(i), 1 => { v.sel_club = Some(i); v.go(Page::Vereine); } _ => v.go(seiten[i].0) }
        return;
    }
    if is_key_pressed(KeyCode::Escape) || (ui.clicked && !r.contains(ui.mouse)) { v.modal = Modal::None; v.suche_text.clear(); }
}
fn zeile_suche(ui: &mut Ui, z: Rect, name: &str, wo: &str) -> bool {
    rrect(z.x, z.y, z.w, z.h, 8.0, if ui.hover(z) { Theme::ueberlagerung() } else { Theme::flaeche() });
    ui.text_semi(&ui.fit(name, 16.0, z.w * 0.55), z.x + 12.0, z.y + 23.0, 16.0, Theme::text());
    ui.text_right(&ui.fit(wo, 13.0, z.w * 0.4), z.x + z.w - 12.0, z.y + 22.0, 13.0, Theme::text2());
    ui.hit(z)
}

/// The right-click menu of a player (Punkt 6): profile, watch list, negotiate (or renew for an own player).
pub fn kontext_menue(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let Some((p, pos)) = v.kontext else { return };
    let eigen = c.world.players[p].team == c.own as i32;
    let gemerkt = c.merkliste.contains(&p);
    let t = crate::inhalt::t;
    let punkte: Vec<(u8, &str)> = vec![(0, t("ui.kontext.profil")), (1, if gemerkt { t("ui.kontext.merkliste_weg") } else { t("ui.kontext.merkliste") }), (2, if eigen { t("ui.kontext.verlaengern") } else { t("ui.kontext.verhandeln") })];
    let w = 230.0;
    let r = Rect::new(pos.x.min(breite() - w - 8.0), pos.y.min(hoehe() - 40.0 * punkte.len() as f32 - 50.0), w, 40.0 + 36.0 * punkte.len() as f32);
    ui.card(r);
    let pl = &c.world.players[p];
    ui.text_semi(&ui.fit(&pl.name, 15.0, w - 24.0), r.x + 12.0, r.y + 24.0, 15.0, Theme::text());
    let mut wahl = None;
    for (k, (id, txt)) in punkte.iter().enumerate() {
        let z = Rect::new(r.x + 6.0, r.y + 36.0 + k as f32 * 36.0, w - 12.0, 32.0);
        rrect(z.x, z.y, z.w, z.h, 6.0, if ui.hover(z) { Theme::ueberlagerung() } else { Theme::flaeche() });
        ui.text(txt, z.x + 10.0, z.y + 21.0, 15.0, Theme::text());
        if ui.hit(z) { wahl = Some(*id); }
    }
    match wahl {
        Some(0) => { v.kontext = None; v.modal = Modal::Player(p); }
        Some(1) => { v.kontext = None; if gemerkt { c.merkliste.retain(|&x| x != p); } else { c.merkliste.push(p); } v.toast(if gemerkt { t("ui.kontext.ist_weg") } else { t("ui.post.auf_merkliste") }, true); }
        Some(_) => { v.kontext = None; match c.open_neg(p) { Ok(k) => v.modal = Modal::Neg(k), Err(e) => v.toast(&e, false) } }
        None => if (ui.clicked && !r.contains(ui.mouse)) || is_key_pressed(KeyCode::Escape) { v.kontext = None; },
    }
}

// ================================================================ Vorbereitung (Punkt 14)
/// Friendlies, trips with tournaments and team events: plan them, see them, enter the score of one played in FL27.
fn vorbereitung_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let (bw, bh) = ((cbreite() - 12.0) / 2.0, (hoehe() - Y0 - 28.0) / 2.0);
    let felder = [Rect::new(X0 + 4.0, Y0, bw, bh), Rect::new(X0 + 16.0 + bw, Y0, bw, bh), Rect::new(X0 + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(X0 + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
    // dates: friendlies with simulate / play in FL27 (default simulate), trips; results
    let r = felder[0];
    ui.card_titled(r, t("ui.vorb.termine"), Theme::akzent());
    let mut ix: Vec<usize> = (0..c.termine.len()).collect();
    ix.sort_by_key(|&i| (c.termine[i].ergebnis.is_some(), c.termine[i].tag));
    if ix.is_empty() { ui.paragraph(t("ui.vorb.keine_termine"), r.x + 14.0, r.y + 64.0, 15.0, r.w - 28.0, Theme::text2()); }
    let mut y = r.y + 50.0;
    for i in ix {
        if y + 34.0 > r.y + r.h - 6.0 { break; }
        let tm = c.termine[i].clone();
        let was = if tm.art == 2 { crate::vorbereitung::lager_orte().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["name"].as_str()).unwrap_or("").to_string() } else if tm.art == 0 { crate::tf!("ui.vorb.testspiel_gegen", c.world.clubs[tm.gegner].name) } else { crate::vorbereitung::turniere().iter().find(|x| x["id"].as_str() == Some(tm.turnier.as_str())).and_then(|x| x["name"].as_str()).unwrap_or("").to_string() };
        ui.text(&date::short(c.world.season, tm.tag), r.x + 14.0, y + 20.0, 14.0, Theme::text2());
        ui.text(&ui.fit(&was, 15.0, r.w * 0.42), r.x + 90.0, y + 20.0, 15.0, Theme::text());
        let rx = r.x + r.w * 0.6;
        if let Some(_) = tm.ergebnis { ui.text(&ui.fit(&tm.text, 13.0, r.w * 0.38), rx, y + 20.0, 13.0, Theme::text2()); }
        else if tm.art == 0 && tm.selbst && tm.tag <= c.world.day {
            // played in FL27: the score by hand
            let (a, b) = v.vorb_tore;
            if ui.button(Rect::new(rx, y + 2.0, 26.0, 26.0), "+", false) { v.vorb_tore.0 = (a + 1).min(9); }
            ui.text_center(&format!("{a}:{b}"), rx + 46.0, y + 20.0, 16.0, Theme::text());
            if ui.button(Rect::new(rx + 66.0, y + 2.0, 26.0, 26.0), "+", false) { v.vorb_tore.1 = (b + 1).min(9); }
            if ui.button(Rect::new(rx + 98.0, y + 2.0, r.x + r.w - rx - 112.0, 26.0), t("ui.vorb.eintragen"), true) { c.testspiel_eintragen(i, a, b); v.vorb_tore = (0, 0); }
        } else if tm.art == 0 {
            if let Some(k) = ui.segment(Rect::new(rx, y + 2.0, r.x + r.w - rx - 14.0, 26.0), &[t("ui.vorb.simulieren"), t("ui.vorb.selbst")], tm.selbst as usize) { c.termine[i].selbst = k == 1; }
        } else { ui.text(t("ui.vorb.geplant"), rx, y + 20.0, 13.0, Theme::gedaempft()); }
        y += 34.0;
    }
    // ask for a friendly: a free day, then one of three opponents
    let r = felder[1];
    ui.card_titled(r, t("ui.vorb.anfragen"), Theme::gut());
    let tage = c.freie_tage(5, 1);
    if tage.is_empty() { ui.text(t("ui.vorb.kein_tag"), r.x + 14.0, r.y + 64.0, 15.0, Theme::text2()); } else {
        v.vorb_tag = v.vorb_tag.min(tage.len() - 1);
        let namen: Vec<String> = tage.iter().map(|d| date::short(c.world.season, *d)).collect();
        let nr: Vec<&str> = namen.iter().map(|s| s.as_str()).collect();
        if let Some(k) = ui.segment(Rect::new(r.x + 14.0, r.y + 48.0, r.w - 28.0, 30.0), &nr, v.vorb_tag) { v.vorb_tag = k; }
        ui.text(t("ui.vorb.abstand"), r.x + 14.0, r.y + 100.0, 13.0, Theme::gedaempft());
        for (k, g) in c.testspiel_gegner().into_iter().enumerate() {
            let b = Rect::new(r.x + 14.0, r.y + 114.0 + k as f32 * 42.0, r.w - 28.0, 36.0);
            let label = crate::tf!("ui.vorb.gegner_zeile", c.world.clubs[g].name, stufe_wort(crate::mensch::kaderstaerke(&c.world, g)));
            if ui.button(b, &label, false) { match c.testspiel_anfragen(g, tage[v.vorb_tag]) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
        }
    }
    // trips with a small tournament (turniere.toml: real and invented names)
    let r = felder[2];
    ui.card_titled(r, t("ui.vorb.reisen"), Theme::gold());
    if let Some(k) = ui.segment(Rect::new(r.x + r.w - 330.0, r.y + 6.0, 316.0, 26.0), &[t("ui.vorb.reisen"), t("ui.lager.titel")], v.lager_reiter as usize) { v.lager_reiter = k as u8; }
    if v.lager_reiter == 1 { lager_karte(ui, v, c, r); } else {
    let mut y = r.y + 50.0;
    for tu in crate::vorbereitung::turniere() {
        if y + 40.0 > r.y + r.h - 6.0 { break; }
        let id = tu["id"].as_str().unwrap_or("");
        let tage = tu["tage"].as_u64().unwrap_or(5) as u32;
        let kennz = if tu["echt"].as_bool().unwrap_or(false) { t("ui.vorb.echt") } else { t("ui.vorb.erfunden") };
        ui.text(&ui.fit(tu["name"].as_str().unwrap_or(id), 15.0, r.w * 0.55), r.x + 14.0, y + 16.0, 15.0, Theme::text());
        ui.text(&ui.fit(&crate::tf!("ui.vorb.reise_zeile", kennz, tu["region"].as_str().unwrap_or(""), tage, tu["teams"].as_u64().unwrap_or(4), money(tu["kosten"].as_f64().unwrap_or(0.0))), 12.0, r.w * 0.62), r.x + 14.0, y + 33.0, 12.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 124.0, y + 4.0, 110.0, 30.0), t("ui.vorb.buchen"), false) {
            match c.freie_tage(1, tage).first() { Some(&d) => match c.reise_buchen(id, d) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }, None => v.toast(t("ui.vorb.kein_tag"), false) }
        }
        y += 42.0;
    }
    }
    // team events
    let r = felder[3];
    ui.card_titled(r, t("ui.vorb.teamevents"), hex(0xa78bfa));
    ui.text(&crate::tf!("ui.vorb.summand", format!("{:+.1}", c.event_summand())), r.x + 14.0, r.y + 58.0, 13.0, Theme::gedaempft());
    let abs = c.abs();
    let mut y = r.y + 70.0;
    for e in crate::vorbereitung::events() {
        if y + 40.0 > r.y + r.h - 6.0 { break; }
        let id = e["id"].as_str().unwrap_or("");
        let frei = c.teamevents.iter().find(|x| x.0 == id).map(|x| x.1 + e["sperre"].as_u64().unwrap_or(30) as u32).unwrap_or(0);
        ui.text(t(&format!("ui.vorb.event.{id}")), r.x + 14.0, y + 16.0, 15.0, Theme::text());
        let zeile = if abs < frei { crate::tf!("ui.vorb.event_gesperrt", frei - abs) } else { crate::tf!("ui.vorb.event_zeile", money(e["kosten"].as_f64().unwrap_or(0.0)), format!("{:.0}", e["teamgeist"].as_f64().unwrap_or(0.0))) };
        ui.text(&ui.fit(&zeile, 12.0, r.w * 0.6), r.x + 14.0, y + 33.0, 12.0, Theme::text2());
        if abs >= frei && ui.button(Rect::new(r.x + r.w - 124.0, y + 4.0, 110.0, 30.0), t("ui.vorb.ausrichten"), false) {
            match c.teamevent(id) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
        }
        y += 42.0;
    }
}


/// A reward moment full screen (SPIELGEFUEHL.md measure 3): a pulsing gold seal with the player's photo or the crest,
/// the title, one paragraph and "Weiter" (Enter, space or a click). The first frame plays the title jingle.
pub fn feier_overlay(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let Some(f) = c.feiern.first().cloned() else { return };
    if !v.feier_ton { ui.play(if f.art == crate::id("meilenstein") || f.art == crate::id("lizenz") { crate::id("meilenstein") } else { crate::id("titel") }); v.feier_ton = true; }
    let (w, h) = (breite(), hoehe());
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.78));
    let t = get_time() as f32;
    let r = Rect::new(w / 2.0 - 380.0, h / 2.0 - 250.0, 760.0, 500.0);
    rrect(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 20.0, Theme::gold());
    rrect(r.x, r.y, r.w, r.h, 18.0, Theme::flaeche());
    let (cx, cy) = (r.x + r.w / 2.0, r.y + 130.0);
    // rays that turn slowly, a breathing halo, the seal
    for k in 0..16 {
        let a = t * 0.4 + k as f32 * std::f32::consts::TAU / 16.0;
        draw_line(cx + a.cos() * 78.0, cy + a.sin() * 78.0, cx + a.cos() * 104.0, cy + a.sin() * 104.0, 4.0, alpha(Theme::gold(), 0.55));
    }
    let puls = 1.0 + 0.05 * (t * 3.0).sin();
    draw_circle(cx, cy, 74.0 * puls, alpha(Theme::gold(), 0.25));
    draw_circle(cx, cy, 62.0, Theme::gold());
    match (f.spieler, f.club) {
        (Some(p), _) => ui.player_photo(cx, cy, 54.0, c.world.players[p].id, &c.world.players[p].name.clone()),
        (None, Some(cl)) => { let k = &c.world.clubs[cl]; ui.club_crest(cx, cy, 50.0, k.id, hex(k.c1), &k.short.clone()); }
        _ => {}
    }
    let art = match f.art.as_str() { "neuzugang" => crate::inhalt::t("ui.feier.art.neuzugang"), "titel" => crate::inhalt::t("ui.feier.art.titel"), "lizenz" => crate::inhalt::t("ui.feier.art.lizenz"), _ => crate::inhalt::t("ui.meilenstein.titel") };
    ui.text_center(&art.to_uppercase(), cx, r.y + 258.0, 15.0, Theme::gold());
    let size = if ui.measure_title(&f.titel, 40.0) > r.w - 60.0 { 30.0 } else { 40.0 };
    let tw = ui.measure_title(&f.titel, size);
    ui.title(&f.titel, cx - tw / 2.0, r.y + 304.0, size, Theme::text());
    let mut y = r.y + 340.0;
    for z in ui.wrap(&f.text, 17.0, r.w - 100.0).into_iter().take(4) { ui.text_center(&z, cx, y, 17.0, Theme::text2()); y += 24.0; }
    let weiter = ui.button_gold(Rect::new(cx - 120.0, r.y + r.h - 70.0, 240.0, 48.0), crate::inhalt::t("ui.feier.weiter"));
    if weiter || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
        c.feiern.remove(0);
        v.feier_ton = false;
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

/// Training camps (Konzept Teil 4): the places that fit now with costs, the length, booking on the next free days.
fn lager_karte(ui: &mut Ui, v: &mut View, c: &mut Career, r: Rect) {
    use crate::inhalt::t;
    let laengen = [3u32, 5, 7, 10];
    let namen: Vec<String> = laengen.iter().map(|n| crate::tf!("ui.lager.tage", n)).collect();
    let nr: Vec<&str> = namen.iter().map(|s| s.as_str()).collect();
    let sel = laengen.iter().position(|&n| n == v.lager_tage).unwrap_or(2);
    if let Some(k) = ui.segment(Rect::new(r.x + 14.0, r.y + 44.0, r.w - 28.0, 28.0), &nr, sel) { v.lager_tage = laengen[k]; }
    let tage = laengen[sel];
    let mut y = r.y + 82.0;
    let orte = c.lager_passend();
    if orte.is_empty() { ui.text(t("ui.lager.keine"), r.x + 14.0, y + 16.0, 15.0, Theme::text2()); }
    for o in orte {
        if y + 40.0 > r.y + r.h - 6.0 { break; }
        let id = o["id"].as_str().unwrap_or("");
        ui.text(&ui.fit(o["name"].as_str().unwrap_or(id), 15.0, r.w * 0.58), r.x + 14.0, y + 16.0, 15.0, Theme::text());
        let klima = t(&format!("ui.lager.klima.{}", o["klima"].as_str().unwrap_or("mild")));
        let zeile = crate::tf!("ui.lager.zeile", klima, o["plaetze"].as_u64().unwrap_or(3), o["ruhe"].as_u64().unwrap_or(3), money(c.lager_kosten(id, tage)));
        ui.text(&ui.fit(&zeile, 12.0, r.w * 0.62), r.x + 14.0, y + 33.0, 12.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 124.0, y + 4.0, 110.0, 30.0), t("ui.vorb.buchen"), false) {
            match c.freie_tage(1, tage).first() { Some(&d) => match c.lager_buchen(id, d, tage) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }, None => v.toast(t("ui.vorb.kein_tag"), false) }
        }
        y += 42.0;
    }
}
