// Paket 5: the pages that were still missing (included into pages.rs, so they use its building blocks).
// Each page shows data that the career already holds; texts in plain German.

/// Day-step durations in ms for the diagnosis page (Paket 6).
pub static TAGESZEIT: std::sync::Mutex<Vec<f32>> = std::sync::Mutex::new(Vec::new());

fn zeilen(ui: &mut Ui, r: Rect, items: &[(String, String)], start: f32) {
    for (k, (a, b)) in items.iter().enumerate() {
        let yy = r.y + start + k as f32 * 28.0;
        if yy > r.y + r.h - 12.0 { break; }
        ui.text(&ui.fit(a, 16.0, r.w * 0.6), r.x + 14.0, yy, 16.0, Theme::text());
        ui.text_right(b, r.x + r.w - 14.0, yy, 16.0, Theme::text2());
    }
}
fn links_rechts() -> (Rect, Rect) { (Rect::new(X0 + 4.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0), Rect::new(X0 + cbreite() * 0.5 + 10.0, Y0, cbreite() * 0.5 - 6.0, hoehe() - Y0 - 16.0)) }

fn ziele(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.seiten.saisonziel_des_praesidiums"), Theme::gold());
    ui.title(&c.club().goal_kind.clone(), l.x + 14.0, l.y + 80.0, 34.0, Theme::text());
    let rank = c.world.rank_of(c.own);
    ui.text(&crate::tf!("ui.pages.mindestens_platz_aktuell_platz", c.club().goal_pos, rank), l.x + 14.0, l.y + 112.0, 18.0, if rank <= c.club().goal_pos { Theme::gut() } else { Theme::achtung() });
    ui.paragraph(crate::inhalt::t("ui.seiten.das_ziel_richtet_sich_nach"), l.x + 14.0, l.y + 150.0, 15.0, l.w - 28.0, Theme::text2());
    if !c.mensch.ziel_angenommen {
        if ui.button_gold(Rect::new(l.x + 14.0, l.y + 250.0, 220.0, 38.0), crate::inhalt::t("ui.seiten.ziel_annehmen")) { c.mensch.ziel_angenommen = true; c.grund("Präsidium", crate::inhalt::t("zentrale.k.ziel"), 0.0, crate::inhalt::t("ui.seiten.ziel_angenommen")); v.toast(crate::inhalt::t("ui.seiten.ziel_angenommen"), true); } // roh
    } else { ui.text(crate::inhalt::t("ui.seiten.du_hast_das_ziel_angenommen"), l.x + 14.0, l.y + 270.0, 16.0, Theme::gut()); }
    ui.text(&crate::tf!("ui.seiten.vision_des_praesidiums", c.vision()), l.x + 14.0, l.y + 320.0, 16.0, Theme::text2());
    // milestones below the promises: seal when reached, otherwise progress (meilensteine.toml)
    let hm = (r.h * 0.55).min(420.0);
    let m = Rect::new(r.x, r.y + r.h - hm, r.w, hm);
    let r = Rect::new(r.x, r.y, r.w, r.h - hm - 12.0);
    ui.card_titled(m, crate::inhalt::t("ui.meilenstein.titel"), Theme::gold());
    for (k, (id, zaehlt, ziel)) in crate::meilensteine::liste().iter().enumerate() {
        let (sp, zeile) = (k % 2, k / 2);
        let bw = (m.w - 42.0) / 2.0;
        let (x, y) = (m.x + 14.0 + sp as f32 * (bw + 14.0), m.y + 58.0 + zeile as f32 * ((m.h - 70.0) / 5.0));
        let erreicht = c.meilensteine.iter().any(|x| &x.0 == id);
        let stand = c.meilenstein_stand(zaehlt).min(*ziel);
        draw_circle(x + 12.0, y - 5.0, 11.0, if erreicht { Theme::gold() } else { alpha(Theme::gedaempft(), 0.3) });
        if erreicht { ui.text_center("✓", x + 12.0, y + 1.0, 14.0, hex(0x1b1f27)); }
        ui.text_semi(crate::inhalt::t(&format!("ui.meilenstein.{id}")), x + 30.0, y, 15.0, if erreicht { Theme::text() } else { Theme::text2() });
        ui.bar(x + 30.0, y + 8.0, bw - 110.0, 6.0, stand / ziel.max(1.0), if erreicht { Theme::gold() } else { Theme::akzent() });
        ui.text_right(&crate::tf!("ui.meilenstein.fortschritt", format!("{stand:.0}"), format!("{ziel:.0}")), x + bw, y + 14.0, 12.0, Theme::gedaempft());
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.oeffentliche_zusagen"), Theme::akzent());
    let z: Vec<(String, String)> = c.me.zusagen.iter().map(|z| (if z.0 == crate::id("saisonziel") { crate::inhalt::t("ui.seiten.saisonziel_erreichen").to_string() } else { crate::tf!("ui.seiten.bleibt", z.1.map(|p| c.world.players[p].name.clone()).unwrap_or_default()) }, [crate::id("offen"), crate::id("gehalten"), crate::id("gebrochen")][z.4.min(2) as usize].to_string())).collect();
    if z.is_empty() { ui.text(crate::inhalt::t("ui.seiten.keine_zusagen_gegeben_in_der"), r.x + 14.0, r.y + 60.0, 16.0, Theme::text2()); }
    zeilen(ui, r, &z, 60.0);
}

fn manager_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    c.manager_an();
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("uebersicht.faehigkeiten"), Theme::akzent());
    for i in 0..7 {
        let yy = l.y + 64.0 + i as f32 * 40.0;
        ui.text(crate::rest3::werte_de_t()[i], l.x + 14.0, yy, 17.0, Theme::text());
        ui.bar(l.x + 220.0, yy - 10.0, l.w - 300.0, 10.0, c.mgr_wert(i) / 100.0, Theme::akzent());
        ui.text_right(&format!("{:.0}", c.mgr_wert(i)), l.x + l.w - 14.0, yy, 17.0, Theme::text2());
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.zustand_und_stil"), Theme::gold());
    for (i, (n, w)) in [(crate::inhalt::t("ui.seiten.energie"), c.mgr.energie), (crate::inhalt::t("ui.seiten.stress"), c.mgr.stress), (crate::inhalt::t("manager.wert.motivation"), c.mgr.motivation)].iter().enumerate() {
        let yy = r.y + 64.0 + i as f32 * 40.0;
        ui.text(n, r.x + 14.0, yy, 17.0, Theme::text());
        ui.bar(r.x + 160.0, yy - 10.0, r.w - 240.0, 10.0, w / 100.0, if (i == 1) == (*w > 60.0) { Theme::problem() } else { Theme::gut() });
        ui.text_right(&format!("{:.0}", w), r.x + r.w - 14.0, yy, 17.0, Theme::text2());
    }
    ui.text(&crate::tf!("gm.mg.hero.stil", c.stil().map(|i| crate::rest3::stile_de_t()[i]).unwrap_or(crate::inhalt::t("ui.seiten.noch_nicht_erkennbar"))), r.x + 14.0, r.y + 200.0, 18.0, Theme::text());
    ui.paragraph(crate::inhalt::t("ui.seiten.ab_stress_70_oder_energie"), r.x + 14.0, r.y + 236.0, 15.0, r.w - 28.0, Theme::text2());
    if ui.button(Rect::new(r.x + 14.0, r.y + 300.0, 200.0, 36.0), crate::inhalt::t("ui.seiten.freizeit_nehmen"), false) { match c.freizeit() { Ok(()) => v.toast(crate::inhalt::t("ui.seiten.durchatmen"), true), Err(e) => v.toast(&e, false) } }
}

/// Vereinsgeschichte (Teil 3, Rekorde und Geschichte): the coach's career and honours, the club's trophy cabinet, records
/// from the own matches and a hall of fame from every player's career stations at this club.
fn geschichte(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let (bw, bh) = ((cbreite() - 12.0) / 2.0, (hoehe() - Y0 - 28.0) / 2.0);
    let felder = [Rect::new(X0 + 4.0, Y0, bw, bh), Rect::new(X0 + 16.0 + bw, Y0, bw, bh), Rect::new(X0 + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(X0 + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
    // the coach's career
    let l = felder[0];
    ui.card_titled(l, t("ui.seiten.laufbahn_und_ehrungen"), Theme::gold());
    let h: Vec<(String, String)> = c.history.iter().rev().map(|x| (x.clone(), String::new())).collect();
    if h.is_empty() { ui.text(t("ui.seiten.die_erste_saison_laeuft_noch"), l.x + 14.0, l.y + 60.0, 16.0, Theme::text2()); }
    zeilen(ui, l, &h, 60.0);
    // trophy cabinet: one cup per title with competition and season
    let r = felder[1];
    ui.card_titled(r, t("ui.geschichte.vitrine"), Theme::akzent());
    let titel: Vec<&crate::cups::Title> = c.world.titles.iter().filter(|x| x.club == c.own).collect();
    if titel.is_empty() { ui.paragraph(t("ui.seiten.noch_kein_titel_in_deiner"), r.x + 14.0, r.y + 60.0, 16.0, r.w - 28.0, Theme::text2()); }
    let je_reihe = ((r.w - 28.0) / 110.0).max(1.0) as usize;
    for (k, x) in titel.iter().rev().enumerate() {
        let (cx, cy) = (r.x + 14.0 + 55.0 + (k % je_reihe) as f32 * 110.0, r.y + 84.0 + (k / je_reihe) as f32 * 96.0);
        if cy + 50.0 > r.y + r.h { break; }
        // a simple cup: bowl, handles, stem and base in gold
        draw_circle(cx, cy, 16.0, Theme::gold());
        draw_rectangle(cx - 16.0, cy - 16.0, 32.0, 16.0, Theme::gold());
        draw_circle_lines(cx - 18.0, cy - 6.0, 6.0, 2.5, Theme::gold());
        draw_circle_lines(cx + 18.0, cy - 6.0, 6.0, 2.5, Theme::gold());
        draw_rectangle(cx - 3.0, cy + 14.0, 6.0, 10.0, Theme::gold());
        draw_rectangle(cx - 12.0, cy + 23.0, 24.0, 5.0, Theme::gold());
        ui.text_center(&ui.fit(&x.comp, 12.0, 104.0), cx, cy + 44.0, 12.0, Theme::text());
        ui.text_center(&date::season_label(x.season), cx, cy + 58.0, 11.0, Theme::gedaempft());
    }
    // records from the own matches (all seasons with this club in PES M)
    let r = felder[2];
    ui.card_titled(r, t("ui.geschichte.rekorde"), hex(0xa78bfa));
    let sp = &c.spiele;
    let name = |x: &crate::spielgefuehl::SpielZeile| format!("{}:{} {} {} · {}", x.tore, x.gegentore, if x.heim { t("ui.geschichte.gegen") } else { t("ui.geschichte.bei") }, c.world.clubs[x.gegner].name, date::season_label(x.season));
    let sieg = sp.iter().filter(|x| x.tore > x.gegentore).max_by_key(|x| (x.tore as i32 - x.gegentore as i32, x.tore));
    let pleite = sp.iter().filter(|x| x.tore < x.gegentore).max_by_key(|x| (x.gegentore as i32 - x.tore as i32, x.gegentore));
    let torreich = sp.iter().max_by_key(|x| x.tore as u32 + x.gegentore as u32);
    let mut serie = (0usize, 0usize);
    for x in sp { if x.tore >= x.gegentore { serie.0 += 1; serie.1 = serie.1.max(serie.0); } else { serie.0 = 0; } }
    let z = vec![
        (t("ui.geschichte.hoechster_sieg").to_string(), sieg.map(name).unwrap_or_else(|| "–".into())),
        (t("ui.geschichte.hoechste_niederlage").to_string(), pleite.map(name).unwrap_or_else(|| "–".into())),
        (t("ui.geschichte.torreichstes").to_string(), torreich.map(name).unwrap_or_else(|| "–".into())),
        (t("ui.saison.laengste_ungeschlagen").to_string(), if sp.is_empty() { "–".into() } else { serie.1.to_string() }),
        (t("ui.geschichte.spiele").to_string(), sp.len().to_string()),
    ];
    for (j, (a, b)) in z.iter().enumerate() {
        let y = r.y + 64.0 + j as f32 * 30.0;
        ui.text(a, r.x + 14.0, y, 15.0, Theme::text2());
        ui.text_right(&ui.fit(b, 15.0, r.w * 0.6), r.x + r.w - 14.0, y, 15.0, Theme::text());
    }
    // hall of fame: matches and goals for this club over every career station (and the running season of its squad)
    let r = felder[3];
    ui.card_titled(r, t("ui.geschichte.hall_of_fame"), Theme::gut());
    let club = c.club().name.clone();
    let mut ruhm: Vec<(usize, u32, u32)> = c.world.players.iter().enumerate().filter_map(|(i, pl)| {
        let (mut a, mut g) = pl.profil.laufbahn.iter().filter(|x| x.1 == club).fold((0u32, 0u32), |s, x| (s.0 + x.2 as u32, s.1 + x.3 as u32));
        if pl.team == c.own as i32 { a += pl.apps as u32; g += pl.goals as u32; }
        (a > 0).then_some((i, a, g))
    }).collect();
    ruhm.sort_by_key(|x| (std::cmp::Reverse(x.1), std::cmp::Reverse(x.2)));
    player_row_head(ui, r.x, r.y + 58.0, &[(t("ui.geschichte.spieler"), 14.0), (t("ui.geschichte.spiele"), r.w * 0.66), (t("ui.saison.tore"), r.w * 0.84)]);
    let platz = ((r.h - 80.0) / 26.0).max(1.0) as usize;
    for (j, (p, a, g)) in ruhm.iter().take(platz).enumerate() {
        let y = r.y + 84.0 + j as f32 * 26.0;
        let pl = &c.world.players[*p];
        ui.player_photo(r.x + 26.0, y - 5.0, 10.0, pl.id, &pl.name);
        let n = if pl.team == c.own as i32 { pl.name.clone() } else { crate::tf!("ui.geschichte.ehemalig", pl.name) };
        ui.text(&ui.fit(&n, 15.0, r.w * 0.55), r.x + 44.0, y, 15.0, Theme::text());
        ui.text(&a.to_string(), r.x + r.w * 0.66, y, 15.0, Theme::text2());
        ui.text(&g.to_string(), r.x + r.w * 0.84, y, 15.0, Theme::text2());
    }
}

fn tagebuch(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(area, crate::inhalt::t("ui.seiten.tagebuch_jede_wirkung_mit_ihrem"), Theme::akzent());
    let vis = ((area.h - 70.0) / 28.0) as usize;
    let n = c.mensch.gruende.len();
    let start = scroll_rows(ui, v, area, n, vis);
    for (k, g) in c.mensch.gruende.iter().rev().skip(start).take(vis).enumerate() {
        let yy = area.y + 64.0 + k as f32 * 28.0;
        ui.text(&date::short(g.abs / date::SEASON_DAYS, g.abs % date::SEASON_DAYS), area.x + 14.0, yy, 15.0, Theme::gedaempft());
        ui.text(&ui.fit(&crate::tf!("profil.kurz_text", crate::inhalt::anzeige(&g.bezug), g.was), 15.0, 300.0), area.x + 110.0, yy, 15.0, Theme::text());
        ui.text(&ui.fit(&g.grund, 15.0, area.w - 560.0), area.x + 430.0, yy, 15.0, Theme::text2());
        if g.wert != 0.0 { ui.text_right(&format!("{:+.1}", g.wert), area.x + area.w - 14.0, yy, 15.0, if g.wert > 0.0 { Theme::gut() } else { Theme::problem() }); }
    }
}

fn kaderplanung(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.seiten.kader_nach_mannschaftsteilen"), Theme::akzent());
    let sq = c.squad();
    let mut z = vec![];
    for (t, n) in [crate::inhalt::t("gb2.sp.teil.tor"), crate::inhalt::t("achse.Abwehr"), crate::inhalt::t("gb2.vgl.mittel"), crate::inhalt::t("bericht.achse.angriff")].iter().enumerate() {
        let ps: Vec<usize> = sq.iter().copied().filter(|&p| part_of(&c.world.players[p].pos) == t).collect();
        let st = if ps.is_empty() { 0.0 } else { ps.iter().map(|&p| c.world.players[p].ovr).sum::<f32>() / ps.len() as f32 };
        let alt = if ps.is_empty() { 0.0 } else { ps.iter().map(|&p| c.world.players[p].age as f32).sum::<f32>() / ps.len() as f32 };
        let aus = ps.iter().filter(|&&p| c.world.players[p].contract_end <= c.world.season).count();
        z.push((crate::tf!("ui.seiten.spieler", n, ps.len()), crate::tf!("ui.seiten.staerke_alter_vertraege_enden", stufe_wort(st), format!("{:.0}", alt), aus)));
    }
    zeilen(ui, l, &z, 64.0);
    ui.card_titled(r, crate::inhalt::t("ui.seiten.was_fehlt"), Theme::gold());
    let mut w: Vec<(String, String)> = vec![];
    for (k, n) in [(30, crate::inhalt::t("ui.seiten.ueber_30")), (23, crate::inhalt::t("ui.seiten.bis_23"))] { let a = sq.iter().filter(|&&p| if k == 30 { c.world.players[p].age > 30 } else { c.world.players[p].age <= 23 }).count(); w.push((crate::tf!("r.ad.taktikkraft.spieler", n), a.to_string())); }
    let bereit = c.world.clubs[c.own].squad.iter().filter(|&&p| c.world.players[p].youth == 19).count();
    w.push((crate::id("U23-Spieler").into(), bereit.to_string()));
    w.push((crate::inhalt::t("zentrale.k.budget").into(), money(c.transfer_budget)));
    for l2 in c.taktikurteil().into_iter().take(3) { w.push((l2, String::new())); }
    zeilen(ui, r, &w, 64.0);
}
pub fn stufe_wort(o: f32) -> String { (if o >= 82.0 { crate::inhalt::t("stufe.staerke.5") } else if o >= 77.0 { crate::inhalt::t("stufe.staerke.4") } else if o >= 72.0 { crate::id("stark") } else if o >= 66.0 { crate::id("solide") } else { crate::inhalt::t("ui.seiten.ausbaufaehig") }).to_string() }

fn vergleich(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let sq = c.squad();
    let a = v.sel_player.filter(|&p| p < c.world.players.len()).unwrap_or(sq[0]);
    let b = sq.iter().copied().filter(|&p| p != a && c.world.players[p].pos == c.world.players[a].pos).max_by(|x, y| c.world.players[*x].ovr.partial_cmp(&c.world.players[*y].ovr).unwrap()).unwrap_or(sq[1.min(sq.len() - 1)]);
    let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    let (pa, pb) = (c.world.players[a].clone(), c.world.players[b].clone());
    ui.card_titled(area, &crate::tf!("spielbericht.gegen", pa.name, pb.name), Theme::akzent());
    ui.player_photo(area.x + area.w - 90.0, area.y + 18.0, 15.0, pa.id, &pa.name);
    ui.player_photo(area.x + area.w - 54.0, area.y + 18.0, 15.0, pb.id, &pb.name);
    if pa.attrs.len() != 25 || pb.attrs.len() != 25 { ui.text(crate::inhalt::t("ui.seiten.fuer_diese_spieler_sind_keine"), area.x + 14.0, area.y + 64.0, 16.0, Theme::text2()); return; }
    for i in 0..25 {
        let (col, row) = (i / 13, i % 13);
        let x = area.x + 14.0 + col as f32 * area.w / 2.0;
        let yy = area.y + 64.0 + row as f32 * 30.0;
        ui.text(crate::pesdb::attr_de_t()[i], x, yy, 15.0, Theme::text());
        let w = area.w / 2.0 - 260.0;
        ui.bar(x + 190.0, yy - 14.0, w, 6.0, pa.attrs[i] / 99.0, if pa.attrs[i] >= pb.attrs[i] { Theme::gut() } else { Theme::achtung() });
        ui.bar(x + 190.0, yy - 5.0, w, 6.0, pb.attrs[i] / 99.0, Theme::akzent());
        ui.text_right(&format!("{:.0} : {:.0}", pa.attrs[i], pb.attrs[i]), x + area.w / 2.0 - 30.0, yy, 15.0, Theme::text2());
    }
    if ui.button(Rect::new(area.x + area.w - 230.0, area.y + 8.0, 216.0, 30.0), crate::inhalt::t("ui.seiten.naechster_spieler"), false) { let i = sq.iter().position(|&p| p == a).map(|i| (i + 1) % sq.len()).unwrap_or(0); v.sel_player = Some(sq[i]); }
}

fn vorbereitung(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    match c.gegnerbericht() {
        Some((gn, z)) => { ui.card_titled(l, &crate::tf!("ui.seiten.naechster_gegner", gn), Theme::akzent()); for (k, x) in z.iter().enumerate() { ui.paragraph(x, l.x + 14.0, l.y + 64.0 + k as f32 * 44.0, 15.0, l.w - 28.0, Theme::text()); } }
        None => { ui.card_titled(l, crate::inhalt::t("ui.seiten.kein_spiel_in_sicht"), Theme::akzent()); }
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.startelf_und_zustand"), Theme::gold());
    let z: Vec<(String, String)> = c.current_xi().iter().map(|(s, p, _)| { let pl = &c.world.players[*p]; (format!("{} {}", pos_de(s), pl.name), crate::tf!("ui.seiten.fitness", if crate::mensch::pfeil(pl.form) >= 3 { crate::inhalt::t("gs.kontext.form.gut") } else if crate::mensch::pfeil(pl.form) <= 1 { crate::inhalt::t("ui.seiten.ausser_form") } else { crate::inhalt::t("ui.wort.normal") }, format!("{:.0}", pl.fit))) }).collect();
    zeilen(ui, r, &z, 64.0);
}

/// Verlauf (Teil 3, „Verlauf statt Zahl“): four curves of the season, each with one sentence on what it says.
fn verlauf_seite(ui: &mut Ui, c: &Career) {
    let v = c.verlauf_saison();
    let (bw, bh) = ((cbreite() - 12.0) / 2.0, (hoehe() - Y0 - 28.0) / 2.0);
    let felder = [Rect::new(X0 + 4.0, Y0, bw, bh), Rect::new(X0 + 16.0 + bw, Y0, bw, bh), Rect::new(X0 + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(X0 + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
    let titel = ["ui.verlauf.platz", "ui.verlauf.staerke", "ui.verlauf.stimmung", "ui.verlauf.kasse"];
    for (k, r) in felder.iter().enumerate() {
        ui.card_titled(*r, crate::inhalt::t(titel[k]), [Theme::akzent(), Theme::gold(), hex(0xa78bfa), Theme::gut()][k]);
        if v.len() < 2 { ui.text(crate::inhalt::t("ui.verlauf.zu_wenig"), r.x + 16.0, r.y + 64.0, 15.0, Theme::text2()); continue; }
        let (a, b) = (v.first().unwrap(), v.last().unwrap());
        let wochen = v.len() - 1;
        let satz = match k {
            0 => { let d = a.platz as i32 - b.platz as i32; if d > 0 { crate::tf!("ui.verlauf.platz.besser", d, wochen) } else if d < 0 { crate::tf!("ui.verlauf.platz.schlechter", -d, wochen) } else { crate::tf!("ui.verlauf.platz.gleich", wochen) } }
            1 => {
                let d = b.staerke - a.staerke;
                let z = format!("{:.1}", d.abs());
                let z = if crate::inhalt::sprache() == crate::inhalt::Sprache::De { z.replace('.', ",") } else { z };
                crate::tf!(if d >= 0.0 { "ui.verlauf.staerke.besser" } else { "ui.verlauf.staerke.schlechter" }, z, wochen)
            }
            2 => crate::tf!("ui.verlauf.stimmung.satz", format!("{:.0}", b.teamgeist), format!("{:.0}", b.moral)),
            _ => crate::tf!("ui.verlauf.kasse.satz", money(b.kasse - a.kasse), wochen),
        };
        ui.text(&ui.fit(&satz, 15.0, r.w - 32.0), r.x + 16.0, r.y + 60.0, 15.0, Theme::text());
        let kr = Rect::new(r.x + 12.0, r.y + 72.0, r.w - 24.0, r.h - 84.0);
        match k {
            0 => ui.kurve(kr, &v.iter().map(|p| p.platz as f32).collect::<Vec<_>>(), Theme::akzent(), true, &|x| format!("{x:.0}.")),
            1 => ui.kurve(kr, &v.iter().map(|p| p.staerke).collect::<Vec<_>>(), Theme::gold(), false, &|x| format!("{x:.0}")),
            2 => ui.kurve(kr, &v.iter().map(|p| p.teamgeist).collect::<Vec<_>>(), hex(0xa78bfa), false, &|x| format!("{x:.0}")),
            _ => ui.kurve(kr, &v.iter().map(|p| p.kasse as f32).collect::<Vec<_>>(), Theme::gut(), false, &|x| money(x as f64)),
        }
    }
}

/// Season (Teil 3, Mannschaft and Rekorde): home and away, streaks, the record against every opponent (derbies marked),
/// best and weakest own players by grade. Each card leads with one sentence.
fn saison_seite(ui: &mut Ui, c: &Career) {
    use crate::inhalt::t;
    let s = c.world.season;
    let diese: Vec<&crate::spielgefuehl::SpielZeile> = c.spiele.iter().filter(|x| x.season == s).collect();
    let (bw, bh) = ((cbreite() - 12.0) / 2.0, (hoehe() - Y0 - 28.0) / 2.0);
    let felder = [Rect::new(X0 + 4.0, Y0, bw, bh), Rect::new(X0 + 16.0 + bw, Y0, bw, bh), Rect::new(X0 + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(X0 + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
    let bilanz = |l: &[&crate::spielgefuehl::SpielZeile]| -> (usize, usize, usize, u32, u32) {
        (l.iter().filter(|x| x.tore > x.gegentore).count(), l.iter().filter(|x| x.tore == x.gegentore).count(), l.iter().filter(|x| x.tore < x.gegentore).count(),
         l.iter().map(|x| x.tore as u32).sum(), l.iter().map(|x| x.gegentore as u32).sum())
    };
    let ppg = |b: (usize, usize, usize, u32, u32)| { let n = b.0 + b.1 + b.2; if n == 0 { 0.0 } else { (3 * b.0 + b.1) as f32 / n as f32 } };
    let dez = |x: f32| { let z = format!("{x:.2}"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { z.replace('.', ",") } else { z } };
    // home and away
    let r = felder[0];
    ui.card_titled(r, t("ui.saison.heim_auswaerts"), Theme::akzent());
    let heim: Vec<_> = diese.iter().copied().filter(|x| x.heim).collect();
    let ausw: Vec<_> = diese.iter().copied().filter(|x| !x.heim).collect();
    if diese.is_empty() { ui.text(t("ui.saison.noch_kein_spiel"), r.x + 16.0, r.y + 64.0, 15.0, Theme::text2()); } else {
        ui.text(&ui.fit(&crate::tf!("ui.saison.heim_satz", dez(ppg(bilanz(&heim))), dez(ppg(bilanz(&ausw)))), 15.0, r.w - 32.0), r.x + 16.0, r.y + 60.0, 15.0, Theme::text());
        player_row_head(ui, r.x, r.y + 92.0, &[("", 16.0), (t("ui.saison.s"), r.w * 0.38), (t("ui.saison.u"), r.w * 0.48), (t("ui.saison.n"), r.w * 0.58), (t("ui.saison.tore"), r.w * 0.7), (t("ui.saison.punkte"), r.w * 0.86)]);
        for (k, (name, l)) in [(t("ui.saison.heim"), heim.clone()), (t("ui.saison.auswaerts"), ausw.clone()), (t("ui.saison.gesamt"), diese.clone())].iter().enumerate() {
            let y = r.y + 122.0 + k as f32 * 30.0;
            let b = bilanz(l);
            ui.text_semi(name, r.x + 16.0, y, 16.0, Theme::text());
            for (i, w) in [b.0.to_string(), b.1.to_string(), b.2.to_string(), format!("{}:{}", b.3, b.4), (3 * b.0 + b.1).to_string()].iter().enumerate() {
                ui.text(w, r.x + r.w * [0.38, 0.48, 0.58, 0.7, 0.86][i], y, 16.0, Theme::text2());
            }
        }
    }
    // streaks
    let r = felder[1];
    ui.card_titled(r, t("ui.saison.serien"), Theme::gold());
    let mut lauf = (0usize, 0usize, 0usize, 0usize); // current wins, current unbeaten, longest wins, longest unbeaten
    for x in &diese {
        if x.tore > x.gegentore { lauf.0 += 1; } else { lauf.0 = 0; }
        if x.tore >= x.gegentore { lauf.1 += 1; } else { lauf.1 = 0; }
        lauf.2 = lauf.2.max(lauf.0);
        lauf.3 = lauf.3.max(lauf.1);
    }
    let zu_null = diese.iter().filter(|x| x.gegentore == 0).count();
    let satz = if diese.is_empty() { t("ui.saison.noch_kein_spiel").to_string() } else if lauf.0 >= 2 { crate::tf!("ui.saison.serie_siege", lauf.0) } else if lauf.1 >= 2 { crate::tf!("ui.saison.serie_ungeschlagen", lauf.1) }
        else { let ohne = diese.iter().rev().take_while(|x| x.tore <= x.gegentore).count(); if ohne >= 2 { crate::tf!("ui.saison.serie_ohne_sieg", ohne) } else { t("ui.saison.serie_keine").to_string() } };
    ui.text(&ui.fit(&satz, 15.0, r.w - 32.0), r.x + 16.0, r.y + 60.0, 15.0, Theme::text());
    let z = vec![(t("ui.saison.laengste_siege").to_string(), lauf.2.to_string()), (t("ui.saison.laengste_ungeschlagen").to_string(), lauf.3.to_string()), (t("ui.saison.zu_null").to_string(), zu_null.to_string())];
    for (k, (a, b)) in z.iter().enumerate() {
        let y = r.y + 100.0 + k as f32 * 30.0;
        ui.text(a, r.x + 16.0, y, 16.0, Theme::text());
        ui.text_right(b, r.x + r.w - 16.0, y, 16.0, Theme::text2());
    }
    // record against every opponent (all seasons), derbies first marked
    let r = felder[2];
    ui.card_titled(r, t("ui.saison.bilanz_gegner"), hex(0xa78bfa));
    let mut gegner: Vec<(usize, Vec<&crate::spielgefuehl::SpielZeile>)> = vec![];
    for x in &c.spiele { match gegner.iter_mut().find(|g| g.0 == x.gegner) { Some(g) => g.1.push(x), None => gegner.push((x.gegner, vec![x])) } }
    gegner.sort_by_key(|g| (std::cmp::Reverse(g.1.iter().any(|x| x.derby)), std::cmp::Reverse(g.1.len())));
    if gegner.is_empty() { ui.text(t("ui.saison.noch_kein_spiel"), r.x + 16.0, r.y + 64.0, 15.0, Theme::text2()); } else {
        let liebling = gegner.iter().max_by_key(|g| bilanz(&g.1).0).map(|g| (c.world.clubs[g.0].name.clone(), bilanz(&g.1).0)).unwrap();
        ui.text(&ui.fit(&crate::tf!(if liebling.1 == 1 { "ui.saison.liebling_eins" } else { "ui.saison.liebling" }, liebling.0, liebling.1), 15.0, r.w - 32.0), r.x + 16.0, r.y + 60.0, 15.0, Theme::text());
        let platz = ((r.h - 92.0) / 26.0).max(1.0) as usize;
        for (k, (g, l)) in gegner.iter().take(platz).enumerate() {
            let y = r.y + 92.0 + k as f32 * 26.0;
            let b = bilanz(l);
            let cl = &c.world.clubs[*g];
            ui.club_crest(r.x + 26.0, y - 5.0, 9.0, cl.id, hex(cl.c1), &cl.short);
            let name = if l.iter().any(|x| x.derby) { crate::tf!("ui.saison.derby", cl.name) } else { cl.name.clone() };
            ui.text(&ui.fit(&name, 15.0, r.w * 0.5), r.x + 42.0, y, 15.0, Theme::text());
            ui.text_right(&crate::tf!(if l.len() == 1 { "ui.saison.bilanz_zeile_eins" } else { "ui.saison.bilanz_zeile" }, l.len(), b.0, b.1, b.2, format!("{}:{}", b.3, b.4)), r.x + r.w - 16.0, y, 14.0, Theme::text2());
        }
    }
    // best and weakest by grade (own players with at least three grades)
    let r = felder[3];
    ui.card_titled(r, t("ui.saison.beste_schwaechste"), Theme::gut());
    let mut noten: Vec<(usize, f32, usize)> = c.squad().into_iter().filter_map(|p| c.noten_verlauf.get(&c.world.players[p].id).filter(|v| v.len() >= 3).map(|v| (p, v.iter().sum::<f32>() / v.len() as f32, v.len()))).collect();
    noten.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    if noten.is_empty() { ui.text(t("ui.saison.zu_wenig_noten"), r.x + 16.0, r.y + 64.0, 15.0, Theme::text2()); } else {
        ui.text(&ui.fit(&crate::tf!("ui.saison.bester", c.world.players[noten[0].0].name, dez(noten[0].1)), 15.0, r.w - 32.0), r.x + 16.0, r.y + 60.0, 15.0, Theme::text());
        let n = noten.len();
        let zeigen: Vec<(usize, f32, usize)> = if n <= 6 { noten.clone() } else { noten.iter().take(3).chain(noten.iter().skip(n - 3)).copied().collect() };
        for (k, (p, avg, anz)) in zeigen.iter().enumerate() {
            let y = r.y + 92.0 + k as f32 * 28.0 + if n > 6 && k >= 3 { 10.0 } else { 0.0 };
            let pl = &c.world.players[*p];
            ui.player_photo(r.x + 28.0, y - 5.0, 10.0, pl.id, &pl.name);
            ui.text(&ui.fit(&pl.name, 15.0, r.w * 0.55), r.x + 46.0, y, 15.0, Theme::text());
            ui.text(&crate::tf!("ui.saison.spiele_kurz", anz), r.x + r.w * 0.68, y, 13.0, Theme::gedaempft());
            ui.note_pill(r.x + r.w - 36.0, y - 5.0, *avg);
        }
    }
}

fn analyse_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("ui.verlauf.reiter_spiel"), crate::inhalt::t("ui.verlauf.reiter_verlauf"), crate::inhalt::t("ui.saison.reiter")]);
    if v.tab == 1 { verlauf_seite(ui, c); return; }
    if v.tab == 2 { saison_seite(ui, c); return; }
    let (l, r) = links_rechts();
    let Some(res) = c.last_result.clone() else { ui.card_titled(l, crate::inhalt::t("menu.analyse.titel"), Theme::akzent()); ui.text(crate::inhalt::t("spielbericht.kein_spiel"), l.x + 14.0, l.y + 64.0, 16.0, Theme::text2()); return; };
    ui.card_titled(l, &format!("{} – {} {}:{}", c.world.clubs[res.h].name, c.world.clubs[res.a].name, res.hg, res.ag), Theme::akzent());
    ui.paragraph(&c.analyse(&res), l.x + 14.0, l.y + 64.0, 16.0, l.w - 28.0, Theme::text());
    ui.card_titled(r, crate::inhalt::t("bericht.noten"), Theme::gold());
    let mut n: Vec<(usize, f32)> = res.ratings.iter().copied().filter(|x| c.world.players[x.0].team == c.own as i32).collect();
    n.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let z: Vec<(String, String)> = n.iter().map(|(p, r2)| (c.world.players[*p].name.clone(), format!("{r2:.1}"))).collect();
    zeilen(ui, r, &z, 64.0);
}

fn entwicklung(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(area, crate::inhalt::t("ui.seiten.entwicklung_der_spieler"), Theme::akzent());
    let sq = c.squad();
    let vis = ((area.h - 70.0) / 28.0) as usize;
    let start = scroll_rows(ui, v, area, sq.len(), vis);
    for (k, &p) in sq.iter().skip(start).take(vis).enumerate() {
        let pl = &c.world.players[p];
        let yy = area.y + 64.0 + k as f32 * 28.0;
        let (a, b) = c.pot_range(p);
        let phase = ["", crate::inhalt::t("en3.phase.aufschwung"), crate::inhalt::t("en3.betreff.formtief"), crate::inhalt::t("en3.betreff.krise")][pl.ps.phase.min(3) as usize];
        ui.player_photo(area.x + 24.0, yy - 5.0, 10.0, pl.id, &pl.name);
        ui.text(&pl.name, area.x + 40.0, yy, 15.0, Theme::text());
        ui.text(&crate::tf!("ui.seiten.jahre_potenzial", pl.age, stufe_wort(pl.ovr), stufe_wort((a + b) / 2.0)), area.x + 260.0, yy, 15.0, Theme::text2());
        let extra = [phase.to_string(), pl.ps.umschulung.as_ref().map(|u| crate::tf!("ui.seiten.lernt", pos_de(&u.0))).unwrap_or_default(), if pl.ps.mentor.is_some() { crate::inhalt::t("ui.seiten.hat_einen_mentor").into() } else { String::new() }, if pl.ps.last >= 70.0 { crate::inhalt::t("r.ui.demo.ueberlastet").into() } else { String::new() }].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(", ");
        ui.text_right(&extra, area.x + area.w - 14.0, yy, 15.0, Theme::achtung());
    }
}

fn zeitung(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
    ui.card_titled(area, crate::inhalt::t("ui.seiten.die_zeitung"), Theme::akzent());
    let vis = ((area.h - 60.0) / 64.0) as usize;
    let start = scroll_rows(ui, v, area, c.feed.len(), vis);
    for (k, n) in c.feed.iter().skip(start).take(vis).enumerate() {
        let yy = area.y + 62.0 + k as f32 * 64.0;
        ui.text(&format!("{} · {}", n.paper, date::short(n.season, n.day)), area.x + 14.0, yy, 13.0, Theme::gedaempft());
        ui.text_semi(&ui.fit(&n.head, 19.0, area.w - 28.0), area.x + 14.0, yy + 22.0, 19.0, Theme::text());
        ui.text(&ui.fit(&n.text, 15.0, area.w - 28.0), area.x + 14.0, yy + 44.0, 15.0, Theme::text2());
    }
}

/// Liga in Zahlen (Teil 3): scorers, assists, grades and, per club, clean sheets and fair play (yellow cards of the squad),
/// each card led by its leader in one sentence.
/// A number with two decimals as the language writes it (German with a comma).
fn dez2(x: f32) -> String { let z = format!("{x:.2}"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { z.replace('.', ",") } else { z } }

fn ranglisten(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let l = c.league();
    let teams = c.world.leagues[l].teams.clone();
    let alle: Vec<usize> = teams.iter().flat_map(|&t| c.world.clubs[t].squad.clone()).collect();
    let (w3, h2) = ((cbreite() - 24.0) / 3.0, (hoehe() - Y0 - 28.0) / 2.0);
    let feld = |k: usize| Rect::new(X0 + 4.0 + (k % 3) as f32 * (w3 + 12.0), Y0 + (k / 3) as f32 * (h2 + 12.0), w3, h2);
    let kurz = |p: usize| c.world.clubs[c.world.players[p].team.max(0) as usize].short.clone();
    let liste = |ui: &mut Ui, k: usize, titel: &str, farbe: Color, z: Vec<(String, String)>, satz: String| {
        let r = feld(k);
        ui.card_titled(r, titel, farbe);
        ui.text(&ui.fit(&satz, 14.0, r.w - 28.0), r.x + 14.0, r.y + 58.0, 14.0, Theme::text());
        let platz = ((r.h - 84.0) / 24.0).max(1.0) as usize;
        for (j, (a, b)) in z.iter().take(platz).enumerate() {
            let y = r.y + 86.0 + j as f32 * 24.0;
            ui.text(&format!("{}.", j + 1), r.x + 14.0, y, 14.0, Theme::gedaempft());
            ui.text(&ui.fit(a, 14.0, r.w - 100.0), r.x + 40.0, y, 14.0, Theme::text());
            ui.text_right(b, r.x + r.w - 14.0, y, 14.0, Theme::text2());
        }
    };
    let leer = || t("ui.liga.noch_leer").to_string();
    // scorers and assists
    let mut tore = alle.clone();
    tore.sort_by_key(|&p| std::cmp::Reverse(c.world.players[p].goals));
    let satz = tore.first().filter(|&&p| c.world.players[p].goals > 0).map(|&p| crate::tf!("ui.liga.torjaeger_satz", c.world.players[p].name, c.world.players[p].goals)).unwrap_or_else(leer);
    liste(ui, 0, t("ui.seiten.torjaeger_der_liga"), Theme::gold(), tore.iter().map(|&p| (format!("{} ({})", c.world.players[p].name, kurz(p)), c.world.players[p].goals.to_string())).collect(), satz);
    let mut vorl = alle.clone();
    vorl.sort_by_key(|&p| std::cmp::Reverse(c.world.players[p].assists));
    let satz = vorl.first().filter(|&&p| c.world.players[p].assists > 0).map(|&p| crate::tf!("ui.liga.vorlagen_satz", c.world.players[p].name, c.world.players[p].assists)).unwrap_or_else(leer);
    liste(ui, 1, t("ui.liga.vorlagen"), Theme::akzent(), vorl.iter().map(|&p| (format!("{} ({})", c.world.players[p].name, kurz(p)), c.world.players[p].assists.to_string())).collect(), satz);
    // grades (at least five)
    let mut noten: Vec<usize> = alle.iter().copied().filter(|&p| c.world.players[p].rn >= 5).collect();
    noten.sort_by(|a, b| c.world.players[*b].avg_rating().partial_cmp(&c.world.players[*a].avg_rating()).unwrap());
    let satz = noten.first().map(|&p| crate::tf!("ui.liga.noten_satz", c.world.players[p].name, dez2(c.world.players[p].avg_rating()))).unwrap_or_else(|| t("ui.liga.noten_leer").to_string());
    liste(ui, 2, t("ui.seiten.beste_noten_der_liga"), hex(0xa78bfa), noten.iter().map(|&p| (format!("{} ({})", c.world.players[p].name, kurz(p)), dez2(c.world.players[p].avg_rating()))).collect(), satz);
    // clean sheets per club (played league matches without a goal against)
    let fx = &c.world.leagues[l].fixtures;
    let mut null: Vec<(usize, usize)> = teams.iter().map(|&t| (t, fx.iter().filter(|f| f.hg >= 0 && ((f.h == t && f.ag == 0) || (f.a == t && f.hg == 0))).count())).collect();
    null.sort_by_key(|x| std::cmp::Reverse(x.1));
    let satz = null.first().filter(|x| x.1 > 0).map(|x| crate::tf!("ui.liga.zu_null_satz", c.world.clubs[x.0].name, x.1)).unwrap_or_else(leer);
    liste(ui, 3, t("ui.liga.zu_null"), Theme::gut(), null.iter().map(|x| (c.world.clubs[x.0].name.clone(), x.1.to_string())).collect(), satz);
    // fair play: fewest yellow cards of the squad (red cards are counted in suspensions, not kept per player)
    let mut fair: Vec<(usize, u32)> = teams.iter().map(|&t| (t, c.world.clubs[t].squad.iter().map(|&p| c.world.players[p].yellow as u32).sum())).collect();
    fair.sort_by_key(|x| x.1);
    let satz = fair.first().map(|x| crate::tf!("ui.liga.fair_satz", c.world.clubs[x.0].name, x.1)).unwrap_or_else(leer);
    liste(ui, 4, t("ui.liga.fairness"), hex(0xf5d90a), fair.iter().map(|x| (c.world.clubs[x.0].name.clone(), x.1.to_string())).collect(), satz);
    // the own club among them: places in every table
    let r = feld(5);
    ui.card_titled(r, &crate::tf!("ui.liga.wir", c.club().name), Theme::verein());
    let pos = |v: &[usize], p: usize| v.iter().position(|&x| x == p).map(|i| i + 1);
    let best = |v: &[usize]| v.iter().copied().filter(|&p| c.world.players[p].team == c.own as i32).next();
    let z = vec![
        (t("ui.liga.bester_torschuetze").to_string(), best(&tore).map(|p| format!("{} · {}.", c.world.players[p].name, pos(&tore, p).unwrap_or(0))).unwrap_or_default()),
        (t("ui.liga.bester_vorbereiter").to_string(), best(&vorl).map(|p| format!("{} · {}.", c.world.players[p].name, pos(&vorl, p).unwrap_or(0))).unwrap_or_default()),
        (t("ui.liga.zu_null").to_string(), null.iter().position(|x| x.0 == c.own).map(|i| format!("{} · {}.", null[i].1, i + 1)).unwrap_or_default()),
        (t("ui.liga.fairness").to_string(), fair.iter().position(|x| x.0 == c.own).map(|i| format!("{} · {}.", fair[i].1, i + 1)).unwrap_or_default()),
    ];
    for (j, (a, b)) in z.iter().enumerate() {
        let y = r.y + 64.0 + j as f32 * 30.0;
        ui.text(a, r.x + 14.0, y, 15.0, Theme::text2());
        ui.text_right(&ui.fit(b, 15.0, r.w * 0.55), r.x + r.w - 14.0, y, 15.0, Theme::text());
    }
}

fn vereine(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, &c.world.leagues[c.league()].name.clone(), Theme::akzent());
    let ts = c.world.leagues[c.league()].teams.clone();
    // the map of the league below the list (geo.rs): click a dot to select the club
    let kh = (l.h * 0.45).max(240.0);
    let kr = Rect::new(l.x + 8.0, l.y + l.h - kh - 8.0, l.w - 16.0, kh);
    let (mut zoom, mut off) = (v.karte_zoom, v.karte_off);
    if let Some(k) = crate::geo::karte(ui, kr, &c.world, &ts, Some(v.sel_club.unwrap_or(c.own)), &mut zoom, &mut off) { v.sel_club = Some(k); }
    v.karte_zoom = zoom; v.karte_off = off;
    for (k, &t) in ts.iter().enumerate() {
        let yy = l.y + 56.0 + k as f32 * 26.0;
        if yy > kr.y - 6.0 { break; }
        let rr = Rect::new(l.x + 6.0, yy - 18.0, l.w - 12.0, 24.0);
        if v.sel_club == Some(t) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, alpha(Theme::akzent(), 0.15)); }
        ui.text(&c.world.clubs[t].name, l.x + 14.0, yy, 15.0, Theme::text());
        ui.text_right(&format!("{} · {}", stufe_wort(crate::mensch::kaderstaerke(&c.world, t)), c.world.clubs[t].manager), l.x + l.w - 14.0, yy, 14.0, Theme::text2());
        if ui.hit(rr) { v.sel_club = Some(t); }
    }
    let t = v.sel_club.unwrap_or(c.own);
    ui.card_titled(r, &crate::tf!("ui.seiten.kader", c.world.clubs[t].name), Theme::gold());
    let mut oben = if ui.stadion_bild(Rect::new(r.x + 8.0, r.y + 44.0, r.w - 16.0, 120.0), c.world.clubs[t].id) { 132.0 } else { 0.0 };
    // the coach from the coach bank: name, nationality, since when, tags and the bank's description of his style
    if let Some(tr) = c.world.clubs[t].trainer.clone() {
        let y0 = r.y + 56.0 + oben;
        let seit = match tr.seit.strip_prefix('S') { Some(s) => date::season_label(s.parse().unwrap_or(0)), None => tr.seit.clone() };
        ui.text_semi(&crate::tf!("ui.trainer.zeile", tr.name, tr.nat, crate::tf!("tb.seit", seit)), r.x + 14.0, y0, 15.0, Theme::text());
        let tags: Vec<&str> = tr.tags.iter().take(5).map(|x| crate::inhalt::t(&format!("tb.tag.{x}"))).filter(|s| !s.is_empty()).collect();
        ui.text(&tags.join(" · "), r.x + 14.0, y0 + 20.0, 13.0, Theme::akzent());
        let unten = if tr.stil.is_empty() { y0 + 28.0 } else { ui.paragraph(&tr.stil, r.x + 14.0, y0 + 40.0, 13.0, r.w - 28.0, Theme::text2()) };
        oben += unten - y0 + 16.0;
    }
    // the club's real history up to summer 2026 (realdaten): founding, stadium, titles, record players, rivals, legends
    if let Some(rd) = crate::realdaten::verein(&c.world.clubs[t].name) {
        let mut zeilen_rd: Vec<String> = vec![];
        let mut kopf = vec![];
        if let Some(g) = rd.gruendung { kopf.push(crate::tf!("ui.real.gruendung", g, rd.ort)); }
        if !rd.stadion.is_empty() { kopf.push(rd.stadion.clone()); }
        if rd.meister > 0 { kopf.push(crate::tf!("ui.real.meister", rd.meister)); }
        if !kopf.is_empty() { zeilen_rd.push(kopf.join(" · ")); }
        let mut rek = vec![];
        if let Some((n, s)) = &rd.rekordspieler { rek.push(crate::tf!("ui.real.rekordspieler", n, s)); }
        if let Some((n, s)) = &rd.rekordtorschuetze { rek.push(crate::tf!("ui.real.rekordtorschuetze", n, s)); }
        if !rek.is_empty() { zeilen_rd.push(rek.join(" · ")); }
        if !rd.rivalen.is_empty() { zeilen_rd.push(crate::tf!("ui.real.rivalen", rd.rivalen.iter().map(|(cl, l)| format!("{cl} ({l})")).collect::<Vec<_>>().join(", "))); }
        if !rd.europa_frueher.is_empty() { zeilen_rd.push(rd.europa_frueher.clone()); }
        if !rd.legenden.is_empty() { zeilen_rd.push(crate::tf!("ui.real.legenden", rd.legenden.iter().take(6).map(|l| l.0.clone()).collect::<Vec<_>>().join(", "))); }
        let mut y = r.y + 56.0 + oben;
        ui.text_semi(&crate::inhalt::t("ui.real.titel"), r.x + 14.0, y, 13.0, Theme::gedaempft());
        y += 18.0;
        for z in zeilen_rd { y = ui.paragraph(&z, r.x + 14.0, y, 13.0, r.w - 28.0, Theme::text2()) + 2.0; }
        oben = y - r.y - 56.0 + 14.0;
    }
    let mut sq = c.world.clubs[t].squad.clone();
    sq.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
    for (k, &p) in sq.iter().enumerate() {
        let yy = r.y + 56.0 + oben + k as f32 * 26.0;
        if yy > r.y + r.h - 10.0 { break; }
        let rr = Rect::new(r.x + 6.0, yy - 18.0, r.w - 12.0, 24.0);
        ui.player_photo(r.x + 24.0, yy - 5.0, 10.0, c.world.players[p].id, &c.world.players[p].name);
        ui.text(&format!("{} {}", pos_de(&c.world.players[p].pos), c.world.players[p].name), r.x + 40.0, yy, 15.0, Theme::text());
        ui.text_right(&crate::tf!("as.ent.talent", c.world.players[p].age, money(c.world.players[p].value)), r.x + r.w - 14.0, yy, 14.0, Theme::text2());
        if ui.hit(rr) { v.modal = Modal::Player(p); }
    }
}

fn vereinswelt(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.seiten.die_welt_in_zahlen"), Theme::akzent());
    let aktiv = c.world.players.iter().filter(|p| p.team >= 0).count();
    let z = vec![(crate::inhalt::t("kurve.saison").to_string(), date::season_label(c.world.season)), (crate::inhalt::t("liga.art.liga_mz").into(), c.world.leagues.len().to_string()), (crate::inhalt::t("gs.partner.verein").into(), c.world.clubs.len().to_string()), (crate::inhalt::t("ui.seiten.spieler_mit_verein").into(), aktiv.to_string()),
        (crate::inhalt::t("sc.profil.vereinslos").into(), c.world.players.iter().filter(|p| p.team == -1).count().to_string()), (crate::inhalt::anzeige("Interimstrainer").into(), c.world.clubs.iter().filter(|x| x.manager == "Interimstrainer").count().to_string()), // roh
        (crate::inhalt::t("zentrale.wettbewerbe").into(), c.world.cups.len().to_string())];
    zeilen(ui, l, &z, 64.0);
    ui.card_titled(r, crate::inhalt::t("ui.seiten.was_in_der_welt_passiert"), Theme::gold());
    let own = c.club().name.clone();
    let z: Vec<(String, String)> = c.feed.iter().filter(|n| !n.head.contains(&own)).take(16).map(|n| (n.head.clone(), date::short(n.season, n.day))).collect();
    zeilen(ui, r, &z, 64.0);
}

fn handbuch(ui: &mut Ui, v: &mut View, _c: &mut Career) {
    let h = crate::inhalt::eigen("handbuch");
    let kap: Vec<String> = h["kapitel"].as_array().map(|a| a.iter().filter_map(|x| x["id"].as_str().map(String::from)).collect()).unwrap_or_default();
    let (kap, t) = (&kap, crate::inhalt::eigen("handbuch_texte"));
    let tx = |k: &str| t[k].as_str().unwrap_or("").to_string();
    let l = Rect::new(X0 + 4.0, Y0, 300.0, hoehe() - Y0 - 16.0);
    ui.card_titled(l, crate::inhalt::t("ui.seiten.kapitel"), Theme::akzent());
    for (i, k) in kap.iter().enumerate() {
        let r = Rect::new(l.x + 6.0, l.y + 48.0 + i as f32 * 30.0, l.w - 12.0, 28.0);
        if v.tab == i { rrect(r.x, r.y, r.w, r.h, 4.0, alpha(Theme::akzent(), 0.15)); }
        ui.text(&tx(&format!("handbuch.{k}.titel")), r.x + 8.0, r.y + 20.0, 15.0, Theme::text());
        if ui.hit(r) { v.tab = i; }
    }
    let r = Rect::new(X0 + 316.0, Y0, cbreite() - 312.0, hoehe() - Y0 - 16.0);
    let k = kap.get(v.tab).cloned().unwrap_or_default();
    ui.card_titled(r, &tx(&format!("handbuch.{k}.titel")), Theme::gold());
    let mut yy = r.y + 64.0;
    for teil in [crate::id("kurz"), crate::id("was"), crate::id("ablauf"), crate::id("wirkung"), crate::id("mehr"), crate::id("tipps")] {
        let s = tx(&format!("handbuch.{k}.{teil}"));
        if s.is_empty() { continue; }
        if teil != crate::id("kurz") { ui.text_semi(&tx(&format!("handbuch.abschnitt.{teil}")), r.x + 14.0, yy, 15.0, Theme::akzent()); yy += 22.0; }
        let n = ui.wrap(&s, 15.0, r.w - 28.0).len() as f32;
        ui.paragraph(&s, r.x + 14.0, yy, 15.0, r.w - 28.0, Theme::text());
        yy += n * 21.0 + 12.0;
        if yy > r.y + r.h - 20.0 { break; }
    }
}

fn regeln_seite(ui: &mut Ui, v: &mut View, _c: &mut Career) {
    let ordner = crate::regeln::eigene_regeln();
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.seiten.regeldateien"), Theme::akzent());
    ui.paragraph(&crate::tf!("ui.seiten.alle_zahlen_stehen_in_regeldateien", ordner.display()), l.x + 14.0, l.y + 60.0, 15.0, l.w - 28.0, Theme::text2());
    for (i, (n, eigen)) in crate::regeln::dateien().iter().enumerate() {
        let yy = l.y + 150.0 + i as f32 * 26.0;
        if yy > l.y + l.h - 10.0 { break; }
        ui.text(n, l.x + 14.0, yy, 15.0, Theme::text());
        if *eigen { ui.text_right(crate::inhalt::t("ui.wort.angepasst"), l.x + l.w - 150.0, yy, 14.0, Theme::achtung()); }
        else if ui.button(Rect::new(l.x + l.w - 130.0, yy - 18.0, 116.0, 24.0), crate::inhalt::t("ui.seiten.anpassen"), false) { match crate::regeln::kopie_anlegen(n) { Ok(p) => v.toast(&crate::tf!("ui.seiten.kopie", p.display()), true), Err(e) => v.toast(&e, false) } }
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.so_geht_s"), Theme::gold());
    ui.paragraph(crate::inhalt::t("ui.seiten.anpassen_legt_eine_kopie_der"), r.x + 14.0, r.y + 60.0, 15.0, r.w - 28.0, Theme::text2());
    for (k, f) in crate::regeln::fehler().iter().enumerate() { ui.paragraph(f, r.x + 14.0, r.y + 200.0 + k as f32 * 40.0, 15.0, r.w - 28.0, Theme::problem()); }
}

fn profi(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("pm.f.diagnose"), Theme::akzent());
    let t = TAGESZEIT.lock().map(|x| x.clone()).unwrap_or_default();
    let mittel = if t.is_empty() { 0.0 } else { t.iter().sum::<f32>() / t.len() as f32 };
    let max = t.iter().cloned().fold(0.0, f32::max);
    let groesse = std::fs::metadata(c.save_path()).map(|m| m.len()).unwrap_or(0);
    let z = vec![(crate::inhalt::t("ui.seiten.tagesschritt_im_mittel").to_string(), crate::tf!("ui.seiten.ms", format!("{:.0}", mittel))), (crate::inhalt::t("ui.seiten.laengster_tagesschritt").into(), crate::tf!("ui.seiten.ms", format!("{:.0}", max))), (crate::inhalt::t("ui.seiten.gemessene_tage").into(), t.len().to_string()),
        (crate::inhalt::t("quelle.spielstand").into(), crate::tf!("spielstand.groesse", format!("{:.1}", groesse as f64 / 1e6))), (crate::inhalt::t("ui.seiten.spieler_in_der_welt").into(), c.world.players.len().to_string()), (crate::inhalt::t("gm.zt.z.meldungen").into(), c.inbox.len().to_string()),
        (crate::inhalt::t("ui.seiten.gruende_im_tagebuch").into(), c.mensch.gruende.len().to_string()), (crate::inhalt::t("reiter.beziehungen").into(), c.mensch.beziehungen.len().to_string()), (crate::inhalt::t("ui.seiten.angepasste_regeldateien").into(), crate::regeln::dateien().iter().filter(|x| x.1).count().to_string())];
    zeilen(ui, l, &z, 64.0);
    ui.card_titled(r, crate::id("FL27"), Theme::gold()); // roh: product name
    let z = vec![(crate::inhalt::t("ticker.kat.welt").to_string(), if c.fl27_welt.is_empty() { crate::inhalt::t("ui.main.eingebaute_probewelt_2").into() } else { c.fl27_welt.clone() }), (crate::inhalt::t("ui.seiten.spiel_starten_erlaubt").into(), if c.fl27_erlaubt { crate::id("ja").into() } else { crate::id("nein").into() }), (crate::inhalt::t("ui.seiten.karriere_ordner_im_spiel").into(), c.fl27_cpk.clone()), (crate::inhalt::t("ui.seiten.in_fl27_gespielte_spiele").into(), c.fl27_spiele.len().to_string())];
    zeilen(ui, r, &z, 64.0);
}

fn klima_seite(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.main.vereinsklima"), Theme::akzent());
    for (i, (n, w)) in [(crate::inhalt::t("ui.mensch.vorstandsvertrauen"), c.mensch.vorstand), (crate::inhalt::t("story.groesse.fanstimmung"), c.fans_eff()), (crate::inhalt::t("story.groesse.medienbild"), c.media), (crate::inhalt::t("charakter.teamgeist"), c.mensch.teamgeist)].iter().enumerate() {
        let yy = l.y + 70.0 + i as f32 * 54.0;
        ui.text_semi(n, l.x + 14.0, yy, 18.0, Theme::text());
        ui.bar(l.x + 230.0, yy - 12.0, l.w - 300.0, 12.0, w / 100.0, if *w >= 55.0 { Theme::gut() } else if *w >= 35.0 { Theme::achtung() } else { Theme::problem() });
        ui.text_right(&format!("{:.0}", w), l.x + l.w - 14.0, yy, 18.0, Theme::text());
    }
    ui.text(c.protest_wort(c.protest_stufe()), l.x + 14.0, l.y + 300.0, 17.0, Theme::text2());
    ui.card_titled(r, crate::inhalt::t("ui.seiten.die_letzten_gruende"), Theme::gold());
    let z: Vec<(String, String)> = c.mensch.gruende.iter().rev().filter(|g| ["Fans", "Medien", "Präsidium"].contains(&g.bezug.as_str())).take(18).map(|g| (format!("{}: {}", g.was, g.grund), format!("{:+.0}", g.wert))).collect(); // roh
    zeilen(ui, r, &z, 64.0);
}

fn portal(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::id("FL-Welt"), Theme::akzent()); // roh: product name
    let welt = if c.fl27_welt.is_empty() { crate::inhalt::t("ui.main.eingebaute_probewelt_2").to_string() } else { std::path::Path::new(&c.fl27_welt).file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_default() };
    let z = vec![(crate::inhalt::t("ticker.kat.welt").to_string(), welt), (crate::inhalt::t("liga.art.liga_mz").into(), c.world.leagues.len().to_string()), (crate::inhalt::t("gs.partner.verein").into(), c.world.clubs.len().to_string()), (crate::inhalt::t("ui.pages.eigene_liga").into(), c.world.leagues[c.league()].name.clone())];
    zeilen(ui, l, &z, 64.0);
    if ui.button(Rect::new(l.x + 14.0, l.y + 200.0, 220.0, 36.0), crate::inhalt::t("ui.seiten.zu_den_einstellungen"), false) { v.go(Page::Optionen); }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.in_fl27_gespielt"), Theme::gold());
    let z: Vec<(String, String)> = c.fl27_spiele.iter().rev().take(16).map(|s| (s.clone(), String::new())).collect();
    if z.is_empty() { ui.text(crate::inhalt::t("ui.seiten.noch_kein_spiel_in_fl27"), r.x + 14.0, r.y + 64.0, 16.0, Theme::text2()); }
    zeilen(ui, r, &z, 64.0);
}

fn rollen_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // Paket 4: roles per slot and instructions per player (rollenwissen.json of the original)
    let r = Rect::new(X0 + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
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
        let rollen = crate::taktik::rollen_fuer(slot);
        let akt = c.std.rollen.iter().find(|x| x.0 == k).map(|x| x.1.clone()).unwrap_or_else(|| rollen.first().map(|r| r.0.to_string()).unwrap_or_default());
        ui.text(&crate::tf!("ui.farm.text", pos_de(slot), pl), r.x + 14.0, yy, 16.0, Theme::text());
        let namen: Vec<&str> = rollen.iter().map(|x| x.1).collect();
        let sel = rollen.iter().position(|x| x.0 == akt).unwrap_or(0);
        if !namen.is_empty() { if let Some(i) = ui.segment(Rect::new(r.x + 280.0, yy - 20.0, (namen.len() as f32 * 150.0).min(r.w * 0.5), 28.0), &namen, sel) { c.std.rollen.retain(|x| x.0 != k); c.std.rollen.push((k, rollen[i].0.to_string())); } }
        let anw = c.std.anweisungen.iter().find(|x| x.0 == c.world.players[*p].id).map(|x| x.1).unwrap_or(0);
        if let Some(i) = ui.segment(Rect::new(r.x + r.w - 380.0, yy - 20.0, 366.0, 28.0), &crate::taktik::anweisungen_t(), anw as usize) { let id = c.world.players[*p].id; c.std.anweisungen.retain(|x| x.0 != id); c.std.anweisungen.push((id, i as u8)); }
    }
    let _ = v;
}

fn jugendnetz(ui: &mut Ui, v: &mut View, c: &mut Career) {
    c.netz_an();
    let (l, r) = links_rechts();
    ui.card_titled(l, crate::inhalt::t("ui.seiten.netz_je_land"), Theme::akzent());
    let netz = c.mgr.netz.clone();
    let mut sorted = netz.clone();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (k, (land, st, inv)) in sorted.iter().enumerate() {
        let yy = l.y + 62.0 + k as f32 * 34.0;
        if yy > l.y + l.h - 14.0 { break; }
        ui.text(land, l.x + 14.0, yy, 16.0, Theme::text());
        ui.bar(l.x + 150.0, yy - 10.0, 120.0, 8.0, st / 100.0, if *st >= 35.0 { Theme::gut() } else if *st >= 20.0 { Theme::achtung() } else { Theme::ueberlagerung() });
        if let Some(i) = ui.segment(Rect::new(l.x + 290.0, yy - 22.0, l.w - 304.0, 28.0), &[crate::inhalt::t("ui.seiten.kein_geld"), crate::inhalt::t("ui.wort.wenig"), crate::inhalt::t("ui.wort.mittel"), crate::inhalt::t("ui.wort.viel")], *inv as usize) { c.netz_stufe(land, i as u8); }
    }
    ui.card_titled(r, crate::inhalt::t("ui.seiten.talente_ab_netzstaerke_20_sichtbar"), Theme::gold());
    for (k, t) in c.talente().into_iter().enumerate() {
        let yy = r.y + 62.0 + k as f32 * 40.0;
        if yy > r.y + r.h - 14.0 { break; }
        ui.text(&crate::tf!("ui.wort.name_pos_alter_land", t.name, pos_de(&t.pos), t.alter, t.land), r.x + 14.0, yy, 15.0, Theme::text());
        ui.text(&crate::tf!("ui.seiten.potenzial", stufe_wort(t.ovr + 20.0), stufe_wort(t.pot)), r.x + 14.0, yy + 18.0, 13.0, Theme::text2());
        let laeuft = c.mgr.anwerbung.iter().any(|a| a.0 == t.key);
        if laeuft { ui.text_right(crate::inhalt::t("ui.seiten.wird_angeworben"), r.x + r.w - 14.0, yy, 14.0, Theme::achtung()); }
        else if ui.button(Rect::new(r.x + r.w - 140.0, yy - 18.0, 126.0, 28.0), crate::inhalt::t("jn.ui.anwerben"), false) { match c.anwerben(&t) { Ok(()) => v.toast(crate::inhalt::t("ui.seiten.anwerbung_laeuft_28_tage"), true), Err(e) => v.toast(&e, false) } }
    }
}

// ---------------------------------------------------------------- Spielerprofil (Python: SpielerprofilSeite.qml, Spieleruebersicht.qml)
/// A text of assets/texte/spielerprofil.json (copied from core/texts/spielerprofil.json, own keys added).
fn tx(k: &str) -> &'static str { profil_texte()[k].as_str().unwrap_or("") }
fn tx_liste(k: &str) -> Vec<&'static str> { profil_texte()[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default() }
fn profil_texte() -> &'static serde_json::Value { crate::inhalt::eigen("spielerprofil") }
fn pw(k: &str) -> &'static serde_json::Value { crate::regeln::wert("profilseite", k) }
use crate::regeln::f;
fn indizes(v: &serde_json::Value) -> Vec<usize> { v.as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|x| x as usize)).collect()).unwrap_or_default() }
/// Groups of the 25 abilities: (id, indices, colour).
fn gruppen() -> Vec<(&'static str, Vec<usize>, Color)> {
    pw("gruppen").as_array().unwrap().iter().map(|g| (g["id"].as_str().unwrap(), indizes(&g["werte"]), hex(u32::from_str_radix(g["farbe"].as_str().unwrap(), 16).unwrap()))).collect()
}
/// Stars of a value as on the template (40 → ½, every 11 points one star, at most 5).
fn profil_sterne(w: f32) -> f32 { (((w - f("profilseite", "sterne.von")) / f("profilseite", "sterne.je_stern") * 2.0).round() / 2.0).clamp(0.5, 5.0) }
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

fn spielerprofil(ui: &mut Ui, v: &mut View, c: &mut Career) {
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
    tabs(ui, v, &reiter);
    if own { ui.text_right(tx("spp.stand_aktuell"), X0 + cbreite(), Y0 - 20.0, 15.0, Theme::text2()); }
    else { ui.text_right(&tx("spp.stand").replace("%1", &date::long(c.world.season, c.world.day)).replace("%2", &format!("{:.0}", c.know_of(p) * 100.0)), X0 + cbreite(), Y0 - 20.0, 15.0, Theme::text2()); }
    ticker(ui, c, Rect::new(X0 + 4.0, Y0, cbreite(), 26.0));

    // ---- head: number, photo, names, body, position, club, contract, buttons
    let hr = Rect::new(X0 + 4.0, Y0 + 34.0, cbreite(), 112.0);
    let club = if pl.team >= 0 { pl.team as usize } else { c.own };
    let (c1, _) = club_color(c, club);
    let fg = on(c1);
    rrect(hr.x, hr.y, hr.w, hr.h, 10.0, c1);
    ui.rahmen(hr);
    ui.rahmen(Rect::new(hr.x + 226.0, hr.y, 406.0, hr.h));
    ui.title(&if pl.no > 0 { pl.no.to_string() } else { "–".into() }, hr.x + 18.0, hr.y + 86.0, 76.0, alpha(fg, 0.9));
    let fr = Rect::new(hr.x + 118.0, hr.y + 6.0, 96.0, 100.0);
    rrect(fr.x, fr.y, fr.w, fr.h, 8.0, alpha(Theme::grund(), 0.35));
    if !ui.spieler_foto(fr.x, fr.y, fr.w, fr.h, pl.id) {
        draw_circle(fr.x + 48.0, fr.y + 38.0, 18.0, alpha(fg, 0.35));
        draw_circle(fr.x + 48.0, fr.y + 92.0, 32.0, alpha(fg, 0.35));
        ui.text_center(tx("spp.foto_fehlt"), fr.x + 48.0, fr.y + 96.0, 12.0, fg);
    }
    let teile: Vec<&str> = pl.name.split_whitespace().collect();
    let (vor, nach) = if teile.len() > 1 { (teile[..teile.len() - 1].join(" "), teile[teile.len() - 1].to_string()) } else { (String::new(), pl.name.clone()) };
    ui.text_semi(&vor, hr.x + 232.0, hr.y + 30.0, 18.0, alpha(fg, 0.85));
    let nach = ui.fit(&nach, 38.0, 380.0);
    ui.title(&nach, hr.x + 232.0, hr.y + 70.0, 38.0, fg);
    let k = pl.profil.koerper;
    let koerper = if k[0] > 0 { crate::tf!("ui.seiten.cm_kg", k[0], k[1], tx(if k[2] == 1 { "spp.fuss.links" } else { "spp.fuss.rechts" })) } else { crate::tf!("ui.seiten.groesse_gewicht_fuss", unbek) };
    ui.text(&crate::tf!("as.ent.talent", pl.age, koerper), hr.x + 232.0, hr.y + 98.0, 16.0, alpha(fg, 0.85));
    let rx = hr.x + 640.0;
    rrect(rx, hr.y + 10.0, 54.0, 24.0, 6.0, alpha(fg, 0.18));
    ui.text_center(pos_de(&pl.pos), rx + 27.0, hr.y + 28.0, 15.0, fg);
    ui.text_semi(&crate::tf!("liga.nummer", pl.no), rx + 66.0, hr.y + 28.0, 16.0, fg);
    let clubname = if pl.team >= 0 { c.world.clubs[pl.team as usize].name.clone() } else if pl.team == -1 { crate::inhalt::t("ui.wort.vereinslos").into() } else { crate::inhalt::t("ui.pages.karriereende").into() };
    if pl.team >= 0 { let cl = &c.world.clubs[pl.team as usize]; ui.club_crest(rx + 150.0, hr.y + 22.0, 13.0, cl.id, hex(cl.c1), &cl.short); }
    let cn = ui.fit(&clubname, 16.0, 200.0);
    ui.text_semi(&cn, rx + 170.0, hr.y + 28.0, 16.0, fg);
    ui.text_right(&tx("spp.vertrag_bis").replace("%1", &(date::START_YEAR as u32 + pl.contract_end + 1).to_string()), hr.x + hr.w - 16.0, hr.y + 28.0, 16.0, fg);
    // buttons (own: club, watch, talk, transfer list; foreign: club, watch, scout, offer)
    let bw = (hr.w - 656.0 - 16.0) / 3.0;
    let bt = |i: usize| Rect::new(rx + (i % 3) as f32 * (bw + 8.0), hr.y + 42.0 + (i / 3) as f32 * 34.0, bw, 30.0);
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
            if let Some(le) = &pl.leihe { ui.text(&crate::tf!("ui.seiten.verliehen_bis", date::season_label(le.bis)), bt(3).x + 4.0, bt(3).y + 21.0, 14.0, fg); }
        } else if pl.listed {
            if ui.button(bt(3), tx("spp.k.von_liste"), true) { c.world.players[p].listed = false; c.world.players[p].profil.mindestpreis = 0.0; }
            if pl.profil.mindestpreis > 0.0 { ui.text(&crate::tf!("tf.liste.modus.preis", money(pl.profil.mindestpreis)), bt(4).x + 4.0, bt(4).y + 21.0, 14.0, fg); }
        } else {
            if ui.button(bt(3), tx("spp.k.liste_angebote"), false) { c.world.players[p].listed = true; c.world.players[p].profil.mindestpreis = 0.0; v.toast(crate::inhalt::t("ui.seiten.auf_der_transferliste_du_pruefst"), true); }
            if ui.button(bt(4), tx("spp.k.liste_mindest"), false) { c.world.players[p].listed = true; c.world.players[p].profil.mindestpreis = round_money(pl.value); v.toast(&crate::tf!("ui.seiten.auf_der_transferliste_angebote_unter", money(round_money(pl.value))), true); }
        }
    } else if pl.team != -2 {
        let watched = c.scouts.iter().any(|s| s.watch == Some(p));
        if ui.button(bt(2), tx("spp.k.scouten"), watched) && !watched {
            if let Some(s) = c.scouts.iter_mut().find(|s| s.watch.is_none()) { s.watch = Some(p); let n = s.name.clone(); v.toast(&crate::tf!("ui.pages.beobachtet_ihn_ab_jetzt", n), true); } else { v.toast(crate::inhalt::t("ui.pages.alle_scouts_haben_schon_einen"), false); }
        }
        if ui.button(bt(3), tx("spp.k.angebot"), false) { match c.open_neg(p) { Ok(i) => { v.modal = Modal::Neg(i); return; } Err(e) => v.toast(&e, false) } }
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
        let r = Rect::new(X0 + 4.0 + i as f32 * (kw + 10.0), Y0 + 154.0, kw, 54.0);
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
            let a = Rect::new(X0 + 4.0, cy, 280.0, ch);
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
                ui.text(pos_de(crate::pesdb::POS[k]), a.x + 14.0, y, 15.0, Theme::text());
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
                let gk: Vec<usize> = gr.iter().find(|g| g.0 == crate::id("torwart")).map(|g| g.1.clone()).unwrap_or_default();
                let mut zaehlen: Vec<usize> = if tw { gk.iter().copied().chain(indizes(pw("torwart_zusatz"))).collect() } else { (0..25).filter(|k| !gk.contains(k)).collect() };
                zaehlen.sort_by(|x, y| pl.attrs[*y].partial_cmp(&pl.attrs[*x]).unwrap());
                let n3 = f("profilseite", "staerken_anzahl") as usize;
                ui.text_semi(tx("spp.staerken_schwaechen"), d.x + 14.0, d.y + 222.0, 14.0, Theme::text2());
                let mut y = d.y + 244.0;
                for (k, gut) in zaehlen.iter().take(n3).map(|k| (*k, true)).chain(zaehlen.iter().rev().take(n3).map(|k| (*k, false))) {
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
            let (l, r) = (Rect::new(X0 + 4.0, cy, cbreite() * 0.5 - 6.0, ch), Rect::new(X0 + cbreite() * 0.5 + 10.0, cy, cbreite() * 0.5 - 6.0, ch));
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
                (crate::inhalt::t("ui.nation").into(), { let n = crate::nationen::name(&c.world, pl.nat); if n.is_empty() { "–".to_string() } else { n } }),
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
            let (l, r) = (Rect::new(X0 + 4.0, cy, cbreite() * 0.5 - 6.0, ch), Rect::new(X0 + cbreite() * 0.5 + 10.0, cy, cbreite() * 0.5 - 6.0, ch));
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
        _ => {
            // career stations per season and the contract
            let (l, r) = (Rect::new(X0 + 4.0, cy, cbreite() * 0.62 - 6.0, ch), Rect::new(X0 + cbreite() * 0.62 + 10.0, cy, cbreite() * 0.38 - 6.0, ch));
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
            if let Some(le) = &pl.leihe { z.push((crate::inhalt::t("dlg.punkt.leihe").into(), crate::tf!("ui.seiten.von_bis", c.world.clubs[le.von].name, date::season_label(le.bis)))); }
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
/// Tab „Entwicklung“: history per group with the reasons of the last weeks, phase, focus, traits and progress paths (entwicklung.rs).
fn entwicklung_reiter(ui: &mut Ui, c: &Career, p: usize, cy: f32, ch: f32) {
    use crate::entwicklung::{GRUPPEN, PHASEN};
    let pl = &c.world.players[p];
    let (l, r) = (Rect::new(X0 + 4.0, cy, cbreite() * 0.62 - 6.0, ch), Rect::new(X0 + cbreite() * 0.62 + 10.0, cy, cbreite() * 0.38 - 6.0, ch));
    ui.card_titled(l, crate::inhalt::t("ui.ent.verlauf"), Theme::akzent());
    let e = &pl.ent;
    if !c.gehoert_uns(p) || !e.an || e.verlauf.is_empty() {
        ui.paragraph(crate::inhalt::t(if c.gehoert_uns(p) { "ui.ent.noch_kein_verlauf" } else { "ui.ent.nur_eigene" }), l.x + 14.0, l.y + 64.0, 15.0, l.w - 28.0, Theme::text2());
        ui.card_titled(r, crate::inhalt::t("ui.ent.stand"), Theme::gold());
        return;
    }
    let farben = [hex(0xE04A4A), hex(0xE0A526), hex(0x3BA3DA), hex(0x8E44AD), hex(0x27AE60)];
    let gruppe_name = |g: usize| crate::inhalt::t(&format!("ui.ent.gruppe.{}", GRUPPEN[g]));
    let zahl = |x: f32| { let s = format!("{x:+.1}"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { s.replace('.', ",") } else { s } };
    let torwart = pl.pos == "GK";
    // cumulative change per group, one line each (a field player's goalkeeping abilities hardly move and are left out)
    let mut summe = [0.0f32; 5];
    let reihen: Vec<[f32; 5]> = e.verlauf.iter().map(|w| { for g in 0..5 { summe[g] += w.gruppen[g]; } summe }).collect();
    let zeigen: Vec<usize> = (0..5).filter(|&g| torwart || g != 4).collect();
    let (gx, gy, gw, gh) = (l.x + 50.0, l.y + 60.0, l.w - 70.0, (l.h * 0.34).clamp(110.0, 200.0));
    let lo = reihen.iter().flat_map(|x| zeigen.iter().map(move |&g| x[g])).fold(-0.3f32, f32::min);
    let hi = reihen.iter().flat_map(|x| zeigen.iter().map(move |&g| x[g])).fold(0.3f32, f32::max);
    let yv = |v: f32| gy + gh - (v - lo) / (hi - lo) * gh;
    rrect(gx, gy, gw, gh, 4.0, Theme::erhoehung());
    draw_line(gx, yv(0.0), gx + gw, yv(0.0), 1.0, Theme::kontur());
    ui.text_right(&zahl(hi), gx - 6.0, gy + 10.0, 12.0, Theme::gedaempft());
    ui.text_right(&zahl(lo), gx - 6.0, gy + gh, 12.0, Theme::gedaempft());
    let n = reihen.len();
    let xv = |i: usize| gx + 6.0 + if n > 1 { i as f32 / (n - 1) as f32 * (gw - 12.0) } else { 0.0 };
    for &g in &zeigen {
        for i in 1..n { draw_line(xv(i - 1), yv(reihen[i - 1][g]), xv(i), yv(reihen[i][g]), 2.0, farben[g]); }
        if let Some(last) = reihen.last() { draw_circle(xv(n - 1), yv(last[g]), 3.0, farben[g]); }
    }
    // legend with the change over the shown weeks
    let mut x = gx;
    for &g in &zeigen {
        draw_rectangle(x, gy + gh + 14.0, 12.0, 4.0, farben[g]);
        let t = crate::tf!("ui.ent.legende_gruppe", gruppe_name(g), zahl(summe[g]));
        ui.text(&t, x + 16.0, gy + gh + 21.0, 13.0, Theme::text2());
        x += ui.measure(&t, 13.0) + 34.0;
    }
    // the last weeks: their biggest change and its reasons
    let mut y = gy + gh + 52.0;
    ui.text_semi(crate::inhalt::t("ui.ent.letzte_wochen"), l.x + 14.0, y, 14.0, Theme::gedaempft());
    y += 24.0;
    for w in e.verlauf.iter().rev().take(crate::regeln::f("entwicklung_pesm", "anzeige.gruende_wochen") as usize) {
        if y > l.y + l.h - 12.0 { break; }
        let g = zeigen.iter().copied().max_by(|a, b| w.gruppen[*a].abs().partial_cmp(&w.gruppen[*b].abs()).unwrap()).unwrap();
        let gruende: Vec<&str> = w.gruende.iter().map(|k| crate::inhalt::t(&format!("ui.ent.grund.{}", k.replace('+', "_plus").replace('-', "_minus")))).filter(|s| !s.is_empty()).take(3).collect();
        let warum = if gruende.is_empty() { crate::inhalt::t(&format!("en3.phase.{}", PHASEN[w.phase as usize])).to_string() } else { gruende.join(", ") };
        let n_gruppe = crate::entwicklung::GRUPPE.iter().filter(|&&x| x == g).count() as f32;
        let wert = w.gruppen[g] * n_gruppe;
        // a small weekly change keeps two decimals, so it doesn't read as „+0,0“
        let text = if wert.abs() < 0.1 { let s = format!("{wert:+.2}"); if crate::inhalt::sprache() == crate::inhalt::Sprache::De { s.replace('.', ",") } else { s } } else { zahl(wert) };
        let zeile = crate::tf!("ui.ent.zeile", text, gruppe_name(g), warum);
        ui.text(&date::short(c.world.season, w.tag % date::SEASON_DAYS), l.x + 14.0, y, 14.0, Theme::text2());
        ui.text(&ui.fit(&zeile, 15.0, l.w - 120.0), l.x + 100.0, y, 15.0, if w.gruppen[g] >= 0.0 { Theme::text() } else { Theme::problem() });
        y += 22.0;
    }
    // state: phase, expected development, focus, load, traits, progress paths
    ui.card_titled(r, crate::inhalt::t("ui.ent.stand"), Theme::gold());
    let abs = c.abs();
    let phase = crate::inhalt::t(&format!("en3.phase.{}", PHASEN[e.phase as usize]));
    let phase = if (1..=3).contains(&e.phase) && e.bis > abs { crate::tf!("ui.ent.phase_bis", phase, (e.bis - abs) / 7 + 1) } else { phase.to_string() };
    let fokus = crate::entwicklung::fokus_von(pl);
    let z: Vec<(String, String)> = vec![
        (crate::inhalt::t("ui.ent.phase").into(), phase),
        (crate::inhalt::t("ui.ent.fokus").into(), if fokus.is_empty() { crate::inhalt::t("ui.ent.kein_fokus").into() } else { crate::inhalt::t(&format!("ui.ent.fokus.{fokus}")).into() }),
        (crate::inhalt::t("ui.ent.belastung").into(), format!("{:.0}", pl.ps.last)),
    ];
    zeilen(ui, r, &z, 64.0);
    let mut y = r.y + 64.0 + z.len() as f32 * 30.0;
    ui.text(crate::inhalt::t("ui.ent.erwartet"), r.x + 14.0, y, 15.0, Theme::text2());
    entwicklung_pfeil(ui, r.x + 220.0, y - 5.0, c.ent_erwartung(p));
    y += 30.0;
    ui.text_semi(crate::inhalt::t("ui.ent.merkmale"), r.x + 14.0, y, 14.0, Theme::gedaempft());
    y += 10.0;
    let mut x = r.x + 14.0;
    for m in &e.merkmale {
        let t = crate::inhalt::t(&format!("mk.{m}"));
        let w = ui.measure(t, 14.0) + 20.0;
        if x + w > r.x + r.w - 14.0 { x = r.x + 14.0; y += 30.0; }
        rrect(x, y, w, 24.0, 12.0, alpha(Theme::akzent(), 0.16));
        ui.text(t, x + 10.0, y + 17.0, 14.0, Theme::text());
        x += w + 8.0;
    }
    y += 46.0;
    ui.text_semi(crate::inhalt::t("ui.ent.wege"), r.x + 14.0, y, 14.0, Theme::gedaempft());
    y += 20.0;
    let bis = crate::regeln::f("entwicklung2", "wege.treffer_bis");
    if e.wege.is_empty() { ui.text(crate::inhalt::t("ui.ent.keine_wege"), r.x + 14.0, y + 4.0, 14.0, Theme::text2()); }
    for (id, t, _) in &e.wege {
        if y > r.y + r.h - 20.0 { break; }
        ui.text(&ui.fit(crate::inhalt::t(&format!("en2.weg.{id}")), 15.0, r.w * 0.5), r.x + 14.0, y + 4.0, 15.0, Theme::text());
        if *t as f32 >= bis { ui.text_right(crate::inhalt::t("ui.ent.erreicht"), r.x + r.w - 14.0, y + 4.0, 14.0, Theme::gut()); }
        else { ui.bar(r.x + r.w * 0.55, y - 4.0, r.w * 0.45 - 14.0, 8.0, *t as f32 / bis, Theme::akzent()); }
        y += 26.0;
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

/// „Was jetzt?“ (Spielgefühl Teil 4, Maßnahme 1): the one thing to do now, why, until when, and a button that goes there.
fn was_jetzt_karte(ui: &mut Ui, v: &mut View, c: &Career, r: Rect) {
    let w = c.was_jetzt();
    let farbe = if w.vorrang { Theme::problem() } else { Theme::akzent() };
    ui.card_titled(r, crate::inhalt::t("ui.wj.titel"), farbe);
    if !w.frist.is_empty() {
        let fw = ui.measure_semi(&w.frist, 13.0) + 16.0;
        rrect(r.x + r.w - fw - 12.0, r.y + 7.0, fw, 22.0, 11.0, alpha(hex(0xffffff), 0.25));
        ui.text_semi(&w.frist, r.x + r.w - fw - 4.0, r.y + 23.0, 13.0, on(farbe));
    }
    let kw = if w.knopf.is_empty() { 0.0 } else { (r.w * 0.36).min(190.0) };
    ui.title(&ui.fit(&w.titel, 20.0, r.w - kw - 36.0), r.x + 16.0, r.y + 66.0, 20.0, Theme::text());
    ui.text_passend(&w.grund, r.x + 16.0, r.y + 90.0, 14.0, r.w - kw - 36.0, 2, Theme::text2());
    if kw > 0.0 && ui.button(Rect::new(r.x + r.w - kw - 12.0, r.y + 58.0, kw, 36.0), &w.knopf, w.vorrang) {
        if let Some(p) = w.ziel { v.go(p); }
        if w.msg.is_some() { v.sel_msg = w.msg; }
    }
}
/// Saisonreise (Maßnahme 2): the stages of the season on a band, today's marker, the season goal as a bar.
fn saisonreise_karte(ui: &mut Ui, c: &Career, r: Rect) {
    ui.card_titled(r, crate::inhalt::t("ui.reise.titel"), Theme::akzent());
    let (etappen, letzt, (ziel, platz, teams)) = c.saisonreise();
    let (bx, bw, by) = (r.x + 22.0, r.w - 44.0, r.y + 56.0);
    let pos = |d: u32| bx + bw * (d.min(letzt) as f32 / letzt as f32);
    rrect(bx, by - 2.0, bw, 4.0, 2.0, Theme::linie());
    rrect(bx, by - 2.0, pos(c.world.day) - bx, 4.0, 2.0, Theme::verein());
    for (k, (key, tag)) in etappen.iter().enumerate() {
        let x = pos(*tag);
        let erreicht = c.world.day >= *tag;
        draw_circle(x, by, 5.0, if erreicht { Theme::verein() } else { Theme::flaeche() });
        draw_circle_lines(x, by, 5.0, 1.5, if erreicht { Theme::verein() } else { Theme::gedaempft() });
        // labels alternate above and below the band so neighbours never overlap
        let ly = if k % 2 == 0 { by + 18.0 } else { by - 9.0 };
        let l = crate::inhalt::t(key);
        let lw = ui.measure(l, 11.0);
        ui.text(l, (x - lw / 2.0).clamp(r.x + 6.0, r.x + r.w - lw - 6.0), ly, 11.0, if erreicht { Theme::text2() } else { Theme::gedaempft() });
    }
    let hx = pos(c.world.day);
    draw_triangle(vec2(hx - 6.0, by - 12.0), vec2(hx + 6.0, by - 12.0), vec2(hx, by - 4.0), Theme::problem());
    // season goal: full when the place is reached, else how far up the table there is still to go
    let anteil = if platz <= ziel { 1.0 } else { (teams.saturating_sub(platz)) as f32 / (teams.saturating_sub(ziel)).max(1) as f32 };
    let col = if anteil >= 1.0 { Theme::gut() } else if anteil >= 0.6 { Theme::achtung() } else { Theme::problem() };
    // the goal on its own line under the stage labels: text on the left, bar on the right
    let gy = r.y + r.h - 16.0;
    let zt = crate::tf!("ui.reise.ziel", ziel, platz);
    let tw = ui.measure(&zt, 12.0) + 12.0;
    ui.text(&zt, r.x + 16.0, gy + 3.0, 12.0, Theme::text2());
    rrect(r.x + 16.0 + tw, gy - 4.0, r.w - 32.0 - tw, 7.0, 3.5, Theme::ueberlagerung());
    rrect(r.x + 16.0 + tw, gy - 4.0, (r.w - 32.0 - tw) * anteil.clamp(0.02, 1.0), 7.0, 3.5, col);
}
/// Geführte erste Woche (Maßnahme 9): five steps with ticks, the count, and a button to hide it.
fn erste_woche_karte(ui: &mut Ui, c: &mut Career, r: Rect) {
    let n = c.erste_woche.iter().filter(|x| **x).count();
    ui.card_titled(r, &crate::tf!("profil.kurz_text", crate::inhalt::t("ui.woche.titel"), crate::tf!("ui.woche.stand", n)), Theme::gold());
    for (i, key) in crate::spielgefuehl::SCHRITTE.iter().enumerate() {
        let y = r.y + 50.0 + i as f32 * 12.5;
        let fertig = c.erste_woche[i];
        if fertig { draw_circle(r.x + 22.0, y - 4.0, 4.5, Theme::gut()); } else { draw_circle_lines(r.x + 22.0, y - 4.0, 4.5, 1.2, Theme::gedaempft()); }
        ui.text(crate::inhalt::t(key), r.x + 34.0, y, 11.5, if fertig { Theme::gedaempft() } else { Theme::text() });
    }
    if ui.button(Rect::new(r.x + r.w - 92.0, r.y + r.h - 30.0, 80.0, 22.0), crate::inhalt::t("ui.woche.aus"), false) { c.erste_woche_aus = true; }
}

fn zentrale(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // Spielgefühl Teil 4: the one thing to do now, the season journey and, while it runs, the guided first week
    let kopf_h = 112.0;
    let woche = c.erste_woche_offen();
    let bw = cbreite();
    let (w1, w2) = if woche { (bw * 0.4, bw * 0.34) } else { (bw * 0.5 - 6.0, bw * 0.5 - 6.0) };
    was_jetzt_karte(ui, v, c, Rect::new(X0 + 4.0, Y0, w1, kopf_h));
    saisonreise_karte(ui, c, Rect::new(X0 + 16.0 + w1, Y0, w2, kopf_h));
    if woche { erste_woche_karte(ui, c, Rect::new(X0 + 28.0 + w1 + w2, Y0, bw - w1 - w2 - 24.0, kopf_h)); }
    let y0 = Y0 + kopf_h + 12.0;
    let w3 = (cbreite() - 24.0) / 3.0;
    for (k, (gruppe, haupt)) in MENU.iter().enumerate() {
        let seiten: Vec<(Page, &str)> = haupt.iter().flat_map(|h| reiter(*h, c)).collect();
        let (col, row) = (k % 3, k / 3);
        // three rows share the rest of the height (198 each at most)
        let rh = ((hoehe() - y0 - 16.0) / 3.0).min(198.0);
        let r = Rect::new(X0 + 4.0 + col as f32 * (w3 + 12.0), y0 + row as f32 * rh, w3, rh - 10.0);
        let schritt = ((r.h - 50.0) / 6.0).min(22.0);
        let kurz = match k { 0 => crate::tf!("vg.zaehler.ungelesen", c.unread()), 1 => crate::tf!("ui.seiten.spieler_teamgeist", c.squad().len(), format!("{:.0}", c.mensch.teamgeist)), 2 => crate::tf!("ui.seiten.budget", money(c.transfer_budget)), 3 => crate::tf!("ui.seiten.talente", c.world.clubs[c.own].squad.iter().filter(|&&p| c.world.players[p].youth > 0).count()), 4 => crate::tf!("ui.seiten.vertrauen_kasse", format!("{:.0}", c.mensch.vorstand), money(c.club().balance)), 5 => crate::tf!("zentrale.k.platz_kurz", c.world.rank_of(c.own)), _ => crate::tf!("an.mind.medien", format!("{:.0}", c.media)) };
        ui.card_titled(r, &crate::tf!("profil.kurz_text", crate::inhalt::t(gruppe), kurz), Theme::akzent());
        for (i, (pg, name)) in seiten.iter().enumerate().take(12) {
            let (cc, rr) = (i % 2, i / 2);
            let b = Rect::new(r.x + 10.0 + cc as f32 * (r.w / 2.0 - 6.0), r.y + 44.0 + rr as f32 * schritt, r.w / 2.0 - 14.0, schritt - 2.0);
            if ui.button(b, name, false) { v.go(*pg); }
        }
    }
    // the home stadium, big, in the free space of the last row
    let n = MENU.len();
    if n % 3 != 0 {
        let col = n % 3;
        let rh = ((hoehe() - y0 - 16.0) / 3.0).min(198.0);
        let r = Rect::new(X0 + 4.0 + col as f32 * (w3 + 12.0), y0 + (n / 3) as f32 * rh, (3 - col) as f32 * (w3 + 12.0) - 12.0, rh - 10.0);
        if ui.stadion_bild(r, c.club().id) {
            draw_rectangle(r.x, r.y + r.h - 34.0, r.w, 34.0, alpha(BLACK, 0.55));
            ui.text_semi(crate::inhalt::t("ui.seiten.unser_stadion"), r.x + 12.0, r.y + r.h - 11.0, 17.0, WHITE);
        }
    }
}
