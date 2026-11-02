# Local-only, per-invocation credential lifetime. Never included in generated data.
function Start-DeploymentPassword([string]$Directory) {
    $saved = @{}
    foreach ($name in @('SSH_ASKPASS', 'SSH_ASKPASS_REQUIRE', 'CELEBTIR_PASSWORD_FILE')) {
        $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    }
    $session = [pscustomobject]@{ Saved=$saved; Helper=(Join-Path $Directory 'askpass.exe'); Secret=(Join-Path $Directory 'password.dpapi') }
    $password = $null
    try {
        # Children inherit this user-only ACL. The credential itself is also DPAPI encrypted.
        $acl = New-Object System.Security.AccessControl.DirectorySecurity
        $sid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User
        $acl.SetOwner($sid)
        $acl.SetAccessRuleProtection($true, $false)
        $acl.AddAccessRule((New-Object System.Security.AccessControl.FileSystemAccessRule($sid, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow')))
        Set-Acl -LiteralPath $Directory -AclObject $acl
        $password = Read-Host 'SSH / sudo password (used for this deployment only)' -AsSecureString
        if ($password.Length -eq 0) { throw 'A password is required for this deployment.' }
        $password | ConvertFrom-SecureString | Set-Content -LiteralPath $session.Secret -Encoding ASCII
        $windowsPowerShell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
        & $windowsPowerShell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'build-password-helper.ps1') -Source (Join-Path $PSScriptRoot 'password-helper.cs') -Output $session.Helper
        if ($LASTEXITCODE -ne 0) { throw 'Could not build the local password helper.' }
        $env:SSH_ASKPASS = $session.Helper
        $env:SSH_ASKPASS_REQUIRE = 'force'
        $env:CELEBTIR_PASSWORD_FILE = $session.Secret
        return $session
    } catch {
        Stop-DeploymentPassword $session
        throw
    } finally {
        if ($null -ne $password) { $password.Dispose() }
    }
}

function Stop-DeploymentPassword($Session) {
    if ($null -eq $Session) { return }
    foreach ($name in $Session.Saved.Keys) {
        [Environment]::SetEnvironmentVariable($name, $Session.Saved[$name], 'Process')
    }
    foreach ($path in @($Session.Secret, $Session.Helper)) {
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
    }
}

function Invoke-DeploymentSudo($Session, [string[]]$SshArguments, [string]$Command, [string]$Marker) {
    # A readiness marker avoids sending any password while the PTY could echo it.
    # Always consume the frame before invoking sudo: cached/NOPASSWD credentials
    # must not leak the frame into bootstrap's subsequent /dev/tty prompts.
    $remoteCommand = 'tty_state=$(stty -g) && stty -echo || exit 1; trap ''stty "$tty_state"'' EXIT; printf ''%s\n'' ''{0}''; IFS= read -r password || exit 1; stty "$tty_state"; printf ''%s\n'' "$password" | sudo -S -p '''' bash -c ''exec </dev/tty; exec "$@"'' bash {1}; result=$?; unset password; exit $result' -f $Marker, $Command
    $encoded = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($remoteCommand))
    $ssh = (Get-Command ssh.exe -CommandType Application -ErrorAction Stop).Source
    & $Session.Helper '--run' $Marker $encoded $ssh @SshArguments
    if ($LASTEXITCODE -ne 0) { throw "SSH/sudo deployment failed (exit $LASTEXITCODE). Deployment incomplete." }
}
