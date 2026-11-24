# Pokale und Ligen: was am 24.11.2026 korrigiert wurde

Zweig `welt/pokalregeln` (Worktree `~/Projects/pesm-rust-pokal`), Commits 3e7b712, 75153cc, e0aa150.
Noch nicht in `main` gemergt, weil dort gerade eine andere Sitzung ungespeicherte Änderungen hat.
Alle Tests grün (247 + 26). Regeln: `daten/pokalregeln.toml`. Amateurvereine: `daten/amateure.toml`.

## Für alle Pokale
- Es spielen alle Erstteams aller Ligen des Landes mit, nicht mehr nur die Liste aus der Datenbank (vorher: DFB-Pokal nur 18 Bundesligisten).
- Fehlende Ligen werden mit Amateurvereinen mit echten Namen aufgefüllt (Wikipedia, Saison 2025/26): 1324 Vereine aus 10 Ländern. Sie haben nur Name, Liga-Stufe und Stärke. Sie tauchen nicht im Transfermarkt, in Spielerlisten, Tabellen oder in der Ehrentafel auf.
- Spiele gegen Amateurvereine werden nur berechnet, auch die des eigenen Vereins. Sie laufen nie in FL27 und stehen deshalb nicht als eigener Spieltag im Kalender. **Änderbar.**
- Qualifikationsrunden nur unter Amateuren laufen nebenbei.
- Freilose für die Stärksten, wenn die Zahl nicht aufgeht. Die Runden vor dem Finale werden vom Ende her gezählt, damit eine Zusatzrunde das Halbfinale nicht verschiebt.

## Je Land
- **England, FA Cup:** 747 Vereine, 6 Qualifikationsrunden (Stufe 5 bis 9), League One und League Two in der 1. Hauptrunde, die oberen zwei Ligen in der 3. Hauptrunde. Keine Wiederholungsspiele.
- **England, EFL Cup:** neu, 92 Vereine. Europapokal-Teams steigen erst in Runde 3 ein. Bis zum Finale ohne Verlängerung, Halbfinale mit Hin- und Rückspiel, der Sieger kommt in die Conference League.
- **Deutschland, DFB-Pokal:** 64 Vereine (18 + 18 + die ersten 4 der 3. Liga + 24 Landespokalsieger). Etwa ein Viertel der Landespokalsieger sind Drittligisten, der Rest kommt aus der Regionalliga oder Oberliga. In Runde 1 gibt es zwei Töpfe, der Tiefere hat Heimrecht. **Die Landespokale sind nicht nach Region verteilt, änderbar.**
- **Deutschland, Regionalliga** (Ligaregel): 5 Staffeln. West und Südwest steigen direkt auf. Von Nord, Nordost und Bayern steigt reihum einer direkt auf (2025/26 Nord, 2026/27 Nordost, 2027/28 Bayern), die anderen zwei spielen ein Aufstiegsspiel. Absteiger je Staffel nach 2025/26 (Bayern mit Relegation gegen die Bayernliga). Die Welt hat keine Regionalliga, deshalb gilt das nur als Daten.
- **Spanien:** Copa del Rey mit 115 Vereinen. Die 4 Supercopa-Teilnehmer steigen im Sechzehntelfinale gegen die tiefsten Vereine ein, Halbfinale mit Hin- und Rückspiel. Die Supercopa wird mit 4 Vereinen gespielt (Halbfinale und Finale).
- **Italien:** Coppa Italia mit 44 Vereinen (4 aus der Serie C neu). Bis zum Viertelfinale ohne Verlängerung, die Besten steigen später ein, Halbfinale mit Hin- und Rückspiel. Die Supercoppa wird mit 4 Vereinen gespielt.
- **Frankreich:** Zweiunddreißigstelfinale mit 64 Vereinen (Ligue 1, Ligue 2, National und 10 Amateure). Bis zum Finale ohne Verlängerung (**änderbar**), der Tiefere hat Heimrecht.
- **Niederlande:** KNVB-Beker mit 110 Vereinen, 2 Amateur-Vorrunden, Europapokal-Teams erst ab Runde 2. Der Name fehlte bisher für Supercup und Finalort, das ist korrigiert.
- **Portugal:** Taça mit 106 Vereinen, die Erstligisten steigen in Runde 3 ein, Halbfinale mit Hin- und Rückspiel.
- **Belgien:** 67 Vereine, die Pro League steigt im Sechzehntelfinale ein, Halbfinale mit Hin- und Rückspiel. Keinen Supercup mehr (seit 2024 nicht gespielt, **änderbar**).
- **Türkei:** 117 Vereine, Gruppenphase mit 3 Gruppen und je 4 Spielen. Weiter kommen die besten zwei jeder Gruppe und die zwei besten Dritten. Europapokal-Teams erst ab der Gruppenphase. Supercup mit 4 Vereinen. TFF 1. Lig: Play-offs jetzt mit den Plätzen 3 bis 7 (vorher 3 bis 6).
- **Schottland:** 102 Vereine, die Premiership steigt in Runde 4 ein. Keinen Supercup (gibt es nicht).
- **Übrige Länder:** alle Erstteams aller Ligen des Landes, Einzelspiele mit Verlängerung und Elfmeterschießen. Ohne Amateurvereine, weil es keine Listen gibt (**änderbar**).
- **Europa:** Die Conference League hat jetzt 6 Ligaphasen-Spiele (6 Töpfe) statt 8.

## Geprüft, ohne Änderung
England (alle 6 Stufen), Bundesliga bis 3. Liga, Spanien, Italien, Frankreich, Niederlande, Portugal, Belgien, Schottland, Österreich, Schweiz, Dänemark, Griechenland, Norwegen, Schweden, Kroatien, Russland, Japan, Saudi-Arabien, Brasilien, Argentinien, USA und Mexiko stimmen mit 2025/26 überein. Die übrigen der rund 340 Einträge habe ich nicht einzeln neu nachgeschlagen; sie tragen weiter ihr `sicher`-Kennzeichen.

## Discord

**#dev-log**
> National cups now work like the real ones. Every first team of a country's leagues plays, not just the database list. Missing tiers are filled with 1,324 amateur clubs under their real 2025-26 names (all 680 English non-league and League Two clubs in the FA Cup, Regionalliga clubs in the DFB-Pokal, RFEF clubs in the Copa del Rey …). Amateur clubs exist only in the cups: no players, no market, no honours, and their ties are simulated. Rules per cup live in `daten/pokalregeln.toml`: entry round per tier, qualifying rounds among amateurs, byes, home advantage for the lower club, two-legged semi-finals, penalties without extra time, two pots in DFB-Pokal round 1 and the Turkish group stage. New: the EFL Cup (winner to the Conference League), four-club supercups in Spain, Italy and Turkey, and the Conference League with six league-phase matches. League rules: Regionalliga per group with the promotion rotation, TFF 1. Lig play-offs 3–7.

**#changelog**
> - FA Cup with all 747 English clubs, from the extra preliminary round to Wembley
> - DFB-Pokal with 64 clubs: Bundesliga, 2. Bundesliga, the top four of the 3. Liga and 24 regional cup winners down to the Oberliga
> - EFL Cup added; Copa del Rey, Coppa Italia, Coupe de France, KNVB Beker, Taça, Belgian, Turkish and Scottish Cup follow their real formats
> - Supercopa, Supercoppa and Turkish Super Cup with four clubs; no Scottish or Belgian supercup any more
> - Conference League: six league-phase matches
> - Ties against amateur clubs are simulated, never played in FL27
