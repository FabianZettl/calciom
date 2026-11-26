# Dev log

Short notes on how things work under the hood, newest first. Player-facing changes are in [CHANGELOG.md](CHANGELOG.md).

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
