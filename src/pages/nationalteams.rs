use super::*;

/// Konzept Nationalteams Teil 7: our internationals, the FIFA ranking and the tournaments (past and coming).
pub(super) fn nationalteams_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    tabs(ui, v, &[t("ui.nt.reiter.ueberblick"), t("ui.nt.reiter.laender"), t("ui.nl.seite")]);
    if v.tab == 1 { return nation_seite(ui, v, c); }
    if v.tab == 2 { return nl_seite(ui, v, c); }
    let w3 = (cbreite() - 24.0) / 3.0;
    let h = hoehe() - Y0 - 16.0;
    let (l, m, r) = (Rect::new(x0() + 4.0, Y0, w3, h), Rect::new(x0() + 16.0 + w3, Y0, w3, h), Rect::new(x0() + 28.0 + 2.0 * w3, Y0, w3, h));
    ui.card_titled(l, t("ui.nt.unsere"), Theme::verein());
    let mut eigene: Vec<usize> = c.squad().into_iter().filter(|&p| c.world.players[p].nat > 0).collect();
    eigene.sort_by_key(|&p| (c.world.nat_status(p).unwrap_or(9), std::cmp::Reverse(c.world.players[p].profil.laenderspiele)));
    let mut y = l.y + 60.0;
    let mut klick = None;
    for p in eigene.into_iter().take(((l.h - 70.0) / 26.0) as usize) {
        let pl = &c.world.players[p];
        let rr = Rect::new(l.x + 8.0, y - 18.0, l.w - 16.0, 24.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::ueberlagerung()); }
        ui.text(&ui.fit(&pl.name, 14.0, l.w * 0.42), l.x + 14.0, y, 14.0, Theme::text());
        ui.text(&ui.fit(&crate::nationen::name_anzeige(&c.world, pl.nat), 13.0, l.w * 0.24), l.x + l.w * 0.46, y, 13.0, Theme::text2());
        let st = c.world.nat_status(p).map(|s| t(&format!("ui.nt.kurz.{s}"))).unwrap_or("–");
        ui.text(st, l.x + l.w * 0.70, y, 13.0, Theme::akzent());
        ui.text_right(&format!("{}", pl.profil.laenderspiele), l.x + l.w - 14.0, y, 13.0, Theme::text2());
        if ui.hit(rr) { klick = Some(p); }
        y += 26.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
    // the ranking: the nations with a team, by points
    ui.card_titled(m, t("ui.nt.rangliste"), Theme::gold());
    let mut nats: Vec<u16> = c.world.nationalteams.iter().map(|x| x.nat).filter(|&n| !crate::nationen::name(&c.world, n).is_empty()).collect();
    nats.sort_by(|a, b| c.world.fifa_punkte(*b).partial_cmp(&c.world.fifa_punkte(*a)).unwrap());
    if nats.is_empty() { ui.paragraph(t("ui.nt.noch_keine"), m.x + 14.0, m.y + 60.0, 14.0, m.w - 28.0, Theme::text2()); }
    let eigen_land = c.world.land_nat(c.league());
    for (i, n) in nats.iter().take(((m.h - 70.0) / 22.0) as usize).enumerate() {
        let y = m.y + 60.0 + i as f32 * 22.0;
        let col = if *n == eigen_land { Theme::gold() } else { Theme::text() };
        ui.text(&format!("{}.", i + 1), m.x + 14.0, y, 13.0, Theme::gedaempft());
        ui.text(&ui.fit(&crate::nationen::name_anzeige(&c.world, *n), 14.0, m.w * 0.55), m.x + 48.0, y, 14.0, col);
        ui.text_right(&format!("{:.0}", c.world.fifa_punkte(*n)), m.x + m.w - 14.0, y, 13.0, Theme::text2());
    }
    // tournaments: the coming ones from the cycle, then the history
    ui.card_titled(r, t("ui.nt.turniere"), Theme::akzent());
    let (jahr, _, _) = crate::date::ymd(c.world.season, c.world.day);
    let mut kommend: Vec<(i32, String)> = vec![];
    for w in crate::regeln::wert("nationalwettbewerbe", crate::id("wettbewerb")).as_array().cloned().unwrap_or_default() {
        let kz = w["kuerzel"].as_str().unwrap_or("");
        if w["format"].as_str() != Some("gruppen_ko") || kz.starts_with(crate::id("u2")) || kz == "olympia" { continue; }
        let (e, rh) = (w["erstes_jahr"].as_i64().unwrap_or(0) as i32, w["rhythmus"].as_i64().unwrap_or(4).max(1) as i32);
        let mut j = e.max(jahr);
        while (j - e) % rh != 0 { j += 1; }
        if c.world.turniere.iter().any(|x| x.jahr == j && Some(x.name.as_str()) == w["name"].as_str()) { j += rh; }
        kommend.push((j, w["name"].as_str().unwrap_or("").to_string()));
    }
    kommend.sort();
    let mut y = r.y + 60.0;
    ui.text_semi(t("ui.nt.kommend"), r.x + 14.0, y, 14.0, Theme::akzent());
    y += 22.0;
    for (j, n) in kommend.iter().take(6) { ui.text(&format!("{j} · {n}"), r.x + 14.0, y, 14.0, Theme::text()); y += 20.0; }
    y += 14.0;
    ui.text_semi(t("ui.nt.geschichte"), r.x + 14.0, y, 14.0, Theme::akzent());
    y += 22.0;
    if c.world.turniere.is_empty() { ui.text(t("ui.cup.keine"), r.x + 14.0, y, 14.0, Theme::gedaempft()); }
    for x in c.world.turniere.iter().rev() {
        if y > r.y + r.h - 12.0 { break; }
        ui.text(&ui.fit(&crate::tf!("ui.nt.sieger_zeile", x.jahr, x.name.clone(), crate::nationen::name_anzeige(&c.world, x.sieger)), 14.0, r.w - 28.0), r.x + 14.0, y, 14.0, Theme::text());
        y += 20.0;
    }
}

/// Nationalteams › Länder, built like the club page (Fabian 23.11.): the ranking on the left, on the right the nation's
/// hero with facts, the coach and his philosophy, titles and the squad by lines.
fn nation_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let t = crate::inhalt::t;
    let h = hoehe() - Y0 - 16.0;
    let lw = (cbreite() * 0.30).max(280.0);
    let l = Rect::new(x0() + 4.0, Y0, lw, h);
    let r = Rect::new(x0() + lw + 16.0, Y0, cbreite() - lw - 12.0, h);
    // every nation with enough players, also before the first international break (then ranked by strength)
    let mut zahl: std::collections::HashMap<u16, usize> = Default::default();
    for p in &c.world.players { if p.nat > 0 && p.team != -2 { *zahl.entry(p.nat).or_default() += 1; } }
    let mut nats: Vec<u16> = zahl.into_iter().filter(|&(n, k)| k >= 16 || c.world.nationalteams.iter().any(|x| x.nat == n)).map(|x| x.0).filter(|&n| !crate::nationen::name(&c.world, n).is_empty()).collect();
    let punkte: std::collections::HashMap<u16, f32> = nats.iter().map(|&n| (n, c.world.fifa_punkte(n))).collect();
    nats.sort_by(|a, b| punkte[b].partial_cmp(&punkte[a]).unwrap());
    // the world rank before filtering, then grouped by confederation and sorted by strength or name (Fabian 23.11.)
    let weltrang: std::collections::HashMap<u16, usize> = nats.iter().enumerate().map(|(i, &n)| (n, i + 1)).collect();
    const VERBAENDE: [&str; 6] = ["UEFA", "CONMEBOL", "CONCACAF", "CAF", "AFC", "OFC"]; // roh
    ui.card_titled(l, t("ui.nt.rangliste"), Theme::gold());
    let alle = t("ui.wort.alle");
    let vt: Vec<&str> = std::iter::once(alle).chain(VERBAENDE.iter().copied()).collect();
    if let Some(i) = ui.segment(Rect::new(l.x + 8.0, l.y + 44.0, l.w - 16.0, 28.0), &vt, v.nt_verband) { v.nt_verband = i; v.nt_scroll = 0; }
    if let Some(i) = ui.segment(Rect::new(l.x + 8.0, l.y + 78.0, l.w - 16.0, 28.0), &[t("ui.nt.sort_staerke"), t("ui.nt.sort_name")], v.nt_sort) { v.nt_sort = i; }
    if v.nt_verband > 0 { nats.retain(|&n| crate::nationalteams::verband(n) == VERBAENDE[v.nt_verband - 1]); }
    if v.nt_sort == 1 { nats.sort_by_key(|&n| crate::nationen::name(&c.world, n)); }
    let l = Rect::new(l.x, l.y + 68.0, l.w, l.h - 68.0);
    if nats.is_empty() { ui.paragraph(t("ui.nt.noch_keine"), l.x + 14.0, l.y + 60.0, 14.0, l.w - 28.0, Theme::text2()); return; }
    if nats.is_empty() { return; }
    let wahl = v.nt_wahl.filter(|n| nats.contains(n)).unwrap_or_else(|| { let e = c.world.land_nat(c.league()); if nats.contains(&e) { e } else { nats[0] } });
    let platz = ((l.h - 60.0) / 26.0).max(1.0) as usize;
    if l.contains(ui.mouse) && ui.wheel != 0.0 { v.nt_scroll = (v.nt_scroll as i32 - ui.wheel.signum() as i32 * 3).clamp(0, nats.len().saturating_sub(platz) as i32) as usize; }
    for (i, &n) in nats.iter().enumerate().skip(v.nt_scroll).take(platz) {
        let rr = Rect::new(l.x + 6.0, l.y + 48.0 + (i - v.nt_scroll) as f32 * 26.0, l.w - 12.0, 24.0);
        if n == wahl { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, alpha(Theme::akzent(), 0.16)); } else if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, Theme::erhoehung()); }
        ui.text(&format!("{}.", weltrang[&n]), rr.x + 6.0, rr.y + 17.0, 13.0, Theme::gedaempft());
        ui.text(&ui.fit(&crate::nationen::name_anzeige(&c.world, n), 14.0, rr.w * 0.6), rr.x + 44.0, rr.y + 17.0, 14.0, Theme::text());
        ui.text_right(&format!("{:.0}", punkte[&n]), rr.x + rr.w - 8.0, rr.y + 17.0, 13.0, Theme::text2());
        if ui.hit(rr) { v.nt_wahl = Some(n); }
    }
    // the hero in a colour of the nation's own (no flag data yet: fixed from its number)
    ui.card(r);
    let farbe = macroquad::color::hsl_to_rgb((wahl as f32 * 0.137).fract(), 0.55, 0.32);
    let hero = Rect::new(r.x, r.y, r.w, 170.0);
    gradient_h(hero.x, hero.y, hero.w, hero.h, farbe, alpha(farbe, 0.7));
    let weiss = hex(0xffffff);
    let name = crate::nationen::name_anzeige(&c.world, wahl);
    let rang = weltrang[&wahl];
    ui.title(&ui.fit(&name, 36.0, hero.w - 40.0), hero.x + 24.0, hero.y + 62.0, 36.0, weiss);
    let team = c.world.nationalteams.iter().find(|x| x.nat == wahl).cloned().unwrap_or_default();
    ui.text(&crate::tf!("ui.nt.held_zeile", crate::nationalteams::verband(wahl), rang, format!("{:.0}", c.world.fifa_punkte(wahl)), stufe_wort(c.world.nat_staerke(wahl))), hero.x + 24.0, hero.y + 92.0, 17.0, alpha(weiss, 0.9));
    let titel = c.world.turniere.iter().filter(|x| x.sieger == wahl).count();
    let chips = [crate::tf!("ui.nt.chip_bilanz", team.spiele, team.siege, team.remis, team.spiele.saturating_sub(team.siege + team.remis)), crate::tf!("ui.nt.chip_titel", titel), crate::tf!("ui.nt.chip_kader", team.kader.len())];
    let mut cx = hero.x + 24.0;
    for ch in chips { let w = ui.measure(&ch, 14.0) + 20.0; rrect(cx, hero.y + 114.0, w, 26.0, 13.0, alpha(BLACK, 0.35)); ui.text(&ch, cx + 10.0, hero.y + 132.0, 14.0, weiss); cx += w + 8.0; }
    let sp = r.w * 0.42;
    let links = Rect::new(r.x + 16.0, hero.y + hero.h + 14.0, sp - 24.0, r.h - hero.h - 24.0);
    let rechts = Rect::new(r.x + sp + 8.0, hero.y + hero.h + 14.0, r.w - sp - 22.0, r.h - hero.h - 24.0);
    // the coach and what he looks for (Teil 2.2)
    let tr = crate::nationalteams::trainer(&c.world, wahl);
    let mut y = links.y;
    ui.text_semi(&t("ui.verein.trainer").to_uppercase(), links.x, y + 12.0, 13.0, Theme::akzent());
    ui.text_semi(if tr.name.is_empty() { t("ui.nt.trainer_unbekannt") } else { &tr.name }, links.x, y + 36.0, 16.0, Theme::text());
    y += 50.0;
    for (k, w) in [("treue", tr.treue), ("form", tr.form), ("spielzeit", tr.spielzeit), ("jugend", tr.jugend), ("liga", tr.liga), ("disziplin", tr.disziplin)] { // roh
        ui.text(t(&format!("ui.nt.phil.{k}")), links.x, y + 12.0, 13.0, Theme::text2());
        ui.bar(links.x + links.w * 0.45, y + 4.0, links.w * 0.55, 8.0, w, Theme::akzent());
        y += 22.0;
    }
    y += 12.0;
    ui.text_semi(&t("ui.nt.geschichte").to_uppercase(), links.x, y + 12.0, 13.0, Theme::akzent());
    y += 30.0;
    let erfolge: Vec<_> = c.world.turniere.iter().rev().filter(|x| x.sieger == wahl || x.zweiter == wahl).collect();
    if erfolge.is_empty() { ui.text(t("ui.cup.keine"), links.x, y + 6.0, 14.0, Theme::gedaempft()); }
    for x in erfolge { if y > links.y + links.h - 20.0 { break; } ui.text(&ui.fit(&crate::tf!(if x.sieger == wahl { "ui.nt.erfolg_sieg" } else { "ui.nt.erfolg_finale" }, x.jahr, x.name.clone()), 14.0, links.w), links.x, y + 6.0, 14.0, Theme::text()); y += 20.0; }
    // the squad by lines, own players marked; a click opens the profile
    ui.text_semi(&t("ui.nt.aktueller_kader").to_uppercase(), rechts.x, rechts.y + 12.0, 13.0, Theme::akzent());
    if let Some(i) = ui.segment(Rect::new(rechts.x + rechts.w - 200.0, rechts.y - 4.0, 200.0, 26.0), &[t("ui.nt.a_team"), t("ui.nt.u21")], v.nt_u21) { v.nt_u21 = i; }
    let mut kader = if v.nt_u21 == 1 { team.u21.clone() } else { team.kader.clone() };
    kader.sort_by(|a, b| (part_of(&c.world.players[*a].pos), -(c.world.players[*a].ovr as i32)).cmp(&(part_of(&c.world.players[*b].pos), -(c.world.players[*b].ovr as i32))));
    if kader.is_empty() { ui.paragraph(t("ui.nt.kein_kader"), rechts.x, rechts.y + 40.0, 14.0, rechts.w, Theme::text2()); }
    // column heads, and a gap with the line's name between keepers, defence, midfield and attack
    if !kader.is_empty() {
        ui.text(t("ui.wort.gross_name"), rechts.x + 52.0, rechts.y + 40.0, 12.0, Theme::gedaempft());
        ui.text(t("ui.wort.gross_verein"), rechts.x + rechts.w * 0.52, rechts.y + 40.0, 12.0, Theme::gedaempft());
        ui.text_right(t("ui.nt.spiele_kurz"), rechts.x + rechts.w - 44.0, rechts.y + 40.0, 12.0, Theme::gedaempft());
    }
    let mut yy = rechts.y + 48.0;
    let mut klick = None;
    let mut teil_vor = usize::MAX;
    for p in kader {
        if yy > rechts.y + rechts.h - 24.0 { break; }
        let pl = &c.world.players[p];
        let teil = part_of(&pl.pos) as usize;
        if teil != teil_vor { if teil_vor != usize::MAX { yy += 6.0; } draw_rectangle(rechts.x, yy, rechts.w, 1.0, Theme::kontur()); yy += 4.0; teil_vor = teil; }
        let rr = Rect::new(rechts.x, yy, rechts.w, 24.0);
        if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
        ui.pos_tag(rr.x + 4.0, rr.y + 12.0, pos_de(&pl.pos));
        let eigen = pl.team == c.own as i32;
        ui.text(&ui.fit(&pl.name, 14.0, rr.w * 0.42), rr.x + 52.0, rr.y + 17.0, 14.0, if eigen { Theme::gold() } else { Theme::text() });
        let club = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else { String::new() };
        ui.text(&ui.fit(&club, 13.0, rr.w * 0.3), rr.x + rr.w * 0.52, rr.y + 17.0, 13.0, Theme::text2());
        ui.text_right(&format!("{}", pl.profil.laenderspiele), rr.x + rr.w - 44.0, rr.y + 17.0, 13.0, Theme::text2());
        ui.pill(rr.x + rr.w - 18.0, rr.y + 12.0, pl.ovr, 30.0);
        if ui.hit(rr) { klick = Some(p); }
        yy += 26.0;
    }
    if let Some(p) = klick { v.modal = Modal::Player(p); }
}

/// The Nations League (Fabian 24.11.): every group of the chosen league with its table and the results by matchday;
/// nations with our players marked, their squad of the last window as faces.
fn nl_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let r = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    if c.world.nl.is_empty() { ui.card_titled(r, t("ui.nl.seite"), Theme::akzent()); ui.text(t("ui.nl.noch_nicht"), r.x + 16.0, r.y + 64.0, 15.0, Theme::text2()); return; }
    let ligen = ["A", "B", "C", "D"];
    let n_ligen = c.world.nl.iter().map(|g| g.liga as usize + 1).max().unwrap_or(1);
    let refs: Vec<String> = (0..n_ligen).map(|k| crate::tf!("ui.nl.liga", ligen[k])).collect();
    let rr: Vec<&str> = refs.iter().map(|x| x.as_str()).collect();
    let wahl = v.nt_verband.min(n_ligen - 1);
    if let Some(k) = ui.segment(Rect::new(r.x, r.y, 120.0 * n_ligen as f32, 34.0), &rr, wahl) { v.nt_verband = k; }
    let eigene: Vec<u16> = c.squad().into_iter().map(|p| c.world.players[p].nat).collect();
    let gruppen: Vec<crate::nationalteams::NlGruppe> = c.world.nl.iter().filter(|g| g.liga as usize == wahl).cloned().collect();
    let sp = 2usize;
    let gw = (r.w - 12.0) / sp as f32;
    let gh = ((r.h - 50.0) / gruppen.len().div_ceil(sp).max(1) as f32 - 12.0).max(150.0);
    for (k, g) in gruppen.iter().enumerate() {
        let gr = Rect::new(r.x + (k % sp) as f32 * (gw + 12.0), r.y + 46.0 + (k / sp) as f32 * (gh + 12.0), gw, gh);
        if gr.y > r.y + r.h { break; }
        ui.card_titled(gr, &crate::tf!("ui.nl.gruppe", ligen[wahl], k + 1), Theme::akzent());
        let tab = c.world.nl_tabelle(g);
        for (i, row) in tab.iter().enumerate() {
            let y = gr.y + 64.0 + i as f32 * 24.0;
            if eigene.contains(&row.0) { rrect(gr.x + 8.0, y - 17.0, gr.w * 0.55, 22.0, 5.0, alpha(Theme::akzent(), 0.16)); }
            ui.text(&format!("{}. {}", i + 1, crate::nationen::name(&c.world, row.0)), gr.x + 16.0, y, 14.0, Theme::text());
            ui.text_right(&format!("{}  {}:{}  {}", row.1, row.5, row.6, row.7), gr.x + gr.w * 0.55, y, 14.0, Theme::text2());
        }
        // the results, newest matchday first
        let mut gespielt: Vec<&crate::nationalteams::NlSpiel> = g.spiele.iter().filter(|x| x.hg >= 0).collect();
        gespielt.sort_by_key(|x| std::cmp::Reverse(x.runde));
        for (i, x) in gespielt.iter().take(((gh - 70.0) / 20.0) as usize).enumerate() {
            let y = gr.y + 64.0 + i as f32 * 20.0;
            let kurz = |n: u16| crate::nationen::name(&c.world, n).chars().take(12).collect::<String>();
            ui.text(&ui.fit(&format!("{}. {} {}:{} {}", x.runde + 1, kurz(x.h), x.hg, x.ag, kurz(x.a)), 12.0, gr.w * 0.42), gr.x + gr.w * 0.57, y, 12.0, Theme::text2());
        }
    }
}
