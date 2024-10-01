# Global variables
$script:processes = @()
$script:status = "Running"
$script:logFolder = Join-Path $PSScriptRoot "logs"

# Define the patterns to filter out
$patterns = @(
    "wgpu_hal::auxil::dxgi::exception",
    "id3d12commandqueue::executecommandlists",
    "d3d12_resource_state_render_target",
    "d3d12_resource_state_[common|present]",
    "invalid_subresource_state"
)

# Function to print the menu
function Show-Menu {
    Clear-Host
    Write-Host "2 Game Orchestrator"
    Write-Host "------------------------"
    Write-Host "Current status: $script:status"
    Write-Host "------------------------"
    Write-Host "Press Q to quit"
    Write-Host "Press R to restart the games."
    Write-Host ""
}

# Function to filter content
function Filter-Content {
    param (
        [Parameter(Mandatory=$false)]
        [AllowNull()]
        [AllowEmptyString()]
        [string[]]$content
    )
    if ($null -eq $content -or $content.Count -eq 0) {
        return @()
    }
    return $content | Where-Object {
        $line = $_
        -not ($patterns | Where-Object { $line -match $_ })
    }
}

# Function to start all processes
function Start-AllProcesses {
    if (-not (Test-Path $script:logFolder)) {
        New-Item -ItemType Directory -Path $script:logFolder | Out-Null
    }
    $script:processes = @(
        Start-Process cargo -ArgumentList "run", "--", "--auto-matchmaking", "--inputs-logging" -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $script:logFolder "game1_raw.log") -RedirectStandardError (Join-Path $script:logFolder "game1_error_raw.log")
        Start-Process cargo -ArgumentList "run", "--", "--auto-matchmaking" -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $script:logFolder "game2_raw.log") -RedirectStandardError (Join-Path $script:logFolder "game2_error_raw.log")
    )
}

# Function to stop all processes
function Stop-AllProcesses {
    foreach ($process in $script:processes) {
        if (!$process.HasExited) {
            Stop-Process -Id $process.Id -Force
        }
    }
}

# Function to filter and update log files
function Update-FilteredLogs {
    $logPairs = @(
        @("game1_raw.log", "game1.log"),
        @("game1_error_raw.log", "game1_error.log"),
        @("game2_raw.log", "game2.log"),
        @("game2_error_raw.log", "game2_error.log")
    )

    foreach ($pair in $logPairs) {
        $rawLog = Join-Path $script:logFolder $pair[0]
        $filteredLog = Join-Path $script:logFolder $pair[1]

        if (Test-Path $rawLog) {
            $content = Get-Content $rawLog
            $filteredContent = Filter-Content $content
            $filteredContent | Out-File $filteredLog
        }
    }
}

# Function to clean up raw log files
function Remove-RawLogs {
    $rawLogs = @("game1_raw.log", "game1_error_raw.log", "game2_raw.log", "game2_error_raw.log")
    foreach ($rawLog in $rawLogs) {
        $fullPath = Join-Path $script:logFolder $rawLog
        if (Test-Path $fullPath) {
            Remove-Item $fullPath -Force
        }
    }
}

# Main script
Show-Menu
Start-AllProcesses
$lastFilterTime = Get-Date

while ($true) {
    $currentTime = Get-Date

    # Update filtered logs every second
    if (($currentTime - $lastFilterTime).TotalSeconds -ge 1) {
        Update-FilteredLogs
        $lastFilterTime = $currentTime
    }

    if ($host.UI.RawUI.KeyAvailable) {
        $key = $host.UI.RawUI.ReadKey("NoEcho,IncludeKeyUp")
        switch ($key.Character) {
            'Q' {
                $script:status = "Closing"
                Show-Menu
                Write-Host "Quitting..."
                Stop-AllProcesses
                Remove-RawLogs
                Start-Sleep -Seconds 1
                exit
            }
            'R' {
                $script:status = "Restarting"
                Show-Menu
                Write-Host "Restarting processes..."
                Stop-AllProcesses
                Remove-RawLogs
                Start-Sleep -Seconds 1
                $script:status = "Running"
                Show-Menu
                Start-AllProcesses
            }
        }
    }
    Start-Sleep -Milliseconds 100
}