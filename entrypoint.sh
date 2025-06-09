#!/bin/bash
set -e

echo "Starting entrypoint script..."

# Print all environment variables (excluding secrets)
echo "Environment variables:"
env | grep -v "PASSWORD\|SECRET\|KEY"

# Test database connection with netstat
echo "Checking for listening PostgreSQL port..."
netstat -tuln | grep 5432 || true

# Test database connection
echo "Testing database connection..."
max_attempts=30
attempt=0

while [ $attempt -lt $max_attempts ]; do
    if pg_isready -h localhost -p 5432 -U postgres; then
        echo "Database is ready!"
        break
    fi
    
    attempt=$((attempt + 1))
    echo "Waiting for database... (Attempt $attempt of $max_attempts)"
    
    # More detailed connection debugging
    echo "TCP port status:"
    netstat -tuln | grep 5432 || true
    
    echo "Testing connection with nc:"
    nc -zv localhost 5432 2>&1 || true
    
    sleep 2
done

if [ $attempt -eq $max_attempts ]; then
    echo "Failed to connect to database after $max_attempts attempts"
    echo "Final connection debugging:"
    netstat -tuln
    pg_isready -h localhost -p 5432 -U postgres -d materials || true
    exit 1
fi

echo "Starting application..."
exec /app/ensenas-service
