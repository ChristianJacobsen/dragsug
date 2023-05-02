use tokio::io::AsyncBufReadExt;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Payload {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: usize,
    },
    Echo {
        msg_id: usize,
        echo: String,
    },
    EchoOk {
        msg_id: usize,
        in_reply_to: usize,
        echo: String,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Payload,
}

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

    while let Some(msg) = rx.recv().await {
        eprintln!("{:?}", msg);

        match msg.body {
            Payload::Init {
                msg_id,
                node_id,
                node_ids,
            } => {
                _my_node_id = node_id;
                _my_node_ids = node_ids;
                let reply = Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::InitOk {
                        in_reply_to: msg_id,
                    },
                };
                let reply = serde_json::to_string(&reply)?;
                println!("{}", reply);
            }
            Payload::Echo { msg_id, echo } => {
                let reply = Message {
                    src: msg.dst,
                    dst: msg.src,
                    body: Payload::EchoOk {
                        msg_id,
                        in_reply_to: msg_id,
                        echo,
                    },
                };
                let reply = serde_json::to_string(&reply)?;
                println!("{}", reply);
            }
            Payload::InitOk { .. } | Payload::EchoOk { .. } => {
                panic!("Should never receive a reply type message");
            }
        }
    }

    Ok(())
}
