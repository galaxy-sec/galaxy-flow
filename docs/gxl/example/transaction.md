# Transaction Example

This example demonstrates how to use transactions in GXL.

```rust
extern mod base { path = "./_gal/"; }

mod envs {
    env default {};
}

mod main {
    flow trans1 | step1 | step2 | base.base_step1 | step3;
    flow trans2 | step1 | step3 | step2;

    #[transaction, undo(_undo_step1)]
    flow step1 {
        gx.echo(" step1 ");
    }
    #[undo(_undo_step2)]
    flow step2 {
        gx.echo(" step2 ");
    }
    #[undo(_undo_step3)]
    flow step3 {
        gx.echo(" step3 ");
        gx.assert(value: "true", expect: "false");
    }

    flow _undo_step1 {
        gx.echo(" undo step1 ");
    }
    flow _undo_step2 {
        gx.echo(" undo step2 ");
    }
    flow _undo_step3 {
        gx.echo(" undo step3 ");
    }
}
```

```mermaid
graph TD
    A[Start] --> B[Load base module]
    B --> C[Load envs module]
    C --> D[Load main module]
    D --> E[Define flows]
    E --> F[Execute trans1 flow]
    F --> G[Execute step1 flow]
    G --> H[Execute step2 flow]
    H --> I[Execute base.base_step1 flow]
    I --> J[Execute step3 flow]
    J --> K[Execute trans2 flow]
    K --> L[Execute step1 flow]
    L --> M[Execute step3 flow]
    M --> N[Execute step2 flow]
    N --> O[End]
```