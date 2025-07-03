#!/usr/bin/env nu
# podman.nu

# Exit on error by using 'try' with 'if' or '||' wherever appropriate.

# Create a network if it doesn't exist
if not ($"ensenas-network" in (podman network ls --format json | from json | get Name)) {
    print "Creating network..."
    podman network create ensenas-network --driver bridge
} else {
    print "Network already exists, skipping creation."
}

# Check if the pod exists
if not ($"ensenas-pod" in (podman pod ls --format json | from json | get Name)) {
    print "Creating pod..."
    podman pod create --name ensenas-pod --network ensenas-network --share net --publish 5001:8080 --publish 5435:5432
} else {
    print "Pod already exists, skipping creation."
}

# Ensure the database volume exists
if not ($"ensenas-db-data" in (podman volume ls --format json | from json | get Name)) {
    print "Creating database volume..."
    podman volume create ensenas-db-data
} else {
    print "Database volume exists, keeping data."
}

# Check if the database container exists
if not ($"ensenas-db" in (podman ps -a --format json | from json | get Names | flatten)) {
    print "Starting database..."
    (podman run -d
        --pod ensenas-pod
        --name ensenas-db
        -e POSTGRES_USER=postgres
        -e POSTGRES_PASSWORD=postgres
        -e POSTGRES_DB=ensenas
        -v ensenas-db-data:/var/lib/postgresql/data:Z
        postgres:17)
} else {
    print "Database container already exists, ensuring it's running..."
    podman start ensenas-db
}

# Wait for database to be ready
print "Waiting for database to be ready..."
mut timeout = 60
while $timeout > 0 {
    let isready = (podman exec ensenas-db pg_isready -U postgres | complete)
    if $isready.exit_code == 0 {
        break
    }
    print $"Database is not ready - waiting... ($timeout) seconds left"
    sleep 2sec
    $timeout = ($timeout) - 2
}
if $timeout <= 0 {
    print "Timeout waiting for database to become ready"
    exit 1
}

print "Building service..."
podman build -t ensenas-service .

# Run ensenas-service with --replace
print "Running new ensenas-service container..."
(podman run -d --replace
    --pod ensenas-pod
    --name ensenas-service
    --restart unless-stopped
    --env-file .env
    ensenas-service)

# Follow the logs with timeout
print "Service logs (showing first 30 seconds):"
podman logs -f ensenas-service | take 30

# Check container status
let status = (podman container inspect ensenas-service --format '{{.State.Status}}' | str trim)
if $status != "running" {
    print $"Container is not running (status: $status)"
    print "Last few logs:"
    podman logs --tail 50 ensenas-service

    # Add network debugging
    print "Network connectivity ensenas from ensenas-service:"
    podman exec -it ensenas-service nc -zv localhost 5432 or true

    exit 1
}

print "Service is running successfully!"
