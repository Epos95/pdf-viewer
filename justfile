default:
	@just --list

tui command:
	cargo {{command}} --bin pdf-tui

server command:
	cargo {{command}} --bin pdf-viewer
