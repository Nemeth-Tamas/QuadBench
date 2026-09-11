param(
    [string]$BetaflightDir = "~/betaflight"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

function Test-TcpPort {
    param(
        [string]$HostName,
        [int]$Port,
        [int]$TimeoutMs = 300
    )

    $client = [System.Net.Sockets.TcpClient]::new()

    try {
        $task = $client.ConnectAsync($HostName, $Port)

        if (-not $task.Wait($TimeoutMs)) {
            return $false
        }

        return $client.Connected
    }
    catch {
        return $false
    }
    finally {
        $client.Dispose()
    }
}

Write-Host ""
Write-Host "=== QuadBench Dev Launcher ==="
Write-Host ""

$wslIpRaw = wsl.exe hostname -I

if ($LASTEXITCODE -ne 0) {
    throw "Failed to query WSL IP."
}

$wslIp = ($wslIpRaw.Trim() -split "\s+")[0]

$windowsHost = (
    wsl.exe -e bash -lc "ip route show default | sed -n 's/^default via \([^ ]*\).*/\1/p' | head -n 1"
).Trim()

if (-not $wslIp) {
    throw "Could not determine WSL IP."
}

if (-not $windowsHost) {
    throw "Could not determine Windows host IP from WSL."
}

Write-Host "WSL IP:      $wslIp"
Write-Host "Windows IP:  $windowsHost"
Write-Host ""

$env:QUADBENCH_SITL_HOST = $wslIp

Write-Host "QUADBENCH_SITL_HOST=$env:QUADBENCH_SITL_HOST"
Write-Host ""

$resolvedBetaflightDir = (
    wsl.exe -e bash -lc "cd $BetaflightDir && pwd"
).Trim()

if ($LASTEXITCODE -ne 0 -or -not $resolvedBetaflightDir) {
    throw "Could not resolve Betaflight directory in WSL: $BetaflightDir"
}

$sitlCommand =
    "cd '$resolvedBetaflightDir' && exec ./obj/main/betaflight_SITL.elf --ip '$windowsHost'"

Write-Host "Betaflight:  $resolvedBetaflightDir"
Write-Host ""
Write-Host "Starting Betaflight SITL..."

$startInfo = [System.Diagnostics.ProcessStartInfo]::new()

$startInfo.FileName = "wsl.exe"
$startInfo.UseShellExecute = $false

$startInfo.ArgumentList.Add("-e")
$startInfo.ArgumentList.Add("bash")
$startInfo.ArgumentList.Add("-lc")
$startInfo.ArgumentList.Add($sitlCommand)

$sitlProcess = [System.Diagnostics.Process]::Start(
    $startInfo
)

if ($null -eq $sitlProcess) {
    throw "Failed to start Betaflight SITL process."
}

try {
    Write-Host "Waiting for SITL UART1 on ${wslIp}:5761..."

    $deadline = [DateTime]::UtcNow.AddSeconds(10)

    while ([DateTime]::UtcNow -lt $deadline) {
        if ($sitlProcess.HasExited) {
            throw "Betaflight SITL exited unexpectedly with code $($sitlProcess.ExitCode)."
        }

        if (Test-TcpPort -HostName $wslIp -Port 5761) {
            break
        }

        Start-Sleep -Milliseconds 250
    }

    if (-not (Test-TcpPort -HostName $wslIp -Port 5761)) {
        throw "SITL started, but UART1 TCP port 5761 never became reachable."
    }

    Write-Host "SITL UART1 reachable."
    Write-Host ""
    Write-Host "Starting QuadBench..."
    Write-Host ""

    Set-Location $repoRoot

    cargo run -p quadbench-app
}
finally {
    Write-Host ""
    Write-Host "Stopping Betaflight SITL..."

    if (
        $null -ne $sitlProcess -and
        -not $sitlProcess.HasExited
    ) {
        Stop-Process `
            -Id $sitlProcess.Id `
            -Force `
            -ErrorAction SilentlyContinue
    }
}