use std::collections::HashMap;

use image::{imageops::{resize, FilterType}, GrayImage};
use pancurses::Window;
use pancurses::A_REVERSE;

#[derive(Default)]
pub struct LabeledPhotoGallery {
    label2photos: HashMap<String, Vec<GrayImage>>
}

impl LabeledPhotoGallery {
    pub fn with_labels<I: Iterator<Item=String>>(labels: I) -> Self {
        Self {
            label2photos: labels.map(|s| (s, vec![])).collect()
        }
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

    pub fn show_in_terminal(&self, terminal: &Window, header: &str, img: &GrayImage) {
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
        for (row, row_pixels) in resized.rows().enumerate() {
            for (col, pixel) in row_pixels.enumerate() {
                let c = gray2char(pixel.0[0]);
                terminal.mvaddch((row + header_size + self.choices.len()) as i32, col as i32, c);
            }
        }
        terminal.refresh();
    }
}

const ENCODINGS: [char; 10] = [' ', '.', ':', ';', '!', '?', '+', '*', '@', '#'];

pub fn encode_in_terminal(header: &str, img: &GrayImage, terminal: &Window) {
    show_in_terminal(header, &resize_to_terminal(header, img, terminal), terminal);
}

pub fn header_height(header: &str) -> usize {
    header.chars().filter(|c| *c == '\n').count()
}

pub fn resize_to_terminal(header: &str, img: &GrayImage, terminal: &Window) -> GrayImage {
    let (height, width) = terminal.get_max_yx();
    let (scaled_width, scaled_height) = target_terminal_width_height(
        img.width(),
        img.height(),
        width,
        height - header_height(header) as i32,
    );
    resize(img, scaled_width, scaled_height, FilterType::Nearest)
}

pub fn show_in_terminal(header: &str, terminal_sized: &GrayImage, terminal: &Window) {
    show_header(header, terminal);
    let header_height = header_height(header);
    for (row, row_pixels) in terminal_sized.rows().enumerate() {
        for (col, pixel) in row_pixels.enumerate() {
            let c = gray2char(pixel.0[0]);
            terminal.mvaddch((row + header_height) as i32, col as i32, c);
        }
    }
    terminal.refresh();
}

pub fn show_header(header: &str, terminal: &Window) {
    terminal.clear();
    for (row, line) in header.lines().enumerate() {
        terminal.mvaddstr(row as i32, 0, line);
    }
}

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