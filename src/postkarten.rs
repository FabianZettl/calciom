// ---------------------------------------------------------------- Posteingang: eigenes Layout je Art (Python: VorgangSteckbrief,
// VorgangWochenbericht, VorgangVertragsblatt, VorgangSzene, VorgangSitzung, Meldungskarte, Scoutkarte, Ergebnischip, Wertring,
// Minidiagramm). Jede Art baut eine Liste von Bausteinen aus echten Spielstanddaten; ein Zeichner stellt sie dar.

/// The kind of a message: colour, symbol, layout.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Art { Scout, Woche, Spiel, Angebot, Gespraech, Vorstand, Verletzung, Presse, Talent, Meilenstein, Allgemein, Gegner }
pub const ARTEN: [Art; 12] = [Art::Scout, Art::Woche, Art::Spiel, Art::Angebot, Art::Gespraech, Art::Vorstand, Art::Verletzung, Art::Presse, Art::Talent, Art::Meilenstein, Art::Allgemein, Art::Gegner];

impl Art {
    /// The id stored in Msg::art.
    pub fn id(self) -> &'static str {
        match self { Art::Scout => crate::id("scout"), Art::Woche => crate::id("woche"), Art::Spiel => crate::id("spiel"), Art::Angebot => crate::id("angebot"), Art::Gespraech => crate::id("gespraech"),
            Art::Vorstand => crate::id("vorstand"), Art::Verletzung => crate::id("verletzung"), Art::Presse => crate::id("presse"), Art::Talent => crate::id("talent"), Art::Meilenstein => crate::id("meilenstein"), Art::Allgemein => crate::id("allgemein"), Art::Gegner => crate::id("gegner") }
    }
    pub fn name(self) -> &'static str { crate::inhalt::t(&format!("ui.post.art.{}", self.id())) }
    /// The colour of the area (Bereichsfarbe).
    pub fn farbe(self) -> Color {
        hex(match self { Art::Scout => 0x3BA3DA, Art::Woche => 0x6C5CE7, Art::Spiel => 0x27AE60, Art::Angebot => 0xE0A526, Art::Gespraech => 0xE67E22, Art::Vorstand => 0x8E44AD,
            Art::Verletzung => 0xE04A4A, Art::Presse => 0x5D7A99, Art::Talent => 0x16A99A, Art::Meilenstein => 0xC9A227, Art::Allgemein => 0x7F8C8D, Art::Gegner => 0xC0392B })
    }
    /// The inbox filter tab of this kind (1 = must decide is separate): 2 match, 3 squad, 4 market, 5 club, 6 media.
    pub fn gruppe(self) -> usize {
        match self { Art::Spiel | Art::Woche | Art::Meilenstein | Art::Gegner => 2, Art::Gespraech | Art::Verletzung | Art::Talent => 3, Art::Scout | Art::Angebot => 4, Art::Vorstand | Art::Allgemein => 5, Art::Presse => 6 }
    }
}

/// The kind of a message: the stored one, else from its action and area.
pub fn art_von(m: &Msg) -> Art {
    if let Some(a) = ARTEN.iter().copied().find(|a| a.id() == m.art) { return a; }
    match &m.action {
        Action::Report { .. } => Art::Scout,
        Action::TransferOffer { .. } | Action::JobOffer { .. } | Action::Expiring { .. } => Art::Angebot,
        Action::Complaint { .. } | Action::Wahl { .. } => Art::Gespraech,
        Action::Graduate { .. } => Art::Talent,
        Action::None => match m.cat { 5 => Art::Vorstand, 4 => Art::Presse, _ => Art::Allgemein },
    }
}

/// A simple drawn symbol of the kind (no font glyphs needed).
fn art_symbol(art: Art, cx: f32, cy: f32, r: f32, col: Color) {
    let l = 2.0;
    match art {
        Art::Scout => { draw_circle_lines(cx - r * 0.45, cy, r * 0.42, l, col); draw_circle_lines(cx + r * 0.45, cy, r * 0.42, l, col); }
        Art::Woche => { draw_rectangle_lines(cx - r * 0.8, cy - r * 0.7, r * 1.6, r * 1.5, l, col); draw_line(cx - r * 0.8, cy - r * 0.25, cx + r * 0.8, cy - r * 0.25, l, col); }
        Art::Spiel => { draw_circle_lines(cx, cy, r * 0.8, l, col); draw_poly(cx, cy, 5, r * 0.32, 0.0, col); }
        Art::Angebot | Art::Presse => { draw_rectangle_lines(cx - r * 0.6, cy - r * 0.8, r * 1.2, r * 1.6, l, col); for k in 0..3 { let y = cy - r * 0.35 + k as f32 * r * 0.35; draw_line(cx - r * 0.35, y, cx + r * 0.35, y, l, col); } }
        Art::Gespraech => { draw_rectangle_lines(cx - r * 0.85, cy - r * 0.6, r * 1.7, r * 1.0, l, col); draw_triangle(vec2(cx - r * 0.4, cy + r * 0.4), vec2(cx - r * 0.1, cy + r * 0.4), vec2(cx - r * 0.55, cy + r * 0.85), col); }
        Art::Vorstand => { draw_triangle(vec2(cx - r * 0.9, cy - r * 0.35), vec2(cx + r * 0.9, cy - r * 0.35), vec2(cx, cy - r * 0.9), col); for k in 0..3 { draw_rectangle(cx - r * 0.65 + k as f32 * r * 0.55, cy - r * 0.25, r * 0.22, r * 0.85, col); } }
        Art::Verletzung => { draw_rectangle(cx - r * 0.2, cy - r * 0.7, r * 0.4, r * 1.4, col); draw_rectangle(cx - r * 0.7, cy - r * 0.2, r * 1.4, r * 0.4, col); }
        Art::Talent | Art::Meilenstein => {
            let pt = |k: usize| { let a = -std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 5.0; let rr = if k % 2 == 0 { r * 0.85 } else { r * 0.38 }; vec2(cx + a.cos() * rr, cy + a.sin() * rr) };
            for k in 0..10 { draw_triangle(vec2(cx, cy), pt(k), pt((k + 1) % 10), col); }
        }
        Art::Gegner => { draw_circle_lines(cx, cy, r * 0.75, l, col); draw_line(cx - r, cy, cx + r, cy, l, col); draw_line(cx, cy - r, cx, cy + r, l, col); }
        Art::Allgemein => { draw_rectangle_lines(cx - r * 0.8, cy - r * 0.55, r * 1.6, r * 1.1, l, col); draw_line(cx - r * 0.8, cy - r * 0.55, cx, cy + r * 0.1, l, col); draw_line(cx + r * 0.8, cy - r * 0.55, cx, cy + r * 0.1, l, col); }
    }
}

/// One building block of a message layout.
#[derive(Clone, Debug)]
pub enum B {
    Foto { pid: usize, r: f32 },
    Wappen { club: usize },
    Gesicht { name: String },
    Zahl { titel: String, wert: String },
    Balken { titel: String, anteil: f32, text: String, farbe: u8 },
    Sterne { titel: String, von: f32, bis: f32 },
    Ring { titel: String, anteil: f32, text: String },
    Diagramm { titel: String, werte: Vec<f32> },
    Radar { namen: Vec<String>, werte: Vec<f32> },
    Chips { gut: Vec<String>, schlecht: Vec<String> },
    Zitat { pid: Option<usize>, name: String, text: String },
    Anzeige { h: usize, a: usize, hg: i8, ag: i8 },
    Ergebnisse(Vec<(usize, usize, i8, i8)>),
    Tabelle(Vec<(usize, usize, i32, i32)>),
    Transfer { pid: usize, nach: usize, wert: f64 },
    Pillen(Vec<(usize, f32)>),
    Zeitung { name: String, schlagzeile: String, text: String },
    Siegel { titel: String },
    Phasen(Vec<(String, bool)>),
    Titel(String),
    Text(String),
    /// Player tiles: photo, name, a big number (change or grade), its colour by sign, a line under it.
    Kacheln(Vec<(usize, String, Color, String)>),
    /// The scout's verdict as a banner: level 0–5 (Sofort holen … Nicht holen), the sentence, the certainty.
    Urteil { stufe: usize, satz: String, sicher: String },
    /// The own players of his position with the candidate among them: (player, role strength, is the candidate, note).
    Rang(Vec<(usize, f32, bool, String)>),
    /// A team of the week on a pitch: (player, grade) in the order keeper, four defenders, three midfielders, three forwards.
    Elf(Vec<(usize, f32)>),
    Reihe(Vec<B>),
}
impl B {
    /// Shows a picture (photo, crest, face).
    fn bild(&self) -> bool {
        match self { B::Foto { .. } | B::Wappen { .. } | B::Gesicht { .. } | B::Anzeige { .. } | B::Ergebnisse(_) | B::Transfer { .. } | B::Siegel { .. } | B::Zitat { .. } => true,
            B::Reihe(v) => v.iter().any(|x| x.bild()), _ => false }
    }
    /// Shows a number or a graphic.
    fn zahl(&self) -> bool {
        match self { B::Zahl { .. } | B::Balken { .. } | B::Sterne { .. } | B::Ring { .. } | B::Diagramm { .. } | B::Radar { .. } | B::Anzeige { .. } | B::Ergebnisse(_) | B::Tabelle(_)
            | B::Pillen(_) | B::Kacheln(_) | B::Elf(_) | B::Transfer { .. } | B::Phasen(_) => true, B::Reihe(v) => v.iter().any(|x| x.zahl()), _ => false }
    }
}
pub fn hat_bild(bs: &[B]) -> bool { bs.iter().any(|b| b.bild()) }
pub fn hat_zahl(bs: &[B]) -> bool { bs.iter().any(|b| b.zahl()) }

fn spieler_spanne(c: &Career, p: usize) -> f32 { if c.gehoert_uns(p) { 0.0 } else { c.spanne(p) / 2.0 } }
fn verein_von(c: &Career, p: usize) -> Option<usize> { let t = c.world.players[p].team; (t >= 0).then_some(t as usize) }

/// The players a message is about (stored ones, its action, the player it names).
pub fn beteiligte(c: &Career, m: &Msg) -> Vec<usize> {
    let mut v = m.spieler.clone();
    match &m.action {
        Action::TransferOffer { pid, .. } | Action::Complaint { pid } | Action::Expiring { pid } | Action::Report { pid } | Action::Graduate { pid } => v.push(*pid),
        Action::Wahl { bezug, .. } => v.extend(bezug.iter().copied()),
        _ => {}
    }
    if v.is_empty() { if let Some(p) = msg_spieler(c, m) { v.push(p); } }
    v.retain(|&p| p < c.world.players.len());
    v.dedup();
    v
}
/// The clubs a message is about.
pub fn vereine_von(c: &Career, m: &Msg) -> Vec<usize> {
    let mut v = m.vereine.clone();
    match &m.action { Action::TransferOffer { club, .. } | Action::JobOffer { club, .. } => v.push(*club), _ => {} }
    for p in beteiligte(c, m) { if let Some(k) = verein_von(c, p) { v.push(k); } }
    v.dedup();
    v.retain(|&k| k < c.world.clubs.len());
    v
}

/// Players and clubs a layout shows (photos, transfers, quotes; crests, score boards, results), for the context column.
fn b_bezug(bs: &[B], sp: &mut Vec<usize>, ve: &mut Vec<usize>) {
    for b in bs {
        match b {
            B::Foto { pid, .. } | B::Zitat { pid: Some(pid), .. } => sp.push(*pid),
            B::Transfer { pid, nach, .. } => { sp.push(*pid); ve.push(*nach); }
            B::Wappen { club } => ve.push(*club),
            B::Anzeige { h, a, .. } => { ve.push(*h); ve.push(*a); }
            B::Reihe(v) => b_bezug(v, sp, ve),
            _ => {}
        }
    }
}
/// The building blocks of a message, from the game state.
/// Who speaks for a sender: the person of an office (the president for „Präsidium“ …), else the sender itself.
pub fn sprecher(c: &Career, from: &str) -> String { crate::pool::absender_person(c.club().id, from).map(|x| x.0).unwrap_or_else(|| from.to_string()) }
pub fn bausteine(c: &Career, m: &Msg) -> Vec<B> {
    let t = |k: &str| crate::inhalt::t(&format!("ui.post.{k}")).to_string();
    let wer = beteiligte(c, m);
    let erst = wer.first().copied();
    // picture cards (Fabian 23.11.: no bare lists): players as tiles with their number
    // a transfer between two clubs: the move as a card, the player's figures and his last grades, then the text
    if m.vereine.len() == 2 && m.karte.is_empty() {
        if let Some(&p) = m.spieler.first().filter(|&&p| p < c.world.players.len()) {
            let pl = &c.world.players[p];
            let mut v = vec![B::Transfer { pid: p, nach: m.vereine[1], wert: m.werte.first().copied().unwrap_or(0.0) as f64 }];
            let (lo, hi) = c.ovr_range(p);
            v.push(B::Reihe(vec![B::Zahl { titel: t("staerke"), wert: if hi - lo <= 1.0 { format!("{:.0}", (lo + hi) / 2.0) } else { format!("{lo:.0}–{hi:.0}") } }, B::Zahl { titel: t("alter"), wert: pl.age.to_string() },
                B::Zahl { titel: t("position"), wert: pos_de(&pl.pos).to_string() }, B::Zahl { titel: t("marktwert"), wert: money(pl.value) }]));
            if pl.last.len() >= 2 { v.push(B::Diagramm { titel: t("letzte_noten"), werte: pl.last.iter().map(|n| n * 10.0).collect() }); }
            v.push(B::Text(m.body.clone()));
            return v;
        }
    }
    if m.karte == "elf" && m.spieler.len() == 11 {
        let elf: Vec<(usize, f32)> = m.spieler.iter().copied().zip(m.werte.iter().copied().chain(std::iter::repeat(0.0))).filter(|x| x.0 < c.world.players.len()).collect();
        return vec![B::Elf(elf), B::Text(m.body.clone())];
    }
    if !m.karte.is_empty() && !m.spieler.is_empty() {
        let kacheln: Vec<(usize, String, Color, String)> = m.spieler.iter().zip(m.werte.iter().chain(std::iter::repeat(&0.0))).filter(|(p, _)| **p < c.world.players.len()).take(9).map(|(&p, &w)| {
            let pl = &c.world.players[p];
            match m.karte.as_str() {
                "ehrung" => (p, format!("{w:.1}"), Theme::gold(), crate::tf!("ui.post.karte_ehrung", pos_de(&pl.pos), pl.goals)),
                _ => (p, format!("{:+.0}", w).replace('-', "−"), if w >= 0.0 { Theme::gut() } else { Theme::problem() }, crate::tf!("ui.post.karte_staerke", format!("{:.0}", pl.ovr), pos_de(&pl.pos))),
            }
        }).collect();
        let mut v = vec![];
        if m.karte == "ehrung" { v.push(B::Siegel { titel: m.title.clone() }); }
        v.push(B::Kacheln(kacheln));
        v.push(B::Text(m.body.clone()));
        return v;
    }
    match art_von(m) {
        Art::Scout => {
            let Some(p) = erst else { return vec![B::Text(m.body.clone())] };
            let pl = &c.world.players[p];
            let sp = spieler_spanne(c, p);
            let bekannt = bekannte_werte(c, p);
            let gr = gruppen();
            let (namen, werte): (Vec<String>, Vec<f32>) = gr.iter().map(|(id, idx, _)| {
                let w: Vec<f32> = idx.iter().filter(|&&i| bekannt.get(i).copied().unwrap_or(false)).map(|&i| pl.attrs[i]).collect();
                (tx(&format!("spp.gruppe.{id}")).to_string(), if w.is_empty() { 0.0 } else { w.iter().sum::<f32>() / w.len() as f32 })
            }).unzip();
            let mut wa: Vec<(usize, f32)> = (0..pl.attrs.len().min(25)).filter(|&i| bekannt[i]).map(|i| (i, pl.attrs[i])).collect();
            wa.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let name = |i: usize| crate::pesdb::attr_de_t()[i].to_string();
            let gut: Vec<String> = wa.iter().take(3).map(|x| name(x.0)).collect();
            let schlecht: Vec<String> = wa.iter().rev().take(3).filter(|x| x.1 < 70.0).map(|x| name(x.0)).collect();
            let preis = c.ask_price(p);
            let ps = c.preis_spanne(p) as f64;
            let beste = c.squad().into_iter().filter(|&q| part_of(&c.world.players[q].pos) == part_of(&pl.pos)).map(|q| c.world.players[q].ovr).fold(1.0f32, f32::max);
            let passung = pl.ovr / beste;
            let pk = if passung >= 1.0 { "ui.post.passung_stamm" } else if passung >= 0.93 { "ui.post.passung_rotation" } else { "ui.post.passung_breite" };
            let mut v = vec![B::Reihe(vec![B::Foto { pid: p, r: 44.0 }, verein_von(c, p).map(|k| B::Wappen { club: k }).unwrap_or(B::Text(crate::inhalt::t("ui.wort.vereinslos").into())), B::Ring { titel: t("wissen"), anteil: c.know_of(p), text: format!("{:.0} %", c.know_of(p) * 100.0) }]),
                B::Reihe(vec![B::Zahl { titel: t("alter"), wert: pl.age.to_string() }, B::Zahl { titel: t("position"), wert: pos_de(&pl.pos).into() }, B::Zahl { titel: t("vertrag"), wert: crate::date::season_label(pl.contract_end) }]),
                B::Reihe(vec![B::Zahl { titel: t("marktwert"), wert: money(pl.value) }, B::Zahl { titel: t("gehalt"), wert: money(pl.wage) }, B::Balken { titel: t("passung"), anteil: passung.min(1.2) / 1.2, text: crate::inhalt::t(pk).into(), farbe: if passung >= 0.93 { 1 } else { 3 } }]),
                B::Reihe(vec![B::Sterne { titel: t("staerke"), von: profil_sterne(pl.ovr - sp), bis: profil_sterne(pl.ovr + sp) }, B::Sterne { titel: t("potenzial"), von: profil_sterne(pl.pot - sp), bis: profil_sterne(pl.pot + sp) }]),
            ];
            // the verdict and the squad effect of the scouting concept (scoutbericht.rs), right under the head
            let b = c.scout_urteil(p);
            let sicher = if c.gehoert_uns(p) { 0 } else { let s = c.spanne(p); if s <= 2.0 { 0 } else if s <= 4.0 { 1 } else { 2 } };
            let satz = if let Some(k) = b.ko { format!("{} {}", crate::inhalt::t(&format!("ui.sb.satz.{}", b.urteil)), crate::inhalt::t(k)) } else { crate::inhalt::t(&format!("ui.sb.satz.{}", b.urteil)).to_string() };
            v.insert(1, B::Urteil { stufe: b.urteil, satz, sicher: crate::inhalt::t(&format!("ui.sb.sicher.{sicher}")).to_string() });
            v.push(B::Titel(t("bei_uns")));
            let mut rang: Vec<(usize, f32, bool, String)> = b.rang.iter().take(4).map(|&(q, s)| (q, s, false, String::new())).collect();
            rang.insert(b.platz.min(rang.len()), (p, b.staerke, true, String::new()));
            for (k, x) in rang.iter_mut().enumerate() {
                x.3 = if x.2 { crate::inhalt::t(match k { 0 => "ui.sb.rolle.stamm", 1 => "ui.sb.rolle.rotation", _ => "ui.sb.rolle.ersatz" }).to_string() }
                    else if b.verlierer.contains(&x.0) { crate::inhalt::t("ui.post.verliert_platz").to_string() }
                    else if k > b.platz && k <= 2 { crate::inhalt::t("ui.post.rutscht_ab").to_string() } else { String::new() };
            }
            v.push(B::Rang(rang));
            v.push(B::Reihe(vec![B::Zahl { titel: t("startelf"), wert: format!("{:.1} → {:.1}", b.startelf.0, b.startelf.1) }, B::Zahl { titel: t("rollen_staerke"), wert: format!("{:.0}", b.staerke) },
                B::Zahl { titel: t("preis"), wert: crate::tf!("ui.post.von_bis", money(preis * (1.0 - ps)), money(preis * (1.0 + ps))) }]));
            let teile = [crate::id("kader"), crate::id("system"), "preis_leistung", crate::id("zukunft"), crate::id("risiko")];
            v.push(B::Reihe(teile.iter().take(3).enumerate().map(|(k, n)| B::Balken { titel: t(&format!("teil_{n}")), anteil: b.teil[k] / 100.0, text: format!("{:.0}", b.teil[k]), farbe: if b.teil[k] >= 60.0 { 1 } else if b.teil[k] >= 40.0 { 2 } else { 3 } }).collect()));
            v.push(B::Reihe(teile.iter().enumerate().skip(3).map(|(k, n)| B::Balken { titel: t(&format!("teil_{n}")), anteil: b.teil[k] / 100.0, text: format!("{:.0}", b.teil[k]), farbe: if (k == 4) == (b.teil[k] >= 50.0) { 3 } else { 1 } }).collect()));
            // strengths and weaknesses only from what counts for his position, against our league's regulars
            let wort = |m: &crate::scoutbericht::Merkmal, art: &str| format!("{}{}", if m.wohl { crate::inhalt::t("ui.sb.wohl") } else { "" }, crate::inhalt::t(&format!("ui.sb.{art}.{}", crate::entwicklung::NAMEN[m.i])));
            let gut2: Vec<String> = b.staerken.iter().map(|m| wort(m, crate::id("plus"))).collect();
            let schlecht2: Vec<String> = b.schwaechen.iter().map(|m| wort(m, crate::id("minus"))).collect();
            let _ = (&gut, &schlecht);
            if !gut2.is_empty() || !schlecht2.is_empty() { v.push(B::Titel(t("staerken_schwaechen_rolle"))); v.push(B::Chips { gut: gut2, schlecht: schlecht2 }); }
            // the season in numbers (Konzept Spielerstatistiken 8.2): six numbers for his group, per 90 and converted to our
            // league, with the percentile in his league; below 40 knowledge only the basics
            {
                use crate::statistik as sx;
                let (st, hoch) = sx::saison_von(&c.world, p);
                let liga = verein_von(c, p).map(|k| c.world.clubs[k].league).unwrap_or(usize::MAX);
                if sx::get(&st, sx::MIN) >= 90 {
                    v.push(B::Titel(if hoch { t("zahlen_saison_hoch") } else { t("zahlen_saison") }));
                    if c.know_of(p) >= crate::regeln::f("scouting", "wissen.bericht_ab") / 100.0 {
                        let gr = crate::scoutbericht::gruppe(&pl.pos);
                        let vergleich = sx::liga_vergleich(&c.world, liga, gr);
                        let art = |k: &str, q: bool| -> u8 { if q { 4 } else if gr == "tw" { 3 } else { match k { "prog_paesse" | "prog_laeufe" | "dribblings" | "flanken" | "paesse" => 1, "tacklings" | "abgefangen" | "klaerungen" => 2, _ => 0 } } };
                        let zahlen: Vec<B> = sx::kernzahlen(gr).into_iter().filter_map(|(k, f, q)| {
                            let w = f(&st)?;
                            let um = if liga < c.world.leagues.len() { sx::umrechnen(&c.world, w, liga, c.league(), art(k, q)) } else { w };
                            let pz = sx::perzentil(&vergleich, f, w, false).map(|x| format!(" · {x:.0}.")).unwrap_or_default();
                            Some(B::Zahl { titel: crate::inhalt::t(&format!("ui.stat.{k}")).to_string(), wert: if q { format!("{um:.0} %{pz}") } else { format!("{um:.2}{pz}") } })
                        }).collect();
                        let mut it = zahlen.into_iter();
                        loop { let r: Vec<B> = it.by_ref().take(3).collect(); if r.is_empty() { break; } v.push(B::Reihe(r)); }
                        // the scout puts numbers and values together (Konzept 8.2): luck, bad luck, the numbers confirm him
                        let (tore, xg) = (sx::get(&st, sx::TORE) as f32, sx::get(&st, sx::XG) as f32 / 1000.0);
                        let fin = if pl.attrs.len() == 25 { pl.attrs[6] } else { pl.ovr };
                        let anlass = if tore - xg >= 3.0 && fin < 80.0 { crate::id("glueck") } else if xg - tore >= 2.5 { crate::id("pech") } else { crate::id("passt") };
                        let n = (crate::mensch::zufall(&["scoutsatz", &pl.id.to_string(), anlass]) * 5.0) as usize % 5; // roh
                        v.push(B::Text(crate::tf!(&format!("ui.post.zahlsatz.{anlass}.{n}"), format!("{tore:.0}"), format!("{xg:.1}"))));
                    } else {
                        v.push(B::Reihe(vec![B::Zahl { titel: crate::inhalt::t("ui.stat.tore").into(), wert: sx::get(&st, sx::TORE).to_string() }, B::Zahl { titel: crate::inhalt::t("ui.stat.vorlagen").into(), wert: sx::get(&st, sx::VORLAGEN).to_string() }, B::Zahl { titel: t("minuten"), wert: sx::get(&st, sx::MIN).to_string() }]));
                    }
                }
                // costs and who else is after him (Konzept 4.1 Block 8, 4.6), and how often this scout was right
                let gehalt = crate::verhandlung::wunschgehalt(pl, c.world.leagues[c.league()].level, 2);
                let machbar = c.machbar(p, false).map(|k| crate::inhalt::t(k).to_string()).unwrap_or_else(|| t("machbar"));
                v.push(B::Reihe(vec![B::Zahl { titel: t("wunschgehalt"), wert: money(gehalt) }, B::Zahl { titel: t("machbarkeit"), wert: machbar }]));
                let konkurrenz = if c.anbahnung.iter().any(|a| a.p == p) { t("konkurrenz.angebot") } else if c.grundwissen(p) >= 0.40 { t("konkurrenz.bekannt") } else { t("konkurrenz.niemand") };
                v.push(B::Text(konkurrenz));
                if let Some(pe) = c.scouts.iter().find(|s| s.name == m.from).and_then(|s| s.p.as_ref()) {
                    if pe.treffer + pe.fehler > 0 { v.push(B::Text(crate::tf!("ui.post.trefferquote", m.from.clone(), pe.treffer, pe.treffer + pe.fehler))); }
                }
            }
            if werte.iter().any(|&w| w > 0.0) { v.push(B::Radar { namen, werte }); }
            v.push(B::Zitat { pid: None, name: sprecher(c, &m.from), text: m.body.clone() });
            v
        }
        Art::Woche => woche_bausteine(c, m),
        Art::Spiel => {
            let l = c.league();
            let f = c.world.leagues[l].fixtures.iter().find(|f| f.day == m.day && f.hg >= 0 && (f.h == c.own || f.a == c.own)).copied();
            let res = c.last_result.as_ref().filter(|r| f.map_or(true, |f| r.h == f.h && r.a == f.a));
            let mut v = vec![];
            if let Some(r) = res {
                v.push(B::Anzeige { h: r.h, a: r.a, hg: r.hg as i8, ag: r.ag as i8 });
                let tore: Vec<String> = r.events.iter().filter(|e| e.kind <= 1).map(|e| crate::tf!("ui.post.tor_minute", e.min, c.world.players[e.pid].name)).collect();
                if !tore.is_empty() { v.push(B::Text(tore.join(" · "))); }
                let eigen = if r.h == c.own { &r.xi_h } else { &r.xi_a };
                let mut noten: Vec<(usize, f32)> = r.ratings.iter().filter(|x| eigen.iter().any(|e| e.1 == x.0)).copied().collect();
                noten.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                let ballbesitz = if r.h == c.own { r.possession } else { 100 - r.possession } as f32;
                let (s1, s2) = if r.h == c.own { r.shots } else { (r.shots.1, r.shots.0) };
                let (x1, x2) = if r.h == c.own { (r.lh, r.la) } else { (r.la, r.lh) };
                if let Some(&(bp, note)) = noten.first() { v.push(B::Reihe(vec![B::Foto { pid: bp, r: 30.0 }, B::Zahl { titel: t("bester_spieler"), wert: crate::tf!("ui.post.name_note", c.world.players[bp].name, format!("{note:.1}")) }])); }
                v.push(B::Reihe(vec![B::Balken { titel: t("ballbesitz"), anteil: ballbesitz / 100.0, text: format!("{ballbesitz:.0} %"), farbe: 0 },
                    B::Balken { titel: t("schuesse"), anteil: s1 as f32 / (s1 + s2).max(1) as f32, text: format!("{s1} : {s2}"), farbe: 0 },
                    B::Balken { titel: t("xg"), anteil: x1 / (x1 + x2).max(0.01), text: format!("{x1:.1} : {x2:.1}"), farbe: 0 }]));
                if !noten.is_empty() { v.push(B::Pillen(noten)); }
            } else if let Some(f) = f {
                v.push(B::Anzeige { h: f.h, a: f.a, hg: f.hg, ag: f.ag });
            }
            v.push(B::Text(m.body.clone()));
            v
        }
        Art::Angebot => {
            let mut v = vec![];
            match &m.action {
                Action::TransferOffer { pid, club, fee } => {
                    let pl = &c.world.players[*pid];
                    v.push(B::Reihe(vec![B::Foto { pid: *pid, r: 36.0 }, B::Titel(pl.name.clone()), B::Wappen { club: *club }]));
                    v.push(B::Reihe(vec![B::Zahl { titel: t("abloese"), wert: money(*fee) }, B::Zahl { titel: t("raten"), wert: t("raten_keine") }, B::Zahl { titel: t("gehalt"), wert: money(pl.wage) }, B::Zahl { titel: t("vertrag"), wert: crate::date::season_label(pl.contract_end) }]));
                    v.push(B::Balken { titel: t("gegen_marktwert"), anteil: (*fee / pl.value.max(1.0) / 2.0) as f32, text: format!("{:.0} %", fee / pl.value.max(1.0) * 100.0), farbe: if *fee >= pl.value { 1 } else { 2 } });
                    v.push(B::Zahl { titel: t("klauseln"), wert: t("klauseln_keine") });
                }
                Action::Expiring { pid } => {
                    let pl = &c.world.players[*pid];
                    let wunsch = crate::verhandlung::wunschgehalt(pl, c.world.leagues[c.league()].level, pl.ps.rolle);
                    v.push(B::Reihe(vec![B::Foto { pid: *pid, r: 36.0 }, B::Titel(pl.name.clone())]));
                    v.push(B::Reihe(vec![B::Zahl { titel: t("gehalt"), wert: money(pl.wage) }, B::Zahl { titel: t("wunsch"), wert: money(wunsch) }, B::Zahl { titel: t("laufzeit"), wert: crate::tf!("ui.post.jahre", crate::verhandlung::wunschlaufzeit(pl.age)) }]));
                    v.push(B::Balken { titel: t("wunsch_gegen_gehalt"), anteil: (pl.wage / wunsch.max(1.0)).min(1.0) as f32, text: format!("{:.0} %", pl.wage / wunsch.max(1.0) * 100.0), farbe: 3 });
                }
                Action::JobOffer { club, wage } => {
                    let k = &c.world.clubs[*club];
                    v.push(B::Reihe(vec![B::Wappen { club: *club }, B::Titel(k.name.clone())]));
                    v.push(B::Reihe(vec![B::Zahl { titel: t("gehalt"), wert: money(*wage) }, B::Zahl { titel: t("liga"), wert: c.world.leagues[k.league].name.clone() }, B::Zahl { titel: t("platz"), wert: c.world.rank_of(*club).to_string() }]));
                }
                _ => { if let Some(p) = erst { v.push(B::Foto { pid: p, r: 36.0 }); v.push(B::Zahl { titel: t("marktwert"), wert: money(c.world.players[p].value) }); } }
            }
            if let Some(n) = erst.and_then(|p| c.negs.iter().find(|n| n.pid == p && n.stage < 2)) {
                v.push(B::Reihe(vec![B::Zahl { titel: t("runde"), wert: (n.round + 1).to_string() }, B::Balken { titel: t("geduld"), anteil: 1.0 - n.round as f32 / 6.0, text: crate::tf!("ui.post.runde_von", n.round + 1, 6), farbe: 3 }]));
            }
            v.push(B::Text(m.body.clone()));
            v
        }
        Art::Gespraech => {
            let Some(p) = erst else { return vec![B::Gesicht { name: sprecher(c, &m.from) }, B::Text(m.body.clone())] };
            let pl = &c.world.players[p];
            let mut werte = c.moral_verlauf.get(&pl.id).cloned().unwrap_or_default();
            werte.push(pl.morale);
            vec![B::Reihe(vec![B::Foto { pid: p, r: 50.0 }, B::Zahl { titel: t("stimmung"), wert: format!("{:.0}", pl.morale) }, B::Diagramm { titel: t("moral_verlauf"), werte }]),
                B::Reihe(vec![B::Zahl { titel: t("einsaetze"), wert: pl.apps.to_string() }, B::Zahl { titel: t("bank"), wert: pl.benched.to_string() }, B::Zahl { titel: t("rolle"), wert: crate::mensch::rolle_name(pl.ps.rolle).into() }]),
                B::Zitat { pid: Some(p), name: pl.name.clone(), text: m.body.clone() }]
        }
        Art::Vorstand => {
            let rang = c.world.rank_of(c.own);
            vec![B::Reihe(vec![B::Ring { titel: t("vertrauen"), anteil: c.mensch.vorstand / 100.0, text: format!("{:.0}", c.mensch.vorstand) },
                    B::Zahl { titel: t("ziel_platz"), wert: crate::tf!("ui.post.ziel_gegen_platz", c.club().goal_pos, rang) }, B::Zahl { titel: t("budget"), wert: money(c.transfer_budget) }, B::Zahl { titel: t("kasse"), wert: money(c.club().balance) }]),
                B::Zitat { pid: None, name: sprecher(c, &m.from), text: m.body.clone() }]
        }
        Art::Verletzung => {
            let Some(p) = erst else { return vec![B::Gesicht { name: sprecher(c, &m.from) }, B::Text(m.body.clone())] };
            let pl = &c.world.players[p];
            let tage = pl.inj as f32;
            let phasen = vec![(t("phase_behandlung"), tage > 21.0), (t("phase_aufbau"), tage > 7.0 && tage <= 21.0), (t("phase_mannschaft"), tage <= 7.0)];
            vec![B::Reihe(vec![B::Foto { pid: p, r: 40.0 }, B::Titel(pl.name.clone())]),
                B::Reihe(vec![B::Zahl { titel: t("koerperstelle"), wert: if pl.inj_name.is_empty() { t("unbekannt") } else { pl.inj_name.clone() } }, B::Zahl { titel: t("rueckkehr"), wert: crate::date::short(c.world.season, c.world.day + pl.inj as u32) }]),
                B::Balken { titel: t("ausfall"), anteil: (tage / 90.0).min(1.0), text: crate::tf!("ui.post.tage", pl.inj), farbe: 2 },
                B::Phasen(phasen),
                B::Zitat { pid: None, name: sprecher(c, &m.from), text: m.body.clone() }]
        }
        Art::Presse => vec![B::Zeitung { name: sprecher(c, &m.from), schlagzeile: m.title.clone(), text: m.body.clone() },
            B::Reihe(vec![B::Gesicht { name: sprecher(c, &m.from) }, B::Balken { titel: t("fans"), anteil: c.fans / 100.0, text: format!("{:.0}", c.fans), farbe: 1 }, B::Balken { titel: t("vorstand"), anteil: c.mensch.vorstand / 100.0, text: format!("{:.0}", c.mensch.vorstand), farbe: 0 }])],
        Art::Talent => {
            let Some(p) = erst else { return vec![B::Text(m.body.clone())] };
            let pl = &c.world.players[p];
            let kader: Vec<f32> = c.squad().into_iter().filter(|&q| part_of(&c.world.players[q].pos) == part_of(&pl.pos) && q != p).map(|q| c.world.players[q].ovr).collect();
            let schnitt = if kader.is_empty() { pl.ovr } else { kader.iter().sum::<f32>() / kader.len() as f32 };
            vec![B::Reihe(vec![B::Foto { pid: p, r: 44.0 }, B::Titel(pl.name.clone())]),
                B::Reihe(vec![B::Zahl { titel: t("alter"), wert: pl.age.to_string() }, B::Zahl { titel: t("position"), wert: pos_de(&pl.pos).into() }, B::Sterne { titel: t("potenzial"), von: profil_sterne(pl.pot - 4.0), bis: profil_sterne(pl.pot + 4.0) }]),
                B::Balken { titel: t("gegen_kader"), anteil: (pl.ovr / schnitt.max(1.0)).min(1.0), text: crate::tf!("ui.post.staerke_gegen_schnitt", format!("{:.0}", pl.ovr), format!("{schnitt:.0}")), farbe: 0 },
                B::Text(m.body.clone())]
        }
        Art::Gegner => {
            // the opponent: from the key players the report names, else the next match
            let g = erst.and_then(|p| verein_von(c, p)).or_else(|| c.next_match().map(|f| if f.h == c.own { f.a } else { f.h }));
            let Some(g) = g else { return vec![B::Text(m.body.clone())] };
            let l = c.world.clubs[g].league;
            let tab = c.world.table(l);
            let platz = tab.iter().position(|r| r.club == g).map(|i| (i + 1).to_string()).unwrap_or_default();
            let st = crate::mensch::kaderstaerke(&c.world, g) - crate::mensch::kaderstaerke(&c.world, c.own);
            let stil = match c.world.clubs[g].style { -1 => crate::inhalt::t("ui.taktik.steht_tief_und_kontert"), 1 => crate::inhalt::t("ui.taktik.spielt_offensiv_nach_vorn"), _ => crate::inhalt::t("ui.post.ausgewogen") };
            let mut form: Vec<(u32, usize, usize, i8, i8)> = c.world.leagues[l].fixtures.iter().filter(|f| f.hg >= 0 && (f.h == g || f.a == g)).map(|f| (f.day, f.h, f.a, f.hg, f.ag)).collect();
            form.sort_by_key(|x| std::cmp::Reverse(x.0));
            let form: Vec<(usize, usize, i8, i8)> = form.into_iter().take(4).map(|x| (x.1, x.2, x.3, x.4)).collect();
            let sw = crate::taktik::schwaechen(&c.world, g, c.own);
            let gut: Vec<String> = sw.iter().filter(|x| x.1).map(|x| crate::taktik::bereich_wort(x.0).to_string()).collect();
            let schlecht: Vec<String> = sw.iter().filter(|x| !x.1).map(|x| crate::taktik::bereich_wort(x.0).to_string()).collect();
            let mut v = vec![B::Reihe(vec![B::Wappen { club: g }, B::Zahl { titel: t("platz"), wert: platz }, B::Zahl { titel: t("staerke_gegen_uns"), wert: format!("{st:+.1}").replace('-', "−") }, B::Zahl { titel: t("spielweise"), wert: stil.into() }])];
            if !form.is_empty() { v.push(B::Titel(t("form_letzte"))); v.push(B::Ergebnisse(form)); }
            if !gut.is_empty() || !schlecht.is_empty() { v.push(B::Titel(t("staerken_schwaechen"))); v.push(B::Chips { gut, schlecht }); }
            let fotos: Vec<B> = wer.iter().take(3).map(|&p| B::Foto { pid: p, r: 28.0 }).collect();
            if !fotos.is_empty() { v.push(B::Titel(t("schluesselspieler"))); v.push(B::Reihe(fotos)); }
            v.push(B::Zitat { pid: None, name: sprecher(c, &m.from), text: m.body.rsplit(" · ").next().unwrap_or(&m.body).to_string() });
            v
        }
        Art::Meilenstein => vec![B::Siegel { titel: m.title.clone() }, B::Reihe(vec![B::Wappen { club: c.own }, B::Zahl { titel: t("datum"), wert: crate::date::long(m.season, m.day) }, B::Zahl { titel: t("platz"), wert: c.world.rank_of(c.own).to_string() }]), B::Text(m.body.clone())],
        Art::Allgemein => {
            let mut v = vec![B::Reihe(vec![B::Gesicht { name: sprecher(c, &m.from) }, B::Zahl { titel: t("datum"), wert: crate::date::long(m.season, m.day) }, B::Wappen { club: c.own }])];
            if let Some(p) = erst { v.push(B::Foto { pid: p, r: 30.0 }); }
            v.push(B::Text(m.body.clone()));
            v
        }
    }
}

/// The week in the world: results of the own league, the table around the own place with arrows, the top scorer,
/// the transfers of the week, the surprise, European results, injuries at the rivals.
fn woche_bausteine(c: &Career, m: &Msg) -> Vec<B> {
    let t = |k: &str| crate::inhalt::t(&format!("ui.post.{k}")).to_string();
    let (bis, von) = (m.day, m.day.saturating_sub(7));
    let l = c.league();
    let lg = &c.world.leagues[l];
    let woche = |d: u32| d > von && d <= bis;
    let rang = c.world.rank_of(c.own);
    let pkt = c.world.table(l).iter().find(|r| r.club == c.own).map_or(0, |r| r.pts);
    let mut v = vec![B::Reihe(vec![B::Wappen { club: c.own }, B::Zahl { titel: t("platz"), wert: rang.to_string() }, B::Zahl { titel: crate::inhalt::t("ui.post.punkte").into(), wert: pkt.to_string() }, B::Zahl { titel: t("liga"), wert: lg.name.clone() }]), B::Titel(t("ergebnisse"))];
    let erg: Vec<(usize, usize, i8, i8)> = lg.fixtures.iter().filter(|f| woche(f.day) && f.hg >= 0).map(|f| (f.h, f.a, f.hg, f.ag)).collect();
    if erg.is_empty() { v.push(B::Text(t("keine_spiele"))); } else { v.push(B::Ergebnisse(erg)); }
    // table around the own place, the arrow from the table of a week ago
    let tab = c.world.table(l);
    let jetzt: Vec<usize> = tab.iter().map(|r| r.club).collect();
    let vorher = if lg.fixtures.iter().any(|f| f.day <= von && f.hg >= 0) { tabelle_bis(c, l, von) } else { jetzt.clone() };
    if let Some(i) = jetzt.iter().position(|&k| k == c.own) {
        let a = i.saturating_sub(2);
        let zeilen: Vec<(usize, usize, i32, i32)> = (a..(a + 5).min(jetzt.len())).map(|j| { let alt = vorher.iter().position(|&k| k == jetzt[j]).unwrap_or(j) as i32; (j + 1, jetzt[j], tab[j].pts, alt - j as i32) }).collect();
        v.push(B::Tabelle(zeilen));
    }
    let mut torj: Vec<usize> = lg.teams.iter().flat_map(|&k| c.world.clubs[k].squad.iter().copied()).filter(|&p| c.world.players[p].goals > 0).collect();
    torj.sort_by_key(|&p| std::cmp::Reverse(c.world.players[p].goals));
    let mut reihe = vec![];
    if let Some(&p) = torj.first() { reihe.push(B::Foto { pid: p, r: 28.0 }); reihe.push(B::Zahl { titel: t("torjaeger"), wert: crate::tf!("ui.post.name_tore", c.world.players[p].name, c.world.players[p].goals) }); }
    // the surprise: the weakest winner of the week by club strength, in the detailed leagues
    let mut ueber: Option<(f32, usize, usize, i8, i8)> = None;
    for g in c.world.leagues.iter().filter(|g| g.detail <= 1) {
        for f in g.fixtures.iter().filter(|f| woche(f.day) && f.hg >= 0 && f.hg != f.ag) {
            let (s, n) = if f.hg > f.ag { (f.h, f.a) } else { (f.a, f.h) };
            let d = c.world.clubs[n].level - c.world.clubs[s].level;
            if d > ueber.map_or(4.0, |u| u.0) { ueber = Some((d, f.h, f.a, f.hg, f.ag)); }
        }
    }
    if let Some((_, h, a, hg, ag)) = ueber { reihe.push(B::Zahl { titel: t("ueberraschung"), wert: crate::tf!("ui.post.ergebnis_text", c.world.clubs[h].short, hg, ag, c.world.clubs[a].short) }); }
    if !reihe.is_empty() { v.push(B::Reihe(reihe)); }
    let neue: Vec<B> = c.world.players.iter().enumerate().filter(|(_, p)| p.team >= 0 && p.youth == 0 && p.joined > von && p.joined <= bis && p.joined > 0 && p.ovr >= 70.0)
        .take(3).map(|(i, p)| B::Transfer { pid: i, nach: p.team as usize, wert: p.value }).collect();
    if !neue.is_empty() { v.push(B::Titel(t("transfers"))); v.push(B::Reihe(neue)); }
    let euro: Vec<(usize, usize, i8, i8)> = c.world.cups.iter().filter(|k| (1..=3).contains(&k.kind)).flat_map(|k| k.fx.iter()).filter(|x| woche(x.f.day) && x.f.hg >= 0).take(6).map(|x| (x.f.h, x.f.a, x.f.hg, x.f.ag)).collect();
    if !euro.is_empty() { v.push(B::Titel(t("europapokal"))); v.push(B::Ergebnisse(euro)); }
    if let Some(i) = jetzt.iter().position(|&k| k == c.own) {
        let rivalen: Vec<usize> = jetzt.iter().enumerate().filter(|(j, &k)| k != c.own && (*j as i32 - i as i32).abs() <= 3).map(|(_, &k)| k).collect();
        let mut verl: Vec<usize> = rivalen.iter().flat_map(|&k| c.world.clubs[k].squad.iter().copied()).filter(|&p| c.world.players[p].inj > 0).collect();
        verl.sort_by(|a, b| c.world.players[*b].ovr.partial_cmp(&c.world.players[*a].ovr).unwrap());
        let chips: Vec<String> = verl.iter().take(4).map(|&p| { let pl = &c.world.players[p]; crate::tf!("ui.post.verletzt_bei", pl.name, c.world.clubs[pl.team as usize].short, pl.inj) }).collect();
        if !chips.is_empty() { v.push(B::Titel(t("verletzte_rivalen"))); v.push(B::Chips { gut: vec![], schlecht: chips }); }
    }
    v
}
/// The table order of a league counting only the matches played up to a day.
fn tabelle_bis(c: &Career, l: usize, tag: u32) -> Vec<usize> {
    let lg = &c.world.leagues[l];
    let mut pkt: Vec<(usize, i32, i32)> = lg.teams.iter().map(|&k| (k, 0, 0)).collect();
    for f in lg.fixtures.iter().filter(|f| f.day <= tag && f.hg >= 0) {
        let (hp, ap) = if f.hg > f.ag { (3, 0) } else if f.hg < f.ag { (0, 3) } else { (1, 1) };
        if let Some(x) = pkt.iter_mut().find(|x| x.0 == f.h) { x.1 += hp; x.2 += f.hg as i32 - f.ag as i32; }
        if let Some(x) = pkt.iter_mut().find(|x| x.0 == f.a) { x.1 += ap; x.2 += f.ag as i32 - f.hg as i32; }
    }
    pkt.sort_by(|a, b| (b.1, b.2).cmp(&(a.1, a.2)));
    pkt.into_iter().map(|x| x.0).collect()
}

/// Height a block needs at width `w`.
fn b_hoehe(ui: &Ui, b: &B, w: f32) -> f32 {
    match b {
        B::Foto { r, .. } => 2.0 * r + 8.0,
        B::Wappen { .. } | B::Gesicht { .. } => 64.0,
        B::Zahl { .. } => 54.0,
        B::Balken { .. } | B::Sterne { .. } => 50.0,
        B::Ring { .. } => 96.0,
        B::Diagramm { .. } => 80.0,
        B::Radar { .. } => 170.0,
        B::Chips { gut, schlecht } => 34.0 * ((gut.len() + schlecht.len()).max(1) as f32 / 3.0).ceil(),
        B::Zitat { text, .. } => (ui.wrap(text, 16.0, w - 84.0).len() as f32 * 21.0 + 30.0).max(64.0),
        B::Anzeige { .. } => 86.0,
        B::Ergebnisse(v) => 36.0 * ((v.len() as f32) / 2.0).ceil(),
        B::Tabelle(v) => 26.0 * v.len() as f32 + 6.0,
        B::Transfer { .. } => 64.0,
        B::Pillen(v) => 30.0 * ((v.len() as f32) / 4.0).ceil(),
        B::Zeitung { text, .. } => ui.wrap(text, 16.0, w - 40.0).len().min(8) as f32 * 21.0 + 84.0,
        B::Siegel { .. } => 110.0,
        B::Phasen(_) => 40.0,
        B::Titel(_) => 30.0,
        B::Kacheln(v) => 124.0 * ((v.len() as f32) / 3.0).ceil(),
        B::Elf(_) => 300.0,
        B::Urteil { .. } => 86.0,
        B::Rang(v) => 10.0 + v.len() as f32 * 36.0,
        B::Text(s) => ui.wrap(s, 16.0, w).len().min(10) as f32 * 21.0 + 6.0,
        B::Reihe(v) => v.iter().map(|x| b_hoehe(ui, x, w / v.len().max(1) as f32 - 10.0)).fold(0.0, f32::max),
    }
}
fn farbe_von(f: u8) -> Color { match f { 1 => Theme::gut(), 2 => Theme::problem(), 3 => Theme::achtung(), _ => Theme::akzent() } }
/// Draws the blocks top down into `r`; when they do not all fit, the area scrolls block by block (wheel or bar).
/// Something a name in a message text leads to (Fabian 23.11.: clubs and players clickable).
#[derive(Clone, Copy, PartialEq)]
pub enum Ziel { Spieler(usize), Verein(usize) }
thread_local! {
    /// The names of the message being drawn with their targets, and the one clicked in this frame.
    static LINKS: std::cell::RefCell<Vec<(String, Ziel)>> = const { std::cell::RefCell::new(Vec::new()) };
    static KLICK: std::cell::Cell<Option<Ziel>> = const { std::cell::Cell::new(None) };
}
/// Underlines the known names in one drawn line and notes a click on one.
fn links_zeichnen(ui: &Ui, zeile: &str, x: f32, y: f32) {
    LINKS.with(|l| {
        for (name, ziel) in l.borrow().iter() {
            let mut ab = 0;
            while let Some(i) = zeile[ab..].find(name.as_str()) {
                let start = ab + i;
                let x0 = x + ui.measure(&zeile[..start], 16.0);
                let w = ui.measure(name, 16.0);
                let feld = Rect::new(x0, y - 15.0, w, 20.0);
                let col = if feld.contains(ui.mouse) { Theme::akzent() } else { alpha(Theme::akzent(), 0.75) };
                ui.text(name, x0, y, 16.0, col);
                draw_rectangle(x0, y + 3.0, w, 1.5, col);
                if ui.clicked && feld.contains(ui.mouse) { KLICK.with(|k| k.set(Some(*ziel))); }
                ab = start + name.len();
            }
        }
    });
}
fn b_bereich(ui: &Ui, v: &mut View, c: &Career, r: Rect, bs: &[B]) {
    let w = r.w - 18.0;
    let hs: Vec<f32> = bs.iter().map(|b| b_hoehe(ui, b, w) + 8.0).collect();
    // how many blocks fit at the end: the furthest the area can scroll
    let (mut rest, mut ab) = (0.0, bs.len());
    while ab > 0 && rest + hs[ab - 1] <= r.h { rest += hs[ab - 1]; ab -= 1; }
    let start = scroll_schritt(ui, v, r, bs.len(), bs.len() - ab, 1.0);
    let mut y = r.y;
    for (b, h) in bs.iter().zip(&hs).skip(start) {
        if y + h - 8.0 > r.y + r.h { break; }
        b_eins(ui, c, Rect::new(r.x, y, w, h - 8.0), b);
        y += h;
    }
}
fn b_eins(ui: &Ui, c: &Career, r: Rect, b: &B) {
    if !matches!(b, B::Reihe(_)) { ui.rahmen(r); }
    let wappen = |k: usize, cx: f32, cy: f32, rr: f32| { let (c1, _) = club_color(c, k); ui.club_crest(cx, cy, rr, c.world.clubs[k].id, c1, &c.world.clubs[k].short); };
    match b {
        B::Foto { pid, r: rr } => { let pl = &c.world.players[*pid]; ui.player_photo(r.x + rr + 4.0, r.y + rr + 4.0, *rr, pl.id, &pl.name); }
        B::Wappen { club } => wappen(*club, r.x + 32.0, r.y + 32.0, 28.0),
        B::Gesicht { name } => ui.avatar(r.x + 32.0, r.y + 32.0, 28.0, name),
        B::Zahl { titel, wert } => { ui.text(&ui.fit(titel, 13.0, r.w - 6.0), r.x, r.y + 16.0, 13.0, Theme::gedaempft()); ui.text_semi(&ui.fit(wert, 19.0, r.w - 6.0), r.x, r.y + 42.0, 19.0, Theme::text()); }
        B::Balken { titel, anteil, text, farbe } => {
            let wert = ui.fit(text, 13.0, r.w * 0.7);
            ui.text(&ui.fit(titel, 13.0, r.w - ui.measure(&wert, 13.0) - 18.0), r.x, r.y + 16.0, 13.0, Theme::gedaempft());
            ui.text_right(&wert, r.x + r.w - 6.0, r.y + 16.0, 13.0, Theme::text());
            ui.bar(r.x, r.y + 26.0, r.w - 6.0, 10.0, anteil.clamp(0.0, 1.0), farbe_von(*farbe));
        }
        B::Sterne { titel, von, bis } => {
            ui.text(&ui.fit(titel, 13.0, r.w * 0.55), r.x, r.y + 16.0, 13.0, Theme::gedaempft());
            if von < bis { ui.text_right(&crate::tf!("ui.post.sterne_spanne", format!("{von:.1}"), format!("{bis:.1}")), r.x + r.w - 6.0, r.y + 16.0, 13.0, Theme::text2()); }
            ui.sterne(r.x, r.y + 34.0, *bis, 8.0);
        }
        B::Ring { titel, anteil, text } => {
            ui.text(&ui.fit(titel, 13.0, r.w - 6.0), r.x, r.y + 16.0, 13.0, Theme::gedaempft());
            let (cx, cy, rr) = (r.x + 36.0, r.y + 58.0, 30.0);
            for i in 0..48 { let a = -std::f32::consts::FRAC_PI_2 + i as f32 / 48.0 * std::f32::consts::TAU; draw_circle(cx + a.cos() * rr, cy + a.sin() * rr, 3.2, if (i as f32 / 48.0) < *anteil { Theme::akzent() } else { Theme::ueberlagerung() }); }
            ui.text_center(text, cx, cy + 6.0, 16.0, Theme::text());
        }
        B::Diagramm { titel, werte } => {
            ui.text(titel, r.x, r.y + 14.0, 13.0, Theme::gedaempft());
            let (x0, y0, w, h) = (r.x, r.y + 22.0, r.w - 10.0, r.h - 26.0);
            rrect(x0, y0, w, h, 4.0, Theme::erhoehung());
            if werte.len() >= 2 {
                let pt = |i: usize| vec2(x0 + 6.0 + i as f32 / (werte.len() - 1) as f32 * (w - 12.0), y0 + h - 4.0 - werte[i].clamp(0.0, 100.0) / 100.0 * (h - 8.0));
                for i in 1..werte.len() { let (a, b2) = (pt(i - 1), pt(i)); draw_line(a.x, a.y, b2.x, b2.y, 2.0, Theme::akzent()); }
                let e = pt(werte.len() - 1); draw_circle(e.x, e.y, 3.5, Theme::akzent());
            } else if let Some(w0) = werte.first() { ui.text_center(&format!("{w0:.0}"), x0 + w / 2.0, y0 + h / 2.0 + 6.0, 18.0, Theme::text()); }
        }
        B::Radar { namen, werte } => {
            let (cx, cy, rr) = (r.x + r.w / 2.0, r.y + r.h / 2.0 + 4.0, (r.h / 2.0 - 22.0).min(r.w / 2.0 - 70.0));
            let n = werte.len().max(3);
            let wert = |i: usize| werte.get(i % werte.len().max(1)).copied().unwrap_or(0.0);
            let pt = |i: usize, f: f32| { let a = -std::f32::consts::FRAC_PI_2 + i as f32 / n as f32 * std::f32::consts::TAU; vec2(cx + a.cos() * rr * f, cy + a.sin() * rr * f) };
            for k in 1..=2 { for i in 0..n { let (a, b2) = (pt(i, k as f32 / 2.0), pt((i + 1) % n, k as f32 / 2.0)); draw_line(a.x, a.y, b2.x, b2.y, 1.0, Theme::kontur()); } }
            for i in 0..n { let (a, b2) = (pt(i, (wert(i) / 99.0).clamp(0.05, 1.0)), pt((i + 1) % n, (wert(i + 1) / 99.0).clamp(0.05, 1.0))); draw_triangle(vec2(cx, cy), a, b2, alpha(Theme::akzent(), 0.35)); draw_line(a.x, a.y, b2.x, b2.y, 2.0, Theme::akzent()); }
            for (i, nm) in namen.iter().enumerate().take(n) { let p = pt(i, 1.2); ui.text_center(&ui.fit(nm, 12.0, 90.0), p.x, p.y + 4.0, 12.0, Theme::text2()); }
        }
        B::Chips { gut, schlecht } => {
            let mut x = r.x; let mut y = r.y;
            for (s, col) in gut.iter().map(|s| (s, Theme::gut())).chain(schlecht.iter().map(|s| (s, Theme::problem()))) {
                let s = ui.fit(s, 14.0, r.w - 30.0);
                let w = ui.measure(&s, 14.0) + 22.0;
                if x + w > r.x + r.w { x = r.x; y += 34.0; }
                rrect(x, y + 4.0, w, 26.0, 13.0, alpha(col, 0.18)); ui.text(&s, x + 11.0, y + 22.0, 14.0, col);
                x += w + 8.0;
            }
        }
        B::Zitat { pid, name, text } => {
            rrect(r.x, r.y, r.w - 6.0, r.h, 10.0, Theme::erhoehung());
            match pid { Some(p) => ui.player_photo(r.x + 32.0, r.y + 32.0, 22.0, c.world.players[*p].id, name), None => ui.avatar(r.x + 32.0, r.y + 32.0, 22.0, name) }
            ui.text_semi(&ui.fit(crate::inhalt::anzeige(name), 14.0, r.w - 90.0), r.x + 64.0, r.y + 20.0, 14.0, Theme::akzent());
            for (k, l) in ui.wrap(text, 16.0, r.w - 84.0).iter().enumerate() { ui.text(l, r.x + 64.0, r.y + 42.0 + k as f32 * 21.0, 16.0, Theme::text()); }
        }
        B::Anzeige { h, a, hg, ag } => {
            rrect(r.x, r.y, r.w - 6.0, r.h, 12.0, Theme::erhoehung());
            wappen(*h, r.x + 46.0, r.y + 40.0, 28.0); wappen(*a, r.x + r.w - 52.0, r.y + 40.0, 28.0);
            let s = format!("{hg} : {ag}");
            ui.title(&s, r.x + r.w / 2.0 - ui.measure_title(&s, 44.0) / 2.0, r.y + 56.0, 44.0, Theme::text());
            ui.text_center(&ui.fit(&c.world.clubs[*h].name, 14.0, r.w / 3.0), r.x + 52.0 + r.w * 0.08, r.y + r.h - 6.0, 14.0, Theme::text2());
            ui.text_center(&ui.fit(&c.world.clubs[*a].name, 14.0, r.w / 3.0), r.x + r.w - 58.0 - r.w * 0.08, r.y + r.h - 6.0, 14.0, Theme::text2());
        }
        B::Ergebnisse(v) => {
            let w = r.w / 2.0 - 8.0;
            for (k, (h, a, hg, ag)) in v.iter().enumerate() {
                let (x, y) = (r.x + (k % 2) as f32 * (w + 8.0), r.y + (k / 2) as f32 * 36.0);
                rrect(x, y, w, 32.0, 16.0, Theme::erhoehung());
                wappen(*h, x + 16.0, y + 16.0, 11.0); wappen(*a, x + w - 16.0, y + 16.0, 11.0);
                let s = crate::tf!("ui.post.ergebnis_text", c.world.clubs[*h].short, hg, ag, c.world.clubs[*a].short);
                ui.text_center(&ui.fit(&s, 14.0, w - 60.0), x + w / 2.0, y + 21.0, 14.0, if *h == c.own || *a == c.own { Theme::akzent() } else { Theme::text() });
            }
        }
        B::Tabelle(v) => {
            for (k, (platz, club, pkt, pfeil)) in v.iter().enumerate() {
                let y = r.y + k as f32 * 26.0;
                if *club == c.own { rrect(r.x, y, r.w - 6.0, 24.0, 4.0, alpha(Theme::akzent(), 0.14)); }
                ui.text(&platz.to_string(), r.x + 6.0, y + 17.0, 14.0, Theme::text2());
                wappen(*club, r.x + 40.0, y + 12.0, 9.0);
                ui.text(&ui.fit(&c.world.clubs[*club].name, 14.0, r.w - 150.0), r.x + 56.0, y + 17.0, 14.0, Theme::text());
                ui.text_right(&pkt.to_string(), r.x + r.w - 40.0, y + 17.0, 14.0, Theme::text());
                let (px, py) = (r.x + r.w - 22.0, y + 12.0);
                if *pfeil > 0 { draw_triangle(vec2(px - 5.0, py + 4.0), vec2(px + 5.0, py + 4.0), vec2(px, py - 5.0), Theme::gut()); }
                else if *pfeil < 0 { draw_triangle(vec2(px - 5.0, py - 4.0), vec2(px + 5.0, py - 4.0), vec2(px, py + 5.0), Theme::problem()); }
                else { draw_rectangle(px - 5.0, py - 1.0, 10.0, 2.0, Theme::gedaempft()); }
            }
        }
        B::Transfer { pid, nach, wert } => {
            let pl = &c.world.players[*pid];
            rrect(r.x, r.y, r.w - 8.0, r.h - 4.0, 10.0, Theme::erhoehung());
            ui.player_photo(r.x + 28.0, r.y + 30.0, 20.0, pl.id, &pl.name);
            ui.text_semi(&ui.fit(&pl.name, 14.0, r.w - 100.0), r.x + 54.0, r.y + 24.0, 14.0, Theme::text());
            ui.text(&ui.fit(&crate::tf!("ui.post.nach_wert", c.world.clubs[*nach].short, money(*wert)), 13.0, r.w - 100.0), r.x + 54.0, r.y + 44.0, 13.0, Theme::text2());
            wappen(*nach, r.x + r.w - 30.0, r.y + 30.0, 13.0);
        }
        B::Urteil { stufe, satz, sicher } => {
            let farbe = [hex(0x1E9E5A), Theme::gut(), hex(0x8BC34A), Theme::achtung(), Theme::akzent(), Theme::problem()][(*stufe).min(5)];
            rrect(r.x, r.y, r.w - 6.0, r.h, 10.0, alpha(farbe, 0.16));
            draw_rectangle(r.x, r.y + 8.0, 6.0, r.h - 16.0, farbe);
            if *stufe == 0 { draw_poly(r.x + 28.0, r.y + 30.0, 5, 11.0, -90.0, Theme::gold()); }
            ui.title(crate::inhalt::t(&format!("ui.sb.urteil.{stufe}")), r.x + if *stufe == 0 { 46.0 } else { 18.0 }, r.y + 36.0, 26.0, farbe);
            ui.text_passend(satz, r.x + 18.0, r.y + 58.0, 15.0, r.w - 40.0, 1, Theme::text());
            ui.text(sicher, r.x + 18.0, r.y + 78.0, 13.0, Theme::text2());
        }
        B::Rang(v) => {
            for (k, (q, s, kandidat, hinweis)) in v.iter().enumerate() {
                let pl = &c.world.players[*q];
                let y = r.y + 4.0 + k as f32 * 36.0;
                if *kandidat { rrect(r.x, y, r.w - 6.0, 32.0, 6.0, alpha(Theme::akzent(), 0.16)); }
                ui.title(&format!("{}", k + 1), r.x + 10.0, y + 24.0, 20.0, Theme::text2());
                ui.player_photo(r.x + 50.0, y + 16.0, 13.0, pl.id, &pl.name);
                ui.text_semi(&ui.fit(&pl.name, 15.0, r.w * 0.4), r.x + 70.0, y + 21.0, 15.0, if *kandidat { Theme::akzent() } else { Theme::text() });
                ui.text(&format!("{} J.", pl.age), r.x + r.w * 0.5, y + 21.0, 14.0, Theme::text2()); // roh
                ui.pill(r.x + r.w * 0.62, y + 15.0, *s, 34.0);
                ui.text(&ui.fit(hinweis, 13.0, r.w * 0.3), r.x + r.w * 0.68, y + 21.0, 13.0, if *kandidat { Theme::akzent() } else { Theme::achtung() });
            }
        }
        B::Elf(elf) => {
            // the pitch, then the four lines from the keeper up; each with photo, grade, name and crest
            rrect(r.x, r.y, r.w, r.h, 10.0, hex(0x2E7D32));
            for k in 0..6 { if k % 2 == 0 { draw_rectangle(r.x, r.y + k as f32 * r.h / 6.0, r.w, r.h / 6.0, alpha(hex(0xffffff), 0.05)); } }
            let lc = alpha(hex(0xffffff), 0.55);
            draw_rectangle_lines(r.x + 6.0, r.y + 6.0, r.w - 12.0, r.h - 12.0, 1.5, lc);
            draw_line(r.x + 6.0, r.y + r.h / 2.0, r.x + r.w - 6.0, r.y + r.h / 2.0, 1.5, lc);
            draw_circle_lines(r.x + r.w / 2.0, r.y + r.h / 2.0, 40.0, 1.5, lc);
            draw_rectangle_lines(r.x + r.w * 0.3, r.y + r.h - 50.0, r.w * 0.4, 44.0, 1.5, lc);
            let reihen: [(usize, usize, f32); 4] = [(0, 1, 0.86), (1, 4, 0.63), (5, 3, 0.39), (8, 3, 0.15)];
            for (ab, n, fy) in reihen {
                for j in 0..n {
                    let Some(&(p, note)) = elf.get(ab + j) else { continue };
                    let pl = &c.world.players[p];
                    let (cx, cy) = (r.x + r.w * (j as f32 + 1.0) / (n as f32 + 1.0), r.y + r.h * fy);
                    ui.player_photo(cx, cy - 10.0, 17.0, pl.id, &pl.name);
                    if note > 0.0 { ui.note_pill(cx + 30.0, cy - 18.0, note); }
                    let nach = pl.name.split_whitespace().last().unwrap_or(&pl.name).to_string();
                    let nw = ui.measure_semi(&nach, 14.0);
                    rrect(cx - nw / 2.0 - 6.0, cy + 9.0, nw + 12.0, 20.0, 6.0, alpha(hex(0x0b0f17), if pl.team == c.own as i32 { 0.9 } else { 0.65 }));
                    ui.text_semi(&nach, cx - nw / 2.0, cy + 24.0, 14.0, if pl.team == c.own as i32 { Theme::gold() } else { hex(0xffffff) });
                    if pl.team >= 0 { let k = pl.team as usize; let (c1, _) = club_color(c, k); ui.club_crest(cx - 22.0, cy - 24.0, 8.0, c.world.clubs[k].id, c1, &c.world.clubs[k].short); }
                }
            }
        }
        B::Kacheln(v) => {
            let kw = (r.w - 16.0) / 3.0;
            for (k, (p, gross, farbe, unter)) in v.iter().enumerate() {
                let (x, y) = (r.x + (k % 3) as f32 * (kw + 8.0), r.y + (k / 3) as f32 * 124.0);
                rrect(x, y, kw, 116.0, 12.0, Theme::erhoehung());
                draw_rectangle(x, y + 10.0, 4.0, 96.0, *farbe);
                let pl = &c.world.players[*p];
                ui.player_photo(x + 40.0, y + 44.0, 28.0, pl.id, &pl.name);
                ui.title(gross, x + kw - 14.0 - ui.measure_title(gross, 34.0), y + 52.0, 34.0, *farbe);
                ui.text_semi(&ui.fit(&pl.name, 15.0, kw - 20.0), x + 14.0, y + 92.0, 15.0, Theme::text());
                ui.text(&ui.fit(unter, 13.0, kw - 20.0), x + 14.0, y + 109.0, 13.0, Theme::text2());
            }
        }
        B::Pillen(v) => {
            let sw = r.w / 4.0;
            for (k, (p, note)) in v.iter().enumerate() {
                let (x, y) = (r.x + (k % 4) as f32 * sw, r.y + (k / 4) as f32 * 30.0);
                let nm = c.world.players[*p].name.split_whitespace().last().unwrap_or("").to_string();
                ui.text(&ui.fit(&nm, 13.0, sw - 52.0), x, y + 19.0, 13.0, Theme::text());
                let col = if *note >= 7.5 { Theme::gut() } else if *note >= 6.5 { Theme::akzent() } else if *note >= 5.5 { Theme::achtung() } else { Theme::problem() };
                rrect(x + sw - 46.0, y + 4.0, 38.0, 22.0, 11.0, col); ui.text_center(&format!("{note:.1}"), x + sw - 27.0, y + 20.0, 13.0, hex(0xffffff));
            }
        }
        B::Zeitung { name, schlagzeile, text } => {
            rrect(r.x, r.y, r.w - 6.0, r.h, 6.0, hex(0xF4EFE3));
            ui.text_semi(&ui.fit(&crate::inhalt::anzeige(name).to_uppercase(), 13.0, r.w - 30.0), r.x + 14.0, r.y + 20.0, 13.0, hex(0x6b5d43));
            draw_rectangle(r.x + 14.0, r.y + 26.0, r.w - 34.0, 1.5, hex(0x6b5d43));
            ui.title(&ui.fit(schlagzeile, 24.0, r.w - 30.0), r.x + 14.0, r.y + 56.0, 24.0, hex(0x1b1b1b));
            for (k, l) in ui.wrap(text, 16.0, r.w - 40.0).iter().take(8).enumerate() { ui.text(l, r.x + 14.0, r.y + 80.0 + k as f32 * 21.0, 16.0, hex(0x333333)); }
        }
        B::Siegel { titel } => {
            let (cx, cy) = (r.x + 56.0, r.y + 52.0);
            let puls = 1.0 + 0.05 * (get_time() as f32 * 3.0).sin();
            draw_triangle(vec2(cx - 22.0, cy + 30.0), vec2(cx - 6.0, cy + 30.0), vec2(cx - 28.0, cy + 54.0), Theme::problem());
            draw_triangle(vec2(cx + 22.0, cy + 30.0), vec2(cx + 6.0, cy + 30.0), vec2(cx + 28.0, cy + 54.0), Theme::problem());
            draw_circle(cx, cy, 40.0 * puls, Theme::gold()); draw_circle_lines(cx, cy, 33.0 * puls, 2.0, hex(0xffffff));
            art_symbol(Art::Meilenstein, cx, cy, 22.0, hex(0xffffff));
            ui.title(&ui.fit(titel, 20.0, r.w - 130.0), r.x + 112.0, r.y + 60.0, 20.0, Theme::text());
        }
        B::Phasen(v) => {
            let w = (r.w - 16.0) / v.len().max(1) as f32;
            for (k, (s, jetzt)) in v.iter().enumerate() {
                let x = r.x + k as f32 * (w + 8.0);
                rrect(x, r.y + 4.0, w, 30.0, 15.0, if *jetzt { Theme::akzent() } else { Theme::erhoehung() });
                ui.text_center(&ui.fit(s, 13.0, w - 10.0), x + w / 2.0, r.y + 24.0, 13.0, if *jetzt { hex(0xffffff) } else { Theme::text2() });
            }
        }
        B::Titel(s) => ui.title(&ui.fit(s, 20.0, r.w - 6.0), r.x, r.y + 22.0, 20.0, Theme::text()),
        B::Text(s) => { for (k, l) in ui.wrap(s, 16.0, r.w).iter().take(10).enumerate() { let y = r.y + 16.0 + k as f32 * 21.0; ui.text(l, r.x, y, 16.0, Theme::text()); links_zeichnen(ui, l, r.x, y); } }
        B::Reihe(v) => {
            let w = r.w / v.len().max(1) as f32;
            for (k, x) in v.iter().enumerate() { b_eins(ui, c, Rect::new(r.x + k as f32 * w, r.y, w - 10.0, r.h), x); }
        }
    }
}

/// The answers of a message: (label, option, yes, tone 0 friendly / 1 factual / 2 hard from the option's id).
pub fn antworten(m: &Msg) -> Vec<(String, u8, bool, u8)> {
    let ton = |k: &str| -> u8 {
        let k = k.to_lowercase();
        if [crate::id("lob"), crate::id("warm"), crate::id("ruhig"), crate::id("verstaend"), crate::id("einsaetze"), crate::id("geduld"), crate::id("annehm"), crate::id("foerder"), crate::id("vertrau")].iter().any(|x| k.contains(x)) { 0 }
        else if [crate::id("hart"), crate::id("kritik"), crate::id("druck"), crate::id("straf"), crate::id("verkauf"), crate::id("suchen"), crate::id("ablehn"), crate::id("abmahn")].iter().any(|x| k.contains(x)) { 2 } else { 1 }
    };
    if m.done { return vec![]; }
    match &m.action {
        Action::Wahl { optionen, .. } => optionen.iter().take(6).enumerate().map(|(k, o)| (o.1.clone(), k as u8, true, ton(&o.0))).collect(),
        Action::Complaint { .. } => [crate::inhalt::t("ui.pages.1_du_bekommst_deine_einsaetze"), crate::inhalt::t("ui.pages.2_zeig_es_im_training"), crate::inhalt::t("ui.pages.3_dann_suchen_wir_dir")]
            .iter().enumerate().map(|(k, s)| (s.to_string(), k as u8, true, [0u8, 1, 2][k])).collect(),
        Action::TransferOffer { .. } => vec![(crate::inhalt::t("kue.menu.verkauf").into(), 0, true, 2), (crate::inhalt::t("gs.ui.nahelegen").into(), 1, true, 1), (crate::inhalt::t("gs.ui.zum_bleiben").into(), 2, true, 0), (crate::inhalt::t("ui.post.ablehnen").into(), 0, false, 0)],
        Action::JobOffer { .. } => vec![(crate::inhalt::t("aktion.angebot_annehmen").into(), 0, true, 1), (crate::inhalt::t("lb.aktion.gespraech").into(), 1, true, 1), (crate::inhalt::t("ui.post.ablehnen").into(), 0, false, 0)],
        Action::Graduate { .. } => vec![(crate::inhalt::t("ui.pages.zu_den_profis").into(), 0, true, 0), (crate::inhalt::t("ui.pages.verein_verlassen").into(), 0, false, 2)],
        Action::Expiring { .. } => vec![(crate::inhalt::t("gs.anfrage.opt.verhandeln").into(), 0, true, 0), (crate::inhalt::t("gs.anfrage.opt.bedingung").into(), 1, true, 1), (crate::inhalt::t("gs.anfrage.opt.spaeter").into(), 2, true, 1), (crate::inhalt::t("ui.pages.gehen_lassen").into(), 0, false, 2)],
        _ => vec![],
    }
}
/// The visible consequence of an answer: the answer is played out on a copy of the career and compared (morale of the
/// players concerned, board, fans, media, team spirit, cash).
pub fn folgen(c: &Career, i: usize, option: u8, ja: bool) -> Vec<(String, f32)> {
    let mut k = c.clone();
    let wer = beteiligte(c, &c.inbox[i]);
    // morale counts the talk effects still to come (ps.gespr), the relationship counts at once
    let schnitt = |x: &Career, f: &dyn Fn(&Player) -> f32| if wer.is_empty() { 0.0 } else { wer.iter().map(|&p| f(&x.world.players[p])).sum::<f32>() / wer.len() as f32 };
    let moral = |p: &Player| p.morale + p.ps.gespr.iter().map(|g| g.1).sum::<f32>();
    let bez = |p: &Player| p.ps.bez;
    let vorher = (schnitt(c, &moral), c.mensch.vorstand, c.fans, c.media, c.mensch.teamgeist, c.club().balance, schnitt(c, &bez), c.mensch.versprechen.len());
    k.answer(i, ja, option);
    let nachher = (schnitt(&k, &moral), k.mensch.vorstand, k.fans, k.media, k.mensch.teamgeist, k.club().balance, schnitt(&k, &bez), k.mensch.versprechen.len());
    let t = |s: &str| crate::inhalt::t(&format!("ui.post.folge.{s}")).to_string();
    let mut v = vec![];
    if nachher.7 > vorher.7 { v.push((t("versprechen"), 0.0)); } // 0: shown without + or −
    for (name, d) in [(t("moral"), nachher.0 - vorher.0), (t("beziehung"), nachher.6 - vorher.6), (t("vorstand"), nachher.1 - vorher.1), (t("fans"), nachher.2 - vorher.2), (t("medien"), nachher.3 - vorher.3), (t("teamgeist"), nachher.4 - vorher.4)] {
        if d.abs() >= 0.5 { v.push((name, d)); }
    }
    let geld = nachher.5 - vorher.5;
    if geld.abs() >= 1000.0 { v.push((crate::tf!("ui.post.folge.kasse", format!("{}{}", if geld > 0.0 { "+" } else { "−" }, money(geld.abs()))), 0.0)); }
    v
}

/// The whole message: band, urgency, blocks, answers as cards, context on the right.
fn meldung_zeichnen(ui: &mut Ui, v: &mut View, c: &mut Career, rd: Rect, i: usize) {
    let m = c.inbox[i].clone();
    let art = art_von(&m);
    // band in the colour of the kind: symbol, sender with face, kind and time stamp
    let band = Rect::new(rd.x, rd.y, rd.w, 74.0);
    rrect(band.x, band.y, band.w, band.h, 14.0, art.farbe());
    ui.rahmen(band);
    draw_rectangle(band.x, band.y + band.h - 14.0, band.w, 14.0, art.farbe());
    draw_circle(band.x + 36.0, band.y + 37.0, 22.0, alpha(hex(0xffffff), 0.18));
    art_symbol(art, band.x + 36.0, band.y + 37.0, 14.0, hex(0xffffff));
    let wer = beteiligte(c, &m);
    // a sender that is an office (board, press, agents, fans) shows its person: name, face and role
    let person = crate::pool::absender_person(c.club().id, &m.from);
    let gesicht = person.as_ref().map(|x| x.0.clone()).unwrap_or_else(|| m.from.clone());
    match msg_spieler(c, &m).filter(|_| art == Art::Gespraech) {
        Some(p) => ui.player_photo(band.x + 90.0, band.y + 37.0, 22.0, c.world.players[p].id, &m.from),
        None => ui.avatar(band.x + 90.0, band.y + 37.0, 22.0, &gesicht),
    }
    let kopf = person.as_ref().map(|x| x.0.clone()).unwrap_or_else(|| crate::inhalt::anzeige(&m.from).to_string());
    ui.text_semi(&ui.fit(&kopf, 17.0, band.w - 380.0), band.x + 122.0, band.y + 32.0, 17.0, hex(0xffffff));
    let unter = crate::tf!("profil.kurz_text", art.name(), date::long(m.season, m.day));
    let unter = match &person { Some((_, rolle)) if !rolle.is_empty() => crate::tf!("profil.kurz_text", rolle, unter), _ => unter };
    ui.text(&ui.fit(&unter, 14.0, band.w - 380.0), band.x + 122.0, band.y + 54.0, 14.0, alpha(hex(0xffffff), 0.85));
    // urgency flag: Muss, Soll or Info, with the deadline as a count-down
    let (fw, fc) = match m.pflicht { 2 => (crate::inhalt::t("ui.post.muss"), Theme::problem()), 1 => (crate::inhalt::t("ui.post.soll"), Theme::achtung()), _ => (crate::inhalt::t("ui.post.info"), hex(0x5d6676)) };
    let fx = band.x + band.w - 210.0;
    draw_triangle(vec2(fx, band.y + 18.0), vec2(fx, band.y + 50.0), vec2(fx - 12.0, band.y + 34.0), fc);
    draw_rectangle(fx, band.y + 18.0, 196.0, 32.0, fc);
    let frist = if m.frist > 0 && !m.done && m.pflicht > 0 { let d = m.frist as i64 - c.abs() as i64; if d <= 0 { crate::inhalt::t("ui.post.frist_heute").into() } else { crate::tf!("ui.post.frist_noch", d) } } else { String::new() };
    let fahne = if frist.is_empty() { fw.to_string() } else { crate::tf!("profil.kurz_text", fw, frist) };
    ui.text_semi(&ui.fit(&fahne, 15.0, 184.0), fx + 8.0, band.y + 40.0, 15.0, hex(0xffffff));
    // title
    let mut y = band.y + band.h + 14.0;
    let kontext_w = 200.0;
    let haupt = Rect::new(rd.x + 18.0, y, rd.w - kontext_w - 44.0, rd.h - (y - rd.y) - 12.0);
    for l in ui.wrap(&m.title, 24.0, haupt.w).into_iter().take(2) { ui.title(&l, haupt.x, y + 22.0, 24.0, Theme::text()); y += 28.0; }
    y += 14.0;
    // answers as cards at the bottom, their consequences played out once per message
    let antw = antworten(&m);
    // long answers (content pool) get three lines: the cards grow from 70 to 88 units
    let kw0 = (haupt.w - 16.0) / 3.0;
    let lang = antw.iter().any(|a| ui.wrap(&a.0, 14.0, kw0 - 40.0).len() > 2);
    let (kh, zeilen) = if lang { (88.0, 3) } else { (70.0, 2) };
    let karten_h = if antw.is_empty() { 0.0 } else { 26.0 + ((antw.len() as f32) / 3.0).ceil() * (kh + 8.0) };
    let knopf_h = if matches!(m.action, Action::Report { .. }) || (matches!(m.action, Action::TransferOffer { .. }) && !m.done) || art == Art::Gegner { 50.0 } else { 0.0 };
    let bloecke = bausteine(c, &m);
    let (mut wer, mut vs) = (wer, vereine_von(c, &m));
    b_bezug(&bloecke, &mut wer, &mut vs);
    if vs.is_empty() { vs.push(c.own); }
    for l in [&mut wer, &mut vs] { let mut seen = std::collections::HashSet::new(); l.retain(|x| seen.insert(*x)); }
    // names in the text become links: the players and clubs of the message, and any club the text names
    let mut links: Vec<(String, Ziel)> = wer.iter().map(|&p| (c.world.players[p].name.clone(), Ziel::Spieler(p))).collect();
    let text: String = bloecke.iter().filter_map(|b| if let B::Text(t) = b { Some(t.as_str()) } else { None }).collect::<Vec<_>>().join(" ");
    for k in 0..c.world.clubs.len() { let n = &c.world.clubs[k].name; if n.len() > 3 && text.contains(n.as_str()) && !links.iter().any(|x| &x.0 == n) { links.push((n.clone(), Ziel::Verein(k))); if !vs.contains(&k) { vs.push(k); } } }
    links.sort_by_key(|x| std::cmp::Reverse(x.0.len()));
    LINKS.with(|l| *l.borrow_mut() = links);
    KLICK.with(|k| k.set(None));
    b_bereich(ui, v, c, Rect::new(haupt.x, y, haupt.w, haupt.y + haupt.h - y - karten_h - knopf_h - 8.0), &bloecke);
    LINKS.with(|l| l.borrow_mut().clear());
    match KLICK.with(|k| k.take()) {
        Some(Ziel::Spieler(p)) => { v.modal = Modal::Player(p); ui.clicked = false; }
        Some(Ziel::Verein(k)) => { v.sel_club = Some(k); v.go(Page::Vereine); ui.clicked = false; return; }
        None => {}
    }
    if m.done { ui.text(crate::inhalt::t("md.ui.ok.erledigt"), haupt.x, haupt.y + haupt.h - 6.0, 16.0, Theme::gut()); }
    let ky = haupt.y + haupt.h - karten_h - knopf_h;
    if !antw.is_empty() {
        let key = (i, c.inbox.len(), c.abs());
        if v.post_folgen.as_ref().map_or(true, |x| x.0 != key) { let f = antw.iter().map(|a| folgen(c, i, a.1, a.2)).collect(); v.post_folgen = Some((key, f)); }
        ui.text_semi(crate::inhalt::t("ui.post.deine_antwort"), haupt.x, ky + 16.0, 14.0, Theme::gedaempft());
        let kw = (haupt.w - 16.0) / 3.0;
        let mut geklickt = None;
        for (k, (txt, opt, ja, ton)) in antw.iter().enumerate() {
            let r = Rect::new(haupt.x + (k % 3) as f32 * (kw + 8.0), ky + 26.0 + (k / 3) as f32 * (kh + 8.0), kw, kh);
            rrect(r.x, r.y, r.w, r.h, 10.0, if ui.hover(r) { Theme::ueberlagerung() } else { Theme::erhoehung() });
            let tc = [Theme::gut(), Theme::akzent(), Theme::problem()][*ton as usize % 3];
            match ton { 0 => draw_circle(r.x + 16.0, r.y + 18.0, 7.0, tc), 2 => draw_triangle(vec2(r.x + 9.0, r.y + 25.0), vec2(r.x + 23.0, r.y + 25.0), vec2(r.x + 16.0, r.y + 11.0), tc), _ => draw_rectangle(r.x + 10.0, r.y + 12.0, 12.0, 12.0, tc) }
            // long answers of the content pool: smaller, two lines, the rest with „…“; the whole answer on hovering the card
            let zl = ui.wrap(txt, 14.0, r.w - 40.0);
            for (z, l) in zl.iter().take(zeilen).enumerate() {
                let l = if z + 1 == zeilen && zl.len() > zeilen { ui.fit(&format!("{l} {}", zl[z + 1]), 14.0, r.w - 40.0) } else { l.clone() };
                ui.text_semi(&l, r.x + 32.0, r.y + 22.0 + z as f32 * 17.0, 14.0, Theme::text());
            }
            if ui.hover(r) && zl.len() > zeilen { *ui.tooltip.borrow_mut() = Some(txt.clone()); }
            let folg = v.post_folgen.as_ref().and_then(|x| x.1.get(k)).cloned().unwrap_or_default();
            let ft = if folg.is_empty() { crate::inhalt::t("ui.post.folge.keine").to_string() } else { folg.iter().take(3).map(|(n, d)| if *d == 0.0 { n.clone() } else { crate::tf!("ui.post.folge.eine", n, if *d > 0.0 { "+" } else { "−" }) }).collect::<Vec<_>>().join(", ") };
            ui.text(&ui.fit(&ft, 13.0, r.w - 16.0), r.x + 10.0, r.y + r.h - 10.0, 13.0, Theme::text2());
            if ui.hit(r) || (v.modal == Modal::None && !v.market_focus && is_key_pressed([KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6][k])) { geklickt = Some((*opt, *ja)); }
        }
        if let Some((opt, ja)) = geklickt { c.answer(i, ja, opt); v.post_folgen = None; v.toast(crate::inhalt::t("vg.ausgang.kurz.abgeschlossen"), true); return; }
        // delegate: the assistant answers it (Fabian 23.11.)
        if !m.done && ui.button(Rect::new(haupt.x + haupt.w - 220.0, ky - 6.0, 220.0, 28.0), crate::inhalt::t("pk.delegieren"), false) { c.delegieren(i); v.post_folgen = None; v.toast(crate::inhalt::t("ui.vorgang.vom_co_trainer_beantwortet"), true); return; }
    }
    // the buttons of a scout report and an offer
    let by = haupt.y + haupt.h - 44.0;
    match m.action {
        Action::Report { pid } => {
            if ui.button(Rect::new(haupt.x, by, 160.0, 38.0), crate::inhalt::t("kue.menu.profil"), false) { v.modal = Modal::Player(pid); }
            let gemerkt = c.merkliste.contains(&pid);
            if ui.button(Rect::new(haupt.x + 168.0, by, 150.0, 38.0), crate::inhalt::t("ui.post.beobachten"), gemerkt) && !gemerkt { c.merkliste.push(pid); v.toast(crate::inhalt::t("ui.post.auf_merkliste"), true); }
            if c.world.players[pid].team != c.own as i32 && ui.button_gold(Rect::new(haupt.x + 326.0, by, (haupt.w - 330.0).min(184.0), 38.0), crate::inhalt::t("ui.post.angebot")) { match c.open_neg(pid) { Ok(n) => v.modal = Modal::Neg(n), Err(e) => v.toast(&e, false) } }
        }
        Action::TransferOffer { pid, club, fee } if !m.done => {
            if ui.button(Rect::new(haupt.x, by, 220.0, 38.0), crate::inhalt::t("aktion.vertragsgespraech"), false) { c.inbox[i].done = true; let k = c.verkauf_verhandeln(pid, club, fee); v.modal = Modal::Neg(k); }
        }
        _ if art == Art::Gegner => {
            if ui.button(Rect::new(haupt.x, by, 240.0, 38.0), crate::inhalt::t("ui.post.zur_vorbereitung"), false) { v.go(Page::Vorbereitung); }
        }
        _ => {}
    }
    // context on the right: the players concerned with photo and strength, the crests of the clubs
    let kx = rd.x + rd.w - kontext_w - 14.0;
    let kr = Rect::new(kx, band.y + band.h + 12.0, kontext_w, rd.h - band.h - 24.0);
    rrect(kr.x, kr.y, kr.w, kr.h, 10.0, Theme::grund_tief());
    ui.text_semi(crate::inhalt::t("ui.post.beteiligt"), kr.x + 12.0, kr.y + 22.0, 13.0, Theme::gedaempft());
    let mut ky2 = kr.y + 40.0;
    for &p in wer.iter().take(5) {
        let pl = &c.world.players[p];
        let r = Rect::new(kr.x + 6.0, ky2, kr.w - 12.0, 40.0);
        if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 6.0, Theme::erhoehung()); }
        ui.player_photo(r.x + 20.0, r.y + 20.0, 15.0, pl.id, &pl.name);
        ui.text(&ui.fit(&pl.name, 14.0, r.w - 90.0), r.x + 42.0, r.y + 25.0, 14.0, Theme::text());
        ui.pill(r.x + r.w - 20.0, r.y + 20.0, pl.ovr, 30.0);
        if ui.hit(r) { v.modal = Modal::Player(p); }
        ky2 += 44.0;
    }
    if wer.is_empty() { ui.text(crate::inhalt::t("ui.post.niemand"), kr.x + 12.0, ky2 + 8.0, 13.0, Theme::text2()); ky2 += 24.0; }
    // the clubs with crest and name, a click opens the club
    for &club in vs.iter().take(4) {
        let r = Rect::new(kr.x + 6.0, ky2, kr.w - 12.0, 40.0);
        if ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 6.0, Theme::erhoehung()); }
        let (c1, _) = club_color(c, club);
        ui.club_crest(r.x + 20.0, r.y + 20.0, 15.0, c.world.clubs[club].id, c1, &c.world.clubs[club].short);
        ui.text(&ui.fit(&c.world.clubs[club].name, 14.0, r.w - 50.0), r.x + 42.0, r.y + 25.0, 14.0, Theme::text());
        if ui.hit(r) { v.sel_club = Some(club); v.go(Page::Vereine); return; }
        ky2 += 44.0;
    }
}

/// One row of the inbox list: kind colour and symbol, photo or crest, subject (bold with a dot when unread), a preview
/// line and the deadline.
fn meldung_zeile(ui: &mut Ui, c: &Career, r: Rect, m: &Msg, sel: bool) {
    let art = art_von(m);
    if sel || ui.hover(r) { rrect(r.x, r.y, r.w, r.h, 6.0, if sel { alpha(art.farbe(), 0.14) } else { Theme::erhoehung() }); }
    draw_rectangle(r.x, r.y + 4.0, 4.0, r.h - 8.0, art.farbe());
    draw_circle(r.x + 22.0, r.y + 22.0, 12.0, art.farbe());
    art_symbol(art, r.x + 22.0, r.y + 22.0, 7.5, hex(0xffffff));
    let bild_x = r.x + 56.0;
    if let Some(p) = beteiligte(c, m).first() { ui.player_photo(bild_x, r.y + r.h / 2.0, 16.0, c.world.players[*p].id, &c.world.players[*p].name); }
    else if let Some(&k) = vereine_von(c, m).first() { let (c1, _) = club_color(c, k); ui.club_crest(bild_x, r.y + r.h / 2.0, 15.0, c.world.clubs[k].id, c1, &c.world.clubs[k].short); }
    else { ui.avatar(bild_x, r.y + r.h / 2.0, 15.0, &sprecher(c, &m.from)); }
    let tx = r.x + 80.0;
    let frist = if m.frist > 0 && !m.done && m.pflicht > 0 { let d = m.frist as i64 - c.abs() as i64; Some(if d <= 0 { crate::inhalt::t("ui.post.frist_heute").to_string() } else { crate::tf!("ui.post.frist_kurz", d) }) } else { None };
    let rechts = frist.as_ref().map_or(0.0, |f| ui.measure(f, 13.0) + 14.0);
    if !m.read { draw_circle(r.x + r.w - 10.0 - rechts, r.y + 16.0, 4.0, art.farbe()); }
    let t = ui.fit(&m.title, 17.0, r.w - (tx - r.x) - 22.0 - rechts);
    if m.read { ui.text(&t, tx, r.y + 22.0, 17.0, Theme::text2()); } else { ui.text_semi(&t, tx, r.y + 22.0, 17.0, Theme::text()); }
    let vorschau = m.body.lines().next().unwrap_or("").to_string();
    ui.text(&ui.fit(&vorschau, 13.0, r.w - (tx - r.x) - 12.0), tx, r.y + 42.0, 13.0, Theme::gedaempft());
    if let Some(f) = frist { ui.text_right(&f, r.x + r.w - 8.0, r.y + 22.0, 13.0, if m.pflicht == 2 { Theme::problem() } else { Theme::achtung() }); }
}

impl Career {
    /// Marks the newest message with its kind and the players it is about.
    /// The newest message gets a picture card: tiles of players with a number each (development, awards).
    pub fn meldung_karte(&mut self, art: Art, karte: &str, spieler: Vec<(usize, f32)>) {
        if let Some(m) = self.inbox.first_mut() { m.art = art.id().into(); m.karte = karte.into(); m.spieler = spieler.iter().map(|x| x.0).collect(); m.werte = spieler.iter().map(|x| x.1).collect(); }
    }
    pub fn meldung_art(&mut self, art: Art, spieler: Vec<usize>) {
        if let Some(m) = self.inbox.first_mut() { m.art = art.id().into(); m.spieler = spieler; }
    }
    /// Once a week: the morale of the own players for the morale curve, and the week in the world as a message.
    pub fn post_woche(&mut self) {
        for p in self.squad() {
            let (id, mo) = (self.world.players[p].id, self.world.players[p].morale);
            let v = self.moral_verlauf.entry(id).or_default();
            v.push(mo);
            if v.len() > 12 { v.remove(0); }
        }
        let l = self.league();
        let (bis, von) = (self.world.day, self.world.day.saturating_sub(7));
        let n = self.world.leagues[l].fixtures.iter().filter(|f| f.day > von && f.day <= bis && f.hg >= 0).count();
        if n == 0 { return; }
        let rang = self.world.rank_of(self.own);
        self.msg(0, crate::inhalt::t("ui.post.redaktion"), crate::inhalt::t("ui.post.woche_titel"), &crate::tf!("ui.post.woche_text", n, rang), Action::None);
        self.meldung_art(Art::Woche, vec![]);
    }
    /// One example message of every kind, from the real game state (previews and the test).
    pub fn beispiel_meldungen(&mut self) {
        let sq = self.squad();
        let own = self.own;
        let fremd: Vec<usize> = (0..self.world.players.len()).filter(|&p| self.world.players[p].team >= 0 && self.world.players[p].team != own as i32 && self.world.players[p].youth == 0).collect();
        let star = *fremd.iter().max_by(|a, b| self.world.players[**a].ovr.partial_cmp(&self.world.players[**b].ovr).unwrap()).unwrap();
        let k = self.world.players[star].team as usize;
        let t = |s: &str| crate::inhalt::t(&format!("ui.post.beispiel.{s}"));
        let co = self.co_name.clone();
        self.msg(0, &co, t("allgemein_titel"), t("allgemein_text"), Action::None); self.meldung_art(Art::Allgemein, vec![]);
        self.msg(0, crate::inhalt::t("ui.post.redaktion"), t("meilenstein_titel"), t("meilenstein_text"), Action::None); self.meldung_art(Art::Meilenstein, vec![]);
        let y = self.world.clubs[own].squad.iter().copied().find(|&p| self.world.players[p].youth > 0).unwrap_or(sq[sq.len() - 2]);
        let yn = self.world.players[y].name.clone();
        self.msg(1, crate::inhalt::t("meldung.absender.jugendleiter"), &crate::tf!("ui.post.beispiel.talent_titel", yn), t("talent_text"), Action::Graduate { pid: y });
        self.msg(4, crate::inhalt::t("ui.post.zeitung"), t("presse_titel"), t("presse_text"), Action::None); self.meldung_art(Art::Presse, vec![]);
        let v = sq[sq.len() / 2];
        self.world.players[v].inj = 24;
        self.world.players[v].inj_name = crate::world::injury_names(24)[0].to_string();
        let vn = self.world.players[v].name.clone();
        self.msg(1, crate::inhalt::t("med.vg.absender"), &crate::tf!("ui.game.verletzung", vn), t("verletzung_text"), Action::None); self.meldung_art(Art::Verletzung, vec![v]);
        self.msg(5, crate::inhalt::t("ui.anzeige.Präsidium"), t("vorstand_titel"), t("vorstand_text"), Action::None); self.meldung_art(Art::Vorstand, vec![]);
        let g = sq[sq.len() - 1];
        let gn = self.world.players[g].name.clone();
        self.msg(1, &gn, &crate::tf!("ui.post.beispiel.gespraech_titel", gn), t("gespraech_text"), Action::Complaint { pid: g });
        let a = sq[1];
        let (kn, an) = (self.world.clubs[k].name.clone(), self.world.players[a].name.clone());
        let fee = self.world.players[a].value * 1.1;
        self.msg(2, crate::inhalt::t("ui.game.transferabteilung"), &crate::tf!("ui.post.beispiel.angebot_titel", kn, an), t("angebot_text"), Action::TransferOffer { pid: a, club: k, fee });
        // a played match for the match report
        let gegner = self.world.leagues[self.league()].teams.iter().copied().find(|&x| x != own).unwrap();
        let res = self.world.play(own, gegner, 0, None, None);
        let (hg, ag) = (res.hg, res.ag);
        self.last_result = Some(res);
        let (os, gs) = (self.world.clubs[own].short.clone(), self.world.clubs[gegner].short.clone());
        self.msg(0, crate::inhalt::t("bericht.titel"), &crate::tf!("ui.post.beispiel.spiel_titel", os, hg, ag, gs), t("spiel_text"), Action::None); self.meldung_art(Art::Spiel, vec![]);
        self.msg(0, crate::inhalt::t("ui.post.redaktion"), crate::inhalt::t("ui.post.woche_titel"), t("woche_text"), Action::None); self.meldung_art(Art::Woche, vec![]);
        self.gegnerbericht_senden();
        let sn = self.world.players[star].name.clone();
        self.msg(6, crate::inhalt::t("ui.post.scout_name"), &crate::tf!("ui.post.beispiel.scout_titel", sn), t("scout_text"), Action::Report { pid: star });
        let heute = self.abs();
        for m in self.inbox.iter_mut().take(12) { if !matches!(m.action, Action::None) { m.pflicht = m.pflicht.max(1); m.frist = heute + 3; } }
    }
}

#[cfg(test)]
mod post_t {
    use super::*;
    /// Every kind has an example, and every layout shows a picture and a number or a graphic; answers are cards with
    /// a visible consequence.
    #[test]
    fn jede_art_hat_bild_und_zahl() {
        let w = crate::world::World::load(include_str!("../assets/world.json"), 5);
        let own = w.clubs.iter().position(|c| c.name.contains("Bayern")).unwrap_or(0);
        let mut c = Career::new(w, own, "Test");
        c.inbox.clear();
        c.beispiel_meldungen();
        for a in ARTEN {
            let m = c.inbox.iter().find(|m| art_von(m) == a).unwrap_or_else(|| panic!("kein Beispiel für {a:?}"));
            let bs = bausteine(&c, m);
            assert!(hat_bild(&bs), "{a:?} ohne Bild");
            assert!(hat_zahl(&bs), "{a:?} ohne Zahl oder Grafik");
        }
        let i = c.inbox.iter().position(|m| matches!(m.action, Action::Complaint { .. })).unwrap();
        let antw = antworten(&c.inbox[i]);
        assert!((3..=6).contains(&antw.len()));
        assert!(antw.iter().any(|a| !folgen(&c, i, a.1, a.2).is_empty()), "mindestens eine Antwort hat eine sichtbare Folge");
    }
}
