#![feature(core)]
#![feature(collections)]

extern crate curl;
extern crate url;
extern crate "rustc-serialize" as rustc_serialize;
extern crate regex;

use std::collections::BTreeMap;
use curl::http;
use url::form_urlencoded;
use rustc_serialize::json::{self, ToJson};
use self::Parameters::*;

pub mod config;

#[derive(PartialEq, Clone)]
pub enum Parameters {
    Priority(i8),
    Title(String),
    Device(String),
    Sound(String),
    URL(String),
    URLTitle(String),
    Gist,
    Debug
}

#[derive(RustcDecodable)]
struct MessagesJson {
    status: isize,
    errors: Vec<String>
}

#[derive(RustcEncodable)]
struct GistPost {
    files: BTreeMap<String, json::Json>
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

pub fn gist(message: &str, title: String) -> Result<String, (u32, String)> {
    let mut content = BTreeMap::new();
    content.insert("content".to_string(), message.to_json());
    let mut gist_file = BTreeMap::new();
    gist_file.insert(title, content.to_json());
    let gist = GistPost {
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
                return Ok(response.html_url);
            }
        }
    }

    Err((0, format!("Generic: Couldn't post to Gist.")))
}

pub fn push(token: &str, user: &str, message: &str,
                       parameters: &[Parameters]) -> Result<(), Vec<String>> {
    // Keep these here for now to satisfy the borrow checker:
    let msg = if message.len() > 1024 {
        message[0..1024].as_slice()
    }
    else {
        message
    };
    let mut title = "po".to_string();
    let mut debug = false;

    let mut notification = vec![
        ("token".to_string(), token.to_string()),
        ("user".to_string(), user.to_string()),
        ("message".to_string(), msg.to_string())];

    // Copy the parameters collection into a vector we own; slightly inefficent
    // but much more convenient for the caller.
    let mut para = Vec::new();
    para.push_all(parameters);

    for parameter in para.into_iter() {
        match parameter {
            Priority(p)  => notification.push(("priority".to_string(), p.to_string())),
            Title(t)     => {
                notification.push(("title".to_string(), t.clone()));
                title = t;
            },
            Device(d)    => notification.push(("device".to_string(), d)),
            Sound(s)     => notification.push(("sound".to_string(), s)),
            URL(u)       => notification.push(("url".to_string(), u)),
            URLTitle(ut) => notification.push(("url_title".to_string(), ut)),
            Gist         => {
                if let Ok(gist_url) = gist(message, title.clone()) {
                    notification.push(("url".to_string(), gist_url));
                    notification.push(("url_title".to_string(),
                        "Full Output (GitHub Gist)".to_string()));
                }
            },
            Debug        => debug = true
        }
    }

    let body = form_urlencoded::serialize_owned(notification.as_slice());
    if debug {
        println!("push body:\n{}", body);
    }
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

pub fn send_with_url(token: &str, user: &str, message: &str, priority: i8,
            title: Option<&str>, device: Option<&str>,
            sound: Option<&str>, url: Option<&str>,
            url_title: Option<&str>) -> Result<(), Vec<String>> {
    let mut parameters: Vec<Parameters> = vec![Parameters::Priority(priority)];

    if let Some(t) = title {
        parameters.push(Parameters::Title(t.to_string()));
    }
    if let Some(d) = device {
        parameters.push(Parameters::Device(d.to_string()));
    }
    if let Some(s) = sound {
        parameters.push(Parameters::Sound(s.to_string()));
    }
    if let Some(u) = url {
        parameters.push(Parameters::URL(u.to_string()));
    }
    if let Some(ut) = url_title {
        parameters.push(Parameters::URLTitle(ut.to_string()));
    }
    push(token, user, message, parameters.as_slice())
}

pub fn send(token: &str, user: &str, message: &str, priority: i8,
            title: Option<&str>, device: Option<&str>,
            sound: Option<&str>) -> Result<(), Vec<String>> {
    send_with_url(token, user, message, priority, title, device, sound, None, None)
}

pub fn send_gist(token: &str, user: &str, message: &str, priority: i8,
                 title: Option<&str>, device: Option<&str>,
                 sound: Option<&str>) -> Result<(), Vec<String>> {
    let mut parameters: Vec<Parameters> = vec![Parameters::Priority(priority)];

    if let Some(t) = title {
        parameters.push(Parameters::Title(t.to_string()));
    }
    if let Some(d) = device {
        parameters.push(Parameters::Device(d.to_string()));
    }
    if let Some(s) = sound {
        parameters.push(Parameters::Sound(s.to_string()));
    }
    parameters.push(Parameters::Gist);
    push(token, user, message, parameters.as_slice())
}

pub fn send_basic(token: &str, user: &str,
                  message: &str) -> Result<(), Vec<String>> {
    return push(token, user, message, vec![].as_slice());
}
