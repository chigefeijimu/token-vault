#!/bin/bash
# TokenVault Functional Test Suite
# Tests core wallet functionality

echo "============================================"
echo "TokenVault Functional Test Suite"
echo "============================================"
echo ""

cd ~/token-vault

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0

pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    PASS=$((PASS + 1))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    FAIL=$((FAIL + 1))
}

info() {
    echo -e "${YELLOW}ℹ INFO${NC}: $1"
}

# ============================================
# Test 1: Build Verification
# ============================================
echo ""
echo "=== Test 1: Build Verification ==="

info "Running cargo check..."
if cargo check --manifest-path src-tauri/Cargo.toml > /dev/null 2>&1; then
    pass "Rust backend compiles successfully"
else
    fail "Rust backend failed to compile"
fi

info "Running npm build..."
if npm run build > /dev/null 2>&1; then
    pass "Frontend builds successfully"
else
    fail "Frontend failed to build"
fi

info "Running cargo test..."
TEST_OUTPUT=$(cargo test --lib --manifest-path src-tauri/Cargo.toml 2>&1)
if echo "$TEST_OUTPUT" | grep -q "test result: ok"; then
    pass "All unit tests pass"
else
    fail "Some unit tests failed"
fi

# ============================================
# Test 2: Rust Backend Commands (Unit Test)
# ============================================
echo ""
echo "=== Test 2: Backend Command Verification ==="

# Check all Tauri commands are registered
info "Checking Tauri command registrations..."
CMD_COUNT=$(grep -rh "#\[tauri::command\]" src-tauri/src/*.rs 2>/dev/null | wc -l)
info "Found $CMD_COUNT Tauri commands registered"
if [ "$CMD_COUNT" -gt 50 ]; then
    pass "Tauri commands properly registered ($CMD_COUNT commands)"
else
    fail "Expected >50 commands, found $CMD_COUNT"
fi

# ============================================
# Test 3: Key Module Verification
# ============================================
echo ""
echo "=== Test 3: Key Module Verification ==="

MODULES=("wallet" "rpc" "transaction" "crypto" "erc20" "storage" "security")
for mod in "${MODULES[@]}"; do
    if [ -f "src-tauri/src/$mod.rs" ]; then
        pass "Module exists: $mod.rs"
    else
        fail "Missing module: $mod.rs"
    fi
done

# ============================================
# Test 4: Frontend Page Verification
# ============================================
echo ""
echo "=== Test 4: Frontend Page Verification ==="

PAGES=("Dashboard" "CreateWallet" "ImportWallet" "WalletDashboard" "SendTransfer" "Settings" "TokenManagement" "WalletConnect")
for page in "${PAGES[@]}"; do
    if [ -f "src/pages/${page}.tsx" ]; then
        pass "Page exists: $page.tsx"
    else
        fail "Missing page: $page.tsx"
    fi
done

# ============================================
# Test 5: RPC Chain Config Verification
# ============================================
echo ""
echo "=== Test 5: Chain Configuration ==="

# Check chain configs exist in rpc.rs
CHAINS=("ethereum" "binance" "polygon" "arbitrum" "optimism" "avalanche")
CHAIN_NAMES=("Ethereum" "Binance/BSC" "Polygon" "Arbitrum" "Optimism" "Avalanche")
i=0
for chain in "${CHAINS[@]}"; do
    chain_name="${CHAIN_NAMES[$i]}"
    if grep -qi "$chain" src-tauri/src/rpc.rs; then
        pass "Chain configured: $chain_name"
    else
        fail "Missing chain config: $chain_name"
    fi
    i=$((i + 1))
done

# ============================================
# Test 6: Security Features
# ============================================
echo ""
echo "=== Test 6: Security Features ==="

SECURITY_CMDS=("setup_pin_code" "verify_pin_code" "lock_app" "unlock_app" "get_auth_state")
for cmd in "${SECURITY_CMDS[@]}"; do
    if grep -q "$cmd" src-tauri/src/lib.rs; then
        pass "Security command: $cmd"
    else
        fail "Missing security command: $cmd"
    fi
done

# ============================================
# Test 7: Transaction Features
# ============================================
echo ""
echo "=== Test 7: Transaction Features ==="

TX_CMDS=("send_transaction" "send_erc20_token" "get_transaction_receipt" "sign_data" "estimate_gas")
for cmd in "${TX_CMDS[@]}"; do
    if grep -q "$cmd" src-tauri/src/lib.rs; then
        pass "Transaction command: $cmd"
    else
        fail "Missing transaction command: $cmd"
    fi
done

# ============================================
# Summary
# ============================================
echo ""
echo "============================================"
echo "Test Summary"
echo "============================================"
echo -e "${GREEN}Passed: $PASS${NC}"
echo -e "${RED}Failed: $FAIL${NC}"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
