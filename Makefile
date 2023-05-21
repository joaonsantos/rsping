.PHONY: build
build:
	cargo build
	sudo setcap cap_net_raw=eip ${PWD}/target/debug/rsping
	