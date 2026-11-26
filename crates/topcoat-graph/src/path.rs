//! SVG path data for lines and areas.

use crate::fmt::num;

/// How a line gets from one point to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Curve {
    /// Straight segments.
    #[default]
    Linear,
    /// A smooth curve through every point that never overshoots the data: between two
    /// points it stays between their values. Points must be ordered by x.
    Smooth,
}

type Point = (f64, f64);

/// One step of a path: a cubic segment with two control points, or a straight one.
struct Segment {
    controls: Option<(Point, Point)>,
    end: Point,
}

/// Path data for a line through `points`.
///
/// A missing point (`None`, or a non-finite coordinate) breaks the line: each run of
/// present points is its own subpath. A run of one point is a zero-length segment,
/// which a round line cap draws as a dot. With no points the path is empty.
pub fn line(points: &[Option<Point>], curve: Curve) -> String {
    let mut path = String::new();
    for run in runs(points, points) {
        let points: Vec<Point> = run.iter().map(|&(upper, _)| upper).collect();
        move_to(&mut path, points[0]);
        if points.len() == 1 {
            line_to(&mut path, points[0]);
        }
        forward(&mut path, &points, curve);
    }
    path
}

/// Path data for the area between an `upper` and a `lower` line.
///
/// Each run goes along the upper line, down to the lower line, back along it, and
/// closes. The two slices pair up by index; a position is missing when either is.
pub fn area(upper: &[Option<Point>], lower: &[Option<Point>], curve: Curve) -> String {
    let mut path = String::new();
    for run in runs(upper, lower) {
        let top: Vec<Point> = run.iter().map(|&(upper, _)| upper).collect();
        let bottom: Vec<Point> = run.iter().map(|&(_, lower)| lower).collect();
        move_to(&mut path, top[0]);
        forward(&mut path, &top, curve);
        line_to(&mut path, bottom[bottom.len() - 1]);
        backward(&mut path, &bottom, curve);
        path.push('Z');
    }
    path
}

/// The runs of consecutive positions where both slices have a finite point.
fn runs(upper: &[Option<Point>], lower: &[Option<Point>]) -> Vec<Vec<(Point, Point)>> {
    let finite = |point: &Option<Point>| point.filter(|(x, y)| x.is_finite() && y.is_finite());
    let mut runs = Vec::new();
    let mut current = Vec::new();
    for (upper, lower) in upper.iter().zip(lower) {
        match (finite(upper), finite(lower)) {
            (Some(upper), Some(lower)) => current.push((upper, lower)),
            _ if !current.is_empty() => runs.push(std::mem::take(&mut current)),
            _ => {}
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    runs
}

fn move_to(path: &mut String, (x, y): Point) {
    path.push_str(&format!("M{},{}", num(x), num(y)));
}

fn line_to(path: &mut String, (x, y): Point) {
    path.push_str(&format!("L{},{}", num(x), num(y)));
}

fn curve_to(path: &mut String, (a, b): (Point, Point), end: Point) {
    path.push_str(&format!(
        "C{},{} {},{} {},{}",
        num(a.0),
        num(a.1),
        num(b.0),
        num(b.1),
        num(end.0),
        num(end.1)
    ));
}

/// Appends the path from `points[0]`, where the pen already is, to the last point.
fn forward(path: &mut String, points: &[Point], curve: Curve) {
    for segment in segments(points, curve) {
        match segment.controls {
            Some(controls) => curve_to(path, controls, segment.end),
            None => line_to(path, segment.end),
        }
    }
}

/// Appends the path from the last point, where the pen already is, back to `points[0]`.
fn backward(path: &mut String, points: &[Point], curve: Curve) {
    let segments = segments(points, curve);
    for (index, segment) in segments.iter().enumerate().rev() {
        // Retrace the forward segment that ended here: same curve, controls swapped.
        let start = points[index];
        match segment.controls {
            Some((a, b)) => curve_to(path, (b, a), start),
            None => line_to(path, start),
        }
    }
}

/// The segments joining `points` in order: one per point after the first.
fn segments(points: &[Point], curve: Curve) -> Vec<Segment> {
    let straight = || {
        points
            .iter()
            .skip(1)
            .map(|&end| Segment {
                controls: None,
                end,
            })
            .collect()
    };
    if curve == Curve::Linear || points.len() < 3 {
        return straight();
    }
    let tangents = monotone_tangents(points);
    points
        .windows(2)
        .zip(tangents.windows(2))
        .map(|(p, t)| {
            let third = (p[1].0 - p[0].0) / 3.0;
            let a = (p[0].0 + third, p[0].1 + third * t[0]);
            let b = (p[1].0 - third, p[1].1 - third * t[1]);
            Segment {
                controls: Some((a, b)),
                end: p[1],
            }
        })
        .collect()
}

/// The slope of the curve at each point, chosen so the curve is monotone between
/// neighbors (the Fritsch-Carlson method, as in d3's `curveMonotoneX`).
///
/// Needs at least three points.
fn monotone_tangents(points: &[Point]) -> Vec<f64> {
    let slope = |a: Point, b: Point| {
        let width = b.0 - a.0;
        if width == 0.0 {
            0.0
        } else {
            (b.1 - a.1) / width
        }
    };
    let last = points.len() - 1;
    let mut tangents = vec![0.0; points.len()];
    for i in 1..last {
        let (before, here, after) = (points[i - 1], points[i], points[i + 1]);
        let (s0, s1) = (slope(before, here), slope(here, after));
        let (h0, h1) = (here.0 - before.0, after.0 - here.0);
        let weighted = if h0 + h1 == 0.0 {
            0.0
        } else {
            (s0 * h1 + s1 * h0) / (h0 + h1)
        };
        // Zero at a peak or a flat neighbor; otherwise no steeper than either side allows.
        let sign = |s: f64| {
            if s > 0.0 {
                1.0
            } else if s < 0.0 {
                -1.0
            } else {
                0.0
            }
        };
        tangents[i] = (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * weighted.abs());
    }
    // An end's slope completes the parabola through its segment and its neighbor's slope.
    let end = |a: Point, b: Point, neighbor: f64| (3.0 * slope(a, b) - neighbor) / 2.0;
    tangents[0] = end(points[0], points[1], tangents[1]);
    tangents[last] = end(points[last - 1], points[last], tangents[last - 1]);
    tangents
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every number in a path string.
    fn numbers(path: &str) -> Vec<f64> {
        path.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect()
    }

    #[test]
    fn a_straight_line_joins_the_points() {
        let points = [Some((0.0, 10.0)), Some((50.0, 0.0)), Some((100.0, 5.0))];
        assert_eq!(line(&points, Curve::Linear), "M0,10L50,0L100,5");
    }

    #[test]
    fn a_missing_point_splits_the_line() {
        let points = [Some((0.0, 1.0)), None, Some((2.0, 3.0)), Some((4.0, 5.0))];
        assert_eq!(line(&points, Curve::Linear), "M0,1L0,1M2,3L4,5");
        assert_eq!(line(&points, Curve::Smooth), "M0,1L0,1M2,3L4,5");
    }

    #[test]
    fn no_points_is_an_empty_path() {
        assert_eq!(line(&[], Curve::Smooth), "");
        assert_eq!(line(&[None, None], Curve::Linear), "");
        assert_eq!(area(&[], &[], Curve::Linear), "");
    }

    #[test]
    fn an_area_runs_along_the_top_and_back_along_the_bottom() {
        let upper = [Some((0.0, 10.0)), Some((100.0, 20.0))];
        let lower = [Some((0.0, 100.0)), Some((100.0, 100.0))];
        assert_eq!(
            area(&upper, &lower, Curve::Linear),
            "M0,10L100,20L100,100L0,100Z"
        );
    }

    #[test]
    fn a_missing_point_splits_the_area() {
        let upper = [
            Some((0.0, 1.0)),
            Some((1.0, 1.0)),
            None,
            Some((3.0, 1.0)),
            Some((4.0, 1.0)),
        ];
        let lower = [
            Some((0.0, 9.0)),
            Some((1.0, 9.0)),
            None,
            Some((3.0, 9.0)),
            Some((4.0, 9.0)),
        ];
        assert_eq!(
            area(&upper, &lower, Curve::Linear),
            "M0,1L1,1L1,9L0,9ZM3,1L4,1L4,9L3,9Z"
        );
    }

    #[test]
    fn a_smooth_line_passes_through_its_points_without_overshooting() {
        let points = [
            (0.0, 0.0),
            (10.0, 0.0),
            (20.0, 50.0),
            (30.0, 50.0),
            (40.0, 100.0),
        ]
        .map(Some);
        let path = line(&points, Curve::Smooth);
        assert!(path.starts_with("M0,0C"));
        assert!(path.ends_with("40,100"));
        assert_eq!(path.matches('C').count(), 4);
        let n = numbers(&path);
        let (xs, ys): (Vec<f64>, Vec<f64>) = n.chunks(2).map(|c| (c[0], c[1])).unzip();
        assert!(ys.iter().all(|y| (0.0..=100.0).contains(y)), "{path}");
        assert!(xs.windows(2).all(|w| w[0] <= w[1]), "{path}");
        // Flat stretches stay flat: the first segment's control points sit on y = 0.
        assert_eq!(&ys[1..4], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn a_smooth_line_through_two_points_is_straight() {
        let points = [Some((0.0, 10.0)), Some((100.0, 20.0))];
        assert_eq!(line(&points, Curve::Smooth), "M0,10L100,20");
    }

    #[test]
    fn a_smooth_area_closes_back_along_a_smooth_bottom() {
        let upper = [(0.0, 10.0), (50.0, 30.0), (100.0, 20.0)].map(Some);
        let lower = [(0.0, 60.0), (50.0, 90.0), (100.0, 70.0)].map(Some);
        let path = area(&upper, &lower, Curve::Smooth);
        assert!(
            path.starts_with("M0,10C") && path.ends_with("0,60Z"),
            "{path}"
        );
        assert_eq!(path.matches('C').count(), 4);
        assert!(path.contains("L100,70C"), "{path}");
    }

    #[test]
    fn non_finite_coordinates_count_as_missing() {
        let points = [Some((0.0, 1.0)), Some((1.0, f64::NAN)), Some((2.0, 3.0))];
        assert_eq!(line(&points, Curve::Linear), "M0,1L0,1M2,3L2,3");
    }
}
