alias r := run
alias b := build
alias d := dev
alias i := install

run *args:
	cargo run -p clashctl -- {{ args }}

reset_terminal:
	pkill clashctl && stty sane && stty cooked

dev:
	cargo watch -x 'check -p clashctl > /dev/null 2>&1 ' -s 'touch .trigger' > /dev/null &
	cargo watch --no-gitignore -w .trigger -x 'run -p clashctl'

build:
	cargo build --release

install:
  cargo install --path ./clashctl

release os: build
	#!/usr/bin/env bash
	pushd target/release
	rm clashctl*.d
	mv clashctl-tui* clashctl-tui-{{ os }}
	mv clashctl* clashctl-{{ os }}
	popd

test *args:
	cargo test -- {{ args }} --nocapture
