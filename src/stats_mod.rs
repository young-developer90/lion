use std::rc::Rc;

use crate::gc::*;

fn to_f64(val: &Value, heap: &GcHeap) -> Result<f64, String> {
    match val {
        Value::Int(n) => Ok(*n as f64),
        Value::UInt(n) => Ok(*n as f64),
        Value::Float(n) => Ok(*n),
        _ => Err(format!("expected number, got {}", val.type_name())),
    }
}

fn list_to_f64s(val: &Value, heap: &GcHeap) -> Result<Vec<f64>, String> {
    match val {
        Value::List(r) => match heap.get(*r) {
            GcObj::List(items) => items.iter().map(|v| to_f64(v, heap)).collect(),
            _ => Err("expected list".to_string()),
        },
        _ => Err("expected list".to_string()),
    }
}

pub fn build_stats() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    funcs.push((
        "sum".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.sum>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.sum requires list")?, ctx.heap)?;
                Ok(Value::Float(data.iter().sum()))
            }),
        }),
    ));

    funcs.push((
        "mean".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.mean>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.mean requires list")?, ctx.heap)?;
                if data.is_empty() {
                    return Err("empty list".to_string());
                }
                Ok(Value::Float(data.iter().sum::<f64>() / data.len() as f64))
            }),
        }),
    ));

    funcs.push((
        "median".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.median>".to_string(),
            func: Rc::new(|args, ctx| {
                let mut data = list_to_f64s(args.first().ok_or("stats.median requires list")?, ctx.heap)?;
                if data.is_empty() {
                    return Err("empty list".to_string());
                }
                data.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let len = data.len();
                if len % 2 == 0 {
                    Ok(Value::Float((data[len / 2 - 1] + data[len / 2]) / 2.0))
                } else {
                    Ok(Value::Float(data[len / 2]))
                }
            }),
        }),
    ));

    funcs.push((
        "mode".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.mode>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.mode requires list")?, ctx.heap)?;
                if data.is_empty() {
                    return Err("empty list".to_string());
                }
                use std::collections::HashMap;
                let mut counts: HashMap<i64, usize> = HashMap::new();
                for &v in &data {
                    *counts.entry(v as i64).or_insert(0) += 1;
                }
                let max_count = counts.values().cloned().max().unwrap_or(0);
                let modes: Vec<Value> = counts
                    .iter()
                    .filter(|(_, &c)| c == max_count)
                    .map(|(k, _)| Value::Int(*k))
                    .collect();
                Ok(make_list(ctx.heap, modes))
            }),
        }),
    ));

    funcs.push((
        "variance".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.variance>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.variance requires list")?, ctx.heap)?;
                if data.len() < 2 {
                    return Err("need at least 2 values".to_string());
                }
                let mean = data.iter().sum::<f64>() / data.len() as f64;
                let variance = data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
                Ok(Value::Float(variance))
            }),
        }),
    ));

    funcs.push((
        "std".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.std>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.std requires list")?, ctx.heap)?;
                if data.len() < 2 {
                    return Err("need at least 2 values".to_string());
                }
                let mean = data.iter().sum::<f64>() / data.len() as f64;
                let variance = data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
                Ok(Value::Float(variance.sqrt()))
            }),
        }),
    ));

    funcs.push((
        "min".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.min>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.min requires list")?, ctx.heap)?;
                data.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap())
                    .map(|v| Ok(Value::Float(v)))
                    .unwrap_or(Err("empty list".to_string()))
            }),
        }),
    ));

    funcs.push((
        "max".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.max>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.max requires list")?, ctx.heap)?;
                data.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap())
                    .map(|v| Ok(Value::Float(v)))
                    .unwrap_or(Err("empty list".to_string()))
            }),
        }),
    ));

    funcs.push((
        "quantile".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.quantile>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("stats.quantile requires list and q".to_string());
                }
                let mut data = list_to_f64s(&args[0], ctx.heap)?;
                let q = to_f64(&args[1], ctx.heap)?;
                if data.is_empty() {
                    return Err("empty list".to_string());
                }
                if !(0.0..=1.0).contains(&q) {
                    return Err("q must be between 0 and 1".to_string());
                }
                data.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let idx = q * (data.len() - 1) as f64;
                let lo = idx.floor() as usize;
                let hi = idx.ceil() as usize;
                if lo == hi {
                    Ok(Value::Float(data[lo]))
                } else {
                    let frac = idx - lo as f64;
                    Ok(Value::Float(data[lo] * (1.0 - frac) + data[hi] * frac))
                }
            }),
        }),
    ));

    funcs.push((
        "covariance".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.covariance>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("stats.covariance requires two lists".to_string());
                }
                let x = list_to_f64s(&args[0], ctx.heap)?;
                let y = list_to_f64s(&args[1], ctx.heap)?;
                if x.len() != y.len() || x.len() < 2 {
                    return Err("lists must have same length >= 2".to_string());
                }
                let n = x.len() as f64;
                let mx = x.iter().sum::<f64>() / n;
                let my = y.iter().sum::<f64>() / n;
                let cov: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| (xi - mx) * (yi - my)).sum::<f64>() / (n - 1.0);
                Ok(Value::Float(cov))
            }),
        }),
    ));

    funcs.push((
        "correlation".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.correlation>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("stats.correlation requires two lists".to_string());
                }
                let x = list_to_f64s(&args[0], ctx.heap)?;
                let y = list_to_f64s(&args[1], ctx.heap)?;
                if x.len() != y.len() || x.len() < 2 {
                    return Err("lists must have same length >= 2".to_string());
                }
                let n = x.len() as f64;
                let mx = x.iter().sum::<f64>() / n;
                let my = y.iter().sum::<f64>() / n;
                let cov: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| (xi - mx) * (yi - my)).sum();
                let var_x: f64 = x.iter().map(|xi| (xi - mx).powi(2)).sum();
                let var_y: f64 = y.iter().map(|yi| (yi - my).powi(2)).sum();
                if var_x == 0.0 || var_y == 0.0 {
                    return Ok(Value::Float(0.0));
                }
                Ok(Value::Float(cov / (var_x.sqrt() * var_y.sqrt())))
            }),
        }),
    ));

    funcs.push((
        "normalize".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.normalize>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.normalize requires list")?, ctx.heap)?;
                if data.is_empty() {
                    return Err("empty list".to_string());
                }
                let mn = data.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                let mx = data.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                let range = mx - mn;
                if range == 0.0 {
                    return Ok(make_list(ctx.heap, vec![Value::Float(0.0); data.len()]));
                }
                let result: Vec<Value> = data.iter().map(|&v| Value::Float((v - mn) / range)).collect();
                Ok(make_list(ctx.heap, result))
            }),
        }),
    ));

    funcs.push((
        "standardize".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<stats.standardize>".to_string(),
            func: Rc::new(|args, ctx| {
                let data = list_to_f64s(args.first().ok_or("stats.standardize requires list")?, ctx.heap)?;
                if data.len() < 2 {
                    return Err("need at least 2 values".to_string());
                }
                let n = data.len() as f64;
                let mean = data.iter().sum::<f64>() / n;
                let std = (data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
                if std == 0.0 {
                    return Ok(make_list(ctx.heap, vec![Value::Float(0.0); data.len()]));
                }
                let result: Vec<Value> = data.iter().map(|&v| Value::Float((v - mean) / std)).collect();
                Ok(make_list(ctx.heap, result))
            }),
        }),
    ));

    funcs
}
