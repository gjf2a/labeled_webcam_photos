use nokhwa::{
    pixel_format::LumaFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use pancurses::{endwin, initscr, noecho, Input};
use std::time::Instant;
use labeled_webcam_photos::LabeledPhotoGallery;

fn main() -> anyhow::Result<()> {
    let photos = LabeledPhotoGallery::with_labels(std::env::args().skip(1));
    let mut menu = photos.make_menu();

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
        let header = format!("terminal rows: {wrows} cols: {wcols}\n{fps:.2} fps\n");
        let frame = camera.frame()?;
        let img = frame.decode_image::<LumaFormat>()?;
        menu.show_in_terminal(&window, header.as_str(), &img);
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