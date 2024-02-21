use acts_channel::ActsChannel;

pub async fn connect(url: &str) -> Result<ActsChannel, Box<dyn std::error::Error>> {
    let client = ActsChannel::new(url).await?;
    Ok(client)
}
