use std::rc::Rc;

use crate::gc::*;
use crate::http;

fn response_to_dict(resp: http::Response, heap: &mut GcHeap) -> Value {
    let body_str = String::from_utf8_lossy(&resp.body).to_string();
    let mut header_dict = Vec::new();
    for (name, value) in &resp.headers {
        header_dict.push((
            make_string(heap, name),
            make_string(heap, value),
        ));
    }
    let entries = vec![
        (make_string(heap, "status"), Value::Int(resp.status_code as i64)),
        (
            make_string(heap, "headers"),
            Value::Dict(heap.alloc(GcObj::Dict(header_dict))),
        ),
        (make_string(heap, "body"), make_string(heap, &body_str)),
    ];
    Value::Dict(heap.alloc(GcObj::Dict(entries)))
}

pub fn build_http() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    funcs.push((
        "get".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<http.get>".to_string(),
            func: Rc::new(|args, ctx| {
                let url = args
                    .first()
                    .map(|a| a.to_string(ctx.heap))
                    .unwrap_or_default();
                match http::request("GET", &url, &[], None) {
                    Ok(resp) => Ok(response_to_dict(resp, ctx.heap)),
                    Err(e) => Err(e),
                }
            }),
        }),
    ));

    funcs.push((
        "post".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<http.post>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("http.post requires url and body".to_string());
                }
                let url = args[0].to_string(ctx.heap);
                let body = args[1].to_string(ctx.heap);
                match http::request("POST", &url, &[], Some(body.as_bytes())) {
                    Ok(resp) => Ok(response_to_dict(resp, ctx.heap)),
                    Err(e) => Err(e),
                }
            }),
        }),
    ));

    funcs.push((
        "request".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<http.request>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("http.request requires method and url".to_string());
                }
                let method = args[0].to_string(ctx.heap);
                let url = args[1].to_string(ctx.heap);

                let mut headers = Vec::new();
                if let Some(h) = args.get(2) {
                    if let Value::Dict(r) = h {
                        if let GcObj::Dict(entries) = ctx.heap.get(*r) {
                            for (k, v) in entries {
                                let key = k.to_string(ctx.heap);
                                let val = v.to_string(ctx.heap);
                                headers.push((key, val));
                            }
                        }
                    }
                }

                let body = args.get(3).map(|a| a.to_string(ctx.heap));
                let body_bytes = body.as_ref().map(|b| b.as_bytes());

                match http::request(&method, &url, &headers, body_bytes) {
                    Ok(resp) => Ok(response_to_dict(resp, ctx.heap)),
                    Err(e) => Err(e),
                }
            }),
        }),
    ));

    funcs
}
