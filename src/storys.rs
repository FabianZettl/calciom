//! Paket 3q: scenarios and stories (Mechanik-Übergabe §2; core/story.py). The rules, values and texts are the
//! original's data, copied unchanged into assets/story/. After every own match (one "Takt") the engine checks the
//! active rules, rolls their chance with fixed randomness and sends at most 6 messages (2 per rule) by priority.
use crate::game::*;
use crate::mensch::{cw, pfeil, typen, zufall};
use serde_json::Value;
use std::sync::OnceLock;

struct Daten { regeln: Vec<Value>, texte: serde_json::Map<String, Value>, zustand: Value }
fn daten() -> &'static Daten {
    static D: OnceLock<Daten> = OnceLock::new();
    D.get_or_init(|| {
        let mut regeln = vec![];
        for t in [include_str!("../assets/story/regeln/kader.json"), include_str!("../assets/story/regeln/verein.json"), include_str!("../assets/story/regeln/kabine_storys.json"), include_str!("../assets/story/regeln/folgen.json")] {
            let v: Value = serde_json::from_str(t).unwrap();
            regeln.extend(v["regeln"].as_array().unwrap().iter().cloned());
        }
        let mut texte = serde_json::Map::new();
        for t in [include_str!("../assets/story/texte/charakter.json"), include_str!("../assets/story/texte/ergebnisse.json"), include_str!("../assets/story/texte/folgen.json"), include_str!("../assets/story/texte/integration.json"),
                  include_str!("../assets/story/texte/kabine.json"), include_str!("../assets/story/texte/leistung.json"), include_str!("../assets/story/texte/pressekonferenz.json"), include_str!("../assets/story/texte/storys.json"),
                  include_str!("../assets/story/texte/spielzeit.json"), include_str!("../assets/story/texte/tabelle.json"), include_str!("../assets/story/texte/vertrag.json"), include_str!("../assets/story/texte/wirkung.json"), include_str!("../assets/story/texte/schritt5.json")] {
            if let Value::Object(m) = serde_json::from_str::<Value>(t).unwrap() { for (k, v) in m { if !k.starts_with('_') { texte.insert(k, v); } } }
        }
        Daten { regeln, texte, zustand: serde_json::from_str(include_str!("../assets/story/zustand.json")).unwrap() }
    })
}
fn regel(id: &str) -> Option<&'static Value> { daten().regeln.iter().find(|r| r["id"].as_str() == Some(id)) }
fn aktiv(r: &Value) -> bool {
    let id = r["id"].as_str().unwrap_or("");
    r["aktiv"].as_bool().unwrap_or(false) || daten().zustand["aktive_regeln"].as_array().map_or(false, |a| a.iter().any(|x| x.as_str() == Some(id)))
}
fn prio(r: &Value) -> usize { match r["prioritaet"].as_str() { Some("rot") => 0, Some("gelb") => 1, _ => 2 } }

impl Career {
    /// A value the rules ask for (story/werte.json), for a player or the club.
    fn story_wert(&self, name: &str, p: Option<usize>) -> Value {
        let l = self.league();
        let table = self.world.table(l);
        let pos = table.iter().position(|r| r.club == self.own).unwrap_or(0);
        let res: Vec<char> = self.results.iter().map(|r| r.chars().next().unwrap_or('U')).collect();
        let num = |x: f64| Value::from(x);
        if let Some(p) = p {
            let pl = &self.world.players[p];
            let wi = |w: &str| crate::mensch::WERTE.iter().position(|x| *x == w).map(|i| cw(pl, i) as f64);
            return match name {
                "alter" => num(pl.age as f64),
                "ausfalltage" => num(pl.inj as f64),
                "einsaetze" => num(pl.apps as f64),
                "formpfeil" => num(pfeil(pl.form) as f64),
                "gesamtwert" => num(pl.ovr as f64),
                "gesamtwert_rang_im_kader" => num((self.squad().iter().filter(|&&q| self.world.players[q].ovr > pl.ovr).count() + 1) as f64),
                "neuzugang" => num((pl.joined > 0 && self.abs() < pl.joined + 60) as u8 as f64),
                "rolle" => Value::from(crate::mensch::ROLLEN[pl.ps.rolle.min(6) as usize]),
                "spielzeit_anteil" => num(if pl.ps.einsatz.is_empty() { 0.0 } else { pl.ps.einsatz.iter().sum::<f32>() as f64 / pl.ps.einsatz.len() as f64 * 100.0 }),
                "tage_bis_vertragsende" => num((self.monate_rest(p) * 30.4) as f64),
                "tore_saison" => num(pl.goals as f64),
                "typen" => Value::from(typen(pl).iter().map(|t| Value::from(*t)).collect::<Vec<_>>()),
                w => wi(w).map(num).unwrap_or_else(|| self.story_wert(w, None)),
            };
        }
        match name {
            "abstand_zum_ziel" => num((pos + 1) as f64 - self.club().goal_pos as f64),
            "abstiegsrang" => num((pos + 1 + crate::regeln::f("klima", "liga.abstiegsplaetze") as usize > table.len()) as u8 as f64),
            "aufstiegsrang" => num(0.0),
            "niederlagen_in_folge" => num(res.iter().rev().take_while(|&&x| x == 'N').count() as f64),
            "siege_in_folge" => num(res.iter().rev().take_while(|&&x| x == 'S').count() as f64),
            "spiele_ohne_sieg" => num(res.iter().rev().take_while(|&&x| x != 'S').count() as f64),
            "saisonziel_fehlt" => num((!self.mensch.ziel_angenommen) as u8 as f64),
            "spiele_offen" => num(self.world.leagues[l].fixtures.iter().filter(|x| x.hg < 0 && (x.h == self.own || x.a == self.own)).count() as f64),
            "tabellenplatz" => num((pos + 1) as f64),
            "tage_bis_naechstes_spiel" => num(self.next_match().map(|f| f.day.saturating_sub(self.world.day)).unwrap_or(99) as f64),
            "teamgeist" => num(self.mensch.teamgeist as f64),
            "vereinsspiele_saison" => num(table.get(pos).map(|r| r.p).unwrap_or(0) as f64),
            "vorstandsvertrauen" => num(self.mensch.vorstand as f64),
            "ziel_vorschlag" => num(self.club().goal_pos as f64),
            _ => Value::Null,
        }
    }
    fn story_erfuellt(&self, r: &Value, p: Option<usize>) -> bool {
        r["bedingungen"].as_array().map_or(true, |bs| bs.iter().all(|b| {
            let v = self.story_wert(b["wert"].as_str().unwrap_or(""), p);
            if let Some(e) = b.get("enthaelt") { return match &v { Value::Array(a) => a.contains(e), Value::String(s) => e.as_str().map_or(false, |x| s == x || x.split(',').any(|y| y == s)), _ => false }; }
            if let Some(g) = b.get("gleich") { return match (&v, g) { (Value::String(a), Value::String(b2)) => a == b2, _ => v.as_f64() == g.as_f64() }; }
            let x = v.as_f64().unwrap_or(f64::NAN);
            b["min"].as_f64().map_or(true, |m| x >= m) && b["max"].as_f64().map_or(true, |m| x <= m)
        }))
    }
    fn story_chance(&self, r: &Value, p: Option<usize>) -> f32 {
        let w = &r["wahrscheinlichkeit"];
        let mut c = w["grund"].as_f64().unwrap_or(1.0) as f32;
        if let Some(p) = p {
            let ty = typen(&self.world.players[p]);
            if let Some(m) = w["typ"].as_object() { for (k, v) in m { if ty.contains(&k.as_str()) { c += v.as_f64().unwrap_or(0.0) as f32; } } }
        }
        if let Some(m) = w["wert"].as_object() { for (k, v) in m { c += v.as_f64().unwrap_or(0.0) as f32 * self.story_wert(k, p).as_f64().unwrap_or(0.0) as f32; } }
        c.clamp(0.0, 1.0)
    }
    /// The text of a rule, in the version of the player's first message type (persoenlichkeit.json nachrichten_typen).
    fn story_text(&mut self, key: &str, p: Option<usize>, zweck: &str) -> String {
        let Some(t) = daten().texte.get(key) else { return String::new() };
        let mut v = t["standard"].as_array().cloned().unwrap_or_default();
        if let Some(p) = p {
            let nt = crate::mensch::arr("persoenlichkeit", "nachrichten_typen");
            if let Some(ty) = typen(&self.world.players[p]).into_iter().find(|t| nt.iter().any(|x| x.as_str() == Some(t))) {
                if let Some(a) = t["typ"][ty].as_array() { if !a.is_empty() { v = a.clone(); } }
            }
        }
        if v.is_empty() { return String::new(); }
        // the no-repeat bag per text (Fabian 23.11.): a variant comes back only after all others were used
        let i = self.variante(&format!("story:{key}"), v.len(), zweck);
        let s = v[i].as_str().unwrap_or("").to_string();
        let name = p.map(|p| self.world.players[p].name.clone()).unwrap_or_default();
        let kurz = name.split(' ').last().unwrap_or("").to_string();
        let gegner = self.last_result.as_ref().map(|r| self.world.clubs[if r.h == self.own { r.a } else { r.h }].name.clone()).unwrap_or_default();
        let ergebnis = self.last_result.as_ref().map(|r| format!("{}:{}", r.hg, r.ag)).unwrap_or_default();
        let mut s = s.replace("{spieler_kurz}", &kurz).replace("{spieler}", &name).replace("{verein}", &self.club().name).replace("{gegner}", &gegner)
            .replace("{platz}", &self.world.rank_of(self.own).to_string()).replace("{ergebnis}", &ergebnis).replace("{vorstand}", crate::inhalt::t("ui.storys.der_praesident"))
            .replace("{reporter}", crate::inhalt::t("ui.storys.ein_reporter")).replace("{medium}", PAPERS[self.abs() as usize % PAPERS.len()]).replace("{fanvertreter}", crate::inhalt::t("ui.storys.der_fanvertreter"))
            .replace("{berater}", crate::inhalt::t("ui.storys.sein_berater")).replace("{zahl}", &self.club().goal_pos.to_string()).replace("{tage}", crate::id("einigen"));
        while let (Some(a), Some(b)) = (s.find('{'), s.find('}')) { if b > a { s.replace_range(a..=b, ""); } else { break; } }
        s
    }
    /// Send the message of a rule (also for follow-up rules, without conditions).
    fn story_senden(&mut self, r: &Value, p: Option<usize>) {
        let id = r["id"].as_str().unwrap_or("").to_string();
        let opts: Vec<(String, String)> = r["antworten"].as_array().map(|a| a.iter().map(|x| {
            let aid = x["id"].as_str().unwrap_or("").to_string();
            let t = self.story_text(&format!("{id}.antwort.{aid}"), p, crate::id("antwort"));
            (aid.clone(), if t.is_empty() { aid } else { t })
        }).collect()).unwrap_or_default();
        let betreff = self.story_text(&format!("{id}.betreff"), p, crate::id("betreff"));
        let text = self.story_text(&format!("{id}.text"), p, crate::id("text"));
        let von = match r["absender"].as_str() { Some("spieler") => p.map(|p| self.world.players[p].name.clone()).unwrap_or_default(), Some("vorstand") => "Präsidium".into(), Some("presse") => crate::inhalt::t("feed.art.journalist").into(), Some("fans") => crate::inhalt::t("ui.storys.fanvertreter").into(), _ => self.co_name.clone() }; // roh
        let abs = self.abs();
        // a rule without any text of its own is not sent (it came as "News from the squad" with an empty body)
        if text.is_empty() && betreff.is_empty() { return; }
        self.mensch.story_erzaehlt.push((id.clone(), abs, p.map(|p| self.world.players[p].id).unwrap_or(0)));
        let key = format!("{id}|{}", p.map(|p| self.world.players[p].id).unwrap_or(0));
        self.mensch.story_sperre.retain(|x| x.0 != key);
        self.mensch.story_sperre.push((key, abs + r["sperrfrist_tage"].as_u64().unwrap_or(14) as u32));
        let action = if opts.is_empty() { Action::None } else { Action::Wahl { art: format!("story:{id}"), bezug: p.into_iter().collect(), optionen: opts } };
        let cat = if p.is_some() { 1 } else { 5 };
        // a rule without its own subject gets the first sentence of its text
        let titel = if !betreff.is_empty() { betreff } else if !text.is_empty() { let t: String = text.split(['.', '!', '?']).next().unwrap_or("").chars().take(70).collect(); t } else { crate::inhalt::t("ui.storys.neuigkeit_aus_der_mannschaft").to_string() };
        self.msg(cat, &von, &titel, &text, action);
    }
    /// One "Takt" after an own match: the active rules by priority, at most 6 messages and 2 per rule;
    /// player concerns at most 2 a day, 6 a week and 7 days apart per player (zustand.json anliegen).
    pub fn story_takt(&mut self) {
        let abs = self.abs();
        self.mensch.story_sperre.retain(|x| abs < x.1);
        let z = &daten().zustand;
        let (max_takt, max_regel) = (z["nachrichten_je_takt"].as_u64().unwrap_or(6) as usize, z["je_regel_je_takt"].as_u64().unwrap_or(2) as usize);
        let mut kand: Vec<(usize, &Value, Option<usize>)> = vec![];
        for r in daten().regeln.iter().filter(|r| aktiv(r)) {
            let id = r["id"].as_str().unwrap_or("");
            let bezuege: Vec<Option<usize>> = if r["bezug"].as_str() == Some(crate::id("spieler")) { self.squad().into_iter().filter(|&p| self.world.players[p].youth == 0).map(Some).collect() } else { vec![None] };
            for p in bezuege {
                let key = format!("{id}|{}", p.map(|p| self.world.players[p].id).unwrap_or(0));
                if self.mensch.story_sperre.iter().any(|x| x.0 == key) || !self.story_erfuellt(r, p) { continue; }
                // Fabian 23.11. ("the lucky socks four times a season"): a story is told once per career by the same
                // player and by anyone at most once in the rule's long quiet time (einmal_tage, default 120)
                if self.mensch.story_erzaehlt.iter().any(|x| x.0 == id && (p.map_or(false, |p| x.2 == self.world.players[p].id) || abs < x.1 + r["einmal_tage"].as_u64().unwrap_or(120) as u32)) { continue; }
                let u = zufall(&["2026", id, &key, &abs.to_string(), crate::id("ausloesen")]);
                if u < self.story_chance(r, p) { kand.push((prio(r), r, p)); }
            }
        }
        kand.sort_by_key(|k| k.0);
        let mut je_regel: Vec<(&str, usize)> = vec![];
        let mut n = 0;
        for (_, r, p) in kand {
            if n >= max_takt { break; }
            let id = r["id"].as_str().unwrap_or("");
            let c = je_regel.iter().find(|x| x.0 == id).map(|x| x.1).unwrap_or(0);
            if c >= max_regel { continue; }
            if let Some(p) = p {
                let pid = self.world.players[p].id;
                self.mensch.story_anliegen.retain(|x| abs < x.1 + 7);
                if self.mensch.story_anliegen.iter().filter(|x| x.1 == abs).count() >= 2 || self.mensch.story_anliegen.len() >= 6 || self.mensch.story_anliegen.iter().any(|x| x.0 == pid) { continue; }
                self.mensch.story_anliegen.push((pid, abs));
            }
            self.story_senden(r, p);
            je_regel.retain(|x| x.0 != id);
            je_regel.push((id, c + 1));
            n += 1;
        }
    }
    /// Daily: follow-up rules that are due; unanswered story messages after 14 days cost the player 3 mood.
    pub fn story_tag(&mut self) {
        if !self.feature("gespraeche") { return; } // roh
        let abs = self.abs();
        let faellig: Vec<(String, Option<usize>)> = self.mensch.story_folgen.iter().filter(|x| abs >= x.2).map(|x| (x.0.clone(), x.1)).collect();
        self.mensch.story_folgen.retain(|x| abs < x.2);
        for (id, p) in faellig { if let Some(r) = regel(&id) { if p.map_or(true, |p| self.world.players[p].team == self.own as i32) { self.story_senden(r, p); } } }
        let frist = daten().zustand["frist_tage"].as_u64().unwrap_or(14) as u32;
        let ohne = daten().zustand["ohne_antwort"]["stimmung"].as_f64().unwrap_or(-3.0) as f32;
        let mut alt = vec![];
        for m in self.inbox.iter_mut() {
            if m.done { continue; }
            if let Action::Wahl { art, bezug, .. } = &m.action { if art.starts_with(crate::id("story:")) && abs >= abs_day(m.season, m.day) + frist { m.done = true; m.body += crate::inhalt::t("ui.storys.unbeantwortet"); alt.push(bezug.first().copied()); } }
        }
        for p in alt.into_iter().flatten() { self.moral_wirkung(p, ohne, 0, crate::inhalt::t("ui.storys.seine_nachricht_blieb_unbeantwortet")); }
    }
    /// The answer to a story message: effects (at most ±10 each), feedback text, promise, follow-up rule.
    pub fn story_antwort(&mut self, id: &str, bezug: &[usize], key: &str) -> String {
        let Some(r) = regel(id) else { return String::new() };
        let Some(a) = r["antworten"].as_array().and_then(|a| a.iter().find(|x| x["id"].as_str() == Some(key))) else { return String::new() };
        let p = bezug.first().copied();
        let cap = daten().zustand["zustand"]["aenderung_je_antwort"].as_f64().unwrap_or(10.0) as f32;
        let grund = crate::tf!("ui.storys.antwort_auf", self.story_text(&format!("{id}.betreff"), p, crate::id("betreff")));
        for w in a["wirkungen"].as_array().cloned().unwrap_or_default() {
            let d = (w["delta"].as_f64().unwrap_or(0.0) as f32).clamp(-cap, cap);
            match w["ziel"].as_str().unwrap_or("") {
                "stimmung" => if let Some(p) = p { self.moral_wirkung(p, d, 0, &grund); },
                "beziehung_trainer" => if let Some(p) = p { self.beziehung(p, d, &grund); },
                "medienbild" => self.klima(0.0, d, 0.0, &grund),
                "vorstandsvertrauen" => self.klima(0.0, 0.0, d, &grund),
                "fanstimmung" => self.klima(d, 0.0, 0.0, &grund),
                "versprechen" => if let Some(p) = p { self.versprechen_geben(p, if w["art"].as_str() == Some(crate::id("einsatzzeit")) { crate::id("spielzeit") } else { crate::id("rolle") }); },
                "saisonziel" => { self.mensch.ziel_angenommen = true; self.grund("Präsidium", crate::inhalt::t("zentrale.k.ziel"), 0.0, &grund); } // roh
                "charakter" => if let (Some(p), Some(wn)) = (p, w["wert"].as_str()) { if let Some(i) = crate::mensch::WERTE.iter().position(|x| *x == wn) { self.verschieben(p, i, d, &grund); } },
                _ => {}
            }
        }
        if let Some(fo) = a.get("folge") {
            let u = zufall(&["2026", crate::id("folge"), id, key, &self.abs().to_string()]);
            if u < fo["wahrscheinlichkeit"].as_f64().unwrap_or(1.0) as f32 {
                let tage = fo["nach_tagen"].as_u64().or(fo["tage"].as_u64()).unwrap_or(7) as u32;
                let abs = self.abs();
                self.mensch.story_folgen.push((fo["regel"].as_str().unwrap_or("").to_string(), p, abs + tage));
            }
        }
        self.story_text(&format!("{id}.rueckmeldung.{key}"), p, crate::id("rueckmeldung"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regeln_und_texte_aus_der_vorlage() {
        assert_eq!(daten().regeln.len(), 78);
        assert!(daten().texte.len() > 400);
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        c.psyche_an();
        let t = c.story_text("wenig_spielzeit_stammkraft.text", Some(c.squad()[0]), "x");
        assert!(!t.is_empty() && !t.contains('{'));
        // a whole season of takts stays within the limits and leaves story messages
        let mut gesamt = 0;
        for _ in 0..crate::date::SEASON_DAYS {
            let vor = c.inbox.iter().filter(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("story:"))).count();
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            c.ultimatum = None;
            c.end_day().unwrap();
            let nach = c.inbox.iter().filter(|m| matches!(&m.action, Action::Wahl { art, .. } if art.starts_with("story:"))).count();
            assert!(nach.saturating_sub(vor) <= 6);
            gesamt += nach.saturating_sub(vor);
        }
        let aktive: Vec<&str> = daten().regeln.iter().filter(|r| aktiv(r)).map(|r| r["id"].as_str().unwrap()).collect();
        let sq = c.squad();
        let erf: Vec<String> = daten().regeln.iter().filter(|r| aktiv(r)).filter(|r| if r["bezug"].as_str() == Some("spieler") { sq.iter().any(|&p| c.story_erfuellt(r, Some(p))) } else { c.story_erfuellt(r, None) }).map(|r| r["id"].as_str().unwrap().to_string()).collect();
        assert!(gesamt > 0, "the season brings stories: aktiv {} {:?}, erfüllt jetzt {:?}, sperren {}", aktive.len(), aktive, erf, c.mensch.story_sperre.len());
    }
}
