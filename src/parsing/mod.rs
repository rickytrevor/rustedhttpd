use std::{collections::HashMap, io};
use regex::Regex;
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
//    let mut path = parts.next().unwrap_or_default().to_string().replace("%20", " ");
    let mut path =  parts.next().unwrap_or_default().to_string();
    let mut headers = HashMap::new();
    let mut params= String::new();

    for line in lines.iter().skip(1) {
        let mut parts = line.splitn(2, ':');
        if let Some(key) = parts.next() {
            if let Some(value) = parts.next() {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                headers.insert(key.clone(), value.clone());
            }
        }
    }

    let re = Regex::new(r"(\?.*?)( |$)").unwrap();
    if let Some(captures) = re.captures(&first_line) {
        params = captures.get(1).map_or(String::from(""), |m| m.as_str().to_string());
    }

    path = path.replace(&params, "");

    
    let http_req = HttpReq {
        method: method,
        path: path,
        headers: headers,
        params: params,
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