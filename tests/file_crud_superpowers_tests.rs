use serde_json::json;
use std::fs;
use tagisan::ecc::{AgentShieldScanner, AgentShieldVerdict};
use tagisan::tools::builtin::{DeleteFileTool, EditFileTool, ListDirTool};
use tagisan::tools::{ToolHandler, ToolRegistry};

#[tokio::test]
async fn test_edit_file_basic_and_backup() {
    let temp_dir = std::env::temp_dir().join(format!("tgs_test_edit_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let test_file = temp_dir.join("sample.txt");
    fs::write(&test_file, "Hello, World!\nThis is Tagisan.\nWelcome to Rust!\n").unwrap();

    let tool = EditFileTool::new();
    let args = json!({
        "path": test_file.to_str().unwrap(),
        "target_content": "Tagisan",
        "replacement_content": "Tagisan Engine v0.2",
        "create_backup": true
    });

    let result = tool.execute(args).await;
    assert!(result.is_ok(), "Edit should succeed: {:?}", result.err());
    let res_msg = result.unwrap();
    assert!(res_msg.contains("Successfully edited file"));
    assert!(res_msg.contains("1 occurrence(s)"));

    // Check modified file content
    let updated = fs::read_to_string(&test_file).unwrap();
    assert_eq!(
        updated,
        "Hello, World!\nThis is Tagisan Engine v0.2.\nWelcome to Rust!\n"
    );

    // Check backup file
    let bak_file = temp_dir.join("sample.txt.bak");
    assert!(bak_file.exists(), "Backup file sample.txt.bak must exist");
    let bak_content = fs::read_to_string(&bak_file).unwrap();
    assert_eq!(
        bak_content,
        "Hello, World!\nThis is Tagisan.\nWelcome to Rust!\n"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_edit_file_multiple_occurrences() {
    let temp_dir = std::env::temp_dir().join(format!("tgs_test_edit_multi_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let test_file = temp_dir.join("multi.txt");
    fs::write(&test_file, "foo bar foo baz foo").unwrap();

    let tool = EditFileTool::new();

    // Default allow_multiple: false -> should error on multiple occurrences
    let args_fail = json!({
        "path": test_file.to_str().unwrap(),
        "target_content": "foo",
        "replacement_content": "qux",
        "allow_multiple": false
    });
    let err_res = tool.execute(args_fail).await;
    assert!(err_res.is_err(), "Should error when multiple matches exist and allow_multiple is false");
    let err_str = err_res.unwrap_err().to_string();
    assert!(err_str.contains("found 3 times"));

    // allow_multiple: true -> should replace all occurrences
    let args_ok = json!({
        "path": test_file.to_str().unwrap(),
        "target_content": "foo",
        "replacement_content": "qux",
        "allow_multiple": true,
        "create_backup": false
    });
    let ok_res = tool.execute(args_ok).await;
    assert!(ok_res.is_ok(), "Multiple replacement should succeed");
    let updated = fs::read_to_string(&test_file).unwrap();
    assert_eq!(updated, "qux bar qux baz qux");

    // Check backup was disabled
    let bak_file = temp_dir.join("multi.txt.bak");
    assert!(!bak_file.exists(), "Backup should not be created when create_backup is false");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_edit_file_scoped_lines() {
    let temp_dir = std::env::temp_dir().join(format!("tgs_test_edit_scope_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let test_file = temp_dir.join("scoped.txt");
    let initial = "line 1: alpha\nline 2: target\nline 3: target\nline 4: beta\n";
    fs::write(&test_file, initial).unwrap();

    let tool = EditFileTool::new();

    // Scope search to lines 2..=2: target should be unique on line 2
    let args_scoped = json!({
        "path": test_file.to_str().unwrap(),
        "target_content": "target",
        "replacement_content": "REPLACED_ONCE",
        "start_line": 2,
        "end_line": 2,
        "allow_multiple": false
    });

    let res = tool.execute(args_scoped).await;
    assert!(res.is_ok(), "Scoped edit on line 2 must succeed: {:?}", res.err());

    let updated = fs::read_to_string(&test_file).unwrap();
    assert_eq!(
        updated,
        "line 1: alpha\nline 2: REPLACED_ONCE\nline 3: target\nline 4: beta\n"
    );

    // If target not in scope, must return error
    let args_out_of_scope = json!({
        "path": test_file.to_str().unwrap(),
        "target_content": "alpha",
        "replacement_content": "omega",
        "start_line": 3,
        "end_line": 4
    });
    let fail_res = tool.execute(args_out_of_scope).await;
    assert!(fail_res.is_err(), "Target outside scoped lines must error");
    assert!(fail_res.unwrap_err().to_string().contains("not found within lines"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_delete_file_trash_and_permanent() {
    let temp_dir = std::env::temp_dir().join(format!("tgs_test_delete_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let file_to_trash = temp_dir.join("trash_me.txt");
    fs::write(&file_to_trash, "Delete this safely into trash").unwrap();

    let tool = DeleteFileTool::new().with_working_dir(&temp_dir);

    // 1. Delete into trash (trash: true default)
    let args_trash = json!({
        "path": "trash_me.txt",
        "trash": true
    });
    let res = tool.execute(args_trash).await;
    assert!(res.is_ok(), "Trash deletion must succeed: {:?}", res.err());
    assert!(!file_to_trash.exists(), "Original file must no longer exist at original path");

    let trash_dir = temp_dir.join(".tagisan").join("trash");
    assert!(trash_dir.exists(), "Trash folder must be created");
    let trash_entries: Vec<_> = fs::read_dir(&trash_dir).unwrap().collect();
    assert_eq!(trash_entries.len(), 1, "Exactly one file should be in trash");
    let trashed_name = trash_entries[0].as_ref().unwrap().file_name().to_string_lossy().to_string();
    assert!(trashed_name.ends_with("_trash_me.txt"), "Trash filename must follow timestamp_filename format: {}", trashed_name);

    // 2. Permanent deletion
    let file_perm = temp_dir.join("perm_delete.txt");
    fs::write(&file_perm, "Permanently destroy this file").unwrap();

    let args_perm = json!({
        "path": "perm_delete.txt",
        "trash": false
    });
    let res_perm = tool.execute(args_perm).await;
    assert!(res_perm.is_ok(), "Permanent file deletion must succeed");
    assert!(!file_perm.exists(), "File must be permanently unlinked");

    // 3. Directory deletion without recursive must fail
    let sub_dir = temp_dir.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("inner.txt"), "inside").unwrap();

    let args_dir_no_rec = json!({
        "path": "subdir",
        "recursive": false
    });
    let fail_dir = tool.execute(args_dir_no_rec).await;
    assert!(fail_dir.is_err(), "Deleting directory without recursive: true must fail");

    // 4. Directory deletion with recursive: true
    let args_dir_rec = json!({
        "path": "subdir",
        "recursive": true,
        "trash": false
    });
    let ok_dir = tool.execute(args_dir_rec).await;
    assert!(ok_dir.is_ok(), "Recursive dir deletion must succeed");
    assert!(!sub_dir.exists(), "Subdir must be deleted");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_list_dir_structure_and_filters() {
    let temp_dir = std::env::temp_dir().join(format!("tgs_test_list_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // Create a hierarchy:
    // temp_dir/
    //   file1.txt
    //   file2.rs
    //   .hidden_file
    //   sub/
    //     nested.rs
    //     .hidden_sub/
    fs::write(temp_dir.join("file1.txt"), "hello world").unwrap();
    fs::write(temp_dir.join("file2.rs"), "fn main() {}").unwrap();
    fs::write(temp_dir.join(".hidden_file"), "secret").unwrap();

    let sub_dir = temp_dir.join("sub");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("nested.rs"), "pub fn nested() {}").unwrap();

    let hidden_sub = temp_dir.join(".hidden_sub");
    fs::create_dir_all(&hidden_sub).unwrap();
    fs::write(hidden_sub.join("stealth.txt"), "shh").unwrap();

    let tool = ListDirTool::new().with_working_dir(&temp_dir);

    // 1. max_depth: 1, include_hidden: false (default)
    let res1 = tool.execute(json!({ "path": "." })).await.unwrap();
    assert!(res1.contains("file1.txt"));
    assert!(res1.contains("file2.rs"));
    assert!(res1.contains("sub/"));
    assert!(!res1.contains(".hidden_file"), "Dotfiles must not be shown when include_hidden is false");
    assert!(!res1.contains(".hidden_sub"), "Dotdirs must not be shown when include_hidden is false");
    assert!(!res1.contains("nested.rs"), "nested.rs is at depth 2, must not be shown at max_depth 1");

    // 2. max_depth: 2, include_hidden: true
    let res2 = tool.execute(json!({
        "path": ".",
        "max_depth": 2,
        "include_hidden": true
    })).await.unwrap();
    assert!(res2.contains("file1.txt"));
    assert!(res2.contains(".hidden_file"));
    assert!(res2.contains("sub/nested.rs"));
    assert!(res2.contains(".hidden_sub/"));

    // 3. pattern filter: ".rs"
    let res3 = tool.execute(json!({
        "path": ".",
        "max_depth": 2,
        "pattern": ".rs"
    })).await.unwrap();
    assert!(res3.contains("file2.rs"));
    assert!(res3.contains("sub/nested.rs"));
    assert!(!res3.contains("file1.txt"), "file1.txt does not match pattern .rs");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_tool_registry_and_agentshield_integration() {
    // Verify tools registered in ToolRegistry::with_builtins()
    let reg = ToolRegistry::with_builtins();
    assert!(reg.contains("read_file"));
    assert!(reg.contains("write_file"));
    assert!(reg.contains("edit_file"), "ToolRegistry::with_builtins must contain 'edit_file'");
    assert!(reg.contains("delete_file"), "ToolRegistry::with_builtins must contain 'delete_file'");
    assert!(reg.contains("list_dir"), "ToolRegistry::with_builtins must contain 'list_dir'");

    // Verify AgentShield blocks malicious paths on all file tools
    for tool_name in &["read_file", "write_file", "edit_file", "delete_file", "list_dir"] {
        let bad_args = json!({ "path": "/etc/shadow" });
        let verdict = AgentShieldScanner::scan_tool_call(tool_name, &bad_args);
        match verdict {
            AgentShieldVerdict::Block { threat_level: _, reason } => {
                assert!(reason.contains("/etc/shadow") || reason.contains("System path"));
            }
            _ => panic!("AgentShield must block sensitive system path on '{}'", tool_name),
        }

        let good_args = json!({ "path": "./src/main.rs" });
        let safe_verdict = AgentShieldScanner::scan_tool_call(tool_name, &good_args);
        assert!(
            matches!(safe_verdict, AgentShieldVerdict::Allow),
            "AgentShield must allow safe paths on '{}'",
            tool_name
        );
    }
}
