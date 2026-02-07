# Comprehensive CLI Testing Script for AI Model Vault
# Tests all CLI commands with real operations

Write-Host "=== AI Model Vault CLI Comprehensive Testing ===" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"
$testResults = @()

# Build the CLI
Write-Host "Building CLI..." -ForegroundColor Yellow
cargo build --release --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Build successful" -ForegroundColor Green
Write-Host ""

$aim = ".\target\release\aim.exe"

# Test 1: Help command
Write-Host "Test 1: Help Command" -ForegroundColor Yellow
& $aim --help | Out-Null
$result = if ($LASTEXITCODE -eq 0) { "PASS" } else { "FAIL" }
$testResults += "Help command: $result"
Write-Host "  Result: $result" -ForegroundColor $(if ($result -eq "PASS") { "Green" } else { "Red" })
Write-Host ""

# Test 2: Stats command
Write-Host "Test 2: Stats Command" -ForegroundColor Yellow
& $aim stats 2>&1 | Out-Null
$result = if ($LASTEXITCODE -eq 0) { "PASS" } else { "FAIL" }
$testResults += "Stats command: $result"
Write-Host "  Result: $result" -ForegroundColor $(if ($result -eq "PASS") { "Green" } else { "Red" })
Write-Host ""

# Test 3: List command
Write-Host "Test 3: List Command (requires passphrase)" -ForegroundColor Yellow
Write-Host "  Note: Skipping interactive test" -ForegroundColor Gray
$testResults += "List command: SKIP (interactive)"
Write-Host ""

# Test 4: Compliance command
Write-Host "Test 4: Compliance Command" -ForegroundColor Yellow
& $aim compliance 2>&1 | Select-String "FIPS|CMMC" | Out-Null
$result = if ($LASTEXITCODE -eq 0) { "PASS" } else { "FAIL" }
$testResults += "Compliance command: $result"
Write-Host "  Result: $result" -ForegroundColor $(if ($result -eq "PASS") { "Green" } else { "Red" })
Write-Host ""

# Test 5: Cache info command
Write-Host "Test 5: Cache Info Command" -ForegroundColor Yellow
& $aim cache 2>&1 | Select-String "Cache" | Out-Null
$result = if ($LASTEXITCODE -eq 0) { "PASS" } else { "FAIL" }
$testResults += "Cache command: $result"
Write-Host "  Result: $result" -ForegroundColor $(if ($result -eq "PASS") { "Green" } else { "Red" })
Write-Host ""

# Test 6: Version flag
Write-Host "Test 6: Version Flag" -ForegroundColor Yellow
& $aim --version 2>&1 | Out-Null
$result = if ($LASTEXITCODE -eq 0) { "PASS" } else { "FAIL" }
$testResults += "Version flag: $result"
Write-Host "  Result: $result" -ForegroundColor $(if ($result -eq "PASS") { "Green" } else { "Red" })
Write-Host ""

# Summary
Write-Host ""
Write-Host "=== Test Summary ===" -ForegroundColor Cyan
$passed = ($testResults | Where-Object { $_ -like "*PASS*" }).Count
$failed = ($testResults | Where-Object { $_ -like "*FAIL*" }).Count
$skipped = ($testResults | Where-Object { $_ -like "*SKIP*" }).Count

foreach ($result in $testResults) {
    $color = if ($result -like "*PASS*") { "Green" } elseif ($result -like "*FAIL*") { "Red" } else { "Gray" }
    Write-Host "  $result" -ForegroundColor $color
}

Write-Host ""
Write-Host "Total Tests: $($testResults.Count)" -ForegroundColor Cyan
Write-Host "Passed: $passed" -ForegroundColor Green
Write-Host "Failed: $failed" -ForegroundColor Red
Write-Host "Skipped: $skipped" -ForegroundColor Gray

if ($failed -eq 0) {
    Write-Host ""
    Write-Host "✓ All non-interactive CLI tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host ""
    Write-Host "✗ Some CLI tests failed!" -ForegroundColor Red
    exit 1
}
