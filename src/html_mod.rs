use std::rc::Rc;

use crate::gc::*;

fn element_to_value(el: &scraper::ElementRef, heap: &mut GcHeap, recursive: bool) -> Value {
    let tag = el.value().name.local.to_string();
    let text: String = el.text().collect::<Vec<_>>().join(" ");
    let inner = el.inner_html();

    let mut attr_dict = Vec::new();
    for (k, v) in el.value().attrs() {
        attr_dict.push((make_string(heap, k), make_string(heap, v)));
    }

    let mut entries = Vec::new();
    entries.push((make_string(heap, "tag"), make_string(heap, &tag)));
    entries.push((make_string(heap, "text"), make_string(heap, &text.trim().to_string())));
    entries.push((make_string(heap, "html"), make_string(heap, &inner)));
    entries.push((
        make_string(heap, "attrs"),
        Value::Dict(heap.alloc(GcObj::Dict(attr_dict))),
    ));

    if recursive {
        let mut children = Vec::new();
        for child in el.children() {
            if let scraper::node::Node::Element(_) = child.value() {
                if let Some(child_el) = scraper::ElementRef::wrap(child) {
                    children.push(element_to_value(&child_el, heap, true));
                }
            }
        }
        entries.push((
            make_string(heap, "children"),
            make_list(heap, children),
        ));
    }

    Value::Dict(heap.alloc(GcObj::Dict(entries)))
}

pub fn build_html() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    funcs.push((
        "parse".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<html.parse>".to_string(),
            func: Rc::new(|args, ctx| {
                let html = args.first().map(|a| a.to_string(ctx.heap)).unwrap_or_default();
                let doc = scraper::Html::parse_document(&html);
                let root = doc.root_element();
                Ok(element_to_value(&root, ctx.heap, true))
            }),
        }),
    ));

    funcs.push((
        "parse_fragment".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<html.parse_fragment>".to_string(),
            func: Rc::new(|args, ctx| {
                let html = args.first().map(|a| a.to_string(ctx.heap)).unwrap_or_default();
                let frag = scraper::Html::parse_fragment(&html);
                let mut results = Vec::new();
                for node in frag.tree.root().children() {
                    if let scraper::node::Node::Element(_) = node.value() {
                        if let Some(el) = scraper::ElementRef::wrap(node) {
                            results.push(element_to_value(&el, ctx.heap, true));
                        }
                    }
                }
                Ok(make_list(ctx.heap, results))
            }),
        }),
    ));

    funcs.push((
        "select".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<html.select>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("html.select requires html string and css selector".to_string());
                }
                let html = args[0].to_string(ctx.heap);
                let selector_str = args[1].to_string(ctx.heap);
                let doc = scraper::Html::parse_fragment(&html);
                let selector = scraper::Selector::parse(&selector_str)
                    .map_err(|e| format!("invalid CSS selector: {:?}", e))?;
                let mut results = Vec::new();
                for element in doc.select(&selector) {
                    results.push(element_to_value(&element, ctx.heap, false));
                }
                Ok(make_list(ctx.heap, results))
            }),
        }),
    ));

    funcs.push((
        "text".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<html.text>".to_string(),
            func: Rc::new(|args, ctx| {
                let el_val = args.first().ok_or("html.text requires element")?;
                match el_val {
                    Value::Dict(r) => match ctx.heap.get(*r) {
                        GcObj::Dict(entries) => {
                            for (k, v) in entries {
                                if k.to_string(ctx.heap) == "text" {
                                    return Ok(v.clone());
                                }
                            }
                            Ok(make_string(ctx.heap, ""))
                        }
                        _ => Err("expected element dict".to_string()),
                    },
                    _ => Err("expected element dict".to_string()),
                }
            }),
        }),
    ));

    funcs.push((
        "attr".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<html.attr>".to_string(),
            func: Rc::new(|args, ctx| {
                if args.len() < 2 {
                    return Err("html.attr requires element and attr_name".to_string());
                }
                let attr_name = args[1].to_string(ctx.heap);
                match &args[0] {
                    Value::Dict(r) => match ctx.heap.get(*r) {
                        GcObj::Dict(entries) => {
                            for (k, v) in entries {
                                if k.to_string(ctx.heap) == "attrs" {
                                    if let Value::Dict(ar) = v {
                                        if let GcObj::Dict(attrs) = ctx.heap.get(*ar) {
                                            for (ak, av) in attrs {
                                                if ak.to_string(ctx.heap) == attr_name {
                                                    return Ok(av.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(Value::Nil)
                        }
                        _ => Err("expected element dict".to_string()),
                    },
                    _ => Err("expected element dict".to_string()),
                }
            }),
        }),
    ));

    funcs.push((
        "encode".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<html.encode>".to_string(),
            func: Rc::new(|args, ctx| {
                let text = args.first().map(|a| a.to_string(ctx.heap)).unwrap_or_default();
                let cap = text.len();
                let mut out = String::with_capacity(cap);
                for c in text.chars() {
                    match c {
                        '&' => out.push_str("&amp;"),
                        '<' => out.push_str("&lt;"),
                        '>' => out.push_str("&gt;"),
                        '"' => out.push_str("&quot;"),
                        '\'' => out.push_str("&#39;"),
                        c => out.push(c),
                    }
                }
                Ok(make_string_owned(ctx.heap, out))
            }),
        }),
    ));

    funcs.push((
        "decode".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "<html.decode>".to_string(),
            func: Rc::new(|args, ctx| {
                let text = args.first().map(|a| a.to_string(ctx.heap)).unwrap_or_default();
                let cap = text.len();
                let mut out = String::with_capacity(cap);
                let bytes = text.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'&' && i + 3 < bytes.len() {
                        if bytes[i+1] == b'a' && bytes[i+2] == b'm' && bytes[i+3] == b'p' && i + 4 < bytes.len() && bytes[i+4] == b';' {
                            out.push('&'); i += 5; continue;
                        }
                        if bytes[i+1] == b'l' && bytes[i+2] == b't' && bytes[i+3] == b';' {
                            out.push('<'); i += 4; continue;
                        }
                        if bytes[i+1] == b'g' && bytes[i+2] == b't' && bytes[i+3] == b';' {
                            out.push('>'); i += 4; continue;
                        }
                        if i + 5 < bytes.len() && bytes[i+1] == b'q' && bytes[i+2] == b'u' && bytes[i+3] == b'o' && bytes[i+4] == b't' && bytes[i+5] == b';' {
                            out.push('"'); i += 6; continue;
                        }
                        if i + 4 < bytes.len() && bytes[i+1] == b'#' {
                            if bytes[i+2] == b'3' && bytes[i+3] == b'9' && bytes[i+4] == b';' {
                                out.push('\''); i += 5; continue;
                            }
                            if i + 6 < bytes.len() && bytes[i+2] == b'x' && bytes[i+3] == b'2' && bytes[i+4] == b'7' && bytes[i+5] == b';' {
                                out.push('\''); i += 6; continue;
                            }
                            if i + 6 < bytes.len() && bytes[i+2] == b'x' && bytes[i+3] == b'2' && bytes[i+4] == b'F' && bytes[i+5] == b';' {
                                out.push('/'); i += 6; continue;
                            }
                        }
                    }
                    out.push(bytes[i] as char);
                    i += 1;
                }
                Ok(make_string_owned(ctx.heap, out))
            }),
        }),
    ));

    funcs
}
