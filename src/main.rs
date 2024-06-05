mod absence;

use std::{
    collections::HashMap,
    fs::{create_dir, File},
    io::{self, BufRead, Read, Write},
    path::Path,
    process::Command,
};

use absence::read_all_absences;
use homedir::get_my_home;
use image::Rgba;
use imageproc::drawing::Canvas;
use rusttype::{Font, Scale};

const FEHLSTUNDEN_FILE_DATA: &[u8] = include_bytes!("../fehlstunden.png");

fn main() {
    let home = get_my_home().unwrap().unwrap();
    #[allow(non_snake_case)]
    let AUTOSIGN_DIR = &format!("{}/.autosign", home.display());
    let mut handle = Command::new("node")
        .arg("./webuntis-node-test/index.js")
        .arg(std::env::var("CUTOFF_DATE").unwrap_or("2024-03-04".to_owned()))
        .arg(std::env::var("USER_ID").expect("Missing user id e.g. (200111)"))
        .arg(std::env::var("USER_SECRET").expect("Missing user secret"))
        .spawn()
        .unwrap();

    handle.wait().unwrap();

    let absences = read_all_absences("absentLessons.json").unwrap();

    if !Path::new(AUTOSIGN_DIR).exists() {
        create_dir(AUTOSIGN_DIR).unwrap();
        std::fs::write(
            &format!("{AUTOSIGN_DIR}/fehlstunden.png"),
            FEHLSTUNDEN_FILE_DATA,
        )
        .unwrap();
    }
    if !Path::new(&format!("{AUTOSIGN_DIR}/out.png")).exists() {
        std::fs::write(&format!("{AUTOSIGN_DIR}/out.png"), FEHLSTUNDEN_FILE_DATA).unwrap();
    }
    let (last_id, offset) = if Path::new(&format!("{AUTOSIGN_DIR}/last_id")).exists() {
        let mut line = String::new();
        let mut file = File::open(&format!("{AUTOSIGN_DIR}/last_id")).unwrap();
        file.read_to_string(&mut line).unwrap();
        let input = line
            .split(',')
            .into_iter()
            .map(|x| x.parse::<u32>().unwrap_or_default())
            .collect::<Vec<_>>();
        (Some(input[0]), input.get(1).copied().unwrap_or_default())
    } else {
        File::create(&format!("{AUTOSIGN_DIR}/last_id")).unwrap();
        (None, 0)
    };

    let excuse_width = 469;
    let excuse_start = (370, 950);

    let date_start_start = 170;
    let date_end_start = 270;
    let box_height = 48;
    let hours_wh = (30, 96);
    let hours_start = (165, 323);

    let x1 = excuse_start.0 + excuse_width;

    let signature =
        image::open(std::env::var("SIGNATURE").unwrap_or("test_signature.png".to_owned())).unwrap();
    let signature = signature.resize_exact(
        255,
        box_height as u32,
        image::imageops::FilterType::CatmullRom,
    );

    // consider a monospace font
    let font = Vec::from(include_bytes!("../Roboto-Regular.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    let inteded_text_height = 14.4;
    let scale = Scale {
        x: inteded_text_height * 2.0,
        y: inteded_text_height,
    };

    let month_to_month_offset = HashMap::from([
        (2u32, 1),
        (3, 2),
        (4, 3),
        (5, 4),
        (6, 5),
        (7, 6),
        (9, 1),
        (10, 2),
        (11, 3),
        (12, 4),
        (1, 5),
    ]);

    let mut polled_last_id = 0;
    let mut updated = false;

    let img = image::open(format!("{AUTOSIGN_DIR}/out.png")).unwrap();
    let mut img = img.resize(1261, 1784, image::imageops::FilterType::CatmullRom);

    let mut running_offset = offset;
    let mut idx = 0;
    for absence in absences.iter() {
        if absence.is_excused {
            continue;
        }
        if absence.id <= last_id.unwrap_or_default() as u64 {
            continue;
        }
        if (idx + 1) % 15 == 0 {
            img.save(format!("{idx}_out.png")).unwrap();
            std::fs::write(&format!("{AUTOSIGN_DIR}/out.png"), FEHLSTUNDEN_FILE_DATA).unwrap();
            img = image::open(format!("{AUTOSIGN_DIR}/out.png")).unwrap();

            running_offset = 0;
        }
        let i = running_offset;
        let month = (absence.start_date / 100) % 100;
        let month_offset = month_to_month_offset[&month];
        let day = absence.start_date % 100;
        if absence.hours_absent_estimate() > 0 {
            imageproc::drawing::draw_text_mut(
                &mut img,
                Rgba([70u8, 40, 30, 255]),
                hours_start.0 + (day - 1) as i32 * hours_wh.0,
                hours_start.1 + hours_wh.1 * (month_offset - 1),
                scale,
                &font,
                &absence.hours_absent_estimate().to_string(),
            );
        }

        imageproc::drawing::draw_text_mut(
            &mut img,
            Rgba([70u8, 40, 30, 255]),
            date_start_start,
            excuse_start.1 + box_height * (i + 1) as i32,
            scale,
            &font,
            &absence.start_date(),
        );
        imageproc::drawing::draw_text_mut(
            &mut img,
            Rgba([70u8, 40, 30, 255]),
            date_end_start,
            excuse_start.1 + box_height * (i + 1) as i32,
            scale,
            &font,
            &absence.end_date(),
        );

        let mut reason_text = absence.reason_text.clone();
        if absence.reason_text.is_empty() {
            print!(
                "Missing reason for absence starting at {} taking {} hours. Add reason: ",
                absence.start_date(),
                absence.hours_absent_estimate()
            );
            io::stdout().flush().unwrap();
            let mut input_reason = String::new();
            io::stdin().lock().read_line(&mut input_reason).unwrap();
            reason_text = input_reason.trim_end().to_string();
        }

        imageproc::drawing::draw_text_mut(
            &mut img,
            Rgba([70u8, 40, 30, 255]),
            excuse_start.0,
            excuse_start.1 + box_height * (i + 1) as i32,
            scale,
            &font,
            &reason_text,
        );

        for row in 0..box_height {
            for col in 0..255 {
                img.draw_pixel(
                    x1 as u32 + col,
                    (excuse_start.1 + box_height * (i + 1) as i32 + row) as u32,
                    signature.get_pixel(col, row as u32),
                );
            }
        }

        polled_last_id = absence.id;
        running_offset += 1;
        idx += 1;
        updated = true;
    }

    if updated {
        let mut file = File::create(&format!("{AUTOSIGN_DIR}/last_id")).unwrap();
        file.write_all(format!("{polled_last_id},{running_offset}").as_bytes())
            .unwrap();
        file.flush().unwrap();
        img.save(format!("{AUTOSIGN_DIR}/out.png")).unwrap();
        img.save(format!("last_out.png")).unwrap();
    }
}
