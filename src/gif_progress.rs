use std::time::Instant;
use console::Style;
use gifski::progress::ProgressReporter;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};

pub struct GifProgress {
	pb: ProgressBar,
	start: Instant,
}

pub fn create_style() -> ProgressStyle {
	ProgressStyle::with_template("{prefix:>12.cyan.bold.dim} [{bar:57}] {pos}/{len}: {wide_msg}")
		.unwrap()
		.progress_chars("=> ")
}

impl GifProgress {
	pub fn new(step: u8, name: String, length: u64) -> GifProgress {
		let progress_style = create_style();
		let pb = ProgressBar::new(length);
		pb.set_style(progress_style.clone());
		pb.set_prefix(format!("[{}/5]", step));
		pb.set_message(name.clone());

		GifProgress {
			pb,
			start: Instant::now(),
		}
	}

	pub fn multi(m: &MultiProgress, step: u8, name: String, length: u64) -> GifProgress {
		let progress_style = create_style();
		let pb = m.add(ProgressBar::new(length));
		pb.set_style(progress_style.clone());
		pb.set_prefix(format!("[{}/5]", step));
		pb.set_message(name.clone());

		GifProgress {
			pb,
			start: Instant::now(),
		}
	}
}

impl ProgressReporter for GifProgress {
	fn increase(&mut self) -> bool {
		self.pb.inc(1);
		true
	}

	fn done(&mut self, msg: &str) {
		let green_bold = Style::new().green().bold();
		self.pb.finish_and_clear();
		let line = format!(
			"{:>12} {} in {}",
			green_bold.apply_to("Finished"),
			msg,
			HumanDuration(self.start.elapsed()),
		);
		self.pb.println(line);
	}
}