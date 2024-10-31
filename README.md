# Test script
A language to write io test for your program.

## Usage
Either be in the the test script main dir and run `cargo run -- <FILENAME>` where FILENAME is the name of you .tesc file. Another way is to directly run the binary by doing `cargo build --release` and then `path/to/test_script/release/test_script <FILENAME>`. Releases are coming soon to simplify the installation process.

## Plans
The `new` keyword.
If the `new` keyword is used, the test should start a new child for every test. If the `new` keyword is not used, the test should run in the same child. Should increase test speed.

The `let` keyword together with `=` and `+` to handle assignment of variables to make the code cleaner.