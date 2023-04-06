mod gif_progress;

use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use clap::Parser;
use console::Style;
use gifski::{Repeat, Settings};
use gifski::progress::ProgressReporter;
use indicatif::{HumanDuration, MultiProgress, ProgressBar};
use rayon::prelude::*;
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

fn get_image_bytes(path: &PathBuf) -> Vec<u8> {
	let decoder = png::Decoder::new(File::open(path).unwrap());
	let mut reader = decoder.read_info().unwrap();

	let mut buf = vec![0; reader.output_buffer_size()];
	reader.next_frame(&mut buf).unwrap();
	buf
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

	let mut loading_files_progress = Arc::new(Mutex::new(GifProgress::new(2, String::from("loading files"), paths.len() as u64)));

	let files = paths.par_iter()
		.map(|path| {
			loading_files_progress.lock().unwrap().increase();
			get_image_bytes(path)
		})
		.collect::<Vec<_>>();

	loading_files_progress.lock().unwrap().done(&*format!("loaded {} files", paths.len()));

	let length = files.len();

	let m = MultiProgress::new();
	let mut diff_pb = Arc::new(Mutex::new(GifProgress::multi(&m, 3, String::from("finding matching frames"), length as u64)));

	// TODO: Compare on GPU
	let mut matches = files
		.par_iter()
		.enumerate()
		.map(|(i, bytes_a)| {
			let pb = m.add(ProgressBar::new((length - i) as u64));
			pb.set_style(progress_style.clone());

			diff_pb.lock().unwrap().increase();

			let mut best_difference: f32 = 0.0;
			let mut best_difference_index: usize = 0;
			let mut best_score: f32 = 0.0;

			for j in (i + 1)..length {
				pb.set_message(format!("comparing {} and {}", i, j));
				pb.inc(1);

				let bytes_b = &files[j];
				if bytes_b.len() != bytes_a.len() {
					continue;
				}

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

	// TODO: Use multi progress bar
	let mut collect_files_progress = GifProgress::new(4, String::from("collecting frames"), frame_count);

	let t = std::thread::spawn(move || {
		for i in start..end {
			collect_files_progress.increase();
			let path = &paths[i];
			collector.add_frame_png_file(i, path.to_path_buf(), fps * i as f64).unwrap();
		}

		collect_files_progress.done(&*format!("collected {} frames", frame_count));
	});

	let file = File::create(args.output).ok().unwrap();

	let mut reporter = GifProgress::new(5, String::from("making the gif"), frame_count);
	writer.write(file, &mut reporter).unwrap();

	t.join().unwrap();
}
