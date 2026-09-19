use tagisan::swarm::repl::ReplEditor;

#[test]
fn test_agy_keyboard_word_navigation_backward() {
    let text: Vec<char> = "antigravity dual-core command-line interface".chars().collect();
    // At the very end (index 44)
    let pos1 = ReplEditor::find_word_backward(&text, text.len());
    assert_eq!(pos1, 35); // start of "interface"
    let word1: String = text[pos1..text.len()].iter().collect();
    assert_eq!(word1, "interface");

    // From 35 (start of "interface")
    let pos2 = ReplEditor::find_word_backward(&text, pos1);
    assert_eq!(pos2, 22); // start of "command-line"
    let word2: String = text[pos2..pos1].iter().collect();
    assert_eq!(word2.trim(), "command-line");

    // From 22 (start of "command-line")
    let pos3 = ReplEditor::find_word_backward(&text, pos2);
    assert_eq!(pos3, 12); // start of "dual-core"

    // From 12 (start of "dual-core")
    let pos4 = ReplEditor::find_word_backward(&text, pos3);
    assert_eq!(pos4, 0); // start of "antigravity"

    // From 0
    let pos5 = ReplEditor::find_word_backward(&text, 0);
    assert_eq!(pos5, 0);
}

#[test]
fn test_agy_keyboard_word_navigation_forward() {
    let text: Vec<char> = "tgs repl --agent architect".chars().collect();
    // At start (0)
    let pos1 = ReplEditor::find_word_forward(&text, 0);
    assert_eq!(pos1, 4); // start of "repl"

    // At 4
    let pos2 = ReplEditor::find_word_forward(&text, pos1);
    assert_eq!(pos2, 9); // start of "--agent"

    // At 9
    let pos3 = ReplEditor::find_word_forward(&text, pos2);
    assert_eq!(pos3, 17); // start of "architect"

    // At 17
    let pos4 = ReplEditor::find_word_forward(&text, pos3);
    assert_eq!(pos4, text.len()); // end of line

    // Past end
    let pos5 = ReplEditor::find_word_forward(&text, text.len());
    assert_eq!(pos5, text.len());
}

#[test]
fn test_agy_keyboard_longest_common_prefix() {
    // Exact match
    assert_eq!(
        ReplEditor::longest_common_prefix(&["/model".to_string(), "/memory".to_string()]),
        "/m"
    );

    // Identical
    assert_eq!(
        ReplEditor::longest_common_prefix(&["/help".to_string(), "/help".to_string()]),
        "/help"
    );

    // Divergent start
    assert_eq!(
        ReplEditor::longest_common_prefix(&["foo".to_string(), "bar".to_string()]),
        ""
    );

    // Empty list
    assert_eq!(ReplEditor::longest_common_prefix(&[]), "");
}

#[test]
fn test_agy_tab_completion_slash_commands() {
    // Tab on "/h" -> "/help ", "/history "
    let comp_h = ReplEditor::get_completions("/h");
    let names: Vec<String> = comp_h.into_iter().map(|(s, _)| s).collect();
    assert!(names.contains(&"/help ".to_string()));
    assert!(names.contains(&"/history ".to_string()));

    // Tab on "/p" -> "/plan "
    let comp_p = ReplEditor::get_completions("/p");
    assert_eq!(comp_p.len(), 1);
    assert_eq!(comp_p[0].0, "/plan ");

    // Tab on "/g" -> "/goal "
    let comp_g = ReplEditor::get_completions("/g");
    assert_eq!(comp_g.len(), 1);
    assert_eq!(comp_g[0].0, "/goal ");

    // Tab on "/b" -> "/budget ", "/bun "
    let comp_b = ReplEditor::get_completions("/b");
    let b_names: Vec<String> = comp_b.into_iter().map(|(s, _)| s).collect();
    assert!(b_names.contains(&"/budget ".to_string()));
    assert!(b_names.contains(&"/bun ".to_string()));
}

#[test]
fn test_agy_tab_completion_agent_personas() {
    // /agent arch -> /agent architect
    let comp = ReplEditor::get_completions("/agent arch");
    assert!(!comp.is_empty());
    assert_eq!(comp[0].0, "/agent architect ");

    // /agent vibe -> /agent vibe-code-reviewer
    let comp_vibe = ReplEditor::get_completions("/agent vibe");
    assert!(!comp_vibe.is_empty());
    assert_eq!(comp_vibe[0].0, "/agent vibe-code-reviewer ");

    // /agent tdd -> /agent tdd-engineer
    let comp_tdd = ReplEditor::get_completions("/agent tdd");
    assert!(!comp_tdd.is_empty());
    assert_eq!(comp_tdd[0].0, "/agent tdd-engineer ");
}

#[test]
fn test_agy_tab_completion_skills() {
    // /skill tdd -> /skill tdd-workflow
    let comp_tdd = ReplEditor::get_completions("/skill tdd");
    assert!(!comp_tdd.is_empty());
    assert_eq!(comp_tdd[0].0, "/skill tdd-workflow ");

    // /skill security -> includes /skill security-review
    let comp_sec = ReplEditor::get_completions("/skill security");
    let names: Vec<String> = comp_sec.into_iter().map(|(s, _)| s).collect();
    assert!(names.contains(&"/skill security-review ".to_string()));
}

#[test]
fn test_agy_tab_completion_vella_subcommands() {
    let comp = ReplEditor::get_completions("/vella es");
    assert_eq!(comp.len(), 1);
    assert_eq!(comp[0].0, "/vella estop ");

    let comp_st = ReplEditor::get_completions("/vella st");
    assert_eq!(comp_st.len(), 1);
    assert_eq!(comp_st[0].0, "/vella status ");
}

#[test]
fn test_agy_keyboard_history_navigation_and_draft_preservation() {
    let mut editor = ReplEditor::with_history(vec![
        "/model gemini-2.0-flash".to_string(),
        "/agent architect".to_string(),
        "/plan refactor memory engine".to_string(),
    ]);

    assert_eq!(editor.history.len(), 3);
    assert_eq!(editor.history_index, 3);

    // Deduplicate adjacent
    editor.add_history("/plan refactor memory engine");
    assert_eq!(editor.history.len(), 3); // didn't add duplicate

    // Add new distinct command
    editor.add_history("/tools");
    assert_eq!(editor.history.len(), 4);
    assert_eq!(editor.history[3], "/tools");

    // Simulate draft preservation:
    let draft_prompt: Vec<char> = "my in-progress thought".chars().collect();
    editor.draft = draft_prompt.clone();

    // User navigates Up: index moves from 4 to 3 ("/tools")
    editor.history_index -= 1;
    assert_eq!(editor.history[editor.history_index], "/tools");

    // User navigates Up again: index moves to 2 ("/plan refactor memory engine")
    editor.history_index -= 1;
    assert_eq!(editor.history[editor.history_index], "/plan refactor memory engine");

    // User navigates Down: index moves to 3 ("/tools")
    editor.history_index += 1;
    assert_eq!(editor.history[editor.history_index], "/tools");

    // User navigates Down to bottom: draft is restored!
    editor.history_index += 1;
    assert_eq!(editor.history_index, 4);
    assert_eq!(editor.draft, draft_prompt);
}

#[test]
fn test_agy_kill_ring_and_yank() {
    let mut buffer: Vec<char> = "cargo test --release --all".chars().collect();

    // Kill to end from cursor at 11 (at '--')
    let cursor = 11;
    let kill_ring: String = buffer[cursor..].iter().collect();
    buffer.truncate(cursor);
    assert_eq!(kill_ring, "--release --all");
    assert_eq!(buffer.iter().collect::<String>(), "cargo test ");

    // Kill word backward (Ctrl+W)
    let word_start = ReplEditor::find_word_backward(&buffer, cursor);
    assert_eq!(word_start, 6);
    let killed_word: String = buffer[word_start..cursor].iter().collect();
    assert_eq!(killed_word.trim(), "test");
    buffer.drain(word_start..cursor);
    assert_eq!(buffer.iter().collect::<String>(), "cargo ");

    // Yank (paste) previously killed text (Ctrl+Y)
    for ch in kill_ring.chars() {
        buffer.push(ch);
    }
    assert_eq!(buffer.iter().collect::<String>(), "cargo --release --all");
}
