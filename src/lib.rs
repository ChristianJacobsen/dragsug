use protocol::Message;
use tokio::io::AsyncBufReadExt;

pub mod protocol;

pub fn setup_input_loop(tx: tokio::sync::mpsc::Sender<Message>) {
    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let reader = tokio::io::BufReader::new(stdin);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await.expect("the line to be read") {
            let msg: Message = serde_json::from_str(&line).expect("the line to be a valid message");
            tx.send(msg).await.unwrap();
        }
    });
}

pub fn send_reply(message: Message) {
    let reply = serde_json::to_string(&message).expect("the message to be serializable");
    println!("{}", reply);
}
