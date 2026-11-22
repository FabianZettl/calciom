//! Permanent development per ability for the own squad and academy (Mechanik-Übergabe §18; Python core/entwicklung:
//! phasen.py, wochenlauf.py, wachstum.py, regeln.py; numbers in entwicklung2.toml, entwicklung3.toml, kalibrierung.json,
//! merkmale.json, own choices in entwicklung_pesm.toml). Weekly points per ability, fractions are carried, whole steps
//! are booked, the overall follows from the abilities. The other clubs keep World::develop_week.
use crate::game::{Action, Career};
use crate::mensch::{arr, zufall, ROLLEN};
use crate::regeln::{f, wert};
use crate::world::Player;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Python names of the 25 abilities, in pesdb::ATTRS order (the keys of the rule files and of the texts `faehigkeit.*`).
pub const NAMEN: [&str; 25] = [crate::id("offensivgeist"), crate::id("ballkontrolle"), crate::id("dribbling"), "enge_ballfuehrung", crate::id("flachpass"), crate::id("steilpass"), crate::id("abschluss"), crate::id("kopfball"), crate::id("freistoss"), crate::id("effet"),
    crate::id("schnelligkeit"), crate::id("antritt"), crate::id("schusskraft"), crate::id("sprungkraft"), crate::id("koerperkontakt"), crate::id("balance"), crate::id("ausdauer"), crate::id("defensivgeist"), crate::id("balleroberung"), crate::id("aggressivitaet"),
    crate::id("torwartgefuehl"), crate::id("fangen"), crate::id("klaeren"), crate::id("reflexe"), crate::id("reichweite")];
/// Groups of the age curves (felder.GRUPPE).
pub const GRUPPEN: [&str; 5] = [crate::id("tempo"), crate::id("kraft"), crate::id("technik"), crate::id("kopf"), crate::id("torwart")];
pub const GRUPPE: [usize; 25] = [3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 2, 1, 1, 1, 1, 3, 3, 3, 4, 4, 4, 4, 4];
const KOERPERLICH: [usize; 6] = [10, 11, 13, 14, 15, 16];
/// Python position codes in pesdb::POS order.
pub const PY_POS: [&str; 13] = [crate::id("TW"), crate::id("IV"), crate::id("LV"), crate::id("RV"), crate::id("DM"), crate::id("ZM"), crate::id("LM"), crate::id("RM"), crate::id("OM"), crate::id("LA"), crate::id("RA"), crate::id("HS"), crate::id("ST")];
/// Phases: 0 stagnation, 1 upswing, 2 slump, 3 crisis (1–3 are limited in time), 4 maturing, 5 ageing.
pub const PHASEN: [&str; 6] = [crate::id("stagnation"), crate::id("aufschwung"), crate::id("formtief"), crate::id("krise"), crate::id("reifung"), crate::id("alterung")];
const BEREICHE: [&str; 4] = [crate::id("tor"), crate::id("abwehr"), crate::id("mitte"), crate::id("angriff")];

fn k3(p: &str) -> f32 { f("entwicklung3", p) }
fn k2(p: &str) -> f32 { f("entwicklung2", p) }
fn kp(p: &str) -> f32 { f("entwicklung_pesm", p) }
fn klemme(x: f32, a: f32, b: f32) -> f32 { x.max(a).min(b) }
fn num(v: &Value) -> f32 { v.as_f64().unwrap_or(0.0) as f32 }
/// Linear between the points of a table [[x, y], …] (regeln._linear).
fn linear(t: &[Value], x: f32) -> f32 {
    let p = |i: usize| (num(&t[i][0]), num(&t[i][1]));
    if x <= p(0).0 { return p(0).1; }
    for i in 1..t.len() { let ((x0, y0), (x1, y1)) = (p(i - 1), p(i)); if x <= x1 { return y0 + (y1 - y0) * (x - x0) / (x1 - x0); } }
    p(t.len() - 1).1
}
pub fn bereich(pos: usize) -> usize { match pos { 0 => 0, 1..=3 => 1, 4..=8 => 2, _ => 3 } }

/// The development state of a player (serialised with the career).
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Ent {
    /// The model runs for him (abilities are whole numbers, fractions are in `rest`).
    pub an: bool,
    pub rest: Vec<f32>,
    /// Loss of a slump or crisis still to come back, and the fleeting part of an upswing still to go (per ability).
    pub abzug: Vec<f32>,
    pub peak: Vec<f32>,
    pub phase: u8,
    pub seit: u32,
    pub bis: u32,
    pub dauer: u32,
    pub tiefe: f32,
    pub staerke: f32,
    pub sperre_bis: u32,
    pub letzter_sprung: Option<u32>,
    pub gruende: Vec<(String, f32)>,
    /// Occasions (art, abs day, strength): refused contract, broken promise, praise, criticism, penalty.
    pub anlaesse: Vec<(String, u32, f32)>,
    pub moral_vorher: f32,
    /// Focus group of the individual programme (1-based index into entwicklung.toml fokusgruppen, 0 = from the position).
    pub fokus: u8,
    pub merkmale: Vec<String>,
    pub verlauf: Vec<Woche>,
    /// Days out with the current injury, day the rehab factor ends.
    pub ausfall: u16,
    pub rueckkehr_bis: u32,
    /// Progress paths: (id, hits 0…3, abs day reached), and since when a reached path has no need in the position.
    pub wege: Vec<(String, u8, u32)>,
    pub ohne_bedarf: Vec<(String, u32)>,
    /// U23 and academy: play share and grade from the academy (akademie.rs).
    pub u23: Option<(f32, f32)>,
}
/// One week of the history: the change per group (mean points per ability), its reasons and the phase.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Woche { pub tag: u32, pub gruppen: [f32; 5], pub gruende: Vec<String>, pub phase: u8 }

/// Everything the week of one player depends on (phasen.py `e`).
#[derive(Clone, Debug)]
pub struct Lage {
    pub alter: u8,
    pub pos: usize,
    pub pot: f32,
    pub gw: f32,
    pub merkmale: Vec<String>,
    /// Share of the last matches from the start; None without matches.
    pub quote: Option<f32>,
    /// Grades, newest first.
    pub noten: Vec<f32>,
    pub serie_siege: u32,
    pub serie_niederlagen: u32,
    pub moral: f32,
    pub moral_vorher: f32,
    pub rolle: String,
    pub fokus: Vec<usize>,
    /// Intensity factor of the plan (0,8 / 1,0 / 1,2) times its repetition factor.
    pub training: f32,
    pub einzel: bool,
    pub mentor: f32,
    pub mentor_werte: Option<Vec<f32>>,
    pub beziehung: f32,
    pub stab: Option<f32>,
    pub belastung: f32,
    pub gespraeche: f32,
    pub vertrag: f32,
    pub teamgeist: Option<f32>,
    pub rueckkehr: bool,
    /// Facilities of this build (≥ 1 favourable), counted with the favourable circumstances.
    pub anlagen: f32,
    /// Straight factor on the growth rate (academy: rate_faktor × akademie_faktor of akademie.toml [entwicklung_plus]).
    pub rate: f32,
    pub seed: String,
    pub heute: u32,
}
impl Lage {
    /// A plain week: no matches, neutral mood, no focus (tests and previews start from here).
    pub fn neutral(alter: u8, pos: usize, pot: f32, gw: f32) -> Lage {
        Lage { alter, pos, pot, gw, merkmale: vec![], quote: None, noten: vec![], serie_siege: 0, serie_niederlagen: 0, moral: 50.0, moral_vorher: 50.0, rolle: "stammspieler".into(),
            fokus: vec![], training: 1.0, einzel: false, mentor: 0.0, mentor_werte: None, beziehung: 50.0, stab: None, belastung: 30.0, gespraeche: 0.0, vertrag: 0.0, teamgeist: None,
            rueckkehr: false, anlagen: 1.0, rate: 1.0, seed: "x".into(), heute: 0 }
    }
}

// ---------------------------------------------------------------- the rates (phasen.py)
fn gruppe_tabelle(g: usize) -> &'static serde_json::Map<String, Value> { wert("kalibrierung", "gruppen")[GRUPPEN[g]].as_object().unwrap() }
fn nah(t: &serde_json::Map<String, Value>, alter: f32, nur_rate: bool) -> Option<&Value> {
    t.iter().filter(|(_, v)| !nur_rate || !v["rate"].is_null())
        .min_by(|a, b| (a.0.parse::<f32>().unwrap() - alter).abs().partial_cmp(&(b.0.parse::<f32>().unwrap() - alter).abs()).unwrap()).map(|x| x.1)
}
/// Ability points per year an average player of this age gains in the group of the ability (negative: loses).
pub fn bevoelkerungsrate(i: usize, alter: u8) -> f32 {
    let t = gruppe_tabelle(GRUPPE[i]);
    let e = nah(t, alter as f32, false).unwrap();
    if !e["rate"].is_null() { return num(&e["rate"]); }
    nah(t, alter as f32, true).map(|e| num(&e["rate"])).unwrap_or(0.0)
}
/// Cap of an ability: the 98 % value of its group at `cap_alter` plus `cap_zuschlag`.
pub fn obergrenze(i: usize) -> f32 {
    let e = nah(gruppe_tabelle(GRUPPE[i]), k3("talent.cap_alter"), false).unwrap();
    (num(&e["p98"]).max(1.0) + k3("talent.cap_zuschlag")).min(99.0)
}
/// Product of the trait factors for `key` (1 without), 0,6 … 1,6.
pub fn effekt(ids: &[String], key: &str) -> f32 {
    let m = wert("entwicklung3", "merkmal");
    klemme(ids.iter().map(|i| m.get(i).and_then(|x| x.get(key)).map(num).unwrap_or(1.0)).product(), 0.6, 1.6)
}
pub fn talent_faktor(pot: f32, gw: f32, alter: u8) -> f32 {
    if alter >= 28 { return 1.0; }
    let erwartet = (k3("talent.reif_alter") - alter as f32).max(0.0) * k3("talent.erwartet_je_jahr");
    klemme(1.0 + k3("talent.steigung") * (pot - gw - erwartet), k3("talent.untergrenze"), k3("talent.obergrenze"))
}
/// Main abilities of the position ×1,3, hardly used ones ×0,55 (wachstum.positionsfaktor).
pub fn positionsfaktor(i: usize, pos: usize) -> f32 {
    let b = bereich(pos);
    let haupt = arr("entwicklung2", &format!("hauptfaehigkeiten.{}", BEREICHE[b]));
    let tw = GRUPPE[i] == 4;
    if haupt.iter().any(|h| h.as_str() == Some(NAMEN[i])) { k2("wachstum.hauptfaehigkeit_faktor") }
    else if (tw && b != 0) || (b == 0 && !tw && ![13, 15, 14].contains(&i)) { k2("wachstum.nebenfaehigkeit_faktor") }
    else { 1.0 }
}
fn anlass_gewicht(art: &str, alter_tage: u32, staerke: f32) -> f32 {
    let h = k3("woche.anlass_halbwertszeit_wochen") * 7.0;
    let neg = wert("entwicklung_pesm", "anlass.negativ").get(art).map(num);
    let (vz, basis) = match neg { Some(b) => (-1.0, b), None => (1.0, wert("entwicklung_pesm", "anlass.positiv").get(art).map(num).unwrap_or(0.0)) };
    vz * basis * staerke * 0.5f32.powf(alter_tage as f32 / h)
}
fn basisphase(l: &Lage) -> u8 {
    let ab = f("entwicklung3", &format!("woche.alterung_ab.{}", BEREICHE[bereich(l.pos)]));
    let q = k3("woche.reifung_quote");
    if l.alter as f32 >= ab { 5 }
    else if l.alter as f32 <= k3("woche.reifung_bis") && l.quote.is_some_and(|x| x >= if l.alter > 20 { q } else { q / 2.0 }) { 4 }
    else { 0 }
}
fn spanne(p: &str, key: &str) -> (f32, f32) { let a = arr("entwicklung3", &format!("phase.{p}.{key}")); (num(&a[0]), num(&a[1])) }

/// A new phase of this week (or the current one).
#[derive(Clone, Debug)]
pub struct Neu { pub phase: u8, pub dauer: u32, pub staerke: f32, pub tiefe: f32, pub gruende: Vec<(String, f32)>, pub neu: bool, pub sperre_wochen: u32, pub verbraucht: bool }

pub fn phase_bestimmen(l: &Lage, e: &Ent) -> Neu {
    let w = |k: &str| f("entwicklung3", &format!("woche.{k}"));
    let ids = &l.merkmale;
    let n: Vec<f32> = l.noten.iter().take(w("noten_fenster") as usize).copied().collect();
    let mittel = (n.len() >= 3).then(|| n.iter().sum::<f32>() / n.len() as f32);
    let gew: Vec<(String, f32)> = e.anlaesse.iter().filter(|a| l.heute.saturating_sub(a.1) as f32 <= w("anlass_fenster_tage"))
        .map(|a| (a.0.clone(), anlass_gewicht(&a.0, l.heute.saturating_sub(a.1), a.2))).collect();
    let neg = -gew.iter().filter(|g| g.1 < 0.0).map(|g| g.1).sum::<f32>();
    let pos = gew.iter().filter(|g| g.1 > 0.0).map(|g| g.1).sum::<f32>();
    let gesperrt = e.sperre_bis > l.heute;
    let (mo, mo0) = (l.moral, l.moral_vorher);
    let q = l.quote.unwrap_or(0.0);
    let ehrgeizig = ids.iter().any(|x| x == "ehrgeizig");
    let wochen = l.heute.saturating_sub(e.seit) / 7;
    let krise = neg >= w("schwelle_krise") || (!gesperrt && ((mo < w("stimmung_krise") && mo0 < w("stimmung_krise"))
        || (e.phase == 2 && wochen as f32 >= w("krise_nach_wochen") && mo < w("stimmung_tief"))));
    let formtief = neg >= w("schwelle_negativ") || (!gesperrt && (mittel.is_some_and(|m| m < w("noten_schlecht")) || (l.serie_niederlagen as f32 >= w("serie_niederlagen") && q >= 0.5)
        || (mo < w("stimmung_tief") && mo0 < w("stimmung_tief")) || (l.quote.is_some_and(|x| x < 0.2) && mo < 45.0 && l.alter <= 27 && ehrgeizig)));
    let aufschwung = !gesperrt && ((pos >= 0.5 && mo >= 55.0) || (mittel.is_some_and(|m| m >= w("noten_gut")) && q >= 0.5) || (l.serie_siege as f32 >= w("serie_siege") && q >= 0.6));
    let (kand, staerke) = if krise { (3, (neg / 1.6).max(1.0)) } else if formtief { (2, if neg > 0.0 { neg.max(0.6) } else { 1.0 }) } else if aufschwung { (1, pos.max(1.0)) } else { (0, 1.0) };
    let mut gruende: Vec<(String, f32)> = vec![];
    if kand > 0 {
        let mut g = gew.clone();
        g.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        for (art, x) in g { if x != 0.0 && (x < 0.0) == (kand >= 2) { gruende.push((art, (x * 100.0).round() / 100.0)); } }
        if kand >= 2 {
            if let Some(m) = mittel.filter(|m| *m < w("noten_schlecht")) { gruende.push(("schwache_leistungen".into(), m - w("noten_schlecht"))); }
            if l.serie_niederlagen as f32 >= w("serie_niederlagen") { gruende.push((crate::id("niederlagenserie").into(), -(l.serie_niederlagen as f32))); }
            if mo < w("stimmung_tief") { gruende.push(("geringe_stimmung".into(), mo - 50.0)); }
            if l.quote.is_some_and(|x| x < 0.2) && ehrgeizig { gruende.push(("kaum_einsaetze".into(), -1.0)); }
        } else {
            if let Some(m) = mittel.filter(|m| *m >= w("noten_gut")) { gruende.push(("gute_leistungen".into(), m - w("note_neutral"))); }
            if l.serie_siege as f32 >= w("serie_siege") { gruende.push((crate::id("siegesserie").into(), l.serie_siege as f32)); }
        }
    }
    // a running limited phase stays, unless a stronger one comes
    if (1..=3).contains(&e.phase) && l.heute < e.bis && (kand == 0 || kand <= e.phase) {
        return Neu { phase: e.phase, dauer: e.dauer, staerke: e.staerke, tiefe: e.tiefe, gruende: e.gruende.clone(), neu: false, sperre_wochen: 0, verbraucht: false };
    }
    if kand > 0 {
        let p = PHASEN[kand as usize];
        let wo = (l.heute / 7).to_string();
        let z = |k: &str| zufall(&[&l.seed, p, &wo, k]);
        let (lo, hi) = spanne(p, crate::id("dauer"));
        let dauer = ((lo + z(crate::id("dauer")) * (hi - lo)) * effekt(ids, crate::id("dauer")) * klemme(0.85 + 0.15 * staerke, 0.85, 1.3)).round().max(2.0) as u32;
        let (lo, hi) = spanne(p, crate::id("punkte"));
        let mut t = lo + z(crate::id("tiefe")) * (hi - lo);
        if kand >= 2 {
            let erster = gruende.first().map(|g| g.0.as_str()).unwrap_or("");
            let bed = if arr("entwicklung3", "bedeutung.anlaesse").iter().any(|a| a.as_str() == Some(erster)) { wert("entwicklung3", "bedeutung.rolle").get(&l.rolle).map(num).unwrap_or(1.0) } else { 1.0 };
            t *= effekt(ids, crate::id("tiefe")) * klemme(0.7 + 0.3 * staerke, 0.7, 1.5) * bed;
        } else { t *= effekt(ids, crate::id("sprung")).sqrt(); }
        gruende.truncate(4);
        return Neu { phase: kand, dauer, staerke: (staerke * 100.0).round() / 100.0, tiefe: (t * 100.0).round() / 100.0, gruende, neu: true, sperre_wochen: w("sperre_wochen") as u32, verbraucht: true };
    }
    let b = basisphase(l);
    let gruende = match b { 4 => { let mut g = vec![(crate::id("jugend").to_string(), 1.0)]; if q >= 0.5 { g.push(("viele_einsaetze".into(), 1.0)); } g } 5 => vec![("alter".into(), -1.0)], _ => vec![] };
    Neu { phase: b, dauer: 0, staerke: 1.0, tiefe: 0.0, gruende, neu: e.phase != b, sperre_wochen: 0, verbraucht: false }
}

/// Many favourable circumstances don't add up unchecked: above 1 they count half, at most `faktor_hoechstens`.
fn verdichten(x: f32) -> f32 { k3("einsatz.faktor_hoechstens").min(if x <= 1.0 { x } else { 1.0 + (x - 1.0) * 0.5 }) }
/// Extra decline per year for the group at this age (≥ 0); goalkeepers count `torwart_versatz` years younger.
pub fn alterungszuschlag(g: usize, alter: u8) -> f32 {
    let x = alter as f32 - if g == 4 { k3("alterung.torwart_versatz") } else { 0.0 };
    linear(arr("entwicklung3", "alterung.kurve"), x).max(0.0) * wert("entwicklung3", "alterung.gewicht").get(GRUPPEN[g]).map(num).unwrap_or(0.0)
}

/// The favourable and unfavourable factors of a week (cause, factor): the reasons the profile shows.
pub fn faktoren(l: &Lage, phase: u8) -> Vec<(&'static str, f32)> {
    let ei = |k: &str| f("entwicklung3", &format!("einsatz.{k}"));
    let w2 = |k: &str| f("entwicklung2", &format!("wachstum.{k}"));
    let mut v = vec![];
    if let Some(q) = l.quote {
        v.push((crate::id("spielzeit"), ei("faktor_unten") + (ei("faktor_oben") - ei("faktor_unten")) * klemme(q, 0.0, 1.0)));
        let n: Vec<f32> = l.noten.iter().take(k3("woche.noten_fenster") as usize).copied().collect();
        if n.len() >= 3 {
            let m = n.iter().sum::<f32>() / n.len() as f32;
            if m >= k3("woche.noten_gut") - 0.2 { v.push((crate::id("noten"), ei("note_gut_plus"))); } else if m <= k3("woche.noten_schlecht") + 0.2 { v.push((crate::id("noten"), ei("note_schlecht_minus"))); }
        }
    }
    if l.training != 1.0 { v.push((crate::id("training"), l.training)); }
    if l.einzel { v.push((crate::id("einzeltraining"), ei(crate::id("einzeltraining")))); }
    if let Some(st) = l.stab { if (st - 50.0).abs() >= 5.0 { v.push((crate::id("trainer"), 1.0 + ei("trainer_hebel") * (st - 50.0) / 50.0)); } }
    if l.mentor > 0.0 { v.push((crate::id("mentor"), 1.0 + ei(crate::id("mentor")) * l.mentor)); }
    if l.moral < ei("moral_niedrig") { v.push((crate::id("moral"), ei("moral_faktor_niedrig"))); } else if l.moral > ei("moral_hoch") { v.push((crate::id("moral"), ei("moral_faktor_hoch"))); }
    let wm = effekt(&l.merkmale, crate::id("wachstum")) * if l.alter <= 20 { effekt(&l.merkmale, crate::id("jung")) } else { 1.0 } * if l.alter >= 24 { effekt(&l.merkmale, crate::id("reif")) } else { 1.0 };
    if wm != 1.0 { v.push((crate::id("merkmale"), wm)); }
    let g = klemme(l.gespraeche, -w2("gespraech_deckel"), w2("gespraech_deckel"));
    if g.abs() >= 1.0 { v.push((crate::id("gespraeche"), 1.0 + g * w2("gespraech_je_punkt"))); }
    if l.vertrag > 0.0 { v.push((crate::id("vertrag"), 1.0 - w2("vertrag_abzug_deckel").min(l.vertrag * w2("vertrag_abzug_je_punkt")))); }
    if let Some(tg) = l.teamgeist { if (tg - 50.0).abs() >= 5.0 { v.push((crate::id("teamgeist"), 1.0 + (tg - 50.0) * w2("kabine_je_punkt"))); } }
    if l.rueckkehr { v.push((crate::id("rueckkehr"), w2("rueckkehr_faktor"))); }
    if l.anlagen != 1.0 { v.push((crate::id("anlagen"), l.anlagen)); }
    let ph = f("entwicklung3", &format!("phase.wachstum.{}", PHASEN[phase as usize]));
    if ph != 1.0 { v.push((PHASEN[phase as usize], ph)); }
    v
}

/// Points of this week per ability; `abzug` and `peak` carry the fleeting part of a phase (phasen.wochenpunkte, with the
/// factors of wachstum.rate for talks, contract, team spirit and the return after an injury).
pub fn wochenpunkte(l: &Lage, ph: &Neu, werte: &[f32], abzug: &mut [f32], peak: &mut [f32]) -> [f32; 25] {
    let ei = |k: &str| f("entwicklung3", &format!("einsatz.{k}"));
    let p = ph.phase;
    let tal = talent_faktor(l.pot, l.gw, l.alter);
    let gunst: f32 = faktoren(l, p).iter().filter(|x| x.0 != PHASEN[p as usize]).map(|x| x.1).product();
    let alt_m = effekt(&l.merkmale, crate::id("altern")) * if l.belastung >= ei("belastung_ab") { ei("belastung_altern") } else { 1.0 };
    let pers = |s: &str, g: usize| wert("entwicklung3", &format!("persistenz.{s}")).get(GRUPPEN[g]).map(num).unwrap_or(0.0);
    let erholung = if p == 2 || p == 3 { 0.0 } else { k3("phase.erholung_je_woche") * effekt(&l.merkmale, crate::id("erholung")) };
    let laufend = (1..=3).contains(&p);
    let dauer = ph.dauer.max(1) as f32;
    let wachstum = f("entwicklung3", &format!("phase.wachstum.{}", PHASEN[p as usize]));
    let altern = f("entwicklung3", &format!("phase.altern.{}", PHASEN[p as usize]));
    let gruppe = |s: &str, g: usize| wert("entwicklung3", &format!("phase.{s}")).get(GRUPPEN[g]).map(num).unwrap_or(0.5);
    let mut out = [0.0f32; 25];
    let gewicht = crate::pesdb::wvec(l.pos as u8);
    for i in 0..25 {
        let g = GRUPPE[i];
        // own decision E-E1: an ability the FL27 overall model weighs negatively for the position (a field player's goalkeeping,
        // a goalkeeper's play abilities) only stands for a constant there; it does not develop, or the overall would move the wrong way
        if gewicht[i] < 0.0 { continue; }
        let rate = bevoelkerungsrate(i, l.alter) * k3("woche.grundrate_anteil") / 52.0 * if bevoelkerungsrate(i, l.alter) > 0.0 { l.rate } else { 1.0 };
        let mut pt = 0.0;
        if rate > 0.0 {
            let pf = positionsfaktor(i, l.pos);
            let fokf = if l.fokus.is_empty() { 1.0 } else if l.fokus.contains(&i) { ei("fokus_plus") } else { ei("fokus_andere") };
            let cap = obergrenze(i);
            let cf = if werte[i] <= cap - k3("talent.cap_abstand") { 1.0 } else { klemme((cap - werte[i]) / k3("talent.cap_abstand"), 0.0, 1.0) };
            pt = rate * tal * verdichten(gunst * wachstum * pf * fokf) * cf;
        } else if rate < 0.0 {
            pt = rate * altern * alt_m;
        }
        pt -= alterungszuschlag(g, l.alter) / 52.0 * alt_m;
        if laufend && p >= 2 {
            let hoechstens = f("entwicklung3", &format!("phase.abzug_hoechstens.{}", PHASEN[p as usize]));
            let verlust = (ph.tiefe * gruppe("verlust_gruppe", g) / dauer).min(hoechstens - abzug[i]).max(0.0);
            pt -= verlust;
            abzug[i] += verlust * (1.0 - pers(crate::id("verlust"), g));
        } else if laufend && p == 1 {
            let gewinn = ph.tiefe * gruppe("gewinn_gruppe", g) * positionsfaktor(i, l.pos).min(1.5) / dauer;
            pt += gewinn;
            peak[i] += gewinn * (1.0 - pers(crate::id("gewinn"), g));
        }
        if peak[i] > 0.0 {
            let mut ruck = if laufend && p == 1 { 0.0 } else { peak[i] * (1.0 - 0.5f32.powf(1.0 / k3("persistenz.peak_halbwertszeit_wochen"))) };
            if peak[i] - ruck < 0.05 { ruck = peak[i]; }
            pt -= ruck;
            peak[i] -= ruck;
        }
        if erholung > 0.0 && abzug[i] > 0.0 {
            let zurueck = abzug[i] * erholung;
            pt += zurueck;
            abzug[i] -= zurueck;
            if abzug[i] < 0.05 { pt += abzug[i]; abzug[i] = 0.0; }
        }
        out[i] = pt;
    }
    out
}

/// Catching up with the mentor: abilities the mentor has at least `abstand` higher and that suit the pupil's position.
pub fn mentor_vorbild(l: &Lage, werte: &[f32]) -> Vec<(usize, f32)> {
    let m = |k: &str| f("entwicklung3", &format!("mentor.{k}"));
    let Some(vb) = &l.mentor_werte else { return vec![] };
    if l.mentor <= 0.0 { return vec![]; }
    let mut st = l.mentor * (0.5 + 0.5 * klemme(l.beziehung / 100.0, 0.0, 1.0));
    st *= klemme(1.0 - (l.alter as f32 - m("alter_voll")).max(0.0) * m("alter_je_jahr"), 0.0, 1.0);
    st *= klemme((l.pot - l.gw) / m("potenzial_voll"), 0.0, 1.0);
    if st <= 0.0 { return vec![]; }
    let gewicht = crate::pesdb::wvec(l.pos as u8);
    let mut kand: Vec<(f32, usize)> = (0..25).filter(|&i| gewicht[i] >= 0.0 && vb[i] - werte[i] >= m(crate::id("abstand")) && werte[i] < obergrenze(i) && positionsfaktor(i, l.pos) >= m("position_ab")).map(|i| (vb[i] - werte[i], i)).collect();
    kand.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.cmp(&b.1)));
    kand.into_iter().take(m(crate::id("hoechstens")) as usize).map(|(_, i)| (i, m("je_woche") * st)).collect()
}

/// A leap: young, many good starts, mood, room to the potential → +3…5 bundled on the main abilities (30 %, lock 26 weeks).
pub fn sprung_pruefen(l: &Lage, e: &Ent, werte: &[f32]) -> Vec<(usize, i32)> {
    let s = |k: &str| f("entwicklung3", &format!("sprung.{k}"));
    let bis = s("alter_bis") + if l.merkmale.iter().any(|x| x == "fruehreif") { 1.0 } else { 0.0 };
    let q = l.quote.unwrap_or(0.0);
    if l.alter as f32 > bis || l.quote.is_none() || q < s("quote_ab") || l.moral < s("moral_ab") { return vec![]; }
    if e.letzter_sprung.is_some_and(|d| ((l.heute.saturating_sub(d)) / 7) < s("sperre_wochen") as u32) { return vec![]; }
    let n: Vec<f32> = l.noten.iter().take(k3("woche.noten_fenster") as usize).copied().collect();
    if n.len() < 3 { return vec![]; }
    let m = n.iter().sum::<f32>() / n.len() as f32;
    if m < s("note_ab") || l.pot - l.gw < s("potenzial_abstand") { return vec![]; }
    let woche = (l.heute / 7).to_string();
    let p = s(crate::id("chance")) * effekt(&l.merkmale, crate::id("sprung")) * klemme(0.7 + 0.6 * (m - s("note_ab")), 0.7, 1.4);
    if zufall(&[&l.seed, crate::id("sprung"), &woche]) >= p { return vec![]; }
    let gewicht = crate::pesdb::wvec(l.pos as u8);
    let mut kand: Vec<usize> = (0..25).filter(|&i| gewicht[i] >= 0.0 && werte[i] < obergrenze(i)).collect();
    kand.sort_by(|a, b| positionsfaktor(*b, l.pos).partial_cmp(&positionsfaktor(*a, l.pos)).unwrap().then(NAMEN[*a].cmp(NAMEN[*b])));
    kand.truncate(6);
    if kand.is_empty() { return vec![]; }
    let pk = arr("entwicklung3", "sprung.punkte");
    let (lo, hi) = (num(&pk[0]) as i32, num(&pk[1]) as i32);
    let gesamt = lo + (zufall(&[&l.seed, "sprung_summe", &woche]) * (hi - lo + 1) as f32) as i32;
    let mut v: Vec<(usize, i32)> = vec![];
    for k in 0..gesamt as usize { let i = kand[k % kand.len()]; match v.iter_mut().find(|x| x.0 == i) { Some(x) => x.1 += 1, None => v.push((i, 1)) } }
    v
}

/// What one week did to a player.
#[derive(Default, Debug)]
pub struct Bericht { pub schritte: [i32; 25], pub punkte: [f32; 25], pub sprung: Vec<(usize, i32)>, pub vorher: u8, pub neu: Option<Neu>, pub mentor: Vec<usize> }

/// The whole week of one player: phase, points, mentor, leap, booking in whole steps (wochenlauf.spieler + kalibrierlauf).
pub fn schritt(l: &Lage, e: &mut Ent, werte: &mut [f32]) -> Bericht {
    if e.rest.len() != 25 { e.rest = vec![0.0; 25]; e.abzug = vec![0.0; 25]; e.peak = vec![0.0; 25]; }
    let mut b = Bericht { vorher: e.phase, ..Default::default() };
    let mut neu = phase_bestimmen(l, e);
    if neu.neu {
        e.phase = neu.phase;
        e.seit = l.heute;
        e.bis = if (1..=3).contains(&neu.phase) { l.heute + neu.dauer * 7 } else { 0 };
        e.dauer = neu.dauer;
        e.tiefe = neu.tiefe;
        e.staerke = neu.staerke;
        e.gruende = neu.gruende.clone();
        if neu.sperre_wochen > 0 { e.sperre_bis = l.heute + (neu.sperre_wochen + neu.dauer) * 7; }
        if neu.verbraucht { let fenster = k3("woche.anlass_fenster_tage") as u32; e.anlaesse.retain(|a| l.heute.saturating_sub(a.1) > fenster); }
    }
    let mut punkte = wochenpunkte(l, &neu, werte, &mut e.abzug, &mut e.peak);
    for (i, d) in mentor_vorbild(l, werte) { punkte[i] += d; b.mentor.push(i); }
    let sp = if neu.phase == 2 || neu.phase == 3 { vec![] } else { sprung_pruefen(l, e, werte) };
    if !sp.is_empty() {
        e.letzter_sprung = Some(l.heute);
        let mut g = vec![(crate::id("sprung").to_string(), 1.0)];
        g.extend(neu.gruende.iter().take(2).cloned());
        neu = Neu { phase: 1, dauer: 6, staerke: 1.0, tiefe: 1.0, gruende: g, neu: true, sperre_wochen: 0, verbraucht: false };
        e.phase = 1; e.seit = l.heute; e.bis = l.heute + 42; e.dauer = 6; e.tiefe = 1.0; e.gruende = neu.gruende.clone();
    }
    for i in 0..25 {
        e.rest[i] += punkte[i];
        let r = e.rest[i];
        let mut step = if r.abs() >= 1.0 - 1e-6 { (r + r.signum() * 1e-6).trunc() as i32 } else { 0 };
        e.rest[i] -= step as f32;
        step += sp.iter().find(|x| x.0 == i).map(|x| x.1).unwrap_or(0);
        if step != 0 { werte[i] = (werte[i] + step as f32).clamp(40.0, 99.0); }
        b.schritte[i] = step;
    }
    b.punkte = punkte;
    b.sprung = sp;
    if neu.neu { b.neu = Some(neu); }
    b
}

// ---------------------------------------------------------------- traits (core/merkmale/schaetzen.py, without the nation term)
const CHARAKTER: [&str; 10] = [crate::id("professionalitaet"), crate::id("ehrgeiz"), crate::id("ego"), crate::id("teamgeist"), crate::id("fuehrung"), crate::id("temperament"), crate::id("nervenstaerke"), crate::id("vertraeglichkeit"), crate::id("loyalitaet"), crate::id("status")];
/// The traits of a player, stable per player id: 2–4 drawn by frequency × inclination from the character and position.
pub fn merkmale_schaetzen(p: &Player) -> Vec<String> {
    let k = wert("merkmale", "merkmale").as_array().unwrap();
    let s = |x: &str| f("merkmale", &format!("schaetzung.{x}"));
    let ch = |n: &str| CHARAKTER.iter().position(|c| *c == n).map(|i| crate::mensch::cw(p, i)).unwrap_or(50.0);
    let py = PY_POS[crate::pesdb::pos_index(&p.pos) as usize];
    let gew: Vec<(String, f32)> = k.iter().map(|m| {
        let e: f32 = m["wirkung"].as_object().map(|o| o.iter().map(|(w, pk)| s("charakter_faktor") * num(pk) * (ch(w) - 50.0) / 25.0).sum()).unwrap_or(0.0);
        let mut g = num(&m["haeufigkeit"]) * e.exp();
        if let Some(a) = m["neigung"]["alter"].as_array() { if !(num(&a[0])..=num(&a[1])).contains(&(p.age as f32)) { g *= s("alter_aussen"); } }
        g *= m["neigung"]["position"].get(py).map(num).unwrap_or(1.0);
        (m["id"].as_str().unwrap().to_string(), g)
    }).collect();
    let nach = |id: &str| k.iter().find(|m| m["id"] == id).unwrap();
    let in_liste = |a: &str, art: &str, x: &str| nach(a)["widerspruch"][art].as_array().is_some_and(|v| v.iter().any(|y| y == x));
    let wider = |a: &str, art: &str, x: &str| in_liste(a, art, x) || in_liste(x, art, a);
    let passt = |a: &str, b: &str| nach(a)["passt_zu"].as_array().is_some_and(|v| v.iter().any(|y| y == b)) || nach(b)["passt_zu"].as_array().is_some_and(|v| v.iter().any(|y| y == a));
    let seed = (s(crate::id("seed")) as i64).to_string();
    let idt = p.id.to_string();
    let z = |n: &str| zufall(&[crate::id("mk"), &seed, &idt, n]);
    let vert = wert("merkmale", "anzahl.verteilung").as_object().unwrap();
    let (x, mut summe, mut anzahl) = (z(crate::id("anzahl")), 0.0, 2);
    let mut keys: Vec<&String> = vert.keys().collect();
    keys.sort();
    for n in keys { summe += num(&vert[n]); if x < summe { anzahl = n.parse().unwrap_or(2); break; } }
    let mut gewaehlt: Vec<String> = vec![];
    for r in 0..anzahl {
        let mut frei: Vec<(String, f32)> = gew.iter().filter(|(i, _)| !gewaehlt.contains(i) && !gewaehlt.iter().any(|c| wider(c, crate::id("stark"), i))).cloned().collect();
        if frei.is_empty() { break; }
        for (i, w) in frei.iter_mut() {
            if gewaehlt.iter().any(|c| wider(c, crate::id("schwach"), i)) { *w *= s("schwach_faktor"); }
            if gewaehlt.iter().any(|c| passt(c, i)) { *w *= s("passt_zu_faktor"); }
        }
        frei.sort_by(|a, b| a.0.cmp(&b.0));
        let ziel = z(&format!("w{r}")) * frei.iter().map(|x| x.1).sum::<f32>();
        let mut acc = 0.0;
        for (i, w) in &frei { acc += w; if ziel <= acc { gewaehlt.push(i.clone()); break; } }
    }
    gewaehlt
}

// ---------------------------------------------------------------- the career: inputs, the week, the day, the paths
/// The focus groups a player can train (name, abilities), in file order.
pub fn fokusgruppen() -> Vec<(String, Vec<usize>)> {
    wert("entwicklung", "entwicklung.fokusgruppen").as_object().unwrap().iter()
        .map(|(n, v)| (n.clone(), v.as_array().unwrap().iter().filter_map(|x| NAMEN.iter().position(|y| Some(*y) == x.as_str())).collect())).collect()
}
/// The focus group from the position when none is chosen.
pub fn fokus_standard(pos: usize) -> &'static str { [crate::id("torwart"), crate::id("defensiv"), crate::id("technik"), crate::id("schuss")][bereich(pos)] }
/// The focus group a player trains (empty without the focus programme).
pub fn fokus_von(p: &Player) -> String {
    if p.program != 1 { return String::new(); }
    let pos = crate::pesdb::pos_index(&p.pos) as usize;
    if p.ent.fokus == 0 { fokus_standard(pos).into() } else { fokusgruppen().get(p.ent.fokus as usize - 1).map(|x| x.0.clone()).unwrap_or_default() }
}

impl Career {
    /// Intensity × repetition of the week's plan (entwicklung_pesm.toml plan, einheiten.toml wiederholung).
    pub fn plan_faktor(&self) -> f32 {
        let st = arr("entwicklung_pesm", "plan.einheit_intensitaet");
        let stufen: Vec<f32> = self.plan.iter().map(|&u| num(&st[u as usize])).filter(|x| *x >= 0.0).collect();
        if stufen.is_empty() { return 1.0; }
        let mittel = (stufen.iter().sum::<f32>() / stufen.len() as f32).round() as usize;
        let intens = f("entwicklung3", &format!("einsatz.training_intensitaet.{}", [crate::id("schonend"), crate::id("normal"), crate::id("hart")][mittel.min(2)]));
        let ab = kp("plan.wiederholung_ab") as usize;
        let mut n = [0usize; 6];
        let (mut summe, mut zahl) = (0.0, 0.0);
        for &u in self.plan.iter().filter(|&&u| num(&st[u as usize]) >= 0.0) {
            n[u as usize] += 1;
            summe += f("einheiten", "raster.wiederholung").powi(n[u as usize].saturating_sub(ab - 1) as i32);
            zahl += 1.0;
        }
        intens * summe / zahl
    }
    /// How well a mentor suits a pupil, 0…1 (regeln.mentor_passung, without the nationality).
    pub fn mentor_passung(&self, m: usize, s: usize) -> f32 {
        use crate::mensch::{cw, EGO, FUEHRUNG, PROF, TEAMGEIST, VERTRAEGLICH};
        let e = |k: &str| f("entwicklung", &format!("entwicklung.{k}"));
        let (mp, sp) = (&self.world.players[m], &self.world.players[s]);
        let (am, asz) = (mp.age as f32, sp.age as f32);
        if am < e("mentor_alter_mentor") || asz > e("mentor_alter_schueler") || am - asz < e("mentor_abstand") { return 0.0; }
        let (pr, fu, tg, vt) = (cw(mp, PROF), cw(mp, FUEHRUNG), cw(mp, TEAMGEIST), cw(mp, VERTRAEGLICH));
        let mut sc = (0.4 * pr + 0.2 * fu + 0.2 * tg + 0.2 * vt) / 100.0 + 0.1 * ((am - e("mentor_alter_mentor")) / (e("mentor_erfahrung_bis") - e("mentor_alter_mentor")).max(1.0)).min(1.0);
        let (bm, bs) = (bereich(crate::pesdb::pos_index(&mp.pos) as usize), bereich(crate::pesdb::pos_index(&sp.pos) as usize));
        if mp.pos != sp.pos { sc *= if bm == bs { 0.85 } else { 0.6 }; }
        sc *= 1.0 - 0.4 * (cw(sp, EGO) - 60.0).max(0.0) / 40.0;
        sc *= 1.0 - 0.3 * (cw(mp, EGO) - 70.0).max(0.0) / 30.0;
        klemme(sc, 0.0, 1.0)
    }
    /// Note an occasion for the development phases (refused contract, broken or kept promise, praise, criticism, penalty).
    pub fn anlass(&mut self, p: usize, art: &str, staerke: f32) {
        let abs = self.abs();
        let e = &mut self.world.players[p].ent;
        e.anlaesse.retain(|a| abs < a.1 + kp("anlass.behalten_tage") as u32);
        e.anlaesse.push((art.into(), abs, staerke));
    }
    /// The inputs of one player's week from the career.
    pub fn ent_lage(&self, p: usize) -> Lage {
        let pl = &self.world.players[p];
        let pos = crate::pesdb::pos_index(&pl.pos) as usize;
        let mut l = Lage::neutral(pl.age, pos, pl.pot, pl.ovr);
        let abs = self.abs();
        l.heute = abs;
        l.seed = format!("sp{}", pl.id);
        l.merkmale = pl.ent.merkmale.clone();
        let fenster = k3("woche.spiele_fenster") as usize;
        if let Some((q, n)) = pl.ent.u23.filter(|_| pl.youth > 0) {
            let a = |k: &str| f(crate::id("akademie"), &format!("entwicklung_plus.{k}"));
            let akademie = pl.youth == 17 || q <= 0.0;
            l.quote = Some(if akademie { a("akademie_quote") } else { q });
            if !akademie { l.noten = vec![n; 4]; }
            l.rate = a("rate_faktor") * if akademie { a("akademie_faktor") } else { 1.0 };
        } else if !pl.ps.einsatz.is_empty() {
            let e: Vec<f32> = pl.ps.einsatz.iter().rev().take(fenster).copied().collect();
            l.quote = Some(e.iter().filter(|x| **x >= 0.5).count() as f32 / e.len() as f32);
            l.noten = pl.last.iter().rev().take(8).copied().collect();
        }
        let res: Vec<char> = self.results.iter().map(|r| r.chars().next().unwrap_or('U')).collect();
        l.serie_siege = res.iter().rev().take_while(|&&x| x == 'S').count() as u32;
        l.serie_niederlagen = res.iter().rev().take_while(|&&x| x == 'N').count() as u32;
        l.moral = pl.morale;
        l.moral_vorher = if pl.ent.an { pl.ent.moral_vorher } else { pl.morale };
        l.rolle = ROLLEN[pl.ps.rolle.min(6) as usize].into();
        let fk = fokus_von(pl);
        if !fk.is_empty() { l.fokus = fokusgruppen().into_iter().find(|x| x.0 == fk).map(|x| x.1).unwrap_or_default(); }
        l.einzel = pl.program == 2;
        l.training = self.plan_faktor();
        if let Some(m) = pl.ps.mentor.and_then(|m| self.pid(m)) {
            l.mentor = self.mentor_passung(m, p);
            if self.world.players[m].attrs.len() == 25 { l.mentor_werte = Some(self.world.players[m].attrs.clone()); }
            l.beziehung = (self.bez_wert(pl.id, self.world.players[m].id) + 100.0) / 2.0;
        } else { l.beziehung = kp("mentor.beziehung_neutral"); }
        l.stab = Some(self.trainingsqualitaet());
        l.belastung = pl.ps.last;
        let hw = f("entwicklung", "moral.gespraech_halbwertszeit");
        l.gespraeche = pl.ps.gespr.iter().filter(|x| x.0 <= abs && abs < x.0 + 90).map(|x| x.1 * 0.5f32.powf((abs - x.0) as f32 / hw)).sum();
        l.vertrag = pl.ps.treiber.iter().filter(|t| t.0 == "vertrag" || t.0 == "gehalt").map(|t| (-t.1).max(0.0)).fold(0.0, f32::max);
        l.teamgeist = Some(self.mensch.teamgeist);
        l.rueckkehr = abs < pl.ent.rueckkehr_bis;
        l.anlagen = self.training_faktor();
        l
    }
    /// Once a week for the own squad and academy: phases, points per ability, leaps, mentor, paths, history, messages.
    pub fn entwicklung_woche(&mut self) {
        let abs = self.abs();
        // the week of the whole squad for one collected report: (player, overall before, steps, lines)
        let mut woche: Vec<(usize, f32, i32, Vec<String>)> = vec![];
        for p in self.world.clubs[self.own].squad.clone() {
            if self.world.players[p].attrs.len() != 25 { continue; }
            if !self.world.players[p].ent.an {
                let m = merkmale_schaetzen(&self.world.players[p]);
                let pl = &mut self.world.players[p];
                for a in pl.attrs.iter_mut() { *a = a.round().clamp(40.0, 99.0); }
                pl.ent.an = true;
                pl.ent.merkmale = m;
                pl.ent.moral_vorher = pl.morale;
            }
            let l = self.ent_lage(p);
            let mut gruende: Vec<String> = faktoren(&l, self.world.players[p].ent.phase).into_iter().filter(|x| (x.1 - 1.0).abs() >= 0.04).map(|x| format!("{}{}", x.0, if x.1 > 1.0 { "+" } else { "-" })).collect();
            let pl = &mut self.world.players[p];
            let mut e = std::mem::take(&mut pl.ent);
            let mut werte = std::mem::take(&mut pl.attrs);
            let b = schritt(&l, &mut e, &mut werte);
            let ovr_vorher = pl.ovr;
            let pos = crate::pesdb::pos_index(&pl.pos);
            pl.ovr = crate::pesdb::ovr(&werte, pos).clamp(30.0, 99.0);
            pl.attrs = werte;
            if pl.age >= 27 { pl.pot = pl.ovr; } else if pl.ovr > pl.pot { pl.pot = pl.ovr; }
            // the people side keeps reading the limited phases (form, profile): 1 upswing, 2 slump, 3 crisis
            pl.ps.phase = if (1..=3).contains(&e.phase) { e.phase } else { 0 };
            pl.ps.phase_bis = e.bis;
            e.moral_vorher = pl.morale;
            let (mut gr, mut zahl) = ([0.0f32; 5], [0.0f32; 5]);
            for i in 0..25 { gr[GRUPPE[i]] += b.punkte[i] + b.sprung.iter().find(|x| x.0 == i).map(|x| x.1 as f32).unwrap_or(0.0); zahl[GRUPPE[i]] += 1.0; }
            for g in 0..5 { gr[g] /= zahl[g]; }
            if !l.fokus.is_empty() { gruende.insert(0, crate::id("fokus+").into()); }
            if !b.mentor.is_empty() { gruende.push(crate::id("vorbild+").into()); }
            if !b.sprung.is_empty() { gruende.insert(0, crate::id("sprung+").into()); }
            if l.alter >= 29 && gr.iter().sum::<f32>() < 0.0 { gruende.push(crate::id("alter-").into()); }
            e.verlauf.push(Woche { tag: abs, gruppen: gr, gruende, phase: e.phase });
            let behalten = kp("anzeige.verlauf_wochen") as usize;
            if e.verlauf.len() > behalten { let weg = e.verlauf.len() - behalten; e.verlauf.drain(..weg); }
            let name = pl.name.clone();
            pl.ent = e;
            let zeilen = self.ent_melden(p, &name, &b);
            woche.push((p, ovr_vorher, b.schritte.iter().sum(), zeilen));
        }
        let gelernt = self.wege_woche();
        if !gelernt.is_empty() { woche.push((usize::MAX, 0.0, 0, gelernt)); }
        self.ent_wochenbericht(&woche);
        if (abs / 7) % 4 == 0 { self.ent_monatsbericht(); }
        if (abs / 7) % 4 == 0 { self.mentor_spezial(); }
    }
    /// What a player's week says: a crisis comes as its own message (it needs a talk); a slump, the way back and a
    /// leap become lines of the weekly report.
    fn ent_melden(&mut self, p: usize, name: &str, b: &Bericht) -> Vec<String> {
        let absender = crate::inhalt::t("en3.absender");
        let mut zeilen = vec![];
        if let Some(n) = &b.neu {
            let ph = PHASEN[n.phase as usize];
            if (n.phase == 2 || n.phase == 3) && !(b.vorher == 2 && n.phase == 2) && (n.phase == 3 || b.vorher != 3) {
                let g = n.gruende.first().map(|g| g.0.clone()).unwrap_or_else(|| "allgemein".into());
                let grund = crate::inhalt::t(&format!("en3.grund.{g}"));
                let text = crate::tf!(&format!("en3.nachricht.{ph}"), name, grund);
                if n.phase == 3 {
                    self.msg(1, absender, crate::inhalt::t(&format!("en3.betreff.{ph}")), &text, Action::None);
                    self.meldung_art(crate::pages::Art::Gespraech, vec![p]);
                } else { zeilen.push(text); }
            } else if (b.vorher == 2 || b.vorher == 3) && n.phase != 2 && n.phase != 3 {
                zeilen.push(crate::tf!("en3.nachricht.genesung", name));
            }
        }
        if !b.sprung.is_empty() {
            let mut s = b.sprung.clone();
            s.sort_by(|a, x| x.1.cmp(&a.1));
            let namen = s.iter().take(3).map(|(i, d)| format!("{} +{d}", crate::inhalt::t(&format!("faehigkeit.{}", NAMEN[*i])))).collect::<Vec<_>>().join(", ");
            zeilen.push(crate::tf!("en3.nachricht.sprung", name, namen));
        }
        zeilen
    }
    /// One collected message per week for the squad (wochenlauf.melden): who went up and down in the overall, then the
    /// slumps, comebacks and leaps. Nothing to say: no message.
    fn ent_wochenbericht(&mut self, woche: &[(usize, f32, i32, Vec<String>)]) {
        let pl = |p: usize| &self.world.players[p];
        let mut auf: Vec<(usize, f32)> = woche.iter().filter(|x| x.0 != usize::MAX).map(|x| (x.0, pl(x.0).ovr - x.1)).filter(|x| x.1.round() >= 1.0).collect();
        let mut ab: Vec<(usize, f32)> = woche.iter().filter(|x| x.0 != usize::MAX).map(|x| (x.0, pl(x.0).ovr - x.1)).filter(|x| x.1.round() <= -1.0).collect();
        auf.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ab.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let werte: i32 = woche.iter().map(|x| x.2).sum();
        let zeilen: Vec<&String> = woche.iter().flat_map(|x| x.3.iter()).collect();
        if werte == 0 && auf.is_empty() && ab.is_empty() && zeilen.is_empty() { return; }
        let liste = |v: &[(usize, f32)]| v.iter().take(5).map(|(p, d)| format!("{} {:+.0} ({:.0})", pl(*p).name, d, pl(*p).ovr)).collect::<Vec<_>>().join(", ");
        let mut body = vec![crate::tf!("ui.ent.woche.summe", format!("{werte:+}"))];
        if !auf.is_empty() { body.push(crate::tf!("ui.ent.woche.auf", liste(&auf))); }
        if !ab.is_empty() { body.push(crate::tf!("ui.ent.woche.ab", liste(&ab))); }
        body.extend(zeilen.into_iter().cloned());
        let spieler: Vec<usize> = auf.iter().chain(ab.iter()).map(|x| x.0).take(6).collect();
        self.msg(1, crate::inhalt::t("en3.absender"), crate::inhalt::t("ui.ent.woche.titel"), &body.join(crate::id("\n")), Action::None);
        self.meldung_art(crate::pages::Art::Allgemein, spieler);
    }
    /// Every fourth week: the month of the squad from the history (winners, losers, phases, leaps).
    fn ent_monatsbericht(&mut self) {
        let sq: Vec<usize> = self.squad().into_iter().filter(|&p| self.world.players[p].ent.an).collect();
        if sq.is_empty() { return; }
        let summe = |p: usize| self.world.players[p].ent.verlauf.iter().rev().take(4).map(|w| w.gruppen.iter().sum::<f32>()).sum::<f32>();
        let mut v: Vec<(usize, f32)> = sq.iter().map(|&p| (p, summe(p))).collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let name = |p: usize| self.world.players[p].name.clone();
        let gewinner = v.iter().filter(|x| x.1 > 0.0).take(3).map(|x| name(x.0)).collect::<Vec<_>>().join(", ");
        let verlierer = v.iter().rev().filter(|x| x.1 < 0.0).take(3).map(|x| name(x.0)).collect::<Vec<_>>().join(", ");
        let phase = |ph: u8| sq.iter().filter(|&&p| self.world.players[p].ent.phase == ph).count();
        let spruenge = sq.iter().filter(|&&p| self.world.players[p].ent.verlauf.iter().rev().take(4).any(|w| w.gruende.iter().any(|g| g == "sprung+"))).count();
        let mut body = vec![];
        if !gewinner.is_empty() { body.push(crate::tf!("ui.ent.monat.gewinner", gewinner)); }
        if !verlierer.is_empty() { body.push(crate::tf!("ui.ent.monat.verlierer", verlierer)); }
        body.push(crate::tf!("ui.ent.monat.phasen", phase(1), phase(2), phase(3), spruenge));
        let spieler: Vec<usize> = v.iter().take(3).map(|x| x.0).collect();
        self.msg(1, crate::inhalt::t("en3.absender"), crate::inhalt::t("ui.ent.monat.titel"), &body.join(crate::id("\n")), Action::None);
        self.meldung_art(crate::pages::Art::Allgemein, spieler);
    }
    /// Expected change of the overall per season from this week's model (training and academy arrows); nothing is changed.
    pub fn ent_erwartung(&self, p: usize) -> f32 {
        let pl = &self.world.players[p];
        if pl.attrs.len() != 25 { return 0.0; }
        let l = self.ent_lage(p);
        let mut e = pl.ent.clone();
        if e.abzug.len() != 25 { e.abzug = vec![0.0; 25]; e.peak = vec![0.0; 25]; }
        let neu = phase_bestimmen(&l, &e);
        let pt = wochenpunkte(&l, &neu, &pl.attrs, &mut e.abzug, &mut e.peak);
        let w = crate::pesdb::wvec(crate::pesdb::pos_index(&pl.pos));
        pt.iter().zip(w).map(|(a, b)| a * b).sum::<f32>() * 52.0
    }
    /// One day: days out of the current injury; on the return the rehab factor starts and a long injury may cost physical points.
    pub fn ent_tag(&mut self) {
        let abs = self.abs();
        for p in self.world.clubs[self.own].squad.clone() {
            if self.world.players[p].inj > 0 { self.world.players[p].ent.ausfall += 1; continue; }
            let tage = self.world.players[p].ent.ausfall;
            if tage == 0 { continue; }
            self.world.players[p].ent.ausfall = 0;
            self.world.players[p].ent.rueckkehr_bis = abs + k2("wachstum.rueckkehr_tage") as u32;
            let v = |k: &str| f("entwicklung2", &format!("verletzung.{k}"));
            if (tage as f32) < v("ab_tage") || self.world.players[p].attrs.len() != 25 { continue; }
            let alter = self.world.players[p].age as f32;
            let mut chance = (tage as f32 * v("chance_je_tag")).min(v("chance_deckel"));
            if alter > v("alter_zuschlag_ab") { chance = (chance + (alter - v("alter_zuschlag_ab")) * v("alter_zuschlag_je_jahr")).min(v("chance_deckel")); }
            let (id, tag) = (self.world.players[p].id.to_string(), abs.to_string());
            if zufall(&[crate::id("verletzungsfolge"), &id, &tag]) >= chance { continue; }
            let i = KOERPERLICH[(zufall(&["verletzungsfolge_wahl", &id, &tag]) * 6.0) as usize % 6];
            let verlust = if tage as f32 >= v("schwer_ab_tage") { v("schwer_verlust") } else { v(crate::id("verlust")) };
            let pl = &mut self.world.players[p];
            pl.attrs[i] = (pl.attrs[i] - verlust).max(40.0);
            pl.ovr = crate::pesdb::ovr(&pl.attrs, crate::pesdb::pos_index(&pl.pos)).clamp(30.0, 99.0);
            let n = pl.name.clone();
            let fa = crate::inhalt::t(&format!("faehigkeit.{}", NAMEN[i]));
            self.msg(1, crate::inhalt::t("med.vg.absender"), &crate::tf!("ui.ent.folge_titel", n), &crate::tf!("ui.ent.folge_text", n, tage, fa, verlust), Action::None);
            self.meldung_art(crate::pages::Art::Verletzung, vec![p]);
        }
    }
    /// Progress paths (entwicklung2.toml [[weg]]): learn a playing style (at most 1) or a special ability (at most 4) once the
    /// prerequisites are met, lose it from 33 or without need of the position; the bits go into Player.bin with the next live write.
    pub fn wege_woche(&mut self) -> Vec<String> {
        let mut gelernt = vec![];
        let wg = |k: &str| f("entwicklung2", &format!("wege.{k}"));
        let abs = self.abs();
        let woche = (abs / 7).to_string();
        let wege = arr("entwicklung2", "weg");
        for p in self.world.clubs[self.own].squad.clone() {
            if self.world.players[p].attrs.len() != 25 || !self.world.players[p].ent.an { continue; }
            let l = self.ent_lage(p);
            let mentor = self.world.players[p].ps.mentor.and_then(|m| self.pid(m));
            for w in wege {
                let id = w["id"].as_str().unwrap();
                let art_stil = w["art"] == "spielstil";
                let pes = wert("entwicklung_pesm", "wege_pes").get(id).and_then(|x| x.as_str()).unwrap_or("");
                let (stil, skill) = (crate::pesdb::STYLES.iter().position(|s| *s == pes), crate::pesdb::SKILLS.iter().position(|s| s.0 == pes));
                let pl = &self.world.players[p];
                let voraus = w["voraussetzung"].as_object().unwrap();
                let wert_von = |n: &str| NAMEN.iter().position(|x| *x == n).map(|i| pl.attrs[i]).unwrap_or(0.0);
                let erfuellt = voraus.iter().all(|(n, v)| wert_von(n) >= num(v));
                let positionen = w["positionen"].as_array().unwrap();
                let passt = positionen.is_empty() || positionen.iter().any(|x| x.as_str() == Some(PY_POS[l.pos]));
                let i = pl.ent.wege.iter().position(|x| x.0 == id);
                let erreicht = i.is_some_and(|i| pl.ent.wege[i].1 as f32 >= wg("treffer_bis"));
                let pid = pl.id.to_string();
                let z = |k: &str| zufall(&[crate::id("weg"), id, &pid, &woche, k]);
                if erreicht {
                    let bedarf = wert("entwicklung2", "bedarf").get(id).and_then(|x| x.as_array()).map_or(true, |a| a.iter().any(|x| x.as_str() == Some(BEREICHE[bereich(l.pos)])));
                    let ohne = if bedarf { None } else { Some(pl.ent.ohne_bedarf.iter().find(|x| x.0 == id).map(|x| x.1).unwrap_or(abs)) };
                    let alt_verlust = pl.age as f32 >= wg("verlust_alter") && voraus.iter().any(|(n, v)| wert_von(n) < num(v) - wg("verlust_toleranz")) && z(crate::id("alter")) < wg("verlust_chance_woche");
                    let pos_verlust = ohne.is_some_and(|s| (abs.saturating_sub(s) / 7) as f32 >= wg("position_wochen")) && z(crate::id("position")) < wg("position_verlust_chance_woche");
                    let plm = &mut self.world.players[p];
                    match ohne { Some(s) => { if !plm.ent.ohne_bedarf.iter().any(|x| x.0 == id) { plm.ent.ohne_bedarf.push((id.into(), s)); } } None => plm.ent.ohne_bedarf.retain(|x| x.0 != id) }
                    if alt_verlust || pos_verlust {
                        plm.ent.wege.retain(|x| x.0 != id);
                        plm.ent.ohne_bedarf.retain(|x| x.0 != id);
                        if let Some(k) = skill { plm.skills &= !(1u64 << k); }
                        if stil.is_some_and(|s| plm.style as usize == s) { plm.style = 0; }
                        let n = plm.name.clone();
                        self.msg(1, crate::inhalt::t("en3.absender"), &crate::tf!("ui.ent.weg_verloren_titel", n), &crate::tf!("ui.ent.weg_verloren_text", n, crate::inhalt::t(&format!("en2.weg.{id}"))), Action::None);
                        self.meldung_art(crate::pages::Art::Allgemein, vec![p]);
                    }
                    continue;
                }
                if !erfuellt || !passt { continue; }
                // limits: one style, four special abilities through paths, ten special abilities in the game
                let anzahl = |stil_art: bool| pl.ent.wege.iter().filter(|x| x.1 as f32 >= wg("treffer_bis") && wege.iter().any(|w2| w2["id"] == x.0.as_str() && (w2["art"] == "spielstil") == stil_art)).count() as f32;
                if art_stil && (anzahl(true) >= wg("spielstile_hoechstens") || (pl.style != 0 && stil.is_some())) { continue; }
                if !art_stil && (anzahl(false) >= wg("spezial_hoechstens") || pl.skills.count_ones() as f32 >= kp("wege_pes.skills_hoechstens")) { continue; }
                if skill.is_some_and(|k| pl.skills >> k & 1 == 1) { continue; }
                let mut chance = wg(crate::id("chance")) * klemme(l.quote.unwrap_or(0.0) / wg("einsaetze_faktor"), 0.0, 1.0);
                if mentor.is_some_and(|m| self.world.players[m].ent.wege.iter().any(|x| x.0 == id && x.1 as f32 >= wg("treffer_bis"))) { chance += wg("mentor_zuschlag"); }
                if w["fokus"].as_str() == Some(fokus_von(pl).as_str()) { chance += wg("fokus_zuschlag"); }
                if l.moral > 65.0 { chance += wg("moral_hoch_zuschlag"); } else if l.moral < 35.0 { chance -= wg("moral_niedrig_abzug"); }
                if z(crate::id("treffer")) >= chance { continue; }
                let plm = &mut self.world.players[p];
                let treffer = match i { Some(i) => { plm.ent.wege[i].1 += 1; plm.ent.wege[i].1 } None => { plm.ent.wege.push((id.into(), 1, 0)); 1 } };
                if treffer as f32 >= wg("treffer_bis") {
                    if let Some(i) = plm.ent.wege.iter().position(|x| x.0 == id) { plm.ent.wege[i].2 = abs; }
                    if let Some(k) = skill { plm.skills |= 1u64 << k; }
                    if let Some(s) = stil { plm.style = s as u8; }
                    let n = plm.name.clone();
                    // a line of the weekly development report, not a message of its own (Fabian 22.11.: too many messages)
                    gelernt.push(crate::tf!("ui.ent.weg_erreicht_text", n, crate::inhalt::t(&format!("en2.weg.{id}"))));
                }
            }
        }
        gelernt
    }
    /// Once a month: a pupil takes over a special ability of his mentor (12 % × fit, fit ≥ 0,5, relation ≥ 60).
    pub fn mentor_spezial(&mut self) {
        let m = |k: &str| f("entwicklung3", &format!("mentor.{k}"));
        let monat = (self.abs() / 28).to_string();
        for p in self.world.clubs[self.own].squad.clone() {
            let Some(mi) = self.world.players[p].ps.mentor.and_then(|x| self.pid(x)) else { continue };
            let st = self.mentor_passung(mi, p);
            let (a, b) = (self.world.players[p].id, self.world.players[mi].id);
            let bez = (self.bez_wert(a, b) + 100.0) / 2.0;
            if st < m("skill_passung_ab") || bez < m("skill_beziehung_ab") { continue; }
            if zufall(&["mentor_skill", &a.to_string(), &monat]) >= m("skill_chance") * st { continue; }
            let (ms, ss) = (self.world.players[mi].skills, self.world.players[p].skills);
            if ss.count_ones() as f32 >= kp("wege_pes.skills_hoechstens") { continue; }
            let Some(k) = (0..35).find(|k| ms >> k & 1 == 1 && ss >> k & 1 == 0) else { continue };
            self.world.players[p].skills |= 1u64 << k;
            let (n, mn) = (self.world.players[p].name.clone(), self.world.players[mi].name.clone());
            self.msg(1, &mn, &crate::tf!("ui.ent.mentor_skill_titel", n), &crate::tf!("ui.ent.mentor_skill_text", mn, n, crate::pesdb::SKILLS[k].0), Action::None);
            self.meldung_art(crate::pages::Art::Talent, vec![p]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn werte(pos: usize, gw: f32) -> Vec<f32> {
        // a profile of the position at this overall: the abilities the position counts are moved until ovr ≈ gw
        let mut v = vec![60.0f32; 25];
        let d = gw - crate::pesdb::ovr(&v, pos as u8);
        crate::pesdb::shift(&mut v, pos as u8, d);
        for x in v.iter_mut() { *x = x.round(); }
        v
    }
    const MERKMALE: [&str; 13] = ["arbeitstier", "fruehreif", "ehrgeizig", "launisch", "sensibel", "hitzkopf", "ruhig", "gelassen", "diszipliniert", "nervenstark", "perfektionist", "kaempfer", "selbstbewusst"];
    /// One season (52 weeks) like tools/entwicklung_kalibrierlauf.py lauf_spieler: results 45 / 25 / 30 % with their series,
    /// grades around the role's mean (±0,3 after a win or a loss, spread 0,55), the mood drifting back to the role, two traits
    /// drawn from the same list unless given, the potential like regeln.potenzial (±3, rare talents +4…9).
    /// Returns (Δ overall, Δ points per group).
    fn saison(alter: u8, pos: usize, gw: f32, rolle: &str, merkmale: Option<&[&str]>, fokus: bool, seed: u32) -> (f32, [f32; 5]) {
        let sd = seed.to_string();
        let z = |k: &str| zufall(&["kal", &sd, k]);
        let mut a = werte(pos, gw);
        let start = a.clone();
        let g0 = crate::pesdb::ovr(&a, pos as u8);
        let mut e = Ent::default();
        let streu = (z("pot") * 2.0 - 1.0) * 3.0 + if alter < 24 && z("talent") > 0.94 { 4.0 + z("talent2") * 5.0 } else { 0.0 };
        let pot = (gw + (26.0 - alter as f32).max(0.0) * 1.7 + streu).min(99.0);
        let ids: Vec<String> = match merkmale {
            Some(m) => m.iter().map(|x| x.to_string()).collect(),
            None => { let i = (z("m1") * 13.0) as usize % 13; let j = (i + 1 + (z("m2") * 12.0) as usize % 12) % 13; vec![MERKMALE[i].into(), MERKMALE[j].into()] }
        };
        let basis = match rolle { "gut" => 62.0, "normal" => 52.0, _ => 44.0 };
        let (quote, mittel) = match rolle { "gut" => (0.9, 6.8), "normal" => (0.78, 6.4), _ => (0.03, 6.2) };
        let (mut moral, mut moral_vor) = (basis, basis);
        let (mut noten, mut siege, mut niederlagen): (Vec<f32>, u32, u32) = (vec![], 0, 0);
        for wo in 0..52u32 {
            let w = wo.to_string();
            let r = |k: &str| zufall(&["kal", &sd, &w, k]);
            let x = r("ausgang");
            let (sieg, niederlage) = (x < 0.45, x >= 0.70);
            siege = if sieg { siege + 1 } else { 0 };
            niederlagen = if niederlage { niederlagen + 1 } else { 0 };
            if rolle != "bank" || r("bank") < 0.05 {
                let m = mittel + if sieg { 0.3 } else if niederlage { -0.3 } else { 0.0 };
                let g = (-2.0 * r("g1").max(1e-6).ln()).sqrt() * (std::f32::consts::TAU * r("g2")).cos();
                noten.insert(0, ((m + 0.55 * g) * 10.0).round().clamp(45.0, 90.0) / 10.0);
            }
            let mut l = Lage::neutral(alter, pos, pot, crate::pesdb::ovr(&a, pos as u8));
            l.heute = wo * 7 + 7;
            l.seed = format!("sp{seed}");
            l.merkmale = ids.clone();
            l.moral = moral;
            l.moral_vorher = moral_vor;
            l.quote = Some(quote);
            l.noten = noten.iter().take(8).copied().collect();
            l.serie_siege = siege;
            l.serie_niederlagen = niederlagen;
            if fokus { l.fokus = vec![10, 11]; }
            schritt(&l, &mut e, &mut a);
            moral_vor = moral;
            moral += (basis - moral) * 0.12 - if e.phase == 2 || e.phase == 3 { 3.0 * 0.3 } else { 0.0 };
        }
        // measured with the fractions not yet booked, so small differences show within one season
        let mit_rest: Vec<f32> = a.iter().zip(&e.rest).map(|(x, r)| x + r).collect();
        let mut gr = [0.0f32; 5];
        for i in 0..25 { gr[GRUPPE[i]] += mit_rest[i] - start[i]; }
        (crate::pesdb::ovr(&mit_rest, pos as u8) - g0, gr)
    }
    #[test]
    fn kalibrierung_und_regeln_geladen() {
        assert!(bevoelkerungsrate(10, 19) > 0.3, "Tempo wächst mit 19");
        assert!(bevoelkerungsrate(10, 33) < 0.0, "Tempo fällt mit 33");
        assert!(bevoelkerungsrate(2, 33) > -0.2, "Technik hält länger");
        assert!((0..25).all(|i| obergrenze(i) <= 99.0));
        assert!(talent_faktor(80.0, 60.0, 18) > talent_faktor(65.0, 60.0, 18));
        assert_eq!(talent_faktor(80.0, 60.0, 30), 1.0);
        let ids = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(effekt(&ids(&["sensibel", "launisch"]), "dauer") > effekt(&ids(&["gelassen"]), "dauer"));
        assert!((effekt(&ids(&["arbeitstier"]), "wachstum") - 1.15).abs() < 1e-4 && (effekt(&ids(&["launisch"]), "dauer") - 1.3).abs() < 1e-4);
        assert!((effekt(&ids(&["fruehreif"]), "jung") - 1.3).abs() < 1e-4 && (effekt(&ids(&["fruehreif"]), "reif") - 0.8).abs() < 1e-4);
        assert!((positionsfaktor(10, 12) - 1.3).abs() < 1e-4 && (positionsfaktor(21, 12) - 0.55).abs() < 1e-4, "Hauptfähigkeit ×1,3, Nebenfähigkeit ×0,55");
        assert!(alterungszuschlag(0, 28) == 0.0 && alterungszuschlag(0, 31) > 1.0 && alterungszuschlag(4, 31) == 0.0, "Abbau ab 29, Torhüter vier Jahre später");
    }
    #[test]
    fn phasen_aus_anlass_und_leistung() {
        let mut l = Lage::neutral(25, 12, 75.0, 72.0);
        l.quote = Some(0.8);
        l.noten = vec![6.5; 4];
        l.heute = 200;
        let mut e = Ent::default();
        e.anlaesse = vec![("vertrag_verwehrt".into(), 199, 1.0)];
        let n = phase_bestimmen(&l, &e);
        assert_eq!((n.phase, n.gruende[0].0.as_str()), (2, "vertrag_verwehrt"));
        assert!((3..=12).contains(&n.dauer));
        e.anlaesse.push(("zusage_gebrochen".into(), 198, 1.0));
        assert_eq!(phase_bestimmen(&l, &e).phase, 3, "zwei Anlässe: Krise");
        let mut gut = l.clone();
        gut.noten = vec![7.6, 7.4, 7.3, 7.2];
        gut.moral = 60.0;
        assert_eq!(phase_bestimmen(&gut, &Ent::default()).phase, 1);
        assert_eq!(phase_bestimmen(&Lage { alter: 34, ..l.clone() }, &Ent::default()).phase, 5);
        assert_eq!(phase_bestimmen(&Lage { alter: 19, quote: Some(0.9), ..l.clone() }, &Ent::default()).phase, 4);
        // after a slump the lock holds 8 weeks beyond its end: the same weak grades start nothing new
        let mut e = Ent::default();
        let mut a = werte(12, 72.0);
        let mut schlecht = l.clone();
        schlecht.noten = vec![5.0; 4];
        schritt(&schlecht, &mut e, &mut a);
        assert_eq!(e.phase, 2);
        assert!(e.sperre_bis >= schlecht.heute + 8 * 7);
    }
    #[test]
    fn formtief_zieht_ab_und_erholung_bringt_zurueck() {
        let l = Lage { quote: Some(0.7), ..Lage::neutral(25, 12, 75.0, 72.0) };
        let w = vec![70.0; 25];
        let (mut ab, mut pk) = (vec![0.0; 25], vec![0.0; 25]);
        let ft = Neu { phase: 2, dauer: 5, staerke: 1.0, tiefe: 2.0, gruende: vec![], neu: true, sperre_wochen: 8, verbraucht: true };
        let p1 = wochenpunkte(&l, &ft, &w, &mut ab, &mut pk);
        assert!(p1.iter().sum::<f32>() < -1.0);
        assert!(ab[0] > ab[10], "Kopf trifft es stärker als Tempo");
        let vorbei = Neu { phase: 0, dauer: 0, staerke: 1.0, tiefe: 0.0, gruende: vec![], neu: true, sperre_wochen: 0, verbraucht: false };
        let a0 = ab[0];
        let p2 = wochenpunkte(&l, &vorbei, &w, &mut ab, &mut pk);
        assert!(p2[0] > p1[0] && ab[0] < a0);
        assert!((a0 - ab[0] - a0 * 0.3).abs() < 1e-3, "30 % je Woche kommen zurück");
        // 10–20 % of a loss and 50–80 % of a gain stay for good
        let pers = |s: &str| GRUPPEN.iter().map(|g| num(&wert("entwicklung3", &format!("persistenz.{s}"))[*g])).collect::<Vec<_>>();
        assert!(pers("verlust").iter().all(|x| (0.1..=0.2).contains(x)) && pers("gewinn").iter().all(|x| (0.5..=0.8).contains(x)));
    }
    #[test]
    fn sprung_nur_jung_mit_vielen_guten_einsaetzen_und_sperre() {
        let mut l = Lage::neutral(18, 12, 82.0, 66.0);
        l.merkmale = vec!["fruehreif".into()];
        l.quote = Some(1.0);
        l.noten = vec![7.5, 7.2, 7.0, 7.1];
        l.moral = 70.0;
        let w = vec![65.0; 25];
        let e = Ent::default();
        let treffer: Vec<u32> = (0..60).filter(|&wo| !sprung_pruefen(&Lage { heute: wo * 7, ..l.clone() }, &e, &w).is_empty()).collect();
        assert!(!treffer.is_empty());
        let t = Lage { heute: treffer[0] * 7, ..l.clone() };
        let s: i32 = sprung_pruefen(&t, &e, &w).iter().map(|x| x.1).sum();
        assert!((3..=5).contains(&s));
        assert!(sprung_pruefen(&Lage { alter: 26, ..t.clone() }, &e, &w).is_empty());
        assert!(sprung_pruefen(&Lage { quote: Some(0.3), ..t.clone() }, &e, &w).is_empty());
        assert!(sprung_pruefen(&Lage { pot: 68.0, ..t.clone() }, &e, &w).is_empty());
        let gesperrt = Ent { letzter_sprung: Some(t.heute.saturating_sub(5 * 7)), ..Ent::default() };
        assert!(sprung_pruefen(&t, &gesperrt, &w).is_empty(), "Sperre 26 Wochen");
    }
    #[test]
    fn spielzeit_fokus_alter_und_merkmale() {
        // 18-year-olds with play time and focus grow faster than the same players without (8 players, the potential spreads)
        let summe = |rolle: &str, fokus: bool, g: Option<usize>| (0..8).map(|s| { let (d, gr) = saison(18, 12, 64.0, rolle, Some(&[]), fokus, 10 + s); g.map_or(d, |g| gr[g]) }).sum::<f32>() / 8.0;
        let (mit, ohne) = (summe("gut", true, None), summe("bank", false, None));
        assert!(mit > ohne + 1.0, "mit Spielzeit und Fokus {mit:.1}, ohne {ohne:.1}");
        // the focus moves growth into its abilities: pace grows more with the pace focus than without
        // (a young starter is at the ×2,1 cap of all favourable circumstances anyway, so this is measured on the bench)
        assert!(summe("bank", true, Some(0)) > summe("bank", false, Some(0)), "Fokus Tempo");
        // a 32-year-old winger loses above all pace (per ability: tempo 2, technik 10, kopf 4 abilities)
        let (d, gr) = saison(32, 9, 78.0, "normal", Some(&[]), false, 4);
        assert!(d < 0.0 && gr[0] / 2.0 < gr[2] / 10.0 && gr[0] / 2.0 < gr[3] / 4.0, "Tempo fällt am stärksten: {gr:?}");
        // a hard worker grows faster
        let mittel = |m: &[&str]| (0..8).map(|s| saison(19, 5, 64.0, "bank", Some(m), false, 20 + s).0).sum::<f32>() / 8.0;
        let (fleiss, normal) = (mittel(&["arbeitstier"]), mittel(&[]));
        assert!(fleiss > normal, "Arbeitstier {fleiss:.2} gegen {normal:.2}");
        // all favourable circumstances together at most ×2,1
        let mut l = Lage::neutral(18, 12, 90.0, 60.0);
        l.quote = Some(1.0); l.training = 1.2; l.einzel = true; l.mentor = 1.0; l.moral = 90.0; l.stab = Some(100.0); l.gespraeche = 10.0; l.teamgeist = Some(100.0);
        let g: f32 = faktoren(&l, 4).iter().map(|x| x.1).product();
        assert!(g > 2.1 && verdichten(g) <= 2.1 + 1e-6);
    }
    /// One season compared with the Python calibration (tests/test_entwicklung3.py): young starter +3…+6 a year, a 25-year-old
    /// 0…+2, a 33-year-old −3…−1 (median over 12 players); the table goes to KALIBRIERUNG.md.
    #[test]
    fn saison_kalibrierungsvergleich() {
        let lauf = |alter: u8, pos: usize, gw: f32, rolle: &str| { let mut v: Vec<f32> = (0..12).map(|s| saison(alter, pos, gw, rolle, None, false, 100 + s).0).collect(); v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v };
        let jung = lauf(19, 12, 66.0, "gut");
        let reif = lauf(25, 5, 72.0, "gut");
        let alt = lauf(33, 1, 74.0, "normal");
        let med = |v: &Vec<f32>| v[v.len() / 2];
        let zeile = |n: &str, v: &Vec<f32>, soll: &str| format!("| {n} | {soll} | {:+.1} | {:+.1} … {:+.1} |\n", med(v), v[0], v[v.len() - 1]);
        let text = format!("# Kalibrierungsvergleich Entwicklung (eine Saison, 52 Wochen, je 12 Spieler)\n\nErzeugt vom Test `entwicklung::tests::saison_kalibrierungsvergleich`. Erwartung aus `tests/test_entwicklung3.py` der Python-Fassung; gemessen wird der Gesamtwert des Rust-Modells (`ovr_weights_fl27.json`).\n\n| Fall | Erwartung (Median) | Rust (Median) | Spanne |\n|---|---|---|---|\n{}{}{}",
            zeile("19 J., Sturm, Stärke 66, Stammspieler mit guten Noten", &jung, "+3 … +6"), zeile("25 J., zentrales Mittelfeld, 72, Stammspieler mit guten Noten", &reif, "0 … +2"), zeile("33 J., Innenverteidiger, 74, Stammspieler", &alt, "−3 … −1"));
        let _ = std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/KALIBRIERUNG.md"), &text);
        assert!((3.0..=6.0).contains(&med(&jung)) && jung[11] <= 9.0, "jung {jung:?}");
        assert!((0.0..=2.0).contains(&med(&reif)) && reif[11] <= 3.0, "reif {reif:?}");
        assert!((-3.0..=-1.0).contains(&med(&alt)), "alt {alt:?}");
    }
    #[test]
    fn wochen_und_monatsbericht_gesammelt() {
        let d = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&d) { return; }
        let w = crate::world::World::from_pesdb(&d, 7).unwrap();
        let own = w.clubs.iter().position(|c| c.id == 396).unwrap();
        let mut c = crate::game::Career::new(w, own, "Test");
        c.inbox.clear();
        // eight weeks of the season with matches and training, like a person who plays them
        c.world.day = 60;
        for _ in 0..56 { if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); } c.ultimatum = None; c.end_day().unwrap(); }
        let titel = |k: &str| c.inbox.iter().filter(|m| m.title == crate::inhalt::t(k)).count();
        assert!((1..=8).contains(&titel("ui.ent.woche.titel")), "eine Sammelmeldung je Woche, {}", titel("ui.ent.woche.titel"));
        assert!(titel("ui.ent.monat.titel") >= 1, "Monatsbericht");
        assert_eq!(titel("en3.betreff.sprung") + titel("en3.betreff.formtief") + titel("en3.betreff.genesung"), 0, "keine Einzelmeldungen mehr");
    }
    #[test]
    fn merkmale_stabil_und_zwei_bis_vier() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 5);
        let mut arbeitstiere = 0;
        for p in w.players.iter().take(300) {
            let m = merkmale_schaetzen(p);
            assert!((2..=4).contains(&m.len()), "{m:?}");
            assert_eq!(m, merkmale_schaetzen(p));
            if m.iter().any(|x| x == "arbeitstier") { arbeitstiere += 1; }
        }
        assert!(arbeitstiere > 0);
    }
}
