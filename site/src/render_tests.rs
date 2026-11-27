//! Renders the site's pages and checks the charts' markup and the docs around them.

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

/// The markup of the chart named `label`: its figure, from its opening tag to its
/// closing one.
///
/// Docs pages also show example and component source, so the label can appear in code
/// too. Highlighted code escapes its quotes, so `aria-label="..."` only matches real
/// markup, and the figure ends at its own closing tag rather than running on into the
/// code panels after it.
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
    &html[start..element_end(html, start)]
}

/// The end of the `<div>` element that opens at `start`: just past its `</div>`.
fn element_end(html: &str, start: usize) -> usize {
    let mut depth = 0;
    let mut at = start;
    loop {
        let open = html[at..].find("<div").map(|i| at + i);
        let close = html[at..]
            .find("</div>")
            .map(|i| at + i)
            .expect("the element closes");
        match open {
            Some(open) if open < close => {
                depth += 1;
                at = open + "<div".len();
            }
            _ => {
                depth -= 1;
                at = close + "</div>".len();
                if depth == 0 {
                    return at;
                }
            }
        }
    }
}

#[test]
fn a_figure_ends_at_its_own_closing_tag() {
    let html = r#"<div role="figure" aria-label="A"><div>in</div></div><pre>after</pre>"#;
    assert_eq!(
        figure(html, "A"),
        r#"<div role="figure" aria-label="A"><div>in</div></div>"#
    );
}

/// The rest of an opening tag, from somewhere inside it up to its closing `>`.
///
/// Attribute values are not searched for the end: the runtime's bind attributes hold
/// arrow functions, and the `>` of a `=>` does not close the tag.
fn tag_rest(html: &str) -> &str {
    let bytes = html.as_bytes();
    let end = (0..bytes.len())
        .find(|&i| bytes[i] == b'>' && (i == 0 || bytes[i - 1] != b'='))
        .expect("the tag closes");
    &html[..end]
}

/// Whether the tag that `rest` is inside of is rendered with the `hidden` attribute.
fn renders_hidden(rest: &str) -> bool {
    tag_rest(rest).contains(r#" hidden="""#)
}

#[test]
fn a_bound_hidden_attribute_alone_does_not_count_as_hidden() {
    let bound = r#" data-topcoat-bind:hidden="(() => x)" class="a">text"#;
    assert_eq!(
        tag_rest(bound),
        r#" data-topcoat-bind:hidden="(() => x)" class="a""#
    );
    assert!(!renders_hidden(bound));
    assert!(renders_hidden(
        r#" data-topcoat-bind:hidden="(() => x)" hidden="">text"#
    ));
}

/// How many elements in `html` carry the marker attribute `data-chart="<part>"`.
fn parts(html: &str, part: &str) -> usize {
    html.matches(&format!(r#"data-chart="{part}""#)).count()
}

#[tokio::test]
async fn a_bar_chart_is_a_named_figure_with_a_stretched_plot() {
    let html = get("/docs/bar_chart").await;
    let chart = figure(&html, "Mana cost distribution");
    assert!(chart.contains(r#"role="figure""#));
    assert!(chart.contains(r#"viewBox="0 0 100 100""#));
    assert!(chart.contains(r#"preserveAspectRatio="none""#));
    let svg = &chart[chart.find("<svg").unwrap()..];
    assert!(
        tag_rest(svg).contains(r#"aria-hidden="true""#),
        "the drawing is decorative"
    );
    assert!(!html.contains("NaN"));
}

#[tokio::test]
async fn a_bar_chart_draws_one_bar_per_nonzero_value() {
    let html = get("/docs/bar_chart").await;
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
    let html = get("/docs/bar_chart").await;
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
    let html = get("/docs/bar_chart").await;
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
    let html = get("/docs/bar_chart").await;
    let chart = figure(&html, "An empty deck");
    assert_eq!(chart.matches("<rect").count(), 0);
    assert!(chart.contains("<svg"));
    assert_eq!(parts(chart, "tick"), 2);
    assert_eq!(parts(chart, "category"), 0);
}

#[tokio::test]
async fn a_crowded_axis_shows_every_nth_label() {
    let html = get("/docs/bar_chart").await;
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
    let html = get("/docs/bar_chart").await;
    let chart = figure(&html, "Cards by element");
    assert_eq!(chart.matches("<rect").count(), 5);
    assert!(chart.contains(r#"x="0""#));
    assert!(chart.contains(">No element</span>"));
}

#[tokio::test]
async fn each_category_has_a_hidden_tooltip_with_its_values() {
    let html = get("/docs/bar_chart").await;
    let chart = figure(&html, "Mana cost distribution");
    assert_eq!(chart.matches(r#"role="tooltip""#).count(), 7);
    let tooltips: Vec<&str> = chart.split(r#"data-chart="tooltip""#).skip(1).collect();
    assert_eq!(tooltips.len(), 7);
    for tooltip in &tooltips {
        assert!(
            renders_hidden(tooltip),
            "a tooltip is visible before any hover"
        );
    }
    // The tooltip for cost 2 names the series and its value.
    let two = tooltips[2];
    let body = &two[..two.find("</div></div>").unwrap_or(two.len())];
    assert!(body.contains("Cards") && body.contains(">12<"), "{body}");
}

#[tokio::test]
async fn each_category_can_be_hovered_or_focused() {
    let html = get("/docs/bar_chart").await;
    let chart = figure(&html, "Mana cost distribution");
    assert_eq!(parts(chart, "hit-band"), 7);
    assert_eq!(chart.matches(r#"tabindex="0""#).count(), 7);
    assert_eq!(chart.matches("aria-describedby=").count(), 7);
    assert!(chart.contains(r#"aria-label="6+""#));
}

#[tokio::test]
async fn a_tooltip_lists_every_series_but_skips_missing_values() {
    let html = get("/docs/bar_chart").await;
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
    let html = get("/docs/bar_chart").await;
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
    let html = get("/docs/bar_chart").await;
    let chart = figure(&html, "Revenue and costs by quarter");
    assert_eq!(parts(chart, "legend"), 1);
    assert_eq!(parts(chart, "legend-item"), 2);
    let plain = figure(&html, "Mana cost distribution");
    assert_eq!(parts(plain, "legend"), 0);
}

#[tokio::test]
async fn a_line_chart_draws_a_stroked_line_per_series() {
    let html = get("/docs/line_chart").await;
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
    let html = get("/docs/line_chart").await;
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
    let html = get("/docs/line_chart").await;
    let chart = figure(&html, "Rating over a season");
    let lines = paths(chart, "line");
    assert!(lines[0].contains('C'), "{}", lines[0]);
}

#[tokio::test]
async fn a_missing_value_breaks_the_line() {
    let html = get("/docs/line_chart").await;
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
    let html = get("/docs/line_chart").await;
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
    let html = get("/docs/line_chart").await;
    let chart = figure(&html, "Games won and lost");
    assert_eq!(parts(chart, "hit-band"), 6);
    assert_eq!(chart.matches(r#"tabindex="0""#).count(), 6);
    assert_eq!(parts(chart, "tooltip"), 6);
    let tooltips: Vec<&str> = chart.split(r#"data-chart="tooltip""#).skip(1).collect();
    for tooltip in &tooltips {
        assert!(renders_hidden(tooltip), "visible before hover");
        assert_eq!(tooltip.matches(r#"data-chart="tooltip-row""#).count(), 2);
    }
}

#[tokio::test]
async fn a_line_chart_carries_every_series_in_its_table() {
    let html = get("/docs/line_chart").await;
    let chart = figure(&html, "Games won and lost");
    assert!(chart.contains(r#"<table class="sr-only">"#));
    assert!(chart.contains(r#"<th scope="col">Week</th>"#));
    assert!(chart.contains(r#"<th scope="col">Wins</th>"#));
    assert!(chart.contains(r#"<th scope="col">Losses</th>"#));
    assert_eq!(chart.matches("<tr>").count(), 7);
}

#[tokio::test]
async fn an_empty_line_chart_keeps_its_frame() {
    let html = get("/docs/line_chart").await;
    let chart = figure(&html, "No games yet");
    assert!(chart.contains("<svg"));
    assert_eq!(chart.matches("<path").count(), 0);
    assert_eq!(parts(chart, "tick"), 2);
    assert_eq!(parts(chart, "hit-band"), 0);
    assert_eq!(parts(chart, "tooltip"), 0);
}

#[tokio::test]
async fn an_area_chart_fills_under_its_line() {
    let html = get("/docs/area_chart").await;
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
    let html = get("/docs/area_chart").await;
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
    let html = get("/docs/area_chart").await;
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
    let html = get("/docs/area_chart").await;
    let chart = figure(&html, "Cards drawn, with a gap");
    let lines = paths(chart, "line");
    assert_eq!(lines[0].matches('M').count(), 2, "{}", lines[0]);
    let areas = paths(chart, "area");
    assert_eq!(areas[0].matches('M').count(), 2, "{}", areas[0]);
}

#[tokio::test]
async fn an_empty_area_chart_keeps_its_frame() {
    let html = get("/docs/area_chart").await;
    let chart = figure(&html, "No turns yet");
    assert!(chart.contains("<svg"));
    assert_eq!(chart.matches("<path").count(), 0);
    assert_eq!(parts(chart, "tick"), 2);
    assert_eq!(parts(chart, "hit-band"), 0);
}

/// The tooltips in `html`, each from its marker to the end of the markup.
fn tooltips(html: &str) -> Vec<&str> {
    html.split(r#"data-chart="tooltip""#).skip(1).collect()
}

#[tokio::test]
async fn a_pie_draws_one_slice_per_value_in_a_square() {
    let html = get("/docs/pie_chart").await;
    assert!(!html.contains("NaN"));
    let chart = figure(&html, "Cards by type");
    assert!(chart.contains(r#"viewBox="0 0 100 100""#));
    assert!(!chart.contains("preserveAspectRatio"));
    // The slices are focusable, so the drawing they are in is not hidden.
    let svg = &chart[chart.find("<svg").unwrap()..];
    assert!(!svg[..svg.find('>').unwrap()].contains("aria-hidden"));
    let slices = paths(chart, "slice");
    assert_eq!(slices.len(), 3);
    assert!(chart.contains(r#"fill-rule="evenodd""#));
    for color in ["var(--chart-1,", "var(--chart-2,", "var(--chart-3,"] {
        assert!(chart.contains(color), "{color}");
    }
    assert_eq!(parts(chart, "legend"), 0);
}

#[tokio::test]
async fn a_single_slice_fills_the_whole_circle() {
    let html = get("/docs/pie_chart").await;
    let chart = figure(&html, "Cards by rarity");
    let slices = paths(chart, "slice");
    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].matches('A').count(), 2, "{}", slices[0]);
}

#[tokio::test]
async fn an_all_zero_pie_draws_no_slices_but_keeps_its_table() {
    let html = get("/docs/pie_chart").await;
    let chart = figure(&html, "No games played");
    assert!(chart.contains("<svg"));
    assert_eq!(parts(chart, "slice"), 0);
    assert_eq!(parts(chart, "tooltip"), 0);
    assert_eq!(chart.matches(r#"tabindex="0""#).count(), 0);
    assert!(chart.contains(r#"<table class="sr-only">"#));
    assert!(
        chart.contains(r#"<th scope="row">Wins</th><td>0</td>"#),
        "{chart}"
    );
}

#[tokio::test]
async fn a_donut_shows_its_center_content() {
    let html = get("/docs/pie_chart").await;
    let chart = figure(&html, "Games by result");
    assert_eq!(paths(chart, "slice").len(), 3);
    assert_eq!(parts(chart, "center"), 1);
    assert!(chart.contains("32"), "{chart}");
    assert!(chart.contains("games"), "{chart}");
    // A plain pie has no center, even though it is given none.
    assert_eq!(parts(figure(&html, "Cards by type"), "center"), 0);
}

#[tokio::test]
async fn a_pie_carries_its_data_as_a_hidden_table() {
    let html = get("/docs/pie_chart").await;
    let chart = figure(&html, "Cards by type");
    assert!(chart.contains(r#"<table class="sr-only">"#));
    assert!(chart.contains(r#"<th scope="col">Type</th>"#));
    assert!(chart.contains(r#"<th scope="col">Cards</th>"#));
    assert!(
        chart.contains(r#"<th scope="row">Minions</th><td>18</td>"#),
        "{chart}"
    );
    assert_eq!(chart.matches("<tr>").count(), 4);
}

#[tokio::test]
async fn each_drawn_slice_has_a_hidden_tooltip_with_its_share() {
    let html = get("/docs/pie_chart").await;
    let chart = figure(&html, "Cards by type");
    let tips = tooltips(chart);
    assert_eq!(tips.len(), 3);
    for tip in &tips {
        assert!(renders_hidden(tip), "visible before hover");
    }
    // Minions: 18 of 32 cards.
    assert!(tips[0].contains("Minions"), "{}", tips[0]);
    assert!(
        tips[0].contains("18") && tips[0].contains("56.25%"),
        "{}",
        tips[0]
    );
}

#[tokio::test]
async fn each_slice_can_be_hovered_or_focused() {
    let html = get("/docs/pie_chart").await;
    let chart = figure(&html, "Cards by type");
    assert_eq!(chart.matches(r#"tabindex="0""#).count(), 3);
    assert_eq!(chart.matches("aria-describedby=").count(), 3);
    assert!(chart.contains(r#"aria-label="Artifacts""#));
}

#[tokio::test]
async fn a_zero_slice_is_skipped_but_listed_and_keeps_the_colors_in_place() {
    let html = get("/docs/pie_chart").await;
    let chart = figure(&html, "Cards by element");
    assert_eq!(paths(chart, "slice").len(), 3);
    assert_eq!(tooltips(chart).len(), 3);
    // Fire comes after the empty Earth slice and keeps the fourth color.
    let at = chart.find(r#"aria-label="Fire""#).unwrap();
    let start = chart[..at].rfind("<path").unwrap();
    let fire = &chart[start..at + chart[at..].find('>').unwrap()];
    assert!(fire.contains("var(--chart-4,"), "{fire}");
    // The legend and the table still list every slice.
    assert_eq!(parts(chart, "legend"), 1);
    assert_eq!(parts(chart, "legend-item"), 4);
    assert!(
        chart.contains(r#"<th scope="row">Earth</th><td>0</td>"#),
        "{chart}"
    );
}

#[tokio::test]
async fn a_tooltip_never_opens_over_its_own_category() {
    let html = get("/docs/bar_chart").await;
    let chart = figure(&html, "Mana cost distribution");
    let tooltips: Vec<&str> = chart.split(r#"data-chart="tooltip""#).skip(1).collect();
    // Cost 3 is the middle band, spanning 44.29 to 55.71. Whichever edge the tooltip
    // is anchored to, it has to open away from the band.
    let middle = tag_rest(tooltips[3]);
    let anchored_right = middle.contains("left:55.71%");
    let opens_left = chart[..chart.find(tooltips[3]).unwrap()]
        .rsplit("<div")
        .next()
        .unwrap()
        .contains("-translate-x-full")
        || middle.contains("-translate-x-full");
    assert!(anchored_right != opens_left, "{middle}");
    // The first band opens right from its right edge and the last opens left from its left.
    assert!(tag_rest(tooltips[0]).contains("left:12.86%"));
    assert!(tag_rest(tooltips[6]).contains("left:87.14%"));
}

#[tokio::test]
async fn small_values_are_not_rounded_away_in_tooltips_or_the_table() {
    let html = get("/docs/line_chart").await;
    let chart = figure(&html, "Error rate by release");
    assert!(chart.contains("<td>0.0012</td>"), "{chart}");
    assert!(chart.contains(">0.0012</span>"));
    assert!(!chart.contains("<td>0</td>"));
}

#[tokio::test]
async fn a_pie_tooltip_shows_small_shares() {
    let html = get("/docs/pie_chart").await;
    let chart = figure(&html, "Games by platform");
    assert!(chart.contains("3 (0.03%)"), "{chart}");
    assert!(!chart.contains("(0%)"));
}

// The docs around the charts.

use crate::{api, catalog, guides};

/// The status of a GET of `path`.
async fn status(path: &str) -> u16 {
    let request = Request::builder().uri(path).body(Body::empty()).unwrap();
    site().handle(request).await.status().as_u16()
}

/// Text from HTML: tags dropped and the escapes the site writes undone.
fn text(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        out.push_str(&rest[..open]);
        rest = &rest[open + rest[open..].find('>').unwrap() + 1..];
    }
    out.push_str(rest);
    out.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// The text of each `<pre><code>` block in `html`, in order.
fn code_blocks(html: &str) -> Vec<String> {
    html.split("<pre")
        .skip(1)
        .map(|rest| {
            let start = rest.find("<code").unwrap();
            let start = start + rest[start..].find('>').unwrap() + 1;
            text(&rest[start..start + rest[start..].find("</code>").unwrap()])
        })
        .collect()
}

/// The markup of the element whose opening tag contains `marker`, through its closing
/// `</div>`.
fn div_with<'a>(html: &'a str, marker: &str) -> &'a str {
    let at = html.find(marker).unwrap_or_else(|| panic!("no {marker}"));
    let start = html[..at].rfind("<div").unwrap();
    &html[start..element_end(html, start)]
}

/// The `<section>` of the example `id`.
fn example_section<'a>(html: &'a str, id: &str) -> &'a str {
    let at = html
        .find(&format!(r#"data-example="{id}""#))
        .unwrap_or_else(|| panic!("no example {id}"));
    let start = html[..at].rfind("<section").unwrap();
    &html[start..start + html[start..].find("</section>").unwrap()]
}

#[tokio::test]
async fn every_component_has_a_page_with_its_examples() {
    for doc in catalog::COMPONENTS {
        let path = format!("/docs/{}", doc.name);
        let html = get(&path).await;
        assert!(!html.contains("NaN"), "{path}");
        assert!(html.contains(&format!(">{}</h1>", doc.title)), "{path}");
        assert!(!doc.examples.is_empty(), "{path}");
        assert_eq!(
            html.matches("data-example=").count(),
            doc.examples.len(),
            "{path}"
        );
        for example in doc.examples {
            let section = example_section(&html, example.id);
            assert!(section.contains(r#"role="tablist""#), "{}", example.id);
            assert!(section.contains(">Preview</button>") && section.contains(">Code</button>"));
            let preview = div_with(section, r#"data-tab="first""#);
            assert!(
                !renders_hidden(&preview[4..]),
                "{}: preview hidden",
                example.id
            );
            assert!(preview.contains(r#"data-slot="preview""#));
            let code = div_with(section, r#"data-tab="second""#);
            assert!(
                renders_hidden(&code[4..]),
                "{}: code shown at first",
                example.id
            );
            // The code shown is the example's whole file.
            assert_eq!(code_blocks(code), [example.source], "{}", example.id);
            assert!(
                code.contains("tk-keyword"),
                "{}: not highlighted",
                example.id
            );
            assert!(code.contains(r#"aria-label="Copy code""#), "{}", example.id);
        }
    }
}

#[tokio::test]
async fn every_component_page_shows_how_to_install_it() {
    for doc in catalog::COMPONENTS {
        let path = format!("/docs/{}", doc.name);
        let html = get(&path).await;
        let install = &html[html.find(r#"id="installation""#).unwrap()..];
        let manual = div_with(install, r#"data-tab="second""#);
        let install = &install[..install.find(r#"id="api""#).unwrap()];
        assert!(
            install.contains(&format!(
                "topcoat ui add {} --registry topcoat-graph",
                doc.name
            )),
            "{path}: no CLI command"
        );
        // Manual: the registry source, as it is, and the parts it needs.
        assert!(
            code_blocks(manual).iter().any(|code| code == doc.source),
            "{path}: the manual tab lacks the registry source"
        );
        let parts = catalog::dependencies(doc.name);
        for part in &parts {
            assert!(
                manual.contains(&format!(r#"href="/docs/{part}""#)),
                "{path}: no link to {part}"
            );
        }
        assert_eq!(manual.contains(r#"data-slot="parts""#), !parts.is_empty());
    }
}

#[tokio::test]
async fn every_component_page_has_a_props_table_per_function() {
    for doc in catalog::COMPONENTS {
        let path = format!("/docs/{}", doc.name);
        let html = get(&path).await;
        let functions = api::functions(doc.source);
        assert!(!functions.is_empty(), "{path}");
        assert_eq!(
            html.matches(r#"data-slot="props""#).count(),
            functions.len(),
            "{path}"
        );
        for function in &functions {
            let at = html
                .find(&format!(">{}</h3>", function.name))
                .unwrap_or_else(|| panic!("{path}: no table for {}", function.name));
            let table = &html[at..];
            let table = &table[..table.find("</table>").unwrap()];
            assert_eq!(
                table.matches("<tr>").count(),
                function.props.len(),
                "{path}: {}",
                function.name
            );
            for prop in &function.props {
                assert!(
                    table.contains(&format!(">{}</td>", prop.name)),
                    "{path}: {} lacks {}",
                    function.name,
                    prop.name
                );
            }
        }
    }
}

#[tokio::test]
async fn the_chart_part_documents_its_types_and_helpers() {
    let html = get("/docs/chart").await;
    for item in [
        "ChartSeries",
        "labels",
        "series_color",
        "percent",
        "CHART_VARS",
    ] {
        assert!(html.contains(&format!(r#"id="item-{item}""#)), "{item}");
    }
    let series = div_with(&html, r#"id="item-ChartSeries""#);
    for field in ["name", "values", "color"] {
        assert!(series.contains(&format!(">{field}</td>")), "{field}");
    }
    assert!(
        text(series)
            .contains("pub fn new(name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self")
    );
    let pie = get("/docs/pie_chart").await;
    assert!(pie.contains(r#"id="item-PieSlice""#));
    assert!(
        !pie.contains(r#"id="item-DrawnSlice""#),
        "private types stay out"
    );
}

#[tokio::test]
async fn previous_and_next_follow_the_catalog() {
    let pages = catalog::pages();
    let hrefs: Vec<&str> = pages.iter().map(|page| page.href.as_str()).collect();
    assert_eq!(hrefs.first(), Some(&"/"));
    assert_eq!(hrefs.len(), 14);
    for (index, page) in pages.iter().enumerate() {
        let html = get(&page.href).await;
        let pager = &html[html.find(r#"aria-label="Pages""#).unwrap()..];
        let pager = &pager[..pager.find("</nav>").unwrap()];
        let linked = |rel: &str| {
            let at = pager.find(&format!(r#"rel="{rel}""#))?;
            let tag = &pager[pager[..at].rfind("<a").unwrap()..];
            let href = &tag[tag.find(r#"href=""#).unwrap() + 6..];
            Some(href[..href.find('"').unwrap()].to_owned())
        };
        let previous = index.checked_sub(1).map(|index| pages[index].href.clone());
        let next = pages.get(index + 1).map(|page| page.href.clone());
        assert_eq!(linked("prev"), previous, "{}", page.href);
        assert_eq!(linked("next"), next, "{}", page.href);
    }
}

#[tokio::test]
async fn the_sidebar_lists_every_page_and_marks_the_current_one() {
    let pages = catalog::pages();
    for page in &pages {
        let html = get(&page.href).await;
        let aside = &html[html.find("<aside").unwrap()..html.find("</aside>").unwrap()];
        for other in &pages {
            assert!(
                aside.contains(&format!(r#"href="{}""#, other.href)),
                "{}: no link to {}",
                page.href,
                other.href
            );
        }
        for href in ["/llms.txt", crate::GITHUB_URL] {
            assert!(aside.contains(&format!(r#"href="{href}""#)), "{href}");
        }
        assert_eq!(
            aside.matches(r#"aria-current="page""#).count(),
            1,
            "{}",
            page.href
        );
        let at = aside.find(r#"aria-current="page""#).unwrap();
        let tag = &aside[aside[..at].rfind("<a").unwrap()..at + aside[at..].find('>').unwrap()];
        assert!(
            tag.contains(&format!(r#"href="{}""#, page.href)),
            "{} is not the current link",
            page.href
        );
        assert!(
            aside.contains(">Dark mode"),
            "{}: no theme toggle",
            page.href
        );
        // The menu sheet for narrow screens has the same links.
        assert!(html.contains(r#"aria-label="Open menu""#));
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 2);
    }
}

#[tokio::test]
async fn llms_txt_lists_every_page() {
    let txt = get("/llms.txt").await;
    for page in catalog::pages().iter().skip(1) {
        assert!(
            txt.contains(&format!("]({})", page.href)),
            "llms.txt lacks {}",
            page.href
        );
    }
    for doc in catalog::COMPONENTS {
        assert!(txt.contains(&format!(
            "`topcoat ui add {} --registry topcoat-graph`",
            doc.name
        )));
    }
}

#[tokio::test]
async fn an_unknown_docs_page_is_not_found() {
    assert_eq!(status("/docs/scatter_chart").await, 404);
    assert_eq!(status("/docs/").await, 404);
    assert_eq!(status("/bar").await, 404, "the old gallery pages are gone");
}

#[tokio::test]
async fn a_playground_renders_above_the_examples() {
    // A page only tests can reach, whose entry has a playground.
    let html = get("/docs/test_playground").await;
    let heading = html
        .find(r#"id="playground""#)
        .expect("a playground heading");
    let examples = html.find(r#"id="examples""#).unwrap();
    assert!(heading < examples);
    assert!(html[heading..].starts_with(r#"id="playground""#));
    let slot = div_with(&html, r#"data-slot="playground""#);
    assert!(slot.contains(r#"data-test="playground""#), "{slot}");
    assert!(html[heading..examples].contains(">Playground</h2>"));
    // Real pages have none yet.
    for doc in catalog::COMPONENTS {
        assert!(doc.playground.is_none());
    }
    let bar = get("/docs/bar_chart").await;
    assert!(!bar.contains(r#"id="playground""#));
}

#[tokio::test]
async fn element_ids_are_unique_on_every_page() {
    for page in catalog::pages() {
        let html = get(&page.href).await;
        let mut ids: Vec<&str> = html
            .split(r#" id=""#)
            .skip(1)
            .map(|rest| &rest[..rest.find('"').unwrap()])
            .collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "{}: a repeated id", page.href);
    }
}

#[tokio::test]
async fn the_home_page_shows_a_chart_its_code_and_where_to_start() {
    let html = get("/").await;
    assert!(!html.contains("NaN"));
    let main = &html[html.find("<main").unwrap()..];
    assert_eq!(main.matches(r#"role="figure""#).count(), 1);
    assert!(main.contains(r#"href="/docs/introduction""#) && main.contains(">Get Started"));
    assert!(main.contains(&format!(r#"href="{}""#, crate::GITHUB_URL)));
    assert!(code_blocks(main).contains(&catalog::HOME_EXAMPLE.source.to_owned()));
    let chart = figure(&html, "Games won and lost");
    assert_eq!(parts(chart, "legend-item"), 2);
}

#[tokio::test]
async fn the_guides_render_what_they_promise() {
    let installation = get("/docs/installation").await;
    for command in guides::INSTALL_COMMANDS {
        assert!(installation.contains(command), "{command}");
    }
    assert!(text(&installation).contains("RuntimeLayer::new()"));

    let theming = get("/docs/theming").await;
    let swatches = div_with(&theming, r#"data-slot="swatches""#);
    assert_eq!(swatches.matches("background:var(--chart-").count(), 5);
    assert!(
        swatches.contains("[--chart-default-1:"),
        "the defaults resolve"
    );
    assert!(text(&theming).contains("--chart-5: oklch(0.645 0.246 16.439);"));
    let colored = figure(&theming, "This season against the last");
    assert!(colored.contains("stroke:var(--primary)"), "{colored}");
    let small = figure(&theming, "Cards by type, 8rem wide");
    assert!(small.contains("max-w-32"));

    let geometry = get("/docs/geometry").await;
    let svg = &geometry[geometry
        .find(r#"role="img" aria-label="Cards by mana cost"#)
        .unwrap()..];
    let svg = &svg[..svg.find("</svg>").unwrap()];
    assert_eq!(svg.matches("<rect").count(), 6);
    for item in [
        "BarOptions",
        "BarLayout",
        "XyOptions",
        "XyLayout",
        "PieOptions",
        "Slice",
        "bar_layout",
        "xy_layout",
        "pie_layout",
    ] {
        assert!(geometry.contains(&format!(r#"id="item-{item}""#)), "{item}");
    }

    let introduction = get("/docs/introduction").await;
    for doc in catalog::COMPONENTS {
        assert!(introduction.contains(&format!(r#"href="/docs/{}""#, doc.name)));
    }
}

#[tokio::test]
async fn the_blocks_page_shows_each_block_with_its_code() {
    let html = get("/blocks").await;
    assert!(!html.contains("NaN"));
    assert_eq!(html.matches("data-example=").count(), catalog::BLOCKS.len());
    for block in catalog::BLOCKS {
        let section = example_section(&html, block.id);
        assert_eq!(code_blocks(section), [block.source]);
    }
    let chart = figure(&html, "Mana curve of an aggro deck");
    assert_eq!(chart.matches("<rect").count(), 7);
    assert!(chart.contains("aspect-auto h-48"));
    let ticks: Vec<&str> = chart
        .split(r#"data-chart="tick""#)
        .skip(1)
        .map(|rest| {
            let rest = &rest[rest.find('>').unwrap() + 1..];
            &rest[..rest.find('<').unwrap()]
        })
        .collect();
    assert!(
        ticks.iter().all(|tick| tick.parse::<u32>().is_ok()),
        "{ticks:?}"
    );
    assert!(html.contains(">Mana curve</h3>"));
}

#[tokio::test]
async fn the_part_examples_draw_their_charts() {
    let chart = get("/docs/chart").await;
    let sparkline = figure(&chart, "Players this week");
    assert!(sparkline.contains(r#"<th scope="row">Sat</th><td>210</td>"#));
    assert_eq!(sparkline.matches("<path").count(), 1);

    let axis = get("/docs/chart_axis").await;
    let lollipop = figure(&axis, "Games per month");
    assert_eq!(parts(lollipop, "category"), 6, "every other month");
    assert_eq!(
        lollipop.matches("<path").count(),
        24,
        "a stem and a head per month"
    );
    let horizontal = figure(&axis, "Average game length by format");
    assert!(horizontal.contains(">Two-headed giant</span>"));

    let legend = get("/docs/chart_legend").await;
    assert_eq!(parts(figure(&legend, "Games by result"), "legend-item"), 3);

    let tooltip = get("/docs/chart_tooltip").await;
    let sparkline = figure(&tooltip, "Players this week, with tooltips");
    assert_eq!(parts(sparkline, "hit-band"), 7);
    assert_eq!(tooltips(sparkline).len(), 7);
    let custom = figure(&tooltip, "Rating by week");
    let tips = tooltips(custom);
    assert_eq!(tips[1].matches(r#"data-chart="tooltip-row""#).count(), 2);
    assert!(tips[1].contains(">+35<"), "{}", tips[1]);
}

// The blocks.

/// The id, title and chart label of each block added after the mana curve.
const BLOCK_CHARTS: [(&str, &str, &str); 5] = [
    (
        "block_stat_cards",
        "Stat cards over a chart",
        "Games played per week, last 12 weeks",
    ),
    (
        "block_interactive_range",
        "Range and metric switcher",
        "Games per day, last 30 days",
    ),
    (
        "block_multi_line",
        "Win rate by deck",
        "Win rate by deck over eight weeks",
    ),
    (
        "block_donut_total",
        "Donut with a total",
        "Games by archetype this season",
    ),
    (
        "block_ranked_bars",
        "Ranked list",
        "Most played cards in winning decks",
    ),
];

#[tokio::test]
async fn each_block_shows_its_title_and_a_named_chart() {
    let html = get("/blocks").await;
    for (id, title, label) in BLOCK_CHARTS {
        let section = example_section(&html, id);
        assert!(
            section.contains(&format!(">{title}</h3>")),
            "{id}: no title"
        );
        figure(section, label);
    }
}

/// The text of each toggle button in `html` and whether it is pressed.
fn toggles(html: &str) -> Vec<(String, bool)> {
    html.split("<button")
        .skip(1)
        .filter_map(|rest| {
            let tag = tag_rest(rest);
            let pressed = if tag.contains(r#"aria-pressed="true""#) {
                true
            } else if tag.contains(r#"aria-pressed="false""#) {
                false
            } else {
                return None;
            };
            let body = &rest[tag.len() + 1..];
            let body = &body[..body.find("</button>").unwrap()];
            Some((text(body).trim().to_owned(), pressed))
        })
        .collect()
}

#[tokio::test]
async fn the_interactive_block_starts_on_thirty_days_of_games() {
    let html = get("/blocks").await;
    let section = example_section(&html, "block_interactive_range");
    let chart = figure(section, "Games per day, last 30 days");
    // A header row and a row per day.
    assert_eq!(chart.matches("<tr>").count(), 31, "{chart}");
    assert!(chart.contains(r#"<th scope="row">Day 61</th>"#));
    assert!(chart.contains(r#"<th scope="row">Day 90</th>"#));
    assert!(!chart.contains(r#"<th scope="row">Day 60</th>"#));
    // One label in five.
    assert_eq!(parts(chart, "category"), 6);
    let toggles = toggles(section);
    let expected = [
        ("7 days", false),
        ("30 days", true),
        ("90 days", false),
        ("Games", true),
        ("Wins", false),
    ];
    assert_eq!(
        toggles,
        expected.map(|(name, pressed)| (name.to_owned(), pressed)),
    );
    for button in section.split("<button").skip(1) {
        let tag = tag_rest(button);
        if tag.contains("aria-pressed=") {
            assert!(tag.contains(r#"type="button""#), "{tag}");
        }
    }
    assert!(section.contains(r#"role="group""#));
}

#[test]
fn the_interactive_block_treats_its_signals_as_untrusted() {
    use crate::examples::block_interactive_range::{METRICS, RANGES, metric, range};

    for option in &RANGES {
        assert_eq!(range(option.days).days, option.days);
    }
    for value in [0, 1, 29, 31, 89, 91, usize::MAX] {
        assert_eq!(range(value).days, 30, "{value}");
    }
    for option in &METRICS {
        assert_eq!(metric(option.key).key, option.key);
    }
    for value in ["", "WINS", " wins", "wins ", "losses", "<script>"] {
        assert_eq!(metric(value).key, "games", "{value:?}");
    }
}

#[test]
fn the_interactive_block_generates_the_same_days_every_time() {
    use crate::examples::block_interactive_range::{METRICS, RANGES, daily};

    let games = &METRICS[0];
    let wins = &METRICS[1];
    let (all_days, all_games) = daily(games, 90);
    assert_eq!(all_days.len(), 90);
    assert_eq!(all_games.len(), 90);
    assert_eq!(all_days.first().unwrap(), "Day 1");
    assert_eq!(all_days.last().unwrap(), "Day 90");
    assert_eq!(daily(games, 90), (all_days.clone(), all_games.clone()));
    let (_, all_wins) = daily(wins, 90);
    for (games, wins) in all_games.iter().zip(&all_wins) {
        assert!(games.is_finite() && *games > 0.0 && games.fract() == 0.0);
        assert!(wins.is_finite() && *wins >= 0.0 && wins <= games);
    }
    // A shorter range is the end of the longest one.
    for option in &RANGES {
        let (days, values) = daily(games, option.days);
        assert_eq!(days.len(), option.days);
        assert_eq!(days[..], all_days[90 - option.days..]);
        assert_eq!(values[..], all_games[90 - option.days..]);
    }
    // More days than there is data for gives what there is.
    assert_eq!(daily(games, 1000).0.len(), 90);
}

/// Each `<th scope="row">` and the number in the cell after it, from a chart's table.
fn table_rows(chart: &str) -> Vec<(String, f64)> {
    chart
        .split(r#"<th scope="row">"#)
        .skip(1)
        .map(|rest| {
            let (name, rest) = rest.split_once("</th><td>").unwrap();
            let value = &rest[..rest.find("</td>").unwrap()];
            (text(name), value.parse().unwrap())
        })
        .collect()
}

#[tokio::test]
async fn the_ranked_block_lists_its_cards_from_most_to_least() {
    let html = get("/blocks").await;
    let chart = figure(&html, "Most played cards in winning decks");
    let rows = table_rows(chart);
    assert!((6..=8).contains(&rows.len()), "{rows:?}");
    assert!(
        rows.windows(2).all(|pair| pair[0].1 >= pair[1].1),
        "{rows:?}"
    );
    assert!(rows.iter().any(|(name, _)| name.len() > 20), "long labels");
    assert_eq!(chart.matches("<rect").count(), rows.len());
    assert!(chart.contains(r#"x="0""#), "horizontal");
}

#[tokio::test]
async fn the_donut_block_shows_its_total_and_shares_that_add_up() {
    let html = get("/blocks").await;
    let chart = figure(&html, "Games by archetype this season");
    let rows = table_rows(chart);
    let total: f64 = rows.iter().map(|(_, value)| value).sum();
    assert_eq!(parts(chart, "center"), 1);
    let center = div_with(chart, r#"data-chart="center""#);
    assert!(text(center).contains(&total.to_string()), "{center}");
    assert_eq!(parts(chart, "legend-item"), rows.len());

    let section = example_section(&html, "block_donut_total");
    let shares = div_with(section, r#"data-slot="shares""#);
    let percents: Vec<f64> = shares
        .split("%</td>")
        .map(|before| &before[before.rfind('>').map_or(0, |at| at + 1)..])
        .filter_map(|cell| cell.parse().ok())
        .collect();
    assert_eq!(percents.len(), rows.len(), "{shares}");
    let sum: f64 = percents.iter().sum();
    assert!((sum - 100.0).abs() <= 0.05 * rows.len() as f64, "{sum}");
    for (name, value) in &rows {
        assert!(shares.contains(&format!(">{name}<")), "{name}");
        assert!(shares.contains(&format!(">{value}<")), "{name}: {value}");
    }
}

#[tokio::test]
async fn the_multi_line_block_sums_up_its_data_in_the_footer() {
    let html = get("/blocks").await;
    let chart = figure(&html, "Win rate by deck over eight weeks");
    assert_eq!(paths(chart, "line").len(), 3);
    assert_eq!(parts(chart, "legend-item"), 3);
    assert!(parts(chart, "dot") > 0);
    assert!(paths(chart, "line")[0].contains('C'), "smooth");
    let section = example_section(&html, "block_multi_line");
    let footer = div_with(section, r#"data-slot="trend""#);
    assert!(text(footer).contains("rose the most"), "{footer}");
}

#[tokio::test]
async fn the_stat_cards_block_shows_four_numbers_over_a_chart() {
    let html = get("/blocks").await;
    let section = example_section(&html, "block_stat_cards");
    assert_eq!(section.matches(r#"data-slot="stat""#).count(), 4);
    assert!(section.contains("from last month"));
    let chart = figure(section, "Games played per week, last 12 weeks");
    assert_eq!(paths(chart, "area").len(), 1);
    assert_eq!(table_rows(chart).len(), 12);
}
