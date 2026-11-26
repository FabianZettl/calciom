//! Paket 3: the people side of the own club (Mechanik-Übergabe §1–6, §11–12). The core (core/) is what the
//! person experiences; the world keeps its simpler drift for all other clubs (world.rs). Numbers come from
//! assets/regeln (regeln.rs), texts from assets/texte/mensch.json. Every change of morale, climate or character
//! goes through one write function with a reason in football language (`grund`, guard test `jede_wirkung_hat_grund`).
use crate::game::*;
use crate::regeln::{f, t, wert};
use crate::world::*;
use serde::{Deserialize, Serialize};

/// SHA-256(parts) → [0, 1): fixed randomness with a purpose key (Grundsatz 2).
pub fn zufall(teile: &[&str]) -> f32 {
    use sha2::{Digest, Sha256};
    let h = Sha256::digest(teile.join("|").as_bytes());
    let mut b = [0u8; 8];
    b.copy_from_slice(&h.as_slice()[..8]);
    (u64::from_be_bytes(b) as f64 / 18446744073709551616.0) as f32
}

pub const WERTE: [&str; 10] = [crate::id("professionalitaet"), crate::id("ehrgeiz"), crate::id("ego"), crate::id("teamgeist"), crate::id("fuehrung"), crate::id("temperament"), crate::id("nervenstaerke"), crate::id("vertraeglichkeit"), crate::id("loyalitaet"), crate::id("status")];
pub const PROF: usize = 0;
pub const EHRGEIZ: usize = 1;
pub const EGO: usize = 2;
pub const TEAMGEIST: usize = 3;
pub const FUEHRUNG: usize = 4;
pub const TEMPERAMENT: usize = 5;
pub const NERVEN: usize = 6;
pub const VERTRAEGLICH: usize = 7;
pub const LOYAL: usize = 8;
pub const STATUS: usize = 9;
pub const ROLLEN: [&str; 7] = [crate::id("kapitaen"), crate::id("vizekapitaen"), crate::id("fuehrungsspieler"), crate::id("stammspieler"), crate::id("rotationsspieler"), crate::id("perspektivspieler"), crate::id("randfigur")];

/// What a talk did (the card „So hat es gewirkt“): the player's morale, trust, relationship, the team spirit and the teammates
/// it reached, each with its reason.
#[derive(Clone, Debug)]
pub struct Wirkung { pub pid: usize, pub moral: f32, pub vertrauen: f32, pub beziehung: f32, pub teamgeist: f32, pub grund: String, pub andere: Vec<(usize, f32, String)>, pub seit: f64 }

/// Per player: character, morale drivers, self-confidence, transfer wish, relation to the coach (§3, §4).
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Psyche {
    pub an: bool,
    /// Rating swing from what happened lately (Fabian 24.11.: drastic, always with a reason): points and the reasons
    /// (abs day, text key, points), newest last.
    pub dyn_z: f32,
    pub dyn_gruende: Vec<(u32, String, f32)>,
    /// Days in a row the swing stayed clearly up (positive) or down (negative), and the base change it made this season
    /// (Fabian 25.11.: a form confirmed for long becomes the new standard).
    pub dyn_tage: i16,
    pub dyn_dauer: (u32, f32),
    pub ch: Vec<f32>,
    pub sh: Vec<f32>,
    pub shm: Vec<f32>,
    pub shm_monat: u32,
    pub sv: f32,
    pub wb: f32,
    pub wunsch: bool,
    pub unzufrieden: bool,
    /// Relation to the coach −100…100 (centre 0).
    pub bez: f32,
    /// Share of minutes in the last club matches (0…1, newest last).
    pub einsatz: Vec<f32>,
    /// Conversation effects on morale: (abs day from which it counts, points).
    pub gespr: Vec<(u32, f32)>,
    pub treiber: Vec<(String, f32)>,
    pub rolle: u8,
    /// Shift of his stance on an offer by talking to him (haltung.rs: ±10 per talk, at most ±30).
    pub schub: f32,
    /// The wage he asked in return for a condition he has met (vertragsanfrage.rs), on top of his demand until he signs.
    pub anfrage_plus: f32,
    pub hier: f32,
    /// Offers of other clubs (abs day), they count 60 days.
    pub angebote: Vec<u32>,
    pub mentor: Option<u32>,
    /// On the own loan list: clubs make loan offers (transfer.rs leihangebote_tag).
    pub leihliste: bool,
    /// In the extended squad instead of the first squad (ideas of other FL tools, 23.11.): no playing-time demands, a
    /// little less standing in the dressing room.
    #[serde(default)]
    pub erweitert: bool,
    /// Youth player training with the first team (Akademie-Konzept 5.4: Trainingsgast).
    pub gast: bool,
    /// Topic → abs day until the same topic may come again.
    pub sperre: Vec<(String, u32)>,
    pub vorfall: u32,
    /// Protest of a player who wants to leave: 0 none, 1 unrest, 2 training strike, 3 match strike.
    pub streik: u8,
    /// Load (neutral 30), phase 0 none 1 Aufschwung 2 Formtief 3 Krise until abs day, locks, retraining (Paket 3o).
    pub last: f32,
    pub phase: u8,
    pub phase_bis: u32,
    pub phase_sperre: u32,
    pub sprung_sperre: u32,
    pub umschulung: Option<(String, u32)>,
    /// The day the retraining was first seen running (for the halfway step).
    #[serde(default)]
    pub umschulung_start: Option<u32>,
    /// abs day a promise to him was last broken.
    pub versprechen_gebrochen: u32,
    /// Youth (Paket 3p): promised U23 games, weeks without games, promotion already proposed.
    pub u23_stamm: bool,
    pub ohne_spiel: u32,
    pub befoerdert_gefragt: bool,
}
impl Psyche {
    /// Growth factor of the current phase (entwicklung3): Aufschwung × 1.3, Formtief × 0.4, Krise × 0.2.
    /// Form shift of the phase: Aufschwung +2, Formtief −2.5, Krise −4.
    /// Target form of the development phase (entwicklung3.toml [phase] form): the form arrow and its bonus show the phase.
    pub fn phasen_form(&self) -> f32 { if self.phase == 0 { 0.0 } else { f("entwicklung3", &format!("phase.form.{}", crate::entwicklung::PHASEN[self.phase.min(3) as usize])) } }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Grund { pub abs: u32, pub bezug: String, pub was: String, pub wert: f32, pub grund: String }
#[derive(Serialize, Deserialize, Clone)]
pub struct Versprechen { pub pid: usize, pub art: String, pub seit: u32, pub frist: u32, pub gewicht: f32, pub stand: u8, pub basis: f64, pub entschieden: u32 }
#[derive(Serialize, Deserialize, Clone)]
pub struct Bez { pub a: u32, pub b: u32, pub w: f32 }
#[derive(Serialize, Deserialize, Clone)]
pub struct Konflikt { pub a: u32, pub b: u32, pub seit: u32, pub schwere: u8 }

pub const LIZENZEN: [&str; 4] = [crate::id("c"), crate::id("b"), crate::id("a"), crate::id("pro")];

/// State of the people side of a career (kept in the save).
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Mensch {
    pub vorstand: f32,
    pub teamgeist: f32,
    pub gruende: Vec<Grund>,
    pub versprechen: Vec<Versprechen>,
    pub beziehungen: Vec<Bez>,
    pub konflikte: Vec<Konflikt>,
    /// (abs, player id, kind, punishment) of discipline cases.
    pub strafen: Vec<(u32, u32, String, String)>,
    pub saisonziel_gebucht: bool,
    /// abs day of the last new player concerns (throttle: 1 per day, 3 per week).
    pub anliegen: Vec<u32>,
    pub titel: u32,
    pub vertrag_bis: u32,
    /// club → abs day until it may approach again (laufbahn motor erneut_sperre_tage).
    pub job_sperre: Vec<(usize, u32)>,
    /// Coaching licence reached so far (c, b, a, pro; "" before the first check): a new level is announced once.
    pub lizenz: String,
    /// player -> abs day until talks may start again after a breakdown (runden.py abkuehl).
    #[serde(default)]
    pub neg_sperre: Vec<(usize, u32)>,
    /// club -> relation -3..3 from negotiations.
    #[serde(default)]
    pub vereine: Vec<(usize, f32)>,
    /// Assistant coach: strength 0…100, relation entries (abs day, value), recommendations on hold (key, until).
    pub co_staerke: f32,
    pub co_eintraege: Vec<(u32, f32)>,
    pub co_sperre: Vec<(String, u32)>,
    /// Stories (Paket 3q): goal accepted, rule locks (rule|player id → until), due follow-ups, concerns (player id, day).
    pub ziel_angenommen: bool,
    pub story_sperre: Vec<(String, u32)>,
    pub story_folgen: Vec<(String, Option<usize>, u32)>,
    pub story_anliegen: Vec<(u32, u32)>,
    /// Stories told this career: rule id, abs day, player id (0 = none) – the no-repeat memory (Fabian 23.11.).
    #[serde(default)]
    pub story_erzaehlt: Vec<(String, u32, u32)>,
}
impl Default for Mensch {
    fn default() -> Self {
        Mensch { story_erzaehlt: vec![], vorstand: 50.0, teamgeist: 50.0, gruende: vec![], versprechen: vec![], beziehungen: vec![], konflikte: vec![], strafen: vec![], saisonziel_gebucht: false, anliegen: vec![], titel: 0, vertrag_bis: 0, job_sperre: vec![], neg_sperre: vec![], vereine: vec![], co_staerke: 50.0, co_eintraege: vec![], co_sperre: vec![], ziel_angenommen: false, story_sperre: vec![], story_folgen: vec![], story_anliegen: vec![], lizenz: String::new() }
    }
}

pub fn arr(datei: &str, pfad: &str) -> &'static Vec<serde_json::Value> { wert(datei, pfad).as_array().unwrap() }

/// Base character value of a player (persoenlichkeit.json, model 4). The four personality axes of the game are
/// not in Player.bin, so every s is 0 (neutral): what remains are age, the leadership bonus, the fixed random
/// part (seed 2026) and the status percentile.
pub fn charakter(id: u32, age: u8, perzentil: f32) -> Vec<f32> {
    let n = f("persoenlichkeit", "neutral");
    let grenze = f("persoenlichkeit", "zufall.grenze");
    let seed = format!("{}", f("persoenlichkeit", "zufall.seed") as i64);
    WERTE.iter().map(|w| {
        let mut v = n;
        if *w == crate::id("loyalitaet") {
            for z in arr("persoenlichkeit", "alter.loyalitaet") {
                let wv = z["wert"].as_f64().unwrap() as f32;
                if let Some(ab) = z["ab"].as_f64() { if age as f64 >= ab { v += wv; } }
                if let Some(bis) = z["bis"].as_f64() { if age as f64 <= bis { v += wv; } }
            }
        }
        if *w == crate::id("fuehrung") && age as f32 >= f("persoenlichkeit", "bonus.fuehrung.alter_ab") { v += f("persoenlichkeit", "bonus.fuehrung.alter_wert"); }
        if *w == crate::id("status") { v = n + f("persoenlichkeit", "abgeleitet.status.perzentil.faktor") * (perzentil - f("persoenlichkeit", "abgeleitet.status.perzentil.bezug")); }
        let g = crate::regeln::f_oder("persoenlichkeit", &format!("zufall.je_wert.{w}"), grenze);
        v += (zufall(&[&seed, &id.to_string(), w]) * 2.0 - 1.0) * g;
        v.clamp(0.0, 100.0)
    }).collect()
}

/// Effective character value (base + shifts by decisions).
pub fn cw(p: &Player, i: usize) -> f32 {
    if !p.ps.an { return 50.0; }
    (p.ps.ch[i] + p.ps.sh.get(i).copied().unwrap_or(0.0)).clamp(0.0, 100.0)
}

/// Player types, first match first (persoenlichkeit.json typen).
pub fn typen(p: &Player) -> Vec<&'static str> {
    arr("persoenlichkeit", "typen").iter().filter(|ty| {
        ty["bedingungen"].as_array().unwrap().iter().all(|b| {
            let w = b["wert"].as_str().unwrap();
            let v = if w == crate::id("alter") { p.age as f32 } else { cw(p, WERTE.iter().position(|x| *x == w).unwrap()) };
            b["min"].as_f64().map_or(true, |m| v as f64 >= m) && b["max"].as_f64().map_or(true, |m| v as f64 <= m)
        })
    }).map(|ty| ty["id"].as_str().unwrap()).collect()
}
pub fn typ_name(id: &str) -> &'static str { t(&format!("typen.{id}")) }
pub fn wert_name(i: usize) -> &'static str { crate::regeln::text_wert("werte").as_array().unwrap()[i].as_str().unwrap() }
pub fn rolle_name(r: u8) -> &'static str { crate::regeln::text_wert("rollen").as_array().unwrap()[r.min(6) as usize].as_str().unwrap() }
pub fn treiber_name(k: &str) -> &'static str { crate::regeln::text_wert("treiber")[k].as_str().unwrap_or(crate::inhalt::t("es.art.standard")) }

/// Form arrow 0…4 of the overlay (entwicklung.toml vorschlag.pfeil_grenzen).
pub fn pfeil(form: f32) -> u8 { arr("entwicklung", "vorschlag.pfeil_grenzen").iter().filter(|g| form as f64 >= g.as_f64().unwrap()).count() as u8 }

/// Tone change of a conversation (gespraeche/werte.py ton_aenderung). Character values 0…100.
pub fn ton_aenderung(ton: &str, p: &[f32], stimmung: f32, vertrauen: f32) -> f32 {
    let q = |i: usize| p[i] / 100.0;
    let em = 0.5 * (1.0 - q(NERVEN)) + 0.3 * q(EGO) + 0.2 * q(TEMPERAMENT);
    let base = f("gespraeche", &format!("ton.{ton}"));
    let v = match ton {
        "warm" => base * (0.8 + 0.4 * q(VERTRAEGLICH)) * if q(NERVEN) < 0.45 { 1.25 } else { 1.0 },
        "sachlich" => base * (0.7 + 0.6 * q(PROF)),
        "lob" => base * (0.8 + 0.4 * q(EGO)) * if stimmung > 75.0 { 0.5 } else { 1.0 },
        "bestimmt" => if q(EHRGEIZ) >= 0.6 && q(PROF) >= 0.5 { 4.0 } else { 3.0 - 8.0 * em },
        "hart" => base * (0.6 + 0.9 * em) * if q(TEMPERAMENT) >= 0.6 { 1.3 } else { 1.0 },
        _ => base * (1.0 + (1.0 - vertrauen / 100.0)),
    };
    if v > 0.0 { v * (0.5 + 0.5 * vertrauen.min(50.0) / 50.0 + (vertrauen - 50.0).max(0.0) / 50.0 * 0.2) } else { v }
}

/// Mean of the best 16 ratings of a club (klima.json saisonziel.spieler_je_verein).
pub fn kaderstaerke(w: &World, c: usize) -> f32 {
    let mut v: Vec<f32> = w.clubs[c].squad.iter().map(|&p| w.players[p].ovr).collect();
    v.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let n = (f("klima", "saisonziel.spieler_je_verein") as usize).min(v.len()).max(1);
    v.iter().take(n).sum::<f32>() / n as f32
}
/// The board's season goal (klima.json saisonziel): rank of squad strength, after 6 games mixed 30 % with the
/// table position; the strongest squad gets place 2, place 1 only with a lead of 4 rating points.
pub fn saisonziel(w: &World, c: usize) -> usize {
    let l = w.clubs[c].league;
    let mut st: Vec<(usize, f32)> = w.leagues[l].teams.iter().map(|&t| (t, kaderstaerke(w, t))).collect();
    st.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let rang = st.iter().position(|x| x.0 == c).unwrap_or(0) + 1;
    let mut ziel = rang as f32;
    if rang == 1 { ziel = if st.len() > 1 && st[0].1 - st[1].1 >= f("klima", "saisonziel.spitze_abstand") { 1.0 } else { 2.0 }; }
    let table = w.table(l);
    if let Some(i) = table.iter().position(|r| r.club == c) {
        if table[i].p as f32 >= f("klima", "saisonziel.tabelle_ab_spielen") {
            let g = f("klima", "saisonziel.gewicht_tabelle");
            ziel = ziel * (1.0 - g) + (i + 1) as f32 * g;
        }
    }
    (ziel.round() as usize).max(1)
}
/// The words for a season goal.
pub fn ziel_wort(pos: usize, n: usize) -> &'static str {
    let abstieg = f("klima", "liga.abstiegsplaetze") as usize;
    if pos == 1 { crate::inhalt::t("ar.titel.art.meisterschaft") } else if pos <= (n / 5).max(2) { crate::inhalt::t("mk.tv.familie.europa") } else if pos + abstieg + 2 <= n { crate::inhalt::t("gb2.vgl.mittel") } else { crate::inhalt::t("r.be.mock.verein.erwartung.klasse") }
}

impl Career {
    // ------------------------------------------------------------ the one write path (Grundsatz 1)
    pub fn grund(&mut self, bezug: &str, was: &str, wert: f32, grund: &str) {
        let abs = self.abs();
        self.mensch.gruende.push(Grund { abs, bezug: bezug.into(), was: was.into(), wert, grund: grund.into() });
        if self.mensch.gruende.len() > 4000 { self.mensch.gruende.drain(..1000); }
    }
    /// Change fans, media image and board confidence (0…100) with a reason.
    pub fn klima(&mut self, fans: f32, medien: f32, vorstand: f32, grund: &str) {
        if fans != 0.0 { self.fans = (self.fans + fans).clamp(0.0, 100.0); self.grund("Fans", crate::inhalt::t("story.groesse.fanstimmung"), fans, grund); } // roh
        if medien != 0.0 { self.media = (self.media + medien).clamp(0.0, 100.0); self.grund("Medien", crate::inhalt::t("story.groesse.medienbild"), medien, grund); } // roh
        if vorstand != 0.0 {
            self.mensch.vorstand = (self.mensch.vorstand + vorstand).clamp(0.0, 100.0);
            self.grund("Präsidium", crate::inhalt::t("ui.mensch.vorstandsvertrauen"), vorstand, grund); // roh
            self.ultimatum_pruefen();
        }
    }
    /// A conversation-like effect on a player's morale (driver "gespraech", from `ab` days on).
    pub fn moral_wirkung(&mut self, p: usize, punkte: f32, ab: u32, grund: &str) {
        let abs = self.abs();
        self.world.players[p].ps.gespr.push((abs + ab, punkte));
        let n = self.world.players[p].name.clone();
        self.grund(&n, crate::inhalt::t("ui.mensch.moral_gespraech"), punkte, grund);
    }
    pub fn beziehung(&mut self, p: usize, d: f32, grund: &str) {
        let pl = &mut self.world.players[p];
        pl.ps.bez = (pl.ps.bez + d).clamp(-100.0, 100.0);
        let n = pl.name.clone();
        self.grund(&n, crate::inhalt::t("ui.mensch.beziehung_zum_trainer"), d, grund);
    }
    /// Shift a character value by a decision: at most ±5 per month and ±40 in total (persoenlichkeit.json verschiebung).
    pub fn verschieben(&mut self, p: usize, i: usize, d: f32, grund: &str) {
        let monat = self.abs() / 30;
        let (jm, ges) = (f("persoenlichkeit", "verschiebung.je_monat"), f("persoenlichkeit", "verschiebung.gesamt"));
        let pl = &mut self.world.players[p];
        if !pl.ps.an { return; }
        if pl.ps.shm_monat != monat { pl.ps.shm = vec![0.0; 10]; pl.ps.shm_monat = monat; }
        let m = (pl.ps.shm[i] + d).clamp(-jm, jm) - pl.ps.shm[i];
        let g = (pl.ps.sh[i] + m).clamp(-ges, ges) - pl.ps.sh[i];
        pl.ps.shm[i] += g;
        pl.ps.sh[i] += g;
        let n = pl.name.clone();
        if g != 0.0 { self.grund(&n, wert_name(i), g, grund); }
    }

    // ------------------------------------------------------------ setup and hierarchy (§4, §5)
    pub fn psyche_an(&mut self) {
        let sq = self.squad();
        if sq.iter().all(|&p| self.world.players[p].ps.an) { return; }
        let mut alle: Vec<f32> = self.world.players.iter().filter(|p| p.team >= 0).map(|p| p.ovr).collect();
        alle.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = alle.len().max(1) as f32;
        for p in sq {
            let pl = &mut self.world.players[p];
            if pl.ps.an { continue; }
            let perz = alle.partition_point(|&x| x < pl.ovr) as f32 / n * 100.0;
            pl.ps.ch = charakter(pl.id, pl.age, perz);
            // SoFIFA emotion: temperament and nerves lean to it (very calm … explosive)
            if (1..=5).contains(&pl.profil.emotion) && pl.profil.emotion != 3 {
                let k = pl.profil.emotion as usize - 1;
                pl.ps.ch[TEMPERAMENT] = 0.4 * pl.ps.ch[TEMPERAMENT] + 0.6 * [10.0, 28.0, 50.0, 72.0, 90.0][k];
                pl.ps.ch[NERVEN] = (pl.ps.ch[NERVEN] + [10.0, 5.0, 0.0, -5.0, -10.0][k]).clamp(0.0, 100.0);
            }
            pl.ps.sh = vec![0.0; 10];
            pl.ps.shm = vec![0.0; 10];
            pl.ps.sv = f("entwicklung", "selbstvertrauen.neutral");
            pl.ps.wb = f("entwicklung", "wechsel.neutral");
            pl.ps.last = f("entwicklung", "belastung.neutral");
            pl.ps.an = true;
        }
        self.hierarchie();
    }
    /// Hierarchy value and roles in the own squad (hierarchie.json). Influence (+341) is not in Player.bin,
    /// so its weight drops out and the others are normed to 1.
    pub fn hierarchie(&mut self) {
        let sq = self.squad();
        if sq.is_empty() { return; }
        let h = crate::id("hierarchie");
        let (lo, hi) = sq.iter().fold((f32::MAX, f32::MIN), |a, &p| (a.0.min(self.world.players[p].ovr), a.1.max(self.world.players[p].ovr)));
        let spielzeit = |c: &Career, p: usize| -> f32 {
            let e = &c.world.players[p].ps.einsatz;
            if e.is_empty() { if c.xi.contains(&p) { 100.0 } else { 40.0 } } else { e.iter().sum::<f32>() / e.len() as f32 * 100.0 }
        };
        let gw = [crate::id("fuehrung"), crate::id("status"), crate::id("loyalitaet"), crate::id("vereinszugehoerigkeit"), crate::id("spielzeit"), crate::id("erfahrung")].map(|k| f(h, &format!("gewichte.{k}")));
        let sum: f32 = gw.iter().sum();
        let (av, ab) = (f(h, "erfahrung.alter_von"), f(h, "erfahrung.alter_bis"));
        let mut werte: Vec<(usize, f32, f32, f32)> = sq.iter().map(|&p| {
            let pl = &self.world.players[p];
            let status = if hi > lo { (pl.ovr - lo) / (hi - lo) * 100.0 } else { 50.0 };
            let zug = if pl.joined == 0 { f(h, "vereinszugehoerigkeit.bestand") } else { f(h, "vereinszugehoerigkeit.neuzugang") };
            let erf = ((pl.age as f32 - av) / (ab - av) * 100.0).clamp(0.0, 100.0);
            let sz = spielzeit(self, p);
            let v = (gw[0] * cw(pl, FUEHRUNG) + gw[1] * status + gw[2] * cw(pl, LOYAL) + gw[3] * zug + gw[4] * sz + gw[5] * erf) / sum;
            (p, v, cw(pl, FUEHRUNG), sz)
        }).collect();
        werte.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let kf = f(h, "rollen.kapitaen_fuehrung_ab");
        let mut rolle = vec![255u8; werte.len()];
        for r in 0..2u8 {
            let k = (0..werte.len()).find(|&i| rolle[i] == 255 && werte[i].2 >= kf)
                .or_else(|| (0..werte.len()).filter(|&i| rolle[i] == 255).max_by(|&a, &b| werte[a].2.partial_cmp(&werte[b].2).unwrap()));
            if let Some(k) = k { rolle[k] = r; }
        }
        let (fa, fu, ff) = (f(h, "rollen.fuehrungsspieler_anzahl") as usize, f(h, "rollen.fuehrungsspieler_unter_den_ersten") as usize, f(h, "rollen.fuehrungsspieler_fuehrung_ab"));
        let mut nf = 0;
        for i in 0..werte.len().min(fu) { if rolle[i] == 255 && werte[i].2 >= ff && nf < fa { rolle[i] = 2; nf += 1; } }
        let (sa, ra, pa) = (f(h, "rollen.stammspieler_ab"), f(h, "rollen.rotation_ab"), f(h, "rollen.perspektive_alter_bis"));
        for (i, x) in werte.iter().enumerate() {
            let pl = &mut self.world.players[x.0];
            if rolle[i] == 255 { rolle[i] = if x.3 >= sa { 3 } else if x.3 >= ra { 4 } else if pl.age as f32 <= pa { 5 } else { 6 }; }
            pl.ps.rolle = rolle[i];
            pl.ps.hier = x.1;
        }
        // a captain chosen by the coach keeps the armband without a threshold (hierarchie.json rollen)
        if let Some(k) = self.std.kapitaen.and_then(|id| self.pid(id)) {
            for &p in &sq { if self.world.players[p].ps.rolle == 0 && p != k { self.world.players[p].ps.rolle = 1; } }
            self.world.players[k].ps.rolle = 0;
        }
        self.mentoren();
    }
    pub fn kapitaen(&self) -> Option<usize> { self.squad().into_iter().find(|&p| self.world.players[p].ps.an && self.world.players[p].ps.rolle == 0) }
    /// Mentor pairs come only from the co-trainer's proposals (jugend.rs mentor_vorschlaege); here a pair ends when the
    /// mentor has left the club.
    fn mentoren(&mut self) {
        for p in self.squad() {
            if self.world.players[p].ps.mentor.map_or(false, |m| self.pid(m).map_or(true, |q| self.world.players[q].team != self.own as i32)) { self.world.players[p].ps.mentor = None; }
        }
    }

    // ------------------------------------------------------------ relations in the dressing room (§6)
    fn bez_idx(&self, a: u32, b: u32) -> Option<usize> { let (a, b) = (a.min(b), a.max(b)); self.mensch.beziehungen.iter().position(|x| x.a == a && x.b == b) }
    pub fn bez_wert(&self, a: u32, b: u32) -> f32 { self.bez_idx(a, b).map(|i| self.mensch.beziehungen[i].w).unwrap_or(0.0) }
    fn bez_start(&self, p: usize, q: usize) -> f32 {
        let k = crate::id("entwicklung");
        let (x, y) = (&self.world.players[p], &self.world.players[q]);
        let mut v = 0.0;
        if (x.age as i32 - y.age as i32).abs() <= f(k, "kabine.beziehung_start_alter_diff") as i32 { v += f(k, "kabine.beziehung_start_alter"); }
        if x.pos == y.pos { v += f(k, "kabine.beziehung_start_konkurrenz"); }
        let ab = |i: usize, s: &str| cw(x, i) >= f(k, s) && cw(y, i) >= f(k, s);
        if ab(TEMPERAMENT, "kabine.beziehung_start_hitzig_ab") { v += f(k, "kabine.beziehung_start_hitzig"); }
        if ab(EGO, "kabine.beziehung_start_eitel_ab") { v += f(k, "kabine.beziehung_start_eitel"); }
        if ab(VERTRAEGLICH, "kabine.beziehung_start_vertraeglich_ab") { v += f(k, "kabine.beziehung_start_vertraeglich"); }
        if x.joined == y.joined { v += f(k, "kabine.beziehung_start_vereinszeit"); }
        if (part_of(&x.pos) as i32 - part_of(&y.pos) as i32).abs() == 1 { v += f(k, "kabine.beziehung_start_ergaenzung"); }
        if x.contract_end == y.contract_end { v += f(k, "kabine.beziehung_start_vertrag"); }
        let (a, b) = (x.id.min(y.id).to_string(), x.id.max(y.id).to_string());
        v + (zufall(&["2026", crate::id("beziehung"), &a, &b]) * 2.0 - 1.0) * f(k, "kabine.beziehung_start_streuung")
    }
    fn beziehungen_an(&mut self) {
        let sq = self.squad();
        let ids: Vec<u32> = sq.iter().map(|&p| self.world.players[p].id).collect();
        self.mensch.beziehungen.retain(|b| ids.contains(&b.a) && ids.contains(&b.b));
        if self.mensch.beziehungen.len() == ids.len() * ids.len().saturating_sub(1) / 2 { return; }
        for i in 0..sq.len() { for j in i + 1..sq.len() {
            let (a, b) = (ids[i], ids[j]);
            if self.bez_idx(a, b).is_none() {
                let w = self.bez_start(sq[i], sq[j]);
                self.mensch.beziehungen.push(Bez { a: a.min(b), b: a.max(b), w });
            }
        } }
    }
    /// Friends (≥ freunde_ab) and enemies (< rivale_unter) of a player in the squad.
    pub fn freunde_feinde(&self, p: usize) -> (usize, usize) {
        let id = self.world.players[p].id;
        let (fa, ru) = (f("entwicklung", "kabine.freunde_ab"), f("entwicklung", "kabine.rivale_unter"));
        self.mensch.beziehungen.iter().filter(|b| b.a == id || b.b == id).fold((0, 0), |a, b| (a.0 + (b.w >= fa) as usize, a.1 + (b.w < ru) as usize))
    }
    pub fn stufe(w: f32) -> &'static str {
        for s in arr("entwicklung", "kabine.stufen") { if w >= s[0].as_f64().unwrap() as f32 { return s[1].as_str().unwrap(); } }
        crate::id("spannung")
    }

    // ------------------------------------------------------------ morale (§3)
    fn erwartung(rolle: u8) -> f32 { f("entwicklung", &format!("moral.erwartet.{}", ROLLEN[rolle.min(6) as usize])) }
    pub fn monate_rest(&self, p: usize) -> f32 {
        let pl = &self.world.players[p];
        (pl.contract_end as f32 - self.world.season as f32) * 12.0 + (crate::date::SEASON_DAYS - self.world.day) as f32 / 30.4
    }
    pub fn vereins_ppg(&self) -> Option<f32> {
        let table = self.world.table(self.league());
        table.iter().find(|r| r.club == self.own).filter(|r| r.p > 0).map(|r| r.pts as f32 / r.p as f32)
    }
    /// The morale drivers of one player (regeln.py moral_treiber), negative ones already × sensitivity.
    pub fn moral_treiber(&self, p: usize) -> Vec<(String, f32)> {
        let m = |k: &str| f("entwicklung", &format!("moral.{k}"));
        let pl = &self.world.players[p];
        let c = |i| cw(pl, i);
        let mut tr: Vec<(String, f32)> = vec![];
        let abs = self.abs();
        // no complaints about playing time while nobody plays (Fabian 24.11.: after the season's end)
        if pl.inj == 0 && !pl.ps.einsatz.is_empty() && !self.spielfrei() {
            let anteil = pl.ps.einsatz.iter().sum::<f32>() / pl.ps.einsatz.len() as f32;
            let erwartet = if pl.ps.erweitert { 0.0 } else { Self::erwartung(pl.ps.rolle) };
            let mut d = ((anteil - erwartet) * m("spielzeit_faktor")).clamp(m("spielzeit_min"), m("spielzeit_max"));
            if d < 0.0 { d *= 0.7 + 0.6 * (c(EHRGEIZ) + c(EGO)) / 200.0; }
            tr.push((crate::id("spielzeit").into(), d));
        }
        if pl.ps.erweitert { tr.push(("erweiterter_kader".into(), -m("erweitert_abzug"))); } // roh
        if let Some(ppg) = self.vereins_ppg() { tr.push((crate::id("erfolg").into(), ((ppg - 1.5) * m("erfolg_faktor") * (0.8 + 0.4 * c(EHRGEIZ) / 100.0)).clamp(-m("erfolg_grenze"), m("erfolg_grenze")))); }
        let sq = self.squad();
        if sq.len() > 1 {
            let n1 = (sq.len() - 1) as f32;
            let lr = sq.iter().filter(|&&q| self.world.players[q].ovr < pl.ovr).count() as f32 / n1;
            let gr = sq.iter().filter(|&&q| self.world.players[q].wage < pl.wage).count() as f32 / n1;
            let d = lr - gr;
            if d > m("gehalt_schwelle") { tr.push((crate::id("gehalt").into(), (-d * m("gehalt_faktor") * (0.7 + 0.6 * c(EGO) / 100.0)).max(-m("gehalt_grenze")))); }
            else if -d > m("gehalt_schwelle") { tr.push((crate::id("gehalt").into(), ((-d - m("gehalt_schwelle")) * 4.0).min(3.0))); }
        }
        if self.monate_rest(p) < m("vertrag_kurz_monate") { tr.push((crate::id("vertrag").into(), -m("vertrag_unsicher") * (1.0 - c(LOYAL) / 100.0))); }
        let vs: Vec<&Versprechen> = self.mensch.versprechen.iter().filter(|v| v.pid == p && v.stand < 3 && abs < v.entschieden.max(v.seit) + 120).collect();
        if !vs.is_empty() {
            let gb = (vs.iter().filter(|v| v.stand == 2).count() as f32 * -m("zusage_gebrochen")).max(-m("zusage_gebrochen_deckel"));
            let gh = (vs.iter().filter(|v| v.stand == 1).count() as f32 * m("zusage_gehalten")).min(m("zusage_gehalten_deckel"));
            let of = vs.iter().filter(|v| v.stand == 0).count() as f32 * -m("zusage_offen");
            tr.push((crate::id("zusagen").into(), gb + gh + of));
        }
        let hw = m("gespraech_halbwertszeit");
        let g: f32 = pl.ps.gespr.iter().filter(|x| x.0 <= abs).map(|x| x.1 * 0.5f32.powf((abs - x.0) as f32 / hw)).sum();
        if g.abs() > 0.05 { tr.push((crate::id("gespraech").into(), g.clamp(-m("gespraech_deckel"), m("gespraech_deckel")))); }
        if pl.ps.rolle >= 5 && c(EGO) >= m("rolle_ego_ab") { tr.push((crate::id("rolle").into(), -m("rolle_abzug"))); }
        let ang = pl.ps.angebote.iter().filter(|&&d| abs < d + 60).count() as f32;
        if ang > 0.0 && (c(EGO) >= 60.0 || c(EHRGEIZ) >= 60.0) { tr.push((crate::id("geruecht").into(), (ang * m("geruecht_zuschlag")).min(m("geruecht_deckel")))); }
        let (fr, fe) = self.freunde_feinde(p);
        if fr > 0 { tr.push((crate::id("freunde").into(), (fr as f32 * m("freund_zuschlag")).min(m("freund_deckel")))); } else if sq.len() > 1 { tr.push((crate::id("isoliert").into(), -m("isoliert_abzug") * (0.5 + c(TEAMGEIST) / 100.0))); }
        if fe > 0 { tr.push((crate::id("feinde").into(), (fe as f32 * -m("feind_abzug")).max(-m("feind_deckel")))); }
        let schuetz = sq.iter().filter(|&&q| self.world.players[q].ps.mentor == Some(pl.id)).count() as f32;
        if pl.ps.mentor.is_some() || schuetz > 0.0 { tr.push((crate::id("mentor").into(), if pl.ps.mentor.is_some() { m("mentor_bund") } else { 0.0 } + schuetz * m("schuetzling_zuschlag"))); }
        // own decision: the six training units of this build carry the morale of einheiten.toml (Regeneration +1, Ausdauer −1)
        let tw: f32 = self.plan.iter().map(|&u| match u { 0 => 1.0, 5 => -1.0, _ => 0.0 }).sum::<f32>() * m("training_faktor");
        if tw != 0.0 { tr.push((crate::id("training").into(), tw.clamp(-m("training_deckel"), m("training_deckel")))); }
        let tg = self.mensch.teamgeist - 50.0;
        if tg.abs() >= 3.0 { tr.push((crate::id("gruppe").into(), tg * m("gruppe_faktor") * (0.6 + 0.8 * c(TEAMGEIST) / 100.0))); }
        let pf = pfeil(pl.form) as f32;
        if pf != 2.0 { tr.push((crate::id("form").into(), (pf - 2.0) * m("form_je_pfeil"))); }
        if pl.inj > 0 { tr.push((crate::id("verletzt").into(), -m("verletzt_abzug"))); }
        // load zone „überlastet“ from 70 (entwicklung.toml belastung.zonen)
        if pl.ps.last >= 70.0 { tr.push((crate::id("ueberlastet").into(), -m("ueberlastung_abzug"))); }
        let emp = (1.0 + 0.25 * (c(TEMPERAMENT) - 50.0) / 50.0 - 0.2 * (c(PROF) - 50.0) / 50.0).clamp(0.7, 1.4);
        for x in tr.iter_mut() { if x.1 < 0.0 { x.1 *= emp; } }
        tr
    }

    /// One day of the people side for the own squad.
    pub fn psyche_tag(&mut self) {
        self.psyche_an();
        self.world.teamgeist_eigen = Some((self.own, self.mensch.teamgeist));
        if self.world.day % 7 == 0 { self.hierarchie(); }
        self.nummern_vergeben();
        self.beziehungen_an();
        self.kabine_tag();
        let sq = self.squad();
        let abs = self.abs();
        let rate = f("entwicklung", "moral.rate");
        let (lo, hi) = (f("entwicklung", "moral.deckel_unten"), f("entwicklung", "moral.deckel_oben"));
        let fo = |k: &str| f("entwicklung", &format!("form.{k}"));
        let we = |k: &str| f("entwicklung", &format!("wechsel.{k}"));
        let mut anliegen = vec![];
        for &p in &sq {
            let tr = self.moral_treiber(p);
            let ziel = (50.0 + tr.iter().map(|x| x.1).sum::<f32>()).clamp(lo, hi);
            let isoliert = tr.iter().any(|x| x.0 == crate::id("isoliert"));
            let rest = self.monate_rest(p);
            let pl = &mut self.world.players[p];
            pl.morale += rate * (ziel - pl.morale);
            pl.ps.treiber = tr;
            pl.ps.gespr.retain(|x| abs < x.0 + 150);
            pl.ps.angebote.retain(|&d| abs < d + 60);
            let n = f("entwicklung", "selbstvertrauen.neutral");
            pl.ps.sv += (n - pl.ps.sv) * f("entwicklung", "selbstvertrauen.rueckkehr_zur_mitte");
            let k = if pl.fit < fo("kondition_unter") { fo("kondition_abzug") * (fo("kondition_unter") - pl.fit) / fo("kondition_unter") * 2.0 } else { 0.0 };
            let fz = (pl.morale - 50.0) / 50.0 * fo("moral_gewicht") + (pl.ps.sv - 50.0) / 50.0 * fo("selbstvertrauen_gewicht") - k + pl.ps.phasen_form();
            pl.form = (pl.form + fo(crate::id("rate")) * (fz - pl.form)).clamp(-fo(crate::id("bereich")), fo(crate::id("bereich")));
            let loy = cw(pl, LOYAL);
            let wz = we(crate::id("neutral")) + (50.0 - pl.morale).max(0.0) * we("moral_hebel") + (1.0 - loy / 100.0) * we("loyalitaet_hebel") * if pl.morale < 50.0 { 1.0 } else { 0.3 }
                + pl.ps.angebote.len() as f32 * we("angebot_je") + if rest < 12.0 { we("vertrag_kurz_zuschlag") } else { 0.0 } + if isoliert { 5.0 } else { 0.0 };
            pl.ps.wb += we(crate::id("rate")) * (wz.clamp(0.0, 100.0) - pl.ps.wb);
            if !pl.ps.unzufrieden && pl.morale < f("entwicklung", "moral.unzufrieden_unter") { pl.ps.unzufrieden = true; anliegen.push((p, true)); }
            else if pl.ps.unzufrieden && pl.morale >= f("entwicklung", "moral.unzufrieden_wieder") { pl.ps.unzufrieden = false; }
            pl.unhappy_days = if pl.ps.unzufrieden { pl.unhappy_days.saturating_add(1) } else { 0 };
            if !pl.ps.wunsch && pl.ps.wb >= we("wunsch_ab") { pl.ps.wunsch = true; anliegen.push((p, false)); }
            else if pl.ps.wunsch && pl.ps.wb < we("wunsch_zurueck_unter") {
                pl.ps.wunsch = false;
                let n = pl.name.clone();
                self.msg(1, &n, &crate::tf!("gs.vorfall.angebot_antwort_spieler_will_bleiben.betreff", n), &crate::tf!("ui.mensch.hat_seinen_wechselwunsch_zurueckgenommen", n), Action::None);
            }
        }
        for (p, unz) in anliegen {
            if unz {
                let tr = &self.world.players[p].ps.treiber;
                let schlimm = tr.iter().filter(|x| x.1 < 0.0).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|x| x.0.clone()).unwrap_or_default();
                let thema = match schlimm.as_str() { "spielzeit" => "spielzeit", "rolle" => "rolle", "gehalt" | "vertrag" => "zukunft", _ => crate::id("form") };
                self.anliegen(p, thema, true);
            } else {
                let n = self.world.players[p].name.clone();
                self.news(&crate::tf!("gs.vorfall.anliegen_stufe2.betreff", n), &crate::tf!("ui.mensch.denkt_an_einen_wechsel_sein", n));
                self.anliegen(p, crate::id("zukunft"), true);
            }
        }
        self.themen_tag();
        self.pool_tag();
        self.zeitung_woche();
        self.sendungen_woche();
        self.verlauf_tag();
        self.ehrungen_woche();
        self.ehrungen_monat();
        self.vorbereitung_tag();
        self.versprechen_tag();
        self.disziplin_tag();
        self.gutgespraech_woche();
    }

    /// After an own match: minutes, self-confidence, relations and the club climate.
    pub fn nach_spiel(&mut self, res: &MatchResult) {
        self.psyche_an();
        let home = res.h == self.own;
        let (gf, ga) = if home { (res.hg, res.ag) } else { (res.ag, res.hg) };
        let sv = |k: &str| f("entwicklung", &format!("selbstvertrauen.{k}"));
        let played: Vec<(usize, u8)> = res.minutes.iter().copied().filter(|m| self.world.players[m.0].team == self.own as i32).collect();
        let w = f("entwicklung", "moral.spielzeit_fenster") as usize;
        for p in self.squad() {
            let min = played.iter().find(|m| m.0 == p).map(|m| m.1).unwrap_or(0);
            let tore = res.events.iter().filter(|e| e.kind == 0 && e.pid == p).count() as f32;
            let note = res.ratings.iter().find(|r| r.0 == p).map(|r| r.1).unwrap_or(sv("note_neutral"));
            let pl = &mut self.world.players[p];
            pl.ps.einsatz.push((min as f32 / 90.0).min(1.0));
            if min > 0 { pl.ps.last += f("entwicklung", "belastung.spiel"); }
            // own decision: pressing costs 2 more load per match, holding back saves 2 (Paket 4 instructions)
            let anw = self.std.anweisungen.iter().find(|x| x.0 == self.world.players[p].id).map(|x| x.1).unwrap_or(0);
            let pl = &mut self.world.players[p];
            if min > 0 { pl.ps.last = (pl.ps.last + match anw { 3 => 2.0, 2 => -2.0, _ => 0.0 }).max(0.0); }
            if pl.ps.einsatz.len() > w { pl.ps.einsatz.remove(0); }
            if min == 0 { continue; }
            let d = if gf > ga { sv(crate::id("sieg")) } else if gf < ga { sv(crate::id("niederlage")) } else { 0.0 } + tore * sv(crate::id("tor")) + (note - sv("note_neutral")) * sv("note_faktor");
            pl.ps.sv = (pl.ps.sv + d).clamp(0.0, 100.0);
        }
        let ids: Vec<u32> = played.iter().map(|m| self.world.players[m.0].id).collect();
        let k = if gf > ga { f("entwicklung", "kabine.beziehung_spiel_sieg") } else { f("entwicklung", "kabine.beziehung_spiel_sonst") };
        for b in self.mensch.beziehungen.iter_mut() { if ids.contains(&b.a) && ids.contains(&b.b) { b.w = (b.w + k).min(100.0); } }
        self.klima_nach_spiel();
    }

    // ------------------------------------------------------------ dressing room: team spirit, conflicts (§6)
    fn kabine_tag(&mut self) {
        let k = |s: &str| f("entwicklung", &format!("kabine.{s}"));
        let abs = self.abs();
        let unz: Vec<u32> = self.squad().iter().filter(|&&p| self.world.players[p].ps.unzufrieden).map(|&p| self.world.players[p].id).collect();
        let mentoren: Vec<(u32, u32)> = self.squad().iter().filter_map(|&p| self.world.players[p].ps.mentor.map(|m| (m, self.world.players[p].id))).collect();
        let mut neu = vec![];
        for b in self.mensch.beziehungen.iter_mut() {
            if unz.contains(&b.a) || unz.contains(&b.b) { b.w += k("beziehung_unzufrieden"); }
            if mentoren.iter().any(|m| (m.0 == b.a && m.1 == b.b) || (m.0 == b.b && m.1 == b.a)) { b.w += k("beziehung_mentor_je_tag"); }
            b.w -= b.w * k("beziehung_zur_mitte");
            if b.w < k("konflikt_unter") && zufall(&["2026", crate::id("konflikt"), &b.a.to_string(), &b.b.to_string(), &abs.to_string(), &self.salz]) < k("konflikt_chance") { neu.push((b.a, b.b)); }
        }
        for (a, b) in neu {
            if self.mensch.konflikte.iter().any(|x| (x.a == a && x.b == b) || (x.a == b && x.b == a)) { continue; }
            self.mensch.konflikte.push(Konflikt { a, b, seit: abs, schwere: 0 });
            if let (Some(pa), Some(pb)) = (self.pid(a), self.pid(b)) {
                let (na, nb) = (self.world.players[pa].name.clone(), self.world.players[pb].name.clone());
                let opts = self.optionen_fuer(crate::id("konflikt"), &abs.to_string());
                let co = self.co_name.clone();
                self.msg(1, &co, &crate::tf!("ui.mensch.streit_in_der_kabine_und", na, nb), &crate::tf!("ui.mensch.und_sind_heftig_aneinandergeraten_die", na, nb), Action::Wahl { art: crate::id("konflikt").into(), bezug: vec![pa, pb], optionen: opts });
            }
        }
        let esk = f("konflikte", "erkennen.streit_eskaliert_tage") as u32;
        let ende = k("konflikt_ende_tage") as u32;
        for x in self.mensch.konflikte.iter_mut() { if x.schwere == 0 && abs >= x.seit + esk { x.schwere = 1; } }
        self.mensch.konflikte.retain(|x| abs < x.seit + ende);
        // Teamgeist (entwicklung/kabine.py:76)
        let sq = self.squad();
        if sq.is_empty() { return; }
        let mm = sq.iter().map(|&p| self.world.players[p].morale).sum::<f32>() / sq.len() as f32;
        let kap = self.kapitaen().map(|p| cw(&self.world.players[p], FUEHRUNG)).unwrap_or(50.0);
        let ppg = self.vereins_ppg().unwrap_or(1.5);
        let ziel = k("teamgeist_neutral") + (mm - 50.0) * k("moral_gewicht") - k("konflikt_abzug") * self.mensch.konflikte.len() as f32 + (kap - 50.0) * k("kapitaen_gewicht") + (ppg - 1.5) * 3.0 + self.event_summand();
        self.mensch.teamgeist += k(crate::id("rate")) * (ziel.clamp(0.0, 100.0) - self.mensch.teamgeist);
    }
    pub fn pid(&self, id: u32) -> Option<usize> { self.world.clubs[self.own].squad.iter().copied().find(|&p| self.world.players[p].id == id) }
    pub fn optionen(&self, thema: &str) -> Vec<(String, String)> { self.optionen_fuer(thema, "") }
    /// The answers of a theme in one of their wordings (Fabian 22.11.: not always the same words): [key, text, variant …],
    /// picked by `samen` (player and day), the first wording without one.
    pub fn optionen_fuer(&self, thema: &str, samen: &str) -> Vec<(String, String)> {
        crate::regeln::text_wert(&format!("optionen.{thema}")).as_array().map(|a| a.as_slice()).unwrap_or(&[]).iter().map(|o| {
            let n = o.as_array().map_or(2, |a| a.len()).max(2) - 1;
            let i = if samen.is_empty() { 0 } else { (zufall(&[crate::id("wortlaut"), samen, o[0].as_str().unwrap_or("")]) * n as f32) as usize % n };
            (o[0].as_str().unwrap().to_string(), o[1 + i].as_str().unwrap_or(o[1].as_str().unwrap()).to_string())
        }).collect()
    }

    // ------------------------------------------------------------ conversations (§1)
    /// Trust of a player in the coach (werte.py:95).
    pub fn vertrauen(&self, p: usize) -> f32 {
        let g = |k: &str| f("gespraeche", &format!("motor.{k}"));
        let abs = self.abs();
        let s: f32 = self.mensch.versprechen.iter().filter(|v| v.pid == p && (v.stand == 1 || v.stand == 2)).map(|v| {
            let x = if v.stand == 1 { g("vertrauen_gehalten") } else { g("vertrauen_gebrochen") };
            x * 0.5f32.powf(abs.saturating_sub(v.entschieden) as f32 / g("vertrauen_halbwert_tage")) * v.gewicht
        }).sum();
        (g("vertrauen_start") + s).clamp(0.0, 100.0)
    }
    /// A player comes with a concern (topic engine with throttle: 1 per day, 3 per week; important ones always).
    /// The time without competitive matches: after the last league matchday until the first of the next season.
    pub fn spielfrei(&self) -> bool { self.world.day > crate::world::LEAGUE_END || self.world.day < crate::world::LEAGUE_START }
    pub fn anliegen(&mut self, p: usize, thema: &str, wichtig: bool) {
        if thema == "spielzeit" && self.spielfrei() { return; }
        let abs = self.abs();
        if self.world.players[p].ps.sperre.iter().any(|s| s.0 == thema && abs < s.1) { return; }
        self.mensch.anliegen.retain(|&d| abs < d + 7);
        if !wichtig && (self.mensch.anliegen.iter().any(|&d| d == abs) || self.mensch.anliegen.len() >= 3) { return; }
        self.mensch.anliegen.push(abs);
        let n = self.world.players[p].name.clone();
        let sperre = f("gespraeche", "motor.sperre_tage") as u32;
        self.world.players[p].ps.sperre.retain(|s| abs < s.1);
        self.world.players[p].ps.sperre.push((thema.into(), abs + sperre.max(21)));
        let anlass = match thema { "spielzeit" => Some(crate::id("spielzeit")), "rolle" => Some(crate::id("rolle")), "form" => Some(crate::id("formtief")), "lob" => Some(crate::id("lob")),
            "kritik" => Some(crate::id("kritik_nach_ausrutscher")), "zukunft" => Some(crate::id("vertrag")), _ => None };
        if let Some(tid) = anlass.and_then(|a| crate::pool::thema_fuer(a, &self.pool_kontext(p).0, true)) {
            if self.pool_starten(p, &tid) { return; }
        }
        let opts = self.optionen_fuer(thema, &format!("{}:{abs}", self.world.players[p].id));
        // how he comes to you depends on his mood and his character (Fabian 24.11.): same topic, another tone
        let ton = self.haltung_ton(p);
        let vk = self.vkey(&format!("ui.ton.auftakt.{ton}"), &format!("{}{abs}", self.world.players[p].id));
        let text = format!("{}\n\n{}", crate::tf!(&vk, n.clone()), t(&format!("anlass.{thema}")));
        let (title, from) = if thema == "lob" || thema == crate::id("kritik") { (crate::tf!("ui.mensch.gespraech_mit", n), self.co_name.clone()) } else { (crate::tf!(&format!("ui.ton.titel.{ton}"), n), n.clone()) };
        self.msg(1, &from, &title, &text, Action::Wahl { art: format!("gespraech:{thema}"), bezug: vec![p], optionen: opts });
    }
    /// The tone a player takes with the coach: furious (low mood, hot temper), demanding (big ego or status), hurt (low
    /// mood), unsure (weak nerves), else polite.
    pub fn haltung_ton(&self, p: usize) -> &'static str {
        let pl = &self.world.players[p];
        let k = |i| cw(pl, i);
        if pl.morale < 40.0 && k(TEMPERAMENT) >= 60.0 { crate::id("wuetend") }
        else if k(EGO) >= 65.0 || k(STATUS) >= 70.0 { crate::id("fordernd") }
        else if pl.morale < 45.0 { crate::id("enttaeuscht") }
        else if k(NERVEN) < 40.0 { crate::id("unsicher") }
        else { crate::id("hoeflich") }
    }
    /// His reaction to the answer, in his own way: a hothead slams the door, a professional nods, a sensitive one
    /// goes quiet (the effect itself comes from gespraech()).
    pub fn reaktion_text(&mut self, p: usize, d: f32) -> String {
        let pl = &self.world.players[p];
        let k = |i| cw(pl, i);
        let typ = if k(TEMPERAMENT) >= 65.0 { crate::id("hitzkopf") } else if k(PROF) >= 65.0 { crate::id("profi") } else if k(NERVEN) < 40.0 { crate::id("sensibel") } else if k(EGO) >= 65.0 { crate::id("ego") } else { crate::id("normal") };
        let stufe = if d >= 2.0 { crate::id("gut") } else if d <= -2.0 { crate::id("schlecht") } else { crate::id("mittel") };
        let n = pl.name.clone();
        let vk = self.vkey(&format!("ui.ton.reaktion.{typ}.{stufe}"), &format!("{}{}", pl.id, self.abs()));
        crate::tf!(&vk, n)
    }
    /// Weekly topic engine (themen.toml): a fixed chance per week and player.
    fn themen_tag(&mut self) {
        if !self.feature("gespraeche") { return; } // roh
        if crate::date::weekday(self.world.day) != 1 || self.abs() < self.ruhe_bis { return; }
        let woche = (self.abs() / 7).to_string();
        for p in self.squad() {
            let pl = &self.world.players[p];
            let id = pl.id.to_string();
            let z = |a: &str| zufall(&["2026", crate::id("anlass"), a, &id, &woche, &self.salz]);
            let sz = pl.ps.treiber.iter().find(|x| x.0 == crate::id("spielzeit")).map(|x| x.1).unwrap_or(0.0);
            let thema = if sz <= -4.0 && z(crate::id("spielzeit")) < 0.8 { Some(crate::id("spielzeit")) }
                else if pl.ps.rolle >= 5 && cw(pl, EGO) >= 60.0 && z(crate::id("rolle")) < 0.7 { Some(crate::id("rolle")) }
                else if pfeil(pl.form) <= 1 && pl.morale < 50.0 && z(crate::id("formtief")) < 0.6 { Some(crate::id("form")) }
                else if pl.morale >= 65.0 && pfeil(pl.form) >= 3 && z(crate::id("lob")) < 0.3 { Some(crate::id("lob")) }
                else if pl.last.last().map_or(false, |&r| r < 5.5) && z(crate::id("kritik")) < 0.2 { Some(crate::id("kritik")) }
                else if self.monate_rest(p) <= 8.0 && pl.ps.rolle <= 4 && z(crate::id("vertrag")) < 0.4 { Some(crate::id("zukunft")) }
                else { None };
            if let Some(th) = thema { self.anliegen(p, th, false); }
        }
    }
    /// One conversation answer: start mood, tone change, trust damping, after-effects, promise. Returns the mood change.
    pub fn gespraech(&mut self, p: usize, thema: &str, option: &str) -> f32 {
        let g = |k: &str| f("gespraeche", &format!("motor.{k}"));
        let Some(opt) = arr("gespraeche", &format!("thema.{thema}.optionen")).iter().find(|o| o["id"].as_str() == Some(option)) else { return 0.0 };
        let ton = opt["ton"].as_str().unwrap();
        let pl = &self.world.players[p];
        let chs: Vec<f32> = (0..10).map(|i| cw(pl, i)).collect();
        let anlass = pl.ps.treiber.iter().filter(|x| x.1 < 0.0).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|x| x.0 == thema).unwrap_or(false);
        let start = (g("stimmung_start") + (pl.morale - 50.0) * g("moral_einfluss") + pl.ps.bez * g("beziehung_einfluss") - if anlass { g("anlass_abzug") } else { 0.0 }).clamp(5.0, 95.0);
        // the manager's people skills scale the effect; the tone counts for his style (manager.toml)
        let d = ton_aenderung(ton, &chs, start, self.vertrauen(p)) * self.mgr_faktor(0);
        let vorher = (self.vertrauen(p), self.world.players[p].ps.bez, self.mensch.teamgeist);
        self.stil_punkte(crate::id("ton"), match ton { "warm" | "lob" => "einfuehlsam", "sachlich" => "sachlich", "hart" => "hart", "bestimmt" => "bestimmt", _ => crate::id("offen") });
        let grund = crate::tf!("ui.mensch.gespraech", self.optionen(thema).iter().find(|o| o.0 == option).map(|o| o.1.clone()).unwrap_or_default());
        self.beziehung(p, d * g("beziehung_je_stimmung"), &grund);
        self.moral_wirkung(p, d, g("folge_tage") as u32, &grund);
        if let Some(art) = opt["versprechen"].as_str() { self.versprechen_geben(p, art); }
        self.wellen(p, d, opt["versprechen"].as_str(), &grund, vorher);
        // praise or a hard word can start a phase of the development (wochenlauf.anlaesse_ablegen)
        if matches!(ton, "lob" | "warm") && d > 0.0 { self.anlass(p, crate::id("lob"), (d.abs() / 5.0 + 0.3).min(1.5)); }
        if matches!(ton, "hart" | "kritik") && d < 0.0 { self.anlass(p, crate::id("kritik"), (d.abs() / 5.0 + 0.3).min(1.5)); }
        d
    }
    /// A talk reaches others too (Mechanik-Übergabe §6): friends take on part of the mood change, a talk with the captain or a
    /// leader moves the team spirit, and whoever sees a teammate of the same role preferred on the same topic is unhappy.
    /// Every effect gets a reason; the card „So hat es gewirkt“ shows them (Career::wirkung).
    pub(crate) fn wellen(&mut self, p: usize, d: f32, zusage: Option<&str>, grund: &str, vorher: (f32, f32, f32)) {
        let w = |k: &str| f("entwicklung_pesm", &format!("wellen.{k}"));
        let tage = f("gespraeche", "motor.folge_tage") as u32;
        let (id, name, teil, rolle) = { let pl = &self.world.players[p]; (pl.id, pl.name.clone(), part_of(&pl.pos), pl.ps.rolle) };
        let mut andere: Vec<(usize, f32, String)> = vec![];
        let freund_ab = f("entwicklung", "kabine.freunde_ab");
        let zusage = zusage.filter(|z| arr("entwicklung_pesm", "wellen.gleich_arten").iter().any(|a| a.as_str() == Some(*z)));
        for q in self.squad() {
            if q == p || andere.len() >= w(crate::id("hoechstens")) as usize { continue; }
            let b = self.bez_wert(id, self.world.players[q].id);
            if b >= freund_ab {
                let dq = d * w("freund_anteil") * b / 100.0;
                if dq.abs() >= 0.1 { let g = crate::tf!("ui.wellen.freund", name); self.moral_wirkung(q, dq, tage, &g); andere.push((q, dq, g)); }
            } else if let Some(z) = zusage {
                let qp = &self.world.players[q];
                if part_of(&qp.pos) == teil && (qp.ps.rolle as i32 - rolle as i32).abs() <= 1 {
                    let g = crate::tf!(&format!("ui.wellen.gleich_{z}"), name);
                    self.moral_wirkung(q, -w("gleich_abzug"), 0, &g);
                    andere.push((q, -w("gleich_abzug"), g));
                }
            }
        }
        if rolle <= 2 {
            let tg = d * w("fuehrung_teamgeist");
            self.mensch.teamgeist = (self.mensch.teamgeist + tg).clamp(0.0, 100.0);
            self.grund(crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("ui.wellen.teamgeist"), tg, &crate::tf!("ui.wellen.fuehrung", name));
        }
        let nachher = (self.vertrauen(p), self.world.players[p].ps.bez, self.mensch.teamgeist);
        self.wirkung = Some(Wirkung { pid: p, moral: d, vertrauen: nachher.0 - vorher.0, beziehung: nachher.1 - vorher.1, teamgeist: nachher.2 - vorher.2, grund: grund.to_string(), andere, seit: crate::ui::jetzt() });
    }
    /// A short talk from the squad page (entwicklung/kabine.py:87): 30 % at once, 40 % after 3 days, 7 days lock.
    pub fn kabinen_gespraech(&mut self, p: usize, ton: &str) -> f32 {
        let k = |s: &str| f("entwicklung", &format!("kabine.{s}"));
        let pl = &self.world.players[p];
        let c = |i| cw(pl, i);
        let emp = (1.0 + 0.25 * (c(TEMPERAMENT) - 50.0) / 50.0 - 0.2 * (c(PROF) - 50.0) / 50.0).clamp(0.7, 1.4);
        let base = f("entwicklung", &format!("kabine.toene.{ton}"));
        let w = match ton {
            "lob" => base * (0.8 + 0.4 * c(EGO) / 100.0) * if pl.morale > 75.0 { 0.5 } else { 1.0 },
            "ermutigen" => base * if c(NERVEN) < 45.0 { 1.3 } else { 1.0 } * if pl.morale < 40.0 { 1.2 } else { 1.0 },
            "kritik" => if c(EHRGEIZ) >= 60.0 && c(PROF) >= 50.0 { 1.5 } else { base * emp },
            "hart" => base * emp * if c(TEMPERAMENT) >= 60.0 { 1.4 } else { 1.0 },
            _ => base,
        };
        let grund = crate::tf!("ui.mensch.kurzes_gespraech_in_der_kabine", ton);
        self.moral_wirkung(p, w * k("sofort_anteil"), 0, &grund);
        self.moral_wirkung(p, w * k("folge_anteil"), k("folge_tage") as u32, &grund);
        let abs = self.abs();
        let pl = &mut self.world.players[p];
        pl.ps.sv = (pl.ps.sv + w * k("selbstvertrauen_je_wirkung")).clamp(0.0, 100.0);
        pl.talk_day = abs.max(1);
        w
    }
    pub fn mimik(d: f32) -> &'static str {
        let gr = arr("gespraeche", "motor.mimik_grenzen");
        let i = gr.iter().position(|x| d >= x.as_f64().unwrap() as f32).unwrap_or(4);
        crate::regeln::text_wert("mimik").as_array().unwrap()[i].as_str().unwrap()
    }
    pub fn versprechen_geben(&mut self, p: usize, art: &str) {
        let abs = self.abs();
        let frist = f("gespraeche", &format!("versprechen.frist_tage.{art}")) as u32;
        let gewicht = f("gespraeche", &format!("versprechen.gewicht.{art}"));
        let basis = if art == crate::id("vertrag") { self.world.players[p].contract_end as f64 } else { self.world.players[p].wage };
        self.mensch.versprechen.push(Versprechen { pid: p, art: art.into(), seit: abs, frist: abs + frist, gewicht, stand: 0, basis, entschieden: 0 });
        let n = self.world.players[p].name.clone();
        self.grund(&n, crate::inhalt::t("ui.mensch.versprechen"), 0.0, &crate::tf!("ui.mensch.zusage_gegeben_frist_tage", crate::inhalt::t(&format!("ui.zusage.art.{art}")), frist));
    }
    /// Decide a promise: kept (true) or broken.
    pub fn versprechen_entscheiden(&mut self, i: usize, gehalten: bool) {
        let abs = self.abs();
        let v = &mut self.mensch.versprechen[i];
        if v.stand != 0 { return; }
        v.stand = if gehalten { 1 } else { 2 };
        v.entschieden = abs;
        let (p, art) = (v.pid, crate::inhalt::t(&format!("ui.zusage.art.{}", v.art)).to_string());
        let g = |k: &str| f("gespraeche", &format!("motor.{k}"));
        let n = self.world.players[p].name.clone();
        if gehalten {
            self.beziehung(p, g("beziehung_gehalten"), &crate::tf!("ui.mensch.zusage_gehalten", art));
            self.verschieben(p, LOYAL, 2.0, crate::inhalt::t("story.regel.versprechen_gehalten"));
            self.verschieben(p, PROF, 1.0, crate::inhalt::t("story.regel.versprechen_gehalten"));
            self.anlass(p, crate::id("zusage_gehalten"), 1.0);
        } else {
            self.beziehung(p, g("beziehung_gebrochen"), &crate::tf!("ui.mensch.zusage_gebrochen", art));
            self.verschieben(p, LOYAL, -3.0, crate::inhalt::t("story.regel.versprechen_gebrochen"));
            self.world.players[p].ps.versprechen_gebrochen = abs;
            self.anlass(p, crate::id("zusage_gebrochen"), 1.0);
            self.verschieben(p, VERTRAEGLICH, -1.0, crate::inhalt::t("story.regel.versprechen_gebrochen"));
            let (vt, vb) = (self.variante("getaeuscht_titel", crate::inhalt::varianten("ui.mensch.fuehlt_sich_getaeuscht"), &n), self.variante("getaeuscht_text", crate::inhalt::varianten("ui.mensch.du_hattest_etwas_versprochen_er"), &n)); // roh
            self.msg(1, &n, &crate::tf!(&crate::inhalt::variante_key("ui.mensch.fuehlt_sich_getaeuscht", vt), n.clone()), &crate::tf!(&crate::inhalt::variante_key("ui.mensch.du_hattest_etwas_versprochen_er", vb), n, art), Action::None);
        }
    }
    fn versprechen_tag(&mut self) {
        let abs = self.abs();
        let anteil = f("gespraeche", "versprechen.spielzeit_anteil");
        let anstieg = f("gespraeche", "versprechen.gehalt_anstieg") as f64;
        for i in 0..self.mensch.versprechen.len() {
            let v = self.mensch.versprechen[i].clone();
            if v.stand != 0 { continue; }
            let pl = &self.world.players[v.pid];
            let weg = pl.team != self.own as i32;
            let erfuellt = match v.art.as_str() {
                "spielzeit" => !pl.ps.einsatz.is_empty() && pl.ps.einsatz.iter().sum::<f32>() / pl.ps.einsatz.len() as f32 >= anteil,
                "rolle" => pl.ps.rolle <= 3,
                "vertrag" => pl.contract_end as f64 > v.basis,
                "gehalt" => pl.wage >= v.basis * anstieg,
                _ => weg || pl.listed,
            };
            if erfuellt && (v.art != "spielzeit" || abs >= v.frist) { self.versprechen_entscheiden(i, true); }
            else if abs >= v.frist {
                // a player who left before the deadline: the promise lapses (stand 3), unless it was the release itself
                if weg && v.art != crate::id("freigabe") { self.mensch.versprechen[i].stand = 3; self.mensch.versprechen[i].entschieden = abs; } else { self.versprechen_entscheiden(i, false); }
            }
        }
        self.mensch.versprechen.retain(|v| v.stand == 0 || abs < v.entschieden + 400);
    }
    /// An offer for the player was turned down while a release was promised: promise broken (beziehung_verweigert).
    pub fn freigabe_verweigert(&mut self, p: usize) {
        if let Some(i) = self.mensch.versprechen.iter().position(|v| v.pid == p && v.art == crate::id("freigabe") && v.stand == 0) {
            self.beziehung(p, f("gespraeche", "versprechen.beziehung_verweigert"), crate::inhalt::t("ui.mensch.freigabe_verweigert"));
            self.versprechen_entscheiden(i, false);
        }
    }

    // ------------------------------------------------------------ discipline (§1.6)
    pub(crate) fn disziplin_tag(&mut self) {
        let d = |k: &str| f("gespraeche", &format!("disziplin.{k}"));
        let abs = self.abs();
        // own decision: incidents share the throttle of normal concerns (drosselung.toml: 1 per day, 3 per week)
        self.mensch.anliegen.retain(|&x| abs < x + 7);
        if self.mensch.anliegen.iter().any(|&x| x == abs) || self.mensch.anliegen.len() >= 3 { return; }
        // Fabian 23.11.: far too many — at most one incident in the whole squad within the club quiet time
        if abs < self.ruhe_bis { return; }
        let (letzt, letzte_art) = self.vorfall_zuletzt.clone();
        if letzt > 0 && abs < letzt + d("verein_ruhe_tage") as u32 { return; }
        for p in self.squad() {
            let pl = &self.world.players[p];
            if pl.youth > 0 || (pl.ps.vorfall > 0 && abs < pl.ps.vorfall + d("vorfall_sperre_tage") as u32) { continue; }
            let (pr, te) = (cw(pl, PROF) / 100.0, cw(pl, TEMPERAMENT) / 100.0);
            let chance = d("vorfall_je_tag") * (1.5 - pr * 1.5).max(0.0) * (1.0 + (50.0 - pl.morale) / 100.0) * (1.0 + te - 0.5);
            let id = pl.id.to_string();
            if zufall(&["2026", crate::id("vorfall"), &id, &abs.to_string(), &self.salz]) >= chance { continue; }
            // the kind fits the person, and never the same kind twice in a row
            let arten: Vec<&str> = arr("gespraeche", "disziplin.arten").iter().map(|a| a.as_str().unwrap())
                .filter(|&a| (a != "oeffentliche_kritik" || cw(pl, EGO) >= 55.0) && (a != "streit" || cw(pl, TEMPERAMENT) >= 55.0) && (a != "nachtleben" || pl.age <= 27) && (a != "uebergewicht" || pl.age >= 24) && a != letzte_art).collect();
            if arten.is_empty() { continue; }
            let art = arten[(zufall(&["2026", "vorfall_art", &id, &abs.to_string(), &self.salz]) * arten.len() as f32) as usize % arten.len()];
            let variante = (zufall(&["vorfall_text", &id, &abs.to_string()]) * 3.0) as usize % 3; // roh
            self.vorfall_zuletzt = (abs, art.to_string());
            let n = pl.name.clone();
            self.world.players[p].ps.vorfall = abs;
            self.mensch.anliegen.push(abs);
            let opts = self.optionen_fuer(crate::id("disziplin"), &format!("{id}:{abs}"));
            let co = self.co_name.clone();
            // a row is always with someone we can see (Fabian 24.11.): a teammate, or the captain in variant 1
            let mit = if art == "streit" { // roh
                let kap = self.kapitaen().filter(|&k| k != p);
                let sq: Vec<usize> = self.squad().into_iter().filter(|&q| q != p && Some(q) != kap).collect();
                if variante == 1 { kap } else if sq.is_empty() { None } else { Some(sq[(zufall(&["streit_mit", &id, &abs.to_string()]) * sq.len() as f32) as usize % sq.len()]) } // roh
            } else { None };
            let was = match mit { Some(m) => crate::tf!(&format!("ui.vorfall.{art}_mit.{variante}"), self.world.players[m].name.clone()), None => crate::inhalt::t(&format!("ui.vorfall.{art}.{variante}")).to_string() };
            self.msg(1, &crate::tf!("ui.pool.von_co", co), &crate::tf!("ui.mensch.vorfall", n), &crate::tf!("ui.mensch.wie_reagierst_du", n, was), Action::Wahl { art: format!("disziplin:{art}"), bezug: mit.map_or(vec![p], |m| vec![p, m]), optionen: opts });
            break; // vorfall_max_je_tag 1
        }
    }
    fn strafe(&mut self, p: usize, art: &str, strafe: &str) {
        let abs = self.abs();
        let w = arr("gespraeche", &format!("disziplin.wirkung.{strafe}"));
        let (st, bz, tg) = (w[0].as_f64().unwrap() as f32, w[1].as_f64().unwrap() as f32, w[2].as_f64().unwrap() as f32);
        let grund = crate::tf!("ui.mensch.strafe_nach_vorfall", strafe);
        self.anlass(p, crate::id("strafe"), 1.0);
        self.moral_wirkung(p, st, 0, &grund);
        self.beziehung(p, bz, &grund);
        let gl = f("gespraeche", "disziplin.gleichbehandlung_tage") as u32;
        let unfair = strafe != crate::id("nachsicht") && self.mensch.strafen.iter().any(|s| abs < s.0 + gl && s.2 == art && s.3 == crate::id("nachsicht") && self.pid(s.1).map_or(false, |q| self.world.players[q].ps.rolle <= 3));
        let tgd = tg - if unfair { f("gespraeche", "disziplin.gleichbehandlung_abzug") } else { 0.0 };
        self.mensch.teamgeist = (self.mensch.teamgeist + tgd).clamp(0.0, 100.0);
        let g2 = if unfair { crate::tf!("ui.mensch.eine_stammkraft_kam_fuer_dasselbe", grund) } else { grund.clone() };
        self.grund(crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("charakter.teamgeist"), tgd, &g2);
        match strafe {
            "geldstrafe" => { let m = self.world.players[p].wage / 12.0 * 0.1; self.book(crate::inhalt::t("ui.mensch.geldstrafen"), m); }
            "bank" => { self.world.players[p].susp = self.world.players[p].susp.max(1); }
            _ => {}
        }
        let id = self.world.players[p].id;
        self.mensch.strafen.push((abs, id, art.into(), strafe.into()));
        self.mensch.strafen.retain(|s| abs < s.0 + 120);
    }

    // ------------------------------------------------------------ the answer to a choice in the inbox
    pub fn wahl(&mut self, art: &str, bezug: &[usize], key: &str) -> String {
        let (kind, sub) = art.split_once(':').unwrap_or((art, ""));
        match kind {
            "koop" => return self.koop_antwort(sub.parse().unwrap_or(0), key, None),
            "testanfrage" => return self.testanfrage_antwort(sub, key),
            "lager" => return self.lager_antwort(sub, key),
            "wv" => return self.wv_antwort(sub, key),
            "vorkauf" => {
                let Ok(p) = sub.parse::<usize>() else { return crate::inhalt::t("ui.akademie.notiert").into() };
                if key != crate::id("kaufen") { for x in self.vorkauf.iter_mut().filter(|x| x.0 == p) { x.1 = 0; } return crate::inhalt::t("ui.mensch.er_geht_an_einen_anderen").into(); }
                return match self.open_neg(p) { Ok(_) => crate::inhalt::t("ui.mensch.die_verhandlung_steht_unter_verhandlunge").into(), Err(e) => e };
            }
            "pool" => {
                self.erste_woche_schritt(1);
                let p = bezug[0];
                if self.world.players[p].team != self.own as i32 { return crate::inhalt::t("ui.mensch.er_ist_nicht_mehr_bei").into(); }
                self.pool_antwort(sub, p, key)
            }
            "gespraech" => {
                self.erste_woche_schritt(1);
                let p = bezug[0];
                if self.world.players[p].team != self.own as i32 { return crate::inhalt::t("ui.mensch.er_ist_nicht_mehr_bei").into(); }
                let d = self.gespraech(p, sub, key);
                if sub == crate::id("zukunft") && key == crate::id("freigabe") && self.verkauf_gesperrt(p).is_none() { self.world.players[p].listed = true; }
                format!("{} {}", self.reaktion_text(p, d), crate::tf!("ui.mensch.wirkt", self.world.players[p].name, Self::mimik(d)))
            }
            "konflikt" => {
                let (a, b) = (bezug[0], bezug[1]);
                let (ia, ib) = (self.world.players[a].id, self.world.players[b].id);
                let Some(ki) = self.mensch.konflikte.iter().position(|k| (k.a == ia && k.b == ib) || (k.a == ib && k.b == ia)) else { return crate::inhalt::t("ui.mensch.der_streit_ist_schon_beigelegt").into() };
                let k = self.mensch.konflikte[ki].clone();
                let bonus = self.kapitaen().filter(|&c| c != a && c != b && cw(&self.world.players[c], FUEHRUNG) >= 70.0).map(|_| f("konflikte", "ausgang.kapitaen_bonus")).unwrap_or(0.0);
                let chance = f("konflikte", &format!("ausgang.{key}")) - f("konflikte", "ausgang.schwere_abzug") * k.schwere as f32 + bonus;
                let ok = zufall(&["2026", crate::id("ausgang"), &ia.to_string(), &ib.to_string(), &k.seit.to_string(), key]) < chance;
                if key == crate::id("strafe") { self.strafe(a, crate::id("streit"), crate::id("geldstrafe")); self.strafe(b, crate::id("streit"), crate::id("geldstrafe")); }
                if ok {
                    self.mensch.konflikte.remove(ki);
                    if let Some(i) = self.bez_idx(ia, ib) { self.mensch.beziehungen[i].w = self.mensch.beziehungen[i].w.max(-20.0); }
                    self.grund(crate::inhalt::t("bereich.mannschaft"), crate::inhalt::t("ui.mensch.konflikt"), 1.0, crate::inhalt::t("ui.mensch.streit_beigelegt"));
                    crate::inhalt::t("en.vermitteln.ok").into()
                } else {
                    self.mensch.konflikte[ki].schwere = (k.schwere + 1).min(3);
                    crate::inhalt::t("ui.mensch.es_brodelt_weiter_der_streit").into()
                }
            }
            "disziplin" => { self.strafe(bezug[0], sub, key); crate::inhalt::t("ui.mensch.entschieden").into() }
            "medizin" | "nachforderung" | "hijack" => self.drama_antwort(kind, bezug[0], key),
            "geruecht" => self.geruecht_antwort(bezug[0], key),
            "nachfrage" => self.nachfrage_antwort(bezug[0], key),
            "umschulen" => self.umschulung_antwort(sub, bezug[0], key),
            "privat" => self.privat_antwort(sub, key),
            "sendung" => self.sendung_antwort(key),
            "rat" => self.rat_antwort(sub, bezug, key),
            "story" => self.story_antwort(sub, bezug, key),
            "befoerderung" | "eltern" => self.akademie_antwort(kind, bezug[0], key),
            "mentor" => self.mentor_antwort(bezug[0], bezug[1], key),
            "dl" => self.dl_antwort(sub.parse().unwrap_or(usize::MAX), bezug[0], key),
            "leihe" => self.leihe_antwort(sub, bezug[0], key),
            "abschluss" => self.abschluss_antwort(bezug[0], key),
            "sichtung" => self.sichtung_antwort(key),
            "halbzeit" => self.halbzeit_antwort(sub, key),
            "mind" => self.mind_antwort(sub, key),
            "gut" => self.gut_antwort(sub, bezug[0], key),
            "boykott" => self.boykott_antwort(key),
            "vvwarn" => if key == "verlaengern" { match self.open_neg(bezug[0]) { Ok(_) => crate::inhalt::t("ui.sd.verhandlung_offen").into(), Err(e) => e } } else { crate::inhalt::t("ui.sd.abgelehnt").into() }, // roh
            "sd" => self.sd_antwort(sub, bezug[0], key),
            "bewerbung" => self.bewerbung_antwort(key),
            "jobverh" => self.job_antwort(key),
            "mv" => self.versammlung_antwort(sub, key),
            "krise" => self.krise_antwort(key),
            _ => String::new(),
        }
    }

    // ------------------------------------------------------------ club climate (§11)
    fn klima_nach_spiel(&mut self) {
        let l = self.league();
        let table = self.world.table(l);
        let Some(pos) = table.iter().position(|r| r.club == self.own) else { return };
        let spiele = table[pos].p as usize;
        let n = table.len();
        let res: Vec<char> = self.results.iter().map(|r| r.chars().next().unwrap_or('U')).collect();
        let serie = |c: char| res.iter().rev().take_while(|&&x| x == c).count();
        let ohne_sieg = res.iter().rev().take_while(|&&x| x != 'S').count();
        let pkt5: usize = res.iter().rev().take(5).map(|&x| match x { 'S' => 3, 'U' => 1, _ => 0 }).sum();
        let ziel = self.club().goal_pos;
        let abstand = (pos + 1) as i32 - ziel as i32;
        let ab = f("klima", "liga.abstiegsplaetze") as usize;
        let abstieg = pos + 1 + ab > n;
        let offen = self.world.leagues[l].fixtures.iter().filter(|x| x.hg < 0 && (x.h == self.own || x.a == self.own)).count();
        let mut summe = [0.0f32; 3]; // fans, medien, vorstand
        let mut gruende: Vec<&str> = vec![];
        for tr in arr("klima", "treiber") {
            let id = tr["id"].as_str().unwrap();
            let einmalig = tr["einmalig"].as_bool().unwrap_or(false);
            if einmalig && self.mensch.saisonziel_gebucht { continue; }
            let erfuellt = tr["bedingungen"].as_array().unwrap().iter().all(|b| {
                let v = match b["wert"].as_str().unwrap() {
                    "siege_in_folge" => serie('S') as f64, "spiele_ohne_sieg" => ohne_sieg as f64, "niederlagen_in_folge" => serie('N') as f64,
                    "punkte_letzte_fuenf" => pkt5 as f64, "vereinsspiele_saison" => spiele as f64, "tabellenplatz" => (pos + 1) as f64,
                    "abstiegsrang" => abstieg as u8 as f64, "abstand_zum_ziel" => abstand as f64, "spiele_offen" => offen as f64, _ => f64::NAN,
                };
                b["min"].as_f64().map_or(true, |m| v >= m) && b["max"].as_f64().map_or(true, |m| v <= m) && b["gleich"].as_f64().map_or(true, |m| v == m)
            });
            if !erfuellt { continue; }
            // own decision: a club whose goal already lies in the relegation zone loses no board confidence for being there
            let zone_ok = id == crate::id("abstiegszone") && ziel + ab > n;
            let w = &tr["wirkungen"];
            summe[0] += w["fanstimmung"].as_f64().unwrap_or(0.0) as f32;
            summe[1] += w["medienbild"].as_f64().unwrap_or(0.0) as f32;
            if !zone_ok { summe[2] += w["vorstandsvertrauen"].as_f64().unwrap_or(0.0) as f32; }
            if einmalig { self.mensch.saisonziel_gebucht = true; }
            gruende.push(id);
        }
        if gruende.is_empty() { return; }
        let cap = f("klima", "je_takt");
        let txt: Vec<&str> = gruende.iter().map(|g| match *g {
            "siegesserie" => crate::inhalt::t("story.regel.siegesserie"), "sieglos" => crate::inhalt::t("ui.mensch.spiele_ohne_sieg"), "niederlagen" => crate::inhalt::t("story.wert.niederlagen_in_folge"), "form_stark" => crate::inhalt::t("ui.mensch.starke_form"),
            "form_schwach" => crate::inhalt::t("ui.mensch.schwache_form"), "tabellenspitze" => crate::inhalt::t("ui.mensch.tabellenspitze"), "abstiegszone" => crate::inhalt::t("story.treiber.abstiegszone"), "ziel_erfuellt" => crate::inhalt::t("ui.mensch.saisonziel_im_plan"),
            "ziel_verfehlt" => crate::inhalt::t("ui.mensch.saisonziel_in_gefahr"), "ziel_weit_weg" => crate::inhalt::t("ui.mensch.saisonziel_weit_weg"), "saison_ziel_erreicht" => crate::inhalt::t("story.regel.saisonziel_erreicht"), _ => crate::inhalt::t("story.regel.saisonziel_verfehlt"),
        }).collect();
        self.klima(summe[0].clamp(-cap, cap), summe[1].clamp(-cap, cap), summe[2].clamp(-cap, cap), &txt.join(", "));
    }
    /// The board's season goal for the own club, refreshed after each league match (klima.json saisonziel).
    pub fn ziel_auffrischen(&mut self) {
        let z = self.vision_ziel(saisonziel(&self.world, self.own));
        let n = self.world.leagues[self.league()].teams.len();
        let c = &mut self.world.clubs[self.own];
        c.goal_pos = z;
        c.goal_kind = ziel_wort(z, n).to_string();
    }

    // ------------------------------------------------------------ board, career, dismissal (§12)
    fn ultimatum_pruefen(&mut self) {
        if self.ultimatum.is_some() || self.fired || self.abs() < self.mgr_since + f("vereinswelt", "praesidium.schonzeit_tage") as u32 { return; }
        if self.mensch.vorstand < self.ultimatum_schwelle() {
            let tage = f("vereinswelt", "praesidium.ultimatum_tage") as u32;
            self.ultimatum = Some(self.abs() + tage);
            self.ultimatum_ab = Some(self.results.len());
            let (n, ziel) = (f("vereinswelt", "praesidium.ultimatum_spiele"), f("vereinswelt", "praesidium.ultimatum_punkte"));
            self.msg(5, crate::inhalt::t("meldung.absender.praesident"), "Ultimatum", &crate::tf!("ui.ultimatum.text", format!("{ziel:.0}"), format!("{n:.0}")), Action::None); // roh
            let cn = self.club().name.clone();
            self.news(&crate::tf!("ui.mensch.eilmeldung_ultimatum_fuer_den_trainer", cn), crate::inhalt::t("ui.mensch.das_praesidium_setzt_dem_trainer"));
            self.nachfolger_geruechte();
        }
    }
    /// A request to the board (Fabian 23.11.): transfer funds, wage room or a facility grant, answered in full, half or
    /// not by confidence, form and what it already gave this season. Returns the answer's text.
    pub fn vorstand_bitte(&mut self, art: u8) -> Result<String, String> {
        let b = |k: &str| f("vereinswelt", &format!("bitten.{k}"));
        let abs = self.abs();
        if self.bitten.iter().any(|x| x.1 == art && abs < x.0 + b("sperre_tage") as u32) { return Err(crate::inhalt::t("ui.bitte.zu_frueh").into()); }
        let saison_start = abs - abs % crate::date::SEASON_DAYS;
        let gegeben: f32 = self.bitten.iter().filter(|x| x.0 >= saison_start).map(|x| x.2).sum();
        let bereit = self.mensch.vorstand / 100.0 * b("vertrauen_gewicht") + self.vereins_ppg().unwrap_or(1.3) / 3.0 * b("form_gewicht") - gegeben * b("je_bitte_abzug");
        let anteil = if bereit >= b("ganz_ab") { 1.0 } else if bereit >= b("halb_ab") { 0.5 } else { 0.0 };
        self.bitten.push((abs, art, anteil));
        if anteil == 0.0 {
            self.klima(0.0, 0.0, b("nein_vertrauen"), crate::inhalt::t("ui.bitte.titel"));
            return Ok(crate::inhalt::t("ui.bitte.nein").into());
        }
        let umsatz = self.umsatz_prognose();
        let wort = crate::inhalt::t(if anteil >= 1.0 { "ui.bitte.ganz" } else { "ui.bitte.halb" });
        Ok(match art {
            0 => { let x = crate::world::round_money(umsatz * (b("transfer_anteil") * anteil) as f64); self.transfer_budget += x; crate::tf!("ui.bitte.transfer", wort, money(x)) }
            1 => { let x = crate::world::round_money(umsatz * (b("gehalt_anteil") * anteil) as f64); self.gehalt_plus += x; crate::tf!("ui.bitte.gehalt", wort, money(x)) }
            _ => {
                self.wirtschaft_an();
                let k = (0..self.wi.stufen.len()).map(|a| self.ausbau_kosten(a)).fold(f64::MAX, f64::min);
                let x = crate::world::round_money(k * (b("ausbau_anteil") * anteil) as f64);
                self.book(crate::inhalt::t("ui.bitte.zuschuss"), x);
                self.infra_topf = Some(self.infra() + x);
                crate::tf!("ui.bitte.ausbau", wort, money(x))
            }
        })
    }
    /// Share of the monthly surplus that goes into the transfer budget (vereinswelt.toml [budget]).
    pub fn budget_anteil(&self) -> f32 {
        let b = |k: &str| f("vereinswelt", &format!("budget.{k}"));
        (if self.mensch.vorstand >= b("freigabe_ab") { b("transferanteil_freigabe") } else if self.mensch.vorstand < b("knapp_unter") { b("transferanteil_knapp") } else { b(crate::id("transferanteil")) } + self.vision_budget()).max(0.0)
    }
    /// Dismissal risk for the display, 0…1 (laufbahn/service.py:167).
    pub fn entlassungsrisiko(&self) -> f32 {
        let n = self.world.leagues[self.league()].teams.len().max(1) as f32;
        let platz = self.world.rank_of(self.own) as f32;
        let vertrag = self.mensch.vertrag_bis.saturating_sub(self.abs()) as f32 / 30.4;
        0.5 * ((50.0 - self.mensch.vorstand) / 50.0).clamp(0.0, 1.0) + 0.35 * ((platz - self.club().goal_pos as f32) / n / 0.35).clamp(0.0, 1.0) + if vertrag <= 3.0 { 0.15 } else { 0.0 }
    }
    /// Coach reputation 0…100 (laufbahn.toml [ruf]).
    pub fn trainerruf(&self) -> f32 {
        let r = |k: &str| f("laufbahn", &format!("ruf.{k}"));
        let table = self.world.table(self.league());
        let Some(pos) = table.iter().position(|x| x.club == self.own) else { return self.mgr_rep };
        let row = &table[pos];
        let quote = if row.p > 0 { row.pts as f32 / row.p as f32 / 3.0 } else { 0.5 };
        let lage = if table.len() > 1 { 1.0 - pos as f32 / (table.len() - 1) as f32 } else { 0.5 };
        let halb = if (row.p as f32) < r("min_spiele") { 0.5 } else { 1.0 };
        let monate = (self.abs().saturating_sub(self.mgr_since)) as f32 / 30.4 + self.history.len() as f32 * 12.0 + self.trainerprofil.monate();
        (r(crate::id("basis")) + halb * (r("je_punktequote") * quote + r("je_platz") * lage) + r("je_titel") * self.mensch.titel.min(4) as f32
            + (r("je_erfahrung_monat") * monate).min(r("max_erfahrung")) + r("je_vereinsruf") * self.club().rep / 100.0 + self.trainerprofil.ruf()).clamp(0.0, 100.0)
    }
    /// Months of coaching experience: this post plus a year for every earlier station (as in trainerruf).
    pub fn erfahrung_monate(&self) -> f32 { (self.abs().saturating_sub(self.mgr_since)) as f32 / 30.4 + self.history.len() as f32 * 12.0 + self.trainerprofil.monate() }
    /// laufbahn werte.py lizenz(): the highest level whose minimum reputation and experience are reached.
    pub fn lizenz(&self) -> &'static str {
        let (ruf, monate) = (self.trainerruf(), self.erfahrung_monate());
        LIZENZEN.iter().copied().filter(|s| { let a = arr("laufbahn", &format!("lizenz.{s}")); ruf >= a[0].as_f64().unwrap_or(0.0) as f32 && monate >= a[1].as_f64().unwrap_or(0.0) as f32 }).last().unwrap_or(LIZENZEN[0])
    }
    /// The highest club reputation (share of the top, 0…1) the licence is good for (lizenz_grenze; no course module yet).
    pub fn lizenz_grenze(&self) -> f32 { arr("laufbahn", &format!("lizenz.{}", self.lizenz()))[2].as_f64().unwrap_or(1.0).min(1.0) as f32 }
    /// What is missing for the next level: (level, reputation points, months), None at the top (lizenz_naechste).
    pub fn lizenz_naechste(&self) -> Option<(&'static str, f32, f32)> {
        let i = LIZENZEN.iter().position(|s| *s == self.lizenz())?;
        let n = *LIZENZEN.get(i + 1)?;
        let a = arr("laufbahn", &format!("lizenz.{n}"));
        Some((n, (a[0].as_f64().unwrap_or(0.0) as f32 - self.trainerruf()).max(0.0).round(), (a[1].as_f64().unwrap_or(0.0) as f32 - self.erfahrung_monate()).max(0.0).round()))
    }
    /// Weekly: a new licence level is announced in the inbox once (a milestone with reason).
    pub fn lizenz_pruefen(&mut self) {
        let neu = self.lizenz();
        if self.mensch.lizenz == neu { return; }
        let alt = std::mem::replace(&mut self.mensch.lizenz, neu.to_string());
        if alt.is_empty() || LIZENZEN.iter().position(|s| *s == alt) >= LIZENZEN.iter().position(|s| *s == neu) { return; }
        let n = crate::inhalt::t(&format!("lb.lizenz.{neu}"));
        let g = crate::tf!("lb.lizenz.reicht", format!("{:.0}", self.lizenz_grenze() * 100.0));
        let co = self.co_name.clone();
        self.msg(5, &co, &crate::tf!("ui.lizenz.neu_titel", n), &crate::tf!("ui.lizenz.neu_text", n, g), Action::None);
        let coach = self.coach.clone();
        self.grund(&coach, crate::id("lizenz"), 1.0, &crate::tf!("ui.lizenz.neu_titel", n));
        let own = self.own;
        self.feiern.push(crate::meilensteine::Feier { art: crate::id("lizenz").into(), titel: crate::tf!("ui.lizenz.neu_titel", n), text: crate::tf!("ui.lizenz.neu_text", n, g), spieler: None, club: Some(own), stimmen: vec![], ..Default::default() });
        self.cp_geben(crate::regeln::f("skilltree", "manager.lizenz") as u8, crate::inhalt::t("ui.mst.grund.lizenz"));
    }
    pub fn ansehen(&self) -> f32 { f("laufbahn", "reputation.ruf") * self.mgr_rep + f("laufbahn", "reputation.medien") * self.media + f("laufbahn", "reputation.vertrauen") * self.mensch.vorstand }
    /// Coach salary at a club (laufbahn.toml [gehalt]).
    pub fn trainergehalt(&self, club: usize) -> f64 {
        let g = |k: &str| f("laufbahn", &format!("gehalt.{k}")) as f64;
        let ruf = self.world.clubs[club].rep as f64 / 100.0;
        let sp = arr("laufbahn", "gehalt.trainer_spanne");
        let (a, b) = (sp[0].as_f64().unwrap(), sp[1].as_f64().unwrap());
        let v = (g(crate::id("basis")) + g("je_ruf2") * ruf * ruf) * (a + (b - a) * self.mgr_rep as f64 / 100.0);
        (v / g(crate::id("runden"))).round() * g(crate::id("runden"))
    }
    /// Weekly job offers (laufbahn.toml [interesse] [motor]).
    pub fn jobangebote_woche(&mut self) {
        let i = |k: &str| f("laufbahn", &format!("interesse.{k}"));
        let abs = self.abs();
        let frist = f("laufbahn", "motor.frist_tage") as u32;
        self.lizenz_pruefen();
        for m in self.inbox.iter_mut() { if !m.done && matches!(m.action, Action::JobOffer { .. }) && abs >= abs_day(m.season, m.day) + frist { m.done = true; } }
        // mid-December near the top with a delighted board: bigger clubs notice too
        let (_, monat, tag) = crate::date::ymd(self.world.season, self.world.day);
        if monat == 12 && (10..17).contains(&tag) && self.world.rank_of(self.own) <= 3 && self.mensch.vorstand >= i("lockruf_dezember_vorstand") && self.lockruf_bis < abs { self.lockruf_bis = abs + i("lockruf_tage") as u32; }
        let lockruf = abs < self.lockruf_bis;
        if !self.fired && abs < self.mgr_since + 120 { return; }
        let offen = self.inbox.iter().filter(|m| !m.done && matches!(m.action, Action::JobOffer { .. })).count();
        if offen as f32 >= f("laufbahn", "motor.offene_hoechstens") { return; }
        self.mensch.job_sperre.retain(|x| abs < x.1);
        let grenze = self.lizenz_grenze();
        let rt = self.mgr_rep / 100.0;
        let erfolg = self.vereins_ppg().unwrap_or(1.3) / 3.0;
        let vertrag_ende = (self.mensch.vertrag_bis.saturating_sub(abs) as f32 / 30.4) <= i("vertrag_ende_monate");
        let own_rep = self.club().rep;
        let mut kand: Vec<(usize, f32)> = (0..self.world.clubs.len()).filter(|&c| c != self.own && self.angebot_erlaubt(c) && !self.mensch.job_sperre.iter().any(|x| x.0 == c)).map(|c| {
            let cl = &self.world.clubs[c];
            let rc = cl.rep / 100.0;
            let ruf = if rc <= rt + i("ruf_luft") { (1.0 - (rt - rc).max(0.0) * 2.0).max(0.0) } else { 0.0 };
            let passung = if cl.style == self.tactic.style() { 1.0 } else { 0.5 };
            let vakanz = cl.manager == "Interimstrainer"; // roh
            let v = i("w_ruf") * ruf + i("w_passung") * passung + i("w_erfolg") * erfolg + i("w_vakanz") * vakanz as u8 as f32
                + if vakanz { i("vakanz_bonus") } else { 0.0 } + if vertrag_ende { i("vertrag_ende_bonus") } else { 0.0 }
                + if lockruf && rc > own_rep / 100.0 { i("lockruf_bonus") } else { 0.0 };
            (c, v)
        }).filter(|x| x.1 >= i("schwelle_gespraech") && self.world.clubs[x.0].rep / 100.0 <= grenze && (self.fired || self.world.clubs[x.0].rep > own_rep - 15.0)).collect();
        kand.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let woche = (abs / 7).to_string();
        for (c, v) in kand.into_iter().take(30) {
            if zufall(&["2026", crate::id("jobangebot"), &c.to_string(), &woche]) >= f("laufbahn", "motor.angebot_wahrscheinlichkeit") * v * v { continue; }
            let wage = self.trainergehalt(c);
            let n = self.world.clubs[c].name.clone();
            let own = self.club().name.clone();
            let sperre = f("laufbahn", "motor.erneut_sperre_tage") as u32;
            self.mensch.job_sperre.push((c, abs + sperre));
            let vb = self.variante("job_text", crate::inhalt::varianten("ui.mensch.hat_deine_arbeit_verfolgt_und"), &n); // roh
            let vt = self.variante("job_titel", crate::inhalt::varianten("ui.mensch.wirbt_um_dich"), &n); // roh
            let t = if self.fired { crate::tf!("ui.mensch.bietet_dir_den_trainerposten_an", n.clone(), money(wage), frist) } else { crate::tf!(&crate::inhalt::variante_key("ui.mensch.hat_deine_arbeit_verfolgt_und", vb), n.clone(), money(wage), own, frist) };
            self.msg(5, &n, &crate::tf!(&crate::inhalt::variante_key("ui.mensch.wirbt_um_dich", vt), n.clone()), &t, Action::JobOffer { club: c, wage });
            if zufall(&["2026", crate::id("leck"), &c.to_string(), &woche]) < f("laufbahn", "motor.leck_wahrscheinlichkeit") {
                let coach = self.coach.clone();
                self.news(&crate::tf!("ui.mensch.geruecht_will", n, coach), crate::inhalt::t("ui.mensch.hinter_den_kulissen_soll_es"));
            }
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn karriere() -> Career {
        let w = crate::world::testwelt(7);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(0);
        Career::new(w, own, "Test")
    }
    /// Fabian 24.11.: the same topic in another tone by mood and character, and a reaction in his own way.
    #[test]
    fn ton_nach_charakter() {
        let mut c = karriere();
        let p = c.squad()[2];
        c.world.players[p].ps.an = true;
        c.world.players[p].ps.ch = vec![50.0; 10];
        c.world.players[p].ps.sh = vec![];
        c.world.players[p].morale = 70.0;
        assert_eq!(c.haltung_ton(p), "hoeflich");
        c.world.players[p].ps.ch[TEMPERAMENT] = 80.0;
        c.world.players[p].morale = 30.0;
        assert_eq!(c.haltung_ton(p), "wuetend");
        let r = c.reaktion_text(p, -4.0);
        assert!(r.contains(&c.world.players[p].name) && !r.contains("%1"));
        c.world.players[p].ps.ch[TEMPERAMENT] = 50.0;
        c.world.players[p].ps.ch[EGO] = 80.0;
        assert_eq!(c.haltung_ton(p), "fordernd");
    }

    #[test]
    fn lizenz_nach_ruf_und_erfahrung() {
        let mut c = karriere();
        // a new coach: little experience, so at most the B licence; the limit comes from laufbahn.toml
        let l = c.lizenz();
        assert!(l == "c" || l == "b", "{l}");
        let (n, _, monate) = c.lizenz_naechste().unwrap();
        assert!(n != l && monate > 0.0);
        // three earlier stations = 36 months: with enough reputation the professional licence
        c.history = vec!["a".into(), "b".into(), "c".into()];
        c.mensch.titel = 4;
        if c.trainerruf() >= 65.0 { assert_eq!(c.lizenz(), "pro"); assert!(c.lizenz_naechste().is_none()); }
        // job offers only from clubs the licence is good for
        c.history.clear();
        c.mensch.titel = 0;
        c.fired = true;
        for _ in 0..30 { c.world.day += 7; c.jobangebote_woche(); }
        let g = c.lizenz_grenze(); // the licence only grows, so the last limit is the highest
        for m in c.inbox.iter() { if let Action::JobOffer { club, .. } = m.action { assert!(c.world.clubs[club].rep / 100.0 <= g + 1e-6, "Angebot über der Lizenz"); } }
        // a higher level is announced once
        c.mensch.lizenz = "c".into();
        let vorher = c.inbox.len();
        c.history = vec!["a".into(); 3];
        c.lizenz_pruefen();
        if c.lizenz() != "c" { assert_eq!(c.inbox.len(), vorher + 1); c.lizenz_pruefen(); assert_eq!(c.inbox.len(), vorher + 1); }
    }

    #[test]
    fn vorfaelle_selten_und_abwechslungsreich() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 1, "Test");
        c.psyche_an();
        for p in c.squad() { c.world.players[p].ps.ch[PROF] = 10.0; } // worst case: a squad of slackers
        let mut arten = vec![];
        for _ in 0..365 {
            c.world.day += 1;
            c.mensch.anliegen.clear();
            let vorher = c.vorfall_zuletzt.0;
            c.disziplin_tag();
            if c.vorfall_zuletzt.0 != vorher { arten.push(c.vorfall_zuletzt.1.clone()); }
        }
        let ruhe = f("gespraeche", "disziplin.verein_ruhe_tage") as usize;
        assert!(arten.len() <= 365 / ruhe + 1, "{} incidents in a year", arten.len());
        assert!(arten.len() >= 3, "slackers still cause incidents");
        assert!(arten.windows(2).all(|w| w[0] != w[1]), "same kind twice in a row: {arten:?}");
    }

    #[test]
    fn charakter_fest_und_typen() {
        let a = charakter(4711, 31, 80.0);
        assert_eq!(a, charakter(4711, 31, 80.0), "same player, same values");
        assert!((a[STATUS] - 80.0).abs() < 1e-3, "status is the percentile, no random part");
        assert!(a.iter().all(|v| (0.0..=100.0).contains(v)));
        let mut c = karriere();
        c.psyche_an();
        let sq = c.squad();
        assert!(sq.iter().any(|&p| !typen(&c.world.players[p]).is_empty()), "some players get a type");
        assert_eq!(sq.iter().filter(|&&p| c.world.players[p].ps.rolle == 0).count(), 1, "one captain");
    }

    #[test]
    fn ton_und_vertrauen() {
        let p = vec![50.0; 10];
        assert!(ton_aenderung("warm", &p, 50.0, 50.0) > 0.0);
        assert!(ton_aenderung("hart", &p, 50.0, 50.0) < 0.0);
        assert!(ton_aenderung("warm", &p, 50.0, 0.0) < ton_aenderung("warm", &p, 50.0, 50.0), "low trust damps kind words");
        assert_eq!(pfeil(0.0), 2);
        assert_eq!(pfeil(4.0), 4);
        assert_eq!(pfeil(-4.0), 0);
    }

    #[test]
    fn gebrochenes_versprechen_kostet_vertrauen() {
        let mut c = karriere();
        c.psyche_an();
        let p = *c.squad().last().unwrap();
        c.world.players[p].ps.einsatz = vec![0.0; 5];
        let v0 = c.vertrauen(p);
        c.gespraech(p, "spielzeit", "zusage");
        assert_eq!(c.mensch.versprechen.len(), 1);
        for _ in 0..30 { c.psyche_tag(); c.world.day += 1; }
        assert_eq!(c.mensch.versprechen[0].stand, 2, "he did not play: broken at the deadline");
        assert!(c.vertrauen(p) < v0 - 10.0, "trust falls: {} → {}", v0, c.vertrauen(p));
        assert!(c.mensch.gruende.iter().any(|g| g.grund.contains("Zusage gebrochen")));
    }

    #[test]
    fn moral_laeuft_zum_ziel() {
        let mut c = karriere();
        c.psyche_an();
        let p = c.squad()[0];
        c.world.players[p].morale = 90.0;
        for _ in 0..60 { c.psyche_tag(); c.world.day += 1; }
        let pl = &c.world.players[p];
        let ziel = (50.0 + pl.ps.treiber.iter().map(|x| x.1).sum::<f32>()).clamp(5.0, 95.0);
        assert!((pl.morale - ziel).abs() < 3.0, "morale {} target {}", pl.morale, ziel);
    }

    #[test]
    fn saisonziel_staerkster_kader_platz_zwei() {
        let c = karriere();
        let l = c.league();
        let best = *c.world.leagues[l].teams.iter().max_by(|&&a, &&b| kaderstaerke(&c.world, a).partial_cmp(&kaderstaerke(&c.world, b)).unwrap()).unwrap();
        let z = saisonziel(&c.world, best);
        let mut st: Vec<f32> = c.world.leagues[l].teams.iter().map(|&t| kaderstaerke(&c.world, t)).collect();
        st.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert_eq!(z == 1, st[0] - st[1] >= 4.0);
        assert!(z <= 2);
    }

    #[test]
    fn saison_mit_allen_systemen() {
        let mut c = karriere();
        let k0 = c.club().balance;
        let mut moral = vec![];
        for _ in 0..crate::date::SEASON_DAYS {
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            // answer every open choice with its first option, like a person who reads the inbox
            for i in 0..c.inbox.len() { if !c.inbox[i].done && matches!(c.inbox[i].action, Action::Wahl { .. }) { c.answer(i, true, 0); } }
            c.ultimatum = None;
            c.end_day().unwrap();
            if c.world.day % 30 == 0 { let sq = c.squad(); moral.push(sq.iter().map(|&p| c.world.players[p].morale).sum::<f32>() / sq.len() as f32); }
        }
        let m = moral.iter().sum::<f32>() / moral.len() as f32;
        // 23.11.: stories are told once per player now (no repeats), so a season brings fewer feel-good answers; the
        // lower bound went from 38 to 35. shortcut: balance of the morale drivers, re-tune with Fabian's fine-tuning
        assert!((35.0..=62.0).contains(&m), "mean morale {m}");
        assert!((25.0..=75.0).contains(&c.mensch.teamgeist), "team spirit {}", c.mensch.teamgeist);
        assert!(c.mensch.gruende.len() > 50, "the season leaves reasons");
        let k1 = c.club().balance;
        eprintln!("Kasse {} → {}, Umsatz {}, Gehälter {}", money(k0), money(k1), money(c.umsatz_prognose()), money(c.world.wage_bill(c.own)));
        assert!(k1 > k0 - c.umsatz_prognose(), "the club does not burn more than a year's income in one season");
    }

    #[test]
    fn jede_wirkung_hat_grund() {
        // guard (Grundsatz 1): fans, media and board confidence are only written here, through klima()
        let seiten: Vec<(String, String)> = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src/pages")).unwrap().map(|e| e.unwrap().path()).map(|p| (format!("pages/{}", p.file_name().unwrap().to_string_lossy()), std::fs::read_to_string(&p).unwrap())).collect();
        for (name, src) in [("game.rs", include_str!("game.rs")), ("world.rs", include_str!("world.rs"))].into_iter().chain(seiten.iter().map(|(n, s)| (n.as_str(), s.as_str()))) {
            for pat in ["self.fans =", "self.media =", "c.fans =", "c.media =", ".vorstand ="] {
                assert!(!src.contains(pat), "{name} writes `{pat}` without a reason");
            }
        }
    }
}
