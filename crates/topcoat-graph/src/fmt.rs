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
}
