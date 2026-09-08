use tagisan::{
    ChatSession, ContentBlock, Message, ProviderCapabilities, Role, TokenBudgetTracker,
};

#[test]
fn test_message_creation_and_extraction() {
    let msg = Message::user("Hello World");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.extract_text(), "Hello World");
    assert_eq!(msg.extract_thinking(), None);

    let thinking_msg = Message {
        role: Role::Assistant,
        content: vec![
            ContentBlock::Thinking {
                thinking: "Let me calculate 2+2".to_string(),
                signature: None,
            },
            ContentBlock::Text {
                text: "The answer is 4.".to_string(),
            },
        ],
        name: None,
        metadata: Default::default(),
    };

    assert_eq!(thinking_msg.extract_text(), "The answer is 4.");
    assert_eq!(
        thinking_msg.extract_thinking(),
        Some("Let me calculate 2+2".to_string())
    );
}

#[test]
fn test_token_budget_tracker_calculation_and_limits() {
    let tracker = TokenBudgetTracker::new(0.05); // $0.05 budget

    // Claude 3.5 Sonnet: 1000 prompt tokens ($0.003), 1000 completion tokens ($0.015) = $0.018
    let cost = tracker
        .record("claude-3-5-sonnet-20241022", 1000, 1000)
        .expect("Should succeed within budget");

    assert!((cost - 0.018).abs() < 0.001);
    assert!((tracker.current_spent_usd() - 0.018).abs() < 0.001);

    // Another 3000 completion tokens -> exceeds $0.05 budget
    let result = tracker.record("claude-3-5-sonnet-20241022", 1000, 3000);
    assert!(result.is_err(), "Should trip budget limit error");
}

#[test]
fn test_provider_capabilities_bitflags() {
    let caps = ProviderCapabilities::STREAMING
        | ProviderCapabilities::REASONING_EXTRACTION
        | ProviderCapabilities::PROMPT_CACHING;

    assert!(caps.contains(ProviderCapabilities::STREAMING));
    assert!(caps.contains(ProviderCapabilities::REASONING_EXTRACTION));
    assert!(caps.contains(ProviderCapabilities::PROMPT_CACHING));
    assert!(!caps.contains(ProviderCapabilities::VISION));
}

#[test]
fn test_chat_session_builder() {
    let mut session = ChatSession::new().with_system("You are a helpful Rust expert.");
    session.add_user_message("How do I write async streams in Rust?");
    session.add_assistant_message("Use the async-stream crate or futures::stream.");

    assert_eq!(session.history.len(), 2);
    assert_eq!(
        session.system_prompt.as_deref(),
        Some("You are a helpful Rust expert.")
    );

    let req = session.build_request("claude-3-5-sonnet-20241022");
    assert_eq!(req.model, "claude-3-5-sonnet-20241022");
    assert_eq!(req.messages.len(), 2);
    assert_eq!(req.system_prompt, session.system_prompt);
}
