use crate::lazy_data::{
    remote::test_remote::TestRemote,
    serialization::{CompressedSerializer, MessagePackSerializer, Serializer, ZstdCompressor},
};

type Data = String;
type S = CompressedSerializer<MessagePackSerializer, ZstdCompressor, Data>;

fn serializer() -> S {
    <_>::default()
}

#[tokio::test]
async fn empty_should_not_return_value() {
    let index = Store::new();
    let val = index.get("not_a_real_key", &serializer()).await;
    if let Ok(None) = val {
        return;
    }
    panic!("Expected empty value, got: {val:?}");
}

async fn remote() -> (String, TestRemote<&'static str>) {
    let data = "hello world".to_string();

    let serialized_data = serializer()
        .serialize(&data)
        .await
        .expect("Failed to serialize");

    let remote = TestRemote::new([("key", serialized_data)]);

    (data, remote)
}

#[tokio::test]
async fn basic_retrieval() {
    let (data, remote) = remote().await;

    let index = Store::new();
    let val = index.get_or_fetch("key", &serializer(), &remote).await;
    if let Ok(val) = val {
        assert_eq!(*val, data);
        return;
    }
    panic!("Expected value, got: {val:?}");
}

#[tokio::test]
async fn persistance() {
    let (_, remote) = remote().await;

    let index = Store::new();
    _ = index
        .get_or_fetch("key", &serializer(), &remote)
        .await
        .expect("Basic Retrieval failed");
    _ = index
        .get_or_fetch("key", &serializer(), &remote)
        .await
        .expect("Failed retrieving value on the second request");

    assert_eq!(remote.get_request_count().await, 1);
}

// #[tokio::test]
// async fn last_use() {
//     tokio::time::
//     let time = Instant::now();
//     let (data, remote) = remote().await;

//     let index = Store::new();
//     Instant::delay_for(1);
//     let val = index.get_or_fetch("key", &serializer(), &remote).await.expect("basic retrieval error");

//     panic!("Expected value, got: {:?}", val);
// }
