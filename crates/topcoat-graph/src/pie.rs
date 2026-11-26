//! Pie and donut slice geometry.

use std::f64::consts::{PI, TAU};

use crate::fmt::num;

/// The middle of the `0 0 100 100` box slices are drawn in.
const CENTER: f64 = 50.0;
/// The outer radius: slices reach the edge of the box.
const RADIUS: f64 = 50.0;

/// How a pie is laid out. The default is a full pie starting at twelve o'clock.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PieOptions {
    /// The hole's radius as a fraction of the outer radius: `0.0` for a pie, around
    /// `0.6` for a donut. Clamped to `0.0..=0.95`.
    pub inner_radius: f64,
    /// The gap between neighboring slices, in radians. A lone slice gets no gap.
    pub pad_angle: f64,
    /// Where the first slice starts, in radians clockwise from twelve o'clock.
    pub start_angle: f64,
}

/// One slice of a pie.
#[derive(Clone, Debug, PartialEq)]
pub struct Slice {
    /// The position of this slice's value in the input.
    pub index: usize,
    /// SVG path data for the slice, in a `0 0 100 100` box. Fill it with
    /// `fill-rule="evenodd"`: a donut slice covering the whole circle is two rings.
    pub path: String,
    /// The middle of the slice, for anchoring a label or tooltip.
    pub centroid: (f64, f64),
    /// This slice's share of the total, from 0 to 1.
    pub fraction: f64,
}

/// Lays out `values` as slices, clockwise in input order.
///
/// Values that are zero, negative or non-finite get no slice, so the result may be
/// shorter than the input; [`Slice::index`] says which value a slice belongs to. When
/// nothing is positive the result is empty.
pub fn pie_layout(values: &[f64], options: &PieOptions) -> Vec<Slice> {
    let finite_or_zero = |v: f64| if v.is_finite() { v } else { 0.0 };
    let inner = RADIUS * finite_or_zero(options.inner_radius).clamp(0.0, 0.95);
    let start = finite_or_zero(options.start_angle) % TAU;

    let shown: Vec<(usize, f64)> = values
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| value.is_finite() && *value > 0.0)
        .collect();
    let total: f64 = shown.iter().map(|(_, value)| value).sum();
    if !total.is_finite() || total <= 0.0 {
        return Vec::new();
    }
    let pad = if shown.len() > 1 {
        finite_or_zero(options.pad_angle).max(0.0)
    } else {
        0.0
    };

    let mut angle = start;
    shown
        .into_iter()
        .map(|(index, value)| {
            let fraction = value / total;
            let span = fraction * TAU;
            let from = angle;
            angle += span;
            // A gap never eats more than half of its slice.
            let inset = (pad / 2.0).min(span / 4.0);
            let (a0, a1) = (from + inset, from + span - inset);
            let whole = fraction == 1.0;
            Slice {
                index,
                path: if whole {
                    ring(inner)
                } else {
                    wedge(a0, a1, inner)
                },
                centroid: if whole && inner == 0.0 {
                    (CENTER, CENTER)
                } else {
                    point((a0 + a1) / 2.0, (inner + RADIUS) / 2.0)
                },
                fraction,
            }
        })
        .collect()
}

/// The point at `angle` (clockwise from twelve o'clock) and `radius` from the center.
fn point(angle: f64, radius: f64) -> (f64, f64) {
    (CENTER + radius * angle.sin(), CENTER - radius * angle.cos())
}

fn pair((x, y): (f64, f64)) -> String {
    format!("{},{}", num(x), num(y))
}

/// A slice from angle `a0` to `a1`: a wedge to the center, or a ring segment.
fn wedge(a0: f64, a1: f64, inner: f64) -> String {
    let large = if a1 - a0 > PI { 1 } else { 0 };
    let r = num(RADIUS);
    let outer = format!("A{r},{r} 0 {large} 1 {}", pair(point(a1, RADIUS)));
    if inner == 0.0 {
        return format!(
            "M{c},{c}L{}{outer}Z",
            pair(point(a0, RADIUS)),
            c = num(CENTER)
        );
    }
    let i = num(inner);
    format!(
        "M{}{outer}L{}A{i},{i} 0 {large} 0 {}Z",
        pair(point(a0, RADIUS)),
        pair(point(a1, inner)),
        pair(point(a0, inner)),
    )
}

/// The whole circle. An arc cannot span 360 degrees, so each circle is two half arcs;
/// a hole is a second circle wound the other way.
fn ring(inner: f64) -> String {
    let circle = |radius: f64, sweep: u8| {
        let r = num(radius);
        let (top, bottom) = (pair(point(0.0, radius)), pair(point(PI, radius)));
        format!("M{top}A{r},{r} 0 1 {sweep} {bottom}A{r},{r} 0 1 {sweep} {top}Z")
    };
    let mut path = circle(RADIUS, 1);
    if inner > 0.0 {
        path.push_str(&circle(inner, 0));
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pie(values: &[f64]) -> Vec<Slice> {
        pie_layout(values, &PieOptions::default())
    }

    #[test]
    fn slices_run_clockwise_from_twelve_o_clock() {
        let s = pie(&[1.0, 1.0, 2.0]);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].path, "M50,50L50,0A50,50 0 0 1 100,50Z");
        assert_eq!(s[1].path, "M50,50L100,50A50,50 0 0 1 50,100Z");
        assert_eq!(s[2].fraction, 0.5);
        assert!((s[2].centroid.0 - 25.0).abs() < 0.01 && (s[2].centroid.1 - 50.0).abs() < 0.01);
    }

    #[test]
    fn a_slice_over_half_the_circle_uses_the_large_arc() {
        let s = pie(&[3.0, 1.0]);
        assert_eq!(s[0].path, "M50,50L50,0A50,50 0 1 1 0,50Z");
    }

    #[test]
    fn nothing_to_show_gives_no_slices() {
        assert!(pie(&[0.0, 0.0]).is_empty());
        assert!(pie(&[]).is_empty());
        assert!(pie(&[-4.0, f64::NAN]).is_empty());
    }

    #[test]
    fn a_whole_circle_is_drawn_as_two_half_arcs() {
        let full = pie(&[0.0, 7.0]);
        assert_eq!(full.len(), 1);
        assert_eq!(full[0].index, 1);
        assert_eq!(full[0].fraction, 1.0);
        assert_eq!(full[0].path, "M50,0A50,50 0 1 1 50,100A50,50 0 1 1 50,0Z");
        assert_eq!(full[0].centroid, (50.0, 50.0));
    }

    #[test]
    fn negative_and_non_finite_values_get_no_slice() {
        let s = pie(&[-1.0, f64::NAN, 2.0, f64::INFINITY]);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].index, 2);
    }

    #[test]
    fn a_donut_slice_is_a_ring_segment() {
        let options = PieOptions {
            inner_radius: 0.6,
            ..Default::default()
        };
        let donut = pie_layout(&[1.0, 3.0], &options);
        assert_eq!(
            donut[0].path,
            "M50,0A50,50 0 0 1 100,50L80,50A30,30 0 0 0 50,20Z"
        );
        assert!(!donut[1].path.contains("L50,50"));
        // The centroid sits in the middle of the ring's thickness.
        let (x, y) = donut[1].centroid;
        assert!(((x - 50.0).hypot(y - 50.0) - 40.0).abs() < 0.01);
    }

    #[test]
    fn a_whole_donut_has_a_hole() {
        let options = PieOptions {
            inner_radius: 0.5,
            ..Default::default()
        };
        let ring = pie_layout(&[5.0], &options);
        assert_eq!(
            ring[0].path,
            "M50,0A50,50 0 1 1 50,100A50,50 0 1 1 50,0ZM50,25A25,25 0 1 0 50,75A25,25 0 1 0 50,25Z"
        );
    }

    #[test]
    fn padding_separates_slices_but_not_a_lone_slice() {
        let options = PieOptions {
            pad_angle: 0.1,
            ..Default::default()
        };
        let padded = pie_layout(&[1.0, 1.0], &options);
        assert!(!padded[0].path.contains("L50,0A"), "{}", padded[0].path);
        let alone = pie_layout(&[1.0], &options);
        assert_eq!(alone[0].path, pie(&[1.0])[0].path);
    }

    #[test]
    fn odd_options_never_produce_non_finite_output() {
        let options = PieOptions {
            inner_radius: f64::NAN,
            pad_angle: 99.0,
            start_angle: f64::INFINITY,
        };
        for slice in pie_layout(&[1.0, 2.0, 3.0], &options) {
            assert!(
                !slice.path.contains("NaN") && !slice.path.contains("inf"),
                "{}",
                slice.path
            );
            assert!(slice.centroid.0.is_finite() && slice.centroid.1.is_finite());
        }
        let options = PieOptions {
            inner_radius: 5.0,
            ..Default::default()
        };
        assert!(
            pie_layout(&[1.0, 1.0], &options)[0]
                .path
                .contains("A47.5,47.5")
        );
    }
}
