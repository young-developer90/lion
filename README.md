# Lion Programming Language

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-dea584?logo=rust)](https://rustup.rs/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.4.2-green)](Cargo.toml)

Lion is a simple, expressive scripting language with a Rust-based interpreter. It combines modern language features — closures, pattern matching, string interpolation, and a module system — with a lightweight bytecode VM and optional CUDA GPU acceleration.

## Quick Start

```bash
git clone https://github.com/young-developer90/lion.git
cd lion
cargo build --release
./target/release/lion run examples/hello.lion
```

## Features

| Category | Details |
|----------|---------|
| **Syntax** | Clean, modern — inspired by Swift, Kotlin, and Lua |
| **Functions** | First-class, closures, lambdas (`\|x\| x * 2`), variadic and named args |
| **Types** | Int, Float, String, Bool, List, Dict, Set, Tuple, ranges |
| **Strings** | Interpolation (`f"Hello, {name}!"`), multi-line (triple quotes) |
| **Control Flow** | `if`/`elif`/`else`, `while`, `for..in`, ternary `? :`, `match` |
| **Error Handling** | `try`/`catch`/`throw` |
| **Modules** | `import`/`export` with aliases |
| **Standard Library** | `math`, `time`, `rand`, `fs`, `os`, `json`, `csv`, `html`, `http`, `url` |
| **GPU** | Optional CUDA acceleration for matrix operations |
| **Tooling** | REPL, bytecode disassembler, formatter, test runner |
| **Editor Support** | VS Code extension with LSP (diagnostics, completions, hover) |
| **Cross-platform** | Windows, macOS, Linux |

## Usage

```bash
lion run <file>          # Run a script
lion repl                # Interactive REPL
lion run --disassemble   # Show bytecode
lion fmt <file>          # Format source
lion test [path]         # Run tests
lion version             # Show version
```

### Example

```lion
func fibonacci(n) {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

for i in 0..10 {
    print(f"fib({i}) = {fibonacci(i)}");
}
```

## Documentation

See [TUTORIAL.md](TUTORIAL.md) for a comprehensive guide covering all language features.

## Building

### Prerequisites

- [Rust](https://rustup.rs/) 1.80+ (edition 2021)

### Release build

```bash
cargo build --release
```

### LSP server (optional)

```bash
cargo build --bin lion-lsp
```

### VS Code extension

```bash
cd vscode-lion && npm install && cd ..
code --install-extension vscode-lion/
```

### CUDA support (optional)

Install the [CUDA Toolkit](https://developer.nvidia.com/cuda-downloads) and set `CUDA_PATH`. The build script detects it automatically.

## Running Tests

```bash
cargo build --release
./target/release/lion test tests/
```

## Project Structure

```
src/           # Rust source — lexer, parser, compiler, VM, stdlib, CLI, LSP
examples/      # Example .lion scripts
tests/         # Test .lion scripts
vscode-lion/   # VS Code extension
```

## License

MIT
