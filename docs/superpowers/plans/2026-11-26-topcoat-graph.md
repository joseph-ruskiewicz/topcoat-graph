# topcoat-graph Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bar, line, area and pie charts for Topcoat apps: a pure-Rust geometry crate plus copy-in `topcoat ui` components, ending with the mana curve on contested-realm's deck pages.

**Architecture:** `crates/topcoat-graph` computes all geometry in percent units (0 to 100) and returns plain structs. Registry components under `crates/topcoat-graph/registry/components/` turn those structs into a stretched SVG plot with HTML labels, legend and tooltips over it. Hover is one `Signal<Option<usize>>` per chart, driven by the Topcoat runtime. `site/` is a gallery that installs every component and is their compile and test check.

**Tech Stack:** Rust 2024 (rust-version 1.88, stable toolchain), Topcoat 0.12.1 (`ui`, `tailwind`, `runtime` features), Tailwind through Topcoat's build config, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-11-26-topcoat-graph-design.md`. Read it before any task.

**Who does what** (Joseph's instruction): tasks marked **[Fable]** are done by the main Fable session. Tasks marked **[Opus]** are dispatched to subagents with `model: opus` and reviewed by Fable before the next task starts.

**Reference reading:** `../topcoat/demos/admin/src/components/chart.rs` (stretched SVG in `view!`), `../topcoat/docs/runtime.md`, `../topcoat/docs/runtime/expr.md`, `../topcoat/docs/ui.md`, and `../topcoat-prompt` for repo conventions (`Cargo.toml`, `site/`, `site/tests/registry_sync.rs`, `.github/workflows/ci.yml`, a component such as `crates/topcoat-prompt/src/components/branch_picker.rs`).

## Global Constraints

- Topcoat `0.12.1`; edition `2024`; `rust-version = "1.88"`; `rustfmt.toml` and `rust-toolchain.toml` copied from `../topcoat-prompt`. License MIT. Workspace version `0.1.0`. `repository = "https://github.com/joseph-ruskiewicz/topcoat-graph"`.
- The math crate has no dependencies, no `unsafe`, no Topcoat dependency, and knows nothing about HTML.
- Geometry is in percent units: the plot is a `0 0 100 100` box, x right, y down. Every number written into path data or returned for CSS is rounded to 2 decimal places with trailing zeros trimmed.
- Missing value = any non-finite `f64` (`NaN`, `inf`). No public function panics or emits `NaN`/`inf` for any input, including empty data.
- Charts are readable with no script: all marks, labels, legend and the hidden data table are server-rendered. No hand-written `.js` file. `raw!` only for a small piece, and report if more is needed.
- The hover signal's value is untrusted browser input. Only ever compare it for equality with a server-side index. Never index a collection with it.
- The runtime expression subset has no `&&`, `||` or `Option` equality. The hover test is `if h.is_some() { h.unwrap() == i } else { false }`. Don't bind camelCase SVG attributes.
- Components contain no hardcoded user-visible or announced English. All such text comes in as props.
- Component names: `chart`, `chart_axis`, `chart_legend`, `chart_tooltip`, `bar_chart`, `line_chart`, `area_chart`, `pie_chart`. Conventions follow built-ins: enum props for variants, `#[default] attrs: Attributes` on the root element, doc comments on every prop.
- Colors: `--chart-1` to `--chart-5` with built-in light and `dark:` fallbacks; other colors from neutral theme tokens (`text-muted-foreground`, `border`, `bg-popover`, `text-popover-foreground`).
- Not in scope: animation, zoom/pan, time scales, crates.io publishing, site deployment.
- Commit after every task with a conventional message ending in `Co-Authored-By: Claude Fable 5.2 <noreply@anthropic.com>`. Push only at the points the plan says.

## Review Focus

Each line names the task whose tests pin it.

1. **Empty or all-zero data** (a new deck with no cards): the chart renders its frame, axis `0` to `1`, no marks, no panic. Tasks 1, 4, 5.
2. **Series and categories of different lengths:** extra values are ignored, absent ones are missing. Nothing panics or shifts. Task 4.
3. **A single 100% pie slice, and a pie of all zeros:** the first draws a full ring or disc (an SVG arc can't span 360°), the second draws nothing. Task 3.
4. **Small integer counts** (a deck whose tallest bar is 1 or 2): ticks are whole numbers, never `0.5`. Tasks 1, 4.
5. **Many categories or long labels** (30 bars, or labels like "Unknown cost"): labels don't overlap into mush. Axis labels truncate within their band and a `label_every` prop thins them. Task 5.

---

### Task 1: Workspace, formatting, scales and ticks **[Fable]**

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `rustfmt.toml`, `rust-toolchain.toml`, `LICENSE`
- Create: `crates/topcoat-graph/Cargo.toml`, `crates/topcoat-graph/src/lib.rs`, `src/fmt.rs`, `src/scale.rs`, `src/ticks.rs`

**Interfaces (produces):**

```rust
// fmt.rs
pub fn num(value: f64) -> String;            // 2 decimals, trimmed: 12.0 -> "12", 0.125 -> "0.13", -0.0 -> "0"; non-finite -> "0"
// scale.rs
pub struct Linear;                            // Linear::new(domain: (f64, f64), range: (f64, f64))
impl Linear { pub fn map(&self, value: f64) -> f64; }   // zero-width domain maps to range midpoint
pub struct Band { pub start: f64, pub size: f64, pub center: f64 }
pub fn bands(count: usize, range: (f64, f64), padding: f64) -> Vec<Band>;  // padding = fraction of each step left empty, split evenly both sides
// ticks.rs
pub struct Ticks { pub values: Vec<f64>, pub step: f64 }
impl Ticks { pub fn domain(&self) -> (f64, f64); pub fn label(&self, value: f64) -> String; }
pub fn nice(min: f64, max: f64, target: usize, integer: bool) -> Ticks;
```

`nice` uses the d3 rule: `raw = span / target`, `power = 10^floor(log10(raw))`, `error = raw / power`; step is `10*power` if `error >= sqrt(50)`, `5*power` if `>= sqrt(10)`, `2*power` if `>= sqrt(2)`, else `power`. With `integer`, the step is at least 1. Values run from `floor(min/step)*step` to `ceil(max/step)*step`, each computed as `i * step` (not by repeated addition). Non-finite inputs, `min > max`, or `target == 0` are normalized (swap, treat non-finite as 0, target at least 1). If `min == max`: `0` gives domain `(0, 1)`; otherwise `(v - |v|, v + |v|)` before picking steps. `label` prints with the decimals `step` needs and no float noise.

- [ ] **Step 1:** Create the workspace files. Members: `crates/topcoat-graph` only for now. Crate `Cargo.toml` includes `[package.metadata.topcoat-ui] registry = "registry"`, description "Charts for Topcoat: line, bar, area and pie, installed with `topcoat ui add --registry topcoat-graph`.", and `[lints.rust] unsafe_code = "forbid"`.
- [ ] **Step 2:** Write failing tests in each module:

```rust
// fmt
assert_eq!(num(12.0), "12"); assert_eq!(num(0.125), "0.13"); assert_eq!(num(-0.0), "0"); assert_eq!(num(f64::NAN), "0");
// scale
let s = Linear::new((0.0, 10.0), (100.0, 0.0)); assert_eq!(s.map(2.5), 75.0);
assert_eq!(Linear::new((3.0, 3.0), (0.0, 100.0)).map(3.0), 50.0);
let b = bands(4, (0.0, 100.0), 0.2); assert_eq!(b.len(), 4);
assert!((b[0].start - 2.5).abs() < 1e-9); assert!((b[0].size - 20.0).abs() < 1e-9); assert!((b[3].center - 87.5).abs() < 1e-9);
assert!(bands(0, (0.0, 100.0), 0.2).is_empty());
// ticks
assert_eq!(nice(0.0, 12.0, 5, false).values, [0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0]);
assert_eq!(nice(0.0, 97.0, 5, false).domain(), (0.0, 100.0));
assert_eq!(nice(0.0, 1.0, 5, true).values, [0.0, 1.0]);                 // Review Focus 4
assert_eq!(nice(0.0, 2.0, 5, true).values, [0.0, 1.0, 2.0]);
assert_eq!(nice(0.0, 0.0, 5, false).domain(), (0.0, 1.0));              // Review Focus 1
let t = nice(0.0, 1.0, 5, false); assert_eq!(t.values.iter().map(|v| t.label(*v)).collect::<Vec<_>>(), ["0", "0.2", "0.4", "0.6", "0.8", "1"]);
assert_eq!(nice(-7.0, 7.0, 4, false).values, [-10.0, -5.0, 0.0, 5.0, 10.0]);
assert!(nice(f64::NAN, f64::INFINITY, 0, false).values.iter().all(|v| v.is_finite()));
```

- [ ] **Step 3:** Run `cargo test -p topcoat-graph`. Expected: compile failures for the missing items.
- [ ] **Step 4:** Implement the three modules.
- [ ] **Step 5:** Run `cargo test -p topcoat-graph` and `cargo clippy -p topcoat-graph --all-targets -- -D warnings`. Expected: all pass, no warnings.
- [ ] **Step 6:** Commit: `feat: add scales and nice ticks`.

### Task 2: Stacking and paths **[Fable]**

**Files:** Create `crates/topcoat-graph/src/stack.rs`, `src/path.rs`; modify `src/lib.rs`.

**Interfaces (produces):**

```rust
// stack.rs — one Vec per series, one entry per category; None where the value is missing
pub fn stack(series: &[&[f64]], category_count: usize) -> Vec<Vec<Option<(f64, f64)>>>;   // (lower, upper)
// path.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Curve { #[default] Linear, Smooth }
pub fn line(points: &[Option<(f64, f64)>], curve: Curve) -> String;
pub fn area(upper: &[Option<(f64, f64)>], lower: &[Option<(f64, f64)>], curve: Curve) -> String;
```

Stacking: positives accumulate up from 0, negatives down from 0, per category. A value past the end of a short series is missing. `line`: each run of consecutive `Some` points is its own `M...` subpath; a run of one point emits `Mx,yLx,y`, a zero-length segment, so a round-capped stroke shows a dot. `Smooth` is monotone cubic in x (Fritsch–Carlson tangents, as d3's `curveMonotoneX`) emitted as `C` segments; with two points it equals the straight line. `area`: for each run, upper forward then lower backward, closed with `Z`; `lower` has the same shape and missing pattern as `upper`.

- [ ] **Step 1:** Write failing tests:

```rust
let s = stack(&[&[1.0, -2.0, f64::NAN], &[3.0, -1.0]], 3);
assert_eq!(s[0], [Some((0.0, 1.0)), Some((-2.0, 0.0)), None]);
assert_eq!(s[1], [Some((1.0, 4.0)), Some((-3.0, -2.0)), None]);
assert_eq!(line(&[Some((0.0, 10.0)), Some((50.0, 0.0)), Some((100.0, 5.0))], Curve::Linear), "M0,10L50,0L100,5");
assert_eq!(line(&[Some((0.0, 1.0)), None, Some((2.0, 3.0)), Some((4.0, 5.0))], Curve::Linear), "M0,1L0,1M2,3L4,5");
assert_eq!(line(&[], Curve::Smooth), "");
assert_eq!(area(&[Some((0.0, 10.0)), Some((100.0, 20.0))], &[Some((0.0, 100.0)), Some((100.0, 100.0))], Curve::Linear), "M0,10L100,20L100,100L0,100Z");
```

Plus a property test for `Smooth`: for the points `(0,0) (10,0) (20,50) (30,50) (40,100)`, parse every number out of the path and assert each control point's y lies within `[0, 100]` and x values never decrease (no overshoot), and that a two-point smooth path's endpoints equal the inputs.

- [ ] **Step 2:** Run tests; expect failures. **Step 3:** Implement. **Step 4:** Run tests and clippy; expect pass.
- [ ] **Step 5:** Commit: `feat: add stacking and line and area paths`.

### Task 3: Pie geometry **[Fable]**

**Files:** Create `crates/topcoat-graph/src/pie.rs`; modify `src/lib.rs`.

**Interfaces (produces):**

```rust
#[derive(Clone, Copy, Debug)]
pub struct PieOptions { pub inner_radius: f64, pub pad_angle: f64, pub start_angle: f64 }  // inner_radius 0..1 of outer; angles in radians, 0 = 12 o'clock, clockwise
impl Default for PieOptions { /* 0.0, 0.0, 0.0 */ }
pub struct Slice { pub index: usize, pub path: String, pub centroid: (f64, f64), pub fraction: f64 }
pub fn pie_layout(values: &[f64], options: &PieOptions) -> Vec<Slice>;
```

Box is `0 0 100 100`, center `(50, 50)`, outer radius 50. Negative and non-finite values count as 0 and get no slice; `index` is the position in `values`. `inner_radius` is clamped to `0..=0.95`. Padding is skipped when only one slice is drawn. A slice covering the whole circle is emitted as two half arcs (and, for a donut, the inner circle as a second subpath with `fill-rule` left to the component: document that the path needs `fill-rule="evenodd"`).

- [ ] **Step 1:** Write failing tests:

```rust
let s = pie_layout(&[1.0, 1.0, 2.0], &PieOptions::default());
assert_eq!(s.len(), 3); assert_eq!(s[2].fraction, 0.5);
assert!(s[0].path.starts_with("M50,50L50,0A50,50 0 0 1 100,50"));       // first quarter, clockwise from top
assert!((s[2].centroid.0 - 25.0).abs() < 0.01 && (s[2].centroid.1 - 50.0).abs() < 0.01);
assert!(pie_layout(&[0.0, 0.0], &PieOptions::default()).is_empty());      // Review Focus 3
assert!(pie_layout(&[], &PieOptions::default()).is_empty());
let full = pie_layout(&[0.0, 7.0], &PieOptions::default());               // Review Focus 3
assert_eq!(full.len(), 1); assert_eq!(full[0].index, 1); assert_eq!(full[0].path.matches('A').count(), 2);
let donut = pie_layout(&[1.0, 3.0], &PieOptions { inner_radius: 0.6, ..Default::default() });
assert!(!donut[0].path.contains("L50,50"));                               // no line to center
assert!(pie_layout(&[-1.0, f64::NAN, 2.0], &PieOptions::default()).len() == 1);
```

Also assert no path in any test contains `NaN` or `inf`.

- [ ] **Step 2–4:** Fail, implement, pass, clippy.
- [ ] **Step 5:** Commit: `feat: add pie and donut slice geometry`.

### Task 4: Chart layouts **[Fable]**

**Files:** Create `crates/topcoat-graph/src/layout.rs`; modify `src/lib.rs` (re-export the public types at the crate root).

**Interfaces (produces):** these are what every cartesian component consumes.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub enum Orientation { #[default] Vertical, Horizontal }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)] pub enum BarMode { #[default] Grouped, Stacked }
pub struct Rect { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }
pub struct Tick { pub position: f64, pub label: String }   // percent along the value axis, measured in plot coordinates (vertical: from top; horizontal: from left)
pub struct Bar { pub series: usize, pub category: usize, pub value: f64, pub rect: Rect }

#[derive(Clone, Debug)]
pub struct BarOptions { pub orientation: Orientation, pub mode: BarMode, pub tick_target: usize, pub integer_ticks: bool, pub band_padding: f64, pub group_gap: f64 }
// Default: Vertical, Grouped, 5, false, 0.2, 0.05 (group_gap = fraction of the band between bars in a group)
pub struct BarLayout { pub bars: Vec<Bar>, pub ticks: Vec<Tick>, pub bands: Vec<Band>, pub zero: f64 }
pub fn bar_layout(category_count: usize, series: &[&[f64]], options: &BarOptions) -> BarLayout;

#[derive(Clone, Debug)]
pub struct XyOptions { pub curve: Curve, pub stacked: bool, pub include_zero: bool, pub tick_target: usize, pub integer_ticks: bool }
// Default: Linear, false, false, 5, false
pub struct SeriesShape { pub line: String, pub area: String, pub points: Vec<Option<(f64, f64)>> }
pub struct XyLayout { pub series: Vec<SeriesShape>, pub ticks: Vec<Tick>, pub bands: Vec<Band>, pub zero: f64 }
pub fn xy_layout(category_count: usize, series: &[&[f64]], options: &XyOptions) -> XyLayout;
```

Rules. `bands` always has `category_count` entries along the category axis. The value domain always includes 0 for bars and for `xy_layout` when `stacked` or `include_zero`; the domain is the `nice` domain. Bars run from the zero line to the value (negative bars go the other way); a missing or zero value gets no `Bar`. Grouped bars split the band evenly among all series (a missing value leaves its slot empty). In `xy_layout`, points sit at band centers with band padding 0; `area` runs down to the zero line, or to the series below when stacked; `line` for a stacked series follows its upper bound. Values past a short series are missing; values past `category_count` are ignored. `Horizontal` swaps axes: bands run top to bottom, values left to right.

- [ ] **Step 1:** Write failing tests:

```rust
// The mana curve: one series, integer ticks.
let counts = [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0];
let l = bar_layout(7, &[&counts], &BarOptions { integer_ticks: true, ..Default::default() });
assert_eq!(l.bars.len(), 6);                                   // the zero has no bar
assert_eq!(l.ticks.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(), ["0", "2", "4", "6", "8", "10", "12"]);
assert_eq!(l.zero, 100.0); assert_eq!(l.ticks[0].position, 100.0); assert_eq!(l.ticks[6].position, 0.0);
let tallest = &l.bars[2]; assert_eq!(tallest.rect.y, 0.0); assert_eq!(tallest.rect.height, 100.0);
assert!((tallest.rect.x - l.bands[2].start).abs() < 1e-9);
// Review Focus 1: empty and all-zero
let e = bar_layout(0, &[], &BarOptions::default()); assert!(e.bars.is_empty() && e.bands.is_empty()); assert_eq!(e.ticks.first().unwrap().label, "0");
let z = bar_layout(3, &[&[0.0, 0.0, 0.0]], &BarOptions::default()); assert!(z.bars.is_empty()); assert_eq!(z.bands.len(), 3);
// Review Focus 2: length mismatch
let m = bar_layout(3, &[&[1.0], &[1.0, 2.0, 3.0, 99.0]], &BarOptions::default());
assert_eq!(m.bars.iter().filter(|b| b.series == 0).count(), 1); assert_eq!(m.bars.iter().filter(|b| b.series == 1).count(), 3);
assert_eq!(m.ticks.last().unwrap().label, "3");               // the ignored 99 doesn't stretch the axis
// Review Focus 4
let small = bar_layout(2, &[&[1.0, 2.0]], &BarOptions { integer_ticks: true, ..Default::default() });
assert!(small.ticks.iter().all(|t| !t.label.contains('.')));
```

Plus: stacked bars of `[1,2]` and `[3,4]` give the second series' rects sitting exactly on the first's (shared edge) with domain to 6; grouped bars of two series don't overlap within a band; a negative value's rect starts at `zero` and extends down; `Horizontal` gives `rect.x == zero` and `width` proportional to value; `xy_layout(3, &[&[1.0, f64::NAN, 3.0]], ..)` has `points[1] == None` and two subpaths in `line`; stacked `xy_layout` area of series 1 has its lower edge equal to series 0's line points; `xy_layout(0, &[], ..)` returns empty series and a `0`..`1` axis; no output string anywhere contains `NaN`.

- [ ] **Step 2–4:** Fail, implement, pass, clippy.
- [ ] **Step 5:** Write the crate-level docs in `lib.rs` (what the crate is, the percent coordinate system, missing values, one `bar_layout` doctest). Run `cargo test -p topcoat-graph --doc`.
- [ ] **Step 6:** Commit and push: `feat: add bar and line/area chart layouts`.

### Task 5: Site, registry, and a static bar chart **[Fable]**

**Files:**
- Create: `crates/topcoat-graph/registry/registry.toml`, `registry/components/{chart,chart_axis,bar_chart}.rs`
- Create: `site/` (`Cargo.toml`, `build.rs`, `styles.css`, `components.toml`, `src/main.rs`, `src/components.rs`, `src/gallery/bar.rs`, `tests/registry_sync.rs`, `tests/render.rs`); modify root `Cargo.toml` (add `site`).

**Interfaces (produces):**

```rust
// chart.rs
pub struct ChartSeries<'a> { pub name: &'a str, pub values: &'a [f64], pub color: Option<&'a str> }
impl<'a> ChartSeries<'a> { pub fn new(name: &'a str, values: &'a [f64]) -> Self; pub fn color(self, css: &'a str) -> Self; }
pub fn series_color(index: usize, explicit: Option<&str>) -> String;   // explicit, else "var(--chart-N)" cycling N = index % 5 + 1
pub const CHART_VARS: &str;                                            // Tailwind classes setting --chart-1..5 fallbacks for light and dark: on the chart root
#[component] pub async fn chart(cx, label: String, attrs: Attributes, child: Child<'_>)             // root: relative container, role="figure", aria-label
#[component] pub async fn chart_table(cx, category_header: &str, categories: &[&str], series: &[ChartSeries<'_>])  // visually hidden <table>
// chart_axis.rs
#[component] pub async fn chart_value_axis(cx, ticks: &[Tick], orientation: Orientation)             // HTML tick labels + grid lines by percent
#[component] pub async fn chart_category_axis(cx, labels: &[&str], bands: &[Band], orientation: Orientation, label_every: usize)
// bar_chart.rs
#[component] pub async fn bar_chart(cx, label, category_header, categories: &[&str], series: &[ChartSeries<'_>],
    #[default] orientation: Orientation, #[default] mode: BarMode, #[default] integer_ticks: bool,
    #[default(1)] label_every: usize, #[default] attrs: Attributes)
```

Exact prop attributes and lifetimes follow what Topcoat's `#[component]` accepts; keep the names. The plot is `<svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">` inside a grid that reserves a left gutter for value labels and a bottom row for category labels; rects use `vector-effect="non-scaling-stroke"`. Category labels are centered on `band.center`, limited to the band's width with `truncate`, and only every `label_every`-th is shown. Default chart height comes from a class on the root (`aspect-video`), overridable through `attrs`.

- [ ] **Step 1:** Check the CLI: `topcoat --version` should print `0.12.1`. If it's missing, `cargo install topcoat-cli --version 0.12.1 --locked`.
- [ ] **Step 2:** Scaffold `site/` as a Topcoat app modeled on `../topcoat-prompt/site` (no icons, no session, no fly config): dependencies `topcoat = { version = "0.12.1", features = ["tailwind", "ui", "runtime"] }` plus whatever its features need, `topcoat-graph = { path = "../crates/topcoat-graph" }`, dev-dependency `topcoat-ui = "0.12.1"`. Run `topcoat ui init --theme neutral` in `site/`. A layout with `topcoat::runtime::script()` and the `RuntimeLayer`, and a `/bar` page.
- [ ] **Step 3:** Write `registry.toml` (`version = 1`; `chart`; `chart_axis` and `bar_chart` depending on `chart`, `bar_chart` also on `chart_axis`). Write minimal component files that compile, then `topcoat ui add --all --registry topcoat-graph` in `site/`.
- [ ] **Step 4:** Write `site/tests/registry_sync.rs` after `../topcoat-prompt/site/tests/registry_sync.rs` (registry `topcoat-graph`, path `../crates/topcoat-graph`, expected count 3 for now).
- [ ] **Step 5:** Write failing render tests in `site/tests/render.rs`. A helper `get(path) -> String` builds the site router and returns a page's HTML (in-process if the router is a tower service, else served on `127.0.0.1:0` and fetched). The `/bar` page's first example is the mana curve from Task 4's test, labelled "Mana cost distribution" with category header "Cost" and series "Cards":

```rust
let html = get("/bar");
assert!(html.contains(r#"aria-label="Mana cost distribution""#));
assert_eq!(html.matches("<rect").count() >= 6, true);
assert!(html.contains(r#"preserveAspectRatio="none""#));
assert!(html.contains("<th") && html.contains(">Cost<") && html.contains(">Cards<"));   // hidden table
for n in ["0", "2", "4", "6", "8", "10", "12"] { assert!(html.contains(&format!(">{n}<"))); }
assert!(!html.contains("NaN"));
```

Plus one test per Review Focus line owned here: an empty chart example on `/bar` (no categories) renders with its `aria-label` and no `<rect>` inside its figure (Focus 1); a 30-category example with `label_every: 5` renders 6 visible category labels in its figure (Focus 5).

- [ ] **Step 6:** Run `cargo test -p site`; expect failures. Implement the three components and the `/bar` gallery page with examples: single series, grouped, stacked, horizontal, negative values, empty, 30 categories. Re-run `topcoat ui add --all --registry topcoat-graph --overwrite` after each registry edit (edit the registry source, never the installed copy).
- [ ] **Step 7:** Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `rustfmt --check crates/topcoat-graph/registry/components/*.rs`, and `topcoat fmt --check` in `site/` and `crates/topcoat-graph/`. Then run the site and look at `/bar` in a browser at a wide and a phone width, in light and dark: text stays its real size, bars and labels line up, strokes aren't distorted.
- [ ] **Step 8:** Commit: `feat: add chart base, axes and a server-rendered bar chart`.

### Task 6: Hover tooltip and highlight on the bar chart **[Fable]**

**Files:** Create `registry/components/chart_tooltip.rs`; modify `registry.toml`, `chart.rs`, `bar_chart.rs`, `site/src/gallery/bar.rs`, `site/tests/render.rs`, `site/tests/registry_sync.rs` (count 4).

**Interfaces (produces):** this is the pattern Tasks 7 and 8 copy.

```rust
// chart.rs
pub fn hover_signal(cx: &Cx) -> Signal<Option<usize>>;       // one per chart instance; starts None
// chart_tooltip.rs
pub struct TooltipRow<'a> { pub name: &'a str, pub value: String, pub color: String }
#[component] pub async fn chart_tooltip(cx, hovered: &Signal<Option<usize>>, index: usize, title: &str, rows: &[TooltipRow<'_>],
    x: f64, y: f64)   // percent anchor in the plot; rendered hidden; shown when hovered == index; flips to the left of the anchor when x > 50
// chart.rs
#[component] pub async fn chart_hit_bands(cx, hovered: &Signal<Option<usize>>, bands: &[Band], orientation: Orientation, labels: &[&str])
```

`chart_hit_bands` renders one transparent, focusable HTML element per band over the full plot (`tabindex="0"`, `aria-label` = the category label), with `@mouseenter`/`@focus` setting `Some(i)` and a highlight background bound on when hovered. The chart root clears the signal on `@mouseleave`; each band clears on `@blur`. Tooltips are `role="tooltip"`, `pointer-events-none`, styled with `bg-popover text-popover-foreground border`, one per category, each listing every series' value for that category (formatted with `topcoat_graph::fmt::num`). Missing values are left out of the tooltip.

- [ ] **Step 1:** Spike first, in a scratch page: one hit band and one bound `:hidden` on an element over a stretched SVG, checked in a browser. If the runtime can't express it without more than a small `raw!`, stop and report to Joseph (the spec's "another Topcoat library" case).
- [ ] **Step 2:** Write failing render tests: the mana curve on `/bar` has 7 `role="tooltip"` elements, all hidden in the server HTML; 7 elements with `tabindex="0"`; the tooltip for category "2" contains "Cards" and "12".
- [ ] **Step 3:** Implement; re-install into the site; make the tests pass.
- [ ] **Step 4:** Browser check on `/bar`: hover shows the right tooltip and highlight and clears on leave; Tab moves through bands showing tooltips; tooltips near the right edge flip left and never overflow the chart; with JavaScript disabled the chart still shows bars, axes and labels. Check the horizontal and stacked examples too.
- [ ] **Step 5:** Full checks as in Task 5 Step 7. Commit and push: `feat: add hover tooltips and keyboard focus to charts`.

### Task 7: Legend, line chart and area chart **[Opus]**

**Files:** Create `registry/components/{chart_legend,line_chart,area_chart}.rs`, `site/src/gallery/{line,area}.rs`; modify `registry.toml`, `site/src/main.rs`, `site/tests/render.rs`, `site/tests/registry_sync.rs` (count 7), `bar_chart.rs` (add the legend).

**Interfaces:**
- Consumes: everything in Tasks 4 to 6. Follow `bar_chart.rs` exactly for structure, props, hit bands and tooltips.
- Produces:

```rust
#[component] pub async fn chart_legend(cx, items: &[(&str, String)])   // (name, css color); swatch + label, wraps
#[component] pub async fn line_chart(cx, label, category_header, categories, series, #[default] curve: Curve,
    #[default] dots: bool, #[default] integer_ticks: bool, #[default] include_zero: bool, #[default(1)] label_every: usize,
    #[default] legend: bool, #[default] attrs: Attributes)
#[component] pub async fn area_chart(cx, /* as line_chart, minus dots and include_zero */ #[default] stacked: bool, ...)
```

Lines: `fill="none"`, `stroke-width="2"`, round joins and caps, `vector-effect="non-scaling-stroke"`. Dots are zero-length round-capped strokes (`M x,y L x,y`) with a wider stroke width so they stay round when the plot stretches. Areas: the `area` path filled with the series color at reduced opacity plus the `line` path on top. `area_chart` always passes `include_zero: true`. `bar_chart` gains the same `legend` prop. `legend` is an explicit `bool`, default `false`.

- [ ] **Step 1:** Failing render tests for `/line` and `/area`: path `d` attributes present and free of `NaN`; a two-series example renders two stroked paths and, with `legend: true`, both names in the legend; a series with a missing middle value renders a path with two `M` commands; tooltips and hit bands counted as in Task 6; the hidden table lists every series; an empty example renders its frame.
- [ ] **Step 2:** Implement the components and gallery pages (examples: single, multi-series, smooth, dots, missing values, stacked area, empty). Re-install into the site.
- [ ] **Step 3:** Full checks as in Task 5 Step 7, including a browser look at both pages in light and dark at two widths.
- [ ] **Step 4:** Commit: `feat: add legend, line chart and area chart`.

### Task 8: Pie chart **[Opus]**

**Files:** Create `registry/components/pie_chart.rs`, `site/src/gallery/pie.rs`; modify `registry.toml`, `site/src/main.rs`, tests (count 8).

**Interfaces:**
- Consumes: `pie_layout`, `PieOptions`, `Slice` (Task 3); `chart`, `chart_table`, `hover_signal`, `series_color` (Tasks 5, 6); `chart_tooltip`, `chart_legend`.
- Produces:

```rust
pub struct PieSlice<'a> { pub name: &'a str, pub value: f64, pub color: Option<&'a str> }
#[component] pub async fn pie_chart(cx, label, category_header, value_header, slices: &[PieSlice<'_>],
    #[default] donut: bool, #[default] legend: bool, #[default] attrs: Attributes, #[default] child: Child<'_>)   // child = donut center content
```

A square SVG with a normal `viewBox="0 0 100 100"` (no stretching), `fill-rule="evenodd"` on slices. `donut` uses `inner_radius: 0.6`. Each slice path is its own hover and focus target (`tabindex="0"`, `aria-label` = name and value); hovering dims the other slices through a bound class. One tooltip per slice anchored at its centroid, showing name, value and percent. The center content is HTML centered over the SVG, only rendered when `donut`.

- [ ] **Step 1:** Failing render tests for `/pie`: three slices give three `<path`s in the figure; a single-slice example renders one path with two arcs; an all-zero example renders its figure, table and no slice paths (Review Focus 3); the donut example contains its center text; the table has name and value columns.
- [ ] **Step 2:** Implement, re-install, pass. **Step 3:** Full checks and a browser look.
- [ ] **Step 4:** Commit and push: `feat: add pie and donut chart`.

### Task 9: Gallery home, README and CI **[Opus]**

**Files:** Create `README.md`, `.github/workflows/ci.yml`, `.github/scripts/cargo-retry.sh`, `site/src/gallery/home.rs`; modify `site/src/main.rs`, each gallery page.

- [ ] **Step 1:** Gallery: a home page at `/` with install steps and one of each chart linking to its page; a shared nav; each example on the chart pages shows its source next to it (kept in sync by including the example's own source text, not a retyped copy); a light/dark toggle.
- [ ] **Step 2:** `README.md` modeled on `../topcoat-prompt/README.md`: what it is and that it's a community project; install (`cargo add topcoat --features ui,tailwind,runtime`, `cargo add --git https://github.com/joseph-ruskiewicz/topcoat-graph topcoat-graph`, `topcoat ui init --theme neutral`, `topcoat ui add bar_chart --registry topcoat-graph`); the runtime setup charts need for hover; a component table; a bar chart example; the `--chart-1` to `--chart-5` block to add to `styles.css` for `:root` and `.dark`; using the math crate alone.
- [ ] **Step 3:** CI copied from `../topcoat-prompt/.github/workflows/ci.yml` without the deploy job, with the registry rustfmt path `crates/topcoat-graph/registry/components/*.rs`.
- [ ] **Step 4:** Run every CI command locally. Commit and push. Confirm the Actions run on GitHub is green (`gh run watch`); fix and re-push until it is.

### Task 10: Whole-library review **[Fable]**

- [ ] **Step 1:** Read every registry component against the spec and Global Constraints: no hardcoded English, hover value only compared, consistent props across the four charts, doc comments on every prop.
- [ ] **Step 2:** Browser pass over every gallery example: hover, keyboard, light and dark, 360px and 1280px widths, JavaScript disabled.
- [ ] **Step 3:** Fix what's found (registry source first, then re-install). Full checks. Commit and push; CI green.

### Task 11: Mana curve in contested-realm **[Opus]**

Work in `/Users/joseph/Projects/contested-realm` on a new branch `mana-curve`. Read that repo's `CLAUDE.md`/`AGENTS.md`, `README.md` and `TODO.md` first and follow its conventions (it merges feature branches into main with merge commits; don't merge, leave that to Joseph).

**Files:** Modify `Cargo.toml` (add `topcoat-graph = { git = "https://github.com/joseph-ruskiewicz/topcoat-graph" }`), `components.toml` and `src/components/` (through `topcoat ui add bar_chart --registry topcoat-graph`), `src/app/decks/deck_id.rs`, `src/app/public/owner_id/deck_id.rs`, the matching `l10n/en.ftl` files; create the curve function next to the deck features code.

**Interfaces (produces):**

```rust
/// Counts spellbook cards by mana cost, every copy counted. Index 0..=5 are costs 0 to 5; index 6 is cost 6 or more.
/// Cards with no cost (sites, avatars) are skipped.
pub fn mana_curve(costs: impl IntoIterator<Item = (Option<u8>, u32)>) -> [u32; 7];   // (cost, copies)
```

- [ ] **Step 1:** Find how each deck page loads its spellbook cards and copy counts (cards have `cost: Option<u8>`, `src/features/cards.rs`), and what distinguishes spellbook cards from sites and the avatar.
- [ ] **Step 2:** Failing unit test: `mana_curve([(Some(0), 1), (Some(2), 3), (Some(2), 1), (Some(6), 2), (Some(9), 1), (None, 4)]) == [1, 0, 4, 0, 0, 0, 3]`; an empty deck gives all zeros. Implement.
- [ ] **Step 3:** Add a "Mana curve" card to both pages using `bar_chart` with categories `0`..`5`, `6+`, one series, `integer_ticks: true`, and all text from Fluent (`t!`): chart label, category header "Cost", series name "Cards". On the builder page, place it so it re-renders with the deck list when `version` changes.
- [ ] **Step 4:** Run that repo's full checks (its CI commands) and its Docker build if one runs locally, to confirm the git dependency resolves. Look at both pages in a browser: a real deck, an empty deck, and adding a card on the builder updates the curve.
- [ ] **Step 5:** Commit on the branch. Don't push or merge; report to Joseph with a screenshot.
