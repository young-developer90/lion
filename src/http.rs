pub struct Response {
    pub status_code: i64,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub fn request(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: Option<&[u8]>,
) -> Result<Response, String> {
    let agent = ureq::Agent::new();
    let method_upper = method.to_uppercase();

    let mut req = match method_upper.as_str() {
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
            let body_str = r.into_string().map_err(|e| format!("read body: {}", e))?;
            Ok(Response {
                status_code: status,
                headers: resp_headers,
                body: body_str.into_bytes(),
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
