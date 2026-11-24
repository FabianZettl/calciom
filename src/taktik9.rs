//! The PES M tactic (Konzept taktiksystem.md Teil 1–2, Fabian 24.11.: "FM27 level, translated into the FL values and
//! written into Tactics"): a playing idea, nine sliders with football words (1–5), four switches. A pure translation
//! turns them into the game's values (attack style, build-up, attacking area, positioning, support, defensive style,
//! containment area, pressing, defensive line, compactness), each with its reason.
use serde::{Deserialize, Serialize};

pub const REGLER: [&str; 9] = ["tempo", "passweg", "seite", "risiko", "freiheit", "pressing", "haerte", "linie", "kompakt"]; // roh
pub const SCHALTER: [&str; 4] = ["gegenpressing", "absichern", "abseitsfalle", "zeitspiel"]; // roh
/// The game's advanced instructions (as The Gaffer lists them for the save): attack 1–10, defence 11–16.
pub const ANWEISUNGEN_ANGRIFF: [(u8, &str); 10] = [(1, "seitenlinie"), (2, "falsche_neun"), (3, "falsche_av"), (4, "offensive_av"), (5, "fluegelrotation"), (6, "tikitaka"), (7, "flankenziele"), (8, "defensiv"), (9, "falscher_fluegel"), (10, "anker")]; // roh
pub const ANWEISUNGEN_ABWEHR: [(u8, &str); 6] = [(11, "strafraum"), (12, "tiefe_linie"), (13, "gegenpressing"), (14, "enge_deckung"), (15, "konterziel"), (16, "fluegelverteidiger")]; // roh
/// The key of an instruction id ("" for none).
pub fn anweisung_key(id: u8) -> &'static str { ANWEISUNGEN_ANGRIFF.iter().chain(ANWEISUNGEN_ABWEHR.iter()).find(|x| x.0 == id).map(|x| x.1).unwrap_or("") }
pub const IDEEN: [&str; 8] = ["gegenpressing", "ballbesitz", "konter", "mauern", "fluegel", "direkt", "kontrolliert", "tikitaka"]; // roh

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Taktik9 { pub idee: u8, pub regler: [u8; 9], pub schalter: [bool; 4], pub angepasst: bool, #[serde(default)] pub aktiv: bool }
// not yet taken over from the tactic: the page derives the sliders from it once (saves from before 24.11. too)
impl Default for Taktik9 { fn default() -> Self { Taktik9 { aktiv: false, ..Self::aus_idee(6) } } }
impl Taktik9 {
    /// The starting values of a playing idea (spielideen.toml).
    pub fn aus_idee(i: usize) -> Taktik9 {
        let k = IDEEN[i.min(IDEEN.len() - 1)];
        let r = crate::regeln::wert("spielideen", &format!("ideen.{k}.regler"));
        let s = crate::regeln::wert("spielideen", &format!("ideen.{k}.schalter"));
        let mut regler = [3u8; 9];
        for (j, x) in r.as_array().into_iter().flatten().enumerate().take(9) { regler[j] = x.as_u64().unwrap_or(3).clamp(1, 5) as u8; }
        let mut schalter = [false; 4];
        for (j, x) in s.as_array().into_iter().flatten().enumerate().take(4) { schalter[j] = x.as_bool().unwrap_or(false); }
        Taktik9 { idee: i as u8, regler, schalter, angepasst: false, aktiv: true }
    }
    /// The sliders that give an existing tactic of the game's switches (saves from before the PES M tactic), so the
    /// first look at the page changes nothing on the pitch.
    pub fn aus_tactic(t: &crate::game::Tactic) -> Taktik9 {
        let mut k = Self::aus_idee(6);
        let halb = |x: i32| ((x + 1) / 2).clamp(1, 5) as u8;
        k.regler = [if t.angriff == 1 { 2 } else { 4 }, if t.aufbau == 0 { 4 } else { 2 }, if t.breite == 0 { 4 } else { 2 }, 3, if t.stellung == 1 { 4 } else { 3 }, if t.abwehr == 0 { 4 } else { 2 }, if t.pressing == 0 { 4 } else { 2 }, halb(t.linie), halb(t.kompakt)];
        k.angepasst = true;
        k
    }
    pub fn r(&self, name: &str) -> u8 { REGLER.iter().position(|x| *x == name).map(|i| self.regler[i]).unwrap_or(3) }
    pub fn s(&self, name: &str) -> bool { SCHALTER.iter().position(|x| *x == name).map(|i| self.schalter[i]).unwrap_or(false) }
}

/// One game value with what it is set to and why (text keys with their placeholders already filled).
#[derive(Clone, Debug)]
pub struct Fl27Wert { pub name: &'static str, pub wert: String, pub grund: String, pub geschrieben: bool }

/// The translation (Konzept 2.2): sliders and switches → the game's tactic switches and sliders. `zielspieler`: a target
/// man is in the eleven (long balls make sense from passweg 3). Writes the switch values into `t` and returns the table.
pub fn uebersetzen(k: &Taktik9, t: &mut crate::game::Tactic, zielspieler: bool) -> Vec<Fl27Wert> {
    let tx = |key: &str| crate::inhalt::t(key).to_string();
    let (tempo, pass, seite, frei, press, haerte, linie, kompakt) = (k.r("tempo"), k.r("passweg"), k.r("seite"), k.r("freiheit"), k.r("pressing"), k.r("haerte"), k.r("linie"), k.r("kompakt")); // roh
    let mut v = vec![];
    let besitz = (tempo <= 2 && pass <= 3) || (tempo == 3 && pass <= 2);
    t.angriff = if besitz { 1 } else { 0 };
    v.push(Fl27Wert { name: "angriff", wert: tx(if besitz { "ui.t9.w.ballbesitz" } else { "ui.t9.w.konter" }), grund: tx(if besitz { "ui.t9.g.ruhig_kurz" } else { "ui.t9.g.schnell" }), geschrieben: false }); // roh
    let lang = pass >= 4 || (zielspieler && pass >= 3);
    t.aufbau = if lang { 0 } else { 1 };
    v.push(Fl27Wert { name: "aufbau", wert: tx(if lang { "ui.t9.w.lang" } else { "ui.t9.w.kurz" }), grund: tx(if lang && pass < 4 { "ui.t9.g.zielspieler" } else if lang { "ui.t9.g.passweg_lang" } else { "ui.t9.g.passweg_kurz" }), geschrieben: false }); // roh
    let aussen = seite >= 4;
    t.breite = if aussen { 0 } else { 1 };
    v.push(Fl27Wert { name: "bereich", wert: tx(if aussen { "ui.t9.w.fluegel" } else { "ui.t9.w.mitte" }), grund: tx(if aussen { "ui.t9.g.ueber_aussen" } else { "ui.t9.g.durch_mitte" }), geschrieben: false }); // roh
    let flexibel = frei >= 4;
    t.stellung = flexible_wert(flexibel);
    v.push(Fl27Wert { name: "stellung", wert: tx(if flexibel { "ui.t9.w.flexibel" } else { "ui.t9.w.halten" }), grund: tx(if flexibel { "ui.t9.g.freie_rollen" } else { "ui.t9.g.ordnung" }), geschrieben: false }); // roh
    let unterst = (2 * (6 - tempo as i32) - (pass as i32 - 3)).clamp(1, 10);
    v.push(Fl27Wert { name: "unterstuetzung", wert: format!("{unterst}/10"), grund: tx(if unterst >= 6 { "ui.t9.g.nah_dran" } else { "ui.t9.g.weit_weg" }), geschrieben: true }); // roh
    let vorne = press >= 4 || k.s("gegenpressing"); // roh
    t.abwehr = if vorne { 0 } else { 1 };
    v.push(Fl27Wert { name: "abwehr", wert: tx(if vorne { "ui.t9.w.vorne" } else { "ui.t9.w.tief" }), grund: tx(if k.s("gegenpressing") { "ui.t9.g.gegenpressing" } else if vorne { "ui.t9.g.hoch_pressen" } else { "ui.t9.g.erwarten" }), geschrieben: false }); // roh
    let mitte = kompakt >= 3;
    t.kontrolle = if mitte { 0 } else { 1 };
    v.push(Fl27Wert { name: "kontrolle", wert: tx(if mitte { "ui.t9.w.mitte" } else { "ui.t9.w.fluegel" }), grund: tx(if mitte { "ui.t9.g.eng" } else { "ui.t9.g.breit" }), geschrieben: false }); // roh
    let aggressiv = haerte >= 4 || press == 5;
    t.pressing = if aggressiv { 0 } else { 1 };
    v.push(Fl27Wert { name: "pressing", wert: tx(if aggressiv { "ui.t9.w.aggressiv" } else { "ui.t9.w.zurueck" }), grund: tx(if aggressiv { "ui.t9.g.hart_rein" } else { "ui.t9.g.stellen" }), geschrieben: false }); // roh
    t.linie = (2 * linie as i32 + if k.s("abseitsfalle") { 1 } else { 0 }).clamp(1, 10); // roh
    v.push(Fl27Wert { name: "linie", wert: format!("{}/10", t.linie), grund: tx(if k.s("abseitsfalle") { "ui.t9.g.abseitsfalle" } else if linie >= 4 { "ui.t9.g.linie_hoch" } else { "ui.t9.g.linie_tief" }), geschrieben: true }); // roh
    t.kompakt = (2 * kompakt as i32).clamp(1, 10);
    v.push(Fl27Wert { name: "kompakt", wert: format!("{}/10", t.kompakt), grund: tx(if kompakt >= 4 { "ui.t9.g.eng" } else { "ui.t9.g.breit" }), geschrieben: true }); // roh
    t.haltung = match tempo + k.r("risiko") { 0..=4 => 0, 5..=7 => 1, _ => 2 }; // roh
    v
}
fn flexible_wert(f: bool) -> u8 { if f { 1 } else { 0 } }
/// The support value (1–10) the translation gives, for writing into Tactics.bin.
pub fn unterstuetzung(k: &Taktik9) -> u8 { (2 * (6 - k.r("tempo") as i32) - (k.r("passweg") as i32 - 3)).clamp(1, 10) as u8 } // roh

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ideen_und_uebersetzung() {
        let mut t = crate::game::Tactic::default();
        // possession: short, patient → possession, short build-up, support close
        let b = Taktik9::aus_idee(1);
        let v = uebersetzen(&b, &mut t, false);
        assert_eq!((t.angriff, t.aufbau), (1, 1));
        assert!(unterstuetzung(&b) >= 8, "{}", unterstuetzung(&b));
        assert_eq!(v.len(), 10);
        assert!(v.iter().all(|x| !x.grund.is_empty() && !x.wert.is_empty()));
        // park the bus: deep, compact, counter
        let m = Taktik9::aus_idee(3);
        uebersetzen(&m, &mut t, false);
        assert_eq!((t.angriff, t.abwehr), (0, 1));
        assert!(t.linie <= 3 && t.kompakt >= 9);
        // gegenpressing switch: defending high even with a medium slider
        let mut g = Taktik9::aus_idee(6);
        g.schalter[0] = true;
        uebersetzen(&g, &mut t, false);
        assert_eq!(t.abwehr, 0);
        // a target man makes a medium passing slider long
        uebersetzen(&Taktik9::aus_idee(6), &mut t, true);
        assert_eq!(t.aufbau, 0);
    }
}

impl crate::game::Career {
    /// Is a target man up front (a striker strong in the air)? Then a medium passing slider plays long.
    pub fn zielspieler_in_elf(&self) -> bool {
        self.xi.iter().any(|&p| { let pl = &self.world.players[p]; pl.pos == "CF" && pl.attrs.get(7).copied().unwrap_or(0.0) >= 80.0 }) // roh
    }
    /// Translate the PES M tactic into the game's values (Tactic), keeping the stance in step.
    pub fn taktik9_anwenden(&mut self) {
        let z = self.zielspieler_in_elf();
        let k = self.t9.clone();
        uebersetzen(&k, &mut self.tactic, z);
    }
}
