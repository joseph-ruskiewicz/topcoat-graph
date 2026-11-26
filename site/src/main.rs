//! The topcoat-graph gallery: every chart component, with the data that drives it.

// Installed with `topcoat ui add --all --registry topcoat-graph`. Like any vendored
// library, not every item is used by this app.
#[allow(dead_code)]
mod components;

mod bar;
mod shell;

#[cfg(test)]
mod render_tests;

use topcoat::{
    Result,
    asset::{AssetBundle, AssetConfig, RouterBuilderAssetExt},
    context::Cx,
    router::{Router, RouterBuilderDiscoverExt, Slot, layout, module_router},
    runtime::RuntimeLayer,
    tailwind,
    view::{View, view},
};

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

/// The document every page sits in.
#[layout]
async fn layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <title>"topcoat-graph"</title>
                <meta name="viewport" content="width=device-width, initial-scale=1">
                topcoat::dev::script()
                topcoat::runtime::script()
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
            </head>
            <body>
                <main class="mx-auto flex max-w-4xl flex-col gap-10 p-6">(slot)</main>
            </body>
        </html>
    })
}
