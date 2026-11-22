# Formation und Taktik ins Spiel schreiben – Stand der Forschung (22.11.2026)

Quellen: alte Python-Fassung (`pesm-ml-raus/pesm/fl/taktikdb.py`, `docs/intern/MATCHSTART.md`), Gaffer (`gaffer-linux/GEGNERTAKTIK.md`,
`app/lib/mlwrite.py`), Touchline (`pesm-sicherung/touchline-native-2026-10-09/touchline_native/tactics.py`, `gameplan.py`), eigene Messung
an `_FL26MyWorld` und dem ML-Spielstand `save__FL26MyWorld/ML00000000` (nur gelesen, entschlüsselt in den Notizordner).

## Was belegt ist
1. **Startelf und Bank** stehen im Kaderplatz der `PlayerAssignment.bin` (Bits 10..15 von `k`): 0..10 Startelf in der Folge der
   Formationsplätze, 11..22 Bank. PES M schreibt das schon (live in die Karriere-pesdb).
2. **Formation** steht in `TacticsFormation.bin`: 12-Byte-Sätze `u32 Kennung | u32 Positionscode (0 TW … 12 MS) | u8 Tiefe | u8 Quer |
   u16 Platz (Satz·16 + 0..10)`, 3 Sätze × 11 Plätze je Kennung. Die Kennungen eines Vereins stehen in `Tactics.bin`. Die Python-Fassung
   hatte Leser und Schreiber (`taktikdb.formation_setzen`, mit Sicherung und Wiederlesen), im Spiel nie bestätigt, deshalb nur hinter
   einem Schalter.
3. **Gaffer und Touchline** schreiben Taktik (Regler, Schalter, Anweisungen, Formation, Aufstellung) **nur in den Master-League-Spielstand**
   (Game-Plan-Zeile, 600 Byte je Trainer) bzw. Touchline zusätzlich live in den Spielspeicher. Gaffer: Regler und Schalter auch für
   KI-Vereine (`set_club_tactics`), Formation und Aufstellung nur für den eigenen Verein; im Spiel bestätigt nur Regler und die eigene
   Formation. Für PES M (Anstoß-Spiel über den Direktstart, Welt-Datenbank) ist dieser Weg nicht nutzbar: Im Anstoß gilt die pesdb.

## Was neu gemessen wurde (Tactics.bin, `_FL26MyWorld`)
- 2 Sätze je Verein, `u32 Kennung | u32 Team-ID | u32 gepackt`; **Bit 24 unterscheidet die beiden Sätze** (0/1).
- In den unteren drei Halbbytes stehen kleine Zahlen (Satz 0: 3–8, 3–7, 1–8), im zweiten Satz jeweils um +1, +1, −2 verschoben.
  Das sieht nach drei Reglern aus (wahrscheinlich Unterstützung, Abwehrlinie, Kompaktheit), passt aber in der Verteilung nicht
  1:1 zu den Reglern der Game-Plan-Zeilen im ML-Spielstand. Bits 12–15 und 25–30 wirken wie Schalter, Bits 19 und 21 sind immer gesetzt.
- Nicht entschlüsselt: die genaue Bedeutung je Bit. Ohne Bestätigung im Spiel schreibt PES M hier nichts.

## Nächster Schritt (Test im Spiel, ca. 15 Minuten)
1. In einer Karriere-pesdb (Kopie) für den Gegner nur das unterste Halbbyte von Satz 0 von z. B. 6 auf 1 setzen, Spiel starten,
   im Spiel „Spielplan des Gegners“ ansehen (Anstoß → Gegner → Taktik): Welcher Regler ist jetzt 1?
2. Dasselbe für Halbbyte 2 und 3 und für die Bits 28–30.
3. Formation: Satz 0 der Kennung des eigenen Vereins auf eine andere Formation (z. B. 4-4-2 → 3-5-2) schreiben und im Spiel ansehen.
Werkzeug dafür baue ich auf Wunsch (Kommandozeile `pesm --taktik-probe`).
