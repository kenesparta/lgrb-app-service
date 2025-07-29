const AUTH_SERVICE_PROTO: &str = "proto/auth_service.proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure().compile_protos(&[AUTH_SERVICE_PROTO], &["proto"])?;
    Ok(())
}
