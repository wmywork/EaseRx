# Set shell for Windows OSs:
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# list all commands
default:
  @just --list --unsorted

test:
    cargo test -p easerx

test-output:
    cargo test -p easerx -- --show-output

test-single-thread:
    cargo test -p easerx -- --test-threads=1