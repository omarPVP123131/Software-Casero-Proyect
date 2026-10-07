//! Widgets gráficos auxiliares: barra espectral y gráfica de datos externos.

use egui_macroquad::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2,
};

use crate::state::{ChartViewState, PhysicsReadout};
use crate::theme::Palette;

pub fn spectrum_color(wavelength_nm: f32) -> Color32 {
    match wavelength_nm {
        value if value < 380.0 => Color32::from_rgb(165, 111, 255),
        value if value < 450.0 => Color32::from_rgb(107, 139, 255),
        value if value < 510.0 => Color32::from_rgb(68, 197, 232),
        value if value < 570.0 => Color32::from_rgb(83, 218, 160),
        value if value < 610.0 => Color32::from_rgb(255, 207, 91),
        value if value < 700.0 => Color32::from_rgb(255, 143, 86),
        _ => Color32::from_rgb(255, 91, 106),
    }
}

pub fn draw_spectrum_bar(ui: &mut egui::Ui, wavelength_nm: f32, palette: Palette) {
    let width = ui.available_width().max(220.0);
    let (response, painter) = ui.allocate_painter(Vec2::new(width, 52.0), Sense::hover());
    let rect = response.rect;
    let bar = Rect::from_min_max(
        Pos2::new(rect.left() + 8.0, rect.top() + 7.0),
        Pos2::new(rect.right() - 8.0, rect.top() + 28.0),
    );
    let colors = [
        Color32::from_rgb(147, 91, 231),
        Color32::from_rgb(91, 112, 247),
        Color32::from_rgb(57, 179, 232),
        Color32::from_rgb(72, 211, 159),
        Color32::from_rgb(239, 216, 77),
        Color32::from_rgb(250, 147, 74),
        Color32::from_rgb(233, 74, 90),
    ];
    let segment_width = bar.width() / colors.len() as f32;
    for (index, color) in colors.iter().enumerate() {
        let left = bar.left() + segment_width * index as f32;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(left, bar.top()),
                Pos2::new(left + segment_width + 1.0, bar.bottom()),
            ),
            egui::CornerRadius::ZERO,
            *color,
        );
    }
    painter.rect_stroke(
        bar,
        egui::CornerRadius::same(4),
        Stroke::new(1.0_f32, palette.border),
        StrokeKind::Inside,
    );

    let fraction = ((wavelength_nm - 180.0) / (900.0 - 180.0)).clamp(0.0, 1.0);
    let marker_x = bar.left() + fraction * bar.width();
    painter.line_segment(
        [
            Pos2::new(marker_x, bar.top() - 4.0),
            Pos2::new(marker_x, bar.bottom() + 4.0),
        ],
        Stroke::new(2.0_f32, Color32::WHITE),
    );
    painter.circle_filled(Pos2::new(marker_x, bar.top() - 4.0), 3.2, Color32::WHITE);

    painter.text(
        Pos2::new(bar.left(), rect.bottom() - 4.0),
        Align2::LEFT_BOTTOM,
        "UV · 180 nm",
        FontId::proportional(9.0),
        palette.muted,
    );
    painter.text(
        Pos2::new(bar.center().x, rect.bottom() - 4.0),
        Align2::CENTER_BOTTOM,
        "VISIBLE",
        FontId::proportional(9.0),
        palette.text,
    );
    painter.text(
        Pos2::new(bar.right(), rect.bottom() - 4.0),
        Align2::RIGHT_BOTTOM,
        "IR · 900 nm",
        FontId::proportional(9.0),
        palette.muted,
    );
    let band = if wavelength_nm < 380.0 {
        "UV"
    } else if wavelength_nm <= 700.0 {
        "visible"
    } else {
        "IR"
    };
    response.on_hover_text(format!(
        "Longitud de onda: {:.0} nm · banda {} · f = c/λ para el motor físico",
        wavelength_nm, band
    ));
}

pub fn draw_chart(
    ui: &mut egui::Ui,
    readout: &PhysicsReadout,
    selected_wavelength_nm: f32,
    chart: &mut ChartViewState,
    palette: Palette,
) {
    egui::Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(14))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("ENERGÍA CINÉTICA MÁXIMA")
                            .size(10.0)
                            .strong()
                            .color(palette.muted),
                    );
                    ui.label(
                        RichText::new("Curva suministrada por el motor físico")
                            .small()
                            .color(palette.text),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Restablecer vista").clicked() {
                        *chart = ChartViewState::default();
                    }
                    ui.add(
                        egui::Slider::new(&mut chart.zoom, 0.75..=3.0)
                            .text("Zoom")
                            .show_value(true),
                    );
                });
            });
            ui.add_space(8.0);

            let desired_size = Vec2::new(ui.available_width().max(260.0), ui.available_height().clamp(220.0, 430.0));
            let (response, painter) = ui.allocate_painter(desired_size, Sense::click_and_drag());
            let rect = response.rect;
            painter.rect_filled(
                rect,
                egui::CornerRadius::same(10),
                palette.background,
            );
            painter.rect_stroke(
                rect,
                egui::CornerRadius::same(10),
                Stroke::new(1.0_f32, palette.border),
                StrokeKind::Inside,
            );
            let plot = plot_rect(rect);
            draw_axes(&painter, rect, plot, palette);

            if response.dragged() {
                let drag_x = ui.input(|input| input.pointer.delta().x);
                chart.pan += drag_x / plot.width().max(1.0) * 0.8;
                chart.pan = chart.pan.clamp(-1.5, 1.5);
            }
            let scroll = if response.hovered() {
                ui.input(|input| input.smooth_scroll_delta.y)
            } else {
                0.0
            };
            if scroll.abs() > 0.0 {
                chart.zoom = (chart.zoom * (1.0 + scroll * 0.001)).clamp(0.75, 3.0);
            }

            if readout.kinetic_energy_curve.len() >= 2 {
                draw_data(
                    &painter,
                    ChartDataView {
                        full: rect,
                        plot,
                        readout,
                        selected_wavelength_nm,
                        chart,
                        palette,
                        hover: response.hover_pos(),
                    },
                );
            } else {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "Calculando curva Kmax(λ)…",
                    FontId::proportional(13.0),
                    palette.muted,
                );
            }

            if let Some(position) = response.hover_pos() {
                if plot.contains(position) && readout.kinetic_energy_curve.len() >= 2 {
                    ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::Crosshair);
                }
            }
            response.on_hover_text("Arrastra para desplazar y usa la rueda para ampliar. El cursor inspecciona la curva Kmax(λ) del motor.");
        });
}

fn plot_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        Pos2::new(rect.left() + 54.0, rect.top() + 31.0),
        Pos2::new(rect.right() - 18.0, rect.bottom() - 43.0),
    )
}

fn draw_axes(painter: &egui::Painter, full: Rect, plot: Rect, palette: Palette) {
    let x_ticks = (plot.width() / 100.0).floor().clamp(2.0, 6.0) as usize;
    let y_ticks = (plot.height() / 58.0).floor().clamp(2.0, 6.0) as usize;
    for i in 0..=y_ticks {
        let fraction = i as f32 / y_ticks as f32;
        let y = plot.bottom() - fraction * plot.height();
        painter.line_segment(
            [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
            Stroke::new(0.7_f32, palette.border.gamma_multiply(0.8)),
        );
    }
    for i in 0..=x_ticks {
        let fraction = i as f32 / x_ticks as f32;
        let x = plot.left() + fraction * plot.width();
        painter.line_segment(
            [Pos2::new(x, plot.top()), Pos2::new(x, plot.bottom())],
            Stroke::new(0.6_f32, palette.border.gamma_multiply(0.45)),
        );
    }
    painter.line_segment(
        [
            Pos2::new(plot.left(), plot.top()),
            Pos2::new(plot.left(), plot.bottom()),
        ],
        Stroke::new(1.2_f32, palette.muted),
    );
    painter.line_segment(
        [
            Pos2::new(plot.left(), plot.bottom()),
            Pos2::new(plot.right(), plot.bottom()),
        ],
        Stroke::new(1.2_f32, palette.muted),
    );
    painter.text(
        Pos2::new(full.left() + 8.0, full.top() + 13.0),
        Align2::LEFT_CENTER,
        "Kmax (eV)",
        FontId::proportional(10.0),
        palette.muted,
    );
    painter.text(
        Pos2::new(plot.center().x, full.bottom() - 11.0),
        Align2::CENTER_CENTER,
        "Longitud de onda (nm)",
        FontId::proportional(10.0),
        palette.muted,
    );
}

struct ChartDataView<'a> {
    full: Rect,
    plot: Rect,
    readout: &'a PhysicsReadout,
    selected_wavelength_nm: f32,
    chart: &'a ChartViewState,
    palette: Palette,
    hover: Option<Pos2>,
}

fn draw_data(painter: &egui::Painter, view: ChartDataView<'_>) {
    let ChartDataView {
        full,
        plot,
        readout,
        selected_wavelength_nm,
        chart,
        palette,
        hover,
    } = view;
    let points = &readout.kinetic_energy_curve;
    let min_x = points
        .iter()
        .map(|p| p.wavelength_nm)
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|p| p.wavelength_nm)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = points
        .iter()
        .map(|p| p.max_kinetic_energy_ev)
        .fold(f64::INFINITY, f64::min)
        .min(0.0);
    let max_y = points
        .iter()
        .map(|p| p.max_kinetic_energy_ev)
        .fold(f64::NEG_INFINITY, f64::max);
    let x_center = (min_x + max_x) * 0.5 + (max_x - min_x) * chart.pan as f64;
    let x_span = ((max_x - min_x) / chart.zoom.max(0.75) as f64).max(f64::EPSILON);
    let visible_min_x = x_center - x_span * 0.5;
    let y_span = (max_y - min_y).max(f64::EPSILON);

    let to_screen = |x: f64, y: f64| -> Pos2 {
        let x_fraction = ((x - visible_min_x) / x_span) as f32;
        let y_fraction = ((y - min_y) / y_span) as f32;
        Pos2::new(
            plot.left() + x_fraction * plot.width(),
            plot.bottom() - y_fraction * plot.height(),
        )
    };

    let x_ticks = (plot.width() / 100.0).floor().clamp(2.0, 6.0) as usize;
    let y_ticks = (plot.height() / 58.0).floor().clamp(2.0, 6.0) as usize;
    for tick in 0..=x_ticks {
        let fraction = tick as f32 / x_ticks as f32;
        let x_value = visible_min_x + fraction as f64 * x_span;
        painter.text(
            Pos2::new(plot.left() + fraction * plot.width(), plot.bottom() + 13.0),
            Align2::CENTER_CENTER,
            format!("{x_value:.0}"),
            FontId::proportional(9.0),
            palette.muted,
        );
    }
    for tick in 0..=y_ticks {
        let fraction = tick as f32 / y_ticks as f32;
        let y_value = min_y + fraction as f64 * y_span;
        painter.text(
            Pos2::new(plot.left() - 7.0, plot.bottom() - fraction * plot.height()),
            Align2::RIGHT_CENTER,
            format!("{y_value:.2}"),
            FontId::proportional(9.0),
            palette.muted,
        );
    }

    for pair in points.windows(2) {
        painter.line_segment(
            [
                to_screen(pair[0].wavelength_nm, pair[0].max_kinetic_energy_ev),
                to_screen(pair[1].wavelength_nm, pair[1].max_kinetic_energy_ev),
            ],
            Stroke::new(2.2_f32, palette.cyan),
        );
    }
    for point in points.iter().step_by((points.len() / 18).max(1)) {
        let p = to_screen(point.wavelength_nm, point.max_kinetic_energy_ev);
        painter.circle_filled(p, 3.0, palette.cyan);
    }

    if let Some(threshold_nm) = readout.threshold_wavelength_nm {
        if threshold_nm >= visible_min_x && threshold_nm <= visible_min_x + x_span {
            let x = to_screen(threshold_nm, min_y).x;
            painter.line_segment(
                [Pos2::new(x, plot.top()), Pos2::new(x, plot.bottom())],
                Stroke::new(1.2_f32, palette.amber),
            );
            painter.text(
                Pos2::new(x + 4.0, plot.top() + 12.0),
                Align2::LEFT_CENTER,
                "umbral λ0 = c/f0",
                FontId::proportional(9.0),
                palette.amber,
            );
        }
    }

    let selected = selected_wavelength_nm as f64;
    if selected >= visible_min_x && selected <= visible_min_x + x_span {
        if let Some(nearest) = points.iter().min_by(|a, b| {
            (a.wavelength_nm - selected)
                .abs()
                .total_cmp(&(b.wavelength_nm - selected).abs())
        }) {
            let pos = to_screen(nearest.wavelength_nm, nearest.max_kinetic_energy_ev);
            painter.circle_filled(pos, 5.0, palette.amber);
            painter.circle_stroke(pos, 8.0, Stroke::new(1.0_f32, palette.amber));
        }
    }

    if let Some(pointer) = hover.filter(|p| plot.contains(*p)) {
        painter.line_segment(
            [
                Pos2::new(pointer.x, plot.top()),
                Pos2::new(pointer.x, plot.bottom()),
            ],
            Stroke::new(0.8_f32, palette.muted),
        );
        let x_fraction = ((pointer.x - plot.left()) / plot.width()).clamp(0.0, 1.0);
        let target_x = visible_min_x + x_fraction as f64 * x_span;
        if let Some(nearest) = points.iter().min_by(|a, b| {
            (a.wavelength_nm - target_x)
                .abs()
                .total_cmp(&(b.wavelength_nm - target_x).abs())
        }) {
            let point = to_screen(nearest.wavelength_nm, nearest.max_kinetic_energy_ev);
            painter.circle_filled(point, 4.5, palette.amber);
            painter.text(
                Pos2::new(
                    (point.x + 10.0).min(full.right() - 12.0),
                    (point.y - 10.0).max(full.top() + 28.0),
                ),
                Align2::LEFT_BOTTOM,
                format!(
                    "Longitud de onda {:.1} nm · Kmax {:.3} eV",
                    nearest.wavelength_nm, nearest.max_kinetic_energy_ev
                ),
                FontId::proportional(10.0),
                palette.text,
            );
        }
    }
}
