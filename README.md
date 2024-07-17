# wallpaper-picker

Simple utility to rotate wallpapers using feh or other utility
By default it is using feh which is working well with managers like dwm and LeftWM

# Building

```
cargo build --release
```
## Format
```
cargo fmt
```

# Installation

## Local instalation

```
cargo install  --path ./

```

## system location
```
cargo install --path ./ --root /usr/bin
```

# Enable in systemd 
Copy  [service/wallpaper-picker.service](service/wallpaper-picker.service) to /$HOME/.config/systemd/user/ .
Configuration can be done in file

Execute
```
systemctl --user enable wallpaper-picker
```
# Usage
```
Usage: wallpaper-picker [OPTIONS]

Options:
  -i, --image-paths [<IMAGE_PATHS>...]
          List of directories where you can find images Configurable in the configuration file
  -c, --command <DIR>
          Binary to execute Configurable in the configuration file [default: /usr/bin/feh]
      --config <DIR>

      --command-args [<COMMAND_ARGS>...]
          Configure the command that will set the wallpaper Configurable in the configuration file [default: --no-fehbg --bg-scale]
      --image-extentions [<IMAGE_EXTENTIONS>...]
          [default: png jpg]
  -s, --sleep <SECONDS>
          Sleep time Configurable in the configuration file [default: 7200]
  -r, --rotate
          Rotate immediatley and exit
  -f, --force-duplicate
          Force duplicate process
  -o, --only-print
          Only print the image path to the standard out
  -h, --help
          Print help information
```
