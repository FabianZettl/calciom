// PES M (Rust): a football manager on a Football Life world database.
/// An identifier in the code (compared, stored, looked up), never shown and never translated.
pub const fn id(s: &'static str) -> &'static str { s }
mod update;
mod netzbilder;
mod sofifa;
mod kalibrieren;
mod zufall_ereignis;
mod gutgespraech;
mod wiederholung;
mod cups;
mod farm;
mod inhalt;
mod ligaregeln;
mod pokalregeln;
mod entwicklung;
mod cpk;
mod live;
mod livespiel;
mod pool;
mod vereinsfuehrung;
mod taktik9;
mod transferki;
mod spielgefuehl;
mod klauseln;
mod ehrungen;
mod preise;
mod privat;
mod meldung;
mod vorbereitung;
mod trainerbank;
mod realdaten;
mod meilensteine;
mod kimarkt;
mod berater;
mod altimport;
mod taktikprobe;
mod nationen;
mod geo;
mod date;
mod fl27;
mod game;
mod mensch;
mod regeln;
mod rest3;
mod taktik;
mod akademie;
mod ki;
mod storys;
mod personal;
mod vorgang;
mod medien;
mod wirtschaft;
mod stadionbetrieb;
mod wirtschaftsvorgaenge;
mod medizin;
mod haltung;
mod suchprofil;
mod termine;
mod praemien;
mod sondierung;
mod jobverhandlung;
mod prematch;
mod vertragsanfrage;
mod geschichten;
mod kredit;
mod mitglieder;
mod musik;
mod krise;
mod trainingsplan;
mod jugend;
mod gesichter;
mod scoutbericht;
mod statistik;
mod auftraege;
mod nationalteams;
mod livepatch;
mod praesidium;
mod handy;
mod taktikwirkung;
mod neubau;
mod prognose;
mod trainerprofil;
mod vereinsbudget;
mod scouting;
mod transfer;
mod verhandlung;
mod pages;
mod pesdb;
mod ui;
mod world;

use game::Career;
use macroquad::prelude::*;
use pages::*;
use ui::*;
use world::World;

const WORLD_JSON: &str = include_str!("../assets/world.json");

fn ui_gradient(r: Rect, c: Color) {
    let left = if Theme::dark() { c } else { Theme::verein_weich() };
    gradient_h(r.x, r.y, r.w, r.h, left, alpha(left, 0.0));
}

/// Guide §4 Bereichsfarben for the card heads of each page.
fn area_color(p: Page) -> u32 {
    match p {
        Page::Training => 0x3ecf8e, Page::Finanzen => 0xf5b942, Page::Presse | Page::Posteingang => 0x38bdf8, Page::Akademie => 0x2dd4bf,
        Page::Medizin => 0xf472b6, Page::Transfermarkt | Page::Verhandlungen | Page::Vertraege | Page::Scouting => 0xfb923c,
        Page::Kabine => 0xa78bfa, Page::Praesidium | Page::Trainer | Page::Optionen => 0x94a3b8, _ => 0x38bdf8,
    }
}

/// Stage banner: Überzeile, title and the one question the page answers (guide §1.3, §7).
fn stage_text(v: &View) -> (&'static str, &'static str, &'static str) {
    match v.page {
        Page::Uebersicht => (crate::inhalt::t("gm.in.kicker"), crate::inhalt::t("reiter.uebersicht"), crate::inhalt::t("ui.main.was_steht_heute_an")),
        Page::Posteingang => (crate::inhalt::t("gm.in.kicker"), crate::inhalt::t("zentrale.k.posteingang"), crate::inhalt::t("ui.main.dein_assistent_hat_die_post")),
        Page::Kalender => (crate::inhalt::t("gm.in.kicker"), crate::inhalt::t("gl.kopf.ergebnisse.alle"), crate::inhalt::t("ui.main.was_steht_wann_an")),
        Page::Kader => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("zentrale.k.kader"), crate::inhalt::t("ui.main.wer_spielt_wer_wartet_wer")),
        Page::Aufstellung => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("aktion.aufstellung"), crate::inhalt::t("ui.main.wer_laeuft_auf_und_wie")),
        Page::Kabine => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("vg.pg.dim.kabine"), crate::inhalt::t("kabine.neu.unter")),
        Page::Training => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("status.training"), crate::inhalt::t("ui.main.woran_arbeiten_wir_diese_woche")),
        Page::Medizin => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("kachel.medizin"), crate::inhalt::t("ui.main.wer_fehlt_wer_ist_angeschlagen")),
        Page::Praesidium => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("ui.main.praesidium"), crate::inhalt::t("ui.main.was_erwartet_der_vorstand")),
        Page::Finanzen => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("bereich.finanzen"), crate::inhalt::t("ui.main.wie_steht_der_verein_da")),
        Page::Trainer => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("aktion.trainerprofil"), crate::inhalt::t("ui.main.wie_laeuft_deine_laufbahn")),
        Page::Transfermarkt => (crate::inhalt::t("nav.gruppe.markt"), crate::inhalt::t("bereich.transfers"), crate::inhalt::t("ui.main.wen_suchen_wir")),
        Page::Verhandlungen => (crate::inhalt::t("nav.gruppe.markt"), crate::inhalt::t("ux.abschnitt.verhandlungen"), crate::inhalt::t("ui.main.bekommen_wir_ihn")),
        Page::Vertraege => (crate::inhalt::t("nav.gruppe.markt"), crate::inhalt::t("as.reiter.vertraege"), crate::inhalt::t("ui.main.wen_halten_wir_wen_lassen")),
        Page::Scouting => (crate::inhalt::t("nav.gruppe.markt"), crate::inhalt::t("profil.teil.8"), crate::inhalt::t("ui.main.wen_suchen_wir")),
        Page::Akademie => (crate::inhalt::t("profil.jugend"), crate::inhalt::t("aka.gruppe.akademie"), crate::inhalt::t("ui.main.wer_ist_das_naechste_talent")),
        Page::Tabellen => (crate::inhalt::t("ticker.kat.welt"), crate::inhalt::t("zentrale.wettbewerbe"), crate::inhalt::t("ui.main.was_passiert_in_europa")),
        Page::Presse => ("Medien", crate::inhalt::t("feed.art.journalist"), crate::inhalt::t("ui.main.was_schreibt_die_presse")), // roh
        Page::Spieltag => if v.mv.as_ref().map(|m| m.phase == 2).unwrap_or(false) { (crate::inhalt::t("bereich.spieltag"), crate::inhalt::t("bericht.titel"), crate::inhalt::t("ui.main.so_lief_das_spiel")) } else { (crate::inhalt::t("bereich.spieltag"), crate::inhalt::t("gl.anpfiff"), crate::inhalt::t("ui.main.mach_deine_elf_bereit")) },
        Page::Optionen => (crate::inhalt::t("menu.bereich.system"), crate::inhalt::t("bereich.system"), crate::inhalt::t("ui.main.wie_soll_pes_m_aussehen")),
        Page::Ziele => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("aktion.ziele"), crate::inhalt::t("ui.main.was_will_der_verein_erreichen")),
        Page::Manager => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("gm.mg.hero"), crate::inhalt::t("ui.main.wie_geht_es_dir")),
        Page::Stadion => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("menu.stadion_und_anlagen.titel"), crate::inhalt::t("ui.main.wo_bauen_wir_aus")),
        Page::Personalmarkt => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("ui.main.personalmarkt"), crate::inhalt::t("ui.main.bewerber_und_freie_leute_trainer")),
        Page::Geschichte => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("menu.vereinsgeschichte.titel"), crate::inhalt::t("ui.main.was_haben_wir_erreicht")),
        Page::Tagebuch => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("gm.tb.hero"), crate::inhalt::t("ui.main.was_ist_warum_passiert")),
        Page::Kadermeldung => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("ui.meldung.seite"), crate::inhalt::t("ui.meldung.unter")),
        Page::Uefa => (crate::inhalt::t("ticker.kat.welt"), crate::inhalt::t("ui.uefa.seite"), crate::inhalt::t("ui.uefa.unter")),
        Page::Privat => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("ui.privat.seite"), crate::inhalt::t("ui.privat.unter")),
        Page::Klima => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("ui.main.vereinsklima"), crate::inhalt::t("ui.main.wie_ist_die_stimmung_rund")),
        Page::Marken => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("ui.main.marken_und_sponsoren"), crate::inhalt::t("ui.main.wer_zahlt_fuer_unseren_namen")),
        Page::Kaderplanung => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("aktion.kaderplanung"), crate::inhalt::t("ui.main.was_braucht_der_kader")),
        Page::Vergleich => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("menu.spielervergleich.titel"), crate::inhalt::t("ui.main.wer_ist_besser")),
        Page::Entwicklung => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("profil.entwicklung_verlauf"), crate::inhalt::t("ui.main.wer_waechst_wer_stockt")),
        Page::Vorbereitung => (crate::inhalt::t("bereich.spieltag"), crate::inhalt::t("gl.vorb.kicker"), crate::inhalt::t("ui.main.wie_gehen_wir_das_naechste")),
        Page::Analyse => (crate::inhalt::t("bereich.spieltag"), crate::inhalt::t("menu.analyse.titel"), crate::inhalt::t("ui.main.was_lief_im_letzten_spiel")),
        Page::Zeitung => ("Medien", crate::inhalt::t("md.pool.typ.zeitung"), crate::inhalt::t("ui.main.was_steht_in_der_zeitung")), // roh
        Page::Fans => ("Medien", crate::inhalt::t("menu.fans_und_stimmung.titel"), crate::inhalt::t("ui.main.was_denken_die_fans")), // roh
        Page::Ranglisten => (crate::inhalt::t("ticker.kat.welt"), crate::inhalt::t("ux.abschnitt.rankings"), crate::inhalt::t("ui.main.wer_ist_der_beste")),
        Page::Awards => (crate::inhalt::t("ui.award.seite"), crate::inhalt::t("ui.preis.ballon_seite"), crate::inhalt::t("ui.award.unter")),
        Page::TheBest => (crate::inhalt::t("ui.award.seite"), crate::inhalt::t("ui.preis.best_seite"), crate::inhalt::t("ui.preis.best_unter")),
        Page::LigaPreise => (crate::inhalt::t("ticker.kat.welt"), crate::inhalt::t("ui.preis.liga_seite"), crate::inhalt::t("ui.preis.liga_unter")),
        Page::Nationalteams => (crate::inhalt::t("ticker.kat.welt"), crate::inhalt::t("ui.nt.seite"), crate::inhalt::t("ui.nt.seite_unter")),
        Page::Vereine => (crate::inhalt::t("ticker.kat.welt"), crate::inhalt::t("menu.vereine_und_spieler.titel"), crate::inhalt::t("ui.main.wer_spielt_wo")),
        Page::Vereinswelt => (crate::inhalt::t("ticker.kat.welt"), crate::inhalt::t("ui.main.vereinswelt"), crate::inhalt::t("ui.main.was_passiert_in_der_welt")),
        Page::Portal => (crate::inhalt::t("ticker.kat.welt"), crate::inhalt::t("md.pool.typ.portal"), crate::inhalt::t("ui.main.die_fl_welt_im_ueberblick")),
        Page::Handbuch => (crate::inhalt::t("menu.bereich.system"), crate::inhalt::t("ui.main.handbuch"), crate::inhalt::t("menu.handbuch.titel")),
        Page::Regeln => (crate::inhalt::t("menu.bereich.system"), crate::inhalt::t("menu.regeln_und_anpassung.titel"), crate::inhalt::t("ui.main.welche_zahlen_gelten")),
        Page::Spielerprofil => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("reiter.profil"), crate::inhalt::t("ui.main.alles_ueber_einen_spieler")),
        Page::Zentrale => (crate::inhalt::t("gm.in.kicker"), crate::inhalt::t("bereich.zentrale"), crate::inhalt::t("ui.main.alle_bereiche_auf_einen_blick")),
        Page::Kooperationen => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("ui.farm.kooperationen"), crate::inhalt::t("ui.main.zweite_mannschaft_farmteams_und_partners")),
        Page::ZweiteMannschaft => (crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("ui.game.zweite_mannschaft"), crate::inhalt::t("ui.main.wer_spielt_oben_wer_unten")),
        Page::Jugendnetz => (crate::inhalt::t("profil.jugend"), crate::inhalt::t("ui.main.jugendnetz"), crate::inhalt::t("ui.main.wo_wachsen_die_talente_von")),
        Page::Merkliste => (crate::inhalt::t("nav.gruppe.markt"), crate::inhalt::t("aka.pille.merkliste"), crate::inhalt::t("ui.main.wen_hast_du_im_blick")),
        Page::Trainerteam => (crate::inhalt::t("bereich.verein"), crate::inhalt::t("ui.main.trainerteam"), crate::inhalt::t("ui.main.wer_arbeitet_mit_dir")),
        Page::Profi => (crate::inhalt::t("menu.bereich.system"), crate::id("Profi-Details"), crate::inhalt::t("ui.main.wie_laeuft_pes_m")),
    }
}

fn conf() -> Conf {
    // preview aid: PESM_FENSTER=1920x1080 starts with that window size (checks of other screen shapes)
    let (w, h) = std::env::var(crate::id("PESM_FENSTER")).ok().and_then(|s| { let (a, b) = s.split_once('x')?; Some((a.parse().ok()?, b.parse().ok()?)) }).unwrap_or((crate::ui::W_BASIS as i32, 900));
    Conf { window_title: "PES M".into(), window_width: w, window_height: h, high_dpi: false, sample_count: 4, ..Default::default() } // roh
}

#[derive(PartialEq)]
enum Screen { Start, Game }
/// The folder dialog of the start screen, running in the background.
static CPK_LESEN: std::sync::Mutex<Option<std::sync::mpsc::Receiver<Result<std::path::PathBuf, String>>>> = std::sync::Mutex::new(None);
static ORDNER_DIALOG: std::sync::Mutex<Option<std::sync::mpsc::Receiver<Option<std::path::PathBuf>>>> = std::sync::Mutex::new(None);

/// The club card of the career start: crest, name, league and country, strength per part of the team, stadium and
/// the real history (realdaten), and the map of its league with the club marked. Returns a club clicked on the map.
fn verein_info(ui: &mut Ui, r: Rect, w: &world::World, t: usize, zoom: &mut f32, off: &mut (f32, f32)) -> Option<usize> {
    let cl = &w.clubs[t];
    let lg = &w.leagues[cl.league];
    ui.card(r);
    ui.club_crest(r.x + 44.0, r.y + 44.0, 28.0, cl.id, hex(cl.c1), &cl.short);
    let nm = ui.fit(&cl.name, 24.0, r.w - 100.0);
    ui.title(&nm, r.x + 84.0, r.y + 40.0, 24.0, Theme::text());
    let unter = ui.fit(&format!("{} · {}", lg.name, lg.land), 15.0, r.w - 100.0);
    ui.text(&unter, r.x + 84.0, r.y + 62.0, 15.0, Theme::text2());
    ui.text(&stufe_wort(cl.level), r.x + 84.0, r.y + 82.0, 14.0, Theme::akzent());
    // strength per part of the team: the mean of the best players of each part (1 keeper, 4, 4, 2)
    let teile: [(&str, &[usize], usize); 4] = [(crate::id("TW"), &[0], 1), (crate::id("ABW"), &[1, 2], 4), (crate::id("MF"), &[3, 4], 4), (crate::id("ST"), &[5], 2)];
    let mut y = r.y + 108.0;
    for (name, gr, n) in teile {
        let mut o: Vec<f32> = cl.squad.iter().filter(|&&p| gr.contains(&crate::kimarkt::gruppe(&w.players[p].pos))).map(|&p| w.players[p].ovr).collect();
        o.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let m = if o.is_empty() { 0.0 } else { o.iter().take(n).sum::<f32>() / o.len().min(n) as f32 };
        ui.text_semi(name, r.x + 16.0, y + 5.0, 13.0, Theme::text2());
        ui.bar(r.x + 60.0, y - 4.0, r.w - 120.0, 10.0, (m - 50.0) / 45.0, Theme::akzent());
        ui.text_right(&format!("{m:.0}"), r.x + r.w - 16.0, y + 5.0, 14.0, Theme::text());
        y += 22.0;
    }
    // the real history (only when known)
    if let Some(rd) = crate::realdaten::verein(&cl.name) {
        let mut kopf = vec![];
        if let Some(g) = rd.gruendung { kopf.push(crate::tf!("ui.real.gruendung", g, rd.ort)); }
        if !rd.stadion.is_empty() { kopf.push(rd.stadion.clone()); }
        if rd.meister > 0 { kopf.push(crate::tf!("ui.real.meister", rd.meister)); }
        let mut z = vec![];
        if !kopf.is_empty() { z.push(kopf.join(" · ")); }
        if !rd.rivalen.is_empty() { z.push(crate::tf!("ui.real.rivalen", rd.rivalen.iter().take(3).map(|(c, _)| c.clone()).collect::<Vec<_>>().join(", "))); }
        if !rd.legenden.is_empty() { z.push(crate::tf!("ui.real.legenden", rd.legenden.iter().take(4).map(|l| l.0.clone()).collect::<Vec<_>>().join(", "))); }
        y += 4.0;
        for x in z { y = ui.paragraph(&x, r.x + 16.0, y, 13.0, r.w - 32.0, Theme::text2()) + 2.0; }
    }
    // the map of the league
    let kr = Rect::new(r.x + 8.0, y + 6.0, r.w - 16.0, (r.y + r.h - 8.0 - y - 6.0).clamp(120.0, r.w * 0.9));
    let k = crate::geo::karte(ui, kr, w, &lg.teams, Some(t), zoom, off);
    // the stadium photo in the room left below the map
    let unten = r.y + r.h - 8.0 - (kr.y + kr.h + 8.0);
    if unten > 100.0 { ui.stadion_bild(Rect::new(r.x + 8.0, kr.y + kr.h + 8.0, r.w - 16.0, unten.min(r.w * 0.5)), cl.id, &cl.name); }
    k
}

struct StartState {
    /// Zoom and offset of the map in the club step.
    karte_zoom: f32,
    karte_off: (f32, f32),
    step: usize,
    db_path: String,
    db_focus: bool,
    league: usize,
    team: Option<usize>,
    coach: String,
    coach_focus: bool,
    saves: Vec<std::path::PathBuf>,
    /// Career to delete after the question, the save whose backups are open.
    loeschen: Option<std::path::PathBuf>,
    offen: Option<std::path::PathBuf>,
    error: String,
    scroll: usize,
    /// The chosen FL world folder (None = built-in demo world) and the result of its check.
    welt: Option<std::path::PathBuf>,
    welt_info: String,
    /// Team search of step 2: text, focus, country and league filter, the prepared index of the world, the marked hit.
    such: String,
    such_focus: bool,
    f_land: Option<String>,
    f_liga: Option<usize>,
    index: Option<world::SuchIndex>,
    treffer: usize,
    /// Second teams found in the chosen world (shown in step 1) and the unclear cases ("bitte prüfen").
    paare: Vec<String>,
    unklar: Vec<String>,
    /// The coach being created (trainerprofil.rs), a start without a club and where its offers may come from
    /// (0 whole database, 1 the own country's leagues, 2 up to three chosen leagues).
    profil: crate::trainerprofil::Trainerprofil,
    ohne: bool,
    angebot_modus: usize,
    ligen: Vec<usize>,
    ligen_scroll: usize,
    /// League list of the new career: 0 by strength, 1 grouped by country (Fabian 23.11.).
    liga_sort: u8,
}

struct SelfTest {
    path: String,
    frame: u32,
    day0: u32,
    steps: Vec<(String, bool, String)>,
    stage: u8,
}

/// `--start-test <bericht> [welt]`: clicks through the four start steps like a person (world, club via search,
/// coach, start). Its careers go to a fresh temporary folder (PESM_KARRIEREN), never the person's or the game folder.
struct StartTest { path: String, welt: Option<String>, frame: u32, stage: u8, steps: Vec<(String, bool, String)>, team: String }

struct App {
    starttest: Option<StartTest>,
    ui: Ui,
    v: View,
    screen: Screen,
    start: StartState,
    template: Option<World>,
    career: Option<Career>,
    fast: bool,
    last_inbox: usize,
    selftest: Option<SelfTest>,
    /// The phone server (handy/), when "Handy-Zugang" is on; the time of the last phone command that changed something.
    handy: Option<handy::Handy>,
    handy_geaendert: Option<f64>,
    /// Background music (Einstellungen → Musik), paused during a match.
    musik: musik::Musik,
    /// The song or stream title that just changed and when (popup bottom right for a few seconds).
    musik_titel: Option<(String, f32)>,
    /// A new career while the world is backed up in the background (loading bar instead of a frozen window).
    start_lauf: Option<(Career, std::sync::mpsc::Receiver<Result<(), String>>)>,
}

impl App {
    async fn new() -> App {
        let head = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/BarlowCondensed-ExtraBoldItalic.ttf")).unwrap();
        let body = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/BarlowSemiCondensed-Medium.ttf")).unwrap();
        let semi = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/BarlowSemiCondensed-SemiBold.ttf")).unwrap();
        let mut sfx = vec![];
        for (n, b) in [
            (crate::id("klick"), &include_bytes!("../assets/sfx/klick.wav")[..]), (crate::id("tagesabschluss"), &include_bytes!("../assets/sfx/tagesabschluss.wav")[..]),
            (crate::id("angebot"), &include_bytes!("../assets/sfx/angebot.wav")[..]), (crate::id("posteingang"), &include_bytes!("../assets/sfx/posteingang.wav")[..]),
            ("wechsel_vollzogen", &include_bytes!("../assets/sfx/wechsel_vollzogen.wav")[..]), (crate::id("fehler"), &include_bytes!("../assets/sfx/fehler.wav")[..]),
            (crate::id("bestaetigt"), &include_bytes!("../assets/sfx/bestaetigt.wav")[..]), (crate::id("titel"), &include_bytes!("../assets/sfx/titel.wav")[..]),
            (crate::id("verletzung"), &include_bytes!("../assets/sfx/verletzung.wav")[..]),
            (crate::id("tor"), &include_bytes!("../assets/sfx/tor.wav")[..]), (crate::id("karte"), &include_bytes!("../assets/sfx/karte.wav")[..]),
            (crate::id("elfmeter_vorbei"), &include_bytes!("../assets/sfx/elfmeter_vorbei.wav")[..]),
            (crate::id("anpfiff"), &include_bytes!("../assets/sfx/anpfiff.wav")[..]), (crate::id("abpfiff"), &include_bytes!("../assets/sfx/abpfiff.wav")[..]),
            (crate::id("sieg"), &include_bytes!("../assets/sfx/sieg.wav")[..]), (crate::id("niederlage"), &include_bytes!("../assets/sfx/niederlage.wav")[..]),
            (crate::id("meilenstein"), &include_bytes!("../assets/sfx/meilenstein.wav")[..]), (crate::id("pressefrage"), &include_bytes!("../assets/sfx/pressefrage.wav")[..]),
        ] {
            sfx.push((n.to_string(), macroquad::audio::load_sound_from_bytes(b).await.ok()));
        }
        let ui = Ui { head, body, semi, mouse: Vec2::ZERO, clicked: false, released_consumed: false, wheel: 0.0, time: 0.0, typed: vec![], key_back: false, key_enter: false, sfx, sound_queue: vec![], hover_any: false, tooltip: Default::default(), rahmen: Default::default(), foto_orte: Default::default(), wappen_orte: Default::default(), inject_click: None, crests: Default::default(), lautstaerke: pages::klang_lesen(), wappen_farben: Default::default(), bilder: Default::default(), bilder_fuer: None, logos: Default::default(), fotos: Default::default(), stadien: Default::default(), foto_budget: std::cell::Cell::new(6), netz: Default::default(), nachweise: Default::default() };
        let default_db = fl27::welt_ordner(&fl27::sider()).display().to_string();
        App {
            ui, v: View::new(), screen: Screen::Start,
            start: StartState { step: 0, db_path: default_db, db_focus: false, league: 0, team: None, coach: String::new(), coach_focus: false, saves: game::list_saves(), loeschen: None, offen: None, error: String::new(), scroll: 0, welt: None, welt_info: String::new(), such: String::new(), such_focus: false, f_land: None, f_liga: None, index: None, treffer: 0, paare: vec![], unklar: vec![], karte_zoom: 1.0, karte_off: (0.0, 0.0), profil: Default::default(), ohne: false, angebot_modus: 0, ligen: vec![], ligen_scroll: 0, liga_sort: 0 },
            template: None, career: None, fast: false, last_inbox: 0, selftest: None, starttest: None, handy: None, handy_geaendert: None, musik: Default::default(), musik_titel: std::env::var(crate::id("PESM_MUSIK_TITEL")).ok().map(|t| (t, 1e6)), start_lauf: None,
        }
    }

    fn builtin_world() -> World { World::load(WORLD_JSON, 7) }

    fn new_demo_career(&mut self) {
        let w = Self::builtin_world();
        let own = w.clubs.iter().position(|c| c.name.contains("Bayern")).unwrap_or(0); // roh
        let c = Career::new(w, own, crate::inhalt::t("ui.main.fabian_trainer"));
        self.last_inbox = c.inbox.len();
        self.career = Some(c);
        self.screen = Screen::Game;
    }

    // ------------------------------------------------------------ frame
    fn frame(&mut self) {
        Theme::set(self.v.dark, 0x38bdf8, 0x38bdf8);
        Theme::set_akzent(0x38bdf8);
        clear_background(Theme::grund());
        self.ui.begin();
        // the career's world (the built-in demo world uses the FL world folder of the game for its pictures)
        let welt = match (&self.screen, &self.career) { (Screen::Game, Some(c)) => Some(c.welt()).filter(|w| w.is_dir()), _ => self.start.welt.clone() };
        self.ui.bilder_fuer_welt(welt);
        self.handy_bearbeiten();
        match self.screen {
            Screen::Start => { self.start_screen(); self.ui.nachweise_zeichnen(); }
            Screen::Game => self.game_screen(),
        }
        self.ui.flush_sounds();
        // a match running: FL27's reader sees a live score, or PES M's own match plays live
        let spiel = (self.v.fl27.is_some() && self.v.fl27_live.is_some()) || self.v.mv.as_ref().map(|m| m.phase == 1).unwrap_or(false);
        if let Some(t) = self.musik.takt(&musik::url(), spiel, get_frame_time()) { self.musik_titel = Some((t, get_time() as f32)); }
        self.musik_popup();
    }

    /// The new song or stream title, bottom right: slides in, stays five seconds, fades out.
    fn musik_popup(&mut self) {
        let Some((t, t0)) = self.musik_titel.clone() else { return };
        let age = if t0 >= 1e6 { 1.0 } else { get_time() as f32 - t0 }; // preview (PESM_MUSIK_TITEL): stays on screen
        if age > 6.0 { self.musik_titel = None; return; }
        let ui = &mut self.ui;
        let text = ui.fit(&t, 16.0, 380.0);
        let w = ui.measure_semi(&text, 16.0).max(ui.measure(crate::inhalt::t("ui.musik.jetzt"), 12.0)) + 56.0 + 36.0;
        let a = (age * 4.0).min(1.0).min((6.0 - age).max(0.0));
        let (x, y) = (crate::ui::breite() - w - 56.0 + (1.0 - (age * 5.0).min(1.0)) * 40.0, crate::ui::hoehe() - 84.0);
        let mut f = Theme::flaeche(); f.a = a;
        let mut l = Theme::akzent(); l.a = a;
        let mut t1 = Theme::text(); t1.a = a;
        let mut t2 = Theme::text2(); t2.a = a;
        rrect(x + 3.0, y + 4.0, w, 58.0, 8.0, Color::new(0.0, 0.0, 0.0, 0.25 * a)); // shadow, so it stands off the page
        rrect(x, y, w, 58.0, 8.0, f);
        draw_rectangle_lines(x, y, w, 58.0, 1.5, l);
        // a drawn note (the fonts have no ♪)
        draw_circle(x + 20.0, y + 38.0, 5.0, l);
        draw_rectangle(x + 23.5, y + 16.0, 2.0, 22.0, l);
        draw_rectangle(x + 23.5, y + 16.0, 7.0, 3.0, l);
        ui.text(crate::inhalt::t("ui.musik.jetzt"), x + 40.0, y + 22.0, 12.0, t2);
        ui.text_semi(&text, x + 40.0, y + 44.0, 16.0, t1);
        // skip to the next song (Fabian 23.11.)
        let b = Rect::new(x + w - 2.0, y + 13.0, 32.0, 32.0);
        rrect(b.x, b.y, b.w, b.h, 16.0, if ui.hover(b) { l } else { f });
        draw_triangle(vec2(b.x + 9.0, b.y + 9.0), vec2(b.x + 9.0, b.y + 23.0), vec2(b.x + 19.0, b.y + 16.0), t1);
        draw_rectangle(b.x + 20.0, b.y + 9.0, 3.0, 14.0, t1);
        if ui.hit(b) { musik::weiter(); }
    }

    /// The phone's requests of this frame: pages and commands through the same functions as the PC (handy/ansicht.rs);
    /// "Tag weiter" and "Bis zum nächsten Ereignis" go through advance() and the fast run like the PC button.
    fn handy_bearbeiten(&mut self) {
        // switched on in the settings (file handy.txt), started once; a taken port leaves it off with a note
        let an = handy::an();
        if an && self.handy.is_none() && !handy::start_fehlgeschlagen() { match handy::starten() { Ok(h) => self.handy = Some(h), Err(e) => { handy::merke_fehler(); self.v.toast(&crate::tf!("ui.handy.start_fehler", e), false); } } }
        if !an { self.handy = None; handy::info_setzen(None); return; }
        let Some(h) = self.handy.as_ref() else { return };
        handy::info_setzen(Some((h.adresse.clone(), h.code(), h.verbunden())));
        let mut auftraege = vec![];
        while let Ok(b) = h.rx.try_recv() { auftraege.push(b); }
        for b in auftraege {
            let stand = self.handy.as_ref().map(|h| h.stand.load(std::sync::atomic::Ordering::Relaxed)).unwrap_or(0);
            let antwort = match (&b.auftrag, self.career.is_some() && matches!(self.screen, Screen::Game)) {
                // crests and photos from the loaders PES M already has; DDS turned into PNG for the browser
                (handy::Auftrag::Bild(art, id), _) => {
                    let q = match art.as_str() { "verein" => self.ui.bilder.verein.get(id), _ => self.ui.bilder.spieler.get(id) }; // roh
                    handy::Antwort::Png(q.and_then(|q| q.lesen()).map(|b| if b.starts_with(b"\x89PNG") { b } else { image::load_from_memory(&b).ok().map(|i| { let mut out = std::io::Cursor::new(vec![]); let _ = i.write_to(&mut out, image::ImageFormat::Png); out.into_inner() }).unwrap_or_default() }).unwrap_or_default())
                }
                (_, false) => handy::Antwort::Json(serde_json::json!({"fehler": crate::inhalt::t("ui.handy.keine_karriere")})), // roh
                (handy::Auftrag::Seite(n), true) => handy::Antwort::Json(handy::ansicht::seite(self.career.as_mut().unwrap(), n, stand)),
                (handy::Auftrag::Befehl(v), true) => {
                    let was = v["befehl"].as_str().unwrap_or("").to_string(); // roh
                    let a = match was.as_str() {
                        "tag_weiter" => { let ok = self.advance(); serde_json::json!({"ok": ok, "meldung": ""}) } // roh
                        "bis_ereignis" => { self.fast = true; serde_json::json!({"ok": true, "meldung": crate::inhalt::t("ui.main.bis_zum_naechsten_termin")}) } // roh
                        _ => handy::ansicht::befehl(self.career.as_mut().unwrap(), v),
                    };
                    self.handy_geaendert = Some(get_time());
                    if let Some(h) = self.handy.as_ref() { h.stand.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
                    handy::Antwort::Json(a)
                }
            };
            let _ = b.antwort.send(antwort);
        }
        // the PC moved on by itself (day, match, a decision): the phone hears it through the state number
        if let (Some(h), Some(c)) = (self.handy.as_ref(), self.career.as_ref()) {
            let fp = (c.world.day as u64) << 32 | (c.inbox.len() as u64) << 8 | (c.unread() as u64 & 0xff);
            if handy::fingerabdruck_neu(fp) { h.stand.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
        }
        // saved two minutes after the last phone command (mobile-ansicht.md 2.8)
        if let (Some(t), Some(c)) = (self.handy_geaendert, self.career.as_ref()) {
            if get_time() - t > crate::regeln::f("handy", "speichern.nach_befehl_s") as f64 { let _ = c.save_mit_sicherung(); self.handy_geaendert = None; }
        }
    }

    /// The loading card of a career start: phase, files done of all, bar; at the end into the game.
    fn start_laden(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        let ui = &mut self.ui;
        let (phase, fertig, gesamt) = (fl27::FORT_PHASE.load(Relaxed), fl27::FORT_FERTIG.load(Relaxed), fl27::FORT_GESAMT.load(Relaxed));
        let r = Rect::new(screen_width() / 2.0 - 330.0, hoehe() / 2.0 - 110.0, 660.0, 220.0);
        ui.card_titled(r, crate::inhalt::t("ui.main.laden.titel"), Theme::akzent());
        ui.text_semi(crate::inhalt::t(&format!("ui.main.laden.phase.{}", phase.min(3))), r.x + 24.0, r.y + 80.0, 18.0, Theme::text());
        let anteil = if gesamt > 0 { fertig as f32 / gesamt as f32 } else { ((get_time() * 0.7).fract()) as f32 };
        ui.bar(r.x + 24.0, r.y + 104.0, r.w - 48.0, 16.0, anteil.min(1.0), Theme::akzent());
        if gesamt > 0 { ui.text(&crate::tf!("ui.main.laden.dateien", fertig.min(gesamt), gesamt), r.x + 24.0, r.y + 150.0, 15.0, Theme::text2()); }
        ui.paragraph(crate::inhalt::t("ui.main.laden.hinweis"), r.x + 24.0, r.y + 176.0, 14.0, r.w - 48.0, Theme::text2());
        let fertig_ = self.start_lauf.as_ref().and_then(|(_, rx)| match rx.try_recv() { Ok(x) => Some(x), Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Err("abgebrochen".into())), Err(_) => None }); // roh
        if let Some(r) = fertig_ {
            let (mut c, _) = self.start_lauf.take().unwrap();
            if let Err(e) = r { fl27::log(&format!("Sicherung fehlgeschlagen: {e}")); c.msg(5, "PES M", crate::inhalt::t("ui.main.sicherung_der_welt_fehlgeschlagen"), &crate::tf!("ui.main.die_karriere_laeuft_fuer_fl27", e), game::Action::None); } // roh
            let _ = c.fl27_cpk();
            let _ = c.save();
            fl27::log(&format!("Karriere gestartet: {}", c.save_name));
            self.last_inbox = c.inbox.len();
            self.career = Some(c);
            self.screen = Screen::Game;
            self.v = View::new();
            self.ui.play(crate::id("titel"));
        }
    }
    fn start_screen(&mut self) {
        if self.start_lauf.is_some() { return self.start_laden(); }
        let ui = &mut self.ui;
        // a newer release on GitHub: a banner with a button to its page
        if let Some(Some((tag, url))) = update::NEUER.get() {
            let r = Rect::new(screen_width() - 520.0, 24.0, 490.0, 52.0);
            rrect(r.x, r.y, r.w, r.h, 10.0, alpha(Theme::akzent(), 0.18));
            ui.text_semi(&crate::tf!("ui.update.neu", tag, update::VERSION), r.x + 16.0, r.y + 32.0, 16.0, Theme::text());
            if ui.button(Rect::new(r.x + r.w - 130.0, r.y + 10.0, 118.0, 32.0), crate::inhalt::t("ui.update.ansehen"), true) { update::oeffnen(url); }
        }
        let s = &mut self.start;
        // background stripes
        for i in 0..14 { let x = 600.0 + i as f32 * 70.0; draw_triangle(vec2(x, hoehe()), vec2(x + 260.0, 0.0), vec2(x + 300.0, 0.0), alpha(Theme::akzent(), 0.03)); }
        ui.title("PES M", 60.0, 110.0, 96.0, Theme::text()); // roh
        ui.text_semi(crate::inhalt::t("ui.main.pro_evolution_soccer_manager_fussball"), 64.0, 146.0, 22.0, Theme::akzent());
        // left: continue
        let lr = Rect::new(60.0, 190.0, 420.0, 650.0);
        ui.card_titled(lr, crate::inhalt::t("ui.main.karriere_fortsetzen"), Theme::akzent());
        if s.saves.is_empty() { ui.text(crate::inhalt::t("ui.main.noch_keine_karriere_gespeichert"), lr.x + 16.0, lr.y + 60.0, 18.0, Theme::text2()); }
        let mut load: Option<std::path::PathBuf> = None;
        // Karrieren verwalten (Punkt 6): club and date of each career, its backups, deleting with a question first
        let mut y = lr.y + 44.0;
        let mut neu_offen = s.offen.clone();
        let mut frage = s.loeschen.clone();
        for p in s.saves.iter().take(10) {
            if y + 56.0 > lr.y + lr.h - 8.0 { break; }
            let r = Rect::new(lr.x + 12.0, y, lr.w - 24.0, 56.0);
            let haupt = Rect::new(r.x, r.y, r.w - 120.0, r.h);
            rrect(r.x, r.y, r.w, r.h, 6.0, if ui.hover(haupt) { Theme::ueberlagerung() } else { Theme::erhoehung() });
            let n = p.file_stem().map(|x| x.to_string_lossy().to_string()).unwrap_or_default();
            ui.text_semi(&ui.fit(&n, 18.0, r.w - 140.0), r.x + 14.0, r.y + 24.0, 18.0, Theme::text());
            let info = game::save_info(p).unwrap_or_default();
            let (verein, datum) = (info["verein"].as_str().unwrap_or(""), info["datum"].as_str().unwrap_or(""));
            let unter = if verein.is_empty() { String::new() } else { crate::tf!("ui.karrieren.zeile", verein, datum) };
            ui.text(&ui.fit(&unter, 13.0, r.w - 140.0), r.x + 14.0, r.y + 44.0, 13.0, Theme::text2());
            if ui.hit(haupt) { load = Some(p.clone()); }
            let sich = game::sicherungen(p);
            if !sich.is_empty() && ui.button(Rect::new(r.x + r.w - 112.0, r.y + 14.0, 70.0, 28.0), crate::inhalt::t("ui.karrieren.sicherungen"), s.offen.as_ref() == Some(p)) {
                neu_offen = if s.offen.as_ref() == Some(p) { None } else { Some(p.clone()) };
            }
            if ui.button(Rect::new(r.x + r.w - 36.0, r.y + 14.0, 28.0, 28.0), "×", false) { frage = Some(p.clone()); }
            y += 62.0;
            if s.offen.as_ref() == Some(p) {
                for b in sich {
                    if y + 30.0 > lr.y + lr.h - 8.0 { break; }
                    let zeit = std::fs::metadata(&b).and_then(|m| m.modified()).ok().and_then(|m| m.elapsed().ok()).map(|d| d.as_secs() / 3600).unwrap_or(0);
                    let rb = Rect::new(r.x + 24.0, y, r.w - 24.0, 28.0);
                    rrect(rb.x, rb.y, rb.w, rb.h, 6.0, if ui.hover(rb) { Theme::ueberlagerung() } else { Theme::grund_tief() });
                    let wann = if zeit < 1 { crate::inhalt::t("ui.karrieren.gerade").to_string() } else if zeit < 48 { crate::tf!("ui.karrieren.stunden", zeit) } else { crate::tf!("ui.karrieren.tage", zeit / 24) };
                    ui.text(&crate::tf!("ui.karrieren.sicherung_zeile", b.file_stem().map(|x| x.to_string_lossy().to_string()).unwrap_or_default(), wann), rb.x + 10.0, rb.y + 19.0, 13.0, Theme::text2());
                    if ui.hit(rb) { load = Some(b.clone()); }
                    y += 32.0;
                }
            }
        }
        s.offen = neu_offen;
        // the question before deleting: the career goes to the bin folder, nothing is lost for good
        if let Some(p) = frage.clone() {
            let q = Rect::new(lr.x + 12.0, lr.y + lr.h - 150.0, lr.w - 24.0, 140.0);
            rrect(q.x, q.y, q.w, q.h, 10.0, Theme::problem());
            rrect(q.x + 2.0, q.y + 2.0, q.w - 4.0, q.h - 4.0, 9.0, Theme::flaeche());
            let n = p.file_stem().map(|x| x.to_string_lossy().to_string()).unwrap_or_default();
            ui.text_semi(&ui.fit(&crate::tf!("ui.karrieren.frage", n), 17.0, q.w - 28.0), q.x + 14.0, q.y + 30.0, 17.0, Theme::text());
            ui.paragraph(crate::inhalt::t("ui.karrieren.frage_text"), q.x + 14.0, q.y + 54.0, 13.0, q.w - 28.0, Theme::text2());
            if ui.button(Rect::new(q.x + 14.0, q.y + q.h - 44.0, 150.0, 32.0), crate::inhalt::t("ui.karrieren.ja_loeschen"), false) {
                match game::karriere_wegwerfen(&p) { Ok(()) => { s.saves = game::list_saves(); s.error.clear(); } Err(e) => s.error = e }
                frage = None;
            }
            if ui.button(Rect::new(q.x + 174.0, q.y + q.h - 44.0, 150.0, 32.0), crate::inhalt::t("ui.karrieren.behalten"), true) { frage = None; }
        }
        s.loeschen = frage;
        // right: new career wizard
        // the club step uses the whole width (list, club card and map side by side)
        let rr = if s.step == 1 { Rect::new(510.0, 190.0, (crate::ui::breite() - 540.0).max(830.0), (crate::ui::hoehe() - 220.0).max(650.0)) } else { Rect::new(510.0, 190.0, 830.0, 650.0) };
        ui.card_titled(rr, crate::inhalt::t("start.neu"), Theme::gold());
        let steps = [crate::inhalt::t("ui.main.1_welt"), crate::inhalt::t("ui.main.2_verein"), crate::inhalt::t("ui.main.3_trainer"), crate::inhalt::t("ui.main.4_start")];
        for (i, t) in steps.iter().enumerate() {
            let x = rr.x + 200.0 + i as f32 * 140.0;
            ui.text_semi(t, x, rr.y + 26.0, 17.0, if i == s.step { Theme::akzent() } else { Theme::gedaempft() });
        }
        let mut start_new = false;
        let mut start_felder: Vec<pages::Auswahl> = vec![];
        match s.step {
            0 => {
                // Schritt 1: Welt-Ordner wählen und prüfen (Übergabe Daten §7)
                ui.paragraph(crate::inhalt::t("ui.main.waehle_die_fl_welt_auf"), rr.x + 20.0, rr.y + 70.0, 17.0, rr.w - 40.0, Theme::text2());
                let mut pick: Option<std::path::PathBuf> = None;
                let welten = pesdb::welten(&fl27::sider());
                for (k, w) in welten.iter().take(6).enumerate() {
                    let r = Rect::new(rr.x + 20.0 + (k % 3) as f32 * 266.0, rr.y + 120.0 + (k / 3) as f32 * 50.0, 256.0, 42.0);
                    if ui.button(r, &fl27::welt_name(w), false) { pick = Some(w.clone()); }
                }
                // beta 23.11.: the FL27 folder was not found on every PC and a path could not be pasted
                if welten.is_empty() { ui.paragraph(&crate::tf!("ui.main.ordner_nicht_gefunden", fl27::spielordner().display()), rr.x + 20.0, rr.y + 124.0, 15.0, rr.w - 40.0, Theme::achtung()); }
                // without an extracted world: the game's own database from its CPK files (in the background)
                if fl27::sider().is_dir() && welten.iter().all(|w| fl27::welt_name(w) != pesdb::CPK_WELT) {
                    let laeuft = CPK_LESEN.lock().unwrap().is_some();
                    let y = if welten.is_empty() { rr.y + 184.0 } else { rr.y + 220.0 - 44.0 };
                    if welten.len() <= 3 {
                        ui.text(crate::inhalt::t(if laeuft { "ui.main.aus_cpk_laeuft" } else { "ui.main.aus_cpk_hinweis" }), rr.x + 20.0, y + 26.0, 14.0, Theme::text2());
                        if !laeuft && ui.button(Rect::new(rr.x + rr.w - 300.0, y + 4.0, 280.0, 36.0), crate::inhalt::t("ui.main.aus_cpk"), false) {
                            let g = fl27::spielordner();
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || { let _ = tx.send(pesdb::welt_aus_cpk(&g)); });
                            *CPK_LESEN.lock().unwrap() = Some(rx);
                        }
                    }
                }
                let fertig = CPK_LESEN.lock().unwrap().as_ref().and_then(|rx| rx.try_recv().ok());
                if let Some(r) = fertig { *CPK_LESEN.lock().unwrap() = None; match r { Ok(w) => pick = Some(w), Err(e) => { s.error = e; fl27::log(&s.error); } } }
                let ir = Rect::new(rr.x + 20.0, rr.y + 236.0, rr.w - 340.0, 40.0);
                if ui.hit(ir) { s.db_focus = true; }
                let f = s.db_focus;
                ui.input_max(ir, &mut s.db_path, f, crate::inhalt::t("ui.main.pfad_zu_einem_welt_ordner"), 300);
                if ui.button(Rect::new(rr.x + rr.w - 310.0, rr.y + 236.0, 140.0, 40.0), crate::inhalt::t("ui.main.durchsuchen"), false) { *ORDNER_DIALOG.lock().unwrap() = Some(fl27::ordner_dialog()); }
                if ui.button(Rect::new(rr.x + rr.w - 160.0, rr.y + 236.0, 140.0, 40.0), crate::inhalt::t("gm.in.pruefen"), false) { pick = Some(s.db_path.trim().trim_matches('"').into()); }
                let gewaehlt = ORDNER_DIALOG.lock().unwrap().as_ref().and_then(|rx| rx.try_recv().ok());
                if let Some(g) = gewaehlt { *ORDNER_DIALOG.lock().unwrap() = None; if let Some(p) = g { s.db_path = p.display().to_string(); pick = Some(p); } }
                // the game folder (or SiderAddons / livecpk) instead of a world: remember it and list its worlds
                if let Some(p) = pick.clone().filter(|p| pesdb::ableiten(p).is_none()) {
                    if let Some(g) = fl27::spielordner_aus(p.as_path()) {
                        fl27::spielordner_setzen(&g);
                        fl27::log(&format!("Spielordner gewählt: {}", g.display()));
                        let w = pesdb::welten(&fl27::sider());
                        pick = if w.len() == 1 { w.first().cloned() } else { s.error.clear(); None };
                    }
                }
                if let Some(p) = pick {
                    if let Some((g, _)) = pesdb::ableiten(&p) { if g.join(fl27::EXE).is_file() || g.join("SiderAddons").is_dir() { fl27::spielordner_setzen(&g); } } // roh
                    match pesdb::ableiten(&p).ok_or_else(|| crate::inhalt::t("ui.main.dieser_pfad_liegt_in_keinem").to_string())
                        .and_then(|(_, w)| pesdb::pruefen(&w).map(|_| w))
                        .and_then(|w| World::from_pesdb(&w.join("common/etc/pesdb"), 7).map(|x| (w, x))) {
                        Ok((w, world)) => {
                            let ini = std::fs::read_to_string(fl27::sider().join("sider.ini")).unwrap_or_default();
                            let name = fl27::welt_name(&w);
                            let aktiv = ini.lines().any(|l| l.trim_start().starts_with("cpk.root") && l.contains(&format!("\\{name}")));
                            s.welt_info = crate::tf!("ui.main.ligen_vereine_spieler_in_den", name, world.leagues.len(), world.clubs.len(), world.players.iter().filter(|p| p.team >= 0).count(), world.players.iter().filter(|p| p.team < 0).count(), if aktiv { crate::inhalt::t("ui.main.die_welt_ist_in_sider") } else { crate::inhalt::t("ui.main.die_welt_ist_in_sider_2") });
                            let (paare, unklar) = world.zweite_erkennen();
                            s.paare = paare.iter().map(|(z, e)| format!("{} → {}", world.clubs[*z].name, world.clubs[*e].name)).collect();
                            s.unklar = unklar.iter().map(|(z, t)| format!("{}: {t}", world.clubs[*z].name)).collect();
                            s.welt = Some(w);
                            self.template = Some(world);
                            s.error.clear();
                            ui.play(crate::id("bestaetigt"));
                        }
                        Err(e) => { s.error = e; ui.play(crate::id("fehler")); }
                    }
                }
                ui.text(&s.error, rr.x + 20.0, rr.y + 300.0, 16.0, Theme::problem());
                if !s.welt_info.is_empty() {
                    let mut y = ui.paragraph(&s.welt_info, rr.x + 20.0, rr.y + 330.0, 17.0, rr.w - 40.0, Theme::gut()) + 10.0;
                    // second teams found by name (Team.bin has no field for the parent club)
                    let liste = if s.paare.is_empty() { crate::id("keine").to_string() } else { s.paare.iter().take(6).cloned().collect::<Vec<_>>().join(", ") + if s.paare.len() > 6 { " …" } else { "" } };
                    y = ui.paragraph(&crate::tf!("ui.main.zweite_mannschaften_erkannt", s.paare.len(), liste), rr.x + 20.0, y, 15.0, rr.w - 40.0, Theme::text2()) + 6.0;
                    for u in s.unklar.iter().take(3) { y = ui.paragraph(&crate::tf!("ui.main.bitte_pruefen", u), rr.x + 20.0, y, 15.0, rr.w - 40.0, Theme::achtung()) + 2.0; }
                }
                if self.template.is_some() && s.welt.is_some() && ui.button_gold(Rect::new(rr.x + rr.w - 220.0, rr.y + rr.h - 56.0, 200.0, 44.0), crate::inhalt::t("ui.wort.gross_weiter")) { s.step = 1; s.such_focus = true; s.index = None; }
                if ui.button(Rect::new(rr.x + 20.0, rr.y + rr.h - 56.0, 300.0, 44.0), crate::inhalt::t("ui.main.eingebaute_probewelt"), false) { self.template = Some(Self::builtin_world()); s.welt = None; s.step = 1; s.such_focus = true; s.index = None; }
            }
            1 => {
                let w = self.template.as_ref().unwrap();
                // right: the chosen (or marked) club with its map; the lists below use the left part
                let ganz = rr;
                let pw = (ganz.w * 0.34).clamp(380.0, 560.0);
                let info = Rect::new(ganz.x + ganz.w - pw - 8.0, ganz.y + 92.0, pw, ganz.h - 92.0 - 70.0);
                let zeigen = s.team.or_else(|| if s.such.trim().is_empty() { None } else { w.suche(s.index.as_ref()?, &s.such, s.f_land.as_deref(), s.f_liga).get(s.treffer).copied() }).unwrap_or_else(|| w.leagues[s.league.min(w.leagues.len() - 1)].teams.iter().copied().max_by(|a, b| w.clubs[*a].level.partial_cmp(&w.clubs[*b].level).unwrap()).unwrap_or(0));
                let (mut zoom, mut off) = (s.karte_zoom, s.karte_off);
                if let Some(k) = verein_info(ui, info, w, zeigen, &mut zoom, &mut off) { if w.erste_von(k).is_none() { s.team = Some(k); } }
                s.karte_zoom = zoom; s.karte_off = off;
                if s.index.is_none() { s.index = Some(w.such_index()); s.f_land = None; s.f_liga = None; s.league = s.league.min(w.leagues.len() - 1); }
                // search field and filters (country, league, all) on top
                let y0 = rr.y + 44.0;
                let fr = Rect::new(rr.x + 14.0, y0, 260.0, 36.0);
                if ui.hit(fr) { s.such_focus = true; }
                let alt = s.such.clone();
                let enter = ui.input(fr, &mut s.such, s.such_focus, crate::inhalt::t("ui.main.verein_liga_oder_land_suchen"));
                if is_key_pressed(KeyCode::Escape) { s.such.clear(); }
                if s.such != alt { s.treffer = 0; }
                let mut laender: Vec<&str> = w.leagues.iter().map(|l| l.land.as_str()).filter(|l| !l.is_empty()).collect();
                laender.sort(); laender.dedup();
                let ligen: Vec<usize> = (0..w.leagues.len()).filter(|&l| s.f_land.as_deref().map_or(true, |x| w.leagues[l].land == x)).collect();
                let li = s.f_land.as_deref().and_then(|x| laender.iter().position(|l| *l == x));
                // first the country, then its league, each a list with search (Fabian 23.11.); "all" on top
                let gi = s.f_liga.and_then(|x| ligen.iter().position(|l| *l == x));
                let alle = crate::inhalt::t("ui.wort.alle").to_string();
                start_felder = vec![
                    pages::Auswahl { r: Rect::new(rr.x + 284.0, y0, 190.0, 36.0), titel: crate::inhalt::t("ui.wettbewerbe.land").into(), items: std::iter::once(alle.clone()).chain(laender.iter().map(|x| x.to_string())).collect(), sel: li.map_or(0, |i| i + 1) },
                    pages::Auswahl { r: Rect::new(rr.x + 484.0, y0, 210.0, 36.0), titel: crate::inhalt::t("ui.wettbewerbe.wettbewerb").into(), items: std::iter::once(alle).chain(ligen.iter().map(|&l| w.leagues[l].name.clone())).collect(), sel: gi.map_or(0, |i| i + 1) },
                ];
                match pages::auswahl_vorher(ui, &mut self.v, &start_felder) {
                    Some((0, i)) => { s.f_land = i.checked_sub(1).map(|i| laender[i].to_string()); s.f_liga = None; }
                    Some((_, i)) => { s.f_liga = i.checked_sub(1).map(|i| ligen[i]); }
                    None => {}
                }
                if ui.button(Rect::new(rr.x + 704.0, y0, 112.0, 36.0), crate::inhalt::t("suche.gruppe.alle"), s.f_land.is_none() && s.f_liga.is_none()) { s.f_land = None; s.f_liga = None; }
                if let Some(l) = s.f_liga { if !ligen.contains(&s.league) || s.league != l { s.league = l; } } else if !ligen.contains(&s.league) { if let Some(&l) = ligen.first() { s.league = l; } }
                // below the filter row the lists keep left of the club card
                let rr = Rect::new(ganz.x, ganz.y, ganz.w - pw - 12.0, ganz.h);
                let top = y0 + 48.0;
                let hoehe = rr.h - 44.0 - 48.0 - 70.0;
                if !s.such.trim().is_empty() {
                    // hits: crest, club, league, country, strength word; arrows move the mark, Enter takes it
                    let t = w.suche(s.index.as_ref().unwrap(), &s.such, s.f_land.as_deref(), s.f_liga);
                    if t.is_empty() {
                        ui.paragraph(crate::inhalt::t("ui.main.kein_verein_gefunden_pruefe_die"), rr.x + 20.0, top + 30.0, 18.0, rr.w - 40.0, Theme::text2());
                    } else {
                        if is_key_pressed(KeyCode::Down) { s.treffer = (s.treffer + 1).min(t.len() - 1); }
                        if is_key_pressed(KeyCode::Up) { s.treffer = s.treffer.saturating_sub(1); }
                        s.treffer = s.treffer.min(t.len() - 1);
                        if enter && w.erste_von(t[s.treffer]).is_none() { s.team = Some(t[s.treffer]); }
                        let rh = 40.0;
                        let vis = (hoehe / rh) as usize;
                        let ab = s.treffer.saturating_sub(vis - 1);
                        for (k, &c) in t.iter().enumerate().skip(ab).take(vis) {
                            let r = Rect::new(rr.x + 14.0, top + (k - ab) as f32 * rh, rr.w - 28.0, rh - 4.0);
                            let sel = s.team == Some(c);
                            rrect(r.x, r.y, r.w, r.h, 6.0, if sel { alpha(Theme::akzent(), 0.2) } else if k == s.treffer || ui.hover(r) { Theme::ueberlagerung() } else { Theme::erhoehung() });
                            let cl = &w.clubs[c];
                            let lg = &w.leagues[cl.league];
                            ui.club_crest(r.x + 20.0, r.y + r.h / 2.0, 14.0, cl.id, hex(cl.c1), &cl.short);
                            let nm = ui.fit(&cl.name, 17.0, 260.0);
                            let erste = w.erste_von(c);
                            ui.text_semi(&nm, r.x + 44.0, r.y + 25.0, 17.0, if erste.is_some() { Theme::gedaempft() } else { Theme::text() });
                            let ln = ui.fit(&erste.map(|e| crate::tf!("ui.main.gehoert_zu", w.clubs[e].name)).unwrap_or_else(|| lg.name.clone()), 15.0, 220.0);
                            // narrow list (club card beside it): club and league only
                            let breit = r.w >= 780.0;
                            let ln = if breit { ln } else { ui.fit(&ln, 15.0, r.w - 320.0 - 10.0) };
                            ui.text(&ln, r.x + 320.0, r.y + 24.0, 15.0, Theme::text2());
                            if breit {
                                let la = ui.fit(&lg.land, 15.0, 140.0);
                                ui.text(&la, r.x + 550.0, r.y + 24.0, 15.0, Theme::gedaempft());
                                ui.text(&stufe_wort(cl.level), r.x + 700.0, r.y + 24.0, 15.0, Theme::text2());
                            }
                            if ui.hit(r) && erste.is_none() { s.team = Some(c); s.treffer = k; }
                        }
                        ui.text(&crate::tf!("ui.main.treffer_pfeiltasten_und_enter_waehlen", t.len()), rr.x + 20.0, rr.y + rr.h - 70.0, 14.0, Theme::gedaempft());
                    }
                } else {
                    // the leagues with their overall value (mean strength of the clubs), by strength or grouped by country
                    let lw = 290.0;
                    if let Some(k) = ui.segment(Rect::new(rr.x + 14.0, top, lw, 28.0), &[crate::inhalt::t("ui.start.nach_staerke"), crate::inhalt::t("ui.start.nach_land")], s.liga_sort as usize) { s.liga_sort = k as u8; s.scroll = 0; }
                    let wert = |l: usize| { let t = &w.leagues[l].teams; if t.is_empty() { 0.0 } else { t.iter().map(|&c| w.clubs[c].level).sum::<f32>() / t.len() as f32 } };
                    let mut sortiert = ligen.clone();
                    if s.liga_sort == 0 { sortiert.sort_by(|a, b| wert(*b).partial_cmp(&wert(*a)).unwrap()); } else { sortiert.sort_by(|a, b| (w.leagues[*a].land.as_str(), w.leagues[*a].stufe, -(wert(*a) as i32)).cmp(&(w.leagues[*b].land.as_str(), w.leagues[*b].stufe, -(wert(*b) as i32)))); }
                    // rows: a country head before its leagues when grouped
                    let mut zeilen: Vec<Option<usize>> = vec![];
                    for (k, &l) in sortiert.iter().enumerate() { if s.liga_sort == 1 && (k == 0 || w.leagues[sortiert[k - 1]].land != w.leagues[l].land) { zeilen.push(None); } zeilen.push(Some(l)); }
                    let laender_kopf: Vec<String> = { let mut v = vec![]; for (k, &l) in sortiert.iter().enumerate() { if s.liga_sort == 1 && (k == 0 || w.leagues[sortiert[k - 1]].land != w.leagues[l].land) { v.push(w.leagues[l].land.clone()); } } v };
                    let top_l = top + 36.0;
                    let vis = ((hoehe - 36.0) / 32.0) as usize;
                    let la = Rect::new(rr.x + 14.0, top_l, lw, hoehe - 36.0);
                    if la.contains(ui.mouse) && ui.wheel != 0.0 { s.scroll = (s.scroll as i32 - ui.wheel.signum() as i32 * 2).clamp(0, zeilen.len().saturating_sub(vis) as i32) as usize; }
                    s.scroll = s.scroll.min(zeilen.len().saturating_sub(vis));
                    let mut kopf_i = zeilen.iter().take(s.scroll).filter(|z| z.is_none()).count();
                    for (k, z) in zeilen.iter().skip(s.scroll).take(vis).enumerate() {
                        let r = Rect::new(la.x, la.y + k as f32 * 32.0, lw, 30.0);
                        let Some(i) = *z else {
                            ui.text_semi(&laender_kopf.get(kopf_i).cloned().unwrap_or_default().to_uppercase(), r.x + 6.0, r.y + 22.0, 13.0, Theme::gedaempft());
                            kopf_i += 1;
                            continue;
                        };
                        if i == s.league { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::akzent(), 0.18)); } else if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 4.0, Theme::erhoehung()); }
                        let zusatz = if s.liga_sort == 0 { format!("  {}", w.leagues[i].land) } else { String::new() };
                        let nm = ui.fit(&format!("{}{zusatz}", w.leagues[i].name), 16.0, lw - 60.0);
                        ui.text(&nm, r.x + 10.0, r.y + 21.0, 16.0, if i == s.league { Theme::akzent() } else { Theme::text() });
                        ui.pill(r.x + r.w - 22.0, r.y + 15.0, wert(i), 34.0);
                        if ui.hit(r) { s.league = i; s.team = None; }
                    }
                    let mut ts: Vec<usize> = w.leagues[s.league].teams.clone();
                    ts.sort_by(|a, b| w.clubs[*b].level.partial_cmp(&w.clubs[*a].level).unwrap());
                    let cols = (((rr.w - lw - 50.0) / 230.0) as usize).clamp(1, 3);
                    let cw = (rr.w - lw - 50.0) / cols as f32;
                    let rows = (hoehe / 54.0) as usize;
                    for (k, &t) in ts.iter().enumerate().take(rows * cols) {
                        let r = Rect::new(rr.x + lw + 30.0 + (k % cols) as f32 * cw, top + (k / cols) as f32 * 54.0, cw - 8.0, 48.0);
                        let sel = s.team == Some(t);
                        rrect(r.x, r.y, r.w, r.h, 6.0, if sel { alpha(Theme::akzent(), 0.2) } else if ui.hover(r) { Theme::ueberlagerung() } else { Theme::erhoehung() });
                        let cl = &w.clubs[t];
                        ui.club_crest(r.x + 24.0, r.y + 24.0, 16.0, cl.id, hex(cl.c1), &cl.short);
                        let nm = ui.fit(&cl.name, 16.0, r.w - 96.0);
                        // a second team belongs to its first team: shown greyed, not to be chosen
                        let erste = w.erste_von(t);
                        ui.text_semi(&nm, r.x + 48.0, r.y + 21.0, 16.0, if erste.is_some() { Theme::gedaempft() } else { Theme::text() });
                        let unter = erste.map(|e| ui.fit(&crate::tf!("ui.main.gehoert_zu", w.clubs[e].name), 13.0, r.w - 96.0)).unwrap_or_else(|| cl.goal_kind.clone());
                        ui.text(&unter, r.x + 48.0, r.y + 40.0, 13.0, Theme::gedaempft());
                        if erste.is_none() { ui.pill(r.x + r.w - 26.0, r.y + 24.0, cl.level, 34.0); }
                        if ui.hit(r) && erste.is_none() { s.team = Some(t); }
                    }
                    if ts.len() > rows * cols { ui.text(&crate::tf!("ui.main.weitere_vereine_ueber_die_suche", ts.len() - rows * cols), rr.x + lw + 30.0, rr.y + rr.h - 70.0, 14.0, Theme::gedaempft()); }
                }
                if s.team.is_some() && ui.button_gold(Rect::new(ganz.x + ganz.w - 220.0, ganz.y + ganz.h - 56.0, 200.0, 44.0), crate::inhalt::t("ui.wort.gross_weiter")) { s.step = 2; s.ohne = false; s.coach_focus = true; s.such_focus = false; }
                // or no club at all: offers come (Fabian 23.11.)
                if ui.button(Rect::new(ganz.x + ganz.w - 600.0, ganz.y + ganz.h - 56.0, 250.0, 44.0), crate::inhalt::t("tp.ohne_knopf"), false) { s.team = None; s.ohne = true; s.step = 2; s.coach_focus = true; s.such_focus = false; }
                if ui.button(Rect::new(ganz.x + ganz.w - 340.0, ganz.y + ganz.h - 56.0, 110.0, 44.0), crate::inhalt::t("karrieren.zurueck"), false) { s.step = 0; }
            }
            2 => {
                // the coach is created here (Fabian 23.11., like FM): name, birth date, nationality, knowledge, own career, style
                let w = self.template.as_ref().unwrap();
                let t_ = crate::inhalt::t;
                if let Some(t) = s.team {
                    let cl = &w.clubs[t];
                    ui.club_crest(rr.x + 48.0, rr.y + 70.0, 26.0, cl.id, hex(cl.c1), &cl.short);
                    ui.title(&ui.fit(&cl.name, 26.0, rr.w - 120.0), rr.x + 86.0, rr.y + 76.0, 26.0, Theme::text());
                } else { ui.title(t_("tp.ohne_knopf"), rr.x + 20.0, rr.y + 76.0, 26.0, Theme::text()); }
                let mut y = rr.y + 104.0;
                ui.text_semi(crate::inhalt::t("ui.main.dein_name_als_trainer"), rr.x + 20.0, y + 22.0, 15.0, Theme::text());
                let ir = Rect::new(rr.x + 220.0, y, 360.0, 34.0);
                if ui.hit(ir) { s.coach_focus = true; }
                let f = s.coach_focus;
                ui.input(ir, &mut s.coach, f, crate::inhalt::t("ui.main.z_b_max_mustermann"));
                y += 46.0;
                // birth date with steppers
                ui.text_semi(t_("tp.geburt"), rr.x + 20.0, y + 20.0, 15.0, Theme::text());
                let p = &mut s.profil;
                for (k, (wert, lo, hi)) in [(p.tag as i32, 1, 31), (p.monat as i32, 1, 12), (p.jahr as i32, 1950, 2004)].into_iter().enumerate() {
                    let x = rr.x + 220.0 + k as f32 * 130.0;
                    if ui.button(Rect::new(x, y, 32.0, 30.0), "-", false) { let n = (wert - 1).max(lo); match k { 0 => p.tag = n as u8, 1 => p.monat = n as u8, _ => p.jahr = n as u16 } } // roh
                    ui.text_center(&wert.to_string(), x + 56.0, y + 21.0, 16.0, Theme::text());
                    if ui.button(Rect::new(x + 80.0, y, 32.0, 30.0), "+", false) { let n = (wert + 1).min(hi); match k { 0 => p.tag = n as u8, 1 => p.monat = n as u8, _ => p.jahr = n as u16 } } // roh
                }
                ui.text(&crate::tf!("tp.alter", p.alter()), rr.x + 620.0, y + 20.0, 14.0, Theme::text2());
                y += 44.0;
                // nationality: the countries of the world, stepped through
                let mut laender: Vec<String> = w.leagues.iter().map(|l| l.land.clone()).filter(|l| !l.is_empty()).collect();
                laender.sort();
                laender.dedup();
                if p.nation.is_empty() { p.nation = laender.iter().find(|l| l.as_str() == crate::id("Deutschland")).or(laender.first()).cloned().unwrap_or_default(); }
                ui.text_semi(t_("tp.nation"), rr.x + 20.0, y + 20.0, 15.0, Theme::text());
                let ni = laender.iter().position(|l| *l == p.nation).unwrap_or(0);
                if ui.button(Rect::new(rr.x + 220.0, y, 32.0, 30.0), "<", false) && !laender.is_empty() { p.nation = laender[(ni + laender.len() - 1) % laender.len()].clone(); } // roh
                ui.text_center(&p.nation, rr.x + 386.0, y + 21.0, 16.0, Theme::text());
                if ui.button(Rect::new(rr.x + 520.0, y, 32.0, 30.0), ">", false) && !laender.is_empty() { p.nation = laender[(ni + 1) % laender.len()].clone(); } // roh
                y += 44.0;
                let reihen: [(&str, &str, u8); 3] = [("tp.vorwissen", "tp.vw", p.vorwissen), ("tp.karriere", "tp.ka", p.karriere), ("tp.stil", "tp.st", p.stil)]; // roh
                let mut neu = [p.vorwissen, p.karriere, p.stil];
                for (k, (titel, pre, sel)) in reihen.iter().enumerate() {
                    ui.text_semi(t_(titel), rr.x + 20.0, y + 12.0, 15.0, Theme::text());
                    let opts: Vec<&str> = (0..5).map(|i| crate::inhalt::t(&format!("{pre}.{i}"))).collect(); // roh
                    if let Some(x) = ui.segment(Rect::new(rr.x + 20.0, y + 22.0, rr.w - 40.0, 32.0), &opts, *sel as usize) { neu[k] = x as u8; }
                    y += 66.0;
                }
                p.vorwissen = neu[0]; p.karriere = neu[1]; p.stil = neu[2];
                let liz = p.start_lizenz();
                ui.text_semi(&crate::tf!("tp.lizenz", crate::inhalt::t(&format!("lb.lizenz.{liz}"))), rr.x + 20.0, y + 16.0, 17.0, Theme::gold()); // roh
                if (ui.button_gold(Rect::new(rr.x + rr.w - 220.0, rr.y + rr.h - 60.0, 200.0, 46.0), crate::inhalt::t("ui.wort.gross_weiter"))) && !s.coach.trim().is_empty() { s.step = 3; }
                if ui.button(Rect::new(rr.x + rr.w - 340.0, rr.y + rr.h - 60.0, 110.0, 46.0), crate::inhalt::t("karrieren.zurueck"), false) { s.step = 1; }
            }
            _ => {
                // Schritt 4: Start (Original sichern, Datenbank in den Karriere-Ordner kopieren)
                let w = self.template.as_ref().unwrap();
                ui.title(crate::inhalt::t("ui.main.bereit_fuer_den_anstoss"), rr.x + 20.0, rr.y + 80.0, 34.0, Theme::text());
                let welt = s.welt.as_ref().map(|p| fl27::welt_name(p)).unwrap_or_else(|| crate::inhalt::t("ui.main.eingebaute_probewelt_2").into());
                let verein = match s.team {
                    Some(t) => { let cl = &w.clubs[t]; let farm = { let mut v = vec![]; if let Some(z) = w.zweite_von(t) { v.push(crate::tf!("ui.main.zweite_mannschaft", w.clubs[z].name)); } for p in w.partner_von(t) { v.push(crate::tf!("ui.main.farmteam", w.clubs[p].name)); } if v.is_empty() { crate::inhalt::t("ui.main.keine_zweite_mannschaft_kein_farmteam").to_string() } else { v.join(", ") } }; crate::tf!("ui.main.verein", cl.name, w.leagues[cl.league].name, farm) }
                    None => crate::inhalt::t("tp.ohne_zeile").to_string(),
                };
                if s.profil.nation.is_empty() { s.profil.nation = w.leagues.iter().map(|l| l.land.clone()).find(|l| l.as_str() == crate::id("Deutschland")).or_else(|| w.leagues.first().map(|l| l.land.clone())).unwrap_or_default(); }
                let liz = crate::inhalt::t(&format!("lb.lizenz.{}", s.profil.start_lizenz())); // roh
                let lines = [crate::tf!("training.trainer_ist", s.coach.trim()), crate::tf!("tp.zusammenfassung", s.profil.nation.clone(), s.profil.alter(), liz), verein, crate::tf!("ui.main.welt", welt),
                    if s.welt.is_some() { crate::inhalt::t("ui.main.beim_start_sichert_pes_m").into() } else { crate::inhalt::t("ui.main.die_probewelt_hat_keine_faehigkeiten").into() }];
                let mut y = rr.y + 120.0;
                for l in lines { y = ui.paragraph(&l, rr.x + 20.0, y, 17.0, rr.w - 40.0, Theme::text()) + 6.0; }
                // without a club: where the offers may come from
                if s.ohne {
                    ui.text_semi(crate::inhalt::t("tp.angebote_aus"), rr.x + 20.0, y + 16.0, 15.0, Theme::akzent());
                    let modi = [crate::inhalt::t("tp.modus.0"), crate::inhalt::t("tp.modus.1"), crate::inhalt::t("tp.modus.2")];
                    if let Some(k) = ui.segment(Rect::new(rr.x + 20.0, y + 26.0, rr.w - 40.0, 30.0), &modi, s.angebot_modus) { s.angebot_modus = k; }
                    y += 66.0;
                    if s.angebot_modus == 2 {
                        // up to three leagues, top divisions first
                        let mut ls: Vec<usize> = (0..w.leagues.len()).collect();
                        let st = |l: usize| if w.leagues[l].stufe == 0 { 99 } else { w.leagues[l].stufe };
                        ls.sort_by(|a, b| (st(*a), &w.leagues[*a].land, &w.leagues[*a].name).cmp(&(st(*b), &w.leagues[*b].land, &w.leagues[*b].name)));
                        let liste = Rect::new(rr.x + 20.0, y, rr.w - 40.0, rr.y + rr.h - 80.0 - y);
                        if liste.contains(ui.mouse) && ui.wheel != 0.0 { s.ligen_scroll = (s.ligen_scroll as i32 - ui.wheel.signum() as i32 * 3).clamp(0, ls.len().saturating_sub(3) as i32) as usize; }
                        let sp = 3usize;
                        let zw = liste.w / sp as f32;
                        let zeilen = ((liste.h / 30.0) as usize).max(1);
                        for (k, &l) in ls.iter().skip(s.ligen_scroll).take(zeilen * sp).enumerate() {
                            let r = Rect::new(liste.x + (k % sp) as f32 * zw, liste.y + (k / sp) as f32 * 30.0, zw - 6.0, 26.0);
                            let an = s.ligen.contains(&l);
                            rrect(r.x, r.y, r.w, r.h, 13.0, if an { Theme::akzent() } else if ui.hover(r) { Theme::ueberlagerung() } else { Theme::erhoehung() });
                            let lg = &w.leagues[l];
                            let nm = if lg.land.is_empty() || lg.name.contains(&lg.land) { lg.name.clone() } else { format!("{} · {}", lg.land, lg.name) };
                            ui.text(&ui.fit(&nm, 13.0, r.w - 16.0), r.x + 10.0, r.y + 18.0, 13.0, if an { Theme::auf_akzent() } else { Theme::text() });
                            if ui.hit(r) { if an { s.ligen.retain(|&x| x != l); } else if s.ligen.len() < 3 { s.ligen.push(l); } }
                        }
                    }
                }
                if ui.button_gold(Rect::new(rr.x + 20.0, rr.y + rr.h - 70.0, 300.0, 50.0), crate::inhalt::t("ui.main.karriere_starten")) { start_new = true; }
                if ui.button(Rect::new(rr.x + 340.0, rr.y + rr.h - 70.0, 110.0, 50.0), crate::inhalt::t("karrieren.zurueck"), false) { s.step = 2; }
            }
        }
        pages::auswahl_zeichnen(ui, &self.v, &start_felder);
        if start_new {
            let w = self.template.take().unwrap();
            // without a club the career is held by the smallest club of the world until an offer is taken
            let platz = s.team.unwrap_or_else(|| (0..w.clubs.len()).min_by(|a, b| w.clubs[*a].rep.partial_cmp(&w.clubs[*b].rep).unwrap()).unwrap_or(0));
            let mut c = Career::new(w, platz, s.coach.trim());
            c.trainer_uebernehmen(s.profil.clone());
            if s.ohne {
                c.angebot_ligen = match s.angebot_modus { 1 => (0..c.world.leagues.len()).filter(|&l| c.world.leagues[l].land == s.profil.nation).collect(), 2 => s.ligen.clone(), _ => vec![] };
                c.ohne_verein_starten();
            }
            if let Some(welt) = s.welt.clone() {
                c.fl27_welt = welt.display().to_string();
                let s2 = welt.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_else(fl27::sider);
                // the backup of tens of thousands of files runs in the background, the window shows the loading bar
                let k = c.karriere_dir();
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || { let _ = tx.send(fl27::original_sichern(&k, &s2, &welt).and_then(|_| fl27::karriere_pesdb_anlegen(&k, &welt))); });
                self.start_lauf = Some((c, rx));
                return;
            }
            let _ = c.save();
            self.last_inbox = c.inbox.len();
            self.career = Some(c);
            self.screen = Screen::Game;
            self.v = View::new();
            self.ui.play(crate::id("titel"));
        }
        if let Some(p) = load {
            match game::load(&p) {
                Ok(c) => { self.last_inbox = c.inbox.len(); self.career = Some(c); self.screen = Screen::Game; self.v = View::new(); }
                Err(e) => { self.start.error = crate::tf!("ui.main.spielstand_nicht_lesbar", e); }
            }
        }
    }

    fn advance(&mut self) -> bool {
        let Some(c) = self.career.as_mut() else { return false };
        if c.today_match().is_some() { self.v.go(Page::Spieltag); self.fast = false; return false; }
        let decisions = c.open_decisions();
        let t0 = std::time::Instant::now();
        c.undo_merken();
        let r = c.weiter();
        if let Ok(mut t) = pages::TAGESZEIT.lock() { t.push(t0.elapsed().as_secs_f32() * 1000.0); if t.len() > 200 { t.remove(0); } }
        match r {
            Ok(()) => {
                if c.inbox.len() > self.last_inbox { self.ui.play(crate::id("posteingang")); }
                self.last_inbox = c.inbox.len();
                if c.autosave_heute() { let _ = c.save_mit_sicherung(); }
                if c.open_decisions() > decisions || c.press.is_some() || c.today_match().is_some() { self.fast = false; }
                true
            }
            Err(e) => { self.v.toast(&e, false); self.fast = false; false }
        }
    }

    fn game_screen(&mut self) {
        // Einstellungen › Spielstand: save, then back to the start screen (load another career or start a new one)
        if self.v.zum_start {
            if let Some(c) = self.career.take() { let _ = c.save(); fl27::log(&format!("Zurück zum Start: {}", c.save_name)); }
            self.v = View::new();
            self.fast = false;
            self.start.saves = game::list_saves();
            self.start.step = 0;
            self.screen = Screen::Start;
            return;
        }
        if self.fast { self.advance(); }
        let App { ui, v, career, .. } = self;
        let c = career.as_mut().unwrap();
        pages::fl27_und_db(v, c);
        v.hud_seen.clear();
        // accent from the own crest (Einstellungen: "Akzentfarbe aus dem Wappen"), else the shirt colour, else the default blue
        let akzent = if c.akzent_wappen {
            let cl = c.club();
            ui.wappen_farbe(cl.id).or_else(|| [cl.c1, cl.c2].iter().find_map(|&f| crate::ui::hauptfarbe(&[(f >> 16) as u8, (f >> 8) as u8, f as u8, 255])))
        } else { pages::eigener_akzent() };
        // with the crest accent the banners (club colour) and the blue card heads follow it too (Fabian 22.11.: still blue)
        // "per competition" theme (Fabian 24.11.): on a matchday the UI wears the competition's colours
        let spieltag = if c.thema_wettbewerb { c.today_match().map(|t| pages::thema_farben(&pages::wo_look(c, t.0))) } else { None };
        let akzent = spieltag.map(|x| x.1).or(akzent);
        let bereich = match (akzent, area_color(v.page)) { (Some(a), 0x38bdf8) => a, (_, b) => b };
        let bereich = spieltag.map_or(bereich, |x| x.0);
        // the tables and rankings wear the colour of the chosen competition
        let kopf = if matches!(v.page, Page::Tabellen | Page::Ranglisten) { pages::liga_farbe(v, c) } else if let Some(x) = spieltag { x.0 } else { akzent.unwrap_or(c.club().c1) };
        Theme::set(v.dark, kopf, bereich);
        Theme::set_akzent(akzent.unwrap_or(0x38bdf8));
        clear_background(Theme::grund());
        let (over, title, question) = stage_text(v);
        // the main page's tabs sit in the stage band, in place of the question; the tab last open is remembered
        let haupt = haupt_von(v.page);
        let rs: Vec<(Page, &str)> = haupt.map(|(_, h)| reiter(h, c)).unwrap_or_default();
        if let Some((_, h)) = haupt { v.reiter_merk.insert(h.0, v.page); }
        ui.stage(Rect::new(x0() + 4.0, 62.0, cbreite(), 132.0), over, title, if rs.len() > 1 { "" } else { question });
        if rs.len() > 1 {
            let mut x = x0() + 28.0;
            for (p, label) in &rs {
                let z = zaehler(*p, c);
                let w = ui.measure_semi(label, 16.0) + 24.0 + if z > 0 { 26.0 } else { 0.0 };
                let r = Rect::new(x, 62.0 + 132.0 - 44.0, w, 30.0);
                let sel = *p == v.page;
                ui.rahmen(r);
                rrect(r.x, r.y, r.w, r.h, 15.0, if sel { hex(0xffffff) } else if ui.hover(r) { alpha(hex(0xffffff), 0.25) } else { alpha(hex(0xffffff), 0.12) });
                ui.text_semi(label, r.x + 12.0, r.y + 21.0, 16.0, if sel { Theme::verein_tief() } else { hex(0xffffff) });
                if z > 0 { rrect(r.x + r.w - 32.0, r.y + 7.0, 22.0, 16.0, 8.0, Theme::problem()); ui.text_center(&z.to_string(), r.x + r.w - 21.0, r.y + 20.0, 12.0, hex(0xffffff)); }
                if ui.hit(r) && !sel { v.go(*p); ui.play(crate::id("klick")); }
                x += w + 8.0;
            }
        }
        // content
        if v.page != Page::Spieltag && v.mv.as_ref().map(|m| m.phase == 2).unwrap_or(false) { v.mv = None; }
        // a reward moment (meilensteine.rs) takes the whole screen and all clicks until it is closed
        let feier_offen = !c.feiern.is_empty() || !c.auslosungen.is_empty();
        let modal_open = v.modal != Modal::None || feier_offen;
        // an open right-click menu takes the clicks like a window (but without darkening the page)
        let kontext_offen = v.kontext.is_some() || v.verein_kontext.is_some();
        let saved_click = ui.clicked;
        if modal_open || kontext_offen { ui.clicked = false; }
        draw_page(ui, v, c);
        // a right click on a player row opens his menu (the nearest photo in the row)
        if is_mouse_button_pressed(MouseButton::Right) && v.modal == Modal::None {
            if let Some(id) = ui.foto_unter_maus() { if let Some(p) = c.world.players.iter().position(|x| x.id == id) { v.kontext = Some((p, ui.mouse)); } }
            // else a club row (its crest): the club's menu
            else if let Some(id) = ui.wappen_unter_maus() { if let Some(k) = c.world.clubs.iter().position(|x| x.id == id) { v.verein_kontext = Some((k, ui.mouse)); } }
        }
        // guided first week: looking at the squad and handing in the line-up are steps
        match v.page { Page::Kader => c.erste_woche_schritt(0), Page::Aufstellung => c.erste_woche_schritt(2), _ => {} }
        // a frame holds only the texts of its own area: what is drawn later (bar, sidebar, windows, toasts) starts free
        ui.rahmen.borrow_mut().clear();
        let t = (get_time() as f32 - v.changed_at) / 0.25;
        if t < 1.0 { draw_rectangle(x0() - 4.0, 196.0, breite() - x0() + 4.0, hoehe() - 196.0, alpha(Theme::grund(), 1.0 - t)); }
        ui.clicked = saved_click && !modal_open && !kontext_offen;
        let weiter = Self::top_bar(ui, v, c);
        Self::sidebar(ui, v, c);
        ui.rahmen.borrow_mut().clear();
        if v.modal == Modal::None && kontext_offen { ui.clicked = saved_click; pages::kontext_menue(ui, v, c); pages::verein_kontext_menue(ui, v, c); }
        if feier_offen {
            ui.clicked = saved_click;
            if c.feiern.is_empty() { pages::auslosung_overlay(ui, v, c); } else { pages::feier_overlay(ui, v, c); }
        } else if modal_open {
            ui.clicked = saved_click;
            draw_rectangle(0.0, 0.0, breite(), hoehe(), Color::new(0.0, 0.0, 0.0, 0.6));
            match v.modal {
                Modal::Player(p) => { v.sel_player = Some(p); v.modal = Modal::None; v.go(Page::Spielerprofil); }
                Modal::Aktionen(p) => player_modal(ui, v, c, p),
                Modal::Neg(i) => neg_modal(ui, v, c, i),
                Modal::Suche => pages::suche_overlay(ui, v, c),
                Modal::Spielplan(k) => pages::spielplan_modal(ui, v, c, k),
                Modal::None => {}
            }
        }
        ui.rahmen.borrow_mut().clear();
        if let Some((s, t0, ok)) = v.toast.clone() {
            let age = get_time() as f32 - t0;
            if age > 3.0 { v.toast = None; } else {
                let w = ui.measure_semi(&s, 18.0) + 40.0;
                let y = hoehe() - 70.0 + (1.0 - (age * 6.0).min(1.0)) * 30.0;
                rrect(breite() / 2.0 - w / 2.0, y, w, 40.0, 8.0, if ok { hex(0x173828) } else { hex(0x3A1C1A) });
                ui.text_center(&s, breite() / 2.0, y + 26.0, 18.0, if ok { Theme::gut() } else { Theme::problem() });
            }
        }
        crate::pages::wirkung_karte(ui, c);
        ui.nachweise_zeichnen();
        // the full text of a cut text under the mouse (text_passend, buttons)
        let tipp = ui.tooltip.borrow().clone();
        if let Some(t) = tipp {
            let w = ui.measure(&t, 15.0) + 20.0;
            let (x, y) = ((ui.mouse.x + 12.0).min(crate::ui::breite() - w - 4.0), ui.mouse.y + 22.0);
            rrect(x, y, w, 28.0, 6.0, Theme::flaeche());
            draw_rectangle_lines(x, y, w, 28.0, 1.0, Theme::akzent());
            ui.text(&t, x + 10.0, y + 19.0, 15.0, Theme::text());
        }
        let key_weiter = !v.market_focus && v.auswahl.is_none() && v.modal == Modal::None && (is_key_pressed(KeyCode::Space));
        // Ctrl+Z: the last day step or own match undone, exactly as it was
        if v.modal == Modal::None && is_key_pressed(KeyCode::Z) && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            if let Some(c) = self.career.as_mut() {
                if c.rueckgaengig() { self.v.mv = None; self.last_inbox = c.inbox.len(); self.v.toast(crate::inhalt::t("undo.ok"), true); } else { self.v.toast(crate::inhalt::t("undo.nichts"), false); }
            }
        }
        if weiter == 1 || key_weiter { if self.advance() { self.ui.play(crate::id("tagesabschluss")); } }
        if weiter == 2 { self.fast = true; }
    }

    /// Returns 1 when "Nächster Tag" was clicked, 2 for "bis zum nächsten Termin".
    fn top_bar(ui: &mut Ui, v: &mut View, c: &mut Career) -> u8 {
        draw_rectangle(x0() - 18.0, 0.0, breite(), 54.0, Theme::grund());
        draw_rectangle(x0() - 18.0, 54.0, breite(), 1.0, Theme::linie());
        let mut out = 0;
        if ui.button(Rect::new(x0() + 4.0, 11.0, 32.0, 32.0), "<", false) { if let Some(p) = v.back.pop() { v.fwd.push(v.page); v.page = p; v.changed_at = get_time() as f32; } }
        if ui.button(Rect::new(x0() + 42.0, 11.0, 32.0, 32.0), ">", false) { if let Some(p) = v.fwd.pop() { v.back.push(v.page); v.page = p; v.changed_at = get_time() as f32; } }
        v.hud_seen.push(c.club().name.clone());
        // search pill
        let sr = Rect::new(x0() + 90.0, 9.0, 300.0, 36.0);
        rrect(sr.x, sr.y, sr.w, sr.h, 18.0, Theme::linie());
        rrect(sr.x + 1.0, sr.y + 1.0, sr.w - 2.0, sr.h - 2.0, 17.0, if ui.hover(sr) { Theme::erhoehung() } else { Theme::flaeche() });
        draw_circle_lines(sr.x + 22.0, sr.y + 17.0, 6.0, 1.8, Theme::text2());
        draw_line(sr.x + 26.5, sr.y + 21.5, sr.x + 30.0, sr.y + 25.0, 1.8, Theme::text2());
        ui.text(crate::inhalt::t("ui.main.spieler_oder_verein_suchen"), sr.x + 40.0, sr.y + 24.0, 17.0, Theme::text2());
        if ui.hit(sr) { v.modal = Modal::Suche; v.suche_text.clear(); v.suche_fuer = "-".into(); }
        // bookmark the page you are on (star next to the search); bookmarks show in the empty search
        let br = Rect::new(sr.x + sr.w + 64.0, 13.0, 28.0, 28.0);
        let key = format!("{:?}", v.page);
        let gemerkt = c.lesezeichen.contains(&key);
        draw_poly(br.x + 14.0, br.y + 14.0, 5, 10.0, -90.0, if gemerkt { Theme::gold() } else if ui.hover(br) { Theme::text2() } else { Theme::linie() });
        if ui.hit(br) { if gemerkt { c.lesezeichen.retain(|x| *x != key); } else { c.lesezeichen.push(key); } }
        let un = c.unread();
        let ur = Rect::new(x0() + 404.0, 13.0, 46.0, 28.0);
        rrect(ur.x, ur.y, ur.w, ur.h, 14.0, if un > 0 { Theme::problem() } else { Theme::erhoehung() });
        ui.text_center(&format!("{un}"), ur.x + ur.w / 2.0, ur.y + 20.0, 16.0, if un > 0 { hex(0xffffff) } else { Theme::text2() });
        if ui.hit(ur) { v.go(Page::Posteingang); }
        // FL27 running: end it from anywhere (the result is saved first)
        if v.fl27_laeuft && v.fl27_ende.is_none() && ui.button(Rect::new(664.0, 13.0, 146.0, 28.0), crate::inhalt::t("ui.main.fl27_beenden"), false) { pages::fl27_beenden_klick(v); }
        // date card
        // the right part of the bar follows the layout's right edge (wider windows)
        let dx = breite() - crate::ui::W_BASIS;
        let dr = Rect::new(820.0 + dx, 6.0, 260.0, 42.0);
        ui.card(dr);
        let dl = date::long(c.world.season, c.world.day);
        ui.text_center(&dl, dr.x + dr.w / 2.0, dr.y + 19.0, 18.0, Theme::text());
        v.hud_seen.push(dl);
        let sub = match c.next_match() {
            Some(f) => crate::tf!("ui.main.gegen", if f.h == c.own { crate::inhalt::t("zentrale.k.heim") } else { crate::inhalt::t("zentrale.k.auswaerts") }, c.world.clubs[if f.h == c.own { f.a } else { f.h }].short, if f.day == c.world.day { crate::inhalt::t("ui.wort.heute").to_string() } else { crate::tf!("zentrale.in_tagen.viele", f.day - c.world.day) }),
            None => crate::inhalt::t("ui.main.saisonende").into(),
        };
        ui.text_center(&sub, dr.x + dr.w / 2.0, dr.y + 36.0, 13.0, Theme::text2());
        let gr = Rect::new(1090.0 + dx, 13.0, 28.0, 28.0);
        draw_circle_lines(gr.x + 14.0, gr.y + 14.0, 9.0, 2.0, if ui.hover(gr) { Theme::akzent() } else { Theme::text2() });
        draw_circle(gr.x + 14.0, gr.y + 14.0, 3.0, Theme::text2());
        if ui.hit(gr) { v.go(Page::Optionen); }
        // Nächster Tag (guide §2: a day is one step)
        let match_today = c.today_match().is_some();
        // something that can't wait (Was jetzt?, Spielgefühl Teil 4): „Weiter“ leads there instead of the next day
        // once there, „Weiter“ starts the next day as usual: the person may leave it on purpose (a skipped press conference counts)
        let zuerst = Some(c.was_jetzt()).filter(|w| w.vorrang && !c.fired && !match_today && w.ziel != Some(v.page));
        let label = if c.fired { crate::inhalt::t("ui.main.angebote_ansehen") } else if match_today { crate::inhalt::t("ui.main.zum_spiel") } else if zuerst.is_some() { crate::inhalt::t("ui.main.zuerst") } else if c.deadline() && c.dl_stunde < 12 { crate::inhalt::t("ui.dl.naechste_stunde") } else { crate::inhalt::t("ui.main.naechster_tag") };
        // deadline day: the clock beside the button, red, ticking up hour by hour
        if c.deadline() && !c.fired {
            let uhr = crate::tf!("ui.dl.uhr", 9 + c.dl_stunde.min(12) as u32);
            let dr = Rect::new(1130.0 + dx, 52.0, 256.0, 24.0);
            let puls = 0.75 + 0.25 * (ui.time * 3.0).sin().abs();
            rrect(dr.x, dr.y, dr.w, dr.h, 12.0, alpha(Theme::problem(), puls));
            ui.text_center(&uhr, dr.x + dr.w / 2.0, dr.y + 17.0, 14.0, hex(0xffffff));
        }
        v.hud_seen.push("Weiter".into()); // roh
        let wr = Rect::new(1130.0 + dx, 6.0, 216.0, 42.0);
        let h = ui.hover(wr);
        rrect(wr.x, wr.y + 4.0, wr.w + 40.0, wr.h, 10.0, alpha(Theme::akzent(), 0.3));
        rrect(wr.x, wr.y, wr.w + 40.0, wr.h, 10.0, if h { hex(0x6fd0fa) } else { Theme::akzent() });
        let tw = measure_text(label, Some(&ui.head), 24, 1.0).width;
        ui.title(label, wr.x + wr.w / 2.0 - tw / 2.0, wr.y + 30.0, 24.0, Theme::auf_akzent());
        draw_line(wr.x + wr.w, wr.y + 8.0, wr.x + wr.w, wr.y + 34.0, 1.0, alpha(Theme::auf_akzent(), 0.3));
        let fr = Rect::new(wr.x + wr.w, wr.y, 40.0, 42.0);
        draw_triangle(vec2(fr.x + 13.0, fr.y + 17.0), vec2(fr.x + 27.0, fr.y + 17.0), vec2(fr.x + 20.0, fr.y + 25.0), Theme::auf_akzent());
        if ui.hit(wr) {
            if c.fired { if !c.inbox.iter().any(|m| !m.done && matches!(m.action, game::Action::JobOffer { .. })) { c.angebote_ohne_verein(); } v.go(Page::Posteingang); } else if match_today { v.go(Page::Spieltag); }
            else if let Some(z) = &zuerst { if let Some(p) = z.ziel { v.go(p); } if z.msg.is_some() { v.sel_msg = z.msg; } }
            else { out = 1; }
        }
        if ui.hit(fr) && !match_today && !c.fired && zuerst.is_none() { out = 2; v.toast(crate::inhalt::t("ui.main.bis_zum_naechsten_termin"), true); }
        out
    }

    /// Linke Navigation (guide §2): crest with a glow, club name and league, small upper-case groups,
    /// active entry as a club-tone gradient with a 3 px stripe, red counters.
    fn sidebar(ui: &mut Ui, v: &mut View, c: &mut Career) {
        let sw = x0() - 18.0;
        let zu = SEITENLEISTE_ZU.load(std::sync::atomic::Ordering::Relaxed);
        draw_rectangle(0.0, 0.0, sw, hoehe(), if Theme::dark() { hex(0x0b0e13) } else { hex(0xfffdf9) });
        // a simple background (Fabian 23.11.): a soft wash of the club colour from the top and thin diagonal stripes
        gradient_v(0.0, 0.0, sw, 260.0, alpha(Theme::verein(), if Theme::dark() { 0.22 } else { 0.12 }), alpha(Theme::verein(), 0.0));
        let mut x = -hoehe();
        while x < sw { draw_line(x, hoehe(), x + hoehe() * 0.6, 0.0, 1.0, alpha(Theme::verein(), if Theme::dark() { 0.05 } else { 0.035 })); x += 22.0; }
        ui.rahmen(Rect::new(0.0, 0.0, sw, hoehe()));
        draw_rectangle(sw, 0.0, 1.0, hoehe(), Theme::linie());
        let cl = c.club().clone();
        if zu { Self::sidebar_zu(ui, v, c, sw); return; }
        for k in 0..6 { draw_circle(34.0, 40.0, 23.0 + k as f32 * 2.5, alpha(Theme::verein(), 0.38 - k as f32 * 0.06)); }
        ui.club_crest(34.0, 40.0, 23.0, cl.id, Theme::verein(), &cl.short);
        let nm = ui.fit(&cl.name, 19.0, sw - 74.0);
        ui.title(&nm, 66.0, 38.0, 19.0, Theme::text());
        ui.text(&c.world.leagues[cl.league].name, 66.0, 56.0, 14.0, Theme::text2());
        // fold the bar in, open all groups
        if ui.button(Rect::new(sw - 38.0, 68.0, 28.0, 24.0), "<", false) { SEITENLEISTE_ZU.store(true, std::sync::atomic::Ordering::Relaxed); }
        if ui.button(Rect::new(10.0, 68.0, sw - 56.0, 24.0), crate::inhalt::t(if v.menu_alle { "ui.main.menue_zu" } else { "ui.main.menue_alle" }), v.menu_alle) { v.menu_alle = !v.menu_alle; }
        let mut y = 112.0;
        // all open: smaller rows, so the 17 entries fit above the match-day button
        let (zh, zf) = if v.menu_alle { (26.0, 16.0) } else { (34.0, 18.0) };
        let aktiv = haupt_von(v.page).map(|(g, h)| (g, h.0));
        for (gi, (g, hs)) in MENU.iter().enumerate() {
            let open = v.menu_alle || aktiv.map_or(false, |a| a.0 == gi);
            let hr = Rect::new(8.0, y - 6.0, sw - 16.0, 22.0);
            let badge = match gi { 0 => c.unread(), 1 => c.squad().iter().filter(|&&p| c.world.players[p].unhappy_days > 0).count(), 2 => zaehler(Page::Verhandlungen, c), _ => 0 };
            ui.text_semi(crate::inhalt::t(g), 18.0, y + 10.0, 13.0, if open { Theme::verein() } else { Theme::gedaempft() });
            if badge > 0 { let bx = 18.0 + ui.measure_semi(crate::inhalt::t(g), 13.0) + 16.0; rrect(bx - 11.0, y - 2.0, 22.0, 16.0, 8.0, Theme::problem()); ui.text_center(&format!("{badge}"), bx, y + 11.0, 12.0, hex(0xffffff)); }
            ui.text_semi(&format!("{}", gi + 1), sw - 24.0, y + 10.0, 13.0, Theme::gedaempft());
            let ziel = |v: &View, h: &Haupt| v.reiter_merk.get(h.0).copied().filter(|p| reiter(*h, c).iter().any(|x| x.0 == *p)).unwrap_or(h.1[0].0);
            if ui.hit(hr) { let p = ziel(v, &hs[0]); v.go(p); }
            // keys 1 to 7 open the groups
            let key = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6, KeyCode::Key7][gi];
            if is_key_pressed(key) && v.modal == Modal::None && !v.market_focus && v.page != Page::Posteingang && v.page != Page::Presse && v.page != Page::Spieltag && v.page != Page::Kabine { let p = ziel(v, &hs[0]); v.go(p); }
            y += 22.0;
            if open {
                for h in hs.iter() {
                    let r = Rect::new(8.0, y, sw - 16.0, zh - 2.0);
                    let sel = aktiv.map_or(false, |a| a.1 == h.0);
                    if sel { ui_gradient(r, Theme::verein_tief()); draw_rectangle(r.x, r.y, 3.0, r.h, Theme::verein()); } else if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 8.0, Theme::erhoehung()); }
                    ui.text_semi(&ui.fit(crate::inhalt::t(h.0), zf, sw - 80.0), 38.0, y + zh * 0.65, zf, if sel && Theme::dark() { hex(0xffffff) } else if sel { Theme::verein_tief() } else { Theme::text() });
                    draw_circle_lines(23.0, y + zh / 2.0 - 1.0, 5.0, 1.8, if sel { Theme::verein() } else { Theme::gedaempft() });
                    let z: usize = h.1.iter().map(|x| zaehler(x.0, c)).sum();
                    if z > 0 { rrect(r.x + r.w - 30.0, y + zh / 2.0 - 9.0, 24.0, 16.0, 8.0, Theme::problem()); ui.text_center(&format!("{z}"), r.x + r.w - 18.0, y + zh / 2.0 + 4.0, 12.0, hex(0xffffff)); }
                    if ui.hit(r) { let p = ziel(v, h); v.go(p); }
                    y += zh;
                }
            }
            y += 10.0;
        }
        // music on: next-song button and the title playing now, above settings and match day (Fabian 24.11.)
        let musik_an = !musik::url().is_empty();
        if musik_an {
            let y = hoehe() - 130.0;
            let b = Rect::new(10.0, y, 28.0, 28.0);
            rrect(b.x, b.y, b.w, b.h, 14.0, if ui.hover(b) { Theme::verein() } else { Theme::erhoehung() });
            draw_triangle(vec2(b.x + 8.0, b.y + 8.0), vec2(b.x + 8.0, b.y + 20.0), vec2(b.x + 17.0, b.y + 14.0), Theme::text());
            draw_rectangle(b.x + 18.0, b.y + 8.0, 2.5, 12.0, Theme::text());
            if ui.hit(b) { musik::weiter(); }
            let t = musik::titel();
            ui.text(crate::inhalt::t("ui.musik.jetzt"), 46.0, y + 10.0, 11.0, Theme::gedaempft());
            ui.text(&ui.fit(if t.is_empty() { "…" } else { &t }, 14.0, sw - 56.0), 46.0, y + 26.0, 14.0, Theme::text2());
        }
        if c.today_match().is_some() {
            let r = Rect::new(10.0, hoehe() - if musik_an { 182.0 } else { 150.0 }, sw - 20.0, 44.0);
            rrect(r.x, r.y, r.w, r.h, 12.0, Theme::verein());
            ui.title(crate::inhalt::t("ui.main.heute_spieltag"), r.x + 14.0, r.y + 29.0, 20.0, Theme::verein_auf());
            if ui.hit(r) { v.go(Page::Spieltag); }
        }
        draw_rectangle(14.0, hoehe() - 96.0, sw - 28.0, 1.0, Theme::linie());
        for (i, (label, p)) in [(crate::inhalt::t("bereich.system"), Page::Optionen), (crate::inhalt::t("bereich.spieltag"), Page::Spieltag)].iter().enumerate() {
            let r = Rect::new(8.0, hoehe() - 86.0 + i as f32 * 34.0, sw - 16.0, 30.0);
            if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 8.0, Theme::erhoehung()); }
            ui.text(label, 38.0, r.y + 21.0, 17.0, Theme::text2());
            if ui.hit(r) { v.go(*p); }
        }
    }

    /// The folded bar: crest, one numbered button per group (opens its last page), settings at the bottom.
    fn sidebar_zu(ui: &mut Ui, v: &mut View, c: &mut Career, sw: f32) {
        let cl = c.club().clone();
        let mx = sw / 2.0;
        ui.club_crest(mx, 32.0, 18.0, cl.id, Theme::verein(), &cl.short);
        if ui.button(Rect::new(mx - 14.0, 60.0, 28.0, 24.0), ">", false) { SEITENLEISTE_ZU.store(false, std::sync::atomic::Ordering::Relaxed); }
        let aktiv = haupt_von(v.page).map(|(g, _)| g);
        for (gi, (g, hs)) in MENU.iter().enumerate() {
            let r = Rect::new(6.0, 100.0 + gi as f32 * 46.0, sw - 12.0, 40.0);
            let sel = aktiv == Some(gi);
            if sel { rrect(r.x, r.y, r.w, r.h, 8.0, Theme::verein_tief()); draw_rectangle(r.x, r.y, 3.0, r.h, Theme::verein()); } else if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 8.0, Theme::erhoehung()); }
            let kurz = format!("{}", gi + 1);
            ui.text_center(&kurz, mx, r.y + 26.0, 17.0, if sel && Theme::dark() { hex(0xffffff) } else if sel { Theme::verein_auf() } else { Theme::text() });
            let badge = match gi { 0 => c.unread(), 2 => zaehler(Page::Verhandlungen, c), _ => 0 };
            if badge > 0 { draw_circle(r.x + r.w - 6.0, r.y + 6.0, 5.0, Theme::problem()); }
            if ui.hover(r) { let tw = ui.measure_semi(crate::inhalt::t(g), 14.0); rrect(sw + 6.0, r.y + 6.0, tw + 20.0, 28.0, 6.0, Theme::flaeche()); rrect_line(sw + 6.0, r.y + 6.0, tw + 20.0, 28.0, 6.0, Theme::linie()); ui.text_semi(crate::inhalt::t(g), sw + 16.0, r.y + 25.0, 14.0, Theme::text()); }
            if ui.hit(r) { let h = &hs[0]; let p = v.reiter_merk.get(h.0).copied().filter(|p| reiter(*h, c).iter().any(|x| x.0 == *p)).unwrap_or(h.1[0].0); v.go(p); }
        }
        if !musik::url().is_empty() {
            let b = Rect::new(mx - 14.0, hoehe() - 124.0, 28.0, 28.0);
            rrect(b.x, b.y, b.w, b.h, 14.0, if ui.hover(b) { Theme::verein() } else { Theme::erhoehung() });
            draw_triangle(vec2(b.x + 8.0, b.y + 8.0), vec2(b.x + 8.0, b.y + 20.0), vec2(b.x + 17.0, b.y + 14.0), Theme::text());
            draw_rectangle(b.x + 18.0, b.y + 8.0, 2.5, 12.0, Theme::text());
            if ui.hit(b) { musik::weiter(); }
        }
        for (i, p) in [Page::Optionen, Page::Spieltag].iter().enumerate() {
            let r = Rect::new(6.0, hoehe() - 86.0 + i as f32 * 40.0, sw - 12.0, 34.0);
            if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 8.0, Theme::erhoehung()); }
            let label = crate::inhalt::t(if i == 0 { "bereich.system" } else { "bereich.spieltag" });
            ui.text_center(&label.chars().take(2).collect::<String>(), mx, r.y + 23.0, 15.0, if i == 1 && c.today_match().is_some() { Theme::verein() } else { Theme::text2() });
            if ui.hit(r) { v.go(*p); }
        }
    }

    // ------------------------------------------------------------ start test
    fn start_test_step(&mut self) -> bool {
        let st = self.starttest.as_mut().unwrap();
        st.frame += 1;
        if st.frame % 6 != 0 { return false; }
        let s = &mut self.start;
        let (rx, ry, rw, rh) = (510.0, 190.0, 830.0, 650.0);
        let mut ende = st.frame > 3000;
        match st.stage {
            0 => {
                if let Some(w) = &st.welt { s.db_path = w.clone(); self.ui.inject_click = Some(vec2(rx + rw - 90.0, ry + 256.0)); } // Prüfen
                else { self.ui.inject_click = Some(vec2(rx + 170.0, ry + rh - 34.0)); } // Eingebaute Probewelt
                st.stage = 1;
            }
            1 => {
                if st.welt.is_some() && s.step == 0 {
                    if s.welt.is_some() { self.ui.inject_click = Some(vec2(rx + rw - 120.0, ry + rh - 34.0)); } // WEITER
                    else if !s.error.is_empty() { st.steps.push((crate::id("welt").into(), false, s.error.clone())); ende = true; }
                } else if s.step == 1 {
                    let w = self.template.as_ref().unwrap();
                    st.steps.push((crate::id("welt").into(), true, crate::tf!("ui.main.welt_geladen_ligen_vereine", w.leagues.len(), w.clubs.len())));
                    let best = (0..w.clubs.len()).max_by(|a, b| w.clubs[*a].level.partial_cmp(&w.clubs[*b].level).unwrap()).unwrap();
                    st.team = w.clubs[best].name.clone();
                    s.such = st.team.chars().take(5).collect();
                    st.stage = 2;
                }
            }
            2 => { self.ui.inject_click = Some(vec2(rx + 300.0, ry + 92.0 + 18.0)); st.stage = 3; } // first hit
            3 => {
                let w = self.template.as_ref().unwrap();
                let ok = s.team.map(|t| w.clubs[t].name == st.team).unwrap_or(false);
                st.steps.push((crate::id("verein").into(), ok, crate::tf!("ui.main.suche", s.such, s.team.map(|t| w.clubs[t].name.clone()).unwrap_or(crate::inhalt::t("ui.wort.nichts").into()))));
                if !ok { ende = true; } else { self.ui.inject_click = Some(vec2(rx + rw - 120.0, ry + rh - 34.0)); st.stage = 4; }
            }
            4 => {
                if s.step == 2 { s.coach = "Test Trainer".into(); self.ui.inject_click = Some(vec2(rx + rw - 120.0, ry + rh - 37.0)); st.stage = 5; } // roh
            }
            5 => {
                if s.step == 3 { st.steps.push((crate::id("trainer").into(), true, crate::inhalt::t("ui.main.trainername_uebernommen").into())); self.ui.inject_click = Some(vec2(rx + 170.0, ry + rh - 45.0)); st.stage = 6; }
            }
            _ => {
                let ok = self.screen == Screen::Game && self.career.as_ref().map(|c| c.club().name == st.team && c.coach == "Test Trainer").unwrap_or(false); // roh
                st.steps.push((crate::id("start").into(), ok, crate::tf!("ui.main.karriere_gestartet", self.career.as_ref().map(|c| c.club().name.clone()).unwrap_or_default())));
                ende = true;
            }
        }
        if ende {
            if st.stage < 6 && st.steps.iter().all(|x| x.1) { st.steps.push((crate::id("ablauf").into(), false, crate::tf!("ui.main.haengt_in_stufe_schritt", st.stage, s.step + 1))); }
            let steps: Vec<serde_json::Value> = st.steps.iter().map(|(s, p, d)| serde_json::json!({"step": s, "passed": p, "detail": d})).collect();
            let _ = std::fs::write(&st.path, serde_json::to_string_pretty(&serde_json::json!({"version": 1, "steps": steps})).unwrap());
        }
        ende
    }

    // ------------------------------------------------------------ self-test
    fn self_test_step(&mut self) -> bool {
        let st = self.selftest.as_mut().unwrap();
        st.frame += 1;
        let f = st.frame;
        let c = self.career.as_ref().unwrap();
        match st.stage {
            0 => { st.day0 = c.world.day; st.stage = 1; }
            1 => {
                // press the "Weiter" button like a player would, every 4 frames
                if f % 4 == 0 { self.ui.inject_click = Some(vec2(1230.0, 27.0)); }
                let d = c.world.day - st.day0;
                if d >= 5 || f > 200 {
                    st.steps.push((crate::id("move").into(), d >= 3, crate::tf!("ui.main.tag_schritt_per_knopf_tage", d, date::short(c.world.season, st.day0), date::short(c.world.season, c.world.day))));
                    st.stage = 2;
                }
            }
            2 => {
                // keep pressing until the match day page opens, then kick off and play the match
                if self.v.page != Page::Spieltag {
                    if f % 3 == 0 { self.ui.inject_click = Some(vec2(1230.0, 27.0)); }
                    if f > 900 { st.steps.push((crate::id("act").into(), false, crate::inhalt::t("ui.main.kein_spieltag_erreicht").into())); st.stage = 3; }
                } else {
                    let c = self.career.as_mut().unwrap();
                    let before = c.world.table(c.league()).iter().find(|r| r.club == c.own).map(|r| r.p).unwrap_or(0);
                    let res = c.prepare_match(None);
                    let (hn, an, hg, ag) = (c.world.clubs[res.h].name.clone(), c.world.clubs[res.a].name.clone(), res.hg, res.ag);
                    let nratings = res.ratings.len();
                    c.apply_own_match(res);
                    let after = c.world.table(c.league()).iter().find(|r| r.club == c.own).map(|r| r.p).unwrap_or(0);
                    let ok = after == before + 1 && nratings >= 22 && c.end_day().is_ok();
                    st.steps.push((crate::id("act").into(), ok, crate::tf!("ui.main.spiel_gespielt_spielernoten_tabelle_spie", hn, an, hg, ag, nratings, before, after)));
                    self.v.go(Page::Uebersicht);
                    st.stage = 3;
                }
            }
            3 => {
                if f % 30 == 0 {
                    let seen = &self.v.hud_seen;
                    let c = self.career.as_ref().unwrap();
                    let has_date = seen.iter().any(|s| s.contains(&date::long(c.world.season, c.world.day)));
                    let ok = has_date && seen.iter().any(|s| s == &c.club().name) && seen.iter().any(|s| s == "Weiter"); // roh
                    st.steps.push((crate::id("hud").into(), ok, crate::tf!("ui.main.kopfleiste_gezeichnet_datum_verein_weite", date::long(c.world.season, c.world.day), c.club().name, c.world.rank_of(c.own), c.world.leagues[c.league()].name)));
                    let steps: Vec<serde_json::Value> = st.steps.iter().map(|(s, p, d)| serde_json::json!({"step": s, "passed": p, "detail": d})).collect();
                    let out = serde_json::json!({"version": 1, "steps": steps});
                    let _ = std::fs::write(&st.path, serde_json::to_string_pretty(&out).unwrap());
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

#[macroquad::main(conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let shot = arg(crate::id("--screenshot"));
    // --vorschau: the preview career of PESM_WELT / PESM_VEREIN / PESM_PAGE, but the program keeps running (phone test)
    let vorschau = shot.clone().or(args.iter().any(|a| a == "--vorschau").then(String::new)); // roh
    if let Some(i) = args.iter().position(|a| a == crate::id("--neue-karriere")) {
        let g = |k: usize| args.get(i + k).cloned().unwrap_or_default();
        match altimport::neue_karriere(&g(1), &g(2), &g(3)) { Ok(t) => { println!("{t}"); std::process::exit(0) } Err(e) => { eprintln!("{e}"); std::process::exit(1) } }
    }
    // pesm --kalibrieren <seasons> [<league>]: the calibration report (Statistik Teil 9)
    if let Some(i) = args.iter().position(|a| a == "--kalibrieren") { // roh
        let n = args.get(i + 1).and_then(|x| x.parse().ok()).unwrap_or(1);
        println!("{}", kalibrieren::lauf(n, args.get(i + 2).map(|s| s.as_str())));
        std::process::exit(0);
    }
    // pesm --taktik-zeigen / --taktik-setzen / --formation-kopieren: probe tool for the in-game test (taktikprobe.rs)
    if let Some(r) = taktikprobe::kommando(&args) { match r { Ok(t) => { println!("{t}"); std::process::exit(0) } Err(e) => { eprintln!("{e}"); std::process::exit(1) } } }
    // pesm --import-alt <json> <alte-karriere> <welt>: a career of the Python version as a new career (altimport.rs)
    if let Some(i) = args.iter().position(|a| a == crate::id("--import-alt")) {
        let (j, alt, welt) = (args.get(i + 1).cloned().unwrap_or_default(), args.get(i + 2).cloned().unwrap_or_default(), args.get(i + 3).cloned().unwrap_or_default());
        match altimport::kommando(&j, &alt, &welt) { Ok(t) => { println!("{t}"); std::process::exit(0) } Err(e) => { eprintln!("{e}"); std::process::exit(1) } }
    }
    let selftest = arg(crate::id("--self-test"));
    // check runs never touch the real FL27 process or game folder (fl27::testmodus)
    if shot.is_some() || selftest.is_some() || args.iter().any(|a| a == crate::id("--start-test")) { std::env::set_var("PESM_TESTMODUS", "1"); }
    if arg(crate::id("--start-test")).is_some() {
        // never the person's careers: a fresh folder of its own, whatever the environment says
        let d = std::env::temp_dir().join(format!("pesm-starttest-{}", std::process::id())).join(crate::id("karrieren"));
        let _ = std::fs::create_dir_all(&d);
        std::env::set_var("PESM_KARRIEREN", &d);
    }
    let mut app = App::new().await;
    if shot.is_none() { update::pruefen(); }
    // Einstellungen → Sprache (sprache.txt); preview aid: PESM_SPRACHE=en|de overrides it (e.g. for --screenshot)
    inhalt::sprache_setzen(inhalt::sprache_aus(&std::env::var("PESM_SPRACHE").unwrap_or_else(|_| std::fs::read_to_string(sprache_file()).unwrap_or_default())));
    if (shot.is_some() && std::env::var("PESM_PAGE").as_deref() != Ok(crate::id("start"))) || selftest.is_some() { app.new_demo_career(); }
    // preview aid: PESM_PAGE=start PESM_KARRIEREN_BEISPIEL=1 shows the list with two example careers (in the preview's own
    // folder, never the person's), one with backups, and the question before deleting the second
    if shot.is_some() && std::env::var("PESM_KARRIEREN_BEISPIEL").is_ok() {
        for (k, verein) in [crate::id("Bayern"), crate::id("Dortmund")].iter().enumerate() {
            let w = App::builtin_world();
            let own = w.clubs.iter().position(|c| c.name.contains(verein)).unwrap_or(k);
            let mut c = Career::new(w, own, crate::inhalt::t("ui.main.fabian_trainer"));
            c.save_name = format!("{}{}", crate::id("Beispiel"), k + 1);
            for _ in 0..(3 - 2 * k) { let _ = c.save_mit_sicherung(); }
        }
        app.start.saves = game::list_saves();
        app.start.offen = app.start.saves.iter().find(|p| p.to_string_lossy().contains(crate::id("Beispiel1"))).cloned();
        app.start.loeschen = app.start.saves.iter().find(|p| p.to_string_lossy().contains(crate::id("Beispiel2"))).cloned();
    }
    // preview aid: PESM_PAGE=start PESM_SUCHE=<text> [PESM_WELT=<Weltordner>] shows the club step with that search
    if let (Some(_), Ok(q)) = (&shot, std::env::var("PESM_SUCHE")) {
        let welt = std::env::var("PESM_WELT").ok().map(std::path::PathBuf::from);
        app.template = Some(welt.as_ref().and_then(|w| World::from_pesdb(&w.join("common/etc/pesdb"), 7).ok()).unwrap_or_else(App::builtin_world));
        app.start.welt = welt;
        app.start.step = 1; app.start.such = q; app.start.such_focus = true;
        // PESM_SCHRITT=2|3: the coach step or the summary of a start without a club
        if let Ok(n) = std::env::var("PESM_SCHRITT") { app.start.step = n.parse().unwrap_or(2); app.start.ohne = true; app.start.coach = crate::id("Max Muster").into(); app.start.angebot_modus = 2; }
    }
    // preview aid: PESM_WELT=<Weltordner> plays the preview on a real world (own club: the strongest of its first league)
    if let (Some(_), Ok(wp), true) = (&vorschau, std::env::var("PESM_WELT"), std::env::var("PESM_PAGE").as_deref() != Ok(crate::id("start"))) {
        if let Ok(w) = World::from_pesdb(&std::path::Path::new(&wp).join("common/etc/pesdb"), 7) {
            // PESM_VEREIN=<Name>: that club (else the strongest of the first league)
            let own = std::env::var("PESM_VEREIN").ok().and_then(|n| w.clubs.iter().position(|c| c.name == n))
                .unwrap_or_else(|| w.leagues[0].teams.iter().copied().max_by(|a, b| w.clubs[*a].level.partial_cmp(&w.clubs[*b].level).unwrap()).unwrap_or(0));
            let mut c = Career::new(w, own, crate::inhalt::t("ui.main.fabian_trainer"));
            c.fl27_welt = wp.clone();
            // PESM_KOOP_SUCHE=1: a farm-team search runs (applications come during PESM_DAYS)
            if std::env::var("PESM_KOOP_SUCHE").is_ok() { let _ = c.koop_suche_starten(crate::farm::KoopSuche { bis: 0, nur_land: true, stufe_von: 1, stufe_bis: 2, plaetze: 3, max_betrag: c.koop_rahmen() }); }
            app.career = Some(c);
            app.screen = Screen::Game;
        }
    }
    // preview aid for checks: PESM_PAGE=kader|aufstellung|... opens that page, PESM_DAYS=n advances days first
    if let (Ok(pg), Some(_), true) = (std::env::var("PESM_PAGE"), &vorschau, app.career.is_some()) {
        // PESM_AUFTRAG=<group>: a player search of the first scout in the own league starts first (preview of assignments)
        if let (Ok(g), Some(c)) = (std::env::var("PESM_AUFTRAG"), app.career.as_mut()) {
            let x = crate::auftraege::Auftrag { art: crate::auftraege::SUCHE, scout: c.scouts[0].name.clone(), gruppe: g, alter_min: 17, alter_max: 30, liga: Some(c.league()), ..Default::default() };
            let _ = c.auftrag_anlegen(x, 14);
        }
        if let Ok(n) = std::env::var("PESM_DAYS").unwrap_or_default().parse::<u32>() {
            let c = app.career.as_mut().unwrap();
            for _ in 0..n { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); c.answer_press(Some(0)); } let _ = c.end_day(); }
        }
        // PESM_THEMA=1: the "per competition" theme on; PESM_BIS_SPIELTAG=1: on to the next matchday (not played)
        if std::env::var("PESM_THEMA").is_ok() { app.career.as_mut().unwrap().thema_wettbewerb = true; }
        if std::env::var("PESM_BIS_SPIELTAG").is_ok() { let c = app.career.as_mut().unwrap(); for _ in 0..14 { if c.today_match().is_some() { break; } let _ = c.end_day(); } }
        let p = match pg.as_str() { "posteingang" => Page::Posteingang, "kalender" => Page::Kalender, "awards" => Page::Awards, "privat" => Page::Privat, "uefa" => Page::Uefa, "kadermeldung" => Page::Kadermeldung, "thebest" => Page::TheBest, "ligapreise" => Page::LigaPreise, "kader" => Page::Kader, "aufstellung" => Page::Aufstellung, "training" => Page::Training, "medizin" => Page::Medizin, "praesidium" => Page::Praesidium, "finanzen" => Page::Finanzen, "trainer" => Page::Trainer, "transfermarkt" => Page::Transfermarkt, "verhandlungen" => Page::Verhandlungen, "vertraege" => Page::Vertraege, "scouting" => Page::Scouting, "akademie" => Page::Akademie, "tabellen" => Page::Tabellen, "presse" => Page::Presse, "spieltag" => Page::Spieltag, "kabine" => Page::Kabine, "optionen" => Page::Optionen, "ziele" => Page::Ziele, "manager" => Page::Manager, "stadion" => Page::Stadion, "personalmarkt" => Page::Personalmarkt, "geschichte" => Page::Geschichte, "tagebuch" => Page::Tagebuch, "klima" => Page::Klima, "marken" => Page::Marken, "kaderplanung" => Page::Kaderplanung, "vergleich" => Page::Vergleich, "entwicklung" => Page::Entwicklung, "vorbereitung" => Page::Vorbereitung, "analyse" => Page::Analyse, "zeitung" => Page::Zeitung, "fans" => Page::Fans, "ranglisten" => Page::Ranglisten, "vereine" => Page::Vereine, "vereinswelt" => Page::Vereinswelt, "portal" => Page::Portal, "handbuch" => Page::Handbuch, "regeln" => Page::Regeln, "profi" => Page::Profi, "spielerprofil" => Page::Spielerprofil, "zentrale" => Page::Zentrale, "kooperationen" => Page::Kooperationen, "zweite" => Page::ZweiteMannschaft, "jugendnetz" => Page::Jugendnetz, "merkliste" => Page::Merkliste, "trainerteam" => Page::Trainerteam, "nationalteams" => Page::Nationalteams, _ => Page::Uebersicht };
        app.v.go(p);
        if p == Page::Tabellen || p == Page::Ranglisten { app.v.table_league = app.career.as_ref().unwrap().world.cups.iter().position(|c| c.kind == 1).map(|i| 1000 + i); }
        // PESM_LIGA=eigene|<Liganame>: that league's table instead of the Champions League
        if let (true, Ok(n)) = (p == Page::Tabellen, std::env::var("PESM_LIGA")) { let c = app.career.as_ref().unwrap(); app.v.table_league = if n == crate::id("eigene") { Some(c.league()) } else { c.world.leagues.iter().position(|l| l.name == n).or_else(|| c.world.cups.iter().position(|x| x.name == n).map(|i| 1000 + i)) }; }
        if p == Page::Transfermarkt { app.v.sel_player = app.career.as_ref().unwrap().world.players.iter().position(|x| x.name.contains("Wirtz")); } // roh
        // PESM_SPIELER=<Name>: that player; PESM_REITER=<n>: that tab
        if let Ok(n) = std::env::var("PESM_SPIELER") { app.v.sel_player = app.career.as_ref().unwrap().world.players.iter().position(|x| x.name.contains(&n)); }
        // PESM_FEIER=1: a reward moment for the best own player (preview of the full-screen moment)
        // the moments the days collected stay away from other previews
        if std::env::var("PESM_FEIER").is_err() { app.career.as_mut().unwrap().feiern.clear(); }
        if std::env::var("PESM_AUSLOSUNG").is_err() { app.career.as_mut().unwrap().auslosungen.clear(); }
        if std::env::var("PESM_FEIER").is_ok() { let c = app.career.as_mut().unwrap(); let p = c.squad()[0]; c.feier_neuzugang(p, crate::id("Testverein"), 25_000_000.0); }
        // PESM_PRESSE=1: a press conference is waiting (preview of the scene)
        if std::env::var("PESM_PRESSE").is_ok() { let c = app.career.as_mut().unwrap(); let q = c.press_question(false); c.press = Some(q); }
        // PESM_KLUB=<Name>: that club on the clubs page
        if let Ok(n) = std::env::var("PESM_KLUB") { app.v.sel_club = app.career.as_ref().unwrap().world.clubs.iter().position(|x| x.name == n); }
        if let Ok(t) = std::env::var("PESM_REITER") { app.v.tab = t.parse().unwrap_or(0); }
        // PESM_RAD=1: the percentile wheel on the statistics tab
        if std::env::var("PESM_RAD").is_ok() { crate::pages::stat_rad_an(); }
        // PESM_STAT_FENSTER=<area>:<row>: the league ranking of that number open (needs PESM_SPIELER)
        if let (Ok(t), Some(p)) = (std::env::var("PESM_STAT_FENSTER"), app.v.sel_player) { if let Some((b, z)) = t.split_once(':') { crate::pages::stat_fenster_auf(p, b.parse().unwrap_or(0), z.parse().unwrap_or(0)); } }
        // PESM_AUSWAHL=<n>: that choice field open; PESM_LEISTE=zu|alle: side bar folded in or all groups open
        if let Ok(t) = std::env::var("PESM_AUSWAHL") { app.v.auswahl = t.parse().ok(); }
        match std::env::var("PESM_LEISTE").as_deref() { Ok("zu") => pages::SEITENLEISTE_ZU.store(true, std::sync::atomic::Ordering::Relaxed), Ok("alle") => app.v.menu_alle = true, _ => {} }
        // PESM_TP=<ready-made week>[:day,slot]: that training week, with a session chosen (preview of the plan)
        if let Ok(t) = std::env::var("PESM_TP") { let mut it = t.split(':'); let n = it.next().unwrap_or(""); if let Some(w) = crate::trainingsplan::werk().into_iter().find(|w| w.0 == n) { if let Some(c) = app.career.as_mut() { c.wochenplan = w.1; c.plan_uebernehmen(); } } if let Some((d, s)) = it.next().and_then(|x| x.split_once(',')) { app.v.tp_zelle = Some((d.parse().unwrap_or(0), s.parse().unwrap_or(0))); } }
        // PESM_SORT=<list>:<column>[:auf]: a sorted list (preview of the clickable header)
        if let Ok(s) = std::env::var("PESM_SORT") { let t: Vec<&str> = s.split(':').collect(); if let (Some(l), Some(k)) = (t.first().and_then(|x| x.parse().ok()), t.get(1).and_then(|x| x.parse().ok())) { app.v.sortierung.insert(l, (k, t.get(2) == Some(&crate::id("auf")))); } }
        // PESM_FOKUS=1: the three youngest own players get an individual programme with different focus groups (training preview)
        if std::env::var("PESM_FOKUS").is_ok() {
            let c = app.career.as_mut().unwrap();
            let mut jung = c.squad();
            jung.sort_by_key(|&p| c.world.players[p].age);
            for (k, p) in jung.into_iter().take(3).enumerate() { c.world.players[p].program = if k == 2 { 2 } else { 1 }; c.world.players[p].ent.fokus = k as u8 * 2; }
        }
        // PESM_BEISPIELE=1: one example message of every kind (previews of the inbox layouts)
        if std::env::var("PESM_BEISPIELE").is_ok() { app.career.as_mut().unwrap().beispiel_meldungen(); }
        // PESM_POOL=<thema>: a pool conversation of that theme with the first squad player, opened in the inbox
        if let Ok(th) = std::env::var("PESM_POOL") {
            let c = app.career.as_mut().unwrap();
            let p = c.squad()[0];
            if c.pool_starten(p, &th) { app.v.sel_msg = Some(0); app.v.go(Page::Posteingang); }
        }
        if let Ok(m) = std::env::var("PESM_MELDUNG") { app.v.sel_msg = m.parse().ok().or_else(|| app.career.as_ref().and_then(|c| c.inbox.iter().position(|x| x.title.contains(&m) || x.karte == m))); }
        // PESM_VERLETZT=<n>: n squad players injured (5, 12, 25, 40 … days), for the medical page
        if let Ok(n) = std::env::var("PESM_VERLETZT") {
            let c = app.career.as_mut().unwrap();
            for (k, p) in c.squad().into_iter().take(n.parse().unwrap_or(3)).enumerate() { let d = [5u16, 12, 25, 40, 18][k % 5]; c.world.players[p].inj = d; c.world.players[p].inj_name = crate::world::injury_names(d)[0].to_string(); c.world.players[p].ent.inj_gesamt = d + 6 * k as u16; }
        }
        // PESM_LAGER=1: the Vorbereitung page shows the training camps
        if std::env::var("PESM_LAGER").is_ok() { app.v.lager_reiter = 1; }
        // PESM_KONTEXT=<name>: the right-click menu of that player over the page
        if let Ok(n) = std::env::var("PESM_KONTEXT") {
            let c = app.career.as_ref().unwrap();
            if let Some(p) = c.world.players.iter().position(|x| x.name.contains(&n)) { app.v.kontext = Some((p, vec2(520.0, 360.0))); }
        }
        // PESM_SUCHFENSTER=<text>: the search over the page with that text (empty: the bookmarks; two are set)
        if let Ok(q) = std::env::var("PESM_SUCHFENSTER") {
            app.v.modal = pages::Modal::Suche;
            app.v.suche_text = q;
            app.v.suche_fuer = "-".into();
            let c = app.career.as_mut().unwrap();
            c.lesezeichen = vec![crate::id("Transfermarkt").into(), crate::id("Tabellen").into()];
        }
        // PESM_VERHANDLUNG=<name>: the contract talks with that own player open (clauses, Punkt 6); PESM_VERKAUF=1: a sale of him
        if let Ok(n) = std::env::var("PESM_VERHANDLUNG") {
            let c = app.career.as_mut().unwrap();
            // another club's player: the purchase talks (fee stage)
            if !c.squad().into_iter().any(|p| c.world.players[p].name.contains(&n)) {
                if let Some(p) = (0..c.world.players.len()).find(|&p| c.world.players[p].team >= 0 && c.world.players[p].name.contains(&n)) { match c.open_neg(p) { Ok(k) => app.v.modal = pages::Modal::Neg(k), Err(e) => eprintln!("open_neg: {e}") } }
            }
            if let Some(p) = c.squad().into_iter().find(|&p| c.world.players[p].name.contains(&n)) {
                let k = if std::env::var("PESM_VERKAUF").is_ok() {
                    let kaeufer = (0..c.world.clubs.len()).find(|&k| k != c.own && c.world.erste_von(k).is_none()).unwrap_or(0);
                    Some(c.verkauf_verhandeln(p, kaeufer, c.world.players[p].value))
                } else { c.world.players[p].contract_end = c.world.season; c.open_neg(p).ok() };
                if let Some(k) = k {
                    app.v.modal = pages::Modal::Neg(k);
                    if let Some(ng) = c.negs.get_mut(k) { ng.kl.ausstieg = 1; ng.kl.tor = 1; ng.kl.weiterverkauf = 2; }
                    // PESM_RUNDEN=n: that many offers sent already, each a bit lower than asked (chat preview)
                    let runden: usize = std::env::var("PESM_RUNDEN").ok().and_then(|x| x.parse().ok()).unwrap_or(0);
                    for r in 0..runden { if c.negs[k].state != game::NegState::Counter { break; } c.negs[k].wage = (c.negs[k].need * (0.7 + 0.08 * r as f64) / 5000.0).round() * 5000.0; c.negs[k].argument = (r % 6 + 1) as u8; c.submit_neg(k); }
                }
            }
        }
        // PESM_TICKER=<minute|ende>: the simulated match on the ticker at that minute, or after the final whistle
        if let Ok(t) = std::env::var("PESM_TICKER") {
            let c = app.career.as_mut().unwrap();
            for _ in 0..21 { if c.today_match().is_some() { break; } let _ = c.end_day(); }
            if c.today_match().is_some() {
                let res = c.prepare_match(None);
                let ende = t == crate::id("ende");
                let mut mv = pages::MatchView { res: res.clone(), minute: t.parse().unwrap_or(90.0), speed: if ende { 1.0 } else { 0.0 }, phase: if ende { 2 } else { 1 }, ended_at: 0.0, manual: (0, 0), manual_open: false, halbzeit: 2, wirkung: vec![], nach_gesagt: false, angesprochen: vec![], eingriff: Default::default(), vorlauf: 3, vor_pause: 1.0, duell: vec![], parallel: vec![] };

                mv.parallel = c.parallel_spielen();
                // PESM_NACH_REITER=1: the matchday view after the match
                mv.eingriff.nach_reiter = std::env::var("PESM_NACH_REITER").ok().and_then(|x| x.parse().ok()).unwrap_or(0);
                // PESM_LIVE_REITER=<0|1|2>: that menu on the right
                mv.eingriff.reiter = std::env::var("PESM_LIVE_REITER").ok().and_then(|x| x.parse().ok()).unwrap_or(0);
                if ende { c.apply_own_match(res); c.press = None; if let Some(r) = c.last_result.clone() { mv.res = r; } }
                app.v.mv = Some(mv);
                app.v.go(Page::Spieltag);
            }
        }
        // PESM_LIVE=1: the live view of a match in FL27 with made-up numbers (the next match day, minute 67)
        if std::env::var("PESM_LIVE").is_ok() {
            let c = app.career.as_mut().unwrap();
            for _ in 0..21 { if c.today_match().is_some() { break; } let _ = c.end_day(); }
            app.v.fl27_ansicht = crate::livespiel::beispiel(c);
            // PESM_LIVE=halbzeit: the same at the break (half-time talk)
            if std::env::var("PESM_LIVE").as_deref() == Ok(crate::id("halbzeit")) { if let Some(m) = app.v.fl27_ansicht.as_mut() { m.minute = 45; m.period = 2; } }
            app.v.fl27_vorschau = app.v.fl27_ansicht.is_some();
            app.v.fl27_seq = Some(7);
            app.v.go(Page::Spieltag);
        }
    }
    if let Some(p) = selftest { app.selftest = Some(SelfTest { path: p, frame: 0, day0: 0, steps: vec![], stage: 0 }); }
    if let Some(p) = arg(crate::id("--start-test")) { app.starttest = Some(StartTest { path: p, welt: arg(crate::id("--welt")), frame: 0, stage: 0, steps: vec![], team: String::new() }); }
    // the UI scale of the settings (skala.txt); PESM_SKALA for previews
    if let Some(s) = std::env::var("PESM_SKALA").ok().or_else(|| std::fs::read_to_string(pages::skala_file()).ok()).and_then(|x| x.trim().parse::<f32>().ok()) { crate::ui::ui_skala_setzen(s); }
    let mut frames = 0;
    loop {
        hoehe_setzen(screen_width(), screen_height());
        let k = view_scale();
        let mut cam = Camera2D::from_display_rect(Rect::new(-versatz(), 0.0, screen_width() * k, screen_height() * k));
        cam.zoom.y = -cam.zoom.y; // from_display_rect is made for render targets; the window needs y pointing down
        set_camera(&cam);
        app.frame();
        frames += 1;
        if app.selftest.is_some() && app.self_test_step() { break; }
        if app.starttest.is_some() && app.start_test_step() { break; }
        if let Some(p) = &shot {
            // the window-size check records the last frames only (the layout has settled)
            if frames == 110 && std::env::var_os("PESM_PRUEF").is_some() { crate::ui::pruefung_starten(); }
            // wait until the window manager gave the asked size (a tiling layout resizes it after the first frames)
            let soll = std::env::var("PESM_FENSTER").ok().and_then(|f| f.split('x').next().and_then(|w| w.parse::<f32>().ok())).unwrap_or(0.0);
            if frames >= 120 && ((screen_width() * screen_dpi_scale() - soll).abs() <= 2.0 || soll == 0.0 || frames >= 900) {
                let mut img = get_screen_data();
                for px in img.bytes.chunks_mut(4) { px[3] = 255; }
                img.export_png(p);
                if let Ok(b) = std::env::var("PESM_PRUEF") { let _ = std::fs::write(b, crate::ui::pruefung_bericht()); }
                break;
            }
        }
        next_frame().await;
    }
}
