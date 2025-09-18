#!/bin/bash

echo "🔍 Testing PR Status JSON Integration..."
echo "======================================="

CLI_PATH="./target/release/my-prs"

# Check if CLI tool exists
if [ ! -f "$CLI_PATH" ]; then
    echo "❌ CLI tool not found at: $CLI_PATH"
    echo "💡 Run 'cargo build --release' first"
    exit 1
fi

echo "✅ CLI tool found: $CLI_PATH"

# Test JSON output
echo ""
echo "📊 Testing JSON output:"
echo "Command: $CLI_PATH status-check --json"
echo "---------------------------------------"

JSON_OUTPUT=$($CLI_PATH status-check --json 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ JSON command executed successfully"
    echo ""
    echo "📋 JSON Output:"
    echo "$JSON_OUTPUT" | jq . 2>/dev/null || echo "$JSON_OUTPUT"
    echo ""
    
    # Parse key fields
    TOTAL_PRS=$(echo "$JSON_OUTPUT" | jq -r '.total_prs' 2>/dev/null)
    OVERALL_STATUS=$(echo "$JSON_OUTPUT" | jq -r '.overall_status' 2>/dev/null)
    FAILING_COUNT=$(echo "$JSON_OUTPUT" | jq -r '.failing_prs | length' 2>/dev/null)
    RUNNING_COUNT=$(echo "$JSON_OUTPUT" | jq -r '.running_prs | length' 2>/dev/null)
    PASSING_COUNT=$(echo "$JSON_OUTPUT" | jq -r '.passing_prs | length' 2>/dev/null)
    
    if [ "$TOTAL_PRS" != "null" ] && [ "$TOTAL_PRS" != "" ]; then
        echo "📈 Summary:"
        echo "   Total PRs: $TOTAL_PRS"
        echo "   Overall Status: $OVERALL_STATUS"
        echo "   🔴 Failing: $FAILING_COUNT"
        echo "   🟡 Running: $RUNNING_COUNT"  
        echo "   🟢 Passing: $PASSING_COUNT"
        echo ""
        
        # Determine menu bar icon
        case "$OVERALL_STATUS" in
            "failing") ICON="🔴" ;;
            "running") ICON="🟡" ;;
            "passing") ICON="🟢" ;;
            "no_prs") ICON="📭" ;;
            *) ICON="❓" ;;
        esac
        
        echo "🍎 Menu Bar Icon: $ICON"
        echo ""
        echo "✅ JSON integration ready for Swift app!"
        echo ""
        echo "🚀 Next Steps:"
        echo "   1. Open Xcode and create new macOS app project"
        echo "   2. Copy contents of PRStatusMenuBar/main.swift"
        echo "   3. Update CLI path to: $(pwd)/$CLI_PATH"
        echo "   4. Build and run!"
        
    else
        echo "⚠️  JSON output seems invalid - check format"
    fi
    
else
    echo "❌ CLI command failed with exit code: $EXIT_CODE"
    echo "Error output:"
    echo "$JSON_OUTPUT"
    echo ""
    echo "💡 Troubleshooting:"
    echo "   - Check if GitHub token is configured"
    echo "   - Verify repository access"
    echo "   - Run 'my config --show' to check settings"
fi

echo ""
echo "🔧 Available Commands:"
echo "   $CLI_PATH status-check --json    # Get JSON status"
echo "   $CLI_PATH status-check           # Human readable"
echo "   $CLI_PATH monitor --json         # JSON monitoring mode"
echo "   $CLI_PATH config --show          # Show configuration"
