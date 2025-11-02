use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, Timelike, Weekday};
use regex::Regex;

// parse things like "+10 hours", "+2 days", "+30 minutes"
pub fn parse_relative_time(spec: &str) -> Result<DateTime<Local>> {
    let re = Regex::new(r"^\+(\d+)\s*(hours?|days?|minutes?)$")?;
    let captures = re
        .captures(spec)
        .ok_or_else(|| anyhow::anyhow!("bad format, try: +10 hours"))?;

    let amount: i64 = captures[1].parse()?;
    if amount <= 0 {
        return Err(anyhow::anyhow!("amount must be positive"));
    }

    let unit = &captures[2].to_lowercase();
    let duration = if unit.starts_with("hour") {
        Duration::hours(amount)
    } else if unit.starts_with("day") {
        Duration::days(amount)
    } else {
        Duration::minutes(amount)
    };

    Ok(Local::now() + duration)
}

// parse day names like "Monday", "Tuesday", etc
// defaults to 9am on the next occurrence of that day
pub fn parse_named_day(spec: &str) -> Result<DateTime<Local>> {
    let target_weekday = match spec.to_lowercase().as_str() {
        "monday" => Weekday::Mon,
        "tuesday" => Weekday::Tue,
        "wednesday" => Weekday::Wed,
        "thursday" => Weekday::Thu,
        "friday" => Weekday::Fri,
        "saturday" => Weekday::Sat,
        "sunday" => Weekday::Sun,
        _ => return Err(anyhow::anyhow!("unknown day: {}", spec)),
    };

    let now = Local::now();
    let current = now.weekday();

    // if it's the same day but past 9am, schedule for next week
    let days_ahead = if current == target_weekday {
        if now.hour() >= 9 {
            7
        } else {
            0
        }
    } else {
        let curr = current.num_days_from_monday();
        let targ = target_weekday.num_days_from_monday();
        if targ > curr {
            targ - curr
        } else {
            7 - (curr - targ)
        }
    };

    let target_date = now.date_naive() + Duration::days(days_ahead as i64);
    let target_time = target_date
        .and_hms_opt(9, 0, 0)
        .ok_or_else(|| anyhow::anyhow!("couldn't create time"))?;

    Ok(target_time.and_local_timezone(Local).unwrap())
}

// parse absolute times like "2025-11-04 09:00"
pub fn parse_absolute_time(spec: &str) -> Result<DateTime<Local>> {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ];

    for format in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(spec, format) {
            return Ok(dt.and_local_timezone(Local).unwrap());
        }
    }

    Err(anyhow::anyhow!("bad datetime format, try: 2025-11-04 09:00"))
}

// main entry point - figures out what kind of time spec it is
pub fn parse_time_spec(spec: &str) -> Result<DateTime<Local>> {
    // relative time starts with +
    if spec.starts_with('+') {
        return parse_relative_time(spec);
    }

    // try named day
    if let Ok(dt) = parse_named_day(spec) {
        if dt > Local::now() {
            return Ok(dt);
        }
    }

    // try absolute time
    if let Ok(dt) = parse_absolute_time(spec) {
        if dt <= Local::now() {
            return Err(anyhow::anyhow!("that time is in the past"));
        }
        return Ok(dt);
    }

    Err(anyhow::anyhow!(
        "couldn't parse time. try: +10 hours, Monday, or 2025-11-04 09:00"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_hours() {
        let result = parse_relative_time("+10 hours");
        assert!(result.is_ok());
    }

    #[test]
    fn test_relative_days() {
        let result = parse_relative_time("+2 days");
        assert!(result.is_ok());
    }

    #[test]
    fn test_relative_minutes() {
        let result = parse_relative_time("+30 minutes");
        assert!(result.is_ok());
    }

    #[test]
    fn test_relative_bad_format() {
        let result = parse_relative_time("10 hours");
        assert!(result.is_err());
    }

    #[test]
    fn test_relative_negative() {
        let result = parse_relative_time("+-10 hours");
        assert!(result.is_err());
    }

    #[test]
    fn test_named_day_monday() {
        let result = parse_named_day("Monday");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.weekday(), Weekday::Mon);
        assert_eq!(dt.hour(), 9);
    }

    #[test]
    fn test_named_day_case_insensitive() {
        let result = parse_named_day("monday");
        assert!(result.is_ok());
    }

    #[test]
    fn test_named_day_invalid() {
        let result = parse_named_day("Funday");
        assert!(result.is_err());
    }

    #[test]
    fn test_absolute_time() {
        let result = parse_absolute_time("2025-12-25 09:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_absolute_time_iso() {
        let result = parse_absolute_time("2025-12-25T09:00:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_absolute_time_bad_format() {
        let result = parse_absolute_time("tomorrow");
        assert!(result.is_err());
    }

    #[test]
    fn test_time_spec_relative() {
        let result = parse_time_spec("+5 hours");
        assert!(result.is_ok());
    }

    #[test]
    fn test_time_spec_named() {
        let result = parse_time_spec("Friday");
        assert!(result.is_ok());
    }

    #[test]
    fn test_time_spec_absolute() {
        let result = parse_time_spec("2030-01-01 00:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_time_spec_past_time() {
        let result = parse_time_spec("2020-01-01 00:00");
        assert!(result.is_err());
    }
}
