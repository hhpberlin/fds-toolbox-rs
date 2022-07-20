// use std::future::Future;

// use futures::FutureExt;
// use tokio::sync::{Notify, RwLock};

// pub struct CASNode<Value, Error>(RwLock<Value>);

// impl<Value, Error> CASNode<Value, Error> {
//     pub fn new(fut: impl Future<Output = Result<Value, Error>>) -> Self {
//         fut.shared()

//         Self(RwLock::new(CASNodeInner::new()))
//     }
    
// }

// enum ValueOrFuture<Value, Error> {
//     Value(Value),
//     Faulted(Error),
//     Future(Arc<Notify>),
// }

// impl<Value, Error> ValueOrFuture<Value, Error> {
//     pub async fn get_val(self: &RwLock<Self>) -> &T {
//         match self.read().await {
//             ValueOrFuture::Value(ref val) => val,
//             ValueOrFuture::Future(notify) => {
//                 while let ValueOrFuture::Future(notify) = self {
//                     notify.notified().await;
//                 }
//             },
//         }
//     }

//     pub fn from_future(future: impl Future<Output = Result<Value, Error>>) -> Self {
//         let notify = Notify::new();
//         let notify = Arc::new(notify);
//         tokio::spawn(async {
//             let result = match future.await {
//                 Ok(val) => ValueOrFuture::Value(val),
//                 Err(err) => ValueOrFuture::Faulted(err),
//             };

//         });
//         ValueOrFuture::Future(notify)
//     }
// }