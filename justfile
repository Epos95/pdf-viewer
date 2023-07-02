default:
	@just --list

tui command:
	cargo {{command}} --manifest-path pdf-tui/Cargo.toml

server command:
	cargo {{command}}
