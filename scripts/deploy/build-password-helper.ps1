# Windows PowerShell 5.1 can emit a standalone .NET Framework executable.
param([Parameter(Mandatory=$true)][string]$Source, [Parameter(Mandatory=$true)][string]$Output)
$ErrorActionPreference = 'Stop'
Add-Type -LiteralPath $Source -OutputAssembly $Output -OutputType ConsoleApplication -ReferencedAssemblies System.Security.dll,System.Core.dll
