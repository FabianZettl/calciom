//! The phone's views and commands (mobile-ansicht.md 2.4, Teil 3 Stufe 1): small JSON forms built on the PC from the
//! career (texts already translated), never the whole career; commands call the same functions as the PC buttons.
//! "Tag weiter" and "Bis zum nächsten Ereignis" need the app and are handled in main.rs.
use crate::game::{Action, Career};
use crate::world::{money, pos_de, FORMATIONS};
use serde_json::{json, Value};

fn t(k: &str) -> &'static str { crate::inhalt::t(k) }
fn farbe(c: u32) -> String { format!("#{:06x}", c & 0xffffff) }

/// Gives every message without a number one (saves from before the number).
pub fn nummern(c: &mut Career) {
    for i in (0..c.inbox.len()).rev() { if c.inbox[i].id == 0 { c.msg_id += 1; c.inbox[i].id = c.msg_id; } }
}

pub fn seite(c: &mut Career, name: &str, stand: u64) -> Value {
    nummern(c);
    let kopf = kopf(c, stand);
    let inhalt = match name {
        "stand" => Value::Null,
        "zentrale" => zentrale(c),
        "post" => post(c),
        "kader" => kader(c),
        "taktik" => taktik(c),
        "tabelle" => tabelle(c),
        "markt" => markt(c),
        "training" => training(c),
        _ => json!({"nur_pc": true}),
    };
    json!({"kopf": kopf, "seite": inhalt})
}

fn kopf(c: &Career, stand: u64) -> Value {
    let cl = c.club();
    let heute = c.today_match().is_some();
    json!({
        "stand": stand, "verein": cl.name, "vid": cl.id, "farbe": farbe(cl.c1), "farbe2": farbe(cl.c2), "kurz": cl.short,
        "datum": crate::date::short(c.world.season, c.world.day), "ungelesen": c.unread(), "offen": c.open_decisions(),
        "spiel_heute": heute, "deadline": c.deadline(), "stunde": c.dl_stunde,
        "weiter": if heute { t("ui.main.zum_spiel") } else if c.deadline() && c.dl_stunde < 12 { t("ui.dl.naechste_stunde") } else { t("ui.main.naechster_tag") },
        "presse": c.press.as_ref().map(|p| json!({"frage": p.q, "antworten": p.answers.iter().map(|a| a.0.clone()).collect::<Vec<_>>()})),
        "rueckgaengig": c.undo.is_some(),
    })
}

fn zentrale(c: &Career) -> Value {
    let naechstes = c.next_match().map(|f| {
        let heim = f.h == c.own;
        let g = if heim { f.a } else { f.h };
        json!({"gegner": c.world.clubs[g].name, "gid": c.world.clubs[g].id, "heim": heim, "datum": crate::date::short(c.world.season, f.day), "tage": f.day.saturating_sub(c.world.day), "farbe": farbe(c.world.clubs[g].c1)})
    });
    let tab = c.world.table(c.league());
    let platz = tab.iter().position(|r| r.club == c.own).map(|i| i + 1).unwrap_or(0);
    let letzte: Vec<Value> = c.spiele.iter().rev().take(5).map(|s| json!({"gegner": c.world.clubs[s.gegner].short, "tore": s.tore, "gegentore": s.gegentore, "heim": s.heim})).collect();
    let wichtig: Vec<Value> = c.inbox.iter().filter(|m| !m.done && m.action != Action::None && !matches!(m.action, Action::Report { .. })).take(4).map(|m| json!({"id": m.id, "titel": m.title, "von": m.from})).collect();
    json!({
        "naechstes": naechstes, "platz": platz, "teams": tab.len(), "liga": c.world.leagues[c.league()].name, "ziel": c.club().goal_pos, "ziel_wort": c.club().goal_kind,
        "vertrauen": c.mensch.vorstand.round(), "fans": c.fans.round(), "kasse": money(c.club().balance), "budget": money(c.transfer_budget),
        "letzte": letzte, "wichtig": wichtig,
    })
}

fn post(c: &Career) -> Value {
    let cats = crate::game::cats_t();
    let liste: Vec<Value> = c.inbox.iter().take(60).map(|m| {
        let optionen: Vec<Value> = match &m.action {
            _ if m.done => vec![],
            Action::Wahl { optionen, .. } => optionen.iter().enumerate().map(|(k, o)| json!({"wahl": k, "text": o.1})).collect(),
            Action::TransferOffer { .. } | Action::Complaint { .. } | Action::Graduate { .. } | Action::Expiring { .. } => vec![json!({"wahl": 0, "ja": true, "text": t("ui.handy.ja")}), json!({"wahl": 0, "ja": false, "text": t("ui.handy.nein")})],
            Action::JobOffer { .. } => vec![json!({"wahl": 0, "ja": true, "text": t("ui.handy.annehmen")}), json!({"wahl": 1, "ja": true, "text": t("ui.handy.verhandeln")}), json!({"wahl": 0, "ja": false, "text": t("ui.handy.ablehnen")})],
            _ => vec![],
        };
        json!({"id": m.id, "titel": m.title, "von": m.from, "text": m.body, "gelesen": m.read, "erledigt": m.done, "datum": crate::date::short(m.season, m.day), "bereich": cats.get(m.cat as usize).copied().unwrap_or(""), "optionen": optionen, "pflicht": m.pflicht})
    }).collect();
    json!({"liste": liste})
}

fn kader(c: &Career) -> Value {
    let liste: Vec<Value> = c.squad().into_iter().map(|p| {
        let pl = &c.world.players[p];
        json!({"p": p, "fid": pl.id, "nr": pl.no, "name": pl.name, "pos": pos_de(&pl.pos), "alter": pl.age, "staerke": pl.ovr.round(), "potenzial": pl.pot.round(), "fitness": pl.fit.round(), "moral": pl.morale.round(),
            "form": pl.form, "verletzt": pl.inj, "gesperrt": pl.susp, "elf": c.xi.contains(&p), "wert": money(pl.value), "vertrag": crate::date::START_YEAR as u32 + pl.contract_end + 1,
            "tore": pl.goals, "spiele": pl.apps, "note": if pl.rn > 0 { (pl.avg_rating() * 10.0).round() / 10.0 } else { 0.0 }})
    }).collect();
    json!({"liste": liste})
}

fn taktik(c: &Career) -> Value {
    let slots = crate::world::formation_slots(c.formation);
    let elf: Vec<Value> = c.xi.iter().enumerate().map(|(k, &p)| { let pl = &c.world.players[p]; json!({"p": p, "platz": pos_de(slots.get(k).copied().unwrap_or("")), "name": pl.name, "pos": pos_de(&pl.pos), "staerke": pl.ovr.round(), "fitness": pl.fit.round()}) }).collect();
    let bank: Vec<Value> = c.squad().into_iter().filter(|p| !c.xi.contains(p)).map(|p| { let pl = &c.world.players[p]; json!({"p": p, "name": pl.name, "pos": pos_de(&pl.pos), "staerke": pl.ovr.round(), "fitness": pl.fit.round(), "verletzt": pl.inj > 0 || pl.susp > 0}) }).collect();
    json!({"formation": c.formation, "formationen": FORMATIONS, "elf": elf, "bank": bank})
}

fn tabelle(c: &Career) -> Value {
    let l = c.league();
    let zeilen: Vec<Value> = c.world.table(l).iter().enumerate().map(|(i, r)| json!({"platz": i + 1, "verein": c.world.clubs[r.club].name, "eigen": r.club == c.own, "sp": r.p, "tore": format!("{}:{}", r.gf, r.ga), "pkt": r.pts})).collect();
    json!({"liga": c.world.leagues[l].name, "zeilen": zeilen})
}

/// The market from the sofa (Stufe 2, first part): the scouts' reports with the verdict, and the shortlist.
fn markt(c: &Career) -> Value {
    let zeile = |p: usize| {
        let pl = &c.world.players[p];
        let (lo, hi) = c.ovr_range(p);
        let urteil = if c.know_of(p) >= crate::regeln::f("scouting", "wissen.bericht_ab") / 100.0 { t(&format!("ui.sb.urteil.{}", c.scout_urteil(p).urteil)) } else { "" };
        json!({"p": p, "fid": pl.id, "name": pl.name, "pos": pos_de(&pl.pos), "alter": pl.age, "verein": if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { t("ui.wort.vereinslos").to_string() },
            "staerke": if hi - lo <= 1.0 { format!("{:.0}", (lo + hi) / 2.0) } else { format!("{lo:.0}–{hi:.0}") }, "preis": if pl.team >= 0 { money(c.ask_price(p)) } else { t("iq.wechsel.frei").to_string() }, "urteil": urteil, "gemerkt": c.merkliste.contains(&p)})
    };
    let mut berichte: Vec<usize> = c.inbox.iter().filter_map(|m| if let Action::Report { pid } = m.action { Some(pid) } else { None }).filter(|&p| c.world.players[p].team != c.own as i32).collect();
    berichte.dedup();
    json!({"berichte": berichte.into_iter().take(15).map(zeile).collect::<Vec<_>>(), "merkliste": c.merkliste.iter().map(|&p| zeile(p)).collect::<Vec<_>>(), "budget": money(c.transfer_budget), "fenster": c.in_window().is_some()})
}

/// The training week to read, and the ready-made weeks to pick.
fn training(c: &mut Career) -> Value {
    c.wochenplan_an();
    let tage: Vec<Value> = c.wochenplan.tage.iter().enumerate().map(|(d, tag)| json!({"tag": t(&format!("training.tag.{d}")), "typ": t(&format!("tp.typ.{}", crate::trainingsplan::TYPEN[tag.typ as usize])),
        "einheiten": tag.slots.iter().map(|s| s.as_ref().map(|z| crate::trainingsplan::name(&z.einheit)).unwrap_or("–")).collect::<Vec<_>>()})).collect();
    let vorlagen: Vec<Value> = crate::trainingsplan::werk().into_iter().map(|(id, _)| json!({"id": id, "name": t(&format!("tp.vorlage.{id}.name"))})).collect();
    json!({"tage": tage, "vorlagen": vorlagen, "co": c.co_training})
}

/// A command from the phone (except the day steps, main.rs). Answer {ok, meldung}.
pub fn befehl(c: &mut Career, b: &Value) -> Value {
    nummern(c);
    let was = b["befehl"].as_str().unwrap_or("");
    let ok = |m: &str| json!({"ok": true, "meldung": m});
    let fehler = |m: &str| json!({"ok": false, "meldung": m});
    match was {
        "rueckgaengig" => if c.rueckgaengig() { ok(t("ui.handy.rueckgaengig_ok")) } else { fehler(t("ui.handy.nichts_rueckgaengig")) },
        "speichern" => match c.save_mit_sicherung() { Ok(()) => ok(t("ui.pages.gesichert")), Err(e) => fehler(&e) },
        "post_gelesen" => {
            let id = b["id"].as_u64().unwrap_or(0);
            match c.inbox.iter_mut().find(|m| m.id == id) { Some(m) => { m.read = true; ok("") } None => fehler(t("ui.handy.nachricht_weg")) }
        }
        "post_entscheiden" => {
            let id = b["id"].as_u64().unwrap_or(0);
            let Some(i) = c.inbox.iter().position(|m| m.id == id) else { return fehler(t("ui.handy.nachricht_weg")) };
            if c.inbox[i].done { return fehler(t("ui.handy.schon_erledigt")); }
            c.undo_merken();
            c.answer(i, b["ja"].as_bool().unwrap_or(true), b["wahl"].as_u64().unwrap_or(0) as u8);
            ok(t("ui.handy.entschieden"))
        }
        "presse" => {
            if c.press.is_none() { return fehler(t("ui.handy.keine_presse")); }
            c.answer_press(b["wahl"].as_u64().map(|x| x as usize));
            ok(t("ui.handy.geantwortet"))
        }
        "formation" => { let f = b["nr"].as_u64().unwrap_or(0) as usize; if f < FORMATIONS.len() { c.formation = f; ok(FORMATIONS[f]) } else { fehler("") } }
        "co_aufstellen" => { c.auto_lineup(); ok(t("ui.handy.co_aufgestellt")) }
        "merken" => {
            let p = b["p"].as_u64().unwrap_or(u64::MAX) as usize;
            if p >= c.world.players.len() { return fehler(""); }
            if c.merkliste.contains(&p) { c.merkliste.retain(|&x| x != p); ok(t("ui.handy.von_merkliste")) } else { c.merkliste.push(p); ok(t("ui.post.auf_merkliste")) }
        }
        "trainingswoche" => {
            let id = b["id"].as_str().unwrap_or("");
            match crate::trainingsplan::werk().into_iter().find(|w| w.0 == id) { Some(w) => { c.wochenplan = w.1; c.plan_uebernehmen(); ok(t("tp.ok.uebernommen")) } None => fehler("") }
        }
        "tauschen" => {
            let (a, z) = (b["a"].as_u64().unwrap_or(u64::MAX) as usize, b["b"].as_u64().unwrap_or(u64::MAX) as usize);
            let sq = c.squad();
            if !sq.contains(&a) || !sq.contains(&z) { return fehler(t("ui.handy.nicht_im_kader")); }
            match (c.xi.iter().position(|&x| x == a), c.xi.iter().position(|&x| x == z)) {
                (Some(i), Some(j)) => c.xi.swap(i, j),
                (Some(i), None) => c.xi[i] = z,
                (None, Some(j)) => c.xi[j] = a,
                _ => return fehler(t("ui.handy.einer_in_der_elf")),
            }
            ok(t("ui.handy.getauscht"))
        }
        "simulieren" => {
            if c.today_match().is_none() { return fehler(t("ui.handy.kein_spiel")); }
            c.undo_merken();
            c.press = None;
            let res = c.prepare_match(None);
            c.apply_own_match(res);
            let r = c.last_result.clone().unwrap_or_default();
            let (eig, geg) = if r.h == c.own { (r.hg, r.ag) } else { (r.ag, r.hg) };
            let _ = c.save_mit_sicherung();
            // the events for the phone's minute-by-minute replay (goals, cards, substitutions, chances)
            let name = |p: usize| c.world.players[p].name.clone();
            let ereignisse: Vec<Value> = r.events.iter().map(|e| json!({"min": e.min, "seite": e.side, "art": e.kind, "name": name(e.pid), "zweiter": e.pid2.map(name)})).collect();
            json!({"ok": true, "meldung": format!("{} {}:{} {}", c.world.clubs[r.h].name, r.hg, r.ag, c.world.clubs[r.a].name),
                "ergebnis": {"eigen": eig, "gegner": geg, "bericht": r.reason, "heim": c.world.clubs[r.h].name, "gast": c.world.clubs[r.a].name, "hid": c.world.clubs[r.h].id, "aid": c.world.clubs[r.a].id,
                    "hfarbe": farbe(c.world.clubs[r.h].c1), "afarbe": farbe(c.world.clubs[r.a].c1), "hg": r.hg, "ag": r.ag, "ereignisse": ereignisse}})
        }
        _ => fehler(t("ui.handy.nur_pc")),
    }
}
