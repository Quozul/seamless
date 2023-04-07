mod gif_progress;
mod entry;
mod seamless_fast;

use std::ffi::OsStr;
use clap::{arg, Parser, Subcommand};
use crate::seamless_fast::seamless_fast;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Make a seamless looping gif by finding the most similar frames
    Fast {
        /// Where all frames are stored
        path: String,

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
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Fast { path, extension, duration_importance, quality, output }) => {
            seamless_fast(path, OsStr::new(extension.as_str()), duration_importance, quality, output);
        }
        None => {}
    }
}
