# Local Vision Runtime Requirements

- Prefer Linux with an NVIDIA GPU and a working CUDA + PyTorch stack.
- Treat macOS as config-only by default.
- Confirm the configured Python interpreter is available on the target machine.
- Confirm the configured local vision training entrypoint exists before execution.
- Confirm dataset directories, annotation files, and image roots are reachable from the target host.
- Prefer one generated launch script and two env files over ad hoc shell commands.
