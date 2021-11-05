use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::env;
use std::thread;
use threadpool::ThreadPool;
use std::str::FromStr;

use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut ip = "127.0.0.1".to_string();
    let mut port = "7878".to_string();
    let mut threads = 5;

    for idx in 0..args.len() {
        if args[idx] == "i".to_string() ||  args[idx] == "ip".to_string() {
            ip = args[idx+1].clone();
        }
        else if args[idx] == "p".to_string() || args[idx] == "port".to_string() {
            port = args[idx+1].clone();
        }
        else if args[idx] == "t".to_string() || args[idx] == "threads".to_string() {
            let arg: i32 = FromStr::from_str(args[idx+1].as_str()).unwrap();
            if arg > 0 || arg <= 15 {
                threads = arg;
            }            
        }
    }

    let addr = format!("{}:{}", ip, port);

    println!("Launching server on: {}", addr);
    let listener = TcpListener::bind(addr).unwrap();

    let t_pool = ThreadPool::new(threads as usize);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        t_pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    // let response = "HTTP/1.1 200 OK\r\n\r\n";

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let stop = b"GET /stop HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(stop) {
        ("HTTP/1.1 200 OK", "shutdown.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    if buffer.starts_with(stop) {
        std::process::exit(0);
    }
}