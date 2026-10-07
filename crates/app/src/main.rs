use fotoelectrico_app::{draw_ui, AppState, UiFrame, UiPersistence};
use fotoelectrico_engine::{RenderLayers, RenderQuality, Renderer, SceneFrame, Viewport};
use macroquad::prelude::{clear_background, get_time, next_frame, Color, Conf};

fn window_conf() -> Conf {
    Conf {
        window_title: "Software Casera".to_owned(),
        window_width: 1440,
        window_height: 900,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = AppState::default();
    let mut persistence = UiPersistence::restore(&mut app);
    app.refresh_physics();
    let mut renderer = Renderer;
    let mut last_frame_time = get_time();

    loop {
        let now = get_time();
        let delta_seconds = (now - last_frame_time).max(0.0);
        last_frame_time = now;
        app.tick(delta_seconds);
        // La física se recalcula cada frame: controles -> motor -> lecturas.
        app.refresh_physics();

        let background = match app.preferences.theme {
            fotoelectrico_app::state::ThemeChoice::Midnight => Color::from_rgba(10, 15, 28, 255),
            fotoelectrico_app::state::ThemeChoice::Slate => Color::from_rgba(22, 27, 34, 255),
            fotoelectrico_app::state::ThemeChoice::HighContrast => Color::from_rgba(0, 0, 0, 255),
        };
        clear_background(background);
        let mut layout = UiFrame::default();
        egui_macroquad::ui(|ctx| {
            layout = draw_ui(ctx, &mut app);
        });

        app.render_stats = if let Some(rect) = layout.scene_rect {
            let scale = layout.coordinate_scale;
            let material_label = format!(
                "{} ({})",
                app.controls.material.name(),
                app.controls.material.symbol()
            );
            let layers = RenderLayers {
                photons: app.preferences.layers.photons,
                electrons: app.preferences.layers.electrons,
                arrows: app.preferences.layers.arrows,
                labels: app.preferences.layers.labels,
                trails: app.preferences.layers.trails,
                grid: app.preferences.layers.grid,
                ruler: app.preferences.layers.ruler,
                electrode_texture: app.preferences.layers.electrode_texture,
            };
            let frame = SceneFrame {
                viewport: Viewport::new(
                    rect.left() * scale,
                    rect.top() * scale,
                    rect.width() * scale,
                    rect.height() * scale,
                ),
                wavelength_nm: app.controls.wavelength_nm,
                intensity_percent: app.controls.intensity_percent,
                visual_voltage_v: app.controls.applied_voltage_v,
                material_label: &material_label,
                time_seconds: app.animation_time_seconds,
                animation_running: app.animation_running,
                animation_speed: app.animation_speed,
                demo_electrons: app.preferences.layers.demo_electrons,
                emission_possible: app.readout.emission_possible,
                collection_possible: app.readout.collected_possible,
                k_max_ev: app.readout.max_kinetic_energy_ev,
                electron_max_speed_m_s: app.readout.electron_max_speed_m_s,
                zoom: app.scene_camera.zoom,
                pan_x: app.scene_camera.pan_x,
                pan_y: app.scene_camera.pan_y,
                selected_object: app.inspector.map(Into::into),
                show_hitboxes: app.preferences.show_hitboxes,
                font_scale: app.preferences.font_scale,
                layers,
                quality: match app.preferences.performance {
                    fotoelectrico_app::state::PerformanceMode::Balanced => RenderQuality::Balanced,
                    fotoelectrico_app::state::PerformanceMode::Eco => RenderQuality::Performance,
                },
                high_contrast: app.preferences.theme
                    == fotoelectrico_app::state::ThemeChoice::HighContrast,
            };
            renderer.render(&frame)
        } else {
            Default::default()
        };

        persistence.save_if_changed(&app, now);
        // egui se pinta al final y queda por encima del canvas Macroquad.
        egui_macroquad::draw();
        next_frame().await;
    }
}
