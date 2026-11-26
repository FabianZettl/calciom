// The career around the world simulation: day steps, training, transfers and negotiations, scouting,
// academy, board, press, inbox, finances and the manager's own career. Numbers: see MECHANICS.md.
use crate::date;
use crate::fl27;
use crate::world::*;
use serde::{Deserialize, Serialize};

pub fn units_t() -> [&'static str; 6] { [crate::inhalt::t("en.einheit.frei"), crate::inhalt::t("en.einheit.regeneration"), crate::inhalt::t("en.einheit.leicht"), crate::inhalt::t("en.einheit.normal"), crate::inhalt::t("en.einheit.taktik"), crate::inhalt::t("en.einheit.intensiv")] }
/// per training day: (fitness change, injury factor, development factor, familiarity)
/// from training.json: free_day, focus Regeneration, intensity leicht/normal/hart, focus Taktik (weekly cost spread over 6 days)
pub fn unit_effect(u: u8) -> (f32, f32, f32, f32) {
    match u {
        0 => (0.5, 0.0, 0.99, -0.07),
        1 => (1.8 + 2.0 / 6.0, 0.4, 0.97, 0.5 / 6.0),
        2 => (1.0 - 1.0 / 6.0, 0.5, 0.94, 0.7 / 6.0),
        3 => (-4.0 / 6.0, 1.0, 1.0, 1.0 / 6.0),
        4 => (0.2 - 3.5 / 6.0, 0.8, 1.0, 5.0 / 6.0),
        _ => (-1.2 - 12.0 / 6.0, 2.5, 1.07, 1.1 / 6.0),
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Action {
    None,
    TransferOffer { pid: usize, club: usize, fee: f64 },
    Complaint { pid: usize },
    JobOffer { club: usize, wage: f64 },
    Graduate { pid: usize },
    Expiring { pid: usize },
    Report { pid: usize },
    /// A choice with options (key, words): conversations, conflicts, discipline (mensch::wahl).
    Wahl { art: String, bezug: Vec<usize>, optionen: Vec<(String, String)> },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Msg {
    /// The clubs a message is about (a transfer: from, to), for the context column and the links.
    #[serde(default)]
    pub vereine: Vec<usize>,
    /// Stable number of the message (the phone names messages by it, never by their place in the list).
    #[serde(default)]
    pub id: u64,
    pub day: u32,
    pub season: u32,
    pub cat: u8,
    pub from: String,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub action: Action,
    pub done: bool,
    /// Case rating (Paket 3m): 0 Info, 1 Soll, 2 Muss; level 0 background … 3 flash; deadline (abs day); thread key.
    #[serde(default)]
    pub pflicht: u8,
    #[serde(default)]
    pub stufe: u8,
    #[serde(default)]
    pub frist: u32,
    #[serde(default)]
    pub faden: String,
    /// Kind of the message for its layout (postkarten.rs Art::id) and the players it is about.
    #[serde(default)]
    pub art: String,
    #[serde(default)]
    pub spieler: Vec<usize>,
    /// A picture card for the message (postkarten.rs): „entwicklung“ (players with their change in `werte`), „ehrung“.
    #[serde(default)]
    pub karte: String,
    #[serde(default)]
    pub werte: Vec<f32>,
}
impl Msg {
    /// The family of a choice ("leihe" of "leihe:12:…"), None for other actions.
    pub fn action_art(&self) -> Option<&str> { if let Action::Wahl { art, .. } = &self.action { art.split(':').next() } else { None } }
}
pub fn cats_t() -> [&'static str; 7] { [crate::inhalt::t("menu.posteingang.reiter.6"), crate::inhalt::t("menu.posteingang.reiter.7"), crate::inhalt::t("bereich.transfers"), crate::inhalt::t("bereich.finanzen"), crate::inhalt::t("zentrale.k.medien"), crate::inhalt::t("klima.praesidium"), crate::inhalt::t("profil.teil.8")] }

/// Plan B: from `minute` on, when the score is `wenn` (0 behind, 1 not ahead, 2 ahead), play with `mentalitaet` (−1/0/1).
#[derive(Serialize, Deserialize, Clone)]
pub struct PlanB { pub an: bool, pub minute: u8, pub wenn: u8, pub mentalitaet: i8 }
impl Default for PlanB { fn default() -> Self { PlanB { an: false, minute: 70, wenn: 0, mentalitaet: 1 } } }

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Anbahnung { pub p: usize, pub von: usize, pub nach: usize, pub preis: f64, pub grund: String, pub fertig: u32, #[serde(default)] pub stufe: u8 }

#[derive(Serialize, Deserialize, Clone)]
pub struct News {
    pub day: u32,
    pub season: u32,
    pub paper: String,
    /// The outlet of the web blog (medien.toml [[webblog]] id); empty in old saves (then by the paper).
    #[serde(default)]
    pub medium: String,
    pub head: String,
    pub text: String,
}
pub const PAPERS: [&str; 5] = ["Tribünenblatt", "Abendkurier", "Flutlicht-Zeitung", "Morgenecho", "Sportbote am Sonntag"]; // roh

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum NegState { Waiting, Counter, Accepted, Rejected }

fn verein_unbekannt() -> i32 { i32::MIN }
#[derive(Serialize, Deserialize, Clone)]
pub struct Neg {
    pub pid: usize,
    pub renew: bool,
    pub stage: u8, // 0 club, 1 player, 2 done
    pub round: u8,
    pub ask: f64,
    pub fee: f64,
    pub counter: f64,
    pub wage: f64,
    pub need: f64,
    pub years: u32,
    pub state: NegState,
    pub reply_day: u32,
    pub ends: u32,
    pub text: String,
    /// The round model of the current phase (club, then player; verhandlung.rs).
    #[serde(default)]
    pub ph: Option<crate::verhandlung::Phase>,
    #[serde(default)]
    pub ton: u8,
    #[serde(default)]
    pub handlung: u8,
    /// The next offer is the last one (take it or leave it).
    #[serde(default)]
    pub letzt: bool,
    /// The offer lies on the table until this abs day (negotiation pace, Fabian 23.11.); 0 = nothing pending.
    #[serde(default)]
    pub liegt_bis: u32,
    /// The player's club when the talks began: if he moves elsewhere meanwhile, the talks end (Fabian 24.11.).
    #[serde(default = "verein_unbekannt")]
    pub verein: i32,
    /// Talks about a pre-contract: no fee, the move happens at the season's change (termine.rs).
    #[serde(default)]
    pub vorvertrag: bool,
    /// The argument brought with the next offer (berater.rs ARGUMENTE, 0 none) and those used already.
    #[serde(default)]
    pub argument: u8,
    #[serde(default)]
    pub argumente: Vec<u8>,
    /// The question „what matters most to him?“ was asked in this talk (Konzept 2.6: costs patience, brings a hint).
    #[serde(default)]
    pub gefragt: bool,
    /// More fee for the agent, a smaller wage for the player (Konzept 2.7); refused by a family adviser.
    #[serde(default)]
    pub tausch: bool,
    /// Open drama: 0 none, 1 medical, 2 last-minute demand, 3 rival; until abs day haken_bis.
    #[serde(default)]
    pub haken: u8,
    #[serde(default)]
    pub haken_bis: u32,
    #[serde(default)]
    pub abschlag: f64,
    #[serde(default)]
    pub med: bool,
    #[serde(default)]
    pub gate: bool,
    /// A sale of an own player to `kaeufer` (values are negated in the round model: runden.py adapter).
    #[serde(default)]
    pub verkauf: bool,
    #[serde(default)]
    pub kaeufer: usize,
    /// The clauses offered (klauseln.rs): release, bonuses, raise for the player; sell-on and buy-back in a sale.
    #[serde(default)]
    pub kl: crate::klauseln::Wahl,
    /// Key of the last proposal: sending the same again gives the same answer and changes nothing.
    #[serde(default)]
    pub gesendet: String,
    /// Times the person asked for more time (at most 2, 2 days each).
    #[serde(default)]
    pub zeit: u8,
    /// The talk as a chat (Fabian 23.11.): (0 we, 1 the other side, 2 a note), oldest first.
    #[serde(default)]
    pub verlauf: Vec<(u8, String)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Scout {
    pub name: String,
    pub q: u8,
    pub league: Option<usize>,
    pub pos: u8,
    pub watch: Option<usize>,
    pub seen: u32,
    /// The scout as a person (Paket 3g); filled on first use for scouts of older saves.
    #[serde(default)]
    pub p: Option<crate::scouting::Person>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct YRow { pub name: String, pub own: bool, pub str_: f32, pub p: u32, pub w: u32, pub d: u32, pub l: u32, pub gf: i32, pub ga: i32, pub pts: i32,
    /// The rival academy's real players (FC-26 wish, 23.11.): scoutable and signable; and the club they belong to.
    #[serde(default)] pub kader: Vec<YSpieler>, #[serde(default)] pub verein: usize }
/// A player of a rival academy (not part of the world until he is signed).
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct YSpieler { pub name: String, pub pos: String, pub age: u8, pub ovr: f32, pub pot: f32, pub tore: u16, pub spiele: u16 }
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct YouthLeague { pub name: String, pub rows: Vec<YRow>, pub round: u32, pub last: String }

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Tactic {
    pub angriff: u8,
    pub aufbau: u8,
    pub breite: u8,
    pub abwehr: u8,
    pub pressing: u8,
    pub linie: i32,
    pub kompakt: i32,
    pub haltung: u8,
    /// The game's remaining style switches (gegner.toml [schalter]): positioning 0 hold the formation, 1 flexible;
    /// control area 0 centre, 1 wide.
    #[serde(default)]
    pub stellung: u8,
    #[serde(default)]
    pub kontrolle: u8,
}
impl Tactic {
    pub fn style(&self) -> i8 { self.haltung as i8 - 1 }
    pub fn idea(&self) -> usize {
        if self.linie <= 3 && self.abwehr == 1 { 3 } else if self.pressing == 0 && self.abwehr == 0 { 1 } else if self.angriff == 0 { 2 } else { 0 }
    }
}
pub fn ideas_t() -> [&'static str; 4] { [crate::inhalt::t("tm.stil.ballbesitz.name"), crate::inhalt::t("ui.game.pressingspiel"), crate::inhalt::t("gb.training.konter"), crate::inhalt::t("ui.game.abwehrspiel")] }
/// psyche.json idea_pos
pub fn idea_fit(pos: &str, idea: usize) -> f32 {
    let v: [f32; 4] = match pos {
        "GK" => [0.0, 0.0, 0.0, 0.0], "CB" => [0.3, 0.0, 0.3, 1.0], "LB" | "RB" => [0.0, 0.7, 0.5, 0.3], "DMF" => [0.5, 0.6, 0.0, 0.8],
        "CMF" => [1.0, 0.7, -0.2, 0.0], "AMF" => [1.0, 0.2, -0.3, -0.8], "LMF" | "RMF" => [0.3, 0.7, 0.6, 0.0], "LWF" | "RWF" => [0.2, 0.6, 1.0, -0.7],
        "SS" => [0.4, 0.5, 0.7, -0.8], _ => [0.0, 0.4, 0.8, -0.5],
    };
    v[idea]
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PressQ {
    pub q: String,
    pub answers: Vec<(String, u8)>,
    pub post: bool,
    /// Topic of the question (medien.toml [toene]), the asking journalist, the player it is about (Paket 3l).
    #[serde(default)]
    pub thema: String,
    #[serde(default)]
    pub journalist: u8,
    #[serde(default)]
    pub bezug: Option<usize>,
    /// The questions still to come, and the summed effects so far (media, board, fans, morale).
    #[serde(default)]
    pub rest: Vec<PressQ>,
    #[serde(default)]
    pub summe: [f32; 4],
}
/// medien.toml tones: (media image, board, fans, morale)
pub fn tone_effect(t: u8) -> (f32, f32, f32, f32) {
    match t { 0 => (2., 1., 0., 0.), 1 => (0., 0., 2., 1.), 2 => (-2., 0., -1., 0.), 3 => (1., 0., 1., 3.), 4 => (1., 1., 0., -3.), 5 => (1., 1., 0., 0.), _ => (1., -1., 2., 0.) }
}

/// The switchable features (Einstellungen › Features): id and text key; off means no messages, popups or effects.
pub const FEATURES: [&str; 7] = ["privat", "sponsoren", "medien", "feiern", "skilltree", "gespraeche", "manager"]; // roh
impl Career {
    pub fn feature(&self, f: &str) -> bool { !self.aus.iter().any(|a| a == f) }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Career {
    pub world: World,
    pub own: usize,
    pub coach: String,
    pub save_name: String,
    pub formation: usize,
    /// Freely moved spots of the own eleven on the tactics board (Fabian 22.11.), empty = the formation's spots.
    #[serde(default)]
    pub frei: Vec<(f32, f32)>,
    pub xi: Vec<usize>,
    pub tactic: Tactic,
    /// Transfer fees still to be paid or received in instalments, and conditional add-ons (klauseln.rs).
    #[serde(default)]
    pub raten: Vec<crate::klauseln::Rate>,
    #[serde(default)]
    pub boni: Vec<crate::klauseln::Bonus>,
    /// The PES M tactic behind it: playing idea, nine sliders, four switches (taktik9.rs); translated into `tactic`.
    #[serde(default)]
    pub t9: crate::taktik9::Taktik9,
    /// The formation with the ball and against it (Fabian 24.11., like The Gaffer's three formations: kick-off, in
    /// possession, out of possession); None = the base formation. Index into FORMATIONS.
    #[serde(default)]
    pub formation_ball: Option<usize>,
    #[serde(default)]
    pub formation_gegen: Option<usize>,
    /// Players placed freely in the formation with and against the ball (Discord 24.11.: they could not be moved).
    #[serde(default)]
    pub frei_ball: Vec<(f32, f32)>,
    #[serde(default)]
    pub frei_gegen: Vec<(f32, f32)>,
    /// The advanced instructions: two in attack and two in defence (ids as in the game's list, 0 none; taktik9.rs).
    #[serde(default)]
    pub anweisungen: [u8; 4],
    pub plan: [u8; 7],
    pub co_lineup: bool,
    /// The training week: seven days, three sessions each (trainingsplan.rs).
    #[serde(default)]
    pub wochenplan: crate::trainingsplan::Woche,
    /// This week: gains of the own players in the trained abilities, and injuries in training (for the training report).
    #[serde(skip)]
    pub tr_gewinn: Vec<(usize, f32)>,
    #[serde(default)]
    pub tr_verletzt: u32,
    /// How the last half-time talk landed (for the second half of a simulated match).
    #[serde(skip)]
    pub hz_wirkung: f32,
    /// Deadline day: the hours already played (0–12); reset at the end of the day.
    #[serde(default)]
    pub dl_stunde: u8,
    /// Deals between AI clubs in progress (kimarkt.rs): they take a few days, with rumours.
    #[serde(default)]
    pub anbahnung: Vec<Anbahnung>,
    /// The club names whose news concern us (nachricht_relevant), per season, club and cup count.
    #[serde(skip)]
    pub rel_cache: ((u32, usize, usize), Vec<String>),
    /// The board's three people, the club they belong to, and this season's side goals (praesidium.rs).
    #[serde(default)]
    pub praesidium: Vec<crate::praesidium::Person>,
    #[serde(default)]
    pub praesidium_verein: u32,
    #[serde(default)]
    pub nebenziele: Vec<crate::praesidium::Nebenziel>,
    /// The last message number given out (Msg.id).
    #[serde(default)]
    pub msg_id: u64,
    /// Plan B for simulated matches (Taktik-Konzept 1.7).
    #[serde(default)]
    pub plan_b: PlanB,
    /// The season (+1) of the last scouting tournament (jugend.rs).
    #[serde(default)]
    pub sichtung_saison: u32,
    pub co_training: bool,
    pub co_press: bool,
    pub co_name: String,
    pub inbox: Vec<Msg>,
    pub feed: Vec<News>,
    /// Players on the watch list (Beobachten on the profile page).
    #[serde(default)]
    pub merkliste: Vec<usize>,
    /// Konzept Scouting 1.1: per player id the abs day of the last scout look and whether he was ever known exactly.
    #[serde(default)]
    pub wissen_info: std::collections::HashMap<u32, (u32, bool)>,
    /// Scouting assignments (auftraege.rs) and the last id given.
    #[serde(default)]
    pub auftraege: Vec<crate::auftraege::Auftrag>,
    #[serde(default)]
    pub auftrag_id: u32,
    /// The analyst's latest "Auffällige Zahlen" (Konzept Scouting 5.2).
    #[serde(default)]
    pub analyst_liste: Vec<usize>,
    /// Talents on trial (Konzept Akademie 3.2): player, abs day it ends.
    #[serde(default)]
    pub probetrainings: Vec<(usize, u32)>,
    /// The co-trainer's line-up criteria as bits: 0 strength, 1 morale, 2 fitness, 3 form of the last 5, 4 rotation,
    /// 5 few appearances, 6 young players (Fabian 23.11.: chosen by the person, one or several). Default strength.
    #[serde(default = "kriterien_start")]
    pub aufstellung_kriterien: u8,
    /// The last discipline incident of the squad: abs day and kind (one club-wide quiet time, no kind twice in a row).
    #[serde(default)]
    pub vorfall_zuletzt: (u32, String),
    /// Autosave (Fabian 23.11.): 0 weekly on Mondays, 1 every day, 2 once a month; manual saving always works.
    #[serde(default)]
    pub autosave: u8,
    /// Features switched off in Einstellungen › Features (Fabian 25.11.): ids from FEATURES; empty = all on, so old saves keep everything.
    #[serde(default)]
    pub aus: Vec<String>,
    /// School of own players up to 18 by id: 0 good, 1 wobbling, 2 broken off, 3 finished (Konzept Akademie 4.3).
    #[serde(default)]
    pub schule: std::collections::HashMap<u32, u8>,
    /// Weekly morale of the own players by player id (the last 12 weeks, for the talk layout).
    #[serde(default)]
    pub moral_verlauf: std::collections::HashMap<u32, Vec<f32>>,
    /// Farm-team search and the open applications (farm.rs).
    #[serde(default)]
    pub koop_suche: Option<crate::farm::KoopSuche>,
    #[serde(default)]
    pub koop_angebote: Vec<crate::farm::KoopAngebot>,
    /// Players a partner offered us first (Vorkaufsrecht) and the abs day our option ends (0: we passed).
    #[serde(default)]
    pub vorkauf: Vec<(usize, u32)>,
    /// World transfers posted to the inbox today: (abs day, count) (kimarkt.rs welt_bezug, at most je_tag a day).
    #[serde(default)]
    pub welt_post: (u32, u8),
    /// Saved search profiles (suchprofil.rs).
    #[serde(default)]
    pub suchprofile: Vec<crate::suchprofil::Suchprofil>,
    /// Agreed moves at the season's change (termine.rs).
    #[serde(default)]
    pub vorvertraege: Vec<crate::termine::Vorvertrag>,
    /// Sounding out other clubs' players (sondierung.rs).
    #[serde(default)]
    pub sondierungen: Vec<crate::sondierung::Sondierung>,
    /// The team talk before today's match (prematch.rs).
    #[serde(default)]
    pub ansprache_vor: Option<crate::prematch::Vorgabe>,
    /// The other clubs' moves of today (kimarkt.rs, the deadline-day ticker).
    #[serde(default)]
    pub welt_tag: Vec<(usize, usize, usize, f64)>,
    /// Today's quiet moves of the summary leagues (transferki.rs), for tests and the world ticker.
    #[serde(skip)]
    pub strom_log: Vec<(usize, usize, usize, f64)>,
    /// Contract promises with a condition (vertragsanfrage.rs).
    #[serde(default)]
    pub bedingungen: Vec<crate::vertragsanfrage::Bedingung>,
    /// Stories, reputation and memory of the own club (geschichten.rs).
    #[serde(default)]
    pub geschichten: crate::geschichten::Geschichten,
    /// Bank loans (kredit.rs).
    #[serde(default)]
    pub kredite: Vec<crate::kredit::Kredit>,
    /// The state before the last day step or own match, for Undo (never saved, never nested).
    #[serde(skip)]
    pub undo: Option<Box<Career>>,
    /// Transfers agreed outside a window, carried out when the next one opens.
    #[serde(default)]
    pub geplant: Vec<Neg>,
    /// Talks planned in the calendar (Fabian 25.11.): (abs day, player, 0 talk / 1 contract talk); they reach the inbox on the day.
    #[serde(default)]
    pub gespraech_termine: Vec<(u32, usize, u8)>,
    /// The coach as created at the start, and the leagues offers may come from (empty: all; trainerprofil.rs).
    #[serde(default)]
    pub trainerprofil: crate::trainerprofil::Trainerprofil,
    #[serde(default)]
    pub angebot_ligen: Vec<usize>,
    /// The crisis meeting of this season and its plan (krise.rs).
    #[serde(default)]
    pub krise: crate::krise::Krise,
    /// A new stadium being built or open (neubau.rs).
    #[serde(default)]
    pub neubau: crate::neubau::Neubau,
    /// The experts' predicted place of this season (prognose.rs).
    #[serde(default)]
    pub experten_platz: Option<(u32, usize)>,
    pub negs: Vec<Neg>,
    pub scouts: Vec<Scout>,
    pub ticket: u8,
    pub merch: u8,
    pub fans: f32,
    pub media: f32,
    pub warned: bool,
    pub ultimatum: Option<u32>,
    /// The ultimatum's points target: the results count when it was set (the next matches are counted from there).
    #[serde(default)]
    pub ultimatum_ab: Option<usize>,
    /// Until this abs day bigger clubs take a closer look after a success (Fabian 23.11.).
    #[serde(default)]
    pub lockruf_bis: u32,
    /// Requests to the board this career: (abs day, kind 0 transfer / 1 wages / 2 facilities, share granted 0, 0.5, 1).
    #[serde(default)]
    pub bitten: Vec<(u32, u8, f32)>,
    /// Wage room the board granted on top of the usual share.
    #[serde(default)]
    pub gehalt_plus: f64,
    /// Quiet start at a new club (Fabian 23.11.): until this abs day no incidents, talks or board meetings.
    #[serde(default)]
    pub ruhe_bis: u32,
    /// Negotiation pace (Einstellungen): 0 instant, 1 brisk (2 days), 2 realistic (2–4 days).
    #[serde(default)]
    pub verhandlungstempo: u8,
    /// The infrastructure pot (None in older saves: started from the revenue share on first use).
    #[serde(default)]
    pub infra_topf: Option<f64>,
    /// The award gala of the autumn (Fabian 23.11.): (category key, player, club) chosen at the season's end.
    #[serde(default)]
    pub gala: Vec<(String, usize, usize)>,
    /// Awards given (preise.rs) and the nominee lists of the coming award nights, with the season they are for.
    #[serde(default)]
    pub preise: Vec<crate::preise::Preis>,
    #[serde(default)]
    pub preis_kand: Vec<(String, Vec<(usize, f32)>)>,
    #[serde(default)]
    pub preis_kand_saison: u32,
    /// Theme "per competition" (Fabian 24.11.): on a matchday the UI takes the competition's colours.
    #[serde(default)]
    pub thema_wettbewerb: bool,
    /// Private life (privat.rs).
    #[serde(default)]
    pub privat: crate::privat::Privat,
    /// Squad registrations for the European competitions (meldung.rs).
    #[serde(default)]
    pub meldungen: Vec<crate::meldung::Meldung>,
    /// Write PES M's fitness into start_stamina.txt for BlankState222's start_stamina module (setting, off by default).
    #[serde(default)]
    pub start_stamina: bool,
    /// Direct start into the match (Fabian 26.11.: switchable in Einstellungen › Football Life 2027); off = FL27 opens on
    /// the main menu as before. Off by default while the direct start is new.
    #[serde(default)]
    pub direktstart: bool,
    /// Real match length in FL27 for the direct start (Fabian 26.11.: set before the match); 0 = as set in the game.
    #[serde(default)]
    pub spielminuten: u8,
    /// Switches (FC-26 wish): no pre-contracts and no release clauses by AI clubs for our players.
    #[serde(default)]
    pub ohne_ki_vorvertraege: bool,
    #[serde(default)]
    pub ohne_ki_klauseln: bool,
    /// Fan boycott of the home end until this abs day (zufall_ereignis.rs).
    #[serde(default)]
    pub boykott_bis: u32,
    /// Market-value era in percent (0 = 100, the game's scale; 220 "modern").
    #[serde(default)]
    pub marktwert_aera: u32,
    /// Bought data packages (Statistik 8.1): (league, season) – extended numbers of that league for the season.
    #[serde(default)]
    pub datenpakete: Vec<(usize, u32)>,
    /// Knock-out draws to show as a ceremony (cup index, stage) and the ones already shown (Fabian 23.11.).
    #[serde(default)]
    pub auslosungen: Vec<(usize, u8)>,
    #[serde(default)]
    pub auslosung_gesehen: Vec<(String, u32, u8)>,
    pub fired: bool,
    pub mgr_rep: f32,
    pub mgr_wage: f64,
    pub mgr_money: f64,
    pub mgr_since: u32,
    pub book: Vec<(String, f64)>,
    pub balance_hist: Vec<f64>,
    pub youth: [YouthLeague; 2],
    pub history: Vec<String>,
    pub transfer_budget: f64,
    pub last_result: Option<MatchResult>,
    pub press: Option<PressQ>,
    pub hard_streak: u8,
    pub results: Vec<String>,
    #[serde(default)]
    pub fl27_erlaubt: bool,
    #[serde(default)]
    pub fl27_bestaetigt: bool,
    #[serde(default)]
    pub fl27_spiele: Vec<String>,
    #[serde(default)]
    pub fl27_cpk: String,
    /// Live writing of the pesdb (live.rs): switch (on by default), last write, auto end of FL27 after the final whistle.
    #[serde(default = "an")]
    pub db_live: bool,
    /// Einstellungen: the theme accent comes from the own crest (true, default) or is the standard blue.
    #[serde(default = "an")]
    pub akzent_wappen: bool,
    /// Einstellungen: squad rules for foreigners (kaderregeln.toml) apply (default on).
    #[serde(default = "an")]
    pub kaderregeln: bool,
    /// Milestones reached (id, abs day), own titles already celebrated, reward moments waiting to be shown.
    #[serde(default)]
    pub meilensteine: Vec<(String, u32)>,
    /// Player milestones reached at the club (player id in the database, milestone id, abs day); `None` until the first
    /// check has marked what was reached before the career (Fabian 24.11.).
    #[serde(default)]
    pub spieler_ms: Option<Vec<(u32, String, u32)>>,
    #[serde(default)]
    pub titel_gefeiert: usize,
    #[serde(skip)]
    pub feiern: Vec<crate::meilensteine::Feier>,
    /// AI market (kimarkt.rs): status and reason of the players of the detailed AI clubs, reviewed weekly.
    #[serde(default)]
    pub marktstatus: Vec<crate::kimarkt::MarktStatus>,
    /// Our relation with the advisers by name (−3 … +3, berater.rs).
    #[serde(default)]
    pub berater_bez: Vec<(String, f32)>,
    #[serde(default)]
    pub db_zuletzt: String,
    #[serde(default)]
    pub fl27_auto_ende: bool,
    /// The presets PES M wrote for the last match start (file in SiderAddons/flw, SHA-256); undone after the match.
    #[serde(default)]
    pub vorgaben_gesetzt: Vec<(String, String)>,
    /// After the match also switch the career database off in sider.ini (until the next match start).
    #[serde(default)]
    pub db_nach_spiel_aus: bool,
    /// The last talk's effect for the card „So hat es gewirkt“ (not saved).
    #[serde(skip)]
    pub wirkung: Option<crate::mensch::Wirkung>,
    /// Answers in pool conversations (player id, abs day): after three in seven days the mood effects stop (pool.rs).
    #[serde(default)]
    pub pool_antworten: Vec<(u32, u32)>,
    /// Days a pool theme of the "noise" level started (drosselung.toml: at most one a week).
    #[serde(default)]
    pub pool_rauschen: Vec<u32>,
    /// When each pool theme last started by itself (theme, abs day): the same theme waits its lock time squad-wide.
    #[serde(default)]
    pub pool_zuletzt: Vec<(String, u32)>,
    /// Openings shown last per theme and player ("theme|player id" → ids), for the repeat protection.
    #[serde(default)]
    pub pool_gezeigt: Vec<(String, Vec<String>)>,
    /// Chains in progress (chain id, player id, step waiting for its talk) and scheduled follow-up talks (theme, player id, abs day).
    #[serde(default)]
    pub pool_ketten: Vec<(String, u32, usize)>,
    #[serde(default)]
    pub pool_folgen: Vec<(String, u32, u32)>,
    /// Guided first week (spielgefuehl.rs): steps done (squad, talk, line-up, scout, match day), switched off, from which day.
    #[serde(default)]
    pub erste_woche: [bool; 5],
    #[serde(default)]
    pub erste_woche_aus: bool,
    #[serde(default)]
    pub erste_woche_ab: u32,
    /// Weekly history of the club (spielgefuehl.rs, Teil 3).
    #[serde(default)]
    pub verlauf: Vec<crate::spielgefuehl::VerlaufPunkt>,
    /// Own matches, own players' season per competition, own players' grades this season (Teil 3).
    #[serde(default)]
    pub spiele: Vec<crate::spielgefuehl::SpielZeile>,
    #[serde(default)]
    pub spieler_saison: Vec<crate::spielgefuehl::SpielerWb>,
    #[serde(default)]
    pub noten_verlauf: std::collections::HashMap<u32, Vec<f32>>,
    /// Awards over the year (ehrungen.rs), picks for the team of the week this season (player id, count), the last team.
    #[serde(default)]
    pub ehrungen: Vec<crate::ehrungen::Ehrung>,
    #[serde(default)]
    pub elf_der_woche: Vec<(u32, u32)>,
    #[serde(default)]
    pub letzte_elf: Vec<u32>,
    /// Bookmarked pages (their names in the code, Punkt 6).
    #[serde(default)]
    pub lesezeichen: Vec<String>,
    /// The other clubs' players got their release clauses (klauseln.rs, once per career, also for older saves).
    #[serde(default)]
    pub klauseln_gesaet: bool,
    /// Preparation (vorbereitung.rs): friendlies and trips, team events held (id, abs day), the events' team spirit points.
    #[serde(default)]
    pub termine: Vec<crate::vorbereitung::Termin>,
    #[serde(default)]
    pub teamevents: Vec<(String, u32)>,
    #[serde(default)]
    pub event_punkte: Vec<(u32, f32)>,
    #[serde(skip)]
    pub db_abdruck: u64,
    #[serde(skip)]
    pub db_unruhe: Option<std::time::Instant>,
    #[serde(skip)]
    pub db_schreibvorgaenge: usize,
    #[serde(skip)]
    pub db_hinweis: bool,
    #[serde(skip)]
    pub fl27_letzte: (usize, usize, usize),
    /// What the last database write changed for the running game (livepatch.rs): attributes per player id, teams whose
    /// squad order/numbers/formation changed with the same players, and whether players were added or moved (restart).
    #[serde(skip)]
    pub live_aenderung: (Vec<(u32, Vec<(usize, u8)>)>, Vec<u32>, bool),
    /// The last "Match starten" went to an FL27 that was open already (LivePatch): the instructions say F7 first.
    #[serde(skip)]
    pub fl27_offen: bool,
    /// The FL world folder the career was started from (absolute); empty = the default world.
    #[serde(default)]
    pub fl27_welt: String,
    /// The people side: board confidence, team spirit, promises, relations, reasons (Paket 3).
    #[serde(default)]
    pub mensch: crate::mensch::Mensch,
    /// Finances of the own club: facilities, sponsors (Paket 3j).
    #[serde(default)]
    pub wi: crate::wirtschaft::Wirtschaft,
    /// Fans and media: journalists, public promises, rumours, fan actions (Paket 3k/3l).
    #[serde(default)]
    pub me: crate::medien::Medien,
    /// The staff: trainers, assistants, doctor, physio, youth leader (Paket 3o).
    #[serde(default)]
    pub stab: Vec<crate::personal::Stab>,
    /// Set pieces, captain and saved tactics (Paket 4).
    #[serde(default)]
    pub std: crate::taktik::Standards,
    /// The manager as a person, youth network, applications (rest of Paket 3).
    #[serde(default)]
    pub mgr: crate::rest3::Manager,
    /// The manager's Skill Tree (Coaching Points, skilltree.rs).
    #[serde(default)]
    pub mst: crate::skilltree::Baum,
}

/// Where an own match takes place: a league (index) or a cup/European competition (index into World::cups).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Wo { Liga(usize), Pokal(usize) }

fn an() -> bool { true }
pub fn abs_day(season: u32, day: u32) -> u32 { season * date::SEASON_DAYS + day }

/// Bring PlayerAssignment.bin in line with the world: every player of the database at his club (a move between first
/// and second team, a loan or a transfer is an ordinary transfer entry). Returns the number of changes.
/// `spielklubs`: the clubs of the coming match; their injured and suspended players leave the FL27 squad (Fabian 23.11.:
/// not available means not selectable) and come back with their shirt number at the next write once they are fit.
pub fn asg_abgleichen(w: &World, a: &mut Vec<fl27::Asg>, known: &std::collections::HashSet<u32>, spielklubs: &[usize]) -> Result<usize, String> {
    let clubs: std::collections::HashSet<u32> = w.clubs.iter().map(|c| c.id).collect();
    let mut club_of: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for x in a.iter() { if clubs.contains(&x.team) { club_of.entry(x.pid).or_insert(x.team); } }
    let mut n = 0;
    for p in &w.players {
        if !known.contains(&p.id) { continue; }
        // only first-team players belong to a club squad in FL27; youth players have their record but no club
        let fehlt = p.team >= 0 && spielklubs.contains(&(p.team as usize)) && !p.available();
        let want = if p.team >= 0 && p.youth == 0 && !fehlt { w.clubs[p.team as usize].id } else { 0 };
        let have = club_of.get(&p.id).copied().unwrap_or(0);
        if want != have { fl27::transfer(a, p.id, have, want, p.no as u32)?; n += 1; }
    }
    // a shirt number changed in PES M (Fabian 24.11.: in the profile during a window); twice, so two players can swap
    for _ in 0..2 {
        for p in w.players.iter().filter(|p| p.no > 0 && p.team >= 0 && known.contains(&p.id)) {
            let t = w.clubs[p.team as usize].id;
            let Some(i) = a.iter().position(|x| x.pid == p.id && x.team == t) else { continue };
            if a[i].nummer() == p.no as u32 || a.iter().any(|x| x.team == t && x.nummer() == p.no as u32) { continue; }
            a[i].k = (a[i].k & !0x7f) | (p.no as u32 - 1);
            n += 1;
        }
    }
    Ok(n)
}

impl Career {
    pub fn new(mut world: World, own: usize, coach: &str) -> Career {
        // real potentials, wages, values, contracts and loans, once per career (tests keep the world's own values)
        let sofifa = if cfg!(test) || !world.players.iter().any(|p| p.db) { None } else { crate::sofifa::laden_aus(&crate::sofifa::ordner()).map(|(q, r)| world.sofifa_uebernehmen(&q, &r)) };
        let name = world.clubs[own].name.clone();
        let save_name = format!("PESM{}01", name.chars().filter(|c| c.is_alphanumeric()).take(16).collect::<String>());
        let rev = world.revenue(own);
        let big: Vec<usize> = world.leagues.iter().enumerate().filter(|(_, l)| ["ENGLAND_D1_LEAGUE", "SPAIN_D1_LEAGUE", "GERMANY_D1_LEAGUE", "ITALY_D1_LEAGUE", "FRANCE_D1_LEAGUE", "PORTUGAL_D1_LEAGUE"].contains(&l.key.as_str())).map(|(i, _)| i).collect();
        let own_league = world.clubs[own].league;
        for p in world.players.iter_mut() {
            if p.team < 0 { continue; }
            if p.team == own as i32 { p.know = 1.0; continue; }
            let lg = world.clubs[p.team as usize].league;
            // markt.base_know 0.35 for famous players and top leagues, staff base_know 0.25; own league better known (estimated 0.55)
            p.know = if lg == own_league { 0.55 } else if p.ovr >= 80.0 || big.contains(&lg) { 0.35 } else { 0.1 };
        }
        let mut c = Career {
            own, coach: coach.to_string(), save_name, formation: 0, frei: vec![], xi: vec![],
            tactic: Tactic { angriff: 0, aufbau: 0, breite: 0, abwehr: 0, pressing: 0, linie: 5, kompakt: 5, haltung: 1, stellung: 0, kontrolle: 0 },
            t9: Default::default(), formation_ball: None, formation_gegen: None, frei_ball: vec![], frei_gegen: vec![], anweisungen: [0; 4], raten: vec![], boni: vec![],
            plan: [3, 3, 2, 3, 4, 0, 0], co_lineup: false, co_training: false, co_press: false, wissen_info: Default::default(), auftraege: vec![], auftrag_id: 0, analyst_liste: vec![], probetrainings: vec![], aufstellung_kriterien: 1, vorfall_zuletzt: Default::default(), autosave: 0, aus: vec![], schule: Default::default(), sichtung_saison: 0, plan_b: Default::default(), msg_id: 0, praesidium: vec![], praesidium_verein: 0, nebenziele: vec![], rel_cache: Default::default(), anbahnung: vec![], dl_stunde: 0, hz_wirkung: 0.0, wochenplan: Default::default(), tr_gewinn: vec![], tr_verletzt: 0,
            co_name: [crate::inhalt::t("ui.game.thomas_brueck"), crate::inhalt::t("ui.game.mehmet_ayd_n"), crate::inhalt::t("ui.game.rafael_ortega"), crate::inhalt::t("ui.game.sven_lindqvist"), crate::inhalt::t("ui.game.marco_ferri")][own % 5].to_string(),
            inbox: vec![], feed: vec![], merkliste: vec![], moral_verlauf: Default::default(), db_live: true, akzent_wappen: true, kaderregeln: true, meilensteine: vec![], spieler_ms: None, titel_gefeiert: 0, feiern: vec![], marktstatus: vec![], berater_bez: vec![], db_zuletzt: String::new(), fl27_auto_ende: false, vorgaben_gesetzt: vec![], db_nach_spiel_aus: false, wirkung: None, pool_antworten: vec![], pool_rauschen: vec![], pool_zuletzt: vec![], pool_gezeigt: vec![], pool_ketten: vec![], pool_folgen: vec![], erste_woche: [false; 5], erste_woche_aus: false, erste_woche_ab: 0, verlauf: vec![], spiele: vec![], spieler_saison: vec![], noten_verlauf: Default::default(), ehrungen: vec![], elf_der_woche: vec![], letzte_elf: vec![], lesezeichen: vec![], klauseln_gesaet: false, termine: vec![], teamevents: vec![], event_punkte: vec![], db_abdruck: 0, db_unruhe: None, db_schreibvorgaenge: 0, db_hinweis: false, fl27_letzte: (0, 0, 0), live_aenderung: Default::default(), fl27_offen: false, koop_suche: None, koop_angebote: vec![], vorkauf: vec![], welt_post: (0, 0), suchprofile: vec![], vorvertraege: vec![], sondierungen: vec![], ansprache_vor: None, welt_tag: vec![], strom_log: vec![], bedingungen: vec![], geschichten: Default::default(), kredite: vec![], undo: None, geplant: vec![], trainerprofil: Default::default(), angebot_ligen: vec![], krise: Default::default(), neubau: Default::default(), experten_platz: None, negs: vec![], scouts: vec![
                Scout { name: crate::inhalt::t("ui.game.erik_haldorsen").into(), q: 6, league: Some(own_league), pos: 0, watch: None, seen: 0, p: None },
                Scout { name: crate::inhalt::t("ui.game.bruno_valente").into(), q: 4, league: None, pos: 0, watch: None, seen: 0, p: None },
            ],
            ticket: 1, merch: 1, fans: 60.0, media: 50.0, warned: false, ultimatum: None, ultimatum_ab: None, lockruf_bis: 0, bitten: vec![], gehalt_plus: 0.0, ruhe_bis: 0, verhandlungstempo: if cfg!(test) { 0 } else { 1 }, infra_topf: None, gala: vec![], preise: vec![], preis_kand: vec![], preis_kand_saison: 0, thema_wettbewerb: false, privat: Default::default(), meldungen: vec![], start_stamina: false, direktstart: false, spielminuten: 0, ohne_ki_vorvertraege: false, ohne_ki_klauseln: false, boykott_bis: 0, marktwert_aera: 0, datenpakete: vec![], auslosungen: vec![], auslosung_gesehen: vec![], fired: false,
            mgr_rep: (world.clubs[own].rep * 0.6 + 10.0).min(80.0), mgr_wage: (rev * 0.004).max(120000.0), mgr_money: 0.0, mgr_since: 0,
            book: vec![], balance_hist: vec![world.clubs[own].balance], youth: Default::default(), history: vec![],
            transfer_budget: rev * 0.22, last_result: None, press: None, hard_streak: 0, results: vec![], fl27_erlaubt: false, fl27_bestaetigt: false, fl27_spiele: vec![], fl27_cpk: String::new(), fl27_welt: String::new(), mensch: Default::default(), wi: Default::default(), me: Default::default(), stab: vec![], std: Default::default(), mgr: Default::default(), mst: Default::default(), gespraech_termine: vec![],
            world,
        };
        crate::world::WERT_AERA.store(100, std::sync::atomic::Ordering::Relaxed); // a new career starts with the game's values
        c.world.leagues[own_league].detail = 0;
        c.world.trainer_ausstatten(Some(own));
        c.world.clubs[own].manager = coach.to_string();
        c.mensch.vertrag_bis = (crate::regeln::f("laufbahn", "vertrag.laufzeit_start_monate") * 30.4) as u32;
        c.psyche_an();
        c.wirtschaft_an();
        c.stab_an();
        c.academy_intake(30, true);
        c.make_youth_leagues();
        c.auto_lineup();
        let cl = c.world.clubs[own].clone();
        let goal = crate::tf!("ui.game.das_praesidium_erwartet_mindestens_platz", cl.goal_kind, cl.goal_pos);
        let tb = money(c.transfer_budget);
        c.msg(5, "Präsidium", &crate::tf!("station.neu.vorstellung.betreff", cl.name), &crate::tf!("ui.game.willkommen_der_verein_freut_sich", coach, goal, tb), Action::None); // roh
        let op = c.co_opinion();
        let co = c.co_name.clone();
        c.msg(1, &co, crate::inhalt::t("ui.game.erste_gedanken_zum_kader"), &op, Action::None);
        if let Some(b) = sofifa { let t = crate::tf!("ui.sofifa.text", b.gefunden, b.spieler, b.leihen, b.quelle); c.msg(5, &co, crate::inhalt::t("ui.sofifa.betreff"), &t, Action::None); }
        c.ligaregeln_laden();
        c.wirtschaft_an();
        c.start_erwartung();
        c.nebenziele_setzen();
        c.experten_tipp();
        // Konzept Scouting 1.1: the start knowledge is the base knowledge (stars known, small leagues unknown)
        for p in 0..c.world.players.len() { if c.world.players[p].team >= 0 && c.world.players[p].team != c.own as i32 { c.world.players[p].know = c.grundwissen(p); } }
        c
    }

    pub fn abs(&self) -> u32 { abs_day(self.world.season, self.world.day) }
    /// Remember the whole state before a day step or an own match (Undo).
    pub fn undo_merken(&mut self) { let mut k = self.clone(); k.undo = None; self.undo = Some(Box::new(k)); }
    /// Back to the remembered state, exactly; the save is written again so the file agrees.
    pub fn rueckgaengig(&mut self) -> bool {
        let Some(k) = self.undo.take() else { return false };
        *self = *k;
        let _ = self.save();
        true
    }
    pub fn club(&self) -> &Club { &self.world.clubs[self.own] }
    pub fn league(&self) -> usize { self.world.clubs[self.own].league }

    /// A message from the transfer department about the talks with player `p` (Fabian 24.11.: every move in a
    /// negotiation reaches the inbox, with a button to the talks).
    pub fn neg_msg(&mut self, p: usize, title: &str, body: &str) {
        self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), title, body, Action::None);
        self.inbox[0].art = crate::id("angebot").into(); // postkarten::Art::Angebot
        self.inbox[0].spieler = vec![p];
    }
    pub fn msg(&mut self, cat: u8, from: &str, title: &str, body: &str, action: Action) {
        self.msg_id += 1;
        let mut m = Msg { vereine: vec![], id: self.msg_id, day: self.world.day, season: self.world.season, cat, from: from.into(), title: title.into(), body: body.into(), read: false, action, done: false, pflicht: 0, stufe: 0, frist: 0, faden: String::new(), art: String::new(), spieler: vec![], karte: String::new(), werte: vec![] };
        self.einstufen(&mut m);
        // no repeats (Fabian 22.11.): the same message again replaces the older one; an unanswered half-time or
        // mind-games question is replaced by the next match's, the same rumour or advice by the newer one
        let abs = self.abs();
        let familie = |a: &Action| match a { Action::Wahl { art, .. } => art.split(':').next().map(|x| x.to_string()), _ => None };
        let neu_fam = familie(&m.action);
        self.inbox.retain(|o| {
            let alter = abs.saturating_sub(o.season * crate::date::SEASON_DAYS + o.day);
            let gleich = o.title == m.title && o.body == m.body && alter <= 30;
            let ersetzt = neu_fam.is_some() && familie(&o.action) == neu_fam && matches!(neu_fam.as_deref(), Some("halbzeit" | "mind" | "jobverh"));
            let gleiches_thema = alter <= 30 && familie(&o.action) == neu_fam && matches!(neu_fam.as_deref(), Some("geruecht" | "rat")) && (o.title == m.title || o.body == m.body);
            !(gleich || ersetzt || gleiches_thema)
        });
        self.inbox.insert(0, m);
        if self.inbox.len() > 300 { self.inbox.truncate(300); }
    }
    /// Whether a headline concerns the career (Medien-Konzept 2.2, Fabian 23.11.: the ticker only from competitions that
    /// interest us, the top five leagues always): the own club, its league and the country's next tiers, the top-five first
    /// divisions, the cups and European competitions the own club plays in. Everything else is "fern" and makes no news.
    pub fn nachricht_relevant(&mut self, head: &str) -> bool {
        let key = (self.world.season, self.own, self.world.cups.len());
        if self.rel_cache.0 != key || self.rel_cache.1.is_empty() {
            let own = &self.world.clubs[self.own];
            let lo = &self.world.leagues[own.league];
            let top5 = [crate::id("England"), crate::id("Spanien"), crate::id("Italien"), crate::id("Deutschland"), crate::id("Frankreich")];
            let ligen: Vec<usize> = (0..self.world.leagues.len()).filter(|&l| {
                let lg = &self.world.leagues[l];
                l == own.league || (lg.land == lo.land && lg.stufe <= lo.stufe.saturating_add(1)) || (lg.stufe == 1 && top5.contains(&lg.land.as_str()))
            }).collect();
            let mut vereine: Vec<usize> = ligen.iter().flat_map(|&l| self.world.leagues[l].teams.iter().copied()).collect();
            // big clubs anywhere (Fabian 23.11.: Bayern doesn't care about Al Shabab); cups only bring their clubs of the own country
            let gross = crate::regeln::f("medien", "relevanz.grosser_verein_ab");
            vereine.extend((0..self.world.clubs.len()).filter(|&c| self.world.clubs[c].level >= gross));
            for cup in self.world.cups.iter().filter(|c| c.teams.contains(&self.own)) { vereine.extend(cup.teams.iter().copied().filter(|&t| !self.world.ist_amateur(t) && crate::cups::country(&self.world.leagues[self.world.clubs[t].league]) == crate::cups::country(lo))); }
            vereine.push(self.own);
            let mut namen: Vec<String> = vereine.into_iter().map(|c| self.world.clubs[c].name.clone()).collect();
            namen.sort(); namen.dedup();
            self.rel_cache = (key, namen);
        }
        self.rel_cache.1.iter().any(|n| head.contains(n.as_str()))
    }
    /// Which outlet writes a headline (Fabian 23.11.: all outlets of the media pool, each in its style, and the local paper):
    /// the country of the club named, the kind of story (rumour, transfer, own club, league) and a variant that does not
    /// repeat. Returned as "layout|name" (pages.rs artikel_karte).
    pub fn medium_fuer(&mut self, head: &str, text: &str) -> String {
        let w = |k: &str| crate::regeln::wert("medien", &format!("webblog_regeln.{k}")).as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>()).unwrap_or_default();
        let alles = format!("{head} {text}");
        let transfer = w("transfer_woerter").iter().any(|x| alles.contains(x));
        let geruecht = w("geruecht_woerter").iter().any(|x| alles.contains(x));
        let gross = alles.split_whitespace().zip(alles.split_whitespace().skip(1)).any(|(z, e)| e.starts_with(crate::id("Mio")) && z.replace(',', ".").trim_start_matches('(').parse::<f32>().map_or(false, |x| x >= crate::regeln::f("medien", "webblog_regeln.boulevard_ab_mio")));
        // the country of the first club the headline names, else the own one
        let own = self.club().name.clone();
        let eigen = head.contains(&own);
        let land = self.world.clubs.iter().enumerate().filter(|(_, c)| c.name.len() > 3 && head.contains(c.name.as_str())).map(|(k, _)| self.world.leagues[self.world.clubs[k].league].land.clone()).next().unwrap_or_else(|| self.world.leagues[self.league()].land.clone());
        let kurz = crate::medien::land_kuerzel(&land);
        let art = if geruecht || (transfer && gross) { "geruecht" } else if transfer { "transfer" } else if eigen { "eigen" } else { "liga" }; // roh
        let passt: &[&str] = match art { "geruecht" => &["boulevard", "tabloid_en", "datenportal", "tv"], "transfer" => &["datenportal", "serioes", "fachmagazin", "datenportal"], "eigen" => &["lokal", "fachmagazin", "tv", "boulevard", "podcast"], _ => &["fachmagazin", "serioes", "analyse", "tv", "podcast", "lokal"] }; // roh
        let mut kand: Vec<String> = crate::medien::medienpool_voll().iter().filter(|m| m.1 == kurz && passt.contains(&m.2.as_str())).map(|m| format!("{}|{}", m.2, m.0)).collect();
        // the own club also has its local paper and its fan blog
        if eigen { let k = crate::medien::kern_name(&own); kand.push(format!("lokal|{}", crate::tf!("ui.blog.lokal_name", k))); kand.push(format!("podcast|{}", crate::tf!("ui.blog.fanblog_name", k))); }
        if kand.is_empty() { kand = crate::medien::medienpool_voll().iter().filter(|m| m.1 == "INT" && m.2 != "reporter").map(|m| format!("{}|{}", m.2, m.0)).collect(); } // roh
        if kand.is_empty() { return "fachmagazin|kicker".into(); } // roh
        let i = self.variante(&format!("medium:{art}:{kurz}"), kand.len(), head);
        kand[i].clone()
    }
    pub fn news(&mut self, head: &str, text: &str) {
        if !self.nachricht_relevant(head) { return; }
        let medium = self.medium_fuer(head, text);
        let paper = medium.split_once('|').map(|x| x.1.to_string()).unwrap_or_else(|| PAPERS[0].to_string());
        self.feed.insert(0, News { day: self.world.day, season: self.world.season, paper, medium, head: head.into(), text: text.into() });
        if self.feed.len() > 200 { self.feed.truncate(200); }
    }
    pub fn unread(&self) -> usize { self.inbox.iter().filter(|m| !m.read).count() }
    pub fn open_decisions(&self) -> usize { self.inbox.iter().filter(|m| !m.done && m.action != Action::None && !matches!(m.action, Action::Report { .. })).count() }

    pub fn squad(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self.world.clubs[self.own].squad.iter().copied().filter(|&p| self.world.players[p].youth == 0).collect();
        v.sort_by(|a, b| {
            let (pa, pb) = (&self.world.players[*a], &self.world.players[*b]);
            (part_of(&pa.pos), -(pa.ovr as i32)).cmp(&(part_of(&pb.pos), -(pb.ovr as i32)))
        });
        v
    }
    pub fn youth_squad(&self, age_group: u8) -> Vec<usize> {
        let mut v: Vec<usize> = self.world.clubs[self.own].squad.iter().copied().filter(|&p| self.world.players[p].youth == age_group).collect();
        v.sort_by(|a, b| self.world.players[*b].ovr.partial_cmp(&self.world.players[*a].ovr).unwrap());
        v
    }
    pub fn in_window(&self) -> Option<(u32, u32)> {
        WINDOWS.iter().copied().find(|(a, b)| self.world.day >= *a && self.world.day <= *b)
    }
    /// A deadline of `tage` days, never past the end of the open window (Fabian 24.11.: on deadline day nobody says
    /// "I'll get back to you in four days"); 0 = today.
    pub fn frist(&self, tage: u32) -> u32 { self.in_window().map_or(tage, |(_, b)| tage.min(b - self.world.day)) }
    /// "in 3 days" or "by tonight" for a deadline of `tage` days.
    pub fn frist_text(&self, tage: u32) -> String { if tage == 0 { crate::inhalt::t("ui.frist.heute").into() } else { crate::tf!("ui.frist.tage", tage) } }

    // ------------------------------------------------------------ fixtures
    /// All own matches of the season (league, cups, Europe), by day.
    pub fn own_fixtures(&self) -> Vec<(Wo, Fixture)> {
        let l = self.league();
        let mine = |f: &Fixture| f.h == self.own || f.a == self.own;
        let mut v: Vec<(Wo, Fixture)> = self.world.leagues[l].fixtures.iter().filter(|f| mine(f)).map(|f| (Wo::Liga(l), *f)).collect();
        // a cup match against an amateur club is simulated with the cup (pokalregeln.rs), never an own match day
        let echt = |f: &Fixture| !self.world.ist_amateur(f.h) && !self.world.ist_amateur(f.a);
        for (ci, c) in self.world.cups.iter().enumerate() { v.extend(c.fx.iter().filter(|x| mine(&x.f) && echt(&x.f)).map(|x| (Wo::Pokal(ci), x.f))); }
        v.sort_by_key(|x| x.1.day);
        v
    }
    pub fn today_match(&self) -> Option<(Wo, usize)> {
        let d = self.world.day;
        let here = |f: &Fixture| f.day == d && f.hg < 0 && (f.h == self.own || f.a == self.own) && !self.world.ist_amateur(f.h) && !self.world.ist_amateur(f.a);
        let l = self.league();
        if let Some(i) = self.world.leagues[l].fixtures.iter().position(here) { return Some((Wo::Liga(l), i)); }
        self.world.cups.iter().enumerate().find_map(|(ci, c)| c.fx.iter().position(|x| here(&x.f)).map(|i| (Wo::Pokal(ci), i)))
    }
    pub fn fixture(&self, t: (Wo, usize)) -> Fixture {
        match t.0 { Wo::Liga(l) => self.world.leagues[l].fixtures[t.1], Wo::Pokal(c) => self.world.cups[c].fx[t.1].f }
    }
    /// Name of a competition, with the round for cups ("England-Pokal, Viertelfinale").
    /// Competition id of a match (for its logo).
    pub fn comp_cid(&self, w: Wo) -> Option<u16> {
        let key = match w { Wo::Liga(l) => &self.world.leagues[l].key, Wo::Pokal(c) => &self.world.cups[c].key };
        self.world.cids.get(key).copied()
    }
    pub fn comp_name(&self, t: (Wo, usize)) -> String {
        match t.0 {
            Wo::Liga(l) => self.world.leagues[l].name.clone(),
            Wo::Pokal(c) => { let cup = &self.world.cups[c]; format!("{}, {}", cup.name, crate::cups::stage_name(cup.fx[t.1].stage, cup.kind)) }
        }
    }
    /// Competition of the next own match (today or later) for the FL27 presets.
    fn next_termin(&self) -> Option<(Wo, usize)> {
        if let Some(t) = self.today_match() { return Some(t); }
        let mut best: Option<((Wo, usize), u32)> = None;
        let l = self.league();
        let mine = |f: &Fixture| f.hg < 0 && (f.h == self.own || f.a == self.own) && !self.world.ist_amateur(f.h) && !self.world.ist_amateur(f.a);
        for (i, f) in self.world.leagues[l].fixtures.iter().enumerate() { if mine(f) && best.map(|b| f.day < b.1).unwrap_or(true) { best = Some(((Wo::Liga(l), i), f.day)); } }
        for (ci, c) in self.world.cups.iter().enumerate() { for (i, x) in c.fx.iter().enumerate() { if mine(&x.f) && best.map(|b| x.f.day < b.1).unwrap_or(true) { best = Some(((Wo::Pokal(ci), i), x.f.day)); } } }
        best.map(|b| b.0)
    }
    pub fn next_match(&self) -> Option<Fixture> {
        self.own_fixtures().into_iter().map(|x| x.1).find(|f| f.hg < 0)
    }

    // ------------------------------------------------------------ lineup
    pub fn auto_lineup(&mut self) {
        let k = self.aufstellung_kriterien;
        let alt = self.xi.clone();
        let sq = self.squad();
        let schnitt = sq.iter().map(|&p| self.world.players[p].apps as f32).sum::<f32>() / sq.len().max(1) as f32;
        let w = &self.world;
        // Fabian 23.11.: the criteria the person picked, one or several (bit 0 strength … bit 6 young players)
        let bonus = |p: usize| -> f32 {
            let pl = &w.players[p];
            let an = |b: u8| k & (1 << b) != 0;
            let mut b = 0.0;
            if !an(0) { b -= 0.6 * (pl.ovr - 60.0); }
            if an(1) { b += (pl.morale - 60.0) / 5.0; }
            if an(2) { b += (pl.fit - 85.0) / 3.0; }
            if an(3) && !pl.last.is_empty() { let l: Vec<f32> = pl.last.iter().rev().take(5).copied().collect(); b += (l.iter().sum::<f32>() / l.len() as f32 - 6.5) * 6.0; }
            if an(4) { b += if alt.contains(&p) { -4.0 } else { 2.0 }; }
            if an(5) { b += ((schnitt - pl.apps as f32) * 0.8).clamp(-5.0, 8.0); }
            if an(6) { b += (23.0 - pl.age as f32).max(0.0) * 1.5; }
            b
        };
        let (xi, _) = self.world.lineup_slots_mit(self.own, self.eigene_slots(), &[], &bonus);
        self.xi = xi.iter().map(|x| x.1).collect();
    }
    pub fn current_xi(&self) -> Vec<(String, usize, f32)> {
        self.world.lineup_slots(self.own, self.eigene_slots(), &self.xi).0
    }
    /// The own spots on the tactics board: freely moved ones, else the formation's.
    pub fn eigene_spots(&self) -> [(f32, f32); 11] {
        if self.frei.len() == 11 { let mut a = [(0.0, 0.0); 11]; for i in 0..11 { a[i] = self.frei[i]; } a } else { crate::world::formation_spots(self.formation) }
    }
    /// Spots and slot roles of set `k` (0 kick-off, 1 with the ball, 2 against it): free placement first, then the
    /// chosen formation, else the kick-off one.
    pub fn satz_frei(&mut self, k: u8) -> &mut Vec<(f32, f32)> { match k { 1 => &mut self.frei_ball, 2 => &mut self.frei_gegen, _ => &mut self.frei } }
    pub fn satz_spots(&self, k: u8) -> [(f32, f32); 11] {
        let (frei, f) = match k { 1 => (&self.frei_ball, self.formation_ball), 2 => (&self.frei_gegen, self.formation_gegen), _ => return self.eigene_spots() };
        if frei.len() == 11 { let mut a = [(0.0, 0.0); 11]; a.copy_from_slice(frei); a } else { f.map(crate::world::formation_spots).unwrap_or_else(|| self.eigene_spots()) }
    }
    pub fn satz_slots(&self, k: u8) -> [&'static str; 11] {
        let (frei, f) = match k { 1 => (&self.frei_ball, self.formation_ball), 2 => (&self.frei_gegen, self.formation_gegen), _ => return self.eigene_slots() };
        if frei.len() == 11 { crate::world::rollen_aus_lagen(frei) } else { f.map(crate::world::formation_slots).unwrap_or_else(|| self.eigene_slots()) }
    }
    /// The own slot roles: from the spots when players were moved freely, else the formation's.
    pub fn eigene_slots(&self) -> [&'static str; 11] {
        if self.frei.len() == 11 { crate::world::rollen_aus_lagen(&self.frei) } else { crate::world::formation_slots(self.formation) }
    }
    pub fn team_strength(&self) -> f32 {
        let xi = self.current_xi();
        xi.iter().map(|x| x.2).sum::<f32>() / xi.len().max(1) as f32 + self.world.fam_bonus(self.own)
    }
    /// Co-trainer advice in football language: where his best XI differs from the user's.
    pub fn co_opinion(&self) -> String {
        let (best, _) = self.world.lineup_slots(self.own, self.eigene_slots(), &[]);
        let mine = self.current_xi();
        let mut tips = vec![];
        for (i, (slot, p, e)) in mine.iter().enumerate() {
            if let Some((_, b, be)) = best.get(i) {
                if b != p && *be > e + 2.0 && !mine.iter().any(|m| m.1 == *b) {
                    let (pp, bp) = (&self.world.players[*p], &self.world.players[*b]);
                    let why = if pp.fit + 15.0 < bp.fit { crate::tf!("ui.game.ist_frischer_gegen", bp.name, format!("{:.0}", bp.fit), format!("{:.0}", pp.fit)) } else if pos_penalty(&pp.pos, slot) > 0.0 && bp.pos == *slot { crate::tf!("ui.game.ist_gelernter", bp.name, pos_de(slot)) } else { crate::tf!("ui.game.ist_im_moment_staerker", bp.name) };
                    tips.push(crate::tf!("ui.game.auf_wuerde_ich_statt_bringen", pos_de(slot), bp.name, pp.name, why));
                }
            }
        }
        let inj: Vec<String> = self.squad().iter().filter(|&&p| self.world.players[p].inj > 0).map(|&p| crate::tf!("ui.game.tage", self.world.players[p].name, self.world.players[p].inj)).collect();
        let mut s = if tips.is_empty() { crate::inhalt::t("ui.game.die_aufstellung_passt_so_ich").to_string() } else { tips.into_iter().take(3).collect::<Vec<_>>().join(" ") };
        if !inj.is_empty() { s += &crate::tf!("ui.game.verletzt_fehlen", inj.join(", ")); }
        let fam = self.world.clubs[self.own].fam;
        s += &crate::tf!("ui.game.die_mannschaft_ist", if fam >= 75.0 { crate::inhalt::t("ui.game.sehr_gut_eingespielt") } else if fam >= 55.0 { crate::inhalt::t("ui.game.gut_eingespielt") } else if fam >= 35.0 { crate::inhalt::t("ui.game.noch_nicht_eingespielt") } else { crate::inhalt::t("ui.game.ein_loser_haufen") });
        s
    }

    // ------------------------------------------------------------ own match
    pub fn prepare_match(&mut self, forced: Option<(u8, u8)>) -> MatchResult {
        if self.co_lineup { self.auto_lineup(); }
        let t = self.today_match().expect("no match today"); // roh
        // in Europe only the registered may play (Fabian 24.11., meldung.rs)
        if let (Wo::Pokal(ci), _) = t { self.meldung_pruefen(ci); }
        let f = self.fixture(t);
        if let (Wo::Pokal(ci), i) = t { self.world.neutral = self.world.cups[ci].fx[i].neutral; }
        let own = Some((self.own, self.formation, self.xi.clone(), self.tactic.style()));
        // the settings act on both sides' expected goals (taktikwirkung.rs), the counter rules go into the report
        let gegner = if f.h == self.own { f.a } else { f.h };
        let (fe, fg, saetze) = self.taktik_wirkung(gegner);
        self.world.taktik_faktor = Some((self.own, fe, fg));
        let mut res = self.world.play(f.h, f.a, 0, own, forced);
        if !saetze.is_empty() { res.reason = format!("{} {}", res.reason, saetze.join(" ")); }
        if forced.is_none() { self.plan_b_anwenden(&mut res); }
        res
    }
    /// Plan B in the simulation: at its minute the score is checked, and if it fits the rest is played with the other mentality.
    fn plan_b_anwenden(&mut self, res: &mut MatchResult) {
        let pb = self.plan_b.clone();
        if !pb.an { return; }
        let side = if res.h == self.own { 0u8 } else if res.a == self.own { 1 } else { return };
        let tore = |s: u8| res.events.iter().filter(|e| e.side == s && e.kind <= 1 && e.min <= pb.minute).count() as i32;
        let d = tore(side) - tore(1 - side);
        if !match pb.wenn { 0 => d < 0, 1 => d <= 0, _ => d > 0 } { return; }
        let mut elf: Vec<usize> = (if side == 0 { &res.xi_h } else { &res.xi_a }).iter().map(|x| x.1).collect();
        for e in res.events.iter().filter(|e| e.side == side && e.min <= pb.minute) {
            match e.kind { 5 => { if let (Some(i), Some(on)) = (elf.iter().position(|&p| p == e.pid), e.pid2) { elf[i] = on; } } 3 => elf.retain(|&p| p != e.pid), _ => {} }
        }
        let (fe, fg) = match pb.mentalitaet { -1 => (0.85, 0.88), 1 => (1.12, 1.10), _ => (1.0, 1.0) };
        self.world.rest_neu(res, pb.minute, side, &elf, fe, fg);
        res.reason = format!("{} {}", res.reason, crate::tf!("ui.planb.bericht", pb.minute, crate::inhalt::t(&format!("ui.live.ment.{}", pb.mentalitaet + 1))));
    }

    pub fn apply_own_match(&mut self, mut res: MatchResult) {
        self.erste_woche_schritt(4);
        // Konzept Scouting 1.1: whoever played against us stays in mind, +6 a match up to 60
        for &(p, _) in &res.minutes {
            if self.gehoert_uns(p) { continue; }
            let k = &mut self.world.players[p].know;
            if *k < 0.60 { *k = (*k + 0.06).min(0.60); }
        }
        let Some(t) = self.today_match() else { return };
        let comp = self.comp_name(t);
        match t.0 {
            Wo::Liga(l) => self.world.apply_result(l, t.1, &res),
            Wo::Pokal(ci) => {
                let vorher = self.world.cups[ci].winner;
                self.world.finish_cup_match(ci, t.1, &mut res); self.world.cup_progress(); self.pokal_praemie(ci);
                if vorher.is_none() && self.world.cups[ci].winner == Some(self.own) && self.world.cups[ci].kind != 6 { self.siegerehrung(ci, &res); }
            }
        }
        if res.et { res.reason = crate::tf!("ui.game.nach_verlaengerung", res.reason, res.pens.map(|p| crate::tf!("ui.game.und_elfmeterschiessen", crate::cups::pens_score(p))).unwrap_or_default()); }
        if !matches!(t.0, Wo::Liga(_)) { res.reason = crate::tf!("diagnose.schalter", comp, res.reason); }
        let home = res.h == self.own;
        let (gf, ga) = if home { (res.hg, res.ag) } else { (res.ag, res.hg) };
        let opp = if home { res.a } else { res.h };
        let opp_name = self.world.clubs[opp].name.clone();
        let own_name = self.club().name.clone();
        let derby = is_derby(&own_name, &opp_name);
        let wb = match t.0 { Wo::Liga(l) => self.world.leagues[l].name.clone(), Wo::Pokal(c) => self.world.cups[c].name.clone() };
        self.statistik_buchen(&res, &wb, derby.is_some());
        // Skill Points (skilltree.rs): friendlies (kind 6) count nothing, cups a little more, Europe more
        let faktor_wb = match t.0 { Wo::Liga(_) => 1.0, Wo::Pokal(ci) => match self.world.cups[ci].kind { 6 => 0.0, 1..=3 => crate::regeln::f("skilltree", "punkte.faktor_europa"), _ => crate::regeln::f("skilltree", "punkte.faktor_pokal") } };
        if faktor_wb > 0.0 { self.st_nach_spiel(&res, faktor_wb); }
        // Coaching Points for a win against a clearly stronger side, at most two a season (skilltree.toml [manager])
        if gf > ga && crate::mensch::kaderstaerke(&self.world, opp) - crate::mensch::kaderstaerke(&self.world, self.own) >= crate::regeln::f("skilltree", "manager.staerker_ab") {
            let s = self.world.season;
            let siege = self.meilensteine.iter().filter(|m| m.0.starts_with(&format!("cp_sieg:{s}:"))).count();
            if siege < crate::regeln::f("skilltree", "manager.sieg_staerker_max") as usize {
                self.meilensteine.push((format!("cp_sieg:{s}:{}", self.abs()), self.abs())); // roh
                self.cp_geben(crate::regeln::f("skilltree", "manager.sieg_staerker") as u8, &crate::tf!("ui.mst.grund.sieg", opp_name.clone()));
            }
        }
        self.teamgespraech_auswertung(gf, ga);
        self.krise_pruefen();
        self.ultimatum_spiele_pruefen();
        self.praemien_nach_spiel(&res);
        // the league's win bonus is income of the club (vereinswelt/service.py einnahmen „praemien“)
        if gf > ga { let pr = (crate::regeln::f("vereinswelt", "einnahmen.praemie_sieg") as f64 + crate::regeln::f("vereinswelt", "einnahmen.praemie_sieg_je_ruf") as f64 * self.kraft()) * self.skala(); self.book(crate::inhalt::t("ui.game.siegpraemien"), pr); }
        if home { self.heimspiel_buchen(); }
        let score = format!("{}:{}", res.hg, res.ag);
        let line = format!("{} – {} {}", self.world.clubs[res.h].name, self.world.clubs[res.a].name, score);
        self.results.push(format!("{} {}", if gf > ga { crate::id("S") } else if gf < ga { crate::id("N") } else { crate::id("U") }, line));
        if matches!(t.0, Wo::Liga(_)) { self.ziel_auffrischen(); }
        self.nach_spiel(&res);
        self.story_takt();
        self.manager_nach_spiel();
        // the half-time talk happens on the ticker or in the FL27 live view (Fabian 22.11.), not in the inbox
        let best = res.ratings.iter().filter(|r| self.world.players[r.0].team == self.own as i32).max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|r| (self.world.players[r.0].name.clone(), r.1));
        let mut body = format!("{} {}", line, res.reason);
        if let Some((n, r)) = &best { body += &crate::tf!("ui.game.bester_spieler_bei_uns_note", n, format!("{:.1}", r)); }
        // a few sentences on who played how (Fabian 23.11.): the weakest of ours and the decisive goal
        if let Some((n, r)) = res.ratings.iter().filter(|r| self.world.players[r.0].team == self.own as i32).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|r| (self.world.players[r.0].name.clone(), r.1)) {
            if r <= 5.5 { body = format!("{body} {}", crate::tf!("sb2.schwach", n, format!("{r:.1}").replace('.', ","))); }
        }
        if gf != ga {
            let sieger_seite = if (gf > ga) == home { 0u8 } else { 1u8 };
            let ziel = gf.min(ga) + 1;
            let mut z = 0;
            for e in res.events.iter().filter(|e| e.kind == 0 || e.kind == 1) {
                let fuer = if e.kind == 0 { e.side } else { 1 - e.side };
                if fuer == sieger_seite { z += 1; if z == ziel { body = format!("{body} {}", crate::tf!("sb2.entscheidend", self.world.players[e.pid].name.clone(), e.min)); break; } }
            }
        }
        let an = self.analyse(&res);
        if !an.is_empty() { body += &crate::tf!("ui.game.text", an); }
        self.msg(0, crate::inhalt::t("bericht.titel"), &format!("{} {}", if gf > ga { crate::inhalt::t("ui.game.sieg") } else if gf < ga { crate::inhalt::t("ui.game.niederlage") } else { crate::inhalt::t("ui.game.remis") }, line), &body, Action::None);
        self.meldung_art(crate::pages::Art::Spiel, vec![]);
        // the paper's headline from a bag of context-rich variants (Fabian 23.11.: every report one of a kind): score,
        // opponent, the player of the match, the table position; the old three per result stay in the bag too
        let art = if gf > ga { "sieg" } else if gf < ga { "niederlage" } else { "remis" }; // roh
        let basis = format!("ui.zt.spiel.{art}");
        let n = crate::inhalt::varianten(&basis);
        let held = best.as_ref().map(|b| b.0.clone()).unwrap_or_else(|| own_name.clone());
        let platz = self.world.rank_of(self.own).to_string();
        let r = self.world.rng.next() as usize;
        let alt = if gf > ga {
            [crate::tf!("ui.game.siegt_gegen", own_name, score, opp_name), crate::tf!("ui.game.drei_punkte_fuer", own_name), crate::tf!("ui.game.laesst_keine_chance", own_name, opp_name)][r % 3].clone()
        } else if gf < ga {
            [crate::tf!("ui.game.rueckschlag_fuer", own_name), crate::tf!("ui.game.entfuehrt_die_punkte", opp_name), crate::tf!("ui.game.fragen_nach_dem", own_name, score)][r % 3].clone()
        } else {
            [crate::tf!("ui.game.punkteteilung_zwischen_und", own_name, opp_name), crate::tf!("ui.game.tritt_auf_der_stelle", own_name), crate::inhalt::t("ui.game.ein_punkt_der_niemandem_weh").to_string()][r % 3].clone()
        };
        let i = self.variante(&format!("zt_spiel_{art}"), n + 1, &opp_name);
        let head = if i == n { alt } else { crate::tf!(&crate::inhalt::variante_key(&basis, i), own_name.clone(), opp_name.clone(), score.clone(), held, platz) };
        let mut text = res.reason.clone();
        if let Some(d) = derby { text = crate::tf!("diagnose.schalter", d, text); }
        if let Some((n, rr)) = best { if rr >= 8.0 { text += &crate::tf!("ui.game.herausragend", n); } }
        self.news(&head, &text);
        for (p, d) in &res.injuries {
            if self.world.players[*p].team == self.own as i32 {
                let pl = &self.world.players[*p];
                let n = pl.name.clone();
                let t = format!("{}\n\n{}", crate::tf!("ui.game.hat_sich_verletzt_er_faellt", pl.name, pl.inj_name, d), self.verletzung_gruende(*p, true, *d));
                self.msg(1, crate::inhalt::t("med.vg.absender"), &crate::tf!("ui.game.verletzung", n), &t, Action::None); self.meldung_art(crate::pages::Art::Verletzung, vec![*p]);
            }
        }
        self.last_result = Some(res);
        let q = self.press_question(true);
        self.press = Some(q);
        if self.co_press { self.answer_press(Some(0)); }
    }

    pub fn book(&mut self, what: &str, v: f64) {
        self.world.clubs[self.own].balance += v;
        if let Some(b) = self.book.iter_mut().find(|b| b.0 == what) { b.1 += v; } else { self.book.push((what.to_string(), v)); }
    }

    // ------------------------------------------------------------ finances (vereinswelt.toml + FL split)
    fn month(&mut self) {
        let vorher: Vec<(String, f64)> = self.book.clone();
        let wages = self.world.wage_bill(self.own);
        let netto = self.wirtschaft_monat();
        self.kredite_monat();
        let entl = self.leih_entlastung();
        // the second team runs through our books as its own post: its wages, its income and running costs
        let (zw_gehalt, zw_netto) = self.world.zweite_haushalt(self.own).unwrap_or((0.0, 0.0));
        self.book(crate::inhalt::t("vertraege.summe"), -(wages - zw_gehalt - entl) / 12.0);
        if zw_gehalt > 0.0 || zw_netto != 0.0 { self.book(crate::inhalt::t("ui.game.zweite_mannschaft"), (zw_netto - zw_gehalt) / 12.0); }
        self.mgr_money += self.mgr_wage / 12.0;
        self.privat_monat();
        let b = self.club().balance;
        self.balance_hist.push(b);
        if self.balance_hist.len() > 24 { self.balance_hist.remove(0); }
        for i in 0..self.world.clubs.len() {
            if i == self.own || self.world.zweite_von(self.own) == Some(i) { continue; }
            let w = self.world.wage_bill(i);
            // a second team's income and costs go to its parent club (its wages are in the parent's bill already)
            let kasse = self.world.erste_von(i).unwrap_or(i);
            let cl = &self.world.clubs[i];
            let d = (cl.tv + cl.gate + cl.comm - cl.opex - w) / 12.0;
            self.world.clubs[kasse].balance += d;
        }
        self.fans_monat();
        self.stab_monat();
        self.netz_monat();
        // vereinswelt.toml [budget]: a share of the monthly surplus goes into the transfer budget, by board confidence
        let ueberschuss = netto - (wages - entl) / 12.0;
        if ueberschuss > 0.0 { self.transfer_budget += ueberschuss * self.budget_anteil() as f64; }
        self.mgr_rep = self.trainerruf();
        if b < 0.0 {
            self.msg(3, crate::inhalt::t("ui.game.finanzabteilung"), crate::inhalt::t("ui.game.das_konto_ist_im_minus"), &crate::tf!("ui.game.der_kontostand_liegt_bei_das", money(b)), Action::None);
            self.klima(0.0, 0.0, -2.0, crate::inhalt::t("ui.game.das_konto_ist_im_minus"));
        }
        // the month's income and costs: the change of every booking line since last month (the book restarts each season)
        let (mut ein, mut aus) = (0.0, 0.0);
        for (n, x) in &self.book { let d = x - vorher.iter().find(|y| y.0 == *n).map(|y| y.1).unwrap_or(0.0); if d >= 0.0 { ein += d; } else { aus -= d; } }
        if !self.fired { self.wv_monat(ein, aus); }
        if !self.fired && self.mensch.vertrag_bis > 0 && self.abs() >= self.mensch.vertrag_bis { self.trainervertrag_ende(); }
    }

    // ------------------------------------------------------------ the day step
    pub fn end_day(&mut self) -> Result<(), String> {
        if self.fired { return Err(crate::inhalt::t("ui.game.du_bist_ohne_verein_nimm").into()); }
        if self.today_match().is_some() { return Err(crate::inhalt::t("ui.game.heute_ist_spieltag_erst_das").into()); }
        // the coaches' formations from the world, once (only with a real world; returns at once when all have them)
        if !self.fl27_welt.is_empty() && self.world.trainer_an { let p = self.welt().join("common/etc/pesdb"); self.world.trainer_formationen(&p); }
        self.auftraege_tag();
        self.probetraining_tag();
        self.sendung_tag();
        self.raten_tag();
        let day = self.world.day;
        for l in 0..self.world.leagues.len() {
            let level = self.world.leagues[l].detail;
            for fi in 0..self.world.leagues[l].fixtures.len() {
                let f = self.world.leagues[l].fixtures[fi];
                if f.day != day || f.hg >= 0 { continue; }
                let res = self.world.play(f.h, f.a, level, None, None);
                self.world.apply_result(l, fi, &res);
            }
        }
        for l in self.world.split_pruefen() {
            if l != self.league() && self.world.leagues[l].detail > 1 { continue; }
            let ln = self.world.leagues[l].name.clone();
            let t = self.world.split_fuer(l).map(|s| s.gruppen.iter().map(|g| crate::tf!("ui.game.plaetze", g.name, g.plaetze[0], g.plaetze[1])).collect::<Vec<_>>().join(", ")).unwrap_or_default();
            if l == self.league() { self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), &crate::tf!("ui.game.wird_geteilt", ln), &crate::tf!("ui.game.die_grundrunde_ist_vorbei_weiter", t), Action::None); }
            self.news(&crate::tf!("ui.game.wird_geteilt", ln), &t);
        }
        for l in self.world.endrunden_pruefen() {
            if l != self.league() { continue; }
            let (ln, k) = (self.world.leagues[l].name.clone(), self.world.endrunde(l).unwrap_or(0));
            let dabei = self.world.cups.last().map_or(false, |c| c.teams.contains(&self.own));
            let t = if dabei { crate::tf!("ui.game.die_hauptrunde_ist_vorbei_wir", k, ln) } else { crate::tf!("ui.game.die_hauptrunde_ist_vorbei_die", k, ln) };
            self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), crate::inhalt::t("ui.game.endrunde_beginnt"), &t, Action::None);
        }
        let meister_vorher: Vec<Option<usize>> = self.world.leagues.iter().map(|l| l.meister).collect();
        let titles = self.world.titles.len();
        self.world.cup_day(day, self.own);
        // play-offs, playouts, relegation: real matches once the leagues are played (ligaregeln.rs)
        let eigene = |w: &World| w.cups.iter().filter(|c| c.kind == 5).flat_map(|c| c.fx.iter().map(move |x| (c, x))).filter(|(_, x)| x.f.h == self.own || x.f.a == self.own).count();
        let vorher = eigene(&self.world);
        self.world.entscheidungen_planen();
        if eigene(&self.world) > vorher {
            let own = self.own;
            if let Some((cn, x)) = self.world.cups.iter().filter(|c| c.kind == 5).flat_map(|c| c.fx.iter().map(move |x| (c.name.clone(), x.clone()))).filter(|(_, x)| (x.f.h == own || x.f.a == own) && x.f.hg < 0).min_by_key(|(_, x)| x.f.day) {
                let g = if x.f.h == own { x.f.a } else { x.f.h };
                let t = crate::tf!("ui.game.entscheidungen_text", cn, self.world.clubs[g].name, date::long(self.world.season, x.f.day));
                self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), crate::inhalt::t("ui.game.entscheidungen_beginnen"), &t, Action::None);
            }
        }
        for l in 0..self.world.leagues.len() {
            let Some(m) = self.world.leagues[l].meister.filter(|_| meister_vorher[l].is_none()) else { continue };
            let (mn, ln) = (self.world.clubs[m].name.clone(), self.world.leagues[l].name.clone());
            if l == self.league() {
                let t = if m == self.own { crate::tf!("ui.game.gewinnt_die_endrunde_und_ist", mn, ln) } else { crate::tf!("ui.game.die_endrunde_ist_gespielt_ist", mn, ln) };
                self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), if m == self.own { crate::inhalt::t("ui.game.meister") } else { crate::inhalt::t("ui.game.endrunde_entschieden") }, &t, Action::None);
            }
            if l == self.league() || self.world.leagues[l].detail <= 1 { self.news(&crate::tf!("ui.game.ist_meister_in", mn, ln), crate::inhalt::t("ui.game.sieger_der_endrunde")); }
        }
        // international break: the nations nominate and play (nationalteams.rs); the own club feels it
        let fenster = self.world.laenderspielfenster(day);
        if !fenster.is_empty() {
            self.laenderspiel_folgen(&fenster);
            // our U21 call-ups in one message
            let u21: Vec<String> = self.squad().into_iter().filter(|p| self.world.nationalteams.iter().any(|t| t.u21.contains(p))).map(|p| format!("{} ({})", self.world.players[p].name, crate::nationen::name_anzeige(&self.world, self.world.players[p].nat))).collect();
            if !u21.is_empty() { self.msg(1, crate::inhalt::t("ui.nt.u21_absender"), &crate::tf!("ui.nt.u21_titel", u21.len()), &crate::tf!("ui.nt.u21_text", u21.join(", ")), Action::None); }
        }
        for (name, spur, sieger) in self.world.turniere_tag() { self.turnier_folgen(&name, &spur, sieger); }
        let own = self.own;
        let mine: f64 = self.world.cup_money.iter().filter(|x| x.0 == own).map(|x| x.1).sum();
        if mine > 0.0 { self.world.clubs[own].balance -= mine; self.book(crate::inhalt::t("ui.game.praemien_europapokal"), mine); }
        if let Some(z) = self.world.zweite_von(own) {
            let unten: f64 = self.world.cup_money.iter().filter(|x| x.0 == z).map(|x| x.1).sum();
            if unten > 0.0 { self.world.clubs[own].balance -= unten; self.book(crate::inhalt::t("ui.game.zweite_mannschaft"), unten); }
        }
        self.world.cup_money.clear();
        for t in self.world.titles[titles..].to_vec() {
            if t.club == own {
                let cn = self.club().name.clone();
                self.msg(0, "Präsidium", &crate::tf!("tb.ui.titel_n", t.comp), &crate::tf!("ui.game.gewinnt_den_wettbewerb_die_stadt", cn, t.comp), Action::None); // roh
                self.news(&crate::tf!("ui.game.holt_den_titel", cn, t.comp), crate::inhalt::t("ui.game.ein_abend_fuer_die_vereinsgeschichte"));
                self.mensch.titel += 1;
                self.klima(5.0, 3.0, 4.0, &crate::tf!("ui.game.titel_gewonnen", t.comp));
            } else if self.world.cups.iter().any(|c| c.name == t.comp && (1..=3).contains(&c.kind)) {
                let n = self.world.clubs[t.club].name.clone();
                self.news(&crate::tf!("ui.game.gewinnt_die", n, t.comp), crate::inhalt::t("ui.game.ein_triumph_auf_der_grossen"));
            }
        }
        self.training_day();
        self.world.recover_all();
        self.psyche_tag();
        self.ent_tag();
        self.medizin_tag();
        self.sondierung_tag();
        self.geplante_wechsel();
        self.bedingungen_pruefen();
        self.geschichten_woche();
        self.neubau_tag();
        if date::weekday(day) == 0 { self.week(); }
        if date::weekday(day) == 6 && day >= LEAGUE_START && day <= LEAGUE_END { self.youth_round(); }
        self.negotiations();
        self.drama_tag();
        self.medien_tag();
        self.posteingang_tag();
        self.koop_tag();
        self.story_tag();
        self.belastung_tag();
        self.umschulung_tag();
        self.privat_tag();
        self.anwerbung_tag();
        self.streik_tag();
        // detailed clubs trade daily with reasons (kimarkt.rs), the rest weekly in the simple market
        // on deadline day the market ran hour by hour already (kimarkt.rs deadline_stunde)
        if self.dl_stunde == 0 { self.ki_markt_tag(); }
        self.negs_pruefen();
        self.dl_stunde = 0;
        if let Some((a, _)) = self.in_window() { if (day - a) % 7 == 3 { self.market_round(); self.zweite_markt_woche(); } }
        if day % 3 == 1 { self.leihangebote_tag(); }
        if day % 30 == 29 { self.month(); self.leih_bericht(); self.sportdirektor_monat(); self.zufall_ereignisse_monat(); if day >= 90 { self.board_check(); } }
        self.leihen_tag();
        self.gala_tag();
        self.preise_tag();
        self.auslosungen_pruefen();
        if day == 239 { self.expiring_news(); }
        if let Some(dl) = self.ultimatum { if self.abs() >= dl { self.ultimatum_end(); } }
        if let Some(f) = self.next_match() { if f.day == day + 1 { self.gegnerbericht_senden(); self.mind_games_frage(); let q = self.press_question(false); self.press = Some(q); if self.co_press { self.answer_press(Some(0)); } } }
        self.world.day += 1;
        if self.world.day >= date::SEASON_DAYS { self.season_end(); }
        self.meilensteine_pruefen();
        self.meldung_erinnern();
        // planned talks of today reach the inbox
        let heute = self.abs();
        for (_, p, art) in self.gespraech_termine.iter().copied().filter(|x| x.0 == heute).collect::<Vec<_>>() {
            if self.world.players[p].team != self.own as i32 { continue; }
            let n = self.world.players[p].name.clone();
            let action = if art == 1 { Action::Expiring { pid: p } } else { Action::Report { pid: p } };
            self.msg(1, &n, &crate::tf!(if art == 1 { "ui.kal.vertrag_titel" } else { "ui.kal.gespraech_titel" }, n.clone()), &crate::tf!(if art == 1 { "ui.kal.vertrag_text" } else { "ui.kal.gespraech_text" }, n), action);
        }
        self.gespraech_termine.retain(|x| x.0 > heute);
        // the card „So hat es gewirkt“ is for the person's own talks; what happened during the day (co-trainer) doesn't pop up
        self.wirkung = None;
        Ok(())
    }

    fn training_day(&mut self) {
        if self.co_training { self.co_wochenplan(); } else { self.plan_uebernehmen(); }
        let wd = date::weekday(self.world.day);
        let u = self.plan[wd];
        let (fit, inj, _, fam) = unit_effect(u);
        let sq = self.world.clubs[self.own].squad.clone();
        let med = (2.0 - self.mgr_faktor(5)) * self.medizin_faktor() * (1.0 - crate::regeln::f("entwicklung", "stab.medizin_hebel") * (self.medizinqualitaet() - 50.0) / 50.0);
        self.gf_training(u as usize);
        let deckel: Vec<Option<f32>> = sq.iter().map(|&p| self.kondition_max(p)).collect();
        let unter = crate::regeln::f("vorbereitung", "grundfitness.verletzung_unter");
        let mut hurt = vec![];
        for (k, p) in sq.into_iter().enumerate() {
            let r = self.world.rng.f();
            let pl = &mut self.world.players[p];
            if pl.inj > 0 { continue; }
            let extra = match pl.program { 1 => -2.0 / 6.0, 2 => -4.0 / 6.0, _ => 0.0 };
            let older = if pl.age >= 31 { 1.15 } else { 1.0 };
            pl.fit = (pl.fit + if fit < 0.0 { fit * older } else { fit } + extra).clamp(20.0, 100.0);
            // base fitness caps the condition and raises the injury risk below 70 (Konzept 3.3)
            if let Some(m) = deckel[k] { pl.fit = pl.fit.min(m); }
            let gf_faktor = if pl.gf > 0.0 && pl.gf < unter { 1.0 + (unter - pl.gf) / 100.0 } else { 1.0 };
            // entwicklung.json injury.train_week 0.0015 per week, scaled by the unit and the program
            let pinj = 0.0015 / 6.0 * inj * match pl.program { 1 => 1.1, 2 => 1.3, _ => 1.0 } * med * gf_faktor;
            if r < pinj { hurt.push(p); }
        }
        self.tr_verletzt += hurt.len() as u32;
        for p in hurt {
            let w = [0.45, 0.3, 0.15, 0.07, 0.03];
            let days = [3u16, 7, 14, 28, 60][self.world.rng.weighted(&w)];
            let pl = &mut self.world.players[p];
            pl.inj = days;
            pl.inj_name = injury_names(days)[0].to_string();
            let eintrag = (self.world.season, self.world.day, pl.inj_name.clone(), days);
            pl.profil.verletzungen.push(eintrag);
            let n = pl.name.clone();
            let t = format!("{}\n\n{}", crate::tf!("ui.game.hat_sich_im_training_verletzt", pl.name, pl.inj_name, days), self.verletzung_gruende(p, false, days));
            self.msg(1, crate::inhalt::t("med.vg.absender"), &crate::tf!("ui.game.trainingsverletzung", n), &t, Action::None); self.meldung_art(crate::pages::Art::Verletzung, vec![p]);
        }
        let c = &mut self.world.clubs[self.own];
        c.fam = (c.fam + fam).clamp(0.0, 100.0);
    }

    fn week(&mut self) {
        let hard = self.plan.iter().filter(|&&u| u == 5).count() >= 2;
        self.hard_streak = if hard { self.hard_streak + 1 } else { 0 };
        if self.hard_streak >= 3 {
            let co = self.co_name.clone();
            self.msg(1, &co, crate::inhalt::t("med.vg.titel.belastung"), crate::inhalt::t("ui.game.drei_harte_wochen_am_stueck"), Action::None);
        }
        // the other clubs keep the season step; the own squad and academy develop per ability after the academy week (entwicklung.rs)
        self.world.develop_week(self.own);
        self.jobangebote_woche();
        self.vorvertraege_woche();
        self.akademie_woche();
        self.entwicklung_woche();
        self.st_woche();
        self.st_vorschlaege();
        self.trainingsbericht();
        self.manager_woche();
        self.ki_woche();
        self.post_woche();
        self.spielerrat_woche();
        self.cotrainer_woche();
        for c in 0..self.world.clubs.len() {
            if c != self.own { let f = &mut self.world.clubs[c].fam; *f = (*f + 1.5).min(85.0); }
        }
        self.world.refresh_levels();
        self.world.update_values();
        self.scouting_week();
    }

    // ------------------------------------------------------------ youth (akademie.json)
    fn academy_intake(&mut self, n: usize, start: bool) {
        // same numbers as the Talenttag (jugendakademie.toml [jahrgang]): most average, few talents, now and then a jewel
        let js = self.jugendstaerke(self.own).wert;
        let (gw, pt, jp, juwel) = (crate::jugend::reihe("jahrgang.gw"), crate::jugend::reihe("jahrgang.pot"), crate::jugend::reihe("jahrgang.juwel_pot"), crate::jugend::reihe("jahrgang.juwel"));
        let pos = [crate::id("GK"), crate::id("CB"), crate::id("CB"), crate::id("LB"), crate::id("RB"), crate::id("DMF"), crate::id("CMF"), crate::id("CMF"), crate::id("AMF"), crate::id("LMF"), crate::id("RMF"), crate::id("LWF"), crate::id("RWF"), crate::id("CF"), crate::id("CF")];
        let mut namen: std::collections::HashMap<u16, (Vec<String>, Vec<String>)> = Default::default();
        for i in 0..n {
            let sk = format!("start:{}:{}:{i}", self.club().id, self.world.season);
            let nat = self.jugend_nat(self.own, &sk);
            let name = self.name_aus(namen.entry(nat).or_insert_with(|| self.namen_von(nat)), &sk);
            let r = &mut self.world.rng;
            let age = if start { r.int(15, 18) as u8 } else { 15 };
            let talent = 1.0;
            // the older ones of the start group are about 3 a year ahead (Fabian 24.11.: they all were alike)
            let ovr = (gw[0] + gw[1] * js + (age as f32 - 15.5) * 3.0 + r.gauss(gw[2])).clamp(gw[3], gw[4] + 6.0);
            let pot = if r.f() < juwel[0] + juwel[1] * js { (jp[0] + r.gauss(jp[1])).min(jp[2]) } else { (pt[0] + pt[1] * js + r.gauss(pt[2])).clamp(pt[3], pt[4]) }.max(ovr + 3.0);
            let ps = pos[(i + r.int(0, 14) as usize) % pos.len()].to_string();
            let persona = (r.next() % 5) as u8;
            let (attrs, pp) = self.world.template_attrs(&ps, ovr);
            let id = self.world.new_id();
            let p = Player { nat,
                id, name, pos: ps, age, ovr, pot, talent,
                no: 0, team: self.own as i32, youth: if age <= 16 { 17 } else { 19 }, form: 0.0, fit: 100.0, gf: 0.0, morale: 60.0, inj: 0, inj_name: String::new(), susp: 0,
                wage: 0.0, contract_end: self.world.season + 2, value: 0.0, apps: 0, goals: 0, assists: 0, rsum: 0.0, rn: 0, yellow: 0, last: vec![], know: 1.0,
                listed: false, persona, unhappy_days: 0, benched: 0, promise: 0, program: 0, joined: 0, talk_day: 0, ps: Default::default(), leihe: None, ent: Default::default(), st: Default::default(),
                attrs, pp, style: 0, skills: 0, db: false, profil: Default::default(),
            };
            let idx = self.world.players.len();
            self.world.players.push(p);
            self.world.clubs[self.own].squad.push(idx);
        }
    }
    fn make_youth_leagues(&mut self) {
        let l = self.league();
        let mut others: Vec<usize> = self.world.leagues[l].teams.iter().copied().filter(|&t| t != self.own).collect();
        others.truncate(11);
        let own_name = self.club().name.clone();
        for (k, (tag, ratio)) in [(crate::id("U17"), 0.68f32), (crate::id("U19"), 0.72f32)].iter().enumerate() {
            let mut rows = vec![YRow { name: crate::tf!("ui.farm.text", own_name, tag), own: true, ..Default::default() }];
            for &o in &others {
                let s = self.world.clubs[o].level * ratio + self.world.rng.gauss(2.0);
                // a real squad of the academy: names of its country, ages of the team, strength around the team's
                let namen = self.namen_von(self.world.land_nat(self.world.clubs[o].league));
                let pos = ["GK", "GK", "CB", "CB", "CB", "LB", "RB", "DMF", "CMF", "CMF", "AMF", "LMF", "RMF", "LWF", "RWF", "CF", "CF", "CMF"]; // roh
                let kader: Vec<YSpieler> = pos.iter().enumerate().map(|(i, p)| {
                    let name = self.name_aus(&namen, &format!("jugend:{}:{}:{k}:{i}", self.world.clubs[o].id, self.world.season)); // roh
                    let r = &mut self.world.rng;
                    let age = if k == 0 { r.int(15, 17) as u8 } else { r.int(17, 19) as u8 };
                    let ovr = (s + r.gauss(4.0)).clamp(35.0, 72.0);
                    YSpieler { name, pos: p.to_string(), age, ovr, pot: (ovr + 6.0 + r.gauss(6.0).abs() * 1.5).min(92.0), tore: 0, spiele: 0 }
                }).collect();
                rows.push(YRow { name: format!("{} {}", self.world.clubs[o].name, tag), own: false, str_: s, kader, verein: o, ..Default::default() });
            }
            self.youth[k] = YouthLeague { name: format!("{tag}-Liga"), rows, round: 0, last: String::new() };
        }
    }
    /// Sign a player of a rival academy (FC-26 wish): a small fee to his club, he joins the own academy as a real player.
    pub fn jugend_verpflichten(&mut self, k: usize, row: usize, i: usize) -> Result<String, String> {
        let Some(s) = self.youth.get(k).and_then(|y| y.rows.get(row)).and_then(|r| r.kader.get(i)).cloned() else { return Err(String::new()) };
        let club = self.youth[k].rows[row].verein;
        let fee = crate::world::round_money(crate::regeln::f("akademie", "fremd.abloese_je_pot") as f64 * s.pot as f64 * self.skala());
        if fee > self.transfer_budget { return Err(crate::inhalt::t("ui.transfer.die_leihgebuehr_sprengt_das_transferbudg").into()); }
        self.transfer_budget -= fee;
        self.book(crate::inhalt::t("ui.game.abloesen_kauf"), -fee);
        if club < self.world.clubs.len() { self.world.clubs[club].balance += fee; }
        let (attrs, pp) = self.world.template_attrs(&s.pos, s.ovr);
        let id = self.world.new_id();
        let nat = self.world.land_nat(self.world.clubs[club.min(self.world.clubs.len() - 1)].league);
        let talent = ((s.pot - s.ovr) / 15.0).clamp(0.3, 2.0);
        let p = Player { nat, id, name: s.name.clone(), pos: s.pos.clone(), age: s.age, ovr: s.ovr, pot: s.pot, talent, no: 0, team: self.own as i32, youth: if s.age <= 16 { 17 } else { 19 }, form: 0.0, fit: 100.0, gf: 0.0, morale: 65.0, inj: 0, inj_name: String::new(), susp: 0,
            wage: 0.0, contract_end: self.world.season + 2, value: 0.0, apps: 0, goals: 0, assists: 0, rsum: 0.0, rn: 0, yellow: 0, last: vec![], know: 1.0,
            listed: false, persona: (id % 5) as u8, unhappy_days: 0, benched: 0, promise: 0, program: 0, joined: self.abs(), talk_day: 0, ps: Default::default(), leihe: None, ent: Default::default(), st: Default::default(),
            attrs, pp, style: 0, skills: 0, db: false, profil: Default::default() };
        let idx = self.world.players.len();
        self.world.players.push(p);
        self.world.clubs[self.own].squad.push(idx);
        self.youth[k].rows[row].kader.remove(i);
        Ok(crate::tf!("ui.jugend.fremd.verpflichtet", s.name, money(fee)))
    }
    pub fn youth_strength(&self, k: usize) -> f32 {
        let g = if k == 0 { 17 } else { 19 };
        let mut o: Vec<f32> = self.youth_squad(g).iter().map(|&p| self.world.players[p].ovr).collect();
        o.sort_by(|a, b| b.partial_cmp(a).unwrap());
        if o.is_empty() { return 35.0; }
        o.iter().take(11).sum::<f32>() / 11.0 + 0.7 // missing players count as 0; akademie level_bonus 0.7
    }
    /// One round of the U17 and U19 leagues (Sundays): akademie.json lambda 1.3, slope 0.06, home 0.2.
    pub(crate) fn youth_round(&mut self) {
        for k in 0..2 {
            let own_str = self.youth_strength(k);
            let n = self.youth[k].rows.len();
            if n < 2 { continue; }
            let round = self.youth[k].round as usize;
            let mut idx: Vec<usize> = (0..n).collect();
            let fixed = idx.remove(0);
            idx.rotate_left(round % (n - 1));
            idx.insert(0, fixed);
            for i in 0..n / 2 {
                let (h, a) = (idx[i], idx[n - 1 - i]);
                let sh = if self.youth[k].rows[h].own { own_str } else { self.youth[k].rows[h].str_ };
                let sa = if self.youth[k].rows[a].own { own_str } else { self.youth[k].rows[a].str_ };
                let lh = 1.3 * (0.06 * (sh - sa) + 0.2).exp();
                let la = 1.3 * (0.06 * (sa - sh)).exp();
                let hg = self.world.rng.poisson(lh).min(9) as i32;
                let ag = self.world.rng.poisson(la).min(9) as i32;
                for (x, gf, ga) in [(h, hg, ag), (a, ag, hg)] {
                    let r = &mut self.youth[k].rows[x];
                    r.p += 1; r.gf += gf; r.ga += ga;
                    if gf > ga { r.w += 1; r.pts += 3; } else if gf == ga { r.d += 1; r.pts += 1; } else { r.l += 1; }
                }
                // the rival academies' scorers (every match)
                for (x, tore) in [(h, hg), (a, ag)] {
                    if self.youth[k].rows[x].own || self.youth[k].rows[x].kader.is_empty() { continue; }
                    let w: Vec<f32> = self.youth[k].rows[x].kader.iter().map(|s| match part_of(&s.pos) { 0 => 0.0, 1 => 0.5, 2 => 1.5, _ => 3.5 }).collect();
                    for _ in 0..tore { let i = self.world.rng.weighted(&w); self.youth[k].rows[x].kader[i].tore += 1; }
                    for s in self.youth[k].rows[x].kader.iter_mut().take(14) { s.spiele += 1; }
                }
                if self.youth[k].rows[h].own || self.youth[k].rows[a].own {
                    let goals = if self.youth[k].rows[h].own { hg } else { ag };
                    let sq = self.youth_squad(if k == 0 { 17 } else { 19 });
                    let mut scorers = vec![];
                    for _ in 0..goals {
                        if sq.is_empty() { break; }
                        let w: Vec<f32> = sq.iter().map(|&p| match part_of(&self.world.players[p].pos) { 0 => 0.0, 1 => 0.5, 2 => 1.5, _ => 3.5 }).collect();
                        let p = sq[self.world.rng.weighted(&w)];
                        self.world.players[p].goals += 1;
                        scorers.push(self.world.players[p].name.clone());
                    }
                    for &p in sq.iter().take(14) { self.world.players[p].apps += 1; }
                    let (hn, an) = (self.youth[k].rows[h].name.clone(), self.youth[k].rows[a].name.clone());
                    self.youth[k].last = crate::tf!("ui.game.text_2", hn, an, hg, ag, if scorers.is_empty() { String::new() } else { crate::tf!("ui.game.tore", scorers.join(", ")) });
                }
            }
            self.youth[k].round += 1;
            self.youth[k].rows.sort_by(|a, b| (b.pts, b.gf - b.ga, b.gf).cmp(&(a.pts, a.gf - a.ga, a.gf)));
        }
    }
    pub fn promote(&mut self, p: usize) {
        let lv = self.world.leagues[self.league()].level;
        let season = self.world.season;
        let pl = &mut self.world.players[p];
        pl.youth = 0;
        pl.contract_end = season + 2;
        pl.wage = fair_wage(pl, lv);
        let n = pl.name.clone();
        self.psyche_an();
        self.moral_wirkung(p, 8.0, 0, crate::inhalt::t("ui.game.zu_den_profis_hochgezogen"));
        if self.vision() == crate::id("traditionell") { self.klima(0.0, 0.0, 2.0, crate::inhalt::t("ui.game.ein_eigengewaechs_rueckt_zu_den")); }
        self.news(&crate::tf!("ui.game.rueckt_zu_den_profis_auf", n), crate::inhalt::t("ui.game.ein_eigengewaechs_bekommt_seine_chance"));
    }
    pub fn release(&mut self, p: usize) {
        let c = self.world.players[p].team;
        if c >= 0 { self.world.clubs[c as usize].squad.retain(|&x| x != p); }
        let pl = &mut self.world.players[p];
        pl.team = -1;
        pl.youth = 0;
        pl.listed = false;
        self.xi.retain(|&x| x != p);
    }

    // ------------------------------------------------------------ scouting (stab.json scouting, fuzz)
    /// The one check "this player is ours", for every page: first team, academy and U23 (all in our club), and our
    /// players loaned to other clubs. Agreed signings join at once in this build, so they are in the club already.
    pub fn gehoert_uns(&self, p: usize) -> bool {
        let pl = &self.world.players[p];
        pl.team == self.own as i32 || pl.leihe.as_ref().is_some_and(|l| l.von == self.own) || (pl.team >= 0 && self.world.zweite_von(self.own) == Some(pl.team as usize))
    }
    pub fn know_of(&self, p: usize) -> f32 {
        if self.gehoert_uns(p) { 1.0 } else { self.world.players[p].know }
    }
    pub fn report_text(&self, p: usize) -> String {
        let pl = &self.world.players[p];
        let (a, b) = self.ovr_range(p);
        let (pa, pb) = self.pot_range(p);
        let club = if pl.team >= 0 { self.world.clubs[pl.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
        let mb = self.moneyball(p);
        let mbt = if mb >= 4.0 { crate::inhalt::t("ui.game.fuer_das_was_er_kann") } else if mb <= -4.0 { crate::inhalt::t("ui.game.sein_preis_ist_eher_hoch") } else { crate::inhalt::t("ui.game.preis_und_leistung_passen_zusammen") };
        let st = crate::scouting::sterne_text(crate::scouting::sterne((a + b) / 2.0));
        let ch = if self.know_of(p) >= crate::regeln::f("scouting", "wissen.eindruck_ab") / 100.0 && pl.ps.an { crate::mensch::typen(pl).first().map(|t| crate::tf!("ui.game.typ", crate::mensch::typ_name(t))).unwrap_or_default() } else { String::new() };
        crate::tf!("ui.game.jahre_einschaetzung_staerke_potenzial_0", pl.name, pos_de(&pl.pos), pl.age, club, st, a, b, pa, pb, ch, mbt, ideas_t()[self.tactic.idea()], fit_text(self.system_fit(p)), money(pl.value))
    }
    /// Moneyball: strength points more than his price suggests (26-year-old value curve; estimated rule).
    pub fn moneyball(&self, p: usize) -> f32 {
        let pl = &self.world.players[p];
        let v = pl.value.max(10000.0).log10() as f32;
        let curve: [(f32, f32); 10] = [(50., 4.331), (55., 4.572), (60., 5.341), (65., 6.11), (70., 6.694), (75., 7.211), (80., 7.619), (85., 7.774), (90., 8.135), (95., 8.282)];
        let mut ovr_for_value = 50.0;
        for w in curve.windows(2) {
            let ((a, fa), (b, fb)) = (w[0], w[1]);
            if v >= fa && v <= fb { ovr_for_value = a + (b - a) * (v - fa) / (fb - fa); }
            if v > fb { ovr_for_value = b; }
        }
        let (lo, hi) = self.ovr_range(p);
        (lo + hi) / 2.0 - ovr_for_value
    }
    pub fn system_fit(&self, p: usize) -> f32 { idea_fit(&self.world.players[p].pos, self.tactic.idea()) }

    // ------------------------------------------------------------ transfers and negotiations (verhandlung.py, transfers.json markt.price)
    pub fn player_rep(&self, p: usize) -> f32 { ((self.world.players[p].ovr - 58.0) * 3.0).clamp(0.0, 100.0) }
    pub fn can_open(&self, p: usize) -> Result<(), String> {
        let pl = &self.world.players[p];
        if self.negs.iter().any(|n| n.pid == p && n.stage < 2) { return Err(crate::inhalt::t("ui.game.es_laeuft_schon_eine_verhandlung").into()); }
        if pl.team == self.own as i32 {
            if let Some(v) = self.vorvertraege.iter().find(|v| v.pid == p) { return Err(crate::tf!("vv.schon_weg", pl.name.clone(), self.world.clubs[v.nach].name.clone())); }
            // contract talks with our own players are always possible (Fabian 24.11.: no 18-month limit)
            return Ok(());
        }
        if pl.team == -2 { return Err(crate::inhalt::t("ui.game.er_hat_seine_karriere_beendet").into()); }
        if let Some(g) = self.kaderregel_verstoss(self.own, p) { return Err(g); }
        if pl.team >= 0 && self.verkaufsbereitschaft(p) <= 0.05 { return Err(crate::tf!("ui.game.ist_unverkaeuflich_sein_verein_gibt", pl.name)); }
        if self.player_rep(p) - self.club().rep > 45.0 { return Err(crate::tf!("ui.game.winkt_ab_dein_verein_ist", pl.name)); }
        Ok(())
    }
    /// Character of a club at the table: fixed per club (transfers.py club_style: 20 % hart, 20 % locker).
    pub fn club_charakter(&self, c: usize) -> &'static str {
        let r = (crate::mensch::zufall(&["2026", crate::id("style"), &self.world.clubs[c].id.to_string()]) * 10.0) as u32;
        if r < 2 { crate::id("hardliner") } else if r >= 8 { crate::id("pragmatiker") } else { crate::id("fair") }
    }
    /// Character of a player's side at the table (vdialog.py CHAR_SPIELER by persona).
    pub fn spieler_charakter(&self, p: usize) -> &'static str { [crate::id("fair"), crate::id("hardliner"), crate::id("pragmatiker"), crate::id("fair"), crate::id("pragmatiker")][self.world.players[p].persona as usize % 5] }
    /// The player's wage demand: own players by werte.py:122 (§8), others by verhandlung.py:168 (§9 step 4).
    pub fn forderung(&mut self, p: usize) -> f64 {
        self.psyche_an();
        let pl = &self.world.players[p];
        let gier = if pl.persona == 1 { 70.0 } else { 40.0 };
        let own = pl.team == self.own as i32;
        let rolle = if own { pl.ps.rolle } else { 4 };
        let wunsch = crate::verhandlung::wunschgehalt(pl, self.world.leagues[self.league()].level, rolle);
        let v = if own {
            crate::verhandlung::forderung(wunsch, crate::mensch::cw(pl, crate::mensch::EGO), pl.morale, crate::mensch::cw(pl, crate::mensch::LOYAL), pl.ps.bez, self.vertrauen(p), gier)
        } else {
            let anziehung = (0.5 + (self.club().rep - self.player_rep(p)) / 100.0).clamp(0.0, 1.0);
            wunsch * (1.0 + gier / 100.0 * 0.15 - (anziehung - 0.5) * 0.2) as f64
        } * self.berater_forderung(p) * (1.0 + self.world.players[p].ps.anfrage_plus as f64);
        // character can push the demand down, but not far under what he earns today
        let pl = &self.world.players[p];
        let v = v.max(pl.wage * crate::verhandlung::aufschlag(pl.age) * 0.9);
        if v >= 1e6 { (v / 100000.0).round() * 100000.0 } else { (v / 5000.0).round() * 5000.0 }
    }
    /// A club that would want the player too: bigger than ours and not his, for players good enough for it.
    pub fn rivale(&self, p: usize) -> Option<usize> {
        let pl = &self.world.players[p];
        let r = |k: &str| crate::regeln::f("transfers", &format!("rivale.{k}"));
        if crate::mensch::zufall(&["rivale", &pl.id.to_string(), &(self.abs() / 30).to_string()]) >= r("chance") * ((pl.ovr - 70.0) / 15.0).clamp(0.0, 1.0) { return None; } // roh
        let eigen = self.club().rep;
        let mut k: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| c != self.own && c as i32 != pl.team && self.world.clubs[c].rep > eigen && self.world.clubs[c].level <= pl.ovr + 3.0 && self.world.clubs[c].level >= pl.ovr - 6.0).collect();
        k.sort_by(|a, b| self.world.clubs[*b].rep.partial_cmp(&self.world.clubs[*a].rep).unwrap());
        k.into_iter().next()
    }
    pub fn open_neg(&mut self, p: usize) -> Result<usize, String> {
        // outside the window a player with an expiring contract can still be signed for the summer
        if self.in_window().is_none() && self.vorvertrag_moeglich(p) { return self.vorvertrag_oeffnen(p); }
        self.can_open(p)?;
        // another club's new signing or loan player is not for sale before the next window (Fabian 24.11.)
        if self.world.players[p].team >= 0 && self.world.players[p].team != self.own as i32 { if let Some(g) = self.verkauf_gesperrt(p) { return Err(g); } }
        if let Some(x) = self.mensch.neg_sperre.iter().find(|x| x.0 == p && self.abs() < x.1) { return Err(crate::tf!("ui.game.nach_dem_scheitern_will_die", x.1 - self.abs())); }
        let pl = &self.world.players[p];
        let renew = pl.team == self.own as i32;
        let free = pl.team < 0;
        let ask = if renew || free { 0.0 } else { self.ask_price(p) };
        let years = crate::verhandlung::wunschlaufzeit(pl.age).min(5);
        let text = if renew { crate::tf!("ui.game.gespraeche_ueber_einen_neuen_vertrag", pl.name) } else if free { crate::tf!("ui.game.ist_vereinslos_wir_verhandeln_direkt", pl.name) } else { crate::tf!("ui.game.der_verein_erwartet_ein_ernsthaftes", pl.name) };
        let ends = self.abs() + crate::verhandlung::FRIST_TAGE;
        let need = self.forderung(p);
        // a rival who wants him too (Fabian 23.11.): a bigger club is named and the selling club asks more
        let (ask, text) = match (renew || free).then_some(()).map_or_else(|| self.rivale(p), |_| None) {
            Some(r) => { let rn = self.world.clubs[r].name.clone(); (round_money(ask * (1.0 + crate::regeln::f("transfers", "rivale.aufschlag") as f64)), format!("{text} {}", crate::tf!("ui.tempo.rivale", rn))) }
            None => (ask, text),
        };
        let ph = if renew || free { self.spieler_phase(p, need) } else { self.vereins_phase(p, ask) };
        self.negs.push(Neg { pid: p, renew, stage: if renew || free { 1 } else { 0 }, round: 0, ask, fee: (ask * 0.8 / 50000.0).round() * 50000.0, counter: 0.0, wage: (need * 0.9 / 5000.0).round() * 5000.0, need, years, state: NegState::Counter, reply_day: 0, ends, text, ph: Some(ph), ton: 0, handlung: 0, letzt: false, argument: 0, argumente: vec![], gefragt: false, tausch: false, haken: 0, haken_bis: 0, abschlag: 0.0, med: false, gate: false, verkauf: false, kaeufer: 0, gesendet: String::new(), kl: Default::default(), zeit: 0, vorvertrag: false, liegt_bis: 0, verlauf: vec![], verein: i32::MIN });
        if let Some(n) = self.negs.last_mut() { n.verein = self.world.players[p].team; }
        Ok(self.negs.len() - 1)
    }
    fn vereins_phase(&self, p: usize, ask: f64) -> crate::verhandlung::Phase {
        let c = self.world.players[p].team.max(0) as usize;
        let rel = self.mensch.vereine.iter().find(|x| x.0 == c).map(|x| x.1).unwrap_or(0.0);
        let lage = if self.world.clubs[c].balance < 0.0 { -0.7 } else if !self.world.players[p].listed && self.world.players[p].ovr >= self.world.clubs[c].level + 2.0 { 1.0 } else { 0.0 };
        crate::verhandlung::Phase::neu(ask, self.club_charakter(c), rel, lage)
    }
    fn spieler_phase(&self, p: usize, need: f64) -> crate::verhandlung::Phase {
        let rel = ((self.world.players[p].morale - 50.0 + self.sond_stimmung(p)) / 25.0).round().clamp(-2.0, 2.0);
        crate::verhandlung::Phase::neu(need, self.spieler_charakter(p), rel, 0.0)
    }
    /// Send the offer of the current phase; the answer comes at once (runden.py senden), a day of thinking at most.
    /// Sends the offer: our line goes into the chat, the answer comes back as the other side's line.
    pub fn submit_neg(&mut self, i: usize) {
        let zeile = self.neg_unsere_zeile(i);
        // the answer takes days (Fabian 23.11.): the offer lies on the table, the day it is due the round is played
        if self.negs[i].liegt_bis > 0 { return; }
        if self.negs[i].round == 0 && !self.negs[i].renew && !self.negs[i].verkauf { self.presse_nachfrage(self.negs[i].pid); }
        if self.verhandlungstempo > 0 && self.negs[i].stage < 2 {
            let now = self.abs();
            let tage = self.frist(match self.verhandlungstempo { 1 => 2, _ => 2 + (crate::mensch::zufall(&["tempo", &self.negs[i].pid.to_string(), &now.to_string()]) * 3.0) as u32 }); // roh
            // the window closes today: they answer today
            if tage == 0 { self.negs[i].verlauf.push((0, zeile)); self.submit_neg_runde(i); return; }
            let n = &mut self.negs[i];
            n.liegt_bis = now + tage;
            n.ends = n.ends.max(n.liegt_bis + 7);
            n.verlauf.push((0, zeile));
            n.text = crate::tf!("ui.tempo.liegt", crate::date::short(self.world.season, self.world.day + tage));
            return;
        }
        let vorher = (self.negs[i].gesendet.clone(), self.negs[i].state.clone());
        self.submit_neg_runde(i);
        if i < self.negs.len() && (self.negs[i].gesendet != vorher.0 || self.negs[i].state != vorher.1) {
            self.negs[i].verlauf.push((0, zeile));
            self.neg_verlauf_pflegen(i);
        }
    }
    /// Our offer in words: the fee, or wage, years, clauses and argument; the tone and the step at the table.
    pub fn neg_unsere_zeile(&self, i: usize) -> String {
        let n = &self.negs[i];
        let t = crate::inhalt::t;
        let mut z = if n.stage == 0 { crate::tf!("ui.chat.wir_abloese", money(n.fee)) } else { crate::tf!("ui.chat.wir_gehalt", money(n.wage), n.years) };
        if n.stage == 1 && !n.verkauf {
            let mut kl = vec![];
            if n.kl.ausstieg > 0 { kl.push(t("ui.chat.k.ausstieg")); }
            if n.kl.steigerung > 0 { kl.push(t("ui.chat.k.steigerung")); }
            if n.kl.tor > 0 || n.kl.einsatz > 0 || n.kl.titel > 0 { kl.push(t("ui.chat.k.praemien")); }
            if !kl.is_empty() { z = format!("{z} {}", crate::tf!("ui.chat.dazu", kl.join(", "))); }
            if n.argument > 0 { z = format!("{z} {}", t(&format!("ui.chat.arg.{}", n.argument))); }
        }
        if n.letzt { z = format!("{z} {}", t("ui.chat.letztes")); }
        z
    }
    /// The other side's newest words into the chat (also answers that come later, on a day or from drama).
    pub fn neg_verlauf_pflegen(&mut self, i: usize) {
        let n = &self.negs[i];
        if n.text.is_empty() { return; }
        let mut neu = n.text.clone();
        if n.state == NegState::Counter && n.counter > 0.0 && n.round > 0 && !neu.contains(&money(n.counter)) {
            neu = format!("{neu} {}", crate::tf!(if n.stage == 0 { "ui.chat.gegen_abloese" } else { "ui.chat.gegen_gehalt" }, money(n.counter)));
        }
        if n.verlauf.iter().rev().find(|x| x.0 == 1).map_or(true, |x| x.1 != neu) { self.negs[i].verlauf.push((1, neu)); }
    }
    fn submit_neg_runde(&mut self, i: usize) {
        use crate::verhandlung::Reaktion as R;
        let now = self.abs();
        let n = self.negs[i].clone();
        let p = n.pid;
        let name = self.world.players[p].name.clone();
        let Some(mut ph) = n.ph.clone() else { return };
        // the clauses' worth counts like wage for the player, like fee for a buyer (klauseln.rs); answers come back without it
        let kv = self.klausel_kv(&n);
        let wert = if n.verkauf { -n.fee } else if n.stage == 0 { n.fee } else { n.wage } + kv;
        let schluessel = format!("{}|{}|{}|{}|{}|{}|{:?}|{}", n.stage, wert, n.ton, n.handlung, n.letzt, n.years, n.kl, n.argument);
        if schluessel == n.gesendet && n.state == NegState::Counter { return; } // the same proposal twice: same answer, nothing changes
        self.negs[i].gesendet = schluessel;
        // hard conditions of the player's side (§9 step 4): contract length; morale veto on renewals (§8)
        if n.stage == 1 {
            let wunsch = crate::verhandlung::wunschlaufzeit(self.world.players[p].age) as i32;
            let tol = if crate::mensch::cw(&self.world.players[p], crate::mensch::LOYAL) > 60.0 { 2 } else { 1 };
            if (n.years as i32 - wunsch).abs() > tol {
                ph.runde += 1;
                ph.geduld = (ph.geduld - crate::verhandlung::charakter(&ph.charakter).3).max(0.0);
                let nn = &mut self.negs[i];
                nn.ph = Some(ph);
                nn.round += 1;
                nn.text = crate::tf!("ui.game.will_eine_andere_laufzeit_etwa", name, wunsch);
                return;
            }
            if n.renew {
                let k = |s: &str| crate::regeln::f("gespraeche", &format!("vertrag.{s}"));
                match crate::verhandlung::bewertung(wert / n.need.max(1.0), self.world.players[p].morale) {
                    "veto" => {
                        // werte.py: below morale 25 he blocks every round under 115 % of his demand
                        ph.runde += 1;
                        ph.geduld = (ph.geduld - crate::verhandlung::charakter(&ph.charakter).3).max(0.0);
                        self.moral_wirkung(p, k("ablehnung_stimmung"), 0, crate::inhalt::t("ui.game.vertragsangebot_abgelehnt_er_ist_zu"));
                        let nn = &mut self.negs[i];
                        nn.ph = Some(ph);
                        nn.round += 1;
                        nn.text = crate::tf!("ui.game.ist_so_unzufrieden_dass_er", name);
                        return;
                    }
                    "beleidigt" => {
                        self.moral_wirkung(p, k("beleidigt_stimmung"), 0, crate::inhalt::t("ui.game.ein_vertragsangebot_das_er_als"));
                        self.beziehung(p, k("beleidigt_beziehung"), crate::inhalt::t("ui.game.ein_vertragsangebot_das_er_als"));
                    }
                    _ => {}
                }
            }
        }
        // Konzept 2.6/2.7: a partial yes when only the release clause is missing for a player who insists on it;
        // swapping wage for agent fee makes the offer worth more to the side, the family adviser refuses it
        if n.stage == 1 && !n.verkauf {
            let wichtig_freiheit = crate::berater::prioritaeten(&self.world.players[p])[4] >= 20.0;
            if n.kl.ausstieg == 0 && (self.will_ausstieg(p) && wichtig_freiheit) && wert >= ph.ziel_akt(ph.runde) * 0.97 {
                let nn = &mut self.negs[i];
                nn.round += 1;
                nn.ph = Some(ph);
                nn.text = crate::tf!("ui.argument.teilannahme", name);
                return;
            }
        }
        let wert = if n.stage == 1 && n.tausch && !n.verkauf {
            if crate::berater::TYPEN[crate::berater::typ(&self.world.players[p])] == "familie" {
                ph.geduld = (ph.geduld - 5.0).max(0.0);
                self.negs[i].tausch = false;
                self.negs[i].text = crate::inhalt::t("ui.argument.tausch_nein").into();
                wert
            } else { wert + n.need * crate::regeln::f("berater", "tausch.wert") as f64 }
        } else { wert };
        let mut bonus = if n.stage == 0 { self.scout_bonus(p) } else { self.familie_bonus(p) + self.berater_bonus(p, n.years) };
        // an argument of the coach (Konzept 2.6): fits his priorities or costs patience
        let mut arg_text = String::new();
        if n.stage == 1 && n.argument > 0 {
            let (b, kosten, t) = self.argument_wirkung(p, n.argument, &n.argumente);
            bonus += b;
            ph.geduld = (ph.geduld - kosten).max(0.0);
            arg_text = t;
            let nn = &mut self.negs[i];
            nn.argumente.push(n.argument);
            nn.argument = 0;
        }
        let key = format!("{}|{}|{}", self.world.players[p].id, n.stage, ph.runde);
        let w = |k: u32| crate::mensch::zufall(&["2026", crate::id("verhandlung"), &key, &k.to_string()]);
        let (r, fo) = ph.senden(wert, n.letzt, n.ton as usize, n.handlung as usize, bonus, &w);
        if n.stage == 0 && fo.beziehung != 0.0 { let c = if n.verkauf { n.kaeufer } else { self.world.players[p].team.max(0) as usize }; self.vereinsbeziehung(c, fo.beziehung); }
        if n.stage == 1 && n.renew {
            if fo.stimmung != 0.0 { self.moral_wirkung(p, fo.stimmung, 0, crate::inhalt::t("ui.game.verlauf_der_vertragsverhandlung")); }
            if fo.beziehung != 0.0 { self.beziehung(p, fo.beziehung * 5.0, crate::inhalt::t("ui.game.verlauf_der_vertragsverhandlung")); }
        }
        if fo.abkuehl > 0 { self.mensch.neg_sperre.retain(|x| x.0 != p); self.mensch.neg_sperre.push((p, now + fo.abkuehl)); }
        let nn = &mut self.negs[i];
        nn.round += 1;
        nn.letzt = false;
        nn.ph = Some(ph);
        nn.ends = nn.ends.max(now + 7);
        let wer = if nn.stage == 0 { crate::inhalt::t("ui.game.der_verein") } else { crate::inhalt::t("ui.game.die_spielerseite") };
        let ist_gegen = matches!(r, R::Gegen(_)) && n.stage == 1 && !n.verkauf;
        match r {
            R::Annehmen(v) => { self.neg_einig(i, v - kv); }
            R::Gegen(g) if n.verkauf => { let g = g - kv; nn.counter = round_money(-g); nn.state = NegState::Counter; nn.text = crate::tf!("ui.game.der_kaeufer_bietet_jetzt", money(nn.counter)); }
            R::LetztesWort(g) if n.verkauf => { let g = g - kv; nn.counter = round_money(-g); nn.state = NegState::Counter; nn.text = crate::tf!("ui.game.letztes_wort_des_kaeufers", money(nn.counter)); }
            R::Gegen(g) => { let g = g - kv; nn.counter = round_money(g); nn.state = NegState::Counter; nn.text = crate::tf!("ui.game.macht_einen_gegenvorschlag", wer, money(nn.counter)); }
            R::LetztesWort(g) => { let g = g - kv; nn.counter = round_money(g); nn.state = NegState::Counter; nn.text = crate::tf!("ui.game.letztes_wort_annehmen_oder_lassen", money(nn.counter)); }
            R::Ablehnen(g) => { nn.state = NegState::Counter; nn.text = match g { "nicht_ernst" => crate::tf!("ui.game.nimmt_das_angebot_nicht_ernst", wer), "druck_geplatzt" => crate::inhalt::t("ui.game.der_druck_ist_nach_hinten").into(), _ => crate::tf!("ui.game.lehnt_ab_zu_wenig", wer) }; }
            R::Bedenkzeit => { nn.state = NegState::Waiting; nn.reply_day = now + 1; nn.text = crate::tf!("ui.game.will_eine_nacht_darueber_schlafen", wer); }
            R::Abbruch(g) => {
                if nn.stage == 1 { self.berater_merken(p, -1.0); }
                let nn = &mut self.negs[i];
                nn.stage = 2;
                nn.state = NegState::Rejected;
                nn.text = match g { "geduld_leer" => crate::tf!("ui.game.hat_genug_und_beendet_die", wer), "nicht_ernst" => crate::tf!("ui.game.fuehlt_sich_nicht_ernst_genommen", wer), "druck_geplatzt" => crate::inhalt::t("ui.game.der_druck_ist_nach_hinten_2").into(), _ => crate::inhalt::t("ui.game.das_letzte_angebot_wurde_abgelehnt").into() };
                let t = nn.text.clone();
                self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.game.verhandlung_mit_gescheitert", name), &t, Action::None);
            }
        }
        if !arg_text.is_empty() { let nn = &mut self.negs[i]; nn.text = format!("{arg_text} {}", nn.text); }
        // Fabian 22.11.: the agent may bluff now and then with a made-up offer of a bigger club (konzept 2, question 1);
        // a coach with a good feel for people (Menschenführung ≥ 60) senses it
        if ist_gegen {
            let key = format!("{}|{}", self.world.players[p].id, now);
            // the sly agent bluffs twice as often, the family and players without agent never (berater.rs)
            let bluff = match crate::berater::TYPEN[crate::berater::typ(&self.world.players[p])] { "schlitzohr" => 0.24, "familie" | "keiner" => 0.0, _ => 0.12 };
            if crate::mensch::zufall(&["2026", crate::id("bluff"), &key]) < bluff {
                let own_rep = self.world.clubs[self.own].rep;
                let gross: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.clubs[c].rep > own_rep + 5.0).collect();
                if !gross.is_empty() {
                    let c = gross[(crate::mensch::zufall(&["2026", crate::id("bluffverein"), &key]) * gross.len() as f32) as usize % gross.len()];
                    let mut t = crate::tf!("ui.verhandlung.bluff", self.world.clubs[c].name);
                    if self.mgr_wert(0) >= 60.0 { t = format!("{t} {}", crate::inhalt::t("ui.verhandlung.bluff_erkannt")); }
                    self.negs[i].text = format!("{} {t}", self.negs[i].text);
                }
            }
        }
    }
        /// The person accepts the counter-offer (runden.py annehmen).
    pub fn neg_annehmen(&mut self, i: usize) {
        let Some(g) = self.negs[i].ph.as_ref().and_then(|ph| ph.gegen) else { return };
        self.neg_einig(i, round_money(g));
    }
    /// Ask the counterpart for more time: at most twice, 2 days each (runden.py zeit_max / zeit_tage).
    pub fn neg_zeit(&mut self, i: usize) -> Result<(), String> {
        let tage = self.frist(crate::verhandlung::ZEIT_TAGE);
        let text = if tage < crate::verhandlung::ZEIT_TAGE { crate::tf!("ui.frist.mehr_zeit_fenster", self.frist_text(tage)) } else { crate::tf!("ui.game.die_gegenseite_gibt_dir_tage", tage) };
        let n = &mut self.negs[i];
        if n.zeit >= crate::verhandlung::ZEIT_MAX { return Err(crate::inhalt::t("ui.game.mehr_zeit_gibt_es_nicht").into()); }
        n.zeit += 1;
        n.ends += tage;
        n.text = text;
        Ok(())
    }
    /// Negotiate a club's offer for an own player instead of a plain yes or no (sale in the round model).
    pub fn verkauf_verhandeln(&mut self, p: usize, kaeufer: usize, angebot: f64) -> usize {
        let ch = self.club_charakter(kaeufer);
        let rel = self.mensch.vereine.iter().find(|x| x.0 == kaeufer).map(|x| x.1).unwrap_or(0.0);
        // the buyer's side: target = his offer, pain limit above it (runden.py: richtung −1, negated values)
        let mut ph = crate::verhandlung::Phase::neu(-angebot, ch, rel, 0.0);
        ph.schmerz = -angebot * (1.0 + (crate::verhandlung::charakter(ch).0 * (1.0 + 0.1 * rel)).max(0.01) as f64);
        ph.skala = angebot.max(1.0);
        ph.gegen = Some(-angebot);
        let ends = self.abs() + crate::verhandlung::FRIST_TAGE;
        let text = crate::tf!("ui.game.bietet_fuer_wir_koennen_mehr", self.world.clubs[kaeufer].name, money(angebot), self.world.players[p].name);
        self.negs.push(Neg { pid: p, renew: false, stage: 0, round: 0, ask: angebot, fee: round_money(angebot * 1.15), counter: angebot, wage: 0.0, need: 0.0, years: 0, state: NegState::Counter, reply_day: 0, ends, text, ph: Some(ph), ton: 0, handlung: 0, letzt: false, argument: 0, argumente: vec![], gefragt: false, tausch: false, haken: 0, haken_bis: 0, abschlag: 0.0, med: true, gate: true, verkauf: true, kaeufer, gesendet: String::new(), kl: Default::default(), zeit: 0, vorvertrag: false, liegt_bis: 0, verlauf: vec![], verein: i32::MIN });
        self.negs.len() - 1
    }
    pub fn neg_einig(&mut self, i: usize, v: f64) {
        // he may have signed elsewhere since the last day: then there is nothing to agree (Fabian 24.11.)
        if self.negs_pruefen() && self.negs[i].stage >= 2 { return; }
        let p = self.negs[i].pid;
        let name = self.world.players[p].name.clone();
        if self.negs[i].verkauf {
            let fee = round_money(v.abs());
            let k = self.negs[i].kaeufer;
            let n = &mut self.negs[i];
            n.fee = fee;
            n.stage = 2;
            n.state = NegState::Accepted;
            n.text = crate::tf!("ui.game.einigung_wechselt_fuer", name, money(fee));
            let neg = n.clone();
            if self.world.players[p].team == self.own as i32 {
                let jetzt = self.zahlweise_buchen(fee, neg.kl, true, p, k);
                self.sell(p, k, jetzt);
                self.klauseln_unterschrift(&neg);
            }
            return;
        }
        if self.negs[i].stage == 0 {
            let need = self.forderung(p);
            let ph = self.spieler_phase(p, need);
            let n = &mut self.negs[i];
            n.fee = v;
            n.stage = 1;
            n.state = NegState::Counter;
            n.need = need;
            n.wage = (need * 0.9 / 5000.0).round() * 5000.0;
            n.counter = 0.0;
            n.ph = Some(ph);
            n.text = crate::tf!("ui.game.der_verein_nimmt_abloese_an", money(v), name);
            if let Some(ft) = self.familie_typ(p) { let z = crate::tf!(&format!("ui.familie.{ft}"), name); let nn = &mut self.negs[i]; nn.text = format!("{} {z}", nn.text); }
            let n = &self.negs[i];
            let t = n.text.clone();
            self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.game.abloese_fuer_vereinbart", name), &t, Action::None);
        } else {
            self.negs[i].wage = v;
            let alt = if self.negs[i].renew { self.world.players[p].wage } else { 0.0 };
            if let Err(e) = self.rahmen_pruefen(v, alt) {
                let n = &mut self.negs[i];
                n.state = NegState::Counter;
                n.text = crate::tf!("ui.game.einig_mit_dem_spieler_aber", e);
                return;
            }
            if !self.vor_unterschrift(i) { return; }
            let n = &mut self.negs[i];
            n.wage = v;
            n.stage = 2;
            n.state = NegState::Accepted;
            n.text = crate::inhalt::t("ui.game.einigung_der_vertrag_ist_unterschrieben").into();
            let neg = n.clone();
            self.klauseln_unterschrift(&neg);
            // the adviser: friendlier after a deal, and his fee on a signing (Konzept 2.7)
            self.berater_merken(p, 1.0);
            if !neg.renew { let h = self.berater_honorar(p, neg.fee) * if neg.tausch { crate::regeln::f("berater", "tausch.honorar") as f64 } else { 1.0 }; if h > 0.0 { self.book(crate::inhalt::t("ui.berater.buchung"), -h); } }
            if neg.renew {
                let k = |s: &str| crate::regeln::f("gespraeche", &format!("vertrag.{s}"));
                self.moral_wirkung(p, k("einigung_stimmung"), 0, crate::inhalt::t("ui.game.neuer_vertrag_unterschrieben"));
                self.beziehung(p, k("einigung_beziehung"), crate::inhalt::t("ui.game.neuer_vertrag_unterschrieben"));
            }
            self.complete(neg);
        }
    }
    pub fn vereinsbeziehung(&mut self, c: usize, d: f32) {
        if let Some(x) = self.mensch.vereine.iter_mut().find(|x| x.0 == c) { x.1 = (x.1 + d).clamp(-3.0, 3.0); } else { self.mensch.vereine.push((c, d.clamp(-3.0, 3.0))); }
        let n = self.world.clubs[c].name.clone();
        self.grund(&n, crate::inhalt::t("ui.game.beziehung_zum_verein"), d, crate::inhalt::t("ui.game.verlauf_einer_verhandlung"));
    }
    /// A scout who knows the player well leads the talks: up to +12 % chance (scouting §10).
    pub fn scout_bonus(&self, p: usize) -> f32 { ((self.know_of(p) - 0.6).max(0.0) / 0.4 * 0.12).min(0.12) + (self.mgr_faktor(6) - 1.0) * 0.25 }
    fn negotiations(&mut self) {
        let now = self.abs() + 1;
        // offers whose answer is due today: the round is played now and the answer arrives as a message
        for i in 0..self.negs.len() {
            if self.negs[i].liegt_bis == 0 || now < self.negs[i].liegt_bis || self.negs[i].stage >= 2 { continue; }
            self.negs[i].liegt_bis = 0;
            self.submit_neg_runde(i);
            if i < self.negs.len() {
                self.neg_verlauf_pflegen(i);
                let (name, t) = (self.world.players[self.negs[i].pid].name.clone(), self.negs[i].text.clone());
                let p = self.negs[i].pid;
                self.neg_msg(p, &crate::tf!("ui.tempo.antwort", name), &t);
            }
        }
        for i in 0..self.negs.len() {
            if self.negs[i].stage >= 2 { continue; }
            if self.negs[i].state == NegState::Counter && now > self.negs[i].ends {
                let n = &mut self.negs[i];
                n.stage = 2;
                n.state = NegState::Rejected;
                n.text = crate::inhalt::t("ui.game.die_frist_ist_abgelaufen_ohne").into();
                let (p, t) = (n.pid, n.text.clone());
                let name = self.world.players[p].name.clone();
                self.neg_msg(p, &crate::tf!("ui.game.antwort_im_fall", name), &t);
                continue;
            }
            if self.negs[i].state != NegState::Waiting || self.negs[i].reply_day > now { continue; }
            let p = self.negs[i].pid;
            if self.negs[i].stage == 0 && (self.world.players[p].team < 0 || self.world.players[p].team == self.own as i32) { self.negs[i].stage = 1; self.negs[i].state = NegState::Counter; continue; }
            let key = format!("{}|{}|bedenk", self.world.players[p].id, now);
            let w = crate::mensch::zufall(&["2026", crate::id("verhandlung"), &key]);
            let Some(mut ph) = self.negs[i].ph.clone() else { self.negs[i].state = NegState::Counter; continue };
            let r = ph.nach_bedenkzeit(w);
            self.negs[i].ph = Some(ph);
            let name = self.world.players[p].name.clone();
            match r {
                crate::verhandlung::Reaktion::Annehmen(v) => { self.neg_einig(i, v); }
                crate::verhandlung::Reaktion::Gegen(g) => {
                    let n = &mut self.negs[i];
                    n.counter = round_money(g);
                    n.state = NegState::Counter;
                    n.text = crate::tf!("ui.game.nach_einer_nacht_bedenkzeit_gegenvorschl", money(n.counter));
                    let t = n.text.clone();
                    self.neg_msg(p, &crate::tf!("ui.game.antwort_im_fall", name), &t);
                }
                _ => {}
            }
        }
        self.negs.retain(|n| n.stage < 2 || now < n.ends + 21);
    }
    /// In an open window: the transfers agreed before are carried out (if the player is still where he was).
    pub fn geplante_wechsel(&mut self) {
        if self.in_window().is_none() || self.geplant.is_empty() { return; }
        for n in std::mem::take(&mut self.geplant) {
            let t = self.world.players[n.pid].team;
            if t >= 0 && t != self.own as i32 { self.complete(n); }
        }
    }
    fn complete(&mut self, n: Neg) {
        let p = n.pid;
        let season = self.world.season;
        if n.vorvertrag { self.vorvertrag_abschliessen(&n); return; }
        if n.renew {
            let pl = &mut self.world.players[p];
            pl.ps.anfrage_plus = 0.0;
            pl.wage = n.wage;
            pl.contract_end = season + n.years - 1;
            let nm = pl.name.clone();
            let until = date::START_YEAR as u32 + season + n.years;
            self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.game.verlaengert", nm), &crate::tf!("ui.game.unterschreibt_bis_gehalt_im_jahr", nm, until, money(n.wage)), Action::None);
            let cn = self.club().name.clone();
            self.news(&crate::tf!("ui.game.verlaengert_bei", nm, cn), crate::inhalt::t("ui.game.der_verein_bindet_einen_wichtigen"));
            // the signature as a moment on screen (Fabian 23.11.: an overlay for contract talks too)
            let own = self.own;
            let stimmen = self.feier_stimmen(p, true);
            self.feiern.push(crate::meilensteine::Feier { art: "vertrag".into(), titel: crate::tf!("ui.feier.vertrag_titel", nm.clone()), text: crate::tf!("ui.feier.vertrag_text", nm, until, money(n.wage)), spieler: Some(p), club: Some(own), stimmen, ..Default::default() }); // roh
            return;
        }
        let from = self.world.players[p].team;
        // agreed outside the window: the move happens when the next window opens (Fabian 23.11.)
        if from >= 0 && self.in_window().is_none() {
            let nm = self.world.players[p].name.clone();
            let ab = WINDOWS.iter().find(|(a, _)| *a > self.world.day).map(|(a, _)| date::short(season, *a)).unwrap_or_else(|| date::short(season + 1, WINDOWS[0].0));
            self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("tf.geplant_titel", nm.clone()), &crate::tf!("tf.geplant", nm, money(n.fee), ab), Action::None);
            self.geplant.push(n);
            return;
        }
        if from >= 0 {
            self.world.clubs[from as usize].squad.retain(|&x| x != p);
            self.world.clubs[from as usize].balance += n.fee;
            let jetzt = self.zahlweise_buchen(n.fee, n.kl, false, p, from as usize);
            self.book(crate::inhalt::t("ui.game.abloesen_kauf"), -jetzt);
            self.transfer_budget -= jetzt;
            self.fans_transfer(n.fee, true);
        }
        let day = self.abs();
        let pl = &mut self.world.players[p];
        pl.team = self.own as i32;
        pl.youth = 0;
        pl.wage = n.wage;
        pl.contract_end = season + n.years - 1;
        pl.know = 1.0;
        pl.listed = false;
        pl.joined = day;
        pl.morale = 70.0;
        self.world.clubs[self.own].squad.push(p);
        let nm = self.world.players[p].name.clone();
        let from_name = if from >= 0 { self.world.clubs[from as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
        self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.game.unterschrift", nm), &crate::tf!("ui.game.kommt_von_fuer_abloese_und", nm, from_name, money(n.fee), n.years), Action::None);
        self.feier_neuzugang(p, &from_name, n.fee);
        self.neuzugang_echo(p, n.fee);
        // a star signing sells shirts: the shop makes more in the next three months (the mirror of wv_verkauf)
        let anteil = n.fee / self.umsatz_prognose().max(1.0);
        if anteil >= 0.04 {
            let plus = self.merch_posten().0 * (anteil * 1.5).clamp(0.05, 0.25) * 2.0 / 3.0;
            let monat = self.abs() / 30;
            for k in 1..=3 { self.wi.wv_plan.push((monat + k, plus)); }
            self.msg(3, crate::inhalt::t("vw.figur.finanz"), &crate::tf!("tr.trikots_titel", nm.clone()), &crate::tf!("tr.trikots", nm.clone(), money(plus * 3.0)), Action::None);
        }
        let own_name = self.club().name.clone();
        self.news(&crate::tf!("ui.game.wechselt_zu", nm, own_name), &crate::tf!("ui.game.der_transfer_ist_perfekt_abloese", money(n.fee), n.years));
        self.world.clubs[self.own].fam = (self.world.clubs[self.own].fam - 2.5).max(0.0); // training.json fam.xi_change
        self.nachspiel(p);
        self.world.refresh_levels();
    }
    /// The echo of a signing (Fabian 23.11.: it must feel great to have signed someone): how fans and papers take it,
    /// what he changes in the squad (rank at his position, first eleven before and after, who loses time).
    fn neuzugang_echo(&mut self, p: usize, fee: f64) {
        let lvl = self.club().level;
        let pl = &self.world.players[p];
        let gross = pl.ovr >= lvl + 4.0 || fee >= self.world.revenue(self.own) * 0.15;
        let stufe = if gross { 0 } else if pl.ovr >= lvl - 1.0 { 1 } else { 2 };
        let nm = pl.name.clone();
        let b = self.scout_urteil(p);
        let name = |q: usize| self.world.players[q].name.clone();
        let mut z = vec![crate::tf!(&format!("ui.echo.fans.{stufe}"), nm.clone()), crate::tf!(&format!("ui.echo.presse.{stufe}"), nm.clone(), self.club().name.clone())];
        z.push(crate::tf!("ui.echo.kader", b.platz + 1, b.rang.iter().take(2).map(|x| name(x.0)).collect::<Vec<_>>().join(", "), format!("{:.1}", b.startelf.0), format!("{:.1}", b.startelf.1)));
        if !b.verlierer.is_empty() { z.push(crate::tf!("ui.echo.verlierer", b.verlierer.iter().map(|&q| name(q)).collect::<Vec<_>>().join(", "))); }
        if gross { self.klima(crate::regeln::f("transfers", "echo.fans_gross"), 0.0, 0.0, &crate::tf!("ui.echo.titel", nm.clone())); }
        self.msg(4, crate::inhalt::t("zentrale.k.medien"), &crate::tf!("ui.echo.titel", nm), &z.join(crate::id("\n")), Action::None);
        self.meldung_art(crate::pages::Art::Presse, vec![p]);
    }
    pub fn sell(&mut self, p: usize, to: usize, fee: f64) {
        self.weiterverkauf_abrechnen(p, self.own, fee);
        self.world.clubs[self.own].squad.retain(|&x| x != p);
        self.world.clubs[to].squad.push(p);
        self.world.clubs[to].balance -= fee;
        self.book(crate::inhalt::t("ui.game.abloesen_verkauf"), fee);
        self.transfer_budget += fee * 0.5;
        self.fans_transfer(fee, false);
        let nm = self.world.players[p].name.clone();
        self.wv_verkauf(fee, &nm);
        let lv = self.world.leagues[self.world.clubs[to].league].level;
        let season = self.world.season;
        let abs = self.abs();
        let pl = &mut self.world.players[p];
        pl.team = to as i32;
        pl.joined = abs; // a fresh signing can't be sold on in the same window
        pl.listed = false;
        pl.contract_end = season + contract_years(pl.age) - 1;
        pl.wage = fair_wage(pl, lv);
        pl.know = 0.8;
        let nm = pl.name.clone();
        let (tn, cn) = (self.world.clubs[to].name.clone(), self.club().name.clone());
        self.news(&crate::tf!("zt.eil.verkauf.titel", nm, cn), &crate::tf!("ui.game.wechselt_fuer_zu", nm, money(fee), tn));
        self.xi.retain(|&x| x != p);
        self.world.refresh_levels();
    }

    /// One market round in an open window (transfers.json market step 7; offers chance, fee 0.9-1.3 of the value).
    fn market_round(&mut self) {
        self.angebote_woche();
        let n = self.world.clubs.len();
        let mut moves = 0;
        for _ in 0..80 {
            if moves >= 25 { break; }
            let buyer = self.world.rng.int(0, n as i32 - 1) as usize;
            let seller = self.world.rng.int(0, n as i32 - 1) as usize;
            if buyer == self.own || seller == self.own || seller == buyer || self.world.leagues[self.world.clubs[buyer].league].detail <= 1 || self.world.erste_von(buyer).is_some() || self.world.erste_von(seller).is_some() || self.world.clubs[buyer].squad.len() >= 34 || self.world.clubs[seller].squad.len() <= 20 { continue; }
            let blevel = self.world.clubs[buyer].level;
            let budget = self.world.clubs[buyer].balance * 0.5 + self.world.revenue(buyer) * 0.1;
            let cand = self.world.clubs[seller].squad.iter().copied().filter(|&p| {
                let pl = &self.world.players[p];
                pl.youth == 0 && pl.ovr >= blevel - 3.0 && pl.ovr <= blevel + 8.0 && pl.age <= 32 && pl.value * 1.2 < budget && self.verkauf_gesperrt(p).is_none()
            }).max_by(|a, b| self.world.players[*a].ovr.partial_cmp(&self.world.players[*b].ovr).unwrap());
            let Some(p) = cand else { continue };
            // a partner gives its parent club the first option (farm.rs): to us as a question, to an AI parent directly
            let mut buyer = buyer;
            if let Some(h) = self.world.koops.iter().find(|k| k.partner == seller).map(|k| k.haupt) {
                let abs = self.abs();
                match self.vorkauf.iter().find(|x| x.0 == p).map(|x| x.1) {
                    _ if h != self.own => {}
                    None if self.world.players[p].ovr >= self.club().level - 6.0 => {
                        let (n, sn) = (self.world.players[p].name.clone(), self.world.clubs[seller].name.clone());
                        let opts = vec![(crate::id("kaufen").to_string(), crate::inhalt::t("ui.game.wir_verhandeln").to_string()), (crate::id("verzichten").to_string(), crate::inhalt::t("lb.antwort.kein_interesse").to_string())];
                        self.msg(2, &sn, &crate::tf!("ui.game.vorkaufsrecht", n), &crate::tf!("ui.game.will_verkaufen_und_fragt_uns", sn, n, money(self.world.players[p].value)), Action::Wahl { art: format!("vorkauf:{p}"), bezug: vec![], optionen: opts });
                        self.vorkauf.push((p, abs + 7));
                        continue;
                    }
                    Some(bis) if abs <= bis => continue,
                    _ => {}
                }
                if h != self.own && h != buyer && self.world.clubs[h].squad.len() < 34 && self.world.clubs[h].balance > self.world.players[p].value { buyer = h; }
            }
            let fee = (self.world.players[p].value * self.world.rng.range(1.0, 1.25) as f64 / 50000.0).round() * 50000.0;
            self.weiterverkauf_abrechnen(p, seller, fee);
            self.world.clubs[seller].squad.retain(|&x| x != p);
            self.world.clubs[seller].balance += fee;
            self.world.clubs[buyer].squad.push(p);
            self.world.clubs[buyer].balance -= fee;
            let lv = self.world.leagues[self.world.clubs[buyer].league].level;
            let season = self.world.season;
            let abs = self.abs();
            let pl = &mut self.world.players[p];
            pl.team = buyer as i32;
            pl.joined = abs;
            pl.listed = false;
            pl.contract_end = season + contract_years(pl.age) - 1;
            pl.wage = fair_wage(pl, lv);
            let big = fee >= 8e6 || pl.ovr >= 78.0;
            moves += 1;
            if big {
                let (nm, bn, sn) = (self.world.players[p].name.clone(), self.world.clubs[buyer].name.clone(), self.world.clubs[seller].name.clone());
                self.news(&crate::tf!("ui.game.wechselt_zu", nm, bn), &crate::tf!("ui.game.gibt_fuer_ab", sn, nm, money(fee)));
            }
        }
        self.world.refresh_levels();
    }

    pub fn answer(&mut self, mi: usize, yes: bool, option: u8) {
        let a = self.inbox[mi].action.clone();
        self.inbox[mi].done = true;
        self.inbox[mi].read = true;
        match a {
            Action::TransferOffer { pid, club, fee } => {
                // option 1 / 2: suggest the move or ask him to stay; the offer stays open (haltung.rs)
                if yes && option > 0 && self.world.players[pid].team == self.own as i32 {
                    let r = self.haltung_schieben(pid, club, if option == 1 { 1 } else { -1 });
                    let h = self.haltung_text(pid, club);
                    let m = &mut self.inbox[mi];
                    m.done = false;
                    m.body = format!("{}\n\n{r}\n{h}", m.body.split(crate::id("\n\n")).next().unwrap_or(""));
                    return;
                }
                // he is not sold against his will
                if yes && self.world.players[pid].team == self.own as i32 && !self.haltung(pid, club).zustimmung {
                    self.inbox[mi].done = false;
                    self.inbox[mi].body = format!("{}\n\n{}", self.inbox[mi].body.split(crate::id("\n\n")).next().unwrap_or(""), self.haltung_text(pid, club));
                    return;
                }
                if yes && self.world.players[pid].team == self.own as i32 { self.sell(pid, club, fee); }
                else if self.world.players[pid].team == self.own as i32 {
                    self.freigabe_verweigert(pid);
                    if self.world.players[pid].ps.wunsch { self.streik_pruefen(pid); }
                    if self.world.players[pid].listed { self.moral_wirkung(pid, -4.0, 0, crate::inhalt::t("ui.game.ein_angebot_fuer_ihn_wurde")); }
                }
            }
            Action::Complaint { pid } => {
                // saves from before Paket 3: the old three answers become the conversation options of „spielzeit“
                let key = [crate::id("zusage"), crate::id("leistung"), crate::id("abwimmeln")][option.min(2) as usize];
                self.wahl("gespraech:spielzeit", &[pid], key);
            }
            Action::Wahl { art, bezug, optionen } => {
                let key = optionen.get(option as usize).map(|o| o.0.clone()).unwrap_or_default();
                let r = self.wahl(&art, &bezug, &key);
                self.inbox[mi].body = crate::tf!("ui.game.antwort", self.inbox[mi].body, optionen.get(option as usize).map(|o| o.1.as_str()).unwrap_or(""), r);
            }
            Action::JobOffer { club, wage } => {
                if yes && option == 1 { self.job_verhandeln(club, wage); } else if yes { self.switch_club(club, wage); }
                else if !self.fired {
                    // staying is noticed: the board's trust and the coach's name grow a little
                    let l = |k: &str| crate::regeln::f("laufbahn", &format!("interesse.{k}"));
                    self.klima(0.0, 0.0, l("bleiben_vorstand"), crate::inhalt::t("ui.lockruf.grund"));
                    self.mgr_rep = (self.mgr_rep + l("bleiben_ruf")).min(100.0);
                    self.inbox[mi].body += crate::inhalt::t("ui.lockruf.bleibt");
                }
            }
            Action::Graduate { pid } => { if yes { self.promote(pid); } else { self.release(pid); } }
            Action::Expiring { pid } => {
                // a real choice (vertragsanfrage.rs): talk now, promise with a condition, not now, or no
                match (yes, option) {
                    (true, 1) => { let r = self.bedingung_stellen(pid); self.inbox[mi].body = format!("{}\n\n{r}", self.inbox[mi].body); }
                    (true, 2) => { let r = self.anfrage_spaeter(pid); self.inbox[mi].body = format!("{}\n\n{r}", self.inbox[mi].body); }
                    (true, _) => { let _ = self.open_neg(pid); }
                    _ => self.anlass(pid, crate::id("vertrag_verwehrt"), 1.0),
                }
            }
            _ => {}
        }
    }

    fn expiring_news(&mut self) {
        let season = self.world.season;
        let level = self.club().level;
        for p in self.squad() {
            let pl = &self.world.players[p];
            if pl.contract_end <= season && pl.ovr >= level - 3.0 {
                let nm = pl.name.clone();
                self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("meldung.typ.vertrag_laeuft_aus", nm), crate::inhalt::t("ui.game.sein_vertrag_endet_mit_dieser"), Action::Expiring { pid: p });
            }
        }
    }

    // ------------------------------------------------------------ board (transfers.py _pressure, lagen.toml)
    pub fn confidence(&self) -> f32 {
        self.mensch.vorstand
    }
    pub fn confidence_label(&self) -> &'static str {
        let v = self.confidence();
        if v >= 80.0 { crate::inhalt::t("ui.game.volles_vertrauen") } else if v >= 65.0 { crate::inhalt::t("ui.wort.zufrieden") } else if v >= 50.0 { crate::inhalt::t("ui.wort.abwartend") } else if v >= 35.0 { crate::inhalt::t("ui.wort.skeptisch") } else if v >= 25.0 { crate::inhalt::t("ui.wort.unzufrieden") } else { crate::inhalt::t("ui.game.kurz_vor_der_trennung") }
    }
    fn board_check(&mut self) {
        let l = self.league();
        let table = self.world.table(l);
        if table.iter().map(|r| r.p).min().unwrap_or(0) < 8 { return; }
        for (i, r) in table.iter().enumerate() {
            let gap = (i + 1) as i32 - self.world.clubs[r.club].goal_pos as i32;
            let cl = &mut self.world.clubs[r.club];
            if gap > 0 { cl.pressure = (cl.pressure + (2.0 * gap as f32).min(10.0)).min(100.0); } else { cl.pressure = (cl.pressure - 6.0).max(0.0); }
        }
        for r in table.iter() {
            let c = r.club;
            if c == self.own || self.world.clubs[c].pressure < self.world.clubs[c].patience { continue; }
            let old = self.world.clubs[c].manager.clone();
            let cn = self.world.clubs[c].name.clone();
            self.news(&crate::tf!("tb.ereignis.entlassen", cn, old), crate::inhalt::t("ui.game.die_ergebnisse_blieben_hinter_den"));
            self.world.clubs[c].pressure = 0.0;
            let heute = self.abs();
            self.world.trainer_entlassen(c, heute);
            self.world.clubs[c].manager = "Interimstrainer".into(); // roh
            self.ki_entlassung(c);
            if self.world.clubs[c].rep <= self.mgr_rep + 10.0 && self.world.clubs[c].rep / 100.0 <= self.lizenz_grenze() && self.world.rng.chance(0.4) && self.abs() > self.mgr_since + 60 {
                let wage = (self.world.revenue(c) * 0.004).max(120000.0);
                self.msg(5, &cn, &crate::tf!("ui.game.fragt_an", cn), &crate::tf!("ui.game.sucht_einen_neuen_trainer_und", cn, money(wage)), Action::JobOffer { club: c, wage });
            }
        }
        // own club: story rules vertrauen_sinkt (≤ 38) / vertrauen_steigt (≥ 62) of the board (§12)
        let v = self.mensch.vorstand;
        if v <= 38.0 && !self.warned {
            self.warned = true;
            let (rank, goal, kind) = (self.world.rank_of(self.own), self.club().goal_pos, self.club().goal_kind.clone());
            let (vt, vb) = (self.variante("brief_titel", crate::inhalt::varianten("ui.game.das_vertrauen_sinkt"), "v"), self.variante("brief_text", crate::inhalt::varianten("ui.game.platz_in_der_tabelle_das"), "v")); // roh
            self.msg(5, "Präsidium", crate::inhalt::t(&crate::inhalt::variante_key("ui.game.das_vertrauen_sinkt", vt)), &crate::tf!(&crate::inhalt::variante_key("ui.game.platz_in_der_tabelle_das", vb), rank, kind, goal), Action::None); // roh
            let cn = self.club().name.clone();
            self.news(&crate::tf!("md.anlass.spieler_streik.betreff.1", cn), crate::inhalt::t("ui.game.im_praesidium_waechst_die_ungeduld"));
        }
        if v >= 50.0 { self.warned = false; }
    }
    /// The points target of the ultimatum after each own match: reached saves the job, missed ends it.
    fn ultimatum_spiele_pruefen(&mut self) {
        let Some(ab) = self.ultimatum_ab.filter(|_| self.ultimatum.is_some()) else { return };
        let (n, ziel) = (crate::regeln::f("vereinswelt", "praesidium.ultimatum_spiele") as usize, crate::regeln::f("vereinswelt", "praesidium.ultimatum_punkte") as i32);
        if self.results.len() < ab + n { return; }
        let punkte: i32 = self.results[ab..].iter().take(n).map(|r| if r.starts_with('S') { 3 } else if r.starts_with('U') { 1 } else { 0 }).sum();
        self.ultimatum = None;
        self.ultimatum_ab = None;
        let cn = self.club().name.clone();
        if punkte >= ziel {
            let plus = (crate::regeln::f("vereinswelt", "praesidium.ultimatum_erfuellt_ab") - self.mensch.vorstand).max(0.0);
            self.klima(0.0, 0.0, plus, crate::inhalt::t("ui.game.ultimatum_erfuellt"));
            self.msg(5, "Präsidium", crate::inhalt::t("ui.game.ultimatum_erfuellt"), &crate::tf!("ui.ultimatum.geschafft", punkte, n), Action::None); // roh
            self.news(&crate::tf!("ui.ultimatum.news_gerettet", cn), "");
        } else {
            self.msg(5, crate::inhalt::t("meldung.absender.praesident"), crate::inhalt::t("md.anlass.praesidium_entlassung.betreff"), &crate::tf!("ui.ultimatum.verfehlt", punkte, n, ziel), Action::None);
            self.fire(crate::inhalt::t("ui.game.das_ultimatum_ist_abgelaufen_die"));
        }
    }
    fn ultimatum_end(&mut self) {
        self.ultimatum_ab = None;
        self.ultimatum = None;
        if self.confidence() < crate::regeln::f("vereinswelt", "praesidium.ultimatum_erfuellt_ab") {
            self.msg(5, crate::inhalt::t("meldung.absender.praesident"), crate::inhalt::t("md.anlass.praesidium_entlassung.betreff"), crate::inhalt::t("ui.game.die_frist_ist_abgelaufen_das"), Action::None);
            self.fire(crate::inhalt::t("ui.game.das_ultimatum_ist_abgelaufen_die"));
        } else {
            self.msg(5, "Präsidium", crate::inhalt::t("ui.game.ultimatum_erfuellt"), crate::inhalt::t("ui.game.die_mannschaft_hat_reagiert_das"), Action::None); // roh
        }
    }
    fn fire(&mut self, why: &str) {
        self.fired = true;
        let cn = self.club().name.clone();
        let sev = self.mgr_wage / 2.0; // manager.severance_months 6
        self.mgr_money += sev;
        self.mgr_rep = (self.mgr_rep - 8.0).max(5.0);
        self.history.push(crate::tf!("ui.game.entlassen", date::long(self.world.season, self.world.day), cn));
        let v = self.variante("entlassen_text", crate::inhalt::varianten("ui.game.trennt_sich_von_dir_abfindung"), "e"); // roh
        self.msg(5, "Präsidium", crate::inhalt::t("ui.game.du_bist_entlassen"), &crate::tf!(&crate::inhalt::variante_key("ui.game.trennt_sich_von_dir_abfindung", v), why, cn.clone(), money(sev)), Action::None); // roh
        let coach = self.coach.clone();
        self.news(&crate::tf!("ui.game.entlaesst", cn, coach), why);
        self.world.clubs[self.own].manager = "Interimstrainer".into(); // roh
        let grenze = self.lizenz_grenze();
        let mut c: Vec<usize> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.angebot_erlaubt(c) && self.world.clubs[c].rep <= self.mgr_rep + 5.0 && self.world.clubs[c].rep >= self.mgr_rep - 30.0 && self.world.clubs[c].rep / 100.0 <= grenze).collect();
        for _ in 0..3 {
            if c.is_empty() { break; }
            let i = self.world.rng.int(0, c.len() as i32 - 1) as usize;
            let club = c.swap_remove(i);
            let wage = (self.world.revenue(club) * 0.004).max(120000.0);
            let n = self.world.clubs[club].name.clone();
            self.msg(5, &n, &crate::tf!("transfer.angebot_von", n), &crate::tf!("ui.game.bietet_dir_den_trainerposten_an", n, money(wage)), Action::JobOffer { club, wage });
        }
    }
    /// The coach's contract runs out (laufbahn.toml [vertrag]): with enough confidence the board extends it.
    fn trainervertrag_ende(&mut self) {
        if self.mensch.vorstand >= 45.0 {
            self.mensch.vertrag_bis = self.abs() + (2.0 * crate::date::SEASON_DAYS as f32) as u32;
            self.mgr_wage = self.trainergehalt(self.own);
            let t = crate::tf!("ui.game.das_praesidium_verlaengert_deinen_vertra", money(self.mgr_wage));
            self.msg(5, "Präsidium", crate::inhalt::t("ui.game.vertrag_verlaengert"), &t, Action::None); // roh
        } else {
            self.mgr_wage = 0.0; // no severance: the contract simply ended
            self.fire(crate::inhalt::t("ui.game.dein_vertrag_laeuft_aus_das"));
        }
    }
    pub fn switch_club(&mut self, club: usize, wage: f64) {
        let old = self.own;
        let war_entlassen = self.fired;
        // the old club's mail stays with the old club (Fabian 23.11.)
        self.inbox.clear();
        let oldn = self.club().name.clone();
        if !self.fired { self.history.push(crate::tf!("ui.game.wechsel_nach", date::season_label(self.world.season), oldn, self.world.clubs[club].name)); }
        self.world.clubs[old].manager = "Interimstrainer".into(); // roh
        let ol = self.world.clubs[old].league;
        self.world.leagues[ol].detail = if self.world.leagues[ol].key.contains("D1_LEAGUE") { 1 } else { 2 };
        for p in self.world.clubs[old].squad.clone() { self.world.players[p].youth = 0; }
        let heute = self.abs();
        self.world.trainer_entlassen(club, heute); // the new club's coach makes room
        self.own = club;
        self.fired = false;
        self.warned = false;
        self.ultimatum = None;
        self.ultimatum_ab = None;
        // what was open at the old club closes without penalty: press, crisis plan, story steps, rumours of old players
        self.press = None;
        self.krise = Default::default();
        self.pool_folgen.clear();
        self.mensch.story_folgen.clear();
        self.mensch.anliegen.clear();
        self.ruhe_bis = self.abs() + crate::regeln::f("laufbahn", "wechsel.ruhe_tage") as u32;
        self.mgr_wage = wage;
        self.mgr_since = self.abs();
        self.world.clubs[club].manager = self.coach.clone();
        self.world.clubs[club].pressure = 0.0;
        let l = self.league();
        self.world.leagues[l].detail = 0;
        for p in self.world.clubs[club].squad.clone() { self.world.players[p].know = 1.0; }
        self.negs.clear();
        self.xi.clear();
        self.formation = 0;
        self.frei.clear();
        self.transfer_budget = self.world.revenue(club) * 0.22;
        self.infra_topf = None;
        self.wi = Default::default();
        self.wirtschaft_an();
        self.book.clear();
        let (fa, me, vo) = (self.fans, self.media, self.mensch.vorstand);
        self.klima(50.0 - fa, 50.0 - me, 50.0 - vo, crate::inhalt::t("ui.game.neuer_verein_alles_beginnt_von"));
        self.start_erwartung();
        self.nebenziele.clear();
        self.nebenziele_setzen();
        self.mensch.vertrag_bis = self.abs() + (crate::regeln::f("laufbahn", "vertrag.laufzeit_start_monate") * 30.4) as u32;
        self.mensch.konflikte.clear();
        self.mensch.versprechen.clear();
        self.mensch.saisonziel_gebucht = false;
        self.mensch.ziel_angenommen = false;
        self.ziel_auffrischen();
        self.academy_intake(30, true);
        self.make_youth_leagues();
        self.auto_lineup();
        let (cn, coach) = (self.club().name.clone(), self.coach.clone());
        self.news(&crate::tf!("ui.game.uebernimmt", coach, cn), crate::inhalt::t("ui.game.der_neue_trainer_soll_frischen"));
        let t = crate::tf!("ui.game.unser_ziel_mindestens_platz", self.club().goal_kind, self.club().goal_pos);
        self.msg(5, "Präsidium", &crate::tf!("station.neu.vorstellung.betreff", cn), &t, Action::None); // roh
        self.wechsel_geschichte(old, war_entlassen);
    }

    // ------------------------------------------------------------ press (medien.toml)
    // ------------------------------------------------------------ season end
    fn season_end(&mut self) {
        self.world.saison_archivieren();
        self.world.vstat_vor = std::mem::take(&mut self.world.vstat);
        // a title or the goal clearly beaten: bigger clubs take a look (Fabian 23.11.)
        {
            let (rang, ziel) = (self.world.rank_of(self.own) as i32, self.club().goal_pos as i32);
            if !self.fired && (rang == 1 || rang + crate::regeln::f("laufbahn", "interesse.lockruf_ziel_plaetze") as i32 <= ziel) { self.lockruf_bis = self.abs() + crate::regeln::f("laufbahn", "interesse.lockruf_tage") as u32; }
        }
        // a season ended with no confidence left costs the job (Fabian 23.11.)
        // (only while the board already stands against the coach: a running ultimatum)
        if !self.fired && self.ultimatum.is_some() && self.mensch.vorstand < crate::regeln::f("vereinswelt", "praesidium.saisonende_entlassung_unter") { self.fire(crate::inhalt::t("ui.ultimatum.saisonende")); }
        // the season's awards before the season's figures start again
        self.ehrungen_saisonende();
        self.nebenziele_abrechnen();
        self.statistik_saisonende();
        self.klauseln_saison();
        let l = self.league();
        let table = self.world.table(l);
        let rank = table.iter().position(|r| r.club == self.own).map(|i| i + 1).unwrap_or(0);
        let goal = self.club().goal_pos;
        let cn = self.club().name.clone();
        let season = self.world.season;
        self.history.push(crate::tf!("ui.game.platz_ziel", date::season_label(season), cn, rank, goal));
        for li in 0..self.world.leagues.len() {
            if li != l && self.world.leagues[li].detail > 1 { continue; }
            if let (Some(r), Some(m)) = (self.world.table(li).first().cloned(), self.world.meister(li)) {
                let (n, ln) = (self.world.clubs[m].name.clone(), self.world.leagues[li].name.clone());
                let t = if m == r.club { crate::tf!("ui.game.punkte_aus_spielen", r.pts, r.p) } else { crate::inhalt::t("ui.game.sieger_der_endrunde_der_besten").into() };
                self.news(&crate::tf!("ui.game.ist_meister_in", n, ln), &t);
            }
        }
        let gap = rank as i32 - goal as i32;
        // Coaching Points: the season goal (beaten clearly: more) and this season's titles (Konzept 5.1)
        {
            let m = |k: &str| crate::regeln::f("skilltree", &format!("manager.{k}")) as u8;
            if gap <= -(m("ziel_uebertroffen_plaetze") as i32) { self.cp_geben(m("ziel_uebertroffen"), crate::inhalt::t("ui.mst.grund.ziel_plus")); } else if gap <= 0 { self.cp_geben(m(crate::id("ziel")), crate::inhalt::t("ui.mst.grund.ziel")); }
            let titel: Vec<String> = self.world.titles.iter().filter(|x| x.club == self.own && x.season == self.world.season).map(|x| x.comp.clone()).collect();
            for t in titel { self.cp_geben(m(crate::id("titel")), &t); }
        }
        self.mgr_rep = self.trainerruf();
        // the board reacts through the climate drivers (saison_ziel_erreicht / _verfehlt, klima.json); no firing here (§12)
        if !self.fired {
            if gap > 0 {
                self.msg(5, "Präsidium", crate::inhalt::t("ui.game.saisonbilanz"), &crate::tf!("ui.game.platz_das_ziel_war_platz", rank, goal), Action::None); // roh
            } else {
                self.msg(5, "Präsidium", crate::inhalt::t("ui.game.saisonbilanz"), &crate::tf!("ui.game.platz_ziel_erreicht_das_praesidium", rank), Action::None); // roh
            }
        }
        for z in self.ehrungen_saison() { self.history.push(format!("{} {z}", date::season_label(self.world.season))); }
        self.leihen_ende();
        self.massstab_saison();
        self.sponsoren_saisonende(gap <= 0);
        self.zusagen_saisonende(gap <= 0);
        let euro = self.world.euro_qualify();
        self.world.season_honours();
        self.tv_zuschlag_saison(rank);
        self.experten_bilanz(rank);
        self.geschichten_saisonende();
        let saldo: f64 = self.book.iter().map(|b| b.1).sum();
        if !self.fired { self.versammlung_einladen(rank, goal, saldo); }
        let stufe_alt = self.world.leagues[self.league()].stufe;
        self.promotion();
        self.koop_saison();
        let stufe_neu = self.world.leagues[self.league()].stufe;
        if stufe_neu != stufe_alt { self.auf_abstieg_geld(stufe_alt, stufe_neu); }
        if stufe_neu < stufe_alt { self.cp_geben(crate::regeln::f("skilltree", "manager.titel") as u8, crate::inhalt::t("ui.mst.grund.aufstieg")); }
        self.geschichten_stufe(stufe_alt, stufe_neu);
        self.vorvertraege_vollziehen();
        let own = self.own as i32;
        for i in 0..self.world.players.len() {
            let pl = &self.world.players[i];
            if pl.team == -2 { continue; }
            let age = pl.age + 1;
            if pl.youth == 0 && self.world.rng.chance(retire_chance(age)) {
                let (n, t, ovr) = (pl.name.clone(), pl.team, pl.ovr);
                if t >= 0 { self.world.clubs[t as usize].squad.retain(|&x| x != i); }
                self.world.players[i].team = -2;
                self.world.players[i].profil.ligen.clear();
                self.xi.retain(|&x| x != i);
                if t == own || ovr >= 78.0 { self.news(&crate::tf!("ui.game.beendet_seine_karriere", n), &crate::tf!("ui.game.mit_jahren_ist_schluss", age)); }
                continue;
            }
            let club = if self.world.players[i].team >= 0 { self.world.clubs[self.world.players[i].team as usize].name.clone() } else { String::new() };
            let p = &mut self.world.players[i];
            if p.apps > 0 && !club.is_empty() { let st = (self.world.season, club, p.apps, p.goals, p.avg_rating()); p.profil.laufbahn.push(st); }
            // dynamic potential (Fabian 25.11., like FC's): a young player's season on the pitch moves his ceiling a
            // little, every club; never below what he is (entwicklung_pesm.toml [potenzial])
            {
                let d = |k: &str| crate::regeln::f("entwicklung_pesm", &format!("potenzial.{k}"));
                if (p.age as f32) <= d("bis_alter") && p.rn as f32 >= d("min_noten") {
                    let schnitt = p.rsum / p.rn as f32;
                    let plus = if schnitt >= d("top_ab") { d(crate::id("top")) } else if schnitt >= d("gut_ab") { d(crate::id("gut")) } else if schnitt <= d("schwach_bis") { d(crate::id("schwach")) } else { 0.0 };
                    if plus != 0.0 { p.pot = (p.pot + plus).clamp(p.ovr, d(crate::id("decke"))); }
                }
            }
            p.age = age;
            p.apps = 0; p.goals = 0; p.assists = 0; p.rsum = 0.0; p.rn = 0; p.yellow = 0; p.susp = 0; p.profil.ligen.clear(); p.profil.vorsaison = std::mem::take(&mut p.profil.saison);
            if p.team >= 0 && p.youth == 0 && p.contract_end < season + 1 {
                let t = p.team as usize;
                if p.team == own || self.world.rng.chance(0.3) {
                    self.world.clubs[t].squad.retain(|&x| x != i);
                    self.world.players[i].team = -1;
                    self.xi.retain(|&x| x != i);
                    if t == self.own { let n = self.world.players[i].name.clone(); self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("tv.drama.vertrag.verloren.betreff", n), crate::inhalt::t("ui.game.sein_vertrag_ist_ausgelaufen"), Action::None); }
                } else {
                    let p = &mut self.world.players[i];
                    p.contract_end = season + contract_years(p.age);
                    self.ki_verlaengerung_klausel(i);
                }
            }
        }
        // U17 → U19 by birth year, the closing round at 19, the AI clubs' graduates (jugend.rs, Akademie-Konzept)
        self.jugend_saisonwechsel();
        self.u23_an_zweite();
        self.akademie_jahrgang();
        let free: Vec<usize> = (0..self.world.players.len()).filter(|&i| self.world.players[i].team == -1).collect();
        let mut fi = 0;
        for c in 0..self.world.clubs.len() {
            if c == self.own { continue; }
            while self.world.clubs[c].squad.len() < 22 && fi < free.len() {
                let p = free[fi];
                fi += 1;
                let lv = self.world.leagues[self.world.clubs[c].league].level;
                let abs = self.abs();
                let pl = &mut self.world.players[p];
                pl.team = c as i32;
                pl.joined = abs;
                pl.contract_end = season + contract_years(pl.age);
                pl.wage = fair_wage(pl, lv);
                self.world.clubs[c].squad.push(p);
            }
            let len = self.world.clubs[c].squad.len();
            if len < 18 { self.regen(c, 22 - len); }
        }
        self.world.season += 1;
        self.world.day = 0;
        self.world.refresh_levels();
        self.world.update_values();
        self.world.set_goals();
        for li in 0..self.world.leagues.len() { self.world.leagues[li].deduct.clear(); self.world.make_fixtures(li); }
        self.world.setup_cups(Some(euro));
        let europa: Vec<String> = self.world.cups.iter().filter(|c| (1..=3).contains(&c.kind) && c.teams.contains(&self.own)).map(|c| c.name.clone()).collect();
        for n in europa { self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), &crate::tf!("ui.game.wir_spielen_in_der", n), crate::inhalt::t("ui.game.die_auslosung_der_ligaphase_steht"), Action::None); }
        self.make_youth_leagues();
        self.book.clear();
        self.warned = false;
        self.results.clear();
        self.mensch.saisonziel_gebucht = false;
        self.ziel_auffrischen();
        self.nebenziele_setzen();
        self.experten_tipp();
        self.transfer_budget = (self.world.revenue(self.own) * 0.22).max(self.club().balance * 0.3).max(self.transferbudget_echt());
        // the infrastructure pot gets its share of the season; what was not spent stays
        self.infra_topf = Some(self.infra() + self.umsatz_prognose() * crate::regeln::f("vereinswelt", "toepfe.infra_anteil") as f64);
        if !self.fired {
            let c = self.club();
            let t = crate::tf!("ui.game.neue_saison_unser_ziel_ist", date::season_label(self.world.season), c.goal_kind, c.goal_pos, money(self.transfer_budget));
            self.msg(5, "Präsidium", crate::inhalt::t("ui.game.die_neue_saison"), &t, Action::None); // roh
        }
        self.urlaub_rueckkehr();
        self.leistungstest();
    }
    fn regen(&mut self, c: usize, n: usize) {
        let lv = self.world.clubs[c].level;
        let names: Vec<String> = self.world.players.iter().step_by(97).take(400).map(|p| p.name.clone()).collect();
        for _ in 0..n {
            let r = &mut self.world.rng;
            let a = names[r.int(0, names.len() as i32 - 1) as usize].split_whitespace().next().unwrap_or(crate::inhalt::t("ui.game.max")).to_string();
            let b = names[r.int(0, names.len() as i32 - 1) as usize].split_whitespace().last().unwrap_or(crate::inhalt::t("ui.game.muster")).to_string();
            let pos = [crate::id("GK"), crate::id("CB"), crate::id("LB"), crate::id("RB"), crate::id("DMF"), crate::id("CMF"), crate::id("AMF"), crate::id("LWF"), crate::id("RWF"), crate::id("CF")][r.int(0, 9) as usize];
            let age = r.int(17, 21) as u8;
            let ovr = (lv - 8.0 + r.gauss(3.0)).clamp(40.0, 80.0);
            let talent = (r.gauss(0.35) + 1.0).clamp(0.15, 2.3);
            let idx = self.world.players.len();
            let season = self.world.season;
            let mut p = self.world.players[0].clone();
            let (attrs, pp) = self.world.template_attrs(pos, ovr);
            p.id = self.world.new_id(); p.attrs = attrs; p.pp = pp; p.db = false; p.skills = 0; p.style = 0; p.name = format!("{a} {b}"); p.pos = pos.into(); p.age = age; p.ovr = ovr; p.pot = (ovr + 8.0 * talent).min(92.0); p.talent = talent;
            p.team = c as i32; p.youth = 0; p.contract_end = season + 3; p.know = 0.1; p.inj = 0; p.susp = 0; p.last.clear(); p.listed = false; p.program = 0;
            p.wage = fair_wage(&p, self.world.leagues[self.world.clubs[c].league].level);
            self.world.players.push(p);
            self.world.clubs[c].squad.push(idx);
        }
    }
    /// Season end: promotion and relegation by the league rules, then second teams out of their first team's league.
    fn promotion(&mut self) {
        let n = self.world.leagues.len();
        let tables: Vec<Vec<Row>> = (0..n).map(|l| self.world.table(l)).collect(); // final tables before anybody moves
        self.aufstieg_abstieg(&tables);
        // a second team may not play in the league of its first team: one tier down, the next club of the table goes up
        let reihen: Vec<Vec<usize>> = tables.iter().map(|t| t.iter().map(|r| r.club).collect()).collect();
        for (z, von, nach, k, geschaetzt) in self.world.zweite_trennen(&reihen) {
            let (zn, en, kn) = (self.world.clubs[z].name.clone(), self.world.clubs[self.world.erste_von(z).unwrap()].name.clone(), self.world.clubs[k].name.clone());
            let t = crate::tf!("ui.game.wird_in_eingeteilt_eine_zweite", zn, self.world.leagues[nach].name, if geschaetzt { crate::inhalt::t("ui.game.staffel_geschaetzt") } else { "" }, en, self.world.leagues[von].name, kn);
            if self.world.erste_von(z) == Some(self.own) || k == self.own { self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), &crate::tf!("ui.game.spielt_eine_stufe_tiefer", zn), &t, Action::None); }
            if k == self.own || z == self.own {
                let (neu, alt) = (self.world.clubs[self.own].league, if k == self.own { nach } else { von });
                self.world.leagues[neu].detail = 0;
                self.world.leagues[alt].detail = if self.world.leagues[alt].key.contains("D1_LEAGUE") { 1 } else { 2 };
            }
            self.news(&crate::tf!("ui.game.spielt_eine_stufe_tiefer", zn), &t);
        }
    }
    /// Promotion and relegation by daten/ligaregeln.toml (ligaregeln.rs saisonwechsel), then money and messages.
    fn aufstieg_abstieg(&mut self, tables: &[Vec<Row>]) {
        for w in self.world.saisonwechsel(tables) {
            let (d, u, upper, lower) = (w.ab, w.auf, w.oben, w.ziel);
            // transfers.py season_start: TV money follows the league class (tv_scale 8), wages cut 0.85 after a big drop
            let (lu, ll) = (self.world.leagues[upper].level, self.world.leagues[lower].level);
            let ratio = ((ll - lu) / 8.0).exp() as f64;
            self.world.clubs[d].tv *= ratio;
            self.world.clubs[u].tv /= ratio;
            if ratio < 0.75 { for p in self.world.clubs[d].squad.clone() { self.world.players[p].wage *= 0.85; } }
            let wie = match w.grund { "Relegation" => crate::inhalt::t("ui.game.ueber_die_relegation"), "Play-off" => crate::inhalt::t("ui.game.ueber_die_play_offs"), "Playout" => crate::inhalt::t("ui.game.nach_dem_playout"), _ => "" }; // roh
            if w.grund == "Relegation" || lower == self.league() || upper == self.league() { // roh
                let (dn, un) = (self.world.clubs[d].name.clone(), self.world.clubs[u].name.clone());
                if w.grund == "Relegation" { self.news(&crate::tf!("ui.game.gewinnt_die_relegation", un), &crate::tf!("ui.game.spielt_naechste_saison_in_muss", un, self.world.leagues[upper].name, dn)); } // roh
            }
            if d == self.own || u == self.own {
                let cn = self.club().name.clone();
                let t = if d == self.own { crate::tf!("ui.game.abstieg_spielt_naechste_saison_in", wie, cn, self.world.leagues[lower].name, if w.geschaetzt { crate::inhalt::t("ui.game.staffel_geschaetzt") } else { "" }) } else { crate::tf!("ui.game.aufstieg_spielt_naechste_saison_in", wie, cn, self.world.leagues[upper].name) };
                self.msg(0, crate::inhalt::t("zentrale.kennzahl.liga"), if d == self.own { crate::inhalt::t("vw.praemie.wb.abstieg") } else { crate::inhalt::t("vw.praemie.wb.aufstieg") }, &t, Action::None);
                self.news(&t, "");
                let (nl, ol) = if d == self.own { (lower, upper) } else { (upper, w.unten) };
                self.world.leagues[nl].detail = 0;
                self.world.leagues[ol].detail = if self.world.leagues[ol].key.contains("D1_LEAGUE") { 1 } else { 2 };
            }
        }
    }

    // ------------------------------------------------------------ FL27 (Modul-Paket FL27-Anbindung)
    /// The career folder: original backup, pesdb copy, backups and match files.
    pub fn karriere_dir(&self) -> std::path::PathBuf { save_dir().join(&self.save_name) }
    /// The career's own CPK (name fixed once, Übergabe Daten §8.1) and quelle.json {spiel, welt, cpk} in the career folder.
    pub fn fl27_cpk(&mut self) -> String {
        if self.fl27_cpk.is_empty() {
            let name = self.club().name.clone();
            self.fl27_cpk = fl27::cpk_name(&name, &fl27::sider().join(crate::id("livecpk")));
            let q = serde_json::json!({"spiel": fl27::spielordner(), "welt": format!("SiderAddons/livecpk/{}", fl27::welt_name(&self.welt())), "cpk": self.fl27_cpk});
            let _ = fl27::atomic_write(&self.karriere_dir().join("quelle.json"), serde_json::to_string_pretty(&q).unwrap().as_bytes());
        }
        self.fl27_cpk.clone()
    }
    /// CPKs of the other PES M careers (their quelle.json); only these may be commented out in sider.ini.
    pub fn fl27_andere(&self) -> Vec<String> {
        std::fs::read_dir(save_dir()).map(|d| d.flatten().filter_map(|e| {
            let q: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(e.path().join("quelle.json")).ok()?).ok()?;
            q["cpk"].as_str().map(|x| x.to_string())
        }).filter(|n| *n != self.fl27_cpk).collect()).unwrap_or_default()
    }
    pub fn welt(&self) -> std::path::PathBuf {
        if self.fl27_welt.is_empty() { fl27::welt_ordner(&fl27::sider()) } else { self.fl27_welt.clone().into() }
    }
    fn match_clubs(&self) -> Option<(usize, usize)> {
        let f = self.today_match().map(|t| self.fixture(t)).or_else(|| self.next_match())?;
        Some((f.h, f.a))
    }
    /// Lineup in the order of the formation places (place 0 = keeper), then the bench; only players of the FL database.
    fn fl27_order(&self, club: usize, known: &std::collections::HashSet<u32>) -> Vec<u32> {
        let (xi, bench) = if club == self.own { (self.current_xi(), self.world.lineup_slots(club, self.eigene_slots(), &self.xi).1) } else { self.world.lineup(club, 0, &[]) };
        xi.iter().map(|x| x.1).chain(bench).filter(|&p| self.world.players[p].available()).map(|p| self.world.players[p].id).filter(|id| known.contains(id)).collect()
    }
    /// Step 5 (Paket 1): abilities, age, new players (own records in Player.bin, also youth and regens), transfers,
    /// shirt numbers and the lineups of both clubs into the career's pesdb copy. The original world is never touched.
    pub fn fl27_vorbereiten(&mut self) -> Result<String, String> {
        let (n, _) = self.fl27_schreiben()?;
        let (changed, new_recs, wechsel) = self.fl27_letzte;
        let _ = n;
        Ok(crate::tf!("ui.game.spieler_aktualisiert_neue_spieler_angele", changed, new_recs, wechsel))
    }
    /// The write itself (match start and live writing, live.rs): returns the number of changed players and the files
    /// that were written (only changed ones). Line-ups of the next match's clubs when one is planned.
    pub fn fl27_schreiben(&mut self) -> Result<(usize, Vec<&'static str>), String> {
        let k = self.karriere_dir();
        let welt = self.welt();
        fl27::karriere_pesdb_anlegen(&k, &welt)?;
        let pesdb = fl27::karriere_pesdb(&k);
        let mut pb = crate::pesdb::PlayerBin::load(&pesdb.join("Player.bin"))?;
        let (mut new_recs, mut changed) = (0, 0);
        let mut live_attr: Vec<(u32, Vec<(usize, u8)>)> = vec![];
        // templates per position for new players: preferably players without a league club (less visible in FL27), up to 200 each
        let free: std::collections::HashSet<u32> = self.world.players.iter().filter(|p| p.team == -1).map(|p| p.id).collect();
        let mut tmpl: std::collections::HashMap<u8, Vec<usize>> = std::collections::HashMap::new();
        for pass in [true, false] {
            for i in 0..pb.n() {
                let v = tmpl.entry(pb.position(i)).or_default();
                if v.len() < 200 && free.contains(&pb.id(i)) == pass && !v.contains(&i) { v.push(i); }
            }
        }
        for p in self.world.players.iter_mut() {
            if p.team == -2 || p.attrs.len() != 25 { continue; }
            let i = match pb.index.get(&p.id) {
                Some(&i) => i,
                None => {
                    let pos = crate::pesdb::pos_index(&p.pos);
                    let pp = if p.pp.len() == 13 { p.pp.clone() } else { let mut v = vec![0; 13]; v[pos as usize] = 2; v };
                    let i = pb.new_player(tmpl.get(&pos).map(|v| v.as_slice()).unwrap_or(&[]), p.id, &p.name, pos, &pp, &p.attrs, p.age, p.style, p.skills);
                    if p.nat > 0 { pb.set_nat(i, p.nat); } // the new player's country (academy: the club's country)
                    new_recs += 1;
                    i
                }
            };
            let before = pb.data[i * crate::pesdb::REC..(i + 1) * crate::pesdb::REC].to_vec();
            let attrs_vor = pb.attrs(i);
            // the form bonus goes into the game for every player (both teams), always from the true strength, never added up
            pb.set_attrs(i, &crate::world::attrs_mit_zuschlag(p));
            pb.set_age(i, p.age);
            // a retrained player plays his new position in the game too (Fabian 25.11.: Xhaka DMF in PES M, CMF in FL27)
            let pos = crate::pesdb::pos_index(&p.pos);
            if pb.position(i) != pos { pb.set_position(i, pos); }
            if p.pp.len() == 13 && pb.positions(i)[..] != p.pp[..] { pb.set_positions(i, &p.pp); }
            // a re-rolled academy player keeps his record: name and country follow
            if p.youth > 0 && pb.name(i) != p.name { pb.set_name(i, &p.name); }
            if p.youth > 0 && p.nat > 0 && pb.nat(i) != p.nat { pb.set_nat(i, p.nat); }
            // the playing style always goes in: it is the player's role chosen in the tactics (Fabian 23.11.)
            if p.ent.an { pb.set_style_skills(i, p.style, p.skills); } else if pb.style(i) != p.style { pb.set_style(i, p.style); }
            if pb.data[i * crate::pesdb::REC..(i + 1) * crate::pesdb::REC] != before[..] { changed += 1; }
            let attrs_nach = pb.attrs(i);
            let diff: Vec<(usize, u8)> = (0..25).filter(|&k| attrs_vor[k] != attrs_nach[k]).map(|k| (k, attrs_nach[k])).collect();
            if !diff.is_empty() && new_recs == 0 { live_attr.push((p.id, diff)); }
            p.db = true;
        }
        let known: std::collections::HashSet<u32> = pb.index.keys().copied().collect();
        let (head, mut a) = fl27::asg_laden(&pesdb)?;
        let a0 = a.clone();
        let spiel = self.match_clubs();
        let raus = self.nicht_gemeldet_heute();
        let n = asg_abgleichen(&self.world, &mut a, &known, &spiel.map(|(h, aw)| vec![h, aw]).unwrap_or_default())?;
        // not registered for today's European match: out of the FL27 squad too (Fabian 25.11.); back at the next write
        let own_id = self.world.clubs[self.own].id;
        for p in raus { let id = self.world.players[p].id; if a.iter().any(|x| x.pid == id && x.team == own_id) { fl27::transfer(&mut a, id, own_id, 0, 0)?; } }
        if let Some((h, aw)) = spiel {
            for c in [h, aw] {
                let mut order = self.fl27_order(c, &known);
                let id = self.world.clubs[c].id;
                if c == self.own {
                    // our formation as in PES M: code and spot of each slot, the eleven in the same order (Fabian 22.11.)
                    let (slots, spots) = (self.eigene_slots(), self.eigene_spots());
                    let mut plaetze: Vec<(u8, u8, u8, u32)> = slots.iter().zip(spots.iter()).enumerate().map(|(i, (s, &(x, y)))| (crate::pesdb::pos_index(s), (3.0 + (y - 0.04).max(0.0) / 0.82 * 40.0).round().clamp(0.0, 47.0) as u8, (x * 104.0).round().clamp(0.0, 104.0) as u8, order.get(i).copied().unwrap_or(0))).collect();
                    // the places in the order of the world's data (game test 22.11.: otherwise RB/CB and RM/CM swap)
                    fl27::formation_ordnen(&mut plaetze);
                    // with and against the ball: the same players on the places of the other formation, in the order the
                    // kick-off places got (Fabian 24.11.)
                    let anders = |k: u8| -> Vec<(u8, u8, u8)> {
                        if k == 1 && self.formation_ball.is_none() && self.frei_ball.len() != 11 || k == 2 && self.formation_gegen.is_none() && self.frei_gegen.len() != 11 { return plaetze.iter().map(|p| (p.0, p.1, p.2)).collect(); }
                        let (sl, sp) = (self.satz_slots(k), self.satz_spots(k));
                        let roh: Vec<(u8, u8, u8, u32)> = sl.iter().zip(sp.iter()).enumerate().map(|(i, (s, &(x, y)))| (crate::pesdb::pos_index(s), (3.0 + (y - 0.04).max(0.0) / 0.82 * 40.0).round().clamp(0.0, 47.0) as u8, (x * 104.0).round().clamp(0.0, 104.0) as u8, order.get(i).copied().unwrap_or(0))).collect();
                        plaetze.iter().map(|b| roh.iter().find(|r| r.3 == b.3).map(|r| (r.0, r.1, r.2)).unwrap_or((b.0, b.1, b.2))).collect()
                    };
                    let (mit, gegen) = (anders(1), anders(2));
                    // our sliders from the PES M tactics: compactness and defensive line (support range stays the world's)
                    let _ = fl27::regler_schreiben(&pesdb, id, Some(self.tactic.kompakt.clamp(1, 10) as u8), Some(crate::taktik9::unterstuetzung(&self.t9)), Some(self.tactic.linie.clamp(1, 10) as u8));
                    let grund: Vec<(u8, u8, u8)> = plaetze.iter().map(|p| (p.0, p.1, p.2)).collect();
                    if fl27::formation_schreiben_saetze(&pesdb, id, [&grund, &mit, &gegen]).is_ok() && order.len() >= 11 {
                        let elf: Vec<u32> = plaetze.iter().map(|p| p.3).collect();
                        order.splice(..11, elf);
                    }
                } else if let Some(mut f) = { self.world.trainer_formationen(&self.welt().join("common/etc/pesdb")); self.world.formation_wahl(c, self.own, self.abs()) } {
                    // the opponent's coach chooses between his favourite and his fallback formation (Fabian 22.11.)
                    let mut p: Vec<(u8, u8, u8, ())> = f.drain(..).map(|x| (x.0, x.1, x.2, ())).collect();
                    fl27::formation_ordnen(&mut p);
                    let plaetze: Vec<(u8, u8, u8)> = p.iter().map(|x| (x.0, x.1, x.2)).collect();
                    if fl27::formation_schreiben(&pesdb, id, &plaetze).is_ok() && order.len() >= 11 {
                        let codes: Vec<u8> = plaetze.iter().map(|x| x.0).collect();
                        let elf: Vec<(u8, u32)> = order[..11].iter().map(|&pid| (self.world.players.iter().find(|p| p.id == pid).map(|p| crate::pesdb::pos_index(&p.pos)).unwrap_or(5), pid)).collect();
                        let neu = fl27::nach_formation(&codes, &elf);
                        order.splice(..11, neu);
                    }
                } else if let Ok(codes) = fl27::formation_codes(&pesdb, id) {
                    // the opponent keeps its formation from the world: its eleven onto the fitting places
                    if codes.len() == 11 && order.len() >= 11 {
                        let elf: Vec<(u8, u32)> = order[..11].iter().map(|&pid| (self.world.players.iter().find(|p| p.id == pid).map(|p| crate::pesdb::pos_index(&p.pos)).unwrap_or(5), pid)).collect();
                        let neu = fl27::nach_formation(&codes, &elf);
                        order.splice(..11, neu);
                    }
                }
                fl27::set_lineup(&mut a, id, &order)?;
                // our set-piece takers as in PES M (Discord 24.11.: they did not reach the game plan)
                if c == self.own { let t = |art: u8| self.schuetze(art).map(|p| self.world.players[p].id); fl27::schuetzen_setzen(&mut a, id, [t(1), t(2), t(0), t(3)]); }
            }
        }
        let bytes = pb.bytes();
        // only changed files are written
        let player_neu = std::fs::read(pesdb.join("Player.bin")).ok().as_deref() != Some(&bytes[..]);
        let asg_neu = a != a0;
        let mut dateien = vec![];
        if player_neu { dateien.push("Player.bin"); }
        if asg_neu { dateien.push("PlayerAssignment.bin"); }
        if !dateien.is_empty() { fl27::pesdb_schreiben(&k, player_neu.then_some(&bytes[..]), head, asg_neu.then_some(&a[..]))?; }
        self.fl27_letzte = (changed, new_recs, n);
        // for the running game: teams whose entries changed with the same players can be patched live, else a restart
        let mut teams: Vec<u32> = a.iter().chain(a0.iter()).map(|x| x.team).collect();
        teams.sort();
        teams.dedup();
        let (mut live_teams, mut neustart) = (vec![], new_recs > 0);
        for t in teams {
            let alt: Vec<&fl27::Asg> = a0.iter().filter(|x| x.team == t).collect();
            let neu: Vec<&fl27::Asg> = a.iter().filter(|x| x.team == t).collect();
            if alt.len() == neu.len() && alt.iter().zip(neu.iter()).all(|(x, y)| x.k == y.k && x.pid == y.pid) { continue; }
            let mut p1: Vec<u32> = alt.iter().map(|x| x.pid).collect();
            let mut p2: Vec<u32> = neu.iter().map(|x| x.pid).collect();
            p1.sort(); p2.sort();
            if p1 == p2 { live_teams.push(t); } else { neustart = true; }
        }
        self.live_aenderung = (live_attr, live_teams, neustart);
        self.gesichter_schreiben(); // faces of new players (gesichter.rs), read by FL27 at its next start
        Ok((changed + new_recs + n, dateien))
    }
    /// Weather and time of day for the presets: midweek games at night, the weather by month. Estimated rule.
    pub fn fl27_wetter(&self, day: u32) -> (&'static str, &'static str) {
        let (_, m, _) = date::ymd(self.world.season, day);
        let r = (day as u64 * 2654435761 + self.world.season as u64 * 97) % 100;
        let w = match m { 12 | 1 | 2 => if r < 15 { crate::id("snowy") } else if r < 50 { crate::id("rainy") } else if r < 80 { crate::id("cloudy") } else { crate::id("sunny") },
            3 | 4 | 10 | 11 => if r < 30 { crate::id("rainy") } else if r < 65 { crate::id("cloudy") } else { crate::id("sunny") },
            _ => if r < 15 { crate::id("rainy") } else if r < 40 { crate::id("cloudy") } else { crate::id("sunny") } };
        (w, if date::weekday(day) == 2 { crate::id("nacht") } else { crate::id("tag") })
    }
    /// Step 7: competition (tournament id), home stadium, weather into S/flw (§4).
    pub fn fl27_vorgaben(&self) -> Result<String, String> {
        let s = fl27::sider();
        let pesdb = fl27::karriere_pesdb(&self.karriere_dir());
        // the competition of the match (league, domestic cup or Europe) gives the FL27 tournament look (Paket 2)
        let tid = self.fl27_turnier(&s, &pesdb)?;
        let (h, _) = self.match_clubs().ok_or(crate::inhalt::t("ui.game.kein_spiel_geplant"))?;
        let mt = std::fs::read_to_string(s.join("content/stadiums/map_teams.txt")).unwrap_or_default();
        let stadion = fl27::heimstadion(mt.trim_start_matches('\u{feff}'), self.world.clubs[h].id);
        let (w, t) = self.fl27_wetter(self.world.day);
        // the match rules (two legs): a single knock-out match goes to extra time and penalties in FL27; first legs,
        // second legs and league matches end after 90 minutes – a tie on aggregate PES M settles after the match
        let regeln = match self.next_termin() {
            Some((Wo::Pokal(ci), i)) => {
                let x = &self.world.cups[ci].fx[i];
                let ko = x.stage >= 20 && x.stage != 33 || self.world.cups[ci].kind == 0;
                let hin = (x.leg == 2).then(|| self.world.cups[ci].fx.iter().find(|y| y.stage == x.stage && y.leg == 1 && y.f.h == x.f.a && y.f.a == x.f.h).map(|y| format!("hinspiel={}:{}\r\n", y.f.hg.max(0), y.f.ag.max(0)))).flatten().unwrap_or_default(); // roh
                let einzel = ko && x.leg == 0;
                Some(format!("verlaengerung={}\r\nelfmeter={}\r\n{hin}", einzel as u8, einzel as u8)) // roh
            }
            _ => Some("verlaengerung=0\r\nelfmeter=0\r\n".to_string()), // roh
        };
        fl27::vorgaben_schreiben_mit(&s, tid, Some(w), Some(t), stadion.as_deref(), regeln)?;
        if self.start_stamina { let _ = self.start_stamina_schreiben(); }
        Ok(crate::tf!("ui.game.wettbewerb_stadion_wetter", tid.map(|t| t.to_string()).unwrap_or(crate::inhalt::t("ui.wort.frei").into()), stadion.unwrap_or(crate::inhalt::t("ui.wort.frei").into()), w, t))
    }
    /// The direct-start line for the next match (fl27::direktmatch_zeile): FL27 team ids, extra time and penalties only
    /// for a single knock-out match (two legs PES M settles itself, as in flw/spielregeln.txt), final or semi-final.
    pub fn direktmatch_text(&self) -> Option<String> {
        let t = self.next_termin()?;
        let f = self.fixture(t);
        let (mut einzel, mut runde) = (false, None);
        if let Wo::Pokal(ci) = t.0 {
            let (c, x) = (&self.world.cups[ci], &self.world.cups[ci].fx[t.1]);
            if c.kind == 6 { return Some(fl27::direktmatch_zeile(self.world.clubs[f.h].id, self.world.clubs[f.a].id, self.spielminuten, false, false, None)); }
            einzel = (x.stage >= 20 && x.stage != 33 || c.kind == 0) && x.leg == 0;
            let (halb, fin) = if c.kind == 4 { (matches!(x.stage, 22 | 32), matches!(x.stage, 23 | 33)) } else { (x.stage == 33, x.stage == 34) };
            runde = if fin { Some(53) } else if halb { Some(52) } else { None };
        }
        Some(fl27::direktmatch_zeile(self.world.clubs[f.h].id, self.world.clubs[f.a].id, self.spielminuten, einzel, einzel, runde))
    }
    /// The FL27 tournament id of the next match, picked from what the content maps know (Fabian 24.11.: entrance and
    /// scoreboard should find a fitting entry): the competition itself, then the same kind (Conference → Europa →
    /// Champions League, a super cup → the country's cup), then the country's league; Competition.bin byte 3 is the country.
    fn fl27_turnier(&self, s: &std::path::Path, pesdb: &std::path::Path) -> Result<Option<u16>, String> {
        let comps = fl27::competitions(&fl27::pesdb_datei(pesdb, "Competition.bin")?);
        let land = fl27::regionen(&fl27::pesdb_datei(pesdb, "Competition.bin")?);
        let ids = fl27::pesdb_datei(pesdb, "CompetitionRegulation.bin").map(|d| fl27::turnier_ids(&d)).unwrap_or_default();
        let liga = self.world.leagues[self.league()].key.clone();
        let r = comps.iter().find(|c| c.1 == liga).and_then(|c| land.get(&c.0)).copied();
        // the country's competitions whose name has `teil` (D1 first), without the super cups unless asked for
        let im_land = |teil: &str| -> Vec<String> {
            let mut v: Vec<String> = comps.iter().filter(|c| r.is_some() && land.get(&c.0) == r.as_ref() && c.1.contains(teil) && (teil.contains(crate::id("SUPER")) || !c.1.contains(crate::id("SUPER")))).map(|c| c.1.clone()).collect();
            v.sort_by_key(|n| !n.contains("_D1_"));
            v
        };
        let uefa = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        let mut keys = vec![];
        match self.next_termin() {
            Some((Wo::Pokal(ci), i)) => {
                let c = &self.world.cups[ci];
                keys.push(c.key.clone());
                if c.key.ends_with("_D1_CUP") { keys.push(c.key.replace("_D1_CUP", "_CUP")); }
                match (c.kind, c.fx[i].stage == 34, c.country.is_empty()) {
                    (4 | 5, ..) => { keys.push(liga.clone()); keys.extend(im_land(crate::id("LEAGUE"))); }
                    (_, true, true) => keys.extend(uefa(&["UEFA_SUPER_CUP", "UEFA_CHAMPIONS_LEAGUE"])), // roh
                    (1, ..) => keys.extend(uefa(&["UEFA_CHAMPIONS_LEAGUE"])), // roh
                    (2, ..) => keys.extend(uefa(&["UEFA_EUROPE_LEAGUE", "UEFA_CHAMPIONS_LEAGUE"])), // roh
                    (3, ..) => keys.extend(uefa(&["FL_UECL", "UEFA_EUROPE_LEAGUE", "UEFA_CHAMPIONS_LEAGUE"])), // roh
                    (_, true, false) => { keys.extend(im_land("SUPER_CUP")); keys.extend(im_land("_CUP")); keys.extend(im_land(crate::id("LEAGUE"))); }
                    _ => { keys.extend(im_land("_CUP")); keys.extend(im_land(crate::id("LEAGUE"))); }
                }
            }
            _ => { keys.push(liga); keys.extend(im_land(crate::id("LEAGUE"))); }
        }
        let mut kand: Vec<u16> = keys.iter().filter(|k| !k.is_empty()).filter_map(|k| comps.iter().find(|c| &c.1 == k).and_then(|c| ids.get(&c.0)).copied()).collect();
        let mut gesehen = std::collections::HashSet::new();
        kand.retain(|t| gesehen.insert(*t));
        // the maps: scoreboard entries of this world (another world's folder there uses the same ids for other leagues) and
        // the walk-out branches of Entrance.lua
        let welt = fl27::welt_name(&self.welt());
        let fremd: Vec<String> = std::fs::read_dir(s.join(crate::id("livecpk"))).map(|d| d.flatten().map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| *n != welt && !n.starts_with(crate::id("PESM")) && s.join(crate::id("livecpk")).join(n).join("common/etc/pesdb").is_dir()).collect()).unwrap_or_default();
        let karten = [fl27::karten_tids(&std::fs::read_to_string(s.join("content/scoreboards/map_competitions.txt")).unwrap_or_default(), &fremd),
            fl27::entrance_tids(&String::from_utf8_lossy(&std::fs::read(s.join("modules/Entrance.lua")).unwrap_or_default()))];
        Ok(fl27::turnier_waehlen(&kand, &karten))
    }
    /// start_stamina.txt for BlankState222's module (its documented format): `teams = <home id>, <away id>`, then one
    /// line `<player id> = <1-100>` per player of both squads with PES M's fitness (100 is left out).
    pub fn start_stamina_schreiben(&self) -> Result<(), String> {
        let t = self.start_stamina_text()?;
        fl27::atomic_write(&fl27::sider().join("modules").join("start_stamina.txt"), t.as_bytes()) // roh
    }
    pub fn start_stamina_text(&self) -> Result<String, String> {
        let (h, a) = self.match_clubs().ok_or(crate::inhalt::t("ui.game.kein_spiel_geplant"))?;
        let mut t = format!("teams = {}, {}\r\n", self.world.clubs[h].id, self.world.clubs[a].id);
        for c in [h, a] {
            for &p in &self.world.clubs[c].squad {
                let pl = &self.world.players[p];
                let f = pl.fit.round().clamp(1.0, 100.0) as u32;
                if f < 100 { t += &format!("{} = {f}\r\n", pl.id); }
            }
        }
        Ok(t)
    }
    /// After a match (FL27 ended, closed by itself or crashed before): the presets PES M wrote are removed, so later
    /// matches in FL27 choose look, stadium and weather by themselves again. On request the career database goes off too.
    pub fn vorgaben_aufraeumen(&mut self) -> Option<String> {
        if self.vorgaben_gesetzt.is_empty() { return None; }
        let s = fl27::sider();
        let mut text = match fl27::vorgaben_zuruecksetzen(&s, &self.vorgaben_gesetzt) {
            Ok(g) if g.is_empty() => crate::inhalt::t("ui.fl27.vorgaben_zurueck").to_string(),
            Ok(g) => crate::tf!("ui.fl27.vorgaben_geaendert", g.join(", ")),
            Err(e) => return Some(crate::tf!("ui.fl27.vorgaben_fehler", e)),
        };
        self.vorgaben_gesetzt.clear();
        if self.db_nach_spiel_aus {
            let (k, cpk) = (self.karriere_dir(), self.fl27_cpk());
            text = match fl27::sider_ini_anwenden(&k, &s, |t| Ok(fl27::sider_ini_abschalten(t, &cpk))) { Ok(_) => crate::tf!("ui.fl27.und_db_aus", text), Err(e) => crate::tf!("ui.fl27.db_aus_fehler", text, e) };
        }
        let _ = self.save();
        Some(text)
    }
    /// What the person does in FL27 (§5.10).
    pub fn fl27_ansage(&self) -> Vec<String> {
        let Some((h, a)) = self.match_clubs() else { return vec![] };
        let s = fl27::sider();
        let mt = std::fs::read_to_string(s.join("content/stadiums/map_teams.txt")).unwrap_or_default();
        let st = fl27::heimstadion(mt.trim_start_matches('\u{feff}'), self.world.clubs[h].id).unwrap_or_else(|| crate::inhalt::t("ui.game.freie_wahl").into());
        let mut v = if self.fl27_offen { vec![crate::inhalt::t("ui.live.livepatch_bereit").to_string()] } else { vec![] };
        v.extend([
            crate::tf!("flw.start.anleitung.controller", if h == self.own { crate::inhalt::t("zentrale.k.heim") } else { crate::inhalt::t("zentrale.k.auswaerts") }),
            crate::tf!("flw.start.anleitung.teams", self.world.clubs[h].name, self.world.clubs[a].name),
            crate::tf!("flw.start.anleitung.stadion", st),
            crate::inhalt::t("flw.start.anleitung.anstoss").into(),
        ]);
        // the tactics note (Konzept Taktik 5.1): what to set in FL27's own tactics menu, before the kick-off line
        let anstoss = v.pop();
        v.push(crate::inhalt::t("ui.tz.titel").to_string());
        v.extend(self.taktikzettel().into_iter().map(|z| format!("  {z}")));
        v.extend(anstoss);
        v
    }
    /// team_fn for the reader: majority of the side's player ids in the career's PlayerAssignment.bin (§1.4.7).
    pub fn fl27_team_fn(&self) -> Box<dyn Fn(&[u32]) -> u32 + Send> {
        let a = fl27::asg_laden(&fl27::karriere_pesdb(&self.karriere_dir())).map(|x| x.1).unwrap_or_default();
        let clubs: std::collections::HashSet<u32> = self.world.clubs.iter().map(|c| c.id).collect();
        Box::new(move |pids| fl27::team_majority(&a, pids, &clubs))
    }
    /// Takes over a match the reader saw: score, events, minutes, statistics, own Sofascore-style grades. Never twice.
    /// A knock-out match from FL27 that ended level: the person is asked who won after extra time or on penalties.
    pub fn fl27_ko_offen(&self, d: &fl27::SpielDoc) -> bool {
        matches!(self.today_match(), Some((Wo::Pokal(ci), i)) if self.world.ko_offen(ci, i, d.hg as u8, d.ag as u8))
    }
    /// `vorgabe`: the person's answer for a level knock-out match (home team won, on penalties); None = PES M decides.
    pub fn apply_fl27(&mut self, d: &fl27::SpielDoc, vorgabe: Option<(bool, bool)>) -> Result<String, String> {
        if self.fl27_spiele.contains(&d.spiel_id) { return Err(crate::inhalt::t("ui.game.dieses_spiel_wurde_schon_uebernommen").into()); }
        let f = self.fixture(self.today_match().ok_or(crate::inhalt::t("ui.game.heute_ist_kein_eigenes_spiel"))?);
        let (hid, aid) = (self.world.clubs[f.h].id, self.world.clubs[f.a].id);
        if (d.home, d.away) != (hid, aid) {
            return Err(crate::tf!("ui.game.die_erkannte_partie_teams_ist", d.home, d.away));
        }
        let idx: std::collections::HashMap<u32, usize> = [f.h, f.a].iter().flat_map(|&c| self.world.clubs[c].squad.iter().map(|&p| (self.world.players[p].id, p))).collect();
        let mut res = MatchResult { h: f.h, a: f.a, hg: d.hg as u8, ag: d.ag as u8, ..Default::default() };
        let sub_min: std::collections::HashMap<u32, u8> = d.ev.iter().filter(|e| matches!(e.art, fl27::EvArt::Wechsel { .. })).map(|e| (e.pid, e.min)).collect();
        let off_min: std::collections::HashMap<u32, u8> = d.ev.iter().filter_map(|e| if let fl27::EvArt::Wechsel { raus: Some(r) } = e.art { Some((r, e.min)) } else { None }).collect();
        for e in &d.ev {
            let Some(&p) = idx.get(&e.pid) else { continue };
            let ev = match &e.art {
                fl27::EvArt::Tor { vorlage, eigentor } => MatchEvent { min: e.min, side: e.seite, kind: if *eigentor { 1 } else { 0 }, pid: p, pid2: vorlage.and_then(|v| idx.get(&v).copied()) },
                fl27::EvArt::Karte { rot } => MatchEvent { min: e.min, side: e.seite, kind: if *rot { 3 } else { 2 }, pid: p, pid2: None },
                fl27::EvArt::Wechsel { raus } => match raus.and_then(|r| idx.get(&r).copied()) { Some(off) => MatchEvent { min: e.min, side: e.seite, kind: 5, pid: off, pid2: Some(p) }, None => continue },
            };
            res.events.push(ev);
        }
        res.events.sort_by_key(|e| e.min);
        for (side, v) in d.st.s.iter().enumerate() {
            let (gf, ga) = if side == 0 { (d.hg, d.ag) } else { (d.ag, d.hg) };
            for x in v.iter().filter(|x| x.im_spiel()) {
                let Some(&p) = idx.get(&x.pid) else { continue };
                let mins = if x.platz < fl27::STARTELF { off_min.get(&x.pid).copied().unwrap_or(90) } else { 90u8.saturating_sub(*sub_min.get(&x.pid).unwrap_or(&60)) };
                res.minutes.push((p, mins));
                if x.platz < fl27::STARTELF {
                    let pos = self.world.players[p].pos.clone();
                    if side == 0 { res.xi_h.push((pos, p)); } else { res.xi_a.push((pos, p)); }
                }
                res.ratings.push((p, fl27_note(x, gf, ga, &self.world.players[p].pos)));
            }
        }
        // measured statistics (konzepte/spielerstatistiken.md Teil 4): the confirmed kinds of the memory block, xG per shot
        // estimated without looking at the outcome (on target ×1,25, wide ×0,85, penalty 0,76), then scaled to the match xG
        for (side, v) in d.st.s.iter().enumerate() {
            let gegner_tore = if side == 0 { d.ag } else { d.hg } as u32;
            let tw = v.iter().filter(|x| x.im_spiel() && idx.get(&x.pid).map_or(false, |&p| self.world.players[p].pos == "GK")).max_by_key(|x| x.sum(fl27::BALLTOUCH)).map(|x| x.pid);
            let mut seite: Vec<(usize, crate::statistik::Stat, f32)> = vec![];
            for x in v.iter().filter(|x| x.im_spiel()) {
                let Some(&p) = idx.get(&x.pid) else { continue };
                use crate::statistik as st;
                let mut s = vec![0u32; st::N];
                s[st::MIN] = res.minutes.iter().find(|m| m.0 == p).map_or(0, |m| m.1 as u32);
                s[st::SPIELE] = 1;
                s[st::TORE] = x.sum(fl27::GOAL); s[st::VORLAGEN] = x.sum(fl27::ASSIST);
                s[st::SCHUESSE] = x.sum(fl27::SHOOT); s[st::AUFS_TOR] = x.sum(fl27::SHOOT_INFRAME);
                s[st::PAESSE] = x.sum(fl27::PASS); s[st::PAESSE_OK] = x.sum(fl27::PASS_OK); s[st::FLANKEN] = x.sum(fl27::CENTERING);
                s[st::TACKL] = x.sum(fl27::TACKLE_OK); s[st::TACKL_OK] = x.sum(fl27::TACKLE_OK); s[st::ABGEF] = x.sum(fl27::CUTPASS);
                s[st::FOULS] = x.sum(fl27::FOUL).saturating_sub(x.sum(fl27::OFFSIDE)); s[st::GELB] = x.sum(fl27::CARD_Y); s[st::ROT] = x.sum(fl27::CARD_R);
                s[st::PARADEN] = x.sum(fl27::GKSAVE); s[st::EIGENTORE] = x.sum(fl27::OWNGOAL);
                if tw == Some(x.pid) { s[st::GEGENTORE] = gegner_tore; }
                let elf = x.sum(fl27::SHOOT_PK) as f32;
                let (auf, alle) = (s[st::AUFS_TOR] as f32 - elf, s[st::SCHUESSE] as f32 - elf);
                let xg = 0.10 * (1.25 * auf.max(0.0) + 0.85 * (alle - auf).max(0.0)) + 0.76 * elf;
                seite.push((p, s, xg));
            }
            let summe: f32 = seite.iter().map(|x| x.2).sum();
            let faktor = if summe > 0.0 { d.xg[side] as f32 / summe } else { 1.0 };
            for (p, mut s, xg) in seite { s[crate::statistik::XG] = (xg * faktor * 1000.0) as u32; res.stat.push((p, s)); }
        }
        let sh = d.team(fl27::SHOOT);
        let ot = d.team(fl27::SHOOT_INFRAME);
        res.shots = (sh[0].min(255) as u8, sh[1].min(255) as u8);
        res.on_target = (ot[0].min(255) as u8, ot[1].min(255) as u8);
        res.possession = 50; // not in the memory block
        res.reason = crate::tf!("ui.game.selbst_gespielt_in_fl27_schuesse", sh[0], sh[1], format!("{:.2}", d.xg[0]), format!("{:.2}", d.xg[1]), d.team(fl27::PASS)[0], d.team(fl27::PASS)[1]);
        let dir = self.karriere_dir().join(crate::id("spiele"));
        let _ = fl27::atomic_write(&dir.join(format!("{}.json", d.spiel_id)), serde_json::to_string_pretty(&d.json()).unwrap().as_bytes());
        self.fl27_spiele.push(d.spiel_id.clone());
        self.world.ko_vorgabe = vorgabe;
        self.apply_own_match(res);
        self.world.ko_vorgabe = None;
        Ok(crate::tf!("ui.game.ergebnis_aus_fl27_uebernommen", d.hg, d.ag))
    }

    /// A new knock-out round of a cup we are still in: shown once as a draw ceremony (Fabian 23.11.).
    pub fn auslosungen_pruefen(&mut self) {
        let season = self.world.season;
        for ci in 0..self.world.cups.len() {
            let cup = &self.world.cups[ci];
            let st = cup.stage;
            if st < 20 || cup.winner.is_some() || !cup.fx.iter().any(|f| f.stage == st && (f.f.h == self.own || f.f.a == self.own)) { continue; }
            let key = (cup.name.clone(), season, st);
            if self.auslosung_gesehen.contains(&key) { continue; }
            self.auslosung_gesehen.push(key);
            self.auslosungen.push((ci, st));
        }
    }
    /// Is today an autosave day for the chosen rhythm?
    pub fn autosave_heute(&self) -> bool {
        match self.autosave { 1 => true, 2 => crate::date::ymd(self.world.season, self.world.day).2 == 1, _ => crate::date::weekday(self.world.day) == 0 }
    }
    /// Renames the save (Einstellungen › Spielstand): the save file, its card and the career folder move along.
    pub fn umbenennen(&mut self, neu: &str) -> Result<(), String> {
        let neu: String = neu.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').take(40).collect();
        if neu.is_empty() || neu == self.save_name { return Err(crate::inhalt::t("ui.spielstand.name_ungueltig").into()); }
        let d = save_dir();
        if d.join(format!("{neu}.json")).exists() || d.join(&neu).exists() { return Err(crate::inhalt::t("ui.spielstand.name_vergeben").into()); }
        self.save()?;
        let alt = self.save_name.clone();
        if d.join(&alt).is_dir() { std::fs::rename(d.join(&alt), d.join(&neu)).map_err(|e| e.to_string())?; }
        let _ = std::fs::remove_file(d.join(format!("{alt}.info")));
        let _ = std::fs::remove_file(d.join(format!("{alt}.json")));
        self.save_name = neu;
        crate::fl27::log(&format!("Spielstand umbenannt: {alt} -> {}", self.save_name));
        self.save()
    }
    /// A second copy of the save under a free name; the career folder (original backup, database) is copied in the
    /// background. The copy gets its own CPK on its first FL27 start, so the two careers never share one.
    pub fn kopie(&self) -> Result<(String, std::sync::mpsc::Receiver<Result<usize, String>>), String> {
        self.save()?;
        let d = save_dir();
        let name = (2..100).map(|n| format!("{}_{n}", self.save_name)).find(|n| !d.join(format!("{n}.json")).exists() && !d.join(n).exists()).ok_or("kein freier Name")?; // roh
        let mut k = self.clone();
        k.undo = None;
        k.save_name = name.clone();
        k.fl27_cpk.clear();
        k.save()?;
        let (von, nach) = (d.join(&self.save_name), d.join(&name));
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || { let _ = tx.send(if von.is_dir() { crate::fl27::ordner_kopieren(&von, &nach) } else { Ok(0) }); });
        Ok((name, rx))
    }
    pub fn save_path(&self) -> std::path::PathBuf { save_dir().join(format!("{}.json", self.save_name)) }
    pub fn save(&self) -> Result<(), String> {
        let dir = save_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let tmp = dir.join(format!("{}.tmp", self.save_name));
        std::fs::write(&tmp, serde_json::to_vec(self).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, self.save_path()).map_err(|e| e.to_string())?;
        // a small card for the list of careers: club, date, coach (the save itself is too big to read for a list)
        let info = serde_json::json!({"verein": self.club().name, "datum": crate::date::long(self.world.season, self.world.day), "trainer": self.coach});
        let _ = std::fs::write(dir.join(format!("{}.info", self.save_name)), info.to_string());
        Ok(())
    }
    /// Weekly save with a backup: the save before is kept as the newest of `sicherungen` copies in karrieren/sicherungen/.
    pub fn save_mit_sicherung(&self) -> Result<(), String> {
        let n = crate::regeln::f("spielgefuehl", "karrieren.sicherungen") as usize;
        let ordner = save_dir().join(crate::id("sicherungen"));
        if self.save_path().exists() && n > 0 {
            std::fs::create_dir_all(&ordner).map_err(|e| e.to_string())?;
            for i in (1..n).rev() { let _ = std::fs::rename(ordner.join(format!("{}-{i}.json", self.save_name)), ordner.join(format!("{}-{}.json", self.save_name, i + 1))); }
            std::fs::copy(self.save_path(), ordner.join(format!("{}-1.json", self.save_name))).map_err(|e| e.to_string())?;
        }
        self.save()
    }
}

/// Tests that point HOME at a temporary folder run one after the other.
#[cfg(test)]
pub static HOME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn save_dir() -> std::path::PathBuf {
    // the start test sets this to a throw-away folder (main.rs), so it never writes into the person's careers
    if let Ok(d) = std::env::var("PESM_KARRIEREN") { return std::path::PathBuf::from(d); }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".local/share/pesm-rust/karrieren")
}
pub fn list_saves() -> Vec<std::path::PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(save_dir()).map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "json").unwrap_or(false)).collect()).unwrap_or_default();
    v.sort_by_key(|p| std::cmp::Reverse(std::fs::metadata(p).and_then(|m| m.modified()).ok()));
    v
}
/// The list card of a save (club, date, coach) or None for saves from before the card.
pub fn save_info(p: &std::path::Path) -> Option<serde_json::Value> { std::fs::read_to_string(p.with_extension("info")).ok().and_then(|t| serde_json::from_str(&t).ok()) }
/// The backups of a save, newest first.
pub fn sicherungen(p: &std::path::Path) -> Vec<std::path::PathBuf> {
    let name = p.file_stem().map(|x| x.to_string_lossy().to_string()).unwrap_or_default();
    let ordner = save_dir().join(crate::id("sicherungen"));
    (1..=9).map(|i| ordner.join(format!("{name}-{i}.json"))).filter(|x| x.exists()).collect()
}
/// Deleting a career moves it into karrieren/papierkorb/ (save, card and career folder), so a slip can be undone by hand.
pub fn karriere_wegwerfen(p: &std::path::Path) -> Result<(), String> {
    let dir = save_dir();
    let korb = dir.join(crate::id("papierkorb"));
    std::fs::create_dir_all(&korb).map_err(|e| e.to_string())?;
    let name = p.file_stem().map(|x| x.to_string_lossy().to_string()).ok_or_else(|| crate::inhalt::t("ui.karrieren.unbekannt").to_string())?;
    let stempel = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let ziel = korb.join(format!("{name}-{stempel}"));
    std::fs::create_dir_all(&ziel).map_err(|e| e.to_string())?;
    std::fs::rename(p, ziel.join(format!("{name}.json"))).map_err(|e| e.to_string())?;
    let _ = std::fs::rename(p.with_extension("info"), ziel.join(format!("{name}.info")));
    let ordner = dir.join(&name);
    if ordner.is_dir() { let _ = std::fs::rename(&ordner, ziel.join(&name)); }
    Ok(())
}
fn kriterien_start() -> u8 { 1 }
pub fn load(path: &std::path::Path) -> Result<Career, String> {
    let b = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut c: Career = serde_json::from_slice(&b).map_err(|e| e.to_string())?;
    // saves from before the league pyramid of the pesdb (tier, cup, league above, places up and down): read it once
    if !c.fl27_welt.is_empty() && c.world.leagues.iter().any(|l| l.stufe == 0) {
        if let Ok(d) = crate::pesdb::read_world(&c.welt().join("common/etc/pesdb")) {
            for l in c.world.leagues.iter_mut() {
                let Some(m) = d.leagues.iter().find(|m| m.key == l.key) else { continue };
                if l.land.is_empty() { l.land = m.land.clone(); }
                (l.stufe, l.ueber, l.auf, l.ab, l.pokal, l.land_nr) = (m.stufe, m.ueber.clone(), m.auf, m.ab, m.pokal.clone(), m.land_nr);
            }
        }
    }
    c.world.ueber_ergaenzen(); // saves from before the country field
    if !c.world.farm_an { c.world.farm_einrichten(); } // saves from before second teams and farm teams
    if !c.world.trainer_an { let own = c.own; c.world.trainer_ausstatten(Some(own)); } // saves from before the coach bank
    // saves from before the nationality: read it once from the world's Player.bin (only reading)
    if !c.fl27_welt.is_empty() && c.world.players.iter().all(|p| p.nat == 0) {
        if let Ok(pb) = crate::pesdb::PlayerBin::load(&c.welt().join("common/etc/pesdb/Player.bin")) {
            for p in c.world.players.iter_mut() { p.nat = pb.index.get(&p.id).map(|&i| pb.nat(i)).unwrap_or(0); }
        }
    }
    c.world.laender_benennen(); // saves from before the country names ("Land der …")
    crate::world::WERT_AERA.store(if c.marktwert_aera == 0 { 100 } else { c.marktwert_aera }, std::sync::atomic::Ordering::Relaxed);
    // saves from before the real cup names (Fabian 23.11.): "<Land>-Pokal" becomes "DFB-Pokal", "FA Cup" …
    for cup in c.world.cups.iter_mut().filter(|x| x.kind == 0) { if cup.name.ends_with("-Pokal") || cup.name.ends_with(" Cup") && cup.name.starts_with(&cup.country) { cup.name = crate::cups::pokalname(&cup.key, &cup.country); } } // roh
    c.world.ligen_nachtragen(); // saves from before per-league figures
    // saves from before the youth spread (Fabian 24.11.): nearly equal academy ratings drawn once more, the save kept
    if c.jugend_neu_wuerfeln() { let _ = std::fs::copy(path, path.with_extension(crate::id("vor-jugend.json"))); }
    c.ligaregeln_laden();
    c.ki_klauseln_saeen();
    c.echte_namen_einsetzen(); // saves from before the real club staff (24.11.)
    c.praesidium_an(); // saves from before the board (made at a season start otherwise)
    crate::statistik::nachtragen(&mut c.world); // imported saves: detailed numbers of the old matches from the totals
    Ok(c)
}

/// Own grade from the FL27 statistics (Sofascore style): the match.py base (6.3, ±0.25 for the result, +0.9 per goal,
/// −1.2 own goal, −1.4 red, +0.5 clean sheet at the back) plus estimated statistic parts: assist +0.5, on target +0.1
/// (max 0.3), pass accuracy over 10 passes, won tackles +0.15 and intercepted passes +0.1 (max 0.45 / 0.3), saves +0.25 (max 1.0), fouls −0.1.
pub fn fl27_note(x: &fl27::Spieler, gf: u32, ga: u32, pos: &str) -> f32 {
    let mut b = 6.3 + if gf > ga { 0.25 } else if gf < ga { -0.25 } else { 0.0 };
    b += 0.9 * x.sum(fl27::GOAL) as f32 + 0.5 * x.sum(fl27::ASSIST) as f32 - 1.2 * x.sum(fl27::OWNGOAL) as f32 - 1.4 * x.sum(fl27::CARD_R) as f32;
    if matches!(pos, "GK" | "CB" | "LB" | "RB") && ga == 0 { b += 0.5; }
    b += (0.1 * x.sum(fl27::SHOOT_INFRAME) as f32).min(0.3);
    let (p, ok) = (x.sum(fl27::PASS) as f32, x.sum(fl27::PASS_OK) as f32);
    if p >= 10.0 { b += (ok / p - 0.75) * 1.0; }
    b += (0.15 * x.sum(fl27::TACKLE_OK) as f32).min(0.45) + (0.1 * x.sum(fl27::CUTPASS) as f32).min(0.3) + (0.25 * x.sum(fl27::GKSAVE) as f32).min(1.0);
    b -= 0.1 * x.sum(fl27::FOUL) as f32;
    (b.clamp(3.0, 10.0) * 10.0).round() / 10.0
}

/// The real steps of "Match starten" (§5) on a career.
pub struct Fl27Start<'a> { pub c: &'a mut Career, pub assets: std::path::PathBuf, pub log: Vec<String> }
impl fl27::Ablauf for Fl27Start<'_> {
    fn erlaubt(&self) -> bool { self.c.fl27_erlaubt }
    fn bestaetigt(&self) -> bool { self.c.fl27_bestaetigt }
    fn speichern(&mut self) -> Result<(), String> { self.c.save().map(|_| self.log.push("Karriere gesichert".into())) }
    fn spiel_laeuft(&self) -> bool { fl27::fl27_pid().is_some() }
    fn sicherung(&mut self) -> Result<(), String> {
        fl27::original_sichern(&self.c.karriere_dir(), &fl27::sider(), &self.c.welt())?;
        self.log.push("Original-Welt gesichert und geprüft".into());
        Ok(())
    }
    fn vorbereiten(&mut self) -> Result<(), String> {
        // written live: only what is still open (usually nothing), the data are current already
        if self.c.db_live { self.c.db_unruhe = None; self.c.db_schreiben()?; self.log.push("Datenbank ist aktuell (live mitgeschrieben)".into()); return Ok(()); }
        let t = self.c.fl27_vorbereiten()?; self.log.push(t); Ok(())
    }
    fn laden(&mut self) -> Result<(), String> {
        let cpk = self.c.fl27_cpk();
        if self.c.db_live && fl27::sider().join("livecpk").join(&cpk).join("common/etc/pesdb/PlayerAssignment.bin").exists() {
            let neu = fl27::karriere_abgleichen(&self.c.karriere_dir(), &fl27::sider(), &cpk)?;
            if !neu.is_empty() { self.log.push(format!("Abgeglichen: {}", neu.join(", "))); } // roh
            return Ok(());
        }
        fl27::karriere_laden(&self.c.karriere_dir(), &fl27::sider(), &self.c.welt(), &cpk)?;
        self.log.push("Karriere-Datenbank in den Spielordner geladen".into());
        Ok(())
    }
    fn konfiguration(&mut self) -> Result<(), String> {
        let s = fl27::sider();
        if !fl27::direktstart_einrichten(&s, &self.assets)? { self.log.push("Direktstart-Vorlagen fehlen, Schritt übersprungen".into()); }
        // stage 2: straight into the match (side selection for the controllers, then the match day screen)
        if fl27::direktmatch_einrichten(&s, &self.assets)? {
            if fl27::sider_ini_anwenden(&self.c.karriere_dir(), &s, |t| Ok(crate::livepatch::ini_mit_lua(t, "pesm_direktmatch.lua")))? { self.log.push("sider.ini: PESM-Direktstart eingetragen".into()); } // roh
        }
        // LivePatch module: later database changes reach the running game with F7 (no restart)
        if crate::livepatch::modul_einrichten(&s, &self.assets)? {
            if fl27::sider_ini_anwenden(&self.c.karriere_dir(), &s, |t| Ok(crate::livepatch::ini_mit_modul(t)))? { self.log.push("sider.ini: PESM-LivePatch eingetragen".into()); }
        }
        let k = self.c.karriere_dir();
        let (cpk, mut andere) = (self.c.fl27_cpk(), self.c.fl27_andere());
        let wn = fl27::welt_name(&self.c.welt());
        // other worlds with a database of their own (e.g. _FL26Megapack) would put their database over the career's:
        // their lines are switched off too (Fabian 22.11.: Chelsea career on MyWorld, Megapack active above it)
        if let Ok(d) = std::fs::read_dir(s.join(crate::id("livecpk"))) {
            for e in d.flatten() {
                let n = e.file_name().to_string_lossy().to_string();
                if n != cpk && n != wn && n != fl27::DIRECT_ROOT && e.path().join("common/etc/pesdb").is_dir() && !andere.contains(&n) { andere.push(n); }
            }
        }
        if fl27::sider_ini_anwenden(&k, &s, |t| fl27::sider_ini_einrichten(t, &cpk, &wn, &andere, true))? { self.log.push(format!("sider.ini: {cpk} eingerichtet (Sicherung daneben)")); }
        self.log.extend(fl27::module_pruefen(&s, &self.assets, &k, true));
        Ok(())
    }
    fn vorgaben(&mut self) -> Result<(), String> {
        let t = self.c.fl27_vorgaben()?;
        self.c.vorgaben_gesetzt = fl27::vorgaben_merken(&fl27::sider());
        self.log.push(t);
        Ok(())
    }
    fn start(&mut self) -> Result<(), String> {
        self.c.fl27_offen = false;
        // the game starts straight into this match (the module uses the line once, then the main menu comes back)
        if fl27::sider().join("modules/pesm_direktmatch.lua").is_file() {
            match self.c.direktmatch_text().filter(|_| self.c.direktstart) {
                Some(z) => { fl27::direktmatch_schreiben(&fl27::sider(), &z)?; self.log.push(format!("Direktstart: {z}")); } // roh
                // switched off: no line, so the module leaves the game on its main menu
                None => fl27::direktmatch_schreiben(&fl27::sider(), "# aus: Hauptmenue")?, // roh
            }
        }
        let t = fl27::spiel_starten()?;
        crate::livepatch::neustart_loeschen(&self.live_pesdb()); // the game loads the files as they are
        self.log.push(t.into());
        Ok(())
    }
    fn live_bereit(&mut self) -> Result<(), String> {
        let ini = std::fs::read_to_string(fl27::sider().join("sider.ini")).unwrap_or_default();
        if !crate::livepatch::ini_passt(&ini, &self.c.fl27_cpk()) { return Err(crate::inhalt::t("ui.live.neustart_karriere").into()); }
        if crate::livepatch::neustart_noetig(&self.live_pesdb()) { return Err(crate::inhalt::t("ui.live.neustart_noetig").into()); }
        Ok(())
    }
    fn live_patch(&mut self) -> Result<(), String> {
        let (h, a) = self.c.match_clubs().ok_or(crate::inhalt::t("ui.game.kein_spiel_geplant"))?;
        let db = self.live_pesdb();
        for c in [h, a] {
            if !crate::livepatch::add_team(&db, self.c.world.clubs[c].id)? { self.log.push(format!("LivePatch: {} nicht patchbar", self.c.world.clubs[c].name)); } // roh
        }
        if crate::livepatch::neustart_noetig(&db) { return Err(crate::inhalt::t("ui.live.neustart_noetig").into()); }
        self.c.fl27_offen = true;
        self.log.push(crate::inhalt::t("ui.live.livepatch_bereit").into());
        Ok(())
    }
}
impl Fl27Start<'_> {
    /// The career's database in the game folder (what the running game loaded and the patches are made from).
    fn live_pesdb(&mut self) -> std::path::PathBuf { fl27::sider().join(crate::id("livecpk")).join(self.c.fl27_cpk()).join("common/etc/pesdb") }
}
/// Where the FL27 templates (module copies, Direktstart files) are.
pub fn fl27_assets() -> std::path::PathBuf {
    let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default();
    [std::path::PathBuf::from("assets/fl27"), exe.join("../../assets/fl27"), exe.join("assets/fl27")].into_iter().find(|p| p.exists()).unwrap_or_else(|| "assets/fl27".into())
}

pub fn fit_text(f: f32) -> &'static str {
    if f >= 0.8 { "passt er sehr gut" } else if f >= 0.45 { "passt er gut" } else if f >= 0.0 { "passt er bedingt" } else { "passt er kaum" }
}

// verein.json rivalries (excerpt)
const DERBIES: [(&str, &str, &str); 12] = [
    ("Bayern München", "Borussia Dortmund", "Klassiker"), ("Borussia Dortmund", "FC Schalke 04", "Revierderby"), ("Real Madrid CF", "FC Barcelona", "Clásico"),
    ("Real Madrid CF", "Atlético Madrid", "Madrider Derby"), ("AC Milan", "Inter Milan", "Mailänder Derby"), ("AS Roma", "SS Lazio", "Römisches Derby"),
    ("Arsenal FC", "Tottenham Hotspur", "Nordlondon-Derby"), ("Liverpool FC", "Everton FC", "Merseyside-Derby"), ("Manchester City", "Manchester United", "Manchester-Derby"),
    ("Celtic FC", "Rangers FC", "Old Firm"), ("Newcastle United", "Sunderland AFC", "Tyne-Wear-Derby"), ("Galatasaray SK", "Fenerbahçe SK", "Interkontinental-Derby"),
];
pub fn is_derby(a: &str, b: &str) -> Option<&'static str> {
    // the rivals of the real data first (realdaten.py rivalries, as sim/verein.py adds them), then the built-in pairs
    if let Some(l) = crate::realdaten::derby(a, b) { return Some(l); }
    DERBIES.iter().find(|(x, y, _)| (a.contains(x) && b.contains(y)) || (a.contains(y) && b.contains(x))).map(|d| d.2)
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn einigung_ausserhalb_des_fensters() {
        let w = World::load(include_str!("../assets/world.json"), 3);
        let mut c = Career::new(w, 0, "Test");
        c.world.day = 60;
        assert!(c.in_window().is_none());
        let andere = (0..c.world.clubs.len()).find(|&x| x != c.own && c.world.clubs[x].squad.len() > 15).unwrap();
        let p = c.world.clubs[andere].squad[5];
        assert!(c.can_open(p).is_ok() || c.can_open(p).unwrap_err() != crate::inhalt::t("ui.farm.das_transferfenster_ist_geschlossen"), "talks outside the window");
        let mut n = c.negs.first().cloned().unwrap_or_else(|| { let i = c.open_neg(p).unwrap(); c.negs[i].clone() });
        n.pid = p; n.renew = false; n.vorvertrag = false; n.fee = 1e6; n.wage = 1e6; n.years = 3;
        c.complete(n);
        assert_eq!(c.world.players[p].team, andere as i32, "agreed, not yet moved");
        c.world.day = WINDOWS[1].0;
        c.geplante_wechsel();
        assert_eq!(c.world.players[p].team, c.own as i32, "moved when the window opens");
    }
    #[test]
    fn two_seasons() {
        let w = World::load(include_str!("../assets/world.json"), 3);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(5);
        let mut c = Career::new(w, own, "Test");
        let mut matches = 0;
        let mut cup_games = 0;
        let mut goals = 0u32;
        for _ in 0..(2 * date::SEASON_DAYS + 5) {
            if c.today_match().is_some() {
                if matches!(c.today_match(), Some((Wo::Pokal(_), _))) { cup_games += 1; }
                let r = c.prepare_match(None);
                goals += (r.hg + r.ag) as u32;
                c.apply_own_match(r);
                matches += 1;
                c.answer_press(Some(1));
            }
            // a few user actions on the way: buy a free agent, answer decisions
            if c.world.day == 5 && c.world.season == 0 {
                if let Some(p) = (0..c.world.players.len()).find(|&p| c.world.players[p].team >= 0 && c.world.players[p].team != c.own as i32 && c.world.players[p].ovr < 70.0 && c.verkauf_gesperrt(p).is_none()) {
                    let i = c.open_neg(p).unwrap();
                    c.negs[i].fee = c.negs[i].ask;
                    c.submit_neg(i);
                }
            }
            if let Some(i) = c.negs.iter().position(|n| n.state == NegState::Counter && n.stage == 1) { c.negs[i].wage = c.negs[i].need * 1.1; c.submit_neg(i); }
            for i in 0..c.inbox.len() { if !c.inbox[i].done && matches!(c.inbox[i].action, Action::Complaint { .. } | Action::Graduate { .. }) { c.answer(i, true, 0); } }
            if c.fired { let j = c.inbox.iter().position(|m| matches!(m.action, Action::JobOffer { .. }) && !m.done); if let Some(j) = j { c.answer(j, true, 0); } }
            // a new club may play on the day the job starts
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            // cup ties against amateur clubs are simulated with the cup (pokalregeln.rs), not played as own matches
            cup_games += c.world.cups.iter().flat_map(|k| k.fx.iter()).filter(|x| x.f.day == c.world.day && (x.f.h == c.own || x.f.a == c.own) && (c.world.ist_amateur(x.f.h) || c.world.ist_amateur(x.f.a))).count();
            c.end_day().unwrap_or_else(|e| panic!("{e}"));
        }
        assert_eq!(c.world.season, 2);
        assert!(cup_games >= 1, "das eigene Team hat kein Pokalspiel gespielt");
        let mut ids: Vec<u32> = c.world.players.iter().map(|p| p.id).collect();
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n, "doppelte Spieler-Kennungen");
        // the test coach decides carelessly and gets sacked now and then: gaps between jobs cost matches
        assert!(matches >= 50, "{matches} {:?}", c.history);
        let per = goals as f32 / matches as f32;
        assert!(per > 1.5 && per < 4.5, "{per}");
        // all leagues finished a full season: every team played 2(n-1) matches in season 2? fresh fixtures exist
        for l in &c.world.leagues { assert!(!l.fixtures.is_empty()); }
        let json = serde_json::to_string(&c).unwrap();
        let back: Career = serde_json::from_str(&json).unwrap();
        assert_eq!(back.world.day, c.world.day);
        eprintln!("matches {matches}, goals/match {per:.2}, own league {}, rank {}, history {:?}", c.world.leagues[c.league()].name, c.world.rank_of(c.own), c.history);
    }
}

#[cfg(test)]
mod calib {
    use super::*;
    #[test]
    fn season_calibration() {
        let w = World::load(include_str!("../assets/world.json"), 11);
        let mut c = Career::new(w, 0, "Test");
        for _ in 0..303 {
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            // the statistics need the leagues, not the job: a sacking (a run of bad luck in the demo world) ends the loop
            if c.end_day().is_err() { break; }
        }
        for detail in 0..3u8 {
            let (mut n, mut g, mut hw, mut d) = (0f32, 0f32, 0f32, 0f32);
            for l in c.world.leagues.iter().filter(|l| l.detail == detail) {
                for f in l.fixtures.iter().filter(|f| f.hg >= 0) { n += 1.0; g += (f.hg + f.ag) as f32; if f.hg > f.ag { hw += 1.0 } else if f.hg == f.ag { d += 1.0 } }
            }
            eprintln!("detail {detail}: {n} games, goals {:.2}, home {:.2}, draw {:.2}", g / n, hw / n, d / n);
        }
        let l = c.league();
        let t = c.world.table(l);
        eprintln!("champion {} {} pts", c.world.clubs[t[0].club].name, t[0].pts);
    }
}

#[cfg(test)]
mod fl27_karriere {
    use super::*;
    /// Career side of the FL27 link on a temporary career folder (reads the real world database only).
    #[test]
    fn vorbereiten_und_uebernehmen() {
        let _g = crate::game::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let src = fl27::welt_ordner(&fl27::sider()).join("common/etc/pesdb");
        if !fl27::echte_welt(&src) { return; }
        let tmp = std::env::temp_dir().join(format!("pesm-fl27-{}", std::process::id()));
        std::env::set_var("HOME", &tmp);
        let w = World::load(include_str!("../assets/world.json"), 5);
        let own = w.clubs.iter().position(|c| c.id == 396).unwrap();
        let mut c = Career::new(w, own, "Test");
        let pesdb = c.karriere_dir().join("original").join(fl27::WELT_ROOT).join("common/etc/pesdb");
        std::fs::create_dir_all(&pesdb).unwrap();
        for e in std::fs::read_dir(&src).unwrap() { let e = e.unwrap(); std::fs::copy(e.path(), pesdb.join(e.file_name())).unwrap(); }
        // a transfer in the career: a Bournemouth player moves to Sunderland
        let bou = c.world.clubs.iter().position(|x| x.id == 4071).unwrap();
        let p = c.world.clubs[bou].squad[0];
        c.world.clubs[bou].squad.retain(|&x| x != p);
        c.world.clubs[own].squad.push(p);
        c.world.players[p].team = own as i32;
        while c.today_match().is_none() { c.end_day().unwrap(); }
        let t = c.fl27_vorbereiten().unwrap();
        assert!(t.contains("Wechsel"), "{t}"); // the AI clubs also trade in the window up to match day
        let (_, a) = fl27::asg_laden(&fl27::karriere_pesdb(&c.karriere_dir())).unwrap();
        let mine: Vec<&fl27::Asg> = a.iter().filter(|x| x.team == 396).collect();
        let keeper = mine.iter().find(|x| x.platz() == 0).unwrap();
        assert_eq!(keeper.pid, c.world.players[c.current_xi()[0].1].id);
        assert!(mine.iter().any(|x| x.pid == c.world.players[p].id));
        // the reader saw the match: build its state from both starting elevens
        let l = c.league();
        let f = c.fixture(c.today_match().unwrap());
        let known: std::collections::HashSet<u32> = crate::pesdb::PlayerBin::load(&fl27::karriere_pesdb(&c.karriere_dir()).join("Player.bin")).unwrap().index.keys().copied().collect();
        let side = |club: usize, k: u32| -> Vec<fl27::Spieler> {
            c.fl27_order(club, &known).into_iter().take(11).enumerate().map(|(i, pid)| {
                let mut w = vec![(fl27::PASS, [3, 3, 3, 3, 3, 3, 0, 0, 0]), (fl27::PASS_OK, [2, 2, 2, 2, 2, 2, 0, 0, 0])];
                if i == 10 { w.push((fl27::GOAL, [0, k, 0, 0, 0, 0, 0, 0, 0])); }
                fl27::Spieler { platz: i as u8, pid, w }
            }).collect()
        };
        let st = fl27::Stand { s: [side(f.h, 2), side(f.a, 1)] };
        let _ = &known;
        let d = fl27::doc(&st, "test-1".into(), c.world.clubs[f.h].id, c.world.clubs[f.a].id, &Default::default(), None);
        let before = c.world.table(l).iter().find(|r| r.club == own).unwrap().p;
        c.apply_fl27(&d, None).unwrap();
        let r = c.last_result.clone().unwrap();
        assert_eq!((r.hg, r.ag), (2, 1));
        assert_eq!(r.ratings.len(), 22);
        assert_eq!(c.world.table(l).iter().find(|r| r.club == own).unwrap().p, before + 1);
        assert!(c.apply_fl27(&d, None).is_err());
        assert!(c.karriere_dir().join("spiele/test-1.json").exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    /// „Match in FL27 starten“ writes the form bonus right before the start for both teams (step `vorbereiten`, live or not,
    /// goes through `fl27_schreiben`): after it the career's Player.bin holds, for a home and an away player, the true
    /// abilities moved by the bonus of their form arrow — +3 and −3 here — and the career keeps the true values.
    #[test]
    fn zuschlag_direkt_vor_dem_start_fuer_beide() {
        let _g = crate::game::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let src = fl27::welt_ordner(&fl27::sider()).join("common/etc/pesdb");
        if !fl27::echte_welt(&src) { return; }
        let tmp = std::env::temp_dir().join(format!("pesm-zuschlag-{}", std::process::id()));
        std::env::set_var("HOME", &tmp);
        let w = World::from_pesdb(&src, 7).unwrap();
        let own = w.leagues[0].teams[0];
        let mut c = Career::new(w, own, "Test");
        let pesdb = c.karriere_dir().join("original").join(fl27::WELT_ROOT).join("common/etc/pesdb");
        std::fs::create_dir_all(&pesdb).unwrap();
        for e in std::fs::read_dir(&src).unwrap() { let e = e.unwrap(); std::fs::copy(e.path(), pesdb.join(e.file_name())).unwrap(); }
        while c.today_match().is_none() { c.end_day().unwrap(); }
        let f = c.fixture(c.today_match().unwrap());
        let erster = |club: usize| c.world.clubs[club].squad.iter().copied().find(|&p| c.world.players[p].attrs.len() == 25).unwrap();
        let (ph, pa) = (erster(f.h), erster(f.a));
        c.world.players[ph].form = 5.0;
        c.world.players[pa].form = -5.0;
        let echt: Vec<Vec<f32>> = [ph, pa].iter().map(|&p| c.world.players[p].attrs.clone()).collect();
        c.fl27_vorbereiten().unwrap();
        let pb = crate::pesdb::PlayerBin::load(&fl27::karriere_pesdb(&c.karriere_dir()).join("Player.bin")).unwrap();
        for (k, (p, z)) in [(ph, 3.0f32), (pa, -3.0f32)].into_iter().enumerate() {
            let pl = &c.world.players[p];
            assert_eq!(crate::world::form_zuschlag(pl), z);
            let geschrieben: Vec<f32> = pb.attrs(pb.index[&pl.id]).iter().map(|&x| x as f32).collect();
            let soll: Vec<f32> = crate::world::attrs_mit_zuschlag(pl).iter().map(|x| x.round().clamp(40.0, 99.0)).collect();
            assert_eq!(geschrieben, soll, "Spieler {} (Mannschaft {}) mit Zuschlag {z}", pl.name, if k == 0 { "Heim" } else { "Gast" });
            let pos = crate::pesdb::pos_index(&pl.pos);
            let d = crate::pesdb::ovr(&geschrieben, pos) - crate::pesdb::ovr(&echt[k], pos);
            assert!((d - z).abs() < 1.0, "Gesamtwert im Spiel = Stärke {z:+}: {d:.2}");
            assert_eq!(pl.attrs, echt[k], "die Karriere behält die echten Werte");
        }
        eprintln!("Zuschlag vor dem Start geprüft: {} und {}", c.world.players[ph].name, c.world.players[pa].name);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod paket1 {
    /// Inbox load over 60 days on the real world: counts per title (run with --ignored --nocapture).
    #[test]
    #[ignore]
    fn posteingang_last() {
        let d = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&d) { return; }
        let w = World::from_pesdb(&d, 7).unwrap();
        let own = w.clubs.iter().position(|c| c.id == 396).unwrap();
        let mut c = Career::new(w, own, "Test");
        for _ in 0..60 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } c.ultimatum = None; c.end_day().unwrap(); }
        let mut z: std::collections::HashMap<String, usize> = Default::default();
        for m in &c.inbox { *z.entry(format!("{} | {}", m.title.split(':').next().unwrap_or(""), m.body.chars().take(50).collect::<String>())).or_default() += 1; }
        let mut v: Vec<_> = z.into_iter().collect(); v.sort_by(|a, b| b.1.cmp(&a.1));
        eprintln!("{} Nachrichten in 60 Tagen", c.inbox.len());
        for (k, n) in v.iter().take(5) { eprintln!("{n:4}  {k}"); }
        let mut fam: std::collections::HashMap<String, usize> = Default::default();
        for m in &c.inbox { let k = match &m.action { Action::Wahl { art, .. } => art.split(':').next().unwrap_or("").to_string(), Action::None => format!("info:{}", m.from), _ => "aktion".into() }; *fam.entry(k).or_default() += 1; }
        let mut f: Vec<_> = fam.into_iter().collect(); f.sort_by(|a, b| b.1.cmp(&a.1));
        for (k, n) in f.iter().take(25) { eprintln!("F{n:4}  {k}"); }
        let mut tag: std::collections::HashMap<u32, usize> = Default::default();
        for m in &c.inbox { *tag.entry(m.day).or_default() += 1; }
        let mut t: Vec<_> = tag.into_iter().collect(); t.sort_by(|a, b| b.1.cmp(&a.1));
        eprintln!("Tage mit den meisten: {:?}", &t[..6.min(t.len())]);
    }

    use super::*;
    fn tree_sha(d: &std::path::Path) -> Vec<(String, String)> {
        let mut out = vec![];
        let mut stack = vec![d.to_path_buf()];
        while let Some(x) = stack.pop() {
            for e in std::fs::read_dir(&x).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() { stack.push(p); } else { out.push((p.strip_prefix(d).unwrap().display().to_string(), fl27::sha_file(&p).unwrap())); }
            }
        }
        out.sort();
        out
    }
    /// A career on a world read straight from its pesdb (temporary copy of the world): a whole season with the database
    /// written every few match days; new youth players get their own records; the original world stays byte-equal.
    #[test]
    fn season_on_own_database() {
        let _g = crate::game::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let src = fl27::welt_ordner(&fl27::sider());
        if !fl27::echte_welt(&src.join("common/etc/pesdb/Player.bin")) { return; }
        let tmp = std::env::temp_dir().join(format!("pesm-paket1-{}", std::process::id()));
        let welt = tmp.join("SiderAddons/livecpk/_FL26MyWorld");
        for d in ["common/etc/pesdb", "common/character0"] {
            let mut stack = vec![src.join(d)];
            while let Some(x) = stack.pop() {
                for e in std::fs::read_dir(&x).unwrap().flatten() {
                    let p = e.path();
                    let to = welt.join(p.strip_prefix(&src).unwrap());
                    if p.is_dir() { stack.push(p); } else { std::fs::create_dir_all(to.parent().unwrap()).unwrap(); std::fs::copy(&p, &to).unwrap(); }
                }
            }
        }
        std::fs::write(tmp.join("SiderAddons/sider.ini"), "[sider]\r\n\tcpk.root = \".\\livecpk\\_FL26MyWorld\"\r\n").unwrap();
        let vorher = tree_sha(&welt);
        std::env::set_var("HOME", tmp.join("home"));
        let w = World::from_pesdb(&welt.join("common/etc/pesdb"), 3).unwrap();
        assert!(w.players.iter().filter(|p| p.team >= 0).all(|p| p.attrs.len() == 25 && p.pp.len() == 13));
        let own = w.clubs.iter().position(|c| c.id == 396).unwrap();
        let mut c = Career::new(w, own, "Test");
        c.fl27_welt = welt.display().to_string();
        fl27::original_sichern(&c.karriere_dir(), &tmp.join("SiderAddons"), &welt).unwrap();
        let start_ovr: Vec<(u32, f32)> = c.squad().iter().map(|&p| (c.world.players[p].id, c.world.players[p].ovr)).collect();
        let mut written = 0;
        let mut entlassen = None;
        for d in 0..date::SEASON_DAYS {
            if c.today_match().is_some() {
                if d % 60 < 7 { c.fl27_vorbereiten().unwrap(); written += 1; }
                let r = c.prepare_match(None);
                c.apply_own_match(r);
            }
            // the board runs normally (§11/§12); a dismissed coach takes the next offer like a person would
            if c.fired {
                entlassen = Some(c.abs());
                if let Some(j) = c.inbox.iter().position(|m| matches!(m.action, Action::JobOffer { .. }) && !m.done) { c.answer(j, true, 0); }
                if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            }
            if c.fired { c.world.day += 1; continue; }
            if let Err(e) = c.end_day() {
                let gr: Vec<String> = c.mensch.gruende.iter().filter(|g| g.bezug == "Präsidium").map(|g| format!("{} {:+.0} {}", g.abs, g.wert, g.grund)).collect();
                panic!("{e} Tag {d}, Platz {}, Ziel {} ({}), Vertrauen {:.0}\n{}", c.world.rank_of(own), c.club().goal_pos, c.club().goal_kind, c.mensch.vorstand, gr.join("\n"));
            }
        }
        assert!(written >= 3);
        // abilities moved and the rating followed the model
        let moved = start_ovr.iter().filter(|(id, o)| c.world.players.iter().find(|p| p.id == *id).map(|p| (p.ovr - o).abs() > 0.5).unwrap_or(false)).count();
        assert!(moved > 5, "{moved}");
        for p in c.world.players.iter().filter(|p| p.attrs.len() == 25).take(200) {
            assert!((crate::pesdb::ovr(&p.attrs, crate::pesdb::pos_index(&p.pos)) - p.ovr).abs() < 0.6, "{}", p.name);
        }
        // the season's new players (academy intake) have unique ids and records
        while c.today_match().is_none() && c.next_match().is_some() { c.end_day().unwrap(); }
        c.fl27_vorbereiten().unwrap();
        let pb = crate::pesdb::PlayerBin::load(&fl27::karriere_pesdb(&c.karriere_dir()).join("Player.bin")).unwrap();
        eprintln!("Saison auf der eigenen Datenbank: entlassen an Tag {:?}, Verein am Ende {}, Präsidium {:.0}", entlassen, c.club().name, c.mensch.vorstand);
        let own = c.own;
        let youth: Vec<&Player> = c.world.players.iter().filter(|p| p.team == own as i32 && p.youth > 0).collect();
        assert!(!youth.is_empty() && youth.iter().all(|p| pb.index.contains_key(&p.id)));
        let y = youth[0];
        let i = pb.index[&y.id];
        assert_eq!((pb.name(i), pb.age(i)), (y.name.clone(), y.age));
        let looks: std::collections::HashSet<Vec<u8>> = youth.iter().map(|p| { let k = pb.index[&p.id]; pb.data[k * crate::pesdb::REC + 12..k * crate::pesdb::REC + 68].to_vec() }).collect();
        assert!(looks.len() * 2 > youth.len(), "neue Spieler sehen gleich aus: {} Vorlagen für {} Spieler", looks.len(), youth.len());
        let mut ids: Vec<u32> = (0..pb.n()).map(|i| pb.id(i)).collect();
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n, "doppelte Kennungen in Player.bin");
        // the original world and its backup are unchanged
        assert_eq!(tree_sha(&welt), vorher);
        fl27::original_pruefen(&c.karriere_dir()).unwrap();
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

#[cfg(test)]
mod ultimatum {
    use super::*;
    /// Fabian 23.11.: the ultimatum has a points target; 4 from 3 saves the job, less ends it.
    #[test]
    fn punkteziel() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        for (ergebnisse, gefeuert) in [(["S", "U", "N"], false), (["U", "U", "N"], true)] {
            let mut c = Career::new(w.clone(), 1, "Test");
            c.ultimatum = Some(c.abs() + 60);
            c.ultimatum_ab = Some(c.results.len());
            for e in ergebnisse { c.results.push(format!("{e} 1:0")); c.ultimatum_spiele_pruefen(); }
            assert_eq!(c.fired, gefeuert, "{ergebnisse:?}");
            assert!(c.ultimatum.is_none());
        }
    }
}

#[cfg(test)]
mod bitten {
    use super::*;
    /// Fabian 23.11.: a trusting board gives in full, then less; a wary one says no and the trust drops.
    #[test]
    fn ganz_halb_nein() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        let d = 95.0 - c.mensch.vorstand; c.klima(0.0, 0.0, d, "Test");
        let tb = c.transfer_budget;
        c.vorstand_bitte(0).unwrap();
        assert!(c.transfer_budget > tb && c.bitten[0].2 == 1.0);
        assert!(c.vorstand_bitte(0).is_err(), "not twice in a row");
        let g = c.gehalt_frei();
        c.vorstand_bitte(1).unwrap();
        assert!(c.gehalt_frei() > g);
        let d = 20.0 - c.mensch.vorstand; c.klima(0.0, 0.0, d, "Test");
        c.vorstand_bitte(2).unwrap();
        assert_eq!(c.bitten.last().unwrap().2, 0.0);
        assert!(c.mensch.vorstand < 20.0);
    }
}

#[cfg(test)]
mod wechsel {
    use super::*;
    /// Fabian 23.11.: a change of club closes the old club's open things and starts quietly.
    #[test]
    fn sauber_und_ruhig() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        c.ultimatum = Some(c.abs() + 10);
        c.ultimatum_ab = Some(0);
        c.switch_club(2, 500_000.0);
        assert!(c.ultimatum.is_none() && c.ultimatum_ab.is_none() && c.press.is_none());
        assert!(c.ruhe_bis > c.abs());
        let n = c.inbox.len();
        c.psyche_an();
        for p in c.squad() { if let Some(x) = c.world.players[p].ps.ch.get_mut(crate::mensch::PROF) { *x = 0.0; } }
        for _ in 0..15 { c.world.day += 1; c.disziplin_tag(); }
        assert!(c.inbox.len() == n, "no incidents in the quiet start");
    }
}

#[cfg(test)]
mod tempo {
    use super::*;
    /// Fabian 23.11.: with a pace the offer lies on the table and the answer comes days later as a message.
    #[test]
    fn antwort_kommt_spaeter() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        c.verhandlungstempo = 1;
        c.world.day = 10; // in the summer window
        let other = (2..c.world.clubs.len()).find(|&k| c.world.clubs[k].squad.len() > 22).unwrap();
        let p = *c.world.clubs[other].squad.iter().min_by(|a, b| c.world.players[**a].ovr.partial_cmp(&c.world.players[**b].ovr).unwrap()).unwrap();
        let i = c.open_neg(p).unwrap();
        let runde = c.negs[i].round;
        c.submit_neg(i);
        assert!(c.negs[i].liegt_bis > 0 && c.negs[i].round == runde, "nothing answered yet");
        let post = c.inbox.len();
        c.world.day += 2;
        c.negotiations();
        assert_eq!(c.negs[i].liegt_bis, 0);
        assert!(c.negs[i].round > runde || c.negs[i].stage == 2 || c.negs[i].state != NegState::Counter, "the round was played");
        assert!(c.inbox.len() > post, "the answer comes as a message");
    }
}

#[cfg(test)]
mod toepfe {
    use super::*;
    /// Fabian 23.11.: three pots; small moves are free, the wage pot never drops below the wages, facilities come
    /// from the infrastructure pot.
    #[test]
    fn schieben_und_grenzen() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        c.wirtschaft_an();
        let (tb, gf) = (c.transfer_budget, c.gehalt_frei());
        let x = crate::world::round_money(c.topf_frei(0) * 0.05);
        c.budget_schieben(0, 1, x).unwrap();
        assert!((c.transfer_budget - (tb - x)).abs() < 1.0 && (c.gehalt_frei() - (gf + x)).abs() < 1.0);
        assert!(c.budget_schieben(1, 0, c.gehalt_frei() + 1e6).is_err(), "not below current wages");
        let i = c.infra();
        c.infra_topf = Some(0.0);
        assert!(c.ausbauen(1).is_err(), "no money in the infrastructure pot");
        c.infra_topf = Some(i.max(1e9));
        c.ausbauen(1).unwrap();
        assert!(c.infra() < i.max(1e9));
    }
}

#[cfg(test)]
mod gala {
    use super::*;
    /// The gala shows its categories on its day (late October) and only once.
    #[test]
    fn gala_am_tag() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        let p = c.squad()[0];
        c.gala = vec![("ballon".into(), p, c.own), ("mueller".into(), p, c.own)];
        c.world.day = 87;
        c.gala_tag();
        assert_eq!(c.gala.len(), 2, "not before its day");
        c.world.day = 88;
        let f = c.feiern.len();
        c.gala_tag();
        assert!(c.gala.is_empty() && c.feiern.len() == f + 2);
    }
}

#[cfg(test)]
mod start_stamina {
    use super::*;
    /// The file for BlankState222's module: the two team ids first, then player id = fitness (below 100 only).
    #[test]
    fn format() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        let p = c.squad()[0];
        c.world.players[p].fit = 73.4;
        let t = c.start_stamina_text().unwrap();
        let mut z = t.lines();
        assert!(z.next().unwrap().starts_with("teams = "));
        assert!(t.contains(&format!("{} = 73", c.world.players[p].id)));
        assert!(z.all(|l| l.split(" = ").nth(1).and_then(|x| x.parse::<u32>().ok()).map_or(false, |f| (1..100).contains(&f))));
    }
}

#[cfg(test)]
mod jugendliga {
    use super::*;
    /// FC-26 wish: rival academies have real players who score, and one can be signed into the own academy.
    #[test]
    fn echte_gegner_und_verpflichtung() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        assert!(c.youth[1].rows.iter().filter(|r| !r.own).all(|r| r.kader.len() == 18));
        for _ in 0..8 { c.youth_round(); }
        assert!(c.youth[1].rows.iter().flat_map(|r| r.kader.iter()).any(|s| s.tore > 0), "rival players score");
        c.transfer_budget = 1e9;
        let row = c.youth[1].rows.iter().position(|r| !r.own).unwrap();
        let n = c.world.players.len();
        c.jugend_verpflichten(1, row, 0).unwrap();
        assert_eq!(c.world.players.len(), n + 1);
        assert!(c.world.players[n].youth > 0 && c.world.players[n].team == c.own as i32);
        assert_eq!(c.youth[1].rows[row].kader.len(), 17);
    }
}

#[cfg(test)]
mod datenpakete {
    use super::*;
    /// Statistik 8.1: a bought data package adds the league to the extended numbers for the season, once.
    #[test]
    fn kaufen_und_gilt() {
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        let l = (0..c.world.leagues.len()).find(|&l| c.world.leagues[l].detail == 2 && !c.analyst_ligen().contains(&l)).unwrap_or(0);
        c.transfer_budget = 1e9;
        c.datenpaket_kaufen(l).unwrap();
        assert!(c.analyst_ligen().contains(&l));
        assert!(c.datenpaket_kaufen(l).is_err());
    }
}

#[cfg(test)]
mod spielstand {
    use super::*;
    /// Fabian 23.11.: rename a save (file and career folder move along) and make a second copy with its own CPK.
    #[test]
    fn umbenennen_und_kopie() {
        let _g = crate::game::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let w = World::load(include_str!("../assets/world.json"), 5);
        let mut c = Career::new(w, 1, "Test");
        c.save_name = format!("PESMTest{}", std::process::id());
        c.fl27_cpk = "PESMTest01".into();
        std::fs::create_dir_all(c.karriere_dir().join("pesdb")).unwrap();
        std::fs::write(c.karriere_dir().join("pesdb/Team.bin"), b"x").unwrap();
        c.save().unwrap();
        let neu = format!("Umbenannt{}", std::process::id());
        c.umbenennen(&neu).unwrap();
        assert_eq!(c.save_name, neu);
        assert!(c.save_path().exists() && c.karriere_dir().join("pesdb/Team.bin").exists());
        assert!(c.umbenennen(&neu).is_err(), "same name");
        let (k, rx) = c.kopie().unwrap();
        rx.recv().unwrap().unwrap();
        let kopie = load(&save_dir().join(format!("{k}.json"))).unwrap();
        assert!(kopie.fl27_cpk.is_empty(), "the copy gets its own CPK");
        assert!(kopie.karriere_dir().join("pesdb/Team.bin").exists());
        for n in [neu, k] { let _ = std::fs::remove_dir_all(save_dir().join(&n)); let _ = std::fs::remove_file(save_dir().join(format!("{n}.json"))); let _ = std::fs::remove_file(save_dir().join(format!("{n}.info"))); }
    }
}

impl Career {
    /// The other matches of our league on this day, played now at our kick-off so the live screen can show them
    /// minute by minute (Fabian 24.11.); the day's end then skips them. Empty when today is not a league day for us.
    pub fn parallel_spielen(&mut self) -> Vec<(usize, usize, MatchResult)> {
        let (day, l, own) = (self.world.day, self.league(), self.own);
        if !self.world.leagues[l].fixtures.iter().any(|f| f.day == day && f.hg < 0 && (f.h == own || f.a == own)) { return vec![]; }
        let level = self.world.leagues[l].detail;
        let mut v = vec![];
        for fi in 0..self.world.leagues[l].fixtures.len() {
            let f = self.world.leagues[l].fixtures[fi];
            if f.day != day || f.hg >= 0 || f.h == own || f.a == own { continue; }
            let res = self.world.play(f.h, f.a, level, None, None);
            self.world.apply_result(l, fi, &res);
            v.push((f.h, f.a, res));
        }
        v
    }
}

#[cfg(test)]
mod direktmatch {
    use super::*;
    /// Direktstart stage 2 (Fabian 26.11.): module and flow files land in the Sider folder, sider.ini gets the line
    /// once, and the line names the teams with the rules of the match (final, semi-final, single knock-out).
    #[test]
    fn einrichten_und_zeile() {
        let s = std::env::temp_dir().join(format!("pesm-direktmatch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&s);
        let da = std::path::Path::new("assets/fl27/direktmatch/flow").is_dir();
        assert_eq!(fl27::direktmatch_einrichten(&s, std::path::Path::new("assets/fl27")).unwrap(), da);
        assert!(!da || s.join("modules/pesm_direktmatch.lua").is_file() && s.join("content/pesm-direktmatch/flow/Exhibition/ExhibiSideSetting.json").is_file());
        let ini = crate::livepatch::ini_mit_lua("[sider]\r\n\tlua.module = \"a.lua\"\r\n", "pesm_direktmatch.lua");
        assert!(ini.contains("\tlua.module = \"pesm_direktmatch.lua\"\r\n") && crate::livepatch::ini_mit_lua(&ini, "pesm_direktmatch.lua") == ini);
        assert_eq!(fl27::direktmatch_zeile(101, 103, 10, true, true, Some(53)), "101 103 0 1 10 menue vl=1 em=1 finale=1 ast=1");
        assert_eq!(fl27::direktmatch_zeile(5, 6, 0, false, false, Some(52)), "5 6 0 1 0 menue vl=0 em=0 mi=52 ast=1");
        fl27::direktmatch_schreiben(&s, "1 2 0 1 0 menue").unwrap();
        assert_eq!(std::fs::read_to_string(s.join("flw/direktmatch.txt")).unwrap(), "1 2 0 1 0 menue\r\n");
        let _ = std::fs::remove_dir_all(&s);
        // a cup final of the career: final mark, extra time and penalties
        let mut c = Career::new(crate::world::testwelt(3), 2, "Test");
        let own = c.own;
        let ci = c.world.cups.iter().position(|x| x.kind == 0).unwrap();
        c.world.cups[ci].fx.push(crate::cups::CupFx { f: Fixture { day: c.world.day, h: own, a: 3, hg: -1, ag: -1, zeit: 0 }, stage: 34, leg: 0, neutral: true, pens: None, et: false });
        let z = c.direktmatch_text().unwrap();
        assert!(z.starts_with(&format!("{} {} ", c.world.clubs[own].id, c.world.clubs[3].id)) && z.contains("vl=1 em=1 finale=1"), "{z}");
    }
}
