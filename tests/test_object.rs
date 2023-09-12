mod common;

use std::collections::HashMap;
use std::str::FromStr;

use common::{create_bucket_if_not_exist, get_test_minio};
use futures_util::{stream, StreamExt};
use minio_rsc::errors::Result;
use minio_rsc::types::args::CompressionType;
use minio_rsc::types::args::CopySource;
use minio_rsc::types::args::CsvInput;
use minio_rsc::types::args::InputSerialization;
use minio_rsc::types::args::JsonOutput;
use minio_rsc::types::args::ObjectArgs;
use minio_rsc::types::args::SelectRequest;
use minio_rsc::types::ObjectLockConfiguration;
use minio_rsc::types::Tags;
use tokio;

#[tokio::main]
#[test]
async fn test_base_operate() -> Result<()> {
    let minio = get_test_minio();

    let bucket_name = "test-object-base";
    let object_name = "/test/test.txt";
    create_bucket_if_not_exist(&minio, bucket_name).await?;

    let args: ObjectArgs = ObjectArgs::new(bucket_name, object_name);
    let txt = "hello minio";
    minio.put_object(args.clone().content_type(Some("text/plain".to_string())), txt.into()).await?;
    assert_eq!(minio.get_object(args.clone()).await?.text().await?, txt);
    assert_eq!(minio.stat_object(args.clone()).await?.unwrap().object_name(),object_name);
    assert_eq!(minio.stat_object(args.clone()).await?.unwrap().content_type(),"text/plain");

    let mut tags: Tags = minio.get_object_tags(args.clone()).await?;
    tags.insert("key1", "value1");
    minio.set_object_tags(args.clone(), tags).await?;
    let tags = minio.get_object_tags(args.clone()).await?;
    assert_eq!(tags.get("key1").unwrap(), "value1");
    minio.delete_object_tags(args.clone()).await?;
    let tags = minio.get_object_tags(args.clone()).await?;
    assert!(tags.is_empty());

    let copy = CopySource::from(args.clone()).metadata_replace(true);
    let args2: ObjectArgs = args.clone().content_type(Some("image/jpeg".to_string()));
    minio.copy_object(args2.clone(), copy).await?;
    assert_eq!(minio.stat_object(args.clone()).await?.unwrap().content_type(),"image/jpeg");

    minio.remove_object(args.clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}

#[tokio::main]
#[test]
#[cfg(feature = "fs-tokio")]
async fn test_file_operate() -> Result<()> {
    let minio = get_test_minio();

    let bucket_name = "test-file-operate";
    let object_name = "/test/test.txt";
    let loacl_file = "tests/test.txt";
    create_bucket_if_not_exist(&minio, bucket_name).await?;

    let args: ObjectArgs = ObjectArgs::new(bucket_name, object_name);
    minio.stat_object(args.clone()).await?;
    minio.put_object(args.clone(), "hello minio".into()).await?;

    minio.fget_object(args.clone(), loacl_file).await?;
    minio.fput_object(args.clone(), loacl_file).await?;

    minio
        .fput_object((bucket_name, "lena_std.jpeg"), "tests/lena_std.jpeg")
        .await?;
    minio.remove_object((bucket_name, "lena_std.jpeg")).await?;

    minio.stat_object(args.clone()).await?;
    minio.remove_object(args.clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}

#[tokio::main]
#[test]
async fn test_put_stream() -> Result<()> {
    let minio = get_test_minio();

    let bucket_name = "test-put-stream";
    let object_name = "test.txt";
    let len = 22 * 1024 * 1024; // 22MB
    let size = 128 * 1024;
    let num = len / size;
    let mut bytes = bytes::BytesMut::with_capacity(size);
    for _ in 0..size {
        bytes.extend_from_slice("A".as_bytes());
    }
    create_bucket_if_not_exist(&minio, bucket_name).await?;
    let stm = stream::repeat(bytes.freeze()).take(num).map(|f| Ok(f));
    let mut args: ObjectArgs = ObjectArgs::new(bucket_name, object_name);
    args = args.metadata(HashMap::from([("filename".to_string(),"name.mp4".to_string())]));
    minio.put_object_stream(args.clone(), Box::pin(stm), Some(len)).await?;
    let state = minio.stat_object(args.clone()).await?.unwrap();
    assert_eq!(state.size(), len);
    assert_eq!(state.metadata().get("filename").unwrap(), "name.mp4");

    let mut bytes = bytes::BytesMut::with_capacity(size);
    for _ in 0..size {
        bytes.extend_from_slice("A".as_bytes());
    }

    let stm = stream::repeat(bytes.freeze()).take(num).map(|f|Ok(f));
    minio.put_object_stream(args.clone(), Box::pin(stm), None).await?;

    let state = minio.stat_object(args.clone()).await?.unwrap();
    assert_eq!(state.size(), len);
    assert_eq!(state.metadata().get("filename").unwrap(), "name.mp4");

    minio.remove_object(args.clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}

#[tokio::main]
#[test]
async fn test_select_object() -> Result<()> {
    let minio = get_test_minio();

    let bucket_name = "test-select-object";
    let object_name = "test.scv";

    create_bucket_if_not_exist(&minio, bucket_name).await?;

    let mut fake_csv = String::from_str("id,A,B,C,D,E\n").unwrap();
    for i in 0..10000 {
        fake_csv += &format!("{i},A{i},B{i},C{i},D{i},E{i}\r\n");
    }
    minio
        .put_object((bucket_name, object_name), fake_csv.into())
        .await?;
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
    let reader = minio
        .select_object_content((bucket_name, object_name), req)
        .await?;
    let _ = reader.read_all().await?;
    minio.remove_object((bucket_name, object_name).clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}
