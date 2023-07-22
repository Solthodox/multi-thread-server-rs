mod threadpool;
use threadpool::ThreadPool;

use std::{
    thread,
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, time::Duration,
};

const LOCALHOST_ADDRESS: &str = "127.0.0.1:7878";
fn main() {
    let stack_size_bytes = 4096 * 10;
    let thread_pool = ThreadPool::build(4, stack_size_bytes).unwrap();
    let server = TcpListener::bind(LOCALHOST_ADDRESS).unwrap();
    println!("Listening on {LOCALHOST_ADDRESS}...");

    for stream in server.incoming() {
        let stream = stream.unwrap();
        thread_pool
            .execute(|| {
                handle_connection(stream);
            })
            .unwrap();
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    if request_line == "GET / HTTP/1.1" {
        stream
            .write(parse_response("HTTP/1.1 200 OK", "./client/index.html").as_bytes())
            .unwrap();
    }
    else if request_line == "GET /sleep HTTP/1.1" {
        thread::sleep(Duration::from_secs(5));
        stream
            .write(parse_response("HTTP/1.1 200 OK", "./client/index.html").as_bytes())
            .unwrap();
    } 
    else {
        stream
            .write(parse_response("HTTP/1.1 404 NOT FOUND", "./client/404.html").as_bytes())
            .unwrap();
    }
}

fn parse_response(status_line: &str, contents_path: &str) -> String {
    let contents = fs::read_to_string(contents_path).unwrap();
    let length = contents.len();
    let response = format!(
        "{status_line}\r\n\
            Content-Length: {length}\r\n\r\n\
            {contents}"
    );
    response
}
