use std::{hash::Hash, net::SocketAddr};

use async_trait::async_trait;
// use color_eyre::Report;
use quinn::{Connection, Endpoint};

use super::Remote;

pub struct ConnectionInfo<'a> {
    pub remote_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub server_name: &'a str,
}

pub struct QuicRemote<'a> {
    pub connection_info: ConnectionInfo<'a>,
    pub connection: Connection,
}

#[async_trait]
impl<Key: Eq + Hash + Clone + Send + Sync> Remote<Key> for QuicRemote<'_> {
    type Error = std::io::Error;

    async fn get_async(&self, _key: &Key) -> Result<Vec<u8>, Self::Error> {
        // self.connection.
        // Err("TODO")
        unimplemented!()
    }
}

impl QuicRemote<'_> {
    // TODO
    pub async fn connect(
        connection_info: ConnectionInfo<'_>,
    ) -> Result<QuicRemote<'_>, Box<dyn std::error::Error>> {
        // Bind this endpoint to a UDP socket on the given client address.
        let endpoint = Endpoint::client(connection_info.local_addr).unwrap();

        // Connect to the server passing in the server name which is supposed to be in the server certificate.
        let connection = endpoint
            .connect(connection_info.remote_addr, connection_info.server_name)?
            .await?;

        // Start transferring, receiving data, see data transfer page.

        Ok(QuicRemote {
            connection_info,
            connection,
        })
    }
}
