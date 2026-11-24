use super::*;

/// Kaderplanung (Fabian 23.11.: "unspectacular, needs much more info, assessments and ideas"): the depth chart per
/// position group with a traffic light, the timeline of contracts and age, and suggestions with their reasons.
pub(super) fn kaderplanung(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    const GR: [&str; 9] = ["tw", "iv", "av", "dm", "zm", "om", "fl", "hs", "st"]; // roh
    let sq: Vec<usize> = c.squad().into_iter().filter(|&p| c.world.players[p].youth == 0).collect();
    let season = c.world.season;
    let w3 = (cbreite() - 24.0) / 3.0;
    let h = hoehe() - Y0 - 16.0;
    let (l, m, r) = (Rect::new(x0() + 4.0, Y0, w3, h), Rect::new(x0() + 16.0 + w3, Y0, w3, h), Rect::new(x0() + 28.0 + 2.0 * w3, Y0, w3, h));
    // 1) depth chart
    ui.card_titled(l, t("ui.kp.tiefe"), Theme::akzent());
    let mut y = l.y + 56.0;
    let mut klick = None;
    for gr in GR {
        let mut ps: Vec<usize> = sq.iter().copied().filter(|&p| crate::scoutbericht::gruppe(&c.world.players[p].pos) == gr).collect();
        ps.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
        let ampel = c.kaderluecke(gr);
        let farbe = [Theme::gut(), Theme::achtung(), Theme::problem()][ampel.min(2) as usize];
        if y > l.y + l.h - 30.0 { break; }
        draw_circle(l.x + 20.0, y - 5.0, 6.0, farbe);
        ui.text_semi(t(&format!("ui.auftrag.gruppe.{gr}")), l.x + 34.0, y, 15.0, Theme::text());
        ui.text_right(&crate::tf!("ui.kp.anzahl", ps.len()), l.x + l.w - 14.0, y, 13.0, Theme::text2());
        y += 20.0;
        for p in ps.iter().take(4) {
            let pl = &c.world.players[*p];
            let rr = Rect::new(l.x + 30.0, y - 14.0, l.w - 44.0, 20.0);
            if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
            let ende = pl.contract_end <= season;
            ui.text(&ui.fit(&pl.name, 13.0, l.w * 0.5), l.x + 34.0, y, 13.0, if ende { Theme::achtung() } else { Theme::text2() });
            ui.text_right(&format!("{} · {:.0}", pl.age, pl.ovr), l.x + l.w - 14.0, y, 13.0, Theme::text2());
            if ui.hit(rr) { klick = Some(*p); }
            y += 20.0;
        }
        y += 8.0;
    }
    // 2) timeline: contracts, age, wages
    ui.card_titled(m, t("ui.kp.zeitachse"), Theme::gold());
    let mut y = m.y + 56.0;
    let abschnitt = |ui: &mut Ui, y: &mut f32, titel: &str| { ui.text_semi(&titel.to_uppercase(), m.x + 14.0, *y, 12.0, Theme::akzent()); *y += 20.0; };
    let enden_jetzt: Vec<usize> = sq.iter().copied().filter(|&p| c.world.players[p].contract_end <= season).collect();
    let enden_naechst: Vec<usize> = sq.iter().copied().filter(|&p| c.world.players[p].contract_end == season + 1).collect();
    let alt: Vec<usize> = sq.iter().copied().filter(|&p| c.world.players[p].age >= 31).collect();
    let talente: Vec<usize> = c.world.clubs[c.own].squad.iter().copied().filter(|&p| { let pl = &c.world.players[p]; pl.age <= 21 && pl.pot >= pl.ovr + 8.0 }).collect();
    for (titel, liste) in [(t("ui.kp.vertrag_endet"), &enden_jetzt), (t("ui.kp.vertrag_naechstes"), &enden_naechst), (t("ui.kp.aelter"), &alt), (t("ui.kp.talente"), &talente)] {
        if y > m.y + m.h - 60.0 { break; }
        abschnitt(ui, &mut y, &crate::tf!("ui.kp.titel_anzahl", titel, liste.len()));
        if liste.is_empty() { ui.text(t("ui.kp.niemand"), m.x + 14.0, y, 13.0, Theme::gedaempft()); y += 22.0; continue; }
        for p in liste.iter().take(5) {
            let pl = &c.world.players[*p];
            let rr = Rect::new(m.x + 10.0, y - 14.0, m.w - 20.0, 20.0);
            if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
            ui.pos_tag(m.x + 14.0, y - 4.5, pos_de(&pl.pos));
            ui.text(&ui.fit(&pl.name, 13.0, m.w * 0.6 - 44.0), m.x + 58.0, y, 13.0, Theme::text());
            ui.text_right(&format!("{} · {:.0}/{:.0} · {}", pl.age, pl.ovr, pl.pot, money(pl.wage)), m.x + m.w - 14.0, y, 12.0, Theme::text2());
            if ui.hit(rr) { klick = Some(*p); }
            y += 20.0;
        }
        y += 10.0;
    }
    let lohn = c.world.wage_bill(c.own);
    if y < m.y + m.h - 40.0 {
        abschnitt(ui, &mut y, t("ui.kp.geld"));
        ui.text(&crate::tf!("ui.kp.geld_zeile", money(lohn), money(c.gehalt_frei()), money(c.transfer_budget)), m.x + 14.0, y, 13.0, Theme::text2());
    }
    // 3) ideas with reasons
    ui.card_titled(r, t("ui.kp.ideen"), Theme::akzent());
    let mut ideen: Vec<(String, Option<usize>, Option<&str>)> = vec![];
    // a missing group only counts when the formation plays it (no second striker in a 4-3-3 is no gap)
    let slots: Vec<&str> = c.eigene_slots().iter().map(|s| crate::scoutbericht::gruppe(s)).collect();
    for gr in GR { if c.kaderluecke(gr) == 2 && slots.contains(&gr) { ideen.push((crate::tf!("ui.kp.idee_kaufen", t(&format!("ui.auftrag.gruppe.{gr}"))), None, Some(gr))); } }
    for &p in &enden_jetzt { let pl = &c.world.players[p]; if pl.ovr >= c.team_strength() - 2.0 && pl.age <= 30 { ideen.push((crate::tf!("ui.kp.idee_verlaengern", pl.name.clone()), Some(p), None)); } }
    for &p in &sq { let pl = &c.world.players[p]; if pl.age >= 30 && pl.apps < 5 && pl.wage > lohn / sq.len().max(1) as f64 * 1.3 && c.world.day > 60 { ideen.push((crate::tf!("ui.kp.idee_verkaufen", pl.name.clone(), money(pl.wage)), Some(p), None)); } }
    for &p in &sq { let pl = &c.world.players[p]; if pl.age <= 21 && pl.pot >= pl.ovr + 8.0 && pl.apps < 3 && c.world.day > 45 && pl.leihe.is_none() { ideen.push((crate::tf!("ui.kp.idee_leihen", pl.name.clone()), Some(p), None)); } }
    for &p in &talente { let pl = &c.world.players[p]; if pl.youth > 0 && pl.ovr >= c.team_strength() - 8.0 { ideen.push((crate::tf!("ui.kp.idee_hochziehen", pl.name.clone()), Some(p), None)); } }
    for &p in &alt { let pl = &c.world.players[p]; if pl.ps.rolle <= 3 && pl.age >= 33 { ideen.push((crate::tf!("ui.kp.idee_nachfolger", pl.name.clone()), Some(p), None)); } }
    if ideen.is_empty() { ui.paragraph(t("ui.kp.keine_ideen"), r.x + 14.0, r.y + 60.0, 14.0, r.w - 28.0, Theme::text2()); }
    let mut y = r.y + 58.0;
    for (text, p, gruppe) in ideen.into_iter().take(12) {
        if y > r.y + r.h - 40.0 { break; }
        let hh = ui.paragraph(&text, r.x + 14.0, y, 13.0, r.w - 120.0, Theme::text()) - y;
        let br = Rect::new(r.x + r.w - 98.0, y - 14.0, 84.0, 24.0);
        if ui.button(br, t(if gruppe.is_some() { "ui.kp.suchen" } else { "ui.kp.ansehen" }), false) {
            if let Some(p) = p { klick = Some(p); } else if let Some(g) = gruppe { let _ = g; v.go(Page::Scouting); }
        }
        y += hh.max(24.0) + 8.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
}
/// Player comparison (Fabian 23.11.): up to three players side by side, each chosen by a search over all players; photo,
/// club, age, height, weight, foot, values, contract, season numbers, skills and all 25 abilities with the best marked.
pub(super) fn vergleich(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    if v.vergleich.iter().all(|x| x.is_none()) { v.vergleich[0] = v.sel_player.or_else(|| c.squad().first().copied()); }
    // a second player to start with: the strongest other one of the same position in the squad
    if let (Some(a), None, None) = (v.vergleich[0], v.vergleich[1], v.vergleich[2]) {
        v.vergleich[1] = c.squad().into_iter().filter(|&p| p != a && c.world.players[p].pos == c.world.players[a].pos).max_by(|x, y| c.world.players[*x].ovr.partial_cmp(&c.world.players[*y].ovr).unwrap());
    }
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card(area);
    let lw = 190.0;
    // three columns and a strip on the right for the spider
    let sw = (area.w - lw - 28.0 - 270.0) / 3.0;
    let spieler: Vec<Option<usize>> = v.vergleich.to_vec();
    let farben = [Theme::akzent(), Theme::gold(), hex(0xa78bfa)];
    // the head of each column: photo, name and a search to change the player
    for k in 0..3 {
        let x = area.x + lw + 14.0 + k as f32 * sw;
        let feld = Rect::new(x + 6.0, area.y + 10.0, sw - 12.0, 34.0);
        let aktiv = v.vergleich_suche.0 == k;
        if ui.hit(feld) { v.vergleich_suche = (k, String::new()); }
        let mut q = if aktiv { v.vergleich_suche.1.clone() } else { String::new() };
        ui.input(feld, &mut q, aktiv, t("ui.vgl.suchen"));
        if aktiv {
            v.vergleich_suche.1 = q.clone();
            if q.chars().count() >= 2 {
                let ql = q.to_lowercase();
                let treffer: Vec<usize> = (0..c.world.players.len()).filter(|&p| c.world.players[p].team != -2 && c.world.players[p].name.to_lowercase().contains(&ql)).take(8).collect();
                for (i, p) in treffer.iter().enumerate() {
                    let rr = Rect::new(feld.x, feld.y + 38.0 + i as f32 * 26.0, feld.w, 24.0);
                    rrect(rr.x, rr.y, rr.w, rr.h, 4.0, if ui.hover(rr) { Theme::ueberlagerung() } else { Theme::erhoehung() });
                    let pl = &c.world.players[*p];
                    let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].short.clone() } else { "–".into() };
                    ui.text(&ui.fit(&format!("{} · {} · {:.0}", pl.name, club, pl.ovr), 13.0, rr.w - 12.0), rr.x + 6.0, rr.y + 17.0, 13.0, Theme::text());
                    if ui.hit(rr) { v.vergleich[k] = Some(*p); v.vergleich_suche = (usize::MAX, String::new()); }
                }
                if !treffer.is_empty() { continue; }
            }
        }
        let Some(p) = spieler[k] else { ui.text_center(t("ui.vgl.leer"), x + sw / 2.0, area.y + 110.0, 14.0, Theme::gedaempft()); continue };
        let pl = &c.world.players[p];
        ui.player_photo(x + 34.0, area.y + 86.0, 26.0, pl.id, &pl.name);
        draw_circle_lines(x + 34.0, area.y + 86.0, 27.0, 2.5, farben[k]);
        ui.text_semi(&ui.fit(&pl.name, 16.0, sw - 80.0), x + 68.0, area.y + 80.0, 16.0, Theme::text());
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { t("ui.vgl.vereinslos").to_string() };
        ui.text(&ui.fit(&club, 13.0, sw - 80.0), x + 68.0, area.y + 100.0, 13.0, Theme::text2());
        if spieler.iter().flatten().count() > 1 && ui.button(Rect::new(x + sw - 34.0, area.y + 60.0, 26.0, 24.0), "×", false) { v.vergleich[k] = None; } // roh
    }
    // the rows: facts, then season numbers, then abilities; the best value of a row in its column's colour
    let fakten: Vec<(String, Box<dyn Fn(&crate::world::Player) -> (String, f32)>)> = vec![
        (t("ui.vgl.alter").into(), Box::new(|p| (p.age.to_string(), -(p.age as f32)))),
        (t("ui.vgl.groesse").into(), Box::new(|p| (if p.profil.koerper[0] > 0 { format!("{} cm", p.profil.koerper[0]) } else { "–".into() }, p.profil.koerper[0] as f32))),
        (t("ui.vgl.gewicht").into(), Box::new(|p| (if p.profil.koerper[1] > 0 { format!("{} kg", p.profil.koerper[1]) } else { "–".into() }, f32::NAN))),
        (t("ui.vgl.position").into(), Box::new(|p| (pos_de(&p.pos).to_string(), f32::NAN))),
        (t("ui.vgl.staerke").into(), Box::new(|p| (format!("{:.0}", p.ovr), p.ovr))),
        (t("ui.vgl.potenzial").into(), Box::new(|p| (format!("{:.0}", p.pot), p.pot))),
        (t("ui.vgl.marktwert").into(), Box::new(|p| (money(p.value), p.value as f32))),
        (t("ui.vgl.gehalt").into(), Box::new(|p| (money(p.wage), -(p.wage as f32)))),
        (t("ui.vgl.vertrag").into(), Box::new(|p| ((crate::date::START_YEAR as u32 + p.contract_end + 1).to_string(), f32::NAN))),
        (t("ui.vgl.spiele_tore").into(), Box::new(|p| (format!("{} / {}", p.apps, p.goals), p.goals as f32))),
    ];
    let mut y = area.y + 136.0;
    let zeile = |ui: &mut Ui, y: f32, name: &str, werte: &[(String, f32)]| {
        ui.text(name, area.x + 14.0, y, 14.0, Theme::text2());
        let best = werte.iter().filter(|w| !w.1.is_nan()).map(|w| w.1).fold(f32::MIN, f32::max);
        for (k, (txt, w)) in werte.iter().enumerate() {
            let x = area.x + lw + 14.0 + k as f32 * sw;
            let top = werte.iter().filter(|w| !w.1.is_nan()).count() > 1 && *w == best;
            ui.text(txt, x + 10.0, y, 14.0, if top { farben[k] } else { Theme::text() });
        }
    };
    let aktive: Vec<(usize, usize)> = spieler.iter().enumerate().filter_map(|(k, p)| p.map(|p| (k, p))).collect();
    for (name, f) in &fakten {
        let mut werte = vec![(String::new(), f32::NAN); 3];
        for &(k, p) in &aktive { werte[k] = f(&c.world.players[p]); }
        zeile(ui, y, name, &werte);
        y += 22.0;
    }
    // the spider of the ability groups, all players on top of each other in their colours (Fabian 23.11.)
    if aktive.iter().all(|&(_, p)| c.world.players[p].attrs.len() == 25) && !aktive.is_empty() {
        let gr = gruppen();
        let n = gr.len();
        let (mx, my, rad) = (area.x + area.w - 135.0, area.y + 200.0, 90.0);
        let pt = |i: usize, t: f32| { let w = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / n as f32; vec2(mx + w.cos() * rad * t, my + w.sin() * rad * t) };
        for ring in [0.5, 0.75, 1.0] { for i in 0..n { let (a, b) = (pt(i, ring), pt((i + 1) % n, ring)); draw_line(a.x, a.y, b.x, b.y, 1.0, Theme::kontur()); } }
        for (i, (id, _, _)) in gr.iter().enumerate() { let q = pt(i, 1.18); ui.text_center(&ui.fit(tx(&format!("spp.gruppe.{id}")), 11.0, 70.0), q.x, q.y + 4.0, 11.0, Theme::text2()); }
        for &(k, p) in &aktive {
            let pl = &c.world.players[p];
            let wert: Vec<f32> = gr.iter().map(|(_, ix, _)| ix.iter().map(|&i| pl.attrs[i]).sum::<f32>() / ix.len().max(1) as f32).collect();
            for i in 0..n {
                let (a, b) = (pt(i, (wert[i] - 30.0).max(0.0) / 69.0), pt((i + 1) % n, (wert[(i + 1) % n] - 30.0).max(0.0) / 69.0));
                draw_line(a.x, a.y, b.x, b.y, 2.5, farben[k]);
                draw_triangle(vec2(mx, my), a, b, alpha(farben[k], 0.12));
            }
        }
    }
    // skills, shortened
    ui.text(t("ui.vgl.skills"), area.x + 14.0, y, 14.0, Theme::text2());
    for &(k, p) in &aktive {
        let pl = &c.world.players[p];
        let sk: Vec<&str> = crate::pesdb::SKILLS.iter().enumerate().filter(|(i, _)| pl.skills >> i & 1 == 1).map(|(_, s)| s.0).collect();
        let x = area.x + lw + 14.0 + k as f32 * sw;
        ui.paragraph(&if sk.is_empty() { "–".to_string() } else { sk.join(", ") }, x + 10.0, y, 12.0, sw - 20.0, Theme::text());
    }
    y += 46.0;
    // the 25 abilities as bars
    let hat_werte = aktive.iter().all(|&(_, p)| c.world.players[p].attrs.len() == 25);
    if !hat_werte { ui.text(t("ui.seiten.fuer_diese_spieler_sind_keine"), area.x + 14.0, y, 14.0, Theme::text2()); return; }
    let zh = ((area.y + area.h - y - 10.0) / 25.0).clamp(12.0, 22.0);
    for i in 0..25 {
        let yy = y + i as f32 * zh;
        ui.text(crate::pesdb::attr_de_t()[i], area.x + 14.0, yy + zh * 0.7, (zh * 0.65).min(14.0), Theme::text2());
        let best = aktive.iter().map(|&(_, p)| c.world.players[p].attrs[i]).fold(0.0f32, f32::max);
        for &(k, p) in &aktive {
            let w = c.world.players[p].attrs[i];
            let x = area.x + lw + 14.0 + k as f32 * sw;
            ui.bar(x + 10.0, yy + zh * 0.3, sw - 60.0, zh * 0.4, w / 99.0, if w >= best && aktive.len() > 1 { farben[k] } else { Theme::ueberlagerung() });
            ui.text_right(&format!("{w:.0}"), x + sw - 10.0, yy + zh * 0.75, (zh * 0.65).min(14.0), Theme::text());
        }
    }
}

pub(super) fn rollen_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // Paket 4: roles per slot and instructions per player (rollenwissen.json of the original)
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(r, crate::inhalt::t("ui.seiten.rollen_je_platz_und_anweisungen"), Theme::akzent());
    // the game's style switches that the tactic panel does not have yet (gegner.toml [schalter] stellungsspiel, kontrollbereich)
    ui.text(crate::inhalt::t("achse.Stellungsspiel"), r.x + 14.0, r.y + 62.0, 15.0, Theme::text2());
    if let Some(i) = ui.segment(Rect::new(r.x + 140.0, r.y + 42.0, 300.0, 28.0), &[crate::inhalt::t("gb.stilfeld.stellungsspiel.0"), crate::inhalt::t("gb.stilfeld.stellungsspiel.1")], c.tactic.stellung as usize) { c.tactic.stellung = i as u8; }
    ui.text(crate::inhalt::t("taktik.wert.kontrollbereich"), r.x + 470.0, r.y + 62.0, 15.0, Theme::text2());
    if let Some(i) = ui.segment(Rect::new(r.x + 600.0, r.y + 42.0, 240.0, 28.0), &[crate::inhalt::t("tk.kompakt.k.angriffsbereich.1"), crate::inhalt::t("tk.kompakt.k.kontrollbereich.1")], c.tactic.kontrolle as usize) { c.tactic.kontrolle = i as u8; }
    let xi = c.current_xi();
    for (k, (slot, p, _)) in xi.iter().enumerate() {
        let yy = r.y + 110.0 + k as f32 * 40.0;
        let pl = c.world.players[*p].name.clone();
        // the role is the player's real playing style of the game (Fabian 23.11.: "The Destroyer", "Creative
        // Playmaker" …); a change goes into Player.bin with the next write
        let stile: Vec<u8> = crate::regeln::wert("spielstile", slot).as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|v| v as u8)).collect()).unwrap_or_default();
        ui.pos_tag(r.x + 14.0, yy - 5.0, pos_de(slot));
        ui.text(&pl, r.x + 58.0, yy, 16.0, Theme::text());
        let alle = tx_liste("spp.stil");
        let namen: Vec<&str> = stile.iter().map(|&s| alle.get(s as usize).copied().unwrap_or("?")).collect();
        let akt = c.world.players[*p].style;
        let sel = stile.iter().position(|&s| s == akt).unwrap_or(usize::MAX);
        if !namen.is_empty() { if let Some(i) = ui.segment(Rect::new(r.x + 280.0, yy - 20.0, (namen.len() as f32 * 150.0).min(r.w * 0.5), 28.0), &namen, sel) { c.world.players[*p].style = stile[i]; } }
        let anw = c.std.anweisungen.iter().find(|x| x.0 == c.world.players[*p].id).map(|x| x.1).unwrap_or(0);
        if let Some(i) = ui.segment(Rect::new(r.x + r.w - 380.0, yy - 20.0, 366.0, 28.0), &crate::taktik::anweisungen_t(), anw as usize) { let id = c.world.players[*p].id; c.std.anweisungen.retain(|x| x.0 != id); c.std.anweisungen.push((id, i as u8)); }
    }
    let _ = v;
}
