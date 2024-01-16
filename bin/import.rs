use rustnews::import;

#[tokio::main]
async fn main() {
    import().await.unwrap()
}
