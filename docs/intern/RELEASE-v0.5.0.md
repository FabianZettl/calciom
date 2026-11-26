# CalcioM 0.5.0 – Calcio Management (formerly PES M)

The first stable release, under a new name. CalcioM is the football manager that used to be called PES M: same game, same saves, Football Life 2027 as the match engine.

## Upgrading from PES M
- Unpack CalcioM **into the same folder** as your old PES M (or anywhere else) and start `calciom` / `calciom.exe`. At the first start your data folder moves to the new name, so careers, settings and your game folder are all still there. Old careers keep their `PESM…` folder in the game and load as before.
- On Linux run `sudo setcap cap_sys_ptrace+ep <folder>/calciom-speicher` once.
- **Old PES M versions won't show this update.** Please download CalcioM from this release page and tell your friends.

## New

**Quick start: FL27 stays open**
- After the final whistle FL27 waits at the side selection. Play on in CalcioM; on the next match day "Start match in FL27" loads the next fixture, your line-up and attribute changes, stadium and weather into the waiting game and brings FL27 to the front. You only confirm your side.
- New or transferred players in one of the two teams need an FL27 restart. CalcioM then offers **Start anyway** (transferred players may be missing; this happens during or after a transfer window).
- Injured players taken out for a match no longer count as a squad change.

**Direct start** (since 0.2.1)
- FL27 boots straight into your match: no main menu, no team selection, the right kits. The first start asks once and sets everything up from your own game files; nothing from the game is shipped.

**Start from your Master League save**
- In the world step, **Import ML save…** reads an FL27 Master League save (`ML00000001` … in `Documents\SPGAME\Football Life\2027\save`), shows a preview (club, date, coach, league position) and starts your career from it: date, every club's squad, contracts and wages, abilities, fitness, form, injuries, tables, fixtures, the coaches and your transfer budget. The save is only read, never changed. Tested with copies of real saves.

**New look**
- The CalcioM logo and word mark fade in at start, and the default accent is lawn green.

**Also in this release** (since 0.2.0)
- Spanish and French as interface languages.
- Fix for random FL27 crashes with GigaPack's fl26match while direct start is on (statistics call switched off with a backup, restored when you switch direct start off).

## Known gaps
- **Windows is not tested on a real PC yet.** Please report how it goes in #bug-reports.
- The entrance animation before kick-off is not there yet.
- Tested with FL27 1.00 only.
- In Spanish and French the long story texts stay English for now.
