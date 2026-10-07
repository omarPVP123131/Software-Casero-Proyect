use fotoelectrico_app::{draw, LabControls, PhysicsReadout};
use macroquad::prelude::{clear_background, next_frame, Color, Conf};

fn window_conf() -> Conf {
    Conf {
        window_title: "PhotoLab · Efecto fotoeléctrico".to_owned(),
        window_width: 1440,
        window_height: 900,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut controls = LabControls::default();
    // La UI no deriva estos valores: quedan listos para recibir la salida del
    // modelo físico cuando se conecte desde la capa de aplicación.
    let readout = PhysicsReadout::default();

    loop {
        clear_background(Color::from_rgba(10, 15, 28, 255));
        let time_seconds = macroquad::prelude::get_time();

        egui_macroquad::ui(|ctx| {
            draw(ctx, &mut controls, &readout, time_seconds);
        });
        egui_macroquad::draw();
        next_frame().await;
    }
}
