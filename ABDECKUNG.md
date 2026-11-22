# Abdeckung der Python-Fassung

Erzeugt von `tools/abdeckung.py` (neu erzeugen nach jedem größeren Bau). Eine Zeile je Datei aus `pesm-ml-raus/pesm/` ohne Tests, `__pycache__` und `ui/demo/`. Status: **gebaut** = eine Rust-Datei nennt die Python-Datei; **Bereich** = die Rust-Datei des Bereichs gibt es, die Datei selbst ist dort nicht genannt (prüfen); **entfällt** = Web-Backend, Oberfläche der Python-Fassung, Speicher-Beobachter (Rust zeichnet selbst, liest über die Coaching Zone); **offen** = keine Rust-Stelle.

| Datei | Aufgabe | Rust | Status |
|---|---|---|---|
| `__init__.py` | PESM: Manager-Oberbau für PES. Reines Python im Kern, Oberfläche folgt in späteren Arbeitspaketen. | — (Paketdatei) | entfällt |
| `diagnose.py` | Live-Diagnose der Spielverbindung: eine Logdatei, in die alle Stationen schreiben (Backend, Brücke, Einlesen, Weg A). | pages.rs (Profi-Details) | Bereich |
| `gesundheit.py` | Gesundheitswächter: erkennt auch stilles Scheitern und meldet es in die Problemliste (pesm/probleme.py, docs/intern/WAECHTER.md). | fl27.rs (Wächter) | Bereich |
| `pfade.py` | Zentrale Pfadverwaltung: alle Nutzerorte (Einstellungen, Daten, Zwischenspeicher) folgen den XDG-Regeln; Tests leiten HOME/XDG_* um (tests/_ | game.rs (Karrieren-Ordner) | Bereich |
| `probleme.py` | Problemliste von FL-World: alles, was nicht durchging oder nicht geschrieben werden konnte, mit Ursache, Zustand und Reparaturkontext. | pages.rs (Profi-Details) | Bereich |
| `protocol.py` | Nachrichtenformat v1 zwischen Oberfläche und Backend (Bauauftrag §11). Kein Qt. | entfällt (Backend) | entfällt |
| `stand.py` | Code-Stand des Backends: Prüfsumme über Name, Größe und Änderungszeit der Python-Dateien von Backend, Kern und | entfällt (Backend) | entfällt |
| `waechter.py` | Wächter für Hänger der Oberfläche: Steht der Qt-Thread länger als 5 s, schreibt ein eigener Faden einen Stapelauszug aller Threads in diagno | entfällt (Qt) | entfällt |
| `adapters/__init__.py` |  | — (Paketdatei) | entfällt |
| `adapters/assets/__init__.py` |  | — (Paketdatei) | entfällt |
| `adapters/assets/bilder.py` | Bildschicht getBild (DESIGN §6): `image://bild/<typ>/<id>/<groesse>`. | fl27.rs | gebaut |
| `adapters/bilder/__init__.py` | Bilder aus dem Netz (Wappen, Spielerbilder) über austauschbare Anbieter. | — (Paketdatei) | entfällt |
| `adapters/bilder/anbieter.py` | Schnittstelle der Bildanbieter. Ein Anbieter sucht nach einem Namen und liefert Kandidaten mit den Merkmalen, die | ui.rs | Bereich |
| `adapters/bilder/dds.py` | DDS-Bilder (DXT1, DXT3, DXT5) entpacken und als PNG schreiben – ohne fremde Bibliothek. | ui.rs | Bereich |
| `adapters/bilder/dienst.py` | Bilddienst: holt Wappen und Spielerbilder nur bei Bedarf (eine Seite fragt danach), nur mit Erlaubnis, höchstens | ui.rs | Bereich |
| `adapters/bilder/speicher.py` | Zwischenspeicher für Bilder: ~/.cache/pesm/bilder/ (bzw. XDG_CACHE_HOME), nie im Repo. Je Anfrage ein Eintrag | ui.rs | Bereich |
| `adapters/bilder/thesportsdb.py` | Bildanbieter TheSportsDB (https://www.thesportsdb.com). Suche nach Vereins- bzw. Spielernamen über die öffentliche | ui.rs | Bereich |
| `adapters/bilder/zuordnung.py` | Sichere Zuordnung von Suchergebnissen: lieber kein Bild als ein falsches. | ui.rs | Bereich |
| `adapters/fremd/__init__.py` |  | — (Paketdatei) | entfällt |
| `adapters/karriere/__init__.py` | Reale Vorgeschichte eines Spielers (Stationen, Jugendvereine, Geburtsort, Ausbildung) über austauschbare Anbieter. | — (Paketdatei) | entfällt |
| `adapters/karriere/anbieter.py` | Schnittstelle der Karriereanbieter. Drei Schritte, je in der Regel eine Anfrage ins Netz: Namen suchen (Kennungen), | game.rs, geo.rs | Bereich |
| `adapters/karriere/dienst.py` | Karrieredienst: holt die Vorgeschichte eines Spielers nur bei Bedarf (das Profil fragt danach), nur mit Erlaubnis, | game.rs, geo.rs | Bereich |
| `adapters/karriere/speicher.py` | Zwischenspeicher für Karrieredaten: ~/.cache/pesm/karriere/ (bzw. XDG_CACHE_HOME), nie im Repo. Je Anfrage eine | game.rs, geo.rs | Bereich |
| `adapters/karriere/wikidata.py` | Karriereanbieter Wikidata (https://www.wikidata.org, Daten unter CC0). Kein Schlüssel. Jede Anfrage trägt einen | game.rs, geo.rs | Bereich |
| `adapters/karriere/zuordnung.py` | Sichere Zuordnung einer Person: lieber keine Karriere als die eines Namensvetters. | game.rs, geo.rs | Bereich |
| `adapters/pes/__init__.py` |  | — (Paketdatei) | entfällt |
| `adapters/pes/attribute.py` | Reine Rechenfunktion: Attribut-Grenzen am Spielspeicher (PES-Skala 1-99). | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/cpk.py` | CPK-Archive des Spiels (Data/*.cpk) nur lesen: Verzeichnis (TOC) auslesen und einzelne Dateien herausholen. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/datenbank.py` | PESM-Datenbank je Speicherplatz (SQLite): Spieldaten mit Herkunft und Stand, PESM-eigene Tabellen getrennt. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/felder.py` | Katalog der Spielerfelder (Fähigkeiten, Spielstil, Spielerfähigkeiten, Bereiche, Stile je Position, Anzeigestatus) und ihre Prüfung. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/journal.py` | Unterbrochener Schreibvorgang: ein liegengebliebenes `-journal` neben einer PESM-Datenbank erkennen, vorher sichern, zurücksetzen. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/positionen.py` | Positionscode eines Formationsplatzes aus seiner Lage (Tiefe, Breite), mit den Zonentabellen des Spiels (`positionen.toml`). | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/prozess.py` | Spielprozess finden und seine Speicherbereiche auflisten (Spezifikation 02, „Auffinden“). | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/sicherheit.py` | Ablage der Backups vor dem Löschen einer Karriere (karrieren.loeschen). | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/speicherdienst.py` | Privilegierter Speicher-Hilfsdienst: liest und schreibt nur den Speicher des Spielprozesses. NUR Standardbibliothek, eine einzelne Datei. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/speicherzugriff.py` | Speicherzugriff auf den laufenden Spielprozess (nur für den Spielbeobachter, pesm/fl/beobachter/speicher.py): `/proc/<pid>/mem` direkt, wenn | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/spielerprofil.py` | Anzeige der Rohwerte aus dem Spielersatz: Seiten der Persönlichkeitspaare, Namen der Teamrollen, Einfluss. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/spielfassungen.py` | Spielgenerationen (FL26, FL27, …): eine Konfiguration (`spielfassungen.toml`) statt fester Namen im Code. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/sprache_t.py` | Texte der Spielzugriffs-Adapter aus den Textdateien (`pesm/core/texts/zugriff.json`, Englisch in `en/`), nach Sprache gemerkt. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/taktikkraft.py` | Datenbankstand -> `Kraft` und `Plan` für die Taktik-Engine. Nur lesend. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/taktikprofil.py` | Vorhandene Datenbankfelder in Eingaben für den reinen Taktikkern übersetzen. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/texte.py` | Namen zu Nummern des Spiels (Spielstile, Spielerfähigkeiten) aus der Zuordnung in `spielerprofil.toml`. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/transfer.py` | Anzeige der Rohwerte der Transferangebote: Namen der Zustände und Status je Feld. | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/vereinslook.py` | Look des eigenen Vereins je Karriere: gepinntes Wappen (`karrieren/<K-ID>/wappen.png`) und die daraus ermittelten Vereinsfarben (`verein_loo | pesdb.rs, fl27.rs | Bereich |
| `adapters/pes/vereinssuche.py` | Vereine über einen Namen von außen finden (nur Datenbank, kein Raten). | pesdb.rs, fl27.rs | Bereich |
| `adapters/storage/__init__.py` |  | — (Paketdatei) | entfällt |
| `app/__init__.py` |  | — (Paketdatei) | entfällt |
| `app/__main__.py` | Einstieg der Oberfläche (AP4): `python -m pesm.app [karriere] [--shots ORDNER] [--ablauf NAME]`. | — | entfällt |
| `app/bruecke.py` | Brücke zum Backend-Prozess (Bauauftrag §11, AP4-PLAN §1). Läuft nur im Oberflächen-Prozess. | — | entfällt |
| `app/demo.py` | Demo-Seite für AP3a: Theme und Bausteine im Rahmen der oberen Navigation. | — | entfällt |
| `app/einstellungen.py` | Einstellungen der Oberfläche (AP4-PLAN §2): Lesezeichen je Karriere in der Einstellungsdatei der Oberfläche. | — | entfällt |
| `app/klang.py` | Klangdienst: Musik (zufällig, überblendet) und Geräusche in vier Stufen (Bedienung, Bestätigung, Meldung, Ereignis). | — | entfällt |
| `app/navigator.py` | Navigator (DESIGN §14, AP4-PLAN §3): Verlaufsstapel mit erhaltenem Zustand. Zustand in Python, QML liest. | — | entfällt |
| `app/probe.py` | Messung und Selbsttest der Oberfläche: Bildzeiten, erste Reaktion, Screenshots, Marker für Tests von außen, | — | entfällt |
| `app/qtmeldungen.py` | Qt- und QML-Meldungen in die Problemliste (pesm/probleme.py): Warnungen und Fehler aus QML (ReferenceError, TypeError, fehlende Seiten, | — | entfällt |
| `app/seitenmodell.py` | Seitenmodell (AP4-PLAN §1): Listenmodell mit Seitenpuffer für `*.page`-Abfragen des Backends. | — | entfällt |
| `app/texte.py` | Textbrücke: QML fragt Texte nur hier ab (DESIGN §8), nie hartcodiert. | — | entfällt |
| `app/umleitung.py` | Umleitung gebündelter Seiten: alte Seiten-ID -> neue Seite plus Reiter-Versatz (registry/umleitungen.yaml). | — | entfällt |
| `backend/__init__.py` |  | — (Paketdatei) | entfällt |
| `backend/__main__.py` | Backend-Prozess (Bauauftrag §11), in AP1 nur mit Mock-Diensten. | — | entfällt |
| `backend/abfragezwischen.py` | Kleiner Zwischenspeicher für rechenintensive, rein lesende Seitenabfragen (taktik.startelf, taktik.planer). | — | entfällt |
| `backend/aenderungen.py` | Änderungsablage im Backend (SCHNITTSTELLEN §10): Vorschläge anderer Module ablegen, anzeigen, verwerfen. | — | entfällt |
| `backend/akademie.py` | Akademie im Backend (Sitzung S, Grundgerüst aus Sitzung B): U23 und Akademie, U23-Spiel, Entwicklung, Verpflichten fremder Jugendspieler übe | — | entfällt |
| `backend/anzeige.py` | Datum, Zahl und Geld für Anzeigetexte des Backends in der gewählten Sprache (nie für Daten, Protokolle oder Dateinamen). | — | entfällt |
| `backend/archiv.py` | Saisonarchiv als Seiten (Saisonrückblick, Auszeichnungen, Vereinsgeschichte) in der allgemeinen Form von `seitendaten.py`: | — | entfällt |
| `backend/assistent.py` | Co-Trainer-Hub im Backend (nur FL-World-Karriere): baut die `lage` aus der Weltsimulation und der Karriere-Datenbank, ruft die Regeln (`core | — | entfällt |
| `backend/ausbau_vorgaenge.py` | Jugend, Akademie, Mentor und Kabine als Vorgänge (Jugendausbau): Backend-Adapter zu den reinen Kernmodulen. | — | entfällt |
| `backend/begruendung.py` | Backend-Slots der Begründung (Grundsatz: keine Wirkung ohne Ursache). | — | entfällt |
| `backend/beobachter.py` | Spielbeobachter (FL-World) für die Oberfläche: Stand, Einstellungen (Modus, Bildweg, Anzeigeprofil, Attrappe) und Spielbericht beobachteter  | — | entfällt |
| `backend/bericht.py` | Spielbericht in voller Aufbereitung (UX-Nacht, Etappe 7), nur lesend: Ergebnis und Verlauf aus dem Bericht des Spieltags (H2, | taktik.rs | gebaut |
| `backend/besetzung.py` | Besetzung je Position, Formationsvorlagen und Rollen der Startelf: lesende Abfragen auf Basis der Rollenanalyse. | — | entfällt |
| `backend/bilder.py` | Bilder für die Oberfläche: Erlaubnis, Einstellung und Abruf über `adapters/bilder` (Wappen, Spielerbilder). | fl27.rs | gebaut |
| `backend/charakter.py` | Charakterwerte als Startwerte in der PESM-Datenbank (`pesm_persoenlichkeit`) und ihr Abruf für das Profil. | — | entfällt |
| `backend/cotrainer_aufstellung.py` | Der Co-Trainer stellt am Spieltag nach Fristende selbst auf, wenn du keine Aufstellung abgegeben hast. | — | entfällt |
| `backend/datenbankdienst.py` | Datenbank-Dienst der Karriere-Datenbank: Sicherungen, Zustandsbild, Prüfung und die Neuberechnung der Startwerte. | — | entfällt |
| `backend/dienste.py` | Methoden des Backends für den Rahmen (AP4): Registry, Status, Seitenabfragen, Änderungen, Suche. Nur Mocks. | — | entfällt |
| `backend/direktmatch_dienst.py` | Dienst „Direktmatch“: bindet den Controller `pesm/fl/direktmatch.py` an das Backend (Methoden `direktmatch.*`) und taktet ihn, solange ein A | — | entfällt |
| `backend/drama_vorgaenge.py` | Transfer- und Vertragsdrama in der Inbox: Backend-Adapter zu `core/transfers/drama_vorgang.py`. | — | entfällt |
| `backend/einrichtung.py` | Ort der lokalen Einstellung `~/.config/pesm/verbindung.toml` (überschreibbar mit PESM_VERBINDUNG, z. B. für Selbsttests). | — | entfällt |
| `backend/einstufung.py` | Einstufung im Posteingang (docs/intern/POSTEINGANG.md): verbindet die Regeln aus `core/vorgang/einstufung.py` mit dem VorgangDienst. | — | entfällt |
| `backend/entwicklung.py` | Entwicklung und Moral im Backend (Sitzung E): verbindet den Kern (`core/entwicklung`) mit Spielzugriff, Datenbank und Seiten. | world.rs | gebaut |
| `backend/ereignisse.py` | Ereignisse der Tagesphasen als Vorgänge der Inbox: Aufstellung durch den Co-Trainer (Abend), Reaktionen auf die Entscheidungen des Tages (Ab | — | entfällt |
| `backend/fans.py` | Seite „Fans und Stimmung“: sammelt die Fakten (Spiele, Tabelle, Wechsel, Torjäger, Legenden, Preise) und gibt sie an `core/fans.py`. | — | entfällt |
| `backend/feed.py` | Ticker, Fan-Feed und Gerüchte (UX-Nacht, Etappe 5). Nur lesend und nur aus Ereignissen, die die Module schon kennen: | — | entfällt |
| `backend/fl_welt.py` | Weltdaten der FL27-Welt (ohne Master League) für vorhandene Seiten, in der allgemeinen Form von `seitendaten.py` ({"kacheln", "listen"}). | — | entfällt |
| `backend/flwelt.py` | Dienst „FL-World“: die Weltsimulation ist der Motor einer FL-World-Karriere (docs/intern/WELT-TAKT.md). | — | entfällt |
| `backend/gegnerbericht.py` | Gegnerbericht (Sitzung S): Kopf, Aufstellung, Spielweise, Schlüsselspieler und Schwächen, Empfehlung des Co-Trainers, Trainingsfokus. | — | entfällt |
| `backend/gespraeche.py` | Gespräche im Backend (Sitzung H): verbindet den Kern (`core/gespraeche`) mit Transfers (D), Entwicklung und Moral (E), Vereinswelt (F), | — | entfällt |
| `backend/hierarchie.py` | Hierarchie des eigenen Kaders für die Oberfläche (nur lesend gegenüber Spiel und Spielstand). | — | entfällt |
| `backend/hintergrund.py` | Schwere Hintergrundläufe schonend ausführen: niedrige Priorität und Pause, solange das Spiel läuft. | — | entfällt |
| `backend/inbox_quellen.py` | Nebenkanäle als Nachrichten der Inbox: Ergebnisse der Liga, Tabelle, Spieltag-Neuigkeiten (Termine, Verträge, Stimmung, Rekorde), Markt-Gerü | — | entfällt |
| `backend/jugend.py` | Jugendnetzwerk im Backend (Sitzung S): U17-Talente aus der Spieldatenbank beobachten, Netzwerk je Region und Kontakte zu Akademien aufbauen, | — | entfällt |
| `backend/kabine.py` | Kabine im Backend (Sitzung Kabine): Tonkarten für die Ansprachen vor, in der Halbzeit und nach dem Spiel, Mind Games und die | mensch.rs, pages.rs | gebaut |
| `backend/kaderuebersicht.py` | Dienst `kader.uebersicht` für die Kaderseite: baut je Spieler des eigenen Kaders eine Zeile aus den vorhandenen Quellen und gibt Kennzahlen, | — | entfällt |
| `backend/karriere.py` | Karriere im Spielerprofil: reale Vorgeschichte über `adapters/karriere` (erster Anbieter Wikidata, CC0). | — | entfällt |
| `backend/karrieren.py` | Karrieren: Das eigentliche Objekt in PESM. Eine Karriere hat einen eigenen Namen, einen Verein und eine eigene Welt (pesm/fl) mit PESM-Daten | — | entfällt |
| `backend/ki_trainer.py` | KI-Trainer: Eingabe aus den PESM-Daten bauen, mit dem Stand der Entscheidung (Karriere-Tabelle) verbinden und den Wochenentwurf rechnen. | — | entfällt |
| `backend/kleinigkeiten.py` | Mehrere kleine Spielwelt-Funktionen (nur lesend gegenüber dem Spiel; nichts wird ins Spiel geschrieben): | — | entfällt |
| `backend/ladestand.py` | Ladezustand für den Ladebildschirm: was PESM gerade tut, bevor alles bedienbar ist (Spielstand lesen, Welt aktualisieren, Station, Vorwärmen | — | entfällt |
| `backend/lagen.py` | Lageberichte der Tagesphasen als Vorgänge der Inbox: Stufenwechsel des Vorstandsvertrauens (Morgen), Kadertiefe des Co-Trainers (Morgen, SOL | — | entfällt |
| `backend/laufbahn.py` | Laufbahn im Backend: verbindet den Kern (`core/laufbahn`) mit Spielzugriff, Tabelle, Trainerwelt, Posteingang (Medien) und Story-Ereignissen | — | entfällt |
| `backend/liga.py` | Wettbewerbe, Tabellen, Spielplan für die Oberfläche (nur lesend): dünne Schicht über `PesmDatenbank.wettbewerbe()`, | — | entfällt |
| `backend/lokalbilder.py` | Bilder aus dem Spiel selbst: Spielerfotos, Vereinswappen, Flaggen, Wettbewerbs-Embleme und Trainerbilder. | — | entfällt |
| `backend/manager.py` | Manager im Backend: verbindet den Kern (`core/manager`) mit Laufbahn (Gehalt, Ruf), Spielplan (spielfreie Zeit), Gesprächswelt (Wirkungen) u | — | entfällt |
| `backend/marken.py` | Markenwelt im Backend: Portal-Daten (Transfermarkt-Stil), Sponsoren, TV-Rechte und die Markenliste für die Oberfläche. | — | entfällt |
| `backend/medien.py` | Medien im Backend (Sitzung G): verbindet den Kern (`core/medien`) mit Spielzugriff, Datenbank und Seiten. | — | entfällt |
| `backend/merkmale.py` | Persönlichkeitsmerkmale im Backend: Ablage je Spieler (mk_-Tabellen), Schätzung für alle ohne Vorlage, Import einer lokalen Vorlage. | — | entfällt |
| `backend/ml_taktik.py` | Aufstellung und Taktik des eigenen Vereins aus den eingelesenen Tabellen (Spezifikation 21) als lesende Abfrage. | — | entfällt |
| `backend/note.py` | Spielernote für die Oberfläche (nur Daten): `note.letzte`, `note.saison`, `note.erklaerung`, `note.liste`. | — | entfällt |
| `backend/optionen.py` | Optionen: gewählte Karriere und Sicherungen der PESM-Datenbank (Ordner, Anzahl, jetzt sichern). | — | entfällt |
| `backend/personal.py` | Personalmarkt im Backend (Sitzung S): Bewerber, Stab, Kosten und Wirkung; verbindet den Kern (`core/personal`) mit Scouting (S), Vereinswelt | — | entfällt |
| `backend/postkarten.py` | Reiche Detailkarten der Inbox: nur schon vorhandene Daten (Spieltag-Lage, Gespräche, ML-Taktik, Tabelle); fehlt etwas, steht es als Lücke | — | entfällt |
| `backend/postleben.py` | Nachrichten, die dem Posteingang noch fehlten: Verletzung und Genesung, Physio (Rückkehr bald, Belastung des Kaders) und Finanzen (Monatsber | — | entfällt |
| `backend/saison.py` | Saisonthemen der Tagesphasen: Ereignisse, die ohne Spielverlauf tragen (frische Karriere, Vorbereitung, spielfreie Tage). | — | entfällt |
| `backend/saisonabschluss.py` | Auszeichnungen und Saisonabschluss im Backend: liest die Leistungszeilen aus dem Spielstand, vergibt im Tagestakt (einmal je Datum) und spie | — | entfällt |
| `backend/scouting.py` | Scouting im Backend (Sitzung S): verbindet den Kern (`core/scouting`) mit Transfers (D), Spieldaten, Posteingang (G) und der Story-Schicht. | — | entfällt |
| `backend/seitendaten.py` | Seiten aus eingelesenen Daten füllen, die bisher nur Probe-Inhalte kannten. | — | entfällt |
| `backend/spielordner.py` | Spielordner erkennen: Der Anwender nennt nur den Installationsordner des Spiels, PESM findet den Rest. | — | entfällt |
| `backend/spieltag.py` | Spieltag im Backend (Sitzung H2): verbindet den Kern (`core/spieltag`) mit Spielplan, Spielberichten, Transfers (D), Entwicklung (E), | — | entfällt |
| `backend/spielwelt.py` | Mock-Spielwelt für AP5 (Seiten-Skelett): Grunddaten für die Seiten, gezielte Lese-Abfragen an die Mock-Dienste, | — | entfällt |
| `backend/spielwelt2.py` | Mock-Arbeitswelt für AP5 Phase 2: Daten der Seiten Training, Kabine, Medizin, Analyse, Kaderplanung, Scouting, | — | entfällt |
| `backend/spielzugriff.py` | Dienst „Spielzugriff“: eine Quelle für die Seiten in drei Stufen. | — | entfällt |
| `backend/station.py` | Jobwechsel des Anwenders im selben Spielstand-Platz („Neue Station“): Entlassung beim alten, neue Aufgabe beim neuen Verein. | — | entfällt |
| `backend/stationen.py` | Zu welcher Station (Verein des Anwenders) gehört eine Nachricht oder ein Gespräch? Die Stationen stehen im Merker „karriere“ der Story | — | entfällt |
| `backend/story.py` | Story-Schicht für die Oberfläche: Vorschau (Schritt 1) und Motor mit Posteingang (Schritt 2). | storys.rs | gebaut |
| `backend/tagebuch.py` | Das Manager-Tagebuch im Backend: sammelt Archiv (ar_), Spielpläne und -berichte sowie Ereignisse und lässt `core/tagebuch` den Zeitstrahl ba | — | entfällt |
| `backend/tagesphase.py` | Tagesphase: ein Tag in PESM ist ein Schritt (Fabian, 20.11.2026; vorher Morgen/Mittag/Abend, docs/intern/TAGESABLAUF-IDEE.md). „Tag abschlie | — | entfällt |
| `backend/taktikanalyse.py` | Lesender Backend-Endpunkt für Spielerrollen; Konfiguration außerhalb des Kerns. | — | entfällt |
| `backend/taktikboard.py` | Spielstile je Position für die Taktiktafel. | — | entfällt |
| `backend/taktikengine.py` | Backend-Endpunkte der Taktik-Engine (nur lesend): Aufstellung analysieren, Co-Trainer fragen, Spieler gegen Rolle. | — | entfällt |
| `backend/taktikklug.py` | Backend-Endpunkte der klügeren Taktik-Engine (nur lesend, zwischengespeichert nach Datenstand): Positionseignung, Umschulungsempfehlung, Mod | — | entfällt |
| `backend/taktikmodus.py` | Backend zur Wahl des Trainers je Spiel (`taktik.modus_wahl`): merken, bei "einstellen" die Vorschläge als Entwurf vorbereiten (nur Vorschlag | — | entfällt |
| `backend/taktikplaner.py` | Taktik-Menü: Daten für Planer, Standards, Elfmeter, Gegneranweisungen und Datenhub (nur lesend, schreibt nichts). | — | entfällt |
| `backend/taktikschreiben.py` | Taktiktafel -> Karriere: der Entwurf der Tafel wird die Aufstellung des eigenen Vereins in der PESM-Datenbank der FL-World-Karriere (pesm.fl | — | entfällt |
| `backend/trainer.py` | Trainerwelt im Backend (Sitzung S): Trainerbank-Zuordnung je Verein, Vertrautheit, Trainerkarussell (Wochenlauf im Tagestakt) und die Seite  | — | entfällt |
| `backend/trainingsplan.py` | Trainingsplan im Backend (`training.*`): Einheitenkatalog, Wochenraster, Vorschau (Wirkung, Belastungsampel), Vorlagen je Karriere und die Ü | — | entfällt |
| `backend/trainingswoche.py` | Trainingswoche (UX-Nacht, Etappe 7), nur lesend aus Entwicklung und Moral (E, E2): je Spieler die in PESM entwickelten | — | entfällt |
| `backend/transfers.py` | Transferwelt im Backend (Sitzung D): verbindet den Kern (`core/transfers`) mit Spielzugriff, Datenbank und Seiten. | game.rs, transfer.rs, world.rs | gebaut |
| `backend/transfers_vorgang.py` | Transfer-Lebenszyklus im Backend: Methoden `transfers.vorgang_*` und Seitenzeilen (Vorgangsübersicht, Verhandlung, Kalender). | — | entfällt |
| `backend/vereinslook.py` | Look des eigenen Vereins (Wappen und Vereinsfarben je Karriere, siehe `adapters/pes/vereinslook.py`). | — | entfällt |
| `backend/vereinswelt.py` | Vereinswelt im Backend (Sitzung F): verbindet den Kern (`core/vereinswelt`) mit Spielzugriff, Datenbank und Seiten. | — | entfällt |
| `backend/verhandlung_dienst.py` | Verhandlungsdienst der FL-World-Karriere: dünne Schicht zwischen Oberfläche und `pesm/fl/sim/vdialog.py` (Regeln in `pesm/core/verhandlung/r | — | entfällt |
| `backend/vorgang.py` | Dienst des Vorgangsmodells (AP1): `vorgang.liste`, `vorgang.detail`, `vorgang.verlauf`, `vorgang.antworten`, `vorgang.zaehler`. | pool.rs | gebaut |
| `backend/welt.py` | Welt-Index (nur lesend): alle Spieler und Vereine der PESM-Datenbank im Arbeitsspeicher, für Suche, Vereinsseiten | pesdb.rs, world.rs | gebaut |
| `backend/wettbewerbsansicht.py` | Wettbewerbsansicht der Fußballwelt (nur lesend): Kopfkarte eines Wettbewerbs (Kennzahlen, Spieltag, Saisonwerte), Zonen der FL27-Welt, | — | entfällt |
| `backend/wettbewerbsregeln.py` | Regeln der Wettbewerbe im Backend: Zonen der Tabelle, Seite „Regeln“, Turnierbaum, Saisonende (Nachricht und Archiv). | — | entfällt |
| `backend/wirtschaft.py` | Wirtschaft im Backend: Seiten (Wirtschaftszentrale, Preise, Betrieb, Fanaktionen, Stadion-Bauprojekte) und Methoden. | — | entfällt |
| `backend/zeitmessung.py` | Zeitmessung je Backend-Aufruf (Vorgabe aus): Dauer jedes Aufrufs ins Log, dazu eine Übersicht je Methode. | — | entfällt |
| `backend/zeitung.py` | Die Zeitung im Backend: sammelt den Stoff aus Spielplan, Berichten, Ereignissen, Gerüchten und Pressekonferenzen und lässt die Redaktion (`c | — | entfällt |
| `backend/zentrale.py` | Zentralen der Bereiche (UX-Nacht, Etappe 2): `zentrale.daten {bereich}` sammelt aus den vorhandenen Modulen, was der | — | entfällt |
| `core/__init__.py` | Kern: reines Python, kein Qt, keine festen Dateipfade. | — (Paketdatei) | entfällt |
| `core/attribute.py` | Reine Funktion: Attribut-Anzeige (Bauauftrag §3). Kein Qt. | ui.rs, pages.rs | Bereich |
| `core/aufstellung_eindeutig.py` | Eine Wahrheit für „jeder Spieler in genau einer Gruppe“: Startelf (Plätze 0 … 10) > Bank (11 … 22) > Reserve (23 …). | game.rs | Bereich |
| `core/berechtigung.py` | Ist ein Anliegen berechtigt? Eine Frage vor jeder Meldung (Gespräche H, Hinweise E): schlechte Reservisten beschweren sich nicht. | mensch.rs | Bereich |
| `core/dateistand.py` | Ergebnis einer Prüfung am Stand der Datenbankdatei merken: Jede neue Verbindung zu einer großen Datenbank kostet über 1 ms (Schema laden), | entfällt (SQLite) | entfällt |
| `core/einfluesse.py` | Einflüsse der PESM-internen Attribute (DOMAIN-PRUEFUNG §8a). Reine Funktionen, kein Qt. | mensch.rs | Bereich |
| `core/fans.py` | Fans und Stimmung (Seite „Fans und Stimmung“): Stimmung je Spiel mit Ursachen, Stimmungssatz, Fanbanner und Fanstimmen. | pages.rs (Fans), wirtschaft.rs | Bereich |
| `core/gesamtwert.py` | Gesamtwert (SGB) eines Spielers für seine Hauptposition – eine Näherung von PESM, nicht der Wert des Spiels. | pesdb.rs (ovr) | Bereich |
| `core/handbuch.py` | Handbuch „Wie funktioniert das?“: die Kapitelliste liegt als Daten in `config/handbuch.json` (Kennung, Symbol, Zielseite), | pages.rs (Handbuch) | Bereich |
| `core/hierarchie.py` | Hierarchie im eigenen Kader: Hierarchiewert, Rangfolge, Rollen und Hinweise (reine Berechnung, nur Anzeige). | mensch.rs | Bereich |
| `core/kaderuebersicht.py` | Kaderübersicht: Kennzahlen, Filter, Sortierung, Empfehlungen und Aufteilung nach Mannschaftsteil für die Kaderseite (reine Rechnung, kein Le | pages.rs (Kader) | Bereich |
| `core/kleinigkeiten.py` | Reine Rechnung für mehrere kleine Spielwelt-Funktionen: Radar der Fähigkeitsgruppen, Rückennummern-Entwurf, Formtabelle. | pages.rs, postkarten.rs | Bereich |
| `core/nachweis.py` | Änderungs-Nachweis (docs/intern/AENDERUNGS-NACHWEIS.md): Urteil am Ist-Wert des eingelesenen Spielstands und das Lesen genau der Felder offe | fl27.rs | Bereich |
| `core/persoenlichkeit.py` | PES-Persönlichkeit als Eingabe für die Charakterwerte von PESM. | mensch.rs | Bereich |
| `core/pool.py` | Verbindungen zur großen Karriere-Datenbank wiederverwenden: Jede neue Verbindung lädt das Schema (über 1 ms), und eine Seite öffnet hunderte | pool.rs | gebaut |
| `core/ports.py` | Schnittstellen (Ports) zwischen Kern und Außenwelt. Kein Qt, keine festen Dateipfade. | entfällt | entfällt |
| `core/sicherung.py` | Sicherungen der Karriere-Datenbank: SQLite-Sicherungsfunktion (nie Dateikopie), Name `<Kennung>-<Anlass>-<Datum>.db`, höchstens `BEHALTEN` j | game.rs (Sicherung) | Bereich |
| `core/sicht.py` | Reine Funktion: Sicht auf versteckte Rohwerte, je Wissensstand. | scouting.rs | Bereich |
| `core/sprache.py` | Sprachwahl der Oberfläche (Standard Englisch) und Zahlen-, Datums- und Währungsformate je Sprache. | inhalt.rs | Bereich |
| `core/startelfentwurf.py` | Entwurf der PESM-Aufstellung (Startelf) in der Karriere-Datenbank: eine Zeile, gilt nur für genau die Aufnahme (`datei_sha256`), zu der er g | game.rs, fl27.rs | Bereich |
| `core/story.py` | Story-Schicht, Schritt 1: Regeln aus Konfigurationsdateien gegen benannte Werte prüfen (reine Berechnung). | storys.rs | gebaut |
| `core/stufen.py` | Stärke und Potenzial als Stufenwort statt Zahl (Regel des Anwenders: Stufenwörter statt Rechenwerte im Anwendertext). | seiten.rs (stufe_wort) | Bereich |
| `core/tabellensperre.py` | Sperre „keine Tabellenurteile ohne gespielte Spiele“ (Zahlen: config/tabellensperre.toml, je Quelle einstellbar). | medien.rs | Bereich |
| `core/taktik.py` | Erklärbare Rollenheuristiken ohne Qt, Dateien oder PES-Offsets. | taktik.rs | Bereich |
| `core/taktikeignung.py` | Positionseignung und Umschulungsempfehlung. Reiner Kern ohne Qt und Spielzugriff, baut auf `taktikengine.passung` auf. | taktik.rs | gebaut |
| `core/taktikengine.py` | Taktik-Engine: bewertet eine Aufstellung und stellt als "Co-Trainer" automatisch auf. | taktik.rs | Bereich |
| `core/taktikfm.py` | Taktik-Menü (Anzeige): abgeleitete Größen nach dem Vorbild des Taktik-Planers, rein und ohne Daten. | taktik.rs, pages.rs | Bereich |
| `core/taktikmodus.py` | Wahl des Trainers je Spiel: "auf den Gegner einstellen" oder "eigenes Spiel durchziehen" (Tabelle tk_modus der Karriere, nur PESM, idempoten | taktik.rs | Bereich |
| `core/taktikurteil.py` | Urteile des Co-Trainers über Rollen und Formation: Synergien zwischen Mitspielern, Konsistenzhinweise, Erklärung je Platz und die | taktik.rs | gebaut |
| `core/taktikvorschlag.py` | Co-Trainer-Vorschläge zur Taktik gegen einen Gegner: Reglervorschläge aus Gegnerbericht und eigener Kaderlage. Reine Funktionen. | taktik.rs | Bereich |
| `core/vereinsfarben_ableitung.py` | Entwurf (UI-Basis Schritt A, Abschnitt 1.2): aus der Vereinsfarbe die Töne vHaupt, vAuf, vTief, vSanft ableiten. Reines Python, noch nicht i | ui.rs | gebaut |
| `core/verfuegbarkeit.py` | Wer für eine Aufstellung zur Verfügung steht: eine Definition für Co-Trainer, Startelf, Planer, Hub-Vorschlag und Taktik-Seite. | game.rs | Bereich |
| `core/aenderungen/__init__.py` | Änderungsablage: Module legen Vorschläge für Spielerwerte ab, der Nutzer übernimmt oder verwirft sie. | — (Paketdatei) | entfällt |
| `core/aenderungen/aufraeumen.py` | Welche offenen Einträge der Änderungsliste sind erledigt oder veraltet? Reine Regeln, ohne Datenbank und Spiel (der Dienst gibt Kader, Vorgä | game.rs | Bereich |
| `core/aenderungen/schema.py` | Tabellen der Änderungsablage (Präfix ak_, Schemanummern 50 bis 59 gehören diesem Modul). | game.rs | Bereich |
| `core/akademie/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/akademie/ebene.py` | Ebenen der Akademie (Jugend 16-17, U23 ab 18) und Erkennen einer Beförderung aus dem Weltvergleich. Reine Funktionen, Zahlen aus `u23.toml`; | akademie.rs | Bereich |
| `core/akademie/saisonbericht.py` | Saisonbericht der Akademie als Vorgang (Jugendausbau): die Talente mit Bandbreite der Potenzialschätzung, wer uns verlässt, wer reif ist. Re | akademie.rs | Bereich |
| `core/akademie/schema.py` | Tabellen der Akademie (Präfix aka_, Schemanummer 97 im Bereich 90 bis 99 von Sitzung S; ak_ gehört der Änderungsablage). Eigenes Schema, ide | akademie.rs | Bereich |
| `core/akademie/service.py` | Die Akademie eines Speicherplatzes: Jugendkader (höchstens 40), Zuteilung U23 oder Akademie, wöchentliches U23-Spiel gegen eine fiktive | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/archiv/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/archiv/schema.py` | Tabellen des Saisonarchivs (Präfix ar_, Schemanummer 95). Eigenes Schema, idempotent; Fassung in `ar_meta`. | spielgefuehl.rs | Bereich |
| `core/archiv/service.py` | Saisonarchiv: hält bei jedem Einlesen den Stand des eigenen Vereins fest (Tabellenplatz, Punkte, Kasse, Budgets, Moral, Form), | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/assistent/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/assistent/regeln.py` | Co-Trainer-Hub: Regeln für Aufgaben und Empfehlungen, Beobachtungsliste, Spielerentwicklung und die Beziehung zum Co-Trainer. Reine Funktion | entwicklung.rs, mensch.rs | gebaut |
| `core/auszeichnungen/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/auszeichnungen/abschluss.py` | Saisonabschluss als Vorgang (Kanal Sitzung, eigene Szene): Rückblick, Spieler und Tor der Saison, Erfolge, Abschiede. Reine Funktionen, `t(k | ehrungen.rs | Bereich |
| `core/auszeichnungen/ballon.py` | Ballon d'Or als jährliches Ereignis: Kurzliste aus den Leistungsdaten der Welt und Wirkung auf eigene Spieler. Reine Funktionen, Zahlen aus  | ehrungen.rs | Bereich |
| `core/auszeichnungen/rechnen.py` | Punktformel der Auszeichnungen: reine Funktionen über Leistungszeilen, Zahlen aus `awards.toml`. | ehrungen.rs | Bereich |
| `core/auszeichnungen/schema.py` | Tabellen der Auszeichnungen (az_), idempotent angelegt in der Karriere-Datenbank. | ehrungen.rs | Bereich |
| `core/auszeichnungen/service.py` | Auszeichnungen aus echten Leistungen: Spieler des Monats je Liga, Elf der Woche, Spieler der Saison, bester Nachwuchsspieler, Ballon d'Or, T | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/begruendung/__init__.py` | Begründung: keine Wirkung ohne Ursache, keine Ursache ohne Erklärung (docs/intern/GRUNDSAETZE.md). Ohne Qt-Abhängigkeit. | — (Paketdatei) | entfällt |
| `core/begruendung/dienst.py` | Erfassen, Auflösen und Erzählen von Begründungen. `c` ist eine offene Karriere-Verbindung (sqlite3). | mensch.rs | Bereich |
| `core/begruendung/modell.py` | Datentypen der Begründung (Grundsatz: keine Wirkung ohne Ursache, keine Ursache ohne Erklärung). Ohne Qt, reine Daten. | mensch.rs | Bereich |
| `core/begruendung/schema.py` | Tabellen der Begründung (bg_), je Karriere, idempotent. | mensch.rs | Bereich |
| `core/begruendung/schreiben.py` | Zentrale Schreib-API: Moral, Stimmung, Werte, Gehalt, Entscheidungen, Meldungen, Absagen. `grund` ist Pflicht (kein Default, kein Durchlasse | mensch.rs | Bereich |
| `core/config/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/dialog/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/dialog/motor.py` | Gesprächsmotor (Sitzung H3): Optionen entstehen in jeder Runde neu aus dem Zustand des Gesprächs. | mensch.rs | Bereich |
| `core/domain/__init__.py` | Datenklassen: Spieler, Verein, Vertrag, Angebot, Scouting-Akte, Beziehung, Meldung, Änderung ... | — (Paketdatei) | entfällt |
| `core/domain/meldung.py` | Datenklassen für Ereignis- und Aktionsvertrag (Bauauftrag §6) und Statusmodell (§2, §17). | world.rs, game.rs | Bereich |
| `core/domain/regeln.py` | Karriereregeln (Bauauftrag §12): Modus je System, versioniert, protokolliert. | entwicklung.rs, mensch.rs | gebaut |
| `core/domain/sonstiges.py` | Datenklassen für Trainerprofil, Ziele, Kalender, Presse, Start und Optionen. | world.rs, game.rs | Bereich |
| `core/domain/spieler.py` | Datenklassen rund um den Spieler: Vertrag, Beziehung, Ausfall, Entwicklung, Training. | world.rs, game.rs | Bereich |
| `core/domain/spieltag.py` | Datenklassen für Spieltag und Spielstand. | world.rs, game.rs | Bereich |
| `core/domain/statistik.py` | Datenklassen für Statistik und Karriere eines Spielers (DOMAIN-PRUEFUNG.md §6). | world.rs, game.rs | Bereich |
| `core/domain/transfer.py` | Datenklassen für Transfers, Verhandlungen und Scouting. | world.rs, game.rs | Bereich |
| `core/domain/verein.py` | Datenklassen rund um den Verein: Budget, Klima, Anlagen, Geschichte, Nachwuchs, Staff. | game.rs | gebaut |
| `core/domain/zustand.py` | Datenklassen für Zustandsgrößen aus Bauauftrag §3, geprüft in DOMAIN-PRUEFUNG.md §2/§6. | world.rs, game.rs | Bereich |
| `core/entwicklung/__init__.py` | Entwicklung und Moral (Sitzung E): Kern ohne Oberfläche und ohne Spiel. | — (Paketdatei) | entfällt |
| `core/entwicklung/ausfall.py` | Laufende Ausfälle aus offenen Einträgen `ausfalltage` (quelle entwicklung): die EINE Stelle, die Datum + Tage gegen den Spielstand-Tag rechn | entwicklung.rs | Bereich |
| `core/entwicklung/ausgabe.py` | Ausgabeschicht an den Schreibdienst: Änderungsvorschläge. | entwicklung.rs | Bereich |
| `core/entwicklung/entwicklung2.py` | Entwicklung, Teil 2: Wachstumsraten je Fähigkeit, Fortschrittswege, Spielertyp, Verletzungsfolgen, Berichte. | entwicklung.rs | Bereich |
| `core/entwicklung/felder.py` | Die 25 Fähigkeiten des Spiels (Spielstand trägt 15, die zehn übrigen kommen nur aus dem Speicher) und ihre Gruppen. | entwicklung.rs | Bereich |
| `core/entwicklung/felderweg.py` | Weg ins Spiel für Felder ohne Dateiweg: die 25 Fähigkeiten, Spielstil und Spielerfähigkeiten (skill_00 … skill_40) über `felder.json`. | entwicklung.rs | Bereich |
| `core/entwicklung/gruende.py` | Gründe der Kabine: aus den echten Treibern der Moralregel (`regeln.moral_treiber`) werden Ursachen mit Gewicht, nie aus dem Ergebnis erfunde | entwicklung.rs | Bereich |
| `core/entwicklung/haltbar.py` | Haltbarkeits-Logbuch: Wie lange trägt das Spiel einen Wert, den PESM hineingeschrieben hat, bevor es ihn selbst ändert? | entwicklung.rs | Bereich |
| `core/entwicklung/kabine.py` | Kabine: Beziehungen, Gruppen, Konflikte, Gespräche, Teamgeist (reine Funktionen, Zahlen aus `entwicklung.toml`). | mensch.rs, pages.rs | gebaut |
| `core/entwicklung/medizin.py` | Medizinische Abteilung v1 (reine Funktionen, Zahlen aus `medizin.toml`): Belastungsampel, Prognose-Spanne, Reha-Phasen und Rückfallrisiko. | entwicklung.rs | Bereich |
| `core/entwicklung/medizin_gruende.py` | Gründe der Medizin: Verletzung (Belastung, Müdigkeit, Alter, Einsatz, Anfälligkeit, Vorgeschichte, Pech), Dauer-Schätzung, Meldung und Arzte | entwicklung.rs | Bereich |
| `core/entwicklung/phasen.py` | Entwicklungsphasen und Wochenpunkte (reine Funktionen, Zahlen aus `entwicklung3.toml` und `kalibrierung.json`). | entwicklung.rs | gebaut |
| `core/entwicklung/regeln.py` | Regeln von Entwicklung, Moral, Belastung und Form: reine Funktionen, Zahlen aus `entwicklung.toml`. | entwicklung.rs, mensch.rs | gebaut |
| `core/entwicklung/schema.py` | Tabellen von Entwicklung und Moral (Präfix en_, Schemanummern 30 bis 39 gehören E). | entwicklung.rs | Bereich |
| `core/entwicklung/schema2.py` | Tabellen der Entwicklung, Teil 2 (Präfix en2_, Schemanummern 110 bis 119): Wachstumsraten, Wege (Spielstile und | entwicklung.rs | Bereich |
| `core/entwicklung/service.py` | Entwicklung und Moral eines Speicherplatzes: Zustand in den en_-Tabellen, Regeln aus den Nachbardateien. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/entwicklung/verletzung.py` | Verletzungsszenario: Anfälligkeit je Spieler, Würfeln (deterministisch), Arten und Dauer. Reine Funktionen; Zahlen aus `verletzung.toml`. | entwicklung.rs | Bereich |
| `core/entwicklung/vorgaenge.py` | Medizin als Vorgänge (AP10 ↔ AP1): Rückkehr nach einer Verletzung, Verletzung und Belastung des Kaders. Reine Funktionen; `t(key, *args)` li | entwicklung.rs | Bereich |
| `core/entwicklung/wachstum.py` | Wachstumsrate je Fähigkeit (reine Funktionen, Zahlen aus `entwicklung2.toml`). | entwicklung.rs | gebaut |
| `core/entwicklung/wege.py` | Fortschrittswege: Spielstile und Spezialfähigkeiten als Entwicklungsziele (reine Funktionen). | entwicklung.rs | Bereich |
| `core/entwicklung/wochenlauf.py` | Wochenlauf der Spielerentwicklung: Phasen, Wochenpunkte, Sprünge, Verlauf, Auswahl der Einträge für die Übersicht. | entwicklung.rs | gebaut |
| `core/gegner/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/gegner/anpassung.py` | Taktikanpassung der Gegner: Anpassungstyp je Trainer (starr, reaktiv, experimentierfreudig, dogmatisch, pragmatisch) und die Taktik, die er  | taktik.rs | Bereich |
| `core/gegner/bericht.py` | Gegnerbericht: Spielweise, Schlüsselspieler, Schwächen, Empfehlung, Training, Trainerprofil (reine Funktionen, sprachneutral). | taktik.rs | gebaut |
| `core/gespraeche/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/gespraeche/anfrage.py` | Vertragsanfrage eines eigenen Spielers: echte Wahl mit Folgen (reine Funktionen, Zahlen aus gespraeche.toml, Abschnitt [anfrage]). | mensch.rs | Bereich |
| `core/gespraeche/angebot.py` | Eingehende Angebote für eigene Spieler besprechen (Angebotsgespräch): ein Gespräch mit dem Käufer (Verein) und eines mit dem Spieler. | mensch.rs | Bereich |
| `core/gespraeche/drosselung.py` | Zentrale Drosselung neuer Spieler-Anliegen (Konfiguration: drosselung.toml). Quellen: Anliegen der Gespräche (`_anliegen_sync`) und Pool-The | pool.rs | gebaut |
| `core/gespraeche/druck.py` | „Druck machen“ und Gesprächsklima: reine Regeln (Zahlen aus druck.toml) plus die Ablage in gs_meta (`DruckMixin`). | mensch.rs | Bereich |
| `core/gespraeche/haltung.py` | Haltung eines Spielers zu einem Angebot für ihn (Angebotsgespräch) und Dauer der Antworten. | mensch.rs | Bereich |
| `core/gespraeche/rhythmus.py` | Rhythmus der Anliegen: wie viele neue je Woche, wie viele offen, in welcher Reihenfolge; Eskalation und Verfall nach Zeit. Reine Funktionen. | mensch.rs | Bereich |
| `core/gespraeche/schema.py` | Tabellen der Gesprächswelt (Präfix gs_, Schemanummern 70 bis 79 gehören H). | mensch.rs | Bereich |
| `core/gespraeche/service.py` | Die Gesprächswelt eines Speicherplatzes: Gesprächsmotor, Vertrags- und Transferverhandlung, Disziplin, Gedächtnis. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/gespraeche/themen.py` | Themenmotor: aus Anlässen (themen.toml) werden Gesprächsangebote. Reine Funktionen, Zufall nur über `W.zufall` (je Woche und Partner fest). | mensch.rs | Bereich |
| `core/gespraeche/tisch.py` | Vertragsgespräch als Hin und Her: unverbindliche Vorschläge, Wünsche des Gegenübers, Geduld, Einigung, Abbruch (Dokumentation: docs/intern/V | mensch.rs | Bereich |
| `core/gespraeche/werte.py` | Zahlenmodell der Gesprächswelt (reine Funktionen, Zahlen aus gespraeche.toml): Töne, Persönlichkeit, Vertrag, Transfer, Strafe. | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/inhalt/__init__.py` | Inhaltspool (AP3): Themen mit Eröffnungen, Antworten, Reaktionen und Ausgängen, datengetrieben (TOML + Texte de/en), durch Komposition aus B | — (Paketdatei) | entfällt |
| `core/inhalt/komposition.py` | Kompositionsmaschine des Inhaltspools: Persönlichkeitstyp bestimmen, Bausteine wählen (mit Färbung), Vorlagen zusammensetzen. Zufall nur übe | pool.rs | gebaut |
| `core/inhalt/pool.py` | Der Inhaltspool: Themen aus themen/*.toml, Texte aus den Textdateien (pool.<thema>.<art>.<id>), Auswahl und Komposition reproduzierbar. | pool.rs | gebaut |
| `core/inhalt/pruefung.py` | Prüfung des Inhaltspools (Qualitätsregeln aus dem Ausbauplan, Abschnitt 6). `pruefen()` liefert eine Liste von Befunden (leer = in Ordnung). | inhalt.rs, pool.rs | gebaut |
| `core/inhalt/vorgang.py` | Pool-Themen als Vorgänge: erzeugen (Eröffnung, Optionen) und beantworten (Reaktion, Ausgang, Wirkungen über die Wege des Backends). Reine Fu | pool.rs | gebaut |
| `core/jugend/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/jugend/entscheidung.py` | Jugendspieler als Entscheidung (Jugendausbau): Beförderung, Leihe, Vertrag, mit Berater und Eltern als Gegenüber. Reine Funktionen. | akademie.rs | Bereich |
| `core/jugend/leihe.py` | Leihe eines Jugendspielers (Jugendausbau): Zielvereine aus der Vereinswelt mit Einsatzerwartung und Gehaltsanteil. Reine Funktionen, nichts  | akademie.rs | Bereich |
| `core/jugend/mentor.py` | Mentor-Zuordnung als Entscheidung (Jugendausbau; Nutzerkonzept ~/pesm-konzepte/MEDIZIN-KONZEPT.md): ein erfahrener Spieler gibt eine Spezial | akademie.rs | Bereich |
| `core/jugend/schema.py` | Tabellen des Jugendnetzwerks (Präfix jn_, Schemanummer 93 im Bereich 90 bis 99 von Sitzung S). Eigenes Schema, idempotent; Fassung in `jn_me | akademie.rs | Bereich |
| `core/jugend/service.py` | Das Jugendnetzwerk eines Speicherplatzes: Netzwerkstärke je Region, Akademie-Kontakte, Talente (U17 aus der Spieldatenbank), Beobachtung, An | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/jugend/vorgangsbau.py` | Gemeinsame Bausteine der Entscheidungs-Vorgänge von Jugend, Akademie, Mentor und Kabine (Jugendausbau). Reine Funktionen, `t(key, *args)` li | akademie.rs | Bereich |
| `core/kabine/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/kabine/ansprachen.py` | Halbzeitansprache und Mind Games (reine Funktionen, Zahlen aus kabine.toml). Die Ansprachen vor und nach dem Spiel liegen im Spieltag (H2). | rest3.rs | gebaut |
| `core/kabine/konflikte.py` | Kabinenkonflikte und Hierarchie als Vorgänge (Jugendausbau): Streit, gespaltene Gruppen, wankender Kapitän, nicht aufgenommener Neuzugang. R | mensch.rs | Bereich |
| `core/ki_trainer/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/ki_trainer/entscheidung.py` | KI-Trainer: was ein Trainer in dieser Woche mit seiner Mannschaft macht (reine Funktionen, ohne Spiel, Datenbank und Qt). | ki.rs | Bereich |
| `core/ki_trainer/speicher.py` | Karriere-Tabellen des KI-Trainers (idempotent angelegt): der Schalter je Karriere (Standard AUS) und der Entwurf je Verein und Woche. | ki.rs | Bereich |
| `core/ki_trainer/welt.py` | KI-Trainer-Welt (reine Funktionen): Spielidee, Reaktion auf Ergebnisse, Elf nach Spielwichtigkeit (Entlassung und Einstellung: core/trainer/ | pesdb.rs, world.rs | gebaut |
| `core/laufbahn/__init__.py` | Laufbahn des Nutzers als Trainer: Profil, Ruf, Vertrag, Angebote anderer Vereine, Verhandlung, Bewerbung, Wechsel als Vorgang. | — (Paketdatei) | entfällt |
| `core/laufbahn/gespraech.py` | Gespräche der Laufbahn: Verhandlung (sechs Punkte) und Bewerbung beim Präsidium (vier Schritte). Reine Funktionen auf JSON-tauglichen Zustän | mensch.rs | Bereich |
| `core/laufbahn/schema.py` | Tabellen der Laufbahn (Präfix lb_, Schemanummern 120 bis 129). Alles PESM-intern: nichts davon geht ins Spiel. | mensch.rs | Bereich |
| `core/laufbahn/service.py` | Die Laufbahn des Nutzers als Trainer: Profil, Vertrag, Stationen, Angebotsmotor, Gespräche, Entscheidung und Wechsel als Vorgang. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/laufbahn/werte.py` | Zahlenmodell der Laufbahn (reine Funktionen, Zahlen aus laufbahn.toml): Ruf, Reputation, Lizenz, Passung, Interesse, Rahmen der Verhandlung. | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/manager/schema.py` | Tabellen des Managers (Präfix mg_, Schemanummer 130). Alles PESM-intern: nichts davon geht ins Spiel. Idempotent; die Fassung steht in `mg_m | mensch.rs | Bereich |
| `core/manager/service.py` | Der Manager als Person: Werte, Stil aus dem Verhalten, Passung zu Spielertypen, Lehrgänge, Freizeit, Wohnen und eine Privatkasse. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/marken/__init__.py` | Markenwelt: liest `marken.toml` (einzige Quelle für Marken). UI und Textbausteine fragen nur hier. | — (Paketdatei) | entfällt |
| `core/mechanics/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/medien/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/medien/pool.py` | Medienpool: Medien mit Typ, Land, Größe, Ton und Tags (core/config/story/medienpool.json, Quelle Pressetribüne, Lücken erfunden). | pool.rs | gebaut |
| `core/medien/posteingang.py` | Posteingang: Meldungen ablegen (SCHNITTSTELLEN §13.1). Arbeitet auf einer offenen Verbindung, damit auch Module mit eigener | medien.rs | Bereich |
| `core/medien/schema.py` | Tabellen der Medien- und Ereigniswelt (Präfix md_, Schemanummern 60 bis 69 gehören G). | medien.rs | Bereich |
| `core/medien/service.py` | Die Medienwelt eines Speicherplatzes: Posteingang, Pressekonferenzen, Journalisten, Gerüchte und Dementis. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/medien/werte.py` | Zahlenmodell der Medienwelt (reine Funktionen, Zahlen aus medien.toml). | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/merkmale/__init__.py` | Persönlichkeitsmerkmale (41 Begriffe für das öffentliche Auftreten): Katalog, Wirkung, Schätzung, Zuordnung einer Vorlage. | — (Paketdatei) | entfällt |
| `core/merkmale/katalog.py` | Katalog der Persönlichkeitsmerkmale (pesm/core/config/merkmale.json) und seine Auswertung: Wirkung, Hooks, Widersprüche. | entwicklung.rs | Bereich |
| `core/merkmale/schaetzen.py` | Merkmale für Spieler ohne Vorlage: stabil aus dem Satzindex erzeugt (gleiche Eingabe, gleiche Merkmale). | entwicklung.rs | gebaut |
| `core/merkmale/schema.py` | Tabellen der Merkmale (Präfix mk_, Schemanummern 110 bis 119). | entwicklung.rs | Bereich |
| `core/merkmale/zuordnen.py` | Zuordnung einer Vorlage (Name, Nation, Position, Merkmale) zu den Spielern der Datenbank. | entwicklung.rs | Bereich |
| `core/note/__init__.py` | Spielernote (Sitzung Note): PESM benotet die eigenen Spieler selbst. Die Note lebt nur in PESM und geht nie ins Spiel. | — (Paketdatei) | entfällt |
| `core/note/service.py` | Spielernote aus der Karriere-Datenbank (nur lesen). Gerechnet wird einmal je Datenstand für den ganzen Kader und im Speicher gehalten; | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/note/werte.py` | Rechenkern der Spielernote: reine Funktionen, keine Datenbank, kein Zufall. Zahlen aus `note.toml`. | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/personal/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/personal/foto.py` | Prozedurale Porträts für Mitarbeiter: Merkmale stabil aus Name und Herkunft, als Vektorgrafik (SVG) oder für den Bildanbieter (Qt). | personal.rs | Bereich |
| `core/personal/medizin.py` | Medizinische Abteilung (MEDIZIN-KONZEPT): aus Posten (Arzt, Physios, Reha-Trainer, Sportwissenschaftler) und Einrichtung (Reha-Zentrum, Diag | personal.rs | Bereich |
| `core/personal/schema.py` | Tabellen des Personalmarkts (Präfix pm_, Schemanummer 92 im Bereich 90 bis 99 von Sitzung S). | personal.rs | Bereich |
| `core/personal/service.py` | Der Personalmarkt eines Speicherplatzes: Bewerber, Stab, Kosten, Wirkung. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/personal/stabplan.py` | Stabplan eines Vereins: Plätze (Slots) je Größenklasse, Nationen mit passenden Namen (erfundene Personen), Philosophie, Beziehung, Wirkungst | personal.rs | Bereich |
| `core/personal/werte.py` | Zahlenmodell des Personalmarkts (reine Funktionen, Zahlen aus personal.toml): Bewerber, Gehalt, Fähigkeiten, Wirkung. | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/scouting/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/scouting/analyse.py` | Analyse: Spiel, Mannschaft, Gegner, Spielervergleich (reine Funktionen auf Lage und Daten). | scouting.rs | Bereich |
| `core/scouting/gebiet.py` | Gebiete des Scoutings: Welt → Weltregion → Land → Liga → Verein (reine Funktionen auf der Lage). | scouting.rs | Bereich |
| `core/scouting/genauigkeit.py` | Berichtsstufen und Genauigkeit (reine Funktionen): erst ein grober Bericht, dann nach Beobachtungen ein genauerer, nach einem Spielertreffen | scouting.rs | Bereich |
| `core/scouting/passung.py` | Passungsmodell (reine Funktionen, Zahlen aus scouting.toml): passt der Spieler ins Team? Sechs Dimensionen mit Ampel (grün, gelb, rot oder u | scouting.rs | Bereich |
| `core/scouting/schema.py` | Tabellen des Scoutings (Präfix sc_, Schemanummern 90 bis 99 gehören S). | scouting.rs | Bereich |
| `core/scouting/scoutperson.py` | Scouts als Personen (reine Funktionen, Zahlen aus scouting.toml): Persönlichkeit färbt die Einschätzung, das Vertrauen des Trainers wächst o | scouting.rs | gebaut |
| `core/scouting/service.py` | Die Scoutingwelt eines Speicherplatzes: Scouts als Personen, Aufträge, Berichte, Beobachtung, Hinweise, Analyse. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/scouting/sterne.py` | Sterne und Wissensstufen (reine Funktionen, Zahlen aus `scouting.toml` [sterne]): eine Stelle für CA (aktuelle Stärke) und PA (Potenzial) al | scouting.rs | Bereich |
| `core/scouting/suchprofil.py` | Suchprofile: Kriterien, nach denen Spieler gefunden werden (sofort aus den Spielerdaten oder als Auftrag an einen Scout). | scouting.rs | Bereich |
| `core/scouting/umfeld.py` | Umfeldaussagen mit Quelle und Verlässlichkeit, Persönlichkeits-Tags (reine Funktionen, Zahlen aus scouting.toml). | scouting.rs | Bereich |
| `core/scouting/vorgaenge.py` | Scouting als Vorgänge (AP7a ↔ AP1): Steckbrief-Mail (mit Umfeldaussagen und Passungskarte als Schritte) und Scout-Widerspruch. Reine Funktio | scouting.rs | Bereich |
| `core/scouting/werte.py` | Zahlenmodell des Scoutings (reine Funktionen, Zahlen aus scouting.toml): Scouts, Reichweite, Wissen, Berichte, Vorsprung. | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/services/__init__.py` | Dienst-Schnittstellen (Protocol), eine je Menüpunkt aus der Registry. | — (Paketdatei) | entfällt |
| `core/services/mannschaft.py` | Dienst-Schnittstellen für den Bereich Mannschaft (Bauauftrag §5). | game.rs | Bereich |
| `core/services/profilwerte.py` | Reiner Dienst: Profilwerte (sechs Achsen) aus Einzelattributen (neu, s. DOMAIN-PRUEFUNG.md). | game.rs | Bereich |
| `core/services/spieltag.py` | Dienst-Schnittstellen für den Bereich Spieltag (Bauauftrag §5). | game.rs | Bereich |
| `core/services/system.py` | Dienst-Schnittstellen für den Bereich System (Bauauftrag §5). | game.rs | Bereich |
| `core/services/transfers.py` | Dienst-Schnittstellen für den Bereich Transfers (Bauauftrag §5). | game.rs, transfer.rs, world.rs | gebaut |
| `core/services/verein.py` | Dienst-Schnittstellen für den Bereich Verein (Bauauftrag §5). | game.rs | gebaut |
| `core/services/welt.py` | Dienst-Schnittstellen für den Bereich Fußballwelt (Bauauftrag §5). | pesdb.rs, world.rs | gebaut |
| `core/services/zentrale.py` | Dienst-Schnittstellen für den Bereich Zentrale (Bauauftrag §5). | game.rs | Bereich |
| `core/services/mocks/__init__.py` | Mock-Dienste: ein Mock je Dienst aus der Registry, mit erfundenen Beispieldaten. | — (Paketdatei) | entfällt |
| `core/services/mocks/beispieldaten.py` | Gemeinsame, frei erfundene Beispieldaten für alle Mock-Dienste. | game.rs | Bereich |
| `core/services/mocks/mannschaft.py` | Mock-Dienste für den Bereich Mannschaft. Erfundene Beispieldaten, deterministisch. | game.rs | Bereich |
| `core/services/mocks/regeln.py` | Mock-Beimischung für `ModusFaehig` (core/ports.py, Bauauftrag §12). Hält den Modus nur im | entwicklung.rs, mensch.rs | gebaut |
| `core/services/mocks/spieltag.py` | Mock-Dienste für den Bereich Spieltag. Erfundene Beispieldaten, deterministisch. | game.rs | Bereich |
| `core/services/mocks/system.py` | Mock-Dienste für den Bereich System. Erfundene Beispieldaten, deterministisch. | game.rs | Bereich |
| `core/services/mocks/transfers.py` | Mock-Dienste für den Bereich Transfers. Erfundene Beispieldaten, deterministisch. | game.rs, transfer.rs, world.rs | gebaut |
| `core/services/mocks/verein.py` | Mock-Dienste für den Bereich Verein. Erfundene Beispieldaten, deterministisch. | game.rs | gebaut |
| `core/services/mocks/welt.py` | Mock-Dienste für den Bereich Fußballwelt. Erfundene Beispieldaten, deterministisch. | pesdb.rs, world.rs | gebaut |
| `core/services/mocks/zentrale.py` | Mock-Dienste für den Bereich Zentrale. Erfundene Beispieldaten, deterministisch. | game.rs | Bereich |
| `core/spieltag/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/spieltag/prematch.py` | Erwartung vor dem Spiel und Voreinstellung der Ansprache (Pre-Match, wie im Football Manager). | game.rs, livespiel.rs | Bereich |
| `core/spieltag/schema.py` | Tabellen des Spieltag-Erlebnisses (Präfix sp_, Schemanummern 100 bis 109 gehören H2). | game.rs, livespiel.rs | Bereich |
| `core/spieltag/service.py` | Die Spieltagwelt eines Speicherplatzes: Spielberichte, Teamgespräche vor und nach dem Spiel, Spieltag-Ablauf, Neuigkeiten der Zentrale. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/spieltag/werte.py` | Zahlenmodell und Textbau des Spieltag-Erlebnisses (reine Funktionen, Zahlen aus spieltag.toml). | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/tagebuch/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/tagebuch/service.py` | Das Manager-Tagebuch: aus Saisonarchiv, Spielberichten und Ereignissen wird ein Zeitstrahl Saison > Monat > Eintrag (reine Funktionen, keine | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/texts/__init__.py` | Texte der Oberfläche: `de.json` und je System eine weitere Datei im selben Ordner (etwa `taktik.json`); die englische | — (Paketdatei) | entfällt |
| `core/trainer/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/trainer/bank.py` | Trainerbank: laden, Vereinsnamen tolerant zuordnen (reine Funktionen). | trainerbank.rs | gebaut |
| `core/trainer/schema.py` | Tabellen der Trainerwelt (Präfix tb_, Schemanummer 96): Trainer je Verein, freie Trainer (Pool), Ereignisse des Trainerkarussells. Idempoten | trainerbank.rs, mensch.rs | Bereich |
| `core/trainer/service.py` | Trainerwelt: ein Trainer je Verein (aus der Trainerbank oder ersatzweise aus dem Spielstand), Vertrautheit der Mannschaft mit seinem Stil, T | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/trainingsplan/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/trainingsplan/schema.py` | Tabellen des Trainingsplans (Präfix tp_, Schemanummer 98): der aktive Wochenplan und die benannten Vorlagen der Karriere. Idempotent; Fassun | game.rs | Bereich |
| `core/trainingsplan/service.py` | Trainingsplan: Einheitenkatalog, Wochenraster (7 Tage x 4 Slots), Wirkung, Belastungsampel, Vorlagen und Übernahme in das Entwicklungsmodell | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/trainingsplan/spezial.py` | Spezialtraining: welche Spezialfähigkeiten (Fortschrittswege) ein Spieler im Trainingsplan wählen darf. Reine Funktionen, keine Datenbank. | game.rs | Bereich |
| `core/trainingsplan/vorgaenge.py` | Training als Vorgänge (Mail des Trainerstabs, Thema `entwicklung`; die Inbox zeigt sie als Spielerliste mit Gesicht und Wert, `VorgangWochen | game.rs | Bereich |
| `core/transfers/__init__.py` | Transferwelt (Sitzung D): Marktwert, Scouting, Verhandlung, Angebote, Fenster, Gerüchte, Ausführungsschicht. | — (Paketdatei) | entfällt |
| `core/transfers/abgleich.py` | Saga-Abgleich: PESM trackt den Stand selbst. Nach jedem Einlesen und Tageswechsel prüft jeder offene Transfer-Vorgang, ob seine Voraussetzun | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/ausfuehrung.py` | Ausführungsschicht: setzt eine in PESM vereinbarte Verhandlung im Spiel um. | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/begruendet.py` | Begründung der Transfers (Grundsatz „Keine Wirkung ohne Ursache“, docs/intern/GRUNDSAETZE.md). | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/bruecke.py` | Brücke vom Transfer-Vorgang (`tv_vorgang`) zum Vorgangsmodell (H, `Vorgangswelt.einspielen`): reine Beschreibung, kein Schreiben. | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/chance.py` | Erfolgschance einer Verhandlung, 0 bis 100 Prozent, getrennt für Verein und Spieler plus Gesamt. | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/drama.py` | Transfer- und Vertragsdrama: Rivalen und Abwerbeversuche, Berater, die verzögern, ein Verein, der abbricht, der Deal im letzten Moment, | transfer.rs | gebaut |
| `core/transfers/drama_vorgang.py` | Transfer- und Vertragsdrama als Vorgänge der Inbox (Quelle `drama`): Arztbericht, Nachforderung, Rivale, Streik, Vertragspoker und Meldungen | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/druck.py` | Druck machen in den Phasen des Transfers: Anfrage (Interesse klären), Vereinsangebot, Spielerverhandlung. Regeln und Ablage wie in `core/ges | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/eignung.py` | Eignungsbewertung je Paar Spieler/Verein (reine Funktionen): ob ein Wechsel logisch ist. | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/formular.py` | Angebotsformular für einen fremden Spieler: Vorbelegung, Erfolgschance live (Verein, Spieler, Gesamt) und Abschicken nach Bestätigung. | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/ki_regeln.py` | Regeln der KI-Transferwelt über dem Plan (ki_welt.py): Sperren, Bedarf, Reihenfolge, Deadline Day, Wiederkehr gescheiterter Ziele, Gerüchte. | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/ki_welt.py` | KI-Transferwelt mit Plan und Gedächtnis (Schalter `ki_welt` je Karriere, alles PESM-intern, nichts geht ins Spiel). | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/markt.py` | Was der Markt von selbst tut: eingehende Angebote der KI-Vereine, Gerüchte, Rolle und Verkaufsbereitschaft. | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/schema.py` | Tabellen der Transferwelt (Präfix tf_, Schemanummern 20 bis 29 gehören D). | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/scouting.py` | Scouting: Wissen, Berichte mit Spannen, Kandidatenlisten (reine Funktionen). | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/service.py` | Die Transferwelt eines Speicherplatzes: Zustand in den tf_-Tabellen, Regeln aus den Nachbardateien. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/transfers/sondierung.py` | Sondierung eines fremden Spielers vor dem Angebot: wie im echten Transfergeschäft erst fragen, sprechen, Interesse zeigen (Schema 86). | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/verhandlung.py` | Verhandlungsregeln (reine Funktionen): Antwort eines Vereins, Antwort eines Spielers, Rahmen aus Budget und Fenster. | game.rs, transfer.rs | gebaut |
| `core/transfers/vorgang.py` | Transfer-Lebenszyklus (Schema 80 bis 89): ein Vorgang führt Interesse, Gerücht, Anfrage, Gespräche, Gegenangebot, Einigung, | pool.rs | gebaut |
| `core/transfers/wechselbezug.py` | Fremde Wechsel verständlich machen: Warum geht es den Nutzer etwas an? (Ursachen: weltrelevanz.py.) | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/weltrelevanz.py` | Relevanzfilter der Weltmeldungen für die Inbox: Was in der Fußballwelt passiert, steht in der Transfertabelle, im Nachrichtenarchiv und in d | transfer.rs, kimarkt.rs | Bereich |
| `core/transfers/werte.py` | Marktwert, Ablöseforderung und Gehaltserwartung (reine Funktionen, Zahlen aus transfers.toml). | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/vereinswelt/__init__.py` | Vereinswelt (Sitzung F): Finanzen, Fans, Stadion und Anlagen, Präsidium, Staff, Nachwuchs, Sponsoren. | — (Paketdatei) | entfällt |
| `core/vereinswelt/schema.py` | Tabellen der Vereinswelt (Präfix vw_, Schemanummern 40 bis 49 gehören F). | farm.rs, seiten_farm.rs | Bereich |
| `core/vereinswelt/service.py` | Die Vereinswelt eines Speicherplatzes: Zustand in den vw_-Tabellen, Regeln aus vereinswelt.toml. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/vereinswelt/werte.py` | Zahlenmodell der Vereinswelt (reine Funktionen, Zahlen aus vereinswelt.toml). | game.rs, klauseln.rs, mensch.rs, transfer.rs, verhandlung.rs, wirtschaft.rs | gebaut |
| `core/vereinswelt/wirtschaft.py` | Wirtschaft des Vereins: Stellschrauben, Nachfrage je Heimspiel, Vorschau, Bauprojekte und Fanaktionen. | farm.rs, seiten_farm.rs | Bereich |
| `core/verhandlung/__init__.py` | Verhandlungsablauf in Runden (reine Regeln, ohne Welt). Siehe docs/intern/VERHANDLUNG.md. | — (Paketdatei) | entfällt |
| `core/verhandlung/runden.py` | Verhandlung als Zustand mit Zügen: Vorschlag -> sofortige Reaktion -> Runden -> Einigung oder Ende (reine Funktionen, die Welt liefert nur Z | game.rs, mensch.rs, pages.rs, verhandlung.rs | gebaut |
| `core/vorgang/__init__.py` | Vorgangsmodell (AP1): jedes Thema ist ein Vorgang mit Absender, Anlass, Kanal, Kontext, Optionen, Verlauf, Folgen und Gedächtnis. | — (Paketdatei) | entfällt |
| `core/vorgang/berichte.py` | Berichte der Nebenkanäle als Vorgänge der Inbox (reine Funktionen; Zahlen in inbox_quellen.toml). | vorgang.rs | Bereich |
| `core/vorgang/braucht_dich.py` | Was braucht dich? Eine einzige Definition für Zähler, Reiter „Entscheidungen“, Kachel der Zentrale und Spielflussband. | vorgang.rs | Bereich |
| `core/vorgang/bruecke.py` | Brücke zu den heutigen Quellen: aus Gesprächen (gs_gespraech + gs_runde), Anliegen (gs_anliegen) und Posteingangs-Meldungen (md_meldung) wer | vorgang.rs | Bereich |
| `core/vorgang/cotrainer.py` | Der Co-Trainer übernimmt SOLL-Nachrichten, wenn ihre Frist abläuft (feste Standardlinie, Zahlen in pflicht.toml [vertrauen]). Reine Funktion | vorgang.rs | gebaut |
| `core/vorgang/dringend.py` | Wer ist wirklich wichtig? Reine Regel über die Köpfe der Inbox (Schwellen in dringend.toml): nur offene Vorgänge mit echter Frist oder laufe | vorgang.rs | Bereich |
| `core/vorgang/eingang.py` | Eine Eingangs-Schnittstelle für Rohereignisse ohne eigenen Spiegel (docs/intern/POSTEINGANG.md): Module liefern ein Ereignis mit Grund, die  | vorgang.rs | Bereich |
| `core/vorgang/einstufung.py` | Einstufung des Posteingangs: eine zentrale Regel für alle Nachrichten (Regeln und Zahlen in einstufung.toml, docs/intern/POSTEINGANG.md). | vorgang.rs | Bereich |
| `core/vorgang/ereignisse.py` | Konfiguration der Tagesereignisse (ereignisse.toml). | vorgang.rs | Bereich |
| `core/vorgang/kabine_stimmung.py` | Stimmungs-Kleinigkeiten der Kabine (Folgen `kb_*_gut`/`kb_*_szene`) sind Rauschen: kein Einzelbrief je Spieler, sondern Zeilen EINER Sammeln | vorgang.rs | Bereich |
| `core/vorgang/kanal.py` | Kanalwahl nach Anlass, Dringlichkeit und Beziehung (reine Funktionen, Zahlen in kanaele.toml). | vorgang.rs | Bereich |
| `core/vorgang/lagen.py` | Lageberichte der Tagesphasen: Stufe des Vorstandsvertrauens, Kadertiefe-Befunde des Co-Trainers (reine Funktionen; Zahlen in lagen.toml). | vorgang.rs | Bereich |
| `core/vorgang/modell.py` | Lebenszyklus eines Vorgangs: erlaubte Statuswechsel. | vorgang.rs | Bereich |
| `core/vorgang/pflicht.py` | Muss / Soll / Info: welche Nachrichten du beantworten MUSST, welche der Co-Trainer übernehmen darf, welche nur Info sind (Regel in pflicht.t | vorgang.rs | Bereich |
| `core/vorgang/phasen.py` | Konfiguration der Tagesphasen (tagesphase.toml): Anzahl und Themenarten je Phase, Training. Eine Quelle für Deckel und Mindestplatz. | entwicklung.rs | gebaut |
| `core/vorgang/schema.py` | Tabellen des Vorgangsmodells (vg_), idempotent angelegt in derselben Karriere-Datenbank wie die gs_-Tabellen. | vorgang.rs | Bereich |
| `core/vorgang/schlagzeile.py` | Texte der Einstufung: Begründungssatz („Eilmeldung: eine Stammkraft fällt etwa 4 Wochen aus.“), Eilmeldung im Stil einer TV-Schlagzeile (Str | vorgang.rs | Bereich |
| `core/vorgang/service.py` | Die Vorgangswelt einer Karriere: anlegen (idempotent), Verlauf, Statuswechsel, Liste und Detail, Spiegeln aus den heutigen Quellen, Gedächtn | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `core/vorgang/sitzung.py` | Sitzungsraum (Boardroom) eines Vorgangs im Kanal `sitzung`: wer am Tisch sitzt, wie jedes Mitglied zum Trainer steht und wie fest der Stuhl  | vorgang.rs | Bereich |
| `core/wettbewerbe/__init__.py` | Regeln der Ligen, Pokale und Europapokale: Datenmodell (wb_), Ableitung aus dem Spielstand, Konfiguration je Spielwelt, Zonen, Turnierbaum,  | — (Paketdatei) | entfällt |
| `core/wettbewerbe/ableiten.py` | Regeln aus dem Spielstand ableiten und mit der Konfiguration abgleichen. Reine Funktionen auf einfachen Listen (kein Datenbankzugriff). | cups.rs, ligaregeln.rs | Bereich |
| `core/wettbewerbe/konfig.py` | Konfiguration der Wettbewerbsregeln je Spielwelt (wettbewerbe.toml). | cups.rs, ligaregeln.rs | Bereich |
| `core/wettbewerbe/regelwerk.py` | Regelwerk der Wettbewerbe: speichern (Ableitung plus Konfiguration), Zonen der Tabelle, Qualifikation, Turnierbaum, Saisonende. | cups.rs, ligaregeln.rs | Bereich |
| `core/wettbewerbe/schema.py` | Tabellen der Wettbewerbsregeln (Präfix wb_, Schemanummern 130 bis 139). PESM-eigen: sie bleiben beim Neu-Einlesen bestehen und werden dabei  | cups.rs, ligaregeln.rs | Bereich |
| `core/wirtschaft/budget_start.py` | Budget-Herleitung aus dem Kader: eine einzige Rechnung für Karrierestart, Saisonwechsel und Neuberechnung. | wirtschaft.rs | gebaut |
| `core/wirtschaft/praemien.py` | Wettbewerbs-Prämien, Liga-Fernsehgeld und Auf-/Abstiegsgeld (Konfiguration: `core/config/wirtschaft/praemien/*.toml`). | wirtschaft.rs | Bereich |
| `core/wirtschaft/vorgaenge.py` | Wirtschaft als Vorgänge: Stadionausbau, Sponsorenangebote, Merch-Kampagnen, Ticketpreis-/Fanprotest und der monatliche Finanzbericht. | wirtschaft.rs | Bereich |
| `core/zeitung/__init__.py` |  | — (Paketdatei) | entfällt |
| `core/zeitung/artikel.py` | Berichte der Blätter: aus dem Stoff einer Ausgabe schreiben mehrere Zeitungen mit eigenem Ton je einige Artikel (Überschrift, Unterzeile, | medien.rs | Bereich |
| `core/zeitung/redaktion.py` | Die Redaktion: aus dem Stoff eines Zeitraums wird die Titelseite einer Ausgabe (reine Funktionen, Zahlen aus zeitung.toml, Texte aus zeitung | medien.rs | Bereich |
| `core/zeitung/schema.py` | Tabellen der Zeitung (Präfix zt_, Schemanummer 140). Alles PESM-intern; idempotent, die Fassung steht in `zt_meta`. | medien.rs | Bereich |
| `core/zeitung/service.py` | Die Zeitung einer Karriere: Ausgaben in den zt_-Tabellen, erzeugt im Tagestakt (nie beim Öffnen), idempotent je Datum. | game.rs, ki.rs, mensch.rs, trainerbank.rs | gebaut |
| `fl/__init__.py` | FL27-Welt ohne Master League: Import aus der FL-Datenbank (pesdb), Weltsimulation (aus fl-world), Original-Welt sichern/zurückladen, Export  | — (Paketdatei) | entfällt |
| `fl/aufstellung.py` | Aufstellung des eigenen Vereins in der FL27-Welt (ohne Master League): Erstbelegung aus der Weltsimulation, Speichern aus der Taktiktafel, | fl27.rs, live.rs | Bereich |
| `fl/bilder.py` | Bilder der FL27-Welt aus den Spieldateien: Gesichter, Wappen, Stadionfotos (nur lesen im Spielordner, nichts ändern). | fl27.rs | gebaut |
| `fl/dbschicht.py` | Datenbank-Schicht ohne Meister Liga: Basis -> Modifikatoren -> Wirkwert -> Datenbank (Beschreibung: docs/intern/DB-SCHICHT.md). Werte: confi | fl27.rs, live.rs | Bereich |
| `fl/dbtreiber.py` | Schreib-Schnittstelle der Datenbank-Schicht: zwei Treiber mit derselben Form (docs/intern/DB-SCHICHT.md). | fl27.rs, live.rs | Bereich |
| `fl/direktmatch.py` | PESM Direct Match Controller: PESM simuliert die Welt, Football Life ist nur die 3D-Engine für die vom Anwender gewählte Partie. | fl27.rs, live.rs | Bereich |
| `fl/export.py` | Weltsimulation -> PESM-Datenbank: legt dieselben Tabellen an, die sonst der Master-League-Einleser füllt (Spieler, Werte, Fähigkeiten, | fl27.rs, live.rs | Bereich |
| `fl/export_module.py` | Weltdaten der Module (Transfers, Gerüchte, Ehrungen, Chronik, Europapokal, Akademie/U19, Rücktritte, Spielberichte, Begründungen) -> PESM-Da | fl27.rs, live.rs | Bereich |
| `fl/formmodell.py` | Form und Verletzungen im Freundschaftsmodus (Beschreibung: docs/intern/FORM-VERLETZUNG.md, Werte: config/formmodell.json). | fl27.rs, live.rs | Bereich |
| `fl/gegnerbericht.py` | Scoutbericht über den Gegner eines kommenden Spiels einer FL-World-Karriere (Seite `gegnerbericht`, Methode `flwelt.gegnerbericht`). | fl27.rs, live.rs | Bereich |
| `fl/karriere.py` | FL-World-Karriere: eine eigene Karriereart ohne Master League und ohne ML-Platz (docs/intern/WELT-TAKT.md). | fl27.rs, live.rs | Bereich |
| `fl/marken_zugriff.py` | Zugriff der Weltsimulation auf die Markenkonfiguration (pesm/core/marken); hält sim/ frei von core-Importen im Kopf. | fl27.rs, live.rs | Bereich |
| `fl/matchstart.py` | Match-Konfiguration für das Spiel (Datei im Briefkasten, die das Sider-Modul `flw_matchstart.lua` liest). Beschreibung, Schema, Grenzen: doc | fl27.rs, live.rs | Bereich |
| `fl/originalwelt.py` | Original-Spielwelt sichern und zurückladen, Karriere-Datenbank ins Spiel laden. | fl27.rs, live.rs | Bereich |
| `fl/pesdb.py` | Nur lesend: pesdb/*.bin der FL27-Datenbank -> ein dict (Vereine, Wettbewerbe, Kader). Aus fl-world (world.py) übernommen. | fl27.rs, live.rs | Bereich |
| `fl/portal.py` | Daten des Transfer- und Datenportals (Marke `transfermarkt`): alles aus der Weltsimulation, nichts aus dem Netz. | fl27.rs, live.rs | Bereich |
| `fl/profil.py` | Weltangaben für das Spielerprofil einer FL-World-Karriere (Dienst `flwelt.spieler`, Seite Spielerprofil > Überblick). | fl27.rs, live.rs | Bereich |
| `fl/sofort.py` | Sofort-Schreiben: jede Änderung, die das Spiel betrifft, geht ohne Umweg in die pesdb-Kopie der Karriere (docs/intern/MATCHSTART.md, Teil 1) | fl27.rs, live.rs | Bereich |
| `fl/spielstart.py` | 'Match starten': die Kette von der Karriere bis ins Spiel (docs/intern/MATCHSTART.md, Teil 2). | fl27.rs, live.rs | Bereich |
| `fl/stadion.py` | Stadionangaben der FL27-Welt: Kapazität je Verein aus `sim/config/stadien.json` (nach Vereinsnamen der Welt). | fl27.rs, live.rs | Bereich |
| `fl/taktikdb.py` | Taktik-Dateien der FL27-Datenbank (Tactics.bin, TacticsFormation.bin): Leser, Schreiber der Formationsplätze. Befund und Belege: docs/intern | fl27.rs, live.rs | Bereich |
| `fl/vollzug.py` | Rückkanal PESM -> Welt: Entscheidungen der PESM-Module kommen in die Weltsimulation. | fl27.rs, live.rs | Bereich |
| `fl/vorgaben.py` | Vorgaben für das Freundschaftsspiel in FL27, die PES M vor dem Start in den Sider-Ordner schreibt (kleine Textdateien). | fl27.rs, live.rs | Bereich |
| `fl/zeitung.py` | Zeitung der FL27-Welt: Stoff für PESMs Redaktion (`core/zeitung`) aus der Weltsimulation (`fl/sim`) statt aus dem Master-League-Spielstand. | fl27.rs, live.rs | Bereich |
| `fl/zusatzdatei.py` | Zusatzdatei: Leihen und schon vereinbarte Wechsel, die die Spieldatenbank nicht kennt (siehe docs/intern/ZUSATZDATEI.md). | fl27.rs, live.rs | Bereich |
| `fl/beobachter/__init__.py` |  | — (Paketdatei) | entfällt |
| `fl/beobachter/abgleich.py` | Weg 3: Abgleich von Speicher (Lua-Modul, quelle fl27) und Bild (bild/lauf.py, quelle bild) zu einer Spieldatei. | — | entfällt |
| `fl/beobachter/demo.py` | Erfundene Demo-Welt (ohne Spieldatenbank) für Tests und den Mock-Schalter der Oberfläche: eine Liga mit 10 Vereinen à 20 Spielern, Namen mit | — | entfällt |
| `fl/beobachter/dienst.py` | Spielbeobachter, Linux-Seite im laufenden Programm: Takt alle zwei Sekunden (eigener Hintergrundfaden, nur wenn der Modus nicht 'aus' ist). | — | entfällt |
| `fl/beobachter/kern.py` | Spielbeobachter, Kern ohne Welt: Schema-Prüfung, Aufzeichnung (gzip), Einstellungen, Diagnose, Notenformel mit Gründen. | — | entfällt |
| `fl/beobachter/namen.py` | Namensabgleich: erkannter Text (Texterkennung einer Einblendung) -> bekannter Spieler des Spiels (etwa 25 bis 30 Kandidaten). | — | entfällt |
| `fl/beobachter/speicher.py` | Spielbeobachter, Quelle Speicher: liest die Spielerstatistik der laufenden FL27-Partie direkt aus dem Prozessspeicher (nur lesend). | — | entfällt |
| `fl/beobachter/welt.py` | Spielbeobachter, Anbindung an die FL-World-Weltsimulation (pesm/fl/sim): Zuordnung des beobachteten Spiels zum eigenen Spiel des Tages, | pesdb.rs, world.rs | gebaut |
| `fl/beobachter/bild/__init__.py` |  | — (Paketdatei) | entfällt |
| `fl/beobachter/bild/aufnahme.py` | Bildaufnahme des Spielfensters, rein passiv (nur Bilder holen, keine Eingaben, kein Eingriff ins Spiel). | — | entfällt |
| `fl/beobachter/bild/bilder.py` | Bilder als numpy-Felder (Höhe x Breite x 3, uint8, RGB): PPM lesen (Ausgabe von grim), PNG/JPG über Qt laden und speichern, | fl27.rs | gebaut |
| `fl/beobachter/bild/erkennung.py` | Erkennung auf Einzelbildern nach Anzeigeprofil (profil.py): Ziffern per Schablonenvergleich (nicht Tesseract), Einblendungen per Farbe, | — | entfällt |
| `fl/beobachter/bild/lauf.py` | Bild-Beobachter (Weg 2): wertet eine Folge von Bildern aus (live aus aufnahme.Aufnahme oder aus einer Aufzeichnung) und baut daraus eine | — | entfällt |
| `fl/beobachter/bild/ocr.py` | Texterkennung für Namen in Einblendungen und auf dem Aufstellungsbildschirm. Ziffern laufen nie hierüber (Schablonen, erkennung.py). | — | entfällt |
| `fl/beobachter/bild/profil.py` | Anzeigeprofile (Scoreboards sind Mods mit eigenem Layout): je Profil eine JSON-Datei in bild/profile/ oder im Datenordner | — | entfällt |
| `fl/beobachter/bild/synth.py` | Synthetische Testbilder in der Layoutlogik eines Anzeigeprofils: Spielszene mit Anzeige, Einblendung und Radar, Ergebnisbildschirm, | — | entfällt |
| `fl/sim/__init__.py` | Weltsimulation aus fl-world (Kern: Welt, Spiel, Wettbewerbe, Kalender, Spielstände). | — (Paketdatei) | entfällt |
| `fl/sim/akademie.py` | Akademie und U19 (Modul): ein erfundener Nachwuchspool und die eigene Jugend (ca. 30 Spieler) mit U19-Spielen an jedem Ligaspieltag. | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/comp.py` | Competition engine: league schedule (circle method, double round robin), table with tiebreakers, knockout cup (draw with byes, extra time, p | cups.rs, world.rs | gebaut |
| `fl/sim/config.py` |  | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/ehrungen.py` | Modul Ehrungen: Spieler des Spiels, Elf der Woche (+Dauergast), Monats-, Saison- und Turnierpreise, Weltfußballer-Wahl mit Jury, Meilenstein | ehrungen.rs, ki.rs | gebaut |
| `fl/sim/entwicklung.py` | Entwicklung, Medizin, Rücktritte, Nachwuchs (Modul F). Werte: config/entwicklung.json (CFG['entwicklung']). | world.rs | gebaut |
| `fl/sim/geld.py` | Geld-Schicht: EINE Einheit (ganze Euro, Jahresbeträge) und EINE Kalibrierung nach Vereinsgröße für alle Module. | wirtschaft.rs | gebaut |
| `fl/sim/geschichten.py` | Geschichten, Vereinsruf und Gedächtnis der Welt (eigener Verein). Werte: config/geschichten.json, Texte als Baukasten: config/texte/geschich | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/gloan.py` | Leihgeschäfte der Gaffer-Oberfläche. (a) Einleihen: Klub (Leihgebühr, Kaufoption) -> Spieler (Rolle, Dauer, Treueprämie) -> Abschluss; (b) V | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/gneg.py` | Verhandlungsablauf der Gaffer-Oberfläche (Klub -> Spieler -> Berater -> Abschluss) auf der Verhandlungs-Engine sim/verhandlung.py. | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/grund.py` | Zentraler Begründungsspeicher: JEDES Modul schreibt hier jede Änderung mit Grund hinein (Moral, Stimmung, Form, Werte, Beziehungen, Aussagen | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/gsell.py` | Verkaufsverhandlung der Gaffer-Oberfläche (Seite 'Verkäufe'): Klub (Ablöse nachverhandeln, Marktspiele wait/bluff/leak) -> Spieler (Beteilig | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/kader.py` | Squad guard (module of the main session): every club keeps unique shirt numbers and at least 2 goalkeepers; checks only clubs whose squad ch | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/kalender.py` | Calendar helpers and international duty. Day 0 = 1 August of the season's first year (2026-08-01, a Saturday). | date.rs | gebaut |
| `fl/sim/markt.py` | Markt-Ansichten und -Aktionen für den eigenen Verein (keine Hooks, kein Zustand außer w.ext['transfers']['probed'/'rumor_fx']). | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/match.py` | Match simulator. Strength of the XI -> Dixon-Coles goals -> scorers, cards, injuries, minutes, ratings, and a reason in football language. | cups.rs, game.rs, world.rs | gebaut |
| `fl/sim/namen.py` | Schöne Wettbewerbsnamen ohne Oberfläche: schoen('ENGLAND_D1_LEAGUE') -> 'Premier League'; ersetze(text) wandelt rohe Namen in Texten um. Map | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/psyche.py` | Psyche des eigenen Vereins (Modul): Wesenszüge, Stimmung als sichtbare Posten, Trainerphilosophie, Berater, Zusagen mit Frist, Beziehungen,  | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/realdaten.py` | Nur-Lese-Lader der Echtdaten bis Sommer 2026 (config/realdaten/*.json, gebaut von tools/realdaten/bau.py). Nichts davon wird in einen Spiels | game.rs, realdaten.rs | gebaut |
| `fl/sim/ruf.py` | Ruf (Reputation) 0..100 als zentrale Währung der Welt: Vereine und Ligen (gespeichert), Spieler (aus Stärke und Ehrungen, berechnet). Werte: | world.rs | gebaut |
| `fl/sim/sponsoren.py` | Modul Sponsoren und TV-Rechte (Markenwelt, Marken aus core/marken/marken.toml). Beträge sind Euro je Jahr, bezogen auf die Handelseinnahmen  | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/stab.py` | Stab und Scouting des eigenen Vereins (Modul). Werte: config/stab.json. Zustand: w.ext['stab']. | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/store.py` | Save games: one SQLite file per slot. save() dumps the state, load() rebuilds World (tables, levels, tired list are derived). Autosave and u | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/szenarien.py` | Szenarien, Medien, Gespräche, Fan- und Vorstandsstimmung (Modul G). Alle Texte und Zahlen: config/szenarien.json (Vorlagen mit Auslöser, Wah | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/termine.py` | Terminplan der Zukunft: was schon vereinbart ist und an einem Stichtag geschieht (Vorvertrag, ablösefreier Wechsel zum 1.7., Kaufpflicht, ve | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/texte.py` | Textbaukasten mit Varianz: mehrere Formulierungen je Situation, austauschbare Satzteile, Ton nach Persönlichkeit, Fakten eingesetzt, kürzlic | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/training.py` | Trainingspläne (Modul TR). Werte: config/training.json (CFG['training']). Nicht in SYSTEMS eingetragen: das macht die Hauptsitzung (nach 'en | world.rs, cups.rs, ligaregeln.rs | Bereich |
| `fl/sim/transfers.py` | Module E: contracts, market value, club finances, AI transfers, managers (extension of the world, see docs/intern/WELT-ERWEITERUNGEN.md). Al | game.rs, transfer.rs, world.rs | gebaut |
| `fl/sim/uefa.py` | Internationale Wettbewerbe: Champions League, Europa League, Conference League (+ optional Supercup). Einstellungen: config/uefa.json. | cups.rs, pesdb.rs | gebaut |
| `fl/sim/vdialog.py` | Verhandlungsdialog der Weltseite: setzt das Rundenmodell (pesm/core/verhandlung/runden.py) auf die Engine in verhandlung.py. Siehe docs/inte | game.rs | gebaut |
| `fl/sim/verein.py` | Modul Verein: Vereinsprofil, Ehrentafel/Chronik, Rivalitäten und Derbys, Auszeichnungen, Kapitän und Mannschaftsrat. Alle Werte und Textbaus | game.rs | gebaut |
| `fl/sim/verhandlung.py` | Verhandlungen des eigenen Vereins (Modul): erst mit dem abgebenden Klub (Ablöse, Raten, Boni, Weiterverkaufsbeteiligung, Tausch; bei Leihen  | game.rs, transfer.rs | gebaut |
| `fl/sim/welt.py` | World state and time loop. One World = one save game; runs without the game and without the UI. See docs/intern/SCHNITTSTELLE-WELT.md. | pesdb.rs, world.rs | gebaut |
| `tools/__init__.py` |  | — (Paketdatei) | entfällt |
| `tools/abfrage.py` | Kleines Abfrage-Werkzeug für das Backend (nur lesen). | tools/ | Bereich |
| `tools/funktionen.py` | Erzeugt docs/FUNKTIONEN.md aus registry/menus.yaml (Bauauftrag §7). | tools/ | Bereich |
| `tools/kontrast.py` | Kontrastprüfung der Paletten (WCAG 2.1). Aufruf: python -m pesm.tools.kontrast | tools/ | Bereich |
| `tools/probleme.py` | Problemliste von FL-World am Terminal (für Anwender und eine mitlesende Claude-Sitzung, docs/intern/WAECHTER.md). | tools/ | Bereich |
| `ui/__init__.py` |  | — (Paketdatei) | entfällt |

**Zahlen:** Bereich 236, entfällt 165, gebaut 94, zusammen 495.
