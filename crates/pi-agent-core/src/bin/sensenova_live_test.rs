use anyhow::Result;
use pi_agent_core::{AgentEvent, Message, ModelConfig, PiAgent};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Testing SenseNova Live Agent Integration ===");

    let api_key = std::env::var("SENSENOVA_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .unwrap_or_default();
    if api_key.is_empty() {
        eprintln!("Skipping live test: SENSENOVA_API_KEY or OPENAI_API_KEY is not set.");
        return Ok(());
    }

    // Test with the full chat completions URL to verify URL normalization
    let config = ModelConfig {
        provider: "openai-compatible".to_string(),
        model: "sensenova-6.8-flash-lite".to_string(),
        api_base: Some("https://token.sensenova.cn/v1/chat/completions".to_string()),
        api_key: Some(api_key),
        temperature: 0.1,
        max_tokens: Some(1024),
    };

    let workspace = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let agent = PiAgent::new(workspace, config);

    let mut messages = vec![Message::user(
        "请使用 bash 工具执行命令 `echo 'SenseNova-Volt-Integration-Success'` 并输出结果。",
    )];

    let (tx, mut rx) = mpsc::channel::<AgentEvent>(100);

    let agent_handle = tokio::spawn(async move { agent.run_turn(&mut messages, tx).await });

    while let Some(event) = rx.recv().await {
        match event {
            AgentEvent::WorkflowUpdated(snapshot) => {
                println!("[WORKFLOW] {} {:?}", snapshot.id, snapshot.status);
            }
            AgentEvent::Thought(thought) => {
                print!("[THOUGHT] {}", thought);
            }
            AgentEvent::TextDelta(delta) => {
                print!("{}", delta);
            }
            AgentEvent::ToolCallStarted {
                id,
                name,
                arguments,
            } => {
                println!(
                    "\n>>> [TOOL CALL START] {} (id: {}): {}",
                    name, id, arguments
                );
            }
            AgentEvent::ToolCallFinished {
                id,
                name,
                output,
                is_error,
            } => {
                println!(
                    "<<< [TOOL CALL FINISHED] {} (id: {}, err: {}):\n{}",
                    name, id, is_error, output
                );
            }
            AgentEvent::Completed { total_tokens } => {
                println!("\n[COMPLETED] Total tokens: {:?}", total_tokens);
            }
            AgentEvent::Error(err) => {
                eprintln!("\n[ERROR] {}", err);
            }
        }
    }

    let res = agent_handle.await?;
    res?;
    println!("\n=== SenseNova Live Agent Integration Test PASSED! ===");
    Ok(())
}
