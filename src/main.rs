#[tokio::main(flavor = "current_thread")]
async fn main() {
    let client = kmoni::KMoniClient::new();
    let res = client.await.fetch().await;
    println!("{:?}", res);
}
