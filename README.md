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
* Implement single line / col scrolling.
* Ring bell: print!("\x07") when user tries to scroll out of the screen.
* Implement more less's [functionalities](https://en.wikipedia.org/wiki/Less_(Unix)#Frequently_used_commands).
* If the output is redirected to anything other than a terminal, for example a pipe to another command, less behaves like cat. 