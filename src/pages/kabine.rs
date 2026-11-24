use super::*;

/// Kabine (guide §7): mood bar, unhappy / happy columns with reason chips, hierarchy pyramid.
pub(super) fn kabine(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("chat.stimmung"), crate::inhalt::t("ui.pages.hierarchie")]);
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
                // Grund-Chips: Spielzeit, Gehalt, Rang
                let mut chips: Vec<(&str, Color)> = vec![];
                if pl.benched >= 3 { chips.push((crate::inhalt::t("hierarchie.teil.spielzeit"), Theme::problem())); } else if pl.apps > 0 { chips.push((crate::id("spielt"), Theme::gut())); }
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
