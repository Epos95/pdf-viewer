default:
	@just --list

# Start the TUI app
tui command +ARGS="":
	cargo {{command}} --bin pdf-tui -- {{ARGS}}

# Start the server
server command +ARGS="":
	cargo {{command}} --bin pdf-viewer -- {{ARGS}}

# Run the server in a docker image
docker_run:

# Create a docker image
docker_build:
	docker build -t pdf-viewer .
