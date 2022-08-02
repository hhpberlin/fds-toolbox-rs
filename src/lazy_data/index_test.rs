#[cfg(test)]
mod tests {
    use std::{fmt::Debug, any::Any};

    use crate::{sync::Arc, lazy_data::{
        index::Store,
        remote::{
            test_remote::{self, TestRemote},
            Remote,
        },
        serialization::{CompressedSerializer, MessagePackSerializer, ZstdCompressor, Serializer, Data},
    }};

    type S = CompressedSerializer<MessagePackSerializer, ZstdCompressor>;

    fn serializer() -> S { <_>::default() }

    #[tokio::test]
    async fn empty_should_not_return_value() {
        let index = Store::new();
        let val = index.get("not_a_real_key", &serializer()).await;
        if let Ok(None) = val {
            return;
        }
        panic!("Expected empty value, got: {:?}", val);
    }

    #[typetag::serde(name = "thing")]
    impl Data for [i32; 6] {
        fn as_any(&self) -> &dyn Any { self }
    }

    #[tokio::test]
    async fn basic_retrieval() {
        static DATA: [i32; 6] = [1, 2, 3, 3, 2, 1];
        let serialized_data =
            serializer().serialize(&(Box::new(DATA) as Box<dyn Data + 'static>)).expect("Failed to serialize");

        let remote = TestRemote::new([
            ("key", serialized_data),
        ]);

        let index = Store::new();
        let val = index.get_or_fetch("key", &serializer(), &remote).await;
        if let Ok(val) = val {
            let val: &[i32; 6] = val.as_any().downcast_ref::<[i32; 6]>().unwrap();
            assert_eq!(*val, DATA);
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
