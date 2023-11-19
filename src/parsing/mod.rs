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


pub fn parse_content_type(path: &str) -> String {
    let mut content_type = String::from("");
    let mut path = path.split(".").collect::<Vec<&str>>();
    let extension = path.pop().unwrap();

    match extension {
        "html" => content_type = String::from("text/html"),
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