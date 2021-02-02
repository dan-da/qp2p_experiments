//! This example fails because a fixed port is specified in
//! the QuicP2p config, and when we try to call ::new_endpoint()
//! a second time, that port is already in use.
//!
//! A workaround is to use a second QuicP2p instance for outgoing
//! connections.  That is what we do in node_uni.rs test case.
//!
//! An issue has been created at:
//! https://github.com/maidsafe/qp2p/issues/205

use anyhow::{anyhow, Error, Result};
use bytes::Bytes;
use qp2p::{Config, QuicP2p};
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
//use std::time::Duration;
//use std::thread;
use env_logger;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let genesis = if args.len() > 1 && &args[1][..] == "create" {
        true
    } else {
        false
    };

    let myport = if genesis { 10000 } else { 10001 };
    let peer: SocketAddr = if genesis {
        "127.0.0.1:10001".parse()?
    } else {
        "127.0.0.1:10000".parse()?
    };

    // We use a single port for receiving and sending.
    let qp2p = QuicP2p::with_config(
        Some(Config {
            ip: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            port: Some(myport),
            ..Default::default()
        }),
        &vec![],
        false,
    )?;

    if genesis {
        listen_for_network_msgs(&qp2p, &peer, 0).await
    } else {
        send(&qp2p, &peer, 1).await?;
        listen_for_network_msgs(&qp2p, &peer, 1).await
    }
}

async fn listen_for_network_msgs(
    qp2p: &QuicP2p,
    peer: &SocketAddr,
    mut cnt: usize,
) -> Result<(), Error> {
    let endpoint_recv = qp2p.new_endpoint()?;
    let socket_addr = endpoint_recv.socket_addr().await?;

    let mut conns = endpoint_recv.listen();
    println!("Listening for messages on {}", socket_addr);

    while let Some(mut msgs) = conns.next().await {
        println!("Received a connection from {}", msgs.remote_addr());
        while let Some(msg) = msgs.next().await {
            let bytes = msg.get_message_data();
            let msg_str = std::str::from_utf8(&bytes[..])
                .map_err(|err| anyhow!("Bytes received cannot read as UTF8 string: {}", err))?;

            println!("Got message: {}", msg_str);

            let intval: usize = msg_str.parse()?;
            if intval != cnt + 1 {
                panic!("Expected: {}, got: {}", cnt + 1, intval);
            }
            cnt = intval + 1;

            //          thread::sleep(Duration::from_millis(500));

            send(qp2p, peer, cnt).await?;
        }
        println!("done with msgs.  cnt = {}", cnt);
    }

    info!("Finished listening for connections");
    Ok(())
}

async fn send(qp2p: &QuicP2p, peer: &SocketAddr, cnt: usize) -> Result<(), Error> {
    let input = format!("{}", cnt);
    let endpoint = qp2p.new_endpoint()?;
    let (conn, _) = endpoint.connect_to(peer).await?;
    conn.send_uni(Bytes::from(input.clone())).await?;
    conn.close();

    println!("Sent message: {} to: {}", input, peer);
    Ok(())
}
