nix-shell

rustc src/main.rs -o /tmp/dot && /tmp/dot | dot -Tx11

cargo run
./target/debug/lb4 | dot -Tx11
