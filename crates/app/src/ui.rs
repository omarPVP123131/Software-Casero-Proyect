//! Paneles, controles y lecturas de la interfaz.

use egui_macroquad::egui::{self, Align, Color32, Layout, RichText, Stroke};

use crate::state::{LabControls, MaterialChoice, PhysicsReadout};
use crate::{theme, visualization};

pub fn draw(
    ctx: &egui::Context,
    controls: &mut LabControls,
    readout: &PhysicsReadout,
    time_seconds: f64,
) {
    draw_top_bar(ctx);
    draw_footer(ctx);
    draw_controls(ctx, controls);
    draw_readouts(ctx, readout);
    draw_workspace(ctx, controls, readout, time_seconds);
}

fn draw_top_bar(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_bar")
        .exact_height(66.0)
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL)
                .stroke(Stroke::new(1.0_f32, theme::BORDER))
                .inner_margin(egui::Margin::symmetric(18, 10)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Φ").size(30.0).strong().color(theme::CYAN));
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("PHOTOLAB")
                            .size(17.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new("LABORATORIO DEL EFECTO FOTOELÉCTRICO")
                            .size(9.0)
                            .strong()
                            .color(theme::MUTED),
                    );
                });
                ui.add_space(18.0);
                ui.separator();
                ui.label(RichText::new("UAEMéx · Valle de Chalco").color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    status_pill(ui, "UI PROTOTIPO", theme::VIOLET);
                    ui.add_space(8.0);
                    ui.label(RichText::new("v0.1 · GRÁFICA").small().color(theme::MUTED));
                });
            });
        });
}

fn draw_footer(ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("footer")
        .exact_height(30.0)
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL)
                .stroke(Stroke::new(1.0_f32, theme::BORDER))
                .inner_margin(egui::Margin::symmetric(14, 5)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("●").color(theme::AMBER));
                ui.label(
                    RichText::new("Capa física aún no conectada · valores de salida en espera")
                        .small()
                        .color(theme::MUTED),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("Macroquad + egui")
                            .small()
                            .color(theme::MUTED),
                    );
                    ui.add_space(12.0);
                    ui.label(RichText::new("ES").small().strong().color(theme::CYAN));
                });
            });
        });
}

fn draw_controls(ctx: &egui::Context, controls: &mut LabControls) {
    egui::SidePanel::left("controls_panel")
        .exact_width(280.0)
        .resizable(false)
        .frame(panel_frame())
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                section_heading(ui, "01", "PARÁMETROS DEL EXPERIMENTO");
                ui.add_space(7.0);
                ui.label(RichText::new("Material del cátodo").color(theme::MUTED));
                egui::ComboBox::from_id_salt("material_choice")
                    .selected_text(format!("{}  ·  {}", controls.material.symbol(), controls.material.name()))
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for material in MaterialChoice::ALL {
                            ui.selectable_value(
                                &mut controls.material,
                                material,
                                format!("{}  ·  {}", material.symbol(), material.name()),
                            );
                        }
                    });

                ui.add_space(15.0);
                ui.label(RichText::new("Longitud de onda · visual").color(theme::MUTED));
                ui.add(
                    egui::Slider::new(&mut controls.wavelength_nm, 180.0..=900.0)
                        .suffix(" nm")
                        .show_value(true),
                );
                ui.horizontal(|ui| {
                    ui.label(RichText::new("180 nm").small().color(theme::MUTED));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("900 nm").small().color(theme::MUTED));
                    });
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Espectro seleccionado").small().color(theme::MUTED));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(spectrum_name(controls.wavelength_nm))
                                .small()
                                .color(visualization::spectrum_color(controls.wavelength_nm)),
                        );
                    });
                });

                ui.add_space(15.0);
                ui.label(RichText::new("Intensidad · visual").color(theme::MUTED));
                ui.add(
                    egui::Slider::new(&mut controls.intensity_percent, 0.0..=100.0)
                        .suffix(" %")
                        .show_value(true),
                );
                ui.small("Ajusta la densidad de fotones dibujados; no calcula corriente.");

                ui.add_space(15.0);
                ui.label(RichText::new("Voltaje aplicado · visual").color(theme::MUTED));
                ui.add(
                    egui::Slider::new(&mut controls.applied_voltage_v, -5.0..=5.0)
                        .suffix(" V")
                        .show_value(true),
                );
                ui.small("Control preparado para conectar el modelo de celda; no calcula el campo.");

                ui.add_space(18.0);
                ui.separator();
                section_heading(ui, "02", "REPRODUCCIÓN VISUAL");
                ui.checkbox(&mut controls.animation_running, "Animar partículas");
                ui.checkbox(&mut controls.demo_electrons, "Mostrar electrones de ejemplo");
                ui.small("La animación es ilustrativa hasta recibir trayectorias físicas.");
                ui.add_space(12.0);

                if ui.button(RichText::new("↺  Restablecer controles").color(theme::TEXT)).clicked() {
                    *controls = LabControls::default();
                }
                ui.add_space(18.0);
                note_card(
                    ui,
                    "INTEGRACIÓN",
                    "Los controles están separados de la física. La app puede recibir resultados sin cambiar el diseño de esta pantalla.",
                    theme::CYAN,
                );
            });
        });
}

fn draw_readouts(ctx: &egui::Context, readout: &PhysicsReadout) {
    egui::SidePanel::right("readouts_panel")
        .exact_width(282.0)
        .resizable(false)
        .frame(panel_frame())
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                section_heading(ui, "03", "LECTURAS DEL MODELO");
                ui.add_space(9.0);
                match readout.emission_possible {
                    Some(true) => status_pill(ui, "EMISIÓN", theme::GREEN),
                    Some(false) => status_pill(ui, "SIN EMISIÓN", theme::AMBER),
                    None => status_pill(ui, "ESPERANDO FÍSICA", theme::VIOLET),
                }
                ui.add_space(13.0);

                metric_card(
                    ui,
                    "Frecuencia de la luz",
                    format_scientific(readout.frequency_hz),
                    "Hz",
                    theme::BLUE,
                );
                metric_card(
                    ui,
                    "Energía del fotón",
                    format_optional(readout.photon_energy_ev, 3),
                    "eV",
                    theme::CYAN,
                );
                metric_card(
                    ui,
                    "Función de trabajo",
                    format_optional(readout.work_function_ev, 3),
                    "eV",
                    theme::VIOLET,
                );
                metric_card(
                    ui,
                    "Energía cinética máx.",
                    format_optional(readout.max_kinetic_energy_ev, 3),
                    "eV",
                    theme::GREEN,
                );
                metric_card(
                    ui,
                    "Potencial de frenado",
                    format_optional(readout.stopping_potential_v, 3),
                    "V",
                    theme::AMBER,
                );
                metric_card(
                    ui,
                    "Frecuencia umbral",
                    format_scientific(readout.threshold_frequency_hz),
                    "Hz",
                    theme::BLUE,
                );
                metric_card(
                    ui,
                    "Longitud de onda umbral",
                    format_optional(readout.threshold_wavelength_nm, 1),
                    "nm",
                    theme::VIOLET,
                );
                metric_card(
                    ui,
                    "Flujo de fotones",
                    format_scientific(readout.photon_flux_density_per_m2_s),
                    "fotones / m²·s",
                    theme::CYAN,
                );

                ui.add_space(5.0);
                ui.separator();
                ui.label(
                    RichText::new("ESTADO DE CONEXIÓN")
                        .size(10.0)
                        .strong()
                        .color(theme::MUTED),
                );
                ui.label(
                    RichText::new("Los guiones se reemplazarán al conectar el módulo físico.")
                        .small()
                        .color(theme::MUTED),
                );
            });
        });
}

fn draw_workspace(
    ctx: &egui::Context,
    controls: &LabControls,
    readout: &PhysicsReadout,
    time_seconds: f64,
) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme::BACKGROUND)
                .inner_margin(14.0),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(RichText::new("Vista experimental").color(theme::TEXT));
                        ui.label(
                            RichText::new("Celda fotoeléctrica · esquema interactivo")
                                .small()
                                .color(theme::MUTED),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        status_pill(ui, "ESQUEMA 2D", theme::CYAN);
                    });
                });
                ui.add_space(12.0);
                visualization::draw_cell(ui, controls, readout, time_seconds);
                ui.add_space(12.0);
                visualization::draw_curve(ui, readout, controls);
            });
        });
}

fn section_heading(ui: &mut egui::Ui, number: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(number).small().strong().color(theme::CYAN));
        ui.label(RichText::new(title).size(10.0).strong().color(theme::MUTED));
    });
}

fn status_pill(ui: &mut egui::Ui, text: &str, accent: Color32) {
    egui::Frame::new()
        .fill(accent.gamma_multiply(0.14))
        .stroke(Stroke::new(1.0_f32, accent.gamma_multiply(0.45)))
        .corner_radius(egui::CornerRadius::same(20))
        .inner_margin(egui::Margin::symmetric(9, 5))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(9.0).strong().color(accent));
        });
}

fn metric_card(ui: &mut egui::Ui, label: &str, value: String, unit: &str, accent: Color32) {
    egui::Frame::new()
        .fill(theme::PANEL_RAISED)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("●").size(9.0).color(accent));
                ui.label(RichText::new(label).small().color(theme::MUTED));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new(value).size(20.0).strong().color(theme::TEXT));
                ui.label(RichText::new(unit).small().color(theme::MUTED));
            });
        });
}

fn note_card(ui: &mut egui::Ui, title: &str, body: &str, accent: Color32) {
    egui::Frame::new()
        .fill(theme::PANEL_RAISED)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(10, 9))
        .show(ui, |ui| {
            ui.label(RichText::new(title).size(9.0).strong().color(accent));
            ui.label(RichText::new(body).small().color(theme::MUTED));
        });
}

fn panel_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .inner_margin(egui::Margin::symmetric(14, 14))
}

fn format_optional(value: Option<f64>, decimals: usize) -> String {
    value
        .map(|number| format!("{number:.decimals$}"))
        .unwrap_or_else(|| "—".to_owned())
}

fn format_scientific(value: Option<f64>) -> String {
    value
        .map(|number| format!("{number:.2e}"))
        .unwrap_or_else(|| "—".to_owned())
}

fn spectrum_name(wavelength_nm: f32) -> &'static str {
    match wavelength_nm {
        value if value < 380.0 => "UV",
        value if value < 450.0 => "Violeta / azul",
        value if value < 520.0 => "Azul / verde",
        value if value < 590.0 => "Verde / amarillo",
        value if value < 650.0 => "Naranja",
        _ => "Rojo / IR",
    }
}
