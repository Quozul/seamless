mod entry;
mod gaussian;
mod gif_progress;
mod seamless_fast;
mod similarity;

use crate::entry::get_image_bytes;
use crate::gaussian::gaussian;
use crate::seamless_fast::seamless_fast;
use crate::similarity::similarity;
use clap::{arg, Parser, Subcommand, ValueEnum};
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum CompareAlgorithm {
    NormalizedEuclideanDistance,
}

impl std::fmt::Display for CompareAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Make a seamless looping gif by finding the most similar frames
    Fast {
        /// Where all frames are stored
        path: PathBuf,

        /// Frames' file extension
        #[arg(short, long, default_value = "png")]
        extension: String,

        /// Whether duration is more important than similarity
        /// A too small value might result in a gif on 1 frame
        #[arg(short, long, default_value = "0.5")]
        duration_importance: f32,

        /// Quality of the gif encoding, from 0 to 100
        #[arg(short, long, default_value = "90", value_parser = clap::value_parser ! (u8).range(0..100))]
        quality: u8,

        /// Output gif file
        #[arg(short, long, default_value = "output.gif")]
        output: String,
    },

    Compare {
        /// The first image to compare
        source: PathBuf,

        /// The second image to compare
        target: PathBuf,

        /// Algorithm to use
        #[arg(
            long,
            short,
            default_value_t = CompareAlgorithm::NormalizedEuclideanDistance
        )]
        algorithm: CompareAlgorithm,
    },

    /// Blurs the image using gaussian blur
    Gaussian {
        input: String,

        /// Must not be 0
        #[arg(long, short, default_value = "3")]
        radius: u32,

        /// Must not be 0
        #[arg(long, short, default_value = "1.0")]
        sigma: f32,

        /// Must not be 0
        #[arg(long, short, default_value = "output.png")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {}
        Some(Commands::Fast {
            path,
            extension,
            duration_importance,
            quality,
            output,
        }) => {
            seamless_fast(
                path,
                OsStr::new(extension.as_str()),
                duration_importance,
                quality,
                output,
            );
        }
        Some(Commands::Compare {
            source,
            target,
            algorithm,
        }) => {
            let source_file = get_image_bytes(source);
            let target_file = get_image_bytes(target);

            match algorithm {
                CompareAlgorithm::NormalizedEuclideanDistance => {
                    let result = similarity(&*source_file, &*target_file);
                    println!("These two images are {}% similar.", result * 100.0);
                }
            }
        }
        Some(Commands::Gaussian {
            input,
            radius,
            sigma,
            output,
        }) => {
            gaussian(input, radius * 2, sigma, output);
        }
    }
}
