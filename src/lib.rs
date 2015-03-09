#![feature(core)]
#![feature(io)]
#![feature(path)]

extern crate curl;
extern crate url;
extern crate "rustc-serialize" as rustc_serialize;
extern crate regex;

use curl::http;
use url::form_urlencoded;
use rustc_serialize::json::{self, ToJson};

pub mod config;

#[derive(RustcDecodable)]
struct MessagesJson {
    status: isize,
    errors: Vec<String>
}

#[derive(RustcEncodable)]
struct Gist {
    files: std::collections::BTreeMap<String, json::Json>
}

#[derive(RustcDecodable)]
struct GistResponse {
    html_url: String
}

fn api_error(response_body: &str) -> Result<(), Vec<String>> {
    let response: MessagesJson = json::decode(response_body).unwrap();

    if response.status != 1 {
        return Err(response.errors);
    }

    Err(vec![format!("general API error")])
}

pub fn send_with_url(token: &str, user: &str, message: &str, priority: i8,
            title: Option<&str>, device: Option<&str>,
            sound: Option<&str>, url: Option<&str>,
            url_title: Option<&str>) -> Result<(), Vec<String>> {
    // Keep these here for now to satisfy the borrow checker:
    let p_string = priority.to_string();
    let p_slice = p_string.as_slice();
    let msg = if message.len() > 1024 {
        message[0..1024].as_slice()
    }
    else {
         message
    };

    let mut parameters = vec![
        ("token", token),
        ("user", user),
        ("message", msg)];

    if let Some(t) = title {
        parameters.push(("title", t));
    }
    if priority != 0 {
        parameters.push(("priority", p_slice));
    }

    if let Some(d) = device {
        parameters.push(("device", d));
    }

    if let Some(s) = sound {
        parameters.push(("sound", s));
    }

    if let Some(u) = url {
        parameters.push(("url", u));
    }

    if let Some(ut) = url_title {
        parameters.push(("url_title", ut));
    }

    let body = form_urlencoded::serialize(parameters.into_iter());
    let mut handle = http::handle();
    let message = handle
                    .post("https://api.pushover.net/1/messages.json", body.as_slice())
                    .header("Content-Type", "application/x-www-form-urlencoded");
    match message.exec() {
        Ok(res) => {
            match res.get_code() {
                200 => Ok(()),
                400...499 => api_error(std::str::from_utf8(res.get_body()).unwrap()),
                n => Err(vec![format!("API error {}", n)])
            }
        },
        Err(code) => Err(vec![format!("curl error {}", code)])
    }
}

pub fn send(token: &str, user: &str, message: &str, priority: i8,
            title: Option<&str>, device: Option<&str>,
            sound: Option<&str>) -> Result<(), Vec<String>> {
    send_with_url(token, user, message, priority, title, device, sound, None, None)
}

pub fn send_gist(token: &str, user: &str, message: &str, priority: i8,
                 title: Option<&str>, device: Option<&str>,
                 sound: Option<&str>) -> Result<(), Vec<String>> {
    let gist_title = title.unwrap_or("po");
    let mut content = std::collections::BTreeMap::new();
    content.insert("content".to_string(), message.to_json());
    let mut gist_file = std::collections::BTreeMap::new();
    gist_file.insert(gist_title.to_string(), content.to_json());
    let gist = Gist {
        files: gist_file
    };

    if let Ok(json) = json::encode(&gist) {
        let mut handle = http::handle();
        let upload = handle
                        .post("https://api.github.com/gists", json.as_slice())
                        .header("Content-Type", "application/json")
                        .header("User-Agent", "po");
        if let Ok(res) = upload.exec() {
            if res.get_code() == 201 || res.get_code() == 200 {
                let body = std::str::from_utf8(res.get_body()).unwrap();
                let response: GistResponse = json::decode(body).unwrap();
                return send_with_url(token, user, message, priority, title, device,
                    sound, Some(response.html_url.as_slice()), Some("Full output (GitHub Gist)"));
            }
        }
    }

    send_with_url(token, user, message, priority, title, device, sound, None, None)
}
