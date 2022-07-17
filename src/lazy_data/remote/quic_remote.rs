use std::{error::Error, net::SocketAddr};

use async_trait::async_trait;
use quinn::{Connection, Endpoint, NewConnection};

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
impl Remote for QuicRemote<'_> {
    async fn get_async(&self, key: &[u8; 32]) -> Result<&[u8], Box<dyn Error>> {
        // self.connection.
        // Err("TODO")
        unimplemented!()
    }
}

impl QuicRemote<'_> {
    pub async fn connect(
        connection_info: ConnectionInfo<'_>,
    ) -> Result<QuicRemote<'_>, Box<dyn Error>> {
        // Bind this endpoint to a UDP socket on the given client address.
        let endpoint = Endpoint::client(connection_info.local_addr).unwrap();

        // Connect to the server passing in the server name which is supposed to be in the server certificate.
        let new_connection = endpoint
            .connect(connection_info.remote_addr, connection_info.server_name)?
            .await?;
        let NewConnection { connection, .. } = new_connection;

        // Start transferring, receiving data, see data transfer page.

        Ok(QuicRemote {
            connection_info,
            connection,
        })
    }
}
