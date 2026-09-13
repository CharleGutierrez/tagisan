fn main() {
    const STACK_SIZE: usize = 8 * 1024 * 1024;
    let child = std::thread::Builder::new()
        .name("tagisan-main".to_string())
        .stack_size(STACK_SIZE)
        .spawn(|| -> Result<(), String> {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(STACK_SIZE)
                .build()
            {
                Ok(rt) => rt,
                Err(e) => return Err(e.to_string()),
            };
            rt.block_on(tagisan::cli::run()).map_err(|e| e.to_string())
        });

    let res = match child {
        Ok(handle) => match handle.join() {
            Ok(r) => r,
            Err(_) => {
                eprintln!("Error: Main thread panicked");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error: Failed to spawn thread: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = res {
        if e.contains("No local LLM models are installed") || e.contains("NoModelsInstalled") {
            // Notification was already printed cleanly to stderr
            std::process::exit(1);
        }
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
