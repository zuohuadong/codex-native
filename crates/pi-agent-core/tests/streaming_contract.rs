use pi_agent_core::{provider::LlmClient, AgentEvent, Message, ModelConfig};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};

async fn serve(body: Vec<u8>, split: bool) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let worker = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        loop {
            let mut buf = [0; 4096];
            let count = socket.read(&mut buf).await.unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buf[..count]);
            if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..end]).to_lowercase();
                let size: usize = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                if request.len() >= end + 4 + size {
                    break;
                }
            }
        }
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n").await.unwrap();
        for chunk in body.chunks(if split { 1 } else { body.len() }) {
            let frame = format!("{:x}\r\n", chunk.len());
            if socket.write_all(frame.as_bytes()).await.is_err() {
                return;
            }
            if socket.write_all(chunk).await.is_err() {
                return;
            }
            if socket.write_all(b"\r\n").await.is_err() {
                return;
            }
            if split {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        let _ = socket.write_all(b"0\r\n\r\n").await;
    });
    (format!("http://{address}/v1"), worker)
}

#[tokio::test]
async fn chinese_deltas_survive_split_utf8_packets() {
    let body = "data: {\"choices\":[{\"delta\":{\"content\":\"中文恢复🙂\"}}]}\n\ndata: [DONE]\n\n";
    let (base, server) = serve(body.as_bytes().to_vec(), true).await;
    let config = ModelConfig {
        api_base: Some(base),
        ..Default::default()
    };
    let (tx, mut rx) = mpsc::channel(16);
    let message = LlmClient::new()
        .stream_chat(&config, &[Message::user("test")], None, tx)
        .await
        .unwrap();
    assert_eq!(message.content, "中文恢复🙂");
    assert!(matches!(rx.recv().await,Some(AgentEvent::TextDelta(text)) if text=="中文恢复🙂"));
    server.await.unwrap();
}

#[tokio::test]
async fn disconnected_or_malformed_streams_do_not_report_success() {
    for body in [
        "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n",
        "data: {invalid json}\n\ndata: [DONE]\n\n",
        "data: {\"error\":{\"message\":\"failed\"}}\n\n",
        "data: {\"choices\":[{\"finish_reason\":false}]}\n\n",
        "data: {\"choices\":[{\"finish_reason\":\"length\"}]}\n\n",
    ] {
        let (base, server) = serve(body.as_bytes().to_vec(), false).await;
        let config = ModelConfig {
            api_base: Some(base),
            ..Default::default()
        };
        let (tx, mut rx) = mpsc::channel(16);
        assert!(LlmClient::new()
            .stream_chat(&config, &[Message::user("test")], None, tx)
            .await
            .is_err());
        while let Some(event) = rx.recv().await {
            assert!(!matches!(event, AgentEvent::Completed { .. }));
        }
        server.await.unwrap();
    }
}
