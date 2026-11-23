//! Sounding out another club's player before an offer (pesm/core/transfers/sondierung.py): ask the club (answer after a
//! few days: not for sale, only for a high fee, for sale, loan, release clause), talk to player and agent (his interest
//! and wage idea; without asking the club first it counts as tapping up), or show interest (small effects).
use crate::game::*;
use crate::inhalt::t;
use crate::mensch::{cw, EGO, EHRGEIZ, LOYAL};
use crate::mensch::zufall;
use crate::world::*;
use serde::{Deserialize, Serialize};

pub const KATEGORIEN: [&str; 6] = [crate::id("frei"), crate::id("klausel"), crate::id("unverkaeuflich"), crate::id("hohe_abloese"), crate::id("verkaeuflich"), crate::id("leihe")];
pub const HALTUNGEN: [&str; 4] = [crate::id("kein"), crate::id("zoegernd"), crate::id("offen"), crate::id("gross")];
const STIMMUNG: [f32; 4] = [-10.0, 0.0, 5.0, 10.0];
const MAX_TAGE: u32 = 8;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(default)]
pub struct Sondierung {
    pub pid: usize,
    /// 0 asking the club, 1 talk with player and agent, 2 interest shown.
    pub art: u8,
    pub tag: u32,
    pub antwort: u32,
    pub fertig: bool,
    pub kat: u8,
    pub spanne: (f64, f64),
    pub leihe: bool,
    pub stufe: u8,
    pub gehalt: f64,
    pub abwerben: bool,
    pub verstimmt: bool,
}

impl Career {
    /// What the club would answer (its view, not an offer): category, fee range, loan possible.
    pub fn einschaetzung(&self, p: usize) -> (u8, (f64, f64), bool) {
        let pl = &self.world.players[p];
        if pl.team < 0 { return (0, (0.0, 0.0), false); }
        let klausel = pl.profil.klauseln.ausstieg;
        let c = pl.team as usize;
        let bereit = self.verkaufsbereitschaft(p);
        let mut basis = if pl.listed && pl.profil.mindestpreis > 0.0 { pl.profil.mindestpreis } else { self.abloese_forderung(p) };
        // the club moves its idea: importance, cover on the position, contract left, its cash, a little chance per month
        let rang = self.world.clubs[c].squad.iter().filter(|&&q| self.world.players[q].ovr > pl.ovr).count();
        let mut fak = if rang < 3 { 1.10 } else if rang < 11 { 1.04 } else if pl.age <= 21 { 1.02 } else { 1.0 };
        let ersatz = self.world.clubs[c].squad.iter().filter(|&&q| q != p && self.world.players[q].pos == pl.pos && self.world.players[q].ovr >= pl.ovr - 4.0).count();
        fak *= if ersatz == 0 { 1.08 } else if ersatz >= 2 { 0.94 } else { 1.0 };
        if self.monate_rest(p) <= 12.0 { fak *= 0.93; }
        if self.world.clubs[c].balance < 2_000_000.0 { fak *= 0.92; }
        fak *= 0.96 + 0.08 * zufall(&[crate::id("preis"), &(self.abs() / 30).to_string(), &pl.id.to_string()]) as f64;
        basis *= fak;
        let leihe = pl.age <= 23 && rang >= 11;
        let r = |x: f64| round_money(x);
        if klausel > 0.0 { return (1, (klausel, klausel), false); }
        if bereit <= 0.05 { return (if leihe { 5 } else { 2 }, (r(basis * 1.6), r(basis * 2.2)), leihe); }
        if bereit < 0.35 { return (3, (r(basis * 1.25), r(basis * 1.6)), leihe); }
        (4, (r(basis * if pl.listed { 0.97 } else { 0.9 }), r(basis * 1.1)), leihe)
    }
    pub fn sond_letzte(&self, p: usize, art: u8) -> Option<&Sondierung> { self.sondierungen.iter().rev().find(|s| s.pid == p && s.art == art && s.fertig) }
    /// Ask the club: the sporting director's answer comes after a few days (at once for a free agent).
    pub fn sondierung_verein(&mut self, p: usize) -> Result<String, String> {
        let pl = &self.world.players[p];
        if pl.team == self.own as i32 || pl.team == -2 { return Err(t("tv.sond.rueck.nicht_bereit").into()); }
        if self.sondierungen.iter().any(|s| s.pid == p && s.art == 0 && !s.fertig) { return Err(t("tv.sond.rueck.laeuft_schon").into()); }
        let tage = if pl.team < 0 { 0 } else { (2.0 + 4.0 * zufall(&[crate::id("sondierung"), &self.abs().to_string(), &pl.id.to_string()]) + if pl.value > 20e6 { 2.0 } else { 0.0 }).round().min(MAX_TAGE as f32) as u32 };
        let abs = self.abs();
        self.sondierungen.push(Sondierung { pid: p, art: 0, tag: abs, antwort: abs + tage, ..Default::default() });
        if tage == 0 { self.sondierung_tag(); return Ok(t("tv.sond.rueck.sofort").into()); }
        let (n, v) = (pl.name.clone(), self.world.clubs[pl.team as usize].name.clone());
        self.msg(3, &crate::tf!("sd.sportdirektor", v.clone()), &crate::tf!("tv.sond.verein.laeuft_betreff", n.clone()), &crate::tf!("tv.sond.verein.laeuft_text", n, v, tage), Action::None);
        Ok(crate::tf!("tv.sond.rueck.verein", tage))
    }
    /// Daily: due answers of the clubs.
    pub fn sondierung_tag(&mut self) {
        let abs = self.abs();
        for i in 0..self.sondierungen.len() {
            let s = self.sondierungen[i].clone();
            if s.fertig || s.art != 0 || s.antwort > abs { continue; }
            let (kat, spanne, leihe) = self.einschaetzung(s.pid);
            let x = &mut self.sondierungen[i];
            x.fertig = true; x.kat = kat; x.spanne = spanne; x.leihe = leihe;
            let pl = &self.world.players[s.pid];
            let n = pl.name.clone();
            let v = if pl.team >= 0 { self.world.clubs[pl.team as usize].name.clone() } else { String::new() };
            let mut satz = vec![crate::tf!(&format!("tv.sond.verein.text.{}", KATEGORIEN[kat as usize]), n.clone(), v.clone(), money(spanne.0), money(spanne.1))]; // roh
            if leihe && kat != 5 { satz.push(crate::tf!("tv.sond.verein.leihe", n.clone())); }
            if kat != 0 { satz.push(crate::tf!("tv.sond.verein.vertrag", crate::date::START_YEAR as u32 + pl.contract_end + 1)); }
            let da = self.negs.iter().filter(|x| x.pid == s.pid).count() + self.marktstatus.iter().filter(|x| x.p == s.pid).count();
            satz.push(if da == 1 { t("tv.sond.verein.interessenten.eins").into() } else if da > 1 { crate::tf!("tv.sond.verein.interessenten.viele", da) } else { t("tv.sond.verein.keine").into() });
            self.msg(3, &crate::tf!("sd.sportdirektor", v), &crate::tf!("tv.sond.verein.betreff", n), &satz.join(" "), Action::None);
        }
    }
    /// Talk with player and agent: his interest (0 none … 3 big), wage idea; before asking the club it is tapping up.
    pub fn sondierung_spieler(&mut self, p: usize) -> Result<String, String> {
        let pl = &self.world.players[p];
        if pl.team == self.own as i32 || pl.team == -2 { return Err(t("tv.sond.rueck.nicht_bereit").into()); }
        let ruf = (self.club().rep - if pl.team >= 0 { self.world.clubs[pl.team as usize].rep } else { 40.0 }) / 100.0;
        let qualitaet = (0.5 + (self.mgr_rep - 50.0) / 200.0).clamp(0.2, 0.8);
        let rest = self.monate_rest(p);
        let mut punkte = qualitaet * 100.0 + ruf * 30.0 + (cw(pl, EHRGEIZ) - 50.0) * 0.3 - (cw(pl, LOYAL) - 50.0) * 0.3;
        if rest <= 12.0 { punkte += 8.0; }
        let stufe = if punkte >= 70.0 { 3 } else if punkte >= 50.0 { 2 } else if punkte >= 30.0 { 1 } else { 0 };
        let gehalt = round_money(pl.wage.max(1.0) * (1.0 + 0.1 + (cw(pl, EGO) as f64 - 50.0) / 100.0 * 0.1));
        let abwerben = pl.team >= 0 && rest > 6.0 && !pl.listed && self.sond_letzte(p, 0).is_none();
        let abs = self.abs();
        self.sondierungen.push(Sondierung { pid: p, art: 1, tag: abs, antwort: abs, fertig: true, stufe, gehalt, abwerben, ..Default::default() });
        let n = pl.name.clone();
        if abwerben {
            let c = pl.team as usize;
            let v = self.world.clubs[c].name.clone();
            self.verein_verstimmen(c);
            self.msg(2, &crate::tf!("sd.sportdirektor", v.clone()), &crate::tf!("tv.sond.abwerben.betreff", n.clone()), &crate::tf!("tv.sond.abwerben.text", n.clone(), v), Action::None);
        }
        Ok(crate::tf!("tv.sond.status.spieler", t(&format!("tv.sond.haltung.{}", HALTUNGEN[stufe as usize])), money(gehalt), t(if self.world.players[p].ovr >= self.club().level { "tv.sond.rolle.stammspieler" } else if self.world.players[p].age <= 21 { "tv.sond.rolle.perspektive" } else { "tv.sond.rolle.rotation" }))) // roh
    }
    fn verein_verstimmen(&mut self, c: usize) {
        if let Some(x) = self.mensch.vereine.iter_mut().find(|x| x.0 == c) { x.1 = (x.1 - 1.0).max(-3.0); } else { self.mensch.vereine.push((c, -1.0)); }
    }
    /// Show interest: the player warms to the idea; a club that does not want to sell takes it badly; maybe a rumour.
    pub fn sondierung_interesse(&mut self, p: usize) -> Result<String, String> {
        let abs = self.abs();
        if self.sondierungen.iter().any(|s| s.pid == p && s.art == 2 && s.tag + 14 > abs) { return Err(t("tv.sond.rueck.schon_bekundet").into()); }
        let (kat, _, _) = self.einschaetzung(p);
        let pl = &self.world.players[p];
        let verstimmt = (kat == 2 || kat == 3) && !pl.listed;
        if verstimmt && pl.team >= 0 { let c = pl.team as usize; self.verein_verstimmen(c); }
        self.world.players[p].ps.wb = (self.world.players[p].ps.wb + 5.0).min(100.0);
        self.sondierungen.push(Sondierung { pid: p, art: 2, tag: abs, antwort: abs, fertig: true, kat, verstimmt, ..Default::default() });
        if zufall(&[crate::id("interesse_geruecht"), &abs.to_string(), &self.world.players[p].id.to_string()]) < 0.5 {
            let (n, cn) = (self.world.players[p].name.clone(), self.club().name.clone());
            self.news(&crate::tf!("lg.welt.ki_geruecht", n, cn), t("tv.sond.aktion.interesse"));
        }
        Ok(t("tv.sond.rueck.interesse").into())
    }
    /// The player's mood towards us from talk and interest (−20 … +20): starts the later negotiation.
    pub fn sond_stimmung(&self, p: usize) -> f32 {
        self.sondierungen.iter().filter(|s| s.pid == p && s.fertig).map(|s| match s.art { 1 => STIMMUNG[s.stufe as usize], 2 => 5.0, _ => 0.0 }).sum::<f32>().clamp(-20.0, 20.0)
    }
    /// One line of what is known for the market panel.
    pub fn sond_status(&self, p: usize) -> Option<String> {
        if let Some(s) = self.sondierungen.iter().rev().find(|s| s.pid == p && s.art == 0 && !s.fertig) { return Some(crate::tf!("tv.sond.rueck.verein", s.antwort.saturating_sub(self.abs()))); }
        let v = self.sond_letzte(p, 0).map(|s| crate::tf!("tv.sond.status.antwort", crate::tf!(&format!("tv.sond.kat.{}", KATEGORIEN[s.kat as usize]), money(s.spanne.0), money(s.spanne.1)))); // roh
        let h = self.sond_letzte(p, 1).map(|s| t(&format!("tv.sond.haltung.{}", HALTUNGEN[s.stufe as usize])).to_string()); // roh
        match (v, h) { (Some(a), Some(b)) => Some(format!("{a} · {b}")), (a, b) => a.or(b) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn anfragen_sprechen_interesse() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let mut c = Career::new(w, 0, "Test");
        let andere = (0..c.world.clubs.len()).find(|&x| x != c.own && c.world.clubs[x].squad.len() > 15).unwrap();
        let p = c.world.clubs[andere].squad[0];
        // talking first is tapping up: the club is upset
        c.world.players[p].contract_end = c.world.season + 3;
        c.sondierung_spieler(p).unwrap();
        assert!(c.mensch.vereine.iter().any(|x| x.0 == andere && x.1 < 0.0));
        c.sondierung_verein(p).unwrap();
        assert!(c.sondierung_verein(p).is_err(), "one request at a time");
        for _ in 0..10 { c.world.day += 1; c.sondierung_tag(); }
        assert!(c.sond_letzte(p, 0).is_some() && c.sond_status(p).is_some());
        c.sondierung_interesse(p).unwrap();
        assert!(c.sondierung_interesse(p).is_err(), "not twice in two weeks");
    }
}
