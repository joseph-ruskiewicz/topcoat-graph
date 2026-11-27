# topcoat-graph docs site: design

Approved by Joseph on 2026-11-26 ("yes, that's what i meant. go ahead").

## Intent

Turn `site/` from a plain gallery into the project's website: the place someone decides
whether to use topcoat-graph, learns to install it, and copies working code. It is
modeled on `../topcoat-prompt/site` (same author, same stack), which is the reference for
layout, page anatomy and how docs stay tied to code.

## Goals

- Same shell as topcoat-prompt: fixed left sidebar with grouped links and a theme toggle,
  a narrow content column, a sheet for the sidebar on narrow screens.
- Pages: Home, Introduction, Installation, Theming, Geometry crate, one page per chart
  (`bar_chart`, `line_chart`, `area_chart`, `pie_chart`) and per part (`chart`,
  `chart_axis`, `chart_legend`, `chart_tooltip`), Blocks, `/llms.txt`.
- Component page anatomy: title and description; (charts only) a playground; examples
  with Preview | Code tabs; Installation with CLI | Manual tabs; a props table per public
  component function; previous and next links.
- Interactive, in two ways beyond hover tooltips:
  1. A **playground** at the top of each chart page. Controls (per chart: curve, dots,
     stacked or grouped, orientation, legend, integer ticks, number of series, donut)
     redraw the chart on the server, and a code panel beside it shows the call that
     draws exactly what is on screen.
  2. One **interactive block**: tabs switch the series or the date range of a chart.
- Blocks: composed dashboard pieces (stat cards over a chart, the mana curve card, a
  multi-series line with legend, a donut with a total, a ranked horizontal bar list, the
  interactive block).
- One catalog drives the sidebar, pages, previous/next and `llms.txt`. Each example is a
  real `#[component]` in its own file under `site/src/examples/`, rendered as the preview
  and shown as code through `include_str!`. Manual-install source and props tables are
  read from the registry sources. Code panels are syntax-highlighted on the server.
- Ready to deploy like topcoat-prompt: `Dockerfile`, `.dockerignore`, `fly.toml` (app
  `topcoat-graph`).

## Non-goals

- Deploying. The files are written; nothing is launched until Joseph asks.
- Search, analytics, a blog, versioned docs.
- Editing chart data in the browser.
- New chart types or component API changes, unless a page exposes a real defect.

## Anti-goals

- Docs that can drift from code: no hand-copied snippets, no hand-written props tables.
- Losing test coverage: the existing 42 render tests are the components' regression
  suite. Every existing example and every assertion survives the move.
- Copying topcoat-prompt's prose. Structure and code patterns are reused; words are new.

## Approach

Port topcoat-prompt's site architecture (shell, catalog, docs slug page, API-table
extraction, highlighting, registry-sync test) rather than invent one. It is proven, the
author already likes it, and it keeps both sites familiar. The alternatives, extending
the current single-file-per-chart gallery or a static markdown site, either can't hold
install tabs and props tables without duplication or lose live components.

The playground is a `#[shard]` that reads one signal per control and re-renders when a
control sets its signal, the pattern topcoat-prompt's `*_run` examples use. The code
panel is generated in the same shard from the same option values.

## Left to the builder

Exact prose, example selection beyond the existing ones, block layouts, sidebar tags.
