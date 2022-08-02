#[cfg(test)]
mod tests {
    use std::{any::Any, fmt::Debug};

    use crate::{
        lazy_data::{
            index::Store,
            remote::{
                test_remote::{self, TestRemote},
                Remote,
            },
            serialization::{
                CompressedSerializer, MessagePackSerializer, Serializer, ZstdCompressor,
            },
        },
        sync::Arc,
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
        panic!("Expected empty value, got: {:?}", val);
    }

    #[tokio::test]
    async fn basic_retrieval() {
        let data = "hello world".to_string();

        let serialized_data = serializer().serialize(&data).await.expect("Failed to serialize");

        let remote = TestRemote::new([("key", serialized_data)]);

        let index = Store::new();
        let val = index.get_or_fetch("key", &serializer(), &remote).await;
        if let Ok(val) = val {
            assert_eq!(*val, data);
            return;
        }
        panic!("Expected value, got: {:?}", val);
    }

    // #[tokio::test]
    // async fn basic_retrieval() {
    //     let remote = test_remote::simple_remote();
    //     let index = CAS::new();
    //     index.get_or_fetch(&remote)
    // }
}
