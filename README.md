# topcoat-graph

Charts for [Topcoat](https://github.com/tokio-rs/topcoat): bar, line, area and pie. Charts
render on the server as SVG with HTML labels, so they read fine with no script loaded. With
the Topcoat runtime on the page, pointing at a category or tabbing to it shows its values in
a tooltip. Colors come from your app's theme tokens and work in light and dark mode.

topcoat-graph is a community project, not part of Topcoat or maintained by tokio-rs.

It has two parts. The first is a `topcoat ui` component registry: the chart components are
copied into your app as source you own, like the built-in components, so you can restyle or
rearrange them freely. The second is a small crate of chart geometry (scales, ticks,
stacking, paths and pie slices) that the components call. The geometry is plain Rust with
no dependencies, and fixes to it arrive with `cargo update`.

## Install

Your app needs Topcoat's `ui` and `tailwind` features and a theme from `topcoat ui init`.
The `runtime` feature, which tooltips use, is on by default. The crate is not on crates.io
yet, so add it from GitHub:

```sh
cargo add topcoat --features ui,tailwind
cargo add topcoat-graph --git https://github.com/joseph-ruskiewicz/topcoat-graph
topcoat ui init --theme neutral
topcoat ui add bar_chart --registry topcoat-graph
```

`topcoat ui` only offers registries that are direct dependencies of your app, which is why
`topcoat-graph` is in `[dependencies]`. Unlike a registry crate with no code, this one adds
a little to your binary: the geometry the components call.

The parts a chart is built from, such as `chart` and `chart_axis`, are installed with it.
To install everything:

```sh
topcoat ui add --all --registry topcoat-graph
```

As with any `topcoat ui` app, the installed `styles.css` is your Tailwind input. Render it
in `build.rs`, with `topcoat` also in `[build-dependencies]` with the `tailwind` feature:

```rust
fn main() {
    topcoat::tailwind::BuildConfig::new()
        .input("styles.css")
        .render()
        .unwrap();
}
```

### Runtime setup for tooltips

Tooltips need the runtime's browser script in the document head and a `RuntimeLayer` on
the router, with the asset bundle that serves the script:

```rust
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt, module_router},
    runtime::RuntimeLayer,
};

fn router() -> Router {
    module_router!()
        .assets(AssetBundle::load().unwrap())
        .discover()
        .layer(RuntimeLayer::new())
        .build()
}
```

```rust
view! {
    <head>
        topcoat::runtime::script()
        <link rel="stylesheet" href=(tailwind::stylesheet!())>
    </head>
}
```

Without the runtime, charts still render in full: marks, axes, labels and the data table.
Only the tooltips and hover highlight are missing.

## Components

| Name | What it is |
| --- | --- |
| `chart` | The frame every chart sits in: a named figure, the `ChartSeries` type, the `labels` helper, color defaults, the hidden data table and the hover targets. Every chart depends on it. |
| `chart_axis` | Value axis labels, category axis labels and grid lines. Category labels truncate within their band. |
| `chart_legend` | A color swatch and name per series or slice, wrapping onto more lines as needed. |
| `chart_tooltip` | The panel shown for a hovered or focused category: its label, then a swatch, name and value per series. |
| `bar_chart` | Bars, vertical or horizontal, with several series grouped side by side or stacked. Negative values grow down from the zero line. |
| `line_chart` | One or more lines, straight or smooth (`curve`), with optional dots (`dots`). `include_zero` starts the value axis at zero, and `legend` shows a legend. |
| `area_chart` | One or more filled areas, straight or smooth (`curve`), optionally stacked (`stacked`), with `legend`. The value axis always starts at zero. |
| `pie_chart` | A pie from `slices: Vec<PieSlice>`, each made with `PieSlice::new(name, value)`, or a donut with `donut: true` and its center content passed as children. `value_header` names the values in the tooltip and data table. Each slice has its own tooltip. `legend` shows a legend. |

### A bar chart

A deck's mana curve, the number of cards at each mana cost:

```rust
use topcoat::view::view;

use crate::components::{
    bar_chart::bar_chart,
    chart::{ChartSeries, labels},
};

view! {
    bar_chart(
        label: "Mana cost distribution",
        category_header: "Cost",
        categories: labels(["0", "1", "2", "3", "4", "5", "6+"]),
        series: vec![ChartSeries::new("Cards", [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0])],
        integer_ticks: true
    )
}
```

`label` is the chart's accessible name. `category_header` heads the category column of the
chart's data table. Each `ChartSeries` has a name and one value per category.

### Props worth knowing

- `integer_ticks: true` keeps the value axis on whole numbers, for charts that count things.
- `label_every: 5` shows one category label in five, for charts with many categories.
- `orientation: Orientation::Horizontal` draws bars from the left, with categories down the
  side. It suits long labels.
- `mode: BarMode::Stacked` stacks several series on each other instead of grouping them.
- `attrs` sets attributes on the chart's root element. A chart is as wide as its container
  with a 16:9 shape by default. For a fixed height, pass
  `attrs: attributes! { class="aspect-auto h-64" }`. A pie chart is square, centered and at
  most 18rem wide. To change its width, pass a class that overrides, such as `max-w-96!`.
- A missing value is `f64::NAN` (any non-finite number). A bar chart leaves a gap, a line
  breaks around it, and the tooltip leaves it out.

`Orientation` and `BarMode` come from the crate: `use topcoat_graph::{BarMode, Orientation};`.

## Accessibility

Each chart is a figure named by its `label` prop, and its data is in a visually hidden
table, so the numbers are there without sight or hover. The drawing itself is hidden from
screen readers. The hover targets, one per category or pie slice, are named and
keyboard-focusable, so Tab moves through them and shows each tooltip. Every piece of text a chart shows or announces comes from props, so
an app can pass localized strings.

## Colors

Series take the colors `--chart-1` to `--chart-5` in turn, and more than five series cycle.
The neutral theme doesn't define these tokens, so the components carry built-in defaults
for light and dark mode. To set your own, add them to `styles.css` under `:root` and
`.dark`. These are the defaults:

```css
:root {
  --chart-1: oklch(0.646 0.222 41.116);
  --chart-2: oklch(0.6 0.118 184.704);
  --chart-3: oklch(0.398 0.07 227.392);
  --chart-4: oklch(0.828 0.189 84.429);
  --chart-5: oklch(0.769 0.188 70.08);
}

.dark {
  --chart-1: oklch(0.488 0.243 264.376);
  --chart-2: oklch(0.696 0.17 162.48);
  --chart-3: oklch(0.769 0.188 70.08);
  --chart-4: oklch(0.627 0.265 303.9);
  --chart-5: oklch(0.645 0.246 16.439);
}
```

To give one series a color of its own, pass any CSS color:

```rust
ChartSeries::new("Cards", [2.0, 5.0, 12.0]).color("var(--primary)")
```

Text, grid lines and the tooltip surface use the theme's existing tokens, such as
`text-muted-foreground`, `border` and `bg-popover`.

## Using the geometry on its own

The crate's layouts work without the components, for drawing charts your own way:

```rust
use topcoat_graph::{BarOptions, bar_layout};

let counts = [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0];
let options = BarOptions { integer_ticks: true, ..Default::default() };
let layout = bar_layout(counts.len(), &[&counts], &options);

for bar in &layout.bars {
    // bar.series, bar.category, bar.value, and bar.rect.{x, y, width, height}
}
for tick in &layout.ticks {
    // tick.position and tick.label: "0" to "12"
}
```

Every layout is computed in a `0 0 100 100` box, x running right and y running down, so
positions double as percentages. Draw the plot as an SVG with that `viewBox` and
`preserveAspectRatio="none"`, and place labels over it in HTML. A non-finite value means
"no value here". Nothing in the crate panics or writes `NaN` into its output, and an empty
chart still gets an axis. `xy_layout` does the same for line and area charts and
`pie_layout` for pies.

## Development

- `crates/topcoat-graph/src` is the geometry crate.
- `crates/topcoat-graph/registry` holds the component sources and `registry.toml`. No crate
  compiles them here.
- `site` is a gallery that installs every component. It is their compile and test check.

Run the gallery:

```sh
cd site
topcoat dev
```

Edit components in the registry, never the copies in `site/src/components`. After editing
one, format the sources and reinstall them into the site:

```sh
./scripts/sync-site.sh
```

`cargo test --workspace` fails if an installed copy differs from its registry source.

## License

MIT
