mod cli;
mod commands;
mod entry;
mod gif_progress;
mod similarity;

use crate::cli::{Cli, Commands, CompareAlgorithm};
use crate::entry::get_image_bytes;
use crate::similarity::similarity;
use clap::Parser;
use commands::borders::borders;
use commands::gaussian::gaussian;
use commands::seamless_fast::seamless_fast;
use std::ffi::OsStr;

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
        Some(Commands::Borders { input }) => {
            borders(input);
        }
    }
}
