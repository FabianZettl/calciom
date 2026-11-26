# Getting started with PES M

This guide takes you from the first launch of PES M to a running career, and then to your first match played in Football Life 2027 (FL27).

> [!NOTE]
> PES M is a beta. Linux is the tested platform. **Windows has not been tested on a real Windows PC yet**, so every Windows step below comes from the code and may be wrong. If something does not match, please tell us on Discord (link at the end).

## What you need

- **Football Life 2027** (SP Football Life 2027) installed and working, with **Sider** (the `SiderAddons` folder inside the game folder).
- **PES M** from the [Releases page](https://github.com/FabianZettl/pesm/releases):
  - Windows: `pesm-<version>-windows-x86_64.zip`
  - Linux: `pesm-<version>-linux-x86_64.tar.gz`
- Linux only: FL27 installed through **Lutris** (Wine/Proton), and the tool `setcap` (package `libcap` on most distributions).

PES M never changes your original FL27 database. Each career gets its own copy of the database in its own LiveCPK folder.

---

## 1. Install PES M

### Windows

1. Unzip `pesm-<version>-windows-x86_64.zip` into a normal folder, for example `C:\PESM`. Keep `pesm.exe` and the `assets` folder together.
2. Do **not** put it in `C:\Program Files` (Windows protects that folder, see [os error 5](#os-error-5-access-denied-windows)).
3. Double-click `pesm.exe`.

### Linux

1. Unpack the archive into your home folder, for example:
   ```sh
   tar xzf pesm-<version>-linux-x86_64.tar.gz -C ~/
   cd ~/pesm-<version>-linux-x86_64
   ```
2. Allow the small helper `pesm-speicher` to **read** FL27's memory. PES M needs this to get the result and stats of a match you play in FL27. Run this once:
   ```sh
   sudo setcap cap_sys_ptrace+ep ~/pesm-<version>-linux-x86_64/pesm-speicher
   ```
   - Do this again after **every update**, because a new file loses the permission.
   - It only works on a normal Linux file system (ext4, btrfs …), not on an NTFS or exFAT drive.
   - The helper only reads. It never writes into the game.
   - Optional: you can give the same permission to `pesm` itself (`sudo setcap cap_sys_ptrace+ep .../pesm`). Then PES M reads the memory directly.
3. Start PES M with `./pesm.sh` (or `./pesm`).

You can check the memory access in PES M under **Settings › Football Life 2027**. The line "Memory helper" shows whether the helper is there, and the exact `setcap` command for your folder.

---

## 2. Choose your FL27 world (step "1 World")

On the start screen, click **New career**. The first step is **1 World**.

PES M looks for your game folder by itself:

- **Windows:** it looks for `FL_2027.exe` next to PES M and on every drive, in places like `X:\SP Football Life 2027`, `X:\Games\SP Football Life 2027` and `X:\Program Files (x86)\SP Football Life 2027`.
- **Linux:** it only knows the author's own Lutris prefix. On your PC it will most likely **not** find the game by itself, so pick the folder by hand (see below).

If no world is shown, click **Browse …** and choose your **`SP Football Life 2027`** folder (the one that contains `SiderAddons`). You can also paste the path with Ctrl+V into the field and click **Check**. Typical paths:

| | Game folder |
|---|---|
| Windows | `C:\Games\SP Football Life 2027` (or wherever you installed it) |
| Linux (Lutris) | `<your Wine prefix>/drive_c/Games/SP Football Life 2027` |

You may also pick `SiderAddons`, `SiderAddons\livecpk` or a world folder inside it. PES M finds the game folder from there and remembers it in `spielordner.txt` (see [Where PES M keeps its files](#where-pes-m-keeps-its-files)).

### Which database? Extracted world or `.cpk` mode

PES M reads the FL27 database (the "pesdb") in one of two ways:

- **Extracted world (best):** a mod world in `SiderAddons\livecpk\<world>` that contains the unpacked database in `common\etc\pesdb` (for example a Megapack world). PES M shows every world it finds as a button. Click the one you play with.
- **`.cpk` mode:** if you have no extracted world, click **"No extracted world: read from the FL27 .cpk files"**. PES M reads the game's own database straight from the `.cpk` files of the game folder. This takes a moment. It saves the result as the folder `SiderAddons\livecpk\FL27-Datenbank aus CPK`. That folder is **not** added to `sider.ini`, so the game itself never loads it.

When the world is loaded, PES M shows how many leagues, clubs and players it found. Click **NEXT**.

> Without FL27 you can try PES M with the **Built-in test world** (real clubs, made-up player names). You cannot play its matches in FL27.

## 3. Choose your club, manager and start

- **2 Club:** search for a club, league or country, or filter by country and league. Click a club, then go on.
- **3 Manager:** enter your manager's name (and optionally a photo, a PNG file).
- **4 Start:** check everything and start the career.

When the career starts, PES M copies the world into your career folder. The original world stays untouched. With a big world (for example a Megapack with tens of thousands of files) this can take a few minutes. A progress bar shows the steps.

### Optional: real contract data from SoFIFA

At the start of a career, PES M can take over real contract data (potential, wage, market value, contract end, release clause, loans) from a **German SoFIFA export** of the FCFL Stats Converter (a `.csv` file). PES M only reads the files. It looks in these folders and uses the newest full export:

| | Folder |
|---|---|
| Windows | `%APPDATA%\pesm\sofifa\` , `%APPDATA%\FL-Stats-Converter\ea-base\` , `%APPDATA%\FL-Stats-Converter\ea-updates\` |
| Linux | `~/.local/share/pesm-rust/sofifa/` , `~/.local/share/fl-stats-converter/ea-base/` , `~/.local/share/fl-stats-converter/ea-updates/` |

Put the file there **before** you start the career. Without it, PES M estimates these values itself.

---

## 4. Play a match in FL27

On match day you can **Simulate** the match in PES M, or **Play (FL27)** and play it yourself in Football Life 2027.

### The first match of a career (FL27 closed)

1. **Close FL27** if it is running.
2. On the match day page choose **Play (FL27)**.
3. The first time, click **Allow match launch**, then read the note and click **Understood, don't ask again**.
4. Click **Start match in FL27**.

PES M now does this, step by step:

- It saves your career and backs up the original world (once).
- It writes your career database (squads, transfers, **both line-ups**) into the career's own LiveCPK folder: `SiderAddons\livecpk\PESM<Club><Number>`, for example `PESMArsenalFC01`.
- It adds that folder as a `cpk.root` line to `SiderAddons\sider.ini`, and switches off other worlds that would cover it.
- It installs the Sider module **PESM-LivePatch** (`SiderAddons\modules\pesm-livepatch.lua`) and adds its `lua.module` line to `sider.ini`.
- It writes the match settings into `SiderAddons\flw\`: `wettbewerb.txt` (competition look: scoreboard, walk-out, ball), `stadion.txt` (home stadium), `wetter.txt` (weather), `spielregeln.txt`.
- It starts FL27:
  - **Windows:** with `FL 2027 start.exe` from the game folder. If that is missing, with `sider.exe` and then `FL_2027.exe`. Windows may ask for admin rights (UAC), click **Yes**. *(untested)*
  - **Linux:** through Lutris. **Known limit:** the beta starts Lutris game number 7 (the author's install). If FL27 does not open, or a different game opens, start FL27 yourself in Lutris. PES M waits up to 4 minutes for it.

Then, in FL27 (PES M shows these steps on screen too):

1. In the main menu choose **Kick Off** and move your controller to the side PES M tells you (**Home** or **Away**).
2. Choose the teams PES M names (home team and away team).
3. Leave the stadium as it is. PES M has set it.
4. If you like, set the tactics from PES M's **tactics note** in FL27's own tactics menu.
5. Kick off and play.

After the final whistle, PES M reads the score and the real stats (goals, assists, shots, passes, cards, saves …) from the game's memory and rates your players. If PES M misses the final whistle, click **Confirm result**: it books the score as it stands. If the reading does not work at all, you can always enter the result by hand.

Tip: switch on **Settings › Football Life 2027 › "Quit FL27 after full time"** if you want PES M to close the game after each match.

### The next matches (FL27 stays open)

You do not have to restart FL27 between matches. If FL27 is still running when you click **Start match in FL27**, PES M prepares the changes for the running game. Then in FL27:

1. Open the Sider overlay (**Space**).
2. Pick the module **PESM-LivePatch** (key **1**).
3. Press **F7**.
4. Close the overlay and start the match as above.

This updates the 25 attributes, line-ups, shirt numbers and formations. **New or transferred players, skills, positions and names need a restart of FL27.** PES M tells you when that is the case: quit FL27 and click **Start match in FL27** again.

### Coming soon: direct start

We have a direct start working in tests: FL27 boots straight into the match with the right teams and kits, without the main menu (a Sider module and a small file `SiderAddons\flw\direktmatch.txt` with the two team IDs). It is **not in PES M yet**, and its files are not in this repository. Until then, pick the teams yourself as described above.

### Undo everything

**Settings › Football Life 2027 › Uninstall (career from FL27)** removes the career from the game folder and restores what PES M changed. PES M backs up `sider.ini` and the original files before it changes them.

---

## Where PES M keeps its files

| What | Windows | Linux |
|---|---|---|
| Log file | `%APPDATA%\pesm\pesm.log` | `~/.local/share/pesm-rust/pesm.log` |
| Chosen game folder | `%APPDATA%\pesm\spielordner.txt` | `~/.local/share/pesm-rust/spielordner.txt` |
| SoFIFA exports | `%APPDATA%\pesm\sofifa\` | `~/.local/share/pesm-rust/sofifa/` |
| Careers (saves, career database, world backup) | see note below *(untested)* | `~/.local/share/pesm-rust/karrieren/` |
| Career in the game | `<game folder>\SiderAddons\livecpk\PESM<Club><Number>` | same, inside the Wine prefix |
| Match settings for FL27 | `<game folder>\SiderAddons\flw\` | same |

Inside `karrieren` each career has a save file `<name>.json` and a folder `<name>` with the career database (`pesdb`) and the world backup (`original`). Older saves go to `karrieren/sicherungen/`, deleted careers to `karrieren/papierkorb/`.

> **Windows careers folder (untested):** the beta builds this path from the `HOME` variable, which Windows usually does not set. Then the careers end up in `.local\share\pesm-rust\karrieren` inside the folder PES M was started from (usually next to `pesm.exe`). Look there first if you need your saves.

`%APPDATA%` is usually `C:\Users\<you>\AppData\Roaming`. Type `%APPDATA%\pesm` into the Explorer address bar to open it.

---

## Troubleshooting

**Bug report:** **Settings › Save & career › Diagnostics for bug reports** packs `pesm.log`, `sider.log`, `sider.ini` and the versions into one archive next to the log (your user name is replaced, no save goes in). Please attach it.

### "os error 5" (access denied, Windows)

Windows did not let PES M write a file. Usual reasons:

- **The file is in use.** Close FL27 (and Sider) and try again. If two PES M windows are open, close one.
- **The folder is protected.** PES M or FL27 is inside `C:\Program Files` or `C:\Program Files (x86)`. Move PES M to a normal folder like `C:\PESM`. If FL27 is there, start PES M **as administrator** (right-click `pesm.exe` › *Run as administrator*).
- **FL27 runs as administrator, PES M does not.** Then PES M may also not be able to read the match result. Start PES M as administrator too.
- An antivirus program blocks the file. Allow PES M's folder.

### No world found / "This folder is missing the file …"

You picked a folder that is not an FL27 world. Pick the `SP Football Life 2027` folder (it must contain `SiderAddons\livecpk`). If you have no extracted world, use the `.cpk` mode button.

### FL27 does not start, or "FL27 didn't start within 240 s"

- Windows: check that `FL 2027 start.exe` or `sider.exe` and `FL_2027.exe` are in the game folder.
- Linux: start FL27 yourself in Lutris (see the known limit above).
- Or enter the result by hand.

### The match result does not come back (Linux)

Run the `setcap` command from [step 1](#linux) again (after every update) and check **Settings › Football Life 2027 › Memory helper**.

### My changes (line-up, transfers) are not in FL27

- Another world may be above your career in `sider.ini`. Start the match from PES M with FL27 closed: PES M puts the career line in the right place.
- With FL27 open, you have to press **F7** in the PESM-LivePatch module. New players need a restart.

### FL27 says "FL27 runs another career or without the module PESM-LivePatch"

Quit FL27 and click **Start match in FL27** again.

---

## Help and feedback

Questions, bugs, ideas: join us on Discord: **https://discord.gg/ACW3wF2MqW**
