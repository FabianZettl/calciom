//! CalcioM's marks (Fabian's brand sheet and pictures 26.11.2026): the word mark, the start picture and the monogram
//! as the window icon. The pictures are in assets/marke/.
use crate::ui::hex;
use macroquad::prelude::*;

thread_local! {
    /// Fabian's pictures (26.11.2026): logo, word mark light and dark; loaded once, smoothed when scaled.
    static BILDER: [Texture2D; 3] = [
        include_bytes!("../assets/marke/logo.png").as_slice(),
        include_bytes!("../assets/marke/wortmarke_hell.png").as_slice(),
        include_bytes!("../assets/marke/wortmarke_dunkel.png").as_slice(),
    ].map(|b| { let t = Texture2D::from_file_with_format(b, None); t.set_filter(FilterMode::Linear); t });
}
fn bild(i: usize) -> Texture2D { BILDER.with(|b| b[i].clone()) }

/// The word mark (CALCIOM, gold M, CALCIO MANAGEMENT below) with its top left corner at (x, y), `breite` wide;
/// dark letters on the light theme, light ones on the dark theme. Returns its height.
pub fn wortmarke(x: f32, y: f32, breite: f32, dunkel: bool, deckung: f32) -> f32 {
    let t = bild(if dunkel { 2 } else { 1 });
    let h = breite * t.height() / t.width();
    draw_texture_ex(&t, x, y, Color::new(1.0, 1.0, 1.0, deckung), DrawTextureParams { dest_size: Some(vec2(breite, h)), ..Default::default() });
    h
}

/// The start picture (Fabian 26.11.): logo and word mark on the dark ground, fading in and out over `DAUER` seconds.
pub const DAUER: f32 = 2.4;
pub fn startbild(t: f32) {
    clear_background(hex(0x0d1310));
    let (b, h) = (crate::ui::breite(), crate::ui::hoehe());
    let x0 = -crate::ui::versatz();
    draw_rectangle(x0, 0.0, b - 2.0 * x0, h, hex(0x0d1310));
    let glatt = |v: f32| { let v = v.clamp(0.0, 1.0); v * v * (3.0 - 2.0 * v) };
    let deckung = glatt(t / 0.5) * (1.0 - glatt((t - (DAUER - 0.45)) / 0.45));
    let (logo, wort) = (220.0 * (0.96 + 0.04 * glatt(t / 0.8)), 520.0);
    let wh = wort * 442.0 / 1475.0;
    let oben = (h - (220.0 + 44.0 + wh)) / 2.0;
    let l = bild(0);
    draw_texture_ex(&l, b / 2.0 - logo / 2.0, oben + (220.0 - logo) / 2.0, Color::new(1.0, 1.0, 1.0, deckung), DrawTextureParams { dest_size: Some(vec2(logo, logo)), ..Default::default() });
    wortmarke(b / 2.0 - wort / 2.0, oben + 220.0 + 44.0, wort, true, deckung * glatt((t - 0.25) / 0.5));
}

/// Fabian's app icon (26.11.2026) as the window icon, 16, 32 and 64 pixels (made from assets/marke/logo.png).
pub fn icon() -> macroquad::miniquad::conf::Icon {
    fn rgba<const N: usize>(png: &[u8]) -> [u8; N] {
        let mut a = [0u8; N];
        let b = image::load_from_memory(png).map(|i| i.to_rgba8().into_raw()).unwrap_or_default();
        if b.len() == N { a.copy_from_slice(&b); }
        a
    }
    macroquad::miniquad::conf::Icon {
        small: rgba(include_bytes!("../assets/marke/icon16.png")),
        medium: rgba(include_bytes!("../assets/marke/icon32.png")),
        big: rgba(include_bytes!("../assets/marke/icon64.png")),
    }
}

#[cfg(test)]
mod t {
    #[test]
    fn icon_hat_alle_groessen_und_farben_lesen_sich() {
        let i = super::icon();
        // the corner pixel of the 64 icon is transparent (rounded corner), the middle row carries the deep green
        assert!(i.big[3] < 40 && i.big.chunks(4).any(|p| p[..3] == [0x0e, 0x3b, 0x2a]));
        assert!(i.small.iter().any(|&b| b != 0) && i.medium.iter().any(|&b| b != 0));
        // text colours of both themes read on the backgrounds (at least 4.5:1, brand sheet)
        use crate::ui::{kontrast, Theme};
        for dunkel in [false, true] {
            Theme::set(dunkel, 0xd4152f, 0x38bdf8);
            for t in [Theme::text(), Theme::text2(), Theme::gedaempft()] {
                for g in [Theme::grund(), Theme::flaeche()] { assert!(kontrast(t, g) >= 4.5, "dunkel={dunkel} {t:?} auf {g:?}: {}", kontrast(t, g)); }
            }
            assert!(kontrast(Theme::akzent(), Theme::grund()) >= 3.0);
        }
    }
}
