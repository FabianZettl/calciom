use super::*;

pub(super) fn ranglisten(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    // every league has its rankings (Fabian 22.11.): country and league chosen in the banner like on the table page
    // leagues, cups and European competitions alike (Fabian 24.11.): a cup is 1000 + its index
    let gruppen: Vec<(String, Vec<usize>)> = crate::pages::wettbewerb_gruppen(c);
    let gueltig = |x: usize| if x >= 1000 { x - 1000 < c.world.cups.len() } else { x < c.world.leagues.len() };
    let l = v.table_league.filter(|&x| gueltig(x)).unwrap_or(c.league());
    let felder = crate::pages::wettbewerb_felder(c, &gruppen, l, Rect::new(x0() + 4.0 + cbreite() - 480.0, Y0 - 40.0, 470.0, 36.0));
    if let Some((f, i)) = crate::pages::auswahl_vorher(ui, v, &felder) { v.table_league = Some(crate::pages::wahl_umsetzen(&gruppen, l, f, i)); }
    let l = v.table_league.filter(|&x| gueltig(x)).unwrap_or(c.league());
    let wo = if l >= 1000 { Wo::Pokal(l - 1000) } else { Wo::Liga(l) };
    let lk = crate::pages::wo_look(c, wo);
    let name = match wo { Wo::Liga(x) => c.world.leagues[x].name.clone(), Wo::Pokal(x) => c.world.cups[x].name.clone() };
    let _ = crate::pages::wettbewerb_banner(ui, Rect::new(x0() + 4.0, Y0 - 44.0, cbreite(), 46.0), &lk, c.comp_cid(wo), &name);
    let teams = match wo { Wo::Liga(x) => c.world.leagues[x].teams.clone(), Wo::Pokal(x) => c.world.cups[x].teams.clone() };
    // only what was scored in this league counts (Fabian 23.11.: youth and U23 games stay out)
    let ls: std::collections::HashMap<usize, crate::world::LigaStat> = c.world.players.iter().enumerate().filter_map(|(p, pl)| pl.profil.ligen.iter().find(|x| x.l as usize == l).map(|x| (p, x.clone()))).collect();
    let mut alle: Vec<usize> = ls.keys().copied().collect();
    alle.sort_unstable();
    let (w3, h2) = ((cbreite() - 24.0) / 3.0, (hoehe() - Y0 - 36.0) / 2.0);
    let feld = |k: usize| Rect::new(x0() + 4.0 + (k % 3) as f32 * (w3 + 12.0), Y0 + 8.0 + (k / 3) as f32 * (h2 + 12.0), w3, h2);
    let kurz = |p: usize| c.world.clubs[c.world.players[p].team.max(0) as usize].short.clone();
    // a ranking card (Fabian 22.11.: more spectacular): the top three as podium cards with photo (players) or crest
    // (clubs), gold/silver/bronze edge and the big value, then the rest as rows with crest
    let liste = |ui: &mut Ui, k: usize, titel: &str, farbe: Color, z: Vec<(Option<usize>, usize, String, String)>, satz: String| {
        let r = feld(k);
        ui.card_titled(r, titel, farbe);
        ui.text(&ui.fit(&satz, 14.0, r.w - 28.0), r.x + 14.0, r.y + 56.0, 14.0, Theme::text2());
        let medaille = [hex(0xD4AF37), hex(0xB8C0CC), hex(0xCD7F32)];
        let bw = (r.w - 28.0 - 16.0) / 3.0;
        let bh = 132.0;
        for (j, (sp, club, name, wert)) in z.iter().take(3).enumerate() {
            // the first in the middle and a little higher, like a podium
            let pos = [1usize, 0, 2][j];
            let hx = r.x + 14.0 + pos as f32 * (bw + 8.0);
            let hy = r.y + 68.0 + if j == 0 { 0.0 } else { 14.0 };
            let hh = bh - if j == 0 { 0.0 } else { 14.0 };
            rrect(hx, hy, bw, hh, 10.0, alpha(medaille[j], 0.16));
            rrect_line(hx, hy, bw, hh, 10.0, medaille[j]);
            // our players stand out (Fabian 24.11.: so we find them)
            if *club == c.own { rrect(hx, hy, bw, hh, 10.0, alpha(Theme::akzent(), 0.14)); rrect_line(hx - 2.0, hy - 2.0, bw + 4.0, hh + 4.0, 11.0, Theme::akzent()); }
            let cl = &c.world.clubs[*club];
            let mx = hx + bw / 2.0;
            match sp {
                Some(p) => { ui.player_photo(mx, hy + 36.0, 26.0, c.world.players[*p].id, &c.world.players[*p].name); ui.club_crest(mx + 22.0, hy + 54.0, 9.0, cl.id, hex(cl.c1), &cl.short); }
                None => { ui.club_crest(mx, hy + 36.0, 26.0, cl.id, hex(cl.c1), &cl.short); }
            }
            draw_circle(hx + 16.0, hy + 16.0, 11.0, medaille[j]);
            ui.text_center(&format!("{}", j + 1), hx + 16.0, hy + 21.0, 14.0, hex(0x10141c));
            let kurzname = name.split(" (").next().unwrap_or(name);
            ui.text_center(&ui.fit(kurzname, 13.0, bw - 10.0), mx, hy + hh - 30.0, 13.0, Theme::text());
            ui.title(wert, mx - ui.measure_title(wert, 20.0) / 2.0, hy + hh - 8.0, 20.0, medaille[j]);
        }
        let start_y = r.y + 68.0 + bh + 18.0;
        let platz = ((r.y + r.h - start_y) / 24.0).max(0.0) as usize;
        // our best player below the visible rows takes the last row, with his real place
        let eigen_unten = z.iter().enumerate().skip(3 + platz.saturating_sub(1)).find(|(_, x)| x.1 == c.own).map(|(i, _)| i);
        let mut zeilen: Vec<usize> = (3..z.len()).take(platz).collect();
        if let (Some(i), Some(letzte)) = (eigen_unten, zeilen.last_mut()) { if !(3..3 + platz).contains(&i) { *letzte = i; } }
        for (j, &idx) in zeilen.iter().enumerate() {
            let (_, club, a, b) = &z[idx];
            let y = start_y + j as f32 * 24.0;
            if *club == c.own { rrect(r.x + 8.0, y - 17.0, r.w - 16.0, 23.0, 5.0, alpha(Theme::akzent(), 0.16)); }
            ui.text(&format!("{}.", idx + 1), r.x + 14.0, y, 14.0, Theme::gedaempft());
            let cl = &c.world.clubs[*club];
            ui.club_crest(r.x + 48.0, y - 5.0, 8.0, cl.id, hex(cl.c1), &cl.short);
            ui.text(&ui.fit(a, 14.0, r.w - 120.0), r.x + 62.0, y, 14.0, Theme::text());
            ui.text_right(b, r.x + r.w - 14.0, y, 14.0, Theme::text2());
        }
    };
    let leer = || t("ui.liga.noch_leer").to_string();
    // scorers and assists
    let mut tore = alle.clone();
    tore.sort_by_key(|&p| std::cmp::Reverse(ls[&p].goals));
    let satz = tore.first().filter(|&&p| ls[&p].goals > 0).map(|&p| crate::tf!("ui.liga.torjaeger_satz", c.world.players[p].name, ls[&p].goals)).unwrap_or_else(leer);
    liste(ui, 0, t("ui.seiten.torjaeger_der_liga"), Theme::gold(), tore.iter().map(|&p| (Some(p), c.world.players[p].team.max(0) as usize, if ls[&p].st.is_empty() { format!("{} ({})", c.world.players[p].name, kurz(p)) } else { format!("{} ({}) · xG {:.1}", c.world.players[p].name, kurz(p), crate::statistik::get(&ls[&p].st, crate::statistik::XG) as f32 / 1000.0) }, ls[&p].goals.to_string())).collect(), satz);
    let mut vorl = alle.clone();
    vorl.sort_by_key(|&p| std::cmp::Reverse(ls[&p].assists));
    let satz = vorl.first().filter(|&&p| ls[&p].assists > 0).map(|&p| crate::tf!("ui.liga.vorlagen_satz", c.world.players[p].name, ls[&p].assists)).unwrap_or_else(leer);
    liste(ui, 1, t("ui.liga.vorlagen"), Theme::akzent(), vorl.iter().map(|&p| (Some(p), c.world.players[p].team.max(0) as usize, format!("{} ({})", c.world.players[p].name, kurz(p)), ls[&p].assists.to_string())).collect(), satz);
    // grades (at least five)
    let mut noten: Vec<usize> = alle.iter().copied().filter(|&p| ls[&p].rn >= if l >= 1000 { 3 } else { 5 }).collect();
    noten.sort_by(|a, b| ls[b].note().partial_cmp(&ls[a].note()).unwrap());
    let satz = noten.first().map(|&p| crate::tf!("ui.liga.noten_satz", c.world.players[p].name, dez2(ls[&p].note()))).unwrap_or_else(|| t("ui.liga.noten_leer").to_string());
    liste(ui, 2, t("ui.seiten.beste_noten_der_liga"), hex(0xa78bfa), noten.iter().map(|&p| (Some(p), c.world.players[p].team.max(0) as usize, format!("{} ({})", c.world.players[p].name, kurz(p)), dez2(ls[&p].note()))).collect(), satz);
    // clean sheets per club (played league matches without a goal against)
    let fx: Vec<crate::world::Fixture> = match wo { Wo::Liga(x) => c.world.leagues[x].fixtures.clone(), Wo::Pokal(x) => c.world.cups[x].fx.iter().map(|y| y.f.clone()).collect() };
    let mut null: Vec<(usize, usize)> = teams.iter().map(|&t| (t, fx.iter().filter(|f| f.hg >= 0 && ((f.h == t && f.ag == 0) || (f.a == t && f.hg == 0))).count())).collect();
    null.sort_by_key(|x| std::cmp::Reverse(x.1));
    let satz = null.first().filter(|x| x.1 > 0).map(|x| crate::tf!("ui.liga.zu_null_satz", c.world.clubs[x.0].name, x.1)).unwrap_or_else(leer);
    liste(ui, 3, t("ui.liga.zu_null"), Theme::gut(), null.iter().map(|x| (None, x.0, c.world.clubs[x.0].name.clone(), x.1.to_string())).collect(), satz);
    // fair play: fewest yellow cards of the squad (red cards are counted in suspensions, not kept per player)
    let mut fair: Vec<(usize, u32)> = teams.iter().map(|&t| (t, c.world.clubs[t].squad.iter().map(|&p| c.world.players[p].yellow as u32).sum())).collect();
    fair.sort_by_key(|x| x.1);
    let satz = fair.first().map(|x| crate::tf!("ui.liga.fair_satz", c.world.clubs[x.0].name, x.1)).unwrap_or_else(leer);
    liste(ui, 4, t("ui.liga.fairness"), hex(0xf5d90a), fair.iter().map(|x| (None, x.0, c.world.clubs[x.0].name.clone(), x.1.to_string())).collect(), satz);
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
    crate::pages::auswahl_zeichnen(ui, v, &felder);
}

pub(super) fn vereine(ui: &mut Ui, v: &mut View, c: &mut Career) {
    // left a third: the league's clubs with crest and strength, the map below; right the club's profile (Fabian 22.11.:
    // more spectacular): a hero in the club's colours over its stadium, facts, coach and history, the squad by lines
    let h = hoehe() - Y0 - 16.0;
    let lw = (cbreite() * 0.34).max(300.0);
    let l = Rect::new(x0() + 4.0, Y0, lw, h);
    let r = Rect::new(x0() + lw + 16.0, Y0, cbreite() - lw - 12.0, h);
    // country and league to search in (Fabian 22.11.: not only the own league)
    let gruppen: Vec<(String, Vec<usize>)> = crate::pages::ligen_gruppen(c, |_| true);
    let liga = v.vereine_liga.unwrap_or(c.league());
    let felder = crate::pages::wettbewerb_felder(c, &gruppen, liga, Rect::new(l.x + 10.0, l.y + 10.0, l.w - 20.0, 36.0));
    if let Some((f, i)) = crate::pages::auswahl_vorher(ui, v, &felder) { let neu = crate::pages::wahl_umsetzen(&gruppen, liga, f, i); v.vereine_liga = Some(neu); v.sel_club = c.world.leagues[neu].teams.first().copied(); v.karte_off = (0.0, 0.0); v.karte_zoom = 1.0; }
    let liga = v.vereine_liga.unwrap_or(c.league());
    ui.card(l);
    let ts = c.world.leagues[liga].teams.clone();
    let kh = (l.h * 0.4).max(220.0);
    let kr = Rect::new(l.x + 8.0, l.y + l.h - kh - 8.0, l.w - 16.0, kh);
    let (mut zoom, mut off) = (v.karte_zoom, v.karte_off);
    if let Some(k) = crate::geo::karte(ui, kr, &c.world, &ts, Some(v.sel_club.unwrap_or(c.own)), &mut zoom, &mut off) { v.sel_club = Some(k); }
    v.karte_zoom = zoom; v.karte_off = off;
    // the list scrolls with the wheel when the league has more clubs than fit above the map
    let platz = ((kr.y - 4.0 - (l.y + 56.0)) / 28.0).max(1.0) as usize;
    let liste_r = Rect::new(l.x, l.y + 50.0, l.w, kr.y - l.y - 54.0);
    if liste_r.contains(ui.mouse) && ui.wheel != 0.0 { v.league_scroll = (v.league_scroll as i32 - ui.wheel.signum() as i32).clamp(0, ts.len().saturating_sub(platz) as i32) as usize; }
    v.league_scroll = v.league_scroll.min(ts.len().saturating_sub(platz));
    for (k, &t) in ts.iter().enumerate().skip(v.league_scroll) {
        let k = k - v.league_scroll;
        let yy = l.y + 56.0 + k as f32 * 28.0;
        if yy + 26.0 > kr.y - 4.0 { break; }
        let rr = Rect::new(l.x + 6.0, yy, l.w - 12.0, 26.0);
        if v.sel_club.unwrap_or(c.own) == t { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, alpha(Theme::akzent(), 0.16)); } else if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 6.0, Theme::erhoehung()); }
        let cl = &c.world.clubs[t];
        ui.club_crest(rr.x + 14.0, rr.y + 13.0, 10.0, cl.id, hex(cl.c1), &cl.short);
        ui.text(&ui.fit(&cl.name, 15.0, rr.w * 0.55), rr.x + 30.0, rr.y + 18.0, 15.0, Theme::text());
        ui.text_right(&ui.fit(&cl.manager, 13.0, rr.w * 0.3), rr.x + rr.w - 44.0, rr.y + 18.0, 13.0, Theme::text2());
        ui.pill(rr.x + rr.w - 20.0, rr.y + 13.0, crate::mensch::kaderstaerke(&c.world, t), 30.0);
        if ui.hit(rr) { v.sel_club = Some(t); }
    }
    let t = v.sel_club.unwrap_or(c.own);
    let cl = c.world.clubs[t].clone();
    // the club colour for the hero: the first colour, the second when the first is too light for white text, else darkened
    let farbe = { let (a, b) = (ui.wappen_farbe(cl.id).map(hex).unwrap_or(hex(cl.c1)), hex(cl.c1)); if crate::ui::kontrast(a, WHITE) >= 3.0 { a } else if crate::ui::kontrast(b, WHITE) >= 3.0 { b } else { crate::ui::with_light(a, 0.25, 0.8) } };
    ui.card(r);
    // the hero: stadium, gradient in the club colour, big crest, name and the line with league, strength and coach
    let hero = Rect::new(r.x, r.y, r.w, 200.0);
    let bild = ui.stadion_bild(hero, cl.id, &cl.name);
    gradient_h(hero.x, hero.y, hero.w, hero.h, alpha(farbe, if bild { 0.96 } else { 1.0 }), alpha(farbe, if bild { 0.25 } else { 0.75 }));
    draw_rectangle(hero.x, hero.y + hero.h - 5.0, hero.w, 5.0, hex(cl.c2));
    draw_circle(hero.x + 86.0, hero.y + 96.0, 62.0, alpha(WHITE, 0.92));
    ui.club_crest(hero.x + 86.0, hero.y + 96.0, 52.0, cl.id, hex(cl.c1), &cl.short);
    let weiss = hex(0xffffff);
    let nm = ui.fit(&cl.name, 36.0, hero.w - 200.0);
    ui.title(&nm, hero.x + 168.0, hero.y + 78.0, 36.0, weiss);
    let lg = &c.world.leagues[cl.league];
    let zeile = format!("{} · {} · {}", lg.name, stufe_wort(crate::mensch::kaderstaerke(&c.world, t)), cl.manager);
    ui.text(&ui.fit(&zeile, 17.0, hero.w - 200.0), hero.x + 168.0, hero.y + 106.0, 17.0, alpha(weiss, 0.9));
    // fact chips: founded, stadium, champion titles, place in the table, the coach's formation
    let rd = crate::realdaten::verein(&cl.name);
    let mut chips: Vec<String> = vec![];
    if let Some(rd) = rd { if let Some(g) = rd.gruendung { chips.push(crate::tf!("ui.verein.chip_gegruendet", g)); } if !rd.stadion.is_empty() { chips.push(rd.stadion.clone()); } if rd.meister > 0 { chips.push(crate::tf!("ui.real.meister", rd.meister)); } }
    chips.push(crate::tf!("ui.verein.chip_platz", c.world.rank_of(t)));
    if let Some(tr) = cl.trainer.as_ref().filter(|x| x.formation.len() == 11) { chips.push(crate::fl27::formation_name(&tr.formation)); }
    let mut cx = hero.x + 168.0;
    for ch in chips {
        let w = ui.measure(&ch, 14.0) + 20.0;
        if cx + w > hero.x + hero.w - 12.0 { break; }
        rrect(cx, hero.y + 128.0, w, 26.0, 13.0, alpha(BLACK, 0.35));
        ui.text(&ch, cx + 10.0, hero.y + 146.0, 14.0, weiss);
        cx += w + 8.0;
    }
    // below: left the coach and the history, right the squad by lines
    let sp = r.w * 0.44;
    let links = Rect::new(r.x + 16.0, hero.y + hero.h + 14.0, sp - 24.0, r.h - hero.h - 24.0);
    let rechts = Rect::new(r.x + sp + 8.0, hero.y + hero.h + 14.0, r.w - sp - 22.0, r.h - hero.h - 24.0);
    let mut y = links.y + 4.0;
    let ueberschrift = |ui: &Ui, t: &str, y: f32| { ui.text_semi(&t.to_uppercase(), links.x, y + 12.0, 13.0, farbe_lesbar(farbe)); draw_rectangle(links.x, y + 18.0, links.w, 2.0, alpha(farbe_lesbar(farbe), 0.5)); y + 30.0 };
    if let Some(tr) = cl.trainer.clone() {
        y = ueberschrift(ui, crate::inhalt::t("ui.verein.trainer"), y);
        let seit = match tr.seit.strip_prefix('S') { Some(s) => date::season_label(s.parse().unwrap_or(0)), None => tr.seit.clone() };
        ui.text_semi(&crate::tf!("ui.trainer.zeile", tr.name, tr.nat, crate::tf!("tb.seit", seit)), links.x, y + 12.0, 15.0, Theme::text());
        let tags: Vec<&str> = tr.tags.iter().take(5).map(|x| crate::inhalt::t(&format!("tb.tag.{x}"))).filter(|s| !s.is_empty()).collect();
        let zeile = if tr.formation.len() == 11 { format!("{} · {}", tags.join(" · "), crate::tf!("ui.trainer.formation", crate::fl27::formation_name(&tr.formation), crate::fl27::formation_name(&tr.ausweich))) } else { tags.join(" · ") };
        ui.text(&ui.fit(&zeile, 13.0, links.w), links.x, y + 32.0, 13.0, farbe_lesbar(farbe));
        y = if tr.stil.is_empty() { y + 44.0 } else { ui.paragraph(&tr.stil, links.x, y + 52.0, 13.0, links.w, Theme::text2()) + 8.0 };
    }
    if let Some(rd) = rd {
        y = ueberschrift(ui, crate::inhalt::t("ui.real.titel"), y);
        let mut zeilen: Vec<String> = vec![];
        if let Some(g) = rd.gruendung { zeilen.push(crate::tf!("ui.real.gruendung", g, rd.ort)); }
        if let Some((n, s)) = &rd.rekordspieler { zeilen.push(crate::tf!("ui.real.rekordspieler", n, s)); }
        if let Some((n, s)) = &rd.rekordtorschuetze { zeilen.push(crate::tf!("ui.real.rekordtorschuetze", n, s)); }
        if !rd.europa_frueher.is_empty() { zeilen.push(rd.europa_frueher.clone()); }
        if !rd.legenden.is_empty() { zeilen.push(crate::tf!("ui.real.legenden", rd.legenden.iter().take(6).map(|l| l.0.clone()).collect::<Vec<_>>().join(", "))); }
        for z in zeilen { if y > links.y + links.h - 40.0 { break; } y = ui.paragraph(&z, links.x, y + 12.0, 13.0, links.w, Theme::text2()) + 2.0; }
        // rivals with their crests when they are in this world
        if !rd.rivalen.is_empty() && y < links.y + links.h - 60.0 {
            y = ueberschrift(ui, crate::inhalt::t("ui.verein.rivalen"), y + 6.0);
            for (name, duell) in rd.rivalen.iter().take(4) {
                if y > links.y + links.h - 26.0 { break; }
                if let Some(k) = c.world.clubs.iter().position(|x| crate::trainerbank::norm(&x.name) == crate::trainerbank::norm(name)) { let rc = &c.world.clubs[k]; ui.club_crest(links.x + 10.0, y + 8.0, 9.0, rc.id, hex(rc.c1), &rc.short); }
                ui.text(&ui.fit(&format!("{name} · {duell}"), 14.0, links.w - 26.0), links.x + 26.0, y + 13.0, 14.0, Theme::text());
                y += 24.0;
            }
        }
    }
    // the squad by lines: keeper, defence, midfield, attack; photo, number, name, age, strength
    let mut sq = cl.squad.clone();
    sq.retain(|&p| c.world.players[p].youth == 0);
    sq.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
    let teile = [(crate::inhalt::t("aka.pos.tw"), 0usize), (crate::inhalt::t("achse.Abwehr"), 1), (crate::inhalt::t("gb2.vgl.mittel"), 2), (crate::inhalt::t("bericht.achse.angriff"), 3)];
    let mut y = rechts.y + 4.0;
    for (titel, g) in teile {
        let in_teil: Vec<usize> = sq.iter().copied().filter(|&p| part_of(&c.world.players[p].pos) == g).collect();
        if in_teil.is_empty() || y > rechts.y + rechts.h - 40.0 { continue; }
        ui.text_semi(&titel.to_uppercase(), rechts.x, y + 12.0, 13.0, farbe_lesbar(farbe));
        draw_rectangle(rechts.x, y + 18.0, rechts.w, 2.0, alpha(farbe_lesbar(farbe), 0.5));
        y += 26.0;
        for p in in_teil {
            if y + 24.0 > rechts.y + rechts.h { break; }
            let pl = &c.world.players[p];
            let rr = Rect::new(rechts.x - 4.0, y, rechts.w + 8.0, 24.0);
            if ui.hover(rr) { rrect(rr.x, rr.y, rr.w, rr.h, 4.0, Theme::erhoehung()); }
            ui.player_photo(rechts.x + 10.0, y + 12.0, 10.0, pl.id, &pl.name);
            ui.text_semi(&format!("{}", pl.no), rechts.x + 26.0, y + 17.0, 13.0, Theme::gedaempft());
            ui.pos_tag(rechts.x + 50.0, y + 12.0, pos_de(&pl.pos));
            ui.text(&ui.fit(&pl.name, 14.0, rechts.w - 214.0), rechts.x + 94.0, y + 17.0, 14.0, Theme::text());
            ui.text_right(&crate::tf!("as.ent.talent", pl.age, money(pl.value)), rechts.x + rechts.w - 40.0, y + 17.0, 13.0, Theme::text2());
            ui.pill(rechts.x + rechts.w - 16.0, y + 12.0, pl.ovr, 30.0);
            if ui.hit(rr) { v.modal = Modal::Player(p); }
            y += 25.0;
        }
        y += 6.0;
    }
    crate::pages::auswahl_zeichnen(ui, v, &felder);
}

pub(super) fn vereinswelt(ui: &mut Ui, _v: &mut View, c: &mut Career) {
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

pub(super) fn portal(ui: &mut Ui, v: &mut View, c: &mut Career) {
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
