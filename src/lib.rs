use std::{collections::HashMap, fs, path::{Path, PathBuf}};

use anyhow::anyhow;
use hash_histogram::mode;
use image::{
    GrayImage, Pixel, RgbImage,
    imageops::{FilterType, resize},
};
use pancurses::A_REVERSE;
use pancurses::Window;

const K: usize = 5;

#[derive(Default)]
pub struct LabeledPhotoGallery {
    project_name: String,
    label2photos: HashMap<String, Vec<RgbImage>>,
}

impl LabeledPhotoGallery {
    pub fn from_disk(project_name: &str) -> anyhow::Result<Self> {
        let mut result = Self {
            project_name: project_name.to_string(),
            label2photos: HashMap::default(),
        };
        let project_dir = Path::new(project_name);
        if !project_dir.exists() || !project_dir.is_dir() {
            return Err(anyhow!("{project_name} is not a directory"));
        }

        for label in fs::read_dir(project_dir)? {
            let label = label?;
            let label_str = label.path().to_str().unwrap().to_string();
            let mut photos = vec![];
            for photo in fs::read_dir(label.path())? {
                let photo = photo?;
                let image = image::open(photo.path())?.to_rgb8();
                photos.push(image);
            }
            result.label2photos.insert(label_str, photos);
        }
        Ok(result)
    }

    pub fn label_for(&self, img: &RgbImage) -> String {
        let mut distances = vec![];
        for (label, photos) in self.label2photos.iter() {
            for photo in photos.iter() {
                let dist = euclidean_distance(&img, photo);
                distances.push((dist, label.clone()));
            }
        }
        distances.sort_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap());
        mode(distances[..K].iter().map(|(_, k)| k)).unwrap()
    }

    pub fn with_labels<I: Iterator<Item = String>>(project_name: &str, labels: I) -> anyhow::Result<Self> {
        let result = Self {
            project_name: project_name.to_string(),
            label2photos: labels.map(|s| (s, vec![])).collect(),
        };
        result.create_directories()?;
        Ok(result)
    }

    fn create_directories(&self) -> anyhow::Result<()> {
        let project_dir = Path::new(self.project_name.as_str());
        Self::create_dir_if_needed(project_dir)?;

        for label in self.all_labels().iter() {
            let label_path = project_dir.join(label);
            Self::create_dir_if_needed(label_path.as_path())?;
        }
        Ok(())
    }

    fn create_dir_if_needed(path: &Path) -> anyhow::Result<()> {
        if path.exists() {
            if !path.is_dir() {
                return Err(anyhow!(
                    "'{}' already exists as a file, not a directory.", path.display()
                ));
            }
        } else {
            fs::create_dir(path)?;
        }
        Ok(())
    }

    fn label_dir_path(&self, label: &str) -> anyhow::Result<PathBuf> {
        let project_dir = Path::new(self.project_name.as_str());
        let label_dir = project_dir.join(label);
        if label_dir.is_dir() {
            Ok(label_dir)
        } else {
            Err(anyhow!("{label} does not have a directory defined"))
        }
    }

    pub fn record_photo(&mut self, label: &str, img: &RgbImage) -> anyhow::Result<()> {
        if self.label2photos.contains_key(label) {
            self.label2photos.get_mut(label).unwrap().push(img.clone());
            let label_dir = self.label_dir_path(label)?;
            let file_count = fs::read_dir(label_dir.as_path())?.count();
            let filename = format!("photo_{}.png", file_count + 1);
            let file_path = label_dir.join(filename);
            img.save(file_path)?;
            Ok(())
        } else {
            Err(anyhow!("{label} is undefined"))
        }
    }

    pub fn all_labels(&self) -> Vec<String> {
        let mut labels = self.label2photos.keys().cloned().collect::<Vec<_>>();
        labels.sort();
        labels
    }

    pub fn make_menu(&self) -> Menu {
        Menu::from_choices(self.all_labels())
    }
}

// From https://www.perplexity.ai/search/use-the-rust-image-crate-to-fi-RelkEa9VQjOTA7TKiF.yCA
fn euclidean_distance(img1: &RgbImage, img2: &RgbImage) -> f64 {
    assert_eq!(
        img1.dimensions(),
        img2.dimensions(),
        "Image dimensions must match"
    );

    img1.pixels()
        .zip(img2.pixels())
        .map(|(p1, p2)| {
            let channels1 = p1.channels();
            let channels2 = p2.channels();
            channels1
                .iter()
                .zip(channels2.iter())
                .map(|(&c1, &c2)| (c1 as f64 - c2 as f64).powi(2))
                .sum::<f64>()
        })
        .sum()
}

pub struct Menu {
    choices: Vec<String>,
    choice: usize,
}

impl Menu {
    pub fn from_choices(choices: Vec<String>) -> Self {
        Self { choices, choice: 0 }
    }

    pub fn current_choice(&self) -> &str {
        self.choices[self.choice].as_str()
    }

    pub fn up(&mut self) {
        if self.choice == 0 {
            self.choice = self.choices.len() - 1;
        } else {
            self.choice -= 1;
        }
    }

    pub fn down(&mut self) {
        self.choice = (self.choice + 1) % self.choices.len();
    }

    pub fn show_in_terminal(&self, terminal: &Window, header: &str, img: &GrayImage, taken: bool) {
        let header_size = header.lines().count();
        terminal.clear();
        for (row, line) in header.lines().enumerate() {
            terminal.mvaddstr(row as i32, 0, line);
        }
        for row in 0..self.choices.len() {
            if row == self.choice {
                terminal.attron(A_REVERSE);
            }
            terminal.mvaddstr((row + header_size) as i32, 0, self.choices[row].as_str());
            terminal.attroff(A_REVERSE);
        }

        let (height, width) = terminal.get_max_yx();
        let (scaled_width, scaled_height) = target_terminal_width_height(
            img.width(),
            img.height(),
            width,
            height - header_size as i32,
        );
        let resized = resize(img, scaled_width, scaled_height, FilterType::Nearest);
        if taken {
            terminal.attron(A_REVERSE);
        }
        for (row, row_pixels) in resized.rows().enumerate() {
            for (col, pixel) in row_pixels.enumerate() {
                let c = gray2char(pixel.0[0]);
                terminal.mvaddch(
                    (row + header_size + self.choices.len()) as i32,
                    col as i32,
                    c,
                );
            }
        }
        terminal.attroff(A_REVERSE);
        terminal.refresh();
    }
}

const ENCODINGS: [char; 10] = [' ', '.', ':', ';', '!', '?', '+', '*', '@', '#'];

fn target_terminal_width_height(
    img_width: u32,
    img_height: u32,
    term_width: i32,
    term_height: i32,
) -> (u32, u32) {
    let term_height = term_height as u32;
    let term_width = term_width as u32;
    if term_width < term_height {
        (term_width, term_height * img_height / img_width)
    } else {
        (term_width * img_width / img_height, term_height)
    }
}

fn gray2char(gray: u8) -> char {
    let gap = 1 + (u8::MAX / ENCODINGS.len() as u8);
    ENCODINGS[(gray / gap) as usize]
}
