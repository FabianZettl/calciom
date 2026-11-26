# Changelog

CalcioM (Calcio Management) was called PES M until version 0.2.1. Full release notes: [Releases page](https://github.com/FabianZettl/calciom/releases).

## Next release (planned)

## 0.6.0 – 26.11.2026

**Before the match**
- Kit choice: pick both teams' kits from FL27's kitserver on the kick-off screen, shown as 2D pictures. The direct start puts them on in FL27.
- Match length, weather (with "Realistic") and difficulty are dropdowns on the kick-off screen.
- The hub's next-match card shows the competition logo and "league, matchday N" between the crests.
- Direct start and quick start are one switch. A career without the direct start asks on load what it misses, until you say you only simulate.
- The direct start no longer loads FL27's own edit data, which could overwrite the career database.
- Stadium packs and other add-ons are no longer switched off in sider.ini (only whole worlds with a Player.bin are). Add-ons an older version switched off come back on at your next match start, if they were on before your career.
- CalcioM warns before kick-off if the home stadium can't be set: no line for the home team in map_teams.txt, its stadium folder missing, or stadiums.lua without the CalcioM addition.
- Safety: once FL27 is closed after a match, sider.ini goes back to the state without CalcioM. FL27 started by hand opens normally (main menu, its own database). The next match start from CalcioM sets everything up again; quick start between matches is not affected.
- If the direct start setup fails, the error page names the real reason.

**Matches and reports**
- Match page for every club: right-click a club crest › Fixtures. The season by month with time, venue, result and competition (round and leg). Next to it is the chosen match with scorers, cards and ratings, plus earlier meetings.
- Every match in the world has a full report (scorers, assists, minutes, cards, ratings), written with the squads of the match day.
- Opponent report: no goalkeeper as the most dangerous player.

**Talks and events**
- Player talks differ from save to save and from player to player: ego, temper, ambition and morale decide how often someone comes forward.
- 28 new talk scenarios from real matches:
  - **Players and captain:** a super-sub who wants to start, the captain speaking up for a teammate, scoring but benched, goal drought, red card, bad game, keeper on the bench, youngster pushing, losing and winning runs, a tired ever-present.
  - **Assistant:** late goals against, a busy week.
  - **Board:** embarrassing defeat, derby win, cup bonus.
  - **Scout:** the opponent's star, an expiring contract.
  - **Youth coach:** a prospect ready for first-team training.
  - **Opposing coach:** a dig after your win.
  - **Press:** title talk, coach under pressure, derby defeat, goal drought, super-sub, keeper debate.
  - **Fans:** after an away defeat, after a comeback.
- "Scorer on a run" no longer reaches goalkeepers.

**Calendar, squad and settings**
- Calendar: click a day and choose "Play on to here". Own matches and press conferences still stop it.
- Settings › Game: switch off this season's summer transfer window for everyone.
- Player actions: send a young first-team player (19 or younger) back to the U19.

**CalcioM shows that it is working**
- Autosave runs in the background with a small "Saving …" badge, and opening a career shows "Loading the career …" instead of a frozen window (Windows reported "not responding").
- Pages that take a moment to build show "Loading …" first, and a slow day step or playing on shows "Simulating …".

**Under the hood**
- Real-world data (club leadership, coaches, competition and cup names, partner teams, journalists) sits in its own data pack; the program has neutral stand-ins without it.

## 0.5.3 – 26.11.2026 (hotfix)
- Read from the FL27 .cpk files: each database file now comes from the .cpk that contains it (FL27 update packages like data_s26272.cpk have players but no competitions).

## 0.5.2 – 26.11.2026
- FL27 started from CalcioM landed in its main menu instead of the side selection (after the rename a folder was missing that the direct start needs; CalcioM creates it now).
- Read from the FL27 .cpk files no longer gets stuck or fails on an incomplete _FL26MyWorld (or old career folder) in sider.ini; it reads the .cpk files instead.
- FL27 closed during a match: the last score read is no longer counted by itself. CalcioM asks: count this result, or discard it and play the match again or simulate it.
- New friendlies are set to "Play in FL27" by default; "Simulate" is still one click away.
- **AI pass photos for academy graduates (optional).** Promote a youngster from your academy and CalcioM paints him a real-looking pass photo on your own PC from his in-game face (skin, hair, beard). It shows in CalcioM and as his portrait in FL27. Turn it on under Settings › Features: one download of about 2.7 GB (stable-diffusion.cpp from GitHub and a smaller copy of SDXL-Lightning by the Hugging Face user mzwing, both checked by checksum). Without it nothing changes. "New photo" in the player profile paints another one.

## 0.5.1 – 26.11.2026
- Master League import: the ML saves found on your computer (FL27 save folder and save… copies) now show up as buttons with club and date under New career › 1 World, no file dialog needed.
- The big club crest at the top left sits on a light disc (a red crest disappeared in the red glow).

## 0.5.0 – 26.11.2026 – first stable release, new name
- **CalcioM, formerly PES M.** Unpack it into your old folder: data folder, careers and settings move over by themselves; old careers keep their names.
- **Quick start:** FL27 stays open between matches. After the final whistle it waits at the side selection. On the next match day, the next fixture, your line-up, stadium and weather go into the waiting game, and FL27 comes to the front.
- The restart check compares against the squads FL27 actually loaded. **Start anyway** covers transfer windows, when transferred players may be missing.
- **Start from a Master League save:** "Import ML save…" in the world step (read only).
- New logo, start picture and lawn-green default accent.
- Update check and links point to github.com/FabianZettl/calciom; the old links forward.

## 0.2.1-beta – 26.11.2026
- **Direct start:** FL27 boots straight into your match. The first start asks once and builds the files from your own game install; no game files are shipped.
- Spanish and French as interface languages.
- With direct start on, GigaPack's fl26match statistics call is switched off (with a backup), which stops random FL27 crashes.

## 0.2.0-beta – 26.11.2026
- Own trophy ceremony after a won final.
- Player looks from FL27 (skin, hair, beard), plus passport photos for players without a photo.
- Sharper text at every resolution and high-DPI support.

## 0.1.4-beta – 25.11.2026
- Award season, a European framework, the Skill Tree, kick-off times per league, an extended calendar and feature switches.

## 0.1.3-beta – 24.11.2026
- Duel screen, opponent view, live match menus, matchday round-up, tactics engine and transfer AI.
