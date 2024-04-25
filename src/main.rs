use std::error::Error;
use std::sync::Arc;
use Embeddings::*;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;
use serde_json::{from_str, json};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

struct Address {
    ip: String,
    port: u16
}

impl Address {
    fn new(port: u16, ip: String) -> Self {
        Address {
            ip,
            port
        }
    }

    fn listener(&self) -> Result<TcpListener, Box<dyn Error>> {
        match TcpListener::bind(format!("{}:{}", self.ip, self.port)) {
            Ok(listener) => Ok(listener),
            Err(e) => {
                Err(Box::new(e))
            }
        }
    }
}


pub async fn handle_client(mut stream: TcpStream, embeddings: Arc<Embedding>) {
    let mut buffer = [0; 512];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            let mut string = String::from_utf8(buffer[..bytes_read].to_vec()).unwrap();
            println!("{}", string);
            if string == "hello" {
                println!("Wait 10 sec");
                sleep(Duration::from_secs(10));
            }
            let satart = std::time::Instant::now();
            let vector = embeddings.average_vector(string.as_str());
            println!("{}", satart.elapsed().as_secs_f32());

            let result = json!(vector).to_string();
            let vec: Vec<f32> = from_str(&*result).unwrap();
            println!("{:?}", vec);

            stream.write_all(result.as_bytes()).unwrap();
        },
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Add (String),
    GetByPath (String, u16),
}

#[tokio::main]
async fn main() {
    // load embeddings

    let mut _embeddings = Embedding::new();
    //_embeddings.get_embeddings("/Users/bogdankoval/Downloads/glove.6B/glove.6B.50d.txt");

    let embeddings = Arc::new(_embeddings);

    let _task_list: VecDeque<Command> = VecDeque::new();
    let task_list = Arc::new(Mutex::new(_task_list));

    // start server
    let address = Address::new(7878, "127.0.0.1".to_string());

    let listener = address.listener().unwrap();
    let mut buffer = [0; 512];

    tokio::spawn({
        let mut task_list = task_list.clone();

        async move{
            loop {
                let mut task_list_clone = task_list.lock().await;
                if !task_list_clone.is_empty() {
                    println!("Doing command {:?}", task_list_clone.pop_front().unwrap());
                } else {
                    std::mem::drop(task_list_clone);
                    println!("There are no tasks aviable at the moment");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok (mut stream) => {
                println!("Connection started");

                match stream.read(&mut buffer) {
                    Ok(buffer_size) => {

                        println!("{}", String::from_utf8(buffer[..buffer_size].to_vec()).unwrap());
                        let command: Command = serde_json::from_slice(&buffer[..buffer_size].to_vec()).unwrap();

                        let mut task_list_clone = task_list.clone();
                        let mut task_list_clone = task_list_clone.lock().await;
                        task_list_clone.push_back(command);

                    }
                    Err(_) => {
                        println!("Error while deserealisation!!!!")
                    }
                }

            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    todo!("Block threads, after finishing")
}
