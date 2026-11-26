# PES M v0.2.0-beta

Your own trophy ceremony, real player looks from FL27, and a sharper picture on every screen.

## New

**Trophy ceremony**
- After you win a final, PES M shows its own ceremony: the trophy and confetti, your crest next to the beaten finalist's, the score including penalties, and every scorer of both sides.

**Player looks**
- Skin and hair colour, bald heads, afros and beards are read from FL27's PlayerAppearance.bin. A new player wears his face donor's look.
- A player without a photo gets a passport photo in his club's kit, rendered once and kept.
- The small figures in lists use the real look too.

## Fixes
- Sharper text at every resolution. Text used to be drawn at the layout size and stretched to the window (×1.08 at 1080p, ×1.44 at 1440p). It's now drawn at its real pixel size.
- High-DPI support: Windows at 150 % or 200 % display scaling no longer stretches a small window (not yet tested on a real Windows PC). pesm.log records the display's DPI factor for bug reports.

## Coming next
- Direct start (switch in Settings › Football Life 2027, match length on the matchday page) into your match from PES M works in the development build. It needs game files that can't be shipped here, so it comes in a later release once PES M builds them from your own FL27 installation.
