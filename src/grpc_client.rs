use std::env;
use tonic::Request;
use tonic::transport::{Channel, ClientTlsConfig};

pub mod auth_service {
    tonic::include_proto!("auth_service");
}

use auth_service::{VerifyTokenRequest, auth_service_client::AuthServiceClient};

pub struct GrpcAuthClient {
    client: AuthServiceClient<Channel>,
}

impl GrpcAuthClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let grpc_host = env::var("GRPC_AUTH_SERVICE_HOST").unwrap_or("".to_owned());
        println!("GRPC_AUTH_SERVICE_HOST: {}", grpc_host);
        let mut channel_builder = Channel::from_shared(grpc_host.clone())?;

        if grpc_host.starts_with("https://") {
            let domain = grpc_host
                .strip_prefix("https://")
                .unwrap_or(&grpc_host)
                .split(':')
                .next()
                .unwrap_or(&grpc_host);

            let tls_config = ClientTlsConfig::new()
                .with_native_roots()
                .domain_name(domain);
            channel_builder = channel_builder.tls_config(tls_config)?;
        }

        let channel = channel_builder.connect().await?;
        let client = AuthServiceClient::new(channel);

        Ok(Self { client })
    }

    pub async fn verify_token(
        &mut self,
        token: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let request = Request::new(VerifyTokenRequest { token });

        let response = self.client.verify_token(request).await?;
        let verify_response = response.into_inner();

        println!("Response message: {}", verify_response.message);

        Ok(verify_response.valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_grpc_client() {
        let mut client = GrpcAuthClient::new()
            .await
            .expect("Failed to create a client");

        let token = "sample_jwt_token_here".to_string();
        match client.verify_token(token).await {
            Ok(is_valid) => println!("Token is valid: {}", is_valid),
            Err(e) => println!("Error verifying token: {}", e),
        }
    }
}
