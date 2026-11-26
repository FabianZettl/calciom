# Getting started with CalcioM

This guide takes you from the first launch of CalcioM to a running career, and then to your first match played in Football Life 2027 (FL27).

> [!NOTE]
> CalcioM 0.5 is the first stable release. Linux is the tested platform. **Windows has not been tested on a real Windows PC yet**, so every Windows step below comes from the code and may be wrong. If something does not match, please tell us on Discord (link at the end).

## What you need

- **Football Life 2027** (SP Football Life 2027) installed and working, with **Sider** (the `SiderAddons` folder inside the game folder).
- **CalcioM** from the [Releases page](https://github.com/FabianZettl/calciom/releases):
  - Windows: `calciom-<version>-windows-x86_64.zip`
  - Linux: `calciom-<version>-linux-x86_64.tar.gz`
- Linux only: FL27 installed through **Lutris** (Wine/Proton), and the tool `setcap` (package `libcap` on most distributions).

CalcioM never changes your original FL27 database. Each career gets its own copy of the database in its own LiveCPK folder.

---

## 1. Install CalcioM

### Windows

1. Unzip `calciom-<version>-windows-x86_64.zip` into a normal folder, for example `C:\CalcioM`. Keep `calciom.exe` and the `assets` folder together.
2. Do **not** put it in `C:\Program Files` (Windows protects that folder, see [os error 5](#os-error-5-access-denied-windows)).
3. Double-click `calciom.exe`.

### Linux

1. Unpack the archive into your home folder, for example:
   ```sh
   tar xzf calciom-<version>-linux-x86_64.tar.gz -C ~/
   cd ~/calciom-<version>-linux-x86_64
   ```
2. Allow the small helper `calciom-speicher` to **read** FL27's memory. CalcioM needs this to get the result and stats of a match you play in FL27. Run this once:
   ```sh
   sudo setcap cap_sys_ptrace+ep ~/calciom-<version>-linux-x86_64/calciom-speicher
   ```
   - Do this again after **every update**, because a new file loses the permission.
   - It only works on a normal Linux file system (ext4, btrfs …), not on an NTFS or exFAT drive.
   - The helper only reads. It never writes into the game.
   - Optional: you can give the same permission to `calciom` itself (`sudo setcap cap_sys_ptrace+ep .../calciom`). Then CalcioM reads the memory directly.
3. Start CalcioM with `./calciom.sh` (or `./calciom`).

You can check the memory access in CalcioM under **Settings › Football Life 2027**. The line "Memory helper" shows whether the helper is there, and the exact `setcap` command for your folder.

---

## 2. Choose your FL27 world (step "1 World")

On the start screen, click **New career**. The first step is **1 World**.

CalcioM looks for your game folder by itself:

- **Windows:** it looks for `FL_2027.exe` next to CalcioM and on every drive, in places like `X:\SP Football Life 2027`, `X:\Games\SP Football Life 2027` and `X:\Program Files (x86)\SP Football Life 2027`.
- **Linux:** it only knows the author's own Lutris prefix. On your PC it will most likely **not** find the game by itself, so pick the folder by hand (see below).

If no world is shown, click **Browse …** and choose your **`SP Football Life 2027`** folder (the one that contains `SiderAddons`). You can also paste the path with Ctrl+V into the field and click **Check**. Typical paths:

| | Game folder |
|---|---|
| Windows | `C:\Games\SP Football Life 2027` (or wherever you installed it) |
| Linux (Lutris) | `<your Wine prefix>/drive_c/Games/SP Football Life 2027` |

You may also pick `SiderAddons`, `SiderAddons\livecpk` or a world folder inside it. CalcioM finds the game folder from there and remembers it in `spielordner.txt` (see [Where CalcioM keeps its files](#where-calciom-keeps-its-files)).

### Which database? Extracted world or `.cpk` mode

CalcioM reads the FL27 database (the "pesdb") in one of two ways:

- **Extracted world (best):** a mod world in `SiderAddons\livecpk\<world>` that contains the unpacked database in `common\etc\pesdb` (for example a Megapack world). CalcioM shows every world it finds as a button. Click the one you play with.
- **`.cpk` mode:** if you have no extracted world, click **"No extracted world: read from the FL27 .cpk files"**. CalcioM reads the game's own database straight from the `.cpk` files of the game folder. This takes a moment. It saves the result as the folder `SiderAddons\livecpk\FL27-Datenbank aus CPK`. That folder is **not** added to `sider.ini`, so the game itself never loads it.

When the world is loaded, CalcioM shows how many leagues, clubs and players it found. Click **NEXT**.

> Without FL27 you can try CalcioM with the **Built-in test world** (real clubs, made-up player names). You cannot play its matches in FL27.

## 3. Choose your club, manager and start

- **2 Club:** search for a club, league or country, or filter by country and league. Click a club, then go on.
- **3 Manager:** enter your manager's name (and optionally a photo, a PNG file).
- **4 Start:** check everything and start the career.

When the career starts, CalcioM copies the world into your career folder. The original world stays untouched. With a big world (for example a Megapack with tens of thousands of files) this can take a few minutes. A progress bar shows the steps.

### Optional: start from a Master League save

In step "1 World", click **Import ML save…** and choose a Master League save of FL27 (`ML00000001` … in `Documents\SPGAME\Football Life\2027\save`). CalcioM shows a preview (club, date, coach, league position) and picks the world that knows the save's players. **Start from this save** takes over the date, every club's squad, contracts and wages, 15 of the 25 abilities (the other 10 follow the same change), fitness, form, injuries, league tables and fixtures, the coaches and your transfer budget. Cups and Europe are played by CalcioM up to that day. The save is only read, never changed.

### Optional: real contract data from SoFIFA

At the start of a career, CalcioM can take over real contract data (potential, wage, market value, contract end, release clause, loans) from a **German SoFIFA export** of the FCFL Stats Converter (a `.csv` file). CalcioM only reads the files. It looks in these folders and uses the newest full export:

| | Folder |
|---|---|
| Windows | `%APPDATA%\calciom\sofifa\` , `%APPDATA%\FL-Stats-Converter\ea-base\` , `%APPDATA%\FL-Stats-Converter\ea-updates\` |
| Linux | `~/.local/share/calciom/sofifa/` , `~/.local/share/fl-stats-converter/ea-base/` , `~/.local/share/fl-stats-converter/ea-updates/` |

Put the file there **before** you start the career. Without it, CalcioM estimates these values itself.

---

## 4. Play a match in FL27

On match day you can **Simulate** the match in CalcioM, or **Play (FL27)** and play it yourself in Football Life 2027.

### The first match of a career (FL27 closed)

1. **Close FL27** if it is running.
2. On the match day page choose **Play (FL27)**.
3. The first time, click **Allow match launch**, then read the note and click **Understood, don't ask again**. (If you said yes to direct start in the first-start question, CalcioM has done this for you and you go straight to step 4.)
4. Click **Start match in FL27**.

CalcioM now does this, step by step:

- It saves your career and backs up the original world (once).
- It writes your career database (squads, transfers, **both line-ups**) into the career's own LiveCPK folder: `SiderAddons\livecpk\CalcioM<Club><Number>`, for example `CalcioMArsenalFC01` (careers started in PES M keep their `PESM…` folder).
- It adds that folder as a `cpk.root` line to `SiderAddons\sider.ini`, and switches off other worlds that would cover it.
- It installs the Sider module **CalcioM-LivePatch** (`SiderAddons\modules\pesm-livepatch.lua`, the file keeps its old name) and adds its `lua.module` line to `sider.ini`.
- It writes the match settings into `SiderAddons\flw\`: `wettbewerb.txt` (competition look: scoreboard, walk-out, ball), `stadion.txt` (home stadium), `wetter.txt` (weather), `spielregeln.txt`.
- It starts FL27:
  - **Windows:** with `FL 2027 start.exe` from the game folder. If that is missing, with `sider.exe` and then `FL_2027.exe`. Windows may ask for admin rights (UAC), click **Yes**. *(untested)*
  - **Linux:** through Lutris. **Known limit:** CalcioM starts Lutris game number 7 (the author's install). If FL27 does not open, or a different game opens, start FL27 yourself in Lutris. CalcioM waits up to 4 minutes for it.

Then, in FL27 (CalcioM shows these steps on screen too):

1. In the main menu choose **Kick Off** and move your controller to the side CalcioM tells you (**Home** or **Away**).
2. Choose the teams CalcioM names (home team and away team).
3. Leave the stadium as it is. CalcioM has set it.
4. If you like, set the tactics from CalcioM's **tactics note** in FL27's own tactics menu.
5. Kick off and play.

After the final whistle, CalcioM reads the score and the real stats (goals, assists, shots, passes, cards, saves …) from the game's memory and rates your players. If CalcioM misses the final whistle, click **Confirm result**: it books the score as it stands. If the reading does not work at all, you can always enter the result by hand.

Tip: switch on **Settings › Football Life 2027 › "Quit FL27 after full time"** if you want CalcioM to close the game after each match.

### Direct start and quick start

**The first time you open CalcioM 0.5** it asks once: *"Play your matches with direct start?"*

- **Yes, switch it on:** CalcioM sets everything up right away, on Linux and Windows. It reads the game's own flow files from `Data\dt16_all.cpk`, changes only a few targets and writes them to `SiderAddons\content\calciom-direktmatch\flow`. It also installs the Sider module `calciom_direktmatch.lua` and its `sider.ini` line, and allows the match launch. **Quick start** is switched on too: FL27 stays open between matches. If you have GigaPack's `modules\fl26match.lua`, CalcioM comments out its statistics file call (`fl26_match_statsfile`): its background thread makes FL27 crash at random, mostly at half-time or when a match ends. The mod keeps working; the original is kept as `fl26match.lua.pesm-original` and is put back when you switch direct start off. A short guide follows. New careers get the same answer.
- **No, keep the menus:** nothing changes, FL27 starts with its normal menus.

You can change both any time under **Settings › Football Life 2027** (*Direct start* and *Quick start: FL27 stays open*). CalcioM ships no game files for this.

**How a match goes:**

1. On match day click **Start match in FL27**. Set the **match length** right below it.
2. FL27 starts and goes straight to the **side selection**: move your controller left or right and confirm.
3. The match day screen comes up with both teams and kits. You press **Kick-off** yourself.
4. At **half-time** the menu stays: substitutions and tactics as usual.
5. After the final whistle FL27 skips the stats and result screens and CalcioM books the result. With quick start, FL27 waits at the side selection.
6. Play on in CalcioM. On the next match day click **Start match in FL27** again: CalcioM writes the next match, your line-up and attribute changes, stadium and weather into the waiting game and brings the FL27 window to the front. You only confirm your side.

**New or transferred players** in one of the two teams can only be loaded by an FL27 restart. CalcioM then shows a note with **Start anyway**: the match starts, but transferred players may be missing. This always happens during or after a transfer window; restarting FL27 once fixes it.

Without direct start, FL27 can stay open too: in FL27 open the Sider overlay (**Space**), pick the module **CalcioM-LivePatch** (key **1**), press **F7**, close the overlay and start the match.

Not there yet: the entrance animation before kick-off. Direct start is tested on Linux; Windows is not tested yet.

### Undo everything

**Settings › Football Life 2027 › Uninstall (career from FL27)** removes the career from the game folder and restores what CalcioM changed. CalcioM backs up `sider.ini` and the original files before it changes them.

---

## Upgrading from PES M

CalcioM is the new name of PES M. Unpack CalcioM **into the same folder** as your old PES M (or anywhere else) and start `calciom` / `calciom.exe`. At the first start it moves your data folder to the new name (`~/.local/share/pesm-rust` → `~/.local/share/calciom`, `%APPDATA%\pesm` → `%APPDATA%\calciom`), so your careers, settings and chosen game folder are all there. Old careers keep their `PESM…` folder in the game and load as before. The old direct start files in the game folder are replaced at the next match start. On Linux run the `setcap` command again for `calciom-speicher`.

Old PES M versions won't show this update. Please download CalcioM from the [Releases page](https://github.com/FabianZettl/calciom/releases).

## Where CalcioM keeps its files

| What | Windows | Linux |
|---|---|---|
| Log file | `%APPDATA%\calciom\calciom.log` | `~/.local/share/calciom/calciom.log` |
| Chosen game folder | `%APPDATA%\calciom\spielordner.txt` | `~/.local/share/calciom/spielordner.txt` |
| SoFIFA exports | `%APPDATA%\calciom\sofifa\` | `~/.local/share/calciom/sofifa/` |
| Careers (saves, career database, world backup) | see note below *(untested)* | `~/.local/share/calciom/karrieren/` |
| Career in the game | `<game folder>\SiderAddons\livecpk\CalcioM<Club><Number>` (old careers: `PESM…`) | same, inside the Wine prefix |
| Match settings for FL27 | `<game folder>\SiderAddons\flw\` | same |

Inside `karrieren` each career has a save file `<name>.json` and a folder `<name>` with the career database (`pesdb`) and the world backup (`original`). Older saves go to `karrieren/sicherungen/`, deleted careers to `karrieren/papierkorb/`.

> **Windows careers folder (untested):** CalcioM builds this path from the `HOME` variable, which Windows usually does not set. Then the careers end up in `.local\share\calciom\karrieren` inside the folder CalcioM was started from (usually next to `calciom.exe`). Look there first if you need your saves.

`%APPDATA%` is usually `C:\Users\<you>\AppData\Roaming`. Type `%APPDATA%\calciom` into the Explorer address bar to open it.

---

## Troubleshooting

**Bug report:** **Settings › Save & career › Diagnostics for bug reports** packs `calciom.log`, `sider.log`, `sider.ini` and the versions into one archive next to the log (your user name is replaced, no save goes in). Please attach it.

### "os error 5" (access denied, Windows)

Windows did not let CalcioM write a file. Usual reasons:

- **The file is in use.** Close FL27 (and Sider) and try again. If two CalcioM windows are open, close one.
- **The folder is protected.** CalcioM or FL27 is inside `C:\Program Files` or `C:\Program Files (x86)`. Move CalcioM to a normal folder like `C:\CalcioM`. If FL27 is there, start CalcioM **as administrator** (right-click `calciom.exe` › *Run as administrator*).
- **FL27 runs as administrator, CalcioM does not.** Then CalcioM may also not be able to read the match result. Start CalcioM as administrator too.
- An antivirus program blocks the file. Allow CalcioM's folder.

### No world found / "This folder is missing the file …"

You picked a folder that is not an FL27 world. Pick the `SP Football Life 2027` folder (it must contain `SiderAddons\livecpk`). If you have no extracted world, use the `.cpk` mode button.

### FL27 does not start, or "FL27 didn't start within 240 s"

- Windows: check that `FL 2027 start.exe` or `sider.exe` and `FL_2027.exe` are in the game folder.
- Linux: start FL27 yourself in Lutris (see the known limit above).
- Or enter the result by hand.

### The match result does not come back (Linux)

Run the `setcap` command from [step 1](#linux) again (after every update) and check **Settings › Football Life 2027 › Memory helper**.

### My changes (line-up, transfers) are not in FL27

- Another world may be above your career in `sider.ini`. Start the match from CalcioM with FL27 closed: CalcioM puts the career line in the right place.
- With FL27 open, you have to press **F7** in the CalcioM-LivePatch module. New players need a restart.

### FL27 says "FL27 runs another career or without the module CalcioM-LivePatch"

Quit FL27 and click **Start match in FL27** again.

---

## Help and feedback

Questions, bugs, ideas: join us on Discord: **https://discord.gg/ACW3wF2MqW**
