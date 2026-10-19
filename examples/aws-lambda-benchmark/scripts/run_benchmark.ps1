<#
.SYNOPSIS
    Convenience wrapper for the AWS Lambda Benchmark runner.
.DESCRIPTION
    Runs the Python benchmark orchestrator with the specified flags.
#>

param(
    [switch]$DryRun,
    [switch]$Deploy,
    [switch]$Run,
    [switch]$Teardown,
    [int]$ColdRuns = 10,
    [int]$WarmRuns = 50
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$PyScript = Join-Path $ScriptDir "run_benchmark.py"

$ArgsList = @()
if ($DryRun) { $ArgsList += "--dry-run" }
if ($Deploy) { $ArgsList += "--deploy" }
if ($Run) { 
    $ArgsList += "--run"
    $ArgsList += "--cold-runs", $ColdRuns
    $ArgsList += "--warm-runs", $WarmRuns
}
if ($Teardown) { $ArgsList += "--teardown" }

if ($ArgsList.Count -eq 0) {
    py -3.9 $PyScript --help
} else {
    py -3.9 $PyScript @ArgsList
}
