# Unused code (found 24.11.2026, not deleted)

The compiler reports these as never used. They were left in on purpose: several are ported from the Python
original and may be meant for a later step. Before deleting one, check PAKETE.md / ABDECKUNG.md for a plan
that needs it. Fields that are only read from TOML/JSON (serde) are not listed; they keep the file format whole.

- `src/date.rs:6,28` `wd_long_t`, `wd_long`
- `src/farm.rs:114` `aufsteiger`
- `src/inhalt.rs:22,26,79,83` `eintraege`, `quellen`, `text`, `waehlen`
- `src/ki.rs:66` `vision_ultimatum`
- `src/kimarkt.rs:12,171` `GRUPPEN`, `passung`
- `src/medien.rs:424` `geruecht`
- `src/mensch.rs:29,418` `STATUS`, `stufe`, `ansehen`
- `src/personal.rs:126` `trainingsstufe`
- `src/postkarten.rs:96,106,107` `bild`, `zahl`, `hat_bild`, `hat_zahl`
- `src/rest3.rs:235` `kontakt_knuepfen`
- `src/statistik.rs:48` `KEYS`
- `src/taktik.rs:32` `rollen_fuer`
- `src/trainerbank.rs:125` `aehnlich`
- `src/transfer.rs:311` `leihe_holen`, `treue_wort`
- `src/ui.rs:535` `banner`
- `src/vertragsanfrage.rs:32` `bedingung_text`
- `src/vorgang.rs:8,9,105` `stufen_t`, `pflicht_t`, `co_beziehung`
- `src/wirtschaft.rs:150` `ticket_price`, `capacity`, `attendance`
- `src/world.rs:512` `yearly_change`

Current list: `cargo build --release --offline -j 4 2>&1 | grep -A2 "never used"`.
