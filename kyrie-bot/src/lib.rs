use serenity::all::GatewayIntents;

pub async fn entrypoint(token: &str) -> anyhow::Result<()> {
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::Client::builder(&token, intents)
        .await
        .map_err(|e| anyhow::anyhow!("[Err] creating client: {}", e))?;

    client
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("[Err] starting client: {}", e))?;

    Ok(())
}
