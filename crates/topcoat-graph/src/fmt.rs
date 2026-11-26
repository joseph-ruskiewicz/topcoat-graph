//! Number formatting for path data and CSS values.

/// Formats a number with at most two decimal places and no trailing zeros.
///
/// Geometry written into SVG path data and inline styles goes through this, so output
/// is compact and the same on every run. A non-finite value prints as `0`.
pub fn num(value: f64) -> String {
    if !value.is_finite() {
        return String::from("0");
    }
    // Round half away from zero first: formatting alone rounds 0.125 to 0.12.
    let mut text = format!("{:.2}", (value * 100.0).round() / 100.0);
    if text.contains('.') {
        let trimmed = text.trim_end_matches('0').trim_end_matches('.').len();
        text.truncate(trimmed);
    }
    if text == "-0" {
        text.remove(0);
    }
    text
}

/// Formats a data value for people to read, in a tooltip or a table.
///
/// Unlike [`num`], which rounds positions to two decimals, this keeps six significant
/// digits, so `0.0012` stays `0.0012`. Whole parts are never rounded away, float noise
/// such as `0.30000000000000004` is, and trailing zeros are trimmed. A non-finite
/// value, which means "missing", is the empty string.
pub fn value(value: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }
    if value == 0.0 {
        return String::from("0");
    }
    let magnitude = value.abs().log10().floor();
    let decimals = (5.0 - magnitude).clamp(0.0, 12.0) as usize;
    let mut text = format!("{value:.decimals$}");
    if text.contains('.') {
        let trimmed = text.trim_end_matches('0').trim_end_matches('.').len();
        text.truncate(trimmed);
    }
    if text == "-0" {
        text.remove(0);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_rounds_to_two_decimals_and_trims() {
        assert_eq!(num(12.0), "12");
        assert_eq!(num(0.125), "0.13");
        assert_eq!(num(2.5), "2.5");
        assert_eq!(num(-3.10), "-3.1");
    }

    #[test]
    fn num_never_prints_negative_zero_or_non_finite() {
        assert_eq!(num(-0.0), "0");
        assert_eq!(num(-0.001), "0");
        assert_eq!(num(f64::NAN), "0");
        assert_eq!(num(f64::INFINITY), "0");
    }

    #[test]
    fn value_keeps_small_numbers_readable() {
        assert_eq!(value(0.0012), "0.0012");
        assert_eq!(value(0.00000123456), "0.00000123456");
        assert_eq!(value(0.123456789), "0.123457");
        assert_eq!(value(12.0), "12");
        assert_eq!(value(56.25), "56.25");
        assert_eq!(value(-3.5), "-3.5");
    }

    #[test]
    fn value_drops_float_noise_and_keeps_large_numbers_whole() {
        assert_eq!(value(0.1 + 0.2), "0.3");
        assert_eq!(value(1234567.0), "1234567");
        assert_eq!(value(123456.789), "123457");
        assert_eq!(value(0.0), "0");
        assert_eq!(value(-0.0), "0");
        assert_eq!(value(f64::NAN), "");
    }
}
