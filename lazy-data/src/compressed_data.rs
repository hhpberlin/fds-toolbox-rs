// enum Data<T, Serializer: Serializer> {
//     Compressed(Vec<u8>),
//     Uncompressed(T),
// }

// impl<T, Serializer: Serializer> Data<T, Serializer> {
//     pub fn new(data: T) -> Self {
//         Self::Uncompressed(data)
//     }

//     pub fn compressed(self) -> Self {
//         match self {
//             Self::Compressed(_) => self,
//             Self::Uncompressed(data) => Self::Compressed(Serializer::serialize(data)),
//         }
//     }
// }
