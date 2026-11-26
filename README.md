# PES M

**A football manager that uses Football Life 2027 as its match engine.**

> [!WARNING]
> **Work in progress.** PES M is under heavy development and this repository gets several pushes a day. Things change fast and can break.
> **Beta downloads** for Linux and Windows are on the [Releases page](https://github.com/FabianZettl/pesm/releases) (Windows is not tested on a real PC yet). Or build it yourself (see [Build](#build)).
> **New here?** Read the [Getting started guide](docs/GETTING-STARTED.md): from the first launch to a career and your first FL27 match, for Windows and Linux.
> PES M never touches your original FL27 database: it always works on **its own copy of the database in a separate LiveCPK folder** per career.

<p align="center">
  <img src="docs/screenshots/zentrale.png" width="49%" alt="Hub: today's task, the season so far, every part of the club">
  <img src="docs/screenshots/aufstellung.png" width="49%" alt="Tactics: formation, starting eleven on the pitch, bench and assistant's advice">
  <img src="docs/screenshots/kader.png" width="49%" alt="Squad: player portraits with rating and mood">
  <img src="docs/screenshots/spielerprofil.png" width="49%" alt="Player profile: photo, abilities, attribute radar, strengths and weaknesses">
  <img src="docs/screenshots/posteingang.png" width="49%" alt="Inbox: rumours, offers, player talks">
  <img src="docs/screenshots/vereine.png" width="49%" alt="Clubs and players: league, coaches, map, club history and squad">
  <img src="docs/screenshots/spieltag.png" width="49%" alt="Matchday: stadium and city before kick-off, then head to head and team talk">
  <img src="docs/screenshots/statistik.png" width="49%" alt="Player statistics: percentile wheel against the position group">
  <img src="docs/screenshots/kaderplanung.png" width="49%" alt="Squad planning: depth per position, contract timeline, staff ideas">
  <img src="docs/screenshots/praesidium.png" width="49%" alt="Board: trust, season objective and the club's real president and directors">
</p>

> Screenshots taken with the author's own FL27 world (FL26 Megapack, Bayern career, autumn 2026). Crests, player photos and stadium pictures are read at runtime from the player's own game install; they are **not** shipped in this repository.

> Unofficial fan project. Not affiliated with Konami or the Football Life team. This repository contains **no** game files, crests, photos or databases. You need your own copy of FL27.

---


PES M started as a small companion app for Master League. It is now a football manager of its own: you run a club over many seasons, and the matches themselves are played in Football Life 2027.

### What PES M does

- **Careers over many seasons** on your FL27 world (leagues, clubs and squads are read straight from the pesdb). One day is one step.
- **Squad and tactics:** line-up, roles and instructions, set pieces, opponent reports, training, medical, form, development per ability.
- **Market:** transfers with negotiations, agents with their own goals, clauses, loans, scouting, shortlist, staff market.
- **Club:** board and objectives, finances, stadium and facilities, sponsors, coaching staff, academy, reserve team, partner clubs.
- **World:** leagues, cups and European competitions with real rules (play-offs, relegation), tables, rankings, club history, world map.
- **People and media:** inbox, player talks, dressing room, press conferences, newspaper, fans, stories that unfold over weeks.
- **Four languages** (English, German, Spanish, French) and two themes (light "Sonntag", dark "Flutlicht").

### FL27 as the match engine

On match day you can simulate the match in PES M (with a live ticker) or play it yourself in FL27:

- PES M writes squads, transfers and **both line-ups** into your career's own copy of the pesdb and mounts it through Sider.
- It sets the **competition** (scoreboard, walk-out, ball), the home team's **stadium** and the **weather**.
- FL27 starts through Lutris and **boots straight into the match**, skipping the main menu and team selection, with the right kits. Switch it on under *Settings › Football Life 2027 › Direct start*; PES M builds the files it needs from your own game install.
- After the match PES M reads the **real statistics** (goals, assists, shots, passes, cards, saves …) from the game's memory and rates the players from them. Reading only.
- Goes well with the [PESM Coaching Zone](https://github.com/FabianZettl/pesm-coaching-zone): live stats on your phone while you play.

PES M backs up the originals before it changes anything in your game folder; "Rückbau" in the settings restores them. The Sider modules and flow files for the direct start are **not** in this repository, because they build on files from the game and from mods.

**Status:** under active development. Tested on Linux (Proton via Lutris). Controller assignment in the direct start and goal celebrations are being finished.

### Build

```sh
cargo build --release
./target/release/pesm
```

Without FL27, PES M starts with a small built-in sample world (real clubs, made-up player names). Tests: `PESM_KARRIEREN=$(mktemp -d) cargo test --release`.

Rules and numbers: [MECHANICS.md](MECHANICS.md). Sources and licences: [CREDITS.md](CREDITS.md).

Thanks to [BlankState222](https://github.com/BlankState222) for the ideas behind the two-legged ties and start-stamina FL27 modules.

**Start fitness in FL27 (optional):** install BlankState222's `start_stamina` module yourself from [github.com/BlankState222/FL27---siders](https://github.com/BlankState222/FL27---siders) and list it in `sider.ini` directly below the PESM LivePatch module. Then switch on *Settings › Football Life 2027 › Write start fitness for start_stamina*: before every match PES M writes `SiderAddons/modules/start_stamina.txt` with the fitness of both squads. All credit for the module goes to BlankState222.

## License

MIT, see [LICENSE](LICENSE). Fonts: SIL Open Font License. Sounds: CC0.
