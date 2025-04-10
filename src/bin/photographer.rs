use nokhwa::{
    pixel_format::{LumaFormat, RgbFormat},
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use pancurses::{endwin, initscr, noecho, Input};
use std::time::Instant;
use labeled_webcam_photos::LabeledPhotoGallery;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 3 {
        println!("Usage: photographer project_name label_1 label_2 ...");
        return Ok(());
    }

    let project = args[0].as_str();
    let mut photos = LabeledPhotoGallery::with_labels(project, args[1..].iter().cloned())?;
    curses_loop(&mut photos)?;
    Ok(())
}

fn curses_loop(photos: &mut LabeledPhotoGallery) -> anyhow::Result<()> {
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
    let mut taken = false;
    let mut num_taken = 0;

    let start = Instant::now();
    let mut frames = 0;
    loop {
        frames += 1;
        let fps = frames as f64 / start.elapsed().as_secs_f64();
        let (wrows, wcols) = window.get_max_yx();
        let header = format!("Type `q` to save images and exit\nterminal rows: {wrows} cols: {wcols}\n{fps:.2} fps; {num_taken} pictures taken\n");
        let frame = camera.frame()?;
        let img = frame.decode_image::<LumaFormat>()?;
        menu.show_in_terminal(&window, header.as_str(), &img, taken);
        if taken {
            taken = false;
        }

        if let Some(k) = window.getch() {
            if k == Input::Character('q') {
                break;
            } else if k == Input::KeyUp {
                menu.up();
            } else if k == Input::KeyDown {
                menu.down();
            } else if k == Input::Character('p') || k == Input::Character('\n') {
                let img = frame.decode_image::<RgbFormat>()?;
                photos.record_photo(menu.current_choice(), &img)?;
                taken = true;
                num_taken += 1;
            }
        }
    }

    endwin();
    Ok(())
}