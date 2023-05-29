use std::sync::atomic::{AtomicUsize, Ordering};

use dragsug::{
    protocol::{Body, ErrorCode, Message, Payload},
    send_reply, setup_input_loop,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    setup_input_loop(tx);

    let my_msg_id = AtomicUsize::new(0);

    while let Some(msg) = rx.recv().await {
        let msg_id = msg.body.msg_id;

        let reply = match msg.body.payload {
            Payload::Init {
                node_id: _,
                node_ids: _,
            } => Message {
                src: msg.dst,
                dst: msg.src,
                body: Body {
                    msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                    in_reply_to: msg_id,
                    payload: Payload::InitOk {},
                },
            },
            Payload::Generate {} => Message {
                src: msg.dst,
                dst: msg.src,
                body: Body {
                    msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                    in_reply_to: msg_id,
                    payload: Payload::GenerateOk {
                        id: uuid::Uuid::new_v4(),
                    },
                },
            },
            _ => Message {
                src: msg.dst,
                dst: msg.src,
                body: Body {
                    msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                    in_reply_to: msg_id,
                    payload: Payload::Error {
                        code: ErrorCode::NotSupported,
                        text: String::from("Operation not supported"),
                    },
                },
            },
        };

        send_reply(reply);
    }

    Ok(())
}
