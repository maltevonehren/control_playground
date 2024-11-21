use std::fmt::Write;
use std::rc::Rc;

use engine::NiceFloat;
use leptos::*;
use leptos_use::{use_element_size, UseElementSizeReturn};
use nalgebra::{DMatrix, Dim, MatrixView1xX};

#[component]
pub fn SVGGraph(
    #[prop(into)] data: Signal<Rc<DMatrix<f64>>>,
    initial_height: f64,
) -> impl IntoView {
    let el = create_node_ref::<html::Div>();
    let UseElementSizeReturn { width, height } = use_element_size(el);

    let colors = &["red", "blue"];
    let margin_left = 50.;
    let margin_top = 20.;
    let margin_right = 20.;
    let margin_bottom = 50.;
    let tightest_x_tick_spacing = 80.;
    let tightest_y_tick_spacing = 60.;
    let height = move || height.get().max(margin_top + margin_bottom + 5.0);
    let graph_width = move || width.get() - margin_left - margin_right;
    let graph_height = move || height() - margin_top - margin_bottom;
    let x_min_max = create_memo(move |_| (0.0, data.get().ncols() as f64 - 1.0));
    let y_min_max = create_memo(move |_| (data.get().min(), data.get().max()));

    let x_axis = create_memo(move |_| {
        let max_num_ticks = (graph_width() / tightest_x_tick_spacing).floor() as usize + 1;
        Axis::new(x_min_max.get(), max_num_ticks)
    });
    let y_axis = create_memo(move |_| {
        let max_num_ticks = (graph_height() / tightest_y_tick_spacing).floor() as usize + 1;
        Axis::new(y_min_max.get(), max_num_ticks)
    });
    let mapping = create_memo(move |_| {
        Mapping::new(x_axis.get(), y_axis.get(), graph_width(), graph_height())
    });
    view! {
        <div node_ref=el style:overflow="hidden" style:resize="vertical" style:height=format!("{initial_height}px")>
        <svg width="100%" height="100%" style:stroke-width="2px" >
        <g transform=move || format!("translate({margin_left} {})", height() - margin_bottom)>
            <path fill="white" stroke="none"
                d={move || format!("M 0,0 V{} H{} V0 H0", -graph_height(), graph_width())} />
            {move || {
                let mapping = mapping.get();
                data.get().row_iter().enumerate().map(
                    |(i, row)| make_path(colors[i % colors.len()], row, &mapping)
                ).collect_view()
            }}
            {move || {
                let mapping = mapping.get();
                x_axis.get().ticks()
                    .map(|pos| make_x_tick(pos, &mapping, graph_height()))
                    .collect_view()
            }}
            {move || {
                let mapping = mapping.get();
                y_axis.get().ticks()
                    .map(|pos| make_y_tick(pos, &mapping, graph_width()))
                    .collect_view()
            }}
            <path fill="none" stroke="black"
                d={move || format!("M 0,0 V{} H{} V0 H0", -graph_height(), graph_width())} />
        </g>
        </svg>
        </div>
    }
}

/// Conversion to svg space
#[derive(Clone, Copy, Debug, PartialEq)]
struct Mapping {
    /// size of one graph space unit in svg space
    x_scale: f64,
    /// size of one graph space unit in svg space
    y_scale: f64,
    /// graph space value that corresponds to 0 in svg space
    x_min: f64,
    /// graph space value that corresponds to 0 in svg space
    y_min: f64,
}

impl Mapping {
    fn new(x: Axis, y: Axis, width: f64, heigth: f64) -> Self {
        Self {
            x_scale: width / (x.max - x.min),
            y_scale: heigth / (y.max - y.min),
            x_min: x.min,
            y_min: y.min,
        }
    }

    fn map_x(&self, x: f64) -> f64 {
        (x - self.x_min) * self.x_scale
    }
    fn map_y(&self, y: f64) -> f64 {
        (self.y_min - y) * self.y_scale
    }
    fn map(&self, (x, y): (f64, f64)) -> (f64, f64) {
        (self.map_x(x), self.map_y(y))
    }
}

/// Specifies an axis: min, max, label, lin or log scaling, tick placement
///
/// TODO:
/// - When displaying radians use multiples of pi
/// - rad2deg
/// - logarithmic scales (for data and or ticks)
/// - optionally force 0 to be included
/// - symmetric wrt. 0
#[derive(Clone, Copy, Debug, PartialEq)]
struct Axis {
    min: f64,
    max: f64,
    /// distance of two tick marks in graph space
    step: f64,
}

impl Axis {
    fn new((mut min, mut max): (f64, f64), max_num_ticks: usize) -> Self {
        if max == min {
            min -= 0.5;
            max += 0.5;
        }
        let delta = max - min;
        // TODO: make sure max_num_ticks is actually respected
        let scale = (delta / (max_num_ticks as f64 + 1.0)).log10().floor();
        let factor = (10f64).powf(scale);
        let (mut step, mut best) = (1.0, 0);
        for mut s in [1., 1.5, 2., 2.5, 3., 4., 5., 6., 8., 10.] {
            s *= factor;
            let num_ticks = (delta / s).floor() as usize;
            if num_ticks >= best && num_ticks <= max_num_ticks {
                step = s;
                best = num_ticks;
            }
        }

        Axis { min, max, step }
    }

    fn ticks(&self) -> impl Iterator<Item = f64> {
        let t_min = (self.min / self.step).ceil() as isize;
        let t_max = (self.max / self.step).floor() as isize;
        let step = self.step;
        (t_min..=t_max).map(move |t| t as f64 * step)
    }
}

fn make_path(
    color: &'static str,
    // x: MatrixView1xX<f64, impl Dim, impl Dim>,
    y: MatrixView1xX<f64, impl Dim, impl Dim>,
    m: &Mapping,
) -> impl IntoView {
    let mut path = "M".to_string();
    for (x, y) in y.iter().enumerate() {
        let (x, y) = m.map((x as f64, *y));
        write!(path, " {},{}", x, y).unwrap();
    }
    view! {
        <path fill="none" stroke=color stroke-linejoin="round" stroke-width=2. stroke-linecap="round" d=path/>
    }
}

fn make_x_tick(pos: f64, m: &Mapping, graph_height: f64) -> impl IntoView {
    let p = m.map_x(pos);
    view! {
        <text text-anchor="middle" x=p y=20 >{format!("{}", NiceFloat(pos))}</text>
        <path fill="none" stroke="gray" stroke-width=1 d=format!("M {p},0 V{}", -graph_height)/>
        <path fill="none" stroke="black" d=format!("M {p},0 V-5")/>
        <path fill="none" stroke="black" d=format!("M {p},{} v5", -graph_height)/>
    }
}

fn make_y_tick(pos: f64, m: &Mapping, graph_width: f64) -> impl IntoView {
    let p = m.map_y(pos);
    view! {
        <text text-anchor="end" x=-5 y=p>{format!("{}", NiceFloat(pos))}</text>
        <path fill="none" stroke="gray" stroke-width=1 d=format!("M 0,{p} H{graph_width}")/>
        <path fill="none" stroke="black" d=format!("M 0,{p} H5")/>
        <path fill="none" stroke="black" d=format!("M {graph_width},{p} h-5")/>
    }
}
