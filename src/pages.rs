// The manager's pages, drawn into the content area right of the sidebar (layout after the original's screenshots).
use crate::date;
use crate::game::*;
use crate::ui::*;
use crate::world::*;
use macroquad::prelude::*;

/// Left edge of the content: right of the side bar, narrower when the bar is folded in (Fabian 23.11.).
pub fn x0() -> f32 { if SEITENLEISTE_ZU.load(std::sync::atomic::Ordering::Relaxed) { 78.0 } else { 214.0 } }
pub static SEITENLEISTE_ZU: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Content width right of the side bar (follows breite()).
pub fn cbreite() -> f32 { crate::ui::breite() - x0() - 18.0 }
pub const Y0: f32 = 244.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Page {
    Uebersicht, Posteingang, Kalender, Kader, Aufstellung, Kabine, Training, Medizin, Praesidium, Finanzen, Trainer,
    Transfermarkt, Verhandlungen, Vertraege, Scouting, Akademie, Tabellen, Presse, Spieltag, Optionen,
    Ziele, Manager, Stadion, Personalmarkt, Geschichte, Tagebuch, Klima, Marken, Kaderplanung, Vergleich, Entwicklung, Vorbereitung, Analyse, Zeitung, Fans, Ranglisten, Vereine, Vereinswelt, Portal, Handbuch, Regeln, Profi, Spielerprofil, Zentrale, Kooperationen, ZweiteMannschaft, Jugendnetz, Merkliste, Trainerteam, Nationalteams,
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
    ("ui.pages.verein", &[("klima.praesidium", &[(Page::Praesidium, "klima.praesidium"), (Page::Ziele, "aktion.ziele"), (Page::Klima, "ui.main.vereinsklima"), (Page::Geschichte, "menu.vereinsgeschichte.titel")]),
        ("bereich.finanzen", &[(Page::Finanzen, "bereich.finanzen"), (Page::Stadion, "menu.stadion_und_anlagen.titel"), (Page::Marken, "ui.main.marken_und_sponsoren"), (Page::Kooperationen, "ui.farm.kooperationen")]),
        ("md.ui.chip.personal", &[(Page::Trainerteam, "ui.main.trainerteam"), (Page::Personalmarkt, "ui.main.personalmarkt")]),
        ("taktik.board.coach", &[(Page::Trainer, "aktion.trainerprofil"), (Page::Manager, "gm.mg.hero"), (Page::Tagebuch, "gm.tb.hero")])]),
    ("ui.pages.welt", &[("zentrale.wettbewerbe", &[(Page::Tabellen, "ui.pages.tabellen"), (Page::Ranglisten, "ux.abschnitt.rankings"), (Page::Nationalteams, "ui.nt.seite")]),
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

#[derive(Clone)]
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
    /// What the talks brought (shown on the ticker and after the match), the post-match talk given, players addressed.
    pub wirkung: Vec<String>,
    pub nach_gesagt: bool,
    pub angesprochen: Vec<usize>,
    /// Interventions in the running match (Fabian 23.11.): mentality −1/0/1, the minute of the last shout, the own player chosen to come off.
    pub eingriff: Eingriff,
    /// The speed before a pause, to go on with it.
    pub vor_pause: f32,
}
#[derive(Clone, Default)]
pub struct Eingriff { pub mentalitaet: i8, pub zuruf_min: f32, pub raus: Option<usize>, pub intro: f32, pub tor_gesehen: usize, pub tor_t: f32 }

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
    /// Tactics board: the slot being dragged and where the drag started.
    pub tafel_zug: Option<(usize, Vec2)>,
    /// Tactics board: a bench player being dragged and where the drag started (Fabian 23.11.: drag and drop).
    pub bank_zug: Option<(usize, Vec2)>,
    /// Nationalteams › Länder: the chosen nation and the list scroll.
    pub nt_wahl: Option<u16>,
    pub nt_scroll: usize,
    /// The open choice field (country / competition) and its list scroll.
    pub auswahl: Option<usize>,
    pub auswahl_scroll: usize,
    /// Vereine page: the league whose clubs are listed (None = own).
    pub vereine_liga: Option<usize>,
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
    /// Exact position (0 any, 1..=13 PES position + 1) and how well he must play it (0 can play, 1 plays it well).
    pub market_genau: usize,
    pub market_gut: usize,
    /// Free search (Konzept Scouting 5.1): a statistic to filter and sort by (0 none, else MARKT_ZAHLEN index + 1).
    pub market_stat: usize,
    pub market_scope: usize,
    pub market_text: String,
    pub market_focus: bool,
    /// Search profile being edited on the market's profile tab, and its cached hits (suchprofil.rs).
    pub sp_edit: crate::suchprofil::Suchprofil,
    pub sp_treffer: (crate::suchprofil::Suchprofil, u32, Vec<usize>),
    /// The team talk choices before today's match (abs day, goal, tone, focus; prematch.rs).
    pub pm_wahl: (u32, [usize; 3]),
    /// New loan: share of the frame in percent and the term in years (kredit.rs).
    pub kr: (i32, u32),
    /// What is typed while a choice list is open (search in country and competition).
    pub auswahl_suche: String,
    /// all menu groups open at once (button in the side bar, Fabian 23.11.)
    pub menu_alle: bool,
    /// Search overlay (Punkt 6): the text, its hits (0 player, 1 club, 2 page + index), the text they were made for.
    pub suche_text: String,
    /// Preparation page: chosen free day for a friendly, the score being entered for one played in FL27.
    pub vorb_tag: usize,
    pub vorb_tore: (u8, u8),
    pub suche_treffer: Vec<(u8, usize)>,
    pub suche_fuer: String,
    pub market_cache: Vec<usize>,
    /// Sorted lists: per list, the column clicked and whether the order is flipped.
    pub sortierung: std::collections::HashMap<u8, (usize, bool)>,
    /// When the last offer went out (the other side "types" for a moment).
    pub neg_tipp: f64,
    /// Media blog: 0 all, 1 transfers, 2 own club, 3 own league.
    pub blog_filter: u8,
    /// Training plan: the session (day, slot) being filled.
    pub tp_zelle: Option<(usize, usize)>,
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
pub fn skala_file() -> std::path::PathBuf { save_dir().join("../skala.txt") } // roh
pub fn sprache_file() -> std::path::PathBuf { save_dir().join(crate::id("../sprache.txt")) }
impl View {
    pub fn new() -> View {
        View { karte_zoom: 1.0, karte_off: (0.0, 0.0), feier_ton: false, page: Page::Uebersicht, back: vec![], fwd: vec![], modal: Modal::None, tab: 0, scroll: 0.0, scrolls: Default::default(), ziehen: None, tafel_zug: None, bank_zug: None, nt_wahl: None, nt_scroll: 0, auswahl: None, auswahl_scroll: 0, vereine_liga: None, post_folgen: None, sel_msg: None, msg_filter: 0, lager_reiter: 0, lager_tage: 7, halbzeit_fl27: None, sel_player: None,
            market_pos: 0, market_age: 40, market_price: 0, market_genau: 0, market_gut: 0, market_stat: 0, market_scope: 0, market_text: String::new(), market_focus: false, sp_edit: Default::default(), sp_treffer: (Default::default(), u32::MAX, vec![]), pm_wahl: (u32::MAX, [0; 3]), kr: (50, 2), auswahl_suche: String::new(), menu_alle: false, suche_text: String::new(), vorb_tag: 0, vorb_tore: (0, 0), suche_treffer: vec![], suche_fuer: String::new(), market_cache: vec![], market_key: String::new(), sortierung: Default::default(), neg_tipp: 0.0, blog_filter: 0, tp_zelle: None,
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
    let mut x = x0() + 4.0;
    for (i, t) in items.iter().enumerate() {
        let w = ui.measure_semi(t, 19.0) + 20.0;
        let r = Rect::new(x, Y0 - 42.0, w, 32.0);
        let h = ui.hover(r);
        if i == v.tab { rrect(r.x, r.y, r.w, r.h, 12.0, Theme::verein()); rrect(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, 10.0, if Theme::dark() { Theme::verein_tief() } else { Theme::verein_weich() }); } else if h { rrect(r.x, r.y, r.w, r.h, 12.0, Theme::erhoehung()); }
        ui.text_semi(t, x + 10.0, Y0 - 20.0, 19.0, if i == v.tab || h { Theme::text() } else { Theme::text2() });
        if ui.hit(r) && v.tab != i { v.tab = i; v.scroll = 0.0; v.scrolls.clear(); v.changed_at = get_time() as f32; ui.play(crate::id("klick")); }
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

/// A sort key: numbers sort high to low, text A to Z, on the first click.
#[derive(PartialEq, PartialOrd)]
enum Sk { N(f32), S(String) }

/// Header of a sortable list: a click on a column sorts by it, the next click flips the order.
fn sort_head(ui: &mut Ui, v: &mut View, liste: u8, x: f32, y: f32, cols: &[(&str, f32)]) {
    let akt = v.sortierung.get(&liste).copied();
    for (i, (t, cx)) in cols.iter().enumerate() {
        if t.is_empty() { continue; }
        let w = ui.measure_semi(t, 14.0);
        let r = Rect::new(x + cx - 4.0, y - 16.0, w + 22.0, 22.0);
        let an = akt.map(|a| a.0) == Some(i);
        if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::ueberlagerung()); }
        ui.text_semi(t, x + cx, y, 14.0, if an { Theme::akzent() } else { Theme::gedaempft() });
        if an {
            let (ax, ay, auf) = (x + cx + w + 9.0, y - 5.0, akt.map_or(false, |a| a.1));
            if auf { draw_triangle(vec2(ax - 5.0, ay + 3.0), vec2(ax + 5.0, ay + 3.0), vec2(ax, ay - 4.0), Theme::akzent()); }
            else { draw_triangle(vec2(ax - 5.0, ay - 3.0), vec2(ax + 5.0, ay - 3.0), vec2(ax, ay + 4.0), Theme::akzent()); }
        }
        if ui.hit(r) {
            match akt { Some((c, f)) if c == i => { if f { v.sortierung.remove(&liste); } else { v.sortierung.insert(liste, (i, true)); } }
                _ => { v.sortierung.insert(liste, (i, false)); } }
            v.scrolls.clear();
        }
    }
}

/// Sorts a list of players by the column chosen in its header (no choice: the list keeps its order).
fn sortieren(v: &View, liste: u8, sq: &mut Vec<usize>, key: impl Fn(usize, usize) -> Sk) {
    let Some(&(col, flip)) = v.sortierung.get(&liste) else { return };
    let mut k: Vec<(Sk, usize)> = sq.iter().map(|&p| (key(p, col), p)).collect();
    k.sort_by(|a, b| {
        let o = a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal);
        let o = if matches!(a.0, Sk::N(_)) { o.reverse() } else { o };
        if flip { o.reverse() } else { o }
    });
    *sq = k.into_iter().map(|x| x.1).collect();
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
        Page::Nationalteams => nationalteams_seite(ui, v, c),
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
    let r = Rect::new(x0() + 4.0, Y0 - 40.0, lw, 300.0);
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
    let rt = Rect::new(x0() + lw + 16.0, Y0 - 40.0, cbreite() - lw - 12.0, 300.0);
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
    // co-trainer hint
    let rc = Rect::new(x0() + lw + 16.0, y2, cbreite() - lw - 12.0, hoehe() - y2 - 14.0);
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
    let area = Rect::new(x0() + 4.0, Y0, lw, hoehe() - Y0 - 16.0);
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
    let rd = Rect::new(x0() + lw + 18.0, Y0, cbreite() - lw - 14.0, hoehe() - Y0 - 16.0);
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
    if ui.button(Rect::new(x0() + cbreite() - 220.0, Y0 - 44.0, 40.0, 32.0), "<", false) { v.cal_month = if m == 1 { 12 } else { m as i32 - 1 }; }
    if ui.button(Rect::new(x0() + cbreite() - 40.0, Y0 - 44.0, 40.0, 32.0), ">", false) { v.cal_month = if m == 12 { 1 } else { m as i32 + 1 }; }
    let mut first = None;
    for d in 0..date::SEASON_DAYS { let (_, mm, dd) = date::ymd(c.world.season, d); if mm == m && dd == 1 { first = Some(d); break; } }
    let Some(f0) = first else { return };
    let (yy, _, _) = date::ymd(c.world.season, f0);
    ui.title(&format!("{} {}", date::month_name(m), yy), x0() + 10.0, Y0 - 16.0, 28.0, Theme::text());
    let cw = (cbreite() - 6.0 * 8.0) / 7.0;
    let ch = (hoehe() - Y0 - 70.0) / 6.0 - 8.0;
    for (i, wdn) in [crate::inhalt::t("training.tag.0"), crate::inhalt::t("training.tag.1"), crate::inhalt::t("training.tag.2"), crate::inhalt::t("training.tag.3"), crate::inhalt::t("training.tag.4"), crate::inhalt::t("training.tag.5"), crate::inhalt::t("training.tag.6")].iter().enumerate() {
        ui.text_semi(wdn, x0() + 4.0 + i as f32 * (cw + 8.0), Y0 + 12.0, 16.0, Theme::text2());
    }
    let off = date::weekday(f0);
    let fx = c.own_fixtures();
    for d in f0..date::SEASON_DAYS {
        let (_, mm, dd) = date::ymd(c.world.season, d);
        if mm != m { break; }
        let cell = off + (dd - 1) as usize;
        let r = Rect::new(x0() + 4.0 + (cell % 7) as f32 * (cw + 8.0), Y0 + 22.0 + (cell / 7) as f32 * (ch + 8.0), cw, ch);
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
    draw_rectangle(x0() + 8.0, ly - 8.0, 20.0, 4.0, Theme::gold()); ui.text(crate::inhalt::t("termin.transferfenster"), x0() + 34.0, ly, 15.0, Theme::text2());
    draw_rectangle(x0() + 168.0, ly - 8.0, 20.0, 4.0, hex(0xB48CFF)); ui.text(crate::inhalt::t("ui.pages.laenderspielpause"), x0() + 194.0, ly, 15.0, Theme::text2());
    draw_rectangle(x0() + 368.0, ly - 8.0, 20.0, 4.0, hex(0x2CC5B5)); ui.text(crate::inhalt::t("ui.vorb.reiter"), x0() + 394.0, ly, 15.0, Theme::text2());
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
    ui.text_right(&facts, x0() + cbreite(), Y0 - 20.0, 16.0, Theme::text2());
    let filters = [crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("aka.pos.tw"), crate::inhalt::t("achse.Abwehr"), crate::inhalt::t("gb2.vgl.mittel"), crate::inhalt::t("bericht.achse.angriff"), crate::inhalt::t("pille.verletzt")];
    if let Some(f) = ui.segment(Rect::new(x0() + 4.0, Y0, 640.0, 36.0), &filters, v.market_pos.min(5)) { v.market_pos = f; v.scroll = 0.0; v.scrolls.clear(); }
    let sq: Vec<usize> = all.into_iter().filter(|&p| { let pl = &c.world.players[p]; match v.market_pos { 1..=4 => part_of(&pl.pos) + 1 == v.market_pos, 5 => pl.inj > 0, _ => true } }).collect();
    let gw = cbreite() - 300.0;
    let ch = 156.0;
    // as many columns of at least 182 as fit, then the cards share the whole width (no gap before „Im Blick“ on wide windows)
    let cols = ((gw + 12.0) / 194.0).max(1.0) as usize;
    let cw = (gw + 12.0) / cols as f32 - 12.0;
    let rows = ((hoehe() - Y0 - 60.0) / (ch + 12.0)) as usize;
    let area = Rect::new(x0(), Y0 + 46.0, gw, hoehe() - Y0 - 60.0);
    let start = scroll_rows(ui, v, area, (sq.len() + cols - 1) / cols, rows) * cols;
    if sq.is_empty() { ui.empty(area, crate::inhalt::t("ui.pages.hier_ist_gerade_niemand_gutes")); }
    for (k, &p) in sq.iter().skip(start).take(rows * cols).enumerate() {
        let pl = &c.world.players[p];
        let r = Rect::new(x0() + 4.0 + (k % cols) as f32 * (cw + 12.0), Y0 + 50.0 + (k / cols) as f32 * (ch + 12.0), cw, ch);
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
    let dr = Rect::new(x0() + gw + 12.0, Y0, cbreite() - gw - 8.0, hoehe() - Y0 - 14.0);
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
        ui.title(head, x0() + 8.0, Y0 + 26.0, 28.0, Theme::text());
        let bw = cbreite() - 8.0;
        rrect(x0() + 4.0, Y0 + 40.0, bw, 12.0, 6.0, Theme::problem());
        rrect(x0() + 4.0 + bw * (1.0 - share), Y0 + 40.0, bw * share, 12.0, 6.0, Theme::gut());
        ui.text(&crate::tf!("kabine.neu.unzufrieden", sad.len()), x0() + 8.0, Y0 + 72.0, 15.0, Theme::problem());
        ui.text_right(&crate::tf!("kabine.neu.zufrieden", happy.len()), x0() + cbreite() - 4.0, Y0 + 72.0, 15.0, Theme::gut());
        let colw = (cbreite() - 12.0) / 2.0;
        let today = c.abs();
        for (k, list) in [&sad, &happy].iter().enumerate() {
            let r = Rect::new(x0() + 4.0 + k as f32 * (colw + 12.0), Y0 + 86.0, colw, hoehe() - Y0 - 100.0);
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
            let lx = x0() + cbreite() / 2.0 - total / 2.0;
            let pw = total + 40.0;
            rrect(x0() + cbreite() / 2.0 - pw / 2.0, y, pw, rowh - 10.0, 18.0, alpha(Theme::verein(), 0.10 + 0.06 * (4 - n.min(4)) as f32));
            ui.text_semi(t, x0() + cbreite() / 2.0 - pw / 2.0 + 14.0, y + 18.0, 13.0, Theme::text2());
            for (j, &p) in take.iter().enumerate() {
                let pl = &c.world.players[p];
                let cx = lx + 46.0 + j as f32 * 92.0;
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
    let mut sq = c.squad();
    let liste = 10 + v.tab as u8;
    sortieren(v, liste, &mut sq, |p, col| {
        let pl = &c.world.players[p];
        let n = |x: f32| Sk::N(x);
        match (v.tab == 1, col) {
            (_, 0) => n(-(pl.no as f32)),
            (_, 1) => Sk::S(pl.name.to_lowercase()),
            (_, 2) => n(-(crate::pesdb::pos_index(&pl.pos) as f32)),
            (true, 3) => n(pl.age as f32), (true, 4) => n(pl.ovr), (true, 5) => n(pl.pot), (true, 6) => n(pl.form),
            (true, 7) => n(pl.fit), (true, 8) => n(pl.morale), (true, 9) => n(pl.value as f32), (true, _) => n(-(pl.contract_end as f32)),
            (false, 3) => n(pl.apps as f32), (false, 4) => n(pl.goals as f32), (false, 5) => n(pl.assists as f32), (false, 6) => n(pl.avg_rating()),
            (false, 7) => n(if pl.last.is_empty() { 0.0 } else { pl.last.iter().sum::<f32>() / pl.last.len() as f32 }),
            (false, 8) => n(pl.yellow as f32), (false, _) => n(pl.wage as f32),
        }
    });
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    let head: Vec<(&str, f32)> = if v.tab == 1 {
        vec![(crate::id("NR"), 14.0), (crate::id("NAME"), 60.0), (crate::id("POS."), 330.0), (crate::id("ALTER"), 390.0), (crate::inhalt::t("ui.pages.staerke"), 450.0), (crate::id("POTENZIAL"), 520.0), (crate::id("FORM"), 620.0), (crate::id("FITNESS"), 680.0), (crate::id("MORAL"), 800.0), (crate::id("MARKTWERT"), 920.0), (crate::id("VERTRAG"), 1050.0)]
    } else {
        vec![(crate::id("NR"), 14.0), (crate::id("NAME"), 60.0), (crate::id("POS."), 330.0), (crate::id("SPIELE"), 400.0), (crate::id("TORE"), 480.0), (crate::id("VORLAGEN"), 550.0), (crate::inhalt::t("ui.pages.note"), 650.0), (crate::inhalt::t("ui.pages.letzte_fuenf"), 740.0), (crate::id("GELB"), 960.0), (crate::id("GEHALT"), 1030.0)]
    };
    sort_head(ui, v, liste, area.x, area.y + 26.0, &head);
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
/// The mean grade of a player's last five matches, if he played any.
fn form5(pl: &crate::world::Player) -> Option<f32> {
    let l: Vec<f32> = pl.last.iter().rev().take(5).copied().collect();
    (!l.is_empty()).then(|| l.iter().sum::<f32>() / l.len() as f32)
}

fn aufstellung(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("gb2.reiter.taktik"), crate::inhalt::t("ui.pages.standards_und_vorlagen"), crate::inhalt::t("bericht.gegner"), crate::inhalt::t("ui.pages.rollen_und_anweisungen")]);
    if v.tab == 3 { rollen_seite(ui, v, c); return; }
    if v.tab == 1 { standards_seite(ui, v, c); return; }
    if v.tab == 2 { gegner_seite(ui, c); return; }
    // left: Meine Taktik
    let lt = Rect::new(x0() + 4.0, Y0, 280.0, hoehe() - Y0 - 16.0);
    ui.card(lt);
    ui.title(crate::inhalt::t("tk.kompakt.meine"), lt.x + 14.0, lt.y + 30.0, 22.0, Theme::text());
    ui.text(crate::inhalt::t("ui.pages.gilt_fuer_alle_phasen"), lt.x + 14.0, lt.y + 50.0, 14.0, Theme::text2());
    let mut y = lt.y + 66.0;
    ui.text_semi(crate::inhalt::t("ui.wort.gross_formation"), lt.x + 14.0, y + 10.0, 14.0, Theme::text2());
    if let Some(f) = ui.segment(Rect::new(lt.x + 10.0, y + 18.0, lt.w - 20.0, 30.0), &FORMATIONS, c.formation) {
        c.formation = f;
        c.frei.clear();
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
    let pitch = Rect::new(mid_x, Y0 + 40.0, mw, hoehe() - Y0 - 126.0);
    // the co-trainer's criteria, freely combined (Fabian 23.11.): a click toggles one, the line-up follows at once
    {
        let cw = (mw - 6.0 * 6.0) / 7.0;
        let y = pitch.y + pitch.h + 52.0;
        for b in 0..7u8 {
            let an = c.aufstellung_kriterien & (1 << b) != 0;
            if ui.button(Rect::new(mid_x + b as f32 * (cw + 6.0), y, cw, 28.0), &ui.fit(crate::inhalt::t(&format!("ui.kriterium.{b}")), 13.0, cw - 8.0), an) {
                let neu = c.aufstellung_kriterien ^ (1 << b);
                c.aufstellung_kriterien = if neu == 0 { 1 } else { neu };
                c.auto_lineup();
            }
        }
    }
    rrect(pitch.x, pitch.y, pitch.w, pitch.h, 8.0, hex(0x1E5A35));
    ui.rahmen(pitch);
    for i in 0..8 { if i % 2 == 0 { draw_rectangle(pitch.x + 6.0, pitch.y + 6.0 + i as f32 * (pitch.h - 12.0) / 8.0, pitch.w - 12.0, (pitch.h - 12.0) / 8.0, hex(0x226440)); } }
    let lc = Color::new(1.0, 1.0, 1.0, 0.35);
    draw_rectangle_lines(pitch.x + 6.0, pitch.y + 6.0, pitch.w - 12.0, pitch.h - 12.0, 2.0, lc);
    draw_line(pitch.x + 6.0, pitch.y + pitch.h / 2.0, pitch.x + pitch.w - 6.0, pitch.y + pitch.h / 2.0, 2.0, lc);
    draw_circle_lines(pitch.x + pitch.w / 2.0, pitch.y + pitch.h / 2.0, 60.0, 2.0, lc);
    draw_rectangle_lines(pitch.x + pitch.w / 2.0 - 120.0, pitch.y + 6.0, 240.0, 80.0, 2.0, lc);
    draw_rectangle_lines(pitch.x + pitch.w / 2.0 - 120.0, pitch.y + pitch.h - 86.0, 240.0, 80.0, 2.0, lc);
    let spots = c.eigene_spots();
    let mut swap: Option<(usize, usize)> = None;
    // free placement (Fabian 22.11.): hold a card and drag it; a short click still picks and swaps
    let zug_ziel = |m: Vec2| (((m.x - pitch.x - 40.0) / (pitch.w - 80.0)).clamp(0.02, 0.98), ((pitch.y + pitch.h - 50.0 - m.y) / (pitch.h - 100.0)).clamp(0.12, 0.95));
    let gezogen = v.tafel_zug.filter(|(_, s)| ui.mouse.distance(*s) > 8.0);
    // drag and drop (Fabian 23.11.): a card dropped on another card swaps the two, on a bench row it swaps with that
    // player, anywhere else on the pitch it moves the spot
    let karten: Vec<Rect> = spots.iter().map(|&(sx, sy)| { let (cx, cy) = (pitch.x + 40.0 + sx * (pitch.w - 80.0), pitch.y + pitch.h - 50.0 - sy * (pitch.h - 100.0)); Rect::new(cx - 46.0, cy - 40.0, 92.0, 96.0) }).collect();
    let bank_rect = { let rx = mid_x + mw + 14.0; Rect::new(rx, Y0, breite() - 18.0 - rx, hoehe() - Y0 - 16.0) };
    let mut auf_bank: Option<usize> = None;
    if let Some((i, _)) = gezogen {
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(j) = karten.iter().position(|k| k.contains(ui.mouse)).filter(|&j| j != i) { swap = Some((i, j)); }
            else if bank_rect.contains(ui.mouse) { auf_bank = Some(i); }
            else if i > 0 {
                let mut f: Vec<(f32, f32)> = spots.to_vec();
                f[i] = zug_ziel(ui.mouse);
                c.frei = f;
                c.co_lineup = false;
            }
            v.tafel_zug = None;
            v.pick = None;
        }
    } else if is_mouse_button_released(MouseButton::Left) { v.tafel_zug = None; }
    for (i, (slot, p, e)) in xi.iter().enumerate() {
        let (sx, sy) = spots[i];
        let mut cx = pitch.x + 40.0 + sx * (pitch.w - 80.0);
        let mut cy = pitch.y + pitch.h - 50.0 - sy * (pitch.h - 100.0);
        if gezogen.map_or(false, |g| g.0 == i) && i > 0 { cx = ui.mouse.x; cy = ui.mouse.y; }
        if is_mouse_button_pressed(MouseButton::Left) && Rect::new(cx - 46.0, cy - 40.0, 92.0, 96.0).contains(ui.mouse) { v.tafel_zug = Some((i, ui.mouse)); }
        let r = Rect::new(cx - 46.0, cy - 40.0, 92.0, 96.0);
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
        // fitness, morale and form of the last five (Fabian 23.11.)
        ui.text(&format!("{:.0}%", pl.fit), r.x + 6.0, r.y + 89.0, 11.0, if pl.fit > 75.0 { Theme::gut() } else if pl.fit > 60.0 { Theme::achtung() } else { Theme::problem() });
        ui.smiley(cx, r.y + 85.0, 6.0, pl.morale);
        if let Some(f) = form5(pl) { ui.text_right(&format!("{f:.1}"), r.x + r.w - 6.0, r.y + 89.0, 11.0, Theme::note(f)); }
        draw_rectangle(r.x + 8.0, r.y + r.h - 4.0, (r.w - 16.0) * pl.fit / 100.0, 2.0, if pl.fit > 75.0 { Theme::gut() } else { Theme::achtung() });
        if gezogen.is_none() && ui.hit(r) {
            if let Some(a) = v.pick { if a != i { swap = Some((a, i)); } v.pick = None; } else { v.pick = Some(i); }
        }
    }
    if c.frei.len() == 11 && ui.button(Rect::new(pitch.x + pitch.w - 200.0, pitch.y + 10.0, 190.0, 28.0), crate::inhalt::t("ui.taktik.frei_zurueck"), false) { c.frei.clear(); c.auto_lineup(); }
    ui.text(&crate::tf!("ui.pages.abwehrlinie_kompaktheit_pressing", c.tactic.linie, c.tactic.kompakt, if c.tactic.breite == 0 { crate::inhalt::t("ui.wort.breit") } else { crate::inhalt::t("ui.wort.eng") }, if c.tactic.pressing == 0 { crate::inhalt::t("ui.wort.hart") } else { crate::inhalt::t("ui.wort.ruhig") }), mid_x + 8.0, pitch.y + pitch.h + 22.0, 15.0, Theme::text2());
    ui.text(crate::inhalt::t("ui.taktik.frei_hinweis"), mid_x + 8.0, pitch.y + pitch.h + 42.0, 14.0, Theme::gedaempft());
    let mut cur: Vec<usize> = xi.iter().map(|x| x.1).collect();
    if let Some((a, b)) = swap { cur.swap(a, b); c.xi = cur.clone(); c.co_lineup = false; }
    // a bench player dropped on a card comes in for that player
    let bank_gezogen = v.bank_zug.filter(|(_, s)| ui.mouse.distance(*s) > 8.0);
    if is_mouse_button_released(MouseButton::Left) {
        if let Some((p, _)) = bank_gezogen {
            if let Some(j) = karten.iter().position(|k| k.contains(ui.mouse)) { if j < cur.len() && c.world.players[p].available() { cur[j] = p; c.xi = cur.clone(); c.co_lineup = false; } }
        }
    }
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
        let nm = ui.fit(&pl.name, 14.0, r.w - 136.0);
        ui.text(&nm, r.x + 72.0, r.y + 13.0, 14.0, if pl.available() { Theme::text() } else { Theme::problem() });
        // fitness, morale and form of the last five under the name (Fabian 23.11.)
        ui.text(&format!("{:.0}%", pl.fit), r.x + 72.0, r.y + 27.0, 11.0, if pl.fit > 75.0 { Theme::gut() } else if pl.fit > 60.0 { Theme::achtung() } else { Theme::problem() });
        ui.smiley(r.x + 112.0, r.y + 23.0, 5.5, pl.morale);
        if let Some(f) = form5(pl) { ui.text(&crate::tf!("ui.taktik.form5", format!("{f:.1}")), r.x + 124.0, r.y + 27.0, 11.0, Theme::note(f)); }
        ui.pill(r.x + r.w - 22.0, r.y + 15.0, pl.ovr, 32.0);
        zuschlag_text(ui, r.x + r.w - 60.0, r.y + 20.0, &pl);
        if let Some(i) = auf_bank.filter(|_| r.contains(ui.mouse)) { if i < cur.len() && pl.available() { cur[i] = p; c.xi = cur.clone(); c.co_lineup = false; } }
        if ui.hit(r) { v.bank_zug = Some((p, ui.mouse)); }
        // a click without moving: as before (swap with the picked card, else the profile)
        if is_mouse_button_released(MouseButton::Left) && bank_gezogen.is_none() && v.bank_zug.map_or(false, |b| b.0 == p) && r.contains(ui.mouse) {
            if let Some(a) = v.pick { if a < cur.len() { cur[a] = p; c.xi = cur.clone(); c.co_lineup = false; } v.pick = None; } else { v.modal = Modal::Player(p); }
        }
        yy += 32.0;
    }
    if is_mouse_button_released(MouseButton::Left) { v.bank_zug = None; }
    // the dragged bench player follows the mouse
    if let Some((p, _)) = bank_gezogen {
        let pl = &c.world.players[p];
        let g = Rect::new(ui.mouse.x - 70.0, ui.mouse.y - 16.0, 140.0, 32.0);
        rrect(g.x, g.y, g.w, g.h, 6.0, alpha(Theme::akzent(), 0.9));
        ui.player_photo(g.x + 16.0, g.y + 16.0, 11.0, pl.id, &pl.name);
        let nm = ui.fit(pl.name.split_whitespace().last().unwrap_or(&pl.name), 14.0, g.w - 40.0);
        ui.text(&nm, g.x + 32.0, g.y + 21.0, 14.0, Theme::auf_akzent());
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
    kpi(ui, Rect::new(x0() + 4.0, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_trainingstage"), &format!("{days}"), crate::inhalt::t("gm.tr.kz.tage.unter"), Theme::gut());
    kpi(ui, Rect::new(x0() + 16.0 + w4, y, w4, 92.0), crate::inhalt::t("ui.pages.hohe_belastung"), &format!("{tired}"), if tired == 0 { crate::inhalt::t("gm.tr.kz.belastung.ok") } else { crate::inhalt::t("ui.pages.spieler_unter_60_fitness") }, if tired == 0 { Theme::gut() } else { Theme::achtung() });
    kpi(ui, Rect::new(x0() + 28.0 + 2.0 * w4, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_einzelprogramme"), &crate::tf!("ui.pages.6", progs), crate::inhalt::t("ui.pages.fokus_oder_extraschicht"), Theme::akzent());
    kpi(ui, Rect::new(x0() + 40.0 + 3.0 * w4, y, w4, 92.0), crate::inhalt::t("ui.wort.gross_eingespielt"), &format!("{:.0} %", c.club().fam), crate::inhalt::t("ui.pages.taktiktraining_hilft"), Theme::akzent());
    let y = y + 106.0;
    let mut co = c.co_training;
    rrect(x0() + 4.0, y, 120.0, 24.0, 12.0, Theme::erhoehung());
    ui.text(if co { crate::inhalt::t("ui.pages.co_trainer_plant") } else { crate::inhalt::t("modus.manuell") }, x0() + 14.0, y + 17.0, 14.0, Theme::text2());
    if ui.button(Rect::new(x0() + 134.0, y - 2.0, 230.0, 28.0), if co { crate::inhalt::t("ui.pages.selbst_planen") } else { crate::inhalt::t("ui.pages.an_den_co_trainer_abgeben") }, false) { co = !co; c.co_training = co; if co { c.co_wochenplan(); v.tp_zelle = None; } }
    // what the plan does for development: intensity × repetition (entwicklung.rs plan_faktor), and the legend of the list
    ui.text(&if v.tab == 0 { crate::tf!("ui.ent.plan_wirkung", format!("{:.2}", c.plan_faktor())) } else { crate::inhalt::t("ui.ent.legende").to_string() }, x0() + 380.0, y + 17.0, 14.0, Theme::gedaempft());
    if v.tab == 0 {
        trainingswoche(ui, v, c, y + 36.0);
    } else {
        let sq = c.squad();
        let area = Rect::new(x0() + 4.0, y + 36.0, cbreite(), hoehe() - y - 52.0);
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

/// Colour of a training group.
fn tp_farbe(g: &str) -> Color {
    match g { "technik" => Theme::akzent(), "standards" => Theme::gold(), "defensive" => Theme::problem(), "athletik" => Theme::achtung(), "torwart" => hex(0xB48CFF), "taktik" => Theme::gut(), _ => Theme::gedaempft() }
}

/// The training week: seven days of three sessions; a click on a session and then on a unit fills it, a click on the day
/// type changes it, the intensity chip cycles schonend / normal / hart (trainingsplan.rs).
fn trainingswoche(ui: &mut Ui, v: &mut View, c: &mut Career, y: f32) {
    use crate::trainingsplan as tp;
    c.wochenplan_an();
    let frei = !c.co_training;
    let pw = 330.0;
    let gw = cbreite() - pw - 12.0;
    let grid = Rect::new(x0() + 4.0, y, gw, 64.0 + 3.0 * 70.0 + 40.0);
    ui.card(grid);
    let lx = 104.0;
    let dw = (gw - lx - 12.0) / 7.0;
    let tage: Vec<&str> = (0..7).map(|d| crate::inhalt::t(&format!("training.tag.{d}"))).collect();
    let heute = date::weekday(c.world.day);
    for (s, k) in [crate::id("vormittag"), crate::id("nachmittag"), crate::id("abend")].iter().enumerate() {
        ui.text_semi(crate::inhalt::t(&format!("tp.slot.{k}")), grid.x + 12.0, grid.y + 64.0 + s as f32 * 70.0 + 38.0, 14.0, Theme::text2());
    }
    let lasten = tp::tageslast(&c.wochenplan, true);
    let mut geaendert = false;
    for d in 0..7 {
        let x = grid.x + lx + d as f32 * dw;
        ui.text_semi(tage[d], x + 6.0, grid.y + 24.0, 16.0, if d == heute { Theme::akzent() } else { Theme::text() });
        let typ = c.wochenplan.tage[d].typ;
        let tr = Rect::new(x + 4.0, grid.y + 32.0, dw - 8.0, 24.0);
        rrect(tr.x, tr.y, tr.w, tr.h, 12.0, alpha(if typ == 1 { Theme::gold() } else if typ == 3 { Theme::gedaempft() } else if typ == 2 { Theme::gut() } else { Theme::akzent() }, 0.18));
        ui.text_center(&ui.fit(crate::inhalt::t(&format!("tp.typ.{}", tp::TYPEN[typ as usize])), 13.0, tr.w - 8.0), tr.x + tr.w / 2.0, tr.y + 17.0, 13.0, Theme::text());
        if frei && ui.hit(tr) { tp::typ_setzen(&mut c.wochenplan.tage[d], (typ + 1) % 4); geaendert = true; }
        for s in 0..tp::SLOTS {
            let r = Rect::new(x + 4.0, grid.y + 64.0 + s as f32 * 70.0, dw - 8.0, 64.0);
            let sel = v.tp_zelle == Some((d, s));
            let geht = typ != 3 && !(typ == 1 && s > 0);
            match c.wochenplan.tage[d].slots[s].clone() {
                Some(z) => {
                    let col = tp_farbe(tp::gruppe(&z.einheit));
                    rrect(r.x, r.y, r.w, r.h, 6.0, alpha(col, 0.14));
                    draw_rectangle(r.x, r.y + 6.0, 3.0, r.h - 12.0, col);
                    ui.text_passend(tp::name(&z.einheit), r.x + 10.0, r.y + 20.0, 14.0, r.w - 14.0, 2, Theme::text());
                    let ch = Rect::new(r.x + 8.0, r.y + r.h - 22.0, (r.w - 16.0).min(86.0), 18.0);
                    rrect(ch.x, ch.y, ch.w, ch.h, 9.0, alpha([Theme::gut(), Theme::akzent(), Theme::problem()][z.stufe.min(2) as usize], 0.25));
                    ui.text_center(crate::inhalt::t(&format!("tp.intensitaet.{}", tp::STUFEN[z.stufe.min(2) as usize])), ch.x + ch.w / 2.0, ch.y + 13.0, 12.0, Theme::text());
                    if frei && ui.hit(ch) {
                        if let Some(z) = c.wochenplan.tage[d].slots[s].as_mut() { z.stufe = (z.stufe + 1) % 3; }
                        geaendert = true;
                    }
                }
                None => {
                    rrect_line(r.x, r.y, r.w, r.h, 6.0, if geht { Theme::kontur() } else { alpha(Theme::kontur(), 0.3) });
                    if geht && frei { ui.text_center("+", r.x + r.w / 2.0, r.y + r.h / 2.0 + 8.0, 22.0, Theme::gedaempft()); } // roh
                }
            }
            if sel { rrect_line(r.x - 1.0, r.y - 1.0, r.w + 2.0, r.h + 2.0, 6.0, Theme::akzent()); }
            if frei && geht && ui.hit(r) { v.tp_zelle = if sel { None } else { Some((d, s)) }; }
        }
        // the day's load
        let l = lasten[d];
        let by = grid.y + 64.0 + 3.0 * 70.0 + 6.0;
        ui.bar(x + 6.0, by, dw - 12.0, 6.0, (l / 12.0).clamp(0.0, 1.0), if l <= 0.0 { Theme::gut() } else if l <= 4.0 { Theme::akzent() } else if l <= 8.0 { Theme::achtung() } else { Theme::problem() });
        let stufe = if l <= 0.0 { crate::id("erholt") } else if l <= 3.0 { crate::id("leicht") } else if l <= 7.0 { crate::id("mittel") } else { crate::id("hoch") };
        ui.text(crate::inhalt::t(&format!("tp.last.{stufe}")), x + 6.0, by + 22.0, 12.0, Theme::gedaempft());
    }
    // what the week brings: the abilities it trains most
    let wr = Rect::new(x0() + 4.0, grid.y + grid.h + 12.0, gw, hoehe() - grid.y - grid.h - 28.0);
    ui.card(wr);
    ui.text_semi(crate::inhalt::t("tp.ui.wirkung"), wr.x + 14.0, wr.y + 26.0, 16.0, Theme::text());
    let pk = tp::punkte(&c.wochenplan);
    let mut rang: Vec<(usize, f32)> = pk.iter().copied().enumerate().filter(|x| x.1 > 0.0).collect();
    rang.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let maxp = rang.first().map_or(1.0, |x| x.1);
    let zeilen = (((wr.h - 44.0) / 24.0) as usize).max(1);
    let spw = (wr.w - 28.0) / 2.0;
    for (k, (i, p)) in rang.iter().take(zeilen * 2).enumerate() {
        let (cx, cy) = (wr.x + 14.0 + (k / zeilen) as f32 * spw, wr.y + 52.0 + (k % zeilen) as f32 * 24.0);
        ui.text(&ui.fit(crate::inhalt::t(&format!("faehigkeit.{}", crate::entwicklung::NAMEN[*i])), 14.0, 150.0), cx, cy, 14.0, Theme::text2());
        ui.bar(cx + 160.0, cy - 9.0, spw - 220.0, 8.0, p / maxp, Theme::akzent());
    }
    if rang.is_empty() { ui.text(crate::inhalt::t("ui.tp.bericht.nichts"), wr.x + 14.0, wr.y + 54.0, 14.0, Theme::gedaempft()); }
    // right: the units for the chosen session, else the ready-made weeks and the load of the week
    let pr = Rect::new(x0() + gw + 16.0, y, pw - 4.0, hoehe() - y - 16.0);
    ui.card(pr);
    let mut yy = pr.y + 26.0;
    if let Some((d, s)) = v.tp_zelle.filter(|_| frei) {
        let typ = c.wochenplan.tage[d].typ;
        ui.text_semi(&format!("{} · {}", tage[d], crate::inhalt::t(&format!("tp.slot.{}", [crate::id("vormittag"), crate::id("nachmittag"), crate::id("abend")][s]))), pr.x + 14.0, yy, 16.0, Theme::text());
        yy += 12.0;
        let mut kat = tp::katalog();
        kat.sort_by_key(|id| (tp::gruppe(id), tp::name(id)));
        let mut hover = None;
        for id in kat {
            let r = Rect::new(pr.x + 8.0, yy, pr.w - 16.0, 26.0);
            let geht = tp::passt(typ, s, id);
            let an = c.wochenplan.tage[d].slots[s].as_ref().map_or(false, |z| z.einheit == id);
            if an { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::akzent(), 0.14)); } else if geht && ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::ueberlagerung()); hover = Some(id); }
            draw_circle(r.x + 10.0, r.y + 13.0, 4.0, tp_farbe(tp::gruppe(id)));
            ui.text(tp::name(id), r.x + 22.0, r.y + 18.0, 14.0, if geht { Theme::text() } else { Theme::gedaempft() });
            if ui.hit(r) {
                if geht { c.wochenplan.tage[d].slots[s] = Some(crate::trainingsplan::Zelle { einheit: id.into(), stufe: 1 }); geaendert = true; } else { v.toast(crate::inhalt::t("tp.fehler.plan"), false); }
            }
            yy += 27.0;
        }
        yy += 6.0;
        if c.wochenplan.tage[d].slots[s].is_some() && ui.button(Rect::new(pr.x + 8.0, yy, pr.w - 16.0, 30.0), crate::inhalt::t("tp.ui.einheit_entfernen"), false) { c.wochenplan.tage[d].slots[s] = None; geaendert = true; }
        yy += 42.0;
        let zeig = hover.or(c.wochenplan.tage[d].slots[s].as_ref().map(|z| z.einheit.as_str()).and_then(|e| tp::katalog().into_iter().find(|x| *x == e)));
        if let Some(id) = zeig { ui.paragraph(crate::inhalt::t(&format!("tp.einheit.{id}.text")), pr.x + 14.0, yy, 13.0, pr.w - 28.0, Theme::text2()); }
    } else {
        ui.text_semi(crate::inhalt::t("tp.ui.vorlagen"), pr.x + 14.0, yy, 16.0, Theme::text());
        yy += 14.0;
        for (id, w) in tp::werk() {
            let r = Rect::new(pr.x + 8.0, yy, pr.w - 16.0, 30.0);
            if frei { if ui.button(r, crate::inhalt::t(&format!("tp.vorlage.{id}.name")), c.wochenplan == w) { c.wochenplan = w; geaendert = true; v.toast(crate::inhalt::t("tp.ok.uebernommen"), true); } }
            else { ui.text(crate::inhalt::t(&format!("tp.vorlage.{id}.name")), r.x + 6.0, r.y + 20.0, 14.0, Theme::text2()); }
            yy += 36.0;
        }
        yy += 10.0;
        let sq = c.squad();
        let start = sq.iter().map(|&p| c.world.players[p].ps.last).sum::<f32>() / sq.len().max(1) as f32;
        let (_, zone) = tp::verlauf(&c.wochenplan, start);
        ui.text_semi(crate::inhalt::t("tp.ui.belastung"), pr.x + 14.0, yy, 16.0, Theme::text());
        draw_circle(pr.x + 20.0, yy + 22.0, 6.0, [Theme::gut(), Theme::gut(), Theme::achtung(), Theme::problem()][zone]);
        yy += 16.0;
        let h = ui.paragraph(crate::inhalt::t(&format!("tp.zone.{}", tp::zone_name(zone))), pr.x + 34.0, yy + 10.0, 14.0, pr.w - 48.0, Theme::text2());
        yy += h.max(20.0) + 16.0;
        if zone >= 2 { ui.paragraph(crate::inhalt::t(if zone == 3 { "tp.hinweis.belastung_rot" } else { "tp.hinweis.belastung_gelb" }), pr.x + 14.0, yy, 13.0, pr.w - 28.0, Theme::gedaempft()); }
        ui.paragraph(crate::inhalt::t(if frei { "ui.tp.waehlen" } else { "ui.tp.co_plan" }), pr.x + 14.0, pr.y + pr.h - 50.0, 13.0, pr.w - 28.0, Theme::gedaempft());
    }
    if geaendert { c.plan_uebernehmen(); }
}

// ================================================================ Medizin
fn medizin(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("gl.kz.verletzte"), crate::inhalt::t("vorschau.belastung")]);
    let mut sq = c.squad();
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    if v.tab == 0 {
        sq.retain(|&p| c.world.players[p].inj > 0);
        if sq.is_empty() { ui.title(crate::inhalt::t("pk.karte.keine_ausfaelle"), area.x + 20.0, area.y + 50.0, 28.0, Theme::gut()); ui.text(crate::inhalt::t("ui.pages.der_mannschaftsarzt_meldet_keine_ausfael"), area.x + 20.0, area.y + 80.0, 18.0, Theme::text2()); }
        // one row per injured player: rehab phase, the physio's estimate and the doctor's options (medizin.rs)
        let mut meldung = None;
        for (k, &p) in sq.iter().enumerate() {
            let y = area.y + 16.0 + k as f32 * 104.0;
            if y + 96.0 > area.y + area.h { break; }
            let Some(st) = c.verletzung_stand(p) else { continue };
            let (von, bis) = c.prognose(p);
            let colw = (area.w - 616.0 - 300.0 - 20.0).clamp(160.0, 380.0);
            let pl = &c.world.players[p];
            ui.pos_tag(area.x + 16.0, y + 22.0, pos_de(&pl.pos));
            ui.player_photo(area.x + 70.0, y + 22.0, 18.0, pl.id, &pl.name);
            ui.text_semi(&pl.name, area.x + 96.0, y + 20.0, 20.0, Theme::text());
            ui.text(&pl.inj_name, area.x + 96.0, y + 44.0, 16.0, Theme::problem());
            ui.text(&crate::tf!("med.hinweis.prognose", crate::tf!("med.dauer.tage", von, bis)), area.x + 300.0, y + 20.0, 16.0, Theme::text());
            ui.text_passend(&crate::tf!("med.hinweis.phase", crate::inhalt::t(&format!("med.phase.{}", ["behandlung", "aufbau", "mannschaftstraining", "einsatzbereit"][st.phase])), st.phase_rest), area.x + 300.0, y + 44.0, 15.0, colw, 1, Theme::text2()); // roh
            // the four phases as a bar
            let bw = colw;
            for i in 0..4 {
                let x = area.x + 300.0 + i as f32 * (bw / 4.0);
                draw_rectangle(x, y + 58.0, bw / 4.0 - 4.0, 8.0, if i < st.phase { Theme::gut() } else if i == st.phase { Theme::achtung() } else { Theme::ueberlagerung() });
            }
            let bx = area.x + area.w - 16.0;
            let risiko = if st.rueckfall > 1.0 { crate::tf!("med.hinweis.rueckfall_frueh", format!("{:.1}", st.rueckfall), crate::regeln::f("medizin", "reha.rueckfall_tage")) } else { crate::inhalt::t("med.hinweis.frueh_ohne").to_string() };
            if ui.button(Rect::new(bx - 600.0, y + 6.0, 190.0, 32.0), crate::inhalt::t("ui.med.einsetzen"), false) { meldung = Some((p, c.frueh_einsetzen(p))); }
            ui.text_passend(&risiko, bx - 600.0, y + 58.0, 13.0, 190.0, 2, Theme::gedaempft());
            if let Some((_, _, _, kosten)) = c.reha_wirkung(p) {
                if ui.button(Rect::new(bx - 400.0, y + 6.0, 190.0, 32.0), crate::inhalt::t("med.akt.reha"), false) { meldung = Some((p, c.reha_programm(p))); }
                ui.text_passend(&crate::tf!("ui.med.kosten", money(kosten)), bx - 400.0, y + 58.0, 13.0, 190.0, 2, Theme::gedaempft());
            }
            if c.fitspritze_moeglich(p) {
                if ui.button(Rect::new(bx - 200.0, y + 6.0, 190.0, 32.0), crate::inhalt::t("med.akt.fitspritze"), false) { meldung = Some((p, c.fitspritze(p))); }
                ui.text_passend(&crate::tf!("med.hinweis.fit_rueckfall", format!("{:.0}", crate::regeln::f("medizin", "fitspritze.rueckfall_faktor")), crate::regeln::f("medizin", "fitspritze.rueckfall_tage")), bx - 200.0, y + 58.0, 13.0, 190.0, 2, Theme::gedaempft());
            }
            draw_rectangle(area.x + 14.0, y + 96.0, area.w - 28.0, 1.0, Theme::kontur());
        }
        if let Some((p, r)) = meldung {
            match r {
                Ok(text) => { let n = c.world.players[p].name.clone(); c.msg(3, crate::inhalt::t("med.vg.absender"), &crate::tf!("ui.med.entscheidung", n), &text, Action::None); v.toast(&text, true); }
                Err(e) => v.toast(&e, false),
            }
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
            // strain light: colour and the biggest driver (medizin.py ampel)
            let (_, farbe, treiber) = c.ampel_von(p);
            let fc = [Theme::gut(), Theme::achtung(), Theme::problem()][farbe as usize];
            draw_circle(area.x + 1010.0, y + 14.0, 8.0, fc);
            ui.text(&format!("{} · {}", crate::inhalt::t(&format!("med.ampel.{}", ["gruen", "gelb", "rot"][farbe as usize])), crate::inhalt::t(&format!("med.ampel.{}", treiber[0].0))), area.x + 1026.0, y + 20.0, 16.0, Theme::text2()); // roh
        }
    }
}

fn standards_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let r = Rect::new(x0() + 4.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0);
    ui.card_titled(r, crate::inhalt::t("ui.pages.standards_und_kapitaen"), Theme::akzent());
    for (k, n) in [crate::inhalt::t("gb.schluessel.standard_elfmeter"), crate::inhalt::t("gb.schluessel.standard_freistoss_lang"), crate::inhalt::t("beob.stat.ecken"), "Kapitän"].iter().enumerate() { // roh
        let yy = r.y + 70.0 + k as f32 * 56.0;
        ui.text_semi(n, r.x + 14.0, yy, 18.0, Theme::text());
        let wer = c.schuetze(k as u8).map(|p| c.world.players[p].name.clone()).unwrap_or("–".into());
        ui.text(&wer, r.x + 150.0, yy, 18.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 140.0, yy - 20.0, 126.0, 30.0), crate::inhalt::t("ui.pages.wechseln"), false) { c.schuetze_waehlen(k as u8); }
    }
    ui.paragraph(crate::inhalt::t("ui.pages.ohne_eigene_wahl_uebernimmt_der"), r.x + 14.0, r.y + 320.0, 15.0, r.w - 28.0, Theme::gedaempft());
    let r2 = Rect::new(x0() + cbreite() * 0.5 + 10.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0);
    ui.card_titled(r2, crate::inhalt::t("ui.wort.taktik_vorlagen"), Theme::gold());
    // Plan B (Taktik-Konzept 1.7): from a minute on, when the score says so, the mentality changes by itself
    let (r, py) = (r2, r2.y + 112.0 + c.std.vorlagen.len() as f32 * 46.0);
    {
        ui.text_semi(crate::inhalt::t("ui.planb.titel"), r.x + 14.0, py, 18.0, Theme::akzent());
        let an = c.plan_b.an;
        if ui.button(Rect::new(r.x + r.w - 140.0, py - 20.0, 126.0, 30.0), crate::inhalt::t(if an { "ui.planb.an" } else { "ui.planb.aus" }), an) { c.plan_b.an = !an; }
        if let Some(s) = ui.segment(Rect::new(r.x + 14.0, py + 14.0, r.w - 28.0, 28.0), &[crate::inhalt::t("ui.planb.wenn.0"), crate::inhalt::t("ui.planb.wenn.1"), crate::inhalt::t("ui.planb.wenn.2")], c.plan_b.wenn as usize) { c.plan_b.wenn = s as u8; }
        if let Some(s) = ui.segment(Rect::new(r.x + 14.0, py + 50.0, r.w - 28.0, 28.0), &[crate::inhalt::t("ui.live.ment.0"), crate::inhalt::t("ui.live.ment.1"), crate::inhalt::t("ui.live.ment.2")], (c.plan_b.mentalitaet + 1) as usize) { c.plan_b.mentalitaet = s as i8 - 1; }
        ui.text(&crate::tf!("ui.planb.ab", c.plan_b.minute), r.x + 14.0, py + 104.0, 15.0, Theme::text());
        if let Some(m) = ui.slider(Rect::new(r.x + 140.0, py + 92.0, r.w - 170.0, 16.0), c.plan_b.minute as i32, 46, 85) { c.plan_b.minute = m as u8; }
    }
    if ui.button(Rect::new(r2.x + 14.0, r2.y + 50.0, 260.0, 32.0), crate::inhalt::t("ui.pages.aktuelle_taktik_ablegen"), false) { match c.vorlage_speichern() { Ok(()) => v.toast(crate::inhalt::t("ui.pages.abgelegt"), true), Err(e) => v.toast(&e, false) } }
    let mut del = None;
    for (i, (n, _, _)) in c.std.vorlagen.clone().iter().enumerate() {
        let yy = r2.y + 110.0 + i as f32 * 46.0;
        ui.text(n, r2.x + 14.0, yy, 17.0, Theme::text());
        if ui.button(Rect::new(r2.x + r2.w - 250.0, yy - 20.0, 110.0, 30.0), crate::inhalt::t("ui.pages.laden"), false) { c.vorlage_laden(i); v.toast(crate::inhalt::t("ui.pages.taktik_geladen"), true); }
        if ui.button(Rect::new(r2.x + r2.w - 130.0, yy - 20.0, 116.0, 30.0), crate::inhalt::t("md.ui.mail.loeschen"), false) { del = Some(i); }
    }
    if let Some(i) = del { c.std.vorlagen.remove(i); }
    let yy = (r2.y + 360.0).max(r2.y + 112.0 + c.std.vorlagen.len() as f32 * 46.0 + 130.0);
    ui.text_semi(crate::inhalt::t("ui.pages.urteil_des_co_trainers"), r2.x + 14.0, yy, 16.0, Theme::akzent());
    for (k, l) in c.taktikurteil().iter().enumerate().take(4) { ui.paragraph(l, r2.x + 14.0, yy + 26.0 + k as f32 * 44.0, 15.0, r2.w - 28.0, Theme::text2()); }
}
fn gegner_seite(ui: &mut Ui, c: &mut Career) {
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
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
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
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
    let karte = |k: usize, titel: &str, ui: &mut Ui| { let r = Rect::new(x0() + 4.0 + k as f32 * (w + 10.0), Y0, w, hoehe() - Y0 - 16.0); ui.card_titled(r, titel, if k == 1 { Theme::gold() } else { Theme::akzent() }); r };
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
    let r = Rect::new(x0() + 4.0, y, 420.0, 330.0);
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
    let r2 = Rect::new(x0() + 438.0, y, cbreite() - 434.0, 160.0);
    ui.card_titled(r2, crate::inhalt::t("zentrale.k.ziel"), Theme::gold());
    ui.title(&c.club().goal_kind, r2.x + 14.0, r2.y + 70.0, 34.0, Theme::text());
    let rank = c.world.rank_of(c.own);
    ui.text(&crate::tf!("ui.pages.mindestens_platz_aktuell_platz", c.club().goal_pos, rank), r2.x + 14.0, r2.y + 100.0, 19.0, if rank <= c.club().goal_pos { Theme::gut() } else { Theme::achtung() });
    let risiko = c.entlassungsrisiko();
    let rw = if risiko >= 0.6 { crate::id("hoch") } else if risiko >= 0.3 { crate::inhalt::t("bg.groesse.mittel") } else { crate::id("gering") };
    ui.text(&crate::tf!("ui.pages.entlassungsgefahr_vom_ueberschuss_gehen_", rw, format!("{:.0}", c.budget_anteil() * 100.0)), r2.x + 14.0, r2.y + 130.0, 16.0, Theme::text2());
    let r3 = Rect::new(x0() + 438.0, y + 172.0, cbreite() - 434.0, 158.0);
    ui.card_titled(r3, crate::inhalt::t("ui.pages.umfeld"), Theme::akzent());
    for (i, (n, val)) in [("Fans", c.fans), (crate::inhalt::t("story.groesse.medienbild"), c.media)].iter().enumerate() { // roh
        let yy = r3.y + 60.0 + i as f32 * 46.0;
        ui.text(n, r3.x + 14.0, yy, 18.0, Theme::text());
        ui.bar(r3.x + 150.0, yy - 10.0, r3.w - 240.0, 10.0, val / 100.0, if *val >= 55.0 { Theme::gut() } else if *val >= 35.0 { Theme::achtung() } else { Theme::problem() });
        ui.text_right(&format!("{:.0}", val), r3.x + r3.w - 14.0, yy, 18.0, Theme::text());
    }
    ui.text_right(&crate::tf!("mv.zeile", format!("{:.0}", c.mitglieder())), r3.x + r3.w - 14.0, r3.y + r3.h - 8.0, 15.0, Theme::text2());
    // the club's reputation over the seasons (geschichten.rs), as a line under the surroundings
    if !c.geschichten.tags.is_empty() {
        let n: Vec<&str> = c.geschichten.tags.iter().map(|k| crate::inhalt::t(&format!("gw.ruf.{k}"))).collect(); // roh
        ui.text(&crate::tf!("gw.ruf_zeile", n.join(" · ")), r3.x + 14.0, r3.y + r3.h - 8.0, 15.0, Theme::gold());
    }
    // the three people of the board, the club type and the stage, and the season's side goals (praesidium.rs)
    c.praesidium_an();
    let pw = (cbreite() * 0.62 - 24.0) / 3.0;
    let stufe = c.eskalation();
    for (k, p) in c.praesidium.clone().iter().enumerate() {
        let r = Rect::new(x0() + 4.0 + k as f32 * (pw + 12.0), y + 344.0, pw, 140.0);
        ui.card(r);
        ui.text_semi(crate::inhalt::t(&format!("ui.pr.rolle.{}", p.rolle)), r.x + 14.0, r.y + 24.0, 13.0, Theme::gedaempft());
        ui.text_semi(&ui.fit(&p.name, 18.0, r.w - 28.0), r.x + 14.0, r.y + 50.0, 18.0, Theme::text());
        ui.text(crate::inhalt::t(&format!("ui.pr.haltung.{}", p.haltung)), r.x + 14.0, r.y + 72.0, 14.0, Theme::text2());
        let m = c.person_stimmung(p);
        let col = if m >= 60.0 { Theme::gut() } else if m >= 40.0 { Theme::achtung() } else { Theme::problem() };
        ui.bar(r.x + 14.0, r.y + 96.0, r.w - 70.0, 10.0, m / 100.0, col);
        ui.text_right(&format!("{m:.0}"), r.x + r.w - 14.0, r.y + 106.0, 16.0, col);
        ui.text(crate::inhalt::t(&format!("ui.pr.stimmung.{}", if m >= 60.0 { 0 } else if m >= 40.0 { 1 } else { 2 })), r.x + 14.0, r.y + 130.0, 13.0, Theme::text2());
    }
    let rz = Rect::new(x0() + 4.0 + 3.0 * (pw + 12.0), y + 344.0, cbreite() - 3.0 * (pw + 12.0), 140.0);
    ui.card(rz);
    ui.text_semi(&crate::tf!("ui.pr.typ_zeile", crate::inhalt::t(&format!("ui.pr.typ.{}", c.vereinstyp())), crate::inhalt::t(&format!("ui.pr.stufe.{stufe}"))), rz.x + 14.0, rz.y + 24.0, 14.0, [Theme::gut(), Theme::text(), Theme::achtung(), Theme::problem(), Theme::problem()][stufe]);
    ui.text_semi(crate::inhalt::t("ui.pr.nebenziele"), rz.x + 14.0, rz.y + 52.0, 13.0, Theme::gedaempft());
    let season = c.world.season;
    let ziele: Vec<String> = c.nebenziele.iter().filter(|z| z.season == season).map(|z| z.key.clone()).collect();
    for (k, z) in ziele.iter().enumerate() {
        let (ok, zeile) = c.nebenziel_stand(z);
        let yy = rz.y + 76.0 + k as f32 * 28.0;
        draw_circle(rz.x + 22.0, yy - 5.0, 6.0, if ok { Theme::gut() } else { Theme::achtung() });
        ui.text_semi(crate::inhalt::t(&format!("ui.pr.ziel.{z}.name")), rz.x + 36.0, yy, 15.0, Theme::text());
        ui.text_right(&ui.fit(&zeile, 13.0, rz.w * 0.55), rz.x + rz.w - 14.0, yy, 13.0, Theme::text2());
    }
    let r4 = Rect::new(x0() + 4.0, y + 496.0, cbreite(), (hoehe() - y - 512.0).max(100.0));
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

fn finanzen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // sponsors, stadium and fans are main tabs of their own now (the wrapper pages choose the view)
    if v.page == Page::Finanzen { tabs(ui, v, &[crate::inhalt::t("as.reiter.ueberblick"), crate::inhalt::t("ui.pages.tickets_und_merchandising"), crate::inhalt::t("kr.reiter")]); }
    if v.page == Page::Finanzen && v.tab == 2 { kredite_seite(ui, v, c); return; }
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
        ui.card_titled(r, crate::inhalt::t("ui.pages.laufende_vertraege"), Theme::gold());
        let mut yy = r.y + 60.0;
        if c.wi.vertraege.is_empty() { ui.text(crate::inhalt::t("ui.pages.kein_sponsor_unter_vertrag"), r.x + 14.0, yy, 17.0, Theme::text2()); }
        for sp in c.wi.vertraege.clone() {
            ui.text_semi(&format!("{}: {}", crate::wirtschaft::sponsor_de_t()[sp.art as usize], sp.name), r.x + 14.0, yy, 18.0, Theme::text());
            ui.text(&crate::tf!("ui.pages.im_jahr_bis_bonus_bei", money(sp.jahr), date::season_label(sp.bis), money(sp.bonus)), r.x + 14.0, yy + 24.0, 15.0, Theme::text2());
            yy += 62.0;
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


// ================================================================ Trainerprofil
/// The coach market: vacant posts and hot seats with league, place, board, budget and goal; apply where we may.
fn trainermarkt(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(r, t("tm.titel"), Theme::akzent());
    let hinweis = if c.fired { t("tm.hinweis_frei") } else { t("tm.hinweis_im_amt") };
    ui.text(hinweis, r.x + 14.0, r.y + 56.0, 14.0, Theme::text2());
    let markt = c.trainermarkt();
    if markt.is_empty() { ui.text(t("ui.pages.gerade_ist_kein_passender_posten"), r.x + 14.0, r.y + 90.0, 16.0, Theme::text2()); return; }
    let kopf = [(t("ui.wort.gross_verein"), 60.0), (t("tm.liga"), 360.0), (t("tm.platz"), 580.0), (t("tm.praesidium"), 650.0), (t("tm.budget"), 800.0), (t("tm.ziel"), 940.0)];
    for (n, x) in kopf { ui.text(n, r.x + x, r.y + 86.0, 13.0, Theme::gedaempft()); }
    let area = Rect::new(r.x, r.y + 96.0, r.w, r.h - 100.0);
    let vis = ((area.h - 8.0) / 44.0) as usize;
    let start = scroll_rows(ui, v, area, markt.len(), vis);
    for (k, &(cl, frei, ok, grund)) in markt.iter().skip(start).take(vis).enumerate() {
        let y = area.y + 6.0 + k as f32 * 44.0;
        let k_ = &c.world.clubs[cl];
        if ui.hover(Rect::new(r.x + 6.0, y, r.w - 12.0, 40.0)) { rrect(r.x + 6.0, y, r.w - 12.0, 40.0, 6.0, Theme::erhoehung()); }
        let (c1, _) = club_color(c, cl);
        ui.club_crest(r.x + 30.0, y + 20.0, 15.0, k_.id, c1, &k_.short);
        ui.text_semi(&ui.fit(&k_.name, 16.0, 280.0), r.x + 60.0, y + 26.0, 16.0, Theme::text());
        ui.text(&ui.fit(&c.world.leagues[k_.league].name, 14.0, 210.0), r.x + 360.0, y + 26.0, 14.0, Theme::text2());
        ui.text(&c.world.rank_of(cl).to_string(), r.x + 580.0, y + 26.0, 14.0, Theme::text());
        ui.text(t(&format!("tm.persona.{}", c.persona_von(cl))), r.x + 650.0, y + 26.0, 14.0, Theme::text2()); // roh
        ui.text(&money(round_money(c.world.revenue(cl) * 0.22)), r.x + 800.0, y + 26.0, 14.0, Theme::text());
        ui.text(&crate::tf!("jv.platz", k_.goal_pos), r.x + 940.0, y + 26.0, 14.0, Theme::text2());
        let bx = Rect::new(r.x + r.w - 190.0, y + 5.0, 176.0, 30.0);
        if ok {
            if ui.button(bx, t("lb.antwort.bewerben"), true) { match c.bewerben(cl) { Ok(()) => v.toast(t("ui.pages.bewerbung_laeuft_fragen_im_posteingang"), true), Err(e) => v.toast(&e, false) } }
        } else {
            let w = if !frei { t("tm.wackelt") } else { t(&format!("tm.grund.{grund}")) }; // roh
            ui.text_right(w, r.x + r.w - 20.0, y + 26.0, 13.0, if frei { Theme::gedaempft() } else { Theme::achtung() });
        }
    }
}

fn trainer(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("menu.trainerprofil.reiter.4"), crate::inhalt::t("tm.reiter")]);
    if v.tab == 1 { trainermarkt(ui, v, c); return; }
    ui.title(&c.coach.to_uppercase(), x0() + 8.0, Y0 - 14.0, 30.0, Theme::text());
    let y = Y0;
    let w3 = (cbreite() - 24.0) / 3.0;
    kpi(ui, Rect::new(x0() + 4.0, y, w3, 96.0), crate::inhalt::t("ui.wort.gross_ruf"), &format!("{:.0}", c.mgr_rep), if c.mgr_rep >= 70.0 { crate::inhalt::t("ui.pages.grosser_name") } else if c.mgr_rep >= 50.0 { crate::inhalt::t("ui.pages.bekannter_name") } else if c.mgr_rep >= 30.0 { crate::inhalt::t("ui.pages.solider_ruf") } else { crate::inhalt::t("ui.pages.kaum_bekannt") }, Theme::akzent());
    kpi(ui, Rect::new(x0() + 16.0 + w3, y, w3, 96.0), crate::inhalt::t("ui.wort.gross_gehalt"), &money(c.mgr_wage), crate::inhalt::t("welt.pro_jahr"), Theme::gold());
    kpi(ui, Rect::new(x0() + 28.0 + 2.0 * w3, y, w3, 96.0), crate::inhalt::t("ui.pages.eigenes_konto"), &money(c.mgr_money), crate::inhalt::t("ui.wort.angespart"), Theme::gut());
    let r = Rect::new(x0() + 4.0, y + 110.0, cbreite() * 0.5 - 6.0, hoehe() - y - 126.0);
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
    let r2 = Rect::new(x0() + cbreite() * 0.5 + 10.0, y + 110.0, cbreite() * 0.5 - 6.0, hoehe() - y - 126.0);
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
/// Search profiles (suchprofil.rs): editor on the left, saved profiles and presets, the hits on the right; a scout can take the profile along.
fn suchprofile_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::suchprofil::{ATTRIBUTE, POSITIONEN};
    let t = crate::inhalt::t;
    let lw = cbreite() * 0.42;
    let r = Rect::new(x0() + 4.0, Y0, lw, hoehe() - Y0 - 16.0);
    ui.card_titled(r, t("sup.neu"), Theme::akzent());
    let k = &mut v.sp_edit;
    let mut y = r.y + 56.0;
    ui.text_semi(t("sup.pos"), r.x + 14.0, y + 4.0, 14.0, Theme::text2());
    y += 12.0;
    let bw = (r.w - 28.0 - 12.0 * 6.0) / 13.0;
    for (i, ps) in POSITIONEN.iter().enumerate() {
        let br = Rect::new(r.x + 14.0 + i as f32 * (bw + 6.0), y, bw, 28.0);
        let an = k.pos.iter().any(|x| x == ps);
        rrect(br.x, br.y, br.w, br.h, 6.0, if an { Theme::akzent() } else { Theme::ueberlagerung() });
        ui.text_center(pos_de(ps), br.x + br.w / 2.0, br.y + 19.0, 13.0, if an { Theme::auf_akzent() } else { Theme::text2() });
        if ui.hit(br) { if an { k.pos.retain(|x| x != ps); } else { k.pos.push(ps.to_string()); } }
    }
    y += 44.0;
    // the number criteria as sliders; 0 = any
    let egal = t("sup.egal");
    let zeilen: [(&str, i32, i32, String); 7] = [
        (t("sup.alter_min"), k.alter_min as i32, 40, if k.alter_min == 0 { egal.into() } else { k.alter_min.to_string() }),
        (t("sup.alter_max"), k.alter_max as i32, 40, if k.alter_max == 0 { egal.into() } else { k.alter_max.to_string() }),
        (t("sup.staerke_min"), k.staerke_min as i32, 95, if k.staerke_min == 0 { egal.into() } else { k.staerke_min.to_string() }),
        (t("sup.potenzial_min"), k.pot_min as i32, 99, if k.pot_min == 0 { egal.into() } else { k.pot_min.to_string() }),
        (t("sup.vertrag"), k.vertrag_monate as i32, 36, if k.vertrag_monate == 0 { egal.into() } else { crate::tf!("sup.monate", k.vertrag_monate) }),
        (t("sup.wert_max"), k.wert_max as i32, 200, if k.wert_max == 0 { egal.into() } else { crate::tf!("sup.mio", k.wert_max) }),
        (t("sup.gehalt_max"), k.gehalt_max as i32, 30, if k.gehalt_max == 0 { egal.into() } else { crate::tf!("sup.mio", k.gehalt_max) }),
    ];
    for (i, (n, wert, max, txt)) in zeilen.iter().enumerate() {
        ui.text(n, r.x + 14.0, y + 14.0, 15.0, Theme::text2());
        ui.text_right(txt, r.x + r.w - 14.0, y + 14.0, 15.0, Theme::text());
        if let Some(x) = ui.slider(Rect::new(r.x + 200.0, y + 4.0, r.w - 300.0, 14.0), *wert, 0, *max) {
            let x = x as u32;
            match i { 0 => k.alter_min = x as u8, 1 => k.alter_max = x as u8, 2 => k.staerke_min = x as u8, 3 => k.pot_min = x as u8, 4 => k.vertrag_monate = x as u8, 5 => k.wert_max = x, _ => k.gehalt_max = x }
        }
        y += 34.0;
    }
    ui.text(t("sup.fuss"), r.x + 14.0, y + 20.0, 15.0, Theme::text2());
    if let Some(s) = ui.segment(Rect::new(r.x + 200.0, y + 2.0, r.w - 214.0, 28.0), &[egal, t("sup.fuss.rechts"), t("sup.fuss.links")], k.fuss as usize) { k.fuss = s as u8; }
    y += 42.0;
    // one ability minimum: the name cycles on click
    ui.text(t("sup.werte"), r.x + 14.0, y + 14.0, 15.0, Theme::text2());
    y += 22.0;
    let (ai, am) = k.attr.first().copied().unwrap_or((0, 0));
    let an = crate::inhalt::t(&format!("faehigkeit.{}", crate::entwicklung::NAMEN[ATTRIBUTE[ai as usize % ATTRIBUTE.len()]])); // roh
    if ui.button(Rect::new(r.x + 14.0, y, 180.0, 28.0), an, false) { let n = ((ai as usize + 1) % ATTRIBUTE.len()) as u8; k.attr = vec![(n, am)]; }
    ui.text_right(&if am == 0 { egal.to_string() } else { am.to_string() }, r.x + r.w - 14.0, y + 20.0, 15.0, Theme::text());
    if let Some(x) = ui.slider(Rect::new(r.x + 200.0, y + 8.0, r.w - 300.0, 14.0), am as i32, 0, 95) { k.attr = if x == 0 { vec![] } else { vec![(ai, x as u8)] }; }
    y += 46.0;
    let kk = k.clone();
    if ui.button(Rect::new(r.x + 14.0, y, 200.0, 32.0), t("sup.speichern"), true) {
        let mut p = kk.clone();
        if p.name.is_empty() || c.profil_vorlagen().iter().any(|x| x.name == p.name) { p.name = crate::tf!("sup.name_vorgabe", c.suchprofile.len() + 1); }
        c.suchprofile.retain(|x| x.name != p.name);
        c.suchprofile.push(p.clone());
        if c.suchprofile.len() > 8 { c.suchprofile.remove(0); }
        v.sp_edit.name = p.name.clone();
        v.toast(&crate::tf!("sup.ok.gespeichert", p.name), true);
    }
    if ui.button(Rect::new(r.x + 224.0, y, 220.0, 32.0), t("sup.beauftragen"), false) {
        let mut p = kk.clone();
        if p.name.is_empty() { p.name = crate::tf!("sup.name_vorgabe", c.suchprofile.len() + 1); }
        match c.profil_beauftragen(&p, 42) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
    }
    // saved profiles and presets as chips
    let r2 = Rect::new(x0() + lw + 16.0, Y0, cbreite() - lw - 12.0, 140.0);
    ui.card_titled(r2, t("sup.meine"), Theme::gold());
    let mut x = r2.x + 14.0;
    let mut yy = r2.y + 54.0;
    let mut weg = None;
    let alle: Vec<(bool, crate::suchprofil::Suchprofil)> = c.suchprofile.iter().cloned().map(|p| (true, p)).chain(c.profil_vorlagen().into_iter().map(|p| (false, p))).collect();
    for (i, (eigen, p)) in alle.iter().enumerate() {
        let w = ui.measure(&p.name, 14.0) + 28.0 + if *eigen { 22.0 } else { 0.0 };
        if x + w > r2.x + r2.w - 14.0 { x = r2.x + 14.0; yy += 38.0; }
        let br = Rect::new(x, yy, w, 30.0);
        let an = v.sp_edit == *p;
        rrect(br.x, br.y, br.w, br.h, 15.0, if an { Theme::akzent() } else if *eigen { Theme::ueberlagerung() } else { Theme::erhoehung() });
        ui.text(&p.name, br.x + 14.0, br.y + 20.0, 14.0, if an { Theme::auf_akzent() } else { Theme::text() });
        if *eigen {
            let xr = Rect::new(br.x + br.w - 26.0, br.y + 4.0, 22.0, 22.0);
            ui.text_center("×", xr.x + 11.0, xr.y + 16.0, 15.0, if an { Theme::auf_akzent() } else { Theme::gedaempft() }); // roh
            if ui.hit(xr) { weg = Some(i); continue; }
        }
        if ui.hit(br) { v.sp_edit = p.clone(); }
        x += w + 8.0;
    }
    if let Some(i) = weg { let n = c.suchprofile[i].name.clone(); c.suchprofile.remove(i); v.toast(&crate::tf!("sup.ok.geloescht", n), true); }
    if alle.is_empty() { ui.paragraph(t("sup.keine"), r2.x + 14.0, r2.y + 60.0, 15.0, r2.w - 28.0, Theme::text2()); }
    // hits, cached per profile and day
    let r3 = Rect::new(r2.x, r2.y + r2.h + 10.0, r2.w, hoehe() - r2.y - r2.h - 26.0);
    ui.card_titled(r3, t("sup.treffer"), Theme::akzent());
    if v.sp_treffer.0 != v.sp_edit || v.sp_treffer.1 != c.abs() { v.sp_treffer = (v.sp_edit.clone(), c.abs(), c.profil_treffer(&v.sp_edit, 200)); }
    let liste = v.sp_treffer.2.clone();
    if liste.is_empty() { ui.paragraph(t("sup.keine_treffer"), r3.x + 14.0, r3.y + 60.0, 15.0, r3.w - 28.0, Theme::text2()); return; }
    let area = Rect::new(r3.x, r3.y + 44.0, r3.w, r3.h - 48.0);
    let vis = ((area.h - 8.0) / 34.0) as usize;
    let start = scroll_rows(ui, v, area, liste.len(), vis);
    for (k, &p) in liste.iter().skip(start).take(vis).enumerate() {
        let pl = &c.world.players[p];
        let rr = Rect::new(area.x + 6.0, area.y + k as f32 * 34.0, area.w - 12.0, 32.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
        let cy = rr.y + 22.0;
        ui.player_photo(rr.x + 16.0, rr.y + 16.0, 12.0, pl.id, &pl.name);
        ui.text_semi(&ui.fit(&pl.name, 16.0, 168.0), rr.x + 34.0, cy, 16.0, Theme::text());
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { t("ui.wort.vereinslos").into() };
        let cw = rr.w - 270.0 - 210.0 - 10.0;
        if cw > 40.0 { ui.text(&ui.fit(&club, 14.0, cw), rr.x + 210.0, cy, 14.0, Theme::text2()); }
        ui.text(&pl.age.to_string(), rr.x + rr.w - 270.0, cy, 15.0, Theme::text());
        ui.pos_tag(rr.x + rr.w - 238.0, cy - 5.0, pos_de(&pl.pos));
        strength_cell(ui, c, p, rr.x + rr.w - 150.0, cy - 6.0);
        ui.text_right(&money(pl.value), rr.x + rr.w - 8.0, cy, 15.0, Theme::text());
        if ui.hit(rr) { v.sel_player = Some(p); v.go(Page::Spielerprofil); return; }
    }
}

/// The numbers the free search can sort by (per 90 or a quote).
const MARKT_ZAHLEN: [(&str, fn(&crate::statistik::Stat) -> Option<f32>); 8] = [
    (crate::id("einfluss"), crate::statistik::einfluss90),
    (crate::id("xg"), |s| crate::statistik::pro90(s, crate::statistik::XG)), (crate::id("xa"), |s| crate::statistik::pro90(s, crate::statistik::XA)),
    (crate::id("schluesselpaesse"), |s| crate::statistik::pro90(s, crate::statistik::SCHLUESSEL)), ("prog_paesse", |s| crate::statistik::pro90(s, crate::statistik::PROG_PAESSE)),
    (crate::id("dribblings"), |s| crate::statistik::pro90(s, crate::statistik::DRIBB)), (crate::id("tacklings"), |s| crate::statistik::pro90(s, crate::statistik::TACKL)),
    (crate::id("abgefangen"), |s| crate::statistik::pro90(s, crate::statistik::ABGEF)),
];
/// Whose extended numbers we see (Konzept Spielerstatistiken 8.1): own league, detailed leagues, players known from 40.
pub fn markt_erweitert(c: &Career, p: usize) -> bool {
    let pl = &c.world.players[p];
    if pl.team < 0 { return false; }
    let l = c.world.clubs[pl.team as usize].league;
    l == c.league() || c.know_of(p) >= 0.40 || c.analyst_ligen().contains(&l)
}

/// How well a player plays a PES position: 2 natural (his main position counts), 1 learned, 0 not.
fn pos_stufe(p: &crate::world::Player, k: usize) -> u8 {
    if crate::pesdb::pos_index(&p.pos) as usize == k { return 2; }
    p.pp.get(k).copied().unwrap_or(0)
}
fn transfermarkt(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("menu.transfermarkt.reiter.0"), crate::inhalt::t("gl.chip.transferliste"), crate::inhalt::t("gl.chip.vertragslos"), crate::inhalt::t("sup.meine")]);
    if v.tab == 3 { suchprofile_seite(ui, v, c); return; }
    let w4 = (cbreite() - 36.0) / 4.0;
    let y = Y0;
    kpi(ui, Rect::new(x0() + 4.0, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_transferbudget"), &money(c.transfer_budget), crate::inhalt::t("gl.bk.transfer.unter"), Theme::gold());
    kpi(ui, Rect::new(x0() + 16.0 + w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_kader"), &format!("{} / 40", c.squad().len()), crate::inhalt::t("gl.bk.kader.unter"), Theme::akzent());
    kpi(ui, Rect::new(x0() + 28.0 + 2.0 * w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_gehaltssumme"), &money(c.world.wage_bill(c.own)), crate::inhalt::t("gl.bk.gehaelter.unter"), Theme::gut());
    kpi(ui, Rect::new(x0() + 40.0 + 3.0 * w4, y, w4, 72.0), crate::inhalt::t("ui.wort.gross_kontostand"), &money(c.club().balance), crate::inhalt::t("bereich.verein"), Theme::gold());
    let y = y + 82.0;
    let wtxt = match c.in_window() { Some((_, b)) => crate::tf!("tv.fenster.frist", b - c.world.day, date::short(c.world.season, b)), None => { let next = WINDOWS.iter().find(|(a, _)| *a > c.world.day).map(|(a, _)| date::short(c.world.season, *a)).unwrap_or_else(|| crate::inhalt::t("ui.pages.sommer").into()); crate::tf!("ui.pages.transferfenster_geschlossen_es_oeffnet_a", next) } };
    ui.card(Rect::new(x0() + 4.0, y, cbreite(), 38.0));
    ui.title(&wtxt, x0() + 18.0, y + 27.0, 20.0, Theme::text());
    let y = y + 46.0;
    let si = Rect::new(x0() + 4.0, y, 330.0, 36.0);
    if ui.hit(si) { v.market_focus = true; } else if ui.clicked { v.market_focus = false; }
    let focus = v.market_focus;
    ui.input(si, &mut v.market_text, focus, crate::inhalt::t("sup.suche.platz"));
    if let Some(s) = ui.segment(Rect::new(x0() + 344.0, y, 300.0, 36.0), &[crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("ui.wort.gross_tw"), crate::inhalt::t("ui.wort.gross_abw"), crate::inhalt::t("ui.wort.gross_mit"), crate::inhalt::t("ui.wort.gross_ang")], v.market_pos) { v.market_pos = s; v.scroll = 0.0; v.scrolls.clear(); }
    if let Some(s) = ui.segment(Rect::new(x0() + 654.0, y, 220.0, 36.0), &[crate::inhalt::t("ui.pages.alle_ligen"), crate::inhalt::t("ui.pages.eigene_liga")], v.market_scope) { v.market_scope = s; v.scroll = 0.0; v.scrolls.clear(); }
    // a statistic to sort by: only players whose extended numbers we see, at least 900 minutes (Konzept 5.1, Statistik 8.1/8.4)
    {
        let zr = Rect::new(x0() + 884.0, y, (cbreite() - 884.0 + x0() - 4.0).max(180.0), 36.0);
        let name = if v.market_stat == 0 { crate::inhalt::t("ui.markt.zahl_keine").to_string() } else { crate::tf!("ui.markt.zahl", crate::inhalt::t(&format!("ui.stat.{}", MARKT_ZAHLEN[v.market_stat - 1].0))) };
        if ui.button(zr, &ui.fit(&name, 14.0, zr.w - 12.0), v.market_stat > 0) { v.market_stat = (v.market_stat + 1) % (MARKT_ZAHLEN.len() + 1); v.scroll = 0.0; v.scrolls.clear(); }
    }
    // Fabian 23.11.: age and value as buttons instead of the short slow slider; value "up to" in million, 0 = any
    let y = y + 46.0;
    const ALTER: [i32; 7] = [40, 19, 21, 23, 25, 29, 32];
    const WERT: [i32; 8] = [0, 1, 5, 10, 25, 50, 100, 200];
    let alle = crate::inhalt::t("ui.wort.alle");
    let at: Vec<String> = ALTER.iter().map(|&a| if a == 40 { alle.to_string() } else { format!("≤ {a}") }).collect();
    ui.text(crate::inhalt::t("ui.wort.gross_alter"), x0() + 4.0, y + 23.0, 14.0, Theme::text2());
    let bw = cbreite() - 4.0;
    let aw = (bw - 160.0) * 0.45;
    if let Some(i) = ui.segment(Rect::new(x0() + 64.0, y, aw, 36.0), &at.iter().map(|s| s.as_str()).collect::<Vec<_>>(), ALTER.iter().position(|&a| a == v.market_age).unwrap_or(0)) { v.market_age = ALTER[i]; v.scroll = 0.0; v.scrolls.clear(); }
    let wt: Vec<String> = WERT.iter().map(|&w| if w == 0 { alle.to_string() } else { format!("≤ {w}") }).collect();
    let wx = x0() + 64.0 + aw + 16.0;
    ui.text(crate::inhalt::t("ui.pages.wert_mio"), wx, y + 23.0, 14.0, Theme::text2());
    if let Some(i) = ui.segment(Rect::new(wx + 80.0, y, x0() + bw - wx - 80.0, 36.0), &wt.iter().map(|s| s.as_str()).collect::<Vec<_>>(), WERT.iter().position(|&w| w == v.market_price).unwrap_or(0)) { v.market_price = WERT[i]; v.scroll = 0.0; v.scrolls.clear(); }
    // the exact position, and "can play" (learned or natural) against "plays it well" (natural)
    let pt: Vec<&str> = std::iter::once(alle).chain(crate::pesdb::POS.iter().map(|p| crate::world::pos_de(p))).collect();
    let y = y + 46.0;
    let pr = Rect::new(x0() + 4.0, y, bw - 250.0, 36.0);
    if let Some(i) = ui.segment(pr, &pt, v.market_genau) { v.market_genau = i; v.scroll = 0.0; v.scrolls.clear(); }
    if let Some(i) = ui.segment(Rect::new(pr.x + pr.w + 10.0, y, 240.0, 36.0), &[crate::inhalt::t("ui.pages.kann_spielen"), crate::inhalt::t("ui.pages.kann_gut_spielen")], v.market_gut) { v.market_gut = i; v.scroll = 0.0; v.scrolls.clear(); }
    let key = format!("{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}", v.tab, v.market_pos, v.market_scope, v.market_age, v.market_price, v.market_text.to_lowercase(), c.abs(), c.world.players.len(), v.market_stat, v.market_genau, v.market_gut);
    if key != v.market_key {
        v.market_key = key;
        let q = v.market_text.to_lowercase();
        let ol = c.league();
        let mut list: Vec<usize> = (0..c.world.players.len()).filter(|&i| {
            let p = &c.world.players[i];
            if p.team == c.own as i32 || p.team == -2 || p.youth > 0 { return false; }
            if v.tab == 1 && !p.listed && c.status_von(i).map_or(true, |s| s.status != crate::kimarkt::Status::Leihliste) { return false; }
            if v.tab == 2 && p.team != -1 { return false; }
            if v.tab != 2 && p.team < 0 { return false; }
            let part = part_of(&p.pos);
            if v.market_pos > 0 && part + 1 != v.market_pos { return false; }
            if v.market_scope == 1 && (p.team < 0 || c.world.clubs[p.team as usize].league != ol) { return false; }
            if p.age as i32 > v.market_age || (v.market_price > 0 && p.value > v.market_price as f64 * 1e6) { return false; }
            if v.market_genau > 0 { let k = v.market_genau - 1; if pos_stufe(p, k) < 1 + v.market_gut as u8 { return false; } }
            if !q.is_empty() && !p.name.to_lowercase().contains(&q) && !(p.team >= 0 && c.world.clubs[p.team as usize].name.to_lowercase().contains(&q)) { return false; }
            true
        }).collect();
        if v.market_stat > 0 {
            let f = MARKT_ZAHLEN[v.market_stat - 1].1;
            let mut mit: Vec<(usize, f32)> = list.into_iter().filter(|&p| markt_erweitert(c, p)).filter_map(|p| { let (st, _) = crate::statistik::saison_von(&c.world, p); (crate::statistik::get(&st, crate::statistik::MIN) >= 900).then_some(())?; f(&st).map(|x| (p, x)) }).collect();
            mit.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            list = mit.into_iter().map(|x| x.0).collect();
        } else {
            list.sort_by(|a, b| { let (x, y) = (c.ovr_range(*a), c.ovr_range(*b)); (y.0 + y.1).partial_cmp(&(x.0 + x.1)).unwrap() });
        }
        list.truncate(2000);
        v.market_cache = list;
    }
    let mut list = v.market_cache.clone();
    sortieren(v, 20, &mut list, |p, col| {
        let pl = &c.world.players[p];
        match col {
            0 => Sk::S(pl.name.to_lowercase()),
            1 => Sk::S(if pl.team >= 0 { c.world.clubs[pl.team as usize].name.to_lowercase() } else { String::new() }),
            2 => Sk::N(pl.age as f32), 3 => Sk::N(-(crate::pesdb::pos_index(&pl.pos) as f32)),
            4 => { let (a, b) = c.ovr_range(p); Sk::N(a + b) }
            5 => Sk::N(pl.value as f32), _ => Sk::N(c.moneyball(p) as f32),
        }
    });
    let y = y + 46.0;
    let lw = cbreite() - 300.0;
    let area = Rect::new(x0() + 4.0, y, lw, hoehe() - y - 16.0);
    ui.card(area);
    sort_head(ui, v, 20, area.x, area.y + 22.0, &[(crate::inhalt::t("ui.wort.gross_name"), 16.0), (crate::inhalt::t("ui.wort.gross_verein"), 250.0), (crate::inhalt::t("ui.wort.gross_alter"), 470.0), (crate::inhalt::t("ui.wort.gross_pos"), 530.0), (crate::inhalt::t("ui.pages.staerke"), 590.0), (crate::inhalt::t("ui.wort.gross_marktwert"), 680.0), (crate::inhalt::t("ui.wort.gross_preis_leist"), 800.0)]);
    let vis = ((area.h - 40.0) / 36.0) as usize;
    let start = scroll_rows(ui, v, area, list.len(), vis);
    ui.text_right(&crate::tf!("zentrale.k.spieler.viele", list.len()), x0() + cbreite() - 14.0, Y0 - 20.0, 16.0, Theme::text2());
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
        if v.market_stat > 0 {
            let (st, hoch) = crate::statistik::saison_von(&c.world, p);
            if let Some(x) = MARKT_ZAHLEN[v.market_stat - 1].1(&st) { ui.text(&format!("{x:.2}{}", if hoch { " ~" } else { "" }), r.x + 820.0, cy, 16.0, Theme::akzent()); }
        } else {
            let mb = c.moneyball(p);
            ui.text(&format!("{:+.0}", mb), r.x + 820.0, cy, 16.0, if mb >= 3.0 { Theme::gut() } else if mb <= -3.0 { Theme::problem() } else { Theme::text2() });
        }
        // labels (Konzept 5.1): over budget, would not come, on the shortlist, report there
        let schild = if c.merkliste.contains(&p) { Some((crate::inhalt::t("ui.markt.schild.merkliste"), Theme::akzent())) }
            else if let Some(k) = c.machbar(p, false) { Some((crate::inhalt::t(k), Theme::problem())) }
            else if c.know_of(p) >= crate::regeln::f("scouting", "wissen.bericht_ab") / 100.0 { Some((crate::inhalt::t("ui.markt.schild.bericht"), Theme::gut())) } else { None };
        if let Some((txt, col)) = schild { ui.text_right(&ui.fit(txt, 13.0, r.w - 900.0), r.x + r.w - 10.0, cy, 13.0, col); }
        if ui.hit(r) { v.sel_player = Some(p); }
    }
    // detail panel
    let dr = Rect::new(x0() + lw + 16.0, y - 0.0, 288.0 - 4.0, hoehe() - y - 16.0);
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
    // why his club lets him go (kimarkt.rs: transfer or loan list and its reason)
    let liste = c.status_von(p).filter(|s| matches!(s.status, crate::kimarkt::Status::Transferliste | crate::kimarkt::Status::Leihliste)).map(|s| {
        let cn = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { String::new() };
        let g = if s.grund.is_empty() { String::new() } else { crate::tf!(&format!("ui.km.grund.{}", s.grund), pl.name.clone(), cn) }; // roh
        format!("{} {g} ", crate::inhalt::t(if s.status == crate::kimarkt::Status::Leihliste { "tf.liste.leihe" } else { "tf.liste.verkauf" }))
    }).unwrap_or_default();
    // the scout's verdict (scoutbericht.rs) once he knows enough
    let urteil = if c.know_of(p) >= crate::regeln::f("scouting", "wissen.bericht_ab") / 100.0 { format!("{} ", crate::inhalt::t(&format!("ui.sb.urteil.{}", c.scout_urteil(p).urteil))) } else { String::new() };
    let pro = urteil + &liste + &if mid > best_own { crate::tf!("ui.pages.verstaerkt_die_position_wertung_gegen", format!("{:.0}", mid), format!("{:.0}", best_own)) } else if pl.age <= 21 && pb > best_own { crate::inhalt::t("ui.pages.junges_talent_mit_luft_nach").to_string() } else { crate::tf!("ui.pages.passt_ins", ideas_t()[c.tactic.idea()], fit_text(c.system_fit(p))) };
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
    // from 1 January a player with an expiring contract can be signed for the summer, without a fee (termine.rs)
    let vv = c.vorvertrag_moeglich(p);
    if ui.button_gold(Rect::new(dr.x + 10.0, dr.y + unten, dr.w - 20.0, 42.0), if vv { crate::inhalt::t("vv.knopf") } else { crate::inhalt::t("ui.wort.gross_verhandeln") }) {
        match if vv { c.vorvertrag_oeffnen(p) } else { c.open_neg(p) } { Ok(i) => { v.modal = Modal::Neg(i); ui.play(crate::id("angebot")); } Err(e) => { v.toast(&e, false); ui.play(crate::id("fehler")); } }
    }
    // a watch assignment with a free scout (auftraege.rs)
    let watched = c.auftraege.iter().any(|x| !x.fertig && x.spieler == Some(p));
    if ui.button(Rect::new(dr.x + 10.0, dr.y + unten + 48.0, (dr.w - 28.0) / 2.0, 32.0), if watched { crate::inhalt::t("ui.scout.laeuft") } else { crate::inhalt::t("ui.scout.schicken") }, false) && !watched {
        let belegt: Vec<String> = c.auftraege.iter().filter(|x| !x.fertig).map(|x| x.scout.clone()).collect();
        match c.scouts.iter().find(|s| !belegt.contains(&s.name)).map(|s| s.name.clone()) {
            Some(scout) => { let x = crate::auftraege::Auftrag { art: crate::auftraege::BEOBACHTEN, scout, spieler: Some(p), ..Default::default() }; match c.auftrag_anlegen(x, 21) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
            None => v.toast(crate::inhalt::t("ui.pages.alle_scouts_haben_schon_einen"), false),
        }
    }
    if ui.button(Rect::new(dr.x + 18.0 + (dr.w - 28.0) / 2.0, dr.y + unten + 48.0, (dr.w - 28.0) / 2.0, 32.0), crate::inhalt::t("aktion.profil"), false) { v.modal = Modal::Player(p); }
}

/// Konzept Scouting 5.2 / Teil 7 "Auffällige Zahlen": the analyst's list of the week, each with "send a scout".
fn analyst_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(r, t("ui.analyst.titel"), Theme::akzent());
    let Some(s) = c.analyst() else {
        ui.paragraph(t(if c.analyst_erlaubt() { "ui.analyst.keiner" } else { "ui.analyst.zu_klein" }), r.x + 14.0, r.y + 60.0, 15.0, r.w - 28.0, Theme::text2());
        return;
    };
    let ligen = c.analyst_ligen().len();
    ui.text(&crate::tf!("ui.analyst.kopf", c.scouts[s].name.clone(), ligen), r.x + 14.0, r.y + 58.0, 14.0, Theme::text2());
    if c.analyst_liste.is_empty() { ui.paragraph(t("ui.analyst.leer"), r.x + 14.0, r.y + 90.0, 15.0, r.w - 28.0, Theme::text2()); return; }
    player_row_head(ui, r.x, r.y + 86.0, &[(t("ui.wort.gross_name"), 14.0), (t("ui.wort.gross_verein"), r.w * 0.30), (t("ui.stat.einfluss"), r.w * 0.55), (t("ui.markt.wert"), r.w * 0.70)]);
    let mut y = r.y + 116.0;
    let mut klick = None;
    let belegt: Vec<String> = c.auftraege.iter().filter(|x| !x.fertig).map(|x| x.scout.clone()).collect();
    for &p in c.analyst_liste.clone().iter() {
        let pl = c.world.players[p].clone();
        let rr = Rect::new(r.x + 8.0, y - 20.0, r.w * 0.80, 30.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
        ui.player_photo(r.x + 26.0, y - 5.0, 11.0, pl.id, &pl.name);
        ui.text(&ui.fit(&format!("{} · {} · {}", pl.name, pos_de(&pl.pos), pl.age), 15.0, r.w * 0.27 - 30.0), r.x + 44.0, y, 15.0, Theme::text());
        let (club, liga) = if pl.team >= 0 { let k = pl.team as usize; (c.world.clubs[k].name.clone(), c.world.clubs[k].league) } else { (String::new(), usize::MAX) };
        ui.text(&ui.fit(&club, 14.0, r.w * 0.24), r.x + r.w * 0.30, y, 14.0, Theme::text2());
        let (st, _) = crate::statistik::saison_von(&c.world, p);
        if let Some(e) = crate::statistik::einfluss90(&st) {
            let um = if liga < c.world.leagues.len() { crate::statistik::umrechnen(&c.world, e, liga, c.league(), 0) } else { e };
            ui.text(&crate::tf!("ui.analyst.wert", format!("{e:.2}"), format!("{um:.2}")), r.x + r.w * 0.55, y, 14.0, Theme::akzent());
        }
        ui.text(&money(pl.value), r.x + r.w * 0.70, y, 14.0, Theme::text());
        if ui.hit(rr) { klick = Some(p); }
        let laeuft = c.auftraege.iter().any(|x| !x.fertig && x.spieler == Some(p));
        if ui.button(Rect::new(r.x + r.w - 190.0, y - 20.0, 176.0, 28.0), t(if laeuft { "ui.scout.laeuft" } else { "ui.scout.schicken" }), false) && !laeuft {
            match c.scouts.iter().enumerate().find(|(i, x)| *i != s && !belegt.contains(&x.name)).map(|(_, x)| x.name.clone()) {
                Some(scout) => { let x = crate::auftraege::Auftrag { art: crate::auftraege::BEOBACHTEN, scout, spieler: Some(p), ..Default::default() }; match c.auftrag_anlegen(x, 21) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
                None => v.toast(t("ui.pages.alle_scouts_haben_schon_einen"), false),
            }
        }
        y += 36.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
}

/// The colour of a scout's verdict (Konzept 4.3): sign now … do not sign.
pub fn urteil_farbe(u: usize) -> Color { [hex(0x1E9E5A), Theme::gut(), hex(0x8BC34A), Theme::achtung(), Theme::akzent(), Theme::problem()][u.min(5)] }

/// The form for a new assignment (kept between frames).
#[derive(Clone)]
struct AuftragForm { art: usize, gruppe: usize, alter: (u8, u8), liga: usize, scout: usize, knapp: bool, lang: bool, spieler: usize }
thread_local! { static FORM: std::cell::RefCell<AuftragForm> = std::cell::RefCell::new(AuftragForm { art: 0, gruppe: 2, alter: (17, 28), liga: 0, scout: 0, knapp: false, lang: false, spieler: 0 }); }
const GRUPPEN: [&str; 9] = [crate::id("tw"), crate::id("iv"), crate::id("av"), crate::id("dm"), crate::id("zm"), crate::id("om"), crate::id("fl"), crate::id("hs"), crate::id("st")];

/// Scouting assignments (konzepte/scouting.md Teil 3 and 7): a new one on the left, the running ones with their proposals
/// on the right. Watching one player and a second opinion start from the player profile and the report.
fn auftraege_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let mut fm = FORM.with(|f| f.borrow().clone());
    let l = Rect::new(x0() + 4.0, Y0, cbreite() * 0.38, hoehe() - Y0 - 16.0);
    ui.card_titled(l, t("ui.auftrag.neu_titel"), Theme::akzent());
    let mut y = l.y + 52.0;
    let arten = [t("ui.auftrag.art.suche"), t("ui.auftrag.art.gebiet"), t("ui.auftrag.art.nachfolger")];
    if let Some(i) = ui.segment(Rect::new(l.x + 14.0, y, l.w - 28.0, 32.0), &arten, fm.art) { fm.art = i; }
    y += 46.0;
    // the scout: free ones only
    let frei: Vec<usize> = (0..c.scouts.len()).filter(|&s| !c.auftraege.iter().any(|x| !x.fertig && x.scout == c.scouts[s].name)).collect();
    let zeile = |ui: &mut Ui, y: f32, label: &str, wert: &str| -> (bool, bool) {
        ui.text_semi(label, l.x + 14.0, y + 20.0, 14.0, Theme::text2());
        let links = ui.button(Rect::new(l.x + l.w * 0.38, y, 32.0, 30.0), "<", false);
        ui.text_center(&ui.fit(wert, 14.0, l.w * 0.62 - 90.0), l.x + l.w * 0.38 + (l.w * 0.62 - 14.0) / 2.0, y + 20.0, 14.0, Theme::text());
        let rechts = ui.button(Rect::new(l.x + l.w - 46.0, y, 32.0, 30.0), ">", false);
        (links, rechts)
    };
    let scout_name = frei.get(fm.scout % frei.len().max(1)).map(|&s| c.scouts[s].name.clone()).unwrap_or_else(|| t("ui.auftrag.kein_freier").to_string());
    let (a, b) = zeile(ui, y, t("ui.auftrag.scout"), &scout_name);
    if a { fm.scout = (fm.scout + frei.len().max(1) - 1) % frei.len().max(1); }
    if b { fm.scout = (fm.scout + 1) % frei.len().max(1); }
    y += 40.0;
    // where: the own league first, then the leagues the scout may work in, strongest first
    let s_idx = frei.get(fm.scout % frei.len().max(1)).copied();
    let mut ligen: Vec<usize> = vec![c.league()];
    if let Some(s) = s_idx {
        let mut andere: Vec<usize> = (0..c.world.leagues.len()).filter(|&k| k != c.league() && c.world.leagues[k].teams.len() >= 8 && c.gebiet_erlaubt(s, Some(k))).collect();
        andere.sort_by(|a, b| crate::statistik::liga_staerke(&c.world, *b).partial_cmp(&crate::statistik::liga_staerke(&c.world, *a)).unwrap());
        ligen.extend(andere);
    }
    let mut lg = ligen[fm.liga % ligen.len()];
    // country, then league, each a list with search (Fabian 23.11.)
    ui.text_semi(t("ui.auftrag.gebiet"), l.x + 14.0, y + 24.0, 14.0, Theme::text2());
    let gebiet = ligen_gruppen(c, |k| ligen.contains(&k));
    let gebiet_felder = wettbewerb_felder(c, &gebiet, lg, Rect::new(l.x + l.w * 0.38, y, l.w * 0.62 - 14.0, 36.0));
    if let Some((f, i)) = auswahl_vorher(ui, v, &gebiet_felder) { lg = wahl_umsetzen(&gebiet, lg, f, i); fm.liga = ligen.iter().position(|&k| k == lg).unwrap_or(0); }
    y += 46.0;
    let sq = c.squad();
    match fm.art {
        0 => {
            ui.text_semi(t("ui.auftrag.position"), l.x + 14.0, y + 20.0, 14.0, Theme::text2());
            y += 28.0;
            let namen: Vec<&str> = GRUPPEN.iter().map(|g| t(&format!("ui.auftrag.gruppe.{g}"))).collect();
            if let Some(i) = ui.segment(Rect::new(l.x + 14.0, y, l.w - 28.0, 30.0), &namen, fm.gruppe) { fm.gruppe = i; }
            y += 42.0;
            let (a, b) = zeile(ui, y, t("ui.auftrag.alter_von"), &fm.alter.0.to_string());
            if a { fm.alter.0 = fm.alter.0.saturating_sub(1).max(16); } if b { fm.alter.0 = (fm.alter.0 + 1).min(fm.alter.1); }
            y += 38.0;
            let (a, b) = zeile(ui, y, t("ui.auftrag.alter_bis"), &fm.alter.1.to_string());
            if a { fm.alter.1 = fm.alter.1.saturating_sub(1).max(fm.alter.0); } if b { fm.alter.1 = (fm.alter.1 + 1).min(38); }
            y += 42.0;
            if ui.button(Rect::new(l.x + 14.0, y, l.w - 28.0, 30.0), t("ui.auftrag.knapp"), fm.knapp) { fm.knapp = !fm.knapp; }
            y += 40.0;
        }
        2 => {
            let p = sq.get(fm.spieler % sq.len().max(1)).copied();
            let n = p.map(|p| format!("{} ({})", c.world.players[p].name, pos_de(&c.world.players[p].pos))).unwrap_or_default();
            let (a, b) = zeile(ui, y, t("ui.auftrag.fuer"), &n);
            if a { fm.spieler = (fm.spieler + sq.len().max(1) - 1) % sq.len().max(1); } if b { fm.spieler = (fm.spieler + 1) % sq.len().max(1); }
            y += 42.0;
        }
        _ => {}
    }
    if let Some(i) = ui.segment(Rect::new(l.x + 14.0, y, l.w - 28.0, 30.0), &[t("ui.auftrag.tage14"), t("ui.auftrag.tage28")], fm.lang as usize) { fm.lang = i == 1; }
    y += 44.0;
    y = ui.paragraph(&crate::tf!("ui.auftrag.budget_hinweis", money(c.transfer_frei()), money(c.gehalt_frei())), l.x + 14.0, y, 13.0, l.w - 28.0, Theme::gedaempft()) + 10.0;
    if ui.button_gold(Rect::new(l.x + 14.0, y, l.w - 28.0, 44.0), t("ui.auftrag.starten")) {
        let art = [crate::auftraege::SUCHE, crate::auftraege::GEBIET, crate::auftraege::NACHFOLGER][fm.art];
        let x = crate::auftraege::Auftrag { art, scout: scout_name.clone(), gruppe: if fm.art == 0 { GRUPPEN[fm.gruppe].to_string() } else { String::new() }, alter_min: fm.alter.0, alter_max: fm.alter.1,
            liga: Some(lg), spieler: if fm.art == 2 { sq.get(fm.spieler % sq.len().max(1)).copied() } else { None }, knapp: fm.knapp, ..Default::default() };
        let tage = if fm.lang || art == crate::auftraege::GEBIET { 28 } else { 14 };
        match c.auftrag_anlegen(x, tage) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
    }
    // the squad gap (Konzept 4.4) as nine chips: a click picks the group for a search
    y += 60.0;
    ui.text_semi(t("ui.auftrag.kaderluecke"), l.x + 14.0, y, 14.0, Theme::text2());
    y += 10.0;
    let cw = (l.w - 28.0 - 8.0 * 4.0) / 5.0;
    for (i, g) in GRUPPEN.iter().enumerate() {
        let rr = Rect::new(l.x + 14.0 + (i % 5) as f32 * (cw + 8.0), y + (i / 5) as f32 * 34.0, cw, 28.0);
        let farbe = [Theme::gut(), Theme::achtung(), Theme::problem()][c.kaderluecke(g) as usize];
        rrect(rr.x, rr.y, rr.w, rr.h, 6.0, alpha(farbe, 0.22));
        rrect_line(rr.x, rr.y, rr.w, rr.h, 6.0, farbe);
        ui.text_center(t(&format!("ui.auftrag.gruppe.{g}")), rr.x + rr.w / 2.0, rr.y + 19.0, 13.0, Theme::text());
        if ui.hit(rr) { fm.art = 0; fm.gruppe = i; }
    }
    auswahl_zeichnen(ui, v, &gebiet_felder);
    FORM.with(|f| *f.borrow_mut() = fm);
    // right: the assignments, newest first
    let r = Rect::new(l.x + l.w + 12.0, Y0, cbreite() - l.w - 12.0, hoehe() - Y0 - 16.0);
    ui.card_titled(r, &crate::tf!("ui.auftrag.liste", c.auftraege_laufend(), c.auftraege_max()), Theme::verein());
    if c.auftraege.is_empty() { ui.paragraph(t("ui.auftrag.leer"), r.x + 14.0, r.y + 60.0, 15.0, r.w - 28.0, Theme::text2()); return; }
    let abs = c.abs();
    let mut y = r.y + 52.0;
    let mut klick = None;
    for x in c.auftraege.iter().rev() {
        if y > r.y + r.h - 60.0 { break; }
        let titel = match x.art {
            crate::auftraege::SUCHE => format!("{} · {}", t(&format!("ui.auftrag.gruppe.{}", if x.gruppe.is_empty() { crate::id("st") } else { &x.gruppe })), x.liga.map(|l| c.world.leagues[l].name.clone()).unwrap_or_default()),
            crate::auftraege::GEBIET => x.liga.map(|l| c.world.leagues[l].name.clone()).unwrap_or_default(),
            _ => x.spieler.map(|p| c.world.players[p].name.clone()).unwrap_or_default(),
        };
        rrect(r.x + 10.0, y, r.w - 20.0, 30.0, 6.0, Theme::erhoehung());
        ui.text_semi(&format!("{} · {}", t(&format!("ui.auftrag.art.{}", crate::auftraege::ARTEN[x.art as usize])), titel), r.x + 20.0, y + 20.0, 14.0, Theme::text());
        let stand = if x.fertig { t("ui.auftrag.fertig").to_string() } else { crate::tf!("ui.auftrag.noch_tage", x.ende.saturating_sub(abs), x.scout.clone()) };
        ui.text_right(&stand, r.x + r.w - 20.0, y + 20.0, 13.0, Theme::text2());
        if !x.fertig { ui.bar(r.x + 10.0, y + 28.0, r.w - 20.0, 3.0, (abs.saturating_sub(x.start)) as f32 / (x.ende - x.start).max(1) as f32, Theme::akzent()); }
        y += 40.0;
        let mut vs = x.vorschlaege.clone();
        vs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for &(p, _) in vs.iter() {
            if y > r.y + r.h - 30.0 { break; }
            let b = c.scout_urteil(p);
            let pl = &c.world.players[p];
            let rr = Rect::new(r.x + 14.0, y - 4.0, r.w - 28.0, 26.0);
            if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
            draw_circle(r.x + 26.0, y + 9.0, 6.0, urteil_farbe(b.urteil));
            let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].short.clone() } else { String::new() };
            ui.text(&ui.fit(&format!("{} · {} · {} {}", pl.name, pos_de(&pl.pos), pl.age, club), 14.0, r.w * 0.5), r.x + 40.0, y + 14.0, 14.0, Theme::text());
            ui.text(t(&format!("ui.sb.urteil.{}", b.urteil)), r.x + r.w * 0.56, y + 14.0, 13.0, urteil_farbe(b.urteil));
            let (lo, hi) = c.ovr_range(p);
            ui.text_right(&format!("{lo:.0}–{hi:.0} · {}", money(c.ask_price(p))), r.x + r.w - 20.0, y + 14.0, 13.0, Theme::text2());
            if ui.hit(rr) { klick = Some(p); }
            y += 26.0;
        }
        if vs.is_empty() && !x.fertig && matches!(x.art, crate::auftraege::SUCHE | crate::auftraege::NACHFOLGER) { ui.text(&crate::tf!("ui.auftrag.im_blick", x.im_blick.len()), r.x + 20.0, y + 10.0, 13.0, Theme::gedaempft()); y += 24.0; }
        y += 8.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
}

// ================================================================ Verhandlungen
fn verhandlungen(ui: &mut Ui, v: &mut View, c: &mut Career) {
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
fn vertraege(ui: &mut Ui, v: &mut View, c: &mut Career) {
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

// ================================================================ Scouting
fn scouting(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("ui.pages.scouts_und_auftraege"), crate::inhalt::t("menu.scout_verwaltung.reiter.1"), crate::inhalt::t("as.reiter.berichte"), crate::inhalt::t("ui.auftrag.reiter"), crate::inhalt::t("ui.analyst.reiter")]);
    if v.tab == 3 { auftraege_seite(ui, v, c); return; }
    if v.tab == 4 { analyst_seite(ui, v, c); return; }
    if v.tab == 0 {
        let mut y = Y0;
        let abs = c.abs();
        let n = c.scouts.len();
        // the area of every shown scout as country + league fields (Fabian 23.11.), two fields per scout
        let zeigbar = n.min(((hoehe() - 170.0 - Y0) / 160.0) as usize + 1);
        let gebiete: Vec<Vec<(String, Vec<usize>)>> = (0..zeigbar).map(|s| ligen_gruppen(c, |l| c.gebiet_erlaubt(s, Some(l)))).collect();
        let felder: Vec<Auswahl> = (0..zeigbar).flat_map(|s| wettbewerb_felder(c, &gebiete[s], c.scouts[s].league.unwrap_or(usize::MAX), Rect::new(x0() + 20.0, Y0 + s as f32 * 160.0 + 100.0, 270.0, 36.0))).collect();
        if let Some((f, i)) = auswahl_vorher(ui, v, &felder) { let s = f / 2; let alt = c.scouts[s].league.unwrap_or(usize::MAX); c.scouts[s].league = Some(wahl_umsetzen(&gebiete[s], alt, f % 2, i)).filter(|&l| l != usize::MAX); }
        for s in 0..n {
            if s >= c.scouts.len() { break; }
            let r = Rect::new(x0() + 4.0, y, cbreite(), 150.0);
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
            // Konzept Scouting 3.1/5.2: one scout can work as data analyst (from club level 3), he travels no more
            if c.analyst_erlaubt() {
                let ist = c.scouts[s].p.as_ref().map_or(false, |p| p.aufgabe == 2);
                let frei = c.analyst().map_or(true, |a| a == s);
                if frei && ui.button(Rect::new(r.x + r.w - 330.0, r.y + 102.0, 180.0, 32.0), crate::inhalt::t(if ist { "ui.analyst.ist" } else { "ui.analyst.machen" }), ist) {
                    c.person(s).aufgabe = if ist { 0 } else { 2 };
                    v.toast(crate::inhalt::t(if ist { "ui.analyst.zurueck" } else { "ui.analyst.ok" }), true);
                }
            }
            y += 160.0;
            if y > hoehe() - 170.0 { break; }
        }
        auswahl_zeichnen(ui, v, &felder);
        if y <= hoehe() - 120.0 {
            let r = Rect::new(x0() + 4.0, y, cbreite(), 110.0);
            ui.card_titled(r, crate::inhalt::t("ui.pages.so_arbeiten_scouts"), Theme::akzent());
            ui.paragraph(&crate::tf!("ui.pages.ein_scout_lernt_je_tag", c.scouts_max()), r.x + 14.0, r.y + 56.0, 16.0, r.w - 28.0, Theme::text2());
        }
    } else if v.tab == 1 {
        let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
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
        let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
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
    // Akademie-Konzept: overview, U19, U17, mentors; with a real second team its tab comes last (farm.rs)
    let zweite = c.zweite_mannschaft().is_some();
    let mut reiter = vec![crate::inhalt::t("reiter.uebersicht"), crate::inhalt::t("ui.wort.gross_u19"), crate::inhalt::t("ui.wort.gross_u17"), crate::inhalt::t("ui.jugend.reiter.mentoren")];
    if zweite { reiter.push(crate::inhalt::t("ui.game.zweite_mannschaft")); }
    tabs(ui, v, &reiter);
    match v.tab { 0 => { akademie_uebersicht(ui, v, c); return; } 3 => { mentoren_seite(ui, v, c); return; } 4 if zweite => { zweite_mannschaft_tab(ui, v, c); return; } _ => {} }
    let k = if v.tab == 1 { 1 } else { 0 };
    let g = if k == 1 { 19 } else { 17 };
    let sq = c.youth_squad(g);
    let lw = cbreite() * 0.58;
    let area = Rect::new(x0() + 4.0, Y0, lw, hoehe() - Y0 - 16.0);
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
        if ui.button(Rect::new(r.x + r.w - 350.0, r.y + 3.0, 124.0, 30.0), crate::inhalt::t(if pl.ps.gast { "ui.jugend.gast.beenden" } else { "ui.jugend.gast.knopf" }), pl.ps.gast) {
            match c.gast_setzen(p, !pl.ps.gast) { Ok(t) => v.toast(&t, true), Err(e) => v.toast(&e, false) }
        }
        if pl.ps.mentor.is_some() { ui.text("M", r.x + 470.0, r.y + 24.0, 15.0, Theme::gold()); } // roh
        if ui.button(Rect::new(r.x + r.w - 220.0, r.y + 3.0, 120.0, 30.0), crate::inhalt::t("ui.pages.zu_den_profis"), false) { c.promote(p); v.toast(&crate::tf!("ui.game.rueckt_zu_den_profis_auf", pl.name), true); }
        if ui.button(Rect::new(r.x + r.w - 92.0, r.y + 3.0, 86.0, 30.0), crate::inhalt::t("ui.pages.abgeben"), false) { c.release(p); }
    }
    let rt = Rect::new(x0() + lw + 18.0, Y0, cbreite() - lw - 14.0, hoehe() - Y0 - 16.0);
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

/// Akademie overview: youth strength with its four parts, youth reputation, the training guests and who is ready.
fn akademie_uebersicht(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let jugend_farbe = hex(0x2dd4bf);
    let j = c.jugendstaerke(c.own);
    let lw = cbreite() * 0.42;
    let l = Rect::new(x0() + 4.0, Y0, lw, 330.0);
    ui.card(l);
    ui.text_semi(crate::inhalt::t("ui.jugend.staerke"), l.x + 16.0, l.y + 30.0, 16.0, Theme::text2());
    ui.title(&format!("{:.1}", j.wert), l.x + 16.0, l.y + 86.0, 54.0, jugend_farbe);
    for s in 0..5 { draw_poly(l.x + 140.0 + s as f32 * 24.0, l.y + 72.0, 5, 10.0, -90.0, if s < c.jugend_ruf() as usize { Theme::gold() } else { Theme::ueberlagerung() }); }
    ui.text(crate::inhalt::t("ui.jugend.ruf"), l.x + 130.0, l.y + 56.0, 13.0, Theme::gedaempft());
    let mut y = l.y + 124.0;
    for (key, w, g) in [("ui.jugend.teil.anlagen", j.anlagen, "staerke.anlagen"), ("ui.jugend.teil.stab", j.stab, "staerke.stab"), ("ui.jugend.teil.region", j.region, "staerke.region"), ("ui.jugend.teil.tradition", j.tradition, "staerke.tradition")] {
        ui.text(crate::inhalt::t(key), l.x + 16.0, y + 4.0, 15.0, Theme::text());
        ui.bar(l.x + 160.0, y - 6.0, l.w - 260.0, 10.0, w / 10.0, jugend_farbe);
        ui.text_right(&format!("{:.1} · {:.0} %", w, crate::regeln::f("jugendakademie", g) * 100.0), l.x + l.w - 16.0, y + 4.0, 14.0, Theme::text2());
        y += 34.0;
    }
    if let Some(b) = j.bekannt { ui.text(&crate::tf!("ui.jugend.bekannt", b), l.x + 16.0, y + 6.0, 13.0, Theme::gedaempft()); }
    // the scouting tournament, once a season
    let schon = c.sichtung_saison == c.world.season + 1;
    if ui.button(Rect::new(l.x + l.w - 250.0, l.y + 12.0, 234.0, 32.0), crate::inhalt::t(if schon { "ui.jugend.sichtung.erledigt" } else { "ui.jugend.sichtung.knopf" }), !schon) && !schon {
        match c.sichtungsturnier() { Ok(t) => v.toast(&t, true), Err(e) => v.toast(&e, false) }
    }
    // right: training guests and who is ready for the first team
    let r = Rect::new(x0() + lw + 16.0, Y0, cbreite() - lw - 12.0, hoehe() - Y0 - 16.0);
    ui.card(r);
    let jung: Vec<usize> = c.world.clubs[c.own].squad.iter().copied().filter(|&p| c.world.players[p].youth > 0).collect();
    let gaeste: Vec<usize> = jung.iter().copied().filter(|&p| c.world.players[p].ps.gast).collect();
    ui.text_semi(&crate::tf!("ui.jugend.gaeste", gaeste.len(), crate::regeln::f("jugendakademie", "hochziehen.gast_hoechstens")), r.x + 16.0, r.y + 30.0, 16.0, Theme::text());
    let mut y = r.y + 58.0;
    for &p in &gaeste { let pl = &c.world.players[p]; ui.text(&format!("{} · {} · {:.0}", pl.name, pos_de(&pl.pos), pl.ovr), r.x + 16.0, y, 15.0, Theme::text2()); y += 24.0; }
    y += 14.0;
    ui.text_semi(crate::inhalt::t("ui.jugend.talente"), r.x + 16.0, y, 16.0, Theme::text());
    y += 12.0;
    let mut beste = jung.clone();
    beste.sort_by(|a, b| c.world.players[*b].pot.partial_cmp(&c.world.players[*a].pot).unwrap());
    for &p in beste.iter().take(((r.y + r.h - y - 20.0) / 34.0).max(0.0) as usize) {
        let pl = c.world.players[p].clone();
        let rr = Rect::new(r.x + 8.0, y, r.w - 16.0, 32.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
        ui.pos_tag(rr.x + 6.0, rr.y + 16.0, pos_de(&pl.pos));
        ui.text_semi(&ui.fit(&pl.name, 15.0, 200.0), rr.x + 50.0, rr.y + 21.0, 15.0, Theme::text());
        ui.text(&format!("{} · {}", pl.age, if pl.youth == 17 { crate::inhalt::t("ui.wort.gross_u17") } else { crate::inhalt::t("ui.wort.gross_u19") }), rr.x + 260.0, rr.y + 21.0, 14.0, Theme::text2());
        ui.pill(rr.x + 360.0, rr.y + 15.0, pl.ovr, 34.0);
        let (pa, pb) = c.pot_range(p);
        let st = (((pa + pb) / 2.0 - 50.0) / 9.0).round().clamp(1.0, 5.0) as usize;
        for s in 0..5 { draw_poly(rr.x + 400.0 + s as f32 * 16.0, rr.y + 16.0, 5, 6.0, -90.0, if s < st { Theme::gold() } else { Theme::ueberlagerung() }); }
        ui.text(crate::jugend::stufe_wort((pa + pb) / 2.0), rr.x + 490.0, rr.y + 21.0, 13.0, Theme::text2());
        if pl.ent.merkmale.iter().any(|m| m == "spaetzuender" || m == "fruehentwickler") { let m = pl.ent.merkmale.iter().find(|m| *m == "spaetzuender" || *m == "fruehentwickler").unwrap(); ui.text(crate::inhalt::t(&format!("ui.jugend.merkmal.{m}")), rr.x + 680.0, rr.y + 21.0, 13.0, Theme::gedaempft()); }
        if ui.hit(rr) { v.modal = Modal::Player(p); }
        y += 34.0;
    }
    // below left: the youth tables in short
    let t = Rect::new(x0() + 4.0, Y0 + 342.0, lw, hoehe() - Y0 - 358.0);
    ui.card(t);
    let mut y = t.y + 30.0;
    for k in 0..2 {
        let yl = &c.youth[k];
        let platz = yl.rows.iter().position(|r| r.own).map(|x| x + 1).unwrap_or(0);
        ui.text_semi(&yl.name, t.x + 16.0, y, 16.0, Theme::text());
        ui.text_right(&crate::tf!("ui.jugend.platz", platz, yl.rows.len()), t.x + t.w - 16.0, y, 15.0, jugend_farbe);
        y += 22.0;
        if !yl.last.is_empty() { let h = ui.paragraph(&yl.last, t.x + 16.0, y, 13.0, t.w - 32.0, Theme::text2()); y += h; }
        y += 18.0;
    }
}

/// Mentors: the pairs with their fit, and the co-trainer's proposals on request.
fn mentoren_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    ui.title(crate::inhalt::t("ui.jugend.reiter.mentoren"), area.x + 16.0, area.y + 34.0, 22.0, Theme::text());
    if ui.button(Rect::new(area.x + area.w - 300.0, area.y + 12.0, 284.0, 32.0), crate::inhalt::t("ui.jugend.mentor.vorschlagen"), true) {
        let n = c.mentor_vorschlaege();
        v.toast(&if n == 0 { crate::inhalt::t("ui.jugend.mentor.keine").to_string() } else { crate::tf!("ui.jugend.mentor.neu", n) }, n > 0);
    }
    ui.paragraph(crate::inhalt::t("ui.jugend.mentor.regel"), area.x + 16.0, area.y + 62.0, 13.0, area.w - 32.0, Theme::gedaempft());
    let sq = c.squad();
    let paare: Vec<(usize, usize)> = sq.iter().filter_map(|&s| c.world.players[s].ps.mentor.and_then(|m| c.pid(m)).map(|m| (m, s))).collect();
    let mut y = area.y + 100.0;
    if paare.is_empty() { ui.text(crate::inhalt::t("ui.jugend.mentor.leer"), area.x + 16.0, y + 20.0, 15.0, Theme::text2()); }
    let mut weg = None;
    for (m, s) in paare {
        let (pa, gr) = c.mentor_paar(m, s);
        let (mn, sn) = (c.world.players[m].name.clone(), c.world.players[s].name.clone());
        let r = Rect::new(area.x + 10.0, y, area.w - 20.0, 56.0);
        rrect(r.x, r.y, r.w, r.h, 8.0, alpha(hex(0x2dd4bf), 0.08));
        ui.player_photo(r.x + 26.0, r.y + 28.0, 16.0, c.world.players[m].id, &mn);
        ui.text_semi(&mn, r.x + 50.0, r.y + 24.0, 16.0, Theme::text());
        ui.text("→", r.x + 250.0, r.y + 24.0, 18.0, Theme::gedaempft()); // roh
        ui.player_photo(r.x + 296.0, r.y + 28.0, 16.0, c.world.players[s].id, &sn);
        ui.text_semi(&sn, r.x + 320.0, r.y + 24.0, 16.0, Theme::text());
        ui.text(&gr.iter().map(|g| crate::inhalt::t(g)).collect::<Vec<_>>().join(" "), r.x + 50.0, r.y + 46.0, 13.0, Theme::text2());
        ui.bar(r.x + 540.0, r.y + 20.0, 160.0, 8.0, pa, if pa >= 0.6 { Theme::gut() } else if pa >= 0.35 { Theme::achtung() } else { Theme::problem() });
        ui.text(&format!("{:.0} %", pa * 100.0), r.x + 710.0, r.y + 28.0, 14.0, Theme::text2());
        if ui.button(Rect::new(r.x + r.w - 140.0, r.y + 12.0, 128.0, 30.0), crate::inhalt::t("ui.jugend.mentor.trennen"), false) { weg = Some(s); }
        y += 64.0;
    }
    if let Some(s) = weg { c.world.players[s].ps.mentor = None; }
}

/// Zone of a table place: 0 CL, 1 EL, 2 Conference League, 3 promotion, 4 play-offs, 5 relegation / playout, 6 relegated.
fn tabellen_zone(pl: usize, rows: usize, ab: usize, rel: usize, pout: &[usize], auf: usize, po: &[usize], euro: &[(usize, usize)]) -> Option<u8> {
    if ab > 0 && pl + ab > rows { return Some(6); }
    if pl == rel || pout.contains(&pl) { return Some(5); }
    if let Some(e) = euro.iter().find(|e| e.0 == pl) { return Some(e.1.min(2) as u8); }
    if pl <= auf { return Some(3); }
    if po.contains(&pl) { return Some(4); }
    None
}
fn zone_farbe(z: u8) -> Color {
    match z { 0 => hex(0x1E6BFF), 1 => hex(0xF28C28), 2 => hex(0x2EB872), 3 => hex(0x18A058), 4 => hex(0x7C5CFF), 5 => Theme::achtung(), _ => Theme::problem() }
}

const INTRO_DAUER: f32 = 2.6;
/// The cut-scene before kick-off: stadium in the dark, both crests slide in, "VS", the competition, then "Anpfiff!".
fn match_intro(ui: &mut Ui, c: &Career, mv: &MatchView, r: Rect, t: f32) {
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
fn aktuelle_elf(mv: &MatchView, side: u8) -> Vec<usize> {
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
fn live_board(ui: &mut Ui, c: &Career, mv: &MatchView, r: Rect) {
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
fn eingriffe(ui: &mut Ui, v: &mut View, c: &mut Career, mv: &mut MatchView, r: Rect) {
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
    for (k, key) in [crate::id("antreiben"), crate::id("beruhigen"), crate::id("konzentration")].iter().enumerate() {
        let br = Rect::new(r.x + k as f32 * (bw + 8.0), r.y + 76.0, bw, 30.0);
        if ui.button(br, &ui.fit(crate::inhalt::t(&format!("ui.live.zuruf.{key}")), 13.0, bw - 8.0), false) {
            if !frei { v.toast(crate::inhalt::t("ui.live.zuruf.warten"), false); continue; }
            mv.eingriff.zuruf_min = mv.minute.max(0.1);
            // a shout works for about a quarter of an hour: its effect on the rest is scaled to that
            let anteil = (15.0 / (90.0 - mv.minute).max(15.0)).min(1.0);
            zuruf = match k { 0 => (1.0 + 0.10 * anteil, 1.0 + 0.04 * anteil), 1 => (1.0, 1.0 - 0.06 * anteil), _ => (1.0 - 0.02 * anteil, 1.0 - 0.08 * anteil) };
            v.toast(&crate::tf!("ui.live.log.zuruf", m, crate::inhalt::t(&format!("ui.live.zuruf.{key}"))), true);
            neu = true;
        }
    }
    // substitution: a click on a player on the pitch, then on one from the bench
    let wechsel = mv.res.events.iter().filter(|e| e.side == side && e.kind == 5 && (e.min as f32) <= mv.minute).count();
    ui.text_semi(&crate::tf!("ui.live.wechsel", wechsel, 5), r.x, r.y + 126.0, 13.0, Theme::gedaempft());
    let elf = aktuelle_elf(mv, side);
    let gespielt: Vec<usize> = mv.res.events.iter().filter(|e| e.side == side && e.kind == 5).flat_map(|e| [Some(e.pid), e.pid2]).flatten().collect();
    let bank: Vec<usize> = c.squad().into_iter().filter(|p| !elf.contains(p) && !gespielt.contains(p) && c.world.players[*p].inj == 0 && c.world.players[*p].susp == 0).collect();
    let spw = (r.w - 8.0) / 2.0;
    let zh = ((r.h - 140.0) / 11.0).clamp(15.0, 22.0);
    let platz = ((r.h - 136.0) / zh).max(0.0) as usize;
    for (k, &p) in elf.iter().take(platz).enumerate() {
        let rr = Rect::new(r.x, r.y + 134.0 + k as f32 * zh, spw, zh - 2.0);
        let sel = mv.eingriff.raus == Some(p);
        if sel { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::problem(), 0.2)); } else if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
        let pl = &c.world.players[p];
        // live grade and stamina so a substitution can be judged (Fabian 23.11.)
        let (note, kraft) = (live_note(&mv.res, p, mv.minute), live_kraft(&mv.res, side, p, mv.minute, pl.fit));
        let rechts = 64.0;
        ui.text(&ui.fit(&format!("{} {}", pos_de(&pl.pos), pl.name), 12.0, spw - rechts - 8.0), rr.x + 4.0, rr.y + zh * 0.7, 12.0, Theme::text());
        ui.text_right(&format!("{note:.1}"), rr.x + spw - 36.0, rr.y + zh * 0.7, 12.0, Theme::note(note));
        ui.text_right(&format!("{kraft:.0}%"), rr.x + spw - 4.0, rr.y + zh * 0.7, 11.0, if kraft < 60.0 { Theme::problem() } else if kraft < 75.0 { Theme::achtung() } else { Theme::text2() });
        if ui.hit(rr) { mv.eingriff.raus = if sel { None } else { Some(p) }; }
    }
    let mut bank_s = bank.clone();
    bank_s.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
    for (k, &p) in bank_s.iter().take(platz).enumerate() {
        let rr = Rect::new(r.x + spw + 8.0, r.y + 134.0 + k as f32 * zh, spw, zh - 2.0);
        let aktiv = mv.eingriff.raus.is_some() && wechsel < 5;
        if aktiv && ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::gut(), 0.2)); }
        let pl = &c.world.players[p];
        ui.text(&ui.fit(&format!("{} {}", pos_de(&pl.pos), pl.name), 12.0, spw - 72.0), rr.x + 4.0, rr.y + zh * 0.7, 12.0, if aktiv { Theme::text() } else { Theme::text2() });
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

// ================================================================ Tabellen
fn tabellen(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // no list on the left any more (Fabian 22.11.): country and competition are chosen in the banner, the page has the
    // whole width for table, scorers, form and news
    let gruppen = wettbewerb_gruppen(c);
    let l = v.table_league.unwrap_or(c.league());
    let x = x0() + 4.0;
    let tw = cbreite();
    let felder = wettbewerb_felder(c, &gruppen, l, Rect::new(x + tw - 480.0, Y0 - 40.0, 470.0, 36.0));
    if let Some((f, i)) = auswahl_vorher(ui, v, &felder) { v.table_league = Some(wahl_umsetzen(&gruppen, l, f, i)); v.league_scroll = 0; }
    let l = v.table_league.unwrap_or(c.league());
    if l >= 1000 && l - 1000 < c.world.cups.len() { cup_view(ui, c, l - 1000, x, tw); auswahl_zeichnen(ui, v, &felder); return; }
    let lg = &c.world.leagues[l];
    let lk = wettbewerb_look(c, &lg.key, 9, &lg.land);
    let lx = wettbewerb_banner(ui, Rect::new(x, Y0 - 44.0, tw, 46.0), &lk, c.world.cids.get(&lg.key).copied(), &lg.name);
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
    let hx = (x + 260.0).max(lx + ui.measure_title(&lg.name, 26.0) + 16.0);
    let hinweis = ui.fit(hinweis, 15.0, (x + tw - 500.0 - hx).max(60.0));
    ui.text(&hinweis, hx, Y0 - 14.0, 15.0, lk.text2);
    // estimated in daten/ligaregeln.toml (sicher = false) or no entry there: the standard rules
    if !regel.sicher && hx + ui.measure(&hinweis, 15.0) + 200.0 < x + tw - 490.0 { ui.text(crate::inhalt::t("ui.pages.regel_geschaetzt"), hx + ui.measure(&hinweis, 15.0), Y0 - 14.0, 15.0, Theme::achtung()); }
    let table = c.world.table(l);
    let rows = table.len();
    let area = Rect::new(x, Y0 + 6.0, tw * 0.64, hoehe() - Y0 - 22.0);
    ui.card(area);
    draw_rectangle(area.x + 10.0, area.y, area.w - 20.0, 3.0, lk.akzent_lesbar);
    player_row_head(ui, area.x, area.y + 22.0, &[("#", 10.0), (crate::inhalt::t("ui.wort.gross_verein"), 70.0), (crate::inhalt::t("ui.wort.gross_sp"), 330.0), (crate::inhalt::t("ui.wort.buchstabe_s"), 370.0), (crate::inhalt::t("ui.wort.buchstabe_u"), 400.0), (crate::inhalt::t("ui.wort.buchstabe_n"), 430.0), (crate::inhalt::t("ui.wort.gross_tore"), 470.0), (crate::inhalt::t("ui.wort.gross_pkt"), 540.0), (crate::inhalt::t("ui.wort.gross_form"), 590.0)]);
    // rows keep a readable height (26 to 32); a long table (league phase, 36 teams) scrolls instead of being squeezed
    let rh = ((area.h - 36.0) / rows as f32).clamp(26.0, 32.0);
    let vis = ((area.h - 36.0) / rh) as usize;
    let start = scroll_rows(ui, v, area, rows, vis);
    let mut zonen: Vec<u8> = vec![];
    for (i, r) in table.iter().enumerate().skip(start).take(vis) {
        let y = area.y + 30.0 + (i - start) as f32 * rh;
        let rr = Rect::new(area.x + 4.0, y, area.w - 8.0, rh - 2.0);
        if r.club == c.own { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::akzent(), 0.16)); }
        let pl = i + 1;
        // the zone of the place (Fabian 23.11.: plainly visible who goes to Europe, up, into the play-offs or down)
        let zone = tabellen_zone(pl, rows, ab, rel, &pout, auf, &po, &euro);
        if let Some(z) = zone {
            let col = zone_farbe(z);
            if !zonen.contains(&z) { zonen.push(z); }
            if r.club != c.own { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(col, 0.09)); }
            draw_rectangle(rr.x, rr.y + 1.0, 5.0, rr.h - 2.0, col);
        }
        // a line between two zones
        if i + 1 < rows && zone != tabellen_zone(pl + 1, rows, ab, rel, &pout, auf, &po, &euro) && i + 1 < start + vis { draw_rectangle(rr.x, rr.y + rh - 2.0, rr.w, 1.0, alpha(Theme::text2(), 0.35)); }
        let ty = y + rh * 0.7;
        let fs = (rh * 0.55).min(17.0);
        ui.text(&format!("{}", i + 1), area.x + 12.0, ty, fs, Theme::text2());
        let (c1, _) = club_color(c, r.club);
        let k = &c.world.clubs[r.club];
        ui.club_crest(area.x + 52.0, y + rh / 2.0 - 1.0, rh * 0.4, k.id, c1, &k.short);
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
    // legend of the zones shown, under the table
    zonen.sort();
    let mut lx = area.x + 12.0;
    for z in zonen {
        let col = zone_farbe(z);
        let t = crate::inhalt::t(&format!("ui.tabelle.zone.{z}"));
        draw_rectangle(lx, area.y + area.h - 18.0, 12.0, 10.0, col);
        ui.text(t, lx + 16.0, area.y + area.h - 8.0, 13.0, Theme::text2());
        lx += ui.measure(t, 13.0) + 34.0;
    }
    // right: scorers and scorer points side by side, the form table and the league's news (Fabian 22.11.)
    let rx = x + tw * 0.64 + 12.0;
    let rw = tw * 0.36 - 12.0;
    let hoch = hoehe() - Y0 - 22.0;
    // only goals and assists of this league (Fabian 23.11.)
    let spieler: Vec<usize> = (0..c.world.players.len()).filter(|&p| c.world.players[p].profil.ligen.iter().any(|x| x.l as usize == l)).collect();
    let r1 = Rect::new(rx, Y0 + 6.0, rw, hoch * 0.42);
    ui.card_titled(r1, crate::inhalt::t("statistik.torjaeger_hinweis"), lk.c1);
    let halb = (r1.w - 36.0) / 2.0;
    for (seite, titel) in [crate::inhalt::t("ui.liga.tore"), crate::inhalt::t("ui.liga.scorer")].iter().enumerate() {
        let sx = r1.x + 14.0 + seite as f32 * (halb + 8.0);
        ui.text_semi(&titel.to_uppercase(), sx, r1.y + 56.0, 12.0, Theme::gedaempft());
        let wert = |p: usize| { let x = c.world.players[p].in_liga(l); if seite == 0 { x.goals as u32 } else { (x.goals + x.assists) as u32 } };
        let mut l2: Vec<usize> = spieler.iter().copied().filter(|&p| wert(p) > 0).collect();
        l2.sort_by_key(|&p| std::cmp::Reverse(wert(p)));
        if l2.is_empty() { ui.text(crate::inhalt::t("statistik.keine_tore"), sx, r1.y + 80.0, 13.0, Theme::text2()); }
        let n = ((r1.h - 76.0) / 24.0) as usize;
        for (j, &p) in l2.iter().take(n).enumerate() {
            let pl = &c.world.players[p];
            let yy = r1.y + 80.0 + j as f32 * 24.0;
            ui.player_photo(sx + 9.0, yy - 5.0, 9.0, pl.id, &pl.name);
            let cl = &c.world.clubs[pl.team.max(0) as usize];
            ui.club_crest(sx + 26.0, yy - 5.0, 6.0, cl.id, hex(cl.c1), &cl.short);
            ui.text(&ui.fit(&pl.name, 13.0, halb - 70.0), sx + 36.0, yy, 13.0, if pl.team == c.own as i32 { Theme::gold() } else { Theme::text() });
            ui.text_right(&format!("{}", wert(p)), sx + halb, yy, 14.0, lk.akzent_lesbar);
        }
    }
    // the form table: points from the last five matches
    let r2 = Rect::new(rx, r1.y + r1.h + 10.0, rw, hoch * 0.3);
    ui.card_titled(r2, crate::inhalt::t("ui.liga.form"), lk.c1);
    let mut form: Vec<(usize, i32, Vec<char>)> = table.iter().map(|r| (r.club, r.form.iter().map(|ch| match ch { 'S' => 3, 'U' => 1, _ => 0 }).sum(), r.form.clone())).collect();
    form.sort_by_key(|x| std::cmp::Reverse(x.1));
    let n = ((r2.h - 56.0) / 24.0) as usize;
    for (j, (k, pkt, f)) in form.iter().take(n).enumerate() {
        let yy = r2.y + 60.0 + j as f32 * 24.0;
        let cl = &c.world.clubs[*k];
        ui.club_crest(r2.x + 22.0, yy - 5.0, 8.0, cl.id, hex(cl.c1), &cl.short);
        ui.text(&ui.fit(&cl.name, 13.0, r2.w - 190.0), r2.x + 36.0, yy, 13.0, if *k == c.own { Theme::gold() } else { Theme::text() });
        for (q, ch) in f.iter().enumerate() { let col = match ch { 'S' => Theme::gut(), 'N' => Theme::problem(), _ => Theme::achtung() }; rrect(r2.x + r2.w - 120.0 + q as f32 * 14.0, yy - 11.0, 10.0, 12.0, 2.0, col); }
        ui.text_right(&format!("{pkt}"), r2.x + r2.w - 14.0, yy, 14.0, Theme::text2());
    }
    // news of this league: the newspaper items naming one of its clubs, newest first
    let r3 = Rect::new(rx, r2.y + r2.h + 10.0, rw, Y0 + 6.0 + hoch - (r2.y + r2.h + 10.0));
    ui.card_titled(r3, crate::inhalt::t("ui.liga.news"), lk.c1);
    let namen: Vec<String> = lg.teams.iter().map(|&t| c.world.clubs[t].name.clone()).collect();
    let mut yy = r3.y + 56.0;
    let mut gezeigt = 0;
    for nw in c.feed.iter().filter(|nw| namen.iter().any(|n| nw.head.contains(n.as_str()) || nw.text.contains(n.as_str()))) {
        if yy > r3.y + r3.h - 30.0 { break; }
        ui.text(&date::short(nw.season, nw.day), r3.x + 14.0, yy, 12.0, Theme::gedaempft());
        yy = ui.paragraph(&ui.fit(&nw.head, 13.0, (r3.w - 28.0) * 2.0), r3.x + 14.0, yy + 16.0, 13.0, r3.w - 28.0, Theme::text()) + 8.0;
        gezeigt += 1;
    }
    // the league's own news when the papers say little: the leader and the last matchday's results
    if gezeigt < 3 && yy < r3.y + r3.h - 40.0 {
        if let Some(erster) = c.world.table(l).first().filter(|r| r.p > 0) {
            ui.text_passend(&crate::tf!("ui.liga.fuehrer", c.world.clubs[erster.club].name.clone(), erster.pts, erster.p), r3.x + 14.0, yy + 4.0, 13.0, r3.w - 28.0, 1, Theme::text());
            yy += 24.0;
            gezeigt += 1;
        }
        if let Some(tag) = lg.fixtures.iter().filter(|f| f.hg >= 0).map(|f| f.day).max() {
            ui.text(&crate::tf!("ui.liga.zuletzt", date::short(c.world.season, tag)), r3.x + 14.0, yy + 4.0, 12.0, Theme::gedaempft());
            yy += 20.0;
            for f in lg.fixtures.iter().filter(|f| f.day == tag && f.hg >= 0) {
                if yy > r3.y + r3.h - 24.0 { break; }
                ui.text_passend(&format!("{} – {} {}:{}", c.world.clubs[f.h].short, c.world.clubs[f.a].short, f.hg, f.ag), r3.x + 14.0, yy + 4.0, 13.0, r3.w - 28.0, 1, Theme::text2());
                yy += 20.0;
                gezeigt += 1;
            }
        }
    }
    if gezeigt == 0 { ui.text(crate::inhalt::t("ui.liga.keine_news"), r3.x + 14.0, yy, 13.0, Theme::text2()); }
    auswahl_zeichnen(ui, v, &felder);
}

// ---------------------------------------------------------------- choice of country and competition (Fabian 22.11.)
/// The competitions grouped by country: Europe first, then the own country, then the others by name; in a country its
/// leagues by tier, then its cups (index 1000 + cup).
pub fn wettbewerb_gruppen(c: &Career) -> Vec<(String, Vec<usize>)> {
    let land_von = |i: usize| -> String { if i >= 1000 { let cu = &c.world.cups[i - 1000]; if !cu.country.is_empty() { cu.country.clone() } else { c.world.leagues.iter().find(|l| l.key == cu.key).map(|l| l.land.clone()).unwrap_or_default() } } else { c.world.leagues[i].land.clone() } };
    let own_country = c.world.leagues[c.league()].land.clone();
    let mut laender: Vec<String> = c.world.leagues.iter().map(|l| l.land.clone()).collect();
    laender.sort(); laender.dedup();
    laender.sort_by_key(|x| (x != &own_country, x.is_empty()));
    let mut out = vec![(crate::inhalt::t("ui.wettbewerbe.europa").to_string(), c.world.cups.iter().enumerate().filter(|(_, x)| (1..=3).contains(&x.kind)).map(|(i, _)| 1000 + i).collect::<Vec<_>>())];
    for land in &laender {
        let mut v: Vec<usize> = (0..c.world.leagues.len()).filter(|&l| &c.world.leagues[l].land == land).collect();
        v.sort_by_key(|&l| (c.world.leagues[l].stufe, l));
        v.extend(c.world.cups.iter().enumerate().filter(|(i, x)| !(1..=3).contains(&x.kind) && &land_von(1000 + i) == land).map(|(i, _)| 1000 + i));
        out.push((if land.is_empty() { crate::inhalt::t("ui.wettbewerbe.weitere").to_string() } else { land.clone() }, v));
    }
    out.retain(|g| !g.1.is_empty());
    out
}
/// Leagues only (no cups), grouped by country like the competition pages, those `ok` allows.
pub fn ligen_gruppen(c: &Career, ok: impl Fn(usize) -> bool) -> Vec<(String, Vec<usize>)> {
    wettbewerb_gruppen(c).into_iter().map(|(n, v)| (n, v.into_iter().filter(|&i| i < 1000 && ok(i)).collect::<Vec<_>>())).filter(|g| !g.1.is_empty()).collect()
}
fn wettbewerb_name(c: &Career, i: usize) -> String { if i >= 1000 { c.world.cups.get(i - 1000).map(|x| x.name.clone()).unwrap_or_default() } else { c.world.leagues[i].name.clone() } }
/// The two fields (country, competition) in `r` for the chosen competition `l`.
pub fn wettbewerb_felder(c: &Career, gruppen: &[(String, Vec<usize>)], l: usize, r: Rect) -> Vec<Auswahl> {
    // nothing chosen (a scout working worldwide): both fields stay empty
    let gw = gruppen.iter().position(|x| x.1.contains(&l));
    let g = gw.unwrap_or(0);
    let w = (r.w - 10.0) / 2.0;
    vec![
        Auswahl { r: Rect::new(r.x, r.y, w * 0.85, r.h), titel: crate::inhalt::t("ui.wettbewerbe.land").into(), items: gruppen.iter().map(|x| x.0.clone()).collect(), sel: gw.unwrap_or(usize::MAX) },
        Auswahl { r: Rect::new(r.x + w * 0.85 + 10.0, r.y, w * 1.15, r.h), titel: crate::inhalt::t("ui.wettbewerbe.wettbewerb").into(), items: gruppen[g].1.iter().map(|&i| wettbewerb_name(c, i)).collect(), sel: gruppen[g].1.iter().position(|&x| x == l).unwrap_or(0) },
    ]
}
/// The competition after a choice: a new country opens its first competition, a competition is taken as it is.
pub fn wahl_umsetzen(gruppen: &[(String, Vec<usize>)], l: usize, feld: usize, i: usize) -> usize {
    if feld == 0 { gruppen.get(i).and_then(|g| g.1.first().copied()).unwrap_or(l) } else {
        let g = gruppen.iter().position(|x| x.1.contains(&l)).unwrap_or(0);
        gruppen[g].1.get(i).copied().unwrap_or(l)
    }
}
/// A choice field with a list that opens below it.
pub struct Auswahl { pub r: Rect, pub titel: String, pub items: Vec<String>, pub sel: usize }
/// The open list: a search row on top, then up to 14 entries; wide enough that names are not cut (Fabian 23.11.).
fn auswahl_liste(a: &Auswahl) -> Rect { Rect::new(a.r.x.min(crate::ui::breite() - a.r.w.max(340.0) - 8.0), a.r.y + a.r.h + 4.0, a.r.w.max(340.0), (a.items.len().min(14) as f32) * 28.0 + 8.0 + 32.0) }
/// The entries matching the search (all without one).
fn auswahl_treffer(a: &Auswahl, q: &str) -> Vec<usize> { let q = q.to_lowercase(); (0..a.items.len()).filter(|&i| q.is_empty() || a.items[i].to_lowercase().contains(&q)).collect() }
/// Before anything else on the page (the list lies over the page and takes the click first): a chosen entry, the open
/// field toggled, a click outside closes.
pub fn auswahl_vorher(ui: &mut Ui, v: &mut View, felder: &[Auswahl]) -> Option<(usize, usize)> {
    if let Some(f) = v.auswahl {
        if let Some(a) = felder.get(f) {
            let lr = auswahl_liste(a);
            // typing filters the list
            while let Some(ch) = get_char_pressed() { if !ch.is_control() { v.auswahl_suche.push(ch); v.auswahl_scroll = 0; } }
            if is_key_pressed(KeyCode::Backspace) { v.auswahl_suche.pop(); v.auswahl_scroll = 0; }
            let treffer = auswahl_treffer(a, &v.auswahl_suche);
            let n = treffer.len();
            if is_key_pressed(KeyCode::Enter) && n > 0 { v.auswahl = None; let i = treffer[0]; v.auswahl_suche.clear(); return Some((f, i)); }
            if lr.contains(ui.mouse) && ui.wheel != 0.0 { v.auswahl_scroll = (v.auswahl_scroll as i32 - ui.wheel.signum() as i32).clamp(0, n.saturating_sub(14) as i32) as usize; }
            for (k, &i) in treffer.iter().skip(v.auswahl_scroll).take(14).enumerate() {
                if ui.hit(Rect::new(lr.x + 4.0, lr.y + 36.0 + k as f32 * 28.0, lr.w - 8.0, 26.0)) { v.auswahl = None; v.auswahl_suche.clear(); return Some((f, i)); }
            }
            if ui.clicked && lr.contains(ui.mouse) { ui.clicked = false; }
            if ui.clicked && !a.r.contains(ui.mouse) { v.auswahl = None; ui.clicked = false; }
        }
    }
    for (k, a) in felder.iter().enumerate() {
        if ui.hit(a.r) { v.auswahl = if v.auswahl == Some(k) { None } else { Some(k) }; v.auswahl_scroll = felder[k].sel.saturating_sub(5); v.auswahl_suche.clear(); while get_char_pressed().is_some() {} }
    }
    None
}
/// At the end of the page: the fields and the open list on top.
pub fn auswahl_zeichnen(ui: &mut Ui, v: &View, felder: &[Auswahl]) {
    for (k, a) in felder.iter().enumerate() {
        rrect(a.r.x, a.r.y, a.r.w, a.r.h, 8.0, Theme::flaeche());
        rrect_line(a.r.x, a.r.y, a.r.w, a.r.h, 8.0, if v.auswahl == Some(k) { Theme::akzent() } else { Theme::linie() });
        ui.text(&a.titel.to_uppercase(), a.r.x + 10.0, a.r.y + 12.0, 10.0, Theme::gedaempft());
        let w = a.items.get(a.sel).cloned().unwrap_or_default();
        ui.text_semi(&ui.fit(&w, 14.0, a.r.w - 34.0), a.r.x + 10.0, a.r.y + a.r.h - 8.0, 14.0, Theme::text());
        // a small arrow (the font has no triangle glyph)
        let (ax, ay) = (a.r.x + a.r.w - 16.0, a.r.y + a.r.h / 2.0);
        if v.auswahl == Some(k) { draw_triangle(vec2(ax - 5.0, ay + 3.0), vec2(ax + 5.0, ay + 3.0), vec2(ax, ay - 3.0), Theme::text2()); } else { draw_triangle(vec2(ax - 5.0, ay - 3.0), vec2(ax + 5.0, ay - 3.0), vec2(ax, ay + 3.0), Theme::text2()); }
    }
    if let Some(a) = v.auswahl.and_then(|f| felder.get(f)) {
        let lr = auswahl_liste(a);
        rrect(lr.x - 2.0, lr.y - 2.0, lr.w + 4.0, lr.h + 4.0, 10.0, Theme::schatten());
        rrect(lr.x, lr.y, lr.w, lr.h, 8.0, Theme::flaeche());
        rrect_line(lr.x, lr.y, lr.w, lr.h, 8.0, Theme::linie());
        // the search row
        let sr = Rect::new(lr.x + 6.0, lr.y + 6.0, lr.w - 12.0, 24.0);
        rrect(sr.x, sr.y, sr.w, sr.h, 6.0, Theme::erhoehung());
        if v.auswahl_suche.is_empty() { ui.text(crate::inhalt::t("ui.wettbewerbe.suchen"), sr.x + 8.0, sr.y + 17.0, 14.0, Theme::gedaempft()); } else { ui.text(&format!("{}|", v.auswahl_suche), sr.x + 8.0, sr.y + 17.0, 14.0, Theme::text()); }
        let treffer = auswahl_treffer(a, &v.auswahl_suche);
        for (k, &i) in treffer.iter().skip(v.auswahl_scroll).take(14).enumerate() {
            let rr = Rect::new(lr.x + 4.0, lr.y + 36.0 + k as f32 * 28.0, lr.w - 8.0, 26.0);
            if i == a.sel { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::akzent(), 0.16)); } else if rr.contains(ui.mouse) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
            ui.text(&ui.fit(&a.items[i], 14.0, rr.w - 16.0), rr.x + 8.0, rr.y + 18.0, 14.0, if i == a.sel { Theme::akzent() } else { Theme::text() });
        }
    }
}

/// A cup or European competition: league-phase table (Europe) and the knock-out rounds with results.
static CUP_REITER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
/// The latest results and the next fixtures of a competition (Fabian 23.11.: always on the schedule); the own club's
/// matches highlighted, the latest stage of play first.
fn cup_spiele_kasten(ui: &mut Ui, c: &Career, ci: usize, r: Rect, lk: &Look) {
    let cup = &c.world.cups[ci];
    ui.card_titled(r, crate::inhalt::t("ui.cup.spiele"), lk.c1);
    let mut gespielt: Vec<&crate::cups::CupFx> = cup.fx.iter().filter(|f| f.f.hg >= 0).collect();
    gespielt.sort_by_key(|f| std::cmp::Reverse(f.f.day));
    let mut offen: Vec<&crate::cups::CupFx> = cup.fx.iter().filter(|f| f.f.hg < 0).collect();
    offen.sort_by_key(|f| f.f.day);
    let n = (((r.h - 110.0) / 2.0) / 20.0).max(1.0) as usize;
    let mut y = r.y + 54.0;
    for (titel, liste) in [(crate::inhalt::t("ui.cup.letzte"), gespielt), (crate::inhalt::t("ui.cup.naechste"), offen)] {
        ui.text_semi(titel, r.x + 14.0, y, 14.0, Theme::akzent());
        y += 22.0;
        if liste.is_empty() { ui.text(crate::inhalt::t("ui.cup.keine"), r.x + 14.0, y, 14.0, Theme::gedaempft()); y += 22.0; }
        // the own matches first, then the rest of that day
        let mut l = liste.clone();
        l.sort_by_key(|f| !(f.f.h == c.own || f.f.a == c.own));
        let tag = l.first().map(|f| f.f.day);
        let mut auswahl: Vec<&&crate::cups::CupFx> = l.iter().filter(|f| f.f.h == c.own || f.f.a == c.own).take(2).collect();
        auswahl.extend(liste.iter().filter(|f| Some(f.f.day) == tag && !(f.f.h == c.own || f.f.a == c.own)));
        for f in auswahl.into_iter().take(n) {
            let mine = f.f.h == c.own || f.f.a == c.own;
            let res = if f.f.hg < 0 { date::short(c.world.season, f.f.day) } else { format!("{}:{}", f.f.hg, f.f.ag) };
            let t = format!("{} – {}", c.world.clubs[f.f.h].short, c.world.clubs[f.f.a].short);
            ui.text(&ui.fit(&t, 14.0, r.w - 100.0), r.x + 14.0, y, 14.0, if mine { Theme::gold() } else { Theme::text() });
            ui.text_right(&res, r.x + r.w - 14.0, y, 14.0, Theme::text2());
            y += 20.0;
        }
        y += 14.0;
    }
}
fn cup_view(ui: &mut Ui, c: &Career, ci: usize, x: f32, tw: f32) {
    let cup = &c.world.cups[ci];
    let land = if cup.country.is_empty() { c.world.leagues.iter().find(|l| l.key == cup.key).map(|l| l.land.clone()).unwrap_or_default() } else { cup.country.clone() };
    let lk = wettbewerb_look(c, &cup.key, cup.kind, &land);
    let nach_name = wettbewerb_banner(ui, Rect::new(x, Y0 - 44.0, tw, 46.0), &lk, c.world.cids.get(&cup.key).copied(), &cup.name);
    let status = match cup.winner { Some(w) => crate::tf!("ui.pages.sieger", c.world.clubs[w].name), None => crate::tf!("ui.pages.laeuft", if cup.stage < 10 { crate::inhalt::t("ui.pages.ligaphase").to_string() } else { crate::cups::stage_name(cup.stage, cup.kind) }) };
    ui.text(&ui.fit(&status, 15.0, (x + tw - 500.0 - nach_name - 16.0).max(60.0)), nach_name + 16.0, Y0 - 14.0, 15.0, lk.text2);
    // league phase and knock-out rounds in two tabs (Fabian 23.11.), the matches box always on the right
    let liga = (1..=3).contains(&cup.kind);
    let mut reiter = if liga { CUP_REITER.load(std::sync::atomic::Ordering::Relaxed) } else { 1 };
    let mut top = Y0 + 6.0;
    if liga {
        if let Some(i) = ui.segment(Rect::new(x, Y0 + 6.0, 360.0, 30.0), &[crate::inhalt::t("ui.pages.ligaphase"), crate::inhalt::t("wb.liste.runden")], reiter) { reiter = i; CUP_REITER.store(i, std::sync::atomic::Ordering::Relaxed); }
        top += 40.0;
    }
    let kw = (tw * 0.30).max(280.0);
    cup_spiele_kasten(ui, c, ci, Rect::new(x + tw - kw, top, kw, hoehe() - top - 16.0), &lk);
    let (rx, rw) = (x, tw - kw - 12.0);
    if reiter == 0 {
        // the detailed league-phase table: played, won, drawn, lost, goals, difference, points, form
        let area = Rect::new(rx, top, rw, hoehe() - top - 16.0);
        ui.card_titled(area, crate::inhalt::t("ui.pages.ligaphase"), lk.c1);
        let tab = c.world.cup_table(ci);
        let rh = ((area.h - 70.0) / tab.len().max(1) as f32).min(24.0);
        let spalten = [(crate::inhalt::t("ui.cup.sp"), 0.50), (crate::inhalt::t("ui.cup.s"), 0.57), (crate::inhalt::t("ui.cup.u"), 0.63), (crate::inhalt::t("ui.cup.n"), 0.69), (crate::inhalt::t("ui.cup.tore"), 0.76), (crate::inhalt::t("ui.cup.diff"), 0.84), (crate::inhalt::t("ui.cup.pkt"), 0.91)];
        for (t, f) in spalten { ui.text_center(t, area.x + area.w * f, area.y + 58.0, 12.0, Theme::gedaempft()); }
        for (i, r) in tab.iter().enumerate() {
            let y = area.y + 66.0 + i as f32 * rh;
            if r.club == c.own { rrect(area.x + 4.0, y, area.w - 8.0, rh - 1.0, 3.0, alpha(Theme::akzent(), 0.16)); }
            let zone = if i < 8 { Theme::gut() } else if i < 24 { Theme::achtung() } else { Theme::problem() };
            draw_rectangle(area.x + 4.0, y + 2.0, 3.0, rh - 4.0, zone);
            let fs = (rh * 0.66).min(15.0);
            ui.text(&format!("{}", i + 1), area.x + 12.0, y + rh * 0.75, fs, Theme::text2());
            let k = &c.world.clubs[r.club];
            ui.club_crest(area.x + 44.0, y + rh / 2.0, rh * 0.42, k.id, hex(k.c1), &k.short);
            ui.text(&ui.fit(&k.name, fs, area.w * 0.44 - 60.0), area.x + 58.0, y + rh * 0.75, fs, Theme::text());
            let werte = [r.p.to_string(), r.w.to_string(), r.d.to_string(), r.l.to_string(), format!("{}:{}", r.gf, r.ga), format!("{:+}", r.gf - r.ga), r.pts.to_string()];
            for (k2, (_, f)) in spalten.iter().enumerate() { ui.text_center(&werte[k2], area.x + area.w * f, y + rh * 0.75, fs, if k2 == 6 { Theme::text() } else { Theme::text2() }); }
            let form: String = r.form.iter().rev().take(5).collect();
            ui.text_right(&form, area.x + area.w - 10.0, y + rh * 0.75, fs * 0.9, Theme::gedaempft());
        }
        let _ = nach_name;
        return;
    }
    let rr = Rect::new(rx, top, rw, hoehe() - top - 16.0);
    ui.card_titled(rr, crate::inhalt::t("wb.liste.runden"), lk.c1);
    let mut stages: Vec<u8> = cup.fx.iter().map(|f| f.stage).filter(|s| *s >= 20).collect();
    stages.sort();
    stages.dedup();
    stages.reverse();
    let mut y = rr.y + 54.0;
    if stages.is_empty() { ui.text(crate::inhalt::t("ui.pages.die_k_o_runden_folgen"), rr.x + 14.0, y, 15.0, Theme::text2()); }
    // from the last 16 on: a bracket, one half left, the other right, the final in the middle (Fabian 22.11.)
    if turnierbaum(ui, c, ci, Rect::new(rr.x + 8.0, rr.y + 44.0, rr.w - 16.0, rr.h - 52.0), &lk) { return; }
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

/// Konzept Nationalteams Teil 7: our internationals, the FIFA ranking and the tournaments (past and coming).
fn nationalteams_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    tabs(ui, v, &[t("ui.nt.reiter.ueberblick"), t("ui.nt.reiter.laender")]);
    if v.tab == 1 { return nation_seite(ui, v, c); }
    let w3 = (cbreite() - 24.0) / 3.0;
    let h = hoehe() - Y0 - 16.0;
    let (l, m, r) = (Rect::new(x0() + 4.0, Y0, w3, h), Rect::new(x0() + 16.0 + w3, Y0, w3, h), Rect::new(x0() + 28.0 + 2.0 * w3, Y0, w3, h));
    ui.card_titled(l, t("ui.nt.unsere"), Theme::verein());
    let mut eigene: Vec<usize> = c.squad().into_iter().filter(|&p| c.world.players[p].nat > 0).collect();
    eigene.sort_by_key(|&p| (c.world.nat_status(p).unwrap_or(9), std::cmp::Reverse(c.world.players[p].profil.laenderspiele)));
    let mut y = l.y + 60.0;
    let mut klick = None;
    for p in eigene.into_iter().take(((l.h - 70.0) / 26.0) as usize) {
        let pl = &c.world.players[p];
        let rr = Rect::new(l.x + 8.0, y - 18.0, l.w - 16.0, 24.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
        ui.text(&ui.fit(&pl.name, 14.0, l.w * 0.42), l.x + 14.0, y, 14.0, Theme::text());
        ui.text(&ui.fit(&crate::nationen::name(&c.world, pl.nat), 13.0, l.w * 0.24), l.x + l.w * 0.46, y, 13.0, Theme::text2());
        let st = c.world.nat_status(p).map(|s| t(&format!("ui.nt.kurz.{s}"))).unwrap_or("–");
        ui.text(st, l.x + l.w * 0.70, y, 13.0, Theme::akzent());
        ui.text_right(&format!("{}", pl.profil.laenderspiele), l.x + l.w - 14.0, y, 13.0, Theme::text2());
        if ui.hit(rr) { klick = Some(p); }
        y += 26.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
    // the ranking: the nations with a team, by points
    ui.card_titled(m, t("ui.nt.rangliste"), Theme::gold());
    let mut nats: Vec<u16> = c.world.nationalteams.iter().map(|x| x.nat).filter(|&n| !crate::nationen::name(&c.world, n).is_empty()).collect();
    nats.sort_by(|a, b| c.world.fifa_punkte(*b).partial_cmp(&c.world.fifa_punkte(*a)).unwrap());
    if nats.is_empty() { ui.paragraph(t("ui.nt.noch_keine"), m.x + 14.0, m.y + 60.0, 14.0, m.w - 28.0, Theme::text2()); }
    let eigen_land = c.world.land_nat(c.league());
    for (i, n) in nats.iter().take(((m.h - 70.0) / 22.0) as usize).enumerate() {
        let y = m.y + 60.0 + i as f32 * 22.0;
        let col = if *n == eigen_land { Theme::gold() } else { Theme::text() };
        ui.text(&format!("{}.", i + 1), m.x + 14.0, y, 13.0, Theme::gedaempft());
        ui.text(&ui.fit(&crate::nationen::name(&c.world, *n), 14.0, m.w * 0.55), m.x + 48.0, y, 14.0, col);
        ui.text_right(&format!("{:.0}", c.world.fifa_punkte(*n)), m.x + m.w - 14.0, y, 13.0, Theme::text2());
    }
    // tournaments: the coming ones from the cycle, then the history
    ui.card_titled(r, t("ui.nt.turniere"), Theme::akzent());
    let (jahr, _, _) = crate::date::ymd(c.world.season, c.world.day);
    let mut kommend: Vec<(i32, String)> = vec![];
    for w in crate::regeln::wert("nationalwettbewerbe", crate::id("wettbewerb")).as_array().cloned().unwrap_or_default() {
        let kz = w["kuerzel"].as_str().unwrap_or("");
        if w["format"].as_str() != Some("gruppen_ko") || kz.starts_with(crate::id("u2")) || kz == "olympia" { continue; }
        let (e, rh) = (w["erstes_jahr"].as_i64().unwrap_or(0) as i32, w["rhythmus"].as_i64().unwrap_or(4).max(1) as i32);
        let mut j = e.max(jahr);
        while (j - e) % rh != 0 { j += 1; }
        if c.world.turniere.iter().any(|x| x.jahr == j && Some(x.name.as_str()) == w["name"].as_str()) { j += rh; }
        kommend.push((j, w["name"].as_str().unwrap_or("").to_string()));
    }
    kommend.sort();
    let mut y = r.y + 60.0;
    ui.text_semi(t("ui.nt.kommend"), r.x + 14.0, y, 14.0, Theme::akzent());
    y += 22.0;
    for (j, n) in kommend.iter().take(6) { ui.text(&format!("{j} · {n}"), r.x + 14.0, y, 14.0, Theme::text()); y += 20.0; }
    y += 14.0;
    ui.text_semi(t("ui.nt.geschichte"), r.x + 14.0, y, 14.0, Theme::akzent());
    y += 22.0;
    if c.world.turniere.is_empty() { ui.text(t("ui.cup.keine"), r.x + 14.0, y, 14.0, Theme::gedaempft()); }
    for x in c.world.turniere.iter().rev() {
        if y > r.y + r.h - 12.0 { break; }
        ui.text(&ui.fit(&crate::tf!("ui.nt.sieger_zeile", x.jahr, x.name.clone(), crate::nationen::name(&c.world, x.sieger)), 14.0, r.w - 28.0), r.x + 14.0, y, 14.0, Theme::text());
        y += 20.0;
    }
}

/// Nationalteams › Länder, built like the club page (Fabian 23.11.): the ranking on the left, on the right the nation's
/// hero with facts, the coach and his philosophy, titles and the squad by lines.
fn nation_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let h = hoehe() - Y0 - 16.0;
    let lw = (cbreite() * 0.30).max(280.0);
    let l = Rect::new(x0() + 4.0, Y0, lw, h);
    let r = Rect::new(x0() + lw + 16.0, Y0, cbreite() - lw - 12.0, h);
    // every nation with enough players, also before the first international break (then ranked by strength)
    let mut zahl: std::collections::HashMap<u16, usize> = Default::default();
    for p in &c.world.players { if p.nat > 0 && p.team != -2 { *zahl.entry(p.nat).or_default() += 1; } }
    let mut nats: Vec<u16> = zahl.into_iter().filter(|&(n, k)| k >= 16 || c.world.nationalteams.iter().any(|x| x.nat == n)).map(|x| x.0).filter(|&n| !crate::nationen::name(&c.world, n).is_empty()).collect();
    let punkte: std::collections::HashMap<u16, f32> = nats.iter().map(|&n| (n, c.world.fifa_punkte(n))).collect();
    nats.sort_by(|a, b| punkte[b].partial_cmp(&punkte[a]).unwrap());
    ui.card_titled(l, t("ui.nt.rangliste"), Theme::gold());
    if nats.is_empty() { ui.paragraph(t("ui.nt.noch_keine"), l.x + 14.0, l.y + 60.0, 14.0, l.w - 28.0, Theme::text2()); return; }
    let wahl = v.nt_wahl.filter(|n| nats.contains(n)).unwrap_or_else(|| { let e = c.world.land_nat(c.league()); if nats.contains(&e) { e } else { nats[0] } });
    let platz = ((l.h - 60.0) / 26.0).max(1.0) as usize;
    if l.contains(ui.mouse) && ui.wheel != 0.0 { v.nt_scroll = (v.nt_scroll as i32 - ui.wheel.signum() as i32 * 3).clamp(0, nats.len().saturating_sub(platz) as i32) as usize; }
    for (i, &n) in nats.iter().enumerate().skip(v.nt_scroll).take(platz) {
        let rr = Rect::new(l.x + 6.0, l.y + 48.0 + (i - v.nt_scroll) as f32 * 26.0, l.w - 12.0, 24.0);
        if n == wahl { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, alpha(Theme::akzent(), 0.16)); } else if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, Theme::erhoehung()); }
        ui.text(&format!("{}.", i + 1), rr.x + 6.0, rr.y + 17.0, 13.0, Theme::gedaempft());
        ui.text(&ui.fit(&crate::nationen::name(&c.world, n), 14.0, rr.w * 0.6), rr.x + 44.0, rr.y + 17.0, 14.0, Theme::text());
        ui.text_right(&format!("{:.0}", punkte[&n]), rr.x + rr.w - 8.0, rr.y + 17.0, 13.0, Theme::text2());
        if ui.hit(rr) { v.nt_wahl = Some(n); }
    }
    // the hero in a colour of the nation's own (no flag data yet: fixed from its number)
    ui.card(r);
    let farbe = macroquad::color::hsl_to_rgb((wahl as f32 * 0.137).fract(), 0.55, 0.32);
    let hero = Rect::new(r.x, r.y, r.w, 170.0);
    gradient_h(hero.x, hero.y, hero.w, hero.h, farbe, alpha(farbe, 0.7));
    let weiss = hex(0xffffff);
    let name = crate::nationen::name(&c.world, wahl);
    let rang = nats.iter().position(|&n| n == wahl).unwrap_or(0) + 1;
    ui.title(&ui.fit(&name, 36.0, hero.w - 40.0), hero.x + 24.0, hero.y + 62.0, 36.0, weiss);
    let team = c.world.nationalteams.iter().find(|x| x.nat == wahl).cloned().unwrap_or_default();
    ui.text(&crate::tf!("ui.nt.held_zeile", crate::nationalteams::verband(wahl), rang, format!("{:.0}", c.world.fifa_punkte(wahl)), stufe_wort(c.world.nat_staerke(wahl))), hero.x + 24.0, hero.y + 92.0, 17.0, alpha(weiss, 0.9));
    let titel = c.world.turniere.iter().filter(|x| x.sieger == wahl).count();
    let chips = [crate::tf!("ui.nt.chip_bilanz", team.spiele, team.siege, team.remis, team.spiele.saturating_sub(team.siege + team.remis)), crate::tf!("ui.nt.chip_titel", titel), crate::tf!("ui.nt.chip_kader", team.kader.len())];
    let mut cx = hero.x + 24.0;
    for ch in chips { let w = ui.measure(&ch, 14.0) + 20.0; rrect(cx, hero.y + 114.0, w, 26.0, 13.0, alpha(BLACK, 0.35)); ui.text(&ch, cx + 10.0, hero.y + 132.0, 14.0, weiss); cx += w + 8.0; }
    let sp = r.w * 0.42;
    let links = Rect::new(r.x + 16.0, hero.y + hero.h + 14.0, sp - 24.0, r.h - hero.h - 24.0);
    let rechts = Rect::new(r.x + sp + 8.0, hero.y + hero.h + 14.0, r.w - sp - 22.0, r.h - hero.h - 24.0);
    // the coach and what he looks for (Teil 2.2)
    let tr = crate::nationalteams::trainer(&c.world, wahl);
    let mut y = links.y;
    ui.text_semi(&t("ui.verein.trainer").to_uppercase(), links.x, y + 12.0, 13.0, Theme::akzent());
    ui.text_semi(if tr.name.is_empty() { t("ui.nt.trainer_unbekannt") } else { &tr.name }, links.x, y + 36.0, 16.0, Theme::text());
    y += 50.0;
    for (k, w) in [("treue", tr.treue), ("form", tr.form), ("spielzeit", tr.spielzeit), ("jugend", tr.jugend), ("liga", tr.liga), ("disziplin", tr.disziplin)] { // roh
        ui.text(t(&format!("ui.nt.phil.{k}")), links.x, y + 12.0, 13.0, Theme::text2());
        ui.bar(links.x + links.w * 0.45, y + 4.0, links.w * 0.55, 8.0, w, Theme::akzent());
        y += 22.0;
    }
    y += 12.0;
    ui.text_semi(&t("ui.nt.geschichte").to_uppercase(), links.x, y + 12.0, 13.0, Theme::akzent());
    y += 30.0;
    let erfolge: Vec<_> = c.world.turniere.iter().rev().filter(|x| x.sieger == wahl || x.zweiter == wahl).collect();
    if erfolge.is_empty() { ui.text(t("ui.cup.keine"), links.x, y + 6.0, 14.0, Theme::gedaempft()); }
    for x in erfolge { if y > links.y + links.h - 20.0 { break; } ui.text(&ui.fit(&crate::tf!(if x.sieger == wahl { "ui.nt.erfolg_sieg" } else { "ui.nt.erfolg_finale" }, x.jahr, x.name.clone()), 14.0, links.w), links.x, y + 6.0, 14.0, Theme::text()); y += 20.0; }
    // the squad by lines, own players marked; a click opens the profile
    ui.text_semi(&t("ui.nt.aktueller_kader").to_uppercase(), rechts.x, rechts.y + 12.0, 13.0, Theme::akzent());
    let mut kader = team.kader.clone();
    kader.sort_by(|a, b| (part_of(&c.world.players[*a].pos), -(c.world.players[*a].ovr as i32)).cmp(&(part_of(&c.world.players[*b].pos), -(c.world.players[*b].ovr as i32))));
    if kader.is_empty() { ui.paragraph(t("ui.nt.kein_kader"), rechts.x, rechts.y + 40.0, 14.0, rechts.w, Theme::text2()); }
    let mut yy = rechts.y + 26.0;
    let mut klick = None;
    for p in kader {
        if yy > rechts.y + rechts.h - 24.0 { break; }
        let pl = &c.world.players[p];
        let rr = Rect::new(rechts.x, yy, rechts.w, 24.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
        ui.pos_tag(rr.x + 4.0, rr.y + 12.0, pos_de(&pl.pos));
        let eigen = pl.team == c.own as i32;
        ui.text(&ui.fit(&pl.name, 14.0, rr.w * 0.42), rr.x + 52.0, rr.y + 17.0, 14.0, if eigen { Theme::gold() } else { Theme::text() });
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { String::new() };
        ui.text(&ui.fit(&club, 13.0, rr.w * 0.3), rr.x + rr.w * 0.52, rr.y + 17.0, 13.0, Theme::text2());
        ui.text_right(&format!("{}", pl.profil.laenderspiele), rr.x + rr.w - 44.0, rr.y + 17.0, 13.0, Theme::text2());
        ui.pill(rr.x + rr.w - 18.0, rr.y + 12.0, pl.ovr, 30.0);
        if ui.hit(rr) { klick = Some(p); }
        yy += 26.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
}

// ================================================================ Presse
fn presse(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("zentrale.k.schlagzeilen"), crate::inhalt::t("vorbereitung.pk")]);
    if v.tab == 1 || c.press.is_some() && v.tab == 0 && false { }
    if v.tab == 1 {
        press_card(ui, v, c, Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0));
        return;
    }
    if c.press.is_some() {
        let r = Rect::new(x0() + 4.0, Y0, cbreite(), 60.0);
        rrect(r.x, r.y, r.w, r.h, 8.0, alpha(Theme::achtung(), 0.12));
        ui.title(crate::inhalt::t("ui.pages.die_presse_wartet_auf_dich"), r.x + 16.0, r.y + 38.0, 24.0, Theme::achtung());
        if ui.button(Rect::new(r.x + r.w - 200.0, r.y + 12.0, 184.0, 36.0), crate::inhalt::t("ui.pages.zur_konferenz"), true) { v.tab = 1; ui.play(crate::id("pressefrage")); }
    }
    let y0 = if c.press.is_some() { Y0 + 72.0 } else { Y0 };
    webblog(ui, v, c, y0);
}

/// One outlet of the blog: (name, layout type, background, text, accent). `id` is "layout|name" (media pool) or an
/// old id of medien.toml [[webblog]]; each name gets its own house colour within the layout's palette.
fn medium_stil(id: &str) -> (String, String, Color, Color, Color) {
    if let Some((layout, name)) = id.split_once('|') {
        let h = name.bytes().fold(7u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
        let wahl = |v: &[u32]| hex(v[h as usize % v.len()]);
        let (g, f, a) = match layout {
            "datenportal" => (wahl(&[0x1a3151, 0x0b3d2e, 0x2b2d42, 0x14213d]), hex(0xffffff), wahl(&[0x5cc1e8, 0x3ddc97, 0xffb703])),
            "boulevard" => (wahl(&[0xe30613, 0xc1121f, 0xd00000]), hex(0xffffff), hex(0xffe600)),
            "tabloid_en" => (wahl(&[0xe4021b, 0x1d3557, 0xb5179e]), hex(0xffffff), hex(0x111111)),
            "tv" => (hex(0x111318), hex(0xffffff), wahl(&[0xe63946, 0x00b4d8, 0xffb703, 0x7bd389])),
            "podcast" => (wahl(&[0x5b2a86, 0x264653, 0x6a040f, 0x3a0ca3]), hex(0xffffff), hex(0xffd166)),
            "lokal" => (hex(0xf4efe3), hex(0x2b2118), wahl(&[0x8a5a2b, 0x2f5d50, 0x7a2e2e])),
            "verein" => (Theme::verein(), Theme::verein_auf(), Theme::gold()),
            "analyse" => (hex(0xfaf7f0), hex(0x121212), wahl(&[0xe8622c, 0x2a9d8f, 0x6d597a])),
            "serioes" | "serioes_en" => (wahl(&[0x052962, 0x1b263b, 0x283618, 0x3d405b]), hex(0xffffff), wahl(&[0xffe500, 0xe9c46a, 0xa8dadc])),
            _ => (hex(0xffffff), hex(0x111111), wahl(&[0xd0021b, 0x0077b6, 0x2a9d8f])),
        };
        let layout = if layout == "serioes" { "serioes_en" } else { layout };
        return (name.to_string(), layout.to_string(), g, f, a);
    }
    let m = crate::regeln::wert("medien", "webblog").as_array().and_then(|a| a.iter().find(|m| m["id"] == id)).cloned().unwrap_or_default();
    let f = |i: usize| hex(m["farben"].get(i).and_then(|x| x.as_u64()).unwrap_or(0x888888) as u32);
    (m["name"].as_str().unwrap_or(id).to_string(), m["typ"].as_str().unwrap_or(crate::id("fachmagazin")).to_string(), f(0), f(1), f(2))
}

/// One article card in the style of its outlet: data portal, sports magazine, tabloid, long read, English papers.
fn artikel_karte(ui: &mut Ui, r: Rect, n: &News, medium: &str, gross: bool) {
    let (name, typ, grund, schrift, akzent) = medium_stil(medium);
    let kopf = if gross { 34.0 } else { 24.0 };
    let hs = if gross { 30.0 } else { 19.0 };
    let datum = date::short(n.season, n.day);
    let (bg, fg, sub) = match typ.as_str() { "boulevard" | "tv" | "podcast" | "verein" => (grund, schrift, alpha(schrift, 0.8)), "datenportal" => (hex(0xf3f5f8), hex(0x1a1f2b), hex(0x55607a)), "analyse" | "lokal" => (grund, schrift, alpha(schrift, 0.7)), _ => (hex(0xffffff), hex(0x141414), hex(0x555555)) };
    rrect(r.x, r.y, r.w, r.h, 8.0, bg);
    if ui.hover(r) { rrect_line(r.x, r.y, r.w, r.h, 8.0, akzent); }
    let mut y = r.y;
    match typ.as_str() {
        // a data portal: a navy bar with the wordmark, the facts first
        "datenportal" => { rrect(r.x, r.y, r.w, kopf, 8.0, grund); draw_rectangle(r.x, r.y + kopf - 8.0, r.w, 8.0, grund); ui.text_semi(&name, r.x + 12.0, r.y + kopf * 0.68, kopf * 0.55, schrift); draw_rectangle(r.x, r.y + kopf, r.w, 3.0, akzent); y += kopf + 8.0; }
        // the tabloid: the wordmark in a box, the headline shouts in capitals
        "boulevard" => { let w = ui.measure_title(&name, kopf * 0.8) + 16.0; draw_rectangle(r.x + 12.0, r.y + 10.0, w, kopf, schrift); ui.title(&name, r.x + 20.0, r.y + 10.0 + kopf * 0.82, kopf * 0.8, grund); y += kopf + 16.0; }
        // the English tabloid: a red masthead
        "tabloid_en" => { rrect(r.x, r.y, r.w, kopf, 8.0, grund); draw_rectangle(r.x, r.y + kopf - 8.0, r.w, 8.0, grund); ui.title(&name, r.x + 12.0, r.y + kopf * 0.8, kopf * 0.75, schrift); y += kopf + 8.0; }
        // the serious English paper: navy head, yellow line
        "serioes_en" => { rrect(r.x, r.y, r.w, kopf, 8.0, grund); draw_rectangle(r.x, r.y + kopf - 8.0, r.w, 8.0, grund); ui.text_semi(&name, r.x + 12.0, r.y + kopf * 0.7, kopf * 0.6, schrift); draw_rectangle(r.x, r.y + kopf, r.w, 3.0, akzent); y += kopf + 8.0; }
        // TV: dark, a red live dot before the channel
        "tv" => { let puls = 0.6 + 0.4 * (get_time() as f32 * 3.0).sin().abs(); draw_circle(r.x + 20.0, r.y + 20.0, 6.0, alpha(akzent, puls)); ui.text_semi(&name, r.x + 34.0, r.y + 26.0, 16.0, schrift); ui.text_semi(crate::inhalt::t("ui.blog.tv"), r.x + r.w - 14.0 - ui.measure_semi(crate::inhalt::t("ui.blog.tv"), 12.0), r.y + 25.0, 12.0, akzent); y += 40.0; }
        // podcast and fan blog: a round "mic" badge
        "podcast" => { draw_circle(r.x + 24.0, r.y + 24.0, 12.0, akzent); draw_rectangle(r.x + 21.0, r.y + 16.0, 6.0, 11.0, grund); draw_rectangle(r.x + 23.0, r.y + 27.0, 2.0, 5.0, grund); ui.text_semi(&name, r.x + 44.0, r.y + 30.0, 16.0, schrift); y += 44.0; }
        // the local paper: old-fashioned masthead with a rule
        "lokal" => { ui.title(&name, r.x + 14.0, r.y + 30.0, 20.0, akzent); ui.text(crate::inhalt::t("ui.blog.lokales"), r.x + r.w - 14.0 - ui.measure(crate::inhalt::t("ui.blog.lokales"), 12.0), r.y + 26.0, 12.0, akzent); draw_rectangle(r.x + 14.0, r.y + 36.0, r.w - 28.0, 2.0, akzent); y += 44.0; }
        // the club's own channel: club colours
        "verein" => { ui.title(&name, r.x + 14.0, r.y + 30.0, 20.0, akzent); y += 40.0; }
        // the long read: quiet, a small orange "Analyse"
        "analyse" => { ui.text_semi(&name, r.x + 14.0, r.y + 24.0, 16.0, schrift); ui.text_semi(crate::inhalt::t("ui.blog.analyse"), r.x + r.w - 14.0 - ui.measure_semi(crate::inhalt::t("ui.blog.analyse"), 12.0), r.y + 22.0, 12.0, akzent); draw_rectangle(r.x + 14.0, r.y + 32.0, r.w - 28.0, 1.0, hex(0xcccccc)); y += 40.0; }
        // the sports magazine: a red line on top, the wordmark in red
        _ => { draw_rectangle(r.x + 6.0, r.y, r.w - 12.0, 4.0, akzent); ui.title(&name, r.x + 14.0, r.y + 30.0, 22.0, akzent); y += 38.0; }
    }
    let kopfzeile = if typ == "boulevard" || typ == "tabloid_en" { n.head.to_uppercase() } else { n.head.clone() };
    let hfarbe = if typ == "tabloid_en" { hex(0x111111) } else { fg };
    if typ == "tabloid_en" { draw_rectangle(r.x, y - 4.0, r.w, r.h - (y - 4.0 - r.y), hex(0xffffff)); }
    let max_z = if gross { 3 } else { 2 };
    for l in ui.wrap(&kopfzeile, hs, r.w - 28.0).into_iter().take(max_z) {
        if typ == "boulevard" { draw_rectangle(r.x + 12.0, y + hs + 2.0, ui.measure_title(&l, hs) + 6.0, 3.0, akzent); }
        if typ == "boulevard" || typ == "tabloid_en" || typ == "fachmagazin" { ui.title(&l, r.x + 14.0, y + hs, hs, hfarbe); } else { ui.text_semi(&l, r.x + 14.0, y + hs, hs, hfarbe); }
        y += hs + 6.0;
    }
    let tz = if gross { 4 } else { 2 };
    let tfarbe = if typ == "tabloid_en" { hex(0x333333) } else { sub };
    for l in ui.wrap(&n.text, 14.0, r.w - 28.0).into_iter().take(tz) { ui.text(&l, r.x + 14.0, y + 16.0, 14.0, tfarbe); y += 19.0; }
    ui.text(&datum, r.x + 14.0, r.y + r.h - 10.0, 12.0, tfarbe);
}

/// The media page as a web blog (Fabian 23.11.): filters, a lead story, the cards of the outlets, and a sidebar with
/// the most read, the rumour mill and the next press conference. Only what concerns us (nachricht_relevant).
fn webblog(ui: &mut Ui, v: &mut View, c: &mut Career, y0: f32) {
    let alle = c.feed.clone();
    let mut liste: Vec<(News, String)> = vec![];
    for n in alle { if c.nachricht_relevant(&n.head) { let m = if n.medium.is_empty() { c.medium_fuer(&n.head, &n.text).to_string() } else { n.medium.clone() }; liste.push((n, m)); } }
    let filter = [crate::id("alle"), crate::id("transfer"), crate::id("eigen"), crate::id("liga")];
    let fl: Vec<&str> = filter.iter().map(|f| crate::inhalt::t(&format!("ui.blog.filter.{f}"))).collect();
    if let Some(k) = ui.segment(Rect::new(x0() + 4.0, y0, 520.0, 32.0), &fl, v.blog_filter as usize) { v.blog_filter = k as u8; v.scrolls.clear(); }
    let own = c.club().name.clone();
    let liga: Vec<String> = c.world.leagues[c.league()].teams.iter().map(|&t| c.world.clubs[t].name.clone()).collect();
    liste.retain(|(n, m)| match v.blog_filter { 1 => matches!(m.as_str(), "transfermarkt" | "bild" | "sun") || m.starts_with("datenportal|") || m.starts_with("boulevard|") || m.starts_with("tabloid_en|") || n.head.contains("Gerücht"), 2 => n.head.contains(&own) || n.text.contains(&own), 3 => liga.iter().any(|l| n.head.contains(l.as_str())), _ => true });
    let y = y0 + 44.0;
    let hw = cbreite() * 0.66;
    let sw = cbreite() - hw - 14.0;
    if liste.is_empty() { ui.text(crate::inhalt::t("md.ui.leer.schlagzeilen"), x0() + 10.0, y + 30.0, 18.0, Theme::text2()); }
    // the lead story, then two columns of cards
    let mut yy = y;
    let area = Rect::new(x0() + 4.0, y, hw, hoehe() - y - 16.0);
    let zeilen = ((area.h - 250.0) / 186.0).max(1.0) as usize;
    let start = scroll_rows(ui, v, area, liste.len().saturating_sub(1).div_ceil(2), zeilen) * 2;
    if let Some((n, m)) = liste.first() { if start == 0 { artikel_karte(ui, Rect::new(x0() + 4.0, yy, hw, 236.0), n, m, true); yy += 250.0; } }
    let kw = (hw - 12.0) / 2.0;
    for (k, (n, m)) in liste.iter().skip(1 + start).take(zeilen * 2).enumerate() {
        artikel_karte(ui, Rect::new(x0() + 4.0 + (k % 2) as f32 * (kw + 12.0), yy + (k / 2) as f32 * 186.0, kw, 174.0), n, m, false);
    }
    // sidebar
    let sx = x0() + 4.0 + hw + 14.0;
    let r1 = Rect::new(sx, y, sw, 230.0);
    ui.card_titled(r1, crate::inhalt::t("ui.blog.meistgelesen"), Theme::akzent());
    let mut top: Vec<&(News, String)> = liste.iter().collect();
    top.sort_by_key(|(n, _)| std::cmp::Reverse((n.head.contains(&own) as u8, n.day)));
    for (k, (n, m)) in top.iter().take(5).enumerate() {
        let ty = r1.y + 62.0 + k as f32 * 33.0;
        ui.title(&format!("{}", k + 1), r1.x + 14.0, ty + 4.0, 22.0, Theme::akzent());
        ui.text(&ui.fit(&n.head, 14.0, r1.w - 60.0), r1.x + 40.0, ty - 2.0, 14.0, Theme::text());
        ui.text(&medium_stil(m).0, r1.x + 40.0, ty + 14.0, 11.0, Theme::gedaempft());
    }
    let r2 = Rect::new(sx, r1.y + r1.h + 12.0, sw, 220.0);
    ui.card_titled(r2, crate::inhalt::t("ui.blog.geruechte"), Theme::gold());
    let gw = |n: &News| ["Gerücht", "baggert", "Spur", "verhandelt", "macht Ernst"].iter().any(|w| n.head.contains(w)); // roh
    let ger: Vec<&(News, String)> = liste.iter().filter(|(n, _)| gw(n)).take(5).collect();
    if ger.is_empty() { ui.text(crate::inhalt::t("ui.blog.keine_geruechte"), r2.x + 14.0, r2.y + 62.0, 14.0, Theme::text2()); }
    for (k, (n, _)) in ger.iter().enumerate() {
        let ty = r2.y + 62.0 + k as f32 * 30.0;
        let heiss = (n.head.len() + n.day as usize) % 5;
        for f in 0..5 { draw_circle(r2.x + 18.0 + f as f32 * 9.0, ty - 5.0, 3.5, if f <= heiss { Theme::problem() } else { Theme::ueberlagerung() }); }
        ui.text(&ui.fit(&n.head, 13.0, r2.w - 80.0), r2.x + 66.0, ty, 13.0, Theme::text());
    }
    let r3 = Rect::new(sx, r2.y + r2.h + 12.0, sw, (hoehe() - r2.y - r2.h - 40.0).max(80.0));
    ui.card_titled(r3, crate::inhalt::t("ui.blog.pk"), Theme::akzent());
    let pk = if c.press.is_some() { crate::inhalt::t("ui.pages.die_presse_wartet_auf_dich").to_string() } else { c.next_match().map(|f| crate::tf!("ui.blog.pk_vor", date::short(c.world.season, f.day.saturating_sub(1)))).unwrap_or_default() };
    ui.paragraph(&pk, r3.x + 14.0, r3.y + 62.0, 14.0, r3.w - 28.0, Theme::text());
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
    // the press seats (Medien-Konzept 6.8): the journalists in a row, the one asking raised and ringed
    let reihe: Vec<(String, u8, f32)> = c.me.journalisten.iter().take(7).cloned().collect();
    for (k, (n, _, hal)) in reihe.iter().enumerate() {
        let fragt = k == j;
        let (sx, sy) = (r.x + 40.0 + k as f32 * 52.0, r.y + 176.0 - if fragt { 6.0 + 3.0 * (ui.time * 4.0).sin().abs() } else { 0.0 });
        if fragt { draw_circle(sx, sy, 22.0, alpha(Theme::akzent(), 0.35)); }
        ui.figur(sx, sy, if fragt { 18.0 } else { 15.0 }, n, crate::ui::Rolle::Presse);
        draw_circle(sx + 12.0, sy + 12.0, 4.0, if *hal > 15.0 { Theme::gut() } else if *hal < -15.0 { Theme::problem() } else { Theme::gedaempft() });
    }
    // the question as a speech bubble from the journalist
    let qy = r.y + 206.0;
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
    let r = Rect::new(x0() + 4.0, Y0 - 40.0, cbreite(), 410.0);
    ui.card_titled(r, crate::inhalt::t("menu.spielstand_und_karriere.reiter.0"), Theme::akzent());
    ui.text(&crate::tf!("ui.pages.gespeichert_wird_in", c.save_path().display()), r.x + 16.0, r.y + 60.0, 16.0, Theme::text2());
    ui.text(crate::inhalt::t("ui.pages.autosave"), r.x + 16.0, r.y + 88.0, 16.0, Theme::text2());
    if let Some(a) = ui.segment(Rect::new(r.x + 16.0, r.y + 100.0, 270.0, 36.0), &[crate::inhalt::t("ui.pages.autosave.montags"), crate::inhalt::t("ui.pages.autosave.taeglich"), crate::inhalt::t("ui.pages.autosave.monatlich")], c.autosave as usize) { c.autosave = a as u8; let _ = c.save(); }
    if ui.button_gold(Rect::new(r.x + 16.0, r.y + 148.0, 240.0, 44.0), crate::inhalt::t("ui.pages.jetzt_sichern")) { match c.save() { Ok(_) => v.toast(crate::inhalt::t("ui.pages.gesichert"), true), Err(e) => v.toast(&crate::tf!("ui.pages.fehler_beim_sichern", e), false) } }
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
    // the UI scale (75–150 %), saved for every start
    ui.text_semi(crate::inhalt::t("ui.skala.titel"), r.x + 340.0, r.y + 266.0, 17.0, Theme::text());
    let stufen = [0.75f32, 0.9, 1.0, 1.15, 1.3, 1.5];
    let namen: Vec<String> = stufen.iter().map(|s| format!("{:.0} %", s * 100.0)).collect();
    let nr: Vec<&str> = namen.iter().map(|s| s.as_str()).collect();
    let jetzt = stufen.iter().position(|s| (s - crate::ui::ui_skala()).abs() < 0.01).unwrap_or(2);
    if let Some(k) = ui.segment(Rect::new(r.x + 340.0, r.y + 280.0, 400.0, 34.0), &nr, jetzt) {
        crate::ui::ui_skala_setzen(stufen[k]);
        let _ = std::fs::create_dir_all(save_dir());
        let _ = std::fs::write(skala_file(), format!("{}", stufen[k]));
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
    // phone access (konzepte/mobile-ansicht.md): switch, address, pairing code with QR, paired phones
    {
        let hr = Rect::new(r.x + r.w - 380.0, r.y + 266.0, 364.0, 60.0);
        let an = crate::handy::an();
        if ui.button(Rect::new(hr.x, hr.y + 14.0, 170.0, 34.0), crate::inhalt::t(if an { "ui.handy.an" } else { "ui.handy.aus" }), an) { crate::handy::setzen(!an); }
        if let Some((adr, code, verbunden)) = crate::handy::info() {
            ui.text_semi(&code, hr.x + 184.0, hr.y + 40.0, 26.0, Theme::akzent());
            if verbunden { ui.text(crate::inhalt::t("ui.handy.verbunden"), hr.x + 290.0, hr.y + 38.0, 13.0, Theme::gut()); }
            let url = format!("{adr}?c={code}");
            ui.text(&crate::tf!("ui.handy.hinweis", adr.clone()), r.x + 760.0, r.y + 296.0, 12.0, Theme::text2());
            if let Ok(q) = qrcode::QrCode::new(url.as_bytes()) {
                let farben = q.to_colors();
                let n = q.width();
                let px = 3.0;
                let (qx, qy) = (r.x + r.w - 380.0 - n as f32 * px - 30.0, r.y + 230.0);
                draw_rectangle(qx - 6.0, qy - 6.0, n as f32 * px + 12.0, n as f32 * px + 12.0, hex(0xffffff));
                for (i, f) in farben.iter().enumerate() { if *f == qrcode::Color::Dark { draw_rectangle(qx + (i % n) as f32 * px, qy + (i / n) as f32 * px, px, px, hex(0x000000)); } }
            }
        }
        let g = crate::handy::geraete();
        if !g.is_empty() {
            ui.text(&crate::tf!("ui.handy.geraete", g.iter().map(|x| x.name.clone()).collect::<Vec<_>>().join(", ")), hr.x, hr.y + 4.0, 12.0, Theme::text2());
            if ui.button(Rect::new(hr.x + 184.0 + 0.0, hr.y - 30.0, 180.0, 26.0), crate::inhalt::t("ui.handy.entfernen"), false) { crate::handy::geraete_speichern(&[]); }
        }
    }
    // background music (musik.rs): a radio stream or YouTube playlist, pasted from the clipboard; paused during a match
    {
        let url = crate::musik::url();
        let (mpv, ytdlp) = crate::musik::programme();
        ui.text_semi(crate::inhalt::t("ui.musik.titel"), r.x + 16.0, r.y + 340.0, 17.0, Theme::text());
        if ui.button(Rect::new(r.x + 16.0, r.y + 354.0, 300.0, 34.0), crate::inhalt::t("ui.musik.einfuegen"), false) {
            let neu = macroquad::miniquad::window::clipboard_get().unwrap_or_default().trim().to_string();
            if neu.starts_with(crate::id("http")) { crate::musik::setzen(&neu); ui.play(crate::id("bestaetigt")); } else { v.toast(crate::inhalt::t("ui.musik.keine_url"), false); }
        }
        if ui.button(Rect::new(r.x + 328.0, r.y + 354.0, 120.0, 34.0), crate::inhalt::t("ui.klang.aus"), url.is_empty()) { crate::musik::setzen(""); }
        if !url.is_empty() && ui.button(Rect::new(r.x + 460.0, r.y + 354.0, 150.0, 34.0), crate::inhalt::t("ui.musik.weiter"), false) { crate::musik::weiter(); }
        let zeile = if !mpv { crate::inhalt::t("ui.musik.ohne_mpv").to_string() }
            else if url.is_empty() { crate::inhalt::t("ui.musik.hinweis").to_string() }
            else if crate::musik::ist_youtube(&url) && !ytdlp { crate::inhalt::t("ui.musik.ohne_ytdlp").to_string() }
            else { crate::tf!("ui.musik.spielt", ui.fit(&url, 13.0, r.w - 500.0)) };
        ui.paragraph(&zeile, r.x + 624.0, r.y + 356.0, 13.0, r.w - 640.0, if mpv { Theme::text2() } else { Theme::achtung() });
    }
    let r2 = Rect::new(x0() + 4.0, Y0 + 384.0, cbreite(), hoehe() - Y0 - 400.0);
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
        if cfg!(windows) { crate::tf!("ui.pages.speicher_direkt", crate::fl27::freischalten_hinweis()) } else { crate::tf!("ui.pages.speicher_helfer", if crate::fl27::helfer_pfad().exists() { crate::inhalt::t("ui.wort.gebaut") } else { crate::inhalt::t("ui.wort.fehlt") }, crate::fl27::freischalten_hinweis()) },
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

fn spieltag(ui: &mut Ui, v: &mut View, c: &mut Career) {
    if v.fl27.is_none() && !v.fl27_vorschau { v.fl27_ansicht = None; v.fl27_seq = None; v.fl27_einblendung = None; }
    if v.mv.is_none() {
        let Some(fi) = c.today_match() else {
            ui.title(crate::inhalt::t("ui.pages.heute_ist_kein_spiel"), x0() + 10.0, Y0 + 40.0, 30.0, Theme::text());
            if let Some(f) = c.next_match() { ui.text(&crate::tf!("ui.pages.naechstes_spiel_gegen", date::long(c.world.season, f.day), c.world.clubs[if f.h == c.own { f.a } else { f.h }].name), x0() + 10.0, Y0 + 76.0, 19.0, Theme::text2()); }
            if let Some(r) = c.last_result.clone() { match_report(ui, c, &r, Rect::new(x0() + 4.0, Y0 + 110.0, cbreite(), hoehe() - Y0 - 126.0)); }
            return;
        };
        let f = c.fixture(fi);
        v.mv = Some(MatchView { res: MatchResult { h: f.h, a: f.a, ..Default::default() }, minute: 0.0, speed: 1.0, phase: 0, ended_at: 0.0, manual: (0, 0), manual_open: false, halbzeit: 0, wirkung: vec![], nach_gesagt: false, angesprochen: vec![], eingriff: Default::default(), vor_pause: 1.0 });
    }
    let mut mv = v.mv.take().unwrap();
    let (h, a) = (mv.res.h, mv.res.a);
    // scoreboard
    let sb = Rect::new(x0() + 4.0, Y0 - 42.0, cbreite(), 110.0);
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
    let lx = if ui.wettbewerb_logo(sb.x + 130.0, sb.y + 85.0, 10.0, cid) { sb.x + 146.0 } else { sb.x + 120.0 };
    ui.text(&format!("{} · {}", comp, date::long(c.world.season, c.world.day)), lx, sb.y + 90.0, 15.0, Theme::text2());
    let body = Rect::new(x0() + 4.0, sb.y + sb.h + 12.0, cbreite(), hoehe() - sb.y - sb.h - 28.0);
    match mv.phase {
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
            if mv.halbzeit != 1 && !im_intro { mv.minute += get_frame_time() * 7.5 * mv.speed; }
            if mv.halbzeit == 0 && before < 45.0 && mv.minute >= 45.0 { mv.minute = 45.0; mv.halbzeit = 1; }
            for e in mv.res.events.iter().filter(|e| (e.min as f32) > before && (e.min as f32) <= mv.minute) { match e.kind { 0 | 1 => ui.play(crate::id("tor")), 2 | 3 => ui.play(crate::id("karte")), 4 => ui.play(crate::id("verletzung")), _ => {} } }
            let r = Rect::new(body.x, body.y, body.w * 0.62, body.h);
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
            // pause (Fabian 23.11.): the clock stands, grades and interventions stay usable; P toggles it too
            let sp = [0.0, 1.0, 2.0, 4.0];
            if is_key_pressed(KeyCode::P) && mv.minute < 90.0 { if mv.speed == 0.0 { mv.speed = mv.vor_pause.max(1.0); } else { mv.vor_pause = mv.speed; mv.speed = 0.0; } }
            let sel = sp.iter().position(|s| *s == mv.speed).unwrap_or(1);
            if let Some(s) = ui.segment(Rect::new(r2.x + 14.0, r2.y + 44.0, r2.w - 28.0, 34.0), &[crate::inhalt::t("ui.live.pause"), crate::inhalt::t("programm.intensitaet.normal"), crate::inhalt::t("ui.pages.schnell"), crate::inhalt::t("ui.pages.sehr_schnell")], sel) { if sp[s] == 0.0 { mv.vor_pause = mv.speed; } mv.speed = sp[s]; }
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
            // the live board and the interventions (Fabian 23.11.)
            let bh = ((r2.w - 28.0) * 0.62).min(260.0).min((r2.h - 180.0) * 0.3);
            live_board(ui, c, &mv, Rect::new(r2.x + 14.0, r2.y + 156.0, r2.w - 28.0, bh));
            if mv.minute < 90.0 { eingriffe(ui, v, c, &mut mv, Rect::new(r2.x + 14.0, r2.y + 166.0 + bh, r2.w - 28.0, r2.h - 180.0 - bh)); }
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
        if ui.button(Rect::new(r.x + 660.0, by - 44.0, 26.0, 26.0), "−", false) { if let Err(e) = c.rueckennummer(p, -1) { v.toast(&e, false); } }
        if ui.button(Rect::new(r.x + 690.0, by - 44.0, 26.0, 26.0), "+", false) { if let Err(e) = c.rueckennummer(p, 1) { v.toast(&e, false); } }
        if pl.leihe.as_ref().map_or(false, |l| l.von != c.own && l.option > 0.0) {
            if ui.button_gold(Rect::new(r.x + 24.0, by - 50.0, 260.0, 40.0), &crate::tf!("ui.pages.kaufoption", money(pl.leihe.as_ref().unwrap().option))) { match c.kaufoption(p) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.kaufoption_gezogen"), true), Err(e) => v.toast(&e, false) } }
        } else if pl.leihe.is_none() && ui.button(Rect::new(r.x + 24.0, by - 50.0, 180.0, 40.0), crate::inhalt::t(if pl.ps.leihliste { "ui.leihe.von_liste" } else { "ui.leihe.auf_leihliste" }), pl.ps.leihliste) {
            if pl.ps.leihliste { c.world.players[p].ps.leihliste = false; } else { match c.leihe_abgeben(p) { Ok(()) => v.toast(crate::inhalt::t("ui.leihe.auf_liste"), true), Err(e) => v.toast(&e, false) } }
        }
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
    let area = Rect::new(x0() + 4.0, Y0 - 40.0, cbreite(), hoehe() - Y0 + 24.0);
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
    let felder = [Rect::new(x0() + 4.0, Y0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0, bw, bh), Rect::new(x0() + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
    // dates: friendlies with simulate / play in FL27 (default simulate), trips; results
    let r = felder[0];
    ui.card_titled(r, t("ui.vorb.termine"), Theme::akzent());
    if c.lager_laeuft().is_some() && ui.button(Rect::new(r.x + r.w - 230.0, r.y + 4.0, 220.0, 28.0), t("ui.lager.vorspulen"), true) { match c.lager_vorspulen() { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
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

/// The look of a competition (Fabian 22.11.: like the real leagues): colours and pattern from wettbewerbslook.toml by key,
/// by kind for Europe (1 CL, 2 EL, 3 Conference), else the country's flag colours.
pub struct Look { pub c1: Color, pub c2: Color, pub akzent: Color, pub akzent_lesbar: Color, pub text: Color, pub text2: Color, pub muster: String }
/// The colour of the chosen competition for the page's head band on the tables and rankings (Fabian 23.11.).
pub fn liga_farbe(v: &View, c: &Career) -> u32 {
    let l = v.table_league.unwrap_or(c.league());
    let lk = if l >= 1000 && l - 1000 < c.world.cups.len() {
        let cu = &c.world.cups[l - 1000];
        let land = if cu.country.is_empty() { c.world.leagues.iter().find(|x| x.key == cu.key).map(|x| x.land.clone()).unwrap_or_default() } else { cu.country.clone() };
        wettbewerb_look(c, &cu.key, cu.kind, &land)
    } else { let lg = &c.world.leagues[l.min(c.world.leagues.len() - 1)]; wettbewerb_look(c, &lg.key, 9, &lg.land) };
    let k = |x: f32| ((x * 255.0).round() as u32).min(255);
    (k(lk.c1.r) << 16) | (k(lk.c1.g) << 8) | k(lk.c1.b)
}
pub fn wettbewerb_look(_c: &Career, key: &str, kind: u8, land: &str) -> Look {
    let farbe = |v: &serde_json::Value| u32::from_str_radix(v.as_str().unwrap_or("").trim_start_matches('#'), 16).ok();
    let aus = |w: &serde_json::Value| -> Option<(u32, u32, u32, String)> { Some((farbe(&w["farbe1"])?, farbe(&w["farbe2"])?, farbe(&w["akzent"])?, w["muster"].as_str().unwrap_or("keins").to_string())) };
    let art = match kind { 1 => Some(crate::id("cl")), 2 => Some(crate::id("el")), 3 => Some(crate::id("uecl")), _ => None };
    let eintrag = art.and_then(|a| crate::regeln::wert_oder("wettbewerbslook", &format!("art.{a}")).and_then(aus))
        .or_else(|| crate::regeln::wert_oder("wettbewerbslook", "liga").and_then(|l| l.get(key)).and_then(aus))
        .or_else(|| crate::regeln::wert_oder("wettbewerbslook", "land").and_then(|l| l.get(land)).and_then(|v| { let a = v.as_array()?; let (x, y) = (farbe(&a[0])?, farbe(&a[1])?); Some((x, y, y, crate::id("keins").to_string())) }));
    let (c1, c2, ak, muster) = eintrag.unwrap_or((0x1e293b, 0x334155, 0x94a3b8, crate::id("keins").to_string()));
    let (c1, c2, ak) = (hex(c1), hex(c2), hex(ak));
    let text = crate::ui::on(c1);
    Look { c1, c2, akzent: ak, akzent_lesbar: crate::ui::lesbar(ak, Theme::flaeche(), 3.0), text, text2: alpha(text, 0.8), muster }
}
/// A five-pointed star (the Champions League pattern).
fn stern(x: f32, y: f32, r: f32, col: Color) {
    let p = |k: usize, rr: f32| { let a = -std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 5.0; vec2(x + rr * a.cos(), y + rr * a.sin()) };
    for k in 0..10 { draw_triangle(vec2(x, y), p(k, if k % 2 == 0 { r } else { r * 0.42 }), p(k + 1, if (k + 1) % 2 == 0 { r } else { r * 0.42 }), col); }
}
/// The banner over a competition page: gradient in its colours, its pattern, the logo on a light disc and the name.
/// Returns the x where text after the name may start.
pub fn wettbewerb_banner(ui: &mut Ui, r: Rect, lk: &Look, cid: Option<u16>, name: &str) -> f32 {
    rrect(r.x, r.y, r.w, r.h, 10.0, lk.c1);
    gradient_h(r.x + r.w * 0.35, r.y, r.w * 0.65 - 10.0, r.h, alpha(lk.c2, 0.0), alpha(lk.c2, 0.9));
    match lk.muster.as_str() {
        "sterne" => { for k in 0..14 { let fx = ((k * 7919) % 100) as f32 / 100.0; let fy = ((k * 104729) % 100) as f32 / 100.0; stern(r.x + r.w * (0.45 + 0.53 * fx), r.y + 6.0 + (r.h - 12.0) * fy, 3.0 + (k % 3) as f32 * 1.6, alpha(lk.akzent, 0.55)); } }
        "streifen" => { for k in 0..8 { let x0 = r.x + r.w * 0.55 + k as f32 * 26.0; draw_triangle(vec2(x0, r.y + r.h), vec2(x0 + 12.0, r.y + r.h), vec2(x0 + 34.0, r.y), alpha(lk.akzent, 0.18)); draw_triangle(vec2(x0, r.y + r.h), vec2(x0 + 34.0, r.y), vec2(x0 + 22.0, r.y), alpha(lk.akzent, 0.18)); } }
        _ => {}
    }
    draw_rectangle(r.x + 10.0, r.y + r.h - 3.0, r.w - 20.0, 3.0, lk.akzent);
    let mut tx = r.x + 16.0;
    if cid.is_some() {
        draw_circle(r.x + 30.0, r.y + r.h / 2.0, 19.0, alpha(WHITE, 0.92));
        if ui.wettbewerb_logo(r.x + 30.0, r.y + r.h / 2.0, 15.0, cid) { tx = r.x + 58.0; }
    }
    let nm = ui.fit(name, 26.0, r.w * 0.5);
    ui.title(&nm, tx, r.y + r.h / 2.0 + 10.0, 26.0, lk.text);
    tx + ui.measure_title(&nm, 26.0)
}

/// One tie of a knock-out round: the two clubs (None = not decided yet), the score text and the winner.
#[derive(Clone)]
struct Paarung { a: Option<usize>, b: Option<usize>, text: String, sieger: Option<usize> }

/// The bracket of a cup from the first round with at most eight ties: the rounds as columns from both sides towards
/// the final in the middle; ties of a round sit next to the tie of the next round their winners met in, rounds not
/// drawn yet as empty boxes. False when no round is small enough yet (then the list is shown).
fn turnierbaum(ui: &mut Ui, c: &Career, ci: usize, r: Rect, lk: &Look) -> bool {
    let cup = &c.world.cups[ci];
    let mut stages: Vec<u8> = cup.fx.iter().map(|f| f.stage).filter(|s| *s >= 20).collect();
    stages.sort();
    stages.dedup();
    let paare = |st: u8| -> Vec<Paarung> {
        let mut v: Vec<Paarung> = vec![];
        for f in cup.fx.iter().filter(|f| f.stage == st) {
            let (h, a) = (f.f.h, f.f.a);
            if v.iter().any(|p| (p.a == Some(h) && p.b == Some(a)) || (p.a == Some(a) && p.b == Some(h))) { continue; }
            let legs: Vec<&crate::cups::CupFx> = cup.fx.iter().filter(|x| x.stage == st && ((x.f.h == h && x.f.a == a) || (x.f.h == a && x.f.a == h))).collect();
            let gespielt = legs.iter().all(|x| x.f.hg >= 0);
            let text = if !gespielt { legs.iter().find(|x| x.f.hg < 0).map(|x| date::short(c.world.season, x.f.day)).unwrap_or_default() } else {
                let (mut ta, mut tb) = (0i32, 0i32);
                for x in &legs { if x.f.h == h { ta += x.f.hg as i32; tb += x.f.ag as i32; } else { ta += x.f.ag as i32; tb += x.f.hg as i32; } }
                let pens = legs.iter().find_map(|x| x.pens).map(|p| crate::tf!("ui.pages.kurz_ie", crate::cups::pens_score(p)).trim().to_string()).unwrap_or_default();
                format!("{ta}:{tb}{}", if pens.is_empty() { String::new() } else { format!(" {pens}") })
            };
            let sieger = if gespielt { Some(c.world.winner_of(ci, st, h, a)) } else { None };
            v.push(Paarung { a: Some(h), b: Some(a), text, sieger });
        }
        v
    };
    let Some(erste) = stages.iter().position(|&st| { let n = paare(st).len(); n > 0 && n <= 8 }) else { return false };
    let n0 = paare(stages[erste]).len().next_power_of_two();
    let runden = (n0 as f32).log2() as usize + 1;
    let mut baum: Vec<Vec<Paarung>> = (0..runden).map(|k| stages.get(erste + k).map(|&st| paare(st)).unwrap_or_default()).collect();
    // empty boxes for rounds still to come, then order every round after the next one (backwards from the final)
    for k in 0..runden { let soll = n0 >> k; while baum[k].len() < soll { baum[k].push(Paarung { a: None, b: None, text: String::new(), sieger: None }); } }
    for k in (0..runden - 1).rev() {
        let mut neu: Vec<Paarung> = vec![];
        let mut rest = baum[k].clone();
        for p in &baum[k + 1] {
            for t in [p.a, p.b] {
                let pos = t.and_then(|t| rest.iter().position(|x| x.sieger == Some(t) || x.a == Some(t) || x.b == Some(t)));
                if let Some(i) = pos { neu.push(rest.remove(i)); } else if !rest.is_empty() { neu.push(rest.remove(0)); }
            }
        }
        neu.extend(rest);
        baum[k] = neu;
    }
    // columns: rounds 0..runden-2 on the left (first half of each), the final in the middle, the right half mirrored
    let spalten = 2 * (runden - 1) + 1;
    let sw = r.w / spalten as f32;
    let bw = (sw - 10.0).min(150.0);
    let bh = 40.0;
    let mitte_y = |k: usize, i: usize, n: usize| -> f32 { let h = r.h - 30.0; r.y + 24.0 + h * (i as f32 + 0.5) / n as f32 };
    let kasten = |ui: &mut Ui, x: f32, y: f32, p: &Paarung, finale: bool| {
        let br = Rect::new(x - bw / 2.0, y - bh / 2.0, bw, bh);
        rrect(br.x, br.y, br.w, br.h, 6.0, if finale { alpha(lk.c1, 0.9) } else { Theme::erhoehung() });
        if finale { rrect_line(br.x, br.y, br.w, br.h, 6.0, lk.akzent); }
        let farbe = if finale { lk.text } else { Theme::text() };
        for (j, t) in [p.a, p.b].iter().enumerate() {
            let yy = br.y + 14.0 + j as f32 * 18.0;
            match t {
                Some(k) => {
                    let cl = &c.world.clubs[*k];
                    ui.club_crest(br.x + 11.0, yy - 4.0, 7.0, cl.id, hex(cl.c1), &cl.short);
                    let raus = p.sieger.is_some() && p.sieger != Some(*k);
                    let col = if *k == c.own { Theme::gold() } else if raus { alpha(farbe, 0.45) } else { farbe };
                    ui.text(&ui.fit(&cl.short, 13.0, bw - 30.0), br.x + 22.0, yy, 13.0, col);
                }
                None => { ui.text("–", br.x + 22.0, yy, 13.0, alpha(farbe, 0.4)); }
            }
        }
        if !p.text.is_empty() { ui.text_right(&ui.fit(&p.text, 11.0, bw * 0.5), br.x + br.w - 6.0, br.y + 22.0, 11.0, if finale { lk.text2 } else { Theme::text2() }); }
    };
    let linie = alpha(Theme::text2(), 0.45);
    for k in 0..runden {
        let n = baum[k].len();
        if k == runden - 1 {
            let x = r.x + sw * (runden - 1) as f32 + sw / 2.0;
            ui.text_center(&crate::cups::stage_name(34, cup.kind), x, r.y + 12.0, 13.0, lk.akzent_lesbar);
            if let Some(w) = baum[k][0].sieger { let cl = &c.world.clubs[w]; ui.club_crest(x, r.y + r.h / 2.0 - 70.0, 26.0, cl.id, hex(cl.c1), &cl.short); ui.text_center(&crate::tf!("ui.pages.sieger", cl.name), x, r.y + r.h / 2.0 - 34.0, 13.0, Theme::gold()); }
            kasten(ui, x, (r.y + 24.0 + r.y + r.h - 6.0) / 2.0, &baum[k][0], true);
            continue;
        }
        let haelfte = n / 2;
        let name = stages.get(erste + k).map(|&st| crate::cups::stage_name(st, cup.kind)).unwrap_or_default();
        for seite in 0..2 {
            let spalte = if seite == 0 { k } else { spalten - 1 - k };
            let x = r.x + sw * spalte as f32 + sw / 2.0;
            ui.text_center(&ui.fit(&name, 12.0, sw), x, r.y + 12.0, 12.0, Theme::gedaempft());
            for i in 0..haelfte {
                let p = &baum[k][seite * haelfte + i];
                let y = mitte_y(k, i, haelfte);
                // connector towards the next round
                let nx = if seite == 0 { x + sw / 2.0 } else { x - sw / 2.0 };
                let kante = if seite == 0 { x + bw / 2.0 } else { x - bw / 2.0 };
                draw_line(kante, y, nx, y, 1.5, linie);
                let ny = if k + 2 == runden { (r.y + 24.0 + r.y + r.h - 6.0) / 2.0 } else { mitte_y(k + 1, i / 2, (haelfte / 2).max(1)) };
                draw_line(nx, y, nx, ny, 1.5, linie);
                kasten(ui, x, y, p, false);
            }
        }
    }
    true
}
