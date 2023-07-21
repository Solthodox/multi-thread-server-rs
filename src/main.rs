mod threadpool;
use threadpool::ThreadPool;

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

const LOCALHOST_ADDRESS: &str = "127.0.0.1:7878";
fn main() {
    let stack_size_bytse = 4096 * 10; // 10 KB
    let thread_pool = ThreadPool::build(1000, stack_size_bytse).unwrap();
    let server = TcpListener::bind(LOCALHOST_ADDRESS).unwrap();

    // el metodo incoming nos devuelve un iterator de intentos de conexion
    // si nos conectamos desde el navegador fallara por que el server no da respuesta
    // pero detectaremos una conexion
    // es posible que nos detecte varios intentos de conexion desde un mismo browser, eso es
    // por que puede que el browser haga mas requests para el favicon o que reintente despues de fallar
    for stream in server.incoming() {
        let stream = stream.unwrap();
        // para poder manejar las requests en un buen tiempode respuesta, usaremos muchos threads
        // pero tiene que haber un limite, no podemos permitir que haya threads ilimitados
        // ya que podria crashear
        // es por eso que crearemos un thread pool, que son una serie de threads listos para ser
        // utilizados
        thread_pool.execute(|| {
            handle_connection(stream);
        }).unwrap();
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream); // con esto parseamos el bufer a texto
    let request_line = buf_reader.lines().next().unwrap().unwrap(); // le hacemos 2 unwrap por que es un option con un result dentro
    if request_line == "GET / HTTP/1.1" {
        stream
            .write(parse_response("HTTP/1.1 200 OK", "./client/index.html").as_bytes())
            .unwrap();
    } else {
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
    // hay que pasarlo como bytes
    response
}
