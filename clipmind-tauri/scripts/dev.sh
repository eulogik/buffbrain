#!/bin/bash
# Start BuffBrain Tauri dev environment
# - Starts Vite dev server in background
# - Starts Tauri app in background
# - Both keep running until Ctrl+C

set -e
cd "$(dirname "$0")/.."

cleanup() {
    echo ""
    echo "Shutting down..."
    pkill -f buffbrain 2>/dev/null || true
    pkill -f "vite$" 2>/dev/null || true
    pkill -f "node.*vite" 2>/dev/null || true
    exit 0
}
trap cleanup INT TERM

# Start Vite
echo "Starting Vite dev server..."
caffeinate -i nohup npm run dev > /tmp/buffbrain-vite.log 2>&1 &
VITE_PID=$!
disown

# Wait for Vite
echo "Waiting for Vite on http://localhost:1420 ..."
for i in {1..30}; do
    if curl -s http://localhost:1420/ -m 2 -o /dev/null -w "%{http_code}" 2>/dev/null | grep -q "200"; then
        echo "Vite is up!"
        break
    fi
    sleep 0.5
done

# Start Tauri
echo "Starting Tauri app..."
caffeinate -i nohup ./src-tauri/target/debug/buffbrain > /tmp/buffbrain-app.log 2> /tmp/buffbrain-err.log &
APP_PID=$!
disown

echo ""
echo "BuffBrain running:"
echo "  Vite:   http://localhost:1420"
echo "  Tauri:  PID $APP_PID"
echo "  Logs:   /tmp/buffbrain-app.log"
echo ""
echo "Press Ctrl+C to stop."

# Keep script running
wait
