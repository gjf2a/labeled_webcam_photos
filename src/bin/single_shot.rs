use labeled_webcam_photos::LabeledPhotoGallery;
use nokhwa::{
    Camera,
    pixel_format::{LumaFormat, RgbFormat},
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: single_shot project_name label");
        return Ok(());
    }

    let project = args[0].as_str();
    let label = args[1].as_str();
    let mut photos = LabeledPhotoGallery::with_labels(project, vec![label.to_string()].iter().cloned())?;
    let mut camera = Camera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<LumaFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
    )?;

    camera.open_stream()?;
    let frame = camera.frame()?;
    let img = frame.decode_image::<RgbFormat>()?;
    photos.record_photo(label, &img)?;

    Ok(())
}