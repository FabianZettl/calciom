use super::*;

// ---------------------------------------------------------------- Spielerprofil (Python: SpielerprofilSeite.qml, Spieleruebersicht.qml)
/// A text of assets/texte/spielerprofil.json (copied from core/texts/spielerprofil.json, own keys added).
pub(super) fn tx(k: &str) -> &'static str { profil_texte()[k].as_str().unwrap_or("") }
pub(super) fn tx_liste(k: &str) -> Vec<&'static str> { profil_texte()[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default() }
fn profil_texte() -> &'static serde_json::Value { crate::inhalt::eigen("spielerprofil") }
fn pw(k: &str) -> &'static serde_json::Value { crate::regeln::wert("profilseite", k) }
use crate::regeln::f;
fn indizes(v: &serde_json::Value) -> Vec<usize> { v.as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|x| x as usize)).collect()).unwrap_or_default() }
/// Groups of the 25 abilities: (id, indices, colour).
pub(super) fn gruppen() -> Vec<(&'static str, Vec<usize>, Color)> {
    pw("gruppen").as_array().unwrap().iter().map(|g| (g["id"].as_str().unwrap(), indizes(&g["werte"]), hex(u32::from_str_radix(g["farbe"].as_str().unwrap(), 16).unwrap()))).collect()
}
/// Stars of a value as on the template (40 → ½, every 11 points one star, at most 5).
pub(super) fn profil_sterne(w: f32) -> f32 { (((w - f("profilseite", "sterne.von")) / f("profilseite", "sterne.je_stern") * 2.0).round() / 2.0).clamp(0.5, 5.0) }
/// Which of the 25 values the person knows: all for own players and from knowledge level 4, the best share of them before
/// (scouting.toml sterne.einzelwerte_anteil, as the short view), none below level 2.
pub fn bekannte_werte(c: &Career, p: usize) -> Vec<bool> {
    let pl = &c.world.players[p];
    if pl.attrs.len() != 25 { return vec![false; 25]; }
    let st = if c.gehoert_uns(p) { 5 } else { c.sicht(p) };
    if st >= 4 { return vec![true; 25]; }
    if st < 2 { return vec![false; 25]; }
    let n = (25.0 * crate::mensch::arr("scouting", "sterne.einzelwerte_anteil")[st].as_f64().unwrap() as f32).round() as usize;
    let mut idx: Vec<usize> = (0..25).collect();
    idx.sort_by(|a, b| pl.attrs[*b].partial_cmp(&pl.attrs[*a]).unwrap());
    let mut v = vec![false; 25];
    for &k in idx.iter().take(n) { v[k] = true; }
    v
}
fn mittel(pl: &Player, ix: &[usize]) -> Option<f32> { if pl.attrs.len() != 25 || ix.is_empty() { None } else { Some(ix.iter().map(|&i| pl.attrs[i]).sum::<f32>() / ix.len() as f32) } }
/// Injury resistance is not decoded in Player.bin: estimated from age, physical contact and stamina (profilseite.json robust).
pub fn robustheit(pl: &Player) -> Option<f32> {
    if pl.attrs.len() != 25 { return None; }
    let r = |k: &str| f("profilseite", &format!("robust.{k}"));
    Some((r(crate::id("basis")) + r(crate::id("koerper")) * (pl.attrs[14] - r(crate::id("basis"))) + r(crate::id("ausdauer")) * (pl.attrs[16] - r(crate::id("basis"))) - r("je_jahr") * (pl.age as f32 - r("alter_ab")).max(0.0)).clamp(1.0, 99.0))
}
/// Same position, similar strength and age (profilseite.json aehnliche), closest first.
pub fn aehnliche(c: &Career, p: usize) -> Vec<usize> {
    let pl = &c.world.players[p];
    let (ds, da, n) = (f("profilseite", "aehnliche.staerke"), f("profilseite", "aehnliche.alter") as i32, f("profilseite", "aehnliche.anzahl") as usize);
    let mut v: Vec<(f32, usize)> = c.world.players.iter().enumerate().filter(|(i, q)| *i != p && q.team != -2 && q.youth == 0 && q.pos == pl.pos && (q.ovr - pl.ovr).abs() <= ds && (q.age as i32 - pl.age as i32).abs() <= da)
        .map(|(i, q)| ((q.ovr - pl.ovr).abs() + (q.age as f32 - pl.age as f32).abs(), i)).collect();
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.cmp(&b.1)));
    v.into_iter().take(n).map(|x| x.1).collect()
}
/// Squad status: the role for own players with a character, else the rank by strength in his club.
fn kaderstatus(c: &Career, p: usize) -> String {
    let pl = &c.world.players[p];
    if pl.team == c.own as i32 && pl.ps.an { return crate::mensch::rolle_name(pl.ps.rolle).into(); }
    if pl.team >= 0 && c.zweite_mannschaft() == Some(pl.team as usize) { return tx("spp.kader.zweite").into(); }
    if pl.team != c.own as i32 && c.gehoert_uns(p) { return if pl.team >= 0 && c.world.koop_index(c.own, pl.team as usize).is_some() { tx("spp.kader.partner").into() } else { tx("spp.kader.verliehen").into() }; }
    if pl.team < 0 { return tx("spp.unbekannt").into(); }
    let mut sq = c.world.clubs[pl.team as usize].squad.clone();
    sq.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
    let r = sq.iter().position(|&x| x == p).unwrap_or(99);
    tx(if r < f("profilseite", "kader.stamm") as usize { "spp.kader.stamm" } else if r < f("profilseite", "kader.rotation") as usize { "spp.kader.rotation" } else { "spp.kader.ergaenzung" }).into()
}
/// News ticker: the latest headlines, one at a time.
fn ticker(ui: &mut Ui, c: &Career, r: Rect) {
    rrect(r.x, r.y, r.w, r.h, 6.0, Theme::erhoehung());
    rrect(r.x, r.y, 86.0, r.h, 6.0, Theme::problem());
    ui.text_semi(crate::inhalt::t("ui.wort.gross_ticker"), r.x + 14.0, r.y + r.h / 2.0 + 6.0, 15.0, hex(0xffffff));
    let n = c.feed.len().min(6);
    if n == 0 { return; }
    let k = (ui.time / 5.0) as usize % n;
    let z = format!("+++ {} · {}", c.feed[k].head, c.feed[k].paper);
    let z = ui.fit(&z, 15.0, r.w - 110.0);
    ui.text(&z, r.x + 98.0, r.y + r.h / 2.0 + 6.0, 15.0, Theme::text());
}

pub(super) fn spielerprofil(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let sq = c.squad();
    let p = v.sel_player.filter(|&p| p < c.world.players.len()).unwrap_or(sq[0]);
    let pl = c.world.players[p].clone();
    let own = c.gehoert_uns(p);
    let im_kader = pl.team == c.own as i32;
    let ohne_werte = pl.attrs.len() != 25;
    let sicht = if own { 5 } else { c.sicht(p) };
    let kennt = bekannte_werte(c, p);
    let alle = kennt.iter().all(|x| *x) && pl.attrs.len() == 25;
    let unbek = tx("spp.unbekannt");
    let reiter = tx_liste("spp.reiter");
    // why values are missing: the world has none (demo world), or the scout does not know them yet
    let luecke = if ohne_werte { tx("spp.ohne_werte").to_string() } else { format!("{} {}", tx("spp.scout_fehlt"), tx("spp.sicht_ab")) };
    // an open league ranking on the statistics tab is modal: the page under it gets no mouse
    let (maus, klick) = (ui.mouse, ui.clicked);
    let stat_modal = v.tab == 5 && stat_fenster_offen(p);
    if stat_modal { ui.mouse = vec2(-1.0e4, -1.0e4); ui.clicked = false; }
    tabs(ui, v, &reiter);
    if own { ui.text_right(tx("spp.stand_aktuell"), x0() + cbreite(), Y0 - 20.0, 15.0, Theme::text2()); }
    else { ui.text_right(&tx("spp.stand").replace("%1", &date::long(c.world.season, c.world.day)).replace("%2", &format!("{:.0}", c.know_of(p) * 100.0)), x0() + cbreite(), Y0 - 20.0, 15.0, Theme::text2()); }
    ticker(ui, c, Rect::new(x0() + 4.0, Y0, cbreite(), 26.0));
    if ui.button(Rect::new(x0() + cbreite() - 480.0, Y0 - 40.0, 170.0, 28.0), crate::inhalt::t("ui.spp.mehr_aktionen"), false) { v.modal = Modal::Aktionen(p); }

    // ---- head: number, photo, names, body, position, club, contract, buttons
    let hr = Rect::new(x0() + 4.0, Y0 + 34.0, cbreite(), 112.0);
    let club = if pl.team >= 0 { pl.team as usize } else { c.own };
    // the club's colour as on the club page (Fabian 23.11.): the crest's main colour, else the kit colour
    let c1 = { let k = &c.world.clubs[club]; ui.wappen_farbe(k.id).map(hex).unwrap_or(hex(k.c1)) };
    let fg = on(c1);
    rrect(hr.x, hr.y, hr.w, hr.h, 10.0, c1);
    ui.rahmen(hr);
    ui.rahmen(Rect::new(hr.x + 226.0, hr.y, 406.0, hr.h));
    ui.title(&if pl.no > 0 { pl.no.to_string() } else { "–".into() }, hr.x + 18.0, hr.y + 86.0, 76.0, alpha(fg, 0.9));
    // the shirt number changes right here, only while a window is open (Fabian 24.11.); taken numbers are skipped
    if pl.team == c.own as i32 && c.in_window().is_some() {
        for (i, (t, d)) in [("−", -1), ("+", 1)].into_iter().enumerate() {
            if ui.button(Rect::new(hr.x + 18.0 + i as f32 * 36.0, hr.y + 88.0, 30.0, 20.0), t, false) { if let Err(e) = c.rueckennummer(p, d) { v.toast(&e, false); } }
        }
    }
    let fr = Rect::new(hr.x + 118.0, hr.y + 6.0, 96.0, 100.0);
    rrect(fr.x, fr.y, fr.w, fr.h, 8.0, alpha(Theme::grund(), 0.35));
    if !ui.spieler_foto(fr.x, fr.y, fr.w, fr.h, pl.id) {
        // no photo: the little footballer in his club's kit with his number (keeper in green); build from his body data
        let (k1, k2) = club_color(c, club);
        let tw = pl.pos == crate::id("GK");
        let gross = if pl.profil.koerper[0] > 0 { ((pl.profil.koerper[1] as f32 / pl.profil.koerper[0].max(1) as f32 * 100.0 - 38.0) / 10.0).clamp(0.0, 1.0) } else { 0.5 };
        ui.kicker(fr.x + 48.0, fr.y + fr.h - 6.0, 88.0, pl.id, if tw { hex(0x16a34a) } else { k1 }, if tw { hex(0x14532d) } else { k2 }, pl.no, gross);
    }
    let teile: Vec<&str> = pl.name.split_whitespace().collect();
    let (vor, nach) = if teile.len() > 1 { (teile[..teile.len() - 1].join(" "), teile[teile.len() - 1].to_string()) } else { (String::new(), pl.name.clone()) };
    ui.text_semi(&vor, hr.x + 232.0, hr.y + 30.0, 18.0, alpha(fg, 0.85));
    let nach = ui.fit(&nach, 38.0, 380.0);
    ui.title(&nach, hr.x + 232.0, hr.y + 70.0, 38.0, fg);
    let k = pl.profil.koerper;
    let koerper = if k[0] > 0 { crate::tf!("ui.seiten.cm_kg", k[0], k[1], tx(if k[2] == 1 { "spp.fuss.links" } else { "spp.fuss.rechts" })) } else { crate::tf!("ui.seiten.groesse_gewicht_fuss", unbek) };
    // nationality, caps and standing with the national coach (nationalteams.rs)
    let land = crate::nationen::name_anzeige(&c.world, pl.nat);
    let nat = if land.is_empty() { String::new() } else if let Some(st) = c.world.nat_status(p) { format!(" · {}", crate::tf!(&format!("ui.nt.status.{st}"), land, pl.profil.laenderspiele, pl.profil.nat_tore)) } else { format!(" · {land}") };
    ui.text(&format!("{}{nat}", crate::tf!("as.ent.talent", pl.age, koerper)), hr.x + 232.0, hr.y + 98.0, 16.0, alpha(fg, 0.85));
    let rx = hr.x + 640.0;
    // the position in its line's colour (keeper, defence, midfield, attack), as everywhere
    rrect(rx, hr.y + 10.0, 54.0, 24.0, 6.0, Theme::teil(&pl.pos));
    ui.text_center(pos_de(&pl.pos), rx + 27.0, hr.y + 28.0, 15.0, hex(0xffffff));
    ui.text_semi(&crate::tf!("liga.nummer", pl.no), rx + 66.0, hr.y + 28.0, 16.0, fg);
    let clubname = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else if pl.team == -1 { crate::inhalt::t("ui.wort.vereinslos").into() } else { crate::inhalt::t("ui.pages.karriereende").into() };
    if pl.team >= 0 { let cl = &c.world.clubs[pl.team as usize]; ui.club_crest(rx + 150.0, hr.y + 22.0, 13.0, cl.id, hex(cl.c1), &cl.short); }
    let cn = ui.fit(&clubname, 16.0, 200.0);
    ui.text_semi(&cn, rx + 170.0, hr.y + 28.0, 16.0, fg);
    ui.text_right(&tx("spp.vertrag_bis").replace("%1", &(date::START_YEAR as u32 + pl.contract_end + 1).to_string()), hr.x + hr.w - 16.0, hr.y + 28.0, 16.0, fg);
    // buttons (own: club, watch, talk, transfer list; foreign: club, watch, scout, offer)
    // a foreign player has four buttons a row (sounding out: ask, talk, interest)
    let spalten = if pl.team >= 0 && pl.team != c.own as i32 { 4 } else { 3 };
    let bw = (hr.w - 656.0 - 8.0 * (spalten as f32 - 1.0) - 8.0) / spalten as f32;
    let bt = |i: usize| Rect::new(rx + (i % spalten) as f32 * (bw + 8.0), hr.y + 42.0 + (i / spalten) as f32 * 34.0, bw, 30.0);
    if pl.team >= 0 && ui.button(bt(0), tx("spp.k.zum_verein"), false) { v.sel_club = Some(pl.team as usize); v.go(Page::Vereine); return; }
    let gemerkt = c.merkliste.contains(&p);
    if ui.button(bt(1), tx(if gemerkt { "spp.k.beobachtet" } else { "spp.k.beobachten" }), gemerkt) { if gemerkt { c.merkliste.retain(|&x| x != p); } else { c.merkliste.push(p); v.toast(crate::inhalt::t("ui.seiten.auf_die_merkliste_gesetzt"), true); } }
    if own {
        if ui.button(bt(2), tx("spp.k.gespraech"), false) {
            let thema = if pl.last.last().map_or(false, |&r| r >= 7.5) { crate::id("lob") } else if pl.last.last().map_or(false, |&r| r < 5.5) { crate::id("kritik") } else if pl.benched >= 3 { crate::id("spielzeit") } else if c.monate_rest(p) <= 8.0 { crate::id("zukunft") } else if pl.form < 0.0 { crate::id("form") } else { crate::id("rolle") };
            let n = c.inbox.len();
            c.anliegen(p, thema, true);
            if c.inbox.len() > n { v.sel_msg = Some(0); v.go(Page::Posteingang); return; } else { v.toast(crate::inhalt::t("ui.seiten.ihr_habt_ueber_dieses_thema"), false); }
        }
        if !im_kader {
            if let Some(le) = &pl.leihe { ui.text(&crate::tf!("ui.seiten.verliehen_bis", le.rueckkehr()), bt(3).x + 4.0, bt(3).y + 21.0, 14.0, fg); }
        } else if pl.listed {
            if ui.button(bt(3), tx("spp.k.von_liste"), true) { c.world.players[p].listed = false; c.world.players[p].profil.mindestpreis = 0.0; }
            if pl.profil.mindestpreis > 0.0 { ui.text(&crate::tf!("tf.liste.modus.preis", money(pl.profil.mindestpreis)), bt(4).x + 4.0, bt(4).y + 21.0, 14.0, fg); }
        } else {
            if let Some(g) = c.verkauf_gesperrt(p) { ui.paragraph(&g, bt(3).x + 4.0, bt(3).y + 8.0, 13.0, bt(3).w * 2.0 + 8.0, fg); }
            else if ui.button(bt(3), tx("spp.k.liste_angebote"), false) { c.world.players[p].listed = true; c.world.players[p].profil.mindestpreis = 0.0; v.toast(crate::inhalt::t("ui.seiten.auf_der_transferliste_du_pruefst"), true); }
            if c.verkauf_gesperrt(p).is_none() && ui.button(bt(4), tx("spp.k.liste_mindest"), false) { c.world.players[p].listed = true; c.world.players[p].profil.mindestpreis = round_money(pl.value); v.toast(&crate::tf!("ui.seiten.auf_der_transferliste_angebote_unter", money(round_money(pl.value))), true); }
        }
    } else if pl.team != -2 {
        // Konzept Scouting 3.2: "Scouten" starts a watch assignment with a free scout; once a report exists the button
        // asks another scout for a second opinion
        let laeuft = c.auftraege.iter().any(|x| !x.fertig && x.spieler == Some(p));
        let zweite = c.know_of(p) >= crate::regeln::f("scouting", "wissen.bericht_ab") / 100.0;
        let label = if laeuft { crate::inhalt::t("ui.auftrag.laeuft") } else if zweite { crate::inhalt::t("ui.auftrag.art.zweite") } else { tx("spp.k.scouten") };
        if ui.button(bt(2), label, laeuft) && !laeuft {
            let belegt: Vec<String> = c.auftraege.iter().filter(|x| !x.fertig).map(|x| x.scout.clone()).collect();
            match c.scouts.iter().find(|s| !belegt.contains(&s.name)).map(|s| s.name.clone()) {
                Some(scout) => {
                    let art = if zweite { crate::auftraege::ZWEITE } else { crate::auftraege::BEOBACHTEN };
                    let x = crate::auftraege::Auftrag { art, scout, spieler: Some(p), ..Default::default() };
                    match c.auftrag_anlegen(x, 21) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
                }
                None => v.toast(crate::inhalt::t("ui.pages.alle_scouts_haben_schon_einen"), false),
            }
        }
        if ui.button(bt(3), tx("spp.k.angebot"), false) { match c.open_neg(p) { Ok(i) => { v.modal = Modal::Neg(i); return; } Err(e) => v.toast(&e, false) } }
        // sounding out before the offer (sondierung.rs); what is known so far under the buttons
        if pl.team >= 0 {
            let mut r = None;
            for (k, key) in ["sd.anfragen", "sd.sprechen", "sd.interesse"].iter().enumerate() { // roh
                if ui.button(bt(4 + k), crate::inhalt::t(key), false) { r = Some(match k { 0 => c.sondierung_verein(p), 1 => c.sondierung_spieler(p), _ => c.sondierung_interesse(p) }); }
            }
            if let Some(r) = r { match r { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) } }
            if let Some(st) = c.sond_status(p) { ui.text_passend(&st, bt(7).x, bt(7).y + 20.0, 13.0, bt(7).w, 1, fg); }
        }
    }

    // ---- boxes: strength, playing style, squad status, market value, wage
    let (lo, hi) = c.ovr_range(p);
    let stil = if pl.attrs.len() == 25 && (own || sicht >= 3) { tx_liste("spp.stil").get(pl.style as usize).copied().unwrap_or(unbek).to_string() } else { unbek.to_string() };
    let spanne = |x: f64| if own || sicht >= 5 { money(x) } else { format!("{} – {}", money(x * 0.9), money(x * 1.15)) };
    // own players: the form bonus next to the strength (works in the match, not for good)
    let z = if own { crate::world::form_zuschlag(&pl) } else { 0.0 };
    let staerke = if hi - lo <= 1.0 { format!("{:.0}", pl.ovr) } else { format!("{lo:.0}–{hi:.0}") };
    let kaesten = [(tx("spp.staerke"), if z != 0.0 { crate::tf!("ui.wellen.mit_zuschlag", staerke, format!("{z:+.0}").replace('-', "−")) } else { staerke }), (tx("spp.spielstil"), stil), (tx("spp.kaderstatus"), kaderstatus(c, p)),
        (tx("spp.marktwert"), spanne(pl.value)), (tx("spp.gehalt"), tx("spp.pro_jahr").replace("%1", &spanne(pl.wage)))];
    let kw = (cbreite() - 4.0 * 10.0) / 5.0;
    for (i, (t, w)) in kaesten.iter().enumerate() {
        let r = Rect::new(x0() + 4.0 + i as f32 * (kw + 10.0), Y0 + 154.0, kw, 54.0);
        ui.card(r);
        ui.text_semi(&t.to_uppercase(), r.x + 12.0, r.y + 19.0, 12.0, Theme::gedaempft());
        let w = ui.fit(w, 20.0, r.w - 24.0);
        ui.title(&w, r.x + 12.0, r.y + 45.0, 20.0, if i == 0 { Theme::skala((lo + hi) / 2.0) } else { Theme::text() });
    }

    let cy = Y0 + 216.0;
    let ch = hoehe() - cy - 16.0;
    match v.tab {
        0 => {
            // positions: small pitch and suitability per position
            let a = Rect::new(x0() + 4.0, cy, 280.0, ch);
            ui.card_titled(a, tx("spp.positionen"), Theme::akzent());
            let pf = Rect::new(a.x + 14.0, a.y + 44.0, a.w - 28.0, 168.0);
            rrect(pf.x, pf.y, pf.w, pf.h, 6.0, alpha(Theme::gut(), 0.16));
            draw_rectangle_lines(pf.x + 4.0, pf.y + 4.0, pf.w - 8.0, pf.h - 8.0, 1.0, alpha(Theme::text(), 0.25));
            draw_line(pf.x + 4.0, pf.y + pf.h / 2.0, pf.x + pf.w - 4.0, pf.y + pf.h / 2.0, 1.0, alpha(Theme::text(), 0.25));
            let haupt = crate::pesdb::POS.iter().position(|x| *x == pl.pos).unwrap_or(12);
            let stufe = |k: usize| if k == haupt { 3 } else if sicht >= 2 { pl.pp.get(k).copied().unwrap_or(0) } else { 0 };
            let lage = pw("feldlage").as_array().unwrap();
            for k in 0..13 {
                let (fx, fy) = (lage[k][0].as_f64().unwrap() as f32, lage[k][1].as_f64().unwrap() as f32);
                let (x, y) = (pf.x + fx * pf.w, pf.y + fy * pf.h);
                let (col, r) = match stufe(k) { 3 => (Theme::akzent(), 11.0), 2 => (Theme::gut(), 9.0), 1 => (Theme::achtung(), 8.0), _ => (alpha(Theme::text(), 0.15), 7.0) };
                draw_circle(x, y, r, col);
                if stufe(k) > 0 { ui.text_center(&pos_de(crate::pesdb::POS[k]).chars().take(3).collect::<String>(), x, y + 4.0, 10.0, hex(0xffffff)); }
            }
            for (i, (t, col)) in [(tx("spp.legende.haupt"), Theme::akzent()), (tx("spp.legende.voll"), Theme::gut()), (tx("spp.legende.halb"), Theme::achtung())].iter().enumerate() {
                let x = a.x + 16.0 + i as f32 * 86.0;
                draw_circle(x, a.y + 228.0, 5.0, *col);
                ui.text(t, x + 9.0, a.y + 233.0, 13.0, Theme::text2());
            }
            ui.text_semi(tx("spp.eignung"), a.x + 14.0, a.y + 262.0, 14.0, Theme::text2());
            let mut ps: Vec<usize> = (0..13).filter(|&k| stufe(k) > 0).collect();
            let eig = |k: usize| if pl.attrs.len() == 25 { crate::pesdb::ovr(&pl.attrs, k as u8) } else { pl.ovr };
            ps.sort_by(|x, y| eig(*y).partial_cmp(&eig(*x)).unwrap());
            for (j, &k) in ps.iter().take(6).enumerate() {
                let y = a.y + 286.0 + j as f32 * 22.0;
                ui.pos_tag(a.x + 14.0, y - 5.0, pos_de(crate::pesdb::POS[k]));
                if alle || own {
                    ui.sterne(a.x + 120.0, y - 5.0, profil_sterne(eig(k)), 6.0);
                    ui.text_right(&format!("{:.0}", eig(k)), a.x + a.w - 14.0, y, 15.0, Theme::skala(eig(k)));
                } else { ui.text_right(unbek, a.x + a.w - 14.0, y, 14.0, Theme::gedaempft()); }
            }
            // abilities by group, coloured
            let b = Rect::new(a.x + a.w + 12.0, cy, 500.0, ch);
            ui.card_titled(b, tx("spp.attribute"), Theme::gold());
            if pl.attrs.len() != 25 || sicht < 2 { ui.paragraph(&luecke, b.x + 14.0, b.y + 60.0, 16.0, b.w - 28.0, Theme::text2()); }
            else {
                let gr = gruppen();
                let mut spalte = [b.y + 52.0, b.y + 52.0];
                for (gi, (id, ix, col)) in gr.iter().enumerate() {
                    let s = if gi < 3 { 0 } else { 1 };
                    let x = b.x + 14.0 + s as f32 * (b.w / 2.0);
                    let cw2 = b.w / 2.0 - 28.0;
                    draw_rectangle(x, spalte[s] - 11.0, 4.0, 14.0, *col);
                    ui.text_semi(tx(&format!("spp.gruppe.{id}")), x + 10.0, spalte[s], 14.0, *col);
                    spalte[s] += 22.0;
                    for &k in ix {
                        let y = spalte[s];
                        ui.text(crate::pesdb::attr_de_t()[k], x, y, 14.0, Theme::text2());
                        if kennt[k] {
                            ui.bar(x + 128.0, y - 8.0, cw2 - 160.0, 7.0, pl.attrs[k] / 99.0, *col);
                            ui.text_right(&format!("{:.0}", pl.attrs[k]), x + cw2, y, 14.0, Theme::skala(pl.attrs[k]));
                        } else { ui.text_right("?", x + cw2, y, 14.0, Theme::gedaempft()); }
                        spalte[s] += 21.0;
                    }
                    spalte[s] += 6.0;
                }
            }
            // radar of the 7 groups, player DNA, strengths and weaknesses
            let d = Rect::new(b.x + b.w + 12.0, cy, cbreite() - a.w - b.w - 24.0, ch);
            ui.card_titled(d, &format!("{} · {}", tx("spp.radar"), tx("spp.dna")), Theme::akzent());
            // from the radar to the comparison (Fabian 23.11.): this player in the first column
            if ui.button(Rect::new(d.x + d.w - 130.0, d.y + 6.0, 120.0, 26.0), crate::inhalt::t("ui.vgl.vergleichen"), false) { v.vergleich[0] = Some(p); v.go(Page::Vergleich); }
            if !alle { ui.paragraph(&luecke, d.x + 14.0, d.y + 60.0, 16.0, d.w - 28.0, Theme::text2()); }
            else {
                let gr = gruppen();
                let (mx, my, rad) = (d.x + 96.0, d.y + 120.0, 58.0);
                let n = gr.len();
                let pt = |i: usize, t: f32| { let w = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / n as f32; vec2(mx + w.cos() * rad * t, my + w.sin() * rad * t) };
                for ring in [0.33, 0.66, 1.0] { for i in 0..n { let (p1, p2) = (pt(i, ring), pt((i + 1) % n, ring)); draw_line(p1.x, p1.y, p2.x, p2.y, 1.0, alpha(Theme::text(), 0.15)); } }
                let wert: Vec<f32> = gr.iter().map(|g| ((mittel(&pl, &g.1).unwrap_or(40.0) - 40.0) / 59.0).clamp(0.05, 1.0)).collect();
                for i in 0..n {
                    let j = (i + 1) % n;
                    let (p1, p2) = (pt(i, wert[i]), pt(j, wert[j]));
                    draw_triangle(vec2(mx, my), p1, p2, alpha(Theme::akzent(), 0.3));
                    draw_line(p1.x, p1.y, p2.x, p2.y, 2.0, Theme::akzent());
                    let e = pt(i, 1.0);
                    draw_line(mx, my, e.x, e.y, 1.0, alpha(Theme::text(), 0.12));
                    let l = pt(i, 1.28);
                    ui.text_center(tx(&format!("spp.gruppe.{}", gr[i].0)), l.x, l.y + 4.0, 10.0, gr[i].2);
                }
                // DNA: 5 bars of 10 cells (means of related values)
                let tw = pl.pos == crate::id("GK");
                let mut dna: Vec<(String, Vec<usize>)> = pw(if tw { "dna_torwart" } else { "dna_feld" }).as_array().unwrap().iter().chain(pw("dna").as_array().unwrap().iter()).map(|x| (x["id"].as_str().unwrap().to_string(), indizes(&x["werte"]))).collect();
                dna.truncate(5);
                for (j, (id, ix)) in dna.iter().enumerate() {
                    let y = d.y + 58.0 + j as f32 * 28.0;
                    let w = mittel(&pl, ix).unwrap_or(40.0);
                    ui.text(tx(&format!("spp.dna.{id}")), d.x + 186.0, y, 13.0, Theme::text2());
                    for z in 0..10 { rrect(d.x + 186.0 + z as f32 * 15.0, y + 5.0, 12.0, 8.0, 2.0, if z < (w / 10.0).round() as usize { Theme::skala(w) } else { alpha(Theme::text(), 0.1) }); }
                }
                // strengths and weaknesses: three highest and lowest of the values that count for his position
                // positionsrollen.toml: strengths from everything that counts for his role group, weaknesses only from kern and rolle
                let rg = crate::scoutbericht::gruppe(&pl.pos);
                let mut zaehlen: Vec<usize> = (0..25).filter(|&k| crate::scoutbericht::wichtig(rg, k) >= 1).collect();
                zaehlen.sort_by(|x, y| pl.attrs[*y].partial_cmp(&pl.attrs[*x]).unwrap());
                let mut schwach: Vec<usize> = (0..25).filter(|&k| crate::scoutbericht::wichtig(rg, k) >= 2).collect();
                schwach.sort_by(|x, y| pl.attrs[*x].partial_cmp(&pl.attrs[*y]).unwrap());
                let n3 = f("profilseite", "staerken_anzahl") as usize;
                ui.text_semi(tx("spp.staerken_schwaechen"), d.x + 14.0, d.y + 222.0, 14.0, Theme::text2());
                let mut y = d.y + 244.0;
                for (k, gut) in zaehlen.iter().take(n3).map(|k| (*k, true)).chain(schwach.iter().take(n3).filter(|k| !zaehlen.iter().take(n3).any(|z| z == *k)).map(|k| (*k, false))) {
                    let z = tx(if gut { "spp.stark" } else { "spp.schwach" }).replace("%1", crate::pesdb::attr_de_t()[k]).replace("%2", &format!("{:.0}", pl.attrs[k]));
                    let z = ui.fit(&z, 14.0, d.w - 40.0);
                    ui.text(if gut { "+" } else { "−" }, d.x + 14.0, y, 15.0, if gut { Theme::gut() } else { Theme::problem() });
                    ui.text(&z, d.x + 28.0, y, 14.0, Theme::text());
                    y += 21.0;
                }
            }
        }
        1 => {
            // details and character
            let (l, r) = (Rect::new(x0() + 4.0, cy, cbreite() * 0.5 - 6.0, ch), Rect::new(x0() + cbreite() * 0.5 + 10.0, cy, cbreite() * 0.5 - 6.0, ch));
            ui.card_titled(l, tx("spp.details"), Theme::akzent());
            let wissen = |ok: bool, s: String| if ok { s } else { unbek.to_string() };
            let form = |x: f32| if x >= 3.0 { crate::inhalt::t("vhd.wort.sehr_gut") } else if x >= 1.0 { crate::inhalt::t("ui.wort.gut") } else if x > -1.0 { crate::inhalt::t("ui.wort.normal") } else if x > -3.0 { crate::inhalt::t("ui.wort.schwach") } else { crate::inhalt::t("ui.seiten.sehr_schwach") };
            let stimmung = if own && pl.ps.an {
                let mut tr = pl.ps.treiber.clone();
                tr.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
                let wort = if pl.morale >= 70.0 { crate::inhalt::t("ui.wort.zufrieden") } else if pl.morale >= 50.0 { crate::inhalt::t("ui.wort.ruhig") } else { crate::inhalt::t("ui.wort.unzufrieden") };
                match tr.first().filter(|x| x.1.abs() >= 0.5) { Some(t) => tx("spp.stimmung_grund").replace("%1", wort).replace("%2", crate::mensch::treiber_name(&t.0)), None => wort.to_string() }
            } else if own { format!("{:.0}", pl.morale) } else { unbek.into() };
            let z: Vec<(String, String)> = vec![
                (tx("spp.d.name").into(), pl.name.clone()),
                (tx("spp.d.groesse").into(), if k[0] > 0 { crate::tf!("profilspiel.cm", k[0]) } else { unbek.into() }),
                (tx("spp.d.gewicht").into(), if k[0] > 0 { crate::tf!("profilspiel.kg", k[1]) } else { unbek.into() }),
                (tx("spp.d.fuss").into(), if k[0] > 0 { tx(if k[2] == 1 { "spp.fuss.links" } else { "spp.fuss.rechts" }).into() } else { unbek.into() }),
                (format!("{} ({})", tx("spp.d.schwachfuss"), tx("spp.schwachfuss_hinweis")), wissen(k[3] > 0 && (own || sicht >= 3), crate::tf!("ui.seiten.von_4_von_4", k[3], k[4]))),
                (crate::inhalt::t("ui.nation").into(), { let n = crate::nationen::name_anzeige(&c.world, pl.nat); if n.is_empty() { "–".to_string() } else { n } }),
                (tx("spp.d.form").into(), wissen(own || sicht >= 3, form(pl.form).into())),
                (tx("spp.d.stimmung").into(), stimmung),
                (tx("spp.d.fitness").into(), wissen(own || sicht >= 3, if pl.inj > 0 { tx("spp.verletzt").replace("%1", &pl.inj.to_string()) } else { format!("{:.0} %", pl.fit) })),
                (tx("spp.d.anfaellig").into(), wissen(own || sicht >= 5, robustheit(&pl).map(|r| format!("{r:.0} ({})", tx("spp.geschaetzt"))).unwrap_or(unbek.into()))),
                (tx("spp.spielstil").into(), kaesten[1].1.clone()),
                (tx("spp.marktwert").into(), spanne(pl.value)),
                (tx("spp.d.vertrag").into(), (date::START_YEAR as u32 + pl.contract_end + 1).to_string()),
            ];
            zeilen(ui, l, &z, 64.0);
            ui.text(tx("spp.robust_hinweis"), l.x + 14.0, l.y + l.h - 14.0, 13.0, Theme::gedaempft());
            ui.card_titled(r, tx("spp.charakter"), Theme::gold());
            let mut y = r.y + 64.0;
            if pl.ps.an && (own || sicht >= 3) {
                let typ = crate::mensch::typen(&pl).first().map(|t| crate::mensch::typ_name(t)).unwrap_or(crate::inhalt::t("gs.kontext.form.mittel"));
                for (t, w) in [(tx("spp.typ"), typ.to_string()), (tx("spp.rolle"), kaderstatus(c, p))] { ui.text(t, r.x + 14.0, y, 15.0, Theme::text2()); ui.text_semi(&w, r.x + 200.0, y, 16.0, Theme::text()); y += 26.0; }
                y += 6.0;
                for i in 0..10 {
                    let w = crate::mensch::cw(&pl, i);
                    ui.text(crate::mensch::wert_name(i), r.x + 14.0, y, 14.0, Theme::text2());
                    ui.bar(r.x + 200.0, y - 8.0, r.w - 270.0, 7.0, w / 100.0, Theme::akzent());
                    ui.text_right(&format!("{w:.0}"), r.x + r.w - 16.0, y, 14.0, Theme::text());
                    y += 22.0;
                }
            } else { ui.paragraph(&luecke, r.x + 14.0, y, 16.0, r.w - 28.0, Theme::text2()); y += 50.0; }
            y += 10.0;
            ui.text_semi(tx("spp.spezial"), r.x + 14.0, y, 14.0, Theme::text2());
            let sk: Vec<&str> = crate::pesdb::SKILLS.iter().enumerate().filter(|(i, _)| *i < 35 && pl.skills >> i & 1 == 1).map(|(_, s)| s.0).collect();
            let st = if !(own || sicht >= 5) { unbek.to_string() } else if sk.is_empty() { tx("spp.keine").into() } else { sk.join(", ") };
            ui.paragraph(&st, r.x + 14.0, y + 20.0, 15.0, r.w - 28.0, Theme::text());
        }
        2 => {
            // form of the last 5 games, season values; similar players
            let (l, r) = (Rect::new(x0() + 4.0, cy, cbreite() * 0.5 - 6.0, ch), Rect::new(x0() + cbreite() * 0.5 + 10.0, cy, cbreite() * 0.5 - 6.0, ch));
            let saison_noten = c.noten_verlauf.get(&pl.id).cloned().unwrap_or_default();
            // the graph takes what the card leaves after the season boxes (150 units at full height, at least 70 on 21:9)
            let gh = (l.h - 190.0).clamp(70.0, 150.0);
            let zeitraum = if saison_noten.len() >= 3 { crate::inhalt::t("ui.saison.diese_saison") } else { tx("spp.letzte5") };
            ui.card_titled(l, &format!("{} · {}", tx("spp.spielform"), zeitraum), Theme::akzent());
            let letzte: Vec<f32> = pl.last.iter().rev().take(5).rev().copied().collect();
            if letzte.is_empty() { ui.text(tx("spp.keine_spiele"), l.x + 14.0, l.y + 64.0, 16.0, Theme::text2()); }
            // own players with three grades or more: the whole season's grades as a curve instead of the last five bars
            if saison_noten.len() >= 3 {
                ui.text(&crate::tf!("ui.saison.notenverlauf", saison_noten.len()), l.x + 14.0, l.y + 62.0, 13.0, Theme::gedaempft());
                ui.kurve(Rect::new(l.x + 14.0, l.y + 72.0, l.w - 28.0, gh), &saison_noten, Theme::akzent(), false, &|x| format!("{x:.1}"));
            } else {
            for (j, n) in letzte.iter().enumerate() {
                let x = l.x + 40.0 + j as f32 * 96.0;
                let hgt = ((n - 4.0) / 6.0).clamp(0.05, 1.0) * (gh - 40.0);
                rrect(x - 18.0, l.y + 36.0 + gh - hgt, 36.0, hgt, 4.0, alpha(Theme::akzent(), 0.6));
                ui.note_pill(x, l.y + 56.0 + gh, *n);
            }
            }
            ui.text_semi(tx("spp.saison"), l.x + 14.0, l.y + 100.0 + gh, 14.0, Theme::text2());
            let werte = [(tx("spp.k.spiele"), pl.apps.to_string()), (tx("spp.k.tore"), pl.goals.to_string()), (tx("spp.k.vorlagen"), pl.assists.to_string()), (tx("spp.k.note"), if pl.rn > 0 { format!("{:.2}", pl.avg_rating()) } else { "–".into() }), (tx("spp.k.gelb"), pl.yellow.to_string())];
            let bw = (l.w - 28.0 - 4.0 * 8.0) / 5.0;
            for (i, (t, w)) in werte.iter().enumerate() {
                let r2 = Rect::new(l.x + 14.0 + i as f32 * (bw + 8.0), l.y + 114.0 + gh, bw, 64.0);
                rrect(r2.x, r2.y, r2.w, r2.h, 6.0, Theme::erhoehung());
                ui.text_semi(&t.to_uppercase(), r2.x + 10.0, r2.y + 18.0, 11.0, Theme::gedaempft());
                ui.title(w, r2.x + 10.0, r2.y + 52.0, 24.0, Theme::text());
            }
            // the season per competition (own players): matches, minutes, goals, assists, cards, grade
            let wb: Vec<&crate::spielgefuehl::SpielerWb> = c.spieler_saison.iter().filter(|x| x.pid == pl.id).collect();
            if !wb.is_empty() && l.h > 230.0 + gh {
                let y0 = l.y + 196.0 + gh;
                player_row_head(ui, l.x, y0, &[(crate::inhalt::t("ui.saison.wettbewerb"), 14.0), (tx("spp.k.spiele"), l.w * 0.46), (crate::inhalt::t("ui.saison.minuten"), l.w * 0.56), (tx("spp.k.tore"), l.w * 0.68), (tx("spp.k.vorlagen"), l.w * 0.78), (tx("spp.k.note"), l.w * 0.9)]);
                for (k, x) in wb.iter().enumerate() {
                    let y = y0 + 26.0 + k as f32 * 24.0;
                    if y > l.y + l.h - 10.0 { break; }
                    ui.text(&ui.fit(&x.wettbewerb, 14.0, l.w * 0.42), l.x + 14.0, y, 14.0, Theme::text());
                    for (i, w) in [x.spiele.to_string(), x.minuten.to_string(), x.tore.to_string(), x.vorlagen.to_string(), if x.noten > 0 { format!("{:.2}", x.notensumme / x.noten as f32) } else { "–".into() }].iter().enumerate() {
                        ui.text(w, l.x + l.w * [0.46, 0.56, 0.68, 0.78, 0.9][i], y, 14.0, Theme::text2());
                    }
                }
            }
            // own players: morale over the weeks with its reasons; the similar players below
            let r = if own && pl.ps.an { moral_karte(ui, c, p, Rect::new(r.x, r.y, r.w, 210.0)); Rect::new(r.x, r.y + 220.0, r.w, r.h - 220.0) } else { r };
            aehnliche_karte(ui, v, c, p, r);
        }
        4 => entwicklung_reiter(ui, c, p, cy, ch),
        6 if own => skilltree_reiter(ui, v, c, p, cy, ch),
        6 => { ui.card(Rect::new(x0() + 4.0, cy, cbreite(), ch)); ui.text(crate::inhalt::t("ui.st.nur_eigene"), x0() + 20.0, cy + 40.0, 16.0, Theme::text2()); }
        5 => { if stat_modal { ui.mouse = maus; ui.clicked = klick; } statistik_reiter(ui, v, c, p, own, cy, ch) }
        _ => {
            // career stations per season and the contract
            let (l, r) = (Rect::new(x0() + 4.0, cy, cbreite() * 0.62 - 6.0, ch), Rect::new(x0() + cbreite() * 0.62 + 10.0, cy, cbreite() * 0.38 - 6.0, ch));
            ui.card_titled(l, tx("spp.laufbahn"), Theme::akzent());
            player_row_head(ui, l.x, l.y + 56.0, &[(tx("spp.t.saison"), 14.0), (tx("spp.t.verein"), 130.0), (tx("spp.t.spiele"), 430.0), (tx("spp.t.tore"), 510.0), (tx("spp.t.note"), 580.0)]);
            let mut stationen: Vec<(String, String, u16, u16, f32)> = pl.profil.laufbahn.iter().map(|(s, cl, a, g, n)| (date::season_label(*s), cl.clone(), *a, *g, *n)).collect();
            stationen.push((crate::tf!("ui.seiten.laeuft", date::season_label(c.world.season)), clubname.clone(), pl.apps, pl.goals, pl.avg_rating()));
            // before PES M: the real career up to summer 2026 (realdaten spieler.json), oldest first
            let echt: Vec<(String, String, u16, u16, f32)> = crate::realdaten::spieler(pl.id).map(|rs| { let mut st: Vec<&crate::realdaten::Station> = rs.stationen.iter().collect(); st.sort_by_key(|x| (x.von, x.bis.unwrap_or(9999))); st }).map(|st| st.into_iter().map(|st| (match st.bis { Some(b) if b != st.von => format!("{}–{}", st.von, b), Some(_) => st.von.to_string(), None => format!("{}–", st.von) }, st.verein.clone(), st.spiele.unwrap_or(0) as u16, st.tore.unwrap_or(0) as u16, 0.0)).collect()).unwrap_or_default();
            let echt_leer = echt.is_empty();
            stationen.splice(0..0, echt);
            for (j, (s, cl, a, g, n)) in stationen.iter().rev().enumerate() {
                let y = l.y + 84.0 + j as f32 * 26.0;
                if y > l.y + l.h - 40.0 { break; }
                ui.text(s, l.x + 14.0, y, 15.0, Theme::text2());
                let cl = ui.fit(cl, 15.0, 290.0);
                ui.text_semi(&cl, l.x + 130.0, y, 15.0, Theme::text());
                let leer = *a == 0 && *g == 0 && *n == 0.0; // a real station without numbers
                ui.text(&if leer { "–".into() } else { a.to_string() }, l.x + 430.0, y, 15.0, Theme::text());
                ui.text(&if leer { "–".into() } else { g.to_string() }, l.x + 510.0, y, 15.0, Theme::text());
                ui.text(&if *n > 0.0 { format!("{n:.2}") } else { "–".into() }, l.x + 580.0, y, 15.0, Theme::text());
            }
            if pl.profil.laufbahn.is_empty() && echt_leer { ui.paragraph(tx("spp.laufbahn_leer"), l.x + 14.0, l.y + l.h - 40.0, 14.0, l.w - 28.0, Theme::gedaempft()); }
            if !echt_leer { ui.text(&crate::inhalt::t("ui.real.titel"), l.x + 14.0, l.y + l.h - 14.0, 12.0, Theme::gedaempft()); }
            ui.card_titled(r, tx("spp.d.vertrag"), Theme::gold());
            let mut z: Vec<(String, String)> = vec![(tx("spp.d.vertrag").into(), (date::START_YEAR as u32 + pl.contract_end + 1).to_string()), (tx("spp.gehalt").into(), spanne(pl.wage)), (tx("spp.marktwert").into(), spanne(pl.value))];
            if let Some(le) = &pl.leihe { z.push((crate::inhalt::t("dlg.punkt.leihe").into(), crate::tf!("ui.seiten.von_bis", c.world.clubs[le.von].name, le.rueckkehr()))); }
            z.extend(c.klausel_zeilen(p));
            // real titles up to summer 2026: the three biggest collections
            if let Some(rs) = crate::realdaten::spieler(pl.id) {
                let mut t: Vec<&(String, Vec<u32>)> = rs.titel.iter().filter(|x| !x.1.is_empty()).collect();
                t.sort_by_key(|x| std::cmp::Reverse(x.1.len()));
                if !t.is_empty() { z.push((crate::inhalt::t("ui.real.titel_spieler").into(), t.iter().take(3).map(|(k, j)| format!("{}× {k}", j.len())).collect::<Vec<_>>().join(", "))); }
            }
            let eh = c.ehrungen_von(p);
            if !eh.is_empty() { z.push((crate::inhalt::t("ui.ehrung.ehrungen").into(), crate::tf!("ui.ehrung.anzahl", eh.len(), eh[0].titel.split(':').next().unwrap_or("").to_string()))); }
            if let Some(n) = c.elf_der_woche.iter().find(|x| x.0 == pl.id).map(|x| x.1) { z.push((crate::inhalt::t("ui.ehrung.elf_der_woche").into(), crate::tf!("ui.ehrung.mal", n))); }
            // the injury history, newest first (FC-26 wish)
            if pl.profil.verletzungen.is_empty() { z.push((crate::inhalt::t("ui.verl.historie").into(), crate::inhalt::t("ui.verl.keine").into())); }
            for (k, (s, d, name, tage)) in pl.profil.verletzungen.iter().rev().take(5).enumerate() {
                z.push((if k == 0 { crate::inhalt::t("ui.verl.historie").to_string() } else { String::new() }, crate::tf!("ui.verl.zeile", date::short(*s, *d), name.clone(), tage)));
            }
            if pl.listed { z.push((crate::inhalt::t("gl.chip.transferliste").into(), if pl.profil.mindestpreis > 0.0 { crate::tf!("ui.seiten.ja_mindestpreis", money(pl.profil.mindestpreis)) } else { crate::inhalt::t("ui.seiten.ja_jedes_angebot_wird_geprueft").into() })); }
            zeilen(ui, r, &z, 64.0);
            // second team and farm teams: move, loan to a partner (with or without play time), call back
            let mut y = r.y + 64.0 + z.len() as f32 * 30.0 + 16.0;
            // a buy-back option of ours (klauseln.rs): bring him back for the agreed fee
            if pl.profil.klauseln.rueckkauf.map_or(false, |x| x.0 == c.own && c.world.season <= x.2) && pl.team >= 0 && pl.team != c.own as i32 {
                if ui.button(Rect::new(r.x + 14.0, y, r.w - 28.0, 32.0), crate::inhalt::t("ui.klausel.ziehen"), true) {
                    match c.rueckkauf_ziehen(p) { Ok(t) => v.toast(&t, true), Err(e) => v.toast(&e, false) }
                }
                y += 40.0;
            }
            if let Some(g) = &pl.leihe { if !g.grund.is_empty() { ui.paragraph(&g.grund, r.x + 14.0, y, 14.0, r.w - 28.0, Theme::text2()); y += 40.0; } }
            let bw = r.w - 28.0;
            let mut t: Option<Result<String, String>> = None;
            if own && pl.leihe.is_none() && c.zweite_mannschaft().is_some() {
                if ui.button(Rect::new(r.x + 14.0, y, bw, 30.0), if im_kader { crate::inhalt::t("ui.seiten.in_die_zweite_mannschaft") } else { crate::inhalt::t("ui.seiten.in_die_erste_mannschaft") }, false) { t = Some(c.mannschaft_wechseln(p)); }
                y += 38.0;
            }
            if own && pl.leihe.is_none() {
                for partner in c.world.partner_von(c.own) {
                    let n = c.world.clubs[partner].name.clone();
                    if ui.button(Rect::new(r.x + 14.0, y, bw * 0.55, 30.0), &crate::tf!("ui.seiten.an_verleihen", ui.fit(&n, 15.0, bw * 0.4)), false) { t = Some(c.leihe_an_partner(p, partner, false)); }
                    if ui.button(Rect::new(r.x + 22.0 + bw * 0.55, y, bw * 0.45 - 8.0, 30.0), crate::inhalt::t("ui.seiten.mit_spielzeit_zusage"), false) { t = Some(c.leihe_an_partner(p, partner, true)); }
                    y += 38.0;
                }
            }
            if pl.leihe.as_ref().is_some_and(|l| l.von == c.own) && pl.team >= 0 && c.world.koop_index(c.own, pl.team as usize).is_some() && ui.button(Rect::new(r.x + 14.0, y, bw, 30.0), crate::inhalt::t("ui.seiten.zurueckrufen"), false) { t = Some(c.leihe_zurueckrufen(p)); }
            match t { Some(Ok(s)) => v.toast(&s, true), Some(Err(e)) => v.toast(&e, false), None => {} }
        }
    }
}
/// Morale of the last weeks as a curve, the reasons as points on it (green good, red bad) and the latest reasons in words.
fn moral_karte(ui: &mut Ui, c: &Career, p: usize, r: Rect) {
    let pl = &c.world.players[p];
    ui.card_titled(r, crate::inhalt::t("ui.wellen.moral_verlauf"), Theme::akzent());
    let mut werte = c.moral_verlauf.get(&pl.id).cloned().unwrap_or_default();
    werte.push(pl.morale);
    let (gx, gy, gw, gh) = (r.x + 14.0, r.y + 52.0, r.w * 0.5 - 20.0, r.h - 66.0);
    rrect(gx, gy, gw, gh, 4.0, Theme::erhoehung());
    let n = werte.len();
    let heute = c.abs();
    let start = heute.saturating_sub(7 * (n as u32 - 1));
    let xv = |i: f32| gx + 6.0 + if n > 1 { i / (n - 1) as f32 * (gw - 12.0) } else { gw / 2.0 };
    let yv = |m: f32| gy + gh - 4.0 - m.clamp(0.0, 100.0) / 100.0 * (gh - 8.0);
    for i in 1..n { draw_line(xv((i - 1) as f32), yv(werte[i - 1]), xv(i as f32), yv(werte[i]), 2.0, Theme::akzent()); }
    let gruende: Vec<&crate::mensch::Grund> = c.mensch.gruende.iter().filter(|g| g.bezug == pl.name && g.wert.abs() >= 0.3 && g.abs >= start).collect();
    for g in &gruende {
        let i = (g.abs - start) as f32 / 7.0;
        let m = werte[(i.round() as usize).min(n - 1)];
        draw_circle(xv(i.min((n - 1) as f32)), yv(m), 4.0, if g.wert > 0.0 { Theme::gut() } else { Theme::problem() });
    }
    let mut y = gy + 14.0;
    for g in gruende.iter().rev().take(6) {
        let t = crate::tf!("ui.wellen.grund_zeile", format!("{:+.1}", g.wert).replace('-', "−"), &g.grund);
        ui.text(&ui.fit(&t, 13.0, r.w * 0.5 - 20.0), r.x + r.w * 0.5 + 4.0, y, 13.0, if g.wert > 0.0 { Theme::text() } else { Theme::problem() });
        y += 22.0;
    }
    if gruende.is_empty() { ui.text(crate::inhalt::t("ui.wellen.keine_gruende"), r.x + r.w * 0.5 + 4.0, y, 13.0, Theme::gedaempft()); }
}
fn aehnliche_karte(ui: &mut Ui, v: &mut View, c: &mut Career, p: usize, r: Rect) {
    ui.card_titled(r, tx("spp.aehnliche"), Theme::gold());
    ui.text(tx("spp.aehnliche_hinweis"), r.x + 14.0, r.y + 60.0, 14.0, Theme::gedaempft());
    for (j, q) in aehnliche(c, p).into_iter().enumerate() {
        let rr = Rect::new(r.x + 10.0, r.y + 74.0 + j as f32 * 46.0, r.w - 20.0, 42.0);
        if rr.y + rr.h > r.y + r.h - 6.0 { break; }
        rrect(rr.x, rr.y, rr.w, rr.h, 6.0, if ui.hover(rr) { Theme::ueberlagerung() } else { Theme::erhoehung() });
        let ql = &c.world.players[q];
        let club = if ql.team >= 0 { c.world.clubs[ql.team as usize].name.clone() } else { crate::inhalt::t("ui.wort.vereinslos").into() };
        let (lo, hi) = c.ovr_range(q);
        ui.player_photo(rr.x + 24.0, rr.y + 21.0, 15.0, ql.id, &ql.name);
        let nm = ui.fit(&ql.name, 16.0, rr.w - 250.0);
        ui.text_semi(&nm, rr.x + 46.0, rr.y + 18.0, 16.0, Theme::text());
        let cl = ui.fit(&club, 13.0, rr.w - 300.0);
        ui.text(&crate::tf!("finanzen.zeile", cl, ql.age), rr.x + 46.0, rr.y + 36.0, 13.0, Theme::text2());
        ui.text_right(&if hi - lo <= 1.0 { format!("{:.0}", ql.ovr) } else { format!("{lo:.0}–{hi:.0}") }, rr.x + rr.w - 14.0, rr.y + 27.0, 18.0, Theme::skala((lo + hi) / 2.0));
        if ui.hit(rr) { v.sel_player = Some(q); v.tab = 0; }
    }
}
