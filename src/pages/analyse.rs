use super::*;

/// Verlauf (Teil 3, „Verlauf statt Zahl“): four curves of the season, each with one sentence on what it says.
fn verlauf_seite(ui: &mut Ui, c: &Career) {
    let v = c.verlauf_saison();
    let (bw, bh) = ((cbreite() - 12.0) / 2.0, (hoehe() - Y0 - 28.0) / 2.0);
    let felder = [Rect::new(x0() + 4.0, Y0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0, bw, bh), Rect::new(x0() + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
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
    let felder = [Rect::new(x0() + 4.0, Y0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0, bw, bh), Rect::new(x0() + 4.0, Y0 + bh + 12.0, bw, bh), Rect::new(x0() + 16.0 + bw, Y0 + bh + 12.0, bw, bh)];
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

pub(super) fn analyse_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
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
