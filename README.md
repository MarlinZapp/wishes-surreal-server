# Setup development environment

## Installation (Linux)
1. Install TiKV: `curl --proto '=https' --tlsv1.2 -sSf https://tiup-mirrors.pingcap.com/install.sh | sh`
2. Re-source the environment, f.e.: `source ~/.bashrc`
3. Install SurrealDB: `curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh`
4. Add SurrealDB to path: `export PATH=$HOME/.surrealdb:$PATH`
5. Install Rust (and with it cargo): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Start the development environment
1. Start TiKV: `tiup playground --tag surrealdb --mode tikv-slim --pd 1 --kv 1`
2. Start the backend which implements SurrealDB: `cargo run`

Now you can connect to [http://localhost:8080](http://localhost:8080) to see available endpoints.
