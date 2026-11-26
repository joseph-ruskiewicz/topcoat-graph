//! "Nice" axis ticks: round steps that cover a numeric domain.

/// Inputs are clamped to this magnitude so spans never overflow.
const LIMIT: f64 = 1e150;

/// Evenly spaced tick values covering a domain.
#[derive(Clone, Debug, PartialEq)]
pub struct Ticks {
    /// The tick values, ascending. There are always at least two.
    pub values: Vec<f64>,
    /// The distance between neighboring ticks.
    pub step: f64,
}

impl Ticks {
    /// The first and last tick: the domain an axis with these ticks spans.
    ///
    /// Ticks built by [`nice`] always have at least two values. For a hand-built value
    /// with none, the domain is `(0.0, 1.0)`.
    pub fn domain(&self) -> (f64, f64) {
        match (self.values.first(), self.values.last()) {
            (Some(&first), Some(&last)) => (first, last),
            _ => (0.0, 1.0),
        }
    }

    /// The text for a tick, with the decimal places the step needs and no trailing zeros.
    pub fn label(&self, value: f64) -> String {
        let decimals = if self.step >= 1.0 || self.step <= 0.0 {
            0
        } else {
            ((-self.step.log10() - 1e-9).ceil() as usize).min(12)
        };
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
}

/// Picks ticks that cover `min..=max` in steps of 1, 2 or 5 times a power of ten.
///
/// `target` is the number of intervals to aim for; the result has about that many. The
/// first tick is at or below `min` and the last at or above `max`. With `integer`, the
/// step is never less than 1, for axes that count things.
///
/// Any input gives usable ticks. Non-finite bounds count as 0 and reversed bounds are
/// swapped. A domain of the single value 0 becomes `0..=1`; any other single value gets
/// room on both sides.
pub fn nice(min: f64, max: f64, target: usize, integer: bool) -> Ticks {
    let clean = |v: f64| {
        if v.is_finite() {
            v.clamp(-LIMIT, LIMIT)
        } else {
            0.0
        }
    };
    let (mut min, mut max) = (clean(min), clean(max));
    if min > max {
        std::mem::swap(&mut min, &mut max);
    }
    let target = target.clamp(1, 100) as f64;

    let mut step = step_for(max - min, target);
    if step.is_none() {
        // A single value, or a span too small to divide.
        (min, max) = if min == 0.0 {
            (0.0, 1.0)
        } else {
            (min - min.abs(), min + min.abs())
        };
        step = step_for(max - min, target);
    }
    let mut step = step.unwrap_or(1.0);
    if integer {
        step = step.max(1.0);
    }

    let first = (min / step + 1e-9).floor();
    let mut last = (max / step - 1e-9).ceil();
    if last <= first {
        last = first + 1.0;
    }
    // For a fractional step, divide by its whole-number inverse: 3 / 5 is 0.6 exactly
    // where 3 * 0.2 is not.
    let inverse = (1.0 / step).round();
    let count = (last - first) as usize;
    let values = (0..=count)
        .map(|i| {
            let n = first + i as f64;
            if step < 1.0 && inverse.is_finite() {
                n / inverse
            } else {
                n * step
            }
        })
        .collect();
    Ticks { values, step }
}

/// The 1-2-5 step nearest `span / target`, or `None` when the span cannot be divided.
fn step_for(span: f64, target: f64) -> Option<f64> {
    let raw = span / target;
    if raw <= 0.0 || !raw.is_finite() {
        return None;
    }
    let power = 10f64.powf(raw.log10().floor());
    let error = raw / power;
    let step = if error >= 50f64.sqrt() {
        10.0 * power
    } else if error >= 10f64.sqrt() {
        5.0 * power
    } else if error >= 2f64.sqrt() {
        2.0 * power
    } else {
        power
    };
    (step > 0.0 && step.is_finite()).then_some(step)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(t: &Ticks) -> Vec<String> {
        t.values.iter().map(|v| t.label(*v)).collect()
    }

    #[test]
    fn picks_steps_of_one_two_or_five() {
        assert_eq!(
            nice(0.0, 12.0, 5, false).values,
            [0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0]
        );
        assert_eq!(
            nice(-7.0, 7.0, 4, false).values,
            [-10.0, -5.0, 0.0, 5.0, 10.0]
        );
    }

    #[test]
    fn widens_the_domain_to_the_outer_ticks() {
        assert_eq!(nice(0.0, 97.0, 5, false).domain(), (0.0, 100.0));
    }

    #[test]
    fn integer_ticks_never_step_below_one() {
        assert_eq!(nice(0.0, 1.0, 5, true).values, [0.0, 1.0]);
        assert_eq!(nice(0.0, 2.0, 5, true).values, [0.0, 1.0, 2.0]);
    }

    #[test]
    fn an_all_zero_domain_becomes_zero_to_one() {
        assert_eq!(nice(0.0, 0.0, 5, false).domain(), (0.0, 1.0));
        assert_eq!(nice(0.0, 0.0, 5, true).values, [0.0, 1.0]);
    }

    #[test]
    fn a_single_nonzero_value_gets_room_on_both_sides() {
        let (lo, hi) = nice(5.0, 5.0, 5, false).domain();
        assert!(lo <= 0.0 && hi >= 10.0);
    }

    #[test]
    fn labels_have_no_float_noise() {
        let t = nice(0.0, 1.0, 5, false);
        assert_eq!(labels(&t), ["0", "0.2", "0.4", "0.6", "0.8", "1"]);
        let t = nice(0.0, 0.3, 3, false);
        assert_eq!(labels(&t), ["0", "0.1", "0.2", "0.3"]);
    }

    #[test]
    fn bad_inputs_still_give_finite_ticks() {
        let t = nice(f64::NAN, f64::INFINITY, 0, false);
        assert!(t.values.len() >= 2);
        assert!(t.values.iter().all(|v| v.is_finite()));
        assert_eq!(nice(10.0, 0.0, 5, false).domain(), (0.0, 10.0));
    }

    #[test]
    fn huge_spans_do_not_explode_the_tick_count() {
        assert!(nice(0.0, 1e300, 5, false).values.len() <= 12);
        assert!(nice(0.0, 1e-300, 5, false).values.len() <= 12);
    }

    #[test]
    fn a_hand_built_empty_ticks_still_has_a_domain() {
        let empty = Ticks {
            values: Vec::new(),
            step: 1.0,
        };
        assert_eq!(empty.domain(), (0.0, 1.0));
    }
}
