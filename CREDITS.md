# Credits

| What | Source | Licence |
|---|---|---|
| Barlow fonts (`assets/fonts/`) | Jeremy Tribby, https://github.com/jpt/barlow | SIL Open Font License 1.1 |
| Interface sounds (`assets/sfx/`) | Kenney, https://kenney.nl | CC0 |
| Club positions and places (`assets/geo/`) | Wikidata (built with `tools/geo_bauen.py`) | CC0 |
| Real career data of selected players and clubs (`assets/inhalt/pesm/fl/sim/config/realdaten/`) | Facts from Wikipedia season articles, written up by hand | facts |
| League rules 2025/26 (`daten/ligaregeln.toml`) | Wikipedia season articles, UEFA access list (sources listed in the file) | facts |
| Sample world (`assets/world.json`) | League and club structure of a Football Life world; **all player names are made up** | MIT |
| Rules, texts, stories, content pool (`assets/regeln`, `assets/texte`, `assets/story`, `assets/inhalt`) | Written for PES M (Python original), list in `assets/inhalt/QUELLEN.txt` | MIT |
| Club leadership, staff, stadiums and training grounds (`assets/vereinsfuehrung.toml`) | Wikipedia club articles (source URL per entry) | CC BY-SA 4.0 |
| Overall rating weights (`assets/ovr_weights_fl27.json`) | Own fit against FL27 ratings | MIT |

Not included: game files, Sider modules, crests, player photos, stadium pictures, kit textures and databases. PES M reads those at runtime from the player's own installation and never ships them.

Newspaper names, journalists and agents are invented. Club names are used only to name the clubs.

## BlankState222
Ideas for two FL27 sider modules come from BlankState222 (https://github.com/BlankState222): two-legged ties (`two_legs`) and start stamina (`start_stamina`). PES M does not copy their code; the match rules for first and second legs are written anew in `assets/fl27/livepatch/pesm-livepatch.lua`.
