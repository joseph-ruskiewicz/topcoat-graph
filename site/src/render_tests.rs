//! Renders the gallery's pages and checks the charts' markup.

use topcoat::{
    asset::{AssetCatalog, AssetConfig, MANIFEST_VERSION, Manifest, ManifestEntry, RawAsset},
    router::{Body, Router, request::Request, to_bytes},
};

/// Builds the site's router, as `main` does. Tests have no asset bundle, so the router
/// gets a catalog of the assets this binary references, as if they were hosted
/// elsewhere; pages render their URLs without the files.
fn site() -> Router {
    let binary = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    let assets = RawAsset::find_in_binary(&binary)
        .into_iter()
        .enumerate()
        .map(|(index, asset)| ManifestEntry {
            id: asset.id(),
            file: format!("asset-{index}"),
            hash: String::new(),
            content_type: "application/octet-stream".to_owned(),
        })
        .collect();
    let catalog = AssetCatalog::from(Manifest {
        version: MANIFEST_VERSION,
        assets,
    });
    crate::router(AssetConfig::hosted_at("https://assets.invalid", catalog))
}

/// The HTML of the page at `path`.
async fn get(path: &str) -> String {
    let request = Request::builder().uri(path).body(Body::empty()).unwrap();
    let response = site().handle(request).await;
    assert_eq!(response.status(), 200, "GET {path}");
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// The markup of the chart named `label`: from its figure's tag to the next figure's.
fn figure<'a>(html: &'a str, label: &str) -> &'a str {
    let marker = format!(r#"aria-label="{label}""#);
    let at = html
        .find(&marker)
        .unwrap_or_else(|| panic!("no chart labelled {label:?}"));
    let start = html[..at].rfind("<div").unwrap();
    let tag_end = at + html[at..].find('>').unwrap();
    assert!(
        html[start..tag_end].contains(r#"role="figure""#),
        "{label:?} is not a figure"
    );
    let end = match html[tag_end..].find(r#"role="figure""#) {
        Some(next) => html[..tag_end + next].rfind("<div").unwrap(),
        None => html.len(),
    };
    &html[start..end]
}

/// How many elements in `html` carry the marker attribute `data-chart="<part>"`.
fn parts(html: &str, part: &str) -> usize {
    html.matches(&format!(r#"data-chart="{part}""#)).count()
}

#[tokio::test]
async fn a_bar_chart_is_a_named_figure_with_a_stretched_plot() {
    let html = get("/bar").await;
    let chart = figure(&html, "Mana cost distribution");
    assert!(chart.contains(r#"role="figure""#));
    assert!(chart.contains(r#"viewBox="0 0 100 100""#));
    assert!(chart.contains(r#"preserveAspectRatio="none""#));
    assert!(chart.contains(r#"aria-hidden="true""#));
    assert!(!html.contains("NaN"));
}

#[tokio::test]
async fn a_bar_chart_draws_one_bar_per_nonzero_value() {
    let html = get("/bar").await;
    let chart = figure(&html, "Mana cost distribution");
    assert_eq!(chart.matches("<rect").count(), 6, "{chart}");
    // The tallest bar, 12 cards at cost 2, fills the plot's height.
    assert!(
        chart.contains(r#"y="0" width="11.43" height="100""#),
        "{chart}"
    );
    let grouped = figure(&html, "Revenue and costs by quarter");
    assert_eq!(grouped.matches("<rect").count(), 8, "{grouped}");
    assert!(grouped.contains("var(--chart-1,") && grouped.contains("var(--chart-2,"));
}

#[tokio::test]
async fn a_bar_chart_labels_its_axes() {
    let html = get("/bar").await;
    let chart = figure(&html, "Mana cost distribution");
    assert_eq!(parts(chart, "tick"), 7);
    for tick in ["0", "2", "4", "6", "8", "10", "12"] {
        assert!(chart.contains(&format!(">{tick}</span>")), "tick {tick}");
    }
    assert_eq!(parts(chart, "category"), 7);
    assert!(chart.contains(">6+</span>"));
    assert_eq!(parts(chart, "grid-line"), 7);
}

#[tokio::test]
async fn a_bar_chart_carries_its_data_as_a_hidden_table() {
    let html = get("/bar").await;
    let chart = figure(&html, "Mana cost distribution");
    assert!(chart.contains(r#"<table class="sr-only">"#));
    assert!(chart.contains(r#"<th scope="col">Cost</th>"#));
    assert!(chart.contains(r#"<th scope="col">Cards</th>"#));
    assert!(
        chart.contains(r#"<th scope="row">2</th><td>12</td>"#),
        "{chart}"
    );
    assert_eq!(chart.matches("<tr>").count(), 8);
}

#[tokio::test]
async fn an_empty_bar_chart_keeps_its_frame() {
    let html = get("/bar").await;
    let chart = figure(&html, "An empty deck");
    assert_eq!(chart.matches("<rect").count(), 0);
    assert!(chart.contains("<svg"));
    assert_eq!(parts(chart, "tick"), 2);
    assert_eq!(parts(chart, "category"), 0);
}

#[tokio::test]
async fn a_crowded_axis_shows_every_nth_label() {
    let html = get("/bar").await;
    let chart = figure(&html, "Games played per day");
    assert_eq!(chart.matches("<rect").count(), 30);
    assert_eq!(parts(chart, "category"), 6);
    assert!(chart.contains(">Day 1</span>") && chart.contains(">Day 6</span>"));
    assert!(!chart.contains(">Day 2</span>"));
    // Every category is still in the table.
    assert!(chart.contains(r#"<th scope="row">Day 2</th>"#));
}

#[tokio::test]
async fn a_horizontal_bar_chart_runs_its_categories_down_the_side() {
    let html = get("/bar").await;
    let chart = figure(&html, "Cards by element");
    assert_eq!(chart.matches("<rect").count(), 5);
    assert!(chart.contains(r#"x="0""#));
    assert!(chart.contains(">No element</span>"));
}

#[tokio::test]
async fn each_category_has_a_hidden_tooltip_with_its_values() {
    let html = get("/bar").await;
    let chart = figure(&html, "Mana cost distribution");
    assert_eq!(chart.matches(r#"role="tooltip""#).count(), 7);
    let tooltips: Vec<&str> = chart.split(r#"data-chart="tooltip""#).skip(1).collect();
    assert_eq!(tooltips.len(), 7);
    for tooltip in &tooltips {
        let tag = &tooltip[..tooltip.find('>').unwrap()];
        assert!(
            tag.contains("hidden"),
            "a tooltip is visible before any hover: {tag}"
        );
    }
    // The tooltip for cost 2 names the series and its value.
    let two = tooltips[2];
    let body = &two[..two.find("</div></div>").unwrap_or(two.len())];
    assert!(body.contains("Cards") && body.contains(">12<"), "{body}");
}

#[tokio::test]
async fn each_category_can_be_hovered_or_focused() {
    let html = get("/bar").await;
    let chart = figure(&html, "Mana cost distribution");
    assert_eq!(parts(chart, "hit-band"), 7);
    assert_eq!(chart.matches(r#"tabindex="0""#).count(), 7);
    assert_eq!(chart.matches("aria-describedby=").count(), 7);
    assert!(chart.contains(r#"aria-label="6+""#));
}

#[tokio::test]
async fn a_tooltip_lists_every_series_but_skips_missing_values() {
    let html = get("/bar").await;
    let chart = figure(&html, "Cards by cost and type");
    let tooltips: Vec<&str> = chart.split(r#"data-chart="tooltip""#).skip(1).collect();
    assert_eq!(
        tooltips[2].matches(r#"data-chart="tooltip-row""#).count(),
        3
    );
    let gaps = figure(&html, "Wins per week");
    let tooltips: Vec<&str> = gaps.split(r#"data-chart="tooltip""#).skip(1).collect();
    assert_eq!(
        tooltips[0].matches(r#"data-chart="tooltip-row""#).count(),
        1
    );
    assert_eq!(
        tooltips[1].matches(r#"data-chart="tooltip-row""#).count(),
        0
    );
}

#[tokio::test]
async fn an_empty_chart_has_nothing_to_hover() {
    let html = get("/bar").await;
    let chart = figure(&html, "An empty deck");
    assert_eq!(parts(chart, "hit-band"), 0);
    assert_eq!(parts(chart, "tooltip"), 0);
}

/// The `d` attribute of every element in `html` marked `data-chart="<part>"`.
fn paths<'a>(html: &'a str, part: &str) -> Vec<&'a str> {
    html.split(&format!(r#"data-chart="{part}""#))
        .skip(1)
        .map(|rest| {
            let tag = &rest[..rest.find('>').unwrap()];
            let d = &tag[tag
                .find(r#"d=""#)
                .unwrap_or_else(|| panic!("no d in {tag}"))
                + 3..];
            &d[..d.find('"').unwrap()]
        })
        .collect()
}

#[tokio::test]
async fn a_bar_chart_can_show_a_legend() {
    let html = get("/bar").await;
    let chart = figure(&html, "Revenue and costs by quarter");
    assert_eq!(parts(chart, "legend"), 1);
    assert_eq!(parts(chart, "legend-item"), 2);
    let plain = figure(&html, "Mana cost distribution");
    assert_eq!(parts(plain, "legend"), 0);
}

#[tokio::test]
async fn a_line_chart_draws_a_stroked_line_per_series() {
    let html = get("/line").await;
    assert!(!html.contains("NaN"));
    let chart = figure(&html, "Win rate by week");
    assert!(chart.contains(r#"preserveAspectRatio="none""#));
    let lines = paths(chart, "line");
    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].starts_with('M') && lines[0].contains('L'),
        "{}",
        lines[0]
    );
    assert!(chart.contains(r#"fill="none""#));
    assert!(chart.contains(r#"vector-effect="non-scaling-stroke""#));
    assert_eq!(parts(chart, "dot"), 0);
    assert_eq!(parts(chart, "legend"), 0);
}

#[tokio::test]
async fn a_multi_series_line_chart_names_its_series_in_a_legend() {
    let html = get("/line").await;
    let chart = figure(&html, "Games won and lost");
    assert_eq!(paths(chart, "line").len(), 2);
    assert_eq!(parts(chart, "legend"), 1);
    assert_eq!(parts(chart, "legend-item"), 2);
    let legend = &chart[chart.find(r#"data-chart="legend""#).unwrap()..];
    assert!(legend.contains(">Wins<") && legend.contains(">Losses<"));
    assert!(chart.contains("var(--chart-1,") && chart.contains("var(--chart-2,"));
}

#[tokio::test]
async fn a_smooth_line_uses_curves() {
    let html = get("/line").await;
    let chart = figure(&html, "Rating over a season");
    let lines = paths(chart, "line");
    assert!(lines[0].contains('C'), "{}", lines[0]);
}

#[tokio::test]
async fn a_missing_value_breaks_the_line() {
    let html = get("/line").await;
    let chart = figure(&html, "Wins per week, with a gap");
    let lines = paths(chart, "line");
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].matches('M').count(), 2, "{}", lines[0]);
    // The gap's tooltip has no rows; its neighbours have one.
    let tooltips: Vec<&str> = chart.split(r#"data-chart="tooltip""#).skip(1).collect();
    assert_eq!(
        tooltips[2].matches(r#"data-chart="tooltip-row""#).count(),
        0
    );
    assert_eq!(
        tooltips[1].matches(r#"data-chart="tooltip-row""#).count(),
        1
    );
}

#[tokio::test]
async fn a_line_chart_can_mark_each_value_with_a_dot() {
    let html = get("/line").await;
    let chart = figure(&html, "Daily players");
    // Six days, one without a value.
    let dots = paths(chart, "dot");
    assert_eq!(dots.len(), 5);
    for dot in dots {
        let (start, end) = dot.trim_start_matches('M').split_once('L').unwrap();
        assert_eq!(start, end, "a dot is a zero-length stroke: {dot}");
    }
    assert!(chart.contains(r#"stroke-linecap="round""#));
}

#[tokio::test]
async fn a_line_chart_has_a_tooltip_and_hit_band_per_category() {
    let html = get("/line").await;
    let chart = figure(&html, "Games won and lost");
    assert_eq!(parts(chart, "hit-band"), 6);
    assert_eq!(chart.matches(r#"tabindex="0""#).count(), 6);
    assert_eq!(parts(chart, "tooltip"), 6);
    let tooltips: Vec<&str> = chart.split(r#"data-chart="tooltip""#).skip(1).collect();
    for tooltip in &tooltips {
        let tag = &tooltip[..tooltip.find('>').unwrap()];
        assert!(tag.contains("hidden"), "visible before hover: {tag}");
        assert_eq!(tooltip.matches(r#"data-chart="tooltip-row""#).count(), 2);
    }
}

#[tokio::test]
async fn a_line_chart_carries_every_series_in_its_table() {
    let html = get("/line").await;
    let chart = figure(&html, "Games won and lost");
    assert!(chart.contains(r#"<table class="sr-only">"#));
    assert!(chart.contains(r#"<th scope="col">Week</th>"#));
    assert!(chart.contains(r#"<th scope="col">Wins</th>"#));
    assert!(chart.contains(r#"<th scope="col">Losses</th>"#));
    assert_eq!(chart.matches("<tr>").count(), 7);
}

#[tokio::test]
async fn an_empty_line_chart_keeps_its_frame() {
    let html = get("/line").await;
    let chart = figure(&html, "No games yet");
    assert!(chart.contains("<svg"));
    assert_eq!(chart.matches("<path").count(), 0);
    assert_eq!(parts(chart, "tick"), 2);
    assert_eq!(parts(chart, "hit-band"), 0);
    assert_eq!(parts(chart, "tooltip"), 0);
}

#[tokio::test]
async fn an_area_chart_fills_under_its_line() {
    let html = get("/area").await;
    assert!(!html.contains("NaN"));
    let chart = figure(&html, "Cards drawn per turn");
    assert!(chart.contains(r#"preserveAspectRatio="none""#));
    let areas = paths(chart, "area");
    let lines = paths(chart, "line");
    assert_eq!((areas.len(), lines.len()), (1, 1));
    assert!(
        areas[0].starts_with('M') && areas[0].ends_with('Z'),
        "{}",
        areas[0]
    );
    assert!(!lines[0].is_empty());
    // The axis starts at zero.
    assert!(chart.contains(">0</span>"));
}

#[tokio::test]
async fn a_multi_series_area_chart_names_its_series_in_a_legend() {
    let html = get("/area").await;
    let chart = figure(&html, "Mana spent and left over");
    assert_eq!(paths(chart, "area").len(), 2);
    assert_eq!(paths(chart, "line").len(), 2);
    let legend = &chart[chart.find(r#"data-chart="legend""#).unwrap()..];
    assert_eq!(parts(legend, "legend-item"), 2);
    assert!(legend.contains(">Spent<") && legend.contains(">Left over<"));
    assert_eq!(parts(chart, "hit-band"), 8);
    assert_eq!(parts(chart, "tooltip"), 8);
    assert!(chart.contains(r#"<th scope="col">Spent</th>"#));
    assert!(chart.contains(r#"<th scope="col">Left over</th>"#));
}

#[tokio::test]
async fn a_stacked_area_chart_draws_one_area_per_series() {
    let html = get("/area").await;
    let chart = figure(&html, "Cards in hand by type");
    assert_eq!(paths(chart, "area").len(), 3);
    assert_eq!(paths(chart, "line").len(), 3);
    let tooltips: Vec<&str> = chart.split(r#"data-chart="tooltip""#).skip(1).collect();
    assert_eq!(
        tooltips[0].matches(r#"data-chart="tooltip-row""#).count(),
        3
    );
}

#[tokio::test]
async fn an_area_with_a_missing_value_breaks_in_two() {
    let html = get("/area").await;
    let chart = figure(&html, "Cards drawn, with a gap");
    let lines = paths(chart, "line");
    assert_eq!(lines[0].matches('M').count(), 2, "{}", lines[0]);
    let areas = paths(chart, "area");
    assert_eq!(areas[0].matches('M').count(), 2, "{}", areas[0]);
}

#[tokio::test]
async fn an_empty_area_chart_keeps_its_frame() {
    let html = get("/area").await;
    let chart = figure(&html, "No turns yet");
    assert!(chart.contains("<svg"));
    assert_eq!(chart.matches("<path").count(), 0);
    assert_eq!(parts(chart, "tick"), 2);
    assert_eq!(parts(chart, "hit-band"), 0);
}
