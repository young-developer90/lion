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

fn alloc_handle(mat: Mat) -> i64 {
    let mut guard = ensure_images().lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    let map = guard.as_mut().unwrap();
    let mut hg = NEXT_HANDLE.lock().unwrap();
    let handle = *hg;
    *hg += 1;
    map.insert(handle, mat);
    handle
}

fn get_mat(handle: i64) -> Result<Mat, String> {
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

macro_rules! ocv_const {
    ($e:expr) => { $e as i64 }
}

pub fn build_opencv() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    let imread_flag = opencv::imgcodecs::IMREAD_COLOR;
    let inter_linear = opencv::imgproc::INTER_LINEAR;
    let border_default = opencv::core::BORDER_DEFAULT;
    funcs.push(("imread".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.imread>".to_string(),
        func: Rc::new(move |args, ctx| {
            let path = arg_str(args, ctx, 0, "imread")?;
            let mat = opencv::imgcodecs::imread(path, imread_flag)
                .map_err(|e| format!("imread failed: {}", e))?;
            if mat.empty() {
                return Err("imread: could not read image".to_string());
            }
            Ok(Value::Int(alloc_handle(mat)))
        }),
    })));
    funcs.push(("imwrite".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.imwrite>".to_string(),
        func: Rc::new(|args, ctx| {
            let path = arg_str(args, ctx, 0, "imwrite")?;
            let handle = arg_int(args, 1, "imwrite")?;
            let mat = get_mat(handle)?;
            opencv::imgcodecs::imwrite(path, &mat, &opencv::core::Vector::new())
                .map_err(|e| format!("imwrite failed: {}", e))?;
            Ok(Value::Bool(true))
        }),
    })));
    funcs.push(("cvt_color".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.cvt_color>".to_string(),
        func: Rc::new(move |args, _ctx| {
            let handle = arg_int(args, 0, "cvt_color")?;
            let code = arg_int(args, 1, "cvt_color")?;
            let src = get_mat(handle)?;
            let mut dst = Mat::default();
            opencv::imgproc::cvt_color(&src, &mut dst, code as i32, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                .map_err(|e| format!("cvt_color failed: {}", e))?;
            Ok(Value::Int(alloc_handle(dst)))
        }),
    })));
    funcs.push(("resize".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.resize>".to_string(),
        func: Rc::new(move |args, _ctx| {
            let handle = arg_int(args, 0, "resize")?;
            let width = arg_int(args, 1, "resize")?;
            let height = arg_int(args, 2, "resize")?;
            let src = get_mat(handle)?;
            let mut dst = Mat::default();
            let dsize = opencv::core::Size { width: width as i32, height: height as i32 };
            opencv::imgproc::resize(&src, &mut dst, dsize, 0.0, 0.0, inter_linear)
                .map_err(|e| format!("resize failed: {}", e))?;
            Ok(Value::Int(alloc_handle(dst)))
        }),
    })));
    funcs.push(("gaussian_blur".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.gaussian_blur>".to_string(),
        func: Rc::new(move |args, _ctx| {
            let handle = arg_int(args, 0, "gaussian_blur")?;
            let kx = arg_int(args, 1, "gaussian_blur")?;
            let ky = arg_int(args, 2, "gaussian_blur")?;
            let sigma = arg_float(args, 3, "gaussian_blur")?;
            let src = get_mat(handle)?;
            let mut dst = Mat::default();
            let ksize = opencv::core::Size { width: kx as i32, height: ky as i32 };
            opencv::imgproc::gaussian_blur(&src, &mut dst, ksize, sigma, sigma, border_default, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                .map_err(|e| format!("gaussian_blur failed: {}", e))?;
            Ok(Value::Int(alloc_handle(dst)))
        }),
    })));
    funcs.push(("canny".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.canny>".to_string(),
        func: Rc::new(|args, _ctx| {
            let handle = arg_int(args, 0, "canny")?;
            let low = arg_float(args, 1, "canny")?;
            let high = arg_float(args, 2, "canny")?;
            let src = get_mat(handle)?;
            let mut dst = Mat::default();
            opencv::imgproc::canny(&src, &mut dst, low, high, 3, false)
                .map_err(|e| format!("canny failed: {}", e))?;
            Ok(Value::Int(alloc_handle(dst)))
        }),
    })));
    funcs.push(("threshold".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.threshold>".to_string(),
        func: Rc::new(|args, _ctx| {
            let handle = arg_int(args, 0, "threshold")?;
            let thresh = arg_float(args, 1, "threshold")?;
            let maxval = arg_float(args, 2, "threshold")?;
            let typ = arg_int(args, 3, "threshold")?;
            let src = get_mat(handle)?;
            let mut dst = Mat::default();
            opencv::imgproc::threshold(&src, &mut dst, thresh, maxval, typ as i32)
                .map_err(|e| format!("threshold failed: {}", e))?;
            Ok(Value::Int(alloc_handle(dst)))
        }),
    })));
    funcs.push(("shape".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.shape>".to_string(),
        func: Rc::new(|args, ctx| {
            let handle = arg_int(args, 0, "shape")?;
            let mat = get_mat(handle)?;
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
        func: Rc::new(|args, _| {
            let handle = arg_int(args, 0, "free")?;
            let mut guard = ensure_images().lock().unwrap();
            if let Some(ref mut map) = *guard {
                map.remove(&handle);
            }
            Ok(Value::Nil)
        }),
    })));
    funcs.push(("imshow".to_string(), Value::NativeFunc(NativeFunc {
        name: "<opencv.imshow>".to_string(),
        func: Rc::new(|args, ctx| {
            let name = arg_str(args, ctx, 0, "imshow")?;
            let handle = arg_int(args, 1, "imshow")?;
            let mat = get_mat(handle)?;
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
    push_ocv_const!("THRESH_BINARY", opencv::imgproc::THRESH_BINARY);
    push_ocv_const!("THRESH_BINARY_INV", opencv::imgproc::THRESH_BINARY_INV);
    push_ocv_const!("BGR2GRAY", opencv::imgproc::COLOR_BGR2GRAY);
    push_ocv_const!("GRAY2BGR", opencv::imgproc::COLOR_GRAY2BGR);

    funcs
}
