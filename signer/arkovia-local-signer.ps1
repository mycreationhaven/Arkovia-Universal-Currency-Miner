# Secure local adapter for the existing Arkovia SignTransactions utility.
$ErrorActionPreference = 'Stop'
if (-not $env:ARKOVIA_NODE_HOME) { throw 'Set ARKOVIA_NODE_HOME to a local, built Arkovia Blockchain checkout.' }
if (-not (Get-Command java -ErrorAction SilentlyContinue)) { throw 'Java is required.' }

$payload = [Console]::In.ReadToEnd() | ConvertFrom-Json
if ([string]::IsNullOrWhiteSpace($payload.unsignedTransactionBytes)) { throw 'No unsignedTransactionBytes provided.' }
$workDir = Join-Path ([System.IO.Path]::GetTempPath()) ("arkovia-signer-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $workDir | Out-Null
try {
  $unsigned = Join-Path $workDir 'unsigned.txt'
  $signed = Join-Path $workDir 'signed.txt'
  [System.IO.File]::WriteAllText($unsigned, $payload.unsignedTransactionBytes + [Environment]::NewLine, [Text.Encoding]::ASCII)

  # Java receives the phrase only through a local pipe. It is not an environment variable or command argument.
  $secure = Read-Host 'Arkovia secret phrase (local only)' -AsSecureString
  $ptr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
  try { $phrase = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($ptr) }
  finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($ptr) }

  $classPath = "$env:ARKOVIA_NODE_HOME\classes;$env:ARKOVIA_NODE_HOME\lib\*;$env:ARKOVIA_NODE_HOME\conf"
  $psi = [Diagnostics.ProcessStartInfo]::new('java')
  $psi.Arguments = "-cp `"$classPath`" nxt.tools.SignTransactions `"$unsigned`" `"$signed`""
  $psi.UseShellExecute = $false
  $psi.RedirectStandardInput = $true
  $psi.RedirectStandardError = $true
  $process = [Diagnostics.Process]::Start($psi)
  $process.StandardInput.WriteLine($phrase)
  $process.StandardInput.Close()
  $phrase = $null
  $stderr = $process.StandardError.ReadToEnd()
  $process.WaitForExit()
  if ($process.ExitCode -ne 0) { throw "Arkovia signer failed: $stderr" }
  if (-not (Test-Path $signed) -or (Get-Item $signed).Length -eq 0) { throw 'Arkovia signer produced no signed transaction.' }
  [Console]::Out.Write(([System.IO.File]::ReadAllText($signed).Trim()))
}
finally { if (Test-Path $workDir) { Remove-Item -Force -Recurse $workDir } }
