//! Vertragsklauseln (Punkt 6): release clause, bonuses per goal and per match, a yearly raise, and for sales a sell-on
//! share and a buy-back option. Negotiated in the round model (their worth counts like part of the wage, or for a buyer
//! like part of the fee), shown in the contract sheet and the profile, and they work: bonuses are paid after each match,
//! the raise comes each season, other clubs can pay the release clause, a sale passes the share on, a buy-back can be
//! drawn. Numbers in assets/regeln/klauseln.toml (from the original where it has them).
use crate::game::*;
use crate::world::{money, round_money};
use serde::{Deserialize, Serialize};

fn f(k: &str) -> f32 { crate::regeln::f("klauseln", k) }
fn stufe(k: &str, i: u8) -> f64 { crate::mensch::arr("klauseln", k).get(i as usize).and_then(|v| v.as_f64()).unwrap_or(0.0) }

/// Clauses of a contract (also the choice in a negotiation: the steps of klauseln.toml).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Klauseln {
    /// Release clause: the fee another club may pay (0 = none).
    pub ausstieg: f64,
    pub praemie_tor: f64,
    pub praemie_einsatz: f64,
    /// Bonus per title the club wins (paid at the end of the season).
    pub praemie_titel: f64,
    /// Raise of the wage at each new season of the contract (0.05 = +5 %).
    pub steigerung: f32,
    /// Sell-on: the club that gets a share of the next sale, and the share in per cent.
    pub weiterverkauf: Option<(usize, u8)>,
    /// Buy-back: the club that may buy the player back, the fee, until which season.
    pub rueckkauf: Option<(usize, f64, u32)>,
}
/// The choice in a negotiation as step numbers (release, goal bonus, match bonus, raise, sell-on, buy-back).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Wahl { pub ausstieg: u8, pub tor: u8, pub einsatz: u8, pub steigerung: u8, pub weiterverkauf: u8, pub rueckkauf: u8, pub titel: u8 }

impl Career {
    /// Does the player want a release clause? (werte.py: ego from 0.7 or the trait „selbstbewusst“)
    pub fn will_ausstieg(&self, p: usize) -> bool {
        let pl = &self.world.players[p];
        crate::mensch::cw(pl, crate::mensch::EGO) >= f("ausstieg.ego_ab") || crate::entwicklung::merkmale_schaetzen(pl).iter().any(|m| m == crate::id("selbstbewusst"))
    }
    /// What the chosen clauses are worth to the player per year, in wage (werte.py): bonuses by what he expects to score
    /// and play, the release clause by his ambition (half when it is so high nobody pays it), a raise by its average over
    /// the contract; a missing release clause he wants counts against.
    pub fn klausel_wert_spieler(&self, p: usize, w: &Wahl, wage: f64, years: u32) -> f64 {
        let pl = &self.world.players[p];
        let tore = crate::regeln::wert("klauseln", &format!("praemie.erwartung_tore.{}", pl.pos)).as_f64().unwrap_or(2.0);
        let mut v = stufe("praemie.tor_stufen", w.tor) * wage * tore + stufe("praemie.einsatz_stufen", w.einsatz) * wage * f("praemie.erwartung_einsaetze") as f64
            + stufe("praemie.titel_stufen", w.titel) * wage * f("praemie.erwartung_titel") as f64;
        let ehrgeiz = crate::mensch::cw(pl, crate::mensch::EHRGEIZ) as f64 / 100.0;
        let a = stufe("ausstieg.stufen", w.ausstieg);
        if a > 0.0 { v += wage * f("ausstieg.wert") as f64 * if a > f("ausstieg.sicher_ab") as f64 { 0.5 } else { 1.0 } * (0.5 + ehrgeiz); }
        else if self.will_ausstieg(p) { v -= wage * f("ausstieg.fehlt") as f64; }
        v += wage * stufe("steigerung.stufen", w.steigerung) * (years.max(1) - 1) as f64 / 2.0;
        v
    }
    /// What the sell-on share and the buy-back option cost a buyer (transfer stage of a sale), as fee.
    pub fn klausel_kosten_kaeufer(&self, p: usize, w: &Wahl, fee: f64) -> f64 {
        let mw = self.world.players[p].value;
        let mut k = stufe("weiterverkauf.stufen", w.weiterverkauf) / 100.0 * mw * f("weiterverkauf.erwartung") as f64;
        let r = stufe("rueckkauf.stufen", w.rueckkauf) * mw;
        if r > 0.0 { k += (mw * f("rueckkauf.basis") as f64 - r).max(0.0) * f("rueckkauf.erwartung") as f64 + mw * f("rueckkauf.grund") as f64; }
        let _ = fee;
        k
    }
    /// The clauses' worth in the units of the round model for negotiation i: + for the player (wage), − for a buyer.
    pub fn klausel_kv(&self, n: &Neg) -> f64 {
        if n.verkauf { -self.klausel_kosten_kaeufer(n.pid, &n.kl, n.fee) } else if n.stage == 1 { self.klausel_wert_spieler(n.pid, &n.kl, n.wage, n.years) } else { 0.0 }
    }
    /// The signed contract takes the clauses (purchase or renewal; for a sale the buyer's club holds them for us).
    pub fn klauseln_unterschrift(&mut self, n: &Neg) {
        let season = self.world.season;
        let mw = self.world.players[n.pid].value;
        let wage = n.wage;
        let k = &mut self.world.players[n.pid].profil.klauseln;
        if n.verkauf {
            let pct = stufe("weiterverkauf.stufen", n.kl.weiterverkauf) as u8;
            k.weiterverkauf = (pct > 0).then_some((self.own, pct));
            let r = stufe("rueckkauf.stufen", n.kl.rueckkauf) * mw;
            k.rueckkauf = (r > 0.0).then(|| (self.own, round_money(r), season + f("rueckkauf.saisons") as u32));
            k.ausstieg = 0.0; k.praemie_tor = 0.0; k.praemie_einsatz = 0.0; k.praemie_titel = 0.0; k.steigerung = 0.0;
        } else {
            k.ausstieg = round_money(stufe("ausstieg.stufen", n.kl.ausstieg) * mw);
            k.praemie_tor = round_money(stufe("praemie.tor_stufen", n.kl.tor) * wage);
            k.praemie_einsatz = round_money(stufe("praemie.einsatz_stufen", n.kl.einsatz) * wage);
            k.praemie_titel = round_money(stufe("praemie.titel_stufen", n.kl.titel) * wage);
            k.steigerung = stufe("steigerung.stufen", n.kl.steigerung) as f32;
        }
    }
    /// After an own match: bonuses for goals and matches of own players with such clauses.
    pub fn praemien_nach_spiel(&mut self, res: &crate::world::MatchResult) {
        let mut summe = 0.0;
        for &(p, _) in &res.minutes { if self.world.players[p].team == self.own as i32 { summe += self.world.players[p].profil.klauseln.praemie_einsatz; } }
        for e in res.events.iter().filter(|e| e.kind == 0) { if self.world.players[e.pid].team == self.own as i32 { summe += self.world.players[e.pid].profil.klauseln.praemie_tor; } }
        if summe > 0.0 { self.book(crate::inhalt::t("ui.klausel.buchung_praemien"), -summe); }
    }
    /// New season: title bonuses for the titles of the season just ended, the raise of every own player whose contract
    /// runs on.
    pub fn klauseln_saison(&mut self) {
        let season = self.world.season;
        let titel = self.world.titles.iter().filter(|t| t.club == self.own && t.season + 1 >= season).count() as f64;
        let summe: f64 = self.squad().iter().map(|&p| self.world.players[p].profil.klauseln.praemie_titel * titel).sum();
        if summe > 0.0 { self.book(crate::inhalt::t("ui.klausel.buchung_praemien"), -summe); }
        for p in self.squad() {
            let pl = &mut self.world.players[p];
            if pl.profil.klauseln.steigerung > 0.0 && pl.contract_end >= season { pl.wage = round_money(pl.wage * (1.0 + pl.profil.klauseln.steigerung as f64)); }
            if pl.contract_end < season { pl.profil.klauseln.ausstieg = 0.0; pl.profil.klauseln.praemie_tor = 0.0; pl.profil.klauseln.praemie_einsatz = 0.0; pl.profil.klauseln.praemie_titel = 0.0; pl.profil.klauseln.steigerung = 0.0; }
        }
    }
    /// Weekly in an open window: a club that can pay the release clause and offers a sporting step may pay it; the player
    /// goes, the coach can't stop it.
    pub fn ausstieg_woche(&mut self) {
        let abs = self.abs();
        let woche = (abs / 7).to_string();
        // a player outgrows his release clause (FC-26 wish): a warning while there is still time to raise it
        let season = self.world.season;
        for p in self.squad() {
            let (a, value, id) = { let pl = &self.world.players[p]; (pl.profil.klauseln.ausstieg, pl.value, pl.id) };
            if a <= 0.0 || value * f("ausstieg.zieh_bis") as f64 * f("ausstieg.warnen_ab") as f64 <= a { continue; }
            if self.mensch.story_erzaehlt.iter().any(|x| x.0 == "warn_klausel" && x.2 == id && x.1 / crate::date::SEASON_DAYS == season) { continue; } // roh
            self.mensch.story_erzaehlt.push(("warn_klausel".into(), abs, id)); // roh
            let n = self.world.players[p].name.clone();
            let opts = vec![("verlaengern".to_string(), crate::inhalt::t("vv.warn.verlaengern").to_string()), ("lassen".to_string(), crate::inhalt::t("vv.warn.lassen").to_string())]; // roh
            self.msg(1, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.klausel.warn_titel", n.clone()), &crate::tf!("ui.klausel.warn_text", n, money(a), money(value)), Action::Wahl { art: "vvwarn:k".into(), bezug: vec![p], optionen: opts }); // roh
        }
        if self.ohne_ki_klauseln { return; }
        for p in self.squad() {
            let (a, value, ovr, id) = { let pl = &self.world.players[p]; (pl.profil.klauseln.ausstieg, pl.value, pl.ovr, pl.id) };
            if a <= 0.0 || a > value * f("ausstieg.zieh_bis") as f64 { continue; }
            if crate::mensch::zufall(&[crate::id("ausstieg"), &id.to_string(), &woche]) >= f("ausstieg.chance_woche") { continue; }
            let eigen = self.club().level;
            let Some(k) = (0..self.world.clubs.len()).filter(|&k| k != self.own && self.world.erste_von(k).is_none() && self.world.clubs[k].level > eigen && self.world.clubs[k].balance >= a && (self.world.clubs[k].level - ovr).abs() <= 6.0)
                .max_by(|x, y| self.world.clubs[*x].level.partial_cmp(&self.world.clubs[*y].level).unwrap()) else { continue };
            let (n, kn) = (self.world.players[p].name.clone(), self.world.clubs[k].name.clone());
            self.sell(p, k, a);
            self.msg(2, &kn, &crate::tf!("ui.klausel.ausstieg_titel", n), &crate::tf!("ui.klausel.ausstieg_text", kn, money(a), n), Action::None);
        }
    }
    /// Release clauses of other clubs' players (fl/sim transfers.json markt.clause): at the world start a share of them gets
    /// one at market value × 1.7, so do a share of the AI's renewals. Each player decided once by his id.
    pub fn ki_klauseln_saeen(&mut self) {
        if self.klauseln_gesaet { return; }
        self.klauseln_gesaet = true;
        for p in 0..self.world.players.len() {
            let pl = &self.world.players[p];
            if pl.team < 0 || pl.team == self.own as i32 || pl.youth > 0 { continue; }
            if crate::mensch::zufall(&[crate::id("klausel_welt"), &pl.id.to_string()]) < f("ki.anteil_welt") { let v = round_money(pl.value * f("ki.faktor") as f64); self.world.players[p].profil.klauseln.ausstieg = v; }
        }
    }
    /// An AI club renews a contract: with a share it carries a release clause.
    pub fn ki_verlaengerung_klausel(&mut self, p: usize) {
        let id = self.world.players[p].id.to_string();
        let s = self.world.season.to_string();
        if crate::mensch::zufall(&[crate::id("klausel_ki"), &id, &s]) < f("ki.anteil_verlaengerung") {
            self.world.players[p].profil.klauseln.ausstieg = round_money(self.world.players[p].value * f("ki.faktor") as f64);
        }
    }
    /// Weekly in an open window: AI clubs pay release clauses of other AI clubs' players (a sporting step, the money
    /// there, at most `ki.je_woche` a week); the buying club takes the clause's fee from its balance.
    pub fn ki_ausstieg_woche(&mut self) {
        self.ki_klauseln_saeen(); // new careers: once, before the first window week
        let woche = (self.abs() / 7).to_string();
        let mut n = 0;
        for p in 0..self.world.players.len() {
            if n >= f("ki.je_woche") as usize { break; }
            let (a, value, ovr, team, id) = { let pl = &self.world.players[p]; (pl.profil.klauseln.ausstieg, pl.value, pl.ovr, pl.team, pl.id) };
            if a <= 0.0 || team < 0 || team == self.own as i32 || a > value * f("ausstieg.zieh_bis") as f64 { continue; }
            if crate::mensch::zufall(&[crate::id("ausstieg_ki"), &id.to_string(), &woche]) >= f("ki.chance_woche") { continue; }
            let von = team as usize;
            let lv = self.world.clubs[von].level;
            let Some(k) = (0..self.world.clubs.len()).filter(|&k| k != self.own && k != von && self.world.erste_von(k).is_none() && self.world.clubs[k].level > lv && self.world.clubs[k].balance >= a && (self.world.clubs[k].level - ovr).abs() <= 6.0 && self.world.clubs[k].squad.len() < 32)
                .max_by(|x, y| self.world.clubs[*x].level.partial_cmp(&self.world.clubs[*y].level).unwrap()) else { continue };
            self.weiterverkauf_abrechnen(p, von, a);
            self.world.clubs[von].squad.retain(|&x| x != p);
            self.world.clubs[von].balance += a;
            self.world.clubs[k].squad.push(p);
            self.world.clubs[k].balance -= a;
            let season = self.world.season;
            let lk = self.world.leagues[self.world.clubs[k].league].level;
            let pl = &mut self.world.players[p];
            pl.team = k as i32;
            pl.contract_end = season + crate::world::contract_years(pl.age) - 1;
            pl.wage = crate::world::fair_wage(pl, lk);
            pl.profil.klauseln.ausstieg = 0.0;
            let (nm, kn, vn) = (pl.name.clone(), self.world.clubs[k].name.clone(), self.world.clubs[von].name.clone());
            if ovr >= 78.0 { self.news(&crate::tf!("ui.klausel.ki_titel", nm, kn), &crate::tf!("ui.klausel.ki_text", kn, money(a), nm, vn)); }
            n += 1;
        }
    }
    /// A sale between other clubs or ours: the club with a sell-on share gets its part (for us booked with a message).
    pub fn weiterverkauf_abrechnen(&mut self, p: usize, verkaeufer: usize, fee: f64) {
        let Some((club, pct)) = self.world.players[p].profil.klauseln.weiterverkauf else { return };
        self.world.players[p].profil.klauseln.weiterverkauf = None;
        if club == verkaeufer { return; }
        let anteil = round_money(fee * pct as f64 / 100.0);
        self.world.clubs[verkaeufer].balance -= anteil;
        if club == self.own {
            self.book(crate::inhalt::t("ui.klausel.buchung_weiterverkauf"), anteil);
            let (n, vk) = (self.world.players[p].name.clone(), self.world.clubs[verkaeufer].name.clone());
            self.msg(3, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.klausel.weiterverkauf_titel", n), &crate::tf!("ui.klausel.weiterverkauf_text", vk, n, money(fee), pct, money(anteil)), Action::None);
        } else { self.world.clubs[club].balance += anteil; }
    }
    /// Draw the buy-back option: the player comes back for the agreed fee (Err with the reason when it can't be done).
    pub fn rueckkauf_ziehen(&mut self, p: usize) -> Result<String, String> {
        let Some((club, fee, bis)) = self.world.players[p].profil.klauseln.rueckkauf else { return Err(crate::inhalt::t("ui.klausel.keine_option").into()) };
        if club != self.own || self.world.season > bis { return Err(crate::inhalt::t("ui.klausel.keine_option").into()); }
        if self.world.players[p].team < 0 || self.world.players[p].team == self.own as i32 { return Err(crate::inhalt::t("ui.klausel.keine_option").into()); }
        if self.club().balance < fee { return Err(crate::tf!("ui.klausel.zu_teuer", money(fee))); }
        let von = self.world.players[p].team as usize;
        self.world.clubs[von].squad.retain(|&x| x != p);
        self.world.clubs[von].balance += fee;
        self.world.clubs[self.own].squad.push(p);
        self.book(crate::inhalt::t("ui.klausel.buchung_rueckkauf"), -fee);
        let lv = self.world.leagues[self.club().league].level;
        let season = self.world.season;
        let pl = &mut self.world.players[p];
        pl.team = self.own as i32;
        pl.contract_end = season + 2;
        pl.wage = crate::world::fair_wage(pl, lv);
        pl.profil.klauseln = Klauseln::default();
        let n = self.world.players[p].name.clone();
        Ok(crate::tf!("ui.klausel.rueckkauf_ok", n, money(fee)))
    }
    /// The clauses of a contract as lines for the contract sheet and the profile (empty when there are none).
    pub fn klausel_zeilen(&self, p: usize) -> Vec<(String, String)> {
        let k = &self.world.players[p].profil.klauseln;
        let mut z = vec![];
        let t = crate::inhalt::t;
        if k.ausstieg > 0.0 { z.push((t("ui.klausel.ausstieg").to_string(), money(k.ausstieg))); }
        if k.praemie_tor > 0.0 { z.push((t("ui.klausel.tor").to_string(), crate::tf!("ui.klausel.je_tor", money(k.praemie_tor)))); }
        if k.praemie_titel > 0.0 { z.push((t("ui.klausel.titelpraemie").to_string(), crate::tf!("ui.klausel.je_titel", money(k.praemie_titel)))); }
        if k.praemie_einsatz > 0.0 { z.push((t("ui.klausel.einsatz").to_string(), crate::tf!("ui.klausel.je_spiel", money(k.praemie_einsatz)))); }
        if k.steigerung > 0.0 { z.push((t("ui.klausel.steigerung").to_string(), crate::tf!("ui.klausel.je_saison", format!("{:.0}", k.steigerung * 100.0)))); }
        if let Some((c, pct)) = k.weiterverkauf { z.push((t("ui.klausel.weiterverkauf").to_string(), crate::tf!("ui.klausel.fuer", format!("{pct}"), self.world.clubs[c].short.clone()))); }
        if let Some((c, fee, bis)) = k.rueckkauf { z.push((t("ui.klausel.rueckkauf").to_string(), crate::tf!("ui.klausel.rueckkauf_wert", money(fee), self.world.clubs[c].short.clone(), crate::date::season_label(bis)))); }
        z
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn karriere() -> Career {
        let w = crate::world::World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(0); // roh
        Career::new(w, own, "Test") // roh
    }
    #[test]
    fn klauseln_wert_wirkung() {
        let mut c = karriere();
        let p = c.squad()[3];
        let wage = 1_000_000.0;
        let ohne = c.klausel_wert_spieler(p, &Wahl::default(), wage, 3);
        let mit = c.klausel_wert_spieler(p, &Wahl { ausstieg: 1, tor: 2, einsatz: 2, steigerung: 1, ..Default::default() }, wage, 3);
        assert!(mit > ohne + 50_000.0, "clauses are worth something to him: {ohne} → {mit}");
        let hoch = c.klausel_wert_spieler(p, &Wahl { ausstieg: 3, ..Default::default() }, wage, 3);
        let niedrig = c.klausel_wert_spieler(p, &Wahl { ausstieg: 1, ..Default::default() }, wage, 3);
        assert!(niedrig > hoch, "a clause nobody pays is worth less");
        assert!(c.klausel_kosten_kaeufer(p, &Wahl { weiterverkauf: 2, rueckkauf: 1, ..Default::default() }, 1e6) > 0.0);
        // bonuses are paid after a match, the raise comes with the season
        c.world.players[p].profil.klauseln = Klauseln { praemie_einsatz: 10_000.0, praemie_tor: 50_000.0, steigerung: 0.1, ..Default::default() };
        let res = crate::world::MatchResult { h: c.own, a: c.own, minutes: vec![(p, 90)], events: vec![crate::world::MatchEvent { min: 10, side: 0, kind: 0, pid: p, pid2: None }], ..Default::default() };
        let k0 = c.club().balance;
        c.praemien_nach_spiel(&res);
        assert!((k0 - c.club().balance - 60_000.0).abs() < 1.0, "one match and one goal");
        let w0 = c.world.players[p].wage;
        c.world.players[p].contract_end = c.world.season + 2;
        c.klauseln_saison();
        assert!(c.world.players[p].wage > w0 * 1.09);
        // title bonus at the season's change, release clauses of the AI's players
        c.world.players[p].profil.klauseln.praemie_titel = 100_000.0;
        c.world.titles.push(crate::cups::Title { season: c.world.season, comp: "x".into(), club: c.own });
        let k2 = c.club().balance;
        c.klauseln_saison();
        assert!(k2 - c.club().balance >= 100_000.0 - 1.0, "title bonus paid");
        c.klauseln_gesaet = false;
        c.ki_klauseln_saeen();
        let mit = c.world.players.iter().filter(|x| x.team >= 0 && x.team != c.own as i32 && x.profil.klauseln.ausstieg > 0.0).count();
        let alle = c.world.players.iter().filter(|x| x.team >= 0 && x.team != c.own as i32 && x.youth == 0).count();
        assert!(mit as f32 > alle as f32 * 0.12 && (mit as f32) < alle as f32 * 0.28, "{mit} of {alle} with a clause");
        // a sell-on share comes back to us when the buyer sells on
        let fremd = (0..c.world.clubs.len()).find(|&k| k != c.own && c.world.erste_von(k).is_none()).unwrap();
        c.world.players[p].profil.klauseln.weiterverkauf = Some((c.own, 20));
        let k1 = c.club().balance;
        c.weiterverkauf_abrechnen(p, fremd, 10_000_000.0);
        assert!((c.club().balance - k1 - 2_000_000.0).abs() < 1.0);
        assert!(c.world.players[p].profil.klauseln.weiterverkauf.is_none(), "paid once");
    }
}
