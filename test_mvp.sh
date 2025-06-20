#!/bin/bash

# Test script for wshowkeys_rs MVP
echo "🚀 Testing wshowkeys_rs MVP"
echo "========================================="
echo ""

# Check if we're in Wayland
if [ "$XDG_SESSION_TYPE" != "wayland" ]; then
    echo "❌ Not running in Wayland session (found: $XDG_SESSION_TYPE)"
    echo "This application requires Wayland"
    exit 1
fi

echo "✅ Wayland session detected"
echo ""

# Build the project
echo "🔨 Building project..."
if cargo build; then
    echo "✅ Build successful"
else
    echo "❌ Build failed"
    exit 1
fi
echo ""

echo "🎹 Starting wshowkeys_rs MVP test..."
echo "📝 Instructions:"
echo "   1. The app will start and show 'wshowkeys_rs started'"  
echo "   2. Press any keys to see 'Hello World' messages"
echo "   3. Press Ctrl+C to exit"
echo ""
echo "Starting in 3 seconds..."
sleep 3

# Run the application with debug output
exec cargo run -- --debug
