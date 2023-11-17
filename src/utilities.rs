use std::fs::{self, File};
use std::io::Read;


pub struct Response<'a> {
    pub http_version: &'a str,
    pub status_code: u32,
    pub status_text: &'a str,
    pub headers: Vec<&'a str>,

    // Deve essere una stream di byte per gestire cose che non siano testo
    pub body: Vec<u8>,
}

impl Response<'_> {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend_from_slice(self.http_version.as_bytes());
        res.extend_from_slice(format!("{} {}\r\n", self.status_code, self.status_text).as_bytes());

        for header in &self.headers {
            res.extend_from_slice(header.as_bytes());
            res.extend_from_slice(b"\r\n");
        }

        res.extend_from_slice(b"\r\n");
        res.extend_from_slice(&self.body);

        res
    }
}

pub struct Request {
    http_version: String,
    method: String,
    path: String,
    headers: Vec<String>,
    body: String,
}

#[derive(Debug, Clone)]
pub struct FileData {
    path: String,
    name: String,
    http_subdir: String,
    content_type: String,
    is_dir: bool,

}


impl FileData {
    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_http_subdir(&self) -> String {
        self.http_subdir.clone()
    }

    pub fn get_by_http_subdir(mut subdir: String, dirs: Vec<FileData>) -> Option<FileData> {
            if subdir.len() > 1 {
                if subdir.chars().last().unwrap() == '/' {
                    subdir.pop();
                }
            }

//        println!("{:?}", dirs);
        for dir in dirs {
            if dir.get_http_subdir() == subdir {
                return Some(dir);
            }
        }
        None
    }

    pub fn get_content_type(&self) -> String {
        self.content_type.clone()
    }

    pub fn cloneVec(vec: Vec<FileData>) -> Vec<FileData> {
        let mut newVec = Vec::new();
        for file in vec {
            newVec.push(file.clone());
        }
        newVec
    }
}





pub fn look_for_files_in_this_dir(path: String) -> Vec<FileData> {
    let mut files = Vec::new();

    process_directory(&path, &mut files, true);

    files
}



pub fn process_if_path_is_dir(path: FileData) -> String {
        let mut buffer = String::from("<h1>Directory</h1>");
        let dirs_and_subdirs = look_for_files_in_this_dir(path.get_path());
//        println!("{:?}", path.get_http_subdir());
        for dir in dirs_and_subdirs {
            buffer.push_str("<ul>");
            buffer.push_str("<li>");
            buffer.push_str("<a href=\"");
            buffer.push_str(&dir.get_http_subdir());
            buffer.push_str("\">");
            buffer.push_str(&dir.get_name());
            buffer.push_str("</a>");
            buffer.push_str("</li>");
            buffer.push_str("</ul>");
            
        }
        buffer.push_str("</ul>");
        buffer
}


pub fn open_file_by_path(path: FileData, files: Vec<FileData> ) -> Vec<u8> {
    let mut fileVec = vec![];
    fileVec = files.clone();

    if path.is_dir {
        for file in files {
            if file.get_name() == "index.html" {
                return open_file_by_path(file, fileVec);
            }
        }
        return process_if_path_is_dir(path).as_bytes().to_vec();
    }

    let mut file = fs::File::open(&path.path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    contents
}

pub fn look_for_dirs_and_subdirs() -> Vec<FileData> {
    let mut files = Vec::new();
    let root_path = "./web";

    files.push(FileData {
        path: root_path.to_string(),
        name: String::from("web"),
        http_subdir: String::from("/"),
        content_type: String::from("text/html"),
        is_dir: true,
        
    });
    process_directory(root_path, &mut files, false);

    files
}


fn parse_content_type(path: &str) -> String {
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

fn process_directory(directory_path: &str, files: &mut Vec<FileData>, no_recursive: bool) {

//    println!("{:?}", "FUNZIONE CHIAMATA");

    if let Ok(paths) = std::fs::read_dir(directory_path) {
        for path in paths {
            if let Ok(entry) = path {
                let entry_path = entry.path();
                let entry_name = entry.file_name().to_string_lossy().replace("'", "\'").replace(" ", "\\").to_string();         
                let mut http_subdir = entry_path.to_str().unwrap().to_string();
                http_subdir.replace_range((0..5), "");
                let content_type = parse_content_type(&entry_path.to_str().unwrap());

                if entry_path.is_dir() {
                    files.push(FileData {
                        path: entry_path.to_str().unwrap().to_string(),
                        name: entry_name.clone(),
                        http_subdir: http_subdir.to_string(),
                        content_type: content_type,
                        is_dir: true,       
                    });

                    if !no_recursive {
                        process_directory(&entry_path.to_str().unwrap(), files, false);
                    }

                } else {

                    files.push(FileData {
                        path: entry_path.to_str().unwrap().to_string(),
                        name: entry_name.clone(),
                        http_subdir: http_subdir.to_string(),
                        content_type: content_type,
                        is_dir: false,
                    });
                }
            }
        }
    }
}


pub fn parse_req_buffer(Buf: Vec<String>) -> String{

    println!("{:?}", Buf);
    let mut buffer = String::new();
    for line in Buf {
        buffer.push_str(&line);
    }

    let mut path = buffer.split("GET").collect::<Vec<&str>>()[1];
    path = path.split("HTTP/1.1").collect::<Vec<&str>>()[0];
    path = path.trim();

    let mut pathStr = String::from(path);
    // replace %20 with a space
    pathStr = pathStr.replace("%20", " ");      
    pathStr

}