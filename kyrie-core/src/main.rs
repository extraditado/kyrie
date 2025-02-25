use kyrie_bot::entrypoint;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = "";

    entrypoint(&token).await?;

    Ok(())
}
