# Builds the local Arkovia signing dependency without starting a node.
$ErrorActionPreference = 'Stop'
if (-not $env:ARKOVIA_NODE_HOME) { throw 'Set ARKOVIA_NODE_HOME, e.g. $env:ARKOVIA_NODE_HOME = "C:\Arkovia-Blockchain"' }
if (-not (Get-Command git -ErrorAction SilentlyContinue)) { throw 'Install Git for Windows first.' }
if (-not (Get-Command javac -ErrorAction SilentlyContinue)) { throw 'Install a Java JDK (not only a JRE) first.' }

if (-not (Test-Path (Join-Path $env:ARKOVIA_NODE_HOME '.git'))) {
  git clone https://github.com/mycreationhaven/Arkovia-Blockchain.git $env:ARKOVIA_NODE_HOME
}
Set-Location $env:ARKOVIA_NODE_HOME
New-Item -ItemType Directory -Force classes, 'addons/classes' | Out-Null
$sources = Get-ChildItem 'src/java/nxt' -Recurse -Filter '*.java' | ForEach-Object { $_.FullName }
javac -encoding utf8 -sourcepath 'src/java' -classpath 'lib/*;classes;javafx-sdk/lib/*' -d classes $sources
if ($LASTEXITCODE -ne 0) { throw 'Arkovia signing classes did not compile.' }
Write-Host "Offline signer dependency is ready at: $env:ARKOVIA_NODE_HOME"
