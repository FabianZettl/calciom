//! Pass photo of a player without a photo (Fabian 25.11.): head and shoulders in the club's kit, drawn in 3D like a
//! studio portrait, serious rather than childish. Skin, hair colour, bald, afro and beard come from FL27's appearance
//! record (gesichter::aussehen_von; a new player wears his face donor's), else from a fixed dice of his id. Drawn once
//! per player into a render target and kept (at most KEEP), not every frame. Built after the cloud prototype.
use crate::gesichter::Aussehen;
use macroquad::prelude::*;
use macroquad::models::{Mesh, Vertex};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Comparison, CullFace, Equation, PipelineParams, UniformType};
use std::cell::RefCell;
use std::collections::HashMap;

const KEEP: usize = 200;
/// Render size (2× the profile box for a smooth picture, the prototype's 388:438 shape).
const BREITE: u32 = 194;
const HOEHE: u32 = 219;

pub fn haut_stufe(s: u8) -> [u8; 4] { [[246, 216, 194, 255], [236, 200, 172, 255], [222, 180, 146, 255], [196, 148, 110, 255], [150, 104, 74, 255], [100, 68, 50, 255]][(s.clamp(1, 6) - 1) as usize] }
/// The game's 6-bit hair RGB as a drawing colour: lifted (the game's values are dark) and a little more saturated.
pub fn haar_rgb(r: u8, g: u8, b: u8) -> [u8; 4] {
    let k = |c: u8| 255.0 * (c.min(63) as f32 / 63.0).powf(0.55);
    let (r, g, b) = (k(r), k(g), k(b));
    let grau = (r + g + b) / 3.0;
    let satt = |c: f32| (grau + (c - grau) * 1.6).clamp(0.0, 255.0) as u8;
    [satt(r), satt(g), satt(b), 255]
}

/// What the photo shows: skin, hair, hair style (0 short, 1 quiff, 2 afro, 3 side parting, 4 bald), beard.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Merkmale { pub haut: [u8; 4], pub haar: [u8; 4], pub frisur: u8, pub bart: bool }
/// The photo's features from the appearance record, else a fixed dice of the id (same player, same face).
pub fn merkmale(id: u32, look: Option<Aussehen>) -> Merkmale {
    let z = |k: u32| (id.wrapping_mul(2654435761).wrapping_add(k.wrapping_mul(40503)) >> 8) % 1000;
    let zufall_frisur = [0, 1, 3][(z(3) % 3) as usize];
    match look {
        Some(l) => Merkmale {
            haut: haut_stufe(l.haut),
            haar: haar_rgb(l.haar[0], l.haar[1], l.haar[2]),
            frisur: match l.stil { 0 => 4, 4 => 2, _ => zufall_frisur },
            bart: l.bart,
        },
        None => {
            // roughly the spread of FL27's records (skin 1–3 most often; hair black/dark brown most often)
            let haut = [1, 1, 2, 2, 3, 3, 4, 5, 6][(z(1) % 9) as usize];
            let h = [[5, 4, 3], [8, 5, 3], [8, 5, 3], [11, 7, 4], [14, 10, 7], [18, 14, 10], [33, 28, 21]][(z(2) % 7) as usize];
            Merkmale { haut: haut_stufe(haut), haar: haar_rgb(h[0], h[1], h[2]), frisur: zufall_frisur, bart: z(4) % 3 == 0 }
        }
    }
}

// ---------------------------------------------------------------- building the meshes (prototype 1:1)
struct Bau { v: Vec<Vertex>, i: Vec<u16>, fertig: Vec<Mesh> }
impl Bau {
    fn neu() -> Self { Bau { v: vec![], i: vec![], fertig: vec![] } }
    /// macroquad draws a mesh in one call of at most 10,000 vertices and 5,000 indices: a new mesh before that
    fn platz(&mut self, n: usize, ni: usize) { if self.v.len() + n > 9000 || self.i.len() + ni > 4500 { self.abschliessen(); } }
    fn abschliessen(&mut self) { if self.v.is_empty() { return; } self.fertig.push(Mesh { vertices: std::mem::take(&mut self.v), indices: std::mem::take(&mut self.i), texture: None }); }
    fn fertig(mut self) -> Vec<Mesh> { self.abschliessen(); self.fertig }
    fn teil(&mut self, m: Mat4, punkte: &[(Vec3, Vec3)], dreiecke: &[u16], farbe: [u8; 4]) {
        self.platz(punkte.len(), dreiecke.len());
        let nm = Mat3::from_mat4(m).inverse().transpose();
        let basis = self.v.len() as u16;
        for (p, n) in punkte {
            let wp = m.transform_point3(*p);
            let wn = (nm * *n).normalize_or_zero();
            self.v.push(Vertex { position: wp, uv: vec2(0., 0.), color: farbe, normal: vec4(wn.x, wn.y, wn.z, 0.) });
        }
        self.i.extend(dreiecke.iter().map(|d| basis + d));
    }
    fn kugel(&mut self, m: Mat4, farbe: [u8; 4]) {
        let (ringe, segs) = (12u16, 18u16);
        let mut p = vec![];
        for r in 0..=ringe {
            let th = std::f32::consts::PI * r as f32 / ringe as f32;
            for s in 0..=segs { let ph = std::f32::consts::TAU * s as f32 / segs as f32; let n = vec3(th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()); p.push((n, n)); }
        }
        let mut d = vec![];
        for r in 0..ringe { for s in 0..segs { let a = r * (segs + 1) + s; let b = a + segs + 1; d.extend_from_slice(&[a, a + 1, b, b, a + 1, b + 1]); } }
        self.teil(m, &p, &d, farbe);
    }
    fn kasten(&mut self, m: Mat4, farbe: [u8; 4]) {
        let mut p = vec![];
        let mut d = vec![];
        for n in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z] {
            let u = if n.y.abs() > 0.5 { Vec3::X } else { Vec3::Y };
            let w = n.cross(u);
            let b = p.len() as u16;
            for (a, c) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)] { p.push((n * 0.5 + u * 0.5 * a + w * 0.5 * c, n)); }
            d.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
        }
        self.teil(m, &p, &d, farbe);
    }
    fn zylinder(&mut self, m: Mat4, oben: f32, farbe: [u8; 4]) {
        let segs = 18u16;
        let mut p = vec![];
        let mut d = vec![];
        for s in 0..=segs {
            let a = std::f32::consts::TAU * s as f32 / segs as f32;
            let (c, sn) = (a.cos(), a.sin());
            let n = vec3(c, 1.0 - oben, sn).normalize();
            p.push((vec3(c, 0., sn), n));
            p.push((vec3(c * oben, 1., sn * oben), n));
        }
        for s in 0..segs { let a = s * 2; d.extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]); }
        let mitte = p.len() as u16;
        p.push((Vec3::Y, Vec3::Y));
        for s in 0..=segs { let a = std::f32::consts::TAU * s as f32 / segs as f32; p.push((vec3(a.cos() * oben, 1., a.sin() * oben), Vec3::Y)); }
        for s in 0..segs { d.extend_from_slice(&[mitte, mitte + 2 + s, mitte + 1 + s]); }
        self.teil(m, &p, &d, farbe);
    }
}
fn at(x: f32, y: f32, z: f32) -> Mat4 { Mat4::from_translation(vec3(x, y, z)) }
fn sk(x: f32, y: f32, z: f32) -> Mat4 { Mat4::from_scale(vec3(x, y, z)) }
fn rgb(r: u8, g: u8, b: u8) -> [u8; 4] { [r, g, b, 255] }

/// Upper body and head, standing, looking towards +z.
fn oberkoerper(b: &mut Bau, basis: Mat4, m: Merkmale, trikot: [u8; 4], kragen: [u8; 4]) {
    let (haut, haar, frisur, bart) = (m.haut, m.haar, m.frisur, m.bart);
    let dunkel = |f: [u8; 4], k: f32| [(f[0] as f32 * k) as u8, (f[1] as f32 * k) as u8, (f[2] as f32 * k) as u8, 255];
    b.zylinder(basis * at(0., 0.98, 0.) * sk(0.2, 0.6, 0.13), 1.35, trikot);
    b.kugel(basis * at(0., 1.58, 0.) * sk(0.28, 0.08, 0.17), trikot);
    b.kasten(basis * at(0., 1.62, 0.04) * Mat4::from_rotation_x(0.2) * sk(0.12, 0.03, 0.1), kragen);
    b.zylinder(basis * at(0., 1.6, 0.) * sk(0.065, 0.16, 0.065), 1.0, dunkel(haut, 0.92));
    for (seite, sgn) in [(-0.3f32, -1.0f32), (0.3, 1.0)] {
        let schulter = basis * at(seite, 1.55, 0.) * Mat4::from_rotation_z(sgn * 0.12);
        b.zylinder(schulter * at(0., -0.24, 0.) * sk(0.075, 0.26, 0.075), 1.2, trikot);
        let ell = schulter * at(0., -0.24, 0.) * Mat4::from_rotation_x(-0.5);
        b.zylinder(ell * at(0., -0.22, 0.) * sk(0.05, 0.22, 0.05), 1.2, haut);
    }
    let kopf = basis * at(0., 1.96, 0.);
    b.kugel(kopf * sk(0.21, 0.26, 0.23), haut);
    b.kugel(kopf * at(0., -0.13, 0.04) * sk(0.16, 0.12, 0.17), haut);
    for x in [-0.21f32, 0.21] { b.kugel(kopf * at(x, -0.01, -0.01) * sk(0.035, 0.06, 0.04), dunkel(haut, 0.93)); }
    let hinten = |b: &mut Bau| b.kugel(kopf * at(0., 0.02, -0.06) * sk(0.215, 0.22, 0.2), haar);
    match frisur {
        0 => { b.kugel(kopf * at(0., 0.16, 0.) * sk(0.225, 0.16, 0.235), haar); hinten(b); }
        1 => { b.kugel(kopf * at(0., 0.16, 0.) * sk(0.225, 0.16, 0.235), haar); hinten(b); b.kugel(kopf * at(0., 0.25, 0.08) * sk(0.15, 0.09, 0.15), haar); }
        2 => { b.kugel(kopf * at(0., 0.14, -0.02) * sk(0.255, 0.22, 0.255), haar); hinten(b); }
        3 => { b.kugel(kopf * at(0.02, 0.17, 0.01) * sk(0.23, 0.165, 0.24), haar); hinten(b); b.kugel(kopf * at(-0.05, 0.215, 0.07) * sk(0.16, 0.06, 0.16), haar); }
        _ => {}
    }
    if bart {
        let bf = dunkel(haar, 0.85);
        b.kugel(kopf * at(0., -0.125, 0.035) * sk(0.185, 0.14, 0.185), bf);
        b.kasten(kopf * at(0., -0.098, 0.222) * Mat4::from_rotation_x(-0.3) * sk(0.11, 0.018, 0.02), bf);
    }
    for x in [-0.075f32, 0.075] {
        b.kugel(kopf * at(x, 0.0, 0.205) * sk(0.024, 0.02, 0.012), rgb(36, 30, 28));
        b.kasten(kopf * at(x, 0.05, 0.205) * Mat4::from_rotation_x(-0.3) * Mat4::from_rotation_z(-x.signum() * 0.12) * sk(0.07, 0.016, 0.02), dunkel(haar, 0.9));
    }
    b.kugel(kopf * at(0., -0.05, 0.225) * sk(0.03, 0.045, 0.03), dunkel(haut, 0.93));
    b.kasten(kopf * at(0., -0.14, if bart { 0.226 } else { 0.212 }) * Mat4::from_rotation_x(-0.4) * sk(0.085, 0.014, 0.02), if bart { rgb(150, 90, 84) } else { dunkel(haut, 0.62) });
}

const VS: &str = r#"#version 100
attribute vec3 position; attribute vec2 texcoord; attribute vec4 color0; attribute vec4 normal;
varying lowp vec4 farbe; varying mediump vec3 n;
uniform mat4 Model; uniform mat4 Projection; uniform float Rand;
void main() { farbe = color0 / 255.0; n = normal.xyz; gl_Position = Projection * Model * vec4(position + normal.xyz * Rand, 1.0); }"#;
const FS_TOON: &str = r#"#version 100
precision mediump float;
varying lowp vec4 farbe; varying mediump vec3 n;
uniform vec3 Sonne; uniform vec3 Licht; uniform vec3 Schatten;
void main() { float d = dot(normalize(n), normalize(Sonne)); float stufe = 0.62 + 0.38 * smoothstep(-0.2, 0.7, d);
  vec3 c = farbe.rgb * mix(Schatten, Licht, stufe); c += farbe.rgb * 0.08 * max(normalize(n).y, 0.0); gl_FragColor = vec4(c, farbe.a); }"#;
const FS_RAND: &str = r#"#version 100
precision mediump float;
varying lowp vec4 farbe; varying mediump vec3 n;
void main() { gl_FragColor = vec4(0.16, 0.15, 0.15, 1.0); }"#;

struct Kamera { rt: RenderTarget }
impl Camera for Kamera {
    fn matrix(&self) -> Mat4 {
        let proj = Mat4::perspective_rh_gl(0.4, BREITE as f32 / HOEHE as f32, 0.5, 50.0);
        proj * Mat4::look_at_rh(vec3(0.7, 3.6, 4.9), vec3(0., 3.25, 0.), Vec3::Y)
    }
    fn depth_enabled(&self) -> bool { true }
    fn render_pass(&self) -> Option<macroquad::texture::RenderPass> { Some(self.rt.render_pass.clone()) }
    fn viewport(&self) -> Option<(i32, i32, i32, i32)> { Some((0, 0, BREITE as i32, HOEHE as i32)) }
}

struct Lager { toon: Material, rand: Material, bilder: HashMap<(u32, u32), RenderTarget>, reihe: Vec<(u32, u32)>, offen: Vec<((u32, u32), [u8; 4], [u8; 4])> }
thread_local! { static LAGER: RefCell<Option<Lager>> = const { RefCell::new(None) }; }

fn material(fs: &str, cull: CullFace) -> Material {
    let pipeline_params = PipelineParams {
        depth_test: Comparison::LessOrEqual,
        depth_write: true,
        cull_face: cull,
        color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
        ..Default::default()
    };
    let uniforms = vec![UniformDesc::new(crate::id("Sonne"), UniformType::Float3), UniformDesc::new(crate::id("Licht"), UniformType::Float3), UniformDesc::new(crate::id("Schatten"), UniformType::Float3), UniformDesc::new(crate::id("Rand"), UniformType::Float1)];
    load_material(ShaderSource::Glsl { vertex: VS, fragment: fs }, MaterialParams { pipeline_params, uniforms, ..Default::default() }).expect("pass photo shader")
}

/// Renders one photo (transparent background) into a new render target (kept alive with the photo: its pass must
/// outlive the frame that draws into it).
fn rendern(l: &Lager, m: Merkmale, trikot: [u8; 4], kragen: [u8; 4]) -> RenderTarget {
    let rt = render_target_ex(BREITE, HOEHE, RenderTargetParams { sample_count: 1, depth: true });
    rt.texture.set_filter(FilterMode::Linear);
    let mut b = Bau::neu();
    oberkoerper(&mut b, Mat4::from_scale(Vec3::splat(1.75)), m, trikot, kragen);
    let meshes = b.fertig();
    push_camera_state();
    set_camera(&Kamera { rt: rt.clone() });
    clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
    for (mat, rand) in [(&l.toon, 0.0f32), (&l.rand, 0.008)] {
        mat.set_uniform(crate::id("Sonne"), vec3(-0.5, 0.7, 0.8));
        mat.set_uniform(crate::id("Licht"), vec3(1.05, 1.02, 0.98));
        mat.set_uniform(crate::id("Schatten"), vec3(0.6, 0.62, 0.72));
        mat.set_uniform(crate::id("Rand"), rand);
        gl_use_material(mat);
        for me in &meshes { draw_mesh(me); }
    }
    gl_use_default_material();
    pop_camera_state();
    rt
}

/// Renders the photos asked for in the last frame. Called at the start of a frame, before anything is drawn:
/// clearing a render target drops the frame's queued draw calls, so it never happens in the middle of a page.
pub fn vorbereiten() {
    LAGER.with(|z| {
        let mut z = z.borrow_mut();
        let Some(l) = z.as_mut() else { return };
        for (schluessel, t, k) in std::mem::take(&mut l.offen) {
            if l.bilder.contains_key(&schluessel) { continue; }
            let tex = rendern(l, merkmale(schluessel.0, crate::gesichter::aussehen_von(schluessel.0)), t, k);
            if l.reihe.len() >= KEEP { let alt = l.reihe.remove(0); l.bilder.remove(&alt); }
            l.reihe.push(schluessel);
            l.bilder.insert(schluessel, tex);
        }
    });
}

fn lager_neu() -> Lager { Lager { toon: material(FS_TOON, CullFace::Nothing), rand: material(FS_RAND, CullFace::Front), bilder: HashMap::new(), reihe: vec![], offen: vec![] } }
fn schluessel_von(id: u32, trikot: Color, kragen: Color) -> ((u32, u32), [u8; 4], [u8; 4]) {
    let farbe = |c: Color| -> [u8; 4] { [(c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8, 255] };
    let (t, k) = (farbe(trikot), farbe(kragen));
    ((id, u32::from_le_bytes(t) ^ u32::from_le_bytes(k).rotate_left(7)), t, k)
}

/// The pass photo as a picture on a plain studio grey (`grund`), top row first: the template of the AI photo
/// (kifoto.rs). None while it is not rendered yet: it is asked for and rendered at the start of the next frame.
pub fn vorlage(id: u32, trikot: Color, kragen: Color, grund: [u8; 3]) -> Option<image::RgbaImage> {
    let (schluessel, t, k) = schluessel_von(id, trikot, kragen);
    LAGER.with(|z| {
        let mut z = z.borrow_mut();
        let l = z.get_or_insert_with(lager_neu);
        let Some(rt) = l.bilder.get(&schluessel) else {
            if !l.offen.iter().any(|o| o.0 == schluessel) { l.offen.push((schluessel, t, k)); }
            return None;
        };
        let d = rt.texture.get_texture_data();
        // the render target is stored bottom row first (zeichnen draws it with flip_y)
        Some(image::RgbaImage::from_fn(BREITE, HOEHE, |x, y| {
            let p = d.get_pixel(x, HOEHE - 1 - y);
            let m = |c: f32, g: u8| (c * 255.0 * p.a + g as f32 * (1.0 - p.a)).round().clamp(0.0, 255.0) as u8;
            image::Rgba([m(p.r, grund[0]), m(p.g, grund[1]), m(p.b, grund[2]), 255])
        }))
    })
}

/// The pass photo into the box (x, y, w, h): studio grey gradient, the player in his kit (keeper: keeper's kit).
/// The first frame shows the backdrop only; the photo is rendered before the next one.
pub fn zeichnen(x: f32, y: f32, w: f32, h: f32, id: u32, trikot: Color, kragen: Color) {
    let (schluessel, t, k) = schluessel_von(id, trikot, kragen);
    let tex = LAGER.with(|z| {
        let mut z = z.borrow_mut();
        let l = z.get_or_insert_with(lager_neu);
        let tex = l.bilder.get(&schluessel).map(|r| r.texture.clone());
        if tex.is_none() && !l.offen.iter().any(|o| o.0 == schluessel) { l.offen.push((schluessel, t, k)); }
        tex
    });
    // the studio backdrop: a vertical grey gradient
    let n = 24;
    for i in 0..n {
        let f = i as f32 / (n - 1) as f32;
        let c = Color::new(0.80 + (0.62 - 0.80) * f, 0.81 + (0.63 - 0.81) * f, 0.83 + (0.67 - 0.83) * f, 1.0);
        draw_rectangle(x, y + h * i as f32 / n as f32, w, h / n as f32 + 0.5, c);
    }
    let Some(tex) = tex else { return };
    let s = (w / BREITE as f32).min(h / HOEHE as f32);
    let (bw, bh) = (BREITE as f32 * s, HOEHE as f32 * s);
    draw_texture_ex(&tex, x + (w - bw) / 2.0, y + h - bh, WHITE, DrawTextureParams { dest_size: Some(vec2(bw, bh)), flip_y: true, ..Default::default() });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn farben_und_frisur() {
        // skin darkens step by step; out-of-range steps stay in range
        let hell: Vec<u32> = (1..=6).map(|s| haut_stufe(s)[..3].iter().map(|&x| x as u32).sum()).collect();
        assert!(hell.windows(2).all(|w| w[0] > w[1]), "{hell:?}");
        assert_eq!(haut_stufe(0), haut_stufe(1));
        assert_eq!(haut_stufe(9), haut_stufe(6));
        // blond (step 7) is lighter than black (step 1) and keeps a warm tone
        let (blond, schwarz) = (haar_rgb(33, 28, 21), haar_rgb(5, 4, 3));
        assert!(blond[0] as u32 + blond[1] as u32 > 2 * (schwarz[0] as u32 + schwarz[1] as u32) && blond[0] > blond[2]);
        // bald → no hair, afro → the round one, a beard stays; other styles: fixed by the id
        let mut l = Aussehen { haut: 6, haar: [5, 4, 3], haar_stufe: 1, stil: 0, bart: true };
        let m = merkmale(1, Some(l));
        assert_eq!((m.frisur, m.bart, m.haut), (4, true, haut_stufe(6)));
        l.stil = 4;
        assert_eq!(merkmale(1, Some(l)).frisur, 2);
        l.stil = 1;
        assert!([0, 1, 3].contains(&merkmale(1, Some(l)).frisur));
        assert_eq!(merkmale(77, Some(l)), merkmale(77, Some(l)));
        // without a record: the same face every time
        assert_eq!(merkmale(4242, None), merkmale(4242, None));
    }
}
