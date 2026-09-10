#!/usr/bin/env -S node

const token = process.env.MODELSCOPE_API_TOKEN;
const url = new URL("https://www.modelscope.cn/openapi/v1/models");
url.searchParams.set("page_number", "1");
url.searchParams.set("page_size", "20");

const headers = token ? { Authorization: `Bearer ${token}` } : {};

fetch(url, { headers })
  .then((res) => {
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  })
  .then((data) => {
    process.stdout.write(JSON.stringify(data, null, 2) + "\n");
  })
  .catch((err) => {
    process.stderr.write(String(err) + "\n");
    process.exit(1);
  });
