# PES M v0.1.3-beta

A big matchday and transfer update. Matches now feel like events, from the duel screen before kick-off to the matchday round-up after the final whistle. The transfer market runs on clubs that think like real clubs.

## New

**Matchday**
- Duel screen before kick-off: both coaches, form, last five results, key players, head-to-head record, odds, and voices from a TV pundit, the press and fans.
- Opponent report and opponent view: their probable eleven with faces and ratings, danger men, their last five results and your assistant's read in his own words, with up to five suggestions you can apply in one click.
- Live match screen: ticker lines in club colours, a touchline menu, a live table, other matches of the round, and six shouts.
- After the match: a matchday view with the table and its movement, today's results, player of the match, top ratings, and voices from pundits, press and fans.
- Mind games about the actual opponent: their coach and their best player are named in the questions and answers.

**Tactics**
- Tactics engine part 1: eight playing ideas, nine sliders in football words and four switches. Each one is translated into FL27's values, with a reason.
- Formations with and without the ball next to the kick-off shape. They are written into FL27's three formation sets.
- Advanced instructions: two for attack and two for defence, each with its own effect in the simulation.
- Bench and reserves shown apart, grouped by line, as cards like on the pitch.

**Transfers**
- Transfer AI part 1: clubs shop in realistic leagues and countries. The search radius matches their size and money.
- Transfer lock: new signings and loan players can't be sold before the next window. This holds for every club, with the reason and the date.
- Negotiations: fees in two or three instalments, and an add-on of 10 or 20 % after 25 competitive matches.
- Talks end with a message when the player signs for another club in the meantime.
- Deadlines never run past the transfer window. On deadline day, answers come the same day.
- Transfers screen with clear tabs (Search, Transfer list, Loan list, Free agents, Search profiles) and one row of dropdown filters.
- The world loan list fills up properly: young players behind the starting XI and surplus players up to 27.

**Scouting**
- Free scouts as cards with face, region and abilities.
- Searches by player profile, such as a number ten with many assists or a ball-winning centre-back. Or search for one like a model player you pick.

**Players and world**
- Real player data from SoFIFA at career start: potential, wage, market value, contract end, release clause and loans, plus growth and emotion profiles.
- Press answers about one of your players change his morale and form, depending on your tone and his character.
- Position tags in line colours across all player lists, and own players marked in rankings.
- Right-click menu on clubs: profile, squad, fixtures and opponent report.
- Young new players get faces without beards. The youth intake follows realistic talent numbers.
- Loans: the player page names the exact return day.
- Pictures from free sources where the game has none: stadium photos (Wikidata, Wikimedia Commons), press outlet logos and reporter photos, credited, loaded in the background and cached.
- Journalists have beats (transfers, tactics, club insider, opinion, player stories) and ask the questions of their beat.
- Music: a volume slider in the settings, a next-song button, and the current song shown in the sidebar.

## Fixes
- A player you sold could be sold on by the AI club in the same window.
- Talks could still succeed after the player had already signed elsewhere.
- Negotiation deadlines could run past the end of the window.
- Tactic checks read some switches the wrong way round.
- The statistics header could overlap the filter pills.
- Data packages only load the leagues you need.
- Imported saves get detailed match numbers from the season totals.

## Faster
- Transfer window days take about 1.7 s instead of minutes.
- Under the hood, the UI code is split into one file per page.
