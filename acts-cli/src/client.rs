use acts_grpc::acts_service_client::*;
use tonic::transport::{Channel, Endpoint};

pub async fn connect(
    addr: Endpoint,
) -> Result<ActsServiceClient<Channel>, Box<dyn std::error::Error>> {
    let client = ActsServiceClient::connect(addr).await?;
    Ok(client)
}
