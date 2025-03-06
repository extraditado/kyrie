use kyrie_bot::entrypoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = "";

    entrypoint(token).await?;

    Ok(())
}
