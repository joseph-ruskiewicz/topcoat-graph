//! Charts for [Topcoat](https://github.com/tokio-rs/topcoat): line, bar, area and pie.
//!
//! This crate is two things:
//!
//! - A `topcoat ui` component registry. Add the crate as a dependency, then copy the
//!   chart components into your app:
//!
//!   ```sh
//!   topcoat ui add bar_chart --registry topcoat-graph
//!   ```
//!
//! - The chart geometry those components call: scales, ticks, stacking, paths and
//!   slices. It is plain Rust with no dependencies and knows nothing about HTML, so it
//!   can be used without the components.
//!
//! # Coordinates
//!
//! Every layout is computed in a `0 0 100 100` box, x running right and y running
//! down, so positions double as percentages. A component draws the plot as an SVG with
//! that `viewBox`, stretched to any size, and places labels over it in HTML.
//!
//! # Missing values
//!
//! A non-finite value (`f64::NAN`, an infinity) means "no value here". A bar chart
//! leaves a gap for it and a line breaks around it. Nothing in this crate panics or
//! writes `NaN` into its output, whatever the input: an empty chart still gets an axis.
//!
//! # Example
//!
//! The mana curve of a deck: how many cards cost 0, 1, 2 and so on.
//!
//! ```
//! use topcoat_graph::{BarOptions, bar_layout};
//!
//! let counts = [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0];
//! let options = BarOptions { integer_ticks: true, ..Default::default() };
//! let layout = bar_layout(counts.len(), &[&counts], &options);
//!
//! // One bar per nonzero count, and whole-number ticks from 0 to 12.
//! assert_eq!(layout.bars.len(), 6);
//! assert_eq!(layout.ticks.first().unwrap().label, "0");
//! assert_eq!(layout.ticks.last().unwrap().label, "12");
//!
//! // The tallest bar fills the plot from top to bottom.
//! let tallest = &layout.bars[2];
//! assert_eq!((tallest.rect.y, tallest.rect.height), (0.0, 100.0));
//! ```

pub mod fmt;
pub mod layout;
pub mod path;
pub mod pie;
pub mod scale;
pub mod stack;
pub mod ticks;

pub use layout::{
    Bar, BarLayout, BarMode, BarOptions, Orientation, Rect, SeriesShape, Tick, XyLayout, XyOptions,
    bar_layout, xy_layout,
};
pub use path::Curve;
pub use pie::{PieOptions, Slice, pie_layout};
pub use scale::Band;
