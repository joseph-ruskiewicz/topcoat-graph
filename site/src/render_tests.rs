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
    let html = get("/bar").await;
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
        assert!(renders_hidden(tooltip), "visible before hover");
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

/// The tooltips in `html`, each from its marker to the end of the markup.
fn tooltips(html: &str) -> Vec<&str> {
    html.split(r#"data-chart="tooltip""#).skip(1).collect()
}

#[tokio::test]
async fn a_pie_draws_one_slice_per_value_in_a_square() {
    let html = get("/pie").await;
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
    let html = get("/pie").await;
    let chart = figure(&html, "Cards by rarity");
    let slices = paths(chart, "slice");
    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].matches('A').count(), 2, "{}", slices[0]);
}

#[tokio::test]
async fn an_all_zero_pie_draws_no_slices_but_keeps_its_table() {
    let html = get("/pie").await;
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
    let html = get("/pie").await;
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
    let html = get("/pie").await;
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
    let html = get("/pie").await;
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
    let html = get("/pie").await;
    let chart = figure(&html, "Cards by type");
    assert_eq!(chart.matches(r#"tabindex="0""#).count(), 3);
    assert_eq!(chart.matches("aria-describedby=").count(), 3);
    assert!(chart.contains(r#"aria-label="Artifacts""#));
}

#[tokio::test]
async fn a_zero_slice_is_skipped_but_listed_and_keeps_the_colors_in_place() {
    let html = get("/pie").await;
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
async fn the_home_page_shows_one_of_each_chart_and_links_to_its_page() {
    let html = get("/").await;
    assert!(!html.contains("NaN"));
    assert_eq!(html.matches(r#"role="figure""#).count(), 4);
    let main = &html[html.find("<main").unwrap()..];
    for page in ["/bar", "/line", "/area", "/pie"] {
        assert!(
            main.contains(&format!(r#"href="{page}""#)),
            "no link to {page}"
        );
    }
    assert!(
        html.contains("cargo add topcoat-graph --git"),
        "no install steps"
    );
}

#[tokio::test]
async fn every_page_has_the_navigation_and_marks_the_current_one() {
    for page in ["/", "/bar", "/line", "/area", "/pie"] {
        let html = get(page).await;
        let header = &html[html.find("<header").unwrap()..html.find("</header>").unwrap()];
        for link in ["/", "/bar", "/line", "/area", "/pie"] {
            assert!(
                header.contains(&format!(r#"href="{link}""#)),
                "{page}: {link}"
            );
        }
        assert_eq!(
            header.matches(r#"aria-current="page""#).count(),
            1,
            "{page}"
        );
        let at = header.find(r#"aria-current="page""#).unwrap();
        let tag = &header[header[..at].rfind("<a").unwrap()..at + header[at..].find('>').unwrap()];
        assert!(
            tag.contains(&format!(r#"href="{page}""#)),
            "{page} is not the current link"
        );
        assert!(header.contains("aria-pressed="), "{page}: no theme toggle");
    }
}

#[tokio::test]
async fn every_example_shows_the_code_that_drew_it() {
    for page in ["/bar", "/line", "/area", "/pie"] {
        let html = get(page).await;
        let sections: Vec<&str> = html.split("<section").skip(1).collect();
        assert!(!sections.is_empty(), "{page}");
        for section in sections {
            let code = &section[section
                .find("<code>")
                .unwrap_or_else(|| panic!("{page}: no code"))
                + "<code>".len()..];
            let code = &code[..code.find("</code>").unwrap()];
            assert!(
                code.contains("_chart("),
                "{page}: no chart call in {code:?}"
            );
            assert!(
                !code.contains("// example:") && !code.contains("// end"),
                "{code}"
            );
            // The code is dedented: its first line starts at the margin.
            assert!(
                code.starts_with(|c: char| c.is_ascii_lowercase()),
                "{code:?}"
            );
        }
        assert_eq!(
            html.matches("<section").count(),
            html.matches(r#"role="figure""#).count(),
            "{page}: an example without a chart, or a chart outside an example"
        );
    }
}

#[tokio::test]
async fn a_tooltip_never_opens_over_its_own_category() {
    let html = get("/bar").await;
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
    let html = get("/line").await;
    let chart = figure(&html, "Error rate by release");
    assert!(chart.contains("<td>0.0012</td>"), "{chart}");
    assert!(chart.contains(">0.0012</span>"));
    assert!(!chart.contains("<td>0</td>"));
}

#[tokio::test]
async fn a_pie_tooltip_shows_small_shares() {
    let html = get("/pie").await;
    let chart = figure(&html, "Games by platform");
    assert!(chart.contains("3 (0.03%)"), "{chart}");
    assert!(!chart.contains("(0%)"));
}
