use std::sync::Arc;
use nokhwa::{
    pixel_format::{LumaFormat, RgbFormat},
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};

use labeled_webcam_photos::LabeledPhotoGallery;
use r2r::{std_msgs::msg::String as Ros2String, Context, Node, Publisher, QosProfile};
use crossbeam::atomic::AtomicCell;

const PERIOD: u64 = 100;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: knn_photo_node robot_name project_name");
        return;
    }

    let robot_name = args[0].as_str();
    let project_name = args[1].as_str();

    let gallery = LabeledPhotoGallery::from_disk(project_name).unwrap();
    if let Err(e) = runner(robot_name, gallery) {
        eprintln!("Unrecoverable error: {e}");
    }
}

fn runner(robot_name: &str, gallery: LabeledPhotoGallery) -> anyhow::Result<()> {
    let label_topic = format!("/{robot_name}_image_label");
    let context = Context::create()?;
    let node_name = format!("{robot_name}_image_labeler");
    let mut node = Node::create(context, node_name.as_str(), "")?;
    let publisher = node.create_publisher::<Ros2String>(label_topic.as_str(), QosProfile::sensor_data())?;
    println!("Publishing image label on topic {label_topic}.");

    let running = Arc::new(AtomicCell::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false))?;


    smol::block_on(async {
        smol::spawn(image_handler(gallery, publisher)).detach();
        while running.load() {
            node.spin_once(std::time::Duration::from_millis(PERIOD));
        }
    });
    Ok(())
}

async fn image_handler(gallery: LabeledPhotoGallery, publisher: Publisher<Ros2String>) {
    let mut camera = Camera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    ).unwrap();

    camera.open_stream().unwrap();

    loop {
        let frame = camera.frame().unwrap();
        let img = frame.decode_image::<RgbFormat>().unwrap();
        let msg = Ros2String { data: gallery.label_for(&img)};
        if let Err(e) = publisher.publish(&msg) {
            eprintln!("Error publishing {msg:?}: {e}");
        }
    }
}