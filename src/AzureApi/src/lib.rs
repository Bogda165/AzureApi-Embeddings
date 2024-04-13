use std::io::Read;
use reqwest::Response;
use serde::Deserialize;
use serde_json::Value;
use std::time::Instant;
pub struct MyRequest {
    client: reqwest::Client,
    headers: reqwest::header::HeaderMap,
    img: Vec<u8>,
    request_adress: String,
}

impl MyRequest {
    pub fn new(key: &str, args: Vec<&str>) -> Self {
        let client = reqwest::Client::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Ocp-Apim-Subscription-Key", reqwest::header::HeaderValue::from_str(key).unwrap());
        headers.insert("Content-Type", reqwest::header::HeaderValue::from_str("application/octet-stream").unwrap());

        MyRequest {
            client,
            headers,
            img: Vec::new(),
            request_adress: "https://file-search-rust-paid.cognitiveservices.azure.com/computervision/imageanalysis:analyze?api-version=2024-02-01&features=".to_string() + args.join(",").as_str(),
        }
    }

    pub fn set_img(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        match std::fs::File::open(path)?.read_to_end(&mut self.img) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn send_request(&self) -> Result<Response, Box<dyn std::error::Error>> {
        println!("Start sending request");
        let start = Instant::now();
        let response = self.client.post(&self.request_adress)
            .headers(self.headers.clone())
            .body(self.img.clone())
            .send()
            .await;

        println!("Request recieved\nTime: {:?}", start.elapsed());
        match response {
            Ok(response) => Ok(response),
            Err(e) => Err(Box::new(e)),
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct MyLabel {
    pub name: String,
    score: f64,
}
impl MyLabel {
    pub fn new(name: String, score: f64) -> Self {
        MyLabel {
            name,
            score,
        }
    }
}

impl From<&Value> for MyLabel {
    fn from(value: &Value) -> Self {
        let name = value["name"].as_str().unwrap().to_string();
        let score = value["confidence"].as_f64().unwrap();
        MyLabel {
            name,
            score,
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct MyResponse {
    pub caption: String,
    pub labels: Vec<MyLabel>,
}

impl From<Value> for MyResponse {
    fn from(value: Value) -> Self {
        let caption = value["captionResult"]["text"].as_str().unwrap().to_string();
        let mut labels = Vec::new();

        for label in value["tagsResult"].get("values").unwrap().as_array().unwrap() {
            labels.push(MyLabel::new(label.get("name").unwrap().as_str().unwrap().to_string(), label.get("confidence").unwrap().as_f64().unwrap()));
        }

        MyResponse {
            caption,
            labels,
        }
    }
}