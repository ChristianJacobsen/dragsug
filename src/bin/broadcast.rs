use std::{
    collections::HashSet,
    sync::atomic::{AtomicUsize, Ordering},
};

use dragsug::{
    protocol::{Body, ErrorCode, Message, Payload},
    send_reply, setup_input_loop,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    setup_input_loop(tx);

    let mut my_messages = HashSet::new();

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
            Payload::Broadcast { message } => {
                my_messages.insert(message);

                Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Body {
                        msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                        in_reply_to: msg_id,
                        payload: Payload::BroadcastOk {},
                    },
                }
            }
            Payload::Read {} => Message {
                src: msg.dst,
                dst: msg.src,
                body: Body {
                    msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                    in_reply_to: msg_id,
                    payload: Payload::ReadOk {
                        messages: my_messages.iter().copied().collect(),
                    },
                },
            },
            Payload::Topology { topology: _ } => Message {
                src: msg.dst,
                dst: msg.src,
                body: Body {
                    msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                    in_reply_to: msg_id,
                    payload: Payload::TopologyOk {},
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
