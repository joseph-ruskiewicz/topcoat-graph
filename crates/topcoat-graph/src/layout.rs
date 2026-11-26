//! Whole-chart layouts: everything a bar, line or area chart needs to draw itself.
//!
//! Positions are in the plot's `0 0 100 100` box: x runs right and y runs down.

use crate::{
    path::{self, Curve},
    scale::{Band, Linear, bands},
    stack::stack,
    ticks::nice,
};

/// Which way a chart's value axis runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Categories along the bottom, values going up.
    #[default]
    Vertical,
    /// Categories down the side, values going right.
    Horizontal,
}

/// How several bar series share a category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BarMode {
    /// Side by side.
    #[default]
    Grouped,
    /// On top of each other.
    Stacked,
}

/// A rectangle in the plot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// The left edge.
    pub x: f64,
    /// The top edge.
    pub y: f64,
    /// The extent to the right.
    pub width: f64,
    /// The extent downward.
    pub height: f64,
}

/// A mark on the value axis.
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    /// Where the tick sits along the value axis, in plot coordinates: measured from
    /// the top for a vertical chart and from the left for a horizontal one.
    pub position: f64,
    /// The tick's text.
    pub label: String,
}

/// One bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    /// The index of the series the bar belongs to.
    pub series: usize,
    /// The index of the bar's category.
    pub category: usize,
    /// The value the bar shows.
    pub value: f64,
    /// Where to draw it.
    pub rect: Rect,
}

/// Settings for [`bar_layout`].
#[derive(Clone, Debug, PartialEq)]
pub struct BarOptions {
    /// Which way the bars grow.
    pub orientation: Orientation,
    /// How several series share a category.
    pub mode: BarMode,
    /// About how many intervals the value axis should have.
    pub tick_target: usize,
    /// Keep ticks on whole numbers, for charts that count things.
    pub integer_ticks: bool,
    /// The fraction of each category's step left empty around its bars.
    pub band_padding: f64,
    /// The gap between grouped bars, as a fraction of the band. The gaps in a group
    /// never take more than half of the band between them.
    pub group_gap: f64,
}

impl Default for BarOptions {
    fn default() -> Self {
        Self {
            orientation: Orientation::Vertical,
            mode: BarMode::Grouped,
            tick_target: 5,
            integer_ticks: false,
            band_padding: 0.2,
            group_gap: 0.05,
        }
    }
}

/// A laid-out bar chart.
#[derive(Clone, Debug, PartialEq)]
pub struct BarLayout {
    /// The bars, by series and then by category. Missing and zero values have none.
    pub bars: Vec<Bar>,
    /// The value axis ticks, ascending by value.
    pub ticks: Vec<Tick>,
    /// Each category's slot along the category axis: left to right for a vertical
    /// chart, top to bottom for a horizontal one.
    pub bands: Vec<Band>,
    /// Where the zero line sits along the value axis.
    pub zero: f64,
}

/// Lays out a bar chart with `category_count` categories.
///
/// Each series holds one value per category. A non-finite value, or one past the end
/// of a short series, is missing; values past `category_count` are ignored. The value
/// axis always includes zero, and bars grow from the zero line, downward or leftward
/// for negative values.
pub fn bar_layout(category_count: usize, series: &[&[f64]], options: &BarOptions) -> BarLayout {
    let vertical = options.orientation == Orientation::Vertical;
    let bands = bands(category_count, (0.0, 100.0), options.band_padding);

    // Each bar's lower and upper bound in value space.
    let bounds: Vec<Vec<Option<(f64, f64)>>> = match options.mode {
        BarMode::Stacked => stack(series, category_count),
        BarMode::Grouped => series
            .iter()
            .map(|values| {
                (0..category_count)
                    .map(|category| value_at(values, category).map(|v| (v.min(0.0), v.max(0.0))))
                    .collect()
            })
            .collect(),
    };

    let axis = value_axis(
        extent(&bounds, true),
        options.tick_target,
        options.integer_ticks,
        vertical,
    );
    let scale = axis.scale;

    let slots = match options.mode {
        BarMode::Grouped => series.len().max(1),
        BarMode::Stacked => 1,
    };
    // All the gaps in a group together take at most half of its band.
    let gap = finite_or(options.group_gap, 0.0).clamp(0.0, 0.5 / (slots.max(2) - 1) as f64);
    let mut bars = Vec::new();
    for (index, values) in series.iter().enumerate() {
        for (category, band) in bands.iter().enumerate() {
            let Some(value) = value_at(values, category).filter(|v| *v != 0.0) else {
                continue;
            };
            let Some((lower, upper)) = bounds[index][category] else {
                continue;
            };
            let gap = band.size * gap;
            let thickness = (band.size - gap * (slots - 1) as f64) / slots as f64;
            let slot = if slots == 1 { 0 } else { index };
            let across = band.start + (thickness + gap) * slot as f64;
            let (a, b) = (scale.map(lower), scale.map(upper));
            let (along, length) = (a.min(b), (a - b).abs());
            let rect = if vertical {
                Rect {
                    x: across,
                    y: along,
                    width: thickness,
                    height: length,
                }
            } else {
                Rect {
                    x: along,
                    y: across,
                    width: length,
                    height: thickness,
                }
            };
            bars.push(Bar {
                series: index,
                category,
                value,
                rect,
            });
        }
    }

    BarLayout {
        bars,
        ticks: axis.ticks,
        bands,
        zero: scale.map(0.0),
    }
}

/// Settings for [`xy_layout`].
#[derive(Clone, Debug, PartialEq)]
pub struct XyOptions {
    /// How lines get from point to point.
    pub curve: Curve,
    /// Stack the series on top of each other.
    pub stacked: bool,
    /// Start the value axis at zero even when the data sits far from it. Stacked
    /// charts always do.
    pub include_zero: bool,
    /// About how many intervals the value axis should have.
    pub tick_target: usize,
    /// Keep ticks on whole numbers, for charts that count things.
    pub integer_ticks: bool,
}

impl Default for XyOptions {
    fn default() -> Self {
        Self {
            curve: Curve::Linear,
            stacked: false,
            include_zero: false,
            tick_target: 5,
            integer_ticks: false,
        }
    }
}

/// One series of a line or area chart.
#[derive(Clone, Debug, PartialEq)]
pub struct SeriesShape {
    /// Path data for the series' line.
    pub line: String,
    /// Path data for the area under the line: down to the zero line, or to the series
    /// below when stacked.
    pub area: String,
    /// The series' point in each category, `None` where the value is missing.
    pub points: Vec<Option<(f64, f64)>>,
}

/// A laid-out line or area chart.
#[derive(Clone, Debug, PartialEq)]
pub struct XyLayout {
    /// One shape per series, in input order.
    pub series: Vec<SeriesShape>,
    /// The value axis ticks, ascending by value.
    pub ticks: Vec<Tick>,
    /// Each category's slot along the bottom. Points sit at the band centers.
    pub bands: Vec<Band>,
    /// Where areas rest: the zero line, or the edge of the plot nearest zero when the
    /// axis does not include it.
    pub zero: f64,
}

/// Lays out a line or area chart with `category_count` categories.
///
/// Missing values follow the same rules as [`bar_layout`], and break a line in two.
pub fn xy_layout(category_count: usize, series: &[&[f64]], options: &XyOptions) -> XyLayout {
    let bands = bands(category_count, (0.0, 100.0), 0.0);

    let bounds: Vec<Vec<Option<(f64, f64)>>> = if options.stacked {
        stack(series, category_count)
    } else {
        series
            .iter()
            .map(|values| {
                (0..category_count)
                    .map(|category| value_at(values, category).map(|v| (0.0, v)))
                    .collect()
            })
            .collect()
    };

    // Stacked charts measure the stack; others hug the values themselves.
    let measured: Vec<Vec<Option<(f64, f64)>>> = if options.stacked {
        bounds.clone()
    } else {
        bounds
            .iter()
            .map(|b| b.iter().map(|b| b.map(|(_, v)| (v, v))).collect())
            .collect()
    };
    let with_zero = options.stacked || options.include_zero;
    let axis = value_axis(
        extent(&measured, with_zero),
        options.tick_target,
        options.integer_ticks,
        true,
    );
    let scale = axis.scale;
    let zero = scale.map(0.0f64.clamp(axis.domain.0, axis.domain.1));

    let shapes = bounds
        .iter()
        .map(|bounds| {
            let at = |pick: fn((f64, f64)) -> f64| -> Vec<Option<(f64, f64)>> {
                bounds
                    .iter()
                    .zip(&bands)
                    .map(|(bound, band)| bound.map(|b| (band.center, scale.map(pick(b)))))
                    .collect()
            };
            // A stacked series' own value is the bound away from zero.
            let points = at(|(lower, upper)| if lower < 0.0 { lower } else { upper });
            let base: Vec<Option<(f64, f64)>> = if options.stacked {
                at(|(lower, upper)| if lower < 0.0 { upper } else { lower })
            } else {
                points.iter().map(|p| p.map(|(x, _)| (x, zero))).collect()
            };
            SeriesShape {
                line: path::line(&points, options.curve),
                area: path::area(&points, &base, options.curve),
                points,
            }
        })
        .collect();

    XyLayout {
        series: shapes,
        ticks: axis.ticks,
        bands,
        zero,
    }
}

/// The value at `category`, unless it is missing.
fn value_at(values: &[f64], category: usize) -> Option<f64> {
    values.get(category).copied().filter(|v| v.is_finite())
}

fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() { value } else { fallback }
}

/// The smallest and largest bound, widened to include zero when asked.
fn extent(bounds: &[Vec<Option<(f64, f64)>>], with_zero: bool) -> (f64, f64) {
    let mut all = bounds
        .iter()
        .flatten()
        .flatten()
        .flat_map(|&(lower, upper)| [lower, upper]);
    let first = if with_zero { Some(0.0) } else { all.next() };
    let Some(first) = first else {
        return (0.0, 0.0);
    };
    all.fold((first, first), |(min, max), v| (min.min(v), max.max(v)))
}

/// A value axis: its scale, the domain the scale covers, and its ticks.
struct ValueAxis {
    scale: Linear,
    domain: (f64, f64),
    ticks: Vec<Tick>,
}

/// The value axis for an extent of values.
fn value_axis(extent: (f64, f64), tick_target: usize, integer: bool, vertical: bool) -> ValueAxis {
    let ticks = nice(extent.0, extent.1, tick_target, integer);
    let domain = ticks.domain();
    // Larger values sit higher on a vertical chart and further right on a horizontal one.
    let range = if vertical { (100.0, 0.0) } else { (0.0, 100.0) };
    let scale = Linear::new(domain, range);
    let marks = ticks
        .values
        .iter()
        .map(|&value| Tick {
            position: scale.map(value),
            label: ticks.label(value),
        })
        .collect();
    ValueAxis {
        scale,
        domain,
        ticks: marks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(ticks: &[Tick]) -> Vec<&str> {
        ticks.iter().map(|t| t.label.as_str()).collect()
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    const CURVE: [f64; 7] = [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0];

    #[test]
    fn a_mana_curve_lays_out_as_bars_from_the_zero_line() {
        let options = BarOptions {
            integer_ticks: true,
            ..Default::default()
        };
        let l = bar_layout(7, &[&CURVE], &options);
        assert_eq!(l.bars.len(), 6, "the zero has no bar");
        assert_eq!(labels(&l.ticks), ["0", "2", "4", "6", "8", "10", "12"]);
        assert_eq!(l.zero, 100.0);
        assert_eq!(l.ticks[0].position, 100.0);
        assert_eq!(l.ticks[6].position, 0.0);
        let tallest = &l.bars[2];
        assert_eq!(
            (tallest.series, tallest.category, tallest.value),
            (0, 2, 12.0)
        );
        assert_eq!((tallest.rect.y, tallest.rect.height), (0.0, 100.0));
        assert!(close(tallest.rect.x, l.bands[2].start));
        assert!(close(tallest.rect.width, l.bands[2].size));
        let shortest = &l.bars[5];
        assert!(close(shortest.rect.y + shortest.rect.height, 100.0));
        assert!(close(shortest.rect.height, 100.0 / 12.0));
    }

    #[test]
    fn empty_and_all_zero_bar_charts_keep_their_frame() {
        let e = bar_layout(0, &[], &BarOptions::default());
        assert!(e.bars.is_empty() && e.bands.is_empty());
        assert_eq!(labels(&e.ticks).first(), Some(&"0"));
        assert_eq!(labels(&e.ticks).last(), Some(&"1"));
        let z = bar_layout(3, &[&[0.0, 0.0, 0.0]], &BarOptions::default());
        assert!(z.bars.is_empty());
        assert_eq!(z.bands.len(), 3);
        let n = bar_layout(2, &[&[f64::NAN, f64::INFINITY]], &BarOptions::default());
        assert!(n.bars.is_empty());
        assert_eq!(n.bands.len(), 2);
    }

    #[test]
    fn series_and_categories_of_different_lengths_do_not_shift_or_stretch() {
        let m = bar_layout(3, &[&[1.0], &[1.0, 2.0, 3.0, 99.0]], &BarOptions::default());
        assert_eq!(m.bars.iter().filter(|b| b.series == 0).count(), 1);
        assert_eq!(m.bars.iter().filter(|b| b.series == 1).count(), 3);
        assert_eq!(
            labels(&m.ticks).last(),
            Some(&"3"),
            "the ignored 99 doesn't stretch the axis"
        );
        assert!(m.bars.iter().all(|b| b.category < 3));
    }

    #[test]
    fn small_counts_get_whole_number_ticks() {
        let options = BarOptions {
            integer_ticks: true,
            ..Default::default()
        };
        let small = bar_layout(2, &[&[1.0, 2.0]], &options);
        assert_eq!(labels(&small.ticks), ["0", "1", "2"]);
    }

    #[test]
    fn grouped_bars_share_a_band_without_overlapping() {
        let l = bar_layout(1, &[&[1.0], &[2.0], &[3.0]], &BarOptions::default());
        let band = l.bands[0];
        assert!(close(l.bars[0].rect.x, band.start));
        for pair in l.bars.windows(2) {
            assert!(pair[0].rect.x + pair[0].rect.width < pair[1].rect.x);
        }
        let last = &l.bars[2].rect;
        assert!(close(last.x + last.width, band.start + band.size));
    }

    #[test]
    fn a_missing_grouped_value_leaves_its_slot_empty() {
        let both = bar_layout(1, &[&[1.0], &[2.0]], &BarOptions::default());
        let one = bar_layout(1, &[&[f64::NAN], &[2.0]], &BarOptions::default());
        assert_eq!(one.bars.len(), 1);
        assert!(close(one.bars[0].rect.x, both.bars[1].rect.x));
    }

    #[test]
    fn stacked_bars_sit_on_each_other() {
        let options = BarOptions {
            mode: BarMode::Stacked,
            ..Default::default()
        };
        let l = bar_layout(2, &[&[1.0, 2.0], &[3.0, 4.0]], &options);
        assert_eq!(labels(&l.ticks).last(), Some(&"6"));
        let (lower, upper) = (&l.bars[1].rect, &l.bars[3].rect);
        assert_eq!((l.bars[1].category, l.bars[3].category), (1, 1));
        assert!(close(upper.y + upper.height, lower.y));
        assert!(close(upper.y, 0.0));
        assert!(close(upper.x, lower.x) && close(upper.width, l.bands[1].size));
    }

    #[test]
    fn negative_bars_hang_below_the_zero_line() {
        let l = bar_layout(2, &[&[5.0, -5.0]], &BarOptions::default());
        assert_eq!(l.zero, 50.0);
        assert!(close(l.bars[0].rect.y + l.bars[0].rect.height, 50.0));
        assert!(close(l.bars[1].rect.y, 50.0));
        assert!(l.bars[1].rect.height > 0.0);
    }

    #[test]
    fn horizontal_bars_grow_to_the_right() {
        let options = BarOptions {
            orientation: Orientation::Horizontal,
            ..Default::default()
        };
        let l = bar_layout(2, &[&[5.0, 10.0]], &options);
        assert_eq!(l.zero, 0.0);
        assert_eq!(l.ticks[0].position, 0.0);
        let (first, second) = (&l.bars[0].rect, &l.bars[1].rect);
        assert_eq!((first.x, first.width), (0.0, 50.0));
        assert_eq!((second.x, second.width), (0.0, 100.0));
        assert!(first.y < second.y, "categories run top to bottom");
        assert!(close(first.height, l.bands[0].size));
    }

    #[test]
    fn a_line_has_a_point_at_each_band_center_and_breaks_at_a_gap() {
        let l = xy_layout(3, &[&[1.0, f64::NAN, 3.0]], &XyOptions::default());
        assert_eq!(l.bands.len(), 3);
        let points = &l.series[0].points;
        assert_eq!(points[1], None);
        assert!(close(points[0].unwrap().0, l.bands[0].center));
        assert!(close(points[0].unwrap().0, 100.0 / 6.0));
        assert_eq!(l.series[0].line.matches('M').count(), 2);
    }

    #[test]
    fn a_line_axis_hugs_the_data_unless_zero_is_asked_for() {
        let tight = xy_layout(2, &[&[100.0, 120.0]], &XyOptions::default());
        assert_eq!(labels(&tight.ticks).first(), Some(&"100"));
        let options = XyOptions {
            include_zero: true,
            ..Default::default()
        };
        let based = xy_layout(2, &[&[100.0, 120.0]], &options);
        assert_eq!(labels(&based.ticks).first(), Some(&"0"));
        assert_eq!(based.zero, 100.0);
        // Without zero in the domain, areas fill down to the bottom of the plot.
        assert_eq!(tight.zero, 100.0);
        assert!(tight.series[0].area.ends_with("Z"));
    }

    #[test]
    fn stacked_areas_rest_on_the_series_below() {
        let options = XyOptions {
            stacked: true,
            ..Default::default()
        };
        let l = xy_layout(2, &[&[1.0, 2.0], &[3.0, 2.0]], &options);
        assert_eq!(labels(&l.ticks).last(), Some(&"4"));
        let below = l.series[0].points.clone();
        let (x0, y0) = below[0].unwrap();
        let (x1, y1) = below[1].unwrap();
        let expected = format!(
            "L{},{}L{},{}Z",
            crate::fmt::num(x1),
            crate::fmt::num(y1),
            crate::fmt::num(x0),
            crate::fmt::num(y0)
        );
        assert!(
            l.series[1].area.ends_with(&expected),
            "{}",
            l.series[1].area
        );
        assert_eq!(
            l.series[1].points[0].unwrap().1,
            0.0,
            "the top series reaches 1 + 3 = 4"
        );
    }

    #[test]
    fn an_empty_line_chart_keeps_its_frame() {
        let l = xy_layout(0, &[], &XyOptions::default());
        assert!(l.series.is_empty() && l.bands.is_empty());
        assert_eq!(labels(&l.ticks), ["0", "0.2", "0.4", "0.6", "0.8", "1"]);
        let l = xy_layout(2, &[&[]], &XyOptions::default());
        assert_eq!(l.series[0].line, "");
        assert_eq!(l.series[0].points, [None, None]);
    }

    #[test]
    fn nothing_emits_non_finite_numbers() {
        let wild: [&[f64]; 3] = [&[f64::NAN, 1e308, -1e308], &[f64::INFINITY, 0.0], &[]];
        for stacked in [false, true] {
            let options = XyOptions {
                stacked,
                curve: Curve::Smooth,
                ..Default::default()
            };
            let l = xy_layout(3, &wild, &options);
            for shape in &l.series {
                assert!(!shape.line.contains("NaN") && !shape.line.contains("inf"));
                assert!(!shape.area.contains("NaN") && !shape.area.contains("inf"));
            }
            let mode = if stacked {
                BarMode::Stacked
            } else {
                BarMode::Grouped
            };
            let l = bar_layout(
                3,
                &wild,
                &BarOptions {
                    mode,
                    ..Default::default()
                },
            );
            for bar in &l.bars {
                let r = &bar.rect;
                assert!([r.x, r.y, r.width, r.height].iter().all(|v| v.is_finite()));
            }
            assert!(l.ticks.iter().all(|t| t.position.is_finite()));
        }
    }

    #[test]
    fn stacks_that_overflow_stay_finite() {
        let options = BarOptions {
            mode: BarMode::Stacked,
            ..Default::default()
        };
        let l = bar_layout(1, &[&[1e308], &[1e308], &[-1e308], &[-1e308]], &options);
        assert_eq!(l.bars.len(), 4);
        for bar in &l.bars {
            let r = &bar.rect;
            assert!(
                [r.x, r.y, r.width, r.height].iter().all(|v| v.is_finite()),
                "{r:?}"
            );
        }
        let options = XyOptions {
            stacked: true,
            ..Default::default()
        };
        let l = xy_layout(1, &[&[1e308], &[1e308]], &options);
        assert!(
            l.series
                .iter()
                .all(|s| s.points[0].is_some_and(|(x, y)| x.is_finite() && y.is_finite()))
        );
    }

    #[test]
    fn a_wide_group_gap_never_squeezes_bars_to_nothing() {
        let options = BarOptions {
            group_gap: 0.5,
            ..Default::default()
        };
        let l = bar_layout(1, &[&[1.0], &[1.0], &[1.0], &[1.0]], &options);
        assert!(l.bars.iter().all(|b| b.rect.width > 0.0));
        let last = &l.bars[3].rect;
        assert!(close(
            last.x + last.width,
            l.bands[0].start + l.bands[0].size
        ));
    }
}
