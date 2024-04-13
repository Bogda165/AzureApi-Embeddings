use AzureApi::*;
use Embeddings::*;
use serde_json::Value;

#[tokio::main]
async fn main() {

    let mut request = MyRequest::new("4d7bd39a70c249eebd19f5b8d62f5d7b", vec!["tags", "caption"]);
    request.set_img("/Users/bogdankoval/Downloads/photos/IMG_6141.HEIC").unwrap();

    let response= request.send_request().await.unwrap();
    let response_copy = response.json::<Value>().await.unwrap();
    let response_struct: MyResponse = response_copy.clone().into();
    println!("{:?}", response_struct.caption);

    let mut embed = Embedding::new();

    embed.get_embeddings("/Users/bogdankoval/Downloads/glove.6B/glove.6B.100d.txt");
    let first_vec = embed.semantic_vector(response_struct.labels.iter().map(|x| x.name.as_str()).collect());

    println!("{}", Embedding::cosine_similarity(&first_vec, &embed.average_vector("yellow car is on the road")));
    println!("{}", Embedding::cosine_similarity(&embed.average_vector(&response_struct.caption), &embed.average_vector("blue car is on the road")));

}
