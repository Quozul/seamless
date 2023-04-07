mod gif_progress;
mod entry;

use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::sync::{Arc, Mutex};
use clap::Parser;
use gifski::{Repeat, Settings};
use gifski::progress::ProgressReporter;
use indicatif::{MultiProgress, ProgressBar};
use rayon::prelude::*;
use crate::entry::Entry;
use crate::gif_progress::{create_style, GifProgress};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./")]
    path: String,
    #[arg(short, long, default_value = "png")]
    extension: String,
    #[arg(short, long, default_value = "0.5")]
    distance: f32,
    #[arg(short, long, default_value = "90")]
    quality: u8,
    #[arg(short, long, default_value = "output.gif")]
    output: String,
}

// Normalized euclidean distance
// Thanks ChatGPT
fn similarity(a: &[u8], b: &[u8]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len() {
        let diff = (a[i] as f32 - b[i] as f32) / 255.0;
        sum += diff * diff;
    }
    1.0 - (sum / a.len() as f32).sqrt()
}

struct SimilarityScore {
    start: usize,
    end: usize,
    similarity: f32,
    score: f32,
}

fn main() {
    let args = Args::parse();
    let extension = OsStr::new(args.extension.as_str());

    let progress_style = create_style();
    let mut indexing_progress = GifProgress::new(1, String::from("indexing"), 1);

    let mut paths = fs::read_dir(args.path)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_file() && path.extension().unwrap() == extension {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    paths.sort_by(|a, b| a.partial_cmp(b).unwrap());

    indexing_progress.done(&*format!("indexed {} files", paths.len()));

    let input = paths.iter()
        .map(|path| {
            Arc::new(Entry {
                path: Box::new(path.clone()),
                ..Entry::default()
            })
        })
        .collect::<Vec<_>>();

    let files = Arc::new(input);
    let length = paths.len();

    let files_b = files.clone();

    let m = MultiProgress::new();
    let loading_files_progress = Arc::new(Mutex::new(GifProgress::multi(&m, 2, String::from("loading files"), length as u64)));

    let load_files_thread = std::thread::spawn(move || {
        files_b
            .par_iter()
            .for_each(|element| {
                loading_files_progress.lock().unwrap().increase();
                element.load();
            });

        loading_files_progress.lock().unwrap().done(&*format!("loaded {} files", length));
    });

    let diff_pb = Arc::new(Mutex::new(GifProgress::multi(&m, 3, String::from("finding matching frames"), length as u64)));

    // TODO: Compare on GPU
    // TODO: Start while loading files
    let mut matches = files
        .par_iter()
        .enumerate()
        .map(|(i, element)| {
            let pb = m.add(ProgressBar::new((length - i) as u64));
            pb.set_style(progress_style.clone());

            pb.set_message(format!("comparing {} (waiting)", i));
            let bytes_a = element.wait_ready().get_data().unwrap();

            diff_pb.lock().unwrap().increase();

            let mut best_difference: f32 = 0.0;
            let mut best_difference_index: usize = 0;
            let mut best_score: f32 = 0.0;

            for j in (i + 1)..length {
                pb.set_message(format!("comparing {} and {} (waiting)", i, j));
                pb.inc(1);

                let bytes_b = files[j].wait_ready().get_data().unwrap();

                if bytes_b.len() != bytes_a.len() {
                    continue;
                }

                pb.set_message(format!("comparing {} and {}", i, j));

                let difference = similarity(&bytes_a, &bytes_b);

                // TODO: Update this formula
                let distance = ((j as f32 - i as f32) / length as f32).sqrt();
                let score = distance * args.distance + difference * (1.0 - args.distance);

                if score > best_score {
                    best_difference = difference;
                    best_difference_index = j;
                    best_score = score;
                }
            }

            pb.finish_and_clear();

            SimilarityScore {
                start: i,
                end: best_difference_index,
                similarity: best_difference,
                score: best_score,
            }
        })
        .collect::<Vec<_>>();

    load_files_thread.join().unwrap();

    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    diff_pb.lock().unwrap().done("calculating differences");

    let best = matches.first().unwrap();
    let start = best.start;
    let end = best.end;
    let fps = 1.0 / 24.0;
    let frame_count = (end - start) as u64;

    let (collector, writer) = gifski::new(Settings {
        width: None,
        height: None,
        quality: args.quality,
        fast: false,
        repeat: Repeat::Infinite,
    }).unwrap();

    let m = MultiProgress::new();
    let mut collect_files_progress = GifProgress::multi(&m, 4, String::from("collecting frames"), frame_count);

    let t = std::thread::spawn(move || {
        for i in start..end {
            collect_files_progress.increase();
            let path = &paths[i];
            collector.add_frame_png_file(i, path.to_path_buf(), fps * i as f64).unwrap();
        }

        collect_files_progress.done(&*format!("collected {} frames", frame_count));
    });

    let file = File::create(args.output).ok().unwrap();

    let mut reporter = GifProgress::multi(&m, 5, String::from("making the gif"), frame_count);
    writer.write(file, &mut reporter).unwrap();

    t.join().unwrap();
}
