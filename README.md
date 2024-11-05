# Test script
A language to write io test for your program.

## Usage
Either be in the the test script main dir and run `cargo run -- <FILENAME>` where FILENAME is the name of you .tesc file. Another way is to directly run the binary by doing `cargo build --release` and then `path/to/test_script/release/test_script <FILENAME>`. Releases are coming soon to simplify the installation process.

## Plans
Regex support in loops. Syntax `/regex/`.
The `for` loop. Syntax: `for reg in /regex/ {...}`.
