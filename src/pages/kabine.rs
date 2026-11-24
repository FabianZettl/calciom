use super::*;

/// Kabine (guide §7): mood bar, unhappy / happy columns with reason chips, hierarchy pyramid.
pub(super) fn kabine(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("chat.stimmung"), crate::inhalt::t("ui.pages.hierarchie"), crate::inhalt::t("ui.kabine.gruppen")]);
    if v.tab == 2 { return gruppen_seite(ui, v, c); }
    let sq = c.squad();
    let level = c.club().level;
    if v.tab == 0 {
        let happy: Vec<usize> = sq.iter().copied().filter(|&p| c.world.players[p].morale >= 50.0).collect();
        let sad: Vec<usize> = sq.iter().copied().filter(|&p| c.world.players[p].morale < 50.0).collect();
        let share = happy.len() as f32 / sq.len().max(1) as f32;
        let head = if share >= 0.8 { crate::inhalt::t("ui.pages.die_kabine_ist_gut_gelaunt") } else if share >= 0.6 { crate::inhalt::t("ui.pages.die_stimmung_ist_ordentlich") } else if share >= 0.4 { crate::inhalt::t("ui.pages.die_kabine_ist_angespannt") } else { crate::inhalt::t("ui.pages.in_der_kabine_brodelt_es") };
        ui.title(head, x0() + 8.0, Y0 + 26.0, 28.0, Theme::text());
        let bw = cbreite() - 8.0;
        rrect(x0() + 4.0, Y0 + 40.0, bw, 12.0, 6.0, Theme::problem());
        rrect(x0() + 4.0 + bw * (1.0 - share), Y0 + 40.0, bw * share, 12.0, 6.0, Theme::gut());
        ui.text(&crate::tf!("kabine.neu.unzufrieden", sad.len()), x0() + 8.0, Y0 + 72.0, 15.0, Theme::problem());
        ui.text_right(&crate::tf!("kabine.neu.zufrieden", happy.len()), x0() + cbreite() - 4.0, Y0 + 72.0, 15.0, Theme::gut());
        let colw = (cbreite() - 12.0) / 2.0;
        let today = c.abs();
        for (k, list) in [&sad, &happy].iter().enumerate() {
            let r = Rect::new(x0() + 4.0 + k as f32 * (colw + 12.0), Y0 + 86.0, colw, hoehe() - Y0 - 100.0);
            ui.card_titled(r, if k == 0 { crate::inhalt::t("ui.pages.braucht_ein_gespraech") } else { crate::inhalt::t("status.zufrieden") }, if k == 0 { Theme::problem() } else { Theme::gut() });
            let vis = ((r.h - 46.0) / 62.0) as usize;
            if list.is_empty() { ui.empty(Rect::new(r.x, r.y + 36.0, r.w, r.h - 36.0), if k == 0 { crate::inhalt::t("ui.pages.keiner_murrt_weiter_so") } else { crate::inhalt::t("ui.pages.gerade_ist_niemand_richtig_zufrieden") }); }
            for (j, &p) in list.iter().take(vis).enumerate() {
                let pl = c.world.players[p].clone();
                let y = r.y + 44.0 + j as f32 * 62.0;
                ui.player_photo_nr(r.x + 34.0, y + 28.0, 20.0, pl.id, &pl.name, pl.no, Theme::verein());
                ui.smiley(r.x + 52.0, y + 42.0, 8.0, pl.morale);
                ui.text_semi(&pl.name, r.x + 70.0, y + 22.0, 17.0, Theme::text());
                // Grund-Chips: Spielzeit, Gehalt, Rang; the role in the hierarchy first (Fabian 24.11.: make it felt)
                let mut chips: Vec<(&str, Color)> = vec![];
                if pl.ps.an && pl.ps.rolle <= 2 { chips.push((crate::inhalt::t(&format!("ui.kabine.rolle.{}", pl.ps.rolle)), Theme::gold())); }
                if pl.benched >= 3 { chips.push((crate::inhalt::t("hierarchie.teil.spielzeit"), Theme::problem())); } else if pl.apps > 0 { chips.push((crate::inhalt::t("ui.kabine.spielt"), Theme::gut())); }
                let fair = fair_wage(&pl, c.world.leagues[c.league()].level);
                if pl.wage < fair * 0.8 { chips.push((crate::inhalt::t("profil.gehalt"), Theme::achtung())); }
                if pl.ovr >= level + 3.0 { chips.push((crate::inhalt::t("ui.pages.leistungstraeger"), hex(0xa78bfa))); } else if pl.ovr < level - 6.0 { chips.push((crate::inhalt::t("pille.ergaenzung"), Theme::gedaempft())); }
                let mut cx = r.x + 70.0;
                for (t, col) in chips {
                    let w = ui.measure_semi(t, 13.0) + 18.0;
                    rrect(cx, y + 32.0, w, 20.0, 10.0, alpha(col, 0.18));
                    ui.text_semi(t, cx + 9.0, y + 46.0, 13.0, col);
                    cx += w + 6.0;
                }
                if k == 0 {
                    let can = today >= pl.talk_day + 7 || pl.talk_day == 0;
                    if can && ui.button(Rect::new(r.x + r.w - 130.0, y + 12.0, 116.0, 32.0), crate::inhalt::t("story.ansicht.gespraech"), false) {
                        // entwicklung/kabine.py:87: an encouraging word, 30 % now and 40 % after 3 days
                        c.kabinen_gespraech(p, crate::id("ermutigen"));
                        v.toast(&crate::tf!("ui.pages.fuehlt_sich_gehoert", pl.name), true);
                    }
                    if !can { ui.text_right(crate::inhalt::t("vg.ausgang.kurz.abgeschlossen"), r.x + r.w - 14.0, y + 32.0, 14.0, Theme::text2()); }
                    // with a car the coach can visit him at home (Fabian 24.11., privat.rs)
                    if (c.privat.auto > 0 || c.privat.motorrad) && ui.button(Rect::new(r.x + r.w - 252.0, y + 12.0, 116.0, 32.0), crate::inhalt::t("ui.privat.besuch"), false) {
                        match c.hausbesuch(p) { Ok(m) => v.toast(&m, true), Err(e) => v.toast(&e, false) }
                    }
                }
            }
        }
    } else {
        // Hierarchie-Pyramide: captain on top, then leaders, regulars, the rest (rank by strength and age)
        let mut ranked = sq.clone();
        ranked.sort_by(|a, b| {
            let w = |p: usize| { let x = &c.world.players[p]; x.ovr + (x.age as f32 - 24.0).clamp(0.0, 8.0) * 0.8 };
            w(*b).partial_cmp(&w(*a)).unwrap()
        });
        let tiers: [(&str, usize); 4] = [("Kapitän", 1), (crate::inhalt::t("hierarchie.rolle.fuehrungsspieler"), 3), (crate::inhalt::t("ui.pages.stammkraefte"), 7), (crate::inhalt::t("ui.pages.ergaenzung_und_talente"), 12)]; // roh
        let mut idx = 0;
        let mut y = Y0 + 10.0;
        for (t, n) in tiers {
            let take: Vec<usize> = ranked.iter().skip(idx).take(n).copied().collect();
            idx += n;
            let rowh = 112.0;
            let total = take.len() as f32 * 92.0;
            let lx = x0() + cbreite() / 2.0 - total / 2.0;
            let pw = total + 40.0;
            rrect(x0() + cbreite() / 2.0 - pw / 2.0, y, pw, rowh - 10.0, 18.0, alpha(Theme::verein(), 0.10 + 0.06 * (4 - n.min(4)) as f32));
            ui.text_semi(t, x0() + cbreite() / 2.0 - pw / 2.0 + 14.0, y + 18.0, 13.0, Theme::text2());
            for (j, &p) in take.iter().enumerate() {
                let pl = &c.world.players[p];
                let cx = lx + 46.0 + j as f32 * 92.0;
                ui.player_photo_nr(cx, y + 50.0, 22.0, pl.id, &pl.name, pl.no, if t == "Kapitän" { Theme::gold() } else { Theme::verein() }); // roh
                ui.smiley(cx + 18.0, y + 64.0, 7.0, pl.morale);
                let last = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
                let nm = ui.fit(&last, 14.0, 88.0);
                ui.text_center(&nm, cx, y + 92.0, 14.0, Theme::text());
                if ui.hit(Rect::new(cx - 40.0, y + 20.0, 80.0, 80.0)) { v.modal = Modal::Player(p); }
            }
            y += rowh;
        }
    }
}

/// Groups and topics (Fabian 24.11.: who belongs together, what is on the players' minds, the hierarchy felt): the
/// countrymen, the closest pairs and the tensions from the dressing-room relations, and the topics with their players.
fn gruppen_seite(ui: &mut Ui, v: &mut View, c: &mut Career) {
    use crate::inhalt::t;
    let sq: Vec<usize> = c.squad().into_iter().filter(|&p| c.world.players[p].youth == 0).collect();
    let (l, r) = links_rechts();
    ui.card_titled(l, t("ui.kabine.wer_zusammen"), Theme::akzent());
    let mut y = l.y + 64.0;
    let reihe = |ui: &mut Ui, v: &mut View, y: f32, titel: &str, leute: &[usize], farbe: Color| {
        ui.text_semi(&ui.fit(titel, 14.0, l.w * 0.34), l.x + 16.0, y + 4.0, 14.0, farbe);
        for (k, &p) in leute.iter().take(8).enumerate() {
            let x = l.x + l.w * 0.38 + k as f32 * 40.0;
            let pl = &c.world.players[p];
            ui.player_photo(x, y, 16.0, pl.id, &pl.name);
            if pl.ps.rolle <= 2 && pl.ps.an { draw_circle_lines(x, y, 18.0, 2.0, Theme::gold()); }
            if Rect::new(x - 16.0, y - 16.0, 32.0, 32.0).contains(ui.mouse) { *ui.tooltip.borrow_mut() = Some(pl.name.clone()); }
            if ui.hit(Rect::new(x - 16.0, y - 16.0, 32.0, 32.0)) { v.modal = Modal::Player(p); }
        }
    };
    // countrymen: a group from two
    let mut laender: std::collections::BTreeMap<u16, Vec<usize>> = Default::default();
    for &p in &sq { let n = c.world.players[p].nat; if n > 0 { laender.entry(n).or_default().push(p); } }
    let mut gruppen: Vec<(u16, Vec<usize>)> = laender.into_iter().filter(|x| x.1.len() >= 2).collect();
    gruppen.sort_by_key(|x| std::cmp::Reverse(x.1.len()));
    ui.text(t("ui.kabine.landsleute"), l.x + 16.0, y, 13.0, Theme::gedaempft());
    y += 26.0;
    for (n, leute) in gruppen.iter().take(5) {
        reihe(ui, v, y, &crate::nationen::name(&c.world, *n), leute, Theme::text());
        y += 42.0;
    }
    // the closest pairs and the tensions
    let mut paare: Vec<(f32, usize, usize)> = vec![];
    for (i, &a) in sq.iter().enumerate() { for &b in &sq[i + 1..] { let w = c.bez_wert(c.world.players[a].id, c.world.players[b].id); if w.abs() >= 1.0 { paare.push((w, a, b)); } } }
    paare.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    y += 8.0;
    ui.text(t("ui.kabine.freunde"), l.x + 16.0, y, 13.0, Theme::gedaempft());
    y += 26.0;
    for &(_, a, b) in paare.iter().filter(|x| x.0 > 0.0).take(3) { reihe(ui, v, y, &format!("{} & {}", kurz(c, a), kurz(c, b)), &[a, b], Theme::gut()); y += 42.0; }
    y += 8.0;
    ui.text(t("ui.kabine.spannungen"), l.x + 16.0, y, 13.0, Theme::gedaempft());
    y += 26.0;
    let streit: Vec<&(f32, usize, usize)> = paare.iter().rev().filter(|x| x.0 < 0.0).take(3).collect();
    if streit.is_empty() { ui.text(t("ui.kabine.keine_spannungen"), l.x + 16.0, y, 14.0, Theme::text2()); }
    for &&(_, a, b) in &streit { reihe(ui, v, y, &format!("{} – {}", kurz(c, a), kurz(c, b)), &[a, b], Theme::problem()); y += 42.0; }
    // right: what is on their minds, with the players
    ui.card_titled(r, t("ui.kabine.themen"), Theme::gold());
    let season = c.world.season;
    let themen: Vec<(&str, Color, Vec<usize>)> = vec![
        (t("ui.kabine.thema.spielzeit"), Theme::problem(), sq.iter().copied().filter(|&p| c.world.players[p].benched >= 3).collect()),
        (t("ui.kabine.thema.vertrag"), Theme::achtung(), sq.iter().copied().filter(|&p| c.world.players[p].contract_end <= season).collect()),
        (t("ui.kabine.thema.wechsel"), hex(0xa78bfa), sq.iter().copied().filter(|&p| c.world.players[p].ps.wunsch || c.world.players[p].listed).collect()),
        (t("ui.kabine.thema.stimmung"), Theme::problem(), sq.iter().copied().filter(|&p| c.world.players[p].morale < 40.0).collect()),
        (t("ui.kabine.thema.verletzt"), Theme::achtung(), sq.iter().copied().filter(|&p| c.world.players[p].inj > 0).collect()),
        (t("ui.kabine.thema.form"), Theme::gut(), sq.iter().copied().filter(|&p| crate::world::form_zuschlag(&c.world.players[p]) >= 2.0).collect()),
    ];
    let mut y = r.y + 70.0;
    for (titel, farbe, leute) in themen {
        if leute.is_empty() { continue; }
        rrect(r.x + 14.0, y - 18.0, 8.0, 36.0, 4.0, farbe);
        ui.text_semi(&format!("{titel} · {}", leute.len()), r.x + 32.0, y - 2.0, 15.0, Theme::text());
        for (k, &p) in leute.iter().take(9).enumerate() {
            let x = r.x + r.w * 0.5 + k as f32 * 34.0;
            let pl = &c.world.players[p];
            ui.player_photo(x, y - 6.0, 14.0, pl.id, &pl.name);
            if Rect::new(x - 14.0, y - 20.0, 28.0, 28.0).contains(ui.mouse) { *ui.tooltip.borrow_mut() = Some(pl.name.clone()); }
            if ui.hit(Rect::new(x - 14.0, y - 20.0, 28.0, 28.0)) { v.modal = Modal::Player(p); }
        }
        y += 54.0;
    }
}
fn kurz(c: &Career, p: usize) -> String { c.world.players[p].name.split(' ').last().unwrap_or("").to_string() }
