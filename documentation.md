# Documentation
The goal of this doc is not for the reader to understand the entire language, but rather get started writing some basic tests. As this project  is still in development, things are subject to change which might make certain parts of the documentation outdated.

## Writing a simple test
Every test has the exact same structure:
```javascript
test_name("command") { ... }
```
The command is the command to run the program you are trying to test. For example if I have a project called `hash` written in c with a main file called main.c that compiles to main, you would write "./main" instead of "command". The field can also include for example `make` or `java`.

## Types
The types available are `string`, `regex`, `int` `float`, `bool`, `none`.

### Type casting
To cast a type to another, use the `as` keyword.
#### Syntax
`a as T`  

## Variables
Variables are declared with the `let` keyword.
Constants are declared with the `const` keyword.
A constant cannot be reassigned.
All variables must be declared with a type.

### Example
`let a: string = "Hello, World!";`  
`const B: int = 42;`  

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
`input(string)`  

### Output
Expect the next line of the programs Output to be string. Adds a new line at the end of the string.
#### Syntax
`output(string)`  

## Builtins
### Print
Print the string to the console. No extra newline.
#### Syntax
`print(string)`  

### Println
Print the string to the console. Adds a newline at the end of the string.
#### Syntax
`println(string)`  

## Conditionals
The only conditional available is the if/else statement.

### If
If the condition is true, run the next statement.
All conditions must be of type `bool`.
#### Syntax
`if condition { ... }`  

### Else
If the condition was false, run the next statement.
#### Syntax
`if condition { ... } else { ... }`  

### Else if
If the condition was false, check the next condition.
#### Syntax
`if condition { ... } else if condition { ... }`  

## Loops
The only loop available is the for loop.

### For
For each element in the iterable, name it var_name and run the next statement.
#### Syntax
`for var_name: var_type in iterable { ... }`  

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
`a = b`  

### Plus
Concatenates the two strings a and b.
#### Syntax
`a + b`  
#### Supported types
`string` + `string`  
`int` + `int`  
`float` + `float`  

### Minus
Subtracts b from a.
#### Syntax
`a - b`  
#### Supported types
`int` - `int`  
`float` - `float`  

### Multiply
Multiplies a and b.
#### Syntax
`a * b`  
#### Supported types
`int` * `int`  
`float` * `float`  
`string` * `int`  

### Divide
Divides a by b.
#### Syntax
`a / b`  
#### Supported types
`int` / `int`  
`float` / `float`  

### Equal
Checks if a is equal to b.
#### Syntax
`a == b`  
#### Supported types
`int` == `int`  
`float` == `float`  
`string` == `string`  
`bool` == `bool`  

### Not equal
Checks if a is not equal to b.
#### Syntax
`a != b`  
#### Supported types
`int` != `int`  
`float` != `float`  
`string` != `string`  
`bool` != `bool`  

### Greater than
Checks if a is greater than b.
#### Syntax
`a > b`  
#### Supported types
`int` > `int`  
`float` > `float`  

### Less than
Checks if a is less than b.
#### Syntax
`a < b`  
#### Supported types
`int` < `int`  
`float` < `float`  

### Greater than or equal
Checks if a is greater than or equal to b.
#### Syntax
`a >= b`  
#### Supported types
`int` >= `int`  
`float` >= `float`  

### Less than or equal
Checks if a is less than or equal to b.
#### Syntax
`a <= b`  
#### Supported types
`int` <= `int`  
`float` <= `float`  

### And
Checks if a and b are both true.
#### Syntax
`a && b`  
#### Supported types
`bool` && `bool`  

### Or
Checks if a or b is true.
#### Syntax
`a || b`  
#### Supported types
`bool` || `bool`  

### Not
Checks if a is false.
#### Syntax
`!a`  
#### Supported types
`!bool`  

## Functions
Functions are declared with the `fn` keyword.  

### Syntax
`fn function_name(arg1: type, arg2: type, ...): return_type { ... }`  

### Note
There is no return keyword.
Instead, the value of the last statement in the function is returned.

### Example
```javascript
fn min(a: int, b: int): int {
    if a < b {
        a;
    } else {
        b;
    }
}

fn fib(n: int): int {
    if n <= 1 {
        n;
    } else {
        fib(n - 1) + fib(n - 2);
    }
}
```

## Comments
Comments are written by `//`.

## Style Convention
### Naming
Variable names should be written in snake_case.
Constants should be written in UPPER_SNAKE_CASE.

### Indentation
Indentation should be 4 spaces.

### Line length
Lines should be at most 80 characters long.

### Curly braces
Curly braces should be on the same line as the statement.

### Conditionals and loops
There shoulde be no parentheses around the condition / declaration.

### Magic numbers
Magic numbers should be avoided.  
Use constants instead.  
Use the `-M` flag to disable magic number warnings.  

### Unused values
All values should be used.  
If a value is not used, the interpreter will warn about it.  
Allocate the value to `_` to indicate that it is not used and to tell the interpreter that it is intentional.  

### Unused variables
Unused variables should be avoided.  
Use the `_` character to indicate that a variable is unused.  
The interpreter will not warn about unused variables that start with an underscore.  
The interpreter will also warn if a variable is assigned but never used after the assignment.

### Constants
All values that are not reassigned should be declared as constants.  

### Example
```javascript
test("./main") {
    let integer_value: int = 42;
    const HELLO_WORLD: string = "Hello, World!";
    if integer_value > 0 {
        println(HELLO_WORLD);
    }
    for i: int in `\\d` {
        println(i);
    }
}
```
