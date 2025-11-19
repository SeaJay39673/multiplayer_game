use std::net::SocketAddr;

use anyhow::Result;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::{UnboundedSender, unbounded_channel},
    },
};

use shared::{ClientMessage, Player, ServerMessage, read_message, send_message};
use uuid::Uuid;

use crate::state::{PLAYERS, TILE_MANAGER};

mod state;

async fn handle_client_message(
    msg: ClientMessage,
    tx: &broadcast::Sender<ServerMessage>,
    incoming_tx: &UnboundedSender<ServerMessage>,
    addr: &SocketAddr,
    id: String,
) {
    match msg {
        ClientMessage::Disconnect => {
            println!("Client disconnected");
            return;
        }
        ClientMessage::ConnectionRequest => {
            let player = {
                let mut players = PLAYERS.write().await;
                let map = TILE_MANAGER.read().await;
                let pos = if let Some(pos) = map
                    .tiles
                    .iter()
                    .filter(|((x, y, z), tile)| *x == 0 && *y == 0)
                    .map(|((x, y, z), tile)| [*x as f32, *y as f32, *z as f32])
                    .next()
                {
                    pos
                } else {
                    [0.0, 0.0, 0.0]
                };
                let player = players.entry(id.clone()).or_insert(Player {
                    id: id.clone(),
                    position: pos,
                    speed: 0.025,
                });
                player.clone()
            };

            if let Err(e) = incoming_tx.send(ServerMessage::Player(player.clone())) {
                println!("Could not update client {addr} new player: {e}");
                return;
            }

            if let Err(e) = incoming_tx.send(ServerMessage::Map(TILE_MANAGER.read().await.clone()))
            {
                println!("Could not update client {addr} with map: {e}");
            }

            if let Err(e) = tx.send(ServerMessage::OtherPlayer(player)) {
                println!("Could not broadcast client {addr} player to other clients: {e}");
            }
        }
        ClientMessage::MoveRequest { player, direction } => {
            println!("Move request from client {addr}");
            let player: Option<Player> = {
                let mut players = PLAYERS.write().await;

                let player = if let Some(player) = players.get_mut(&player) {
                    let new_x = player.position[0] + direction[0] * player.speed;
                    let new_y = player.position[1] + direction[1] * player.speed;
                    let current_z = player.position[2];

                    let tx = new_x.floor() as i64;
                    let ty = new_y.floor() as i64;
                    let tz = current_z.floor() as i64;

                    let should_jump = {
                        let map = TILE_MANAGER.read().await;
                        map.tiles.get(&(tx, ty, tz + 1)).is_some()
                    };

                    let should_fall = {
                        let map = TILE_MANAGER.read().await;
                        map.tiles.get(&(tx, ty, tz - 1)).is_some()
                    };

                    if should_jump {
                        player.position[2] += 1.0;
                    } else if should_fall {
                        player.position[2] -= 1.0;
                    } else {
                        player.position[0] = new_x;
                        player.position[1] = new_y;
                    }

                    Some(player.clone())
                } else {
                    None
                };

                player
            };

            if let Some(player) = player {
                if let Err(e) = incoming_tx.send(ServerMessage::Player(player.clone())) {
                    println!("Could not update player location: {e}");
                    return;
                }
                if let Err(e) = tx.send(ServerMessage::OtherPlayer(player.clone())) {
                    println!("Could not broadcast player location: {e}");
                }
            }
        }
        _ => {}
    }
}

// async fn handle_broadcast_message()

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    tx: broadcast::Sender<ServerMessage>,
    mut rx: broadcast::Receiver<ServerMessage>,
) {
    let (mut reader, mut writer) = stream.into_split();

    let id = Uuid::new_v4().to_string();

    let (incoming_tx, mut incoming_rx) = unbounded_channel::<ServerMessage>();

    tokio::spawn({
        let id = id.clone();
        async move {
            loop {
                match read_message::<ClientMessage>(&mut reader).await {
                    Ok(msg) => {
                        handle_client_message(msg, &tx, &incoming_tx, &addr, id.clone()).await
                    }
                    Err(e) => {
                        println!("Client {addr} disconnected: {e}");
                        break;
                    }
                }
            }
        }
    });

    tokio::spawn({
        let id = id.clone();
        async move {
            loop {
                tokio::select! {
                    incoming_msg = incoming_rx.recv() => {
                                if let Some(msg) = incoming_msg {
                                    if let Err(e) = send_message(&mut writer, &msg).await {
                                        println!("Error sending message to client {addr}: {e}");
                                    }
                                };
                            }

                    msg = rx.recv() => {
                        match msg {
                            Ok(msg) => {
                                let broadcast: bool = match msg.clone() {
                                    ServerMessage::OtherPlayer(p) => {
                                        p.id != id.clone()
                                    }
                                    _ => true
                                };

                                if broadcast && let Err(e) = send_message(&mut writer, &msg).await {
                                    println!("Error broadcasting message to client {addr}: {e}");
                                }
                            },
                            Err(RecvError::Closed) => {
                                println!("Broadcast channel closed for {addr}");
                                break;
                            },
                            Err(e) => {
                                println!("Error receiving from broadcast channel: {e}");
                            }
                        }
                    }
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:5250").await?;

    let (tx, _rx) = broadcast::channel::<ServerMessage>(100);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Client connected: {addr}");

        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(async move { handle_connection(stream, addr, tx, rx).await });
    }
}
