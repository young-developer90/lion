use std::rc::Rc;

use crate::gc::*;

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' => {
                if in_quotes {
                    current.push(',');
                } else {
                    fields.push(current.trim().to_string());
                    current = String::new();
                }
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

pub fn build_csv() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    funcs.push((
        "parse".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<csv.parse>".to_string(),
            func: Rc::new(|args, ctx| {
                let text = args
                    .first()
                    .map(|a| a.to_string(ctx.heap))
                    .unwrap_or_default();
                let mut rows = Vec::new();
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let fields: Vec<Value> = parse_csv_line(trimmed)
                        .into_iter()
                        .map(|f| make_string(ctx.heap, &f))
                        .collect();
                    rows.push(make_list(ctx.heap, fields));
                }
                Ok(make_list(ctx.heap, rows))
            }),
        }),
    ));

    funcs.push((
        "parse_header".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<csv.parse_header>".to_string(),
            func: Rc::new(|args, ctx| {
                let text = args
                    .first()
                    .map(|a| a.to_string(ctx.heap))
                    .unwrap_or_default();
                let mut lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
                if lines.is_empty() {
                    return Ok(make_list(ctx.heap, Vec::new()));
                }
                let headers: Vec<String> = parse_csv_line(lines[0]);
                let mut rows = Vec::new();
                for line in &lines[1..] {
                    let fields = parse_csv_line(line);
                    let mut dict_entries = Vec::new();
                    for (i, header) in headers.iter().enumerate() {
                        let val = fields.get(i).map(|s| s.as_str()).unwrap_or("");
                        dict_entries.push((
                            make_string(ctx.heap, header),
                            make_string(ctx.heap, val),
                        ));
                    }
                    rows.push(Value::Dict(ctx.heap.alloc(GcObj::Dict(dict_entries))));
                }
                Ok(make_list(ctx.heap, rows))
            }),
        }),
    ));

    funcs.push((
        "stringify".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<csv.stringify>".to_string(),
            func: Rc::new(|args, ctx| {
                let val = args.first().ok_or("csv.stringify requires rows")?;
                let rows = match val {
                    Value::List(r) => match ctx.heap.get(*r) {
                        GcObj::List(items) => items.clone(),
                        _ => return Err("expected list of rows".to_string()),
                    },
                    _ => return Err("expected list".to_string()),
                };

                let mut output = String::new();
                for row in rows {
                    let fields = match &row {
                        Value::List(r) => match ctx.heap.get(*r) {
                            GcObj::List(items) => items.clone(),
                            _ => continue,
                        },
                        Value::Dict(r) => {
                            if let GcObj::Dict(entries) = ctx.heap.get(*r) {
                                entries.iter().map(|(_, v)| v.clone()).collect()
                            } else {
                                continue;
                            }
                        }
                        _ => continue,
                    };

                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            output.push(',');
                        }
                        let s = field.to_string(ctx.heap);
                        if s.contains(',') || s.contains('"') || s.contains('\n') {
                            output.push('"');
                            output.push_str(&s.replace('"', "\"\""));
                            output.push('"');
                        } else {
                            output.push_str(&s);
                        }
                    }
                    output.push('\n');
                }
                Ok(make_string(ctx.heap, &output))
            }),
        }),
    ));

    funcs.push((
        "load".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<csv.load>".to_string(),
            func: Rc::new(|args, ctx| {
                let path = args
                    .first()
                    .map(|a| a.to_string(ctx.heap))
                    .unwrap_or_default();
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("cannot read '{}': {}", path, e))?;
                let mut rows = Vec::new();
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let fields: Vec<Value> = parse_csv_line(trimmed)
                        .into_iter()
                        .map(|f| make_string(ctx.heap, &f))
                        .collect();
                    rows.push(make_list(ctx.heap, fields));
                }
                Ok(make_list(ctx.heap, rows))
            }),
        }),
    ));

    funcs.push((
        "save".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<csv.save>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("csv.save requires path and rows".to_string());
                }
                let path = args[0].to_string(ctx.heap);
                let val = &args[1];
                let rows = match val {
                    Value::List(r) => match ctx.heap.get(*r) {
                        GcObj::List(items) => items.clone(),
                        _ => return Err("expected list of rows".to_string()),
                    },
                    _ => return Err("expected list".to_string()),
                };

                let mut output = String::new();
                for row in rows {
                    let fields = match &row {
                        Value::List(r) => match ctx.heap.get(*r) {
                            GcObj::List(items) => items.clone(),
                            _ => continue,
                        },
                        _ => continue,
                    };
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            output.push(',');
                        }
                        let s = field.to_string(ctx.heap);
                        if s.contains(',') || s.contains('"') || s.contains('\n') {
                            output.push('"');
                            output.push_str(&s.replace('"', "\"\""));
                            output.push('"');
                        } else {
                            output.push_str(&s);
                        }
                    }
                    output.push('\n');
                }
                std::fs::write(&path, &output)
                    .map_err(|e| format!("cannot write '{}': {}", path, e))?;
                Ok(Value::Bool(true))
            }),
        }),
    ));

    funcs
}
