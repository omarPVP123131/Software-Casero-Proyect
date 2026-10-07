//! Composición de paneles, controles, pestañas y diálogos de PhotoLab.

use egui_macroquad::egui::{
    self, Align, Align2, Color32, Layout, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2,
};
use fotoelectrico_engine::layers::{
    cell::layout_geometry,
    electrons::{electron_count, electron_speed_scale, electrons_visible, visual_electrons},
    field::arrow_segments,
    photons::{beam_lane_y, beam_x_bounds},
};
use fotoelectrico_engine::{RenderQuality, Viewport};

use crate::physics_adapter::photoelectron_strength;
use crate::state::{
    electrons_visible_in_scene, AppState, InspectedObject, LabTab, MaterialChoice, PerformanceMode,
    PhysicsReadout, ShortcutKey, ThemeChoice,
};
use crate::theme::{self, Palette};
use crate::visualization;
use crate::UiFrame;

#[derive(Debug, Clone, Copy)]
enum Command {
    TogglePlayback,
    ResetAnimation,
    StepFrame,
    CellTab,
    ChartTab,
    CompareTab,
    LogTab,
    TogglePresentation,
    Preferences,
    Diagnostics,
    ResetControls,
    Undo,
    Redo,
}

const COMMANDS: [(Command, &str); 13] = [
    (Command::TogglePlayback, "Reproducir o pausar animación"),
    (Command::ResetAnimation, "Reiniciar animación"),
    (Command::StepFrame, "Avanzar un cuadro"),
    (Command::CellTab, "Abrir pestaña Celda"),
    (Command::ChartTab, "Abrir pestaña Gráfica"),
    (Command::CompareTab, "Abrir pestaña Comparar"),
    (Command::LogTab, "Abrir pestaña Registro"),
    (Command::TogglePresentation, "Alternar modo presentación"),
    (Command::Preferences, "Abrir preferencias"),
    (Command::Diagnostics, "Alternar diagnóstico"),
    (Command::ResetControls, "Restablecer controles"),
    (Command::Undo, "Deshacer controles"),
    (Command::Redo, "Rehacer controles"),
];

pub fn draw(ctx: &egui::Context, state: &mut AppState) -> UiFrame {
    handle_keyboard(ctx, state);
    state.reconcile_inspector_visibility();
    state.sync_camera_target();
    let before_controls = state.controls;
    let mut skip_control_history = false;
    let palette = theme::palette(state.preferences.theme);
    let coordinate_scale =
        ctx.pixels_per_point() / macroquad::prelude::screen_dpi_scale().max(f32::EPSILON);
    let screen = ctx.screen_rect();
    let docked_width = (if state.preferences.controls_panel_open {
        state.preferences.controls_panel_width
    } else {
        0.0
    }) + (if state.preferences.readouts_panel_open {
        state.preferences.readouts_panel_width
    } else {
        0.0
    });
    let central_width_estimate = screen.width() - docked_width - 48.0;
    let wide_layout = !state.preferences.presentation_mode
        && central_width_estimate >= 640.0
        && screen.height() >= 520.0;
    let compact_top = screen.width() < 1000.0;

    draw_top_bar(ctx, state, palette, wide_layout, compact_top);
    draw_footer(ctx, state, palette);

    if wide_layout {
        if state.inspector.is_none() {
            state.active_drawer = None;
        }
        draw_docked_panels(ctx, state, palette, &mut skip_control_history);
    }

    let mut scene_rect = None;
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(Color32::TRANSPARENT)
                .inner_margin(egui::Margin::same(12)),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    draw_tabs(ui, state, palette);
                    match state.active_tab {
                        LabTab::Cell => {
                            scene_rect =
                                draw_cell_tab(ui, state, palette, coordinate_scale, wide_layout);
                        }
                        LabTab::Chart => draw_chart_tab(ui, state, palette),
                        LabTab::Compare => draw_comparison_tab(ui, state, palette),
                        LabTab::Log => draw_log_tab(ui, state, palette),
                    }
                });
        });

    if (!wide_layout || state.active_drawer.is_some()) && !state.preferences.presentation_mode {
        draw_panel_drawer(ctx, state, palette, &mut skip_control_history);
    }
    draw_preferences_window(ctx, state, palette);
    draw_shortcuts_window(ctx, state, palette);
    let command = draw_command_palette(ctx, state, palette);
    draw_diagnostics_window(ctx, state, palette);

    let mut command_is_undo_redo = false;
    if let Some(command) = command {
        command_is_undo_redo = matches!(command, Command::Undo | Command::Redo);
        apply_command(state, command);
    }
    if !skip_control_history && !command_is_undo_redo {
        state.record_control_change(before_controls);
    }
    // Controles -> motor físico -> lecturas, siempre sincronizado.
    state.refresh_physics();

    UiFrame {
        scene_rect,
        coordinate_scale,
    }
}

fn handle_keyboard(ctx: &egui::Context, state: &mut AppState) {
    let command = ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::K));
    if command {
        state.command_search.clear();
        state.show_command_palette = true;
    }
    let text_has_focus = ctx.wants_keyboard_input();
    if !text_has_focus && ctx.input(|input| input.key_pressed(egui::Key::F1)) {
        state.show_shortcuts = true;
    }
    if state.show_command_palette && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        state.show_command_palette = false;
    }
    if !text_has_focus && ctx.input(|input| input.key_pressed(egui::Key::F11)) {
        state.preferences.presentation_mode = !state.preferences.presentation_mode;
        state.push_history(if state.preferences.presentation_mode {
            "Modo presentación activado"
        } else {
            "Modo presentación desactivado"
        });
    }
    if !text_has_focus && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Z))
    {
        state.undo();
    }
    if !text_has_focus && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Y))
    {
        state.redo();
    }

    let shortcuts = state.preferences.shortcuts;
    if key_pressed(ctx, shortcuts.toggle_play) {
        state.animation_running = !state.animation_running;
        state.push_history(if state.animation_running {
            "Animación reanudada"
        } else {
            "Animación pausada"
        });
    }
    if key_pressed(ctx, shortcuts.reset_animation) {
        state.reset_animation();
    }
    if key_pressed(ctx, shortcuts.step_frame) {
        state.step_frame();
    }
    if key_pressed(ctx, shortcuts.diagnostics) {
        state.show_diagnostics = !state.show_diagnostics;
    }
}

fn key_pressed(ctx: &egui::Context, key: ShortcutKey) -> bool {
    if ctx.wants_keyboard_input() {
        return false;
    }
    let egui_key = match key {
        ShortcutKey::Space => egui::Key::Space,
        ShortcutKey::P => egui::Key::P,
        ShortcutKey::R => egui::Key::R,
        ShortcutKey::N => egui::Key::N,
        ShortcutKey::D => egui::Key::D,
    };
    ctx.input(|input| input.key_pressed(egui_key) && !input.modifiers.ctrl && !input.modifiers.alt)
}

fn draw_top_bar(
    ctx: &egui::Context,
    state: &mut AppState,
    palette: Palette,
    wide_layout: bool,
    compact_top: bool,
) {
    let height = if state.preferences.presentation_mode {
        42.0
    } else if compact_top {
        48.0
    } else {
        58.0
    };
    egui::TopBottomPanel::top("photolab_top_bar")
        .exact_height(height)
        .frame(
            egui::Frame::new()
                .fill(palette.panel)
                .stroke(Stroke::new(1.0_f32, palette.border))
                .inner_margin(egui::Margin::symmetric(14, 6)),
        )
        .show(ctx, |ui| {
            let short_labels = ui.available_width() < 720.0;
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new("◈").size(21.0).strong().color(palette.cyan));
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Laboratorio de efecto fotoeléctrico")
                            .size(16.0)
                            .strong()
                            .color(palette.text),
                    );
                    if !compact_top && !state.preferences.presentation_mode {
                        ui.label(
                            RichText::new("Software Casero")
                                .size(8.0)
                                .color(palette.muted),
                        );
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if state.preferences.presentation_mode {
                        if ui.button("Salir de presentación · F11").clicked() {
                            state.preferences.presentation_mode = false;
                            state.preferences.layout_preset = crate::state::LayoutPreset::Workbench;
                            state.preferences.controls_panel_open = true;
                            state.preferences.readouts_panel_open = true;
                            state.push_history("Modo presentación desactivado");
                        }
                        return;
                    }

                    draw_panel_quick_button(
                        ui,
                        state,
                        wide_layout,
                        short_labels,
                        crate::state::DockPanel::Controls,
                    );
                    draw_panel_quick_button(
                        ui,
                        state,
                        wide_layout,
                        short_labels,
                        crate::state::DockPanel::Readouts,
                    );

                    if compact_top {
                        ui.menu_button(if short_labels { "Más" } else { "Menú" }, |ui| {
                            if ui.button("Paleta de comandos · Ctrl+K").clicked() {
                                state.command_search.clear();
                                state.show_command_palette = true;
                                ui.close_menu();
                            }
                            if ui.button("Preferencias").clicked() {
                                state.show_preferences = true;
                                ui.close_menu();
                            }
                            if ui.button("F1  Ayuda y atajos").clicked() {
                                state.show_shortcuts = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui
                                .button(if state.preferences.simple_mode {
                                    "Modo avanzado"
                                } else {
                                    "Modo simple"
                                })
                                .clicked()
                            {
                                state.preferences.simple_mode = !state.preferences.simple_mode;
                                ui.close_menu();
                            }
                            ui.menu_button("Diseño de paneles", |ui| draw_layout_menu(ui, state));
                            if ui.button("Presentación · F11").clicked() {
                                state.set_layout_preset(crate::state::LayoutPreset::Presentation);
                                ui.close_menu();
                            }
                        });
                    } else {
                        ui.menu_button("Diseño", |ui| draw_layout_menu(ui, state));
                        if ui.button("Preferencias").clicked() {
                            state.show_preferences = true;
                        }
                        if ui.button("F1 · Ayuda").clicked() {
                            state.show_shortcuts = true;
                        }
                        if ui.button("Ctrl+K · Comandos").clicked() {
                            state.command_search.clear();
                            state.show_command_palette = true;
                        }
                        let mode_label = if state.preferences.simple_mode {
                            "Simple"
                        } else {
                            "Avanzado"
                        };
                        if ui
                            .button(mode_label)
                            .on_hover_text("Oculta o muestra los controles secundarios.")
                            .clicked()
                        {
                            state.preferences.simple_mode = !state.preferences.simple_mode;
                        }
                        if ui
                            .button("F11")
                            .on_hover_text("Modo presentación · F11")
                            .clicked()
                        {
                            state.set_layout_preset(crate::state::LayoutPreset::Presentation);
                        }
                    }
                });
            });
        });
}

fn draw_panel_quick_button(
    ui: &mut egui::Ui,
    state: &mut AppState,
    wide_layout: bool,
    short_labels: bool,
    panel: crate::state::DockPanel,
) {
    let (label, short_label, is_open) = match panel {
        crate::state::DockPanel::Controls => {
            ("Controles", "Cont.", state.preferences.controls_panel_open)
        }
        crate::state::DockPanel::Readouts => {
            ("Lecturas", "Datos", state.preferences.readouts_panel_open)
        }
    };
    if wide_layout {
        if ui.selectable_label(is_open, label).clicked() {
            match panel {
                crate::state::DockPanel::Controls => {
                    state.preferences.controls_panel_open = !is_open
                }
                crate::state::DockPanel::Readouts => {
                    state.preferences.readouts_panel_open = !is_open
                }
            }
        }
    } else if ui
        .button(if short_labels { short_label } else { label })
        .on_hover_text(label)
        .clicked()
    {
        state.active_drawer = if state.active_drawer == Some(panel) {
            None
        } else {
            Some(panel)
        };
    }
}

fn draw_layout_menu(ui: &mut egui::Ui, state: &mut AppState) {
    for (preset, label, detail) in [
        (
            crate::state::LayoutPreset::Workbench,
            "Mesa de trabajo",
            "Celda, controles y lecturas.",
        ),
        (
            crate::state::LayoutPreset::Analysis,
            "Análisis",
            "Gráfica amplia y paneles mínimos.",
        ),
        (
            crate::state::LayoutPreset::Presentation,
            "Presentación",
            "Canvas limpio, sin paneles laterales.",
        ),
    ] {
        if ui
            .selectable_label(state.preferences.layout_preset == preset, label)
            .on_hover_text(detail)
            .clicked()
        {
            state.set_layout_preset(preset);
            ui.close_menu();
        }
    }
}

fn draw_footer(ctx: &egui::Context, state: &AppState, palette: Palette) {
    egui::TopBottomPanel::bottom("photolab_status_bar")
        .exact_height(25.0)
        .frame(
            egui::Frame::new()
                .fill(palette.panel)
                .stroke(Stroke::new(1.0_f32, palette.border))
                .inner_margin(egui::Margin::symmetric(12, 3)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let (status, color) = if state.animation_running {
                    ("ANIMACIÓN ACTIVA", palette.green)
                } else {
                    ("ANIMACIÓN EN PAUSA", palette.amber)
                };
                ui.label(RichText::new(status).strong().size(9.0).color(color));
                ui.separator();
                // El motor siempre está conectado tras el primer refresh: la
                // emisión/colección son estados físicos, no desconexiones.
                ui.label(
                    RichText::new("MOTOR FÍSICO CONECTADO")
                        .size(9.0)
                        .color(palette.green),
                );
                ui.separator();
                let (emission_label, emission_color) = match (
                    state.readout.emission_possible,
                    state.readout.collected_possible,
                ) {
                    (Some(true), Some(true)) => ("EMISIÓN + COLECCIÓN", palette.green),
                    (Some(true), _) => ("EMISIÓN · BLOQUEADA EN ÁNODO", palette.amber),
                    (Some(false), _) => ("SIN EMISIÓN · λ > λ₀", palette.muted),
                    _ => ("CALCULANDO…", palette.amber),
                };
                ui.label(
                    RichText::new(emission_label)
                        .size(9.0)
                        .color(emission_color),
                );
                ui.separator();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("t visual {:.1} s", state.animation_time_seconds))
                            .size(9.0)
                            .color(palette.muted),
                    );
                    if state.show_diagnostics {
                        ui.label(
                            RichText::new(format!(
                                "{} partículas · {:.1} ms",
                                (state.render_stats.photons_drawn
                                    + state.render_stats.electrons_drawn),
                                state.render_stats.frame_time_ms
                            ))
                            .size(9.0)
                            .color(palette.cyan),
                        );
                    }
                });
            });
        });
}

#[derive(Debug, Clone, Copy)]
enum DockSide {
    Left,
    Right,
}

fn draw_docked_panels(
    ctx: &egui::Context,
    state: &mut AppState,
    palette: Palette,
    skip_control_history: &mut bool,
) {
    let controls_side = if state.preferences.controls_on_left {
        DockSide::Left
    } else {
        DockSide::Right
    };
    let readouts_side = if state.preferences.controls_on_left {
        DockSide::Right
    } else {
        DockSide::Left
    };

    if state.preferences.controls_panel_open && state.preferences.readouts_panel_open {
        if state.preferences.controls_on_left {
            draw_controls_docked(ctx, state, palette, controls_side, skip_control_history);
            draw_readouts_docked(ctx, state, palette, readouts_side);
        } else {
            draw_readouts_docked(ctx, state, palette, readouts_side);
            draw_controls_docked(ctx, state, palette, controls_side, skip_control_history);
        }
    } else if state.preferences.controls_panel_open {
        draw_controls_docked(ctx, state, palette, controls_side, skip_control_history);
    } else if state.preferences.readouts_panel_open {
        draw_readouts_docked(ctx, state, palette, readouts_side);
    }
}

fn draw_controls_docked(
    ctx: &egui::Context,
    state: &mut AppState,
    palette: Palette,
    side: DockSide,
    skip_control_history: &mut bool,
) {
    let width = state.preferences.controls_panel_width;
    let mut panel = egui::SidePanel::left("experiment_controls");
    if matches!(side, DockSide::Right) {
        panel = egui::SidePanel::right("experiment_controls");
    }
    let output = panel
        .resizable(true)
        .default_width(width)
        .width_range(230.0..=410.0)
        .frame(side_panel_frame(palette))
        .show(ctx, |ui| {
            draw_control_panel(ui, state, palette, skip_control_history)
        });
    state.preferences.controls_panel_width = output.response.rect.width();
}

fn draw_readouts_docked(
    ctx: &egui::Context,
    state: &mut AppState,
    palette: Palette,
    side: DockSide,
) {
    let width = state.preferences.readouts_panel_width;
    let mut panel = egui::SidePanel::left("experiment_readouts");
    if matches!(side, DockSide::Right) {
        panel = egui::SidePanel::right("experiment_readouts");
    }
    let output = panel
        .resizable(true)
        .default_width(width)
        .width_range(220.0..=390.0)
        .frame(side_panel_frame(palette))
        .show(ctx, |ui| draw_readout_panel(ui, state, palette));
    state.preferences.readouts_panel_width = output.response.rect.width();
}

fn draw_panel_drawer(
    ctx: &egui::Context,
    state: &mut AppState,
    palette: Palette,
    skip_control_history: &mut bool,
) {
    let Some(panel) = state.active_drawer else {
        return;
    };
    let screen = ctx.screen_rect();
    let controls = panel == crate::state::DockPanel::Controls;
    let width = if controls {
        state.preferences.controls_panel_width
    } else {
        state.preferences.readouts_panel_width
    };
    let use_left = if controls {
        state.preferences.controls_on_left
    } else {
        !state.preferences.controls_on_left
    };
    let mut open = true;
    let title = if controls {
        "Controles"
    } else {
        "Lecturas e inspector"
    };
    let mut window = egui::Window::new(title)
        .id(egui::Id::new(if controls {
            "controls-drawer"
        } else {
            "readouts-drawer"
        }))
        .open(&mut open)
        .default_width(width.min(screen.width() - 32.0).max(220.0))
        .resizable(true)
        .collapsible(false)
        .frame(side_panel_frame(palette));
    window = if use_left {
        window.anchor(Align2::LEFT_TOP, Vec2::new(10.0, 70.0))
    } else {
        window.anchor(Align2::RIGHT_TOP, Vec2::new(-10.0, 70.0))
    };
    let output = window.show(ctx, |ui| {
        if controls {
            draw_control_panel(ui, state, palette, skip_control_history);
        } else {
            draw_readout_panel(ui, state, palette);
        }
    });
    if let Some(output) = output {
        let width = output.response.rect.width();
        if controls {
            state.preferences.controls_panel_width = width;
        } else {
            state.preferences.readouts_panel_width = width;
        }
    }
    if !open {
        state.active_drawer = None;
    }
}

fn side_panel_frame(palette: Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .inner_margin(egui::Margin::same(12))
}

fn draw_control_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    palette: Palette,
    skip_control_history: &mut bool,
) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        section_title(ui, "PARÁMETROS DE ENTRADA", "Entradas del motor físico (hf, Φ, intensidad, voltaje).", palette);
        ui.add_space(8.0);

        ui.label(RichText::new("Material del cátodo").strong().color(palette.text));
        material_combo(ui, "cathode_material", &mut state.controls.material);
        ui.label(RichText::new(format!("Φ = {:.2} eV · valor estándar de literatura", state.controls.material.work_function_ev())).size(10.0).color(palette.muted));

        ui.add_space(8.0);
        let color = visualization::spectrum_color(state.controls.wavelength_nm);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Longitud de onda").strong().color(palette.text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(format!("{:.0} nm", state.controls.wavelength_nm)).color(color).strong());
            });
        });
        ui.add(egui::Slider::new(&mut state.controls.wavelength_nm, 180.0..=900.0).show_value(false));
        ui.label(RichText::new("Define f = c/λ y la energía E = hf del motor.").size(10.0).color(palette.muted));

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Intensidad").strong().color(palette.text));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(format!("{:.0}%", state.controls.intensity_percent)).color(palette.amber).strong());
            });
        });
        ui.add(egui::Slider::new(&mut state.controls.intensity_percent, 0.0..=100.0).show_value(false));
        ui.label(RichText::new("100% = 10 mW/cm². Cambia el flujo y nº de electrones, no Kmax.").size(10.0).color(palette.muted));

        if !state.preferences.simple_mode {
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Voltaje aplicado").strong().color(palette.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:+.1} V", state.controls.applied_voltage_v)).color(palette.violet).strong());
                });
            });
            ui.add(egui::Slider::new(&mut state.controls.applied_voltage_v, -5.0..=5.0).show_value(false));
            ui.label(RichText::new("Si V < −V₀ bloquea la colección (sin corriente) sin cambiar Kmax.").size(10.0).color(palette.muted));

            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new("Comparación").strong().color(palette.text));
            material_combo(ui, "comparison_material", &mut state.controls.comparison_material);

            ui.add_space(10.0);
            egui::CollapsingHeader::new("Capas del canvas")
                .default_open(false)
                .show(ui, |ui| draw_layer_controls(ui, state, palette));
        }

        ui.add_space(10.0);
        egui::Frame::new()
            .fill(palette.raised)
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.label(RichText::new("Animación visual").strong().color(palette.text));
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    let label = if state.animation_running { "Pausar" } else { "Reproducir" };
                    if ui.button(label).clicked() {
                        state.animation_running = !state.animation_running;
                        state.push_history(if state.animation_running { "Animación reanudada" } else { "Animación pausada" });
                    }
                    if ui.button("Paso").on_hover_text("Avanza la animación en un cuadro visual.").clicked() {
                        state.step_frame();
                    }
                    if ui.button("Reiniciar").clicked() {
                        state.reset_animation();
                    }
                });
                ui.add(egui::Slider::new(&mut state.animation_speed, 0.25..=2.5).text("Velocidad"));
            });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("Deshacer").on_hover_text("Ctrl+Z").clicked() {
                state.undo();
                *skip_control_history = true;
            }
            if ui.button("Rehacer").on_hover_text("Ctrl+Y").clicked() {
                state.redo();
                *skip_control_history = true;
            }
            if ui.button("Restablecer").on_hover_text("Vuelve a los valores de control iniciales.").clicked() {
                state.controls = Default::default();
                state.refresh_physics();
            }
        });
        ui.label(RichText::new("Trayectorias dibujadas como esquema; los valores numéricos y la curva los calcula el motor físico.").size(10.0).color(palette.muted));
    });
}

fn section_title(ui: &mut egui::Ui, title: &str, detail: &str, palette: Palette) {
    ui.label(RichText::new(title).size(10.0).strong().color(palette.cyan));
    ui.label(RichText::new(detail).small().color(palette.muted));
}

fn material_combo(ui: &mut egui::Ui, id: &'static str, material: &mut MaterialChoice) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(format!("{} ({})", material.name(), material.symbol()))
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for choice in MaterialChoice::ALL {
                ui.selectable_value(
                    material,
                    choice,
                    format!("{} ({})", choice.name(), choice.symbol()),
                );
            }
        });
    ui.add_space(2.0);
}

fn draw_layer_controls(ui: &mut egui::Ui, state: &mut AppState, palette: Palette) {
    let layers = &mut state.preferences.layers;
    ui.checkbox(&mut layers.photons, "Fotones");
    ui.checkbox(&mut layers.electrons, "Electrones ilustrativos");
    ui.checkbox(
        &mut layers.demo_electrons,
        "Mostrar electrones de demostración",
    );
    ui.checkbox(&mut layers.arrows, "Flechas de polaridad");
    ui.checkbox(&mut layers.labels, "Etiquetas interactivas");
    ui.checkbox(&mut layers.trails, "Estelas de movimiento");
    ui.checkbox(&mut layers.grid, "Cuadrícula");
    ui.checkbox(&mut layers.ruler, "Regla visual");
    ui.checkbox(&mut layers.electrode_texture, "Textura de electrodos");
    ui.label(
        RichText::new("Estos interruptores solo cambian capas de dibujo.")
            .size(10.0)
            .color(palette.muted),
    );
}

fn draw_readout_panel(ui: &mut egui::Ui, state: &mut AppState, palette: Palette) {
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        section_title(ui, "LECTURAS DEL EXPERIMENTO", "Calculadas por fotoelectrico-physics (Kmax = hf − Φ).", palette);
        ui.add_space(8.0);

        let emission_color = match state.readout.emission_possible {
            Some(true) => palette.green,
            Some(false) => palette.red,
            None => palette.amber,
        };
        let collection_color = match state.readout.collected_possible {
            Some(true) => palette.green,
            Some(false) => palette.red,
            None => palette.amber,
        };
        metric_row(ui, "Emisión (hf > Φ)", state.readout.emission_possible.map(|value| if value { "Sí" } else { "No" }.to_owned()), emission_color, "hf supera la función de trabajo del material.", palette);
        metric_row(ui, "Colección en ánodo", state.readout.collected_possible.map(|value| if value { "Sí · hay corriente" } else { "No · bloqueada" }.to_owned()), collection_color, "Bloqueada si V aplicado < −V₀ o intensidad ≈ 0.", palette);
        metric_row(ui, "Frecuencia", format_option(state.readout.frequency_hz, " Hz", 3), palette.cyan, "f = c/λ calculada por el motor.", palette);
        metric_row(ui, "Energía del fotón", format_option(state.readout.photon_energy_ev, " eV", 4), palette.blue, "E = hf.", palette);
        metric_row(ui, "Función de trabajo", format_option(state.readout.work_function_ev, " eV", 4), palette.violet, "Φ del material (literatura).", palette);
        metric_row(ui, "Frecuencia umbral", format_option(state.readout.threshold_frequency_hz, " Hz", 3), palette.amber, "f₀ = Φ/h.", palette);
        metric_row(ui, "Longitud de onda umbral", format_option(state.readout.threshold_wavelength_nm, " nm", 2), palette.amber, "λ₀ = c/f₀.", palette);
        metric_row(ui, "Energía cinética máxima", format_option(state.readout.max_kinetic_energy_ev, " eV", 4), palette.green, "Kmax = hf − Φ (0 si no hay emisión).", palette);
        metric_row(ui, "Potencial de frenado", format_option(state.readout.stopping_potential_v, " V", 4), palette.blue, "V₀ = Kmax/e.", palette);
        metric_row(ui, "Flujo de fotones", format_option(state.readout.photon_flux_density_per_m2_s, " m^-2 s^-1", 3), palette.cyan, "Flujo = irradiancia/E_fotón. 100% = 10 mW/cm².", palette);
        metric_row(ui, "Velocidad máx. electrón", format_option(state.readout.electron_max_speed_m_s, " m/s", 3), palette.green, "v = sqrt(2·Kmax/mₑ). Guía la animación.", palette);

        // Explicación del porqué cuando no hay electrones: el motor sigue
        // conectado; es física (umbral o frenado), no una falla.
        let why_no_electrons = match (
            state.readout.emission_possible,
            state.readout.collected_possible,
        ) {
            (Some(false), _) => Some(
                "Sin electrones porque λ > λ₀: el fotón no supera Φ. \
                 El haz sigue incidiendo (sube intensidad o baja λ para emitir).",
            ),
            (Some(true), Some(false)) if state.controls.intensity_percent <= 0.1 => Some(
                "Sin electrones porque la intensidad ≈ 0: no llegan fotones. Sube la intensidad.",
            ),
            (Some(true), Some(false)) => Some(
                "Hay emisión pero no colección: V aplicado < −V₀ (frenado). \
                 Haz V menos negativo para recuperar la corriente; Kmax no cambia.",
            ),
            _ => None,
        };
        if let Some(explanation) = why_no_electrons {
            ui.add_space(4.0);
            egui::Frame::new()
                .fill(palette.raised)
                .stroke(Stroke::new(1.0_f32, palette.amber))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.label(RichText::new("Motor conectado · sin electrones por física").strong().color(palette.amber));
                    ui.add_space(4.0);
                    ui.label(RichText::new(explanation).small().color(palette.text));
                });
        }

        if let Some(selected) = state.inspector {
            ui.add_space(14.0);
            egui::Frame::new()
                .fill(palette.raised)
                .stroke(Stroke::new(1.0_f32, palette.cyan))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Inspector").strong().color(palette.cyan));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("x").clicked() {
                                state.close_inspector();
                            }
                        });
                    });
                    ui.label(RichText::new(selected.label()).strong().color(palette.text));
                    ui.label(RichText::new(format!("Referencia: {}", selected.screen_hint())).small().color(palette.muted));
                    ui.label(RichText::new("Esquema visual; los valores numéricos los calcula el motor físico.").small().color(palette.muted));
                });
        }
    });
}

fn metric_row(
    ui: &mut egui::Ui,
    label: &str,
    value: Option<String>,
    color: Color32,
    hint: &str,
    palette: Palette,
) {
    egui::Frame::new()
        .fill(palette.raised)
        .stroke(Stroke::new(0.7_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(9, 7))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).small().color(palette.muted));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(value.unwrap_or_else(|| "—".to_owned()))
                            .strong()
                            .color(color),
                    );
                });
            });
        })
        .response
        .on_hover_text(hint);
    ui.add_space(5.0);
}

fn format_option(value: Option<f64>, suffix: &str, decimals: usize) -> Option<String> {
    value.map(|number| format!("{number:.decimals$}{suffix}"))
}

fn draw_tabs(ui: &mut egui::Ui, state: &mut AppState, palette: Palette) {
    egui::Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (tab, label) in [
                    (LabTab::Cell, "Celda"),
                    (LabTab::Chart, "Gráfica"),
                    (LabTab::Compare, "Comparar"),
                    (LabTab::Log, "Registro"),
                ] {
                    let selected = state.active_tab == tab;
                    let color = if selected {
                        palette.cyan
                    } else {
                        palette.muted
                    };
                    if ui
                        .selectable_label(selected, RichText::new(label).color(color).strong())
                        .clicked()
                        && !selected
                    {
                        state.active_tab = tab;
                        state.push_history(format!("Pestaña abierta: {label}"));
                    }
                }
            });
        });
    ui.add_space(10.0);
}

fn draw_cell_tab(
    ui: &mut egui::Ui,
    state: &mut AppState,
    palette: Palette,
    coordinate_scale: f32,
    wide_layout: bool,
) -> Option<Rect> {
    let _ = ui.label(
        RichText::new("Espectro electromagnético visible en la escena")
            .small()
            .color(palette.muted),
    );
    visualization::draw_spectrum_bar(ui, state.controls.wavelength_nm, palette);
    ui.add_space(2.0);

    egui::Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| draw_canvas_toolbar(ui, state, palette));
    ui.add_space(8.0);

    let desired_height = ui.available_height().clamp(300.0, 700.0);
    let (canvas_response, painter) = ui.allocate_painter(
        Vec2::new(ui.available_width().max(300.0), desired_height),
        Sense::click(),
    );
    painter.rect_stroke(
        canvas_response.rect,
        egui::CornerRadius::same(10),
        Stroke::new(1.0_f32, palette.border),
        StrokeKind::Inside,
    );
    draw_scene_hotspots(
        ui,
        &canvas_response,
        state,
        palette,
        coordinate_scale,
        wide_layout,
    );
    Some(canvas_response.rect)
}

fn draw_canvas_toolbar(ui: &mut egui::Ui, state: &mut AppState, palette: Palette) {
    let compact_toolbar = ui.available_width() < 820.0;
    ui.horizontal_wrapped(|ui| {
        let play_text = if state.animation_running {
            "Pausar"
        } else {
            "Reproducir"
        };
        if ui.button(play_text).clicked() {
            state.animation_running = !state.animation_running;
            state.push_history(if state.animation_running {
                "Animación reanudada"
            } else {
                "Animación pausada"
            });
        }
        if ui
            .button("Paso")
            .on_hover_text("Avanza un cuadro visual.")
            .clicked()
        {
            state.step_frame();
        }
        if ui
            .button("Reiniciar")
            .on_hover_text("Restablecer animación.")
            .clicked()
        {
            state.reset_animation();
        }

        if compact_toolbar {
            ui.menu_button("Más opciones", |ui| {
                ui.add(egui::Slider::new(&mut state.animation_speed, 0.25..=2.5).text("Velocidad"));
                ui.add(
                    egui::Slider::new(&mut state.preferences.zoom, 0.7..=1.8)
                        .text("Zoom de escena"),
                );
                if !state.preferences.simple_mode {
                    ui.separator();
                    ui.checkbox(&mut state.preferences.layers.photons, "Fotones");
                    ui.checkbox(&mut state.preferences.layers.electrons, "Electrones");
                    ui.checkbox(&mut state.preferences.layers.arrows, "Flechas");
                    ui.checkbox(&mut state.preferences.layers.labels, "Etiquetas");
                    ui.checkbox(&mut state.preferences.layers.grid, "Cuadrícula");
                    ui.checkbox(&mut state.preferences.layers.ruler, "Regla");
                    ui.checkbox(&mut state.preferences.layers.trails, "Estelas");
                    ui.checkbox(
                        &mut state.preferences.layers.electrode_texture,
                        "Textura de electrodos",
                    );
                }
                ui.checkbox(
                    &mut state.preferences.show_hitboxes,
                    "Hitboxes visibles (depuración)",
                );
            });
        } else {
            ui.add(egui::Slider::new(&mut state.animation_speed, 0.25..=2.5).text("Velocidad"));
            ui.add(
                egui::Slider::new(&mut state.preferences.zoom, 0.7..=1.8)
                    .text("Zoom")
                    .show_value(false),
            );
            ui.label(
                RichText::new(format!("{:.0}%", state.preferences.zoom * 100.0))
                    .small()
                    .color(palette.muted),
            );
        }
    });

    if !compact_toolbar && !state.preferences.simple_mode {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Capas:")
                    .small()
                    .strong()
                    .color(palette.muted),
            );
            ui.checkbox(&mut state.preferences.layers.photons, "Fotones");
            ui.checkbox(&mut state.preferences.layers.electrons, "Electrones");
            ui.checkbox(&mut state.preferences.layers.grid, "Cuadrícula");
            ui.checkbox(&mut state.preferences.layers.ruler, "Regla");
            ui.checkbox(&mut state.preferences.layers.trails, "Estelas");
        });
    }
    if state.preferences.presentation_mode {
        ui.label(
            RichText::new("Presentación · F11 para salir")
                .small()
                .color(palette.amber),
        );
    }
}

fn draw_scene_hotspots(
    ui: &mut egui::Ui,
    response: &egui::Response,
    state: &mut AppState,
    palette: Palette,
    coordinate_scale: f32,
    wide_layout: bool,
) {
    let canvas = response.rect;
    let scale = coordinate_scale.max(f32::EPSILON);
    let viewport = Viewport::new(
        canvas.left() * scale,
        canvas.top() * scale,
        canvas.width() * scale,
        canvas.height() * scale,
    );
    let geometry = layout_geometry(
        viewport,
        state.scene_camera.zoom,
        state.scene_camera.pan_x,
        state.scene_camera.pan_y,
    );

    // La escena no reacciona al hover: el hit-test solo se ejecuta al hacer clic.
    if response.clicked_by(egui::PointerButton::Primary) {
        let selected = response
            .interact_pointer_pos()
            .map(|position| (position.x * scale, position.y * scale))
            .and_then(|point| hit_test_object(state, geometry, point))
            .unwrap_or(InspectedObject::Canvas);
        let focus_point = object_focus_point(state, selected, viewport, geometry);
        let focus_fraction = scene_point_to_base_fraction(
            viewport,
            focus_point,
            state.scene_camera.zoom,
            state.scene_camera.pan_x,
            state.scene_camera.pan_y,
        );
        state.select_object_at(selected, focus_fraction);
        if !wide_layout {
            state.active_drawer = Some(crate::state::DockPanel::Readouts);
        }
    }

    draw_hitbox_debug(ui, canvas, geometry, state, palette, scale);
}

fn object_focus_point(
    state: &AppState,
    object: InspectedObject,
    viewport: Viewport,
    geometry: fotoelectrico_engine::layers::cell::CellGeometry,
) -> (f32, f32) {
    let cell_center = (
        viewport.x + viewport.width * 0.5,
        viewport.y + viewport.height * 0.5,
    );
    match object {
        InspectedObject::Canvas => cell_center,
        InspectedObject::Cathode => (geometry.cathode_x, geometry.mid_y),
        InspectedObject::Anode => (geometry.anode_x, geometry.mid_y),
        InspectedObject::PhotonBeam => {
            let (start_x, end_x) = beam_x_bounds(geometry);
            let lane_y = (0..5).map(|lane| beam_lane_y(geometry, lane)).sum::<f32>() / 5.0;
            ((start_x + end_x) * 0.5, lane_y)
        }
        InspectedObject::ElectronLayer => {
            let electrons = ui_visual_electrons(state, geometry);
            if electrons.is_empty() {
                (
                    (geometry.cathode_x + geometry.anode_x) * 0.5,
                    geometry.mid_y,
                )
            } else {
                let count = electrons.len() as f32;
                (
                    electrons.iter().map(|particle| particle.x).sum::<f32>() / count,
                    electrons.iter().map(|particle| particle.y).sum::<f32>() / count,
                )
            }
        }
        InspectedObject::FieldArrows => {
            let segments = arrow_segments(geometry, state.controls.applied_voltage_v);
            let points: Vec<(f32, f32)> = segments
                .into_iter()
                .flat_map(|segment| {
                    [
                        (segment.start_x, segment.start_y),
                        (segment.end_x, segment.end_y),
                    ]
                })
                .collect();
            if points.is_empty() {
                (
                    (geometry.cathode_x + geometry.anode_x) * 0.5,
                    geometry.mid_y,
                )
            } else {
                let (min_x, max_x) = points.iter().fold(
                    (f32::INFINITY, f32::NEG_INFINITY),
                    |(min_x, max_x), (x, _)| (min_x.min(*x), max_x.max(*x)),
                );
                let (min_y, max_y) = points.iter().fold(
                    (f32::INFINITY, f32::NEG_INFINITY),
                    |(min_y, max_y), (_, y)| (min_y.min(*y), max_y.max(*y)),
                );
                ((min_x + max_x) * 0.5, (min_y + max_y) * 0.5)
            }
        }
    }
}

fn scene_point_to_base_fraction(
    viewport: Viewport,
    point: (f32, f32),
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
) -> (f32, f32) {
    let zoom = zoom.clamp(0.7, 1.8);
    let center_x = viewport.x + viewport.width * 0.5;
    let center_y = viewport.y + viewport.height * 0.5;
    let base_x = center_x + (point.0 - center_x - pan_x.clamp(-0.75, 0.75) * viewport.width) / zoom;
    let base_y =
        center_y + (point.1 - center_y - pan_y.clamp(-0.75, 0.75) * viewport.height) / zoom;
    (
        ((base_x - viewport.x) / viewport.width.max(f32::EPSILON)).clamp(0.0, 1.0),
        ((base_y - viewport.y) / viewport.height.max(f32::EPSILON)).clamp(0.0, 1.0),
    )
}

fn hit_test_object(
    state: &AppState,
    geometry: fotoelectrico_engine::layers::cell::CellGeometry,
    point: (f32, f32),
) -> Option<InspectedObject> {
    let (x, y) = point;
    let electrode_pad = geometry.plate_width * 0.5 + 7.0;
    if y >= geometry.plate_top - 4.0 && y <= geometry.plate_bottom + 4.0 {
        if (x - geometry.cathode_x).abs() <= electrode_pad {
            return Some(InspectedObject::Cathode);
        }
        if (x - geometry.anode_x).abs() <= electrode_pad {
            return Some(InspectedObject::Anode);
        }
    }

    // Prioridad explícita: partículas móviles, flechas y haz; el canvas es fallback.
    let mut candidates: Vec<(u8, f32, InspectedObject)> = Vec::new();

    if state.preferences.layers.electrons && electrons_visible_in_scene(state) {
        for particle in ui_visual_electrons(state, geometry) {
            let distance = ((x - particle.x).powi(2) + (y - particle.y).powi(2)).sqrt();
            if distance <= 13.0 {
                candidates.push((0, distance / 13.0, InspectedObject::ElectronLayer));
            }
        }
    }

    if state.preferences.layers.arrows {
        for segment in arrow_segments(geometry, state.controls.applied_voltage_v) {
            let distance = distance_to_segment(
                point,
                (segment.start_x, segment.start_y),
                (segment.end_x, segment.end_y),
            );
            if distance <= 9.0 {
                candidates.push((1, distance / 9.0, InspectedObject::FieldArrows));
            }
        }
    }

    if state.preferences.layers.photons && state.controls.intensity_percent > 0.1 {
        let (start_x, end_x) = beam_x_bounds(geometry);
        for lane in 0..5 {
            let y_lane = beam_lane_y(geometry, lane);
            let distance = distance_to_segment(point, (start_x, y_lane), (end_x, y_lane));
            if distance <= 9.0 {
                candidates.push((2, distance / 9.0, InspectedObject::PhotonBeam));
            }
        }
    }

    candidates
        .into_iter()
        .min_by(|left, right| left.0.cmp(&right.0).then(left.1.total_cmp(&right.1)))
        .map(|(_, _, object)| object)
}

fn distance_to_segment(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let (px, py) = point;
    let (sx, sy) = start;
    let (ex, ey) = end;
    let dx = ex - sx;
    let dy = ey - sy;
    let length_squared = dx * dx + dy * dy;
    if length_squared <= f32::EPSILON {
        return ((px - sx).powi(2) + (py - sy).powi(2)).sqrt();
    }
    let t = (((px - sx) * dx + (py - sy) * dy) / length_squared).clamp(0.0, 1.0);
    let nearest_x = sx + t * dx;
    let nearest_y = sy + t * dy;
    ((px - nearest_x).powi(2) + (py - nearest_y).powi(2)).sqrt()
}

fn ui_electron_quality(state: &AppState) -> RenderQuality {
    match state.preferences.performance {
        PerformanceMode::Balanced => RenderQuality::Balanced,
        PerformanceMode::Eco => RenderQuality::Performance,
    }
}

/// Mismo muestreo que el renderer: velocidad por Kmax y conteo por
/// intensidad/colección, para que el hit-test coincida con lo dibujado.
fn ui_visual_electrons(
    state: &AppState,
    geometry: fotoelectrico_engine::layers::cell::CellGeometry,
) -> Vec<fotoelectrico_engine::layers::electrons::ElectronVisual> {
    let quality = ui_electron_quality(state);
    let visible = electrons_visible(
        state.readout.collected_possible,
        state.readout.emission_possible,
        state.controls.intensity_percent,
        state.preferences.layers.demo_electrons,
    );
    if !visible {
        return Vec::new();
    }
    let speed = electron_speed_scale(state.readout.max_kinetic_energy_ev);
    let count = electron_count(state.controls.intensity_percent, true, quality);
    // Compatibilidad: si la lectura aún es None (primer frame), muestra demo.
    let count = if count == 0
        && state.readout.emission_possible.is_none()
        && state.preferences.layers.demo_electrons
    {
        match quality {
            RenderQuality::Balanced => 4,
            RenderQuality::Performance => 2,
        }
    } else {
        count
    };
    let _ = photoelectron_strength(
        state.controls.intensity_percent,
        state.readout.collected_possible,
        state.readout.emission_possible,
    );
    visual_electrons(
        geometry,
        state.animation_time_seconds,
        state.animation_running,
        state.animation_speed,
        quality,
        speed,
        count,
    )
}

fn draw_hitbox_debug(
    ui: &egui::Ui,
    canvas: Rect,
    geometry: fotoelectrico_engine::layers::cell::CellGeometry,
    state: &AppState,
    palette: Palette,
    scale: f32,
) {
    if !state.preferences.show_hitboxes {
        return;
    }
    let painter = ui.painter().with_clip_rect(canvas);
    painter.rect_stroke(
        canvas,
        egui::CornerRadius::same(10),
        Stroke::new(1.4_f32, palette.cyan.gamma_multiply(0.7)),
        StrokeKind::Inside,
    );

    let to_ui = |x: f32, y: f32| Pos2::new(x / scale, y / scale);
    let electrode_pad = geometry.plate_width * 0.5 + 7.0;
    for center_x in [geometry.cathode_x, geometry.anode_x] {
        let rect = Rect::from_min_max(
            to_ui(center_x - electrode_pad, geometry.plate_top - 4.0),
            to_ui(center_x + electrode_pad, geometry.plate_bottom + 4.0),
        );
        painter.rect_stroke(
            rect,
            egui::CornerRadius::same(4),
            Stroke::new(1.25_f32, palette.amber.gamma_multiply(0.8)),
            StrokeKind::Inside,
        );
    }

    if state.preferences.layers.photons && state.controls.intensity_percent > 0.1 {
        let (start_x, end_x) = beam_x_bounds(geometry);
        for lane in 0..5 {
            let y = beam_lane_y(geometry, lane);
            let start = to_ui(start_x, y);
            let end = to_ui(end_x, y);
            painter.line_segment(
                [start, end],
                Stroke::new(18.0_f32 / scale, palette.amber.gamma_multiply(0.17)),
            );
            painter.line_segment(
                [start, end],
                Stroke::new(1.0_f32, palette.amber.gamma_multiply(0.85)),
            );
        }
    }

    if state.preferences.layers.electrons && electrons_visible_in_scene(state) {
        for particle in ui_visual_electrons(state, geometry) {
            let center = to_ui(particle.x, particle.y);
            painter.circle_filled(center, 12.5_f32 / scale, palette.green.gamma_multiply(0.16));
            painter.circle_stroke(
                center,
                12.5_f32 / scale,
                Stroke::new(1.1_f32, palette.green.gamma_multiply(0.85)),
            );
        }
    }

    if state.preferences.layers.arrows {
        for segment in arrow_segments(geometry, state.controls.applied_voltage_v) {
            let start = to_ui(segment.start_x, segment.start_y);
            let end = to_ui(segment.end_x, segment.end_y);
            painter.line_segment(
                [start, end],
                Stroke::new(18.0_f32 / scale, palette.violet.gamma_multiply(0.17)),
            );
            painter.line_segment(
                [start, end],
                Stroke::new(1.0_f32, palette.violet.gamma_multiply(0.85)),
            );
        }
    }
}

fn draw_chart_tab(ui: &mut egui::Ui, state: &mut AppState, palette: Palette) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("GRÁFICA INTERACTIVA")
                .size(10.0)
                .strong()
                .color(palette.cyan),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.checkbox(&mut state.chart.show_data_table, "Tabla de datos");
        });
    });
    ui.add_space(6.0);
    visualization::draw_chart(
        ui,
        &state.readout,
        state.controls.wavelength_nm,
        &mut state.chart,
        palette,
    );
    ui.add_space(8.0);
    if state.chart.show_data_table {
        draw_curve_table(ui, &state.readout, palette);
    }
    ui.label(RichText::new("Curva Kmax(λ) calculada por el motor (61 puntos, 180–900 nm). Arrastra para desplazar, rueda para zoom.").small().color(palette.muted));
}

fn draw_curve_table(ui: &mut egui::Ui, readout: &PhysicsReadout, palette: Palette) {
    if readout.kinetic_energy_curve.is_empty() {
        ui.label(
            RichText::new("Sin filas: todavía no se recibieron puntos de datos.")
                .color(palette.muted),
        );
        return;
    }
    egui::Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            egui::Grid::new("curve-data-table")
                .striped(true)
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("Longitud de onda (nm)")
                            .strong()
                            .color(palette.text),
                    );
                    ui.label(RichText::new("Kmax (eV)").strong().color(palette.text));
                    ui.end_row();
                    for point in readout.kinetic_energy_curve.iter().take(100) {
                        ui.label(format!("{:.3}", point.wavelength_nm));
                        ui.label(format!("{:.5}", point.max_kinetic_energy_ev));
                        ui.end_row();
                    }
                });
        });
}

fn draw_comparison_tab(ui: &mut egui::Ui, state: &AppState, palette: Palette) {
    ui.label(
        RichText::new("COMPARACIÓN LADO A LADO")
            .size(10.0)
            .strong()
            .color(palette.cyan),
    );
    ui.label(
        RichText::new("Misma λ e intensidad, distinto Φ. Ambas columnas las calcula el motor.")
            .small()
            .color(palette.muted),
    );
    ui.add_space(10.0);
    ui.columns(2, |columns| {
        draw_comparison_card(
            &mut columns[0],
            state.controls.material,
            &state.readout,
            "MATERIAL ACTUAL",
            palette,
        );
        draw_comparison_card(
            &mut columns[1],
            state.controls.comparison_material,
            &state.comparison_readout,
            "MATERIAL DE COMPARACIÓN",
            palette,
        );
    });
}

fn draw_comparison_card(
    ui: &mut egui::Ui,
    material: MaterialChoice,
    readout: &PhysicsReadout,
    title: &str,
    palette: Palette,
) {
    egui::Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.label(RichText::new(title).size(9.0).strong().color(palette.muted));
            ui.heading(
                RichText::new(format!("{} ({})", material.name(), material.symbol()))
                    .color(palette.text),
            );
            ui.add_space(8.0);
            metric_row(
                ui,
                "Emisión",
                readout
                    .emission_possible
                    .map(|v| if v { "Sí" } else { "No" }.to_owned()),
                palette.green,
                "hf > Φ según el motor.",
                palette,
            );
            metric_row(
                ui,
                "Colección",
                readout
                    .collected_possible
                    .map(|v| if v { "Sí" } else { "No" }.to_owned()),
                palette.green,
                "Bloqueada si V < −V₀.",
                palette,
            );
            metric_row(
                ui,
                "Función de trabajo",
                format_option(readout.work_function_ev, " eV", 4),
                palette.violet,
                "Φ del material.",
                palette,
            );
            metric_row(
                ui,
                "Frecuencia umbral",
                format_option(readout.threshold_frequency_hz, " Hz", 3),
                palette.amber,
                "f₀ = Φ/h.",
                palette,
            );
            metric_row(
                ui,
                "Kmax",
                format_option(readout.max_kinetic_energy_ev, " eV", 4),
                palette.cyan,
                "Kmax = hf − Φ.",
                palette,
            );
            metric_row(
                ui,
                "V₀",
                format_option(readout.stopping_potential_v, " V", 4),
                palette.blue,
                "V₀ = Kmax/e.",
                palette,
            );
            if readout.kinetic_energy_curve.is_empty() {
                ui.label(
                    RichText::new("Curva calculándose…")
                        .small()
                        .color(palette.muted),
                );
            } else {
                ui.label(
                    RichText::new(format!(
                        "{} puntos de curva calculados",
                        readout.kinetic_energy_curve.len()
                    ))
                    .small()
                    .color(palette.cyan),
                );
            }
        });
}

fn draw_log_tab(ui: &mut egui::Ui, state: &mut AppState, palette: Palette) {
    ui.label(
        RichText::new("REGISTRO DE SESIÓN")
            .size(10.0)
            .strong()
            .color(palette.cyan),
    );
    ui.label(
        RichText::new(format!(
            "Tiempo de sesión: {} · {} eventos",
            format_duration(state.session_seconds),
            state.history.len()
        ))
        .small()
        .color(palette.muted),
    );
    ui.add_space(8.0);
    egui::Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0_f32, palette.border))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(ui, |ui| {
                    for entry in state.history.iter().rev() {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format_duration(entry.elapsed_seconds))
                                    .monospace()
                                    .small()
                                    .color(palette.cyan),
                            );
                            ui.label(RichText::new(&entry.text).color(palette.text));
                        });
                    }
                });
        });
    ui.add_space(12.0);
    ui.label(
        RichText::new("Notas del operador")
            .strong()
            .color(palette.text),
    );
    ui.add(
        egui::TextEdit::multiline(&mut state.notes)
            .hint_text("Anota observaciones de la sesión…")
            .desired_rows(5)
            .desired_width(f32::INFINITY),
    );
    ui.label(
        RichText::new(
            "Las notas se guardan en el estado de esta sesión; no alteran parámetros físicos.",
        )
        .small()
        .color(palette.muted),
    );
}

fn format_duration(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn draw_preferences_window(ctx: &egui::Context, state: &mut AppState, palette: Palette) {
    if !state.show_preferences {
        return;
    }
    let mut open = state.show_preferences;
    egui::Window::new("Preferencias de Software caser")
        .open(&mut open)
        .default_width(440.0)
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Apariencia y accesibilidad")
                    .strong()
                    .color(palette.cyan),
            );
            egui::ComboBox::from_id_salt("theme-choice")
                .selected_text(match state.preferences.theme {
                    ThemeChoice::Midnight => "Midnight",
                    ThemeChoice::Slate => "Slate",
                    ThemeChoice::HighContrast => "Alto contraste",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.preferences.theme,
                        ThemeChoice::Midnight,
                        "Midnight",
                    );
                    ui.selectable_value(&mut state.preferences.theme, ThemeChoice::Slate, "Slate");
                    ui.selectable_value(
                        &mut state.preferences.theme,
                        ThemeChoice::HighContrast,
                        "Alto contraste",
                    );
                });
            ui.add(
                egui::Slider::new(&mut state.preferences.font_scale, 0.8..=1.5)
                    .text("Tamaño de texto"),
            );
            ui.add(
                egui::Slider::new(&mut state.preferences.zoom, 0.7..=1.8).text("Zoom de escena"),
            );
            ui.checkbox(&mut state.preferences.simple_mode, "Modo simple");
            ui.separator();
            ui.label(
                RichText::new("Diseño guardado")
                    .strong()
                    .color(palette.cyan),
            );
            ui.horizontal_wrapped(|ui| {
                for (preset, label) in [
                    (crate::state::LayoutPreset::Workbench, "Mesa"),
                    (crate::state::LayoutPreset::Analysis, "Análisis"),
                    (crate::state::LayoutPreset::Presentation, "Presentación"),
                ] {
                    if ui
                        .selectable_label(state.preferences.layout_preset == preset, label)
                        .clicked()
                    {
                        state.set_layout_preset(preset);
                    }
                }
            });
            ui.checkbox(
                &mut state.preferences.controls_panel_open,
                "Panel de controles visible",
            );
            ui.checkbox(
                &mut state.preferences.readouts_panel_open,
                "Panel de lecturas visible",
            );
            ui.checkbox(
                &mut state.preferences.controls_on_left,
                "Fijar controles a la izquierda",
            );
            ui.add(
                egui::Slider::new(&mut state.preferences.controls_panel_width, 230.0..=410.0)
                    .text("Ancho controles"),
            );
            ui.add(
                egui::Slider::new(&mut state.preferences.readouts_panel_width, 220.0..=390.0)
                    .text("Ancho lecturas"),
            );
            ui.checkbox(
                &mut state.preferences.show_hitboxes,
                "Mostrar hitboxes siempre (depuración)",
            );
            ui.separator();
            ui.label(RichText::new("Rendimiento").strong().color(palette.cyan));
            egui::ComboBox::from_id_salt("performance-choice")
                .selected_text(match state.preferences.performance {
                    PerformanceMode::Balanced => "Equilibrado",
                    PerformanceMode::Eco => "Ahorro de recursos",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.preferences.performance,
                        PerformanceMode::Balanced,
                        "Equilibrado",
                    );
                    ui.selectable_value(
                        &mut state.preferences.performance,
                        PerformanceMode::Eco,
                        "Ahorro de recursos",
                    );
                });
            ui.label(
                RichText::new("Ahorro de recursos reduce detalles del dibujo, no modifica datos.")
                    .small()
                    .color(palette.muted),
            );
            ui.separator();
            ui.label(
                RichText::new("Atajos editables")
                    .strong()
                    .color(palette.cyan),
            );
            shortcut_combo(
                ui,
                "shortcut-play",
                "Reproducir / pausar",
                &mut state.preferences.shortcuts.toggle_play,
            );
            shortcut_combo(
                ui,
                "shortcut-reset",
                "Reiniciar animación",
                &mut state.preferences.shortcuts.reset_animation,
            );
            shortcut_combo(
                ui,
                "shortcut-step",
                "Avanzar un cuadro",
                &mut state.preferences.shortcuts.step_frame,
            );
            shortcut_combo(
                ui,
                "shortcut-diagnostics",
                "Diagnóstico",
                &mut state.preferences.shortcuts.diagnostics,
            );
            ui.label(
                RichText::new("Ctrl+K, F1, Ctrl+Z, Ctrl+Y y F11 permanecen fijos.")
                    .small()
                    .color(palette.muted),
            );
        });
    state.show_preferences = open;
}

fn shortcut_combo(ui: &mut egui::Ui, id: &'static str, label: &str, key: &mut ShortcutKey) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(id)
            .selected_text(key.label())
            .show_ui(ui, |ui| {
                for option in ShortcutKey::ALL {
                    ui.selectable_value(key, option, option.label());
                }
            });
    });
}

fn draw_shortcuts_window(ctx: &egui::Context, state: &mut AppState, palette: Palette) {
    if !state.show_shortcuts {
        return;
    }
    let mut open = state.show_shortcuts;
    egui::Window::new("Ayuda y atajos")
        .open(&mut open)
        .default_width(430.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(RichText::new("Atajos de teclado").strong().color(palette.cyan));
            ui.label(format!("{} · Reproducir o pausar", state.preferences.shortcuts.toggle_play.label()));
            ui.label(format!("{} · Reiniciar animación", state.preferences.shortcuts.reset_animation.label()));
            ui.label(format!("{} · Avanzar un cuadro", state.preferences.shortcuts.step_frame.label()));
            ui.label(format!("{} · Mostrar diagnóstico", state.preferences.shortcuts.diagnostics.label()));
            ui.label("Ctrl+K · Paleta de comandos");
            ui.label("F1 · Esta ayuda");
            ui.label("Ctrl+Z / Ctrl+Y · Deshacer / rehacer controles");
            ui.label("F11 · Modo presentación");
            ui.add_space(8.0);
            ui.label(RichText::new("Sobre esta escena").strong().color(palette.cyan));
            ui.label(RichText::new("Valores, curva y emisión los calcula fotoelectrico-physics (Kmax = hf − Φ, V₀ = Kmax/e). El dibujo de trayectorias es un esquema: la cantidad de electrones escala con intensidad/colección y su velocidad con sqrt(Kmax).").color(palette.text));
        });
    state.show_shortcuts = open;
}

fn draw_command_palette(
    ctx: &egui::Context,
    state: &mut AppState,
    palette: Palette,
) -> Option<Command> {
    if !state.show_command_palette {
        return None;
    }
    let mut open = state.show_command_palette;
    let mut selected = None;
    egui::Window::new("Paleta de comandos")
        .id(egui::Id::new("command-palette-window"))
        .open(&mut open)
        .default_width(500.0)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 90.0))
        .show(ctx, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.command_search).hint_text("Buscar acción…"),
            );
            ui.add_space(6.0);
            let query = state.command_search.to_lowercase();
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for (command, label) in COMMANDS {
                        if (query.is_empty() || label.to_lowercase().contains(&query))
                            && ui.selectable_label(false, label).clicked()
                        {
                            selected = Some(command);
                        }
                    }
                });
            ui.label(
                RichText::new("Enter para ejecutar · Esc para cerrar · Ctrl+K abre esta paleta")
                    .small()
                    .color(palette.muted),
            );
        });
    if selected.is_none() && ctx.input(|input| input.key_pressed(egui::Key::Enter)) {
        let query = state.command_search.to_lowercase();
        selected = COMMANDS
            .iter()
            .find(|(_, label)| query.is_empty() || label.to_lowercase().contains(&query))
            .map(|(command, _)| *command);
    }
    state.show_command_palette = open && selected.is_none();
    selected
}

fn draw_diagnostics_window(ctx: &egui::Context, state: &mut AppState, palette: Palette) {
    if !state.show_diagnostics {
        return;
    }
    let mut open = state.show_diagnostics;
    egui::Window::new("HUD · Diagnóstico")
        .open(&mut open)
        .default_pos(Pos2::new(18.0, 84.0))
        .default_width(250.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(format!("FPS estimados: {}", state.render_stats.fps));
            ui.label(format!(
                "Frame time: {:.2} ms",
                state.render_stats.frame_time_ms
            ));
            ui.label(format!(
                "Fotones dibujados: {}",
                state.render_stats.photons_drawn
            ));
            ui.label(format!(
                "Electrones dibujados: {}",
                state.render_stats.electrons_drawn
            ));
            ui.label(format!(
                "Tiempo de escena: {:.2} s",
                state.animation_time_seconds
            ));
            ui.label(format!(
                "Viewport egui: {:.0} x {:.0}",
                ctx.screen_rect().width(),
                ctx.screen_rect().height()
            ));
            ui.label(
                RichText::new("Conteos del renderer guiados por física (colección + intensidad).")
                    .small()
                    .color(palette.muted),
            );
        });
    state.show_diagnostics = open;
}

fn apply_command(state: &mut AppState, command: Command) {
    match command {
        Command::TogglePlayback => {
            state.animation_running = !state.animation_running;
            state.push_history(if state.animation_running {
                "Animación reanudada"
            } else {
                "Animación pausada"
            });
        }
        Command::ResetAnimation => state.reset_animation(),
        Command::StepFrame => state.step_frame(),
        Command::CellTab => set_tab(state, LabTab::Cell, "Celda"),
        Command::ChartTab => set_tab(state, LabTab::Chart, "Gráfica"),
        Command::CompareTab => set_tab(state, LabTab::Compare, "Comparar"),
        Command::LogTab => set_tab(state, LabTab::Log, "Registro"),
        Command::TogglePresentation => {
            state.preferences.presentation_mode = !state.preferences.presentation_mode;
            state.push_history(if state.preferences.presentation_mode {
                "Modo presentación activado"
            } else {
                "Modo presentación desactivado"
            });
        }
        Command::Preferences => state.show_preferences = true,
        Command::Diagnostics => state.show_diagnostics = !state.show_diagnostics,
        Command::ResetControls => {
            state.controls = Default::default();
            state.refresh_physics();
        }
        Command::Undo => state.undo(),
        Command::Redo => state.redo(),
    }
}

fn set_tab(state: &mut AppState, tab: LabTab, label: &str) {
    if state.active_tab != tab {
        state.active_tab = tab;
        state.push_history(format!("Pestaña abierta: {label}"));
    }
}

#[cfg(test)]
#[path = "ui_hit_test_tests.rs"]
mod hit_test_tests;
