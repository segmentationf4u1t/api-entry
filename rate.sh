#!/bin/bash

# Configuration
API_URL="http://localhost:8080/api/rate-test"  # Updated to use the new rate-test endpoint
REQUESTS=100000  # Total number of requests to send
CONCURRENT=1000  # Number of concurrent requests
TIMEOUT=1  # Timeout for each request in seconds

# Function to send a single request
send_request() {
    curl -s -o /dev/null -w "%{http_code}\n" -m $TIMEOUT "$API_URL"
}

# Function to send requests concurrently
send_concurrent_requests() {
    for i in $(seq 1 $CONCURRENT); do
        send_request &
    done
    wait
}

# Main test loop
echo "Starting rate limiter test..."
echo "Sending $REQUESTS requests to $API_URL"
echo "Concurrent requests: $CONCURRENT"
echo

success_count=0
failure_count=0

for i in $(seq 1 $((REQUESTS / CONCURRENT))); do
    echo "Batch $i:"
    results=$(send_concurrent_requests)
    
    success=$(echo "$results" | grep -c "^200$")
    failure=$(echo "$results" | grep -c -v "^200$")
    
    success_count=$((success_count + success))
    failure_count=$((failure_count + failure))
    
    echo "  Successful requests: $success"
    echo "  Failed requests: $failure"
    echo
    
    sleep 1  # Short pause between batches
done

echo "Test completed."
echo "Total successful requests: $success_count"
echo "Total failed requests: $failure_count"

if [ $failure_count -eq 0 ]; then
    echo "Rate limiter may not be working as expected. All requests succeeded."
elif [ $success_count -eq 0 ]; then
    echo "Rate limiter may be too strict. All requests failed."
else
    echo "Rate limiter appears to be working. Some requests succeeded, some failed."
fi