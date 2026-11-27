//! The topcoat-graph docs site, built with Topcoat and the topcoat-graph components.

// The chart components were installed with `topcoat ui add --all --registry
// topcoat-graph`, and the built-in button, card, sheet and tooltip, which the site uses,
// with `topcoat ui add`. Like any vendored library, not every item is used by this app.
#[allow(dead_code)]
mod components;

mod api;
mod blocks;
mod catalog;
mod code;
mod docs;
mod examples;
mod guides;
mod playground;
mod shell;

#[cfg(test)]
mod render_tests;

use components::button::{ButtonSize, ButtonVariant, button_variants};
use topcoat::{
    Result,
    asset::{AssetBundle, AssetConfig, RouterBuilderAssetExt},
    context::Cx,
    cookie::CookieLayer,
    font::fontsource::fontsource_font,
    icon::{icon, iconify::iconify_icon},
    router::{
        Router, RouterBuilderDiscoverExt, Slot, layout, module_router, page, request::uri, route,
    },
    runtime::{RuntimeLayer, link, signal},
    tailwind,
    view::{View, attributes, view},
};

/// Where the GitHub links point.
pub const GITHUB_URL: &str = "https://github.com/joseph-ruskiewicz/topcoat-graph";

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
        .layer(CookieLayer::new())
        .build()
}

/// The document every page sits in, inside the docs shell with the sidebar.
#[layout]
async fn layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    // Runtime links keep signals the pages share, so the theme survives navigation. A
    // full load starts from the theme cookie, light when there is none.
    let saved = shell::dark_from_cookie(cx);
    let dark = signal(cx, || saved);
    let path = uri(cx).path().to_owned();
    // Each page is named in its tab; the home page is the site's name alone.
    let title = match catalog::pages().into_iter().find(|page| page.href == path) {
        Some(page) if page.href != "/" => format!("{} · topcoat-graph", page.title),
        _ => String::from("topcoat-graph"),
    };

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
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(title)</title>
                <meta
                    name="description"
                    content="Bar, line, area and pie charts for Topcoat apps, drawn on the server."
                >
                topcoat::dev::script()
                topcoat::runtime::script()
                topcoat::font::link(font: fontsource_font!(GEIST, host: Asset))
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                // Tabs switch with the runtime. Without script, show every panel.
                <noscript>
                    <style>"[role=tabpanel][hidden]{display:block!important}"</style>
                </noscript>
            </head>
            // The body's colors and font come from the theme's base layer.
            <body class="antialiased">
                shell::shell(path: &path, dark: dark, (slot))
            </body>
        </html>
    })
}

/// The home page: what topcoat-graph is, and a live chart with its code.
#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    let example = &catalog::HOME_EXAMPLE;

    Ok(view! {
        <div class="flex flex-col gap-10">
            <div class="flex flex-col gap-5">
                <h1 class="text-3xl leading-9 font-[450] tracking-tight text-balance">
                    "Charts for Topcoat."
                    <br>
                    <span class="text-muted-foreground">
                        "Bar, line, area and pie, drawn on the server, readable without script."
                    </span>
                </h1>
                <div class="flex flex-wrap items-center gap-3">
                    link(
                        href: "/docs/introduction",
                        attrs: attributes! {
                            class=(button_variants(
                                ButtonVariant::Primary,
                                ButtonSize::Md,
                            ))
                        },
                        "Get Started"
                    )
                    <a
                        href=(GITHUB_URL)
                        class=(button_variants(ButtonVariant::Outline, ButtonSize::Md))
                    >
                        icon(data: iconify_icon!("lucide:github"))
                        "GitHub"
                    </a>
                </div>
            </div>
            shell::preview_panel(((example.render)(cx)))
            shell::code_panel(source: example.source)
            shell::pager(path: "/")
        </div>
    })
}

/// A plain-text index of the docs for language models, generated from the catalog.
#[route(GET "./llms.txt")]
async fn llms_txt(cx: &Cx) -> Result<String> {
    Ok(catalog::llms_txt())
}
