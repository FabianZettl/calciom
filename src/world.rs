// The football world: leagues, clubs, players and the match engine.
// Numbers come from the original FL world simulation (pesm/fl/sim/config/*.json); see MECHANICS.md.
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------- random numbers
#[derive(Serialize, Deserialize, Clone)]
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        // splitmix64
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    pub fn f(&mut self) -> f32 { (self.next() >> 40) as f32 / (1u64 << 24) as f32 }
    pub fn range(&mut self, a: f32, b: f32) -> f32 { a + (b - a) * self.f() }
    pub fn int(&mut self, a: i32, b: i32) -> i32 { a + (self.next() % ((b - a + 1) as u64)) as i32 }
    pub fn chance(&mut self, p: f32) -> bool { self.f() < p }
    pub fn gauss(&mut self, sd: f32) -> f32 {
        let u1 = self.f().max(1e-6);
        let u2 = self.f();
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos() * sd
    }
    pub fn tri(&mut self, lo: f32, hi: f32, mode: f32) -> f32 {
        let u = self.f();
        let c = (mode - lo) / (hi - lo);
        if u < c { lo + ((hi - lo) * (mode - lo) * u).sqrt() } else { hi - ((hi - lo) * (hi - mode) * (1.0 - u)).sqrt() }
    }
    pub fn poisson(&mut self, lam: f32) -> u32 {
        let l = (-lam).exp();
        let (mut k, mut p) = (0, 1.0f32);
        loop {
            p *= self.f();
            if p <= l { return k; }
            k += 1;
        }
    }
    pub fn weighted(&mut self, w: &[f32]) -> usize {
        let tot: f32 = w.iter().sum();
        if tot <= 0.0 { return 0; }
        let mut r = self.f() * tot;
        for (i, x) in w.iter().enumerate() {
            if r < *x { return i; }
            r -= x;
        }
        w.len() - 1
    }
}

// ---------------------------------------------------------------- constants (sim.json "match")
pub const HOME_GOALS: f32 = 1.5;
pub const AWAY_GOALS: f32 = 1.17;
pub const BETA: f32 = 0.072;
pub const RHO: f32 = -0.07;
pub const MAX_GOALS: usize = 9;
pub const MORALE_SCALE: f32 = 1.5;
pub const FITNESS_WEIGHT: f32 = 0.15;
pub const YELLOW_PER_TEAM: f32 = 1.7;
pub const RED_PER_TEAM: f32 = 0.06;
pub const INJURY_PER_PLAYER: f32 = 0.02;
pub const INJURY_DAYS: [(u16, f32); 6] = [(3, 0.3), (7, 0.25), (14, 0.2), (28, 0.12), (56, 0.08), (120, 0.05)];
pub const FATIGUE_PER_MATCH: f32 = 16.0;
/// Keeps the mean grade where it was before the actions counted (measured with statistik.rs notenschnitt_messen).
const NOTE_AUSGLEICH: f32 = 0.22;
pub const RECOVERY_PER_DAY: f32 = 5.0;
pub const OWN_GOAL: f32 = 0.04;
// calendar (sim.json "calendar")
pub const LEAGUE_START: u32 = 14;
pub const LEAGUE_END: u32 = 301;
/// Day 0 is 1 August. The summer window runs 1 July–31 August (Fabian 24.11.: it opens on 1 July, England and most of
/// Europe): its July part lies at the end of the season before and ends "on day 395" (31 August of the next season),
/// its August part is the season's first days. Winter 1–31 January.
pub const WINDOWS: [(u32, u32); 3] = [(0, 30), (153, 183), (334, 395)];
pub const INTL_BREAKS: [(u32, u32); 4] = [(35, 41), (70, 76), (105, 111), (238, 244)];
pub const WINTER_BREAK: (u32, u32) = (139, 152);

pub fn scorer_weight(pos: &str) -> f32 {
    match pos {
        "GK" => 0.01, "CB" => 0.5, "LB" | "RB" => 0.6, "DMF" => 0.8, "CMF" => 1.6, "LMF" | "RMF" => 2.2,
        "AMF" => 3.0, "LWF" | "RWF" => 3.4, "SS" => 4.0, "CF" => 5.0, _ => 1.0,
    }
}
/// 0 goalkeeper, 1 defence, 2 midfield, 3 attack
pub fn part_of(pos: &str) -> usize {
    match pos {
        "GK" => 0,
        "CB" | "LB" | "RB" => 1,
        "DMF" | "CMF" | "LMF" | "RMF" | "AMF" => 2,
        _ => 3,
    }
}
pub fn pos_de(pos: &str) -> &'static str {
    // English: the PES codes themselves (GK, CB …)
    if crate::inhalt::sprache() != crate::inhalt::Sprache::De { return crate::pesdb::POS.iter().copied().find(|p| *p == pos).unwrap_or("?"); }
    match pos {
        "GK" => "TW", "CB" => "IV", "LB" => "LV", "RB" => "RV", "DMF" => "DM", "CMF" => "ZM", "LMF" => "LM", "RMF" => crate::id("RM"),
        "AMF" => "OM", "LWF" => "LA", "RWF" => "RA", "SS" => "HS", "CF" => "ST", _ => "?",
    }
}
// training.json individual.pos.near
const NEAR: [(&str, &str); 12] = [
    (crate::id("LB"), crate::id("RB")), (crate::id("LMF"), crate::id("RMF")), (crate::id("LWF"), crate::id("RWF")), (crate::id("LMF"), crate::id("LWF")), (crate::id("RMF"), crate::id("RWF")), (crate::id("CMF"), crate::id("DMF")),
    (crate::id("CMF"), crate::id("AMF")), (crate::id("AMF"), crate::id("SS")), (crate::id("SS"), crate::id("CF")), (crate::id("CB"), crate::id("DMF")), (crate::id("LB"), crate::id("LMF")), (crate::id("RB"), crate::id("RMF")),
];
/// Position misfit penalty (sim.json pos_penalty 14/8/4, GK swap 40). The FL data here has no per-position
/// proficiency, so "near" positions count as level 2 (4) and the same line as level 1 (8).
pub fn pos_penalty(pos: &str, slot: &str) -> f32 {
    if pos == slot { return 0.0; }
    if (pos == crate::id("GK")) != (slot == crate::id("GK")) { return 40.0; }
    if NEAR.iter().any(|(a, b)| (*a == pos && *b == slot) || (*b == pos && *a == slot)) { return 4.0; }
    if part_of(pos) == part_of(slot) { return 8.0; }
    14.0
}

// ---------------------------------------------------------------- data
#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub pos: String,
    pub age: u8,
    pub ovr: f32,
    pub pot: f32,
    pub talent: f32,
    pub no: u8,
    pub team: i32, // club index, -1 free agent, -2 retired
    pub youth: u8, // 0 first team, 17 U17, 19 U19 (own academy only)
    pub form: f32,
    pub fit: f32,
    /// Base fitness 0–100 (Konzept 3.3): slow, from holidays, endurance work, friendlies and camps; caps the
    /// condition. 0 = not started yet (own squad only, vorbereitung.rs gf_an).
    #[serde(default)]
    pub gf: f32,
    pub morale: f32,
    pub inj: u16,
    pub inj_name: String,
    pub susp: u8,
    pub wage: f64,
    pub contract_end: u32, // last season (index) of the contract
    pub value: f64,
    pub apps: u16,
    pub goals: u16,
    pub assists: u16,
    pub rsum: f32,
    pub rn: u16,
    pub yellow: u8,
    pub last: Vec<f32>,
    pub know: f32,
    pub listed: bool,
    pub persona: u8,
    pub unhappy_days: u16,
    pub benched: u8,
    pub promise: i16,
    pub program: u8, // individual training: 0 none, 1 Fokus, 2 Extraschicht
    pub joined: u32,
    #[serde(default)]
    pub talk_day: u32,
    /// The 25 abilities from Player.bin (empty for the built-in demo world); the rating follows from them.
    #[serde(default)]
    pub attrs: Vec<f32>,
    /// Position levels 0..2 per position (0 none, 1 learned, 2 natural).
    #[serde(default)]
    pub pp: Vec<u8>,
    #[serde(default)]
    pub style: u8,
    /// Special abilities and COM styles as bits (pesdb::SKILLS).
    #[serde(default)]
    pub skills: u64,
    /// Nationality (Player.bin bit 233, 9 bits; the same numbers as country= in fl26world.txt, e.g. 210 Germany, 204 England); 0 unknown.
    #[serde(default)]
    pub nat: u16,
    /// The player has a record in the career's Player.bin.
    #[serde(default)]
    pub db: bool,
    /// Character, morale drivers and relation to the coach (own squad, Paket 3).
    #[serde(default)]
    pub ps: crate::mensch::Psyche,
    /// On loan: home club, last season, wage share of the borrower, purchase option (Paket 3f).
    #[serde(default)]
    pub leihe: Option<crate::transfer::Leihe>,
    /// Body from Player.bin, minimum price on the transfer list, career stations (profile page).
    #[serde(default)]
    pub profil: Profil,
    /// Development per ability of the own players (entwicklung.rs).
    #[serde(default)]
    pub ent: crate::entwicklung::Ent,
    /// The Skill Tree (skilltree.rs, own players).
    #[serde(default)]
    pub st: crate::skilltree::Baum,
}
/// [height cm, weight kg, foot (0 right, 1 left), weak foot use 1..4, weak foot accuracy 1..4]; zeros when unknown (demo world).
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Profil {
    pub koerper: [u8; 5],
    /// Injury history (FC-26 wish, 23.11.): season, day, injury, days out; newest last, at most 30.
    #[serde(default)]
    pub verletzungen: Vec<(u32, u32, String, u16)>,
    pub mindestpreis: f64,
    /// (season, club name, games, goals, average grade) per finished season.
    pub laufbahn: Vec<(u32, String, u16, u16, f32)>,
    /// Abs day + 1 of the last game for a first team that has a second team (0: never; eligibility below, farm.rs).
    #[serde(default)]
    pub oben: u32,
    /// Contract clauses (klauseln.rs, Punkt 6).
    #[serde(default)]
    pub klauseln: crate::klauseln::Klauseln,
    /// This season's league figures per league (the rankings count only what was scored in that competition).
    #[serde(default)]
    pub ligen: Vec<LigaStat>,
    /// This season's statistics from detailed matches (all competitions) and last season's (statistik.rs).
    /// Caps and goals for his national team (nationalteams.rs).
    #[serde(default)]
    pub laenderspiele: u16,
    #[serde(default)]
    pub nat_tore: u16,
    #[serde(default)]
    pub saison: crate::statistik::Stat,
    #[serde(default)]
    pub vorsaison: crate::statistik::Stat,
    /// SoFIFA id of the player (0: not found, sofifa.rs) and his market value as a factor on the model's value (0: none).
    #[serde(default)]
    pub sofifa: u32,
    #[serde(default)]
    pub wert_faktor: f32,
    /// SoFIFA emotion (1 very calm … 5 explosive) and growth profile (sofifa::kurve); 0 unknown.
    #[serde(default)]
    pub emotion: u8,
    #[serde(default)]
    pub wachstum: u8,
}
/// One player's figures in one league this season (league index, games, goals, assists, grade sum and count).
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct LigaStat { pub l: u16, pub apps: u16, pub goals: u16, pub assists: u16, pub rsum: f32, pub rn: u16, #[serde(default)] pub st: crate::statistik::Stat }
impl LigaStat { pub fn note(&self) -> f32 { if self.rn == 0 { 0.0 } else { self.rsum / self.rn as f32 } } }
impl Player {
    /// The player's figures in league `l` this season (zero when he has not played there).
    pub fn in_liga(&self, l: usize) -> LigaStat { self.profil.ligen.iter().find(|x| x.l as usize == l).cloned().unwrap_or_default() }
    pub fn avg_rating(&self) -> f32 { if self.rn == 0 { 0.0 } else { self.rsum / self.rn as f32 } }
    pub fn available(&self) -> bool { self.inj == 0 && self.susp == 0 }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Club {
    /// Key of the regional group (one of several leagues under the same league) it last played in.
    #[serde(default)] pub heimat: Option<String>,
    pub id: u32,
    pub name: String,
    pub short: String,
    pub league: usize,
    pub c1: u32,
    pub c2: u32,
    pub squad: Vec<usize>,
    pub level: f32,
    pub rep: f32,
    pub tv: f64,
    pub gate: f64,
    pub comm: f64,
    pub opex: f64,
    pub balance: f64,
    pub goal_kind: String,
    pub goal_pos: usize,
    pub pressure: f32,
    pub patience: f32,
    pub manager: String,
    pub style: i8, // -1 defensive, 0 balanced, 1 attacking
    pub fam: f32,
    pub static_str: f32,
    /// AI coach (Paket 3r): stable set-up until abs day ki_bis, the style before it.
    #[serde(default)]
    pub ki_bis: u32,
    #[serde(default)]
    pub ki_stil: i8,
    /// The coach from the coach bank (trainerbank.rs); None: generated name or interim.
    #[serde(default)]
    pub trainer: Option<crate::trainerbank::Trainer>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Fixture {
    pub day: u32,
    pub h: usize,
    pub a: usize,
    pub hg: i8,
    pub ag: i8,
    /// Kick-off in minutes after midnight (Fabian 25.11.: real days and times per league); 0 = not set (old saves).
    #[serde(default)]
    pub zeit: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct League {
    pub name: String,
    pub key: String,
    pub teams: Vec<usize>,
    pub fixtures: Vec<Fixture>,
    pub detail: u8, // 0 full, 1 light, 2 summary
    pub level: f32,
    pub deduct: Vec<(usize, i32)>,
    /// Country (display name; empty when unknown), tier, the league above (key), how many go up and down, the cup (key).
    #[serde(default)] pub land: String,
    #[serde(default)] pub stufe: u8,
    #[serde(default)] pub ueber: Option<String>,
    #[serde(default = "drei")] pub auf: u8,
    #[serde(default = "drei")] pub ab: u8,
    #[serde(default)] pub pokal: Option<String>,
    /// fl26world country= number (0 unknown): finds the country in daten/ligaregeln.toml.
    #[serde(default)] pub land_nr: u32,
    /// After a split (ligaregeln.rs): the groups in table order; the table lists them one after the other.
    #[serde(default)] pub gruppen: Vec<Vec<usize>>,
    /// Winner of the title round of only the best (Liguilla, Apertura/Clausura finals); the champion when set.
    #[serde(default)] pub meister: Option<usize>,
    /// A table base taken over from an imported career (club, [p, w, d, l, gf, ga, pts]); the fixtures count on top.
    #[serde(default)] pub basis: Vec<(usize, [i32; 7])>,
}
fn drei() -> u8 { 3 }

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Row {
    pub club: usize,
    pub p: u32,
    pub w: u32,
    pub d: u32,
    pub l: u32,
    pub gf: i32,
    pub ga: i32,
    pub pts: i32,
    pub form: Vec<char>,
}

/// A club's figures in one competition (index as in wettbewerb_zaehlen): matches, possession summed, big chances
/// (a shot of at least statistik.toml grosschance xG) for and against, the team's sums and the opponents' sums.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VereinStat { pub club: usize, pub l: u16, pub spiele: u16, pub besitz: u32, pub gross: u16, pub gross_gegen: u16, pub eigen: crate::statistik::Stat, pub gegen: crate::statistik::Stat }
/// A league's season as it ended: the final table and its scorers (player, goals, assists), best first.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct LigaArchiv { pub season: u32, pub liga: usize, pub tabelle: Vec<Row>, pub torjaeger: Vec<(usize, u16, u16)>, #[serde(default)] pub meister: Option<usize> }

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct MatchEvent {
    pub min: u8,
    pub side: u8, // 0 home 1 away
    pub kind: u8, // 0 goal, 1 own goal, 2 yellow, 3 red, 4 injury, 5 sub, 6 chance
    pub pid: usize,
    pub pid2: Option<usize>, // assist or player coming on
}

/// The ticker's chances after minute `ab` from the real shots (statistik.rs): the biggest misses and saves, up to six
/// per side and match; those up to `ab` stay as they were.
fn chancen_ereignisse(res: &mut MatchResult, ab: u8) {
    res.events.retain(|e| e.kind != 6 || e.min <= ab);
    for side in 0..2u8 {
        let mut v: Vec<&crate::statistik::Schuss> = res.schuesse.iter().filter(|x| x.side == side && x.min > ab && x.aus != crate::statistik::TOR && x.aus != crate::statistik::GEBLOCKT_S).collect();
        v.sort_by_key(|x| std::cmp::Reverse(x.xg));
        let n = (6.0 * (90 - ab.min(89)) as f32 / 90.0).ceil() as usize;
        let neu: Vec<MatchEvent> = v.iter().take(n).map(|x| MatchEvent { min: x.min, side, kind: 6, pid: x.pid, pid2: None }).collect();
        res.events.extend(neu);
    }
}
impl MatchResult {
    /// Which side a player of this match was on (0 home, 1 away): starting eleven, else the side he came on for.
    pub fn seite(&self, p: usize) -> u8 {
        if self.xi_h.iter().any(|x| x.1 == p) { 0 } else if self.xi_a.iter().any(|x| x.1 == p) { 1 }
        else { self.events.iter().find(|e| e.kind == 5 && e.pid2 == Some(p)).map_or(0, |e| e.side) }
    }
    /// A side's sum of one statistic (statistik.rs index).
    pub fn summe(&self, side: u8, i: usize) -> u32 { self.stat.iter().filter(|(p, _)| self.seite(*p) == side).map(|x| crate::statistik::get(&x.1, i)).sum() }
}
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct MatchResult {
    pub h: usize,
    pub a: usize,
    pub hg: u8,
    pub ag: u8,
    pub sh: f32,
    pub sa: f32,
    pub lh: f32,
    pub la: f32,
    pub events: Vec<MatchEvent>,
    pub xi_h: Vec<(String, usize)>,
    pub xi_a: Vec<(String, usize)>,
    pub minutes: Vec<(usize, u8)>,
    pub ratings: Vec<(usize, f32)>,
    pub shots: (u8, u8),
    pub on_target: (u8, u8),
    pub possession: u8,
    pub reason: String,
    pub injuries: Vec<(usize, u16)>,
    #[serde(default)]
    pub pens: Option<(u8, u8)>,
    /// Per player the statistics of this match (statistik.rs); empty for summary-level matches.
    #[serde(default)]
    pub stat: Vec<(usize, crate::statistik::Stat)>,
    /// Every shot with minute, shooter, xG and outcome (statistik.rs).
    #[serde(default)]
    pub schuesse: Vec<crate::statistik::Schuss>,
    #[serde(default)]
    pub et: bool,
}

#[derive(Deserialize)]
struct RawWorld {
    leagues: Vec<RawLeague>,
    teams: Vec<RawTeam>,
}
#[derive(Deserialize)]
struct RawLeague {
    name: String,
    key: String,
    teams: Vec<usize>,
}
#[derive(Deserialize)]
struct RawTeam {
    id: u32,
    name: String,
    league: usize,
    squad: Vec<(u32, String, String, u8, f32, u8)>,
    #[serde(default)]
    color: Option<u32>, // main colour taken from the FL-World crest
}

#[derive(Serialize, Deserialize, Clone)]
pub struct World {
    /// Amateur clubs of this season's national cups (pokalregeln.rs): only in the cups, no players, no league.
    #[serde(default)]
    pub amateure: Vec<(Club, u8)>,
    /// The result profile per league, corrected for its spread of strength (liga_profil; filled on first use, never saved).
    #[serde(skip)]
    pub profil_cache: Vec<Option<(f32, f32, f32)>>,
    /// Home-grown players: player id → club id whose youth he came from (jugend.rs, Akademie-Konzept 6.3).
    #[serde(default)]
    pub eigen: std::collections::BTreeMap<u32, u32>,
    /// Faces of new players: player id → id of the real player whose appearance he wears in FL27 (gesichter.rs).
    #[serde(default)]
    pub gesichter: std::collections::BTreeMap<u32, u32>,
    /// The own tactic's factors for the next play() of that club: (club, own goals, opponent goals) (taktikwirkung.rs).
    #[serde(skip)]
    pub taktik_faktor: Option<(usize, f32, f32)>,
    /// Grades of league matches since the last team of the week / player of the month: (player, grade, goals) (ehrungen.rs).
    #[serde(default)]
    pub noten_woche: Vec<(usize, f32, u8)>,
    #[serde(default)]
    pub noten_monat: Vec<(usize, f32, u8)>,
    pub leagues: Vec<League>,
    pub clubs: Vec<Club>,
    pub players: Vec<Player>,
    pub season: u32,
    pub day: u32,
    pub rng: Rng,
    pub source: String,
    /// Next free player id (new youth players and regens get unique ids above every id of the database).
    #[serde(default)]
    pub next_id: u32,
    /// Paket 2: domestic cups and European competitions of the season, honours, country points, prize money not yet booked.
    #[serde(default)]
    pub cups: Vec<crate::cups::Cup>,
    /// The last winner of each competition by name (title holder, Fabian 23.11.), kept over the season change.
    #[serde(default)]
    pub titelverteidiger: std::collections::HashMap<String, usize>,
    /// Financial fair play of the AI clubs (ideas of other FL tools, 23.11.): fees paid this season per club.
    #[serde(default)]
    pub ki_ausgaben: std::collections::HashMap<usize, f64>,
    #[serde(default)]
    pub titles: Vec<crate::cups::Title>,
    #[serde(default)]
    pub coef: Vec<(String, u32, f32)>,
    /// The clubs' European points per season (club, season, points): the UEFA club coefficient (Fabian 24.11.).
    #[serde(default)]
    pub club_coef: Vec<(usize, u32, f32)>,
    /// The nations' A-teams with their squads (nationalteams.rs).
    #[serde(default)]
    pub nationalteams: Vec<crate::nationalteams::Nationalteam>,
    /// FIFA-style points per nationality, the finished tournaments, goals of the running tournament (nationalteams.rs).
    #[serde(default)]
    pub nat_punkte: std::collections::HashMap<u16, f32>,
    #[serde(default)]
    pub turniere: Vec<crate::nationalteams::Turnier>,
    /// The Nations League groups of this autumn and the season they were drawn for (+1, 0 = none yet).
    #[serde(default)]
    pub nl: Vec<crate::nationalteams::NlGruppe>,
    #[serde(default)]
    pub nl_saison: u32,
    /// The final tables and scorers of the seasons before (Fabian 24.11.: last season stays readable).
    #[serde(default)]
    pub archiv: Vec<LigaArchiv>,
    /// Team figures per club and competition this season and the season before (Fabian 24.11.: xG and co for clubs).
    #[serde(default)]
    pub vstat: Vec<VereinStat>,
    #[serde(default)]
    pub vstat_vor: Vec<VereinStat>,
    #[serde(skip)]
    pub turnier_tore: std::collections::HashMap<usize, u16>,
    #[serde(default)]
    pub cup_money: Vec<(usize, f64)>,
    /// The next match is on neutral ground (a final): no home advantage. Set by the caller right before play().
    #[serde(skip)]
    pub neutral: bool,
    /// Decided play-off, playout and relegation ties of this season: (a, b, winner) (ligaregeln.rs entscheidungen_planen).
    #[serde(default)]
    pub entscheid: Vec<(usize, usize, usize)>,
    /// While planning those ties: the league above and the kind of tie.
    #[serde(skip)]
    pub planen: Option<(usize, &'static str)>,
    /// The round of the knock-out being planned (0 first): each round of a play-off its own stage.
    #[serde(skip)]
    pub ko_runde_nr: u8,
    /// Real FL world: the cups (Competition.bin name, clubs) and the European entries of the database (first season).
    #[serde(default)]
    pub db_cups: Vec<(String, Vec<usize>)>,
    /// Competition key → 16-bit competition id (Competition.bin), for the logos emb_<id>_l.png.
    #[serde(default)]
    pub cids: std::collections::HashMap<String, u16>,
    #[serde(default)]
    pub db_euro: Option<[Vec<usize>; 3]>,
    /// The person's answer for a level knock-out match played in FL27: (home team won, on penalties).
    #[serde(skip)]
    pub ko_vorgabe: Option<(bool, bool)>,
    /// Second teams (second club, first club) and farm-team cooperations (farm.rs); farm_an: set up once.
    #[serde(default)]
    pub zweite: Vec<(usize, usize)>,
    #[serde(default)]
    pub koops: Vec<crate::farm::Koop>,
    #[serde(default)]
    pub farm_an: bool,
    /// Coach bank (trainerbank.rs): set up once; coaches without a club.
    #[serde(default)]
    pub trainer_an: bool,
    #[serde(default)]
    pub trainer_pool: Vec<crate::trainerbank::Trainer>,
    /// daten/ligaregeln.toml (or the career folder's copy), set on start and load.
    #[serde(skip, default = "crate::ligaregeln::eingebaut")]
    pub ligaregeln: std::sync::Arc<crate::ligaregeln::Datei>,
    /// The own club's team spirit for the match strength (set by the career every day; the other clubs have none).
    #[serde(default)]
    pub teamgeist_eigen: Option<(usize, f32)>,
}

// ---------------------------------------------------------------- value and wage (transfers.json)
fn interp(pts: &[(f32, f32)], x: f32) -> f32 {
    if x <= pts[0].0 { return pts[0].1; }
    for w in pts.windows(2) {
        let ((a, fa), (b, fb)) = (w[0], w[1]);
        if x <= b { return fa + (fb - fa) * (x - a) / (b - a); }
    }
    pts[pts.len() - 1].1
}
const VALUE_CURVE: [(f32, f32); 10] = [(50., 4.331), (55., 4.572), (60., 5.341), (65., 6.11), (70., 6.694), (75., 7.211), (80., 7.619), (85., 7.774), (90., 8.135), (95., 8.282)];
const VALUE_AGE: [(f32, f32); 9] = [(17., 0.637), (20., 0.696), (23., 1.147), (26., 1.0), (29., 0.83), (31., 0.717), (33., 0.647), (35., 0.605), (40., 0.426)];
const WAGE_CURVE: [(f32, f32); 10] = [(50., 4.943), (55., 4.851), (60., 5.352), (65., 5.661), (70., 5.863), (75., 6.146), (80., 6.427), (85., 6.628), (90., 7.342), (95., 7.505)];
const WAGE_AGE: [(f32, f32); 9] = [(17., 0.452), (20., 0.648), (23., 0.917), (26., 1.0), (29., 1.094), (31., 1.096), (33., 1.101), (35., 1.151), (40., 1.19)];
// entwicklung.json
const GROWTH: [(f32, f32); 12] = [(15., 3.0), (16., 2.8), (17., 2.6), (18., 2.4), (19., 2.0), (20., 1.6), (21., 1.3), (22., 1.0), (23., 0.8), (24., 0.5), (25., 0.3), (26., 0.2)];
const DECLINE: [(f32, f32); 10] = [(27., -0.05), (28., -0.2), (29., -0.4), (30., -0.7), (31., -1.0), (32., -1.4), (33., -1.8), (34., -2.1), (35., -2.4), (36., -2.8)];
const NORM_OVR: [(f32, f32); 11] = [(15., 46.), (16., 50.), (17., 54.), (18., 58.), (19., 59.), (20., 61.), (21., 62.), (22., 63.), (23., 65.), (24., 66.), (25., 66.)];
const RETIRE: [(u8, f32); 8] = [(33, 0.02), (34, 0.06), (35, 0.15), (36, 0.3), (37, 0.5), (38, 0.7), (39, 0.85), (40, 1.0)];

pub fn yearly_change(age: f32) -> f32 {
    if age <= 26.0 { interp(&GROWTH, age) } else { interp(&DECLINE, age) }
}

fn eff_value_strength(p: &Player) -> f32 {
    let mut e = p.ovr;
    if p.pot > e && p.age <= 24 {
        e += (p.pot - e) * if p.age <= 21 { 0.2 } else { 0.1 };
    }
    e
}
pub fn round_money(v: f64) -> f64 {
    let g = if v >= 1e6 { 50000.0 } else if v >= 1e5 { 10000.0 } else { 5000.0 };
    (v / g).round() * g
}
pub fn league_factor(level: f32, scale: f32, lo: f32, hi: f32) -> f32 {
    ((level - 68.0) / scale).exp().clamp(lo, hi)
}
/// Market-value era (setting, ideas of other FL tools): values × this percent (100 the game's own scale).
pub static WERT_AERA: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(100);
pub fn market_value(p: &Player, season: u32, league_level: f32) -> f64 {
    let mut v = 10f64.powf(interp(&VALUE_CURVE, eff_value_strength(p)) as f64) * interp(&VALUE_AGE, p.age as f32) as f64;
    let years = (p.contract_end as i64 - season as i64 + 1).clamp(1, 3);
    v *= [0.72, 0.9, 1.0][years as usize - 1];
    v *= league_factor(league_level, 60.0, 0.7, 1.3) as f64;
    v *= WERT_AERA.load(std::sync::atomic::Ordering::Relaxed) as f64 / 100.0;
    if p.profil.wert_faktor > 0.0 { v *= p.profil.wert_faktor as f64; }
    round_money(v)
}
pub fn fair_wage(p: &Player, league_level: f32) -> f64 {
    let mut v = 10f64.powf(interp(&WAGE_CURVE, eff_value_strength(p)) as f64) * interp(&WAGE_AGE, p.age as f32) as f64;
    v *= league_factor(league_level, 18.0, 0.5, 1.3) as f64;
    let g = if v >= 1e6 { 5000.0 } else { 1000.0 };
    ((v / g).round() * g).max(30000.0)
}
pub fn contract_years(age: u8) -> u32 {
    if age <= 23 { 4 } else if age <= 28 { 3 } else if age <= 31 { 2 } else { 1 }
}

/// Readable name of a league key (ENGLAND_D1_LEAGUE → England 1; FL_011_LEAGUE stays a code, names come from fl26world.txt later).
/// The real name of a fixed competition key from wettbewerbsnamen.json (fl/sim/config); generated entries
/// such as "Chile D1 Cup" (only the key in title case) are no real name.
pub fn echter_name(key: &str) -> Option<String> {
    static N: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();
    let n = N.get_or_init(|| serde_json::from_str(include_str!("../assets/wettbewerbsnamen.json")).unwrap_or_default());
    n.get(key).filter(|v| v.to_lowercase() != key.replace('_', " ").to_lowercase()).cloned()
}
/// Country of a fixed key (ENGLAND_D1_LEAGUE → England); FL_ keys have none.
pub fn fest_land(key: &str) -> Option<String> {
    if key.starts_with("FL_") || fest_stufe(key) == 0 { return None; }
    league_name(key).rsplit_once(' ').filter(|(_, t)| t.parse::<u32>().is_ok()).map(|(c, _)| c.to_string())
}
pub fn fest_stufe(key: &str) -> u8 { key.split("_D").nth(1).and_then(|t| t.split('_').next()).and_then(|t| t.parse().ok()).unwrap_or(0) }
/// Lower case without accents ("München" → "munchen", "Atlético" → "atletico", "ß" → "ss") for the team search.
pub fn falten(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars().flat_map(|c| c.to_lowercase()) {
        let f = match c {
            'à'..='å' | 'ā' | 'ă' | 'ą' => 'a', 'ç' | 'ć' | 'č' => 'c', 'ď' | 'đ' => 'd', 'è'..='ë' | 'ē' | 'ė' | 'ę' | 'ě' => 'e', 'ì'..='ï' | 'ī' | 'į' | 'ı' => 'i',
            'ñ' | 'ń' | 'ň' => 'n', 'ò'..='ö' | 'ø' | 'ō' | 'ő' => 'o', 'ù'..='ü' | 'ū' | 'ů' | 'ű' => 'u', 'ý' | 'ÿ' => 'y', 'ś' | 'š' | 'ş' => 's', 'ź' | 'ż' | 'ž' => 'z',
            'ł' => 'l', 'ř' => 'r', 'ť' | 'ţ' => 't', 'ğ' => 'g', 'ß' => { o.push_str(crate::id("ss")); continue }, 'æ' => { o.push_str(crate::id("ae")); continue }, c => c,
        };
        o.push(f);
    }
    o
}
/// The team search's prepared text per club: folded name, short name, league name and country (built once per world).
pub struct SuchIndex(Vec<[String; 4]>);
impl World {
    pub fn such_index(&self) -> SuchIndex {
        SuchIndex(self.clubs.iter().map(|c| { let l = &self.leagues[c.league]; [falten(&c.name), falten(&c.short), falten(&l.name), falten(&l.land)] }).collect())
    }
    /// Clubs matching `q` (name, short name, league, country), the best match first: the name starts with it, a word of
    /// the name does, the name holds it, then short name, league and country; equal matches by strength.
    /// `land` and `liga` filter; an empty `q` gives nothing (the start step then shows its league list).
    pub fn suche(&self, idx: &SuchIndex, q: &str, land: Option<&str>, liga: Option<usize>) -> Vec<usize> {
        let q = falten(q.trim());
        if q.is_empty() { return vec![]; }
        let mut v: Vec<(u8, usize)> = idx.0.iter().enumerate().filter_map(|(c, [n, k, l, a])| {
            if land.is_some_and(|x| self.leagues[self.clubs[c].league].land != x) || liga.is_some_and(|x| self.clubs[c].league != x) { return None; }
            let g = if n.starts_with(&q) { 0 } else if n.split([' ', '-', '.']).any(|w| w.starts_with(&q)) { 1 } else if n.contains(&q) { 2 }
                else if k.contains(&q) { 3 } else if l.contains(&q) { 4 } else if a.contains(&q) { 5 } else { return None };
            Some((g, c))
        }).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0).then(self.clubs[b.1].level.partial_cmp(&self.clubs[a.1].level).unwrap()));
        v.into_iter().map(|x| x.1).collect()
    }
}
pub fn league_name(key: &str) -> String {
    let k = key.trim_end_matches("_LEAGUE");
    if let Some((land, tier)) = k.rsplit_once("_D") {
        let land = land.replace('_', " ");
        let de = match land.as_str() { "ENGLAND" => "England", "GERMANY" => crate::inhalt::t("md.pool.land.DE"), "SPAIN" => crate::inhalt::t("md.pool.land.ES"), "ITALY" => crate::inhalt::t("md.pool.land.IT"), "FRANCE" => crate::inhalt::t("md.pool.land.FR"), // roh
            "NETHERLANDS" => crate::inhalt::t("md.pool.land.NL"), "PORTUGAL" => crate::inhalt::t("md.pool.land.PT"), "BELGIUM" => crate::inhalt::t("md.pool.land.BE"), "SCOTLAND" => "Schottland", "TURKEY" => "Türkei", "GREECE" => crate::inhalt::t("ui.world.griechenland"), // roh
            "RUSSIA" => crate::inhalt::t("ui.world.russland"), "DENMARK" => crate::inhalt::t("ui.world.daenemark"), "BRAZIL" => crate::inhalt::t("md.pool.land.BR"), "ARGENTINA" => crate::inhalt::t("md.pool.land.AR"), "CHILE" => crate::inhalt::t("ui.world.chile"), "COLOMBIA" => crate::inhalt::t("ui.world.kolumbien"),
            "US" => "USA", "JAPAN" => crate::inhalt::t("md.pool.land.JP"), "CHINA" => crate::inhalt::t("ui.world.china"), "KSA" => "Saudi-Arabien", x => return format!("{} {}", titlecase(x), tier) };
        return crate::tf!("ui.farm.text", de, tier);
    }
    k.replace('_', " ")
}
fn titlecase(s: &str) -> String { s.split(' ').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase()).unwrap_or_default() }).collect::<Vec<_>>().join(" ") }

/// Money in the chosen language (the one place that formats it).
pub fn money(v: f64) -> String { geld(crate::inhalt::sprache(), v) }
/// German "1,2 Mrd. €", "1,2 Mio. €", "850 Tsd. €", "500 €"; English "€1.2bn", "€1.2m", "€850k", "€500"; negative with a minus sign.
pub fn geld(sprache: crate::inhalt::Sprache, v: f64) -> String {
    let en = sprache != crate::inhalt::Sprache::De;
    let a = v.abs();
    let zehntel_mio = (a / 1e5).round() as i64;
    let tsd = (a / 1e3).round() as i64;
    let zehntel_mrd = (a / 1e8).round() as i64;
    let s = if a >= 1e9 || zehntel_mio >= 10_000 {
        let (g, z) = (zehntel_mrd / 10, zehntel_mrd % 10);
        if en { format!("€{g}.{z}bn") } else { format!("{g},{z} Mrd. €") } // roh: number format of each language
    } else if a >= 1e6 || tsd >= 1000 {
        let (g, z) = (zehntel_mio / 10, zehntel_mio % 10);
        if en { format!("€{g}.{z}m") } else { format!("{g},{z} Mio. €") } // roh: number format of each language
    } else if a >= 1e3 {
        if en { format!("€{tsd}k") } else { format!("{tsd} Tsd. €") } // roh: number format of each language
    } else {
        let e = a.round() as i64;
        if en { format!("€{e}") } else { format!("{e} €") }
    };
    if v < 0.0 { format!("−{s}") } else { s }
}

pub fn personas_t() -> [&'static str; 5] { [crate::inhalt::t("ui.world.fair"), crate::inhalt::t("ui.world.geldgierig"), crate::inhalt::t("as.bez.stufe.4"), crate::inhalt::t("mk.ehrgeizig"), crate::inhalt::t("ui.world.heimatverbunden")] }

// ---------------------------------------------------------------- formations
pub const FORMATIONS: [&str; 5] = ["4-3-3", "4-4-2", "4-2-3-1", "3-5-2", "5-3-2"];
pub fn formation_slots(f: usize) -> [&'static str; 11] {
    match f {
        1 => [crate::id("GK"), crate::id("RB"), crate::id("CB"), crate::id("CB"), crate::id("LB"), crate::id("RMF"), crate::id("CMF"), crate::id("CMF"), crate::id("LMF"), crate::id("CF"), crate::id("CF")],
        2 => [crate::id("GK"), crate::id("RB"), crate::id("CB"), crate::id("CB"), crate::id("LB"), crate::id("DMF"), crate::id("DMF"), crate::id("RWF"), crate::id("AMF"), crate::id("LWF"), crate::id("CF")],
        3 => [crate::id("GK"), crate::id("CB"), crate::id("CB"), crate::id("CB"), crate::id("RMF"), crate::id("DMF"), crate::id("CMF"), crate::id("CMF"), crate::id("LMF"), crate::id("CF"), crate::id("CF")],
        4 => [crate::id("GK"), crate::id("RB"), crate::id("CB"), crate::id("CB"), crate::id("CB"), crate::id("LB"), crate::id("CMF"), crate::id("DMF"), crate::id("CMF"), crate::id("CF"), crate::id("CF")],
        _ => [crate::id("GK"), crate::id("RB"), crate::id("CB"), crate::id("CB"), crate::id("LB"), crate::id("DMF"), crate::id("CMF"), crate::id("CMF"), crate::id("RWF"), crate::id("CF"), crate::id("LWF")], // sim.json slots
    }
}
/// The role of a freely placed slot by its spot (x 0 left .. 1 right, y 0 own goal .. 1 opponent goal), like the game's
/// tactics board: back line, defensive midfield, midfield, attacking midfield, attack; wide or central by x.
pub fn rolle_aus_lage(x: f32, y: f32) -> &'static str {
    let (r, l) = (x > 0.72, x < 0.28);
    if y < 0.33 { if r { crate::id("RB") } else if l { crate::id("LB") } else { crate::id("CB") } }
    else if y < 0.47 { if x > 0.78 { crate::id("RB") } else if x < 0.22 { crate::id("LB") } else { crate::id("DMF") } }
    else if y < 0.6 { if x > 0.75 { crate::id("RMF") } else if x < 0.25 { crate::id("LMF") } else { crate::id("CMF") } }
    else if y < 0.74 { if x > 0.75 { if y < 0.66 { crate::id("RMF") } else { crate::id("RWF") } } else if x < 0.25 { if y < 0.66 { crate::id("LMF") } else { crate::id("LWF") } } else { crate::id("AMF") } }
    else if r { crate::id("RWF") } else if l { crate::id("LWF") } else if y < 0.8 { crate::id("SS") } else { crate::id("CF") }
}
/// Roles of eleven freely placed spots (slot 0 is the keeper): a wide role taken twice becomes the central one of its
/// line (the game allows each wide position once).
pub fn rollen_aus_lagen(spots: &[(f32, f32)]) -> [&'static str; 11] {
    let mut out = [crate::id("GK"); 11];
    for i in 1..11.min(spots.len()) {
        let mut r = rolle_aus_lage(spots[i].0, spots[i].1);
        if out[1..i].contains(&r) { r = match r { "RB" | "LB" => crate::id("CB"), "RMF" | "LMF" => crate::id("CMF"), "RWF" | "LWF" => crate::id("CF"), x => x }; }
        out[i] = r;
    }
    out
}
/// Pitch spot (x 0 left .. 1 right, y 0 own goal .. 1 opponent goal) of each slot.
pub fn formation_spots(f: usize) -> [(f32, f32); 11] {
    match f {
        1 => [(0.5, 0.04), (0.85, 0.25), (0.62, 0.2), (0.38, 0.2), (0.15, 0.25), (0.85, 0.55), (0.62, 0.5), (0.38, 0.5), (0.15, 0.55), (0.62, 0.82), (0.38, 0.82)],
        2 => [(0.5, 0.04), (0.85, 0.25), (0.62, 0.2), (0.38, 0.2), (0.15, 0.25), (0.62, 0.42), (0.38, 0.42), (0.85, 0.66), (0.5, 0.64), (0.15, 0.66), (0.5, 0.86)],
        3 => [(0.5, 0.04), (0.72, 0.2), (0.5, 0.18), (0.28, 0.2), (0.88, 0.55), (0.5, 0.4), (0.68, 0.56), (0.32, 0.56), (0.12, 0.55), (0.62, 0.84), (0.38, 0.84)],
        4 => [(0.5, 0.04), (0.88, 0.3), (0.7, 0.2), (0.5, 0.18), (0.3, 0.2), (0.12, 0.3), (0.72, 0.55), (0.5, 0.45), (0.28, 0.55), (0.62, 0.84), (0.38, 0.84)],
        _ => [(0.5, 0.04), (0.85, 0.24), (0.62, 0.2), (0.38, 0.2), (0.15, 0.24), (0.5, 0.42), (0.7, 0.55), (0.3, 0.55), (0.85, 0.82), (0.5, 0.86), (0.15, 0.82)],
    }
}

// ---------------------------------------------------------------- world building
fn hash(s: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() { h = (h ^ b as u64).wrapping_mul(1099511628211); }
    h
}
fn short_name(name: &str) -> String {
    let skip: Vec<&str> = crate::mensch::arr("namen", "kurzname_ueberspringen").iter().filter_map(|x| x.as_str()).collect();
    let w: Vec<&str> = name.split_whitespace().filter(|w| !skip.contains(w) && w.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)).collect();
    let base = w.first().copied().unwrap_or(name);
    base.chars().filter(|c| c.is_alphabetic()).take(3).collect::<String>().to_uppercase()
}

/// Other three-letter short names from a club name, best first: each word (without the skipped ones) by its first
/// letters, then first letter plus the next consonants, then the first letters of two words.
fn kuerzel_kandidaten(name: &str) -> Vec<String> {
    let skip: Vec<&str> = crate::mensch::arr("namen", "kurzname_ueberspringen").iter().filter_map(|x| x.as_str()).collect();
    let woerter: Vec<String> = name.split_whitespace().filter(|w| !skip.contains(w)).map(|w| w.chars().filter(|c| c.is_alphabetic()).collect::<String>().to_uppercase()).filter(|w| w.chars().count() >= 2).collect();
    let mut v: Vec<String> = woerter.iter().filter(|w| w.chars().count() >= 3).map(|w| w.chars().take(3).collect()).collect();
    for w in &woerter {
        let c: Vec<char> = w.chars().collect();
        let kons: Vec<char> = c[1..].iter().copied().filter(|x| !crate::id("AEIOUÄÖÜY").contains(*x)).collect();
        if kons.len() >= 2 { v.push([c[0], kons[0], kons[1]].iter().collect()); }
        if c.len() >= 3 { v.push([c[0], c[1], c[c.len() - 1]].iter().collect()); }
    }
    for a in &woerter { for b in &woerter { if a != b { v.push(a.chars().take(2).chain(b.chars().take(1)).collect()); v.push(a.chars().take(1).chain(b.chars().take(2)).collect()); } } }
    v
}

impl World {
    /// Short names unique within each country (Bayern and Leverkusen both „BAY“ before): fixed ones from
    /// assets/regeln/kuerzel.toml, then the stronger club keeps its short name, the others take the next free candidate.
    pub fn kuerzel_eindeutig(&mut self) {
        let fest: std::collections::HashMap<String, String> = toml::from_str::<toml::Value>(include_str!("../assets/regeln/kuerzel.toml")).ok()
            .and_then(|v| v.get("verein").and_then(|x| x.as_table()).map(|t| t.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect())).unwrap_or_default();
        let mut gruppen: std::collections::HashMap<String, Vec<usize>> = Default::default();
        for (k, c) in self.clubs.iter().enumerate() { let l = &self.leagues[c.league]; gruppen.entry(if l.land.is_empty() { l.key.clone() } else { l.land.clone() }).or_default().push(k); }
        for c in self.clubs.iter_mut() { if let Some(f) = fest.get(&c.name) { c.short = f.clone(); } }
        for (_, mut g) in gruppen {
            g.sort_by(|a, b| (fest.contains_key(&self.clubs[*b].name), self.clubs[*b].level).partial_cmp(&(fest.contains_key(&self.clubs[*a].name), self.clubs[*a].level)).unwrap());
            let mut belegt: std::collections::HashSet<String> = Default::default();
            for k in g {
                if belegt.insert(self.clubs[k].short.clone()) { continue; }
                let neu = kuerzel_kandidaten(&self.clubs[k].name).into_iter().find(|x| !belegt.contains(x))
                    .unwrap_or_else(|| (2..).map(|i| format!("{}{i}", self.clubs[k].short.chars().take(2).collect::<String>())).find(|x| !belegt.contains(x)).unwrap());
                belegt.insert(neu.clone());
                self.clubs[k].short = neu;
            }
        }
    }
    /// The built-in demo world (world.json, no abilities).
    pub fn load(json: &str, seed: u64) -> World {
        let raw: RawWorld = serde_json::from_str(json).expect("world.json");
        let mut colors = vec![];
        let teams = raw.teams.into_iter().map(|t| {
            colors.push(t.color);
            crate::pesdb::DbTeam { id: t.id, name: t.name, squad: t.squad.into_iter().map(|(id, name, pos, age, ovr, no)| crate::pesdb::DbPlayer { id, name, pos, age, ovr, no, nat: 0, attrs: vec![], pp: vec![], style: 0, skills: 0, koerper: [0; 5] }).collect() }
        }).collect();
        let leagues = raw.leagues.into_iter().map(|l| (l.name, l.key, l.teams)).collect();
        let mut w = World::build(leagues, teams, vec![], colors, seed, crate::inhalt::t("ui.world.eingebaute_welt_fl_world").into(), false);
        w.farm_einrichten();
        w.setup_cups(None);
        w
    }
    /// A world straight from a pesdb folder of an FL world (Paket 1: no Python, no pesdb-cache.json).
    pub fn from_pesdb(dir: &std::path::Path, seed: u64) -> Result<World, String> {
        let d = crate::pesdb::read_world(dir)?;
        let meta: Vec<_> = d.leagues.iter().map(|l| (l.land.clone(), l.stufe, l.ueber.clone(), l.auf, l.ab, l.pokal.clone(), l.land_nr)).collect();
        let leagues = d.leagues.into_iter().map(|l| (l.name, l.key, l.teams)).collect();
        let n = d.teams.len();
        let (cups, euro) = (d.cups, d.euro);
        let mut w = World::build(leagues, d.teams, d.pool, vec![None; n], seed, dir.display().to_string(), true);
        for (l, m) in w.leagues.iter_mut().zip(meta) { (l.land, l.stufe, l.ueber, l.auf, l.ab, l.pokal, l.land_nr) = m; }
        for l in 0..w.leagues.len() { w.make_fixtures(l); } // again, now that countries and tiers are known
        w.ueber_ergaenzen();
        w.laender_benennen();
        w.kuerzel_eindeutig();
        w.db_cups = cups;
        w.cids = d.cids;
        w.db_euro = Some(euro);
        w.farm_einrichten();
        w.setup_cups(None);
        Ok(w)
    }
    fn build(leagues: Vec<(String, String, Vec<usize>)>, teams: Vec<crate::pesdb::DbTeam>, pool: Vec<crate::pesdb::DbPlayer>, colors: Vec<Option<u32>>, seed: u64, source: String, db: bool) -> World {
        let mut rng = Rng(seed);
        let mut players = vec![];
        let mut clubs = vec![];
        let mut league_of = vec![0usize; teams.len()];
        for (li, l) in leagues.iter().enumerate() { for t in &l.2 { league_of[*t] = li; } }
        let mut mk = |p: crate::pesdb::DbPlayer, team: i32, rng: &mut Rng| -> Player {
            // potential (Übergabe Daten §1.7, entwicklung.py _ens): tal = 1 + 0.02·(ovr − norm_ovr[age]) + N(0, 0.35), clipped 0.15..2.3;
            // pot = ovr + REM[age]·tal, above 80 compressed 80 + (pot − 80)·0.5, at most 99; from 26 pot = ovr
            let norm = interp(&NORM_OVR, p.age as f32);
            let talent = (1.0 + 0.02 * (p.ovr - norm) + rng.gauss(0.35)).clamp(0.15, 2.3);
            let rem: f32 = GROWTH.iter().filter(|g| g.0 >= p.age as f32).map(|g| g.1).sum();
            let mut pot = if p.age >= 26 { p.ovr } else { p.ovr + rem * talent };
            if pot > 80.0 { pot = 80.0 + (pot - 80.0) * 0.5; }
            let persona = (rng.next() % 5) as u8;
            Player {
                id: p.id, name: p.name, pos: p.pos, age: p.age, ovr: p.ovr, pot: pot.min(99.0).max(p.ovr), talent, no: p.no,
                team, youth: 0, form: 0.0, fit: 100.0, gf: 0.0, morale: 55.0, inj: 0, inj_name: String::new(), susp: 0, wage: 0.0,
                contract_end: 0, value: 0.0, apps: 0, goals: 0, assists: 0, rsum: 0.0, rn: 0, yellow: 0, last: vec![], know: 0.1,
                listed: false, persona, unhappy_days: 0, benched: 0, promise: 0, program: 0, joined: 0, talk_day: 0, ps: Default::default(), leihe: None, ent: Default::default(), st: Default::default(),
                attrs: p.attrs, pp: p.pp, style: p.style, skills: p.skills, nat: p.nat, db, profil: Profil { koerper: p.koerper, ..Default::default() },
            }
        };
        for (ci, t) in teams.into_iter().enumerate() {
            let h = hash(&t.name);
            let palette = [0xC8102E, 0x003DA5, 0x1E7A46, 0xF2C230, 0x6C1D45, 0x111111, 0xEF7D00, 0x5BC2E7, 0x8A1538, 0x00843D, 0x2B2D84, 0xD40F7D];
            let c1 = colors.get(ci).copied().flatten().unwrap_or(palette[(h % palette.len() as u64) as usize]);
            let c2 = if (h >> 8) % 3 == 0 { 0x111111 } else { 0xF2F2F2 };
            let mut squad = vec![];
            for p in t.squad { squad.push(players.len()); let pl = mk(p, ci as i32, &mut rng); players.push(pl); }
            clubs.push(Club {
                id: t.id, name: t.name.clone(), short: short_name(&t.name), league: league_of[ci], c1, c2, squad, level: 0.0, rep: 0.0,
                tv: 0.0, gate: 0.0, comm: 0.0, opex: 0.0, balance: 0.0, goal_kind: String::new(), goal_pos: 1, pressure: 0.0, patience: 50.0, ki_bis: 0, ki_stil: 0, trainer: None,
                manager: String::new(), style: 0, fam: rng.range(40.0, 70.0), static_str: 0.0, heimat: None,
            });
        }
        for p in pool { let pl = mk(p, -1, &mut rng); players.push(pl); } // players without a league club are free agents
        let leagues = leagues.into_iter().map(|(name, key, teams)| League {
            detail: if key.contains("D1_LEAGUE") { 1 } else { 2 },
            land: fest_land(&key).unwrap_or_default(), stufe: fest_stufe(&key), ueber: None, auf: 3, ab: 3, pokal: None, land_nr: 0, gruppen: vec![], meister: None, basis: vec![],
            name, key, teams, fixtures: vec![], level: 68.0, deduct: vec![],
        }).collect();
        let next_id = players.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        let mut w = World { amateure: vec![], profil_cache: vec![], titelverteidiger: Default::default(), ki_ausgaben: Default::default(), eigen: Default::default(), gesichter: Default::default(), taktik_faktor: None, noten_woche: vec![], noten_monat: vec![], leagues, clubs, players, season: 0, day: 0, rng, source, next_id, cups: vec![], titles: vec![], coef: vec![], club_coef: vec![], cup_money: vec![], neutral: false, entscheid: vec![], planen: None, ko_runde_nr: 0, db_cups: vec![], db_euro: None, ko_vorgabe: None, cids: Default::default(), zweite: vec![], koops: vec![], farm_an: false, trainer_an: false, trainer_pool: vec![], ligaregeln: crate::ligaregeln::eingebaut(), teamgeist_eigen: None, nationalteams: vec![], nat_punkte: Default::default(), turniere: vec![], nl: vec![], nl_saison: 0, archiv: vec![], vstat: vec![], vstat_vor: vec![], turnier_tore: Default::default() };
        w.ueber_ergaenzen();
        w.init_economy();
        w.kuerzel_eindeutig();
        w
    }
    /// Abilities for a new player: those of a player of the same position, moved to the wanted rating; natural at his position.
    /// Empty in a world without abilities (the built-in demo world).
    pub fn template_attrs(&self, pos: &str, ovr: f32) -> (Vec<f32>, Vec<u8>) {
        let Some(t) = self.players.iter().find(|p| p.pos == pos && p.attrs.len() == 25) else { return (vec![], vec![]) };
        let pi = crate::pesdb::pos_index(pos);
        let mut a = t.attrs.clone();
        for _ in 0..3 { let d = ovr - crate::pesdb::ovr(&a, pi); crate::pesdb::shift(&mut a, pi, d); }
        let mut pp = vec![0u8; 13];
        pp[pi as usize] = 2;
        (a, pp)
    }
    /// A unique id for a new player (above every id of the database and of the world).
    pub fn new_id(&mut self) -> u32 {
        if self.next_id == 0 { self.next_id = self.players.iter().map(|p| p.id).max().unwrap_or(0) + 1; }
        self.next_id += 1;
        self.next_id - 1
    }
    /// Rating change through the abilities: with abilities from Player.bin the abilities move and the rating follows the model.
    pub fn grow(&mut self, i: usize, delta: f32) {
        let p = &mut self.players[i];
        if p.attrs.len() == 25 {
            let pos = crate::pesdb::pos_index(&p.pos);
            crate::pesdb::shift(&mut p.attrs, pos, delta);
            p.ovr = crate::pesdb::ovr(&p.attrs, pos).clamp(30.0, 99.0);
        } else {
            p.ovr = (p.ovr + delta).clamp(30.0, 99.0);
        }
    }

    pub fn club_level(&self, c: usize) -> f32 {
        let mut o: Vec<f32> = self.clubs[c].squad.iter().filter(|&&p| self.players[p].youth == 0).map(|&p| self.players[p].ovr).collect();
        o.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let n = o.len().min(16).max(1);
        o.iter().take(16).sum::<f32>() / n as f32
    }

    pub fn refresh_levels(&mut self) {
        for c in 0..self.clubs.len() {
            let l = self.club_level(c);
            self.clubs[c].level = l;
            self.clubs[c].static_str = {
                let mut o: Vec<f32> = self.clubs[c].squad.iter().map(|&p| self.players[p].ovr).collect();
                o.sort_by(|a, b| b.partial_cmp(a).unwrap());
                o.iter().take(11).sum::<f32>() / o.len().min(11).max(1) as f32
            };
        }
        for l in 0..self.leagues.len() {
            let ts = &self.leagues[l].teams;
            self.leagues[l].level = ts.iter().map(|&t| self.clubs[t].level).sum::<f32>() / ts.len().max(1) as f32;
        }
    }

    pub fn league_level_of(&self, p: &Player) -> f32 {
        if p.team >= 0 { self.leagues[self.clubs[p.team as usize].league].level } else { 62.0 }
    }

    pub fn update_values(&mut self) {
        for i in 0..self.players.len() {
            let lv = self.league_level_of(&self.players[i]);
            let v = market_value(&self.players[i], self.season, lv);
            self.players[i].value = v;
        }
    }

    /// transfers.py new_world + ruf.py anchors: contracts, wages, finances, reputation, season goals.
    fn init_economy(&mut self) {
        self.refresh_levels();
        let n = self.clubs.len();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|a, b| self.clubs[*b].level.partial_cmp(&self.clubs[*a].level).unwrap());
        for (r, c) in order.iter().enumerate() {
            self.clubs[*c].rep = 100.0 * (1.0 - r as f32 / n as f32).powi(2);
        }
        let names = ["Hans Müller", "Carlo Bianchi", "Luis Fernández", "Jean Dupont", "Pedro Silva", "Jan de Vries", "John Smith", "Marko Petrović", "Ali Yılmaz", "Takeshi Sato", "Tomasz Nowak", "Lars Eriksen", "Paolo Ferri", "Diego Ramos", "Henrik Berg", "Marc Lefèvre", "Stefan Brandt", "Kenji Mori", "Rui Costa Neves", "Oliver Grant"]; // roh
        for c in 0..n {
            let lg_level = self.leagues[self.clubs[c].league].level;
            let mut sq = self.clubs[c].squad.clone();
            sq.sort_by(|a, b| self.players[*b].ovr.partial_cmp(&self.players[*a].ovr).unwrap());
            let mut bill = 0.0;
            for (rank, &pi) in sq.iter().enumerate() {
                let age = self.players[pi].age;
                let mode = if age <= 24 { 4.0 } else if age <= 29 { 3.0 } else { 1.5 };
                let yl = (self.rng.tri(1.0, 5.0, mode).round() as u32 + if rank < 3 && age <= 31 { 1 } else { 0 }).clamp(1, 5);
                let p = &mut self.players[pi];
                p.contract_end = yl - 1;
                p.wage = fair_wage(p, lg_level);
                bill += p.wage;
            }
            let ratio = self.rng.tri(0.5, 0.95, 0.6) as f64;
            let r = bill / ratio;
            let cl = &mut self.clubs[c];
            cl.tv = r * 0.38;
            cl.gate = r * 0.30;
            cl.comm = r * 0.32;
            cl.opex = r * 0.25;
            cl.balance = r * self.rng.range(0.05, 0.5) as f64;
            cl.manager = names[(hash(&cl.name) % names.len() as u64) as usize].to_string();
            cl.style = ((hash(&cl.name) >> 5) % 3) as i8 - 1;
        }
        self.update_values();
        self.set_goals();
        for l in 0..self.leagues.len() { self.make_fixtures(l); }
    }

    pub fn revenue(&self, c: usize) -> f64 { let cl = &self.clubs[c]; cl.tv + cl.gate + cl.comm }
    /// Wage bill of a club; a second team has no budget of its own, its wages count for the parent club (farm.rs).
    pub fn wage_bill(&self, c: usize) -> f64 {
        if self.erste_von(c).is_some() { return 0.0; }
        let eigen = |k: usize| -> f64 { self.clubs[k].squad.iter().map(|&p| self.players[p].wage).sum() };
        eigen(c) + self.zweite_von(c).map_or(0.0, eigen)
    }
    /// What the second team of `c` costs and earns a year: (wages, income − running costs).
    pub fn zweite_haushalt(&self, c: usize) -> Option<(f64, f64)> {
        let z = self.zweite_von(c)?;
        let cl = &self.clubs[z];
        Some((cl.squad.iter().map(|&p| self.players[p].wage).sum(), cl.tv + cl.gate + cl.comm - cl.opex))
    }

    /// transfers.py _set_goals: goal by strength rank inside the league.
    pub fn set_goals(&mut self) {
        // the goal is the season goal of klima.json (mensch::saisonziel); patience stays the AI boards' measure (transfers.py)
        let goals = [(0.08, 40.0), (0.3, 48.0), (0.7, 58.0), (1.01, 68.0)];
        for l in 0..self.leagues.len() {
            let n = self.leagues[l].teams.len();
            for t in self.leagues[l].teams.clone() {
                let z = crate::mensch::saisonziel(self, t).min(n.max(1));
                let frac = z as f32 / n.max(1) as f32;
                let c = &mut self.clubs[t];
                c.goal_pos = z;
                c.goal_kind = crate::mensch::ziel_wort(z, n).to_string();
                c.patience = goals.iter().find(|g| frac <= g.0).unwrap_or(&goals[3]).1;
                c.pressure = 0.0;
            }
        }
    }

    /// `rounds` match days spread over the season (Saturdays, Wednesdays when needed).
    pub fn match_days(rounds: usize, winter: bool) -> Vec<u32> {
        if rounds == 0 { return vec![]; } // a league with fewer than two clubs plays no matches
        let blocked = |d: u32| INTL_BREAKS.iter().any(|(a, b)| d >= *a && d <= *b) || (winter && d >= WINTER_BREAK.0 && d <= WINTER_BREAK.1);
        let mut cand: Vec<u32> = (LEAGUE_START..=LEAGUE_END).filter(|d| d % 7 == 0 && !blocked(*d)).collect();
        if cand.len() < rounds {
            cand.extend((LEAGUE_START..=LEAGUE_END).filter(|d| d % 7 == 4 && !blocked(*d)));
            cand.sort();
        }
        (0..rounds).map(|i| cand[i * (cand.len() - 1) / (rounds - 1).max(1)]).collect()
    }

    /// comp.py: double round robin by the circle method.
    /// Leagues without a known league above get the one of the same country one tier up (old saves: country from the key).
    pub fn ueber_ergaenzen(&mut self) {
        for i in 0..self.leagues.len() {
            if self.leagues[i].land.is_empty() { self.leagues[i].land = fest_land(&self.leagues[i].key).unwrap_or_default(); self.leagues[i].stufe = fest_stufe(&self.leagues[i].key); }
        }
        for i in 0..self.leagues.len() {
            let (land, stufe) = (self.leagues[i].land.clone(), self.leagues[i].stufe);
            if self.leagues[i].ueber.is_some() || land.is_empty() || stufe < 2 { continue; }
            self.leagues[i].ueber = self.leagues.iter().find(|o| o.land == land && o.stufe + 1 == stufe).map(|o| o.key.clone());
        }
        self.heimat_setzen();
    }
    pub fn table(&self, l: usize) -> Vec<Row> {
        let lg = &self.leagues[l];
        let mut rows: Vec<Row> = lg.teams.iter().map(|&t| Row { club: t, ..Default::default() }).collect();
        let idx = |c: usize, rows: &Vec<Row>| rows.iter().position(|r| r.club == c);
        for f in &lg.fixtures {
            if f.hg < 0 { continue; }
            let (Some(ih), Some(ia)) = (idx(f.h, &rows), idx(f.a, &rows)) else { continue };
            let (hg, ag) = (f.hg as i32, f.ag as i32);
            for (i, gf, ga) in [(ih, hg, ag), (ia, ag, hg)] {
                let r = &mut rows[i];
                r.p += 1;
                r.gf += gf;
                r.ga += ga;
                let c = if gf > ga { r.w += 1; r.pts += 3; 'S' } else if gf == ga { r.d += 1; r.pts += 1; 'U' } else { r.l += 1; 'N' };
                r.form.push(c);
            }
        }
        for (c, b) in &lg.basis { if let Some(r) = rows.iter_mut().find(|r| r.club == *c) { r.p += b[0] as u32; r.w += b[1] as u32; r.d += b[2] as u32; r.l += b[3] as u32; r.gf += b[4]; r.ga += b[5]; r.pts += b[6]; } }
        for (c, d) in &lg.deduct { if let Some(r) = rows.iter_mut().find(|r| r.club == *c) { r.pts -= d; } }
        for r in rows.iter_mut() { let n = r.form.len(); if n > 5 { r.form = r.form[n - 5..].to_vec(); } }
        rows.sort_by(|a, b| (b.pts, b.gf - b.ga, b.gf, self.clubs[a.club].id as i32 * -1).cmp(&(a.pts, a.gf - a.ga, a.gf, self.clubs[b.club].id as i32 * -1)));
        if !lg.gruppen.is_empty() { rows.sort_by_key(|r| lg.gruppen.iter().position(|g| g.contains(&r.club)).unwrap_or(lg.gruppen.len())); }
        rows
    }
    pub fn rank_of(&self, c: usize) -> usize {
        self.table(self.clubs[c].league).iter().position(|r| r.club == c).map(|i| i + 1).unwrap_or(0)
    }

    // ------------------------------------------------------------ match engine (match.py)
    pub fn eff(&self, p: usize, slot: &str) -> f32 {
        let pl = &self.players[p];
        let mut v = pl.ovr - pos_penalty(&pl.pos, slot);
        v *= 1.0 - FITNESS_WEIGHT * (1.0 - pl.fit / 100.0);
        v + (pl.morale - 50.0) / 50.0 * MORALE_SCALE + form_zuschlag(pl) + self.seele(pl)
    }
    /// Self-confidence and the own club's team spirit on the strength in a match, together at most ± seele_hoechstens.
    pub fn seele(&self, pl: &Player) -> f32 {
        let z = zahlen();
        let sv = if pl.ps.an { (pl.ps.sv - z.sv_neutral) / 50.0 * z.sv } else { 0.0 };
        let tg = match self.teamgeist_eigen { Some((c, t)) if pl.team == c as i32 => (t - 50.0) / 50.0 * z.tg, _ => 0.0 };
        (sv + tg).clamp(-z.hoechstens, z.hoechstens)
    }

    /// match.py lineup(): the user's choice first, then a natural player unless someone is clearly better (tol 8), then the rest by worth.
    pub fn lineup(&self, c: usize, formation: usize, pref: &[usize]) -> (Vec<(String, usize, f32)>, Vec<usize>) { self.lineup_slots(c, formation_slots(formation), pref) }
    /// The line-up for given slot roles (the own team with freely moved players, game.rs eigene_slots).
    pub fn lineup_slots(&self, c: usize, slots: [&'static str; 11], pref: &[usize]) -> (Vec<(String, usize, f32)>, Vec<usize>) { self.lineup_slots_mit(c, slots, pref, &|_| 0.0) }
    /// lineup_slots with a bonus per player on top of his strength for the slot (the co-trainer's criteria).
    pub fn lineup_slots_mit(&self, c: usize, slots: [&'static str; 11], pref: &[usize], bonus: &dyn Fn(usize) -> f32) -> (Vec<(String, usize, f32)>, Vec<usize>) {
        let squad: Vec<usize> = self.clubs[c].squad.iter().copied().filter(|&p| self.players[p].youth == 0 && self.spielberechtigt(p, c)).collect();
        let mut pool: Vec<usize> = squad.iter().copied().filter(|&p| self.players[p].available()).collect();
        let mut forced = vec![];
        if pool.len() < 14 {
            let mut rest: Vec<usize> = squad.iter().copied().filter(|p| !pool.contains(p)).collect();
            rest.sort_by(|a, b| self.players[*b].ovr.partial_cmp(&self.players[*a].ovr).unwrap());
            forced = rest.into_iter().take(14 - pool.len()).collect();
            pool.extend(forced.iter());
        }
        // AI coaches who rotate (ki_trainer.toml typ.*.rotiert): a starter below fitness 55 loses 4 points
        // (rotation_kondition / rotation_spielraum), so a substitute who is not much weaker plays
        let rotiert = pref.is_empty() && { let t = self.ki_typ(c);
            crate::regeln::wert_oder("ki_trainer", &format!("typ.{t}.rotiert")).and_then(|v| v.as_bool()).unwrap_or(false) };
        let (rk, rs) = (crate::regeln::f("ki_trainer", "startelf.rotation_kondition"), crate::regeln::f("ki_trainer", "startelf.rotation_spielraum"));
        // a loan player with a play-time promise plays more often at the partner (farm.toml koop.spielzeit_bonus)
        let sb = crate::regeln::f("farm", "koop.spielzeit_bonus");
        let eff = |p: usize, slot: &str| self.eff(p, slot) + bonus(p) - if rotiert && self.players[p].fit < rk { rs } else { 0.0 } + if pref.is_empty() && self.players[p].leihe.as_ref().is_some_and(|l| l.spielzeit) { sb } else { 0.0 };
        let mut got: Vec<Option<usize>> = vec![None; 11];
        let mut used: Vec<usize> = vec![];
        for i in 0..11 {
            if let Some(&p) = pref.get(i) {
                if pool.contains(&p) && !used.contains(&p) { got[i] = Some(p); used.push(p); }
            }
        }
        for i in 0..11 {
            if got[i].is_some() { continue; }
            let free: Vec<usize> = pool.iter().copied().filter(|p| !used.contains(p)).collect();
            if free.is_empty() { break; }
            let best = free.iter().map(|&p| eff(p, slots[i])).fold(f32::MIN, f32::max);
            let nat = free.iter().copied().filter(|&p| self.players[p].pos == slots[i]).max_by(|a, b| eff(*a, slots[i]).partial_cmp(&eff(*b, slots[i])).unwrap());
            if let Some(n) = nat {
                if eff(n, slots[i]) >= best - 8.0 { got[i] = Some(n); used.push(n); }
            }
        }
        for i in 0..11 {
            if got[i].is_some() { continue; }
            let b = pool.iter().copied().filter(|p| !used.contains(p)).max_by(|a, b| eff(*a, slots[i]).partial_cmp(&eff(*b, slots[i])).unwrap());
            if let Some(b) = b { got[i] = Some(b); used.push(b); }
        }
        let xi: Vec<(String, usize, f32)> = (0..11).filter_map(|i| got[i].map(|p| (slots[i].to_string(), p, self.eff(p, slots[i]) - if forced.contains(&p) { 10.0 } else { 0.0 }))).collect();
        let mut bench: Vec<usize> = pool.into_iter().filter(|p| !used.contains(p)).collect();
        bench.sort_by(|a, b| self.players[*b].ovr.partial_cmp(&self.players[*a].ovr).unwrap());
        bench.truncate(9);
        (xi, bench)
    }

    pub fn fam_bonus(&self, c: usize) -> f32 { ((self.clubs[c].fam - 50.0) / 50.0).clamp(-1.0, 1.0) }

    pub fn rates(sh: f32, sa: f32, sty_h: i8, sty_a: i8) -> (f32, f32) {
        let d = sh - sa;
        let mut lh = HOME_GOALS * (BETA * d).exp();
        let mut la = AWAY_GOALS * (-BETA * d).exp();
        let own = |s: i8| match s { 1 => 0.05, -1 => -0.06, _ => 0.0 };
        let opp = |s: i8| match s { 1 => 0.04, -1 => -0.05, _ => 0.0 };
        lh *= (1.0 + own(sty_h)) * (1.0 + opp(sty_a));
        la *= (1.0 + own(sty_a)) * (1.0 + opp(sty_h));
        (lh, la)
    }

    pub fn pmf(lh: f32, la: f32) -> Vec<(usize, usize, f32)> {
        let fact = |k: usize| (1..=k).map(|x| x as f32).product::<f32>();
        let ph: Vec<f32> = (0..=MAX_GOALS).map(|k| (-lh).exp() * lh.powi(k as i32) / fact(k)).collect();
        let pa: Vec<f32> = (0..=MAX_GOALS).map(|k| (-la).exp() * la.powi(k as i32) / fact(k)).collect();
        let mut out = vec![];
        for x in 0..=MAX_GOALS {
            for y in 0..=MAX_GOALS {
                let t = match (x, y) { (0, 0) => 1.0 - lh * la * RHO, (0, 1) => 1.0 + lh * RHO, (1, 0) => 1.0 + la * RHO, (1, 1) => 1.0 - RHO, _ => 1.0 };
                out.push((x, y, ph[x] * pa[y] * t));
            }
        }
        out
    }
    pub fn win_probs(lh: f32, la: f32) -> (f32, f32, f32) {
        let c = Self::pmf(lh, la);
        let tot: f32 = c.iter().map(|x| x.2).sum();
        let h: f32 = c.iter().filter(|x| x.0 > x.1).map(|x| x.2).sum();
        let a: f32 = c.iter().filter(|x| x.0 < x.1).map(|x| x.2).sum();
        (h / tot, 1.0 - (h + a) / tot, a / tot)
    }
    /// The result profile of the league a home team plays in (ligaprofile.toml): home and away goals of equal teams and
    /// the factor on the draw cells that gives the league's share of draws.
    pub fn liga_profil(&mut self, club: usize) -> (f32, f32, f32) {
        let li = self.clubs[club].league;
        if self.profil_cache.len() != self.leagues.len() { self.profil_cache = vec![None; self.leagues.len()]; }
        if let Some(p) = self.profil_cache[li] { return p; }
        let p = self.liga_profil_rechnen(li);
        self.profil_cache[li] = Some(p);
        p
    }
    /// The league's targets turned into rates: the strength model makes uneven pairings score more and draw less than
    /// even ones, so goals and draws are corrected over all pairings of the league (by the clubs' level).
    fn liga_profil_rechnen(&self, li: usize) -> (f32, f32, f32) {
        let l = &self.leagues[li];
        let land = l.land.to_lowercase();
        let tab = crate::regeln::wert("ligaprofile", crate::id("land"));
        let p = if l.stufe >= 2 { &crate::regeln::wert("ligaprofile", crate::id("unterklasse")).clone() } else {
            &tab.as_object().and_then(|o| o.iter().find(|(n, _)| land.contains(n.as_str())).map(|(_, v)| v.clone())).unwrap_or_else(|| crate::regeln::wert("ligaprofile", crate::id("standard")).clone())
        };
        let x = |k: &str| p[k].as_f64().unwrap_or(0.0) as f32;
        let (h, a, ziel) = (x(crate::id("heim")), x(crate::id("gast")), x(crate::id("remis")));
        let (mut fh, mut fa, mut d0, mut n) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for &x in &l.teams { for &y in &l.teams {
            if x == y { continue; }
            let (rh, ra) = Self::rates(self.clubs[x].level, self.clubs[y].level, 0, 0);
            fh += rh / HOME_GOALS; fa += ra / AWAY_GOALS; d0 += Self::win_probs(rh / HOME_GOALS * h, ra / AWAY_GOALS * a).1; n += 1.0;
        } }
        if n == 0.0 { return (h, a, 1.0); }
        let (fh, fa, d0) = (fh / n, fa / n, d0 / n);
        let k = if d0 > 0.0 && d0 < 1.0 && ziel > 0.0 { ziel * (1.0 - d0) / (d0 * (1.0 - ziel)) } else { 1.0 };
        (h / fh, a / fa, k)
    }
    fn draw_score(&mut self, lh: f32, la: f32, remis_k: f32) -> (u8, u8) {
        let cells = Self::pmf(lh, la);
        let w: Vec<f32> = cells.iter().map(|c| if c.0 == c.1 { c.2 * remis_k } else { c.2 }).collect();
        let i = self.rng.weighted(&w);
        (cells[i].0 as u8, cells[i].1 as u8)
    }

    /// Plays one match. level 0 full (events, ratings), 1 light (scorers, injuries), 2 summary (static strengths).
    pub fn play(&mut self, h: usize, a: usize, level: u8, own: Option<(usize, usize, Vec<usize>, i8)>, forced: Option<(u8, u8)>) -> MatchResult {
        let mut res = MatchResult { h, a, ..Default::default() };
        let (sh, sa, xi_h, xi_a, bench_h, bench_a);
        let (mut sty_h, mut sty_a) = (self.clubs[h].style, self.clubs[a].style);
        if level == 2 {
            sh = self.clubs[h].static_str;
            sa = self.clubs[a].static_str;
            xi_h = vec![]; xi_a = vec![]; bench_h = vec![]; bench_a = vec![];
        } else {
            let pick = |w: &World, c: usize| -> (Vec<(String, usize, f32)>, Vec<usize>) {
                match &own {
                    Some((oc, f, pref, _)) if *oc == c => w.lineup(c, *f, pref),
                    _ => w.lineup(c, 0, &[]),
                }
            };
            let (x1, b1) = pick(self, h);
            let (x2, b2) = pick(self, a);
            sh = x1.iter().map(|x| x.2).sum::<f32>() / x1.len().max(1) as f32 + self.fam_bonus(h);
            sa = x2.iter().map(|x| x.2).sum::<f32>() / x2.len().max(1) as f32 + self.fam_bonus(a);
            xi_h = x1; xi_a = x2; bench_h = b1; bench_a = b2;
        }
        if let Some((oc, _, _, s)) = &own {
            if *oc == h { sty_h = *s; } else if *oc == a { sty_a = *s; }
        }
        let (mut lh, mut la) = Self::rates(sh, sa, sty_h, sty_a);
        // the league's own profile (home advantage, goals, draws) on top of the strength model
        let (ph, pa, remis_k) = self.liga_profil(h);
        lh *= ph / HOME_GOALS;
        la *= pa / AWAY_GOALS;
        if let Some((c, fe, fg)) = self.taktik_faktor.take() { if c == h { lh *= fe; la *= fg; } else if c == a { la *= fe; lh *= fg; } }
        if std::mem::take(&mut self.neutral) { let m = (ph + pa) / 2.0; lh *= m / ph; la *= m / pa; } // neutral ground: both the mean
        let (hg, ag) = match forced { Some(x) => x, None => self.draw_score(lh, la, remis_k) };
        res.hg = hg; res.ag = ag; res.sh = sh; res.sa = sa; res.lh = lh; res.la = la;
        if level < 2 {
            res.xi_h = xi_h.iter().map(|x| (x.0.clone(), x.1)).collect();
            res.xi_a = xi_a.iter().map(|x| (x.0.clone(), x.1)).collect();
            for (side, xi, bench, goals) in [(0u8, &xi_h, &bench_h, hg), (1u8, &xi_a, &bench_a, ag)] {
                let opp = if side == 0 { &xi_a } else { &xi_h };
                for _ in 0..goals {
                    let og = self.rng.chance(OWN_GOAL);
                    let min = self.rng.int(1, 90) as u8;
                    if og {
                        let i = self.rng.int(0, opp.len() as i32 - 1) as usize;
                        res.events.push(MatchEvent { min, side, kind: 1, pid: opp[i].1, pid2: None });
                    } else {
                        // nobody takes the lion's share of his team's goals: above 35 % of them his weight falls off steeply
                        let club = if side == 0 { h } else { a };
                        let team_tore: f32 = self.clubs[club].squad.iter().map(|&q| self.players[q].goals as f32).sum::<f32>().max(8.0);
                        let w: Vec<f32> = xi.iter().map(|x| { let pl = &self.players[x.1]; let anteil = pl.goals as f32 / team_tore; scorer_weight(&pl.pos) * (pl.ovr / 75.0).powi(3) * if anteil > 0.35 { (0.35 / anteil).powi(4) } else { 1.0 } }).collect();
                        let sc = xi[self.rng.weighted(&w)].1;
                        // assist: estimated 70 % of goals, weighted towards creative positions
                        let assist = if self.rng.chance(0.7) {
                            let w2: Vec<f32> = xi.iter().map(|x| if x.1 == sc { 0.0 } else { match self.players[x.1].pos.as_str() { "AMF" | "LWF" | "RWF" => 3.0, "CMF" | "LMF" | "RMF" | "SS" => 2.2, "CF" => 1.6, "LB" | "RB" => 1.2, "GK" => 0.05, _ => 0.6 } }).collect();
                            Some(xi[self.rng.weighted(&w2)].1)
                        } else { None };
                        res.events.push(MatchEvent { min, side, kind: 0, pid: sc, pid2: assist });
                    }
                }
                if level == 0 {
                    let ps: Vec<usize> = xi.iter().map(|x| x.1).collect();
                    let n_y = self.rng.poisson(YELLOW_PER_TEAM);
                    for _ in 0..n_y {
                        let w: Vec<f32> = ps.iter().map(|&p| match self.players[p].pos.as_str() { "DMF" => 1.6, "CB" => 1.3, "CMF" => 1.2, "GK" => 0.2, _ => 1.0 }).collect();
                        let p = ps[self.rng.weighted(&w)];
                        let min = self.rng.int(5, 90) as u8;
                        res.events.push(MatchEvent { min, side, kind: 2, pid: p, pid2: None });
                    }
                    if self.rng.chance(RED_PER_TEAM) {
                        let w: Vec<f32> = ps.iter().map(|&p| match self.players[p].pos.as_str() { "DMF" => 1.6, "CB" => 1.4, "GK" => 0.3, _ => 1.0 }).collect();
                        let p = ps[self.rng.weighted(&w)];
                        let min = self.rng.int(10, 90) as u8;
                        res.events.push(MatchEvent { min, side, kind: 3, pid: p, pid2: None });
                    }
                    // minutes and up to three substitutions after minute 55
                    let mut mins: Vec<(usize, u8)> = ps.iter().map(|&p| (p, 90u8)).collect();
                    for e in res.events.iter().filter(|e| e.kind == 3 && e.side == side) {
                        if let Some(m) = mins.iter_mut().find(|m| m.0 == e.pid) { m.1 = e.min; }
                    }
                    let mut free = bench.clone();
                    for _ in 0..3.min(free.len()) {
                        let cands: Vec<usize> = mins.iter().filter(|m| m.1 == 90 && self.players[m.0].pos != crate::id("GK")).map(|m| m.0).collect();
                        if cands.is_empty() || free.is_empty() { break; }
                        let off = cands[self.rng.int(0, cands.len() as i32 - 1) as usize];
                        let m = self.rng.int(55, 85) as u8;
                        let opos = self.players[off].pos.clone();
                        let (bi, on) = free.iter().copied().enumerate().max_by(|x, y| {
                            let fx = self.players[x.1].ovr - if self.players[x.1].pos == opos { 0.0 } else { 6.0 };
                            let fy = self.players[y.1].ovr - if self.players[y.1].pos == opos { 0.0 } else { 6.0 };
                            fx.partial_cmp(&fy).unwrap()
                        }).unwrap();
                        free.remove(bi);
                        if let Some(x) = mins.iter_mut().find(|x| x.0 == off) { x.1 = m; }
                        mins.push((on, 90 - m));
                        res.events.push(MatchEvent { min: m, side, kind: 5, pid: off, pid2: Some(on) });
                    }
                    res.minutes.extend(mins);
                } else {
                    res.minutes.extend(xi.iter().map(|x| (x.1, 90u8)));
                }
                for x in xi.iter() {
                    if self.rng.chance(INJURY_PER_PLAYER) {
                        let w: Vec<f32> = INJURY_DAYS.iter().map(|d| d.1).collect();
                        let days = INJURY_DAYS[self.rng.weighted(&w)].0;
                        res.injuries.push((x.1, days));
                        if level == 0 {
                            let min = self.rng.int(1, 89) as u8;
                            res.events.push(MatchEvent { min, side, kind: 4, pid: x.1, pid2: None });
                        }
                    }
                }
            }
            // shots and possession: estimated from expected goals (not in the original sim)
            let sh_h = (hg as u32 + self.rng.poisson(lh * 7.5)).min(35) as u8;
            let sh_a = (ag as u32 + self.rng.poisson(la * 7.5)).min(35) as u8;
            res.shots = (sh_h, sh_a);
            res.on_target = (hg + ((sh_h - hg) as f32 * 0.3) as u8, ag + ((sh_a - ag) as f32 * 0.3) as u8);
            res.possession = (50.0 + (sh - sa) * 1.2 + self.rng.gauss(3.0)).clamp(28.0, 72.0) as u8;
            self.statistik_rechnen(&mut res);
            if level == 0 {
                let _ = (sh_h, sh_a);
                chancen_ereignisse(&mut res, 0);
                self.ratings(&mut res, &xi_h, &xi_a);
            }
            res.events.sort_by_key(|e| e.min);
        }
        res.reason = self.reason(&res);
        res
    }

    /// The rest of a running match from minute `ab` on, after an intervention of the side `side` (Fabian 23.11.: substitutions,
    /// mentality and shouts act on the match still to come): what happened up to `ab` stays, the rest is drawn again from the
    /// strength of the current eleven `xi` (after substitutions and red cards) times the factors of the intervention.
    pub fn rest_neu(&mut self, res: &mut MatchResult, ab: u8, side: u8, xi: &[usize], f_eigen: f32, f_gegner: f32) {
        let ab = ab.min(89);
        let rest = (90 - ab) as f32 / 90.0;
        res.events.retain(|e| e.min <= ab);
        let club = if side == 0 { res.h } else { res.a };
        let neu = xi.iter().map(|&p| self.eff(p, &self.players[p].pos.clone())).sum::<f32>() / 11.0 + self.fam_bonus(club);
        let alt = if side == 0 { res.sh } else { res.sa };
        let d = neu - alt;
        let (l_eigen, l_gegner) = if side == 0 { (res.lh, res.la) } else { (res.la, res.lh) };
        let le = l_eigen * rest * (0.06 * d).exp() * f_eigen;
        let lg = l_gegner * rest * (-0.06 * d).exp() * f_gegner;
        // the opponents on the pitch: their eleven minus those sent off or taken off
        let weg: Vec<usize> = res.events.iter().filter(|e| e.kind == 3 || e.kind == 5).map(|e| e.pid).collect();
        let rein: Vec<usize> = res.events.iter().filter(|e| e.kind == 5 && e.side != side).filter_map(|e| e.pid2).collect();
        let gegner: Vec<usize> = (if side == 0 { &res.xi_a } else { &res.xi_h }).iter().map(|x| x.1).chain(rein).filter(|p| !weg.contains(p)).collect();
        for (s, elf, lam) in [(side, xi.to_vec(), le), (1 - side, gegner.clone(), lg)] {
            if elf.is_empty() { continue; }
            let opp = if s == side { &gegner } else { xi };
            for _ in 0..self.rng.poisson(lam) {
                let min = self.rng.int(ab as i32 + 1, 90) as u8;
                if self.rng.chance(OWN_GOAL) && !opp.is_empty() {
                    let i = self.rng.int(0, opp.len() as i32 - 1) as usize;
                    res.events.push(MatchEvent { min, side: s, kind: 1, pid: opp[i], pid2: None });
                    continue;
                }
                let w: Vec<f32> = elf.iter().map(|&p| scorer_weight(&self.players[p].pos) * (self.players[p].ovr / 75.0).powi(3)).collect();
                let sc = elf[self.rng.weighted(&w)];
                let assist = if self.rng.chance(0.7) && elf.len() > 1 {
                    let w2: Vec<f32> = elf.iter().map(|&p| if p == sc { 0.0 } else { match self.players[p].pos.as_str() { "AMF" | "LWF" | "RWF" => 3.0, "CMF" | "LMF" | "RMF" | "SS" => 2.2, "CF" => 1.6, "LB" | "RB" => 1.2, "GK" => 0.05, _ => 0.6 } }).collect();
                    Some(elf[self.rng.weighted(&w2)])
                } else { None };
                res.events.push(MatchEvent { min, side: s, kind: 0, pid: sc, pid2: assist });
            }
            for _ in 0..self.rng.poisson(YELLOW_PER_TEAM * rest) {
                let p = elf[self.rng.int(0, elf.len() as i32 - 1) as usize];
                res.events.push(MatchEvent { min: self.rng.int(ab as i32 + 1, 90) as u8, side: s, kind: 2, pid: p, pid2: None });
            }
        }
        res.events.sort_by_key(|e| e.min);
        res.hg = res.events.iter().filter(|e| e.side == 0 && e.kind <= 1).count() as u8;
        res.ag = res.events.iter().filter(|e| e.side == 1 && e.kind <= 1).count() as u8;
        res.injuries.retain(|(p, _)| res.events.iter().any(|e| e.kind == 4 && e.pid == *p));
        res.shots = (res.shots.0.max(res.hg), res.shots.1.max(res.ag));
        res.on_target = (res.on_target.0.max(res.hg).min(res.shots.0), res.on_target.1.max(res.ag).min(res.shots.1));
        // minutes from the starting elevens, substitutions and red cards
        let mut mins: Vec<(usize, u8)> = res.xi_h.iter().chain(res.xi_a.iter()).map(|x| (x.1, 90u8)).collect();
        for e in res.events.iter().filter(|e| e.kind == 3 || e.kind == 5) {
            if let Some(m) = mins.iter_mut().find(|m| m.0 == e.pid) { m.1 = e.min; }
            if let Some(on) = e.pid2.filter(|_| e.kind == 5) { mins.push((on, 90 - e.min)); }
        }
        res.minutes = mins;
        self.statistik_rechnen(res);
        chancen_ereignisse(res, ab);
        res.events.sort_by_key(|e| e.min);
        res.ratings.clear();
        let xh: Vec<(String, usize, f32)> = res.xi_h.iter().map(|x| (x.0.clone(), x.1, self.eff(x.1, &x.0))).collect();
        let xa: Vec<(String, usize, f32)> = res.xi_a.iter().map(|x| (x.0.clone(), x.1, self.eff(x.1, &x.0))).collect();
        self.ratings(res, &xh, &xa);
        res.reason = self.reason(res);
    }

    /// The players' statistics of a detailed match (statistik.rs) and the team shots from them.
    fn statistik_rechnen(&mut self, res: &mut MatchResult) {
        let mut r = Rng(self.rng.next());
        (res.stat, res.schuesse) = crate::statistik::spiel(self, res, &mut r);
        if res.stat.is_empty() { return; }
        let summe = |res: &MatchResult, side: u8, i: usize| res.summe(side, i).min(255) as u8;
        let shots = (summe(res, 0, crate::statistik::SCHUESSE), summe(res, 1, crate::statistik::SCHUESSE));
        let on_target = (summe(res, 0, crate::statistik::AUFS_TOR), summe(res, 1, crate::statistik::AUFS_TOR));
        res.shots = shots;
        res.on_target = on_target;
    }

    /// match.py _ratings plus an assist bonus (estimated) for the Sofascore-style grade.
    fn ratings(&mut self, res: &mut MatchResult, xi_h: &[(String, usize, f32)], xi_a: &[(String, usize, f32)]) {
        let seed = res.hg as u64 * 7 + res.ag as u64;
        let mut subs: Vec<(usize, u8)> = vec![];
        for e in res.events.iter().filter(|e| e.kind == 5) { subs.push((e.pid2.unwrap(), e.side)); }
        for (side, xi, gf, ga) in [(0u8, xi_h, res.hg, res.ag), (1u8, xi_a, res.ag, res.hg)] {
            let mut people: Vec<(usize, f32)> = xi.iter().map(|x| (x.1, x.2)).collect();
            for (p, s) in &subs { if *s == side { people.push((*p, self.eff(*p, &self.players[*p].pos.clone()))); } }
            for (p, e) in people {
                let mut b = 6.3 + if gf > ga { 0.25 } else if gf < ga { -0.25 } else { 0.0 } + 0.04 * (e - 70.0) / 10.0;
                b += 0.9 * res.events.iter().filter(|x| x.kind == 0 && x.pid == p).count() as f32;
                b += 0.5 * res.events.iter().filter(|x| x.kind == 0 && x.pid2 == Some(p)).count() as f32;
                b -= 1.2 * res.events.iter().filter(|x| x.kind == 1 && x.pid == p).count() as f32;
                b -= 1.4 * res.events.iter().filter(|x| x.kind == 3 && x.pid == p).count() as f32;
                if matches!(self.players[p].pos.as_str(), "GK" | "CB" | "LB" | "RB") && ga == 0 { b += 0.5; }
                // the actions of the match (statistik.rs, Konzept 8.5): saves, key passes, duels, pass accuracy, ball losses
                if let Some((_, st)) = res.stat.iter().find(|x| x.0 == p) {
                    use crate::statistik::*;
                    let g = |i| get(st, i) as f32;
                    b += (0.2 * g(PARADEN)).min(1.0) + (0.1 * g(SCHLUESSEL)).min(0.3) + (0.06 * g(TACKL_OK) + 0.04 * g(ABGEF)).min(0.35)
                        + (0.08 * (g(AUFS_TOR) - g(TORE))).min(0.24) + (0.05 * g(DRIBB_OK)).min(0.2) - (0.03 * g(BALLVERL)).min(0.3);
                    if g(PAESSE) >= 20.0 { b += (g(PAESSE_OK) / g(PAESSE) - 0.82) * 1.5; }
                    b -= NOTE_AUSGLEICH;
                }
                let jitter = ((self.players[p].id as u64).wrapping_mul(2654435761).wrapping_add(seed + self.rng.next() % 97) % 1000) as f32 / 1000.0 - 0.5;
                res.ratings.push((p, ((b + jitter).clamp(3.0, 10.0) * 10.0).round() / 10.0));
            }
        }
    }

    fn reason(&self, r: &MatchResult) -> String {
        let (ph, _, pa) = Self::win_probs(r.lh, r.la);
        let (hn, an) = (&self.clubs[r.h].name, &self.clubs[r.a].name);
        let gap = (r.sh - r.sa).abs();
        let kl = if gap >= 8.0 { crate::inhalt::t("ui.world.klar_besser_besetzt") } else if gap >= 4.0 { crate::inhalt::t("ui.world.etwas_besser_besetzt") } else { crate::inhalt::t("ui.world.auf_augenhoehe") };
        let v = (r.hg as usize * 3 + r.ag as usize + r.h) % 3;
        if r.hg == r.ag {
            if r.hg == 0 {
                [crate::inhalt::t("ui.world.torloses_remis_beide_abwehrreihen_stande"), crate::inhalt::t("ui.world.null_zu_null_ein_spiel"), crate::inhalt::t("ui.world.viel_kampf_wenig_glanz_keiner")][v].to_string()
            } else if ph.max(pa) >= 0.5 {
                let fav = if ph > pa { hn } else { an };
                crate::tf!("ui.world.war_kam_aber_nicht_ueber", fav, kl)
            } else {
                [crate::inhalt::t("ui.world.ein_gerechtes_unentschieden_zwischen_zwe"), crate::inhalt::t("ui.world.beide_hatten_ihre_phasen_am"), crate::inhalt::t("ui.world.offener_schlagabtausch_ohne_sieger")][v].to_string()
            }
        } else {
            let (w, l, pw) = if r.hg > r.ag { (hn, an, ph) } else { (an, hn, pa) };
            let base = if pw >= 0.5 {
                [crate::tf!("ui.world.war_und_gewann_verdient_gegen", w, kl, l), crate::tf!("ui.world.die_favoritenrolle_angenommen_setzte_sic", w, l), crate::tf!("ui.world.spielte_seine_qualitaet_aus_fand", w, l)][v].clone()
            } else if pw >= 0.35 {
                [crate::tf!("ui.world.ein_enges_spiel_das_fuer", w), crate::tf!("ui.world.nutzte_seine_chancen_besser_als", w, l), crate::tf!("ui.world.auf_augenhoehe_aber_war_vor", w)][v].clone()
            } else {
                [crate::tf!("ui.world.ueberraschung_schlug_den_favoriten", w, l), crate::tf!("ui.world.war_doch_hatte_den_laengeren", l, kl, w), crate::tf!("ui.world.ein_aussenseitersieg_wuchs_ueber_sich", w)][v].clone()
            };
            if (r.hg as i32 - r.ag as i32).abs() >= 3 { crate::tf!("ui.world.am_ende_eine_deutliche_angelegenheit", base) } else { base }
        }
    }

    /// Saves from before the per-league figures: the season so far from each first-team player's totals, under his
    /// club's league. shortcut: cup goals of the season so far count as league goals once; exact from the next season.
    pub fn ligen_nachtragen(&mut self) {
        if self.players.iter().any(|p| !p.profil.ligen.is_empty()) { return; }
        for ci in 0..self.clubs.len() {
            let l = self.clubs[ci].league;
            if l >= self.leagues.len() { continue; }
            for &p in &self.clubs[ci].squad {
                let pl = &mut self.players[p];
                if pl.youth != 0 || pl.apps == 0 { continue; }
                pl.profil.ligen.push(LigaStat { l: l as u16, apps: pl.apps, goals: pl.goals, assists: pl.assists, rsum: pl.rsum, rn: pl.rn, st: vec![] });
            }
        }
    }
    /// Keeps every league's final table and top 30 scorers before the season turns (the last five seasons).
    pub fn saison_archivieren(&mut self) {
        let s = self.season;
        for l in 0..self.leagues.len() {
            let tabelle = self.table(l);
            if tabelle.iter().all(|r| r.p == 0) { continue; }
            let mut tj: Vec<(usize, u16, u16)> = self.players.iter().enumerate().filter_map(|(p, pl)| pl.profil.ligen.iter().find(|x| x.l as usize == l).filter(|x| x.goals + x.assists > 0).map(|x| (p, x.goals, x.assists))).collect();
            tj.sort_by_key(|x| (std::cmp::Reverse(x.1), std::cmp::Reverse(x.2)));
            tj.truncate(30);
            let meister = self.meister(l);
            self.archiv.push(LigaArchiv { season: s, liga: l, tabelle, torjaeger: tj, meister });
        }
        self.archiv.retain(|a| a.season + 5 > s);
    }
    /// Writes a played match into the world: table, stats, fitness, morale, form, cards, injuries.
    pub fn apply_result(&mut self, l: usize, fi: usize, res: &MatchResult) {
        let f = &mut self.leagues[l].fixtures[fi];
        f.hg = res.hg as i8;
        f.ag = res.ag as i8;
        self.apply_stats(res);
        self.wettbewerb_zaehlen(l, res);
        // the awards of the week and month read these grades (ehrungen.rs)
        for &(p, r) in &res.ratings {
            let g = res.events.iter().filter(|e| e.kind == 0 && e.pid == p).count() as u8;
            self.noten_woche.push((p, r, g));
            self.noten_monat.push((p, r, g));
        }
    }
    /// Per-competition figures for the rankings: a league by its index, a cup or European competition as 1000 + its
    /// index (Fabian 24.11.: scorers etc. for every competition, the Champions League too).
    pub fn wettbewerb_zaehlen(&mut self, l: usize, res: &MatchResult) {
        let mut eintrag = |w: &mut World, p: usize| -> usize {
            let v = &mut w.players[p].profil.ligen;
            if let Some(i) = v.iter().position(|x| x.l as usize == l) { i } else { v.push(LigaStat { l: l as u16, ..Default::default() }); v.len() - 1 }
        };
        for &(p, _) in &res.minutes { let i = eintrag(self, p); self.players[p].profil.ligen[i].apps += 1; }
        for (p, s) in &res.stat { let i = eintrag(self, *p); crate::statistik::add(&mut self.players[*p].profil.ligen[i].st, s); }
        for &(p, r) in &res.ratings { let i = eintrag(self, p); let x = &mut self.players[p].profil.ligen[i]; x.rsum += r; x.rn += 1; }
        for e in res.events.iter().filter(|e| e.kind == 0) {
            let i = eintrag(self, e.pid); self.players[e.pid].profil.ligen[i].goals += 1;
            if let Some(a) = e.pid2 { let i = eintrag(self, a); self.players[a].profil.ligen[i].assists += 1; }
        }
        // the clubs' figures: the team's sums and the opponent's (detailed matches only)
        if res.stat.is_empty() { return; }
        let gross = (crate::regeln::f("statistik", "grosschance") * 1000.0) as u16;
        for (side, club) in [(0u8, res.h), (1u8, res.a)] {
            let i = match self.vstat.iter().position(|x| x.club == club && x.l as usize == l) { Some(i) => i, None => { self.vstat.push(VereinStat { club, l: l as u16, ..Default::default() }); self.vstat.len() - 1 } };
            let summe = |sd: u8| -> crate::statistik::Stat { (0..crate::statistik::N).map(|k| res.summe(sd, k)).collect() };
            let (eigen, gegen) = (summe(side), summe(1 - side));
            let v = &mut self.vstat[i];
            v.spiele += 1;
            v.besitz += if side == 0 { res.possession as u32 } else { 100 - res.possession.min(100) as u32 };
            v.gross += res.schuesse.iter().filter(|x| x.side == side && x.xg >= gross).count() as u16;
            v.gross_gegen += res.schuesse.iter().filter(|x| x.side != side && x.xg >= gross).count() as u16;
            crate::statistik::add(&mut v.eigen, &eigen);
            crate::statistik::add(&mut v.gegen, &gegen);
        }
    }
    /// Player side of a played match (league or cup): bans, apps, fitness, goals, cards, injuries, grades, form, morale.
    pub fn apply_stats(&mut self, res: &MatchResult) {
        if res.minutes.is_empty() { return; } // welt.py: summary-level matches skip _apply
        // welt.py _apply (line 266): every squad player of both clubs serves one match of a ban
        for c in [res.h, res.a] { for &p in &self.clubs[c].squad { let pl = &mut self.players[p]; if pl.susp > 0 { pl.susp -= 1; } } }
        let heute = self.heute();
        let oben: Vec<bool> = res.minutes.iter().map(|(p, _)| self.players[*p].team >= 0 && self.zweite_von(self.players[*p].team as usize).is_some()).collect();
        for (p, s) in &res.stat { crate::statistik::add(&mut self.players[*p].profil.saison, s); }
        for (k, (p, m)) in res.minutes.iter().enumerate() {
            let pl = &mut self.players[*p];
            if oben[k] { pl.profil.oben = heute + 1; }
            pl.apps += 1;
            pl.fit = (pl.fit - FATIGUE_PER_MATCH * *m as f32 / 90.0).max(20.0);
            pl.benched = 0;
        }
        for e in &res.events {
            let p = &mut self.players[e.pid];
            match e.kind {
                0 => { p.goals += 1; p.morale = (p.morale + 1.0).min(95.0); }
                2 => { p.yellow += 1; if p.yellow % 5 == 0 { p.susp += 1; } }
                3 => { p.susp += 1; } // welt.py line 299: a red card is a one-match ban
                _ => {}
            }
            if e.kind == 0 { if let Some(a) = e.pid2 { self.players[a].assists += 1; } }
        }
        for (p, d) in &res.injuries {
            let names = injury_names(*d);
            let pl = &mut self.players[*p];
            pl.inj = pl.inj.max(*d);
            pl.inj_name = names[(pl.id as usize) % names.len()].to_string();
            let eintrag = (self.season, self.day, pl.inj_name.clone(), *d);
            pl.profil.verletzungen.push(eintrag);
            if pl.profil.verletzungen.len() > 30 { pl.profil.verletzungen.remove(0); }
        }
        for (p, r) in &res.ratings {
            let pl = &mut self.players[*p];
            pl.rsum += r;
            pl.rn += 1;
            pl.last.push(*r);
            if pl.last.len() > 5 { pl.last.remove(0); }
            pl.form = (pl.form * 0.8 + (r - 6.4) * 1.1).clamp(-3.0, 3.0);
        }
        // the swing from what happened in the match (Fabian 24.11.: a keeper who lets in three goes down, a brace up)
        let d = |k: &str| crate::regeln::f("entwicklung_pesm", &format!("dynamik.{k}"));
        let heute = self.heute();
        let mut anstoss: Vec<(usize, &'static str, f32)> = vec![];
        for &(p, _) in &res.minutes {
            let tore = res.events.iter().filter(|e| e.kind == 0 && e.pid == p).count();
            if tore >= 3 { anstoss.push((p, crate::id("dreierpack"), d(crate::id("dreierpack")))); } else if tore == 2 { anstoss.push((p, crate::id("doppelpack"), d(crate::id("doppelpack")))); }
            if res.events.iter().any(|e| e.kind == 3 && e.pid == p) { anstoss.push((p, crate::id("rot"), d(crate::id("rot")))); }
            let r = res.ratings.iter().find(|x| x.0 == p).map(|x| x.1).unwrap_or(6.5);
            if r >= d("note_top_ab") { anstoss.push((p, "note_top", d("note_top"))); } else if r <= d("note_flop_bis") { anstoss.push((p, "note_flop", d("note_flop"))); }
            let pl = &self.players[p];
            let gegen = if pl.team == res.h as i32 { res.ag } else { res.hg };
            match (pl.pos.as_str(), gegen) {
                ("GK", g) if g >= 3 => anstoss.push((p, "drei_gegentore", d("gk_drei_gegentore"))), // roh
                ("GK", 0) => anstoss.push((p, "zu_null", d("gk_zu_null"))), // roh
                ("CB" | "LB" | "RB", g) if g >= 3 => anstoss.push((p, "abwehr_drei", d("abwehr_drei_gegentore"))), // roh
                _ => {}
            }
        }
        for (p, k, w) in anstoss {
            let ps = &mut self.players[p].ps;
            ps.dyn_z = (ps.dyn_z + w).clamp(-d(crate::id("grenze")), d(crate::id("grenze")));
            ps.dyn_gruende.push((heute, k.to_string(), w));
            if ps.dyn_gruende.len() > 5 { ps.dyn_gruende.remove(0); }
        }
        let dm = |gf: u8, ga: u8| if gf > ga { 3.0 } else if gf < ga { -3.0 } else { 0.0 };
        for (side, gf, ga) in [(res.h, res.hg, res.ag), (res.a, res.ag, res.hg)] {
            let played: Vec<usize> = res.minutes.iter().map(|m| m.0).collect();
            let sq = self.clubs[side].squad.clone();
            for p in sq {
                let pl = &mut self.players[p];
                if played.contains(&p) { pl.morale = (pl.morale + dm(gf, ga)).clamp(10.0, 95.0); } else if pl.youth == 0 && pl.available() { pl.benched = pl.benched.saturating_add(1); }
            }
        }
    }

    /// One day passes for everybody (welt.py _recover): fitness, injuries, form and morale drift.
    pub fn recover_all(&mut self) {
        // the swing from events fades (a brace is forgotten in a few weeks) and old reasons go with it
        let (abkling, heute) = (crate::regeln::f("entwicklung_pesm", "dynamik.abklingen_je_tag"), self.heute());
        for p in self.players.iter_mut() {
            // a swing confirmed for weeks becomes his new standard (Fabian 25.11.): one rating point moves from the swing
            // into the abilities, at most a few a season, upwards only up to his potential (entwicklung_pesm.toml [dauer])
            {
                let d = |k: &str| crate::regeln::f("entwicklung_pesm", &format!("dauer.{k}"));
                let z = p.ps.dyn_z;
                p.ps.dyn_tage = if z >= d(crate::id("ab")) { p.ps.dyn_tage.max(0) + 1 } else if z <= -d(crate::id("ab")) { p.ps.dyn_tage.min(0) - 1 } else { 0 };
                if p.ps.dyn_dauer.0 != self.season { p.ps.dyn_dauer = (self.season, 0.0); }
                let rauf = p.ps.dyn_tage as f32 >= d(crate::id("tage")) && p.ovr + 1.0 <= p.pot && p.ps.dyn_dauer.1 < d("saison_max");
                let runter = p.ps.dyn_tage as f32 <= -d(crate::id("tage")) && p.ps.dyn_dauer.1 > -d("saison_max");
                if (rauf || runter) && p.attrs.len() == 25 {
                    let s = if rauf { 1.0 } else { -1.0 };
                    let pi = crate::pesdb::pos_index(&p.pos);
                    crate::pesdb::shift(&mut p.attrs, pi, s);
                    p.ovr = crate::pesdb::ovr(&p.attrs, pi).clamp(30.0, 99.0);
                    p.ps.dyn_z -= s;
                    p.ps.dyn_dauer.1 += s;
                    p.ps.dyn_tage = 0;
                    p.ps.dyn_gruende.push((heute, crate::id(if rauf { "dauer_hoch" } else { "dauer_runter" }).to_string(), s));
                }
            }
            if p.ps.dyn_z != 0.0 {
                p.ps.dyn_z *= abkling;
                if p.ps.dyn_z.abs() < 0.3 { p.ps.dyn_z = 0.0; p.ps.dyn_gruende.clear(); }
                p.ps.dyn_gruende.retain(|g| heute < g.0 + 28);
            }
            // welt.py _recover (line 311): only players still tired or injured recover and drift
            if p.team < 0 || (p.fit >= 100.0 && p.inj == 0) { continue; }
            // base fitness (own squad, vorbereitung.rs) caps the condition: 60 + 0.4 × base fitness
            let deckel = if p.gf > 0.0 { (crate::regeln::f("vorbereitung", "grundfitness.kondition_basis") + crate::regeln::f("vorbereitung", "grundfitness.kondition_je_gf") * p.gf).min(100.0) } else { 100.0 };
            p.fit = (p.fit + RECOVERY_PER_DAY).min(deckel.max(p.fit.min(100.0)));
            if p.inj > 0 { p.inj -= 1; if p.inj == 0 { p.inj_name.clear(); } }
            p.form *= 0.97;
            p.morale += (50.0 - p.morale) * 0.02;
        }
    }

    /// Weekly share of the season step of entwicklung.py _step (line 187): young players grow towards their
    /// potential, (pot − ovr) × GROW[age] / REM[age]; from 27 the decline table. Training and play time scale it,
    /// match grades add a performance part, noise 0.9 / 0.6 per season. own_tr = training plan multiplier (tr_mult).
    pub fn develop_week(&mut self, own: usize) {
        const TL: f32 = 5.0; // staff training level 1..10 of the other clubs (level_default); the own club's comes from its staff
        let lerp = |a: f32, b: f32| a + (b - a) * (TL - 1.0) / 9.0;
        let rem = |a: f32| GROWTH.iter().filter(|g| g.0 >= a).map(|g| g.1).sum::<f32>();
        // play share: own club and full leagues from games (minutes_full 2700), others by squad rank (proxy_share 0.85/0.4/0.1)
        let mut proxy = vec![0.1f32; self.players.len()];
        for c in &self.clubs {
            let mut sq = c.squad.clone();
            sq.sort_by(|a, b| self.players[*b].ovr.partial_cmp(&self.players[*a].ovr).unwrap());
            for (r, p) in sq.into_iter().enumerate() { proxy[p] = if r < 11 { 0.85 } else if r < 16 { 0.4 } else { 0.1 }; }
        }
        for i in 0..self.players.len() {
            let team = self.players[i].team;
            if team < 0 { continue; }
            let real = self.leagues[self.clubs[team as usize].league].detail == 0;
            let p = &self.players[i];
            let ae = p.age as f32 - if p.pos == crate::id("GK") { 3.0 } else { 0.0 }; // gk_shift 3
            let young = ae <= 26.0;
            let d0 = if young { let a = ae.max(15.0); (p.pot - p.ovr).max(-2.0) * interp(&GROWTH, a) / rem(a) } else { interp(&DECLINE, ae.min(36.0)) } * crate::sofifa::kurve(p.profil.wachstum, ae);
            // the own club and academy develop per ability (entwicklung.rs); only players without the 25 abilities (the demo world) keep this step
            if team as usize == own && self.players[i].attrs.len() == 25 { continue; }
            let tr = 1.0;
            let share = if real { (p.apps as f32 * 90.0 / 2700.0).min(1.0) } else { proxy[i] };
            let (mut tm, pf, sd) = if young {
                (if d0 > 0.0 { lerp(0.85, 1.2) * tr } else { 1.0 }, 0.9 + 0.33 * share, 0.9)
            } else {
                (lerp(1.1, 0.85) / tr, 1.1 - 0.2 * share, 0.6)
            };
            if young && p.ovr > 88.0 && d0 > 0.0 { tm *= (1.0 - (p.ovr - 88.0) / 20.0).max(0.3); } // top_soft
            let mut perf = 0.0;
            if real && p.rn >= 8 { perf = ((p.avg_rating() - 6.5) * 0.8).clamp(-0.5, 1.0); if perf > 0.0 && ae > 28.0 { perf = 0.0; } }
            let noise = self.rng.gauss(sd) / 52f32.sqrt();
            self.grow(i, (d0 * tm * pf + perf) / 52.0 + noise);
            let p = &mut self.players[i];
            if !young { p.pot = p.ovr; } else if perf != 0.0 { p.pot = (p.pot + 0.5 * perf / 52.0).max(p.ovr).min(99.0); }
        }
    }
}

struct Zuschlag { pfeil: [f32; 5], sv: f32, tg: f32, hoechstens: f32, sv_neutral: f32 }
fn zahlen() -> &'static Zuschlag {
    static Z: std::sync::OnceLock<Zuschlag> = std::sync::OnceLock::new();
    Z.get_or_init(|| {
        let f = |k: &str| crate::regeln::f("entwicklung_pesm", k);
        Zuschlag { pfeil: [0, 1, 2, 3, 4].map(|i| f(&format!("zuschlag.pfeil.{i}"))), sv: f("zuschlag.selbstvertrauen"), tg: f("zuschlag.teamgeist"),
            hoechstens: f("zuschlag.seele_hoechstens"), sv_neutral: crate::regeln::f("entwicklung", "selbstvertrauen.neutral") }
    })
}
/// Form bonus from the form arrow (entwicklung_pesm.toml zuschlag.pfeil): works in the match and in the live-written database,
/// never on the true strength.
/// The rating bonus written into the game: the form arrow, the swing from recent events (a brace, three conceded …)
/// and the state of mind and body (morale on the floor −3, tired −1, flying +1). Limits in entwicklung_pesm.toml [dynamik].
pub fn form_zuschlag(p: &Player) -> f32 {
    let d = |k: &str| crate::regeln::f("entwicklung_pesm", &format!("dynamik.{k}"));
    let zustand = if p.morale < d("moral_tief_unter") { d("moral_tief") } else if p.morale < d("moral_niedrig_unter") { d("moral_niedrig") } else if p.morale > d("moral_hoch_ueber") { d("moral_hoch") } else { 0.0 }
        + if p.fit < d("muede_unter") { d(crate::id("muede")) } else { 0.0 };
    (zahlen().pfeil[crate::mensch::pfeil(p.form).min(4) as usize] + p.ps.dyn_z.round() + zustand).clamp(-d(crate::id("grenze")), d(crate::id("grenze")))
}
/// Why the bonus is what it is, in words (newest first): the recent events and the state.
pub fn zuschlag_gruende(p: &Player) -> Vec<String> {
    let d = |k: &str| crate::regeln::f("entwicklung_pesm", &format!("dynamik.{k}"));
    let mut v: Vec<String> = p.ps.dyn_gruende.iter().rev().map(|(_, k, w)| format!("{} {:+.0}", crate::inhalt::t(&format!("ui.dyn.{k}")), w).replace('-', "−")).collect();
    if p.morale < d("moral_tief_unter") { v.push(format!("{} {:+.0}", crate::inhalt::t("ui.dyn.moral_tief"), d("moral_tief")).replace('-', "−")); }
    else if p.morale < d("moral_niedrig_unter") { v.push(format!("{} {:+.0}", crate::inhalt::t("ui.dyn.moral_niedrig"), d("moral_niedrig")).replace('-', "−")); }
    else if p.morale > d("moral_hoch_ueber") { v.push(format!("{} {:+.0}", crate::inhalt::t("ui.dyn.moral_hoch"), d("moral_hoch"))); }
    if p.fit < d("muede_unter") { v.push(format!("{} {:+.0}", crate::inhalt::t("ui.dyn.muede"), d(crate::id("muede"))).replace('-', "−")); }
    v
}
/// The abilities to write for a player: the true ones moved so the overall changes by exactly the form bonus (pesdb::shift).
pub fn attrs_mit_zuschlag(p: &Player) -> Vec<f32> {
    let mut a = p.attrs.clone();
    let z = form_zuschlag(p);
    if z != 0.0 && a.len() == 25 { crate::pesdb::shift(&mut a, crate::pesdb::pos_index(&p.pos), z); }
    a
}

pub fn injury_names(days: u16) -> Vec<&'static str> {
    match days {
        0..=3 => vec![crate::inhalt::t("verl.art.prellung"), crate::inhalt::t("r.ui.demo.muskelverhaertung")],
        4..=7 => vec![crate::inhalt::t("verl.art.zerrung"), crate::inhalt::t("ui.world.oberschenkelprellung")],
        8..=14 => vec![crate::inhalt::t("verl.art.muskelfaserriss"), crate::inhalt::t("ui.world.sprunggelenksverletzung")],
        15..=28 => vec![crate::inhalt::t("ui.world.baenderdehnung"), crate::inhalt::t("ui.world.muskelbuendelriss"), crate::inhalt::t("ui.world.syndesmoseverletzung")],
        29..=56 => vec![crate::inhalt::t("ui.world.baenderriss"), crate::inhalt::t("ui.world.mittelfussbruch"), crate::inhalt::t("ui.world.meniskusverletzung")],
        _ => vec![crate::inhalt::t("ui.world.kreuzbandriss"), crate::inhalt::t("ui.world.schienbeinbruch"), crate::inhalt::t("ui.world.aussenbandriss_mit_operation")],
    }
}

pub fn retire_chance(age: u8) -> f32 {
    RETIRE.iter().rev().find(|(a, _)| age >= *a).map(|x| x.1).unwrap_or(0.0)
}

/// The built-in test world; one shared copy of assets/world.json instead of one per test module.
#[cfg(test)]
pub fn testwelt(seed: u64) -> World { World::load(crate::WORLD_JSON, seed) }

#[cfg(test)]
mod t {
    use super::World;
    /// Fabian 24.11.: a keeper who lets in three drops, a brace lifts, morale on the floor costs three, all with reasons.
    #[test]
    fn dynamik_mit_gruenden() {
        let mut w = testwelt(3);
        let (mut gk, mut doppel) = (false, false);
        for k in 0..300 {
            let (h, a) = (k % 10, 10 + k % 7);
            let r = w.play(h, a, 0, None, None);
            w.apply_stats(&r);
            gk |= w.players.iter().any(|p| p.ps.dyn_gruende.iter().any(|g| g.1 == "drei_gegentore") && p.ps.dyn_z < 0.0); // roh
            doppel |= w.players.iter().any(|p| p.ps.dyn_gruende.iter().any(|g| g.1 == "doppelpack")); // roh
            if gk && doppel { break; }
        }
        assert!(gk && doppel, "keeper down after three, brace up");
        let mut p = w.players[0].clone();
        p.ps.dyn_z = 0.0; p.form = 0.0; p.fit = 100.0;
        p.morale = 20.0;
        let tief = form_zuschlag(&p);
        p.morale = 60.0;
        assert!(form_zuschlag(&p) - tief >= 2.9, "morale on the floor costs three");
        p.morale = 20.0;
        assert!(!zuschlag_gruende(&p).is_empty());
    }
    #[test]
    fn ranglisten_zaehlen_nur_die_liga() {
        let mut w = crate::world::testwelt(5);
        let (l, fi) = (0, 0);
        let (h, a) = (w.leagues[l].fixtures[fi].h, w.leagues[l].fixtures[fi].a);
        // goals from elsewhere (youth, second team, cups) sit in the season totals only
        let fremd = w.clubs[h].squad[0];
        w.players[fremd].goals += 5;
        let res = w.play(h, a, 0, None, None);
        w.apply_result(l, fi, &res);
        let tore: u32 = w.players.iter().map(|p| p.in_liga(l).goals as u32).sum();
        let eigen = res.events.iter().filter(|e| e.kind == 0).count() as u32;
        assert_eq!(tore, eigen, "only goals of this league's matches count");
        assert!(w.players[fremd].in_liga(l).goals < 5);
        assert!(w.players.iter().all(|p| p.profil.ligen.iter().all(|x| x.l as usize == l)));
    }
    #[test]
    fn eingriff_wirkt_auf_den_rest() {
        // an attacking mentality from minute 30 brings more goals for both sides than a defensive one; the first half stays
        let mut w = crate::world::testwelt(5);
        let (mut off, mut def) = (0u32, 0u32);
        for k in 0..600 {
            let res = w.play(0, 1, 0, None, None);
            let xi: Vec<usize> = res.xi_h.iter().map(|x| x.1).collect();
            let bis30 = res.events.iter().filter(|e| e.min <= 30).count();
            let mut a = res.clone();
            w.rest_neu(&mut a, 30, 0, &xi, if k % 2 == 0 { 1.12 } else { 0.85 }, if k % 2 == 0 { 1.10 } else { 0.88 });
            assert_eq!(a.events.iter().filter(|e| e.min <= 30).count(), bis30);
            assert_eq!(a.hg as usize, a.events.iter().filter(|e| e.side == 0 && e.kind <= 1).count());
            let tore = (a.hg + a.ag) as u32 - res.events.iter().filter(|e| e.min <= 30 && e.kind <= 1).count() as u32;
            if k % 2 == 0 { off += tore; } else { def += tore; }
        }
        assert!(off > def, "offensiv {off} defensiv {def}");
    }
    #[test]
    fn rollen_aus_lagen_passen_zu_den_formationen() {
        // the board's zones give the formations' own roles back, and a moved player changes his role
        for f in 0..super::FORMATIONS.len() { assert_eq!(super::rollen_aus_lagen(&super::formation_spots(f)), super::formation_slots(f), "Formation {}", super::FORMATIONS[f]); }
        let mut s = super::formation_spots(1).to_vec();
        s[2] = (0.6, 0.42);
        assert_eq!(super::rollen_aus_lagen(&s)[2], "DMF", "Innenverteidiger ins Mittelfeld gezogen");
        s[5] = (0.15, 0.55);
        let r = super::rollen_aus_lagen(&s);
        assert_eq!((r[5], r[8]), ("LMF", "CMF"), "zweiter LM wird zentral");
    }
    #[test]
    fn kuerzel_je_land_eindeutig() {
        let mut welten = vec![crate::world::testwelt(5)];
        for n in ["_FL26-27-Def", "_FL26Megapack", "_FL26MyWorld"] {
            let d = crate::fl27::sider().join("livecpk").join(n).join("common/etc/pesdb");
            if crate::fl27::echte_welt(&d) { if let Ok(w) = World::from_pesdb(&d, 7) { welten.push(w); } }
        }
        assert!(welten.len() >= 2, "mindestens eine echte Welt");
        for w in &welten {
            let mut seen: std::collections::HashMap<(String, String), &str> = Default::default();
            for c in &w.clubs {
                let l = &w.leagues[c.league];
                let land = if l.land.is_empty() { l.key.clone() } else { l.land.clone() };
                assert!(!c.short.is_empty(), "{} ohne Kürzel", c.name);
                if let Some(o) = seen.insert((land, c.short.clone()), &c.name) { panic!("{} und {} beide {}", o, c.name, c.short); }
            }
            if let (Some(b), Some(l)) = (w.clubs.iter().find(|c| c.name.contains("Bayern")), w.clubs.iter().find(|c| c.name.contains("Leverkusen"))) { assert_ne!(b.short, l.short); }
        }
    }
    /// Team spirit counts in the match: the same squad with team spirit 80 plays measurably better than with 30 (500 matches).
    #[test]
    fn teamgeist_wirkt_ueber_500_spiele() {
        let w = crate::world::testwelt(5);
        let l = &w.leagues[0];
        let (h, a) = (l.teams[0], l.teams[1]);
        let tordiff = |tg: f32| {
            let mut x = w.clone();
            x.teamgeist_eigen = Some((h, tg));
            (0..500).map(|_| { let r = x.play(h, a, 0, None, None); r.hg as i32 - r.ag as i32 }).sum::<i32>()
        };
        let (gut, schlecht) = (tordiff(80.0), tordiff(30.0));
        assert!(gut > schlecht + 20, "Teamgeist 80: {gut}, 30: {schlecht}");
    }
    /// The form bonus moves the written abilities so the overall changes by exactly the bonus, from the true strength:
    /// back at a neutral arrow the abilities are the true ones again, and writing twice never adds up.
    #[test]
    fn form_zuschlag_und_ruecknahme() {
        let w = crate::world::testwelt(5);
        let mut p = w.players[0].clone();
        p.pos = "CF".into();
        p.attrs = vec![65.0; 25];
        let pos = crate::pesdb::pos_index(&p.pos);
        let echt = crate::pesdb::ovr(&p.attrs, pos);
        p.form = 5.0;
        assert_eq!(super::form_zuschlag(&p), 3.0);
        let a1 = super::attrs_mit_zuschlag(&p);
        assert!((crate::pesdb::ovr(&a1, pos) - echt - 3.0).abs() < 0.01, "Gesamtwert = Stärke + Zuschlag");
        assert_eq!(p.attrs, vec![65.0; 25], "die echte Stärke bleibt");
        assert_eq!(super::attrs_mit_zuschlag(&p), a1, "kein Stapeln");
        p.form = -2.0;
        assert_eq!(super::form_zuschlag(&p), -1.0);
        assert!((crate::pesdb::ovr(&super::attrs_mit_zuschlag(&p), pos) - echt + 1.0).abs() < 0.01);
        p.form = 0.0;
        assert_eq!(super::attrs_mit_zuschlag(&p), p.attrs, "Rücknahme: neutral heißt die echten Werte");
    }
    #[test]
    fn geld_in_beiden_sprachen() {
        use crate::inhalt::Sprache::{De, En};
        assert_eq!(super::geld(De, 1_200_000.0), "1,2 Mio. €");
        assert_eq!(super::geld(De, 850_000.0), "850 Tsd. €");
        assert_eq!(super::geld(De, 500.0), "500 €");
        assert_eq!(super::geld(De, -2_350_000.0), "−2,4 Mio. €");
        assert_eq!(super::geld(De, 999_960.0), "1,0 Mio. €", "no „1000 Tsd.“");
        assert_eq!(super::geld(En, 1_200_000.0), "€1.2m");
        assert_eq!(super::geld(En, 850_000.0), "€850k");
        assert_eq!(super::geld(En, 500.0), "€500");
        assert_eq!(super::geld(En, -2_350_000.0), "−€2.4m");
        assert_eq!(super::geld(De, 0.0), "0 €");
        assert_eq!(super::geld(En, 0.0), "€0");
        assert_eq!(super::geld(De, 1_250_000_000.0), "1,3 Mrd. €");
        assert_eq!(super::geld(En, 1_250_000_000.0), "€1.3bn");
        assert_eq!(super::geld(De, -3_000_000_000.0), "−3,0 Mrd. €");
        assert_eq!(super::geld(En, 999_990_000.0), "€1.0bn", "no „€1000.0m“");
    }
    use super::*;
    #[test]
    fn calibration() {
        // equal teams: home win 41-49 %, draw 22-29 %, goals 2.5-2.95 (sim.json calibration)
        let (h, d, a) = World::win_probs(HOME_GOALS, AWAY_GOALS);
        assert!(h > 0.4 && h < 0.5, "{h}");
        assert!(d > 0.22 && d < 0.3, "{d}");
        assert!(a > 0.24 && a < 0.34, "{a}");
        let mut v = Player { nat: 0, id: 1, name: "x".into(), pos: "CF".into(), age: 26, ovr: 77.0, pot: 77.0, talent: 1.0, no: 9, team: 0, youth: 0, form: 0.0, fit: 100.0, gf: 0.0, morale: 50.0, inj: 0, inj_name: String::new(), susp: 0, wage: 0.0, contract_end: 3, value: 0.0, apps: 0, goals: 0, assists: 0, rsum: 0.0, rn: 0, yellow: 0, last: vec![], know: 0.0, listed: false, persona: 0, unhappy_days: 0, benched: 0, promise: 0, program: 0, joined: 0, talk_day: 0, ps: Default::default(), leihe: None, ent: Default::default(), st: Default::default(), attrs: vec![], pp: vec![], style: 0, skills: 0, db: false, profil: Default::default() };
        let m = market_value(&v, 0, 68.0);
        assert!(m > 15e6 && m < 35e6, "{m}"); // MARKTWERT-EICHUNG: OVR 77 at 26 ≈ 24 Mio.
        v.ovr = 92.0; v.age = 25;
        let m = market_value(&v, 0, 68.0);
        assert!(m > 100e6 && m < 250e6, "{m}");
        assert_eq!(World::match_days(34, true).len(), 34);
        assert_eq!(World::match_days(46, false).len(), 46);
    }
}

#[cfg(test)]
mod saison_realismus {
    use super::*;
    /// Fabian 25.11.: a swing confirmed for three weeks becomes the new standard (one point into the abilities, the
    /// swing smaller by as much, so the rating in the game stays the same at that moment); never above the potential.
    #[test]
    fn bestaetigte_form_bleibt() {
        let mut w = crate::world::testwelt(3);
        let p = w.clubs[0].squad[0];
        w.players[p].attrs = vec![70.0; 25];
        w.players[p].ovr = crate::pesdb::ovr(&w.players[p].attrs, crate::pesdb::pos_index(&w.players[p].pos));
        w.players[p].pot = w.players[p].ovr + 5.0;
        let vorher = w.players[p].ovr;
        for _ in 0..22 { w.players[p].ps.dyn_z = 3.0; w.recover_all(); }
        assert!((w.players[p].ovr - vorher - 1.0).abs() < 0.2, "{} → {}", vorher, w.players[p].ovr);
        w.players[p].pot = w.players[p].ovr;
        let jetzt = w.players[p].ovr;
        for _ in 0..30 { w.players[p].ps.dyn_z = 3.0; w.recover_all(); }
        assert!((w.players[p].ovr - jetzt).abs() < 0.01, "not above the potential");
    }
    /// Fabian 24.11.: last season's table and scorers stay readable after the season ends.
    #[test]
    fn vorsaison_archiviert() {
        let mut w = crate::world::testwelt(3);
        let l = 0;
        w.leagues[l].fixtures[0].hg = 2; w.leagues[l].fixtures[0].ag = 0;
        let vorher = w.table(l);
        w.saison_archivieren();
        let ar = w.archiv.iter().find(|x| x.liga == l && x.season == w.season).expect("archive");
        assert_eq!(ar.tabelle.len(), vorher.len());
        assert_eq!(ar.tabelle[0].pts, vorher[0].pts);
    }
    /// One season of the test world: goals and draws per league near its profile, a real spread of points, no striker
    /// with the lion's share of his team's goals, and table, results and scorers that agree.
    #[test]
    fn eine_saison_wie_echt() {
        let w = crate::world::testwelt(7);
        let mut c = crate::game::Career::new(w, 0, "Test");
        c.mensch.vertrag_bis = u32::MAX;
        while c.world.day < LEAGUE_END + 2 {
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            for i in 0..c.inbox.len() { if !c.inbox[i].done && matches!(c.inbox[i].action, crate::game::Action::Wahl { .. } | crate::game::Action::TransferOffer { .. }) { c.answer(i, false, 0); } }
            c.ultimatum = None; c.fired = false;
            c.end_day().unwrap();
        }
        let wl = &c.world;
        for (li, l) in wl.leagues.iter().enumerate() {
            let fx: Vec<&Fixture> = l.fixtures.iter().filter(|f| f.hg >= 0).collect();
            if fx.len() < 100 || l.detail > 1 || !l.gruppen.is_empty() { continue; }
            let tore = fx.iter().map(|f| (f.hg + f.ag) as f32).sum::<f32>() / fx.len() as f32;
            let remis = fx.iter().filter(|f| f.hg == f.ag).count() as f32 / fx.len() as f32;
            let heim = fx.iter().filter(|f| f.hg > f.ag).count() as f32 / fx.len() as f32;
            let (ph, pa) = { let p = crate::regeln::wert("ligaprofile", "land"); let land = l.land.to_lowercase(); let q = if l.stufe >= 2 { crate::regeln::wert("ligaprofile", "unterklasse") } else { p.as_object().and_then(|o| o.iter().find(|(n, _)| land.contains(n.as_str())).map(|x| x.1)).unwrap_or(crate::regeln::wert("ligaprofile", "standard")) }; (q["heim"].as_f64().unwrap() as f32, q["gast"].as_f64().unwrap() as f32) };
            eprintln!("{}: Tore {tore:.2} (Profil {:.2}), Remis {:.0} %, Heimsiege {:.0} %", l.name, ph + pa, remis * 100.0, heim * 100.0);
            assert!((tore - (ph + pa)).abs() < 0.5, "{}: {tore} goals", l.name);
            assert!((0.16..0.34).contains(&remis), "{}: draws {remis}", l.name);
            assert!(heim > 0.36 && heim < 0.56, "{}: home wins {heim}", l.name);
            let t = wl.table(li);
            let ppg = |r: &Row| r.pts as f32 / r.p.max(1) as f32;
            assert!(ppg(&t[0]) >= 1.65 && ppg(t.last().unwrap()) < 1.2, "{}: spread {} … {}", l.name, ppg(&t[0]), ppg(t.last().unwrap()));
            // the table agrees with the results
            let tg: i32 = t.iter().map(|r| r.gf).sum();
            assert_eq!(tg as i64 - l.basis.iter().map(|b| b.1[4] as i64).sum::<i64>(), fx.iter().map(|f| (f.hg + f.ag) as i64).sum::<i64>(), "{}: table goals", l.name);
            // the scorers: nobody above 50 % of his club's goals
            for &club in &l.teams {
                // player goals count every competition: compare with the squad's goals, not the league's
                let alle: u32 = wl.clubs[club].squad.iter().map(|&p| wl.players[p].goals as u32).sum();
                if alle < 25 { continue; }
                let best = wl.clubs[club].squad.iter().map(|&p| wl.players[p].goals).max().unwrap_or(0);
                assert!((best as f32) < 0.5 * alle as f32, "{}: top scorer {best} of {alle}", wl.clubs[club].name);
            }
        }
    }
}
