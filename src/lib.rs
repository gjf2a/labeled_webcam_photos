use std::{collections::HashMap, path::Path};

use image::{imageops::{resize, FilterType}, GrayImage};
use pancurses::Window;
use pancurses::A_REVERSE;
use anyhow::anyhow;

#[derive(Default)]
pub struct LabeledPhotoGallery {
    label2photos: HashMap<String, Vec<GrayImage>>,
}

impl LabeledPhotoGallery {
    pub fn with_labels<I: Iterator<Item=String>>(labels: I) -> Self {
        Self {
            label2photos: labels.map(|s| (s, vec![])).collect(),
        }
    }

    pub fn create_directories(&self, project: &str) -> anyhow::Result<()> {
        let project_dir = Path::new(project);
        if project_dir.exists() {
            if !project_dir.is_dir() {
                return Err(anyhow!("'{project} already exists as a file, not a directory."));
            }
        } else {
            std::fs::create_dir(project_dir)?;
        }
    
        for label in self.all_labels() {
            let label_path = project_dir.join(label);
            if label_path.exists() {
                if !label_path.is_dir() {
                    return Err(anyhow!("{label} already exists as a file, not a directory."));
                }
            } else {
                std::fs::create_dir(label_path)?;
            }
        }
        Ok(())
    }

    pub fn save_images(&self, project: &str) -> anyhow::Result<()> {
        self.create_directories(project)?;
        let project_dir = Path::new(project);
        for (label, photos) in self.label2photos.iter() {
            let label_dir = project_dir.join(label);
            assert!(label_dir.is_dir());
            let file_count = std::fs::read_dir(label_dir.as_path())?.count();
            for (i, img) in photos.iter().enumerate() {
                let filename = format!("photo_{}.png", file_count + i + 1);
                let file_path = label_dir.join(filename);
                img.save(file_path)?;
            }
        }
        Ok(())
    }

    pub fn record_photo(&mut self, label: &str, img: &GrayImage) {
        assert!(self.label2photos.contains_key(label));
        self.label2photos.get_mut(label).unwrap().push(img.clone());
    }

    pub fn all_labels(&self) -> impl Iterator<Item=&String> {
        self.label2photos.keys()
    }

    pub fn make_menu(&self) -> Menu {
        Menu::from_choices(self.all_labels().cloned())
    }
}

pub struct Menu {
    choices: Vec<String>,
    choice: usize,
}

impl Menu {
    pub fn from_choices<I: Iterator<Item=String>>(choices: I) -> Self {
        Self {
            choices: choices.collect(),
            choice: 0
        }
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
                terminal.mvaddch((row + header_size + self.choices.len()) as i32, col as i32, c);
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