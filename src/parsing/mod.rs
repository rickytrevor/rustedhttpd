use std::{collections::HashMap, io};

use crate::structs::HttpReq;

pub fn parse_req_buffer(Buf: Vec<String>) -> String{
    let mut buffer = String::new();
    for line in Buf {
        buffer.push_str(&line.replace("%20", " "));
    }

    let mut path = match buffer.split("GET").collect::<Vec<&str>>().get(1) {
        Some(p) => p,
        None => "",
    };
    path = match path.split("HTTP/1.1").collect::<Vec<&str>>().get(0) {
        Some(p) => p,
        None => path,
    };
    path = path.trim();
    path.to_string()
}



pub fn parse_request(input: &str) -> Result<HttpReq, io::Error> {
    println!("{}", input);
    let lines: Vec<&str> = input.lines().collect();

    if lines.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "InvalidData"));
    }

    let first_line = lines.first().unwrap().to_string();
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string().replace("%20", " ");

    let mut headers = HashMap::new();
    let mut params = None;
    let mut content_length = None;

    for line in lines.iter().skip(1) {
        let mut parts = line.splitn(2, ':');
        if let Some(key) = parts.next() {
            if let Some(value) = parts.next() {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                headers.insert(key.clone(), value.clone());

                if key.to_lowercase() == "host" {
                    // Extract parameters from the path after the '?' character
                    if let Some(index) = path.find('?') {
                        params = Some(path[index + 1..].to_string());
                    }
                }

                if key.to_lowercase() == "content-length" {
                    content_length = Some(value.parse::<usize>().unwrap_or(0));
                }
            }
        }
    }


    let http_req = HttpReq {
        method: method,
        path: path,
        headers: headers,
        params: params.unwrap_or(String::from("")),
        body: String::new() // TODO Unificare il parsing del body
    };

    Ok(http_req)
}

pub fn parse_content_type(path: &str) -> String {
    let mut content_type = String::from("");
    let mut path = path.split(".").collect::<Vec<&str>>();
    let extension = path.pop().unwrap();

    match extension {
        "html" => content_type = String::from("text/html"),
        "php" => content_type = String::from("application/x-httpd-php"),
        "css" => content_type = String::from("text/css"),
        "js" => content_type = String::from("text/javascript"),
        "jpg" => content_type = String::from("image/jpeg"),
        "png" => content_type = String::from("image/png"),
        "gif" => content_type = String::from("image/gif"),
        "ico" => content_type = String::from("image/x-icon"),
        "zip" => content_type = String::from("application/zip"),
        "pdf" => content_type = String::from("application/pdf"),
        "json" => content_type = String::from("application/json"),
        "mp4" => content_type = String::from("video/mp4"),
        "mp3" => content_type = String::from("audio/mpeg"),
        _ => content_type = String::from("text/html"),
    }

    content_type
}