# Lion Programming Language

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-dea584?logo=rust)](https://rustup.rs/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.7.0-green)](https://github.com/young-developer90/lion/releases)
[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/young-developer90/lion/actions)
[![PRs](https://img.shields.io/badge/PRs-welcome-orange)](https://github.com/young-developer90/lion/pulls)

Lion is a simple, expressive scripting language with a Rust-based interpreter (v1.7.0). It combines modern language features — closures, pattern matching, string interpolation, and a module system — with a lightweight bytecode VM, optional GPU acceleration, and a built-in project manager.

```
print("Hello, Lion!");
```

## Philosophy

- **Readable** — syntax inspired by Swift, Kotlin, and Lua. No sigils, no ceremony.
- **Expressive** — first-class functions, closures, pattern matching, ternaries, list comprehensions.
- **Approachable** — you can learn the whole language in an afternoon.
- **Self-contained** — batteries included: HTTP client, JSON/CSV/HTML parsers, stats, regex, datetime, logging, subprocess, hashlib, collections, itertools, unit testing, and native GUI toolkits for **Windows** (leopard) and **Linux** (panther).
- **Extensible** — module system with import/export, optional Python interop, optional CUDA GPU acceleration, and C extension API.

## Quick Start

```bash
git clone https://github.com/young-developer90/lion.git
cd lion
cargo build --release
./target/release/lion run examples/hello.lion
```

### Start the REPL

```bash
./target/release/lion repl
```

```
Lion> let x = 42;
Lion> print(f"the answer is {x}");
the answer is 42
Lion> func fib(n) { if n <= 1 { return n; } return fib(n-1) + fib(n-2); }
Lion> print(fib(20));
6765
```

## First Script

```bash
echo 'print("Hello, Lion!")' > hello.lion
./target/release/lion run hello.lion
```

## Examples

### Hello World

```lion
print("Hello, Lion!");
```

### Fibonacci

```lion
func fibonacci(n) {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
for i in 0..10 { print(f"fib({i}) = {fibonacci(i)}"); }
```

### HTTP Request

```lion
let resp = http.get("https://api.github.com/repos/young-developer90/lion");
print(resp.status);
print(resp.json()["description"]);
```

### File I/O

```lion
fs.write("hello.txt", "Hello, Lion!");
let content = fs.read("hello.txt");
print(content);                       // Hello, Lion!
print(fs.exists("hello.txt"));        // true
```

### Regular Expressions

```lion
let matches = re.find_all(r"\d+", "abc 123 def 456");
print(matches);                       // ["123", "456"]
let result = re.sub(r"\d+", "X", "abc 123 def 456");
print(result);                        // "abc X def X"
```

### JSON

```lion
let data = json.parse('{"name": "Lion", "version": 1.0}');
print(data["name"]);                  // Lion
let encoded = json.stringify(data);
print(encoded);
```

### GUI (Windows with leopard)

```lion
let root = leopard.Leo("App", 400, 300);
let label = leopard.Label(root, "Hello from Leopard!");
leopard.pack(label, "top", 0, 10);
let btn = leopard.Button(root, "Click", func() {
    leopard.config(label, "text", "Clicked!");
});
leopard.pack(btn, "top", 0, 5);
leopard.mainloop(root);
```

### GUI (Linux with panther)

Build with `--features panther` and GTK4 development libraries:

```bash
# Ubuntu/Debian
sudo apt install libgtk-4-dev
cargo build --release --features panther
```

```lion
let root = panther.Leo("App", 400, 300);
let label = panther.Label(root, "Hello from Panther!");
panther.pack(label);
let btn = panther.Button(root, "Click", func() {
    panther.config(label, "text", "Clicked!");
});
panther.pack(btn);
panther.mainloop(root);
```

Full text editor example at `examples/textedit.lion`.

### Python Interop

Build with `--features python`:

```bash
cargo build --release --features python
```

```lion
import py
let np = py.import("numpy")
let arr = np.array([1, 2, 3])
print(arr)  // [1 2 3]
```

## Language Tour

### Variables & Constants

```lion
let name = "Lion";           // mutable
const pi = 3.14159;          // immutable
let count = 42;              // Int
let price = 19.99;           // Float
let active = true;           // Bool
let data = nil;              // Nil
```

### Strings

```lion
let s = "hello";
let multi = """line one
line two
line three""";
let interpolated = f"value: {s}, sum: {2 + 2}";
let combined = "hello" + " world";  // concat with +
```

### Collections

```lion
let list = [1, 2, 3];  list.push(4);
let dict = {"name": "Lion", "version": 1.0};
let set = {1, 2, 3};  set.add(4);
let tuple = (1, "hello", true);
```

### Control Flow

```lion
if x > 0 { print("positive"); } elif x < 0 { print("negative"); } else { print("zero"); }
while count > 0 { count -= 1; }
for i in 0..10 { print(i); }
for i in 0..100..5 { print(i); }
let max = a > b ? a : b;
```

### Functions & Closures

```lion
func greet(name) { return f"Hello, {name}!"; }
let double = |x| x * 2;
func sum(...nums) { let t = 0; for n in nums { t += n; } return t; }
func connect(host, port = 8080) { print(f"{host}:{port}"); }

// closures
func make_counter(start) {
    let count = start;
    func inc() { count = count + 1; return count; }
    return inc;
}
let c = make_counter(0);
print(c());  // 1
print(c());  // 2
```

### Pattern Matching

```lion
match value {
    0 => print("zero"),
    1..10 => print("small"),
    42 => print("answer!"),
    _ => print("something else"),
}
```

### Error Handling

```lion
try { let result = risky_operation(); } catch e { print(f"caught: {e}"); }
throw "something went wrong";
```

### Structs

```lion
struct Counter {
    func new() { return Counter{ count = 0 }; }
    func increment(self) { self.count += 1; }
    func value(self) { return self.count; }
}
let c = Counter.new();
c.increment();
print(c.value());  // 1
```

### Modules

```lion
// import.lion
export func hello() { print("hi"); }

// main.lion
import "import.lion" as mymod;
mymod.hello();
```

### Comprehensions

```lion
let squares = [x * x for x in 0..10];       // [0, 1, 4, 9, 16, ...]
let evens   = [x for x in 0..20 if x % 2 == 0];
```

## Standard Library Reference

| Module | Description |
|--------|-------------|
| `math` | `abs`, `sqrt`, `sin`, `cos`, `tan`, `floor`, `ceil`, `round`, `min`, `max`, `pow`, `log`, `pi`, `e` |
| `time` | `sleep`, `now` |
| `rand` | `int`, `float`, `shuffle`, `choice` |
| `fs` | `read`, `write`, `append`, `exists`, `remove`, `mkdir`, `read_dir`, `stat`, `copy`, `rename` |
| `os` | `args`, `env`, `set_env`, `cwd`, `chdir`, `exit`, `platform`, `type` |
| `json` | `parse`, `stringify`, `load`, `dump`, `pretty` |
| `csv` | `parse`, `stringify`, `load`, `save`, `parse_header` |
| `html` | `parse`, `query`, `inner_text`, `inner_html`, `attr`, `children` |
| `http` | `get`, `post`, `put`, `delete`, `patch`, `head`, `options` |
| `url` | `encode`, `decode`, `parse`, `build` |
| `stats` | `mean`, `median`, `mode`, `std`, `variance`, `min`, `max`, `sum`, `correlation`, `regression`, `normalize`, `standardize` |
| `re` | `find`, `find_all`, `sub`, `sub_all`, `split`, `is_match`, `captures` |
| `datetime` | `now`, `from_unix`, `format`, `parse`, `unix` |
| `logging` | `info`, `warn`, `error`, `debug`, `set_level` |
| `subprocess` | `run`, `run_shell`, `run_output`, `run_shell_output` |
| `path` | `join`, `basename`, `dirname`, `ext`, `abs`, `is_file`, `is_dir`, `size`, `rename`, `copy`, `remove`, `remove_dir`, `list_dir`, `walk`, `split` |
| `hashlib` | `sha256`, `sha512`, `sha1`, `md5`, `base64_encode`, `base64_decode`, `hex_encode`, `hex_decode` |
| `string` | `len`, `upper`, `lower`, `trim`, `trim_start`, `trim_end`, `split`, `join`, `contains`, `starts_with`, `ends_with`, `replace`, `reverse`, `repeat`, `substring`, `bytes`, `from_bytes` |
| `collections` | `Counter`, `deque`, `push_left`, `push_right`, `pop_left`, `pop_right`, `flatten`, `group_by` |
| `itertools` | `sorted`, `unique`, `reverse`, `enumerate`, `zip`, `map`, `filter`, `reduce`, `take`, `drop`, `slice`, `cycle`, `repeat`, `chunks`, `any`, `all`, `product`, `compose` |
| `test` | `assert_eq`, `assert_ne`, `assert_true`, `assert_false`, `assert_lt`, `assert_gt`, `assert_approx` |
| **C Extensions** | |
| `panda` | NumPy-like arrays: `arange`, `zeros`, `ones`, `linspace`, `sum`, `mean`, `min`, `max`, `std`, `abs`, `sin`, `cos`, `sqrt`, `pow`, `add`, `sub`, `mul`, `dot`, `shape`, `reshape`, `eye` (build: `make -C modules`) |
| **Optional (feature flags)** | |
| `opencv` | Computer vision: `imread`, `imwrite`, `imread_gray`, `cvt_color`, `resize`, `resize_fast`, `gaussian_blur`, `blur`, `canny`, `edges`, `threshold`, `shape`, `grayscale`, `invert`, `sepia`, `brightness`, `contrast`, `pixelate`, `flip`, `rotate`, `crop`, `letterbox`, `draw_rect`, `draw_text`, `equalize_hist`, `normalize`, `match_template`, `copy`, `thumbnail`, `to_ascii`, `read_ascii`, `imshow`, `wait_key`, `destroy_all_windows`, `free`, `detect_cascade`, `detect_people`, `detect_dnn`, plus 40+ constants (feature=opencv, requires system OpenCV 4/5) |
| **Windows** | |
| `leopard` | Native GUI toolkit (Win32) |
| **Linux** | |
| `panther` | Native GUI toolkit (GTK4, feature=panther) |

## Performance Benchmarks

Benchmarks comparing Lion 1.7.0 (release build) against Python 3.14 on the same workloads. Lower is better.

| Benchmark | Lion (ms) | Python (ms) | vs Python |
|-----------|-----------|-------------|-----------|
| `re.find_all` — 10k lines | 5.53 | 5.35 | ~1.0× (on par) |
| `re.sub_all` — 10k lines | 11.24 | 27.74 | **~2.5× faster** |
| `re.split` — 10k lines | 2.49 | 1.54 | ~1.6× slower |
| `collections.Counter` — 50k words | 5.25 | 3.38 | ~1.6× slower |
| `itertools.unique` — 20k items | 1.52 | 0.41 | ~3.7× slower |
| `itertools.sorted` — 10k items | 0.21 | 0.14 | ~1.5× slower |
| `datetime.now` — 10k calls | 18.38 | 11.81 | ~1.6× slower |
| `datetime.format` — 10k calls | 126.01 | 51.78 | ~2.4× slower |
| `hashlib.sha256` — 1k strings | 2.21 | 1.10 | ~2.0× slower |
| `hashlib.base64` — 1k strings | 2.39 | 0.93 | ~2.6× slower |
| `subprocess.run_shell` — 100 calls | 508.09 | 523.89 | **~1.0× faster** |

Performance improved significantly in v1.6.3 — `datetime.now` went from 7.3× slower than Python to 1.6× slower via GC pool pre-allocation. v1.7.0 adds the project manager and major OpenCV expansion with automatic GC-tracked image memory.

Run benchmarks yourself:

```bash
cargo build --release --bin lion
./target/release/lion run benchmarks/bench_lion.lion
python3 benchmarks/bench_python.py
```

## CLI

| Command | Description |
|---------|-------------|
| `lion run <file>` | Run a script |
| `lion repl` | Interactive REPL |
| `lion run --disassemble <file>` | Show bytecode |
| `lion fmt <file>` | Format source code |
| `lion test [filter]` | Run tests (default `./tests/` or `.`) |
| `lion version` | Show version |
| `lion new <name>` | Create a new Lion project |
| `lion init` | Initialize Lion project in current dir |
| `lion build` | Check all `.lion` files in project for errors |
| `lion-rs <file>` | Quick-run a file without subcommands |

## Project Management (v1.7.0)

Lion includes a built-in project manager via the `lion` binary. Create, build, and run Lion projects:

```bash
# Create a new project
lion new my-app
cd my-app

# Run the project entry point
lion run

# Check all .lion files for errors
lion build

# Run tests from the tests/ directory
lion test
```

### Project Structure

```
my-app/
  lion.json      # Project manifest
  src/
    main.lion    # Entry point (default)
  tests/         # Test files (optional)
```

### `lion.json`

```json
{
  "name": "my-app",
  "version": "0.1.0",
  "entry": "src/main.lion",
  "dependencies": {}
}
```

### Quick Runner

The `lion-rs` binary runs any Lion file directly without subcommands:

```bash
lion-rs script.lion
```

This is equivalent to `lion run script.lion` but shorter — useful for shebangs and quick scripts.

## Advanced Builds

### Linux GUI (Panther)

Requires GTK4 development headers:

```bash
# Ubuntu/Debian
sudo apt install libgtk-4-dev
# Fedora
sudo dnf install gtk4-devel
# Arch
sudo pacman -S gtk4
```

Build and run:

```bash
cargo build --release --features panther
./target/release/lion run examples/textedit.lion
```

### Portable Package

Create a self-contained tarball for distribution:

```bash
# Without GTK (portable to any Linux with glibc)
bash scripts/package.sh

# With GTK (bundles GTK4 shared libraries)
bash scripts/package.sh --panther
```

The output appears in `dist/` and includes the binary, C extensions, examples, and a launcher script.

### OpenCV Computer Vision (v1.7.0)

Requires system OpenCV 4/5 development libraries:

```bash
# Ubuntu/Debian
sudo apt install libopencv-dev

# Fedora
sudo dnf install opencv-devel

# Arch
sudo pacman -S opencv
```

Build and set `PKG_CONFIG_PATH` if needed:

```bash
PKG_CONFIG_PATH=/usr/lib/pkgconfig cargo build --release --features opencv
```

Images are **automatically garbage-collected** — no manual `free` needed (though `opencv.free()` still works for explicit cleanup).

#### Image I/O

| Function | Description |
|----------|-------------|
| `imread(path)` | Load image (BGR) |
| `imread_gray(path)` | Load image as grayscale |
| `imwrite(path, img)` | Save image to file |
| `copy(img)` | Deep-clone an image |
| `shape(img)` | Return `[height, width, channels]` |

#### Color & Conversion

| Function | Description |
|----------|-------------|
| `cvt_color(img, code)` | Color space conversion |
| `grayscale(img)` | Convert to grayscale |
| `to_ascii(img, [width])` | Convert image to ASCII art |
| `read_ascii(path, [width])` | Load file and convert to ASCII art |

#### Filters & Effects

| Function | Description |
|----------|-------------|
| `gaussian_blur(img, kx, ky, sigma)` | Gaussian blur |
| `blur(img, ksize)` | Simple average blur |
| `canny(img, low, high)` | Edge detection |
| `edges(img)` | Auto-thresholded edge detection |
| `threshold(img, thresh, maxval, type)` | Binary thresholding |
| `invert(img)` | Bitwise NOT (invert colors) |
| `sepia(img)` | Sepia tone filter |
| `brightness(img, delta)` | Adjust brightness |
| `contrast(img, alpha)` | Adjust contrast |
| `pixelate(img, block)` | Pixelate effect |
| `equalize_hist(img)` | Histogram equalization (auto-grayscale) |
| `normalize(img, [alpha], [beta], [norm_type])` | Normalize value range |
| `match_template(img, template, [method])` | Template matching |

#### Geometry & Drawing

| Function | Description |
|----------|-------------|
| `resize(img, width, height)` | Resize with linear interpolation |
| `resize_fast(img, width, height)` | Faster resize (nearest neighbor) |
| `thumbnail(img, max_size)` | Aspect-ratio-preserving thumbnail |
| `flip(img, code)` | Flip (0=vertical, 1=horizontal, -1=both) |
| `rotate(img, code)` | Rotate 90/180/270 |
| `crop(img, x, y, w, h)` | Extract sub-region |
| `letterbox(img, tw, th, [r], [g], [b])` | Resize with padding |
| `draw_rect(img, x, y, w, h, [r], [g], [b], [thick])` | Draw rectangle |
| `draw_text(img, text, x, y, [scale], [r], [g], [b], [thick])` | Draw text |

#### Object Detection (v1.7.0)

| Function | Description |
|----------|-------------|
| `detect_cascade(img, xml_path, [scale], [min_neighbors], [min_w], [min_h], [max_w], [max_h])` | Haar/LBP cascade detection → list of `[x,y,w,h]` |
| `detect_people(img, [hit_thresh], [scale], [group_thresh])` | HOG people detection → list of `[x,y,w,h]` |
| `detect_dnn(img, model, [config], [conf_thresh], [nms_thresh])` | DNN object detection → list of `[class_id, conf, x, y, w, h]` |

#### Display

| Function | Description |
|----------|-------------|
| `imshow(name, img)` | Show image in window |
| `wait_key(delay)` | Wait for key press |
| `destroy_all_windows()` | Close all windows |

#### Constants

| Group | Values |
|-------|--------|
| Color codes | `BGR2GRAY`, `GRAY2BGR`, `BGR2RGB`, `RGB2GRAY`, `BGRA2GRAY`, `GRAY2BGRA`, `BGR2HSV`, `HSV2BGR`, `BGR2HLS`, `BGR2LAB` |
| Threshold | `THRESH_BINARY`, `THRESH_BINARY_INV`, `THRESH_TRUNC`, `THRESH_TOZERO`, `THRESH_TOZERO_INV`, `THRESH_OTSU` |
| Interpolation | `INTER_LINEAR`, `INTER_NEAREST`, `INTER_CUBIC`, `INTER_AREA` |
| Flip/Rotate | `FLIP_VERTICAL`, `FLIP_HORIZONTAL`, `FLIP_BOTH`, `ROTATE_90_CLOCKWISE`, `ROTATE_180`, `ROTATE_90_COUNTERCLOCKWISE` |
| Drawing | `FILLED`, `LINE_8`, `LINE_AA`, `FONT_HERSHEY_SIMPLEX` |
| Template matching | `TM_CCOEFF`, `TM_CCOEFF_NORMED`, `TM_CCORR`, `TM_CCORR_NORMED`, `TM_SQDIFF`, `TM_SQDIFF_NORMED` |
| Normalize | `NORM_MINMAX`, `NORM_L1`, `NORM_L2` |
| Border | `BORDER_CONSTANT` |

Example:

```lion
let img = opencv.imread("photo.jpg");
let gray = opencv.cvt_color(img, opencv.BGR2GRAY);
let edges = opencv.canny(gray, 50.0, 150.0);
opencv.imwrite("edges.jpg", edges);

// Detect faces
let faces = opencv.detect_cascade(img, "/usr/share/opencv5/haarcascades/haarcascade_frontalface_default.xml");
for f in faces {
    let [x, y, w, h] = f;
    img = opencv.draw_rect(img, x, y, w, h, 0, 255, 0, 2);
}
opencv.imwrite("faces.jpg", img);
```

**Memory management**: Images are GC-tracked — they auto-free when no longer referenced. No need to call `free()`, but it's available for eager cleanup.

### CUDA Support

```bash
cargo build --release --features cuda
```

### LSP Server

```bash
cargo build --bin lion-lsp
```

### VS Code Extension

```bash
# Package and install
cd vscode-lion
npm install
npx @vscode/vsce package
code --install-extension lion-lang-*.vsix
cd ..
```

## Running Tests

```bash
cargo build --release --bin lion
./target/release/lion test tests/

# Or from a Lion project root:
cd my-project
lion test          # runs tests/ directory
lion test filter   # runs tests matching "filter"
```

## Embedding Lion as a Library

Lion can be embedded in other Rust applications via the `lion` crate:

```rust
use lion::{execute_source, execute_file};

fn main() {
    match execute_source("print(\"Hello from embedded Lion!\");") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

Add to your `Cargo.toml`:
```toml
[dependencies]
lion = { path = "/path/to/lion" }
```

## Project Structure

```
src/           # Rust source (lexer, parser, compiler, VM, GC, stdlib, modules)
examples/      # Example .lion scripts
tests/         # Test .lion scripts
benchmarks/    # Performance benchmarks (Lion + Python)
vscode-lion/   # VS Code extension (syntax highlighting + LSP client)
include/       # C header for native extensions
modules/       # C extension shared libraries (.dll/.so/.dylib)
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `cargo build` warns about CUDA | Warning only — build succeeds without CUDA |
| Python interop not working | Build with `--features python`, ensure Python dev headers installed |
| OpenCV build fails (`opencv4.pc` not found) | Set `PKG_CONFIG_PATH` to the directory containing `opencv5.pc` (e.g. `/usr/lib/pkgconfig`) |
| Slow performance | Always use `cargo build --release` — debug builds are ~50× slower |
| Tests fail | Build release binary first: `cargo build --release --bin lion` |

## License

MIT — see [LICENSE](LICENSE).
