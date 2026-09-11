fn main() -> Result<(), Box<dyn std::error::Error>> {
    const STACK_SIZE: usize = 8 * 1024 * 1024;
    let child = std::thread::Builder::new()
        .name("tagisan-main".to_string())
        .stack_size(STACK_SIZE)
        .spawn(|| -> Result<(), String> {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(STACK_SIZE)
                .build()
                .map_err(|e| e.to_string())?;
            rt.block_on(tagisan::cli::run())
                .map_err(|e| e.to_string())
        })?;
    child
        .join()
        .map_err(|_| "Main thread panicked")?
        .map_err(|e| e.into())
}
