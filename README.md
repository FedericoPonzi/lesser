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
# Pipe a file:
cat file | lesser
```
### Commands:
 * h: move left one column
 * l: move right one column
 * j: move down one page
 * k: move up one page
 * Down arrow: Move down one page
 * Up arrow: Move up one page
 * Left arrow: Move left one page
 * Right arrow: Move right one page.
 * Ctrl + C, q: Exit.
 
---

You can also run it with cargo using:
```
cargo run -- /path/to/filename 
```
### Development
For showing logs:
```
LESSER_LOG=DEBUG cargo run -- /path/to/filename 2> /tmp/lesser.stderr
```
You need to redirect stderr to some file, otherwise the content of the file will override the printed log line.


## TODO:
* Ring bell: print!("\x07") when user tries to scroll out of the screen.
* Implement more less's [functionalities](https://en.wikipedia.org/wiki/Less_(Unix)#Frequently_used_commands).
* If the output is redirected to anything other than a terminal, for example a pipe to another command, less behaves like cat. 