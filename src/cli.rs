use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
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
        #[arg(short, long, default_value = "90", value_parser = clap::value_parser ! (u8).range(0..=100))]
        quality: u8,

        /// Output gif file
        #[arg(short, long, default_value = "output.gif")]
        output: PathBuf,
    },

    /// Compares two images
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
        input: PathBuf,

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

    /// Gives the average color of the first and last rows
    Borders { input: PathBuf },
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompareAlgorithm {
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
