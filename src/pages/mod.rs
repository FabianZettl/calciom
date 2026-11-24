// The manager's pages, drawn into the content area right of the sidebar (layout after the original's screenshots).
use crate::date;
use crate::game::*;
use crate::ui::*;
use crate::world::*;
use macroquad::prelude::*;

// Paket 5: the pages that were still missing (included into pages.rs, so they use its building blocks).
// Each page shows data that the career already holds; texts in plain German.

mod akademie;
mod analyse;
mod aufstellung;
mod common;
mod finanzen;
mod fl27_steuerung;
mod kabine;
mod kader;
mod kaderplanung;
mod kalender;
mod live;
mod manager;
mod medizin;
mod nationalteams;
mod optionen;
mod overlays;
mod personal;
mod posteingang;
mod praesidium;
mod presse;
mod scouting;
mod spielerkarte;
mod spielerprofil;
mod spieltag;
mod statistik;
mod tabellen;
mod trainer;
mod training;
mod transfermarkt;
mod uebersicht;
mod verein;
mod verhandlungen;
mod vorbereitung;
mod welt;
mod zentrale;
use akademie::*;
use analyse::*;
use aufstellung::*;
pub use common::*;
use finanzen::*;
pub use fl27_steuerung::*;
use kabine::*;
use kader::*;
use kaderplanung::*;
use kalender::*;
pub use live::*;
use manager::*;
use medizin::*;
use nationalteams::*;
use optionen::*;
pub use overlays::*;
use personal::*;
use posteingang::*;
use praesidium::*;
pub use presse::*;
use scouting::*;
pub use spielerkarte::*;
pub use spielerprofil::*;
pub use spieltag::*;
pub use statistik::*;
use tabellen::*;
use trainer::*;
use training::*;
pub use transfermarkt::*;
use uebersicht::*;
use verein::*;
pub use verhandlungen::*;
use vorbereitung::*;
use welt::*;
use zentrale::*;


// ================================================================ dispatch
pub fn draw_page(ui: &mut Ui, v: &mut View, c: &mut Career) {
    match v.page {
        Page::Uebersicht => uebersicht(ui, v, c),
        Page::Posteingang => posteingang(ui, v, c),
        Page::Kalender => kalender(ui, v, c),
        Page::Kader => kader(ui, v, c),
        Page::Aufstellung => aufstellung(ui, v, c),
        Page::Kabine => kabine(ui, v, c),
        Page::Training => training(ui, v, c),
        Page::Medizin => medizin(ui, v, c),
        Page::Praesidium => praesidium(ui, v, c),
        Page::Finanzen => finanzen(ui, v, c),
        Page::Trainer => trainer(ui, v, c),
        Page::Transfermarkt => transfermarkt(ui, v, c),
        Page::Verhandlungen => verhandlungen(ui, v, c),
        Page::Vertraege => vertraege(ui, v, c),
        Page::Scouting => scouting(ui, v, c),
        Page::Akademie => akademie(ui, v, c),
        Page::Tabellen => tabellen(ui, v, c),
        Page::Presse => presse(ui, v, c),
        Page::Spieltag => spieltag(ui, v, c),
        Page::Optionen => optionen(ui, v, c),
        Page::Ziele => ziele(ui, v, c),
        Page::Manager => manager_seite(ui, v, c),
        Page::Stadion => stadion(ui, v, c),
        Page::Personalmarkt => personalmarkt(ui, v, c),
        Page::Geschichte => geschichte(ui, v, c),
        Page::Tagebuch => tagebuch(ui, v, c),
        Page::Klima => klima_seite(ui, v, c),
        Page::Marken => marken(ui, v, c),
        Page::Kaderplanung => kaderplanung(ui, v, c),
        Page::Vergleich => vergleich(ui, v, c),
        Page::Entwicklung => entwicklung(ui, v, c),
        Page::Vorbereitung => vorbereitung(ui, v, c),
        Page::Analyse => analyse_seite(ui, v, c),
        Page::Zeitung => zeitung(ui, v, c),
        Page::Fans => fans_seite(ui, v, c),
        Page::Ranglisten => ranglisten(ui, v, c),
        Page::Nationalteams => nationalteams_seite(ui, v, c),
        Page::Vereine => vereine(ui, v, c),
        Page::Vereinswelt => vereinswelt(ui, v, c),
        Page::Portal => portal(ui, v, c),
        Page::Handbuch => handbuch(ui, v, c),
        Page::Regeln => regeln_seite(ui, v, c),
        Page::Profi => profi(ui, v, c),
        Page::Spielerprofil => spielerprofil(ui, v, c),
        Page::Zentrale => zentrale(ui, v, c),
        Page::Kooperationen => kooperationen(ui, v, c),
        Page::ZweiteMannschaft => zweite_mannschaft_tab(ui, v, c),
        Page::Jugendnetz => jugendnetz(ui, v, c),
        Page::Merkliste => merkliste(ui, v, c),
        Page::Trainerteam => trainerteam(ui, v, c),
    }
}

include!("../seiten_farm.rs");
include!("../postkarten.rs");

#[cfg(test)]
mod scroll_t {
    /// A squad of 40 in the portrait grid (4 a row, 3 rows fit) and a bench of 12 (8 fit): scrolling shows every one.
    #[test]
    fn nach_dem_scrollen_sind_alle_zu_sehen() {
        for (total, vis) in [(10usize, 3usize), (12, 8), (40, 12), (5, 9), (0, 4)] {
            let mut gesehen = vec![false; total];
            for s in 0..=total { for i in super::fenster(total, vis, s) { gesehen[i] = true; } }
            assert!(gesehen.iter().all(|&g| g), "{total} Einträge, {vis} sichtbar");
            assert!(super::fenster(total, vis, 999).end == total, "ganz unten ist der letzte Eintrag");
        }
    }
}
