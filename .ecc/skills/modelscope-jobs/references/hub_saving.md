# Saving Outputs To ModelScope Hub

Use `HubApi` for deterministic uploads.

## Recommended Layout

- `data/` for JSONL/parquet outputs
- `reports/` for markdown and metrics summaries
- `logs/` for optional execution traces

## Create Repo

```python
from modelscope.hub.api import HubApi

api = HubApi()
api.create_repo("your-org/your-dataset", repo_type="dataset", exist_ok=True, token="<token>")
```

## Upload Folder

```python
from modelscope.hub.api import HubApi

api = HubApi()
api.upload_folder(
    repo_id="your-org/your-dataset",
    folder_path="./outputs",
    path_in_repo="data",
    repo_type="dataset",
    token="<token>",
    commit_message="Upload generated dataset artifacts",
)
```

## Upload Single File

```python
api.upload_file(
    path_or_fileobj="./outputs/summary.json",
    path_in_repo="reports/summary.json",
    repo_id="your-org/your-dataset",
    repo_type="dataset",
    token="<token>",
    commit_message="Add workflow summary",
)
```
