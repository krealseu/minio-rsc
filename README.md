# minio-rsc
Rust Library for Minio

## Minio client
```rust
use minio_rsc::client::Minio;
use minio_rsc::provider::StaticProvider;
use tokio;

#[tokio::main]
async fn main() {
    let provider = StaticProvider::new("minio-access-key-test", "minio-secret-key-test", None);
    let minio = Minio::builder()
        .host("localhost:9022")
        .provider(provider)
        .secure(false)
        .build()
        .unwrap();
}
```