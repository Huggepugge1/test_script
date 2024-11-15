# Documentation
The goal of this doc is not for the reader to understand the entire language, but rather get started writing some basic tests. As this project  is still in development, things are subject to change which might make certain parts of the documentation outdated.

## Writing a simple test
Every test has the exact same structure:
```javascript
test_name("command") { ... }
```
The command is the command to run the program you are trying to test. For example if I have a project called `hash` written in c with a main file called main.c that compiles to main, you would write "./main" instead of "command". The field can also include for example `make` or `java`.

## Types
There are four types, `string`, `regex`, `int` and `bool`.

### Type casting
To cast a type to another, use the `as` keyword.
#### Syntax
`a as T`<br>

## Variables
Variables are declared with the `let` keyword.
Constants are declared with the `const` keyword.
A constant cannot be reassigned.
All variables must be declared with a type.

### Example
`let a: string = "Hello, World!";`<br>
`const b: int = 42;`<br>

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

## Conditionals
The only conditional available is the if/else statement.

### If
If the condition is true, run the next statement.
All conditions must be of type `bool`.
#### Syntax
`if condition { ... }`<br>

### Else
If the condition was false, run the next statement.
#### Syntax
`if condition { ... } else { ... }`<br>

### Else if
If the condition was false, check the next condition.
#### Syntax
`if condition { ... } else if condition { ... }`<br>

## Loops
The only loop available is the for loop.

### For
For each element in the iterable, name it var_name and run the next statement.
#### Syntax
`for var_name: var_type in iterable { ... }`<br>

## Iterables
The only iterable available is the regular expression (regex).

### Regex
Creates an iterable containing all the different combinations that the Regex matches.
Note: The star operation repeats `0-max_len` inclusive times.
`max_len` is set by the command line argument `--max-len`.
#### Syntax
```
`regular expression`
```

#### Example
```
`\\d`
```
would create an iterable containg all digits 0-9.

## Operators
### Assignment
Assigns the value of b to a. The let keyword is not needed when assigning a value to a variable that has already been defined.
#### Syntax
`a = b`<br>

### Plus
Concatenates the two strings a and b.
#### Syntax
`a + b`<br>
#### Supported types
`string` + `string`<br>
`int` + `int`<br>

### Minus
Subtracts b from a.
#### Syntax
`a - b`<br>
#### Supported types
`int` - `int`<br>

### Multiply
Multiplies a and b.
#### Syntax
`a * b`<br>
#### Supported types
`int` * `int`<br>
`string` * `int`<br>

### Divide
Divides a by b.
#### Syntax
`a / b`<br>
#### Supported types
`int` / `int`<br>

### Equal
Checks if a is equal to b.
#### Syntax
`a == b`<br>
#### Supported types
`int` == `int`<br>
`string` == `string`<br>
`bool` == `bool`<br>

### Not equal
Checks if a is not equal to b.
#### Syntax
`a != b`<br>
#### Supported types
`int` != `int`<br>
`string` != `string`<br>
`bool` != `bool`<br>

### Greater than
Checks if a is greater than b.
#### Syntax
`a > b`<br>
#### Supported types
`int` > `int`<br>

### Less than
Checks if a is less than b.
#### Syntax
`a < b`<br>
#### Supported types
`int` < `int`<br>

### Greater than or equal
Checks if a is greater than or equal to b.
#### Syntax
`a >= b`<br>
#### Supported types
`int` >= `int`<br>

### Less than or equal
Checks if a is less than or equal to b.
#### Syntax
`a <= b`<br>
#### Supported types
`int` <= `int`<br>

### And
Checks if a and b are both true.
#### Syntax
`a && b`<br>
#### Supported types
`bool` && `bool`<br>

### Or
Checks if a or b is true.
#### Syntax
`a || b`<br>
#### Supported types
`bool` || `bool`<br>

### Not
Checks if a is false.
#### Syntax
`!a`<br>
#### Supported types
`!bool`<br>

## Comments
Comments are written by `//`.
