# Documentation
The goal of this doc is not for the reader to understand the entire language, but rather get started writing some basic tests. As this project  is still in development, things are subject to change which might make certain parts of the documentation outdated.

## Writing a simple test
Every test has the exact same structure:
```javascript
test_name("command") { ... }
```
The command is the command to run the program you are trying to test. For example if I have a project called `hash` written in c with a main file called main.c that compiles to main, you would write "./main" instead of "command". The field can also include for example `make` or `java`.

## Types
There are three types, `string`, `regex` and `int`.

### Type casting
To cast a type to another, use the `as` keyword.
#### Syntax
`a as T`<br>

## Variables
Variables are declared with the `let` keyword. They need to have a known type at compile time. At this time the only effect of the type is visual, as the language is not type checked yet.

### Example
`let a: string = "Hello, World!"`;

## Blocks
Blocks are defined by curly braces `{ ... }`.
They are used to group statements together.
The block itself is a statement that returns the value of the last statement in the block.
A block also creates a new scope meaning that any variable declared inside the block is not accessible outside the block.

## Input and Output
The language is designed to test IO. There is two builtins that handle IO, `input` and `output`.

### Input
Sends the line string to the program being tested. Adds a new line at the end of the string. 
#### Syntax
`input(string)`<br>

### Output
Expect the next line of the programs Output to be string. Adds a new line at the end of the string.
#### Syntax
`output(string)`<br>

## Builtins
### Print
Print the string to the console. No extra newline.
#### Syntax
`print(string)`<br>

### Println
Print the string to the console. Adds a newline at the end of the string.
#### Syntax
`println(string)`<br>

## Loops
The only loop available is the for loop.

### For
For each element in the iterable, name it var_name and run the next statement.
#### Syntax
`for var_name: var_type in iterable { ... }`<br>

## Iterables
The only iterable available is the regular expression (regex).

### Regex
Creates an iterable containing all the different combinations that the Regex matches. Note: The star operation repeats `0-max_len` inclusive times. `max_len` is set by the command line argument `--max-len`.
#### Syntax
`/regular expression/`<br>

#### Example
/\d/ would create an iterable containg all digits 0-9.

## Operators
There are only two operators, `=` and `+`.

### Assignment
Assigns the value of b to a. The let keyword is not needed when assigning a value to a variable that has not yet been defined.
#### Syntax
`a = b`<br>

### Plus
Concatenates the two strings a and b.
#### Syntax
`a + b`<br>
#### Supported types
`string` + `string`<br>
`int` + `int`<br>

## Comments
Comments are written by `//`.
