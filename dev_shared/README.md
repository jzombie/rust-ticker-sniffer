# `dev_shared` – Development-Only Includes  

Instead of being a separate workspace crate, it is directly referenced using `#[path = "dev_shared/lib.rs"]`.

This is a workaround which allows `Cargo` to publish the main package.
