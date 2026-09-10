# Cache

- Node.js: cache model artifacts on disk between runs.
- Browser: rely on IndexedDB/Cache storage where available.
- Prefer explicit local model directories for reproducibility.

```ts
import { env } from '@huggingface/transformers';
env.allowLocalModels = true;
```
