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
        let pf = arr("vereinswelt", "einnahmen.preisstufen")[self.ticket as usize].as_f64().unwrap() as f32;
        // own decision: the price level sets the target of the economy mood (cheap +4, normal 0, dear −5)
        let ziel = ((1.0 - pf) * 20.0).clamp(-f("vereinswelt", "wirtschaft.stimmung_grenze"), f("vereinswelt", "wirtschaft.stimmung_grenze"));
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
    /// The outlet of journalist `j` ("" when unknown).
    pub fn blatt(&self, j: usize) -> String { self.me.blaetter.get(j).cloned().unwrap_or_default() }

    fn journalisten_an(&mut self) {
        if !self.me.journalisten.is_empty() { self.blaetter_an(); return; }
        let seed = self.club().id.to_string();
        let n = f("medien", "journalist.anzahl") as usize;
        for i in 0..n {
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
        if let Some(&(p, ..)) = self.me.geruechte.iter().rev().find(|g| self.world.players[g.0].team == self.own as i32) { t.push((crate::id("geruecht").into(), Some(p))); t.push(("zusage_spieler".into(), Some(p))); }
        if let Some(p) = self.squad().into_iter().filter(|&p| self.world.players[p].inj >= 14 && self.world.players[p].ps.rolle <= 3).max_by(|&a, &b| self.world.players[a].ovr.partial_cmp(&self.world.players[b].ovr).unwrap()) { t.push((crate::id("ausfall").into(), Some(p))); }
        if let Some(p) = self.squad().into_iter().filter(|&p| self.world.players[p].joined > 0 && self.abs() < self.world.players[p].joined + 30).max_by(|&a, &b| self.world.players[a].ovr.partial_cmp(&self.world.players[b].ovr).unwrap()) { t.push((crate::id("neuzugang").into(), Some(p))); }
        if !self.me.zusagen.iter().any(|z| z.0 == crate::id("saisonziel") && z.4 == 0) && self.world.day < 120 { t.push(("zusage_ziel".into(), None)); }
        let abs = self.abs();
        let wh = f("medien", "konferenz.wiederholung_tage") as u32;
        t.sort_by_key(|x| self.me.themen.iter().any(|y| y.0 == x.0 && abs < y.1 + wh) as u8);
        t
    }
    fn frage_text(&self, thema: &str, p: Option<usize>, k: usize) -> String {
        let fr = presse()["fragen"][thema].as_array().cloned().unwrap_or_default();
        let s = fr.get(k % fr.len().max(1)).and_then(|x| x.as_str()).unwrap_or(crate::inhalt::t("ui.medien.ihre_meinung"));
        self.platzhalter(s, thema, p)
    }
    /// {gegner}, {spieler}, {platz}, {ziel} in a question or an answer of the press conference.
    fn platzhalter(&self, s: &str, thema: &str, p: Option<usize>) -> String {
        let gegner = self.next_match().map(|f| self.world.clubs[if f.h == self.own { f.a } else { f.h }].name.clone())
            .or_else(|| self.last_result.as_ref().map(|r| self.world.clubs[if r.h == self.own { r.a } else { r.h }].name.clone())).unwrap_or_default();
        let gegner = if self.last_result.is_some() && thema != crate::id("standard") { self.last_result.as_ref().map(|r| self.world.clubs[if r.h == self.own { r.a } else { r.h }].name.clone()).unwrap_or(gegner) } else { gegner };
        s.replace("{gegner}", &gegner).replace("{spieler}", &p.map(|p| self.world.players[p].name.clone()).unwrap_or_default())
            .replace("{platz}", &self.world.rank_of(self.own).to_string()).replace("{ziel}", &self.club().goal_pos.to_string())
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
            let mut answers: Vec<(String, u8)> = toene.iter().filter_map(|t| t.as_str()).map(|t| {
                let v = presse()["antworten_thema"][&thema][t].as_array().cloned().unwrap_or_default();
                let roh = v.get((abs as usize + k) % v.len().max(1)).and_then(|x| x.as_str()).or(presse()["antworten"][t].as_str()).unwrap_or("");
                (self.platzhalter(roh, &thema, p), TOENE.iter().position(|x| *x == t).unwrap_or(0) as u8)
            }).collect();
            if thema.starts_with(crate::id("zusage")) {
                let z = presse()["zusage_antwort"][&thema].as_str().unwrap_or("").replace("{spieler}", &p.map(|p| self.world.players[p].name.clone()).unwrap_or_default());
                let ton = if thema == "zusage_ziel" { 1 } else { 0 };
                answers.insert(0, (z, ton + 10)); // + 10 marks the answer that gives the public promise
            }
            let q = self.frage_text(&thema, p, (abs as usize + k) % 3);
            let j = (zufall(&["2026", crate::id("frage"), &abs.to_string(), &k.to_string()]) * self.me.journalisten.len() as f32) as u8;
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
        let quote = pq.answers[i].0.clone();
        let rest = std::mem::take(&mut pq.rest);
        if let Some(mut next) = rest.first().cloned() {
            next.rest = rest[1..].to_vec();
            next.summe = pq.summe;
            self.press = Some(next);
            return;
        }
        let cap = f("medien", "konferenz.deckel");
        let s = pq.summe.map(|x| x.clamp(-cap, cap));
        let g = crate::inhalt::t("ui.medien.auftritt_auf_der_pressekonferenz");
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
            let z = |k: &str, n: usize| (crate::mensch::zufall(&[crate::id("pk"), k, &abs.to_string()]) * n as f32) as usize % n;
            let kurz: String = if quote.chars().count() > 70 { quote.chars().take(68).collect::<String>() + "…" } else { quote.clone() };
            self.news(&tt[z(crate::id("t"), tt.len())].replace("%1", &kurz), &tx[z(crate::id("x"), tx.len())].replace("%1", &quote));
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
            zeilen.push(crate::tf!(&format!("ui.zeitung.bezug.{art}"), kurz, tage, bilanz));
            if zeilen.len() >= 2 { break; }
        }
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
        if let Some(p) = key { if !gesperrt && zufall(&["2026", crate::id("falschgeruecht"), &abs.to_string()]) < f("medien", "geruecht.falsch_wahrscheinlichkeit") { self.geruecht(p, false); } }
    }
    /// A rumour about an own player (real after an offer, or made up): the person may deny it.
    pub fn geruecht(&mut self, p: usize, echt: bool) {
        let abs = self.abs();
        self.me.geruechte.push((p, abs, echt, false));
        let n = self.world.players[p].name.clone();
        // the rumour comes from a tabloid journalist of the own country's press (medienpool), else any outlet
        self.journalisten_an();
        let j = self.me.journalisten.iter().position(|x| x.1 == 1).unwrap_or(abs as usize % self.me.journalisten.len().max(1));
        let paper_s = { let b = self.blatt(j); if b.is_empty() { PAPERS[(abs as usize) % PAPERS.len()].to_string() } else { b } };
        let paper = paper_s.as_str();
        self.news(&crate::tf!("ui.medien.geruecht_vor_dem_absprung", n), &crate::tf!("ui.medien.laut_gibt_es_interesse_anderer", paper));
        let opts = vec![(crate::id("dementieren").to_string(), crate::inhalt::t("ui.medien.das_ist_unsinn_er_bleibt").to_string()), (crate::id("schweigen").to_string(), crate::inhalt::t("ui.medien.kein_kommentar").to_string())];
        self.msg(4, crate::inhalt::t("ui.medien.pressestelle"), &crate::tf!("ui.medien.geruecht_um", n), &crate::tf!("ui.medien.die_zeitungen_schreiben_ueber_einen", n), Action::Wahl { art: crate::id("geruecht").into(), bezug: vec![p], optionen: opts });
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
    use super::*;
    use crate::world::World;
    #[test]
    fn pressekonferenz_vier_fragen_und_deckel() {
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
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
        let w = World::load(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/world.json")).unwrap(), 7);
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
pub fn medienpool() -> &'static Vec<(String, String, String)> {
    static P: std::sync::OnceLock<Vec<(String, String, String)>> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/inhalt/pesm/core/config/story/medienpool.json")).expect("medienpool.json");
        v["medien"].as_array().map(|a| a.iter().map(|m| (m["name"].as_str().unwrap_or("").to_string(), m["land"].as_str().unwrap_or("").to_string(), m["ton"].as_str().unwrap_or("").to_string())).filter(|m| !m.0.is_empty()).collect()).unwrap_or_default()
    })
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
