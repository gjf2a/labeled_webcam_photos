use crossbeam::atomic::AtomicCell;
use image::GrayImage;
use labeled_webcam_photos::{Menu, groundline, groundline_image};
use nokhwa::{
    Camera,
    pixel_format::LumaFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use pancurses::{Input, endwin, initscr, noecho};
use r2r::{Context, Node, Publisher, QosProfile, std_msgs::msg::String as Ros2String};
use smol::lock::Mutex;
use std::{sync::Arc, time::Instant};

const PERIOD: u64 = 100;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: groundline_node robot_name");
        Ok(())
    } else {
        curses_loop(&args[1])
    }
}

fn curses_loop(robot_name: &str) -> anyhow::Result<()> {
    let mut menu = Menu::default();

    let image = Arc::new(Mutex::new(None));
    let thread_image = image.clone();
    let robot_name = robot_name.to_string();
    std::thread::spawn(move || {
        if let Err(e) = runner(&robot_name, thread_image) {
            eprintln!("Couldn't start groundline thread: {e}");
        }
    });
    

    let window = initscr();
    window.keypad(true);
    window.nodelay(true);
    noecho();

    let start = Instant::now();
    let mut frames = 0;
    loop {
        let fps = frames as f64 / start.elapsed().as_secs_f64();
        let (wrows, wcols) = window.get_max_yx();
        let header =
            format!("Type `q` to exit\nterminal rows: {wrows} cols: {wcols}\n{fps:.2} fps;\n");
        if let Some(mut image_buffer) = image.try_lock() {
            if let Some(image) = image_buffer.as_ref() {
                frames += 1;
                menu.show_in_terminal(&window, header.as_str(), &image, false);
                *image_buffer = None;
            }
        }

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

fn runner(robot_name: &str, image: Arc<Mutex<Option<GrayImage>>>) -> anyhow::Result<()> {
    let label_topic = format!("/{robot_name}_groundline");
    let context = Context::create()?;
    let node_name = format!("{robot_name}_groundline_node");
    let mut node = Node::create(context, node_name.as_str(), "")?;
    let publisher =
        node.create_publisher::<Ros2String>(label_topic.as_str(), QosProfile::sensor_data())?;
    println!("Publishing groundline on topic {label_topic}.");

    let running = Arc::new(AtomicCell::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false))?;

    smol::block_on(async {
        smol::spawn(image_handler(publisher, image)).detach();
        while running.load() {
            node.spin_once(std::time::Duration::from_millis(PERIOD));
        }
    });
    Ok(())
}

async fn image_handler(publisher: Publisher<Ros2String>, image: Arc<Mutex<Option<GrayImage>>>) {
    let mut camera = Camera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    )
    .unwrap();

    camera.open_stream().unwrap();

    loop {
        let frame = camera.frame().unwrap();
        let img = frame.decode_image::<LumaFormat>().unwrap();
        let groundline = groundline(&img);
        let groundline_image = groundline_image(&groundline);
        if let Some(mut image) = image.try_lock() {
            *image = Some(groundline_image);
        }
        let msg = Ros2String {
            data: format!("{groundline:?}"),
        };
        if let Err(e) = publisher.publish(&msg) {
            eprintln!("Error publishing {msg:?}: {e}");
        }
    }
}
