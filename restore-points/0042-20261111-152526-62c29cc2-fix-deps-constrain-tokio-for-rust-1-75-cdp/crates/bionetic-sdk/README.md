# bionetic-sdk

Embeddable SDK for [NeuralBrain](https://github.com/pfn000/NueralBrain) —
drop a bionetic brain into any Rust application with a small, typed Rust API.

## Quick start

```rust
use bionetic_sdk::BrainBuilder;

let mut brain = BrainBuilder::new()
    .with_corpora(8)
    .with_dots_per_corpus(128)
    .with_dreaming(true)
    .build();

let output = brain.think(&[0.5, 0.8, 0.1]);
brain.learn(&[0.5, 0.8, 0.1], 0.9); // reward
brain.dream(100);                    // consolidate memories
```

## Features

| Feature         | Enables                              |
|-----------------|--------------------------------------|
| `native`        | Native Rust build (default)          |
| `wasm`          | WebAssembly bindings for browsers    |
| `multimodal`    | Image / audio / text input helpers   |
| `brain-network` | Brain-to-brain communication         |
| `full`          | Everything                           |

## Persistence

```rust
let bytes = brain.save();                  // serialize state
let mut other = BrainBuilder::new().build();
other.load(&bytes)?;                        // restore state
```