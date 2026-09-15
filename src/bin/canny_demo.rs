use edge_detection::canny;
use labeled_webcam_photos::Menu;
use nokhwa::{
    Camera,
    pixel_format::LumaFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use pancurses::{Input, endwin, initscr, noecho};
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 4 {
        println!("Usage: canny_demo sigma strong_threshold weak_threshold");
        Ok(())
    } else {
        curses_loop(args[1].parse().unwrap(), args[2].parse().unwrap(), args[3].parse().unwrap())
    }    
}

fn curses_loop(sigma: f32, strong_threshold: f32, weak_threshold: f32) -> anyhow::Result<()> {
    let mut menu = Menu::default();

    let mut camera = Camera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    )?;

    camera.open_stream()?;

    let window = initscr();
    window.keypad(true);
    window.nodelay(true);
    noecho();

    let start = Instant::now();
    let mut frames = 0;
    loop {
        frames += 1;
        let fps = frames as f64 / start.elapsed().as_secs_f64();
        let (wrows, wcols) = window.get_max_yx();
        let header = format!(
            "Type `q` to exit\nterminal rows: {wrows} cols: {wcols}\n{fps:.2} fps;\n"
        );
        let frame = camera.frame()?;
        let image = frame.decode_image::<LumaFormat>()?;
        let edges = canny(image, sigma, strong_threshold, weak_threshold).as_image().into_luma8();
        menu.show_in_terminal(&window, header.as_str(), &edges, false);
        
        if let Some(k) = window.getch() {
            if k == Input::Character('q') {
                break;
            } else if k == Input::KeyUp {
                menu.up();
            } else if k == Input::KeyDown {
                menu.down();
            } 
        }
    }

    endwin();
    Ok(())
}
