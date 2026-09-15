use labeled_webcam_photos::{Menu, groundline, groundline_image};
use nokhwa::{
    Camera,
    pixel_format::LumaFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use pancurses::{Input, endwin, initscr, noecho};
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    curses_loop()
}

fn curses_loop() -> anyhow::Result<()> {
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
        let frame = camera.frame()?;
        let image = frame.decode_image::<LumaFormat>()?;

        let header = format!(
            "Type `q` to exit\nimage width: {} height: {}\nterminal rows: {wrows} cols: {wcols}\n{fps:.2} fps;\n",
            image.width(),
            image.height()
        );
        let groundline = groundline(&image);
        let image = groundline_image(&groundline, image.height());
        menu.show_in_terminal(&window, header.as_str(), &image, false);

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
