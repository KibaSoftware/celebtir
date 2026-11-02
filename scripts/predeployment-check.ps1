#Requires -Version 5.1
param([ValidateSet('Rust','Site','Browser')][string]$Group, [string[]]$Only)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
Set-Location $repo
$out = Join-Path $repo 'output\predeployment'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$env:CARGO_TARGET_DIR = Join-Path $repo 'target\review'
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
$env:CARGO_BUILD_JOBS = '1'
$env:CELEBTIR_DATA = Join-Path $repo 'output\qa-v02-data'
$env:CELEBTIR_SITE_URL = 'https://celebtir.com'
$env:PATH = 'C:\Program Files\Git\bin;C:\Program Files\Git\usr\bin;' + $env:PATH
$results = @()
if ($Only -and (Test-Path (Join-Path $out ($Group.ToLowerInvariant() + '-results.json')))) {
    $results = @(Get-Content -Raw (Join-Path $out ($Group.ToLowerInvariant() + '-results.json')) | ConvertFrom-Json)
}
function Check([string]$Name, [string]$Program, [string[]]$Arguments) {
    if ($Only -and $Name -notin $Only) { return }
    $started = [DateTime]::UtcNow
    Write-Host "START $Name"
    $log = Join-Path $out ($Name + '.log')
    if (Test-Path -LiteralPath $log) { Copy-Item -LiteralPath $log -Destination ($log + '.previous') -Force }
    # Native stderr is diagnostic text, not a PowerShell terminating error.
    $ErrorActionPreference = 'Continue'
    & $Program @Arguments > $log 2>&1
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    $script:results = @($script:results | Where-Object { $_.name -ne $Name }) + [pscustomobject]@{name=$Name;exitCode=$code;startedAt=$started.ToString('o');seconds=([DateTime]::UtcNow-$started).TotalSeconds;log=$log}
    $script:results | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $out ($Group.ToLowerInvariant() + '-results.json'))
    Write-Host "END $Name exit=$code"
}
if ($Group -eq 'Rust') {
    Check 'rust-fmt' 'cargo.exe' @('+1.88.0','fmt','--all','--check')
    Check 'rust-clippy' 'cargo.exe' @('+1.88.0','clippy','--locked','--workspace','--all-targets','--','-D','warnings')
    Check 'rust-tests' 'cargo.exe' @('+1.88.0','test','--locked','--workspace')
    Check 'rust-release' 'cargo.exe' @('+1.88.0','build','--locked','--release','-p','celebtir')
    Check 'rust-wasm' 'cargo.exe' @('+1.88.0','build','--locked','-p','celebtir-score','--target','wasm32-unknown-unknown')
    Check 'trace' 'C:\Program Files\Git\bin\bash.exe' @('scripts/trace.sh')
    Check 'trace-regression' 'C:\Program Files\Git\bin\bash.exe' @('scripts/tests/trace_test.sh')
    if (Test-Path -LiteralPath 'target\review\release\celebtir.exe') {
        Copy-Item -LiteralPath 'target\review\release\celebtir.exe' -Destination (Join-Path $out 'celebtir.exe')
        Get-FileHash -LiteralPath (Join-Path $out 'celebtir.exe') -Algorithm SHA256 | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out 'binary-hash.json')
    }
} elseif ($Group -eq 'Site') {
    Check 'site-lint' 'pnpm.cmd' @('-C','site','lint')
    Check 'site-unit' 'pnpm.cmd' @('-C','site','test')
    Check 'site-build' 'pnpm.cmd' @('-C','site','build')
    Check 'site-typecheck' 'pnpm.cmd' @('-C','site','typecheck')
    Check 'site-budget' 'pnpm.cmd' @('-C','site','budget')
} else {
    $env:CI = '1'
    $env:CELEBTIR_TEST_PORT = '13211'
    $env:CELEBTIR_REVIEW_SCREENSHOTS = Join-Path $repo 'output\playwright\predeployment'
    Check 'site-browser' 'pnpm.cmd' @('-C','site','exec','playwright','test','--retries=0')
}
if (@($results | Where-Object { $_.exitCode -ne 0 }).Count) { exit 1 }
