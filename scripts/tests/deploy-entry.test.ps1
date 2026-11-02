# Local entry-point checks. No SSH invocation is permitted by these tests.
$ErrorActionPreference = 'Stop'
$repo = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$deploy = Join-Path $repo 'deploy.ps1'
foreach ($argsToReject in @(@{RemoteHost='host;touch'}, @{RemoteUser='bad user'}, @{Port=0}, @{Port=65536})) {
    $rejected = $false
    try { & $deploy @argsToReject -Mode Deploy -DryRun } catch { $rejected = $true }
    if (-not $rejected) { throw 'Unsafe parameter accepted' }
}
Write-Output 'PASS invalid SSH parameters rejected'
$rejected = $false
try { & $deploy -Mode Unknown -DryRun } catch { $rejected = $true }
if (-not $rejected) { throw 'Invalid deployment mode accepted' }
& $deploy -Mode Install -DryRun
Write-Output 'PASS install dry run'
foreach ($choice in @('1', '2')) {
    $global:DeploymentTestChoice = $choice
    function Read-Host { return $global:DeploymentTestChoice }
    try { & $deploy -DryRun } finally { Remove-Item Function:Read-Host }
}
Remove-Variable DeploymentTestChoice -Scope Global
Write-Output 'PASS both interactive menu choices'
$scratch = Join-Path $repo 'output/predeployment/temporary path with spaces'
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$oldTemp = $env:TEMP
$oldTmp = $env:TMP
try {
    $env:TEMP = $scratch
    $env:TMP = $scratch
    # Resolve-Path and tar arguments must preserve spaces in temporary paths.
    & $deploy -Mode Install -PackageOnly
    $installMembers = Get-Content (Join-Path $repo 'output/predeployment/package-files.txt')
    if ($installMembers -match '^output/' -or 'scripts/deploy/bootstrap.sh' -notin $installMembers) { throw 'Install package must not seed QA data' }
    if (@(Get-ChildItem -LiteralPath $scratch -Force).Count) { throw 'Package left temporary files' }
    Write-Output 'PASS real archive with spaces and successful cleanup'
    & $deploy -Mode Deploy -PackageOnly
    $members = Get-Content (Join-Path $repo 'output/predeployment/package-files.txt')
    if ($members -match '^output/' -or 'scripts/deploy/update.sh' -notin $members) { throw 'Invalid update package' }
    Write-Output 'PASS update package contains updater and no seed/output datasets'
    $global:CelebtirUpdateTestHelperSeen = $false
    $global:CelebtirPasswordPromptCount = 0
    $peerDir = Join-Path $repo ('output/deployment-password/entry-' + [guid]::NewGuid().ToString('N'))
    $null = New-Item -ItemType Directory -Path $peerDir
    $peer = Join-Path $peerDir 'ssh.exe'
    & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File (Join-Path $repo 'scripts/deploy/build-password-helper.ps1') -Source (Join-Path $PSScriptRoot 'password-peer.cs') -Output $peer
    if ($LASTEXITCODE -ne 0) { throw 'Offline SSH peer compilation failed' }
    $previousPath = $env:PATH
    $previousHash = $env:CELEBTIR_TEST_PASSWORD_HASH
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $env:CELEBTIR_TEST_PASSWORD_HASH = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes('synthetic entry-test password'))).Replace('-','') } finally { $sha.Dispose() }
    $env:PATH = $peerDir + ';' + $env:PATH
    function Read-Host {
        param([string]$Prompt, [switch]$AsSecureString)
        if (-not $AsSecureString) { throw 'Unexpected prompt' }
        $global:CelebtirPasswordPromptCount++
        ConvertTo-SecureString 'synthetic entry-test password' -AsPlainText -Force
    }
    function ssh.exe { $global:LASTEXITCODE = 0 }
    function scp.exe {
        if ($args -match 'update\.sh$') { $global:CelebtirUpdateTestHelperSeen = $true }
        $global:LASTEXITCODE = 0
    }
    try {
        & $deploy -Mode Deploy
        if ($global:CelebtirPasswordPromptCount -ne 1) { throw 'Deploy must ask for the password exactly once' }
        if (-not $global:CelebtirUpdateTestHelperSeen) { throw 'Deploy did not upload complete deployment helper' }
        if (@(Get-ChildItem -LiteralPath $scratch -Force).Count) { throw 'Update left temporary files' }
        Write-Output 'PASS Deploy asks once and selects correct helper (SSH/SCP doubled; no connection)'
        & $deploy -Mode Install -CertificateEmail 'offline@example.test'
        if ($global:CelebtirPasswordPromptCount -ne 2) { throw 'Install must ask for the password exactly once' }
        if (@(Get-ChildItem -LiteralPath $scratch -Force).Count) { throw 'Install left temporary credentials/files' }
        Write-Output 'PASS Install asks once and cleans all temporary credentials (SSH/SCP doubled)'
        function ssh.exe { $global:LASTEXITCODE = 255 }
        $rejected = $false
        try { & $deploy -Mode Deploy } catch {
            if ($_.Exception.Message -notmatch 'exit 255') { throw }
            $rejected = $true
        }
        if (-not $rejected -or @(Get-ChildItem -LiteralPath $scratch -Force).Count) { throw 'Failed authentication did not stop and clean up' }
        Write-Output 'PASS SSH authentication failure stops before packaging and cleans credentials'
    } finally {
        Remove-Item Function:ssh.exe; Remove-Item Function:scp.exe; Remove-Item Function:Read-Host
        Remove-Variable CelebtirUpdateTestHelperSeen,CelebtirPasswordPromptCount -Scope Global
        $env:PATH = $previousPath; $env:CELEBTIR_TEST_PASSWORD_HASH = $previousHash
        Remove-Item -LiteralPath $peer; Remove-Item -LiteralPath $peerDir
    }
    function tar.exe { $global:LASTEXITCODE = 42 }
    try {
        $rejected = $false
        try { & $deploy -Mode Install -PackageOnly } catch {
            if ($_.Exception.Message -notmatch 'exit 42') { throw }
            $rejected = $true
        }
        if (-not $rejected) { throw 'Native failure was ignored' }
        if (@(Get-ChildItem -LiteralPath $scratch -Force).Count) { throw 'Failure left temporary files' }
        Write-Output 'PASS native-command failure stops and removes temporary directory'
    } finally { Remove-Item Function:tar.exe }
} finally {
    $env:TEMP = $oldTemp
    $env:TMP = $oldTmp
    # Non-recursive: fails if anything unexpected remains.
    Remove-Item -LiteralPath $scratch
}
