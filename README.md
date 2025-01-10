# Setup testing environment

## Installation (Linux)
1. Install TiKV: `curl --proto '=https' --tlsv1.2 -sSf https://tiup-mirrors.pingcap.com/install.sh | sh`
2. Re-source the environment, f.e.: `source ~/.bashrc`
3. Install SurrealDB: `curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh`
4. Add SurrealDB to path: `export PATH=$HOME/.surrealdb:$PATH`

## Start the testing environment
1. Start TiKV: `tiup playground --tag surrealdb --mode tikv-slim --pd 1 --kv 1`
2. Start SurrealDB: `surreal start tikv://127.0.0.1:2379`
3. Start the release of my backend: `./target/release/backend`

Now you can connect to http://localhost:8080 to see available endpoints.
