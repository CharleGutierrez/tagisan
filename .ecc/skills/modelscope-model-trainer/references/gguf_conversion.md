# GGUF Conversion

Typical flow:

1. Export merged model weights.
2. Convert with a compatible `llama.cpp` conversion utility.
3. Validate quantized output with a short inference test set.
4. Upload GGUF artifacts as separate files in model repo.
