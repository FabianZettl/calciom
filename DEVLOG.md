# Dev log

Short notes on how things work under the hood, newest first. Player-facing changes are in [CHANGELOG.md](CHANGELOG.md).

## 26.11.2026 – CalcioM 0.6.0

- **Kit choice.** `src/trikots.rs` reads the kitserver's `content/kits/map.txt` and order lists, decodes the `.ftex` atlases (BGRA8, BC2, BC3, BC7) and draws a 2D kit. The numbers go into `flw/direktmatch.txt`; module 1.28 sets them again for 15 s after FL27's match day screen resets them. Module 1.29 skips `Edit/Load` (`edit=0`), so EDIT00000000 cannot overwrite the career database.
- **Match reports.** `World.berichte` keeps a `Spielbericht` for every club match of this and last season. Matches known only by their score get one from `bericht_simulieren`, seeded by the match, so it is always the same.
- **Talks.** Each save has its own salt for the daily rolls (`Career.salz`). `pool_tag` scales the frequency by ego, temper, ambition and mood. `Career::spieldaten` gives the scenarios their facts from the last five own reports.
- **Working hints.** `save_hintergrund` writes in a thread (one at a time, `SCHREIBEN` lock); a career loads over a channel. Pages slower than 0.3 s (`LANGSAM_S`) draw "Loading …" first.
- **Data pack.** `src/pakete.rs` loads `assets/pakete/echte-daten` (manifest `calciom-paket.toml`) over neutral stand-ins; its rule files merge over the built-in ones.
- **sider.ini after the match.** `fl27::sider_ini_ohne_calciom` comments out the career root, the direct-start root and CalcioM's Lua modules once FL27 has closed.
- **sider.ini and add-ons.** `Fl27Start` switched off every other livecpk root with a `common/etc/pesdb` folder, so boot, commentary, cup and stadium packs went dark. Now only roots with a `Player.bin` (a whole world) are switched off. `fl27::zusaetze_wieder_an` turns an add-on root back on when it was active in the career's `original/sider.ini`.
- **Stadium check.** `Career::stadion_warnung` looks for the home team's line in `map_teams.txt`, its folder under `content/stadiums`, and `pesm_stadion` in `stadiums.lua`, and shows a toast at match start.

## 26.11.2026 – CalcioM 0.5.3 (hotfix)

- **Read from cpk.** Each database file (Player.bin, Competition.bin …) is taken from the `.cpk` that contains it, instead of all from the one with the biggest Player.bin: an FL27 update package like `data_s26272.cpk` has players but no competitions.

## 26.11.2026 – CalcioM 0.5.2 (hotfix)

- **Broken-off matches.** The reader (`fl27::Mitlauf`) notes when FL27's process ends while it follows a match (`abgebrochen`). The figures of that match come as `Meldung::Abgebrochen`, and the matchday page asks whether to count or discard them. A whistled match (FL27 alive) is booked as before.
- **Read from cpk.** An incomplete world entry in `sider.ini` (for example a partial `_FL26MyWorld`) is skipped, the `.cpk` files are read instead, and there is no endless retry.
- **Friendlies** are created with `selbst: true` and go straight onto the match calendar (`testspiel_neu`).
- **AI pass photos** (optional): stable-diffusion.cpp paints a pass photo from the drawn one, written as a 128×128 DXT5 portrait into the career and its livecpk.

## 26.11.2026 – CalcioM 0.5.0

- **Rename.** `tools/calciom_umbenennen.py` renames every tracked file and name. The hand-written part (`src/marke.rs`, `fl27::umzug`, `fl27::alte_namen_weg`) moves the old data folders and switches off the old direct-start lines in `sider.ini`. Details: [docs/CALCIOM.md](docs/CALCIOM.md).
- **Direct start without game files.**
  - `fl27::direktmatch_einrichten` reads the flow files from the player's `Data/dt16_all.cpk`.
  - `fl27::flow_umleiten` changes only the `to` of the listed events and checks, as JSON, that nothing else changed.
  - The Sider module `calciom_direktmatch.lua` loads them only in a direct start.
- **Quick start.**
  - With `schnell=1` in `flw/direktmatch.txt` the module routes the end of the match back to the side selection.
  - When the side selection is left, it re-reads `direktmatch.txt` and calls the LivePatch reload (`pesm_livepatch_laden`).
  - "Start match in FL27" with FL27 open writes the next line and the patches, then brings the window to the front (Hyprland, X11 or Windows).
- **Restart check.**
  - At FL27's start, CalcioM keeps a copy of the squads the game loaded (`start_asg.bin`).
  - A restart is only asked for when one of the two match teams differs from it.
- **Master League import.** `src/mlimport.rs` reads an FL27 ML save, read only. It can also run from the command line: `calciom --import-ml <save> [<world>]`.
