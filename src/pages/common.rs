use super::*;

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
    Awards, TheBest, LigaPreise, Privat, Uefa, Kadermeldung, Spielplan, Vereinsstatistik, TrainerBaum, Ziele, Manager, Stadion, Personalmarkt, Geschichte, Tagebuch, Klima, Marken, Kaderplanung, Vergleich, Entwicklung, Vorbereitung, Analyse, Zeitung, Fans, Ranglisten, Vereine, Vereinswelt, Portal, Handbuch, Regeln, Profi, Spielerprofil, Zentrale, Kooperationen, ZweiteMannschaft, Jugendnetz, Merkliste, Trainerteam, Nationalteams,
}

/// The side menu: group → main pages → their tabs. Every page keeps its id (links from the inbox, ticker and so on
/// open the right tab); a menu item opens the tab last used on that main page.
pub type Haupt = (&'static str, &'static [(Page, &'static str)]);
pub const MENU: [(&str, &[Haupt]); 7] = [
    ("ui.pages.buero", &[("bereich.zentrale", &[(Page::Zentrale, "bereich.zentrale"), (Page::Uebersicht, "reiter.uebersicht")]), ("zentrale.k.posteingang", &[(Page::Posteingang, "zentrale.k.posteingang")]), ("gl.kopf.ergebnisse.alle", &[(Page::Kalender, "gl.kopf.ergebnisse.alle")])]),
    ("ui.pages.mannschaft", &[("zentrale.k.kader", &[(Page::Kader, "zentrale.k.kader"), (Page::ZweiteMannschaft, "ui.game.zweite_mannschaft"), (Page::Kooperationen, "ui.farm.kooperationen"), (Page::Kaderplanung, "aktion.kaderplanung"), (Page::Kadermeldung, "ui.meldung.seite"), (Page::Vergleich, "menu.spielervergleich.titel")]),
        ("gb2.reiter.taktik", &[(Page::Aufstellung, "aktion.aufstellung"), (Page::Vorbereitung, "ui.pages.spielvorbereitung"), (Page::Analyse, "menu.analyse.titel")]),
        ("vg.pg.dim.kabine", &[(Page::Kabine, "vg.pg.dim.kabine")]),
        ("status.training", &[(Page::Training, "status.training"), (Page::Entwicklung, "profil.entwicklung_verlauf"), (Page::Medizin, "kachel.medizin")])]),
    ("ui.pages.markt", &[("bereich.transfers", &[(Page::Transfermarkt, "ux.abschnitt.markt"), (Page::Verhandlungen, "ux.abschnitt.verhandlungen"), (Page::Vertraege, "as.reiter.vertraege"), (Page::Merkliste, "aka.pille.merkliste")]), ("profil.teil.8", &[(Page::Scouting, "profil.teil.8")])]),
    ("ui.pages.jugend", &[("aka.gruppe.akademie", &[(Page::Akademie, "aka.gruppe.akademie"), (Page::Jugendnetz, "ui.main.jugendnetz")])]),
    ("ui.pages.verein", &[("klima.praesidium", &[(Page::Praesidium, "klima.praesidium"), (Page::Ziele, "aktion.ziele"), (Page::Klima, "ui.main.vereinsklima"), (Page::Geschichte, "menu.vereinsgeschichte.titel")]),
        ("bereich.finanzen", &[(Page::Finanzen, "bereich.finanzen"), (Page::Stadion, "menu.stadion_und_anlagen.titel"), (Page::Marken, "ui.main.marken_und_sponsoren")]),
        ("md.ui.chip.personal", &[(Page::Trainerteam, "ui.main.trainerteam"), (Page::Personalmarkt, "ui.main.personalmarkt")]),
        ("taktik.board.coach", &[(Page::Trainer, "aktion.trainerprofil"), (Page::Manager, "gm.mg.hero"), (Page::TrainerBaum, "ui.mst.seite"), (Page::Tagebuch, "gm.tb.hero"), (Page::Privat, "ui.privat.seite")])]),
    ("ui.pages.welt", &[("zentrale.wettbewerbe", &[(Page::Tabellen, "ui.pages.tabellen"), (Page::Spielplan, "ui.spielplan.seite"), (Page::Ranglisten, "ux.abschnitt.rankings"), (Page::Vereinsstatistik, "ui.vstat.seite"), (Page::Uefa, "ui.uefa.seite"), (Page::Nationalteams, "ui.nt.seite")]),
        ("ui.award.seite", &[(Page::Awards, "ui.preis.ballon_seite"), (Page::TheBest, "ui.preis.best_seite"), (Page::LigaPreise, "ui.preis.liga_seite")]),
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
/// Player opens the full-page profile at once (Fabian 23.11.: no quick view in between); Aktionen is the old card with
/// the actions the page has no room for (shirt number, loan, retraining, release, buy option).
pub enum Modal { None, Player(usize), Aktionen(usize), Neg(usize), Suche, Spielplan(usize) }

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
    /// The way into a match (Fabian 23.11., like Anstoss): 0 stadium and city, 1 head-to-head, 2 team talk, 3 kick-off.
    pub vorlauf: u8,
    /// Interventions in the running match (Fabian 23.11.): mentality −1/0/1, the minute of the last shout, the own player chosen to come off.
    pub eingriff: Eingriff,
    /// The speed before a pause, to go on with it.
    pub vor_pause: f32,
    /// The voices of the duel screen (pundit, press, one interview per side), written once per match.
    pub duell: Vec<String>,
    /// The other matches of our league on this day, played at kick-off so they can run alongside (Fabian 24.11.).
    pub parallel: Vec<(usize, usize, MatchResult)>,
}
#[derive(Clone, Default)]
pub struct Eingriff { pub mentalitaet: i8, pub zuruf_min: f32, pub raus: Option<usize>, pub intro: f32, pub tor_gesehen: usize, pub tor_t: f32, pub reiter: u8, pub nach_reiter: u8, pub nach_stimmen: Vec<String> }

pub struct View {
    /// World map: zoom and offset (geo.rs).
    pub karte_zoom: f32,
    pub karte_off: (f32, f32),
    /// The jingle of the reward moment on screen was played.
    pub feier_ton: bool,
    /// When the current full-screen moment appeared (for its grow-in).
    pub feier_start: f32,
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
    /// Einstellungen › Spielstand: back to the start screen (main.rs acts on it), the new name being typed, a copy running.
    pub zum_start: bool,
    /// Loan request in the player card: duration index (6/12/18/24 months) and buy terms (none, option, obligation).
    pub leih_wahl: (usize, usize),
    /// Finanzen › Budget: from pot, to pot, share index.
    pub topf_wahl: (usize, usize, usize),
    /// Player comparison: up to three players and the search text of the column being chosen.
    pub vergleich: [Option<usize>; 3],
    pub vergleich_suche: (usize, String),
    /// Start time of the draw ceremony being shown.
    pub auslosung_t0: f32,
    /// Academy: the rival academy whose squad is shown.
    pub jugend_verein: Option<usize>,
    pub name_neu: Option<String>,
    /// Tactics board: a bench player being dragged and where the drag started (Fabian 23.11.: drag and drop).
    pub bank_zug: Option<(usize, Vec2)>,
    /// Nationalteams › Länder: the chosen nation and the list scroll.
    pub nt_wahl: Option<u16>,
    pub nt_scroll: usize,
    /// Nationalteams › Länder: confederation filter (0 all) and sort (0 strength, 1 name).
    pub nt_verband: usize,
    pub nt_sort: usize,
    pub nt_u21: usize,
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
    /// The right-click menu of a club (Fabian 24.11.): the club and where it opened.
    pub verein_kontext: Option<(usize, Vec2)>,
    /// Which formation the tactics pitch shows: 0 kick-off, 1 with the ball, 2 against the ball.
    pub taktik_ansicht: u8,
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
pub(super) fn klang_file() -> std::path::PathBuf { save_dir().join(crate::id("../klang.txt")) }
/// The stored volume (klang.txt: aus, leise, normal); default leise (SPIELGEFUEHL.md: „Standard leise“).
pub fn klang_lesen() -> f32 { klang_wert(std::fs::read_to_string(klang_file()).unwrap_or_default().trim()) }
pub(super) fn klang_wert(s: &str) -> f32 { match s { "aus" => 0.0, "normal" => 0.6, _ => 0.3 } }
/// Einstellungen › Anzeige: an own accent colour (hex) instead of the crest's, beside thema.txt.
pub fn akzent_file() -> std::path::PathBuf { save_dir().join("../akzent.txt") } // roh
static AKZENT: std::sync::Mutex<Option<Option<u32>>> = std::sync::Mutex::new(None);
pub fn eigener_akzent() -> Option<u32> {
    *AKZENT.lock().unwrap().get_or_insert_with(|| std::fs::read_to_string(akzent_file()).ok().and_then(|s| u32::from_str_radix(s.trim().trim_start_matches('#'), 16).ok()))
}
pub(super) fn akzent_setzen(f: Option<u32>) {
    let _ = std::fs::create_dir_all(save_dir());
    match f { Some(x) => { let _ = std::fs::write(akzent_file(), format!("{x:06x}")); } None => { let _ = std::fs::remove_file(akzent_file()); } }
    *AKZENT.lock().unwrap() = Some(f);
}
pub(super) fn theme_file() -> std::path::PathBuf { save_dir().join(crate::id("../thema.txt")) }
/// Einstellungen → Sprache: "de" or "en", beside thema.txt (read at start in main.rs).
pub fn skala_file() -> std::path::PathBuf { save_dir().join("../skala.txt") } // roh
pub fn sprache_file() -> std::path::PathBuf { save_dir().join(crate::id("../sprache.txt")) }
impl View {
    pub fn new() -> View {
        View { karte_zoom: 1.0, karte_off: (0.0, 0.0), feier_ton: false, feier_start: 0.0, page: Page::Uebersicht, back: vec![], fwd: vec![], modal: Modal::None, tab: 0, scroll: 0.0, scrolls: Default::default(), ziehen: None, tafel_zug: None, leih_wahl: (1, 0), topf_wahl: (0, 1, 1), vergleich: [None; 3], vergleich_suche: (usize::MAX, String::new()), auslosung_t0: 0.0, jugend_verein: None, zum_start: false, name_neu: None, bank_zug: None, nt_wahl: None, nt_scroll: 0, nt_verband: 0, nt_sort: 0, nt_u21: 0, auswahl: None, auswahl_scroll: 0, vereine_liga: None, post_folgen: None, sel_msg: None, msg_filter: 0, lager_reiter: 0, lager_tage: 7, halbzeit_fl27: None, sel_player: None,
            market_pos: 0, market_age: 40, market_price: 0, market_genau: 0, market_gut: 0, market_stat: 0, market_scope: 0, market_text: String::new(), market_focus: false, sp_edit: Default::default(), sp_treffer: (Default::default(), u32::MAX, vec![]), pm_wahl: (u32::MAX, [0; 3]), kr: (50, 2), auswahl_suche: String::new(), menu_alle: false, suche_text: String::new(), vorb_tag: 0, vorb_tore: (0, 0), suche_treffer: vec![], suche_fuer: String::new(), market_cache: vec![], market_key: String::new(), sortierung: Default::default(), neg_tipp: 0.0, blog_filter: 0, tp_zelle: None,
            table_league: None, cal_month: -1, pick: None, toast: None, mv: None, sel_neg: None, changed_at: 0.0, hud_seen: vec![], league_scroll: 0, sel_club: None, kontext: None, verein_kontext: None, taktik_ansicht: 0,
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
pub(super) fn scroll_rows(ui: &Ui, v: &mut View, area: Rect, total: usize, visible: usize) -> usize { scroll_schritt(ui, v, area, total, visible, 3.0) }
/// The same with its own step per wheel notch (blocks of different height move one at a time).
pub(super) fn scroll_schritt(ui: &Ui, v: &mut View, area: Rect, total: usize, visible: usize, schritt: f32) -> usize {
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

pub(super) fn tabs(ui: &mut Ui, v: &mut View, items: &[&str]) {
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

pub(super) fn kpi(ui: &Ui, r: Rect, label: &str, value: &str, sub: &str, edge: Color) {
    ui.card(r);
    draw_rectangle(r.x, r.y, r.w, 3.0, edge);
    ui.text_semi(label, r.x + 16.0, r.y + 28.0, 15.0, Theme::text2());
    if r.h < 90.0 { ui.title(value, r.x + 16.0, r.y + 58.0, 28.0, Theme::text()); return; } // compact: no sub line
    ui.title(value, r.x + 16.0, r.y + 62.0, 30.0, Theme::text());
    ui.text(sub, r.x + 16.0, r.y + 84.0, 15.0, Theme::gedaempft());
}

/// Expected development of the overall per season (entwicklung.rs ent_erwartung) as an arrow with the number.
pub(super) fn entwicklung_pfeil(ui: &Ui, x: f32, y: f32, d: f32) { entwicklung_pfeil_mit(ui, x, y, d, true) }
pub(super) fn entwicklung_pfeil_mit(ui: &Ui, x: f32, y: f32, d: f32, mit_zahl: bool) {
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
pub(super) fn zuschlag_text(ui: &Ui, x: f32, y: f32, p: &Player) {
    let z = crate::world::form_zuschlag(p);
    if z != 0.0 {
        let t = format!("{z:+.0}").replace('-', "−");
        ui.text(&t, x, y, 14.0, if z > 0.0 { Theme::gut() } else { Theme::problem() });
        // the reasons on hover (Fabian 24.11.: every change with its reason)
        if Rect::new(x - 4.0, y - 16.0, ui.measure(&t, 14.0) + 8.0, 20.0).contains(ui.mouse) { *ui.tooltip.borrow_mut() = Some(crate::world::zuschlag_gruende(p).join("\n")); }
    }
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
    let cl = c.world.club(club);
    (hex(crate::ui::wappen_farbe_von(cl.id).unwrap_or(cl.c1)), hex(cl.c2))
}

pub(super) fn player_row_head(ui: &Ui, x: f32, y: f32, cols: &[(&str, f32)]) {
    for (t, cx) in cols { ui.text_semi(t, x + cx, y, 14.0, Theme::gedaempft()); }
}

/// A sort key: numbers sort high to low, text A to Z, on the first click.
#[derive(PartialEq, PartialOrd)]
pub(super) enum Sk { N(f32), S(String) }

/// Header of a sortable list: a click on a column sorts by it, the next click flips the order.
pub(super) fn sort_head(ui: &mut Ui, v: &mut View, liste: u8, x: f32, y: f32, cols: &[(&str, f32)]) {
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
pub(super) fn sortieren(v: &View, liste: u8, sq: &mut Vec<usize>, key: impl Fn(usize, usize) -> Sk) {
    let Some(&(col, flip)) = v.sortierung.get(&liste) else { return };
    let mut k: Vec<(Sk, usize)> = sq.iter().map(|&p| (key(p, col), p)).collect();
    k.sort_by(|a, b| {
        let o = a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal);
        let o = if matches!(a.0, Sk::N(_)) { o.reverse() } else { o };
        if flip { o.reverse() } else { o }
    });
    *sq = k.into_iter().map(|x| x.1).collect();
}

pub(super) fn strength_cell(ui: &Ui, c: &Career, p: usize, cx: f32, cy: f32) {
    let (lo, hi) = c.ovr_range(p);
    if hi - lo <= 1.0 { ui.pill(cx, cy, (lo + hi) / 2.0, 38.0); } else {
        let col = Theme::skala((lo + hi) / 2.0);
        rrect(cx - 30.0, cy - 11.0, 60.0, 22.0, 11.0, alpha(col, 0.25));
        rrect_line(cx - 30.0, cy - 11.0, 60.0, 22.0, 11.0, col);
        ui.text_center(&format!("{}–{}", lo, hi), cx, cy + 6.0, 15.0, Theme::text());
    }
}

// ================================================================ Übersicht
pub(super) fn form_squares(ui: &Ui, x: f32, y: f32, form: &[char]) {
    for (j, ch) in form.iter().enumerate() {
        let col = match ch { 'S' => Theme::gut(), 'N' => Theme::problem(), _ => Theme::gedaempft() };
        rrect(x + j as f32 * 26.0, y, 22.0, 22.0, 5.0, col);
        // the letters are kept German in the save (S/U/N); the screen shows them in the interface language
        let zeichen = crate::inhalt::t(&format!("ui.form.{ch}"));
        ui.text_center(if zeichen.is_empty() { "?" } else { zeichen }, x + j as f32 * 26.0 + 11.0, y + 16.0, 14.0, hex(0xffffff)); // roh
    }
}

/// A speech-bubble card with an avatar (guide §3.5).
pub(super) fn bubble(ui: &mut Ui, r: Rect, who: &str, role: &str, text: &str) {
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
/// The local hour (0–23), asked once every ten minutes (the clock of the PC, for the greeting).
pub(super) fn ortsstunde() -> Option<u32> {
    static S: std::sync::Mutex<Option<(f64, Option<u32>)>> = std::sync::Mutex::new(None);
    let jetzt = get_time();
    let mut g = S.lock().unwrap();
    if let Some((t, h)) = *g { if jetzt - t < 600.0 { return h; } }
    let u = crate::fl27::uhrzeit();
    let h = u.split(':').next().and_then(|x| x.trim().parse::<u32>().ok()).map(|h| if u.to_uppercase().contains("PM") && h < 12 { h + 12 } else if u.to_uppercase().contains("AM") && h == 12 { 0 } else { h }); // roh
    *g = Some((jetzt, h));
    h
}
// ================================================================ Kader (Porträt-Raster) and Kabine
pub(super) fn mood_words(m: f32) -> &'static str {
    if m >= 70.0 { crate::inhalt::t("ui.pages.bester_laune") } else if m >= 55.0 { crate::inhalt::t("ui.wort.zufrieden") } else if m >= 40.0 { crate::inhalt::t("ui.wort.durchwachsen") } else { crate::inhalt::t("ui.wort.unzufrieden") }
}

// ================================================================ Aufstellung
/// The mean grade of a player's last five matches, if he played any.
pub fn form5_pub(pl: &crate::world::Player) -> Option<f32> { form5(pl) }
pub(super) fn form5(pl: &crate::world::Player) -> Option<f32> {
    let l: Vec<f32> = pl.last.iter().rev().take(5).copied().collect();
    (!l.is_empty()).then(|| l.iter().sum::<f32>() / l.len() as f32)
}

/// The colour of a scout's verdict (Konzept 4.3): sign now … do not sign.
pub fn urteil_farbe(u: usize) -> Color { [hex(0x1E9E5A), Theme::gut(), hex(0x8BC34A), Theme::achtung(), Theme::akzent(), Theme::problem()][u.min(5)] }

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
            if ui.tippen(&mut v.auswahl_suche, 40) { v.auswahl_scroll = 0; }
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
        if ui.hit(a.r) { v.auswahl = if v.auswahl == Some(k) { None } else { Some(k) }; v.auswahl_scroll = felder[k].sel.saturating_sub(5); v.auswahl_suche.clear(); }
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

/// The player a message is from or about (talks, contracts, injuries, offers), for its photo.
pub(super) fn msg_spieler(c: &Career, m: &Msg) -> Option<usize> {
    let p = match &m.action {
        Action::TransferOffer { pid, .. } | Action::Complaint { pid } | Action::Expiring { pid } | Action::Report { pid } | Action::Graduate { pid } => Some(*pid),
        Action::Wahl { bezug, .. } => bezug.first().copied(),
        _ => c.squad().into_iter().find(|&p| { let n = &c.world.players[p].name; !n.is_empty() && (m.title.contains(n.as_str()) || &m.from == n) }),
    };
    p.filter(|&p| p < c.world.players.len())
}
/// The look of a competition (Fabian 22.11.: like the real leagues): colours and pattern from wettbewerbslook.toml by key,
/// by kind for Europe (1 CL, 2 EL, 3 Conference), else the country's flag colours.
pub struct Look { pub c1: Color, pub c2: Color, pub akzent: Color, pub akzent_lesbar: Color, pub text: Color, pub text2: Color, pub muster: String }
/// The look of the competition a fixture belongs to (league, cup or European competition).
pub fn wo_look(c: &Career, wo: Wo) -> Look {
    match wo {
        Wo::Liga(l) => { let lg = &c.world.leagues[l]; wettbewerb_look(c, &lg.key, 9, &lg.land) }
        Wo::Pokal(k) => {
            let cu = &c.world.cups[k];
            let land = if cu.country.is_empty() { c.world.leagues.iter().find(|x| x.key == cu.key).map(|x| x.land.clone()).unwrap_or_default() } else { cu.country.clone() };
            wettbewerb_look(c, &cu.key, cu.kind, &land)
        }
    }
}
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
    let pokal = (kind == 0).then(|| crate::regeln::wert_oder("wettbewerbslook", "pokal").and_then(|l| l.get(land)).and_then(aus)).flatten();
    let eintrag = pokal.or_else(|| art.and_then(|a| crate::regeln::wert_oder("wettbewerbslook", &format!("art.{a}")).and_then(aus)))
        .or_else(|| crate::regeln::wert_oder("wettbewerbslook", "liga").and_then(|l| l.get(key)).and_then(aus))
        .or_else(|| crate::regeln::wert_oder("wettbewerbslook", "land").and_then(|l| l.get(land)).and_then(|v| { let a = v.as_array()?; let (x, y) = (farbe(&a[0])?, farbe(&a[1])?); Some((x, y, y, crate::id("keins").to_string())) }));
    let (c1, c2, ak, muster) = eintrag.unwrap_or((0x1e293b, 0x334155, 0x94a3b8, crate::id("keins").to_string()));
    let (c1, c2, ak) = (hex(c1), hex(c2), hex(ak));
    let text = crate::ui::on(c1);
    Look { c1, c2, akzent: ak, akzent_lesbar: crate::ui::lesbar(ak, Theme::flaeche(), 3.0), text, text2: alpha(text, 0.8), muster }
}
/// The colours of the "per competition" theme (Fabian 24.11.): (head band, accent). A dark neutral first colour (the
/// Europa League's black) gives way to a colourful accent; a pale accent (the Bundesliga's white) to the first colour.
pub fn thema_farben(lk: &Look) -> (u32, u32) {
    // colourful: clearly saturated and not near-black (the Premier League's deep purple counts, the Europa League's black not)
    let bunt = |c: Color| { let (mx, mn) = (c.r.max(c.g).max(c.b), c.r.min(c.g).min(c.b)); mx > 0.12 && (mx - mn) / mx > 0.35 };
    let u = |c: Color| { let k = |x: f32| ((x * 255.0).round() as u32).min(255); (k(c.r) << 16) | (k(c.g) << 8) | k(c.b) };
    let kopf = if bunt(lk.c1) || !bunt(lk.akzent) { lk.c1 } else { lk.akzent };
    // one colour for head and accent: the competition's own (Fabian 24.11.: Premier League purple, not its neon green)
    (u(kopf), u(kopf))
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

/// Day-step durations in ms for the diagnosis page (Paket 6).
pub static TAGESZEIT: std::sync::Mutex<Vec<f32>> = std::sync::Mutex::new(Vec::new());

pub(super) fn zeilen(ui: &mut Ui, r: Rect, items: &[(String, String)], start: f32) {
    for (k, (a, b)) in items.iter().enumerate() {
        let yy = r.y + start + k as f32 * 28.0;
        if yy > r.y + r.h - 12.0 { break; }
        ui.text(&ui.fit(a, 16.0, r.w * 0.6), r.x + 14.0, yy, 16.0, Theme::text());
        ui.text_right(b, r.x + r.w - 14.0, yy, 16.0, Theme::text2());
    }
}
pub(super) fn links_rechts() -> (Rect, Rect) { (Rect::new(x0() + 4.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0), Rect::new(x0() + cbreite() * 0.5 + 10.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0)) }

pub fn stufe_wort(o: f32) -> String { (if o >= 82.0 { crate::inhalt::t("stufe.staerke.5") } else if o >= 77.0 { crate::inhalt::t("stufe.staerke.4") } else if o >= 72.0 { crate::inhalt::t("stufe.staerke.3") } else if o >= 66.0 { crate::inhalt::t("stufe.staerke.2") } else { crate::inhalt::t("ui.seiten.ausbaufaehig") }).to_string() }

/// Liga in Zahlen (Teil 3): scorers, assists, grades and, per club, clean sheets and fair play (yellow cards of the squad),
/// each card led by its leader in one sentence.
/// A number with two decimals as the language writes it (German with a comma).
pub(super) fn dez2(x: f32) -> String { let z = format!("{x:.2}"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { z.replace('.', ",") } else { z } }

/// The club colour, readable as text and lines on the current background.
pub(super) fn farbe_lesbar(f: Color) -> Color { crate::ui::lesbar(f, Theme::flaeche(), 3.0) }
