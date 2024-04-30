use std::error::Error;
use std::sync::Arc;
use Embeddings::*;
use std::io::{ErrorKind, Read, Write};
use tokio::net::{TcpListener, TcpStream};
use serde_json::{from_str, json, Value};
use std::collections::VecDeque;
use std::convert::Infallible;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use AzureApi::{MyRequest, MyResponse};

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

    async fn listener(&self) -> Result<TcpListener, Box<dyn Error>> {
        match TcpListener::bind(format!("{}:{}", self.ip, self.port)).await {
            Ok(listener) => Ok(listener),
            Err(e) => {
                Err(Box::new(e))
            }
        }
    }
}


pub async fn handle_add_command() {

}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Command {
    Add (String),
    GetByPath (String, u16),
}

#[tokio::main]
async fn main() {
    // load embeddings

    let mut _embeddings = Embedding::new();
    _embeddings.get_embeddings("/Users/bogdankoval/Downloads/glove.6B/glove.6B.50d.txt");

    let _task_list: VecDeque<Command> = VecDeque::new();
    let task_list = Arc::new(Mutex::new(_task_list));

    // start server
    let address = Address::new(7878, "127.0.0.1".to_string());

    let listener = address.listener().await.unwrap();
    let mut buffer = [0; 512];

    tokio::spawn({
        let mut task_list = task_list.clone();
        async move{
            loop {
                let mut task_list_clone = task_list.lock().await;
                if !task_list_clone.is_empty() {
                    let command = task_list_clone.pop_front().unwrap();
                    std::mem::drop(task_list_clone);
                    match command {
                        Command::Add(path) => {
                            let mut request = MyRequest::new("4d7bd39a70c249eebd19f5b8d62f5d7b", vec!["tags", "caption"]);
                            request.set_img(&*path).unwrap();
                            let response = request.send_request().await.unwrap();
                            let response_copy = response.json::<Value>().await.unwrap();
                            let response_struct: Result<MyResponse, Infallible> = MyResponse::try_from(response_copy.clone());


                            println!("Adding, then sending");
                            // send to db service
                        },
                        Command::GetByPath(sentence, top) => {
                            todo!("get top top from se")
                        },
                        _ => {
                            println!("Not the right service");
                        }
                    }
                    println!("Doing command {:?}", command);
                } else {
                    println!("There are no tasks aviable at the moment");
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    });

    loop {
        println!("wating for data");
        let (stream, _) =  listener.accept().await.unwrap();
        stream.readable().await.unwrap();
        println!("Connection started");
        match stream.try_read(&mut buffer) {
            Ok(0) => {
                println!("There is no data to read!!!");
                break;
            },
            Ok(buffer_size) => {

                println!("{}", String::from_utf8(buffer[..buffer_size].to_vec()).unwrap());
                let command: Command = serde_json::from_slice(&buffer[..buffer_size].to_vec()).unwrap();

                let mut task_list_clone = task_list.clone();
                let mut task_list_clone = task_list_clone.lock().await;
                task_list_clone.push_back(command);
                println!("Added");
            },
            Err(_) => {
                println!("Error while reading!!!!");
                break;
            }
        }
    }
    println!("Crash");


    todo!("Block threads, after finishing")
}
