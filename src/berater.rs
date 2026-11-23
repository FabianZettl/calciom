//! Berater and hidden priorities (Konzept Transfers 2.2, 2.5, 2.7): every player has six weights (money, playing
//! time, security, success, freedom, home) that add up to 100, drawn from his character and age; the person sees them
//! only as a rough hint (with people skills or a scout). Most players have an adviser: a person with a type and a
//! memory (the relation to the club after earlier deals). Numbers in assets/regeln/berater.toml.
use crate::game::*;
use crate::mensch::{arr, cw, zufall, EHRGEIZ, EGO, LOYAL};
use crate::world::Player;

fn f(k: &str) -> f32 { crate::regeln::f("berater", k) }
fn liste(k: &str) -> Vec<f32> { crate::mensch::arr("berater", k).iter().filter_map(|x| x.as_f64()).map(|x| x as f32).collect() }

pub const PRIO: [&str; 6] = [crate::id("geld"), crate::id("spielzeit"), crate::id("sicherheit"), crate::id("erfolg"), crate::id("freiheit"), crate::id("heimat")];
pub const TYPEN: [&str; 5] = [crate::id("keiner"), crate::id("grossagentur"), crate::id("schlitzohr"), crate::id("familie"), crate::id("serioes")];

/// The six weights of a player (sum 100): the greedy and self-centred want money, the ambitious young want playing time
/// and success, the older and the loyal security and home; a fixed random part per player.
pub fn prioritaeten(p: &Player) -> [f32; 6] {
    let a = p.age as f32;
    let id = p.id.to_string();
    let z = |k: &str| zufall(&[crate::id("prio"), &id, k]) * 12.0;
    let roh = [
        10.0 + cw(p, EGO) / 5.0 + z(PRIO[0]),
        8.0 + cw(p, EHRGEIZ) / 6.0 + (26.0 - a).max(0.0) * 1.5 + z(PRIO[1]),
        6.0 + (a - 26.0).max(0.0) * 2.5 + z(PRIO[2]),
        6.0 + cw(p, EHRGEIZ) / 5.0 + z(PRIO[3]),
        4.0 + (100.0 - cw(p, LOYAL)) / 10.0 + z(PRIO[4]),
        5.0 + cw(p, LOYAL) / 8.0 + (a - 29.0).max(0.0) * 1.5 + z(PRIO[5]),
    ];
    let s: f32 = roh.iter().sum();
    roh.map(|x| x / s * 100.0)
}

/// The adviser type: big players mostly with a big agency, small ones often without anyone; under 20 the family
/// steps in more often.
pub fn typ(p: &Player) -> usize {
    let anteile = if p.ovr >= f("typ.oben") { liste("typ.anteil_oben") } else if p.ovr < f("typ.unten") { liste("typ.anteil_unten") } else { liste("typ.anteil_mitte") };
    let mut a = anteile.clone();
    if p.age < 20 { a[3] *= 2.5; }
    let s: f32 = a.iter().sum();
    let x = zufall(&[crate::id("beratertyp"), &p.id.to_string()]) * s;
    let mut acc = 0.0;
    for (i, w) in a.iter().enumerate() { acc += w; if x < acc { return i; } }
    4
}

/// (name, agency) of the adviser; a family adviser is the father, no adviser is none.
pub fn person(p: &Player) -> Option<(String, String)> {
    match typ(p) {
        0 => None,
        3 => { let nach = p.name.split_whitespace().last().unwrap_or(&p.name).to_string(); Some((crate::tf!("ui.berater.vater", nach), crate::inhalt::t("ui.berater.familie_agentur").into())) }
        _ => {
            let b = arr("berater", "berater");
            let i = (zufall(&[crate::id("berater"), &p.id.to_string()]) * b.len() as f32) as usize % b.len();
            Some((b[i]["name"].as_str()?.to_string(), b[i]["agentur"].as_str()?.to_string()))
        }
    }
}

impl Career {
    /// How well our offer meets his priorities, as a bonus on the chance in the player talk: playing time (would he
    /// start?), security (years against his wish), success (our name against his club's), home (our league's country is
    /// his), money is in the wage already, freedom in the clauses. Plus the adviser type and our relation with him.
    pub fn berater_bonus(&self, p: usize, years: u32) -> f32 {
        let pl = &self.world.players[p];
        let w = prioritaeten(pl);
        let g = crate::kimarkt::gruppe(&pl.pos);
        let besser = self.squad().iter().filter(|&&x| x != p && crate::kimarkt::gruppe(&self.world.players[x].pos) == g && self.world.players[x].ovr > pl.ovr).count();
        let spielzeit = if besser == 0 { 1.0 } else if besser == 1 { 0.3 } else { -0.6 };
        let wunsch = crate::verhandlung::wunschlaufzeit(pl.age) as f32;
        let sicherheit = ((years as f32 - wunsch) / 2.0).clamp(-1.0, 1.0);
        let alt = if pl.team >= 0 { self.world.clubs[pl.team as usize].rep } else { 40.0 };
        let erfolg = ((self.club().rep - alt) / 20.0).clamp(-1.0, 1.0);
        let heimat = if pl.nat != 0 && self.world.land_nat(self.league()) == pl.nat { 1.0 } else { -0.3 };
        let summe = (w[1] * spielzeit + w[2] * sicherheit + w[3] * erfolg + w[5] * heimat) / 100.0 * f("prioritaet.wirkung");
        let t = TYPEN[typ(pl)];
        summe + f(&format!("wirkung.{t}.bonus")) + self.berater_beziehung(p) * f("prioritaet.beziehung_je_punkt")
    }
    /// Our relation with the player's adviser (−3 … +3), kept by the adviser's name.
    pub fn berater_beziehung(&self, p: usize) -> f32 {
        let Some((n, _)) = person(&self.world.players[p]) else { return 0.0 };
        self.berater_bez.iter().find(|x| x.0 == n).map(|x| x.1).unwrap_or(0.0)
    }
    /// After a talk: a deal makes the adviser friendlier, a break cooler (Konzept 2.7 memory).
    pub fn berater_merken(&mut self, p: usize, d: f32) {
        let Some((n, _)) = person(&self.world.players[p]) else { return };
        match self.berater_bez.iter_mut().find(|x| x.0 == n) { Some(x) => x.1 = (x.1 + d).clamp(-3.0, 3.0), None => self.berater_bez.push((n, d.clamp(-3.0, 3.0))) }
    }
    /// The adviser's factor on the wage demand.
    pub fn berater_forderung(&self, p: usize) -> f64 { f(&format!("wirkung.{}.forderung", TYPEN[typ(&self.world.players[p])])) as f64 }
    /// The fee of the adviser on a signing (share of the transfer fee, at least a small sum for a free player).
    pub fn berater_honorar(&self, p: usize, abloese: f64) -> f64 {
        let s = f(&format!("wirkung.{}.honorar", TYPEN[typ(&self.world.players[p])])) as f64;
        if s <= 0.0 { 0.0 } else { crate::world::round_money((abloese * s).max(self.world.players[p].wage * 0.5 * s * 10.0)) }
    }
    /// A rough hint at his two strongest priorities, only with people skills (Menschenführung) or a good scout report.
    pub fn prioritaet_hinweis(&self, p: usize) -> Option<String> {
        if self.mgr_wert(0) < 50.0 && self.scout_bonus(p) <= 0.0 { return None; }
        let w = prioritaeten(&self.world.players[p]);
        let mut ix: Vec<usize> = (0..6).collect();
        ix.sort_by(|a, b| w[*b].partial_cmp(&w[*a]).unwrap());
        Some(crate::tf!("ui.berater.hinweis", self.world.players[p].name, crate::inhalt::t(&format!("ui.berater.prio.{}", PRIO[ix[0]])), crate::inhalt::t(&format!("ui.berater.prio.{}", PRIO[ix[1]]))))
    }
    /// What an argument does (Konzept 2.6): (bonus on the chance, patience it costs, the counterpart's answer). It must be
    /// believable (a real starting place, a real table position, home that fits), else it costs; it works as much as
    /// the priority behind it weighs; „Trainer ruft an“ and „Budget“ only once.
    pub fn argument_wirkung(&self, p: usize, a: u8, genutzt: &[u8]) -> (f32, f32, String) {
        let pl = &self.world.players[p];
        let w = prioritaeten(pl);
        let g = crate::kimarkt::gruppe(&pl.pos);
        let besser = self.squad().iter().filter(|&&x| x != p && crate::kimarkt::gruppe(&self.world.players[x].pos) == g && self.world.players[x].ovr > pl.ovr).count();
        let europa = self.world.cups.iter().any(|c| (1..=3).contains(&c.kind) && c.teams.contains(&self.own));
        let (gewicht, glaubhaft) = match a {
            1 => (w[1], besser <= 1),
            2 => (w[3], self.world.rank_of(self.own) <= 4 || europa),
            3 => (w[1].max(w[3]) * if pl.age <= 23 && pl.pot > pl.ovr + 3.0 { 1.0 } else { 0.3 }, self.training_faktor() >= 1.0),
            4 => (cw(pl, EGO) / 3.0, !genutzt.contains(&4)),
            5 => (w[5], pl.nat != 0 && self.world.land_nat(self.league()) == pl.nat),
            _ => (w[0], !genutzt.contains(&6)),
        };
        let k = |s: &str| f(&format!("argument.{s}"));
        let schluessel = if !glaubhaft { crate::id("unglaubwuerdig") } else if gewicht >= k("sehr_ab") { crate::id("sehr") } else if gewicht >= k("passt_ab") { crate::id("passt") } else { crate::id("egal") };
        let (b, kosten) = match schluessel { "sehr" => (k("sehr"), 0.0), "passt" => (k("passt"), 0.0), "egal" => (0.0, k("egal_geduld")), _ => (-k(crate::id("unpassend")), k("unpassend_geduld")) };
        (b, kosten, crate::inhalt::t(&format!("ui.argument.antwort.{schluessel}")).to_string())
    }
    /// „What matters most to him?“ (Konzept 2.6): the counterpart answers with his strongest priority, honestly or a little
    /// coloured by a sly agent; it costs patience and can be asked once per talk.
    pub fn neg_frage(&mut self, i: usize) -> Result<String, String> {
        let n = self.negs[i].clone();
        if n.gefragt { return Err(crate::inhalt::t("ui.argument.schon_gefragt").into()); }
        let p = n.pid;
        let w = prioritaeten(&self.world.players[p]);
        let mut ix: Vec<usize> = (0..6).collect();
        ix.sort_by(|a, b| w[*b].partial_cmp(&w[*a]).unwrap());
        // the sly agent names money first, whatever it is
        let erste = if TYPEN[typ(&self.world.players[p])] == "schlitzohr" { 0 } else { ix[0] };
        let kosten = f("argument.frage_geduld");
        let nn = &mut self.negs[i];
        nn.gefragt = true;
        if let Some(ph) = nn.ph.as_mut() { ph.geduld = (ph.geduld - kosten).max(0.0); }
        let text = crate::tf!("ui.argument.frage_antwort", crate::inhalt::t(&format!("ui.berater.prio.{}", PRIO[erste])));
        nn.text = text.clone();
        Ok(text)
    }
    /// Who speaks for the player in the chat: his agent with the agency, or the player himself.
    pub fn berater_name(&self, p: usize) -> String {
        let pl = &self.world.players[p];
        person(pl).map(|(n, a)| format!("{n} · {a}")).unwrap_or_else(|| pl.name.clone())
    }
    /// One line for the talk: who sits at the table for the player.
    pub fn berater_zeile(&self, p: usize) -> String {
        let pl = &self.world.players[p];
        let t = TYPEN[typ(pl)];
        match person(pl) {
            None => crate::tf!("ui.berater.ohne", pl.name),
            Some((n, a)) => {
                let bez = self.berater_beziehung(p);
                let b = if bez >= 1.0 { crate::inhalt::t("ui.berater.bez_gut") } else if bez <= -1.0 { crate::inhalt::t("ui.berater.bez_schlecht") } else { "" };
                crate::tf!("ui.berater.zeile", n, a, crate::inhalt::t(&format!("ui.berater.typ.{t}")), b)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prioritaeten_typen_und_gedaechtnis() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 7);
        let own = w.clubs.iter().position(|c| c.name.contains("Sunderland")).unwrap_or(0);
        let mut c = Career::new(w, own, "Test");
        let mut typen = [0usize; 5];
        for p in &c.world.players {
            let x = prioritaeten(p);
            assert!((x.iter().sum::<f32>() - 100.0).abs() < 0.01 && x.iter().all(|&v| v > 0.0));
            typen[typ(p)] += 1;
        }
        assert!(typen.iter().all(|&n| n > 0), "alle Typen kommen vor: {typen:?}");
        // the young want playing time more than the old, the old security more
        let jung = c.world.players.iter().filter(|p| p.age <= 20).map(|p| prioritaeten(p)[1]).sum::<f32>() / c.world.players.iter().filter(|p| p.age <= 20).count().max(1) as f32;
        let alt = c.world.players.iter().filter(|p| p.age >= 32).map(|p| prioritaeten(p)[1]).sum::<f32>() / c.world.players.iter().filter(|p| p.age >= 32).count().max(1) as f32;
        assert!(jung > alt, "Spielzeit jung {jung} alt {alt}");
        // memory: a deal raises the relation and the bonus
        let p = c.world.players.iter().position(|x| x.team >= 0 && x.team != own as i32 && person(x).is_some() && typ(x) != 3).unwrap();
        let b0 = c.berater_bonus(p, 3);
        c.berater_merken(p, 1.0);
        assert!(c.berater_bonus(p, 3) > b0);
        assert!(c.berater_zeile(p).contains(&person(&c.world.players[p]).unwrap().0));
        // arguments: the budget line only once, a starting place promised to the fifth keeper is not believed
        let (_, k1, _) = c.argument_wirkung(p, 6, &[]);
        let (b2, k2, _) = c.argument_wirkung(p, 6, &[6]);
        assert!(b2 < 0.0 && k2 > k1, "zweites Mal Budget kostet");
        let sq = c.squad();
        let schwach = *sq.iter().min_by(|a, b| c.world.players[**a].ovr.partial_cmp(&c.world.players[**b].ovr).unwrap()).unwrap();
        assert!(c.argument_wirkung(schwach, 1, &[]).0 < 0.0, "Stammplatz unglaubwürdig");
        // the question once per talk, it costs patience
        let q = c.squad()[3];
        c.world.players[q].contract_end = c.world.season;
        let i = c.open_neg(q).unwrap();
        let g0 = c.negs[i].ph.as_ref().unwrap().geduld;
        assert!(c.neg_frage(i).is_ok());
        assert!(c.negs[i].ph.as_ref().unwrap().geduld < g0);
        assert!(c.neg_frage(i).is_err());
    }
}
