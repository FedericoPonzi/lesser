# lesser ![CI](https://github.com/FedericoPonzi/lesser/workflows/CI/badge.svg)
A simple text reader, with lesser functionalities then less.

## Usage:
Build it with:
```bash
cargo build --release
```
Then you can use:
```
# Read a file:
lesser /path/to/filename
# help:
lesser --help
```
Move up or down with arrows, and exit with `q` or `CTRL^C`.

Alternatively:
```
cargo run -- /path/to/filename 
```

## TODO:
* Handle SIGWINCH signal for redrawing the screen
* Support pipes
* Implement more less's functionalities.
