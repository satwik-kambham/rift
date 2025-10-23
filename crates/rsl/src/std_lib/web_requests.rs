use crate::primitive::Primitive;

pub fn get_request(arguments: Vec<Primitive>) -> Primitive {
    if arguments.len() == 1 {
        if let Primitive::String(url) = arguments.first().unwrap() {
            let body = reqwest::blocking::get(url).unwrap().text().unwrap();
            return Primitive::String(body);
        }
        return Primitive::Error("Expected url".to_string());
    }
    Primitive::Error("Expected 1 argument".to_string())
}

pub fn post_request(arguments: Vec<Primitive>) -> Primitive {
    if let Primitive::String(url) = arguments.first().unwrap() {
        if let Primitive::String(body) = arguments.get(1).unwrap() {
            let client = reqwest::blocking::Client::new();
            let response = client.post(url).body(body.clone()).send().unwrap();
            let content = response.text().unwrap();
            return Primitive::String(content);
        }
        return Primitive::Error("Expected string body".to_string());
    }
    return Primitive::Error("Expected url".to_string());
}

pub fn post_request_with_bearer_token(arguments: Vec<Primitive>) -> Primitive {
    if let Primitive::String(url) = arguments.first().unwrap() {
        if let Primitive::String(body) = arguments.get(1).unwrap() {
            if let Primitive::String(bearer_token) = arguments.get(2).unwrap() {
                let client = reqwest::blocking::Client::new();
                let response = client
                    .post(url)
                    .bearer_auth(bearer_token)
                    .body(body.clone())
                    .send()
                    .unwrap();
                let content = response.text().unwrap();
                return Primitive::String(content);
            }
            return Primitive::Error("Expected bearer token".to_string());
        }
        return Primitive::Error("Expected string body".to_string());
    }
    return Primitive::Error("Expected url".to_string());
}
