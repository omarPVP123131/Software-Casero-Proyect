//! Orquestación de capas Macroquad.

use macroquad::prelude::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, Color,
};

use crate::layers::cell::{draw_cell, Palette};
use crate::layers::{electrons, field, overlays, photons};
use crate::scene::{RenderQuality, RenderStats, SceneFrame, SceneObject};

#[derive(Debug, Default)]
pub struct Renderer;

impl Renderer {
    /// Dibuja la escena dentro del viewport que la UI reservó.
    /// No calcula magnitudes físicas; visualiza las lecturas del motor.
    pub fn render(&mut self, frame: &SceneFrame<'_>) -> RenderStats {
        if !frame.viewport.is_visible() {
            return RenderStats::default();
        }
        let palette = palette(frame.high_contrast);
        let rect = frame.viewport;
        draw_rectangle(rect.x, rect.y, rect.width, rect.height, palette.chamber);
        draw_line(
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y,
            1.0,
            palette.outline,
        );
        draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            1.0,
            palette.outline,
        );
        overlays::draw_grid(rect, frame.layers.grid, palette.grid);

        let geometry = draw_cell(rect, frame, palette);
        field::draw_field_arrows(frame, geometry, palette.field);
        let photon_color = photons::spectrum_color(frame.wavelength_nm, frame.high_contrast);
        let photons_drawn = photons::draw_photons(frame, geometry, photon_color);
        let electrons_drawn = electrons::draw_electrons(frame, geometry, palette.electron);
        overlays::draw_labels(frame, geometry, palette.label);
        overlays::draw_ruler(rect, frame.layers.ruler, palette.ruler, frame.font_scale);

        if frame.show_hitboxes {
            draw_rectangle_lines(
                rect.x + 1.0,
                rect.y + 1.0,
                rect.width - 2.0,
                rect.height - 2.0,
                1.0,
                Color::from_rgba(67, 211, 231, 120),
            );
        }
        if let Some(selected) = frame.selected_object {
            highlight_object(
                frame,
                geometry,
                selected,
                Color::from_rgba(255, 184, 103, 255),
                0.24,
            );
        }
        if frame.quality == RenderQuality::Balanced && !frame.animation_running {
            draw_rectangle(
                rect.x + rect.width - 98.0,
                rect.y + 12.0,
                82.0,
                22.0,
                Color::from_rgba(38, 49, 69, 220),
            );
            macroquad::prelude::draw_text(
                "PAUSADO",
                rect.x + rect.width - 86.0,
                rect.y + 28.0,
                11.0,
                palette.label,
            );
        }

        RenderStats {
            fps: macroquad::prelude::get_fps() as u32,
            frame_time_ms: macroquad::prelude::get_frame_time() * 1000.0,
            photons_drawn,
            electrons_drawn,
        }
    }
}

fn highlight_object(
    frame: &SceneFrame<'_>,
    geometry: crate::layers::cell::CellGeometry,
    object: SceneObject,
    color: Color,
    glow_alpha: f32,
) {
    let with_alpha = |alpha: f32| Color::new(color.r, color.g, color.b, alpha);
    match object {
        SceneObject::Canvas => {
            let rect = frame.viewport;
            draw_rectangle_lines(
                rect.x + 2.0,
                rect.y + 2.0,
                rect.width - 4.0,
                rect.height - 4.0,
                2.2,
                with_alpha(0.85),
            );
        }
        SceneObject::Cathode | SceneObject::Anode => {
            let x = if object == SceneObject::Cathode {
                geometry.cathode_x
            } else {
                geometry.anode_x
            };
            let padding = 6.0 * frame.zoom.clamp(0.7, 1.8);
            let left = x - geometry.plate_width * 0.5 - padding;
            let top = geometry.plate_top - padding;
            let width = geometry.plate_width + padding * 2.0;
            let height = geometry.plate_bottom - geometry.plate_top + padding * 2.0;
            draw_rectangle(left, top, width, height, with_alpha(glow_alpha));
            draw_rectangle_lines(left, top, width, height, 2.2, with_alpha(0.95));
        }
        SceneObject::PhotonBeam => photons::draw_beam_highlight(frame, geometry, color),
        SceneObject::ElectronLayer => {
            let visible = electrons::electrons_visible(
                frame.collection_possible,
                frame.emission_possible,
                frame.intensity_percent,
                frame.demo_electrons,
            );
            if frame.layers.electrons && visible {
                let speed = electrons::electron_speed_scale(frame.k_max_ev);
                let count = electrons::electron_count(frame.intensity_percent, true, frame.quality);
                for particle in electrons::visual_electrons(
                    geometry,
                    frame.time_seconds,
                    frame.animation_running,
                    frame.animation_speed,
                    frame.quality,
                    speed,
                    count,
                ) {
                    draw_circle(particle.x, particle.y, 11.0, with_alpha(glow_alpha));
                    draw_circle_lines(particle.x, particle.y, 9.0, 2.0, with_alpha(0.9));
                }
            }
        }
        SceneObject::FieldArrows => field::draw_field_highlight(frame, geometry, color),
    }
}

fn palette(high_contrast: bool) -> Palette {
    if high_contrast {
        Palette {
            chamber: Color::from_rgba(5, 10, 18, 255),
            outline: Color::from_rgba(205, 228, 255, 230),
            texture: Color::from_rgba(255, 245, 180, 170),
            photon: Color::from_rgba(255, 230, 0, 255),
            electron: Color::from_rgba(0, 255, 120, 255),
            field: Color::from_rgba(255, 90, 220, 255),
            label: Color::from_rgba(255, 255, 255, 255),
            muted: Color::from_rgba(211, 225, 240, 220),
            grid: Color::from_rgba(91, 122, 154, 42),
            ruler: Color::from_rgba(220, 235, 255, 190),
        }
    } else {
        Palette {
            chamber: Color::from_rgba(11, 20, 37, 245),
            outline: Color::from_rgba(45, 65, 93, 170),
            texture: Color::from_rgba(255, 229, 190, 110),
            photon: Color::from_rgba(84, 210, 235, 255),
            electron: Color::from_rgba(65, 211, 151, 255),
            field: Color::from_rgba(165, 135, 255, 255),
            label: Color::from_rgba(231, 239, 250, 230),
            muted: Color::from_rgba(139, 157, 181, 220),
            grid: Color::from_rgba(81, 111, 145, 32),
            ruler: Color::from_rgba(168, 190, 215, 170),
        }
    }
}
