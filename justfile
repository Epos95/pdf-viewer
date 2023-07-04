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
	docker run --name pdf-viewer -d -p 3000:3000 -v state:/state_dir -v "$(pwd)"/content/:/content pdf-viewer

# Create a docker image
docker_build:
	docker build -t pdf-viewer .
