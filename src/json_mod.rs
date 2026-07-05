use std::rc::Rc;

use crate::gc::*;

fn skip_ws(s: &[u8], pos: &mut usize) {
    while *pos < s.len() && s[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn parse_json_value(s: &[u8], pos: &mut usize, heap: &mut GcHeap) -> Result<Value, String> {
    skip_ws(s, pos);
    if *pos >= s.len() {
        return Err("json.parse: unexpected end".to_string());
    }
    match s[*pos] {
        b'"' => {
            *pos += 1;
            let mut string = String::new();
            while *pos < s.len() && s[*pos] != b'"' {
                if s[*pos] == b'\\' {
                    *pos += 1;
                    if *pos >= s.len() { return Err("json.parse: unexpected end in string".to_string()); }
                    match s[*pos] {
                        b'"' => string.push('"'),
                        b'\\' => string.push('\\'),
                        b'/' => string.push('/'),
                        b'n' => string.push('\n'),
                        b'r' => string.push('\r'),
                        b't' => string.push('\t'),
                        b'u' => {
                            let hex = std::str::from_utf8(&s[*pos+1..(*pos+5).min(s.len())]).ok();
                            let code = hex.and_then(|h| u32::from_str_radix(h, 16).ok()).ok_or("json.parse: invalid unicode escape")?;
                            string.push(char::from_u32(code).ok_or("json.parse: invalid unicode code point")?);
                            *pos += 4;
                        }
                        _ => return Err(format!("json.parse: invalid escape '\\{}'", s[*pos] as char)),
                    }
                } else {
                    string.push(s[*pos] as char);
                }
                *pos += 1;
            }
            if *pos >= s.len() { return Err("json.parse: unterminated string".to_string()); }
            *pos += 1;
            Ok(make_string(heap, &string))
        }
        b't' => {
            if *pos + 4 <= s.len() && &s[*pos..*pos+4] == b"true" {
                *pos += 4;
                Ok(Value::Bool(true))
            } else {
                Err("json.parse: expected 'true'".to_string())
            }
        }
        b'f' => {
            if *pos + 5 <= s.len() && &s[*pos..*pos+5] == b"false" {
                *pos += 5;
                Ok(Value::Bool(false))
            } else {
                Err("json.parse: expected 'false'".to_string())
            }
        }
        b'n' => {
            if *pos + 4 <= s.len() && &s[*pos..*pos+4] == b"null" {
                *pos += 4;
                Ok(Value::Nil)
            } else {
                Err("json.parse: expected 'null'".to_string())
            }
        }
        b'[' => {
            *pos += 1;
            let mut items = Vec::new();
            skip_ws(s, pos);
            if *pos < s.len() && s[*pos] == b']' {
                *pos += 1;
                return Ok(make_list(heap, items));
            }
            loop {
                items.push(parse_json_value(s, pos, heap)?);
                skip_ws(s, pos);
                if *pos >= s.len() { return Err("json.parse: unterminated array".to_string()); }
                if s[*pos] == b']' { *pos += 1; break; }
                if s[*pos] != b',' { return Err("json.parse: expected ',' or ']'".to_string()); }
                *pos += 1;
            }
            Ok(make_list(heap, items))
        }
        b'{' => {
            *pos += 1;
            let mut entries = Vec::new();
            skip_ws(s, pos);
            if *pos < s.len() && s[*pos] == b'}' {
                *pos += 1;
                return Ok(Value::Dict(heap.alloc(GcObj::Dict(entries))));
            }
            loop {
                skip_ws(s, pos);
                if *pos >= s.len() || s[*pos] != b'"' { return Err("json.parse: expected string key".to_string()); }
                let key = parse_json_value(s, pos, heap)?;
                skip_ws(s, pos);
                if *pos >= s.len() || s[*pos] != b':' { return Err("json.parse: expected ':'".to_string()); }
                *pos += 1;
                let val = parse_json_value(s, pos, heap)?;
                entries.push((key, val));
                skip_ws(s, pos);
                if *pos >= s.len() { return Err("json.parse: unterminated object".to_string()); }
                if s[*pos] == b'}' { *pos += 1; break; }
                if s[*pos] != b',' { return Err("json.parse: expected ',' or '}'".to_string()); }
                *pos += 1;
            }
            Ok(Value::Dict(heap.alloc(GcObj::Dict(entries))))
        }
        b'-' | b'0'..=b'9' => {
            let start = *pos;
            if *pos < s.len() && s[*pos] == b'-' { *pos += 1; }
            while *pos < s.len() && s[*pos].is_ascii_digit() { *pos += 1; }
            let mut is_float = false;
            if *pos < s.len() && s[*pos] == b'.' {
                is_float = true;
                *pos += 1;
                while *pos < s.len() && s[*pos].is_ascii_digit() { *pos += 1; }
            }
            if *pos < s.len() && (s[*pos] == b'e' || s[*pos] == b'E') {
                is_float = true;
                *pos += 1;
                if *pos < s.len() && (s[*pos] == b'+' || s[*pos] == b'-') { *pos += 1; }
                while *pos < s.len() && s[*pos].is_ascii_digit() { *pos += 1; }
            }
            let num_str = std::str::from_utf8(&s[start..*pos]).map_err(|_| "json.parse: invalid number".to_string())?;
            if is_float {
                let n: f64 = num_str.parse().map_err(|_| "json.parse: invalid float".to_string())?;
                Ok(Value::Float(n))
            } else {
                let n: i64 = num_str.parse().map_err(|_| "json.parse: invalid int".to_string())?;
                Ok(Value::Int(n))
            }
        }
        c => Err(format!("json.parse: unexpected character '{}'", c as char)),
    }
}

fn json_stringify(val: &Value, heap: &GcHeap) -> String {
    match val {
        Value::Nil => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::UInt(n) => n.to_string(),
        Value::Float(n) => {
            if n.fract() == 0.0 && n.is_finite() {
                format!("{}.0", n)
            } else {
                n.to_string()
            }
        }
        Value::String(r) => {
            let s = match heap.get(*r) { GcObj::String(s) => s.clone(), _ => return "\"\"".to_string() };
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c.is_ascii_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        Value::List(r) => {
            let items = match heap.get(*r) { GcObj::List(items) => items, _ => return "[]".to_string() };
            let mut out = String::from('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 { out.push(','); }
                out.push_str(&json_stringify(item, heap));
            }
            out.push(']');
            out
        }
        Value::Dict(r) => {
            let entries = match heap.get(*r) { GcObj::Dict(entries) => entries, _ => return "{}".to_string() };
            let mut out = String::from('{');
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 { out.push(','); }
                out.push_str(&json_stringify(k, heap));
                out.push(':');
                out.push_str(&json_stringify(v, heap));
            }
            out.push('}');
            out
        }
        Value::Tuple(r) => {
            let items = match heap.get(*r) { GcObj::Tuple(items) => items, _ => return "[]".to_string() };
            let mut out = String::from('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 { out.push(','); }
                out.push_str(&json_stringify(item, heap));
            }
            out.push(']');
            out
        }
        _ => "null".to_string(),
    }
}

fn json_pretty(val: &Value, heap: &GcHeap, indent: &str, depth: usize) -> String {
    let pad = indent.repeat(depth);
    let child_pad = indent.repeat(depth + 1);
    match val {
        Value::Nil => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::UInt(n) => n.to_string(),
        Value::Float(n) => {
            if n.fract() == 0.0 && n.is_finite() { format!("{}.0", n) } else { n.to_string() }
        }
        Value::String(r) => {
            let s = match heap.get(*r) { GcObj::String(s) => s.clone(), _ => return "\"\"".to_string() };
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c.is_ascii_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        Value::List(r) => {
            let items = match heap.get(*r) { GcObj::List(items) => items, _ => return "[]".to_string() };
            if items.is_empty() { return "[]".to_string(); }
            let mut out = String::from("[\n");
            for (i, item) in items.iter().enumerate() {
                if i > 0 { out.push_str(",\n"); }
                out.push_str(&child_pad);
                out.push_str(&json_pretty(item, heap, indent, depth + 1));
            }
            out.push('\n');
            out.push_str(&pad);
            out.push(']');
            out
        }
        Value::Dict(r) => {
            let entries = match heap.get(*r) { GcObj::Dict(entries) => entries, _ => return "{}".to_string() };
            if entries.is_empty() { return "{}".to_string(); }
            let mut out = String::from("{\n");
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 { out.push_str(",\n"); }
                out.push_str(&child_pad);
                out.push_str(&json_pretty(k, heap, indent, depth + 1));
                out.push_str(": ");
                out.push_str(&json_pretty(v, heap, indent, depth + 1));
            }
            out.push('\n');
            out.push_str(&pad);
            out.push('}');
            out
        }
        Value::Tuple(r) => {
            let items = match heap.get(*r) { GcObj::Tuple(items) => items, _ => return "[]".to_string() };
            if items.is_empty() { return "[]".to_string(); }
            let mut out = String::from("[\n");
            for (i, item) in items.iter().enumerate() {
                if i > 0 { out.push_str(",\n"); }
                out.push_str(&child_pad);
                out.push_str(&json_pretty(item, heap, indent, depth + 1));
            }
            out.push('\n');
            out.push_str(&pad);
            out.push(']');
            out
        }
        _ => "null".to_string(),
    }
}

fn to_i64(val: &Value) -> Result<i64, String> {
    match val {
        Value::Int(n) => Ok(*n),
        Value::UInt(n) => Ok(*n as i64),
        Value::Float(n) => Ok(*n as i64),
        _ => Err(format!("cannot convert {} to int", val.type_name())),
    }
}

pub fn build_json() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    let json_parse_fn = Rc::new(|args: &[Value], ctx: &mut VmContext| -> Result<Value, String> {
        let s = args.first().ok_or("json.parse requires a string")?;
        let s_str = s.to_string(ctx.heap);
        let mut pos: usize = 0;
        let bytes = s_str.as_bytes();
        skip_ws(bytes, &mut pos);
        let result = parse_json_value(bytes, &mut pos, ctx.heap)?;
        skip_ws(bytes, &mut pos);
        if pos < bytes.len() {
            Err("json.parse: trailing characters".to_string())
        } else {
            Ok(result)
        }
    });

    let json_stringify_fn = Rc::new(|args: &[Value], ctx: &mut VmContext| -> Result<Value, String> {
        let val = args.first().ok_or("json.stringify requires a value")?;
        Ok(make_string(ctx.heap, &json_stringify(val, ctx.heap)))
    });

    funcs.push(("parse".to_string(), Value::NativeFunc(NativeFunc { name: "<json.parse>".to_string(), func: json_parse_fn.clone() })));
    funcs.push(("loads".to_string(), Value::NativeFunc(NativeFunc { name: "<json.loads>".to_string(), func: json_parse_fn })));
    funcs.push(("stringify".to_string(), Value::NativeFunc(NativeFunc { name: "<json.stringify>".to_string(), func: json_stringify_fn.clone() })));
    funcs.push(("dumps".to_string(), Value::NativeFunc(NativeFunc { name: "<json.dumps>".to_string(), func: json_stringify_fn })));

    funcs.push((
        "dump".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<json.dump>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 { return Err("json.dump requires value and path".to_string()); }
                let val = &args[0];
                let path = args[1].to_string(ctx.heap);
                let json_str = json_stringify(val, ctx.heap);
                std::fs::write(&path, &json_str).map_err(|e| format!("json.dump: {}", e))?;
                Ok(Value::Nil)
            }),
        }),
    ));
    funcs.push((
        "load".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<json.load>".to_string(),
            func: Rc::new(|args, ctx| {
                let path = args.first().ok_or("json.load requires a path")?.to_string(ctx.heap);
                let content = std::fs::read_to_string(&path).map_err(|e| format!("json.load: {}", e))?;
                let mut pos: usize = 0;
                let bytes = content.as_bytes();
                skip_ws(bytes, &mut pos);
                let result = parse_json_value(bytes, &mut pos, ctx.heap)?;
                skip_ws(bytes, &mut pos);
                if pos < bytes.len() {
                    Err("json.load: trailing characters".to_string())
                } else {
                    Ok(result)
                }
            }),
        }),
    ));
    funcs.push((
        "pretty".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<json.pretty>".to_string(),
            func: Rc::new(|args, ctx| {
                let val = args.first().ok_or("json.pretty requires a value")?;
                let indent = if args.len() > 1 { args[1].to_string(ctx.heap) } else { "  ".to_string() };
                Ok(make_string(ctx.heap, &json_pretty(val, ctx.heap, &indent, 0)))
            }),
        }),
    ));

    funcs
}
