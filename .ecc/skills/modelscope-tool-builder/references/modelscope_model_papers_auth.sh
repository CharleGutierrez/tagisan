#!/usr/bin/env bash
set -euo pipefail

MODEL_ID="${1:?usage: $0 <namespace/model-name>}"

curl -sS "https://www.modelscope.cn/models/${MODEL_ID}" \
  -H "Authorization: Bearer ${MODELSCOPE_API_TOKEN:-}" \
  -H "Accept: application/json"
