# Documentation
The goal of this doc is not for the reader to understand the entire language, but rather get started writing some basic tests. As this project  is still in development, things are subject to change which might make certain parts of the documentation outdated.

## Writing a simple test
Every test has the exact same structure:
```javascript
test_name("command") { ... }
```
The command is the command to run the program you are trying to test. For example if I have a project called `hash` written in c with a main file called main.c that compiles to main, you would write "./main" instead of "command". The field can also include for example `make` or `java`.

### Input and Output
The language is designed to test IO. There is two buildings that handle IO, `input` and `output`.

#### Input
Syntax: `input(string)`
Sends the line string to the program being tested. Adds a new line at the end of the string. 

#### Output
Syntax: `output(string)`
Expect the next line of the programs Output to be string. Adds a new line at the end of the string.

### Builtins
#### Print
Syntax: `print(string)`
Print the string to the console. No extra newline.

#### Println
Syntax: `println(string)`
Print the string to the console. Adds a newline at the end of the string.