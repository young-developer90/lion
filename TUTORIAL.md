# Lion Programming Language Tutorial

Lion is a simple, expressive scripting language. This tutorial covers everything you need to get started.

## Table of Contents

1. [Installation and Running](#installation-and-running)
2. [Comments](#comments)
3. [Variables](#variables)
4. [Data Types](#data-types)
5. [Operators](#operators)
6. [Strings and Interpolation](#strings-and-interpolation)
7. [Control Flow](#control-flow)
8. [Functions](#functions)
9. [Lambdas](#lambdas)
10. [Data Structures](#data-structures)
11. [Ranges](#ranges)
12. [Match Expressions](#match-expressions)
13. [Error Handling](#error-handling)
14. [Modules and Imports](#modules-and-imports)
15. [Built-in Standard Library](#built-in-standard-library)
16. [Full Example](#full-example)
17. [VS Code Setup](#vs-code-setup)

---

## Installation and Running

```bash
# Run a Lion source file
lion run hello.lion

# Start the interactive REPL
lion repl

# Show bytecode disassembly
lion run --disassemble hello.lion

# Format a source file
lion fmt hello.lion

# Run tests in a directory
lion test

# Run a specific test file
lion test tests/my_test.lion

# Show version
lion version
```

Inside the REPL:

```
Lion REPL v0.1.0
Type 'exit' to quit, 'help' for help.
lion> print("hello world")
hello world
=> nil
lion> 1 + 2
=> 3
```

---

## Comments

```lion
// Line comment

/*
 * Block comment
 */
```

---

## Variables

```lion
let x = 10;         // mutable variable
x = 20;             // reassign
x += 5;             // add-assign (also -=, *=, /=)

const y = 42;       // immutable constant

let z;              // declare without initializing (defaults to nil)
```

Optional type annotations are supported:

```lion
let name: String = "Lion";
let count: Int = 5;
```

---

## Data Types

| Type    | Examples                      |
|---------|-------------------------------|
| Int     | `42`, `-17`, `0`              |
| UInt    | (unsigned integer literals)   |
| Float   | `3.14`, `-0.5`, `2.0`        |
| String  | `"hello"`, `"""multi-line"""` |
| Bool    | `true`, `false`               |
| Nil     | `nil`                         |
| List    | `[1, 2, 3]`                   |
| Dict    | `{"a": 1, "b": 2}`            |
| Set     | `{1, 2, 3}`                   |
| Tuple   | `(1, "hello", 3.0)`           |
| Function| `func(x) { return x * 2 }`   |
| Lambda  | `\|x\| x * 2`                 |

```lion
print(42);         // Int
print(3.14);       // Float
print("hello");    // String
print(true);       // Bool
print(nil);        // Nil
```

---

## Operators

### Arithmetic

```lion
1 + 2       // add -> 3
5 - 3       // subtract -> 2
4 * 3       // multiply -> 12
10 / 3      // divide -> 3.333...
10 // 3     // integer division -> 3
10 % 3      // modulo -> 1
2 ** 10     // power -> 1024
```

### Comparison

```lion
1 == 1      // true
1 != 2      // true
2 < 3       // true
2 > 3       // false
2 <= 2      // true
3 >= 2      // true
```

### Containment (`in`)

```lion
1 in [1, 2, 3]       // true
4 in [1, 2, 3]       // false
"a" in {"a": 1}      // true (checks dict keys)
2 in {1, 2, 3}       // true (checks set membership)
"el" in "hello"      // true (substring check)
"x" in "hello"       // false
3 in 0..10           // true (range containment)
15 in 0..10          // false
```

### Chained Comparisons

Comparison operators can be chained: `a < b < c` is equivalent to `(a < b) and (b < c)`.

```lion
0 < 5 < 10           // true
0 < 10 < 5           // false
1 <= x <= 10         // check x is in range [1, 10]
```

Only `<`, `>`, `<=`, `>=` can be chained. `==`, `!=`, and `in` do not chain.

### Logical

```lion
true and false  // false
true or false   // true
not true        // false
```

### String Concatenation

```lion
"hello" + " world"   // "hello world"
"hello" .. " world"  // also concatenation
```

---

## Strings and Interpolation

```lion
let name = "Lion";
print(f"Hello, {name}!");   // Hello, Lion!
print(f"2 + 2 = {2 + 2}");  // 2 + 2 = 4
```

Multi-line strings use triple quotes:

```lion
let s = """this is
a multi-line
string""";
```

---

## Control Flow

### If / Elif / Else

```lion
let x = 10;

if (x > 5) {
    print("x is big");
} elif (x < 0) {
    print("x is negative");
} else {
    print("x is small");
}
```

Parentheses around conditions are optional:

```lion
if x > 5 {
    print("big");
}
```

### While Loop

```lion
let i = 0;
while (i < 5) {
    print(i);
    i = i + 1;
}
```

### For Loop

```lion
for i in 0..4 {
    print(i);   // prints 0, 1, 2, 3
}

let items = [10, 20, 30];
for item in items {
    print(item);
}
```

### Ternary Expression

```lion
let max = a > b ? a : b;
let result = x >= 0 ? "positive" : "negative";

// Nested ternaries
let label = x > 0 ? "pos" : x < 0 ? "neg" : "zero";
```

### Break and Continue

```lion
for i in 0..10 {
    if i == 3 {
        continue;   // skip i == 3
    }
    if i == 7 {
        break;      // stop at i == 7
    }
    print(i);
}
```

---

## Functions

```lion
func add(a, b) {
    return a + b;
}

print(add(3, 4));   // 7
```

Functions without a return statement return `nil`:

```lion
func greet(name) {
    print(f"Hello, {name}!");
}

let result = greet("Lion");   // prints "Hello, Lion!"
print(result);                 // nil
```

Functions are closures:

```lion
func make_counter(start) {
    let count = start;
    func inc() {
        count = count + 1;
        return count;
    }
    return inc;
}

let c = make_counter(0);
print(c());   // 1
print(c());   // 2
print(c());   // 3
```

Anonymous functions:

```lion
let double = func(x) { return x * 2 };
print(double(5));   // 10
```

Variadic functions (using `...`):

```lion
func sum(...) {
    // receives arguments as a list
}
```

Named arguments:

```lion
func draw_rect(width, height, color) {
    print(f"Drawing {color} rect {width}x{height}");
}

// Call with positional arguments
draw_rect(100, 50, "red");

// Call with named arguments (any order)
draw_rect(color = "blue", width = 200, height = 100);
```

---

## Lambdas

```lion
let double = |x| x * 2;
print(double(10));   // 20

// Multiple parameters
let add = |a, b| a + b;
print(add(3, 4));    // 7

// With map/filter style usage
let nums = [1, 2, 3, 4, 5];
```

---

## Data Structures

### Lists

```lion
let list = [1, 2, 3, 4, 5];

print(list[0]);      // 1 (zero-indexed)
print(list[-1]);     // 5 (negative index from end)
print(list.len());   // 5

list.push(6);        // [1, 2, 3, 4, 5, 6]
let last = list.pop();  // 6
print(list);         // [1, 2, 3, 4, 5]
```

### Dicts

```lion
let dict = {"a": 1, "b": 2, "c": 3};

print(dict["a"]);         // 1
dict["d"] = 4;            // add/modify entry
print(dict.contains("a"));// true
print(dict.keys());       // list of keys
```

### Sets

```lion
let s = {1, 2, 3};

s.insert(4);
print(s.contains(2));     // true
s.remove(2);
print(s);                 // {1, 3, 4}
```

### Tuples

```lion
let t = (1, "hello", 3.0);
print(t[0]);              // 1
```

---

## Ranges

```lion
// Range from 0 to 4 (exclusive end)
for i in 0..4 {
    print(i);   // 0, 1, 2, 3
}

// Ranges can be used elsewhere too
let r = 1..10;

// Range with step: start..step..end
for i in 0..2..10 {
    print(i);   // 0, 2, 4, 6, 8
}

for i in 10..-3..0 {
    print(i);   // 10, 7, 4, 1
}
```

---

## Match Expressions

```lion
let x = 3;

match x {
    1 => print("one"),
    2 => print("two"),
    3 => {
        print("three");
    },
    _ => print("other"),
}
```

---

## Error Handling

```lion
throw "something went wrong";

try {
    let result = risky_operation();
    print(result);
} catch err {
    print(f"caught: {err}");
}
```

---

## Modules and Imports

```lion
// Import an entire module
import math;
print(math.sqrt(16));   // 4.0

// Import specific symbols
from math import sqrt, pow;
print(sqrt(16));        // 4.0

// Import with alias
import math as m;
print(m.sqrt(16));

// Import symbol with alias
from math import sqrt as sq;
print(sq(16));
```

### Exporting

```lion
// Export a function
export func greet(name) {
    print(f"Hello, {name}!");
}

// Export a constant
export const PI = 3.14159;

// Export specific names from a list
export { greet, PI };
```

---

## Built-in Standard Library

Lion comes with several built-in modules:

### io

```lion
print("hello");        // print with newline
io.println("hello");   // same as print
input("Enter: ");      // read user input with prompt
```

### math

```lion
math.sqrt(16);       // 4.0
math.pow(2, 10);     // 1024.0
math.abs(-5);        // 5.0
math.sin(0);         // 0.0
math.cos(0);         // 1.0
math.tan(0);         // 0.0
math.pi;             // 3.14159...
math.e;              // 2.71828...
```

### time

```lion
time.now();          // current timestamp (Float, seconds since epoch)
time.unix();         // current Unix timestamp (Int)
time.sleep(1000);    // sleep for 1000ms (1 second)
```

### rand

```lion
rand.int(1, 100);     // random Int between 1 and 100
rand.float();         // random Float between 0.0 and 1.0
rand.choice([1, 2, 3]);// random element from list
```

### fs

```lion
fs.read("file.txt");     // read file contents as string
fs.write("file.txt", "hello");  // write text to file
fs.exists("file.txt");   // check if file exists (returns Bool)
fs.mkdir("new_dir");     // create directory
```

### os

```lion
os.cwd();             // current working directory
os.name();            // operating system name
os.args();            // command line arguments (list)
os.getenv("HOME");    // environment variable value
```

---

## Full Example

```lion
func fibonacci(n) {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

func main() {
    print("Fibonacci sequence:");
    for i in 0..10 {
        print(f"fib({i}) = {fibonacci(i)}");
    }

    let nums = [3, 7, 1, 9, 4];
    print(f"Sum: {sum(nums)}");

    let doubled = map(nums, |x| x * 2);
    print(f"Doubled: {doubled}");
}

func sum(list) {
    let total = 0;
    for n in list {
        total = total + n;
    }
    return total;
}

func map(list, fn) {
    let result = [];
    for item in list {
        result.push(fn(item));
    }
    return result;
}
```

Save this as `example.lion` and run:

```bash
lion run example.lion
```

---

## VS Code Setup

Lion provides a VS Code extension with syntax highlighting and a language server for diagnostics and completions.

### Installing the Extension

```bash
# 1. Install the LSP server (from the project root)
cargo build --bin lion-lsp

# 2. Install extension dependencies
cd vscode-lion
npm install
cd ..

# 3. Link the extension to VS Code
code --install-extension vscode-lion/
```

If `code` is not on PATH, open VS Code, open the Extensions panel (Ctrl+Shift+X), click `...` → `Install from VSIX...` or point to the `vscode-lion/` folder.

### LSP Features

The `lion-lsp` server provides:

- **Diagnostics** — syntax errors are underlined as you type
- **Completions** — keyword and built-in function suggestions
- **Hover info** — shows position info (line/column)

The LSP starts automatically when you open a `.lion` file in VS Code. It reuses the same parser as the `lion` CLI, so diagnostic messages match exactly.

### Testing the LSP Manually

```bash
# Run the LSP directly (it reads JSON-RPC on stdin)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"capabilities":{}}}' | target/debug/lion-lsp.exe
```

### Troubleshooting

| Problem | Fix |
|---------|-----|
| "lion-lsp not found" error | Run `cargo build --bin lion-lsp` in the project root |
| No syntax highlighting | Open a `.lion` file and check the bottom-right shows "Lion" as the language mode |
| LSP not starting | Open the VS Code Output panel (Ctrl+Shift+U) and select "Lion Language Server" from the dropdown |
