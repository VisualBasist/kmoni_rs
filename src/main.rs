fn main() {
    let client = kmoni::KMoniClient::new();
    let res = client.fetch();
    println!("{:?}", res);
}
