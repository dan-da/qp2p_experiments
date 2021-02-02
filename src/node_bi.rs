//! This is the echo_service.rs example from qp2p lib
//! modified so that it iterates a counter and sends that
//! back and forth between two nodes infinitely.

use anyhow::{anyhow, bail, Error, Result};
use bytes::Bytes;
use qp2p::{Config, Message, QuicP2p};
use std::env;
use std::net::{IpAddr, Ipv4Addr};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let (bootstrap_nodes, genesis) = match &args[1][..] {
        "create" => (vec![], true),
        "connect" => {
            let bootstrap_node = args[2]
                .parse()
                .map_err(|err| anyhow!("SocketAddr format not recognized: {}", err))?;
            (vec![bootstrap_node], false)
        }
        other => {
            bail!("Unexpected argument: {}", other);
        }
    };

    let qp2p = QuicP2p::with_config(
        Some(Config {
            ip: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            port: Some(0),
            ..Default::default()
        }),
        &bootstrap_nodes,
        false,
    )?;

    let endpoint = qp2p.new_endpoint()?;
    let socket_addr = endpoint.socket_addr().await?;
    println!("Process running at: {}", &socket_addr);

    if genesis {
        println!("Waiting for connections");
        let mut incoming = endpoint.listen();
        let mut messages = incoming
            .next()
            .await
            .ok_or_else(|| anyhow!("Missing expected incomming connection"))?;
        let connecting_peer = messages.remote_addr();
        println!("Incoming connection from: {}", &connecting_peer);

        let message = messages
            .next()
            .await
            .ok_or_else(|| anyhow!("Missing expected incomming message"))?;
        println!("Responded to peer with EchoService response");

        println!("Waiting for messages...");
        let (mut bytes, mut send, mut recv) = if let Message::BiStream {
            bytes, send, recv, ..
        } = message
        {
            (bytes, send, recv)
        } else {
            println!("Only bidirectional streams are supported in this example");
            bail!("Only bidirectional streams are supported in this example");
        };

        let mut cnt = 1;
        loop {
            let msg = std::str::from_utf8(&bytes[..])
                .map_err(|err| anyhow!("Bytes received cannot read as UTF8 string: {}", err))?;

            let intval: usize = msg.parse().unwrap();
            if intval != cnt {
                panic!("Expected: {}, got: {}", cnt, intval);
            }
            println!("Got message: {}", msg);

            let input = format!("{}", cnt);
            cnt += 1;
            send.send_user_msg(Bytes::from(input)).await?;
            bytes = recv.next().await?;
        }
    } else {
        let node_addr = bootstrap_nodes[0];
        let (connection, _) = endpoint.connect_to(&node_addr).await?;
        let (mut send, mut recv) = connection.open_bi().await?;

        let mut cnt = 1;

        loop {
            let input = format!("{}", cnt);
            send.send_user_msg(Bytes::from(input)).await?;

            let bytes = recv.next().await?;
            let msg = std::str::from_utf8(&bytes[..])
                .map_err(|err| anyhow!("Bytes received cannot read as UTF8 string: {}", err))?;

            let intval: usize = msg.parse().unwrap();
            if intval != cnt {
                panic!("Expected: {}, got: {}", cnt, intval);
            }

            cnt += 1;
            println!("Got message: {}", msg);
        }
    }
}
