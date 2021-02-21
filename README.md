# lesser ![CI](https://github.com/FedericoPonzi/lesser/workflows/CI/badge.svg)
A simple text reader, with less functionalities then less.

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
 * j: move down one row
 * k: move up one row
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
For enabling logging you can use the LESSER_LOG env variable, for instance:
```
LESSER_LOG=DEBUG cargo run -- /path/to/filename 2> /tmp/lesser.stderr
```
I would suggest to redirect stderr (or stdout) to a different output, otherwise the content of the file will clash with the printed log line.


## TODO:
* Ignore the new line at the end of the file (if there is any).
* If the output is redirected to anything other than a terminal, for example a pipe to another command, less behaves like cat. 
* Smarter Tab handling (now they're just filtered out).
* Implement more less's [functionalities](https://en.wikipedia.org/wiki/Less_(Unix)#Frequently_used_commands).
