use std::f32::consts::E;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io;
use std::fs::{self, File};

mod utilities;

pub fn write_data(stream: &mut impl io::Write, data: utilities::Response) -> io::Result<()> {
    stream.write_all(&data.as_bytes())?;
    stream.flush()
}


fn handle_client(mut stream: TcpStream) {
    
    let mut buffer = Vec::new();
    let http_ver = "HTTP/1.1 ";

    let reader = BufReader::new(stream.try_clone().unwrap());

    for line in reader.lines() {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }
        buffer.push(line);
    }


    let path = utilities::parse_req_buffer(buffer);


//    let dirs = utilities::look_for_dirs_and_subdirs();
    let mut dirs = vec![];
    dirs = utilities::look_for_dirs_and_subdirs();


    let file = utilities::FileData::get_by_http_subdir(path, dirs.clone());

    println!("{:?}", file);
    let mut buffer_page: Vec<u8> = Vec::new();
    let mut status_code = 200;
    let mut file_type = String::from("");


    match file {
        Some(buf) =>  {
            buffer_page = utilities::open_file_by_path(buf.clone(), dirs);
            file_type =  buf.get_content_type();
            status_code = 200;
        },
        
        None => {
            status_code = 404;
            buffer_page = String::from("<h1>404 NOT FOUND</h1>").as_bytes().to_vec();
        }
    }


    let mut res = utilities::Response {
        http_version: http_ver,
        status_code: status_code,
        status_text: "OK",
        headers: Vec::new(),
        body: buffer_page,
    };


    let len = format!("Content-Length: {}", res.body.len());
    let content = format!("Content-Type: {}", file_type);

    res.headers = vec![
        &content,
        &len, // negli header della risposta devo specificare il numero di bytes nel body
        "Connection: close",
        "Server: La mia brutta copia di httpd"
    ];

    match write_data(&mut stream, res) {
        Ok(_) => println!("OK"),
        Err(e) => println!("Error: {:?}", e),
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8452")?;
    for stream in listener.incoming() {
        thread::spawn(move || {
            handle_client(stream.unwrap());
        });
    }
    Ok(())
}




