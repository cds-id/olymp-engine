#!/bin/bash
echo "Testing direct connection..."
docker-compose exec -T postgres psql -U blurp -d blurp -c "SELECT 1" && echo "✓ DB works"
echo ""
echo "Testing from host..."
timeout 5 nc -zv 127.0.0.1 5433 && echo "✓ Port open" || echo "✗ Port closed"
