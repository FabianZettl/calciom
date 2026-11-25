use super::*;

/// Konzept Scouting 5.2 / Teil 7 "Auffällige Zahlen": the analyst's list of the week, each with "send a scout".
fn analyst_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    // data packages (Statistik 8.1): extended numbers of a league for the season, bought here
    let dk = Rect::new(x0() + 4.0, hoehe() - 150.0, cbreite(), 134.0);
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0 - 146.0);
    ui.card_titled(dk, t("ui.daten.titel"), Theme::gold());
    {
        let mut ligen: Vec<usize> = (0..c.world.leagues.len()).filter(|&l| c.world.leagues[l].detail == 2 && c.world.leagues[l].teams.len() >= 8 && !c.analyst_ligen().contains(&l)).collect();
        // the leagues that fit the club (Fabian 24.11.): where our signings would realistically come from, i.e. the
        // corridor from that league to ours (transferki.rs), then strength; leagues without a corridor are left out
        let eigene = c.league();
        let passt = |l: usize| crate::transferki::korridor(&c.world, l, eigene, 25);
        ligen.retain(|&l| passt(l) > 0.05);
        ligen.sort_by(|a, b| passt(*b).partial_cmp(&passt(*a)).unwrap().then(crate::statistik::liga_staerke(&c.world, *b).partial_cmp(&crate::statistik::liga_staerke(&c.world, *a)).unwrap()));
        let bw = (dk.w - 28.0 - 4.0 * 10.0) / 5.0;
        let mut kauf = None;
        for (k, &l) in ligen.iter().take(5).enumerate() {
            let b = Rect::new(dk.x + 14.0 + k as f32 * (bw + 10.0), dk.y + 50.0, bw, 64.0);
            rrect(b.x, b.y, b.w, b.h, 8.0, Theme::erhoehung());
            ui.text(&ui.fit(&c.world.leagues[l].name, 14.0, bw - 16.0), b.x + 8.0, b.y + 20.0, 14.0, Theme::text());
            ui.text(&money(c.datenpaket_preis(l)), b.x + 8.0, b.y + 40.0, 13.0, Theme::text2());
            if ui.button(Rect::new(b.x + b.w - 74.0, b.y + 36.0, 66.0, 22.0), t("ui.daten.kaufen"), false) { kauf = Some(l); }
        }
        if ligen.is_empty() { ui.text(t("ui.daten.alle"), dk.x + 14.0, dk.y + 70.0, 14.0, Theme::text2()); }
        if let Some(l) = kauf { match c.datenpaket_kaufen(l) { Ok(s) => v.toast(&s, true), Err(e) => v.toast(&e, false) } }
    }
    ui.card_titled(r, t("ui.analyst.titel"), Theme::akzent());
    let Some(s) = c.analyst() else {
        ui.paragraph(t(if c.analyst_erlaubt() { "ui.analyst.keiner" } else { "ui.analyst.zu_klein" }), r.x + 14.0, r.y + 60.0, 15.0, r.w - 28.0, Theme::text2());
        return;
    };
    let ligen = c.analyst_ligen().len();
    ui.text(&crate::tf!("ui.analyst.kopf", c.scouts[s].name.clone(), ligen), r.x + 14.0, r.y + 58.0, 14.0, Theme::text2());
    if c.analyst_liste.is_empty() { ui.paragraph(t("ui.analyst.leer"), r.x + 14.0, r.y + 90.0, 15.0, r.w - 28.0, Theme::text2()); return; }
    player_row_head(ui, r.x, r.y + 86.0, &[(t("ui.wort.gross_name"), 14.0), (t("ui.wort.gross_verein"), r.w * 0.30), (t("ui.stat.einfluss"), r.w * 0.55), (t("ui.markt.wert"), r.w * 0.70)]);
    let mut y = r.y + 116.0;
    let mut klick = None;
    let belegt: Vec<String> = c.auftraege.iter().filter(|x| !x.fertig).map(|x| x.scout.clone()).collect();
    for &p in c.analyst_liste.clone().iter() {
        let pl = c.world.players[p].clone();
        let rr = Rect::new(r.x + 8.0, y - 20.0, r.w * 0.80, 30.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
        ui.player_photo(r.x + 26.0, y - 5.0, 11.0, pl.id, &pl.name);
        ui.pos_tag(r.x + 44.0, y - 5.0, pos_de(&pl.pos));
        ui.text(&ui.fit(&format!("{} · {}", pl.name, pl.age), 15.0, r.w * 0.27 - 74.0), r.x + 88.0, y, 15.0, Theme::text());
        let (club, liga) = if pl.team >= 0 { let k = pl.team as usize; (c.world.clubs[k].name.clone(), c.world.clubs[k].league) } else { (String::new(), usize::MAX) };
        ui.text(&ui.fit(&club, 14.0, r.w * 0.24), r.x + r.w * 0.30, y, 14.0, Theme::text2());
        let (st, _) = crate::statistik::saison_von(&c.world, p);
        if let Some(e) = crate::statistik::einfluss90(&st) {
            let um = if liga < c.world.leagues.len() { crate::statistik::umrechnen(&c.world, e, liga, c.league(), 0) } else { e };
            ui.text(&crate::tf!("ui.analyst.wert", format!("{e:.2}"), format!("{um:.2}")), r.x + r.w * 0.55, y, 14.0, Theme::akzent());
        }
        ui.text(&money(pl.value), r.x + r.w * 0.70, y, 14.0, Theme::text());
        if ui.hit(rr) { klick = Some(p); }
        let laeuft = c.auftraege.iter().any(|x| !x.fertig && x.spieler == Some(p));
        if ui.button(Rect::new(r.x + r.w - 190.0, y - 20.0, 176.0, 28.0), t(if laeuft { "ui.scout.laeuft" } else { "ui.scout.schicken" }), false) && !laeuft {
            match c.scouts.iter().enumerate().find(|(i, x)| *i != s && !belegt.contains(&x.name)).map(|(_, x)| x.name.clone()) {
                Some(scout) => { let x = crate::auftraege::Auftrag { art: crate::auftraege::BEOBACHTEN, scout, spieler: Some(p), ..Default::default() }; match c.auftrag_anlegen(x, 21) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
                None => v.toast(t("ui.pages.alle_scouts_haben_schon_einen"), false),
            }
        }
        y += 36.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
}

/// The model player typed for a "like him" search: the best-known match of the name (at least three letters).
fn vorbild_finden(c: &Career, q: &str) -> Option<usize> {
    let q = q.trim().to_lowercase();
    if q.chars().count() < 3 { return None; }
    (0..c.world.players.len()).filter(|&p| c.world.players[p].name.to_lowercase().contains(&q)).max_by(|a, b| c.world.players[*a].ovr.partial_cmp(&c.world.players[*b].ovr).unwrap())
}
/// The form for a new assignment (kept between frames).
#[derive(Clone)]
struct AuftragForm { art: usize, gruppe: usize, alter: (u8, u8), liga: usize, scout: usize, knapp: bool, lang: bool, spieler: usize, profil: usize, vorbild: String, vorbild_fokus: bool, weite: usize }
thread_local! { static FORM: std::cell::RefCell<AuftragForm> = std::cell::RefCell::new(AuftragForm { art: 0, gruppe: 2, alter: (17, 28), liga: 0, scout: 0, knapp: false, lang: false, spieler: 0, profil: 0, vorbild: String::new(), vorbild_fokus: false, weite: 0 }); }
const GRUPPEN: [&str; 9] = [crate::id("tw"), crate::id("iv"), crate::id("av"), crate::id("dm"), crate::id("zm"), crate::id("om"), crate::id("fl"), crate::id("hs"), crate::id("st")];

/// Scouting assignments (konzepte/scouting.md Teil 3 and 7): a new one on the left, the running ones with their proposals
/// on the right. Watching one player and a second opinion start from the player profile and the report.
fn auftraege_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let mut fm = FORM.with(|f| f.borrow().clone());
    let l = Rect::new(x0() + 4.0, Y0, cbreite() * 0.38, hoehe() - Y0 - 16.0);
    ui.card_titled(l, t("ui.auftrag.neu_titel"), Theme::akzent());
    let mut y = l.y + 52.0;
    let arten = [t("ui.auftrag.art.suche"), t("ui.auftrag.art.gebiet"), t("ui.auftrag.art.nachfolger")];
    if let Some(i) = ui.segment(Rect::new(l.x + 14.0, y, l.w - 28.0, 32.0), &arten, fm.art) { fm.art = i; }
    y += 46.0;
    // the scout: free ones only
    let frei: Vec<usize> = (0..c.scouts.len()).filter(|&s| !c.auftraege.iter().any(|x| !x.fertig && x.scout == c.scouts[s].name)).collect();
    let zeile = |ui: &mut Ui, y: f32, label: &str, wert: &str| -> (bool, bool) {
        ui.text_semi(label, l.x + 14.0, y + 20.0, 14.0, Theme::text2());
        let links = ui.button(Rect::new(l.x + l.w * 0.38, y, 32.0, 30.0), "<", false);
        ui.text_center(&ui.fit(wert, 14.0, l.w * 0.62 - 90.0), l.x + l.w * 0.38 + (l.w * 0.62 - 14.0) / 2.0, y + 20.0, 14.0, Theme::text());
        let rechts = ui.button(Rect::new(l.x + l.w - 46.0, y, 32.0, 30.0), ">", false);
        (links, rechts)
    };
    let scout_name = frei.get(fm.scout % frei.len().max(1)).map(|&s| c.scouts[s].name.clone()).unwrap_or_else(|| t("ui.auftrag.kein_freier").to_string());
    // the free scouts as cards with face, region and the four abilities (Fabian 24.11.): a click picks one
    if frei.is_empty() { ui.text(t("ui.auftrag.kein_freier"), l.x + 14.0, y + 20.0, 14.0, Theme::problem()); y += 34.0; } else {
        let n = frei.len().min(3);
        let kw = (l.w - 28.0 - 8.0 * (n as f32 - 1.0)) / n as f32;
        let start = (fm.scout % frei.len()).saturating_sub(n - 1).min(frei.len() - n);
        for (k, &si) in frei.iter().enumerate().skip(start).take(n) {
            let kr = Rect::new(l.x + 14.0 + (k - start) as f32 * (kw + 8.0), y, kw, 112.0);
            let an = k == fm.scout % frei.len();
            rrect(kr.x, kr.y, kr.w, kr.h, 8.0, if an { alpha(Theme::akzent(), 0.16) } else { Theme::erhoehung() });
            rrect_line(kr.x, kr.y, kr.w, kr.h, 8.0, if an { Theme::akzent() } else { Theme::linie() });
            let name = c.scouts[si].name.clone();
            ui.figur(kr.x + 24.0, kr.y + 26.0, 16.0, &name, crate::ui::Rolle::Scout);
            ui.text_semi(&ui.fit(&name, 13.0, kr.w - 52.0), kr.x + 46.0, kr.y + 22.0, 13.0, Theme::text());
            let region = c.scouts[si].league.map(|lg| c.world.leagues[lg].name.clone()).unwrap_or_else(|| t("ui.auftrag.welt").to_string());
            ui.text(&ui.fit(&region, 11.0, kr.w - 52.0), kr.x + 46.0, kr.y + 38.0, 11.0, Theme::text2());
            let f = c.person(si).f;
            for (j, n) in crate::scouting::faehigkeiten_t().iter().enumerate() {
                let yy = kr.y + 56.0 + j as f32 * 13.0;
                ui.text(&ui.fit(n, 10.0, kr.w * 0.45), kr.x + 8.0, yy + 4.0, 10.0, Theme::gedaempft());
                ui.bar(kr.x + kr.w * 0.5, yy - 3.0, kr.w * 0.46, 5.0, f[j] / 100.0, Theme::akzent());
            }
            if ui.hit(kr) { fm.scout = k; }
        }
        if frei.len() > n {
            if ui.button(Rect::new(l.x + l.w - 74.0, y + 116.0, 28.0, 22.0), "<", false) { fm.scout = (fm.scout + frei.len() - 1) % frei.len(); } // roh
            if ui.button(Rect::new(l.x + l.w - 42.0, y + 116.0, 28.0, 22.0), ">", false) { fm.scout = (fm.scout + 1) % frei.len(); } // roh
        }
        y += 140.0;
    }
    // where: the own league first, then the leagues the scout may work in, strongest first
    let s_idx = frei.get(fm.scout % frei.len().max(1)).copied();
    let mut ligen: Vec<usize> = vec![c.league()];
    if let Some(s) = s_idx {
        let mut andere: Vec<usize> = (0..c.world.leagues.len()).filter(|&k| k != c.league() && c.world.leagues[k].teams.len() >= 8 && c.gebiet_erlaubt(s, Some(k))).collect();
        andere.sort_by(|a, b| crate::statistik::liga_staerke(&c.world, *b).partial_cmp(&crate::statistik::liga_staerke(&c.world, *a)).unwrap());
        ligen.extend(andere);
    }
    let mut lg = ligen[fm.liga % ligen.len()];
    // country, then league, each a list with search (Fabian 23.11.)
    ui.text_semi(t("ui.auftrag.gebiet"), l.x + 14.0, y + 24.0, 14.0, Theme::text2());
    let gebiet = ligen_gruppen(c, |k| ligen.contains(&k));
    let gebiet_felder = wettbewerb_felder(c, &gebiet, lg, Rect::new(l.x + l.w * 0.38, y, l.w * 0.62 - 14.0, 36.0));
    if let Some((f, i)) = auswahl_vorher(ui, v, &gebiet_felder) { lg = wahl_umsetzen(&gebiet, lg, f, i); fm.liga = ligen.iter().position(|&k| k == lg).unwrap_or(0); }
    y += 46.0;
    // how far: this league, its country, its continent or the world (Fabian 24.11., every kind of assignment)
    let weiten = [t("ui.auftrag.weite.0"), t("ui.auftrag.weite.1"), t("ui.auftrag.weite.2"), t("ui.auftrag.weite.3")];
    if let Some(i) = ui.segment(Rect::new(l.x + 14.0, y, l.w - 28.0, 30.0), &weiten, fm.weite) { fm.weite = i; }
    y += 42.0;
    let weite_wo = match fm.weite { 1 => crate::preise::liga_land(&c.world, lg), 2 => crate::transferki::land_info(&crate::preise::liga_land(&c.world, lg)).0, _ => String::new() };
    let sq = c.squad();
    match fm.art {
        0 => {
            ui.text_semi(t("ui.auftrag.position"), l.x + 14.0, y + 20.0, 14.0, Theme::text2());
            y += 28.0;
            let namen: Vec<&str> = GRUPPEN.iter().map(|g| t(&format!("ui.auftrag.gruppe.{g}"))).collect();
            if let Some(i) = ui.segment(Rect::new(l.x + 14.0, y, l.w - 28.0, 30.0), &namen, fm.gruppe) { fm.gruppe = i; fm.profil = 0; }
            y += 42.0;
            // what kind of player: a profile ("a number ten with many assists", "a ball-winning centre-back who steps
            // forward", "someone like …") instead of only a position (Fabian 24.11.)
            let liste = crate::regeln::wert_oder("scouting", "profil_liste").and_then(|v| v.as_array().cloned()).unwrap_or_default();
            let profile: Vec<String> = liste.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
            let pname = if fm.profil == 0 { t("ui.auftrag.profil.keins").to_string() } else { t(&format!("ui.auftrag.profil.{}", profile[(fm.profil - 1) % profile.len().max(1)])).to_string() };
            let (a, b) = zeile(ui, y, t("ui.auftrag.profil"), &pname);
            let n = profile.len() + 1;
            if a { fm.profil = (fm.profil + n - 1) % n; } if b { fm.profil = (fm.profil + 1) % n; }
            if (a || b) && fm.profil > 0 {
                let k = &profile[fm.profil - 1];
                if let Some(g) = crate::regeln::wert_oder("scouting", &format!("profil.{k}.gruppe")).and_then(|v| v.as_str()) { if let Some(i) = GRUPPEN.iter().position(|x| *x == g) { fm.gruppe = i; } }
            }
            y += 40.0;
            if fm.profil > 0 && profile.get(fm.profil - 1).map(|s| s.as_str()) == Some(crate::id("wie")) {
                let ir = Rect::new(l.x + 14.0, y, l.w - 28.0, 32.0);
                if ui.hit(ir) { fm.vorbild_fokus = true; } else if ui.clicked { fm.vorbild_fokus = false; }
                ui.input(ir, &mut fm.vorbild, fm.vorbild_fokus, t("ui.auftrag.vorbild_suche"));
                y += 38.0;
                if let Some(q) = vorbild_finden(c, &fm.vorbild) {
                    let pl = &c.world.players[q];
                    let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { String::new() };
                    ui.player_photo(l.x + 26.0, y + 10.0, 11.0, pl.id, &pl.name);
                    ui.text(&ui.fit(&format!("{} · {}", pl.name, club), 13.0, l.w - 60.0), l.x + 44.0, y + 15.0, 13.0, Theme::akzent());
                    y += 28.0;
                }
            }
            let (a, b) = zeile(ui, y, t("ui.auftrag.alter_von"), &fm.alter.0.to_string());
            if a { fm.alter.0 = fm.alter.0.saturating_sub(1).max(16); } if b { fm.alter.0 = (fm.alter.0 + 1).min(fm.alter.1); }
            y += 38.0;
            let (a, b) = zeile(ui, y, t("ui.auftrag.alter_bis"), &fm.alter.1.to_string());
            if a { fm.alter.1 = fm.alter.1.saturating_sub(1).max(fm.alter.0); } if b { fm.alter.1 = (fm.alter.1 + 1).min(38); }
            y += 42.0;
            if ui.button(Rect::new(l.x + 14.0, y, l.w - 28.0, 30.0), t("ui.auftrag.knapp"), fm.knapp) { fm.knapp = !fm.knapp; }
            y += 40.0;
        }
        2 => {
            let p = sq.get(fm.spieler % sq.len().max(1)).copied();
            let n = p.map(|p| format!("{} ({})", c.world.players[p].name, pos_de(&c.world.players[p].pos))).unwrap_or_default();
            let (a, b) = zeile(ui, y, t("ui.auftrag.fuer"), &n);
            if a { fm.spieler = (fm.spieler + sq.len().max(1) - 1) % sq.len().max(1); } if b { fm.spieler = (fm.spieler + 1) % sq.len().max(1); }
            y += 42.0;
        }
        _ => {}
    }
    if let Some(i) = ui.segment(Rect::new(l.x + 14.0, y, l.w - 28.0, 30.0), &[t("ui.auftrag.tage14"), t("ui.auftrag.tage28")], fm.lang as usize) { fm.lang = i == 1; }
    y += 44.0;
    y = ui.paragraph(&crate::tf!("ui.auftrag.budget_hinweis", money(c.transfer_frei()), money(c.gehalt_frei())), l.x + 14.0, y, 13.0, l.w - 28.0, Theme::gedaempft()) + 10.0;
    // what the scout is told, in one sentence
    let was = match fm.art {
        0 => if fm.profil > 0 { let liste = crate::regeln::wert_oder("scouting", "profil_liste").and_then(|v| v.as_array().cloned()).unwrap_or_default(); let k = liste.get(fm.profil - 1).and_then(|x| x.as_str()).unwrap_or(""); if k == crate::id("wie") { crate::tf!("ui.auftrag.satz.wie", vorbild_finden(c, &fm.vorbild).map(|q| c.world.players[q].name.clone()).unwrap_or_else(|| "?".into())) } else { t(&format!("ui.auftrag.satz.profil.{k}")).to_string() } } else { t(&format!("ui.auftrag.gruppe.{}", GRUPPEN[fm.gruppe])).to_string() },
        1 => t("ui.auftrag.satz.gebiet").to_string(),
        _ => crate::tf!("ui.auftrag.satz.ersatz", sq.get(fm.spieler % sq.len().max(1)).map(|&p| c.world.players[p].name.clone()).unwrap_or_default()),
    };
    let wo = match fm.weite { 1 => weite_wo.clone(), 2 => t(&format!("ui.kontinent.{weite_wo}")).to_string(), 3 => t("ui.auftrag.welt").to_string(), _ => c.world.leagues[lg].name.clone() };
    let satz = crate::tf!("ui.auftrag.satz", scout_name.clone(), was, fm.alter.0, fm.alter.1, wo, if fm.lang { 28 } else { 14 });
    let n = ui.text_passend(&satz, l.x + 14.0, y + 14.0, 14.0, l.w - 28.0, 3, Theme::akzent());
    y += 10.0 + 19.0 * n as f32;
    if ui.button_gold(Rect::new(l.x + 14.0, y, l.w - 28.0, 44.0), t("ui.auftrag.starten")) {
        let art = [crate::auftraege::SUCHE, crate::auftraege::GEBIET, crate::auftraege::NACHFOLGER][fm.art];
        let x = crate::auftraege::Auftrag { art, scout: scout_name.clone(), gruppe: if fm.art == 0 { GRUPPEN[fm.gruppe].to_string() } else { String::new() }, alter_min: fm.alter.0, alter_max: fm.alter.1,
            liga: Some(lg), spieler: if fm.art == 2 { sq.get(fm.spieler % sq.len().max(1)).copied() } else { None }, knapp: fm.knapp,
            profil: if fm.art == 0 && fm.profil > 0 { crate::regeln::wert_oder("scouting", "profil_liste").and_then(|v| v.as_array().and_then(|a| a.get(fm.profil - 1).and_then(|x| x.as_str().map(|s| s.to_string())))).unwrap_or_default() } else { String::new() },
            vorbild: if fm.art == 0 { vorbild_finden(c, &fm.vorbild) } else { None }, weite: fm.weite as u8, weite_wo: weite_wo.clone(), ..Default::default() };
        let tage = if fm.lang || art == crate::auftraege::GEBIET { 28 } else { 14 };
        match c.auftrag_anlegen(x, tage) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
    }
    // the squad gap (Konzept 4.4) as nine chips: a click picks the group for a search
    y += 60.0;
    ui.text_semi(t("ui.auftrag.kaderluecke"), l.x + 14.0, y, 14.0, Theme::text2());
    y += 10.0;
    let cw = (l.w - 28.0 - 8.0 * 4.0) / 5.0;
    for (i, g) in GRUPPEN.iter().enumerate() {
        let rr = Rect::new(l.x + 14.0 + (i % 5) as f32 * (cw + 8.0), y + (i / 5) as f32 * 34.0, cw, 28.0);
        let farbe = [Theme::gut(), Theme::achtung(), Theme::problem()][c.kaderluecke(g) as usize];
        rrect(rr.x, rr.y, rr.w, rr.h, 6.0, alpha(farbe, 0.22));
        rrect_line(rr.x, rr.y, rr.w, rr.h, 6.0, farbe);
        ui.text_center(t(&format!("ui.auftrag.gruppe.{g}")), rr.x + rr.w / 2.0, rr.y + 19.0, 13.0, Theme::text());
        if ui.hit(rr) { fm.art = 0; fm.gruppe = i; }
    }
    auswahl_zeichnen(ui, v, &gebiet_felder);
    FORM.with(|f| *f.borrow_mut() = fm);
    // right: the assignments, newest first
    let r = Rect::new(l.x + l.w + 12.0, Y0, cbreite() - l.w - 12.0, hoehe() - Y0 - 16.0);
    ui.card_titled(r, &crate::tf!("ui.auftrag.liste", c.auftraege_laufend(), c.auftraege_max()), Theme::verein());
    if c.auftraege.is_empty() { ui.paragraph(t("ui.auftrag.leer"), r.x + 14.0, r.y + 60.0, 15.0, r.w - 28.0, Theme::text2()); return; }
    let abs = c.abs();
    let mut y = r.y + 52.0;
    let mut klick = None;
    for x in c.auftraege.iter().rev() {
        if y > r.y + r.h - 60.0 { break; }
        let titel = match x.art {
            crate::auftraege::SUCHE => format!("{} · {}", t(&format!("ui.auftrag.gruppe.{}", if x.gruppe.is_empty() { crate::id("st") } else { &x.gruppe })), x.liga.map(|l| c.world.leagues[l].name.clone()).unwrap_or_default()),
            crate::auftraege::GEBIET => x.liga.map(|l| c.world.leagues[l].name.clone()).unwrap_or_default(),
            _ => x.spieler.map(|p| c.world.players[p].name.clone()).unwrap_or_default(),
        };
        rrect(r.x + 10.0, y, r.w - 20.0, 30.0, 6.0, Theme::erhoehung());
        ui.text_semi(&format!("{} · {}", t(&format!("ui.auftrag.art.{}", crate::auftraege::ARTEN[x.art as usize])), titel), r.x + 20.0, y + 20.0, 14.0, Theme::text());
        let stand = if x.fertig { t("ui.auftrag.fertig").to_string() } else { crate::tf!("ui.auftrag.noch_tage", x.ende.saturating_sub(abs), x.scout.clone()) };
        ui.text_right(&stand, r.x + r.w - 20.0, y + 20.0, 13.0, Theme::text2());
        if !x.fertig { ui.bar(r.x + 10.0, y + 28.0, r.w - 20.0, 3.0, (abs.saturating_sub(x.start)) as f32 / (x.ende - x.start).max(1) as f32, Theme::akzent()); }
        y += 40.0;
        let mut vs = x.vorschlaege.clone();
        vs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for &(p, _) in vs.iter() {
            if y > r.y + r.h - 30.0 { break; }
            let b = c.scout_urteil(p);
            let pl = &c.world.players[p];
            let rr = Rect::new(r.x + 14.0, y - 4.0, r.w - 28.0, 26.0);
            if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
            draw_circle(r.x + 26.0, y + 9.0, 6.0, urteil_farbe(b.urteil));
            let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].short.clone() } else { String::new() };
            ui.pos_tag(r.x + 40.0, y + 9.0, pos_de(&pl.pos));
            ui.text(&ui.fit(&format!("{} · {} {}", pl.name, pl.age, club), 14.0, r.w * 0.5 - 44.0), r.x + 84.0, y + 14.0, 14.0, Theme::text());
            ui.text(t(&format!("ui.sb.urteil.{}", b.urteil)), r.x + r.w * 0.56, y + 14.0, 13.0, urteil_farbe(b.urteil));
            let (lo, hi) = c.ovr_range(p);
            ui.text_right(&format!("{lo:.0}–{hi:.0} · {}", money(c.ask_price(p))), r.x + r.w - 20.0, y + 14.0, 13.0, Theme::text2());
            if ui.hit(rr) { klick = Some(p); }
            y += 26.0;
        }
        if vs.is_empty() && !x.fertig && matches!(x.art, crate::auftraege::SUCHE | crate::auftraege::NACHFOLGER) { ui.text(&crate::tf!("ui.auftrag.im_blick", x.im_blick.len()), r.x + 20.0, y + 10.0, 13.0, Theme::gedaempft()); y += 24.0; }
        y += 8.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
}

// ================================================================ Scouting
pub(super) fn scouting(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("ui.pages.scouts_und_auftraege"), crate::inhalt::t("menu.scout_verwaltung.reiter.1"), crate::inhalt::t("as.reiter.berichte"), crate::inhalt::t("ui.auftrag.reiter"), crate::inhalt::t("ui.analyst.reiter")]);
    if v.tab == 3 { auftraege_seite(ui, v, c); return; }
    if v.tab == 4 { analyst_seite(ui, v, c); return; }
    if v.tab == 0 {
        let mut y = Y0;
        let abs = c.abs();
        let n = c.scouts.len();
        // the area of every shown scout as country + league fields (Fabian 23.11.), two fields per scout
        let zeigbar = n.min(((hoehe() - 170.0 - Y0) / 160.0) as usize + 1);
        let gebiete: Vec<Vec<(String, Vec<usize>)>> = (0..zeigbar).map(|s| ligen_gruppen(c, |l| c.gebiet_erlaubt(s, Some(l)))).collect();
        let felder: Vec<Auswahl> = (0..zeigbar).flat_map(|s| wettbewerb_felder(c, &gebiete[s], c.scouts[s].league.unwrap_or(usize::MAX), Rect::new(x0() + 20.0, Y0 + s as f32 * 160.0 + 100.0, 270.0, 36.0))).collect();
        if let Some((f, i)) = auswahl_vorher(ui, v, &felder) { let s = f / 2; let alt = c.scouts[s].league.unwrap_or(usize::MAX); c.scouts[s].league = Some(wahl_umsetzen(&gebiete[s], alt, f % 2, i)).filter(|&l| l != usize::MAX); }
        for s in 0..n {
            if s >= c.scouts.len() { break; }
            let r = Rect::new(x0() + 4.0, y, cbreite(), 150.0);
            ui.card(r);
            let sc = c.scouts[s].clone();
            ui.title(&sc.name, r.x + 16.0, r.y + 34.0, 24.0, Theme::text());
            if let Some(pe) = &sc.p {
                let fw: Vec<String> = (0..4).map(|k| format!("{} {}", crate::scouting::faehigkeiten_t()[k], if pe.f[k] >= 70.0 { crate::id("stark") } else if pe.f[k] >= 50.0 { crate::id("solide") } else { crate::id("schwach") })).collect();
                ui.text(&crate::tf!("ui.pages.fachgebiet", fw.join(", "), crate::scouting::gruppen_t()[pe.spezial as usize % 5], crate::scouting::typ_wort_t()[pe.typ as usize % 4]), r.x + 16.0, r.y + 58.0, 15.0, Theme::text2());
                let vt = if pe.vertrauen >= 60.0 { crate::inhalt::t("md.pool.tag.verlässlich") } else if pe.vertrauen >= 40.0 { crate::inhalt::t("zentrale.k.keins") } else { crate::inhalt::t("ui.pages.lag_oft_daneben") }; // roh: key
                ui.text_right(&crate::tf!("ui.pages.im_jahr_urteil", money(pe.gehalt), if pe.auftrag_bis > abs { crate::tf!("ui.pages.auftrag_noch_tage", pe.auftrag_bis - abs) } else { crate::inhalt::t("ui.pages.kein_auftrag").into() }, vt), r.x + r.w - 16.0, r.y + 34.0, 15.0, Theme::text2());
            } else {
                ui.text(&crate::tf!("ui.pages.spieler_gesichtet", sc.seen), r.x + 16.0, r.y + 58.0, 16.0, Theme::text2());
            }
            ui.text_semi(crate::inhalt::t("ui.wort.gross_gebiet"), r.x + 16.0, r.y + 92.0, 14.0, Theme::text2());
            if c.gebiet_erlaubt(s, None) && ui.button(Rect::new(r.x + 150.0, r.y + 76.0, 120.0, 22.0), crate::inhalt::t("ui.wort.weltweit"), sc.league.is_none()) { c.scouts[s].league = None; }
            ui.text_semi(crate::inhalt::t("ui.wort.gross_position"), r.x + 300.0, r.y + 92.0, 14.0, Theme::text2());
            if let Some(p) = ui.segment(Rect::new(r.x + 300.0, r.y + 102.0, 250.0, 32.0), &[crate::inhalt::t("suche.gruppe.alle"), crate::inhalt::t("ui.wort.gross_tw"), crate::inhalt::t("ui.wort.gross_abw"), crate::inhalt::t("ui.wort.gross_mit"), crate::inhalt::t("ui.wort.gross_ang")], sc.pos as usize) { c.scouts[s].pos = p as u8; }
            ui.text_semi(crate::inhalt::t("ui.wort.gross_alter"), r.x + 562.0, r.y + 92.0, 14.0, Theme::text2());
            let am = sc.p.as_ref().map(|p| crate::scouting::ALTER_MAX.iter().position(|&a| a == p.alter_max).unwrap_or(0)).unwrap_or(0);
            if let Some(k) = ui.segment(Rect::new(r.x + 562.0, r.y + 102.0, 220.0, 32.0), &[crate::inhalt::t("ui.wort.alle"), crate::inhalt::t("ui.pages.bis_21"), crate::inhalt::t("ui.pages.bis_25"), crate::inhalt::t("ui.pages.bis_29")], am) { if let Some(p) = c.scouts[s].p.as_mut() { p.alter_max = crate::scouting::ALTER_MAX[k]; } }
            ui.text_semi(crate::inhalt::t("ui.wort.gross_auftrag"), r.x + 796.0, r.y + 92.0, 14.0, Theme::text2());
            if let Some(k) = ui.segment(Rect::new(r.x + 796.0, r.y + 102.0, 250.0, 32.0), &[crate::inhalt::t("ui.pages.1_wo"), crate::inhalt::t("ui.pages.2_wo"), crate::inhalt::t("ui.pages.3_wo"), crate::inhalt::t("ui.pages.4_wo")], 9) { let am = sc.p.as_ref().map(|p| p.alter_max).unwrap_or(99); match c.auftrag(s, crate::scouting::DAUER[k], am) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.auftrag_erteilt"), true), Err(e) => v.toast(&e, false) } }
            let w = sc.watch.map(|p| crate::tf!("ui.pages.beobachtet", c.world.players[p].name, format!("{:.0}", c.world.players[p].know * 100.0))).unwrap_or(crate::inhalt::t("ui.pages.beobachtet_niemanden_im_transfermarkt_wa").into());
            ui.text(&ui.fit(&w, 15.0, 300.0), r.x + r.w - 330.0, r.y + 58.0, 15.0, Theme::text2());
            if ui.button(Rect::new(r.x + r.w - 140.0, r.y + 102.0, 124.0, 32.0), crate::inhalt::t("aka.entlassen.ja"), false) { c.scout_entlassen(s); v.toast(crate::inhalt::t("ui.pages.scout_entlassen"), true); break; }
            // Konzept Scouting 3.1/5.2: one scout can work as data analyst (from club level 3), he travels no more
            if c.analyst_erlaubt() {
                let ist = c.scouts[s].p.as_ref().map_or(false, |p| p.aufgabe == 2);
                let frei = c.analyst().map_or(true, |a| a == s);
                if frei && ui.button(Rect::new(r.x + r.w - 330.0, r.y + 102.0, 180.0, 32.0), crate::inhalt::t(if ist { "ui.analyst.ist" } else { "ui.analyst.machen" }), ist) {
                    c.person(s).aufgabe = if ist { 0 } else { 2 };
                    v.toast(crate::inhalt::t(if ist { "ui.analyst.zurueck" } else { "ui.analyst.ok" }), true);
                }
            }
            y += 160.0;
            if y > hoehe() - 170.0 { break; }
        }
        auswahl_zeichnen(ui, v, &felder);
        if y <= hoehe() - 120.0 {
            let r = Rect::new(x0() + 4.0, y, cbreite(), 110.0);
            ui.card_titled(r, crate::inhalt::t("ui.pages.so_arbeiten_scouts"), Theme::akzent());
            ui.paragraph(&crate::tf!("ui.pages.ein_scout_lernt_je_tag", c.scouts_max()), r.x + 14.0, r.y + 56.0, 16.0, r.w - 28.0, Theme::text2());
        }
    } else if v.tab == 1 {
        let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
        ui.card(area);
        ui.text(&crate::tf!("ui.pages.jeden_monat_stellen_sich_fuenf", c.scouts.len(), c.scouts_max()), area.x + 16.0, area.y + 30.0, 17.0, Theme::text2());
        for (k, (name, pe)) in c.bewerber().into_iter().enumerate() {
            let r = Rect::new(area.x + 8.0, area.y + 50.0 + k as f32 * 76.0, area.w - 16.0, 68.0);
            rrect(r.x, r.y, r.w, r.h, 6.0, Theme::erhoehung());
            ui.text_semi(&name, r.x + 14.0, r.y + 26.0, 19.0, Theme::text());
            let fw: Vec<String> = (0..4).map(|i| format!("{} {:.0}", crate::scouting::faehigkeiten_t()[i], pe.f[i])).collect();
            ui.text(&crate::tf!("ui.pages.fachgebiet", fw.join(" · "), crate::scouting::gruppen_t()[pe.spezial as usize % 5], crate::scouting::typ_wort_t()[pe.typ as usize % 4]), r.x + 14.0, r.y + 50.0, 15.0, Theme::text2());
            ui.text_right(&crate::tf!("uebersicht.gehalt_jahr", money(pe.gehalt)), r.x + r.w - 160.0, r.y + 38.0, 17.0, Theme::text());
            if ui.button(Rect::new(r.x + r.w - 140.0, r.y + 18.0, 124.0, 32.0), crate::inhalt::t("menu.staff.reiter.1"), false) { match c.scout_einstellen(k) { Ok(()) => v.toast(crate::inhalt::t("ui.pages.scout_eingestellt"), true), Err(e) => v.toast(&e, false) } }
        }
    } else {
        let reps: Vec<usize> = (0..c.inbox.len()).filter(|&i| c.inbox[i].cat == 6).collect();
        let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
        ui.card(area);
        if reps.is_empty() { ui.text(crate::inhalt::t("ui.pages.noch_keine_berichte_die_scouts"), area.x + 20.0, area.y + 40.0, 18.0, Theme::text2()); }
        let vis = ((area.h - 10.0) / 70.0) as usize;
        let start = scroll_rows(ui, v, area, reps.len(), vis);
        for (k, &i) in reps.iter().skip(start).take(vis).enumerate() {
            let m = c.inbox[i].clone();
            let r = Rect::new(area.x + 8.0, area.y + 8.0 + k as f32 * 70.0, area.w - 16.0, 64.0);
            rrect(r.x, r.y, r.w, r.h, 6.0, if ui.hover(r) { Theme::ueberlagerung() } else { Theme::erhoehung() });
            ui.text_semi(&m.title, r.x + 14.0, r.y + 24.0, 18.0, if m.title.starts_with("Geheimtipp") { Theme::gold() } else { Theme::text() }); // roh
            let t = ui.fit(&m.body, 15.0, r.w - 28.0);
            ui.text(&t, r.x + 14.0, r.y + 48.0, 15.0, Theme::text2());
            if ui.hit(r) { if let Action::Report { pid } = m.action { v.modal = Modal::Player(pid); c.inbox[i].read = true; } }
        }
    }
}
