//! The world map (Gesamtauftrag Punkt 13): country outlines (Natural Earth 1:50m, assets/geo/laender.json) and club
//! positions (Wikidata: home venue coordinates, assets/geo/vereine.json), both built once by tools/geo_bauen.py.
//! A club without a match gets the middle of its country with a small offset and counts as "Lage geschätzt".
//! Everything is offline; CalcioM never goes online.
use crate::world::World;
use crate::ui::*;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct Land { pub iso: String, pub iso2: String, pub de: String, pub en: String, pub ringe: Vec<Vec<(f32, f32)>> }
pub struct Daten { pub laender: Vec<Land>, vereine: HashMap<String, Vec<(f32, f32, String)>>, orte: HashMap<(String, String), (f32, f32, u32)> }

pub fn daten() -> &'static Daten {
    static D: OnceLock<Daten> = OnceLock::new();
    D.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/geo/laender.json")).expect("laender.json");
        let laender = v["laender"].as_array().map(|a| a.iter().map(|l| Land {
            iso: l["iso"].as_str().unwrap_or("").into(), iso2: l["iso2"].as_str().unwrap_or("").into(), de: l["de"].as_str().unwrap_or("").into(), en: l["en"].as_str().unwrap_or("").into(),
            ringe: l["ringe"].as_array().map(|r| r.iter().map(|ring| ring.as_array().map(|pts| pts.iter().filter_map(|p| Some((p.get(0)?.as_f64()? as f32, p.get(1)?.as_f64()? as f32))).collect()).unwrap_or_default()).collect()).unwrap_or_default(),
        }).collect()).unwrap_or_default();
        let w: serde_json::Value = serde_json::from_str(include_str!("../assets/geo/vereine.json")).expect("vereine.json");
        let mut vereine: HashMap<String, Vec<(f32, f32, String)>> = HashMap::new();
        for e in w["vereine"].as_array().into_iter().flatten() {
            let (lon, lat) = (e["lon"].as_f64().unwrap_or(0.0) as f32, e["lat"].as_f64().unwrap_or(0.0) as f32);
            let land = e["land"].as_str().unwrap_or("").to_string();
            for n in [e["de"].as_str(), e["en"].as_str()].into_iter().flatten() {
                let k = crate::trainerbank::norm(n);
                if !k.is_empty() { vereine.entry(k).or_default().push((lon, lat, land.clone())); }
            }
        }
        let o: serde_json::Value = serde_json::from_str(include_str!("../assets/geo/orte.json")).expect("orte.json");
        let mut orte: HashMap<(String, String), (f32, f32, u32)> = HashMap::new();
        for e in o["orte"].as_array().into_iter().flatten() {
            let (Some(n), Some(l)) = (e.get(0).and_then(|x| x.as_str()), e.get(1).and_then(|x| x.as_str())) else { continue };
            let v = (e.get(2).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32, e.get(3).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32, e.get(4).and_then(|x| x.as_u64()).unwrap_or(0) as u32);
            let k = (crate::trainerbank::norm(n), l.to_string());
            if orte.get(&k).map_or(true, |x| x.2 < v.2) { orte.insert(k, v); }
        }
        Daten { laender, vereine, orte }
    })
}

/// The map country of a league country name (German display names; England, Scotland and Wales lie in the UK).
pub fn land_von(name: &str) -> Option<&'static Land> {
    let n = match name { "England" | "Schottland" | "Wales" | "Nordirland" => "Vereinigtes Königreich", "USA" => "Vereinigte Staaten", "VAE" => "Vereinigte Arabische Emirate", "China" => "Volksrepublik China", x => x };
    daten().laender.iter().find(|l| l.de == n || l.en == n)
}

/// Middle of a country: the mean of its largest ring.
pub fn mitte(l: &Land) -> (f32, f32) {
    let r = l.ringe.iter().max_by_key(|r| r.len()).cloned().unwrap_or_default();
    let n = r.len().max(1) as f32;
    (r.iter().map(|p| p.0).sum::<f32>() / n, r.iter().map(|p| p.1).sum::<f32>() / n)
}

/// Where a club lies: (lon, lat, estimated). The Wikidata point of the same normalized name (in the same country when
/// several), otherwise the middle of the league's country with an offset fixed by the club id.
pub fn lage(w: &World, club: usize) -> Option<(f32, f32, bool)> {
    thread_local! { static CACHE: std::cell::RefCell<HashMap<u32, Option<(f32, f32, bool)>>> = Default::default(); }
    let cl = &w.clubs[club];
    let id = cl.id;
    if let Some(x) = CACHE.with(|m| m.borrow().get(&id).copied()) { return x; }
    // the league's country by its name, else by the nationality most of its players have (nationen.rs)
    let land = { let n = w.leagues[cl.league].land.clone(); if land_von(&n).is_some() { n } else { let m = crate::nationen::name(w, w.land_nat(cl.league)); if land_von(&m).is_some() { m } else { liga_land_wikidata(w, cl.league) } } };
    let k = crate::trainerbank::norm(&cl.name);
    let treffer = daten().vereine.get(&k).and_then(|v| {
        let l = land_von(&land);
        v.iter().find(|x| l.map_or(false, |l| x.2 == l.de || x.2 == l.en || (l.iso == "GBR" && x.2.contains(crate::id("Vereinigtes"))))).or(if v.len() == 1 || l.is_none() { v.first() } else { None }).map(|x| (x.0, x.1, false))
    });
    // otherwise the town in the club's name (GeoNames: "Sunderland AFC" -> Sunderland), the biggest one that fits
    let treffer = treffer.or_else(|| {
        let l = land_von(&land)?;
        let t: Vec<&str> = k.split(' ').collect();
        let mut kand: Vec<String> = t.iter().map(|x| x.to_string()).collect();
        for w in t.windows(2) { kand.push(w.join(" ")); }
        kand.iter().filter_map(|n| daten().orte.get(&(n.clone(), l.iso2.clone()))).max_by_key(|x| x.2).map(|x| (x.0, x.1, false))
    });
    let aus = treffer.or_else(|| land_von(&land).map(|l| {
        let (x, y) = mitte(l);
        let h = (id.wrapping_mul(2654435761)) as f32 / u32::MAX as f32;
        let a = h * std::f32::consts::TAU;
        (x + a.cos() * 1.2 * h, y + a.sin() * 0.8 * h, true)
    }));
    CACHE.with(|m| m.borrow_mut().insert(id, aus));
    aus
}

/// The country of a league from Wikidata: the country most of its clubs that have an entry are listed in.
/// The same, but only when it is clear: at least three clubs and half of the league agree (for naming a country).
pub fn liga_land_sicher(w: &World, l: usize) -> Option<String> {
    let mut z: HashMap<String, usize> = HashMap::new();
    for &k in &w.leagues[l].teams {
        if let Some(v) = daten().vereine.get(&crate::trainerbank::norm(&w.clubs[k].name)) { if let Some(x) = v.iter().find(|x| !x.2.is_empty()) { *z.entry(x.2.clone()).or_default() += 1; } }
    }
    let (land, n) = z.into_iter().max_by_key(|x| x.1)?;
    (n >= 3 && n * 2 >= w.leagues[l].teams.len()).then_some(land)
}
pub fn liga_land_wikidata(w: &World, l: usize) -> String {
    let mut z: HashMap<String, usize> = HashMap::new();
    for &k in &w.leagues[l].teams {
        if let Some(v) = daten().vereine.get(&crate::trainerbank::norm(&w.clubs[k].name)) { for x in v { if !x.2.is_empty() { *z.entry(x.2.clone()).or_default() += 1; } } }
    }
    z.into_iter().max_by_key(|x| x.1).map(|x| x.0).unwrap_or_default()
}

/// Sutherland-Hodgman: the part of a polygon inside the rectangle (so the country fill never spills over the map).
fn clip(poly: &[Vec2], r: Rect) -> Vec<Vec2> {
    let kanten: [fn(Vec2, Rect) -> f32; 4] = [|p, r| p.x - r.x, |p, r| r.x + r.w - p.x, |p, r| p.y - r.y, |p, r| r.y + r.h - p.y];
    let mut out = poly.to_vec();
    for f in kanten {
        if out.is_empty() { break; }
        let ein = std::mem::take(&mut out);
        for i in 0..ein.len() {
            let (a, b) = (ein[i], ein[(i + 1) % ein.len()]);
            let (da, db) = (f(a, r), f(b, r));
            if da >= 0.0 { out.push(a); }
            if (da >= 0.0) != (db >= 0.0) { out.push(a + (b - a) * (da / (da - db))); }
        }
    }
    out
}

/// Mercator: y grows to the north.
fn merc(lat: f32) -> f32 { let p = lat.clamp(-80.0, 80.0).to_radians(); ((std::f32::consts::FRAC_PI_4 + p / 2.0).tan()).ln().to_degrees() }

/// Draws the map in `r` around the given clubs (their countries), highlights `markiert` and returns a clicked club.
/// `zoom` and `mitte_off` come from the view (mouse wheel zooms, dragging moves).
pub fn karte(ui: &mut Ui, r: Rect, w: &World, vereine: &[usize], markiert: Option<usize>, zoom: &mut f32, off: &mut (f32, f32)) -> Option<usize> {
    let punkte: Vec<(usize, f32, f32, bool)> = vereine.iter().filter_map(|&k| lage(w, k).map(|(x, y, g)| (k, x, merc(y), g))).collect();
    if punkte.is_empty() { return None; }
    // frame: all points plus a margin, then zoom and offset
    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for p in &punkte { x0 = x0.min(p.1); x1 = x1.max(p.1); y0 = y0.min(p.2); y1 = y1.max(p.2); }
    let rand = ((x1 - x0).max(y1 - y0) * 0.25).max(2.0);
    let (cx, cy) = ((x0 + x1) / 2.0 + off.0, (y0 + y1) / 2.0 + off.1);
    let spanne = ((x1 - x0 + 2.0 * rand) / r.w).max((y1 - y0 + 2.0 * rand) / r.h) / zoom.max(0.2);
    let bild = |x: f32, y: f32| vec2(r.x + r.w / 2.0 + (x - cx) / spanne, r.y + r.h / 2.0 - (y - cy) / spanne);
    rrect(r.x, r.y, r.w, r.h, 10.0, if Theme::dark() { hex(0x0d1b2a) } else { hex(0xdbe9f4) });
    let land_farbe = if Theme::dark() { hex(0x1f2a37) } else { hex(0xf3efe6) };
    let linie = alpha(Theme::text2(), 0.6);
    let innen = |p: Vec2| p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;
    for l in &daten().laender {
        for ring in &l.ringe {
            let pts: Vec<Vec2> = ring.iter().map(|&(x, y)| bild(x, merc(y))).collect();
            if !pts.iter().any(|&p| innen(p)) { continue; }
            // a simple fan fill from the ring's first point (good enough for the mostly convex country shapes at this scale)
            for w in pts.windows(2).skip(1) { let d = clip(&[pts[0], w[0], w[1]], r); for k in 1..d.len().saturating_sub(1) { draw_triangle(d[0], d[k], d[k + 1], land_farbe); } }
            for w in pts.windows(2) { if innen(w[0]) && innen(w[1]) { draw_line(w[0].x, w[0].y, w[1].x, w[1].y, 1.0, linie); } }
        }
    }
    // the clubs: small dots, the marked one big with its crest
    let mut geklickt = None;
    let mut nah: Option<(f32, usize)> = None;
    for &(k, x, y, geschaetzt) in &punkte {
        let p = bild(x, y);
        if !innen(p) { continue; }
        let d = p.distance(ui.mouse);
        if d < 10.0 && nah.map_or(true, |n| d < n.0) { nah = Some((d, k)); }
        if Some(k) == markiert { continue; }
        draw_circle(p.x, p.y, 4.5, if geschaetzt { alpha(Theme::akzent(), 0.45) } else { Theme::akzent() });
        draw_circle_lines(p.x, p.y, 4.5, 1.0, Theme::flaeche());
    }
    if let Some(m) = markiert { if let Some(&(_, x, y, _)) = punkte.iter().find(|p| p.0 == m) {
        let p = bild(x, y);
        if innen(p) {
            draw_circle(p.x, p.y, 17.0, Theme::gold());
            let k = &w.clubs[m];
            ui.club_crest(p.x, p.y, 14.0, k.id, hex(k.c1), &k.short.clone());
        }
    } }
    if let Some((_, k)) = nah {
        let p = ui.mouse;
        let t = w.clubs[k].name.clone();
        let tw = ui.measure(&t, 13.0) + 16.0;
        rrect(p.x + 10.0, p.y - 26.0, tw, 22.0, 6.0, Theme::erhoehung());
        ui.text(&t, p.x + 18.0, p.y - 10.0, 13.0, Theme::text());
        if ui.hit(Rect::new(p.x - 10.0, p.y - 10.0, 20.0, 20.0)) { geklickt = Some(k); }
    }
    // zoom with the wheel, move by dragging (only while the mouse is over the map)
    if Rect::new(r.x, r.y, r.w, r.h).contains(ui.mouse) {
        if ui.wheel != 0.0 { *zoom = (*zoom * if ui.wheel > 0.0 { 1.2 } else { 1.0 / 1.2 }).clamp(0.3, 12.0); }
        if is_mouse_button_down(MouseButton::Left) && nah.is_none() { let d = mouse_delta_position(); off.0 += d.x * spanne * screen_width() / 2.0; off.1 -= d.y * spanne * screen_height() / 2.0; }
    }
    if punkte.iter().any(|p| p.3) { ui.text(crate::inhalt::t("ui.karte.geschaetzt"), r.x + 10.0, r.y + r.h - 10.0, 12.0, Theme::gedaempft()); }
    geklickt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_schneidet_auf_das_rechteck() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        let d = clip(&[vec2(-5.0, 5.0), vec2(5.0, 5.0), vec2(5.0, 20.0)], r);
        assert!(d.len() >= 3 && d.iter().all(|p| p.x >= -1e-3 && p.x <= 10.001 && p.y >= -1e-3 && p.y <= 10.001));
        assert!(clip(&[vec2(20.0, 20.0), vec2(30.0, 20.0), vec2(25.0, 30.0)], r).is_empty());
    }

    #[test]
    fn karte_kennt_laender_und_lagen() {
        assert!(daten().laender.len() > 200);
        let de = land_von("Deutschland").unwrap();
        let (x, y) = mitte(de);
        assert!((5.0..16.0).contains(&x) && (47.0..56.0).contains(&y), "{x} {y}");
        assert!(land_von("England").is_some() && land_von("Schottland").is_some());
    }
}

#[cfg(test)]
mod abdeckung {
    use super::*;
    #[test]
    fn jeder_verein_hat_einen_punkt() {
        let d = crate::fl27::sider().join("livecpk/_FL26-27-Def/common/etc/pesdb");
        if !crate::fl27::echte_welt(&d) { return; }
        let w = crate::world::World::from_pesdb(&d, 7).unwrap();
        let own = w.clubs.iter().position(|c| c.id == 396).unwrap();
        let c = crate::game::Career::new(w, own, "Test");
        let l: Vec<Option<(f32, f32, bool)>> = (0..c.world.clubs.len()).map(|k| lage(&c.world, k)).collect();
        let mit = l.iter().filter(|x| x.is_some()).count();
        let genau = l.iter().filter(|x| matches!(x, Some((_, _, false)))).count();
        eprintln!("Vereine {}, mit Punkt {mit}, genau {genau}", l.len());
        let mut ohne: std::collections::HashMap<String, usize> = Default::default();
        for (k, x) in l.iter().enumerate() { if x.is_none() { let lg = c.world.clubs[k].league; *ohne.entry(format!("{} / {} / {}", c.world.leagues[lg].land, crate::nationen::name(&c.world, c.world.land_nat(lg)), c.world.land_nat(lg))).or_default() += 1; } }
        eprintln!("ohne Punkt: {ohne:?}");
        assert!(mit as f32 >= l.len() as f32 * 0.95, "fast jeder Verein hat einen Punkt");
        // Bayern liegt in München, Sunderland im Nordosten Englands
        let bay = c.world.clubs.iter().position(|k| k.name.contains("Bayern")).unwrap();
        let (x, y, _) = lage(&c.world, bay).unwrap();
        assert!((x - 11.6).abs() < 0.5 && (y - 48.15).abs() < 0.4, "{x} {y}");
        let (x, y, _) = lage(&c.world, own).unwrap();
        assert!((x + 1.4).abs() < 0.5 && (y - 54.9).abs() < 0.4, "{x} {y}");
    }
}
