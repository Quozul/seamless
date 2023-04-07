# Seamless

Create seamlessly looping gifs from the command line.

## Demo

`TODO: Add a demo`

## Installation

Clone the repo and install using cargo:

```bash
git clone git@github.com:Quozul/seamless.git
cargo install --path .
```

## Usage/Examples

Get a video that might loop and run the following commands:

```shell
# Get usage of the command
seamless --help

# Convert a video into png frames
ffmpeg -i video.mp4 frame%04d.png

# Find 2 frames that loops and generates a gif named `my_gif.gif`
seamless fast -e=png -o=my_gif.gif .
```
