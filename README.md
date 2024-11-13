# Test script
A language to write io test for your program.

## Installation
```bash
curl -s --compressed "https://huggepugge1.github.io/apt/KEY.gpg" | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/apt.gpg >/dev/null
sudo curl -s --compressed -o /etc/apt/sources.list.d/huggepugge1.list "https://huggepugge1.github.io/apt/huggepugge1.list"
sudo apt update
sudo apt install test-script
```

To update, run `sudo apt update && sudo apt install test-script`

## Building
Clone the repo and run `cargo build --release`.

## Usage
### Using apt
After installing the package, you can run the program by using `test-script [file name]`.

### Using cargo
Run the program by either doing `cargo run -- [file name]` if you are in this repo or running the binary directly using `path/to/test-script [file name]`.

## Help
Read documentation.md and look at the examples. If you need help with using the interpreter, use `./test_script --help`.
