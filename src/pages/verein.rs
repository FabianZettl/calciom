use super::*;

/// Vereinsgeschichte (Teil 3, Rekorde und Geschichte): the coach's career and honours, the club's trophy cabinet, records
/// from the own matches and a hall of fame from every player's career stations at this club.
pub(super) fn geschichte(ui: &mut Ui, _v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let (bw, bh) = ((cbreite() - 12.0) / 2.0, (hoehe() - Y0 - 28.0) / 2.0);
    let felder = [Rect::new(x0() + 4.0, Y0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0, bw, bh), Rect::new(x0() + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
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
    if titel.is_empty() {
        let y0 = ui.paragraph(t("ui.seiten.noch_kein_titel_in_deiner"), r.x + 14.0, r.y + 60.0, 16.0, r.w - 28.0, Theme::text2());
        // the club's real honours before you (realdaten, up to summer 2026): kind, how many, the last one
        if let Some(rd) = crate::realdaten::verein(&c.club().name).filter(|rd| !rd.titel.is_empty()) {
            ui.text_semi(&t("ui.real.titel").to_uppercase(), r.x + 14.0, y0 + 18.0, 13.0, Theme::gedaempft());
            for (k, (art, jahre)) in rd.titel.iter().enumerate() {
                let y = y0 + 46.0 + k as f32 * 28.0;
                if y > r.y + r.h - 12.0 { break; }
                draw_circle(r.x + 24.0, y - 5.0, 7.0, Theme::gold());
                ui.text(t(&format!("ui.titelart.{art}")), r.x + 40.0, y, 15.0, Theme::text());
                let letzt = jahre.iter().max().map(|j| crate::tf!("ui.titelart.zuletzt", j)).unwrap_or_default();
                ui.text_right(&format!("{}× · {letzt}", jahre.len()), r.x + r.w - 14.0, y, 15.0, Theme::text2());
            }
        }
    }
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
    // records from the own matches (all seasons with this club in CalcioM)
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

pub(super) fn klima_seite(ui: &mut Ui, _v: &mut View, c: &mut Career) {
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
    // the same reason on several days in a row is one line with its sum and how often it came (no list of repeats)
    let mut z: Vec<(String, f32, u32)> = vec![];
    for g in c.mensch.gruende.iter().rev().filter(|g| ["Fans", "Medien", "Präsidium"].contains(&g.bezug.as_str())) { // roh
        let t = format!("{}: {}", g.was, g.grund);
        match z.iter_mut().find(|x| x.0 == t) { Some(x) => { x.1 += g.wert; x.2 += 1; } None => { if z.len() >= 18 { break; } z.push((t, g.wert, 1)); } }
    }
    let z: Vec<(String, String)> = z.into_iter().map(|(t, w, n)| (if n > 1 { format!("{t} (×{n})") } else { t }, format!("{w:+.0}"))).collect();
    zeilen(ui, r, &z, 64.0);
}
