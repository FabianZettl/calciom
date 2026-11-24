//! A player's contract request answered with a real choice (pesm/core/gespraeche/anfrage.py, gespraeche.toml [anfrage]):
//! talk now, promise with a condition (goals, starts or a rating in the next matches), put him off, or say no. The player
//! accepts the condition, asks something for it, or refuses, by character; the condition is checked match by match.
use crate::game::*;
use crate::inhalt::t;
use crate::mensch::{cw, EGO, LOYAL};
use crate::mensch::zufall;
use crate::regeln::f;
use serde::{Deserialize, Serialize};

pub const ARTEN: [&str; 3] = [crate::id("bedingung_tore"), crate::id("bedingung_einsaetze"), crate::id("bedingung_note")];
const ANGRIFF: [&str; 7] = [crate::id("CF"), crate::id("SS"), crate::id("AMF"), crate::id("LWF"), crate::id("RWF"), crate::id("LMF"), crate::id("RMF")];

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Bedingung { pub pid: usize, pub art: u8, pub ziel: f32, pub spiele: u32, pub start_spiele: usize, pub tore0: u16, pub apps0: u16, pub rsum0: f32, pub rn0: u16, pub frist: u32, pub mehr_gehalt: bool }

fn a(k: &str) -> f32 { f("gespraeche", &format!("anfrage.{k}")) } // roh

impl Career {
    /// The condition fitting the player: goals for forwards, a rating for regulars, starts for the others.
    pub fn bedingung_vorschlag(&self, p: usize) -> (u8, f32, u32) {
        let pl = &self.world.players[p];
        let spiele = a(crate::id("spiele")) as u32;
        if ANGRIFF.contains(&pl.pos.as_str()) {
            let rate = crate::regeln::wert("gespraeche", "anfrage.tore_rate")[crate::world::pos_de(&pl.pos)].as_f64().unwrap_or(0.2) as f32;
            return (0, (rate * spiele as f32 * a("tore_faktor")).ceil().max(1.0), spiele);
        }
        if self.xi.contains(&p) { return (2, a("note_ziel"), spiele); }
        (1, (spiele as f32 * 0.6).ceil(), spiele)
    }
    pub fn bedingung_text(art: u8, ziel: f32, spiele: u32) -> String { crate::tf!(&format!("gs.anfrage.bed.{}", ARTEN[art as usize]), if art == 2 { format!("{ziel:.1}") } else { format!("{ziel:.0}") }, spiele) } // roh
    /// „Promise with a condition“: he accepts, asks for more wage in return, or refuses.
    pub fn bedingung_stellen(&mut self, p: usize) -> String {
        let (art, ziel, spiele) = self.bedingung_vorschlag(p);
        let pl = &self.world.players[p];
        let n = pl.name.clone();
        let schwer = match art { 0 => (ziel / 4.0).min(1.0), 2 => ((ziel - 6.0) / 1.5).clamp(0.0, 1.0), _ => 0.5 };
        let akz = a("akzeptanz_basis") + a("akzeptanz_loyal") * (cw(pl, LOYAL) / 100.0 - 0.5) * 2.0 * 0.5 + a("akzeptanz_ego") * (cw(pl, EGO) / 100.0 - 0.5).max(0.0) * 2.0 + a("akzeptanz_schwer") * schwer + a("akzeptanz_moral") * (pl.morale - 50.0) / 100.0;
        let u = zufall(&[crate::id("anfrage"), &pl.id.to_string(), &self.abs().to_string()]);
        let satz = crate::tf!(&format!("gs.anfrage.satz.{}", ARTEN[art as usize]), n.clone(), if art == 2 { format!("{ziel:.1}") } else { format!("{ziel:.0}") }, spiele); // roh
        if u >= akz + a("gegen_spanne") {
            self.beziehung(p, -2.0, t("gs.anfrage.opt.bedingung"));
            return format!("{satz}\n{}", crate::tf!("gs.anfrage.antwort.ablehnen", n));
        }
        let mehr = u >= akz;
        let b = Bedingung { pid: p, art, ziel, spiele, start_spiele: self.results.len(), tore0: pl.goals, apps0: pl.apps, rsum0: pl.rsum, rn0: pl.rn, frist: self.abs() + spiele * a("frist_tage_je_spiel") as u32, mehr_gehalt: mehr };
        self.bedingungen.retain(|x| x.pid != p);
        self.bedingungen.push(b);
        let antwort = if mehr { crate::tf!("gs.anfrage.antwort.gegen", n, t("gs.anfrage.forderung.gehalt")) } else { crate::tf!("gs.anfrage.antwort.annehmen", n) };
        format!("{satz}\n{antwort}")
    }
    /// „Not now“: he comes back in a few weeks; sensitive players take it a little badly.
    pub fn anfrage_spaeter(&mut self, p: usize) -> String {
        self.beziehung(p, a("spaeter_beziehung"), t("gs.anfrage.opt.spaeter"));
        let n = self.world.players[p].name.clone();
        crate::tf!("gs.anfrage.antwort.spaeter", n)
    }
    /// Daily: conditions met (the talks are ready) or missed (he is upset); the counting starts with the season's matches.
    pub fn bedingungen_pruefen(&mut self) {
        let abs = self.abs();
        let alle = std::mem::take(&mut self.bedingungen);
        let mut bleiben = vec![];
        for b in alle {
            let pl = &self.world.players[b.pid];
            if pl.team != self.own as i32 { continue; }
            // the season's figures restart in summer: a condition across the change counts from zero
            let (tore, apps) = (pl.goals.saturating_sub(b.tore0) as f32, pl.apps.saturating_sub(b.apps0) as f32);
            let rn = pl.rn.saturating_sub(b.rn0);
            let note = if rn > 0 { (pl.rsum - b.rsum0) / rn as f32 } else { 0.0 };
            let gespielt = self.results.len().saturating_sub(b.start_spiele) as u32;
            let erfuellt = match b.art { 0 => tore >= b.ziel, 1 => apps >= b.ziel, _ => rn as f32 >= a("note_min_spiele") && note >= b.ziel && gespielt >= b.spiele };
            let vorbei = gespielt >= b.spiele || abs > b.frist;
            let n = pl.name.clone();
            if erfuellt {
                if b.mehr_gehalt { self.world.players[b.pid].ps.anfrage_plus = a("gehalt_plus"); }
                self.msg(1, &n, &crate::tf!("gs.vorfall.anfrage_bereit.betreff", n.clone()), &crate::tf!("gs.vorfall.anfrage_bereit.text", n.clone()), Action::Expiring { pid: b.pid });
                self.beziehung(b.pid, 2.0, t("gs.anfrage.antwort.erfuellt"));
            } else if vorbei {
                self.moral_wirkung(b.pid, a("verfehlt_moral"), 0, t("gs.anfrage.verfehlt.betreff"));
                self.beziehung(b.pid, a("verfehlt_beziehung"), t("gs.anfrage.verfehlt.betreff"));
                self.msg(2, &n, &crate::tf!("gs.vorfall.anfrage_verfehlt.betreff", n.clone()), &crate::tf!("gs.vorfall.anfrage_verfehlt.text", n), Action::None);
            } else { bleiben.push(b); }
        }
        self.bedingungen = bleiben;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zusage_mit_bedingung() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let p = c.squad().into_iter().find(|&p| ANGRIFF.contains(&c.world.players[p].pos.as_str())).unwrap();
        let (art, ziel, _) = c.bedingung_vorschlag(p);
        assert_eq!(art, 0, "a forward gets a goals condition");
        let r = c.bedingung_stellen(p);
        assert!(!r.is_empty());
        if let Some(i) = c.bedingungen.iter().position(|b| b.pid == p) {
            c.world.players[p].goals += ziel as u16;
            c.bedingungen_pruefen();
            assert!(c.bedingungen.is_empty() && matches!(c.inbox[0].action, Action::Expiring { .. }), "met: the talks are ready {i}");
        }
    }
}
