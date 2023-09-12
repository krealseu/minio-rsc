mod common;

use std::collections::HashMap;
use std::str::FromStr;

use common::{create_bucket_if_not_exist, get_test_minio};
use futures_util::{stream, StreamExt};
use minio_rsc::client::CopySource;
use minio_rsc::client::KeyArgs;
use minio_rsc::error::Result;
use minio_rsc::types::args::CompressionType;
use minio_rsc::types::args::CsvInput;
use minio_rsc::types::args::InputSerialization;
use minio_rsc::types::args::JsonOutput;
use minio_rsc::types::args::SelectRequest;
use minio_rsc::types::ObjectLockConfiguration;
use minio_rsc::types::Tags;
use tokio;

#[tokio::main]
#[test]
async fn test_base_operate() -> Result<()> {
    let minio = get_test_minio();

    let bucket_name = "test-object-base";
    let object = "/test/test.txt";
    let bucket = minio.bucket("test-object-base");
    create_bucket_if_not_exist(&minio, bucket_name).await?;

    let txt = "hello minio";
    let key = KeyArgs::new(object).content_type(Some("text/plain".to_string()));
    bucket.put_object(key, txt.into()).await?;

    assert_eq!(bucket.get_object(object).await?.text().await?, txt);

    let stat = bucket.stat_object(object).await?.unwrap();
    assert_eq!(stat.object_name(), object);
    assert_eq!(stat.content_type(), "text/plain");

    let mut tags: Tags = bucket.get_object_tags(object).await?;
    tags.insert("key1", "value1");
    bucket.set_object_tags(object, tags).await?;
    let tags = bucket.get_object_tags(object).await?;
    assert_eq!(tags.get("key1").unwrap(), "value1");
    bucket.delete_object_tags(object).await?;
    let tags = bucket.get_object_tags(object).await?;
    assert!(tags.is_empty());

    let copy = CopySource::new(bucket_name, object).metadata_replace(true);
    let args2: KeyArgs = <KeyArgs>::from(object).content_type(Some("image/jpeg".to_string()));
    bucket.copy_object(args2, copy).await?;

    let stat = bucket.stat_object(object).await?.unwrap();
    assert_eq!(stat.content_type(), "image/jpeg");

    bucket.remove_object(object).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}

// #[tokio::main]
// #[test]
// async fn test_retention() -> Result<()> {
//     let minio = get_test_minio();
//     let bucket = "test-object-retention";
//     let object = "test.txt";
//     let exists = minio.bucket_exists(bucket).await?;
//     if !exists {
//         minio.make_bucket(bucket, true).await?;
//     }
//     let config = ObjectLockConfiguration::new(1, true, false);
//     minio.set_object_lock_config(bucket, config).await?;
//     // minio.put_object((bucket,object), "hello".into()).await?;
//     // println!("ss");
//     let retention = minio.get_object_retention((bucket, object)).await?;
//     // let ss = minio.disable_object_legal_hold_enabled((bucket,object)).await?;
//     println!("sss {}", retention.to_xml());
//     let retention = minio
//         .set_object_retention((bucket, object), retention)
//         .await?;
//     println!("{retention:?}");

//     minio.remove_object((bucket, object).clone()).await?;
//     minio.remove_bucket(bucket).await?;
//     Ok(())
// }

#[tokio::main]
#[test]
#[cfg(feature = "fs-tokio")]
async fn test_file_operate() -> Result<()> {
    let minio = get_test_minio();

    let bucket_name = "test-file-operate";
    let object_name = "/test/test.txt";
    let loacl_file = "tests/test.txt";
    create_bucket_if_not_exist(&minio, bucket_name).await?;
    let bucket = minio.bucket(bucket_name);

    let key: KeyArgs = KeyArgs::new(object_name);
    bucket.stat_object(key.clone()).await?;
    bucket.put_object(key.clone(), "hello minio".into()).await?;

    bucket.fget_object(key.clone(), loacl_file).await?;
    bucket.fput_object(key.clone(), loacl_file).await?;

    bucket
        .fput_object("lena_std.jpeg", "tests/lena_std.jpeg")
        .await?;
    bucket.remove_object("lena_std.jpeg").await?;

    bucket.stat_object(key.clone()).await?;
    bucket.remove_object(key.clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}

#[tokio::main]
#[test]
async fn test_put_stream() -> Result<()> {
    let minio = get_test_minio();

    let bucket = "test-put-stream";
    let object_name = "test.txt";
    let len = 22 * 1024 * 1024; // 22MB
    let size = 128 * 1024;
    let num = len / size;
    let mut bytes = bytes::BytesMut::with_capacity(size);
    for _ in 0..size {
        bytes.extend_from_slice("A".as_bytes());
    }
    create_bucket_if_not_exist(&minio, bucket).await?;
    let stm = stream::repeat(bytes.freeze()).take(num).map(|f| Ok(f));
    let mut key: KeyArgs = KeyArgs::new(object_name);
    key = key.metadata(HashMap::from([(
        "filename".to_string(),
        "name.mp4".to_string(),
    )]));
    minio
        .put_object_stream(bucket, key.clone(), Box::pin(stm), Some(len))
        .await?;
    let state = minio.stat_object(bucket, key.clone()).await?.unwrap();
    assert_eq!(state.size(), len);
    assert_eq!(state.metadata().get("filename").unwrap(), "name.mp4");

    let mut bytes = bytes::BytesMut::with_capacity(size);
    for _ in 0..size {
        bytes.extend_from_slice("A".as_bytes());
    }

    let stm = stream::repeat(bytes.freeze()).take(num).map(|f| Ok(f));
    minio
        .put_object_stream(bucket, key.clone(), Box::pin(stm), None)
        .await?;

    let state = minio.stat_object(bucket, key.clone()).await?.unwrap();
    assert_eq!(state.size(), len);
    assert_eq!(state.metadata().get("filename").unwrap(), "name.mp4");

    minio.remove_object(bucket, key.clone()).await?;
    minio.remove_bucket(bucket).await?;
    Ok(())
}

#[tokio::main]
#[test]
async fn test_select_object() -> Result<()> {
    let minio = get_test_minio();

    let bucket = "test-select-object";
    let key = "test.scv";

    create_bucket_if_not_exist(&minio, bucket).await?;

    let mut fake_csv = String::from_str("id,A,B,C,D,E\n").unwrap();
    for i in 0..10000 {
        fake_csv += &format!("{i},A{i},B{i},C{i},D{i},E{i}\r\n");
    }
    minio.put_object(bucket, key, fake_csv.into()).await?;
    let input_serialization = InputSerialization::new(CsvInput::default(), CompressionType::NONE);
    let output_serialization = JsonOutput::default().into();
    let req = SelectRequest::new(
        r#"Select * from s3object where s3object._1>100"#.to_owned(),
        input_serialization,
        output_serialization,
        true,
        None,
        None,
    );
    let reader = minio.select_object_content(bucket, key, req).await?;
    let _ = reader.read_all().await?;
    minio.remove_object(bucket, key.clone()).await?;
    minio.remove_bucket(bucket).await?;
    Ok(())
}
