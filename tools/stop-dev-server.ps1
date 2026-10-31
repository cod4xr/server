# Stop only the old development executable owned by this repository.
# User requirement: reuse the normal target directory; never build around locks.
$ErrorActionPreference = 'Stop'
$repositoryPath = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$debugExecutable = [IO.Path]::GetFullPath((Join-Path $repositoryPath 'target\debug\cod4x-server.exe'))
$releaseExecutable = [IO.Path]::GetFullPath((Join-Path $repositoryPath 'target\release\cod4x-server.exe'))
$candidates = @(Get-Process -Name cod4x-server -ErrorAction SilentlyContinue)
foreach ($candidate in $candidates) {
    if ($candidate.HasExited) {
        continue
    }
    $executablePath = $candidate.Path
    if ($executablePath -ine $debugExecutable -and $executablePath -ine $releaseExecutable) {
        continue
    }
    $developmentProcessId = $candidate.Id
    Stop-Process -InputObject $candidate -Force -ErrorAction Stop
    if (-not $candidate.WaitForExit(5000)) {
        throw "Development server PID $developmentProcessId did not stop."
    }
    Write-Output "Stopped project development server PID $developmentProcessId"
}
