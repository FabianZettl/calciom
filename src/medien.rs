//! Paket 3k and 3l: fans (Mechanik-Übergabe §14) and media, press conferences, journalists, public promises,
//! rumours and denials (§15, medien.toml). Numbers from assets/regeln, texts from assets/texte/presse.json.
use crate::game::*;
use crate::mensch::{arr, zufall};
use crate::regeln::f;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

fn presse() -> &'static serde_json::Value { crate::inhalt::eigen("presse") }
pub const TOENE: [&str; 7] = [crate::id("ehrlich"), crate::id("kaempferisch"), crate::id("ausweichend"), crate::id("lob"), crate::id("kritik"), crate::id("diplomatisch"), crate::id("kontern")];
pub const STILE: [&str; 4] = [crate::id("sachlich"), crate::id("boulevard"), crate::id("wohlwollend"), crate::id("kritisch")];
const JOURNALISTEN: [&str; 10] = ["Katrin Albers", "Mehmet Kaya", "Sophie Brandl", "Jan Wiegand", "Lena Hofer", "Tobias Rehm", "Clara Neumann", "Marco Salis", "Ines Vogt", "Rafael Duarte"]; // roh

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Medien {
    /// Journalists: (name, style index, attitude to the coach).
    pub journalisten: Vec<(String, u8, f32)>,
    /// The paper or outlet each journalist writes for (medienpool.json, from the own country), same order.
    pub blaetter: Vec<String>,
    /// Topic → abs day it was last asked.
    pub themen: Vec<(String, u32)>,
    /// The variants used last (key → index), so a question, an answer or a rumour does not come twice in a row.
    pub zuletzt: Vec<(String, usize)>,
    /// What the last answer about a player did to him (shown once on the press card).
    #[serde(skip)]
    pub letzte_wirkung: String,
    /// Public promises: (kind nicht_verkaufen | saisonziel, player, since, deadline, state 0 open 1 kept 2 broken).
    pub zusagen: Vec<(String, Option<usize>, u32, u32, u8)>,
    /// Rumours: (player, abs day, real?, denied?).
    pub geruechte: Vec<(usize, u32, bool, bool)>,
    /// The coach's statements at press conferences (abs day, tone, quote, number of own results then), for the weekly paper.
    pub aussagen: Vec<(u32, String, String, usize)>,
    pub fan_transfer: f32,
    pub fan_wirtschaft: f32,
    pub fan_aktion: f32,
    pub aktion_sperre: Vec<(String, u32)>,
    pub protest_stufe: u8,
    /// TV shows and podcasts (Konzept 6b): an open invitation (name, 0 TV / 1 podcast), an accepted appearance (name,
    /// kind, abs day), the one running now (name, effect factor) and the last invitation per kind.
    #[serde(default)]
    pub sendung_angebot: Option<(String, u8)>,
    #[serde(default)]
    pub sendung: Option<(String, u8, u32)>,
    #[serde(default)]
    pub sendung_aktiv: Option<(String, f32)>,
    #[serde(default)]
    pub sendung_zuletzt: [u32; 2],
}

impl Career {
    // ------------------------------------------------------------ fans (§14)
    /// Effective fan mood = climate fans + transfer effect + economy mood + fan actions, 0…100.
    pub fn fans_eff(&self) -> f32 { (self.fans + self.me.fan_transfer + self.me.fan_wirtschaft + self.me.fan_aktion).clamp(0.0, 100.0) }
    /// A transfer moves the fans: ±40 · fee / yearly income (buy +, sale −), at most ±15 (vereinswelt.toml [fans]).
    pub fn fans_transfer(&mut self, fee: f64, kauf: bool) {
        let u = self.umsatz_prognose().max(1.0);
        let g = f("vereinswelt", "fans.transfer_wirkung_grenze");
        let d = (f("vereinswelt", "fans.transfer_wirkung_je_anteil") * (fee / u) as f32 * if kauf { 1.0 } else { -1.0 }).clamp(-g, g);
        self.me.fan_transfer = (self.me.fan_transfer + d).clamp(-g, g);
        self.grund("Fans", crate::inhalt::t("ui.medien.transferwirkung"), d, if kauf { crate::inhalt::t("ui.medien.ein_neuzugang_begeistert_die_fans") } else { crate::inhalt::t("ui.medien.ein_verkauf_aergert_die_fans") }); // roh
    }
    /// Monthly: transfer effect decays to 80 %, the economy mood follows its target by 30 % (prices), actions fade.
    pub fn fans_monat(&mut self) {
        self.me.fan_transfer *= f("vereinswelt", "fans.abklingen");
        self.me.fan_aktion *= f("vereinswelt", "fans.abklingen");
        // prices, atmosphere, catering and advertising set the target of the economy mood (wirtschaft.py monatsabschluss)
        let ziel = self.spiel_neutral().stimmung_ziel as f32;
        let alt = self.me.fan_wirtschaft;
        self.me.fan_wirtschaft += f("vereinswelt", "wirtschaft.stimmung_glaettung") * (ziel - alt);
        if (self.me.fan_wirtschaft - alt).abs() > 0.05 { let d = self.me.fan_wirtschaft - alt; self.grund("Fans", crate::inhalt::t("ui.medien.wirtschaftsstimmung"), d, crate::inhalt::t("vw.preis.titel")); } // roh
        let abs = self.abs();
        self.me.aktion_sperre.retain(|x| abs < x.1);
        let st = self.protest_stufe();
        if st > self.me.protest_stufe {
            let cn = self.club().name.clone();
            let w = self.protest_wort(st);
            self.news(&crate::tf!("ui.medien.bei", w, cn), crate::inhalt::t("ui.medien.die_anhaenger_machen_ihrem_aerger"));
        }
        self.me.protest_stufe = st;
    }
    /// Fan actions (Fanfest, Familientag, Legendenspiel, Wohltätigkeit, Auswärtsfahrt): cost scaled by club power, fans and media, lock.
    pub fn fanaktionen(&self) -> Vec<(String, String, f64, f32, f32, u32)> {
        presse()["aktionen"].as_array().unwrap().iter().map(|a| (a[0].as_str().unwrap().to_string(), a[1].as_str().unwrap().to_string(), a[2].as_f64().unwrap() * (0.3 + 0.7 * self.kraft()), a[3].as_f64().unwrap() as f32, a[4].as_f64().unwrap() as f32, a[5].as_u64().unwrap() as u32)).collect()
    }
    pub fn fanaktion(&mut self, i: usize) -> Result<(), String> {
        let (id, name, kosten, fans, medien, sperre) = self.fanaktionen()[i].clone();
        let abs = self.abs();
        if let Some(x) = self.me.aktion_sperre.iter().find(|x| x.0 == id) { return Err(crate::tf!("ui.medien.geht_erst_in_tagen_wieder", name, x.1 - abs)); }
        self.book(crate::inhalt::t("ui.medien.fanaktionen"), -kosten);
        self.me.fan_aktion = (self.me.fan_aktion + fans).min(15.0);
        self.grund("Fans", crate::inhalt::t("ui.medien.fanaktion"), fans, &name); // roh
        if medien != 0.0 { self.klima(0.0, medien, 0.0, &name); }
        self.me.aktion_sperre.push((id, abs + sperre));
        Ok(())
    }
    /// Protest value (medien.toml [proteste]): effective fans − 4 per loss in a row from the second − 8 if the board barely trusts the coach.
    pub fn protest_wert(&self) -> f32 {
        let n = self.results.iter().rev().take_while(|r| r.starts_with('N')).count() as f32;
        self.fans_eff() - f("medien", "proteste.abzug_je_niederlage") * (n - 1.0).max(0.0) - if self.mensch.vorstand < 30.0 { f("medien", "proteste.abzug_vertrauen") } else { 0.0 }
    }
    /// 0 calm, 1 murmurs, 2 whistles, 3 banners, 4 stadium protest.
    pub fn protest_stufe(&self) -> u8 {
        let w = self.protest_wert();
        let st = arr("medien", "proteste.stufen");
        (st.iter().filter(|s| w < s[1].as_f64().unwrap() as f32).count()) as u8
    }
    pub fn protest_wort(&self, st: u8) -> &'static str {
        match st { 4 => crate::inhalt::t("ui.medien.stadionprotest"), 3 => crate::inhalt::t("ui.medien.transparente_gegen_die_fuehrung"), 2 => crate::inhalt::t("ui.medien.pfiffe_im_stadion"), 1 => crate::inhalt::t("ui.medien.murren_auf_den_raengen"), _ => crate::inhalt::t("ui.medien.ruhe_auf_den_raengen") }
    }

    // ------------------------------------------------------------ press conference (§15)
    /// Every journalist gets an outlet from the press pool of the own country (medienpool.json), chosen by his
    /// style: factual → serious/tactics, tabloid → tabloid/rumours, friendly → fan culture/insider, critical →
    /// investigative/serious. Fills the gaps of old saves too.
    pub fn blaetter_an(&mut self) {
        if self.me.blaetter.len() >= self.me.journalisten.len() { return; }
        let land = land_kuerzel(&self.world.leagues[self.league()].land);
        let pool = medienpool();
        let seed = self.club().id.to_string();
        for i in self.me.blaetter.len()..self.me.journalisten.len() {
            let toene: &[&str] = match self.me.journalisten[i].1 { 0 => &["serioes", "taktik"], 1 => &["boulevard", "geruechte"], 2 => &["fankultur", "insider"], _ => &[crate::id("investigativ"), crate::id("serioes")] };
            let mut kand: Vec<&(String, String, String)> = pool.iter().filter(|m| m.1 == land && toene.contains(&m.2.as_str()) && !self.me.blaetter.contains(&m.0)).collect();
            if kand.is_empty() { kand = pool.iter().filter(|m| m.1 == land && !self.me.blaetter.contains(&m.0)).collect(); }
            if kand.is_empty() { kand = pool.iter().filter(|m| m.1 == "INT" && !self.me.blaetter.contains(&m.0)).collect(); }
            let name = if kand.is_empty() { PAPERS[i % PAPERS.len()].to_string() } else { kand[(zufall(&["2026", crate::id("blatt"), &seed, &i.to_string()]) * kand.len() as f32) as usize % kand.len()].0.clone() };
            self.me.blaetter.push(name);
        }
    }
    /// The two beats of journalist `j` (FAECHER): a real reporter by the keywords of his media pool entry, else by
    /// his tone; an invented one by his style, the second beat fixed by his name.
    pub fn faecher(&self, j: usize) -> Vec<&'static str> {
        let Some((name, stil, _)) = self.me.journalisten.get(j) else { return vec![] };
        let mut v: Vec<&'static str> = vec![];
        if let Some(tags) = pool_schlagworte().get(name) {
            for fach in FAECHER { let w = crate::regeln::wert("medien", &format!("journalist.fach_schlagworte.{fach}")); if tags.iter().any(|t| w.as_array().map_or(false, |a| a.iter().any(|x| x.as_str() == Some(t.as_str())))) { v.push(fach); } }
        }
        let erst = match pool_eintraege().iter().find(|e| &e.0 == name).map(|e| e.3.as_str()) {
            Some("geruechte") => "transfers", Some("taktik") => "taktik", Some("insider") | Some("investigativ") => "interna", Some("fankultur") => crate::id("spieler"),
            _ => match stil { 0 => "taktik", 1 => "transfers", 2 => "spieler", _ => crate::id("meinung") },
        };
        if !v.contains(&erst) { v.insert(0, erst); }
        let h = name.bytes().fold(7usize, |a, b| a.wrapping_mul(31).wrapping_add(b as usize));
        let rest: Vec<&'static str> = FAECHER.iter().copied().filter(|f| !v.contains(f)).collect();
        if v.len() < 2 && !rest.is_empty() { v.push(rest[h % rest.len()]); }
        v.truncate(2);
        v
    }
    /// The journalists whose beats cover `thema` (medien.toml journalist.fach).
    fn fuer_thema(&self, thema: &str) -> Vec<usize> {
        (0..self.me.journalisten.len()).filter(|&j| self.faecher(j).iter().any(|f| crate::regeln::wert("medien", &format!("journalist.fach.{f}")).as_array().map_or(false, |a| a.iter().any(|x| x.as_str() == Some(thema))))).collect()
    }
    /// The outlet of journalist `j` ("" when unknown).
    pub fn blatt(&self, j: usize) -> String { self.me.blaetter.get(j).cloned().unwrap_or_default() }

    fn journalisten_an(&mut self) {
        if !self.me.journalisten.is_empty() { self.blaetter_an(); return; }
        let seed = self.club().id.to_string();
        let n = f("medien", "journalist.anzahl") as usize;
        // Konzept 1.4 (Fabian 23.11.): the real reporters of the own country first (medienpool.json), the rest invented;
        // their tone sets their style (rumours → boulevard, tactics → serious)
        let land = land_kuerzel(&self.world.leagues[self.league()].land);
        if f("medien", "journalist.echte_personen") > 0.0 {
            for (name, _, typ, ton) in pool_eintraege().iter().filter(|e| e.1 == land && e.2 == "reporter").take(n / 2) {
                let _ = typ;
                let stil = match ton.as_str() { "geruechte" | "boulevard" => 1, "fankultur" | "insider" => 2, "investigativ" => 3, _ => 0 };
                self.me.journalisten.push((name.clone(), stil, 0.0));
            }
        }
        for i in self.me.journalisten.len()..n {
            let k = (zufall(&["2026", crate::id("journalist"), &seed, &i.to_string()]) * JOURNALISTEN.len() as f32) as usize % JOURNALISTEN.len();
            let name = JOURNALISTEN[(k + i) % JOURNALISTEN.len()].to_string();
            self.me.journalisten.push((name, (i % 4) as u8, 0.0));
        }
        self.blaetter_an();
    }
    /// Topics that fit now (medien.toml [toene] keys), the fresh ones first (wiederholung_tage 21).
    fn themen_jetzt(&self, post: bool) -> Vec<(String, Option<usize>)> {
        let mut t: Vec<(String, Option<usize>)> = vec![];
        let res: Vec<char> = self.results.iter().map(|r| r.chars().next().unwrap_or('U')).collect();
        if post {
            let last = res.last().copied().unwrap_or('U');
            t.push((match last { 'S' => "sieg", 'N' => "niederlage", _ => crate::id("remis") }.into(), None));
            if let Some(r) = &self.last_result {
                if let Some(b) = r.ratings.iter().filter(|x| self.world.players[x.0].team == self.own as i32).max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) { if b.1 >= 7.5 { t.push((crate::id("star").into(), Some(b.0))); } }
            }
        } else {
            t.push((crate::id("standard").into(), None));
        }
        if res.iter().rev().take_while(|&&x| x == 'S').count() >= 3 { t.push(("serie_gut".into(), None)); }
        if res.iter().rev().take_while(|&&x| x != 'S').count() >= 3 { t.push(("serie_schlecht".into(), None)); }
        if self.mensch.vorstand < 45.0 { t.push((crate::id("vorstand").into(), None)); }
        // the rumour about one of ours: asked about once in a while, not at every conference (Fabian 23.11.)
        let abs0 = self.abs();
        if let Some(&(p, ..)) = self.me.geruechte.iter().rev().find(|g| self.world.players[g.0].team == self.own as i32) {
            let zuletzt = self.me.themen.iter().find(|y| y.0 == crate::id("geruecht")).map_or(0, |y| y.1);
            if zuletzt == 0 || abs0 >= zuletzt + f("medien", "konferenz.geruecht_pause_tage") as u32 { t.push((crate::id("geruecht").into(), Some(p))); t.push(("zusage_spieler".into(), Some(p))); }
        }
        if let Some(p) = self.squad().into_iter().filter(|&p| self.world.players[p].inj >= 14 && self.world.players[p].ps.rolle <= 3).max_by(|&a, &b| self.world.players[a].ovr.partial_cmp(&self.world.players[b].ovr).unwrap()) { t.push((crate::id("ausfall").into(), Some(p))); }
        if let Some(p) = self.squad().into_iter().filter(|&p| self.world.players[p].joined > 0 && self.abs() < self.world.players[p].joined + 30).max_by(|&a, &b| self.world.players[a].ovr.partial_cmp(&self.world.players[b].ovr).unwrap()) { t.push((crate::id("neuzugang").into(), Some(p))); }
        if !self.me.zusagen.iter().any(|z| z.0 == crate::id("saisonziel") && z.4 == 0) && self.world.day < 120 { t.push(("zusage_ziel".into(), None)); }
        let abs = self.abs();
        let wh = f("medien", "konferenz.wiederholung_tage") as u32;
        t.sort_by_key(|x| self.me.themen.iter().any(|y| y.0 == x.0 && abs < y.1 + wh) as u8);
        t
    }
    /// A variant out of `n` like a shuffle bag (Fabian 23.11.: nothing may repeat): never one already used for this
    /// key until all were used; then the bag starts again, but not with the one used last.
    pub fn variante(&mut self, schluessel: &str, n: usize, salz: &str) -> usize {
        if n <= 1 { return 0; }
        let mut benutzt: Vec<usize> = self.me.zuletzt.iter().filter(|x| x.0 == schluessel).map(|x| x.1).collect();
        let letzte = benutzt.last().copied();
        if benutzt.iter().filter(|&&i| i < n).count() >= n {
            self.me.zuletzt.retain(|x| x.0 != schluessel);
            benutzt = letzte.into_iter().collect();
        }
        let frei: Vec<usize> = (0..n).filter(|i| !benutzt.contains(i)).collect();
        let i = frei[(zufall(&["variante", schluessel, salz, &self.abs().to_string()]) * frei.len() as f32) as usize % frei.len()]; // roh
        if benutzt.len() == 1 && self.me.zuletzt.iter().all(|x| x.0 != schluessel) { if let Some(l) = letzte { self.me.zuletzt.push((schluessel.to_string(), l)); } }
        self.me.zuletzt.push((schluessel.to_string(), i));
        i
    }
    /// The key of the next variant of `basis` (basis, basis.1, basis.2 …) out of its own shuffle bag.
    pub fn vkey(&mut self, basis: &str, salz: &str) -> String { let i = self.variante(&format!("v:{basis}"), crate::inhalt::varianten(basis), salz); crate::inhalt::variante_key(basis, i) } // roh
    fn frage_text(&mut self, thema: &str, p: Option<usize>, salz: &str) -> String {
        let fr = presse()["fragen"][thema].as_array().cloned().unwrap_or_default();
        let i = self.variante(&format!("frage:{thema}"), fr.len(), salz);
        let s = fr.get(i).and_then(|x| x.as_str()).unwrap_or(crate::inhalt::t("ui.medien.ihre_meinung")).to_string();
        // a lead-in with the real context (score, streak, table, next opponent) in front of most questions, from its
        // own bag, so question and lead-in combine anew each time (Fabian 23.11.: every question completely new)
        let ein = presse()["einleitung"][thema].as_array().cloned().unwrap_or_default();
        let mit = zufall(&["einleitung", thema, salz, &self.abs().to_string()]) < 0.9; // roh
        let s = if mit && !ein.is_empty() { let j = self.variante(&format!("einleitung:{thema}"), ein.len(), salz); format!("{} {s}", ein[j].as_str().unwrap_or("")) } else { s };
        self.platzhalter(&s, thema, p)
    }
    /// {gegner}, {spieler}, {platz}, {ziel} in a question or an answer of the press conference.
    fn platzhalter(&self, s: &str, thema: &str, p: Option<usize>) -> String {
        let gegner = self.next_match().map(|f| self.world.clubs[if f.h == self.own { f.a } else { f.h }].name.clone())
            .or_else(|| self.last_result.as_ref().map(|r| self.world.clubs[if r.h == self.own { r.a } else { r.h }].name.clone())).unwrap_or_default();
        let gegner = if self.last_result.is_some() && thema != crate::id("standard") { self.last_result.as_ref().map(|r| self.world.clubs[if r.h == self.own { r.a } else { r.h }].name.clone()).unwrap_or(gegner) } else { gegner };
        let naechster = self.next_match().map(|f| self.world.clubs[if f.h == self.own { f.a } else { f.h }].name.clone()).unwrap_or_else(|| gegner.clone());
        let ergebnis = self.last_result.as_ref().map(|r| if r.h == self.own { format!("{}:{}", r.hg, r.ag) } else { format!("{}:{}", r.ag, r.hg) }).unwrap_or_default();
        let res: Vec<char> = self.results.iter().map(|r| r.chars().next().unwrap_or('U')).collect();
        let serie = res.iter().rev().take_while(|&&x| x == 'S').count().max(1);
        let sieglos = res.iter().rev().take_while(|&&x| x != 'S').count().max(1);
        let punkte = self.world.table(self.league()).iter().find(|x| x.club == self.own).map(|x| x.pts).unwrap_or(0);
        s.replace("{gegner}", &gegner).replace("{spieler}", &p.map(|p| self.world.players[p].name.clone()).unwrap_or_default())
            .replace("{platz}", &self.world.rank_of(self.own).to_string()).replace("{ziel}", &self.club().goal_pos.to_string())
            .replace("{naechster}", &naechster).replace("{ergebnis}", &ergebnis).replace("{serie}", &serie.to_string())
            .replace("{sieglos}", &sieglos.to_string()).replace("{punkte}", &punkte.to_string())
    }
    /// A conference of up to 4 questions (konferenz.fragen); answers are the tones of the topic.
    pub fn press_question(&mut self, post: bool) -> PressQ {
        self.journalisten_an();
        let themen = self.themen_jetzt(post);
        let n = (f("medien", "konferenz.fragen") as usize).min(themen.len()).max(1);
        let abs = self.abs();
        let mut fragen: Vec<PressQ> = vec![];
        for (k, (thema, p)) in themen.into_iter().take(n).enumerate() {
            let toene = crate::regeln::wert("medien", &format!("toene.{}", if thema.starts_with(crate::id("zusage")) { &thema } else { &thema })).as_array().cloned().unwrap_or_default();
            // the answer fits the question (Fabian 22.11.): text per topic and tone, a variant by day, the generic one only as a fallback
            let mut answers: Vec<(String, u8)> = vec![];
            for t in toene.iter().filter_map(|t| t.as_str()) {
                let v = presse()["antworten_thema"][&thema][t].as_array().cloned().unwrap_or_default();
                let i = self.variante(&format!("antwort:{thema}:{t}"), v.len(), &k.to_string());
                let roh = v.get(i).and_then(|x| x.as_str()).or(presse()["antworten"][t].as_str()).unwrap_or("");
                answers.push((self.platzhalter(roh, &thema, p), TOENE.iter().position(|x| *x == t).unwrap_or(0) as u8));
            }
            if thema.starts_with(crate::id("zusage")) {
                let z = presse()["zusage_antwort"][&thema].as_str().unwrap_or("").replace("{spieler}", &p.map(|p| self.world.players[p].name.clone()).unwrap_or_default());
                let ton = if thema == "zusage_ziel" { 1 } else { 0 };
                answers.insert(0, (z, ton + 10)); // + 10 marks the answer that gives the public promise
            }
            let q = self.frage_text(&thema, p, &k.to_string());
            // the question comes from a journalist whose beat it is (Fabian 24.11.), else from anyone
            let wer = { let v = self.fuer_thema(&thema); if v.is_empty() { (0..self.me.journalisten.len()).collect() } else { v } };
            let j = wer[(zufall(&["2026", crate::id("frage"), &abs.to_string(), &k.to_string()]) * wer.len() as f32) as usize % wer.len().max(1)] as u8;
            self.me.themen.retain(|x| x.0 != thema);
            self.me.themen.push((thema.clone(), abs));
            fragen.push(PressQ { q, answers, post, thema, journalist: j, bezug: p, rest: vec![], summe: [0.0; 4] });
        }
        let mut erste = fragen.remove(0);
        erste.rest = fragen;
        erste
    }
    /// Answer the current question (None = not attended); after the last one the effects are booked, at most ±6 per target.
    pub fn answer_press(&mut self, choice: Option<usize>) {
        let Some(mut pq) = self.press.take() else { return };
        let Some(i) = choice else {
            let k = |s: &str| f("medien", &format!("konferenz.nicht_erschienen.{s}"));
            self.klima(k(crate::id("fanstimmung")), k(crate::id("medienbild")), k(crate::id("vorstandsvertrauen")), crate::inhalt::t("ui.medien.nicht_zur_pressekonferenz_erschienen"));
            let coach = self.coach.clone();
            self.news(&crate::tf!("ui.medien.schwaenzt_die_pressekonferenz", coach), crate::inhalt::t("ui.medien.die_journalisten_warteten_vergeblich"));
            return;
        };
        let code = pq.answers[i].1;
        let ton = TOENE[(code % 10) as usize];
        let w = |ziel: &str| -> f32 {
            let basis = f("medien", &format!("ton.{ton}.{ziel}"));
            basis * crate::regeln::f_oder("medien", &format!("thema.{}.{ton}.{ziel}", pq.thema), 1.0)
        };
        let mut eff = [w(crate::id("medienbild")) * self.mgr_faktor(3), w(crate::id("vorstandsvertrauen")), w(crate::id("fanstimmung")), w(crate::id("moral"))];
        let stil = crate::regeln::wert_oder("manager", &format!("presse_ton.{ton}")).and_then(|v| v.as_str()).unwrap_or(crate::id("ruhig"));
        self.stil_punkte(crate::id("presse"), stil);
        if let Some(j) = self.me.journalisten.get_mut(pq.journalist as usize) {
            let neigung = crate::regeln::f_oder("medien", &format!("journalist.neigung.{}.{ton}", STILE[j.1 as usize % 4]), 0.0);
            let v = f("medien", "journalist.verstaerkung");
            if neigung > 0.0 { eff[0] *= 1.0 + v; } else if neigung < 0.0 { eff[0] *= 1.0 - v; }
            j.2 = (j.2 + neigung * f("medien", "journalist.haltung_je_antwort")).clamp(-100.0, 100.0);
        }
        for k in 0..4 { pq.summe[k] += eff[k]; }
        if code >= 10 { self.zusage_geben(&pq.thema, pq.bezug); }
        // an answer about one of our players hits him straight away (Fabian 24.11.): morale and form by the tone,
        // stronger with his character, with the reason in his diary and on the card
        if let Some(p) = pq.bezug.filter(|&p| self.world.players[p].team == self.own as i32) {
            let sw = |x: &str| crate::regeln::f_oder("medien", &format!("spieler_ton.{ton}.{x}"), 0.0);
            let pl = &self.world.players[p];
            let (ego, temp) = (crate::mensch::cw(pl, crate::mensch::EGO) / 100.0, crate::mensch::cw(pl, crate::mensch::TEMPERAMENT) / 100.0);
            let mut moral = sw(crate::id("moral"));
            if moral < 0.0 { moral *= 1.0 + (f("medien", "spieler_ton.ego_kritik") - 1.0) * ego; }
            let mut form = sw(crate::id("form"));
            if form > 0.0 { form *= 1.0 + (f("medien", "spieler_ton.temperament_form") - 1.0) * temp; }
            let (name, grund) = (pl.name.clone(), crate::tf!("ui.medien.ueber_ihn", crate::inhalt::t(&format!("md.ui.ton.{ton}"))));
            if moral != 0.0 { self.moral_wirkung(p, moral, 0, &grund); }
            if form != 0.0 { let pl = &mut self.world.players[p]; pl.form = (pl.form + form).clamp(-3.0, 3.0); }
            self.me.letzte_wirkung = crate::tf!("ui.medien.wirkung_spieler", name, format!("{moral:+.0}"), format!("{form:+.1}"));
        } else { self.me.letzte_wirkung.clear(); }
        let quote = pq.answers[i].0.clone();
        let rest = std::mem::take(&mut pq.rest);
        if let Some(mut next) = rest.first().cloned() {
            next.rest = rest[1..].to_vec();
            next.summe = pq.summe;
            self.press = Some(next);
            return;
        }
        // a TV show or podcast reaches more people: its factor on the effects, a wider cap (Konzept 6b.5)
        let sendung = self.me.sendung_aktiv.take();
        let (faktor, cap) = match &sendung { Some((_, f_)) => (*f_, f("medien", "sendung.deckel")), None => (1.0, f("medien", "konferenz.deckel")) };
        let s = pq.summe.map(|x| (x * faktor).clamp(-cap, cap));
        let g = crate::inhalt::t("ui.medien.auftritt_auf_der_pressekonferenz");
        if let Some((name, _)) = &sendung {
            self.klima(s[2], s[0], s[1], &crate::tf!("ui.sendung.auftritt", name.clone()));
            let coach = self.coach.clone();
            self.news(&crate::tf!("ui.sendung.nachlese_titel", coach.clone(), name.clone()), &crate::tf!("ui.sendung.nachlese_text", coach, name.clone(), quote));
            return;
        }
        self.klima(s[2], s[0], s[1], g);
        if s[3] != 0.0 { for p in self.squad() { self.moral_wirkung(p, s[3], 0, g); } }
        let coach = self.coach.clone();
        let abs = self.abs();
        let n_res = self.results.len();
        self.me.aussagen.push((abs, ton.to_string(), quote.clone(), n_res));
        if self.me.aussagen.len() > 20 { self.me.aussagen.remove(0); }
        // the press writes about the quote with the original's articles (zeitung.json zt.pk.<tone>), where it has them
        let tt = crate::pool::varianten(&format!("zt.pk.{ton}.titel"));
        let tx = crate::pool::varianten(&format!("zt.pk.{ton}.text"));
        if !tt.is_empty() && !tx.is_empty() {
            // from the no-repeat bag (Fabian 23.11.: every newspaper report one of a kind)
            let (it, ix) = (self.variante(&format!("zt.pk.{ton}.titel"), tt.len(), &quote), self.variante(&format!("zt.pk.{ton}.text"), tx.len(), &quote));
            let kurz: String = if quote.chars().count() > 70 { quote.chars().take(68).collect::<String>() + "…" } else { quote.clone() };
            self.news(&tt[it].replace("%1", &kurz), &tx[ix].replace("%1", &quote));
            return;
        }
        let head = match ton { "kaempferisch" => crate::inhalt::t("sp.ton.vor.bestimmt"), "kritik" => crate::inhalt::t("ui.medien.trainer_kritisiert_eigene_spieler"), "kontern" => crate::inhalt::t("ui.medien.trainer_teilt_aus"), "ausweichend" => crate::inhalt::t("ui.medien.schweigen_auf_dem_podium"), "lob" => crate::inhalt::t("ui.medien.rueckendeckung_fuer_die_mannschaft"), _ => crate::inhalt::t("ui.medien.klare_worte_vom_trainer") };
        self.news(head, &crate::tf!("ui.medien.auf_der_pressekonferenz", coach, quote));
    }
    /// The weekly paper (Teil 2, posteingang.toml [zeitung]): on its weekday the headlines of the own club from the last
    /// seven days, led by what the coach said at the podium and what has happened since (results after the quote).
    pub fn zeitung_woche(&mut self) {
        let z = |k: &str| crate::regeln::f("posteingang", &format!("zeitung.{k}")) as u32;
        if (crate::date::weekday(self.world.day) as u32 + 6) % 7 != z(crate::id("wochentag")) { return; }
        let abs = self.abs();
        let own = self.club().name.clone();
        let mut zeilen: Vec<String> = vec![];
        for (tag, ton, quote, n) in self.me.aussagen.iter().rev().filter(|a| abs < a.0 + z("aussagen_tage")) {
            let seit: Vec<&String> = self.results.iter().skip(*n).collect();
            let (s, u, nl) = (seit.iter().filter(|r| r.starts_with('S')).count(), seit.iter().filter(|r| r.starts_with('U')).count(), seit.iter().filter(|r| r.starts_with('N')).count());
            let bilanz = crate::tf!("ui.zeitung.bilanz", s, u, nl);
            let kurz: String = if quote.chars().count() > 60 { quote.chars().take(58).collect::<String>() + "…" } else { quote.clone() };
            let tage = abs.saturating_sub(*tag);
            let art = if seit.is_empty() { crate::id("offen") } else if nl > s { crate::id("verpufft") } else { crate::id("gehalten") };
            let _ = ton;
            zeilen.push(format!("{art}\t{kurz}\t{tage}\t{bilanz}")); // roh
            if zeilen.len() >= 2 { break; }
        }
        let mut zeilen: Vec<String> = zeilen.into_iter().map(|z| { let v: Vec<&str> = z.splitn(4, '\t').collect(); let k = self.vkey(&format!("ui.zeitung.bezug.{}", v[0]), v[1]); crate::tf!(&k, v[1], v[2], v[3]) }).collect();
        let max = z(crate::id("schlagzeilen")) as usize;
        for nw in self.feed.iter().filter(|n| crate::game::abs_day(n.season, n.day) + 7 > abs && n.head.contains(&own)) {
            if zeilen.len() >= max { break; }
            zeilen.push(nw.head.clone());
        }
        // one weekly paper instead of two (Posteingang, five questions): the rest of the league follows the own club's lines
        let welt: Vec<String> = self.feed.iter().filter(|n| crate::game::abs_day(n.season, n.day) + 7 > abs && !n.head.contains(&own)).take(max).map(|n| n.head.clone()).collect();
        if welt.len() >= 3 { zeilen.push(crate::tf!("ui.zeitung.aus_der_liga", welt.join(" · "))); }
        if zeilen.is_empty() { return; }
        self.journalisten_an();
        let blatt_s = if self.me.blaetter.is_empty() { PAPERS[(abs as usize / 7) % PAPERS.len()].to_string() } else { self.me.blaetter[(abs as usize / 7) % self.me.blaetter.len()].clone() };
        let blatt = blatt_s.as_str();
        self.msg(4, blatt, crate::inhalt::t("ui.zeitung.wochenschau"), &zeilen.iter().map(|z| format!("• {z}")).collect::<Vec<_>>().join(" "), Action::None);
    }
    /// A promise to the public (gespraeche.toml [zusage_verein]): not selling a player (120 days) or the season goal (300 days).
    fn zusage_geben(&mut self, thema: &str, p: Option<usize>) {
        let art = if thema == "zusage_ziel" { crate::id("saisonziel") } else { "nicht_verkaufen" };
        let abs = self.abs();
        let frist = f("gespraeche", &format!("zusage_verein.frist_tage.{art}")) as u32;
        self.me.zusagen.push((art.into(), p, abs, abs + frist, 0));
        self.grund(crate::inhalt::t("ui.medien.oeffentlichkeit"), crate::inhalt::t("chat.zusage"), 0.0, &crate::tf!("ui.medien.oeffentlich_versprochen", if art == crate::id("saisonziel") { crate::inhalt::t("ui.medien.das_saisonziel").to_string() } else { crate::tf!("ui.seiten.bleibt", p.map(|p| self.world.players[p].name.clone()).unwrap_or_default()) }));
    }
    fn zusage_entscheiden(&mut self, i: usize, gehalten: bool) {
        let z = self.me.zusagen[i].clone();
        if z.4 != 0 { return; }
        self.me.zusagen[i].4 = if gehalten { 1 } else { 2 };
        let teil = if gehalten { crate::id("gehalten") } else { crate::id("gebrochen") };
        let k = |s: &str| crate::regeln::f_oder("gespraeche", &format!("zusage_verein.{teil}.{}.{s}", z.0), 0.0);
        let grund = crate::tf!("ui.medien.oeffentliches_versprechen", teil, if z.0 == crate::id("saisonziel") { crate::inhalt::t("zentrale.k.ziel").to_string() } else { z.1.map(|p| crate::tf!("ui.seiten.bleibt", self.world.players[p].name)).unwrap_or_default() });
        self.klima(k(crate::id("fanstimmung")), k(crate::id("medienbild")), k(crate::id("vorstandsvertrauen")), &grund);
        if k(crate::id("moral")) != 0.0 { for p in self.squad() { self.moral_wirkung(p, k(crate::id("moral")), 0, &grund); } }
    }
    /// Daily: public promises and rumours, a made-up rumour about a key player (4 % a day, 21 days lock).
    pub fn medien_tag(&mut self) {
        let abs = self.abs();
        for i in 0..self.me.zusagen.len() {
            let z = self.me.zusagen[i].clone();
            if z.4 != 0 || z.0 != "nicht_verkaufen" { continue; }
            let weg = z.1.map_or(false, |p| self.world.players[p].team != self.own as i32 && self.world.players[p].team >= -1);
            if weg { self.zusage_entscheiden(i, false); } else if abs >= z.3 { self.zusage_entscheiden(i, true); }
        }
        // denials: after 45 days or when the player left
        let frist = f("medien", "geruecht.dementi_frist_tage") as u32;
        for i in 0..self.me.geruechte.len() {
            let (p, d, echt, dementi) = self.me.geruechte[i];
            if !dementi || d == u32::MAX { continue; }
            let weg = self.world.players[p].team != self.own as i32;
            let teil = if weg { crate::id("widerlegt") } else if abs >= d + frist { if echt { crate::id("bestaetigt") } else { "dementiert_falsch" } } else { continue };
            let k = |s: &str| crate::regeln::f_oder("medien", &format!("geruecht.{teil}.{s}"), 0.0);
            let n = self.world.players[p].name.clone();
            self.klima(k(crate::id("fanstimmung")), k(crate::id("medienbild")), k(crate::id("vorstandsvertrauen")), &crate::tf!("ui.medien.dementi_zu", n, if weg { crate::inhalt::t("ui.medien.der_wechsel_kam_doch") } else { crate::inhalt::t("ui.medien.er_ist_geblieben") }));
            self.me.geruechte[i].1 = u32::MAX;
        }
        self.me.geruechte.retain(|g| g.1 != u32::MAX && abs < g.1 + 90);
        let key = self.squad().into_iter().filter(|&p| self.world.players[p].ps.rolle <= 3).max_by(|&a, &b| self.world.players[a].ovr.partial_cmp(&self.world.players[b].ovr).unwrap());
        let gesperrt = self.me.geruechte.iter().any(|g| !g.2 && abs < g.1 + f("medien", "geruecht.falsch_sperre_tage") as u32);
        // a rumour needs a core (Medien-Konzept 5.1/5.2): only a club that needs his position, can pay and is at least as big
        if let Some(p) = key { if !gesperrt && zufall(&["2026", crate::id("falschgeruecht"), &abs.to_string()]) < f("medien", "geruecht.falsch_wahrscheinlichkeit") {
            if let Some(c) = self.plausibler_interessent(p) { self.geruecht_mit(p, false, Some(c)); }
        } }
    }
    /// A rumour about an own player (real after an offer, or made up): the person may deny it.
    pub fn geruecht(&mut self, p: usize, echt: bool) { self.geruecht_mit(p, echt, None) }
    /// A club that could really want him: needs his position group, has the money, at least the own club's reputation.
    pub fn plausibler_interessent(&self, p: usize) -> Option<usize> {
        let pl = &self.world.players[p];
        let gr = crate::kimarkt::gruppe(&pl.pos);
        let own_rep = self.club().rep;
        (0..self.world.clubs.len()).filter(|&c| c != self.own && self.world.clubs[c].rep >= own_rep - 5.0 && self.world.clubs[c].balance * 0.5 + self.world.revenue(c) * 0.1 >= pl.value && crate::game::is_derby(&self.world.clubs[c].name, &self.club().name).is_none())
            .filter(|&c| self.bedarf(c).iter().any(|b| b.0 == gr)).max_by(|a, b| self.world.clubs[*a].rep.partial_cmp(&self.world.clubs[*b].rep).unwrap())
    }
    pub fn geruecht_mit(&mut self, p: usize, echt: bool, club: Option<usize>) {
        let abs = self.abs();
        // Fabian 23.11.: a denied rumour does not come back the next day — none while the denial runs, and no new one
        // for the same player within the quiet time after the last
        let ruhe = |g: &(usize, u32, bool, bool)| g.1 + if g.3 { f("medien", "geruecht.dementi_frist_tage") as u32 } else { f("medien", "geruecht.ruhe_tage") as u32 };
        if self.me.geruechte.iter().any(|g| g.0 == p && abs < ruhe(g)) { return; }
        self.me.geruechte.push((p, abs, echt, false));
        let n = self.world.players[p].name.clone();
        // the rumour comes from a tabloid journalist of the own country's press (medienpool), else any outlet
        self.journalisten_an();
        let j = (0..self.me.journalisten.len()).find(|&j| self.faecher(j).first() == Some(&crate::id("transfers"))).or_else(|| self.me.journalisten.iter().position(|x| x.1 == 1)).unwrap_or(abs as usize % self.me.journalisten.len().max(1));
        let paper_s = { let b = self.blatt(j); if b.is_empty() { PAPERS[(abs as usize) % PAPERS.len()].to_string() } else { b } };
        let paper = paper_s.as_str();
        let v = self.variante("geruecht_text", 5, &n);
        let text = match club { Some(c) => crate::tf!(&format!("ui.medien.geruecht_verein.{v}"), paper, self.world.clubs[c].name.clone(), n.clone()), None => crate::tf!("ui.medien.laut_gibt_es_interesse_anderer", paper) };
        let vk = self.variante("geruecht_kopf", 5, &n);
        self.news(&crate::tf!(&format!("ui.medien.geruecht_kopf.{vk}"), n.clone(), club.map(|c| self.world.clubs[c].name.clone()).unwrap_or_default()), &text);
        // the press office's note and its answers in a variant (Fabian 23.11.: not always the same words)
        let (vd, vs, vt, vb) = (self.variante("geruecht_dementi", 4, &n), self.variante("geruecht_schweigen", 4, &n), self.variante("geruecht_titel", 4, &n), self.variante("geruecht_body", 4, &n));
        let opts = vec![(crate::id("dementieren").to_string(), crate::inhalt::t(&format!("ui.medien.geruecht_dementi.{vd}")).to_string()), (crate::id("schweigen").to_string(), crate::inhalt::t(&format!("ui.medien.geruecht_schweigen.{vs}")).to_string())];
        let klub = club.map(|c| self.world.clubs[c].name.clone()).unwrap_or_else(|| crate::inhalt::t("ui.medien.ein_verein").to_string());
        self.msg(4, crate::inhalt::t("ui.medien.pressestelle"), &crate::tf!(&format!("ui.medien.geruecht_titel.{vt}"), n.clone()), &crate::tf!(&format!("ui.medien.geruecht_body.{vb}"), n.clone(), klub, paper.to_string()), Action::Wahl { art: crate::id("geruecht").into(), bezug: vec![p], optionen: opts });
    }
    // ------------------------------------------------------------ TV shows and podcasts (Konzept 6b)
    /// Once a week: an invitation when there is a reason (success, crisis, else the club's standing), at most one TV
    /// show and one podcast a month; shows and podcasts of the own country from medienpool.json.
    pub fn sendungen_woche(&mut self) {
        if self.me.sendung_angebot.is_some() || self.me.sendung.is_some() { return; }
        let abs = self.abs();
        let res: Vec<char> = self.results.iter().map(|r| r.chars().next().unwrap_or('U')).collect();
        let gespielt = self.world.table(self.league()).iter().find(|r| r.club == self.own).map_or(0, |r| r.p);
        let erfolg = (gespielt >= 5 && self.world.rank_of(self.own) <= 3) || res.iter().rev().take_while(|&&x| x == 'S').count() >= 3;
        let krise = res.len() >= 4 && res.iter().rev().take(4).all(|&x| x != 'S');
        let g = |k: &str| f("medien", &format!("sendung.{k}"));
        let chance = if erfolg { g("chance_erfolg") } else if krise { g("chance_krise") } else { g("chance_grund") } * (0.4 + 0.6 * self.club().rep / 100.0);
        if zufall(&["sendung", &self.club().id.to_string(), &(abs / 7).to_string()]) >= chance { return; } // roh
        let land = land_kuerzel(&self.world.leagues[self.league()].land);
        let frei = |art: usize| abs >= self.me.sendung_zuletzt[art] + 28 || self.me.sendung_zuletzt[art] == 0;
        let art: u8 = if frei(0) && (krise || erfolg || !frei(1)) { 0 } else if frei(1) { 1 } else { return };
        let typen: &[&str] = if art == 0 { &["tv_radio"] } else { &[crate::id("podcast"), crate::id("youtube")] };
        let kand: Vec<&(String, String, String, String)> = pool_eintraege().iter().filter(|e| e.1 == land && typen.contains(&e.2.as_str())).collect();
        let Some(e) = kand.get((zufall(&["sendung_wahl", &abs.to_string()]) * kand.len() as f32) as usize % kand.len().max(1)) else { return }; // roh
        self.me.sendung_zuletzt[art as usize] = abs;
        self.me.sendung_angebot = Some((e.0.clone(), art));
        let anlass = crate::inhalt::t(if erfolg { "ui.sendung.anlass.erfolg" } else if krise { "ui.sendung.anlass.krise" } else { "ui.sendung.anlass.ruf" });
        let opts = vec![(crate::id("zusage").to_string(), crate::inhalt::t("ui.sendung.zusage").to_string()), (crate::id("absage").to_string(), crate::inhalt::t("ui.sendung.absage").to_string())];
        self.msg(4, &e.0.clone(), &crate::tf!("ui.sendung.einladung_titel", e.0.clone()), &crate::tf!(if art == 0 { "ui.sendung.einladung_tv" } else { "ui.sendung.einladung_podcast" }, e.0.clone(), anlass), Action::Wahl { art: crate::id("sendung").into(), bezug: vec![], optionen: opts });
    }
    pub fn sendung_antwort(&mut self, key: &str) -> String {
        let Some((name, art)) = self.me.sendung_angebot.take() else { return crate::inhalt::t("ui.akademie.notiert").into() };
        if key != crate::id("zusage") { return crate::tf!("ui.sendung.abgesagt", name); }
        let tag = self.abs() + f("medien", "sendung.tage_bis") as u32;
        self.me.sendung = Some((name.clone(), art, tag));
        crate::tf!("ui.sendung.zugesagt", name)
    }
    /// Daily: on the day of an accepted appearance the show starts as a longer press scene with its factor.
    pub fn sendung_tag(&mut self) {
        let Some((name, art, tag)) = self.me.sendung.clone() else { return };
        if self.abs() < tag || self.press.is_some() || self.today_match().is_some() { return; }
        self.me.sendung = None;
        let mut q = self.press_question(false);
        let zweite = self.press_question(true);
        let mut rest = std::mem::take(&mut q.rest);
        let mut extra = vec![zweite.clone()];
        extra.extend(zweite.rest.clone());
        for mut x in extra { if !rest.iter().any(|r| r.thema == x.thema) && x.thema != q.thema { x.rest.clear(); rest.push(x); } }
        q.rest = rest;
        q.q = format!("{name}: {}", q.q);
        let faktor = if art == 0 { f("medien", "sendung.faktor_tv") } else { f("medien", "sendung.faktor_podcast") };
        self.me.sendung_aktiv = Some((name, faktor));
        self.press = Some(q);
    }
    /// Our first offer for player `p` (Fabian 24.11.): the bigger the deal, the likelier a transfer reporter asks
    /// the press office whether we are negotiating. Once per player within the rumour quiet time.
    pub fn presse_nachfrage(&mut self, p: usize) {
        let r = |k: &str| f("medien", &format!("nachfrage.{k}"));
        let abs = self.abs();
        if self.me.geruechte.iter().any(|g| g.0 == p && abs < g.1 + f("medien", "geruecht_angebot.sperre_tage") as u32) { return; }
        let pl = self.world.players[p].clone();
        let lvl = self.club().level;
        let chance = if pl.ovr >= lvl + r("gross_ab_ueber_niveau") || pl.value >= r("grosse_summe") as f64 { r(crate::id("gross")) } else if pl.ovr >= lvl { r(crate::id("mittel")) } else { 0.0 };
        if crate::mensch::zufall(&[crate::id("nachfrage"), &pl.id.to_string(), &abs.to_string()]) >= chance { return; }
        self.me.geruechte.push((p, abs, true, false));
        self.journalisten_an();
        let j = self.fuer_thema(crate::id("geruecht")).first().copied().unwrap_or(abs as usize % self.me.journalisten.len().max(1));
        let wer = self.me.journalisten.get(j).map(|x| x.0.clone()).unwrap_or_default();
        let blatt = { let b = self.blatt(j); if b.is_empty() { PAPERS[(abs as usize) % PAPERS.len()].to_string() } else { b } };
        let n = pl.name.clone();
        let von = if pl.team >= 0 { self.world.clubs[pl.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").to_string() };
        let (vt, vb, va, vd, vs) = (self.variante("nachfrage_titel", 3, &n), self.variante("nachfrage_body", 3, &n), self.variante("nachfrage_ja", 3, &n), self.variante("nachfrage_nein", 3, &n), self.variante("nachfrage_still", 3, &n));
        let t = |k: &str, v: usize| crate::inhalt::t(&format!("ui.nachfrage.{k}.{v}")).to_string();
        let opts = vec![(crate::id("bestaetigen").to_string(), t("ja", va)), (crate::id("dementieren").to_string(), t("nein", vd)), (crate::id("schweigen").to_string(), t("still", vs))];
        self.msg(4, &blatt, &crate::tf!(&format!("ui.nachfrage.titel.{vt}"), n.clone()), &crate::tf!(&format!("ui.nachfrage.body.{vb}"), wer, blatt.clone(), n, von), Action::Wahl { art: format!("nachfrage:{p}"), bezug: vec![p], optionen: opts });
        self.inbox[0].spieler = vec![p];
    }
    /// The press office's answer to a question about our talks with `p`, each with its visible effect.
    pub fn nachfrage_antwort(&mut self, p: usize, key: &str) -> String {
        let r = |k: &str| f("medien", &format!("nachfrage.{k}"));
        let name = self.world.players[p].name.clone();
        match key {
            "bestaetigen" => {
                let d = r("fans_bestaetigt");
                self.me.fan_aktion = (self.me.fan_aktion + d).min(15.0);
                self.grund(crate::id("Fans"), crate::inhalt::t("ui.nachfrage.wirkung_fans"), d, &crate::tf!("ui.nachfrage.grund_bestaetigt", name.clone()));
                let x = r("preis_bestaetigt") as f64;
                if let Some(n) = self.negs.iter_mut().rev().find(|n| n.pid == p && n.stage == 0 && !n.verkauf) { n.ask = crate::world::round_money(n.ask * (1.0 + x)); }
                self.grund(&name, crate::inhalt::t("ui.nachfrage.wirkung_preis"), (x * 100.0) as f32, crate::inhalt::t("ui.nachfrage.grund_preis"));
                crate::tf!("ui.nachfrage.ergebnis_ja", name)
            }
            "dementieren" => {
                let d = r("berater_dementi");
                self.berater_merken(p, d);
                self.grund(&name, crate::inhalt::t("ui.nachfrage.wirkung_berater"), d, crate::inhalt::t("ui.nachfrage.grund_dementi"));
                crate::tf!("ui.nachfrage.ergebnis_nein", name)
            }
            _ => crate::inhalt::t("ui.medien.wir_schweigen").into(),
        }
    }
    pub fn geruecht_antwort(&mut self, p: usize, key: &str) -> String {
        if key != crate::id("dementieren") { return crate::inhalt::t("ui.medien.wir_schweigen").into(); }
        if let Some(g) = self.me.geruechte.iter_mut().rev().find(|g| g.0 == p && !g.3) { g.3 = true; g.1 = self.world.season * crate::date::SEASON_DAYS + self.world.day; }
        crate::inhalt::t("ui.medien.dementiert_die_frist_laeuft_45").into()
    }
    /// Season end: the public promise of the season goal is kept or broken.
    pub fn zusagen_saisonende(&mut self, ziel_erreicht: bool) {
        for i in 0..self.me.zusagen.len() { if self.me.zusagen[i].0 == crate::id("saisonziel") && self.me.zusagen[i].4 == 0 { self.zusage_entscheiden(i, ziel_erreicht); } }
        self.me.zusagen.retain(|z| z.4 == 0);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sendung_einladung_und_auftritt() {
        let w = crate::world::testwelt(7);
        let mut c = crate::game::Career::new(w, 0, "Test");
        c.me.sendung_angebot = Some(("Rasenfunk".into(), 1));
        c.sendung_antwort("zusage");
        assert!(c.me.sendung.is_some());
        c.world.day += 3;
        if c.today_match().is_some() { c.world.day += 1; }
        c.sendung_tag();
        let q = c.press.clone().expect("the show starts as a press scene");
        assert!(q.q.starts_with("Rasenfunk") && !q.rest.is_empty());
        let f0 = c.fans;
        while c.press.is_some() { c.answer_press(Some(0)); }
        assert!(c.me.sendung_aktiv.is_none() && (c.fans - f0).abs() <= 8.0 + 1e-3);
    }
    #[test]
    fn dementiertes_geruecht_kommt_nicht_wieder() {
        let w = crate::world::testwelt(7);
        let mut c = crate::game::Career::new(w, 0, "Test");
        let p = c.squad()[0];
        c.geruecht(p, true);
        let n = c.me.geruechte.len();
        c.geruecht_antwort(p, "dementieren");
        c.world.day += 1;
        c.geruecht(p, true);
        assert_eq!(c.me.geruechte.len(), n, "no new rumour the day after a denial");
    }
    #[test]
    fn varianten_wiederholen_sich_nicht() {
        let w = crate::world::testwelt(7);
        let mut c = crate::game::Career::new(w, 0, "Test");
        let mut alt = usize::MAX;
        // a bag: five picks in a row are five different ones
        let erste: std::collections::HashSet<usize> = (0..5).map(|k| { c.world.day += 1; c.variante("beutel", 5, &k.to_string()) }).collect();
        assert_eq!(erste.len(), 5, "no repeat until the bag is empty");
        for k in 0..30 { let i = c.variante("probe", 5, &k.to_string()); assert!(i < 5 && i != alt, "{i} twice"); alt = i; c.world.day += 1; }
        // press conferences in a row do not open with the same question
        let a = c.press_question(false).q;
        let b = c.press_question(false).q;
        assert_ne!(a, b);
    }
    use super::*;
    #[test]
    fn pressekonferenz_vier_fragen_und_deckel() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let pq = c.press_question(false);
        let n = 1 + pq.rest.len();
        assert!(n >= 1 && n <= 4);
        c.press = Some(pq);
        let m0 = c.media;
        for _ in 0..n { let i = c.press.as_ref().map(|p| p.answers.iter().position(|a| a.1 % 10 == 0).unwrap_or(0)).unwrap(); c.answer_press(Some(i)); }
        assert!(c.press.is_none());
        assert!((c.media - m0).abs() <= 6.0 + 1e-3, "at most ±6 per target and conference");
        // not attending costs media 3
        c.press = Some(c.press_question(false));
        let m1 = c.media;
        c.answer_press(None);
        assert!((c.media - (m1 - 3.0)).abs() < 1e-3);
    }
    #[test]
    fn fans_transfer_und_proteste() {
        let w = crate::world::testwelt(7);
        let mut c = Career::new(w, 0, "Test");
        let u = c.umsatz_prognose();
        c.fans_transfer(u, true);
        assert_eq!(c.me.fan_transfer, 15.0, "capped at 15");
        c.fans_monat();
        assert!((c.me.fan_transfer - 12.0).abs() < 1e-3, "decays to 80 %");
        let (fans, medien) = (c.fans, c.media);
        let _ = fans;
        c.fanaktion(0).unwrap();
        assert!(c.fanaktion(0).is_err(), "locked");
        assert!(c.media > medien);
        assert_eq!(c.protest_stufe(), 0);
    }
}

/// The press pool (pesm/core/config/story/medienpool.json): (name, country code, tone) of 175 outlets.
/// Every entry of medienpool.json with its kind: (name, country, typ, tone).
pub fn pool_eintraege() -> &'static Vec<(String, String, String, String)> {
    static P: std::sync::OnceLock<Vec<(String, String, String, String)>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/inhalt/pesm/core/config/story/medienpool.json")).expect("medienpool.json");
        v["medien"].as_array().map(|a| a.iter().map(|m| (m["name"].as_str().unwrap_or("").to_string(), m["land"].as_str().unwrap_or("").to_string(), m["typ"].as_str().unwrap_or("").to_string(), m["ton"].as_str().unwrap_or("").to_string())).filter(|m| !m.0.is_empty()).collect()).unwrap_or_default()
    })
}
/// Beats a journalist can have (medien.toml journalist.fach).
pub const FAECHER: [&str; 5] = ["transfers", "taktik", "interna", "meinung", "spieler"];
/// Keywords of the media pool entries, lower case, by name.
fn pool_schlagworte() -> &'static std::collections::HashMap<String, Vec<String>> {
    static P: std::sync::OnceLock<std::collections::HashMap<String, Vec<String>>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/inhalt/pesm/core/config/story/medienpool.json")).expect("medienpool.json");
        v["medien"].as_array().map(|a| a.iter().map(|m| (m["name"].as_str().unwrap_or("").to_string(), m["tags"].as_array().map(|t| t.iter().filter_map(|x| x.as_str()).map(|x| x.to_lowercase()).collect()).unwrap_or_default())).collect()).unwrap_or_default()
    })
}
/// The country code of an outlet or reporter of the media pool ("" when unknown).
pub fn pool_land(name: &str) -> &'static str { pool_eintraege().iter().find(|e| e.0 == name).map_or("", |e| e.1.as_str()) }
pub fn medienpool() -> &'static Vec<(String, String, String)> {
    static P: std::sync::OnceLock<Vec<(String, String, String)>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/inhalt/pesm/core/config/story/medienpool.json")).expect("medienpool.json");
        v["medien"].as_array().map(|a| a.iter().map(|m| (m["name"].as_str().unwrap_or("").to_string(), m["land"].as_str().unwrap_or("").to_string(), m["ton"].as_str().unwrap_or("").to_string())).filter(|m| !m.0.is_empty()).collect()).unwrap_or_default()
    })
}
/// The press pool with the layout of the web blog: (name, country code, layout). Reporters are people, not outlets.
pub fn medienpool_voll() -> &'static Vec<(String, String, String)> {
    static P: std::sync::OnceLock<Vec<(String, String, String)>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/inhalt/pesm/core/config/story/medienpool.json")).expect("medienpool.json");
        v["medien"].as_array().map(|a| a.iter().filter_map(|m| {
            let (typ, ton, land) = (m["typ"].as_str()?, m["ton"].as_str().unwrap_or(""), m["land"].as_str().unwrap_or(""));
            let layout = match (typ, ton) {
                ("reporter", _) => return None,
                ("daten_app", _) => "datenportal",
                ("zeitung", "boulevard") => if land == "GB" { "tabloid_en" } else { "boulevard" },
                ("tv_radio", _) => "tv",
                ("podcast" | "youtube", _) => "podcast",
                ("vereinsmedium", _) => "verein",
                ("portal" | "magazin", "taktik" | "investigativ") | ("portal", "insider") => "analyse",
                ("magazin", _) => "fachmagazin",
                ("zeitung", "fankultur") => if land == "GB" { "serioes" } else { "lokal" },
                _ => "serioes",
            };
            Some((m["name"].as_str()?.to_string(), land.to_string(), layout.to_string()))
        }).collect()).unwrap_or_default()
    })
}
/// The core of a club name for a local paper ("Hertha BSC" → "Hertha", "FC Bayern München" → "Bayern München").
pub fn kern_name(n: &str) -> String {
    let weg = ["FC", "SC", "SV", "VfB", "VfL", "TSG", "1.", "BSC", "AFC", "CF", "SSC", "AS", "AC", "SL", "CD", "RCD", "UD", "SD", "FK", "SK", "BV", "Borussia", "Eintracht", "Hertha"];
    let teile: Vec<&str> = n.split_whitespace().filter(|t| !weg.contains(t) || *t == "Hertha" || *t == "Borussia" || *t == "Eintracht").collect(); // roh
    if teile.is_empty() { n.to_string() } else { teile.join(" ") }
}
/// Country code of the press pool for a league country (display name, German or English).
pub fn land_kuerzel(land: &str) -> &'static str {
    match land {
        "England" | "Wales" => "GB", "Schottland" | "Scotland" => "SC", "Deutschland" | "Germany" => "DE", "Spanien" | "Spain" => "ES", "Italien" | "Italy" => "IT",
        "Frankreich" | "France" => "FR", "Portugal" => "PT", "Österreich" | "Austria" => "AT", "Schweiz" | "Switzerland" => "CH", "Niederlande" | "Netherlands" => "NL",
        "Türkei" | "Turkey" => "TR", "Brasilien" | "Brazil" => "BR", "Argentinien" | "Argentina" => "AR", "Belgien" | "Belgium" => "BE", "Mexiko" | "Mexico" => "MX",
        "Japan" => "JP", "USA" => "US", "Saudi-Arabien" | "Katar" | "VAE" | "Saudi Arabia" | "Qatar" => "ME",
        _ => "INT",
    }
}

#[cfg(test)]
mod spieler_wirkung_tests {
    use crate::game::*;
    /// Fabian 24.11.: what we say about a player at the podium hits him directly – praise lifts morale and form,
    /// criticism costs, stronger for a big ego.
    #[test]
    fn antwort_ueber_spieler_wirkt_direkt() {
        let w = crate::world::testwelt(4);
        let mut c = Career::new(w, 1, "Test");
        let p = c.squad()[5];
        let mut probe = |c: &mut Career, ton: u8| -> (f32, f32) {
            c.world.players[p].form = 0.0;
            c.world.players[p].morale = 50.0;
            let mut q = c.press_question(false);
            q.bezug = Some(p);
            q.rest.clear();
            q.answers = vec![("x".into(), ton)];
            c.press = Some(q);
            c.answer_press(Some(0));
            // morale moves through its "talk" driver: the newest entry is this answer
            (c.world.players[p].ps.gespr.last().map_or(0.0, |x| x.1), c.world.players[p].form)
        };
        let lob = crate::medien::TOENE.iter().position(|t| *t == "lob").unwrap() as u8;
        let kritik = crate::medien::TOENE.iter().position(|t| *t == "kritik").unwrap() as u8;
        let (ml, fl) = probe(&mut c, lob);
        let (mk, fk) = probe(&mut c, kritik);
        assert!(ml > mk && fl > fk, "praise {ml}/{fl} must beat criticism {mk}/{fk}");
        assert!(fl > 0.0 && fk < 0.0 && ml > 0.0 && mk < 0.0);
    }
}

#[cfg(test)]
mod fach_tests {
    /// Fabian 24.11.: a big first offer brings a press question; confirming excites the fans and lifts the asking
    /// price, every move in the talks lands in the inbox with the player attached.
    #[test]
    fn nachfrage_bei_grossem_angebot() {
        let w = crate::world::testwelt(9);
        let mut c = crate::game::Career::new(w, 0, "Test");
        c.world.day = 3;
        let lvl = c.club().level;
        let kand: Vec<usize> = (0..c.world.players.len()).filter(|&p| { let pl = &c.world.players[p]; pl.team >= 0 && pl.team != c.own as i32 && pl.ovr >= lvl + 4.0 }).collect();
        let (p, i) = kand.into_iter().find_map(|p| c.open_neg(p).ok().map(|i| (p, i))).expect("one is for sale");
        c.verhandlungstempo = 1;
        c.submit_neg(i);
        let m = c.inbox.iter().position(|m| matches!(&m.action, crate::game::Action::Wahl { art, .. } if art == &format!("nachfrage:{p}"))).expect("the press asks");
        let ask = c.negs[i].ask;
        let f0 = c.me.fan_aktion;
        assert!(!c.nachfrage_antwort(p, "bestaetigen").is_empty());
        assert!(c.negs[i].ask > ask && c.me.fan_aktion > f0);
        assert!(m < c.inbox.len());
        // not twice for the same player
        let n = c.inbox.len();
        c.presse_nachfrage(p);
        assert_eq!(c.inbox.len(), n);
        // the answer to the offer arrives as a message with the player and a way back to the talks
        for _ in 0..4 { let _ = c.end_day(); }
        assert!(c.inbox.iter().any(|m| m.spieler == vec![p] && m.art == "angebot"), "a move in the talks reaches the inbox");
    }
    /// Fabian 24.11.: journalists have beats; a transfer reporter asks the transfer questions.
    #[test]
    fn faecher_und_fragesteller() {
        let mut c = crate::game::Career::new(crate::world::testwelt(4), 0, "Test");
        c.me.journalisten = vec![("Florian Plettenberg".into(), 1, 0.0), ("Raphael Honigstein".into(), 0, 0.0), ("Ines Vogt".into(), 2, 0.0)];
        assert_eq!(c.faecher(0)[0], "transfers");
        assert!(c.faecher(1).contains(&"taktik"));
        for j in 0..3 { assert_eq!(c.faecher(j).len(), 2, "{:?}", c.faecher(j)); }
        assert!(c.fuer_thema("geruecht").contains(&0));
        assert!(!c.fuer_thema("geruecht").contains(&1) || c.faecher(1).contains(&"transfers"));
    }
}
