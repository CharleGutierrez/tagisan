# Local Runtime Requirements

- Prefer Linux with an NVIDIA GPU and a working CUDA + PyTorch stack.
- Treat macOS as config-only by default.
- Confirm `swift` is installed before any real execution.
- Confirm the dataset path is reachable from the target training host.
- Prefer preparing one launch script and one env file instead of executing ad hoc shell commands.
