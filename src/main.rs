use std::collections::HashMap;
use std::io::{BufRead};
use std::{io, thread};
use std::fs::{self, File};
use std::sync::{Arc, mpsc};
use tokio::sync::{Mutex, Semaphore};
use utilities::FileData;
use utilities::fileDataTtl;

use std::net::{TcpListener, TcpStream};
use std::io::BufReader;
use std::io::Write;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::time::{SystemTime, UNIX_EPOCH};

mod utilities;

pub fn write_data(mut stream: &TcpStream, data: utilities::Response) -> io::Result<()> {
//    let mut writer = BufWriter::new(stream);
    stream.write_all(&data.as_bytes());
    stream.flush();  
    Ok(())
}


async fn handle_client(mut stream: &TcpStream, dirs: &mut fileDataTtl, map: &mut HashMap<String, Vec<u8>>) -> bool {

    let now = get_epoch_now();
    if now > (dirs.timestamp + dirs.ttl) {
        dirs.ttl = dirs.ttl;
        dirs.files = utilities::look_for_dirs_and_subdirs();
        dirs.timestamp = now;
        *map = HashMap::new();
    }


    let mut buffer = Vec::new();
    let http_ver = "HTTP/1.1 ";

    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }
        buffer.push(line);
    }

    let path = utilities::parse_req_buffer(buffer);




    let file = utilities::FileData::get_by_http_subdir(path, dirs.get_file().clone());


    let mut buffer_page: Vec<u8> = Vec::new();
    let mut status_code = 200;
    let mut file_type = String::from("");


    match file {
        Some(buf) =>  {
            if map.contains_key(&buf.get_path()) {
                buffer_page = map.get(&buf.get_path()).unwrap().to_vec();
            } else {
                buffer_page = utilities::open_file_by_path(buf.clone(), dirs.get_file().to_vec());
                map.insert(buf.get_path(), buffer_page.clone());
            }
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

    match write_data(&stream, res) {
        Ok(_) => true,
        Err(e) => false,
    }
}




#[tokio::main]
async fn main() -> io::Result<()> {
    let mut senders: Vec<Sender<TcpStream>> = vec![];
    let NUM_THREADS: usize = 16;

    for i in 0..NUM_THREADS {
        let (sender, receiver): (Sender<TcpStream>, _) = unbounded();
        senders.push(sender.clone());
        let (dir_sender, dir_receiver): (Sender<Vec<utilities::FileData>>, Receiver<Vec<utilities::FileData>>) = unbounded();

        let (task_sender, task_receiver): (Sender<bool>, _) = unbounded();
        let receiver = receiver.clone();
        let task_sender = task_sender.clone();        
        tokio::spawn(process_message(receiver, task_sender, dir_receiver.clone(),i.try_into().unwrap()));
    }

    let listener = TcpListener::bind("0.0.0.0:8453")?;

    let mut i = 0;
    for stream in listener.incoming() {
        let stream = stream?;
        senders[i].send(stream).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        i += 1;

        if i == NUM_THREADS {
            i = 0;
        }
    }

    Ok(())
}


fn get_epoch_now() -> u32 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_secs() as u32
}
async fn process_message(receiver: Receiver<TcpStream>, task_sender: Sender<bool>, dir_receiver: Receiver<Vec<utilities::FileData>>,idx: i32) {
        let mut dirs: fileDataTtl = fileDataTtl {
            timestamp: get_epoch_now(),
            files: utilities::look_for_dirs_and_subdirs(),
            ttl: 10,
        };


        let mut hashMap: HashMap<String, Vec<u8>> = HashMap::new();

        println!("Thread {} started", idx);

        while let Ok(stream) = receiver.recv() {
                handle_client(&stream, &mut dirs, &mut hashMap).await;
        }
    }

