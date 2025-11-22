use crate::primitive::Primitive;
use crate::std_lib::args;

pub fn get_request(arguments: Vec<Primitive>) -> Primitive {
    let url = args!(arguments; url: String);
    let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    Primitive::String(body)
}

pub fn post_request(arguments: Vec<Primitive>) -> Primitive {
    let (url, body) = args!(arguments; url: String, body: String);
    let client = reqwest::blocking::Client::new();
    let response = client.post(url).body(body.clone()).send().unwrap();
    let content = response.text().unwrap();
    Primitive::String(content)
}

pub fn post_request_with_bearer_token(arguments: Vec<Primitive>) -> Primitive {
    let (url, body, bearer_token) =
        args!(arguments; url: String, body: String, bearer_token: String);
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .bearer_auth(bearer_token)
        .body(body.clone())
        .send()
        .unwrap();
    let content = response.text().unwrap();
    Primitive::String(content)
}
