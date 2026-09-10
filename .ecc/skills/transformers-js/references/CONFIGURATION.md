# Configuration

```ts
import { env } from '@huggingface/transformers';

env.allowLocalModels = true;
env.allowRemoteModels = false; // set true only when needed
```

Recommended:

- Pin runtime version.
- Use local model path in production.
- Keep tokenizer/model files in same release artifact.
