# Prerequesites:

- OS: Linux or MacOS
- TiUP: `curl --proto '=https' --tlsv1.2 -sSf https://tiup-mirrors.pingcap.com/install.sh | sh` and add `~/.tiup/bin` to your PATH
- Install my [surreal db server](https://github.com/MarlinZapp/wishes-surreal-server)

# Installation (Linux)
1. Install TiKV: `curl --proto '=https' --tlsv1.2 -sSf https://tiup-mirrors.pingcap.com/install.sh | sh`
2. Re-source the environment, f.e.: `source ~/.bashrc`

# Start the development environment
1. Start TiKV: `tiup playground --tag surrealdb --mode tikv-slim --pd 1 --kv 1`
2. Start the surreal db server using `cargo run` or a release binary

Now you can connect to [http://localhost:8080](http://localhost:8080) to see available endpoints.
