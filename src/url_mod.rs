use std::rc::Rc;

use crate::gc::*;

pub fn build_url() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    funcs.push((
        "parse".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<url.parse>".to_string(),
            func: Rc::new(|args, ctx| {
                let url_str = args
                    .first()
                    .map(|a| a.to_string(ctx.heap))
                    .unwrap_or_default();
                let parsed = url::Url::parse(&url_str)
                    .map_err(|e| format!("invalid URL '{}': {}", url_str, e))?;

                let mut entries = Vec::new();
                entries.push((make_string(ctx.heap, "scheme"), make_string(ctx.heap, parsed.scheme())));
                entries.push((make_string(ctx.heap, "host"), make_string(ctx.heap, parsed.host_str().unwrap_or(""))));
                entries.push((make_string(ctx.heap, "path"), make_string(ctx.heap, parsed.path())));
                entries.push((make_string(ctx.heap, "query"), make_string(ctx.heap, parsed.query().unwrap_or(""))));
                entries.push((make_string(ctx.heap, "fragment"), make_string(ctx.heap, parsed.fragment().unwrap_or(""))));
                entries.push((make_string(ctx.heap, "port"), Value::Int(parsed.port().unwrap_or(0) as i64)));

                let mut query_params = Vec::new();
                for (k, v) in parsed.query_pairs() {
                    query_params.push((
                        make_string(ctx.heap, &k),
                        make_string(ctx.heap, &v),
                    ));
                }
                entries.push((
                    make_string(ctx.heap, "query_params"),
                    Value::Dict(ctx.heap.alloc(GcObj::Dict(query_params))),
                ));

                Ok(Value::Dict(ctx.heap.alloc(GcObj::Dict(entries))))
            }),
        }),
    ));

    funcs.push((
        "encode".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<url.encode>".to_string(),
            func: Rc::new(|args, ctx| {
                let text = args
                    .first()
                    .map(|a| a.to_string(ctx.heap))
                    .unwrap_or_default();
                let encoded: String = url::form_urlencoded::byte_serialize(text.as_bytes()).collect();
                Ok(make_string(ctx.heap, &encoded))
            }),
        }),
    ));

    funcs.push((
        "decode".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<url.decode>".to_string(),
            func: Rc::new(|args, ctx| {
                let text = args
                    .first()
                    .map(|a| a.to_string(ctx.heap))
                    .unwrap_or_default();
                let decoded: String = url::form_urlencoded::parse(text.as_bytes())
                    .map(|(_, v)| v)
                    .collect();
                Ok(make_string(ctx.heap, &decoded))
            }),
        }),
    ));

    funcs.push((
        "join".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<url.join>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("url.join requires base and relative".to_string());
                }
                let base_str = args[0].to_string(ctx.heap);
                let relative = args[1].to_string(ctx.heap);
                let base = url::Url::parse(&base_str)
                    .map_err(|e| format!("invalid base URL '{}': {}", base_str, e))?;
                let joined = base.join(&relative)
                    .map_err(|e| format!("cannot join '{}' to '{}': {}", relative, base_str, e))?;
                Ok(make_string(ctx.heap, joined.as_str()))
            }),
        }),
    ));

    funcs
}
