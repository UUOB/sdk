//! Small date helpers shared by apps, the same as ui/dates.js. Dates are
//! local "YYYY-MM-DD" strings, times "HH:MM": simple, sortable, timezone-free.
//! Each function that needs "today" has a `_on` twin that is given it, for tests.

use jiff::civil::Date;
use jiff::{ToSpan, Zoned};
use regex_lite::Regex;
use serde::Serialize;
use std::sync::LazyLock;

pub fn pad(n: i64) -> String {
    format!("{n:02}")
}

pub fn iso(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

/// "YYYY-MM-DD", or None.
pub fn parse(s: &str) -> Option<Date> {
    let mut it = s.split('-').map(|p| p.parse::<i64>().ok());
    let (y, m, d) = (it.next()??, it.next()??, it.next()??);
    from_parts(y, m, d)
}

/// What JS's `new Date(y, m - 1, d)` gives: a month or day past its end runs on.
fn from_parts(y: i64, m: i64, d: i64) -> Option<Date> {
    let first = Date::new(i16::try_from(y).ok()?, 1, 1).ok()?;
    first.checked_add((m - 1).months()).ok()?.checked_add((d - 1).days()).ok()
}

/// This device's date and time now.
fn now() -> Zoned {
    Zoned::now()
}

pub fn today() -> String {
    iso(now().date())
}

/// "HH:MM" now, on this device.
pub fn now_time() -> String {
    let n = now();
    format!("{:02}:{:02}", n.hour(), n.minute())
}

pub fn add_days(s: &str, n: i64) -> String {
    parse(s).and_then(|d| d.checked_add(n.days()).ok()).map(iso).unwrap_or_else(|| s.to_string())
}

const WEEKDAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
const LONG: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// 0 for Sunday, as JS's getDay().
fn day_of_week(d: Date) -> i64 {
    d.weekday().to_sunday_zero_offset() as i64
}

/// "Today", "Tomorrow", "Friday", "Mon 24 Nov" …
pub fn friendly(s: &str) -> String {
    friendly_on(s, &today())
}

pub fn friendly_on(s: &str, today: &str) -> String {
    if s == today {
        return "Today".into();
    }
    if s == add_days(today, 1) {
        return "Tomorrow".into();
    }
    if s == add_days(today, -1) {
        return "Yesterday".into();
    }
    let (Some(d), Some(t)) = (parse(s), parse(today)) else { return s.to_string() };
    let diff = (d - t).get_days();
    let wd = day_of_week(d) as usize;
    if diff > 0 && diff < 7 {
        return LONG[wd].into();
    }
    let short = &LONG[wd][..3];
    let month = MONTHS[d.month() as usize - 1];
    if d.year() == t.year() { format!("{short} {} {month}", d.day()) } else { format!("{short} {} {month} {}", d.day(), d.year()) }
}

/// "14:30", or "2:30 PM" when the user chose AM/PM (`hour12` in UUOB's settings).
pub fn friendly_time(t: &str, hour12: bool) -> String {
    let mut it = t.split(':').map(|p| p.parse::<i64>().unwrap_or(0));
    let (h, m) = (it.next().unwrap_or(0), it.next().unwrap_or(0));
    if !hour12 {
        return format!("{h}:{}", pad(m));
    }
    format!("{}:{} {}", if h % 12 == 0 { 12 } else { h % 12 }, pad(m), if h < 12 { "AM" } else { "PM" })
}

/// A time as typed: "14:30", "1430", "9", "9.15", "2:30 pm", "2pm" -> "HH:MM",
/// or None if it isn't one.
pub fn parse_time(text: &str) -> Option<String> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})(?:[:.h ]?(\d{2}))?\s*(am|pm|a|p)?$").unwrap());
    let lower = text.trim().to_lowercase();
    let m = RE.captures(&lower)?;
    let mut h: i64 = m[1].parse().ok()?;
    let min: i64 = m.get(2).map_or(Some(0), |x| x.as_str().parse().ok())?;
    if let Some(ap) = m.get(3) {
        if !(1..=12).contains(&h) {
            return None;
        }
        h = h % 12 + if ap.as_str().starts_with('p') { 12 } else { 0 };
    }
    (h <= 23 && min <= 59).then(|| format!("{}:{}", pad(h), pad(min)))
}

/// What quick-add text says: the title without the date words, the date, the time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Quick {
    pub title: String,
    pub due: Option<String>,
    pub time: Option<String>,
}

/// Pulls a date and time out of quick-add text.
///   "call mom fri 5pm" -> { title: "call mom", due: <next Friday>, time: "17:00" }
/// Understands: today, tonight, tomorrow, weekday names, next week,
/// in N days/weeks, 24.12. / 24.12.2026 / 12/24, 5pm, 5:30pm, 17:00, at 9.
pub fn parse_quick(text: &str) -> Quick {
    parse_quick_on(text, &today(), &now_time())
}

pub fn parse_quick_on(text: &str, t: &str, now: &str) -> Quick {
    static DMY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})\.(\d{1,2})\.(\d{4})?$").unwrap());
    static MD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})/(\d{1,2})$").unwrap());
    static AMPM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,2})(?::(\d{2}))?(am|pm)$").unwrap());
    static H24: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([01]?\d|2[0-3]):([0-5]\d)$").unwrap());
    static DAYNAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(sun|mon|tue|wed|thu|fri|sat)[a-z]*$").unwrap());
    static NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());
    static UNIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(day|days|week|weeks)$").unwrap());
    static AT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{1,2}$").unwrap());

    let mut due: Option<String> = None;
    let mut time: Option<String> = None;
    let mut words: Vec<String> = text.split_whitespace().map(String::from).collect();
    let year = parse(t).map_or(2000, |d| d.year() as i64);
    let dow = parse(t).map_or(0, day_of_week);
    let date = |y: i64, m: i64, d: i64| from_parts(y, m, d).map(iso).unwrap_or_default();

    let mut i = 0;
    while i < words.len() {
        let raw = &words[i];
        // A trailing comma or full stop goes, unless the word has a digit ("24.12.").
        let lower = raw.to_lowercase();
        let w = if !raw.chars().any(|c| c.is_ascii_digit()) && (lower.ends_with(',') || lower.ends_with('.')) {
            lower[..lower.len() - 1].to_string()
        } else {
            lower
        };
        let next = words.get(i + 1).map(|s| s.to_lowercase());
        let after = words.get(i + 2).map(|s| s.to_lowercase()).unwrap_or_default();
        let mut take = 0;
        if w == "today" {
            due = Some(t.into());
            take = 1;
        } else if w == "tonight" {
            due = Some(t.into());
            time.get_or_insert_with(|| "20:00".into());
            take = 1;
        } else if w == "tomorrow" || w == "tmr" {
            due = Some(add_days(t, 1));
            take = 1;
        } else if w == "next" && next.as_deref() == Some("week") {
            let n = (8 - dow) % 7;
            due = Some(add_days(t, if n == 0 { 7 } else { n }));
            take = 2;
        } else if w == "in" && NUM.is_match(next.as_deref().unwrap_or("")) && UNIT.is_match(&after) {
            let n: i64 = next.as_deref().unwrap_or("0").parse().unwrap_or(0) * if after.starts_with("week") { 7 } else { 1 };
            due = Some(add_days(t, n));
            take = 3;
        } else if let Some(m) = WEEKDAYS.iter().position(|d| w.get(..3) == Some(*d)).filter(|_| DAYNAME.is_match(&w)) {
            let n = (m as i64 - dow + 7) % 7;
            due = Some(add_days(t, if n == 0 { 7 } else { n }));
            take = 1;
        } else if let Some(c) = DMY.captures(&w) {
            let (d, mo): (i64, i64) = (c[1].parse().unwrap(), c[2].parse().unwrap());
            let y = c.get(3).map_or(year, |y| y.as_str().parse().unwrap());
            let mut v = date(y, mo, d);
            if c.get(3).is_none() && v.as_str() < t {
                v = date(y + 1, mo, d);
            }
            due = Some(v);
            take = 1;
        } else if let Some(c) = MD.captures(&w) {
            let (mo, d): (i64, i64) = (c[1].parse().unwrap(), c[2].parse().unwrap());
            let mut v = date(year, mo, d);
            if v.as_str() < t {
                v = date(year + 1, mo, d);
            }
            due = Some(v);
            take = 1;
        } else if let Some(c) = AMPM.captures(&w) {
            let mut h: i64 = c[1].parse::<i64>().unwrap() % 12;
            if &c[3] == "pm" {
                h += 12;
            }
            time = Some(format!("{}:{}", pad(h), c.get(2).map_or("00", |m| m.as_str())));
            take = 1;
        } else if let Some(c) = H24.captures(&w) {
            time = Some(format!("{}:{}", pad(c[1].parse().unwrap()), &c[2]));
            take = 1;
        } else if w == "at" && AT.is_match(next.as_deref().unwrap_or("")) {
            let mut h: i64 = next.as_deref().unwrap().parse().unwrap();
            if h < 8 {
                h += 12; // "at 5" means 5pm
            }
            time = Some(format!("{}:00", pad(h)));
            take = 2;
        }
        if take > 0 {
            words.drain(i..(i + take).min(words.len()));
        } else {
            i += 1;
        }
    }
    if let (Some(tm), None) = (&time, &due) {
        due = Some(if tm.as_str() < now { add_days(t, 1) } else { t.to_string() });
    }
    Quick { title: words.join(" ").trim().to_string(), due, time }
}

/// Monday of the week `s` is in (Sunday when the week starts on Sunday).
pub fn week_start(s: &str, sunday_first: bool) -> String {
    let Some(d) = parse(s) else { return s.to_string() };
    let back = if sunday_first { day_of_week(d) } else { d.weekday().to_monday_zero_offset() as i64 };
    add_days(s, -back)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_are_24_hour_unless_the_user_chose_am_pm() {
        let h24: Vec<_> = ["00:05", "09:00", "14:30", "23:59"].iter().map(|t| friendly_time(t, false)).collect();
        assert_eq!(h24, ["0:05", "9:00", "14:30", "23:59"]);
        let h12: Vec<_> = ["00:05", "09:00", "12:00", "14:30"].iter().map(|t| friendly_time(t, true)).collect();
        assert_eq!(h12, ["12:05 AM", "9:00 AM", "12:00 PM", "2:30 PM"]);
    }

    #[test]
    fn typed_times() {
        let cases = [
            ("14:30", "14:30"), ("1430", "14:30"), ("930", "09:30"), ("9", "09:00"), ("9.15", "09:15"), ("14h30", "14:30"),
            (" 7:05 ", "07:05"), ("2pm", "14:00"), ("2:30 PM", "14:30"), ("12am", "00:00"), ("12:15 pm", "12:15"), ("11 a", "11:00"),
        ];
        for (typed, want) in cases {
            assert_eq!(parse_time(typed).as_deref(), Some(want), "{typed}");
        }
        for bad in ["", "24:00", "9:60", "13pm", "noon", "1:2", "12345"] {
            assert_eq!(parse_time(bad), None, "{bad}");
        }
    }

    // 2026-11-19 is a Thursday.
    fn q(text: &str) -> (String, Option<String>, Option<String>) {
        let r = parse_quick_on(text, "2026-11-19", "12:00");
        (r.title, r.due, r.time)
    }
    fn s(x: &str) -> Option<String> {
        Some(x.to_string())
    }

    // What ui/dates.js answers for the same text at the same moment (Thu 2026-11-19 12:00).
    #[test]
    fn quick_add_as_dates_js() {
        assert_eq!(q("call mom fri 5pm"), ("call mom".into(), s("2026-11-20"), s("17:00")));
        assert_eq!(q("buy milk"), ("buy milk".into(), None, None));
        assert_eq!(q("pay rent tomorrow"), ("pay rent".into(), s("2026-11-20"), None));
        assert_eq!(q("tmr x"), ("x".into(), s("2026-11-20"), None));
        assert_eq!(q("dinner tonight"), ("dinner".into(), s("2026-11-19"), s("20:00")));
        assert_eq!(q("tonight 9pm party"), ("party".into(), s("2026-11-19"), s("21:00")));
        assert_eq!(q("thursday standup"), ("standup".into(), s("2026-11-26"), None));
        assert_eq!(q("Mon. review"), ("review".into(), s("2026-11-23"), None));
        assert_eq!(q("monster truck"), ("truck".into(), s("2026-11-23"), None));
        assert_eq!(q("sat"), ("".into(), s("2026-11-21"), None));
        assert_eq!(q("plan next week"), ("plan".into(), s("2026-11-23"), None));
        assert_eq!(q("next weeks"), ("next weeks".into(), None, None));
        assert_eq!(q("dentist in 3 days"), ("dentist".into(), s("2026-11-22"), None));
        assert_eq!(q("trip in 2 weeks"), ("trip".into(), s("2026-12-03"), None));
        assert_eq!(q("in 1 week go"), ("go".into(), s("2026-11-26"), None));
        assert_eq!(q("in two days"), ("in two days".into(), None, None));
        assert_eq!(q("gifts 24.12."), ("gifts".into(), s("2026-12-24"), None));
        assert_eq!(q("gifts 24.12"), ("gifts 24.12".into(), None, None));
        assert_eq!(q("old 1.1."), ("old".into(), s("2027-01-01"), None));
        assert_eq!(q("x 3.2.2028"), ("x".into(), s("2028-02-03"), None));
        assert_eq!(q("x 12/24"), ("x".into(), s("2026-12-24"), None));
        assert_eq!(q("x 1/5"), ("x".into(), s("2027-01-05"), None));
        assert_eq!(q("x 13/40"), ("x".into(), s("2027-02-09"), None));
        assert_eq!(q("run 7:30"), ("run".into(), s("2026-11-20"), s("07:30")));
        assert_eq!(q("run 17:30"), ("run".into(), s("2026-11-19"), s("17:30")));
        assert_eq!(q("run 7:30am"), ("run".into(), s("2026-11-20"), s("07:30")));
        assert_eq!(q("5:99pm odd"), ("odd".into(), s("2026-11-19"), s("17:99")));
        assert_eq!(q("12am wake"), ("wake".into(), s("2026-11-20"), s("00:00")));
        assert_eq!(q("12pm lunch"), ("lunch".into(), s("2026-11-19"), s("12:00")));
        assert_eq!(q("call at 5"), ("call".into(), s("2026-11-19"), s("17:00")));
        assert_eq!(q("meet at 9"), ("meet".into(), s("2026-11-20"), s("09:00")));
        assert_eq!(q("at 12 noon"), ("noon".into(), s("2026-11-19"), s("12:00")));
        assert_eq!(q("Today, write"), ("write".into(), s("2026-11-19"), None));
        assert_eq!(q("31.2. odd"), ("odd".into(), s("2027-03-03"), None));
        assert_eq!(q("  spaces   between  "), ("spaces between".into(), None, None));
        assert_eq!(q("5pm"), ("".into(), s("2026-11-19"), s("17:00")));
        assert_eq!(q("tomorrow today"), ("".into(), s("2026-11-19"), None));
        assert_eq!(q("24:00 late"), ("24:00 late".into(), None, None));
        assert_eq!(q("call, mom."), ("call, mom.".into(), None, None));
        assert_eq!(q("fri, 10am, meeting"), ("10am, meeting".into(), s("2026-11-20"), None));
    }

    #[test]
    fn friendly_dates() {
        let t = "2026-11-19";
        assert_eq!(friendly_on("2026-11-19", t), "Today");
        assert_eq!(friendly_on("2026-11-20", t), "Tomorrow");
        assert_eq!(friendly_on("2026-11-18", t), "Yesterday");
        assert_eq!(friendly_on("2026-11-24", t), "Tuesday");
        assert_eq!(friendly_on("2026-11-30", t), "Mon 30 Nov");
        assert_eq!(friendly_on("2027-01-04", t), "Mon 4 Jan 2027");
        assert_eq!(week_start("2026-11-19", false), "2026-11-16");
        assert_eq!(week_start("2026-11-19", true), "2026-11-15");
    }
}
