# Offline tests with synthetic credentials only; no network calls.
$ErrorActionPreference = 'Stop'
$repo = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
. (Join-Path $repo 'scripts/deploy/password-session.ps1')
$scratch = Join-Path $repo ('output/deployment-password/test-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $scratch
$session = $null
$original = @{}
foreach ($name in @('SSH_ASKPASS','SSH_ASKPASS_REQUIRE','CELEBTIR_PASSWORD_FILE','CELEBTIR_TEST_PASSWORD_HASH','CELEBTIR_TEST_FOLLOWUP','CELEBTIR_TEST_PEER_FAILURE','PATH')) { $original[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
$global:PasswordTestPrompts = 0
$global:PasswordTestValue = '  synthetic only $() ` " '' % ! & | < > spaces ' + [char]0x00E9 + [char]0x4E2D + '  '
function Read-Host {
    param([string]$Prompt, [switch]$AsSecureString)
    if (-not $AsSecureString) { throw 'Password prompt must be hidden' }
    $global:PasswordTestPrompts++
    ConvertTo-SecureString $global:PasswordTestValue -AsPlainText -Force
}
function Run-Helper([string]$Arguments, [string]$InputText = '') {
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $session.Helper; $start.Arguments = $Arguments
    $start.UseShellExecute = $false; $start.CreateNoWindow = $true
    $start.RedirectStandardInput = $true; $start.RedirectStandardOutput = $true; $start.RedirectStandardError = $true
    $start.StandardOutputEncoding = New-Object Text.UTF8Encoding($false)
    $process = [Diagnostics.Process]::Start($start)
    try {
        $process.StandardInput.Write($InputText); $process.StandardInput.Close()
        $output = $process.StandardOutput.ReadToEndAsync(); $errorText = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(15000)) { $process.Kill(); throw 'Helper test timed out' }
        [pscustomobject]@{ Code=$process.ExitCode; Output=$output.GetAwaiter().GetResult(); Error=$errorText.GetAwaiter().GetResult() }
    } finally { $process.Dispose() }
}
try {
    $env:SSH_ASKPASS = 'previous-askpass'; $env:SSH_ASKPASS_REQUIRE = 'never'; $env:CELEBTIR_PASSWORD_FILE = 'previous-secret-path'
    $session = Start-DeploymentPassword $scratch
    if ($global:PasswordTestPrompts -ne 1) { throw 'Expected one hidden prompt' }
    if ((Get-Acl $scratch).AreAccessRulesProtected -ne $true) { throw 'Credential directory must not inherit broad access' }
    if ((Get-Content $session.Secret -Raw) -match [regex]::Escape($global:PasswordTestValue)) { throw 'Credential was stored in plaintext' }
    foreach ($attempt in 1..3) {
        $result = Run-Helper '"jarvis@example.test''s password: "'
        if ($result.Code -ne 0 -or $result.Output.TrimEnd("`r", "`n") -cne $global:PasswordTestValue) { throw 'ASKPASS did not return the exact password' }
    }
    Write-Output 'PASS one prompt, DPAPI encryption, user-only directory and repeat askpass reuse (including Unicode/metacharacters)'
    foreach ($prompt in @('"Enter passphrase for key:"','"Are you sure you want to continue connecting (yes/no)?"','"Verification code:"')) {
        $result = Run-Helper $prompt
        if ($result.Code -eq 0 -or $result.Output) { throw 'Unrelated authentication prompt received password' }
    }
    Write-Output 'PASS private-key, host-confirmation and MFA prompts cannot receive the account password'
    $peer = Join-Path $scratch 'ssh.exe'
    & (Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe') -NoProfile -NonInteractive -ExecutionPolicy Bypass -File (Join-Path $repo 'scripts/deploy/build-password-helper.ps1') -Source (Join-Path $PSScriptRoot 'password-peer.cs') -Output $peer
    if ($LASTEXITCODE -ne 0) { throw 'Peer compilation failed' }
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $env:CELEBTIR_TEST_PASSWORD_HASH = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($global:PasswordTestValue))).Replace('-','') } finally { $sha.Dispose() }
    $env:PATH = $scratch + ';' + $env:PATH
    Invoke-DeploymentSudo $session @('-tt','example.test') "bash '/tmp/test path/helper.sh'" 'CELEBTIR_PASSWORD_READY_offline'
    Write-Output 'PASS real relay child process, split readiness marker, command quoting and exact UTF-8 password transfer'
    $command = 'stty -echo; CELEBTIR_PASSWORD_READY_followup; exec </dev/tty; exec "$@"'
    $encoded = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($command))
    $env:CELEBTIR_TEST_FOLLOWUP = 'yes'
    $result = Run-Helper ('--run CELEBTIR_PASSWORD_READY_followup ' + $encoded + ' "' + $peer + '"') "follow-up input`n"
    if ($result.Code -ne 0 -or $result.Output -notmatch 'PASS credential frame' -or $result.Error -notmatch 'PASS stderr') { throw 'Follow-up input/output relay failed' }
    if ($result.Output.Contains($global:PasswordTestValue) -or $result.Error.Contains($global:PasswordTestValue)) { throw 'Relay exposed password' }
    $env:CELEBTIR_TEST_PEER_FAILURE = 'yes'
    $result = Run-Helper ('--run CELEBTIR_PASSWORD_READY_followup ' + $encoded + ' "' + $peer + '"')
    if ($result.Code -ne 42) { throw 'Remote failure status lost' }
    Write-Output 'PASS subsequent stdin, stdout/stderr forwarding, no password echo and failed-child status'
    Stop-DeploymentPassword $session
    if ((Test-Path $session.Secret) -or (Test-Path $session.Helper)) { throw 'Credential/helper not removed' }
    if ($env:SSH_ASKPASS -ne 'previous-askpass' -or $env:SSH_ASKPASS_REQUIRE -ne 'never' -or $env:CELEBTIR_PASSWORD_FILE -ne 'previous-secret-path') { throw 'Prior environment not restored' }
    $session = $null
    Write-Output 'PASS credential cleanup and environment restoration'
} finally {
    Stop-DeploymentPassword $session
    foreach ($name in $original.Keys) { [Environment]::SetEnvironmentVariable($name, $original[$name], 'Process') }
    Remove-Item Function:Read-Host
    Remove-Variable PasswordTestPrompts,PasswordTestValue -Scope Global
    if (Test-Path (Join-Path $scratch 'ssh.exe')) { Remove-Item -LiteralPath (Join-Path $scratch 'ssh.exe') }
    Remove-Item -LiteralPath $scratch
}
