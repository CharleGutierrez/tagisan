# Pipeline Options

```ts
import { pipeline } from '@huggingface/transformers';

const generator = await pipeline('text-generation', './models/local-gen', {
  dtype: 'fp32',
});
```

Common options:

- `max_new_tokens`
- `temperature`
- `top_p`
- `do_sample`
