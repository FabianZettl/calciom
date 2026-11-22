# Kalibrierungsvergleich Entwicklung (eine Saison, 52 Wochen, je 12 Spieler)

Erzeugt vom Test `entwicklung::tests::saison_kalibrierungsvergleich`. Erwartung aus `tests/test_entwicklung3.py` der Python-Fassung; gemessen wird der Gesamtwert des Rust-Modells (`ovr_weights_fl27.json`).

| Fall | Erwartung (Median) | Rust (Median) | Spanne |
|---|---|---|---|
| 19 J., Sturm, Stärke 66, Stammspieler mit guten Noten | +3 … +6 | +4.1 | +3.1 … +4.4 |
| 25 J., zentrales Mittelfeld, 72, Stammspieler mit guten Noten | 0 … +2 | +1.3 | +0.5 … +1.7 |
| 33 J., Innenverteidiger, 74, Stammspieler | −3 … −1 | -2.6 | -4.0 … -2.4 |
