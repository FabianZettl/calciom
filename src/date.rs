// Game calendar. Like the original FL world (pesm/fl/sim/kalender.py, sim.json "calendar"): a season has
// 365 days, day 0 is 1 August and counts as a Saturday (weekday0 = 5). `abs` = season * 365 + day.
pub const START_YEAR: i32 = 2026;
pub const SEASON_DAYS: u32 = 365;
fn wd_t() -> [&'static str; 7] { [crate::inhalt::t("kalender.wt.0"), crate::inhalt::t("kalender.wt.1"), crate::inhalt::t("kalender.wt.2"), crate::inhalt::t("kalender.wt.3"), crate::inhalt::t("kalender.wt.4"), crate::inhalt::t("kalender.wt.5"), crate::inhalt::t("kalender.wt.6")] }
fn wd_long_t() -> [&'static str; 7] { [crate::inhalt::t("training.tag.0"), crate::inhalt::t("training.tag.1"), crate::inhalt::t("training.tag.2"), crate::inhalt::t("training.tag.3"), crate::inhalt::t("training.tag.4"), crate::inhalt::t("training.tag.5"), crate::inhalt::t("training.tag.6")] }
fn mon_t() -> [&'static str; 12] { [crate::inhalt::t("r.be.dienste.monat.0"), crate::inhalt::t("r.be.dienste.monat.1"), crate::inhalt::t("kalender.monat.2"), crate::inhalt::t("r.be.dienste.monat.3"), crate::inhalt::t("kalender.monat.4"), crate::inhalt::t("kalender.monat.5"), crate::inhalt::t("kalender.monat.6"), crate::inhalt::t("r.be.dienste.monat.7"), crate::inhalt::t("r.be.dienste.monat.8"), crate::inhalt::t("r.be.dienste.monat.9"), crate::inhalt::t("r.be.dienste.monat.10"), crate::inhalt::t("r.be.dienste.monat.11")] }
const MLEN: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// (year, month 1-12, day 1-31) of a season day; shortcut: no 29 Feb (seasons are 365 days, as in the original).
pub fn ymd(season: u32, day: u32) -> (i32, u32, u32) {
    let mut y = START_YEAR + season as i32;
    let (mut m, mut d) = (8u32, 1u32);
    let mut left = day;
    loop {
        let rest = MLEN[m as usize - 1] - d + 1;
        if left < rest { d += left; break; }
        left -= rest;
        d = 1;
        m += 1;
        if m > 12 { m = 1; y += 1; }
    }
    (y, m, d)
}
/// 0 = Monday
pub fn weekday(day: u32) -> usize { ((5 + day) % 7) as usize }
pub fn wd(day: u32) -> &'static str { wd_t()[weekday(day)] }
pub fn wd_long(day: u32) -> &'static str { wd_long_t()[weekday(day)] }
pub fn long(season: u32, day: u32) -> String {
    let (y, m, d) = ymd(season, day);
    format!("{}, {}. {} {}", wd(day), d, mon_t()[m as usize - 1], y)
}
pub fn short(season: u32, day: u32) -> String {
    let (y, m, d) = ymd(season, day);
    format!("{:02}.{:02}.{}", d, m, y)
}
pub fn month_name(m: u32) -> &'static str { mon_t()[m as usize - 1] }
pub fn season_label(season: u32) -> String {
    let y = START_YEAR + season as i32;
    format!("{}/{:02}", y, (y + 1) % 100)
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn calendar() {
        assert_eq!(ymd(0, 0), (2026, 8, 1));
        assert_eq!(wd(0), "Sa"); // 1 Aug 2026 is a Saturday (original screenshot)
        assert_eq!(ymd(0, 153), (2027, 1, 1));
        assert_eq!(ymd(0, 364), (2027, 7, 31));
        assert_eq!(ymd(1, 0), (2027, 8, 1));
    }
}
