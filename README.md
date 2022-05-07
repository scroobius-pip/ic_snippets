# ic_snippets

Demo project demonstrating [scaled_storage](https://crates.io/crates/scaled_storage) usage.


## How to run
Starts the canister and initializes Scaled Storage by uploading the wasm file.
`./deploy_dev.sh`

## How it works
Ic snippets is a solution for storing IC related code snippets. It allows snippets to be paginated by storing them in a Page. This is neccessary to allow the frontend not to load all snippets at once.

