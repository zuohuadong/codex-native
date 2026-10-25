use pi_agent_core::{workflow::Status, AgentEvent, Message, ModelConfig, PiAgent};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Barrier},
    task::{JoinHandle, JoinSet},
};

#[derive(Clone, Copy, PartialEq)]
enum Scenario {
    Success,
    Failure,
    Hold,
    DeniedTools,
    SlowHeaders,
    Heartbeats,
}

struct Server {
    base: String,
    requests: Arc<Mutex<Vec<Value>>>,
    closed: Arc<AtomicUsize>,
    task: JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Server {
    async fn start(scenario: Scenario) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}/v1", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let closed = Arc::new(AtomicUsize::new(0));
        let seen = requests.clone();
        let disconnected = closed.clone();
        // Both worker HTTP requests must arrive before either can complete.
        // A sequential executor times out instead of passing this contract.
        let barrier = Arc::new(Barrier::new(2));
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        let (socket, _) = result.unwrap();
                        connections.spawn(handle(socket, scenario, seen.clone(), disconnected.clone(), barrier.clone()));
                    }
                    Some(result) = connections.join_next() => { result.unwrap(); }
                }
            }
        });
        Self {
            base,
            requests,
            closed,
            task,
        }
    }
}

async fn read_request(socket: &mut TcpStream) -> Value {
    let mut bytes = Vec::new();
    loop {
        let mut buffer = [0; 4096];
        let count = socket.read(&mut buffer).await.unwrap();
        assert!(count > 0);
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
            let length: usize = headers
                .lines()
                .find_map(|line| line.strip_prefix("content-length:"))
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            if bytes.len() >= end + 4 + length {
                return serde_json::from_slice(&bytes[end + 4..end + 4 + length]).unwrap();
            }
        }
    }
}

fn tool(id: &str, name: &str, arguments: Value) -> Value {
    json!({"tool_calls":[{"index":0,"id":id,"type":"function","function":{"name":name,"arguments":arguments.to_string()}}]})
}

async fn handle(
    mut socket: TcpStream,
    scenario: Scenario,
    requests: Arc<Mutex<Vec<Value>>>,
    closed: Arc<AtomicUsize>,
    barrier: Arc<Barrier>,
) {
    let request = read_request(&mut socket).await;
    requests.lock().unwrap().push(request.clone());
    let messages = request["messages"].as_array().unwrap();
    let specialist = messages[0]["content"]
        .as_str()
        .unwrap()
        .starts_with("You are a read-only specialist");
    let last = messages.last().unwrap();
    let is_worker_a = messages
        .iter()
        .any(|message| message["role"] == "user" && message["content"] == "inspect alpha");
    let delta = if specialist {
        if last["role"] == "user" {
            barrier.wait().await;
            if is_worker_a && matches!(scenario, Scenario::SlowHeaders | Scenario::Heartbeats) {
                if scenario == Scenario::Heartbeats {
                    socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n").await.unwrap();
                }
                let mut buf = [0; 1];
                loop {
                    tokio::select! {
                        _ = socket.read(&mut buf) => break,
                        _ = tokio::time::sleep(Duration::from_millis(10)), if scenario == Scenario::Heartbeats => {
                            if socket.write_all(b": heartbeat\n\n").await.is_err() { break; }
                        }
                    }
                }
                closed.fetch_add(1, Ordering::SeqCst);
                return;
            }
            if scenario == Scenario::Hold {
                socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: {\"choices\":[{\"delta\":{\"content\":\"working\"}}]}\n\n").await.unwrap();
                let mut buf = [0; 1];
                if matches!(socket.read(&mut buf).await, Ok(0) | Err(_)) {
                    closed.fetch_add(1, Ordering::SeqCst);
                }
                return;
            }
            if scenario == Scenario::Failure && is_worker_a {
                socket.write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await.unwrap();
                return;
            }
            if scenario == Scenario::DeniedTools {
                if is_worker_a {
                    tool(
                        "forbidden-write",
                        "write_file",
                        json!({"path":"forbidden.txt","content":"unexpected write"}),
                    )
                } else {
                    tool(
                        "forbidden-delegate",
                        "delegate_agents",
                        json!({"tasks":[{"label":"Nested","prompt":"inspect nested"}]}),
                    )
                }
            } else {
                tool("read", "read_file", json!({"path":"evidence.txt"}))
            }
        } else {
            json!({"content": if is_worker_a {"alpha evidence"} else {"beta evidence"}})
        }
    } else if last["role"] == "user" {
        tool(
            "delegation-1",
            "delegate_agents",
            json!({"title":"Inspect in parallel","tasks":[{"label":"Alpha","prompt":"inspect alpha"},{"label":"Beta","prompt":"inspect beta"}]}),
        )
    } else {
        json!({"content":"Parent synthesis"})
    };
    let body = format!(
        "data: {}\n\ndata: [DONE]\n\n",
        json!({"choices":[{"delta":delta}]})
    );
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
    socket.write_all(response.as_bytes()).await.unwrap();
}

fn agent(server: &Server, root: &std::path::Path) -> PiAgent {
    PiAgent::new(
        root,
        ModelConfig {
            api_base: Some(server.base.clone()),
            api_key: None,
            model: "local-contract".into(),
            ..ModelConfig::default()
        },
    )
}

#[tokio::test]
async fn stopping_one_workflow_keeps_completed_results_and_parent_synthesis() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("evidence.txt"), "local evidence").unwrap();
        let server = Server::start(Scenario::SlowHeaders).await;
        let agent = agent(&server, root.path());
        let controls = agent.workflow_controls.clone();
        let (tx, mut rx) = mpsc::channel(128);
        let run = tokio::spawn(async move {
            let mut messages = vec![Message::user("inspect")];
            agent.run_turn(&mut messages, tx).await.unwrap();
            messages
        });
        let id = loop {
            if let Some(AgentEvent::WorkflowUpdated(state)) = rx.recv().await {
                if state.stages[0].nodes[1].status == Status::Completed {
                    break state.id;
                }
            }
        };
        assert!(!controls.stop("another-workflow"));
        assert!(controls.stop(&id));
        assert!(!controls.stop(&id));
        let mut terminal = None;
        let mut parent_text = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::WorkflowUpdated(state) => terminal = Some(state),
                AgentEvent::TextDelta(text) => parent_text.push_str(&text),
                _ => {}
            }
        }
        let messages = run.await.unwrap();
        let terminal = terminal.unwrap();
        assert_eq!(terminal.status, Status::Interrupted);
        assert_eq!(terminal.stages[0].nodes[0].status, Status::Interrupted);
        assert_eq!(terminal.stages[0].nodes[1].status, Status::Completed);
        assert_eq!(terminal.stages[0].visible_progress().unwrap().settled, 1);
        assert!(terminal.delivery.contains("beta evidence"));
        assert!(terminal.delivery.contains("Stopped by user"));
        assert_eq!(parent_text, "Parent synthesis");
        assert_eq!(messages.last().unwrap().content, "Parent synthesis");
        assert_eq!(controls.stopping(&id), None);
        while server.closed.load(Ordering::SeqCst) < 1 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("scoped stop must settle without aborting the parent");
}

#[tokio::test]
async fn parallel_workers_emit_scoped_snapshots_and_return_real_results() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("evidence.txt"), "local evidence").unwrap();
        let server = Server::start(Scenario::Success).await;
        let mut messages = vec![Message::user("please inspect")];
        let anchor = messages[0].id.clone();
        let (tx, mut rx) = mpsc::channel(128);
        agent(&server, root.path())
            .run_turn(&mut messages, tx)
            .await
            .unwrap();
        let mut snapshots = Vec::new();
        let mut text = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::WorkflowUpdated(snapshot) => snapshots.push(snapshot),
                AgentEvent::TextDelta(delta) => text.push_str(&delta),
                _ => {}
            }
        }
        assert_eq!(
            text, "Parent synthesis",
            "child output must stay inside its workflow"
        );
        assert!(snapshots
            .iter()
            .any(|snapshot| snapshot.running_agents() == 2));
        assert!(snapshots
            .windows(2)
            .all(|pair| pair[0].revision < pair[1].revision));
        assert!(
            snapshots
                .iter()
                .all(|snapshot| snapshot.parent_message_id == anchor
                    && snapshot.id == snapshots[0].id)
        );
        let final_state = snapshots.last().unwrap();
        assert_eq!(final_state.status, Status::Completed);
        assert_eq!(final_state.stages[0].visible_progress().unwrap().settled, 2);
        assert_eq!(final_state.running_agents(), 0);
        assert!(
            final_state.delivery.contains("alpha evidence")
                && final_state.delivery.contains("beta evidence")
        );
        assert!(messages.iter().any(|message| message
            .tool_result
            .as_ref()
            .is_some_and(|result| result.name == "delegate_agents"
                && result.output.contains("alpha evidence"))));
        let requests = server.requests.lock().unwrap();
        let worker_requests: Vec<_> = requests
            .iter()
            .filter(|request| {
                request["messages"][0]["content"]
                    .as_str()
                    .unwrap()
                    .starts_with("You are a read-only specialist")
            })
            .collect();
        assert_eq!(worker_requests.len(), 4);
        for request in worker_requests {
            let names: Vec<_> = request["tools"]
                .as_array()
                .unwrap()
                .iter()
                .map(|tool| tool["function"]["name"].as_str().unwrap())
                .collect();
            assert_eq!(names, ["read_file", "grep", "find"]);
        }
        assert_eq!(
            requests
                .iter()
                .filter(|request| request["messages"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|message| message["role"] == "tool"
                        && message["content"]
                            .as_str()
                            .unwrap()
                            .contains("local evidence")))
                .count(),
            2
        );
    })
    .await
    .expect("parallel delegation should finish");
}

#[tokio::test]
async fn failed_worker_settles_without_discarding_successful_sibling() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("evidence.txt"), "local evidence").unwrap();
        let server = Server::start(Scenario::Failure).await;
        let (tx, mut rx) = mpsc::channel(128);
        agent(&server, root.path())
            .run_turn(&mut vec![Message::user("inspect")], tx)
            .await
            .unwrap();
        let mut snapshots = Vec::new();
        while let Some(event) = rx.recv().await {
            if let AgentEvent::WorkflowUpdated(snapshot) = event {
                snapshots.push(snapshot);
            }
        }
        let state = snapshots.last().unwrap();
        assert_eq!(state.status, Status::Failed);
        assert_eq!(state.stages[0].nodes[0].status, Status::Failed);
        assert_eq!(state.stages[0].nodes[1].status, Status::Completed);
        assert_eq!(state.stages[0].visible_progress().unwrap().settled, 2);
        assert!(state.delivery.contains("beta evidence"));
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn aborting_parent_closes_both_inflight_worker_requests() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let server = Server::start(Scenario::Hold).await;
        let agent = agent(&server, root.path());
        let (tx, mut rx) = mpsc::channel(128);
        let run = tokio::spawn(async move {
            agent
                .run_turn(&mut vec![Message::user("inspect")], tx)
                .await
        });
        // Wait for actual streamed child output, not just an initial snapshot.
        loop {
            if let Some(AgentEvent::WorkflowUpdated(state)) = rx.recv().await {
                if state.stages[0]
                    .nodes
                    .iter()
                    .all(|node| node.detail.contains("working"))
                {
                    break;
                }
            }
        }
        run.abort();
        assert!(run.await.unwrap_err().is_cancelled());
        while server.closed.load(Ordering::SeqCst) < 2 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(server.requests.lock().unwrap().len(), 3);
    })
    .await
    .expect("cancellation must not leave a worker request alive");
}

#[tokio::test]
async fn child_cannot_execute_writer_or_recursive_delegation_tools() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let server = Server::start(Scenario::DeniedTools).await;
        let (tx, mut rx) = mpsc::channel(128);
        agent(&server, root.path())
            .run_turn(&mut vec![Message::user("inspect")], tx)
            .await
            .unwrap();
        let mut workflow_ids = std::collections::HashSet::new();
        while let Some(event) = rx.recv().await {
            if let AgentEvent::WorkflowUpdated(state) = event {
                workflow_ids.insert(state.id);
            }
        }
        assert_eq!(
            workflow_ids.len(),
            1,
            "no nested workflow may be dispatched"
        );
        assert!(!root.path().join("forbidden.txt").exists());
        let requests = server.requests.lock().unwrap();
        for name in ["write_file", "delegate_agents"] {
            assert!(requests.iter().any(|request| request["messages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|message| {
                    message["role"] == "tool"
                        && message["content"]
                            .as_str()
                            .unwrap()
                            .contains(&format!("Unknown tool: {name}"))
                })));
        }
        assert_eq!(
            requests.len(),
            6,
            "only the parent and two initial children may call the model"
        );
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn deadline_covers_missing_headers_and_continuous_heartbeats() {
    for scenario in [Scenario::SlowHeaders, Scenario::Heartbeats] {
        tokio::time::timeout(Duration::from_secs(10), async {
            let root = tempfile::tempdir().unwrap();
            std::fs::write(root.path().join("evidence.txt"), "local evidence").unwrap();
            let server = Server::start(scenario).await;
            let mut agent = agent(&server, root.path());
            agent.delegation_timeout = Duration::from_secs(1);
            let (tx, mut rx) = mpsc::channel(128);
            agent
                .run_turn(&mut vec![Message::user("inspect")], tx)
                .await
                .unwrap();
            let mut last = None;
            while let Some(event) = rx.recv().await {
                if let AgentEvent::WorkflowUpdated(state) = event {
                    last = Some(state);
                }
            }
            let state = last.unwrap();
            assert_eq!(state.status, Status::Failed);
            assert_eq!(state.stages[0].nodes[0].status, Status::Failed);
            assert!(state.stages[0].nodes[0].detail.contains("exceeded"));
            assert_eq!(state.stages[0].nodes[1].status, Status::Completed);
            assert_eq!(state.stages[0].visible_progress().unwrap().settled, 2);
            while server.closed.load(Ordering::SeqCst) < 1 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("deadline should settle the stalled worker");
    }
}
