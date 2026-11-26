//! Scales: a linear scale for values and evenly spaced bands for categories.

/// Maps a numeric domain onto a range, linearly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linear {
    domain: (f64, f64),
    range: (f64, f64),
}

impl Linear {
    /// A scale that maps `domain.0` to `range.0` and `domain.1` to `range.1`.
    ///
    /// Either pair may run in either direction. A range of `(100.0, 0.0)` puts larger
    /// values nearer the top of a y-down plot.
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self { domain, range }
    }

    /// The position of `value` in the range. Values outside the domain extrapolate.
    ///
    /// A domain of zero width maps every value to the middle of the range.
    pub fn map(&self, value: f64) -> f64 {
        let width = self.domain.1 - self.domain.0;
        let t = if width == 0.0 || !width.is_finite() {
            0.5
        } else {
            (value - self.domain.0) / width
        };
        self.range.0 + t * (self.range.1 - self.range.0)
    }
}

/// The slot one category occupies along the category axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    /// Where the band begins.
    pub start: f64,
    /// The band's extent along the axis.
    pub size: f64,
    /// The middle of the band.
    pub center: f64,
}

/// Splits `range` into `count` equal steps and returns the band inside each.
///
/// `padding` is the fraction of each step left empty, half on each side of the band.
/// It is clamped to `0.0..=0.95`. With `0.0` the bands tile the range.
pub fn bands(count: usize, range: (f64, f64), padding: f64) -> Vec<Band> {
    if count == 0 {
        return Vec::new();
    }
    let padding = if padding.is_finite() {
        padding.clamp(0.0, 0.95)
    } else {
        0.0
    };
    let step = (range.1 - range.0) / count as f64;
    let size = step * (1.0 - padding);
    (0..count)
        .map(|index| {
            let start = range.0 + step * index as f64 + step * padding / 2.0;
            Band {
                start,
                size,
                center: start + size / 2.0,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_maps_domain_to_range() {
        let s = Linear::new((0.0, 10.0), (100.0, 0.0));
        assert_eq!(s.map(2.5), 75.0);
        assert_eq!(s.map(0.0), 100.0);
        assert_eq!(s.map(10.0), 0.0);
    }

    #[test]
    fn linear_with_zero_width_domain_maps_to_range_midpoint() {
        assert_eq!(Linear::new((3.0, 3.0), (0.0, 100.0)).map(3.0), 50.0);
    }

    #[test]
    fn bands_split_range_with_padding_on_both_sides() {
        let b = bands(4, (0.0, 100.0), 0.2);
        assert_eq!(b.len(), 4);
        assert!((b[0].start - 2.5).abs() < 1e-9);
        assert!((b[0].size - 20.0).abs() < 1e-9);
        assert!((b[3].center - 87.5).abs() < 1e-9);
    }

    #[test]
    fn bands_without_padding_tile_the_range() {
        let b = bands(2, (0.0, 100.0), 0.0);
        assert_eq!((b[0].start, b[0].size, b[1].start), (0.0, 50.0, 50.0));
    }

    #[test]
    fn zero_bands_is_empty() {
        assert!(bands(0, (0.0, 100.0), 0.2).is_empty());
    }
}
