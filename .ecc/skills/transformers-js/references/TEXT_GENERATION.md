# Text Generation

```ts
import { pipeline } from '@huggingface/transformers';

const gen = await pipeline('text-generation', './models/local-gen');
const out = await gen('Write a concise deployment checklist:', {
  max_new_tokens: 120,
  temperature: 0.7,
});
console.log(out);
```
