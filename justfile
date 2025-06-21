# Set shell for Windows OSs:

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# list all commands
default:
    @just --list --unsorted

_test:
    cargo test -p easerx --features "serde"

_test-output:
    cargo test -p easerx -- --show-output

_test-single-thread:
    cargo test -p easerx -- --test-threads=1

_doc:
    cargo doc -p easerx --no-deps --open

_cov:
    cargo llvm-cov -p easerx --html

_clippy:
    cargo clippy -p easerx

# basic1 state store
b1:
    cargo run -p basic1_state_store

# basic2 multiple state2
b2:
    cargo run -p basic2_multiple_states

# basic3 collections
b3:
    cargo run -p basic3_collections

# basic4 execute
b4:
    cargo run -p basic4_execute

# basic5 async execute
b5:
    cargo run -p basic5_async_execute

# extended1 order of nested
e1:
    cargo run -p extended1_order_of_nested

# extended2 execute_with_retain
e2:
    cargo run -p extended2_execute_with_retain

# extended3 execute_with_cancelable
e3:
    cargo run -p extended3_execute_with_cancelable

# extended4 execute_cancelable_with_retain
e4:
    cargo run -p extended4_execute_cancelable_with_retain

# extended5 execute_with_timeout
e5:
    cargo run -p extended5_execute_with_timeout

# extended6 async_execute_with_retain
e6:
    cargo run -p extended6_async_execute_with_retain

# extended7 async_execute_with_cancelable
e7:
    cargo run -p extended7_async_execute_with_cancelable

# extended8 async_execute_cancelable_with_retain
e8:
    cargo run -p extended8_async_execute_cancelable_with_retain

# extended9 async_execute_with_timeout
e9:
    cargo run -p extended9_async_execute_with_timeout

# extended10 execution without blocking
e10:
    cargo run -p extended10_execution_without_blocking

# demo simple_todo
todo:
    cargo run -p demo_simple_todo

# demo ratatui
ratatui:
    cargo run -p demo_ratatui

# demo cursive
cursive:
    cargo run -p demo_cursive