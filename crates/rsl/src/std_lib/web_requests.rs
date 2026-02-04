use crate::primitive::Primitive;
use crate::std_lib::args;
use rsl_macros::rsl_native;

#[rsl_native]
pub fn get_request(arguments: Vec<Primitive>) -> Primitive {
    let url = args!(arguments; url: String);
    match reqwest::blocking::get(&url) {
        Ok(response) => {
            let status = response.status();
            match response.text() {
                Ok(body) if status.is_success() => Primitive::String(body),
                Ok(body) => Primitive::Error(format!("GET {} returned {}: {}", url, status, body)),
                Err(err) => Primitive::Error(format!("GET {} failed to read body: {}", url, err)),
            }
        }
        Err(err) => Primitive::Error(format!("GET {} failed: {}", url, err)),
    }
}

#[rsl_native]
pub fn post_request(arguments: Vec<Primitive>) -> Primitive {
    let (url, body) = args!(arguments; url: String, body: String);
    let client = reqwest::blocking::Client::new();
    match client.post(&url).body(body.clone()).send() {
        Ok(response) => {
            let status = response.status();
            match response.text() {
                Ok(content) if status.is_success() => Primitive::String(content),
                Ok(content) => {
                    Primitive::Error(format!("POST {} returned {}: {}", url, status, content))
                }
                Err(err) => Primitive::Error(format!("POST {} failed to read body: {}", url, err)),
            }
        }
        Err(err) => Primitive::Error(format!("POST {} failed: {}", url, err)),
    }
}

#[rsl_native]
pub fn post_request_with_bearer_token(arguments: Vec<Primitive>) -> Primitive {
    let (url, body, bearer_token) =
        args!(arguments; url: String, body: String, bearer_token: String);
    let client = reqwest::blocking::Client::new();
    match client
        .post(&url)
        .bearer_auth(bearer_token)
        .body(body.clone())
        .send()
    {
        Ok(response) => {
            let status = response.status();
            match response.text() {
                Ok(content) if status.is_success() => Primitive::String(content),
                Ok(content) => {
                    Primitive::Error(format!("POST {} returned {}: {}", url, status, content))
                }
                Err(err) => Primitive::Error(format!("POST {} failed to read body: {}", url, err)),
            }
        }
        Err(err) => Primitive::Error(format!("POST {} failed: {}", url, err)),
    }
}
