//! Celda, cátodo y ánodo: representación geométrica, sin modelo físico.

use macroquad::prelude::{draw_line, draw_rectangle, Color};

use crate::scene::{SceneFrame, Viewport};

#[derive(Debug, Clone, Copy)]
pub struct CellGeometry {
    pub beam_start_x: f32,
    pub cathode_x: f32,
    pub anode_x: f32,
    pub plate_top: f32,
    pub plate_bottom: f32,
    pub plate_width: f32,
    pub mid_y: f32,
}

/// Aplica la transformación de cámara sobre una coordenada Macroquad.
pub fn transform_point(
    viewport: Viewport,
    x: f32,
    y: f32,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
) -> (f32, f32) {
    let zoom = zoom.clamp(0.7, 1.8);
    let center_x = viewport.x + viewport.width * 0.5;
    let center_y = viewport.y + viewport.height * 0.5;
    (
        center_x + (x - center_x) * zoom + pan_x.clamp(-0.75, 0.75) * viewport.width,
        center_y + (y - center_y) * zoom + pan_y.clamp(-0.75, 0.75) * viewport.height,
    )
}

/// Geometría común para el dibujo y los hitboxes de la UI.
/// Mantener este cálculo compartido evita que la selección se desplace al cambiar
/// el tamaño del canvas, el zoom o el foco de cámara.
pub fn layout_geometry(viewport: Viewport, zoom: f32, pan_x: f32, pan_y: f32) -> CellGeometry {
    let center_x = viewport.x + viewport.width * 0.52;
    let base_gap_half = viewport.width * 0.19;
    let base_mid_y = viewport.y + viewport.height * 0.53;
    let base_half_height = viewport.height * 0.29;
    let base_beam_start = viewport.x + 30.0;
    let base_cathode = center_x - base_gap_half;
    let base_anode = center_x + base_gap_half;
    let (cathode_x, mid_y) =
        transform_point(viewport, base_cathode, base_mid_y, zoom, pan_x, pan_y);
    let (anode_x, _) = transform_point(viewport, base_anode, base_mid_y, zoom, pan_x, pan_y);
    let (_, plate_top) = transform_point(
        viewport,
        center_x,
        base_mid_y - base_half_height,
        zoom,
        pan_x,
        pan_y,
    );
    let (_, plate_bottom) = transform_point(
        viewport,
        center_x,
        base_mid_y + base_half_height,
        zoom,
        pan_x,
        pan_y,
    );
    let (beam_start_x, _) =
        transform_point(viewport, base_beam_start, base_mid_y, zoom, pan_x, pan_y);

    CellGeometry {
        beam_start_x,
        cathode_x,
        anode_x,
        plate_top,
        plate_bottom,
        plate_width: (viewport.width * 0.018).clamp(9.0, 17.0) * zoom.clamp(0.7, 1.8),
        mid_y,
    }
}

pub fn draw_cell(viewport: Viewport, frame: &SceneFrame<'_>, palette: Palette) -> CellGeometry {
    let geometry = layout_geometry(viewport, frame.zoom, frame.pan_x, frame.pan_y);
    let CellGeometry {
        beam_start_x: _,
        cathode_x,
        anode_x,
        plate_top,
        plate_bottom,
        plate_width,
        mid_y: _,
    } = geometry;

    // Vaso de vacío esquemático.
    let (chamber_left, chamber_top) = transform_point(
        viewport,
        viewport.x + 18.0,
        viewport.y + 30.0,
        frame.zoom,
        frame.pan_x,
        frame.pan_y,
    );
    let (chamber_right, chamber_bottom) = transform_point(
        viewport,
        viewport.x + viewport.width - 18.0,
        viewport.y + viewport.height - 32.0,
        frame.zoom,
        frame.pan_x,
        frame.pan_y,
    );
    let chamber = macroquad::prelude::Rect::new(
        chamber_left,
        chamber_top,
        chamber_right - chamber_left,
        chamber_bottom - chamber_top,
    );
    draw_rectangle(chamber.x, chamber.y, chamber.w, chamber.h, palette.chamber);
    draw_line(
        chamber.x,
        chamber.y,
        chamber.x + chamber.w,
        chamber.y,
        1.0,
        palette.outline,
    );
    draw_line(
        chamber.x,
        chamber.y + chamber.h,
        chamber.x + chamber.w,
        chamber.y + chamber.h,
        1.0,
        palette.outline,
    );

    let cathode_color = if frame.high_contrast {
        Color::from_rgba(255, 205, 70, 255)
    } else {
        Color::from_rgba(244, 168, 91, 255)
    };
    let anode_color = if frame.high_contrast {
        Color::from_rgba(72, 194, 255, 255)
    } else {
        Color::from_rgba(88, 164, 196, 255)
    };

    draw_rectangle(
        cathode_x - plate_width * 0.5,
        plate_top,
        plate_width,
        plate_bottom - plate_top,
        cathode_color,
    );
    draw_rectangle(
        anode_x - plate_width * 0.5,
        plate_top,
        plate_width,
        plate_bottom - plate_top,
        anode_color,
    );

    if frame.layers.electrode_texture && frame.quality == crate::scene::RenderQuality::Balanced {
        let mut y = plate_top + 7.0;
        while y < plate_bottom {
            draw_line(
                cathode_x - plate_width * 0.42,
                y,
                cathode_x + plate_width * 0.42,
                y,
                0.65,
                palette.texture,
            );
            draw_line(
                anode_x - plate_width * 0.42,
                y,
                anode_x + plate_width * 0.42,
                y,
                0.65,
                palette.texture,
            );
            y += 8.0;
        }
    }

    geometry
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub chamber: Color,
    pub outline: Color,
    pub texture: Color,
    pub photon: Color,
    pub electron: Color,
    pub field: Color,
    pub label: Color,
    pub muted: Color,
    pub grid: Color,
    pub ruler: Color,
}

#[cfg(test)]
mod tests {
    use super::layout_geometry;
    use crate::scene::Viewport;

    #[test]
    fn hit_test_geometry_tracks_viewport_and_scene_zoom() {
        let viewport = Viewport::new(100.0, 40.0, 1000.0, 600.0);
        let normal = layout_geometry(viewport, 1.0, 0.0, 0.0);
        let zoomed = layout_geometry(viewport, 1.4, 0.0, 0.0);

        assert!((normal.cathode_x - 430.0).abs() < 0.01);
        assert!((normal.anode_x - 810.0).abs() < 0.01);
        assert!(zoomed.cathode_x < normal.cathode_x);
        assert!(zoomed.anode_x > normal.anode_x);
        assert!((normal.plate_top - 184.0).abs() < 0.01);
        assert!((normal.plate_bottom - 532.0).abs() < 0.01);
    }
}
