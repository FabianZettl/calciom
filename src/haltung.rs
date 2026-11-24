//! A player's stance on an offer for him (pesm/core/gespraeche/haltung.py): character, role, playing time, contract,
//! morale, readiness to move and the buyer's standing. He is not sold against his will; talking to him moves it.
use crate::game::*;
use crate::inhalt::t;
use crate::mensch::{cw, EHRGEIZ, LOYAL};
use crate::regeln::f;

pub const STUFEN: [&str; 5] = [crate::id("unbedingt"), crate::id("chance"), crate::id("offen"), crate::id("zoegernd"), crate::id("bleiben")];
/// "Suggest the move" / "ask him to stay": step and limit of the shift (service.py SCHUB_SCHRITT, SCHUB_MAX).
const SCHUB_SCHRITT: f32 = 10.0;
const SCHUB_MAX: f32 = 30.0;

pub struct Haltung { pub wert: i32, pub stufe: usize, pub zustimmung: bool, pub gruende: Vec<(&'static str, i8)> }

fn h(k: &str) -> f32 { f("gespraeche", &format!("haltung.{k}")) } // roh

impl Career {
    /// The role in the original's words (haltung.py ROLLEN, ranked): from the squad role of the mensch model.
    fn rolle_haltung(&self, p: usize) -> &'static str {
        let pl = &self.world.players[p];
        let rang = self.world.clubs[self.own].squad.iter().filter(|&&q| self.world.players[q].ovr > pl.ovr).count();
        if self.kapitaen() == Some(p) { return crate::id("kapitaen"); }
        if rang < 2 { return crate::id("star"); }
        match pl.ps.rolle { 0..=2 => crate::id("fuehrungsspieler"), 3 => crate::id("stammspieler"), 4 => crate::id("rotation"), 5 => crate::id("perspektive"), _ => crate::id("ersatz") }
    }
    /// The stance of own player `p` on a move to `kaeufer`.
    pub fn haltung(&self, p: usize, kaeufer: usize) -> Haltung {
        let pl = &self.world.players[p];
        let mut teile: Vec<(&'static str, f32)> = vec![];
        teile.push((crate::id("ehrgeiz"), (cw(pl, EHRGEIZ) - 50.0) * h(crate::id("ehrgeiz"))));
        let (rk, re) = (self.world.clubs[kaeufer].rep, self.club().rep);
        teile.push((crate::id("ruf"), ((rk - re) * h(crate::id("ruf"))).clamp(-25.0, 25.0)));
        teile.push((crate::id("loyalitaet"), -(cw(pl, LOYAL) - 50.0) * h(crate::id("loyalitaet"))));
        teile.push((crate::id("wechselbereitschaft"), (pl.ps.wb - 10.0) * h(crate::id("wechselbereitschaft"))));
        let rolle = self.rolle_haltung(p);
        teile.push((crate::id("rolle"), f("gespraeche", &format!("haltung.rolle.{rolle}")))); // roh
        if !pl.ps.einsatz.is_empty() {
            let n = pl.ps.einsatz.len().min(10);
            let anteil = pl.ps.einsatz.iter().rev().take(n).sum::<f32>() / n as f32;
            teile.push((crate::id("spielzeit"), (0.5 - anteil) * h(crate::id("spielzeit"))));
        }
        teile.push((crate::id("moral"), (50.0 - pl.morale) * h(crate::id("moral"))));
        let rest = self.monate_rest(p);
        let kurz = crate::regeln::wert("gespraeche", "haltung.vertrag_kurz").as_array().unwrap();
        let v = kurz.iter().find(|x| rest <= x[0].as_f64().unwrap_or(0.0) as f32).map(|x| x[1].as_f64().unwrap_or(0.0) as f32).unwrap_or(if rest >= 36.0 { -h("vertrag_lang") } else { 0.0 });
        teile.push((crate::id("vertrag"), v));
        let schub = pl.ps.schub;
        let wert = (teile.iter().map(|x| x.1).sum::<f32>() + schub).round() as i32;
        let grenzen = crate::regeln::wert("gespraeche", "haltung.stufen").as_array().unwrap();
        let mut stufe = grenzen.iter().position(|g| wert as f64 >= g[1].as_f64().unwrap_or(0.0)).unwrap_or(4);
        // a "big chance" only with a bigger club
        if stufe == 1 && rk <= re { stufe = 2; }
        let mut gr: Vec<(&'static str, f32)> = teile.into_iter().filter(|x| x.1.abs() >= 5.0).collect();
        gr.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        Haltung { wert, stufe, zustimmung: wert as f32 >= h("zustimmung_ab"), gruende: gr.into_iter().take(3).map(|(n, v)| (n, if v > 0.0 { 1 } else { -1 })).collect() }
    }
    /// The player on the offer in his words, the reasons without numbers, and the note when he does not agree.
    pub fn haltung_text(&self, p: usize, kaeufer: usize) -> String {
        let hl = self.haltung(p, kaeufer);
        let verein = self.world.clubs[kaeufer].name.clone();
        let v = (self.world.players[p].id as usize + kaeufer) % 3;
        let mut s = format!("{}: {}", self.world.players[p].name, crate::tf!(&format!("gs.angebot.haltung.{}.{v}", STUFEN[hl.stufe]), verein.clone())); // roh
        let gr: Vec<String> = hl.gruende.iter().map(|(n, x)| crate::tf!(&format!("gs.angebot.grund.{n}.{}", if *x > 0 { "plus" } else { "minus" }), verein.clone())).filter(|x| !x.is_empty()).collect(); // roh
        if !gr.is_empty() { s = format!("{s}\n{}", gr.join(" · ")); }
        if !hl.zustimmung { s = format!("{s}\n{}", t("gs.angebot.ohne_zustimmung")); }
        s
    }
    /// Suggest the move (+) or ask him to stay (−): moves his stance, against his will it costs relation.
    pub fn haltung_schieben(&mut self, p: usize, kaeufer: usize, richtung: i8) -> String {
        let vorher = self.haltung(p, kaeufer).stufe;
        let sgn = if richtung > 0 { 1.0 } else { -1.0 };
        let alt = self.world.players[p].ps.schub;
        let neu = (alt + sgn * SCHUB_SCHRITT).clamp(-SCHUB_MAX, SCHUB_MAX);
        let name = self.world.players[p].name.clone();
        if neu == alt { return t("gs.ui.fehler.schon_ausgereizt").into(); }
        self.world.players[p].ps.schub = neu;
        let gegen = (sgn > 0.0 && vorher >= 3) || (sgn < 0.0 && vorher <= 1);
        self.beziehung(p, if gegen { -1.5 } else { 0.5 }, t("gs.ui.nahelegen"));
        let mut s = crate::tf!(if sgn > 0.0 { "gs.ui.ok.haltung_nahegelegt" } else { "gs.ui.ok.haltung_bleiben" }, name.clone());
        if gegen { s = format!("{s} {}", crate::tf!("gs.ui.haltung.gegen_willen", name)); }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn haltung_und_gespraech() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let p = c.squad()[0];
        let k = (0..c.world.clubs.len()).find(|&x| x != c.own).unwrap();
        let a = c.haltung(p, k);
        assert!(a.stufe < 5 && a.gruende.len() <= 3);
        c.haltung_schieben(p, k, 1);
        assert_eq!(c.haltung(p, k).wert, a.wert + 10, "suggesting the move shifts by one step");
        for _ in 0..5 { c.haltung_schieben(p, k, 1); }
        assert_eq!(c.world.players[p].ps.schub, 30.0, "at most three steps");
        assert!(!c.haltung_text(p, k).is_empty());
    }
}
