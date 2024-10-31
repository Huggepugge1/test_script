# Test script
A language to write io test for your program.

## Usage
Either be in the the test script main dir and run `cargo run -- <FILENAME>` where FILENAME is the name of you .tesc file. Another way is to directly run the binary by doing `cargo build --release` and then `path/to/test_script/release/test_script <FILENAME>`. Releases are coming soon to simplify the installation process.

## Plans
The `!` and `?` operator.
If the `!` or `?` is found in a `new` test, they are ommited and a warning is displayed.
A `!` followed by a statement means that the statement is ran before the iterative test has begun. This allows for initialization.
A `?` followed by a statement means that the statement is ran after the iterative test has finished. Tihs allows for quitting a program.

The `new` keyword.
If the `new` keyword is used, the test should start a new child for every test. If the `new` keyword is not used, the test should run in the same child. Should increase test speed.

The `let` keyword together with `=` and `+` to handle assignment of variables to make the code cleaner.
