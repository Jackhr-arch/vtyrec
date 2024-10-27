use super::error::ParseError;

pub fn parse_sleep(s: &str) -> Result<u64, ParseError> {
    match s.strip_suffix('s') {
        Some(s) => Ok(match s.strip_suffix('m') {
            Some(v) => v.parse()?,
            None => s.parse::<u64>()? * 1000,
        }),
        None => Err(ParseError::empty()),
    }
}

pub fn parse_times(s: &str) -> Result<usize, core::num::ParseIntError> {
    if !s.is_empty() {
        Ok(s.parse()?)
    } else {
        Ok(1)
    }
}
/// (@100ms 2) or (2)
///
/// (@100ms "12") or ("12")
///
/// (content, Option<delay>)
pub fn parse_with_delay_or(s: &str) -> Result<(&str, Option<u64>), ParseError> {
    match s.strip_prefix('@') {
        Some(s) => parse_delay(s).map(|(s, d)| (s, Some(d))),
        None => Ok((s, None)),
    }
}
/// (times, Option<delay>)
pub fn parse_with_delay_times(s: &str) -> Result<(usize, Option<u64>), ParseError> {
    let (s, delay) = parse_with_delay_or(s)?;
    Ok((parse_times(s)?, delay))
}
/// (100ms 2) or (100ms "12")
///
/// (content, delay)
fn parse_delay(s: &str) -> Result<(&str, u64), ParseError> {
    let mut rel = s.split('s');
    let maybe_delay = rel.next().unwrap();
    Ok((
        rel.next()
            .unwrap_or_default()
            .trim_start()
            .trim_start_matches('"')
            .trim_end_matches('"'),
        match maybe_delay.strip_suffix('m') {
            Some(s) => s.parse()?,
            None => maybe_delay
                .parse()
                .map(|n: f64| n * 1000.0)
                // Ok when value is not very big, otherwise it might be incorrect
                // as 1e50_f64 => 18446744073709551615
                // but it's fine
                .map(|n| n.trunc() as u64)?,
        },
    ))
}
