use std::rc::Rc;
use std::sync::Mutex;
use std::collections::HashMap;
use opencv::core::Mat;
use opencv::prelude::*;
use crate::gc::*;

static NEXT_HANDLE: Mutex<i64> = Mutex::new(1);
static IMAGES: Mutex<Option<HashMap<i64, Mat>>> = Mutex::new(None);

fn ensure_images() -> &'static Mutex<Option<HashMap<i64, Mat>>> {
    &IMAGES
}

fn free_image(handle: i64) {
    let mut guard = ensure_images().lock().unwrap();
    if let Some(ref mut map) = *guard {
        map.remove(&handle);
    }
}

fn alloc_handle(heap: &mut GcHeap, mat: Mat) -> Value {
    let mut guard = ensure_images().lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    let map = guard.as_mut().unwrap();
    let mut hg = NEXT_HANDLE.lock().unwrap();
    let handle = *hg;
    *hg += 1;
    map.insert(handle, mat);
    make_image(heap, handle)
}

fn get_handle(val: &Value, heap: &GcHeap) -> Result<i64, String> {
    match val {
        Value::Image(r) => match heap.get(*r) {
            GcObj::OcvHandle(h) => Ok(*h),
            _ => Err("not an image handle".to_string()),
        },
        _ => Err("expected image".to_string()),
    }
}

fn get_mat(val: &Value, heap: &GcHeap) -> Result<Mat, String> {
    let handle = get_handle(val, heap)?;
    let guard = ensure_images().lock().unwrap();
    let map = guard.as_ref().ok_or("no images loaded")?;
    map.get(&handle).cloned().ok_or_else(|| format!("no image with handle {}", handle))
}

fn arg_str<'a>(args: &'a [Value], ctx: &'a VmContext, idx: usize, name: &str) -> Result<&'a str, String> {
    let val = args.get(idx).ok_or_else(|| format!("opencv.{}: argument {} required", name, idx))?;
    match val {
        Value::String(r) => match ctx.heap.get(*r) {
            GcObj::String(s) => Ok(s.as_str()),
            _ => Err(format!("opencv.{}: invalid string", name)),
        },
        _ => Err(format!("opencv.{}: expected string", name)),
    }
}

fn arg_int(args: &[Value], idx: usize, name: &str) -> Result<i64, String> {
    let val = args.get(idx).ok_or_else(|| format!("opencv.{}: argument {} required", name, idx))?;
    match val {
        Value::Int(n) => Ok(*n),
        Value::UInt(n) => Ok(*n as i64),
        Value::Float(n) => Ok(*n as i64),
        _ => Err(format!("opencv.{}: expected number", name)),
    }
}

fn arg_float(args: &[Value], idx: usize, name: &str) -> Result<f64, String> {
    let val = args.get(idx).ok_or_else(|| format!("opencv.{}: argument {} required", name, idx))?;
    match val {
        Value::Int(n) => Ok(*n as f64),
        Value::UInt(n) => Ok(*n as f64),
        Value::Float(n) => Ok(*n),
        _ => Err(format!("opencv.{}: expected number", name)),
    }
}

fn arg_optional_float(args: &[Value], idx: usize, default: f64) -> f64 {
    args.get(idx).map(|v| match v {
        Value::Int(n) => *n as f64,
        Value::UInt(n) => *n as f64,
        Value::Float(n) => *n,
        _ => default,
    }).unwrap_or(default)
}

fn rects_to_list(heap: &mut GcHeap, rects: &opencv::core::Vector<opencv::core::Rect>) -> Value {
    let n = rects.len();
    let mut items = Vec::with_capacity(n);
    for i in 0..n {
        if let Ok(r) = rects.get(i) {
            items.push(make_list(heap, vec![
                Value::Int(r.x as i64),
                Value::Int(r.y as i64),
                Value::Int(r.width as i64),
                Value::Int(r.height as i64),
            ]));
        }
    }
    make_list(heap, items)
}

fn arg_optional_int(args: &[Value], idx: usize, default: i64) -> i64 {
    args.get(idx).map(|v| match v {
        Value::Int(n) => *n,
        Value::UInt(n) => *n as i64,
        Value::Float(n) => *n as i64,
        _ => default,
    }).unwrap_or(default)
}

macro_rules! ocv_const {
    ($e:expr) => { $e as i64 }
}

const ASCII_CHARS: &[u8] = b"$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";

fn mat_to_ascii(src: &Mat, width: i32) -> Result<String, String> {
    let mut gray = Mat::default();
    let chans = src.channels();

    if chans == 3 || chans == 4 {
        let code = if chans == 4 {
            opencv::imgproc::COLOR_BGRA2GRAY
        } else {
            opencv::imgproc::COLOR_BGR2GRAY
        };
        opencv::imgproc::cvt_color_def(src, &mut gray, code)
            .map_err(|e| format!("to_ascii cvt_color: {}", e))?;
    } else if chans == 1 {
        gray = src.clone();
    } else {
        return Err(format!("to_ascii: unsupported channels {}", chans));
    }

    let h = gray.rows();
    let w = gray.cols();
    let aspect = h as f64 / w as f64;
    let new_w = width.max(8).min(400);
    let new_h = (new_w as f64 * aspect).round() as i32;
    let new_h = new_h.max(1).min(200);

    let mut resized = Mat::default();
    let dsize = opencv::core::Size { width: new_w, height: new_h };
    opencv::imgproc::resize(&gray, &mut resized, dsize, 0.0, 0.0, opencv::imgproc::INTER_LINEAR)
        .map_err(|e| format!("to_ascii resize: {}", e))?;

    let data = resized.data();
    let rstep = resized.step1_def().map_err(|e| format!("to_ascii step: {}", e))?;
    let nchars = ASCII_CHARS.len() - 1;

    let mut result = String::with_capacity((new_w as usize + 1) * new_h as usize);

    unsafe {
        for r in 0..new_h {
            for c in 0..new_w {
                let p = *data.add(r as usize * rstep + c as usize) as usize;
                let idx = p * nchars / 255;
                result.push(ASCII_CHARS[idx] as char);
            }
            result.push('\n');
        }
    }

    Ok(result)
}

pub fn build_opencv() -> Vec<(String, Value)> {
    // Register the resource dropper so GC auto-frees images
    set_resource_dropper(free_image);

    let mut funcs = Vec::new();

    let imread_flag = opencv::imgcodecs::IMREAD_COLOR;
    let inter_linear = opencv::imgproc::INTER_LINEAR;
    let inter_nearest = opencv::imgproc::INTER_NEAREST;
    let border_default = opencv::core::BORDER_DEFAULT;

    // --- Original functions (now return GC-tracked Image values) ---

    funcs.push(("imread".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.imread>".to_string(),
        func: Rc::new(move |args, ctx| {
            let path = arg_str(args, ctx, 0, "imread")?;
            let mat = opencv::imgcodecs::imread(path, imread_flag)
                .map_err(|e| format!("imread failed: {}", e))?;
            if mat.empty() {
                return Err("imread: could not read image".to_string());
            }
            Ok(alloc_handle(ctx.heap, mat))
        }),
    })));

    funcs.push(("imwrite".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.imwrite>".to_string(),
        func: Rc::new(|args, ctx| {
            let path = arg_str(args, ctx, 0, "imwrite")?;
            let img = args.get(1).ok_or("imwrite: argument 1 required")?;
            let mat = get_mat(img, ctx.heap)?;
            opencv::imgcodecs::imwrite(path, &mat, &opencv::core::Vector::new())
                .map_err(|e| format!("imwrite failed: {}", e))?;
            Ok(Value::Bool(true))
        }),
    })));

    funcs.push(("cvt_color".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.cvt_color>".to_string(),
        func: Rc::new(move |args, ctx| {
            let img = args.get(0).ok_or("cvt_color: argument 0 required")?;
            let code = arg_int(args, 1, "cvt_color")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            opencv::imgproc::cvt_color(&src, &mut dst, code as i32, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                .map_err(|e| format!("cvt_color failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("resize".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.resize>".to_string(),
        func: Rc::new(move |args, ctx| {
            let img = args.get(0).ok_or("resize: argument 0 required")?;
            let width = arg_int(args, 1, "resize")?;
            let height = arg_int(args, 2, "resize")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            let dsize = opencv::core::Size { width: width as i32, height: height as i32 };
            opencv::imgproc::resize(&src, &mut dst, dsize, 0.0, 0.0, inter_linear)
                .map_err(|e| format!("resize failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("gaussian_blur".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.gaussian_blur>".to_string(),
        func: Rc::new(move |args, ctx| {
            let img = args.get(0).ok_or("gaussian_blur: argument 0 required")?;
            let kx = arg_int(args, 1, "gaussian_blur")?;
            let ky = arg_int(args, 2, "gaussian_blur")?;
            let sigma = arg_float(args, 3, "gaussian_blur")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            let ksize = opencv::core::Size { width: kx as i32, height: ky as i32 };
            opencv::imgproc::gaussian_blur(&src, &mut dst, ksize, sigma, sigma, border_default, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                .map_err(|e| format!("gaussian_blur failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("canny".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.canny>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("canny: argument 0 required")?;
            let low = arg_float(args, 1, "canny")?;
            let high = arg_float(args, 2, "canny")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            opencv::imgproc::canny(&src, &mut dst, low, high, 3, false)
                .map_err(|e| format!("canny failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("threshold".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.threshold>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("threshold: argument 0 required")?;
            let thresh = arg_float(args, 1, "threshold")?;
            let maxval = arg_float(args, 2, "threshold")?;
            let typ = arg_int(args, 3, "threshold")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            opencv::imgproc::threshold(&src, &mut dst, thresh, maxval, typ as i32)
                .map_err(|e| format!("threshold failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("shape".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.shape>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("shape: argument 0 required")?;
            let mat = get_mat(img, ctx.heap)?;
            let list = make_list(ctx.heap, vec![
                Value::Int(mat.rows() as i64),
                Value::Int(mat.cols() as i64),
                Value::Int(mat.channels() as i64),
            ]);
            Ok(list)
        }),
    })));

    funcs.push(("free".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.free>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("free: argument 0 required")?;
            let handle = get_handle(img, ctx.heap)?;
            free_image(handle);
            Ok(Value::Nil)
        }),
    })));

    funcs.push(("imshow".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.imshow>".to_string(),
        func: Rc::new(|args, ctx| {
            let name = arg_str(args, ctx, 0, "imshow")?;
            let img = args.get(1).ok_or("imshow: argument 1 required")?;
            let mat = get_mat(img, ctx.heap)?;
            opencv::highgui::imshow(name, &mat)
                .map_err(|e| format!("imshow failed: {}", e))?;
            Ok(Value::Nil)
        }),
    })));

    funcs.push(("wait_key".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.wait_key>".to_string(),
        func: Rc::new(|args, _| {
            let delay = arg_int(args, 0, "wait_key")?;
            let key = opencv::highgui::wait_key(delay as i32)
                .map_err(|e| format!("wait_key failed: {}", e))?;
            Ok(Value::Int(key as i64))
        }),
    })));

    funcs.push(("destroy_all_windows".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.destroy_all_windows>".to_string(),
        func: Rc::new(|_, _| {
            let _ = opencv::highgui::destroy_all_windows();
            Ok(Value::Nil)
        }),
    })));

    // --- New combined / convenience functions ---

    funcs.push(("to_ascii".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.to_ascii>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("to_ascii: argument 0 required")?;
            let width = arg_optional_int(args, 1, 80) as i32;
            let src = get_mat(img, ctx.heap)?;
            let ascii = mat_to_ascii(&src, width)?;
            Ok(make_string_owned(ctx.heap, ascii))
        }),
    })));

    funcs.push(("read_ascii".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.read_ascii>".to_string(),
        func: Rc::new(move |args, ctx| {
            let path = arg_str(args, ctx, 0, "read_ascii")?;
            let width = arg_optional_int(args, 1, 80) as i32;
            let mat = opencv::imgcodecs::imread(path, imread_flag)
                .map_err(|e| format!("read_ascii: imread failed: {}", e))?;
            if mat.empty() {
                return Err("read_ascii: could not read image".to_string());
            }
            let ascii = mat_to_ascii(&mat, width)?;
            Ok(make_string_owned(ctx.heap, ascii))
        }),
    })));

    funcs.push(("grayscale".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.grayscale>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("grayscale: argument 0 required")?;
            let src = get_mat(img, ctx.heap)?;
            let chans = src.channels();
            let mut dst = Mat::default();
            if chans == 1 {
                dst = src.clone();
            } else if chans == 3 {
                opencv::imgproc::cvt_color_def(&src, &mut dst, opencv::imgproc::COLOR_BGR2GRAY)
                    .map_err(|e| format!("grayscale failed: {}", e))?;
            } else if chans == 4 {
                opencv::imgproc::cvt_color_def(&src, &mut dst, opencv::imgproc::COLOR_BGRA2GRAY)
                    .map_err(|e| format!("grayscale failed: {}", e))?;
            }
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("thumbnail".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.thumbnail>".to_string(),
        func: Rc::new(move |args, ctx| {
            let img = args.get(0).ok_or("thumbnail: argument 0 required")?;
            let max_size = arg_int(args, 1, "thumbnail")? as i32;
            let src = get_mat(img, ctx.heap)?;
            let h = src.rows();
            let w = src.cols();
            let max_size = max_size.max(1);
            let (new_w, new_h) = if h > w {
                (w * max_size / h, max_size)
            } else {
                (max_size, h * max_size / w)
            };
            let new_w = new_w.max(1);
            let new_h = new_h.max(1);
            let mut dst = Mat::default();
            let dsize = opencv::core::Size { width: new_w, height: new_h };
            opencv::imgproc::resize(&src, &mut dst, dsize, 0.0, 0.0, inter_linear)
                .map_err(|e| format!("thumbnail failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("flip".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.flip>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("flip: argument 0 required")?;
            let flip_code = arg_int(args, 1, "flip")? as i32;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            opencv::core::flip(&src, &mut dst, flip_code)
                .map_err(|e| format!("flip failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("brightness".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.brightness>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("brightness: argument 0 required")?;
            let delta = arg_float(args, 1, "brightness")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            src.convert_to(&mut dst, -1, 1.0, delta)
                .map_err(|e| format!("brightness failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("contrast".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.contrast>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("contrast: argument 0 required")?;
            let alpha = arg_float(args, 1, "contrast")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            src.convert_to(&mut dst, -1, alpha, 0.0)
                .map_err(|e| format!("contrast failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("invert".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.invert>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("invert: argument 0 required")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            opencv::core::bitwise_not_def(&src, &mut dst)
                .map_err(|e| format!("invert failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("sepia".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.sepia>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("sepia: argument 0 required")?;
            let src = get_mat(img, ctx.heap)?;
            let rows = src.rows();
            let cols = src.cols();
            let chans = src.channels();
            if chans != 3 {
                return Err("sepia requires a 3-channel BGR image".to_string());
            }
            let typ = src.typ();
            let step = src.step1_def().unwrap_or(cols as usize * 3);
            let src_data = src.data();
            let mut dst = Mat::new_rows_cols_with_default(rows, cols, typ, opencv::core::Scalar::all(0.0))
                .map_err(|e| format!("sepia: {}", e))?;
            let dst_data = dst.data_mut();

            unsafe {
                for r in 0..rows {
                    let row_off = r as usize * step;
                    for c in 0..cols {
                        let idx = row_off + c as usize * 3;
                        let b = *src_data.add(idx) as f64;
                        let g = *src_data.add(idx + 1) as f64;
                        let rv = *src_data.add(idx + 2) as f64;

                        let nb = (rv * 0.272 + g * 0.534 + b * 0.131).clamp(0.0, 255.0) as u8;
                        let ng = (rv * 0.349 + g * 0.686 + b * 0.168).clamp(0.0, 255.0) as u8;
                        let nr = (rv * 0.393 + g * 0.769 + b * 0.189).clamp(0.0, 255.0) as u8;

                        *dst_data.add(idx) = nb;
                        *dst_data.add(idx + 1) = ng;
                        *dst_data.add(idx + 2) = nr;
                    }
                }
            }

            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("blur".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.blur>".to_string(),
        func: Rc::new(move |args, ctx| {
            let img = args.get(0).ok_or("blur: argument 0 required")?;
            let ksize = arg_int(args, 1, "blur")? as i32;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            let size = opencv::core::Size { width: ksize, height: ksize };
            opencv::imgproc::blur_def(&src, &mut dst, size)
                .map_err(|e| format!("blur failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("edges".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.edges>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("edges: argument 0 required")?;
            let src = get_mat(img, ctx.heap)?;

            let mut gray = Mat::default();
            let chans = src.channels();
            if chans == 3 || chans == 4 {
                let code = if chans == 4 { opencv::imgproc::COLOR_BGRA2GRAY } else { opencv::imgproc::COLOR_BGR2GRAY };
                opencv::imgproc::cvt_color_def(&src, &mut gray, code)
                    .map_err(|e| format!("edges cvt_color: {}", e))?;
            } else {
                gray = src.clone();
            }

            let mean = opencv::core::mean_def(&gray)
                .map_err(|e| format!("edges mean: {}", e))?;
            let low = mean[0] * 0.4;
            let high = mean[0] * 1.2;

            let mut dst = Mat::default();
            opencv::imgproc::canny(&gray, &mut dst, low, high, 3, false)
                .map_err(|e| format!("edges failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("pixelate".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.pixelate>".to_string(),
        func: Rc::new(move |args, ctx| {
            let img = args.get(0).ok_or("pixelate: argument 0 required")?;
            let block = arg_int(args, 1, "pixelate")? as i32;
            let src = get_mat(img, ctx.heap)?;
            let h = src.rows();
            let w = src.cols();
            let block = block.max(1);
            let small_w = (w / block).max(1);
            let small_h = (h / block).max(1);

            let mut small = Mat::default();
            let ssize = opencv::core::Size { width: small_w, height: small_h };
            opencv::imgproc::resize(&src, &mut small, ssize, 0.0, 0.0, inter_nearest)
                .map_err(|e| format!("pixelate shrink: {}", e))?;

            let mut dst = Mat::default();
            let dsize = opencv::core::Size { width: w, height: h };
            opencv::imgproc::resize(&small, &mut dst, dsize, 0.0, 0.0, inter_nearest)
                .map_err(|e| format!("pixelate expand: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("rotate".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.rotate>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("rotate: argument 0 required")?;
            let code = arg_int(args, 1, "rotate")? as i32;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            opencv::core::rotate(&src, &mut dst, code)
                .map_err(|e| format!("rotate failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("imread_gray".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.imread_gray>".to_string(),
        func: Rc::new(|args, ctx| {
            let path = arg_str(args, ctx, 0, "imread_gray")?;
            let mat = opencv::imgcodecs::imread(path, opencv::imgcodecs::IMREAD_GRAYSCALE)
                .map_err(|e| format!("imread_gray failed: {}", e))?;
            if mat.empty() {
                return Err("imread_gray: could not read image".to_string());
            }
            Ok(alloc_handle(ctx.heap, mat))
        }),
    })));

    funcs.push(("copy".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.copy>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("copy: argument 0 required")?;
            let mat = get_mat(img, ctx.heap)?;
            Ok(alloc_handle(ctx.heap, mat.clone()))
        }),
    })));

    // ======= Object Detection =======

    funcs.push(("detect_cascade".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.detect_cascade>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("detect_cascade: argument 0 (image) required")?;
            let cascade_path = arg_str(args, ctx, 1, "detect_cascade")?;
            let scale_factor = arg_optional_float(args, 2, 1.1);
            let min_neighbors = arg_optional_int(args, 3, 3) as i32;
            let min_w = arg_optional_int(args, 4, 0) as i32;
            let min_h = arg_optional_int(args, 5, 0) as i32;
            let max_w = arg_optional_int(args, 6, 0) as i32;
            let max_h = arg_optional_int(args, 7, 0) as i32;

            let mut cascade = opencv::xobjdetect::CascadeClassifier::new(cascade_path)
                .map_err(|e| format!("detect_cascade: failed to load cascade: {}", e))?;
            let src = get_mat(img, ctx.heap)?;
            let mut gray = Mat::default();
            if src.channels() > 1 {
                opencv::imgproc::cvt_color_def(&src, &mut gray, opencv::imgproc::COLOR_BGR2GRAY)
                    .map_err(|e| format!("detect_cascade: {}", e))?;
            } else {
                gray = src.clone();
            }

            let mut objects = opencv::core::Vector::<opencv::core::Rect>::new();
            let min_size = opencv::core::Size { width: min_w, height: min_h };
            let max_size = opencv::core::Size { width: max_w, height: max_h };
            cascade.detect_multi_scale(&gray, &mut objects, scale_factor, min_neighbors, 0, min_size, max_size)
                .map_err(|e| format!("detect_cascade: {}", e))?;

            Ok(rects_to_list(ctx.heap, &objects))
        }),
    })));

    funcs.push(("detect_people".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.detect_people>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("detect_people: argument 0 (image) required")?;
            let hit_threshold = arg_optional_float(args, 1, 0.0);
            let scale = arg_optional_float(args, 2, 1.05);
            let group_threshold = arg_optional_float(args, 3, 2.0);

            let src = get_mat(img, ctx.heap)?;
            let mut hog = opencv::xobjdetect::HOGDescriptor::new_def()
                .map_err(|e| format!("detect_people: {}", e))?;
            let detector = opencv::xobjdetect::HOGDescriptor::get_default_people_detector()
                .map_err(|e| format!("detect_people: {}", e))?;
            hog.set_svm_detector(detector);

            let mut found = opencv::core::Vector::<opencv::core::Rect>::new();
            let win_stride = opencv::core::Size { width: 0, height: 0 };
            let padding = opencv::core::Size { width: 0, height: 0 };
            hog.detect_multi_scale(&src, &mut found, hit_threshold, win_stride, padding, scale, group_threshold, false)
                .map_err(|e| format!("detect_people: {}", e))?;

            Ok(rects_to_list(ctx.heap, &found))
        }),
    })));

    funcs.push(("detect_dnn".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.detect_dnn>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("detect_dnn: argument 0 (image) required")?;
            let model_path = arg_str(args, ctx, 1, "detect_dnn")?;
            let config_path = args.get(2).and_then(|v| match v {
                Value::String(r) => match ctx.heap.get(*r) {
                    GcObj::String(s) if s.len() > 0 => Some(s.clone()),
                    _ => None,
                },
                _ => None,
            });
            let conf_threshold = arg_optional_float(args, 3, 0.5) as f32;
            let nms_threshold = arg_optional_float(args, 4, 0.4) as f32;

            let src = get_mat(img, ctx.heap)?;
            let model = if let Some(ref cfg) = config_path {
                opencv::dnn::DetectionModel::new(model_path, cfg)
                    .map_err(|e| format!("detect_dnn: failed to load model: {}", e))?
            } else {
                opencv::dnn::DetectionModel::new_def(model_path)
                    .map_err(|e| format!("detect_dnn: failed to load model: {}", e))?
            };

            let mut class_ids = opencv::core::Vector::<i32>::new();
            let mut confidences = opencv::core::Vector::<f32>::new();
            let mut boxes = opencv::core::Vector::<opencv::core::Rect>::new();

            // DetectionModel::detect takes &mut self; need ownership
            let mut model = model;
            model.detect(&src, &mut class_ids, &mut confidences, &mut boxes, conf_threshold, nms_threshold)
                .map_err(|e| format!("detect_dnn: {}", e))?;

            let n = class_ids.len();
            let mut items = Vec::with_capacity(n);
            for i in 0..n {
                let cls = class_ids.get(i).unwrap_or(0);
                let conf = confidences.get(i).unwrap_or(0.0);
                let r = boxes.get(i).unwrap_or(opencv::core::Rect { x: 0, y: 0, width: 0, height: 0 });
                items.push(make_list(ctx.heap, vec![
                    Value::Int(cls as i64),
                    Value::Float(conf as f64),
                    Value::Int(r.x as i64),
                    Value::Int(r.y as i64),
                    Value::Int(r.width as i64),
                    Value::Int(r.height as i64),
                ]));
            }
            Ok(make_list(ctx.heap, items))
        }),
    })));

    // ======= Drawing & Geometry =======

    funcs.push(("draw_rect".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.draw_rect>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("draw_rect: argument 0 (image) required")?;
            let x = arg_int(args, 1, "draw_rect")?;
            let y = arg_int(args, 2, "draw_rect")?;
            let w = arg_int(args, 3, "draw_rect")?;
            let h = arg_int(args, 4, "draw_rect")?;
            let r = arg_optional_int(args, 5, 0);
            let g = arg_optional_int(args, 6, 255);
            let b = arg_optional_int(args, 7, 0);
            let thickness = arg_optional_int(args, 8, 2) as i32;

            let mut src = get_mat(img, ctx.heap)?;
            let pt1 = opencv::core::Point { x: x as i32, y: y as i32 };
            let pt2 = opencv::core::Point { x: (x + w) as i32, y: (y + h) as i32 };
            let color = opencv::core::Scalar::new(b as f64, g as f64, r as f64, 0.0);
            opencv::imgproc::rectangle_points(&mut src, pt1, pt2, color, thickness, 8, 0)
                .map_err(|e| format!("draw_rect failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, src))
        }),
    })));

    funcs.push(("draw_text".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.draw_text>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("draw_text: argument 0 (image) required")?;
            let text = arg_str(args, ctx, 1, "draw_text")?;
            let x = arg_int(args, 2, "draw_text")?;
            let y = arg_int(args, 3, "draw_text")?;
            let scale = arg_optional_float(args, 4, 0.8);
            let r = arg_optional_int(args, 5, 255);
            let g = arg_optional_int(args, 6, 255);
            let b = arg_optional_int(args, 7, 255);
            let thickness = arg_optional_int(args, 8, 2) as i32;

            let mut src = get_mat(img, ctx.heap)?;
            let org = opencv::core::Point { x: x as i32, y: y as i32 };
            let color = opencv::core::Scalar::new(b as f64, g as f64, r as f64, 0.0);
            opencv::imgproc::put_text(&mut src, text, org, opencv::imgproc::FONT_HERSHEY_SIMPLEX, scale, color, thickness, 8, false)
                .map_err(|e| format!("draw_text failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, src))
        }),
    })));

    funcs.push(("crop".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.crop>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("crop: argument 0 (image) required")?;
            let x = arg_int(args, 1, "crop")?;
            let y = arg_int(args, 2, "crop")?;
            let w = arg_int(args, 3, "crop")?;
            let h = arg_int(args, 4, "crop")?;
            let src = get_mat(img, ctx.heap)?;
            if x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > src.cols() as i64 || y + h > src.rows() as i64 {
                return Err(format!("crop: region ({}x{} at {}, {}) out of bounds for image {}x{}",
                    w, h, x, y, src.cols(), src.rows()));
            }
            let rect = opencv::core::Rect { x: x as i32, y: y as i32, width: w as i32, height: h as i32 };
            let roi = src.roi(rect).map_err(|e| format!("crop: {}", e))?;
            let dst = roi.clone_pointee();
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("letterbox".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.letterbox>".to_string(),
        func: Rc::new(move |args, ctx| {
            let img = args.get(0).ok_or("letterbox: argument 0 (image) required")?;
            let target_w = arg_int(args, 1, "letterbox")? as i32;
            let target_h = arg_int(args, 2, "letterbox")? as i32;
            let bg_r = arg_optional_int(args, 3, 0);
            let bg_g = arg_optional_int(args, 4, 0);
            let bg_b = arg_optional_int(args, 5, 0);

            let src = get_mat(img, ctx.heap)?;
            let h = src.rows();
            let w = src.cols();
            let scale = ((target_w as f64) / (w as f64)).min((target_h as f64) / (h as f64));
            let new_w = (w as f64 * scale).round() as i32;
            let new_h = (h as f64 * scale).round() as i32;
            let new_w = new_w.max(1);
            let new_h = new_h.max(1);

            let mut resized = Mat::default();
            let dsize = opencv::core::Size { width: new_w, height: new_h };
            opencv::imgproc::resize(&src, &mut resized, dsize, 0.0, 0.0, inter_linear)
                .map_err(|e| format!("letterbox: {}", e))?;

            let x_off = (target_w - new_w) / 2;
            let y_off = (target_h - new_h) / 2;
            let mut canvas = opencv::core::Mat::new_rows_cols_with_default(
                target_h, target_w,
                src.typ(),
                opencv::core::Scalar::new(bg_b as f64, bg_g as f64, bg_r as f64, 0.0),
            ).map_err(|e| format!("letterbox: {}", e))?;

            let canvas_rect = opencv::core::Rect { x: x_off, y: y_off, width: new_w, height: new_h };
            let mut roi = canvas.roi_mut(canvas_rect)
                .map_err(|e| format!("letterbox: {}", e))?;
            resized.copy_to(&mut roi)
                .map_err(|e| format!("letterbox: {}", e))?;
            Ok(alloc_handle(ctx.heap, canvas))
        }),
    })));

    // ======= Image Processing =======

    funcs.push(("equalize_hist".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.equalize_hist>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("equalize_hist: argument 0 required")?;
            let src = get_mat(img, ctx.heap)?;
            let mut gray = Mat::default();
            if src.channels() != 1 {
                opencv::imgproc::cvt_color_def(&src, &mut gray, opencv::imgproc::COLOR_BGR2GRAY)
                    .map_err(|e| format!("equalize_hist: {}", e))?;
            } else {
                gray = src.clone();
            }
            let mut dst = Mat::default();
            opencv::imgproc::equalize_hist(&gray, &mut dst)
                .map_err(|e| format!("equalize_hist: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("normalize".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.normalize>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("normalize: argument 0 required")?;
            let alpha = arg_optional_float(args, 1, 0.0);
            let beta = arg_optional_float(args, 2, 255.0);
            let norm_type = arg_optional_int(args, 3, opencv::core::NORM_MINMAX as i64) as i32;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            opencv::core::normalize(&src, &mut dst, alpha, beta, norm_type, -1, &opencv::core::Mat::default())
                .map_err(|e| format!("normalize: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    funcs.push(("match_template".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.match_template>".to_string(),
        func: Rc::new(|args, ctx| {
            let img = args.get(0).ok_or("match_template: argument 0 (image) required")?;
            let templ = args.get(1).ok_or("match_template: argument 1 (template) required")?;
            let method = arg_optional_int(args, 2, opencv::imgproc::TM_CCOEFF_NORMED as i64) as i32;
            let src = get_mat(img, ctx.heap)?;
            let tpl = get_mat(templ, ctx.heap)?;
            let mut result = Mat::default();
            opencv::imgproc::match_template_def(&src, &tpl, &mut result, method)
                .map_err(|e| format!("match_template: {}", e))?;

            let mut min_val = 0.0_f64;
            let mut max_val = 0.0_f64;
            let mut min_loc = opencv::core::Point { x: 0, y: 0 };
            let mut max_loc = opencv::core::Point { x: 0, y: 0 };
            opencv::core::min_max_loc(
                &result,
                Some(&mut min_val), Some(&mut max_val),
                Some(&mut min_loc), Some(&mut max_loc),
                &opencv::core::Mat::default(),
            ).map_err(|e| format!("match_template: min_max_loc: {}", e))?;

            let out = make_list(ctx.heap, vec![
                Value::Float(min_val),
                Value::Float(max_val),
                Value::Int(min_loc.x as i64),
                Value::Int(min_loc.y as i64),
                Value::Int(max_loc.x as i64),
                Value::Int(max_loc.y as i64),
            ]);
            Ok(out)
        }),
    })));

    funcs.push(("resize_fast".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.resize_fast>".to_string(),
        func: Rc::new(move |args, ctx| {
            let img = args.get(0).ok_or("resize_fast: argument 0 required")?;
            let width = arg_int(args, 1, "resize_fast")?;
            let height = arg_int(args, 2, "resize_fast")?;
            let src = get_mat(img, ctx.heap)?;
            let mut dst = Mat::default();
            let dsize = opencv::core::Size { width: width as i32, height: height as i32 };
            opencv::imgproc::resize(&src, &mut dst, dsize, 0.0, 0.0, inter_nearest)
                .map_err(|e| format!("resize_fast failed: {}", e))?;
            Ok(alloc_handle(ctx.heap, dst))
        }),
    })));

    // Constants
    macro_rules! push_ocv_const {
        ($name:expr, $val:expr) => {
            funcs.push(($name.to_string(), Value::Int(ocv_const!($val))));
        }
    }
    push_ocv_const!("COLOR_BGR2GRAY", opencv::imgproc::COLOR_BGR2GRAY);
    push_ocv_const!("COLOR_GRAY2BGR", opencv::imgproc::COLOR_GRAY2BGR);
    push_ocv_const!("COLOR_BGR2RGB", opencv::imgproc::COLOR_BGR2RGB);
    push_ocv_const!("COLOR_RGB2GRAY", opencv::imgproc::COLOR_RGB2GRAY);
    push_ocv_const!("COLOR_BGRA2GRAY", opencv::imgproc::COLOR_BGRA2GRAY);
    push_ocv_const!("COLOR_GRAY2BGRA", opencv::imgproc::COLOR_GRAY2BGRA);
    push_ocv_const!("COLOR_BGR2HSV", opencv::imgproc::COLOR_BGR2HSV);
    push_ocv_const!("COLOR_HSV2BGR", opencv::imgproc::COLOR_HSV2BGR);
    push_ocv_const!("COLOR_BGR2HLS", opencv::imgproc::COLOR_BGR2HLS);
    push_ocv_const!("COLOR_BGR2LAB", opencv::imgproc::COLOR_BGR2Lab);
    push_ocv_const!("THRESH_BINARY", opencv::imgproc::THRESH_BINARY);
    push_ocv_const!("THRESH_BINARY_INV", opencv::imgproc::THRESH_BINARY_INV);
    push_ocv_const!("THRESH_TRUNC", opencv::imgproc::THRESH_TRUNC);
    push_ocv_const!("THRESH_TOZERO", opencv::imgproc::THRESH_TOZERO);
    push_ocv_const!("THRESH_TOZERO_INV", opencv::imgproc::THRESH_TOZERO_INV);
    push_ocv_const!("THRESH_OTSU", opencv::imgproc::THRESH_OTSU);
    push_ocv_const!("BGR2GRAY", opencv::imgproc::COLOR_BGR2GRAY);
    push_ocv_const!("GRAY2BGR", opencv::imgproc::COLOR_GRAY2BGR);
    push_ocv_const!("INTER_LINEAR", opencv::imgproc::INTER_LINEAR);
    push_ocv_const!("INTER_NEAREST", opencv::imgproc::INTER_NEAREST);
    push_ocv_const!("INTER_CUBIC", opencv::imgproc::INTER_CUBIC);
    push_ocv_const!("INTER_AREA", opencv::imgproc::INTER_AREA);
    push_ocv_const!("FLIP_VERTICAL", 0i64);
    push_ocv_const!("FLIP_HORIZONTAL", 1i64);
    push_ocv_const!("FLIP_BOTH", -1i64);
    push_ocv_const!("ROTATE_90_CLOCKWISE", 0i64);
    push_ocv_const!("ROTATE_180", 1i64);
    push_ocv_const!("ROTATE_90_COUNTERCLOCKWISE", 2i64);
    push_ocv_const!("FILLED", opencv::imgproc::FILLED);
    push_ocv_const!("LINE_8", opencv::imgproc::LINE_8);
    push_ocv_const!("LINE_AA", opencv::imgproc::LINE_AA);
    push_ocv_const!("FONT_HERSHEY_SIMPLEX", opencv::imgproc::FONT_HERSHEY_SIMPLEX);
    push_ocv_const!("TM_CCOEFF", opencv::imgproc::TM_CCOEFF);
    push_ocv_const!("TM_CCOEFF_NORMED", opencv::imgproc::TM_CCOEFF_NORMED);
    push_ocv_const!("TM_CCORR", opencv::imgproc::TM_CCORR);
    push_ocv_const!("TM_CCORR_NORMED", opencv::imgproc::TM_CCORR_NORMED);
    push_ocv_const!("TM_SQDIFF", opencv::imgproc::TM_SQDIFF);
    push_ocv_const!("TM_SQDIFF_NORMED", opencv::imgproc::TM_SQDIFF_NORMED);
    push_ocv_const!("NORM_MINMAX", opencv::core::NORM_MINMAX);
    push_ocv_const!("NORM_L1", opencv::core::NORM_L1);
    push_ocv_const!("NORM_L2", opencv::core::NORM_L2);
    push_ocv_const!("BORDER_CONSTANT", opencv::core::BORDER_CONSTANT);

    funcs
}
