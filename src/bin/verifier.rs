use std::fmt::Debug;

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream, UdpSocket};
use std::str::FromStr;

use clap::Parser;
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;

use tracing::{error, info, Level};

use robots::serialize::deserializer::Deserializer;
use robots::serialize::{deserializer, DeserError};
use robots::{ClientMessage, DisplayMessage, InputMessage, ServerMessage, MAX_UDP_LENGTH};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    port: u16,

    #[clap(short, long)]
    udp: bool,

    #[clap(short, long)]
    message_type: MessageType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MessageType {
    Client,
    Server,
    Display,
    Input,
}

impl FromStr for MessageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "client" => Ok(MessageType::Client),
            "server" => Ok(MessageType::Server),
            "display" => Ok(MessageType::Display),
            "input" => Ok(MessageType::Input),
            _ => Err(format!("Unknown message type: {}", s)),
        }
    }
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

fn deserialize_message_from_buffer<T: DeserializeOwned + Debug>(
    message: &[u8],
    address: SocketAddr,
) {
    match deserializer::from_bytes::<T>(message) {
        Ok(message) => info!(addr = ?address, message = ?message),
        Err(err) => error!(addr = ?address, error = ?err),
    }
}

fn deserialize_message_from_stream<T: DeserializeOwned + Debug>(
    deserialize: &mut Deserializer<TcpStream>,
    address: SocketAddr,
) {
    loop {
        match deserialize.deserialize::<T>() {
            Ok(message) => info!(addr = ?address, message = ?message),
            Err(DeserError::Io(io)) if io.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(err) => {
                error!(addr = ?address, error = ?err);
                break;
            }
        }
    }
}

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::TRACE)
        .init();

    info!(args = ?ARGS.clone());

    let server_address = SocketAddr::from((Ipv6Addr::UNSPECIFIED, ARGS.port));

    if ARGS.udp {
        let udp_server = UdpSocket::bind(server_address).unwrap();
        let mut buffer = [0u8; MAX_UDP_LENGTH];
        loop {
            match udp_server.recv_from(&mut buffer) {
                Ok((size, addr)) => {
                    let bytes = &buffer[..size];
                    match ARGS.message_type {
                        MessageType::Client => {
                            deserialize_message_from_buffer::<ClientMessage>(bytes, addr)
                        }
                        MessageType::Server => {
                            deserialize_message_from_buffer::<ServerMessage>(bytes, addr)
                        }
                        MessageType::Display => {
                            deserialize_message_from_buffer::<DisplayMessage>(bytes, addr)
                        }
                        MessageType::Input => {
                            deserialize_message_from_buffer::<InputMessage>(bytes, addr)
                        }
                    };
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }
    } else {
        let tcp_server = TcpListener::bind(server_address).unwrap();
        loop {
            match tcp_server.accept() {
                Ok((stream, addr)) => {
                    info!(message = "Received TCP connection", addr = ?addr);
                    stream.set_nodelay(true).unwrap();
                    let mut deserializer = Deserializer::new(stream);
                    match ARGS.message_type {
                        MessageType::Client => deserialize_message_from_stream::<ClientMessage>(
                            &mut deserializer,
                            addr,
                        ),
                        MessageType::Server => deserialize_message_from_stream::<ServerMessage>(
                            &mut deserializer,
                            addr,
                        ),
                        MessageType::Display => deserialize_message_from_stream::<DisplayMessage>(
                            &mut deserializer,
                            addr,
                        ),
                        MessageType::Input => {
                            deserialize_message_from_stream::<InputMessage>(&mut deserializer, addr)
                        }
                    }
                }
                Err(e) => {
                    error!("Error when connecting: {}", e);
                }
            }
        }
    }
}
