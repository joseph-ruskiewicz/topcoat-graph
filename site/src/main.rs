//! The topcoat-graph gallery: every chart component, with the data that drives it.

// Installed with `topcoat ui add --all --registry topcoat-graph`. Like any vendored
// library, not every item is used by this app.
#[allow(dead_code)]
mod components;

mod area;
mod bar;
mod line;
mod pie;
mod shell;

#[cfg(test)]
mod render_tests;

use topcoat::{
    Result,
    asset::{AssetBundle, AssetConfig, RouterBuilderAssetExt},
    context::Cx,
    router::{Router, RouterBuilderDiscoverExt, Slot, layout, module_router, page, request::uri},
    runtime::{RuntimeLayer, link, signal},
    tailwind,
    view::{Child, StaticClass, View, attributes, class, component, view},
};

use crate::components::{
    area_chart::area_chart,
    bar_chart::bar_chart,
    chart::{ChartSeries, labels},
    line_chart::line_chart,
    pie_chart::{PieSlice, pie_chart},
};

/// The gallery's pages, in the order the navigation lists them.
const PAGES: [(&str, &str); 4] = [
    ("/bar", "Bar"),
    ("/line", "Line"),
    ("/area", "Area"),
    ("/pie", "Pie"),
];

/// The README's install commands: its first `sh` code block. The home page shows them,
/// so the two always agree.
fn install_commands() -> &'static str {
    let readme = include_str!("../../README.md");
    let start = readme
        .find("```sh\n")
        .map_or(readme.len(), |at| at + "```sh\n".len());
    let end = readme[start..].find("```").map_or(start, |len| start + len);
    readme[start..end].trim_end()
}

#[tokio::main]
async fn main() {
    topcoat::start(router(AssetBundle::load().unwrap()))
        .await
        .unwrap();
}

/// The application's routes and services.
fn router(assets: impl Into<AssetConfig>) -> Router {
    module_router!()
        .assets(assets)
        .discover()
        .layer(RuntimeLayer::new())
        .build()
}

/// Classes for a link or button in the header.
const NAV_ITEM: StaticClass = class!(
    "rounded-md px-2 py-1 text-sm text-muted-foreground outline-none transition-colors",
    "hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring",
    "aria-[current=page]:text-foreground aria-[current=page]:font-medium",
    "aria-[pressed=true]:text-foreground",
);

/// The document every page sits in: a header with the navigation and the theme
/// toggle, then the page.
#[layout]
async fn layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    // Runtime links keep the layout's signals, so the theme survives navigation. A
    // full page load starts light.
    let dark = signal(cx, || false);
    let path = uri(cx).path().to_owned();

    Ok(view! {
        <!DOCTYPE html>
        <html
            lang="en"
            :class=$(if dark.get() { "dark" } else { "" })
            :style=$(if dark.get() {
                "color-scheme: dark"
            } else {
                "color-scheme: light"
            })
        >
            <head>
                <title>"topcoat-graph"</title>
                <meta name="viewport" content="width=device-width, initial-scale=1">
                topcoat::dev::script()
                topcoat::runtime::script()
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
            </head>
            <body>
                <header class="border-b">
                    <nav
                        class="mx-auto flex max-w-4xl flex-wrap items-center gap-x-2 gap-y-1 px-6 py-3"
                    >
                        link(
                            href: "/",
                            attrs: attributes! {
                                class="mr-auto text-base font-semibold outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-md"
                                aria-current=((path == "/").then_some("page"))
                            },
                            "topcoat-graph"
                        )
                        for (href, name) in PAGES {
                            link(
                                href: href,
                                attrs: attributes! {
                                    class=(NAV_ITEM)
                                    aria-current=((path == href).then_some("page"))
                                },
                                (name)
                            )
                        }
                        <button
                            type="button"
                            class=(NAV_ITEM)
                            :aria-pressed=$(if dark.get() { "true" } else { "false" })
                            @click=$(|_e| dark.toggle())
                        >
                            "Dark mode"
                        </button>
                    </nav>
                </header>
                <main class="mx-auto flex max-w-4xl flex-col gap-10 p-6">(slot)</main>
            </body>
        </html>
    })
}

/// The home page: what topcoat-graph is, how to install it, and one of each chart.
#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <div class="flex flex-col gap-3">
            <h1 class="text-2xl font-semibold">"topcoat-graph"</h1>
            <p class="text-muted-foreground">
                "Bar, line, area and pie charts for Topcoat apps. They render on the server "
                "as SVG with HTML labels, so they read fine with no script. With the Topcoat "
                "runtime on the page, pointing at a chart or tabbing to it shows a tooltip. "
                "The components are copied into your app with topcoat ui, as source you own."
            </p>
        </div>

        <section class="flex flex-col gap-3">
            <h2 class="text-base font-medium">"Install"</h2>
            <pre
                class="overflow-x-auto rounded-lg border bg-card px-4 py-3 text-xs leading-5"
            >
                <code>(install_commands())</code>
            </pre>
        </section>

        <div class="grid gap-6 sm:grid-cols-2">
            preview(
                href: "/bar",
                title: "Bar chart",
                bar_chart(
                    label: "Mana cost distribution",
                    category_header: "Cost",
                    categories: labels(bar::COSTS),
                    series: vec![ChartSeries::new("Cards", bar::MANA_CURVE)],
                    integer_ticks: true
                )
            )
            preview(
                href: "/line",
                title: "Line chart",
                line_chart(
                    label: "Games won and lost",
                    category_header: "Week",
                    categories: labels(
                        ["Week 1", "Week 2", "Week 3", "Week 4", "Week 5", "Week 6"],
                    ),
                    series: vec![
                        ChartSeries::new("Wins", [3.0, 5.0, 4.0, 7.0, 6.0, 8.0]),
                        ChartSeries::new("Losses", [4.0, 3.0, 4.0, 2.0, 3.0, 2.0]),
                    ],
                    integer_ticks: true,
                    include_zero: true
                )
            )
            preview(
                href: "/area",
                title: "Area chart",
                area_chart(
                    label: "Cards drawn per turn",
                    category_header: "Turn",
                    categories: labels(
                        ["Turn 1", "Turn 2", "Turn 3", "Turn 4", "Turn 5", "Turn 6"],
                    ),
                    series: vec![
                        ChartSeries::new("Cards drawn", [7.0, 1.0, 2.0, 1.0, 3.0, 2.0]),
                    ],
                    integer_ticks: true
                )
            )
            preview(
                href: "/pie",
                title: "Pie chart",
                pie_chart(
                    label: "Games by result",
                    category_header: "Result",
                    value_header: "Games",
                    slices: vec![
                        PieSlice::new("Wins", 18.0),
                        PieSlice::new("Losses", 12.0),
                        PieSlice::new("Draws", 2.0),
                    ],
                    donut: true,
                    attrs: attributes! { class="max-w-40!" },
                    <span class="text-lg font-semibold text-foreground tabular-nums">
                        "32"
                    </span>
                    <span>"games"</span>
                )
            )
        </div>
    })
}

/// A chart on the home page, in a card headed by a link to its page.
#[component]
async fn preview(
    cx: &Cx,
    /// The chart's gallery page.
    href: &'static str,
    /// The link's text.
    title: &str,
    /// The chart.
    #[default]
    child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <section class="flex flex-col gap-3 rounded-lg border bg-card p-4">
            link(
                href: href,
                attrs: attributes! {
                    class="self-start rounded-md text-sm font-medium underline-offset-4 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
                },
                (title)
            )
            (child)
        </section>
    })
}
