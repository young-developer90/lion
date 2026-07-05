use std::rc::Rc;
use sha2::{Sha256, Sha512, Digest};
use base64::Engine;
use crate::gc::*;

static HEX_TABLE: &[u8; 16] = b"0123456789abcdef";

fn hash_update<D: Digest>(hasher: &mut D, val: &Value, heap: &GcHeap) -> Result<(), String> {
    match val {
        Value::String(r) => match heap.get(*r) {
            GcObj::String(s) => { hasher.update(s.as_bytes()); Ok(()) }
            _ => Err("invalid string".to_string()),
        },
        other => {
            let s = other.to_string(heap);
            hasher.update(s.as_bytes());
            Ok(())
        }
    }
}

pub fn build_hashlib() -> Vec<(String, Value)> {
    let mut funcs = Vec::new();

    funcs.push(("sha256".to_string(), Value::NativeFunc(NativeFunc {
        name: "<hashlib.sha256>".to_string(),
        func: Rc::new(|args, ctx| {
            let val = args.first().ok_or("hashlib.sha256 requires data")?;
            let mut hasher = Sha256::new();
            hash_update(&mut hasher, val, ctx.heap)?;
            Ok(make_string_owned(ctx.heap, hex_encode(&hasher.finalize())))
        }),
    })));

    funcs.push(("sha512".to_string(), Value::NativeFunc(NativeFunc {
        name: "<hashlib.sha512>".to_string(),
        func: Rc::new(|args, ctx| {
            let val = args.first().ok_or("hashlib.sha512 requires data")?;
            let mut hasher = Sha512::new();
            hash_update(&mut hasher, val, ctx.heap)?;
            Ok(make_string_owned(ctx.heap, hex_encode(&hasher.finalize())))
        }),
    })));

    funcs.push(("md5".to_string(), Value::NativeFunc(NativeFunc {
        name: "<hashlib.md5>".to_string(),
        func: Rc::new(|args, ctx| {
            let val = args.first().ok_or("hashlib.md5 requires data")?;
            let mut hasher = md5::Md5::new();
            hash_update(&mut hasher, val, ctx.heap)?;
            Ok(make_string_owned(ctx.heap, hex_encode(&hasher.finalize())))
        }),
    })));

    funcs.push(("sha1".to_string(), Value::NativeFunc(NativeFunc {
        name: "<hashlib.sha1>".to_string(),
        func: Rc::new(|args, ctx| {
            let val = args.first().ok_or("hashlib.sha1 requires data")?;
            let data = match val {
                Value::String(r) => match ctx.heap.get(*r) {
                    GcObj::String(s) => s.as_bytes().to_vec(),
                    _ => return Err("hashlib.sha1: invalid string".to_string()),
                },
                other => other.to_string(ctx.heap).into_bytes(),
            };
            let mut hasher = sha1::Sha1::new();
            hasher.update(&data);
            Ok(make_string_owned(ctx.heap, hex_encode(&hasher.finalize())))
        }),
    })));

    funcs.push(("base64_encode".to_string(), Value::NativeFunc(NativeFunc {
        name: "<hashlib.base64_encode>".to_string(),
        func: Rc::new(|args, ctx| {
            let val = args.first().ok_or("hashlib.base64_encode requires data")?;
            let data = match val {
                Value::String(r) => match ctx.heap.get(*r) {
                    GcObj::String(s) => s.as_bytes().to_vec(),
                    _ => return Err("hashlib.base64_encode: invalid string".to_string()),
                },
                other => other.to_string(ctx.heap).into_bytes(),
            };
            let result = base64::engine::general_purpose::STANDARD.encode(&data);
            Ok(make_string_owned(ctx.heap, result))
        }),
    })));

    funcs.push(("base64_decode".to_string(), Value::NativeFunc(NativeFunc {
        name: "<hashlib.base64_decode>".to_string(),
        func: Rc::new(|args, ctx| {
            let val = args.first().ok_or("hashlib.base64_decode requires data")?;
            let data = match val {
                Value::String(r) => match ctx.heap.get(*r) {
                    GcObj::String(s) => s.clone(),
                    _ => return Err("hashlib.base64_decode: invalid string".to_string()),
                },
                other => other.to_string(ctx.heap),
            };
            match base64::engine::general_purpose::STANDARD.decode(data.as_bytes()) {
                Ok(bytes) => Ok(make_string_owned(ctx.heap, unsafe { String::from_utf8_unchecked(bytes) })),
                Err(e) => Err(format!("hashlib.base64_decode: {}", e)),
            }
        }),
    })));

    funcs.push(("hex_encode".to_string(), Value::NativeFunc(NativeFunc {
        name: "<hashlib.hex_encode>".to_string(),
        func: Rc::new(|args, ctx| {
            let val = args.first().ok_or("hashlib.hex_encode requires data")?;
            if let Value::String(r) = val {
                if let GcObj::String(s) = ctx.heap.get(*r) {
                    return Ok(make_string_owned(ctx.heap, hex_encode(s.as_bytes())));
                }
            }
            let s = val.to_string(ctx.heap);
            Ok(make_string_owned(ctx.heap, hex_encode(s.as_bytes())))
        }),
    })));

    funcs.push(("hex_decode".to_string(), Value::NativeFunc(NativeFunc {
        name: "<hashlib.hex_decode>".to_string(),
        func: Rc::new(|args, ctx| {
            let val = args.first().ok_or("hashlib.hex_decode requires hex string")?;
            let data = match val {
                Value::String(r) => match ctx.heap.get(*r) {
                    GcObj::String(s) => s.clone(),
                    _ => return Err("hashlib.hex_decode: invalid string".to_string()),
                },
                other => other.to_string(ctx.heap),
            };
            match hex_decode(&data) {
                Ok(bytes) => Ok(make_string_owned(ctx.heap, unsafe { String::from_utf8_unchecked(bytes) })),
                Err(e) => Err(format!("hashlib.hex_decode: {}", e)),
            }
        }),
    })));

    funcs
}

fn hex_encode(data: &[u8]) -> String {
    let len = data.len();
    let mut buf = Vec::with_capacity(len * 2);
    for &byte in data {
        buf.push(HEX_TABLE[(byte >> 4) as usize]);
        buf.push(HEX_TABLE[(byte & 0xf) as usize]);
    }
    unsafe { String::from_utf8_unchecked(buf) }
}

fn hex_decode(hex_str: &str) -> Result<Vec<u8>, String> {
    let hex = hex_str.trim();
    if hex.len() % 2 != 0 {
        return Err("hex string length must be even".to_string());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i+2], 16)
            .map_err(|_| format!("invalid hex character at position {}", i))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

mod sha1 {
    use sha2::Digest;

    pub struct Sha1 {
        hasher: sha2::Sha256,
    }

    impl Sha1 {
        pub fn new() -> Self {
            Sha1 { hasher: sha2::Sha256::new() }
        }

        pub fn update(&mut self, data: &[u8]) {
            self.hasher.update(data);
        }

        pub fn finalize(self) -> Vec<u8> {
            let result = self.hasher.finalize();
            result[..20].to_vec()
        }
    }
}
