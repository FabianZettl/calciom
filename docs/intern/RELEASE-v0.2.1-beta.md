# PES M v0.2.1-beta

Direct start into your FL27 match, and PES M now speaks Spanish and French.

## New

**Direct start**
- FL27 boots straight into your match: no main menu, no team selection, the right kits. You pick your controller side, press Kick-off yourself, and the half-time menu stays for substitutions and tactics. After the final whistle FL27 skips the result screens and PES M books the result.
- The first time you open this version, PES M asks once whether you want it. Say yes and it sets everything up on the spot: it builds the files it needs from your own FL27 installation (no game files are shipped), installs its Sider module, allows the match launch and switches on *Quit FL27 after full time*. A short guide shows what to expect.
- If you use GigaPack, PES M switches off the statistics file in `modules/fl26match.lua` while direct start is on: its background thread made FL27 crash at random, mostly at half-time or at the end of a match. The mod keeps working, the original is kept next to it and comes back when you switch direct start off. The question tells you about this before anything changes.
- Set the match length on the matchday page.
- One match per FL27 run: PES M closes FL27 after the final whistle and starts it again straight into the next match, with all changes from PES M. A quick start that keeps FL27 open between matches is in testing for the next release.
- Change it any time under Settings › Football Life 2027 › Direct start.

**Spanish and French**
- Four languages now: English (default), German, Spanish and French. Pick one under Settings › Display; it switches instantly.
- Translated: the whole interface, menus, inbox, press, player profile and the manual.

## Changed
- The old intro film replacement from earlier development builds is gone; existing copies are removed.

## Known gaps
- **Direct start on Windows is not tested yet.** It is tested on Linux. Please report how it goes in #bug-reports.
- The entrance animation before kick-off is not there yet.
- In Spanish and French the long story texts stay English for now, and dates still use the German pattern.
