# Saving To ModelScope Hub

```bash
modelscope create your-name/your-model --repo_type model --visibility public
modelscope upload your-name/your-model ./outputs checkpoints --repo-type model
```

If `MODELSCOPE_API_TOKEN` is not configured, treat publishing as `skipped` and keep training as successful.

Python upload example:

```python
from modelscope.hub.api import HubApi

api = HubApi()
api.upload_folder(
    repo_id="your-name/your-model",
    folder_path="./outputs",
    path_in_repo="checkpoints",
    repo_type="model",
    token="<token>",
)
```
