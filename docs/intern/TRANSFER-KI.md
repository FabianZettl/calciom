# Transfer-KI der ganzen Welt (Kurzfassung, Fabian 24.11.)

Ziel: plausible Wechsel überall, ohne 150.000 Spieler × 5.000 Vereine täglich zu rechnen. Drei Schichten.

## 1. Korridore (wohin ein Spieler überhaupt wechseln kann)
- Jede Liga hat eine **Stärke** (Mittel der Vereinsstärken, `statistik::liga_staerke`) und eine **Stufe A–D** (`korridore.toml [stufen]`).
- **Korridorgewicht** zwischen zwei Ligen, 0…1 (`transferki::korridor`), Produkt aus:
  - **Stufenabstand:** gleich 1,0; eine höher 0,8; eine tiefer 0,6; zwei tiefer 0,15; drei und mehr tiefer 0.
  - **Nähe:** gleiches Land 1,0; gleicher Kontinent 0,7; anderer 0,35.
  - **Sprache** gleich: ×1,2.
  - **Historische Ströme** (Länderpaare in der Datei): ×1,4.
  - **Geldligen:** für Spieler ab 28 Jahren ×1,5.
- **Ausnahmen mit Grund:** Heimkehr ins Herkunftsland; über 34 und vereinslos.

## 2. Marktradius (wer kommt für wen in Frage)
- Weltklasse (ab 85): global, aber nur Stufe A/B.
- Gut (ab 78): eigener Kontinent plus Geldligen.
- Durchschnitt: eigenes Land, Nachbarn (gleicher Kontinent ×0,5), Herkunftsland.
- Schwach oder alt (ab 33): eigenes Land und darunter.

## 3. Bedarf, Profil, Suche
- **Schon da** (`kimarkt.rs`):
  - Bedarf je Positionsgruppe.
  - Philosophie und Budget mit FFP.
  - Sichtfeld des Scoutings.
  - Index je Positionsgruppe nach Stärke sortiert, Fit-Score `passung_mit`.
  - Gründe als Codes (`bedarf_grund`).
  - Anbahnung mit Gerüchten, Deadline Day.
- **Neu:** Der Fit-Score wird mit Korridorgewicht × Radius multipliziert. Ein Gewicht unter 0,05 schließt den Spieler aus, außer bei einer Ausnahme.

## 4. Detailstufen
- Detaillierte Ligen (die eigene, Stufe A und B): volle Bedarfssuche wie bisher.
- Zusammengefasste Ligen (C/D): **Strom-Ziehung** (`transferki::strom_tag`).
  - Je Liga und Fenster N Wechsel (Teams × `strom.je_team`, im Winter ein Drittel), verteilt über die Fenstertage.
  - Käufer zufällig, Spieler aus einer Korridorliga im passenden Stärkeband.
  - Ablöse nach Marktwert, Grund `strom`.

## 5. Prüfung
Langzeittest `transferki::tests::zehn_jahre` (ignoriert, `cargo test --release zehn_jahre -- --ignored --nocapture`).

Er misst:
- den Anteil der Wechsel im eigenen Land (Ziel 60–70 %),
- Wechsel mehr als zwei Stufen nach unten (nur mit Ausnahme),
- Altersverteilung und mittlere Ablöse je Stufe.

Alle Zahlen stehen in `assets/regeln/korridore.toml` und sind änderbar.

Offen: Transferfenster je Land, Südhalbkugel; Arbeitserlaubnis; Angebote an uns aus derselben Suche.
