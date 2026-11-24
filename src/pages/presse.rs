use super::*;

// ================================================================ Presse
pub(super) fn presse(ui: &mut Ui, v: &mut View, c: &mut Career) {
    tabs(ui, v, &[crate::inhalt::t("zentrale.k.schlagzeilen"), crate::inhalt::t("vorbereitung.pk")]);
    if v.tab == 1 || c.press.is_some() && v.tab == 0 && false { }
    if v.tab == 1 {
        press_card(ui, v, c, Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0));
        return;
    }
    if c.press.is_some() {
        let r = Rect::new(x0() + 4.0, Y0, cbreite(), 60.0);
        rrect(r.x, r.y, r.w, r.h, 8.0, alpha(Theme::achtung(), 0.12));
        ui.title(crate::inhalt::t("ui.pages.die_presse_wartet_auf_dich"), r.x + 16.0, r.y + 38.0, 24.0, Theme::achtung());
        if ui.button(Rect::new(r.x + r.w - 200.0, r.y + 12.0, 184.0, 36.0), crate::inhalt::t("ui.pages.zur_konferenz"), true) { v.tab = 1; ui.play(crate::id("pressefrage")); }
    }
    let y0 = if c.press.is_some() { Y0 + 72.0 } else { Y0 };
    webblog(ui, v, c, y0);
}

/// One outlet of the blog: (name, layout type, background, text, accent). `id` is "layout|name" (media pool) or an
/// old id of medien.toml [[webblog]]; each name gets its own house colour within the layout's palette.
fn medium_stil(id: &str) -> (String, String, Color, Color, Color) {
    if let Some((layout, name)) = id.split_once('|') {
        let h = name.bytes().fold(7u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
        let wahl = |v: &[u32]| hex(v[h as usize % v.len()]);
        let (g, f, a) = match layout {
            "datenportal" => (wahl(&[0x1a3151, 0x0b3d2e, 0x2b2d42, 0x14213d]), hex(0xffffff), wahl(&[0x5cc1e8, 0x3ddc97, 0xffb703])),
            "boulevard" => (wahl(&[0xe30613, 0xc1121f, 0xd00000]), hex(0xffffff), hex(0xffe600)),
            "tabloid_en" => (wahl(&[0xe4021b, 0x1d3557, 0xb5179e]), hex(0xffffff), hex(0x111111)),
            "tv" => (hex(0x111318), hex(0xffffff), wahl(&[0xe63946, 0x00b4d8, 0xffb703, 0x7bd389])),
            "podcast" => (wahl(&[0x5b2a86, 0x264653, 0x6a040f, 0x3a0ca3]), hex(0xffffff), hex(0xffd166)),
            "lokal" => (hex(0xf4efe3), hex(0x2b2118), wahl(&[0x8a5a2b, 0x2f5d50, 0x7a2e2e])),
            "verein" => (Theme::verein(), Theme::verein_auf(), Theme::gold()),
            "analyse" => (hex(0xfaf7f0), hex(0x121212), wahl(&[0xe8622c, 0x2a9d8f, 0x6d597a])),
            "serioes" | "serioes_en" => (wahl(&[0x052962, 0x1b263b, 0x283618, 0x3d405b]), hex(0xffffff), wahl(&[0xffe500, 0xe9c46a, 0xa8dadc])),
            _ => (hex(0xffffff), hex(0x111111), wahl(&[0xd0021b, 0x0077b6, 0x2a9d8f])),
        };
        let layout = if layout == "serioes" { "serioes_en" } else { layout };
        return (name.to_string(), layout.to_string(), g, f, a);
    }
    let m = crate::regeln::wert("medien", "webblog").as_array().and_then(|a| a.iter().find(|m| m["id"] == id)).cloned().unwrap_or_default();
    let f = |i: usize| hex(m["farben"].get(i).and_then(|x| x.as_u64()).unwrap_or(0x888888) as u32);
    (m["name"].as_str().unwrap_or(id).to_string(), m["typ"].as_str().unwrap_or(crate::id("fachmagazin")).to_string(), f(0), f(1), f(2))
}

/// One article card in the style of its outlet: data portal, sports magazine, tabloid, long read, English papers.
fn artikel_karte(ui: &mut Ui, r: Rect, n: &News, medium: &str, gross: bool) {
    let (name, typ, grund, schrift, akzent) = medium_stil(medium);
    let kopf = if gross { 34.0 } else { 24.0 };
    let hs = if gross { 30.0 } else { 19.0 };
    let datum = date::short(n.season, n.day);
    let (bg, fg, sub) = match typ.as_str() { "boulevard" | "tv" | "podcast" | "verein" => (grund, schrift, alpha(schrift, 0.8)), "datenportal" => (hex(0xf3f5f8), hex(0x1a1f2b), hex(0x55607a)), "analyse" | "lokal" => (grund, schrift, alpha(schrift, 0.7)), _ => (hex(0xffffff), hex(0x141414), hex(0x555555)) };
    rrect(r.x, r.y, r.w, r.h, 8.0, bg);
    if ui.hover(r) { rrect_line(r.x, r.y, r.w, r.h, 8.0, akzent); }
    let mut y = r.y;
    match typ.as_str() {
        // a data portal: a navy bar with the wordmark, the facts first
        "datenportal" => { rrect(r.x, r.y, r.w, kopf, 8.0, grund); draw_rectangle(r.x, r.y + kopf - 8.0, r.w, 8.0, grund); ui.text_semi(&name, r.x + 12.0, r.y + kopf * 0.68, kopf * 0.55, schrift); draw_rectangle(r.x, r.y + kopf, r.w, 3.0, akzent); y += kopf + 8.0; }
        // the tabloid: the wordmark in a box, the headline shouts in capitals
        "boulevard" => { let w = ui.measure_title(&name, kopf * 0.8) + 16.0; draw_rectangle(r.x + 12.0, r.y + 10.0, w, kopf, schrift); ui.title(&name, r.x + 20.0, r.y + 10.0 + kopf * 0.82, kopf * 0.8, grund); y += kopf + 16.0; }
        // the English tabloid: a red masthead
        "tabloid_en" => { rrect(r.x, r.y, r.w, kopf, 8.0, grund); draw_rectangle(r.x, r.y + kopf - 8.0, r.w, 8.0, grund); ui.title(&name, r.x + 12.0, r.y + kopf * 0.8, kopf * 0.75, schrift); y += kopf + 8.0; }
        // the serious English paper: navy head, yellow line
        "serioes_en" => { rrect(r.x, r.y, r.w, kopf, 8.0, grund); draw_rectangle(r.x, r.y + kopf - 8.0, r.w, 8.0, grund); ui.text_semi(&name, r.x + 12.0, r.y + kopf * 0.7, kopf * 0.6, schrift); draw_rectangle(r.x, r.y + kopf, r.w, 3.0, akzent); y += kopf + 8.0; }
        // TV: dark, a red live dot before the channel
        "tv" => { let puls = 0.6 + 0.4 * (get_time() as f32 * 3.0).sin().abs(); draw_circle(r.x + 20.0, r.y + 20.0, 6.0, alpha(akzent, puls)); ui.text_semi(&name, r.x + 34.0, r.y + 26.0, 16.0, schrift); ui.text_semi(crate::inhalt::t("ui.blog.tv"), r.x + r.w - 14.0 - ui.measure_semi(crate::inhalt::t("ui.blog.tv"), 12.0), r.y + 25.0, 12.0, akzent); y += 40.0; }
        // podcast and fan blog: a round "mic" badge
        "podcast" => { draw_circle(r.x + 24.0, r.y + 24.0, 12.0, akzent); draw_rectangle(r.x + 21.0, r.y + 16.0, 6.0, 11.0, grund); draw_rectangle(r.x + 23.0, r.y + 27.0, 2.0, 5.0, grund); ui.text_semi(&name, r.x + 44.0, r.y + 30.0, 16.0, schrift); y += 44.0; }
        // the local paper: old-fashioned masthead with a rule
        "lokal" => { ui.title(&name, r.x + 14.0, r.y + 30.0, 20.0, akzent); ui.text(crate::inhalt::t("ui.blog.lokales"), r.x + r.w - 14.0 - ui.measure(crate::inhalt::t("ui.blog.lokales"), 12.0), r.y + 26.0, 12.0, akzent); draw_rectangle(r.x + 14.0, r.y + 36.0, r.w - 28.0, 2.0, akzent); y += 44.0; }
        // the club's own channel: club colours
        "verein" => { ui.title(&name, r.x + 14.0, r.y + 30.0, 20.0, akzent); y += 40.0; }
        // the long read: quiet, a small orange "Analyse"
        "analyse" => { ui.text_semi(&name, r.x + 14.0, r.y + 24.0, 16.0, schrift); ui.text_semi(crate::inhalt::t("ui.blog.analyse"), r.x + r.w - 14.0 - ui.measure_semi(crate::inhalt::t("ui.blog.analyse"), 12.0), r.y + 22.0, 12.0, akzent); draw_rectangle(r.x + 14.0, r.y + 32.0, r.w - 28.0, 1.0, hex(0xcccccc)); y += 40.0; }
        // the sports magazine: a red line on top, the wordmark in red
        _ => { draw_rectangle(r.x + 6.0, r.y, r.w - 12.0, 4.0, akzent); ui.title(&name, r.x + 14.0, r.y + 30.0, 22.0, akzent); y += 38.0; }
    }
    let kopfzeile = if typ == "boulevard" || typ == "tabloid_en" { n.head.to_uppercase() } else { n.head.clone() };
    let hfarbe = if typ == "tabloid_en" { hex(0x111111) } else { fg };
    if typ == "tabloid_en" { draw_rectangle(r.x, y - 4.0, r.w, r.h - (y - 4.0 - r.y), hex(0xffffff)); }
    let max_z = if gross { 3 } else { 2 };
    for l in ui.wrap(&kopfzeile, hs, r.w - 28.0).into_iter().take(max_z) {
        if typ == "boulevard" { draw_rectangle(r.x + 12.0, y + hs + 2.0, ui.measure_title(&l, hs) + 6.0, 3.0, akzent); }
        if typ == "boulevard" || typ == "tabloid_en" || typ == "fachmagazin" { ui.title(&l, r.x + 14.0, y + hs, hs, hfarbe); } else { ui.text_semi(&l, r.x + 14.0, y + hs, hs, hfarbe); }
        y += hs + 6.0;
    }
    let tz = if gross { 4 } else { 2 };
    let tfarbe = if typ == "tabloid_en" { hex(0x333333) } else { sub };
    for l in ui.wrap(&n.text, 14.0, r.w - 28.0).into_iter().take(tz) { ui.text(&l, r.x + 14.0, y + 16.0, 14.0, tfarbe); y += 19.0; }
    ui.text(&datum, r.x + 14.0, r.y + r.h - 10.0, 12.0, tfarbe);
    // the outlet's real logo, when a free one exists (netzbilder.rs), bottom right
    ui.medium_logo(Rect::new(r.x + r.w - 74.0, r.y + r.h - 30.0, 64.0, 24.0), &name, crate::medien::pool_land(&name));
}

/// The media page as a web blog (Fabian 23.11.): filters, a lead story, the cards of the outlets, and a sidebar with
/// the most read, the rumour mill and the next press conference. Only what concerns us (nachricht_relevant).
fn webblog(ui: &mut Ui, v: &mut View, c: &mut Career, y0: f32) {
    let alle = c.feed.clone();
    let mut liste: Vec<(News, String)> = vec![];
    for n in alle { if c.nachricht_relevant(&n.head) { let m = if n.medium.is_empty() { c.medium_fuer(&n.head, &n.text).to_string() } else { n.medium.clone() }; liste.push((n, m)); } }
    let filter = [crate::id("alle"), crate::id("transfer"), crate::id("eigen"), crate::id("liga")];
    let fl: Vec<&str> = filter.iter().map(|f| crate::inhalt::t(&format!("ui.blog.filter.{f}"))).collect();
    if let Some(k) = ui.segment(Rect::new(x0() + 4.0, y0, 520.0, 32.0), &fl, v.blog_filter as usize) { v.blog_filter = k as u8; v.scrolls.clear(); }
    let own = c.club().name.clone();
    let liga: Vec<String> = c.world.leagues[c.league()].teams.iter().map(|&t| c.world.clubs[t].name.clone()).collect();
    liste.retain(|(n, m)| match v.blog_filter { 1 => matches!(m.as_str(), "transfermarkt" | "bild" | "sun") || m.starts_with("datenportal|") || m.starts_with("boulevard|") || m.starts_with("tabloid_en|") || n.head.contains("Gerücht"), 2 => n.head.contains(&own) || n.text.contains(&own), 3 => liga.iter().any(|l| n.head.contains(l.as_str())), _ => true });
    let y = y0 + 44.0;
    let hw = cbreite() * 0.66;
    let sw = cbreite() - hw - 14.0;
    if liste.is_empty() { ui.text(crate::inhalt::t("md.ui.leer.schlagzeilen"), x0() + 10.0, y + 30.0, 18.0, Theme::text2()); }
    // the lead story, then two columns of cards
    let mut yy = y;
    let area = Rect::new(x0() + 4.0, y, hw, hoehe() - y - 16.0);
    let zeilen = ((area.h - 250.0) / 186.0).max(1.0) as usize;
    let start = scroll_rows(ui, v, area, liste.len().saturating_sub(1).div_ceil(2), zeilen) * 2;
    if let Some((n, m)) = liste.first() { if start == 0 { artikel_karte(ui, Rect::new(x0() + 4.0, yy, hw, 236.0), n, m, true); yy += 250.0; } }
    let kw = (hw - 12.0) / 2.0;
    for (k, (n, m)) in liste.iter().skip(1 + start).take(zeilen * 2).enumerate() {
        artikel_karte(ui, Rect::new(x0() + 4.0 + (k % 2) as f32 * (kw + 12.0), yy + (k / 2) as f32 * 186.0, kw, 174.0), n, m, false);
    }
    // sidebar
    let sx = x0() + 4.0 + hw + 14.0;
    let r1 = Rect::new(sx, y, sw, 230.0);
    ui.card_titled(r1, crate::inhalt::t("ui.blog.meistgelesen"), Theme::akzent());
    let mut top: Vec<&(News, String)> = liste.iter().collect();
    top.sort_by_key(|(n, _)| std::cmp::Reverse((n.head.contains(&own) as u8, n.day)));
    for (k, (n, m)) in top.iter().take(5).enumerate() {
        let ty = r1.y + 62.0 + k as f32 * 33.0;
        ui.title(&format!("{}", k + 1), r1.x + 14.0, ty + 4.0, 22.0, Theme::akzent());
        ui.text(&ui.fit(&n.head, 14.0, r1.w - 60.0), r1.x + 40.0, ty - 2.0, 14.0, Theme::text());
        ui.text(&medium_stil(m).0, r1.x + 40.0, ty + 14.0, 11.0, Theme::gedaempft());
    }
    let r2 = Rect::new(sx, r1.y + r1.h + 12.0, sw, 220.0);
    ui.card_titled(r2, crate::inhalt::t("ui.blog.geruechte"), Theme::gold());
    let gw = |n: &News| ["Gerücht", "baggert", "Spur", "verhandelt", "macht Ernst"].iter().any(|w| n.head.contains(w)); // roh
    let ger: Vec<&(News, String)> = liste.iter().filter(|(n, _)| gw(n)).take(5).collect();
    if ger.is_empty() { ui.text(crate::inhalt::t("ui.blog.keine_geruechte"), r2.x + 14.0, r2.y + 62.0, 14.0, Theme::text2()); }
    for (k, (n, _)) in ger.iter().enumerate() {
        let ty = r2.y + 62.0 + k as f32 * 30.0;
        let heiss = (n.head.len() + n.day as usize) % 5;
        for f in 0..5 { draw_circle(r2.x + 18.0 + f as f32 * 9.0, ty - 5.0, 3.5, if f <= heiss { Theme::problem() } else { Theme::ueberlagerung() }); }
        ui.text(&ui.fit(&n.head, 13.0, r2.w - 80.0), r2.x + 66.0, ty, 13.0, Theme::text());
    }
    let r3 = Rect::new(sx, r2.y + r2.h + 12.0, sw, (hoehe() - r2.y - r2.h - 40.0).max(80.0));
    ui.card_titled(r3, crate::inhalt::t("ui.blog.pk"), Theme::akzent());
    let pk = if c.press.is_some() { crate::inhalt::t("ui.pages.die_presse_wartet_auf_dich").to_string() } else { c.next_match().map(|f| crate::tf!("ui.blog.pk_vor", date::short(c.world.season, f.day.saturating_sub(1)))).unwrap_or_default() };
    ui.paragraph(&pk, r3.x + 14.0, r3.y + 62.0, 14.0, r3.w - 28.0, Theme::text());
}

pub fn press_card(ui: &mut Ui, v: &mut View, c: &mut Career, r: Rect) {
    use crate::inhalt::t;
    ui.card_titled(r, t("vorbereitung.pk"), Theme::akzent());
    let Some(pq) = c.press.clone() else {
        ui.text(t("ui.pages.gerade_steht_keine_pressekonferenz_an"), r.x + 16.0, r.y + 60.0, 17.0, Theme::text2());
        return;
    };
    // the journalist as a person: drawn face, name, outlet, style and attitude (medien.rs, medienpool.json)
    let j = pq.journalist as usize;
    let (jn, stil, haltung) = c.me.journalisten.get(j).cloned().unwrap_or_else(|| (t("ui.medien.pressestelle").to_string(), 0, 0.0));
    let blatt = c.blatt(j);
    let stil_t = t(&format!("md.ui.journalist.stil.{}", crate::medien::STILE[(stil as usize).min(3)]));
    let halt_t = t(if haltung > 15.0 { "md.ui.journalist.haltung.gut" } else if haltung < -15.0 { "md.ui.journalist.haltung.schlecht" } else { "md.ui.journalist.haltung.neutral" });
    // the beats (Fabian 24.11.): what this journalist mostly covers
    let faecher: Vec<&str> = c.faecher(j).iter().map(|f| t(&format!("md.ui.journalist.fach.{f}"))).collect();
    let land = crate::medien::land_kuerzel(&c.world.leagues[c.league()].land);
    let (fx, fy, fr) = (r.x + 64.0, r.y + 100.0, 44.0);
    ui.presse_kopf(fx, fy, fr, &jn, crate::medien::pool_land(&jn));
    ui.text_semi(&jn, r.x + 124.0, r.y + 84.0, 20.0, Theme::text());
    // the outlet's logo next to the name when a free one exists
    let lx = r.x + 134.0 + ui.measure_semi(&jn, 20.0);
    if !blatt.is_empty() { ui.medium_logo(Rect::new(lx, r.y + 64.0, 70.0, 26.0), &blatt, land); }
    ui.text(&ui.fit(&format!("{} · {} · {}", if blatt.is_empty() { "–".to_string() } else { blatt.clone() }, stil_t, faecher.join(", ")), 15.0, r.w * 0.45), r.x + 124.0, r.y + 106.0, 15.0, Theme::text2());
    ui.text(halt_t, r.x + 124.0, r.y + 128.0, 14.0, if haltung > 15.0 { Theme::gut() } else if haltung < -15.0 { Theme::problem() } else { Theme::gedaempft() });
    // the needles: what the answer moves (media, board, fans, dressing room), current values
    let zeiger = [(t("ui.presse.medien"), c.media), (t("ui.presse.vorstand"), c.mensch.vorstand), (t("ui.presse.fans"), c.fans), (t("ui.presse.kabine"), c.mensch.teamgeist)];
    let zx = r.x + r.w * 0.58;
    for (i, (n, w)) in zeiger.iter().enumerate() {
        let y = r.y + 70.0 + i as f32 * 20.0;
        ui.text(n, zx, y, 13.0, Theme::text2());
        ui.bar(zx + 110.0, y - 9.0, r.w * 0.42 - 170.0, 8.0, (*w / 100.0).clamp(0.0, 1.0), Theme::skala(40.0 + *w * 0.59));
        ui.text_right(&format!("{w:.0}"), r.x + r.w - 16.0, y, 13.0, Theme::text());
    }
    // the press seats (Medien-Konzept 6.8): the journalists in a row, the one asking raised and ringed
    let reihe: Vec<(String, u8, f32)> = c.me.journalisten.iter().take(7).cloned().collect();
    for (k, (n, _, hal)) in reihe.iter().enumerate() {
        let fragt = k == j;
        let (sx, sy) = (r.x + 40.0 + k as f32 * 52.0, r.y + 176.0 - if fragt { 6.0 + 3.0 * (ui.time * 4.0).sin().abs() } else { 0.0 });
        if fragt { draw_circle(sx, sy, 22.0, alpha(Theme::akzent(), 0.35)); }
        ui.presse_kopf(sx, sy, if fragt { 18.0 } else { 15.0 }, n, crate::medien::pool_land(n));
        draw_circle(sx + 12.0, sy + 12.0, 4.0, if *hal > 15.0 { Theme::gut() } else if *hal < -15.0 { Theme::problem() } else { Theme::gedaempft() });
    }
    // the question as a speech bubble from the journalist
    let qy = r.y + 206.0;
    let zeilen = ui.wrap(&pq.q, 19.0, r.w - 64.0);
    let qh = 22.0 + zeilen.len() as f32 * 25.0;
    rrect(r.x + 16.0, qy, r.w - 32.0, qh, 12.0, Theme::erhoehung());
    draw_triangle(vec2(fx - 8.0, fy + fr - 2.0), vec2(fx + 14.0, fy + fr - 2.0), vec2(fx + 2.0, qy + 1.0), Theme::erhoehung());
    let mut yy = qy + 28.0;
    for z in &zeilen { ui.text(z, r.x + 32.0, yy, 19.0, Theme::text()); yy += 25.0; }
    // answers as tone cards: tone, the answer, its effect
    let mut y = qy + qh + 12.0;
    let spalten = if r.w > 900.0 { 2 } else { 1 };
    let kw = (r.w - 32.0 - (spalten as f32 - 1.0) * 10.0) / spalten as f32;
    let mut picked = None;
    for (i, (a, ton)) in pq.answers.iter().enumerate() {
        let (sx, sy) = (r.x + 16.0 + (i % spalten) as f32 * (kw + 10.0), y + (i / spalten) as f32 * 82.0);
        let kr = Rect::new(sx, sy, kw, 74.0);
        let hover = ui.hover(kr);
        rrect(kr.x, kr.y, kr.w, kr.h, 10.0, if hover { alpha(Theme::akzent(), 0.18) } else { Theme::erhoehung() });
        let key = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4][i.min(3)];
        let ton_n = t(&format!("md.ui.ton.{}", crate::medien::TOENE[(*ton % 10) as usize % crate::medien::TOENE.len()]));
        ui.text_semi(&format!("{}  {}", i + 1, ton_n), kr.x + 12.0, kr.y + 20.0, 14.0, Theme::akzent());
        ui.paragraph(&ui.fit(a, 15.0, (kr.w - 24.0) * 2.0), kr.x + 12.0, kr.y + 42.0, 15.0, kr.w - 24.0, Theme::text());
        let (m, b, f, mo) = tone_effect(*ton % 10);
        let eff = crate::tf!("ui.pages.medien_praesidium_fans_kabine", format!("{:+}", m), format!("{:+}", b), format!("{:+}", f), format!("{:+}", mo));
        ui.text_right(&eff, kr.x + kr.w - 12.0, kr.y + 20.0, 12.0, Theme::gedaempft());
        // an answer about one of our players: what it does to him, before choosing
        if let Some(p) = pq.bezug.filter(|&p| c.world.players[p].team == c.own as i32) {
            let tn = crate::medien::TOENE[(*ton % 10) as usize % crate::medien::TOENE.len()];
            let (mo, fo) = (crate::regeln::f_oder("medien", &format!("spieler_ton.{tn}.moral"), 0.0), crate::regeln::f_oder("medien", &format!("spieler_ton.{tn}.form"), 0.0));
            if mo != 0.0 || fo != 0.0 {
                let kurz = c.world.players[p].name.split_whitespace().last().unwrap_or("").to_string();
                ui.text_right(&crate::tf!("ui.medien.vorschau_spieler", kurz, format!("{mo:+.0}"), format!("{fo:+.1}")), kr.x + kr.w - 12.0, kr.y + kr.h - 8.0, 11.0, if mo >= 0.0 { Theme::gut() } else { Theme::problem() });
            }
        }
        if ui.hit(kr) || is_key_pressed(key) { picked = Some(i); }
    }
    y += ((pq.answers.len() + spalten - 1) / spalten) as f32 * 82.0;
    if ui.button(Rect::new(r.x + 16.0, (y + 4.0).min(r.y + r.h - 44.0), 220.0, 34.0), t("ui.pages.nicht_hingehen"), false) { c.answer_press(None); return; }
    if let Some(i) = picked {
        c.answer_press(Some(i));
        ui.play(crate::id("bestaetigt"));
        // what the answer did to the player it was about, at once (Fabian 24.11.)
        if !c.me.letzte_wirkung.is_empty() { let w = std::mem::take(&mut c.me.letzte_wirkung); v.toast(&w, true); }
    }
}

pub(super) fn zeitung(ui: &mut Ui, v: &mut View, c: &mut Career) {
    let area = Rect::new(x0() + 4.0, Y0, cbreite(), hoehe() - Y0 - 16.0);
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
