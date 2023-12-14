#![feature(lazy_cell)]
use anyhow::{anyhow, bail as yeet, Result};
use azalea::protocol::connect::Connection;
use azalea::protocol::packets::handshaking::{
    ClientboundHandshakePacket, ServerboundHandshakePacket,
};
use azalea::protocol::packets::status::clientbound_pong_response_packet::ClientboundPongResponsePacket;
use azalea::protocol::packets::status::clientbound_status_response_packet::{
    ClientboundStatusResponsePacket as Status, Players, Version,
};
use azalea::protocol::packets::status::{ClientboundStatusPacket, ServerboundStatusPacket};
use azalea::protocol::packets::ConnectionProtocol;
use azalea::FormattedText;
use azalea_chat::text_component::TextComponent;
use sqlx::PgPool;
use tokio::net::{TcpListener, TcpStream};

type ServerHandshakeConn = Connection<ServerboundHandshakePacket, ClientboundHandshakePacket>;
type ServerStatusConn = Connection<ServerboundStatusPacket, ClientboundStatusPacket>;

mod db;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("localhost:8080").await?;

    let pool = db::get_conn().await?;

    loop {
        let (socket, incoming_addr) = listener.accept().await?;
        println!("[*] Got a connection from {}", incoming_addr);
        match handle_conn(socket, &pool).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        };
    }

    Ok(())
}

pub async fn handle_conn(incoming: TcpStream, pool: &PgPool) -> Result<()> {
    let peer = incoming.peer_addr()?;
    let mut conn: ServerHandshakeConn = Connection::wrap(incoming);

    let Ok(ServerboundHandshakePacket::ClientIntention(handshake)) = conn.read().await else {
        return Ok(());
    };

    db::add_entry(pool, peer.ip().to_string()).await;
    let target = &handshake.hostname;
    println!("Saved {} {} to the db", peer, target);
    println!("[*] handshake: {:?}", handshake);

    match handshake.intention {
        ConnectionProtocol::Status => entice(Connection::from(conn)).await,
        _ => Err(anyhow!("[!] unexpected data")),
    }
}

async fn entice(mut conn: ServerStatusConn) -> Result<()> {
    let ServerboundStatusPacket::StatusRequest(_) = conn.read().await? else {
        yeet!("[!] expected status request")
    };

    let status = get_response(None);
    conn.write(status.get()).await?;

    let ServerboundStatusPacket::PingRequest(ping_request) = conn.read().await? else {
        yeet!("[!] expected ping request")
    };

    let ping_response = ClientboundPongResponsePacket {
        time: ping_request.time,
    };
    conn.write(ping_response.get()).await?;

    Ok(())
}

fn get_response(s: Option<&str>) -> Status {
    let motd = s.unwrap_or("A Minecraft Server");
    let text = TextComponent::new(motd.to_string());

    Status {
        description: FormattedText::Text(text),
        favicon: None,
        players: Players {
            max: 20,
            online: 0,
            sample: vec![],
        },
        version: Version {
            name: "1.20.1".to_string(),
            protocol: 763,
        },
        enforces_secure_chat: Some(false),
    }
}
