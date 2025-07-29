use std::env;
use tonic::transport::Channel;
use tonic::Request;

pub mod auth_service {
    tonic::include_proto!("auth_service");
}

use auth_service::{auth_service_client::AuthServiceClient, VerifyTokenRequest};

pub struct GrpcAuthClient {
    client: AuthServiceClient<Channel>,
}

impl GrpcAuthClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let grpc_host = env::var("GRPC_AUTH_SERVICE_HOST").unwrap_or("".to_owned());
        let channel = Channel::from_shared(grpc_host)?
            .connect()
            .await?;

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
