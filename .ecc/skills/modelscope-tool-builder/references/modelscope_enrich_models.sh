#!/usr/bin/env bash
set -euo pipefail

# Enrich a local model list with ModelScope metadata.
# Input format: one repo id per line, e.g. "Qwen/Qwen2.5-7B-Instruct"

while IFS= read -r repo_id; do
  [[ -z "$repo_id" ]] && continue
  curl -sS "https://www.modelscope.cn/models/${repo_id}" \
    -H "Authorization: Bearer ${MODELSCOPE_API_TOKEN:-}" \
    -H "Accept: application/json"
  echo
  sleep 0.2
done
