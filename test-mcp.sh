#!/bin/bash
# dm-mcp verification script
# Run this to verify MCP protocol works before publishing

set -e
cd "$(dirname "$0")"

echo "🔧 Building dm-mcp..."
cargo build --release 2>&1 | tail -3

echo ""
echo "📡 Testing MCP protocol..."

# Test initialize
INIT_RESULT=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | timeout 5 ./target/release/dm-mcp 2>/dev/null)

if echo "$INIT_RESULT" | grep -q '"result"'; then
    echo "✅ MCP initialize: PASS"
else
    echo "❌ MCP initialize: FAIL"
    exit 1
fi

# Test tools/list
TOOLS_RESULT=$(echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | timeout 5 ./target/release/dm-mcp 2>/dev/null)
TOOL_COUNT=$(echo "$TOOLS_RESULT" | jq '.result.tools | length' 2>/dev/null || echo "0")

if [ "$TOOL_COUNT" -eq "17" ]; then
    echo "✅ tools/list: PASS (17 tools)"
else
    echo "❌ tools/list: FAIL (expected 17, got $TOOL_COUNT)"
    exit 1
fi

# Test with real .dme file if available
if [ -f "../NewKV/New NKV.dme" ]; then
    echo ""
    echo "🎮 Testing with NewKV project..."
    
    PARSE_RESULT=$({
        echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
        sleep 0.1
        echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"dm_parse_environment","arguments":{"dme_path":"'"$(realpath ../NewKV/New\ NKV.dme)"'"}}}'
    } | timeout 30 ./target/release/dm-mcp 2>/dev/null | tail -1)
    
    if echo "$PARSE_RESULT" | grep -q "Successfully parsed"; then
        TYPE_COUNT=$(echo "$PARSE_RESULT" | grep -oP 'Total types: \K\d+' || echo "0")
        echo "✅ dm_parse_environment: PASS ($TYPE_COUNT types)"
    else
        echo "⚠️ dm_parse_environment: Could not parse (may need fixes)"
    fi
fi

echo ""
echo "═══════════════════════════════════════"
echo "✨ dm-mcp verification complete!"
echo "═══════════════════════════════════════"
