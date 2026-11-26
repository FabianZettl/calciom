# CalcioM (Rust) – Regelblatt

Every number that shapes play, with where it came from. "FL sim" = the original's FL world simulation in
`pesm/fl/sim/ (Python original)` (config files under `config/`), read in Deep mode.
The original has no machine code: its launcher (`pes-management/.venv/bin/pyside6-qmlimportscanner`, opened with core_target_open, target t_c9e52544560b) is a Python script, and native_functions_list refuses it ("not native machine code"), so Ghidra pseudocode does not apply. The game logic is compiled Python 3.14 bytecode; `__pycache__/match.cpython-314.pyc` was read with formats_bytecode_show (evidence ev_4c3468fed1eff124c0c4f48ed2e0d79f08066a2c2482d0fd221976c9f8ba4e5e: 41 code objects, e.g. `eff` line 10, `lineup` 21, `pmf` 60, `rates` 69, `play` 94, `_events` 130, `_minutes` 167, `_ratings` 187). The bytecode reader lists functions and their names, not their bodies, so the formulas below come from the matching source lines. "Address" in this sheet = `file:line` of the function.
"estimated" = looked for in the original and not found, or the original needs FL27 at runtime.

## Time and calendar
| Value | Number | Source |
|---|---|---|
| One step | one day ("Tag abschließen"); Space key = same | request + `fl/sim/welt.py` advance |
| Season length | 365 days, day 0 = 1 August, counted as Saturday | `config/sim.json` season_days, calendar.weekday0 = 5 |
| League days | Saturdays between day 14 and 301; midweek (Wednesday) added when a league needs more rounds | `sim.json` league_start_day/league_end_day, `comp.py` |
| International breaks | days 35–41, 70–76, 105–111, 238–244 | `sim.json` intl_breaks |
| Winter break | days 139–152, only leagues with ≤ 18 teams, not England/Scotland/Turkey | `sim.json` winter_break |
| Transfer windows | days 0–31 and 153–183 | `sim.json` transfer_windows |
| Schedule | double round robin, circle method | `comp.py` |
| Points | 3 / 1 / 0; tiebreak points, goal difference, goals | `comp.py` |
| Detail levels | own league full (lineups, cards, ratings), first divisions light (scorers, injuries), rest summary (results from fixed strength) | `welt.py`, `sim.json` detail.light_pattern "D1_LEAGUE" |
| Promotion/relegation | 3 up / 3 down between tiers of the same country | `sim.json` pyramid.count 3 |
| Autosave | weekly (day % 7 == 2) | `sim.json` autosave_days 7 |

## Match engine (`fl/sim/match.py`, `sim.json` match)
| Value | Number | Source |
|---|---|---|
| Player worth in a slot | (OVR − position penalty) × (1 − 0.15 × (1 − fitness/100)) + (morale − 50)/50 × 1.5 | `match.py eff()` |
| Position penalty | 0 own position, 4 near position, 8 same line, 14 other line, 40 keeper/outfield swap | `sim.json` pos_penalty; near pairs from `training.json` (FL data here has no per-position proficiency) |
| Team strength | mean worth of the XI + familiarity bonus (fam − 50)/50 (max ±1) | `match.py strength()`, `training.py fam_bonus` |
| Goal expectation | home 1.5 × e^(0.072 × Δ), away 1.17 × e^(−0.072 × Δ) | `sim.json` home_goals, away_goals, beta |
| Style | attacking own +5 %/opp +4 %, defensive own −6 %/opp −5 % | `sim.json` style_matrix |
| Score | Dixon-Coles, rho −0.07, max 9 goals | `match.py pmf()` |
| Scorer weight | GK 0.01, CB 0.5, FB 0.6, DM 0.8, CM 1.6, wide mid 2.2, AM 3.0, wing 3.4, SS 4.0, CF 5.0, × (OVR/75)³ | `sim.json` scorer_weight |
| Own goal | 4 % per goal | `match.py _events` |
| Assist | 70 % of goals, creative positions weighted | estimated |
| Cards | yellows Poisson(1.7) per team, red 6 % per team; every 5th yellow = 1 match ban, red = 1 match ban; a ban is served when the club plays (all squad players count down) | `sim.json`; bans `welt.py _apply` line 266/299 |
| Injuries | longer of old and new injury (`welt.py` line 306); 2 % per player per match; 3/7/14/28/56/120 days at 30/25/20/12/8/5 % | `sim.json` injury_per_player, injury_days |
| Substitutions | up to 3 after minute 55, best bench player of the same position | `match.py _minutes` |
| Rating (Sofascore style) | 6.3 ± 0.25 win/loss + 0.04 × (worth − 70)/10 + 0.9 per goal − 1.2 own goal − 1.4 red + 0.5 clean sheet (GK/defenders) ± 0.5 jitter, 3.0–10.0 | `match.py _ratings` |
| Assist bonus in rating | +0.5 | estimated (FL rating has none) |
| Shots, on target, possession | shots = goals + Poisson(7.5 × λ); on target 30 % of misses; possession 50 + 1.2 × Δ | estimated (FL27 memory reading not ported) |
| Fatigue | −16 × minutes/90 per match, floor 20 | `sim.json` fatigue_per_match |
| Recovery | +5 per day, only while a player is tired or injured | `sim.json` recovery_per_day, `welt.py _recover` line 311 |
| Form | form × 0.8 + (rating − 6.4) × 1.1, ±3; daily × 0.97 while recovering | `welt.py _apply` line 288, `_recover` line 319 |
| Morale | players who played: ±3 win/loss, +1 per goal, 10–95; daily 2 % back to 50 while recovering | `welt.py _apply` line 291, `_recover` line 318 |
| Calibration (measured) | 2.69–2.81 goals per game, 44–47 % home wins, 23–26 % draws, champion 86 pts | target ranges `sim.json` calibration; test `game::calib` |

## Training (`training.json`, per day of the week plan)
| Unit | Fitness/day | Injury factor | Development | Familiarity |
|---|---|---|---|---|
| Frei | +0.5 | 0 | 0.99 | −0.07 | 
| Erholung | +1.8 + 0.33 | 0.4 | 0.97 | +0.08 |
| Leichte Einheit | +0.83 | 0.5 | 0.94 | +0.12 |
| Normale Einheit | −0.67 | 1.0 | 1.0 | +0.17 |
| Taktiktraining | −0.38 | 0.8 | 1.0 | +0.83 |
| Intensive Einheit | −3.2 | 2.5 | 1.07 | +0.18 |
Weekly costs in `training.json` (intensity cost 1/4/12, focus Taktik fam 5, Regeneration rec 1.8, free day rec 0.5) were spread over six training days.
Players 31+ lose 15 % more fitness (old_cost_mult 1.15). Three hard weeks in a row: morale −1.5 (streak). Training injuries 0.0015 per player per week × unit factor (`entwicklung.json` injury.train_week).
Individual programs (max 6): Fokus development ×1.12, extra cost, injury ×1.1; Extraschicht (≤ 23) ×1.2, injury ×1.3 (`training.json` individual).

## Development (`entwicklung.py _step` line 187, `season_end` line 239; `entwicklung.json`)
Spread weekly (/52) over the season like the original's weekly plan for full-detail clubs:
- Age for the tables: age − 3 for goalkeepers (gk_shift 3).
- Up to 26: delta = max(−2, potential − strength) × GROW[age] / (sum of GROW from that age to 26). GROW 15→26: 3.0, 2.8, 2.6, 2.4, 2.0, 1.6, 1.3, 1.0, 0.8, 0.5, 0.3, 0.2.
- From 27: DECLINE 27→36: −0.05, −0.2, −0.4, −0.7, −1.0, −1.4, −1.8, −2.1, −2.4, −2.8; potential follows strength.
- Training: growth × lerp(0.85, 1.2, level) × plan multiplier; decline × lerp(1.1, 0.85, level) / plan multiplier (`train_growth`, `train_decline`, `tr_mult`). Staff level is fixed at 5 of 10 (estimated: no staff model yet).
- Play time: growth × (0.9 + 0.33 × share), decline × (1.1 − 0.2 × share); share = minutes / 2700 for the own league, otherwise 0.85 / 0.4 / 0.1 by squad rank 1–11 / 12–16 / rest (`proxy_share`, `proxy_ranks`).
- Above strength 88 growth slows: × max(0.3, 1 − (strength − 88)/20) (`top_soft`).
- Performance with ≥ 8 graded games: (average grade − 6.5) × 0.8, clipped −0.5 … +1.0, no positive part past 28 (`perf`).
- Noise: 0.9 per season while growing, 0.6 otherwise; strength 40–99.
- Potential at the start: strength + remaining growth × talent (talent N(1, 0.35), 0.15–2.3; estimated, the FL data here has no potential).
Retirement 33: 2 %, 34: 6 %, 35: 15 %, 36: 30 %, 37: 50 %, 38: 70 %, 39: 85 %, 40: 100 %.

## Money (`transfers.json`, `geld.json`, `core/vereinswelt/vereinswelt.toml`)
| Value | Number | Source |
|---|---|---|
| Market value | 10^curve(strength) × age factor × contract factor (1/2/3+ years: 0.72/0.9/1.0) × league factor e^((level−68)/60) clipped 0.7–1.3 | `transfers.py market_value` |
| Value curve (log10 €) | 50: 4.331, 60: 5.341, 70: 6.694, 80: 7.619, 90: 8.135, 95: 8.282 | `transfers.json` value.curve |
| Age factor | 17: 0.637, 20: 0.696, 23: 1.147, 26: 1.0, 29: 0.83, 31: 0.717, 35: 0.605 | `transfers.json` value.age |
| Fair wage | 10^wage curve × wage age factor × league factor e^((level−68)/18) clipped 0.5–1.3, min 30,000 € | `transfers.py wage_of` |
| Contract length | ≤ 23: 4, ≤ 28: 3, ≤ 31: 2, else 1 year | `transfers.json` contract.years |
| Club revenue | wage bill / triangular(0.5, 0.95, mode 0.6); TV 38 %, gate 30 %, commercial 32 %; running costs 25 % | `transfers.json` club |
| Start balance | 5–50 % of revenue | `transfers.json` balance_start |
| Transfer budget | 22 % of revenue | `core/wirtschaft/budget_start.toml` |
| Ticket price | (18 + 40 × √reputation) × 0.8 / 1.0 / 1.25 | `vereinswelt.toml` (curve shape estimated) |
| Attendance | 0.30 + 0.70 × fans/100 − 0.10 × (price step − 1), 0.25–1.0; capacity 8,000 + 55,000 × reputation | `vereinswelt.toml`, `werte.py` |
| Merchandising | 30 % of commercial income × price × volume × (0.7 + 0.6 × fans/100) | formula `werte.py`; split estimated |
| Bookings | monthly (every 30 days): TV, sponsors, merchandising, wages /12, costs /12; gate per home match | `transfers.json` finance.month_days |
| Manager salary | 0.4 % of club revenue, min 120,000 € | estimated |

## Transfers and negotiations (`verhandlung.py`, `transfers.json` markt)
| Value | Number | Source |
|---|---|---|
| Asking price | value × 1.3 (not listed) / 1.0 (listed) / 2.0 (protected key player); hard club ×1.1, relaxed ×0.95; expiring ×0.92; injured ×0.85; rival ×1.5; last 3 days ×1.08 | `transfers.json` markt.price |
| Club answer | accepts ≥ asking price; counters ≥ 80 % (midpoint); otherwise no; max 3 rounds, 14-day session | `verhandlung.py` (close_ratio 0.8, rounds 3, ttl 14) |
| Player answer | need = fair wage × persona (money-driven 1.15, fair 1.0, loyal 0.95, ambitious 1.05, home 0.97) × (1 + 0.04 × reputation gap over 25); ±1 year off his wish +5 %; counter ≥ 85 % | `verhandlung.py` persona, `ruf.json` absage |
| Refusal | player reputation (3 × (OVR − 58)) more than 45 above the club | estimated from `ruf.json` absage_gap 25 |
| Answer time | 1 day + 1 per 20 reputation points the other club is bigger + 2 for money-driven agents + 1 for stars | estimated ("Dauer hängt von Spieler und Ansehen ab") |
| Offers for own players | in window rounds every 7 days: listed 50 %, regulars 8 %, others 3 %; fee 0.9–1.3 × value | `transfers.json` offers fee_range; chances estimated |
| AI market | up to 25 moves per round between clubs, upgrades within budget (50 % of balance + 10 % revenue) | `transfers.json` market (simplified) |

## Scouting (`stab.json`)
Knowledge per player 0–100 %. Start: own league 55 % (estimated), stars (OVR ≥ 80) and top-6 leagues 35 % (`markt.base_know`), others 10 %. Weekly gain 0.25 + 0.04 × scout quality (1–10), +0.35 for a watch order; 3–8 players per scout and week. Shown strength range ±5 × (1 − knowledge), potential ±12 × (1 − knowledge) (`stab.json` fuzz). Moneyball = strength minus the strength his price buys (value curve inverted; estimated). System fit from `psyche.json` idea_pos (Ballbesitz, Pressing, Konter, Abwehr).

## Academy (`akademie.json`)
30 youth players at the start (15–18 years), OVR = 0.78 × club level (36–58); U17 (≤ 16) and U19 (17–18) with own 12-team leagues every Sunday: λ = 1.3 × e^(0.06 × Δ (+0.2 home)); other academies 0.68/0.72 × club level (`str_ratio` 0.72). Six new 15-year-olds every summer (estimated); at 19 promote or release.

## Board, press, career (`transfers.json` manager, `core/vorgang/lagen.toml`, `core/medien/medien.toml`)
| Value | Number | Source |
|---|---|---|
| Season goal | strength rank in league: top 8 % title (place ≤ 10 %), top 30 % Europe (≤ 30 %), top 70 % mid-table (≤ 65 %), rest survival (≤ 85 %) | `transfers.json` manager.goals |
| Patience | 40 / 48 / 58 / 68 | `transfers.json` patience |
| Pressure | monthly from day 90 (≥ 8 games): +2 per place below the goal (max +10), −6 on target; season end +3 per place | `transfers.py _pressure` |
| Confidence | 100 − pressure/patience × 100; labels 80/65/50/35/25 | `lagen.toml` |
| Warning / ultimatum / sacking | warning < 50, ultimatum < 25 for 60 days, sacked if still < 35; 60-day grace for a new coach | `lagen.toml` (warning threshold estimated) |
| AI coaches | sacked at pressure ≥ patience | `transfers.py` |
| Press tones | media/board/fans/dressing room: honest +2/+1/0/0, fighting 0/0/+2/+1, evasive −2/0/−1/0, praise +1/0/+1/+3, criticism +1/+1/0/−3, diplomatic +1/+1/0/0, hit back +1/−1/+2/0; skipping −3/−1/−1 | `medien.toml` |
| Fans | ±2 per win/loss (×2 in derbies), ticket price ±1 per month | `verein.json` derby fans_win |
| Job offers | monthly 15 % from clubs with more reputation within reach of the manager's; offers after sacking | estimated |

## Controls
Mouse for everything; Space = Weiter (end the day); 1–7 open the sidebar groups; 1–4 answer conversations and press questions; Esc closes windows. "▼" next to Weiter runs day by day until the next match, decision or press conference.

## FL27-Anbindung (Modul-Paket of 20.11.2026; grades BESTÄTIGT / CODE / ENTWURF as given there)
| Value | Number | Source |
|---|---|---|
| Statistics record | 0x2134 bytes, 40 places per side, away side +0x59650, header 65533/65534 + player id, kind k = 9 × u32 at rec + 8 + k·36 | spec §1.2 (BESTÄTIGT), `fl27.rs` |
| Kinds used | 0 Goal, 1 OwnGoal, 4 Assist, 6 Shoot, 17 ShootInFrame, 21 ShootPK, 22/23 Pass, 30 Centering, 50 TackleSuccess, 55 Foul, 56 OffSide, 57/58 Cards, 61 FK, 63 CK, 67 BallTouch, 78 SnatchByPassCut, 133 GKSave, 173 run (subs) | spec §1.3 |
| Beat / search | read every 2 s, block search at most every 30 s | spec §1.4 |
| Minute | 15 × section + 15 × (t − section start) / median section length; without following from kick-off the middle, marked uncertain | spec §1.4.3 |
| Yellow-red | second yellow of a player with a red is the red | spec §1.3 |
| xG | penalty 0.76, on target 0.12, off target 0.04 | spec §1.4.6 |
| Grades from FL27 stats | match.py base + estimated parts (assist +0.5, on target +0.1 ≤ 0.3, pass accuracy, tackles +0.15 ≤ 0.45, interceptions +0.1 ≤ 0.3, saves +0.25 ≤ 1.0, foul −0.1) | estimated, `game.rs fl27_note` |
| pesdb container | 16-byte header (WESYS), zlib level 9 | spec §2.1 |
| PlayerAssignment | 16 bytes: nr, player, team, k (bits 0..6 number − 1, bits 10..15 squad place) | spec §2.3 |
| Tournament id | CompetitionRegulation 2352-byte records, u16 at +2, competition at +8, first one; checked: PL 17, Bundesliga 50, CL 2 | spec §4, test `real_pesdb` |
| Weather | midweek games at night; winter 15 % snow, 35 % rain | estimated, `game.rs fl27_wetter` |

### Decompiled from FL_2027.exe (Ghidra, Deep mode)
CalcioM itself is Python (no machine code). The game engine is native: `FL_2027.exe` (SHA-256 14dd630a…a7997d, the same file the original's
table `config/spielversionen/fl27_1.00.json` names; image base 0x140000000, no ASLR) was opened with core_target_open (target t_14dd630aaaa0)
and the statistics getters the original lists were decompiled with native_function_pseudocode / native_functions_decompile.

| Address | What the code does | Used in the build |
|---|---|---|
| 0x1406a65d0 `StatsPlayerGoal` | `FUN_1406a7850(player, 0)` → float result | kind 0 = goals (ev_695e22b6…) |
| 0x1406a6700 / 0x1406a6780 / 0x1406a67c0 | same with kind 6 / 0x11 / 0x15 | Shoot 6, ShootInFrame 17, ShootPK 21 (ev_467db0dc…) |
| 0x1406a6c00 / 0x1406a6d80 / 0x1406a6f00 | kind 0x37 / 0x3a / 0x85 | Foul 55, CardRed 58, GKSave 133 |
| 0x1406a7850 | finds the player's statistics record (place at +0xc; places 11..21 flagged as bench) and calls 0x1408ca920 with mode 5 | places 0..10 = starting XI (ev_fd3e0aaf…) |
| 0x1408ca920 | `if kind < 0xb1`: value = *(rec + 8 + kind·0x24 + 4·section); mode 0 = sections 0+1+2, mode 1 = 3+4+5, mode 2/3/4 = section 6/7/8 alone, **mode 5 = sections 0..7** | 177 kinds (`ARTEN = 0xb1`); totals and events use sections 0..7 only, section 8 (probably the shoot-out) stays out (ev_4ffdd9bd…) |

## Karriere-CPK and sider.ini (Übergabe "Daten und Datenbank" §8.1)
| Rule | Value | Source |
|---|---|---|
| CPK name | "CalcioM" + club name (accents dropped, ASCII letters/digits, ≤ 30) + first free 01..99 | §8.1, `fl27::cpk_name` |
| quelle.json | `{spiel, welt, cpk}` in the career folder | §8.1 |
| sider.ini | other CalcioM careers' active lines commented with ";"; own line removed and set again as first active line below the active CalcioM_DirectMatch line; world switched on or added below; backup `sider.ini.bak-calciom-YYYYmmdd_HHMMSS` next to the file; written only when something changes | §8.1, `fl27::sider_ini_einrichten` |

## Paket 1: own FL database (Anweisungen 21.11.2026)
| Rule | Value | Source |
|---|---|---|
| World from a pesdb folder | leagues: name ends _LEAGUE, not UEFA_/AFC_, at least 10 teams with at least 11 players; a player belongs to the first league club only; players without a league club are free agents | `fl/sim/welt.py import_pesdb`, `pesdb::read_world`; checked on `_FL26-27-Def`: 99 leagues, 1,643 clubs, 48,777 players |
| Player.bin | 312-byte records; 25 abilities 6 bit + 40; position 4 bit at 434; age 6 bit at 408 (+15); position levels, style (5 bit at 155), 42 skill/COM bits; name at +68 and +251, shirt name +129 | Übergabe Daten §1.1-1.5, plbin.py |
| Rating | ovr = sum of w[pos][i]·ability[i] + w[pos][25] (`ovr_weights_fl27.json`, fallbacks 6 to 5, 7 to 5, 11 to 12); within 1 point of the Python value for 50 known players (test) | Übergabe §1.6, convert.ovr_model |
| Potential | tal = 1 + 0.02·(ovr − norm_ovr[age]) + N(0, 0.35) in 0.15..2.3; pot = ovr + REM[age]·tal; above 80: 80 + (pot − 80)·0.5; from 26 pot = ovr | Übergabe §1.7, entwicklung.py _ens |
| Development on abilities | the weekly change moves every ability that counts for the position by delta·w/sum(w²) (40..99); the rating follows the model | choice for Rust (continuous form of convert.fit_overall) |
| New players | unique id above all ids; template chosen by the id among up to 200 players of the position (players without a league club first); set: name, position and levels, abilities, age, height (position typical ± 6 cm), weight, foot, playing style, special abilities. Face, nationality and birth date are not decoded and stay the template's | Paket 1; body values estimated |
| Writing | Player.bin and PlayerAssignment.bin of the career copy: backup of both (5 kept), atomic, read back, rolled back on a difference; unchanged files keep their original bytes | Übergabe §0, Teil B 1 |
| Youth players in FL27 | own record in Player.bin, no club entry (not in the first-team squad) | choice for Rust |
| Career start | 1 world folder chosen and checked, 2 club, 3 coach, 4 start: world and sider.ini backed up into the career folder, pesdb copy made | commit 0696dcdd (karriere.py anlegen) |
| League names | fl26world.txt `league <n> cid=… name=…` by cid → wettbewerbsnamen.json (generated "X D1 Cup" entries ignored) → "England 1" | `pesdb::liga_meta`, fl/sim/config/wettbewerbsnamen.json |
| Countries | fixed key → country; FL leagues grouped by fl26world `country`, named after a fixed league with the same Competition.bin byte 3, else "Land der <tier-1 league>" | measured on MyWorld (Germany 176, England 16, Austria 232) |
| Pyramid | `above`/`cup` are league-space numbers (league lines, then CompetitionRegulation tournament ids); k = min(promote below, demote above), 3 when unknown; no exchange when 0 | fl26world.txt, sim.json pyramid count 3 |
| Big leagues, winter break | by key (ENGLAND/SPAIN/GERMANY/ITALY/FRANCE/PORTUGAL_D1_LEAGUE) and by country (England, Schottland, Türkei play through) | game.rs Career::new, world.rs make_fixtures |
| Crests and logos | index per world: world, active cpk.root folders, other livecpk folders, then assets/wappen; `flag/e_<id:06>_<var>.png` best of r_l, r_w_l, r_b_l, r, r_w, r_b, r_ll, r_w_ll, r_b_ll; `emblemLc/emb_<id:04>_l.png`; paths case-insensitive, .dds skipped | Übergabe Daten §5.1–5.3, fl/bilder.py |
| Player.bin body | height 216/8 bit +100, weight 256/7 bit +30, foot 514/1 bit (1 = left), weak foot use 454 and accuracy 462 (2 bit +1, 1..4) | flconv/plbin.py (measured there) |
| Robustheit (estimated) | 70 + 0.25·(Körpereinsatz − 70) + 0.25·(Ausdauer − 70) − 3 per year above 29, 1..99; injury resistance is not decoded | assets/regeln/profilseite.json, own estimate |
| Profile groups and DNA | groups Angriff, Technik, Bewegung, Kraft, Mentalität, Abwehr, Torwart; DNA = means of related values; stars = (value − 40) / 11, halves, 0.5..5; strengths/weaknesses = 3 highest/lowest values that count for the position | ui/seiten/Spieleruebersicht.qml |
| Profile knowledge | ours (Career::gehoert_uns: in the club incl. academy/U23, or loaned out by us) and knowledge level 4+: everything; level 2–3: the best share of values (scouting.toml einzelwerte_anteil), weak foot, form, fitness, character; level 5: injury resistance, special abilities, exact market value and wage (else 0.9–1.15 range) | scouting.toml, SpielerprofilSeite.qml |
| Similar players | same position, strength ±3, age ±3, closest 6 | Spieleruebersicht.qml hint text, numbers in profilseite.json |
| Cash calibration | a rolling year of months at scale 1; step ±25 % a month for the first 6 months, then ±10 % | own correction (was 3 months, ±10 %) |
| Minimum price | listed with a minimum price: weekly offers below it are not passed on | choice for Rust |
| Second teams | name rule (suffix II/B/2/Reserves/Amateure, prefix Jong; the rest = a club of the same country, club words like FC/SpVgg left out, else a unique name start) + assets/zweite_mannschaften.json | own rule; Team.bin has no parent-club field |
| Second team limits | first team 18–32, second team 16–30 players; age rule per country [older than, at most] DE [23, 3], NL [23, 3], ES [23, 5], PT [23, 6], EN [23, 5], else [23, 5] (estimated, sources per country still open); who played up is barred below for 3 days | assets/regeln/farm.toml, estimated |
| Second team budget | none of its own: wages count in the parent's wage bill and salary frame, its income, running costs and prizes go to the parent's cash as the post „Zweite Mannschaft“; no AI transfers of its own | own rule |
| Second team in the first team's league | at season end one tier down (first lower league by index), the next eligible club of that table takes the place | own rule |
| Farm teams | max 2; contract 2 seasons, 400 000 € × money scale a year; 4 loan places; partner yes 95 %; search 14 days, application chance 5 % a day × interest, at most 5, deadline 7 days, 3 negotiation rounds; board frame 0.6 % of revenue, above it trust 55 needed | assets/regeln/farm.toml, estimated |
| Applicant interest | strength gap 3–14 best, from 22 none; short of money ×1.5 (and asks 1.5× more); partner elsewhere ×0.4; other country ×0.35; experimental coach ×1.4; squad gap ×1.2 | farm.toml koop.interesse, estimated |
| Live pesdb writing | change found by a fingerprint (once a second), one write after 2 s calm; only changed files; backup (5 kept), atomic, read back, rollback; mirror into livecpk/PESM_<Verein> with SHA-256 check once that folder exists | own rule after the Python version |
| Ending FL27 | only FL_2027.exe whose WINEPREFIX is the Lutris prefix; SIGTERM, 10 s, then SIGKILL and wineserver -k; optional 30 s after the saved result | own rule |
| Pictures | world folder and active cpk.root folders, then the CPK archives in Data/ and download/ (later wins), then other livecpk folders; .png before .dds; at most 6 new photos per frame | fl/bilder.py index_bauen, adapters/pes/cpk.py |
| Team search | folded text (lower case, no accents, ß → ss), rank: name prefix, word prefix, name, short name, league, country; then strength | choice for Rust |

## Paket 2: competitions (fl/sim/uefa.py, comp.py, config/uefa.json, sim.json)
| Rule | Value | Source |
|---|---|---|
| Domestic cup | real FL world: the database's cups (name ends in CUP, not WORLD/NATIONS/ASIA/SUPER/CUSTOM/UEFA/CLUB, at least 8 teams from the leagues; Übergabe §3.4); built-in demo world: one per country with all clubs of its leagues; single matches; round 1: the strongest get byes up to the next power of two, then open draw | comp.py cup_draw; participants: choice (see below) |
| Cup days | Tuesdays (day % 7 == 3) between day 24 and 296 outside the international breaks, rounds spread, final on the last Tuesday | comp.py cup_days |
| Extra time and penalties | level knock-out match (single match, or second leg level on aggregate): extra time with rates × 0.33, then 5 penalties each at 0.76, sudden death with ± (p_h − 0.5)·0.1, p_h = 0.5 + 0.002·(strength difference) in 0.35..0.65 | match.py play, uefa.py _extra_time/_pens, sim.json et_factor/pens_strength |
| Europe | Champions League, Europa League, Conference League with 36 clubs each: league phase in four pots by strength, 8 games per club (two per pot, one home, one away), places 1-8 into the last 16, 9-24 play-offs (9-16 seeded against 17-24), last 16, quarter and semi-final over two legs (seeded club at home in the second leg), final on neutral ground | uefa.py |
| European days | CL Wednesday, EL/ECL Thursday: league phase days 46/60/81/95/116/123/130/158/172/186 (CL), play-offs 214/221, last 16 228/249, quarter 263/270, semi 277/284, final 298 (CL) / 299 | uefa.json days |
| No double booking | a match moves to the nearest free day (offsets 0, +1, −1, +2, −2, +3, −3) outside international breaks | uefa.py _place; test: 20 seasons |
| First season | real FL world: the database's entries of CL (2), EL (3), ECL (174), topped up to 36 | uefa.py _db_entries |
| Level knock-out match played in FL27 | the person picks the winner (extra time or penalties); without an answer CalcioM decides as for simulated matches | the person's decision 21.11.2026 |
| Places | holder to CL, EL winner to CL, ECL winner to EL, then quota per league (England, Spain, Italy, Germany 4/1/1, France 3/1/1, Portugal, Netherlands, Belgium 2/1/1, Turkey, Scotland, Greece, Denmark, Russia 1/1/1) from last season's final table; domestic cup winner of these countries into the EL; filled to 36 with the strongest clubs of the quota leagues, at most 7 per country in each competition | uefa.json quota/fill_cap, uefa.py _qualify |
| Prize money | start 18.6 M, win 2.1 M, draw 0.7 M, play-offs 1.0 M, last 16 11 M, quarter 12.5 M, semi 15 M, final 18.5 M, champion 6.5 M; EL × 0.25, ECL × 0.12 | uefa.json prizes |
| Reputation | per season: league title +2, domestic cup +1, CL win +3, EL +1.5, ECL +1, taking part CL +0.6, EL +0.3, ECL +0.2; at most +4 | ruf.json club |
| International breaks | days 35-41, 70-76, 105-111, 238-244 without club matches; on the first day up to 3 players per club with strength 76+ travel: fitness −8, injury risk 1.2 % | sim.json calendar.intl_breaks, intl_duty |
| Country points | per European match won 2, drawn 1, kept for 5 seasons | choice (simple country ranking, uefa.py view_coefficients) |
| FL27 look | the match's competition gives the tournament id: league key, cup key `<LAND>_D1_CUP` or `<LAND>_CUP`, UEFA_CHAMPIONS_LEAGUE, UEFA_EUROPE_LEAGUE, FL_UECL | uefa.json db_comps, Übergabe Daten §3.3 |

## Paket 3: the people side (Mechanik-Übergabe §1–19, numbers in assets/regeln/*)
All numbers are read from copies of the Python rule files (`core/**/*.toml`, `core/config/*.json`) in `assets/regeln/`, texts from `assets/texte/mensch.json`. Randomness is SHA-256 of a seed and purpose key (`mensch::zufall`). Every change of morale, relation, character, fans, media and board confidence goes through one write path with a reason (`Career::grund`, `klima`, `moral_wirkung`, `beziehung`, `verschieben`; guard test `jede_wirkung_hat_grund`). Core (own club) vs world: the own squad uses the core rules below; all other clubs keep the world's simple drift (welt.py _recover) and the AI boards' pressure (transfers.py), until 3r.

| Rule | Value | Source |
|---|---|---|
| Character values | 10 values 0–100 = 50 + age (loyalty ≥ 30 +10, ≥ 34 +5, ≤ 22 −5) + leadership +5 from 28 + fixed random ±12 (seed 2026, per player and value); status = percentile of the rating in the world, no random part. The game's four personality axes are not in Player.bin, so their weights count as neutral (s = 0) | persoenlichkeit.json model 4, §4 |
| Player types | first matching rule of persoenlichkeit.json typen (Vorbildprofi, Anführer, Kumpel, …) | §4 |
| Shifts by decisions | at most ±5 per value and month, ±40 in total; promise kept: loyalty +2, professionalism +1; broken: loyalty −3, agreeableness −1 | persoenlichkeit.json verschiebung, story charakter_bei |
| Hierarchy | value = leadership 0.30, status (rank in squad) 0.15, loyalty 0.10, belonging 0.05 (Bestand 50 / Neuzugang 0), play time 0.10, experience 0.10 (age 18 → 0 … 32 → 100); influence (+341) missing, weights normed to 1. Captain and vice: the two highest with leadership ≥ 55; up to 3 leaders among the first 10; others by play time ≥ 60 Stamm, ≥ 25 Rotation, else Perspektive (≤ 21) or Randfigur. Before the first match play time = 100 for the starting eleven, 40 for the others | hierarchie.json, §5; start value: choice |
| Morale | target = clamp(50 + Σ drivers, 5, 95), daily step 0.12; drivers spielzeit (last 5 club matches vs. role expectation ·30, −15…+8), erfolg, gehalt (rank of rating vs. rank of wage), vertrag (< 6 months), zusagen, gespraech (half-life 21 days, ±10), rolle, geruecht, freunde/isoliert, feinde, mentor, training, gruppe, form, verletzt, überlastet; negative drivers × sensitivity clamp(1 + 0.25·(T−50)/50 − 0.2·(P−50)/50, 0.7, 1.4); unhappy below 35, content again from 45 | entwicklung.toml [moral], regeln.py, §3 |
| Training driver | the six units of this build: Regeneration +1, Ausdauer (hard) −1, others 0; weekly sum × 0.5, ±3 | choice (einheiten.toml morale of the matching units) |
| Overload | there is no load model yet; Kondition = 95 − 0.45·load gives the zone „überlastet“ (load ≥ 70) at fitness ≤ 63.5 | choice from entwicklung.toml [kondition] |
| Self-confidence | per match played: win +2, loss −2.5, +1.5 per goal, (grade − 6)·4; daily 3 % back to 50 | [selbstvertrauen] |
| Form | overlay −6…+6, target (M−50)/50·2.5 + (SV−50)/50·3 − fitness term, step 0.35; arrow 0–4 with limits −3.5/−1.2/1.2/3.5 | [form], [vorschlag] |
| Transfer wish | readiness 10 + max(0, 50 − M)·1.6 + (1 − loyalty)·10·(1 or 0.3) + 10 per offer + 10 if < 12 months contract + 5 if isolated, step 0.1; wish from 70, withdrawn below 45 | [wechsel] |
| Conversations | start mood 50 + (M − 50)·0.35 + relation·0.25 − 10 if the topic is the cause; tone change by character (warm, sachlich, lob, bestimmt, hart, ausweichend) with trust damping; relation +0.35 per mood point; morale effect as driver „gespraech“ after 3 days. One answer per conversation in this build (Python: 1–2 questions) | gespraeche.toml, werte.py, §1.2 |
| Promises | Spielzeit 28 days (kept at a share ≥ 0.6), Rolle 28 (Stammspieler or higher), Vertrag 45, Gehalt 45 (+5 %), Freigabe 120; trust = 50 + Σ(+8 kept / −14 broken)·0.5^(age/120)·weight; relation +5/−9; a refused offer breaks a promised release (−6) | [versprechen], werte.py:95, §1.3 |
| Concerns | unhappy players and a weekly fixed chance per topic (spielzeit 0.8, rolle 0.7, formtief 0.6, lob 0.3, kritik 0.2, vertrag 0.4); at most 1 normal concern per day and 3 per week, important ones (unhappy, wish to leave) always; 21 days lock per topic and player | themen.toml, drosselung.toml |
| Discipline | chance per player and day 0.035·max(0, 1.5 − 1.5·Prof)·(1 + (50 − M)/100)·(1 + T − 0.5); 21 days lock; punishments [mood, relation, team spirit]: nachsicht [3, 1, 0], ermahnung [−2, −1, 1], geldstrafe [−5, −3, 3] (10 % of a monthly wage booked), bank [−8, −4, 5] (sits out the next match); equal treatment −6 team spirit. Incidents share the concern throttle (choice) | [disziplin], service.py:2167 |
| Dressing room | relations −100…100: start from age (±3 years +10), same position −8, both hot-headed −15, both vain −12, both agreeable +8, same arrival +8, neighbouring lines +5, same contract end +3, spread ±22 (nation not in the data); daily back 0.2 %, unhappy −0.05, mentor +0.15, match played together +1 win / +0.3 otherwise; conflict below −45 at 3 % a day, ends after 60 days, one level worse after 14 days | entwicklung.toml [kabine], konflikte.toml |
| Team spirit | target 50 + (mean morale − 50)·0.5 − 6 per open conflict + (captain's leadership − 50)·0.1 + (points per game − 1.5)·3, step 0.1 | kabine.py:76 |
| Conflict answers | chance schlichten 0.55, aussprache 0.6, kapitaen_bitten 0.45, strafe 0.3, laufen_lassen 0.1, −0.08 per level, +0.08 if the captain leads (≥ 70) and is not involved | konflikte.toml [ausgang] |
| Squad talk | lob +5, ermutigen +4, sachlich +1.5, kritik −3, hart −7 with the character factors; 30 % at once, 40 % after 3 days, self-confidence +0.8 per point, 7 days lock | [kabine] toene, kabine.py:87 |
| Club climate | fans, media image, board confidence 0–100 from 50; after every own match each driver of klima.json books its effect, at most ±8 per value and match; no drift back to 50 (as in Python) | story/klima.json, §11 |
| Relegation zone and the board | a club whose season goal already lies in the relegation zone loses no board confidence for being there (fans and media still react) | choice |
| Season goal | rank of squad strength (mean of the best 16 ratings), after 6 league games mixed 30 % with the table position; the strongest squad gets place 2, place 1 only with a lead of 4 rating points; refreshed after every league match; words: 1 Meisterschaft, up to a fifth of the league Europapokal, up to 5 above the bottom Mittelfeld, else Klassenerhalt | klima.json saisonziel; words: choice |
| Board consequences | ≥ 60: 70 % of the monthly surplus into the transfer budget (normal 50 %, below 35 only 20 %); below 25 an ultimatum of 60 days (not in the first 60 days of a job); after it, below 35: „Das Präsidium berät über deine Zukunft“ and the dismissal | vereinswelt.toml [budget] [praesidium], §12 |
| Dismissal with consequences | Python only reports it; here it is built (Paket 3i): severance half a year's wage, reputation −8, up to three offers, the career goes on elsewhere. The old firing at season end and the monthly pressure model for the own club are gone | §12, §20.2; choice |
| Coach contract | 18 months at the start; at its end the board extends by two years if confidence ≥ 45 (salary by the formula), otherwise the job ends without severance | laufbahn.toml [vertrag]; extension rule: choice |
| Coach reputation | 12 + 28·points quota + 22·table position + 5 per title (≤ 4) + 0.12 per month (≤ 14) + 14·club reputation, results half before 6 games; monthly | laufbahn.toml [ruf] |
| Dismissal risk (display) | 0.5·clamp((50 − V)/50) + 0.35·clamp((place − goal)/clubs/0.35) + 0.15 with ≤ 3 months contract | laufbahn/service.py:167 |
| Job offers | weekly: interest 0.4·reputation fit + 0.3·style fit + 0.2·success + 0.1·vacancy (+0.35 vacant, +0.1 contract ≤ 6 months); from 0.3, chance 0.35·interest²; at most 3 open, 14 days to answer, 60 days lock per club, 45 % become a rumour; salary 250 000 € + 7.5 M €·club reputation², × 0.7…1.3 by coach reputation | laufbahn.toml [interesse] [motor] [gehalt] |
| Negotiation (3e) | round model: characters pragmatiker/fair/hardliner (flex 14/9/4 %, concession 35/25/12 % per round, patience 100/90/70, cost 8/9/12, zone 0.25–0.80 / 0.20–0.70 / 0.10–0.55, last offer 0.65–0.97 / 0.45–0.92 / 0.12–0.60, thinking 5/7/10 %); pain limit target·(1 − flex·(1 + 0.1·relation − 0.2·position)); chance = zone · (0.5 + 0.5·patience/100) + tone/action + 3 %·relation + 4 %·position + scout bonus, max 95 %; tones herzlich +3 %, sachlich +1 %; actions erklären +4 %, Gefühl +5 %, Druck +12 % (risk 12 % + 8 %·max(0, −relation) + 8 % hardliner: patience −45, mood −8, 50 % breakdown); patience −(cost + tone)·(1 + 3·shortfall); not serious from 30 % under the pain limit (−35 patience, mood −3); last word under 25 patience; 8 rounds, 7 days, 2 × 2 days more time, cooling 14 / 21 days; sales with negated values | runden.py, vdialog.py, §7 |
| Club and player at the table | club character fixed per club (20 % hart, 20 % locker); position −0.7 when in debt, +1 for a regular not on the list; player side by persona (geldgierig hardliner, loyal and heimatverbunden pragmatiker, else fair), relation round((morale − 50)/25) in −2…2 | vdialog.py |
| Contract demand (3e) | own player: wish wage × clamp(1 + (Ego − 0.5)·0.3 + (50 − morale)/100·0.3 − (loyalty − 0.5)·0.15 − relation/100·0.1 + max(0, 50 − trust)/100·0.2 + (greed − 40)/100·0.15, 0.7, 1.6); others: wish·(1 + greed/100·0.15 − (attraction − 0.5)·0.2); wish wage = value·0.12·role (star 1.25, regular 1.0, rotation 0.85, prospect 0.7), ≤ 21 × 0.7, ≥ 30 × 1.15, min 20 000 €; wish length 3 / 5 from 24 / 3 from 29 / 1 from 32; verdict ≥ 1 accept, ≥ 0.88 counter, ≥ 0.65 refusal, below insulted (mood −15, relation −4); veto morale < 25 and ratio < 1.15; agreement mood +12, relation +6; renewal from 18 months before the end | werte.py:122/169, gespraeche.toml [vertrag], transfers.toml [verlaengerung] |
| Transfers (3f) | asking fee = value × (1.25 regular, 1.05 rotation, 0.85 reserve), × 0.7 at ≤ 6 months; price target × (1 + (0.5 − readiness)·0.3), × 1.06 upset, × 1.1 thin squad; offers for own players: at most 2 a week, rating ≥ 55, 85–125 % of value (+10 % with a wish to leave), 7 days; drama 3.5 % a day per talk (deadline × 4): agent delays, rival (4 days, outbid +10 % or give up), club pulls out; medical 7 % (+8 %/year from 31, +1.2 %/injury day, max 70 %), 28 % of findings fail, discount min(0.2, 0.04 + 0.5·risk) granted at 62 %; last-minute demand 9 % (deadline × 3); no answer = against the deal; aftermath +2 or −1.5, up to 2 rivals −2 (max 5); strike from readiness 60: unrest, training strike, match strike, team spirit −influence/20 per level; loans: fee 15 %, borrower pays 70 % of the wage | werte.py, verhandlung.py, drama.py, §9 |
| Scouting (3g) | scouts with Urteil, Potenzial, Menschenkenntnis, Netzwerk; salary 40 000 € + 2 500 € per point over 50; 2-year contract; 5 applicants a month; specialisation × 1.25 / × 0.85; reach by club level 1–5 (scouts 2–6, abroad 0.45–1.0); knowledge 4 per day × skill (home 0.9, video 0.5, live +9); report from 40, character from 60, exact from 85; half span 12 → 1 × (1 + 0.9·(0.5 − judgement)) × personality (vorsichtig 1.15, euphorisch 0.9), bias −2 / +3 / 0 / −1.5 × 0.4; potential × 1.4; stars from 55 to 90; trust start 50, after 8 weeks hit (±3) +3, miss −4; assignments 7–28 days, expenses 1 500 / 4 500 € a week; scout leading talks up to +12 % | scouting.toml, §10 |
| Club finances (3j) | power = (rank of squad strength among all clubs)³; capacity (8 000 + 55 000·power)·(1 + 0.12·(stadium − 1)); fan base 15 000 + 350 000·power; ticket (18 + 40·power) € × 0.8/1.0/1.25; attendance clamp(0.30 + 0.70·fans/100 − 0.10·(price − 1), 0.25, 1), cup early ≥ 0.52, top match ≥ 0.92, European KO 0.98, final 1.0; merchandising fan base·0.9 €·(1 + 0.25·(shop − 1))·(0.7 + 0.6·fans/100); TV (1.1 M + 21 M·power)/12·(1 + 0.2·table position); win bonus 50 000 + 400 000·power; running costs 60 000 + 1.5 M·power + (4 000 + 60 000·power) per facility level a month; sponsors shirt 840 000, outfitter 340 000, stadium name 500 000 × (1 + 25·power), 1–3 years, bonus 15 %, 2 offers per kind and month, 45 days; wages ≤ 62 % of projected income, debts ≤ 15 %; facilities stadium 2 M/10 months, shop 0.4 M/3, training 0.8 M/5 (+5 % development), youth 0.6 M/6 (+4), medical 0.5 M/4 (−4 % injuries), cost base·level² | vereinswelt.toml, §13 |
| Fans (3k) | effective = climate fans + transfer effect (±40·fee/income, ±15, 80 % a month) + economy mood (price target, 30 % a month, ±12) + fan actions (Fanfest 160 000 € +5/+2 90 days, Familientag 45 000 € +2/+1 30, Legendenspiel 320 000 € +6/+5 180, Wohltätigkeit 90 000 € +1/+4 60, Auswärtsfahrt 60 000 € +3/0 30; cost × (0.3 + 0.7·power)); protest value = fans − 4 per loss in a row from the 2nd − 8 if board < 30: < 42 murmurs, < 32 whistles, < 22 banners, < 12 stadium protest | §14, medien.toml [proteste] |
| Press (3l) | up to 4 questions by topic, tones per topic from medien.toml [toene], base effects per tone [media, board, fans, morale] with topic factors, journalist style × 1.4 / × 0.6 on media, attitude ±4; at most ±6 per target and conference; not attending −3/−1/−1; topic again only after 21 days; public promises „nicht verkaufen“ 120 days, „Saisonziel“ 300 days with the effects of gespraeche.toml [zusage_verein]; rumours from offers ≥ 2 M and made up (4 % a day, 21 days lock), denial checked after 45 days: refuted −6/−2/−1, confirmed +1, false rumour denied +1/+1 | §15, medien.toml |
| Inbox (3m) | Muss (conflict, discipline, medical, last-minute demand, rival, job, youth) 10 days, Soll 5 days, Info; flash for ultimatum, dismissal, medical, rival, strike, broken deal, wish to leave; a regular raises one level, ≥ 30 M raises; at most 3 flashes and 4 in full priority a day; outdated cases archived; overdue Muss escalates; weekly „Die Woche in der Welt“ | einstufung.toml, pflicht.toml, §16 |
| Assistant (3n) | from 3 own matches: in form (grades ≥ 7.0 or form ≥ 1) and < 40 % play → bring him; slump (≤ 6.0 or ≤ −1) and ≥ 50 % → bench him; talent ≤ 21 with potential ≥ 6 above and < 25 %; morale < 35 (< 25 very); injured ≥ 21 days; gaps (available GK < 2, DEF < 6, MID < 5, ATT < 3); 21 days lock; relation start 55, +2/−2 (urgent × 1.5), fades over 60 days, ±1 per win/loss of the last five; takes over Soll cases at the deadline with a calm answer, −1 relation to the player (max −3 in 30 days, ±30 % by his strength) | assistent/regeln.toml, cotrainer.py, §17 |
| Staff (3o) | roles trainer (area or all), assistant, doctor, physio, youth leader; salary base by role × (1 + 0.012·(skill − 50)); applicants per month; 10 % signing fee, 50 % of the rest as severance; training quality = trainers of an area 1.0, general 0.6, assistants 0.4; training level 1 + 9·quality/100 instead of 5; development ± 12 %, injuries ± 20 % by medical quality (doctor 0.35, physio 0.2) | personal.toml, entwicklung.toml [stab] |
| Load and phases (3o) | load b + unit − 12 %·b a day (free 0, recovery −1, light 2, normal 4, tactics 3, intensive 8; programme +2; match +10), „überlastet“ from 70; phases weekly: Formtief from causes ≥ 0.6 (grades < 5.5, 4 losses, morale < 35, broken promise in 21 days), Krise ≥ 1.6, Aufschwung at grades ≥ 7.2 or 5 wins; growth × 1.3 / 0.4 / 0.2, form +2 / −2.5 / −4, 8 weeks lock; leap ≤ 21, ≥ 75 % play, grade ≥ 6.8, morale ≥ 50, potential ≥ 4 above: 30 %, +3…5, 26 weeks lock; retraining 120 days + 6 per year over 21, other line × 1.8 | entwicklung.toml, entwicklung3, §18 |
| Academy (3p) | Akademie 16–17, U23 18–22, leaves in the year of the 23rd birthday; places 20 + 22 (+2 per youth level), at most 40; intake in July 3–6 aged 16–17, strength 34–52, potential 55–90 (+4 per level), 60 000 € for 3 years; weekly 0.9·(gap/15) + 0.5·play share + 0.12·(grade − 6) + 0.25 mentor + 0.2·training − 0.12 per year over 17, academy × 0.55, phases 6 % a week (3–6 weeks, 40 % crisis × 0.2, else × 1.8); promotion when at most 4 below the weakest regular; parents ask after 4 weeks without games | akademie.toml, vereinswelt.toml [jugend], §19 |
| Stories (3q) | the original's 78 rules and 486 texts (assets/story); active: zustand.json aktive_regeln and rules marked aktiv; after each own match: conditions, chance = base + type bonus + value bonus, fixed random; at most 6 messages and 2 per rule by priority; player concerns 2 a day, 6 a week, 7 days apart; effects at most ±10 per answer; follow-up rules after their days; unanswered after 14 days −3 mood | story.py, §2 |
| AI coaches (3r) | a type per coach (starr, reaktiv, experimentierfreudig, dogmatisch, pragmatisch): after a losing series of 6/3/4/8/3 games a defensive set-up for 8/3/2/10/4 weeks; interim coach replaced after 14 days | ki_trainer.toml |
| Why Sunderland was fired in season 1 (old build) | the FL-sim pressure model: +2 per place below the goal each month (up to +10), patience 40–68, plus a second firing at season end; the goal came from the strength rank alone | investigation 21.11.2026 |

## Paket 4: tactics and match day
| Rule | Value | Source |
|---|---|---|
| Opponent report | key figures from the abilities: pace of the wide players and centre-backs (Schnelligkeit, Antritt), aerial duels (Sprungkraft, Kopfball), technique in midfield (Ballkontrolle, Dribbling, flaches Passspiel), finishing (Abschluss, Schusskraft), set pieces (Standards, Effet); weakness when 3 below the own squad or centre-backs under 66, strength when 3 above; 3 key players; plan against the weakest area | gegner/bericht.py, gegner.toml |
| Tactic verdict | out of position (no level, other line), high pressing with mean stamina < 68, line ≥ 7 with centre-back pace < 66, short passing with midfield technique < 68, fast wide players (≥ 74) without width | taktikurteil.py / taktikeignung.py heuristics; thresholds 68: choice |
| Set pieces | penalty: Abschluss + Schusskraft, free kicks: Standards + Schusskraft, corners: Standards + Effet, best of the starting eleven unless chosen; captain chosen by the coach without threshold, +3 morale | choice after hierarchie.json |
| Saved tactics | up to 5 (formation and all settings) | choice |
| Analysis | grades per line, best and weakest player, shots and possession, one sentence on what to work on | choice after taktikanalyse.py |

## Second pass (21.11.2026)
| Rule | Value | Source |
|---|---|---|
| Money scale | revenue target = wages / quota, quota 0.62 → 0.57 by prestige (best 16 in 62…84 0.45, squad value 2 M…1 bn log 0.35, best player 65…92 0.2), corridor 0.55…0.62; all money values × scale (ticket price × √scale); start scale by bisection on the formula revenue (real fan mood, real league home games); then every month the income of the last three months as a year against the target, at most ±10 % a month, within 0.2…15 | budget_start.py / budget_start.toml, Vereinswelt._kalibrieren; monthly step: choice |
| Win bonus, upkeep | the league's win bonus is income; upkeep only for facility levels above 1 | vereinswelt/service.py, werte.py unterhalt_monat |
| Money check | big, average and weakest club over three seasons with the squad kept: cash change within one year's income (test `kasse_drei_saisons`; about 20 % surplus a year) | test |
| 16-bit competitions | Competition.bin id = byte 5 \| byte 7 << 8; CompetitionEntry.bin id = byte 0 \| byte 3 << 8 of the u32 at +8 (byte 1 = slot); CompetitionRegulation.bin keeps one byte at +8 (no layout gives more than 261 ids in the MegaPack); leagues with fewer than 10 clubs after removing clubs already placed are dropped | the person's report 21.11.2026, measured on _FL26Megapack |
| Manager as a person | start values 45/45/45/40/40/40/40, energy 70, stress 30, motivation 55; weekly stress +5, energy −4, motivation 10 % to the middle; stress ≥ 70 up to −8, energy < 30 up to −6 on all skills; 3 free-time slots a week (−10 stress, +10 energy: choice); style points from tones (manager.toml verhalten), fading 0.97 a week, recognisable from 3 points and 35 %; skills act ±10 % (choice) | manager.toml |
| Half-time talk | tone change per player (conversation engine) + fit to the score at the break (kabine.toml halbzeit_passung); team morale + mean/2; form tendency ±0.5 from mean ±4 | ansprachen.py:30 |
| Mind games | statement × favourite (≥ 2 better squad) / open / underdog → morale and media (kabine.toml mind) | ansprachen.py mind_wirkung |
| Youth network | per country: home 25, foreign 4, scout region 12; investment 0/6 000/18 000/45 000 € a month → +0/1.2/2.6/4.5 (+3 × youth leader), decay 0.6 without care; contacts 12 000 €, start 15, +3 a month, at most 6; talents visible from 20, recruit from 35: 28 days, 2 at once, chance 0.25 + 0.45·network + 0.15·contact + 0.2·(reputation − 50)/100, compensation 25 % (≥ 10 000 €) | jugend.toml, §19 |
| Application | four steps (project, style, question, reaction), answer effect −1…+1 by the board's persona, 33 % an incident −0.25; value 0.3·fit + 0.45·arguments + 0.25·reputation; offer from 0.62, waiting from 0.42 | laufbahn.toml [bewerbung] |
| AI rotation | coaches of types reaktiv, experimentierfreudig, pragmatisch rotate: a starter below fitness 55 loses 4 points in the line-up choice | ki_trainer.toml startelf, typ.*.rotiert |
| Roles and instructions | 21 roles by position (rollen.json), minimums from rollenwissen.json in the verdict; instructions Normal / Nach vorn / Zurückhaltend / Pressen, pressing +2 load a match, holding back −2 (choice) | config/taktik |
| Rule files at runtime | a copy in `~/.local/share/calciom/regeln/<name>.toml|json` overrides only the values it names; broken copies are ignored and listed | choice for Paket 6 |
| Diagnosis | duration of every day step (last 200), save size, counts | choice for Paket 6 |

## Paket 6
| Rule | Value | Source |
|---|---|---|
| Awards | per detailed league and the own league: player of the season (best average grade, ≥ 15 graded matches) and top scorer, into the career history | choice (simplest form of fl/sim/ehrungen.py) |
| Autosave | weekly (day % 7 == 2) and on leaving the game | existing |

## League rules (daten/ligaregeln.toml, 21.11.2026)
| Rule | Value | Source |
|---|---|---|
| File | `app/daten/ligaregeln.toml` built in; `<career>/ligaregeln.toml` wins; read on career start and load | Fabian's file |
| Country | `pes_land` = fl26world `country=`; else `schluessel` = Competition.bin key; else the country of the league below (`above=`); else the country name | request |
| League | `schluessel`, else `stufe` = tier (a base league without its own line: tier of the league below − 1) | request |
| Standard | relegated by size: ≥ 18 → 3, ≥ 14 → 2, else 1; promoted = relegated above; `runden` 2; no play-offs; no league below → 0 relegated | `[standard]` |
| Play-offs | the next `len(plaetze)` clubs after the direct ones (second teams skipped), best against worst, two legs, level → penalties 0.5 + 0.002·strength difference, clamped 0.35…0.65; a bye for the best when odd | choice; unknown formats as the request says |
| Relegation | upper club at `platz` against the play-off winner below, else the next club below after the direct ones; two legs | `[land.liga.relegation]` |
| Playout | `plaetze` paired from outside in, loser down, at most `absteiger` pairs | `[land.liga.playout]` |
| Split | after `nach_spieltagen` (the schedule keeps the days of the longest group free); points `behalten` / `halbiert_aufgerundet` ⌈p/2⌉ / `halbiert_abgerundet` ⌊p/2⌋ / `neu` 0; each group a double round robin; table lists groups in order | `[land.liga.split]` |
| Split only when | the groups cover places 1…n without gaps (last group stretched to n when it ends at `vereine`) | choice |
| Second teams | never into the first team's league or a higher tier; not above `hoechste_stufe` (0 or `erlaubt = false`: never up); no cups; recognition by `erkennung` | `[standard.zweite_mannschaften]`, `[land.zweite_mannschaften]` |
| UEFA places | in file order; a taken place or cup winner passes to the next league place; "ligapokal" → next league place (no league cup in the world); fill to 36 from the UEFA top leagues of the file, at most 7 per league | `[land.europa]`, `[uefa].weitergabe` |
| EPS | from season 2: the `leistungsplaetze_eps` (2) countries with the best European points per participant of the season get one more CL place | `[uefa]` |
| Title round | split groups that don't cover every club (Liguilla, Apertura/Clausura finals): after the normal schedule the best k (end of the last group, mostly 8) play a knock-out, two legs, best against worst; the winner is champion | `[land.liga.split]`, choice |
| Regional groups | a relegated club goes to the group it last played in, else its first team's, else by `assets/regionen.json`, else the group of the club it swaps with | choice; `region=` of fl26world is per country |
| Stadium picture | `SiderAddons/content/stadiums/map_teams.txt` → stadium folder → `common/render/thumbnail/stadium/*.dds` (first by name) | game content, read only |

## Posteingang (Layouts je Art)
- Art einer Nachricht: Feld `art`, sonst aus der Aktion (Scoutbericht ← Bericht, Angebot ← Transferangebot/Jobangebot/Vertragsende, Gespräch ← Beschwerde/Wahl, Talent ← Hochziehen) oder dem Bereich (Präsidium → Vorstand, Medien → Presse)
- Wochenbericht: einmal je Woche nach `ki_woche`, nur wenn die eigene Liga in den letzten 7 Tagen gespielt hat; Pfeil = Platz vor 7 Tagen minus Platz jetzt (nur wenn es damals schon Spiele gab); Überraschung = Sieger mit mehr als 4 Stärkepunkten Abstand nach unten; Verletzte der Konkurrenz = Vereine ±3 Plätze
- Moral-Verlauf: jede Woche die Moral jedes eigenen Spielers, die letzten 12 Werte
- Folge einer Antwort: Antwort auf einer Kopie gespielt; angezeigt ab 0,5 Punkten Änderung (Kasse ab 1.000 €), Moral einschließlich der noch ausstehenden Gesprächswirkung
- Passung im Scoutbericht: Stärke gegen den besten eigenen Spieler derselben Mannschaftsteil-Gruppe: ≥ 100 % Stammspieler, ≥ 93 % Rotation, sonst Kaderbreite

## Entwicklung je Fähigkeit (§18, eigener Kader und Akademie)
- Takt: einmal je Woche (Montag), nach der Akademiewoche; andere Vereine weiter mit der Saisonrechnung `develop_week`
- Wochenpunkte je Fähigkeit = Rate der Gruppe für das Alter (`kalibrierung.json`, Punkte je Jahr) × 0,9 / 52 × Talentfaktor (1 + 0,3 × (Potenzial − Gesamtwert − erwarteter Zuwachs bis 26), 0,3 … 2,5) × günstige Umstände (über 1 halb gezählt, höchstens 2,1) × Abflachen in den letzten 5 Punkten vor der Obergrenze; bei negativer Rate × Phasenfaktor Altern × Merkmale; dazu Abbau ab 29 (Kurve 29: 0, 31: 1,4, 33: 2,5, 36: 3,6, 40: 4,5 Punkte je Jahr × Gruppengewicht Tempo 1,0, Kraft 0,8, Technik 0,55, Kopf 0,45, Torwart 0,7; Torhüter 4 Jahre später)
- Umstände: Einsatzquote 0,55 … 1,45 (Anteil der letzten 6 Spiele mit mindestens halber Spielzeit), Noten ×1,1 / ×0,9, Plan-Intensität 0,8 / 1,0 / 1,2 × Wiederholung, Extraschicht ×1,2, Trainerqualität ±12 %, Mentor 1 + 0,25 × Passung, Moral ×0,8 / ×1,1, Merkmale, Gespräche ±1 % je Punkt (±10), Vertragsfrust bis −10 %, Teamgeist ±0,2 % je Punkt über/unter 50, Rückkehr nach Verletzung 21 Tage ×0,5, Trainingsanlagen; Phase ×1,8 Reifung, 0,9 Stagnation, 1,3 Aufschwung, 0,4 Formtief, 0,2 Krise
- Fokus: ×1,7 auf die Fähigkeiten der Gruppe, ×0,95 auf die anderen (wirkt nur, solange die günstigen Umstände nicht schon an der Grenze 2,1 sind)
- Phasen: Formtief ab Anlasssumme 0,6 oder Notenschnitt unter 5,5, 4 Niederlagen bei Einsätzen, Moral unter 35 zwei Wochen; Krise ab 1,6 oder Moral unter 22; Aufschwung bei Notenschnitt ≥ 7,2, 5 Siegen oder Lob; Dauer und Tiefe aus `entwicklung3.toml [phase]` × Merkmale; Sperre 8 Wochen nach dem Ende; Anlässe wirken mit Halbwertszeit 4 Wochen
- Sprung: ≤ 21 (Frühreif 22), Startelfquote ≥ 75 %, Note ≥ 6,8, Moral ≥ 50, Potenzial ≥ Gesamtwert + 4: 30 % × Merkmale × Notenfaktor, +3 … +5 auf die (bis zu 6) wichtigsten Fähigkeiten, Sperre 26 Wochen
- Mentor: Passung aus Professionalität, Führung, Teamgeist, Verträglichkeit, Erfahrung, Position und Ego (regeln.mentor_passung); Angleichen 0,07 je Woche an bis zu 4 Fähigkeiten mit Abstand ≥ 8; Spezialfähigkeit 12 % je Monat × Passung
- Wege: Chance 0,35 je Woche × Einsatzanteil (voll ab 0,5) + 0,15 Mentor hat den Weg + 0,1 passender Fokus ± Moral; drei Treffer = erreicht; Verlust ab 33 mit 4 % je Woche, wenn eine Voraussetzung mehr als 6 unter dem Wert liegt, oder nach 26 Wochen ohne Bedarf der Position mit 6,5 % je Woche
- Lange Verletzung: ab 28 Tagen Chance Tage × 1 % (höchstens 90 %, ab 30 Jahren +2 % je Jahr) auf −1 an einem Körperwert, ab 120 Tagen −2
- Erwartung (Pfeil): Wochenpunkte dieser Woche × Gewichte des Gesamtwertmodells × 52; ab +2 steil hoch, ab +0,5 hoch, über −0,5 gleich, über −2 runter

## Live-Ansicht eines Spiels in FL27 (Punkt 4, Coaching Zone als Bibliothek)
- Quelle: der Statistik-Block im Speicher von FL_2027.exe, gelesen über `pesm_coaching_zone::stats` (Blocksuche: drei Datensätze mit Heim-Kopf in Folge, Gast-Kopf eine Seite weiter, plausible Spieler-ID; obere 16 Bit des Kopfs beliebig). Alle 2 Sekunden ein Schnappschuss; CalcioM rechnet Tore, Karten, Minuten und Wechsel für die Karriere weiter selbst (`fl27::doc`), die Live-Ansicht kommt aus `live::Tracker` der Coaching Zone
- Anzeigetafel: Stand, Minute und Abschnitt, Balken = Anteil der Ballkontakte im laufenden Abschnitt (FL27 speichert keinen Ballbesitz)
- Note je Spieler: die Formel der Coaching Zone (6,0 + 1,0 je Tor + 0,6 je Vorlage + 0,12 je Schuss aufs Tor + 0,004 je angekommenem Pass + 0,06 je gewonnenem Zweikampf + 0,05 je abgefangenem Pass + 0,3 je Parade − 0,3 Gelb − 1,5 Rot − 0,1 je Foul − 0,8 je Eigentor, 3,0 … 10,0)
- Kondition in der Liste: der Stand vor dem Anpfiff (FL27s Live-Wert wird noch nicht gelesen); Formpfeil aus der PES-M-Welt
- Wechselideen (eigene Elf, `live_spiel.toml`): Gelb ab Minute 55 („noch ein Foul, und ihr spielt zu zehnt“), Note unter 6,0 ab Minute 55, Formpfeil 0 oder 1 ab Minute 65; je Idee der Bankspieler mit der höchsten Stärke auf der Position des Ausgewechselten (`World::eff`, mit Positionsabzug), Torhüter ausgenommen; höchstens 3 Ideen und nie mehr als Wechsel übrig (5, wenn das Spiel keine Grenze nennt)
- Einblendung: neues Tor, Eigentor, Gelb, Gelb-Rot, Rot oder verschossener Elfmeter → Karte für 5 Sekunden mit Klang (Tor: Jubel, Karte: Pfiff, Elfmeter: Raunen); wer mitten im Spiel einsteigt, bekommt für die alten Ereignisse keine Einblendung
- Während die Live-Ansicht offen ist, nimmt die Seite darunter keine Klicks an; „Mitlesen beenden“ steht auf der Anzeigetafel

## Inhaltspool im Posteingang (Punkt 5, Teil 1.2; pesm/core/inhalt, backend/vorgang.py)
- Persönlichkeitstyp: erst aus den Merkmalen (launisch, hitzig, wechselfreudig, sensibel, ehrgeizig, loyal, gelassen, in dieser Reihenfolge), sonst aus den Charakterwerten (Temperament ≥ 0,8 hitzig, Loyalität ≤ 0,25 wechselfreudig, Nerven ≤ 0,25 sensibel, Ehrgeiz ≥ 0,8 ehrgeizig, Loyalität ≥ 0,8 loyal, Nerven ≥ 0,8 und Temperament ≤ 0,3 gelassen), sonst neutral; er färbt die Bausteine der Texte und bestimmt, wie eine Antwort ankommt
- Antworten: 4–6 je Runde aus dem Thema, je eine lösende (gewichtete Wirkung ≥ 2), verschlimmernde (≤ −2) und neutrale, verschiedene Strategien, höchstens zwei gleiche Töne; Reihenfolge Zusagen, Erklären, Vertrösten, Ablehnen, Ausgleich, Nachfragen, Hart bleiben, Verständnis, Abgeben
- Reaktion: Klasse der Antwort, verschoben nach Typ (z. B. hitzig: harter Ton −1, Verständnis +1; ehrgeizig: Zusage/Ausgleich +1, Ablehnen −1); Launische kippen mit 30 % je Richtung
- Wirkungen: Moral × 0,5 (halbe Gesprächswirkung), Wechselwunsch über die Moral (÷ 1,6 × 0,5), Beziehung voll, Zusage = Versprechen mit CalcioMs Fristen, Kosten als Buchung, Fans/Medien/Vorstand; nach drei Antworten eines Spielers in sieben Tagen wirkt nichts mehr auf die Stimmung (die Sache selbst schon). Die Wellen eines Gesprächs (Freunde, Führungsspieler, Gleichbehandlung) gelten wie bei den übrigen Gesprächen
- Von selbst: je Tag höchstens 2 neue, nichts Neues ab 8 offenen, eins je Spieler; je Thema Wurf mit `haeufigkeit`, Sperre `sperre_tage` je Spieler und kaderweit; Drosselung: Wichtiges immer, sonst 1 am Tag und 3 in 7 Tagen zusammen mit den übrigen Anliegen, Rauschen 1 in 7 Tagen
- Wiederholschutz: je Thema und Spieler werden die zuletzt gezeigten Eröffnungen gemerkt (höchstens 12, nie alle); erst kommen die Eröffnungen des eigenen Typs, dann die allgemeinen, erst danach wiederholt sich etwas
- Ketten (`posteingang.toml [[kette]]`): Der Neue findet keinen Anschluss → nach 21 Tagen Nachfrage (Mentor) → nach 14 Tagen Szene in der Kabine; Heimweh und Müde Beine → nach 21 Tagen die gute oder die schlechte Szene, je nach Ausgang des Gesprächs
- Absender: Präsidium, Presse, Berater, Fanvertretung, Sportliche Leitung … sind Personen aus `absender.json` (je Verein fest gewählt), gezeigt mit neutraler Amtsbezeichnung
- Wochenschau (montags): bis zu zwei Aussagen des Trainers aus der Pressekonferenz der letzten 7 Tage, jeweils mit der Bilanz seither (mehr Niederlagen als Siege: „Die Worte verpuffen“, sonst „Die Mannschaft liefert“, ohne Spiel: „Das nächste Spiel wird zeigen …“), aufgefüllt mit den Schlagzeilen des eigenen Vereins bis 4

## Spielgefühl, Teil 4 (1, 2, 9) und Verlauf (Teil 3; spielgefuehl.rs, assets/regeln/spielgefuehl.toml)
- Was jetzt? Reihenfolge: ohne Verein → Pressekonferenz → Muss heute fällig → Spiel heute → Muss mit der nächsten Frist → Verhandlungen mit Gegenangebot → Soll mit der nächsten Frist → Transferfenster schließt in höchstens 5 Tagen → nächstes Spiel → „Alles erledigt“. Pressekonferenz und Muss heute fällig können nicht warten: der Weiter-Knopf führt dorthin
- Saisonreise: Etappen aus dem eigenen Ligaspielplan (erster Spieltag, mittlerer Spieltag, Beginn des Winterfensters, letzter Spieltag); Saisonziel-Balken voll, wenn der Zielplatz erreicht ist, sonst (Mannschaften − Platz) ÷ (Mannschaften − Zielplatz)
- Geführte erste Woche: fünf Schritte, je +1 Vorstandsvertrauen, alle fünf: +2 Fans und Vorstand; 21 Tage ab Antritt, ausblendbar
- Verlauf: montags ein Punkt (Platz, Stärke der Elf, Teamgeist, Moral im Schnitt, Kontostand), höchstens 160 Punkte- Saison (Analyse, Reiter Saison): Punkte je Spiel heim und auswärts, Serien (Siege in Folge, ohne Niederlage, ohne Sieg ab zwei), Bilanz gegen jeden Gegner über alle Saisons (Derbys zuerst), beste und schwächste eigene Spieler nach Notenschnitt ab drei Noten
- Saison je Wettbewerb (Profil, Leistung): nur eigene Spieler, gezählt bei der eigenen Partie; Tore des Gegners als Eigentor zählen nur für die Mannschaft
- „Weiter“ führt einmal zur Sache, die nicht warten kann; steht man schon dort, beginnt der nächste Tag wie gewohnt (eine bewusst ausgelassene Pressekonferenz zählt als nicht erschienen)

## Vertragsklauseln (Punkt 6; klauseln.rs, assets/regeln/klauseln.toml)
- Wert für den Spieler je Jahr: Torprämie × erwartete Tore der Position (ST 14 … TW 0) + Einsatzprämie × 28 + bei einer Ausstiegsklausel Gehalt × 0,04 × (0,5 + Ehrgeiz), halb so viel über 1,5 × Marktwert; ohne Klausel, die er will (Ego ≥ 70 oder selbstbewusst): −0,04 × Gehalt; Gehaltssteigerung: Gehalt × Satz × (Laufzeit − 1) ÷ 2
- Kosten für einen Käufer: Weiterverkauf % × Marktwert × 0,5; Rückkauf (1,3 × Marktwert − Rückkaufpreis, nicht unter 0) × 0,3 + 5 % Marktwert
- Im Rundenmodell: Angebot = Gehalt + Wert (bzw. Ablöse + Kosten beim Käufer); Gegenangebote und Einigung werden wieder ohne den Klauselwert gerechnet
- Wirkung: Prämien nach jedem eigenen Spiel gebucht; Steigerung zu jeder Saison der Laufzeit; Ausstiegsklausel: im offenen Fenster je Woche 20 %, wenn sie ≤ 1,6 × Marktwert ist und ein Verein mit höherer Stärke und dem Geld da ist; Weiterverkauf: beim nächsten Verkauf (auch zwischen fremden Vereinen) einmal gezahlt; Rückkauf: zwei Saisons lang zum vereinbarten Preis, der Spieler bekommt zwei Jahre und ein faires Gehalt

## Karrieren verwalten (Punkt 6)
- Wöchentliches Speichern (Tag 2 jeder Woche) legt vorher eine Sicherung an; drei je Karriere, die älteste fällt weg
- Löschen verschiebt Spielstand, Karte und Karriereordner nach karrieren/papierkorb/<Name>-<Zeitstempel>


## Ehrungen übers Jahr (Punkt 6; ehrungen.rs, Original fl/sim/ehrungen.py)
- Elf der Woche (montags, eigene Liga, Noten der Ligaspiele der letzten 7 Tage, je Spieler die beste): 1 Torwart, 4 Abwehr, 3 Mittelfeld, 3 Angriff nach Note, höchstens 3 je Verein; eigene Spieler +1,5 Moral; bei der 3., 5., 8., 12. Nominierung „Dauergast“
- Spieler des Monats: Notenschnitt + 0,35 × Tore, ab 3 Spielen im Monat; Trainer des Monats: beste Punkte je Spiel im Monat, ab 2 Spielen; eigene Preisträger +4 Moral
- Saisonende: Talent der Saison (≤ 21 Jahre, ab 18 Noten), Fairplay der Liga (wenigste Gelbe Karten), Ballon d'Or (Welt, ab 15 Noten und Stärke 78; Notenschnitt + 0,35 × Tore je Spiel + 0,1 × Titelpunkte (Liga 2, Pokal 1) + Stärke ÷ 100)
