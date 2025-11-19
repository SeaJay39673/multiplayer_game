use serde::{Deserialize, Serialize, de::DeserializeOwned};
mod map;
pub use map::*;
mod player;
pub use player::*;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::tcp::{OwnedReadHalf, OwnedWriteHalf}};
use anyhow::{Result};

#[derive(Deserialize, Serialize)]
pub enum ClientMessage {
    MessageRequest(PlayerMessage),
    MapRequest(String),
    ConnectionRequest,
    MoveRequest{
        player: String,
        direction: [f32;3],
    },
    Disconnect,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum ServerMessage {
    Map(TileManager),
    Player(Player),
    OtherPlayer(Player),
    Message(PlayerMessage),
    Disconnect(String),
}


pub async fn send_message<T: Serialize>(writer: &mut OwnedWriteHalf, msg: &T) -> Result<()> {
    let bytes = bincode::serde::encode_to_vec(msg, bincode::config::standard())?;
    let len = (bytes.len() as u32).to_be_bytes();

    writer.write_all(&len).await?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn read_message<T: DeserializeOwned>(reader: &mut OwnedReadHalf) -> Result<T> {
    let mut len_buf = [0u8;4];
    reader.read_exact(&mut len_buf).await?;

    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut msg_buf = vec![0u8; msg_len];
    reader.read_exact(&mut msg_buf).await?;

    let (msg, _): (T, usize) = 
        bincode::serde::decode_from_slice(&msg_buf, bincode::config::standard())?; 

    Ok(msg)
}