.PHONY: build
build:
	cargo build
	sudo setcap cap_net_raw=ep ${PWD}/target/debug/rsping
	