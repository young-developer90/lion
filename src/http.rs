use std::io::Read;
use std::sync::OnceLock;

fn global_agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| ureq::Agent::new())
}

pub struct Response {
    pub status_code: i64,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

fn method_from_str(method: &str) -> Result<&'static str, String> {
    match method {
        "GET" | "get" | "Get" => Ok("GET"),
        "POST" | "post" | "Post" => Ok("POST"),
        "PUT" | "put" | "Put" => Ok("PUT"),
        "DELETE" | "delete" | "Delete" => Ok("DELETE"),
        "PATCH" | "patch" | "Patch" => Ok("PATCH"),
        "HEAD" | "head" | "Head" => Ok("HEAD"),
        _ => Err(format!("unsupported HTTP method: {}", method)),
    }
}

pub fn request(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: Option<&[u8]>,
) -> Result<Response, String> {
    let agent = global_agent();
    let method_upper = method_from_str(method)?;

    let mut req = match method_upper {
        "GET" => agent.get(url),
        "POST" => agent.post(url),
        "PUT" => agent.put(url),
        "DELETE" => agent.delete(url),
        "PATCH" => agent.patch(url),
        "HEAD" => agent.head(url),
        m => return Err(format!("unsupported HTTP method: {}", m)),
    };

    for (name, value) in headers {
        req = req.set(name, value);
    }

    let result = if let Some(b) = body {
        req.send_bytes(b)
    } else {
        req.call()
    };

    match result {
        Ok(r) => {
            let status = r.status() as i64;
            let mut resp_headers = Vec::new();
            for name in &[
                "content-type", "content-length", "location", "set-cookie",
                "server", "date", "cache-control", "last-modified", "etag",
            ] {
                if let Some(val) = r.header(name) {
                    resp_headers.push((name.to_string(), val.to_string()));
                }
            }
            let mut reader = r.into_reader();
            let mut body = Vec::new();
            reader.read_to_end(&mut body).map_err(|e| format!("read body: {}", e))?;
            Ok(Response {
                status_code: status,
                headers: resp_headers,
                body,
            })
        }
        Err(ureq::Error::Status(code, _)) => {
            Ok(Response {
                status_code: code as i64,
                headers: Vec::new(),
                body: vec![],
            })
        }
        Err(e) => Err(format!("HTTP request failed: {}", e)),
    }
}
