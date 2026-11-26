# Konzept: CalcioM-Modstore (Fabian 26.11.2026, Entwurf, noch nicht gebaut)

**Zweck (Fabian 26.11.):** CalcioM selbst enthält **keine lizenzierten Daten**, damit ein Release auf Steam oder Itch.io möglich wird. Echte Namen, Presse und ähnliche Daten kommen nur aus Mod-Quellen, die der Nutzer selbst einträgt.

Ein Ort, an dem Datenpakete und Mods liegen, die CalcioM lädt und verarbeitet. Nutzer können eigene Pakete bauen, einreichen und sich Pakete wünschen. Damit gehen drei ältere Ideen in einer Sache auf:
- die **Datenpakete**: echte Namen, Stadien und Pokalnamen kommen aus einem Paket statt aus dem Programm (Rechte);
- die **Mod-Library** (PAKETE 24);
- ein Teil der **Sandbox** (PAKETE 23): Ligen- und Regelpakete.

Alles mit „änderbar“ ist meine Entscheidung und kann anders werden.

## 1. Was ein Paket ist

Ein Paket ist ein Ordner oder eine ZIP-Datei mit `calciom-paket.toml` (Manifest) und **nur Daten**: TOML, JSON, PNG oder DDS, OGG.

```toml
[paket]
id = "echte-presse-de"          # eindeutig, klein, mit Bindestrichen
name = "Echte Presse Deutschland"
art = "presse"                  # siehe Abschnitt 3
version = "1.2.0"
autor = "Name oder Discord-Name"
lizenz = "CC BY 4.0"            # Pflicht
calciom = ">=0.6"               # ab welcher CalcioM-Version
sprachen = ["de", "en"]
beschreibung = "Reale Zeitungen und Reporter der Bundesliga-Städte"

[[datei]]
pfad = "presse.toml"
sha256 = "…"                    # jede Datei mit Prüfsumme
```

- Es gibt keine ausführbaren Dateien. Lua, EXE, DLL, Skripte und Sider-Module werden beim Laden abgelehnt (änderbar: Sider-Module höchstens später, nur verlinkt, nie gehostet).
- Jede Art hat ein Schema. CalcioM prüft das Paket beim Laden und meldet Fehler mit Datei und Zeile, statt abzustürzen.
- Pakete überschreiben nichts im Programm. Sie legen sich in einer festen Reihenfolge darüber: Programm → Paket A → Paket B. Die Reihenfolge ist in den Einstellungen verschiebbar.

## 2. Wo die Pakete liegen: Quellen

- **Quellen-Liste in den Einstellungen:** Der Nutzer trägt eine oder mehrere Mod-Quellen ein. Eine Quelle ist die Adresse einer `katalog.json`, etwa ein GitHub-Repo, eine eigene Webseite oder ein Ordner.
- **Standard:**
  - In der Steam/Itch-Fassung ist die Liste leer oder enthält nur die offizielle Quelle mit freien Paketen (Szenarien, Sprachen, Themes).
  - Quellen mit lizenzierten oder echten Daten trägt der Nutzer selbst ein. Sie werden nie mit dem Programm ausgeliefert.
- **Hinweis beim Eintragen einer fremden Quelle:** „Pakete aus dieser Quelle kommen nicht von CalcioM. Du bist selbst für ihre Nutzung verantwortlich. Es werden nur Daten geladen, keine Programme.“ Der Nutzer muss bestätigen.
- **Jede Quelle** zeigt ihren Namen und Betreiber (aus der `katalog.json`) und lässt sich an- und ausschalten oder entfernen.
- **Katalog:** `katalog.json` enthält je Paket id, Version, Größe, sha256, Art, Lizenz und den Download-Link. Unser offizielles Repo wäre `FabianZettl/calciom-mods`, die Pakete als GitHub-Release-Assets.
- **Laden:** CalcioM liest jeden Katalog wie heute die Update-Prüfung (`curl -L`, ohne Konto). Pakete landen in `<Datenordner>/pakete/<quelle>/<id>/<version>/`.
- **Offline:** Was einmal geladen ist, bleibt nutzbar. Ohne Netz zeigt der Store „offline“.
- **Von Hand:** Eine ZIP-Datei lässt sich auch per „Paket aus Datei hinzufügen“ einspielen.
- **Lizenz sichtbar:** Das Feld `lizenz` aus dem Manifest steht im Store und bei installierten Paketen. Ohne Lizenzangabe wird ein Paket nicht installiert.

## 3. Paket-Arten (erste Liste)

| Art | Inhalt | Bemerkung |
|---|---|---|
| presse | echte Zeitungen, Sender, Reporter je Land und Stadt | standardmäßig aus; nie erfundene Gerüchte unter echtem Namen (Abschnitt 6) |
| gesichter | vorgefertigte Spielergesichter, Passfotos, Minifaces nach Spieler-ID | nur selbst gemachte oder freie Bilder, keine aus Spielen gerippten |
| szenarien | Gesprächsthemen im Pool-Format (Themen-TOML + Texte) | dieselbe Themenprüfung wie im Programm |
| vereine | echte Vereinsnamen, Kurznamen, Stadionnamen, Pokal- und Wettbewerbsnamen | ersetzt die Datenpakete-Idee |
| regeln | Pokal- und Ligaregeln, Auf- und Abstieg, Prämien | Format wie `pokalregeln.toml` |
| sprache | Übersetzungen (`ui.<code>.json` und Kerntexte) | z. B. Italienisch von der Community |
| trainer | Trainerlisten mit Stil und Werdegang | |
| geruechte | Texte für Gerüchte, Schlagzeilen, Zeitungsberichte | |
| musik | Hintergrundmusik und Klänge (OGG) | nur mit freier Lizenz |
| thema | UI-Farben und Akzente | |

Weitere Ideen: Stadion- und Städtebilder (eigene Fotos), Kommentatoren-Sprüche im Live-Ticker, historische Saisons als Startpunkt, Trikotsponsoren, Taktik-Vorlagen berühmter Trainer (Spielidee und Regler), Erfolge und Meilensteine, Vorlagen für Suchprofile.

## 4. In CalcioM

- **Einstellungen › Pakete** hat zwei Reiter: **Installiert** (an/aus, Reihenfolge, Version, Update verfügbar, Entfernen) und **Store** (Katalog nach Art, Suche, Größe, Lizenz, „Installieren“).
- Jedes Paket lässt sich einzeln an- und ausschalten, auch im laufenden Spielstand. Der Spielstand merkt sich, mit welchen Paketen er gestartet wurde. Fehlt eins, gibt es einen Hinweis, aber kein Abbruch.
- Mit **„Wunsch äußern“** schickt man eine Nachricht an die Wunschliste (Abschnitt 5).

## 5. Einreichen und Wünsche

- **Einreichen:** ein Pull Request auf `calciom-mods` mit einem Ordner unter `pakete/<id>/`. Eine GitHub-Action prüft:
  - Manifest und Schema,
  - Prüfsummen,
  - Dateitypen (nur erlaubte),
  - Größe (Grenze, z. B. 50 MB),
  - Lizenz vorhanden,
  - bei `szenarien` dieselbe Themenprüfung wie im Programm.

  Danach schaut ein Mensch darüber (Fabian oder ein Moderator). Erst nach dem Merge baut die Action das Release-Asset und trägt das Paket in den Katalog ein.
- **Ohne GitHub-Konto:** ein Discord-Kanal `#mod-uploads`. Der Bot nimmt ZIPs an und legt den Pull Request an (änderbar).
- **Wünsche:** GitHub-Issues mit der Vorlage „Paketwunsch“ oder ein Discord-Kanal `#mod-wuensche`. Die Liste wird im Store unter „Gewünscht“ angezeigt, mit Stimmen (Reaktionen).
- Ein **Ersteller-Werkzeug** im Programm prüft ein Paket lokal mit derselben Prüfung wie die Action und erzeugt Manifest und Prüfsummen.

## 6. Recht und Sicherheit

- **Keine Spieldateien:** keine Dateien aus FL27, PES, eFootball oder FC (Datenbanken, CPKs, Gesichter aus dem Spiel, Trikots). Die Prüfung lehnt bekannte Dateiendungen und Header (WESYS, CPK) ab. Wer so etwas einreicht, wird nicht aufgenommen.
- **Echte Personen:** Die Pakete `presse` und `trainer` mit echten Namen sind in CalcioM standardmäßig aus und lassen sich mit einem Schalter einschalten. Erfundene Gerüchte, Zitate oder Skandale nie unter dem Namen einer echten Person: Mit echtem Namen dürfen nur neutrale Texte laufen (Fragen, Spielberichte); Gerüchte bekommen dann eine erfundene Quelle.
- **Bilder:** nur mit Lizenz, die eine Weitergabe erlaubt. Bilder realer Personen nur mit Erlaubnis oder freier Lizenz.
- **Technisch:**
  - Nur Daten.
  - Prüfsumme pro Datei, Pfade ohne `..` (kein Ausbruch aus dem Paketordner).
  - Größengrenzen, Zeitlimit beim Laden.
  - Ein kaputtes Paket schaltet sich selbst ab und meldet sich im Log.
- Das Mod-Repo bekommt eine eigene LICENSE und eine kurze Richtlinie (CONTRIBUTING).

## 7. Schritte

1. Paketformat und Laden lokal, ohne Store: Ordner `pakete/`, Manifest, Prüfung, an/aus. Erste Art: `szenarien`, das Format gibt es schon.
2. Arten `vereine` und `presse` (die alten Datenpakete) und `sprache`.
3. Katalog-Repo, Store-Reiter, Download mit Prüfsumme, Updates.
4. Einreichung: Action, Vorlagen, Discord-Weg, Wunschliste.
5. Weitere Arten (gesichter, musik, thema, trainer …).

## 8. Was für ein Store-Release ausgelagert werden müsste (Stand 26.11., noch nichts umgebaut)

Heute im Programm enthalten, für eine Steam- oder Itch-Fassung in Pakete zu verschieben (Art in Klammern):

- **Echte Spieler, Vereine und Ligen der eingebauten Probewelt:** `assets/world.json` (vereine). Die öffentliche Kopie anonymisiert die Spielernamen schon (`anon.py`); Vereinsnamen sind echt.
- **Echte Journalisten und Medien:** `assets/inhalt/pesm/core/config/story/medienpool.json` (presse). Die öffentliche Kopie ersetzt sie schon durch erfundene. Dazu kommen Reporternamen im Code (`src/medien.rs`, `src/netzbilder.rs`), die das Export-Skript tauscht.
- **Echte Vereinsführung, Stadien und Trainingsgelände:** `assets/vereinsfuehrung.toml`, aus Wikipedia (vereine).
- **Echte Trainer:** `assets/regeln/nationaltrainer.toml` (Nationaltrainer) und `assets/inhalt/pesm/core/config/scouting/trainerbank.json` (Trainerbank) (trainer).
- **Echte Wettbewerbs- und Pokalnamen:** `assets/regeln/pokalnamen.toml`, `assets/wettbewerbsnamen.json`, Sponsorennamen in Wettbewerben (vereine/regeln).
- **Partner- und Zweitvereine:** `assets/kooperationen.json` und `assets/zweite_mannschaften.json`, mit echten Vereinsnamen (vereine).
- **Vereinsorte und Koordinaten:** `assets/geo/vereine.json` und `orte.json`, aus Wikidata CC0. Rechtlich unkritisch, Vereinsnamen sind aber echt (vereine).
- **Wappen:** `assets/wappen/` (in der öffentlichen Kopie schon entfernt). Wappen kommen sonst zur Laufzeit aus dem Spiel des Nutzers (vereine).
- **SoFIFA-Werte (echte Verträge, Gehälter, Potenziale):** werden aus einem Ordner des Nutzers gelesen (`src/sofifa.rs`) und nicht mitgeliefert. Das kann so bleiben, besser aber als Paketart „werte“ beschreiben.
- **Bilder aus Wikimedia/Wikidata** (`src/netzbilder.rs`): werden zur Laufzeit geladen, freie Lizenzen. Für Steam sollte das trotzdem abschaltbar sein und standardmäßig aus.
- **Namen in Texten:** zum Beispiel echte Vereine in Gesprächs- und Story-Texten. Das müsste ein Suchlauf über `assets/inhalt` und `assets/story` klären (offen).
- **FL27-Bezug:** Daten aus dem Spiel liest CalcioM nur zur Laufzeit von der Platte des Nutzers. Ausgeliefert wird nichts davon. Für einen Store-Release ist aber zu klären, wie CalcioM ohne FL27 dasteht (Probewelt, erfundene Namen).
