use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicUsize, Ordering},
};

use dragsug::{
    protocol::{Body, ErrorCode, Message, Payload},
    send_reply, setup_gossip_loop, setup_input_loop,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    setup_input_loop(tx.clone());
    setup_gossip_loop(tx);

    let mut my_node_id = String::new();

    let mut messages_seen_by: HashMap<String, HashSet<usize>> = HashMap::new();

    let mut topology: HashMap<String, Vec<String>> = HashMap::new();

    let my_msg_id = AtomicUsize::new(0);

    while let Some(msg) = rx.recv().await {
        if let Payload::Gossip {} = msg.body.payload {
            if let Some(neighbors) = topology.get(&my_node_id) {
                let messages_seen_by_me = messages_seen_by
                    .entry(my_node_id.clone())
                    .or_default()
                    .clone();

                for neighbor in neighbors {
                    let messages_seen_by_neighbor = messages_seen_by
                        .entry(neighbor.clone())
                        .or_default()
                        .clone();

                    let messages_to_send =
                        messages_seen_by_me.difference(&messages_seen_by_neighbor);

                    for message in messages_to_send {
                        let reply = Message {
                            src: my_node_id.clone(),
                            dst: neighbor.clone(),
                            body: Body {
                                msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                                in_reply_to: None,
                                payload: Payload::Broadcast { message: *message },
                            },
                        };

                        send_reply(reply);
                    }
                }
            }
        } else {
            let msg_id = msg.body.msg_id;

            let reply = match msg.body.payload {
                Payload::Init {
                    node_id,
                    node_ids: _,
                } => {
                    my_node_id = node_id;

                    Message {
                        src: msg.dst,
                        dst: msg.src,
                        body: Body {
                            msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                            in_reply_to: msg_id,
                            payload: Payload::InitOk {},
                        },
                    }
                }
                Payload::Broadcast { message } => {
                    messages_seen_by
                        .entry(msg.src.clone())
                        .or_default()
                        .insert(message);

                    messages_seen_by
                        .entry(my_node_id.clone())
                        .or_default()
                        .insert(message);

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
                Payload::Read {} => {
                    let messages_seen_by_me = messages_seen_by
                        .get(&my_node_id)
                        .unwrap_or(&HashSet::new())
                        .iter()
                        .cloned()
                        .collect();

                    Message {
                        src: msg.dst,
                        dst: msg.src,
                        body: Body {
                            msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                            in_reply_to: msg_id,
                            payload: Payload::ReadOk {
                                messages: messages_seen_by_me,
                            },
                        },
                    }
                }
                Payload::Topology {
                    topology: new_topology,
                } => {
                    topology = new_topology;

                    Message {
                        src: msg.dst,
                        dst: msg.src,
                        body: Body {
                            msg_id: Some(my_msg_id.fetch_add(1, Ordering::Relaxed)),
                            in_reply_to: msg_id,
                            payload: Payload::TopologyOk {},
                        },
                    }
                }
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
    }

    Ok(())
}
