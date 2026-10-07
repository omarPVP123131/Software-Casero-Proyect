//! Dibujo esquemático de la celda y área para gráficas externas.

use egui_macroquad::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2,
};

use crate::state::{CurvePoint, LabControls, PhysicsReadout};
use crate::theme;

pub fn draw_cell(
    ui: &mut egui::Ui,
    controls: &LabControls,
    readout: &PhysicsReadout,
    time_seconds: f64,
) {
    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(14))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("CELDA FOTOELÉCTRICA")
                        .size(10.0)
                        .strong()
                        .color(theme::MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let status = if readout.emission_possible.is_some() {
                        "LECTURA EXTERNA"
                    } else {
                        "ANIMACIÓN ILUSTRATIVA"
                    };
                    ui.label(RichText::new(status).size(9.0).strong().color(theme::CYAN));
                });
            });
            ui.add_space(8.0);

            let desired_size = Vec2::new(ui.available_width().max(280.0), 286.0);
            let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
            let rect = response.rect;
            painter.rect_filled(
                rect,
                egui::CornerRadius::same(12),
                Color32::from_rgb(11, 20, 37),
            );
            painter.rect_stroke(
                rect,
                egui::CornerRadius::same(12),
                Stroke::new(1.0_f32, theme::BORDER),
                StrokeKind::Inside,
            );

            draw_background_grid(&painter, rect);
            draw_cell_elements(&painter, rect, controls, readout, time_seconds);
            draw_labels(&painter, rect, controls, readout);

            ui.add_space(7.0);
            ui.horizontal(|ui| {
                legend_dot(ui, theme::CYAN, "Fotones");
                legend_dot(ui, theme::GREEN, "Fotoelectrones");
                legend_dot(ui, theme::VIOLET, "Polaridad visual");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("No son trayectorias calculadas")
                            .small()
                            .color(theme::MUTED),
                    );
                });
            });
        });
}

fn draw_background_grid(painter: &egui::Painter, rect: Rect) {
    let grid = Color32::from_rgba_unmultiplied(81, 111, 145, 32);
    let step = 24.0;
    let mut x = rect.left() + 14.0;
    while x < rect.right() {
        painter.line_segment(
            [
                Pos2::new(x, rect.top() + 1.0),
                Pos2::new(x, rect.bottom() - 1.0),
            ],
            Stroke::new(0.5_f32, grid),
        );
        x += step;
    }
    let mut y = rect.top() + 14.0;
    while y < rect.bottom() {
        painter.line_segment(
            [
                Pos2::new(rect.left() + 1.0, y),
                Pos2::new(rect.right() - 1.0, y),
            ],
            Stroke::new(0.5_f32, grid),
        );
        y += step;
    }
}

fn draw_cell_elements(
    painter: &egui::Painter,
    rect: Rect,
    controls: &LabControls,
    readout: &PhysicsReadout,
    time_seconds: f64,
) {
    let cathode_x = rect.left() + rect.width() * 0.32;
    let anode_x = rect.left() + rect.width() * 0.72;
    let mid_y = rect.center().y + 6.0;
    let electrode_half_height = rect.height() * 0.29;
    let beam_start_x = rect.left() + 24.0;
    let beam_end_x = cathode_x - 12.0;
    let gap_width = anode_x - cathode_x;

    // Resplandor suave detrás de la zona de interacción.
    painter.circle_filled(
        Pos2::new((cathode_x + anode_x) * 0.5, mid_y),
        rect.width() * 0.20,
        Color32::from_rgba_unmultiplied(34, 115, 145, 20),
    );

    // Haz de fotones: solo cambia su apariencia con los controles visuales.
    let photon_color = spectrum_color(controls.wavelength_nm);
    let photon_count = if controls.intensity_percent <= 0.5 {
        0
    } else {
        (2.0 + controls.intensity_percent / 16.0).round() as usize
    };
    let rows = 5;
    for row in 0..rows {
        let y = mid_y - electrode_half_height * 0.82
            + row as f32 * (electrode_half_height * 1.64 / (rows - 1) as f32);
        painter.line_segment(
            [Pos2::new(beam_start_x, y), Pos2::new(beam_end_x, y)],
            Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(
                    photon_color.r(),
                    photon_color.g(),
                    photon_color.b(),
                    70,
                ),
            ),
        );
        for index in 0..photon_count {
            let phase_offset = index as f32 / photon_count as f32 + row as f32 * 0.19;
            let phase = if controls.animation_running {
                (time_seconds as f32 * 0.38 + phase_offset).fract()
            } else {
                phase_offset.fract()
            };
            let x = beam_start_x + phase * (beam_end_x - beam_start_x);
            painter.circle_filled(Pos2::new(x, y), 3.6, photon_color);
            painter.circle_stroke(
                Pos2::new(x, y),
                6.2,
                Stroke::new(
                    0.8_f32,
                    Color32::from_rgba_unmultiplied(
                        photon_color.r(),
                        photon_color.g(),
                        photon_color.b(),
                        95,
                    ),
                ),
            );
        }
    }

    // Placas esquemáticas del cátodo y el ánodo.
    let plate_top = mid_y - electrode_half_height;
    let plate_bottom = mid_y + electrode_half_height;
    let cathode_rect = Rect::from_min_max(
        Pos2::new(cathode_x - 7.0, plate_top),
        Pos2::new(cathode_x + 7.0, plate_bottom),
    );
    let anode_rect = Rect::from_min_max(
        Pos2::new(anode_x - 7.0, plate_top),
        Pos2::new(anode_x + 7.0, plate_bottom),
    );
    painter.rect_filled(
        cathode_rect,
        egui::CornerRadius::same(5),
        Color32::from_rgb(255, 183, 92),
    );
    painter.rect_stroke(
        cathode_rect,
        egui::CornerRadius::same(5),
        Stroke::new(1.0_f32, Color32::from_rgb(255, 218, 154)),
        StrokeKind::Inside,
    );
    painter.rect_filled(
        anode_rect,
        egui::CornerRadius::same(5),
        Color32::from_rgb(88, 164, 196),
    );
    painter.rect_stroke(
        anode_rect,
        egui::CornerRadius::same(5),
        Stroke::new(1.0_f32, Color32::from_rgb(153, 220, 236)),
        StrokeKind::Inside,
    );

    // Las flechas se orientan por el control visual; no son un cálculo de E.
    if controls.applied_voltage_v.abs() > 0.15 {
        let direction = controls.applied_voltage_v.signum();
        for factor in [-0.55, 0.0, 0.55] {
            let y = mid_y + electrode_half_height * factor;
            let center = (cathode_x + anode_x) * 0.5;
            let origin = if direction > 0.0 {
                Pos2::new(center - 18.0, y)
            } else {
                Pos2::new(center + 18.0, y)
            };
            painter.arrow(
                origin,
                Vec2::new(direction * 40.0, 0.0),
                Stroke::new(1.5_f32, theme::VIOLET),
            );
        }
    }

    // Si hay un resultado físico externo, respétalo. Si aún no se conectó,
    // la UI puede mostrar la animación de ejemplo para desarrollar el diseño.
    let show_electrons = readout.emission_possible.unwrap_or(controls.demo_electrons);
    if show_electrons && gap_width > 30.0 {
        for index in 0..4 {
            let offset = index as f32 * 0.23;
            let phase = if controls.animation_running {
                (time_seconds as f32 * 0.24 + offset).fract()
            } else {
                0.48
            };
            let x = cathode_x + 15.0 + phase * (gap_width - 30.0);
            let lane = (index as f32 - 1.5) * 13.0;
            let y = mid_y + lane + (phase * std::f32::consts::TAU).sin() * 4.0;
            painter.line_segment(
                [Pos2::new(cathode_x + 12.0, mid_y + lane), Pos2::new(x, y)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(65, 211, 151, 75)),
            );
            painter.circle_filled(Pos2::new(x, y), 4.3, theme::GREEN);
            painter.circle_stroke(Pos2::new(x, y), 7.2, Stroke::new(1.0_f32, theme::GREEN));
        }
    }
}

fn draw_labels(
    painter: &egui::Painter,
    rect: Rect,
    controls: &LabControls,
    readout: &PhysicsReadout,
) {
    let cathode_x = rect.left() + rect.width() * 0.32;
    let anode_x = rect.left() + rect.width() * 0.72;
    let label_y = rect.bottom() - 21.0;
    let top_y = rect.top() + 20.0;

    painter.text(
        Pos2::new(cathode_x, label_y),
        Align2::CENTER_CENTER,
        format!("CÁTODO · {}", controls.material.symbol()),
        FontId::proportional(10.0),
        Color32::from_rgb(255, 199, 124),
    );
    painter.text(
        Pos2::new(anode_x, label_y),
        Align2::CENTER_CENTER,
        "ÁNODO",
        FontId::proportional(10.0),
        Color32::from_rgb(153, 220, 236),
    );

    let incoming = format!("λ = {:.0} nm", controls.wavelength_nm);
    painter.text(
        Pos2::new(rect.left() + 16.0, top_y),
        Align2::LEFT_CENTER,
        incoming,
        FontId::proportional(10.0),
        theme::MUTED,
    );

    let result_text = match readout.emission_possible {
        Some(true) => "EMISIÓN · DATO FÍSICO",
        Some(false) => "SIN EMISIÓN · DATO FÍSICO",
        None if controls.demo_electrons => "MODO DEMO · ELECTRONES ILUSTRATIVOS",
        None => "ESPERANDO DATOS DEL MODELO",
    };
    let result_color = match readout.emission_possible {
        Some(true) => theme::GREEN,
        Some(false) => theme::AMBER,
        None => theme::CYAN,
    };
    painter.text(
        Pos2::new(rect.right() - 15.0, top_y),
        Align2::RIGHT_CENTER,
        result_text,
        FontId::proportional(9.0),
        result_color,
    );

    if controls.applied_voltage_v.abs() > 0.15 {
        painter.text(
            Pos2::new((cathode_x + anode_x) * 0.5, rect.top() + 40.0),
            Align2::CENTER_CENTER,
            format!("V visual = {:+.1} V", controls.applied_voltage_v),
            FontId::proportional(9.0),
            theme::VIOLET,
        );
    }
}

pub fn draw_curve(ui: &mut egui::Ui, readout: &PhysicsReadout, controls: &LabControls) {
    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(14))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("ENERGÍA CINÉTICA MÁXIMA")
                            .size(10.0)
                            .strong()
                            .color(theme::MUTED),
                    );
                    ui.label(
                        RichText::new("Curva por longitud de onda")
                            .small()
                            .color(theme::TEXT),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("DATOS DE FÍSICA · OPCIONALES")
                            .size(9.0)
                            .color(theme::MUTED),
                    );
                });
            });
            ui.add_space(8.0);
            let desired_size = Vec2::new(ui.available_width().max(280.0), 176.0);
            let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
            let rect = response.rect;
            painter.rect_filled(
                rect,
                egui::CornerRadius::same(10),
                Color32::from_rgb(11, 20, 37),
            );
            painter.rect_stroke(
                rect,
                egui::CornerRadius::same(10),
                Stroke::new(1.0_f32, theme::BORDER),
                StrokeKind::Inside,
            );
            draw_chart_axes(&painter, rect);
            if readout.kinetic_energy_curve.len() >= 2 {
                draw_curve_points(&painter, rect, &readout.kinetic_energy_curve, controls);
            } else {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "La gráfica se dibujará al recibir puntos del modelo físico",
                    FontId::proportional(11.0),
                    theme::MUTED,
                );
            }
        });
}

fn draw_chart_axes(painter: &egui::Painter, rect: Rect) {
    let plot = plot_rect(rect);
    for fraction in [0.0_f32, 0.33, 0.66, 1.0] {
        let y = plot.bottom() - fraction * plot.height();
        painter.line_segment(
            [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
            Stroke::new(0.7_f32, Color32::from_rgba_unmultiplied(91, 117, 148, 65)),
        );
    }
    painter.line_segment(
        [
            Pos2::new(plot.left(), plot.top()),
            Pos2::new(plot.left(), plot.bottom()),
        ],
        Stroke::new(1.0_f32, theme::MUTED),
    );
    painter.line_segment(
        [
            Pos2::new(plot.left(), plot.bottom()),
            Pos2::new(plot.right(), plot.bottom()),
        ],
        Stroke::new(1.0_f32, theme::MUTED),
    );
    painter.text(
        Pos2::new(plot.left(), rect.top() + 10.0),
        Align2::LEFT_CENTER,
        "Kmax (eV)",
        FontId::proportional(9.0),
        theme::MUTED,
    );
    painter.text(
        Pos2::new(plot.right(), rect.bottom() - 8.0),
        Align2::RIGHT_CENTER,
        "λ (nm)",
        FontId::proportional(9.0),
        theme::MUTED,
    );
}

fn draw_curve_points(
    painter: &egui::Painter,
    rect: Rect,
    points: &[CurvePoint],
    controls: &LabControls,
) {
    let plot = plot_rect(rect);
    let min_x = points
        .iter()
        .map(|point| point.wavelength_nm)
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.wavelength_nm)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = points
        .iter()
        .map(|point| point.max_kinetic_energy_ev)
        .fold(0.0_f64, f64::max)
        .max(f64::EPSILON);
    let x_span = (max_x - min_x).max(f64::EPSILON);

    let to_screen = |point: &CurvePoint| {
        let x_fraction = ((point.wavelength_nm - min_x) / x_span) as f32;
        let y_fraction = (point.max_kinetic_energy_ev.max(0.0) / max_y) as f32;
        Pos2::new(
            plot.left() + x_fraction * plot.width(),
            plot.bottom() - y_fraction * plot.height(),
        )
    };

    for pair in points.windows(2) {
        painter.line_segment(
            [to_screen(&pair[0]), to_screen(&pair[1])],
            Stroke::new(2.2_f32, theme::CYAN),
        );
    }
    for point in points.iter().step_by((points.len() / 14).max(1)) {
        painter.circle_filled(to_screen(point), 2.5, theme::CYAN);
    }

    let selected_x = controls.wavelength_nm as f64;
    if selected_x >= min_x && selected_x <= max_x {
        let nearest = points.iter().min_by(|a, b| {
            (a.wavelength_nm - selected_x)
                .abs()
                .total_cmp(&(b.wavelength_nm - selected_x).abs())
        });
        if let Some(point) = nearest {
            painter.circle_filled(to_screen(point), 5.0, theme::AMBER);
            painter.circle_stroke(to_screen(point), 8.0, Stroke::new(1.2_f32, theme::AMBER));
        }
    }
}

fn plot_rect(rect: Rect) -> Rect {
    Rect::from_min_max(
        Pos2::new(rect.left() + 42.0, rect.top() + 22.0),
        Pos2::new(rect.right() - 16.0, rect.bottom() - 25.0),
    )
}

fn legend_dot(ui: &mut egui::Ui, color: Color32, label: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("●").color(color));
        ui.label(RichText::new(label).small().color(theme::MUTED));
    });
}

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
