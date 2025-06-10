#!/bin/bash
set -e

# Create a network if it doesn't exist
if ! podman network exists ensenas-network; then
    echo "Creating network..."
    podman network create ensenas-network \
        --driver bridge 
fi

# Check if the pod exists
if ! podman pod exists ensenas-pod; then
    echo "Creating pod..."
    podman pod create \
        --name ensenas-pod \
        --network ensenas-network \
        --share net \
        --publish 8080:8080 \
        --publish 5435:5432
else
    echo "Pod already exists, skipping creation."
fi

# Ensure the database volume exists
if ! podman volume exists ensenas-db-data; then
    echo "Creating database volume..."
    podman volume create ensenas-db-data
else
    echo "Database volume exists, keeping data."
fi

# Check if the database container exists
if ! podman container exists ensenas-db; then
    echo "Starting database..."
    podman run -d \
        --pod ensenas-pod \
        --name ensenas-db \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_PASSWORD=postgres \
        -e POSTGRES_DB=ensenas \
        -v ensenas-db-data:/var/lib/postgresql/data:Z \
        postgres:17
else
    echo "Database container already exists, ensuring it's running..."
    podman start ensenas-db
fi

# Wait for database to be ready
echo "Waiting for database to be ready..."
timeout=60
while [ $timeout -gt 0 ]; do
    if podman exec ensenas-db pg_isready -U postgres; then
        break
    fi
    echo "Database is not ready - waiting... ($timeout seconds left)"
    sleep 2
    timeout=$((timeout - 2))
done

if [ $timeout -le 0 ]; then
    echo "Timeout waiting for database to become ready"
    exit 1
fi

echo "Building service..."
podman build -t ensenas-service .

# Run ensenas-service with --replace
echo "Running new ensenas-service container..."
podman run -d --replace \
    --pod ensenas-pod \
    --name ensenas-service \
    --restart unless-stopped \
    --env-file .env \
    ensenas-service

# Follow the logs with timeout
echo "Service logs (showing first 30 seconds):"
timeout 30 podman logs -f ensenas-service || true

# Check container status
status=$(podman container inspect -f '{{.State.Status}}' ensenas-service)
if [ "$status" != "running" ]; then
    echo "Container is not running (status: $status)"
    echo "Last few logs:"
    podman logs --tail 50 ensenas-service
    
    # Add network debugging
    echo "Network connectivity ensenas from ensenas-service:"
    podman exec -it ensenas-service nc -zv localhost 5432 || true
    
    exit 1
fi

echo "Service is running successfully!"
