use std::collections::HashMap;
use std::io::{BufRead, Error, Read};
use std::{io, thread, env};
use std::fs::{self, File};
use std::sync::{Arc, mpsc};
use fastcgi_client::{Client, Params, Request};
use tokio::sync::{Mutex, Semaphore};
use std::net::{TcpListener, TcpStream};
use std::io::BufReader;
use std::io::Write;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::time::{SystemTime, UNIX_EPOCH};
mod config;
mod parsing;
mod structs;
mod files;
mod loggers_other_misc;

use structs::*;
use files::*;


pub fn write_data(mut stream: &TcpStream, data: structs::Response) -> io::Result<()> {
    stream.write_all(&data.as_bytes());
    stream.flush();  
    Ok(())
}



async fn handle_client(mut stream: TcpStream, dirs: &mut fileDataTtl, map: &mut HashMap<String, Vec<u8>>, phpConnection: &Option<phpConnection>) -> Result<String, io::Error> {
    loop {
        let now = loggers_other_misc::get_epoch_now();
        let mut Method: &str = "";
        if now > (dirs.timestamp + dirs.ttl) {
            dirs.ttl = dirs.ttl;
            dirs.files = look_for_dirs_and_subdirs();
            dirs.timestamp = now;
            *map = HashMap::new();
        }

        let http_ver = "HTTP/1.1 ";


        let mut reader = BufReader::new(&mut stream);
        let mut buf_body = [0; 1024];
        reader.read(&mut buf_body).unwrap(); 

        let mut buffer_body = buf_body.to_vec();
        let mut buffer_body = String::from_utf8(buffer_body).unwrap();


        fn parse_body(buf: &mut String) -> Option<String>{
            let mut body = buf.clone();
            let mut body = body.split("Content-Length: ").collect::<Vec<&str>>();
            if let Some(value) = body.get(1) {            
                let mut body = body[1].split("\r\n\r\n").collect::<Vec<&str>>();
                return Some(body[1].to_string())  
            }
            None
        }
        buffer_body.retain(|c| c != '\0');

        let mut parsed_body = parse_body(&mut buffer_body).unwrap_or(String::new());
        buffer_body.retain(|c| c != '\r');

        let mut buffer_req = buffer_body.replace(parsed_body.as_str(), "");

        


        let mut buffer_page: Vec<u8> = Vec::new();
        let mut http_req_struct: HttpReq;

        match parsing::parse_request(buffer_req.replace("\n", " ").as_str()) {
            Ok(buf) => {
                http_req_struct = buf;
                http_req_struct.body = parsed_body;
            },
            Err(e) => {
                return Err(e);
            },
        }


        let file = FileData::get_by_http_subdir(http_req_struct.path.clone(), dirs.get_file().clone());
        let (status_code, mut buffer_page, mut file_type) = match file {
            Some(buf) => {
                if map.contains_key(&buf.get_path()) {
                    buffer_page = map.get(&buf.get_path()).unwrap().to_vec();
                } else {
                    buffer_page = open_file_by_path(buf.clone(), dirs.get_file().to_vec());
                    map.insert(buf.get_path(), buffer_page.clone());
                }
                (200, buffer_page, buf.get_content_type())
            }
            None => {
                buffer_page = b"<h1>404 NOT FOUND</h1>".to_vec();
                (404, buffer_page, "text/html".to_string())
            }
        };

        if file_type == "application/x-httpd-php"  &&  phpConnection.is_some() {

        let split = http_req_struct.path.split("/");


        let mut script_filename = env::current_dir()
            .unwrap()
            .join("web");

        for s in split {
            if s != "" {
               script_filename = script_filename.join(s);
            }
        }
        let script_filename = script_filename.to_str().unwrap();

        let script_name = &http_req_struct.path[http_req_struct.path.rfind('/').unwrap()..];
        println!("{} {}", script_filename, script_name);
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", 9000)).await.unwrap();
        let body_slice = http_req_struct.body_to_u8();


        let req_params = http_req_struct.params.clone();

        let mut params = Params::default()
            .request_method(http_req_struct.method.to_string())
            .script_name(script_name)
            .script_filename(script_filename)
            .request_uri(format!("{}{}", script_name, req_params))
            .query_string(req_params.chars().skip(1).collect::<String>())
            .document_uri(script_name)
            .remote_addr("127.0.0.1")
            .remote_port(12345)
            .server_addr("127.0.0.1")
            .server_port(8090)
            .server_name("RustedHttpd")
            .content_type("application/x-www-form-urlencoded")
            .content_length(body_slice.len());

            let client = Client::new(stream);

            
            let mut output = client.execute_once(Request::new(params.clone(), &mut &body_slice[..])).await;


            match output {
                Ok(o) => {
                    buffer_page = o.stdout.clone().unwrap();
                    file_type = "text/html".to_string();
                },
                Err(e) => {
                    buffer_page = b"<h1>500 Internal Server Error</h1>".to_vec();
                    file_type = "text/html".to_string();
                }
            }    



        }




let mut res_headers = format!(
    "Content-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\nServer: RustedHttpd\r\n",
    buffer_page.len(),
    file_type + "; charset=utf-8"
);

let mut seen_content_length = false;

for (key, value) in http_req_struct.clone().headers {
    if key.eq_ignore_ascii_case("Content-Length") {
        if seen_content_length {
            continue;
        }
        seen_content_length = true;
    }
    res_headers.push_str(&format!("{}: {}\r\n", key, value));
}


        let response = format!(
            "{}{} {}\r\n{}\r\n",
            http_ver,
            status_code,
            match status_code {
                200 => "OK",
                404 => "Not Found",
                _ => "Internal Server Error", // Add more cases as needed
            },
            res_headers
        );

        if let Err(e) = stream.write_all(response.as_bytes()) {
            return Err(e);
        }

        if let Err(e) = stream.write_all(&buffer_page) {
            return Err(e);
        }

        if let Err(e) = stream.flush() {
            return Err(e);
        }

        if !should_keep_alive(http_req_struct.headers.get("Connection")) {
            break;
        }
    }

    Ok("Ok".to_string())
}

fn should_keep_alive(http_req: std::option::Option<&std::string::String>) -> bool {
    if let Some(connection_header) = http_req {
        connection_header == "Keep-Alive: timeout=5, max=1000"
    } else {
        false
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut senders: Vec<Sender<TcpStream>> = vec![];
    let NUM_THREADS: usize;
    let parsed = config::parse_config();
    NUM_THREADS = parsed.server.threads;

    let mut phpConnection: Option<phpConnection> = None;
        
    if parsed.get_php().is_some() == true {
        let phpDetails = parsed.get_php().clone().unwrap();
        let conn = tokio::net::TcpStream::connect((phpDetails.server.as_str(), phpDetails.port)).await;

        match conn {
            Ok(c) => {
                phpConnection = Some(phpConnection {
                    connection: c,
                    is_enabled: true,
                });
            },
            Err(e) => {
                    panic!("PHP extension not working, check address:port")
                }
        }
    }

    let phpConnectionArc = Arc::new(phpConnection);

    for i in 0..NUM_THREADS {
        let (sender, receiver): (Sender<TcpStream>, _) = unbounded();
        senders.push(sender.clone());
        let receiver = receiver.clone();
        tokio::spawn(process_message(receiver, phpConnectionArc.clone(), parsed.clone(), i.try_into().unwrap()));
    }

    let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", parsed.server.port))?;

    for (i, stream) in listener.incoming().enumerate() {
        let stream = stream?;
        senders[i % NUM_THREADS].send(stream).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    Ok(())
}




async fn process_message(receiver: Receiver<TcpStream>, phpConnection: Arc<Option<phpConnection>> ,  config: config::ServerConfig, idx: i32) {
    let mut dirs: fileDataTtl = fileDataTtl {
        timestamp:  loggers_other_misc::get_epoch_now(),
        files: look_for_dirs_and_subdirs(),
        ttl: config.server.ttl,
    };


    println!("Worker thread {} started and ready to accept connections", idx);

    let mut hashMap: HashMap<String, Vec<u8>> = HashMap::new();
    while let Ok(stream) = receiver.recv() {
        //println!("{:#?}", stream);
        if let Err(e) = handle_client(stream, &mut dirs, &mut hashMap, &phpConnection).await {
        }
    }
}


