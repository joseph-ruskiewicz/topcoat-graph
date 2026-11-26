use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, StaticClass, View, class, component, view},
};
use topcoat_graph::fmt::num;

/// One series of a chart: a name and a value per category.
#[derive(Clone, Debug)]
pub struct ChartSeries {
    /// The series' name, shown in the legend, the tooltip and the data table.
    pub name: String,
    /// One value per category, in category order. A non-finite value such as
    /// `f64::NAN` means there is no value for that category.
    pub values: Vec<f64>,
    /// A CSS color for the series. Without one it takes the next chart color.
    pub color: Option<String>,
}

impl ChartSeries {
    /// A series in the next chart color.
    ///
    /// ```ignore
    /// ChartSeries::new("Cards", [2.0, 5.0, 12.0])
    /// ```
    pub fn new(name: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self {
            name: name.into(),
            values: values.into(),
            color: None,
        }
    }

    /// Sets the series' color to any CSS color, such as `"var(--primary)"`.
    pub fn color(mut self, css: impl Into<String>) -> Self {
        self.color = Some(css.into());
        self
    }
}

/// Category labels from anything string-like: `labels(["Q1", "Q2"])`.
pub fn labels<T: Into<String>>(items: impl IntoIterator<Item = T>) -> Vec<String> {
    items.into_iter().map(Into::into).collect()
}

/// The CSS color of the series at `index`: `explicit` when given, otherwise one of the
/// five chart colors in turn.
///
/// The chart colors are the theme's `--chart-1` to `--chart-5`. A theme without them
/// gets the defaults in [`CHART_VARS`].
pub fn series_color(index: usize, explicit: Option<&str>) -> String {
    match explicit {
        Some(color) => color.to_owned(),
        None => {
            let n = index % 5 + 1;
            format!("var(--chart-{n}, var(--chart-default-{n}))")
        }
    }
}

/// Default chart colors for light and dark mode, used when the theme does not define
/// `--chart-1` to `--chart-5`. To change a chart's colors, define those tokens in
/// `styles.css` rather than editing these.
pub const CHART_VARS: StaticClass = class!(
    "[--chart-default-1:oklch(0.646_0.222_41.116)] dark:[--chart-default-1:oklch(0.488_0.243_264.376)]",
    "[--chart-default-2:oklch(0.6_0.118_184.704)] dark:[--chart-default-2:oklch(0.696_0.17_162.48)]",
    "[--chart-default-3:oklch(0.398_0.07_227.392)] dark:[--chart-default-3:oklch(0.769_0.188_70.08)]",
    "[--chart-default-4:oklch(0.828_0.189_84.429)] dark:[--chart-default-4:oklch(0.627_0.265_303.9)]",
    "[--chart-default-5:oklch(0.769_0.188_70.08)] dark:[--chart-default-5:oklch(0.645_0.246_16.439)]",
);

/// A percentage for an inline style, from a position in the plot's 0 to 100 box.
pub fn percent(value: f64) -> String {
    format!("{}%", num(value))
}

/// The frame every chart sits in: a figure with an accessible name.
///
/// The chart is as wide as its container and has a 16:9 shape by default. Set a
/// different shape or a fixed height through `attrs`, for example
/// `attributes! { class="aspect-auto h-64" }`.
///
/// `attrs` are forwarded to the `<div>`. Extra classes are added to its classes.
#[component]
pub async fn chart(
    cx: &Cx,
    /// The chart's accessible name: what it shows, such as "Mana cost distribution".
    #[into]
    label: String,
    /// Extra attributes for the root `<div>` element.
    #[default]
    mut attrs: Attributes,
    /// The plot, axes, legend and data table.
    #[default]
    child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div
            role="figure"
            aria-label=(label)
            class=(class!(
                "relative flex aspect-video w-full flex-col gap-2 text-xs text-muted-foreground",
                CHART_VARS,
                attrs.remove("class"),
            ))
            (attrs)
        >
            (child)
        </div>
    })
}

/// A chart's data as a table that only assistive technology reads.
///
/// The drawn chart is hidden from screen readers; this is what they get instead. Rows
/// are categories and columns are series. A missing value is an empty cell.
#[component]
pub async fn chart_table(
    cx: &Cx,
    /// The chart's accessible name, repeated as the table's caption.
    #[into]
    label: String,
    /// The heading of the category column, such as "Cost".
    #[into]
    category_header: String,
    /// The category labels, one per row.
    categories: Vec<String>,
    /// The series, one per column.
    series: Vec<ChartSeries>,
) -> Result<impl View> {
    Ok(view! {
        <table class="sr-only">
            <caption>(label)</caption>
            <thead>
                <tr>
                    <th scope="col">(category_header)</th>
                    for one in &series {
                        <th scope="col">(&one.name)</th>
                    }
                </tr>
            </thead>
            <tbody>
                for (index, category) in categories.iter().enumerate() {
                    <tr>
                        <th scope="row">(category)</th>
                        for one in &series {
                            <td>
                                (one
                                    .values
                                    .get(index)
                                    .filter(|v| v.is_finite())
                                    .map(|v| num(*v)))
                            </td>
                        }
                    </tr>
                }
            </tbody>
        </table>
    })
}
