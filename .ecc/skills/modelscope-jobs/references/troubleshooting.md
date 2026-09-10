# Troubleshooting

## 401 Or Permission Errors

- Confirm `MODELSCOPE_API_TOKEN` is set.
- Verify token belongs to the owner namespace.
- Re-run `modelscope login --token ...`.

## Upload Fails Midway

- Retry with smaller folder batches.
- Exclude very large temporary files.
- Check local disk and network stability.

## Missing Output Files

- Ensure script writes to explicit output directory.
- Add final directory listing before upload.
- Validate glob patterns used in uploader.

## Slow Inference Throughput

- Lower batch size first to avoid OOM.
- Use multi-GPU runtime when model size demands it.
- Profile tokenizer and I/O overhead separately.
