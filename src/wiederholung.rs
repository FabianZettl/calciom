//! No-repeat check (Fabian 23.11.: "the lucky-socks story four times a season — every conversation, report and press
//! question must feel new"). A long run collects every inbox message, news item and press question; names and numbers
//! are masked, and what is left (the template) is counted per kind.
#[cfg(test)]
mod tests {
    use crate::game::*;
    /// The template of a text: names of players, clubs and the coach and every number masked.
    pub fn vorlage(c: &Career, t: &str) -> String {
        let mut s = t.to_string();
        let mut namen: Vec<&str> = c.world.clubs.iter().map(|k| k.name.as_str()).chain(c.world.players.iter().filter(|p| p.team >= 0).map(|p| p.name.as_str())).filter(|n| n.len() > 2).collect();
        namen.sort_by_key(|n| std::cmp::Reverse(n.len()));
        for n in namen { if s.contains(n) { s = s.replace(n, "·"); } }
        for p in c.world.players.iter().filter(|p| p.team >= 0) { if let Some(l) = p.name.split_whitespace().last().filter(|l| l.len() > 3) { s = s.replace(l, "·"); } }
        s.chars().map(|ch| if ch.is_ascii_digit() { '#' } else { ch }).collect()
    }
    /// Run with: cargo test --release wiederholung -- --ignored --nocapture
    #[test]
    #[ignore]
    fn wiederholungen_messen() {
        let w = crate::world::testwelt(9);
        let mut c = Career::new(w, 3, "Test");
        let mut post: Vec<String> = vec![];
        let mut news: Vec<String> = vec![];
        let mut pk: Vec<String> = vec![];
        let mut letzte_id = 0u64;
        let mut news_gesehen: std::collections::HashSet<(u32, u32, String)> = Default::default();
        let tage = std::env::var("PESM_WDH_SAISONS").ok().and_then(|x| x.parse::<u32>().ok()).unwrap_or(2) * 365; // roh
        for _ in 0..tage {
            if c.today_match().is_some() { let r = c.prepare_match(None); c.apply_own_match(r); }
            while let Some(q) = c.press.clone() { pk.push(vorlage(&c, &q.q)); for r in &q.rest { pk.push(vorlage(&c, &r.q)); } c.answer_press(Some(0)); if c.press.as_ref().map_or(false, |x| x.q == q.q) { break; } }
            for m in c.inbox.iter().filter(|m| m.id > letzte_id) { post.push(format!("{} || {}", vorlage(&c, &m.title), vorlage(&c, &m.body.chars().take(120).collect::<String>()))); }
            letzte_id = c.inbox.iter().map(|m| m.id).max().unwrap_or(letzte_id).max(letzte_id);
            for x in c.feed.iter() { if news_gesehen.insert((x.season, x.day, x.head.clone())) { news.push(vorlage(&c, &x.head)); } }
            c.ultimatum = None;
            c.fired = false;
            if c.end_day().is_err() { break; }
            // answer what waits so the stories run on
            for i in 0..c.inbox.len() { if !c.inbox[i].done && matches!(c.inbox[i].action, Action::Wahl { .. }) { c.answer(i, true, 0); } }
        }
        for (art, v) in [("post", &post), ("news", &news), ("pk", &pk)] {
            let mut z: std::collections::HashMap<&String, usize> = Default::default();
            for x in v.iter() { *z.entry(x).or_default() += 1; }
            let mut l: Vec<(&&String, &usize)> = z.iter().collect();
            l.sort_by(|a, b| b.1.cmp(a.1));
            let doppelt: usize = l.iter().map(|x| x.1 - 1).sum();
            eprintln!("== {art}: {} texts, {} templates, {} repeats ({:.0} %)", v.len(), l.len(), doppelt, 100.0 * doppelt as f32 / v.len().max(1) as f32);
            // the same without the regular reports (weekly, monthly) and the test's endless ultimatum: the messages that
            // are meant to be one-offs
            let bericht = |t: &str| ["Trainingsbericht", "Die Woche in der Liga", "Elf der Woche", "Akademie-Bericht", "Die Trikots des Monats", "Die Wochenschau", "Ultimatum", "Training report", "Team of the week"].iter().any(|b| t.starts_with(b)); // roh
            if art == "post" { let r: Vec<&(&&String, &usize)> = l.iter().filter(|x| !bericht(x.0)).collect(); let n: usize = r.iter().map(|x| *x.1).sum(); let d: usize = r.iter().map(|x| x.1 - 1).sum(); eprintln!("== post ohne Berichte: {n} texts, {} templates, {d} repeats ({:.0} %)", r.len(), 100.0 * d as f32 / n.max(1) as f32); } // roh
            for (t, n) in l.iter().take(25) { if **n > 1 { eprintln!("{n:>4}× {}", t.chars().take(150).collect::<String>()); } }
        }
    }
}
