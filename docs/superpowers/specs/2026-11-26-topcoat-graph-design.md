# topcoat-graph design

Date: 2026-11-26

## Intent

Charts for [Topcoat](https://github.com/tokio-rs/topcoat) apps, in the spirit of shadcn
charts: line, bar, area and pie, themed with the same tokens as the built-in UI, for
dashboards in Joseph's own apps.

The first real use is a mana cost distribution chart on a deck page in
`../contested-realm`. That app runs Topcoat 0.12 on the `neutral` theme and already
installs components from two registries (`topcoat` and `topcoat-prompt`).

topcoat-graph is a community project, like `../topcoat-prompt`, and follows that repo's
conventions where they apply.

## Goals

- Four chart types: bar, line, area, pie.
- Charts render on the server as SVG and are readable with no script loaded.
- Hover tooltips and hover highlight through the Topcoat runtime. No hand-written
  JavaScript file, no client build step, no wasm.
- Charts fill the width of their container.
- Colors come from theme tokens and work in light and dark mode.
- Installed with `topcoat ui add ... --registry topcoat-graph`, the same way as
  `topcoat-prompt`.
- A gallery site in the repo that shows every chart and doubles as the compile and test
  check for the component sources.
- The mana curve renders on a contested-realm deck page.

## Non-goals (first version)

- Animation and transitions.
- Zoom, pan, brush selection.
- Live-streaming data. A chart inside a `#[shard]` re-renders with new data like any
  other markup; nothing beyond that.
- Scatter, radar, radial bar, and other chart types.
- Time-scale axes with date-aware ticks. X axes are categories; y axes are numbers.
- Publishing to crates.io. That happens when Joseph asks for it.

## Anti-goals

- A chart that is blank or meaningless until JavaScript runs.
- A second styling system. Charts use the theme's tokens and Tailwind classes, nothing
  parallel.
- Math that is copied into every app. See Approaches.

## Constraints

- Topcoat 0.12.x, Rust edition 2024, `rust-version` 1.88, matching `topcoat-prompt`.
- The Topcoat runtime is experimental. Runtime expressions support a limited Rust subset
  (`../topcoat/docs/runtime/expr.md`). Interactivity must fit that subset; `raw!` is the
  escape hatch and should be rare.
- `topcoat ui` copies component sources into the app's shared components directory, next
  to built-ins and other registries' components. Component names must not collide.
- Built-in components reach each other as sibling modules (`super::button`).

## Approaches

Three were considered.

**A. Registry only.** The crate ships no code, like `topcoat-prompt`. All math lives in a
copied component. One install path and the app owns every line. But several hundred lines
of math get copied into each app, fixes arrive only through `topcoat ui add --overwrite`,
and the hardest code is the least convenient to test and version.

**B. Math crate plus copy-in components. (Chosen.)** `topcoat-graph` is a real crate of
pure Rust: scales, ticks, stacking, path and arc geometry. Registry components on top are
thin `view!` code that turns those numbers into SVG and classes. Apps restyle the
components freely; math fixes arrive with `cargo update`. The cost is that copied
components depend on the crate's API, so a breaking change in the crate means updating
installed components.

**C. Compiled-in components.** No registry; apps call `topcoat_graph::bar_chart(...)`.
Simplest to use, but restyling is limited to exposed props, it departs from how every
other Topcoat UI piece works, and it pins the crate to Topcoat's macro internals while
Topcoat is still shipping breaking releases.

Joseph picked B. The math is the part worth testing hard and fixing centrally; the markup
is the part an app wants to bend.

## Decided by Joseph

- Charts, not node-and-edge graphs.
- Line, bar, area, pie. "Like shadcn charts."
- For dashboards in his own apps; first use is the contested-realm mana curve.
- Approach B.
- The first-version scope listed under Goals and Non-goals, including: stacked and
  grouped bars, vertical or horizontal; multi-series line and area with straight or
  smooth curves; stacked area; donut variant with a center label; axes with nice ticks,
  grid lines, legend, tooltip; a gallery site.
- Model split for the build: Fable does the hard parts, Opus does everything else. See
  Build split.
- If the work turns out to need another Topcoat library, stop and tell him; that becomes
  its own project. Nothing found so far needs one.

## Design

Everything in this section is Claude's call unless listed above. Each is open to
correction.

### Repository layout

```
Cargo.toml                      workspace: crates/topcoat-graph, site
crates/topcoat-graph/
  Cargo.toml                    [package.metadata.topcoat-ui] registry = "registry"
  src/                          the math crate (compiled)
  registry/
    registry.toml
    components/*.rs             component sources (copied into apps, not compiled here)
site/                           gallery; installs every component
docs/superpowers/               specs and plans
```

Component sources live under `registry/`, apart from `src/`, so it's clear which code
compiles into the crate and which is copied. (`topcoat-prompt` keeps sources in `src/`
because it has no compiled code to confuse them with.)

### The math crate

Pure Rust, `f64` throughout, no dependencies, no Topcoat dependency, no `unsafe`. It
knows nothing about HTML. Modules:

- **scale**: linear scale (domain to range) and band scale (categories to evenly spaced
  bands, with inner and outer padding; grouped bars subdivide a band).
- **ticks**: "nice" tick selection for a numeric domain and a target tick count. Steps
  are 1, 2 or 5 times a power of ten. The domain widens to the outer ticks. Bar and area
  charts force zero into the domain. Tick labels format without float noise (`0.3`, not
  `0.30000000000000004`).
- **stack**: stacks series into lower and upper bounds per category. Positive and
  negative values stack in opposite directions from zero.
- **path**: SVG path data for lines and areas. Curves: straight, and smooth using
  monotone cubic interpolation in x, which never overshoots the data. A missing value
  (`None`) breaks the line into separate segments.
- **arc**: pie and donut slice geometry from values, with start angle, padding angle and
  inner radius. Path data plus each slice's centroid.
- **layout**: plot-area rectangle from outer size and margins, and tooltip anchor points
  as fractions of the plot area.

Input rules: non-finite numbers are treated as missing. An empty dataset, a single
point, and an all-equal domain all produce a valid drawable result, never a panic or
`NaN` in output. Pie treats negative values as zero. Numbers in emitted path data are
rounded to a fixed precision so output is stable and compact.

This crate carries the test weight: unit tests per module, edge cases above, and
expected path strings for known inputs.

### Components

Installed names are prefixed so they don't collide in the shared components directory:

| Component | What it is |
| --- | --- |
| `chart` | Container, shared data types (series, config), color token helper, hidden data table for assistive tech. Every chart depends on it. |
| `chart_axis` | X and y axis labels and grid lines. |
| `chart_legend` | Color swatch and label per series or slice. |
| `chart_tooltip` | Tooltip panel: category label, then a swatch, name and value per series. |
| `bar_chart` | Vertical or horizontal; single, grouped or stacked. |
| `line_chart` | One or more series; straight or smooth; optional dots. |
| `area_chart` | One or more series; straight or smooth; optional stacking. |
| `pie_chart` | Pie, or donut with a center label. |

Each chart type is one component with props, built from the smaller parts. An app that
wants a different arrangement edits its copy. shadcn composes charts from child elements,
which relies on React context to share scales; that doesn't translate, and owning the
source gives the same freedom.

Data goes in as prepared values, not accessor closures: a list of category labels and a
list of series, each with a name, a color slot and one value per category. This keeps
component props free of generics. The mana curve is seven or so labels and one series of
counts. Conventions follow the built-ins: enum props for variants, `attrs` for extra
attributes on the root element.

Sketch (exact prop syntax is the builder's to settle against Topcoat's component rules):

```rust
bar_chart(
    categories: &["0", "1", "2", "3", "4", "5", "6+"],
    series: &[ChartSeries::new("Cards", &counts)],
    label: "Mana cost distribution",
)
```

### Rendering and sizing

A chart is a relatively positioned container with a set aspect ratio or height.

For bar, line and area charts, the plot is an SVG that stretches to the container
(`preserveAspectRatio="none"`), with strokes kept at constant width
(`vector-effect: non-scaling-stroke`). Tick labels, axis titles, legend and tooltip are
HTML, positioned by percentage over or around the plot. Text stays at its real size at
any chart width, which a plain scaled `viewBox` would not give without measuring in
JavaScript. Line dots are drawn so they stay round under stretching (for example
zero-length round-capped strokes).

Pie charts use an ordinary square `viewBox`, since they must not stretch.

The number of ticks is fixed at render time; it does not adapt to width. A prop sets the
target count.

### Interactivity

One signal per chart holds the hovered index (`Option<usize>`): a category for bar, line
and area; a slice for pie.

- Each category gets a transparent full-height hit band over the plot, so hovering
  anywhere in a column counts. Pie slices are their own hit targets. Entering sets the
  signal; leaving the chart clears it.
- The server renders one tooltip per category, hidden, positioned from the layout
  anchors. A bind attribute shows the one matching the signal. No cursor tracking and no
  nearest-point math in the browser.
- The hovered category gets a highlight (a cursor band or line; a dimming of other pie
  slices) through bound classes.
- Hit targets are focusable so the tooltip also works from the keyboard.

All of this fits the runtime's expression subset, checked against Topcoat's source:

- Events and binds work on SVG elements. Hydration walks every element
  (`topcoat-runtime/browser/src/dom/hydrate.ts`), handlers use `addEventListener`, and
  binds write with `setAttribute`, so `:class` works on `<rect>` and `<path>`.
- Don't bind camelCase SVG attributes (`:viewBox`): the bind name is read back from the
  DOM, which lowercases it. The design doesn't need to.
- `&&`, `||` and `Option` equality aren't in the subset. The hover test is written
  `if h.is_some() { h.unwrap() == i } else { false }`.
- A signal's identity comes from `cx` and the call site, so each chart instance has its
  own. Charts rendered in a `view!` loop need `#[key]`.
- Signal values coming back from the browser are untrusted. An out-of-range hover index
  must render as "nothing hovered", never panic.

If the builder finds something that doesn't fit, `raw!` is acceptable for a small piece.
If it needs more than that, stop and report: that is the "another Topcoat library" case.

### Theming

Series colors are five tokens, `--chart-1` through `--chart-5`, as in shadcn. The
neutral theme doesn't define them. Components read them with built-in fallbacks, so
charts look right with no setup. Dark mode in the neutral theme is a `.dark` class, so
the fallbacks need a light value and a `dark:` value. The README shows
the block to add to `styles.css` for light and dark values. A series can also be given
an explicit color. More than five series cycle.

Everything else (text, grid, tooltip surface, borders) uses existing neutral tokens such
as `text-muted-foreground`, `border`, `bg-popover`.

### Accessibility

Each chart has an accessible name from its `label` prop and includes its data as a
visually hidden table, so the numbers are available without sight or hover. The SVG
itself is presentational.

Every piece of text a chart shows or announces (label, table headers, series names)
comes in as a prop. Components contain no hardcoded English, so localized apps such as
contested-realm can pass `t!` values.

### Gallery site

A Topcoat app in `site/`, modeled on `topcoat-prompt/site` but smaller: one page per
chart type showing its variants with the source for each, plus an install page. It
installs every component through `topcoat ui`, so building it compiles every component
source, and it carries a sync test like `topcoat-prompt/site/tests/registry_sync.rs`
that fails when installed copies drift from the registry sources. Deployment is out of
scope for now.

### Testing

- Math crate: thorough unit tests, as above.
- Components: render tests in the site crate that check the produced markup for known
  data (bar count and geometry, path data present, tooltip count, table contents).
- A browser check of hover and keyboard tooltips on the gallery before calling it done.
- CI as in `topcoat-prompt`: fmt, clippy with warnings denied, tests.

### contested-realm

The last milestone adds the mana curve to contested-realm. Decided by Joseph:

- It shows on both deck pages: the owner's deck builder (`src/app/decks/deck_id.rs`) and
  the public deck page (`src/app/public/owner_id/deck_id.rs`).
- contested-realm depends on topcoat-graph as a git dependency on the GitHub repo for
  now, since its Dockerfile and CI build from its own directory and can't resolve a
  sibling path. It switches to a crates.io version when the crate is published.

Proposed by Claude and not objected to: the curve counts the spellbook only (sites and
avatars have no cost), counts every copy, has one bar per cost from 0 up with a final
"6+" bucket, and uses a single color. Stacking by element can follow later.

On the deck builder the deck list re-renders on each card change through a
`version: Signal<u64>`; the curve should update with it. Labels come from Fluent (`t!`).

The work happens in the contested-realm repo, on a branch.

### Hosting

Decided by Joseph: the repo is public at `joseph-ruskiewicz/topcoat-graph` on GitHub,
next to `topcoat-prompt`, with the same GitHub Actions CI.

## Build split

- **Fable (hard parts):** the math crate, the rendering and sizing technique, the
  interactivity mechanism and its fit with the runtime, the first chart component end to
  end (bar) as the pattern, and review of everything else.
- **Opus (everything else):** the remaining chart components following the bar pattern,
  legend and tooltip parts, the gallery site, README and docs, CI.

## Left to the builder

- Exact prop names and types, and the Topcoat component syntax details.
- A quick early spike of a stretched SVG with a hover bind, to confirm the technique end
  to end. It's expected to pass: `topcoat/demos/admin/src/components/chart.rs` already
  renders a stretched line and bar chart with `viewBox`, `preserveAspectRatio="none"`
  and `vector-effect="non-scaling-stroke"`, and is worth reading first.
- Default colors for the five chart tokens in light and dark.
- Default margins, tick counts, bar padding, corner radius.
- Module and file organization inside the crate and the site.
- Gallery page design.
