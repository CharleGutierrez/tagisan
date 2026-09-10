# Token Usage

## Environment Variable

```bash
export MODELSCOPE_API_TOKEN=<token>
```

## CLI Login

```bash
modelscope login --token "$MODELSCOPE_API_TOKEN"
```

## Python Login

```python
from modelscope.hub.api import HubApi

api = HubApi()
api.login("<token>")
```

## Security Rules

- Never commit tokens into git history.
- Do not print full token values in logs.
- Prefer runtime injection in CI/CD secret stores.
