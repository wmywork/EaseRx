# Set shell for Windows OSs:
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# list all commands
default:
  @just --list --unsorted

_test:
    cargo test -p easerx

_test-output:
    cargo test -p easerx -- --show-output

_test-single-thread:
    cargo test -p easerx -- --test-threads=1

_doc:
    cargo doc -p easerx --no-deps --open

_cov:
    cargo llvm-cov -p easerx --html

# demo ratatui
ratatui:
    cargo run -p demo_ratatui