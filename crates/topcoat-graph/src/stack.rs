//! Stacking: turning series values into lower and upper bounds per category.

/// Stacks `series` on top of each other, category by category.
///
/// The result has one list per series with `category_count` entries, each the
/// `(lower, upper)` bounds of that series' value, or `None` where the value is missing
/// (non-finite, or past the end of a short series). Positive values stack upward from
/// zero and negative values downward, so a mixed category diverges around the zero
/// line. A missing value takes no room. Values past `category_count` are ignored.
pub fn stack(series: &[&[f64]], category_count: usize) -> Vec<Vec<Option<(f64, f64)>>> {
    let mut positive = vec![0.0; category_count];
    let mut negative = vec![0.0; category_count];
    series
        .iter()
        .map(|values| {
            (0..category_count)
                .map(|category| {
                    let value = values.get(category).copied().filter(|v| v.is_finite())?;
                    let base = if value >= 0.0 {
                        &mut positive
                    } else {
                        &mut negative
                    };
                    let from = base[category];
                    // Saturate rather than overflow to infinity.
                    let to = (from + value).clamp(f64::MIN, f64::MAX);
                    base[category] = to;
                    Some(if value >= 0.0 { (from, to) } else { (to, from) })
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positives_stack_up_and_negatives_stack_down() {
        let s = stack(&[&[1.0, -2.0, f64::NAN], &[3.0, -1.0]], 3);
        assert_eq!(s[0], [Some((0.0, 1.0)), Some((-2.0, 0.0)), None]);
        assert_eq!(s[1], [Some((1.0, 4.0)), Some((-3.0, -2.0)), None]);
    }

    #[test]
    fn a_missing_value_does_not_shift_the_series_above_it() {
        let s = stack(&[&[f64::NAN], &[2.0]], 1);
        assert_eq!(s[1], [Some((0.0, 2.0))]);
    }

    #[test]
    fn values_past_the_category_count_are_ignored() {
        let s = stack(&[&[1.0, 2.0, 3.0]], 2);
        assert_eq!(s[0].len(), 2);
        assert!(stack(&[], 3).is_empty());
    }
}
