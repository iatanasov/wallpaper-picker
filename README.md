# wallpaper-picker

Simple utility to rotate wallpapers using feh or other utility


# Building

```
cargo build --release
```
## Format
```
cargo fmt
```

# Installation

```
cargo install  --path ./

```

# Usage
```
Usage: wallpaper-picker [OPTIONS] --image-paths [<IMAGE_PATHS>...]


Options:
  -i, --image-paths [<IMAGE_PATHS>...]    List of directories where you can find images
  -c, --command <DIR>                     Binary to execute [default: feh]
      --command-args [<COMMAND_ARGS>...]  [default: --no-fehbg --bg-scale]
  -s, --sleep <SECONDS>                   Sleep time [default: 7200]
  -r, --rotate                            Rotate immediatley and exit
  -h, --help                              Print help information
```
