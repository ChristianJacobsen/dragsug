use std::sync::atomic::{AtomicUsize, Ordering};

use protocol::{ErrorCode, Message, Payload};
use tokio::io::AsyncBufReadExt;

mod protocol;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let reader = tokio::io::BufReader::new(stdin);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await.expect("the line to be read") {
            let msg: Message = serde_json::from_str(&line).expect("the line to be a valid message");
            tx.send(msg).await.unwrap();
        }
    });

    let mut _my_node_id: String;
    let mut _my_node_ids: Vec<String>;

    let my_msg_id = AtomicUsize::new(0);

    while let Some(msg) = rx.recv().await {
        let reply = match msg.body {
            Payload::Init {
                msg_id,
                node_id,
                node_ids,
            } => {
                _my_node_id = node_id;
                _my_node_ids = node_ids;
                Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::InitOk {
                        msg_id: my_msg_id.fetch_add(1, Ordering::Relaxed),
                        in_reply_to: msg_id,
                    },
                }
            }
            Payload::Echo { msg_id, echo } => Message {
                src: msg.dst,
                dst: msg.src,
                body: Payload::EchoOk {
                    msg_id: my_msg_id.fetch_add(1, Ordering::Relaxed),
                    in_reply_to: msg_id,
                    echo,
                },
            },
            Payload::Generate { msg_id } => Message {
                src: msg.dst,
                dst: msg.src,
                body: Payload::GenerateOk {
                    msg_id: my_msg_id.fetch_add(1, Ordering::Relaxed),
                    in_reply_to: msg_id,
                    id: uuid::Uuid::new_v4(),
                },
            },
            _ => Message {
                src: msg.dst,
                dst: msg.src,
                body: Payload::Error {
                    msg_id: my_msg_id.fetch_add(1, Ordering::Relaxed),
                    in_reply_to: 0,
                    code: ErrorCode::NotSupported,
                    text: String::from("Operation not supported"),
                },
            },
        };
        let reply = serde_json::to_string(&reply).expect("the reply to be serializable");
        println!("{}", reply);
    }

    Ok(())
}
