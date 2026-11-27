use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, shard, signal},
    view::{View, attributes, component, view},
};

use crate::components::{
    bar_chart::bar_chart,
    button::{ButtonSize, ButtonVariant, button},
    card::{card, card_content, card_description, card_header, card_title},
    chart::ChartSeries,
};

/// A date range the reader can pick.
pub struct Range {
    pub days: usize,
    pub name: &'static str,
    /// Show one day label in this many, so the axis doesn't crowd.
    pub label_every: usize,
}

pub const RANGES: [Range; 3] = [
    Range {
        days: 7,
        name: "7 days",
        label_every: 1,
    },
    Range {
        days: 30,
        name: "30 days",
        label_every: 5,
    },
    Range {
        days: 90,
        name: "90 days",
        label_every: 15,
    },
];

/// The range shown first, and whenever the browser sends one we don't offer.
const DEFAULT_RANGE: usize = 1;

/// A number the chart can show.
pub struct Metric {
    pub key: &'static str,
    pub name: &'static str,
}

pub const METRICS: [Metric; 2] = [
    Metric {
        key: "games",
        name: "Games",
    },
    Metric {
        key: "wins",
        name: "Wins",
    },
];

/// The range with `days` days, or the default one. The value comes from the browser,
/// so it can be anything.
pub fn range(days: usize) -> &'static Range {
    RANGES
        .iter()
        .find(|range| range.days == days)
        .unwrap_or(&RANGES[DEFAULT_RANGE])
}

/// The metric named `key`, or the first one.
pub fn metric(key: &str) -> &'static Metric {
    METRICS
        .iter()
        .find(|metric| metric.key == key)
        .unwrap_or(&METRICS[0])
}

/// How many days of data there are.
const DAYS: usize = 90;

/// Games played on `day`, from 1 to 90: a slow rise, busy weekends and a fixed
/// wobble, so every render draws the same chart.
fn games_on(day: usize) -> usize {
    let weekend = [0, 2, 1, 3, 4, 9, 8][day % 7];
    24 + day / 6 + weekend + (day * 37) % 9
}

/// Games won on `day`: between 40% and 60% of the games played.
fn wins_on(day: usize) -> usize {
    games_on(day) * (40 + (day * 13) % 21) / 100
}

/// The day labels and values of `metric` for the last `days` days.
pub fn daily(metric: &Metric, days: usize) -> (Vec<String>, Vec<f64>) {
    let first = DAYS - days.min(DAYS) + 1;
    (first..=DAYS)
        .map(|day| {
            let value = if metric.key == "wins" {
                wins_on(day)
            } else {
                games_on(day)
            };
            (format!("Day {day}"), value as f64)
        })
        .unzip()
}

/// The card. It is a shard: when a button sets a signal, the browser asks the server
/// to render it again, and the chart is drawn for the new choice.
#[shard]
async fn games_card(cx: &Cx) -> Result<impl View> {
    let days = signal(cx, || RANGES[DEFAULT_RANGE].days);
    let key = signal(cx, || METRICS[0].key.to_owned());
    // Both values come back from the browser: check them before use.
    let range = range(days.get());
    let metric = metric(&key.get());

    let (categories, values) = daily(metric, range.days);
    let total: f64 = values.iter().sum();
    let noun = metric.name.to_lowercase();

    Ok(view! {
        card(
            attrs: attributes! { class="@container w-full" },
            card_header(
                attrs: attributes! {
                    class="gap-4 @xl:flex-row @xl:items-start @xl:justify-between"
                },
                <div class="flex flex-col gap-1.5">
                    card_title("Daily activity")
                    card_description(
                        (format!("{total} {noun} in the last {} days", range.days))
                    )
                </div>
                <div class="flex flex-wrap gap-2">
                    <div
                        role="group"
                        aria-label="Range"
                        class="flex rounded-lg border border-border p-0.5"
                    >
                        for option in &RANGES {
                            let value = option.days;
                            button(
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Sm,
                                attrs: attributes! {
                                    type="button"
                                    class="aria-pressed:bg-foreground/10"
                                    aria-pressed=(if option.days == range.days {
                                        "true"
                                    } else {
                                        "false"
                                    })
                                    @click=$(|_e: Event| days.set(value))
                                },
                                (option.name)
                            )
                        }
                    </div>
                    <div
                        role="group"
                        aria-label="Metric"
                        class="flex rounded-lg border border-border p-0.5"
                    >
                        for option in &METRICS {
                            let value = option.key;
                            button(
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Sm,
                                attrs: attributes! {
                                    type="button"
                                    class="aria-pressed:bg-foreground/10"
                                    aria-pressed=(if option.key == metric.key {
                                        "true"
                                    } else {
                                        "false"
                                    })
                                    @click=$(|_e: Event| key.set(value.to_owned()))
                                },
                                (option.name)
                            )
                        }
                    </div>
                </div>
            )
            card_content(
                bar_chart(
                    label: format!("{} per day, last {} days", metric.name, range.days),
                    category_header: "Day",
                    categories: categories,
                    series: vec![ChartSeries::new(metric.name, values)],
                    integer_ticks: true,
                    label_every: range.label_every,
                    attrs: attributes! { class="aspect-auto h-60" }
                )
            )
        )
    })
}

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! { games_card() })
}
