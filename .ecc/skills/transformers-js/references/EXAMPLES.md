# Examples

## Node.js Classification

```ts
import { pipeline } from '@huggingface/transformers';

const clf = await pipeline('text-classification', './models/my-text-model');
console.log(await clf('Inference from local model files'));
```

## Browser Usage

```html
<script type="module">
  import { pipeline } from 'https://cdn.jsdelivr.net/npm/@huggingface/transformers';
  const classifier = await pipeline('text-classification', './models/browser-model');
  console.log(await classifier('hello'));
</script>
```
