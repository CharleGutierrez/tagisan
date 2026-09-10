#!/usr/bin/env bash
set -euo pipefail

API="https://www.modelscope.cn/openapi/v1/models"

curl -sS "$API?page_number=1&page_size=20" \
  -H "Authorization: Bearer ${MODELSCOPE_API_TOKEN:-}" \
  -H "Accept: application/json"
