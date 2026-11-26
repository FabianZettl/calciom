# PES M → CalcioM

On 26.11.2026 PES M was renamed to **CalcioM** (long form: Calcio Management), so the game carries no Konami trademark.

## How the rename is made

1. The hand-written part (one commit): the brand (`src/marke.rs`, `assets/marke/`, colours in `src/ui.rs`), the
   take-over of old names (`fl27::umzug`, `fl27::alte_namen_weg`, `fl27::eigener_cpk`, `fl27::von_uns`) and their test.
2. The mechanical part: `python3 tools/calciom_umbenennen.py` (`--pruefen` only lists). It renames files and replaces
   names in every tracked text file. It can run again on a newer main; a second run changes nothing.
3. Hand-written follow-ups on top of the renamed code (colours of the start screen, reading the old folders in place).

Merging onto a newer main: rebase step 1, run the script (then `python3 -I tools/kennungen.py`), cherry-pick the
step 3 commits, `cargo test`.

## A PES M player switches to CalcioM

Copy CalcioM into the same folder as PES M (or anywhere) and start it. Nothing has to be copied by hand:

- At the first start the old folders are **moved** (renamed, no copy, also for 17 GB), before anything is written.
- If the move fails (Windows: a file in it still open, e.g. PES M still running), CalcioM simply reads and writes the
  old folder where it is (`fl27::neu_oder_alt`) and tries the move again at the next start.
- Linux leaves the old name as a link, so an older PES M build still finds everything. Windows has no link: an old
  `pesm.exe` started afterwards would begin empty, so remove it after the switch.
- Old careers (`PESM<Verein>01.json` and folders) keep their names and load unchanged; new careers are `CalcioM…`.
- Settings files (language, theme, scale, sound, music, game folder, direct-start answer, phone pairing) and the
  `sofifa/` exports move with the folder; `pesm.log` becomes `calciom.log` and is continued.
- In the game folder the old direct start is switched off and removed at the next match start, the new one set up
  (backup `sider.ini.bak-calciom-…`); the old career line stays active, other old careers stay switched off.
- Update notice: it still asks `FabianZettl/pesm` and follows GitHub's forward (`curl -L`) after the repository is
  renamed. Old PES M builds do not follow the forward, so they see no notice once the repository is renamed.

Checked 26.11.2026 with copies of real data (Liverpool career, settings, sofifa, log, cache), once in the Linux layout
and once in the Windows layout (`%APPDATA%\pesm`, `%LOCALAPPDATA%\pesm`, careers next to the program): folders moved,
career listed and loaded, log continued. Repeatable with the ignored tests:
`CALCIOM_SPIELSTAND=<copy of PESM….json> cargo test alter_spielstand -- --ignored` and
`CALCIOM_SIDER_KOPIE=<copy of SiderAddons, Data linked next to it> cargo test echte_sider_kopie -- --ignored`.

## What a player already has, and what happens to it

| Old | New | How |
| --- | --- | --- |
| `~/.local/share/pesm-rust` | `~/.local/share/calciom` | moved at the first start; on Linux the old name stays as a link |
| `%APPDATA%\pesm` | `%APPDATA%\calciom` | moved at the first start |
| `pesm.log` | `calciom.log` | renamed at the first start |
| picture cache `…/pesm/netzbilder` | `…/calciom/netzbilder` | moved at the first start |
| `livecpk/PESM_DirectMatch`, `modules/pesm_direktmatch.lua`, `content/pesm-direktmatch` | `CalcioM_DirectMatch`, `calciom_direktmatch.lua`, `calciom-direktmatch` | old ones removed, their sider.ini lines switched off with `;` (backup next to sider.ini), new ones set up right after, at the next match start |
| career folders and saves `PESM<Verein>01` | new careers: `CalcioM<Verein>01` | old careers keep their name; both count as ours |
| Lua additions marked `PES M` in other mods | marked `CalcioM` | both markers are recognised, nothing is added twice |
| environment switches `PESM_*` | `CALCIOM_*` | developer and test switches only |
| programs `pesm`, `pesm-speicher` | `calciom`, `calciom-speicher` | `sudo setcap cap_sys_ptrace+ep target/release/calciom-speicher` once; the menu launcher must point to `target/release/calciom` |

## Kept on purpose

- `content/pesm-livepatch` and `pesm-livepatch.lua`: the FL Stats Converter writes there too; rename both together.
- `fl26match.lua.pesm-original`, `*.txt.vor-pesm`: backups made by older versions are restored from these names.
- the phone cookie `pesm_geraet` (paired phones stay paired).
- `assets/inhalt/pesm/**` and Lua names like `pesm_stadion`: internal, never shown.
- the GitHub repositories `FabianZettl/pesm` and `pesm-coaching-zone` (renamed later; GitHub forwards old links).
