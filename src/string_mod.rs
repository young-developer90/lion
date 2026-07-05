use std::rc::Rc;

use crate::gc::*;

fn to_i64(val: &Value) -> Result<i64, String> {
    match val {
        Value::Int(n) => Ok(*n),
        Value::UInt(n) => Ok(*n as i64),
        Value::Float(n) => Ok(*n as i64),
        _ => Err(format!("cannot convert {} to int", val.type_name())),
    }
}

pub fn build_string() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    funcs.push(("len".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.len>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.len requires a string")?.to_string(ctx.heap);
            Ok(Value::Int(s.len() as i64))
        }),
    })));

    funcs.push(("upper".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.upper>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.upper requires a string")?.to_string(ctx.heap);
            Ok(make_string(ctx.heap, &s.to_uppercase()))
        }),
    })));

    funcs.push(("lower".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.lower>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.lower requires a string")?.to_string(ctx.heap);
            Ok(make_string(ctx.heap, &s.to_lowercase()))
        }),
    })));

    funcs.push(("trim".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.trim>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.trim requires a string")?.to_string(ctx.heap);
            Ok(make_string(ctx.heap, s.trim()))
        }),
    })));

    funcs.push(("trim_start".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.trim_start>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.trim_start requires a string")?.to_string(ctx.heap);
            Ok(make_string(ctx.heap, s.trim_start()))
        }),
    })));

    funcs.push(("trim_end".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.trim_end>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.trim_end requires a string")?.to_string(ctx.heap);
            Ok(make_string(ctx.heap, s.trim_end()))
        }),
    })));

    funcs.push(("split".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.split>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.split requires a string")?.to_string(ctx.heap);
            let delim = if args.len() > 1 { args[1].to_string(ctx.heap) } else { " ".to_string() };
            let parts: Vec<Value> = s.split(&delim).map(|p| make_string(ctx.heap, p)).collect();
            Ok(make_list(ctx.heap, parts))
        }),
    })));

    funcs.push(("join".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.join>".to_string(),
        func: Rc::new(|args, ctx| {
            let list_val = args.first().ok_or("string.join requires a list")?;
            let sep = if args.len() > 1 { args[1].to_string(ctx.heap) } else { "".to_string() };
            let items = match list_val {
                Value::List(r) => match ctx.heap.get(*r) { GcObj::List(items) => items.clone(), _ => return Err("string.join: not a list".to_string()) },
                _ => return Err("string.join requires a list".to_string()),
            };
            let parts: Vec<String> = items.iter().map(|v| v.to_string(ctx.heap)).collect();
            Ok(make_string(ctx.heap, &parts.join(&sep)))
        }),
    })));

    funcs.push(("contains".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.contains>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.contains requires a string")?.to_string(ctx.heap);
            let sub = if args.len() > 1 { args[1].to_string(ctx.heap) } else { return Err("string.contains requires a substring".to_string()) };
            Ok(Value::Bool(s.contains(&sub)))
        }),
    })));

    funcs.push(("starts_with".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.starts_with>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.starts_with requires a string")?.to_string(ctx.heap);
            let prefix = if args.len() > 1 { args[1].to_string(ctx.heap) } else { return Err("string.starts_with requires a prefix".to_string()) };
            Ok(Value::Bool(s.starts_with(&prefix)))
        }),
    })));

    funcs.push(("ends_with".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.ends_with>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.ends_with requires a string")?.to_string(ctx.heap);
            let suffix = if args.len() > 1 { args[1].to_string(ctx.heap) } else { return Err("string.ends_with requires a suffix".to_string()) };
            Ok(Value::Bool(s.ends_with(&suffix)))
        }),
    })));

    funcs.push(("replace".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.replace>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.replace requires a string")?.to_string(ctx.heap);
            if args.len() < 3 { return Err("string.replace requires from and to".to_string()); }
            let from = args[1].to_string(ctx.heap);
            let to = args[2].to_string(ctx.heap);
            Ok(make_string(ctx.heap, &s.replace(&from, &to)))
        }),
    })));

    funcs.push(("reverse".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.reverse>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.reverse requires a string")?.to_string(ctx.heap);
            Ok(make_string(ctx.heap, &s.chars().rev().collect::<String>()))
        }),
    })));

    funcs.push(("repeat".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.repeat>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.repeat requires a string")?.to_string(ctx.heap);
            if args.len() < 2 { return Err("string.repeat requires a count".to_string()); }
            let n = to_i64(&args[1])?;
            Ok(make_string(ctx.heap, &s.repeat(n as usize)))
        }),
    })));

    funcs.push(("substring".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.substring>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.substring requires a string")?.to_string(ctx.heap);
            if args.len() < 2 { return Err("string.substring requires a start index".to_string()); }
            let start = to_i64(&args[1])? as usize;
            let end = if args.len() > 2 { to_i64(&args[2])? as usize } else { s.len() };
            let end = end.min(s.len());
            let start = start.min(end);
            Ok(make_string(ctx.heap, &s[start..end]))
        }),
    })));

    funcs.push(("bytes".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.bytes>".to_string(),
        func: Rc::new(|args, ctx| {
            let s = args.first().ok_or("string.bytes requires a string")?.to_string(ctx.heap);
            let bytes: Vec<Value> = s.bytes().map(|b| Value::Int(b as i64)).collect();
            Ok(make_list(ctx.heap, bytes))
        }),
    })));

    funcs.push(("from_bytes".to_string(), Value::NativeFunc(NativeFunc {
        name: "<string.from_bytes>".to_string(),
        func: Rc::new(|args, ctx| {
            let list_val = args.first().ok_or("string.from_bytes requires a list of ints")?;
            let items = match list_val {
                Value::List(r) => match ctx.heap.get(*r) { GcObj::List(items) => items.clone(), _ => return Err("string.from_bytes: not a list".to_string()) },
                _ => return Err("string.from_bytes requires a list".to_string()),
            };
            let mut bytes = Vec::new();
            for item in items {
                match item {
                    Value::Int(n) => bytes.push(n as u8),
                    _ => return Err("string.from_bytes: all elements must be integers".to_string()),
                }
            }
            Ok(make_string(ctx.heap, &String::from_utf8_lossy(&bytes)))
        }),
    })));

    funcs
}
